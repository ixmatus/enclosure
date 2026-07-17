//! The ITF1788 reduction conformance vectors over `TightF64`.
//!
//! Every assertion of the vendored `libieeep1788_reduction.itl` is transcribed
//! here, including the `dot` cancellation vector that only exact accumulation
//! answers: `(2^52 + 1)(2^52 - 1) + 2^104 * (-1)` is exactly `-1`, while a naive
//! loop rounds `(2^52 + 1)(2^52 - 1) = 2^104 - 1` to `2^104` and then cancels to
//! `0`. The corpus fixes only the nearest variants (the standard's reference
//! surface exposes four modes but tests one), so the directed modes rest on the
//! astro-float and oracle-free lanes in `reduction_oracle.rs`; the corpus values
//! are asserted for `to_nearest` and, where the value is exactly representable,
//! for all four modes as a consistency anchor.
//!
//! Cases derived from ITF1788 `libieeep1788_reduction.itl` (Apache-2.0, original
//! author Marco Nehmeier, ITL port Oliver Heimlich; see
//! `docs/references/vendor/itf1788-framework/`).

// The corpus values are exact, so exact float equality is the intended check.
#![allow(clippy::float_cmp)]

use round_float::{RoundReduction, TightF64};

fn tf(xs: &[f64]) -> Vec<TightF64> {
    xs.iter().copied().map(TightF64).collect()
}

const NAN: f64 = f64::NAN;
const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

/// `+0` and `-0` compare equal here; `NaN` equals `NaN`. The conformance values
/// are all exactly representable, so bit exactness is not needed at this layer.
fn same(x: f64, y: f64) -> bool {
    (x.is_nan() && y.is_nan()) || x == y
}

// --- sum (minimal_sum_test) -------------------------------------------------

#[test]
fn sum_nearest_vectors() {
    assert!(same(TightF64::sum_to_nearest(&tf(&[1.0, 2.0, 3.0])).0, 6.0));
    assert!(same(
        TightF64::sum_to_nearest(&tf(&[1.0, 2.0, NAN, 3.0])).0,
        NAN
    ));
    assert!(same(
        TightF64::sum_to_nearest(&tf(&[1.0, NEG_INF, 2.0, INF, 3.0])).0,
        NAN
    ));
}

// --- sum_abs (minimal_sum_abs_test) -----------------------------------------

#[test]
fn sum_abs_nearest_vectors() {
    assert!(same(
        TightF64::sum_abs_to_nearest(&tf(&[1.0, -2.0, 3.0])).0,
        6.0
    ));
    assert!(same(
        TightF64::sum_abs_to_nearest(&tf(&[1.0, -2.0, NAN, 3.0])).0,
        NAN
    ));
    assert!(same(
        TightF64::sum_abs_to_nearest(&tf(&[1.0, NEG_INF, 2.0, INF, 3.0])).0,
        INF
    ));
}

// --- sum_square (minimal_sum_sqr_test) --------------------------------------

#[test]
fn sum_square_nearest_vectors() {
    assert!(same(
        TightF64::sum_square_to_nearest(&tf(&[1.0, 2.0, 3.0])).0,
        14.0
    ));
    assert!(same(
        TightF64::sum_square_to_nearest(&tf(&[1.0, 2.0, NAN, 3.0])).0,
        NAN
    ));
    assert!(same(
        TightF64::sum_square_to_nearest(&tf(&[1.0, NEG_INF, 2.0, INF, 3.0])).0,
        INF
    ));
}

// --- dot (minimal_dot_test) -------------------------------------------------

#[test]
fn dot_nearest_vectors() {
    // dot {1, 2, 3} {1, 2, 3} = 14.
    assert!(same(
        TightF64::dot_to_nearest(&tf(&[1.0, 2.0, 3.0]), &tf(&[1.0, 2.0, 3.0])).0,
        14.0
    ));

    // The cancellation vector: {0x10000000000001p0, 0x1p104} . {0x0fffffffffffffp0, -1.0} = -1.
    let a1 = 4_503_599_627_370_497.0_f64; // 2^52 + 1  (0x10000000000001)
    let a2 = 2.0_f64.powi(104); // 2^104     (0x1p104)
    let b1 = 4_503_599_627_370_495.0_f64; // 2^52 - 1  (0x0fffffffffffff)
    let b2 = -1.0_f64;
    assert!(same(
        TightF64::dot_to_nearest(&tf(&[a1, a2]), &tf(&[b1, b2])).0,
        -1.0
    ));

    // A NaN in either factor poisons.
    assert!(same(
        TightF64::dot_to_nearest(&tf(&[1.0, 2.0, NAN, 3.0]), &tf(&[1.0, 2.0, 3.0, 4.0])).0,
        NAN
    ));
    assert!(same(
        TightF64::dot_to_nearest(&tf(&[1.0, 2.0, 3.0, 4.0]), &tf(&[1.0, 2.0, NAN, 3.0])).0,
        NAN
    ));

    // A zero against an infinity is the indeterminate 0 * inf, hence NaN.
    assert!(same(
        TightF64::dot_to_nearest(&tf(&[1.0, 2.0, 0.0, 4.0]), &tf(&[1.0, 2.0, INF, 3.0])).0,
        NAN
    ));
    assert!(same(
        TightF64::dot_to_nearest(&tf(&[1.0, 2.0, NEG_INF, 4.0]), &tf(&[1.0, 2.0, 0.0, 3.0])).0,
        NAN
    ));
}

// --- The exactly representable corpus values agree across all four modes ------
// The nearest corpus pins these; an exactly representable result must be equal
// in every direction, which anchors the directed methods to the corpus too.

#[test]
fn representable_corpus_values_agree_in_all_modes() {
    let check_sum = |xs: &[f64], want: f64| {
        let v = tf(xs);
        assert!(same(TightF64::sum_down(&v).0, want));
        assert!(same(TightF64::sum_up(&v).0, want));
        assert!(same(TightF64::sum_to_nearest(&v).0, want));
        assert!(same(TightF64::sum_to_zero(&v).0, want));
    };
    check_sum(&[1.0, 2.0, 3.0], 6.0);

    let v = tf(&[1.0, -2.0, 3.0]);
    assert!(same(TightF64::sum_abs_down(&v).0, 6.0));
    assert!(same(TightF64::sum_abs_up(&v).0, 6.0));
    assert!(same(TightF64::sum_abs_to_nearest(&v).0, 6.0));
    assert!(same(TightF64::sum_abs_to_zero(&v).0, 6.0));

    let v = tf(&[1.0, 2.0, 3.0]);
    assert!(same(TightF64::sum_square_down(&v).0, 14.0));
    assert!(same(TightF64::sum_square_up(&v).0, 14.0));
    assert!(same(TightF64::sum_square_to_nearest(&v).0, 14.0));
    assert!(same(TightF64::sum_square_to_zero(&v).0, 14.0));

    let a = tf(&[1.0, 2.0, 3.0]);
    let b = tf(&[1.0, 2.0, 3.0]);
    assert!(same(TightF64::dot_down(&a, &b).0, 14.0));
    assert!(same(TightF64::dot_up(&a, &b).0, 14.0));
    assert!(same(TightF64::dot_to_nearest(&a, &b).0, 14.0));
    assert!(same(TightF64::dot_to_zero(&a, &b).0, 14.0));
}
