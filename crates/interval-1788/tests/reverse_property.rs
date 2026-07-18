//! The reverse-image property lane: the independent soundness hedge for the
//! reverse operations, arbitrated only through the forward battery.
//!
//! For a unary reverse `f_rev(c, x) = hull({ t in x : f(t) in c })`, soundness
//! is: every `t` in the candidate whose forward image lands in the constraint
//! must lie in the reverse result. This lane checks exactly that, using the
//! crate's own FORWARD operations as the oracle (never the reverse
//! derivations): pick a witness `t`, form the forward enclosure `f([t, t])`, and
//! whenever that enclosure is nonempty and a subset of `c` (so the true `f(t)`
//! is certainly in `c`) with `t` in `x`, assert the reverse result contains `t`.
//! Because the antecedent uses a sound forward enclosure and the consequent a
//! sound reverse enclosure, a failure is a genuine non-enclosure, not a rounding
//! coincidence.
//!
//! For `mul_rev` the co-factor `b0` is sampled alongside; for `mul_rev_to_pair`
//! a second block pins the pair invariants (a nonempty second piece implies a
//! disjoint, ordered split) and that `mul_rev` equals the hull of the pair
//! intersected with the candidate.

use interval_1788::Interval;
use proptest::prelude::*;

/// A finite, non-NaN `f64` in a range wide enough to mix signs and magnitudes
/// without routinely overflowing.
fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![
        -30.0..30.0,
        Just(0.0),
        Just(-0.0),
        -1.0e-3..1.0e-3,
        -1.0e6..1.0e6
    ]
}

/// A nonnegative pad for widening an interval around a witness.
fn pad() -> impl Strategy<Value = f64> {
    prop_oneof![Just(0.0), 0.0..5.0, 0.0..1.0e3]
}

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    Interval::new(lo, hi).unwrap()
}

/// A candidate interval guaranteed to contain the witness `t`.
fn around(t: f64, pl: f64, ph: f64) -> Interval<f64> {
    iv(t - pl, t + ph)
}

/// The reverse-image soundness check for a unary reverse: if the forward image
/// of the witness is a nonempty subset of the constraint and the witness is in
/// the candidate, the reverse result must contain the witness.
fn assert_sound(
    forward: Interval<f64>,
    c: Interval<f64>,
    x: Interval<f64>,
    t: f64,
    rev: Interval<f64>,
) -> Result<(), TestCaseError> {
    if !forward.is_empty() && forward.subset(c) && x.contains(t) {
        prop_assert!(
            rev.contains(t),
            "reverse missed t={t}: forward={:?} c={:?} x={:?} rev={:?}",
            (forward.inf(), forward.sup()),
            (c.inf(), c.sup()),
            (x.inf(), x.sup()),
            (rev.inf(), rev.sup())
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    // --- Unary arithmetic reverses ---

    #[test]
    fn sqr_rev_is_sound(t in finite(), clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        assert_sound(Interval::point(t).unwrap().sqr(), c, x, t, c.sqr_rev(x))?;
    }

    #[test]
    fn abs_rev_is_sound(t in finite(), clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        assert_sound(Interval::point(t).unwrap().abs(), c, x, t, c.abs_rev(x))?;
    }

    #[test]
    fn pown_rev_is_sound(t in finite(), n in -5i32..6, clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        assert_sound(Interval::point(t).unwrap().pown(n), c, x, t, c.pown_rev(x, n))?;
    }

    // --- Transcendental reverses ---

    #[test]
    fn sin_rev_is_sound(t in -20.0f64..20.0, clo in -1.5f64..1.5, chi in -1.5f64..1.5, pl in pad(), ph in pad()) {
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        assert_sound(Interval::point(t).unwrap().sin(), c, x, t, c.sin_rev(x))?;
    }

    #[test]
    fn cos_rev_is_sound(t in -20.0f64..20.0, clo in -1.5f64..1.5, chi in -1.5f64..1.5, pl in pad(), ph in pad()) {
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        assert_sound(Interval::point(t).unwrap().cos(), c, x, t, c.cos_rev(x))?;
    }

    #[test]
    fn tan_rev_is_sound(t in -20.0f64..20.0, clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        assert_sound(Interval::point(t).unwrap().tan(), c, x, t, c.tan_rev(x))?;
    }

    #[test]
    fn cosh_rev_is_sound(t in -20.0f64..20.0, clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        assert_sound(Interval::point(t).unwrap().cosh(), c, x, t, c.cosh_rev(x))?;
    }

    // --- Power reverses ---

    #[test]
    fn pow_rev1_is_sound(t in 0.0f64..20.0, e in -6.0f64..6.0, clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        // t is the base (>= 0), e the fixed exponent.
        let b = Interval::point(e).unwrap();
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        let forward = Interval::point(t).unwrap().pow(b);
        assert_sound(forward, c, x, t, b.pow_rev1(c, x))?;
    }

    #[test]
    fn pow_rev2_is_sound(t in -6.0f64..6.0, base in 0.01f64..20.0, clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        // t is the exponent, base the fixed (positive) base.
        let a = Interval::point(base).unwrap();
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        let forward = a.pow(Interval::point(t).unwrap());
        assert_sound(forward, c, x, t, a.pow_rev2(c, x))?;
    }

    // --- mul_rev: sample the co-factor too ---

    #[test]
    fn mul_rev_is_sound(t in finite(), b0 in finite(), blo in finite(), bhi in finite(), clo in finite(), chi in finite(), pl in pad(), ph in pad()) {
        // b0 is a witnessed co-factor; force it into b.
        let b = iv(blo.min(b0), bhi.max(b0));
        let c = iv(clo, chi);
        let x = around(t, pl, ph);
        // The forward image of the witnessed product b0 * t.
        let forward = Interval::point(b0).unwrap() * Interval::point(t).unwrap();
        if !forward.is_empty() && forward.subset(c) && x.contains(t) && b.contains(b0) {
            let rev = b.mul_rev(c, x);
            prop_assert!(
                rev.contains(t),
                "mul_rev missed t={t}, b0={b0}: b={:?} c={:?} x={:?} rev={:?}",
                (b.inf(), b.sup()),
                (c.inf(), c.sup()),
                (x.inf(), x.sup()),
                (rev.inf(), rev.sup())
            );
        }
    }

    // --- mul_rev_to_pair invariants ---

    #[test]
    fn mul_rev_to_pair_invariants(blo in finite(), bhi in finite(), clo in finite(), chi in finite()) {
        let b = iv(blo, bhi);
        let c = iv(clo, chi);
        let (p1, p2) = b.mul_rev_to_pair(c);
        if !p2.is_empty() {
            // A nonempty second piece implies a genuine split: the first piece
            // is nonempty, the two are disjoint, and the first sits wholly below
            // the second.
            prop_assert!(!p1.is_empty(), "second piece nonempty but first empty");
            prop_assert!(p1.disjoint(p2), "split pieces overlap: {:?} {:?}", (p1.inf(), p1.sup()), (p2.inf(), p2.sup()));
            prop_assert!(p1.sup() <= p2.inf(), "first piece not below second");
        }
        // mul_rev equals the hull of the pair intersected with the candidate,
        // for any candidate.
        for x in [Interval::entire(), iv(-1.0, 1.0), iv(blo, chi), iv(0.0, 10.0)] {
            let via_pair = p1.intersection(x).convex_hull(p2.intersection(x));
            let direct = b.mul_rev(c, x);
            prop_assert_eq!(
                (direct.inf(), direct.sup(), direct.is_empty()),
                (via_pair.inf(), via_pair.sup(), via_pair.is_empty()),
                "mul_rev != hull(pair ∩ x)"
            );
        }
    }
}
