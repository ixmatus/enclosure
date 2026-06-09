//! Property tests over the `f64` fixture instance of `RoundFloat`.
//!
//! Phase A covers the construction invariant and the observers. Phase B adds the
//! enclosure properties: an operation's result contains the pointwise result
//! over the inputs (the fundamental theorem), and the singleton merge gate (a
//! point operation's result is enclosed), which is the property the SMIL engine
//! also gates on.

use interval_1788::Interval;
use proptest::prelude::*;

/// A finite, non-NaN `f64` in a range wide enough to exercise sign mixing and
/// magnitude spread without routinely overflowing to infinity.
fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![-1.0e6..1.0e6, Just(0.0), Just(-0.0), -1.0e-6..1.0e-6,]
}

/// A bounded interval paired with a point that provably lies inside it: sort
/// three finite values, take the outer two as endpoints and the middle as the
/// witness, so the membership needs no arithmetic that could round the witness
/// out of the interval.
fn interval_and_point() -> impl Strategy<Value = (Interval<f64>, f64)> {
    (finite(), finite(), finite()).prop_map(|(a, b, c)| {
        let mut v = [a, b, c];
        v.sort_by(|p, q| p.partial_cmp(q).unwrap());
        (Interval::new(v[0], v[2]).unwrap(), v[1])
    })
}

proptest! {
    // --- Phase A: construction invariant and observers ---

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
        let mut v = [a, b, c];
        v.sort_by(|p, q| p.partial_cmp(q).unwrap());
        let i = Interval::<f64>::new(v[0], v[2]).unwrap();
        prop_assert!(i.contains(v[1]));
    }

    #[test]
    fn point_is_a_singleton_containing_itself(x in finite()) {
        let i = Interval::<f64>::point(x).unwrap();
        prop_assert!(i.is_singleton());
        prop_assert!(i.contains(x));
        prop_assert_eq!(i.inf(), i.sup());
    }

    // --- Phase B: enclosure (the fundamental theorem of interval arithmetic) ---

    #[test]
    fn add_encloses_pointwise((a, x) in interval_and_point(), (b, y) in interval_and_point()) {
        prop_assert!((a + b).contains(x + y));
    }

    #[test]
    fn sub_encloses_pointwise((a, x) in interval_and_point(), (b, y) in interval_and_point()) {
        prop_assert!((a - b).contains(x - y));
    }

    #[test]
    fn mul_encloses_pointwise((a, x) in interval_and_point(), (b, y) in interval_and_point()) {
        prop_assert!((a * b).contains(x * y));
    }

    #[test]
    fn sqr_encloses_pointwise((a, x) in interval_and_point()) {
        prop_assert!(a.sqr().contains(x * x));
    }

    // --- Phase B: the singleton merge gate ---
    // A point operation's result is enclosed by the corresponding interval
    // operation on singletons. This is the property the SMIL engine gates on.

    #[test]
    fn merge_gate_encloses_nearest_point_result(x in finite(), y in finite()) {
        let px = Interval::<f64>::point(x).unwrap();
        let py = Interval::<f64>::point(y).unwrap();
        prop_assert!((px + py).contains(x + y));
        prop_assert!((px - py).contains(x - y));
        prop_assert!((px * py).contains(x * y));
        if y != 0.0 {
            prop_assert!((px / py).contains(x / y));
        }
        if x >= 0.0 {
            prop_assert!(px.sqrt().contains(x.sqrt()));
        }
    }

    // --- Phase B: structural properties ---

    #[test]
    fn add_and_mul_commute((a, _x) in interval_and_point(), (b, _y) in interval_and_point()) {
        prop_assert_eq!(a + b, b + a);
        prop_assert_eq!(a * b, b * a);
    }

    #[test]
    fn every_result_upholds_the_invariant(
        (a, _x) in interval_and_point(),
        (b, _y) in interval_and_point(),
    ) {
        for r in [
            a + b,
            a - b,
            a * b,
            a / b,
            a.sqr(),
            a.sqrt(),
            a.recip(),
            -a,
        ] {
            prop_assert!(r.is_empty() || r.inf() <= r.sup());
        }
    }
}
