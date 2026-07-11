# ADR-0005: Transcendental growth: per-family capability traits, reduction-aware margins, and the critical-point rule

**Status:** Accepted
**Date:** 2026-07-11

This record promotes the design-sensitive half of beads `enc-34n` (the per-function arms are the implementing half, deliberately mechanical after this record). Builds on round-float ADR-0001 (the `RoundTranscendental` extension trait), enclosure ADR-0002 (affine Chebyshev elementary functions), and the elementary-oracle lane. Gates the SMIL `enclosed-sin` family.

## Context

`RoundTranscendental` shipped with exp and ln: four methods, a fixture over libm with a derived relative margin (`8 · 2^-52`, dominating the musl 1.5 ulp goal with headroom), and an empirical oracle lane against pfloat-libm. Both consumers are monotone-friendly: the interval arms are two endpoint evaluations with outward rounding, and the affine fits ride one-sign-of-`f''` Chebyshev linearization. Growing to trig, hyperbolic, and pow breaks each of those comforts in a different place, and the three breaks are the design content of this record.

First, capability. A backend that can supply exp cheaply may not supply sin at the same quality on the same schedule, and trig drags two new obligations with it that exp never had: an enclosure of π (argument reduction is subtraction of multiples of π against a constant the format cannot represent exactly) and an exact `floor` (extracting the multiple). Second, accuracy trust. The exp margin's trust anchor is the musl accuracy table; for trig the anchor must stretch to cover argument reduction at large magnitudes, where a naive reduction loses all significance and the advertised ulp goal silently stops being about the mathematical function. Third, shape. sin and cos are periodic and non-monotone: the interval image is no longer two endpoint evaluations, the affine residual is no longer one-signed over an arbitrary range, and tan adds poles with IEEE 1788 set semantics that the ITF1788 vectors pin.

## Decision

Five parts: the trait shape, the fixture margin doctrine, the interval algorithm, the affine fit policy, and the oracle lane growth.

### 1. Per-family capability traits; π and floor ride the trait that needs them

Three new extension traits, each independently optional, each `: RoundFloat`:

- `RoundTrig`: `sin_down/up`, `cos_down/up`, `tan_down/up`, plus `pi_down()`/`pi_up()` (constants: the two floats bracketing π in the backend format) and `floor(self)` (exact in every float format). The reduction obligations live here because they are trig obligations: a backend that never implements trig owes no π enclosure, and `RoundFloat` stays untouched so no shipped impl breaks.
- `RoundHyperbolic`: `sinh_down/up`, `cosh_down/up`, `tanh_down/up`. No constants needed; the family is reduction-free.
- `RoundPow`: `pow_down/up(self, exp)`, `rootn_down/up(self, n)`. Domain preconditions per IEEE 1788 (positive base for real pow; the interval layer owns set semantics for the edges).

`RoundTranscendental` keeps its name and its exp/ln surface unchanged: renaming a shipped trait is an identity change with no compensating benefit, and the mild wart (the general name on the narrow trait) is recorded here so it is not re-litigated.

**Rejected: growing `RoundTranscendental` in place.** It breaks every shipped impl (SMIL's `MyFloat` among them) at one stroke and couples three families that backends will land at different times. **Rejected: sound-but-vacuous defaults** (`sin_down` defaulting to −1 is sound). A capability trait whose default silently returns the worst legal answer converts a missing capability from a compile error into a quality regression the consumer discovers in a rendered plot; the honesty cost exceeds the convenience.

### 2. Fixture margins: the same doctrine, plus range clamps, plus the reduction anchor

The f64 fixture extends the shipped doctrine unchanged in form: relative margin dominating the documented musl accuracy goal with headroom, then a trailing outward step. Three additions, each a trig or hyperbolic fact the exp/ln fixture never faced:

- **Range clamps.** After widening: `sin`/`cos` results clamp into [−1, 1], `tanh` into (−1, 1), `cosh` below at 1. The clamp is sound (the mathematical range is a theorem, not an approximation) and restores tightness exactly where the margin would otherwise push past an extremum.
- **The reduction trust anchor is named.** musl's trig inherits the fdlibm Payne-Hanek reduction, so the ulp goal is claimed relative to the mathematical result at every finite magnitude, including near zeros of sin where the result is tiny and a relative margin is correspondingly tiny. That claim, not the generic accuracy table line, is what the margin leans on for trig; the registry entry `musl-libm-accuracy` gains this statement, and the oracle lane discharges it empirically at large arguments and near critical points (part 5). If the oracle ever falsifies it, the fixture margin for trig is wrong, not merely loose.
- **Overflow edges follow the exp precedent**: `sinh`/`cosh` at ±710-ish return the [largest_finite, +∞] style bounds the exp fixture established; `tan` near poles follows libm's finite results (the fixture never manufactures a pole: pole semantics are the interval layer's, part 3).

Per-function margin constants are implementing decisions, pinned by the oracle; this record fixes the doctrine, not the numbers.

### 3. Interval arms: the critical-point rule, derived, with ambiguity widening

For a non-monotone `f` over `[a, b]` the image is determined by the endpoint values plus any interior critical points. The algorithm this record fixes, derived from that statement (prior implementations are behavior oracles only; the ITF1788 vectors are the conformance pin):

1. If the interval width (computed `_up`) is at least one full period (computed against `pi_down`), the image is the full range: `[−1, 1]` exactly for sin and cos. Tight and immediate.
2. Otherwise enumerate the candidate critical points `c_k` (maxima and minima of the family: `π/2 + kπ` for sin, `kπ` for cos, extremum at 0 for cosh) whose ENCLOSURES — computed from `k` (via `floor` of directed division) and the π enclosure — could intersect `[a, b]`.
3. **Ambiguity widens, never guesses.** If a critical point's enclosure intersects `[a, b]`, the corresponding range extremum (+1 or −1, or `cosh(0) = 1`) enters the result, whether or not the true critical point actually lies inside. If the directed computations of `k` disagree at a quadrant boundary, both candidates are enumerated. The cost is tightness within a π-enclosure's width of a critical point; the benefit is that no rounding of the reduction can ever produce a non-enclosure. This is the trig instance of the same rule that made ADR-0059's collapse sound in SMIL: when correlation to the truth is uncertain, pay width, never soundness.
4. Endpoints contribute `f_down(a) rmin f_down(b)` and `f_up(a) rmax f_up(b)` exactly as the monotone arms do.

`tan` over an interval whose interior may contain a pole (`π/2 + kπ` enclosure intersects it) returns entire with the decoration transition the ITF1788 `libieeep1788_elem.itl` vectors specify; the vectors are the acceptance tests for every edge of this paragraph. `sinh`/`tanh` are monotone (exp-shaped arms); `cosh` uses the one-extremum case of the rule. `pow` decomposes by monotonicity in each argument over the sign cases 1788 defines, four-corner style where both arguments are intervals; `rootn` splits on parity of n. The full case tables are implementing work against the vectors.

### 4. Affine fits: per-arc Chebyshev, `None` past one arc

The shipped Chebyshev builder requires a one-signed residual second derivative. For sin and cos that holds exactly on arcs between inflection points (`[kπ, (k+1)π]` for sin). The affine arms therefore: reduce the input form's range, and if it lies within a single concavity arc, fit exactly as exp does (secant slope, interior tangent residual extremum, endpoints); if it spans an arc boundary, return `None` and let the caller fall back — the same convention ADR-0002 fixed for domain violations and the sqrt zero-touch decline. The SMIL consumer already computes both affine and interval candidates and keeps the tighter sound one, so the fallback is a tightness downgrade on wide ranges, not a capability hole. `sinh`/`cosh`/`tanh` are one-signed on the half-lines and split at 0; pow fits ride the exp/ln composition on the positive domain in v1 rather than a bespoke bivariate fit (recorded as the frugal choice; a direct fit is future work if the composition's width proves material).

### 5. Oracle lane growth: stress where the new risk is

Each new function joins the elementary-oracle nightly lane against pfloat-libm with grids aimed at its specific hazard, not just the generic sweep: trig grids include large arguments (reduction stress well past 2^50) and arguments within 1e-9 of critical points and poles (the ambiguity-widening paths); hyperbolic grids include the overflow shoulders; pow grids sweep the (base, exponent) plane edges and the domain boundary rays. The fixture margins, the interval arms, and the affine fits all discharge against the same oracle, as exp/ln do today.

## Consequences

### Positive

- Backends opt into exactly the families they can honestly supply; nothing shipped breaks; SMIL gains `enclosed-sin`/`cos`/`tan` the release after the interval arms land, with the BAND plotter as the visible consumer.
- The soundness argument for trig is localized to two auditable places: the fixture margin (empirically discharged) and the ambiguity-widening rule (structurally incapable of guessing wrong, only of being loose).
- The ITF1788 vectors, already vendored and license-audited, become live acceptance tests for the hardest semantics (tan poles, decorations) instead of roadmap artifacts.

### Negative

- Three ways this design could be wrong, recorded per the inversion discipline. (a) Trait proliferation: generic code over "a float that can do everything" now writes four bounds; if that friction recurs, a blanket umbrella alias is the remedy, added then, not preemptively. (b) The critical-point rule's tightness cost concentrates exactly where users look (extrema of plotted sin); if the π-enclosure width proves visible in BAND plots, the remedy is a tighter π pair per backend, never a change to the widening rule. (c) The affine `None` past one arc may fire on most real BAND ranges, making the affine trig surface nearly dormant; that is a known, sound underdelivery, and the interval fallback carries the feature; measuring the fire rate is part of the implementing lane's report.
- `RoundTrig` carrying `floor` and π duplicates what some future numeric tower might want elsewhere; the duplication is accepted to keep `RoundFloat` frozen.

### Neutral

- Per-function margin constants, the pow/rootn case tables, and the per-arc fit derivations are implementing decisions, filed as the mechanical half of `enc-34n` and pinned by oracle and vectors, not by this record.
- The `RoundTranscendental` name stays on the exp/ln pair; the wart is recorded above.

## References

- beads `enc-34n`: the working record this promotes (design half).
- `crates/round-float/docs/decisions/0001-round-transcendental-extension-trait.md`: the extension-trait rationale this extends to three families.
- `docs/decisions/0002-affine-elementary-functions.md`: the Chebyshev builder, the `None` convention, and the oracle anchoring the fits inherit.
- `docs/references/musl-libm-accuracy.md`: the accuracy trust anchor, gaining the reduction statement in the implementing slice.
- `docs/references/itf1788-framework.md` (vendored `libieeep1788_elem.itl`): the conformance vectors that pin tan poles, decorations, and the elementary edge semantics.
- `docs/references/pfloat-libm-oracle.md`: the correctly-rounded truth source for every new oracle grid.
- `crates/round-float/src/f64_impl.rs:104-207` (the shipped margin doctrine), `crates/interval-1788/src/elementary.rs` (the monotone arms the new cases join), `crates/affine-arith/src/elementary.rs:40-72` (the linearize builder the per-arc fits reuse).
- SMIL side: `docs/decisions/0056-*` (affine-backed Enclosed) and the BAND plotter, the consumers this unblocks.
