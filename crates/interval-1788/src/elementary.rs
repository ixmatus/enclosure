//! Elementary transcendental functions over intervals: the Phase E opener.
//!
//! These functions require the backend to supply outward-rounded
//! transcendentals, so they live behind `F: RoundFloat + RoundTranscendental`
//! rather than enlarging the core [`RoundFloat`] contract every backend must
//! satisfy (see round-float decision record 0001). A backend without the
//! extension trait simply does not have these methods; everything else about
//! the interval type is unaffected.
//!
//! Monotone functions need no four-corner analysis: the image of `[a, b]`
//! under an increasing `f` is `[f(a), f(b)]`, one endpoint rounded down and
//! the other up. The set-based flavor keeps every operation total by
//! restricting the input to the function's natural domain first, exactly as
//! [`sqrt`](Interval::sqrt) does: an input wholly outside the domain has the
//! empty image.

use crate::interval::Interval;
use round_float::{RoundFloat, RoundTranscendental};

impl<F: RoundFloat + RoundTranscendental> Interval<F> {
    /// The exponential, the set `{ e^x : x in self }`.
    ///
    /// Defined and increasing on the whole line, so the image of `[a, b]` is
    /// `[e^a, e^b]` with outward rounding. The exponential of a value
    /// unbounded below reaches down to (but never attains) zero, so zero is
    /// the lower endpoint there; a value unbounded above maps to an upper
    /// endpoint of `+inf`. The lower endpoint is clamped to zero: the true
    /// image is positive, and the clamp keeps a loose backend (the f64
    /// fixture's unconditional outward step) from reporting a spuriously
    /// negative bound.
    #[must_use]
    pub fn exp(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        // The representation invariant leaves only `-inf` possible for an
        // infinite lower endpoint, and only `+inf` for an infinite upper.
        let lo = if a.is_infinite() {
            F::ZERO
        } else {
            a.exp_down().rmax(F::ZERO)
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.exp_up()
        };
        Self::hull(lo, hi)
    }

    /// The natural logarithm, the set `{ ln(x) : x in self, x > 0 }`.
    ///
    /// Total in the set-based flavor: the domain is `(0, +inf)`, and the input
    /// is restricted to it. An interval wholly at or below zero has the empty
    /// image. An interval whose positive part reaches down to zero has an
    /// image unbounded below (`ln` decreases without bound toward `0+`); zero
    /// itself is not in the domain, so `[0, b]` contributes `[-inf, ln b]`.
    #[must_use]
    pub fn ln(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if b <= zero {
            // No point of the input is in the domain.
            return Self::empty();
        }
        let lo = if a <= zero {
            // The restricted set (0, b] reaches arbitrarily close to zero.
            F::NEG_INFINITY
        } else {
            // `a > 0` is finite here: the invariant excludes `+inf` from a
            // lower endpoint.
            a.ln_down()
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.ln_up()
        };
        Self::hull(lo, hi)
    }
}

#[cfg(all(test, feature = "fixture"))]
mod tests {
    use super::*;

    fn iv(lo: f64, hi: f64) -> Interval<f64> {
        Interval::new(lo, hi).unwrap()
    }

    #[test]
    fn exp_encloses_endpoint_images() {
        let r = iv(0.0, 1.0).exp();
        assert!(r.inf() <= 1.0 && 1.0 <= r.sup());
        assert!(r.contains(0.5_f64.exp()));
        assert!(r.contains(core::f64::consts::E));
    }

    #[test]
    fn exp_of_empty_is_empty() {
        assert!(Interval::<f64>::empty().exp().is_empty());
    }

    #[test]
    fn exp_unbounded_below_reaches_zero() {
        let r = iv(f64::NEG_INFINITY, 0.0).exp();
        assert_eq!(r.inf(), 0.0);
        assert!(r.sup() >= 1.0);
    }

    #[test]
    fn exp_unbounded_above_is_unbounded() {
        let r = iv(0.0, f64::INFINITY).exp();
        assert_eq!(r.sup(), f64::INFINITY);
        assert!(r.inf() >= 0.0);
    }

    #[test]
    fn exp_overflow_is_unbounded_above() {
        let r = iv(0.0, 1.0e3).exp();
        assert_eq!(r.sup(), f64::INFINITY);
    }

    #[test]
    fn exp_lower_bound_never_negative() {
        let r = iv(-1.0e3, -1.0e2).exp();
        assert!(r.inf() >= 0.0);
        assert!(r.sup() > 0.0);
    }

    #[test]
    fn ln_encloses_endpoint_images() {
        let r = iv(1.0, core::f64::consts::E).ln();
        assert!(r.contains(0.0));
        assert!(r.contains(1.0));
        assert!(r.contains(0.5));
    }

    #[test]
    fn ln_of_empty_is_empty() {
        assert!(Interval::<f64>::empty().ln().is_empty());
    }

    #[test]
    fn ln_wholly_nonpositive_is_empty() {
        assert!(iv(-2.0, -1.0).ln().is_empty());
        assert!(iv(-1.0, 0.0).ln().is_empty());
        assert!(Interval::<f64>::point(0.0).unwrap().ln().is_empty());
    }

    #[test]
    fn ln_touching_zero_is_unbounded_below() {
        let r = iv(0.0, 1.0).ln();
        assert_eq!(r.inf(), f64::NEG_INFINITY);
        assert!(r.contains(0.0));
    }

    #[test]
    fn ln_straddling_zero_restricts_the_domain() {
        let r = iv(-3.0, core::f64::consts::E).ln();
        assert_eq!(r.inf(), f64::NEG_INFINITY);
        assert!(r.contains(1.0));
    }

    #[test]
    fn ln_unbounded_above_is_unbounded() {
        let r = iv(1.0, f64::INFINITY).ln();
        assert!(r.contains(0.0));
        assert_eq!(r.sup(), f64::INFINITY);
    }

    #[test]
    fn exp_ln_round_trip_encloses_identity() {
        let x = iv(0.5, 2.0);
        let r = x.ln().exp();
        assert!(r.inf() <= 0.5 && 2.0 <= r.sup());
    }
}
