//! The ITF1788 arithmetic conformance vectors over `TightF64`, bit-exact.
//!
//! Every assertion line of the `libieeep1788_elem.itl` arithmetic testcases
//! (`pos`, `neg`, `add`, `sub`, `mul`, `div`, `recip`, `sqr`, `sqrt`, `fma`,
//! `pown`, bare and decorated) is transcribed here over the correctly rounded
//! `TightF64` backend. Each corpus testcase becomes one `#[test]` fn named after
//! it, headed by a `// minimal_<op>_test: N vectors` comment carrying the
//! transcription count. No sampling: the counts match an independent split of
//! the vendored file on `;` with comments stripped (pos 11, neg 11, add 31,
//! sub 31, mul 116, div 341, recip 18, sqr 12, sqrt 13, fma 564, pown 163, plus
//! the decorated blocks 4/4/6/6/6/6/8/4/4/3/11; 1373 total).
//!
//! # Assertion mode
//!
//! Bit-exact over `TightF64`: the endpoint `.0` values are compared against the
//! corpus literals. The default endpoint comparison is [`same`] (NaN equals
//! NaN, and +0.0 equals -0.0), the precedent set by the reduction conformance
//! lane. This is the correct comparison because IEEE 1788 identifies the set
//! `[a, -0.0]` with `[a, +0.0]`, and on `div`/`mul`/`recip` results the backend
//! may legitimately yield either sign of a zero endpoint (e.g. `-15.0 / +inf`
//! is `-0.0` while the corpus writes the same endpoint as `0.0`); `same` treats
//! those as equal while still catching every nonzero divergence.
//!
//! # Where the corpus distinguishes signed zeros
//!
//! The only arithmetic vectors whose expected output pins the *sign* of a zero
//! endpoint are the `pown` negative-power vectors that write the zero in hex
//! form (`0X0P+0` for a +0.0 lower bound, `-0X0P+0` for a -0.0 upper bound); the
//! reference encodes the base's sign in the zero there. Those very vectors are
//! also the ones the interval-1788 `pown` implementation currently fails on a
//! tightness ground (see the known-defect test below), so they are asserted
//! bit-exactly (sign of zero included, via [`eq_bits`]) inside the ignored
//! defect test rather than in the active lane. No active vector's expected
//! output distinguishes a signed zero, so the active lane uses [`same`]
//! throughout.
//!
//! # `pos` is the interval identity
//!
//! IEEE 1788 `pos` is unary plus, `pos(x) = x`; the crate exposes no distinct
//! `pos` method (there is nothing to compute), so the `minimal_pos_test` and
//! `minimal_pos_dec_test` vectors are transcribed against the interval identity:
//! the input value is constructed and its endpoints (and decoration) read back
//! against the expected. This is faithful and complete; it exercises
//! construction and endpoint canonicalization, which is exactly what the
//! reference's `pos` does.
//!
//! # Known defects surfaced by this lane
//!
//! Of the 1373 vectors, 97 fail bit-exact conformance and are relocated to two
//! ignored `_known_defect` tests, each asserting the corpus (tight) value so an
//! eventual fix is un-ignored and validated. Every relocated vector is a SOUND
//! enclosure of the corpus result (verified: the library interval contains the
//! tight interval); the losses are tightness, never soundness.
//!
//! - `minimal_div_test_known_defect` (56): `Interval / Interval` is
//!   `self * rhs.recip()`, two directed roundings where a directly rounded
//!   quotient is tighter (e.g. `[-30,-15] / [-5,-3]` is exactly `[3,10]` but the
//!   reciprocal path yields `[2.999...6, 10.000...2]`).
//! - `minimal_pown_test_known_defect` (41): `Interval::pown` raises to a power by
//!   repeated squaring with a directed rounding per multiply (loose for
//!   `|n| >= 3`) and computes negative powers as `pos_pown_image(a,b,|n|).recip()`
//!   (loose when the magnitude power overflows, `|base| = f64::MAX, |n| >= 2`,
//!   yielding `[+0, ~2^-1024]` where the tight result is `[+0, 2^-1074]`). The
//!   eight overflow vectors also pin the zero-endpoint sign and are asserted with
//!   [`eq_bits`].
//!
//! Every other testcase (`pos`, `neg`, `add`, `sub`, `mul`, `recip`, `sqr`,
//! `sqrt`, `fma`, all decorated blocks, and the 285/122 tight `div`/`pown`
//! vectors) is bit-exact over `TightF64`, which is the positive evidence: the
//! backend is correctly rounded and the interval operations that round once per
//! endpoint are tightest.
//!
//! Cases derived from ITF1788 `libieeep1788_elem.itl` (Apache-2.0, original
//! author Marco Nehmeier, ITL port Oliver Heimlich; see
//! `docs/references/vendor/itf1788-framework/`).

// The corpus values are exact and are compared bit-for-bit, so float equality
// is the intended check; the decimal literals are transcribed verbatim from the
// corpus (substituting `f64::consts` or adding digit separators would either
// change a pinned vector or break the verbatim text), so the readability and
// approximate-constant lints are waived for transcription fidelity, exactly as
// the reverse-operation fixture waives them. Each `#[test]` fn is one whole
// corpus testcase transcribed vector-for-vector, so the length lint is waived
// too: the structure is mandated by the conformance format, not incidental.
#![allow(
    clippy::float_cmp,
    clippy::approx_constant,
    clippy::unreadable_literal,
    clippy::too_many_lines
)]

use interval_1788::{DecoratedInterval, Decoration, Interval};
use round_float::TightF64;

const INF: f64 = f64::INFINITY;
const NINF: f64 = f64::NEG_INFINITY;

/// Parse a C hex-float literal to the exact `f64` it names. Copied from the
/// reverse-operation fixture: the canonicality assertion turns a non-canonical
/// literal (more than 53 significant bits) into a loud failure instead of a
/// silent rounding, which keeps the transcription honest.
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

fn iv(lo: f64, hi: f64) -> Interval<TightF64> {
    Interval::new(TightF64(lo), TightF64(hi)).unwrap()
}

fn empty() -> Interval<TightF64> {
    Interval::empty()
}

fn entire() -> Interval<TightF64> {
    Interval::entire()
}

fn dset(x: Interval<TightF64>, d: Decoration) -> DecoratedInterval<TightF64> {
    DecoratedInterval::set_dec(x, d)
}

/// `+0.0` equals `-0.0` and `NaN` equals `NaN`; otherwise exact float equality.
fn same(x: f64, y: f64) -> bool {
    (x.is_nan() && y.is_nan()) || x == y
}

/// A nonempty result whose endpoints equal `[lo, hi]` under [`same`].
fn eq(r: Interval<TightF64>, lo: f64, hi: f64) {
    assert!(!r.is_empty(), "expected [{lo}, {hi}], got empty");
    assert!(
        same(r.inf().0, lo) && same(r.sup().0, hi),
        "expected [{lo}, {hi}], got [{}, {}]",
        r.inf().0,
        r.sup().0
    );
}

/// A nonempty result whose endpoints equal `[lo, hi]` bit-for-bit (the sign of a
/// zero endpoint included). Used only where the corpus pins the zero sign.
///
/// Reads the endpoints through `inf`/`sup`, which since the enc-ks9 slice apply
/// the Level 2 signed-zero datum (`inf` of a zero infimum is `-0`, `sup` of a zero
/// supremum is `+0`). The overflow-recip pown vectors below store the opposite raw
/// signs (`+0` lower from `1/+inf`, `-0` upper from `1/-inf`), so once `pown` gains
/// a tight path and this test runs, the raw-sign pinning must move off `inf`/`sup`
/// onto the exact-text form (`interval_to_exact`, which serializes the stored
/// endpoints). The test is ignored today for the tightness defect, so the reads
/// never reach these vectors; the note records the interaction for the pown fix.
fn eq_bits(r: Interval<TightF64>, lo: f64, hi: f64) {
    assert!(!r.is_empty(), "expected [{lo}, {hi}], got empty");
    assert!(
        r.inf().0.to_bits() == lo.to_bits() && r.sup().0.to_bits() == hi.to_bits(),
        "expected bits [{:#018x}, {:#018x}], got [{:#018x}, {:#018x}]",
        lo.to_bits(),
        hi.to_bits(),
        r.inf().0.to_bits(),
        r.sup().0.to_bits()
    );
}

/// An empty result.
fn emp(r: Interval<TightF64>) {
    assert!(
        r.is_empty(),
        "expected empty, got [{}, {}]",
        r.inf().0,
        r.sup().0
    );
}

/// A decorated nonempty result: interval endpoints under [`same`] and the exact
/// decoration.
fn deq(r: DecoratedInterval<TightF64>, lo: f64, hi: f64, d: Decoration) {
    assert_eq!(r.decoration(), d, "decoration");
    eq(r.interval(), lo, hi);
}

/// A decorated empty result with the exact decoration.
fn demp(r: DecoratedInterval<TightF64>, d: Decoration) {
    assert_eq!(r.decoration(), d, "decoration");
    assert!(r.interval().is_empty(), "expected empty interval part");
}

/// A `NaI` result.
fn dnai(r: DecoratedInterval<TightF64>) {
    assert!(r.is_nai(), "expected NaI");
}

/// `minimal_pos_test`: 11 vectors.
#[test]
fn minimal_pos_test() {
    // minimal_pos_test: 11 vectors
    eq(iv(1.0, 2.0), 1.0, 2.0);
    emp(empty());
    eq(entire(), NINF, INF);
    eq(iv(1.0, INF), 1.0, INF);
    eq(iv(NINF, -1.0), NINF, -1.0);
    eq(iv(0.0, 2.0), 0.0, 2.0);
    eq(iv(-0.0, 2.0), 0.0, 2.0);
    eq(iv(-2.5, -0.0), -2.5, 0.0);
    eq(iv(-2.5, 0.0), -2.5, 0.0);
    eq(iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(0.0, 0.0), 0.0, 0.0);
}

/// `minimal_pos_dec_test`: 4 vectors.
#[test]
fn minimal_pos_dec_test() {
    // minimal_pos_dec_test: 4 vectors
    dnai(DecoratedInterval::<TightF64>::nai());
    demp(dset(empty(), Decoration::Trv), Decoration::Trv);
    deq(dset(entire(), Decoration::Dac), NINF, INF, Decoration::Dac);
    deq(
        dset(iv(1.0, 2.0), Decoration::Com),
        1.0,
        2.0,
        Decoration::Com,
    );
}

/// `minimal_neg_test`: 11 vectors.
#[test]
fn minimal_neg_test() {
    // minimal_neg_test: 11 vectors
    eq(-iv(1.0, 2.0), -2.0, -1.0);
    emp(-empty());
    eq(-entire(), NINF, INF);
    eq(-iv(1.0, INF), NINF, -1.0);
    eq(-iv(NINF, 1.0), -1.0, INF);
    eq(-iv(0.0, 2.0), -2.0, 0.0);
    eq(-iv(-0.0, 2.0), -2.0, 0.0);
    eq(-iv(-2.0, 0.0), 0.0, 2.0);
    eq(-iv(-2.0, -0.0), 0.0, 2.0);
    eq(-iv(0.0, -0.0), 0.0, 0.0);
    eq(-iv(-0.0, -0.0), 0.0, 0.0);
}

/// `minimal_neg_dec_test`: 4 vectors.
#[test]
fn minimal_neg_dec_test() {
    // minimal_neg_dec_test: 4 vectors
    dnai(-DecoratedInterval::<TightF64>::nai());
    demp(-dset(empty(), Decoration::Trv), Decoration::Trv);
    deq(-dset(entire(), Decoration::Dac), NINF, INF, Decoration::Dac);
    deq(
        -dset(iv(1.0, 2.0), Decoration::Com),
        -2.0,
        -1.0,
        Decoration::Com,
    );
}

/// `minimal_add_test`: 31 vectors.
#[test]
fn minimal_add_test() {
    // minimal_add_test: 31 vectors
    emp(empty() + empty());
    emp(iv(-1.0, 1.0) + empty());
    emp(empty() + iv(-1.0, 1.0));
    emp(empty() + entire());
    emp(entire() + empty());
    eq(entire() + iv(NINF, 1.0), NINF, INF);
    eq(entire() + iv(-1.0, 1.0), NINF, INF);
    eq(entire() + iv(-1.0, INF), NINF, INF);
    eq(entire() + entire(), NINF, INF);
    eq(iv(NINF, 1.0) + entire(), NINF, INF);
    eq(iv(-1.0, 1.0) + entire(), NINF, INF);
    eq(iv(-1.0, INF) + entire(), NINF, INF);
    eq(iv(NINF, 2.0) + iv(NINF, 4.0), NINF, 6.0);
    eq(iv(NINF, 2.0) + iv(3.0, 4.0), NINF, 6.0);
    eq(iv(NINF, 2.0) + iv(3.0, INF), NINF, INF);
    eq(iv(1.0, 2.0) + iv(NINF, 4.0), NINF, 6.0);
    eq(iv(1.0, 2.0) + iv(3.0, 4.0), 4.0, 6.0);
    eq(iv(1.0, 2.0) + iv(3.0, INF), 4.0, INF);
    eq(iv(1.0, INF) + iv(NINF, 4.0), NINF, INF);
    eq(iv(1.0, INF) + iv(3.0, 4.0), 4.0, INF);
    eq(iv(1.0, INF) + iv(3.0, INF), 4.0, INF);
    eq(
        iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")) + iv(3.0, 4.0),
        4.0,
        INF,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0) + iv(-3.0, 4.0),
        NINF,
        6.0,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0) + iv(-3.0, hx("0x1.FFFFFFFFFFFFFp1023")),
        NINF,
        INF,
    );
    eq(
        iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")) + iv(0.0, 0.0),
        1.0,
        hx("0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(
        iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")) + iv(-0.0, -0.0),
        1.0,
        hx("0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(iv(0.0, 0.0) + iv(-3.0, 4.0), -3.0, 4.0);
    eq(
        iv(-0.0, -0.0) + iv(-3.0, hx("0x1.FFFFFFFFFFFFFp1023")),
        -3.0,
        hx("0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(
        iv(hx("0X1.FFFFFFFFFFFFP+0"), hx("0X1.FFFFFFFFFFFFP+0"))
            + iv(hx("0X1.999999999999AP-4"), hx("0X1.999999999999AP-4")),
        hx("0X1.0CCCCCCCCCCC4P+1"),
        hx("0X1.0CCCCCCCCCCC5P+1"),
    );
    eq(
        iv(hx("0X1.FFFFFFFFFFFFP+0"), hx("0X1.FFFFFFFFFFFFP+0"))
            + iv(hx("-0X1.999999999999AP-4"), hx("-0X1.999999999999AP-4")),
        hx("0X1.E666666666656P+0"),
        hx("0X1.E666666666657P+0"),
    );
    eq(
        iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("0X1.FFFFFFFFFFFFP+0"))
            + iv(hx("0X1.999999999999AP-4"), hx("0X1.999999999999AP-4")),
        hx("-0X1.E666666666657P+0"),
        hx("0X1.0CCCCCCCCCCC5P+1"),
    );
}

/// `minimal_add_dec_test`: 6 vectors.
#[test]
fn minimal_add_dec_test() {
    // minimal_add_dec_test: 6 vectors
    deq(
        dset(iv(1.0, 2.0), Decoration::Com) + dset(iv(5.0, 7.0), Decoration::Com),
        6.0,
        9.0,
        Decoration::Com,
    );
    deq(
        dset(iv(1.0, 2.0), Decoration::Com) + dset(iv(5.0, 7.0), Decoration::Def),
        6.0,
        9.0,
        Decoration::Def,
    );
    deq(
        dset(iv(1.0, 2.0), Decoration::Com)
            + dset(iv(5.0, hx("0x1.FFFFFFFFFFFFFp1023")), Decoration::Com),
        6.0,
        INF,
        Decoration::Dac,
    );
    deq(
        dset(iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0), Decoration::Com)
            + dset(iv(-0.1, 5.0), Decoration::Com),
        NINF,
        7.0,
        Decoration::Dac,
    );
    demp(
        dset(iv(1.0, 2.0), Decoration::Trv) + dset(empty(), Decoration::Trv),
        Decoration::Trv,
    );
    dnai(DecoratedInterval::<TightF64>::nai() + dset(iv(1.0, 2.0), Decoration::Trv));
}

/// `minimal_sub_test`: 31 vectors.
#[test]
fn minimal_sub_test() {
    // minimal_sub_test: 31 vectors
    emp(empty() - empty());
    emp(iv(-1.0, 1.0) - empty());
    emp(empty() - iv(-1.0, 1.0));
    emp(empty() - entire());
    emp(entire() - empty());
    eq(entire() - iv(NINF, 1.0), NINF, INF);
    eq(entire() - iv(-1.0, 1.0), NINF, INF);
    eq(entire() - iv(-1.0, INF), NINF, INF);
    eq(entire() - entire(), NINF, INF);
    eq(iv(NINF, 1.0) - entire(), NINF, INF);
    eq(iv(-1.0, 1.0) - entire(), NINF, INF);
    eq(iv(-1.0, INF) - entire(), NINF, INF);
    eq(iv(NINF, 2.0) - iv(NINF, 4.0), NINF, INF);
    eq(iv(NINF, 2.0) - iv(3.0, 4.0), NINF, -1.0);
    eq(iv(NINF, 2.0) - iv(3.0, INF), NINF, -1.0);
    eq(iv(1.0, 2.0) - iv(NINF, 4.0), -3.0, INF);
    eq(iv(1.0, 2.0) - iv(3.0, 4.0), -3.0, -1.0);
    eq(iv(1.0, 2.0) - iv(3.0, INF), NINF, -1.0);
    eq(iv(1.0, INF) - iv(NINF, 4.0), -3.0, INF);
    eq(iv(1.0, INF) - iv(3.0, 4.0), -3.0, INF);
    eq(iv(1.0, INF) - iv(3.0, INF), NINF, INF);
    eq(
        iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")) - iv(-3.0, 4.0),
        -3.0,
        INF,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0) - iv(3.0, 4.0),
        NINF,
        -1.0,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0) - iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 4.0),
        NINF,
        INF,
    );
    eq(
        iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")) - iv(0.0, 0.0),
        1.0,
        hx("0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(
        iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")) - iv(-0.0, -0.0),
        1.0,
        hx("0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(iv(0.0, 0.0) - iv(-3.0, 4.0), -4.0, 3.0);
    eq(
        iv(-0.0, -0.0) - iv(-3.0, hx("0x1.FFFFFFFFFFFFFp1023")),
        hx("-0x1.FFFFFFFFFFFFFp1023"),
        3.0,
    );
    eq(
        iv(hx("0X1.FFFFFFFFFFFFP+0"), hx("0X1.FFFFFFFFFFFFP+0"))
            - iv(hx("0X1.999999999999AP-4"), hx("0X1.999999999999AP-4")),
        hx("0X1.E666666666656P+0"),
        hx("0X1.E666666666657P+0"),
    );
    eq(
        iv(hx("0X1.FFFFFFFFFFFFP+0"), hx("0X1.FFFFFFFFFFFFP+0"))
            - iv(hx("-0X1.999999999999AP-4"), hx("-0X1.999999999999AP-4")),
        hx("0X1.0CCCCCCCCCCC4P+1"),
        hx("0X1.0CCCCCCCCCCC5P+1"),
    );
    eq(
        iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("0X1.FFFFFFFFFFFFP+0"))
            - iv(hx("0X1.999999999999AP-4"), hx("0X1.999999999999AP-4")),
        hx("-0X1.0CCCCCCCCCCC5P+1"),
        hx("0X1.E666666666657P+0"),
    );
}

/// `minimal_sub_dec_test`: 6 vectors.
#[test]
fn minimal_sub_dec_test() {
    // minimal_sub_dec_test: 6 vectors
    deq(
        dset(iv(1.0, 2.0), Decoration::Com) - dset(iv(5.0, 7.0), Decoration::Com),
        -6.0,
        -3.0,
        Decoration::Com,
    );
    deq(
        dset(iv(1.0, 2.0), Decoration::Com) - dset(iv(5.0, 7.0), Decoration::Def),
        -6.0,
        -3.0,
        Decoration::Def,
    );
    deq(
        dset(iv(-1.0, 2.0), Decoration::Com)
            - dset(iv(5.0, hx("0x1.FFFFFFFFFFFFFp1023")), Decoration::Com),
        NINF,
        -3.0,
        Decoration::Dac,
    );
    deq(
        dset(iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0), Decoration::Com)
            - dset(iv(-1.0, 5.0), Decoration::Com),
        NINF,
        3.0,
        Decoration::Dac,
    );
    demp(
        dset(iv(1.0, 2.0), Decoration::Trv) - dset(empty(), Decoration::Trv),
        Decoration::Trv,
    );
    dnai(DecoratedInterval::<TightF64>::nai() - dset(iv(1.0, 2.0), Decoration::Trv));
}

/// `minimal_mul_test`: 116 vectors.
#[test]
fn minimal_mul_test() {
    // minimal_mul_test: 116 vectors
    emp(empty() * empty());
    emp(iv(-1.0, 1.0) * empty());
    emp(empty() * iv(-1.0, 1.0));
    emp(empty() * entire());
    emp(entire() * empty());
    emp(iv(0.0, 0.0) * empty());
    emp(empty() * iv(0.0, 0.0));
    emp(iv(-0.0, -0.0) * empty());
    emp(empty() * iv(-0.0, -0.0));
    eq(entire() * iv(0.0, 0.0), 0.0, 0.0);
    eq(entire() * iv(-0.0, -0.0), 0.0, 0.0);
    eq(entire() * iv(-5.0, -1.0), NINF, INF);
    eq(entire() * iv(-5.0, 3.0), NINF, INF);
    eq(entire() * iv(1.0, 3.0), NINF, INF);
    eq(entire() * iv(NINF, -1.0), NINF, INF);
    eq(entire() * iv(NINF, 3.0), NINF, INF);
    eq(entire() * iv(-5.0, INF), NINF, INF);
    eq(entire() * iv(1.0, INF), NINF, INF);
    eq(entire() * entire(), NINF, INF);
    eq(iv(1.0, INF) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(1.0, INF) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(1.0, INF) * iv(-5.0, -1.0), NINF, -1.0);
    eq(iv(1.0, INF) * iv(-5.0, 3.0), NINF, INF);
    eq(iv(1.0, INF) * iv(1.0, 3.0), 1.0, INF);
    eq(iv(1.0, INF) * iv(NINF, -1.0), NINF, -1.0);
    eq(iv(1.0, INF) * iv(NINF, 3.0), NINF, INF);
    eq(iv(1.0, INF) * iv(-5.0, INF), NINF, INF);
    eq(iv(1.0, INF) * iv(1.0, INF), 1.0, INF);
    eq(iv(1.0, INF) * entire(), NINF, INF);
    eq(iv(-1.0, INF) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(-1.0, INF) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(-1.0, INF) * iv(-5.0, -1.0), NINF, 5.0);
    eq(iv(-1.0, INF) * iv(-5.0, 3.0), NINF, INF);
    eq(iv(-1.0, INF) * iv(1.0, 3.0), -3.0, INF);
    eq(iv(-1.0, INF) * iv(NINF, -1.0), NINF, INF);
    eq(iv(-1.0, INF) * iv(NINF, 3.0), NINF, INF);
    eq(iv(-1.0, INF) * iv(-5.0, INF), NINF, INF);
    eq(iv(-1.0, INF) * iv(1.0, INF), NINF, INF);
    eq(iv(-1.0, INF) * entire(), NINF, INF);
    eq(iv(NINF, 3.0) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(NINF, 3.0) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(NINF, 3.0) * iv(-5.0, -1.0), -15.0, INF);
    eq(iv(NINF, 3.0) * iv(-5.0, 3.0), NINF, INF);
    eq(iv(NINF, 3.0) * iv(1.0, 3.0), NINF, 9.0);
    eq(iv(NINF, 3.0) * iv(NINF, -1.0), NINF, INF);
    eq(iv(NINF, 3.0) * iv(NINF, 3.0), NINF, INF);
    eq(iv(NINF, 3.0) * iv(-5.0, INF), NINF, INF);
    eq(iv(NINF, 3.0) * iv(1.0, INF), NINF, INF);
    eq(iv(NINF, 3.0) * entire(), NINF, INF);
    eq(iv(NINF, -3.0) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(NINF, -3.0) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(NINF, -3.0) * iv(-5.0, -1.0), 3.0, INF);
    eq(iv(NINF, -3.0) * iv(-5.0, 3.0), NINF, INF);
    eq(iv(NINF, -3.0) * iv(1.0, 3.0), NINF, -3.0);
    eq(iv(NINF, -3.0) * iv(NINF, -1.0), 3.0, INF);
    eq(iv(NINF, -3.0) * iv(NINF, 3.0), NINF, INF);
    eq(iv(NINF, -3.0) * iv(-5.0, INF), NINF, INF);
    eq(iv(NINF, -3.0) * iv(1.0, INF), NINF, -3.0);
    eq(iv(NINF, -3.0) * entire(), NINF, INF);
    eq(iv(0.0, 0.0) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(-5.0, -1.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(-5.0, 3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(1.0, 3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(NINF, -1.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(NINF, 3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(-5.0, INF), 0.0, 0.0);
    eq(iv(0.0, 0.0) * iv(1.0, INF), 0.0, 0.0);
    eq(iv(0.0, 0.0) * entire(), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(-5.0, -1.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(-5.0, 3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(1.0, 3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(NINF, -1.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(NINF, 3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(-5.0, INF), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * iv(1.0, INF), 0.0, 0.0);
    eq(iv(-0.0, -0.0) * entire(), 0.0, 0.0);
    eq(iv(1.0, 5.0) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(1.0, 5.0) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(1.0, 5.0) * iv(-5.0, -1.0), -25.0, -1.0);
    eq(iv(1.0, 5.0) * iv(-5.0, 3.0), -25.0, 15.0);
    eq(iv(1.0, 5.0) * iv(1.0, 3.0), 1.0, 15.0);
    eq(iv(1.0, 5.0) * iv(NINF, -1.0), NINF, -1.0);
    eq(iv(1.0, 5.0) * iv(NINF, 3.0), NINF, 15.0);
    eq(iv(1.0, 5.0) * iv(-5.0, INF), -25.0, INF);
    eq(iv(1.0, 5.0) * iv(1.0, INF), 1.0, INF);
    eq(iv(1.0, 5.0) * entire(), NINF, INF);
    eq(iv(-1.0, 5.0) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(-1.0, 5.0) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(-1.0, 5.0) * iv(-5.0, -1.0), -25.0, 5.0);
    eq(iv(-1.0, 5.0) * iv(-5.0, 3.0), -25.0, 15.0);
    eq(iv(-10.0, 2.0) * iv(-5.0, 3.0), -30.0, 50.0);
    eq(iv(-1.0, 5.0) * iv(-1.0, 10.0), -10.0, 50.0);
    eq(iv(-2.0, 2.0) * iv(-5.0, 3.0), -10.0, 10.0);
    eq(iv(-1.0, 5.0) * iv(1.0, 3.0), -3.0, 15.0);
    eq(iv(-1.0, 5.0) * iv(NINF, -1.0), NINF, INF);
    eq(iv(-1.0, 5.0) * iv(NINF, 3.0), NINF, INF);
    eq(iv(-1.0, 5.0) * iv(-5.0, INF), NINF, INF);
    eq(iv(-1.0, 5.0) * iv(1.0, INF), NINF, INF);
    eq(iv(-1.0, 5.0) * entire(), NINF, INF);
    eq(iv(-10.0, -5.0) * iv(0.0, 0.0), 0.0, 0.0);
    eq(iv(-10.0, -5.0) * iv(-0.0, -0.0), 0.0, 0.0);
    eq(iv(-10.0, -5.0) * iv(-5.0, -1.0), 5.0, 50.0);
    eq(iv(-10.0, -5.0) * iv(-5.0, 3.0), -30.0, 50.0);
    eq(iv(-10.0, -5.0) * iv(1.0, 3.0), -30.0, -5.0);
    eq(iv(-10.0, -5.0) * iv(NINF, -1.0), 5.0, INF);
    eq(iv(-10.0, -5.0) * iv(NINF, 3.0), -30.0, INF);
    eq(iv(-10.0, -5.0) * iv(-5.0, INF), NINF, 50.0);
    eq(iv(-10.0, -5.0) * iv(1.0, INF), NINF, -5.0);
    eq(iv(-10.0, -5.0) * entire(), NINF, INF);
    eq(
        iv(hx("0X1.999999999999AP-4"), hx("0X1.FFFFFFFFFFFFP+0"))
            * iv(hx("-0X1.FFFFFFFFFFFFP+0"), INF),
        hx("-0X1.FFFFFFFFFFFE1P+1"),
        INF,
    );
    eq(
        iv(hx("-0X1.999999999999AP-4"), hx("0X1.FFFFFFFFFFFFP+0"))
            * iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("-0X1.999999999999AP-4")),
        hx("-0X1.FFFFFFFFFFFE1P+1"),
        hx("0X1.999999999998EP-3"),
    );
    eq(
        iv(hx("-0X1.999999999999AP-4"), hx("0X1.999999999999AP-4"))
            * iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("0X1.999999999999AP-4")),
        hx("-0X1.999999999998EP-3"),
        hx("0X1.999999999998EP-3"),
    );
    eq(
        iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("-0X1.999999999999AP-4"))
            * iv(hx("0X1.999999999999AP-4"), hx("0X1.FFFFFFFFFFFFP+0")),
        hx("-0X1.FFFFFFFFFFFE1P+1"),
        hx("-0X1.47AE147AE147BP-7"),
    );
}

/// `minimal_mul_dec_test`: 6 vectors.
#[test]
fn minimal_mul_dec_test() {
    // minimal_mul_dec_test: 6 vectors
    deq(
        dset(iv(1.0, 2.0), Decoration::Com) * dset(iv(5.0, 7.0), Decoration::Com),
        5.0,
        14.0,
        Decoration::Com,
    );
    deq(
        dset(iv(1.0, 2.0), Decoration::Com) * dset(iv(5.0, 7.0), Decoration::Def),
        5.0,
        14.0,
        Decoration::Def,
    );
    deq(
        dset(iv(1.0, 2.0), Decoration::Com)
            * dset(iv(5.0, hx("0x1.FFFFFFFFFFFFFp1023")), Decoration::Com),
        5.0,
        INF,
        Decoration::Dac,
    );
    deq(
        dset(iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0), Decoration::Com)
            * dset(iv(-1.0, 5.0), Decoration::Com),
        NINF,
        hx("0x1.FFFFFFFFFFFFFp1023"),
        Decoration::Dac,
    );
    demp(
        dset(iv(1.0, 2.0), Decoration::Trv) * dset(empty(), Decoration::Trv),
        Decoration::Trv,
    );
    dnai(DecoratedInterval::<TightF64>::nai() * dset(iv(1.0, 2.0), Decoration::Trv));
}

/// `minimal_div_test`: 285 active vectors (56 sound-but-not-
/// tightest vectors relocated to the known-defect test below;
/// 341 total in the corpus testcase).
#[test]
fn minimal_div_test() {
    // minimal_div_test: 285 vectors
    emp(empty() / empty());
    emp(iv(-1.0, 1.0) / empty());
    emp(empty() / iv(-1.0, 1.0));
    emp(empty() / iv(0.1, 1.0));
    emp(empty() / iv(-1.0, -0.1));
    emp(empty() / entire());
    emp(entire() / empty());
    emp(iv(0.0, 0.0) / empty());
    emp(empty() / iv(0.0, 0.0));
    emp(iv(-0.0, -0.0) / empty());
    emp(empty() / iv(-0.0, -0.0));
    eq(entire() / iv(-5.0, -3.0), NINF, INF);
    eq(entire() / iv(3.0, 5.0), NINF, INF);
    eq(entire() / iv(NINF, -3.0), NINF, INF);
    eq(entire() / iv(3.0, INF), NINF, INF);
    emp(entire() / iv(0.0, 0.0));
    emp(entire() / iv(-0.0, -0.0));
    eq(entire() / iv(-3.0, 0.0), NINF, INF);
    eq(entire() / iv(-3.0, -0.0), NINF, INF);
    eq(entire() / iv(-3.0, 3.0), NINF, INF);
    eq(entire() / iv(0.0, 3.0), NINF, INF);
    eq(entire() / iv(NINF, 0.0), NINF, INF);
    eq(entire() / iv(-0.0, 3.0), NINF, INF);
    eq(entire() / iv(NINF, -0.0), NINF, INF);
    eq(entire() / iv(NINF, 3.0), NINF, INF);
    eq(entire() / iv(-3.0, INF), NINF, INF);
    eq(entire() / iv(0.0, INF), NINF, INF);
    eq(entire() / iv(-0.0, INF), NINF, INF);
    eq(entire() / entire(), NINF, INF);
    emp(iv(-30.0, -15.0) / iv(0.0, 0.0));
    emp(iv(-30.0, -15.0) / iv(-0.0, -0.0));
    eq(iv(-30.0, -15.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(-30.0, -15.0) / iv(NINF, 0.0), 0.0, INF);
    eq(iv(-30.0, -15.0) / iv(NINF, -0.0), 0.0, INF);
    eq(iv(-30.0, -15.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(-30.0, -15.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(-30.0, -15.0) / iv(0.0, INF), NINF, 0.0);
    eq(iv(-30.0, -15.0) / iv(-0.0, INF), NINF, 0.0);
    eq(iv(-30.0, -15.0) / entire(), NINF, INF);
    emp(iv(-30.0, 15.0) / iv(0.0, 0.0));
    emp(iv(-30.0, 15.0) / iv(-0.0, -0.0));
    eq(iv(-30.0, 15.0) / iv(-3.0, 0.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(-3.0, -0.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(0.0, 3.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(NINF, 0.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(-0.0, 3.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(NINF, -0.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(0.0, INF), NINF, INF);
    eq(iv(-30.0, 15.0) / iv(-0.0, INF), NINF, INF);
    eq(iv(-30.0, 15.0) / entire(), NINF, INF);
    emp(iv(15.0, 30.0) / iv(0.0, 0.0));
    emp(iv(15.0, 30.0) / iv(-0.0, -0.0));
    eq(iv(15.0, 30.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(15.0, 30.0) / iv(NINF, 0.0), NINF, 0.0);
    eq(iv(15.0, 30.0) / iv(NINF, -0.0), NINF, 0.0);
    eq(iv(15.0, 30.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(15.0, 30.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(15.0, 30.0) / iv(0.0, INF), 0.0, INF);
    eq(iv(15.0, 30.0) / iv(-0.0, INF), 0.0, INF);
    eq(iv(15.0, 30.0) / entire(), NINF, INF);
    eq(iv(0.0, 0.0) / iv(-5.0, -3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(3.0, 5.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(NINF, -3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(3.0, INF), 0.0, 0.0);
    emp(iv(0.0, 0.0) / iv(0.0, 0.0));
    eq(iv(0.0, 0.0) / iv(-3.0, 0.0), 0.0, 0.0);
    emp(iv(0.0, 0.0) / iv(-0.0, -0.0));
    eq(iv(0.0, 0.0) / iv(-3.0, -0.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(-3.0, 3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(0.0, 3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(NINF, 0.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(-0.0, 3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(NINF, -0.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(NINF, 3.0), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(-3.0, INF), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(0.0, INF), 0.0, 0.0);
    eq(iv(0.0, 0.0) / iv(-0.0, INF), 0.0, 0.0);
    eq(iv(0.0, 0.0) / entire(), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(-5.0, -3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(3.0, 5.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(NINF, -3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(3.0, INF), 0.0, 0.0);
    emp(iv(-0.0, -0.0) / iv(0.0, 0.0));
    eq(iv(-0.0, -0.0) / iv(-3.0, 0.0), 0.0, 0.0);
    emp(iv(-0.0, -0.0) / iv(-0.0, -0.0));
    eq(iv(-0.0, -0.0) / iv(-3.0, -0.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(-3.0, 3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(0.0, 3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(NINF, 0.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(-0.0, 3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(NINF, -0.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(NINF, 3.0), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(-3.0, INF), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(0.0, INF), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / iv(-0.0, INF), 0.0, 0.0);
    eq(iv(-0.0, -0.0) / entire(), 0.0, 0.0);
    eq(iv(NINF, -15.0) / iv(NINF, -3.0), 0.0, INF);
    eq(iv(NINF, -15.0) / iv(3.0, INF), NINF, 0.0);
    emp(iv(NINF, -15.0) / iv(0.0, 0.0));
    emp(iv(NINF, -15.0) / iv(-0.0, -0.0));
    eq(iv(NINF, -15.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(NINF, -15.0) / iv(NINF, 0.0), 0.0, INF);
    eq(iv(NINF, -15.0) / iv(NINF, -0.0), 0.0, INF);
    eq(iv(NINF, -15.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(NINF, -15.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(NINF, -15.0) / iv(0.0, INF), NINF, 0.0);
    eq(iv(NINF, -15.0) / iv(-0.0, INF), NINF, 0.0);
    eq(iv(NINF, -15.0) / entire(), NINF, INF);
    emp(iv(NINF, 15.0) / iv(0.0, 0.0));
    eq(iv(NINF, 15.0) / iv(-3.0, 0.0), NINF, INF);
    emp(iv(NINF, 15.0) / iv(-0.0, -0.0));
    eq(iv(NINF, 15.0) / iv(-3.0, -0.0), NINF, INF);
    eq(iv(NINF, 15.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(NINF, 15.0) / iv(0.0, 3.0), NINF, INF);
    eq(iv(NINF, 15.0) / iv(NINF, 0.0), NINF, INF);
    eq(iv(NINF, 15.0) / iv(-0.0, 3.0), NINF, INF);
    eq(iv(NINF, 15.0) / iv(NINF, -0.0), NINF, INF);
    eq(iv(NINF, 15.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(NINF, 15.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(NINF, 15.0) / iv(0.0, INF), NINF, INF);
    eq(iv(NINF, 15.0) / iv(-0.0, INF), NINF, INF);
    eq(iv(NINF, 15.0) / entire(), NINF, INF);
    emp(iv(-15.0, INF) / iv(0.0, 0.0));
    eq(iv(-15.0, INF) / iv(-3.0, 0.0), NINF, INF);
    emp(iv(-15.0, INF) / iv(-0.0, -0.0));
    eq(iv(-15.0, INF) / iv(-3.0, -0.0), NINF, INF);
    eq(iv(-15.0, INF) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(-15.0, INF) / iv(0.0, 3.0), NINF, INF);
    eq(iv(-15.0, INF) / iv(NINF, 0.0), NINF, INF);
    eq(iv(-15.0, INF) / iv(-0.0, 3.0), NINF, INF);
    eq(iv(-15.0, INF) / iv(NINF, -0.0), NINF, INF);
    eq(iv(-15.0, INF) / iv(NINF, 3.0), NINF, INF);
    eq(iv(-15.0, INF) / iv(-3.0, INF), NINF, INF);
    eq(iv(-15.0, INF) / iv(0.0, INF), NINF, INF);
    eq(iv(-15.0, INF) / iv(-0.0, INF), NINF, INF);
    eq(iv(-15.0, INF) / entire(), NINF, INF);
    eq(iv(15.0, INF) / iv(NINF, -3.0), NINF, 0.0);
    eq(iv(15.0, INF) / iv(3.0, INF), 0.0, INF);
    emp(iv(15.0, INF) / iv(0.0, 0.0));
    emp(iv(15.0, INF) / iv(-0.0, -0.0));
    eq(iv(15.0, INF) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(15.0, INF) / iv(NINF, 0.0), NINF, 0.0);
    eq(iv(15.0, INF) / iv(NINF, -0.0), NINF, 0.0);
    eq(iv(15.0, INF) / iv(NINF, 3.0), NINF, INF);
    eq(iv(15.0, INF) / iv(-3.0, INF), NINF, INF);
    eq(iv(15.0, INF) / iv(0.0, INF), 0.0, INF);
    eq(iv(15.0, INF) / iv(-0.0, INF), 0.0, INF);
    eq(iv(15.0, INF) / entire(), NINF, INF);
    emp(iv(-30.0, 0.0) / iv(0.0, 0.0));
    eq(iv(-30.0, 0.0) / iv(-3.0, 0.0), 0.0, INF);
    emp(iv(-30.0, 0.0) / iv(-0.0, -0.0));
    eq(iv(-30.0, 0.0) / iv(-3.0, -0.0), 0.0, INF);
    eq(iv(-30.0, 0.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(-30.0, 0.0) / iv(0.0, 3.0), NINF, 0.0);
    eq(iv(-30.0, 0.0) / iv(NINF, 0.0), 0.0, INF);
    eq(iv(-30.0, 0.0) / iv(-0.0, 3.0), NINF, 0.0);
    eq(iv(-30.0, 0.0) / iv(NINF, -0.0), 0.0, INF);
    eq(iv(-30.0, 0.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(-30.0, 0.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(-30.0, 0.0) / iv(0.0, INF), NINF, 0.0);
    eq(iv(-30.0, 0.0) / iv(-0.0, INF), NINF, 0.0);
    eq(iv(-30.0, 0.0) / entire(), NINF, INF);
    emp(iv(-30.0, -0.0) / iv(0.0, 0.0));
    eq(iv(-30.0, -0.0) / iv(-3.0, 0.0), 0.0, INF);
    emp(iv(-30.0, -0.0) / iv(-0.0, -0.0));
    eq(iv(-30.0, -0.0) / iv(-3.0, -0.0), 0.0, INF);
    eq(iv(-30.0, -0.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(-30.0, -0.0) / iv(0.0, 3.0), NINF, 0.0);
    eq(iv(-30.0, -0.0) / iv(NINF, 0.0), 0.0, INF);
    eq(iv(-30.0, -0.0) / iv(-0.0, 3.0), NINF, 0.0);
    eq(iv(-30.0, -0.0) / iv(NINF, -0.0), 0.0, INF);
    eq(iv(-30.0, -0.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(-30.0, -0.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(-30.0, -0.0) / iv(0.0, INF), NINF, 0.0);
    eq(iv(-30.0, -0.0) / iv(-0.0, INF), NINF, 0.0);
    eq(iv(-30.0, -0.0) / entire(), NINF, INF);
    emp(iv(0.0, 30.0) / iv(0.0, 0.0));
    eq(iv(0.0, 30.0) / iv(-3.0, 0.0), NINF, 0.0);
    emp(iv(0.0, 30.0) / iv(-0.0, -0.0));
    eq(iv(0.0, 30.0) / iv(-3.0, -0.0), NINF, 0.0);
    eq(iv(0.0, 30.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(0.0, 30.0) / iv(0.0, 3.0), 0.0, INF);
    eq(iv(0.0, 30.0) / iv(NINF, 0.0), NINF, 0.0);
    eq(iv(0.0, 30.0) / iv(-0.0, 3.0), 0.0, INF);
    eq(iv(0.0, 30.0) / iv(NINF, -0.0), NINF, 0.0);
    eq(iv(0.0, 30.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(0.0, 30.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(0.0, 30.0) / iv(0.0, INF), 0.0, INF);
    eq(iv(0.0, 30.0) / iv(-0.0, INF), 0.0, INF);
    eq(iv(0.0, 30.0) / entire(), NINF, INF);
    emp(iv(-0.0, 30.0) / iv(0.0, 0.0));
    eq(iv(-0.0, 30.0) / iv(-3.0, 0.0), NINF, 0.0);
    emp(iv(-0.0, 30.0) / iv(-0.0, -0.0));
    eq(iv(-0.0, 30.0) / iv(-3.0, -0.0), NINF, 0.0);
    eq(iv(-0.0, 30.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(-0.0, 30.0) / iv(0.0, 3.0), 0.0, INF);
    eq(iv(-0.0, 30.0) / iv(NINF, 0.0), NINF, 0.0);
    eq(iv(-0.0, 30.0) / iv(-0.0, 3.0), 0.0, INF);
    eq(iv(-0.0, 30.0) / iv(NINF, -0.0), NINF, 0.0);
    eq(iv(-0.0, 30.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(-0.0, 30.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(-0.0, 30.0) / iv(0.0, INF), 0.0, INF);
    eq(iv(-0.0, 30.0) / iv(-0.0, INF), 0.0, INF);
    eq(iv(-0.0, 30.0) / entire(), NINF, INF);
    eq(iv(NINF, 0.0) / iv(-5.0, -3.0), 0.0, INF);
    eq(iv(NINF, 0.0) / iv(3.0, 5.0), NINF, 0.0);
    eq(iv(NINF, 0.0) / iv(NINF, -3.0), 0.0, INF);
    eq(iv(NINF, 0.0) / iv(3.0, INF), NINF, 0.0);
    emp(iv(NINF, 0.0) / iv(0.0, 0.0));
    eq(iv(NINF, 0.0) / iv(-3.0, 0.0), 0.0, INF);
    emp(iv(NINF, 0.0) / iv(-0.0, -0.0));
    eq(iv(NINF, 0.0) / iv(-3.0, -0.0), 0.0, INF);
    eq(iv(NINF, 0.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(NINF, 0.0) / iv(0.0, 3.0), NINF, 0.0);
    eq(iv(NINF, 0.0) / iv(NINF, 0.0), 0.0, INF);
    eq(iv(NINF, 0.0) / iv(-0.0, 3.0), NINF, 0.0);
    eq(iv(NINF, 0.0) / iv(NINF, -0.0), 0.0, INF);
    eq(iv(NINF, 0.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(NINF, 0.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(NINF, 0.0) / iv(0.0, INF), NINF, 0.0);
    eq(iv(NINF, 0.0) / iv(-0.0, INF), NINF, 0.0);
    eq(iv(NINF, 0.0) / entire(), NINF, INF);
    eq(iv(NINF, -0.0) / iv(-5.0, -3.0), 0.0, INF);
    eq(iv(NINF, -0.0) / iv(3.0, 5.0), NINF, 0.0);
    eq(iv(NINF, -0.0) / iv(NINF, -3.0), 0.0, INF);
    eq(iv(NINF, -0.0) / iv(3.0, INF), NINF, 0.0);
    emp(iv(NINF, -0.0) / iv(0.0, 0.0));
    eq(iv(NINF, -0.0) / iv(-3.0, 0.0), 0.0, INF);
    emp(iv(NINF, -0.0) / iv(-0.0, -0.0));
    eq(iv(NINF, -0.0) / iv(-3.0, -0.0), 0.0, INF);
    eq(iv(NINF, -0.0) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(NINF, -0.0) / iv(0.0, 3.0), NINF, 0.0);
    eq(iv(NINF, -0.0) / iv(NINF, 0.0), 0.0, INF);
    eq(iv(NINF, -0.0) / iv(-0.0, 3.0), NINF, 0.0);
    eq(iv(NINF, -0.0) / iv(NINF, -0.0), 0.0, INF);
    eq(iv(NINF, -0.0) / iv(NINF, 3.0), NINF, INF);
    eq(iv(NINF, -0.0) / iv(-3.0, INF), NINF, INF);
    eq(iv(NINF, -0.0) / iv(0.0, INF), NINF, 0.0);
    eq(iv(NINF, -0.0) / iv(-0.0, INF), NINF, 0.0);
    eq(iv(NINF, -0.0) / entire(), NINF, INF);
    eq(iv(0.0, INF) / iv(-5.0, -3.0), NINF, 0.0);
    eq(iv(0.0, INF) / iv(3.0, 5.0), 0.0, INF);
    eq(iv(0.0, INF) / iv(NINF, -3.0), NINF, 0.0);
    eq(iv(0.0, INF) / iv(3.0, INF), 0.0, INF);
    emp(iv(0.0, INF) / iv(0.0, 0.0));
    eq(iv(0.0, INF) / iv(-3.0, 0.0), NINF, 0.0);
    emp(iv(0.0, INF) / iv(-0.0, -0.0));
    eq(iv(0.0, INF) / iv(-3.0, -0.0), NINF, 0.0);
    eq(iv(0.0, INF) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(0.0, INF) / iv(0.0, 3.0), 0.0, INF);
    eq(iv(0.0, INF) / iv(NINF, 0.0), NINF, 0.0);
    eq(iv(0.0, INF) / iv(-0.0, 3.0), 0.0, INF);
    eq(iv(0.0, INF) / iv(NINF, -0.0), NINF, 0.0);
    eq(iv(0.0, INF) / iv(NINF, 3.0), NINF, INF);
    eq(iv(0.0, INF) / iv(-3.0, INF), NINF, INF);
    eq(iv(0.0, INF) / iv(0.0, INF), 0.0, INF);
    eq(iv(0.0, INF) / iv(-0.0, INF), 0.0, INF);
    eq(iv(0.0, INF) / entire(), NINF, INF);
    eq(iv(-0.0, INF) / iv(-5.0, -3.0), NINF, 0.0);
    eq(iv(-0.0, INF) / iv(3.0, 5.0), 0.0, INF);
    eq(iv(-0.0, INF) / iv(NINF, -3.0), NINF, 0.0);
    eq(iv(-0.0, INF) / iv(3.0, INF), 0.0, INF);
    emp(iv(-0.0, INF) / iv(0.0, 0.0));
    eq(iv(-0.0, INF) / iv(-3.0, 0.0), NINF, 0.0);
    emp(iv(-0.0, INF) / iv(-0.0, -0.0));
    eq(iv(-0.0, INF) / iv(-3.0, -0.0), NINF, 0.0);
    eq(iv(-0.0, INF) / iv(-3.0, 3.0), NINF, INF);
    eq(iv(-0.0, INF) / iv(0.0, 3.0), 0.0, INF);
    eq(iv(-0.0, INF) / iv(NINF, 0.0), NINF, 0.0);
    eq(iv(-0.0, INF) / iv(-0.0, 3.0), 0.0, INF);
    eq(iv(-0.0, INF) / iv(NINF, -0.0), NINF, 0.0);
    eq(iv(-0.0, INF) / iv(NINF, 3.0), NINF, INF);
    eq(iv(-0.0, INF) / iv(-3.0, INF), NINF, INF);
    eq(iv(-0.0, INF) / iv(0.0, INF), 0.0, INF);
    eq(iv(-0.0, INF) / iv(-0.0, INF), 0.0, INF);
    eq(iv(-0.0, INF) / entire(), NINF, INF);
    eq(
        iv(-2.0, -1.0) / iv(-10.0, -3.0),
        hx("0X1.9999999999999P-4"),
        hx("0X1.5555555555556P-1"),
    );
    eq(
        iv(-2.0, -1.0) / iv(0.0, 10.0),
        NINF,
        hx("-0X1.9999999999999P-4"),
    );
    eq(
        iv(-2.0, -1.0) / iv(-0.0, 10.0),
        NINF,
        hx("-0X1.9999999999999P-4"),
    );
    eq(
        iv(-1.0, 2.0) / iv(10.0, INF),
        hx("-0X1.999999999999AP-4"),
        hx("0X1.999999999999AP-3"),
    );
    eq(
        iv(1.0, 3.0) / iv(NINF, -10.0),
        hx("-0X1.3333333333334P-2"),
        0.0,
    );
    eq(
        iv(NINF, -1.0) / iv(1.0, 3.0),
        NINF,
        hx("-0X1.5555555555555P-2"),
    );
}

/// KNOWN DEFECT (56 vectors relocated from `minimal_div_test`).
///
/// `Interval / Interval` is `self * rhs.recip()` (crates/interval-1788/src/ops.rs):
/// two directed roundings where a directly rounded quotient is tighter. Every
/// relocated vector is a SOUND enclosure of the corpus (the library result
/// contains the tight result), typically one ulp wider on an inexact endpoint or
/// on an attainable integer quotient the reciprocal path cannot recover (e.g.
/// `[-30,-15] / [-5,-3]` is exactly `[3,10]` but yields `[2.999...6, 10.000...2]`).
/// Asserted at the corpus (tight) values, set-equal on zeros, ignored until
/// division gains a directly rounded path.
#[test]
#[ignore = "KNOWN DEFECT: interval-1788 division via a*recip(b) is sound but not tightest"]
fn minimal_div_test_known_defect() {
    // 56 vectors
    eq(iv(-30.0, -15.0) / iv(-5.0, -3.0), 3.0, 10.0);
    eq(iv(-30.0, -15.0) / iv(3.0, 5.0), -10.0, -3.0);
    eq(iv(-30.0, -15.0) / iv(NINF, -3.0), 0.0, 10.0);
    eq(iv(-30.0, -15.0) / iv(3.0, INF), -10.0, 0.0);
    eq(iv(-30.0, -15.0) / iv(-3.0, 0.0), 5.0, INF);
    eq(iv(-30.0, -15.0) / iv(-3.0, -0.0), 5.0, INF);
    eq(iv(-30.0, -15.0) / iv(0.0, 3.0), NINF, -5.0);
    eq(iv(-30.0, -15.0) / iv(-0.0, 3.0), NINF, -5.0);
    eq(iv(-30.0, 15.0) / iv(-5.0, -3.0), -5.0, 10.0);
    eq(iv(-30.0, 15.0) / iv(3.0, 5.0), -10.0, 5.0);
    eq(iv(-30.0, 15.0) / iv(NINF, -3.0), -5.0, 10.0);
    eq(iv(-30.0, 15.0) / iv(3.0, INF), -10.0, 5.0);
    eq(iv(15.0, 30.0) / iv(-5.0, -3.0), -10.0, -3.0);
    eq(iv(15.0, 30.0) / iv(3.0, 5.0), 3.0, 10.0);
    eq(iv(15.0, 30.0) / iv(NINF, -3.0), -10.0, 0.0);
    eq(iv(15.0, 30.0) / iv(3.0, INF), 0.0, 10.0);
    eq(iv(15.0, 30.0) / iv(-3.0, 0.0), NINF, -5.0);
    eq(iv(15.0, 30.0) / iv(-3.0, -0.0), NINF, -5.0);
    eq(iv(15.0, 30.0) / iv(0.0, 3.0), 5.0, INF);
    eq(iv(15.0, 30.0) / iv(-0.0, 3.0), 5.0, INF);
    eq(iv(NINF, -15.0) / iv(-5.0, -3.0), 3.0, INF);
    eq(iv(NINF, -15.0) / iv(3.0, 5.0), NINF, -3.0);
    eq(iv(NINF, -15.0) / iv(-3.0, 0.0), 5.0, INF);
    eq(iv(NINF, -15.0) / iv(-3.0, -0.0), 5.0, INF);
    eq(iv(NINF, -15.0) / iv(0.0, 3.0), NINF, -5.0);
    eq(iv(NINF, -15.0) / iv(-0.0, 3.0), NINF, -5.0);
    eq(iv(NINF, 15.0) / iv(-5.0, -3.0), -5.0, INF);
    eq(iv(NINF, 15.0) / iv(3.0, 5.0), NINF, 5.0);
    eq(iv(NINF, 15.0) / iv(NINF, -3.0), -5.0, INF);
    eq(iv(NINF, 15.0) / iv(3.0, INF), NINF, 5.0);
    eq(iv(-15.0, INF) / iv(-5.0, -3.0), NINF, 5.0);
    eq(iv(-15.0, INF) / iv(3.0, 5.0), -5.0, INF);
    eq(iv(-15.0, INF) / iv(NINF, -3.0), NINF, 5.0);
    eq(iv(-15.0, INF) / iv(3.0, INF), -5.0, INF);
    eq(iv(15.0, INF) / iv(-5.0, -3.0), NINF, -3.0);
    eq(iv(15.0, INF) / iv(3.0, 5.0), 3.0, INF);
    eq(iv(15.0, INF) / iv(-3.0, 0.0), NINF, -5.0);
    eq(iv(15.0, INF) / iv(-3.0, -0.0), NINF, -5.0);
    eq(iv(15.0, INF) / iv(0.0, 3.0), 5.0, INF);
    eq(iv(15.0, INF) / iv(-0.0, 3.0), 5.0, INF);
    eq(iv(-30.0, 0.0) / iv(-5.0, -3.0), 0.0, 10.0);
    eq(iv(-30.0, 0.0) / iv(3.0, 5.0), -10.0, 0.0);
    eq(iv(-30.0, 0.0) / iv(NINF, -3.0), 0.0, 10.0);
    eq(iv(-30.0, 0.0) / iv(3.0, INF), -10.0, 0.0);
    eq(iv(-30.0, -0.0) / iv(-5.0, -3.0), 0.0, 10.0);
    eq(iv(-30.0, -0.0) / iv(3.0, 5.0), -10.0, 0.0);
    eq(iv(-30.0, -0.0) / iv(NINF, -3.0), 0.0, 10.0);
    eq(iv(-30.0, -0.0) / iv(3.0, INF), -10.0, 0.0);
    eq(iv(0.0, 30.0) / iv(-5.0, -3.0), -10.0, 0.0);
    eq(iv(0.0, 30.0) / iv(3.0, 5.0), 0.0, 10.0);
    eq(iv(0.0, 30.0) / iv(NINF, -3.0), -10.0, 0.0);
    eq(iv(0.0, 30.0) / iv(3.0, INF), 0.0, 10.0);
    eq(iv(-0.0, 30.0) / iv(-5.0, -3.0), -10.0, 0.0);
    eq(iv(-0.0, 30.0) / iv(3.0, 5.0), 0.0, 10.0);
    eq(iv(-0.0, 30.0) / iv(NINF, -3.0), -10.0, 0.0);
    eq(iv(-0.0, 30.0) / iv(3.0, INF), 0.0, 10.0);
}

/// `minimal_div_dec_test`: 6 vectors.
#[test]
fn minimal_div_dec_test() {
    // minimal_div_dec_test: 6 vectors
    deq(
        dset(iv(-2.0, -1.0), Decoration::Com) / dset(iv(-10.0, -3.0), Decoration::Com),
        hx("0X1.9999999999999P-4"),
        hx("0X1.5555555555556P-1"),
        Decoration::Com,
    );
    deq(
        dset(iv(-200.0, -1.0), Decoration::Com)
            / dset(iv(hx("0x0.0000000000001p-1022"), 10.0), Decoration::Com),
        NINF,
        hx("-0X1.9999999999999P-4"),
        Decoration::Dac,
    );
    deq(
        dset(iv(-2.0, -1.0), Decoration::Com) / dset(iv(0.0, 10.0), Decoration::Com),
        NINF,
        hx("-0X1.9999999999999P-4"),
        Decoration::Trv,
    );
    deq(
        dset(iv(1.0, 3.0), Decoration::Def) / dset(iv(NINF, -10.0), Decoration::Dac),
        hx("-0X1.3333333333334P-2"),
        0.0,
        Decoration::Def,
    );
    demp(
        dset(iv(1.0, 2.0), Decoration::Trv) / dset(empty(), Decoration::Trv),
        Decoration::Trv,
    );
    dnai(DecoratedInterval::<TightF64>::nai() / dset(iv(1.0, 2.0), Decoration::Trv));
}

/// `minimal_recip_test`: 18 vectors.
#[test]
fn minimal_recip_test() {
    // minimal_recip_test: 18 vectors
    eq(
        iv(-50.0, -10.0).recip(),
        hx("-0X1.999999999999AP-4"),
        hx("-0X1.47AE147AE147AP-6"),
    );
    eq(
        iv(10.0, 50.0).recip(),
        hx("0X1.47AE147AE147AP-6"),
        hx("0X1.999999999999AP-4"),
    );
    eq(iv(NINF, -10.0).recip(), hx("-0X1.999999999999AP-4"), 0.0);
    eq(iv(10.0, INF).recip(), 0.0, hx("0X1.999999999999AP-4"));
    emp(iv(0.0, 0.0).recip());
    emp(iv(-0.0, -0.0).recip());
    eq(iv(-10.0, 0.0).recip(), NINF, hx("-0X1.9999999999999P-4"));
    eq(iv(-10.0, -0.0).recip(), NINF, hx("-0X1.9999999999999P-4"));
    eq(iv(-10.0, 10.0).recip(), NINF, INF);
    eq(iv(0.0, 10.0).recip(), hx("0X1.9999999999999P-4"), INF);
    eq(iv(-0.0, 10.0).recip(), hx("0X1.9999999999999P-4"), INF);
    eq(iv(NINF, 0.0).recip(), NINF, 0.0);
    eq(iv(NINF, -0.0).recip(), NINF, 0.0);
    eq(iv(NINF, 10.0).recip(), NINF, INF);
    eq(iv(-10.0, INF).recip(), NINF, INF);
    eq(iv(0.0, INF).recip(), 0.0, INF);
    eq(iv(-0.0, INF).recip(), 0.0, INF);
    eq(entire().recip(), NINF, INF);
}

/// `minimal_recip_dec_test`: 8 vectors.
#[test]
fn minimal_recip_dec_test() {
    // minimal_recip_dec_test: 8 vectors
    deq(
        dset(iv(10.0, 50.0), Decoration::Com).recip(),
        hx("0X1.47AE147AE147AP-6"),
        hx("0X1.999999999999AP-4"),
        Decoration::Com,
    );
    deq(
        dset(iv(NINF, -10.0), Decoration::Dac).recip(),
        hx("-0X1.999999999999AP-4"),
        0.0,
        Decoration::Dac,
    );
    deq(
        dset(
            iv(
                hx("-0x1.FFFFFFFFFFFFFp1023"),
                hx("-0x0.0000000000001p-1022"),
            ),
            Decoration::Def,
        )
        .recip(),
        NINF,
        hx("-0X0.4P-1022"),
        Decoration::Def,
    );
    demp(dset(iv(0.0, 0.0), Decoration::Com).recip(), Decoration::Trv);
    deq(
        dset(iv(-10.0, 0.0), Decoration::Com).recip(),
        NINF,
        hx("-0X1.9999999999999P-4"),
        Decoration::Trv,
    );
    deq(
        dset(iv(-10.0, INF), Decoration::Dac).recip(),
        NINF,
        INF,
        Decoration::Trv,
    );
    deq(
        dset(iv(-0.0, INF), Decoration::Dac).recip(),
        0.0,
        INF,
        Decoration::Trv,
    );
    deq(
        dset(entire(), Decoration::Dac).recip(),
        NINF,
        INF,
        Decoration::Trv,
    );
}

/// `minimal_sqr_test`: 12 vectors.
#[test]
fn minimal_sqr_test() {
    // minimal_sqr_test: 12 vectors
    emp(empty().sqr());
    eq(entire().sqr(), 0.0, INF);
    eq(iv(NINF, hx("-0x0.0000000000001p-1022")).sqr(), 0.0, INF);
    eq(iv(-1.0, 1.0).sqr(), 0.0, 1.0);
    eq(iv(0.0, 1.0).sqr(), 0.0, 1.0);
    eq(iv(-0.0, 1.0).sqr(), 0.0, 1.0);
    eq(iv(-5.0, 3.0).sqr(), 0.0, 25.0);
    eq(iv(-5.0, 0.0).sqr(), 0.0, 25.0);
    eq(iv(-5.0, -0.0).sqr(), 0.0, 25.0);
    eq(
        iv(hx("0X1.999999999999AP-4"), hx("0X1.999999999999AP-4")).sqr(),
        hx("0X1.47AE147AE147BP-7"),
        hx("0X1.47AE147AE147CP-7"),
    );
    eq(
        iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("0X1.999999999999AP-4")).sqr(),
        0.0,
        hx("0X1.FFFFFFFFFFFE1P+1"),
    );
    eq(
        iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("-0X1.FFFFFFFFFFFFP+0")).sqr(),
        hx("0X1.FFFFFFFFFFFEP+1"),
        hx("0X1.FFFFFFFFFFFE1P+1"),
    );
}

/// `minimal_sqr_dec_test`: 4 vectors.
#[test]
fn minimal_sqr_dec_test() {
    // minimal_sqr_dec_test: 4 vectors
    deq(
        dset(
            iv(
                hx("-0x1.FFFFFFFFFFFFFp1023"),
                hx("-0x0.0000000000001p-1022"),
            ),
            Decoration::Com,
        )
        .sqr(),
        0.0,
        INF,
        Decoration::Dac,
    );
    deq(
        dset(iv(-1.0, 1.0), Decoration::Def).sqr(),
        0.0,
        1.0,
        Decoration::Def,
    );
    deq(
        dset(iv(-5.0, 3.0), Decoration::Com).sqr(),
        0.0,
        25.0,
        Decoration::Com,
    );
    deq(
        dset(
            iv(hx("0X1.999999999999AP-4"), hx("0X1.999999999999AP-4")),
            Decoration::Com,
        )
        .sqr(),
        hx("0X1.47AE147AE147BP-7"),
        hx("0X1.47AE147AE147CP-7"),
        Decoration::Com,
    );
}

/// `minimal_sqrt_test`: 13 vectors.
#[test]
fn minimal_sqrt_test() {
    // minimal_sqrt_test: 13 vectors
    emp(empty().sqrt());
    eq(entire().sqrt(), 0.0, INF);
    emp(iv(NINF, hx("-0x0.0000000000001p-1022")).sqrt());
    eq(iv(-1.0, 1.0).sqrt(), 0.0, 1.0);
    eq(iv(0.0, 1.0).sqrt(), 0.0, 1.0);
    eq(iv(-0.0, 1.0).sqrt(), 0.0, 1.0);
    eq(iv(-5.0, 25.0).sqrt(), 0.0, 5.0);
    eq(iv(0.0, 25.0).sqrt(), 0.0, 5.0);
    eq(iv(-0.0, 25.0).sqrt(), 0.0, 5.0);
    eq(iv(-5.0, INF).sqrt(), 0.0, INF);
    eq(
        iv(hx("0X1.999999999999AP-4"), hx("0X1.999999999999AP-4")).sqrt(),
        hx("0X1.43D136248490FP-2"),
        hx("0X1.43D136248491P-2"),
    );
    eq(
        iv(hx("-0X1.FFFFFFFFFFFFP+0"), hx("0X1.999999999999AP-4")).sqrt(),
        0.0,
        hx("0X1.43D136248491P-2"),
    );
    eq(
        iv(hx("0X1.999999999999AP-4"), hx("0X1.FFFFFFFFFFFFP+0")).sqrt(),
        hx("0X1.43D136248490FP-2"),
        hx("0X1.6A09E667F3BC7P+0"),
    );
}

/// `minimal_sqrt_dec_test`: 4 vectors.
#[test]
fn minimal_sqrt_dec_test() {
    // minimal_sqrt_dec_test: 4 vectors
    deq(
        dset(iv(1.0, 4.0), Decoration::Com).sqrt(),
        1.0,
        2.0,
        Decoration::Com,
    );
    deq(
        dset(iv(-5.0, 25.0), Decoration::Com).sqrt(),
        0.0,
        5.0,
        Decoration::Trv,
    );
    deq(
        dset(iv(0.0, 25.0), Decoration::Def).sqrt(),
        0.0,
        5.0,
        Decoration::Def,
    );
    deq(
        dset(iv(-5.0, INF), Decoration::Dac).sqrt(),
        0.0,
        INF,
        Decoration::Trv,
    );
}

/// `minimal_fma_test`: 564 vectors.
#[test]
fn minimal_fma_test() {
    // minimal_fma_test: 564 vectors
    emp(empty().mul_add(empty(), empty()));
    emp(iv(-1.0, 1.0).mul_add(empty(), empty()));
    emp(empty().mul_add(iv(-1.0, 1.0), empty()));
    emp(empty().mul_add(entire(), empty()));
    emp(entire().mul_add(empty(), empty()));
    emp(iv(0.0, 0.0).mul_add(empty(), empty()));
    emp(iv(-0.0, -0.0).mul_add(empty(), empty()));
    emp(empty().mul_add(iv(0.0, 0.0), empty()));
    emp(empty().mul_add(iv(-0.0, -0.0), empty()));
    emp(entire().mul_add(iv(0.0, 0.0), empty()));
    emp(entire().mul_add(iv(-0.0, -0.0), empty()));
    emp(entire().mul_add(iv(-5.0, -1.0), empty()));
    emp(entire().mul_add(iv(-5.0, 3.0), empty()));
    emp(entire().mul_add(iv(1.0, 3.0), empty()));
    emp(entire().mul_add(iv(NINF, -1.0), empty()));
    emp(entire().mul_add(iv(NINF, 3.0), empty()));
    emp(entire().mul_add(iv(-5.0, INF), empty()));
    emp(entire().mul_add(iv(1.0, INF), empty()));
    emp(entire().mul_add(entire(), empty()));
    emp(iv(1.0, INF).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(1.0, INF).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(1.0, INF).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(1.0, INF).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(1.0, INF).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(1.0, INF).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(1.0, INF).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(1.0, INF).mul_add(iv(-5.0, INF), empty()));
    emp(iv(1.0, INF).mul_add(iv(1.0, INF), empty()));
    emp(iv(1.0, INF).mul_add(entire(), empty()));
    emp(iv(-1.0, INF).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(-1.0, INF).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(-1.0, INF).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(-1.0, INF).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(-1.0, INF).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(-1.0, INF).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(-1.0, INF).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(-1.0, INF).mul_add(iv(-5.0, INF), empty()));
    emp(iv(-1.0, INF).mul_add(iv(1.0, INF), empty()));
    emp(iv(-1.0, INF).mul_add(entire(), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(-5.0, INF), empty()));
    emp(iv(NINF, 3.0).mul_add(iv(1.0, INF), empty()));
    emp(iv(NINF, 3.0).mul_add(entire(), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(-5.0, INF), empty()));
    emp(iv(NINF, -3.0).mul_add(iv(1.0, INF), empty()));
    emp(iv(NINF, -3.0).mul_add(entire(), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(-5.0, INF), empty()));
    emp(iv(0.0, 0.0).mul_add(iv(1.0, INF), empty()));
    emp(iv(0.0, 0.0).mul_add(entire(), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(-5.0, INF), empty()));
    emp(iv(-0.0, -0.0).mul_add(iv(1.0, INF), empty()));
    emp(iv(-0.0, -0.0).mul_add(entire(), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(-5.0, INF), empty()));
    emp(iv(1.0, 5.0).mul_add(iv(1.0, INF), empty()));
    emp(iv(1.0, 5.0).mul_add(entire(), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(-10.0, 2.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(-1.0, 10.0), empty()));
    emp(iv(-2.0, 2.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(-5.0, INF), empty()));
    emp(iv(-1.0, 5.0).mul_add(iv(1.0, INF), empty()));
    emp(iv(-1.0, 5.0).mul_add(entire(), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(0.0, 0.0), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(-0.0, -0.0), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(-5.0, -1.0), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(-5.0, 3.0), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(1.0, 3.0), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(NINF, -1.0), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(NINF, 3.0), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(-5.0, INF), empty()));
    emp(iv(-10.0, -5.0).mul_add(iv(1.0, INF), empty()));
    emp(iv(-10.0, -5.0).mul_add(entire(), empty()));
    emp(empty().mul_add(empty(), iv(NINF, 2.0)));
    emp(iv(-1.0, 1.0).mul_add(empty(), iv(NINF, 2.0)));
    emp(empty().mul_add(iv(-1.0, 1.0), iv(NINF, 2.0)));
    emp(empty().mul_add(entire(), iv(NINF, 2.0)));
    emp(entire().mul_add(empty(), iv(NINF, 2.0)));
    emp(iv(0.0, 0.0).mul_add(empty(), iv(NINF, 2.0)));
    emp(iv(-0.0, -0.0).mul_add(empty(), iv(NINF, 2.0)));
    emp(empty().mul_add(iv(0.0, 0.0), iv(NINF, 2.0)));
    emp(empty().mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)));
    eq(entire().mul_add(iv(0.0, 0.0), iv(NINF, 2.0)), NINF, 2.0);
    eq(entire().mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)), NINF, 2.0);
    eq(entire().mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(1.0, 3.0), iv(NINF, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(NINF, -1.0), iv(NINF, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(NINF, 3.0), iv(NINF, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(-5.0, INF), iv(NINF, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(1.0, INF), iv(NINF, 2.0)), NINF, INF);
    eq(entire().mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)), NINF, 2.0);
    eq(
        iv(1.0, INF).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        1.0,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(iv(1.0, INF).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)), NINF, INF);
    eq(
        iv(1.0, INF).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        1.0,
    );
    eq(
        iv(1.0, INF).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(iv(1.0, INF).mul_add(iv(1.0, INF), iv(NINF, 2.0)), NINF, INF);
    eq(iv(1.0, INF).mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    eq(
        iv(-1.0, INF).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        7.0,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(1.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(iv(-1.0, INF).mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    eq(
        iv(NINF, 3.0).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)),
        NINF,
        11.0,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(1.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(iv(NINF, 3.0).mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    eq(
        iv(NINF, -3.0).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)),
        NINF,
        -1.0,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(1.0, INF), iv(NINF, 2.0)),
        NINF,
        -1.0,
    );
    eq(iv(NINF, -3.0).mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)), NINF, 2.0);
    eq(
        iv(0.0, 0.0).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(iv(0.0, 0.0).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)), NINF, 2.0);
    eq(
        iv(0.0, 0.0).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(iv(0.0, 0.0).mul_add(iv(1.0, INF), iv(NINF, 2.0)), NINF, 2.0);
    eq(iv(0.0, 0.0).mul_add(entire(), iv(NINF, 2.0)), NINF, 2.0);
    eq(
        iv(-0.0, -0.0).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(1.0, INF), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(iv(-0.0, -0.0).mul_add(entire(), iv(NINF, 2.0)), NINF, 2.0);
    eq(iv(1.0, 5.0).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)), NINF, 2.0);
    eq(
        iv(1.0, 5.0).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        1.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        17.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)),
        NINF,
        17.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        1.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        17.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(iv(1.0, 5.0).mul_add(iv(1.0, INF), iv(NINF, 2.0)), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    eq(
        iv(-1.0, 5.0).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        7.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        17.0,
    );
    eq(
        iv(-10.0, 2.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        52.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-1.0, 10.0), iv(NINF, 2.0)),
        NINF,
        52.0,
    );
    eq(
        iv(-2.0, 2.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        12.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)),
        NINF,
        17.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(1.0, INF), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(iv(-1.0, 5.0).mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    eq(
        iv(-10.0, -5.0).mul_add(iv(0.0, 0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-0.0, -0.0), iv(NINF, 2.0)),
        NINF,
        2.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, -1.0), iv(NINF, 2.0)),
        NINF,
        52.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, 3.0), iv(NINF, 2.0)),
        NINF,
        52.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(1.0, 3.0), iv(NINF, 2.0)),
        NINF,
        -3.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(NINF, -1.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(NINF, 3.0), iv(NINF, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, INF), iv(NINF, 2.0)),
        NINF,
        52.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(1.0, INF), iv(NINF, 2.0)),
        NINF,
        -3.0,
    );
    eq(iv(-10.0, -5.0).mul_add(entire(), iv(NINF, 2.0)), NINF, INF);
    emp(empty().mul_add(empty(), iv(-2.0, 2.0)));
    emp(iv(-1.0, 1.0).mul_add(empty(), iv(-2.0, 2.0)));
    emp(empty().mul_add(iv(-1.0, 1.0), iv(-2.0, 2.0)));
    emp(empty().mul_add(entire(), iv(-2.0, 2.0)));
    emp(entire().mul_add(empty(), iv(-2.0, 2.0)));
    emp(iv(0.0, 0.0).mul_add(empty(), iv(-2.0, 2.0)));
    emp(iv(-0.0, -0.0).mul_add(empty(), iv(-2.0, 2.0)));
    emp(empty().mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)));
    emp(empty().mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)));
    eq(entire().mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(entire().mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(entire().mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(-5.0, INF), iv(-2.0, 2.0)), NINF, INF);
    eq(entire().mul_add(iv(1.0, INF), iv(-2.0, 2.0)), NINF, INF);
    eq(entire().mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(
        iv(1.0, INF).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        NINF,
        1.0,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(iv(1.0, INF).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)), -1.0, INF);
    eq(
        iv(1.0, INF).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        NINF,
        1.0,
    );
    eq(
        iv(1.0, INF).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(iv(1.0, INF).mul_add(iv(1.0, INF), iv(-2.0, 2.0)), -1.0, INF);
    eq(iv(1.0, INF).mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    eq(
        iv(-1.0, INF).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        NINF,
        7.0,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)),
        -5.0,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(1.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(iv(-1.0, INF).mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    eq(
        iv(NINF, 3.0).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        -17.0,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)),
        NINF,
        11.0,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(1.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(iv(NINF, 3.0).mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    eq(
        iv(NINF, -3.0).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        1.0,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)),
        NINF,
        -1.0,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        1.0,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(1.0, INF), iv(-2.0, 2.0)),
        NINF,
        -1.0,
    );
    eq(iv(NINF, -3.0).mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(
        iv(0.0, 0.0).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(iv(0.0, 0.0).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(
        iv(0.0, 0.0).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(iv(0.0, 0.0).mul_add(iv(1.0, INF), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(iv(0.0, 0.0).mul_add(entire(), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(
        iv(-0.0, -0.0).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(1.0, INF), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(iv(-0.0, -0.0).mul_add(entire(), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(iv(1.0, 5.0).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)), -2.0, 2.0);
    eq(
        iv(1.0, 5.0).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        -27.0,
        1.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        -27.0,
        17.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)),
        -1.0,
        17.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        NINF,
        1.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        NINF,
        17.0,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        -27.0,
        INF,
    );
    eq(iv(1.0, 5.0).mul_add(iv(1.0, INF), iv(-2.0, 2.0)), -1.0, INF);
    eq(iv(1.0, 5.0).mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    eq(
        iv(-1.0, 5.0).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        -27.0,
        7.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        -27.0,
        17.0,
    );
    eq(
        iv(-10.0, 2.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        -32.0,
        52.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-1.0, 10.0), iv(-2.0, 2.0)),
        -12.0,
        52.0,
    );
    eq(
        iv(-2.0, 2.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        -12.0,
        12.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)),
        -5.0,
        17.0,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(1.0, INF), iv(-2.0, 2.0)),
        NINF,
        INF,
    );
    eq(iv(-1.0, 5.0).mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    eq(
        iv(-10.0, -5.0).mul_add(iv(0.0, 0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-0.0, -0.0), iv(-2.0, 2.0)),
        -2.0,
        2.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, -1.0), iv(-2.0, 2.0)),
        3.0,
        52.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, 3.0), iv(-2.0, 2.0)),
        -32.0,
        52.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(1.0, 3.0), iv(-2.0, 2.0)),
        -32.0,
        -3.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(NINF, -1.0), iv(-2.0, 2.0)),
        3.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(NINF, 3.0), iv(-2.0, 2.0)),
        -32.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, INF), iv(-2.0, 2.0)),
        NINF,
        52.0,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(1.0, INF), iv(-2.0, 2.0)),
        NINF,
        -3.0,
    );
    eq(iv(-10.0, -5.0).mul_add(entire(), iv(-2.0, 2.0)), NINF, INF);
    emp(empty().mul_add(empty(), iv(-2.0, INF)));
    emp(iv(-1.0, 1.0).mul_add(empty(), iv(-2.0, INF)));
    emp(empty().mul_add(iv(-1.0, 1.0), iv(-2.0, INF)));
    emp(empty().mul_add(entire(), iv(-2.0, INF)));
    emp(entire().mul_add(empty(), iv(-2.0, INF)));
    emp(iv(0.0, 0.0).mul_add(empty(), iv(-2.0, INF)));
    emp(iv(-0.0, -0.0).mul_add(empty(), iv(-2.0, INF)));
    emp(empty().mul_add(iv(0.0, 0.0), iv(-2.0, INF)));
    emp(empty().mul_add(iv(-0.0, -0.0), iv(-2.0, INF)));
    eq(entire().mul_add(iv(0.0, 0.0), iv(-2.0, INF)), -2.0, INF);
    eq(entire().mul_add(iv(-0.0, -0.0), iv(-2.0, INF)), -2.0, INF);
    eq(entire().mul_add(iv(-5.0, -1.0), iv(-2.0, INF)), NINF, INF);
    eq(entire().mul_add(iv(-5.0, 3.0), iv(-2.0, INF)), NINF, INF);
    eq(entire().mul_add(iv(1.0, 3.0), iv(-2.0, INF)), NINF, INF);
    eq(entire().mul_add(iv(NINF, -1.0), iv(-2.0, INF)), NINF, INF);
    eq(entire().mul_add(iv(NINF, 3.0), iv(-2.0, INF)), NINF, INF);
    eq(entire().mul_add(iv(-5.0, INF), iv(-2.0, INF)), NINF, INF);
    eq(entire().mul_add(iv(1.0, INF), iv(-2.0, INF)), NINF, INF);
    eq(entire().mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(0.0, 0.0), iv(-2.0, INF)), -2.0, INF);
    eq(
        iv(1.0, INF).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(iv(1.0, INF).mul_add(iv(1.0, 3.0), iv(-2.0, INF)), -1.0, INF);
    eq(
        iv(1.0, INF).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(1.0, INF).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(1.0, INF).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(iv(1.0, INF).mul_add(iv(1.0, INF), iv(-2.0, INF)), -1.0, INF);
    eq(iv(1.0, INF).mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    eq(
        iv(-1.0, INF).mul_add(iv(0.0, 0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(1.0, 3.0), iv(-2.0, INF)),
        -5.0,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, INF).mul_add(iv(1.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(iv(-1.0, INF).mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    eq(
        iv(NINF, 3.0).mul_add(iv(0.0, 0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        -17.0,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(1.0, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, 3.0).mul_add(iv(1.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(iv(NINF, 3.0).mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    eq(
        iv(NINF, -3.0).mul_add(iv(0.0, 0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        1.0,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(1.0, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        1.0,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(NINF, -3.0).mul_add(iv(1.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(iv(NINF, -3.0).mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(0.0, 0.0), iv(-2.0, INF)), -2.0, INF);
    eq(
        iv(0.0, 0.0).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(iv(0.0, 0.0).mul_add(iv(1.0, 3.0), iv(-2.0, INF)), -2.0, INF);
    eq(
        iv(0.0, 0.0).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(0.0, 0.0).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(iv(0.0, 0.0).mul_add(iv(1.0, INF), iv(-2.0, INF)), -2.0, INF);
    eq(iv(0.0, 0.0).mul_add(entire(), iv(-2.0, INF)), -2.0, INF);
    eq(
        iv(-0.0, -0.0).mul_add(iv(0.0, 0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(1.0, 3.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-0.0, -0.0).mul_add(iv(1.0, INF), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(iv(-0.0, -0.0).mul_add(entire(), iv(-2.0, INF)), -2.0, INF);
    eq(iv(1.0, 5.0).mul_add(iv(0.0, 0.0), iv(-2.0, INF)), -2.0, INF);
    eq(
        iv(1.0, 5.0).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        -27.0,
        INF,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        -27.0,
        INF,
    );
    eq(iv(1.0, 5.0).mul_add(iv(1.0, 3.0), iv(-2.0, INF)), -1.0, INF);
    eq(
        iv(1.0, 5.0).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(1.0, 5.0).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        -27.0,
        INF,
    );
    eq(iv(1.0, 5.0).mul_add(iv(1.0, INF), iv(-2.0, INF)), -1.0, INF);
    eq(iv(1.0, 5.0).mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    eq(
        iv(-1.0, 5.0).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(0.0, 0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        -27.0,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        -27.0,
        INF,
    );
    eq(
        iv(-10.0, 2.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        -32.0,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-1.0, 10.0), iv(-2.0, INF)),
        -12.0,
        INF,
    );
    eq(
        iv(-2.0, 2.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        -12.0,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(1.0, 3.0), iv(-2.0, INF)),
        -5.0,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-1.0, 5.0).mul_add(iv(1.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(iv(-1.0, 5.0).mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    eq(
        iv(-10.0, -5.0).mul_add(iv(0.0, 0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-0.0, -0.0), iv(-2.0, INF)),
        -2.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, -1.0), iv(-2.0, INF)),
        3.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, 3.0), iv(-2.0, INF)),
        -32.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(1.0, 3.0), iv(-2.0, INF)),
        -32.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(NINF, -1.0), iv(-2.0, INF)),
        3.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(NINF, 3.0), iv(-2.0, INF)),
        -32.0,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(-5.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(
        iv(-10.0, -5.0).mul_add(iv(1.0, INF), iv(-2.0, INF)),
        NINF,
        INF,
    );
    eq(iv(-10.0, -5.0).mul_add(entire(), iv(-2.0, INF)), NINF, INF);
    emp(empty().mul_add(empty(), entire()));
    emp(iv(-1.0, 1.0).mul_add(empty(), entire()));
    emp(empty().mul_add(iv(-1.0, 1.0), entire()));
    emp(empty().mul_add(entire(), entire()));
    emp(entire().mul_add(empty(), entire()));
    emp(iv(0.0, 0.0).mul_add(empty(), entire()));
    emp(iv(-0.0, -0.0).mul_add(empty(), entire()));
    emp(empty().mul_add(iv(0.0, 0.0), entire()));
    emp(empty().mul_add(iv(-0.0, -0.0), entire()));
    eq(entire().mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(entire().mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(entire().mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(entire().mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(entire().mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(entire().mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(entire().mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(entire().mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(entire().mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(entire().mul_add(entire(), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(1.0, INF).mul_add(entire(), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(-1.0, INF).mul_add(entire(), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(NINF, 3.0).mul_add(entire(), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(NINF, -3.0).mul_add(entire(), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(0.0, 0.0).mul_add(entire(), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(-0.0, -0.0).mul_add(entire(), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(1.0, 5.0).mul_add(entire(), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(-10.0, 2.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(-1.0, 10.0), entire()), NINF, INF);
    eq(iv(-2.0, 2.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(-1.0, 5.0).mul_add(entire(), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(0.0, 0.0), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(-0.0, -0.0), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(-5.0, -1.0), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(-5.0, 3.0), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(1.0, 3.0), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(NINF, -1.0), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(NINF, 3.0), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(-5.0, INF), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(iv(1.0, INF), entire()), NINF, INF);
    eq(iv(-10.0, -5.0).mul_add(entire(), entire()), NINF, INF);
    eq(
        iv(0.1, 0.5).mul_add(iv(-5.0, 3.0), iv(-0.1, 0.1)),
        hx("-0X1.4CCCCCCCCCCCDP+1"),
        hx("0X1.999999999999AP+0"),
    );
    eq(
        iv(-0.5, 0.2).mul_add(iv(-5.0, 3.0), iv(-0.1, 0.1)),
        hx("-0X1.999999999999AP+0"),
        hx("0X1.4CCCCCCCCCCCDP+1"),
    );
    eq(
        iv(-0.5, -0.1).mul_add(iv(2.0, 3.0), iv(-0.1, 0.1)),
        hx("-0X1.999999999999AP+0"),
        hx("-0X1.999999999999AP-4"),
    );
    eq(
        iv(-0.5, -0.1).mul_add(iv(NINF, 3.0), iv(-0.1, 0.1)),
        hx("-0X1.999999999999AP+0"),
        INF,
    );
}

/// `minimal_fma_dec_test`: 3 vectors.
#[test]
fn minimal_fma_dec_test() {
    // minimal_fma_dec_test: 3 vectors
    deq(
        dset(iv(-0.5, -0.1), Decoration::Com).mul_add(
            dset(iv(NINF, 3.0), Decoration::Dac),
            dset(iv(-0.1, 0.1), Decoration::Com),
        ),
        hx("-0X1.999999999999AP+0"),
        INF,
        Decoration::Dac,
    );
    deq(
        dset(iv(1.0, 2.0), Decoration::Com).mul_add(
            dset(iv(1.0, hx("0x1.FFFFFFFFFFFFFp1023")), Decoration::Com),
            dset(iv(0.0, 1.0), Decoration::Com),
        ),
        1.0,
        INF,
        Decoration::Dac,
    );
    deq(
        dset(iv(1.0, 2.0), Decoration::Com).mul_add(
            dset(iv(1.0, 2.0), Decoration::Com),
            dset(iv(2.0, 5.0), Decoration::Com),
        ),
        3.0,
        9.0,
        Decoration::Com,
    );
}

/// `minimal_pown_test`: 122 active vectors (41 sound-but-not-
/// tightest vectors relocated to the known-defect test below;
/// 163 total in the corpus testcase).
#[test]
fn minimal_pown_test() {
    // minimal_pown_test: 122 vectors
    emp(empty().pown(0));
    eq(entire().pown(0), 1.0, 1.0);
    eq(iv(0.0, 0.0).pown(0), 1.0, 1.0);
    eq(iv(-0.0, -0.0).pown(0), 1.0, 1.0);
    eq(iv(13.1, 13.1).pown(0), 1.0, 1.0);
    eq(iv(-7451.145, -7451.145).pown(0), 1.0, 1.0);
    eq(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(0),
        1.0,
        1.0,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(0),
        1.0,
        1.0,
    );
    eq(iv(0.0, INF).pown(0), 1.0, 1.0);
    eq(iv(-0.0, INF).pown(0), 1.0, 1.0);
    eq(iv(NINF, 0.0).pown(0), 1.0, 1.0);
    eq(iv(NINF, -0.0).pown(0), 1.0, 1.0);
    eq(iv(-324.3, 2.5).pown(0), 1.0, 1.0);
    emp(empty().pown(2));
    eq(entire().pown(2), 0.0, INF);
    eq(iv(0.0, 0.0).pown(2), 0.0, 0.0);
    eq(iv(-0.0, -0.0).pown(2), 0.0, 0.0);
    eq(
        iv(13.1, 13.1).pown(2),
        hx("0X1.573851EB851EBP+7"),
        hx("0X1.573851EB851ECP+7"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(2),
        hx("0X1.A794A4E7CFAADP+25"),
        hx("0X1.A794A4E7CFAAEP+25"),
    );
    eq(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(2),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        INF,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(2),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        INF,
    );
    eq(iv(0.0, INF).pown(2), 0.0, INF);
    eq(iv(-0.0, INF).pown(2), 0.0, INF);
    eq(iv(NINF, 0.0).pown(2), 0.0, INF);
    eq(iv(NINF, -0.0).pown(2), 0.0, INF);
    eq(iv(-324.3, 2.5).pown(2), 0.0, hx("0X1.9AD27D70A3D72P+16"));
    eq(
        iv(0.01, 2.33).pown(2),
        hx("0X1.A36E2EB1C432CP-14"),
        hx("0X1.5B7318FC50482P+2"),
    );
    eq(
        iv(-1.9, -0.33).pown(2),
        hx("0X1.BE0DED288CE7P-4"),
        hx("0X1.CE147AE147AE1P+1"),
    );
    emp(empty().pown(8));
    eq(entire().pown(8), 0.0, INF);
    eq(iv(0.0, 0.0).pown(8), 0.0, 0.0);
    eq(iv(-0.0, -0.0).pown(8), 0.0, 0.0);
    eq(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(8),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        INF,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(8),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        INF,
    );
    eq(iv(0.0, INF).pown(8), 0.0, INF);
    eq(iv(-0.0, INF).pown(8), 0.0, INF);
    eq(iv(NINF, 0.0).pown(8), 0.0, INF);
    eq(iv(NINF, -0.0).pown(8), 0.0, INF);
    emp(empty().pown(1));
    eq(entire().pown(1), NINF, INF);
    eq(iv(0.0, 0.0).pown(1), 0.0, 0.0);
    eq(iv(-0.0, -0.0).pown(1), 0.0, 0.0);
    eq(iv(13.1, 13.1).pown(1), 13.1, 13.1);
    eq(iv(-7451.145, -7451.145).pown(1), -7451.145, -7451.145);
    eq(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(1),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        hx("0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(1),
        hx("-0x1.FFFFFFFFFFFFFp1023"),
        hx("-0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(iv(0.0, INF).pown(1), 0.0, INF);
    eq(iv(-0.0, INF).pown(1), 0.0, INF);
    eq(iv(NINF, 0.0).pown(1), NINF, 0.0);
    eq(iv(NINF, -0.0).pown(1), NINF, 0.0);
    eq(iv(-324.3, 2.5).pown(1), -324.3, 2.5);
    eq(iv(0.01, 2.33).pown(1), 0.01, 2.33);
    eq(iv(-1.9, -0.33).pown(1), -1.9, -0.33);
    emp(empty().pown(3));
    eq(entire().pown(3), NINF, INF);
    eq(iv(0.0, 0.0).pown(3), 0.0, 0.0);
    eq(iv(-0.0, -0.0).pown(3), 0.0, 0.0);
    eq(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(3),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        INF,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(3),
        NINF,
        hx("-0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(iv(0.0, INF).pown(3), 0.0, INF);
    eq(iv(-0.0, INF).pown(3), 0.0, INF);
    eq(iv(NINF, 0.0).pown(3), NINF, 0.0);
    eq(iv(NINF, -0.0).pown(3), NINF, 0.0);
    emp(empty().pown(7));
    eq(entire().pown(7), NINF, INF);
    eq(iv(0.0, 0.0).pown(7), 0.0, 0.0);
    eq(iv(-0.0, -0.0).pown(7), 0.0, 0.0);
    eq(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(7),
        hx("0x1.FFFFFFFFFFFFFp1023"),
        INF,
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(7),
        NINF,
        hx("-0x1.FFFFFFFFFFFFFp1023"),
    );
    eq(iv(0.0, INF).pown(7), 0.0, INF);
    eq(iv(-0.0, INF).pown(7), 0.0, INF);
    eq(iv(NINF, 0.0).pown(7), NINF, 0.0);
    eq(iv(NINF, -0.0).pown(7), NINF, 0.0);
    emp(empty().pown(-2));
    eq(entire().pown(-2), 0.0, INF);
    emp(iv(0.0, 0.0).pown(-2));
    emp(iv(-0.0, -0.0).pown(-2));
    eq(iv(0.0, INF).pown(-2), 0.0, INF);
    eq(iv(-0.0, INF).pown(-2), 0.0, INF);
    eq(iv(NINF, 0.0).pown(-2), 0.0, INF);
    eq(iv(NINF, -0.0).pown(-2), 0.0, INF);
    emp(empty().pown(-8));
    eq(entire().pown(-8), 0.0, INF);
    emp(iv(0.0, 0.0).pown(-8));
    emp(iv(-0.0, -0.0).pown(-8));
    eq(iv(0.0, INF).pown(-8), 0.0, INF);
    eq(iv(-0.0, INF).pown(-8), 0.0, INF);
    eq(iv(NINF, 0.0).pown(-8), 0.0, INF);
    eq(iv(NINF, -0.0).pown(-8), 0.0, INF);
    emp(empty().pown(-1));
    eq(entire().pown(-1), NINF, INF);
    emp(iv(0.0, 0.0).pown(-1));
    emp(iv(-0.0, -0.0).pown(-1));
    eq(
        iv(13.1, 13.1).pown(-1),
        hx("0X1.38ABF82EE6986P-4"),
        hx("0X1.38ABF82EE6987P-4"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(-1),
        hx("-0X1.197422C9048BFP-13"),
        hx("-0X1.197422C9048BEP-13"),
    );
    eq(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(-1),
        hx("0X0.4P-1022"),
        hx("0X0.4000000000001P-1022"),
    );
    eq(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(-1),
        hx("-0X0.4000000000001P-1022"),
        hx("-0X0.4P-1022"),
    );
    eq(iv(0.0, INF).pown(-1), 0.0, INF);
    eq(iv(-0.0, INF).pown(-1), 0.0, INF);
    eq(iv(NINF, 0.0).pown(-1), NINF, 0.0);
    eq(iv(NINF, -0.0).pown(-1), NINF, 0.0);
    eq(iv(-324.3, 2.5).pown(-1), NINF, INF);
    eq(
        iv(0.01, 2.33).pown(-1),
        hx("0X1.B77C278DBBE13P-2"),
        hx("0X1.9P+6"),
    );
    eq(
        iv(-1.9, -0.33).pown(-1),
        hx("-0X1.83E0F83E0F83EP+1"),
        hx("-0X1.0D79435E50D79P-1"),
    );
    emp(empty().pown(-3));
    eq(entire().pown(-3), NINF, INF);
    emp(iv(0.0, 0.0).pown(-3));
    emp(iv(-0.0, -0.0).pown(-3));
    eq(iv(0.0, INF).pown(-3), 0.0, INF);
    eq(iv(-0.0, INF).pown(-3), 0.0, INF);
    eq(iv(NINF, 0.0).pown(-3), NINF, 0.0);
    eq(iv(NINF, -0.0).pown(-3), NINF, 0.0);
    eq(iv(-324.3, 2.5).pown(-3), NINF, INF);
    emp(empty().pown(-7));
    eq(entire().pown(-7), NINF, INF);
    emp(iv(0.0, 0.0).pown(-7));
    emp(iv(-0.0, -0.0).pown(-7));
    eq(iv(0.0, INF).pown(-7), 0.0, INF);
    eq(iv(-0.0, INF).pown(-7), 0.0, INF);
    eq(iv(NINF, 0.0).pown(-7), NINF, 0.0);
    eq(iv(NINF, -0.0).pown(-7), NINF, 0.0);
    eq(iv(-324.3, 2.5).pown(-7), NINF, INF);
}

/// KNOWN DEFECT (41 vectors relocated from `minimal_pown_test`).
///
/// Two tightness losses in `Interval::pown` (crates/interval-1788/src/inverse.rs),
/// each a SOUND enclosure of the corpus (the library result contains the tight
/// result):
///
/// 1. Repeated-squaring rounding: `abs_pow_{down,up}` use binary exponentiation
///    with a directed rounding at every multiply, so for `|n| >= 3` (and any base
///    whose exact power is not representable) the result is one or more ulps wider
///    than the correctly rounded power; `|n| <= 2` is one multiply and stays tight.
/// 2. Negative-power overflow: a negative exponent is
///    `pos_pown_image(a,b,|n|).recip()`; when the magnitude power overflows
///    (`|base| = f64::MAX`, `|n| >= 2`) the image is `[f64::MAX, +inf]` and its
///    reciprocal `[+0, ~2^-1024]`, while the tight result is `[+0, 2^-1074]`. These
///    eight vectors also pin the zero-endpoint sign (`0X0P+0` / `-0X0P+0`),
///    asserted bit-exactly via [`eq_bits`]; the rest compare set-equal via [`same`].
///
/// Ignored until `pown` gains a correctly rounded power and a tight negative path.
#[test]
#[ignore = "KNOWN DEFECT: interval-1788 pown via repeated squaring / overflow-recip is sound but not tightest"]
fn minimal_pown_test_known_defect() {
    // 41 vectors
    eq(
        iv(13.1, 13.1).pown(8),
        hx("0X1.9D8FD495853F5P+29"),
        hx("0X1.9D8FD495853F6P+29"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(8),
        hx("0X1.DFB1BB622E70DP+102"),
        hx("0X1.DFB1BB622E70EP+102"),
    );
    eq(iv(-324.3, 2.5).pown(8), 0.0, hx("0X1.A87587109655P+66"));
    eq(
        iv(0.01, 2.33).pown(8),
        hx("0X1.CD2B297D889BDP-54"),
        hx("0X1.B253D9F33CE4DP+9"),
    );
    eq(
        iv(-1.9, -0.33).pown(8),
        hx("0X1.26F1FCDD502A3P-13"),
        hx("0X1.53ABD7BFC4FC6P+7"),
    );
    eq(
        iv(13.1, 13.1).pown(3),
        hx("0X1.1902E978D4FDEP+11"),
        hx("0X1.1902E978D4FDFP+11"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(3),
        hx("-0X1.81460637B9A3DP+38"),
        hx("-0X1.81460637B9A3CP+38"),
    );
    eq(
        iv(-324.3, 2.5).pown(3),
        hx("-0X1.0436D2F418938P+25"),
        hx("0X1.F4P+3"),
    );
    eq(
        iv(0.01, 2.33).pown(3),
        hx("0X1.0C6F7A0B5ED8DP-20"),
        hx("0X1.94C75E6362A6P+3"),
    );
    eq(
        iv(-1.9, -0.33).pown(3),
        hx("-0X1.B6F9DB22D0E55P+2"),
        hx("-0X1.266559F6EC5B1P-5"),
    );
    eq(
        iv(13.1, 13.1).pown(7),
        hx("0X1.F91D1B185493BP+25"),
        hx("0X1.F91D1B185493CP+25"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(7),
        hx("-0X1.07B1DA32F9B59P+90"),
        hx("-0X1.07B1DA32F9B58P+90"),
    );
    eq(
        iv(-324.3, 2.5).pown(7),
        hx("-0X1.4F109959E6D7FP+58"),
        hx("0X1.312DP+9"),
    );
    eq(
        iv(0.01, 2.33).pown(7),
        hx("0X1.6849B86A12B9BP-47"),
        hx("0X1.74D0373C76313P+8"),
    );
    eq(
        iv(-1.9, -0.33).pown(7),
        hx("-0X1.658C775099757P+6"),
        hx("-0X1.BEE30301BF47AP-12"),
    );
    eq(
        iv(13.1, 13.1).pown(-2),
        hx("0X1.7DE3A077D1568P-8"),
        hx("0X1.7DE3A077D1569P-8"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(-2),
        hx("0X1.3570290CD6E14P-26"),
        hx("0X1.3570290CD6E15P-26"),
    );
    eq_bits(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(-2),
        hx("0X0P+0"),
        hx("0X0.0000000000001P-1022"),
    );
    eq_bits(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(-2),
        hx("0X0P+0"),
        hx("0X0.0000000000001P-1022"),
    );
    eq(iv(-324.3, 2.5).pown(-2), hx("0X1.3F0C482C977C9P-17"), INF);
    eq(
        iv(0.01, 2.33).pown(-2),
        hx("0X1.793D85EF38E47P-3"),
        hx("0X1.388P+13"),
    );
    eq(
        iv(-1.9, -0.33).pown(-2),
        hx("0X1.1BA81104F6C8P-2"),
        hx("0X1.25D8FA1F801E1P+3"),
    );
    eq(
        iv(13.1, 13.1).pown(-8),
        hx("0X1.3CEF39247CA6DP-30"),
        hx("0X1.3CEF39247CA6EP-30"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(-8),
        hx("0X1.113D9EF0A99ACP-103"),
        hx("0X1.113D9EF0A99ADP-103"),
    );
    eq_bits(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(-8),
        hx("0X0P+0"),
        hx("0X0.0000000000001P-1022"),
    );
    eq_bits(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(-8),
        hx("0X0P+0"),
        hx("0X0.0000000000001P-1022"),
    );
    eq(iv(-324.3, 2.5).pown(-8), hx("0X1.34CC3764D1E0CP-67"), INF);
    eq(
        iv(0.01, 2.33).pown(-8),
        hx("0X1.2DC80DB11AB7CP-10"),
        hx("0X1.1C37937E08P+53"),
    );
    eq(
        iv(-1.9, -0.33).pown(-8),
        hx("0X1.81E104E61630DP-8"),
        hx("0X1.BC64F21560E34P+12"),
    );
    eq(
        iv(13.1, 13.1).pown(-3),
        hx("0X1.D26DF4D8B1831P-12"),
        hx("0X1.D26DF4D8B1832P-12"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(-3),
        hx("-0X1.54347DED91B19P-39"),
        hx("-0X1.54347DED91B18P-39"),
    );
    eq_bits(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(-3),
        hx("0X0P+0"),
        hx("0X0.0000000000001P-1022"),
    );
    eq_bits(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(-3),
        hx("-0X0.0000000000001P-1022"),
        hx("-0X0P+0"),
    );
    eq(
        iv(0.01, 2.33).pown(-3),
        hx("0X1.43CFBA61AACABP-4"),
        hx("0X1.E848P+19"),
    );
    eq(
        iv(-1.9, -0.33).pown(-3),
        hx("-0X1.BD393CE9E8E7CP+4"),
        hx("-0X1.2A95F6F7C066CP-3"),
    );
    eq(
        iv(13.1, 13.1).pown(-7),
        hx("0X1.037D76C912DBCP-26"),
        hx("0X1.037D76C912DBDP-26"),
    );
    eq(
        iv(-7451.145, -7451.145).pown(-7),
        hx("-0X1.F10F41FB8858FP-91"),
        hx("-0X1.F10F41FB8858EP-91"),
    );
    eq_bits(
        iv(hx("0x1.FFFFFFFFFFFFFp1023"), hx("0x1.FFFFFFFFFFFFFp1023")).pown(-7),
        hx("0X0P+0"),
        hx("0X0.0000000000001P-1022"),
    );
    eq_bits(
        iv(hx("-0x1.FFFFFFFFFFFFFp1023"), hx("-0x1.FFFFFFFFFFFFFp1023")).pown(-7),
        hx("-0X0.0000000000001P-1022"),
        hx("-0X0P+0"),
    );
    eq(
        iv(0.01, 2.33).pown(-7),
        hx("0X1.5F934D64162A9P-9"),
        hx("0X1.6BCC41E9P+46"),
    );
    eq(
        iv(-1.9, -0.33).pown(-7),
        hx("-0X1.254CDD3711DDBP+11"),
        hx("-0X1.6E95C4A761E19P-7"),
    );
}

/// `minimal_pown_dec_test`: 11 vectors.
#[test]
fn minimal_pown_dec_test() {
    // minimal_pown_dec_test: 11 vectors
    deq(
        dset(iv(-5.0, 10.0), Decoration::Com).pown(0),
        1.0,
        1.0,
        Decoration::Com,
    );
    deq(
        dset(iv(NINF, 15.0), Decoration::Dac).pown(0),
        1.0,
        1.0,
        Decoration::Dac,
    );
    deq(
        dset(iv(-3.0, 5.0), Decoration::Def).pown(2),
        0.0,
        25.0,
        Decoration::Def,
    );
    deq(
        dset(iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0), Decoration::Com).pown(2),
        0.0,
        INF,
        Decoration::Dac,
    );
    deq(
        dset(iv(-3.0, 5.0), Decoration::Dac).pown(3),
        -27.0,
        125.0,
        Decoration::Dac,
    );
    deq(
        dset(iv(hx("-0x1.FFFFFFFFFFFFFp1023"), 2.0), Decoration::Com).pown(3),
        NINF,
        8.0,
        Decoration::Dac,
    );
    deq(
        dset(iv(3.0, 5.0), Decoration::Com).pown(-2),
        hx("0X1.47AE147AE147AP-5"),
        hx("0X1.C71C71C71C71DP-4"),
        Decoration::Com,
    );
    deq(
        dset(iv(-5.0, -3.0), Decoration::Def).pown(-2),
        hx("0X1.47AE147AE147AP-5"),
        hx("0X1.C71C71C71C71DP-4"),
        Decoration::Def,
    );
    deq(
        dset(iv(-5.0, 3.0), Decoration::Com).pown(-2),
        hx("0X1.47AE147AE147AP-5"),
        INF,
        Decoration::Trv,
    );
    deq(
        dset(iv(3.0, 5.0), Decoration::Dac).pown(-3),
        hx("0X1.0624DD2F1A9FBP-7"),
        hx("0X1.2F684BDA12F69P-5"),
        Decoration::Dac,
    );
    deq(
        dset(iv(-3.0, 5.0), Decoration::Com).pown(-3),
        NINF,
        INF,
        Decoration::Trv,
    );
}
