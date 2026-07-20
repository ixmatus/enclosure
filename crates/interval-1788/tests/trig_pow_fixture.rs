//! Tests for the trigonometric, hyperbolic, and power functions over the `f64`
//! fixture: `sin`, `cos`, `tan`, `sinh`, `cosh`, `tanh`, `pow`, `rootn`, bare and
//! decorated.
//!
//! The unit tests transcribe a representative selection of the vendored ITF1788
//! `libieeep1788_elem.itl` vectors; the `rootn` cases are derived from integer
//! arithmetic rather than transcribed, because the corpus's only `rootn`
//! vectors live in the LGPL-licensed `c-xsc.itl`, which this permissively
//! licensed tree does not adapt (registry entry: itf1788-framework). Because
//! the fixture is
//! deliberately sound-not-tight (each transcendental is widened by a relative
//! margin and stepped outward), a fixture result is generally WIDER than the
//! correctly-rounded vector interval, so the transcription asserts the SOUND
//! RELATIONSHIP the fixture guarantees: the result encloses the vector's interval.
//! The structural cases (empty, entire, the tan pole transitions) are exact. For
//! decorated vectors the interval encloses and the decoration matches, except at
//! the exact quadrant-boundary tan inputs, where the ambiguity-widening pole rule
//! returns `entire`/`trv` in place of the tight finite/`com` result: `trv` is the
//! weaker (sound) decoration there, checked as `result_dec <= vector_dec`.
//!
//! The property lanes add pointwise enclosure over magnitude ranges (including
//! large arguments past 1e10, where the reduction is stressed), the full-period
//! shortcut, critical-point stress near the `k*pi/2` boundaries, the `cosh`
//! minimum, the `pow`/`rootn` sign-case grid, and the `tan` pole straddle.
//!
//! Cases derived from ITF1788 `libieeep1788_elem.itl` (Apache-2.0, original
//! author Marco Nehmeier, ITL port Oliver Heimlich; see
//! `docs/references/vendor/itf1788-framework/`). An earlier revision
//! attributed `c-xsc.itl` here as Apache-2.0; its header is in fact
//! LGPL-2.1-or-later, and its three vectors were replaced by independent
//! derivations on 2026-07-20.

use core::f64::consts::{FRAC_PI_2, PI};
use interval_1788::{DecoratedInterval, Decoration, Interval};
use proptest::prelude::*;

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

// Vector values, converted from the ITL hex-float literals (Rust has no hex-float
// literal); the hex source is named in the comment where each is used.
const SIN_1_2_LO: f64 = 0.841_470_984_807_896_5;
const SIN_2_3_LO: f64 = 0.141_120_008_059_867_2;
const SIN_2_3_HI: f64 = 0.909_297_426_825_681_7;
const SIN_N07_01_LO: f64 = -0.644_217_687_237_691_1;
const SIN_N07_01_HI: f64 = 0.099_833_416_646_828_17;
const ONE_MINUS_ULP: f64 = 0.999_999_999_999_999_9;
const SIN_PI_LO: f64 = 1.224_646_799_147_353_e-16;
const SIN_PI_HI: f64 = 1.224_646_799_147_353_2e-16;
const COS_1_2_LO: f64 = -0.416_146_836_547_142_4;
const COS_1_2_HI: f64 = 0.540_302_305_868_139_8;
const COS_2_3_LO: f64 = -0.989_992_496_600_445_5;
const COS_N07_01_LO: f64 = 0.764_842_187_284_488_4;
const TAN_IN_LO: f64 = -0.333_33;
const TAN_IN_HI: f64 = 0.1;
const TAN_OUT_LO: f64 = -0.346_249_816_543_148_85;
const TAN_OUT_HI: f64 = 0.100_334_672_085_450_56;
const TAN_BIG_A: f64 = 5345.555;
const TAN_BIG_B: f64 = 5346.01;
const TAN_BIG_LO: f64 = -7.356_840_852_049_277;
const TAN_BIG_HI: f64 = -1.493_205_097_982_578_8;
const TAN_BIG_B2: f64 = 5446.01;
const POW_LO: f64 = 0.794_328_234_724_281_5;
const POW_HI: f64 = 0.933_032_991_536_807_5;
const POW_WIDE_LO: f64 = 0.003_162_277_660_168_379_4;
const POW_WIDE_HI: f64 = 316.227_766_016_837_9;
const SINH_B: f64 = 300.563_234_5;
const SINH_LO: f64 = 1.175_201_193_643_801_4;
const SINH_HI: f64 = 1.705_784_684_221_514_e130;
const COSH_A: f64 = -1.1;
const COSH_B: f64 = 2.3;
const COSH_HI: f64 = 5.037_220_649_268_762;
const TANH_LO: f64 = 0.761_594_155_955_764_9;

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

fn di(lo: f64, hi: f64, d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(iv(lo, hi), d)
}

/// The sound relationship: `result` encloses the vector interval `[lo, hi]`. The
/// fixture is looser than the tight vector, so this is the property a
/// sound-not-tight backend guarantees; a correctly-rounded backend would meet the
/// vector bitwise (the tight-backend lane's job).
fn encloses(result: Interval<f64>, lo: f64, hi: f64) {
    assert!(
        result.inf() <= lo && hi <= result.sup(),
        "expected enclosure of [{lo}, {hi}], got [{}, {}]",
        result.inf(),
        result.sup()
    );
}

/// The decorated sound relationship: the interval encloses and the decoration is
/// no stronger than the vector's (equal for every case here except the tan
/// quadrant-boundary inputs, where the weaker `trv` is the sound answer).
fn dec_encloses(result: DecoratedInterval<f64>, lo: f64, hi: f64, d: Decoration) {
    encloses(result.interval(), lo, hi);
    assert!(
        result.decoration() <= d,
        "decoration {} stronger than the sound bound {d}",
        result.decoration()
    );
}

// --- Bare sin --------------------------------------------------------------

#[test]
fn sin_structural_cases() {
    assert!(iv(NEG_INF, 5.0).sin().inf() == -1.0 && iv(NEG_INF, 5.0).sin().sup() == 1.0);
    // sin [0,infinity] = [-1,1]; sin [entire] = [-1,1].
    encloses(iv(0.0, INF).sin(), -1.0, 1.0);
    encloses(Interval::<f64>::entire().sin(), -1.0, 1.0);
    assert!(Interval::<f64>::empty().sin().is_empty());
    // sin [0,0] = [0,0].
    encloses(iv(0.0, 0.0).sin(), 0.0, 0.0);
}

#[test]
fn sin_quadrant_and_reduction_vectors() {
    // sin [1,2] = [0x1.AED548F090CEEP-1, 1] (the max +1 is admitted at interior pi/2).
    encloses(iv(1.0, 2.0).sin(), SIN_1_2_LO, 1.0);
    // sin [2,3] = [0x1.210386DB6D55BP-3, 0x1.D18F6EAD1B446P-1].
    encloses(iv(2.0, 3.0).sin(), SIN_2_3_LO, SIN_2_3_HI);
    // sin [-0.7,0.1] = [-0x1.49D6E694619B9P-1, 0x1.98EAECB8BCB2DP-4].
    encloses(iv(-0.7, 0.1).sin(), SIN_N07_01_LO, SIN_N07_01_HI);
    // sin at pi/2 (0x1.921FB54442D18P+0), a max: ambiguity widening admits +1.
    // Vector [0x1.FFFFFFFFFFFFFP-1, 1].
    encloses(iv(FRAC_PI_2, FRAC_PI_2).sin(), ONE_MINUS_ULP, 1.0);
    // sin at pi (0x1.921FB54442D18P+1): the reduction stress case, sin(pi_f64) is
    // a tiny positive near 1.2e-16. Vector [0x1.1A62633145C06P-53, ...C07P-53].
    encloses(iv(PI, PI).sin(), SIN_PI_LO, SIN_PI_HI);
}

// --- Bare cos --------------------------------------------------------------

#[test]
fn cos_vectors() {
    encloses(iv(0.0, INF).cos(), -1.0, 1.0);
    encloses(Interval::<f64>::entire().cos(), -1.0, 1.0);
    // cos [0,0] = [1,1].
    encloses(iv(0.0, 0.0).cos(), 1.0, 1.0);
    // cos [1,2] = [-0x1.AA22657537205P-2, 0x1.14A280FB5068CP-1].
    encloses(iv(1.0, 2.0).cos(), COS_1_2_LO, COS_1_2_HI);
    // cos [2,3] = [-0x1.FAE04BE85E5D3P-1, -0x1.AA22657537204P-2]; both endpoints
    // negative, no interior extremum (the min -1 at pi is not reached until >pi).
    encloses(iv(2.0, 3.0).cos(), COS_2_3_LO, -0.416_146_836_547_142_35);
    // cos [-0.7,0.1] = [0x1.87996529F9D92P-1, 1]; the max +1 at 0 is admitted.
    encloses(iv(-0.7, 0.1).cos(), COS_N07_01_LO, 1.0);
    // cos at pi (0x1.921FB54442D18P+1) is the min -1; ambiguity widening admits it.
    encloses(iv(PI, PI).cos(), -1.0, -ONE_MINUS_ULP);
}

// --- Bare tan --------------------------------------------------------------

#[test]
fn tan_monotone_and_pole_vectors() {
    // tan [empty] = empty; tan [entire] = entire; tan [0,inf] = entire.
    assert!(Interval::<f64>::empty().tan().is_empty());
    assert!(Interval::<f64>::entire().tan().is_entire());
    assert!(iv(0.0, INF).tan().is_entire());
    // tan [0,0] = [0,0].
    encloses(iv(0.0, 0.0).tan(), 0.0, 0.0);
    // tan [-0x1.555475A31A4BEP-2, 0x1.999999999999AP-4] monotone branch.
    encloses(iv(TAN_IN_LO, TAN_IN_HI).tan(), TAN_OUT_LO, TAN_OUT_HI);
    // A pole straddle (pi/2 down to pi/2 up) returns entire.
    assert!(iv(FRAC_PI_2, FRAC_PI_2.next_up()).tan().is_entire());
    // tan [1,2] straddles the pi/2 pole -> entire.
    assert!(iv(1.0, 2.0).tan().is_entire());
    // Large-argument pole enumeration: a mid-branch interval near 5345 is finite,
    // a wider one that reaches the next pole is entire.
    encloses(iv(TAN_BIG_A, TAN_BIG_B).tan(), TAN_BIG_LO, TAN_BIG_HI);
    assert!(iv(TAN_BIG_A, TAN_BIG_B2).tan().is_entire());
}

// --- Bare hyperbolic -------------------------------------------------------

#[test]
fn hyperbolic_vectors() {
    // sinh [entire] = entire; sinh [-inf,0] = [-inf,0]; monotone.
    assert!(Interval::<f64>::entire().sinh().is_entire());
    encloses(iv(1.0, SINH_B).sinh(), SINH_LO, SINH_HI);
    // cosh [entire] = [1,inf]; cosh [-1.1,2.3] = [1, 0x1.4261D2B7D6181P+2].
    let ce = Interval::<f64>::entire().cosh();
    assert!(ce.inf() <= 1.0 && ce.sup() == INF);
    encloses(iv(COSH_A, COSH_B).cosh(), 1.0, COSH_HI);
    // tanh [entire] = [-1,1]; tanh [1, 300.56] = [0x1.85EFAB514F394P-1, 1].
    encloses(Interval::<f64>::entire().tanh(), -1.0, 1.0);
    encloses(iv(1.0, SINH_B).tanh(), TANH_LO, 1.0);
    // tanh saturates but never escapes (-1,1) -> clamped into [-1,1].
    let t = iv(-1.0e300, 1.0e300).tanh();
    assert!(t.inf() >= -1.0 && t.sup() <= 1.0);
}

// --- Bare pow --------------------------------------------------------------

#[test]
fn pow_vectors() {
    // Empty absorbs either operand.
    assert!(Interval::<f64>::empty().pow(Interval::entire()).is_empty());
    assert!(iv(0.1, 0.5).pow(Interval::empty()).is_empty());
    // pow [0.1,0.5] [0,0] = [1,1]; pow [x] [entire] = [0,inf].
    encloses(iv(0.1, 0.5).pow(iv(0.0, 0.0)), 1.0, 1.0);
    encloses(iv(0.1, 0.5).pow(Interval::entire()), 0.0, INF);
    // pow [0.1,0.5] [0.1,0.1] = [0x1.96B230BCDC434P-1, 0x1.DDB680117AB13P-1].
    encloses(iv(0.1, 0.5).pow(iv(0.1, 0.1)), POW_LO, POW_HI);
    // pow [0.1,0.5] [-2.5,2.5] = [0x1.9E7C..P-9, 0x1.3C3A..P+8], the four corners.
    encloses(iv(0.1, 0.5).pow(iv(-2.5, 2.5)), POW_WIDE_LO, POW_WIDE_HI);
    // Base reaching zero: pow [0,0.5] [1,1] = [0,0.5]; the zero column contributes 0.
    encloses(iv(0.0, 0.5).pow(iv(1.0, 1.0)), 0.0, 0.5);
    // Base exactly zero with a positive exponent: pow [0,0] [1,inf] = [0,0] (NOT empty).
    let p = iv(0.0, 0.0).pow(iv(1.0, INF));
    assert!(!p.is_empty());
    encloses(p, 0.0, 0.0);
    // Base entirely negative: empty image.
    assert!(iv(-1.0, -0.1).pow(iv(-0.1, 1.0)).is_empty());
    // Base zero with no positive exponent: empty defined image.
    assert!(iv(0.0, 0.0).pow(iv(-1.0, 0.0)).is_empty());
}

// --- Bare rootn (derived) ---------------------------------------------------

#[test]
fn rootn_vectors() {
    // Derived from integer arithmetic, not transcribed from any vector set
    // (the corpus's only rootn cases live in the LGPL c-xsc.itl, which this
    // permissive tree does not adapt; see the registry entry): 5^3 = 125,
    // 0^5 = 0, 3^10 = 59049, and 2^4 = 16 with 3^4 = 81, so each root below
    // is the exact integer preimage.
    encloses(iv(125.0, 125.0).rootn(3), 5.0, 5.0);
    encloses(iv(0.0, 0.0).rootn(5), 0.0, 0.0);
    encloses(iv(59049.0, 59049.0).rootn(10), 3.0, 3.0);
    encloses(iv(16.0, 81.0).rootn(4), 2.0, 3.0);
    // Derived: odd root of a negative base is real; even root drops the negative part.
    encloses(iv(-8.0, -8.0).rootn(3), -2.0, -2.0);
    encloses(iv(-8.0, 27.0).rootn(3), -2.0, 3.0);
    assert!(iv(-4.0, -1.0).rootn(2).is_empty());
    encloses(iv(-4.0, 16.0).rootn(2), 0.0, 2.0);
    // n = 0 is undefined -> empty.
    assert!(iv(1.0, 2.0).rootn(0).is_empty());
}

// --- Decorated vectors -----------------------------------------------------

#[test]
fn decorated_sin_cos_vectors() {
    // sin/cos [entire]_dac = [-1,1]_dac (defined-continuous but unbounded input).
    dec_encloses(
        DecoratedInterval::<f64>::new_dec(Interval::entire()).sin(),
        -1.0,
        1.0,
        Decoration::Dac,
    );
    dec_encloses(
        DecoratedInterval::<f64>::new_dec(Interval::entire()).cos(),
        -1.0,
        1.0,
        Decoration::Dac,
    );
    // A bounded pole-free sin stays com.
    dec_encloses(
        di(1.0, 2.0, Decoration::Com).sin(),
        SIN_1_2_LO,
        1.0,
        Decoration::Com,
    );
    // NaI poisons.
    assert!(DecoratedInterval::<f64>::nai().sin().is_nai());
    assert!(DecoratedInterval::<f64>::nai().cos().is_nai());
}

#[test]
fn decorated_tan_pole_transitions() {
    // tan [empty]_trv = empty_trv.
    let e = DecoratedInterval::<f64>::new_dec(Interval::empty()).tan();
    assert!(e.interval().is_empty() && e.decoration() == Decoration::Trv);
    // tan [0,inf]_dac = entire_trv (unbounded -> spans poles -> domain violation).
    let r = di(0.0, INF, Decoration::Dac).tan();
    assert!(r.interval().is_entire() && r.decoration() == Decoration::Trv);
    // tan pole straddle [pi/2, pi/2.next_up]_dac = entire_trv.
    let r = di(FRAC_PI_2, FRAC_PI_2.next_up(), Decoration::Dac).tan();
    assert!(r.interval().is_entire() && r.decoration() == Decoration::Trv);
    // tan [0,0]_com = [0,0]_com (pole-free, bounded).
    let r = di(0.0, 0.0, Decoration::Com).tan();
    encloses(r.interval(), 0.0, 0.0);
    assert_eq!(r.decoration(), Decoration::Com);
    assert!(DecoratedInterval::<f64>::nai().tan().is_nai());
}

#[test]
fn decorated_hyperbolic_vectors() {
    // sinh [entire]_dac = entire_dac.
    let r = di(NEG_INF, INF, Decoration::Dac).sinh();
    assert!(r.interval().is_entire() && r.decoration() == Decoration::Dac);
    // cosh [entire]_def = [1,inf]_def (input def, unbounded result -> dac local ->
    // meet with def stays def).
    let r = di(NEG_INF, INF, Decoration::Def).cosh();
    assert!(r.interval().sup() == INF && r.decoration() == Decoration::Def);
    // A bounded cosh input stays com; an overflowed (unbounded) result demotes.
    dec_encloses(
        di(COSH_A, COSH_B, Decoration::Com).cosh(),
        1.0,
        COSH_HI,
        Decoration::Com,
    );
    // tanh [entire]_dac = [-1,1]_dac.
    dec_encloses(
        di(NEG_INF, INF, Decoration::Dac).tanh(),
        -1.0,
        1.0,
        Decoration::Dac,
    );
    assert!(DecoratedInterval::<f64>::nai().sinh().is_nai());
}

#[test]
fn decorated_pow_vectors() {
    // pow [0.1,0.5]_com [0,1]_com = [.., 1]_com (base strictly positive, bounded).
    let r = di(0.1, 0.5, Decoration::Com).pow(di(0.0, 1.0, Decoration::Com));
    assert_eq!(r.decoration(), Decoration::Com);
    // pow [0,1]_com [0,0]_com = [1,1]_trv (base reaches 0 with a non-positive exp:
    // 0^0 is undefined).
    let r = di(0.0, 1.0, Decoration::Com).pow(di(0.0, 0.0, Decoration::Com));
    encloses(r.interval(), 1.0, 1.0);
    assert_eq!(r.decoration(), Decoration::Trv);
    // pow [0.1,0.5]_com [-2.5,inf]_dac = [0,..]_dac (base positive, exp unbounded).
    let r = di(0.1, 0.5, Decoration::Com).pow(di(-2.5, INF, Decoration::Dac));
    assert_eq!(r.decoration(), Decoration::Dac);
    // pow [0,0.5]_com [-inf,-2.5]_dac = [.., inf]_trv (base 0 with negative exp).
    let r = di(0.0, 0.5, Decoration::Com).pow(di(NEG_INF, -2.5, Decoration::Dac));
    assert_eq!(r.decoration(), Decoration::Trv);
    // pow [-1,-0.1]_com [-0.1,1]_def = empty_trv (base entirely negative).
    let r = di(-1.0, -0.1, Decoration::Com).pow(di(-0.1, 1.0, Decoration::Def));
    assert!(r.interval().is_empty() && r.decoration() == Decoration::Trv);
    assert!(DecoratedInterval::<f64>::nai()
        .pow(di(1.0, 2.0, Decoration::Com))
        .is_nai());
}

// --- Property lanes --------------------------------------------------------

proptest! {
    /// Pointwise enclosure for sin and cos over magnitudes including large
    /// arguments (the reduction stress well past 1e10).
    #[test]
    fn sin_cos_enclose_pointwise(
        a in -1.0e10f64..1.0e10,
        w in 0.0f64..12.0,
        t in 0.0f64..1.0,
    ) {
        let x = a + w * t;
        let s = iv(a, a + w).sin();
        prop_assert!(s.contains(x.sin()), "sin({x}) escaped [{}, {}]", s.inf(), s.sup());
        prop_assert!(s.inf() >= -1.0 && s.sup() <= 1.0, "sin left [-1,1]");
        let c = iv(a, a + w).cos();
        prop_assert!(c.contains(x.cos()), "cos({x}) escaped [{}, {}]", c.inf(), c.sup());
        prop_assert!(c.inf() >= -1.0 && c.sup() <= 1.0, "cos left [-1,1]");
    }

    /// A width of at least a full period yields the full range exactly.
    #[test]
    fn full_period_shortcut_is_full_range(
        a in -1.0e6f64..1.0e6,
        extra in 0.0f64..100.0,
    ) {
        let w = 2.0 * core::f64::consts::PI + extra;
        let s = iv(a, a + w).sin();
        prop_assert!(s.inf() <= -1.0 && s.sup() >= 1.0, "sin not full range");
        let c = iv(a, a + w).cos();
        prop_assert!(c.inf() <= -1.0 && c.sup() >= 1.0, "cos not full range");
    }

    /// Critical-point stress: an interval within 1e-9 of a `k*pi/2` boundary must
    /// stay a sound enclosure, and the ambiguity-widening rule must admit the
    /// extremum whenever the true critical point is inside.
    #[test]
    fn critical_point_stress(
        k in -1000i32..1000,
        delta in -1.0e-9f64..1.0e-9,
        halfwidth in 1.0e-12f64..1.0e-6,
    ) {
        let center = f64::from(k) * FRAC_PI_2 + delta;
        let (a, b) = (center - halfwidth, center + halfwidth);
        let s = iv(a, b).sin();
        // Soundness: three interior samples are enclosed.
        for &t in &[0.0f64, 0.5, 1.0] {
            let x = a + (b - a) * t;
            prop_assert!(s.contains(x.sin()), "sin({x}) escaped near critical point");
        }
        // Ambiguity widening: if a true sin extremum (odd k, at pi/2 + m*pi) lies
        // in [a, b], +-1 must be admitted.
        if k % 2 != 0 {
            let extremum = if k.rem_euclid(4) == 1 { 1.0 } else { -1.0 };
            let true_crit = f64::from(k) * FRAC_PI_2;
            if a <= true_crit && true_crit <= b {
                if extremum > 0.0 {
                    prop_assert!(s.sup() >= 1.0, "max +1 not admitted");
                } else {
                    prop_assert!(s.inf() <= -1.0, "min -1 not admitted");
                }
            }
        }
    }

    /// tanh, sinh pointwise enclosure over the whole line including overflow.
    #[test]
    fn sinh_tanh_enclose_pointwise(
        a in -800.0f64..800.0,
        w in 0.0f64..50.0,
        t in 0.0f64..1.0,
    ) {
        let x = a + w * t;
        let sh = iv(a, a + w).sinh();
        prop_assert!(sh.contains(x.sinh()), "sinh({x}) escaped [{}, {}]", sh.inf(), sh.sup());
        let th = iv(a, a + w).tanh();
        prop_assert!(th.contains(x.tanh()), "tanh({x}) escaped [{}, {}]", th.inf(), th.sup());
        prop_assert!(th.inf() >= -1.0 && th.sup() <= 1.0, "tanh left [-1,1]");
    }

    /// cosh pointwise enclosure, and the minimum: 1 is in the image exactly when 0
    /// is in the input (and the lower bound is always at least 1, always sound).
    #[test]
    fn cosh_min_and_enclosure(
        a in -20.0f64..20.0,
        w in 0.0f64..40.0,
        t in 0.0f64..1.0,
    ) {
        let c = iv(a, a + w).cosh();
        let x = a + w * t;
        prop_assert!(c.contains(x.cosh()), "cosh({x}) escaped [{}, {}]", c.inf(), c.sup());
        prop_assert!(c.inf() >= 1.0, "cosh floor below 1");
        if a <= 0.0 && 0.0 <= a + w {
            prop_assert!(c.contains(1.0), "cosh min 1 not in image when 0 in input");
        }
    }

    /// pow pointwise enclosure over the (base, exponent) sign-case grid, base > 0.
    #[test]
    fn pow_encloses_pointwise(
        xl in 1.0e-3f64..3.0,
        xw in 0.0f64..3.0,
        yl in -4.0f64..4.0,
        yw in 0.0f64..4.0,
        s in 0.0f64..1.0,
        u in 0.0f64..1.0,
    ) {
        let (xh, yh) = (xl + xw, yl + yw);
        let r = iv(xl, xh).pow(iv(yl, yh));
        let (x, y) = (xl + xw * s, yl + yw * u);
        prop_assert!(r.contains(x.powf(y)), "{x}^{y} escaped [{}, {}]", r.inf(), r.sup());
    }

    /// The four-corner `pow` arm is never wider than the exp/ln composition
    /// `exp(Y * ln(X))` it replaced: on a strictly positive base both are sound
    /// enclosures of the same image, the composition pays extra rounding through
    /// `ln`, the product, and `exp`, and the direct arm pays one widening, so the
    /// direct result must be a subset (per the ADR-0005 part 3 rework; the
    /// composition was part 4's affine seam, misread as the interval arm).
    #[test]
    fn pow_four_corner_within_composition(
        xl in 1.0e-3f64..3.0,
        xw in 0.0f64..3.0,
        yl in -4.0f64..4.0,
        yw in 0.0f64..4.0,
    ) {
        let base = iv(xl, xl + xw);
        let exponent = iv(yl, yl + yw);
        let direct = base.pow(exponent);
        let composed = (exponent * base.ln()).exp();
        prop_assert!(
            composed.inf() <= direct.inf() && direct.sup() <= composed.sup(),
            "four-corner [{}, {}] wider than composition [{}, {}]",
            direct.inf(),
            direct.sup(),
            composed.inf(),
            composed.sup()
        );
    }

    /// rootn pointwise enclosure across the parity split.
    #[test]
    fn rootn_encloses_pointwise(
        a in -50.0f64..50.0,
        w in 0.0f64..50.0,
        n in 1u32..8,
        t in 0.0f64..1.0,
    ) {
        let r = iv(a, a + w).rootn(n);
        let x = a + w * t;
        let reference = if n % 2 == 0 {
            if x < 0.0 { None } else { Some(x.powf(1.0 / f64::from(n))) }
        } else if x < 0.0 {
            Some(-(-x).powf(1.0 / f64::from(n)))
        } else {
            Some(x.powf(1.0 / f64::from(n)))
        };
        if let Some(ref_val) = reference {
            prop_assert!(r.contains(ref_val), "rootn({x}, {n}) = {ref_val} escaped [{}, {}]", r.inf(), r.sup());
        }
    }

    /// A tan interval straddling a pole returns entire.
    #[test]
    fn tan_pole_straddle_is_entire(k in -100i32..100) {
        // The pole is at pi/2 + k*pi; bracket it tightly.
        let pole = FRAC_PI_2 + f64::from(k) * PI;
        let r = iv(pole - 1.0e-6, pole + 1.0e-6).tan();
        prop_assert!(r.is_entire(), "tan straddling pole at k={k} not entire");
    }
}
