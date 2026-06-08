# ADR-0002: The RoundFloat trait shape, and design-now-extract-later

**Status:** Accepted
**Date:** 2026-06-07

## Context

The crate is generic over its float backend so the same interval logic serves
ferrodec `Decimal128` (a fixed-width decimal, the calculator's float, on stable
Rust) and pfloat's arbitrary-precision floats (on nightly Rust), as well as an
`f64` verification fixture. It needs a trait expressing the directed rounding
interval arithmetic depends on.

Two shapes were available. A mode-parameterized method `op(self, rhs, mode)`
mirrors what both ferrodec and pfloat already expose, and matches pfloat-ball's
`RealScalar`. Split-direction methods `add_down` and `add_up` name the rounding
direction at the call site instead.

A further question: the sibling crate pfloat-ball already has a trait
(`RealScalar`) that embodies almost exactly this contract, but it is sealed by
design so a ball can never wrap an unverified float, and it is bound to pfloat's
own rounding-mode and status types. It cannot be reused directly by ferrodec or
by an `f64` fixture.

## Decision

Define `RoundFloat` with split-direction methods (`add_down` / `add_up` and kin),
not a mode parameter. Directionality in outward rounding is static: a lower
endpoint always rounds down, an upper always up. Naming the direction at the call
site makes the rigor-critical four-corner multiplication impossible to mis-round,
keeps the trait decidable for the model checker, and keeps any backend's
rounding-mode enum out of the shared seam (ferrodec's and pfloat's are distinct
types, so a shared mode parameter could not span both).

Author `RoundFloat` in this crate now, deliberately as the common core of
pfloat-ball's `RealScalar` arithmetic and what inf-sup needs, so that it is the
prototype of a future shared contract rather than a divergent second abstraction.

Defer extraction. Once both enclosure crates are in use and the shape is proven
in two consumers, extract `RoundFloat` into a tiny stable foundation crate
(working name `round-float`) that ferrodec and pfloat implement and that both
enclosure crates consume, and refactor pfloat-ball's `RealScalar` to
`RoundFloat + Sealed + the extras it adds`. The seal is preserved as a
supertrait; the arithmetic core is shared. This is a deliberate move taken after
the abstraction is validated, not speculative upfront framework work.

## Consequences

- The four-corner rule is written against `*_down` / `*_up`; a corner cannot be
  rounded the wrong way by passing the wrong mode.
- ferrodec is never expanded: its instance is always provided through a newtype in
  the consuming crate, before and after extraction.
- During this crate's early life there are two directed-rounding traits in the
  family (this one and pfloat-ball's `RealScalar`). That duplication is temporary
  and tracked; convergence is the planned end state.
- Extraction is a move, not a redesign, because the trait was shaped for it from
  the start.

## References

- `src/round.rs`, `src/spec.rs` (Laws 3 and 4).
- pfloat-ball `RealScalar` (the sealed sibling contract).
- ADR-0003 (the f64 fixture and what verification runs over).
