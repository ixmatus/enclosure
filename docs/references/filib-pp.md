---
slug: filib-pp
category: algorithms
citation: >-
  Lerch, M., Tischler, G., Wolff von Gudenberg, J., Hofschuster, W., and
  Kramer, W., "FILIB++, a fast interval library supporting containment
  computations", ACM Transactions on Mathematical Software 32(2), pp. 299-324,
  June 2006. Library version 3.0.2 (2011-05-10).
canonical-url: https://www2.math.uni-wuppertal.de/wrswt/software/filib.html
doi: 10.1145/1141885.1141893
archived-url: http://web.archive.org/web/20260428013447/http://www2.math.uni-wuppertal.de/wrswt/software/filib.html
archive-date: 2026-04-28
retrieved: 2026-06-11
license: LGPL v2 or later per the tarball's grant (LGPL 2.1 text shipped); the web page itself states none
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: secondary
consumers:
  - docs/decisions/0001-enclosure-monorepo-and-round-float-layering.md (ecosystem precedent)
verification:
  - none
sha256: none
notes: >-
  The fast templated C++ interval library of the Wuppertal/Wurzburg school,
  cited in ADR-0001 as ecosystem precedent. Its FI_LIB test vectors live on
  inside the vendored ITF1788 corpus as fi_lib.itl (LGPL-2.1-or-later),
  which is the working connection to this workspace.
---

The page is alive in 2026 but the last release is 2011; a university hosted
software page fifteen years past its last release is the academic personal rot
class on a slow fuse. Beyond the precedent citation, filib++ matters through
its test suite: ~800 of the Octave interval package's generated cases derive
from FI_LIB vectors, vendored here under
[itf1788-framework](itf1788-framework.md). Fresh Wayback saves were rate
limited on 2026-06-11; the recorded snapshot is pre existing and recent.
