# ADR-0004: Integer power rides an exact kernel behind `RoundPown`

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-22

## Context

The conformance lane falsified workspace ADR-0007's recorded assumption that
a directed repeated-squaring chain approaches tightness the corpus cannot
distinguish: 41 of 163 `minimal_pown_test` vectors fail bit-exactness over
`TightF64` (bead enc-5jj). Two mechanisms, both structural: the chain rounds
once per multiply, so error compounds for `|n| >= 3`; and the negative path
takes a set-level reciprocal of the magnitude image, so an overflowing
magnitude power saturates and collapses subnormal-grade results. `RoundFloat`
is frozen; capability grows in per-family extension traits (workspace
ADR-0005, round-float ADR-0003).

## Decision

**Integer power rides a new extension trait `RoundPown: RoundFloat`** with
`pown_down(self, n: i32) -> Self`, `pown_up(self, n: i32) -> Self`, and an
associated const `POWN_TIGHT_MAX: u32`. Point semantics are IEEE 754-2019
`pown` (in particular `pown(x, 0) == 1` for every `x` including NaN and the
infinities; zero bases with negative `n` follow the division semantics by
parity) under directed rounding. The contract: results are correctly rounded
(tightest) for `1 <= |n| <= POWN_TIGHT_MAX`, and sound directed bounds
beyond. Distinct from `RoundPow`, the real-exponent family.

**Both backends delegate to one exact integer kernel** in a private
round-float module. The kernel decomposes a finite nonzero base into sign,
integer significand `s < 2^53`, and exponent; raises `s` to `|n|` by repeated
exact multiplication of a fixed-width little-endian `u64` limb array by the
`u64` significand with `u128` accumulation (integer-exact, no rounding
anywhere in the chain, `|n| - 1` multiplications); and rounds once at the end:
the top 53 bits with a sticky bit for positive `n`, an exact long division
`2^m / s^|n|` with remainder-driven sticky for negative `n`. Overflow and
subnormal underflow ride exact exponent arithmetic: the result exponent is
`|n|` times the base exponent plus the normalization shift, computed in `i64`;
a result below the subnormal floor denormalizes by exact shifting into the
sticky, and a result beyond the finite range clamps to the largest finite or
the infinity the rounding direction demands. Every step is exact, so both
directed results are correctly rounded; there is no error term to argue about.
The base magnitude `1` short-circuits to an exact `1` for every `n`, keeping
the identity past the exact range where the fallback chain would drift off it.

**`POWN_TIGHT_MAX` is 64 for both shipped backends.** The significand of
`x^n` carries `53 * |n|` bits, so exactness for unbounded `n` has no
bounded-memory implementation (near-one bases keep astronomically large
powers finite), and the table maker's dilemma bars an approximate shortcut
from claiming correct rounding. The honest contract is a named exact range
and a sound tail: 64 covers every plausible rigorous-computing integer power
(the corpus maximum is 8) at a bounded cost of 56 limbs per operand buffer
(54 load-bearing, the rest headroom), a few hundred bytes of stack per Big and
about a kilobyte across the operation, no allocation. Beyond the bound the
implementation falls back to the directed repeated-squaring chain, valid
but not tightest, and the trait documentation says exactly that.

**The `f64` fixture implements `RoundPown` through the same kernel** and so
gains a tight `pown`. The fixture's doctrine (sound, never promising
tightness) is a floor, not a ceiling; `RoundInteger` set the precedent of an
exact fixture operation. Fixture pown twins that turn green may un-ignore.

**The interval layer moves `pown` behind `F: RoundFloat + RoundPown`** (its
own impl block; likewise the decorated wrapper). Endpoint powers become
single kernel calls under the existing parity case analysis, and the
negative-exponent path leaves the set-level reciprocal: the image for
`n < 0` is computed endpoint-wise from negative kernel powers under the
same parity and zero-position analysis, producing the unbounded results the
vectors pin without the overflow collapse. The chain helpers leave
inverse.rs; reverse.rs keeps its private copies until the pown_rev slice
(bead enc-cov) consumes the kernel there.

## Consequences

- The kernel is new handwritten multi-limb integer code in the trust base of
  every tight enclosure: a carry or sticky bug produces wrong tight bounds, a
  soundness violation worse than the looseness it replaces. Mitigations: the
  astro-float exact-reference property lane grows a pown family; the corpus
  pins 163 vectors bit-exact; the kernel is pure integer code and a natural
  later Kani target (no harness this slice; the CI kani job budget is fixed).
- The `POWN_TIGHT_MAX` cliff invites reading tight where `|n| > 64` is only
  sound. Mitigations: the bound is a typed associated const, and the trait
  doc plus the conformance document's accuracy statement name it.
- A fixture that is tight for `pown` no longer exercises generic code
  against pown looseness. Accepted: the fixture's contract is soundness, and
  every other directed operation stays loose.
- `Interval::pown` gains a trait bound, a compile break for third-party
  backends implementing bare `RoundFloat`. Accepted pre-1.0; the standard
  requires `pown`, so a conforming backend owes the capability.

## References

- Bead enc-5jj; `crates/interval-1788/docs/conformance.md` section 4 item 2.
- Workspace ADR-0007 (the falsified assumption; erratum rides this slice)
  and ADR-0005 (extension-trait growth doctrine).
- round-float ADR-0003 (`RoundInteger`): the exact-capability precedent.
- IEEE 754-2019 clause 9.2 (`pown`), pointer only; the vendored
  `libieeep1788_elem.itl` corpus is the operational pin.
