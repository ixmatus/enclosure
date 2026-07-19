//! Tests for the plain exponential and natural logarithm over the `f64` fixture:
//! `exp` and `log` (the crate's `ln`), bare and decorated.
//!
//! # The sound-not-tight split
//!
//! The `f64` fixture widens each transcendental by a relative margin and steps
//! the result outward, so a fixture interval is generally WIDER than the
//! correctly-rounded corpus interval. The transcription therefore asserts the
//! SOUND RELATIONSHIP the fixture guarantees, split by arm:
//!
//! - **Value arms** (a finite or half-unbounded image with a computed endpoint):
//!   the fixture result ENCLOSES the corpus interval (`encloses`). This is the
//!   only claim sound on a loose backend; last-ulp tightness is a property of a
//!   correctly-rounded backend, certified by the tight conformance lane, not here.
//! - **Structural cases** (`exp([empty]) = [empty]`; `log` of a wholly
//!   non-positive interval `= [empty]`; the `[entire]` images of `log` at and
//!   below zero): pinned exactly, since the emptiness and entirety are set facts,
//!   not rounded values.
//! - **Decorations**: pinned exactly. The domain and boundedness predicates the
//!   local decoration reads are computed exactly on the endpoints, so `com` /
//!   `dac` / `def` / `trv` do not depend on the transcendental's looseness.
//!
//! The hex-float literals of the ITL are parsed by the `hx` helper (with the
//! canonicality assertion from `reverse_rev_fixture.rs`) rather than pre-converted
//! to decimal, so the transcription stays faithful to the source.
//!
//! Cases hand-translated from ITF1788 `libieeep1788_elem.itl` (`minimal_exp_test`,
//! `minimal_exp_dec_test`, `minimal_log_test`, `minimal_log_dec_test`; Apache-2.0,
//! original author Marco Nehmeier, ITL port Oliver Heimlich; see
//! `docs/references/vendor/itf1788-framework/`).

use interval_1788::{DecoratedInterval, Decoration, Interval};

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

/// Parse a C hex-float literal (`[-]0xH.HHHHp[+-]N`) to the exact `f64` it names.
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
    // A canonical corpus literal carries at most 53 significant bits, so the cast
    // is exact; the assertion turns a non-canonical literal into a loud failure
    // instead of a silent rounding (mirrors reverse_rev_fixture.rs).
    assert!(m < (1 << 53), "hex-float mantissa exceeds f64 exact range");
    #[allow(clippy::cast_precision_loss)]
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

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

fn di(lo: f64, hi: f64, d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(iv(lo, hi), d)
}

/// The sound relationship for a value arm: `result` encloses the corpus interval
/// `[lo, hi]`.
fn encloses(result: Interval<f64>, lo: f64, hi: f64) {
    assert!(
        result.inf() <= lo && hi <= result.sup(),
        "expected enclosure of [{lo}, {hi}], got [{}, {}]",
        result.inf(),
        result.sup()
    );
}

// --- exp -------------------------------------------------------------------

#[test]
fn exp_vectors() {
    // minimal_exp_test: 19 vectors. Empty is exact; every image arm encloses.
    assert!(Interval::<f64>::empty().exp().is_empty());

    encloses(iv(NEG_INF, 0.0).exp(), 0.0, 1.0);
    encloses(iv(NEG_INF, -0.0).exp(), 0.0, 1.0);
    encloses(iv(0.0, INF).exp(), 1.0, INF);
    encloses(iv(-0.0, INF).exp(), 1.0, INF);
    encloses(Interval::<f64>::entire().exp(), 0.0, INF);

    encloses(iv(NEG_INF, hx("0X1.62E42FEFA39FP+9")).exp(), 0.0, INF);
    encloses(
        iv(hx("0X1.62E42FEFA39FP+9"), hx("0X1.62E42FEFA39FP+9")).exp(),
        hx("0X1.FFFFFFFFFFFFFP+1023"),
        INF,
    );
    encloses(
        iv(0.0, hx("0X1.62E42FEFA39EP+9")).exp(),
        1.0,
        hx("0X1.FFFFFFFFFC32BP+1023"),
    );
    encloses(
        iv(-0.0, hx("0X1.62E42FEFA39EP+9")).exp(),
        1.0,
        hx("0X1.FFFFFFFFFC32BP+1023"),
    );
    encloses(
        iv(hx("-0X1.6232BDD7ABCD3P+9"), hx("0X1.62E42FEFA39EP+9")).exp(),
        hx("0X0.FFFFFFFFFFE7BP-1022"),
        hx("0X1.FFFFFFFFFC32BP+1023"),
    );
    encloses(
        iv(hx("-0X1.6232BDD7ABCD3P+8"), hx("0X1.62E42FEFA39EP+9")).exp(),
        hx("0X1.FFFFFFFFFFE7BP-512"),
        hx("0X1.FFFFFFFFFC32BP+1023"),
    );
    encloses(
        iv(hx("-0X1.6232BDD7ABCD3P+8"), 0.0).exp(),
        hx("0X1.FFFFFFFFFFE7BP-512"),
        1.0,
    );
    encloses(
        iv(hx("-0X1.6232BDD7ABCD3P+8"), -0.0).exp(),
        hx("0X1.FFFFFFFFFFE7BP-512"),
        1.0,
    );
    encloses(
        iv(hx("-0X1.6232BDD7ABCD3P+8"), 1.0).exp(),
        hx("0X1.FFFFFFFFFFE7BP-512"),
        hx("0X1.5BF0A8B14576AP+1"),
    );

    encloses(
        iv(1.0, 5.0).exp(),
        hx("0X1.5BF0A8B145769P+1"),
        hx("0X1.28D389970339P+7"),
    );

    encloses(
        iv(hx("-0X1.A934F0979A372P+1"), hx("0X1.CEAECFEA8085AP+0")).exp(),
        hx("0X1.2797F0A337A5FP-5"),
        hx("0X1.86091CC9095C5P+2"),
    );
    encloses(
        iv(hx("0X1.87F42B972949CP-1"), hx("0X1.8B55484710029P+6")).exp(),
        hx("0X1.1337E9E45812AP+1"),
        hx("0X1.805A5C88021B6P+142"),
    );
    encloses(
        iv(hx("0X1.78025C8B3FD39P+3"), hx("0X1.9FD8EEF3FA79BP+4")).exp(),
        hx("0X1.EF461A783114CP+16"),
        hx("0X1.691D36C6B008CP+37"),
    );
}

#[test]
fn exp_decorated_vectors() {
    // minimal_exp_dec_test: 2 vectors. Decorations exact; images enclose.
    // Overflow of a bounded input demotes com to dac.
    let overflow = di(
        hx("0X1.62E42FEFA39FP+9"),
        hx("0X1.62E42FEFA39FP+9"),
        Decoration::Com,
    )
    .exp();
    assert_eq!(overflow.decoration(), Decoration::Dac);
    encloses(overflow.interval(), hx("0X1.FFFFFFFFFFFFFP+1023"), INF);

    let bounded = di(0.0, hx("0X1.62E42FEFA39EP+9"), Decoration::Def).exp();
    assert_eq!(bounded.decoration(), Decoration::Def);
    encloses(bounded.interval(), 1.0, hx("0X1.FFFFFFFFFC32BP+1023"));

    assert!(DecoratedInterval::<f64>::nai().exp().is_nai());
}

// --- log (the crate's natural logarithm `ln`) ------------------------------

#[test]
fn log_vectors() {
    // minimal_log_test: 21 vectors. Empty and the wholly non-positive domains are
    // exact empties; the [entire] images at and below zero are exact; the rest
    // enclose.
    assert!(Interval::<f64>::empty().ln().is_empty());
    assert!(iv(NEG_INF, 0.0).ln().is_empty());
    assert!(iv(NEG_INF, -0.0).ln().is_empty());

    encloses(iv(0.0, 1.0).ln(), NEG_INF, 0.0);
    encloses(iv(-0.0, 1.0).ln(), NEG_INF, 0.0);
    encloses(iv(1.0, INF).ln(), 0.0, INF);

    assert!(iv(0.0, INF).ln().is_entire());
    assert!(iv(-0.0, INF).ln().is_entire());
    assert!(Interval::<f64>::entire().ln().is_entire());

    encloses(
        iv(0.0, hx("0x1.FFFFFFFFFFFFFp1023")).ln(),
        NEG_INF,
        hx("0X1.62E42FEFA39FP+9"),
    );
    encloses(
        iv(-0.0, hx("0x1.FFFFFFFFFFFFFp1023")).ln(),
        NEG_INF,
        hx("0X1.62E42FEFA39FP+9"),
    );
    encloses(
        iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")).ln(),
        0.0,
        hx("0X1.62E42FEFA39FP+9"),
    );
    encloses(
        iv(hx("0x0.0000000000001p-1022"), hx("0x1.FFFFFFFFFFFFFp1023")).ln(),
        hx("-0x1.74385446D71C4p9"),
        hx("0x1.62E42FEFA39Fp9"),
    );
    encloses(
        iv(hx("0x0.0000000000001p-1022"), 1.0).ln(),
        hx("-0x1.74385446D71C4p9"),
        0.0,
    );

    encloses(
        iv(hx("0X1.5BF0A8B145769P+1"), hx("0X1.5BF0A8B145769P+1")).ln(),
        hx("0X1.FFFFFFFFFFFFFP-1"),
        hx("0X1P+0"),
    );
    encloses(
        iv(hx("0X1.5BF0A8B14576AP+1"), hx("0X1.5BF0A8B14576AP+1")).ln(),
        hx("0X1P+0"),
        hx("0X1.0000000000001P+0"),
    );
    encloses(
        iv(hx("0x0.0000000000001p-1022"), hx("0X1.5BF0A8B14576AP+1")).ln(),
        hx("-0x1.74385446D71C4p9"),
        hx("0X1.0000000000001P+0"),
    );
    encloses(
        iv(hx("0X1.5BF0A8B145769P+1"), 32.0).ln(),
        hx("0X1.FFFFFFFFFFFFFP-1"),
        hx("0X1.BB9D3BEB8C86CP+1"),
    );
    encloses(
        iv(hx("0X1.999999999999AP-4"), hx("0X1.CP+1")).ln(),
        hx("-0X1.26BB1BBB55516P+1"),
        hx("0X1.40B512EB53D6P+0"),
    );
    encloses(
        iv(hx("0X1.B333333333333P+0"), hx("0X1.C81FD88228B2FP+98")).ln(),
        hx("0X1.0FAE81914A99P-1"),
        hx("0X1.120627F6AE7F1P+6"),
    );
    encloses(
        iv(hx("0X1.AEA0000721861P+11"), hx("0X1.FCA055555554CP+25")).ln(),
        hx("0X1.04A1363DB1E63P+3"),
        hx("0X1.203E52C0256B5P+4"),
    );
}

#[test]
fn log_decorated_vectors() {
    // minimal_log_dec_test: 3 vectors. Decorations exact; images enclose.
    // Positive and bounded stays com.
    let com = di(
        hx("0x0.0000000000001p-1022"),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        Decoration::Com,
    )
    .ln();
    assert_eq!(com.decoration(), Decoration::Com);
    encloses(
        com.interval(),
        hx("-0x1.74385446D71C4p9"),
        hx("0X1.62E42FEFA39FP+9"),
    );

    // Reaching zero leaves the domain: com demotes to trv, image unbounded below.
    let touches_zero = di(0.0, 1.0, Decoration::Com).ln();
    assert_eq!(touches_zero.decoration(), Decoration::Trv);
    encloses(touches_zero.interval(), NEG_INF, 0.0);

    // An explicit def input dominates the local com.
    let defd = di(
        hx("0X1.5BF0A8B14576AP+1"),
        hx("0X1.5BF0A8B14576AP+1"),
        Decoration::Def,
    )
    .ln();
    assert_eq!(defd.decoration(), Decoration::Def);
    encloses(defd.interval(), hx("0X1P+0"), hx("0X1.0000000000001P+0"));

    assert!(DecoratedInterval::<f64>::nai().ln().is_nai());
}
