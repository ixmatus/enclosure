---
slug: johansson-2017-arb
category: history-philosophy
citation: >-
  Johansson, F., "Arb: Efficient Arbitrary-Precision Midpoint-Radius Interval
  Arithmetic", IEEE Transactions on Computers 66(8), pp. 1281-1292, August
  2017. Library merged into FLINT in 2023.
canonical-url: https://arxiv.org/abs/1611.02831
doi: 10.1109/TC.2017.2690633
archived-url: http://web.archive.org/web/20260401000258/https://arxiv.org/abs/1611.02831
archive-date: 2026-04-01
retrieved: 2026-06-11
license: arXiv copy openly readable; library LGPL-2.1-or-later as Arb, LGPL-3.0-or-later as part of FLINT
vendor-status: pointer-only
rot-risk: stable-publisher
provenance-class: secondary
consumers:
  - crates/interval-1788/docs/decisions/0001-inf-sup-representation-and-invariant.md
verification:
  - none
sha256: none
notes: >-
  The modern case for midpoint-radius (ball) representation, cited in the
  interval-1788 representation decision as the alternative inf-sup was chosen
  against. One half of the three representation comparison the estate needs
  for the pfloat-ball division of labor question; Rump's Acta Numerica survey
  is the other half.
---

Arb demonstrates what ball arithmetic buys at arbitrary precision: a radius
that needs few bits, cheap wrapping of rounding error, and fast asymptotics.
The interval-1788 ADR-0001 chose inf-sup because IEEE 1788's Level 1 semantics
and its conformance vectors are stated in endpoints, and because the directed
rounding backend contract maps one to one onto endpoint arithmetic. The estate
context sharpens the comparison: the sibling pfloat-ball crate is mid-rad
native and plans a 1788 face, so which crate canonically owns the 1788 surface
is an open coordination question (bead enc-pvf input, recorded in the close
out report; see also [rump-2010-acta-numerica](rump-2010-acta-numerica.md)).

The library home (arblib.org, snapshot 2026-03-05) now redirects readers to
FLINT (flintlib.org, snapshot 2026-06-07) after the 2023 merge. Fresh Wayback
saves were rate limited on 2026-06-11; recorded snapshots are pre existing.
