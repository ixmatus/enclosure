---
slug: makino-berz-2003-taylor-models
category: algorithms
citation: >-
  Makino, K. and Berz, M., "Taylor models and other validated functional
  inclusion methods", International Journal of Pure and Applied Mathematics
  4(4), pp. 379-456, 2003.
canonical-url: https://www.bmtdynamics.org/pub/papers/TMIJPAM03/TMIJPAM03.pdf
doi: none (IJPAM of that era issued no DOIs)
archived-url: https://web.archive.org/web/20260611082546/https://www.bmtdynamics.org/pub/papers/TMIJPAM03/TMIJPAM03.pdf
archive-date: 2026-06-11
retrieved: 2026-06-11
license: unstated (Berz group mirror)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: secondary
consumers:
  - docs/decisions/0004-noise-term-condensation.md (the road not taken)
verification:
  - none
sha256: none
notes: >-
  Taylor models: polynomial enclosures with interval remainder, the
  literature's principled fix for nonlinearity beyond affine forms. Recorded
  as the explicit road not taken: memory hungry, wrong fit for a no_std
  bounded form, and the reason the workspace stops at affine plus
  condensation. The MSU host (bt.pa.msu.edu) refuses automated fetches (403);
  the bmtdynamics.org mirror is live, and the fresh save above is that URL's
  first snapshot ever.
---

The improvement ladder over affine arithmetic ends here: when first order
linearization plus error symbols is not enough, Taylor models carry the
nonlinearity in the polynomial part. Every cost the workspace avoids by
declining them (per value memory growth, multiplication cost, remainder
bookkeeping) is documented in this survey, which makes it the citation for
why `affine-arith` deliberately does not go further. The rot finding is
registry relevant on its own: the canonical Berz group host now 403s
unauthenticated readers, the kind of soft death that never makes an
announcement; the mirror plus its first archive snapshot is the hedge.
