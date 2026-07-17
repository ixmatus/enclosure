//! Tightness pins against interval arithmetic on correlated expressions, over
//! the f64 fixture.
//!
//! A named catalogue of expressions where affine arithmetic beats interval
//! arithmetic because it remembers the correlation interval arithmetic forgets.
//! Each is computed twice from identical inputs: once through interval-1788
//! directly, once through affine forms reduced at the end. Two guards per
//! expression, bucketed per the workspace regression-guard rule (no aggregate
//! floor; each expression keeps its own pin so compensating regressions cannot
//! hide):
//!
//! - a property over random ranges asserting the affine width is at most the
//!   interval width plus an ulp-scale dust allowance (both routes run the
//!   sound-not-tight fixture, so at degenerate widths their outward dust is
//!   incomparable, hence the allowance);
//! - a fixed-input pin of the measured win ratio, set below the measured value
//!   with stated headroom so the pin is not a margin at the floor.
//!
//! Pins are fixture-relative: the ratios measured over the sound-not-tight
//! fixture do not transfer numerically to a correctly-rounded backend, where
//! they can only improve (the affine route sheds its unconditional outward
//! steps faster than the interval route sheds fewer). A closing non-goal test
//! pins an uncorrelated multiply where interval arithmetic is tighter, so the
//! lane never reads as a claim of universal affine superiority.

use affine_arith::{with_source, AffineForm, SymbolSource};
use interval_1788::Interval;
use proptest::prelude::*;

/// Build a form enclosing `[lo, hi]` (with `lo <= hi`) from the given source.
fn form_of<'id>(lo: f64, hi: f64, src: &mut SymbolSource<'id>) -> AffineForm<'id, f64> {
    let iv = Interval::new(lo, hi).expect("ordered finite endpoints");
    AffineForm::from_interval(&iv, src).expect("bounded nonempty")
}

/// The reduced width of an interval.
fn width(iv: &Interval<f64>) -> f64 {
    iv.sup() - iv.inf()
}

// --- The catalogue: each returns (affine reduced, interval reduced) ---

/// (1) `x - x`: the canonical collapse. The shared symbols cancel in the affine
/// route; interval arithmetic, having forgotten `x` is the same on both sides,
/// pays the doubled width `2 w(x)`.
fn expr_x_minus_x(a: f64, b: f64) -> (Interval<f64>, Interval<f64>) {
    let affine = with_source(|mut src| {
        let x = form_of(a, b, &mut src);
        x.sub(&x, &mut src).reduce()
    });
    let ivx = Interval::new(a, b).expect("ordered");
    // The self-subtraction is the point: interval arithmetic keeps `ivx - ivx`
    // as the full doubled width, which is exactly what this catalogue entry
    // contrasts against.
    #[allow(clippy::eq_op)]
    let interval = ivx - ivx;
    (affine, interval)
}

/// (2) `(x + y) - x`: partial cancellation. The `x` cancels in the affine route,
/// leaving `w(y)` plus dust; interval arithmetic pays `2 w(x) + w(y)`.
fn expr_x_plus_y_minus_x(ax: f64, bx: f64, ay: f64, by: f64) -> (Interval<f64>, Interval<f64>) {
    let affine = with_source(|mut src| {
        let x = form_of(ax, bx, &mut src);
        let y = form_of(ay, by, &mut src);
        x.add(&y, &mut src).sub(&x, &mut src).reduce()
    });
    let ivx = Interval::new(ax, bx).expect("ordered");
    let ivy = Interval::new(ay, by).expect("ordered");
    (affine, (ivx + ivy) - ivx)
}

/// (3) `x * (1 - x)` on a range inside `[0, 1]`: anticorrelated factors. The
/// affine multiply sees the shared symbol on both factors and cancels the
/// leading correlation; interval arithmetic multiplies two independent-looking
/// ranges and keeps the full product box.
fn expr_x_times_one_minus_x(a: f64, b: f64) -> (Interval<f64>, Interval<f64>) {
    let affine = with_source(|mut src| {
        let x = form_of(a, b, &mut src);
        let one_minus_x = AffineForm::point(1.0).sub(&x, &mut src);
        x.mul(&one_minus_x, &mut src).reduce()
    });
    let ivx = Interval::new(a, b).expect("ordered");
    let interval = ivx * (Interval::point(1.0).expect("finite") - ivx);
    (affine, interval)
}

/// (4) `x*x - 2x` via `sqr`, `scale`, `add`: a polynomial whose interval
/// evaluation forgets that the square and the linear term move together, so it
/// cannot cancel their shared motion; the affine route keeps the shared symbol
/// through both and cancels it.
fn expr_x_squared_minus_2x(a: f64, b: f64) -> (Interval<f64>, Interval<f64>) {
    let affine = with_source(|mut src| {
        let x = form_of(a, b, &mut src);
        let sq = x.sqr(&mut src);
        let lin = x.scale(-2.0, &mut src); // -2x
        sq.add(&lin, &mut src).reduce()
    });
    let ivx = Interval::new(a, b).expect("ordered");
    let interval = ivx.sqr() + ivx * Interval::point(-2.0).expect("finite");
    (affine, interval)
}

/// (5) `x / (1 + x)` on a positive range via `recip` and `mul`: a correlated
/// quotient chain. The numerator and the denominator share `x`, so the affine
/// route cancels the shared motion; interval arithmetic treats them as
/// independent and keeps the whole quotient box.
fn expr_x_over_one_plus_x(a: f64, b: f64) -> (Interval<f64>, Interval<f64>) {
    let affine = with_source(|mut src| {
        let x = form_of(a, b, &mut src);
        let denom = AffineForm::point(1.0).add(&x, &mut src); // 1 + x
        let recip = denom.recip(&mut src).expect("positive denominator");
        x.mul(&recip, &mut src).reduce()
    });
    let ivx = Interval::new(a, b).expect("ordered");
    let interval = ivx * (Interval::point(1.0).expect("finite") + ivx).recip();
    (affine, interval)
}

// --- Guard (a): the width property over random ranges ---

/// The ulp-scale dust allowance: the affine and interval routes both run the
/// sound-not-tight fixture, whose unconditional outward step widens each
/// operation by up to a unit in the last place, so at degenerate widths the
/// affine width can exceed the interval width by a few ulps of the operating
/// magnitude without any loss of tightness. Sized well above that ulp gap and
/// far below any real regression (which widens by a factor, not by ulps).
fn dust(affine: &Interval<f64>, interval: &Interval<f64>) -> f64 {
    let mag = affine
        .inf()
        .abs()
        .max(affine.sup().abs())
        .max(interval.inf().abs())
        .max(interval.sup().abs())
        .max(1.0);
    1.0e-9 * mag
}

proptest! {
    #[test]
    fn x_minus_x_no_wider_than_interval(a in -1.0e3f64..1.0e3, w in 0.0f64..1.0e3) {
        let (affine, interval) = expr_x_minus_x(a, a + w);
        prop_assert!(width(&affine) <= width(&interval) + dust(&affine, &interval),
            "affine {} > interval {}", width(&affine), width(&interval));
    }

    #[test]
    fn x_plus_y_minus_x_no_wider_than_interval(
        ax in -1.0e3f64..1.0e3, wx in 0.0f64..1.0e3,
        ay in -1.0e3f64..1.0e3, wy in 0.0f64..1.0e3,
    ) {
        let (affine, interval) = expr_x_plus_y_minus_x(ax, ax + wx, ay, ay + wy);
        prop_assert!(width(&affine) <= width(&interval) + dust(&affine, &interval),
            "affine {} > interval {}", width(&affine), width(&interval));
    }

    #[test]
    fn x_times_one_minus_x_no_wider_than_interval(a in 0.05f64..0.7, w in 0.0f64..0.24) {
        let (affine, interval) = expr_x_times_one_minus_x(a, a + w);
        prop_assert!(width(&affine) <= width(&interval) + dust(&affine, &interval),
            "affine {} > interval {}", width(&affine), width(&interval));
    }

    // Expressions 4 and 5 carry a nonlinear op (`sqr`, `recip`) whose Chebyshev
    // linearization residual grows with the range width (the `sqr` half-gap is
    // `w^2/8`). The correlation cancellation wins only while that residual stays
    // below what interval arithmetic pays for forgetting the correlation, i.e.
    // over moderate widths; beyond them affine legitimately loses (the
    // documented non-goal territory), so the width ranges here stay in the
    // correlation-dominated regime the catalogue is about.
    //
    // For `x*x - 2x` the square and the linear term only fight (so interval
    // loses and affine's cancellation wins) where their motions oppose, i.e.
    // over positive `x`: there the square increases while `-2x` decreases. Over
    // negative `x` both fall together, interval is already exact, and affine
    // would only pay the `sqr` residual, so the property is stated over positive
    // ranges where the advertised correlation actually exists.
    #[test]
    fn x_squared_minus_2x_no_wider_than_interval(a in 0.2f64..5.0, w in 0.0f64..5.0) {
        let (affine, interval) = expr_x_squared_minus_2x(a, a + w);
        prop_assert!(width(&affine) <= width(&interval) + dust(&affine, &interval),
            "affine {} > interval {}", width(&affine), width(&interval));
    }

    #[test]
    fn x_over_one_plus_x_no_wider_than_interval(a in 0.1f64..5.0, w in 0.0f64..5.0) {
        let (affine, interval) = expr_x_over_one_plus_x(a, a + w);
        prop_assert!(width(&affine) <= width(&interval) + dust(&affine, &interval),
            "affine {} > interval {}", width(&affine), width(&interval));
    }
}

// --- Guard (b): the fixed-input win-ratio pins ---
//
// The pinned quantity is the residual width fraction `affine_w / interval_w`:
// below 1 means affine is tighter, and its reciprocal is the intuitive "affine
// is N times tighter" win. The residual is used rather than that reciprocal
// because for `x - x` the affine width collapses to a few subnormals and the
// reciprocal overflows to infinity; the residual stays finite everywhere. Each
// pin sits above its measured value with stated headroom, so a benign ulp-level
// change does not trip the guard while a real loss of the cancellation (the
// residual climbing toward 1, or past it) does. Values are fixture-relative and
// can only improve on a correctly-rounded backend.

/// The residual width fraction `affine_w / interval_w`.
fn residual(affine: &Interval<f64>, interval: &Interval<f64>) -> f64 {
    width(affine) / width(interval)
}

#[test]
fn pin_x_minus_x() {
    // Correlation win: the shared symbols cancel exactly; interval doubles the
    // width. Measured residual ~2.47e-323 (a handful of subnormals the fixture's
    // unconditional outward step leaves where the true collapse is exact zero).
    // Pinned at 1e-300: ~23 orders of headroom, so subnormal-count wobble never
    // trips it, yet any real loss of cancellation (residual jumping to order
    // 1e-16 or higher) is caught.
    let (affine, interval) = expr_x_minus_x(1.0, 3.0);
    let r = residual(&affine, &interval);
    assert!(r <= 1.0e-300, "x - x residual {r} exceeds the 1e-300 pin");
}

#[test]
fn pin_x_plus_y_minus_x() {
    // Correlation win: the `x` cancels, leaving `w(y)`; interval pays
    // `2 w(x) + w(y)`. Measured residual 0.500 (win ~2.0x). Pinned at 0.55: 10%
    // headroom over the measured value, tripping only if the cancellation
    // degrades by more than a tenth.
    let (affine, interval) = expr_x_plus_y_minus_x(1.0, 3.0, 2.0, 6.0);
    let r = residual(&affine, &interval);
    assert!(r <= 0.55, "(x+y)-x residual {r} exceeds the 0.55 pin");
}

#[test]
fn pin_x_times_one_minus_x() {
    // Correlation win: the affine multiply sees the shared symbol on both
    // anticorrelated factors and cancels the leading term; interval keeps the
    // whole product box. Measured residual 0.550 (win ~1.82x). Pinned at 0.60:
    // ~9% headroom.
    let (affine, interval) = expr_x_times_one_minus_x(0.1, 0.6);
    let r = residual(&affine, &interval);
    assert!(r <= 0.60, "x(1-x) residual {r} exceeds the 0.60 pin");
}

#[test]
fn pin_x_squared_minus_2x() {
    // Correlation win: over `[1, 4]` the range straddles the vertex `x = 1`,
    // where the square and the linear term fight; interval forgets they move
    // together and keeps both spreads, affine cancels the shared symbol.
    // Measured residual 0.536 (win ~1.87x). Pinned at 0.59: ~10% headroom.
    let (affine, interval) = expr_x_squared_minus_2x(1.0, 4.0);
    let r = residual(&affine, &interval);
    assert!(r <= 0.59, "x*x-2x residual {r} exceeds the 0.59 pin");
}

#[test]
fn pin_x_over_one_plus_x() {
    // Correlation win: numerator and denominator share `x`, so the affine route
    // cancels the shared motion of a correlated quotient chain; interval treats
    // them as independent. Measured residual 0.543 (win ~1.84x). Pinned at 0.60:
    // ~10% headroom.
    let (affine, interval) = expr_x_over_one_plus_x(1.0, 5.0);
    let r = residual(&affine, &interval);
    assert!(r <= 0.60, "x/(1+x) residual {r} exceeds the 0.60 pin");
}

/// The documented non-goal: an uncorrelated multiply where interval arithmetic
/// is tighter than affine. Pinned as expected behavior so nobody later "fixes"
/// it and so the lane never claims universal affine superiority.
#[test]
fn non_goal_uncorrelated_mul_interval_is_tighter() {
    // `x * y` with independent `x, y` in `[1, 3]`: the true product range is
    // `[1, 9]`, which the interval four-corner rule reports exactly. Affine
    // cannot: its bilinear remainder is bounded by `radius(x) * radius(y)` over
    // `[-1, 1]`, a conservative box that widens the result to `[-1, 9]`, so it
    // is both wider and loses the true lower bound. There is no shared symbol to
    // cancel, so affine has nothing to trade its remainder against; this is the
    // structural loss class the four-corner product avoids.
    let (affine, interval) = {
        let affine = with_source(|mut src| {
            let x = form_of(1.0, 3.0, &mut src);
            let y = form_of(1.0, 3.0, &mut src);
            x.mul(&y, &mut src).reduce()
        });
        let ivx = Interval::new(1.0, 3.0).expect("ordered");
        let ivy = Interval::new(1.0, 3.0).expect("ordered");
        (affine, ivx * ivy)
    };
    assert!(
        width(&interval) < width(&affine),
        "interval width {} not below affine width {}",
        width(&interval),
        width(&affine)
    );
    // Measured residual 1.25 (interval is tighter, so affine/interval > 1).
    // Pinned at >= 1.15: ~8% below the measured value, so a benign change does
    // not trip it, but affine becoming as tight as interval here (residual
    // toward 1) would, flagging that the documented non-goal has changed.
    let r = residual(&affine, &interval);
    assert!(
        r >= 1.15,
        "uncorrelated x*y residual {r} fell below the 1.15 pin"
    );
}
