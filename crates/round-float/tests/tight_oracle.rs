//! `TightF64` against an independent integer reference oracle.
//!
//! The oracle in [`common`] rounds each operation directionally using exact
//! big-integer arithmetic that shares no code with the error-free-transform and
//! FMA paths under test, so agreement is real evidence, not a tautology. The
//! zones exercise exactly the regions where the mainline derivation is most at
//! risk: dense subnormal inputs through the rescaling path, binade boundaries
//! with asymmetric neighbor gaps, and the overflow shoulder.

// Exact bit-for-bit equality is the point here: the oracle and the backend must
// agree on the exact float, and exact results must collapse. `float_cmp` would
// flag every such comparison, so it is allowed in this test crate only.
#![allow(clippy::float_cmp)]

mod common;

use common::{
    dyad, near_pow2, oracle_add, oracle_div, oracle_mul, oracle_sqrt, overflow_shoulder,
    subnormal_ish, wide_finite,
};
use proptest::prelude::*;
use round_float::{RoundFloat, TightF64};

/// Value-level equality that treats `+0` and `-0` as equal (the documented
/// zero-sign divergence) and `NaN` as equal to `NaN`.
fn same(x: f64, y: f64) -> bool {
    x == y || (x.is_nan() && y.is_nan())
}

fn hx(x: f64) -> String {
    format!("{:#018x}", x.to_bits())
}

fn check_add(a: f64, b: f64) {
    let (od, ou) = oracle_add(a, b);
    let td = TightF64(a).add_down(TightF64(b)).0;
    let tu = TightF64(a).add_up(TightF64(b)).0;
    assert!(
        same(td, od),
        "add_down a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(td),
        hx(od)
    );
    assert!(
        same(tu, ou),
        "add_up a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(tu),
        hx(ou)
    );
}

fn check_sub(a: f64, b: f64) {
    // sub is add of the exact negation; the add oracle is the reference.
    let (od, ou) = oracle_add(a, -b);
    let td = TightF64(a).sub_down(TightF64(b)).0;
    let tu = TightF64(a).sub_up(TightF64(b)).0;
    assert!(
        same(td, od),
        "sub_down a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(td),
        hx(od)
    );
    assert!(
        same(tu, ou),
        "sub_up a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(tu),
        hx(ou)
    );
}

fn check_mul(a: f64, b: f64) {
    let (od, ou) = oracle_mul(a, b);
    let td = TightF64(a).mul_down(TightF64(b)).0;
    let tu = TightF64(a).mul_up(TightF64(b)).0;
    assert!(
        same(td, od),
        "mul_down a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(td),
        hx(od)
    );
    assert!(
        same(tu, ou),
        "mul_up a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(tu),
        hx(ou)
    );
}

fn check_div(a: f64, b: f64) {
    if b == 0.0 {
        return;
    }
    let (od, ou) = oracle_div(a, b);
    let td = TightF64(a).div_down(TightF64(b)).0;
    let tu = TightF64(a).div_up(TightF64(b)).0;
    assert!(
        same(td, od),
        "div_down a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(td),
        hx(od)
    );
    assert!(
        same(tu, ou),
        "div_up a={} b={} tight={} oracle={}",
        hx(a),
        hx(b),
        hx(tu),
        hx(ou)
    );
}

fn check_sqrt(a: f64) {
    let a = a.abs();
    let (od, ou) = oracle_sqrt(a);
    let td = TightF64(a).sqrt_down().0;
    let tu = TightF64(a).sqrt_up().0;
    assert!(
        same(td, od),
        "sqrt_down a={} tight={} oracle={}",
        hx(a),
        hx(td),
        hx(od)
    );
    assert!(
        same(tu, ou),
        "sqrt_up a={} tight={} oracle={}",
        hx(a),
        hx(tu),
        hx(ou)
    );
}

// ---------------------------------------------------------------------------
// The oracle validates itself first: correct directed rounding is a necessary
// property of the reference, checked before it is trusted to judge TightF64.
// ---------------------------------------------------------------------------

#[test]
fn oracle_exact_cases_collapse() {
    assert_eq!(oracle_add(1.0, 2.0), (3.0, 3.0));
    assert_eq!(oracle_add(0.5, 0.25), (0.75, 0.75));
    assert_eq!(oracle_mul(3.0, 4.0), (12.0, 12.0));
    assert_eq!(oracle_div(1.0, 4.0), (0.25, 0.25));
    assert_eq!(oracle_sqrt(9.0), (3.0, 3.0));
    assert_eq!(oracle_sqrt(0.0), (0.0, 0.0));
}

#[test]
// The chosen values have significands well within 53 bits, so `m as f64` is
// exact; the cast-precision lint does not apply to these constructed cases.
#[allow(clippy::cast_precision_loss)]
fn dyad_roundtrip_sanity() {
    // The decomposition reproduces the value: guards the oracle's foundation.
    for &x in &[
        1.0,
        0.5,
        3.0,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        1.0e300,
        -2.5,
    ] {
        let d = dyad(x);
        let v = (d.m as f64) * libm::scalbn(1.0, d.e);
        let signed = if d.neg { -v } else { v };
        assert_eq!(signed, x, "dyad roundtrip failed for {}", hx(x));
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    /// The oracle brackets the round-to-nearest result within one ulp: a
    /// necessary condition for it to be correct directed rounding.
    #[test]
    fn oracle_self_consistent(a in wide_finite(), b in wide_finite()) {
        let cases = [oracle_add(a, b), oracle_mul(a, b)];
        let rns = [a + b, a * b];
        for (i, (down, up)) in cases.into_iter().enumerate() {
            prop_assert!(down <= up, "down {} > up {}", hx(down), hx(up));
            let rn = rns[i];
            if rn.is_finite() {
                prop_assert!(down <= rn && rn <= up, "rn {} outside [{}, {}]", hx(rn), hx(down), hx(up));
            }
            prop_assert!(up == down || up == down.next_up(), "gap > 1 ulp: {} {}", hx(down), hx(up));
        }
    }
}

// ---------------------------------------------------------------------------
// TightF64 versus the oracle, zone by zone.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6000))]

    #[test]
    fn tight_matches_oracle_broad(a in wide_finite(), b in wide_finite()) {
        check_add(a, b);
        check_sub(a, b);
        check_mul(a, b);
        check_div(a, b);
        check_sqrt(a);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8000))]

    /// Both operands in the subnormal / smallest-normal zone: the rescaling path.
    #[test]
    fn tight_matches_oracle_both_subnormal(a in subnormal_ish(), b in subnormal_ish()) {
        check_add(a, b);
        check_mul(a, b);
        check_div(a, b);
        check_sqrt(a);
    }

    /// One subnormal operand against a broad one: mixed-scale products and
    /// quotients that underflow.
    #[test]
    fn tight_matches_oracle_one_subnormal(a in subnormal_ish(), b in wide_finite()) {
        check_add(a, b);
        check_mul(a, b);
        check_mul(b, a);
        check_div(a, b);
        check_div(b, a);
    }

    /// Binade boundaries: neighbor gaps are asymmetric across a power of two.
    #[test]
    fn tight_matches_oracle_binade(a in near_pow2(), b in near_pow2()) {
        check_add(a, b);
        check_mul(a, b);
        check_div(a, b);
        check_sqrt(a);
    }

    /// The overflow shoulder just below MAX.
    #[test]
    fn tight_matches_oracle_overflow(a in overflow_shoulder(), b in overflow_shoulder()) {
        check_add(a, b);
        check_mul(a, b);
        check_div(a, b);
    }
}

// ---------------------------------------------------------------------------
// Hard-case unit vectors (round-float ADR-0002 obligation 4).
// ---------------------------------------------------------------------------

#[test]
fn hard_case_signed_zero() {
    // Exact zero results: down and up both numerically zero.
    let d = TightF64(1.0).add_down(TightF64(-1.0)).0;
    let u = TightF64(1.0).add_up(TightF64(-1.0)).0;
    assert!(d == 0.0 && u == 0.0);
    let d = TightF64(0.0).mul_down(TightF64(-3.0)).0;
    let u = TightF64(0.0).mul_up(TightF64(-3.0)).0;
    assert!(d == 0.0 && u == 0.0);
}

#[test]
fn hard_case_overflow_max_plus_max() {
    assert_eq!(TightF64(f64::MAX).add_down(TightF64(f64::MAX)).0, f64::MAX);
    assert_eq!(
        TightF64(f64::MAX).add_up(TightF64(f64::MAX)).0,
        f64::INFINITY
    );
    // Mirror below.
    assert_eq!(
        TightF64(-f64::MAX).add_down(TightF64(-f64::MAX)).0,
        f64::NEG_INFINITY
    );
    assert_eq!(TightF64(-f64::MAX).add_up(TightF64(-f64::MAX)).0, -f64::MAX);
}

#[test]
fn hard_case_min_positive_product_and_quotient() {
    let mp = f64::MIN_POSITIVE;
    check_mul(mp, mp);
    check_mul(mp, 0.5);
    check_div(mp, 8.0);
    check_div(mp, 3.0);
    let sub = f64::from_bits(1); // smallest positive subnormal
    check_mul(sub, 0.5);
    check_div(sub, 2.0);
    check_add(sub, sub);
}

#[test]
fn hard_case_recurring_and_irrational() {
    check_div(1.0, 3.0);
    check_mul(1.0 / 3.0, 3.0);
    check_sqrt(2.0);
    check_sqrt(3.0);
    check_sqrt(0.5);
}

#[test]
fn hard_case_sqrt_subnormal() {
    check_sqrt(f64::from_bits(1)); // smallest positive subnormal
    check_sqrt(f64::MIN_POSITIVE); // smallest normal
    check_sqrt(f64::MIN_POSITIVE.next_down()); // largest subnormal
    check_sqrt(f64::from_bits(0x000f_ffff_ffff_ffff)); // a mid subnormal
}

#[test]
fn hard_case_cancellation() {
    // (1 + 2^-52) - 1 = 2^-52, exact; and other near-total cancellations.
    check_add(1.0f64.next_up(), -1.0);
    check_sub(1.0f64.next_up(), 1.0);
    check_sub(1.0, 1.0f64.next_down());
    check_add(1.0e16, -1.0e16_f64.next_down());
}

#[test]
fn hard_case_exact_division() {
    // Exact quotients collapse to a single value.
    assert_eq!(TightF64(6.0).div_down(TightF64(4.0)).0, 1.5);
    assert_eq!(TightF64(6.0).div_up(TightF64(4.0)).0, 1.5);
    check_div(1.0, 4.0);
    check_div(7.0, 2.0);
}
