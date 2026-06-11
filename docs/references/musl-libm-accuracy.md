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
  - crates/round-float/docs/decisions/0001-round-transcendental-extension-trait.md
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
