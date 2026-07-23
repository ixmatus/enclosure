//! Forward arithmetic over intervals: the set-based flavor's total operations.
//!
//! The four basic operations are the `core::ops` operators (`+`, `-`, `*`, `/`)
//! and negation (`-`); the rest (`recip`, `sqr`, `sqrt`, `mul_add`) are inherent
//! methods. Every operation is total (the set-based flavor requires it) and sound
//! (its result encloses the pointwise result over the inputs,
//! [`spec`](crate::spec) Law 2). The empty interval is absorbing: any operation
//! with an empty operand is empty. Each endpoint is rounded outward (Law 3), so a
//! correctly-rounded backend yields a tight enclosure.
//!
//! # Non-monotone operations use the four corners
//!
//! Multiplication and (through reciprocal) division are not monotone across a
//! sign change, so the extreme of the result need not sit at a fixed pair of
//! endpoints. The lower endpoint is the least of all four endpoint products
//! rounded down, the upper the greatest rounded up (Law 4). Picking one corner
//! by guessing the sign pattern is the classic rigor bug; the four-corner
//! reduction is correct by construction.
//!
//! # The indeterminate `0 * inf`
//!
//! A product of an exactly-zero endpoint and an infinite endpoint is NaN in
//! floating point, but the interval value is zero: a factor that can be exactly
//! zero contributes exactly zero, regardless of the other factor's unbounded
//! range. The corner helpers map that NaN to zero.
//!
//! # Division rounds each quotient endpoint once
//!
//! `a / b` is not `a * recip(b)`: that route rounds twice, once forming the
//! reciprocal and once forming the product, and loses a unit in the last place on
//! an inexact quotient (`[-30, -15] / [-5, -3]` is exactly `[3, 10]`, but the
//! reciprocal route widens it to `[2.999..., 10.000...]`). Instead each endpoint
//! selects the one corner quotient `x / y` that attains the bound and rounds it
//! once with the backend's directed division (Law 3), so a correctly-rounded
//! backend yields the tightest enclosure. When `0` lies outside the divisor the
//! quotient is monotone and the sign-cased core `odiv` picks the corners; the
//! reverse-division path shares that core so the two cannot drift. When `0` lies
//! inside the divisor the quotient set is a half-line, the whole line, a single
//! point, or (for an exactly-zero divisor) empty; those endpoints are exact zeros
//! and infinities, so no rounding arises and no reciprocal is needed.

use core::ops::{Add, Div, Mul, Neg, Sub};

use crate::interval::Interval;
use round_float::RoundFloat;

/// One corner of a product, rounded down, with the `0 * inf` indeterminate
/// mapped to its interval value of zero.
#[inline]
fn prod_down<F: RoundFloat>(x: F, y: F) -> F {
    let p = x.mul_down(y);
    if p.is_nan() {
        F::ZERO
    } else {
        p
    }
}

/// One corner of a product, rounded up, with `0 * inf` mapped to zero.
#[inline]
fn prod_up<F: RoundFloat>(x: F, y: F) -> F {
    let p = x.mul_up(y);
    if p.is_nan() {
        F::ZERO
    } else {
        p
    }
}

/// Ordinary interval division `[a1, a2] / [b1, b2]` with `0` not in the divisor.
///
/// The quotient is monotone there (increasing in the numerator, decreasing in a
/// positive divisor), so each endpoint is the one sign-selected corner rounded
/// once. The selection never forms `inf / inf`: the lower endpoint pairs the
/// numerator's low end with a divisor endpoint chosen by the numerator's sign,
/// the upper endpoint the numerator's high end, and each pairing keeps one
/// operand finite whenever the true bound is finite. A negative divisor reduces
/// to a positive one by `[a] / [b] = -([a] / [-b])`.
///
/// Shared between the forward `Div` operator and the reverse-division path so the
/// two quotients cannot drift.
pub(crate) fn odiv<F: RoundFloat>(a1: F, a2: F, b1: F, b2: F) -> Interval<F> {
    let zero = F::ZERO;
    if b1 > zero {
        // Divisor entirely positive.
        let lo = if a1 >= zero {
            a1.div_down(b2)
        } else {
            a1.div_down(b1)
        };
        let hi = if a2 >= zero {
            a2.div_up(b1)
        } else {
            a2.div_up(b2)
        };
        Interval::hull(lo, hi)
    } else {
        // Divisor entirely negative: negate it to reach the positive case.
        -odiv(a1, a2, b2.negate(), b1.negate())
    }
}

impl<F: RoundFloat> Interval<F> {
    /// The four-corner product. Private so division can reuse it without writing
    /// a multiplication operator inside a `Div` impl.
    fn mul_impl(self, rhs: Self) -> Self {
        let (Some((a, b)), Some((c, d))) = (self.bounds(), rhs.bounds()) else {
            return Self::empty();
        };
        let lo = prod_down(a, c)
            .rmin(prod_down(a, d))
            .rmin(prod_down(b, c))
            .rmin(prod_down(b, d));
        let hi = prod_up(a, c)
            .rmax(prod_up(a, d))
            .rmax(prod_up(b, c))
            .rmax(prod_up(b, d));
        Self::hull(lo, hi)
    }

    /// The sign-cased quotient, one directed rounding per endpoint. Private so the
    /// `Div` operator has a single derivation to call.
    ///
    /// The divisor's relation to zero drives the shape. `0` outside the divisor is
    /// the monotone case handled by `odiv`. `0` inside splits by which side the
    /// divisor touches zero and by the numerator's sign: the quotient set is then
    /// a half-line to an infinity (finite end rounded once), the whole line, the
    /// single point `[0, 0]` (an exactly-zero numerator), or empty (an
    /// exactly-zero divisor). Those non-monotone endpoints are exact, so only the
    /// half-line's finite end is ever rounded.
    fn div_impl(self, rhs: Self) -> Self {
        let (Some((a1, a2)), Some((b1, b2))) = (self.bounds(), rhs.bounds()) else {
            return Self::empty();
        };
        let zero = F::ZERO;
        // Divisor sign-stable, zero excluded: the monotone corner selection.
        if zero < b1 || b2 < zero {
            return odiv(a1, a2, b1, b2);
        }
        // From here `0` lies in the divisor.
        // Divisor is exactly zero: no nonzero point to divide by.
        if b1 == zero && b2 == zero {
            return Self::empty();
        }
        // Numerator is exactly zero: `0 / y = 0` for every nonzero `y`.
        if a1 == zero && a2 == zero {
            return Self::hull(zero, zero);
        }
        // Divisor straddles zero: divisors of both signs approach zero, so the
        // quotient runs to both infinities.
        if b1 < zero && zero < b2 {
            return Self::entire();
        }
        let inf = F::INFINITY;
        let ninf = F::NEG_INFINITY;
        if b1 == zero {
            // Divisor is `[0, b2]`, `b2 > 0`: every nonzero divisor is positive.
            if a2 < zero {
                Self::hull(ninf, a2.div_up(b2))
            } else if a1 > zero {
                Self::hull(a1.div_down(b2), inf)
            } else if a1 == zero {
                // Numerator `[0, a2]`, `a2 > 0`.
                Self::hull(zero, inf)
            } else if a2 == zero {
                // Numerator `[a1, 0]`, `a1 < 0`.
                Self::hull(ninf, zero)
            } else {
                // Numerator straddles zero.
                Self::entire()
            }
        } else {
            // Divisor is `[b1, 0]`, `b1 < 0`: every nonzero divisor is negative.
            if a2 < zero {
                Self::hull(a2.div_down(b1), inf)
            } else if a1 > zero {
                Self::hull(ninf, a1.div_up(b1))
            } else if a1 == zero {
                // Numerator `[0, a2]`, `a2 > 0`.
                Self::hull(ninf, zero)
            } else if a2 == zero {
                // Numerator `[a1, 0]`, `a1 < 0`.
                Self::hull(zero, inf)
            } else {
                // Numerator straddles zero.
                Self::entire()
            }
        }
    }

    /// Reciprocal, the set `{ 1 / y : y in self, y != 0 }`.
    ///
    /// A divisor that is exactly `[0, 0]` has no nonzero point, so its reciprocal
    /// is the empty set. A divisor straddling zero has an unbounded reciprocal:
    /// `[c, d]` with `c < 0 < d` gives `Entire`, `[0, d]` gives `[1/d, +inf]`,
    /// and `[c, 0]` gives `[-inf, 1/c]`. A sign-stable divisor gives the
    /// monotone `[1/d, 1/c]`.
    #[must_use]
    pub fn recip(self) -> Self {
        let Some((c, d)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if c == zero && d == zero {
            // No nonzero point to invert.
            Self::empty()
        } else if zero < c || d < zero {
            // Sign stable, zero excluded: 1/y is monotone decreasing.
            Self::hull(F::ONE.div_down(d), F::ONE.div_up(c))
        } else if c == zero {
            // [0, d] with d > 0: 1/y ranges over [1/d, +inf).
            Self::hull(F::ONE.div_down(d), F::INFINITY)
        } else if d == zero {
            // [c, 0] with c < 0: 1/y ranges over (-inf, 1/c].
            Self::hull(F::NEG_INFINITY, F::ONE.div_up(c))
        } else {
            // c < 0 < d: zero is interior, the reciprocal is the whole line.
            Self::entire()
        }
    }

    /// Square, the set `{ x * x : x in self }`.
    ///
    /// Not `self * self`: that would treat the two factors as independent and
    /// over-widen (for example `[-1, 1] * [-1, 1]` is `[-1, 1]`, but the square
    /// is `[0, 1]`). The lower endpoint is zero when zero is in the interval,
    /// otherwise the smaller squared endpoint.
    #[must_use]
    pub fn sqr(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        let hi = a.mul_up(a).rmax(b.mul_up(b));
        let lo = if a <= zero && zero <= b {
            zero
        } else {
            a.mul_down(a).rmin(b.mul_down(b))
        };
        Self::hull(lo, hi)
    }

    /// Square root, the set `{ sqrt(x) : x in self, x >= 0 }`.
    ///
    /// Total in the set-based flavor: the domain is restricted to the
    /// nonnegative part. An interval entirely below zero has empty image; an
    /// interval straddling zero contributes `[0, sqrt(sup)]`.
    #[must_use]
    pub fn sqrt(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if b < zero {
            return Self::empty();
        }
        let radicand_lo = if a < zero { zero } else { a };
        // sqrt is nonnegative; clamp the lower bound up to zero so the fixture's
        // outward step does not report a spuriously negative lower endpoint.
        let lo = radicand_lo.sqrt_down().rmax(zero);
        let hi = b.sqrt_up();
        Self::hull(lo, hi)
    }

    /// Multiply then add, `self * factor + addend`.
    ///
    /// A sound enclosure of the fused multiply-add. A tighter, single-rounded
    /// variant using a scalar fused operation is a later refinement.
    #[must_use]
    pub fn mul_add(self, factor: Self, addend: Self) -> Self {
        self.mul_impl(factor) + addend
    }
}

impl<F: RoundFloat> Neg for Interval<F> {
    type Output = Self;

    /// Negation, `-[a, b] = [-b, -a]`. Exact: negation never rounds.
    fn neg(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(b.negate(), a.negate())
    }
}

impl<F: RoundFloat> Add for Interval<F> {
    type Output = Self;

    /// `[a, b] + [c, d] = [a + c, b + d]`, each endpoint rounded outward.
    fn add(self, rhs: Self) -> Self {
        let (Some((a, b)), Some((c, d))) = (self.bounds(), rhs.bounds()) else {
            return Self::empty();
        };
        Self::hull(a.add_down(c), b.add_up(d))
    }
}

impl<F: RoundFloat> Sub for Interval<F> {
    type Output = Self;

    /// `[a, b] - [c, d] = [a - d, b - c]`, each endpoint rounded outward. The
    /// crossing (the lower endpoint subtracts the upper of the subtrahend) is the
    /// subtraction analogue of the four-corner rule.
    fn sub(self, rhs: Self) -> Self {
        let (Some((a, b)), Some((c, d))) = (self.bounds(), rhs.bounds()) else {
            return Self::empty();
        };
        Self::hull(a.sub_down(d), b.sub_up(c))
    }
}

impl<F: RoundFloat> Mul for Interval<F> {
    type Output = Self;

    /// Multiplication, by the four-corner rule with outward rounding.
    fn mul(self, rhs: Self) -> Self {
        self.mul_impl(rhs)
    }
}

impl<F: RoundFloat> Div for Interval<F> {
    type Output = Self;

    /// Division, `self / rhs`, rounding each quotient endpoint once. Total:
    /// division by an interval containing zero yields a half-line, the whole line,
    /// or (for an exactly-zero divisor) the empty set.
    fn div(self, rhs: Self) -> Self {
        self.div_impl(rhs)
    }
}

#[cfg(all(test, feature = "fixture"))]
mod tests {
    use crate::Interval;

    fn iv(lo: f64, hi: f64) -> Interval<f64> {
        Interval::new(lo, hi).unwrap()
    }

    #[test]
    fn add_widens_outward_around_the_exact_sum() {
        let r = iv(1.0, 2.0) + iv(3.0, 4.0);
        // [1+3, 2+4] = [4, 6], possibly one step wider.
        assert!(r.contains(4.0));
        assert!(r.contains(6.0));
        assert!(r.inf() <= 4.0 && r.sup() >= 6.0);
    }

    #[test]
    fn sub_uses_the_crossed_endpoints() {
        let r = iv(1.0, 2.0) - iv(3.0, 5.0);
        // [1-5, 2-3] = [-4, -1]
        assert!(r.contains(-4.0));
        assert!(r.contains(-1.0));
        assert!(!r.contains(0.0));
    }

    #[test]
    fn mul_four_corner_sign_mixing() {
        // [-2, 3] * [-5, 7]: corners -2*-5=10, -2*7=-14, 3*-5=-15, 3*7=21.
        let r = iv(-2.0, 3.0) * iv(-5.0, 7.0);
        assert!(r.contains(-15.0));
        assert!(r.contains(21.0));
        assert!(r.contains(0.0));
        assert!(!r.contains(-16.0));
        assert!(!r.contains(22.0));
    }

    #[test]
    fn mul_zero_times_unbounded_is_zero() {
        // [0, 0] * Entire = [0, 0], despite 0 * inf.
        let r = iv(0.0, 0.0) * Interval::entire();
        assert!(r.is_singleton());
        assert_eq!(r.inf(), 0.0);
        assert_eq!(r.sup(), 0.0);
    }

    #[test]
    fn sqr_is_tighter_than_self_times_self() {
        let r = iv(-1.0, 1.0).sqr();
        assert_eq!(r.inf(), 0.0);
        assert!(r.contains(1.0));
        assert!(!r.contains(-0.5));
    }

    #[test]
    fn recip_of_sign_stable() {
        // 1 / [2, 4] = [0.25, 0.5].
        let r = iv(2.0, 4.0).recip();
        assert!(r.contains(0.25));
        assert!(r.contains(0.5));
        assert!(r.inf() <= 0.25 && r.sup() >= 0.5);
    }

    #[test]
    fn recip_straddling_zero_is_entire() {
        assert!(iv(-1.0, 1.0).recip().is_entire());
    }

    #[test]
    fn recip_touching_zero_is_half_unbounded() {
        let up = iv(0.0, 4.0).recip(); // [1/4, +inf]
        assert!(up.contains(0.25));
        assert!(up.contains(1.0e9));
        assert_eq!(up.sup(), f64::INFINITY);

        let down = iv(-4.0, 0.0).recip(); // [-inf, -1/4]
        assert!(down.contains(-0.25));
        assert!(down.contains(-1.0e9));
        assert_eq!(down.inf(), f64::NEG_INFINITY);
    }

    #[test]
    fn div_by_exact_zero_is_empty() {
        assert!((iv(1.0, 2.0) / iv(0.0, 0.0)).is_empty());
    }

    #[test]
    fn div_sign_stable() {
        // [1, 2] / [2, 4] = [0.25, 1.0].
        let r = iv(1.0, 2.0) / iv(2.0, 4.0);
        assert!(r.contains(0.25));
        assert!(r.contains(1.0));
    }

    #[test]
    fn sqrt_domain_restriction() {
        // sqrt([-2, 4]) = [0, 2].
        let r = iv(-2.0, 4.0).sqrt();
        assert_eq!(r.inf(), 0.0);
        assert!(r.contains(2.0));
        // Entirely negative: empty image.
        assert!(iv(-4.0, -1.0).sqrt().is_empty());
    }

    #[test]
    fn empty_is_absorbing() {
        let e = Interval::<f64>::empty();
        assert!((e + iv(1.0, 2.0)).is_empty());
        assert!((iv(1.0, 2.0) + e).is_empty());
        assert!((iv(1.0, 2.0) * e).is_empty());
        assert!(e.sqrt().is_empty());
        assert!(e.recip().is_empty());
    }

    #[test]
    fn neg_is_exact_and_reflects() {
        let r = -iv(-2.0, 3.0);
        assert_eq!(r.inf(), -3.0);
        assert_eq!(r.sup(), 2.0);
    }

    #[test]
    fn mul_add_encloses() {
        // [2,3] * [4,5] + [1,1] contains 2*4+1=9 and 3*5+1=16.
        let r = iv(2.0, 3.0).mul_add(iv(4.0, 5.0), iv(1.0, 1.0));
        assert!(r.contains(9.0));
        assert!(r.contains(16.0));
    }
}
