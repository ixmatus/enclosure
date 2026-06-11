---
slug: muller-handbook-fp
category: spec-conformance
citation: >-
  Muller, J.-M., Brunie, N., de Dinechin, F., Jeannerod, C.-P., Joldes, M.,
  Lefevre, V., Melquiond, G., Revol, N., and Torres, S., "Handbook of
  Floating-Point Arithmetic", 2nd edition, Birkhauser, 2018. ISBN
  978-3-319-76525-9 (print), 978-3-319-76526-6 (ebook).
canonical-url: https://link.springer.com/book/10.1007/978-3-319-76526-6
doi: 10.1007/978-3-319-76526-6
archived-url: http://web.archive.org/web/20251222022516/https://link.springer.com/book/10.1007/978-3-319-76526-6
archive-date: 2025-12-22
retrieved: 2026-06-11
license: copyrighted commercial book (paywalled)
vendor-status: legally-cannot
rot-risk: stable-publisher
provenance-class: secondary
consumers:
  - docs/references/ieee-754-2019.md (named free-proxy-adjacent reference; the book itself is not free)
verification:
  - none
sha256: none
notes: >-
  The modern comprehensive treatment of floating point arithmetic, including
  the directed rounding and correctly rounded function material that grounds
  this workspace's backend contract reasoning. Two registry connections in the
  author list: Lefevre (the worst case rounding tables pfloat-libm is verified
  against) and Revol (MPFI, the 1788 introduction).
---

Where [goldberg-1991](goldberg-1991.md) is the free conceptual proxy, the
handbook is the working professional reference: rounding error analysis,
elementary function implementation, the table maker's dilemma. It is a
commercial book and stays a pointer; a paper copy is the permacomputing
vendoring move if the estate ever wants one on the shelf. Fresh Wayback saves
were rate limited on 2026-06-11; the recorded snapshot is pre existing.
