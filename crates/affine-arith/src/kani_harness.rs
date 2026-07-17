//! Kani proof harnesses for the discrete parts of affine arithmetic, over the
//! `f64` fixture.
//!
//! These discharge the finite, set-based logic that is easy to get wrong and
//! carries no directed-rounding arithmetic: noise-symbol freshness, the
//! serialization validation of [`from_raw_parts`](crate::AffineForm::from_raw_parts),
//! the shape invariant the addition merge must preserve, and the term-count
//! logic of condensation. The symbolic variables are ids, counts, and budgets;
//! coefficients are concrete, so CBMC never has to reason about the fixture's
//! `next_up`/`next_down` bit manipulation.
//!
//! Run with `cargo kani -p affine-arith --features fixture`; gated on `fixture`
//! for the `f64` instance of [`RoundFloat`](round_float::RoundFloat).
//!
//! # Why the numeric enclosure is not proven here
//!
//! The enclosure theorem, that a form's reduction contains its true value set,
//! is NOT a Kani target over this fixture. interval-1788 ADR-0003 found that the
//! directed-rounding fixture rounds outward with `next_up`/`next_down`, which
//! transmute `f64` to and from its bit pattern and branch on it; CBMC bit-blasts
//! that into a formula too large to discharge even for one two-operand
//! operation. The affine layer inherits the finding: every enclosure-critical
//! step (the multiply's bilinear remainder, the Chebyshev residuals, the
//! outward error folding) runs that arithmetic, so it stays with the property
//! lanes (`tests/*_fixture.rs`) and, for `exp`/`ln`, the `elementary-oracle`
//! crate. A green `kani` badge here therefore attests to the discrete skeleton,
//! not to enclosure; anyone citing the badge cites this caveat. The route to a
//! machine-checked four-corner enclosure is the abstract-axiom backend
//! (ADR-0003's alternative): a Kani-only `RoundFloat` whose directed operations
//! are symbolic values constrained by the soundness inequality rather than
//! computed by nextafter. When it lands, the affine `add`/`mul` enclosure is its
//! natural second target.
//!
//! # What stays out of Kani deliberately
//!
//! The elementary domain declines (`recip` astride zero, `sqrt` below zero, `ln`
//! at or below zero, `exp` overflow) are not Kani targets: each decline predicate
//! reads the form's reduced endpoints, and computing them on any nondegenerate
//! form runs fixture arithmetic, exactly what CBMC cannot handle. Symbolic
//! centers reach only the trivial point-form paths, which prove nothing about the
//! declines. Those declines stay with the unit and property tests; this is the
//! honest boundary of ADR-0002's "discrete domain logic" promise over this
//! fixture.
//!
//! The addition merge's exact-zero drop path (the `if !z.is_zero()` guard on a
//! shared symbol whose coefficients cancel) is expected *unreachable* over the
//! fixture rather than covered: `add_down`/`add_up` step outward unconditionally,
//! so `add_down(c, -c)` is `(-c + c).next_down()`, a subnormal, never an exact
//! zero. The fixture's outward step turns an exact cancellation into dust, so the
//! kept-coefficient is never dropped and never zero. A correctly-rounded backend
//! could reach the drop; the merge harness proves the shape invariant that holds
//! either way (no zero coefficient survives), and this note records why the drop
//! itself carries no `kani::cover`.
//!
//! # Stop loss
//!
//! Pre-committed per ADR-0006: a harness that does not discharge in about two
//! minutes gets its bounds (term counts, id bound, budget bound) shrunk once,
//! then is cut with the gap recorded here. The unwind bounds below are sized to
//! the maximum term counts the inputs admit (`add` leaves at most
//! `3 + 3 + 1 = 7` terms; the condensation form has three); a bound proving too
//! costly in CI is the trigger to shrink.

use alloc::vec::Vec;

use crate::form::{AffineForm, Term};
use crate::symbol::{NoiseSymbol, SymbolSource};

/// The representation invariant, evaluated for the model checker: deviation
/// terms strictly ascending by symbol id, with no zero coefficient.
fn invariant_holds(terms: &[Term<f64>]) -> bool {
    let mut previous: Option<u64> = None;
    for t in terms {
        if t.coeff() == 0.0 {
            return false;
        }
        let id = t.symbol().id();
        if let Some(p) = previous {
            if p >= id {
                return false;
            }
        }
        previous = Some(id);
    }
    true
}

/// Build an operand of `n` terms (`n <= 3`) from symbolic ids and concrete
/// coefficients. Fully unrolled so no loop bound is needed. The caller assumes
/// the ids strictly ascend, so the result satisfies the representation invariant
/// the merge relies on.
fn operand(n: u64, ids: [u64; 3], coeffs: [f64; 3]) -> AffineForm<'static, f64> {
    let mut terms: Vec<Term<f64>> = Vec::new();
    if n >= 1 {
        terms.push(Term::new(NoiseSymbol::from_raw(ids[0]), coeffs[0]));
    }
    if n >= 2 {
        terms.push(Term::new(NoiseSymbol::from_raw(ids[1]), coeffs[1]));
    }
    if n >= 3 {
        terms.push(Term::new(NoiseSymbol::from_raw(ids[2]), coeffs[2]));
    }
    AffineForm::from_parts(0.0, terms)
}

// --- Group 1: the symbol source ---

/// From a symbolic start, `fresh` issues strictly increasing, never-repeating
/// ids over a bounded run.
#[kani::proof]
fn fresh_is_strictly_increasing() {
    let start: u64 = kani::any();
    // Leave headroom so the bounded run cannot exhaust the counter; exhaustion is
    // the dedicated should_panic harness below.
    kani::assume(start <= u64::MAX - 4);
    let mut src = SymbolSource::unscoped_at(start);
    let a = src.fresh();
    let b = src.fresh();
    let c = src.fresh();
    let d = src.fresh();
    kani::assert(a.id() == start, "the first fresh id is the start");
    kani::assert(
        a.id() < b.id() && b.id() < c.id() && c.id() < d.id(),
        "fresh ids strictly increase",
    );
    kani::assert(
        b.id() == start + 1 && c.id() == start + 2 && d.id() == start + 3,
        "fresh ids never skip or repeat",
    );
}

/// Resuming an unscoped source at a saved `next_id` preserves freshness: the
/// next id issued exceeds every id issued before the save.
#[kani::proof]
fn unscoped_at_resumption_is_fresh() {
    let start: u64 = kani::any();
    kani::assume(start <= u64::MAX - 4);
    let mut src = SymbolSource::unscoped_at(start);
    let a = src.fresh();
    let b = src.fresh();
    let saved = src.next_id();
    // Persist the counter and resume; the restored source must not re-issue a
    // live id.
    let mut resumed = SymbolSource::unscoped_at(saved);
    let next = resumed.fresh();
    kani::assert(
        next.id() == saved,
        "resumption issues exactly the saved counter",
    );
    kani::assert(
        next.id() > a.id() && next.id() > b.id(),
        "resumption never re-issues a live id",
    );
}

/// `fresh` at the exhausted counter panics rather than wrapping onto a live id.
#[kani::proof]
#[kani::should_panic]
fn fresh_at_exhaustion_panics() {
    let mut src = SymbolSource::unscoped_at(u64::MAX);
    // The first call reads `u64::MAX`, then `checked_add(1)` overflows and the
    // `expect` panics, aborting rather than aliasing symbol zero.
    let _ = src.fresh();
}

// --- Group 2: from_raw_parts validation ---

/// `from_raw_parts` accepts exactly the faithful serializations (finite center
/// and coefficients, strictly ascending ids), and an accepted form satisfies the
/// representation invariant with zero coefficients dropped.
#[kani::proof]
#[kani::unwind(4)]
fn from_raw_parts_accepts_exactly_faithful_input() {
    let center: f64 = kani::any();
    let id0: u64 = kani::any();
    let id1: u64 = kani::any();
    let id2: u64 = kani::any();
    let c0: f64 = kani::any();
    let c1: f64 = kani::any();
    let c2: f64 = kani::any();

    let raw = [(id0, c0), (id1, c1), (id2, c2)];
    let ascending = id0 < id1 && id1 < id2;
    let finite = center.is_finite() && c0.is_finite() && c1.is_finite() && c2.is_finite();

    let result = AffineForm::<f64>::from_raw_parts(center, raw);
    kani::assert(
        result.is_some() == (ascending && finite),
        "acceptance is exactly strictly-ascending ids with a finite center and coefficients",
    );

    if let Some(form) = result {
        kani::assert(
            invariant_holds(form.terms()),
            "an accepted form satisfies the representation invariant",
        );
        // Zero coefficients are dropped: the term count is the number of nonzero
        // inputs (all coefficients are finite on this branch).
        let nonzero = usize::from(c0 != 0.0) + usize::from(c1 != 0.0) + usize::from(c2 != 0.0);
        kani::assert(
            form.num_terms() == nonzero,
            "zero coefficients are dropped, nonzero ones kept",
        );
    }
}

// --- Group 3: the addition merge invariant ---

/// `add` over operands with symbolic ids and concrete coefficients preserves the
/// representation invariant, for every interleaving of the ids.
#[kani::proof]
#[kani::unwind(8)]
fn add_preserves_the_merge_invariant() {
    let na: u64 = kani::any();
    let nb: u64 = kani::any();
    kani::assume(na <= 3 && nb <= 3);

    let ax0: u64 = kani::any();
    let ax1: u64 = kani::any();
    let ax2: u64 = kani::any();
    let bx0: u64 = kani::any();
    let bx1: u64 = kani::any();
    let bx2: u64 = kani::any();

    // Bound the ids so the source's error symbol can sit above them all, and
    // assume each operand's ids strictly ascend (its representation invariant).
    const BOUND: u64 = 64;
    kani::assume(ax0 < BOUND && ax1 < BOUND && ax2 < BOUND);
    kani::assume(bx0 < BOUND && bx1 < BOUND && bx2 < BOUND);
    if na >= 2 {
        kani::assume(ax0 < ax1);
    }
    if na == 3 {
        kani::assume(ax1 < ax2);
    }
    if nb >= 2 {
        kani::assume(bx0 < bx1);
    }
    if nb == 3 {
        kani::assume(bx1 < bx2);
    }

    // Concrete coefficients keep the merge's float arithmetic out of the symbolic
    // domain (ADR-0003). Every pairwise sum of the two coefficient sets is
    // nonzero, so no shared-symbol coefficient cancels; the merge invariant holds
    // without exercising the (fixture-unreachable) exact-zero drop path.
    let a = operand(na, [ax0, ax1, ax2], [1.0, 2.0, 4.0]);
    let b = operand(nb, [bx0, bx1, bx2], [1.0, -3.0, 5.0]);

    // A source above every operand id: `add` always mints one error symbol (the
    // fixture's `add_err(0, 0)` leaves subnormal dust), and placing it above the
    // ids keeps the result strictly ascending.
    let mut src = SymbolSource::unscoped_at(BOUND);
    let sum = a.add(&b, &mut src);

    kani::assert(
        invariant_holds(sum.terms()),
        "add leaves terms strictly ascending with no zero coefficient",
    );

    // Defend against vacuity: the shared-symbol (Equal) branch and a
    // disjoint-symbol interleaving must both be reachable.
    kani::cover!(
        na >= 1 && nb >= 1 && ax0 == bx0,
        "the shared-symbol merge branch is reachable"
    );
    kani::cover!(
        na >= 1 && nb >= 1 && ax0 < bx0,
        "a disjoint-symbol interleaving is reachable"
    );
}

// --- Group 4: the condensation count logic ---

/// Condensation bounds the term count by `max(budget, 1)`, returns a within-
/// budget form unchanged spending no fresh symbol, and keeps its terms sorted.
#[kani::proof]
#[kani::unwind(8)]
fn condense_bounds_terms_and_preserves_order() {
    let budget: usize = kani::any();
    kani::assume(budget <= 4);

    // A fixed form of three ascending, distinct, nonzero concrete terms; only the
    // budget is symbolic, so the fold arithmetic stays concrete (ADR-0003).
    let terms: Vec<Term<f64>> = alloc::vec![
        Term::new(NoiseSymbol::from_raw(0), 3.0),
        Term::new(NoiseSymbol::from_raw(1), -2.0),
        Term::new(NoiseSymbol::from_raw(2), 1.0),
    ];
    let form = AffineForm::from_parts(0.0, terms);
    let mut src = SymbolSource::unscoped_at(3);
    let before = src.next_id();

    let condensed = form.condense(budget, &mut src);

    let cap = if budget == 0 { 1 } else { budget };
    kani::assert(
        condensed.num_terms() <= cap,
        "condensed term count is at most max(budget, 1)",
    );

    if form.num_terms() <= cap {
        kani::assert(
            condensed.num_terms() == form.num_terms(),
            "a within-budget form keeps its term count",
        );
        kani::assert(
            src.next_id() == before,
            "a within-budget form spends no fresh symbol",
        );
    }

    kani::assert(
        invariant_holds(condensed.terms()),
        "kept terms stay sorted with no zero coefficient",
    );

    // Defend against vacuity: both the over-budget fold and the within-budget
    // identity branch must be reachable (the form has three terms).
    kani::cover!(budget < 3, "the over-budget fold branch is reachable");
    kani::cover!(
        budget >= 3,
        "the within-budget identity branch is reachable"
    );
}
