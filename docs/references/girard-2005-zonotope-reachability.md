---
slug: girard-2005-zonotope-reachability
category: algorithms
citation: >-
  Girard, A., "Reachability of Uncertain Linear Systems Using Zonotopes", in
  Hybrid Systems: Computation and Control (HSCC 2005), LNCS 3414,
  pp. 291-305, Springer, 2005.
canonical-url: https://hal.science/hal-00307003
doi: 10.1007/978-3-540-31954-2_19
archived-url: http://web.archive.org/web/20210802232011/https://hal.archives-ouvertes.fr/hal-00307003
archive-date: 2021-08-02 (legacy HAL URL form; the hal.science form has no snapshot yet)
retrieved: 2026-06-11
license: HAL open deposit (Springer holds the chapter copyright)
vendor-status: pointer-only
rot-risk: stable-publisher
provenance-class: secondary
consumers:
  - docs/decisions/0004-noise-term-condensation.md
verification:
  - none
sha256: none
notes: >-
  Origin of the most cited zonotope order reduction heuristic: fold the
  smallest generators into an axis aligned box. The reachability community's
  counterpart of affine noise term condensation, and the first rung of the
  improvement ladder ADR-0004 leaves open.
---

Girard's reduction keeps the largest generators exact and over approximates
the rest by a box, with a computable bound on the inflation. The
correspondence to `affine-arith` is exact (an affine form is a zonotope, a
deviation term a generator), and the difference is instructive: ADR-0004's
condensation folds the tail into one fresh symbol bounding the summed
magnitude (width preserving by construction), where Girard's box keeps per
axis structure. The systematic comparisons
([kopetzki-schurmann-althoff-2017](kopetzki-schurmann-althoff-2017.md),
[yang-scott-2018](yang-scott-2018.md)) measure when each wins. Fresh Wayback
saves were rate limited on 2026-06-11; the recorded snapshot covers the legacy
HAL URL, and a save of the current hal.science form is owed.
