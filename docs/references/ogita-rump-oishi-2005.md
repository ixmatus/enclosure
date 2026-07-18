---
slug: ogita-rump-oishi-2005
category: algorithms
citation: >-
  Ogita, T., Rump, S.M., Oishi, S., "Accurate Sum and Dot Product", SIAM
  Journal on Scientific Computing 26(6), pp. 1955-1988, 2005.
canonical-url: https://www.tuhh.de/ti3/paper/rump/OgRuOi05.pdf
doi: 10.1137/030601818
archived-url: http://web.archive.org/web/20260717063409/https://www.tuhh.de/ti3/paper/rump/OgRuOi05.pdf
archive-date: 2026-07-17
retrieved: 2026-07-16
license: unstated (author copy on the TUHH page; SIAM holds the journal copyright)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: secondary
consumers:
  - crates/round-float/docs/decisions/0002-tight-f64-backend.md (TwoSum and TwoProduct as the error-free transforms behind the directed adjust)
  - crates/round-float/src/tight_f64.rs (Law 2 TwoSum and Law 3 TwoProduct, the derived error-free transforms whose error sign drives the directed adjust)
  - docs/decisions/0008-reduction-operations-kulisch-accumulator.md (the deferred distillation alternative)
  - crates/affine-arith/src/elementary.rs (the single-term exact domain gate; its doc records why the multi-term analogue, which needs the paper's summation, is blocked under the directed-only RoundFloat contract)
verification:
  - crates/round-float/tests/tight_oracle.rs (the derived transforms certified against an independent integer reference oracle across the subnormal, binade, and overflow zones)
  - crates/round-float/tests/tight_adjacency_fixture.rs (adjacency and exactness of the directed bounds the transforms produce)
  - crates/affine-arith/tests/elementary_domain_fixture.rs (a 2Sum derived from Algorithm 3.1 clamps sample points inside a single-term form's exact range, arbitrating the exact domain gate's acceptances)
sha256: none
notes: >-
  The standard modern citation for error-free transformations: TwoSum
  (Knuth's branch-free six-operation form) and TwoProduct via FMA, with the
  error analyses. Grounds two ledger items of the interval-1788 v1.0 road:
  the tight binary64 backend (the EFT error sign decides the directed
  adjust) and, later, the required clause 12.2.12 reductions (the paper's
  distillation summation and dot algorithms are the natural correctly
  rounded accumulator candidates alongside the Kulisch long accumulator).
  Same author page as [rump-2010-acta-numerica](rump-2010-acta-numerica.md).
---

Derivation source, not a template: the algorithms are taken as published
mathematics (with their error theorems), and the Rust shapes are chosen
fresh. For the underflow boundary conditions the EFT theorems carry, the
companion source is [boldo-daumas-2003](boldo-daumas-2003.md), which pins
exactly when the correcting terms stay representable; the Handbook
([muller-handbook-fp](muller-handbook-fp.md)) is the consolidated reference
for both.
