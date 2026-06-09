//! Property and unit tests for the P2 arithmetic, over the f64 fixture.
//!
//! Soundness (enclosure): the reduction of an operation's result contains every
//! value the true operation can produce. For binary operations over a box the
//! extreme values sit at the corners, so containing the corner results witnesses
//! enclosure of the whole set. Tightness on a correlated expression (`x - x`) is
//! checked directly: the shared symbols cancel, so the result collapses far below
//! the doubled width interval arithmetic would report.

use affine_arith::{AffineForm, SymbolSource};
use interval_1788::Interval;
use proptest::prelude::*;

/// Build a form enclosing `[lo, hi]` (with `lo <= hi`) from the shared source.
fn form_of(lo: f64, hi: f64, src: &mut SymbolSource) -> AffineForm<f64> {
    let iv = Interval::new(lo, hi).expect("ordered finite endpoints");
    AffineForm::from_interval(&iv, src).expect("bounded nonempty")
}

proptest! {
    #[test]
    fn add_encloses_corner_sums(
        ax in -1.0e6f64..1.0e6, wx in 0.0f64..1.0e6,
        ay in -1.0e6f64..1.0e6, wy in 0.0f64..1.0e6,
    ) {
        let (bx, by) = (ax + wx, ay + wy);
        let mut src = SymbolSource::new();
        let x = form_of(ax, bx, &mut src);
        let y = form_of(ay, by, &mut src);
        let sum = x.add(&y, &mut src).reduce();
        for &cx in &[ax, bx] {
            for &cy in &[ay, by] {
                prop_assert!(sum.contains(cx + cy), "sum lost corner {}", cx + cy);
            }
        }
    }

    #[test]
    fn mul_encloses_corner_products(
        ax in -1.0e3f64..1.0e3, wx in 0.0f64..1.0e3,
        ay in -1.0e3f64..1.0e3, wy in 0.0f64..1.0e3,
    ) {
        let (bx, by) = (ax + wx, ay + wy);
        let mut src = SymbolSource::new();
        let x = form_of(ax, bx, &mut src);
        let y = form_of(ay, by, &mut src);
        let prod = x.mul(&y, &mut src).reduce();
        for &cx in &[ax, bx] {
            for &cy in &[ay, by] {
                prop_assert!(prod.contains(cx * cy), "product lost corner {}", cx * cy);
            }
        }
    }

    #[test]
    fn scale_encloses_corners(
        a in -1.0e6f64..1.0e6, w in 0.0f64..1.0e6, s in -1.0e3f64..1.0e3,
    ) {
        let b = a + w;
        let mut src = SymbolSource::new();
        let scaled = form_of(a, b, &mut src).scale(s, &mut src).reduce();
        prop_assert!(scaled.contains(a * s));
        prop_assert!(scaled.contains(b * s));
    }

    #[test]
    fn negate_reflects(a in -1.0e6f64..1.0e6, w in 0.0f64..1.0e6) {
        let b = a + w;
        let mut src = SymbolSource::new();
        let neg = form_of(a, b, &mut src).negate().reduce();
        prop_assert!(neg.contains(-a));
        prop_assert!(neg.contains(-b));
    }

    /// The affine win: `x - x` cancels the shared symbols, so the result is far
    /// narrower than the original width (interval arithmetic would give the
    /// doubled width `2(b - a)`), and still encloses zero.
    #[test]
    fn self_subtraction_collapses(a in -1.0e6f64..1.0e6, w in 1.0f64..1.0e6) {
        let b = a + w;
        let mut src = SymbolSource::new();
        let x = form_of(a, b, &mut src);
        let diff = x.sub(&x, &mut src).reduce();
        prop_assert!(diff.contains(0.0));
        let affine_width = diff.sup() - diff.inf();
        prop_assert!(
            affine_width < (b - a),
            "x - x width {affine_width} not below the original width {}",
            b - a
        );
    }
}

#[test]
fn point_times_point_is_the_product() {
    let mut src = SymbolSource::new();
    let x = AffineForm::<f64>::point(3.0);
    let y = AffineForm::<f64>::point(4.0);
    let prod = x.mul(&y, &mut src).reduce();
    assert!(prod.contains(12.0));
}

#[test]
fn negation_is_exact_on_a_point() {
    let x = AffineForm::<f64>::point(7.0);
    let neg = x.negate().reduce();
    assert_eq!(neg.inf(), -7.0);
    assert_eq!(neg.sup(), -7.0);
}
