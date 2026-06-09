//! Property and unit tests for the P1 construction layer, over the f64 fixture.
//!
//! The load-bearing property is enclosure: the interval an affine form reduces
//! to always contains the interval it was built from. The fixture is sound but
//! not tight, so these check soundness; tightness belongs to a correctly-rounded
//! backend.

use affine_arith::{AffineForm, SymbolSource};
use interval_1788::Interval;
use proptest::prelude::*;

proptest! {
    /// Build a form from a bounded interval, reduce it, and confirm the result
    /// encloses both original endpoints (and therefore the whole set).
    #[test]
    fn from_interval_then_reduce_encloses(a in -1.0e9f64..1.0e9, b in -1.0e9f64..1.0e9) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let iv = Interval::new(lo, hi).expect("finite ordered endpoints");
        let mut src = SymbolSource::new();
        let form = AffineForm::from_interval(&iv, &mut src).expect("bounded nonempty");

        let reduced = form.reduce();
        prop_assert!(reduced.inf() <= lo, "lower endpoint {lo} escaped below {}", reduced.inf());
        prop_assert!(reduced.sup() >= hi, "upper endpoint {hi} escaped above {}", reduced.sup());
        prop_assert!(reduced.contains(lo));
        prop_assert!(reduced.contains(hi));

        // A nondegenerate interval spends exactly one symbol; the center sits in
        // the original interval.
        if lo < hi {
            prop_assert_eq!(form.num_terms(), 1);
        }
    }

    /// A point form carries no uncertainty and reduces to a singleton that
    /// contains its center.
    #[test]
    fn point_form_is_exact(c in -1.0e9f64..1.0e9) {
        let form = AffineForm::<f64>::point(c);
        prop_assert!(form.is_point());
        prop_assert_eq!(form.num_terms(), 0);
        let reduced = form.reduce();
        prop_assert!(reduced.contains(c));
        prop_assert_eq!(reduced.inf(), c);
        prop_assert_eq!(reduced.sup(), c);
    }

    /// A singleton interval round-trips: every value it contains (only `c`)
    /// stays enclosed, and the form spends at most one symbol.
    #[test]
    fn singleton_round_trips(c in -1.0e9f64..1.0e9) {
        let iv = Interval::point(c).expect("finite point");
        let mut src = SymbolSource::new();
        let form = AffineForm::from_interval(&iv, &mut src).expect("bounded nonempty");
        let reduced = form.reduce();
        prop_assert!(reduced.contains(c));
        prop_assert!(form.num_terms() <= 1);
    }
}

#[test]
fn empty_interval_has_no_form() {
    let mut src = SymbolSource::new();
    let empty = Interval::<f64>::empty();
    assert!(AffineForm::from_interval(&empty, &mut src).is_none());
}

#[test]
fn unbounded_interval_has_no_form() {
    let mut src = SymbolSource::new();
    let entire = Interval::<f64>::entire();
    assert!(AffineForm::from_interval(&entire, &mut src).is_none());
}

#[test]
fn fresh_symbols_are_distinct() {
    let mut src = SymbolSource::new();
    let a = src.fresh();
    let b = src.fresh();
    let c = src.fresh();
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
    assert_eq!(a.id(), 0);
    assert_eq!(b.id(), 1);
}

#[test]
fn known_interval_builds_a_centered_form() {
    let iv = Interval::new(2.0_f64, 4.0).expect("ordered");
    let mut src = SymbolSource::new();
    let form = AffineForm::from_interval(&iv, &mut src).expect("bounded");
    assert_eq!(form.num_terms(), 1);
    // The center lies inside the source interval, and the reduction encloses it.
    assert!((2.0..=4.0).contains(&form.center()));
    let reduced = form.reduce();
    assert!(reduced.inf() <= 2.0);
    assert!(reduced.sup() >= 4.0);
}
