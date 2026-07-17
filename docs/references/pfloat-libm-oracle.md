---
slug: pfloat-libm-oracle
category: spec-conformance
citation: >-
  Springmeyer, P., the pfloat workspace, pinned revision
  04c9761cccfc2724dd7d1c941a05840c9a3d9eca: pfloat-libm (correctly rounded pure
  Rust libm) and the base pfloat crate (BigFloat, correctly rounded
  arbitrary-precision arithmetic with directed rounding).
canonical-url: https://github.com/ixmatus/pfloat
doi: none
archived-url: none (estate repository; the pin is the durable reference, not a snapshot)
archive-date: n/a
retrieved: 2026-06-11
license: dual MIT / Apache-2.0 (estate convention)
vendor-status: pointer-only
rot-risk: single-maintainer
provenance-class: primary
consumers:
  - crates/elementary-oracle/Cargo.toml
  - crates/elementary-oracle/src/lib.rs
  - docs/decisions/0002-affine-elementary-functions.md
  - crates/round-float/docs/decisions/0001-round-transcendental-extension-trait.md
  - docs/decisions/0007-transcendental-growth-round-two.md (round two truth source; coverage verified at workspace level 2026-07-16)
verification:
  - crates/elementary-oracle/tests/oracle.rs
  - crates/elementary-oracle/tests/arithmetic.rs
sha256: none
notes: >-
  The truth source of the nightly oracle lane: correctly rounded exp and ln
  values the f64 fixture's and affine-arith's enclosures must bracket, and (from
  2026-07-17) correctly rounded directed add/sub/mul/div/sqrt/recip/sqr values the
  TightF64 backend must match exactly for tightness and the f64 fixture must
  bracket for enclosure. Sibling estate crate, same author; the git pin makes the
  lane reproducible.
---

The round-float f64 fixture rests its `exp`/`ln` bounds on musl's stated accuracy
goal ([musl-libm-accuracy](musl-libm-accuracy.md)); that is an assumption about a
third party, and the oracle lane discharges it empirically. `elementary-oracle`
is workspace excluded, runs on its own pinned nightly (pfloat needs
`generic_const_exprs`), and certifies that the fixture's directed bounds and
`affine-arith`'s Chebyshev enclosures bracket pfloat-libm's correctly rounded
values over the swept grids. The shipped crates never link it.

An oracle is only as good as its own correctness: pfloat-libm's claim to correct
rounding is verified in its own repository against the Lefèvre and Muller worst
case rounding corpus; this entry leans on that lane rather than restating it.
The pinned revision, not a web snapshot, is the durable reference; a moved or
rewritten GitHub repository still resolves through any clone of the estate.

The arithmetic lane (`tests/arithmetic.rs`, bead enc-ebd) grows the same source
into the tight binary64 backend of round-float decision record 0002. pfloat-libm
exposes no bare arithmetic, so the base pfloat crate's `BigFloat` supplies the
correctly rounded directed reference: `add`, `sub`, `mul`, and `sqr` compute the
exact dyadic result at a precision above its width (the lane asserts the INEXACT
flag stays clear) and round it once to `f64` per direction; `div` and `recip`
round the non-terminating quotient to an intermediate precision in the target
direction and then to `f64` in the same direction, which is correctly rounded
because directed double rounding through a higher precision nests (unlike round to
nearest); `sqrt` uses pfloat-libm's `sqrt_round` straight to `f64`. Coverage gaps,
which feed the README disclosure's named failure mode: division by zero and the
square root of a negative are excluded (pole and domain edges the trait routes
through a separate convention, not rounding hazards), and the certification is a
grid sweep, not exhaustive, so it can miss a defect no sampled input hits.
