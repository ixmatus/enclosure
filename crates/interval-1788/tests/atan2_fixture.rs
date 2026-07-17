//! Tests for the two-argument arctangent `atan2` over the `f64` fixture, bare
//! and decorated.
//!
//! The bare block transcribes the WHOLE dedicated `atan2.itl` vector file: it
//! exists in the ITF1788 corpus precisely because independent implementations
//! got the quadrant corners, the branch cut, and the origin wrong, so it is the
//! acceptance authority for the corner table derived in `src/inverse.rs`. The
//! decorated block adds the representative branch-cut and origin cases from
//! `libieeep1788_elem.itl` that pin the `com`/`dac`/`def`/`trv` transitions.
//!
//! The fixture is sound-not-tight, so a finite corner result is generally wider
//! than the correctly-rounded vector; the transcription asserts the SOUND
//! RELATIONSHIP (the result encloses the vector interval). The full-range,
//! empty, and origin cases are structurally exact. Decorations are checked
//! exactly (the geometric predicates the local decoration reads are computed
//! exactly on the endpoints).
//!
//! `atan2.itl` carries its own notice: Copyright 2015-2016 Oliver Heimlich;
//! copying and distribution permitted in any medium without royalty provided the
//! notice is preserved, offered as-is without warranty. The elem cases are
//! Apache-2.0 (original author Marco Nehmeier, ITL port Oliver Heimlich). See
//! `docs/references/vendor/itf1788-framework/`.

use interval_1788::{DecoratedInterval, Decoration, Interval};

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

/// Scale `m` by `2^e` exactly by repeated exact doubling/halving.
fn mul_pow2(m: f64, mut e: i32) -> f64 {
    let mut r = m;
    while e > 0 {
        r *= 2.0;
        e -= 1;
    }
    while e < 0 {
        r *= 0.5;
        e += 1;
    }
    r
}

/// Parse a C hex-float literal to the exact `f64` it names.
fn hx(s: &str) -> f64 {
    let s = s.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .expect("hex-float must start with 0x");
    let p = s.find(['p', 'P']).expect("hex-float needs a p exponent");
    let (mant, exp) = (&s[..p], &s[p + 1..]);
    let exp: i32 = exp.parse().expect("hex-float exponent");
    let (int_part, frac_part) = match mant.find('.') {
        Some(i) => (&mant[..i], &mant[i + 1..]),
        None => (mant, ""),
    };
    let mut m: u64 = 0;
    for c in int_part.chars() {
        m = m * 16 + u64::from(c.to_digit(16).expect("hex digit"));
    }
    let mut frac_len = 0i32;
    for c in frac_part.chars() {
        m = m * 16 + u64::from(c.to_digit(16).expect("hex digit"));
        frac_len += 1;
    }
    #[allow(clippy::cast_precision_loss)]
    let val = mul_pow2(m as f64, exp - 4 * frac_len);
    if neg {
        -val
    } else {
        val
    }
}

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

/// `atan2(Y, X)` at the interval layer: `Y.atan2(X)`, with `Y` the ordinate.
fn atan2(yl: f64, yh: f64, xl: f64, xh: f64) -> Interval<f64> {
    iv(yl, yh).atan2(iv(xl, xh))
}

/// The sound relationship: `result` encloses the vector interval `[lo, hi]`.
fn encloses(result: Interval<f64>, lo: f64, hi: f64) {
    assert!(
        result.inf() <= lo && hi <= result.sup(),
        "expected enclosure of [{lo}, {hi}], got [{}, {}]",
        result.inf(),
        result.sup()
    );
}

// The pi-multiple brackets the vectors use, parsed from their hex forms.
fn pi_lo() -> f64 {
    hx("0x1.921FB54442D18P+1")
}
fn pi_up() -> f64 {
    hx("0x1.921FB54442D19P+1")
}
fn hpi_lo() -> f64 {
    hx("0x1.921FB54442D18P+0")
}
fn hpi_up() -> f64 {
    hx("0x1.921FB54442D19P+0")
}
fn qpi_lo() -> f64 {
    hx("0x1.921FB54442D18P-1")
}
fn qpi_up() -> f64 {
    hx("0x1.921FB54442D19P-1")
}
fn tqpi_lo() -> f64 {
    hx("0x1.2D97C7F3321D2P+1")
}
fn tqpi_up() -> f64 {
    hx("0x1.2D97C7F3321D3P+1")
}

// --- The whole atan2.itl ---------------------------------------------------

#[test]
fn atan2_itl_empty_and_full_range() {
    // Empty operands absorb.
    assert!(Interval::<f64>::empty().atan2(Interval::empty()).is_empty());
    assert!(Interval::<f64>::empty()
        .atan2(Interval::entire())
        .is_empty());
    assert!(Interval::<f64>::entire()
        .atan2(Interval::empty())
        .is_empty());
    // atan2 [0,0] [0,0] = empty (the origin, undefined).
    assert!(atan2(0.0, 0.0, 0.0, 0.0).is_empty());
    // atan2 [entire] [entire] = [-pi_up, pi_up].
    encloses(
        Interval::<f64>::entire().atan2(Interval::entire()),
        -pi_up(),
        pi_up(),
    );
    // The branch-cut full-range cases.
    let tiny = hx("0x1p-1022");
    encloses(atan2(-tiny, 0.0, -tiny, -tiny), -pi_up(), pi_up());
    encloses(atan2(-tiny, tiny, -tiny, -tiny), -pi_up(), pi_up());
    encloses(atan2(-2.0, 2.0, -3.0, -1.0), -pi_up(), pi_up());
    encloses(atan2(-2.0, 0.0, -3.0, -1.0), -pi_up(), pi_up());
    encloses(atan2(-5.0, 0.0, -5.0, 0.0), -pi_up(), pi_up());
}

#[test]
fn atan2_itl_axis_cases() {
    // atan2 [0,0] [-inf,0] = [pi_lo, pi_up].
    encloses(atan2(0.0, 0.0, NEG_INF, 0.0), pi_lo(), pi_up());
    // atan2 [0,0] [0,inf] = [0, 0].
    encloses(atan2(0.0, 0.0, 0.0, INF), 0.0, 0.0);
    // atan2 [0,inf] [0,0] = [pi/2_lo, pi/2_up].
    encloses(atan2(0.0, INF, 0.0, 0.0), hpi_lo(), hpi_up());
    // atan2 [-inf,0] [0,0] = [-pi/2_up, -pi/2_lo].
    encloses(atan2(NEG_INF, 0.0, 0.0, 0.0), -hpi_up(), -hpi_lo());
    // atan2 [0,5] [-5,0] = [pi/2_lo, pi_up].
    encloses(atan2(0.0, 5.0, -5.0, 0.0), hpi_lo(), pi_up());
    // atan2 [0,5] [0,5] = [0, pi/2_up].
    encloses(atan2(0.0, 5.0, 0.0, 5.0), 0.0, hpi_up());
    // atan2 [-5,0] [0,5] = [-pi/2_up, 0].
    encloses(atan2(-5.0, 0.0, 0.0, 5.0), -hpi_up(), 0.0);
}

#[test]
fn atan2_itl_unit_and_tiny_quadrants() {
    let tiny = hx("0x1p-1022");
    // The four unit-diagonal corners: +-pi/4, +-3pi/4.
    encloses(atan2(1.0, 1.0, -1.0, -1.0), tqpi_lo(), tqpi_up());
    encloses(atan2(1.0, 1.0, 1.0, 1.0), qpi_lo(), qpi_up());
    encloses(atan2(-1.0, -1.0, 1.0, 1.0), -qpi_up(), -qpi_lo());
    encloses(atan2(-1.0, -1.0, -1.0, -1.0), -tqpi_up(), -tqpi_lo());
    // Tiny straddles of a single axis (continuous arcs, no cut crossing).
    encloses(atan2(-tiny, tiny, tiny, tiny), -qpi_up(), qpi_up());
    encloses(atan2(-tiny, -tiny, -tiny, tiny), -tqpi_up(), -qpi_lo());
    encloses(atan2(tiny, tiny, -tiny, tiny), qpi_lo(), tqpi_up());
}

#[test]
fn atan2_itl_general_boxes() {
    // The remaining general finite boxes of atan2.itl.
    encloses(
        atan2(0.0, 2.0, -3.0, -1.0),
        hx("0x1.0468A8ACE4DF6p1"),
        pi_up(),
    );
    encloses(
        atan2(1.0, 3.0, -3.0, -1.0),
        hx("0x1.E47DF3D0DD4Dp0"),
        hx("0x1.68F095FDF593Dp1"),
    );
    encloses(
        atan2(1.0, 3.0, -2.0, 0.0),
        hpi_lo(),
        hx("0x1.56C6E7397F5AFp1"),
    );
    encloses(
        atan2(1.0, 3.0, -2.0, 2.0),
        hx("0x1.DAC670561BB4Fp-2"),
        hx("0x1.56C6E7397F5AFp1"),
    );
    encloses(
        atan2(1.0, 3.0, 0.0, 2.0),
        hx("0x1.DAC670561BB4Fp-2"),
        hpi_up(),
    );
    encloses(
        atan2(1.0, 3.0, 1.0, 3.0),
        hx("0x1.4978FA3269EE1p-2"),
        hx("0x1.3FC176B7A856p0"),
    );
    encloses(atan2(0.0, 2.0, 1.0, 3.0), 0.0, hx("0x1.1B6E192EBBE45p0"));
    encloses(
        atan2(-2.0, 2.0, 1.0, 3.0),
        -hx("0x1.1B6E192EBBE45p0"),
        hx("0x1.1B6E192EBBE45p0"),
    );
    encloses(atan2(-2.0, 0.0, 1.0, 3.0), -hx("0x1.1B6E192EBBE45p0"), 0.0);
    encloses(
        atan2(-3.0, -1.0, 1.0, 3.0),
        -hx("0x1.3FC176B7A856p0"),
        -hx("0x1.4978FA3269EE1p-2"),
    );
    encloses(
        atan2(-3.0, -1.0, 0.0, 2.0),
        -hpi_up(),
        -hx("0x1.DAC670561BB4Fp-2"),
    );
    encloses(
        atan2(-3.0, -1.0, -2.0, 2.0),
        -hx("0x1.56C6E7397F5AFp1"),
        -hx("0x1.DAC670561BB4Fp-2"),
    );
    encloses(
        atan2(-3.0, -1.0, -2.0, 0.0),
        -hx("0x1.56C6E7397F5AFp1"),
        -hpi_lo(),
    );
    encloses(
        atan2(-3.0, -1.0, -3.0, -1.0),
        -hx("0x1.68F095FDF593Dp1"),
        -hx("0x1.E47DF3D0DD4Dp0"),
    );
}

// --- Representative decorated cases from libieeep1788_elem.itl -------------

fn di(
    yl: f64,
    yh: f64,
    yd: Decoration,
    xl: f64,
    xh: f64,
    xd: Decoration,
) -> DecoratedInterval<f64> {
    let y = DecoratedInterval::set_dec(iv(yl, yh), yd);
    let x = DecoratedInterval::set_dec(iv(xl, xh), xd);
    y.atan2(x)
}

#[test]
fn atan2_decorated_com_and_dac() {
    // Q4, bounded, no cut, no origin -> com.
    let r = di(-2.0, -0.1, Decoration::Com, 0.1, 1.0, Decoration::Com);
    assert_eq!(r.decoration(), Decoration::Com);
    // X strictly positive, unbounded ordinate input -> dac (continuous, no cut).
    let r = di(NEG_INF, INF, Decoration::Dac, 0.1, 1.0, Decoration::Com);
    assert_eq!(r.decoration(), Decoration::Dac);
    // Ordinate {0}, X strictly positive -> com.
    let r = di(0.0, 0.0, Decoration::Com, 0.1, 1.0, Decoration::Com);
    assert_eq!(r.decoration(), Decoration::Com);
    encloses(r.interval(), 0.0, 0.0);
}

#[test]
fn atan2_decorated_branch_cut_touch_is_dac() {
    // Ordinate [0, 1] touches the cut from above (yl = 0), X strictly negative:
    // continuous restriction but on the discontinuity locus -> dac (not com).
    let r = di(-0.0, 1.0, Decoration::Com, -2.0, -0.1, Decoration::Com);
    assert_eq!(r.decoration(), Decoration::Dac);
    // Ordinate {0}, X strictly negative -> dac, image the single value pi.
    let r = di(0.0, 0.0, Decoration::Com, -2.0, -0.1, Decoration::Dac);
    assert_eq!(r.decoration(), Decoration::Dac);
    encloses(r.interval(), pi_lo(), pi_up());
}

#[test]
fn atan2_decorated_branch_cut_crossing_is_def() {
    // Ordinate [-2, 0] reaches below zero, X strictly negative: the +pi value at
    // y = 0 and the -pi limit both enter, a discontinuity -> def, full range.
    let r = di(-2.0, 0.0, Decoration::Com, -2.0, -0.1, Decoration::Dac);
    assert_eq!(r.decoration(), Decoration::Def);
    encloses(r.interval(), -pi_up(), pi_up());
    // Entire ordinate, X strictly negative -> def as well.
    let r = di(NEG_INF, INF, Decoration::Dac, -2.0, -0.1, Decoration::Com);
    assert_eq!(r.decoration(), Decoration::Def);
    encloses(r.interval(), -pi_up(), pi_up());
}

#[test]
fn atan2_decorated_origin_is_trv() {
    // Box contains the origin (0 in both operands) -> trv.
    let r = di(-2.0, 1.0, Decoration::Dac, -2.0, 1.0, Decoration::Com);
    assert_eq!(r.decoration(), Decoration::Trv);
    encloses(r.interval(), -pi_up(), pi_up());
    // Origin via X = {0} with the ordinate spanning zero.
    let r = di(-2.0, -0.0, Decoration::Com, 0.0, 0.0, Decoration::Com);
    assert_eq!(r.decoration(), Decoration::Trv);
    // NaI poisons.
    assert!(DecoratedInterval::<f64>::nai()
        .atan2(DecoratedInterval::set_dec(iv(1.0, 2.0), Decoration::Com))
        .is_nai());
}
