# ADR-0006: Reverse operations: reverse images, the two-interval division, and the periodic preimage rule

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-18

This record is the design half of bead `enc-2pw`, in the mold of the
workspace ADR-0007: it fixes every seam so the per-operation arms are
mechanical, and it spends its words where the design risk concentrates. The
v1.0 road (ADR-0005, ledger item on reverse operations) requires the
reverse-mode battery of the standard: `sqrRev`, `absRev`, `pownRev`,
`sinRev`, `cosRev`, `tanRev`, `coshRev`, `mulRev`, `mulRevToPair`,
`powRev1`, and `powRev2`, each in its candidate-set form. The acceptance
vectors are already vendored: `libieeep1788_rev.itl`,
`libieeep1788_mul_rev.itl`, `abs_rev.itl`, and `pow_rev.itl` under the
ITF1788 registry entry.

## Context

Reverse operations answer the constraint question the forward battery
cannot: given that `f(t)` lands in `c`, where in the candidate set `x` can
`t` have been? They are the narrowing primitives of every interval
constraint propagator, and the standard requires them even in the
simplified profile. Three facts shape the design:

1. **The semantic ground is one uniform set definition.** Every unary
   reverse is the hull of a reverse image intersected with the candidate:
   `fRev(c, x) = hull({t ∈ x : f(t) ∈ c})`. The binary and ternary forms
   quantify over the extra operand: `mulRev(b, c, x) =
   hull({t ∈ x : ∃ b₀ ∈ b, b₀·t ∈ c})`, and `powRev1`/`powRev2` re-read
   the same shape for each argument of `pow`. Nothing else is needed; every
   arm below is a derivation from this definition, and any conflict between
   a derivation and a vendored vector is resolved by re-deriving, then by
   the vector.
2. **The operations split by what funds them.** The arithmetic reverses
   (`sqrRev`, `absRev`, `pownRev`, `mulRev`, `mulRevToPair`) need only
   directed arithmetic that bare `RoundFloat` already carries. The
   trigonometric reverses need the inverse images that `RoundInverseTrig`
   supplies (`asin`, `acos`, `atan`, and the π enclosure), `coshRev` needs
   `RoundInverseHyperbolic`, and the `pow` reverses need
   `RoundTranscendental`. No new backend trait is required, and part 3
   records the one place that conclusion was in doubt.
3. **The periodic preimage is the design risk.** `sinRev`, `cosRev`, and
   `tanRev` have preimages that are countable unions of arcs; the result
   hulls the arcs that meet the candidate. Enumerating those arcs with a
   π *enclosure* rather than an exact π is where a wrong design would be
   silently unsound, so the rule gets its own part, in the spirit of the
   workspace ADR-0007's `atan2` rule.

## Decision

### 1. Home, names, and the candidate convention

The operations land in a new `reverse.rs` module as methods on `Interval`,
with decorated twins on `DecoratedInterval`, following the crate's
two-layer pattern. Names are the standard's, snake_cased: `sqr_rev`,
`abs_rev`, `pown_rev`, `sin_rev`, `cos_rev`, `tan_rev`, `cosh_rev`,
`mul_rev`, `mul_rev_to_pair`, `pow_rev1`, `pow_rev2`.

The method receiver is the standard's first argument and the remaining
arguments follow in the standard's order: `c.sqr_rev(x)`,
`b.mul_rev(c, x)`, `b.mul_rev_to_pair(c)`, `b.pow_rev1(c, x)`,
`a.pow_rev2(c, x)`, `c.pown_rev(x, n)` (the integer exponent trails, as
`pown`'s does). Positional fidelity makes the vector translation
mechanical and the conformance document's mapping one line per op.

**One method per operation; the candidate is always explicit.** The
standard's candidate-less form is the same operation at `x = entire`, so
the crate ships only the full form and the conformance document maps both
standard forms onto it. Rejected: a second candidate-less method per op
(eleven more methods whose bodies are one call with the entire interval;
the dominant consumer, a propagator, always has a candidate), and an
`Option` candidate (a flag argument wearing a type).

**`mul_rev_to_pair` returns `(Interval, Interval)`**, the standard's
ordered pair: the first element is the lower piece, the second is empty
unless the reverse image genuinely splits (division by a zero-straddling
`b`), in which case both pieces are nonempty and disjoint with the first
wholly below the second. Rejected: a two-variant enum
(`Connected`/`Split`). The pair is the standard's own shape, the vectors
pin both slots on every line including the `[empty] [empty]` cases, and
the enum would make the mechanical translation of every vector line pass
through a constructor the standard does not have. The invariant the enum
would have encoded (second nonempty implies split) is stated in the doc
comment and asserted in the property lane instead.

### 2. Tightness doctrine: tightest where arithmetic funds it, sound elsewhere

The arithmetic reverses are computed to the tightest representable result
over any `RoundFloat`: `sqrRev` from directed `sqrt` on the clipped
constraint (`c ∩ [0, ∞)`, mirrored through zero, intersected with `x`),
`absRev` from the mirrored clip alone, `mulRev` and `mulRevToPair` from
directed division with the zero-straddle case table, and `pownRev` from
the integer root of part 3. Over `TightF64` these are bit-exact against
the vectors and the conformance lane (enc-ac4) asserts them so; over the
deliberately sound-not-tight fixture the fixture tests assert enclosure,
the established doctrine of the forward battery's fixture lane.

The transcendental reverses (`sin_rev`, `cos_rev`, `tan_rev`, `cosh_rev`,
`pow_rev1`, `pow_rev2`) inherit the fixture margins of the traits that
fund them; their results are sound and near-tight, never claimed tightest
over the fixture. Their vectors are asserted as enclosure in the fixture
lane, with the structural cases (empties, entires, domain clips) exact.

### 3. `pownRev` roots by bit bisection over `pown`; the existing `rootn` seam is deliberately not used

The even-exponent reverse needs `n`-th roots with directed rounding. The
crate does carry a root seam already, `RoundPow::rootn_down`/`rootn_up`
behind the forward `rootn`, but only the margin-widened `f64` fixture
implements `RoundPow`; `TightF64`, the conformance lane's backend, does
not. An arithmetic reverse op the vectors pin exactly must exist and be
exact over `TightF64`, so `pownRev` cannot ride `RoundPow` without either
narrowing its availability per backend or growing `TightF64` a
transcendental obligation this record has no charter to add.

The root is instead computed from bare `RoundFloat`, by bisection on the
`f64` bit pattern: the integer root in float space is the largest float
whose `pown` lower bound does not exceed the constraint endpoint, found
in at most 64 probes of the existing directed `pown` chain, with the
directed pair read off the root and its neighbor. Monotone `pown` bounds
and exact float comparisons make the bisection sound with no analytic
error term; over `TightF64` it is exact, and over the fixture it is
exactly as tight as the fixture's own `pown`.

Also rejected: the `exp(ln(c)/n)` composition (drags
`RoundTranscendental` into an arithmetic op and its margin into a result
that can be exact). The bisection is the frugal choice: slower per call
by a bounded factor of the probe count, pure, available on every
backend, and exact where the backend is. Odd exponents are monotone and
need one signed root; negative exponents reuse the positive case through
the set-level reciprocal exactly as forward `pown` does.

### 4. The periodic preimage rule

For `sin_rev`, `cos_rev`, `tan_rev` the preimage is a union of arcs
placed by π. The rule, extending the workspace ADR-0005's ambiguity
discipline (enclosures widen, never guess):

- The base arc comes from the directed inverse images on the clipped
  constraint (`asin` on `c ∩ [-1, 1]`, `acos` likewise, `atan` on all of
  `c`), evaluated with directed rounding.
- Arc `k` is the base arc reflected per the function's symmetry and
  translated by `k·π`, both endpoints computed with the π enclosure
  rounded outward. The result is the hull of the intersections of the
  translated arcs with the candidate `x`, enumerating exactly the `k`
  whose arcs can meet `x` (bounds from directed division of `x`'s
  endpoints by the π enclosure, widened by one on each side).
- **The degradation clause:** when the candidate's endpoints are so large
  that the accumulated π uncertainty (`|k|` times the enclosure width)
  reaches the gap between adjacent arcs, arc placement is no longer
  trustworthy and the result degrades to the sound fallback: the
  candidate clipped to the maximal preimage (`x` itself for `sin`/`cos`
  with nonempty clipped constraint, `x` minus nothing for `tan`). The
  threshold is a named constant derived in the implementing slice from
  the shipped π enclosure width, with the derivation in its doc comment.
  Degradation is a tightness loss, never a soundness loss.
- An empty clipped constraint gives the empty result before any
  enumeration; an unbounded candidate with nonempty clipped constraint
  short-circuits to the maximal preimage by the same fallback.

`cosh_rev` is not periodic: it is `sqrRev`'s shape with `acosh` in place
of `sqrt` (clip below 1, mirror, intersect), on `RoundInverseHyperbolic`'s
fixture margins.

### 5. Decorations: `trv` across the board

Every reverse operation in the decorated layer returns its bare result
decorated `trv` (with `ill` propagating as ever). A reverse image is a
set-level answer, not an evaluation: it carries no witness that `f` was
defined and continuous on the result, and the standard accordingly
assigns reverse operations no better than `trv`. The decorated vectors
pin this on every line (`_com`/`_dac`/`_def` inputs all produce `_trv`
outputs), so the decorated twins are one-line wrappers and the doctrine
is stated once at the module level rather than re-argued per op.

### 6. Verification lanes

- **Vector pinning:** the four vendored ITL files are hand-translated in
  the established fixture-test idiom (the `hx` hex-float parser, exact
  structural cases, enclosure for margin-widened results), one test file
  per ITL file. The implementing agent reports any vector that corrects a
  derivation, per the standing prompt discipline.
- **The reverse-image property lane** is the independent hedge, and it
  needs no oracle beyond the forward battery: for random `c`, `x`, and
  sample points `t ∈ x`, if `f(t) ∈ c` then `t` must lie in the reverse
  result (soundness of the enclosure); for `mul_rev`, sample `b₀ ∈ b`
  alongside. A second property pins the `mul_rev_to_pair` invariants
  (second piece nonempty only with first, disjointness, order) and that
  `mul_rev` equals the hull of the pair intersected with the candidate.
- **Bit-exactness over `TightF64`** for the arithmetic reverses waits for
  the conformance lane (enc-ac4), which asserts the vectors exactly
  there; the fixture lane's enclosure assertions land with this slice.

## Consequences

### Positive

- The last required-operation family of the v1.0 ledger is fully designed;
  with this record and the workspace ADR-0007 the corpus's operation
  surface is complete and every remaining arm is mechanical against fixed
  seams.
- The reverse-image definition gives every arm the same falsifiable
  soundness property, so one property-lane shape covers the whole family,
  including the two-interval division.

### Negative

- Three ways this design could be wrong, per the inversion discipline:
  (a) **An arc-enumeration off-by-one at a period boundary** would clip a
  preimage arc that grazes the candidate. Mitigations: the enumeration
  range is widened by one period on each side by construction, and the
  vectors plus the property lane's boundary-dense sampling exist to catch
  exactly this.
  (b) **The `mul_rev_to_pair` conventions (piece order, empty slots)**
  could diverge from the standard in an unpinned corner. Mitigation: the
  vectors pin every sign-case row including both empty slots; the
  property lane pins the pair-versus-`mul_rev` consistency.
  (c) **The degradation threshold could be set too generously**, silently
  costing tightness at large arguments the vectors never reach.
  Mitigation: the threshold constant's doc comment carries its
  derivation, and the property lane samples beyond the threshold to
  confirm soundness is unaffected either way.
- The reverse battery adds no affine surface, consistent with the
  workspace ADR-0007 part 4's deferral; a propagator wanting correlated
  narrowing reopens that record's trigger, not this one.

### Neutral

- The Rust surface diverges from the standard only in the collapsed
  candidate-less forms and snake_case names; the conformance document
  (enc-su3) records the mapping table.
- `pown_rev`'s probe-count constant and the periodic degradation
  threshold are implementing decisions, pinned by doc-comment derivations
  and the property lane.

## References

- IEEE 1788-2015 (pointer only, paywalled): reverse-mode elementary
  functions and the two-output division, §10.5.4; decorations of reverse
  operations. Free proxies: `pryce-2016-forthcoming-1788.md` and
  `kulisch-2008-p1788-draft.md` in the registry, and the
  simplified-profile draft `p1788-1-d98-draft.md`, whose
  required-operations table names this battery.
- Registry: `docs/references/itf1788-framework.md` (the vendored
  `libieeep1788_rev.itl`, `libieeep1788_mul_rev.itl`, `abs_rev.itl`,
  `pow_rev.itl`; Apache-2.0; the coverage gaps feed the README
  disclosure); `docs/references/libieeep1788-nehmeier.md` (the reference
  implementation whose behavior the vectors transcribe; an oracle for
  behavior, never a template for code).
- Workspace ADR-0005 (the ambiguity discipline the periodic rule
  extends); workspace ADR-0007 (the mold, the trait seams the
  transcendental reverses ride); ADR-0004 (the decoration doctrine the
  `trv` rule sits inside); ADR-0005 (the v1.0 ledger this record
  discharges an item of).
- Beads: enc-2pw (this design and its implementing half), enc-ac4 (the
  conformance lane that hardens the arithmetic reverses to bit-exact),
  enc-su3 (the conformance document consuming part 1's mapping).
