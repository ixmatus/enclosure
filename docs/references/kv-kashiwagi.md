---
slug: kv-kashiwagi
category: spec-conformance
citation: >-
  Kashiwagi, M., "kv - a C++ Library for Verified Numerical Computation",
  version 0.4.60 (2026-05-30), 2013 to 2026.
canonical-url: https://verifiedby.me/kv/index-e.html
doi: none
archived-url: http://web.archive.org/web/20260225213024/https://verifiedby.me/kv/index-e.html
archive-date: 2026-02-25
retrieved: 2026-06-11
license: MIT (LICENSE.txt in github.com/mskashi/kv, Copyright 2013-2026 Masahide Kashiwagi)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: secondary
consumers:
  - none yet (oracle candidate named in bead enc-my9; not wired into any lane)
verification:
  - none yet
sha256: none
notes: >-
  A candidate behavioral oracle for interval operations alongside libieeep1788
  and Octave interval. The site verifiedby.me is a personal domain (the old
  Waseda host www.kashi.info.waseda.ac.jp now serves only a lab page); source
  of record is github.com/mskashi/kv.
---

kv implements interval arithmetic, affine arithmetic, automatic differentiation,
and verified ODE integration in C++ over double and double double endpoints. It
matters to this workspace twice over: as a behavioral oracle candidate for
`interval-1788` outputs, and because Kashiwagi co wrote the affine arithmetic
implementation paper with Rump ([rump-kashiwagi-2015](rump-kashiwagi-2015.md))
whose floating point practicalities are the closest published account of building
affine arithmetic over directed rounding, the same problem `affine-arith` solves.
Outputs are cross checked, never adapted as code; the MIT license would permit
more, the provenance discipline does not need it.

No paper about kv exists; the website is the citation (the site's "papers
related to kv" section collects papers that mention it). The English overview
deck `verifiedby.me/kv/kv-intro-e.pdf` is the closest citable exposition.

Fresh Wayback saves were rate limited on 2026-06-11; the recorded snapshot
(2026-02-25) is pre existing and covers the current 0.4.60 page.
