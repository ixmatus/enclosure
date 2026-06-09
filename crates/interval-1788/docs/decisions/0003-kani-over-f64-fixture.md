# ADR-0003: Verify over an f64 fixture; tightness over the real backend

**Status:** Accepted
**Date:** 2026-06-07

**Update (2026-06-08):** the f64 fixture this record describes moved to the
`round-float` foundation crate, behind its `f64` feature; the source paths below
(`src/f64_impl.rs`) are now round-float-relative. interval-1788 re-surfaces the
instance as its `fixture` feature, so the six structural Kani harnesses still run
via `cargo kani -p interval-1788 --features fixture`. The strategy is unchanged;
only the fixture's home moved. See the workspace record `enclosure` ADR-0001.

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

Prove the discrete, set-based case logic with Kani over this fixture: the
total-operation edge cases that are easy to get wrong (an empty operand absorbs,
a zero divisor gives empty, a reciprocal across zero gives Entire, a wholly
negative square root gives empty). These reach their result through case
selection and early returns, are heap free and bounded, and discharge quickly.

Do NOT attempt the numeric enclosure theorem with Kani over this fixture. The
finding (2026-06-08): the fixture's outward rounding uses `next_up` / `next_down`,
which transmute `f64` to and from its bit pattern and branch on it; CBMC
bit-blasts that into a formula too large to discharge even for one two-operand
operation, so every enclosure harness (including a two-variable singleton merge
gate) times out, while the arithmetic-free structural harnesses pass in seconds.
The enclosure theorem instead rests on the written argument in `spec` (the
`next_after` bracketing of a round-to-nearest result) and on the property tests,
which exercise the general interior-point enclosure and the singleton merge gate
over thousands of cases. The route to a machine-checked four-corner enclosure is
the abstract-axiom alternative below, tracked as its own task.

Verify tightness, and the enclosure of the round-to-nearest point result, with
property tests over a correctly-rounded backend, where the directed operations
return the nearest representable bound exactly. Tightness cannot be shown over the
fixture, because the fixture over-widens on purpose; it is a property of the
backend, checked there.

Cross-check the full conformance surface against the IEEE 1788 test vectors and a
trusted reference reached out of process on a separate lane.

## Consequences

- The discrete set-based logic is machine-checked: six harnesses (empty
  absorption for add and mul, division by exact zero, reciprocal of an exact zero
  and of a zero-straddling interval, square root of a wholly negative interval)
  pass in seconds.
- The numeric enclosure is NOT machine-checked over this fixture; it rests on the
  `spec` argument plus the property tests. A reader expecting "enclosure proven by
  Kani" must see this caveat, so it is stated in the harness module and here. The
  load-bearing claim has a written proof sketch and broad property coverage, not a
  discharged Kani proof, until the abstract-axiom approach lands.
- The fixture's looseness (sound, not tight) must be stated loudly, or a reader
  will mistake it for production behavior. It is in the module, the crate root,
  and `spec` Law 6. Tightness is a backend property, tested over the
  correctly-rounded backend, never over the fixture.
- Square root is left to property tests regardless: its fixture path goes through
  `libm::sqrt`, whose software implementation CBMC cannot discharge.

## Alternatives considered

- **Abstract-axiom Kani enclosure (the planned path to a proven four-corner).** A
  Kani-only backend whose directed operations return symbolic values constrained
  by the soundness inequality (`add_down(x, y) <= x + y <= add_up(x, y)`) rather
  than computed by nextafter. This removes the bit manipulation CBMC cannot
  handle, so the model checker reasons about the interval *algorithm* (endpoint
  selection, the four-corner min and max) given a sound backend, which is the more
  valuable theorem. Deferred to its own task because setting up the symbolic real
  operands for the four-corner case takes care to keep it both honest and bounded.

## References

- `src/f64_impl.rs`, `src/spec.rs` (Laws 2 and 6).
- rpn_engine `kani_proofs.rs` (the in-tree precedent for real-f64 proofs under
  CBMC, the basis for this strategy).
- IEEE 1788 conformance test vectors (ITF1788).
