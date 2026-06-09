//! Differential certification: the f64 fixture's and affine-arith's `exp`/`ln`
//! bounds enclose pfloat-libm's correctly-rounded values.
//!
//! pfloat-libm rounds `exp`/`ln` toward minus and plus infinity to give the
//! tightest true bracket of the real value (correctly rounded, so the toward-minus
//! result is the largest float below the truth and the toward-plus the smallest
//! above). Checking the fixture's `[exp_down, exp_up]` brackets that, and the
//! affine enclosure contains it, certifies the soundness that round-float ADR-0001
//! left resting on musl's accuracy goal.

use affine_arith::{with_source, AffineForm};
use interval_1788::Interval;
use pfloat_libm::f64 as oracle;
use pfloat_libm::RoundingMode::{TowardNegative, TowardPositive};
use round_float::RoundTranscendental;

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
