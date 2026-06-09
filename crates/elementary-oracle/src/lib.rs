//! `elementary-oracle`: a nightly differential verification lane for the
//! transcendental bounds in the enclosure family.
//!
//! `round-float`'s f64 fixture bounds `exp` and `ln` by widening `libm` (which is
//! only faithfully rounded) by a relative margin, so their soundness rests on
//! musl's stated accuracy goal rather than on a proof (see round-float decision
//! record 0001). `affine-arith` builds its Chebyshev `exp`/`ln` on those bounds.
//! This crate discharges that assumption empirically: it checks, against
//! [`pfloat-libm`](https://github.com/ixmatus/pfloat) (a pure-Rust
//! *correctly-rounded* libm), that
//!
//! - the fixture's `[exp_down, exp_up]` and `[ln_down, ln_up]` bracket the
//!   correctly-rounded truth, and
//! - the affine `exp`/`ln` enclosures contain it,
//!
//! over a wide grid of inputs. The checks live in the integration tests; this
//! library has no contents of its own.
//!
//! The crate is **not** a workspace member: `pfloat-libm` needs nightly Rust
//! (`feature(generic_const_exprs)`), which the stable enclosure crates cannot
//! depend on. It carries its own nightly toolchain pin and runs in a dedicated
//! nightly CI job, so the shipped crates and their stable consumers never pull
//! `pfloat` into their graph.

#![forbid(unsafe_code)]
