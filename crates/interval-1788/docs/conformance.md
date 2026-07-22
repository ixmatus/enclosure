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
  type. As of 2026-07-22 this instantiation demonstrates bit-exact tightest
  results on all 2,368 corpus vectors the tight lanes transcribe (1,373
  arithmetic, 995 arithmetic-reverse; section 3 is the accounting), under two
  named accuracy bounds: integer powers and their reverse roots are correctly
  rounded for exponent magnitude at most `RoundPown::POWN_TIGHT_MAX` (64) and
  sound beyond, and the negative-exponent reverse root at the
  subnormal-overflow edge holds the reference's directed composition, the
  value the corpus itself pins (decision record 0006 errata). Section 4's one
  open item, the `mid` nearest-rounding datum, stands between this and the
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

## 3. What the conformance lane certified (2026-07-19, extended 2026-07-22)

The in-tree lane hand-translates the ITF1788 corpus and asserts it bit-exact
over `TightF64`. Two adversarial audits (coverage: corpus statements against
transcriptions, per testcase; fidelity: bit-level re-derivation, an oracle
check of the hex parser over all 198 literals in use, and verification that
every ignored lane asserts corpus data) confirmed the transcriptions
complete and faithful. The wave that closed the open items ran both audits
again against the integrated tree: a per-testcase coverage recount, and an
independent hex re-parse of all 341 division and 163 pown rows, the
signed-zero vectors, and every ignored twin. Certified today:

- Single-rounding arithmetic is bit-exact tightest: `pos`, `neg`, `add`,
  `sub`, `mul`, `div`, `recip`, `sqr`, `sqrt`, `fma`, and `pown`, every row
  of every testcase; 1,373 vectors.
- The arithmetic reverses are bit-exact tightest as decision record 0006
  part 2 promised: `sqrRev`, `absRev`, `mulRev` (unary, binary, ternary), the
  full `mulRevToPair` sign grid including every split and empty-slot row and,
  since 2026-07-22, its decorated grid propagating the division decoration on
  the first output piece (section 4 item 6), and the full `pownRev` root
  family through the `RoundPown` neighbor certification (section 4 item 3);
  995 vectors.
- `mid` at the largest-finite boundary is bit-exact through the
  `RoundLargestFinite` capability (decision record 0009); 7 vectors.
- `setDec` clamps an inconsistent (interval, decoration) pair to the strongest
  consistent decoration, matching the standard across all 22
  `minimal_set_dec_test` vectors. The 6 clamping vectors once parked as a known
  gap are certified as of 2026-07-22 (decision record 0007, bead enc-2hd),
  raising the certified count by 6; a strict `try_set_dec` variant stands beside
  the clamping `set_dec`. These vectors carry a decoration and no rounding, so
  the certification is independent of the backend and holds at the `TightF64`
  target.
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

Every item the lane surfaced is resolved as of 2026-07-22, each kept in place
(not renumbered) with its resolution dated. One new item, surfaced by this
wave's own verification rather than by the lane, is open (item 7):

1. **Division is bit-exact tightest** (resolved 2026-07-22, bead enc-ghz):
   direct directed division rounds each quotient endpoint once; the 56 relocated
   vectors are folded back and all 341 `div` rows pass in `minimal_div_test`.
2. **`pown` tightness (resolved 2026-07-22):** the repeated-squaring
   looseness and the negative-power overflow collapse are fixed by the exact
   `RoundPown` integer kernel (bead enc-5jj, round-float decision record 0004);
   all 163 pown vectors now pass bit-exact.
3. **`pownRev`'s root family (resolved 2026-07-22):** the root bisection left
   one-to-three-ulp brackets for exponent magnitude three and above (bead
   enc-cov) and the negative-exponent path saturated its set-level reciprocal
   at the subnormal edge (bead enc-ral). Both are fixed on the exact `RoundPown`
   integer kernel (round-float decision record 0004; decision record 0006
   erratum): the positive-exponent root ends in a neighbor certification and is
   correctly rounded, and the negative path roots first then reciprocates at the
   scalar level (correctly rounded where the constraint reciprocal is
   representable, matching the reference's directed composition at the subnormal
   overflow edge). All `pownRev` vectors now pass bit-exact: +52 to the
   certified reverse subtotal of section 3.
4. **Resolved 2026-07-22 (decision record 0007):** `set_dec` now clamps an
   inconsistent pair per the standard's `setDec`, and the strict construction
   is kept as `try_set_dec` (returns `None` on an inconsistent pair). The 6
   vectors join the live `set_dec_vectors` lane (bead enc-2hd).
5. **Resolved 2026-07-22 (bead enc-ks9).** The decorated numeric accessors,
   decorated unary predicates, decorated set operations, and bare and decorated
   `isCommonInterval`/`isMember` are now surfaced and transcribed 1:1 (section 3,
   the 2026-07-22 additions). The bare `inf`/`sup` gained the Level 2 signed-zero
   datum in the same slice, so bare and decorated agree bit-exactly.
6. **RESOLVED 2026-07-22: the decorated `mulRevToPair` first piece
   propagates the division decoration** (bead enc-pzd). The standard requires
   it: the two-output division sits in its own subclause, and its first output
   piece is decorated exactly as the normal division `c / b` whenever zero lies
   outside the divisor `b` (P1788/D8.4 clause 12.12.4, carried into IEEE
   1788-2015 clause 12.12.3 per the post-ballot record; registry entries
   `p1788-d8-4-draft` and `p1788-mailing-list-threads`). Only the empty second
   piece and the genuine zero-straddling split grade `trv`. The corpus was
   right; the crate had graded every piece `trv` per decision record 0006 part
   5, whose Errata now records the correction. All 175 decorated vectors pass
   bit-exact, values and decorations. The `trv` doctrine still stands for the
   one-output reverse operations, which the standard's non-arithmetic rule
   covers.
7. **OPEN: `mid` does not compute the Level 2 nearest datum** (bead enc-5p3).
   The standard's Level 2 `mid` is roundTiesToEven of the exact midpoint; the
   crate computes a directed halved sum, which drops subnormal halving
   remainders on any backend, landing an ulp low even where the exact midpoint
   is representable (`mid([2^-1074, 3 * 2^-1074])` computes `2^-1074`; the
   corpus pins `2 * 2^-1074`). The fix is a design decision (a nearest-halving
   backend capability, a documented divergence, or a directed derivation with
   an explicit tie break), not a test promotion; the falsified promotion
   premise is recorded in the bead. The 16 fixture-loose mid/rad/midRad
   vectors stay in ignored twins holding the corpus datum; the largest-finite
   boundary and the symmetric and singleton cases stay certified.

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
  the harnesses: the affine `from_raw_parts` validation splits into three
  proofs, of which two (acceptance, and the drop of zero coefficients) attest
  in CI while the third (the representation invariant over the returned terms)
  is verified locally and left off the CI harness list, its scan of the
  returned heap terms running 15 million SAT variables at the runner's memory
  cliff (bead enc-tgw); the affine addition-merge harness is cut with its gap
  recorded, and no Kani harness attests directed-rounding numerics
  (interval-1788 decision record 0003).
- **The limits:** the corpus is point samples with the registry-documented
  gaps (one Level 2 type, no output vectors, sampling bias); the README
  disclosure carries the named failure mode grounded in exactly those gaps,
  and this document does not retire it.
