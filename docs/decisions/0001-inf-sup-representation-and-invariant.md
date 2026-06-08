# ADR-0001: Inf-sup representation and the construction invariant

**Status:** Accepted
**Date:** 2026-06-07

## Context

IEEE 1788-2015 admits more than one representation of an interval. The two in
common use are inf-sup (store the two endpoints) and mid-rad (store a midpoint
and a radius, the ball representation used by Arb and by the sibling crate
pfloat-ball). A crate must choose one as its in-memory form.

The intervals this crate serves are over fixed-width and arbitrary-precision
correctly-rounded floats, used first by a decimal calculator that displays a
result as a bracket and computes with the four-corner multiplication rule. The
standard's set-based flavor also requires the empty set and unbounded intervals.

## Decision

Use the inf-sup representation. An interval is the empty set or a nonempty
`[lo, hi]` with `lo <= hi`, where endpoints range over the extended reals so that
`[-inf, hi]`, `[lo, +inf]`, and `Entire = [-inf, +inf]` are representable. Plus
infinity is never a lower endpoint and minus infinity is never an upper endpoint,
since neither bounds a nonempty set of reals.

Make the invariant unrepresentable to violate. The in-memory form is a private
two-variant enum (empty, or nonempty with its endpoints) wrapped in a struct with
private fields and no setters. Every value is obtained through a checked
constructor, and every operation re-establishes the invariant by routing its
result through a constructor rather than assuming it. The empty set has one
canonical form, so interval equality is structural.

## Consequences

- Endpoint display, the four-corner rule, and the calculator's bracket output map
  directly onto the stored endpoints with no conversion.
- Illegal intervals cannot be constructed or mutated into existence from outside
  the crate; the type carries its invariant the way the project prefers, in the
  type rather than in runtime guards spread across call sites.
- The midpoint and radius are derived quantities here, computed on demand, where
  pfloat-ball stores them directly. The two crates are deliberately complementary
  representations of the same rigorous-enclosure idea, not competitors.
- Unbounded and empty results from total set-based operations are values, not
  errors; a consumer wanting a bounded-only view maps them to its own error at
  its boundary.

## References

- IEEE Std 1788-2015, the set-based flavor and the interval model.
- "Introduction to the IEEE 1788-2015 Standard for Interval Arithmetic" (Revol).
- `src/interval.rs`, `src/spec.rs` (Law 1).
- pfloat-ball, the mid-rad sibling.
