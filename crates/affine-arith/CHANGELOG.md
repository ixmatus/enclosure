# Changelog

All notable changes to affine-arith are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

- P1, the construction layer.
  - `AffineForm<'id, F>`, the affine form `x₀ + Σ xᵢ εᵢ` over a directed-rounding
    float, with its representation invariant (deviation terms sorted by symbol,
    no duplicates, no zero coefficients) held by construction.
  - `NoiseSymbol`, `SymbolSource`, and `with_source`: explicit, deterministic
    fresh-symbol allocation, threaded into the operations that introduce symbols
    rather than drawn from global state.
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

- P2, the arithmetic.
  - `AffineForm::negate` (exact), `add`, `sub`, `scale`, and the rigor-critical
    `mul`. Linear combinations add coefficients over shared symbols, so `x - x`
    cancels; every rounding error and the multiply's bilinear remainder (bounded
    by `radius(x)·radius(y)`) are folded outward into fresh symbols, so every
    result encloses the true value set. The multiply was adversarially verified
    sound (the algebra, the rounding directions, and edge cases) and exercised by
    a property lane: each operation's reduction encloses the corner results, and
    `x - x` collapses far below the doubled width interval arithmetic reports.

- Source branding.
  - `SymbolSource<'id>` and `AffineForm<'id, F>` carry an invariant lifetime
    brand handed out by `with_source`. The operations require their operands and
    the source to share the brand, so the compiler rejects combining forms built
    from different sources, whose symbols would otherwise collide and break the
    enclosure guarantee. The single-source precondition is enforced by the type
    system rather than left to the caller.

- P4, the verification lane.
  - A composition lane property-tests enclosure over whole random expression
    programs, sampling the input symbols at the binding corners and at interiors
    and arbitrating any containment failure against interval-1788 point
    intervals, so oracle noise is separated from a confirmed bug.
  - A tightness lane pins the per-expression win ratio against interval
    arithmetic on a catalogue of correlated expressions, each with a width
    property over random ranges, and documents the uncorrelated multiply as a
    non-goal where interval arithmetic is tighter.
  - Kani harnesses discharge the discrete skeleton (symbol freshness,
    `from_raw_parts` validation, the addition merge shape, condensation counts);
    the numeric enclosure stays with the property lanes, per interval-1788
    ADR-0003, and a `compile_fail` doctest guards the cross-source brand.
- Per-arc trigonometric, hyperbolic, and power fits (`trig` module, decision
  record 0005 part 4).
  - `AffineForm::sin` and `AffineForm::cos` on a `RoundTrig` backend: the
    Chebyshev secant-slope fit when the reduced range lies within a single
    concavity arc (between the inflection points `k·π` for `sin`, `π/2 + k·π` for
    `cos`), decided against the pi enclosure. A range spanning an arc boundary, or
    sitting within a pi-bracket width of one, returns `None` so the caller falls
    back to the interval arm rather than guess the arc (the pi-ambiguity rule).
  - `AffineForm::sinh`, `AffineForm::cosh`, and `AffineForm::tanh` on a
    `RoundHyperbolic` backend: `cosh` fits every finite range (convex
    everywhere); `sinh` and `tanh` fit on a range not straddling zero, where
    their second derivative keeps one sign, and return `None` otherwise.
  - `AffineForm::pow_scalar` on a `RoundTranscendental` backend: the real power
    `x^y` for a scalar exponent, by the `exp`/`ln` composition `exp(y·ln x)` on
    the positive domain (the recorded frugal v1 choice; a direct bivariate fit is
    future work).
  - The interior residual extremum, which `exp`/`ln` locate through their inverse,
    is bounded here by the classical linear-interpolation error `M·(b − a)²/8`
    (`M` an upper bound on `|f''|` over the arc) because the backend exposes no
    inverse. The band is therefore a documented sound over-estimate, not the true
    minimax: the widening factor is at most `π²/8 ≈ 1.23` over a full `sin`/`cos`
    arc, about `1.3` for `tanh` (whose global curvature bound `1` exceeds the true
    `4/(3√3)`), and grows with range width for the exponentially-curved hyperbolic
    fits. A property lane over the f64 fixture checks pointwise enclosure where
    each fit applies, the decline cases (arc-boundary and zero straddles, the
    pi-ambiguity decline), the structural collapse of `sin(x) − sin(x)`, and
    measures the `sin` fit decline rate (about `0.50` over widths uniform in
    `[0.1, 3]`, per decision record 0005 consequence (c)).

### Fixed

- The `from_raw_parts` Kani harness put the CI runner at its memory cliff
  (about 15 million SAT variables and 67 million clauses for a proof of a
  thirty line function; the wave-5b merge run died by runner OOM during the
  solve). An out of tree ladder localized the cost: the allocator and the
  pushes sit near 15 thousand variables and the acceptance check near 129
  thousand, while scanning the returned terms hits 15.2 million. The cost is
  the scan of a heap `Vec` whose length and contents are symbolic, not the
  container; the earlier reading, that the footprint was bit blasted `Vec` and
  allocator machinery, was wrong, and neither the float class split nor a
  concrete index scan moves it (the concrete index form measured slightly
  larger). The single proof now splits into three, one property each:
  `from_raw_parts_acceptance_is_exact` and
  `from_raw_parts_drops_zero_coefficients` read the outcome without a heap scan,
  stay near 128 thousand variables, and attest in CI;
  `from_raw_parts_accepted_form_satisfies_invariant` carries the scan, is
  verified locally (a minute or two), and stays off the CI harness list until
  the runner grows or the model checker encodes a bounded but symbolic length
  slice more tightly. Notices live in the workflow and the module doc. (Bead
  enc-tgw.)


- The condensation Kani harness (`condense_bounds_terms_and_preserves_order`)
  never discharged: its symbolic budget flowed through `keep = budget - 1` into
  the fold slice, the truncation, and the re-sort, putting the fixture's
  `add_up` and the sort's control flow on the symbolic path, the bit-blasting
  interval-1788 ADR-0003 found intractable (150+ CPU-minutes locally, a
  cancelled six-hour CI run). Re-encoded as an exhaustive concrete case split
  over the assumed budget domain (`0..=4` against the fixed three-term form),
  which preserves the property verbatim, strengthens the over-budget arm to
  exact counts (exactly `max(budget, 1)` terms, exactly one fresh symbol), and
  discharges in seconds. (Bead enc-otn.)
- The addition-merge Kani harness (`add_preserves_the_merge_invariant`), never
  observed either because package runs check the condensation harness first
  and its hang shadowed everything behind it, was cut under the ADR-0006 stop
  loss after the ladder ran out on clean evidence: it carried an unwind bound
  latent-insufficient since wave one (8, failing its own unwinding assertions
  on the seven-term result scan), and with the bound corrected, the
  coefficients made uniform per operand, and the id bound shrunk 64 to 8,
  CBMC still exhausts memory on the error accumulator's ite tree feeding
  chained fixture `add_up` calls under symbolic interleavings. The merge
  invariant stays with the unit tests and the composition property lane; the
  recorded routes back are an exhaustive order-type enumeration or ADR-0003's
  abstract-axiom backend. The module stop-loss note carries the full story
  and the shared lesson: a symbolic value that selects, indexes, or sizes
  concrete float data drags the floats onto the symbolic path.
  (Bead enc-9nq, discovered from enc-otn.)
