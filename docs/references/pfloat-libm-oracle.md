---
slug: pfloat-libm-oracle
category: spec-conformance
citation: >-
  Springmeyer, P., pfloat-libm (correctly rounded pure Rust libm, part of the
  pfloat workspace), pinned revision 04c9761cccfc2724dd7d1c941a05840c9a3d9eca.
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
verification:
  - crates/elementary-oracle/tests/oracle.rs
sha256: none
notes: >-
  The truth source of the nightly oracle lane: correctly rounded exp and ln
  values the f64 fixture's and affine-arith's enclosures must bracket. Sibling
  estate crate, same author; the git pin makes the lane reproducible.
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
