# Changelog

All notable changes to round-float are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

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
