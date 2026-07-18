//! Tests for [`Interval::abs_rev`] over the `f64` fixture, hand-translated from
//! ITF1788 `abs_rev.itl` (public-domain header; original author Oliver
//! Heimlich; see `docs/references/vendor/itf1788-framework/`).
//!
//! The reverse absolute value never rounds (its bounds are selected constraint
//! endpoints and their exact negations), so every assertion in this file is
//! EXACT: the structural empties assert emptiness and the value cases assert the
//! precise endpoint pair. The ITL writes `absRevBin [c] [x]` for the explicit
//! candidate form, which maps to `c.abs_rev(x)`.

use interval_1788::Interval;

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

/// Exact structural equality of a result against a `[lo, hi]` pair.
fn exact(result: Interval<f64>, lo: f64, hi: f64) {
    assert!(
        result.inf() == lo && result.sup() == hi,
        "expected exactly [{lo}, {hi}], got [{}, {}]",
        result.inf(),
        result.sup()
    );
}

#[test]
fn abs_rev_bin_vectors() {
    let entire = Interval::<f64>::entire();
    let empty = Interval::<f64>::empty();

    assert!(empty.abs_rev(entire).is_empty());
    assert!(iv(0.0, 1.0).abs_rev(empty).is_empty());
    assert!(iv(0.0, 1.0).abs_rev(iv(7.0, 9.0)).is_empty());
    assert!(empty.abs_rev(iv(0.0, 1.0)).is_empty());
    assert!(iv(-2.0, -1.0).abs_rev(entire).is_empty());
    exact(iv(1.0, 1.0).abs_rev(entire), -1.0, 1.0);
    exact(iv(0.0, 0.0).abs_rev(entire), 0.0, 0.0);
    assert!(iv(-1.0, -1.0).abs_rev(entire).is_empty());
    // Largest finite and smallest normal magnitudes reflect exactly.
    let big = f64::from_bits(0x7FEF_FFFF_FFFF_FFFF); // 0x1.FFFFFFFFFFFFFp1023
    exact(iv(big, big).abs_rev(entire), -big, big);
    let small = f64::from_bits(0x0010_0000_0000_0000); // 0x1p-1022
    exact(iv(small, small).abs_rev(entire), -small, small);
    assert!(iv(-small, -small).abs_rev(entire).is_empty());
    assert!(iv(-big, -big).abs_rev(entire).is_empty());
    exact(iv(1.0, 2.0).abs_rev(entire), -2.0, 2.0);
    exact(iv(1.0, 2.0).abs_rev(iv(0.0, 2.0)), 1.0, 2.0);
    exact(iv(0.0, 1.0).abs_rev(iv(-0.5, 2.0)), -0.5, 1.0);
    exact(iv(-1.0, 1.0).abs_rev(entire), -1.0, 1.0);
    exact(iv(-1.0, 0.0).abs_rev(entire), 0.0, 0.0);
    exact(iv(0.0, INF).abs_rev(entire), NEG_INF, INF);
    exact(entire.abs_rev(entire), NEG_INF, INF);
    exact(iv(NEG_INF, 0.0).abs_rev(entire), 0.0, 0.0);
    exact(iv(1.0, INF).abs_rev(iv(NEG_INF, 0.0)), NEG_INF, -1.0);
    exact(iv(-1.0, INF).abs_rev(entire), NEG_INF, INF);
    assert!(iv(NEG_INF, -1.0).abs_rev(entire).is_empty());
    exact(iv(NEG_INF, 1.0).abs_rev(entire), -1.0, 1.0);
}
