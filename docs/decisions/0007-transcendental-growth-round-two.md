# ADR-0007: Transcendental growth round two: the inverse families, the base variants, and atan2

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-16

This record promotes the design-sensitive half of bead `enc-dw4` (the
per-function arms are the implementing half, deliberately mechanical after
this record), in the mold of ADR-0005, whose doctrines it extends rather than
re-argues. It exists because the interval-1788 v1.0 road (that crate's
ADR-0005, ledger item 2) requires an elementary surface the first growth
round never scoped: the corpus and the P1788.1 required-operations table
demand `asin`, `acos`, `atan`, `atan2`, `asinh`, `acosh`, `atanh`, `exp2`,
`exp10`, `log2`, `log10`, and `pown`, even in the simplified profile.

## Context

Round one (ADR-0005) settled the hard general questions: per-family
capability traits over a frozen `RoundFloat`, the fixture margin doctrine
(relative margin dominating a documented musl accuracy goal, range clamps,
overflow edges), the critical-point rule for non-monotone images, and oracle
grids aimed at each function's specific hazard. Round two inherits all of
it, and differs in three facts that are this record's design content:

1. **Ten of the eleven functions are monotone on their domains.** No
   argument reduction, no critical-point enumeration: the interval arms are
   endpoint images with domain restriction, the shape `exp`/`ln` fixed.
   `atan2` is the exception, and it is a two-argument function with a branch
   cut rather than a periodic one, so it needs its own rule, not round one's.
2. **The musl accuracy goal excludes the arc hyperbolics.** The registry's
   verbatim quote of the musl statement names "arc hyperbolics" among the
   exceptions to the sub-1.5-ulp goal. For `asinh`/`acosh`/`atanh` the
   shipped margin doctrine has no vendor claim to dominate; their trust
   story must be different, and saying so is the point.
3. **The functions split into three capability families, not one.** A
   backend may supply inverse trig at quality without touching the arc
   hyperbolics (musl itself treats them differently), and the base-2/10
   exponential family is independent of both. Round one's per-family
   opt-in argument applies unchanged.

## Decision

### 1. Three new extension traits; the seams of round one stay frozen

Each independently optional, each `: RoundFloat`, mirroring ADR-0005 part 1:

- `RoundInverseTrig`: `asin_down/up`, `acos_down/up`, `atan_down/up`,
  `atan2_down/up(self, other)` (argument order as `f64::atan2`: `self` is
  `y`), plus `pi_down()`/`pi_up()`. The π enclosure rides this trait for the
  same reason it rides `RoundTrig`: the range clamps of `asin`/`acos`/
  `atan`/`atan2` are stated in π, and a backend implementing only this
  family owes only this obligation. A backend implementing both trig traits
  supplies the pair twice; the duplication is the accepted cost of keeping
  the traits independent, exactly ADR-0005's `floor` trade.
- `RoundInverseHyperbolic`: `asinh_down/up`, `acosh_down/up`,
  `atanh_down/up`. No constants.
- `RoundExpBases`: `exp2_down/up`, `exp10_down/up`, `log2_down/up`,
  `log10_down/up`. No constants.

**Rejected: growing `RoundTrig`/`RoundHyperbolic`/`RoundTranscendental` in
place.** The shipped trait breaks SMIL's impl (round one's argument,
unchanged); the round-one traits are accepted seams that enc-34n is
implementing right now, and reopening them mid-implementation trades a
known-good design freeze for nothing the new traits do not deliver.

**`pown` grows no trait at all.** An integer power over an interval is
even/odd/negative-exponent case logic over directed `mul`/`sqr` chains that
bare `RoundFloat` already funds; a correctly rounded point `pown` on the
backend would buy tightness the chain already approaches and the corpus
vectors do not distinguish. It lands in interval-1788 over `F: RoundFloat`,
with `pown(x, -n)` as the set-level reciprocal of `pown(x, n)` (zero
straddling gives the unbounded results the vectors pin). The v1.0 ledger
(interval-1788 ADR-0005 item 2) is amended to name it, closing a gap this
design surfaced: `pown` appeared in the corpus enumeration but in no ledger
item.

### 2. Fixture margins: the shipped doctrine where the goal covers, a
measured-and-pinned margin where it does not

For the goal-covered functions (`asin`, `acos`, `atan`, `atan2`, `exp2`,
`exp10`, `log2`, `log10`): the shipped doctrine verbatim. Relative margin
dominating the musl 1.5 ulp goal with at least fivefold headroom, trailing
outward step for the subnormal floor, and range clamps where the
mathematical range is a theorem: `asin` into the `pi` half-bracket,
`acos` into `[0, pi_up]`, `atan` strictly inside the `pi/2` brackets,
`atan2` into `(-pi_up, pi_up]`, `exp2`/`exp10` clamped above zero as `exp`
is, `acosh` clamped below at zero.

For the arc hyperbolics, the doctrine changes and the change is documented
at the constant: there is no vendor goal to dominate, so the margin is SET
from measured error against the pfloat-libm oracle over the hazard grids,
with at least fivefold headroom over the worst observed error, and the
module comment states that its anchor is empirical, full stop. A margin
with no stated external claim behind it is weaker than round one's, and
pretending otherwise by reusing the same constant silently would be the
failure; the oracle lane is not a discharge of an assumption here, it is
the assumption. The musl registry entry gains this consumer and the
statement that round two leans on the exception wording.

Domain edges follow the `ln` precedent: out-of-domain arguments pass the
libm sentinel through (`asin`/`acos` outside `[-1, 1]`, `acosh` below 1,
`atanh` outside `(-1, 1)`), and the caller owns the domain guard;
`atanh` at the open endpoints and the log family at zero produce the
unbounded results whose decorated demotion is the interval layer's job.

### 3. Interval arms: monotone endpoint images, plus the atan2 rule

Ten functions take the `exp`/`ln` arm shape: directed endpoint images,
domain restriction per the set-based flavor (an input wholly outside the
domain has the empty image; an input touching the boundary restricts), and
the crate ADR-0004 decoration pattern (demote `com` on an unbounded result;
`trv`/`def` transitions where the domain restriction bit, pinned by the
`libieeep1788_elem.itl` vectors). `acos` is antitone; the arm swaps
endpoints, nothing else.

`atan2` gets the one new rule, derived from the image of the principal
angle over a box, with ADR-0005's ambiguity discipline:

- Boxes wholly inside an open half-plane or quadrant: the image endpoints
  sit at box corners (the function is monotone in each argument at fixed
  sign of the other), evaluated with directed rounding; which corners is a
  fixed table per quadrant, derived once in the implementing slice and
  checked against the `atan2.itl` vectors.
- Boxes meeting the branch cut (the closed negative `x` half-line with
  `y` spanning zero): the true image is two arcs meeting ±π; the set-based
  hull is taken, and any uncertainty from the π enclosure widens outward,
  never guesses (ADR-0005 rule 3, verbatim).
- Boxes containing the origin: the image is taken over the domain minus
  the undefined point and the decoration drops to `trv`, per the vectors.
- Discontinuity crossing without the origin: value hull as above with the
  decoration capped at `def`.

`pown` case tables (even/odd exponent, negative exponent, zero-straddling
bases) are implementing work against the vectors, per part 1's home.

### 4. Affine fits: deferred wholesale

None of the eleven functions gets an affine Chebyshev fit in round two. No
consumer demands correlated tightness through them (SMIL's
tightest-sound-wins falls back to the interval arm, which is exact per-op),
and the odd inverse functions (`asin`, `atanh`, `asinh`) carry an
inflection at zero that would force per-arc splitting machinery for zero
present benefit. Recorded as the frugal deferral with a named trigger: the
first consumer that measures a material width loss through an interval
fallback on these functions reopens the question, and the per-arc pattern
of ADR-0005 part 4 is the shape it would take.

### 5. Oracle lane growth: the exception family gets the densest grids

Every function joins the nightly elementary-oracle lane against
pfloat-libm, grids aimed at hazards: domain edges (`asin`/`acos` within
1e-9 of ±1, `acosh` approaching 1 where the result vanishes, `atanh`
approaching ±1 where it diverges), results near zero (`log2`/`log10` near
1, where a relative claim is most delicate), the `atan2` branch cut and
origin neighborhoods, and the `exp2`/`exp10` overflow shoulders. The arc
hyperbolics get the densest grids and a stronger role: their fixture margin
constant is derived from these measurements (part 2), so the lane is
promoted from check to anchor for exactly that family.

The oracle's own coverage was verified at the workspace level on
2026-07-16 (the pfloat workspace advertises the full function set,
inverses included). The implementing slice verifies the pinned revision
exposes each function and bumps the pin if not; if any function is missing
at every available revision, the named fallback reference is `astro-float`
(pure Rust, stable toolchain, correctly rounded elementary functions), per
the workspace tooling default of pure Rust dev dependencies.

## Consequences

### Positive

- The v1.0 elementary battery is fully designed: round one (ADR-0005) plus
  this record covers every function the corpus requires, and every arm is
  mechanical against fixed seams. The `pown` ledger gap is closed.
- The monotone shape of ten of eleven functions makes round two cheaper per
  function than round one despite the longer list; the design cost
  concentrates in `atan2` and the arc-hyperbolic trust story, which is
  where this record spent its words.

### Negative

- Three ways this design could be wrong, per the inversion discipline:
  (a) **The empirically anchored margin is the weakest link in the fixture's
  soundness story.** A libm regression in an arc hyperbolic after the
  margin is pinned would not be caught by any vendor claim, only by the
  oracle lane; the lane therefore runs in CI, not on demand, and the margin
  comment names the pinned libm crate version its measurement covered.
  (b) **Six extension traits now exist.** ADR-0005's trait-proliferation
  remedy (a blanket umbrella alias, added when friction is felt, not
  preemptively) is restated and inherited unchanged; the trigger is generic
  code writing four-plus bounds in anger.
  (c) **The atan2 corner table is derived once and easy to get wrong in one
  quadrant.** The mitigation is that the table is checked against the
  dedicated `atan2.itl` vector file, which exists precisely because other
  implementations got these corners wrong.
- The affine surface of round two is deliberately empty; downstream reads
  of "elementary functions: done" must not assume affine coverage. The
  deferral and its trigger are stated in part 4.

### Neutral

- Per-function margin constants, the atan2 corner table, and the `pown`
  case tables are implementing decisions, pinned by oracle and vectors.
- interval-1788 keeps `ln` as the natural-log name (idiomatic Rust, matching
  `f64::ln`); the conformance document maps the standard's `log` to it. The
  new arms take the standard's names (`log2`, `log10`, `atan2`).

## References

- ADR-0005 (round one: the traits, the margin doctrine, the critical-point
  rule, the oracle pattern this record extends); round-float ADR-0001 (the
  extension-trait rationale).
- interval-1788 ADR-0005 (the v1.0 road; ledger item 2 is this design's
  charter, amended by it for `pown`); its ADR-0004 (decoration demotion).
- Registry: `docs/references/musl-libm-accuracy.md` (the verbatim goal AND
  its arc-hyperbolic exception, the load-bearing fact of part 2);
  `docs/references/pfloat-libm-oracle.md` (the truth source);
  `docs/references/itf1788-framework.md` (vendored `libieeep1788_elem.itl`
  and `atan2.itl`, the acceptance vectors);
  `docs/references/p1788-1-d98-draft.md` (Table 4.1, which requires this
  battery even in the simplified profile).
- Beads: enc-dw4 (this design and its implementing half), enc-34n (round
  one, untouched), enc-ac4 (the conformance lane consumer).
