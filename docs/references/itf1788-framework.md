---
slug: itf1788-framework
category: spec-conformance
citation: >-
  Kiesner, M., Nehmeier, M., Wolff von Gudenberg, J., "ITF1788: An Interval
  Testframework for IEEE 1788", Technical Report 495, Department of Computer
  Science, University of Wurzburg, June 2015. Vector corpus from the
  github.com/oheim/ITF1788 lineage head (19 itl files, dormant since
  2018-09-27; origin github.com/nehmeier/ITF1788, dormant since 2018-09-22).
canonical-url: https://github.com/oheim/ITF1788
doi: none
archived-url: http://web.archive.org/web/20231224040247/https://github.com/oheim/ITF1788
archive-date: 2023-12-24
retrieved: 2026-06-11
license: >-
  framework Apache-2.0 (LICENSE + NOTICE, copyright 2014 Nehmeier and Kiesner);
  itl vectors per file: libieeep1788_*.itl Apache-2.0; fi_lib.itl and mpfi.itl
  LGPL-2.1-or-later (COPYING.LESSER vendored alongside); ieee1788-constructors
  and ieee1788-exceptions all-permissive (Heimlich 2016)
vendor-status: vendored-at-path
rot-risk: died-once
provenance-class: primary
consumers:
  - crates/interval-1788/src/lib.rs (roadmap conformance lane)
  - crates/interval-1788/docs/decisions/0003-kani-over-f64-fixture.md
  - crates/interval-1788/README.md (verification disclosure)
  - crates/interval-1788/tests/point_functions_fixture.rs (abs, min, max, sign,
    ceil, floor, trunc, roundTiesToEven, roundTiesToAway vectors, bare and
    decorated, from libieeep1788_elem.itl)
  - crates/interval-1788/tests/cancel_fixture.rs (cancelMinus and cancelPlus
    vectors, bare and decorated, from libieeep1788_cancel.itl)
verification:
  - none yet (the conformance lane that will consume these vectors is roadmap work)
sha256:
  - dc626520dcd53a22f727af3ee42c770e56c97a64fe3adb063799d8ab032fe551  vendor/itf1788-framework/COPYING.LESSER
  - cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30  vendor/itf1788-framework/LICENSE
  - 162c150ef5ffc1444a5540ef730d657d639452ce7a1f0ed235445087a1277196  vendor/itf1788-framework/NOTICE
  - 23d4c8abcf1a0e2f23860cf51b71efd3d7c3c854201f519d167d79497a5daeb6  vendor/itf1788-framework/itl/abs_rev.itl
  - e5ba6952210a313abf4b2c60831a412ffc337e990451e751919cd9b776ff0c7f  vendor/itf1788-framework/itl/atan2.itl
  - cbba9d32a22a3c01cc9247c59ccf3497a1d86b5eedd20fd5c39574b05004ad90  vendor/itf1788-framework/itl/c-xsc.itl
  - 71e67cd3e040cceec56617d731e2f645f23a0a113d4d88cbba77ec6b47ca52c6  vendor/itf1788-framework/itl/fi_lib.itl
  - 743bb6c0a5bf5473f54d06570aa66f71399e37501dbc2d7e670caeadbf52b217  vendor/itf1788-framework/itl/ieee1788-constructors.itl
  - 576863aaf4312b653eaa0730d4aa7fff76bd445bd9ec155a4ffe853c4e6a0051  vendor/itf1788-framework/itl/ieee1788-exceptions.itl
  - 8d18f4a8ba7dfcd588544aedc6d2cbfa5b25fe27e4b16f31ef87b20482c4289e  vendor/itf1788-framework/itl/libieeep1788_bool.itl
  - 13b4424f6100c0bd6956cfbe538bb1c79ec5bca762b0a7c889e69e13255602f0  vendor/itf1788-framework/itl/libieeep1788_cancel.itl
  - 9baa1dda2b7601da3738e5a7c87ccf98d537fecb96490f719b57943d8c524774  vendor/itf1788-framework/itl/libieeep1788_class.itl
  - c14a195b0a0a968d0f8e7e05a10430ec6cc9872be6f9d20e61b66f07ea92e62a  vendor/itf1788-framework/itl/libieeep1788_elem.itl
  - 44c7f2abf354796c1938cc09b043ec8b5a4b4259441318bb75c5d317397884b6  vendor/itf1788-framework/itl/libieeep1788_mul_rev.itl
  - 67832843fdc2bb394d2c54c1290e5334ad5a81bf10b838432b40dae80e96c170  vendor/itf1788-framework/itl/libieeep1788_num.itl
  - 353bb0a90c9c20ab3f577c8374b626a0712b4f9afa592f05c5c7cb4192327504  vendor/itf1788-framework/itl/libieeep1788_overlap.itl
  - 45fdd80d7741cb82e469b4595811f12fbfdea096b92dba95e4db9478d1d45f5b  vendor/itf1788-framework/itl/libieeep1788_rec_bool.itl
  - 0d08bcb584f0ca30310a86f8b0484e8a939d774d0e282f7ccd5f9cc8bb4dd86e  vendor/itf1788-framework/itl/libieeep1788_reduction.itl
  - 28232a6d939de4726259bde80f12ea65d2e8df93d3e0679988113af92a7cbd04  vendor/itf1788-framework/itl/libieeep1788_rev.itl
  - 6d1fc9163d0db66a7bc52755a3ba6392a6b8d4e11d34d92ed34080e2a268dbbd  vendor/itf1788-framework/itl/libieeep1788_set.itl
  - d2bfe0c71f6890ef9ab50e944bb804f7c033af46b9b45ca3ff37c2a81af2f57c  vendor/itf1788-framework/itl/mpfi.itl
  - 7256a5ac3d93855f790289ff0135d58caf6128f679ab3264936c04ad7aed0e25  vendor/itf1788-framework/itl/pow_rev.itl
notes: >-
  The interval community's shared conformance suite. Vendored in full because
  the rot is no longer hypothetical: the framework's own technical report
  (tr495.pdf at uni-wuerzburg.de) is dead at source with no Wayback snapshot,
  and every repository in the lineage has been dormant since 2018.
---

ITF1788 generates per library unit tests from a shared interval test language
(ITL). The vendored corpus is the 19 file itl directory of the lineage head
`oheim/ITF1788` (the fork chain runs Nehmeier to Kiesner to Heimlich; Heimlich's
fork adds the MPFI, FI_LIB, C-XSC, constructor, and exception vector sets and is
what the Octave interval package's ~9,500 generated test cases derive from).

## What the vectors exercise

Roughly 9,583 assertion lines over the full set based flavor required operation
set: arithmetic (including `fma`, `pown`, `pow`, the full elementary battery),
cancel ops, set ops, numeric functions, boolean comparisons, overlap, reverse
operations, reductions, text to interval constructors (including uncertain form
and hex literals), and decoration plumbing. Decorations are genuinely exercised:
1,706 assertion lines use decorated literals or NaI, across seven files.

## Coverage gaps (feed the README disclosures)

1. One Level 2 type only: binary64 inf sup intervals. No binary32/binary128, no
   cross format conversions, no implicit types, no compressed arithmetic.
2. No interval output: `intervalToText` and friends have zero assertions;
   parsing is tested, formatting is not.
3. Point sample tightness, not correct rounding: assertions demand the tightest
   binary64 result at fixed sample points; the two tier `tight <= accurate`
   grading the DSL supports is used on exactly 2 lines (both c-xsc.itl). The
   vectors spot check elementary functions at dozens of points; they cannot
   establish correct rounding across a domain, nor grade valid but not tightest
   implementations.
4. Thin exception coverage: ~71 `signal` assertions, concentrated in
   constructor and class files.
5. Reductions minimal: 16 assertions, nearest rounding only.
6. Set based flavor only (the standard defines no other; Kaucher/modal absent).
7. Inherited sampling bias: the corpus is the union of four libraries' own unit
   tests; density mirrors what those libraries chose to test.

Independent corroboration: Revol, Benet, Ferranti, Zhilin, "Testing interval
arithmetic libraries, including their IEEE-1788 compliance", arXiv:2205.11837
(also PPAM 2022, DOI 10.1007/978-3-031-30445-3_36), which notes "not every
important aspect of our libraries fit in these frameworks" (snapshot
web.archive.org/web/20260214173238/https://arxiv.org/abs/2205.11837).

## Reconciliation with the README disclosures (2026-06-11)

The three crate READMEs each name, under "What this does not promise", a
failure mode of the shape "a bound or enclosure wrong on an input no proof or
test reached". The coverage analysis above confirms the vector suite cannot
retire those disclosures: the vectors are point samples with inherited
sampling bias, they grade tightest results at fixed inputs rather than
establishing behavior across a domain, and they exercise one Level 2 type.
Wiring the suite in (the roadmap conformance lane) will strengthen the
evidence and should not soften the disclosure text. No contradiction between
the disclosures and the vector reality was found.

One precision flag, for a future README revision rather than an edit now: the
interval-1788 README's verification paragraph plans to run "the IEEE 1788
conformance test vectors and differential tests against a trusted reference"
out of process to keep copyleft out of the link graph. The copyleft concern
attaches to the GPL oracle implementations
([octave-interval-heimlich](octave-interval-heimlich.md)), not to the vector
data, whose per file licenses (Apache-2.0, LGPL-2.1-or-later, all-permissive)
permit the in repo carriage this registry now does. The sentence bundles the
two; they have different license postures.

## Provenance and rot notes

The technical report URL the README cites
(`se2.informatik.uni-wuerzburg.de/publications/tr495.pdf`) returns connection
failure and has no Wayback snapshot under any checked host path; a ResearchGate
record (publication 278620157) is the only located trace. Fresh Wayback saves of
the repositories were rate limited on 2026-06-11 (520/429 across the session);
recorded snapshots are pre existing. The vendored copy with per file hashes is
this registry's hedge against the next link death.
