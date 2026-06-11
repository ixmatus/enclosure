---
slug: kopetzki-schurmann-althoff-2017
category: algorithms
citation: >-
  Kopetzki, A.-K., Schurmann, B., and Althoff, M., "Methods for order
  reduction of zonotopes", 2017 IEEE 56th Annual Conference on Decision and
  Control (CDC), pp. 5626-5633, December 2017.
canonical-url: https://mediatum.ub.tum.de/doc/1379661/document.pdf
doi: 10.1109/CDC.2017.8264508
archived-url: http://web.archive.org/web/20240819105448/https://mediatum.ub.tum.de/doc/1379661/document.pdf
archive-date: 2024-08-19
retrieved: 2026-06-11
license: unstated (TUM mediatum institutional repository copy; IEEE holds the proceedings copyright)
vendor-status: pointer-only
rot-risk: stable-publisher
provenance-class: secondary
consumers:
  - docs/decisions/0004-noise-term-condensation.md
  - docs/references/kernel-map.md
verification:
  - none
sha256: none
notes: >-
  The systematic survey of zonotope order reduction methods (Girard,
  Combastel, PCA aligned, and others) with measured over approximation
  bounds. The single best map of the design space ADR-0004's condensation
  sits in, and the source of the computable inflation bounds a future
  condensation upgrade would surface in the API.
---

What this paper adds over the origin methods is exactly what the estate's
degradation work wants: each reduction comes with a measured bound on how much
the over approximation grew. The downstream plan (SMIL bead smil-dgv4) records
condensation events as provenance nodes carrying an inflation bound; this
survey is where those bounds come from. The TUM institutional repository copy
was fetched live (HTTP 200, 1.04 MB); fresh Wayback saves were rate limited on
2026-06-11; the recorded snapshot is pre existing.
