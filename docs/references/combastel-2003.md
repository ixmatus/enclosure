---
slug: combastel-2003
category: algorithms
citation: >-
  Combastel, C., "A state bounding observer based on zonotopes", Proceedings
  of the 2003 European Control Conference (ECC), Cambridge, UK, pp. 2589-2594,
  September 2003.
canonical-url: https://skoge.folk.ntnu.no/prost/proceedings/ecc03/pdfs/437.pdf
doi: 10.23919/ECC.2003.7085991
archived-url: https://web.archive.org/web/20260611082519/https://skoge.folk.ntnu.no/prost/proceedings/ecc03/pdfs/437.pdf
archive-date: 2026-06-11
retrieved: 2026-06-11
license: unstated (proceedings mirror on an NTNU personal page; IEEE/EUCA hold the proceedings copyright)
vendor-status: pointer-only
rot-risk: academic-personal
provenance-class: secondary
consumers:
  - docs/decisions/0004-noise-term-condensation.md
verification:
  - none
sha256: none
notes: >-
  The second origin method the order reduction literature benchmarks
  ("Combastel's method"): a covariance weighted generator selection. A
  provenance correction rides on this entry: the paper is from ECC 2003, not
  CDC as commonly miscited; both Kopetzki et al and Yang and Scott credit
  exactly this ECC paper.
---

Combastel's reduction sorts generators by a different criterion than Girard's
and tends to preserve directional structure better on elongated sets; the
comparisons in [kopetzki-schurmann-althoff-2017](kopetzki-schurmann-althoff-2017.md)
and [yang-scott-2018](yang-scott-2018.md) treat the two as the baseline pair.
For ADR-0004's ladder this entry marks the second classical option the
condensation strategy can be measured against. The accessible copy lives on a
proceedings mirror hosted from Sigurd Skogestad's NTNU personal page; that URL
had never been archived, and the fresh 2026-06-11 save above is its first
snapshot.
