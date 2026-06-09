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

use round_float::{RoundFloat, RoundTranscendental};

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

    /// The reciprocal `1/x̂`, by the Chebyshev affine approximation.
    ///
    /// Returns `None` when the range is not sign stable (`0 ∈ [a, b]`, where the
    /// true reciprocal is unbounded and no finite affine form can enclose it) or
    /// when an endpoint, or the slope, is not finite (a range pressed against
    /// zero whose reciprocal overflows). Unlike interval arithmetic, which can
    /// answer such a divisor with an unbounded interval, an affine form is
    /// bounded by construction, so the only sound answer is to decline.
    ///
    /// A negative range is handled by reflection: `1/x = −1/(−x)`, and negation
    /// is exact, so the positive-range core does the work and the result is
    /// negated back. (The reflection is sound for the symmetric Chebyshev fit; it
    /// is the min-range slope choice, not this one, that the reflection would
    /// trip up.)
    #[must_use]
    pub fn recip(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        if a > F::ZERO {
            self.recip_positive(src)
        } else if b < F::ZERO {
            self.negate().recip_positive(src).map(|g| g.negate())
        } else {
            // Zero is in the range: 1/x is unbounded, not a finite affine form.
            None
        }
    }

    /// The reciprocal on a strictly positive range. The residual `1/x − α·x` is
    /// convex there, so its maximum is the larger endpoint value and its minimum
    /// is the parabola-like global minimum `2·√(−α)` at the interior tangent
    /// point `√(ab)` (a sound lower bound over the range whatever rounding did to
    /// `α`). The secant slope is `−1/(ab)`.
    fn recip_positive(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        let ab = a.mul_down(b);
        if !ab.is_finite() || ab <= F::ZERO {
            return None;
        }
        let alpha = F::ONE.div_up(ab).negate();
        if !alpha.is_finite() {
            return None;
        }
        // Maximum of the convex residual at an endpoint, rounded up.
        let r_hi = F::ONE
            .div_up(a)
            .sub_up(alpha.mul_down(a))
            .rmax(F::ONE.div_up(b).sub_up(alpha.mul_down(b)));
        // Global minimum 2·√(−α), rounded down via `s + s` (no inexact constant 2).
        let s = alpha.negate().sqrt_down();
        let r_lo = s.add_down(s);
        if !r_lo.is_finite() || !r_hi.is_finite() {
            return None;
        }
        Some(self.linearize(alpha, r_lo, r_hi, src))
    }

    /// The square root `√x̂`, by the Chebyshev affine approximation.
    ///
    /// Returns `None` unless the whole range is in the domain (`a ≥ 0`): a noise
    /// symbol's range cannot be clamped to the nonnegative part the way interval
    /// arithmetic restricts a domain, so a range dipping below zero has no sound
    /// affine image. A range collapsed to `{0}` returns the point `0`.
    ///
    /// `√x` is concave on the positive reals, so the residual `√x − α·x` has its
    /// minimum at an endpoint and its maximum `1/(4α)` at the interior tangent
    /// point (a sound upper bound over the range). The secant slope is
    /// `1/(√a + √b)`.
    #[must_use]
    pub fn sqrt(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() || a < F::ZERO {
            return None;
        }
        if b.is_zero() {
            // The range is {0}; √0 = 0, spending no symbol.
            return Some(Self::point(F::ZERO));
        }
        // a >= 0, b > 0.
        let den = a.sqrt_down().add_down(b.sqrt_down());
        if !den.is_finite() || den <= F::ZERO {
            return None;
        }
        let alpha = F::ONE.div_up(den);
        if !alpha.is_finite() {
            return None;
        }
        // Maximum of the concave residual, the interior value 1/(4α), rounded up:
        // divide by a lower bound of 4α (built with add_down) so the quotient does
        // not understate the bound.
        let four_alpha = {
            let t = alpha.add_down(alpha);
            t.add_down(t)
        };
        let r_hi = F::ONE.div_up(four_alpha);
        // Minimum of the concave residual at an endpoint, rounded down.
        let r_lo = a
            .sqrt_down()
            .sub_down(alpha.mul_up(a))
            .rmin(b.sqrt_down().sub_down(alpha.mul_up(b)));
        if !r_lo.is_finite() || !r_hi.is_finite() {
            return None;
        }
        Some(self.linearize(alpha, r_lo, r_hi, src))
    }
}

/// The transcendental elementary functions, available only when the backend
/// provides [`RoundTranscendental`]. `recip`/`sqrt`/`sqr` need only the core
/// [`RoundFloat`]; `exp`/`ln` additionally evaluate the function (and, at the
/// interior tangent point, its inverse) with directed rounding, so they live on
/// this stricter bound. A backend without sound transcendentals still gets the
/// algebraic surface above.
impl<'id, F: RoundFloat + RoundTranscendental> AffineForm<'id, F> {
    /// The exponential `exp(x̂)`, by the Chebyshev affine approximation.
    ///
    /// `eˣ` is convex everywhere, so the residual `eˣ − α·x` is convex: its
    /// maximum is the larger endpoint value and its minimum is the global value
    /// `α·(1 − ln α)` at the interior tangent point `ln α` (a sound lower bound
    /// over the range whatever rounding did to `α`). The secant slope is
    /// `(eᵇ − eᵃ)/(b − a)`.
    ///
    /// Returns `None` when an endpoint is not finite or when `eᵇ` overflows: the
    /// true result then exceeds the representable range, which a bounded affine
    /// form cannot enclose. A range collapsed to a point fits the constant `eᶜ`.
    #[must_use]
    pub fn exp(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        let eb_hi = b.exp_up();
        if !eb_hi.is_finite() {
            // Overflow: the result is not a bounded affine form.
            return None;
        }
        let width = b.sub_down(a);
        if width <= F::ZERO {
            // Degenerate range {c}: the constant fit eᶜ, slope zero.
            return Some(self.linearize(F::ZERO, a.exp_down(), eb_hi, src));
        }
        // Secant slope α = (eᵇ − eᵃ)/(b − a) > 0.
        let alpha = eb_hi.sub_up(a.exp_down()).div_up(width);
        if !alpha.is_finite() || alpha <= F::ZERO {
            return None;
        }
        // Global minimum of the convex residual, α·(1 − ln α) = α − α·ln α.
        let r_lo = alpha.sub_down(alpha.mul_up(alpha.ln_up()));
        // Maximum of the convex residual at an endpoint.
        let r_hi = a
            .exp_up()
            .sub_up(alpha.mul_down(a))
            .rmax(b.exp_up().sub_up(alpha.mul_down(b)));
        if !r_lo.is_finite() || !r_hi.is_finite() {
            return None;
        }
        Some(self.linearize(alpha, r_lo, r_hi, src))
    }

    /// The natural logarithm `ln(x̂)`, by the Chebyshev affine approximation.
    ///
    /// Returns `None` unless the whole range is in the domain (`a > 0`): a noise
    /// symbol's range cannot be clamped to the positive part. `ln x` is concave,
    /// so the residual `ln x − α·x` has its minimum at an endpoint and its maximum
    /// `−(ln α + 1)` at the interior tangent point `1/α` (a sound upper bound over
    /// the range). The secant slope is `(ln b − ln a)/(b − a)`. A range collapsed
    /// to a point fits the constant `ln c`.
    #[must_use]
    pub fn ln(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() || a <= F::ZERO {
            return None;
        }
        let width = b.sub_down(a);
        if width <= F::ZERO {
            // Degenerate range {c}: the constant fit ln c, slope zero.
            return Some(self.linearize(F::ZERO, a.ln_down(), b.ln_up(), src));
        }
        // Secant slope α = (ln b − ln a)/(b − a) > 0.
        let alpha = b.ln_up().sub_up(a.ln_down()).div_up(width);
        if !alpha.is_finite() || alpha <= F::ZERO {
            return None;
        }
        // Global maximum of the concave residual, −(ln α + 1) = −ln α − 1.
        let r_hi = alpha.ln_down().negate().sub_up(F::ONE);
        // Minimum of the concave residual at an endpoint.
        let r_lo = a
            .ln_down()
            .sub_down(alpha.mul_up(a))
            .rmin(b.ln_down().sub_down(alpha.mul_up(b)));
        if !r_lo.is_finite() || !r_hi.is_finite() {
            return None;
        }
        Some(self.linearize(alpha, r_lo, r_hi, src))
    }
}
