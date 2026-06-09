//! Property and unit tests for the P2 arithmetic, over the f64 fixture.
//!
//! Soundness (enclosure): the reduction of an operation's result contains every
//! value the true operation can produce. For binary operations over a box the
//! extreme values sit at the corners, so containing the corner results witnesses
//! enclosure of the whole set. Tightness on a correlated expression (`x - x`) is
//! checked directly: the shared symbols cancel, so the result collapses far below
//! the doubled width interval arithmetic would report.
//!
//! Each test does its affine work inside a `with_source` scope and returns the
//! brand-free reduced interval, which the assertions then inspect.

use affine_arith::{with_source, AffineForm, SymbolSource};
use interval_1788::Interval;
use proptest::prelude::*;

/// Build a form enclosing `[lo, hi]` (with `lo <= hi`) from the given source.
fn form_of<'id>(lo: f64, hi: f64, src: &mut SymbolSource<'id>) -> AffineForm<'id, f64> {
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
        let sum = with_source(|mut src| {
            let x = form_of(ax, bx, &mut src);
            let y = form_of(ay, by, &mut src);
            x.add(&y, &mut src).reduce()
        });
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
        let prod = with_source(|mut src| {
            let x = form_of(ax, bx, &mut src);
            let y = form_of(ay, by, &mut src);
            x.mul(&y, &mut src).reduce()
        });
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
        let scaled = with_source(|mut src| form_of(a, b, &mut src).scale(s, &mut src).reduce());
        prop_assert!(scaled.contains(a * s));
        prop_assert!(scaled.contains(b * s));
    }

    #[test]
    fn negate_reflects(a in -1.0e6f64..1.0e6, w in 0.0f64..1.0e6) {
        let b = a + w;
        let neg = with_source(|mut src| form_of(a, b, &mut src).negate().reduce());
        prop_assert!(neg.contains(-a));
        prop_assert!(neg.contains(-b));
    }

    /// The affine win: `x - x` cancels the shared symbols, so the result is far
    /// narrower than the original width (interval arithmetic would give the
    /// doubled width `2(b - a)`), and still encloses zero.
    #[test]
    fn self_subtraction_collapses(a in -1.0e6f64..1.0e6, w in 1.0f64..1.0e6) {
        let b = a + w;
        let diff = with_source(|mut src| {
            let x = form_of(a, b, &mut src);
            x.sub(&x, &mut src).reduce()
        });
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
    let prod = with_source(|mut src| {
        let x = AffineForm::point(3.0_f64);
        let y = AffineForm::point(4.0_f64);
        x.mul(&y, &mut src).reduce()
    });
    assert!(prod.contains(12.0));
}

#[test]
fn negation_is_exact_on_a_point() {
    let neg = AffineForm::point(7.0_f64).negate().reduce();
    assert_eq!(neg.inf(), -7.0);
    assert_eq!(neg.sup(), -7.0);
}
