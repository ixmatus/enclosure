# Changelog

All notable changes to round-float are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

- `RoundInverseTrig`, `RoundInverseHyperbolic`, and `RoundExpBases`, the round-two
  transcendental extension traits (`: RoundFloat`): inverse trigonometric
  functions with the pi enclosure and the two-argument `atan2` (argument order as
  `f64::atan2`), inverse hyperbolics, and the base-2/base-10 exponentials and
  logarithms. The `f64` fixture implements all three behind the `f64` feature:
  the goal-covered functions reuse the shipped `TRANSCENDENTAL_MARGIN` doctrine
  with closed range clamps at every endpoint attainable within one ulp; the arc
  hyperbolics carry their own `ARC_HYPERBOLIC_MARGIN` (16 units in `2^-52`),
  anchored empirically against the pfloat-libm oracle (worst observed libm 0.2.16
  error 1 ulp, headroom past tenfold) because musl's accuracy goal excepts them,
  with the anchor stated at the constant. Workspace decision record 0007.
- `RoundReduction`, the family's first tightness-mandatory extension trait: the
  four IEEE 1788-2015 reduction operations (`sum`, `sum_abs`, `sum_square`,
  `dot`) in the four rounding directions, sixteen methods whose contract is the
  exact mathematical value rounded once. The `f64` fixture deliberately does not
  implement it (a directed loop cannot deliver a correctly rounded nearest);
  `TightF64` does, behind `f64-tight`, through an integer Kulisch accumulator: a
  67-limb (4288-bit) two's complement fixed-point integer that decomposes inputs
  by `to_bits`, multiplies significands exactly into `u128`, and rounds once at
  extraction, with no float arithmetic between decomposition and that rounding.
  Verified by the 15 ITF1788 reduction vectors, an astro-float exact-reference
  property lane, oracle-free mode-consistency properties, and four Kani-proved
  harnesses over the carry and shift-index logic. Workspace decision record 0008.
- `RoundLargestFinite`, a one-constant capability trait (`: RoundFloat`) exposing
  the format's largest finite value exactly (`LARGEST_FINITE`), implemented by
  both `f64` backends with `f64::MAX`; the datum IEEE 1788's Level 2 realmax
  midpoint convention requires. Workspace decision record 0009.

### Fixed

- The `RoundHyperbolic` contract documentation claimed a `tanh` result inside the
  open interval `(-1, 1)`; the open form is unsound at the saturation shoulders
  (the true `tanh` approaches the endpoints closer than one ulp, so the endpoint
  is the tightest sound far bound), and the shipped fixture has always clamped
  closed. The doc now states the closed range; workspace decision record 0005
  carries the errata.
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
