//! Differential certification: the f64 fixture's and affine-arith's `exp`/`ln`
//! bounds enclose pfloat-libm's correctly-rounded values, extended to the
//! trigonometric, hyperbolic, and power growth of decision record 0005.
//!
//! pfloat-libm rounds each function toward minus and plus infinity to give the
//! tightest true bracket of the real value (correctly rounded, so the toward-minus
//! result is the largest float below the truth and the toward-plus the smallest
//! above). Checking the fixture's directed bounds bracket that, and the affine
//! enclosure contains it, certifies the soundness that round-float ADR-0001 left
//! resting on musl's accuracy goal, now including the reduction-stress and
//! critical-point grids decision record 0005 part 5 fixes.
//!
//! pfloat-libm exposes correctly-rounded `sin`/`cos`/`tan`, `sinh`/`cosh`/`tanh`,
//! and `rootn`, so those fixtures are checked directly. It has no `pow`, so the
//! real power `x^y` rides an `exp`/`ln` composition reference built from the
//! correctly-rounded `exp_round`/`ln_round`; the fixture `pow` (a distinct `libm`
//! algorithm, tighter than the composition at large exponents) is checked for
//! overlap with that reference, the sound differential sanity available without a
//! correctly-rounded `pow` oracle, while the looser affine `pow_scalar` (which
//! itself rides the fixture `exp`/`ln`) is checked to enclose it outright.

use affine_arith::{with_source, AffineForm};
use interval_1788::Interval;
use pfloat_libm::f64 as oracle;
use pfloat_libm::RoundingMode::{TowardNegative, TowardPositive};
use round_float::{RoundHyperbolic, RoundPow, RoundTranscendental, RoundTrig};

/// The tightest true bracket of `exp(x)` from the correctly-rounded oracle.
fn exp_bracket(x: f64) -> (f64, f64) {
    (
        oracle::exp_round(x, TowardNegative).0,
        oracle::exp_round(x, TowardPositive).0,
    )
}

/// The tightest true bracket of `ln(x)` (requires `x > 0`).
fn ln_bracket(x: f64) -> (f64, f64) {
    (
        oracle::ln_round(x, TowardNegative).0,
        oracle::ln_round(x, TowardPositive).0,
    )
}

#[test]
fn fixture_exp_brackets_correct_rounding() {
    // exp_down(x) <= true exp(x) <= exp_up(x), checked against the oracle bracket
    // across the finite exponent range with an irregular step over many mantissas.
    let mut x = -700.0_f64;
    while x <= 700.0 {
        let (lo, hi) = exp_bracket(x);
        assert!(
            x.exp_down() <= lo,
            "exp_down({x}) = {} exceeds oracle lower bound {lo}",
            x.exp_down()
        );
        assert!(
            x.exp_up() >= hi,
            "exp_up({x}) = {} below oracle upper bound {hi}",
            x.exp_up()
        );
        x += 0.123_456_789;
    }
}

#[test]
fn fixture_ln_brackets_correct_rounding() {
    // ln_down(x) <= true ln(x) <= ln_up(x), swept multiplicatively over many
    // decades of positive x.
    let mut x = 1.0e-300_f64;
    while x <= 1.0e300 {
        let (lo, hi) = ln_bracket(x);
        assert!(
            x.ln_down() <= lo,
            "ln_down({x}) = {} exceeds oracle lower bound {lo}",
            x.ln_down()
        );
        assert!(
            x.ln_up() >= hi,
            "ln_up({x}) = {} below oracle upper bound {hi}",
            x.ln_up()
        );
        x *= 1.13;
    }
}

#[test]
fn affine_exp_encloses_correct_rounding() {
    // The affine exp over a small range must enclose the correctly-rounded exp at
    // every sampled point of the range.
    let mut a = -50.0_f64;
    while a <= 50.0 {
        let b = a + 0.5;
        let enc = with_source(|mut s| {
            let x = AffineForm::from_interval(&Interval::new(a, b).unwrap(), &mut s).unwrap();
            x.exp(&mut s).unwrap().reduce()
        });
        for k in 0..=8 {
            let x = a + (b - a) * (f64::from(k) / 8.0);
            let (lo, hi) = exp_bracket(x);
            assert!(
                enc.inf() <= lo && hi <= enc.sup(),
                "affine exp over [{a}, {b}] lost e^{x}: enc = [{}, {}], oracle = [{lo}, {hi}]",
                enc.inf(),
                enc.sup()
            );
        }
        a += 1.31;
    }
}

#[test]
fn affine_ln_encloses_correct_rounding() {
    let mut a = 1.0e-3_f64;
    while a <= 1.0e6 {
        let b = a * 1.5;
        let enc = with_source(|mut s| {
            let x = AffineForm::from_interval(&Interval::new(a, b).unwrap(), &mut s).unwrap();
            x.ln(&mut s).unwrap().reduce()
        });
        for k in 0..=8 {
            let x = a + (b - a) * (f64::from(k) / 8.0);
            let (lo, hi) = ln_bracket(x);
            assert!(
                enc.inf() <= lo && hi <= enc.sup(),
                "affine ln over [{a}, {b}] lost ln({x}): enc = [{}, {}], oracle = [{lo}, {hi}]",
                enc.inf(),
                enc.sup()
            );
        }
        a *= 2.7;
    }
}

#[test]
fn interval_exp_encloses_correct_rounding() {
    // The interval exp's endpoints must bracket the correctly-rounded image:
    // for [a, b], exp([a, b]).inf() <= exp_down-oracle(x) and the oracle's
    // upper bound stays below exp([a, b]).sup(), at every sampled x.
    let mut a = -50.0_f64;
    while a <= 50.0 {
        let b = a + 0.5;
        let enc = Interval::new(a, b).unwrap().exp();
        for k in 0..=8 {
            let x = a + (b - a) * (f64::from(k) / 8.0);
            let (lo, hi) = exp_bracket(x);
            assert!(
                enc.inf() <= lo && hi <= enc.sup(),
                "interval exp over [{a}, {b}] lost e^{x}: enc = [{}, {}], oracle = [{lo}, {hi}]",
                enc.inf(),
                enc.sup()
            );
        }
        a += 1.31;
    }
}

#[test]
fn interval_ln_encloses_correct_rounding() {
    let mut a = 1.0e-3_f64;
    while a <= 1.0e6 {
        let b = a * 1.5;
        let enc = Interval::new(a, b).unwrap().ln();
        for k in 0..=8 {
            let x = a + (b - a) * (f64::from(k) / 8.0);
            let (lo, hi) = ln_bracket(x);
            assert!(
                enc.inf() <= lo && hi <= enc.sup(),
                "interval ln over [{a}, {b}] lost ln({x}): enc = [{}, {}], oracle = [{lo}, {hi}]",
                enc.inf(),
                enc.sup()
            );
        }
        a *= 2.7;
    }
}

// --- Trigonometric, hyperbolic, and power grids (decision record 0005 part 5) --
//
// Each new function joins the lane with a grid aimed at its specific hazard, not
// just the generic sweep: trig at large arguments (reduction stress well past
// 2^50) and within 1e-9 of critical points and poles; hyperbolic across the
// overflow shoulders; power over the (base, exponent) plane edges and the domain
// boundary rays.

/// The correctly-rounded bracket of `sin(x)` from the oracle.
fn sin_bracket(x: f64) -> (f64, f64) {
    (
        oracle::sin_round(x, TowardNegative).0,
        oracle::sin_round(x, TowardPositive).0,
    )
}

/// The correctly-rounded bracket of `cos(x)`.
fn cos_bracket(x: f64) -> (f64, f64) {
    (
        oracle::cos_round(x, TowardNegative).0,
        oracle::cos_round(x, TowardPositive).0,
    )
}

/// The correctly-rounded bracket of `tan(x)` (away from a pole).
fn tan_bracket(x: f64) -> (f64, f64) {
    (
        oracle::tan_round(x, TowardNegative).0,
        oracle::tan_round(x, TowardPositive).0,
    )
}

/// The correctly-rounded bracket of `sinh(x)`.
fn sinh_bracket(x: f64) -> (f64, f64) {
    (
        oracle::sinh_round(x, TowardNegative).0,
        oracle::sinh_round(x, TowardPositive).0,
    )
}

/// The correctly-rounded bracket of `cosh(x)`.
fn cosh_bracket(x: f64) -> (f64, f64) {
    (
        oracle::cosh_round(x, TowardNegative).0,
        oracle::cosh_round(x, TowardPositive).0,
    )
}

/// The correctly-rounded bracket of `tanh(x)`.
fn tanh_bracket(x: f64) -> (f64, f64) {
    (
        oracle::tanh_round(x, TowardNegative).0,
        oracle::tanh_round(x, TowardPositive).0,
    )
}

/// The correctly-rounded bracket of the `n`-th root `x^(1/n)`.
fn rootn_bracket(x: f64, n: i32) -> (f64, f64) {
    (
        oracle::rootn_round(x, n, TowardNegative).0,
        oracle::rootn_round(x, n, TowardPositive).0,
    )
}

/// A sound bracket of the real power `x^y` for `x > 0`, built from the
/// correctly-rounded `exp`/`ln` composition: `ln x` correctly rounded outward,
/// multiplied by `y` with an outward `f64` step, then `exp` correctly rounded
/// outward. The true `x^y` lies in the returned interval. This is the reference of
/// last resort because pfloat-libm has no `pow`; it is tight for a moderate
/// exponent and loosens as `|y * ln x|` grows (the `exp` amplifies the multiply's
/// rounding), which is why the fixture `pow` is checked for overlap rather than
/// full containment.
fn powf_ref(x: f64, y: f64) -> (f64, f64) {
    let ln_lo = oracle::ln_round(x, TowardNegative).0;
    let ln_hi = oracle::ln_round(x, TowardPositive).0;
    // y * ln x, outward in f64 (the exponent bracket).
    let (p_lo, p_hi) = if y >= 0.0 {
        ((y * ln_lo).next_down(), (y * ln_hi).next_up())
    } else {
        ((y * ln_hi).next_down(), (y * ln_lo).next_up())
    };
    (
        oracle::exp_round(p_lo, TowardNegative).0,
        oracle::exp_round(p_hi, TowardPositive).0,
    )
}

#[test]
fn fixture_trig_brackets_at_large_arguments() {
    // Reduction stress: the fixture's directed sin/cos/tan must still bracket the
    // correctly-rounded value when the argument dwarfs a period, well past 2^50,
    // where a naive reduction loses all significance. Powers of two land exactly on
    // hard reduction cases; the odd multiplier walks many mantissas between them.
    let mut mag = 1.0e15_f64; // ~2^50
    while mag <= 1.0e17 {
        for &x in &[mag, mag + 0.5, mag * 1.000_000_1, -mag, -mag - 0.25] {
            let (slo, shi) = sin_bracket(x);
            assert!(
                x.sin_down() <= slo && shi <= x.sin_up(),
                "sin bracket lost at large x = {x}: fixture [{}, {}], oracle [{slo}, {shi}]",
                x.sin_down(),
                x.sin_up()
            );
            let (clo, chi) = cos_bracket(x);
            assert!(
                x.cos_down() <= clo && chi <= x.cos_up(),
                "cos bracket lost at large x = {x}: fixture [{}, {}], oracle [{clo}, {chi}]",
                x.cos_down(),
                x.cos_up()
            );
            // tan can be near a pole at a large reduced argument; check only that
            // the fixture bracket, when both sides are finite, encloses the oracle.
            let (tlo, thi) = tan_bracket(x);
            if x.tan_down().is_finite()
                && x.tan_up().is_finite()
                && tlo.is_finite()
                && thi.is_finite()
            {
                assert!(
                    x.tan_down() <= tlo && thi <= x.tan_up(),
                    "tan bracket lost at large x = {x}: fixture [{}, {}], oracle [{tlo}, {thi}]",
                    x.tan_down(),
                    x.tan_up()
                );
            }
        }
        mag *= 3.3;
    }
}

#[test]
fn fixture_trig_brackets_near_critical_points_and_poles() {
    // Within 1e-9 of the extrema (sin peaks at pi/2 + k*pi, cos at k*pi) and the
    // zeros, plus tan's poles at pi/2 + k*pi, the ambiguity-widening paths of the
    // interval arm depend on the fixture staying sound; check the raw brackets here.
    use core::f64::consts::PI;
    for k in -20..=20 {
        let base = f64::from(k) * PI;
        let half = base + PI / 2.0;
        for &off in &[-1.0e-9, -1.0e-12, 0.0, 1.0e-12, 1.0e-9] {
            for &x in &[base + off, half + off] {
                let (slo, shi) = sin_bracket(x);
                assert!(
                    x.sin_down() <= slo && shi <= x.sin_up(),
                    "sin bracket lost near critical x = {x}"
                );
                let (clo, chi) = cos_bracket(x);
                assert!(
                    x.cos_down() <= clo && chi <= x.cos_up(),
                    "cos bracket lost near critical x = {x}"
                );
            }
            // tan just off a pole (half = pi/2 + k*pi): large but finite; skip the
            // exact pole where both may be non-finite.
            let x = half + off;
            if off != 0.0 {
                let (tlo, thi) = tan_bracket(x);
                if tlo.is_finite() && thi.is_finite() {
                    assert!(
                        x.tan_down() <= tlo && thi <= x.tan_up(),
                        "tan bracket lost near pole x = {x}"
                    );
                }
            }
        }
    }
}

#[test]
fn affine_sin_cos_enclose_near_critical_points() {
    // Where the affine sin/cos fit applies over a small range that brackets an
    // extremum inside a single arc, its reduction must still enclose the
    // correctly-rounded value at every sampled point.
    use core::f64::consts::PI;
    for k in -6..=6 {
        // A narrow range centered on the extremum pi/2 + k*pi, which sits strictly
        // inside sin's arc [k*pi, (k+1)*pi], so the single-arc fit applies.
        let center = f64::from(k) * PI + PI / 2.0;
        let (a, b) = (center - 0.2, center + 0.2);
        let sin_enc = with_source(|mut s| {
            let x = AffineForm::from_interval(&Interval::new(a, b).unwrap(), &mut s).unwrap();
            x.sin(&mut s).map(|g| g.reduce())
        });
        if let Some(enc) = sin_enc {
            for j in 0..=8 {
                let x = a + (b - a) * (f64::from(j) / 8.0);
                let (lo, hi) = sin_bracket(x);
                assert!(
                    enc.inf() <= lo && hi <= enc.sup(),
                    "affine sin near extremum lost sin({x}) over [{a}, {b}]"
                );
            }
        }
    }
}

#[test]
fn fixture_hyperbolic_brackets_over_shoulders() {
    // Across the finite region up to the overflow shoulders (cosh overflows near
    // 710.5), the fixture's directed sinh/cosh/tanh bracket the correctly-rounded
    // value; the odd step walks many mantissas.
    let mut x = -705.0_f64;
    while x <= 705.0 {
        let (shlo, shhi) = sinh_bracket(x);
        if shlo.is_finite() && shhi.is_finite() {
            assert!(
                x.sinh_down() <= shlo && shhi <= x.sinh_up(),
                "sinh bracket lost at x = {x}: fixture [{}, {}], oracle [{shlo}, {shhi}]",
                x.sinh_down(),
                x.sinh_up()
            );
        }
        let (chlo, chhi) = cosh_bracket(x);
        if chlo.is_finite() && chhi.is_finite() {
            assert!(
                x.cosh_down() <= chlo && chhi <= x.cosh_up(),
                "cosh bracket lost at x = {x}: fixture [{}, {}], oracle [{chlo}, {chhi}]",
                x.cosh_down(),
                x.cosh_up()
            );
        }
        let (tlo, thi) = tanh_bracket(x);
        assert!(
            x.tanh_down() <= tlo && thi <= x.tanh_up(),
            "tanh bracket lost at x = {x}: fixture [{}, {}], oracle [{tlo}, {thi}]",
            x.tanh_down(),
            x.tanh_up()
        );
        x += 7.13;
    }
    // Past the shoulder the fixture reports the finite-lower, infinite-upper edge
    // the exp precedent established (cosh grows without bound).
    assert!(720.0_f64.cosh_down().is_finite());
    assert!(720.0_f64.cosh_up().is_infinite());
    assert!(720.0_f64.sinh_down().is_finite());
    assert!(720.0_f64.sinh_up().is_infinite());
}

#[test]
fn affine_hyperbolic_enclose_on_half_lines() {
    // The affine sinh/cosh/tanh fits over small ranges away from the zero straddle
    // enclose the correctly-rounded value. cosh is total; sinh/tanh fit on a
    // half-line.
    let mut a = -6.0_f64;
    while a <= 5.5 {
        let b = a + 0.5;
        let (sh, ch, th) = with_source(|mut s| {
            let x = AffineForm::from_interval(&Interval::new(a, b).unwrap(), &mut s).unwrap();
            (
                x.sinh(&mut s).map(|g| g.reduce()),
                x.cosh(&mut s).map(|g| g.reduce()),
                x.tanh(&mut s).map(|g| g.reduce()),
            )
        });
        for j in 0..=8 {
            let x = a + (b - a) * (f64::from(j) / 8.0);
            if let Some(ref enc) = sh {
                let (lo, hi) = sinh_bracket(x);
                if lo.is_finite() && hi.is_finite() {
                    assert!(
                        enc.inf() <= lo && hi <= enc.sup(),
                        "affine sinh lost sinh({x})"
                    );
                }
            }
            if let Some(ref enc) = ch {
                let (lo, hi) = cosh_bracket(x);
                if lo.is_finite() && hi.is_finite() {
                    assert!(
                        enc.inf() <= lo && hi <= enc.sup(),
                        "affine cosh lost cosh({x})"
                    );
                }
            }
            if let Some(ref enc) = th {
                let (lo, hi) = tanh_bracket(x);
                assert!(
                    enc.inf() <= lo && hi <= enc.sup(),
                    "affine tanh lost tanh({x})"
                );
            }
        }
        a += 0.83;
    }
}

#[test]
fn fixture_rootn_brackets_correct_rounding() {
    // pfloat-libm has a correctly-rounded rootn, so the fixture's directed rootn is
    // checked for full containment across the (base, order) plane, including the
    // odd-root negative-base ray.
    for &n in &[2_i32, 3, 4, 5, 7, 10] {
        let mut x = 1.0e-6_f64;
        while x <= 1.0e6 {
            let (lo, hi) = rootn_bracket(x, n);
            let un = u32::try_from(n).unwrap();
            assert!(
                x.rootn_down(un) <= lo && hi <= x.rootn_up(un),
                "rootn bracket lost at {x}^(1/{n}): fixture [{}, {}], oracle [{lo}, {hi}]",
                x.rootn_down(un),
                x.rootn_up(un)
            );
            // Odd roots extend to negative bases.
            if n % 2 == 1 {
                let xn = -x;
                let (nlo, nhi) = rootn_bracket(xn, n);
                assert!(
                    xn.rootn_down(un) <= nlo && nhi <= xn.rootn_up(un),
                    "rootn bracket lost at {xn}^(1/{n})"
                );
            }
            x *= 3.9;
        }
    }
}

#[test]
fn fixture_pow_overlaps_composition_reference() {
    // pfloat-libm has no pow; the fixture pow (a libm algorithm, tighter than the
    // exp/ln composition at large exponents) is checked to OVERLAP the composition
    // reference, the sound differential sanity available here: both must contain
    // the true x^y, so their brackets must intersect. A wrong-sided fixture pow
    // fails this. Base rays sweep toward zero, through one, and up; exponents span
    // the sign cases and the domain boundary at zero.
    let bases = [1.0e-3_f64, 0.01, 0.1, 0.5, 0.9, 1.0, 1.5, 2.0, 10.0, 100.0];
    let exps = [-3.0_f64, -1.5, -0.5, 0.0, 0.5, 1.5, 3.0];
    for &x in &bases {
        for &y in &exps {
            let (lo_ref, hi_ref) = powf_ref(x, y);
            let (pd, pu) = (x.pow_down(y), x.pow_up(y));
            assert!(
                pd <= hi_ref && lo_ref <= pu,
                "fixture pow [{pd}, {pu}] disjoint from reference [{lo_ref}, {hi_ref}] at {x}^{y}"
            );
        }
    }
}

#[test]
fn affine_pow_scalar_encloses_composition_reference() {
    // The affine pow_scalar rides the fixture exp/ln (each looser than the
    // correctly-rounded pfloat exp/ln), so its reduction is wider than the
    // composition reference and must enclose it at every sampled base in a small
    // positive range. Exponents stay moderate so nothing overflows.
    let bases = [0.05_f64, 0.5, 1.5, 4.0, 20.0, 80.0];
    let exps = [-2.5_f64, -0.5, 0.5, 2.0];
    for &c in &bases {
        let (a, b) = (c * 0.98, c * 1.02);
        for &y in &exps {
            let enc = with_source(|mut s| {
                let x = AffineForm::from_interval(&Interval::new(a, b).unwrap(), &mut s).unwrap();
                x.pow_scalar(y, &mut s).map(|g| g.reduce())
            });
            if let Some(enc) = enc {
                for j in 0..=8 {
                    let x = a + (b - a) * (f64::from(j) / 8.0);
                    let (lo, hi) = powf_ref(x, y);
                    assert!(
                        enc.inf() <= lo && hi <= enc.sup(),
                        "affine pow_scalar over [{a}, {b}]^{y} lost {x}^{y}: \
                         enc = [{}, {}], ref = [{lo}, {hi}]",
                        enc.inf(),
                        enc.sup()
                    );
                }
            }
        }
    }
}
