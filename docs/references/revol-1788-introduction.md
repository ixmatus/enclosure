---
slug: revol-1788-introduction
category: spec-conformance
citation: >-
  Revol, N., "Introduction to the IEEE 1788-2015 Standard for Interval
  Arithmetic", in Numerical Software Verification (NSV 2017, a CAV workshop),
  LNCS 10381, Springer, 2017, pp. 14-21.
canonical-url: https://inria.hal.science/hal-01559955
doi: 10.1007/978-3-319-63501-9_2
archived-url: https://web.archive.org/web/20260611082340/https://inria.hal.science/hal-01559955/file/NRevol-NSV17-HAL.pdf
archive-date: 2026-06-11
retrieved: 2026-06-11
license: open HAL deposit (HAL distribution authorization; Springer holds the chapter copyright)
vendor-status: pointer-only
rot-risk: stable-publisher
provenance-class: secondary
consumers:
  - crates/interval-1788/src/spec.rs
  - crates/interval-1788/README.md
  - crates/interval-1788/docs/decisions/0001-inf-sup-representation-and-invariant.md
verification:
  - none
sha256: none
notes: >-
  The open exposition of the standard the spec module cites by name: the
  interval model, flavors, decorations, and Level 1 versus Level 2 in eight
  pages. HAL record hal-01559955 (deposited 2017-07-11); the PDF got a fresh
  Wayback save 2026-06-11. The HAL record lists only the volume DOI; the
  chapter DOI above is the Crossref verified one.
---

Of the three free proxies for [ieee-1788-2015](ieee-1788-2015.md), Revol's is
the one written as an introduction rather than a history: when `spec.rs`
states the interval model in prose, this is the open text a reader can check
the prose against without buying the standard. The HAL record page snapshot
is pre existing (2025-01-22, web.archive.org/web/20250122223117); the Springer
chapter landing page snapshot likewise (2022-01-21). Saves of those two were
rate limited on 2026-06-11; the PDF itself, the load bearing artifact, is
freshly archived.
