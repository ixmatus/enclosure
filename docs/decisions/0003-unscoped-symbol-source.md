# ADR-0003: The unscoped symbol source, an escape hatch from the lifetime brand

**Status:** Accepted
**Date:** 2026-06-10

## Context

The lifetime brand on `SymbolSource<'id>` and `AffineForm<'id, F>` makes
cross-source combination a compile error, which is the right default: colliding
symbol ids from independent sources would be treated as correlated, and
spurious cancellation silently breaks the enclosure guarantee. The brand also
confines every form to its `with_source` scope, so a form cannot outlive the
closure that built it. That confinement is exactly wrong for one real consumer
shape: a calculator whose stack values persist indefinitely across operations,
sessions, and a battery-backed snapshot. The SMIL calculator's `Enclosed`
number type carries an affine form as its representation, and those forms must
live on a stack, in storage registers, and through power cycles.

The alternatives considered: strip the brand entirely (every consumer loses
the compile-time guarantee, and a deliberate, reviewed design is overturned
for one consumer's needs); a runtime source tag checked on every combination
(every operation pays a check, and the failure becomes a runtime error in
exactly the code that wanted static assurance); or an additive escape hatch
that leaves the branded world untouched.

## Decision

**`SymbolSource::unscoped() -> SymbolSource<'static>`, an additive escape
hatch.** All `'static` brands unify, so forms built from an unscoped source
are ordinary `AffineForm<'static, F>` values that can be stored, cloned, and
combined freely — including, deliberately, with forms from a *different*
unscoped source, which the compiler can no longer reject. The soundness
obligation the brand normally discharges becomes a documented caller
invariant: **one unscoped source per universe of values that can ever meet.**
A consumer with exactly one engine, and therefore exactly one source, makes
collisions impossible by construction; that structural fact is the only
license for using the hatch.

**Persistence is part of the contract.** `next_id()` exposes the counter and
`unscoped_at(next)` resumes it. A caller that persists forms must persist the
counter atomically with every form referencing the source's symbols, and must
restore them together: resuming below a surviving form's greatest id would
let `fresh` re-issue a live symbol. Resetting the counter requires relabeling
or reducing every live form first; relabeling forgets correlation, which only
widens future combinations and never breaks enclosure.

**`from_raw_parts` is the deserialization companion, restricted to
`'static`.** `AffineForm<'static, F>::from_raw_parts(center, (id, coeff)
pairs)` rebuilds a previously serialized form, validating what a faithful
serialization guarantees (finite values, strictly ascending ids) and rejecting
anything else. It is defined only on the `'static` form, so the branded world
stays airtight: nothing rebuilt from raw parts can ever enter a `with_source`
scope, and `with_source` remains the canonical API for any computation that
fits a scope.

## Consequences

- The branded API is unchanged; existing consumers and the compile-fail
  guarantees they rely on are untouched. The hatch is opt-in and its
  obligations are stated on the constructor, not discovered in review.
- The single-source invariant moves from the compiler to one auditable site
  in the consumer (the calculator engine owns one `SymbolSource<'static>`),
  with the persistence rules stated here and on `unscoped`.
- Misuse (two unscoped sources whose forms meet) is undetectable at compile
  time by design; the documentation names the failure mode in the same words
  the brand's rationale used, so the trade is visible.
- The serialization path makes the form's representation (center plus sorted
  terms) a de facto stable surface for consumers that persist forms; changes
  to the representation invariant now have a compatibility dimension.

## References

- `crates/affine-arith/src/symbol.rs` (`unscoped`, `unscoped_at`, `next_id`),
  `crates/affine-arith/src/form.rs` (`from_raw_parts`).
- ADR-0001 (the layering and the brand's original rationale); the SMIL
  calculator's decision log (ADR-0055 there) for the consumer-side surface.
- Tests: `crates/affine-arith/tests/persistent_fixture.rs`.
