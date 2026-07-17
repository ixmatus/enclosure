//! Property and unit tests for the per-arc trigonometric, hyperbolic, and power
//! affine fits, over the f64 fixture.
//!
//! Enclosure: where a fit applies (returns `Some`), its reduction contains the
//! true `f(x)` for every `x` in the form's range; a fine sample witnesses it.
//! Decline: a range spanning an arc boundary (`sin`/`cos`) or zero (`sinh`/`tanh`),
//! or one whose endpoint sits within a pi-enclosure width of an inflection point,
//! returns `None` so the caller falls back to the interval arm. Tightness: an
//! affine `sin` computed once and self-subtracted collapses far below the doubled
//! width interval arithmetic reports. The `sin` fit's decline (fire) rate over
//! moderate ranges is measured, per decision record 0005 consequence (c).

use affine_arith::{with_source, AffineForm, SymbolSource, Term};
use interval_1788::Interval;
use proptest::prelude::*;

/// Build a form enclosing `[lo, hi]` (with `lo <= hi`) from the given source.
fn form_of<'id>(lo: f64, hi: f64, src: &mut SymbolSource<'id>) -> AffineForm<'id, f64> {
    let iv = Interval::new(lo, hi).expect("ordered finite endpoints");
    AffineForm::from_interval(&iv, src).expect("bounded nonempty")
}

/// Assert the reduced fit encloses `f` at a 17-point sweep of `[a, b]`. `f` is the
/// exact `f64` function; the fixture's own directed bounds guarantee `libm`'s
/// value is enclosed, and the sweep checks the affine band did not lose it.
fn assert_encloses(enc: &Interval<f64>, a: f64, b: f64, f: impl Fn(f64) -> f64, name: &str) {
    for k in 0..=16 {
        let x = a + (b - a) * (f64::from(k) / 16.0);
        let fx = f(x);
        assert!(
            enc.contains(fx),
            "{name} lost {name}({x}) = {fx} over [{a}, {b}]: enc = [{}, {}]",
            enc.inf(),
            enc.sup()
        );
    }
}

proptest! {
    /// Where the `sin` fit applies, it encloses `sin` over the range.
    #[test]
    fn sin_encloses_where_it_fits(a in -12.0f64..12.0, w in 0.0f64..3.2) {
        let b = a + w;
        let fit = with_source(|mut src| {
            form_of(a, b, &mut src).sin(&mut src).map(|g| g.reduce())
        });
        if let Some(enc) = fit {
            assert_encloses(&enc, a, b, f64::sin, "sin");
        }
    }

    /// Where the `cos` fit applies, it encloses `cos` over the range.
    #[test]
    fn cos_encloses_where_it_fits(a in -12.0f64..12.0, w in 0.0f64..3.2) {
        let b = a + w;
        let fit = with_source(|mut src| {
            form_of(a, b, &mut src).cos(&mut src).map(|g| g.reduce())
        });
        if let Some(enc) = fit {
            assert_encloses(&enc, a, b, f64::cos, "cos");
        }
    }

    /// `cosh` fits every finite range (convex everywhere) and encloses it.
    #[test]
    fn cosh_encloses(a in -6.0f64..6.0, w in 0.0f64..6.0) {
        let b = a + w;
        let fit = with_source(|mut src| {
            form_of(a, b, &mut src).cosh(&mut src).map(|g| g.reduce())
        });
        // cosh is total on a finite range; the only decline is overflow, off this range.
        let enc = fit.expect("cosh fits a finite non-overflowing range");
        assert_encloses(&enc, a, b, f64::cosh, "cosh");
    }

    /// Where the `sinh` fit applies (range not straddling zero), it encloses `sinh`.
    #[test]
    fn sinh_encloses_where_it_fits(a in -6.0f64..6.0, w in 0.0f64..6.0) {
        let b = a + w;
        let fit = with_source(|mut src| {
            form_of(a, b, &mut src).sinh(&mut src).map(|g| g.reduce())
        });
        if let Some(enc) = fit {
            assert_encloses(&enc, a, b, f64::sinh, "sinh");
        }
    }

    /// Where the `tanh` fit applies (range not straddling zero), it encloses `tanh`.
    #[test]
    fn tanh_encloses_where_it_fits(a in -6.0f64..6.0, w in 0.0f64..6.0) {
        let b = a + w;
        let fit = with_source(|mut src| {
            form_of(a, b, &mut src).tanh(&mut src).map(|g| g.reduce())
        });
        if let Some(enc) = fit {
            assert_encloses(&enc, a, b, f64::tanh, "tanh");
        }
    }

    /// Where the scalar power fits (positive base), it encloses `x^y`.
    #[test]
    fn pow_scalar_encloses_where_it_fits(
        a in 1.0e-2f64..50.0, w in 0.0f64..50.0, y in -3.0f64..3.0,
    ) {
        let b = a + w;
        let fit = with_source(|mut src| {
            form_of(a, b, &mut src).pow_scalar(y, &mut src).map(|g| g.reduce())
        });
        if let Some(enc) = fit {
            assert_encloses(&enc, a, b, |x| x.powf(y), "pow_scalar");
        }
    }
}

/// A dense deterministic enclosure sweep over single-arc `sin`/`cos` ranges and
/// half-line `sinh`/`tanh` ranges, a belt-and-braces check alongside proptest that
/// no rounding direction in the residual band is wrong.
#[test]
fn dense_enclosure_sweep() {
    // sin/cos: step centers finely across several periods with a range of widths.
    let mut a = -9.0_f64;
    while a < 9.0 {
        for &w in &[0.05, 0.3, 0.9, 1.7, 2.5] {
            let b = a + w;
            let (s, c) = with_source(|mut src| {
                let f = form_of(a, b, &mut src);
                (
                    f.sin(&mut src).map(|g| g.reduce()),
                    f.cos(&mut src).map(|g| g.reduce()),
                )
            });
            if let Some(enc) = s {
                assert_encloses(&enc, a, b, f64::sin, "sin");
            }
            if let Some(enc) = c {
                assert_encloses(&enc, a, b, f64::cos, "cos");
            }
        }
        a += 0.037;
    }
    // hyperbolic: sweep positive and negative half-lines (avoiding the zero straddle
    // for sinh/tanh) plus cosh everywhere.
    for &(a, b) in &[
        (0.0, 2.5),
        (0.2, 4.0),
        (-3.0, -0.1),
        (-5.0, -2.0),
        (1.0, 1.001),
        (-6.0, 6.0),
    ] {
        let (sh, ch, th) = with_source(|mut src| {
            let f = form_of(a, b, &mut src);
            (
                f.sinh(&mut src).map(|g| g.reduce()),
                f.cosh(&mut src).map(|g| g.reduce()),
                f.tanh(&mut src).map(|g| g.reduce()),
            )
        });
        if let Some(enc) = sh {
            assert_encloses(&enc, a, b, f64::sinh, "sinh");
        }
        if let Some(enc) = ch {
            assert_encloses(&enc, a, b, f64::cosh, "cosh");
        }
        if let Some(enc) = th {
            assert_encloses(&enc, a, b, f64::tanh, "tanh");
        }
    }
}

/// Multi-symbol enclosure for `sin` and `cosh`: build a two-input form (plus the
/// build roundoff symbols), keep it inside a single `sin` arc, and check both fits
/// enclose the true value at a sweep of joint noise-symbol assignments, corners
/// included. This exercises `linearize`'s per-coefficient loop the single-symbol
/// tests never reach.
#[test]
#[allow(clippy::cast_precision_loss)] // the xorshift bits fit f64's mantissa exactly
fn multi_symbol_enclosure_sin_and_cosh() {
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let mut unit = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / ((1u64 << 53) as f64) // [0, 1)
    };

    for _ in 0..1500 {
        // Center each input near 1.4 (inside sin's arc [0, pi]) with small radii, so
        // the combined range stays within the single arc where sin fits.
        let (c0, r0) = (1.4 + (unit() - 0.5) * 0.2, unit() * 0.1);
        let (c1, r1) = ((unit() - 0.5) * 0.2, unit() * 0.1);
        let scale1 = 0.2 + unit() * 0.3;

        let (center, coeffs, sin_enc, cosh_enc) = with_source(|mut src| {
            let f0 = form_of(c0 - r0, c0 + r0, &mut src);
            let f1 = form_of(c1 - r1, c1 + r1, &mut src);
            let combined = f0.add(&f1.scale(scale1, &mut src), &mut src);
            let center = combined.center();
            let coeffs: Vec<f64> = combined.terms().iter().map(Term::coeff).collect();
            (
                center,
                coeffs,
                combined.sin(&mut src).map(|g| g.reduce()),
                combined.cosh(&mut src).map(|g| g.reduce()),
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
            if let Some(ref enc) = sin_enc {
                assert!(enc.contains(x.sin()), "multi-symbol sin lost sin({x})");
            }
            if let Some(ref enc) = cosh_enc {
                assert!(enc.contains(x.cosh()), "multi-symbol cosh lost cosh({x})");
            }
        }
    }
}

#[test]
fn sin_spanning_an_arc_boundary_declines() {
    // [3.0, 3.3] straddles the inflection point at pi (~3.14159), where sin'' flips
    // sign, so the single-symbol fit cannot represent it.
    let is_none = with_source(|mut src| form_of(3.0, 3.3, &mut src).sin(&mut src).is_none());
    assert!(is_none, "sin across an arc boundary must decline");
}

#[test]
fn cos_spanning_an_arc_boundary_declines() {
    // [1.4, 1.8] straddles cosine's inflection point at pi/2 (~1.5708).
    let is_none = with_source(|mut src| form_of(1.4, 1.8, &mut src).cos(&mut src).is_none());
    assert!(is_none, "cos across an arc boundary must decline");
}

#[test]
fn sin_within_the_pi_bracket_of_a_boundary_declines() {
    // An endpoint at pi_up (the float just above the true pi) puts the range across
    // the true inflection point pi even though it never reaches the next float:
    // the pi enclosure cannot certify single-arc membership, so the fit declines
    // rather than guess. This is the pi-ambiguity rule (decision record 0005).
    let pi_up = std::f64::consts::PI.next_up();
    let is_none = with_source(|mut src| form_of(1.0, pi_up, &mut src).sin(&mut src).is_none());
    assert!(is_none, "sin reaching into the pi bracket must decline");
}

#[test]
fn sinh_straddling_zero_declines() {
    // sinh'' = sinh changes sign at 0, so a range with 0 in its interior declines.
    let is_none = with_source(|mut src| form_of(-1.0, 2.0, &mut src).sinh(&mut src).is_none());
    assert!(is_none, "sinh straddling zero must decline");
}

#[test]
fn tanh_straddling_zero_declines() {
    // tanh'' = -2 tanh sech^2 changes sign at 0.
    let is_none = with_source(|mut src| form_of(-1.0, 0.5, &mut src).tanh(&mut src).is_none());
    assert!(is_none, "tanh straddling zero must decline");
}

#[test]
fn sinh_on_a_half_line_fits() {
    // A range wholly on one side of zero fits (the counterpoint to the decline).
    let some = with_source(|mut src| form_of(0.5, 2.0, &mut src).sinh(&mut src).is_some());
    assert!(some, "sinh on a positive half-line must fit");
    let some_neg = with_source(|mut src| form_of(-2.0, -0.5, &mut src).sinh(&mut src).is_some());
    assert!(some_neg, "sinh on a negative half-line must fit");
}

#[test]
fn cosh_is_total_on_a_finite_range() {
    let some = with_source(|mut src| form_of(-3.0, 4.0, &mut src).cosh(&mut src).is_some());
    assert!(some, "cosh fits any finite range");
}

#[test]
fn pow_scalar_of_a_nonpositive_base_declines() {
    // ln of a range reaching zero or below has no real value, so the composition
    // declines at its first stage.
    let is_none = with_source(|mut src| {
        form_of(-1.0, 2.0, &mut src)
            .pow_scalar(2.0, &mut src)
            .is_none()
    });
    assert!(is_none, "pow_scalar of a non-positive base must decline");
}

#[test]
fn pow_scalar_matches_a_known_value() {
    // 4^1.5 = 8; the fit over a tight positive range encloses it.
    let enc = with_source(|mut src| {
        form_of(3.9, 4.1, &mut src)
            .pow_scalar(1.5, &mut src)
            .unwrap()
            .reduce()
    });
    assert!(
        enc.contains(8.0),
        "pow_scalar lost 4^1.5 = 8: got [{}, {}]",
        enc.inf(),
        enc.sup()
    );
}

#[test]
fn affine_sin_collapses_under_self_subtraction() {
    // The structural tightness win: an affine sin computed once and subtracted from
    // itself cancels every shared symbol (including the fresh residual symbol) to a
    // point, where interval arithmetic, having forgotten the correlation, reports
    // the doubled width of the sin image.
    let (a, b) = (1.0, 1.3); // inside sin's arc [0, pi]
    let affine_width = with_source(|mut src| {
        let x = form_of(a, b, &mut src);
        let sx = x.sin(&mut src).expect("single-arc sin fits");
        let collapsed = sx.sub(&sx, &mut src).reduce();
        collapsed.sup() - collapsed.inf()
    });
    let image = Interval::new(a, b).unwrap().sin();
    let interval_self_sub_width = 2.0 * (image.sup() - image.inf());

    // The interval expression has real width (sin varies over [1.0, 1.3]); the
    // affine collapse is a handful of ulps. Demand at least a 1000x gap.
    assert!(
        interval_self_sub_width > 0.05,
        "interval sin self-subtraction should have real width, got {interval_self_sub_width}"
    );
    assert!(
        affine_width * 1000.0 < interval_self_sub_width,
        "affine sin self-subtraction ({affine_width}) not far narrower than interval \
         ({interval_self_sub_width})"
    );
}

/// The `sin` fit decline (fire) rate over moderate ranges, per decision record
/// 0005 consequence (c): the affine trig surface may be nearly dormant because
/// most real ranges span an arc boundary. Width uniform in `[0.1, 3]`, center
/// uniform across several periods so the phase is effectively uniform mod pi.
#[test]
fn sin_fit_fire_rate_over_moderate_ranges() {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut unit = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        #[allow(clippy::cast_precision_loss)] // the low 53 bits fit f64 exactly
        let bits = (state >> 11) as f64 / ((1u64 << 53) as f64);
        bits // [0, 1)
    };

    let trials = 200_000_u32;
    let mut declines = 0_u32;
    for _ in 0..trials {
        let center = (unit() - 0.5) * 40.0; // roughly [-20, 20], several periods
        let w = 0.1 + unit() * 2.9; // width in [0.1, 3]
        let a = center - w / 2.0;
        let b = center + w / 2.0;
        let fit = with_source(|mut src| form_of(a, b, &mut src).sin(&mut src).is_some());
        if !fit {
            declines += 1;
        }
    }
    let rate = f64::from(declines) / f64::from(trials);
    eprintln!("sin fit fire (decline) rate over width~U[0.1,3]: {declines}/{trials} = {rate:.4}");
    // The measurement is the deliverable; a loose sanity band guards against a
    // structural regression (all-decline or never-decline would signal a bug).
    assert!(
        (0.2..0.8).contains(&rate),
        "sin fire rate {rate} outside the plausible band; a structural change?"
    );
}
