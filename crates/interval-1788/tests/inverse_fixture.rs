//! Tests for the round-two elementary functions over the `f64` fixture: the
//! inverse trigonometric `asin`/`acos`/`atan`, the inverse hyperbolic
//! `asinh`/`acosh`/`atanh`, the base variants `exp2`/`exp10`/`log2`/`log10`, and
//! the integer power `pown`, bare and decorated. (`atan2` has its own fixture,
//! `atan2_fixture.rs`.)
//!
//! Because the fixture is deliberately sound-not-tight (each transcendental is
//! widened by a relative margin and stepped outward, and the integer powers ride
//! the outward-rounded `mul`), a fixture result is generally WIDER than the
//! correctly-rounded vector interval, so the transcription asserts the SOUND
//! RELATIONSHIP the fixture guarantees: the result encloses the vector's
//! interval. The structural cases (empty, entire, domain-restricted empties) are
//! exact, and the decorations are checked exactly (the domain and boundedness
//! predicates the local decoration reads are computed exactly on the endpoints).
//!
//! Cases hand-translated from ITF1788 `libieeep1788_elem.itl` (Apache-2.0,
//! original author Marco Nehmeier, ITL port Oliver Heimlich; see
//! `docs/references/vendor/itf1788-framework/`). The hex-float literals of the
//! ITL are parsed by the `hx` helper below rather than pre-converted to decimal,
//! so the transcription stays faithful to the source.

use interval_1788::{DecoratedInterval, Decoration, Interval};

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

/// Scale `m` by `2^e` exactly, by repeated exact doubling or halving (a step
/// only changes the exponent, never the significand, until overflow or the
/// subnormal floor, neither of which the vector values reach short of their true
/// magnitude).
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

/// Parse a C hex-float literal (`[-]0xH.HHHHp[+-]N`) to the exact `f64` it
/// names. The ITL vectors are written in this form because it pins the bit
/// pattern; parsing rather than pre-converting keeps the transcription faithful.
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

fn di(lo: f64, hi: f64, d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(iv(lo, hi), d)
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

// --- asin ------------------------------------------------------------------

#[test]
fn asin_vectors() {
    assert!(Interval::<f64>::empty().asin().is_empty());
    // Domain restriction: [0, inf] restricts to [0, 1].
    encloses(iv(0.0, INF).asin(), 0.0, hx("0x1.921FB54442D19P+0"));
    encloses(iv(NEG_INF, 0.0).asin(), hx("-0x1.921FB54442D19P+0"), 0.0);
    encloses(
        Interval::<f64>::entire().asin(),
        hx("-0x1.921FB54442D19P+0"),
        hx("0x1.921FB54442D19P+0"),
    );
    encloses(
        iv(-1.0, 1.0).asin(),
        hx("-0x1.921FB54442D19P+0"),
        hx("0x1.921FB54442D19P+0"),
    );
    // Saturation at +-1: asin(+-1) = +-pi/2, the endpoint is the tight far bound.
    encloses(
        iv(1.0, INF).asin(),
        hx("0x1.921FB54442D18P+0"),
        hx("0x1.921FB54442D19P+0"),
    );
    encloses(iv(0.0, 0.0).asin(), 0.0, 0.0);
    // Wholly outside [-1, 1] -> empty.
    assert!(iv(NEG_INF, hx("-0x1.0000000000001P+0")).asin().is_empty());
    assert!(iv(hx("0x1.0000000000001P+0"), INF).asin().is_empty());
    // An interior case.
    encloses(
        iv(hx("-0x1.51EB851EB851FP-2"), hx("0x1.FFFFFFFFFFFFFP-1")).asin(),
        hx("-0x1.585FF6E341C3FP-2"),
        hx("0x1.921FB50442D19P+0"),
    );
}

#[test]
fn asin_decorated_vectors() {
    // [0, inf] leaves the domain -> trv.
    let r = di(0.0, INF, Decoration::Dac).asin();
    assert_eq!(r.decoration(), Decoration::Trv);
    // [-1, 1] stays com.
    let r = di(-1.0, 1.0, Decoration::Com).asin();
    assert_eq!(r.decoration(), Decoration::Com);
    encloses(
        r.interval(),
        hx("-0x1.921FB54442D19P+0"),
        hx("0x1.921FB54442D19P+0"),
    );
    assert!(DecoratedInterval::<f64>::nai().asin().is_nai());
}

// --- acos (antitone) -------------------------------------------------------

#[test]
fn acos_vectors() {
    assert!(Interval::<f64>::empty().acos().is_empty());
    encloses(iv(0.0, INF).acos(), 0.0, hx("0x1.921FB54442D19P+0"));
    encloses(
        iv(NEG_INF, 0.0).acos(),
        hx("0x1.921FB54442D18P+0"),
        hx("0x1.921FB54442D19P+1"),
    );
    encloses(
        Interval::<f64>::entire().acos(),
        0.0,
        hx("0x1.921FB54442D19P+1"),
    );
    // acos(1) = 0 (the antitone floor), acos(-1) = pi (the ceiling).
    encloses(iv(1.0, INF).acos(), 0.0, 0.0);
    encloses(
        iv(-1.0, -1.0).acos(),
        hx("0x1.921FB54442D18P+1"),
        hx("0x1.921FB54442D19P+1"),
    );
    encloses(
        iv(0.0, 0.0).acos(),
        hx("0x1.921FB54442D18P+0"),
        hx("0x1.921FB54442D19P+0"),
    );
    assert!(iv(hx("0x1.0000000000001P+0"), INF).acos().is_empty());
    encloses(
        iv(hx("-0x1.51EB851EB851FP-2"), hx("0x1.FFFFFFFFFFFFFP-1")).acos(),
        hx("0x1P-26"),
        hx("0x1.E837B2FD13428P+0"),
    );
}

#[test]
fn acos_decorated_vectors() {
    assert_eq!(
        di(0.0, INF, Decoration::Dac).acos().decoration(),
        Decoration::Trv
    );
    assert_eq!(
        di(-1.0, 1.0, Decoration::Com).acos().decoration(),
        Decoration::Com
    );
    assert!(DecoratedInterval::<f64>::nai().acos().is_nai());
}

// --- atan ------------------------------------------------------------------

#[test]
fn atan_vectors() {
    assert!(Interval::<f64>::empty().atan().is_empty());
    encloses(iv(0.0, INF).atan(), 0.0, hx("0x1.921FB54442D19P+0"));
    encloses(iv(NEG_INF, 0.0).atan(), hx("-0x1.921FB54442D19P+0"), 0.0);
    encloses(
        Interval::<f64>::entire().atan(),
        hx("-0x1.921FB54442D19P+0"),
        hx("0x1.921FB54442D19P+0"),
    );
    encloses(iv(0.0, 0.0).atan(), 0.0, 0.0);
    encloses(
        iv(1.0, hx("0x1.4C2463567C5ACP+25")).atan(),
        hx("0x1.921FB54442D18P-1"),
        hx("0x1.921FB4E19ABD7P+0"),
    );
}

#[test]
fn atan_decorated_vectors() {
    // Defined everywhere: an unbounded input demotes to dac, never trv.
    assert_eq!(
        di(0.0, INF, Decoration::Dac).atan().decoration(),
        Decoration::Dac
    );
    assert_eq!(
        di(NEG_INF, INF, Decoration::Dac).atan().decoration(),
        Decoration::Dac
    );
    assert!(DecoratedInterval::<f64>::nai().atan().is_nai());
}

// --- asinh -----------------------------------------------------------------

#[test]
fn asinh_vectors() {
    assert!(Interval::<f64>::empty().asinh().is_empty());
    encloses(iv(0.0, INF).asinh(), 0.0, INF);
    encloses(iv(NEG_INF, 0.0).asinh(), NEG_INF, 0.0);
    assert!(Interval::<f64>::entire().asinh().is_entire());
    encloses(iv(0.0, 0.0).asinh(), 0.0, 0.0);
    encloses(
        iv(1.0, hx("0x1.2C903022DD7AAP+8")).asinh(),
        hx("0x1.C34366179D426P-1"),
        hx("0x1.9986127438A87P+2"),
    );
}

#[test]
fn asinh_decorated_vectors() {
    assert_eq!(
        di(0.0, INF, Decoration::Dac).asinh().decoration(),
        Decoration::Dac
    );
    let r = di(1.0, hx("0x1.2C903022DD7AAP+8"), Decoration::Com).asinh();
    assert_eq!(r.decoration(), Decoration::Com);
    assert!(DecoratedInterval::<f64>::nai().asinh().is_nai());
}

// --- acosh -----------------------------------------------------------------

#[test]
fn acosh_vectors() {
    assert!(Interval::<f64>::empty().acosh().is_empty());
    // Domain [1, inf): restriction and the acosh(1) = 0 floor.
    encloses(iv(0.0, INF).acosh(), 0.0, INF);
    encloses(iv(1.0, INF).acosh(), 0.0, INF);
    encloses(iv(NEG_INF, 1.0).acosh(), 0.0, 0.0);
    assert!(iv(NEG_INF, hx("0x1.FFFFFFFFFFFFFP-1")).acosh().is_empty());
    encloses(Interval::<f64>::entire().acosh(), 0.0, INF);
    encloses(iv(1.0, 1.0).acosh(), 0.0, 0.0);
    encloses(
        iv(hx("0x1.199999999999AP+0"), hx("0x1.2666666666666P+1")).acosh(),
        hx("0x1.C636C1A882F2CP-2"),
        hx("0x1.799C88E79140DP+0"),
    );
}

#[test]
fn acosh_decorated_vectors() {
    // [0, inf] reaches below 1 -> trv.
    assert_eq!(
        di(0.0, INF, Decoration::Dac).acosh().decoration(),
        Decoration::Trv
    );
    // [1, inf] is in domain but unbounded -> dac.
    assert_eq!(
        di(1.0, INF, Decoration::Dac).acosh().decoration(),
        Decoration::Dac
    );
    assert_eq!(
        di(1.0, 1.0, Decoration::Com).acosh().decoration(),
        Decoration::Com
    );
    // Reaching just below 1 -> trv.
    assert_eq!(
        di(0.9, 1.0, Decoration::Com).acosh().decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai().acosh().is_nai());
}

// --- atanh -----------------------------------------------------------------

#[test]
fn atanh_vectors() {
    assert!(Interval::<f64>::empty().atanh().is_empty());
    encloses(iv(0.0, INF).atanh(), 0.0, INF);
    assert!(iv(1.0, INF).atanh().is_empty());
    encloses(iv(NEG_INF, 0.0).atanh(), NEG_INF, 0.0);
    assert!(iv(NEG_INF, -1.0).atanh().is_empty());
    assert!(iv(-1.0, 1.0).atanh().is_entire());
    encloses(iv(0.0, 0.0).atanh(), 0.0, 0.0);
    assert!(iv(-1.0, -1.0).atanh().is_empty());
    assert!(iv(1.0, 1.0).atanh().is_empty());
    encloses(
        iv(hx("0x1.4C0420F6F08CCP-2"), hx("0x1.FFFFFFFFFFFFFP-1")).atanh(),
        hx("0x1.5871DD2DF9102P-2"),
        hx("0x1.2B708872320E2P+4"),
    );
}

#[test]
fn atanh_decorated_vectors() {
    // Reaching +-1 (the open-domain boundary) -> trv.
    assert_eq!(
        di(0.0, INF, Decoration::Dac).atanh().decoration(),
        Decoration::Trv
    );
    assert_eq!(
        di(-1.0, 1.0, Decoration::Com).atanh().decoration(),
        Decoration::Trv
    );
    assert_eq!(
        di(0.0, 0.0, Decoration::Com).atanh().decoration(),
        Decoration::Com
    );
    // [-1, x] touches -1 -> trv, image unbounded below.
    let r = di(-1.0, hx("0x1.FFFFFFFFFFFFFP-1"), Decoration::Com).atanh();
    assert_eq!(r.decoration(), Decoration::Trv);
    assert_eq!(r.interval().inf(), NEG_INF);
    assert!(DecoratedInterval::<f64>::nai().atanh().is_nai());
}

// --- exp2 / exp10 ----------------------------------------------------------

#[test]
fn exp2_vectors() {
    assert!(Interval::<f64>::empty().exp2().is_empty());
    encloses(iv(NEG_INF, 0.0).exp2(), 0.0, 1.0);
    encloses(iv(0.0, INF).exp2(), 1.0, INF);
    encloses(Interval::<f64>::entire().exp2(), 0.0, INF);
    // Overflow shoulder: 2^1024 overflows to the [MAX, inf] edge.
    encloses(
        iv(1024.0, 1024.0).exp2(),
        hx("0x1.FFFFFFFFFFFFFP+1023"),
        INF,
    );
    encloses(iv(1.0, 5.0).exp2(), 2.0, 32.0);
    encloses(iv(-1022.0, 1.0).exp2(), hx("0x1P-1022"), 2.0);
}

#[test]
fn exp10_vectors() {
    assert!(Interval::<f64>::empty().exp10().is_empty());
    encloses(iv(NEG_INF, 0.0).exp10(), 0.0, 1.0);
    encloses(iv(0.0, INF).exp10(), 1.0, INF);
    encloses(Interval::<f64>::entire().exp10(), 0.0, INF);
    encloses(
        iv(hx("0x1.34413509F79FFP+8"), hx("0x1.34413509F79FFP+8")).exp10(),
        hx("0x1.FFFFFFFFFFFFFP+1023"),
        INF,
    );
    encloses(iv(1.0, 5.0).exp10(), 10.0, 100_000.0);
}

#[test]
fn exp_bases_decorated_vectors() {
    // exp2 overflow: bounded input, unbounded result -> dac.
    assert_eq!(
        di(1024.0, 1024.0, Decoration::Com).exp2().decoration(),
        Decoration::Dac
    );
    assert_eq!(
        di(1.0, 5.0, Decoration::Def).exp2().decoration(),
        Decoration::Def
    );
    assert_eq!(
        di(1.0, 5.0, Decoration::Com).exp10().decoration(),
        Decoration::Com
    );
    assert!(DecoratedInterval::<f64>::nai().exp2().is_nai());
    assert!(DecoratedInterval::<f64>::nai().exp10().is_nai());
}

// --- log2 / log10 ----------------------------------------------------------

#[test]
fn log2_vectors() {
    assert!(Interval::<f64>::empty().log2().is_empty());
    assert!(iv(NEG_INF, 0.0).log2().is_empty());
    encloses(iv(0.0, 1.0).log2(), NEG_INF, 0.0);
    encloses(iv(1.0, INF).log2(), 0.0, INF);
    assert!(iv(0.0, INF).log2().is_entire());
    assert!(Interval::<f64>::entire().log2().is_entire());
    encloses(iv(2.0, 32.0).log2(), 1.0, 5.0);
    encloses(iv(hx("0x0.0000000000001p-1022"), 2.0).log2(), -1074.0, 1.0);
}

#[test]
fn log10_vectors() {
    assert!(Interval::<f64>::empty().log10().is_empty());
    assert!(iv(NEG_INF, 0.0).log10().is_empty());
    encloses(iv(0.0, 1.0).log10(), NEG_INF, 0.0);
    encloses(iv(1.0, INF).log10(), 0.0, INF);
    assert!(Interval::<f64>::entire().log10().is_entire());
    encloses(iv(10.0, 100_000.0).log10(), 1.0, 5.0);
}

#[test]
fn log_bases_decorated_vectors() {
    // Reaching zero -> trv (and unbounded below).
    assert_eq!(
        di(0.0, hx("0x1.FFFFFFFFFFFFFp1023"), Decoration::Com)
            .log2()
            .decoration(),
        Decoration::Trv
    );
    // Positive bounded -> com; positive unbounded -> dac.
    assert_eq!(
        di(2.0, 32.0, Decoration::Com).log2().decoration(),
        Decoration::Com
    );
    assert_eq!(
        di(hx("0x0.0000000000001p-1022"), INF, Decoration::Dac)
            .log2()
            .decoration(),
        Decoration::Dac
    );
    assert!(DecoratedInterval::<f64>::nai().log10().is_nai());
}

// --- pown ------------------------------------------------------------------

#[test]
fn pown_exponent_zero() {
    assert!(Interval::<f64>::empty().pown(0).is_empty());
    encloses(Interval::<f64>::entire().pown(0), 1.0, 1.0);
    encloses(iv(0.0, 0.0).pown(0), 1.0, 1.0);
    encloses(iv(13.1, 13.1).pown(0), 1.0, 1.0);
    encloses(iv(-324.3, 2.5).pown(0), 1.0, 1.0);
}

#[test]
fn pown_positive_even() {
    encloses(Interval::<f64>::entire().pown(2), 0.0, INF);
    encloses(iv(0.0, 0.0).pown(2), 0.0, 0.0);
    encloses(
        iv(13.1, 13.1).pown(2),
        hx("0X1.573851EB851EBP+7"),
        hx("0X1.573851EB851ECP+7"),
    );
    // Straddling zero: minimum 0, maximum at the larger magnitude endpoint.
    encloses(iv(-324.3, 2.5).pown(2), 0.0, hx("0X1.9AD27D70A3D72P+16"));
    encloses(
        iv(-1.9, -0.33).pown(2),
        hx("0X1.BE0DED288CE7P-4"),
        hx("0X1.CE147AE147AE1P+1"),
    );
    encloses(iv(0.0, INF).pown(8), 0.0, INF);
    encloses(iv(NEG_INF, 0.0).pown(8), 0.0, INF);
}

#[test]
fn pown_positive_odd() {
    assert!(Interval::<f64>::entire().pown(3).is_entire());
    encloses(iv(0.0, 0.0).pown(3), 0.0, 0.0);
    encloses(
        iv(13.1, 13.1).pown(3),
        hx("0X1.1902E978D4FDEP+11"),
        hx("0X1.1902E978D4FDFP+11"),
    );
    encloses(
        iv(-7451.145, -7451.145).pown(3),
        hx("-0X1.81460637B9A3DP+38"),
        hx("-0X1.81460637B9A3CP+38"),
    );
    encloses(iv(NEG_INF, 0.0).pown(3), NEG_INF, 0.0);
    encloses(
        iv(-1.9, -0.33).pown(3),
        hx("-0X1.B6F9DB22D0E55P+2"),
        hx("-0X1.266559F6EC5B1P-5"),
    );
}

#[test]
fn pown_negative_exponent() {
    // Zero base with a negative exponent is undefined -> empty.
    assert!(iv(0.0, 0.0).pown(-2).is_empty());
    assert!(iv(0.0, 0.0).pown(-1).is_empty());
    encloses(Interval::<f64>::entire().pown(-2), 0.0, INF);
    // Base straddling zero, even negative: [0, sup^2] reciprocated -> [.., inf].
    encloses(iv(-324.3, 2.5).pown(-2), hx("0X1.3F0C482C977C9P-17"), INF);
    // Base straddling zero, odd negative: reciprocal straddles zero -> entire.
    assert!(iv(-324.3, 2.5).pown(-1).is_entire());
    encloses(iv(NEG_INF, 0.0).pown(-1), NEG_INF, 0.0);
    encloses(
        iv(0.01, 2.33).pown(-1),
        hx("0X1.B77C278DBBE13P-2"),
        hx("0X1.9P+6"),
    );
    encloses(iv(0.0, INF).pown(-2), 0.0, INF);
}

#[test]
fn pown_decorated_vectors() {
    // Nonnegative exponent: defined everywhere.
    assert_eq!(
        di(-5.0, 10.0, Decoration::Com).pown(0).decoration(),
        Decoration::Com
    );
    assert_eq!(
        di(-3.0, 5.0, Decoration::Def).pown(2).decoration(),
        Decoration::Def
    );
    // Even overflow -> unbounded result -> dac.
    assert_eq!(
        di(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0, Decoration::Com)
            .pown(2)
            .decoration(),
        Decoration::Dac
    );
    // Negative exponent, base excludes zero -> com.
    assert_eq!(
        di(3.0, 5.0, Decoration::Com).pown(-2).decoration(),
        Decoration::Com
    );
    // Negative exponent, base straddles zero -> trv.
    assert_eq!(
        di(-5.0, 3.0, Decoration::Com).pown(-2).decoration(),
        Decoration::Trv
    );
    assert_eq!(
        di(-3.0, 5.0, Decoration::Com).pown(-3).decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai().pown(2).is_nai());
}
