---
slug: comba-stolfi-1993
category: algorithms
citation: >-
  Comba, J.L.D. and Stolfi, J., "Affine arithmetic and its applications to
  computer graphics", presented at SIBGRAPI'93, Recife, Brazil, 20-22 October
  1993. 10 pages.
canonical-url: https://www.ic.unicamp.br/~stolfi/EXPORT/papers/by-tag/com-sto-93-aa.ps.gz
doi: none
archived-url: https://web.archive.org/web/20260611084726/https://www.ic.unicamp.br/~stolfi/EXPORT/papers/by-tag/com-sto-93-aa.ps.gz
archive-date: 2026-06-11
retrieved: 2026-06-11
license: unstated (author copy on Stolfi's Unicamp page)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: primary
consumers:
  - docs/decisions/0001-enclosure-monorepo-and-round-float-layering.md
  - crates/affine-arith/README.md
verification:
  - none (the founding model; the derived kernels are verified against the later formulations)
sha256: none
notes: >-
  The paper that introduced affine arithmetic: noise symbols, shared symbol
  correlation, and the affine form model affine-arith implements. Citation
  transcribed from Stolfi's project page (tag com-sto-93-aa); a workshop
  paper with no DOI. PostScript only (78,568 bytes, sha256
  076ee23722e73d738735d102ad355ad4648ae35fd7054b1094abbd32ad5e5f8a at
  retrieval); fresh Wayback save 2026-06-11.
---

Everything in `affine-arith` descends from the model this paper introduced;
the mature statements the kernels were actually derived from are the 1997
monograph ([stolfi-figueiredo-1997](stolfi-figueiredo-1997.md)) and the 2004
survey ([figueiredo-stolfi-2004](figueiredo-stolfi-2004.md)), with this entry
holding the origin. The project page
([stolfi-affine-arith-project](stolfi-affine-arith-project.md)) lists it first
among the main publications.
