# ADR-0001: Transcendentals ride an extension trait, not the core

**Status:** Accepted
**Date:** 2026-06-09

## Context

`affine-arith` is adding the elementary functions `exp` and `ln` to its
nonlinear surface (Chebyshev affine approximation over a form's range). Building
the approximation evaluates the function at the range endpoints and at an
interior tangent point, each with directed rounding, so affine-arith needs
outward-rounded `exp` and `ln` from the float backend it is generic over.

`RoundFloat` does not provide them. It was shaped (interval-1788 ADR-0002, the
enclosure layering ADR-0001) as the common core of what outward-rounded
arithmetic needs from every backend: the four operations, square root,
negation, the classification predicates. `interval-1788` uses none of `exp`/`ln`
yet, and there is no value to evaluate them through in that crate. Two
considerations bear on where the new capability lands:

1. Not every backend can supply a correctly-rounded transcendental as cheaply as
   it supplies `add` or `mul`. The only production `RoundFloat` instance,
   ferrodec `Decimal128`, lives outside this workspace in the SMIL calculator. A
   bare required method added to `RoundFloat` is mandatory for every `impl`, so
   it would break that out-of-repo backend until SMIL supplied four new methods,
   a cross-repo break driven by an affine-arith feature.

2. `libm`'s `exp` and `log` are not correctly rounded. musl (the port `libm`
   tracks) lists the correctly-rounded functions, and `exp`/`log` are not among
   them; the contract is a goal of less than 1.5 ulp in round-to-nearest. `sqrt`
   is correctly rounded (IEEE 754 mandates it), which is why the fixture's
   `sqrt_down`/`sqrt_up` can step one float outward and be sound. `exp`/`ln`
   cannot share that treatment: a single outward step may not clear the error.

## Decision

**`exp` and `ln` ride a separate extension trait, `RoundTranscendental:
RoundFloat`, not the core.** The trait declares `exp_down`, `exp_up`, `ln_down`,
`ln_up` with the same split-direction, soundness-first contract as `RoundFloat`.
A backend opts in by implementing it; one that does not still satisfies
`RoundFloat` unchanged. affine-arith puts `recip`/`sqrt`/`sqr` (which need only
the core) on its `F: RoundFloat` block and `exp`/`ln` on a separate `F:
RoundFloat + RoundTranscendental` block, so the transcendental surface lights up
only for backends that can bound it soundly. ferrodec and the vendored snapshot
keep compiling untouched.

**The f64 fixture implements it with `libm` plus a derived relative margin.**
`exp_up`/`ln_up` widen the `libm` result by a relative factor of `8 * 2^-52`
(rounded outward), and step one float further out for an absolute floor; the
`down` companions mirror it. The relative factor dominates the 1.5 ulp goal with
more than fivefold headroom at every magnitude and, being relative rather than a
fixed step count, stays sound across a binade boundary, where the spacing just
below a power of two is half that just above. The trailing outward step covers a
subnormal result, whose ulp exceeds its own magnitude times `2^-52`. Underflow,
overflow, and out-of-domain arguments are handled explicitly (zero and the
smallest subnormal bound an underflowed `exp`; the largest finite float and
positive infinity bound an overflowed one; an out-of-domain `ln` passes the
`libm` sentinel through for the caller's `self > 0` guard).

## Consequences

- `RoundFloat` stays the minimal common core the layering ADR committed to;
  growth happens in a named sibling trait, exactly the "optional capability above
  the core" shape that ADR's closing scope note anticipated.
- The fixture's transcendental soundness rests on musl's stated accuracy goal,
  not on a proof, which is weaker than its `sqrt` soundness (correct rounding is
  guaranteed). The margin is generous to absorb a rare input that misses the
  goal, and the f64 instance is sound-not-tight by design, so the looseness costs
  only tightness. The assumption is discharged empirically by the nightly
  `pfloat-libm` oracle lane, which checks the fixture (and affine) bounds enclose
  a correctly-rounded reference over a large grid.
- A production backend earns a tight transcendental surface by implementing
  `RoundTranscendental` with its own correctly-rounded `exp`/`ln`; pfloat
  (correctly rounded, arbitrary precision) and ferrodec are the candidates, each
  on its own schedule.

## References

- enclosure ADR-0001 (the round-float layering and the "common core, no larger"
  framing this record extends).
- interval-1788 ADR-0002 (the `RoundFloat` trait shape: split directions,
  soundness over tightness).
- de Figueiredo and Stolfi, "Affine Arithmetic: Concepts and Applications"
  (2004): the Chebyshev elementary-function approximation that needs `exp`/`ln`.
- musl libc, "Mathematical Library" accuracy note: the correctly-rounded list
  (excluding `exp`/`log`) and the less-than-1.5-ulp goal for the rest.
