//! Property and unit tests for the P1 construction layer, over the f64 fixture.
//!
//! The load-bearing property is enclosure: the interval an affine form reduces
//! to always contains the interval it was built from. The fixture is sound but
//! not tight, so these check soundness; tightness belongs to a correctly-rounded
//! backend.

use affine_arith::{with_source, AffineForm};
use interval_1788::Interval;
use proptest::prelude::*;

proptest! {
    /// Build a form from a bounded interval, reduce it, and confirm the result
    /// encloses both original endpoints (and therefore the whole set).
    #[test]
    fn from_interval_then_reduce_encloses(a in -1.0e9f64..1.0e9, b in -1.0e9f64..1.0e9) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let iv = Interval::new(lo, hi).expect("finite ordered endpoints");
        let (reduced, terms) = with_source(|mut src| {
            let form = AffineForm::from_interval(&iv, &mut src).expect("bounded nonempty");
            (form.reduce(), form.num_terms())
        });
        prop_assert!(reduced.inf() <= lo, "lower endpoint {lo} escaped below {}", reduced.inf());
        prop_assert!(reduced.sup() >= hi, "upper endpoint {hi} escaped above {}", reduced.sup());
        prop_assert!(reduced.contains(lo));
        prop_assert!(reduced.contains(hi));

        // A nondegenerate interval spends exactly one symbol.
        if lo < hi {
            prop_assert_eq!(terms, 1);
        }
    }

    /// A point form carries no uncertainty and reduces to a singleton that
    /// contains its center. A point needs no source.
    #[test]
    fn point_form_is_exact(c in -1.0e9f64..1.0e9) {
        let form = AffineForm::point(c);
        prop_assert!(form.is_point());
        prop_assert_eq!(form.num_terms(), 0);
        let reduced = form.reduce();
        prop_assert!(reduced.contains(c));
        prop_assert_eq!(reduced.inf(), c);
        prop_assert_eq!(reduced.sup(), c);
    }

    /// A singleton interval round-trips: the value it contains stays enclosed,
    /// and the form spends at most one symbol.
    #[test]
    fn singleton_round_trips(c in -1.0e9f64..1.0e9) {
        let iv = Interval::point(c).expect("finite point");
        let (reduced, terms) = with_source(|mut src| {
            let form = AffineForm::from_interval(&iv, &mut src).expect("bounded nonempty");
            (form.reduce(), form.num_terms())
        });
        prop_assert!(reduced.contains(c));
        prop_assert!(terms <= 1);
    }
}

#[test]
fn empty_interval_has_no_form() {
    let absent = with_source(|mut src| {
        let empty = Interval::<f64>::empty();
        AffineForm::from_interval(&empty, &mut src).is_none()
    });
    assert!(absent);
}

#[test]
fn unbounded_interval_has_no_form() {
    let absent = with_source(|mut src| {
        let entire = Interval::<f64>::entire();
        AffineForm::from_interval(&entire, &mut src).is_none()
    });
    assert!(absent);
}

#[test]
fn fresh_symbols_are_distinct() {
    with_source(|mut src| {
        let a = src.fresh();
        let b = src.fresh();
        let c = src.fresh();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
        assert_eq!(a.id(), 0);
        assert_eq!(b.id(), 1);
    });
}

#[test]
fn known_interval_builds_a_centered_form() {
    let (terms, center, reduced) = with_source(|mut src| {
        let iv = Interval::new(2.0_f64, 4.0).expect("ordered");
        let form = AffineForm::from_interval(&iv, &mut src).expect("bounded");
        (form.num_terms(), form.center(), form.reduce())
    });
    assert_eq!(terms, 1);
    // The center lies inside the source interval, and the reduction encloses it.
    assert!((2.0..=4.0).contains(&center));
    assert!(reduced.inf() <= 2.0);
    assert!(reduced.sup() >= 4.0);
}
