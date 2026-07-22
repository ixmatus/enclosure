//! The `pown` kernel against an independent exact reference.
//!
//! `astro-float` (pure Rust, stable, arbitrary precision, the workspace's
//! canonical MPFR replacement) computes the exact integer power at a precision
//! wide enough to be exact for every bounded test input. It shares no code with
//! the kernel under test, so agreement is real evidence. For `1 <= |n| <= 64`
//! the kernel is correctly rounded, so the check is exact: `down` is the largest
//! float at or below the exact value and `up` the smallest at or above it. The
//! reciprocal exact value is rational; a candidate float `c` is compared to it
//! by the sign of `c * base^|n| - 1`, which stays exact at the working precision
//! (no rational is ever represented). Beyond `|n| = 64` the kernel is only
//! sound, so the tail lane asserts `down <= value <= up` and `down <= up`.
//!
//! The edge sweep pins the IEEE 754-2019 special table and the boundary
//! magnitudes the kernel's exponent arithmetic must survive: subnormals, the
//! largest finite, powers of two, near-one bases, both signed zeros, the
//! infinities, NaN, and the exponent extremes `{0, 1, -1, i32::MIN, i32::MAX}`.

// Exact bit equality drives the special-table checks; float_cmp would flag it,
// so it is allowed in this test crate.
#![allow(clippy::float_cmp)]

use astro_float::{BigFloat, RoundingMode};
use core::cmp::Ordering;
use proptest::prelude::*;
use round_float::{RoundPown, TightF64};

/// Working precision for the exact reference. `base^|n|` for `|n| <= 250`
/// carries at most `53 * 250 = 13250` significant bits, and the reciprocal check
/// forms `c * base^|n|` (53 more bits); `16384` holds both with margin, and
/// `RoundingMode::None` keeps the arithmetic exact for these bounded inputs.
const P: usize = 16384;
const RM: RoundingMode = RoundingMode::None;

/// An exact `BigFloat` for a finite `f64`. `astro-float`'s `from_f64` mishandles
/// subnormals (it assumes the normal implicit leading bit), so a subnormal is
/// lifted into the normal range by an exact power of two and divided back, both
/// steps exact.
fn bf(x: f64) -> BigFloat {
    if x == 0.0 {
        BigFloat::from_f64(0.0, 53)
    } else if x.is_normal() {
        BigFloat::from_f64(x, 53)
    } else {
        let scaled = libm::scalbn(x, 1000);
        let pow = BigFloat::from_f64(libm::scalbn(1.0, 1000), 53);
        BigFloat::from_f64(scaled, 53).div(&pow, P, RM)
    }
}

/// The exact positive magnitude `|x|^k` (`k >= 1`) by exact repeated multiply.
fn exact_mag_pow(x: f64, k: u32) -> BigFloat {
    let mag = bf(x.abs());
    let mut acc = mag.clone();
    for _ in 1..k {
        acc = acc.mul(&mag, P, RM);
    }
    acc
}

fn sign_of(v: &BigFloat) -> Ordering {
    if v.is_zero() {
        Ordering::Equal
    } else if v.is_negative() {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// `sign(c - x^n)`, where `pexact = |x|^|n|` (a positive `BigFloat`) and
/// `x_neg` is the base sign. The infinities are handled without conversion; the
/// finite arithmetic never represents the reciprocal (it compares by the sign of
/// `c * pexact - sign` instead).
fn cmp_c(c: f64, n: i32, x_neg: bool, pexact: &BigFloat) -> Ordering {
    if c == f64::INFINITY {
        return Ordering::Greater;
    }
    if c == f64::NEG_INFINITY {
        return Ordering::Less;
    }
    let neg_val = x_neg && (n & 1 == 1);
    if n > 0 {
        // V = +/- pexact.
        let diff = if neg_val {
            bf(c).add(pexact, P, RM) // c - (-pexact) = c + pexact
        } else {
            bf(c).sub(pexact, P, RM)
        };
        sign_of(&diff)
    } else {
        // V = sign / pexact; sign(c - V) = sign(c * pexact - sign) (pexact > 0).
        let s = if neg_val { -1.0 } else { 1.0 };
        let t = bf(c).mul(pexact, P, RM).sub(&bf(s), P, RM);
        sign_of(&t)
    }
}

fn hx(x: f64) -> String {
    format!("{:#018x}", x.to_bits())
}

/// Assert the directed pair `(down, up)` is the correctly rounded bracket of
/// `x^n` (`1 <= |n| <= 64`).
fn check_correctly_rounded(x: f64, n: i32) {
    let k = n.unsigned_abs();
    let pexact = exact_mag_pow(x, k);
    let x_neg = x.is_sign_negative();
    let down = TightF64(x).pown_down(n).0;
    let up = TightF64(x).pown_up(n).0;
    // The two backends share the kernel; confirm the fixture agrees bit-for-bit.
    assert_eq!(
        x.pown_down(n),
        down,
        "fixture/tight down disagree for {x} ^ {n}"
    );
    assert_eq!(x.pown_up(n), up, "fixture/tight up disagree for {x} ^ {n}");

    assert!(down <= up, "down {} > up {} for {x}^{n}", hx(down), hx(up));
    // down <= V and down is the largest such.
    assert!(
        cmp_c(down, n, x_neg, &pexact) != Ordering::Greater,
        "down {} exceeds {x}^{n}",
        hx(down)
    );
    assert!(
        cmp_c(down.next_up(), n, x_neg, &pexact) == Ordering::Greater,
        "down {} is not the largest float at or below {x}^{n}",
        hx(down)
    );
    // up >= V and up is the smallest such.
    assert!(
        cmp_c(up, n, x_neg, &pexact) != Ordering::Less,
        "up {} is below {x}^{n}",
        hx(up)
    );
    assert!(
        cmp_c(up.next_down(), n, x_neg, &pexact) == Ordering::Less,
        "up {} is not the smallest float at or above {x}^{n}",
        hx(up)
    );
}

/// Assert the directed pair merely brackets `x^n` and is ordered (`|n| > 64`).
fn check_sound(x: f64, n: i32) {
    let k = n.unsigned_abs();
    let pexact = exact_mag_pow(x, k);
    let x_neg = x.is_sign_negative();
    let down = x.pown_down(n);
    let up = x.pown_up(n);
    assert!(down <= up, "down {} > up {} for {x}^{n}", hx(down), hx(up));
    assert!(
        cmp_c(down, n, x_neg, &pexact) != Ordering::Greater,
        "down {} exceeds {x}^{n}",
        hx(down)
    );
    assert!(
        cmp_c(up, n, x_neg, &pexact) != Ordering::Less,
        "up {} is below {x}^{n}",
        hx(up)
    );
}

// ---------------------------------------------------------------------------
// Random-input strategies.
// ---------------------------------------------------------------------------

/// A normal value with exponent in `[lo, hi]` and a random sign.
fn normal_at(lo: i32, hi: i32) -> impl Strategy<Value = f64> {
    (0u64..(1u64 << 52), lo..=hi, any::<bool>()).prop_map(|(frac, e, neg)| {
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

/// A base near one, where large exponents keep the value finite.
fn near_one() -> impl Strategy<Value = f64> {
    normal_at(-3, 3)
}

/// A broad base: ordinary, wide, or near-one magnitude.
fn wide_base() -> impl Strategy<Value = f64> {
    prop_oneof![
        3 => normal_at(-40, 40),
        1 => normal_at(-1020, 1020),
        2 => near_one(),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2500))]

    /// Random finite bases against every exponent magnitude in the exact range.
    #[test]
    fn kernel_is_correctly_rounded(x in wide_base(), k in 1i32..=64, neg in any::<bool>()) {
        let n = if neg { -k } else { k };
        check_correctly_rounded(x, n);
    }

    /// Beyond the exact range the kernel stays sound and ordered.
    #[test]
    fn kernel_is_sound_past_the_bound(x in near_one(), k in 65i32..=200, neg in any::<bool>()) {
        let n = if neg { -k } else { k };
        check_sound(x, n);
    }
}

// ---------------------------------------------------------------------------
// The edge sweep: the special table and boundary magnitudes.
// ---------------------------------------------------------------------------

/// The full 53-bit significand `mant` (leading one included) at corpus exponent.
fn hf(mant: u64, exp: i32) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let m = mant as f64;
    libm::scalbn(m, exp - 52)
}

#[test]
fn special_table_is_exact_and_signed() {
    // pown(x, 0) == 1 for every x.
    for x in [
        0.0,
        -0.0,
        3.5,
        -3.5,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        f64::MAX,
        f64::from_bits(1),
    ] {
        assert_eq!(x.pown_down(0), 1.0);
        assert_eq!(x.pown_up(0), 1.0);
    }
    // NaN propagates for a nonzero exponent.
    for n in [-2, -1, 1, 3, i32::MIN, i32::MAX] {
        assert!(f64::NAN.pown_down(n).is_nan());
        assert!(f64::NAN.pown_up(n).is_nan());
    }
    // Signed zero base.
    assert_eq!((-0.0_f64).pown_down(3).to_bits(), (-0.0_f64).to_bits());
    assert_eq!((-0.0_f64).pown_down(2).to_bits(), 0.0_f64.to_bits());
    assert_eq!((0.0_f64).pown_down(-1), f64::INFINITY);
    assert_eq!((-0.0_f64).pown_down(-1), f64::NEG_INFINITY);
    assert_eq!((-0.0_f64).pown_down(-2), f64::INFINITY);
    // Infinite base.
    assert_eq!(f64::INFINITY.pown_up(2), f64::INFINITY);
    assert_eq!(f64::NEG_INFINITY.pown_up(3), f64::NEG_INFINITY);
    assert_eq!(f64::NEG_INFINITY.pown_up(2), f64::INFINITY);
    assert_eq!(f64::INFINITY.pown_down(-2).to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        f64::NEG_INFINITY.pown_up(-3).to_bits(),
        (-0.0_f64).to_bits()
    );
}

#[test]
fn boundary_magnitudes_are_correctly_rounded() {
    // Powers of two stay exact across a wide exponent span.
    for &(base, n) in &[
        (2.0, 60),
        (2.0, -60),
        (0.5, 40),
        (0.5, -40),
        (8.0, 7),
        (0.125, -5),
    ] {
        check_correctly_rounded(base, n);
    }
    // The largest finite and the smallest subnormal, positive and negative n.
    for &n in &[1, 2, 3, 8, -1, -2, -3, -8, 64, -64] {
        check_correctly_rounded(f64::MAX, n);
        check_correctly_rounded(-f64::MAX, n);
        check_correctly_rounded(f64::from_bits(1), n); // 2^-1074
        check_correctly_rounded(-f64::from_bits(1), n);
        check_correctly_rounded(f64::MIN_POSITIVE, n); // 2^-1022
    }
    // Near-one bases at the exact-range ceiling exercise the long chain.
    for &x in &[
        1.0 + f64::EPSILON,
        1.0 - f64::EPSILON / 2.0,
        hf(0x1_8000_0000_0001, 0),
    ] {
        check_correctly_rounded(x, 64);
        check_correctly_rounded(x, -64);
    }
}

#[test]
fn exponent_extremes_are_sound() {
    // The `i32::MIN` and `i32::MAX` magnitudes exceed the exact range by billions,
    // so the astro-float reference cannot run; assert the determined edges and, at
    // the underflow edge, soundness only (the fallback is loose but never wrong).

    // 1^huge == 1 exactly (the magnitude-one identity, past the exact range).
    assert_eq!(1.0_f64.pown_down(i32::MAX), 1.0);
    assert_eq!(1.0_f64.pown_up(i32::MAX), 1.0);
    assert_eq!(1.0_f64.pown_down(i32::MIN), 1.0);
    assert_eq!(1.0_f64.pown_up(i32::MIN), 1.0);
    assert_eq!((-1.0_f64).pown_down(i32::MAX), -1.0); // i32::MAX is odd
    assert_eq!((-1.0_f64).pown_up(i32::MAX), -1.0);
    assert_eq!((-1.0_f64).pown_down(i32::MIN), 1.0); // i32::MIN is even
    assert_eq!((-1.0_f64).pown_up(i32::MIN), 1.0);

    // A base above one to a huge power overflows: down clamps to the largest
    // finite, up to infinity. Both edges are exact.
    assert_eq!(2.0_f64.pown_down(i32::MAX), f64::MAX);
    assert_eq!(2.0_f64.pown_up(i32::MAX), f64::INFINITY);

    // A base above one to a huge negative power underflows far below the
    // subnormal floor: down is +0 exactly (largest float at or below a positive
    // value that rounds to zero), up is a sound positive bound (loose in the
    // tail, but never below the true value).
    assert_eq!(2.0_f64.pown_down(i32::MIN).to_bits(), 0.0_f64.to_bits());
    let up = 2.0_f64.pown_up(i32::MIN);
    assert!(
        up > 0.0 && up < 1e-300,
        "underflow up bound unsound or absurd: {up}"
    );
    // The mirror across the exponent sign: 0.5 to a huge positive power underflows.
    assert_eq!(0.5_f64.pown_down(i32::MAX).to_bits(), 0.0_f64.to_bits());
    let up2 = 0.5_f64.pown_up(i32::MAX);
    assert!(
        up2 > 0.0 && up2 < 1e-300,
        "underflow up bound unsound or absurd: {up2}"
    );
}
