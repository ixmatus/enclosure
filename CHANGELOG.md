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
