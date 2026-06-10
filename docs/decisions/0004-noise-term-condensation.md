# ADR-0004: Noise-term condensation with a caller-chosen budget

**Status:** Accepted
**Date:** 2026-06-10

## Context

Every affine operation mints at most one fresh noise symbol, but the count
still grows monotonically over a computation, and a consumer that keeps forms
alive indefinitely needs the per-value memory bounded. The SMIL calculator is
the concrete forcing case: its `Enclosed` stack values are affine forms that
must fit a 2 KB battery-backed engine snapshot, so an unbounded term vector is
disqualifying, not merely untidy. Condensation was deliberately deferred when
the elementary surface landed ("decide when a real consumer hits the growth");
the consumer has now arrived.

## Decision

**`condense(&self, budget, src)` folds the smallest-magnitude terms into one
fresh symbol.** A form within the budget is returned unchanged. Otherwise the
`budget − 1` largest-magnitude terms keep their symbols (ties break toward the
older symbol, so the result is deterministic), and the tail is replaced by one
fresh symbol whose coefficient is the tail's summed magnitude, rounded up.
Soundness is the standard condensation argument (de Figueiredo & Stolfi 2004,
reducing the number of noise symbols), derived for the directed-rounding
setting: any joint noise assignment of the original form is realized by the
condensed form with `δ = (Σ_tail xⱼεⱼ)/c ∈ [−1, 1]`, so the condensed value
set is a superset and enclosure is preserved.

**The budget is a caller knob, and condensation is never automatic.** The
crate does not condense behind the caller's back: when to spend correlation
for memory is a consumer policy (the calculator condenses after each engine
operation to a per-build budget; a one-shot evaluation never needs to). A
`budget` of zero is treated as one, since a nondegenerate form needs at least
one term to carry its deviation.

**Largest-magnitude selection.** Keeping the largest coefficients preserves
the correlations that dominate future cancellation; the folded tail behaves
like interval arithmetic from then on. Width is unaffected up to rounding —
the condensed radius equals the original on a correctly-rounded backend — so
the cost of condensing is exclusively the tail's correlation.

**What the tests pin, and deliberately not more.** The property lane checks
the pointwise superset (sampled joint assignments of the original form land in
the condensed reduction), the budget, and width preservation up to rounding.
It deliberately does not assert `condensed.reduce() ⊇ original.reduce()`: the
two reductions accumulate their up-rounded radius sums in different orders
(magnitude-sorted versus id-sorted), so they are incomparable at the ulp level
even though the value sets nest. The property-test counterexample that
surfaced this is why the pinned property is the pointwise one.

## Consequences

- Persistent consumers get a hard per-value memory bound:
  `budget × sizeof(Term)` plus the center, which is what lets an affine-backed
  value fit a fixed snapshot region.
- Condensed forms lose tail correlations permanently; expressions that later
  cancel through a folded symbol widen to interval-like behavior. Consumers
  trade tightness for memory explicitly via the budget, and should condense as
  late and as wide as they can afford.
- The magnitude sort allocates a scratch copy of the term vector; condensation
  is O(n log n) in the term count. Acceptable for per-operation use at
  calculator scales; a streaming selection is a refinement if a consumer ever
  condenses forms with very large term counts.

## References

- de Figueiredo & Stolfi, "Affine Arithmetic: Concepts and Applications"
  (2004): the condensation construction. Derived from the paper's argument,
  not from any implementation.
- `crates/affine-arith/src/condense.rs`; tests in
  `crates/affine-arith/tests/condense_fixture.rs`.
- ADR-0003 (the unscoped source the persistent consumer pairs this with).
