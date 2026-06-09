//! Nonlinear elementary functions on affine forms, by Chebyshev (min-max)
//! affine approximation.
//!
//! For a univariate `f` over a form `x̂` whose reduced range is `[a, b]`, the
//! result is the affine form `g = α·x̂ + β + δ·ε_new`: a chosen slope `α` scaling
//! the existing symbols, a center shift `β`, and one fresh noise symbol `δ`
//! bounding everything `g` cannot represent exactly. That fresh symbol carries
//! the approximation error of the line `α·x + β` against `f` over `[a, b]` and
//! the rounding error of forming the center and the scaled coefficients. Every
//! bound rounds outward, so `g` encloses `f` over the whole range, the way the
//! multiply encloses a product.
//!
//! # The Chebyshev construction
//!
//! When `f''` keeps one sign on `[a, b]` (every function here is convex or
//! concave on its domain), the min-max affine approximant takes the secant slope
//! `α = (f(b) − f(a)) / (b − a)`. The residual `r(x) = f(x) − α·x` is then convex
//! (or concave), so its range over `[a, b]` is pinned by the two endpoints and
//! the single interior point where `r' = 0`. `β` centers that residual band and
//! `δ` is its half-width. Rather than trust the ideal closed-form `β` and `δ`
//! against rounding, [`linearize`](AffineForm::linearize)
//! takes an outward enclosure `[r_lo, r_hi]` of the residual range and measures
//! `δ` from the center it actually emits, so any rounding of `α` or `β` is
//! absorbed into the fresh symbol.
//!
//! Sources: de Figueiredo and Stolfi, "Affine Arithmetic: Concepts and
//! Applications" (2004), the univariate Chebyshev approximation; Stolfi and de
//! Figueiredo (1997). The construction is derived from the convexity geometry,
//! not transcribed from an implementation.

use alloc::vec::Vec;

use round_float::RoundFloat;

use crate::form::{AffineForm, Term};
use crate::ops::{add_err, mul_err, push_fresh};
use crate::symbol::SymbolSource;

impl<'id, F: RoundFloat> AffineForm<'id, F> {
    /// Assemble `g = α·x̂ + β + δ·ε_new` from a chosen slope `alpha` and an
    /// outward enclosure `[r_lo, r_hi]` of the residual `f(x) − α·x` over the
    /// form's range.
    ///
    /// `β` is the midpoint of the residual band; `δ` is its half-width measured
    /// from that emitted `β` (as the larger of the two up-rounded one-sided gaps,
    /// so `[β − δ, β + δ] ⊇ [r_lo, r_hi]` whatever `β` rounded to), folded
    /// together with the rounding error of the center `α·x₀ + β` and of each
    /// scaled coefficient `α·xᵢ` into the single fresh symbol. This mirrors the
    /// multiply, which folds its bilinear remainder and accumulated roundoff the
    /// same way.
    fn linearize(&self, alpha: F, r_lo: F, r_hi: F, src: &mut SymbolSource<'id>) -> Self {
        let two = F::ONE.add_up(F::ONE);
        // β anywhere in [r_lo, r_hi] is sound: the up-rounded one-sided gaps below
        // make δ dominate both edges of the band whatever β rounded to.
        let beta = r_lo.add_up(r_hi).div_up(two);
        let delta_approx = r_hi.sub_up(beta).rmax(beta.sub_up(r_lo));

        // center = α·x₀ + β, scaled coefficients = α·xᵢ; every rounding error
        // accumulates outward (add_up) into the fresh symbol alongside δ.
        let (c0, e0) = mul_err(self.center(), alpha);
        let (center, eb) = add_err(c0, beta);
        let mut error = e0.add_up(eb);
        let mut terms: Vec<Term<F>> = Vec::with_capacity(self.num_terms() + 1);
        for t in self.terms() {
            let (z, e) = mul_err(t.coeff(), alpha);
            if !z.is_zero() {
                terms.push(Term::new(t.symbol(), z));
            }
            error = error.add_up(e);
        }
        push_fresh(&mut terms, delta_approx.add_up(error), src);
        Self::from_parts(center, terms)
    }

    /// The square `x̂²`, by the Chebyshev affine approximation.
    ///
    /// `x²` is convex everywhere, so this is total (no domain restriction). Over
    /// `[a, b]` the secant slope is `a + b`; the residual `x² − (a+b)x` is a
    /// convex parabola whose maximum sits at the endpoints (value `−ab`) and
    /// whose minimum `−(a+b)²/4` sits at the midpoint, giving the half-gap
    /// `(b − a)²/8`.
    ///
    /// This is tighter than `self.mul(self, …)`: the generic multiply bounds each
    /// `εᵢ²` over `[−1, 1]` though a square ranges only over `[0, 1]`, while the
    /// dedicated form exploits the convex structure and the one-sided range of a
    /// square.
    #[must_use]
    pub fn sqr(&self, src: &mut SymbolSource<'id>) -> Self {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        let alpha = a.add_up(b);

        // Residual r(x) = x² − α·x over [a, b]: convex, so its maximum is the
        // larger endpoint value and its minimum is the parabola's vertex −α²/4.
        // The vertex is the global minimum of r, so it is a sound lower bound over
        // [a, b] whatever rounding did to α. To over-estimate α²/4 (and so make
        // its negation a sound lower bound), divide by a value just below 4, built
        // with add_down: the fixture's add_up would inflate the divisor above 4
        // and could turn the bound the wrong way.
        let four_below = {
            let two_below = F::ONE.add_down(F::ONE);
            two_below.add_down(two_below)
        };
        let r_lo = alpha.mul_up(alpha).div_up(four_below).negate();
        let r_hi = a
            .mul_up(a)
            .sub_up(alpha.mul_down(a))
            .rmax(b.mul_up(b).sub_up(alpha.mul_down(b)));

        self.linearize(alpha, r_lo, r_hi, src)
    }
}
