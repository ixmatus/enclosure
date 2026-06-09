//! Property and unit tests for the Chebyshev elementary functions, over the f64
//! fixture.
//!
//! Enclosure: the reduction of `f(x̂)` contains the true `f(x)` for every `x` in
//! the form's range. Sampling a grid across the range witnesses it (the
//! functions are smooth, so a fine grid bounds the continuous image). Tightness
//! is checked where it bites: a dedicated square is narrower than the generic
//! self-multiply, which forgets that a square is one-sided.
//!
//! Each test does its affine work inside a `with_source` scope and returns the
//! brand-free reduced interval for the assertions to inspect.

use affine_arith::{with_source, AffineForm, SymbolSource};
use interval_1788::Interval;
use proptest::prelude::*;

/// Build a form enclosing `[lo, hi]` (with `lo <= hi`) from the given source.
fn form_of<'id>(lo: f64, hi: f64, src: &mut SymbolSource<'id>) -> AffineForm<'id, f64> {
    let iv = Interval::new(lo, hi).expect("ordered finite endpoints");
    AffineForm::from_interval(&iv, src).expect("bounded nonempty")
}

proptest! {
    /// The reduced square contains `x²` for every sampled `x` in `[a, b]`.
    #[test]
    fn sqr_encloses_the_square_over_the_range(
        a in -1.0e3f64..1.0e3, w in 0.0f64..1.0e3,
    ) {
        let b = a + w;
        let enc = with_source(|mut src| form_of(a, b, &mut src).sqr(&mut src).reduce());
        for k in 0..=16 {
            let x = a + (b - a) * (f64::from(k) / 16.0);
            prop_assert!(enc.contains(x * x), "sqr lost {}^2 = {} over [{a}, {b}]", x, x * x);
        }
    }

    /// A dedicated square is no wider than the generic self-multiply, and far
    /// narrower when the range straddles zero (where the multiply's bilinear
    /// remainder is loosest).
    #[test]
    fn sqr_is_no_wider_than_self_times_self(a in -1.0e3f64..1.0e3, w in 1.0f64..1.0e3) {
        let b = a + w;
        let (sq, prod) = with_source(|mut src| {
            let x = form_of(a, b, &mut src);
            (x.sqr(&mut src).reduce(), x.mul(&x, &mut src).reduce())
        });
        let sq_w = sq.sup() - sq.inf();
        let prod_w = prod.sup() - prod.inf();
        prop_assert!(sq_w <= prod_w, "sqr width {sq_w} > mul width {prod_w} on [{a}, {b}]");
    }

    /// The reciprocal of a strictly positive range encloses `1/x` everywhere on it.
    #[test]
    fn recip_positive_encloses(a in 1.0e-2f64..1.0e3, w in 0.0f64..1.0e3) {
        let b = a + w;
        let enc = with_source(|mut src| form_of(a, b, &mut src).recip(&mut src).unwrap().reduce());
        for k in 0..=16 {
            let x = a + (b - a) * (f64::from(k) / 16.0);
            prop_assert!(enc.contains(1.0 / x), "recip lost 1/{x} over [{a}, {b}]");
        }
    }

    /// And of a strictly negative range, reached by the exact-negation reflection.
    #[test]
    fn recip_negative_encloses(lo in 1.0e-2f64..1.0e3, w in 0.0f64..1.0e3) {
        let (a, b) = (-(lo + w), -lo); // a <= b < 0
        let enc = with_source(|mut src| form_of(a, b, &mut src).recip(&mut src).unwrap().reduce());
        for k in 0..=16 {
            let x = a + (b - a) * (f64::from(k) / 16.0);
            prop_assert!(enc.contains(1.0 / x), "recip lost 1/{x} over [{a}, {b}]");
        }
    }

    /// The square root of a nonnegative range encloses `√x` everywhere on it,
    /// including the infinite-derivative corner at `a = 0`.
    #[test]
    fn sqrt_encloses(a in 0.0f64..1.0e3, w in 0.0f64..1.0e3) {
        let b = a + w;
        let enc = with_source(|mut src| form_of(a, b, &mut src).sqrt(&mut src).unwrap().reduce());
        for k in 0..=16 {
            let x = a + (b - a) * (f64::from(k) / 16.0);
            prop_assert!(enc.contains(x.sqrt()), "sqrt lost sqrt({x}) over [{a}, {b}]");
        }
    }
}

#[test]
fn recip_straddling_zero_is_none() {
    let is_none = with_source(|mut src| {
        let iv = Interval::new(-1.0_f64, 2.0).unwrap();
        AffineForm::from_interval(&iv, &mut src)
            .unwrap()
            .recip(&mut src)
            .is_none()
    });
    assert!(is_none);
}

#[test]
fn recip_of_a_point_is_the_reciprocal() {
    let enc = with_source(|mut src| AffineForm::point(2.0_f64).recip(&mut src).unwrap().reduce());
    assert!(enc.contains(0.5));
}

#[test]
fn sqrt_of_a_negative_part_is_none() {
    let is_none = with_source(|mut src| {
        let iv = Interval::new(-1.0_f64, 4.0).unwrap();
        AffineForm::from_interval(&iv, &mut src)
            .unwrap()
            .sqrt(&mut src)
            .is_none()
    });
    assert!(is_none);
}

#[test]
fn sqrt_of_the_zero_point_is_zero() {
    let enc = with_source(|mut src| AffineForm::point(0.0_f64).sqrt(&mut src).unwrap().reduce());
    assert!(enc.contains(0.0));
}

#[test]
fn sqr_of_a_point_is_the_square() {
    let enc = with_source(|mut src| AffineForm::point(3.0_f64).sqr(&mut src).reduce());
    assert!(enc.contains(9.0));
}

#[test]
fn sqr_straddling_zero_pins_the_nonnegative_image() {
    // x² over [-2, 3] ranges over [0, 9]; the reduced square contains the image
    // and is far tighter than [-6, 9] the self-multiply would give.
    let (sq, prod) = with_source(|mut src| {
        let iv = Interval::new(-2.0_f64, 3.0).unwrap();
        let x = AffineForm::from_interval(&iv, &mut src).unwrap();
        (x.sqr(&mut src).reduce(), x.mul(&x, &mut src).reduce())
    });
    assert!(sq.contains(0.0) && sq.contains(4.0) && sq.contains(9.0));
    assert!((sq.sup() - sq.inf()) < (prod.sup() - prod.inf()));
}
