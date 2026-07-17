//! Arithmetic oracle: the tight binary64 backend certified for TIGHTNESS and the
//! `f64` fixture for ENCLOSURE, against a pfloat correctly-rounded directed
//! reference (round-float decision record 0002, bead enc-ebd).
//!
//! This lane is the independent nightly complement to round-float's in-tree
//! `tight_oracle` lane. That lane judges `TightF64` with an integer big-rational
//! oracle written for the crate; this one judges it with pfloat's
//! arbitrary-precision `BigFloat` and pfloat-libm's correctly-rounded `sqrt`, a
//! wholly separate implementation. Agreement across the two is real evidence, not
//! a restatement of one arithmetic in the vocabulary of the other.
//!
//! # Two properties per operation and direction
//!
//! For each of `add`, `sub`, `mul`, `div`, `sqrt`, and the `recip` (`1 / x`) and
//! `sqr` (`x * x`) specializations, and for both the down and up directions:
//!
//! - TIGHTNESS: `TightF64`'s directed result EQUALS the correctly-rounded directed
//!   value exactly, compared value-level so the documented signed-zero divergence
//!   (decision record 0002 Law 7: a directed-down exact zero is nearest's `+0`, not
//!   `-0`) is not counted a mismatch.
//! - ENCLOSURE: the fixture's directed result soundly brackets that reference,
//!   `fixture_down <= reference_down` and `fixture_up >= reference_up`. The
//!   reference is the tightest float bracket of the exact value, so these two
//!   inequalities are precisely the statement that the fixture never rounds a
//!   bound to the wrong side.
//!
//! # How the correctly-rounded reference is derived
//!
//! pfloat is the correctly-rounded authority here (the same engine pfloat-libm
//! rounds transcendentals through). The derivation is honest per operation:
//!
//! - `add`, `sub`, `mul`, `sqr`: the exact real result of two `f64` operands is a
//!   dyadic rational, so a `BigFloat` at a precision above its width holds it
//!   EXACTLY. A sum or difference needs at most 2098 bits (leading bit `2^1023`,
//!   trailing bit `2^-1074`); a product at most 106 bits. The reference computes at
//!   [`ADD_PREC`] or [`MUL_PREC`], ASSERTS the result is exact (the `INEXACT` flag
//!   is clear), then rounds that exact value to `f64` once in each direction with
//!   [`BigFloat::to_f64_round`]. One directed rounding of an exact value is
//!   correctly rounded by construction.
//! - `div`, `recip`: a quotient is generally non-terminating, so no finite exact
//!   `BigFloat` holds it. The reference rounds the quotient to [`DIV_PREC`] bits in
//!   the target direction, then rounds that to `f64` in the same direction. Directed
//!   double rounding through a higher precision is safe: for any real `x`, target
//!   grid `G` (the `f64` grid, subnormals included), and intermediate precision
//!   `p >= 53`, every `G`-point is `p`-bit representable, so the largest `G`-point
//!   not exceeding `RD_p(x)` is the largest `G`-point not exceeding `x` itself
//!   (`RD_G(RD_p(x)) = RD_G(x)`, and mirrored for `RU`). Round-to-nearest double
//!   rounding lacks this property; the directed modes have it, which is why only
//!   the directed results cross the precision boundary.
//! - `sqrt`: pfloat-libm's [`sqrt_round`](pfloat_libm::f64::sqrt_round) is already
//!   correctly rounded straight to the `f64` grid under a directed mode, so it is
//!   the reference directly, with no intermediate rounding to reason about.
//!
//! `recip` and `sqr` are not distinct trait methods; they are `div(1, x)` and
//! `mul(x, x)`, certified as their own grids because their hazards are specific
//! (reciprocals of powers of two are exact; squares underflow and overflow along
//! their own rays).
//!
//! # Grids
//!
//! Each operation runs a curated hazard pool crossed with itself, plus seeded
//! pseudo-random sweeps over the full finite range, the subnormal and
//! smallest-normal band (the rescaling path of Law 6), and the binade boundaries
//! where neighbor gaps are asymmetric. The pool carries signed zeros, powers of two
//! and their neighbors, subnormals, the overflow shoulder just below `MAX`, exact
//! operands (small integers, perfect squares, reciprocable powers of two), and
//! values spanning the exponent range for mixed-magnitude products and quotients.
//! Division by zero and the square root of a negative are out of scope: those are
//! pole and domain edges the trait routes through a separate convention, not
//! arithmetic-rounding hazards.

// Value-level equality is the point: the reference and the backend must agree on
// the exact float, and exact results must collapse to one value. The `float_cmp`
// lint flags every such comparison, so it is allowed in this differential crate.
#![allow(clippy::float_cmp)]

use pfloat::BigFloat;
use pfloat::RoundingMode::{TowardNegative, TowardPositive};
use pfloat_libm::f64 as cr_libm;
use round_float::{RoundFloat, TightF64};

/// Working precision at which a `BigFloat` sum or difference of two `f64` operands
/// is exact. The widest case (a normal near `MAX` plus a smallest subnormal) spans
/// bit `2^1023` down to bit `2^-1074`, 2098 bit positions; 2400 clears it with
/// margin, and the reference asserts exactness rather than trusting the bound.
const ADD_PREC: u32 = 2400;

/// Working precision at which a `BigFloat` product of two `f64` operands is exact.
/// The exact product of two significands of at most 53 bits each has at most 106
/// bits; 128 clears it, and the reference asserts exactness.
const MUL_PREC: u32 = 128;

/// Intermediate precision for the directed double rounding of a quotient. Any value
/// at or above 53 is correct (see the module docs); 160 gives comfortable headroom.
const DIV_PREC: u32 = 160;

// ---------------------------------------------------------------------------
// The correctly-rounded directed reference, one derivation per operation.
// ---------------------------------------------------------------------------

/// The correctly-rounded directed pair `(down, up)` of the exact real `a + b`. The
/// `BigFloat` sum at [`ADD_PREC`] is exact (asserted), so one directed rounding to
/// `f64` per direction is correctly rounded.
fn add_ref(a: f64, b: f64) -> (f64, f64) {
    let (sum, status) = BigFloat::from_f64(a)
        .add_round(&BigFloat::from_f64(b), ADD_PREC, TowardNegative)
        .expect("ADD_PREC is nonzero");
    assert!(
        !status.inexact(),
        "add reference not exact at ADD_PREC: a={:#018x} b={:#018x}",
        a.to_bits(),
        b.to_bits()
    );
    (
        sum.to_f64_round(TowardNegative).0,
        sum.to_f64_round(TowardPositive).0,
    )
}

/// The correctly-rounded directed pair of the exact real `a - b`, exact at
/// [`ADD_PREC`] for the same reason as [`add_ref`].
fn sub_ref(a: f64, b: f64) -> (f64, f64) {
    let (diff, status) = BigFloat::from_f64(a)
        .sub_round(&BigFloat::from_f64(b), ADD_PREC, TowardNegative)
        .expect("ADD_PREC is nonzero");
    assert!(
        !status.inexact(),
        "sub reference not exact at ADD_PREC: a={:#018x} b={:#018x}",
        a.to_bits(),
        b.to_bits()
    );
    (
        diff.to_f64_round(TowardNegative).0,
        diff.to_f64_round(TowardPositive).0,
    )
}

/// The correctly-rounded directed pair of the exact real `a * b`, exact at
/// [`MUL_PREC`].
fn mul_ref(a: f64, b: f64) -> (f64, f64) {
    let (prod, status) = BigFloat::from_f64(a)
        .mul_round(&BigFloat::from_f64(b), MUL_PREC, TowardNegative)
        .expect("MUL_PREC is nonzero");
    assert!(
        !status.inexact(),
        "mul reference not exact at MUL_PREC: a={:#018x} b={:#018x}",
        a.to_bits(),
        b.to_bits()
    );
    (
        prod.to_f64_round(TowardNegative).0,
        prod.to_f64_round(TowardPositive).0,
    )
}

/// The correctly-rounded directed pair of the exact real `a / b` (`b != 0`) by the
/// directed double rounding of the module docs: round the quotient to [`DIV_PREC`]
/// bits in the target direction, then to `f64` in the same direction.
fn div_ref(a: f64, b: f64) -> (f64, f64) {
    let (x, y) = (BigFloat::from_f64(a), BigFloat::from_f64(b));
    let qd = x
        .div_round(&y, DIV_PREC, TowardNegative)
        .expect("DIV_PREC is nonzero")
        .0;
    let qu = x
        .div_round(&y, DIV_PREC, TowardPositive)
        .expect("DIV_PREC is nonzero")
        .0;
    (
        qd.to_f64_round(TowardNegative).0,
        qu.to_f64_round(TowardPositive).0,
    )
}

/// The correctly-rounded directed pair of `sqrt(a)` for `a >= 0`, straight from
/// pfloat-libm's correctly-rounded directed square root.
fn sqrt_ref(a: f64) -> (f64, f64) {
    (
        cr_libm::sqrt_round(a, TowardNegative).0,
        cr_libm::sqrt_round(a, TowardPositive).0,
    )
}

/// The correctly-rounded directed pair of `1 / x` (`x != 0`).
fn recip_ref(x: f64) -> (f64, f64) {
    div_ref(1.0, x)
}

/// The correctly-rounded directed pair of `x * x`.
fn sqr_ref(x: f64) -> (f64, f64) {
    mul_ref(x, x)
}

// ---------------------------------------------------------------------------
// Comparison primitives.
// ---------------------------------------------------------------------------

/// Value-level equality that treats `+0` and `-0` as equal (the documented
/// zero-sign divergence) and `NaN` as equal to `NaN`.
fn same(x: f64, y: f64) -> bool {
    x == y || (x.is_nan() && y.is_nan())
}

fn hx(x: f64) -> String {
    format!("{:#018x}", x.to_bits())
}

/// Assert the tight backend's directed pair equals the reference, and the fixture's
/// directed pair brackets it. `tight` and `fixture` are the two backends' `(down,
/// up)` outputs; `(rd, ru)` is the correctly-rounded reference.
fn certify(op: &str, ctx: &str, tight: (f64, f64), fixture: (f64, f64), rd: f64, ru: f64) {
    let (td, tu) = tight;
    let (fd, fu) = fixture;
    // TIGHTNESS: the tight backend returns exactly the correctly-rounded bound.
    assert!(
        same(td, rd),
        "TIGHTNESS {op}_down {ctx}: tight={} ref={}",
        hx(td),
        hx(rd)
    );
    assert!(
        same(tu, ru),
        "TIGHTNESS {op}_up {ctx}: tight={} ref={}",
        hx(tu),
        hx(ru)
    );
    // ENCLOSURE: the fixture is never on the wrong side of the tightest bound.
    assert!(
        fd <= rd,
        "ENCLOSURE {op}_down {ctx}: fixture={} exceeds ref_down={}",
        hx(fd),
        hx(rd)
    );
    assert!(
        fu >= ru,
        "ENCLOSURE {op}_up {ctx}: fixture={} below ref_up={}",
        hx(fu),
        hx(ru)
    );
}

fn check_add(a: f64, b: f64) {
    let (rd, ru) = add_ref(a, b);
    let tight = (
        TightF64(a).add_down(TightF64(b)).0,
        TightF64(a).add_up(TightF64(b)).0,
    );
    let fixture = (a.add_down(b), a.add_up(b));
    certify(
        "add",
        &format!("a={} b={}", hx(a), hx(b)),
        tight,
        fixture,
        rd,
        ru,
    );
}

fn check_sub(a: f64, b: f64) {
    let (rd, ru) = sub_ref(a, b);
    let tight = (
        TightF64(a).sub_down(TightF64(b)).0,
        TightF64(a).sub_up(TightF64(b)).0,
    );
    let fixture = (a.sub_down(b), a.sub_up(b));
    certify(
        "sub",
        &format!("a={} b={}", hx(a), hx(b)),
        tight,
        fixture,
        rd,
        ru,
    );
}

fn check_mul(a: f64, b: f64) {
    let (rd, ru) = mul_ref(a, b);
    let tight = (
        TightF64(a).mul_down(TightF64(b)).0,
        TightF64(a).mul_up(TightF64(b)).0,
    );
    let fixture = (a.mul_down(b), a.mul_up(b));
    certify(
        "mul",
        &format!("a={} b={}", hx(a), hx(b)),
        tight,
        fixture,
        rd,
        ru,
    );
}

/// `b == 0` is skipped: the trait routes a division by zero through the pole/overflow
/// convention (decision record 0002 Law 7), not the arithmetic-rounding path.
fn check_div(a: f64, b: f64) {
    if b == 0.0 {
        return;
    }
    let (rd, ru) = div_ref(a, b);
    let tight = (
        TightF64(a).div_down(TightF64(b)).0,
        TightF64(a).div_up(TightF64(b)).0,
    );
    let fixture = (a.div_down(b), a.div_up(b));
    certify(
        "div",
        &format!("a={} b={}", hx(a), hx(b)),
        tight,
        fixture,
        rd,
        ru,
    );
}

/// `a < 0` is skipped: the square root of a negative is a domain edge, not a
/// rounding hazard. Callers pass `a.abs()`.
fn check_sqrt(a: f64) {
    let (rd, ru) = sqrt_ref(a);
    let tight = (TightF64(a).sqrt_down().0, TightF64(a).sqrt_up().0);
    let fixture = (a.sqrt_down(), a.sqrt_up());
    certify("sqrt", &format!("a={}", hx(a)), tight, fixture, rd, ru);
}

fn check_recip(x: f64) {
    if x == 0.0 {
        return;
    }
    let (rd, ru) = recip_ref(x);
    let one = TightF64(1.0);
    let tight = (one.div_down(TightF64(x)).0, one.div_up(TightF64(x)).0);
    let fixture = (1.0_f64.div_down(x), 1.0_f64.div_up(x));
    certify("recip", &format!("x={}", hx(x)), tight, fixture, rd, ru);
}

fn check_sqr(x: f64) {
    let (rd, ru) = sqr_ref(x);
    let tight = (
        TightF64(x).mul_down(TightF64(x)).0,
        TightF64(x).mul_up(TightF64(x)).0,
    );
    let fixture = (x.mul_down(x), x.mul_up(x));
    certify("sqr", &format!("x={}", hx(x)), tight, fixture, rd, ru);
}

// ---------------------------------------------------------------------------
// Seeded reproducible pseudo-random generators (SplitMix64: no dependency, and
// a fixed seed makes any failure a deterministic reproducer).
// ---------------------------------------------------------------------------

struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A finite `f64` with a fully random sign, exponent, and mantissa. Rejects
    /// infinities and NaNs. The uniform exponent field makes large magnitudes and
    /// overflow common, which is deliberate coverage.
    fn finite(&mut self) -> f64 {
        loop {
            let x = f64::from_bits(self.next_u64());
            if x.is_finite() {
                return x;
            }
        }
    }

    /// A value in the subnormal and smallest-normal band (biased exponent in
    /// `[0, 3]`): the rescaling path of Law 6 for products and quotients.
    fn subnormal(&mut self) -> f64 {
        let sign = self.next_u64() & (1 << 63);
        let frac = self.next_u64() & 0x000F_FFFF_FFFF_FFFF;
        let exp = (self.next_u64() % 4) << 52;
        f64::from_bits(sign | exp | frac)
    }

    /// A value a few units in the last place off a random power of two, where
    /// neighbor gaps are asymmetric across the binade boundary.
    fn binade(&mut self) -> f64 {
        let e = i32::try_from(self.next_u64() % 121).unwrap() - 60; // [-60, 60]
        let mut v = (2.0_f64).powi(e);
        let steps = self.next_u64() % 6;
        for _ in 0..steps {
            v = if self.next_u64() & 1 == 0 {
                v.next_up()
            } else {
                v.next_down()
            };
        }
        if self.next_u64() & 1 == 0 {
            v
        } else {
            -v
        }
    }
}

// ---------------------------------------------------------------------------
// The curated hazard pool.
// ---------------------------------------------------------------------------

/// Finite hazard values: signed zeros, small integers and perfect squares, powers
/// of two and their neighbors, subnormals, the overflow shoulder, reciprocable and
/// exact operands, and mixed magnitudes across the exponent range.
fn pool() -> Vec<f64> {
    let mut v = vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        2.0,
        -2.0,
        3.0,
        4.0,
        5.0,
        -5.0,
        6.0,
        9.0,
        16.0,
        25.0,
        100.0,
        1024.0,
        0.5,
        0.25,
        0.125,
        1.5,
        -1.5,
        0.1,
        0.3,
        1.0 / 3.0,
        2.0 / 3.0,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::MIN_POSITIVE.next_down(), // largest subnormal
        f64::MIN_POSITIVE.next_up(),
        f64::from_bits(1),                     // smallest positive subnormal
        f64::from_bits(0x000F_FFFF_FFFF_FFFF), // a mid subnormal
        f64::MAX,
        -f64::MAX,
        f64::MAX.next_down(),
        1.0_f64.next_up(),
        1.0_f64.next_down(),
    ];
    // Powers of two and their neighbors across the range.
    for &e in &[-1022_i32, -600, -100, -52, -1, 1, 52, 100, 500, 1023] {
        let p = (2.0_f64).powi(e);
        v.push(p);
        v.push(-p);
        v.push(p.next_up());
        v.push(p.next_down());
    }
    // Mixed magnitudes for products and quotients that span the exponent range.
    for &m in &[1.0e-300_f64, 1.0e-150, 1.0e-15, 1.0e15, 1.0e150, 1.0e300] {
        v.push(m);
        v.push(-m);
    }
    v
}

/// The nonnegative pool for `sqrt` (magnitudes of the full pool, deduplicated by
/// value is unnecessary; duplicates only repeat an assertion).
fn nonneg_pool() -> Vec<f64> {
    pool().into_iter().map(f64::abs).collect()
}

// ---------------------------------------------------------------------------
// Reference self-validation: a correct reference is a necessary precondition for
// trusting it to judge the backends. Checked before the certification grids.
// ---------------------------------------------------------------------------

#[test]
fn reference_exact_cases_collapse() {
    // On exactly representable results both directions return the one value.
    assert_eq!(add_ref(1.0, 2.0), (3.0, 3.0));
    assert_eq!(add_ref(0.5, 0.25), (0.75, 0.75));
    assert_eq!(sub_ref(1.0, 0.25), (0.75, 0.75));
    assert_eq!(mul_ref(3.0, 4.0), (12.0, 12.0));
    assert_eq!(div_ref(1.0, 4.0), (0.25, 0.25));
    assert_eq!(div_ref(7.0, 2.0), (3.5, 3.5));
    assert_eq!(recip_ref(8.0), (0.125, 0.125));
    assert_eq!(sqr_ref(3.0), (9.0, 9.0));
    assert_eq!(sqrt_ref(9.0), (3.0, 3.0));
    assert_eq!(sqrt_ref(0.0), (0.0, 0.0));
    assert_eq!(sqrt_ref(1024.0 * 1024.0), (1024.0, 1024.0));
}

#[test]
fn tightness_check_has_teeth() {
    // A guard against a vacuous pass: on an inexact result the correctly-rounded
    // reference is a one-ulp-wide directed pair, the tight backend must hit BOTH
    // endpoints exactly (bit-level, no signed zero here), and the sound-but-loose
    // fixture must fall strictly outside on at least one side. If the reference
    // were secretly as loose as the fixture, the strict-widening assertion below
    // would fail, so the tightness equality is not free.
    let teeth = |tight: (f64, f64), fixture: (f64, f64), rd: f64, ru: f64| {
        let (td, tu) = tight;
        let (fd, fu) = fixture;
        assert!(
            rd < ru,
            "case must be inexact: ref_down {} == ref_up",
            hx(rd)
        );
        assert_eq!(
            td.to_bits(),
            rd.to_bits(),
            "tight down must equal ref exactly"
        );
        assert_eq!(
            tu.to_bits(),
            ru.to_bits(),
            "tight up must equal ref exactly"
        );
        assert!(
            fd < rd || fu > ru,
            "fixture must be strictly looser than the tight reference on a side: \
             fixture=[{}, {}] ref=[{}, {}]",
            hx(fd),
            hx(fu),
            hx(rd),
            hx(ru)
        );
    };
    // div 1/3, sqrt 2, and a product of two inexact operands: all irrational or
    // non-terminating, so the reference brackets the truth by two adjacent floats.
    let (rd, ru) = div_ref(1.0, 3.0);
    teeth(
        (
            TightF64(1.0).div_down(TightF64(3.0)).0,
            TightF64(1.0).div_up(TightF64(3.0)).0,
        ),
        (1.0_f64.div_down(3.0), 1.0_f64.div_up(3.0)),
        rd,
        ru,
    );
    let (rd, ru) = sqrt_ref(2.0);
    teeth(
        (TightF64(2.0).sqrt_down().0, TightF64(2.0).sqrt_up().0),
        (2.0_f64.sqrt_down(), 2.0_f64.sqrt_up()),
        rd,
        ru,
    );
    let (a, b) = (0.1_f64, 0.3_f64);
    let (rd, ru) = mul_ref(a, b);
    teeth(
        (
            TightF64(a).mul_down(TightF64(b)).0,
            TightF64(a).mul_up(TightF64(b)).0,
        ),
        (a.mul_down(b), a.mul_up(b)),
        rd,
        ru,
    );
}

#[test]
fn reference_brackets_round_to_nearest_within_one_ulp() {
    // A necessary property of correct directed rounding: down <= up, the pair
    // brackets the round-to-nearest result, and it is at most one ulp wide.
    let mut rng = SplitMix64(0x0102_0304_0506_0708);
    for _ in 0..4000 {
        let (a, b) = (rng.finite(), rng.finite());
        for ((down, up), rn) in [(add_ref(a, b), a + b), (mul_ref(a, b), a * b)] {
            assert!(down <= up, "down {} > up {}", hx(down), hx(up));
            if rn.is_finite() {
                assert!(
                    down <= rn && rn <= up,
                    "rn {} outside [{}, {}]",
                    hx(rn),
                    hx(down),
                    hx(up)
                );
            }
            assert!(
                up == down || up == down.next_up(),
                "gap exceeds one ulp: {} {}",
                hx(down),
                hx(up)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Certification grids, one test per operation. Each reports its case count on
// stderr (visible under `--nocapture`) so the per-op assertion totals are
// recoverable: every case runs two tightness and two enclosure assertions.
// ---------------------------------------------------------------------------

/// Run `check` over the curated pool crossed with itself, then over the seeded
/// random zones, and report the case count. `zones` names the per-zone draw counts.
fn run_binary(op: &str, seed: u64, check: impl Fn(f64, f64)) {
    let p = pool();
    let mut cases = 0_u64;
    for &a in &p {
        for &b in &p {
            check(a, b);
            cases += 1;
        }
    }
    let mut rng = SplitMix64(seed);
    // Broad, both-subnormal, mixed (one subnormal), and binade zones.
    for _ in 0..6000 {
        check(rng.finite(), rng.finite());
        cases += 1;
    }
    for _ in 0..4000 {
        check(rng.subnormal(), rng.subnormal());
        cases += 1;
    }
    for _ in 0..3000 {
        let s = rng.subnormal();
        let f = rng.finite();
        check(s, f);
        check(f, s);
        cases += 2;
    }
    for _ in 0..3000 {
        check(rng.binade(), rng.binade());
        cases += 1;
    }
    eprintln!("ARITH-COUNT op={op} cases={cases} asserts={}", cases * 4);
}

/// Run `check` over a unary grid: the pool, then the random zones.
fn run_unary(op: &str, seed: u64, nonneg: bool, check: impl Fn(f64)) {
    let mut cases = 0_u64;
    let base: Vec<f64> = if nonneg { nonneg_pool() } else { pool() };
    for &x in &base {
        check(x);
        cases += 1;
    }
    let mut rng = SplitMix64(seed);
    let map = |x: f64| if nonneg { x.abs() } else { x };
    for _ in 0..6000 {
        check(map(rng.finite()));
        cases += 1;
    }
    for _ in 0..4000 {
        check(map(rng.subnormal()));
        cases += 1;
    }
    for _ in 0..3000 {
        check(map(rng.binade()));
        cases += 1;
    }
    eprintln!("ARITH-COUNT op={op} cases={cases} asserts={}", cases * 4);
}

#[test]
fn certify_add() {
    run_binary("add", 0xADD0_1111_2222_3333, check_add);
}

#[test]
fn certify_sub() {
    run_binary("sub", 0x50B0_4444_5555_6666, check_sub);
}

#[test]
fn certify_mul() {
    run_binary("mul", 0x0011_7777_8888_9999, check_mul);
}

#[test]
fn certify_div() {
    run_binary("div", 0xD140_AAAA_BBBB_CCCC, check_div);
}

#[test]
fn certify_sqrt() {
    run_unary("sqrt", 0x5027_DDDD_EEEE_FFFF, true, check_sqrt);
}

#[test]
fn certify_recip() {
    run_unary("recip", 0xEC10_1234_5678_9ABC, false, check_recip);
}

#[test]
fn certify_sqr() {
    run_unary("sqr", 0x5008_CAFE_F00D_BEEF, false, check_sqr);
}

// ---------------------------------------------------------------------------
// Named hazard vectors: the edge-convention cases spelled out, so a regression in
// the overflow shoulder or the subnormal rescale is a named failure, not a lucky
// omission from the random draw.
// ---------------------------------------------------------------------------

#[test]
fn edge_overflow_shoulder() {
    // MAX + MAX overflows: down stays at MAX, up goes to +inf (Law 7). The
    // reference agrees because pfloat rounds the exact 2^1024 to the same pair.
    check_add(f64::MAX, f64::MAX);
    check_add(-f64::MAX, -f64::MAX);
    check_sub(f64::MAX, -f64::MAX);
    check_mul(f64::MAX, 2.0);
    check_mul(-f64::MAX, 2.0);
    check_sqr(f64::MAX);
    // Just below the round-to-nearest overflow threshold, the nearest result is
    // still finite while the true value already exceeds MAX: down = MAX, up = +inf.
    check_add(f64::MAX, f64::MAX.next_down() * (2.0_f64).powi(-53));
}

#[test]
fn edge_subnormal_underflow() {
    let tiny = f64::from_bits(1); // smallest positive subnormal
    let mid = f64::from_bits(0x0008_0000_0000_0000);
    // Products and quotients that land in or below the subnormal band, exercising
    // the rescaling path and the round-to-zero shoulder.
    check_mul(tiny, 0.5); // 2^-1075, halfway to zero: down = 0, up = tiny
    check_mul(tiny, tiny);
    check_mul(mid, mid);
    check_mul(f64::MIN_POSITIVE, 0.5);
    check_div(tiny, 2.0);
    check_div(f64::MIN_POSITIVE, 8.0);
    check_sqr(tiny);
    check_sqr(mid);
    check_sqrt(tiny);
    check_sqrt(f64::MIN_POSITIVE);
    check_sqrt(f64::MIN_POSITIVE.next_down());
}

#[test]
fn edge_signed_zero() {
    // Exact-zero results: the tight backend returns nearest's +0 both directions
    // (Law 7 divergence), which `same` accepts as equal to the reference's directed
    // zero; the fixture brackets it.
    check_add(1.0, -1.0);
    check_sub(1.0, 1.0);
    check_add(-0.0, 0.0);
    check_mul(0.0, -3.0);
    check_mul(-0.0, 5.0);
    check_sqrt(0.0);
    check_sqrt(-0.0);
}

#[test]
fn edge_exact_results_collapse() {
    // Representable results: down == up, and the tight backend collapses to the
    // single value while the fixture still brackets it by one ulp on each side.
    check_add(0.5, 0.25);
    check_mul(3.0, 4.0);
    check_div(6.0, 4.0); // 1.5
    check_recip(4.0); // 0.25
    check_recip((2.0_f64).powi(40));
    check_sqr(1024.0);
    check_sqrt(1024.0 * 1024.0);
    check_sqrt((2.0_f64).powi(100));
    // A tight backend on an exact result reports down == up exactly.
    assert_eq!(TightF64(6.0).div_down(TightF64(4.0)).0, 1.5);
    assert_eq!(TightF64(6.0).div_up(TightF64(4.0)).0, 1.5);
}

#[test]
fn edge_near_halfway_and_irrational() {
    // Results near the midpoint of adjacent floats stress the down/up split; the
    // recurring and irrational cases have no exact float, so down and up are the two
    // neighbors of the truth.
    check_div(1.0, 3.0);
    check_div(2.0, 3.0);
    check_div(1.0, 7.0);
    check_recip(3.0);
    check_recip(7.0);
    check_recip(10.0);
    check_sqrt(2.0);
    check_sqrt(3.0);
    check_sqrt(0.5);
    check_mul(1.0 / 3.0, 3.0);
    check_add(1.0_f64.next_up(), -1.0);
    check_sub(1.0, 1.0_f64.next_down());
}
