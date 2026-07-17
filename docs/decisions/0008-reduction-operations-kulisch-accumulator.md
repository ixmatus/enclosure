# ADR-0008: Reduction operations: a tightness-mandatory trait over an integer Kulisch accumulator

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-16

This record promotes the design half of bead `enc-k8l`, whose first half (the
required-versus-recommended status pin) resolved on the same day: the
reduction operations `sum`, `dot`, `sumSquare`, `sumAbs` are REQUIRED in the
set-based flavor of IEEE 1788-2015 (clause 12.2.12, per the P1788.1/D9.8
Annex A), correctly rounded, as witnessed in tree by the corpus's dot
cancellation vector that only exact accumulation answers. The interval-1788
v1.0 road (that crate's ADR-0005, ledger item 7) left two design questions
to this slice: the operation surface (rounding modes) and the home plus
algorithm. Both are settled here.

## Context

Three facts shape the design:

1. **The operations act on vectors of numbers, not intervals.** A reduction
   is a point-vector operation supporting interval algorithms (verified
   linear algebra), which is why it sits in a Level 2 clause. Its natural
   home is the float-contract crate, not the interval crate.
2. **The surface is per-mode, not split-direction.** The draft standard's
   reference implementation (libieeep1788, surveyed 2026-07-16) declares
   every reduction with a rounding-mode parameter over four modes: downward,
   upward, to nearest, toward zero. The corpus tests only the nearest
   variants (16 assertions), so the other modes rest on the reference
   surface plus the oracle lane.
3. **Correct rounding is the contract, not an upgrade.** For every other
   trait in the family, soundness is the obligation and tightness a quality
   a backend may add. A reduction whose `to_nearest` is not the correctly
   rounded exact value is not a loose version of the operation; it is a
   different operation. The house sound-not-tight posture cannot express
   this contract, and pretending it can would be the failure.

## Decision

### 1. `RoundReduction`: the family's first tightness-mandatory trait

A new extension trait in round-float, `RoundReduction: RoundFloat`, with
per-mode method families following the house no-enum style (the mode is
static at the call site, as directions are):

- `sum_down/_up/_to_nearest/_to_zero(values: &[Self]) -> Self`
- `sum_abs_*(values: &[Self]) -> Self`
- `sum_square_*(values: &[Self]) -> Self` (the standard's `sumSquare` name)
- `dot_*(a: &[Self], b: &[Self]) -> Self` (equal lengths, a caller-owned
  precondition, asserted)

Sixteen methods, each one extraction from the same exact accumulation. The
contract, stated at the trait: the result is the exact mathematical value
rounded ONCE in the named mode. **A backend that cannot deliver correct
rounding must not implement this trait.** The f64 fixture therefore does
not implement it: a directed naive loop can bound a sum soundly, but no
cheap loop yields the correctly rounded nearest of the exact sum, and a
trait instance that met three modes honestly and one vacuously would be the
sound-but-vacuous default ADR-0005 already rejected. Capability honesty
over coverage.

Special values take the corpus's extended-real exact-sum semantics, pinned
by `libieeep1788_reduction.itl`: any NaN input gives NaN; `sum`/`dot` with
infinities of both signs give NaN; `sum_abs`/`sum_square` with any infinity
give positive infinity; a `dot` pairing zero with an infinity gives NaN.
The empty vector's value is the operation's identity (zero); the
clause-text convention for its sign per mode is an unverified corner
(paywalled text), noted in the implementation and pinned to the reference
implementation's behavior.

interval-1788 re-exports the trait beside `RoundFloat` for its conformance
surface; nothing interval-shaped is added.

### 2. The `TightF64` implementation: an integer Kulisch accumulator

The first implementation lands in round-float behind the `f64-tight`
feature, on `TightF64`, and it is deliberately NOT built from float
error-free transforms. The product of two binary64 values spans exponents
no float EFT can capture (down to 2^-2148, far below the subnormal floor),
so the FMA TwoProduct route would drag the ADR-0002 underflow machinery
into every element. Instead, the accumulator is exact by integer
construction:

- Each input decomposes by `to_bits` into sign, exponent, and 53-bit
  significand: pure integer data, no rounding anywhere.
- `dot`/`sum_square` products multiply significands exactly (53x53 into
  u128); `sum`/`sum_abs` skip the multiply.
- Every exact partial value shifts into a fixed-point accumulator indexed
  by exponent: a `[u64; N]` limb array sized once from the binary64 product
  exponent span plus carry headroom (the classic Kulisch dimensioning,
  roughly 4288 bits plus slack; the exact N is derived in the module docs).
  Signed accumulation with carries; no allocation, no_std, no dependence on
  `libm` for the accumulation itself.
- The final step locates the leading bit, gathers the sticky tail, and
  rounds ONCE per the requested mode, with overflow to the mode's correct
  boundary (`MAX` versus infinity, per the ADR-0002 edge convention) and
  the documented zero-sign convention.

Float arithmetic appears nowhere between input decomposition and final
rounding, so the exactness argument is an integer-arithmetic argument, and
the one rounding step is auditable in isolation. This is the verifiable
choice, not the fast one, and that ordering is deliberate.

### 3. Verification obligations

1. **The corpus vectors**: all 16 reduction assertions, including the
   cancellation dot that motivated the design; they run in the conformance
   lane over `TightF64` (in-tree, stable).
2. **Property lane against an independent exact reference**: `astro-float`
   (pure Rust, stable, the workspace's canonical MPFR replacement) computes
   the exact sum/dot at high precision for random and adversarial vectors
   (cancellation towers, subnormal swarms, overflow shoulders, mixed
   magnitudes spanning the full exponent range); all four modes checked,
   since the corpus pins only nearest. Dev-dependency of the test lane
   only.
3. **Mode-consistency properties, oracle-free**: `down <= to_nearest <= up`,
   `down <= up` with the pair adjacent or equal, `to_zero` equals whichever
   directed result is closer to zero, and exactness detection (integer
   vectors whose sum is representable give equal results in all modes).
4. **Kani on the limb logic, attempted with a stop loss.** The accumulator
   is integer bitvector arithmetic, CBMC's home ground, unlike the float
   transmute chains ADR-0003 found intractable. Bounded harnesses (short
   vectors, symbolic significands and exponents) target carry propagation
   and the shift-index map. The ADR-0003 stop-loss discipline applies: a
   harness that does not discharge gets its bounds shrunk once, then cut
   and recorded. The float-to-bits decomposition sits at the harness
   boundary so the transmute happens on concrete or lightly constrained
   values.

### 4. Explicitly out of scope

- **EFT distillation** (the Ogita-Rump-Oishi AccSum/AccDot family): the
  faster route and the likely future performance option, deferred because
  its convergence-and-underflow correctness argument costs more
  verification than the accumulator's integer exactness, and this backend
  is correctness-first (the ADR-0002 performance posture, unchanged).
  Registry: ogita-rump-oishi-2005 carries the pointer.
- **Reductions on the fixture or other backends**: pfloat implements its
  own correctly rounded reductions on its own schedule (estate crate);
  ferrodec decimal likewise if SMIL ever needs them. The trait waits.
- **An interval-returning reduction surface**: strictly more informative
  than the standard's per-mode scalar, but nonstandard; the conformance
  mapping wants the standard's shape, and the directed pair already gives
  enclosure consumers what they need.

## Consequences

- Ledger item 7 of the v1.0 road is fully designed: status verified,
  surface pinned to the reference implementation's four modes, home and
  algorithm fixed, verification priced. The implementing half is mechanical
  against this record.
- The three most plausible failures, inverted:
  1. **A limb-indexing or carry bug produces a plausible wrong sum** that
     random testing misses (exactness bugs hide in rare carry chains).
     Mitigations: the adversarial vector families are built to force long
     carries; the astro-float reference is independent of every code path
     here; and the Kani attempt targets exactly the carry logic because
     that is where the residual risk lives.
  2. **The mode surface is wrong** (the standard's clause defines fewer or
     different modes than the reference implementation exposes). The cost
     is API churn in a 0.x crate, not soundness; the surface is pinned to
     the best available proxy and the divergence risk is named in the
     conformance document when the claim is drafted.
  3. **The 16-method trait is friction** for the one consumer (the
     conformance lane) if nothing else ever calls it. Accepted: the trait
     is the standard's surface, not a convenience layer, and collapsing it
     to an enum parameter would re-litigate the house style for no caller.
- A second tightness-mandatory trait now exists as precedent shape; any
  future capability with a correctness-or-nothing contract (the exact text
  representation of clause 13.4 is a candidate) cites this record rather
  than re-deriving the posture.

## References

- Registry: `docs/references/kulisch-computer-arithmetic.md` (the
  accumulator's theory), `docs/references/kulisch-2008-p1788-draft.md` (the
  lineage that put reductions in the standard),
  `docs/references/ogita-rump-oishi-2005.md` (the deferred distillation
  alternative), `docs/references/libieeep1788-nehmeier.md` (the mode-enum
  surface witness, surveyed 2026-07-16),
  `docs/references/itf1788-framework.md`
  (`libieeep1788_reduction.itl`, the acceptance vectors),
  `docs/references/p1788-1-d98-draft.md` (Annex A, the requirement's
  source).
- interval-1788 ADR-0005 (ledger item 7); round-float ADR-0002 (the
  `TightF64` home, the edge conventions, and the performance posture this
  inherits); workspace ADR-0005 (the vacuous-defaults rejection this
  trait's honesty rule extends).
- Beads: enc-k8l (this design), enc-ac4 (the conformance lane consumer).
