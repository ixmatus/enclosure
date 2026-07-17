//! The Kulisch reductions against an independent exact reference, plus the
//! oracle-free mode-consistency properties.
//!
//! `astro-float` (pure Rust, stable, arbitrary precision, the workspace's
//! canonical MPFR replacement) computes the exact sum, `sum_abs`, `sum_square`,
//! and `dot` at a precision wide enough to be exact for the bounded test vectors.
//! It shares no code with the accumulator under test, so agreement is real
//! evidence. For each generated vector the tight down and up results are checked
//! to bracket the exact value tightly (`down` is the largest float at or below
//! it, `up` the smallest at or above), the nearest result to be the correctly
//! rounded value with ties to even, and toward zero to be the directed result
//! facing the origin. The adversarial families are the ones decision record 0008
//! names: cancellation towers, subnormal swarms, wide-magnitude spreads, and the
//! overflow shoulder (a dot of large factors overflows, exercising the boundary).
//!
//! The oracle-free lane needs no reference: it asserts `down <= to_nearest <=
//! up`, the directed pair adjacent or equal, toward zero drawn from the directed
//! pair, and exactness detection (integer vectors with a representable sum agree
//! in every mode).

// Exact bit equality drives the mode agreement and exactness checks; float_cmp
// would flag them, so it is allowed in this test crate.
#![allow(clippy::float_cmp)]

use astro_float::{BigFloat, RoundingMode};
use core::cmp::Ordering;
use proptest::prelude::*;
use round_float::{RoundReduction, TightF64};

/// Working precision for the exact reference. The widest exact value a bounded
/// vector here can reach is a dot spanning `2^-2148` to about `2^2004`, roughly
/// `4200` significant bits; `8192` holds it with margin, and `RoundingMode::None`
/// skips truncation so the accumulation stays exact.
const P: usize = 8192;
const RM: RoundingMode = RoundingMode::None;

/// An exact `BigFloat` for a finite `f64`. `astro-float`'s `from_f64` mishandles
/// subnormals (it assumes the normal implicit leading bit), so a subnormal is
/// lifted into the normal range by an exact power of two and divided back, both
/// steps exact. Normal values and zero convert directly.
fn bf(x: f64) -> BigFloat {
    if x == 0.0 {
        BigFloat::from_f64(0.0, 53)
    } else if x.is_normal() {
        BigFloat::from_f64(x, 53)
    } else {
        // x is subnormal: x * 2^1000 is a normal f64 (exact), and dividing that
        // by 2^1000 in BigFloat recovers x exactly (division by a power of two).
        let scaled = libm::scalbn(x, 1000);
        let pow = BigFloat::from_f64(libm::scalbn(1.0, 1000), 53);
        BigFloat::from_f64(scaled, 53).div(&pow, P, RM)
    }
}

fn tf(xs: &[f64]) -> Vec<TightF64> {
    xs.iter().copied().map(TightF64).collect()
}

/// The exact sum of `xs`.
fn exact_sum(xs: &[f64]) -> BigFloat {
    let mut acc = bf(0.0);
    for &x in xs {
        acc = acc.add(&bf(x), P, RM);
    }
    acc
}

fn exact_sum_abs(xs: &[f64]) -> BigFloat {
    let mut acc = bf(0.0);
    for &x in xs {
        acc = acc.add(&bf(x.abs()), P, RM);
    }
    acc
}

fn exact_sum_square(xs: &[f64]) -> BigFloat {
    let mut acc = bf(0.0);
    for &x in xs {
        acc = acc.add(&bf(x).mul(&bf(x), P, RM), P, RM);
    }
    acc
}

fn exact_dot(a: &[f64], b: &[f64]) -> BigFloat {
    let mut acc = bf(0.0);
    for i in 0..a.len() {
        acc = acc.add(&bf(a[i]).mul(&bf(b[i]), P, RM), P, RM);
    }
    acc
}

/// The sign of a `BigFloat`, read from its sign predicates. The `astro-float`
/// `cmp` misreports the sign when an operand is an inexact zero-adjacent value,
/// so every comparison here goes through a subtraction and these predicates,
/// which are reliable.
fn bf_sign(v: &BigFloat) -> Ordering {
    if v.is_zero() {
        Ordering::Equal
    } else if v.is_negative() {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// The sign of `f - v`, with the infinities handled without converting them.
fn cmp_f64_bf(f: f64, v: &BigFloat) -> Ordering {
    if f == f64::INFINITY {
        return Ordering::Greater;
    }
    if f == f64::NEG_INFINITY {
        return Ordering::Less;
    }
    bf_sign(&bf(f).sub(v, P, RM))
}

/// `+0` equals `-0`, `NaN` equals `NaN`; used where the value, not its zero
/// sign, is under test.
fn same(x: f64, y: f64) -> bool {
    (x.is_nan() && y.is_nan()) || x == y
}

fn hx(x: f64) -> String {
    format!("{:#018x}", x.to_bits())
}

/// Check the four directed results of one reduction against the exact value `v`.
fn check_against_exact(v: &BigFloat, down: f64, up: f64, nearest: f64, to_zero: f64) {
    // down is the largest float at or below v.
    assert!(
        cmp_f64_bf(down, v) != Ordering::Greater,
        "down {} exceeds exact",
        hx(down)
    );
    assert!(
        cmp_f64_bf(down.next_up(), v) == Ordering::Greater,
        "down {} is not the largest float at or below exact",
        hx(down)
    );
    // up is the smallest float at or above v.
    assert!(
        cmp_f64_bf(up, v) != Ordering::Less,
        "up {} is below exact",
        hx(up)
    );
    assert!(
        cmp_f64_bf(up.next_down(), v) == Ordering::Less,
        "up {} is not the smallest float at or above exact",
        hx(up)
    );
    // toward zero is the directed result facing the origin.
    let expect_tz = match bf_sign(v) {
        Ordering::Greater => down,
        Ordering::Less => up,
        Ordering::Equal => 0.0,
    };
    assert!(
        same(to_zero, expect_tz),
        "to_zero {} != {}",
        hx(to_zero),
        hx(expect_tz)
    );
    // nearest is the correctly rounded value (ties to even), where the bracket is
    // finite; the overflow bracket (an infinite endpoint) is pinned deterministically.
    if down.is_finite() && up.is_finite() {
        let expect_n = if same(down, up) {
            down
        } else {
            let lo = v.sub(&bf(down), P, RM); // v - down >= 0
            let hi = bf(up).sub(v, P, RM); // up - v >= 0
            match bf_sign(&lo.sub(&hi, P, RM)) {
                Ordering::Less => down,  // v closer to down
                Ordering::Greater => up, // v closer to up
                Ordering::Equal => {
                    // Exact tie: round to the even significand.
                    if down.to_bits() & 1 == 0 {
                        down
                    } else {
                        up
                    }
                }
            }
        };
        assert!(
            same(nearest, expect_n),
            "nearest {} != {}",
            hx(nearest),
            hx(expect_n)
        );
    }
}

/// The oracle-free mode-consistency invariants for one reduction's four results.
fn check_mode_consistency(down: f64, up: f64, nearest: f64, to_zero: f64) {
    assert!(
        down <= nearest,
        "down {} > nearest {}",
        hx(down),
        hx(nearest)
    );
    assert!(nearest <= up, "nearest {} > up {}", hx(nearest), hx(up));
    assert!(down <= up, "down {} > up {}", hx(down), hx(up));
    // The directed pair is adjacent or equal.
    assert!(
        same(up, down) || up == down.next_up(),
        "directed pair not adjacent: {} {}",
        hx(down),
        hx(up)
    );
    // Toward zero is one of the directed results.
    assert!(
        same(to_zero, down) || same(to_zero, up),
        "to_zero {} not in {{down {}, up {}}}",
        hx(to_zero),
        hx(down),
        hx(up)
    );
}

fn check_sum(xs: &[f64]) {
    let v = tf(xs);
    let down = TightF64::sum_down(&v).0;
    let up = TightF64::sum_up(&v).0;
    let nearest = TightF64::sum_to_nearest(&v).0;
    let to_zero = TightF64::sum_to_zero(&v).0;
    check_mode_consistency(down, up, nearest, to_zero);
    check_against_exact(&exact_sum(xs), down, up, nearest, to_zero);
}

fn check_sum_abs(xs: &[f64]) {
    let v = tf(xs);
    let down = TightF64::sum_abs_down(&v).0;
    let up = TightF64::sum_abs_up(&v).0;
    let nearest = TightF64::sum_abs_to_nearest(&v).0;
    let to_zero = TightF64::sum_abs_to_zero(&v).0;
    check_mode_consistency(down, up, nearest, to_zero);
    check_against_exact(&exact_sum_abs(xs), down, up, nearest, to_zero);
}

fn check_sum_square(xs: &[f64]) {
    let v = tf(xs);
    let down = TightF64::sum_square_down(&v).0;
    let up = TightF64::sum_square_up(&v).0;
    let nearest = TightF64::sum_square_to_nearest(&v).0;
    let to_zero = TightF64::sum_square_to_zero(&v).0;
    check_mode_consistency(down, up, nearest, to_zero);
    check_against_exact(&exact_sum_square(xs), down, up, nearest, to_zero);
}

fn check_dot(a: &[f64], b: &[f64]) {
    let av = tf(a);
    let bv = tf(b);
    let down = TightF64::dot_down(&av, &bv).0;
    let up = TightF64::dot_up(&av, &bv).0;
    let nearest = TightF64::dot_to_nearest(&av, &bv).0;
    let to_zero = TightF64::dot_to_zero(&av, &bv).0;
    check_mode_consistency(down, up, nearest, to_zero);
    check_against_exact(&exact_dot(a, b), down, up, nearest, to_zero);
}

// ---------------------------------------------------------------------------
// Adversarial input families.
// ---------------------------------------------------------------------------

/// A normal value with exponent in `[lo, hi]` and a random sign.
fn normal_at(lo: i32, hi: i32) -> impl Strategy<Value = f64> {
    (0u64..(1u64 << 52), lo..=hi, any::<bool>()).prop_map(|(frac, e, neg)| {
        // value = (2^52 + frac) * 2^(e - 52) = (1 + frac/2^52) * 2^e
        #[allow(clippy::cast_precision_loss)] // (2^52 + frac) < 2^53, exact
        let m = ((1u64 << 52) | frac) as f64;
        let v = libm::scalbn(m, e - 52);
        if neg {
            -v
        } else {
            v
        }
    })
}

/// A subnormal or smallest-normal value with a random sign.
fn subnormal() -> impl Strategy<Value = f64> {
    (any::<bool>(), 1u64..(1u64 << 53)).prop_map(|(neg, frac)| {
        let x = f64::from_bits(frac);
        if neg {
            -x
        } else {
            x
        }
    })
}

/// A broad element: ordinary, wide, or subnormal magnitude.
fn wide_elem() -> impl Strategy<Value = f64> {
    prop_oneof![
        3 => normal_at(-60, 60),
        2 => normal_at(-1000, 1000),
        2 => subnormal(),
    ]
}

prop_compose! {
    /// A cancellation tower: a base multiset paired with its exact negations,
    /// plus a residual, so the exact sum nearly or fully cancels while the
    /// partial sums swing over a wide range.
    fn cancellation_tower()(
        bases in prop::collection::vec(normal_at(-40, 900), 1..6),
        residual in normal_at(-40, 40),
        include_residual in any::<bool>(),
    ) -> Vec<f64> {
        let mut v = Vec::new();
        for &b in &bases {
            v.push(b);
            v.push(-b);
        }
        if include_residual {
            v.push(residual);
        }
        v
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1200))]

    /// Broad random vectors across the magnitude regimes.
    #[test]
    fn sum_matches_exact_broad(xs in prop::collection::vec(wide_elem(), 1..16)) {
        check_sum(&xs);
        check_sum_abs(&xs);
        check_sum_square(&xs);
    }

    /// Dot over broad random vectors (products can reach the overflow shoulder).
    #[test]
    fn dot_matches_exact_broad(
        a in prop::collection::vec(wide_elem(), 1..12),
        seed in prop::collection::vec(wide_elem(), 1..12),
    ) {
        let n = a.len().min(seed.len());
        check_dot(&a[..n], &seed[..n]);
    }

    /// Cancellation towers: the exact sum nearly vanishes after wide swings.
    #[test]
    fn sum_matches_exact_cancellation(xs in cancellation_tower()) {
        check_sum(&xs);
    }

    /// Dense subnormal swarms through the underflow extraction path.
    #[test]
    fn sum_matches_exact_subnormal(xs in prop::collection::vec(subnormal(), 1..24)) {
        check_sum(&xs);
        check_sum_abs(&xs);
        check_sum_square(&xs);
    }
}

// ---------------------------------------------------------------------------
// Exactness detection (oracle-free): representable integer sums agree in all
// modes.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1500))]

    /// A vector of small integers has a representable sum, so every mode returns
    /// the same exact value.
    #[test]
    fn integer_sum_agrees_in_all_modes(xs in prop::collection::vec(-1_000_000i32..1_000_000, 1..40)) {
        let fs: Vec<f64> = xs.iter().map(|&i| f64::from(i)).collect();
        let exact: i64 = xs.iter().map(|&i| i64::from(i)).sum();
        #[allow(clippy::cast_precision_loss)] // |exact| < 40 * 10^6 < 2^53
        let want = exact as f64;
        let v = tf(&fs);
        prop_assert!(same(TightF64::sum_down(&v).0, want));
        prop_assert!(same(TightF64::sum_up(&v).0, want));
        prop_assert!(same(TightF64::sum_to_nearest(&v).0, want));
        prop_assert!(same(TightF64::sum_to_zero(&v).0, want));
    }

    /// A dot of small integers is a representable integer, agreeing in all modes.
    #[test]
    fn integer_dot_agrees_in_all_modes(
        a in prop::collection::vec(-30_000i32..30_000, 1..20),
        b in prop::collection::vec(-30_000i32..30_000, 1..20),
    ) {
        let n = a.len().min(b.len());
        let af: Vec<f64> = a[..n].iter().map(|&i| f64::from(i)).collect();
        let bf_: Vec<f64> = b[..n].iter().map(|&i| f64::from(i)).collect();
        let exact: i64 = (0..n).map(|i| i64::from(a[i]) * i64::from(b[i])).sum();
        #[allow(clippy::cast_precision_loss)] // |exact| < 20 * 9*10^8 < 2^53
        let want = exact as f64;
        let av = tf(&af);
        let bv = tf(&bf_);
        prop_assert!(same(TightF64::dot_down(&av, &bv).0, want));
        prop_assert!(same(TightF64::dot_up(&av, &bv).0, want));
        prop_assert!(same(TightF64::dot_to_nearest(&av, &bv).0, want));
        prop_assert!(same(TightF64::dot_to_zero(&av, &bv).0, want));
    }
}

// ---------------------------------------------------------------------------
// Deterministic corners: the empty vector's zero sign and the overflow boundary.
// ---------------------------------------------------------------------------

#[test]
fn empty_vector_zero_sign() {
    // The identity is zero; the sign follows the directed-rounding convention.
    let empty: [TightF64; 0] = [];
    let d = TightF64::sum_down(&empty).0;
    let u = TightF64::sum_up(&empty).0;
    let n = TightF64::sum_to_nearest(&empty).0;
    let z = TightF64::sum_to_zero(&empty).0;
    assert!(d == 0.0 && d.is_sign_negative(), "sum down should be -0");
    assert!(u == 0.0 && u.is_sign_positive(), "sum up should be +0");
    assert!(n == 0.0 && n.is_sign_positive(), "sum nearest should be +0");
    assert!(z == 0.0 && z.is_sign_positive(), "sum to_zero should be +0");
    // sum_abs, sum_square, dot of empties are likewise +0 except down -> -0.
    assert!(TightF64::sum_abs_down(&empty).0.is_sign_negative());
    assert!(TightF64::sum_square_down(&empty).0.is_sign_negative());
    let ea: [TightF64; 0] = [];
    let eb: [TightF64; 0] = [];
    assert!(TightF64::dot_down(&ea, &eb).0.is_sign_negative());
    assert!(TightF64::dot_up(&ea, &eb).0.is_sign_positive());
}

#[test]
fn exact_cancellation_zero_sign() {
    // A nonempty vector cancelling to exactly zero takes the same convention.
    let v = tf(&[1.0, -1.0]);
    assert!(TightF64::sum_down(&v).0.is_sign_negative());
    assert!(TightF64::sum_up(&v).0.is_sign_positive());
    assert!(TightF64::sum_to_nearest(&v).0 == 0.0);
    assert!(TightF64::sum_to_zero(&v).0.is_sign_positive());
}

#[test]
fn overflow_boundary() {
    let big = tf(&[f64::MAX, f64::MAX]);
    // Exact 2*MAX exceeds MAX: down truncates to MAX, up and nearest to infinity.
    assert_eq!(TightF64::sum_down(&big).0, f64::MAX);
    assert_eq!(TightF64::sum_up(&big).0, f64::INFINITY);
    assert_eq!(TightF64::sum_to_nearest(&big).0, f64::INFINITY);
    assert_eq!(TightF64::sum_to_zero(&big).0, f64::MAX);

    let big_neg = tf(&[-f64::MAX, -f64::MAX]);
    assert_eq!(TightF64::sum_down(&big_neg).0, f64::NEG_INFINITY);
    assert_eq!(TightF64::sum_up(&big_neg).0, -f64::MAX);
    assert_eq!(TightF64::sum_to_nearest(&big_neg).0, f64::NEG_INFINITY);
    assert_eq!(TightF64::sum_to_zero(&big_neg).0, -f64::MAX);

    // sum_square and dot overflow to +inf on the away side, MAX toward zero.
    let sq = tf(&[f64::MAX]);
    assert_eq!(TightF64::sum_square_up(&sq).0, f64::INFINITY);
    assert_eq!(TightF64::sum_square_to_zero(&sq).0, f64::MAX);
    let a = tf(&[f64::MAX]);
    let b = tf(&[2.0]);
    assert_eq!(TightF64::dot_up(&a, &b).0, f64::INFINITY);
    assert_eq!(TightF64::dot_down(&a, &b).0, f64::MAX);
}
