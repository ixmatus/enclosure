//! The ITF1788 arithmetic reverse-operation vectors over `TightF64`
//! (bit-exact), hardening decision record 0006 part 2's promise that the
//! arithmetic reverses are tightest over a correctly-rounded backend.
//!
//! Every assertion line of every ARITHMETIC reverse testcase from the vendored
//! ITL corpus is transcribed here, no sampling:
//!
//! - `libieeep1788_rev.itl` (Apache-2.0; original author Marco Nehmeier, ITL
//!   port Oliver Heimlich): the `sqrRev`, `absRev`, `pownRev`, `mulRev`, and
//!   ternary `mulRevTen` families, unary/bin/ten forms, bare and decorated.
//! - `libieeep1788_mul_rev.itl` (Apache-2.0): the full `mulRevToPair` grid,
//!   bare and decorated.
//! - `abs_rev.itl` (permissive, copy-and-distribute): the `absRevBin` grid.
//!
//! See `docs/references/vendor/itf1788-framework/`.
//!
//! # Exclusions (feed the conformance document's split)
//!
//! The `sinRev`, `cosRev`, `tanRev`, and `coshRev` testcases are EXCLUDED. They
//! need `RoundInverseTrig` (`asin`/`acos`/`atan` plus the pi enclosure) and
//! `RoundInverseHyperbolic` (`acosh`), which `TightF64` does not implement, so
//! those reverses are not even callable over this backend. They stay with the
//! `f64` fixture's enclosure lanes (`reverse_rev_fixture.rs`); the conformance
//! document records the split.
//!
//! # Assertion mode
//!
//! Bit-exact endpoints over `TightF64`. [`same`] compares two `f64` as equal
//! iff their bit patterns agree, with two carve-outs: `NaN == NaN`, and
//! `+0.0 == -0.0`. The signed-zero carve-out is load-bearing and its rule is
//! this: every zero the corpus writes in a RESULT position is `+0.0` (the
//! corpus never pins a `-0.0` output), and the standard does not fix the sign
//! of a zero interval endpoint (`[+0,+0]`, `[-0,-0]`, `[-0,+0]` all denote the
//! singleton set `{0}`), so a result endpoint of `-0.0` where the corpus wrote
//! `0.0` is a faithful match, not a divergence. Input `-0.0` literals, by
//! contrast, ARE transcribed verbatim: the corpus distinguishes rows like
//! `pownRev [-0.0,-0.0]` from `pownRev [0.0,0.0]`, and feeding the signed zero
//! keeps the row faithful even where the operation collapses the distinction.
//!
//! # Decorations
//!
//! Decision record 0006 part 5: every one-output reverse decorates `trv`, with
//! `NaI` propagating. The two-output reverse division is the standard's own
//! exception (clause 12.12.3): its first piece carries the normal division's
//! decoration whenever `0` is outside the divisor `b`, and its empty second
//! piece is `trv`. The decorated testcases assert the output interval AND the
//! output decoration exactly.
//!
//! The single-output `mulRev`, `sqrRev`, `absRev`, and `pownRev` decorated
//! corpora all pin `trv` outputs and pass; `mulRevToPair` propagates on its
//! first piece and is asserted exactly against the corpus in
//! `mul_rev_to_pair_dec_test`. See the decision record's erratum and the
//! conformance document (enc-su3).

// The corpus carries long hex-fraction literals inside strings (parsed by `hx`)
// and a handful of multi-digit decimal literals; neither trips a lint here, but
// the allows mirror the fixture idiom and keep transcription verbatim.
#![allow(clippy::approx_constant, clippy::unreadable_literal)]
// The vector grids are transcribed one array row per corpus line; the tables
// are long by design (completeness, no sampling).
#![allow(clippy::too_many_lines)]

use interval_1788::Decoration::{Com, Dac, Def, Trv};
use interval_1788::{DecoratedInterval, Decoration, Interval};
use round_float::TightF64;

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

/// A vector interval: `None` is `[empty]`, `Some((lo, hi))` is `[lo, hi]`, and
/// [`ENT`] names the entire interval.
type E = Option<(f64, f64)>;
const MT: E = None;
const ENT: E = Some((NEG_INF, INF));

fn build(v: E) -> Interval<TightF64> {
    match v {
        None => Interval::empty(),
        Some((lo, hi)) => Interval::new(TightF64(lo), TightF64(hi)).unwrap(),
    }
}

fn entire() -> Interval<TightF64> {
    Interval::entire()
}

/// `Some((hx(a), hx(b)))`: a vector interval whose endpoints are both C
/// hex-float literals. The `Some` is the [`E`] vector type shared with the
/// tables (empty is `None`), never a "could be empty" here.
#[allow(clippy::unnecessary_wraps)]
fn hh(a: &str, b: &str) -> E {
    Some((hx(a), hx(b)))
}

/// Parse a C hex-float literal to the exact `f64` it names (the shared fixture
/// idiom; see `reverse_rev_fixture.rs`), with the canonicality assertion that
/// turns a non-representable literal into a loud failure.
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
    // A canonical corpus literal (leading hex digit 1, at most thirteen fraction
    // digits) carries at most 53 significant bits, so the cast is exact.
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

/// Bit-exact `f64` equality with the two corpus-faithful carve-outs
/// (`NaN == NaN`, `+0.0 == -0.0`); see the module docs.
fn same(x: f64, y: f64) -> bool {
    (x.is_nan() && y.is_nan()) || x.to_bits() == y.to_bits() || (x == 0.0 && y == 0.0)
}

/// Assert `got` is bit-exactly the vector interval `want` (`None` = empty).
#[track_caller]
fn eq(got: Interval<TightF64>, want: E) {
    match want {
        None => assert!(
            got.is_empty(),
            "expected [empty], got [{}, {}]",
            got.inf().0,
            got.sup().0
        ),
        Some((lo, hi)) => assert!(
            !got.is_empty() && same(got.inf().0, lo) && same(got.sup().0, hi),
            "expected [{lo}, {hi}], got [{}, {}]",
            got.inf().0,
            got.sup().0
        ),
    }
}

/// Assert a decorated reverse result: output decoration is exactly `trv`
/// (decision record 0006 part 5) and the interval is bit-exact.
#[track_caller]
fn eq_trv(got: DecoratedInterval<TightF64>, want: E) {
    assert!(!got.is_nai(), "unexpected NaI");
    assert_eq!(got.decoration(), Trv, "expected trv decoration");
    eq(got.interval(), want);
}

fn dec(v: E, d: Decoration) -> DecoratedInterval<TightF64> {
    DecoratedInterval::set_dec(build(v), d)
}

fn new_dec(v: E) -> DecoratedInterval<TightF64> {
    DecoratedInterval::new_dec(build(v))
}

fn nai() -> DecoratedInterval<TightF64> {
    DecoratedInterval::nai()
}

/// A decorated `mulRevToPair` operand or output piece: `Nai`, or an interval
/// `v` decorated `d`. Used only by the KNOWN GAP lane.
#[derive(Clone, Copy)]
enum DI {
    Nai,
    D(E, Decoration),
}

fn dib(x: DI) -> DecoratedInterval<TightF64> {
    match x {
        DI::Nai => nai(),
        DI::D(v, d) => dec(v, d),
    }
}

/// Empty piece decorated `trv`, the corpus's second slot on every non-split row.
fn et() -> DI {
    DI::D(MT, Trv)
}

fn di(v: E, d: Decoration) -> DI {
    DI::D(v, d)
}

/// Assert a decorated `mulRevToPair` output piece bit-exactly (interval AND
/// decoration). The first piece carries the normal division's decoration when
/// zero is outside the divisor (clause 12.12.3), so the crate matches the
/// corpus's propagated decoration here.
#[track_caller]
fn eq_dpiece(got: DecoratedInterval<TightF64>, want: DI) {
    match want {
        DI::Nai => assert!(got.is_nai(), "expected NaI, got a decorated interval"),
        DI::D(v, d) => {
            assert!(!got.is_nai(), "unexpected NaI");
            assert_eq!(got.decoration(), d, "decoration mismatch");
            eq(got.interval(), v);
        }
    }
}

// ============================================================================
// sqrRev  (libieeep1788_rev.itl)
// ============================================================================

// minimal_sqr_rev_test: 10 vectors
#[test]
fn sqr_rev_test() {
    let v = [
        (MT, MT),
        (Some((-10.0, -1.0)), MT),
        (Some((0.0, INF)), ENT),
        (Some((0.0, 1.0)), Some((-1.0, 1.0))),
        (Some((-0.5, 1.0)), Some((-1.0, 1.0))),
        (Some((-1000.0, 1.0)), Some((-1.0, 1.0))),
        (Some((0.0, 25.0)), Some((-5.0, 5.0))),
        (Some((-1.0, 25.0)), Some((-5.0, 5.0))),
        (
            hh("0X1.47AE147AE147BP-7", "0X1.47AE147AE147CP-7"),
            hh("-0X1.999999999999BP-4", "0X1.999999999999BP-4"),
        ),
        (
            Some((0.0, hx("0X1.FFFFFFFFFFFE1P+1"))),
            hh("-0x1.ffffffffffff1p+0", "0x1.ffffffffffff1p+0"),
        ),
    ];
    for (c, w) in v {
        eq(build(c).sqr_rev(entire()), w);
    }
}

// minimal_sqr_rev_bin_test: 11 vectors
#[test]
fn sqr_rev_bin_test() {
    let v = [
        (MT, Some((-5.0, 1.0)), MT),
        (Some((-10.0, -1.0)), Some((-5.0, 1.0)), MT),
        (Some((0.0, INF)), Some((-5.0, 1.0)), Some((-5.0, 1.0))),
        (Some((0.0, 1.0)), Some((-0.1, 1.0)), Some((-0.1, 1.0))),
        (Some((-0.5, 1.0)), Some((-0.1, 1.0)), Some((-0.1, 1.0))),
        (Some((-1000.0, 1.0)), Some((-0.1, 1.0)), Some((-0.1, 1.0))),
        (Some((0.0, 25.0)), Some((-4.1, 6.0)), Some((-4.1, 5.0))),
        (Some((-1.0, 25.0)), Some((-4.1, 7.0)), Some((-4.1, 5.0))),
        (Some((1.0, 25.0)), Some((0.0, 7.0)), Some((1.0, 5.0))),
        (
            hh("0X1.47AE147AE147BP-7", "0X1.47AE147AE147CP-7"),
            Some((-0.1, INF)),
            Some((-0.1, hx("0X1.999999999999BP-4"))),
        ),
        (
            Some((0.0, hx("0X1.FFFFFFFFFFFE1P+1"))),
            Some((-0.1, INF)),
            Some((-0.1, hx("0x1.ffffffffffff1p+0"))),
        ),
    ];
    for (c, x, w) in v {
        eq(build(c).sqr_rev(build(x)), w);
    }
}

// minimal_sqr_rev_dec_test: 10 vectors
#[test]
fn sqr_rev_dec_test() {
    let v = [
        (Trv, MT, MT),
        (Com, Some((-10.0, -1.0)), MT),
        (Dac, Some((0.0, INF)), ENT),
        (Def, Some((0.0, 1.0)), Some((-1.0, 1.0))),
        (Dac, Some((-0.5, 1.0)), Some((-1.0, 1.0))),
        (Com, Some((-1000.0, 1.0)), Some((-1.0, 1.0))),
        (Def, Some((0.0, 25.0)), Some((-5.0, 5.0))),
        (Dac, Some((-1.0, 25.0)), Some((-5.0, 5.0))),
        (
            Com,
            hh("0X1.47AE147AE147BP-7", "0X1.47AE147AE147CP-7"),
            hh("-0X1.999999999999BP-4", "0X1.999999999999BP-4"),
        ),
        (
            Def,
            Some((0.0, hx("0X1.FFFFFFFFFFFE1P+1"))),
            hh("-0x1.ffffffffffff1p+0", "0x1.ffffffffffff1p+0"),
        ),
    ];
    for (cd, c, w) in v {
        eq_trv(dec(c, cd).sqr_rev(new_dec(ENT)), w);
    }
}

// minimal_sqr_rev_dec_bin_test: 11 vectors
#[test]
fn sqr_rev_dec_bin_test() {
    let v = [
        (Trv, MT, Def, Some((-5.0, 1.0)), MT),
        (Com, Some((-10.0, -1.0)), Dac, Some((-5.0, 1.0)), MT),
        (
            Def,
            Some((0.0, INF)),
            Dac,
            Some((-5.0, 1.0)),
            Some((-5.0, 1.0)),
        ),
        (
            Dac,
            Some((0.0, 1.0)),
            Def,
            Some((-0.1, 1.0)),
            Some((-0.1, 1.0)),
        ),
        (
            Def,
            Some((-0.5, 1.0)),
            Dac,
            Some((-0.1, 1.0)),
            Some((-0.1, 1.0)),
        ),
        (
            Com,
            Some((-1000.0, 1.0)),
            Def,
            Some((-0.1, 1.0)),
            Some((-0.1, 1.0)),
        ),
        (
            Def,
            Some((0.0, 25.0)),
            Com,
            Some((-4.1, 6.0)),
            Some((-4.1, 5.0)),
        ),
        (
            Dac,
            Some((-1.0, 25.0)),
            Def,
            Some((-4.1, 7.0)),
            Some((-4.1, 5.0)),
        ),
        (
            Dac,
            Some((1.0, 25.0)),
            Def,
            Some((0.0, 7.0)),
            Some((1.0, 5.0)),
        ),
        (
            Def,
            hh("0X1.47AE147AE147BP-7", "0X1.47AE147AE147CP-7"),
            Dac,
            Some((-0.1, INF)),
            Some((-0.1, hx("0X1.999999999999BP-4"))),
        ),
        (
            Dac,
            Some((0.0, hx("0X1.FFFFFFFFFFFE1P+1"))),
            Dac,
            Some((-0.1, INF)),
            Some((-0.1, hx("0x1.ffffffffffff1p+0"))),
        ),
    ];
    for (cd, c, xd, x, w) in v {
        eq_trv(dec(c, cd).sqr_rev(dec(x, xd)), w);
    }
}

// ============================================================================
// absRev  (libieeep1788_rev.itl)
// ============================================================================

// minimal_abs_rev_test: 9 vectors
#[test]
fn abs_rev_test() {
    let v = [
        (MT, MT),
        (Some((-1.1, -0.4)), MT),
        (Some((0.0, INF)), ENT),
        (Some((1.1, 2.1)), Some((-2.1, 2.1))),
        (Some((-1.1, 2.0)), Some((-2.0, 2.0))),
        (Some((-1.1, 0.0)), Some((0.0, 0.0))),
        (Some((-1.9, 0.2)), Some((-0.2, 0.2))),
        (Some((0.0, 0.2)), Some((-0.2, 0.2))),
        (Some((-1.5, INF)), ENT),
    ];
    for (c, w) in v {
        eq(build(c).abs_rev(entire()), w);
    }
}

// minimal_abs_rev_bin_test: 7 vectors
#[test]
fn abs_rev_bin_test() {
    let v = [
        (MT, Some((-1.1, 5.0)), MT),
        (Some((-1.1, -0.4)), Some((-1.1, 5.0)), MT),
        (Some((0.0, INF)), Some((-1.1, 5.0)), Some((-1.1, 5.0))),
        (Some((1.1, 2.1)), Some((-1.0, 5.0)), Some((1.1, 2.1))),
        (Some((-1.1, 2.0)), Some((-1.1, 5.0)), Some((-1.1, 2.0))),
        (Some((-1.1, 0.0)), Some((-1.1, 5.0)), Some((0.0, 0.0))),
        (Some((-1.9, 0.2)), Some((-1.1, 5.0)), Some((-0.2, 0.2))),
    ];
    for (c, x, w) in v {
        eq(build(c).abs_rev(build(x)), w);
    }
}

// minimal_abs_rev_dec_test: 9 vectors
#[test]
fn abs_rev_dec_test() {
    let v = [
        (Trv, MT, MT),
        (Dac, Some((-1.1, -0.4)), MT),
        (Dac, Some((0.0, INF)), ENT),
        (Com, Some((1.1, 2.1)), Some((-2.1, 2.1))),
        (Def, Some((-1.1, 2.0)), Some((-2.0, 2.0))),
        (Dac, Some((-1.1, 0.0)), Some((0.0, 0.0))),
        (Com, Some((-1.9, 0.2)), Some((-0.2, 0.2))),
        (Def, Some((0.0, 0.2)), Some((-0.2, 0.2))),
        (Def, Some((-1.5, INF)), ENT),
    ];
    for (cd, c, w) in v {
        eq_trv(dec(c, cd).abs_rev(new_dec(ENT)), w);
    }
}

// minimal_abs_rev_dec_bin_test: 7 vectors
#[test]
fn abs_rev_dec_bin_test() {
    let v = [
        (Trv, MT, Com, Some((-1.1, 5.0)), MT),
        (Dac, Some((-1.1, -0.4)), Dac, Some((-1.1, 5.0)), MT),
        (
            Def,
            Some((0.0, INF)),
            Def,
            Some((-1.1, 5.0)),
            Some((-1.1, 5.0)),
        ),
        (
            Dac,
            Some((1.1, 2.1)),
            Def,
            Some((-1.0, 5.0)),
            Some((1.1, 2.1)),
        ),
        (
            Com,
            Some((-1.1, 2.0)),
            Def,
            Some((-1.1, 5.0)),
            Some((-1.1, 2.0)),
        ),
        (
            Def,
            Some((-1.1, 0.0)),
            Def,
            Some((-1.1, 5.0)),
            Some((0.0, 0.0)),
        ),
        (
            Dac,
            Some((-1.9, 0.2)),
            Def,
            Some((-1.1, 5.0)),
            Some((-0.2, 0.2)),
        ),
    ];
    for (cd, c, xd, x, w) in v {
        eq_trv(dec(c, cd).abs_rev(dec(x, xd)), w);
    }
}

// ============================================================================
// absRevBin  (abs_rev.itl)
// ============================================================================

// minimal.absRevBin_test: 24 vectors
#[test]
fn abs_rev_bin_absrevitl_test() {
    let max = hx("0x1.FFFFFFFFFFFFFp1023");
    let tiny = hx("0x1p-1022");
    let v = [
        (MT, ENT, MT),
        (Some((0.0, 1.0)), MT, MT),
        (Some((0.0, 1.0)), Some((7.0, 9.0)), MT),
        (MT, Some((0.0, 1.0)), MT),
        (Some((-2.0, -1.0)), ENT, MT),
        (Some((1.0, 1.0)), ENT, Some((-1.0, 1.0))),
        (Some((0.0, 0.0)), ENT, Some((0.0, 0.0))),
        (Some((-1.0, -1.0)), ENT, MT),
        (Some((max, max)), ENT, Some((-max, max))),
        (Some((tiny, tiny)), ENT, Some((-tiny, tiny))),
        (Some((-tiny, -tiny)), ENT, MT),
        (Some((-max, -max)), ENT, MT),
        (Some((1.0, 2.0)), ENT, Some((-2.0, 2.0))),
        (Some((1.0, 2.0)), Some((0.0, 2.0)), Some((1.0, 2.0))),
        (Some((0.0, 1.0)), Some((-0.5, 2.0)), Some((-0.5, 1.0))),
        (Some((-1.0, 1.0)), ENT, Some((-1.0, 1.0))),
        (Some((-1.0, 0.0)), ENT, Some((0.0, 0.0))),
        (Some((0.0, INF)), ENT, ENT),
        (ENT, ENT, ENT),
        (Some((NEG_INF, 0.0)), ENT, Some((0.0, 0.0))),
        (
            Some((1.0, INF)),
            Some((NEG_INF, 0.0)),
            Some((NEG_INF, -1.0)),
        ),
        (Some((-1.0, INF)), ENT, ENT),
        (Some((NEG_INF, -1.0)), ENT, MT),
        (Some((NEG_INF, 1.0)), ENT, Some((-1.0, 1.0))),
    ];
    for (c, x, w) in v {
        eq(build(c).abs_rev(build(x)), w);
    }
}

// ============================================================================
// pownRev  (libieeep1788_rev.itl)
// ============================================================================

/// The `minimal_pown_rev_test` corpus (143 vectors), asserted bit-exact by the
/// running lane [`pown_rev_test`].
fn pown_bare_cases() -> Vec<(E, i32, E)> {
    vec![
        // n = 0
        (MT, 0, MT),
        (Some((1.0, 1.0)), 0, ENT),
        (Some((-1.0, 5.0)), 0, ENT),
        (Some((-1.0, 0.0)), 0, MT),
        (Some((-1.0, -0.0)), 0, MT),
        (Some((1.1, 10.0)), 0, MT),
        // n = 1
        (MT, 1, MT),
        (ENT, 1, ENT),
        (Some((0.0, 0.0)), 1, Some((0.0, 0.0))),
        (Some((-0.0, -0.0)), 1, Some((0.0, 0.0))),
        (Some((13.1, 13.1)), 1, Some((13.1, 13.1))),
        (
            Some((-7451.145, -7451.145)),
            1,
            Some((-7451.145, -7451.145)),
        ),
        (
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            1,
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
        ),
        (
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
            1,
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
        ),
        (Some((0.0, INF)), 1, Some((0.0, INF))),
        (Some((-0.0, INF)), 1, Some((0.0, INF))),
        (Some((NEG_INF, 0.0)), 1, Some((NEG_INF, 0.0))),
        (Some((NEG_INF, -0.0)), 1, Some((NEG_INF, 0.0))),
        (Some((-324.3, 2.5)), 1, Some((-324.3, 2.5))),
        (Some((0.01, 2.33)), 1, Some((0.01, 2.33))),
        (Some((-1.9, -0.33)), 1, Some((-1.9, -0.33))),
        // n = 2
        (MT, 2, MT),
        (Some((-5.0, -1.0)), 2, MT),
        (Some((0.0, INF)), 2, ENT),
        (Some((-0.0, INF)), 2, ENT),
        (Some((0.0, 0.0)), 2, Some((0.0, 0.0))),
        (Some((-0.0, -0.0)), 2, Some((0.0, 0.0))),
        (
            hh("0X1.573851EB851EBP+7", "0X1.573851EB851ECP+7"),
            2,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("0X1.A794A4E7CFAADP+25", "0X1.A794A4E7CFAAEP+25"),
            2,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            2,
            hh("-0x1p+512", "0x1p+512"),
        ),
        (
            Some((0.0, hx("0X1.9AD27D70A3D72P+16"))),
            2,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Some((-0.0, hx("0X1.9AD27D70A3D72P+16"))),
            2,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            hh("0X1.A36E2EB1C432CP-14", "0X1.5B7318FC50482P+2"),
            2,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("0X1.BE0DED288CE7P-4", "0X1.CE147AE147AE1P+1"),
            2,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = 8
        (MT, 8, MT),
        (ENT, 8, ENT),
        (Some((0.0, INF)), 8, ENT),
        (Some((-0.0, INF)), 8, ENT),
        (Some((0.0, 0.0)), 8, Some((0.0, 0.0))),
        (Some((-0.0, -0.0)), 8, Some((0.0, 0.0))),
        (
            hh("0X1.9D8FD495853F5P+29", "0X1.9D8FD495853F6P+29"),
            8,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("0X1.DFB1BB622E70DP+102", "0X1.DFB1BB622E70EP+102"),
            8,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            8,
            hh("-0x1p+128", "0x1p+128"),
        ),
        (
            Some((0.0, hx("0X1.A87587109655P+66"))),
            8,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Some((-0.0, hx("0X1.A87587109655P+66"))),
            8,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            hh("0X1.CD2B297D889BDP-54", "0X1.B253D9F33CE4DP+9"),
            8,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("0X1.26F1FCDD502A3P-13", "0X1.53ABD7BFC4FC6P+7"),
            8,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = 3
        (MT, 3, MT),
        (ENT, 3, ENT),
        (Some((0.0, 0.0)), 3, Some((0.0, 0.0))),
        (Some((-0.0, -0.0)), 3, Some((0.0, 0.0))),
        (
            hh("0X1.1902E978D4FDEP+11", "0X1.1902E978D4FDFP+11"),
            3,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("-0X1.81460637B9A3DP+38", "-0X1.81460637B9A3CP+38"),
            3,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            3,
            hh("0x1.428a2f98d728ap+341", "0x1.428a2f98d728bp+341"),
        ),
        (
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
            3,
            hh("-0x1.428a2f98d728bp+341", "-0x1.428a2f98d728ap+341"),
        ),
        (Some((0.0, INF)), 3, Some((0.0, INF))),
        (Some((-0.0, INF)), 3, Some((0.0, INF))),
        (Some((NEG_INF, 0.0)), 3, Some((NEG_INF, 0.0))),
        (Some((NEG_INF, -0.0)), 3, Some((NEG_INF, 0.0))),
        (
            hh("-0X1.0436D2F418938P+25", "0X1.F4P+3"),
            3,
            hh("-0x1.444cccccccccep+8", "0x1.4p+1"),
        ),
        (
            hh("0X1.0C6F7A0B5ED8DP-20", "0X1.94C75E6362A6P+3"),
            3,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("-0X1.B6F9DB22D0E55P+2", "-0X1.266559F6EC5B1P-5"),
            3,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = 7
        (MT, 7, MT),
        (ENT, 7, ENT),
        (Some((0.0, 0.0)), 7, Some((0.0, 0.0))),
        (Some((-0.0, -0.0)), 7, Some((0.0, 0.0))),
        (
            hh("0X1.F91D1B185493BP+25", "0X1.F91D1B185493CP+25"),
            7,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("-0X1.07B1DA32F9B59P+90", "-0X1.07B1DA32F9B58P+90"),
            7,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            7,
            hh("0x1.381147622f886p+146", "0x1.381147622f887p+146"),
        ),
        (
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
            7,
            hh("-0x1.381147622f887p+146", "-0x1.381147622f886p+146"),
        ),
        (Some((0.0, INF)), 7, Some((0.0, INF))),
        (Some((-0.0, INF)), 7, Some((0.0, INF))),
        (Some((NEG_INF, 0.0)), 7, Some((NEG_INF, 0.0))),
        (Some((NEG_INF, -0.0)), 7, Some((NEG_INF, 0.0))),
        (
            hh("-0X1.4F109959E6D7FP+58", "0X1.312DP+9"),
            7,
            hh("-0x1.444cccccccccep+8", "0x1.4p+1"),
        ),
        (
            hh("0X1.6849B86A12B9BP-47", "0X1.74D0373C76313P+8"),
            7,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("-0X1.658C775099757P+6", "-0X1.BEE30301BF47AP-12"),
            7,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = -2
        (MT, -2, MT),
        (Some((0.0, INF)), -2, ENT),
        (Some((-0.0, INF)), -2, ENT),
        (Some((0.0, 0.0)), -2, MT),
        (Some((-0.0, -0.0)), -2, MT),
        (Some((-10.0, 0.0)), -2, MT),
        (Some((-10.0, -0.0)), -2, MT),
        (
            hh("0X1.7DE3A077D1568P-8", "0X1.7DE3A077D1569P-8"),
            -2,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("0X1.3570290CD6E14P-26", "0X1.3570290CD6E15P-26"),
            -2,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (hh("0X0P+0", "0X0.0000000000001P-1022"), -2, ENT),
        (
            Some((hx("0X1.3F0C482C977C9P-17"), INF)),
            -2,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            hh("0X1.793D85EF38E47P-3", "0X1.388P+13"),
            -2,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("0X1.1BA81104F6C8P-2", "0X1.25D8FA1F801E1P+3"),
            -2,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = -8
        (MT, -8, MT),
        (Some((0.0, INF)), -8, ENT),
        (Some((-0.0, INF)), -8, ENT),
        (Some((0.0, 0.0)), -8, MT),
        (Some((-0.0, -0.0)), -8, MT),
        (
            hh("0X1.3CEF39247CA6DP-30", "0X1.3CEF39247CA6EP-30"),
            -8,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("0X1.113D9EF0A99ACP-103", "0X1.113D9EF0A99ADP-103"),
            -8,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (hh("0X0P+0", "0X0.0000000000001P-1022"), -8, ENT),
        (
            Some((hx("0X1.34CC3764D1E0CP-67"), INF)),
            -8,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            hh("0X1.2DC80DB11AB7CP-10", "0X1.1C37937E08P+53"),
            -8,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("0X1.81E104E61630DP-8", "0X1.BC64F21560E34P+12"),
            -8,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = -1
        (MT, -1, MT),
        (ENT, -1, ENT),
        (Some((0.0, 0.0)), -1, MT),
        (Some((-0.0, -0.0)), -1, MT),
        (
            hh("0X1.38ABF82EE6986P-4", "0X1.38ABF82EE6987P-4"),
            -1,
            hh("0x1.a333333333332p+3", "0x1.a333333333335p+3"),
        ),
        (
            hh("-0X1.197422C9048BFP-13", "-0X1.197422C9048BEP-13"),
            -1,
            hh("-0x1.d1b251eb851eep+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            hh("0X0.4P-1022", "0X0.4000000000001P-1022"),
            -1,
            Some((hx("0x1.ffffffffffff8p+1023"), INF)),
        ),
        (
            hh("-0X0.4000000000001P-1022", "-0X0.4P-1022"),
            -1,
            Some((NEG_INF, hx("-0x1.ffffffffffff8p+1023"))),
        ),
        (Some((0.0, INF)), -1, Some((0.0, INF))),
        (Some((-0.0, INF)), -1, Some((0.0, INF))),
        (Some((NEG_INF, 0.0)), -1, Some((NEG_INF, 0.0))),
        (Some((NEG_INF, -0.0)), -1, Some((NEG_INF, 0.0))),
        (
            hh("0X1.B77C278DBBE13P-2", "0X1.9P+6"),
            -1,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("-0X1.83E0F83E0F83EP+1", "-0X1.0D79435E50D79P-1"),
            -1,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = -3
        (MT, -3, MT),
        (ENT, -3, ENT),
        (Some((0.0, 0.0)), -3, MT),
        (Some((-0.0, -0.0)), -3, MT),
        (
            hh("0X1.D26DF4D8B1831P-12", "0X1.D26DF4D8B1832P-12"),
            -3,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("-0X1.54347DED91B19P-39", "-0X1.54347DED91B18P-39"),
            -3,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            hh("0X0P+0", "0X0.0000000000001P-1022"),
            -3,
            Some((hx("0x1p+358"), INF)),
        ),
        (
            hh("-0X0.0000000000001P-1022", "-0X0P+0"),
            -3,
            Some((NEG_INF, hx("-0x1p+358"))),
        ),
        (Some((0.0, INF)), -3, Some((0.0, INF))),
        (Some((-0.0, INF)), -3, Some((0.0, INF))),
        (Some((NEG_INF, 0.0)), -3, Some((NEG_INF, 0.0))),
        (Some((NEG_INF, -0.0)), -3, Some((NEG_INF, 0.0))),
        (
            hh("0X1.43CFBA61AACABP-4", "0X1.E848P+19"),
            -3,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("-0X1.BD393CE9E8E7CP+4", "-0X1.2A95F6F7C066CP-3"),
            -3,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = -7
        (MT, -7, MT),
        (ENT, -7, ENT),
        (Some((0.0, 0.0)), -7, MT),
        (Some((-0.0, -0.0)), -7, MT),
        (
            hh("0X1.037D76C912DBCP-26", "0X1.037D76C912DBDP-26"),
            -7,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            hh("-0X1.F10F41FB8858FP-91", "-0X1.F10F41FB8858EP-91"),
            -7,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            hh("0X0P+0", "0X0.0000000000001P-1022"),
            -7,
            Some((hx("0x1.588cea3f093bcp+153"), INF)),
        ),
        (
            hh("-0X0.0000000000001P-1022", "-0X0P+0"),
            -7,
            Some((NEG_INF, hx("-0x1.588cea3f093bcp+153"))),
        ),
        (Some((0.0, INF)), -7, Some((0.0, INF))),
        (Some((-0.0, INF)), -7, Some((0.0, INF))),
        (Some((NEG_INF, 0.0)), -7, Some((NEG_INF, 0.0))),
        (Some((NEG_INF, -0.0)), -7, Some((NEG_INF, 0.0))),
        (
            hh("0X1.5F934D64162A9P-9", "0X1.6BCC41E9P+46"),
            -7,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            hh("-0X1.254CDD3711DDBP+11", "-0X1.6E95C4A761E19P-7"),
            -7,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
    ]
}

/// The `minimal_pown_rev_bin_test` corpus (37 vectors).
fn pown_bin_cases() -> Vec<(E, E, i32, E)> {
    vec![
        // n = 0
        (MT, Some((1.0, 1.0)), 0, MT),
        (Some((1.0, 1.0)), Some((1.0, 1.0)), 0, Some((1.0, 1.0))),
        (
            Some((-1.0, 5.0)),
            Some((-51.0, 12.0)),
            0,
            Some((-51.0, 12.0)),
        ),
        (Some((-1.0, 0.0)), Some((5.0, 10.0)), 0, MT),
        (Some((-1.0, -0.0)), Some((-1.0, 1.0)), 0, MT),
        (Some((1.1, 10.0)), Some((1.0, 41.0)), 0, MT),
        // n = 1
        (MT, Some((0.0, 100.1)), 1, MT),
        (ENT, Some((-5.1, 10.0)), 1, Some((-5.1, 10.0))),
        (Some((0.0, 0.0)), Some((-10.0, 5.1)), 1, Some((0.0, 0.0))),
        (Some((-0.0, -0.0)), Some((1.0, 5.0)), 1, MT),
        // n = 2
        (MT, Some((5.0, 17.1)), 2, MT),
        (Some((-5.0, -1.0)), Some((5.0, 17.1)), 2, MT),
        (
            Some((0.0, INF)),
            Some((5.6, 27.544)),
            2,
            Some((5.6, 27.544)),
        ),
        (Some((0.0, 0.0)), Some((1.0, 2.0)), 2, MT),
        (
            hh("0X1.A36E2EB1C432CP-14", "0X1.5B7318FC50482P+2"),
            Some((1.0, INF)),
            2,
            Some((1.0, hx("0x1.2a3d70a3d70a5p+1"))),
        ),
        (
            hh("0X1.BE0DED288CE7P-4", "0X1.CE147AE147AE1P+1"),
            Some((NEG_INF, -1.0)),
            2,
            Some((hx("-0x1.e666666666667p+0"), -1.0)),
        ),
        // n = 3
        (MT, Some((-23.0, -1.0)), 3, MT),
        (ENT, Some((-23.0, -1.0)), 3, Some((-23.0, -1.0))),
        (Some((0.0, 0.0)), Some((1.0, 2.0)), 3, MT),
        (
            hh("0X1.0C6F7A0B5ED8DP-20", "0X1.94C75E6362A6P+3"),
            Some((1.0, INF)),
            3,
            Some((1.0, hx("0x1.2a3d70a3d70a5p+1"))),
        ),
        (
            hh("-0X1.B6F9DB22D0E55P+2", "-0X1.266559F6EC5B1P-5"),
            Some((NEG_INF, -1.0)),
            3,
            Some((hx("-0x1.e666666666667p+0"), -1.0)),
        ),
        // n = -2
        (MT, Some((-3.0, 17.3)), -2, MT),
        (Some((0.0, INF)), Some((-5.1, -0.1)), -2, Some((-5.1, -0.1))),
        (Some((0.0, 0.0)), Some((27.2, 55.1)), -2, MT),
        (
            Some((hx("0X1.3F0C482C977C9P-17"), INF)),
            Some((NEG_INF, hx("-0x1.FFFFFFFFFFFFFp1023"))),
            -2,
            MT,
        ),
        (
            hh("0X1.793D85EF38E47P-3", "0X1.388P+13"),
            Some((1.0, INF)),
            -2,
            Some((1.0, hx("0x1.2a3d70a3d70a5p+1"))),
        ),
        (
            hh("0X1.1BA81104F6C8P-2", "0X1.25D8FA1F801E1P+3"),
            Some((NEG_INF, -1.0)),
            -2,
            Some((hx("-0x1.e666666666667p+0"), -1.0)),
        ),
        // n = -1
        (MT, Some((-5.1, 55.5)), -1, MT),
        (ENT, Some((-5.1, 55.5)), -1, Some((-5.1, 55.5))),
        (Some((0.0, 0.0)), Some((-5.1, 55.5)), -1, MT),
        (
            Some((NEG_INF, -0.0)),
            Some((-1.0, 1.0)),
            -1,
            Some((-1.0, 0.0)),
        ),
        (
            hh("0X1.B77C278DBBE13P-2", "0X1.9P+6"),
            Some((-1.0, 0.0)),
            -1,
            MT,
        ),
        // n = -3
        (MT, Some((-5.1, 55.5)), -3, MT),
        (ENT, Some((-5.1, 55.5)), -3, Some((-5.1, 55.5))),
        (Some((0.0, 0.0)), Some((-5.1, 55.5)), -3, MT),
        (Some((NEG_INF, 0.0)), Some((5.1, 55.5)), -3, MT),
        (
            Some((NEG_INF, -0.0)),
            Some((-32.0, 1.1)),
            -3,
            Some((-32.0, 0.0)),
        ),
    ]
}

/// The `minimal_pown_rev_dec_test` corpus (142 vectors).
fn pown_dec_cases() -> Vec<(Decoration, E, i32, E)> {
    vec![
        // n = 0
        (Trv, MT, 0, MT),
        (Com, Some((1.0, 1.0)), 0, ENT),
        (Dac, Some((-1.0, 5.0)), 0, ENT),
        (Def, Some((-1.0, 0.0)), 0, MT),
        (Dac, Some((-1.0, -0.0)), 0, MT),
        (Com, Some((1.1, 10.0)), 0, MT),
        // n = 1
        (Trv, MT, 1, MT),
        (Def, ENT, 1, ENT),
        (Com, Some((0.0, 0.0)), 1, Some((0.0, 0.0))),
        (Dac, Some((-0.0, -0.0)), 1, Some((0.0, 0.0))),
        (Def, Some((13.1, 13.1)), 1, Some((13.1, 13.1))),
        (
            Dac,
            Some((-7451.145, -7451.145)),
            1,
            Some((-7451.145, -7451.145)),
        ),
        (
            Com,
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            1,
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
        ),
        (
            Com,
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
            1,
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
        ),
        (Dac, Some((0.0, INF)), 1, Some((0.0, INF))),
        (Dac, Some((-0.0, INF)), 1, Some((0.0, INF))),
        (Def, Some((NEG_INF, 0.0)), 1, Some((NEG_INF, 0.0))),
        (Def, Some((NEG_INF, -0.0)), 1, Some((NEG_INF, 0.0))),
        (Dac, Some((-324.3, 2.5)), 1, Some((-324.3, 2.5))),
        (Com, Some((0.01, 2.33)), 1, Some((0.01, 2.33))),
        (Def, Some((-1.9, -0.33)), 1, Some((-1.9, -0.33))),
        // n = 2
        (Trv, MT, 2, MT),
        (Dac, Some((0.0, INF)), 2, ENT),
        (Def, Some((-0.0, INF)), 2, ENT),
        (Com, Some((0.0, 0.0)), 2, Some((0.0, 0.0))),
        (Dac, Some((-0.0, -0.0)), 2, Some((0.0, 0.0))),
        (
            Def,
            hh("0X1.573851EB851EBP+7", "0X1.573851EB851ECP+7"),
            2,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            Def,
            hh("0X1.A794A4E7CFAADP+25", "0X1.A794A4E7CFAAEP+25"),
            2,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (
            Dac,
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            2,
            hh("-0x1p+512", "0x1p+512"),
        ),
        (
            Dac,
            Some((0.0, hx("0X1.9AD27D70A3D72P+16"))),
            2,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Def,
            Some((-0.0, hx("0X1.9AD27D70A3D72P+16"))),
            2,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Com,
            hh("0X1.A36E2EB1C432CP-14", "0X1.5B7318FC50482P+2"),
            2,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Def,
            hh("0X1.BE0DED288CE7P-4", "0X1.CE147AE147AE1P+1"),
            2,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = 8
        (Trv, MT, 8, MT),
        (Def, ENT, 8, ENT),
        (Dac, Some((0.0, INF)), 8, ENT),
        (Dac, Some((-0.0, INF)), 8, ENT),
        (Def, Some((0.0, 0.0)), 8, Some((0.0, 0.0))),
        (Dac, Some((-0.0, -0.0)), 8, Some((0.0, 0.0))),
        (
            Com,
            hh("0X1.9D8FD495853F5P+29", "0X1.9D8FD495853F6P+29"),
            8,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            Dac,
            hh("0X1.DFB1BB622E70DP+102", "0X1.DFB1BB622E70EP+102"),
            8,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (
            Def,
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            8,
            hh("-0x1p+128", "0x1p+128"),
        ),
        (
            Dac,
            Some((0.0, hx("0X1.A87587109655P+66"))),
            8,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Def,
            Some((-0.0, hx("0X1.A87587109655P+66"))),
            8,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Com,
            hh("0X1.CD2B297D889BDP-54", "0X1.B253D9F33CE4DP+9"),
            8,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Dac,
            hh("0X1.26F1FCDD502A3P-13", "0X1.53ABD7BFC4FC6P+7"),
            8,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = 3
        (Trv, MT, 3, MT),
        (Def, ENT, 3, ENT),
        (Dac, Some((0.0, 0.0)), 3, Some((0.0, 0.0))),
        (Def, Some((-0.0, -0.0)), 3, Some((0.0, 0.0))),
        (
            Com,
            hh("0X1.1902E978D4FDEP+11", "0X1.1902E978D4FDFP+11"),
            3,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            Def,
            hh("-0X1.81460637B9A3DP+38", "-0X1.81460637B9A3CP+38"),
            3,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            Dac,
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            3,
            hh("0x1.428a2f98d728ap+341", "0x1.428a2f98d728bp+341"),
        ),
        (
            Com,
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
            3,
            hh("-0x1.428a2f98d728bp+341", "-0x1.428a2f98d728ap+341"),
        ),
        (Def, Some((0.0, INF)), 3, Some((0.0, INF))),
        (Def, Some((-0.0, INF)), 3, Some((0.0, INF))),
        (Dac, Some((NEG_INF, 0.0)), 3, Some((NEG_INF, 0.0))),
        (Def, Some((NEG_INF, -0.0)), 3, Some((NEG_INF, 0.0))),
        (
            Com,
            hh("-0X1.0436D2F418938P+25", "0X1.F4P+3"),
            3,
            hh("-0x1.444cccccccccep+8", "0x1.4p+1"),
        ),
        (
            Dac,
            hh("0X1.0C6F7A0B5ED8DP-20", "0X1.94C75E6362A6P+3"),
            3,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Def,
            hh("-0X1.B6F9DB22D0E55P+2", "-0X1.266559F6EC5B1P-5"),
            3,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = 7
        (Trv, MT, 7, MT),
        (Def, ENT, 7, ENT),
        (Com, Some((0.0, 0.0)), 7, Some((0.0, 0.0))),
        (Dac, Some((-0.0, -0.0)), 7, Some((0.0, 0.0))),
        (
            Def,
            hh("0X1.F91D1B185493BP+25", "0X1.F91D1B185493CP+25"),
            7,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            Dac,
            hh("-0X1.07B1DA32F9B59P+90", "-0X1.07B1DA32F9B58P+90"),
            7,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            Com,
            hh("0x1.FFFFFFFFFFFFFp1023", "0x1.FFFFFFFFFFFFFp1023"),
            7,
            hh("0x1.381147622f886p+146", "0x1.381147622f887p+146"),
        ),
        (
            Def,
            hh("-0x1.FFFFFFFFFFFFFp1023", "-0x1.FFFFFFFFFFFFFp1023"),
            7,
            hh("-0x1.381147622f887p+146", "-0x1.381147622f886p+146"),
        ),
        (Dac, Some((0.0, INF)), 7, Some((0.0, INF))),
        (Dac, Some((-0.0, INF)), 7, Some((0.0, INF))),
        (Def, Some((NEG_INF, 0.0)), 7, Some((NEG_INF, 0.0))),
        (Def, Some((NEG_INF, -0.0)), 7, Some((NEG_INF, 0.0))),
        (
            Dac,
            hh("-0X1.4F109959E6D7FP+58", "0X1.312DP+9"),
            7,
            hh("-0x1.444cccccccccep+8", "0x1.4p+1"),
        ),
        (
            Com,
            hh("0X1.6849B86A12B9BP-47", "0X1.74D0373C76313P+8"),
            7,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Def,
            hh("-0X1.658C775099757P+6", "-0X1.BEE30301BF47AP-12"),
            7,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = -2
        (Trv, MT, -2, MT),
        (Dac, Some((0.0, INF)), -2, ENT),
        (Dac, Some((-0.0, INF)), -2, ENT),
        (Def, Some((0.0, 0.0)), -2, MT),
        (Com, Some((-0.0, -0.0)), -2, MT),
        (Dac, Some((-10.0, 0.0)), -2, MT),
        (Def, Some((-10.0, -0.0)), -2, MT),
        (
            Dac,
            hh("0X1.7DE3A077D1568P-8", "0X1.7DE3A077D1569P-8"),
            -2,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            Def,
            hh("0X1.3570290CD6E14P-26", "0X1.3570290CD6E15P-26"),
            -2,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (Com, hh("0X0P+0", "0X0.0000000000001P-1022"), -2, ENT),
        (
            Dac,
            Some((hx("0X1.3F0C482C977C9P-17"), INF)),
            -2,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Def,
            hh("0X1.793D85EF38E47P-3", "0X1.388P+13"),
            -2,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Com,
            hh("0X1.1BA81104F6C8P-2", "0X1.25D8FA1F801E1P+3"),
            -2,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = -8
        (Trv, MT, -8, MT),
        (Def, Some((0.0, INF)), -8, ENT),
        (Dac, Some((-0.0, INF)), -8, ENT),
        (Def, Some((0.0, 0.0)), -8, MT),
        (Dac, Some((-0.0, -0.0)), -8, MT),
        (
            Com,
            hh("0X1.3CEF39247CA6DP-30", "0X1.3CEF39247CA6EP-30"),
            -8,
            hh("-0x1.a333333333334p+3", "0x1.a333333333334p+3"),
        ),
        (
            Def,
            hh("0X1.113D9EF0A99ACP-103", "0X1.113D9EF0A99ADP-103"),
            -8,
            hh("-0x1.d1b251eb851edp+12", "0x1.d1b251eb851edp+12"),
        ),
        (Dac, hh("0X0P+0", "0X0.0000000000001P-1022"), -8, ENT),
        (
            Def,
            Some((hx("0X1.34CC3764D1E0CP-67"), INF)),
            -8,
            hh("-0x1.444cccccccccep+8", "0x1.444cccccccccep+8"),
        ),
        (
            Com,
            hh("0X1.2DC80DB11AB7CP-10", "0X1.1C37937E08P+53"),
            -8,
            hh("-0x1.2a3d70a3d70a5p+1", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Def,
            hh("0X1.81E104E61630DP-8", "0X1.BC64F21560E34P+12"),
            -8,
            hh("-0x1.e666666666667p+0", "0x1.e666666666667p+0"),
        ),
        // n = -1
        (Trv, MT, -1, MT),
        (Def, ENT, -1, ENT),
        (Dac, Some((0.0, 0.0)), -1, MT),
        (Dac, Some((-0.0, -0.0)), -1, MT),
        (
            Def,
            hh("0X1.38ABF82EE6986P-4", "0X1.38ABF82EE6987P-4"),
            -1,
            hh("0x1.a333333333332p+3", "0x1.a333333333335p+3"),
        ),
        (
            Dac,
            hh("-0X1.197422C9048BFP-13", "-0X1.197422C9048BEP-13"),
            -1,
            hh("-0x1.d1b251eb851eep+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            Dac,
            hh("0X0.4P-1022", "0X0.4000000000001P-1022"),
            -1,
            Some((hx("0x1.ffffffffffff8p+1023"), INF)),
        ),
        (
            Def,
            hh("-0X0.4000000000001P-1022", "-0X0.4P-1022"),
            -1,
            Some((NEG_INF, hx("-0x1.ffffffffffff8p+1023"))),
        ),
        (Dac, Some((0.0, INF)), -1, Some((0.0, INF))),
        (Dac, Some((-0.0, INF)), -1, Some((0.0, INF))),
        (Dac, Some((NEG_INF, 0.0)), -1, Some((NEG_INF, 0.0))),
        (Def, Some((NEG_INF, -0.0)), -1, Some((NEG_INF, 0.0))),
        (
            Com,
            hh("0X1.B77C278DBBE13P-2", "0X1.9P+6"),
            -1,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Com,
            hh("-0X1.83E0F83E0F83EP+1", "-0X1.0D79435E50D79P-1"),
            -1,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = -3
        (Trv, MT, -3, MT),
        (Def, ENT, -3, ENT),
        (Def, Some((0.0, 0.0)), -3, MT),
        (Dac, Some((-0.0, -0.0)), -3, MT),
        (
            Com,
            hh("0X1.D26DF4D8B1831P-12", "0X1.D26DF4D8B1832P-12"),
            -3,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            Def,
            hh("-0X1.54347DED91B19P-39", "-0X1.54347DED91B18P-39"),
            -3,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            Dac,
            hh("0X0P+0", "0X0.0000000000001P-1022"),
            -3,
            Some((hx("0x1p+358"), INF)),
        ),
        (
            Def,
            hh("-0X0.0000000000001P-1022", "-0X0P+0"),
            -3,
            Some((NEG_INF, hx("-0x1p+358"))),
        ),
        (Dac, Some((0.0, INF)), -3, Some((0.0, INF))),
        (Dac, Some((-0.0, INF)), -3, Some((0.0, INF))),
        (Def, Some((NEG_INF, 0.0)), -3, Some((NEG_INF, 0.0))),
        (Def, Some((NEG_INF, -0.0)), -3, Some((NEG_INF, 0.0))),
        (
            Com,
            hh("0X1.43CFBA61AACABP-4", "0X1.E848P+19"),
            -3,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Def,
            hh("-0X1.BD393CE9E8E7CP+4", "-0X1.2A95F6F7C066CP-3"),
            -3,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
        // n = -7
        (Trv, MT, -7, MT),
        (Def, ENT, -7, ENT),
        (Com, Some((0.0, 0.0)), -7, MT),
        (Def, Some((-0.0, -0.0)), -7, MT),
        (
            Dac,
            hh("0X1.037D76C912DBCP-26", "0X1.037D76C912DBDP-26"),
            -7,
            hh("0x1.a333333333332p+3", "0x1.a333333333334p+3"),
        ),
        (
            Dac,
            hh("-0X1.F10F41FB8858FP-91", "-0X1.F10F41FB8858EP-91"),
            -7,
            hh("-0x1.d1b251eb851edp+12", "-0x1.d1b251eb851ebp+12"),
        ),
        (
            Def,
            hh("0X0P+0", "0X0.0000000000001P-1022"),
            -7,
            Some((hx("0x1.588cea3f093bcp+153"), INF)),
        ),
        (
            Def,
            hh("-0X0.0000000000001P-1022", "-0X0P+0"),
            -7,
            Some((NEG_INF, hx("-0x1.588cea3f093bcp+153"))),
        ),
        (Dac, Some((0.0, INF)), -7, Some((0.0, INF))),
        (Def, Some((-0.0, INF)), -7, Some((0.0, INF))),
        (Dac, Some((NEG_INF, 0.0)), -7, Some((NEG_INF, 0.0))),
        (Def, Some((NEG_INF, -0.0)), -7, Some((NEG_INF, 0.0))),
        (
            Com,
            hh("0X1.5F934D64162A9P-9", "0X1.6BCC41E9P+46"),
            -7,
            hh("0x1.47ae147ae147ap-7", "0x1.2a3d70a3d70a5p+1"),
        ),
        (
            Com,
            hh("-0X1.254CDD3711DDBP+11", "-0X1.6E95C4A761E19P-7"),
            -7,
            hh("-0x1.e666666666667p+0", "-0x1.51eb851eb851ep-2"),
        ),
    ]
}

/// The `minimal_pown_rev_dec_bin_test` corpus (36 vectors).
fn pown_dec_bin_cases() -> Vec<(Decoration, E, Decoration, E, i32, E)> {
    vec![
        // n = 0
        (Trv, MT, Def, Some((1.0, 1.0)), 0, MT),
        (
            Dac,
            Some((1.0, 1.0)),
            Dac,
            Some((1.0, 1.0)),
            0,
            Some((1.0, 1.0)),
        ),
        (
            Def,
            Some((-1.0, 5.0)),
            Dac,
            Some((-51.0, 12.0)),
            0,
            Some((-51.0, 12.0)),
        ),
        (Com, Some((-1.0, 0.0)), Dac, Some((5.0, 10.0)), 0, MT),
        (Dac, Some((-1.0, -0.0)), Def, Some((-1.0, 1.0)), 0, MT),
        (Def, Some((1.1, 10.0)), Dac, Some((1.0, 41.0)), 0, MT),
        // n = 1
        (Trv, MT, Dac, Some((0.0, 100.1)), 1, MT),
        (Def, ENT, Def, Some((-5.1, 10.0)), 1, Some((-5.1, 10.0))),
        (
            Com,
            Some((0.0, 0.0)),
            Dac,
            Some((-10.0, 5.1)),
            1,
            Some((0.0, 0.0)),
        ),
        (Def, Some((-0.0, -0.0)), Dac, Some((1.0, 5.0)), 1, MT),
        // n = 2
        (Trv, MT, Def, Some((5.0, 17.1)), 2, MT),
        (
            Dac,
            Some((0.0, INF)),
            Dac,
            Some((5.6, 27.544)),
            2,
            Some((5.6, 27.544)),
        ),
        (Def, Some((0.0, 0.0)), Def, Some((1.0, 2.0)), 2, MT),
        (
            Com,
            hh("0X1.A36E2EB1C432CP-14", "0X1.5B7318FC50482P+2"),
            Def,
            Some((1.0, INF)),
            2,
            Some((1.0, hx("0x1.2a3d70a3d70a5p+1"))),
        ),
        (
            Dac,
            hh("0X1.BE0DED288CE7P-4", "0X1.CE147AE147AE1P+1"),
            Def,
            Some((NEG_INF, -1.0)),
            2,
            Some((hx("-0x1.e666666666667p+0"), -1.0)),
        ),
        // n = 3
        (Trv, MT, Dac, Some((-23.0, -1.0)), 3, MT),
        (Def, ENT, Com, Some((-23.0, -1.0)), 3, Some((-23.0, -1.0))),
        (Def, Some((0.0, 0.0)), Dac, Some((1.0, 2.0)), 3, MT),
        (
            Com,
            hh("0X1.0C6F7A0B5ED8DP-20", "0X1.94C75E6362A6P+3"),
            Dac,
            Some((1.0, INF)),
            3,
            Some((1.0, hx("0x1.2a3d70a3d70a5p+1"))),
        ),
        (
            Com,
            hh("-0X1.B6F9DB22D0E55P+2", "-0X1.266559F6EC5B1P-5"),
            Dac,
            Some((NEG_INF, -1.0)),
            3,
            Some((hx("-0x1.e666666666667p+0"), -1.0)),
        ),
        // n = -2
        (Trv, MT, Def, Some((-3.0, 17.3)), -2, MT),
        (
            Dac,
            Some((0.0, INF)),
            Dac,
            Some((-5.1, -0.1)),
            -2,
            Some((-5.1, -0.1)),
        ),
        (Def, Some((0.0, 0.0)), Dac, Some((27.2, 55.1)), -2, MT),
        (
            Def,
            Some((hx("0X1.3F0C482C977C9P-17"), INF)),
            Dac,
            Some((NEG_INF, hx("-0x1.FFFFFFFFFFFFFp1023"))),
            -2,
            MT,
        ),
        (
            Com,
            hh("0X1.793D85EF38E47P-3", "0X1.388P+13"),
            Dac,
            Some((1.0, INF)),
            -2,
            Some((1.0, hx("0x1.2a3d70a3d70a5p+1"))),
        ),
        (
            Com,
            hh("0X1.1BA81104F6C8P-2", "0X1.25D8FA1F801E1P+3"),
            Dac,
            Some((NEG_INF, -1.0)),
            -2,
            Some((hx("-0x1.e666666666667p+0"), -1.0)),
        ),
        // n = -1
        (Trv, MT, Def, Some((-5.1, 55.5)), -1, MT),
        (Def, ENT, Dac, Some((-5.1, 55.5)), -1, Some((-5.1, 55.5))),
        (Dac, Some((0.0, 0.0)), Def, Some((-5.1, 55.5)), -1, MT),
        (
            Dac,
            Some((NEG_INF, -0.0)),
            Com,
            Some((-1.0, 1.0)),
            -1,
            Some((-1.0, 0.0)),
        ),
        (
            Def,
            hh("0X1.B77C278DBBE13P-2", "0X1.9P+6"),
            Dac,
            Some((-1.0, 0.0)),
            -1,
            MT,
        ),
        // n = -3
        (Trv, MT, Dac, Some((-5.1, 55.5)), -3, MT),
        (Def, ENT, Def, Some((-5.1, 55.5)), -3, Some((-5.1, 55.5))),
        (Def, Some((0.0, 0.0)), Def, Some((-5.1, 55.5)), -3, MT),
        (Dac, Some((NEG_INF, 0.0)), Com, Some((5.1, 55.5)), -3, MT),
        (
            Dac,
            Some((NEG_INF, -0.0)),
            Def,
            Some((-32.0, 1.1)),
            -3,
            Some((-32.0, 0.0)),
        ),
    ]
}

// The pownRev corpora are one running bit-exact lane per corpus, no carve-outs.
// Over `TightF64` the `pown_rev` roots ride the exact `RoundPown` kernel and end
// in a neighbor certification (round-float decision record 0004; ADR-0006
// erratum 2026-07-22), so every row is the tightest representable pair. The two
// classes ADR-0006 part 3 had left `#[ignore]` are resolved (beads enc-cov,
// enc-ral):
//
//  * The root looseness for |n| >= 3 (once 44 vectors). The minimized case,
//    `iv(0.0, hx("0X1.A87587109655P+66")).pown_rev(entire(), 8)`, now yields the
//    corpus's sup `0x407444ccccccccce` (was one ulp high at `...ccf`): the
//    certification walks the bracket to the two adjacent floats the corpus
//    pins.
//  * The negative-exponent subnormal edge (once 8 vectors). The negative path
//    roots first then reciprocates at the scalar level, so
//    `iv(0.0, 5e-324).pown_rev(entire(), -3)` reaches `2^358` (was `~2^341`, the
//    saturated set-level reciprocal). Where the constraint reciprocal is
//    representable the pair is certified tightest; where it overflows the
//    directed root-then-reciprocal seed stands, matching the reference's own
//    `rootn` at that edge (the corpus pins that value, one ulp inside the
//    tightest floor for a non-power-of-two root such as `2^(1074/7)`).

// minimal_pown_rev_test: 143 vectors, all bit-exact.
#[test]
fn pown_rev_test() {
    for (c, n, w) in pown_bare_cases() {
        eq(build(c).pown_rev(entire(), n), w);
    }
}

// minimal_pown_rev_bin_test: 37 vectors, all bit-exact.
#[test]
fn pown_rev_bin_test() {
    for (c, x, n, w) in pown_bin_cases() {
        eq(build(c).pown_rev(build(x), n), w);
    }
}

// minimal_pown_rev_dec_test: 142 vectors, all bit-exact.
#[test]
fn pown_rev_dec_test() {
    for (cd, c, n, w) in pown_dec_cases() {
        eq_trv(dec(c, cd).pown_rev(new_dec(ENT), n), w);
    }
}

// minimal_pown_rev_dec_bin_test: 36 vectors, all bit-exact.
#[test]
fn pown_rev_dec_bin_test() {
    for (cd, c, xd, x, n, w) in pown_dec_bin_cases() {
        eq_trv(dec(c, cd).pown_rev(dec(x, xd), n), w);
    }
}

// ============================================================================
// mulRev  (libieeep1788_rev.itl)
// ============================================================================

// minimal_mul_rev_test: 172 vectors
#[test]
fn mul_rev_test() {
    let a = hx("0X1.999999999999AP-3");
    let big = hx("0X1.5P+4");
    let s2 = hx("0X1.745D1745D1745P-2");
    let t = hx("0X1.A400000000001P+7");
    let onetwo = hx("0X1.3333333333333P+0");
    let sm8 = hx("0X1.47AE147AE147BP-8");
    let lo7 = hx("0X1.29E4129E4129DP-7");
    let twelve = hx("0X1.8P+3");
    let a5 = hx("0X1.999999999999AP-5");
    let s4 = hx("0X1.745D1745D1745P-4");
    let three = hx("0X1.8P+1");
    let thirty = hx("0X1.EP+4");
    let two1 = hx("0X1.0CCCCCCCCCCCDP+1");
    let sm6 = hx("0X1.47AE147AE147BP-6");
    let lo5 = hx("0X1.29E4129E4129DP-5");
    // The 13 `b` operands, in the corpus's per-group row order.
    let bs: [E; 13] = [
        Some((-2.0, -0.1)),
        Some((-2.0, 0.0)),
        Some((-2.0, 1.1)),
        Some((0.0, 1.1)),
        Some((0.01, 1.1)),
        Some((0.0, 0.0)),
        Some((NEG_INF, -0.1)),
        Some((NEG_INF, 0.0)),
        Some((NEG_INF, 1.1)),
        Some((-2.0, INF)),
        Some((0.0, INF)),
        Some((0.01, INF)),
        ENT,
    ];
    let group = |c: E, wants: [E; 13]| {
        for (b, w) in bs.iter().copied().zip(wants) {
            eq(build(b).mul_rev(build(c), entire()), w);
        }
    };
    // Three empty-operand rows.
    eq(build(MT).mul_rev(build(Some((1.0, 2.0))), entire()), MT);
    eq(build(Some((1.0, 2.0))).mul_rev(build(MT), entire()), MT);
    eq(build(MT).mul_rev(build(MT), entire()), MT);
    // c = [-2.1, -0.4]
    group(
        Some((-2.1, -0.4)),
        [
            Some((a, big)),
            Some((a, INF)),
            ENT,
            Some((NEG_INF, -s2)),
            Some((-t, -s2)),
            MT,
            Some((0.0, big)),
            Some((0.0, INF)),
            ENT,
            ENT,
            Some((NEG_INF, 0.0)),
            Some((-t, 0.0)),
            ENT,
        ],
    );
    // c = [-2.1, 0.0]
    group(
        Some((-2.1, 0.0)),
        [
            Some((0.0, big)),
            ENT,
            ENT,
            ENT,
            Some((-t, 0.0)),
            ENT,
            Some((0.0, big)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((-t, 0.0)),
            ENT,
        ],
    );
    // c = [-2.1, 0.12]
    group(
        Some((-2.1, 0.12)),
        [
            Some((-onetwo, big)),
            ENT,
            ENT,
            ENT,
            Some((-t, twelve)),
            ENT,
            Some((-onetwo, big)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((-t, twelve)),
            ENT,
        ],
    );
    // c = [0.0, 0.12]
    group(
        Some((0.0, 0.12)),
        [
            Some((-onetwo, 0.0)),
            ENT,
            ENT,
            ENT,
            Some((0.0, twelve)),
            ENT,
            Some((-onetwo, 0.0)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((0.0, twelve)),
            ENT,
        ],
    );
    // c = [0.01, 0.12]
    group(
        Some((0.01, 0.12)),
        [
            Some((-onetwo, -sm8)),
            Some((NEG_INF, -sm8)),
            ENT,
            Some((lo7, INF)),
            Some((lo7, twelve)),
            MT,
            Some((-onetwo, 0.0)),
            Some((NEG_INF, 0.0)),
            ENT,
            ENT,
            Some((0.0, INF)),
            Some((0.0, twelve)),
            ENT,
        ],
    );
    // c = [0.0, 0.0]
    group(
        Some((0.0, 0.0)),
        [
            Some((0.0, 0.0)),
            ENT,
            ENT,
            ENT,
            Some((0.0, 0.0)),
            ENT,
            Some((0.0, 0.0)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((0.0, 0.0)),
            ENT,
        ],
    );
    // c = [-inf, -0.1]
    group(
        Some((NEG_INF, -0.1)),
        [
            Some((a5, INF)),
            Some((a5, INF)),
            ENT,
            Some((NEG_INF, -s4)),
            Some((NEG_INF, -s4)),
            MT,
            Some((0.0, INF)),
            Some((0.0, INF)),
            ENT,
            ENT,
            Some((NEG_INF, 0.0)),
            Some((NEG_INF, 0.0)),
            ENT,
        ],
    );
    // c = [-inf, 0.0]
    group(
        Some((NEG_INF, 0.0)),
        [
            Some((0.0, INF)),
            ENT,
            ENT,
            ENT,
            Some((NEG_INF, 0.0)),
            ENT,
            Some((0.0, INF)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((NEG_INF, 0.0)),
            ENT,
        ],
    );
    // c = [-inf, 0.3]
    group(
        Some((NEG_INF, 0.3)),
        [
            Some((-three, INF)),
            ENT,
            ENT,
            ENT,
            Some((NEG_INF, thirty)),
            ENT,
            Some((-three, INF)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((NEG_INF, thirty)),
            ENT,
        ],
    );
    // c = [-0.21, inf]
    group(
        Some((-0.21, INF)),
        [
            Some((NEG_INF, two1)),
            ENT,
            ENT,
            ENT,
            Some((-big, INF)),
            ENT,
            Some((NEG_INF, two1)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((-big, INF)),
            ENT,
        ],
    );
    // c = [0.0, inf]
    group(
        Some((0.0, INF)),
        [
            Some((NEG_INF, 0.0)),
            ENT,
            ENT,
            ENT,
            Some((0.0, INF)),
            ENT,
            Some((NEG_INF, 0.0)),
            ENT,
            ENT,
            ENT,
            ENT,
            Some((0.0, INF)),
            ENT,
        ],
    );
    // c = [0.04, inf]
    group(
        Some((0.04, INF)),
        [
            Some((NEG_INF, -sm6)),
            Some((NEG_INF, -sm6)),
            ENT,
            Some((lo5, INF)),
            Some((lo5, INF)),
            MT,
            Some((NEG_INF, 0.0)),
            Some((NEG_INF, 0.0)),
            ENT,
            ENT,
            Some((0.0, INF)),
            Some((0.0, INF)),
            ENT,
        ],
    );
    // c = [entire]
    group(ENT, [ENT; 13]);
}

// minimal_mul_rev_ten_test: 5 vectors
#[test]
fn mul_rev_ten_test() {
    let v = [
        (
            Some((-2.0, -0.1)),
            Some((-2.1, -0.4)),
            Some((-2.1, -0.4)),
            MT,
        ),
        (
            Some((-2.0, 1.1)),
            Some((-2.1, -0.4)),
            Some((-2.1, -0.4)),
            Some((-2.1, -0.4)),
        ),
        (
            Some((0.01, 1.1)),
            Some((-2.1, 0.0)),
            Some((-2.1, 0.0)),
            Some((-2.1, 0.0)),
        ),
        (
            Some((NEG_INF, -0.1)),
            Some((0.0, 0.12)),
            Some((0.0, 0.12)),
            Some((0.0, 0.0)),
        ),
        (
            Some((-2.0, 1.1)),
            Some((0.04, INF)),
            Some((0.04, INF)),
            Some((0.04, INF)),
        ),
    ];
    for (b, c, x, w) in v {
        eq(build(b).mul_rev(build(c), build(x)), w);
    }
}

// minimal_mul_rev_dec_test: 10 vectors
#[test]
fn mul_rev_dec_test() {
    // NaI in any operand poisons the result.
    assert!(nai()
        .mul_rev(dec(Some((1.0, 2.0)), Dac), new_dec(ENT))
        .is_nai());
    assert!(dec(Some((1.0, 2.0)), Dac)
        .mul_rev(nai(), new_dec(ENT))
        .is_nai());
    assert!(nai().mul_rev(nai(), new_dec(ENT)).is_nai());
    // The seven bare rows, every one grading trv.
    let a = hx("0X1.999999999999AP-3");
    let big = hx("0X1.5P+4");
    let onetwo = hx("0X1.3333333333333P+0");
    let lo7 = hx("0X1.29E4129E4129DP-7");
    let twelve = hx("0X1.8P+3");
    let thirty = hx("0X1.EP+4");
    let two1 = hx("0X1.0CCCCCCCCCCCDP+1");
    let v = [
        (
            Dac,
            Some((-2.0, -0.1)),
            Dac,
            Some((-2.1, -0.4)),
            Some((a, big)),
        ),
        (
            Def,
            Some((-2.0, -0.1)),
            Def,
            Some((-2.1, 0.0)),
            Some((0.0, big)),
        ),
        (
            Com,
            Some((-2.0, -0.1)),
            Dac,
            Some((-2.1, 0.12)),
            Some((-onetwo, big)),
        ),
        (
            Dac,
            Some((NEG_INF, -0.1)),
            Com,
            Some((0.0, 0.12)),
            Some((-onetwo, 0.0)),
        ),
        (
            Def,
            Some((0.01, 1.1)),
            Dac,
            Some((0.01, 0.12)),
            Some((lo7, twelve)),
        ),
        (
            Dac,
            Some((0.01, 1.1)),
            Def,
            Some((NEG_INF, 0.3)),
            Some((NEG_INF, thirty)),
        ),
        (
            Trv,
            Some((NEG_INF, -0.1)),
            Dac,
            Some((-0.21, INF)),
            Some((NEG_INF, two1)),
        ),
    ];
    for (bd, b, cd, c, w) in v {
        eq_trv(dec(b, bd).mul_rev(dec(c, cd), new_dec(ENT)), w);
    }
}

// minimal_mul_rev_dec_ten_test: 5 vectors
#[test]
fn mul_rev_dec_ten_test() {
    let v = [
        (
            Dac,
            Some((-2.0, -0.1)),
            Dac,
            Some((-2.1, -0.4)),
            Dac,
            Some((-2.1, -0.4)),
            MT,
        ),
        (
            Def,
            Some((-2.0, 1.1)),
            Com,
            Some((-2.1, -0.4)),
            Com,
            Some((-2.1, -0.4)),
            Some((-2.1, -0.4)),
        ),
        (
            Com,
            Some((0.01, 1.1)),
            Dac,
            Some((-2.1, 0.0)),
            Dac,
            Some((-2.1, 0.0)),
            Some((-2.1, 0.0)),
        ),
        (
            Dac,
            Some((NEG_INF, -0.1)),
            Com,
            Some((0.0, 0.12)),
            Com,
            Some((0.0, 0.12)),
            Some((0.0, 0.0)),
        ),
        (
            Def,
            Some((-2.0, 1.1)),
            Dac,
            Some((0.04, INF)),
            Dac,
            Some((0.04, INF)),
            Some((0.04, INF)),
        ),
    ];
    for (bd, b, cd, c, xd, x, w) in v {
        eq_trv(dec(b, bd).mul_rev(dec(c, cd), dec(x, xd)), w);
    }
}

// ============================================================================
// mulRevToPair  (libieeep1788_mul_rev.itl)
// ============================================================================

/// The 13 `b` operands in the corpus's per-group row order (shared by the
/// bare and decorated `mulRevToPair` grids).
fn pair_bs() -> [E; 13] {
    [
        Some((-2.0, -0.1)),
        Some((-2.0, 0.0)),
        Some((-2.0, 1.1)),
        Some((0.0, 1.1)),
        Some((0.01, 1.1)),
        Some((0.0, 0.0)),
        Some((NEG_INF, -0.1)),
        Some((NEG_INF, 0.0)),
        Some((NEG_INF, 1.1)),
        Some((-2.0, INF)),
        Some((0.0, INF)),
        Some((0.01, INF)),
        ENT,
    ]
}

// minimal_mulRevToPair_test: 172 vectors
#[test]
fn mul_rev_to_pair_test() {
    let a = hx("0X1.999999999999AP-3");
    let big = hx("0X1.5P+4");
    let s2 = hx("0X1.745D1745D1745P-2");
    let t = hx("0X1.A400000000001P+7");
    let onetwo = hx("0X1.3333333333333P+0");
    let sm8 = hx("0X1.47AE147AE147BP-8");
    let lo7 = hx("0X1.29E4129E4129DP-7");
    let twelve = hx("0X1.8P+3");
    let a5 = hx("0X1.999999999999AP-5");
    let s4 = hx("0X1.745D1745D1745P-4");
    let three = hx("0X1.8P+1");
    let thirty = hx("0X1.EP+4");
    let two1 = hx("0X1.0CCCCCCCCCCCDP+1");
    let sm6 = hx("0X1.47AE147AE147BP-6");
    let lo5 = hx("0X1.29E4129E4129DP-5");
    let e: E = MT; // empty piece shorthand
    let bs = pair_bs();
    let group = |c: E, wants: [(E, E); 13]| {
        for (b, (p1w, p2w)) in bs.iter().copied().zip(wants) {
            let (p1, p2) = build(b).mul_rev_to_pair(build(c));
            eq(p1, p1w);
            eq(p2, p2w);
        }
    };
    // Three empty-operand rows.
    let (p1, p2) = build(MT).mul_rev_to_pair(build(Some((1.0, 2.0))));
    eq(p1, MT);
    eq(p2, MT);
    let (p1, p2) = build(Some((1.0, 2.0))).mul_rev_to_pair(build(MT));
    eq(p1, MT);
    eq(p2, MT);
    let (p1, p2) = build(MT).mul_rev_to_pair(build(MT));
    eq(p1, MT);
    eq(p2, MT);
    // c = [-2.1, -0.4]
    group(
        Some((-2.1, -0.4)),
        [
            (Some((a, big)), e),
            (Some((a, INF)), e),
            (Some((NEG_INF, -s2)), Some((a, INF))),
            (Some((NEG_INF, -s2)), e),
            (Some((-t, -s2)), e),
            (e, e),
            (Some((0.0, big)), e),
            (Some((0.0, INF)), e),
            (Some((NEG_INF, -s2)), Some((0.0, INF))),
            (Some((NEG_INF, 0.0)), Some((a, INF))),
            (Some((NEG_INF, 0.0)), e),
            (Some((-t, 0.0)), e),
            (Some((NEG_INF, 0.0)), Some((0.0, INF))),
        ],
    );
    // c = [-2.1, 0.0]
    group(
        Some((-2.1, 0.0)),
        [
            (Some((0.0, big)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((-t, 0.0)), e),
            (ENT, e),
            (Some((0.0, big)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((-t, 0.0)), e),
            (ENT, e),
        ],
    );
    // c = [-2.1, 0.12]
    group(
        Some((-2.1, 0.12)),
        [
            (Some((-onetwo, big)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((-t, twelve)), e),
            (ENT, e),
            (Some((-onetwo, big)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((-t, twelve)), e),
            (ENT, e),
        ],
    );
    // c = [0.0, 0.12]
    group(
        Some((0.0, 0.12)),
        [
            (Some((-onetwo, 0.0)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((0.0, twelve)), e),
            (ENT, e),
            (Some((-onetwo, 0.0)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((0.0, twelve)), e),
            (ENT, e),
        ],
    );
    // c = [0.01, 0.12]
    group(
        Some((0.01, 0.12)),
        [
            (Some((-onetwo, -sm8)), e),
            (Some((NEG_INF, -sm8)), e),
            (Some((NEG_INF, -sm8)), Some((lo7, INF))),
            (Some((lo7, INF)), e),
            (Some((lo7, twelve)), e),
            (e, e),
            (Some((-onetwo, 0.0)), e),
            (Some((NEG_INF, 0.0)), e),
            (Some((NEG_INF, 0.0)), Some((lo7, INF))),
            (Some((NEG_INF, -sm8)), Some((0.0, INF))),
            (Some((0.0, INF)), e),
            (Some((0.0, twelve)), e),
            (Some((NEG_INF, 0.0)), Some((0.0, INF))),
        ],
    );
    // c = [0.0, 0.0]
    group(
        Some((0.0, 0.0)),
        [
            (Some((0.0, 0.0)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((0.0, 0.0)), e),
            (ENT, e),
            (Some((0.0, 0.0)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((0.0, 0.0)), e),
            (ENT, e),
        ],
    );
    // c = [-inf, -0.1]
    group(
        Some((NEG_INF, -0.1)),
        [
            (Some((a5, INF)), e),
            (Some((a5, INF)), e),
            (Some((NEG_INF, -s4)), Some((a5, INF))),
            (Some((NEG_INF, -s4)), e),
            (Some((NEG_INF, -s4)), e),
            (e, e),
            (Some((0.0, INF)), e),
            (Some((0.0, INF)), e),
            (Some((NEG_INF, -s4)), Some((0.0, INF))),
            (Some((NEG_INF, 0.0)), Some((a5, INF))),
            (Some((NEG_INF, 0.0)), e),
            (Some((NEG_INF, 0.0)), e),
            (Some((NEG_INF, 0.0)), Some((0.0, INF))),
        ],
    );
    // c = [-inf, 0.0]
    group(
        Some((NEG_INF, 0.0)),
        [
            (Some((0.0, INF)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((NEG_INF, 0.0)), e),
            (ENT, e),
            (Some((0.0, INF)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((NEG_INF, 0.0)), e),
            (ENT, e),
        ],
    );
    // c = [-inf, 0.3]
    group(
        Some((NEG_INF, 0.3)),
        [
            (Some((-three, INF)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((NEG_INF, thirty)), e),
            (ENT, e),
            (Some((-three, INF)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((NEG_INF, thirty)), e),
            (ENT, e),
        ],
    );
    // c = [-0.21, inf]
    group(
        Some((-0.21, INF)),
        [
            (Some((NEG_INF, two1)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((-big, INF)), e),
            (ENT, e),
            (Some((NEG_INF, two1)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((-big, INF)), e),
            (ENT, e),
        ],
    );
    // c = [0.0, inf]
    group(
        Some((0.0, INF)),
        [
            (Some((NEG_INF, 0.0)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((0.0, INF)), e),
            (ENT, e),
            (Some((NEG_INF, 0.0)), e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (ENT, e),
            (Some((0.0, INF)), e),
            (ENT, e),
        ],
    );
    // c = [0.04, inf]
    group(
        Some((0.04, INF)),
        [
            (Some((NEG_INF, -sm6)), e),
            (Some((NEG_INF, -sm6)), e),
            (Some((NEG_INF, -sm6)), Some((lo5, INF))),
            (Some((lo5, INF)), e),
            (Some((lo5, INF)), e),
            (e, e),
            (Some((NEG_INF, 0.0)), e),
            (Some((NEG_INF, 0.0)), e),
            (Some((NEG_INF, 0.0)), Some((lo5, INF))),
            (Some((NEG_INF, -sm6)), Some((0.0, INF))),
            (Some((0.0, INF)), e),
            (Some((0.0, INF)), e),
            (Some((NEG_INF, 0.0)), Some((0.0, INF))),
        ],
    );
    // c = [entire]
    group(ENT, [(ENT, e); 13]);
}

// minimal_mulRevToPair_dec_test: 175 vectors
//
// The corpus assigns the reverse-division's first output piece the PROPAGATED
// decoration of the normal division (`com`/`dac`/`def`) whenever `0` is not in
// `b`, and grades the empty second piece `trv`. The standard defines the two-
// output division in its own subclause and gives the first piece the division
// decoration (clause 12.12.3); the crate matches, so this lane transcribes the
// corpus decorations faithfully and asserts them exactly. The intervals are also
// pinned (and pass) by `mul_rev_to_pair_test`. The generic `trv` doctrine stands
// for the one-output reverse operations; only the two-output division propagates.
#[test]
fn mul_rev_to_pair_dec_test() {
    let a = hx("0X1.999999999999AP-3");
    let big = hx("0X1.5P+4");
    let s2 = hx("0X1.745D1745D1745P-2");
    let t = hx("0X1.A400000000001P+7");
    let onetwo = hx("0X1.3333333333333P+0");
    let sm8 = hx("0X1.47AE147AE147BP-8");
    let lo7 = hx("0X1.29E4129E4129DP-7");
    let twelve = hx("0X1.8P+3");
    let a5 = hx("0X1.999999999999AP-5");
    let s4 = hx("0X1.745D1745D1745P-4");
    let three = hx("0X1.8P+1");
    let thirty = hx("0X1.EP+4");
    let two1 = hx("0X1.0CCCCCCCCCCCDP+1");
    let sm6 = hx("0X1.47AE147AE147BP-6");
    let lo5 = hx("0X1.29E4129E4129DP-5");
    let c1 = Some((-2.1, -0.4));
    let c2 = Some((-2.1, 0.0));
    let c3 = Some((-2.1, 0.12));
    let c4 = Some((0.0, 0.12));
    let c5 = Some((0.01, 0.12));
    let c6 = Some((0.0, 0.0));
    let c7 = Some((NEG_INF, -0.1));
    let c8 = Some((NEG_INF, 0.0));
    let c9 = Some((NEG_INF, 0.3));
    let c10 = Some((-0.21, INF));
    let c11 = Some((0.0, INF));
    let c12 = Some((0.04, INF));
    // A `Vec` (not a stack array): the 175-row grid exceeds the stack-array
    // lint's budget.
    let rows = vec![
        // NaI and empty prefix.
        (DI::Nai, di(Some((1.0, 2.0)), Def), DI::Nai, DI::Nai),
        (di(Some((1.0, 2.0)), Com), DI::Nai, DI::Nai, DI::Nai),
        (DI::Nai, DI::Nai, DI::Nai, DI::Nai),
        (di(MT, Trv), di(Some((1.0, 2.0)), Def), et(), et()),
        (di(Some((1.0, 2.0)), Com), di(MT, Trv), et(), et()),
        (di(MT, Trv), di(MT, Trv), et(), et()),
        // c = [-2.1, -0.4]
        (
            di(Some((-2.0, -0.1)), Com),
            di(c1, Com),
            di(Some((a, big)), Com),
            et(),
        ),
        (
            di(Some((-2.0, 0.0)), Dac),
            di(c1, Com),
            di(Some((a, INF)), Trv),
            et(),
        ),
        (
            di(Some((-2.0, 1.1)), Com),
            di(c1, Dac),
            di(Some((NEG_INF, -s2)), Trv),
            di(Some((a, INF)), Trv),
        ),
        (
            di(Some((0.0, 1.1)), Trv),
            di(c1, Def),
            di(Some((NEG_INF, -s2)), Trv),
            et(),
        ),
        (
            di(Some((0.01, 1.1)), Com),
            di(c1, Com),
            di(Some((-t, -s2)), Com),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c1, Def), et(), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c1, Dac),
            di(Some((0.0, big)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Def),
            di(c1, Com),
            di(Some((0.0, INF)), Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Trv),
            di(c1, Def),
            di(Some((NEG_INF, -s2)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        (
            di(Some((-2.0, INF)), Dac),
            di(c1, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((a, INF)), Trv),
        ),
        (
            di(Some((0.0, INF)), Def),
            di(c1, Com),
            di(Some((NEG_INF, 0.0)), Trv),
            et(),
        ),
        (
            di(Some((0.01, INF)), Def),
            di(c1, Def),
            di(Some((-t, 0.0)), Def),
            et(),
        ),
        (
            di(ENT, Dac),
            di(c1, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        // c = [-2.1, 0.0]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c2, Com),
            di(Some((0.0, big)), Dac),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Dac), di(c2, Com), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Dac), di(c2, Com), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Dac), di(c2, Com), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c2, Com),
            di(Some((-t, 0.0)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c2, Com), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c2, Com),
            di(Some((0.0, big)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c2, Com),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c2, Com),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(c2, Com), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(c2, Com), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Dac),
            di(c2, Com),
            di(Some((-t, 0.0)), Dac),
            et(),
        ),
        (di(ENT, Dac), di(c2, Com), di(ENT, Trv), et()),
        // c = [-2.1, 0.12]
        (
            di(Some((-2.0, -0.1)), Def),
            di(c3, Dac),
            di(Some((-onetwo, big)), Def),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Def), di(c3, Dac), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Def), di(c3, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Def), di(c3, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Def),
            di(c3, Dac),
            di(Some((-t, twelve)), Def),
            et(),
        ),
        (di(Some((0.0, 0.0)), Def), di(c3, Dac), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Def),
            di(c3, Dac),
            di(Some((-onetwo, big)), Def),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Def),
            di(c3, Dac),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Def),
            di(c3, Dac),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Def), di(c3, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Def), di(c3, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Def),
            di(c3, Dac),
            di(Some((-t, twelve)), Def),
            et(),
        ),
        (di(ENT, Def), di(c3, Dac), di(ENT, Trv), et()),
        // c = [0.0, 0.12]
        (
            di(Some((-2.0, -0.1)), Com),
            di(c4, Com),
            di(Some((-onetwo, 0.0)), Com),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Com), di(c4, Com), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Com), di(c4, Com), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Com), di(c4, Com), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Com),
            di(c4, Com),
            di(Some((0.0, twelve)), Com),
            et(),
        ),
        (di(Some((0.0, 0.0)), Com), di(c4, Com), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c4, Com),
            di(Some((-onetwo, 0.0)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c4, Com),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c4, Com),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(c4, Com), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(c4, Com), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Dac),
            di(c4, Com),
            di(Some((0.0, twelve)), Dac),
            et(),
        ),
        (di(ENT, Dac), di(c4, Com), di(ENT, Trv), et()),
        // c = [0.01, 0.12]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c5, Dac),
            di(Some((-onetwo, -sm8)), Dac),
            et(),
        ),
        (
            di(Some((-2.0, 0.0)), Dac),
            di(c5, Dac),
            di(Some((NEG_INF, -sm8)), Trv),
            et(),
        ),
        (
            di(Some((-2.0, 1.1)), Dac),
            di(c5, Dac),
            di(Some((NEG_INF, -sm8)), Trv),
            di(Some((lo7, INF)), Trv),
        ),
        (
            di(Some((0.0, 1.1)), Dac),
            di(c5, Dac),
            di(Some((lo7, INF)), Trv),
            et(),
        ),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c5, Dac),
            di(Some((lo7, twelve)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c5, Dac), et(), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c5, Dac),
            di(Some((-onetwo, 0.0)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c5, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c5, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((lo7, INF)), Trv),
        ),
        (
            di(Some((-2.0, INF)), Dac),
            di(c5, Dac),
            di(Some((NEG_INF, -sm8)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        (
            di(Some((0.0, INF)), Dac),
            di(c5, Dac),
            di(Some((0.0, INF)), Trv),
            et(),
        ),
        (
            di(Some((0.01, INF)), Dac),
            di(c5, Dac),
            di(Some((0.0, twelve)), Dac),
            et(),
        ),
        (
            di(ENT, Dac),
            di(c5, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        // c = [0.0, 0.0]
        (
            di(Some((-2.0, -0.1)), Com),
            di(c6, Com),
            di(Some((0.0, 0.0)), Com),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Com), di(c6, Com), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Com), di(c6, Com), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Com), di(c6, Com), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Com),
            di(c6, Com),
            di(Some((0.0, 0.0)), Com),
            et(),
        ),
        (di(Some((0.0, 0.0)), Com), di(c6, Com), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c6, Com),
            di(Some((0.0, 0.0)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c6, Com),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c6, Com),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(c6, Com), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(c6, Com), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Dac),
            di(c6, Com),
            di(Some((0.0, 0.0)), Dac),
            et(),
        ),
        (di(ENT, Dac), di(c6, Com), di(ENT, Trv), et()),
        // c = [-inf, -0.1]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c7, Dac),
            di(Some((a5, INF)), Dac),
            et(),
        ),
        (
            di(Some((-2.0, 0.0)), Dac),
            di(c7, Dac),
            di(Some((a5, INF)), Trv),
            et(),
        ),
        (
            di(Some((-2.0, 1.1)), Dac),
            di(c7, Dac),
            di(Some((NEG_INF, -s4)), Trv),
            di(Some((a5, INF)), Trv),
        ),
        (
            di(Some((0.0, 1.1)), Dac),
            di(c7, Dac),
            di(Some((NEG_INF, -s4)), Trv),
            et(),
        ),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c7, Dac),
            di(Some((NEG_INF, -s4)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c7, Dac), et(), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c7, Dac),
            di(Some((0.0, INF)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c7, Dac),
            di(Some((0.0, INF)), Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c7, Dac),
            di(Some((NEG_INF, -s4)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        (
            di(Some((-2.0, INF)), Dac),
            di(c7, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((a5, INF)), Trv),
        ),
        (
            di(Some((0.0, INF)), Dac),
            di(c7, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            et(),
        ),
        (
            di(Some((0.01, INF)), Dac),
            di(c7, Dac),
            di(Some((NEG_INF, 0.0)), Dac),
            et(),
        ),
        (
            di(ENT, Dac),
            di(c7, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        // c = [-inf, 0.0]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c8, Dac),
            di(Some((0.0, INF)), Dac),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Dac), di(c8, Dac), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Dac), di(c8, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Dac), di(c8, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c8, Dac),
            di(Some((NEG_INF, 0.0)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c8, Dac), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c8, Dac),
            di(Some((0.0, INF)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c8, Dac),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c8, Dac),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(c8, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(c8, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Dac),
            di(c8, Dac),
            di(Some((NEG_INF, 0.0)), Dac),
            et(),
        ),
        (di(ENT, Dac), di(c8, Dac), di(ENT, Trv), et()),
        // c = [-inf, 0.3]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c9, Dac),
            di(Some((-three, INF)), Dac),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Dac), di(c9, Dac), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Dac), di(c9, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Dac), di(c9, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c9, Dac),
            di(Some((NEG_INF, thirty)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c9, Dac), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c9, Dac),
            di(Some((-three, INF)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c9, Dac),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c9, Dac),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(c9, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(c9, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Dac),
            di(c9, Dac),
            di(Some((NEG_INF, thirty)), Dac),
            et(),
        ),
        (di(ENT, Dac), di(c9, Dac), di(ENT, Trv), et()),
        // c = [-0.21, inf]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c10, Dac),
            di(Some((NEG_INF, two1)), Dac),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Dac), di(c10, Dac), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Dac), di(c10, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Dac), di(c10, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c10, Dac),
            di(Some((-big, INF)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c10, Dac), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c10, Dac),
            di(Some((NEG_INF, two1)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c10, Dac),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c10, Dac),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(c10, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(c10, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Dac),
            di(c10, Dac),
            di(Some((-big, INF)), Dac),
            et(),
        ),
        (di(ENT, Dac), di(c10, Dac), di(ENT, Trv), et()),
        // c = [0.0, inf]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c11, Dac),
            di(Some((NEG_INF, 0.0)), Dac),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Dac), di(c11, Dac), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Dac), di(c11, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Dac), di(c11, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c11, Dac),
            di(Some((0.0, INF)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c11, Dac), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c11, Dac),
            di(Some((NEG_INF, 0.0)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c11, Dac),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c11, Dac),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(c11, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(c11, Dac), di(ENT, Trv), et()),
        (
            di(Some((0.01, INF)), Dac),
            di(c11, Dac),
            di(Some((0.0, INF)), Dac),
            et(),
        ),
        (di(ENT, Dac), di(c11, Dac), di(ENT, Trv), et()),
        // c = [0.04, inf]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(c12, Dac),
            di(Some((NEG_INF, -sm6)), Dac),
            et(),
        ),
        (
            di(Some((-2.0, 0.0)), Dac),
            di(c12, Dac),
            di(Some((NEG_INF, -sm6)), Trv),
            et(),
        ),
        (
            di(Some((-2.0, 1.1)), Dac),
            di(c12, Dac),
            di(Some((NEG_INF, -sm6)), Trv),
            di(Some((lo5, INF)), Trv),
        ),
        (
            di(Some((0.0, 1.1)), Dac),
            di(c12, Dac),
            di(Some((lo5, INF)), Trv),
            et(),
        ),
        (
            di(Some((0.01, 1.1)), Dac),
            di(c12, Dac),
            di(Some((lo5, INF)), Dac),
            et(),
        ),
        (di(Some((0.0, 0.0)), Dac), di(c12, Dac), et(), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(c12, Dac),
            di(Some((NEG_INF, 0.0)), Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(c12, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(c12, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((lo5, INF)), Trv),
        ),
        (
            di(Some((-2.0, INF)), Dac),
            di(c12, Dac),
            di(Some((NEG_INF, -sm6)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        (
            di(Some((0.0, INF)), Dac),
            di(c12, Dac),
            di(Some((0.0, INF)), Trv),
            et(),
        ),
        (
            di(Some((0.01, INF)), Dac),
            di(c12, Dac),
            di(Some((0.0, INF)), Dac),
            et(),
        ),
        (
            di(ENT, Dac),
            di(c12, Dac),
            di(Some((NEG_INF, 0.0)), Trv),
            di(Some((0.0, INF)), Trv),
        ),
        // c = [entire]
        (
            di(Some((-2.0, -0.1)), Dac),
            di(ENT, Dac),
            di(ENT, Dac),
            et(),
        ),
        (di(Some((-2.0, 0.0)), Dac), di(ENT, Dac), di(ENT, Trv), et()),
        (di(Some((-2.0, 1.1)), Dac), di(ENT, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, 1.1)), Dac), di(ENT, Dac), di(ENT, Trv), et()),
        (di(Some((0.01, 1.1)), Dac), di(ENT, Dac), di(ENT, Dac), et()),
        (di(Some((0.0, 0.0)), Dac), di(ENT, Dac), di(ENT, Trv), et()),
        (
            di(Some((NEG_INF, -0.1)), Dac),
            di(ENT, Dac),
            di(ENT, Dac),
            et(),
        ),
        (
            di(Some((NEG_INF, 0.0)), Dac),
            di(ENT, Dac),
            di(ENT, Trv),
            et(),
        ),
        (
            di(Some((NEG_INF, 1.1)), Dac),
            di(ENT, Dac),
            di(ENT, Trv),
            et(),
        ),
        (di(Some((-2.0, INF)), Dac), di(ENT, Dac), di(ENT, Trv), et()),
        (di(Some((0.0, INF)), Dac), di(ENT, Dac), di(ENT, Trv), et()),
        (di(Some((0.01, INF)), Dac), di(ENT, Dac), di(ENT, Dac), et()),
        (di(ENT, Dac), di(ENT, Dac), di(ENT, Trv), et()),
    ];
    for (b, c, p1w, p2w) in rows {
        let (p1, p2) = dib(b).mul_rev_to_pair(dib(c));
        eq_dpiece(p1, p1w);
        eq_dpiece(p2, p2w);
    }
}
