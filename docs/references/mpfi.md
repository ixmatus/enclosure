---
slug: mpfi
category: algorithms
citation: >-
  Revol, N. and Rouillier, F., "Motivations for an Arbitrary Precision
  Interval Arithmetic and the MPFI Library", Reliable Computing 11(4),
  pp. 275-290, August 2005. Library version 1.5.4.
canonical-url: https://perso.ens-lyon.fr/nathalie.revol/software.html
doi: 10.1007/s11155-005-6891-y
archived-url: http://web.archive.org/web/20260506043247/https://perso.ens-lyon.fr/nathalie.revol/software.html
archive-date: 2026-05-06
retrieved: 2026-06-11
license: LGPL-3.0 (COPYING and COPYING.LESSER in the 1.5.4 tarball)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: secondary
consumers:
  - docs/decisions/0001-enclosure-monorepo-and-round-float-layering.md (ecosystem precedent)
verification:
  - none
sha256: none
notes: >-
  Cited in ADR-0001 as ecosystem precedent: interval libraries ship separately
  from affine ones, MPFI being the canonical arbitrary precision interval
  library over MPFR. The repository at gitlab.inria.fr/mpfi/mpfi sits behind
  an Anubis anti bot wall that blocks automated fetches and likely archive
  crawls; Revol's personal page is the reliable accessible mirror and names
  1.5.4 as latest.
---

MPFI grounds one structural claim in the workspace: the monorepo's decision to
keep `interval-1788` and `affine-arith` as separate crates follows what the
ecosystem has always done (MPFI and filib++ on the interval side, libaffa and
aaflib on the affine side). Beyond precedent it is a candidate oracle for
arbitrary precision interval endpoints, though INTLAB and kv sit closer to the
1788 surface. Fresh Wayback saves failed (rate limited); the perso page
snapshot is pre existing and recent; the GitLab snapshot
(web.archive.org/web/20260413100530) may show the anti bot wall rather than
the repository.
