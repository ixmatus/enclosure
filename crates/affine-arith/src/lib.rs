//! `affine-arith`: Stolfi affine arithmetic over a directed-rounding float.
//!
//! Scaffold. This crate will provide rigorous self-validated arithmetic in the
//! affine form
//!
//! ```text
//!     x̂ = x₀ + x₁ε₁ + x₂ε₂ + ... + xₙεₙ,    εᵢ ∈ [−1, 1],
//! ```
//!
//! where the shared noise symbols εᵢ track correlations between quantities, so
//! an expression like `x - x` cancels exactly where interval arithmetic loses
//! the correlation and widens. It is generic over [`round_float::RoundFloat`],
//! the same directed-rounding contract `interval-1788` is built on, and an affine
//! form reduces to an [`interval_1788::Interval`] for a guaranteed enclosure. The
//! crate is `no_std` with `alloc` (the sensitivity terms are a sparse vector).
//!
//! The form, its operations (the affine-by-affine multiply is the rigor-critical
//! one), the elementary functions, and the verification lane arrive in phases;
//! see the workspace decision records for the plan. Nothing is exported yet.

#![no_std]
#![forbid(unsafe_code)]
