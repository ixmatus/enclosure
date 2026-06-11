---
slug: intlab-rump
category: spec-conformance
citation: >-
  Rump, S.M., INTLAB, the MATLAB/Octave toolbox for reliable computing,
  Hamburg University of Technology. Version 14.1 (2026).
canonical-url: https://www.tuhh.de/ti3/rump/intlab/
doi: none
archived-url: http://web.archive.org/web/20250418222055/https://www.tuhh.de/ti3/rump/intlab/
archive-date: 2025-04-18
retrieved: 2026-06-11
license: >-
  restricted: free for private, academic, and internal company use; embedding
  in a commercial product requires a special license from Rump; attribution
  required; provided as is
vendor-status: legally-cannot
rot-risk: academic-personal
provenance-class: secondary
consumers:
  - none yet (oracle candidate named in bead enc-my9)
verification:
  - none
sha256: none
notes: >-
  The de facto standard verified computing toolbox and an oracle candidate
  for interval and affine outputs (INTLAB ships affine arithmetic too, per
  the Rump and Kashiwagi paper). The license is the reason this entry exists
  in its restricted form: pointer only, outputs only, nothing vendored or
  adapted.
---

INTLAB's oracle value is high (Rump wrote both it and much of the verification
methods literature) and its license is the sharpest in this registry: not open
source, embedding requires permission, so the only sound relationship is cross
checking outputs by hand or through a person who holds a copy. The affine
arithmetic implementation described in
[rump-kashiwagi-2015](rump-kashiwagi-2015.md) lives inside INTLAB, which makes
it the executable form of that paper if a comparison lane is ever wanted.
Fresh Wayback saves were rate limited on 2026-06-11; the recorded snapshot is
pre existing.
