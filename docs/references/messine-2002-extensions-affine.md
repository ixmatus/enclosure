---
slug: messine-2002-extensions-affine
category: algorithms
citation: >-
  Messine, F., "Extensions of Affine Arithmetic: Application to Unconstrained
  Global Optimization", Journal of Universal Computer Science 8(11),
  pp. 992-1015, 28 November 2002.
canonical-url: https://lib.jucs.org/article/27919/
doi: 10.3217/jucs-008-11-0992
archived-url: http://web.archive.org/web/20260201065056/https://lib.jucs.org/article/27919
archive-date: 2026-02-01
retrieved: 2026-06-11
license: >-
  J.UCS Open Content License (the journal's 1995 to 2019 tier; not a CC
  license); free to read, copyright held by J.UCS (Verlag der TU Graz);
  reader facing license text not published anywhere locatable
vendor-status: pointer-only
rot-risk: stable-publisher
provenance-class: primary
consumers:
  - docs/decisions/0004-noise-term-condensation.md
verification:
  - none yet
sha256: none
notes: >-
  The AF1/AF2 extended affine forms: structured error accumulator terms that
  let cap pressure consume anonymous noise first so named correlations survive
  longer. The literature anchor for the ADR-0004 follow-up ladder's second
  rung. Two metadata traps recorded below.
---

Messine's AF1 and AF2 forms route new linearization error into one or a few
persistent accumulator terms instead of minting unbounded fresh symbols. For
`affine-arith`, whose ADR-0004 condensation folds the smallest tail terms under
a caller budget, AF1/AF2 is the nearest published alternative design: fixed
memory by construction rather than by periodic folding. The condensation
decision record cross references this entry for that comparison.

Metadata traps for future lookups:

1. The publisher's own metadata (lib.jucs.org page and the DataCite DOI
   record) misspells the title as "Extentions"; the PDF itself prints
   "Extensions". Cite the paper's spelling.
2. The DOI is DataCite registered, not Crossref; Crossref-only tooling
   reports it nonexistent.

The PDF (216,542 bytes, sha256
5a3e2c0f54db650929b7fc97c52898177efc4021e14c45554b0587a177ed5bbe) is preserved:
the 2023-05-28 Wayback capture of the download URL serves bytes identical to
the live copy (hash verified 2026-06-11). Fresh saves failed against an
overloaded Save Page Now (520s); recorded snapshots are pre existing. Given the
unlocatable license text, pointer plus archive, no vendoring.

The sequel is [messine-touhami-2006](messine-touhami-2006.md) (quadratic
forms, paywalled).
