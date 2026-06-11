---
slug: stolfi-figueiredo-1997
category: algorithms
citation: >-
  Stolfi, J. and de Figueiredo, L.H., "Self-Validated Numerical Methods and
  Applications", course monograph for the 21st Brazilian Mathematics
  Colloquium, IMPA, 1997.
canonical-url: https://lhf.impa.br/ftp/papers/cbm97.ps.gz
doi: none
archived-url: http://web.archive.org/web/20260611080520/https://lhf.impa.br/ftp/papers/cbm97.ps.gz
archive-date: 2026-06-11
retrieved: 2026-06-11
license: unstated (author copies on IMPA and Unicamp personal pages)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: primary
consumers:
  - crates/affine-arith/src/elementary.rs
  - docs/decisions/0002-affine-elementary-functions.md
  - crates/affine-arith/README.md
verification:
  - crates/affine-arith/tests/elementary_fixture.rs
sha256: none
notes: >-
  The book length treatment behind the 2004 survey: interval arithmetic,
  affine arithmetic, and the Chebyshev approximation theory the elementary
  functions cite. This monograph IS the "Stolfi course notes" of community
  folklore; it was written as the course book for the 1997 Brazilian
  Mathematics Colloquium. No separate course notes artifact exists on Stolfi's
  pages, a provenance correction this entry records.
---

Where the 2004 survey ([figueiredo-stolfi-2004](figueiredo-stolfi-2004.md))
states the Chebyshev construction, the monograph develops the approximation
theory underneath it; ADR-0002 cites both for the slope and residual bound
derivations. Two author copies exist (116 pages per the project page) and both got fresh
Wayback saves on 2026-06-11: de Figueiredo's (above, sha256
d085f21c2448fc3b4c15cc80cf7ea869d7b1097c61e56d0abad5ac52ad246551) and Stolfi's
mirror at `www.ic.unicamp.br/~stolfi/EXPORT/papers/by-tag/fig-sto-97-iaaa.ps.gz`
(web.archive.org/web/20260611080455, sha256
145602bb5a96ab737d93bf518a5bb3c983324086f5e2009d8ac0e3cc0323c7ac). The two
gzips differ by 16 bytes from separate gzip runs; the extracted PostScript is
the same document. PostScript only; neither page offers a PDF (the 404s are
recorded in the mining transcript). The embedded copyright strings are TeX
font notices, not a document license, so the license stands unstated and the
copies stay pointer only.
