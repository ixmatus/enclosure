---
slug: libieeep1788-nehmeier
category: spec-conformance
citation: >-
  Nehmeier, M., "libieeep1788: A C++ Implementation of the IEEE interval
  standard P1788", 2014 IEEE Conference on Norbert Wiener in the 21st Century
  (21CW), June 2014, pp. 1-6. Repository github.com/nehmeier/libieeep1788.
canonical-url: https://github.com/nehmeier/libieeep1788
doi: 10.1109/NORBERT.2014.6893854
archived-url: http://web.archive.org/web/20260302004757/https://github.com/nehmeier/libieeep1788
archive-date: 2026-03-02
retrieved: 2026-06-11
license: Apache-2.0 (LICENSE file; NOTICE copyright 2013-2015 Marco Nehmeier, University of Wurzburg)
vendor-status: pointer-only
rot-risk: single-maintainer
provenance-class: secondary
consumers:
  - crates/interval-1788/README.md (the "C++ reference" named as behavioral oracle)
  - docs/decisions/0008-reduction-operations-kulisch-accumulator.md (the reduction mode-enum surface witness, surveyed 2026-07-16)
verification:
  - none yet (oracle cross checks are manual; no automated lane)
sha256: none
notes: >-
  The reference C++ implementation written alongside the standardization, and
  the origin of the largest ITF1788 vector block (~6,000 cases). Two caveats
  temper its oracle authority: it implements the preliminary P1788 draft, not
  the final 2015 text, and its own README calls it work in progress and not
  assumed bug free.
---

When the interval-1788 README says outputs are cross checked against "the C++
reference", this is the artifact. Its oracle weight comes from proximity to the
working group (Nehmeier sat in it; the library and the ITF1788 framework share
authorship) rather than from maintenance: last push 2015-06-17, dormant eleven
years, header only C++11 over MPFR/GMP/Boost.

Weight claims by what the source could not have seen: a draft era
implementation cannot witness final text changes, so on any disagreement
between libieeep1788 and the published standard, the standard wins and the
disagreement is worth recording here. Its test suite lives on inside the
vendored ITF1788 corpus ([itf1788-framework](itf1788-framework.md)) as the
libieeep1788_*.itl files. Paper page snapshot:
web.archive.org/web/20250422085713/https://ieeexplore.ieee.org/document/6893854.
Fresh Wayback saves were rate limited on 2026-06-11; recorded snapshots are pre
existing.
