# Changelog

All notable changes to round-float are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

- `RoundInteger`, an exact extension trait (`: RoundFloat`) for the integer
  rounding point functions `floor`, `ceil`, `trunc`, `round_ties_to_even`, and
  `round_ties_to_away`. Every method is exact in any binary or decimal format, so
  there is no `_down`/`_up` split, the same undirected shape as `negate`. The
  `f64` fixture implements it behind the `f64` feature by delegating to `libm`
  (`roundeven` and `round` for the two tie rules). Decision record 0003.
- `RoundFloat`, the directed-rounding float contract, extracted from
  `interval-1788` into this foundation crate so the interval and
  affine-arithmetic crates can share one contract. Split-direction methods
  (`add_down` / `add_up` and kin for subtract, multiply, divide, and square
  root), extended-real constants, exact `negate`, the classification predicates,
  and the `rmin` / `rmax` reducers, with the rounding contract written down.
- The `f64` feature: an `f64` instance of `RoundFloat` for Kani proofs and host
  property tests. It computes in round-to-nearest and steps one float outward,
  so it is sound but deliberately not tight. `libm::sqrt` keeps it
  no_std-capable.
