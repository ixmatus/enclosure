//! Property tests for noise-term condensation, over the f64 fixture.
//!
//! The load-bearing property is pointwise: every value the original form can
//! take under a joint noise assignment lies inside the condensed form's
//! reduced interval (the condensed value set is a superset of the original's).
//! The two REDUCTIONS are deliberately not compared for inclusion: their
//! up-rounded radius sums accumulate in different orders, so they are
//! incomparable at the ulp level.

use affine_arith::{with_source, AffineForm};
use interval_1788::Interval;
use proptest::prelude::*;

/// A concrete value of a form under a joint noise assignment: `x₀ + Σ xᵢ·εᵢ`,
/// with each `εᵢ` drawn from the assignment by symbol id (defaulting to zero).
/// Evaluated in plain f64; the fixture's unconditional outward rounding in the
/// form's construction dominates one nearest-mode evaluation.
fn value_at(form: &AffineForm<'_, f64>, eps: &[f64]) -> f64 {
    let mut v = form.center();
    for term in form.terms() {
        let e = eps
            .get(usize::try_from(term.symbol().id()).unwrap_or(usize::MAX))
            .copied()
            .unwrap_or(0.0);
        v += term.coeff() * e;
    }
    v
}

proptest! {
    /// Condensation preserves enclosure pointwise and width up to rounding,
    /// and lands within the term budget.
    #[test]
    fn condensed_encloses_every_original_value(
        centers in proptest::collection::vec(-1.0e6f64..1.0e6, 2..8),
        radii in proptest::collection::vec(0.0f64..1.0e6, 2..8),
        eps in proptest::collection::vec(-1.0f64..1.0, 64),
        budget in 1usize..6,
    ) {
        let n = centers.len().min(radii.len());
        let (orig_red, cond_red, sample) = with_source(|mut src| {
            // A correlated chain: sum of independent lifts, then a square via
            // mul to mint nonlinear terms too.
            let mut acc = AffineForm::point(0.0);
            for k in 0..n {
                let iv = Interval::new(centers[k] - radii[k], centers[k] + radii[k])
                    .expect("ordered endpoints");
                let x = AffineForm::from_interval(&iv, &mut src).expect("bounded nonempty");
                acc = acc.add(&x, &mut src);
            }
            let squared = acc.mul(&acc, &mut src);
            let condensed = squared.condense(budget, &mut src);
            prop_assert!(condensed.num_terms() <= budget);
            Ok((squared.reduce(), condensed.reduce(), value_at(&squared, &eps)))
        })?;
        // The load-bearing property: the condensed form's value set is a
        // superset of the original's, so its reduction encloses every value the
        // original form can take.
        prop_assert!(
            cond_red.contains(sample),
            "sampled value {sample} escaped condensed enclosure [{}, {}]",
            cond_red.inf(),
            cond_red.sup()
        );
        // Width is preserved up to rounding: condensation trades correlation,
        // not current width.
        let w_orig = orig_red.sup() - orig_red.inf();
        let w_cond = cond_red.sup() - cond_red.inf();
        let tol = 1e-9 * w_orig.max(1.0);
        prop_assert!((w_cond - w_orig).abs() <= tol,
            "condensed width {w_cond} drifted from original {w_orig}");
    }
}
