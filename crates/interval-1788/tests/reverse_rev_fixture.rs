//! Tests for the reverse operations over the `f64` fixture, hand-translated from
//! ITF1788 `libieeep1788_rev.itl` (Apache-2.0; original author Marco Nehmeier,
//! ITL port Oliver Heimlich; see `docs/references/vendor/itf1788-framework/`).
//!
//! This file covers `sqrRev`, `absRev`, `pownRev`, `sinRev`, `cosRev`, `tanRev`,
//! `coshRev`, `mulRev`, and the ternary `mulRevTen`. The ITL's candidate-less
//! forms (`sqrRev [c]`, ...) map to the crate's single method at `x = entire`,
//! and the `*Bin`/`*Ten` forms carry the explicit candidate.
//!
//! Exact versus enclosure, per decision record 0006:
//!
//! - EXACT: every structural case (empty, entire, domain-clipped empty), and
//!   `absRev` (never rounds) and the `pownRev` exponent-1 and integer-endpoint
//!   arms (the root is the identity or is attained exactly). Asserted by
//!   `is_empty` / `is_entire` or exact endpoints.
//! - ENCLOSURE: every arm carrying a directed `sqrt`, integer root, division,
//!   `acosh`, or inverse-trig image over the sound-not-tight fixture. The result
//!   is asserted to enclose the vector interval, the sound relationship the
//!   fixture guarantees.
//!
//! The decorated testcases pin one rule uniformly (decision record 0006 part 5):
//! every reverse grades `trv`, with `NaI` propagating. Each family checks that
//! rule on a representative sample rather than re-listing the full value grid the
//! bare tests already pin.

use interval_1788::{DecoratedInterval, Decoration, Interval};

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

fn entire() -> Interval<f64> {
    Interval::entire()
}

fn empty() -> Interval<f64> {
    Interval::empty()
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
    let mut val = m as f64;
    let mut e = exp - 4 * frac_len;
    while e > 0 {
        val *= 2.0;
        e -= 1;
    }
    while e < 0 {
        val *= 0.5;
        e += 1;
    }
    if neg {
        -val
    } else {
        val
    }
}

/// `result` encloses the vector interval `[lo, hi]`.
fn encloses(result: Interval<f64>, lo: f64, hi: f64) {
    assert!(
        !result.is_empty() && result.inf() <= lo && hi <= result.sup(),
        "expected enclosure of [{lo}, {hi}], got [{}, {}]",
        result.inf(),
        result.sup()
    );
}

/// `result` is exactly `[lo, hi]`.
fn exact(result: Interval<f64>, lo: f64, hi: f64) {
    assert!(
        result.inf() == lo && result.sup() == hi,
        "expected exactly [{lo}, {hi}], got [{}, {}]",
        result.inf(),
        result.sup()
    );
}

// --- sqrRev -----------------------------------------------------------------

#[test]
fn sqr_rev_vectors() {
    assert!(empty().sqr_rev(entire()).is_empty());
    assert!(iv(-10.0, -1.0).sqr_rev(entire()).is_empty());
    assert!(iv(0.0, INF).sqr_rev(entire()).is_entire());
    encloses(iv(0.0, 1.0).sqr_rev(entire()), -1.0, 1.0);
    encloses(iv(-0.5, 1.0).sqr_rev(entire()), -1.0, 1.0);
    encloses(iv(-1000.0, 1.0).sqr_rev(entire()), -1.0, 1.0);
    encloses(iv(0.0, 25.0).sqr_rev(entire()), -5.0, 5.0);
    encloses(iv(-1.0, 25.0).sqr_rev(entire()), -5.0, 5.0);
    encloses(
        iv(hx("0X1.47AE147AE147BP-7"), hx("0X1.47AE147AE147CP-7")).sqr_rev(entire()),
        hx("-0X1.999999999999BP-4"),
        hx("0X1.999999999999BP-4"),
    );
    encloses(
        iv(0.0, hx("0X1.FFFFFFFFFFFE1P+1")).sqr_rev(entire()),
        hx("-0x1.ffffffffffff1p+0"),
        hx("0x1.ffffffffffff1p+0"),
    );
}

#[test]
fn sqr_rev_bin_vectors() {
    assert!(empty().sqr_rev(iv(-5.0, 1.0)).is_empty());
    assert!(iv(-10.0, -1.0).sqr_rev(iv(-5.0, 1.0)).is_empty());
    encloses(iv(0.0, INF).sqr_rev(iv(-5.0, 1.0)), -5.0, 1.0);
    encloses(iv(0.0, 1.0).sqr_rev(iv(-0.1, 1.0)), -0.1, 1.0);
    encloses(iv(-0.5, 1.0).sqr_rev(iv(-0.1, 1.0)), -0.1, 1.0);
    encloses(iv(-1000.0, 1.0).sqr_rev(iv(-0.1, 1.0)), -0.1, 1.0);
    encloses(iv(0.0, 25.0).sqr_rev(iv(-4.1, 6.0)), -4.1, 5.0);
    encloses(iv(-1.0, 25.0).sqr_rev(iv(-4.1, 7.0)), -4.1, 5.0);
    encloses(iv(1.0, 25.0).sqr_rev(iv(0.0, 7.0)), 1.0, 5.0);
    encloses(
        iv(hx("0X1.47AE147AE147BP-7"), hx("0X1.47AE147AE147CP-7")).sqr_rev(iv(-0.1, INF)),
        -0.1,
        hx("0X1.999999999999BP-4"),
    );
    encloses(
        iv(0.0, hx("0X1.FFFFFFFFFFFE1P+1")).sqr_rev(iv(-0.1, INF)),
        -0.1,
        hx("0x1.ffffffffffff1p+0"),
    );
}

#[test]
fn sqr_rev_decorated() {
    // Every reverse grades trv; NaI propagates.
    assert_eq!(
        DecoratedInterval::set_dec(iv(0.0, 1.0), Decoration::Def)
            .sqr_rev(DecoratedInterval::new_dec(entire()))
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .sqr_rev(DecoratedInterval::new_dec(entire()))
        .is_nai());
    let r = DecoratedInterval::set_dec(iv(-10.0, -1.0), Decoration::Com)
        .sqr_rev(DecoratedInterval::new_dec(entire()));
    assert_eq!(r.decoration(), Decoration::Trv);
    assert!(r.interval().is_empty());
}

// --- absRev -----------------------------------------------------------------

#[test]
fn abs_rev_vectors() {
    assert!(empty().abs_rev(entire()).is_empty());
    assert!(iv(-1.1, -0.4).abs_rev(entire()).is_empty());
    assert!(iv(0.0, INF).abs_rev(entire()).is_entire());
    exact(iv(1.1, 2.1).abs_rev(entire()), -2.1, 2.1);
    exact(iv(-1.1, 2.0).abs_rev(entire()), -2.0, 2.0);
    exact(iv(-1.1, 0.0).abs_rev(entire()), 0.0, 0.0);
    exact(iv(-1.9, 0.2).abs_rev(entire()), -0.2, 0.2);
    exact(iv(0.0, 0.2).abs_rev(entire()), -0.2, 0.2);
    assert!(iv(-1.5, INF).abs_rev(entire()).is_entire());
}

#[test]
fn abs_rev_bin_vectors() {
    assert!(empty().abs_rev(iv(-1.1, 5.0)).is_empty());
    assert!(iv(-1.1, -0.4).abs_rev(iv(-1.1, 5.0)).is_empty());
    exact(iv(0.0, INF).abs_rev(iv(-1.1, 5.0)), -1.1, 5.0);
    exact(iv(1.1, 2.1).abs_rev(iv(-1.0, 5.0)), 1.1, 2.1);
    exact(iv(-1.1, 2.0).abs_rev(iv(-1.1, 5.0)), -1.1, 2.0);
    exact(iv(-1.1, 0.0).abs_rev(iv(-1.1, 5.0)), 0.0, 0.0);
    exact(iv(-1.9, 0.2).abs_rev(iv(-1.1, 5.0)), -0.2, 0.2);
}

#[test]
fn abs_rev_decorated() {
    assert_eq!(
        DecoratedInterval::set_dec(iv(1.1, 2.1), Decoration::Com)
            .abs_rev(DecoratedInterval::new_dec(entire()))
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .abs_rev(DecoratedInterval::new_dec(entire()))
        .is_nai());
}

// --- pownRev ----------------------------------------------------------------

#[test]
fn pown_rev_exponent_zero() {
    assert!(empty().pown_rev(entire(), 0).is_empty());
    assert!(iv(1.0, 1.0).pown_rev(entire(), 0).is_entire());
    assert!(iv(-1.0, 5.0).pown_rev(entire(), 0).is_entire());
    assert!(iv(-1.0, 0.0).pown_rev(entire(), 0).is_empty());
    assert!(iv(-1.0, -0.0).pown_rev(entire(), 0).is_empty());
    assert!(iv(1.1, 10.0).pown_rev(entire(), 0).is_empty());
}

#[test]
fn pown_rev_exponent_one() {
    // n == 1 is the identity, so every arm is exact.
    assert!(empty().pown_rev(entire(), 1).is_empty());
    assert!(iv(NEG_INF, INF).pown_rev(entire(), 1).is_entire());
    exact(iv(0.0, 0.0).pown_rev(entire(), 1), 0.0, 0.0);
    exact(iv(-0.0, -0.0).pown_rev(entire(), 1), 0.0, 0.0);
    exact(iv(13.1, 13.1).pown_rev(entire(), 1), 13.1, 13.1);
    exact(
        iv(-7451.145, -7451.145).pown_rev(entire(), 1),
        -7451.145,
        -7451.145,
    );
    let big = hx("0x1.FFFFFFFFFFFFFp1023");
    exact(iv(big, big).pown_rev(entire(), 1), big, big);
    exact(iv(-big, -big).pown_rev(entire(), 1), -big, -big);
    exact(iv(0.0, INF).pown_rev(entire(), 1), 0.0, INF);
    exact(iv(-0.0, INF).pown_rev(entire(), 1), 0.0, INF);
    exact(iv(NEG_INF, 0.0).pown_rev(entire(), 1), NEG_INF, 0.0);
    exact(iv(NEG_INF, -0.0).pown_rev(entire(), 1), NEG_INF, 0.0);
    exact(iv(-324.3, 2.5).pown_rev(entire(), 1), -324.3, 2.5);
    exact(iv(0.01, 2.33).pown_rev(entire(), 1), 0.01, 2.33);
    exact(iv(-1.9, -0.33).pown_rev(entire(), 1), -1.9, -0.33);
}

#[test]
fn pown_rev_positive_even() {
    assert!(empty().pown_rev(entire(), 2).is_empty());
    assert!(iv(-5.0, -1.0).pown_rev(entire(), 2).is_empty());
    assert!(iv(0.0, INF).pown_rev(entire(), 2).is_entire());
    assert!(iv(-0.0, INF).pown_rev(entire(), 2).is_entire());
    exact(iv(0.0, 0.0).pown_rev(entire(), 2), 0.0, 0.0);
    exact(iv(-0.0, -0.0).pown_rev(entire(), 2), 0.0, 0.0);
    encloses(
        iv(hx("0X1.573851EB851EBP+7"), hx("0X1.573851EB851ECP+7")).pown_rev(entire(), 2),
        hx("-0x1.a333333333334p+3"),
        hx("0x1.a333333333334p+3"),
    );
    encloses(
        iv(0.0, hx("0X1.9AD27D70A3D72P+16")).pown_rev(entire(), 2),
        hx("-0x1.444cccccccccep+8"),
        hx("0x1.444cccccccccep+8"),
    );
    encloses(
        iv(-0.0, hx("0X1.9AD27D70A3D72P+16")).pown_rev(entire(), 2),
        hx("-0x1.444cccccccccep+8"),
        hx("0x1.444cccccccccep+8"),
    );
    encloses(
        iv(hx("0X1.BE0DED288CE7P-4"), hx("0X1.CE147AE147AE1P+1")).pown_rev(entire(), 2),
        hx("-0x1.e666666666667p+0"),
        hx("0x1.e666666666667p+0"),
    );
    // n = 8.
    assert!(iv(0.0, INF).pown_rev(entire(), 8).is_entire());
    exact(iv(0.0, 0.0).pown_rev(entire(), 8), 0.0, 0.0);
    encloses(
        iv(hx("0X1.9D8FD495853F5P+29"), hx("0X1.9D8FD495853F6P+29")).pown_rev(entire(), 8),
        hx("-0x1.a333333333334p+3"),
        hx("0x1.a333333333334p+3"),
    );
    encloses(
        iv(0.0, hx("0X1.A87587109655P+66")).pown_rev(entire(), 8),
        hx("-0x1.444cccccccccep+8"),
        hx("0x1.444cccccccccep+8"),
    );
}

#[test]
fn pown_rev_positive_odd() {
    assert!(iv(NEG_INF, INF).pown_rev(entire(), 3).is_entire());
    exact(iv(0.0, 0.0).pown_rev(entire(), 3), 0.0, 0.0);
    encloses(
        iv(hx("0X1.1902E978D4FDEP+11"), hx("0X1.1902E978D4FDFP+11")).pown_rev(entire(), 3),
        hx("0x1.a333333333332p+3"),
        hx("0x1.a333333333334p+3"),
    );
    encloses(
        iv(hx("-0X1.81460637B9A3DP+38"), hx("-0X1.81460637B9A3CP+38")).pown_rev(entire(), 3),
        hx("-0x1.d1b251eb851edp+12"),
        hx("-0x1.d1b251eb851ebp+12"),
    );
    exact(iv(0.0, INF).pown_rev(entire(), 3), 0.0, INF);
    exact(iv(NEG_INF, 0.0).pown_rev(entire(), 3), NEG_INF, 0.0);
    encloses(
        iv(hx("-0X1.0436D2F418938P+25"), hx("0X1.F4P+3")).pown_rev(entire(), 3),
        hx("-0x1.444cccccccccep+8"),
        hx("0x1.4p+1"),
    );
    encloses(
        iv(hx("-0X1.B6F9DB22D0E55P+2"), hx("-0X1.266559F6EC5B1P-5")).pown_rev(entire(), 3),
        hx("-0x1.e666666666667p+0"),
        hx("-0x1.51eb851eb851ep-2"),
    );
    // n = 7.
    assert!(iv(NEG_INF, INF).pown_rev(entire(), 7).is_entire());
    encloses(
        iv(hx("0X1.F91D1B185493BP+25"), hx("0X1.F91D1B185493CP+25")).pown_rev(entire(), 7),
        hx("0x1.a333333333332p+3"),
        hx("0x1.a333333333334p+3"),
    );
}

#[test]
fn pown_rev_negative_even() {
    assert!(empty().pown_rev(entire(), -2).is_empty());
    assert!(iv(0.0, INF).pown_rev(entire(), -2).is_entire());
    assert!(iv(-0.0, INF).pown_rev(entire(), -2).is_entire());
    assert!(iv(0.0, 0.0).pown_rev(entire(), -2).is_empty());
    assert!(iv(-0.0, -0.0).pown_rev(entire(), -2).is_empty());
    assert!(iv(-10.0, 0.0).pown_rev(entire(), -2).is_empty());
    assert!(iv(-10.0, -0.0).pown_rev(entire(), -2).is_empty());
    encloses(
        iv(hx("0X1.7DE3A077D1568P-8"), hx("0X1.7DE3A077D1569P-8")).pown_rev(entire(), -2),
        hx("-0x1.a333333333334p+3"),
        hx("0x1.a333333333334p+3"),
    );
    assert!(iv(hx("0X0P+0"), hx("0X0.0000000000001P-1022"))
        .pown_rev(entire(), -2)
        .is_entire());
    encloses(
        iv(hx("0X1.3F0C482C977C9P-17"), INF).pown_rev(entire(), -2),
        hx("-0x1.444cccccccccep+8"),
        hx("0x1.444cccccccccep+8"),
    );
    // n = -8.
    assert!(iv(0.0, INF).pown_rev(entire(), -8).is_entire());
    assert!(iv(0.0, 0.0).pown_rev(entire(), -8).is_empty());
}

#[test]
fn pown_rev_negative_odd() {
    assert!(empty().pown_rev(entire(), -1).is_empty());
    assert!(iv(NEG_INF, INF).pown_rev(entire(), -1).is_entire());
    assert!(iv(0.0, 0.0).pown_rev(entire(), -1).is_empty());
    assert!(iv(-0.0, -0.0).pown_rev(entire(), -1).is_empty());
    encloses(
        iv(hx("0X1.38ABF82EE6986P-4"), hx("0X1.38ABF82EE6987P-4")).pown_rev(entire(), -1),
        hx("0x1.a333333333332p+3"),
        hx("0x1.a333333333335p+3"),
    );
    // The set-level reciprocal steps a signed zero outward by one subnormal on
    // the fixture, so the half-unbounded arms enclose rather than land exact.
    encloses(iv(0.0, INF).pown_rev(entire(), -1), 0.0, INF);
    encloses(iv(-0.0, INF).pown_rev(entire(), -1), 0.0, INF);
    encloses(iv(NEG_INF, 0.0).pown_rev(entire(), -1), NEG_INF, 0.0);
    encloses(
        iv(hx("0X1.B77C278DBBE13P-2"), hx("0X1.9P+6")).pown_rev(entire(), -1),
        hx("0x1.47ae147ae147ap-7"),
        hx("0x1.2a3d70a3d70a5p+1"),
    );
    // n = -3.
    assert!(iv(NEG_INF, INF).pown_rev(entire(), -3).is_entire());
    assert!(iv(0.0, 0.0).pown_rev(entire(), -3).is_empty());
    encloses(iv(0.0, INF).pown_rev(entire(), -3), 0.0, INF);
    encloses(iv(NEG_INF, 0.0).pown_rev(entire(), -3), NEG_INF, 0.0);
}

#[test]
fn pown_rev_bin_vectors() {
    assert!(empty().pown_rev(iv(1.0, 1.0), 0).is_empty());
    exact(iv(1.0, 1.0).pown_rev(iv(1.0, 1.0), 0), 1.0, 1.0);
    exact(iv(-1.0, 5.0).pown_rev(iv(-51.0, 12.0), 0), -51.0, 12.0);
    assert!(iv(-1.0, 0.0).pown_rev(iv(5.0, 10.0), 0).is_empty());
    assert!(iv(1.1, 10.0).pown_rev(iv(1.0, 41.0), 0).is_empty());
    // n = 1.
    assert!(empty().pown_rev(iv(0.0, 100.1), 1).is_empty());
    exact(iv(NEG_INF, INF).pown_rev(iv(-5.1, 10.0), 1), -5.1, 10.0);
    exact(iv(0.0, 0.0).pown_rev(iv(-10.0, 5.1), 1), 0.0, 0.0);
    assert!(iv(-0.0, -0.0).pown_rev(iv(1.0, 5.0), 1).is_empty());
    // n = 2.
    assert!(iv(-5.0, -1.0).pown_rev(iv(5.0, 17.1), 2).is_empty());
    encloses(iv(0.0, INF).pown_rev(iv(5.6, 27.544), 2), 5.6, 27.544);
    assert!(iv(0.0, 0.0).pown_rev(iv(1.0, 2.0), 2).is_empty());
    encloses(
        iv(hx("0X1.A36E2EB1C432CP-14"), hx("0X1.5B7318FC50482P+2")).pown_rev(iv(1.0, INF), 2),
        1.0,
        hx("0x1.2a3d70a3d70a5p+1"),
    );
    encloses(
        iv(hx("0X1.BE0DED288CE7P-4"), hx("0X1.CE147AE147AE1P+1")).pown_rev(iv(NEG_INF, -1.0), 2),
        hx("-0x1.e666666666667p+0"),
        -1.0,
    );
    // n = 3.
    exact(iv(NEG_INF, INF).pown_rev(iv(-23.0, -1.0), 3), -23.0, -1.0);
    assert!(iv(0.0, 0.0).pown_rev(iv(1.0, 2.0), 3).is_empty());
    // n = -2.
    encloses(iv(0.0, INF).pown_rev(iv(-5.1, -0.1), -2), -5.1, -0.1);
    assert!(iv(0.0, 0.0).pown_rev(iv(27.2, 55.1), -2).is_empty());
    assert!(iv(hx("0X1.3F0C482C977C9P-17"), INF)
        .pown_rev(iv(NEG_INF, hx("-0x1.FFFFFFFFFFFFFp1023")), -2)
        .is_empty());
    // n = -1.
    exact(iv(NEG_INF, INF).pown_rev(iv(-5.1, 55.5), -1), -5.1, 55.5);
    assert!(iv(0.0, 0.0).pown_rev(iv(-5.1, 55.5), -1).is_empty());
    encloses(iv(NEG_INF, -0.0).pown_rev(iv(-1.0, 1.0), -1), -1.0, 0.0);
    assert!(iv(hx("0X1.B77C278DBBE13P-2"), hx("0X1.9P+6"))
        .pown_rev(iv(-1.0, 0.0), -1)
        .is_empty());
    // n = -3.
    encloses(iv(NEG_INF, -0.0).pown_rev(iv(-32.0, 1.1), -3), -32.0, 0.0);
    assert!(iv(NEG_INF, 0.0).pown_rev(iv(5.1, 55.5), -3).is_empty());
}

#[test]
fn pown_rev_decorated() {
    assert_eq!(
        DecoratedInterval::set_dec(iv(1.0, 1.0), Decoration::Com)
            .pown_rev(DecoratedInterval::new_dec(entire()), 0)
            .decoration(),
        Decoration::Trv
    );
    let r = DecoratedInterval::set_dec(iv(13.1, 13.1), Decoration::Def)
        .pown_rev(DecoratedInterval::new_dec(entire()), 1);
    assert_eq!(r.decoration(), Decoration::Trv);
    exact(r.interval(), 13.1, 13.1);
    assert!(DecoratedInterval::<f64>::nai()
        .pown_rev(DecoratedInterval::new_dec(entire()), 2)
        .is_nai());
}

// --- sinRev -----------------------------------------------------------------

#[test]
fn sin_rev_vectors() {
    assert!(empty().sin_rev(entire()).is_empty());
    assert!(iv(-2.0, -1.1).sin_rev(entire()).is_empty());
    assert!(iv(1.1, 2.0).sin_rev(entire()).is_empty());
    assert!(iv(-1.0, 1.0).sin_rev(entire()).is_entire());
    assert!(iv(0.0, 0.0).sin_rev(entire()).is_entire());
    assert!(iv(hx("0X1.1A62633145C06P-53"), hx("0X1.1A62633145C07P-53"))
        .sin_rev(entire())
        .is_entire());
}

#[test]
fn sin_rev_bin_vectors() {
    assert!(empty().sin_rev(iv(-1.2, 12.1)).is_empty());
    assert!(iv(-2.0, -1.1).sin_rev(iv(-5.0, 5.0)).is_empty());
    assert!(iv(1.1, 2.0).sin_rev(iv(-5.0, 5.0)).is_empty());
    encloses(iv(-1.0, 1.0).sin_rev(iv(-1.2, 12.1)), -1.2, 12.1);
    encloses(iv(0.0, 0.0).sin_rev(iv(-1.0, 1.0)), 0.0, 0.0);
    assert!(iv(-0.0, -0.0).sin_rev(iv(2.0, 2.5)).is_empty());
    encloses(
        iv(-0.0, -0.0).sin_rev(iv(3.0, 3.5)),
        hx("0x1.921fb54442d18p+1"),
        hx("0x1.921fb54442d19p+1"),
    );
    encloses(
        iv(hx("0X1.FFFFFFFFFFFFFP-1"), hx("0X1P+0")).sin_rev(iv(1.57, 1.58)),
        hx("0x1.921fb50442d18p+0"),
        hx("0x1.921fb58442d1ap+0"),
    );
    encloses(iv(0.0, hx("0X1P+0")).sin_rev(iv(-0.1, 1.58)), 0.0, 1.58);
    encloses(
        iv(hx("0X1.1A62633145C06P-53"), hx("0X1.1A62633145C07P-53")).sin_rev(iv(3.14, 3.15)),
        hx("0X1.921FB54442D17P+1"),
        hx("0X1.921FB54442D19P+1"),
    );
    encloses(
        iv(0.0, 1.0).sin_rev(iv(-0.1, 3.15)),
        0.0,
        hx("0X1.921FB54442D19P+1"),
    );
    // Unbounded candidate degrades to a sound enclosure (encloses the vector).
    encloses(
        iv(hx("0X1.1A62633145C06P-53"), hx("0X1.1A62633145C07P-53")).sin_rev(iv(NEG_INF, 3.15)),
        NEG_INF,
        hx("0X1.921FB54442D19P+1"),
    );
    encloses(
        iv(hx("-0X1.72CECE675D1FDP-52"), hx("-0X1.72CECE675D1FCP-52")).sin_rev(iv(3.14, INF)),
        hx("0X1.921FB54442D18P+1"),
        INF,
    );
}

#[test]
fn sin_rev_decorated() {
    assert_eq!(
        DecoratedInterval::set_dec(iv(-1.0, 1.0), Decoration::Com)
            .sin_rev(DecoratedInterval::new_dec(entire()))
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .sin_rev(DecoratedInterval::new_dec(iv(-1.2, 12.1)))
        .is_nai());
}

// --- cosRev -----------------------------------------------------------------

#[test]
fn cos_rev_vectors() {
    assert!(empty().cos_rev(entire()).is_empty());
    assert!(iv(-2.0, -1.1).cos_rev(entire()).is_empty());
    assert!(iv(1.1, 2.0).cos_rev(entire()).is_empty());
    assert!(iv(-1.0, 1.0).cos_rev(entire()).is_entire());
    assert!(iv(0.0, 0.0).cos_rev(entire()).is_entire());
}

#[test]
fn cos_rev_bin_vectors() {
    assert!(empty().cos_rev(iv(-1.2, 12.1)).is_empty());
    assert!(iv(-2.0, -1.1).cos_rev(iv(-5.0, 5.0)).is_empty());
    encloses(iv(-1.0, 1.0).cos_rev(iv(-1.2, 12.1)), -1.2, 12.1);
    encloses(iv(1.0, 1.0).cos_rev(iv(-0.1, 0.1)), 0.0, 0.0);
    encloses(
        iv(-1.0, -1.0).cos_rev(iv(3.14, 3.15)),
        hx("0x1.921fb54442d18p+1"),
        hx("0x1.921fb54442d1ap+1"),
    );
    encloses(
        iv(hx("0X1.1A62633145C06P-54"), hx("0X1.1A62633145C07P-54")).cos_rev(iv(1.57, 1.58)),
        hx("0X1.921FB54442D17P+0"),
        hx("0X1.921FB54442D19P+0"),
    );
    encloses(
        iv(hx("0X1.1A62633145C06P-54"), 1.0).cos_rev(iv(-2.0, 2.0)),
        hx("-0X1.921FB54442D19P+0"),
        hx("0X1.921FB54442D19P+0"),
    );
    encloses(
        iv(hx("0X1.87996529F9D92P-1"), 1.0).cos_rev(iv(-1.0, 0.1)),
        hx("-0x1.6666666666667p-1"),
        0.1,
    );
    // Unbounded candidate.
    encloses(
        iv(hx("0X1.1A62633145C06P-53"), hx("0X1.1A62633145C07P-53")).cos_rev(iv(NEG_INF, 1.58)),
        NEG_INF,
        hx("0X1.921FB54442D18P+0"),
    );
}

#[test]
fn cos_rev_decorated() {
    assert_eq!(
        DecoratedInterval::set_dec(iv(-1.0, 1.0), Decoration::Com)
            .cos_rev(DecoratedInterval::new_dec(entire()))
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .cos_rev(DecoratedInterval::new_dec(entire()))
        .is_nai());
}

// --- tanRev -----------------------------------------------------------------

#[test]
fn tan_rev_vectors() {
    assert!(empty().tan_rev(entire()).is_empty());
    assert!(iv(-1.0, 1.0).tan_rev(entire()).is_entire());
    assert!(iv(-156.0, -12.0).tan_rev(entire()).is_entire());
    assert!(iv(0.0, 0.0).tan_rev(entire()).is_entire());
}

#[test]
fn tan_rev_bin_vectors() {
    assert!(empty().tan_rev(iv(-1.5708, 1.5708)).is_empty());
    encloses(
        iv(NEG_INF, INF).tan_rev(iv(-1.5708, 1.5708)),
        -1.5708,
        1.5708,
    );
    encloses(iv(0.0, 0.0).tan_rev(iv(-1.5708, 1.5708)), 0.0, 0.0);
    encloses(
        iv(hx("0X1.D02967C31CDB4P+53"), hx("0X1.D02967C31CDB5P+53")).tan_rev(iv(-1.5708, 1.5708)),
        hx("-0x1.921fb54442d1bp+0"),
        hx("0x1.921fb54442d19p+0"),
    );
    encloses(
        iv(hx("-0X1.1A62633145C07P-53"), hx("-0X1.1A62633145C06P-53")).tan_rev(iv(3.14, 3.15)),
        hx("0X1.921FB54442D17P+1"),
        hx("0X1.921FB54442D19P+1"),
    );
    encloses(
        iv(hx("-0X1.D02967C31p+53"), hx("0X1.D02967C31p+53")).tan_rev(iv(NEG_INF, 1.5707965)),
        NEG_INF,
        hx("0x1.921FB82C2BD7Fp0"),
    );
}

#[test]
fn tan_rev_decorated() {
    assert_eq!(
        DecoratedInterval::set_dec(iv(-1.0, 1.0), Decoration::Com)
            .tan_rev(DecoratedInterval::new_dec(entire()))
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .tan_rev(DecoratedInterval::new_dec(iv(-1.5708, 1.5708)))
        .is_nai());
}

// --- coshRev ----------------------------------------------------------------

#[test]
fn cosh_rev_vectors() {
    assert!(empty().cosh_rev(entire()).is_empty());
    assert!(iv(1.0, INF).cosh_rev(entire()).is_entire());
    assert!(iv(0.0, INF).cosh_rev(entire()).is_entire());
    encloses(iv(1.0, 1.0).cosh_rev(entire()), 0.0, 0.0);
    encloses(
        iv(hx("0X1.8B07551D9F55P+0"), hx("0X1.89BCA168970C6P+432")).cosh_rev(entire()),
        hx("-0X1.2C903022DD7ABP+8"),
        hx("0X1.2C903022DD7ABP+8"),
    );
}

#[test]
fn cosh_rev_bin_vectors() {
    assert!(empty().cosh_rev(iv(0.0, INF)).is_empty());
    encloses(iv(1.0, INF).cosh_rev(iv(0.0, INF)), 0.0, INF);
    encloses(iv(0.0, INF).cosh_rev(iv(1.0, 2.0)), 1.0, 2.0);
    assert!(iv(1.0, 1.0).cosh_rev(iv(1.0, INF)).is_empty());
    encloses(
        iv(hx("0X1.8B07551D9F55P+0"), hx("0X1.89BCA168970C6P+432")).cosh_rev(iv(NEG_INF, 0.0)),
        hx("-0X1.2C903022DD7ABP+8"),
        hx("-0x1.fffffffffffffp-1"),
    );
}

#[test]
fn cosh_rev_decorated() {
    assert_eq!(
        DecoratedInterval::set_dec(iv(1.0, 1.0), Decoration::Def)
            .cosh_rev(DecoratedInterval::new_dec(entire()))
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .cosh_rev(DecoratedInterval::new_dec(iv(0.0, INF)))
        .is_nai());
}

// --- mulRev (single-interval hull form) -------------------------------------

#[test]
fn mul_rev_vectors() {
    // A representative sweep of the b x c grid; the full grid's mechanisms are
    // pinned exhaustively by the two-output division in reverse_mul_rev_fixture.
    assert!(empty().mul_rev(iv(1.0, 2.0), entire()).is_empty());
    assert!(iv(1.0, 2.0).mul_rev(empty(), entire()).is_empty());
    encloses(
        iv(-2.0, -0.1).mul_rev(iv(-2.1, -0.4), entire()),
        hx("0X1.999999999999AP-3"),
        hx("0X1.5P+4"),
    );
    encloses(
        iv(-2.0, 0.0).mul_rev(iv(-2.1, -0.4), entire()),
        hx("0X1.999999999999AP-3"),
        INF,
    );
    assert!(iv(-2.0, 1.1).mul_rev(iv(-2.1, -0.4), entire()).is_entire());
    assert!(iv(0.0, 0.0).mul_rev(iv(-2.1, -0.4), entire()).is_empty());
    assert!(iv(-2.0, 0.0).mul_rev(iv(-2.1, 0.0), entire()).is_entire());
    encloses(
        iv(0.01, 1.1).mul_rev(iv(-2.1, 0.0), entire()),
        hx("-0X1.A400000000001P+7"),
        0.0,
    );
    encloses(
        iv(0.0, 1.1).mul_rev(iv(0.01, 0.12), entire()),
        hx("0X1.29E4129E4129DP-7"),
        INF,
    );
    assert!(iv(0.0, 0.0).mul_rev(iv(0.01, 0.12), entire()).is_empty());
    encloses(iv(-2.0, -0.1).mul_rev(iv(0.0, INF), entire()), NEG_INF, 0.0);
    assert!(iv(-2.0, -0.1).mul_rev(entire(), entire()).is_entire());
    // A bounded candidate that clips one piece of a split image.
    encloses(
        iv(-2.0, 1.1).mul_rev(iv(0.01, 0.12), iv(0.0, 100.0)),
        hx("0X1.29E4129E4129DP-7"),
        100.0,
    );
}

#[test]
fn mul_rev_ten_vectors() {
    assert!(iv(-2.0, -0.1)
        .mul_rev(iv(-2.1, -0.4), iv(-2.1, -0.4))
        .is_empty());
    encloses(
        iv(-2.0, 1.1).mul_rev(iv(-2.1, -0.4), iv(-2.1, -0.4)),
        -2.1,
        -0.4,
    );
    encloses(
        iv(0.01, 1.1).mul_rev(iv(-2.1, 0.0), iv(-2.1, 0.0)),
        -2.1,
        0.0,
    );
    encloses(
        iv(NEG_INF, -0.1).mul_rev(iv(0.0, 0.12), iv(0.0, 0.12)),
        0.0,
        0.0,
    );
    encloses(
        iv(-2.0, 1.1).mul_rev(iv(0.04, INF), iv(0.04, INF)),
        0.04,
        INF,
    );
}

#[test]
fn mul_rev_decorated() {
    let nai = DecoratedInterval::<f64>::nai();
    assert!(nai
        .mul_rev(
            DecoratedInterval::set_dec(iv(1.0, 2.0), Decoration::Dac),
            DecoratedInterval::new_dec(entire())
        )
        .is_nai());
    let r = DecoratedInterval::set_dec(iv(-2.0, -0.1), Decoration::Dac).mul_rev(
        DecoratedInterval::set_dec(iv(-2.1, -0.4), Decoration::Dac),
        DecoratedInterval::new_dec(entire()),
    );
    assert_eq!(r.decoration(), Decoration::Trv);
    encloses(r.interval(), hx("0X1.999999999999AP-3"), hx("0X1.5P+4"));
    // Ternary form, decorated.
    let r = DecoratedInterval::set_dec(iv(-2.0, 1.1), Decoration::Def).mul_rev(
        DecoratedInterval::set_dec(iv(-2.1, -0.4), Decoration::Com),
        DecoratedInterval::set_dec(iv(-2.1, -0.4), Decoration::Com),
    );
    assert_eq!(r.decoration(), Decoration::Trv);
    encloses(r.interval(), -2.1, -0.4);
}
