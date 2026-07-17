# ADR-0003: Integer rounding rides an exact extension trait, `RoundInteger`

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-17

## Context

`interval-1788` is adding the IEEE 1788 integer-rounding point functions
(`floor`, `ceil`, `trunc`, `roundTiesToEven`, `roundTiesToAway`) on intervals.
Each takes the image of a monotone step function over `[a, b]` as
`[op(a), op(b)]`, so the interval layer needs the scalar operation from the
backend it is generic over. `RoundFloat` does not supply it, and `RoundFloat` is
frozen: it is the common core every backend implements, and the transcendental
growth work (workspace ADR-0005) settled that new capability lands in per-family
extension traits rather than in the core, so no shipped backend breaks.

Two facts shape where the capability lands. First, rounding a float to an
integral value is exact in every binary or decimal format: an integer inside the
contiguous-integer range is representable, and above that range every finite
value is already integral. There is no rounding error and so no lower/upper bound
to distinguish, unlike the directed arithmetic of `RoundFloat`. Second,
`RoundTrig` (ADR-0005) already carries its own exact `floor` for argument
reduction, so a naive "put floor on the trig trait" would couple integer rounding
to a backend's trig schedule.

## Decision

**Integer rounding rides a new extension trait `RoundInteger: RoundFloat`** with
five methods (`floor`, `ceil`, `trunc`, `round_ties_to_even`,
`round_ties_to_away`), each **exact and undirected**: no `_down`/`_up` split,
because an exact operation carries no rounding direction. This is the same shape
as `negate`, the existing exact undirected method on `RoundFloat`. A backend opts
in on its own schedule, as it does for `RoundTranscendental` and the ADR-0005
families; the `f64` fixture implements it by delegation to `libm` (`roundeven`
for ties-to-even, `round` for ties-to-away), since those results are exact.

**Rejected: growing `RoundFloat`.** It is frozen and its growth doctrine
(ADR-0005) is per-family traits. **Rejected: folding these onto `RoundTrig`.**
Integer rounding is an independent capability; a backend may want it without trig.

## Consequences

- A backend implementing both `RoundInteger` and `RoundTrig` supplies `floor`
  twice. The duplication is accepted, the same trade ADR-0005 makes for the pi
  pair: `RoundFloat` stays frozen and each capability trait is self-contained.
- The interval layer takes step-function images with no outward widening,
  relying on the documented exactness; the enclosure stays tight even on the
  otherwise sound-not-tight `f64` fixture.
- The trait is small and total; a future decimal or arbitrary-precision backend
  implements it with its own exact integer rounding, no proof obligation beyond
  "the result is the integral value the name promises".

## References

- `crates/round-float/docs/decisions/0001-round-transcendental-extension-trait.md`:
  the extension-trait rationale this reuses.
- `docs/decisions/0005-transcendental-growth-trig-hyp-pow.md`: the per-family
  capability-trait doctrine and the accepted `floor`/pi duplication trade.
- `crates/round-float/src/lib.rs` (`RoundFloat::negate`): the exact-undirected
  precedent.
- `docs/references/vendor/itf1788-framework/itl/libieeep1788_elem.itl`: the
  conformance vectors the interval point functions are checked against.
