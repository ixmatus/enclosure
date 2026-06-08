# Changelog

All notable changes to interval-1788 are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

- Phase A, the construction layer.
  - `RoundFloat`, the directed-rounding float contract the interval layer is
    generic over (split-direction methods, extended-real constants, and the
    classification predicates), with the rounding contract written down.
  - `Interval<F>`, the inf-sup interval type: the empty set, bounded and
    unbounded nonempty intervals, and the whole line. The lower-or-equal-upper
    invariant is held by construction and unrepresentable through the public
    surface. Constructors `new`, `point`, `empty`, `entire`; observers `inf`,
    `sup`, `contains`, and the `is_*` predicates.
  - `IntervalError`, the construction failures (NaN endpoint, unbounding
    infinity, inverted endpoints).
  - The `fixture` feature: an `f64` instance of `RoundFloat` for Kani proofs and
    host property tests. It is sound, not tight, by design.
  - `spec`, the written-down laws (the construction invariant, enclosure
    soundness, outward rounding, the four-corner rule, total set-based results,
    and the fixture's soundness-not-tightness).

- Phase B, forward arithmetic.
  - The `core::ops` operators `+`, `-`, `*`, `/`, and unary `-` on `Interval<F>`,
    each total and outward-rounded. Multiplication is the four-corner rule with
    the `0 * inf` indeterminate resolved to zero; division is multiplication by
    the reciprocal, so a divisor containing zero is handled by `recip` (empty for
    an exact zero divisor, unbounded for a straddling one) with no separate path.
  - Inherent `recip`, `sqr` (tighter than `self * self`), `sqrt` (domain
    restricted to the nonnegative part), and `mul_add`.
  - `RoundFloat` gained `ONE` and exact `negate`.
  - Property tests over the fixture: pointwise enclosure for `+`, `-`, `*`, and
    square; the singleton merge gate (a point operation's result is enclosed, the
    property the SMIL engine also gates on) for all five operations;
    commutativity of `+` and `*`; and the invariant upheld by every result.

- Kani proof harnesses (`fixture` feature, `cargo kani --features fixture`) for
  the discrete set-based case logic: empty absorption for `+` and `*`, division
  by the exact zero interval, reciprocal of the exact zero and of a
  zero-straddling interval, and square root of a wholly negative interval. All
  six pass. The numeric enclosure theorem is NOT discharged by Kani over the
  fixture (its `next_up` / `next_down` bit manipulation is intractable for CBMC);
  enclosure rests on the `spec` argument and the property tests, with an
  abstract-axiom Kani approach recorded in ADR-0003 as the route to a proven
  four-corner. See ADR-0003.
