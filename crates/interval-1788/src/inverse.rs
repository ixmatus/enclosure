//! The second elementary growth round: inverse trigonometric and hyperbolic
//! functions, the base-2 and base-10 exponential and logarithm variants, the
//! two-argument `atan2`, and the integer power `pown`.
//!
//! Ten of the eleven functions are monotone on their domains, so their arms are
//! the `exp`/`ln` shape of [`elementary`](crate::elementary): directed endpoint
//! images with domain restriction per the set-based flavor. `acos` is the lone
//! antitone one; its arm swaps the endpoints. `atan2` is the exception that
//! earns its own rule (a branch cut rather than a period), derived below from
//! the image of the principal angle over a box. `pown` rides
//! [`RoundPown`], whose exact integer kernel rounds each endpoint power
//! correctly; the negative exponent is taken endpoint-wise from negative kernel
//! powers under a parity and zero-position case analysis, not the set-level
//! reciprocal of the positive image. See workspace decision record 0007 and
//! round-float decision record 0004.
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
use round_float::{RoundExpBases, RoundFloat, RoundInverseHyperbolic, RoundInverseTrig, RoundPown};

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

/// The image of `x^n` over `[a, b]` for a positive exponent `n >= 1`, each
/// endpoint power a correctly rounded [`RoundPown`] call.
///
/// An odd power is monotone increasing, so the image is the endpoint pair. An
/// even power is `|x|^n`, so its maximum sits at the larger-magnitude endpoint
/// and its minimum at zero when zero is in `[a, b]`, else at the smaller-
/// magnitude endpoint. Both endpoint choices reduce to `rmin`/`rmax` over the
/// two endpoint powers: for an even `n`, `pown(a, n)` and `pown(b, n)` are the
/// nonnegative `|a|^n` and `|b|^n`, so their min and max are exactly the image
/// bounds. Infinite endpoints ride the kernel's `pown(+/-inf, n)` directly.
fn pos_pown_image<F: RoundFloat + RoundPown>(a: F, b: F, n: i32) -> Interval<F> {
    if n % 2 != 0 {
        // Odd: monotone increasing.
        Interval::hull(a.pown_down(n), b.pown_up(n))
    } else {
        // Even: |x|^n, folded through the magnitude range.
        let hi = a.pown_up(n).rmax(b.pown_up(n));
        let zero = F::ZERO;
        let lo = if a <= zero && zero <= b {
            // Zero is attained and is the minimum of an even power.
            zero
        } else {
            a.pown_down(n).rmin(b.pown_down(n))
        };
        Interval::hull(lo, hi)
    }
}

/// The image of `x^n` over `[a, b]` for a negative exponent `n <= -1`.
///
/// Write `m = |n|` and `g(x) = x^n = 1/x^m`. The case table is the parity of
/// `m` crossed with the position of zero in `[a, b]`, derived from the set
/// definition. The endpoint powers are correctly rounded negative [`RoundPown`]
/// calls, so an overflowing magnitude power denormalizes to the subnormal floor
/// instead of collapsing (the pre-kernel reciprocal-of-the-image defect).
///
/// **`m` odd.** `g` has the pole structure of `1/x`: strictly decreasing on each
/// of `x < 0` and `x > 0`, jumping from `-inf` to `+inf` across zero.
/// - Zero interior (`a < 0 < b`): the image is all reals, `Entire`.
/// - `a` is zero, `b > 0` (`[0, b]`): `g` runs from `+inf` down to `b^n`, so
///   `[pown_down(b, n), +inf]`.
/// - `b` is zero, `a < 0` (`[a, 0]`): `g` runs from `a^n` down to `-inf`, so
///   `[-inf, pown_up(a, n)]`.
/// - Zero absent: `g` is monotone decreasing, so `[pown_down(b, n),
///   pown_up(a, n)]`.
///
/// **`m` even.** `g(x) = 1/|x|^m > 0`, decreasing in `|x|`: the minimum sits at
/// the larger-magnitude endpoint and the maximum at the smaller, running to
/// `+inf` when zero (magnitude zero) is attained. `pown(0, n)` for a negative
/// even `n` is `+inf`, so the larger-magnitude endpoint wins the `rmin`
/// automatically.
/// - The lower bound is `min(pown_down(a, n), pown_down(b, n))`.
/// - The upper bound is `+inf` when zero is in `[a, b]`, else
///   `max(pown_up(a, n), pown_up(b, n))`.
///
/// The point zero (`[0, 0]`), where `0^n` is undefined for `n < 0`, is caught by
/// the caller as the empty set.
fn neg_pown_image<F: RoundFloat + RoundPown>(a: F, b: F, n: i32) -> Interval<F> {
    let zero = F::ZERO;
    if n % 2 != 0 {
        // Odd magnitude: the 1/x pole structure.
        let zero_interior = a < zero && b > zero;
        if zero_interior {
            return Interval::entire();
        }
        if a.is_zero() {
            // [0, b], b > 0: g runs from +inf down to b^n.
            return Interval::hull(b.pown_down(n), F::INFINITY);
        }
        if b.is_zero() {
            // [a, 0], a < 0: g runs from a^n down to -inf.
            return Interval::hull(F::NEG_INFINITY, a.pown_up(n));
        }
        // Zero absent: monotone decreasing.
        Interval::hull(b.pown_down(n), a.pown_up(n))
    } else {
        // Even magnitude: g = 1/|x|^m > 0, decreasing in |x|.
        let lo = a.pown_down(n).rmin(b.pown_down(n));
        let hi = if a <= zero && zero <= b {
            F::INFINITY
        } else {
            a.pown_up(n).rmax(b.pown_up(n))
        };
        Interval::hull(lo, hi)
    }
}

impl<F: RoundFloat + RoundPown> Interval<F> {
    /// The integer power, the set `{ x^n : x in self }` (with `x != 0` when
    /// `n < 0`), for a fixed exponent `n`.
    ///
    /// `n == 0` gives the constant `[1, 1]` on any nonempty input (including the
    /// point zero and the unbounded intervals: `x^0 = 1` everywhere the standard
    /// defines it). For `n > 0` the image follows the parity of `n`: an odd power
    /// is monotone, an even power folds through the magnitude range with the
    /// minimum at zero when zero is in the input. For `n < 0` the image is taken
    /// endpoint-wise from correctly rounded negative powers under a parity and
    /// zero-position case analysis: a base straddling
    /// zero yields the unbounded results `1/x` produces (`[c, 0]` gives
    /// `[-inf, 1/c]`, a zero-straddle gives `Entire`, the point zero gives the
    /// empty set), while an overflowing magnitude power denormalizes to the
    /// subnormal floor rather than collapsing.
    ///
    /// Each endpoint power is a [`RoundPown`] call, correctly rounded (tightest)
    /// for `|n| <= F::POWN_TIGHT_MAX` and a sound bound beyond.
    #[must_use]
    pub fn pown(self, n: i32) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        if n == 0 {
            return Self::hull(F::ONE, F::ONE);
        }
        if n > 0 {
            pos_pown_image(a, b, n)
        } else if a.is_zero() && b.is_zero() {
            // The point zero: 0^n is undefined for n < 0, so the image is empty.
            Self::empty()
        } else {
            neg_pown_image(a, b, n)
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

impl<F: RoundFloat + RoundPown> DecoratedInterval<F> {
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
