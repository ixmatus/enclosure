---
slug: libaffa
category: algorithms
citation: >-
  Gay, O., Coeurjolly, D., and Hurst, N.J., libaffa, a C++ affine arithmetic
  library, version 0.9.6, 23 June 2006.
canonical-url: https://www.nongnu.org/libaffa/
doi: none
archived-url: http://web.archive.org/web/20240814220007/https://www.nongnu.org/libaffa/
archive-date: 2024-08-14
retrieved: 2026-06-11
license: LGPL-2.1 (COPYING in the 0.9.6 tarball; Savannah project page concurs)
vendor-status: pointer-only
rot-risk: community-run
provenance-class: secondary
consumers:
  - docs/decisions/0001-enclosure-monorepo-and-round-float-layering.md (ecosystem precedent)
verification:
  - none
sha256: none
notes: >-
  Affine side of the ADR-0001 ecosystem precedent pair. Dormant since 2006
  (the git mirror at github.com/ogay/libaffa last moved 2013); the page itself
  says development is inactive. Prior art for existence, not for code.
---

libaffa demonstrates the same packaging fact as [aaflib](aaflib.md): affine
arithmetic libraries ship standalone, never folded into interval libraries,
which is the precedent ADR-0001 cites for the workspace's crate split. Nothing
in `affine-arith` derives from it (the provenance discipline derives kernels
from the Stolfi papers), so the entry is a pointer with a license recorded for
completeness. Fresh Wayback saves were rate limited on 2026-06-11; the
recorded snapshot is pre existing; the Savannah project page snapshot
(2025-09-06, web.archive.org/web/20250906234738) covers the hosting side.
