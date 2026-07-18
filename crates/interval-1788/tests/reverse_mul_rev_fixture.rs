//! Tests for [`Interval::mul_rev_to_pair`] over the `f64` fixture,
//! hand-translated from ITF1788 `libieeep1788_mul_rev.itl` (Apache-2.0; original
//! author Marco Nehmeier, ITL port Oliver Heimlich; see
//! `docs/references/vendor/itf1788-framework/`).
//!
//! The ITL writes `mulRevToPair [b] [c] = [p1] [p2]`, which maps to
//! `b.mul_rev_to_pair(c)`. The two-output division rounds outward on the
//! sound-not-tight fixture, so a nonempty finite piece is asserted as ENCLOSURE
//! of the vector piece; the structural pieces (`empty`, `entire`, and the
//! half-infinite pieces whose finite bound is an exact power of two) come out
//! EXACT and are asserted by the same enclosure predicate, which pins an
//! infinity or a zero exactly. Empty pieces are asserted empty. The full 13x13
//! `(b, c)` grid of the bare testcase is translated as a data table; the
//! decorated testcase is checked for its uniform `trv` grade and `NaI`
//! propagation on a representative sample, since decision record 0006 fixes the
//! decoration rule (every reverse grades `trv`) uniformly.

use interval_1788::{DecoratedInterval, Decoration, Interval};

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

/// A vector interval: `None` is `[empty]`.
type Ivl = Option<(f64, f64)>;

fn build(v: Ivl) -> Interval<f64> {
    match v {
        None => Interval::empty(),
        Some((lo, hi)) => Interval::new(lo, hi).unwrap(),
    }
}

/// Parse a C hex-float literal to the exact `f64` it names (see
/// `inverse_fixture.rs` for the shared idiom).
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

/// Assert `piece` encloses the vector piece `want` (or is empty when `want` is).
fn piece_encloses(piece: Interval<f64>, want: Ivl) {
    match want {
        None => assert!(
            piece.is_empty(),
            "expected empty piece, got [{}, {}]",
            piece.inf(),
            piece.sup()
        ),
        Some((lo, hi)) => assert!(
            !piece.is_empty() && piece.inf() <= lo && hi <= piece.sup(),
            "expected enclosure of [{lo}, {hi}], got [{}, {}]",
            piece.inf(),
            piece.sup()
        ),
    }
}

/// One `mulRevToPair` row: `b`, `c`, and the two expected pieces.
struct Row {
    b: Ivl,
    c: Ivl,
    p1: Ivl,
    p2: Ivl,
}

fn check(rows: &[Row]) {
    for r in rows {
        let (p1, p2) = build(r.b).mul_rev_to_pair(build(r.c));
        piece_encloses(p1, r.p1);
        piece_encloses(p2, r.p2);
    }
}

const fn row(b: Ivl, c: Ivl, p1: Ivl, p2: Ivl) -> Row {
    Row { b, c, p1, p2 }
}

#[test]
fn mul_rev_to_pair_empty_rows() {
    check(&[
        row(None, Some((1.0, 2.0)), None, None),
        row(Some((1.0, 2.0)), None, None, None),
        row(None, None, None, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_negative() {
    // c = [-2.1, -0.4]: sign-definite negative.
    let c = Some((-2.1, -0.4));
    let a = hx("0X1.999999999999AP-3"); // 0.2
    let big = hx("0X1.5P+4"); // 21
    let s = hx("0X1.745D1745D1745P-2"); // 0.3636...
    let t = hx("0X1.A400000000001P+7"); // 210.000...
    check(&[
        row(Some((-2.0, -0.1)), c, Some((a, big)), None),
        row(Some((-2.0, 0.0)), c, Some((a, INF)), None),
        row(Some((-2.0, 1.1)), c, Some((NEG_INF, -s)), Some((a, INF))),
        row(Some((0.0, 1.1)), c, Some((NEG_INF, -s)), None),
        row(Some((0.01, 1.1)), c, Some((-t, -s)), None),
        row(Some((0.0, 0.0)), c, None, None),
        row(Some((NEG_INF, -0.1)), c, Some((0.0, big)), None),
        row(Some((NEG_INF, 0.0)), c, Some((0.0, INF)), None),
        row(
            Some((NEG_INF, 1.1)),
            c,
            Some((NEG_INF, -s)),
            Some((0.0, INF)),
        ),
        row(Some((-2.0, INF)), c, Some((NEG_INF, 0.0)), Some((a, INF))),
        row(Some((0.0, INF)), c, Some((NEG_INF, 0.0)), None),
        row(Some((0.01, INF)), c, Some((-t, 0.0)), None),
        row(
            Some((NEG_INF, INF)),
            c,
            Some((NEG_INF, 0.0)),
            Some((0.0, INF)),
        ),
    ]);
}

#[test]
fn mul_rev_to_pair_c_touches_zero_above() {
    // c = [-2.1, 0.0]: contains zero (at the upper end).
    let c = Some((-2.1, 0.0));
    let big = hx("0X1.5P+4"); // 21
    let t = hx("0X1.A400000000001P+7"); // 210.000...
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((0.0, big)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((-t, 0.0)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((0.0, big)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((-t, 0.0)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_straddles_zero() {
    // c = [-2.1, 0.12]: straddles zero.
    let c = Some((-2.1, 0.12));
    let big = hx("0X1.5P+4"); // 21
    let onetwo = hx("0X1.3333333333333P+0"); // 1.2
    let t = hx("0X1.A400000000001P+7"); // 210.000...
    let twelve = hx("0X1.8P+3"); // 12
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((-onetwo, big)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((-t, twelve)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((-onetwo, big)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((-t, twelve)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_zero_touch_below() {
    // c = [0.0, 0.12]: contains zero (at the lower end).
    let c = Some((0.0, 0.12));
    let onetwo = hx("0X1.3333333333333P+0"); // 1.2
    let twelve = hx("0X1.8P+3"); // 12
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((-onetwo, 0.0)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((0.0, twelve)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((-onetwo, 0.0)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((0.0, twelve)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_positive() {
    // c = [0.01, 0.12]: sign-definite positive.
    let c = Some((0.01, 0.12));
    let onetwo = hx("0X1.3333333333333P+0"); // 1.2
    let sm = hx("0X1.47AE147AE147BP-8"); // 0.005...
    let lo = hx("0X1.29E4129E4129DP-7"); // 0.00909...
    let twelve = hx("0X1.8P+3"); // 12
    check(&[
        row(Some((-2.0, -0.1)), c, Some((-onetwo, -sm)), None),
        row(Some((-2.0, 0.0)), c, Some((NEG_INF, -sm)), None),
        row(Some((-2.0, 1.1)), c, Some((NEG_INF, -sm)), Some((lo, INF))),
        row(Some((0.0, 1.1)), c, Some((lo, INF)), None),
        row(Some((0.01, 1.1)), c, Some((lo, twelve)), None),
        row(Some((0.0, 0.0)), c, None, None),
        row(Some((NEG_INF, -0.1)), c, Some((-onetwo, 0.0)), None),
        row(Some((NEG_INF, 0.0)), c, Some((NEG_INF, 0.0)), None),
        row(
            Some((NEG_INF, 1.1)),
            c,
            Some((NEG_INF, 0.0)),
            Some((lo, INF)),
        ),
        row(Some((-2.0, INF)), c, Some((NEG_INF, -sm)), Some((0.0, INF))),
        row(Some((0.0, INF)), c, Some((0.0, INF)), None),
        row(Some((0.01, INF)), c, Some((0.0, twelve)), None),
        row(
            Some((NEG_INF, INF)),
            c,
            Some((NEG_INF, 0.0)),
            Some((0.0, INF)),
        ),
    ]);
}

#[test]
fn mul_rev_to_pair_c_zero() {
    // c = [0.0, 0.0]: the zero singleton.
    let c = Some((0.0, 0.0));
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((0.0, 0.0)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((0.0, 0.0)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((0.0, 0.0)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((0.0, 0.0)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_neg_unbounded() {
    // c = [-inf, -0.1].
    let c = Some((NEG_INF, -0.1));
    let a = hx("0X1.999999999999AP-5"); // 0.05
    let s4 = hx("0X1.745D1745D1745P-4"); // 0.0909...
    check(&[
        row(Some((-2.0, -0.1)), c, Some((a, INF)), None),
        row(Some((-2.0, 0.0)), c, Some((a, INF)), None),
        row(Some((-2.0, 1.1)), c, Some((NEG_INF, -s4)), Some((a, INF))),
        row(Some((0.0, 1.1)), c, Some((NEG_INF, -s4)), None),
        row(Some((0.01, 1.1)), c, Some((NEG_INF, -s4)), None),
        row(Some((0.0, 0.0)), c, None, None),
        row(Some((NEG_INF, -0.1)), c, Some((0.0, INF)), None),
        row(Some((NEG_INF, 0.0)), c, Some((0.0, INF)), None),
        row(
            Some((NEG_INF, 1.1)),
            c,
            Some((NEG_INF, -s4)),
            Some((0.0, INF)),
        ),
        row(Some((-2.0, INF)), c, Some((NEG_INF, 0.0)), Some((a, INF))),
        row(Some((0.0, INF)), c, Some((NEG_INF, 0.0)), None),
        row(Some((0.01, INF)), c, Some((NEG_INF, 0.0)), None),
        row(
            Some((NEG_INF, INF)),
            c,
            Some((NEG_INF, 0.0)),
            Some((0.0, INF)),
        ),
    ]);
}

#[test]
fn mul_rev_to_pair_c_pos_unbounded() {
    // c = [0.04, inf].
    let c = Some((0.04, INF));
    let sm = hx("0X1.47AE147AE147BP-6"); // 0.02...
    let lo = hx("0X1.29E4129E4129DP-5"); // 0.0363...
    check(&[
        row(Some((-2.0, -0.1)), c, Some((NEG_INF, -sm)), None),
        row(Some((-2.0, 0.0)), c, Some((NEG_INF, -sm)), None),
        row(Some((-2.0, 1.1)), c, Some((NEG_INF, -sm)), Some((lo, INF))),
        row(Some((0.0, 1.1)), c, Some((lo, INF)), None),
        row(Some((0.01, 1.1)), c, Some((lo, INF)), None),
        row(Some((0.0, 0.0)), c, None, None),
        row(Some((NEG_INF, -0.1)), c, Some((NEG_INF, 0.0)), None),
        row(Some((NEG_INF, 0.0)), c, Some((NEG_INF, 0.0)), None),
        row(
            Some((NEG_INF, 1.1)),
            c,
            Some((NEG_INF, 0.0)),
            Some((lo, INF)),
        ),
        row(Some((-2.0, INF)), c, Some((NEG_INF, -sm)), Some((0.0, INF))),
        row(Some((0.0, INF)), c, Some((0.0, INF)), None),
        row(Some((0.01, INF)), c, Some((0.0, INF)), None),
        row(
            Some((NEG_INF, INF)),
            c,
            Some((NEG_INF, 0.0)),
            Some((0.0, INF)),
        ),
    ]);
}

#[test]
fn mul_rev_to_pair_c_neg_unbounded_to_zero() {
    // c = [-inf, 0.0].
    let c = Some((NEG_INF, 0.0));
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((0.0, INF)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((NEG_INF, 0.0)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((0.0, INF)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((NEG_INF, 0.0)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_neg_unbounded_to_pos() {
    // c = [-inf, 0.3].
    let c = Some((NEG_INF, 0.3));
    let three = hx("0X1.8P+1"); // 3.0
    let thirty = hx("0X1.EP+4"); // 30
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((-three, INF)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((NEG_INF, thirty)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((-three, INF)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((NEG_INF, thirty)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_neg_to_unbounded() {
    // c = [-0.21, inf].
    let c = Some((-0.21, INF));
    let two1 = hx("0X1.0CCCCCCCCCCCDP+1"); // 2.1...
    let big = hx("0X1.5P+4"); // 21
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((NEG_INF, two1)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((-big, INF)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((NEG_INF, two1)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((-big, INF)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_zero_to_unbounded() {
    // c = [0.0, inf].
    let c = Some((0.0, INF));
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, Some((NEG_INF, 0.0)), None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, Some((0.0, INF)), None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, Some((NEG_INF, 0.0)), None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, Some((0.0, INF)), None),
        row(Some((NEG_INF, INF)), c, ent, None),
    ]);
}

#[test]
fn mul_rev_to_pair_c_entire() {
    // c = [entire]: every t works through b0 = 0.
    let c = Some((NEG_INF, INF));
    let ent = Some((NEG_INF, INF));
    check(&[
        row(Some((-2.0, -0.1)), c, ent, None),
        row(Some((-2.0, 0.0)), c, ent, None),
        row(Some((-2.0, 1.1)), c, ent, None),
        row(Some((0.0, 1.1)), c, ent, None),
        row(Some((0.01, 1.1)), c, ent, None),
        row(Some((0.0, 0.0)), c, ent, None),
        row(Some((NEG_INF, -0.1)), c, ent, None),
        row(Some((NEG_INF, 0.0)), c, ent, None),
        row(Some((NEG_INF, 1.1)), c, ent, None),
        row(Some((-2.0, INF)), c, ent, None),
        row(Some((0.0, INF)), c, ent, None),
        row(Some((0.01, INF)), c, ent, None),
        row(c, c, ent, None),
    ]);
}

// --- Decorated: every reverse grades trv, NaI propagates --------------------

fn di(v: Ivl, d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(build(v), d)
}

#[test]
fn mul_rev_to_pair_decorated() {
    let nai = DecoratedInterval::<f64>::nai();
    // NaI in either operand poisons both pieces.
    let (p1, p2) = nai.mul_rev_to_pair(di(Some((1.0, 2.0)), Decoration::Def));
    assert!(p1.is_nai() && p2.is_nai());
    let (p1, p2) = di(Some((1.0, 2.0)), Decoration::Com).mul_rev_to_pair(nai);
    assert!(p1.is_nai() && p2.is_nai());

    // Empty operands give two trv empties.
    let (p1, p2) = di(None, Decoration::Trv).mul_rev_to_pair(di(Some((1.0, 2.0)), Decoration::Def));
    assert_eq!(p1.decoration(), Decoration::Trv);
    assert_eq!(p2.decoration(), Decoration::Trv);
    assert!(p1.interval().is_empty() && p2.interval().is_empty());

    // A split result: both pieces trv, the values enclose the vector, and the
    // second (upper) piece sits wholly above the first.
    let (p1, p2) = di(Some((-2.0, 1.1)), Decoration::Com)
        .mul_rev_to_pair(di(Some((-2.1, -0.4)), Decoration::Dac));
    assert_eq!(p1.decoration(), Decoration::Trv);
    assert_eq!(p2.decoration(), Decoration::Trv);
    piece_encloses(p1.interval(), Some((NEG_INF, -hx("0X1.745D1745D1745P-2"))));
    piece_encloses(p2.interval(), Some((hx("0X1.999999999999AP-3"), INF)));
    assert!(p1.interval().sup() <= p2.interval().inf());

    // A connected com/com result stays a single trv piece.
    let (p1, p2) = di(Some((-2.0, -0.1)), Decoration::Com)
        .mul_rev_to_pair(di(Some((-2.1, -0.4)), Decoration::Com));
    assert_eq!(p1.decoration(), Decoration::Trv);
    assert!(p2.interval().is_empty());
}
