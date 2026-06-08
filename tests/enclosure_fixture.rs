//! Property tests over the `f64` fixture instance of `RoundFloat`.
//!
//! Phase A: the construction invariant and the observers. Phase B adds the
//! enclosure properties (an operation's result contains the pointwise result)
//! once the arithmetic operations exist.

use interval_1788::{Interval, IntervalError};
use proptest::prelude::*;

/// A finite, non-NaN `f64` in a range wide enough to exercise sign mixing and
/// magnitude spread without routinely overflowing to infinity.
fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![-1.0e6..1.0e6, Just(0.0), Just(-0.0), -1.0e-6..1.0e-6,]
}

proptest! {
    #[test]
    fn ordered_finite_pair_constructs_and_round_trips(a in finite(), b in finite()) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let i = Interval::<f64>::new(lo, hi).expect("ordered finite pair is valid");
        prop_assert_eq!(i.inf(), lo);
        prop_assert_eq!(i.sup(), hi);
        prop_assert!(i.is_bounded());
        prop_assert!(i.contains(lo));
        prop_assert!(i.contains(hi));
    }

    #[test]
    fn contains_holds_for_the_middle_of_three(a in finite(), b in finite(), c in finite()) {
        // Sort three finite values exactly (no arithmetic that could round a
        // witness outside the interval), then the middle lies in [min, max].
        let mut v = [a, b, c];
        v.sort_by(|p, q| p.partial_cmp(q).unwrap());
        let i = Interval::<f64>::new(v[0], v[2]).unwrap();
        prop_assert!(i.contains(v[1]));
    }

    #[test]
    fn strictly_inverted_pair_is_rejected(a in finite(), b in finite()) {
        prop_assume!(a > b);
        prop_assert_eq!(Interval::<f64>::new(a, b), Err(IntervalError::Inverted));
    }

    #[test]
    fn point_is_a_singleton_containing_itself(x in finite()) {
        let i = Interval::<f64>::point(x).unwrap();
        prop_assert!(i.is_singleton());
        prop_assert!(i.contains(x));
        prop_assert_eq!(i.inf(), i.sup());
    }

    #[test]
    fn entire_contains_every_finite_value(x in finite()) {
        prop_assert!(Interval::<f64>::entire().contains(x));
    }

    #[test]
    fn empty_contains_nothing(x in finite()) {
        prop_assert!(!Interval::<f64>::empty().contains(x));
    }
}
