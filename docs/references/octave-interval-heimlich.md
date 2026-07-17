---
slug: octave-interval-heimlich
category: spec-conformance
citation: >-
  Heimlich, O., "interval: Real-valued interval arithmetic", GNU Octave
  package, version 3.2.2 (2026-02-16); manual by Heimlich, Dahne, Pernice,
  2015 to 2017.
canonical-url: https://gnu-octave.github.io/packages/interval/
doi: none
archived-url: http://web.archive.org/web/20260317002628/https://gnu-octave.github.io/packages/interval/
archive-date: 2026-03-17
retrieved: 2026-06-11
license: GPL-3.0-or-later (package DESCRIPTION and the manual)
vendor-status: pointer-only
rot-risk: single-maintainer
provenance-class: secondary
consumers:
  - crates/interval-1788/README.md (named behavioral oracle in the provenance disclosure)
verification:
  - none yet (oracle cross checks are manual; no automated lane)
sha256: none
notes: >-
  The most complete open IEEE 1788 implementation and the named behavioral
  oracle for interval outputs. GPL means pointer only and outputs only: results
  are cross checked, code is never adapted, and the README disclosure promises
  any future differential lane runs out of process to keep copyleft out of the
  link graph.
---

Heimlich's package carries an explicit conformance claim (manual,
`package_doc/Conformance-Claim.html`): set based flavor with IEEE 754
conformance for the infsup binary64 type, no compressed arithmetic, no further
flavors, and some operations (notably reverse operations) at valid or accurate
rather than tightest accuracy. Its ~9,500 generated tests derive from the
ITF1788 corpus this registry vendors ([itf1788-framework](itf1788-framework.md));
the manual's Acknowledgments page itemizes the conversion counts
(~6,000 libieeep1788, ~1,500 MPFI, ~800 FI_LIB, ~160 C-XSC; snapshot
web.archive.org/web/20231224040200 of Acknowledgments.html).

Hosting is in transition: octave.sourceforge.io still serves 3.2.2 but banners
itself unmaintained (snapshot 2025-12-16, web.archive.org/web/20251216132753);
the packages.octave.org entry above is current; source of record is the
SourceForge Mercurial tree, with the GitHub mirror (`oheim/octave-interval`)
lagging at 3.2.1. Single maintainer, one posteo.de address.

The 2026-06-11 archive debt (Conformance-Claim.html and the GitHub mirror,
saves rate limited that session) was cleared on the 2026-07-17 retry:
Conformance-Claim.html at
web.archive.org/web/20260717080736/https://octave.sourceforge.io/interval/package_doc/Conformance-Claim.html
and the mirror at
web.archive.org/web/20260717080758/https://github.com/oheim/octave-interval.
The conformance claim text stays summarized above so the fact survives the
URL regardless.
