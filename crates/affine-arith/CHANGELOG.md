# Changelog

All notable changes to affine-arith are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

- P1, the construction layer.
  - `AffineForm<F>`, the affine form `x₀ + Σ xᵢ εᵢ` over a directed-rounding
    float, with its representation invariant (deviation terms sorted by symbol,
    no duplicates, no zero coefficients) held by construction.
  - `NoiseSymbol` and `SymbolSource`: explicit, deterministic fresh-symbol
    allocation, threaded into the operations that introduce symbols rather than
    drawn from global state.
  - `AffineForm::point`, an exact form with no uncertainty.
  - `AffineForm::from_interval`, the construction of a form enclosing a bounded
    nonempty interval with one fresh symbol (`None` for the empty or unbounded
    interval). Sound for any finite center: the radius is the larger one-sided
    distance to an endpoint, rounded up.
  - `AffineForm::reduce`, the reduction to the enclosing interval `x₀ ± Σ|xᵢ|`,
    rounded outward, with a point form reducing exactly to its center.
  - Accessors `center`, `terms`, `num_terms`, `is_point`, `radius`.
  - The `fixture` feature and a property lane over round-float's f64 instance:
    the interval-to-form-to-interval round-trip always encloses, point forms are
    exact, and fresh symbols are distinct.
