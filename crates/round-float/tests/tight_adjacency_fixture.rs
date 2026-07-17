//! The tight backend's adjacency invariant, exactness detection, and envelope
//! against the `f64` fixture (round-float ADR-0002 obligations 1 through 3).
//!
//! These lanes are cheap and catch most derivation mistakes: a bound that is not
//! adjacent to its partner, an exact result that fails to collapse, or a tight
//! bound that escapes the fixture's sound (looser) bound. The oracle lane
//! (`tight_oracle.rs`) is the independent correctness check; this file pins the
//! structural properties every correct derivation must also satisfy.

// Exactness detection asserts exact float equality; `float_cmp` would flag it, so
// it is allowed in this test crate only.
#![allow(clippy::float_cmp)]

mod common;

use common::wide_finite;
use proptest::prelude::*;
use round_float::{RoundFloat, TightF64};

/// The adjacency invariant for one operation's `(down, up)` pair against the
/// round-to-nearest result `rn`: ordered, at most one ulp apart, and bracketing
/// the nearest result. NaN results (non-finite inputs, out-of-domain) are skipped
/// once confirmed consistent.
fn adjacency(down: f64, up: f64, rn: f64) {
    if down.is_nan() || up.is_nan() {
        assert!(rn.is_nan(), "NaN bound but finite rn {rn}");
        return;
    }
    assert!(down <= up, "down {down} > up {up}");
    assert!(
        up == down || up == down.next_up(),
        "bounds not adjacent: down {down} up {up}"
    );
    if !rn.is_nan() {
        // Holds for finite rn and for the +/-inf overflow / pole cases, where the
        // outward bound is the matching infinity.
        assert!(down <= rn && rn <= up, "rn {rn} outside [{down}, {up}]");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20000))]

    #[test]
    fn adjacency_holds(a in wide_finite(), b in wide_finite()) {
        let (ta, tb) = (TightF64(a), TightF64(b));
        adjacency(ta.add_down(tb).0, ta.add_up(tb).0, a + b);
        adjacency(ta.sub_down(tb).0, ta.sub_up(tb).0, a - b);
        adjacency(ta.mul_down(tb).0, ta.mul_up(tb).0, a * b);
        adjacency(ta.div_down(tb).0, ta.div_up(tb).0, a / b);
        let aa = a.abs();
        adjacency(TightF64(aa).sqrt_down().0, TightF64(aa).sqrt_up().0, libm::sqrt(aa));
    }
}

// ---------------------------------------------------------------------------
// Exactness detection: a representable result collapses to down == up.
// ---------------------------------------------------------------------------

fn assert_exact_add(a: f64, b: f64) {
    let d = TightF64(a).add_down(TightF64(b)).0;
    let u = TightF64(a).add_up(TightF64(b)).0;
    assert_eq!(d, u, "add not exact for {a} + {b}");
    assert_eq!(d, a + b);
}

fn assert_exact_mul(a: f64, b: f64) {
    let d = TightF64(a).mul_down(TightF64(b)).0;
    let u = TightF64(a).mul_up(TightF64(b)).0;
    assert_eq!(d, u, "mul not exact for {a} * {b}");
    assert_eq!(d, a * b);
}

fn assert_exact_div(a: f64, b: f64) {
    let d = TightF64(a).div_down(TightF64(b)).0;
    let u = TightF64(a).div_up(TightF64(b)).0;
    assert_eq!(d, u, "div not exact for {a} / {b}");
    assert_eq!(d, a / b);
}

fn assert_exact_sqrt(a: f64, root: f64) {
    let d = TightF64(a).sqrt_down().0;
    let u = TightF64(a).sqrt_up().0;
    assert_eq!(d, u, "sqrt not exact for {a}");
    assert_eq!(d, root);
}

#[test]
fn exact_sums_collapse() {
    assert_exact_add(1.0, 2.0);
    assert_exact_add(0.5, 0.25);
    assert_exact_add(100.0, 0.5);
    assert_exact_add(1.0, -1.0); // cancellation to zero
    assert_exact_add(f64::from_bits(1), f64::from_bits(1)); // subnormal + subnormal
}

#[test]
fn exact_products_collapse() {
    assert_exact_mul(3.0, 4.0);
    assert_exact_mul(0.5, 0.5);
    assert_exact_mul(2.0, 0.25);
    assert_exact_mul(-8.0, 16.0);
    assert_exact_mul(f64::from_bits(1), 2.0); // subnormal * power of two, exact
}

#[test]
fn exact_quotients_collapse() {
    assert_exact_div(1.0, 4.0);
    assert_exact_div(6.0, 2.0);
    assert_exact_div(9.0, 8.0);
    assert_exact_div(-5.0, 2.0);
    assert_exact_div(f64::from_bits(4), 2.0); // subnormal / power of two, exact
}

#[test]
fn exact_roots_collapse() {
    assert_exact_sqrt(9.0, 3.0);
    assert_exact_sqrt(4.0, 2.0);
    assert_exact_sqrt(0.25, 0.5);
    assert_exact_sqrt(1.0, 1.0);
    assert_exact_sqrt(0.0, 0.0);
    // A perfect square whose root is a small normal, exercising the exact case
    // in the rescaling regime.
    let root = libm::scalbn(1.0, -520); // 2^-520
    assert_exact_sqrt(root * root, root);
}

// ---------------------------------------------------------------------------
// Envelope: the tight bounds never escape the fixture's sound (looser) bounds,
// so both enclose the same exact value.
// ---------------------------------------------------------------------------

/// `fixture_down <= tight_down <= tight_up <= fixture_up`, skipping NaN results.
fn envelope(fixture_down: f64, fixture_up: f64, tight_down: f64, tight_up: f64) {
    if fixture_down.is_nan() || fixture_up.is_nan() || tight_down.is_nan() || tight_up.is_nan() {
        return;
    }
    assert!(
        fixture_down <= tight_down,
        "fixture_down {fixture_down} > tight_down {tight_down}"
    );
    assert!(
        tight_down <= tight_up,
        "tight_down {tight_down} > tight_up {tight_up}"
    );
    assert!(
        tight_up <= fixture_up,
        "tight_up {tight_up} > fixture_up {fixture_up}"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20000))]

    /// Compare the raw `f64` fixture methods against `TightF64` on the same
    /// values. The fixture is the `f64` instance of `RoundFloat`; the tight one
    /// is `TightF64`.
    #[test]
    fn tight_inside_fixture(a in wide_finite(), b in wide_finite()) {
        // add
        envelope(
            RoundFloat::add_down(a, b), RoundFloat::add_up(a, b),
            TightF64(a).add_down(TightF64(b)).0, TightF64(a).add_up(TightF64(b)).0,
        );
        // sub
        envelope(
            RoundFloat::sub_down(a, b), RoundFloat::sub_up(a, b),
            TightF64(a).sub_down(TightF64(b)).0, TightF64(a).sub_up(TightF64(b)).0,
        );
        // mul
        envelope(
            RoundFloat::mul_down(a, b), RoundFloat::mul_up(a, b),
            TightF64(a).mul_down(TightF64(b)).0, TightF64(a).mul_up(TightF64(b)).0,
        );
        // div
        envelope(
            RoundFloat::div_down(a, b), RoundFloat::div_up(a, b),
            TightF64(a).div_down(TightF64(b)).0, TightF64(a).div_up(TightF64(b)).0,
        );
        // sqrt
        let aa = a.abs();
        envelope(
            RoundFloat::sqrt_down(aa), RoundFloat::sqrt_up(aa),
            TightF64(aa).sqrt_down().0, TightF64(aa).sqrt_up().0,
        );
    }
}
