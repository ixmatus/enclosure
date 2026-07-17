//! Tests for the interval point functions over the `f64` fixture: `abs`, `min`,
//! `max`, `sign`, `ceil`, `floor`, `trunc`, `round_ties_to_even`, and
//! `round_ties_to_away`, bare and decorated.
//!
//! The unit tests transcribe a representative selection of the vendored ITF1788
//! conformance vectors (the acceptance semantics). The property lanes add
//! pointwise enclosure (spec Law 2), the exactness of the integer-rounding images
//! against the host `f64` rounding, and the derived decoration rule (a step
//! function's image spans one value exactly when its restriction to the box is
//! continuous).
//!
//! Cases derived from ITF1788 `libieeep1788_elem.itl` (Apache-2.0, see
//! `docs/references/vendor/itf1788-framework/`).

use interval_1788::{DecoratedInterval, Decoration, Interval};
use proptest::prelude::*;

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;
// f64::MAX = 0x1.FFFFFFFFFFFFFp1023, an even integer: the ITF1788 vectors use it
// as a finite integer at the edge of the range.
const MAX: f64 = f64::MAX;

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

/// A decorated interval built from explicit endpoints and a decoration, matching
/// the ITF1788 `[lo, hi]_dec` literal.
fn di(lo: f64, hi: f64, d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(iv(lo, hi), d)
}

// --- Bare interval vectors (the `_test` cases) -----------------------------

#[test]
fn abs_bare_vectors() {
    // minimal_abs_test
    assert!(Interval::<f64>::empty().abs().is_empty());
    assert_eq!(Interval::<f64>::entire().abs(), iv(0.0, INF));
    assert_eq!(iv(1.1, 2.1).abs(), iv(1.1, 2.1));
    assert_eq!(iv(-1.1, 2.0).abs(), iv(0.0, 2.0));
    assert_eq!(iv(-1.1, 0.0).abs(), iv(0.0, 1.1));
    assert_eq!(iv(-1.1, -0.0).abs(), iv(0.0, 1.1)); // -0.0 upper still touches zero
    assert_eq!(iv(-1.1, -0.4).abs(), iv(0.4, 1.1)); // no straddle: min magnitude
    assert_eq!(iv(-1.9, 0.2).abs(), iv(0.0, 1.9));
    assert_eq!(iv(-0.0, 0.2).abs(), iv(0.0, 0.2));
    assert_eq!(iv(-1.5, INF).abs(), iv(0.0, INF));
    assert_eq!(iv(NEG_INF, -2.2).abs(), iv(2.2, INF));
}

#[test]
fn min_bare_vectors() {
    // minimal_min_test
    assert!(Interval::<f64>::empty().min(iv(1.0, 2.0)).is_empty());
    assert!(iv(1.0, 2.0).min(Interval::empty()).is_empty());
    assert_eq!(Interval::entire().min(iv(1.0, 2.0)), iv(NEG_INF, 2.0));
    assert_eq!(iv(1.0, 5.0).min(iv(2.0, 4.0)), iv(1.0, 4.0));
    assert_eq!(iv(0.0, 5.0).min(iv(2.0, 4.0)), iv(0.0, 4.0));
    assert_eq!(iv(1.0, 5.0).min(iv(2.0, 8.0)), iv(1.0, 5.0));
    assert_eq!(iv(-7.0, -5.0).min(iv(2.0, 4.0)), iv(-7.0, -5.0));
    assert_eq!(iv(-7.0, 0.0).min(iv(2.0, 4.0)), iv(-7.0, 0.0));
}

#[test]
fn max_bare_vectors() {
    // minimal_max_test
    assert!(Interval::<f64>::empty().max(iv(1.0, 2.0)).is_empty());
    assert_eq!(Interval::entire().max(iv(1.0, 2.0)), iv(1.0, INF));
    assert_eq!(iv(1.0, 5.0).max(iv(2.0, 4.0)), iv(2.0, 5.0));
    assert_eq!(iv(1.0, 5.0).max(iv(2.0, 8.0)), iv(2.0, 8.0));
    assert_eq!(iv(-7.0, -5.0).max(iv(2.0, 4.0)), iv(2.0, 4.0));
    assert_eq!(iv(-7.0, -5.0).max(iv(0.0, 4.0)), iv(0.0, 4.0));
    assert_eq!(iv(-7.0, -5.0).max(iv(-2.0, 0.0)), iv(-2.0, 0.0));
}

#[test]
fn sign_bare_vectors() {
    // minimal_sign_test
    assert!(Interval::<f64>::empty().sign().is_empty());
    assert_eq!(iv(1.0, 2.0).sign(), iv(1.0, 1.0));
    assert_eq!(iv(-1.0, 2.0).sign(), iv(-1.0, 1.0));
    assert_eq!(iv(-1.0, 0.0).sign(), iv(-1.0, 0.0));
    assert_eq!(iv(0.0, 2.0).sign(), iv(0.0, 1.0));
    assert_eq!(iv(-0.0, 2.0).sign(), iv(0.0, 1.0));
    assert_eq!(iv(-5.0, -2.0).sign(), iv(-1.0, -1.0));
    assert_eq!(iv(0.0, 0.0).sign(), iv(0.0, 0.0));
    assert_eq!(iv(-0.0, -0.0).sign(), iv(0.0, 0.0));
    assert_eq!(Interval::<f64>::entire().sign(), iv(-1.0, 1.0));
}

#[test]
fn ceil_bare_vectors() {
    // minimal_ceil_test
    assert!(Interval::<f64>::empty().ceil().is_empty());
    assert!(Interval::<f64>::entire().ceil().is_entire());
    assert_eq!(iv(1.1, 2.0).ceil(), iv(2.0, 2.0));
    assert_eq!(iv(-1.1, 2.0).ceil(), iv(-1.0, 2.0));
    assert_eq!(iv(-1.1, -0.4).ceil(), iv(-1.0, 0.0));
    assert_eq!(iv(-1.9, 2.2).ceil(), iv(-1.0, 3.0));
    assert_eq!(iv(0.0, 2.2).ceil(), iv(0.0, 3.0));
    assert_eq!(iv(-1.5, INF).ceil(), iv(-1.0, INF));
    assert_eq!(iv(NEG_INF, 2.2).ceil(), iv(NEG_INF, 3.0));
}

#[test]
fn floor_bare_vectors() {
    // minimal_floor_test
    assert!(Interval::<f64>::empty().floor().is_empty());
    assert!(Interval::<f64>::entire().floor().is_entire());
    assert_eq!(iv(1.1, 2.0).floor(), iv(1.0, 2.0));
    assert_eq!(iv(-1.1, 2.0).floor(), iv(-2.0, 2.0));
    assert_eq!(iv(-1.1, -0.4).floor(), iv(-2.0, -1.0));
    assert_eq!(iv(-1.9, 2.2).floor(), iv(-2.0, 2.0));
    assert_eq!(iv(-1.0, 2.2).floor(), iv(-1.0, 2.0));
    assert_eq!(iv(-1.5, INF).floor(), iv(-2.0, INF));
    assert_eq!(iv(NEG_INF, 2.2).floor(), iv(NEG_INF, 2.0));
}

#[test]
fn trunc_bare_vectors() {
    // minimal_trunc_test
    assert!(Interval::<f64>::empty().trunc().is_empty());
    assert_eq!(iv(1.1, 2.1).trunc(), iv(1.0, 2.0));
    assert_eq!(iv(-1.1, 2.0).trunc(), iv(-1.0, 2.0));
    assert_eq!(iv(-1.1, -0.4).trunc(), iv(-1.0, 0.0)); // toward zero
    assert_eq!(iv(-1.9, 2.2).trunc(), iv(-1.0, 2.0));
    assert_eq!(iv(0.0, 2.2).trunc(), iv(0.0, 2.0));
    assert_eq!(iv(-1.5, INF).trunc(), iv(-1.0, INF));
}

#[test]
fn round_ties_to_even_bare_vectors() {
    // minimal_round_ties_to_even_test
    assert_eq!(iv(1.1, 2.1).round_ties_to_even(), iv(1.0, 2.0));
    assert_eq!(iv(1.5, 2.1).round_ties_to_even(), iv(2.0, 2.0));
    assert_eq!(iv(-1.5, 2.0).round_ties_to_even(), iv(-2.0, 2.0));
    assert_eq!(iv(-1.9, 2.5).round_ties_to_even(), iv(-2.0, 2.0)); // 2.5 -> 2 (even)
    assert_eq!(iv(-1.5, 2.5).round_ties_to_even(), iv(-2.0, 2.0));
    assert_eq!(iv(0.0, 2.5).round_ties_to_even(), iv(0.0, 2.0));
    assert_eq!(iv(-1.5, INF).round_ties_to_even(), iv(-2.0, INF));
}

#[test]
fn round_ties_to_away_bare_vectors() {
    // minimal_round_ties_to_away_test
    assert_eq!(iv(1.1, 2.1).round_ties_to_away(), iv(1.0, 2.0));
    assert_eq!(iv(0.5, 2.1).round_ties_to_away(), iv(1.0, 2.0)); // 0.5 -> 1 (away)
    assert_eq!(iv(-2.5, 2.0).round_ties_to_away(), iv(-3.0, 2.0));
    assert_eq!(iv(-1.9, 2.5).round_ties_to_away(), iv(-2.0, 3.0)); // 2.5 -> 3 (away)
    assert_eq!(iv(-1.5, 2.5).round_ties_to_away(), iv(-2.0, 3.0));
    assert_eq!(iv(0.0, 2.5).round_ties_to_away(), iv(0.0, 3.0));
    assert_eq!(iv(NEG_INF, 2.2).round_ties_to_away(), iv(NEG_INF, 2.0));
}

// --- Decorated vectors (the `_dec_test` cases) -----------------------------

/// Assert a decorated result equals the expected interval and decoration.
fn check(r: DecoratedInterval<f64>, lo: f64, hi: f64, d: Decoration) {
    assert_eq!(r.interval(), iv(lo, hi), "interval part mismatch");
    assert_eq!(r.decoration(), d, "decoration mismatch");
}

#[test]
fn abs_decorated_vectors() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_abs_dec_test: abs is continuous everywhere, so `def` only enters
    // by propagation, never locally.
    check(di(-1.1, 2.0, Com).abs(), 0.0, 2.0, Com);
    check(di(-1.1, 0.0, Dac).abs(), 0.0, 1.1, Dac);
    check(di(-1.1, -0.0, Def).abs(), 0.0, 1.1, Def);
    check(di(-1.1, -0.4, Trv).abs(), 0.4, 1.1, Trv);
    check(di(0.0, 0.2, Def).abs(), 0.0, 0.2, Def);
    check(di(-1.5, INF, Dac).abs(), 0.0, INF, Dac);
    assert!(DecoratedInterval::<f64>::nai().abs().is_nai());
}

#[test]
fn min_max_decorated_vectors() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_min_dec_test
    check(
        di(NEG_INF, INF, Dac).min(di(1.0, 2.0, Com)),
        NEG_INF,
        2.0,
        Dac,
    );
    check(di(-7.0, -5.0, Trv).min(di(2.0, 4.0, Def)), -7.0, -5.0, Trv);
    check(di(-7.0, 0.0, Dac).min(di(2.0, 4.0, Def)), -7.0, 0.0, Def);
    check(di(-7.0, -0.0, Com).min(di(2.0, 4.0, Com)), -7.0, 0.0, Com);
    // minimal_max_dec_test
    check(di(NEG_INF, INF, Dac).max(di(1.0, 2.0, Com)), 1.0, INF, Dac);
    check(di(-7.0, -5.0, Trv).max(di(2.0, 4.0, Def)), 2.0, 4.0, Trv);
    check(di(-7.0, 5.0, Dac).max(di(2.0, 4.0, Def)), 2.0, 5.0, Def);
    check(di(3.0, 3.5, Com).max(di(2.0, 4.0, Com)), 3.0, 4.0, Com);
    assert!(DecoratedInterval::<f64>::nai()
        .min(di(1.0, 2.0, Com))
        .is_nai());
}

#[test]
fn sign_decorated_vectors() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_sign_dec_test: `sign` jumps only at zero, so the box straddling
    // zero (image spanning two signs) earns `def`; the singleton {0} is
    // continuous but touches the jump, so it drops from `com` to `dac`.
    check(di(1.0, 2.0, Com).sign(), 1.0, 1.0, Com);
    check(di(-1.0, 2.0, Com).sign(), -1.0, 1.0, Def);
    check(di(-1.0, 0.0, Com).sign(), -1.0, 0.0, Def);
    check(di(0.0, 2.0, Com).sign(), 0.0, 1.0, Def);
    check(di(-0.0, 2.0, Def).sign(), 0.0, 1.0, Def);
    check(di(-5.0, -2.0, Trv).sign(), -1.0, -1.0, Trv);
    check(di(0.0, 0.0, Dac).sign(), 0.0, 0.0, Dac);
}

#[test]
fn ceil_decorated_vectors() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_ceil_dec_test
    check(di(1.1, 2.0, Com).ceil(), 2.0, 2.0, Dac); // integer at the right endpoint
    check(di(-1.1, 2.0, Com).ceil(), -1.0, 2.0, Def);
    check(di(-1.1, 0.0, Dac).ceil(), -1.0, 0.0, Def);
    check(di(-1.9, 2.2, Com).ceil(), -1.0, 3.0, Def);
    check(di(-0.0, 2.2, Def).ceil(), 0.0, 3.0, Def);
    check(di(-1.5, INF, Trv).ceil(), -1.0, INF, Trv);
    check(di(MAX, MAX, Com).ceil(), MAX, MAX, Dac); // singleton at an integer
    check(di(NEG_INF, 2.2, Trv).ceil(), NEG_INF, 3.0, Trv);
}

#[test]
fn floor_decorated_vectors() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_floor_dec_test
    check(di(1.1, 2.0, Com).floor(), 1.0, 2.0, Def);
    check(di(-1.2, -1.1, Com).floor(), -2.0, -2.0, Com); // no integer in the box
    check(di(-1.1, 0.0, Dac).floor(), -2.0, 0.0, Def);
    check(di(-0.0, 2.2, Com).floor(), 0.0, 2.0, Def);
    check(di(-1.0, 2.2, Trv).floor(), -1.0, 2.0, Trv);
    check(di(-1.5, INF, Dac).floor(), -2.0, INF, Def);
    check(di(-MAX, -MAX, Com).floor(), -MAX, -MAX, Dac); // singleton at an integer
}

#[test]
fn trunc_decorated_vectors() {
    use Decoration::{Com, Dac, Def};
    // minimal_trunc_dec_test
    check(di(1.1, 2.1, Com).trunc(), 1.0, 2.0, Def);
    check(di(1.1, 1.9, Com).trunc(), 1.0, 1.0, Com); // no nonzero integer in the box
    check(di(-1.1, -0.0, Def).trunc(), -1.0, 0.0, Def);
    check(di(-1.1, -0.4, Com).trunc(), -1.0, 0.0, Def);
    check(di(-1.9, 2.2, Def).trunc(), -1.0, 2.0, Def);
    check(di(MAX, MAX, Com).trunc(), MAX, MAX, Dac); // singleton at a nonzero integer
    check(di(MAX, INF, Dac).trunc(), MAX, INF, Def);
}

#[test]
fn round_ties_to_even_decorated_vectors() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_round_ties_to_even_dec_test
    check(di(1.1, 2.1, Com).round_ties_to_even(), 1.0, 2.0, Def);
    check(di(-1.1, 2.0, Trv).round_ties_to_even(), -1.0, 2.0, Trv);
    // -1.5 is a half-integer at the right endpoint: continuous but jump-touching.
    check(di(-1.6, -1.5, Com).round_ties_to_even(), -2.0, -2.0, Dac);
    check(di(-1.6, -1.4, Com).round_ties_to_even(), -2.0, -1.0, Def);
    check(di(-1.5, INF, Dac).round_ties_to_even(), -2.0, INF, Def);
    check(
        di(NEG_INF, 2.2, Trv).round_ties_to_even(),
        NEG_INF,
        2.0,
        Trv,
    );
}

#[test]
fn round_ties_to_away_decorated_vectors() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_round_ties_to_away_dec_test
    check(di(1.1, 2.1, Com).round_ties_to_away(), 1.0, 2.0, Def);
    check(di(1.9, 2.2, Com).round_ties_to_away(), 2.0, 2.0, Com); // no half-integer in box
    check(di(2.5, 2.6, Com).round_ties_to_away(), 3.0, 3.0, Dac); // 2.5 half-integer at left
    check(di(-1.0, 2.2, Trv).round_ties_to_away(), -1.0, 2.0, Trv);
    check(
        di(NEG_INF, 2.2, Def).round_ties_to_away(),
        NEG_INF,
        2.0,
        Def,
    );
}

// --- Property lanes --------------------------------------------------------

/// Finite endpoints biased toward the integer and half-integer lattice, so the
/// step functions' discontinuities are exercised, not just skipped over.
fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![
        -8.0..8.0,
        (-8i32..8).prop_map(f64::from),              // integers
        (-8i32..8).prop_map(|n| f64::from(n) + 0.5), // half-integers
        Just(0.0),
        Just(-0.0),
    ]
}

fn bounded() -> impl Strategy<Value = Interval<f64>> {
    (finite(), finite()).prop_map(|(a, b)| {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        Interval::new(lo, hi).unwrap()
    })
}

/// A decorated interval spanning the decoration lattice, including unbounded and
/// empty ones.
fn any_di() -> impl Strategy<Value = DecoratedInterval<f64>> {
    prop_oneof![
        bounded().prop_map(DecoratedInterval::new_dec),
        finite().prop_map(|a| DecoratedInterval::new_dec(Interval::new(a, INF).unwrap())),
        finite().prop_map(|b| DecoratedInterval::new_dec(Interval::new(NEG_INF, b).unwrap())),
        Just(DecoratedInterval::new_dec(Interval::<f64>::empty())),
    ]
}

proptest! {
    /// Pointwise enclosure (spec Law 2) for every point function: a sampled point
    /// of the box maps into the image.
    #[test]
    fn point_functions_enclose_pointwise(a in bounded(), b in bounded(), t in 0.0f64..1.0) {
        let (lo, hi) = (a.inf(), a.sup());
        let x = lo + (hi - lo) * t;
        let sgn = |v: f64| if v < 0.0 { -1.0 } else if v > 0.0 { 1.0 } else { 0.0 };
        prop_assert!(a.abs().contains(x.abs()));
        prop_assert!(a.sign().contains(sgn(x)));
        prop_assert!(a.ceil().contains(x.ceil()));
        prop_assert!(a.floor().contains(x.floor()));
        prop_assert!(a.trunc().contains(x.trunc()));
        prop_assert!(a.round_ties_to_even().contains(x.round_ties_even()));
        prop_assert!(a.round_ties_to_away().contains(x.round()));
        // min/max enclose the pointwise combination of a sample from each box.
        let y = b.inf() + (b.sup() - b.inf()) * t;
        prop_assert!(a.min(b).contains(x.min(y)));
        prop_assert!(a.max(b).contains(x.max(y)));
    }

    /// The integer-rounding images are exact: the endpoints are precisely the
    /// host `f64` rounding of the interval's endpoints, no outward widening.
    #[test]
    fn integer_rounding_is_exact(a in bounded()) {
        let (lo, hi) = (a.inf(), a.sup());
        prop_assert_eq!(a.floor(), Interval::new(lo.floor(), hi.floor()).unwrap());
        prop_assert_eq!(a.ceil(), Interval::new(lo.ceil(), hi.ceil()).unwrap());
        prop_assert_eq!(a.trunc(), Interval::new(lo.trunc(), hi.trunc()).unwrap());
        prop_assert_eq!(
            a.round_ties_to_even(),
            Interval::new(lo.round_ties_even(), hi.round_ties_even()).unwrap()
        );
        prop_assert_eq!(
            a.round_ties_to_away(),
            Interval::new(lo.round(), hi.round()).unwrap()
        );
    }

    /// The derived decoration rule for the step functions: a nonempty result
    /// whose image spans more than one value (the restriction is discontinuous)
    /// is decorated at most `def`; and `com` is reached only with a bounded
    /// nonempty result. Checked without reusing the local-decoration code, by
    /// inspecting the image the operation returned.
    #[test]
    fn step_function_decoration_tracks_the_image(a in any_di()) {
        prop_assume!(!a.is_nai());
        let results = [
            a.sign(),
            a.ceil(),
            a.floor(),
            a.trunc(),
            a.round_ties_to_even(),
            a.round_ties_to_away(),
        ];
        for r in results {
            // Never strengthens past the input decoration.
            prop_assert!(r.decoration() <= a.decoration());
            let img = r.interval();
            if !img.is_empty() && img.inf() != img.sup() {
                // Image spans >= 2 values: the step function is discontinuous on
                // the box, so it cannot be `com` or `dac`.
                prop_assert!(r.decoration() <= Decoration::Def);
            }
            if r.decoration() == Decoration::Com {
                prop_assert!(img.is_bounded() && !img.is_empty());
            }
        }
    }

    /// `abs`, `min`, and `max` are continuous everywhere, so they behave like the
    /// arithmetic operations: `com` on bounded nonempty inputs, never below `dac`
    /// on a nonempty box, and the decoration never strengthens.
    #[test]
    fn continuous_point_functions_stay_dac_or_better_on_nonempty(a in any_di(), b in any_di()) {
        prop_assume!(!a.is_nai() && !b.is_nai());
        let unary = a.abs();
        prop_assert!(unary.decoration() <= a.decoration());
        if !a.interval().is_empty() {
            prop_assert!(unary.decoration() >= Decoration::Dac);
        }
        for r in [a.min(b), a.max(b)] {
            prop_assert!(r.decoration() <= a.decoration());
            prop_assert!(r.decoration() <= b.decoration());
            if !a.interval().is_empty() && !b.interval().is_empty() {
                prop_assert!(r.decoration() >= Decoration::Dac);
            }
        }
    }
}
