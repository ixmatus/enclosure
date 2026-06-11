---
slug: rust-libm-crate
category: spec-conformance
citation: >-
  rust-lang/libm, "A port of MUSL's libm to Rust", the libm crate. Repository
  archived; development continues inside rust-lang/compiler-builtins.
canonical-url: https://github.com/rust-lang/libm
doi: none
archived-url: http://web.archive.org/web/20260216054812/https://github.com/rust-lang/libm
archive-date: 2026-02-16
retrieved: 2026-06-11
license: MIT (LICENSE.txt; contributions accepted under MIT OR Apache-2.0)
vendor-status: pointer-only
rot-risk: stable-publisher
provenance-class: secondary
consumers:
  - crates/round-float/Cargo.toml (optional dependency)
  - crates/round-float/src/f64_impl.rs
  - crates/round-float/README.md
verification:
  - crates/elementary-oracle/tests/oracle.rs (indirectly; the oracle certifies the bounds built over libm values)
sha256: none
notes: >-
  The f64 fixture's host math: libm::sqrt is correctly rounded (consumed
  directly), libm::exp and libm::log are faithfully rounded (widened by the
  fixture's transcendental margin). The repository was archived after the
  merge into compiler-builtins; the crate remains published and the pin
  resolves, but future issues and fixes live in the new home.
---

The dependency exists because `no_std` targets have no host libm; the crate
ports musl's algorithms, so its accuracy story is musl's accuracy story
([musl-libm-accuracy](musl-libm-accuracy.md)). The fixture treats it
accordingly: `sqrt` trusted as correctly rounded, `exp`/`log` wrapped in the
outward margin that the nightly oracle lane then certifies empirically. The
repository archival (merged into `rust-lang/compiler-builtins`) is a registry
relevant pointer move: the crate name stays canonical, the development URL
does not. Fresh Wayback saves were rate limited on 2026-06-11; the recorded
snapshot is pre existing.
