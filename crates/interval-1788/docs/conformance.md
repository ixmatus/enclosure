# Conformance

**Status:** living document, pre-1.0. This document becomes the conformance
claim IEEE 1788-2015 clause 12 asks for when v1.0 ships (crate decision
record 0005, road A, ledger item 13). Until then it states, dated, exactly
what is demonstrated today, what diverges, and what remains open. Nothing
here softens the README's disclosure section; the registry's coverage
analysis of the test corpus concluded the vectors strengthen evidence
without retiring those disclosures, and this document inherits that
conclusion.

**Date of the statements below:** 2026-07-22.

## 1. What is claimed, and to what

The standard's conformance clause attaches to concrete Level 2 types. This
crate is generic over the `RoundFloat` backend family, so the claim attaches
to instantiations, never to the generic crate:

- **`Interval<TightF64>` and `DecoratedInterval<TightF64>`** (the correctly
  rounded binary64 backend from round-float's `f64-tight` feature) are the
  conformance target: the set-based flavor over the binary64 inf-sup Level 2
  type. Today this instantiation demonstrates bit-exact tightest results on
  2,368 corpus vectors across the arithmetic and arithmetic-reverse
  operations, with the open items of section 4 standing between it and the
  full claim.
- **`Interval<f64>` over the fixture** (round-float's `f64` feature) is
  deliberately sound-not-tight: every result encloses, transcendentals carry
  documented margins. It supports development and the enclosure lanes; no
  tightest-accuracy claim attaches to it, ever.
- **The generic surface itself is a named divergence:** the standard speaks
  of concrete types; this crate's operations are generic items over
  `RoundFloat` and its extension traits. The divergence is deliberate
  (decision records 0001 and 0002 of round-float and this crate) and costs
  nothing at any concrete instantiation.

## 2. Operation surface and naming

Every required operation of the set-based flavor is implemented, with these
mappings recorded once here and in the rustdoc:

- Names are snake_case forms of the standard's (`roundTiesToEven` is
  `round_ties_to_even`); the standard's `log` is `ln`, matching `f64::ln`,
  with `log2` and `log10` under their own names.
- Reverse operations ship one method per operation with the candidate always
  explicit; the standard's candidate-less form is the same call at
  `x = entire` (crate decision record 0006 part 1). `mulRevToPair` returns
  the standard's ordered pair.
- Construction failures and text-parsing signals are reported as values
  (`Option`, `Result` with `TextError`, decorated `NaI`), never as
  exceptions or signals; the standard's `UndefinedOperation` and
  `PossiblyUndefinedOperation` signals map to `Err` values and documented
  `Ok` cases respectively in the text lanes.
- The exact text representation (`interval_to_exact` / `exact_to_interval`,
  full-1788 clause 13.4) is implemented; the corpus carries zero output
  vectors, so its correctness rests on the designed round-trip property
  lanes, and this document says so rather than implying vector coverage.

## 3. What the conformance lane certified (2026-07-19)

The in-tree lane hand-translates the ITF1788 corpus and asserts it bit-exact
over `TightF64`. Two adversarial audits (coverage: corpus statements against
transcriptions, per testcase; fidelity: bit-level re-derivation, an oracle
check of the hex parser over all 198 literals in use, and verification that
every ignored lane asserts corpus data) confirmed the transcriptions
complete and faithful. Certified today:

- Single-rounding arithmetic is bit-exact tightest: `pos`, `neg`, `add`,
  `sub`, `mul`, `recip`, `sqr`, `sqrt`, `fma`, and the in-budget `div` and
  `pown` rows; 1,276 vectors.
- The arithmetic reverses are bit-exact tightest as decision record 0006
  part 2 promised, except the `pownRev` root family of section 4: `sqrRev`,
  `absRev`, `mulRev` (unary, binary, ternary), the full `mulRevToPair`
  sign grid including every split and empty-slot row, and `pownRev` for
  exponents 0, ±1, ±2; 768 vectors.
- `mid` at the largest-finite boundary is bit-exact through the
  `RoundLargestFinite` capability (decision record 0009); 7 vectors.
- Zero soundness violations exist anywhere in the corpus, on either
  backend: every non-tight result the lane surfaced is a verified sound
  enclosure.

Added 2026-07-22 (bead enc-ks9, resolving section 4 item 5): the decorated
and recommended non-arithmetic surface, transcribed over the `f64` fixture.
The endpoint-logic families carry no rounding, so their results are bit-exact
on any backend; `mid`/`rad`/`midRad` decorated split into a live fn and an
ignored twin holding the tight datum, exactly as their bare siblings do.

- Decorated numeric accessors `inf`/`sup`/`wid`/`mag`/`mig`/`mid`/`rad`/
  `midRad`: 15 + 15 + 9 + 9 + 12 + 13 + 10 + 13 = 96 vectors (of the 36 for
  `mid`/`rad`/`midRad`, 20 are live and 16 sit in ignored loose twins). The
  accessors return the Level 2 float datum, NaN included, where the bare forms
  return `Option`; `inf`/`sup` gained the Level 2 signed-zero datum (a zero
  infimum reads `-0`, a zero supremum `+0`), and the affected bare vectors now
  assert the sign bit-exactly.
- Decorated unary predicates `isEmpty`/`isEntire`/`isSingleton`:
  15 + 17 + 16 = 48 vectors.
- `isCommonInterval` and `isMember`, bare against the real API and decorated:
  bare 12 + 35 = 47, decorated 21 + 40 = 61 vectors. The decorated `isMember`
  file holds 40 statements covering 38 distinct vectors (the corpus repeats two
  `[empty]_trv = false` lines verbatim; both are transcribed and marked).
- Decorated set operations `intersection`/`convexHull`: 5 + 5 = 10 vectors,
  each pinning the interval part and the propagated `trv`.

Total: 96 + 48 + 61 + 47 + 10 = 262 statements.

## 4. Open items between today and the claim

Each is a tracked bead with a red-when-run ignored test holding the corpus
datum, so the fix is verified by the lane the day it lands:

1. **Division is one rounding looser than tightest** on 56 vectors
   (`a * recip(b)`; bead enc-ghz). Fix: direct directed division.
2. **`pown` is looser than tightest** for exponent magnitude three and
   above, and its negative-power overflow path collapses near the subnormal
   edge; 41 vectors (bead enc-5jj).
3. **`pownRev`'s root bisection** leaves one-to-three-ulp brackets for
   exponent magnitude three and above (44 vectors, bead enc-cov; decision
   record 0006 part 3 carries an erratum obligation for its exactness
   sentence), and its negative-exponent path saturates at the subnormal
   edge (8 vectors, bead enc-ral).
4. **`set_dec` returns `NaI` where the standard's `setDec` clamps** (6
   vectors, bead enc-2hd); conform-or-declare is an open decision.
5. **Resolved 2026-07-22 (bead enc-ks9).** The decorated numeric accessors,
   decorated unary predicates, decorated set operations, and bare and decorated
   `isCommonInterval`/`isMember` are now surfaced and transcribed 1:1 (section 3,
   the 2026-07-22 additions). The bare `inf`/`sup` gained the Level 2 signed-zero
   datum in the same slice, so bare and decorated agree bit-exactly.
6. **The decorated `mulRevToPair` doctrine is unresolved** (bead enc-pzd):
   the draft-era corpus propagates decorations through the two-output
   division where this crate grades `trv` per decision record 0006 part 5.
   All 175 interval values agree bit-exactly; the divergence is
   decoration-only, and whether the published standard sides with the
   draft-era reference or with the trv reading is open research against the
   free proxies. Until settled it is a named divergence, not a defect.

## 5. Evidence basis and its limits

- **The corpus:** the ITF1788 vector suite, vendored with per-file licenses.
  The lane transcribes the Apache-2.0 `libieeep1788` files and the
  all-permissive constructor and exception sets. The LGPL files (`mpfi.itl`,
  `fi_lib.itl`, `c-xsc.itl`) are deliberately not transcribed: adapting
  copyleft vector files into this permissive tree is declined, and their
  coverage duplicates the required-operation surface the Apache files pin.
  The corpus derives from the draft-era libieeep1788 reference (registry:
  named caveats), which is how a draft-vs-final question like item 6 above
  can arise.
- **The lanes:** bit-exact conformance over `TightF64`
  (`conformance_arith_tight`, `conformance_reverse_tight`,
  `reduction_conformance` in round-float); enclosure transcription over the
  fixture (the `*_fixture` lanes); the nightly oracle lane certifying the
  fixture's transcendental margins against a correctly rounded reference;
  property lanes for round-trips, reverse-image soundness, and enclosure;
  Kani harnesses for the discrete skeletons. Kani caveats are recorded at
  the harnesses: the affine `from_raw_parts` harness is verified locally
  but temporarily absent from the CI harness list (runner memory; bead
  enc-tgw), the affine addition-merge harness is cut with its gap recorded,
  and no Kani harness attests directed-rounding numerics (interval-1788
  decision record 0003).
- **The limits:** the corpus is point samples with the registry-documented
  gaps (one Level 2 type, no output vectors, sampling bias); the README
  disclosure carries the named failure mode grounded in exactly those gaps,
  and this document does not retire it.
