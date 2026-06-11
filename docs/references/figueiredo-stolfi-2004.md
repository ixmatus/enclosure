---
slug: figueiredo-stolfi-2004
category: algorithms
citation: >-
  de Figueiredo, L.H. and Stolfi, J., "Affine Arithmetic: Concepts and
  Applications", Numerical Algorithms 37(1-4), pp. 147-158, December 2004.
canonical-url: https://lhf.impa.br/ftp/papers/aa.ps.gz
doi: 10.1023/B:NUMA.0000049462.70970.b6
archived-url: https://web.archive.org/web/20260611081202/https://lhf.impa.br/ftp/papers/aa.ps.gz
archive-date: 2026-06-11
retrieved: 2026-06-11
license: unstated (author copy on de Figueiredo's IMPA page; Springer holds the journal copyright)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: primary
consumers:
  - crates/affine-arith/src/elementary.rs
  - crates/affine-arith/src/condense.rs
  - docs/decisions/0001-enclosure-monorepo-and-round-float-layering.md
  - docs/decisions/0002-affine-elementary-functions.md
  - docs/decisions/0004-noise-term-condensation.md
  - crates/round-float/docs/decisions/0001-round-transcendental-extension-trait.md
  - crates/affine-arith/README.md
verification:
  - crates/affine-arith/tests/elementary_fixture.rs
  - crates/affine-arith/tests/condense_fixture.rs
sha256: none
notes: >-
  The canonical affine arithmetic survey and the single most cited source in
  this workspace: the Chebyshev min max linearization and the noise symbol
  condensation argument both derive from it. Citation metadata verified
  against Crossref; the Springer page is login walled, the IMPA author copy is
  the accessible canonical.
---

This survey is the working reference for `affine-arith`: the univariate
Chebyshev approximation that every nonlinear elementary derives from
(ADR-0002), and the condensation construction whose soundness argument
`condense.rs` re derives for the directed rounding setting (ADR-0004). The in
tree derivations follow the paper's mathematics, not any implementation; the
module prose in `elementary.rs` and `condense.rs` records the re derivations.

Accessible copies: the author postscript above (gzipped PostScript is the only
format; no PDF of the paper exists on either author page, and the fresh
2026-06-11 save was this URL's first snapshot ever). Content hash of aa.ps.gz
at retrieval:
sha256 ffa46b8a1f59c0dc3aa5cfa7ec2302b46678c6ee617a99b310dde530b75d0717.
Invited talk slides (slides, not the paper) at `lhf.impa.br/ftp/oral/aa.pdf`
(snapshot 2025-03-23, web.archive.org/web/20250323142834). De Figueiredo's
publications index at `lhf.impa.br/publications.html` carries the
authoritative citation text; it had no Wayback snapshot at all until the
fresh 2026-06-11 save (web.archive.org/web/20260611083947), its first ever. The deeper companion is the 1997 monograph
([stolfi-figueiredo-1997](stolfi-figueiredo-1997.md)); the project hub is
[stolfi-affine-arith-project](stolfi-affine-arith-project.md).
