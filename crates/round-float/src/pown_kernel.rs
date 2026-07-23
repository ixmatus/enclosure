//! The exact integer-power kernel behind [`RoundPown`](crate::RoundPown).
//!
//! Both shipped `f64` backends (`f64` fixture and `TightF64`) route integer
//! power through this one module. The kernel is exact: it decomposes a finite
//! nonzero base into an integer significand and a power of two, raises the
//! significand to the integer exponent over a fixed-width limb array with no
//! rounding anywhere in the chain, and rounds ONCE at the end in the requested
//! direction. Both directed results are therefore correctly rounded; there is
//! no error term to bound.
//!
//! # The decomposition
//!
//! A finite nonzero `f64` magnitude is `M * 2^E` for an integer significand
//! `M` in `[1, 2^53)` and an integer `E`. A normal reads `M = 2^52 | frac`,
//! `E = biased - 1075`; a subnormal reads `M = frac`, `E = -1074`. The exact
//! value of `|x|^k` is then `M^k * 2^(E*k)`, and the exact value of `|x|^-k` is
//! `2^(-E*k) / M^k`. `M^k` is a nonnegative integer of at most `53 * k` bits,
//! held in a little-endian `u64` limb array and built by `k - 1` exact
//! multiplications of the array by the `u64` significand `M` (integer-exact:
//! every partial product fits a `u128`, and carries propagate without loss).
//!
//! # Rounding once, exactly
//!
//! For a positive exponent the exact value is the dyadic integer `P = M^k`
//! scaled by `2^(E*k)`. The kernel reads the top 53 significand bits of `P`
//! (the result significand), records whether any lower bit is set (the sticky
//! bit), and places the binary point from exact `i64` exponent arithmetic:
//! `result_leading_bit = bit_len(P) - 1 + E*k`. Truncation toward zero gives
//! the down bound; if the sticky bit is set the up bound is the adjacent float
//! (`next_up`), else the value is exact and the two bounds coincide. A leading
//! bit beyond the finite range clamps to the largest finite (down) and the
//! infinity (up); a leading bit below `-1022` denormalizes by shifting into the
//! same sticky machinery, so subnormal underflow is exact too.
//!
//! For a negative exponent the exact value is `2^(-E*k) / P`, a rational the
//! kernel resolves by exact long division: it forms the quotient's top 53 bits
//! and a remainder-driven sticky bit by dividing a power of two by `P`. The
//! same truncate-or-step rounding then applies. An overflowing magnitude power
//! (the mechanism the pre-kernel path collapsed, bead enc-5jj) now rides the
//! exact exponent arithmetic and denormalizes cleanly to the subnormal floor.
//!
//! # The exact range and the sound tail
//!
//! Correct rounding holds for `1 <= |n| <= POWN_TIGHT_MAX`. The significand of
//! `x^n` carries `53 * |n|` bits, so unbounded `|n|` has no bounded-memory
//! exact form (a near-one base keeps astronomically large powers finite).
//! Beyond the bound the kernel falls back to a directed repeated-squaring
//! chain, sound but not tightest, and [`RoundPown`](crate::RoundPown) says so.
//! See round-float decision record 0004.

/// The largest exponent magnitude the kernel rounds correctly (tightest).
///
/// `64` covers every plausible rigorous-computing integer power at a bounded
/// cost (the operand buffer is a few hundred bytes of stack, no allocation).
/// Beyond it the fallback chain is sound but not tightest.
pub(crate) const POWN_TIGHT_MAX: u32 = 64;

/// Little-endian limb count. `M^k` for `M < 2^53` and `k <= 64` carries at most
/// `53 * 64 = 3392` bits (53 limbs); a carry from the final multiply reaches
/// limb 53, and the reciprocal remainder reaches limb 53 after a shift, so 54
/// limbs are load-bearing. The buffer holds 56 with headroom.
const N_LIMBS: usize = 56;

/// A fixed-width nonnegative little-endian big integer. `len` is the number of
/// significant limbs (the highest nonzero limb plus one); a zero has `len == 0`.
#[derive(Clone)]
struct Big {
    limbs: [u64; N_LIMBS],
    len: usize,
}

impl Big {
    /// Zero.
    fn zero() -> Self {
        Big {
            limbs: [0; N_LIMBS],
            len: 0,
        }
    }

    /// The single-limb value `x`.
    fn from_u64(x: u64) -> Self {
        let mut b = Big::zero();
        if x != 0 {
            b.limbs[0] = x;
            b.len = 1;
        }
        b
    }

    /// The power of two `2^n`.
    fn pow2(n: usize) -> Self {
        let mut b = Big::zero();
        let limb = n / 64;
        let off = n % 64;
        debug_assert!(limb < N_LIMBS, "pow2 exceeds the limb buffer");
        b.limbs[limb] = 1u64 << off;
        b.len = limb + 1;
        b
    }

    /// Whether the value is zero.
    fn is_zero(&self) -> bool {
        self.len == 0
    }

    /// The position of the highest set bit plus one, or zero for a zero value.
    fn bit_len(&self) -> u64 {
        if self.len == 0 {
            return 0;
        }
        let top = self.limbs[self.len - 1];
        (self.len as u64 - 1) * 64 + (64 - u64::from(top.leading_zeros()))
    }

    /// Multiply in place by the `u64` factor `m` (exact; `m >= 1` at call sites).
    // `prod as u64` keeps the low limb; the carry is `prod >> 64`, which fits u64
    // (`prod < (2^64) * (2^53)` at these call sites), so neither cast loses data.
    #[allow(clippy::cast_possible_truncation)]
    fn mul_u64(&mut self, m: u64) {
        if self.len == 0 {
            return;
        }
        let mut carry: u64 = 0;
        for i in 0..self.len {
            let prod = u128::from(self.limbs[i]) * u128::from(m) + u128::from(carry);
            self.limbs[i] = prod as u64;
            carry = (prod >> 64) as u64;
        }
        if carry != 0 {
            debug_assert!(self.len < N_LIMBS, "multiply carry exceeds the limb buffer");
            self.limbs[self.len] = carry;
            self.len += 1;
        }
    }

    /// The `count` bits `[start, start + count)` as a `u64` (`count <= 64`).
    fn bits_from(&self, start: usize, count: usize) -> u64 {
        let mut result: u64 = 0;
        for i in 0..count {
            let pos = start + i;
            let limb = pos / 64;
            let off = pos % 64;
            let bit = if limb < self.len {
                (self.limbs[limb] >> off) & 1
            } else {
                0
            };
            result |= bit << i;
        }
        result
    }

    /// Whether any bit below position `shift` is set.
    fn low_nonzero(&self, shift: usize) -> bool {
        let full = shift / 64;
        for i in 0..full.min(self.len) {
            if self.limbs[i] != 0 {
                return true;
            }
        }
        let rem = shift % 64;
        if rem > 0 && full < self.len {
            let mask = (1u64 << rem) - 1;
            if self.limbs[full] & mask != 0 {
                return true;
            }
        }
        false
    }

    /// Shift left by one bit in place (exact; may grow `len`).
    fn shl1(&mut self) {
        if self.len == 0 {
            return;
        }
        let mut carry: u64 = 0;
        for i in 0..self.len {
            let new_carry = self.limbs[i] >> 63;
            self.limbs[i] = (self.limbs[i] << 1) | carry;
            carry = new_carry;
        }
        if carry != 0 {
            debug_assert!(self.len < N_LIMBS, "shift carry exceeds the limb buffer");
            self.limbs[self.len] = carry;
            self.len += 1;
        }
    }

    /// Compare magnitudes.
    fn cmp(&self, other: &Big) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        if self.len != other.len {
            return self.len.cmp(&other.len);
        }
        for i in (0..self.len).rev() {
            if self.limbs[i] != other.limbs[i] {
                return self.limbs[i].cmp(&other.limbs[i]);
            }
        }
        Ordering::Equal
    }

    /// Subtract `other` in place (the caller guarantees `self >= other`).
    fn sub_assign(&mut self, other: &Big) {
        let mut borrow: u64 = 0;
        for i in 0..self.len {
            let rhs = if i < other.len { other.limbs[i] } else { 0 };
            let (d1, b1) = self.limbs[i].overflowing_sub(rhs);
            let (d2, b2) = d1.overflowing_sub(borrow);
            self.limbs[i] = d2;
            borrow = u64::from(b1 || b2);
        }
        debug_assert!(borrow == 0, "subtraction underflowed");
        while self.len > 0 && self.limbs[self.len - 1] == 0 {
            self.len -= 1;
        }
    }
}

/// Decompose a finite nonzero positive `f64` into `(sig, exp)` with
/// `mag = sig * 2^exp`, `sig` an integer in `[1, 2^53)`.
// The 11-bit exponent field is far inside `i64`, so the cast never wraps.
#[allow(clippy::cast_possible_wrap)]
fn decompose(mag: f64) -> (u64, i64) {
    let bits = mag.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i64;
    let frac = bits & ((1u64 << 52) - 1);
    if biased == 0 {
        // Subnormal: value = frac * 2^-1074.
        (frac, -1074)
    } else {
        // Normal: value = (2^52 + frac) * 2^(biased - 1075).
        ((1u64 << 52) | frac, biased - 1075)
    }
}

/// Build `P = sig^k` as an exact big integer (`k >= 1`).
fn pow_significand(sig: u64, k: u32) -> Big {
    let mut prod = Big::from_u64(sig);
    for _ in 1..k {
        prod.mul_u64(sig);
    }
    prod
}

/// The directed magnitude pair `(down, up)` for a value whose truncated-toward-
/// zero significand is `q` at quantum exponent `qe`, with `sticky` recording
/// whether the exact value exceeds `q * 2^qe`.
///
/// `q * 2^qe` is a representable `f64` grid point by construction (a normal
/// significand in `[2^52, 2^53)` at `qe`, or a subnormal `q < 2^52` at
/// `qe = -1074`). Overflow of the down bound means the exact value exceeds the
/// largest finite, so the pair is `(MAX, +inf)`. Otherwise the up bound is the
/// down bound when the value is exact and its `next_up` neighbor when sticky
/// (the two are adjacent floats, so `next_up` is exactly `(q + 1) * 2^qe`, the
/// binade carry included).
fn round_magnitude(q: u64, qe: i64, sticky: bool) -> (f64, f64) {
    debug_assert!(q < (1u64 << 53), "significand exceeds 53 bits");
    // qe is bounded well inside i32 for every finite decomposition; a value that
    // scalbn drives out of range returns infinity, handled next.
    #[allow(clippy::cast_possible_truncation)] // clamped into the i32 range first
    let exp = qe.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
    #[allow(clippy::cast_precision_loss)] // q < 2^53, exact
    let down = libm::scalbn(q as f64, exp);
    if down.is_infinite() {
        // The truncated value already exceeds the largest finite.
        return (f64::MAX, f64::INFINITY);
    }
    let up = if sticky { down.next_up() } else { down };
    (down, up)
}

/// The directed magnitude pair for `|x|^k`, `k >= 1`, from the exact decomposition.
// `bit_len()` returns at most ~3392, far inside i64; the two `i64 as usize`
// casts read a nonnegative `shift` (or its negation), safe on 64-bit targets.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn round_pos(sig: u64, exp: i64, k: u32) -> (f64, f64) {
    let prod = pow_significand(sig, k);
    let ek = exp * i64::from(k);
    let nbits = prod.bit_len() as i64; // >= 1
    let leading = nbits - 1 + ek; // exponent of the result's leading bit
    let qe = (leading - 52).max(-1074);
    let shift = qe - ek;
    let (q, sticky) = if shift >= 0 {
        let shift_u = shift as usize;
        (prod.bits_from(shift_u, 53), prod.low_nonzero(shift_u))
    } else {
        // The exact value fits the grid with room to spare: shift left, no sticky.
        let q = prod.bits_from(0, 53) << (-shift as usize);
        (q, false)
    };
    round_magnitude(q, qe, sticky)
}

/// The directed magnitude pair for `|x|^-k`, `k >= 1`, from the exact
/// decomposition. The exact value is `2^(-exp*k) / P`, resolved by exact long
/// division of a power of two by `P`.
// `bit_len()` is far inside i64; `q >> 53` isolates the power-of-two carry bit.
#[allow(clippy::cast_possible_wrap)]
fn round_recip(sig: u64, exp: i64, k: u32) -> (f64, f64) {
    let prod = pow_significand(sig, k);
    let ek = exp * i64::from(k);
    let nbits = prod.bit_len() as i64; // >= 1
                                       // W = 2^(-ek) / P has leading bit at -ek - nbits (one higher iff P is a power
                                       // of two, corrected by the significand carry below).
    let leading = -ek - nbits;
    let mut qe = (leading - 52).max(-1074);
    let num_exp = -ek - qe; // the numerator power: q = floor(2^num_exp / P)
    let (mut q, sticky) = div_pow2(num_exp, &prod, nbits);
    if q >> 53 != 0 {
        // Power-of-two P: the quotient carried to 2^53. Halve and lift the
        // quantum; the value is exact (no remainder), so sticky stays false.
        q >>= 1;
        qe += 1;
    }
    round_magnitude(q, qe, sticky)
}

/// `(floor(2^num_exp / P), remainder != 0)` for `P` of bit length `nbits >= 1`.
///
/// Long division brings the single numerator bit down and then `num_exp` zero
/// bits. The remainder first reaches `P`-scale after `nbits - 1` doublings, so
/// the first `nbits - 1` quotient bits are zero and are skipped: the loop starts
/// with the remainder preloaded to `2^(nbits-1)`. The quotient carries at most
/// `num_exp - nbits + 2` significant bits, which fits a `u64` for every call.
// `num_exp - nbits + 2` and `nbits - 1` are nonnegative here (the first guarded
// above, the second since `nbits >= 1`), so the `i64 as usize` casts are safe.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn div_pow2(num_exp: i64, p: &Big, nbits: i64) -> (u64, bool) {
    if num_exp < nbits - 1 {
        // 2^num_exp < 2^(nbits-1) <= P: the quotient is zero and 2^num_exp > 0
        // is the remainder.
        return (0, true);
    }
    let steps = (num_exp - nbits + 2) as usize; // >= 1
    let mut rem = Big::pow2((nbits - 1) as usize);
    let mut q: u64 = 0;
    for step in 0..steps {
        if rem.cmp(p) == core::cmp::Ordering::Less {
            q <<= 1;
        } else {
            rem.sub_assign(p);
            q = (q << 1) | 1;
        }
        if step + 1 < steps {
            rem.shl1();
        }
    }
    (q, !rem.is_zero())
}

// ---------------------------------------------------------------------------
// The sound-but-loose fallback for |n| > POWN_TIGHT_MAX.
// ---------------------------------------------------------------------------

/// A lower bound on the product `a * b` for nonnegative `a`, `b` (round toward
/// minus infinity by a nearest multiply and one outward step), clamped to the
/// nonnegative floor. The clamp is sound because `a * b >= 0`, and it keeps the
/// chain from crossing zero when a product underflows (`next_down(+0)` is a
/// negative subnormal), which would otherwise make the reciprocal path blow up.
fn mul_down(a: f64, b: f64) -> f64 {
    let r = (a * b).next_down();
    if r < 0.0 {
        0.0
    } else {
        r
    }
}

/// An upper bound on the product `a * b` for nonnegative `a`, `b`.
fn mul_up(a: f64, b: f64) -> f64 {
    (a * b).next_up()
}

/// A lower bound on `mag^k` for `mag >= 0`, `k >= 1`, by directed binary
/// exponentiation (every operand nonnegative, so the chain never rounds the
/// wrong way).
fn abs_pow_down(mag: f64, k: u32) -> f64 {
    let mut result = 1.0f64;
    let mut base = mag;
    let mut e = k;
    while e > 0 {
        if e & 1 == 1 {
            result = mul_down(result, base);
        }
        e >>= 1;
        if e > 0 {
            base = mul_down(base, base);
        }
    }
    result
}

/// An upper bound on `mag^k`, the companion of [`abs_pow_down`].
fn abs_pow_up(mag: f64, k: u32) -> f64 {
    let mut result = 1.0f64;
    let mut base = mag;
    let mut e = k;
    while e > 0 {
        if e & 1 == 1 {
            result = mul_up(result, base);
        }
        e >>= 1;
        if e > 0 {
            base = mul_up(base, base);
        }
    }
    result
}

/// The directed magnitude pair for `mag^n`, `mag > 0`, when `|n|` exceeds the
/// exact range. Sound but not tightest.
// The `== 0.0` tests are exact overflow/underflow-edge detections, not
// approximate comparisons.
#[allow(clippy::float_cmp)]
fn fallback(mag: f64, n: i32) -> (f64, f64) {
    let k = n.unsigned_abs();
    let lo = abs_pow_down(mag, k);
    let hi = abs_pow_up(mag, k);
    if n > 0 {
        (lo, hi)
    } else {
        // Reciprocal of the magnitude bounds [lo, hi] (both >= 0).
        let down = if hi.is_infinite() {
            0.0
        } else if hi == 0.0 {
            f64::INFINITY
        } else {
            (1.0 / hi).next_down().max(0.0)
        };
        let up = if lo == 0.0 {
            f64::INFINITY
        } else if lo.is_infinite() {
            0.0
        } else {
            (1.0 / lo).next_up()
        };
        (down, up)
    }
}

/// The directed pair `(down, up)` bracketing `x^n` (IEEE 754-2019 `pown`) for a
/// scalar `f64` base.
///
/// The special values follow clause 9.2: `pown(x, 0) == 1` for every `x`
/// (including NaN and the infinities); a nonzero `n` propagates NaN; a zero base
/// gives a signed zero for `n > 0` (the sign kept only for an odd power) and a
/// signed infinity for `n < 0` (division semantics); an infinite base mirrors
/// that (infinity for `n > 0`, a signed zero for `n < 0`). A finite nonzero base
/// rides the exact kernel for `|n| <= POWN_TIGHT_MAX` and the fallback beyond.
/// Every special result is exact, so its down and up bounds coincide.
// The `x == 0.0` and `mag == 1.0` tests are exact special-case detections.
#[allow(clippy::float_cmp)]
fn pown_parts(x: f64, n: i32) -> (f64, f64) {
    if n == 0 {
        return (1.0, 1.0);
    }
    if x.is_nan() {
        return (x, x);
    }
    if x == 0.0 {
        let neg_odd = x.is_sign_negative() && (n & 1 == 1);
        let v = if n > 0 {
            if neg_odd {
                -0.0
            } else {
                0.0
            }
        } else if neg_odd {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        return (v, v);
    }
    if x.is_infinite() {
        let neg_odd = x.is_sign_negative() && (n & 1 == 1);
        let v = if n > 0 {
            if neg_odd {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        } else if neg_odd {
            -0.0
        } else {
            0.0
        };
        return (v, v);
    }
    // Finite nonzero base.
    let neg_base = x.is_sign_negative();
    let result_negative = neg_base && (n & 1 == 1);
    let mag = if neg_base { -x } else { x };
    let k = n.unsigned_abs(); // handles i32::MIN without overflow
    let (dmag, umag) = if mag == 1.0 {
        // 1^n = 1 exactly for every n; the identity holds past the exact range,
        // where the fallback chain would otherwise drift off it.
        (1.0, 1.0)
    } else if k <= POWN_TIGHT_MAX {
        let (sig, exp) = decompose(mag);
        if n > 0 {
            round_pos(sig, exp, k)
        } else {
            round_recip(sig, exp, k)
        }
    } else {
        fallback(mag, n)
    };
    if result_negative {
        // x^n = -(|x|^n): the lower bound negates the upper magnitude bound.
        (-umag, -dmag)
    } else {
        (dmag, umag)
    }
}

/// A lower bound on `x^n` (IEEE 754-2019 `pown`), correctly rounded for
/// `1 <= |n| <= POWN_TIGHT_MAX` and sound beyond.
pub(crate) fn pown_down(x: f64, n: i32) -> f64 {
    pown_parts(x, n).0
}

/// An upper bound on `x^n` (IEEE 754-2019 `pown`), the companion of
/// [`pown_down`].
pub(crate) fn pown_up(x: f64, n: i32) -> f64 {
    pown_parts(x, n).1
}

#[cfg(test)]
mod tests {
    // The expected values are exact and compared bit-for-bit; the corpus-derived
    // hex significands are transcribed verbatim, so the strict-equality and
    // literal-separator lints are waived here as in the conformance lane.
    #![allow(clippy::float_cmp, clippy::unreadable_literal)]

    use super::*;

    /// The `f64` for `1.<frac> * 2^exp` from the full 53-bit significand `mant`
    /// (leading one included) and the corpus `P` exponent.
    fn hf(mant: u64, exp: i32) -> f64 {
        assert!(mant < (1 << 53));
        #[allow(clippy::cast_precision_loss)]
        let m = mant as f64;
        libm::scalbn(m, exp - 52)
    }

    fn bits(x: f64) -> u64 {
        x.to_bits()
    }

    // The IEEE 754-2019 clause 9.2 special table. Every special result is exact,
    // so the down and up bounds coincide and pin bit-for-bit.

    #[test]
    fn exponent_zero_is_one_everywhere() {
        for x in [
            0.0,
            -0.0,
            1.0,
            -3.5,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MAX,
            f64::from_bits(1),
        ] {
            assert_eq!(pown_down(x, 0), 1.0);
            assert_eq!(pown_up(x, 0), 1.0);
        }
    }

    #[test]
    fn nan_propagates_for_nonzero_exponent() {
        for n in [-3, -1, 1, 2, 5] {
            assert!(pown_down(f64::NAN, n).is_nan());
            assert!(pown_up(f64::NAN, n).is_nan());
        }
    }

    #[test]
    fn zero_base_positive_exponent_keeps_sign_for_odd() {
        assert_eq!(bits(pown_down(0.0, 3)), bits(0.0));
        assert_eq!(bits(pown_down(-0.0, 3)), bits(-0.0)); // -0^odd = -0
        assert_eq!(bits(pown_down(-0.0, 2)), bits(0.0)); // -0^even = +0
        assert_eq!(bits(pown_up(-0.0, 3)), bits(-0.0));
    }

    #[test]
    fn zero_base_negative_exponent_is_signed_infinity() {
        assert_eq!(pown_down(0.0, -1), f64::INFINITY);
        assert_eq!(pown_down(-0.0, -1), f64::NEG_INFINITY); // -0^-odd = -inf
        assert_eq!(pown_down(-0.0, -2), f64::INFINITY); // -0^-even = +inf
    }

    #[test]
    fn infinite_base() {
        assert_eq!(pown_down(f64::INFINITY, 2), f64::INFINITY);
        assert_eq!(pown_down(f64::NEG_INFINITY, 3), f64::NEG_INFINITY);
        assert_eq!(pown_down(f64::NEG_INFINITY, 2), f64::INFINITY);
        assert_eq!(bits(pown_up(f64::INFINITY, -2)), bits(0.0)); // +inf^-n = +0
        assert_eq!(bits(pown_up(f64::NEG_INFINITY, -3)), bits(-0.0)); // -inf^-odd = -0
    }

    // Exact-kernel values cross-checked against the ITF1788 corpus literals.

    #[test]
    fn positive_powers_are_correctly_rounded() {
        // 13.1^2
        assert_eq!(pown_down(13.1, 2), hf(0x1573851EB851EB, 7));
        assert_eq!(pown_up(13.1, 2), hf(0x1573851EB851EC, 7));
        // 13.1^8
        assert_eq!(pown_down(13.1, 8), hf(0x19D8FD495853F5, 29));
        assert_eq!(pown_up(13.1, 8), hf(0x19D8FD495853F6, 29));
        // (-7451.145)^3 is negative
        assert_eq!(pown_down(-7451.145, 3), -hf(0x181460637B9A3D, 38));
        assert_eq!(pown_up(-7451.145, 3), -hf(0x181460637B9A3C, 38));
        // 2.5^3 = 15.625 exactly (down == up)
        assert_eq!(pown_down(2.5, 3), 15.625);
        assert_eq!(pown_up(2.5, 3), 15.625);
    }

    #[test]
    fn negative_powers_are_correctly_rounded() {
        // 13.1^-1
        assert_eq!(pown_down(13.1, -1), hf(0x138ABF82EE6986, -4));
        assert_eq!(pown_up(13.1, -1), hf(0x138ABF82EE6987, -4));
        // 13.1^-2
        assert_eq!(pown_down(13.1, -2), hf(0x17DE3A077D1568, -8));
        assert_eq!(pown_up(13.1, -2), hf(0x17DE3A077D1569, -8));
    }

    #[test]
    fn overflow_clamps_to_max_and_infinity() {
        // MAX^2 overflows: down clamps to MAX, up is +inf.
        assert_eq!(pown_down(f64::MAX, 2), f64::MAX);
        assert_eq!(pown_up(f64::MAX, 2), f64::INFINITY);
    }

    #[test]
    fn negative_power_underflows_to_the_subnormal_floor_not_a_collapse() {
        // MAX^-2 is far below the smallest subnormal: [+0, 2^-1074], the sign of
        // the zero endpoint pinned. This is the bead enc-5jj overflow-recip case.
        assert_eq!(bits(pown_down(f64::MAX, -2)), bits(0.0));
        assert_eq!(pown_up(f64::MAX, -2), f64::from_bits(1)); // 2^-1074
                                                              // (-MAX)^-3 is a tiny negative: [-2^-1074, -0.0].
        assert_eq!(pown_down(-f64::MAX, -3), -f64::from_bits(1));
        assert_eq!(bits(pown_up(-f64::MAX, -3)), bits(-0.0));
    }

    #[test]
    fn power_of_two_base_is_exact() {
        // 2^-1074 <= 2^k <= exact powers: down == up.
        assert_eq!(pown_down(8.0, 5), 32768.0);
        assert_eq!(pown_up(8.0, 5), 32768.0);
        assert_eq!(pown_down(0.5, -3), 8.0);
        assert_eq!(pown_up(0.5, -3), 8.0);
        assert_eq!(pown_down(2.0, 10), 1024.0);
        assert_eq!(pown_up(2.0, -10), 1.0 / 1024.0);
    }

    #[test]
    fn fallback_beyond_the_exact_range_is_sound_and_ordered() {
        // |n| > POWN_TIGHT_MAX: a near-one base stays finite; bounds bracket it.
        let n = 65_i32; // POWN_TIGHT_MAX + 1
        let d = pown_down(1.0009765625, n); // (1 + 2^-10)^65
        let u = pown_up(1.0009765625, n);
        assert!(d <= u);
        assert!(d > 1.0 && u.is_finite());
    }
}
