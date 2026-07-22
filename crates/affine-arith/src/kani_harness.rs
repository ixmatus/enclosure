//! Kani proof harnesses for the discrete parts of affine arithmetic, over the
//! `f64` fixture.
//!
//! These discharge the finite, set-based logic that is easy to get wrong and
//! carries no directed-rounding arithmetic: noise-symbol freshness, the
//! serialization validation of [`from_raw_parts`](crate::AffineForm::from_raw_parts),
//! the shape invariant the addition merge must preserve, and the term-count
//! logic of condensation. The symbolic variables are ids and counts;
//! coefficients are concrete, and the condensation budget is covered by an
//! exhaustive concrete case split rather than a symbolic value (see the stop
//! loss below), so CBMC never has to reason about the fixture's
//! `next_up`/`next_down` bit manipulation over symbolic data.
//!
//! Run with `cargo kani -p affine-arith --features fixture`; gated on `fixture`
//! for the `f64` instance of [`RoundFloat`](round_float::RoundFloat).
//!
//! # The from_raw_parts trio
//!
//! The serialization validation splits across three harnesses, one property
//! each, so CBMC gives each its own process and the two cheap ones attest in CI
//! while the costly one stays local. `from_raw_parts_acceptance_is_exact` carries
//! property 1 (acceptance is exactly a finite center and coefficients with
//! strictly ascending ids). `from_raw_parts_drops_zero_coefficients` carries
//! property 3 (an accepted form's term count is its nonzero coefficient count).
//! `from_raw_parts_accepted_form_satisfies_invariant` carries property 2 (an
//! accepted form meets the representation invariant); it scans the returned heap
//! terms and runs 15.2 million SAT variables, so it is verified locally and left
//! off the CI harness list. Its doc comment carries the CI NOTICE and the
//! measured scaling law.
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
//!
//! The stop loss has fired twice (2026-07-17, beads enc-otn and enc-9nq),
//! both times for the same root cause in different clothes: a symbolic value
//! that selects, indexes, or sizes concrete float data puts the floats
//! themselves on the symbolic path, no matter how concrete the coefficients
//! are.
//!
//! First firing (condensation): the harness drew its budget from `kani::any`,
//! and the symbolic budget leaked into the float path through
//! `keep = budget - 1`. That index chose the fold slice, the truncation
//! length, and the length of the second sort, so the fixture's `add_up` ran
//! over symbolically selected operands and the sort ran over a symbolic
//! length, exactly the bit-blasting ADR-0003 found intractable. It never
//! discharged: wave one's CI run was cancelled at the six-hour limit, a
//! wave-four local run passed 150 CPU-minutes without finishing, and the
//! wave-four merge push hung CI the same way. The intervention re-encoded the
//! assumed budget domain (at most 4) as an exhaustive concrete case split
//! over the five budgets, preserving the property verbatim; it now discharges
//! in seconds.
//!
//! Second firing (the addition merge, found only after the first fix because
//! package runs check the condensation harness first and its hang shadowed
//! everything behind it): distinct concrete coefficients let the symbolic id
//! interleaving choose which values met in `add_up`, so the adds ran over
//! symbolically selected operands and passed ten minutes without
//! discharging. Two levers were pulled: the coefficients were made uniform
//! per operand, collapsing every selection to the same concrete pair while
//! leaving the merge's discrete choices, the actual property under proof,
//! fully symbolic; and the id bound was shrunk once from 64 to 8, which
//! preserves every reachable interleaving (six distinct ids fit under 8 with
//! every equality pattern) at half the bits per symbolic comparison. Neither
//! sufficed, and the timing observations along the way were contaminated by
//! CPU contention (an orphaned wave-four kani run and a concurrent gate from
//! another project); the deciding evidence is contention-free. First, the
//! solver's verdict exposed an unwind bound of 8 failing its own unwinding
//! assertions on the seven-term result scan, a genuine harness defect latent
//! since wave one. Second, with the bound corrected to 10 on an otherwise
//! idle machine, CBMC exhausts memory in propositional reduction: the error
//! accumulator threads the Equal branch, so symbolic interleavings feed an
//! ite tree into chained fixture `add_up` calls, ADR-0003's intractable
//! construct. The ladder exhausted, the harness was cut; the gap and the two
//! routes back are recorded at its former site (group 3 below).

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

/// A symbolic float class mapped to a concrete representative. `from_raw_parts`
/// and the invariant scan read a value only through `is_finite` and `is_zero`
/// (`== 0.0`), never its bits: the `f64` fixture implements both as the plain
/// intrinsic and a compare against zero (verified against round-float's
/// `f64_impl.rs`), so behavior is invariant under this quotient and four
/// representatives cover every branch a raw `f64` could take.
///
/// The class split costs almost nothing and buys almost nothing. An out of tree
/// ladder measured the acceptance check over raw symbolic `f64` at 127,260 SAT
/// variables and over the class split at 127,192, a gap of 68. The float bits
/// were never the footprint; the scan of the returned terms is, and its numbers
/// live on the invariant harness below.
fn class_rep(k: u8) -> f64 {
    match k {
        0 => 1.0,
        1 => 0.0,
        2 => f64::INFINITY,
        _ => f64::NAN,
    }
}

/// The symbolic inputs the three `from_raw_parts` harnesses share: a center and
/// three coefficients drawn as float classes, three ids bounded to 8, assembled
/// into the raw serialization and run through `from_raw_parts`. Each harness
/// reads one facet of the outcome, so the original single proof becomes three
/// proofs that assert one property apiece and take one CBMC process each.
struct RawPartsCase {
    result: Option<AffineForm<'static, f64>>,
    ascending: bool,
    finite: bool,
    coeffs: [f64; 3],
}

fn from_raw_parts_case() -> RawPartsCase {
    let kc: u8 = kani::any();
    let k0: u8 = kani::any();
    let k1: u8 = kani::any();
    let k2: u8 = kani::any();
    kani::assume(kc < 4 && k0 < 4 && k1 < 4 && k2 < 4);
    let center = class_rep(kc);
    let c0 = class_rep(k0);
    let c1 = class_rep(k1);
    let c2 = class_rep(k2);
    // The id logic is pure ordering (strict ascent or not), so ids under a small
    // bound realize every order pattern three ids can take; 8 gives slack for
    // equalities and both violation directions while keeping each id to three
    // symbolic bits.
    let id0: u64 = kani::any();
    let id1: u64 = kani::any();
    let id2: u64 = kani::any();
    kani::assume(id0 < 8 && id1 < 8 && id2 < 8);

    let raw = [(id0, c0), (id1, c1), (id2, c2)];
    let ascending = id0 < id1 && id1 < id2;
    let finite = center.is_finite() && c0.is_finite() && c1.is_finite() && c2.is_finite();
    let result = AffineForm::<f64>::from_raw_parts(center, raw);
    RawPartsCase {
        result,
        ascending,
        finite,
        coeffs: [c0, c1, c2],
    }
}

/// Property 1 of 3: `from_raw_parts` accepts exactly the faithful serializations,
/// a finite center and coefficients with strictly ascending ids. Reading the
/// outcome as a boolean touches no heap, so the formula stays near 129 thousand
/// SAT variables and this harness runs in CI.
#[kani::proof]
#[kani::unwind(4)]
fn from_raw_parts_acceptance_is_exact() {
    let case = from_raw_parts_case();
    kani::assert(
        case.result.is_some() == (case.ascending && case.finite),
        "acceptance is exactly strictly-ascending ids with a finite center and coefficients",
    );
}

/// Property 3 of 3: an accepted form drops its zero coefficients and keeps the
/// nonzero ones, so its term count equals the number of nonzero input
/// coefficients. Reading only the result length touches no heap scan, so the
/// formula stays near 128 thousand SAT variables and this harness runs in CI.
#[kani::proof]
#[kani::unwind(4)]
fn from_raw_parts_drops_zero_coefficients() {
    let case = from_raw_parts_case();
    if let Some(form) = case.result {
        let nonzero = usize::from(case.coeffs[0] != 0.0)
            + usize::from(case.coeffs[1] != 0.0)
            + usize::from(case.coeffs[2] != 0.0);
        kani::assert(
            form.num_terms() == nonzero,
            "zero coefficients are dropped, nonzero ones kept",
        );
    }
}

/// CI NOTICE (bead enc-tgw): this is the one property of the `from_raw_parts`
/// trio absent from the CI kani job's harness list. Property 2 of 3: an accepted
/// form satisfies the representation invariant, its terms strictly ascending by
/// id with no zero coefficient.
///
/// It is absent because it scans the returned terms, and scanning a heap `Vec`
/// whose length and contents are both symbolic is the cost the CI runner cannot
/// afford. The scan runs about 15.2 million SAT variables and 67 million clauses
/// and peaks between 14 and 23 GiB depending on the solver's search, the standard
/// runner's memory cliff that killed the wave-5b run by runner OOM. The container
/// is not the cost: an out of tree ladder put the allocator and the pushes near
/// 15 thousand variables and the acceptance check near 129 thousand, while the
/// same construction scanned back through the invariant hit 15.2 million. Walking
/// it differently does not rescue it, since a concrete index scan under a length
/// guard measured slightly larger than the iterator scan; the symbolic heap
/// buffer itself is the cost, not the traversal. The earlier note blamed bit
/// blasted `Vec` and allocator machinery, which the ladder falsified; the numbers
/// here are the correction.
///
/// The two cheap properties above attest in CI. This one is verified locally
/// (a minute or two) and rejoins CI when the runner grows or the model checker
/// gains a tighter encoding for scanning a bounded but symbolic length slice (the
/// upstream issue recorded in the bead). The CI badge does not attest this single
/// property until then.
#[kani::proof]
#[kani::unwind(4)]
fn from_raw_parts_accepted_form_satisfies_invariant() {
    let case = from_raw_parts_case();
    if let Some(form) = case.result {
        kani::assert(
            invariant_holds(form.terms()),
            "an accepted form satisfies the representation invariant",
        );
    }
}

// --- Group 3: the addition merge invariant (cut; the gap and the route back) ---
//
// The harness `add_preserves_the_merge_invariant` proved that `add` over
// operands with symbolic ids and concrete coefficients preserves the
// representation invariant for every interleaving of the ids. It was cut on
// 2026-07-18 (bead enc-9nq) after the stop-loss ladder ran out on clean
// evidence: with uniform per-operand coefficients (so no symbolic selection
// reaches `add_up`), the id bound shrunk 64 to 8, and the unwind bound
// corrected 8 to 10 (the original failed its own unwinding assertions on the
// seven-term result scan, latent since wave one), CBMC still exhausts memory
// in propositional reduction on an otherwise idle machine. The residue is the
// error accumulator: it threads the merge's Equal branch, so under symbolic
// interleavings its value is an ite tree feeding chained fixture `add_up`
// calls, ADR-0003's intractable construct at depth four.
//
// The gap: the merge-shape invariant over symbolic interleavings is not
// machine-checked. It is exercised by the `ops` unit tests and the
// composition property lane, and `from_raw_parts`'s validation harness above
// still proves the invariant's decidable core. The recorded routes back,
// either of which reopens this group: an exhaustive concrete enumeration of
// the id order types (every merge behavior is fixed by the order type of the
// two ascending tuples, so ascending tuples over a six-value universe cover
// all of them with wholly concrete data and trivial unwind), or ADR-0003's
// abstract-axiom backend, under which the fixture arithmetic itself turns
// symbolic-but-axiomatized and this harness's original shape becomes viable.

// --- Group 4: the condensation count logic ---

/// The condensation property at one concrete budget: the term count is bounded
/// by `max(budget, 1)`, a within-budget form is returned unchanged spending no
/// fresh symbol, an over-budget form condenses to exactly the cap spending
/// exactly one, and the result keeps its terms sorted.
///
/// The budget must be concrete: a symbolic budget flows through
/// `keep = budget - 1` into the fold slice, the truncation, and the second
/// sort, putting the fixture's `add_up` on the symbolic path (the blowup the
/// module-level stop loss records).
fn check_condense_at(budget: usize) {
    // A fixed form of three ascending, distinct, nonzero concrete terms, so
    // every fold the checker executes is over concrete data (ADR-0003).
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
    if form.num_terms() <= cap {
        kani::assert(
            condensed.num_terms() == form.num_terms(),
            "a within-budget form keeps its term count",
        );
        kani::assert(
            src.next_id() == before,
            "a within-budget form spends no fresh symbol",
        );
    } else {
        // The folded coefficient is a sum of strictly positive magnitudes
        // rounded up, so the fresh term survives `push_fresh`'s zero drop and
        // the count lands exactly on the cap.
        kani::assert(
            condensed.num_terms() == cap,
            "an over-budget form condenses to exactly max(budget, 1) terms",
        );
        kani::assert(
            src.next_id() == before + 1,
            "the fold spends exactly one fresh symbol",
        );
    }

    kani::assert(
        invariant_holds(condensed.terms()),
        "kept terms stay sorted with no zero coefficient",
    );
}

/// Condensation bounds the term count by `max(budget, 1)`, returns a within-
/// budget form unchanged spending no fresh symbol, and keeps its terms sorted;
/// exhaustive over the budget domain the original harness assumed
/// (`budget <= 4` against the fixed three-term form).
///
/// Budgets 0 through 2 take the over-budget fold and budgets 3 and 4 take the
/// within-budget identity branch unconditionally, so both branches are
/// exercised by construction and the vacuity covers of the symbolic encoding
/// are no longer needed: there is no `assume` left to make any call vacuous.
#[kani::proof]
#[kani::unwind(8)]
fn condense_bounds_terms_and_preserves_order() {
    check_condense_at(0);
    check_condense_at(1);
    check_condense_at(2);
    check_condense_at(3);
    check_condense_at(4);
}
