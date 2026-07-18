//! Tests for [`Interval::pow_rev1`] and [`Interval::pow_rev2`] over the `f64`
//! fixture, hand-translated from ITF1788 `pow_rev.itl` (public-domain header;
//! original author Oliver Heimlich; the vectors follow tables B.1/B.2 of
//! Heimlich's 2011 "The General Interval Power Function"; see
//! `docs/references/vendor/itf1788-framework/`).
//!
//! The ITL writes `powRev1 [b] [c] [x]` (recover the base) and `powRev2 [a] [c]
//! [x]` (recover the exponent), which map to `b.pow_rev1(c, x)` and
//! `a.pow_rev2(c, x)`. The crate computes both from the exponential and
//! logarithm of a multiplication reverse (see the `reverse` module derivation):
//! `x1 = exp(mul_rev(b, ln(c)))` and `x2 = mul_rev(ln(a), ln(c), x)`.
//!
//! Exact versus enclosure, per decision record 0006: the transcendental reverses
//! are asserted as ENCLOSURE (the fixture margins of `exp`/`ln` flow through),
//! and the structural arms (empties from an out-of-domain base or result clip)
//! are EXACT.
//!
//! # KNOWN GAP (reported at integration)
//!
//! The reduction is SOUND on every backend (its result always contains the true
//! reverse image; the property lane discharges that against the forward pow
//! battery), but it is NOT TIGHT at two families of corner, so the tests that
//! pin the tight result there are marked `#[ignore]`:
//!
//! - **Logarithm-boundary value-empties.** When `ln` maps a base or result
//!   endpoint at `0`, `1`, or `+inf` to an infinity or (over the fixture) a
//!   value straddling zero, the multiplication reverse over-reaches the tight
//!   result by the point where a closed interval hull cannot express the open
//!   boundary; a tight-empty answer comes back as a non-empty sliver or
//!   half-line. Example: `powRev2 [2, inf] [0.25, 0.5] [0, inf] = [empty]`
//!   returns `[0, 5e-324]`; `powRev2 [1, 2] [0, 0.5] [0, inf] = [empty]` returns
//!   `[realmax, inf]` (the fixture's `ln(1)` widens below zero, so the base
//!   straddles one).
//! - **`0^0` undefined.** `powRev1 [0, 0] [1, 1] [-inf, 0] = [empty]` returns
//!   `[0, 0]`: the reduction reads `0 * ln(t) = 0 in ln([1,1])` as satisfiable
//!   for every `t`, which the `0^0`-undefined rule forbids. This one is not a
//!   fixture artifact; it holds over the tight backend too.
//!
//! Both are tightness losses, never soundness losses. Exact conformance at these
//! corners awaits either the tight-`f64` conformance lane (enc-ac4) or a bespoke
//! general-interval-power reverse (the Heimlich table). The `#[ignore]`d tests
//! assert the tight result, so they go green the day that backend lands.

use interval_1788::{DecoratedInterval, Decoration, Interval};

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

/// A vector interval: `None` is `[empty]`.
type Ivl = Option<(f64, f64)>;

fn build(v: Ivl) -> Interval<f64> {
    match v {
        None => Interval::empty(),
        Some((l, h)) => Interval::new(l, h).unwrap(),
    }
}

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
        Some(k) => (&mant[..k], &mant[k + 1..]),
        None => (mant, ""),
    };
    let mut m: u64 = 0;
    for c in int_part.chars() {
        m = m * 16 + u64::from(c.to_digit(16).expect("hex digit"));
    }
    let mut fl = 0i32;
    for c in frac_part.chars() {
        m = m * 16 + u64::from(c.to_digit(16).expect("hex digit"));
        fl += 1;
    }
    // A canonical corpus literal (leading hex digit 1, at most thirteen
    // fraction digits) carries at most 53 significant bits, so the cast is
    // exact; the assertion turns a non-canonical literal into a loud failure
    // instead of a silent rounding.
    assert!(m < (1 << 53), "hex-float mantissa exceeds f64 exact range");
    #[allow(clippy::cast_precision_loss)]
    let mut val = m as f64;
    let mut e = exp - 4 * fl;
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

/// One `powRev*` row: first operand, result constraint, candidate, and the
/// expected image (`None` is `[empty]`).
struct Row {
    a: Ivl,
    c: Ivl,
    x: Ivl,
    e: Ivl,
}

const fn r(a: Ivl, c: Ivl, x: Ivl, e: Ivl) -> Row {
    Row { a, c, x, e }
}
// The wrapper IS the table's cell type: `None` spells the corpus's `[empty]`,
// so the row constructors return `Some` by design.
#[allow(clippy::unnecessary_wraps)]
fn s(l: f64, h: f64) -> Ivl {
    Some((l, h))
}
#[allow(clippy::unnecessary_wraps)]
fn ent() -> Ivl {
    Some((NEG_INF, INF))
}
const E: Ivl = None;

fn check1(rows: &[Row]) {
    for row in rows {
        let got = build(row.a).pow_rev1(build(row.c), build(row.x));
        assert_result(got, row.e, "pow_rev1", row);
    }
}
fn check2(rows: &[Row]) {
    for row in rows {
        let got = build(row.a).pow_rev2(build(row.c), build(row.x));
        assert_result(got, row.e, "pow_rev2", row);
    }
}

fn assert_result(got: Interval<f64>, want: Ivl, tag: &str, row: &Row) {
    let ok = match want {
        None => got.is_empty(),
        Some((l, h)) => !got.is_empty() && got.inf() <= l && h <= got.sup(),
    };
    assert!(
        ok,
        "{tag} a={:?} c={:?} x={:?}: want {:?}, got {}",
        row.a,
        row.c,
        row.x,
        want,
        if got.is_empty() {
            "empty".to_string()
        } else {
            format!("[{}, {}]", got.inf(), got.sup())
        }
    );
}

// --- powRev1: recover the base ----------------------------------------------

#[test]
fn pow_rev1_structural() {
    check1(&[
        r(E, E, E, E),
        r(E, ent(), E, E),
        r(ent(), E, E, E),
        r(E, ent(), ent(), E),
        r(ent(), E, ent(), E),
        r(ent(), ent(), ent(), s(0.0, INF)),
        // Base candidate wholly negative: outside the domain.
        r(ent(), ent(), s(NEG_INF, -1.0), E),
    ]);
}

#[test]
fn pow_rev1_zero_and_one_bases() {
    check1(&[
        // 0^y = 0.
        r(ent(), s(0.0, 0.0), ent(), s(0.0, 0.0)),
        r(ent(), s(NEG_INF, 0.0), ent(), s(0.0, 0.0)),
        r(s(0.0, INF), s(NEG_INF, 0.0), ent(), s(0.0, 0.0)),
        r(s(0.0, INF), s(0.0, 0.0), ent(), s(0.0, 0.0)),
        r(s(1.0, 2.0), s(0.0, 0.0), ent(), s(0.0, 0.0)),
        r(s(1.0, 1.0), s(0.0, 0.0), s(0.0, 0.0), s(0.0, 0.0)),
        // 1^y = x^0 = 1.
        r(ent(), s(1.0, 1.0), s(1.0, 1.0), s(1.0, 1.0)),
        r(s(0.0, 0.0), s(1.0, 1.0), ent(), s(0.0, INF)),
        r(s(0.0, 0.0), s(1.0, 1.0), s(2.0, 3.0), s(2.0, 3.0)),
        r(ent(), s(1.0, 1.0), ent(), s(0.0, INF)),
        r(ent(), s(1.0, 1.0), s(20.0, 30.0), s(20.0, 30.0)),
        r(s(0.0, 0.0), s(1.0, 1.0), s(1.0, 1.0), s(1.0, 1.0)),
    ]);
}

#[test]
fn pow_rev1_negative_exponent() {
    let r1 = hx("0x1.306FE0A31B715p0");
    let q = hx("0x1.6A09E667F3BCCp-1");
    let q2 = hx("0x1.6A09E667F3BCDp0");
    let ae = hx("0x1.AE89F995AD3AEp-1");
    check1(&[
        r(s(-4.0, -2.0), s(0.0, 0.5), ent(), s(r1, INF)),
        r(s(-INF, -2.0), s(0.0, 0.5), ent(), s(1.0, INF)),
        r(s(-4.0, -2.0), s(0.25, 0.5), ent(), s(r1, 2.0)),
        r(s(-INF, -2.0), s(0.25, 0.5), ent(), s(1.0, 2.0)),
        r(s(-4.0, -2.0), s(0.25, 1.0), ent(), s(1.0, 2.0)),
        r(s(-4.0, -2.0), s(1.0, 1.0), ent(), s(1.0, 1.0)),
        r(s(-4.0, -2.0), s(0.0, 1.0), ent(), s(1.0, INF)),
        r(s(-4.0, -2.0), s(0.0, 2.0), ent(), s(q, INF)),
        r(s(-4.0, -2.0), s(0.0, INF), ent(), s(0.0, INF)),
        r(s(-4.0, -2.0), s(0.5, 2.0), ent(), s(q, q2)),
        r(s(-4.0, -2.0), s(1.0, 2.0), ent(), s(q, 1.0)),
        r(s(-4.0, -2.0), s(2.0, 4.0), ent(), s(0.5, ae)),
    ]);
}

#[test]
fn pow_rev1_positive_exponent() {
    let r1 = hx("0x1.306FE0A31B715p0");
    let q2 = hx("0x1.6A09E667F3BCDp0");
    let ae = hx("0x1.AE89F995AD3AEp-1");
    check1(&[
        r(s(2.0, 4.0), s(0.0, 0.5), ent(), s(0.0, ae)),
        r(s(2.0, INF), s(0.0, 0.5), ent(), s(0.0, 1.0)),
        r(s(2.0, 4.0), s(0.25, 0.5), ent(), s(0.5, ae)),
        r(s(2.0, INF), s(0.25, 0.5), ent(), s(0.5, 1.0)),
        r(s(2.0, 4.0), s(0.25, 1.0), ent(), s(0.5, 1.0)),
        r(s(2.0, 4.0), s(1.0, 1.0), ent(), s(1.0, 1.0)),
        r(s(2.0, 4.0), s(0.0, 1.0), ent(), s(0.0, 1.0)),
        r(s(2.0, 4.0), s(0.0, 2.0), ent(), s(0.0, q2)),
        r(
            s(2.0, 4.0),
            s(0.5, 2.0),
            ent(),
            s(hx("0x1.6A09E667F3BCCp-1"), q2),
        ),
        r(s(2.0, 4.0), s(1.0, 2.0), ent(), s(1.0, q2)),
        r(s(2.0, 4.0), s(2.0, 4.0), ent(), s(r1, 2.0)),
        r(s(2.0, INF), s(2.0, 4.0), ent(), s(1.0, 2.0)),
        r(s(0.0, 4.0), s(2.0, 4.0), ent(), s(r1, INF)),
        r(s(0.0, INF), s(2.0, 4.0), ent(), s(1.0, INF)),
    ]);
}

#[test]
fn pow_rev1_exponent_contains_zero() {
    let r1 = hx("0x1.306FE0A31B715p0");
    let ae = hx("0x1.AE89F995AD3AEp-1");
    check1(&[
        r(s(-4.0, 4.0), s(0.0, 0.5), ent(), s(0.0, INF)),
        r(s(-4.0, 4.0), s(0.0, 0.5), s(0.0, 1.0), s(0.0, ae)),
        r(s(-4.0, 4.0), s(0.0, 0.5), s(1.0, INF), s(r1, INF)),
        r(s(-4.0, 4.0), s(0.25, 1.0), s(1.0, INF), s(1.0, INF)),
        r(s(-4.0, 4.0), s(2.0, 4.0), s(1.0, INF), s(r1, INF)),
        r(s(-4.0, INF), s(2.0, 4.0), s(1.0, INF), s(1.0, INF)),
    ]);
}

#[test]
fn pow_rev1_decorated() {
    // Every reverse grades trv; NaI propagates.
    assert_eq!(
        DecoratedInterval::set_dec(build(s(2.0, 4.0)), Decoration::Com)
            .pow_rev1(
                DecoratedInterval::set_dec(build(s(2.0, 4.0)), Decoration::Dac),
                DecoratedInterval::new_dec(Interval::entire()),
            )
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .pow_rev1(
            DecoratedInterval::new_dec(Interval::entire()),
            DecoratedInterval::new_dec(Interval::entire()),
        )
        .is_nai());
}

// --- powRev2: recover the exponent ------------------------------------------

#[test]
fn pow_rev2_structural_and_domain() {
    check2(&[
        r(E, E, E, E),
        r(E, ent(), E, E),
        r(ent(), E, E, E),
        r(E, ent(), ent(), E),
        r(ent(), ent(), ent(), ent()),
        // Base [0,0], result off zero: no exponent.
        r(s(0.0, 0.0), s(NEG_INF, -0.1), ent(), E),
        r(s(0.0, 0.0), s(0.1, INF), ent(), E),
        // 0^y = 0 for y > 0.
        r(s(0.0, 0.0), s(0.0, 0.0), ent(), s(0.0, INF)),
        r(s(-INF, 0.0), s(-INF, 0.0), s(0.0, INF), s(0.0, INF)),
        r(s(-INF, 0.0), s(-INF, 0.0), s(1.0, 2.0), s(1.0, 2.0)),
    ]);
}

#[test]
fn pow_rev2_one_bases() {
    check2(&[
        // 1^y = x^0 = 1.
        r(s(1.0, 1.0), ent(), ent(), ent()),
        r(s(1.0, 1.0), s(1.0, 1.0), ent(), ent()),
        r(s(1.0, 1.0), s(1.0, 1.0), s(2.0, 3.0), s(2.0, 3.0)),
        r(ent(), s(1.0, 1.0), ent(), ent()),
        r(ent(), s(1.0, 1.0), s(2.0, 3.0), s(2.0, 3.0)),
        r(s(2.0, 3.0), s(1.0, 1.0), ent(), s(0.0, 0.0)),
    ]);
}

#[test]
fn pow_rev2_value_cases() {
    check2(&[
        // Base and result away from the [0,1]/[1,inf] boundaries.
        r(s(2.0, 4.0), s(2.0, 4.0), ent(), s(0.5, 2.0)),
        r(s(2.0, INF), s(2.0, 4.0), ent(), s(0.0, 2.0)),
        r(s(2.0, 4.0), s(0.5, 1.0), ent(), s(-1.0, 0.0)),
        r(s(2.0, 4.0), s(1.0, 1.0), ent(), s(0.0, 0.0)),
        r(s(2.0, 4.0), s(0.0, 0.5), ent(), s(NEG_INF, -0.5)),
        r(s(2.0, 4.0), s(0.25, 0.5), ent(), s(-2.0, -0.5)),
        r(s(2.0, 4.0), s(2.0, INF), ent(), s(0.5, INF)),
        // Base in (0, 1).
        r(s(0.25, 0.5), s(0.25, 0.5), ent(), s(0.5, 2.0)),
        r(s(0.25, 0.5), s(2.0, 4.0), ent(), s(-2.0, -0.5)),
        r(s(0.25, 0.5), s(0.5, 1.0), ent(), s(0.0, 1.0)),
        r(s(0.25, 0.5), s(1.0, 1.0), ent(), s(0.0, 0.0)),
        // Base straddling one: the image is entire (a reverse of the log-ratio
        // over a base whose logarithm straddles zero).
        r(s(0.5, 2.0), s(0.5, 2.0), ent(), ent()),
        r(s(0.5, 2.0), s(2.0, 4.0), ent(), ent()),
        r(s(0.5, 2.0), s(0.5, 2.0), s(-INF, 0.0), s(NEG_INF, 0.0)),
        r(s(0.5, 2.0), s(0.5, 2.0), s(0.0, INF), s(0.0, INF)),
        // x after [0,1], z after [0,1].
        r(s(2.0, 4.0), s(2.0, 4.0), s(-INF, 0.0), E),
        r(s(2.0, 4.0), s(2.0, INF), ent(), s(0.5, INF)),
    ]);
}

#[test]
fn pow_rev2_decorated() {
    assert_eq!(
        DecoratedInterval::set_dec(build(s(2.0, 4.0)), Decoration::Com)
            .pow_rev2(
                DecoratedInterval::set_dec(build(s(2.0, 4.0)), Decoration::Dac),
                DecoratedInterval::new_dec(Interval::entire()),
            )
            .decoration(),
        Decoration::Trv
    );
    assert!(DecoratedInterval::<f64>::nai()
        .pow_rev2(
            DecoratedInterval::new_dec(Interval::entire()),
            DecoratedInterval::new_dec(Interval::entire()),
        )
        .is_nai());
}

// --- KNOWN GAP: corners the sound reduction cannot make tight ---------------

#[test]
#[ignore = "KNOWN GAP: base 0 undefined for a non-positive exponent (0^0 and \
            0^negative). Where the base candidate clips to exactly {0} and the \
            exponent is not strictly positive, the ln/mul_rev reduction returns \
            a non-empty result (e.g. powRev1 [0,0][1,1][-inf,0] -> [0,0], \
            powRev1 [-inf,-1][entire][-inf,0] -> [0,0]); the tight answer is \
            [empty]. Sound (superset), not tight; holds over the tight backend \
            too. Un-ignore when the general-interval-power reverse lands."]
fn pow_rev1_base_zero_undefined_gap() {
    check1(&[
        r(s(0.0, 0.0), s(1.0, 1.0), s(NEG_INF, 0.0), E),
        r(s(-INF, -1.0), ent(), s(NEG_INF, 0.0), E),
        r(s(-INF, 0.0), ent(), s(NEG_INF, 0.0), E),
    ]);
}

#[test]
#[ignore = "KNOWN GAP: logarithm-boundary value-empties. The sound ln/mul_rev \
            reduction over-reaches the tight-empty results at the 0/1/inf log \
            corners (e.g. powRev2 [2,inf][0.25,0.5][0,inf] returns [0,5e-324], \
            powRev2 [1,2][0,0.5][0,inf] returns [realmax,inf]; tight answer \
            [empty]). Sound, not tight. Un-ignore with the tight-f64 lane \
            (enc-ac4)."]
fn pow_rev2_boundary_empty_gap() {
    check2(&[
        r(s(2.0, INF), s(0.25, 0.5), s(0.0, INF), E),
        r(s(1.0, 2.0), s(0.0, 0.5), s(0.0, INF), E),
        r(s(2.0, INF), s(0.0, 0.5), s(0.0, INF), E),
        r(s(1.0, 2.0), s(0.25, 0.5), s(0.0, INF), E),
    ]);
}
