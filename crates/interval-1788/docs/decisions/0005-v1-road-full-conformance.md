# ADR-0005: v1.0 is full IEEE 1788-2015 set-based conformance

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-16

## Context

The crate is named `interval-1788`. That name consumes the canonical slot for
the standard in the Rust ecosystem, and the workspace's completeness value says
what that costs: a v1.0 whose surface stops short of the spec forces every
future user into workarounds, and a deliberately narrow v1.0 is honest only
when the path to completeness is incremental and stated.

The original v1.0 blocker list lived in session memory and is lost; this
record replaces it in tree, which is where it should have been. The state it
was reconstructed from: Phases A through D are complete (construction layer,
forward arithmetic, numeric/boolean/set functions with named deferrals,
decorations with Kani-proven lattice laws), Phase E is opened but not complete
(`exp`/`ln` only, per this crate's ADR-0004; trig, hyperbolics, and `pow` are
designed at workspace ADR-0005 and tracked as enc-34n), and Phase F (the
conformance lane and Level 2 story) is untouched. Two defects and gaps are
tracked: the decorated basic ops can pack `com` with an overflowed unbounded
result (enc-1ta), and the arithmetic ops lack the oracle lane the elementary
ops already have (enc-ebd).

Three roads were considered, and the choice was made interactively with
Parnell on 2026-07-16:

- **Road A: full 1788 before 1.0.** Complete the set-based flavor's operation
  surface and the conformance evidence first; v1.0 states a documentable
  conformance relationship to IEEE 1788-2015.
- **Road B: self-drawn narrow v1.0.** Ship the current Level 1 surface plus
  decorations as v1.0, no profile claim, the rest additive in 1.x. Fastest,
  but the narrow line is self-defined while the name says 1788.
- **Road C: anchor v1.0 to IEEE 1788.1-2017.** The simplified profile as the
  v1.0 conformance claim, full 1788 as the stated 1.x path. Externally
  anchored and the vendored vector corpus maps onto exactly that territory
  (see the registry entry for 1788.1).

## Decision

**Road A.** v1.0 ships when the set-based flavor of IEEE 1788-2015 is complete
enough to document a conformance claim, and not before. The version number is
the flag planted for the canonical name; it waits for the surface the name
promises. The 0.x tail this buys is accepted with eyes open, priced by the
ledger below.

### The blocker ledger

This ledger is the durable replacement for the lost memory list. The beads
tracker carries queue state (ordering, claims, discovery); this record carries
what must be true for v1.0. The operation enumeration is grounded in the
vendored ITF1788 corpus (`docs/references/vendor/itf1788-framework/itl/`),
which is in-tree primary data; where the corpus cannot distinguish required
from recommended operations, the boundary is pinned from the free proxy
literature and the uncertainty is named (see Consequences).

1. **Elementary battery, first growth round (designed).** `sin`/`cos`/`tan`,
   `sinh`/`cosh`/`tanh`, `pow`/`rootn`: workspace ADR-0005 fixed the seams;
   the implementation half is enc-34n, promoted to a v1.0 blocker.
2. **Elementary battery, second growth round (designed: workspace
   ADR-0007).** The corpus exercises operations outside ADR-0005's scope:
   inverse trig (`asin`, `acos`, `atan`, `atan2`), inverse hyperbolics
   (`asinh`, `acosh`, `atanh`), the exponential/logarithm variants (`exp2`,
   `exp10`, `log2`, `log10`, with `log` naming alignment to the standard),
   and `pown` (surfaced by this ledger's corpus enumeration but scoped by
   neither growth round until ADR-0007 placed it: an interval-layer case
   table over bare `RoundFloat`, no backend trait growth). Workspace
   ADR-0007 fixes the seams (three new capability traits, the margin
   doctrine split for the arc-hyperbolic musl exception, the atan2 rule);
   the interval arms are mechanical against it.
3. **Point-function battery over bare `RoundFloat`.** `abs`, `min`, `max`,
   `sign` (`pos` is identity), `ceil`, `floor`, `trunc`, `roundTiesToEven`,
   `roundTiesToAway`. Piecewise monotone, no transcendental backend needed;
   mechanical against the existing op patterns.
4. **Numeric and boolean completion (the Phase C deferrals).** `mid`, `rad`,
   `mid_rad`; `equal`, `less`, `precedes`, `strict_less`, `strict_precedes`;
   `overlap` (the sixteen-state relation). The orderings are full-1788
   required operations (clause 10.5.10, per D9.8 Annex A), absent from the
   simplified profile. The empty and unbounded conventions for the shared
   subset are pinned by the P1788.1/D9.8 tables (registry:
   p1788-1-d98-draft), notably `mid`/`rad` returning no value on empty or
   unbounded input, matching this crate's `Option` shape.
5. **Cancellative ops.** `cancel_minus`, `cancel_plus`. Required even in the
   simplified profile (D9.8 clause 4.5.3).
6. **Reverse operations.** Required in the full standard: all reverse-mode
   elementary functions (clause 10.5.4) and two-output division
   `mul_rev_to_pair` (clause 10.5.5), per D9.8 Annex A. The corpus
   additionally carries the binary (two-output-constraint) forms and the
   `pow_rev1`/`pow_rev2` pair. The transcendental reverses gate on the
   battery rounds above.
7. **Reduction operations.** `sum`, `dot`, `sum_square`, `sum_abs` (the
   standard's name is `sumSquare`; the corpus's ITL says `sum_sqr`). Status
   VERIFIED during this decision (2026-07-16): required in the set-based
   flavor of full 1788, clause 12.2.12, per Annex A of the P1788.1/D9.8
   draft, which lists them among the full standard's required operations
   dropped from the profile. Accuracy is correctly rounded per rounding-mode
   variant: the corpus's cancellation vector (`dot` of `{2^52+1, 2^104}`
   with `{2^52-1, -1}` demanding exactly `-1`) is answerable only by exact
   accumulation with one final rounding, so the exact-accumulation design
   problem is real and priced in. Two design questions remain for the
   implementation slice: the full rounding-mode list (the corpus tests only
   `_nearest`; the clause text governs) and the operation's home, since
   reductions act on point-number vectors, not intervals (a round-float
   vector extension that this crate re-exports is the natural shape; decide
   in that slice's ADR).
8. **Text I/O.** `nums_to_interval`, `text_to_interval` (bare and decorated,
   including the uncertain form and hex literals the constructor vectors
   exercise), and `interval_to_text`; plus the exact text representation
   `interval_to_exact`/`exact_to_interval`, required by full 1788 clause
   13.4 (per D9.8 Annex A) and carrying ZERO corpus coverage. Output
   correctness therefore rests entirely on round-trip properties designed
   here, not on vectors.
9. **Decoration overflow bug.** enc-1ta: the decorated basic ops demote on an
   unbounded result, per this crate's ADR-0004 pattern.
10. **Tight binary64 backend in round-float.** The conformance vectors demand
    tightest binary64 results; the f64 fixture is sound-not-tight by design
    and fails them structurally. A correctly rounded f64 `RoundFloat`
    instance is buildable on stable for the arithmetic core (TwoSum/FMA
    error-term techniques decide the rounding direction exactly); it gets its
    own ADR in round-float. Every consumer gains a tight binary64 backend;
    the conformance lane is merely its first user.
11. **The conformance vector lane, split by backend.** In-tree, stable
    toolchain: an ITL-reading test harness runs the arithmetic, numeric,
    boolean, set, cancel, class, overlap, and reverse-arithmetic vector files
    over the tight f64 backend. Out-of-tree, nightly (the elementary-oracle
    pattern, workspace-excluded crate, own toolchain pin and CI job): the
    elementary and transcendental-reverse files over pfloat-libm, which is
    where correctly rounded transcendentals live. The LGPL vector files
    (`fi_lib.itl`, `mpfi.itl`) are test data consumed at test time; they do
    not enter the shipped artifact's link graph (license posture per the
    registry entry).
12. **Arithmetic oracle lane.** enc-ebd, unchanged in scope: tightness and
    enclosure certification of the arithmetic ops against pfloat-libm.
13. **Conformance documentation.** The standard's conformance clause takes a
    documented claim. The claim attaches to instantiations, not to the
    generic crate: the crate is generic over `RoundFloat` where the standard
    speaks of concrete Level 2 types, so the document states the claim for
    the tight f64 instantiation and names the generic surface as the
    divergence, alongside the exceptions model (this crate reports
    construction failures as values, not signals).

Decorations stay as they are: the full five-element lattice already exceeds
what any road requires, and nothing in this decision touches it.

### What v1.0 does not wait for

- binary32/binary128 or cross-format Level 2 types (the corpus itself covers
  only binary64; additive later if ever).
- Kaucher/modal extensions (no flavor beyond set-based exists in the 2015
  standard).
- A correctness-proof upgrade of the enclosure theorem (the abstract-axiom
  Kani route, ADR-0003): tracked on its own, valuable, not a conformance
  input.

## Consequences

- enc-34n is promoted from P3 to P2: the battery is now a v1.0 blocker, not
  opportunistic growth. The remaining ledger items become tracked issues at
  P2 with their dependencies encoded; enc-pvf closes against this record.
- The three most plausible ways this decision goes wrong, inverted up front:
  1. **The 0.x tail outlives the author's attention.** This is road A's known
     cost and the reason roads B and C existed. Mitigation, not cure: the
     ledger is finite and enumerated from in-tree primary data rather than
     from recall, every item is bead-tracked, and the two heaviest unknowns
     (reductions, text I/O grammar) are fronted as design-first items so
     their cost surfaces early instead of at the end.
  2. **The required/recommended boundary is misdrawn.** The standard is
     paywalled and the boundary is pinned from proxies, per the decision made
     at the fork. The worst case named at decision time (the reductions) was
     resolved the same day: Annex A of the publicly served P1788.1/D9.8 draft
     enumerates the full standard's required operations dropped from the
     profile (reverse ops 10.5.4, `mulRevToPair` 10.5.5, the orderings
     10.5.10, reductions 12.2.12, exact text 13.4), and the ledger now cites
     it (registry: p1788-1-d98-draft). Residual exposure: the draft is one
     revision short of the published 1788.1, its Annex is informative, and
     clause-level details of full 1788 (the reductions' rounding-mode list,
     Level 2 conversion fine print) still rest on proxies. If a gap surfaces,
     the conformance claim slips in time; soundness is never at stake.
  3. **The tight f64 backend under-delivers.** If the TwoSum/FMA route cannot
     decide the rounding direction for some operation on stable (division and
     sqrt are the candidates to watch), the in-tree vector lane shrinks and
     more files move to the nightly lane. The split in ledger item 11 is a
     preference, not a load-bearing assumption; the fallback is stated so a
     partial result is a re-route, not a failure.
- Second-order effects on consumers: SMIL and the affine crate consume 0.x
  releases already and are unaffected in function; what they wait longer for
  is only the stability signal of 1.0. The workspace's other flag
  (affine-arith) is independent of this road.
- The conformance claim will not retire the README disclosures: the registry's
  coverage-gap analysis of the corpus (point samples, sampling bias, one
  Level 2 type, no output assertions) already concluded the vectors
  strengthen evidence without softening the named failure mode. This record
  inherits that conclusion.

## Alternatives considered

- **Road C (1788.1 anchor).** The recommended option at the fork: externally
  anchored narrow v1.0, vectors matching its territory, full 1788 additive.
  Declined by the author: for a crate carrying the 1788 name, the simplified
  profile is a waypoint, not a flag; the version number should wait for the
  surface the name promises. The 1788.1 mapping remains useful as an interim
  progress marker and the registry entry stays.
- **Road B (self-drawn narrow v1.0).** Declined: fails the completeness test
  outright; the narrow line is anchored to nothing a future user can hold.
- **All vectors in the nightly lane.** Declined at the fork: conformance
  evidence entirely off the stable toolchain, and no tight binary64 backend
  for consumers. The split keeps the arithmetic evidence on stable and makes
  the backend a durable artifact.

## References

- Registry: `docs/references/ieee-1788-2015.md` (the normative source, pointer
  only), `docs/references/ieee-1788-1-2017.md` (the declined anchor),
  `docs/references/p1788-1-d98-draft.md` (the publicly served draft whose
  Annex A settled the required-op boundary; Wayback saved at citation time),
  `docs/references/kulisch-2008-p1788-draft.md` (the complete-arithmetic
  lineage behind the reduction requirement),
  `docs/references/itf1788-framework.md` (the vendored corpus, its licenses,
  and its coverage gaps), with the free proxies
  `revol-1788-introduction.md`, `pryce-2016-forthcoming-1788.md`,
  `kearfott-2013-p1788-overview.md`.
- Workspace ADR-0005 (battery round one design); this crate's ADR-0003
  (fixture strategy) and ADR-0004 (elementary opener, decoration demotion
  pattern).
- Beads: enc-pvf (this decision), enc-34n (round one), enc-1ta (decoration
  bug), enc-ebd (arithmetic oracle lane); the ledger items filed at
  integration time link back to enc-pvf.
- The operation enumeration: extracted from the vendored ITL files on
  2026-07-16 (`libieeep1788_elem`, `_num`, `_bool`, `_set`, `_cancel`,
  `_class`, `_overlap`, `_reduction`, `_rev`, `_mul_rev`,
  `ieee1788-constructors`, `abs_rev`, `pow_rev`, `atan2`).
