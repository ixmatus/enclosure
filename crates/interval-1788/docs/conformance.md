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
  2,409 corpus vectors across the arithmetic and arithmetic-reverse
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
  `sub`, `mul`, `div`, `recip`, `sqr`, `sqrt`, `fma`, and `pown`, every row
  of every testcase; 1,373 vectors.
- The arithmetic reverses are bit-exact tightest as decision record 0006
  part 2 promised, except the `pownRev` root family of section 4: `sqrRev`,
  `absRev`, `mulRev` (unary, binary, ternary), the full `mulRevToPair`
  sign grid including every split and empty-slot row and, since 2026-07-22,
  its decorated grid propagating the division decoration on the first output
  piece (section 4 item 6), and `pownRev` for exponents 0, ±1, ±2; 943
  vectors.
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

## 4. Open items between today and the claim

Each open item is a tracked bead with a red-when-run ignored test holding the
corpus datum, so the fix is verified by the lane the day it lands. Item 6 is
the first discharged; it is kept in place (not renumbered) with its resolution
dated:

1. **Division is bit-exact tightest** (resolved 2026-07-22, bead enc-ghz):
   direct directed division rounds each quotient endpoint once; the 56 relocated
   vectors are folded back and all 341 `div` rows pass in `minimal_div_test`.
2. **`pown` tightness (resolved 2026-07-22):** the repeated-squaring
   looseness and the negative-power overflow collapse are fixed by the exact
   `RoundPown` integer kernel (bead enc-5jj, round-float decision record 0004);
   all 163 pown vectors now pass bit-exact.
3. **`pownRev`'s root bisection** leaves one-to-three-ulp brackets for
   exponent magnitude three and above (44 vectors, bead enc-cov; decision
   record 0006 part 3 carries an erratum obligation for its exactness
   sentence), and its negative-exponent path saturates at the subnormal
   edge (8 vectors, bead enc-ral).
4. **Resolved 2026-07-22 (decision record 0007):** `set_dec` now clamps an
   inconsistent pair per the standard's `setDec`, and the strict construction
   is kept as `try_set_dec` (returns `None` on an inconsistent pair). The 6
   vectors join the live `set_dec_vectors` lane (bead enc-2hd).
5. **The decorated surface lacks** numeric accessors, unary predicates, set
   operations, and `isCommonInterval`/`isMember` (bead enc-ks9); the bare
   forms are covered, two via documented compositions.
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
