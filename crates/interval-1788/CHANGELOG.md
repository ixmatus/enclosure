# Changelog

All notable changes to interval-1788 are documented here. The format follows
Keep a Changelog, and the project adheres to semantic versioning once it reaches
v1.0; before then the API may break between 0.x releases.

## [Unreleased]

### Added

- The conformance document (`docs/conformance.md`, ledger item 13): the
  per-instantiation claim structure (the claim attaches to `TightF64`, never
  the generic crate), the operation and naming map, the lane's certified
  totals (2,368 bit-exact vectors, zero soundness violations), the six open
  items with their beads, the named divergences (values-not-signals, the
  generic surface, the decorated `mulRevToPair` doctrine question), and the
  evidence basis with its limits. A living document, dated, that becomes the
  clause-12 claim at v1.0 and does not retire the README disclosures.

- The conformance vector lane (bead enc-ac4), split by backend. Over the
  correctly rounded `TightF64` (a dev-dependency; the shipped surface is
  untouched): the `libieeep1788_elem.itl` arithmetic testcases, all 1373
  vectors bit-exact (`tests/conformance_arith_tight.rs`), and the arithmetic
  reverse testcases of `libieeep1788_rev.itl`, `libieeep1788_mul_rev.itl`,
  and `abs_rev.itl`, all 995 vectors bit-exact
  (`tests/conformance_reverse_tight.rs`); the trigonometric and hyperbolic
  reverses stay with the fixture enclosure lanes, which `TightF64`'s trait
  set cannot fund. Over the f64 fixture: `libieeep1788_bool.itl` completed to
  full coverage (the prior ordering transcriptions were sampled), the
  `inf`/`sup`/`wid`/`mag`/`mig` numerics, all of `libieeep1788_rec_bool.itl`
  and `libieeep1788_set.itl`, the `libieeep1788_class.itl` decorated
  constructors (`newDec`, `intervalPart`, `decorationPart`, `setDec`), and
  the `exp`/`log` elementary vectors in a new enclosure-idiom
  `tests/exp_log_fixture.rs`. The LGPL vector sets (`mpfi.itl`,
  `fi_lib.itl`) and the license-unstated `c-xsc.itl` are deliberately not
  transcribed; the exclusion and its license rationale are recorded in the
  reference registry.

### Known issues, surfaced by the lane and tracked as beads

- Interval division is `a * recip(b)`, one rounding looser than the tightest
  quotient on 56 vectors (enc-ghz); `pown`'s repeated squaring is loose for
  exponent magnitude three and above, and its negative-power overflow path
  collapses near the subnormal edge, 41 vectors (enc-5jj, falsifying the
  workspace decision record 0007's assumption that the corpus could not
  distinguish chain tightness). `pown_rev`'s root bisection leaves one-to-
  three-ulp brackets for exponent magnitude three and above, 44 vectors
  (enc-cov, an erratum owed to decision record 0006 part 3), and its
  negative-exponent path saturates the set-level reciprocal at the subnormal
  edge, 8 vectors (enc-ral). `set_dec` returns NaI where the standard's
  `setDec` clamps, 6 vectors (enc-2hd). The draft-era corpus propagates
  decorations through the two-output division where decision record 0006
  part 5 grades `trv`, 175 decoration-only divergences pending a
  final-standard reading (enc-pzd). The decorated conformance surface lacks
  numeric accessors, unary predicates, set operations, and
  `isCommonInterval`/`isMember` (enc-ks9). Every non-tight result above is a
  verified sound enclosure; the lane found no soundness violation anywhere
  in the corpus.

- The reverse operations on `Interval<F>` and `DecoratedInterval<F>`, in a new
  `reverse` module, each in its explicit-candidate form
  (`f_rev(c, x) = hull({ t in x : f(t) in c })`, the standard's candidate-less
  form recovered at `x = entire`): `sqr_rev`, `abs_rev`, `pown_rev`, `mul_rev`,
  and the two-output `mul_rev_to_pair` over bare `RoundFloat`; `sin_rev`,
  `cos_rev`, `tan_rev` behind `F: RoundInverseTrig`; `cosh_rev` behind
  `F: RoundInverseHyperbolic`; and `pow_rev1`, `pow_rev2` behind
  `F: RoundTranscendental`. The arithmetic reverses are the tightest
  representable result over any `RoundFloat` from directed arithmetic: the square
  and integer roots come from a two-phase value-domain bisection over the
  directed `pown` chain (geometric probes by directed `sqrt` while the bracket is
  wide, arithmetic probes once it is narrow, a multiplicative straddle refinement
  at the exit; never `RoundPow`, so the arm is exact on backends that lack it),
  and `mul_rev`/`mul_rev_to_pair` from the zero-straddle case table of the
  two-output division, with `mul_rev` the hull of the pair's two pieces each
  intersected with the candidate. The periodic reverses enumerate exactly the
  preimage arcs that can meet the candidate, wherever it sits: arc placement is
  by sound interval sums of doubled period-enclosure blocks (a galloping landing
  and a one-period walk, so no float arc counter can drift off the integers),
  degrading to the candidate-clipped maximal preimage when the window spans more
  than `TRIG_REV_ARC_CAP` periods or the accumulated pi-enclosure uncertainty
  smears adjacent arcs (half a period of position-enclosure width, near `10^15`
  periods over the fixture; a tightness loss, never a soundness loss). The power
  reverses are computed from the exponential and logarithm of a multiplication
  reverse. Every decorated reverse grades `trv` (a reverse image carries no
  definedness or continuity witness), with `ill` propagating. Decision record
  0006.

- Test lanes `tests/reverse_rev_fixture.rs`, `tests/reverse_mul_rev_fixture.rs`,
  and `tests/reverse_abs_rev_fixture.rs` transcribe the ITF1788
  `libieeep1788_rev.itl`, `libieeep1788_mul_rev.itl`, and `abs_rev.itl` vectors
  (the full `mulRevToPair` grid as a data table); structural cases assert exactly
  and the directed-root, division, and inverse-image arms assert the sound
  enclosure relationship on the loose fixture. `tests/reverse_property.rs` is the
  independent reverse-image soundness lane, arbitrated only through the forward
  battery: a witness whose forward image lands in the constraint must lie in the
  reverse result, and the `mul_rev_to_pair` split invariants (disjoint, ordered
  pieces; `mul_rev` equals the hull of the pair intersected with the candidate)
  hold. `tests/reverse_pow_rev_fixture.rs` transcribes `pow_rev.itl`, with two
  `#[ignore]`d KNOWN GAPs: the exp/ln reduction is sound but not tight at the
  0/1/inf logarithm-boundary value-empties and the `0^0`/`0^negative` undefined
  base, where a tight-empty result comes back as a sound superset; exact
  conformance there awaits the tight-`f64` lane or a bespoke general-interval
  power reverse. Soundness at those corners is discharged by the property lane.

- The elementary battery round two on `Interval<F>` and `DecoratedInterval<F>`,
  in a new `inverse` module: `asin`, `acos`, `atan`, `atan2` (behind
  `F: RoundInverseTrig`), `asinh`, `acosh`, `atanh` (behind
  `F: RoundInverseHyperbolic`), `exp2`, `exp10`, `log2`, `log10` (behind
  `F: RoundExpBases`), and `pown` over bare `RoundFloat` as directed integer
  power chains with the negative exponent taken as the set-level reciprocal. Ten
  arms are monotone endpoint images with set-based domain restriction (`acos`
  antitone); `atan2` is a directed corner reduction derived from the vanishing
  gradient, with the full-hull branch-cut crossing, the `+0` normalization for
  the `+pi` convention, and the origin dropping the decoration to `trv`. Test
  lanes `tests/inverse_fixture.rs` and `tests/atan2_fixture.rs` transcribe the
  ITF1788 `libieeep1788_elem.itl` and `atan2.itl` vectors. The three new traits
  are re-exported beside `RoundFloat`, as is `RoundReduction` for the
  conformance surface. Workspace decision record 0007.

- The transcendental growth arms on `Interval<F>` and `DecoratedInterval<F>`, in a
  new `trig` module: `sin`, `cos`, `tan` (behind `F: RoundFloat + RoundTrig`),
  `sinh`, `cosh`, `tanh` (behind `F: RoundFloat + RoundHyperbolic`), and `pow` and
  `rootn` (behind `F: RoundFloat + RoundPow`); the three new extension traits are
  re-exported beside `RoundFloat`. `sin`/`cos` follow the critical-point rule of
  decision record 0005: an input spanning a full period (or unbounded) maps to
  `[-1, 1]`, and otherwise the maxima and minima are admitted by testing whether a
  grid point's sound enclosure (from the pi bracket and a directed index) meets the
  input, so ambiguity widens toward the extremum and never guesses. `tan` returns
  `Entire` when a pole enclosure may lie inside and the monotone branch otherwise.
  `pow(X, Y)` decomposes by monotonicity in each argument, four-corner style
  through the backend's directed `pow` on the nonnegative base domain (decision
  record 0005 part 3; the exp/ln composition is the affine seam of part 4, not
  this arm), with the base's zero column (`0^y = 0`, defined only for `y > 0`)
  folded in; `rootn` splits on the parity of `n`. The decorated forms grade `com`/`dac`
  by boundedness for the everywhere-continuous families, `trv` where `tan` hits a
  pole or `pow`/`rootn` violates its base domain. Test lane
  `tests/trig_pow_fixture.rs` transcribes the ITF1788 `libieeep1788_elem.itl` and
  `c-xsc.itl` vectors (as the sound enclosure relationship on the loose fixture)
  and adds pointwise-enclosure, full-period-shortcut, critical-point-stress,
  `cosh`-minimum, and pole-straddle property lanes. Workspace decision record 0005.
- The text I/O surface, `interval-1788`'s Level 1 literal grammar for `f64`
  (ledger item 8), behind the `fixture` feature and with its threat model named
  in the module doc: the input is attacker-supplied bytes, and the worst it can
  force is a `TextError` value or a sound enclosing interval, never a panic,
  unbounded work, an allocation, or an unsound interval.
  - `nums_to_interval` (the checked endpoint constructor's standard spelling) and
    `text_to_interval` on `Interval<f64>` and `DecoratedInterval<f64>`, parsing
    the bracketed forms (`[empty]`, `[entire]`, `[l, u]`, the half-open and point
    forms, `[nai]`), the uncertain form (`m?`, `m?r`, one-sided `u`/`d`, the
    infinite-radius `??`, with a trailing exponent), and decimal, hex-float, and
    rational `p/q` number literals, with whitespace tolerance and case folding.
    The decoration suffixes `_com`/`_dac`/`_def`/`_trv` are validated against the
    literal's mathematical interval and combined with the stored interval's
    strongest decoration by the lattice meet, so an overflowing literal demotes a
    requested `com` to `dac`; `_ill` is reserved for `[nai]`.
  - The rigor-critical core is a directed decimal-to-`f64` conversion. Every
    finite literal reduces to a signed ratio of integers, and an allocation-free
    fixed-capacity big integer rounds it to the directed endpoint by exact long
    division: the lower endpoint toward minus infinity, the upper toward plus
    infinity, tightest, with the exactness decision made entirely in integer
    arithmetic (no floating point in the comparison). Hex literals are exact by
    construction through the same division with a power-of-two denominator. Work
    is bounded by three named length caps; over-length input is a `TextError`.
  - Output: a `Display` for `Interval<f64>` (`intervalToText`, whose reread
    encloses) and the exact pair `interval_to_exact` / `exact_to_interval` (clause
    13.4), which round-trips any interval bit-for-bit through its hex-float text.
    The output side has zero corpus coverage (registry gap 2), so its correctness
    rests on the round-trip properties in the test lane, which the module doc
    records as the substitute for the missing vectors.
  - The failures the standard raises as the `UndefinedOperation` signal are
    reported as `Result::Err` instead (a documented divergence); a
    `PossiblyUndefinedOperation` case is a valid `Ok` result. The `[nai]` literal
    is the returned `NaI` value.
  - `tests/text_io_fixture.rs`: the constructor, exception, and class
    `textToInterval`/`numsToInterval` vectors transcribed (valid, malformed,
    decorated, hex, and uncertain), the exceptions-file signals mapped to `Err`,
    plus property lanes: the exact hex round-trip, `Display` reread enclosure, the
    directed decimal core bracketing the correctly-rounded `str::parse` oracle to
    one ulp (including at the subnormal and overflow edges), and a no-panic lane
    over random ASCII bytes.

- The numeric and boolean function completion (the Phase C deferrals).
  - Numeric `mid`, `rad`, and `mid_rad` on `Interval<F>`, each `Option` (`None`
    for the empty interval). `mid_rad` is the core; `mid` and `rad` project it, so
    the radius always matches the emitted midpoint. The defining property
    `[mid - rad, mid + rad]` encloses the interval holds structurally: the radius
    is the larger one-sided distance to the actual midpoint, rounded up. The
    midpoint is overflow-safe (the halved form `inf/2 + sup/2`, never `(inf + sup)
    / 2`). `Entire` centers at zero, a singleton at its point, a
    symmetric-about-zero interval at zero, each exactly. A directed-rounding
    surface cannot reproduce the Level 2 nearest-float datum in general, nor reach
    the `realmax` a half-unbounded interval pins (no largest-finite constant on
    `RoundFloat`); those return the finite endpoint with an infinite radius (sound,
    documented at `mid_rad`), and the tightest midpoint is deferred to the
    conformance lane over a correctly-rounded backend.
  - The boolean orderings `equal`, `less`, `strict_less`, `precedes`, and
    `strict_precedes` on `Interval<F>`, exact endpoint comparisons with the empty
    conventions the vectors pin (`less` and `strict_less`: both empty true, one
    empty false; `precedes` and `strict_precedes`: any empty true; `equal`
    coincides with `PartialEq`). The two strict forms treat matching infinite
    endpoints as satisfying the strict clause.
  - The sixteen-state `overlap` relation: a new public `enum Overlap` and
    `Interval::overlap`, the standard's mutually exclusive Allen-algebra states by
    pure endpoint case analysis. Exhaustiveness is guaranteed by a total match over
    the two gap tests plus the nine three-way endpoint-comparison combinations.
  - Decorated forms of all eight orderings and `overlap` on `DecoratedInterval<F>`,
    reading the interval part; the boolean relations return false when either
    operand is `NaI`, per the vectors.
  - Test lane `tests/numeric_boolean_fixture.rs` transcribes the ITF1788
    `libieeep1788_num.itl`, `libieeep1788_bool.itl`, and
    `libieeep1788_overlap.itl` vectors (numeric functions pinned exactly where the
    derivation guarantees it, the enclosure property asserted otherwise; the
    orderings and overlap pinned exactly, empty tables included) and adds the
    enclosure-property, midpoint-membership, ordering-coherence, and
    overlap-consistency property lanes.
- The cancellative operations `cancel_minus` and `cancel_plus` on `Interval<F>`
  and `DecoratedInterval<F>` (bare `F: RoundFloat`). `cancel_minus(x, y)` recovers
  the tightest `z` with `y + z` enclosing `x`; `cancel_plus(x, y)` is
  `cancel_minus(x, -y)`. The width gate certifies `wid(y) <= wid(x)` through the
  addition rearrangement `x_lo + y_hi <= x_hi + y_lo` (exact on a
  correctly-rounded backend, conservative toward Entire on the loose fixture), and
  the endpoints round outward per the defining property so the recovery stays
  sound. Neither is a fundamental-theorem operation, so the decorated forms grade
  `trv` throughout; `NaI` propagates. Test lane `tests/cancel_fixture.rs`
  transcribes the ITF1788 `libieeep1788_cancel.itl` vectors and adds
  defining-property, round-trip, `cancel_plus` identity, and totality lanes.
- The point-function battery: `abs`, `min`, `max`, and `sign` on `Interval<F>`
  and `DecoratedInterval<F>` (bare `F: RoundFloat`), and `ceil`, `floor`,
  `trunc`, `round_ties_to_even`, and `round_ties_to_away` behind
  `F: RoundFloat + RoundInteger` (the new exact extension trait, re-exported
  beside `RoundFloat`). Each is a monotone step or lattice function, so the
  image is an exact endpoint computation with no outward widening. These are the
  first operations to earn the `def` decoration from the operation itself: the
  six discontinuous ones grade `def` when the box spans a jump, `com`/`dac`
  otherwise, following the ITF1788 `libieeep1788_elem.itl` vectors. Test lane
  `tests/point_functions_fixture.rs` transcribes the conformance vectors and adds
  pointwise-enclosure, exactness, and decoration property tests.
- Phase E opener, the first elementary functions.
  - `exp` and `ln` on `Interval<F>` and `DecoratedInterval<F>`, behind
    `F: RoundFloat + RoundTranscendental` (the extension trait, re-exported
    beside `RoundFloat`). Monotone endpoint images with outward rounding;
    `ln` domain-restricted to `(0, +inf)` per the set-based flavor; the
    decorated versions demote `com` to `dac` when the result is unbounded
    (overflow, or an input touching `ln`'s domain edge). Decision record 0004.
  - Property lane `tests/elementary_fixture.rs` (pointwise enclosure over the
    f64 fixture) and oracle certification against pfloat-libm's
    correctly-rounded values in the workspace's nightly lane.

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
  - Then deferred, now completed above (see the Unreleased entry): `mid` and
    `rad`, and the ordering relations (`equal`, `less`, `precedes` and their
    strict forms), which needed the Level 2 conventions for their empty and
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

- `mid`, `rad`, and `mid_rad` now sit behind the `RoundLargestFinite` capability
  bound and follow IEEE 1788's Level 2 realmax convention on half-unbounded
  intervals: `mid([a, +inf]) = +LARGEST_FINITE`, `mid([-inf, b]) =
  -LARGEST_FINITE`, radius infinite, replacing the previous sound
  finite-endpoint approximation; the previously unreachable midpoint vectors are
  pinned bit-exact. Workspace decision record 0009.
- The decorated `exp` and `ln` wrappers drop their inline unbounded-result
  demotion, subsumed by the uniform demotion at the `pack` seam; behavior is
  unchanged, and the `ln` documentation now states the actual domain-check path
  (an input reaching zero grades `trv`; `ln` has no overflow demotion path).
- `RoundFloat` moved to the new `round-float` foundation crate and is re-exported
  here as `interval_1788::RoundFloat`, so downstream `impl RoundFloat for _`
  (the SMIL/ferrodec backend) keeps compiling unchanged. The `fixture` feature
  now pulls the f64 instance from `round-float`'s `f64` feature instead of
  defining it locally. The crate joined the `enclosure` workspace; the repository
  metadata points at the family. No behavior changed (66 unit and property tests
  and 12 Kani proofs unchanged).

  The flat path `interval_1788::RoundFloat` is preserved through the re-export.
  The deep path `interval_1788::round::RoundFloat` is gone, since the `round`
  module moved out; nothing in the family used it.

### Fixed

- Decorated `+`, `-`, `*`, `/`, `recip`, `sqr`, `sqrt`, and `mul_add` no longer
  pack `com` with an overflowed unbounded result. A bounded operation can reach
  `[lo, +inf]`, and `com` promises a bounded result, so the shared `pack` seam
  now demotes `com` to `dac` whenever the assembled interval is unbounded. This
  extends the overflow rule decision record 0004 established for `exp` and `ln`
  to every decorated operation; `mul_add` gains its decorated form here.
