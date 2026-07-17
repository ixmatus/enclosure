# Changelog

All notable changes to round-float are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

- `RoundTrig`, `RoundHyperbolic`, and `RoundPow`, three per-family transcendental
  extension traits (`: RoundFloat`) for the trigonometric, hyperbolic, and power
  functions, each independently optional so a backend lands the families on its
  own schedule. `RoundTrig` carries `sin`/`cos`/`tan` (`_down`/`_up`), the pi
  enclosure `pi_down`/`pi_up`, and an exact `floor` for argument reduction (a
  deliberate duplicate of `RoundInteger::floor` by decision record 0005's accepted
  trade); `RoundHyperbolic` carries `sinh`/`cosh`/`tanh`; `RoundPow` carries `pow`
  and `rootn`. The `f64` fixture implements all three behind the `f64` feature over
  `libm`, reusing the shipped `TRANSCENDENTAL_MARGIN` doctrine plus three
  additions: range clamps after widening (`sin`/`cos` into `[-1, 1]`, `tanh` into
  `[-1, 1]`, `cosh` at or above `1`), the pi bracket `[PI, PI.next_up()]` (the
  nearest `f64` to pi lies below it, so the successor completes a one-ulp
  enclosure), and overflow edges following the exp precedent. `rootn` routes
  through `pow` with a directed reciprocal exponent so the rounding of `1/n` is
  absorbed soundly rather than left to the fixed margin. Workspace decision record
  0005.
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
