//! Property and unit tests for the persistent-forms surface: the unscoped
//! source escape hatch and the serialization round-trip through
//! `from_raw_parts`, over the f64 fixture.
//!
//! The load-bearing properties: the serialization round-trip is faithful
//! (center, terms, reduction, and surviving correlation), and a resumed source
//! never re-issues a live id when the persistence contract is followed.

use affine_arith::{with_source, AffineForm, SymbolSource};
use interval_1788::Interval;
use proptest::prelude::*;

proptest! {
    /// The serialization round-trip is faithful: `from_raw_parts` over
    /// `(center, terms)` rebuilds a form with the same center, terms, and
    /// reduction.
    #[test]
    fn raw_parts_round_trip(
        lo in -1.0e9f64..1.0e9,
        width in 0.0f64..1.0e9,
    ) {
        let iv = Interval::new(lo, lo + width).expect("ordered endpoints");
        let mut src = SymbolSource::unscoped();
        let x = AffineForm::from_interval(&iv, &mut src).expect("bounded nonempty");
        let y = x.add(&x, &mut src);

        let raw: Vec<(u64, f64)> =
            y.terms().iter().map(|t| (t.symbol().id(), t.coeff())).collect();
        let rebuilt = AffineForm::from_raw_parts(y.center(), raw).expect("faithful serialization");

        prop_assert_eq!(rebuilt.center(), y.center());
        prop_assert_eq!(rebuilt.num_terms(), y.num_terms());
        prop_assert_eq!(rebuilt.reduce().inf(), y.reduce().inf());
        prop_assert_eq!(rebuilt.reduce().sup(), y.reduce().sup());

        // The rebuilt form still correlates with the live one: y - y == 0-ish
        // (exact on a correctly-rounded backend; fixture leaves ulp dust).
        let cancelled = rebuilt.sub(&y, &mut src).reduce();
        prop_assert!(cancelled.contains(0.0));
    }
}

#[test]
fn unscoped_resume_respects_freshness() {
    let mut src = SymbolSource::unscoped();
    let iv = Interval::new(1.0_f64, 2.0).unwrap();
    let x = AffineForm::from_interval(&iv, &mut src).unwrap();
    let saved_counter = src.next_id();
    let max_live_id = x.terms().iter().map(|t| t.symbol().id()).max().unwrap();

    // Persist (counter, form), restart, restore both.
    let raw: Vec<(u64, f64)> = x
        .terms()
        .iter()
        .map(|t| (t.symbol().id(), t.coeff()))
        .collect();
    let restored = AffineForm::from_raw_parts(x.center(), raw).unwrap();
    let mut resumed = SymbolSource::unscoped_at(saved_counter);

    // Freshness holds: the next symbol exceeds every live id.
    let fresh = resumed.fresh();
    assert!(fresh.id() > max_live_id);

    // And the restored form still cancels against itself through the resumed
    // source (the correlation survived the round-trip).
    let mut resumed2 = SymbolSource::unscoped_at(resumed.next_id());
    let diff = restored.sub(&restored, &mut resumed2).reduce();
    assert!(diff.contains(0.0));
}

#[test]
fn from_raw_parts_rejects_unfaithful_input() {
    // Out-of-order ids.
    assert!(AffineForm::<f64>::from_raw_parts(0.0, [(3, 1.0), (1, 1.0)]).is_none());
    // Duplicate ids.
    assert!(AffineForm::<f64>::from_raw_parts(0.0, [(1, 1.0), (1, 2.0)]).is_none());
    // Non-finite center and coefficient.
    assert!(AffineForm::<f64>::from_raw_parts(f64::NAN, []).is_none());
    assert!(AffineForm::<f64>::from_raw_parts(0.0, [(1, f64::INFINITY)]).is_none());
    // Zero coefficients are dropped, not rejected.
    let form = AffineForm::<f64>::from_raw_parts(1.0, [(1, 0.0), (2, 3.0)]).unwrap();
    assert_eq!(form.num_terms(), 1);
}

#[test]
fn point_forms_bridge_scoped_and_unscoped_worlds() {
    // A point form has no symbols, so it may combine with forms from any
    // source; the brand is inferred. This pins that both worlds accept it.
    let mut unscoped = SymbolSource::unscoped();
    let iv = Interval::new(1.0_f64, 2.0).unwrap();
    let x = AffineForm::from_interval(&iv, &mut unscoped).unwrap();
    let shifted = x.add(&AffineForm::point(10.0), &mut unscoped).reduce();
    assert!(shifted.contains(11.5));

    let scoped = with_source(|mut src| {
        let y = AffineForm::from_interval(&iv, &mut src).unwrap();
        y.add(&AffineForm::point(10.0), &mut src).reduce()
    });
    assert!(scoped.contains(11.5));
}
