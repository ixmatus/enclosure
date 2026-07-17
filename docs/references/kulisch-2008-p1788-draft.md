---
slug: kulisch-2008-p1788-draft
category: history-philosophy
citation: >-
  "P1788: Draft International Standard for Interval Arithmetic and Complete
  Arithmetic", Draft 1.0.1, September 29, 2008. Early working group proposal
  draft of the Kulisch complete-arithmetic lineage.
canonical-url: https://grouper.ieee.org/groups/1788//email/pdfmFrBWgxZ0a.pdf
doi: none
archived-url: http://web.archive.org/web/20260717061601/https://grouper.ieee.org/groups/1788//email/pdfmFrBWgxZ0a.pdf
archive-date: 2026-07-17
retrieved: 2026-07-16
license: IEEE draft copyright (2008), unapproved standards draft; quotation only
vendor-status: legally-cannot
rot-risk: died-once
provenance-class: primary
consumers:
  - crates/interval-1788/docs/decisions/0005-v1-road-full-conformance.md (the complete-arithmetic lineage behind the reduction requirement)
  - docs/decisions/0008-reduction-operations-kulisch-accumulator.md (the lineage behind the accumulator choice)
verification:
  - none (historical document)
sha256:
  - 84402b76a88c5398205781340e3bc2b0f34858e0c4716faa293d0b404600e17f  (the fetched PDF, integrity pin for the pointer; not vendored)
notes: >-
  Historical witness only: the 2008 proposal that carried Kulisch's complete
  arithmetic (the exact dot product) into the P1788 process, whose surviving
  trace in the published standard is the correctly rounded reduction
  operations of clause 12.2.12. Its clause structure bears no relation to
  IEEE Std 1788-2015; do not cite it for the final standard's content. The
  working group accepted the exact-dot-product proposal in November 2009
  (see the SCITEPRESS P1788 committee paper). Fetched live from
  grouper.ieee.org 2026-07-16, Wayback saved at citation time.
---

Kept because the interval-1788 v1.0 road ADR prices an exact-accumulation
design problem for the required reductions, and the "why does a set-based
interval standard require a correctly rounded dot product on point vectors"
question is answered by this lineage, not by anything in the final text. With
[kulisch-computer-arithmetic](kulisch-computer-arithmetic.md) for the theory
and [p1788-1-d98-draft](p1788-1-d98-draft.md) Annex A for the requirement's
final shape.
