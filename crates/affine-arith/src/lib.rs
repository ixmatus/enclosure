//! `affine-arith`: Stolfi affine arithmetic over a directed-rounding float.
//!
//! An affine form represents a quantity as a central value plus a linear
//! combination of noise symbols,
//!
//! ```text
//!     x̂ = x₀ + x₁ε₁ + x₂ε₂ + ... + xₙεₙ,    εᵢ ∈ [−1, 1],
//! ```
//!
//! where each `εᵢ` is a shared, unknown-but-bounded source of uncertainty. Two
//! forms that reference the same symbol stay correlated, so a combination like
//! `x - x` cancels exactly where interval arithmetic, having forgotten the
//! correlation, would widen. Every form still reduces to a guaranteed interval
//! enclosure ([`AffineForm::reduce`]); affine arithmetic buys tightness on
//! correlated expressions without giving up rigor.
//!
//! The crate is generic over [`round_float::RoundFloat`], the same
//! directed-rounding contract `interval-1788` is built on, and a form is built
//! from and reduces to an [`interval_1788::Interval`].
//!
//! # Status
//!
//! This is the construction layer (P1): the [`AffineForm`] type, the noise-symbol
//! [`SymbolSource`], the interval-to-form construction, and the reduction back to
//! an interval. The arithmetic operations (the rigor-critical affine-by-affine
//! multiply), the nonlinear elementary functions, and the verification lane
//! arrive in later phases; the workspace decision records carry the plan.
//!
//! # No std
//!
//! The crate is `#![no_std]` with `alloc` (the deviation terms are a sparse
//! vector).

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod form;
pub mod symbol;

pub use form::{AffineForm, Term};
pub use symbol::{NoiseSymbol, SymbolSource};
