# ADR-0006: The affine verification lane

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-16

## Context

affine-arith P4 (enc-2qf.4) closes the epic with a dedicated verification lane:
property-tested enclosure across whole computations, tightness against interval
arithmetic on correlated expressions, and model checking on the finite and
discrete parts. This record fixes the lane's seams so the implementation is
mechanical against it.

Three constraints shape the design.

**What P1 through P3 already carry.** Five property lanes exist
(`enclosure_fixture`, `ops_fixture`, `elementary_fixture`,
`persistent_fixture`, `condense_fixture`) plus in-module condensation tests.
They pin per-operation enclosure at box corners and sampled grids, the
`x - x` collapse against the original width, the dedicated square against the
generic self-multiply, a multi-symbol elementary sweep over joint noise
assignments, the serialization round-trip, and the pointwise condensation
superset. The `elementary-oracle` crate separately certifies the `exp`/`ln`
bounds against correctly-rounded pfloat-libm values (ADR-0002). P4 must not
duplicate any of this; its scope is the delta.

**What ADR-0002 promised this lane.** Two obligations were explicitly deferred
here: in-crate model checking of the discrete domain logic, and a compile-fail
guard for the cross-source brand.

**What interval-1788 ADR-0003 found.** CBMC cannot discharge the fixture's
directed rounding on symbolic floats: `next_up`/`next_down` transmute `f64` to
its bit pattern and branch on it, and the resulting formula is intractable even
for one two-operand operation. Any Kani harness here must keep float arithmetic
concrete or absent; the numeric enclosure theorem is not a Kani target over
this fixture. The abstract-axiom backend (ADR-0003's alternative, tracked as
its own task) remains the future route to a machine-checked enclosure; when it
lands, the affine `add`/`mul` enclosure becomes its natural second target.

## Decision

Four lanes, each with a named home.

### Lane 1: composition enclosure (`tests/composition_fixture.rs`)

Property-test enclosure over whole computations rather than single operations:
random expression programs over a pool of input forms, drawn from an op
alphabet of `add`, `sub`, `negate`, `scale`, `mul`, `sqr`, `recip`, `sqrt`,
`exp`, `ln`, and `condense` (random budget of at least one, applied
mid-program so folded correlations flow through later ops).

**Generation keeps programs in domain by construction.** Input pools use
positive centers and moderate radii so `recip`/`sqrt`/`ln` declines are rare;
an op returning `None` is skipped and the program continues. To prevent silent
coverage collapse, each case asserts its program applied at least one
nonlinear op; the generator's job is to make that assertion nearly always
hold rather than lean on proptest discards.

**The oracle is pointwise, in plain `f64`.** Sample joint assignments of the
input forms' base symbols: always the all-plus-one and all-minus-one corners
(where an enclosure bound binds hardest), plus random interior assignments.
Map each assignment to concrete input values, evaluate the same program in
plain `f64`, and assert the affine result's reduction contains the oracle
value. The justifying argument, which belongs in the lane's module docs: fresh
symbols minted mid-program (rounding error, bilinear remainders, condensation
tails) only widen the result's value set, so for every base-symbol assignment
the true pointwise value stays enclosed regardless of how the minted symbols
are set, and the reduction encloses the value set over all symbols.

**A two-tier arbiter separates oracle noise from confirmed bugs.** The plain
`f64` oracle evaluates in round-to-nearest, so its value can differ from the
true real value by accumulated ulps. On a containment failure, re-evaluate the
program at the same point inputs through interval-1788 point intervals, which
yields a guaranteed enclosure of the true value. If that oracle interval is
disjoint from the affine reduction, the enclosure bug is confirmed; if it
overlaps, the failure is oracle noise at the ulp scale and gets a written
note, not a fix.

**Representation invariant as cheap insurance.** A test-support helper asserts
after every op that terms are strictly ascending by symbol id with no zero
coefficient, so a merge bug surfaces at the op that introduced it rather than
downstream as unexplained width.

### Lane 2: tightness pins against interval arithmetic (`tests/tightness_fixture.rs`)

A named catalogue of correlated expressions, each computed twice from
identical inputs: once through interval-1788 directly, once through affine
forms reduced at the end. The initial catalogue, each entry commented with why
correlation wins there:

1. `x - x` (the canonical collapse; interval doubles the width).
2. `(x + y) - x` (partial cancellation; interval pays `2w(x) + w(y)`).
3. `x * (1 - x)` on a range inside `[0, 1]` (anticorrelated factors).
4. `x*x - 2x` via `sqr`, `scale`, `add` (a polynomial whose interval
   evaluation forgets that both terms move together).
5. `x / (1 + x)` on a positive range, via `recip` and `mul` (a correlated
   quotient chain).

Two guards per expression, bucketed per the workspace regression-guard rule
(no aggregate floor across the catalogue; compensating regressions hide
there):

- A property over random ranges: affine width at most interval width plus an
  ulp-scale dust allowance (both routes run the sound-not-tight fixture, and
  at degenerate widths their dust is incomparable).
- A fixed-seed pin of the measured win ratio per expression, set at
  implementation time from measurement with stated headroom. A pin at the
  measured value is a margin at the floor; the headroom and its reason go in
  the comment.

One further test documents the non-goal: an uncorrelated `mul` where interval
arithmetic is tighter than affine (the four-corner product is exact where the
affine bilinear remainder is conservative), pinned as expected behavior so
nobody later "fixes" it and so this lane never claims universal affine
superiority.

Pins are fixture-relative: ratios measured over the sound-not-tight fixture do
not transfer numerically to a correctly-rounded backend (they should only
improve there; the pin comments state the direction).

### Lane 3: Kani on the discrete parts (`src/kani_harness.rs`, `#[cfg(kani)]`)

Mirror interval-1788's placement, feature gating (`--features fixture`), and
its loud caveat: the numeric enclosure is not proven here, per ADR-0003's
finding. Harnesses keep float arithmetic concrete or absent; the symbolic
variables are ids, counts, and budgets.

1. **Symbol source.** From a symbolic starting counter: a bounded run of
   `fresh` calls issues strictly increasing, never-repeating ids;
   `unscoped_at(next_id())` resumption preserves freshness. A
   `should_panic` harness pins that `fresh` at the exhausted counter panics
   rather than wrapping onto a live id.
2. **`from_raw_parts` validation.** Over a bounded array of symbolic
   `(u64, f64)` pairs with coefficients constrained only by predicates
   (`is_finite`, `is_zero`, cheap bit tests): acceptance exactly when ids are
   strictly ascending and values finite, and the accepted form satisfies the
   representation invariant with zero coefficients dropped.
3. **Merge invariant.** `add` over operands with symbolic symbol ids, concrete
   coefficients, and bounded term counts (at most three per side): the result
   is strictly ascending with no zero coefficient, for every interleaving of
   ids. `kani::cover` statements defend against vacuity: the shared-symbol
   branch must be coverable. The exact-zero drop path is expected
   unreachable over the fixture (its unconditional outward step turns an
   exact cancellation into dust) and is documented as such rather than
   covered.
4. **Condensation count logic.** Symbolic budget, bounded term count, concrete
   coefficients: the condensed term count is at most `max(budget, 1)`, a form
   within budget returns unchanged spending no fresh symbol, and kept terms
   stay sorted.

The elementary domain declines (`recip` astride zero, `sqrt` below zero, `ln`
at or below zero, `exp` overflow) stay with the existing unit and property
tests, not Kani: the decline predicate reads reduced endpoints, and computing
them on any nondegenerate form runs fixture arithmetic; symbolic centers reach
only the trivial point-form paths. This is the honest boundary of ADR-0002's
"discrete domain logic" promise over this fixture, and the harness module docs
say so.

Stop loss, pre-committed: a harness that does not discharge in about two
minutes locally gets its bounds shrunk once, then is cut, and the gap is
recorded in the harness module docs; the writeup is the deliverable.

Mechanics: add `unexpected_cfgs = { level = "warn", check-cfg = ["cfg(kani)"] }`
to the crate's `[lints.rust]`, and a second step in the CI `kani` job running
`-p affine-arith --features fixture`.

### Lane 4: the brand compile-fail guard (doc test on `with_source`)

ADR-0002's promised guard, at zero new dependencies: a `compile_fail` doc test
alongside the existing passing `with_source` doctest, identical to it except
that a form from one scope is combined with the source of another. A
`compile_fail` test passes on any compilation error, so the snippet must stay
a minimal edit of the known-good one, keeping the brand mismatch the only
plausible failure. If that precision ever proves insufficient, `trybuild` is
the named escalation; it is deliberately not taken now (a dev-dependency where
a doctest suffices).

### Out of scope, with pointers

- The numeric enclosure by Kani: ADR-0003's finding; the abstract-axiom
  backend is the tracked route.
- Tightness against a correctly-rounded backend: a backend property; the
  oracle lane already anchors `exp`/`ln`.
- The `sqrt` zero-touch refinement: enc-fj5.

## Consequences

- The epic closes with composition-level enclosure evidence, per-expression
  tightness pins, a machine-checked discrete skeleton, and the brand guard,
  at zero new dependencies. The README disclosure's soundness claims gain
  their named backing.
- The three most plausible failures of this design, inverted up front:
  1. **The oracle is blind below an ulp.** A rounding-direction bug whose loss
     is smaller than the oracle's own evaluation noise can pass. The corner
     assignments (where bounds bind hardest) are always sampled and the
     disjointness arbiter separates noise from confirmed bugs, but a sub-ulp
     boundary loss at a non-corner extremum remains the residual risk; it is
     the same failure mode the README disclosure names, and this lane narrows
     it rather than eliminates it.
  2. **A green `kani` CI job reads as "enclosure proven".** It is not: Kani
     here covers symbol freshness, serialization validation, merge shape, and
     condensation counts. The caveat lives in the harness module docs and in
     this record; anyone citing the badge cites the caveat.
  3. **Vacuous merge proofs.** Path constraints could quietly make the
     interesting branches unreachable, proving properties of programs that
     never run. The `kani::cover` statements exist so vacuity surfaces as an
     unsatisfied cover in CI rather than as silence.
- The composition lane is the slowest property lane; its case count is budgeted
  so the suite stays inside the crate's current CI envelope, and the count is a
  named constant with the budget reasoning next to it.
- The tightness catalogue is a curated set, not a search: an expression class
  where affine loses that is absent from the catalogue stays invisible. The
  documented uncorrelated-`mul` pin partially covers this by stating the known
  loss class.

## Alternatives considered

- **The rigorous interval oracle as the primary composition check.** Rejected:
  asserting the reduction contains the oracle's whole interval falsely fails
  at the ulp scale, since both routes carry comparable outward dust. Demoted
  to the failure arbiter, where its rigor is decisive and its strictness is
  safe.
- **`trybuild` for the brand guard.** Rejected for now: a dependency where a
  doctest suffices; named as the escalation path.
- **Symbolic-coefficient merge proofs.** Requires the abstract-axiom
  `RoundFloat` backend; deferred with it (ADR-0003 alternative), not re-scoped
  here.
- **An aggregate tightness floor across the catalogue.** Rejected: floors on a
  total admit silent compensating regressions; each expression keeps its own
  pin.

## References

- Beads: enc-2qf.4 (this lane), enc-2qf (the epic), enc-fj5 (the excluded
  `sqrt` refinement).
- interval-1788 ADR-0003: the fixture verification strategy and the CBMC
  intractability finding this design inherits.
- Workspace ADR-0002: the elementary functions and the two obligations
  deferred to this lane; ADR-0003: the unscoped source contract behind the
  persistence surface; ADR-0004: the condensation superset argument and the
  pointwise-oracle precedent (`tests/condense_fixture.rs`).
- de Figueiredo & Stolfi, "Affine Arithmetic: Concepts and Applications"
  (2004); Comba & Stolfi (1993). Registry: `docs/references/`.
- Existing lanes: `crates/affine-arith/tests/*_fixture.rs`; the harness
  precedent `crates/interval-1788/src/kani_harness.rs`; CI:
  `.github/workflows/ci.yml` (the `kani` job).
