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
//! can be combined only when their symbols come from a common namespace. Forms
//! built from different sources have independent symbols that would collide on the
//! same id and be treated as correlated, which would silently break the enclosure
//! guarantee (an `x - y` of unrelated forms could collapse toward zero) and
//! violate the ordered-symbol invariant the operations rely on.
//!
//! This precondition is enforced by the type system. A [`SymbolSource`] exists
//! only inside a [`with_source`] scope and carries an invariant lifetime brand
//! `'id`; every form built from it carries the same brand, and the operations
//! require their operands and the source to share it. Two `with_source` scopes
//! get distinct, incompatible brands, so the compiler refuses to combine forms
//! from different sources. Branded forms cannot leave their scope; reduce a form
//! to an [`interval_1788::Interval`] to return a result.
//!
//! # Status
//!
//! The construction layer (the [`AffineForm`] type, the noise-symbol
//! [`SymbolSource`], the interval round-trip), the arithmetic
//! ([`negate`](AffineForm::negate), [`add`](AffineForm::add),
//! [`sub`](AffineForm::sub), [`scale`](AffineForm::scale), and the rigor-critical
//! [`mul`](AffineForm::mul)), and the nonlinear elementary functions
//! ([`recip`](AffineForm::recip), [`sqrt`](AffineForm::sqrt),
//! [`sqr`](AffineForm::sqr), [`exp`](AffineForm::exp), and
//! [`ln`](AffineForm::ln), by Chebyshev approximation) are in place, together
//! with the per-arc trigonometric, hyperbolic, and power fits
//! ([`sin`](AffineForm::sin), [`cos`](AffineForm::cos), [`sinh`](AffineForm::sinh),
//! [`cosh`](AffineForm::cosh), [`tanh`](AffineForm::tanh), and
//! [`pow_scalar`](AffineForm::pow_scalar); see the `trig` module). The API may
//! break between 0.x releases; the workspace decision records carry the design.
//!
//! # No std
//!
//! The crate is `#![no_std]` with `alloc` (the deviation terms are a sparse
//! vector).

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod condense;
mod elementary;
pub mod form;
mod ops;
pub mod symbol;
mod trig;

pub use form::{AffineForm, Term};
pub use symbol::{with_source, NoiseSymbol, SymbolSource};
