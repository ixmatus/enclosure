//! Property tests for the interval elementary functions over the f64 fixture.
//!
//! The load-bearing property is pointwise enclosure (spec Law 2): for every
//! sampled `x` in the input interval (and in the function's domain), `f(x)`
//! lies in the image interval. The fixture is sound but not tight, so these
//! check soundness; tightness belongs to a correctly-rounded backend and the
//! oracle lane.

use interval_1788::Interval;
use proptest::prelude::*;

proptest! {
    /// `exp` encloses the pointwise exponential across the input.
    #[test]
    fn exp_encloses_pointwise(
        a in -700.0f64..700.0,
        w in 0.0f64..50.0,
        t in 0.0f64..1.0,
    ) {
        let iv = Interval::new(a, a + w).expect("ordered endpoints");
        let image = iv.exp();
        let x = a + w * t;
        let fx = x.exp();
        prop_assert!(
            image.contains(fx),
            "exp({x}) = {fx} escaped [{}, {}]",
            image.inf(),
            image.sup()
        );
        prop_assert!(image.inf() >= 0.0, "exp image dipped below zero");
    }

    /// `ln` encloses the pointwise logarithm across the in-domain part of the
    /// input, including inputs that straddle or touch zero.
    #[test]
    fn ln_encloses_pointwise(
        a in -10.0f64..1.0e9,
        w in 0.0f64..1.0e9,
        t in 0.0f64..1.0,
    ) {
        let iv = Interval::new(a, a + w).expect("ordered endpoints");
        let image = iv.ln();
        let x = a + w * t;
        if x > 0.0 {
            let fx = x.ln();
            prop_assert!(
                image.contains(fx),
                "ln({x}) = {fx} escaped [{}, {}]",
                image.inf(),
                image.sup()
            );
        } else {
            // The sampled point is outside the domain; the image must still be
            // the enclosure of the defined part (possibly empty when the whole
            // input is nonpositive).
            prop_assert!(a + w <= 0.0 || !image.is_empty());
        }
    }

    /// The exponential and logarithm cancel on positive inputs: `exp(ln(X))`
    /// encloses `X`'s positive part pointwise.
    #[test]
    fn exp_ln_round_trip_encloses(
        a in 1.0e-300f64..1.0e300,
        factor in 1.0f64..1.0e6,
        t in 0.0f64..1.0,
    ) {
        let b = a * factor;
        let iv = Interval::new(a, b).expect("ordered endpoints");
        let round = iv.ln().exp();
        let x = a + (b - a) * t;
        prop_assert!(
            round.contains(x),
            "{x} escaped exp(ln([{a}, {b}])) = [{}, {}]",
            round.inf(),
            round.sup()
        );
    }
}
