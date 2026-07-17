# Changelog

All notable changes to elementary-oracle are documented here. The format follows
Keep a Changelog. This crate is a workspace-excluded nightly verification lane at
version 0.0.0; it ships no public surface, so the log records the grids the
differential lane covers.

## [Unreleased]

### Added

- Arithmetic tightness and enclosure lane (`tests/arithmetic.rs`, bead enc-ebd,
  round-float decision record 0002), certifying the basic operations against a
  pfloat correctly-rounded directed reference. This is the independent nightly
  complement to round-float's in-tree `tight_oracle`, which judges the same
  backend with an integer big-rational oracle; here the reference is pfloat's
  arbitrary-precision `BigFloat` and pfloat-libm's `sqrt_round`, a wholly separate
  implementation, so agreement is real evidence.
  - For `add`, `sub`, `mul`, `div`, `sqrt`, and the `recip` (`1 / x`) and `sqr`
    (`x * x`) specializations, both directions each: `TightF64`'s directed result
    EQUALS the correctly-rounded directed value exactly (tightness, value-level so
    the documented signed-zero divergence of Law 7 is not a mismatch), and the
    `f64` fixture's directed result soundly brackets it (enclosure).
  - Reference derivation: `add`/`sub`/`mul`/`sqr` compute the exact dyadic result
    in a `BigFloat` above its bit width (the lane asserts the `INEXACT` flag stays
    clear) and round once to `f64` per direction; `div`/`recip` round the
    non-terminating quotient to an intermediate precision then to `f64` in the same
    direction (directed double rounding through a higher precision nests, so it
    stays correctly rounded); `sqrt` reads pfloat-libm's correctly-rounded
    `sqrt_round` directly.
  - Grids aimed at the arithmetic hazards of decision record 0002: subnormal
    swarms and gradual underflow through the rescaling path, the overflow shoulder
    and its edge convention (down to `MAX`, up to `+inf`), exact results that
    collapse to one value, near-halfway and irrational results for the
    nearest-adjacent directed pair, signed zeros, mixed magnitudes across the
    exponent range, and seeded pseudo-random sweeps. A `tightness_check_has_teeth`
    guard proves the tightness equality is not vacuous: on an inexact case the
    fixture must fall strictly outside the tight reference on a side.
  - No backend findings: `TightF64` matched the correctly-rounded reference and the
    fixture bracketed it across every grid.
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
