//! Noise-term condensation: bounding a form's term count without giving up
//! enclosure.
//!
//! Every operation on affine forms mints at most one fresh symbol, but over a
//! long computation that is still monotone growth, and a consumer that keeps
//! forms alive indefinitely (a calculator stack, an iterative solver) needs the
//! per-value memory bounded. Condensation folds the smallest deviation terms
//! into a single fresh symbol whose coefficient is an upper bound on their
//! combined magnitude.
//!
//! # Soundness
//!
//! Let the form be `x₀ + Σ_K xᵢεᵢ + Σ_T xⱼεⱼ` with `K` the kept terms and `T`
//! the folded tail, and let `c = Σ_T |xⱼ|` rounded up. The condensed form is
//! `x₀ + Σ_K xᵢεᵢ + c·δ` with `δ` fresh. Any value of the original form is a
//! value of the condensed one: fix the kept `εᵢ`, and choose
//! `δ = (Σ_T xⱼεⱼ) / c`, which lies in `[−1, 1]` because `|Σ_T xⱼεⱼ| ≤ Σ_T |xⱼ|
//! ≤ c`. (If `c` is zero the tail was empty and nothing changed.) So the
//! condensed form's value set is a superset of the original's, and enclosure is
//! preserved. This is the standard condensation argument of de Figueiredo &
//! Stolfi, "Affine Arithmetic: Concepts and Applications" (2004), § on reducing
//! the number of noise symbols; derived here for the directed-rounding setting
//! by rounding the tail sum up.
//!
//! # What condensation costs
//!
//! Width is preserved up to rounding: the condensed radius `Σ_K |xᵢ| + c` equals
//! the original `Σ |xᵢ|` on a correctly-rounded backend (and can only exceed it
//! by the rounding of `c`). What is lost is *correlation*: the folded symbols no
//! longer cancel against other forms that share them, so future combinations
//! involving the tail behave as interval arithmetic would. Keeping the
//! largest-magnitude terms preserves the correlations that matter most.

use alloc::vec::Vec;

use round_float::RoundFloat;

use crate::form::{AffineForm, Term};
use crate::ops::push_fresh;
use crate::symbol::SymbolSource;

/// The magnitude `|x|` of a coefficient via the exact sign flip.
#[inline]
fn magnitude<F: RoundFloat>(x: F) -> F {
    if x.is_sign_negative() {
        x.negate()
    } else {
        x
    }
}

impl<'id, F: RoundFloat> AffineForm<'id, F> {
    /// The form with its term count bounded by `budget`, folding the
    /// smallest-magnitude terms into one fresh symbol.
    ///
    /// A form already within the budget is returned unchanged (no fresh
    /// symbol). Otherwise the `budget − 1` terms of largest magnitude keep
    /// their symbols (ties broken toward the older symbol), and the rest are
    /// folded into a fresh symbol whose coefficient is their summed magnitude,
    /// rounded up — sound by the argument in the [module docs](self). A
    /// `budget` of zero is treated as one: a nondegenerate form needs at least
    /// one term to carry its deviation.
    ///
    /// Condensation preserves the enclosure and (up to rounding) the reduced
    /// width; it forgets the folded symbols' correlations, so combinations
    /// through the tail lose their cancellation. Callers balancing memory
    /// against tightness should condense as late and as wide as they can
    /// afford.
    ///
    /// `self` and `src` must share one [`SymbolSource`] (see the crate-level
    /// note on sharing a symbol source).
    #[must_use]
    pub fn condense(&self, budget: usize, src: &mut SymbolSource<'id>) -> Self {
        let budget = budget.max(1);
        if self.num_terms() <= budget {
            return self.clone();
        }

        // Order by descending magnitude, older symbol first on ties: a total,
        // deterministic order. Coefficients are non-NaN by the representation
        // invariant, so `partial_cmp` is total here; the `Equal` fallback only
        // defends a corrupted input, degrading to an arbitrary (still sound)
        // split.
        let mut ordered: Vec<Term<F>> = self.terms().to_vec();
        ordered.sort_unstable_by(|a, b| {
            magnitude(b.coeff())
                .partial_cmp(&magnitude(a.coeff()))
                .unwrap_or(core::cmp::Ordering::Equal)
                .then(a.symbol().cmp(&b.symbol()))
        });

        let keep = budget - 1;
        let mut folded = F::ZERO;
        for term in &ordered[keep..] {
            folded = folded.add_up(magnitude(term.coeff()));
        }

        // Restore the representation invariant on the kept terms (ascending
        // symbol id), then append the fresh symbol, whose id exceeds them all.
        let mut terms: Vec<Term<F>> = ordered;
        terms.truncate(keep);
        terms.sort_unstable_by_key(Term::symbol);
        push_fresh(&mut terms, folded, src);
        Self::from_parts(self.center(), terms)
    }
}

#[cfg(all(test, feature = "fixture"))]
mod tests {
    use crate::{with_source, AffineForm, SymbolSource};
    use interval_1788::Interval;

    /// A form with `n` independent unit terms via repeated interval lifts.
    fn chain<'id>(n: usize, src: &mut SymbolSource<'id>) -> AffineForm<'id, f64> {
        let iv = Interval::new(-1.0_f64, 1.0).unwrap();
        let mut acc = AffineForm::point(0.0);
        for _ in 0..n {
            let x = AffineForm::from_interval(&iv, src).unwrap();
            acc = acc.add(&x, src);
        }
        acc
    }

    #[test]
    fn within_budget_is_identity() {
        with_source(|mut src| {
            let iv = Interval::new(2.0_f64, 4.0).unwrap();
            let x = AffineForm::from_interval(&iv, &mut src).unwrap();
            let before = src.next_id();
            let c = x.condense(8, &mut src);
            assert_eq!(c.num_terms(), x.num_terms());
            assert_eq!(src.next_id(), before, "no fresh symbol spent");
        });
    }

    #[test]
    fn over_budget_folds_to_exactly_budget_terms() {
        with_source(|mut src| {
            let x = chain(10, &mut src);
            assert!(x.num_terms() >= 10);
            let c = x.condense(4, &mut src);
            assert_eq!(c.num_terms(), 4);
        });
    }

    #[test]
    fn budget_zero_is_treated_as_one() {
        with_source(|mut src| {
            let x = chain(5, &mut src);
            let c = x.condense(0, &mut src);
            assert_eq!(c.num_terms(), 1);
        });
    }

    #[test]
    fn condensed_width_matches_original_up_to_rounding() {
        // The reductions of the original and condensed forms are incomparable
        // at the ulp level (their radius sums accumulate in different orders),
        // but the widths agree up to rounding: condensation spends
        // correlation, not width.
        with_source(|mut src| {
            let x = chain(9, &mut src);
            let c = x.condense(3, &mut src);
            let (orig, cond) = (x.reduce(), c.reduce());
            let (w_orig, w_cond) = (orig.sup() - orig.inf(), cond.sup() - cond.inf());
            assert!((w_cond - w_orig).abs() <= 1e-9 * w_orig.max(1.0));
            // And the center's enclosure is common to both.
            assert!(cond.contains(x.center()) && orig.contains(x.center()));
        });
    }

    #[test]
    fn kept_symbols_still_cancel() {
        with_source(|mut src| {
            // x has one dominant symbol; condensing keeps it, so x_c - x still
            // cancels the dominant deviation and stays much tighter than the
            // interval subtraction would be.
            let iv = Interval::new(0.0_f64, 100.0).unwrap();
            let x = AffineForm::from_interval(&iv, &mut src).unwrap();
            let noisy = chain(6, &mut src).scale(1e-6, &mut src);
            let fat = x.add(&noisy, &mut src);
            let c = fat.condense(2, &mut src);
            let diff = c.sub(&x, &mut src).reduce();
            let width = diff.sup() - diff.inf();
            assert!(
                width < 1.0,
                "dominant correlation survived condensation (width {width})"
            );
        });
    }
}
