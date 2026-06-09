# ADR-0001: The enclosure monorepo and the round-float layering

**Status:** Accepted
**Date:** 2026-06-08

## Context

interval-1788 (rigorous IEEE 1788-2015 interval arithmetic) was built as a
standalone crate, generic over a directed-rounding float trait `RoundFloat` it
defined in-crate. Its ADR-0002 shaped that trait deliberately as the common core
of what rigorous outward-rounded arithmetic needs, and deferred extracting it
into a shared foundation crate until the shape was proven in two consumers. Only
one consumer existed (the SMIL calculator's ferrodec `Decimal128` backend);
pfloat-ball's `RealScalar` is a sibling sealed trait, not a `RoundFloat`
consumer, and cannot become one without a nightly refactor.

Two new facts change the calculus. First, a second rigorous-enclosure crate is
now being built: affine-arith, Stolfi affine arithmetic. It is generic over
exactly the same directed-rounding contract (its form's range and the
multiplication and elementary remainders all round outward), so it is the second
independent `RoundFloat` consumer ADR-0002 was waiting for. Second, the family is
growing, and the two crates plus the trait they share want a home that keeps each
crate independent while letting them evolve together.

## Decision

Three decisions, taken together.

**A monorepo, not an umbrella crate.** Hold the family in a Cargo workspace named
`enclosure` (both intervals and affine forms *enclose* a true value). The
workspace has three members and no umbrella crate that re-exports them. The
workspace is packaging, not a unifying framework: every crate stays independently
versioned and independently publishable, so each still stands alone as a
flag-plant. The crate names are the flags; the repository name is packaging. This
keeps the arrangement consistent with the "each library stands alone first" and
"no premature ecosystem ambition" rules, because a workspace shares only a
lockfile, a toolchain pin, and CI, none of which couple the crates' public
surfaces.

**Extract round-float now.** Move the `RoundFloat` trait out of interval-1788
into a foundation crate `round-float`. interval-1788 re-exports it
(`pub use round_float::RoundFloat`) so every downstream `impl RoundFloat for _`
keeps resolving `interval_1788::RoundFloat` unchanged. The dependencies point one
way:

```text
    round-float         the directed-rounding float contract
      |- interval-1788   IEEE 1788-2015 intervals, generic over it
      |- affine-arith    Stolfi affine forms, generic over it
```

The sound-not-tight f64 fixture moves with the trait, behind a round-float `f64`
feature, because it is shared verification infrastructure: both enclosure crates
run their Kani and property lanes over the same instance. Production instances
stay consumer-side (ferrodec `Decimal128` in SMIL, pfloat's floats in pfloat).
The extraction is a relocation, not a redesign, because ADR-0002 shaped the trait
for it.

**affine-arith is separate from interval-1788, not folded in.** Affine arithmetic
is the Stolfi model, not part of IEEE 1788. Folding it into interval-1788 would
dilute that crate's "this crate IS IEEE 1788" identity, break the ecosystem
precedent (the interval and affine libraries are always separate: MPFI and
filib++ for intervals, libaffa and aaflib for affine forms), and erase the
second-consumer signal that justifies the extraction in the first place. So
affine-arith is its own crate, depending on interval-1788 for range reduction and
the `Interval` an affine form reduces to.

## Consequences

- ADR-0002's deferred extraction is now executed; see that record for the
  trait-shape rationale (split directions, no rounding-mode parameter). Its
  "defer until two consumers" condition is satisfied by affine-arith.
- interval-1788's public `RoundFloat` path is preserved through the re-export, so
  the SMIL/ferrodec backend and the vendored snapshot keep compiling.
  interval-1788 keeps its own version and CHANGELOG; only its dependency structure
  changed, with no behavior change (66 unit and property tests and 12 Kani proofs
  unchanged).
- round-float is the smallest crate that can stand alone: a trait plus an optional
  fixture. It carries its own disclosure README and its own decision records as it
  grows.
- Each crate publishes on its own schedule. The workspace shares the stable 1.86
  toolchain pin (the wall that keeps the family consumable by stable SMIL and
  nightly pfloat alike), one `Cargo.lock`, and one CI matrix.
- GUM-style uncertainty (a value plus a standard uncertainty, ISO/IEC Guide 98-3)
  does NOT live here. It shares affine arithmetic's data structure (a central
  value plus a sparse vector of linear sensitivities) but combines by
  root-sum-square rather than worst-case, so it belongs at the SMIL level as a
  `Number` variant that may reuse the affine linear form only if that form falls
  out generic. Naming this boundary here forestalls scope creep into round-float
  or affine-arith.

## References

- interval-1788 ADR-0002 (the `RoundFloat` trait shape and the deferred
  extraction this record resolves).
- interval-1788 ADR-0003 (the f64 fixture round-float now hosts).
- Comba and Stolfi, "Affine Arithmetic and its Applications to Computer Graphics"
  (1993); de Figueiredo and Stolfi, "Affine Arithmetic: Concepts and
  Applications" (2004): the affine model affine-arith implements.
- The ecosystem precedent: MPFI and filib++ (interval libraries) versus libaffa
  and aaflib (affine-arithmetic libraries), always shipped separately.
