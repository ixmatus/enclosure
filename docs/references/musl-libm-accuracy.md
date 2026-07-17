---
slug: musl-libm-accuracy
category: spec-conformance
citation: >-
  musl libc wiki, "Mathematical Library" page: the accuracy statement for
  musl's libm.
canonical-url: https://wiki.musl-libc.org/mathematical-library.html
doi: none
archived-url: http://web.archive.org/web/20260611083914/https://wiki.musl-libc.org/mathematical-library.html
archive-date: 2026-06-11
retrieved: 2026-06-11
license: unstated
vendor-status: pointer-only
rot-risk: community-run
provenance-class: primary
consumers:
  - crates/round-float/src/f64_impl.rs (TRANSCENDENTAL_MARGIN rests on this statement)
  - crates/round-float/src/f64_impl.rs (the RoundTrig sin/cos/tan fixture; the trig
    margin leans on the Payne-Hanek reduction statement below, not the table line)
  - crates/round-float/docs/decisions/0001-round-transcendental-extension-trait.md
  - docs/decisions/0007-transcendental-growth-round-two.md (the arc-hyperbolic exception is that record's load-bearing fact)
verification:
  - crates/elementary-oracle/tests/oracle.rs (the nightly lane discharges the assumption empirically)
sha256: none
notes: >-
  The single external accuracy claim the f64 fixture's transcendental bounds
  rest on, quoted verbatim below because the soundness argument depends on
  the exact wording. Fresh Wayback save landed 2026-06-11.
---

The page states, verbatim: correctly rounded in all rounding modes for
`ceil, copysign, fabs, fdim, floor, fma, fmax, fmin, fmod, frexp, ldexp,
logb, modf, nearbyint, nextafter, nexttoward, rint, remainder, remquo, round,
scalbln, scalbn, sqrt, trunc`; for other functions, "Other functions should
have small ulp error in nearest rounding mode and the sign of zero should be
correct. (the goal is < 1.5 ulp error, ie. the last bit may be wrong...)",
with named exceptions (several complex functions, Bessel, `lgamma`/`tgamma`,
arc hyperbolics) that do not include `exp` or `log`.

The fixture's chain of trust: `libm::sqrt` is on the correctly rounded list
and is consumed directly; `exp` and `log` fall under the sub 1.5 ulp goal, and
the fixture widens them by `TRANSCENDENTAL_MARGIN` (8 ulp), a margin of more
than five over the stated goal. A goal is not a theorem, which is why the
nightly oracle lane ([pfloat-libm-oracle](pfloat-libm-oracle.md)) checks the
bracketing empirically against correctly rounded values over swept grids. The
Rust port inherits the claim ([rust-libm-crate](rust-libm-crate.md)).

## The trigonometric reduction anchor (workspace ADR-0005 part 2)

`sin`, `cos`, and `tan` are not on the correctly rounded list either, and they
are not among the named exceptions, so they fall under the same sub 1.5 ulp
goal. Trigonometry carries one obligation `exp` and `log` do not: argument
reduction. musl's trig inherits the fdlibm Payne-Hanek reduction, which
computes the reduced argument against a many-digit representation of `2/pi`, so
the sub 1.5 ulp goal is claimed relative to the mathematical function value at
every finite magnitude, including at large arguments (well past `2^50`, where a
naive reduction would lose all significance) and near the zeros of `sin` (where
the true value is tiny and the relative error the goal permits is
correspondingly tiny). It is this reduction-aware reading of the goal, not the
generic table line, that the `RoundTrig` fixture margin leans on: the same
`TRANSCENDENTAL_MARGIN` widening is sound for `sin`/`cos`/`tan` precisely
because the reduction keeps the ulp claim about the mathematical result. If the
oracle lane ever falsifies the claim at a large argument or near a critical
point, the trig fixture margin is wrong, not merely loose. The empirical
discharge landed with the round-one oracle grids (large-argument and
near-critical-point sweeps against pfloat-libm, green at integration).

The exception list is itself load bearing: "arc hyperbolics" appear among the
named exceptions, so `asinh`/`acosh`/`atanh` have NO stated accuracy goal for
a fixture margin to dominate. Transcendental growth round two
(workspace ADR-0007) therefore anchors those three margins to measured
oracle error with headroom rather than to this page, and says so at the
constants; for every other round-two function (inverse trig, exp2/exp10,
log2/log10) the sub 1.5 ulp goal covers and the shipped doctrine applies.
