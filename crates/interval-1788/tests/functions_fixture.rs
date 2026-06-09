//! Property tests for the numeric, boolean, and set functions, over the `f64`
//! fixture. The headline is inclusion isotonicity (the monotonicity half of the
//! fundamental theorem of interval arithmetic), now expressible with `subset`.

use interval_1788::Interval;
use proptest::prelude::*;

fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![-1.0e6..1.0e6, Just(0.0), Just(-0.0), -1.0e-6..1.0e-6,]
}

/// A bounded interval together with a bounded superinterval that contains it.
/// Sort four finite values; the inner two are the subinterval, the outer two the
/// superinterval, so containment holds by construction.
fn nested() -> impl Strategy<Value = (Interval<f64>, Interval<f64>)> {
    (finite(), finite(), finite(), finite()).prop_map(|(a, b, c, d)| {
        let mut v = [a, b, c, d];
        v.sort_by(|p, q| p.partial_cmp(q).unwrap());
        (
            Interval::new(v[1], v[2]).unwrap(),
            Interval::new(v[0], v[3]).unwrap(),
        )
    })
}

fn bounded() -> impl Strategy<Value = Interval<f64>> {
    (finite(), finite()).prop_map(|(a, b)| {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        Interval::new(lo, hi).unwrap()
    })
}

proptest! {
    #[test]
    fn subset_is_reflexive(a in bounded()) {
        prop_assert!(a.subset(a));
    }

    #[test]
    fn nested_pairs_are_subsets(pair in nested()) {
        let (inner, outer) = pair;
        prop_assert!(inner.subset(outer));
    }

    #[test]
    fn intersection_is_contained_in_both(a in bounded(), b in bounded()) {
        let m = a.intersection(b);
        prop_assert!(m.subset(a));
        prop_assert!(m.subset(b));
    }

    #[test]
    fn convex_hull_contains_both(a in bounded(), b in bounded()) {
        let h = a.convex_hull(b);
        prop_assert!(a.subset(h));
        prop_assert!(b.subset(h));
    }

    #[test]
    fn intersection_and_hull_commute(a in bounded(), b in bounded()) {
        prop_assert_eq!(a.intersection(b), b.intersection(a));
        prop_assert_eq!(a.convex_hull(b), b.convex_hull(a));
    }

    #[test]
    fn disjoint_is_symmetric(a in bounded(), b in bounded()) {
        prop_assert_eq!(a.disjoint(b), b.disjoint(a));
    }

    #[test]
    fn disjoint_iff_intersection_empty(a in bounded(), b in bounded()) {
        prop_assert_eq!(a.disjoint(b), a.intersection(b).is_empty());
    }

    #[test]
    fn mignitude_does_not_exceed_magnitude(a in bounded()) {
        let (mig, mag) = (a.mig().unwrap(), a.mag().unwrap());
        prop_assert!(mig <= mag);
    }

    #[test]
    fn width_is_nonnegative(a in bounded()) {
        prop_assert!(a.wid().unwrap() >= 0.0);
    }

    // Inclusion isotonicity: if a ⊆ a' and b ⊆ b', then a ∘ b ⊆ a' ∘ b'.
    // The monotonicity half of the fundamental theorem of interval arithmetic.

    #[test]
    fn add_is_inclusion_isotonic(pa in nested(), pb in nested()) {
        let (a, a2) = pa;
        let (b, b2) = pb;
        prop_assert!((a + b).subset(a2 + b2));
    }

    #[test]
    fn mul_is_inclusion_isotonic(pa in nested(), pb in nested()) {
        let (a, a2) = pa;
        let (b, b2) = pb;
        prop_assert!((a * b).subset(a2 * b2));
    }

    #[test]
    fn sub_is_inclusion_isotonic(pa in nested(), pb in nested()) {
        let (a, a2) = pa;
        let (b, b2) = pb;
        prop_assert!((a - b).subset(a2 - b2));
    }
}
