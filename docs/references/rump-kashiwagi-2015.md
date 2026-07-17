---
slug: rump-kashiwagi-2015
category: algorithms
citation: >-
  Rump, S.M. and Kashiwagi, M., "Implementation and improvements of affine
  arithmetic", Nonlinear Theory and Its Applications, IEICE, 6(3), pp. 341-359,
  2015 (correction 2016-04-11).
canonical-url: https://www.jstage.jst.go.jp/article/nolta/6/3/6_341/_article
doi: 10.1587/nolta.6.341
archived-url: http://web.archive.org/web/20240604163450/https://www.jstage.jst.go.jp/article/nolta/6/3/6_341/_article
archive-date: 2024-06-04
retrieved: 2026-06-11
license: journal free access on J-STAGE; no explicit CC notation; author copy at https://www.tuhh.de/ti3/paper/rump/RuKas14.pdf (archived 2024-08-09)
vendor-status: pointer-only
rot-risk: stable-publisher
provenance-class: primary
consumers:
  - docs/decisions/0004-noise-term-condensation.md
verification:
  - none yet
sha256: none
notes: >-
  The closest published account of implementing affine arithmetic rigorously
  over floating point with directed rounding error capture, by the INTLAB
  author and the kv author. Reading list seed for the condensation and
  inflation bound follow-up work; the in tree kernels were derived from the
  Stolfi line, not from this paper.
---

Rump and Kashiwagi treat exactly the rigor problems `affine-arith` faces:
folding floating point rounding error of each affine operation outward into the
noise term budget, and the cost trade offs of the multiply. The workspace
kernels were derived from the de Figueiredo and Stolfi constructions
([figueiredo-stolfi-2004](figueiredo-stolfi-2004.md)) before this paper was
consulted; it enters the registry as the literature anchor for the condensation
and inflation bound improvements that ADR-0004 leaves open, alongside the
zonotope order reduction line
([kopetzki-schurmann-althoff-2017](kopetzki-schurmann-althoff-2017.md)).

The journal copy is free access on J-STAGE; Rump's author copy lives under his
TUHH pages (academic personal rot risk; fresh Wayback save 2026-06-11 at
web.archive.org/web/20260611081518/https://www.tuhh.de/ti3/paper/rump/RuKas14.pdf).
The J-STAGE page snapshot is pre existing (saves rate limited on 2026-06-11).

Map versus territory: Rump's own publications listing
(`www.tuhh.de/ti3/publications.shtml?author=rump`) cites this paper with the
wrong volume and pages (2(3):1101-1119) while linking the correct DOI; the
J-STAGE publisher record above is authoritative. That listing URL's owed save
landed on the 2026-07-17 retry:
web.archive.org/web/20260717080902/https://www.tuhh.de/ti3/publications.shtml?author=rump
(the discrepancy is thereby preserved as it stood).
The J-STAGE record carries a correction dated 2016-04-11 with empty detail
fields; no separate erratum DOI exists.
