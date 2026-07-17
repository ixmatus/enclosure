//! [`DecoratedInterval`]: an interval paired with a decoration, with operations
//! that propagate the decoration per the fundamental theorem of decorated
//! interval arithmetic.
//!
//! A decorated operation computes the bare interval result, determines the
//! decoration the operation earned on its bare inputs (its *local* decoration),
//! and decorates the result with the meet of that local decoration and the
//! decorations the inputs carried. A `NaI` input poisons the result.
//!
//! For the arithmetic and elementary operations here (`+`, `-`, `*`, `/`, `neg`,
//! `recip`, `sqr`, `sqrt`, `mul_add`, `exp`, `ln`) the local decoration is one of
//! `com`, `dac`, or `trv`. These functions are continuous wherever they are
//! defined, so `def` (defined but discontinuous) does not arise from them. The
//! local decoration is:
//!
//! - `trv` when the operation is undefined somewhere on the input box (division
//!   by an interval containing zero, square root of an interval reaching below
//!   zero) or an input is empty;
//! - `com` when it is defined and continuous, every input is bounded and
//!   nonempty, and the computed result is itself bounded;
//! - `dac` when it is defined and continuous but some input is unbounded, or a
//!   bounded input overflowed to an unbounded result.
//!
//! The last clause of the `com` case is a result obligation the inputs alone
//! cannot discharge: a bounded operation can overflow to an unbounded result
//! (adding `[MAX/2, MAX]` to itself reaches `+inf`), and `com` promises a
//! bounded result. The `pack` seam every operation funnels through demotes
//! `com` to `dac` whenever the assembled interval is unbounded, so no result
//! escapes decorated `com` while unbounded.
//!
//! # The point functions do earn `def`
//!
//! The [point functions](crate::point) `sign`, `ceil`, `floor`, `trunc`,
//! `round_ties_to_even`, and `round_ties_to_away` are defined everywhere but
//! discontinuous at their jump points, so they are the first operations in the
//! crate that produce the `def` decoration from an operation rather than only
//! through [`set_dec`](DecoratedInterval::set_dec). A step function restricted to
//! a connected box is continuous exactly when it is constant there, so the local
//! decoration reads off the image: `def` when the image spans more than one
//! value (the restriction is discontinuous), otherwise `com` (or `dac` when a
//! jump point sits at an endpoint, or the input is unbounded). `abs`, `min`, and
//! `max` are continuous everywhere and behave like the arithmetic operations
//! above. The [`step_local`] helper computes this, and the point functions
//! themselves live in this module beside the operations they decorate.

use core::ops::{Add, Div, Mul, Neg, Sub};

use crate::decoration::Decoration;
use crate::interval::Interval;
use round_float::{RoundFloat, RoundInteger};

/// Whether an interval is bounded and nonempty (so a continuous operation on it
/// yields a bounded nonempty result, the precondition for `com`).
#[inline]
fn bounded_nonempty<F: RoundFloat>(i: Interval<F>) -> bool {
    i.is_bounded() && !i.is_empty()
}

/// The local decoration of an operation given whether it is defined and
/// continuous on the input box, and whether all inputs are bounded and nonempty.
#[inline]
fn local(defined_continuous: bool, all_bounded_nonempty: bool) -> Decoration {
    if !defined_continuous {
        Decoration::Trv
    } else if all_bounded_nonempty {
        Decoration::Com
    } else {
        Decoration::Dac
    }
}

/// The local decoration of a point function that is defined on the whole real
/// line but has jump discontinuities (`sign`, `ceil`, `floor`, `trunc`, and the
/// two roundings), given the input box and the function's image over it, and
/// whether either input endpoint is a jump point of the function.
///
/// A step function restricted to a connected box takes only integer-spaced
/// values, so it is continuous there exactly when it is constant, that is, when
/// the image is a single value. Reading continuity off the image this way handles
/// the one-sided endpoint cases uniformly: an endpoint that sits at a jump on the
/// side the box extends into already makes the image span two values, so it lands
/// in the `def` branch without a per-function limit analysis. When the image is a
/// single value the restriction is continuous, and the grade is `com` unless a
/// jump point coincides with an endpoint (so the closed box still contains a
/// discontinuity, forbidding `com`) or the input is unbounded (`com` promises a
/// bounded input): either demotes to `dac`.
#[inline]
fn step_local<F: RoundFloat>(
    input: Interval<F>,
    image: Interval<F>,
    jump_at_endpoint: bool,
) -> Decoration {
    match image.bounds() {
        // The image of the empty set is empty: nothing is guaranteed.
        None => Decoration::Trv,
        Some((lo, hi)) => {
            if lo != hi {
                // The image spans more than one value, so the step function is
                // not constant, hence discontinuous, on the box.
                Decoration::Def
            } else if jump_at_endpoint || !input.is_bounded() {
                Decoration::Dac
            } else {
                Decoration::Com
            }
        }
    }
}

/// Whether `x` is a half-integer (`n + 0.5` for an integer `n`), the jump lattice
/// of the round-to-nearest point functions.
///
/// `x` is a half-integer exactly when `2x` is an integer and `x` is not.
/// Doubling is exact on a correctly-rounded backend, so `add_down` and `add_up`
/// both collapse to the exact `2x` and the integer test (`ceil <= floor` on the
/// doubled bracket) is exact. On the deliberately sound-not-tight `f64` fixture
/// the doubled bracket is one ulp wide, so a value within one ulp of a
/// half-integer can be misgraded; the window is unreachable by the conformance
/// vectors, and only the `com`-versus-`dac` grade is ever affected, never the
/// enclosure, which the exact bare image already fixes.
#[inline]
fn is_half_integer<F: RoundFloat + RoundInteger>(x: F) -> bool {
    if x.floor() == x {
        // An integer is not a half-integer.
        return false;
    }
    let lo = x.add_down(x);
    let hi = x.add_up(x);
    // `2x` lies in `[lo, hi]`; it is an integer iff the bracket admits one.
    lo.ceil() <= hi.floor()
}

/// Whether `x` is an integer, the jump lattice of `floor` and `ceil`. Exact:
/// `floor` fixes `x` iff `x` is already integral.
#[inline]
fn is_integer<F: RoundFloat + RoundInteger>(x: F) -> bool {
    x.floor() == x
}

/// Whether `x` is a nonzero integer, the jump lattice of `trunc`: `trunc` is
/// continuous at zero (`trunc(x) = 0` across a neighborhood of it) and jumps only
/// at the nonzero integers.
#[inline]
fn is_nonzero_integer<F: RoundFloat + RoundInteger>(x: F) -> bool {
    is_integer(x) && !x.is_zero()
}

/// Whether the decoration is consistent with the interval. `com` needs a bounded
/// nonempty interval; `dac` and `def` need a nonempty interval; `ill` is exactly
/// the `NaI` (empty); `trv` holds for anything.
#[inline]
fn valid_combo<F: RoundFloat>(x: Interval<F>, d: Decoration) -> bool {
    match d {
        Decoration::Ill => x.is_empty(),
        Decoration::Com => bounded_nonempty(x),
        Decoration::Dac | Decoration::Def => !x.is_empty(),
        Decoration::Trv => true,
    }
}

/// An interval carrying a decoration that summarizes its computation history.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DecoratedInterval<F> {
    x: Interval<F>,
    d: Decoration,
}

impl<F: RoundFloat> DecoratedInterval<F> {
    /// The Not-an-Interval value: the empty interval decorated `ill`. It is the
    /// poison produced by an invalid construction and propagated by every
    /// operation.
    #[must_use]
    pub fn nai() -> Self {
        Self {
            x: Interval::empty(),
            d: Decoration::Ill,
        }
    }

    /// Decorate a bare interval with the strongest decoration consistent with it,
    /// as a freshly constructed value: `com` if bounded and nonempty, `dac` if
    /// nonempty and unbounded, `trv` if empty.
    #[must_use]
    pub fn new_dec(x: Interval<F>) -> Self {
        let d = if x.is_empty() {
            Decoration::Trv
        } else if x.is_bounded() {
            Decoration::Com
        } else {
            Decoration::Dac
        };
        Self { x, d }
    }

    /// Pair an interval with an explicit decoration, or return [`nai`](DecoratedInterval::nai)
    /// if the combination is inconsistent (for example `com` with an unbounded
    /// interval, or `ill` with a nonempty one).
    #[must_use]
    pub fn set_dec(x: Interval<F>, d: Decoration) -> Self {
        if valid_combo(x, d) {
            Self { x, d }
        } else {
            Self::nai()
        }
    }

    /// The bare interval (the standard's `intervalPart`).
    #[must_use]
    pub fn interval(self) -> Interval<F> {
        self.x
    }

    /// The decoration (the standard's `decorationPart`).
    #[must_use]
    pub fn decoration(self) -> Decoration {
        self.d
    }

    /// Whether this is the `NaI`.
    #[must_use]
    pub fn is_nai(self) -> bool {
        self.d == Decoration::Ill
    }

    /// Assemble a decorated result at the single seam every operation funnels
    /// through. A basic operation earns `com` from bounded nonempty inputs, but
    /// such an operation can overflow to an unbounded result (adding
    /// `[MAX/2, MAX]` to itself reaches `+inf`), and `com` promises a bounded
    /// result, so `com` demotes to `dac` whenever the assembled interval is
    /// unbounded. Where overflow is unreachable the demotion is a harmless
    /// no-op. The assertion guards the remaining combinations in debug builds;
    /// propagation otherwise always produces a consistent pair.
    fn pack(x: Interval<F>, d: Decoration) -> Self {
        // The empty interval counts as bounded, so this fires only for a
        // genuinely unbounded (overflowed) result, never for an empty one.
        let d = if d == Decoration::Com && !x.is_bounded() {
            Decoration::Dac
        } else {
            d
        };
        debug_assert!(
            valid_combo(x, d),
            "decorated interval result violates the decoration invariant"
        );
        Self { x, d }
    }

    /// Reciprocal, decorated. Undefined where the interval contains zero.
    #[must_use]
    pub fn recip(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && !self.x.contains(F::ZERO);
        let dloc = local(dom, bounded_nonempty(self.x));
        Self::pack(self.x.recip(), self.d.meet(dloc))
    }

    /// Square, decorated. Defined and continuous everywhere.
    #[must_use]
    pub fn sqr(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let dloc = local(!self.x.is_empty(), bounded_nonempty(self.x));
        Self::pack(self.x.sqr(), self.d.meet(dloc))
    }

    /// Square root, decorated. Undefined where the interval reaches below zero.
    #[must_use]
    pub fn sqrt(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && self.x.inf() >= F::ZERO;
        let dloc = local(dom, bounded_nonempty(self.x));
        Self::pack(self.x.sqrt(), self.d.meet(dloc))
    }

    /// Multiply then add, `self * factor + addend`, decorated. Defined and
    /// continuous everywhere, so the local decoration is `com` on bounded
    /// nonempty inputs and `dac` when an input is unbounded; a bounded product
    /// or sum that overflows to an unbounded result demotes `com` to `dac` at
    /// the `pack` seam.
    #[must_use]
    pub fn mul_add(self, factor: Self, addend: Self) -> Self {
        if self.is_nai() || factor.is_nai() || addend.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && !factor.x.is_empty() && !addend.x.is_empty();
        let dloc = local(
            dom,
            bounded_nonempty(self.x) && bounded_nonempty(factor.x) && bounded_nonempty(addend.x),
        );
        Self::pack(
            self.x.mul_add(factor.x, addend.x),
            self.d.meet(factor.d).meet(addend.d).meet(dloc),
        )
    }

    /// Absolute value, decorated. Defined and continuous everywhere, so the
    /// local decoration is `com` on a bounded nonempty input and `dac` on an
    /// unbounded one; `def` never arises.
    #[must_use]
    pub fn abs(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let dloc = local(!self.x.is_empty(), bounded_nonempty(self.x));
        Self::pack(self.x.abs(), self.d.meet(dloc))
    }

    /// Pointwise minimum, decorated. Defined and continuous everywhere, so the
    /// local decoration is `com` on bounded nonempty inputs, `dac` when an input
    /// is unbounded, and `trv` when an input is empty.
    #[must_use]
    pub fn min(self, rhs: Self) -> Self {
        if self.is_nai() || rhs.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && !rhs.x.is_empty();
        let dloc = local(dom, bounded_nonempty(self.x) && bounded_nonempty(rhs.x));
        Self::pack(self.x.min(rhs.x), self.d.meet(rhs.d).meet(dloc))
    }

    /// Pointwise maximum, decorated. The companion of [`min`](DecoratedInterval::min),
    /// with the same continuous-everywhere decoration.
    #[must_use]
    pub fn max(self, rhs: Self) -> Self {
        if self.is_nai() || rhs.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && !rhs.x.is_empty();
        let dloc = local(dom, bounded_nonempty(self.x) && bounded_nonempty(rhs.x));
        Self::pack(self.x.max(rhs.x), self.d.meet(rhs.d).meet(dloc))
    }

    /// Sign, decorated. Defined everywhere but discontinuous at zero, so it earns
    /// `def` whenever the restriction to the box is discontinuous, that is,
    /// whenever the box contains points of more than one sign (its image spans
    /// more than one of `-1`, `0`, `1`). The lone jump point is zero, so `com`
    /// demotes to `dac` on the singleton `{0}` (the closed box still meets the
    /// discontinuity) and on an unbounded sign-constant box.
    #[must_use]
    pub fn sign(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let jump_at_endpoint = self.x.inf().is_zero() || self.x.sup().is_zero();
        let dloc = step_local(self.x, self.x.sign(), jump_at_endpoint);
        Self::pack(self.x.sign(), self.d.meet(dloc))
    }
}

impl<F: RoundFloat + round_float::RoundTranscendental> DecoratedInterval<F> {
    /// Exponential, decorated. Defined and continuous on the whole line, so
    /// the decoration only demotes for an unbounded input — or an unbounded
    /// result: `exp` overflows a bounded input to `[lo, +inf]` well within the
    /// finite range, and `com` promises a bounded result, so an overflow
    /// demotes to `dac`.
    #[must_use]
    pub fn exp(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let result = self.x.exp();
        let dloc = local(
            !self.x.is_empty(),
            bounded_nonempty(self.x) && result.is_bounded(),
        );
        Self::pack(result, self.d.meet(dloc))
    }

    /// Natural logarithm, decorated. Undefined where the interval reaches zero
    /// or below; the image of an input touching zero is unbounded below, which
    /// also demotes `com` to `dac`.
    #[must_use]
    pub fn ln(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let result = self.x.ln();
        let dom = !self.x.is_empty() && self.x.inf() > F::ZERO;
        let dloc = local(dom, bounded_nonempty(self.x) && result.is_bounded());
        Self::pack(result, self.d.meet(dloc))
    }
}

impl<F: RoundFloat + RoundInteger> DecoratedInterval<F> {
    /// Ceiling, decorated. Defined everywhere, discontinuous at every integer, so
    /// it earns `def` when the box spans an integer jump (its image is not a
    /// single integer); `com` demotes to `dac` when an integer sits at an
    /// endpoint of a continuous box.
    #[must_use]
    pub fn ceil(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let image = self.x.ceil();
        let jump = is_integer(self.x.inf()) || is_integer(self.x.sup());
        Self::pack(image, self.d.meet(step_local(self.x, image, jump)))
    }

    /// Floor, decorated. The companion of [`ceil`](DecoratedInterval::ceil), also
    /// jumping at every integer.
    #[must_use]
    pub fn floor(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let image = self.x.floor();
        let jump = is_integer(self.x.inf()) || is_integer(self.x.sup());
        Self::pack(image, self.d.meet(step_local(self.x, image, jump)))
    }

    /// Truncation, decorated. Continuous at zero and discontinuous at every
    /// nonzero integer, so the endpoint demotion tests for a nonzero integer.
    #[must_use]
    pub fn trunc(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let image = self.x.trunc();
        let jump = is_nonzero_integer(self.x.inf()) || is_nonzero_integer(self.x.sup());
        Self::pack(image, self.d.meet(step_local(self.x, image, jump)))
    }

    /// Round to nearest, ties to even, decorated. Discontinuous at every
    /// half-integer, so the endpoint demotion tests for a half-integer.
    #[must_use]
    pub fn round_ties_to_even(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let image = self.x.round_ties_to_even();
        let jump = is_half_integer(self.x.inf()) || is_half_integer(self.x.sup());
        Self::pack(image, self.d.meet(step_local(self.x, image, jump)))
    }

    /// Round to nearest, ties away from zero, decorated. The companion of
    /// [`round_ties_to_even`](DecoratedInterval::round_ties_to_even), also
    /// jumping at every half-integer.
    #[must_use]
    pub fn round_ties_to_away(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let image = self.x.round_ties_to_away();
        let jump = is_half_integer(self.x.inf()) || is_half_integer(self.x.sup());
        Self::pack(image, self.d.meet(step_local(self.x, image, jump)))
    }
}

impl<F: RoundFloat> Neg for DecoratedInterval<F> {
    type Output = Self;

    fn neg(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let dloc = local(!self.x.is_empty(), bounded_nonempty(self.x));
        Self::pack(-self.x, self.d.meet(dloc))
    }
}

impl<F: RoundFloat> Add for DecoratedInterval<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if self.is_nai() || rhs.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && !rhs.x.is_empty();
        let dloc = local(dom, bounded_nonempty(self.x) && bounded_nonempty(rhs.x));
        Self::pack(self.x + rhs.x, self.d.meet(rhs.d).meet(dloc))
    }
}

impl<F: RoundFloat> Sub for DecoratedInterval<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        if self.is_nai() || rhs.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && !rhs.x.is_empty();
        let dloc = local(dom, bounded_nonempty(self.x) && bounded_nonempty(rhs.x));
        Self::pack(self.x - rhs.x, self.d.meet(rhs.d).meet(dloc))
    }
}

impl<F: RoundFloat> Mul for DecoratedInterval<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if self.is_nai() || rhs.is_nai() {
            return Self::nai();
        }
        let dom = !self.x.is_empty() && !rhs.x.is_empty();
        let dloc = local(dom, bounded_nonempty(self.x) && bounded_nonempty(rhs.x));
        Self::pack(self.x * rhs.x, self.d.meet(rhs.d).meet(dloc))
    }
}

impl<F: RoundFloat> Div for DecoratedInterval<F> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        if self.is_nai() || rhs.is_nai() {
            return Self::nai();
        }
        // Undefined where the divisor contains zero.
        let dom = !self.x.is_empty() && !rhs.x.is_empty() && !rhs.x.contains(F::ZERO);
        let dloc = local(dom, bounded_nonempty(self.x) && bounded_nonempty(rhs.x));
        Self::pack(self.x / rhs.x, self.d.meet(rhs.d).meet(dloc))
    }
}

#[cfg(all(test, feature = "fixture"))]
mod tests {
    use super::*;

    fn di(lo: f64, hi: f64) -> DecoratedInterval<f64> {
        DecoratedInterval::new_dec(Interval::new(lo, hi).unwrap())
    }

    #[test]
    fn new_dec_assigns_the_strongest_decoration() {
        assert_eq!(di(1.0, 2.0).decoration(), Decoration::Com);
        assert_eq!(
            DecoratedInterval::new_dec(Interval::<f64>::new(0.0, f64::INFINITY).unwrap())
                .decoration(),
            Decoration::Dac
        );
        assert_eq!(
            DecoratedInterval::new_dec(Interval::<f64>::empty()).decoration(),
            Decoration::Trv
        );
    }

    #[test]
    fn bounded_arithmetic_stays_com() {
        let r = di(1.0, 2.0) + di(3.0, 4.0);
        assert_eq!(r.decoration(), Decoration::Com);
        assert_eq!((di(2.0, 3.0) * di(4.0, 5.0)).decoration(), Decoration::Com);
    }

    #[test]
    fn addition_overflow_demotes_to_dac() {
        // Bounded inputs, but the sum overflows to `[MAX, +inf]`; `com` promises
        // a bounded result, so the overflow demotes to `dac` at the pack seam.
        let r = di(f64::MAX / 2.0, f64::MAX) + di(f64::MAX / 2.0, f64::MAX);
        assert!(!r.interval().is_bounded());
        assert_eq!(r.interval().sup(), f64::INFINITY);
        assert_eq!(r.decoration(), Decoration::Dac);
    }

    #[test]
    fn multiplication_overflow_demotes_to_dac() {
        // Bounded factors whose product overflows above the finite range.
        let r = di(f64::MAX / 2.0, f64::MAX) * di(2.0, 2.0);
        assert!(!r.interval().is_bounded());
        assert_eq!(r.interval().sup(), f64::INFINITY);
        assert_eq!(r.decoration(), Decoration::Dac);
    }

    #[test]
    fn mul_add_overflow_demotes_to_dac() {
        // The fused product overflows before the addend is applied.
        let r = di(f64::MAX / 2.0, f64::MAX).mul_add(di(2.0, 2.0), di(1.0, 1.0));
        assert!(!r.interval().is_bounded());
        assert_eq!(r.interval().sup(), f64::INFINITY);
        assert_eq!(r.decoration(), Decoration::Dac);
    }

    #[test]
    fn recip_overflow_demotes_to_dac() {
        // The smallest positive subnormal inverts above the finite range, so a
        // bounded zero-excluding interval reciprocates to an unbounded upper
        // endpoint, another bounded-input-to-unbounded-result case the
        // input-only decoration would have packed as `com`.
        let tiny = f64::from_bits(1);
        let r = di(tiny, 1.0).recip();
        assert!(!r.interval().is_bounded());
        assert_eq!(r.interval().sup(), f64::INFINITY);
        assert_eq!(r.decoration(), Decoration::Dac);
    }

    #[test]
    fn mul_add_bounded_stays_com() {
        // An ordinary bounded fused multiply-add keeps `com`: the demotion does
        // not touch a result that stays inside the finite range.
        let r = di(2.0, 3.0).mul_add(di(4.0, 5.0), di(1.0, 1.0));
        assert_eq!(r.decoration(), Decoration::Com);
        assert!(r.interval().contains(9.0));
        assert!(r.interval().contains(16.0));
    }

    #[test]
    fn division_by_zero_straddle_drops_to_trv() {
        let r = di(1.0, 2.0) / di(-1.0, 1.0);
        assert_eq!(r.decoration(), Decoration::Trv);
        assert!(r.interval().is_entire());
    }

    #[test]
    fn sqrt_reaching_below_zero_drops_to_trv() {
        let r = di(-2.0, 4.0).sqrt();
        assert_eq!(r.decoration(), Decoration::Trv);
        // The bare interval is still the sound enclosure of the defined part.
        assert_eq!(r.interval().inf(), 0.0);
        assert!(r.interval().contains(2.0));
    }

    #[test]
    fn sqrt_of_nonnegative_stays_com() {
        assert_eq!(di(1.0, 4.0).sqrt().decoration(), Decoration::Com);
    }

    #[test]
    fn unbounded_continuous_is_dac() {
        let unb = DecoratedInterval::new_dec(Interval::<f64>::new(0.0, f64::INFINITY).unwrap());
        // 0 not in [0, inf]? it is at the lower endpoint, so recip is trv-ish;
        // use addition with a bounded com to test dac propagation instead.
        let r = unb + di(1.0, 2.0);
        assert_eq!(r.decoration(), Decoration::Dac);
    }

    #[test]
    fn nai_poisons_every_operation() {
        let nai = DecoratedInterval::<f64>::nai();
        assert!((nai + di(1.0, 2.0)).is_nai());
        assert!((di(1.0, 2.0) + nai).is_nai());
        assert!((di(1.0, 2.0) * nai).is_nai());
        assert!((nai / di(1.0, 2.0)).is_nai());
        assert!(nai.sqrt().is_nai());
        assert!(nai.recip().is_nai());
        assert!((-nai).is_nai());
    }

    #[test]
    fn the_meet_weakens() {
        // A com input through a trv-producing op yields trv.
        let r = di(1.0, 2.0) / di(0.0, 0.0);
        assert_eq!(r.decoration(), Decoration::Trv);
        assert!(r.interval().is_empty());
    }

    #[test]
    fn exp_of_bounded_stays_com() {
        let r = di(0.0, 1.0).exp();
        assert_eq!(r.decoration(), Decoration::Com);
        assert!(r.interval().contains(1.0));
    }

    #[test]
    fn exp_overflow_demotes_to_dac() {
        // exp is defined and continuous everywhere, but the result escapes the
        // finite range, and com promises a bounded result.
        let r = di(0.0, 1.0e3).exp();
        assert_eq!(r.decoration(), Decoration::Dac);
        assert_eq!(r.interval().sup(), f64::INFINITY);
    }

    #[test]
    fn exp_propagates_nai() {
        assert!(DecoratedInterval::<f64>::nai().exp().is_nai());
    }

    #[test]
    fn ln_of_positive_bounded_stays_com() {
        let r = di(1.0, core::f64::consts::E).ln();
        assert_eq!(r.decoration(), Decoration::Com);
        assert!(r.interval().contains(0.5));
    }

    #[test]
    fn ln_reaching_zero_drops_to_trv() {
        // Zero is outside ln's domain, so the restriction is a domain
        // violation (trv), and the bare interval is the enclosure of the
        // defined part.
        let r = di(0.0, 1.0).ln();
        assert_eq!(r.decoration(), Decoration::Trv);
        assert_eq!(r.interval().inf(), f64::NEG_INFINITY);
        assert!(r.interval().contains(0.0));
    }

    #[test]
    fn ln_wholly_nonpositive_is_trv_empty() {
        let r = di(-2.0, -1.0).ln();
        assert_eq!(r.decoration(), Decoration::Trv);
        assert!(r.interval().is_empty());
    }

    #[test]
    fn ln_propagates_nai() {
        assert!(DecoratedInterval::<f64>::nai().ln().is_nai());
    }

    #[test]
    fn set_dec_rejects_inconsistent_combinations() {
        // com with an unbounded interval is inconsistent -> NaI.
        let bad = DecoratedInterval::set_dec(
            Interval::<f64>::new(0.0, f64::INFINITY).unwrap(),
            Decoration::Com,
        );
        assert!(bad.is_nai());
        // ill with a nonempty interval is inconsistent -> NaI.
        let bad2 =
            DecoratedInterval::set_dec(Interval::<f64>::new(1.0, 2.0).unwrap(), Decoration::Ill);
        assert!(bad2.is_nai());
        // A consistent combination is kept.
        let ok =
            DecoratedInterval::set_dec(Interval::<f64>::new(1.0, 2.0).unwrap(), Decoration::Def);
        assert_eq!(ok.decoration(), Decoration::Def);
    }
}
