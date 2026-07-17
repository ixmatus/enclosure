# Changelog

All notable changes to elementary-oracle are documented here. The format follows
Keep a Changelog. This crate is a workspace-excluded nightly verification lane at
version 0.0.0; it ships no public surface, so the log records the grids the
differential lane covers.

## [Unreleased]

### Added

- Trigonometric, hyperbolic, and power grids (decision record 0005 part 5),
  extending the `exp`/`ln` lane to the transcendental growth.
  - Trigonometric reduction stress: the f64 fixture's directed `sin`/`cos`/`tan`
    bracket pfloat-libm's correctly-rounded values at arguments well past `2^50`,
    where a naive argument reduction loses all significance, and within `1e-9` of
    the extrema, the zeros, and (for `tan`) the poles.
  - The affine `sin`/`cos` fits' reductions enclose the correctly-rounded value
    over small ranges bracketing an extremum inside a single arc, where the fit
    applies.
  - Hyperbolic shoulders: the fixture's directed `sinh`/`cosh`/`tanh` bracket the
    correctly-rounded value across the finite region up to the overflow shoulders
    (`cosh` near `710.5`), and the affine hyperbolic fits enclose it on half-line
    ranges.
  - Power: pfloat-libm has a correctly-rounded `rootn`, so the fixture's directed
    `rootn` is checked for full containment across the base-and-order plane,
    including the odd-root negative-base ray. It has no `pow`, so the real power
    `x^y` rides an `exp`/`ln` composition reference built from the correctly-rounded
    `exp_round`/`ln_round`; the fixture `pow` (a distinct, tighter `libm`
    algorithm) is checked to overlap that reference, and the looser affine
    `pow_scalar` to enclose it, over the (base, exponent) plane edges and the
    domain boundary rays.
