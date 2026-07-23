---
slug: astro-float-oracle
category: spec-conformance
citation: >-
  stencillogic, astro-float 0.9.5 (pure Rust arbitrary-precision
  floating-point with correct rounding), released 2025-03-16; the newest of
  32 releases as of 2026-07-18, repository dormant since.
canonical-url: https://github.com/stencillogic/astro-float
doi: none
archived-url: http://web.archive.org/web/20260718062708/https://github.com/stencillogic/astro-float
archive-date: 2026-07-18
retrieved: 2026-07-18
license: MIT
vendor-status: pointer-only
rot-risk: single-maintainer
provenance-class: primary
consumers:
  - crates/round-float/Cargo.toml (dev-dependency, exact-arithmetic oracle)
  - crates/round-float/tests/reduction_oracle.rs (the Kulisch accumulator's
    independent cross-check; carries both quirk workarounds)
  - crates/round-float/tests/reduction_conformance.rs
  - crates/round-float/tests/pown_oracle.rs (the RoundPown kernel's exact
    cross-check: correct rounding in the exact range, soundness past it; carries
    the subnormal from_f64 workaround)
verification:
  - crates/round-float/tests/reduction_oracle.rs (the lane this source grounds;
    its workaround comments cite this entry)
  - crates/round-float/tests/pown_oracle.rs (the pown kernel's correct-rounding
    and soundness lane; compares directed results to the exact integer power)
sha256: none
notes: >-
  The pure Rust arbitrary-precision oracle behind the reduction lanes, chosen
  over MPFR per the workspace tooling default (pure Rust dev dependencies over
  C FFI). Two known defects in 0.9.5, found 2026-07-17 during the reduction
  oracle work (bead enc-aer, discovered from enc-k8l), both worked around in
  the consuming lanes and both still present in the newest release, so any
  new consumer must inherit the workarounds: (1) BigFloat::from_f64 mishandles
  subnormal f64 inputs by assuming the normal implicit leading bit; construct
  subnormals by exact power-of-two scaling of a normal value instead. (2) cmp
  misreports the sign when an operand is a zero-adjacent inexact value; route
  comparisons through subtraction plus is_positive/is_negative/is_zero. If
  astro-float ever became a shipped dependency rather than a dev dependency,
  both defects are blockers, and this entry is the record of why.
---

astro-float supplies correctly rounded arbitrary-precision arithmetic in pure
Rust on stable toolchains, which is what the reduction lanes need to certify
the TightF64 Kulisch accumulator against an independent implementation: the
oracle recomputes each reduction at high precision and the lane demands the
accumulator's exact result agree. It is also the named fallback reference for
the elementary oracle should the pinned pfloat revision ever lose a function
(workspace ADR-0007 part 5).

The quirks recorded in the frontmatter notes are load bearing: they shaped the
reduction oracle's construction helpers, and they are exactly the class of
defect (edge-of-domain value handling) that an oracle exists to catch, so a
future consumer trusting astro-float naively near zero or in the subnormal
range would inherit silently wrong verdicts. The repository's dormancy since
the 0.9.5 release (2025-03-16) makes an upstream fix unlikely on any horizon
this workspace plans for; the workarounds are the durable interface.
