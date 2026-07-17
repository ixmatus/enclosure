# ADR-0002: The tight binary64 backend: correctly rounded directed arithmetic on stable Rust

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-16

## Context

interval-1788's v1.0 road (that crate's ADR-0005, ledger item 10) needs a
correctly rounded binary64 `RoundFloat` instance: the ITF1788 conformance
vectors demand tightest results, and the f64 fixture this crate ships is
sound-not-tight by design and fails them structurally. Beyond the
conformance lane, every consumer gains a fast native tight backend to set
beside the decimal and arbitrary-precision ones. This is bead enc-aet.

Three constraints shape the design:

- **Stable Rust cannot set the rounding mode, and no Rust can do so
  soundly.** The obvious route (switch the hardware mode to round-down,
  compute, switch back) is not merely unavailable on stable; LLVM assumes
  the default floating-point environment and may constant-fold, reorder, or
  hoist float operations across any mode switch, so the technique is
  unsound in Rust at any stability level. The design must produce directed
  results from round-to-nearest arithmetic only.
- **The crate is `no_std`.** The existing optional `libm` dependency
  supplies `sqrt` (correctly rounded) and `fma` (correctly rounded, soft);
  nothing from `std` is available or wanted.
- **The trait surface owed is small.** `RoundFloat` requires directed
  `add`, `sub`, `mul`, `div`, `sqrt`, exact `negate`, predicates, and
  constants. `RoundTranscendental` is explicitly out of scope: no correctly
  rounded transcendental exists in stable pure Rust (pfloat-libm is
  nightly), and the conformance lane already routes elementary vector files
  to the nightly oracle lane.

## Decision

**A newtype `TightF64` behind a new additive `f64-tight` feature.** The
fixture's `RoundFloat` impl on raw `f64` is untouched and keeps its
verification role; the newtype lets both instances coexist (two trait impls
on `f64` itself cannot, and cfg-switched behavior behind mutually exclusive
features is a cargo antipattern). `TightF64` implements `RoundFloat` only.

### The derivation pattern

Every operation follows one shape: compute the round-to-nearest result `r`,
obtain the exact sign of the rounding error `t - r` (where `t` is the exact
real result) via an error-free transform, then adjust one ulp in the losing
direction only:

```text
down = if t < r { next_down(r) } else { r }
up   = if t > r { next_up(r)   } else { r }
```

The correctness argument, stated once and instantiated per op in the
implementation module's numbered laws: round-to-nearest brackets the exact
value between the result's neighbors, `r` equals `t` exactly when `t` is
representable (in which case the error term is zero and both directions
return `r`), and otherwise `t` lies strictly between `r` and the neighbor
on the error's side, so the adjusted float is the correctly rounded
directed result, not merely a sound bound.

Per-operation error terms, each derived from the published mathematics
(registry: ogita-rump-oishi-2005, boldo-daumas-2003, muller-handbook-fp),
with the Rust shapes chosen fresh:

1. **`add`/`sub`: TwoSum** (Knuth's branch-free six-operation form). The
   addition correcting term is representable for ALL inputs, subnormals
   included, so this path has no underflow caveat. `sub` is `add` of the
   exact negation.
2. **`mul`: TwoProduct via FMA.** `r = RN(ab)`, `e = fma(a, b, -r)`, exact
   except where the product underflows (the Boldo-Daumas exponent
   condition). Outside the risky zone the sign of `e` decides the adjust;
   inside it, the rescaling path below.
3. **`div`: FMA residual.** `q = RN(a/b)`, `rho = fma(q, b, -a)`; the sign
   of `q - a/b` is `sign(rho) * sign(b)`, and `rho == 0` means exact. The
   residual's representability has its own Boldo-Daumas condition; the
   risky zone takes the rescaling path.
4. **`sqrt`: FMA residual.** `s = RN(sqrt(a))` from `libm::sqrt` (correctly
   rounded), `rho = fma(s, s, -a)`; the sign of `rho` is the sign of
   `s - sqrt(a)`. The subnormal-argument corner is checked against the same
   representability conditions and rescales (by an even power) if needed.
5. **`negate`, `rmin`, `rmax`, predicates, constants:** exact, no
   derivation.

### The subnormal rescaling path

Where a product or quotient lands in the underflow zone, the error term of
the EFT can itself be inexact, silently voiding the "error-free" premise.
The strategy: scale into the safe exponent range by an exact power of two,
compute the correctly rounded DIRECTED result there (the EFT is exact in
the scaled domain), then unscale with an exactness-checked directed
division by the same power (multiply back and compare; adjust one ulp on
mismatch). The nested-grid lemma justifying that two downward roundings
through the scaled and subnormal grids equal one direct downward rounding
is written out in the implementation module's laws; the ADR fixes the
strategy and names the lemma, the module carries the proof, and the
threshold constants are derived at implementation time against the
Boldo-Daumas conditions rather than guessed here. Double rounding through
the scaled domain is exactly the trap this path is designed around: the
nearest result is never derived by unscaling a scaled nearest result; only
directed results cross the scaling boundary.

### Edge semantics

- **Overflow:** a nearest result of `+inf` from finite inputs means the
  exact value exceeds `MAX`, so `down` returns `MAX` and `up` returns
  `+inf` (mirrored below). This matches IEEE directed rounding, where
  round-down of a value beyond the largest float is the largest float.
- **Non-finite inputs pass through** IEEE semantics untouched (`inf`
  arithmetic is exact in the extended reals; NaN propagates). The trait
  contract owes correct directed rounding for finite inputs; the interval
  layer feeds infinities only where the extended-real result is exact.
- **Zero signs follow round-to-nearest, not the directed modes.** A true
  round-down returns `-0` for an exact zero sum; `TightF64` returns
  nearest's `+0`. The values are numerically equal and every enclosure
  property is value-level, so this is a documented divergence from
  bit-for-bit IEEE directed rounding, not a soundness gap.

### Named portability preconditions

- `f64` arithmetic is IEEE 754 round-to-nearest on the target. True on
  every Tier 1 target; x87-without-SSE2 (extended-precision double
  rounding) is explicitly unsupported and documented.
- `libm::fma` is correctly rounded. Trusted from its musl lineage AND
  certified by the oracle lane rather than assumed; this is a named trust
  boundary, priced in the verification obligations.

### Verification obligations

The backend gets its own lane, one tier at a time:

1. **The adjacency invariant, property-tested:** `down <= up`, `up` is
   `down` or `next_up(down)`, and the nearest result lies in `[down, up]`.
   This single invariant catches most derivation mistakes cheaply.
2. **Exactness detection:** on constructed exact cases (representable sums,
   products of small integers, perfect squares), `down == up`.
3. **Envelope against the fixture:** the fixture's bounds are never
   tighter, and both enclose the same exact value.
4. **A hard-case vector catalogue in-crate:** cancellation, binade
   boundaries (where neighbor gaps are asymmetric), subnormal products and
   quotients through the rescaling path, overflow edges, perfect squares,
   signed zeros.
5. **The nightly oracle lane:** directed results equal pfloat-libm's
   correctly rounded directed values over random and adversarial grids,
   folded into the enc-ebd oracle crate growth.
6. **Kani: not promised.** CBMC bit-blasts float arithmetic (interval-1788
   ADR-0003's finding); TwoSum on symbolic floats is the same wall. The
   correctness rests on the written laws, the property lanes, and the
   oracle, and the module docs say so in the fixture's tradition.

## Consequences

- interval-1788's in-tree conformance vector lane (ledger item 11) is
  unblocked for the arithmetic, set, boolean, and numeric files; consumers
  get tight native binary64 intervals on stable.
- The three most plausible failures, inverted up front:
  1. **A sign error in the rescaling path** flips a bound in the subnormal
     zone: the exact defect class the README disclosures name, in the one
     region where the mainline EFT argument does not apply. Mitigations:
     the conditions are derived from Boldo-Daumas rather than intuition,
     the subnormal catalogue exercises the path directly, and the oracle
     lane certifies against an independent correctly rounded
     implementation.
  2. **`libm::fma` is wrong somewhere:** every mul/div/sqrt bound built on
     it inherits the error while local tests (using the same fma) stay
     green. Mitigation: the oracle lane is the independent check, and the
     certified-platform scope (the CI targets) is documented rather than
     implied universal.
  3. **Performance disappoints:** soft fma per operation makes `TightF64`
     slower than the fixture and much slower than raw float math. Priced
     as a non-goal: this is a correctness backend first, and a
     hardware-fma fast path is a later additive option once correctness is
     certified, not a v1 complication.
- The zero-sign divergence and the unsupported-x87 boundary are documented
  API-level caveats a future conformance claim must restate.

## Alternatives considered

- **Hardware rounding-mode switching:** unsound under LLVM's
  default-FP-environment assumption regardless of Rust stability; rejected
  on soundness grounds, not availability.
- **A second outward-stepping instance:** free to build and fails the
  tightest-result vectors; the fixture already occupies that point in the
  design space.
- **Double-word (Dekker) comparison arithmetic:** strictly more machinery
  than the sign-of-error decision needs; the EFT sign is the minimal
  sufficient artifact.
- **MPFR or C directed-rounding FFI:** rejected on the workspace's
  permacomputing posture (pure Rust over FFI even at a performance cost),
  and it would reintroduce the C toolchain the family exists to avoid.

## References

- Registry: `docs/references/ogita-rump-oishi-2005.md` (TwoSum, TwoProduct);
  `docs/references/boldo-daumas-2003.md` (correcting-term representability
  under underflow, the rescaling boundary);
  `docs/references/muller-handbook-fp.md` (consolidated EFT and double
  rounding reference); `docs/references/rump-2010-acta-numerica.md`
  (directed bounds from nearest arithmetic in verification practice);
  `docs/references/ieee-754-2019.md` (rounding-direction semantics).
- interval-1788 crate ADR-0005 (ledger items 10 and 11) and ADR-0003 (the
  CBMC finding inherited by obligation 6).
- Beads: enc-aet (this design), enc-ebd (the oracle crate this lane grows
  into), enc-ac4 (the conformance lane consumer).
