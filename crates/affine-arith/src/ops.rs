//! Arithmetic on affine forms: negation, addition, subtraction, scaling, and the
//! rigor-critical affine-by-affine multiply.
//!
//! The linear operations combine coefficients over shared noise symbols, which is
//! where affine arithmetic beats interval arithmetic: `x - x` cancels its shared
//! symbols to zero. Every floating-point rounding error those combinations incur,
//! and the bilinear remainder a multiply leaves behind, are bounded outward into
//! fresh noise symbols, so the result always encloses the true value set.

use alloc::vec::Vec;
use core::cmp::Ordering;

use round_float::RoundFloat;

use crate::form::{AffineForm, Term};
use crate::symbol::SymbolSource;

/// A definite, representable approximation of the real sum `a + b`, paired with a
/// nonnegative bound on its error: returns `(z, e)` with `|z - (a + b)| <= e`.
///
/// The true sum lies in `[add_down, add_up]`; taking `z = add_down` and
/// `e = add_up - z` (rounded up) bounds the gap. On a correctly-rounded backend
/// an exactly representable sum gives `e = 0`, so an exact cancellation carries no
/// roundoff.
pub(crate) fn add_err<F: RoundFloat>(a: F, b: F) -> (F, F) {
    let lo = a.add_down(b);
    let hi = a.add_up(b);
    (lo, hi.sub_up(lo))
}

/// As [`add_err`], for the real product `a * b`.
pub(crate) fn mul_err<F: RoundFloat>(a: F, b: F) -> (F, F) {
    let lo = a.mul_down(b);
    let hi = a.mul_up(b);
    (lo, hi.sub_up(lo))
}

/// Append a fresh noise symbol carrying accumulated independent uncertainty
/// (rounding error, or a nonlinear remainder), unless it is zero. A fresh id
/// exceeds every id already issued by the source, so the new term stays in
/// canonical order at the end of the vector.
pub(crate) fn push_fresh<F: RoundFloat>(
    terms: &mut Vec<Term<F>>,
    coeff: F,
    src: &mut SymbolSource<'_>,
) {
    if !coeff.is_zero() {
        terms.push(Term::new(src.fresh(), coeff));
    }
}

impl<'id, F: RoundFloat> AffineForm<'id, F> {
    /// Exact negation: flips the sign of the center and every coefficient.
    ///
    /// Negation is exact (a sign flip never rounds), so it introduces no symbol
    /// and preserves every correlation.
    #[must_use]
    pub fn negate(&self) -> Self {
        let terms = self
            .terms()
            .iter()
            .map(|t| Term::new(t.symbol(), t.coeff().negate()))
            .collect();
        Self::from_parts(self.center().negate(), terms)
    }

    /// The sum `self + other`.
    ///
    /// Coefficients on shared symbols add, so correlated uncertainty combines
    /// exactly; symbols in only one operand carry over unchanged. The rounding
    /// error of the center and of each shared-coefficient sum is bounded into one
    /// fresh symbol, so the result encloses the true sum.
    ///
    /// `self`, `other`, and `src` must share one [`SymbolSource`] (see the
    /// crate-level note on sharing a symbol source).
    #[must_use]
    pub fn add(&self, other: &Self, src: &mut SymbolSource<'id>) -> Self {
        let (center, mut error) = add_err(self.center(), other.center());
        let (xs, ys) = (self.terms(), other.terms());
        let mut terms: Vec<Term<F>> = Vec::with_capacity(xs.len() + ys.len() + 1);
        let (mut i, mut j) = (0, 0);
        while i < xs.len() && j < ys.len() {
            let (a, b) = (xs[i], ys[j]);
            match a.symbol().cmp(&b.symbol()) {
                Ordering::Less => {
                    terms.push(a);
                    i += 1;
                }
                Ordering::Greater => {
                    terms.push(b);
                    j += 1;
                }
                Ordering::Equal => {
                    let (z, e) = add_err(a.coeff(), b.coeff());
                    if !z.is_zero() {
                        terms.push(Term::new(a.symbol(), z));
                    }
                    error = error.add_up(e);
                    i += 1;
                    j += 1;
                }
            }
        }
        terms.extend_from_slice(&xs[i..]);
        terms.extend_from_slice(&ys[j..]);
        push_fresh(&mut terms, error, src);
        Self::from_parts(center, terms)
    }

    /// The difference `self - other`, formed as `self + (-other)`.
    ///
    /// Subtracting a form from itself cancels every shared symbol and, on a
    /// correctly-rounded backend, every center and coefficient exactly, so
    /// `x - x` reduces to the exact point zero where interval arithmetic would
    /// report the full doubled width.
    ///
    /// `self`, `other`, and `src` must share one [`SymbolSource`] (see the
    /// crate-level note on sharing a symbol source).
    #[must_use]
    pub fn sub(&self, other: &Self, src: &mut SymbolSource<'id>) -> Self {
        self.add(&other.negate(), src)
    }

    /// The form scaled by a constant `scalar`.
    ///
    /// Each center and coefficient is multiplied by the scalar; the rounding
    /// error is bounded into one fresh symbol.
    ///
    /// `self` and `src` must share one [`SymbolSource`] (see the crate-level note
    /// on sharing a symbol source).
    #[must_use]
    pub fn scale(&self, scalar: F, src: &mut SymbolSource<'id>) -> Self {
        let (center, mut error) = mul_err(self.center(), scalar);
        let mut terms: Vec<Term<F>> = Vec::with_capacity(self.num_terms() + 1);
        for t in self.terms() {
            let (z, e) = mul_err(t.coeff(), scalar);
            if !z.is_zero() {
                terms.push(Term::new(t.symbol(), z));
            }
            error = error.add_up(e);
        }
        push_fresh(&mut terms, error, src);
        Self::from_parts(center, terms)
    }

    /// The product `self * other`: the rigor-critical operation.
    ///
    /// Writing `x̂ = x₀ + Σ xᵢεᵢ` and `ŷ = y₀ + Σ yⱼεⱼ`, the product is
    ///
    /// ```text
    ///     x̂ŷ = x₀y₀ + Σ (x₀yᵢ + y₀xᵢ) εᵢ + (Σ xᵢεᵢ)(Σ yⱼεⱼ).
    /// ```
    ///
    /// The center `x₀y₀` and the linear coefficients are computed with directed
    /// rounding and their errors bounded. The bilinear remainder is not affine; it
    /// is bounded in magnitude by the product of the two radii `(Σ|xᵢ|)(Σ|yⱼ|)`,
    /// rounded up. That bound and the accumulated rounding error are folded into
    /// one fresh symbol, so the result encloses the true product over every joint
    /// assignment of the shared symbols.
    ///
    /// `self`, `other`, and `src` must share one [`SymbolSource`] (see the
    /// crate-level note on sharing a symbol source). Squaring (`x.mul(&x, …)`)
    /// is sound but not tight: it bounds the `εᵢ²` terms over `[−1, 1]` rather
    /// than their true `[0, 1]` range, which a dedicated squaring operation would
    /// exploit.
    #[must_use]
    pub fn mul(&self, other: &Self, src: &mut SymbolSource<'id>) -> Self {
        let (x0, y0) = (self.center(), other.center());
        let (center, mut error) = mul_err(x0, y0);
        let (xs, ys) = (self.terms(), other.terms());
        let mut terms: Vec<Term<F>> = Vec::with_capacity(xs.len() + ys.len() + 1);
        let (mut i, mut j) = (0, 0);
        while i < xs.len() && j < ys.len() {
            let (a, b) = (xs[i], ys[j]);
            match a.symbol().cmp(&b.symbol()) {
                Ordering::Less => {
                    push_linear(&mut terms, &mut error, a.symbol(), mul_err(y0, a.coeff()));
                    i += 1;
                }
                Ordering::Greater => {
                    push_linear(&mut terms, &mut error, b.symbol(), mul_err(x0, b.coeff()));
                    j += 1;
                }
                Ordering::Equal => {
                    // Shared symbol: coefficient x₀·b + y₀·a.
                    let (p1, e1) = mul_err(x0, b.coeff());
                    let (p2, e2) = mul_err(y0, a.coeff());
                    let (z, e3) = add_err(p1, p2);
                    if !z.is_zero() {
                        terms.push(Term::new(a.symbol(), z));
                    }
                    error = error.add_up(e1).add_up(e2).add_up(e3);
                    i += 1;
                    j += 1;
                }
            }
        }
        while i < xs.len() {
            push_linear(
                &mut terms,
                &mut error,
                xs[i].symbol(),
                mul_err(y0, xs[i].coeff()),
            );
            i += 1;
        }
        while j < ys.len() {
            push_linear(
                &mut terms,
                &mut error,
                ys[j].symbol(),
                mul_err(x0, ys[j].coeff()),
            );
            j += 1;
        }
        // The bilinear remainder, bounded by the product of the radii, folded
        // together with the accumulated rounding error into one fresh symbol.
        let nonlinear = self.radius().mul_up(other.radius());
        push_fresh(&mut terms, nonlinear.add_up(error), src);
        Self::from_parts(center, terms)
    }
}

/// Push one linear product term `(symbol, z)` and accumulate its error, dropping
/// a coefficient that rounded to zero.
fn push_linear<F: RoundFloat>(
    terms: &mut Vec<Term<F>>,
    error: &mut F,
    symbol: crate::symbol::NoiseSymbol,
    (z, e): (F, F),
) {
    if !z.is_zero() {
        terms.push(Term::new(symbol, z));
    }
    *error = error.add_up(e);
}
