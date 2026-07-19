//! Tests for the text I/O surface over the `f64` fixture: `nums_to_interval`,
//! `text_to_interval` (bare and decorated), the exact hex pair, and `Display`.
//!
//! The unit tests transcribe the vendored ITF1788 constructor vectors
//! (`ieee1788-constructors.itl`, `ieee1788-exceptions.itl`) and the
//! `textToInterval`/`numsToInterval` cases of `libieeep1788_class.itl`. The
//! crate reports the standard's `UndefinedOperation` signal as a `Result::Err`
//! rather than raising it, so each `= [...] signal UndefinedOperation` vector is
//! transcribed as an expected `Err` (the documented value-versus-signal
//! divergence). A `= [...] signal PossiblyUndefinedOperation` vector is a valid
//! result with a warning-level signal, so it is transcribed as the expected `Ok`
//! interval. A `[nai]` literal is a decorated value and is expected `Ok(nai)`.
//!
//! Expected hex-float endpoints cannot go through `str::parse` (Rust does not
//! read hex floats), so a vector's expected interval is obtained by parsing its
//! canonical string through this crate. That makes each decimal-input vector a
//! cross-check of the decimal directed-rounding path against the independent hex
//! path (both must land on the same `f64`). The directed decimal core is
//! validated independently against the correctly-rounded `str::parse` oracle, and
//! the hex path against the exact round-trip, in the property lanes below.
//!
//! # The `libieeep1788_class.itl` decorated-constructor testcases
//!
//! Beyond `numsToInterval`/`textToInterval`, this file transcribes the class
//! file's decorated-interval constructors and accessors: `newDec`,
//! `intervalPart` (the crate's `interval()`), `decorationPart` (the crate's
//! `decoration()`), and `setDec`. An input interval is parsed through the crate
//! (so hex endpoints stay exact) and the operation is applied directly; the
//! decoration is asserted exactly, since these constructors never round.
//!
//! `intervalPart([nai])` is expected `[empty]` here: the standard raises an
//! information-loss `IntvlPartOfNaI` signal that the crate does not (the same
//! value-versus-signal divergence the `UndefinedOperation` cases carry).
//!
//! `setDec` has a REQUIRED-OPERATION divergence the conformance lane surfaces:
//! the standard's `setDec` CLAMPS an inconsistent (interval, decoration) pair
//! (an empty interval forces `trv`, an unbounded interval demotes `com` to
//! `dac`), whereas the crate's `set_dec` returns `NaI` for any inconsistent
//! combination. The 16 consistent vectors pass; the 6 clamping vectors are a
//! documented `KNOWN GAP` (`set_dec_clamping_divergence_known_gap`, `#[ignore]`d),
//! asserting the standard datum so the test goes green if the divergence is ever
//! resolved. No library fix lands in this slice.

use interval_1788::text_io::{TextError, MAX_EXP_DIGITS, MAX_INPUT_LEN, MAX_SIG_DIGITS};
use interval_1788::{DecoratedInterval, Decoration, Interval};
use proptest::prelude::*;

// --- helpers ---------------------------------------------------------------

fn p(s: &str) -> Interval<f64> {
    Interval::text_to_interval(s).unwrap_or_else(|e| panic!("parse {s:?} failed: {e:?}"))
}

/// Assert an input parses to the same interval as an expected canonical string.
fn chk(input: &str, expected: &str) {
    assert_eq!(
        p(input),
        p(expected),
        "input {input:?} vs expected {expected:?}"
    );
}

/// Assert a bare `text_to_interval` rejects an input (the value form of the
/// standard's `UndefinedOperation` signal).
fn bad(input: &str) {
    assert!(
        Interval::text_to_interval(input).is_err(),
        "expected {input:?} to be rejected"
    );
}

fn pd(s: &str) -> DecoratedInterval<f64> {
    DecoratedInterval::text_to_interval(s).unwrap_or_else(|e| panic!("dparse {s:?} failed: {e:?}"))
}

fn chk_d(input: &str, expected: &str) {
    assert_eq!(pd(input), pd(expected), "dinput {input:?} vs {expected:?}");
}

fn bad_d(input: &str) {
    assert!(
        DecoratedInterval::text_to_interval(input).is_err(),
        "expected decorated {input:?} to be rejected"
    );
}

// --- nums_to_interval (class + constructor + exceptions) -------------------

#[test]
fn nums_to_interval_bare() {
    // minimal_nums_to_interval_test: 8 vectors (plus one exceptions.itl row)
    let ni = |l, u| Interval::<f64>::nums_to_interval(l, u);
    assert_eq!(ni(-1.0, 1.0), Interval::new(-1.0, 1.0));
    assert_eq!(
        ni(f64::NEG_INFINITY, 1.0),
        Interval::new(f64::NEG_INFINITY, 1.0)
    );
    assert_eq!(ni(-1.0, f64::INFINITY), Interval::new(-1.0, f64::INFINITY));
    assert!(ni(f64::NEG_INFINITY, f64::INFINITY).unwrap().is_entire());

    // signal UndefinedOperation -> Err.
    assert!(ni(f64::NAN, f64::NAN).is_err());
    assert!(ni(1.0, -1.0).is_err());
    assert!(ni(f64::NEG_INFINITY, f64::NEG_INFINITY).is_err());
    assert!(ni(f64::INFINITY, f64::INFINITY).is_err());
    assert!(ni(f64::INFINITY, f64::NEG_INFINITY).is_err()); // exceptions.itl
}

#[test]
fn nums_to_interval_decorated() {
    // minimal_nums_to_decorated_interval_test: 8 vectors
    let dc = |l, u| {
        DecoratedInterval::<f64>::nums_to_interval(l, u)
            .unwrap()
            .decoration()
    };
    assert_eq!(dc(-1.0, 1.0), Decoration::Com);
    assert_eq!(dc(f64::NEG_INFINITY, 1.0), Decoration::Dac);
    assert_eq!(dc(-1.0, f64::INFINITY), Decoration::Dac);
    assert_eq!(dc(f64::NEG_INFINITY, f64::INFINITY), Decoration::Dac);
    // signal UndefinedOperation -> Err (in place of a returned NaI).
    assert!(DecoratedInterval::<f64>::nums_to_interval(f64::NAN, f64::NAN).is_err());
    assert!(DecoratedInterval::<f64>::nums_to_interval(1.0, -1.0).is_err());
    assert!(
        DecoratedInterval::<f64>::nums_to_interval(f64::NEG_INFINITY, f64::NEG_INFINITY).is_err()
    );
    assert!(DecoratedInterval::<f64>::nums_to_interval(f64::INFINITY, f64::INFINITY).is_err());
}

// --- class: newDec / intervalPart / decorationPart / setDec ----------------
//
// The decorated-interval constructors and accessors of libieeep1788_class.itl.
// An input interval is parsed through the crate (hex endpoints stay exact) and
// the operation applied directly; decorations are pinned exactly.

#[test]
fn new_dec_vectors() {
    // minimal_new_dec_test: 13 vectors
    let com = |s: &str| {
        let d = DecoratedInterval::new_dec(p(s));
        assert_eq!(d.interval(), p(s), "newDec interval part for {s}");
        assert_eq!(d.decoration(), Decoration::Com, "newDec decoration for {s}");
    };
    com("[-0x1.99999A842549Ap+4,0X1.9999999999999P-4]");
    com("[-0X1.99999C0000000p+4,0X1.9999999999999P-4]");
    com("[-0x1.99999A842549Ap+4,0x1.99999A0000000p-4]");
    com("[-0X1.99999C0000000p+4,0x1.99999A0000000p-4]");
    com("[-0x0.0000000000001p-1022,-0x0.0000000000001p-1022]");
    com("[-0x0.0000000000001p-1022,0x0.0000000000001p-1022]");
    com("[0x0.0000000000001p-1022,0x0.0000000000001p-1022]");
    com("[-0x1.fffffffffffffp+1023,-0x1.fffffffffffffp+1023]");
    com("[-0x1.fffffffffffffp+1023,0x1.fffffffffffffp+1023]");
    com("[0x1.fffffffffffffp+1023,0x1.fffffffffffffp+1023]");
    // Unbounded nonempty earns dac; empty earns trv.
    let entire = DecoratedInterval::new_dec(Interval::<f64>::entire());
    assert!(entire.interval().is_entire());
    assert_eq!(entire.decoration(), Decoration::Dac);
    let empty = DecoratedInterval::new_dec(Interval::<f64>::empty());
    assert!(empty.interval().is_empty());
    assert_eq!(empty.decoration(), Decoration::Trv);
    com("[-0X1.99999C0000000p+4,0X1.9999999999999P-4]");
}

#[test]
fn interval_part_vectors() {
    // minimal_interval_part_test: 14 vectors. intervalPart = interval(). Built
    // via set_dec with the corpus decoration (all consistent here), so interval()
    // returns the parsed interval unchanged.
    let ip = |s: &str, d: Decoration| {
        assert_eq!(
            DecoratedInterval::set_dec(p(s), d).interval(),
            p(s),
            "intervalPart {s}"
        );
    };
    ip(
        "[-0x1.99999A842549Ap+4,0X1.9999999999999P-4]",
        Decoration::Trv,
    );
    ip(
        "[-0X1.99999C0000000p+4,0X1.9999999999999P-4]",
        Decoration::Com,
    );
    ip(
        "[-0x1.99999A842549Ap+4,0x1.99999A0000000p-4]",
        Decoration::Dac,
    );
    ip(
        "[-0X1.99999C0000000p+4,0x1.99999A0000000p-4]",
        Decoration::Def,
    );
    ip(
        "[-0x0.0000000000001p-1022,-0x0.0000000000001p-1022]",
        Decoration::Trv,
    );
    ip(
        "[-0x0.0000000000001p-1022,0x0.0000000000001p-1022]",
        Decoration::Trv,
    );
    ip(
        "[0x0.0000000000001p-1022,0x0.0000000000001p-1022]",
        Decoration::Trv,
    );
    ip(
        "[-0x1.fffffffffffffp+1023,-0x1.fffffffffffffp+1023]",
        Decoration::Trv,
    );
    ip(
        "[-0x1.fffffffffffffp+1023,0x1.fffffffffffffp+1023]",
        Decoration::Trv,
    );
    ip(
        "[0x1.fffffffffffffp+1023,0x1.fffffffffffffp+1023]",
        Decoration::Trv,
    );
    assert!(
        DecoratedInterval::set_dec(Interval::<f64>::entire(), Decoration::Trv)
            .interval()
            .is_entire()
    );
    assert!(
        DecoratedInterval::set_dec(Interval::<f64>::empty(), Decoration::Trv)
            .interval()
            .is_empty()
    );
    ip(
        "[-0X1.99999C0000000p+4,0X1.9999999999999P-4]",
        Decoration::Com,
    );
    // intervalPart[nai] = [empty]; the IntvlPartOfNaI signal is not raised.
    assert!(DecoratedInterval::<f64>::nai().interval().is_empty());
}

#[test]
fn decoration_part_vectors() {
    // minimal_decoration_part_test: 6 vectors. decorationPart = decoration().
    assert_eq!(
        DecoratedInterval::<f64>::nai().decoration(),
        Decoration::Ill
    );
    assert_eq!(
        DecoratedInterval::set_dec(Interval::<f64>::empty(), Decoration::Trv).decoration(),
        Decoration::Trv
    );
    assert_eq!(
        DecoratedInterval::set_dec(p("[-1.0,3.0]"), Decoration::Trv).decoration(),
        Decoration::Trv
    );
    assert_eq!(
        DecoratedInterval::set_dec(p("[-1.0,3.0]"), Decoration::Def).decoration(),
        Decoration::Def
    );
    assert_eq!(
        DecoratedInterval::set_dec(p("[-1.0,3.0]"), Decoration::Dac).decoration(),
        Decoration::Dac
    );
    assert_eq!(
        DecoratedInterval::set_dec(p("[-1.0,3.0]"), Decoration::Com).decoration(),
        Decoration::Com
    );
}

#[test]
fn set_dec_vectors() {
    // minimal_set_dec_test: 16 of 22 vectors (the 6 clamping vectors are a
    // documented KNOWN GAP; see set_dec_clamping_divergence_known_gap).
    let ok = |s: &str, d: Decoration, want: Decoration| {
        let r = DecoratedInterval::set_dec(p(s), d);
        assert!(!r.is_nai(), "setDec {s} {d:?} unexpectedly NaI");
        assert_eq!(r.interval(), p(s), "setDec {s} interval part");
        assert_eq!(r.decoration(), want, "setDec {s} decoration");
    };
    ok(
        "[-0x1.99999A842549Ap+4,0X1.9999999999999P-4]",
        Decoration::Trv,
        Decoration::Trv,
    );
    ok(
        "[-0X1.99999C0000000p+4,0X1.9999999999999P-4]",
        Decoration::Com,
        Decoration::Com,
    );
    ok(
        "[-0x1.99999A842549Ap+4,0x1.99999A0000000p-4]",
        Decoration::Dac,
        Decoration::Dac,
    );
    ok(
        "[-0X1.99999C0000000p+4,0x1.99999A0000000p-4]",
        Decoration::Def,
        Decoration::Def,
    );
    ok(
        "[-0x0.0000000000001p-1022,-0x0.0000000000001p-1022]",
        Decoration::Trv,
        Decoration::Trv,
    );
    ok(
        "[-0x0.0000000000001p-1022,0x0.0000000000001p-1022]",
        Decoration::Def,
        Decoration::Def,
    );
    ok(
        "[0x0.0000000000001p-1022,0x0.0000000000001p-1022]",
        Decoration::Dac,
        Decoration::Dac,
    );
    ok(
        "[-0x1.fffffffffffffp+1023,-0x1.fffffffffffffp+1023]",
        Decoration::Com,
        Decoration::Com,
    );
    ok(
        "[-0x1.fffffffffffffp+1023,0x1.fffffffffffffp+1023]",
        Decoration::Def,
        Decoration::Def,
    );
    ok(
        "[0x1.fffffffffffffp+1023,0x1.fffffffffffffp+1023]",
        Decoration::Trv,
        Decoration::Trv,
    );
    let e = DecoratedInterval::set_dec(Interval::<f64>::entire(), Decoration::Dac);
    assert!(e.interval().is_entire());
    assert_eq!(e.decoration(), Decoration::Dac);
    let em = DecoratedInterval::set_dec(Interval::<f64>::empty(), Decoration::Trv);
    assert!(em.interval().is_empty());
    assert_eq!(em.decoration(), Decoration::Trv);
    ok(
        "[-0X1.99999C0000000p+4,0X1.9999999999999P-4]",
        Decoration::Com,
        Decoration::Com,
    );
    // setDec X ill = [nai]: the crate returns NaI (value matches; the
    // UndefinedOperation signal is not raised).
    assert!(DecoratedInterval::set_dec(Interval::<f64>::empty(), Decoration::Ill).is_nai());
    assert!(DecoratedInterval::set_dec(p("[-inf,3.0]"), Decoration::Ill).is_nai());
    assert!(DecoratedInterval::set_dec(p("[-1.0,3.0]"), Decoration::Ill).is_nai());
}

#[test]
#[ignore = "KNOWN GAP: set_dec returns NaI for inconsistent combos; the standard's setDec clamps"]
fn set_dec_clamping_divergence_known_gap() {
    // minimal_set_dec_test, the 6 clamping vectors. The standard's setDec forces
    // an empty interval to trv and demotes com to dac on an unbounded interval,
    // returning a valid decorated interval; the crate's set_dec instead returns
    // NaI. These assertions hold the STANDARD datum, so the test goes green if the
    // divergence is ever resolved; today the `!is_nai()` assertions fail, hence
    // #[ignore]. No library fix lands in this slice. Never adjust expected values.
    let clamp = |x: Interval<f64>, d: Decoration, want: Decoration| {
        let r = DecoratedInterval::set_dec(x, d);
        assert!(!r.is_nai());
        assert_eq!(r.decoration(), want);
    };
    // setDec [empty] {def,dac,com} = [empty]_trv
    clamp(Interval::empty(), Decoration::Def, Decoration::Trv);
    clamp(Interval::empty(), Decoration::Dac, Decoration::Trv);
    clamp(Interval::empty(), Decoration::Com, Decoration::Trv);
    // setDec [unbounded] com = [unbounded]_dac
    clamp(p("[1.0,+inf]"), Decoration::Com, Decoration::Dac);
    clamp(p("[-inf,3.0]"), Decoration::Com, Decoration::Dac);
    clamp(Interval::entire(), Decoration::Com, Decoration::Dac);
}

// --- exact anchors against plain f64 literals (independent ground truth) ----

#[test]
fn exact_representable_anchors() {
    assert_eq!(p("[1.5]"), Interval::new(1.5, 1.5).unwrap());
    assert_eq!(p("[0x1.8p+0]"), Interval::new(1.5, 1.5).unwrap()); // 1.5 in hex
    assert_eq!(p("[1/2]"), Interval::new(0.5, 0.5).unwrap());
    assert_eq!(p("[-4/2, 10/5]"), Interval::new(-2.0, 2.0).unwrap());
    assert_eq!(p("[-1/10, 1/10]"), Interval::new(-0.1, 0.1).unwrap()); // 0.1 == the f64 literal
    assert_eq!(p("[-1.0, 1.0]"), Interval::new(-1.0, 1.0).unwrap());
    assert_eq!(p("[1,1000]"), Interval::new(1.0, 1000.0).unwrap());
    assert_eq!(p("3.56?1e2"), Interval::new(355.0, 357.0).unwrap());
    assert_eq!(p("-10?12"), Interval::new(-22.0, 2.0).unwrap());
    assert_eq!(p("10?3"), Interval::new(7.0, 13.0).unwrap());
    assert_eq!(p("0.0?"), Interval::new(-0.05, 0.05).unwrap()); // 0.05 == its f64 literal
    assert_eq!(p("0.000?5"), Interval::new(-0.005, 0.005).unwrap());
}

// --- bracketed keyword forms -----------------------------------------------

#[test]
fn keyword_forms() {
    for s in ["[empty]", "[]", "[ empty ]", "[ Empty  ]", "[  ]"] {
        assert!(p(s).is_empty(), "{s} should be empty");
    }
    for s in [
        "[entire]",
        "[,]",
        "[ entire ]",
        "[ ENTIRE ]",
        "[ -inf , INF  ]",
    ] {
        assert!(p(s).is_entire(), "{s} should be entire");
    }
}

// --- valid literals from the vector sets -----------------------------------

#[test]
fn constructor_and_class_valid() {
    // ieee1788-constructors.itl
    chk("[1.2345]", "[0X1.3C083126E978DP+0, 0X1.3C083126E978EP+0]");
    chk("[1,+infinity]", "[1.0, infinity]");
    chk(
        "[1.e-3, 1.1e-3]",
        "[0X4.189374BC6A7ECP-12, 0X4.816F0068DB8BCP-12]",
    );
    chk(
        "[-0x1.3p-1, 2/3]",
        "[-0X9.8000000000000P-4, +0XA.AAAAAAAAAAAB0P-4]",
    );
    chk("[3.56]", "[0X3.8F5C28F5C28F4P+0, 0X3.8F5C28F5C28F6P+0]");
    chk("3.56?1", "[0X3.8CCCCCCCCCCCCP+0, 0X3.91EB851EB8520P+0]");
    chk("3.560?2", "[0X3.8ED916872B020P+0, 0X3.8FDF3B645A1CCP+0]");
    chk("3.56?", "[0X3.8E147AE147AE0P+0, 0X3.90A3D70A3D70CP+0]");
    chk("3.560?2u", "[0X3.8F5C28F5C28F4P+0, 0X3.8FDF3B645A1CCP+0]");
    chk("-10?", "[-10.5, -9.5]");
    chk("-10?u", "[-10.0, -9.5]");
    chk("[1.234e5,Inf]", "[123400.0, infinity]");
    chk("3.1416?1", "[0X3.24395810624DCP+0, 0X3.24467381D7DC0P+0]");

    // libieeep1788_class.itl textToInterval
    chk("[  -1.0  ,  1.0  ]", "[-1.0, 1.0]");
    chk("[-1,]", "[-1.0, infinity]");
    chk("[-1.0, +inf]", "[-1.0, infinity]");
    chk("[-Inf, 1.000 ]", "[-infinity, 1.0]");
    chk("[-Infinity, 1.000 ]", "[-infinity, 1.0]");
    chk("[1.0E+400 ]", "[0x1.fffffffffffffp+1023, infinity]");
    chk("[ -4/2, 10/5 ]", "[-2.0, 2.0]");
    chk("0.0?u", "[0.0, 0.05]");
    chk("0.0?d", "[-0.05, 0.0]");
    chk("2.5?", "[0x1.3999999999999p+1, 0x1.4666666666667p+1]");
    chk("2.5?u", "[2.5, 0x1.4666666666667p+1]");
    chk("2.5?d", "[0x1.3999999999999p+1, 2.5]");
    chk("0.000?5u", "[0.0, 0.005]");
    chk("2.500?5", "[0x1.3f5c28f5c28f5p+1, 0x1.40a3d70a3d70bp+1]");
    chk("0.0??", "[-infinity, infinity]");
    chk("0.0??u", "[0.0, infinity]");
    chk("0.0??d", "[-infinity, 0.0]");
    chk("2.5??u", "[2.5, infinity]");
    chk("2.5??d", "[-infinity, 2.5]");
    chk(
        "2.500?5e+27",
        "[0x1.01fa19a08fe7fp+91, 0x1.0302cc4352683p+91]",
    );
    chk("2.500?5ue4", "[0x1.86ap+14, 0x1.8768p+14]");
    chk(
        "2.500?5de-5",
        "[0x1.a2976f1cee4d5p-16, 0x1.a36e2eb1c432dp-16]",
    );
    chk("10?3e380", "[0x1.fffffffffffffp+1023, infinity]");
    chk("1.0000000000000001?1", "[1.0, 0x1.0000000000001p+0]");
}

/// The 310-digit radius saturates to the whole line (class file).
#[test]
fn huge_radius_is_entire() {
    let s = "10?1800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    assert!(p(s).is_entire());
}

/// A written lower bound above the written upper bound still forms a valid
/// interval after directed rounding (`PossiblyUndefinedOperation` -> Ok).
#[test]
fn possibly_undefined_is_ok() {
    // exceptions.itl and class.itl.
    chk(
        "[1.0000000000000001, 1.0000000000000002]",
        "[1.0, 0x1.0000000000001p+0]",
    );
    chk(
        "[1.0000000000000002,1.0000000000000001]",
        "[1.0, 0x1.0000000000001p+0]",
    );
    chk(
        "[10000000000000001/10000000000000000,10000000000000002/10000000000000001]",
        "[1.0, 0x1.0000000000001p+0]",
    );
    chk(
        "[0x1.00000000000002p0,0x1.00000000000001p0]",
        "[1.0, 0x1.0000000000001p+0]",
    );
}

// --- malformed bare literals (signal UndefinedOperation -> Err) ------------

#[test]
fn bare_rejects_decoration_and_nai() {
    // A bare textToInterval admits no decoration suffix.
    for s in [
        "[ Empty  ]_trv",
        "[  ]_trv",
        "[,]_trv",
        "[ ENTIRE ]_dac",
        "[ -inf, INF ]_def",
    ] {
        bad(s);
    }
    // Nor the [nai] value.
    for s in ["[ Nai  ]", "[ Nai  ]_ill", "[ Nai  ]_trv"] {
        bad(s);
    }
}

#[test]
fn malformed_bare() {
    for s in [
        "[ Empty  ]_ill",
        "[  ]_com",
        "[,]_com",
        "[   Entire ]_com",
        "[ -inf ,  INF ]_com",
        "[  -1.0  , 1.0]_ill",
        "[  -1.0  , 1.0]_fooo",
        "[  -1.0  , 1.0]_da",
        "[-1.0,]_com",
        "[-Inf, 1.000 ]_ill",
        "[-I  nf, 1.000 ]", // embedded whitespace in a token
        "[-Inf, 1.0  00 ]", // embedded whitespace in a number
        "[-Inf ]",          // -inf cannot be a point
        "[Inf , INF]",      // +inf lower endpoint
        "[ foo ]",          // unknown keyword
        "[+infinity]",      // exceptions.itl: +inf point
        "[1.0,2.0",         // unclosed
        "",                 // empty input
        "[1.0 2.0]",        // missing comma
        "[1,2,3]",          // too many commas
        "1.0",              // bare number without brackets or '?'
        "0x1.0/2",          // hex numerator in a rational
        "1/0",              // zero denominator
    ] {
        bad(s);
    }
}

// --- decorated literals (class minimal_text_to_decorated + constructor) ----

#[test]
fn decorated_valid() {
    // No suffix: strongest consistent decoration.
    assert_eq!(pd("[ Empty  ]").decoration(), Decoration::Trv);
    assert_eq!(pd("[,]").decoration(), Decoration::Dac);
    assert!(pd("[,]").interval().is_entire());
    assert_eq!(pd("[ entire  ]").decoration(), Decoration::Dac);
    assert_eq!(pd("[-1.0,1.0]").decoration(), Decoration::Com);
    assert_eq!(pd("[-1,]").decoration(), Decoration::Dac);
    assert_eq!(pd("0.0?").decoration(), Decoration::Com);

    // Explicit suffix combined with the stored interval's strongest decoration.
    chk_d("[  -1.0  ,  1.0  ]_com", "[-1.0, 1.0]_com");
    chk_d("[  -1.0  , 1.0]_trv", "[-1.0, 1.0]_trv");
    chk_d("[-1.0, +inf]_def", "[-1.0, infinity]_def");
    chk_d("[ ENTIRE ]_dac", "[entire]_dac");
    chk_d("[ -inf, INF ]_def", "[entire]_def");
    chk_d("[-Infinity, 1.000 ]_trv", "[-infinity, 1.0]_trv");
    chk_d("0.0?u_trv", "[0.0, 0.05]_trv");
    chk_d("0.000?5u_def", "[0.0, 0.005]_def");
    chk_d("2.500?5ue4_def", "[0x1.86ap+14, 0x1.8768p+14]_def");
    chk_d(
        "3.56?1_def",
        "[0X3.8CCCCCCCCCCCCP+0, 0X3.91EB851EB8520P+0]_def",
    );

    // Overflow demotes an explicit com to dac (math bounded, stored unbounded).
    assert_eq!(pd("[1.0E+400 ]_com").decoration(), Decoration::Dac);
    assert!(!pd("[1.0E+400 ]_com").interval().is_bounded());
    assert_eq!(pd("10?3e380_com").decoration(), Decoration::Dac);

    // [nai] is a value.
    assert!(pd("[ Nai  ]").is_nai());
    assert!(pd("[1,1e3]_com").decoration() == Decoration::Com);
    assert!(pd("[1,1E3]_COM").decoration() == Decoration::Com); // case folding
}

#[test]
fn malformed_decorated() {
    for s in [
        "[ Nai  ]_ill",
        "[ Nai  ]_trv",
        "[ Empty  ]_ill", // _ill is reserved for [nai]
        "[  ]_com",       // com on empty
        "[,]_com",        // com on entire
        "[   Entire ]_com",
        "[ -inf ,  INF ]_com",
        "[  -1.0  , 1.0]_ill",
        "[  -1.0  , 1.0]_fooo",
        "[  -1.0  , 1.0]_da",
        "[-1.0,]_com", // com on a mathematically unbounded interval
        "[-Inf, 1.000 ]_ill",
        "0.0??_com", // com on an infinite-radius (unbounded) interval
        "0.0??u_ill",
        "0.0??d_com",
        "[1.0,2.0", // unclosed
        "[ foo ]",
    ] {
        bad_d(s);
    }
}

// --- output: Display, exact round-trip -------------------------------------

#[test]
fn display_special_forms() {
    assert_eq!(Interval::<f64>::empty().to_string(), "[empty]");
    assert_eq!(Interval::<f64>::entire().to_string(), "[entire]");
    assert_eq!(Interval::new(1.0, 2.0).unwrap().to_string(), "[1, 2]");
    assert_eq!(
        Interval::new(-1.0, f64::INFINITY).unwrap().to_string(),
        "[-1, inf]"
    );
    assert_eq!(DecoratedInterval::<f64>::nai().to_string(), "[nai]");
    assert_eq!(
        DecoratedInterval::new_dec(Interval::new(1.0, 2.0).unwrap()).to_string(),
        "[1, 2]_com"
    );
}

#[test]
fn exact_round_trip_special() {
    for x in [
        Interval::<f64>::empty(),
        Interval::entire(),
        Interval::new(1.0, 2.0).unwrap(),
        Interval::new(-0.1, 0.1).unwrap(),
        Interval::new(f64::MIN_POSITIVE, f64::MAX).unwrap(),
        Interval::new(-f64::MAX, -5e-324).unwrap(), // negative subnormal upper
        Interval::new(0.0, f64::INFINITY).unwrap(),
    ] {
        let text = x.interval_to_exact().to_string();
        let back = Interval::<f64>::exact_to_interval(&text)
            .unwrap_or_else(|e| panic!("exact reparse {text:?}: {e:?}"));
        assert_eq!(back, x, "exact round trip via {text:?}");
    }
}

// --- property lanes --------------------------------------------------------

/// A random finite interval `[lo, hi]`.
fn finite_iv() -> impl Strategy<Value = Interval<f64>> {
    (-1.0e12..1.0e12, -1.0e12..1.0e12).prop_map(|(a, b)| {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        Interval::new(lo, hi).unwrap()
    })
}

/// A random interval spanning empty, unbounded, and subnormal magnitudes.
fn any_iv() -> impl Strategy<Value = Interval<f64>> {
    prop_oneof![
        finite_iv(),
        Just(Interval::<f64>::empty()),
        Just(Interval::entire()),
        (-1.0e300..1.0e300).prop_map(|a| Interval::new(a, f64::INFINITY).unwrap()),
        (-1.0e-300..1.0e-300, -1.0e-300..1.0e-300).prop_map(|(a, b)| {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            Interval::new(lo, hi).unwrap()
        }),
    ]
}

/// A random well-formed decimal literal string.
fn decimal_str() -> impl Strategy<Value = String> {
    (any::<bool>(), "[0-9]{1,17}", "[0-9]{0,17}", -30i32..30).prop_map(|(neg, int, frac, exp)| {
        let mut s = String::new();
        if neg {
            s.push('-');
        }
        s.push_str(&int);
        if !frac.is_empty() {
            s.push('.');
            s.push_str(&frac);
        }
        s.push('e');
        s.push_str(&exp.to_string());
        s
    })
}

/// A decimal literal whose exponent reaches the subnormal and overflow edges.
fn decimal_str_wide() -> impl Strategy<Value = String> {
    (any::<bool>(), "[0-9]{1,17}", "[0-9]{0,17}", -330i32..330).prop_map(|(neg, int, frac, exp)| {
        let mut s = String::new();
        if neg {
            s.push('-');
        }
        s.push_str(&int);
        if !frac.is_empty() {
            s.push('.');
            s.push_str(&frac);
        }
        s.push('e');
        s.push_str(&exp.to_string());
        s
    })
}

proptest! {
    /// The directed core stays sound and one-ulp-tight against `str::parse` even
    /// at the subnormal and overflow boundaries (`str::parse` is correctly
    /// rounded there too: it yields subnormals, `+-0`, and infinities).
    #[test]
    fn decimal_brackets_oracle_at_edges(s in decimal_str_wide()) {
        let nearest: f64 = s.parse().expect("std parse");
        let iv = p(&format!("[{s}]"));
        let (lo, hi) = (iv.inf(), iv.sup());
        prop_assert!(lo <= nearest && nearest <= hi, "[{lo}, {hi}] misses {nearest} from {s}");
        // Equal (exact), or adjacent floats. `next_up` carries MAX -> +inf and
        // 0 -> the smallest subnormal, so the overflow and underflow brackets
        // satisfy this too.
        prop_assert!(hi == lo || hi == lo.next_up(), "[{lo}, {hi}] not one ulp from {s}");
    }

    /// Exact hex output reread reproduces the interval bit for bit.
    #[test]
    fn exact_round_trip(x in any_iv()) {
        let text = x.interval_to_exact().to_string();
        let back = Interval::<f64>::exact_to_interval(&text).expect("exact reparse");
        prop_assert_eq!(back, x);
    }

    /// Display then reread encloses the original (outward rounding never narrows).
    #[test]
    fn display_reread_encloses(x in finite_iv()) {
        let text = x.to_string();
        let back = Interval::<f64>::text_to_interval(&text).expect("reread");
        prop_assert!(x.subset(back), "{:?} not subset of reread {:?} via {}", x, back, text);
    }

    /// The directed decimal core brackets `str::parse`'s correctly-rounded
    /// nearest value, and the two endpoints are equal or one ulp apart.
    #[test]
    fn decimal_brackets_str_parse_oracle(s in decimal_str()) {
        let nearest: f64 = s.parse().expect("std parse");
        prop_assume!(nearest.is_finite());
        let iv = p(&format!("[{s}]"));
        let (lo, hi) = (iv.inf(), iv.sup());
        prop_assert!(lo <= nearest && nearest <= hi, "[{lo}, {hi}] misses {nearest} from {s}");
        // Tightest: lo and hi are adjacent floats or equal.
        prop_assert!(hi == lo || hi == lo.next_up(), "[{lo}, {hi}] not one ulp wide from {s}");
    }

    /// No input panics: every string yields Ok or Err, across all four entries.
    #[test]
    fn no_input_panics(s in "\\PC{0,40}") {
        let _ = Interval::<f64>::text_to_interval(&s);
        let _ = DecoratedInterval::<f64>::text_to_interval(&s);
        let _ = Interval::<f64>::exact_to_interval(&s);
    }

    /// Random ASCII bytes never panic either (the adversarial byte lane).
    #[test]
    fn no_ascii_byte_panics(bytes in proptest::collection::vec(0u8..=127, 0..48)) {
        let s = core::str::from_utf8(&bytes).unwrap();
        let _ = Interval::<f64>::text_to_interval(s);
        let _ = DecoratedInterval::<f64>::text_to_interval(s);
    }
}

// --- directed-conversion spot checks at the range edges --------------------

#[test]
fn directed_spot_checks() {
    // 0.1 is not representable: [round_down(0.1), round_up(0.1)] brackets it.
    let one_tenth = p("[0.1]");
    assert!(one_tenth.inf() <= 0.1 && 0.1 <= one_tenth.sup());
    assert!(one_tenth.sup() == one_tenth.inf() || one_tenth.sup() == one_tenth.inf().next_up());
    // The f64 literal 0.1 is the nearest; the bracket contains it.
    assert!(one_tenth.contains(0.1_f64));

    // 1.1 likewise.
    let onepointone = p("[1.1]");
    assert!(onepointone.contains(1.1_f64));
    assert!(onepointone.sup() <= onepointone.inf().next_up());

    // Overflow: a point above the finite range rounds to [MAX, +inf].
    let over = p("[1e400]");
    assert_eq!(over.inf(), f64::MAX);
    assert_eq!(over.sup(), f64::INFINITY);
    let neg_over = p("[-1e400]");
    assert_eq!(neg_over.inf(), f64::NEG_INFINITY);
    assert_eq!(neg_over.sup(), -f64::MAX);

    // Underflow: a tiny positive point rounds to [+0, smallest subnormal].
    let under = p("[1e-400]");
    assert_eq!(under.inf(), 0.0);
    assert_eq!(under.sup(), 5e-324);
    let neg_under = p("[-1e-400]");
    assert_eq!(neg_under.inf(), -5e-324);
    assert_eq!(neg_under.sup(), 0.0);

    // Exactly zero stays a point at zero.
    assert_eq!(p("[0.0]"), Interval::new(0.0, 0.0).unwrap());

    // The smallest subnormal, named exactly in hex, round-trips.
    assert_eq!(p("[0x0.0000000000001p-1022]").inf(), 5e-324);
}

// --- adversarial length caps -----------------------------------------------

#[test]
fn length_caps_reject_not_panic() {
    // A digit group past MAX_SIG_DIGITS.
    let long = format!("[{}]", "1".repeat(MAX_SIG_DIGITS + 1));
    assert_eq!(
        Interval::<f64>::text_to_interval(&long),
        Err(TextError::TooLong)
    );

    // Input past MAX_INPUT_LEN.
    let huge = format!("[1e{}]", "0".repeat(MAX_INPUT_LEN));
    assert_eq!(
        Interval::<f64>::text_to_interval(&huge),
        Err(TextError::TooLong)
    );

    // An exponent field past MAX_EXP_DIGITS.
    let long_exp = format!("1e{}", "1".repeat(MAX_EXP_DIGITS + 1));
    assert_eq!(
        Interval::<f64>::text_to_interval(&format!("[{long_exp}]")),
        Err(TextError::TooLong)
    );

    // Deeply nonsensical strings error rather than panic.
    for s in [
        "[[[[", "?????", "0x", "1e", "[1/./2]", "[,,]", "e10", "[0x.p]",
    ] {
        assert!(
            Interval::<f64>::text_to_interval(s).is_err(),
            "{s} should error"
        );
    }
}
