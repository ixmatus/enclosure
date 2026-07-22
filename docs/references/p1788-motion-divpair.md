---
slug: p1788-motion-divpair
category: history-philosophy
citation: >-
  Wolff von Gudenberg, J., "Motion divPair", P1788 working group motion,
  April 23 2013.
canonical-url: https://grouper.ieee.org/groups/1788/email/pdfxEpx6JsTEs.pdf
doi: none
archived-url: http://web.archive.org/web/20181125132511/https://grouper.ieee.org/groups/1788/email/pdfxEpx6JsTEs.pdf
archive-date: 2018-11-25
retrieved: 2026-07-22
license: IEEE working group document (unapproved draft copyright, 2013); pointer only, quotation with citation, do not vendor
vendor-status: legally-cannot
rot-risk: died-once
provenance-class: primary
consumers:
  - crates/interval-1788/docs/decisions/0006-reverse-operations.md (the Errata section's lineage for the split decoration doctrine)
verification:
  - none (historical motion, design lineage only)
sha256:
  - ccfef3cb6efd86a8a4bfbc9e9e1116064201ec221709ea6e0ba8c72029a00020  (the fetched PDF, integrity pin for the pointer; not vendored)
notes: >-
  The working group motion that carried the two output division into the
  standard. Kept as the design lineage behind the split decoration doctrine:
  the two output form and its first piece division decoration originate in this
  proposal, which the D8.4 draft then wrote into clause 12.12.4. Pointer only
  under IEEE draft copyright; leans on the pre existing 2018 Wayback snapshot.
---

Why this source: the enclosure reverse battery implements `mulRevToPair` as the
standard's two output division, and the ADR-0006 erratum needed the lineage that
explains why the standard grades this operation's first piece by the division
rule rather than the generic reverse mode trv rule. This motion is that lineage.
It predates the divisor first `mulRevToPair(b, c)` naming the final adopted;
read it for the design intent, not the final surface, and cross reference the
D8.4 clause text ([p1788-d8-4-draft](p1788-d8-4-draft.md)) for the normative
decoration rule and the mailing list threads
([p1788-mailing-list-threads](p1788-mailing-list-threads.md)) for the final
clause numbering.
