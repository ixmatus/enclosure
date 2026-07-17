//! Shared test support for the tight `f64` backend: proptest input
//! distributions and an independent integer reference oracle.
//!
//! The oracle computes correctly rounded directed results for `add`, `mul`,
//! `div`, and `sqrt` with exact integer arithmetic that shares no code with the
//! error-free-transform paths under test. Every `f64` is decomposed into an
//! exact dyadic value `(-1)^neg * m * 2^e` (`m` a 53-bit integer significand),
//! and the sign of `exact - candidate` is decided by exact big-integer
//! comparison. A candidate float is accepted as the directed rounding exactly
//! when that sign check confirms it, so the oracle's correctness rests only on
//! integer comparison, not on any float rounding it might use to seed the search.
//!
//! Because each `tests/*.rs` file compiles this module as its own copy, not
//! every helper is used by every file; `dead_code` is allowed for that reason.
#![allow(dead_code)]

use core::cmp::Ordering;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Exact dyadic decomposition and big-integer comparison.
// ---------------------------------------------------------------------------

/// An exact value `(-1)^neg * m * 2^e`. A zero has `m == 0`.
#[derive(Clone, Copy, Debug)]
pub struct Dyadic {
    pub neg: bool,
    pub m: u128,
    pub e: i32,
}

/// Decompose a finite `f64` into its exact dyadic value.
pub fn dyad(x: f64) -> Dyadic {
    debug_assert!(x.is_finite());
    let bits = x.to_bits();
    let neg = (bits >> 63) & 1 == 1;
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let frac = u128::from(bits & ((1u64 << 52) - 1));
    if biased == 0 {
        // Subnormal (or zero when frac == 0): value = frac * 2^-1074.
        Dyadic {
            neg,
            m: frac,
            e: -1074,
        }
    } else {
        // Normal: value = (2^52 + frac) * 2^(biased - 1075).
        Dyadic {
            neg,
            m: frac | (1u128 << 52),
            e: biased - 1075,
        }
    }
}

/// Exact product of two dyadic values (`m` stays within `u128`: 53 + 53 bits).
fn mul_dyad(a: Dyadic, b: Dyadic) -> Dyadic {
    Dyadic {
        neg: a.neg ^ b.neg,
        m: a.m * b.m,
        e: a.e + b.e,
    }
}

/// Flip the sign of a dyadic value.
fn neg_dyad(a: Dyadic) -> Dyadic {
    Dyadic { neg: !a.neg, ..a }
}

/// A nonnegative big integer, little-endian `u64` limbs, no trailing zero limb.
#[derive(Clone)]
struct UBig {
    limbs: Vec<u64>,
}

impl UBig {
    fn zero() -> Self {
        UBig { limbs: Vec::new() }
    }

    /// `self += word * 2^(64 * pos)`, propagating carries.
    fn add_word_at(&mut self, word: u64, pos: usize) {
        if word == 0 {
            return;
        }
        while self.limbs.len() <= pos {
            self.limbs.push(0);
        }
        let mut carry = word;
        let mut i = pos;
        while carry != 0 {
            if i == self.limbs.len() {
                self.limbs.push(0);
            }
            let (sum, overflow) = self.limbs[i].overflowing_add(carry);
            self.limbs[i] = sum;
            carry = u64::from(overflow);
            i += 1;
        }
    }

    /// `self += m * 2^shift_bits`.
    // The `u128 -> u64` casts deliberately slice `m` into 64-bit limbs (low word
    // by truncation, high word after a shift), so truncation is intended.
    #[allow(clippy::cast_possible_truncation)]
    fn add_shifted_u128(&mut self, m: u128, shift_bits: u64) {
        if m == 0 {
            return;
        }
        let limb_shift = (shift_bits / 64) as usize;
        let bit = (shift_bits % 64) as u32;
        let m_lo = m as u64;
        let m_hi = (m >> 64) as u64;
        let (w0, w1, w2) = if bit == 0 {
            (m_lo, m_hi, 0u64)
        } else {
            (
                m_lo << bit,
                (m_hi << bit) | (m_lo >> (64 - bit)),
                m_hi >> (64 - bit),
            )
        };
        self.add_word_at(w0, limb_shift);
        self.add_word_at(w1, limb_shift + 1);
        self.add_word_at(w2, limb_shift + 2);
    }

    /// Length ignoring trailing zero limbs.
    fn effective_len(&self) -> usize {
        let mut n = self.limbs.len();
        while n > 0 && self.limbs[n - 1] == 0 {
            n -= 1;
        }
        n
    }

    fn cmp(&self, other: &UBig) -> Ordering {
        let la = self.effective_len();
        let lb = other.effective_len();
        if la != lb {
            return la.cmp(&lb);
        }
        for i in (0..la).rev() {
            if self.limbs[i] != other.limbs[i] {
                return self.limbs[i].cmp(&other.limbs[i]);
            }
        }
        Ordering::Equal
    }
}

/// The sign of `sum(terms)` where each term is a signed dyadic value. Returns
/// `Greater` if the sum is positive, `Less` if negative, `Equal` if zero.
fn dyadic_terms_sign(terms: &[Dyadic]) -> Ordering {
    let mut emin = i32::MAX;
    for t in terms {
        if t.m != 0 {
            emin = emin.min(t.e);
        }
    }
    if emin == i32::MAX {
        return Ordering::Equal; // every term is zero
    }
    let mut pos = UBig::zero();
    let mut neg = UBig::zero();
    for t in terms {
        if t.m == 0 {
            continue;
        }
        // t.e >= emin by construction, so the difference is nonnegative.
        let shift = u64::from((t.e - emin).unsigned_abs());
        if t.neg {
            neg.add_shifted_u128(t.m, shift);
        } else {
            pos.add_shifted_u128(t.m, shift);
        }
    }
    pos.cmp(&neg)
}

// ---------------------------------------------------------------------------
// Exact "sign of (target - candidate)" for each operation.
// ---------------------------------------------------------------------------

/// `sign((a + b) - r)`, exact.
fn cmp_add(a: f64, b: f64, r: f64) -> Ordering {
    dyadic_terms_sign(&[dyad(a), dyad(b), neg_dyad(dyad(r))])
}

/// `sign((a * b) - r)`, exact.
fn cmp_mul(a: f64, b: f64, r: f64) -> Ordering {
    dyadic_terms_sign(&[mul_dyad(dyad(a), dyad(b)), neg_dyad(dyad(r))])
}

/// `sign((a / b) - r)`, exact. `b != 0`.
fn cmp_div(a: f64, b: f64, r: f64) -> Ordering {
    // sign(a/b - r) = sign(a - r*b) * sign(b).
    let s = dyadic_terms_sign(&[dyad(a), neg_dyad(mul_dyad(dyad(r), dyad(b)))]);
    if b < 0.0 {
        s.reverse()
    } else {
        s
    }
}

/// `sign(sqrt(a) - r)`, exact. `a >= 0`.
fn cmp_sqrt(a: f64, r: f64) -> Ordering {
    if r < 0.0 {
        return Ordering::Greater; // sqrt(a) >= 0 > r
    }
    // r >= 0: sqrt increasing, so sign(sqrt(a) - r) = sign(a - r*r).
    dyadic_terms_sign(&[dyad(a), neg_dyad(mul_dyad(dyad(r), dyad(r)))])
}

// ---------------------------------------------------------------------------
// Bracket-and-verify directed rounding, driven by the exact sign checks.
// ---------------------------------------------------------------------------

fn clamp_finite(x: f64) -> f64 {
    if x == f64::INFINITY {
        f64::MAX
    } else if x == f64::NEG_INFINITY {
        -f64::MAX
    } else {
        x
    }
}

/// The largest float `r` with `r <= target`, where `tcmp(r) = sign(target - r)`.
/// `approx` seeds the search near the answer; correctness comes from `tcmp`.
fn round_down(approx: f64, tcmp: &dyn Fn(f64) -> Ordering) -> f64 {
    let mut r = clamp_finite(approx);
    // Step down while target < r.
    while tcmp(r) == Ordering::Less {
        let p = r.next_down();
        if p == f64::NEG_INFINITY {
            return f64::NEG_INFINITY; // target is below -MAX
        }
        r = p;
    }
    // Climb while the next float up is still <= target.
    loop {
        let nu = r.next_up();
        if nu == f64::INFINITY {
            break;
        }
        if tcmp(nu) == Ordering::Less {
            break; // next float up already exceeds the target
        }
        r = nu;
    }
    r
}

/// The smallest float `r` with `r >= target`.
fn round_up(approx: f64, tcmp: &dyn Fn(f64) -> Ordering) -> f64 {
    let mut r = clamp_finite(approx);
    // Step up while target > r.
    while tcmp(r) == Ordering::Greater {
        let p = r.next_up();
        if p == f64::INFINITY {
            return f64::INFINITY; // target is above MAX
        }
        r = p;
    }
    // Descend while the next float down is still >= target.
    loop {
        let nd = r.next_down();
        if nd == f64::NEG_INFINITY {
            break;
        }
        if tcmp(nd) == Ordering::Greater {
            break; // next float down is still below the target
        }
        r = nd;
    }
    r
}

/// Correctly rounded directed bounds for `a + b` (both finite).
pub fn oracle_add(a: f64, b: f64) -> (f64, f64) {
    let approx = a + b;
    let down = round_down(approx, &move |r| cmp_add(a, b, r));
    let up = round_up(approx, &move |r| cmp_add(a, b, r));
    (down, up)
}

/// Correctly rounded directed bounds for `a * b` (both finite).
pub fn oracle_mul(a: f64, b: f64) -> (f64, f64) {
    let approx = a * b;
    let down = round_down(approx, &move |r| cmp_mul(a, b, r));
    let up = round_up(approx, &move |r| cmp_mul(a, b, r));
    (down, up)
}

/// Correctly rounded directed bounds for `a / b` (finite, `b != 0`).
pub fn oracle_div(a: f64, b: f64) -> (f64, f64) {
    let approx = a / b;
    let down = round_down(approx, &move |r| cmp_div(a, b, r));
    let up = round_up(approx, &move |r| cmp_div(a, b, r));
    (down, up)
}

/// Correctly rounded directed bounds for `sqrt(a)` (`a >= 0`, finite).
pub fn oracle_sqrt(a: f64) -> (f64, f64) {
    let approx = libm::sqrt(a);
    let down = round_down(approx, &move |r| cmp_sqrt(a, r));
    let up = round_up(approx, &move |r| cmp_sqrt(a, r));
    (down, up)
}

// ---------------------------------------------------------------------------
// Input distributions.
// ---------------------------------------------------------------------------

/// Any finite `f64` from a uniform draw over the bit pattern, retried until
/// finite. Covers the whole magnitude range including subnormals.
pub fn any_finite() -> impl Strategy<Value = f64> {
    any::<u64>()
        .prop_map(f64::from_bits)
        .prop_filter("finite", |x| x.is_finite())
}

/// A subnormal or near-subnormal magnitude with a random sign: a 52-bit fraction
/// at the fixed subnormal exponent, so both-subnormal and one-subnormal operand
/// pairs are dense.
pub fn subnormal_ish() -> impl Strategy<Value = f64> {
    (any::<bool>(), 0u64..(1u64 << 53)).prop_map(|(neg, frac)| {
        let x = f64::from_bits(frac); // biased exp 0 or 1: subnormal or smallest normals
        if neg {
            -x
        } else {
            x
        }
    })
}

/// A value within a few ulps of a power of two, where neighbor gaps are
/// asymmetric across the binade boundary.
pub fn near_pow2() -> impl Strategy<Value = f64> {
    (-1060i32..1020i32, -4i64..=4i64, any::<bool>()).prop_map(|(k, step, neg)| {
        let base = libm::scalbn(1.0, k);
        let mut v = base;
        // Walk a few ulps off the power of two.
        if step >= 0 {
            for _ in 0..step {
                v = v.next_up();
            }
        } else {
            for _ in 0..(-step) {
                v = v.next_down();
            }
        }
        if neg {
            -v
        } else {
            v
        }
    })
}

/// A value near the overflow shoulder just below `MAX`.
pub fn overflow_shoulder() -> impl Strategy<Value = f64> {
    (0u64..(1u64 << 20), any::<bool>()).prop_map(|(steps, neg)| {
        let mut v = f64::MAX;
        for _ in 0..steps {
            v = v.next_down();
        }
        if neg {
            -v
        } else {
            v
        }
    })
}

/// An ordinary-magnitude value, the common case.
pub fn ordinary() -> impl Strategy<Value = f64> {
    prop_oneof![-1.0f64..1.0, -1.0e6f64..1.0e6, -1.0e18f64..1.0e18,]
}

/// A broad mixture over all the regimes above.
pub fn wide_finite() -> impl Strategy<Value = f64> {
    prop_oneof![
        3 => any_finite(),
        2 => ordinary(),
        2 => subnormal_ish(),
        2 => near_pow2(),
        1 => overflow_shoulder(),
    ]
}
