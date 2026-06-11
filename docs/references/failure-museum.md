# Failure museum

Post mortems of defects that mattered, written at fix time. Verification cultures
publish their failures; a defect whose lesson lives only in a closed bug tracker
teaches nobody. The ritual: when a defect with a lesson is fixed, the fixing slice
appends an entry here naming what failed, why the existing verification missed it,
and what changed so the class is caught next time. Entries are never written
retroactively from memory; a thin museum is honest, a reconstructed one is not.

## Open exhibits (defect known, post mortem owed at fix time)

**Decorated `com` packing over an overflowed unbounded result** (bead `enc-1ta`).
A decorated basic operation can pack the `com` decoration with a result that
overflowed to an unbounded interval; `com` promises bounded. The post mortem
belongs here when the fix lands, and should answer why neither the decoration
property tests nor the Kani lattice proofs constrained the packing site.

## Documented conservative limitations (not defects, recorded for contrast)

**Affine `sqrt` declines ranges touching zero** (bead `enc-fj5`). The Chebyshev
construction needs one signed curvature over a range with finite secant slope; a
range touching zero has none, and the implementation declines rather than widening
silently. Documented at the call site and pinned by a regression test. Listed here
because a future reader hunting "why does sqrt fail near zero" should find the
distinction between a recorded decline and a soundness hole.
