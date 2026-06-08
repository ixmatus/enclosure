# ADR-0003: Verify over an f64 fixture; tightness over the real backend

**Status:** Accepted
**Date:** 2026-06-07

## Context

The load-bearing claim of a rigorous interval crate is the enclosure theorem: the
result of an operation contains the true result over every point of its inputs.
That claim deserves a machine-checked proof, not only tests.

The model checker (Kani over CBMC) can reason about `f64` bit for bit, but it
cannot model a multiword decimal such as ferrodec `Decimal128` or pfloat's
arbitrary-precision float; those are far beyond a tractable bit-vector decision
problem. So the proof cannot run over the production backends directly.

Separately, `f64` has no directed-rounding arithmetic in the standard library.

## Decision

Ship an `f64` instance of `RoundFloat` behind a `fixture` feature, used only for
verification and host tests. It computes each operation in round-to-nearest and
then steps one float outward with `next_down` or `next_up`. For a nearest result
`r` of an exact real `t`, `t` lies between the neighbors of `r`, so stepping
outward is always a correct bound. It widens by up to one unit in the last place
even when the exact result was representable. The fixture is therefore sound but
deliberately not tight.

Prove the enclosure theorem and the structural properties (the construction
invariant survives every operation, the decoration lattice propagates monotonically)
with Kani over this fixture. The harnesses are heap free and bounded.

Verify tightness, and the enclosure of the round-to-nearest point result, with
property tests over a correctly-rounded backend, where the directed operations
return the nearest representable bound exactly. Tightness cannot be shown over the
fixture, because the fixture over-widens on purpose; it is a property of the
backend, checked there.

Cross-check the full conformance surface against the IEEE 1788 test vectors and a
trusted reference reached out of process on a separate lane.

## Consequences

- The enclosure claim is machine-checked, on a float the checker can handle.
- The fixture's looseness must be stated loudly, or a reader will mistake it for
  production behavior. It is documented in the module, in the crate root, and in
  `spec` Law 6.
- There are two notions of correctness with two methods: soundness, proven over
  the fixture; tightness, tested over the real backend. They are not in tension;
  soundness is the obligation and tightness is the quality.
- Square root under the checker is the least certain item: CBMC's support for
  `f64` sqrt is weaker than for the four basic operations. If it proves
  intractable, the sqrt enclosure falls back to a multiplication-based model
  (`r*r` bounds `x`) or to property tests, with the Kani sqrt harness marked
  aspirational.

## References

- `src/f64_impl.rs`, `src/spec.rs` (Laws 2 and 6).
- rpn_engine `kani_proofs.rs` (the in-tree precedent for real-f64 proofs under
  CBMC, the basis for this strategy).
- IEEE 1788 conformance test vectors (ITF1788).
