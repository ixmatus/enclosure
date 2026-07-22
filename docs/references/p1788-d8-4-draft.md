---
slug: p1788-d8-4-draft
category: spec-conformance
citation: >-
  IEEE P1788/D8.4, "Draft Standard for Interval Arithmetic", Pryce, J. D. and
  Keil, C. (eds.), Interval Standard Working Group, June 7 2014. Full standard
  working group draft, the last public draft before the approved IEEE Std
  1788-2015.
canonical-url: https://grouper.ieee.org/groups/1788/email/pdftGpc369cbD.pdf
doi: none
archived-url: http://web.archive.org/web/20181125131704/https://grouper.ieee.org/groups/1788/email/pdftGpc369cbD.pdf
archive-date: 2018-11-25
retrieved: 2026-07-22
license: IEEE working group document (unapproved draft copyright, 2014); pointer only, quotation with clause citation, do not vendor
vendor-status: legally-cannot
rot-risk: died-once
provenance-class: primary
consumers:
  - crates/interval-1788/src/reverse.rs (the decorated `mul_rev_to_pair` first piece grading)
  - crates/interval-1788/docs/decisions/0006-reverse-operations.md (the Errata section correcting part 5)
verification:
  - crates/interval-1788/tests/conformance_reverse_tight.rs (mul_rev_to_pair_dec_test, 175 vectors)
sha256:
  - 7af544ceb07bbebe30e021779fee397a495d2996ea7df62668d6bcf80532c68e  (the fetched PDF, integrity pin for the pointer; not vendored)
notes: >-
  The full standard draft the enclosure reverse decoration doctrine rests on.
  It carries the two output division in its own subclause, separate from the
  reverse mode elementary functions, and this separation is exactly what the
  ADR-0006 part 5 trv doctrine missed. The draft is pointer only under IEEE
  draft copyright; grouper.ieee.org has been decommissioning for years, so the
  entry leans on the pre existing 2018 Wayback snapshot rather than a fresh
  save.
---

What this draft grounds, with clause numbers as of D8.4:

- Clause 12.12.4, the two output division `mulRevToPair`. Its decorated
  version rule is the load bearing text: "There shall be a decorated version
  where each of x, y, u and v is of the corresponding decorated type. If either
  input is NaI then both outputs are NaI. Otherwise, if x and y are nonempty
  and 0 not in y, then u is the same as the result of normal division x/y and
  shall be decorated the same way; while v is empty and shall be decorated trv.
  In all other cases each output, empty or not, shall be decorated trv." The
  first output piece therefore carries the normal division's decoration whenever
  zero lies outside the divisor, not an unconditional trv.
- Clause 11.7, the non arithmetic operations rule that assigns the one output
  reverse mode elementary functions no better than trv. This rule funds the
  generic trv doctrine that still stands for `sqrRev`, `absRev`, `pownRev`, and
  `mulRev`; the two output division sits outside it, in its own subclause.

In the final numbering the operation is `mulRevToPair(b, c)` with the divisor
`b` first, computing `c/b`; the crate's method is `b.mul_rev_to_pair(c)` on
`self = b`. A genuine two component result arises only when zero is interior to
`b`, and then both pieces are trv; the propagated division decoration shows only
on a one component result. The subclause moved to 12.12.3 in the published
standard per the post ballot mailing list record (see
[p1788-mailing-list-threads](p1788-mailing-list-threads.md)).

Weight claims by what the source could not have seen: a June 2014 draft predates
the final ballot, so where the draft text and the published standard disagree on
numbering the standard governs and the disagreement is recorded (the subclause
number is the known instance, 12.12.4 in the draft, 12.12.3 in the final). The
decoration rule itself is stable across that move; the arXiv:2308.10693 verbatim
quotation of the final clause 12.12 corroborates the section's location.
