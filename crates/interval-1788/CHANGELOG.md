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

- Phase C, numeric, boolean, and set functions.
  - Numeric: `wid`, `mag`, `mig`, each returning `Option<F>` (`None` for the
    empty interval, where the standard returns NaN, keeping the empty case
    explicit in the type).
  - Boolean relations: `subset`, `interior`, `disjoint`, with the empty set
    handled as the identity it is.
  - Set operations: `intersection` (empty when the operands do not overlap) and
    `convex_hull` (the empty set as identity).
  - Property tests including inclusion isotonicity for `+`, `-`, `*` (the
    monotonicity half of the fundamental theorem), intersection contained in both
    operands, convex hull containing both, and `disjoint` iff empty intersection.
  - Deferred: `mid` and `rad`, and the ordering relations (`less`, `precedes` and
    their strict forms), pending the Level 2 conventions for their empty and
    unbounded cases.

- Phase D, decorations.
  - `Decoration`, the `com`/`dac`/`def`/`trv`/`ill` lattice, with `meet` (the
    weaker of two, the propagation combine; `ill` absorbing, `com` the identity).
  - `DecoratedInterval<F>`: an interval paired with a decoration. `new_dec`
    (strongest decoration for a bare interval), `nai` (the Not-an-Interval
    poison), `set_dec` (checked pairing, NaI on an inconsistent combination),
    `interval` and `decoration` accessors. The `+`, `-`, `*`, `/` operators plus
    `neg`, `recip`, `sqr`, `sqrt`, each propagating the decoration per the
    fundamental theorem of decorated interval arithmetic: the result carries the
    meet of the inputs' decorations and the operation's local decoration (`trv`
    when undefined somewhere on the box, `com` on bounded inputs, `dac` when
    unbounded). A NaI input poisons the result.
  - Six Kani proofs of the lattice laws (meet commutative, associative,
    idempotent, `ill` absorbing, `com` identity, monotone), all passing. The
    decoration layer is finite-state, so this is the part of the crate the model
    checker discharges fully.
  - Property tests: decoration never strengthens through an operation, NaI poisons
    exactly when an input is NaI, every result is a consistent decorated pair, and
    the bare interval matches the undecorated operation.

### Changed

- `RoundFloat` moved to the new `round-float` foundation crate and is re-exported
  here as `interval_1788::RoundFloat`, so downstream `impl RoundFloat for _`
  (the SMIL/ferrodec backend) keeps compiling unchanged. The `fixture` feature
  now pulls the f64 instance from `round-float`'s `f64` feature instead of
  defining it locally. The crate joined the `enclosure` workspace; the repository
  metadata points at the family. No behavior changed (66 unit and property tests
  and 12 Kani proofs unchanged).
