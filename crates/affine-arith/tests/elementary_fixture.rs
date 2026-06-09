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

use affine_arith::{with_source, AffineForm, SymbolSource, Term};
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

    /// The square root of a strictly positive range encloses `√x` everywhere on
    /// it. The lower bound stays above `1e-2` so the outward-rounded reduced inf
    /// does not dip below zero (which would make `sqrt` decline); the
    /// zero-touching decline is pinned by its own regression test.
    #[test]
    fn sqrt_encloses(a in 1.0e-2f64..1.0e3, w in 0.0f64..1.0e3) {
        let b = a + w;
        let enc = with_source(|mut src| form_of(a, b, &mut src).sqrt(&mut src).unwrap().reduce());
        for k in 0..=16 {
            let x = a + (b - a) * (f64::from(k) / 16.0);
            prop_assert!(enc.contains(x.sqrt()), "sqrt lost sqrt({x}) over [{a}, {b}]");
        }
    }

    /// The exponential of a finite range encloses `eˣ` everywhere on it.
    #[test]
    fn exp_encloses(a in -5.0f64..5.0, w in 0.0f64..5.0) {
        let b = a + w;
        let enc = with_source(|mut src| form_of(a, b, &mut src).exp(&mut src).unwrap().reduce());
        for k in 0..=16 {
            let x = a + (b - a) * (f64::from(k) / 16.0);
            prop_assert!(enc.contains(x.exp()), "exp lost e^{x} over [{a}, {b}]");
        }
    }

    /// The logarithm of a strictly positive range encloses `ln x` everywhere on it.
    #[test]
    fn ln_encloses(a in 1.0e-2f64..1.0e3, w in 0.0f64..1.0e3) {
        let b = a + w;
        let enc = with_source(|mut src| form_of(a, b, &mut src).ln(&mut src).unwrap().reduce());
        for k in 0..=16 {
            let x = a + (b - a) * (f64::from(k) / 16.0);
            prop_assert!(enc.contains(x.ln()), "ln lost ln({x}) over [{a}, {b}]");
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
fn ln_of_a_nonpositive_range_is_none() {
    let is_none = with_source(|mut src| {
        let iv = Interval::new(-1.0_f64, 2.0).unwrap();
        AffineForm::from_interval(&iv, &mut src)
            .unwrap()
            .ln(&mut src)
            .is_none()
    });
    assert!(is_none);
}

#[test]
fn exp_of_an_overflowing_range_is_none() {
    let is_none = with_source(|mut src| {
        let iv = Interval::new(700.0_f64, 800.0).unwrap();
        AffineForm::from_interval(&iv, &mut src)
            .unwrap()
            .exp(&mut src)
            .is_none()
    });
    assert!(is_none);
}

#[test]
fn exp_of_a_point_is_the_exponential() {
    let enc = with_source(|mut src| AffineForm::point(1.0_f64).exp(&mut src).unwrap().reduce());
    assert!(enc.contains(std::f64::consts::E));
}

#[test]
fn ln_of_a_point_is_the_logarithm() {
    let enc = with_source(|mut src| {
        AffineForm::point(std::f64::consts::E)
            .ln(&mut src)
            .unwrap()
            .reduce()
    });
    assert!(enc.contains(1.0));
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

#[test]
fn sqrt_of_a_zero_touching_range_declines() {
    // from_interval([0, b]) rounds its reduced inf a hair below zero, so the whole
    // reduced range is not in the domain and sqrt declines (the conservative
    // choice; see the sqrt doc comment). Only an exact point {0} is special-cased.
    let is_none = with_source(|mut src| {
        let iv = Interval::new(0.0_f64, 4.0).unwrap();
        AffineForm::from_interval(&iv, &mut src)
            .unwrap()
            .sqrt(&mut src)
            .is_none()
    });
    assert!(is_none);
}

#[test]
fn recip_negative_wide_ratio_encloses_at_the_tangent() {
    // A wide negative range, probed at the interior tangent x = -sqrt(ab) where the
    // 2*sqrt(-alpha) lower bound binds hardest, plus the endpoints.
    let (a, b) = (-1.0e6_f64, -1.0);
    let enc = with_source(|mut src| form_of(a, b, &mut src).recip(&mut src).unwrap().reduce());
    let tangent = -(a * b).sqrt(); // in [a, b]
    assert!(
        enc.contains(1.0 / tangent),
        "recip lost 1/{tangent} at the tangent"
    );
    assert!(enc.contains(1.0 / a) && enc.contains(1.0 / b));
}

/// Multi-symbol enclosure: build a two-input form (plus the build roundoff
/// symbols) over a strictly positive range, then check every elementary function
/// encloses the true value at a sweep of joint noise-symbol assignments, corners
/// included. This exercises `linearize`'s per-coefficient loop, which the
/// single-symbol tests above never reach.
#[test]
#[allow(clippy::cast_precision_loss)] // the xorshift bits fit f64's mantissa exactly
fn multi_symbol_enclosure_over_positive_forms() {
    // Deterministic xorshift for reproducible range and assignment generation.
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut unit = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / ((1u64 << 53) as f64) // [0, 1)
    };

    for _ in 0..1500 {
        let (c0, r0) = (1.0 + unit() * 8.0, unit() * 0.4);
        let (c1, r1) = (1.0 + unit() * 8.0, unit() * 0.4);
        let scale1 = 0.2 + unit() * 0.6;

        let (center, coeffs, sqr, recip, sqrt, exp, ln) = with_source(|mut src| {
            let f0 = form_of(c0 - r0, c0 + r0, &mut src);
            let f1 = form_of(c1 - r1, c1 + r1, &mut src);
            let combined = f0.add(&f1.scale(scale1, &mut src), &mut src);
            let center = combined.center();
            let coeffs: Vec<f64> = combined.terms().iter().map(Term::coeff).collect();
            (
                center,
                coeffs,
                combined.sqr(&mut src).reduce(),
                combined.recip(&mut src).unwrap().reduce(),
                combined.sqrt(&mut src).unwrap().reduce(),
                combined.exp(&mut src).unwrap().reduce(),
                combined.ln(&mut src).unwrap().reduce(),
            )
        });

        for iter in 0..18 {
            let mut x = center;
            for &k in &coeffs {
                let eps = match iter {
                    0 => 1.0,
                    1 => -1.0,
                    _ => 2.0 * unit() - 1.0,
                };
                x += k * eps;
            }
            // x is in the form's reduced range, which stays positive, so every
            // function is in-domain and must enclose the true value.
            assert!(sqr.contains(x * x), "sqr lost {x}^2");
            assert!(recip.contains(1.0 / x), "recip lost 1/{x}");
            assert!(sqrt.contains(x.sqrt()), "sqrt lost sqrt({x})");
            assert!(exp.contains(x.exp()), "exp lost e^{x}");
            assert!(ln.contains(x.ln()), "ln lost ln({x})");
        }
    }
}
