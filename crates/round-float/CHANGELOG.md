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
- The `f64-tight` feature: `TightF64`, a correctly rounded directed backend on
  stable `no_std` Rust. Each operation recovers the exact sign of its
  round-to-nearest rounding error from an error-free transform (branch-free
  TwoSum for add and subtract, `libm::fma` for the product and residual
  transforms of multiply, divide, and square root) and steps one float outward
  only when the result is unrepresentable, so every bound is the tightest sound
  one. A power-of-two rescaling path handles the underflow zone where the
  transforms stop being exact, its correctness resting on the nested-grid lemma
  written into the module docs. The backend is validated against an independent
  integer reference oracle over the subnormal, binade-boundary, and overflow
  zones. See round-float decision record 0002.
