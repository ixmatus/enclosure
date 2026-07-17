//! The second elementary growth round: inverse trigonometric and hyperbolic
//! functions, the base-2 and base-10 exponential and logarithm variants, the
//! two-argument `atan2`, and the integer power `pown`.
//!
//! Ten of the eleven functions are monotone on their domains, so their arms are
//! the `exp`/`ln` shape of [`elementary`](crate::elementary): directed endpoint
//! images with domain restriction per the set-based flavor. `acos` is the lone
//! antitone one; its arm swaps the endpoints. `atan2` is the exception that
//! earns its own rule (a branch cut rather than a period), derived below from
//! the image of the principal angle over a box. `pown` rides bare
//! [`RoundFloat`] as directed integer-power chains, with the negative exponent
//! taken as the set-level reciprocal of the positive one. See workspace
//! decision record 0007.
//!
//! # The atan2 rule
//!
//! `atan2(Y, X)` is the set `{ atan2(y, x) : y in Y, x in X, (x, y) != (0, 0) }`,
//! the principal angle in `(-pi, pi]`. The image is derived from three facts:
//!
//! 1. **No interior critical points.** The gradient of `atan2` is
//!    `(-y, x) / (x^2 + y^2)`, which vanishes only at the origin (outside the
//!    domain), and on each edge of a box one argument is fixed, leaving `atan2`
//!    monotone in the other (its partial has the constant sign of the fixed
//!    argument). So over any box that avoids the branch cut, the image endpoints
//!    sit at box corners, evaluated with directed rounding.
//! 2. **The branch cut is the negative `x` half-line.** `atan2` jumps from just
//!    below `+pi` (approaching from `y > 0`) to just above `-pi` (from `y < 0`).
//!    A box that reaches `x < 0` while spanning `y = 0` downward (`xl < 0`,
//!    `yl < 0`, `0 <= yh`) has both limits in its image, so the hull is the full
//!    `[-pi, pi]`; the pi enclosure widens it outward, never guessing.
//! 3. **The origin is undefined.** A box that is exactly the origin has the
//!    empty image; a box containing the origin has its decoration drop to `trv`
//!    (the value hull is still taken over the domain minus the point).
//!
//! Outside the full-range case the corner reduction runs over the (up to four)
//! corners that are not the origin, with a zero coordinate normalized to `+0`
//! so `atan2(+0, x < 0) = +pi` holds; excluding an origin corner is sound
//! because the values approaching it along either incident edge are bounded by
//! that edge's far corner (the edge is monotone).

use crate::interval::Interval;
use round_float::{RoundExpBases, RoundFloat, RoundInverseHyperbolic, RoundInverseTrig};

// --- Inverse trigonometric: asin, acos, atan, atan2 ------------------------

impl<F: RoundFloat + RoundInverseTrig> Interval<F> {
    /// The arcsine, the set `{ asin(x) : x in self, x in [-1, 1] }`.
    ///
    /// Increasing on its domain `[-1, 1]`, with range the `pi/2` half-bracket.
    /// The set-based flavor restricts the input to `[-1, 1]` first: an interval
    /// wholly outside has the empty image, one reaching past an endpoint is
    /// clamped to it.
    #[must_use]
    pub fn asin(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let one = F::ONE;
        let neg_one = one.negate();
        if b < neg_one || a > one {
            return Self::empty();
        }
        let ra = if a < neg_one { neg_one } else { a };
        let rb = if b > one { one } else { b };
        Self::hull(ra.asin_down(), rb.asin_up())
    }

    /// The arccosine, the set `{ acos(x) : x in self, x in [-1, 1] }`.
    ///
    /// Decreasing on `[-1, 1]` with range `[0, pi]`, so the arm swaps the
    /// endpoints (the image is `[acos(sup), acos(inf)]`); everything else is the
    /// `asin` shape.
    #[must_use]
    pub fn acos(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let one = F::ONE;
        let neg_one = one.negate();
        if b < neg_one || a > one {
            return Self::empty();
        }
        let ra = if a < neg_one { neg_one } else { a };
        let rb = if b > one { one } else { b };
        // Antitone: the lower endpoint comes from the larger argument.
        Self::hull(rb.acos_down(), ra.acos_up())
    }

    /// The arctangent, the set `{ atan(x) : x in self }`.
    ///
    /// Increasing and total on the whole line, with range the open
    /// `(-pi/2, pi/2)`; an unbounded endpoint maps to the range limit it
    /// approaches. The backend's `atan` already maps an infinite argument to the
    /// clamped `+-pi/2` bracket, so no endpoint special-casing is needed.
    #[must_use]
    pub fn atan(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(a.atan_down(), b.atan_up())
    }

    /// The two-argument arctangent, the set
    /// `{ atan2(y, x) : y in self, x in other, (x, y) != (0, 0) }`, the principal
    /// angle of `(x, y)` with `self` the ordinate `y` (matching `f64::atan2`).
    ///
    /// See the [module docs](crate::inverse) for the derivation. The image is
    /// the full `[-pi, pi]` bracket when the box crosses the branch cut, the
    /// empty set when the box is exactly the origin, and otherwise the directed
    /// corner reduction over the box minus any origin corner.
    #[must_use]
    pub fn atan2(self, other: Self) -> Self {
        let Some((yl, yh)) = self.bounds() else {
            return Self::empty();
        };
        let Some((xl, xh)) = other.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        // The box is exactly the origin: no defined point, empty image.
        if yl.is_zero() && yh.is_zero() && xl.is_zero() && xh.is_zero() {
            return Self::empty();
        }
        let pi_up = <F as RoundInverseTrig>::pi_up();
        let neg_pi_up = pi_up.negate();
        // Branch-cut crossing: x reaches below zero while y spans zero downward,
        // so both the +pi and -pi limits enter. The hull is the full range.
        if xl < zero && yl < zero && yh >= zero {
            return Self::hull(neg_pi_up, pi_up);
        }
        // Directed corner reduction over the non-origin corners, zeros normalized
        // to +0 for the +pi convention on the negative x-axis.
        let mut lo = F::INFINITY;
        let mut hi = F::NEG_INFINITY;
        for &cy in &[yl, yh] {
            for &cx in &[xl, xh] {
                if cy.is_zero() && cx.is_zero() {
                    continue;
                }
                let ny = if cy.is_zero() { zero } else { cy };
                let nx = if cx.is_zero() { zero } else { cx };
                lo = lo.rmin(ny.atan2_down(nx));
                hi = hi.rmax(ny.atan2_up(nx));
            }
        }
        // Range clamp into [-pi_up, pi_up] (the image lies in (-pi, pi]).
        Self::hull(lo.rmax(neg_pi_up), hi.rmin(pi_up))
    }
}

// --- Inverse hyperbolic: asinh, acosh, atanh -------------------------------

impl<F: RoundFloat + RoundInverseHyperbolic> Interval<F> {
    /// The inverse hyperbolic sine, the set `{ asinh(x) : x in self }`.
    ///
    /// Increasing and total on the whole line, so the image of `[a, b]` is
    /// `[asinh(a), asinh(b)]`; an infinite endpoint maps to the matching
    /// infinity.
    #[must_use]
    pub fn asinh(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let lo = if a.is_infinite() {
            F::NEG_INFINITY
        } else {
            a.asinh_down()
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.asinh_up()
        };
        Self::hull(lo, hi)
    }

    /// The inverse hyperbolic cosine, the set
    /// `{ acosh(x) : x in self, x >= 1 }`.
    ///
    /// Increasing on its domain `[1, +inf)` with range `[0, +inf)`. The input is
    /// restricted to `[1, +inf)`: an interval wholly below 1 has the empty
    /// image, one reaching below 1 is clamped up to 1 (where `acosh(1) = 0`).
    #[must_use]
    pub fn acosh(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let one = F::ONE;
        if b < one {
            return Self::empty();
        }
        let ra = if a < one { one } else { a };
        let lo = ra.acosh_down();
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.acosh_up()
        };
        Self::hull(lo, hi)
    }

    /// The inverse hyperbolic tangent, the set
    /// `{ atanh(x) : x in self, x in (-1, 1) }`.
    ///
    /// Increasing on the open domain `(-1, 1)`, diverging to `-+inf` at the
    /// endpoints. An interval wholly at or outside `+-1` has the empty image;
    /// one reaching an endpoint has an unbounded image on that side.
    #[must_use]
    pub fn atanh(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let one = F::ONE;
        let neg_one = one.negate();
        if b <= neg_one || a >= one {
            return Self::empty();
        }
        let lo = if a <= neg_one {
            F::NEG_INFINITY
        } else {
            a.atanh_down()
        };
        let hi = if b >= one { F::INFINITY } else { b.atanh_up() };
        Self::hull(lo, hi)
    }
}

// --- Base-2 and base-10 exponential and logarithm --------------------------

impl<F: RoundFloat + RoundExpBases> Interval<F> {
    /// The base-2 exponential, the set `{ 2^x : x in self }`.
    ///
    /// Increasing and total, the `exp` shape: the lower endpoint is clamped to
    /// zero (the true image is positive), an unbounded-below input reaches down
    /// to zero, and an unbounded-above input or an overflow maps to `+inf`.
    #[must_use]
    pub fn exp2(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let lo = if a.is_infinite() {
            F::ZERO
        } else {
            a.exp2_down().rmax(F::ZERO)
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.exp2_up()
        };
        Self::hull(lo, hi)
    }

    /// The base-10 exponential, the set `{ 10^x : x in self }`. The `exp2`
    /// shape with base 10.
    #[must_use]
    pub fn exp10(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let lo = if a.is_infinite() {
            F::ZERO
        } else {
            a.exp10_down().rmax(F::ZERO)
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.exp10_up()
        };
        Self::hull(lo, hi)
    }

    /// The base-2 logarithm, the set `{ log2(x) : x in self, x > 0 }`.
    ///
    /// Increasing on its domain `(0, +inf)`, the `ln` shape: an interval wholly
    /// at or below zero has the empty image, one reaching down to zero is
    /// unbounded below.
    #[must_use]
    pub fn log2(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if b <= zero {
            return Self::empty();
        }
        let lo = if a <= zero {
            F::NEG_INFINITY
        } else {
            a.log2_down()
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.log2_up()
        };
        Self::hull(lo, hi)
    }

    /// The base-10 logarithm, the set `{ log10(x) : x in self, x > 0 }`. The
    /// `log2` shape with base 10.
    #[must_use]
    pub fn log10(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if b <= zero {
            return Self::empty();
        }
        let lo = if a <= zero {
            F::NEG_INFINITY
        } else {
            a.log10_down()
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.log10_up()
        };
        Self::hull(lo, hi)
    }
}

// --- Integer power: pown ---------------------------------------------------

/// `|x|` for a finite `x`, by a sign flip (covers `-0`).
#[inline]
fn fabs<F: RoundFloat>(x: F) -> F {
    if x.is_sign_negative() {
        x.negate()
    } else {
        x
    }
}

/// A lower bound on `m^k` for finite `m >= 0` and `k >= 1`, by directed
/// binary exponentiation (every product rounds down, and every operand stays
/// nonnegative, so the chain never rounds the wrong way).
fn abs_pow_down<F: RoundFloat>(m: F, k: u32) -> F {
    if m.is_zero() {
        return F::ZERO;
    }
    if k == 1 {
        return m;
    }
    let mut result = F::ONE;
    let mut base = m;
    let mut e = k;
    while e > 0 {
        if e & 1 == 1 {
            result = result.mul_down(base);
        }
        e >>= 1;
        if e > 0 {
            base = base.mul_down(base);
        }
    }
    result
}

/// An upper bound on `m^k` for finite `m >= 0` and `k >= 1`, the companion of
/// [`abs_pow_down`] rounding every product up.
fn abs_pow_up<F: RoundFloat>(m: F, k: u32) -> F {
    if m.is_zero() {
        return F::ZERO;
    }
    if k == 1 {
        return m;
    }
    let mut result = F::ONE;
    let mut base = m;
    let mut e = k;
    while e > 0 {
        if e & 1 == 1 {
            result = result.mul_up(base);
        }
        e >>= 1;
        if e > 0 {
            base = base.mul_up(base);
        }
    }
    result
}

/// A lower bound on `a^k` for an odd `k >= 1` (monotone increasing in `a`),
/// carrying the sign: `a^k = -|a|^k` for `a < 0`, so a lower bound negates an
/// upper bound on `|a|^k`.
fn odd_pow_down<F: RoundFloat>(a: F, k: u32) -> F {
    let zero = F::ZERO;
    if a.is_infinite() {
        // a is the lower endpoint, so only -inf is possible; (-inf)^odd = -inf.
        return a;
    }
    if a.is_zero() {
        return zero;
    }
    if a > zero {
        abs_pow_down(a, k)
    } else {
        abs_pow_up(fabs(a), k).negate()
    }
}

/// An upper bound on `b^k` for an odd `k >= 1`, the companion of
/// [`odd_pow_down`].
fn odd_pow_up<F: RoundFloat>(b: F, k: u32) -> F {
    let zero = F::ZERO;
    if b.is_infinite() {
        // b is the upper endpoint, so only +inf is possible; (+inf)^odd = +inf.
        return b;
    }
    if b.is_zero() {
        return zero;
    }
    if b > zero {
        abs_pow_up(b, k)
    } else {
        abs_pow_down(fabs(b), k).negate()
    }
}

/// The image of `x^k` over `[a, b]` for a positive exponent `k >= 1`.
fn pos_pown_image<F: RoundFloat>(a: F, b: F, k: u32) -> Interval<F> {
    let zero = F::ZERO;
    if k % 2 == 0 {
        // Even: x^k = |x|^k, driven by the magnitude range over [a, b].
        let mag_a = if a.is_infinite() {
            F::INFINITY
        } else {
            fabs(a)
        };
        let mag_b = if b.is_infinite() {
            F::INFINITY
        } else {
            fabs(b)
        };
        let mh = mag_a.rmax(mag_b);
        let hi = if mh.is_infinite() {
            F::INFINITY
        } else {
            abs_pow_up(mh, k)
        };
        let lo = if a <= zero && zero <= b {
            // Zero is attained, and it is the minimum of an even power.
            zero
        } else {
            let ml = mag_a.rmin(mag_b);
            if ml.is_infinite() {
                F::INFINITY
            } else {
                abs_pow_down(ml, k)
            }
        };
        Interval::hull(lo.rmax(zero), hi)
    } else {
        // Odd: monotone increasing, so the image is the endpoint pair.
        Interval::hull(odd_pow_down(a, k), odd_pow_up(b, k))
    }
}

impl<F: RoundFloat> Interval<F> {
    /// The integer power, the set `{ x^n : x in self }` (with `x != 0` when
    /// `n < 0`), for a fixed exponent `n`.
    ///
    /// `n == 0` gives the constant `[1, 1]` on any nonempty input (including the
    /// point zero and the unbounded intervals: `x^0 = 1` everywhere the standard
    /// defines it). For `n > 0` the image follows the parity of `n`: an odd power
    /// is monotone, an even power folds through the magnitude range with the
    /// minimum at zero when zero is in the input. For `n < 0` the result is the
    /// set-level reciprocal of the positive-exponent image, so a base straddling
    /// zero yields the unbounded results [`recip`](Interval::recip) produces
    /// (`[c, 0]` gives `[-inf, 1/c]`, a zero-straddle gives `Entire`, the point
    /// zero gives the empty set).
    #[must_use]
    pub fn pown(self, n: i32) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        if n == 0 {
            return Self::hull(F::ONE, F::ONE);
        }
        if n > 0 {
            #[allow(clippy::cast_sign_loss)]
            pos_pown_image(a, b, n as u32)
        } else {
            // n.unsigned_abs() handles i32::MIN without overflow.
            let m = n.unsigned_abs();
            pos_pown_image(a, b, m).recip()
        }
    }
}

// --- Decorated wrappers -----------------------------------------------------
//
// Each mirrors the local-decoration logic of the `trig`/`decorated` modules,
// built through the public `DecoratedInterval` surface: `com` when the
// operation is defined and continuous on a bounded nonempty input with a
// bounded result, `dac` when defined and continuous but somewhere unbounded,
// `trv` on a domain violation, and (for `atan2` alone) `def` when a
// discontinuity is crossed without meeting the undefined origin.

use crate::decorated::DecoratedInterval;
use crate::decoration::Decoration;

/// Whether an interval is bounded and nonempty (the precondition for `com`).
#[inline]
fn bounded_nonempty<F: RoundFloat>(i: Interval<F>) -> bool {
    i.is_bounded() && !i.is_empty()
}

/// The local decoration from definedness/continuity and boundedness (the three
/// grades `trv`/`com`/`dac`; only `atan2` reaches `def`).
#[inline]
fn local(defined_continuous: bool, all_bounded: bool) -> Decoration {
    if !defined_continuous {
        Decoration::Trv
    } else if all_bounded {
        Decoration::Com
    } else {
        Decoration::Dac
    }
}

impl<F: RoundFloat + RoundInverseTrig> DecoratedInterval<F> {
    /// Arcsine, decorated. Defined and continuous on `[-1, 1]`, so it grades
    /// `trv` when the input reaches outside that domain.
    #[must_use]
    pub fn asin(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.asin();
        let one = F::ONE;
        let dom = !x.is_empty() && x.inf() >= one.negate() && x.sup() <= one;
        let dloc = local(dom, bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Arccosine, decorated. The same `[-1, 1]` domain as [`asin`](DecoratedInterval::asin).
    #[must_use]
    pub fn acos(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.acos();
        let one = F::ONE;
        let dom = !x.is_empty() && x.inf() >= one.negate() && x.sup() <= one;
        let dloc = local(dom, bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Arctangent, decorated. Defined and continuous everywhere, so it demotes
    /// only for an unbounded input.
    #[must_use]
    pub fn atan(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.atan();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Two-argument arctangent, decorated. The local decoration follows the
    /// geometry of the box relative to the origin and the branch cut: `trv` when
    /// the box contains the undefined origin, `def` when it crosses the branch
    /// cut (the negative `x`-axis) without meeting the origin, `com`/`dac`
    /// otherwise (the image is always bounded, so only an unbounded input
    /// demotes `com`).
    #[must_use]
    pub fn atan2(self, other: Self) -> Self {
        if self.is_nai() || other.is_nai() {
            return Self::nai();
        }
        let y = self.interval();
        let x = other.interval();
        let result = y.atan2(x);
        let dloc = atan2_local(y, x);
        Self::set_dec(
            result,
            self.decoration().meet(other.decoration()).meet(dloc),
        )
    }
}

/// The local decoration of `atan2` over the ordinate box `y` and abscissa box
/// `x`, per the module derivation.
fn atan2_local<F: RoundFloat>(y: Interval<F>, x: Interval<F>) -> Decoration {
    let (Some((yl, yh)), Some((xl, xh))) = (y.bounds(), x.bounds()) else {
        // An empty operand: nothing is guaranteed.
        return Decoration::Trv;
    };
    let zero = F::ZERO;
    let x_has_zero = xl <= zero && zero <= xh;
    let y_has_zero = yl <= zero && zero <= yh;
    if x_has_zero && y_has_zero {
        // The origin is in the box: atan2 is undefined there.
        return Decoration::Trv;
    }
    // The box touches the negative x-axis (a point (x < 0, y = 0)); since the
    // origin is not in the box, x is then strictly negative there.
    let touches_cut = xl < zero && y_has_zero;
    if touches_cut {
        if yl < zero {
            // y reaches below zero, so +pi (at y = 0) and the -pi limit both
            // enter: the restriction is discontinuous.
            Decoration::Def
        } else {
            // yl == 0: the box only touches the cut from above, so the
            // restriction is continuous, but a point of the box sits on the
            // discontinuity locus, which forbids `com`.
            Decoration::Dac
        }
    } else {
        // atan2 is continuous at every point of the box, and its image is always
        // bounded, so `com` when both inputs are bounded, `dac` otherwise.
        local(true, bounded_nonempty(y) && bounded_nonempty(x))
    }
}

impl<F: RoundFloat + RoundInverseHyperbolic> DecoratedInterval<F> {
    /// Inverse hyperbolic sine, decorated. Defined and continuous everywhere.
    #[must_use]
    pub fn asinh(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.asinh();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Inverse hyperbolic cosine, decorated. Defined and continuous on
    /// `[1, +inf)`, so it grades `trv` when the input reaches below 1.
    #[must_use]
    pub fn acosh(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.acosh();
        let dom = !x.is_empty() && x.inf() >= F::ONE;
        let dloc = local(dom, bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Inverse hyperbolic tangent, decorated. Defined and continuous on the open
    /// `(-1, 1)`, so it grades `trv` when the input reaches at or outside `+-1`.
    #[must_use]
    pub fn atanh(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.atanh();
        let one = F::ONE;
        let dom = !x.is_empty() && x.inf() > one.negate() && x.sup() < one;
        let dloc = local(dom, bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }
}

impl<F: RoundFloat + RoundExpBases> DecoratedInterval<F> {
    /// Base-2 exponential, decorated. Defined and continuous everywhere; an
    /// overflowed (unbounded) result demotes `com` to `dac`.
    #[must_use]
    pub fn exp2(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.exp2();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Base-10 exponential, decorated. The [`exp2`](DecoratedInterval::exp2)
    /// pattern.
    #[must_use]
    pub fn exp10(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.exp10();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Base-2 logarithm, decorated. Defined and continuous on `(0, +inf)`, so it
    /// grades `trv` when the input reaches zero or below; an input touching zero
    /// has an unbounded image, which also demotes `com` to `dac`.
    #[must_use]
    pub fn log2(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.log2();
        let dom = !x.is_empty() && x.inf() > F::ZERO;
        let dloc = local(dom, bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Base-10 logarithm, decorated. The [`log2`](DecoratedInterval::log2)
    /// pattern.
    #[must_use]
    pub fn log10(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.log10();
        let dom = !x.is_empty() && x.inf() > F::ZERO;
        let dloc = local(dom, bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }
}

impl<F: RoundFloat> DecoratedInterval<F> {
    /// Integer power, decorated. A nonnegative exponent is defined and
    /// continuous everywhere (a polynomial), so it only demotes for an unbounded
    /// input or an overflowed result; a negative exponent is undefined at zero,
    /// so it grades `trv` when the base contains zero.
    #[must_use]
    pub fn pown(self, n: i32) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.pown(n);
        let zero = F::ZERO;
        let dom = if n >= 0 {
            !x.is_empty()
        } else {
            !x.is_empty() && !x.contains(zero)
        };
        let dloc = local(dom, bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }
}
