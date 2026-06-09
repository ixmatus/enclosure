//! [`DecoratedInterval`]: an interval paired with a decoration, with operations
//! that propagate the decoration per the fundamental theorem of decorated
//! interval arithmetic.
//!
//! A decorated operation computes the bare interval result, determines the
//! decoration the operation earned on its bare inputs (its *local* decoration),
//! and decorates the result with the meet of that local decoration and the
//! decorations the inputs carried. A `NaI` input poisons the result.
//!
//! For the operations here (`+`, `-`, `*`, `/`, `neg`, `recip`, `sqr`, `sqrt`)
//! the local decoration is one of `com`, `dac`, or `trv`. These functions are
//! continuous wherever they are defined, so `def` (defined but discontinuous)
//! never arises from them; it is reachable only through [`set_dec`](DecoratedInterval::set_dec).
//! The local decoration is:
//!
//! - `trv` when the operation is undefined somewhere on the input box (division
//!   by an interval containing zero, square root of an interval reaching below
//!   zero) or an input is empty;
//! - `com` when it is defined and continuous and every input is bounded and
//!   nonempty (so the result is bounded and nonempty);
//! - `dac` when it is defined and continuous but some input is unbounded.

use core::ops::{Add, Div, Mul, Neg, Sub};

use crate::decoration::Decoration;
use crate::interval::Interval;
use round_float::RoundFloat;

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

    /// Construct a result, asserting the decoration is consistent with the
    /// interval. Propagation always produces a consistent pair; the assertion
    /// guards that in debug builds.
    fn pack(x: Interval<F>, d: Decoration) -> Self {
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
