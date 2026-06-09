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
//! # Sharing a symbol source
//!
//! A noise symbol means the same uncertainty everywhere it appears, so two forms
//! can be combined only when their symbols come from a common namespace. Every
//! form that will be combined, and the [`SymbolSource`] passed to the operation,
//! must originate from one source. Combining forms built from different sources
//! is a logic error: their independent symbols would collide on the same id and
//! be treated as correlated, which can silently break the enclosure guarantee
//! (an `x - y` of unrelated forms could collapse toward zero) and violate the
//! ordered-symbol invariant the operations rely on. This precondition is not yet
//! enforced by the type system; honoring it is the caller's responsibility.
//!
//! # Status
//!
//! The construction layer (P1) and the arithmetic (P2): the [`AffineForm`] type,
//! the noise-symbol [`SymbolSource`], the interval round-trip, and the operations
//! [`negate`](AffineForm::negate), [`add`](AffineForm::add),
//! [`sub`](AffineForm::sub), [`scale`](AffineForm::scale), and the rigor-critical
//! [`mul`](AffineForm::mul). The nonlinear elementary functions and the
//! enclosure-and-tightness verification lane arrive in later phases; the
//! workspace decision records carry the plan.
//!
//! # No std
//!
//! The crate is `#![no_std]` with `alloc` (the deviation terms are a sparse
//! vector).

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod form;
mod ops;
pub mod symbol;

pub use form::{AffineForm, Term};
pub use symbol::{NoiseSymbol, SymbolSource};
