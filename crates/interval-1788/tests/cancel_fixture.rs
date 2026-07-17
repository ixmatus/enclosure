//! Tests for the cancellative operations over the `f64` fixture:
//! `cancel_minus` and `cancel_plus`, bare and decorated.
//!
//! The unit tests transcribe a representative selection of the vendored ITF1788
//! `libieeep1788_cancel.itl` vectors. Because the fixture is deliberately
//! sound-not-tight (each subtraction steps one float outward), a recovered
//! bounded `z` is up to one ulp wider than the correctly-rounded reference, and
//! an equal-width or sub-ulp-margin input that the reference resolves to a
//! bounded `z` is resolved here to Entire by the conservative width gate. So the
//! bounded cases are checked for the sound relationship the fixture guarantees:
//! the defining property `y + z` encloses `x`, plus the exact outward-rounded
//! endpoint formula where the gate resolves the input to a bounded result. The
//! Entire and empty cases are structural and are checked exactly. The property
//! lanes add the defining property, the recovery round-trip, the `cancel_plus`
//! identity, and totality over empty and unbounded mixes.
//!
//! Cases derived from ITF1788 `libieeep1788_cancel.itl` (Apache-2.0, original
//! author Marco Nehmeier, ITL port Oliver Heimlich; see
//! `docs/references/vendor/itf1788-framework/`).

use interval_1788::{DecoratedInterval, Decoration, Interval};
use proptest::prelude::*;

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;
const MAX: f64 = f64::MAX;

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

fn di(lo: f64, hi: f64, d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(iv(lo, hi), d)
}

fn di_empty(d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(Interval::<f64>::empty(), d)
}

/// The defining property `y + cancel_minus(x, y)` encloses `x`: the recovered `z`
/// added back to `y` re-encloses `x`. THE soundness claim, and it holds whether
/// the fixture returns the bounded `z` or the wider Entire (both enclose), so long
/// as `y` is nonempty.
fn assert_defining_property(x: Interval<f64>, y: Interval<f64>) {
    let z = x.cancel_minus(y);
    let back = y + z;
    assert!(
        back.contains(x.inf()) && back.contains(x.sup()),
        "y + cancel_minus(x, y) must enclose x; x=[{}, {}] y=[{}, {}] z=[{}, {}]",
        x.inf(),
        x.sup(),
        y.inf(),
        y.sup(),
        z.inf(),
        z.sup(),
    );
}

/// A comfortable-margin vector the width gate resolves to a bounded `z` on the
/// fixture: check the exact outward-rounded endpoint formula (Derivation 2) and
/// the defining property. `z_lo = (x_lo - y_lo) rounded down`,
/// `z_hi = (x_hi - y_hi) rounded up`.
fn assert_bounded_formula(xlo: f64, xhi: f64, ylo: f64, yhi: f64) {
    let x = iv(xlo, xhi);
    let y = iv(ylo, yhi);
    let z = x.cancel_minus(y);
    assert!(
        z.is_bounded(),
        "expected a bounded recovery, got an unbounded one"
    );
    assert_eq!(z.inf(), (xlo - ylo).next_down(), "lower endpoint direction");
    assert_eq!(z.sup(), (xhi - yhi).next_up(), "upper endpoint direction");
    assert_defining_property(x, y);
}

// --- Bare cancel_minus: Entire (no-value) vectors --------------------------

#[test]
fn cancel_minus_unbounded_operand_is_entire() {
    // minimal_cancel_minus_test: an unbounded x or y is a no-value case.
    assert!(iv(NEG_INF, -1.0)
        .cancel_minus(Interval::empty())
        .is_entire());
    assert!(iv(NEG_INF, -1.0).cancel_minus(iv(-1.0, 5.0)).is_entire());
    assert!(iv(NEG_INF, -1.0)
        .cancel_minus(Interval::entire())
        .is_entire());
    assert!(Interval::<f64>::entire()
        .cancel_minus(iv(-1.0, 5.0))
        .is_entire());
    assert!(Interval::<f64>::entire()
        .cancel_minus(Interval::entire())
        .is_entire());
    // x bounded, y unbounded.
    assert!(iv(-1.0, 5.0).cancel_minus(iv(NEG_INF, -1.0)).is_entire());
    assert!(iv(-1.0, 5.0).cancel_minus(Interval::entire()).is_entire());
}

#[test]
fn cancel_minus_empty_x_unbounded_y_is_entire() {
    // The vectors pin Entire (not empty) for an empty x against an unbounded y:
    // the unbounded rule wins over the empty rule.
    assert!(Interval::<f64>::empty()
        .cancel_minus(iv(NEG_INF, -1.0))
        .is_entire());
    assert!(Interval::<f64>::empty()
        .cancel_minus(iv(-1.0, INF))
        .is_entire());
    assert!(Interval::<f64>::empty()
        .cancel_minus(Interval::entire())
        .is_entire());
}

#[test]
fn cancel_minus_nonempty_x_empty_y_is_entire() {
    // Nothing to cancel.
    assert!(iv(-10.0, -1.0).cancel_minus(Interval::empty()).is_entire());
    assert!(iv(-10.0, 5.0).cancel_minus(Interval::empty()).is_entire());
    assert!(iv(1.0, 5.0).cancel_minus(Interval::empty()).is_entire());
}

#[test]
fn cancel_minus_wider_y_is_entire() {
    // wid(y) > wid(x) by a comfortable margin: no value, Entire, resolved on the
    // fixture too.
    assert!(iv(-5.0, -1.0).cancel_minus(iv(-5.1, -1.0)).is_entire());
    assert!(iv(-5.0, -1.0).cancel_minus(iv(-5.0, -0.9)).is_entire());
    assert!(iv(-5.0, -1.0).cancel_minus(iv(-5.1, -0.9)).is_entire());
    assert!(iv(-10.0, 5.0).cancel_minus(iv(-10.1, 5.0)).is_entire());
    assert!(iv(-10.0, 5.0).cancel_minus(iv(-10.1, 5.1)).is_entire());
    assert!(iv(1.0, 5.0).cancel_minus(iv(0.9, 5.0)).is_entire());
    assert!(iv(1.0, 5.0).cancel_minus(iv(1.0, 5.1)).is_entire());
    assert!(iv(1.0, 5.0).cancel_minus(iv(0.9, 5.1)).is_entire());
    // The subnormal too-wide vector.
    let d1 = f64::from_bits(0x0010_0000_0000_0000); // 0X1P-1022
    let d2 = f64::from_bits(0x0010_0000_0000_0001); // 0X1.0000000000001P-1022
    let d3 = f64::from_bits(0x0010_0000_0000_0002); // 0X1.0000000000002P-1022
    assert!(iv(d1, d2).cancel_minus(iv(d1, d3)).is_entire());
}

// --- Bare cancel_minus: empty vectors --------------------------------------

#[test]
fn cancel_minus_empty_x_bounded_y_is_empty() {
    // An empty target needs nothing added back, when y is bounded.
    assert!(Interval::<f64>::empty()
        .cancel_minus(Interval::empty())
        .is_empty());
    assert!(Interval::<f64>::empty()
        .cancel_minus(iv(-10.0, -1.0))
        .is_empty());
    assert!(Interval::<f64>::empty()
        .cancel_minus(iv(-10.0, 5.0))
        .is_empty());
    assert!(Interval::<f64>::empty()
        .cancel_minus(iv(1.0, 5.0))
        .is_empty());
}

// --- Bare cancel_minus: bounded recovery vectors ---------------------------

#[test]
fn cancel_minus_comfortable_bounded_recovery() {
    // minimal_cancel_minus_test bounded rows with a comfortable width margin: the
    // gate resolves them on the fixture, so the exact endpoint formula applies.
    assert_bounded_formula(-5.1, -0.0, -5.0, 0.0);
    assert_bounded_formula(-5.1, -1.0, -5.0, -1.0);
    assert_bounded_formula(-5.0, -0.9, -5.0, -1.0);
    assert_bounded_formula(-5.1, -0.9, -5.0, -1.0);
    assert_bounded_formula(-10.1, 5.0, -10.0, 5.0);
    assert_bounded_formula(-10.0, 5.1, -10.0, 5.0);
    assert_bounded_formula(-10.1, 5.1, -10.0, 5.0);
    assert_bounded_formula(0.9, 5.0, 1.0, 5.0);
    assert_bounded_formula(-0.0, 5.1, 0.0, 5.0);
    assert_bounded_formula(1.0, 5.1, 1.0, 5.0);
    assert_bounded_formula(0.9, 5.1, 1.0, 5.0);
}

#[test]
fn cancel_minus_large_magnitude_recovery() {
    // cancelMinus [-M, M] [-M, next_down(M)] = [0, 0X1P+971] on a correctly-rounded
    // backend. A comfortable relative width margin (one ulp at 2^1023), so the
    // fixture resolves it to a bounded recovery and the endpoint formula applies.
    assert_bounded_formula(-MAX, MAX, -MAX, MAX.next_down());
}

#[test]
fn cancel_minus_equal_width_stays_sound() {
    // wid(y) == wid(x): the reference recovers a bounded z, but the conservative
    // gate resolves the sound-not-tight fixture to Entire. Either way the defining
    // property holds. (Reference values, unchecked here: [0,0], [-4,-4], [-5,-5].)
    for (x, y) in [
        (iv(-5.0, -1.0), iv(-5.0, -1.0)),
        (iv(-10.0, 5.0), iv(-10.0, 5.0)),
        (iv(1.0, 5.0), iv(1.0, 5.0)),
        (iv(-5.0, 1.0), iv(-1.0, 5.0)),
        (iv(-5.0, 0.0), iv(-0.0, 5.0)),
    ] {
        assert_defining_property(x, y);
    }
}

#[test]
fn cancel_minus_width_gate_discriminates_at_one_ulp() {
    // The near-1 borderline pair, differing only in which operand is one ulp
    // wider. Case A (x wider) recovers a bounded z; Case B (y wider) is Entire.
    // The rearranged addition gate resolves both correctly on the fixture: the
    // width difference lands at 2^-105, comfortably outside the fixture's widening
    // of the exactly-zero sum x_lo + y_hi.
    let two_pow_m52 = 2.0_f64.powi(-52);
    let a1 = two_pow_m52.next_down(); // 0X1.FFFFFFFFFFFFFP-53
    let b1 = a1.next_down(); // 0X1.FFFFFFFFFFFFEP-53

    // cancelMinus [-1, a1] [-b1, 1]  (x wider)  = bounded.
    let case_a = iv(-1.0, a1).cancel_minus(iv(-b1, 1.0));
    assert!(case_a.is_bounded(), "case A should recover a bounded z");
    assert_defining_property(iv(-1.0, a1), iv(-b1, 1.0));

    // cancelMinus [-1, b1] [-a1, 1]  (y wider)  = entire.
    let case_b = iv(-1.0, b1).cancel_minus(iv(-a1, 1.0));
    assert!(case_b.is_entire(), "case B should be Entire");
}

// --- Bare cancel_plus vectors ----------------------------------------------

#[test]
fn cancel_plus_structural_vectors() {
    // minimal_cancel_plus_test: cancel_plus(x, y) = cancel_minus(x, -y).
    assert!(iv(NEG_INF, -1.0).cancel_plus(Interval::empty()).is_entire());
    assert!(iv(-1.0, INF).cancel_plus(iv(-5.0, 1.0)).is_entire());
    assert!(Interval::<f64>::entire()
        .cancel_plus(Interval::entire())
        .is_entire());
    // x bounded, y unbounded.
    assert!(iv(-1.0, 5.0).cancel_plus(iv(1.0, INF)).is_entire());
    // Empty x, bounded y -> empty; empty x, unbounded y -> Entire.
    assert!(Interval::<f64>::empty()
        .cancel_plus(iv(1.0, 10.0))
        .is_empty());
    assert!(Interval::<f64>::empty()
        .cancel_plus(iv(1.0, INF))
        .is_entire());
    assert!(Interval::<f64>::empty()
        .cancel_plus(Interval::entire())
        .is_entire());
    assert!(Interval::<f64>::empty()
        .cancel_plus(Interval::empty())
        .is_empty());
    // Nonempty x, empty y -> Entire.
    assert!(iv(1.0, 5.0).cancel_plus(Interval::empty()).is_entire());
    // Wider y -> Entire.
    assert!(iv(-5.0, -1.0).cancel_plus(iv(1.0, 5.1)).is_entire());
}

#[test]
fn cancel_plus_bounded_recovery() {
    // minimal_cancel_plus_test bounded rows: check via the equivalence
    // cancel_plus(x, y) = cancel_minus(x, -y), then soundness.
    // cancelPlus [-5.1,-0.0] [0.0,5.0] mirrors cancelMinus [-5.1,-0.0] [-5.0,-0.0].
    let x = iv(-5.1, -0.0);
    let y = iv(0.0, 5.0);
    assert_eq!(x.cancel_plus(y), x.cancel_minus(-y));
    let z = x.cancel_plus(y);
    let back = (-y) + z;
    assert!(back.contains(x.inf()) && back.contains(x.sup()));
}

// --- Decorated vectors: every non-NaI result grades trv --------------------

/// Assert a decorated cancel result has decoration `trv` and the given structural
/// interval shape, matching a `*_dec_test` row whose result is Entire or empty.
fn check_dec_structural(r: DecoratedInterval<f64>, shape: &str) {
    assert_eq!(
        r.decoration(),
        Decoration::Trv,
        "decorated cancel must grade trv"
    );
    match shape {
        "entire" => assert!(r.interval().is_entire()),
        "empty" => assert!(r.interval().is_empty()),
        _ => unreachable!(),
    }
}

#[test]
fn cancel_minus_decorated_grades_trv() {
    use Decoration::{Com, Dac, Trv};
    // minimal_cancel_minus_dec_test: results grade trv regardless of the input
    // decorations, com/com pairs included.
    check_dec_structural(di(NEG_INF, -1.0, Dac).cancel_minus(di_empty(Trv)), "entire");
    check_dec_structural(
        di(-1.0, INF, Dac).cancel_minus(di(-1.0, 5.0, Com)),
        "entire",
    );
    check_dec_structural(di_empty(Trv).cancel_minus(di(-1.0, INF, Dac)), "entire");
    check_dec_structural(di_empty(Trv).cancel_minus(di(-10.0, -1.0, Com)), "empty");
    check_dec_structural(di_empty(Trv).cancel_minus(di_empty(Trv)), "empty");
    // Bounded reference rows still grade trv; the interval part is sound (Entire
    // or a bounded enclosure on the fixture).
    let r = di(-5.1, -0.0, Com).cancel_minus(di(-5.0, 0.0, Com));
    assert_eq!(r.decoration(), Trv);
    let back = iv(-5.0, 0.0) + r.interval();
    assert!(back.contains(-5.1) && back.contains(-0.0));
    // com/com equal-width row: trv, sound.
    let eq = di(-5.0, -1.0, Com).cancel_minus(di(-5.0, -1.0, Dac));
    assert_eq!(eq.decoration(), Trv);
}

#[test]
fn cancel_plus_decorated_grades_trv() {
    use Decoration::{Com, Dac, Def, Trv};
    // minimal_cancel_plus_dec_test.
    check_dec_structural(di(NEG_INF, -1.0, Dac).cancel_plus(di_empty(Trv)), "entire");
    check_dec_structural(di_empty(Trv).cancel_plus(di(1.0, 10.0, Dac)), "empty");
    check_dec_structural(di_empty(Trv).cancel_plus(di(1.0, INF, Def)), "entire");
    let r = di(-5.1, -0.0, Com).cancel_plus(di(0.0, 5.0, Com));
    assert_eq!(r.decoration(), Trv);
}

#[test]
fn cancel_decorated_propagates_nai() {
    let nai = DecoratedInterval::<f64>::nai();
    let good = di(1.0, 2.0, Decoration::Com);
    assert!(nai.cancel_minus(good).is_nai());
    assert!(good.cancel_minus(nai).is_nai());
    assert!(nai.cancel_plus(good).is_nai());
    assert!(good.cancel_plus(nai).is_nai());
    assert!(nai.cancel_minus(nai).is_nai());
}

// --- Property lanes --------------------------------------------------------

fn wid(i: Interval<f64>) -> f64 {
    i.sup() - i.inf()
}

fn bounded() -> impl Strategy<Value = Interval<f64>> {
    (-20.0..20.0f64, -20.0..20.0f64).prop_map(|(a, b)| {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        iv(lo, hi)
    })
}

/// Any interval: bounded, half-bounded, Entire, or empty.
fn any_iv() -> impl Strategy<Value = Interval<f64>> {
    prop_oneof![
        bounded(),
        (-20.0..20.0f64).prop_map(|a| iv(a, INF)),
        (-20.0..20.0f64).prop_map(|b| iv(NEG_INF, b)),
        Just(Interval::<f64>::entire()),
        Just(Interval::<f64>::empty()),
    ]
}

fn any_di() -> impl Strategy<Value = DecoratedInterval<f64>> {
    prop_oneof![
        any_iv().prop_map(DecoratedInterval::new_dec),
        Just(DecoratedInterval::<f64>::nai()),
    ]
}

proptest! {
    /// (a) The defining property: for bounded nonempty x, y with wid(y) <= wid(x)
    /// (selected from a pair), `y + cancel_minus(x, y)` encloses x. THE soundness
    /// property. It also holds when the fixture widens z to Entire.
    #[test]
    fn defining_property(p in bounded(), q in bounded()) {
        let (x, y) = if wid(q) <= wid(p) { (p, q) } else { (q, p) };
        let z = x.cancel_minus(y);
        let back = y + z;
        prop_assert!(back.contains(x.inf()) && back.contains(x.sup()));
    }

    /// (b) Round-trip recovery: with x = y + z, `cancel_minus(x, y)` encloses z.
    /// Equality would need a correctly-rounded backend; containment is the
    /// fixture-sound claim.
    #[test]
    fn round_trip_recovers_z(y in bounded(), z in bounded()) {
        let x = y + z;
        let rec = x.cancel_minus(y);
        prop_assert!(rec.contains(z.inf()) && rec.contains(z.sup()));
    }

    /// (c) `cancel_plus(x, y)` equals `cancel_minus(x, -y)` exactly, over every
    /// input including empty and unbounded ones.
    #[test]
    fn cancel_plus_is_cancel_minus_of_neg(x in any_iv(), y in any_iv()) {
        prop_assert_eq!(x.cancel_plus(y), x.cancel_minus(-y));
    }

    /// (d) Totality: no panic and a valid interval (empty, or ordered endpoints
    /// without NaN) for every input, empty and unbounded mixes included.
    #[test]
    fn total_and_valid(x in any_iv(), y in any_iv()) {
        for r in [x.cancel_minus(y), x.cancel_plus(y)] {
            prop_assert!(
                r.is_empty()
                    || (r.inf() <= r.sup() && !r.inf().is_nan() && !r.sup().is_nan())
            );
        }
    }

    /// The decorated cancellative operations grade `trv` on every non-NaI input
    /// pair, and propagate `NaI`.
    #[test]
    fn decorated_cancel_is_trv_or_nai(x in any_di(), y in any_di()) {
        for r in [x.cancel_minus(y), x.cancel_plus(y)] {
            if x.is_nai() || y.is_nai() {
                prop_assert!(r.is_nai());
            } else {
                prop_assert_eq!(r.decoration(), Decoration::Trv);
            }
        }
    }
}
