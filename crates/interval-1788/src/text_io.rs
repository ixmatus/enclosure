//! Text I/O for `Interval<f64>`: the IEEE 1788 Level 1 literal grammar, parsed
//! and formatted, behind the `fixture` feature.
//!
//! This surface reads ATTACKER-SUPPLIED bytes. The threat model is stated first
//! because it is the reason the surface exists in the shape it does.
//!
//! # Threat model
//!
//! Who supplies the input: any caller of [`text_to_interval`](Interval::text_to_interval)
//! or [`exact_to_interval`](Interval::exact_to_interval), including a network
//! peer, a file, or a configuration string. The bytes are an untrusted `&str`.
//!
//! The worst a hostile input can do is force one of two outcomes, both bounded:
//! a [`TextError`] value, or a sound interval that encloses the literal's
//! mathematical value. It can NOT cause a panic, unbounded work, an allocation
//! (the surface is allocation free), or an interval that fails to enclose the
//! value it was asked to parse. Every quantitative bound the parser needs to
//! stay bounded is a named constant: [`MAX_INPUT_LEN`], [`MAX_SIG_DIGITS`],
//! [`MAX_EXP_DIGITS`]. An input exceeding any of them is a [`TextError::TooLong`],
//! which is the parser's answer to "an attacker sends a gigabyte of digits". The
//! big-integer scratch is a fixed stack buffer (`Big`); a computation that
//! would exceed it saturates to an overflowed endpoint rather than growing or
//! panicking, so the no-panic and no-alloc guarantees hold for every input.
//!
//! The `Big` scratch buffer and the `directed_ratio` routine named below are
//! private helpers of this module; the source carries their full documentation.
//!
//! # Placement: f64 specific, behind `fixture`
//!
//! Parsing a decimal or hex literal yields an EXACT real value (a rational or a
//! dyadic). Turning that exact value into a directed floating-point endpoint
//! (lower rounded toward minus infinity, upper toward plus infinity) is
//! inherently format specific: it needs the bit layout of the target float, its
//! subnormal boundary, and its overflow threshold. The [`RoundFloat`](crate::RoundFloat)
//! contract deliberately has no "round this exact rational into `Self`" method,
//! and adding one would enlarge the contract every backend must satisfy for a
//! Level 2 feature only the reference fixture needs at this stage. So this
//! surface is implemented for `f64` directly and gated on `fixture`, matching
//! every other host-test lane in the crate. A production backend supplies its own
//! tight text I/O; this lane is the reference and the conformance harness.
//!
//! # The directed decimal-to-float law (the rigor-critical core)
//!
//! A literal denotes an exact real `v`. Its lower-endpoint image is `round_down(v)`
//! (the largest `f64` not exceeding `v`) and its upper-endpoint image is
//! `round_up(v)` (the smallest `f64` not below `v`). When `v` is exactly
//! representable, both equal `v`; otherwise they are the two `f64` neighbors that
//! straddle `v`. This is the only rounding that keeps the enclosure sound: the
//! read-back interval must never be narrower than the set the literal names.
//!
//! Every finite literal reduces to a signed ratio `+-num/den` of nonnegative
//! integers:
//!
//! - decimal `+-C x 10^s`: `num = C x 10^max(0,s)`, `den = 10^max(0,-s)`;
//! - hex-float `+-M x 2^k`: `num = M << max(0,k)`, `den = 1 << max(0,-k)`;
//! - rational `+-p/q`: `num = |p|`, `den = |q|`.
//!
//! `directed_ratio` rounds `+-num/den` to the directed `f64` by exact integer
//! long division: it normalizes `num/den` to a 53-bit significand and a binary
//! exponent, takes the quotient's top bits with an exact nonzero-remainder flag,
//! and rounds down (truncate) or up (truncate then add one when the remainder is
//! nonzero), carrying into the exponent and saturating at the subnormal and
//! overflow boundaries. There is NO floating point in the decision: the side of
//! `v` relative to a representable bound is settled by comparing integers. This
//! is why hex never routes through decimal (its exactness is a property of the
//! same integer division with a power-of-two denominator) and why no `str::parse`
//! "nearest then adjust one ulp" step is needed: the division computes the
//! directed result directly. Rust's correctly-rounded `str::parse::<f64>` is
//! still used, but only in the test lane, as an independent nearest-rounding
//! oracle the directed bounds must bracket.
//!
//! # Failures are values, not signals (a documented divergence)
//!
//! IEEE 1788 clause 12.11 has `textToInterval` and `numsToInterval` SIGNAL the
//! `UndefinedOperation` exception on a malformed or invalid input. This crate has
//! no exception mechanism; it reports the same failures as `Result::Err`
//! ([`TextError`] for text, [`IntervalError`] for the
//! numeric constructor) and, on the decorated side, an invalid numeric pair as
//! `Err` rather than a returned `NaI`. A `[nai]` literal, which is a valid
//! decorated VALUE rather than a failure, parses to
//! [`DecoratedInterval::nai`](crate::DecoratedInterval::nai). The exceptions-file
//! vectors are transcribed in the test lane with each `signal` mapped to the
//! corresponding `Err`, and the divergence noted there.
//!
//! # Grammar coverage
//!
//! Implemented, as the constructor and class vector sets exercise them: the
//! bracketed forms (`[empty]`, `[]`, `[entire]`, `[,]`, `[l, u]`, the half-open
//! `[l,]`/`[,u]`, the point `[m]`, `[nai]`), whitespace tolerance, ASCII-case
//! folding of the keyword and infinity tokens, decimal literals with exponents,
//! hex-float literals, rational `p/q` literals, the uncertain form (`m?`, `m?r`,
//! `m?ru`/`m?rd`, `m??`, `m??u`/`m??d`, with a trailing exponent), and the
//! decoration suffixes `_com`/`_dac`/`_def`/`_trv`/`_ill` (and `[nai]`).
//!
//! Named gaps (no vector coverage, deliberately not implemented): the interval
//! output side (`intervalToText` has zero corpus assertions, gap 2 in the
//! `itf1788-framework` registry) is supplied here as a [`Display`](fmt::Display)
//! whose reread encloses, plus the exact hex pair
//! [`interval_to_exact`](Interval::interval_to_exact) /
//! [`exact_to_interval`](Interval::exact_to_interval); their correctness rests on
//! the round-trip properties in the test lane, not on the corpus. Rational
//! literals are integer/integer only (the grammar's `p/q`), matching every
//! vector; decimal numerators or exponents inside a rational are not part of the
//! literal grammar and are rejected.

// The big-integer and float-assembly core does deliberate width and sign casts:
// truncating a `u128` product to its low `u64` limb, narrowing a bounded bit
// length to a signed exponent, using a nonnegative shift amount as a `usize`.
// Each is safe by construction (the value is range-checked at the site), so the
// pedantic cast lints are allowed for this module, matching the crate's stance on
// the numeric kernel's chosen vocabulary.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use core::fmt;

use crate::decoration::Decoration;
use crate::error::IntervalError;
use crate::{DecoratedInterval, Interval};

/// The maximum accepted input length in bytes. Longer input is
/// [`TextError::TooLong`]: the bound that answers unbounded attacker input. It
/// clears the longest vector (an uncertain form with a 310-digit radius) with
/// headroom.
pub const MAX_INPUT_LEN: usize = 2048;

/// The maximum number of significant digits accepted in a single digit group (an
/// integer part, a fraction, a radius, or a rational operand). Longer is
/// [`TextError::TooLong`]. Chosen to clear the 310-digit radius vector with
/// headroom while keeping the big-integer scratch bounded.
pub const MAX_SIG_DIGITS: usize = 512;

/// The maximum number of digits accepted in a decimal or binary exponent field.
/// Longer is [`TextError::TooLong`]. Nine digits reach `+-999_999_999`, far
/// beyond the `f64` range, so any in-range magnitude is expressible and any
/// larger one saturates to a directed overflow or underflow endpoint.
pub const MAX_EXP_DIGITS: usize = 9;

/// A failure to read an interval from text.
///
/// This is the value form of the standard's `UndefinedOperation` signal for the
/// text constructors (see the [module divergence note](self)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextError {
    /// The input did not match the interval literal grammar.
    Syntax,
    /// The input, or one of its digit groups or exponents, exceeded a length cap
    /// ([`MAX_INPUT_LEN`], [`MAX_SIG_DIGITS`], or [`MAX_EXP_DIGITS`]). This is the
    /// threat-model response to unbounded input.
    TooLong,
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TextError::Syntax => "interval text does not match the literal grammar",
            TextError::TooLong => "interval text exceeds a length bound",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TextError {}

// ===========================================================================
// Fixed-capacity unsigned big integer.
//
// Allocation-free scratch for the exact directed-rounding division. Little-endian
// `u64` limbs; `n` is the count of live limbs (the top limb nonzero), so `n == 0`
// is exactly zero. Every widening operation checks capacity and reports overflow
// (`false`) rather than panicking, so an adversarial magnitude degrades to an
// overflowed endpoint, never a crash. `LIMBS` covers the worst in-range case (a
// ~512-digit coefficient divided out through a subnormal, shifted left by the
// 1074-bit subnormal span) with headroom.
// ===========================================================================

const LIMBS: usize = 128;

#[derive(Clone)]
struct Big {
    d: [u64; LIMBS],
    n: usize,
}

impl Big {
    fn zero() -> Self {
        Big {
            d: [0; LIMBS],
            n: 0,
        }
    }

    fn from_u64(x: u64) -> Self {
        let mut b = Big::zero();
        if x != 0 {
            b.d[0] = x;
            b.n = 1;
        }
        b
    }

    fn is_zero(&self) -> bool {
        self.n == 0
    }

    fn normalize(&mut self) {
        while self.n > 0 && self.d[self.n - 1] == 0 {
            self.n -= 1;
        }
    }

    /// The position of the highest set bit plus one (0 for zero).
    fn bit_len(&self) -> usize {
        if self.n == 0 {
            return 0;
        }
        let top = self.d[self.n - 1];
        (self.n - 1) * 64 + (64 - top.leading_zeros() as usize)
    }

    /// Multiply in place by a small factor. Returns `false` on capacity overflow.
    #[must_use]
    fn mul_small(&mut self, m: u64) -> bool {
        if m == 0 || self.n == 0 {
            *self = Big::zero();
            return true;
        }
        let mut carry: u128 = 0;
        for i in 0..self.n {
            let prod = u128::from(self.d[i]) * u128::from(m) + carry;
            self.d[i] = prod as u64;
            carry = prod >> 64;
        }
        while carry != 0 {
            if self.n >= LIMBS {
                return false;
            }
            self.d[self.n] = carry as u64;
            carry >>= 64;
            self.n += 1;
        }
        true
    }

    /// Add a small value in place. Returns `false` on capacity overflow.
    #[must_use]
    fn add_small(&mut self, a: u64) -> bool {
        let mut carry = u128::from(a);
        let mut i = 0;
        while carry != 0 {
            if i >= LIMBS {
                return false;
            }
            if i >= self.n {
                self.n = i + 1;
            }
            let sum = u128::from(self.d[i]) + carry;
            self.d[i] = sum as u64;
            carry = sum >> 64;
            i += 1;
        }
        true
    }

    /// Multiply in place by `10^p`. Returns `false` on capacity overflow.
    #[must_use]
    fn mul_pow10(&mut self, mut p: u32) -> bool {
        // Chunk by 10^19, the largest power of ten below 2^64.
        const CHUNK: u64 = 10_000_000_000_000_000_000;
        while p >= 19 {
            if !self.mul_small(CHUNK) {
                return false;
            }
            p -= 19;
        }
        if p > 0 {
            let mut f: u64 = 1;
            for _ in 0..p {
                f *= 10;
            }
            if !self.mul_small(f) {
                return false;
            }
        }
        true
    }

    /// Shift left by `bits` (multiply by `2^bits`) in place. Returns `false` on
    /// capacity overflow.
    #[must_use]
    fn shl_bits(&mut self, bits: usize) -> bool {
        if self.n == 0 || bits == 0 {
            return true;
        }
        let limb_shift = bits / 64;
        let bit_shift = bits % 64;
        let need = (self.bit_len() + bits).div_ceil(64);
        if need > LIMBS {
            return false;
        }
        if bit_shift == 0 {
            let mut i = self.n;
            while i > 0 {
                i -= 1;
                self.d[i + limb_shift] = self.d[i];
            }
        } else {
            // Top overflow limb first, then each limb high to low, so a source is
            // read before its destination is written.
            let top = self.d[self.n - 1];
            let hi = top >> (64 - bit_shift);
            if hi != 0 {
                self.d[self.n + limb_shift] = hi;
            }
            let mut prev = top;
            let mut i = self.n - 1;
            while i > 0 {
                let cur = self.d[i - 1];
                self.d[i + limb_shift] = (prev << bit_shift) | (cur >> (64 - bit_shift));
                prev = cur;
                i -= 1;
            }
            self.d[limb_shift] = prev << bit_shift;
        }
        for slot in self.d.iter_mut().take(limb_shift) {
            *slot = 0;
        }
        self.n = need;
        self.normalize();
        true
    }

    fn cmp(&self, other: &Big) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        if self.n != other.n {
            return self.n.cmp(&other.n);
        }
        let mut i = self.n;
        while i > 0 {
            i -= 1;
            if self.d[i] != other.d[i] {
                return self.d[i].cmp(&other.d[i]);
            }
        }
        Ordering::Equal
    }

    /// Add `other` into `self` in place. Returns `false` on capacity overflow.
    #[must_use]
    fn add_assign(&mut self, other: &Big) -> bool {
        let m = self.n.max(other.n);
        let mut carry: u128 = 0;
        for i in 0..m {
            let s = if i < self.n { u128::from(self.d[i]) } else { 0 };
            let o = if i < other.n {
                u128::from(other.d[i])
            } else {
                0
            };
            let sum = s + o + carry;
            self.d[i] = sum as u64;
            carry = sum >> 64;
        }
        self.n = m;
        if carry != 0 {
            if self.n >= LIMBS {
                return false;
            }
            self.d[self.n] = carry as u64;
            self.n += 1;
        }
        true
    }

    /// Subtract `other` from `self` in place, requiring `self >= other`.
    fn sub_assign(&mut self, other: &Big) {
        let mut borrow: i128 = 0;
        for i in 0..self.n {
            let o = if i < other.n {
                i128::from(other.d[i])
            } else {
                0
            };
            let mut diff = i128::from(self.d[i]) - o - borrow;
            if diff < 0 {
                diff += 1i128 << 64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            self.d[i] = diff as u64;
        }
        self.normalize();
    }
}

// ===========================================================================
// Directed rational-to-f64 conversion (the rigor-critical core).
// ===========================================================================

const F64_MAX: f64 = f64::MAX;
const MIN_SUBNORMAL: f64 = 5e-324; // 2^-1074, the smallest positive f64.
const MANT_BIT: u64 = 1 << 52; // the implicit leading significand bit.

/// The floor quotient `a / b` (assumed below `2^55`) together with a flag that is
/// true when the division has a nonzero remainder (the value is inexact). A shift
/// that overflows the scratch makes `b << i` larger than any representable `a`, so
/// that quotient bit is simply zero.
fn div_qr(a: &Big, b: &Big) -> (u64, bool) {
    use core::cmp::Ordering::Greater;
    let mut rem = a.clone();
    let mut q: u64 = 0;
    for i in (0..=54usize).rev() {
        let mut bi = b.clone();
        if !bi.shl_bits(i) {
            continue;
        }
        if bi.cmp(&rem) != Greater {
            rem.sub_assign(&bi);
            q |= 1u64 << i;
        }
    }
    (q, !rem.is_zero())
}

/// Assemble a nonnegative `f64` from a rounded significand and binary exponent:
/// `mantissa * 2^e`, where a normal `mantissa` has bit 52 set and a subnormal one
/// is below `2^52` with `e == -1074`. `over_to_inf` selects the overflow endpoint
/// (`+inf` when rounding up, [`F64_MAX`] when rounding down).
fn assemble(mantissa: u64, e: i32, over_to_inf: bool) -> f64 {
    if mantissa == 0 {
        return 0.0;
    }
    if mantissa >= MANT_BIT {
        // Normal: value = 1.f * 2^(52+e), biased exponent (52+e)+1023.
        let biased = e + 52 + 1023;
        if biased >= 2047 {
            return if over_to_inf { f64::INFINITY } else { F64_MAX };
        }
        debug_assert!(biased >= 1, "normal significand with subnormal exponent");
        let bits = ((biased as u64) << 52) | (mantissa & (MANT_BIT - 1));
        f64::from_bits(bits)
    } else {
        // Subnormal: value = mantissa * 2^-1074, biased exponent 0.
        debug_assert!(
            e == -1074,
            "subnormal significand off the subnormal exponent"
        );
        f64::from_bits(mantissa)
    }
}

/// Round the nonnegative exact value `num / den` (with `den != 0`) to an `f64`,
/// toward plus infinity when `use_ceil`, toward minus infinity otherwise.
fn round_magnitude(num: &Big, den: &Big, use_ceil: bool) -> f64 {
    if num.is_zero() {
        return 0.0;
    }
    // Overflow without shifting: bit-length gap of 1025 forces value >= 2^1024.
    let gap = num.bit_len() as i64 - den.bit_len() as i64;
    if gap >= 1025 {
        return if use_ceil { f64::INFINITY } else { F64_MAX };
    }

    // Preferred exponent so the 53-bit significand lands in [2^52, 2^53); clamp
    // at the subnormal floor. The bit-length estimate can be one low, corrected
    // by the normalization loop below.
    let mut e: i32 = (gap - 52).max(-1074) as i32;

    let (mut mantissa, inexact) = loop {
        // value / 2^e = A / B with A = num << max(0,-e), B = den << max(0,e).
        let mut a = num.clone();
        let mut b = den.clone();
        let ok = if e >= 0 {
            b.shl_bits(e as usize)
        } else {
            a.shl_bits((-e) as usize)
        };
        if !ok {
            // Unreachable given the caller's magnitude pre-check and the gap
            // guard; degrade to a sound (loose) bound rather than panic.
            return if use_ceil { f64::INFINITY } else { 0.0 };
        }
        let (q, rem_nonzero) = div_qr(&a, &b);
        if q >= (1u64 << 53) && e < 971 {
            e += 1; // overshoot: the significand needs one more binary place.
            continue;
        }
        if q < MANT_BIT && e > -1074 {
            e -= 1; // undershoot (still in the normal range): borrow a place.
            continue;
        }
        break (q, rem_nonzero);
    };

    if use_ceil && inexact {
        mantissa += 1;
        if mantissa >= (1u64 << 53) {
            mantissa >>= 1;
            e += 1;
        }
    }

    assemble(mantissa, e, use_ceil)
}

/// Round the signed exact value `+-num/den` to the directed `f64`: toward plus
/// infinity for an upper endpoint (`want_upper`), toward minus infinity for a
/// lower one. Rounding a negative value toward plus infinity is rounding its
/// magnitude toward minus infinity, and vice versa, so the sign selects the
/// magnitude direction.
fn directed_ratio(neg: bool, num: &Big, den: &Big, want_upper: bool) -> f64 {
    let use_ceil = want_upper != neg;
    let mag = round_magnitude(num, den, use_ceil);
    if neg {
        -mag
    } else {
        mag
    }
}

/// An exact real value named by a literal, classified so that its directed `f64`
/// endpoints are computable without materializing an astronomically large power
/// (a huge exponent field lands in `Over`/`Under` rather than building `10^s`).
///
/// The `Ratio` variant carries two fixed-capacity big integers, so it dwarfs the
/// tag-only variants; boxing them would need `alloc`, which this crate forgoes.
/// A `Real` is a short-lived stack local within a single parse, so the size
/// difference costs nothing.
#[allow(clippy::large_enum_variant)]
enum Real {
    /// A signed infinity token.
    Inf { neg: bool },
    /// A finite value `+-num/den`, in the range where the exact division runs.
    Ratio { neg: bool, num: Big, den: Big },
    /// A finite value whose magnitude exceeds [`F64_MAX`].
    Over { neg: bool },
    /// A finite nonzero value below the underflow-to-zero threshold.
    Under { neg: bool },
    /// Exactly zero.
    Zero,
    /// The panic-free sound fallback for an unreachable capacity overflow:
    /// `[-inf, +inf]` encloses any value.
    Wide,
}

impl Real {
    /// The lower endpoint `round_down(value)`.
    fn round_down(&self) -> f64 {
        match self {
            Real::Inf { neg } => sign_inf(*neg),
            Real::Ratio { neg, num, den } => directed_ratio(*neg, num, den, false),
            Real::Over { neg } => {
                if *neg {
                    f64::NEG_INFINITY
                } else {
                    F64_MAX
                }
            }
            Real::Under { neg } => {
                if *neg {
                    -MIN_SUBNORMAL
                } else {
                    0.0
                }
            }
            Real::Zero => 0.0,
            Real::Wide => f64::NEG_INFINITY,
        }
    }

    /// The upper endpoint `round_up(value)`.
    fn round_up(&self) -> f64 {
        match self {
            Real::Inf { neg } => sign_inf(*neg),
            Real::Ratio { neg, num, den } => directed_ratio(*neg, num, den, true),
            Real::Over { neg } => {
                if *neg {
                    -F64_MAX
                } else {
                    f64::INFINITY
                }
            }
            Real::Under { neg } => {
                if *neg {
                    -0.0
                } else {
                    MIN_SUBNORMAL
                }
            }
            Real::Zero => 0.0,
            Real::Wide => f64::INFINITY,
        }
    }
}

fn sign_inf(neg: bool) -> f64 {
    if neg {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    }
}

fn one() -> Big {
    Big::from_u64(1)
}

/// log10(2) as the rational `30103 / 100000`, used to bound a value's decimal
/// exponent from its bit length without floating point.
const LOG10_2_NUM: i64 = 30103;
const LOG10_2_DEN: i64 = 100_000;

/// The exact real `+-coeff x 10^s`, classified for directed conversion. A huge
/// `|s|` (from a nine-digit exponent field) is caught by the bit-length estimate
/// of the decimal exponent before any power of ten is built.
fn decimal_real(neg: bool, coeff: Big, s: i64) -> Real {
    if coeff.is_zero() {
        return Real::Zero;
    }
    let bl = coeff.bit_len() as i64;
    let dexp_lo = s + ((bl - 1) * LOG10_2_NUM) / LOG10_2_DEN - 1;
    let dexp_hi = s + (bl * LOG10_2_NUM) / LOG10_2_DEN + 1;
    if dexp_lo > 340 {
        return Real::Over { neg };
    }
    if dexp_hi < -340 {
        return Real::Under { neg };
    }
    // In band: |s| is bounded, so the power of ten fits the scratch.
    if s.unsigned_abs() > 5000 {
        return Real::Wide;
    }
    let mut num = coeff;
    let mut den = one();
    let ok = if s >= 0 {
        num.mul_pow10(s as u32)
    } else {
        den.mul_pow10((-s) as u32)
    };
    if ok {
        Real::Ratio { neg, num, den }
    } else {
        Real::Wide
    }
}

/// The exact real `+-mant x 2^k`, classified for directed conversion (hex-float).
fn hex_real(neg: bool, mant: Big, k: i64) -> Real {
    if mant.is_zero() {
        return Real::Zero;
    }
    let mag2 = k + mant.bit_len() as i64 - 1;
    if mag2 >= 1024 {
        return Real::Over { neg };
    }
    if mag2 <= -1076 {
        return Real::Under { neg };
    }
    let mut num = mant;
    let mut den = one();
    let ok = if k >= 0 {
        num.shl_bits(k as usize)
    } else {
        den.shl_bits((-k) as usize)
    };
    if ok {
        Real::Ratio { neg, num, den }
    } else {
        Real::Wide
    }
}

// ===========================================================================
// Byte-level literal parsing.
//
// The grammar is ASCII; a non-ASCII byte never matches a token and falls through
// to a syntax error. Every parser consumes its whole token and rejects any
// trailing byte, so embedded whitespace inside a number (the vectors' "1.0  00")
// is a syntax error rather than a silently-truncated parse.
// ===========================================================================

/// ASCII whitespace the grammar tolerates around and inside brackets.
fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Trim leading and trailing ASCII whitespace.
fn trim(mut b: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = b {
        if is_ws(*first) {
            b = rest;
        } else {
            break;
        }
    }
    while let [rest @ .., last] = b {
        if is_ws(*last) {
            b = rest;
        } else {
            break;
        }
    }
    b
}

/// ASCII-case-insensitive equality against a lowercase keyword.
fn eq_ci(bytes: &[u8], keyword: &[u8]) -> bool {
    bytes.len() == keyword.len()
        && bytes
            .iter()
            .zip(keyword)
            .all(|(&b, &k)| b.to_ascii_lowercase() == k)
}

/// Append a decimal digit to a big integer. Returns `false` on capacity overflow
/// (unreachable within the digit cap; guarded to keep the no-panic promise).
#[must_use]
fn push_dec(acc: &mut Big, digit: u8) -> bool {
    acc.mul_small(10) && acc.add_small(u64::from(digit - b'0'))
}

fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// Read `1..=MAX_SIG_DIGITS` decimal digits, consuming the whole slice.
fn read_all_digits(s: &[u8]) -> Result<Big, TextError> {
    if s.is_empty() {
        return Err(TextError::Syntax);
    }
    let mut acc = Big::zero();
    let mut count = 0;
    for &b in s {
        if !b.is_ascii_digit() {
            return Err(TextError::Syntax);
        }
        if !push_dec(&mut acc, b) {
            return Err(TextError::TooLong);
        }
        count += 1;
        if count > MAX_SIG_DIGITS {
            return Err(TextError::TooLong);
        }
    }
    Ok(acc)
}

/// Parse a signed decimal mantissa with no exponent (the uncertain form's `m`),
/// yielding the sign, the digit string as an integer, and the fraction length.
fn parse_mantissa(s: &[u8]) -> Result<(bool, Big, i64), TextError> {
    let (neg, rest) = split_sign(s);
    let mut i = 0;
    let mut coeff = Big::zero();
    let mut ndig = 0usize;
    let mut fraclen: i64 = 0;
    let mut any = false;
    while i < rest.len() && rest[i].is_ascii_digit() {
        if !push_dec(&mut coeff, rest[i]) {
            return Err(TextError::TooLong);
        }
        ndig += 1;
        any = true;
        i += 1;
        if ndig > MAX_SIG_DIGITS {
            return Err(TextError::TooLong);
        }
    }
    if i < rest.len() && rest[i] == b'.' {
        i += 1;
        while i < rest.len() && rest[i].is_ascii_digit() {
            if !push_dec(&mut coeff, rest[i]) {
                return Err(TextError::TooLong);
            }
            ndig += 1;
            fraclen += 1;
            any = true;
            i += 1;
            if ndig > MAX_SIG_DIGITS {
                return Err(TextError::TooLong);
            }
        }
    }
    if !any || i != rest.len() {
        return Err(TextError::Syntax);
    }
    Ok((neg, coeff, fraclen))
}

fn split_sign(s: &[u8]) -> (bool, &[u8]) {
    match s.first() {
        Some(b'+') => (false, &s[1..]),
        Some(b'-') => (true, &s[1..]),
        _ => (false, s),
    }
}

/// Parse one number token (already trimmed, nonempty): infinity, hex-float,
/// rational `p/q`, or decimal.
fn parse_number(tok: &[u8]) -> Result<Real, TextError> {
    let (neg, rest) = split_sign(tok);
    if rest.is_empty() {
        return Err(TextError::Syntax);
    }
    if eq_ci(rest, b"inf") || eq_ci(rest, b"infinity") {
        return Ok(Real::Inf { neg });
    }
    if rest.len() >= 2 && rest[0] == b'0' && (rest[1] | 0x20) == b'x' {
        return parse_hex(neg, &rest[2..]);
    }
    if rest.contains(&b'/') {
        return parse_rational(neg, rest);
    }
    parse_decimal(neg, rest)
}

/// Parse a decimal literal body (after the sign): `intdigits [. frac] [e exp]`.
fn parse_decimal(neg: bool, s: &[u8]) -> Result<Real, TextError> {
    let mut i = 0;
    let mut coeff = Big::zero();
    let mut ndig = 0usize;
    let mut fraclen: i64 = 0;
    let mut any = false;
    while i < s.len() && s[i].is_ascii_digit() {
        if !push_dec(&mut coeff, s[i]) {
            return Err(TextError::TooLong);
        }
        ndig += 1;
        any = true;
        i += 1;
        if ndig > MAX_SIG_DIGITS {
            return Err(TextError::TooLong);
        }
    }
    if i < s.len() && s[i] == b'.' {
        i += 1;
        while i < s.len() && s[i].is_ascii_digit() {
            if !push_dec(&mut coeff, s[i]) {
                return Err(TextError::TooLong);
            }
            ndig += 1;
            fraclen += 1;
            any = true;
            i += 1;
            if ndig > MAX_SIG_DIGITS {
                return Err(TextError::TooLong);
            }
        }
    }
    if !any {
        return Err(TextError::Syntax);
    }
    let mut exp: i64 = 0;
    if i < s.len() && (s[i] | 0x20) == b'e' {
        i += 1;
        let eneg = match s.get(i) {
            Some(b'+') => {
                i += 1;
                false
            }
            Some(b'-') => {
                i += 1;
                true
            }
            _ => false,
        };
        let mut ec = 0;
        while i < s.len() && s[i].is_ascii_digit() {
            exp = exp * 10 + i64::from(s[i] - b'0');
            ec += 1;
            i += 1;
            if ec > MAX_EXP_DIGITS {
                return Err(TextError::TooLong);
            }
        }
        if ec == 0 {
            return Err(TextError::Syntax);
        }
        if eneg {
            exp = -exp;
        }
    }
    if i != s.len() {
        return Err(TextError::Syntax);
    }
    Ok(decimal_real(neg, coeff, exp - fraclen))
}

/// Parse a hex-float literal body (after the `0x`): `hexdigits [. hexfrac] p exp`.
/// The binary exponent is required, matching C99 hex floats and every vector.
fn parse_hex(neg: bool, s: &[u8]) -> Result<Real, TextError> {
    let mut i = 0;
    let mut mant = Big::zero();
    let mut ndig = 0usize;
    let mut fraclen: i64 = 0;
    let mut any = false;
    while i < s.len() && s[i].is_ascii_hexdigit() {
        if !(mant.mul_small(16) && mant.add_small(u64::from(hex_val(s[i])))) {
            return Err(TextError::TooLong);
        }
        ndig += 1;
        any = true;
        i += 1;
        if ndig > MAX_SIG_DIGITS {
            return Err(TextError::TooLong);
        }
    }
    if i < s.len() && s[i] == b'.' {
        i += 1;
        while i < s.len() && s[i].is_ascii_hexdigit() {
            if !(mant.mul_small(16) && mant.add_small(u64::from(hex_val(s[i])))) {
                return Err(TextError::TooLong);
            }
            ndig += 1;
            fraclen += 1;
            any = true;
            i += 1;
            if ndig > MAX_SIG_DIGITS {
                return Err(TextError::TooLong);
            }
        }
    }
    if !any || i >= s.len() || (s[i] | 0x20) != b'p' {
        return Err(TextError::Syntax);
    }
    i += 1;
    let eneg = match s.get(i) {
        Some(b'+') => {
            i += 1;
            false
        }
        Some(b'-') => {
            i += 1;
            true
        }
        _ => false,
    };
    let mut binexp: i64 = 0;
    let mut ec = 0;
    while i < s.len() && s[i].is_ascii_digit() {
        binexp = binexp * 10 + i64::from(s[i] - b'0');
        ec += 1;
        i += 1;
        if ec > MAX_EXP_DIGITS {
            return Err(TextError::TooLong);
        }
    }
    if ec == 0 || i != s.len() {
        return Err(TextError::Syntax);
    }
    if eneg {
        binexp = -binexp;
    }
    Ok(hex_real(neg, mant, binexp - 4 * fraclen))
}

/// Parse a rational literal `p/q` (after the numerator sign): integer over
/// integer, matching the grammar's `p/q`. A zero denominator is a syntax error.
fn parse_rational(neg: bool, s: &[u8]) -> Result<Real, TextError> {
    let slash = s.iter().position(|&c| c == b'/').ok_or(TextError::Syntax)?;
    if s[slash + 1..].contains(&b'/') {
        return Err(TextError::Syntax);
    }
    let p = read_all_digits(&s[..slash])?;
    let (den_neg, den_bytes) = split_sign(&s[slash + 1..]);
    let q = read_all_digits(den_bytes)?;
    if q.is_zero() {
        return Err(TextError::Syntax);
    }
    if p.is_zero() {
        return Ok(Real::Zero);
    }
    Ok(Real::Ratio {
        neg: neg ^ den_neg,
        num: p,
        den: q,
    })
}

/// The three uncertainty directions of the `m?...` form.
#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Both,
    Up,
    Down,
}

/// `a - b` for `a >= b`.
fn big_sub(a: &Big, b: &Big) -> Big {
    let mut r = a.clone();
    r.sub_assign(b);
    r
}

/// `a + b` (saturating to the operands' scratch; overflow is unreachable here).
fn big_add(a: &Big, b: &Big) -> Big {
    let mut r = a.clone();
    let _ = r.add_assign(b);
    r
}

/// Parse the uncertain form `m ? [radius|?] [u|d] [exp]`, returning the directed
/// lower and upper endpoints and whether each mathematical endpoint is finite
/// (an infinite radius makes the widened side unbounded).
fn parse_uncertain(tok: &[u8]) -> Result<(f64, f64, bool, bool), TextError> {
    let q = tok
        .iter()
        .position(|&c| c == b'?')
        .ok_or(TextError::Syntax)?;
    let (mneg, mabs, fraclen) = parse_mantissa(&tok[..q])?;
    let a = &tok[q + 1..];
    let (infinite, a) = if a.first() == Some(&b'?') {
        (true, &a[1..])
    } else {
        (false, a)
    };

    // Radius (only in the finite form), then direction, then exponent.
    let mut idx = 0;
    let (rbig, mult, pscale) = if infinite {
        (Big::zero(), 1u64, fraclen)
    } else {
        let mut r = Big::zero();
        let mut cnt = 0;
        let mut any = false;
        while idx < a.len() && a[idx].is_ascii_digit() {
            if !push_dec(&mut r, a[idx]) {
                return Err(TextError::TooLong);
            }
            cnt += 1;
            any = true;
            idx += 1;
            if cnt > MAX_SIG_DIGITS {
                return Err(TextError::TooLong);
            }
        }
        if any {
            (r, 1u64, fraclen)
        } else {
            (Big::from_u64(5), 10u64, fraclen + 1)
        }
    };

    let a = &a[idx..];
    let (dir, a) = match a.first().map(|b| b | 0x20) {
        Some(b'u') => (Dir::Up, &a[1..]),
        Some(b'd') => (Dir::Down, &a[1..]),
        _ => (Dir::Both, a),
    };

    let mut e: i64 = 0;
    let mut i = 0;
    if i < a.len() && (a[i] | 0x20) == b'e' {
        i += 1;
        let eneg = match a.get(i) {
            Some(b'+') => {
                i += 1;
                false
            }
            Some(b'-') => {
                i += 1;
                true
            }
            _ => false,
        };
        let mut ec = 0;
        while i < a.len() && a[i].is_ascii_digit() {
            e = e * 10 + i64::from(a[i] - b'0');
            ec += 1;
            i += 1;
            if ec > MAX_EXP_DIGITS {
                return Err(TextError::TooLong);
            }
        }
        if ec == 0 {
            return Err(TextError::Syntax);
        }
        if eneg {
            e = -e;
        }
    }
    if i != a.len() {
        return Err(TextError::Syntax);
    }

    let m_real = decimal_real(mneg, mabs.clone(), e - fraclen);
    if infinite {
        return Ok(match dir {
            Dir::Both => (f64::NEG_INFINITY, f64::INFINITY, false, false),
            Dir::Up => (m_real.round_down(), f64::INFINITY, true, false),
            Dir::Down => (f64::NEG_INFINITY, m_real.round_up(), false, true),
        });
    }
    let (lo, hi) = finite_uncertain(&mabs, mneg, &rbig, mult, e - pscale, dir, &m_real)?;
    Ok((lo, hi, true, true))
}

/// The two directed endpoints of a finite uncertain form. The widened endpoints
/// `m -+ radius*ulp` reduce to a single signed decimal coefficient `M*mult -+ R`
/// with scale `s_ep`, so they round through the same decimal path as any literal.
fn finite_uncertain(
    mabs: &Big,
    mneg: bool,
    rbig: &Big,
    mult: u64,
    s_ep: i64,
    dir: Dir,
    m_real: &Real,
) -> Result<(f64, f64), TextError> {
    use core::cmp::Ordering::Less;
    let mut mscaled = mabs.clone();
    if mult == 10 && !mscaled.mul_small(10) {
        return Err(TextError::TooLong);
    }
    let (lneg, lmag) = if mneg {
        (true, big_add(&mscaled, rbig)) // -(X) - R
    } else if mscaled.cmp(rbig) != Less {
        (false, big_sub(&mscaled, rbig)) // X - R >= 0
    } else {
        (true, big_sub(rbig, &mscaled)) // X - R < 0
    };
    let (hneg, hmag) = if !mneg {
        (false, big_add(&mscaled, rbig)) // X + R
    } else if rbig.cmp(&mscaled) != Less {
        (false, big_sub(rbig, &mscaled)) // R - X >= 0
    } else {
        (true, big_sub(&mscaled, rbig)) // R - X < 0
    };
    let lreal = decimal_real(lneg, lmag, s_ep);
    let hreal = decimal_real(hneg, hmag, s_ep);
    Ok(match dir {
        Dir::Both => (lreal.round_down(), hreal.round_up()),
        Dir::Up => (m_real.round_down(), hreal.round_up()),
        Dir::Down => (lreal.round_down(), m_real.round_up()),
    })
}

/// A parsed interval literal body (no decoration): the bare interval, whether its
/// mathematical value is bounded (finite endpoints, so overflow to a stored
/// infinity still counts as bounded), and whether it was the `[nai]` spelling.
struct Parsed {
    iv: Interval<f64>,
    math_bounded: bool,
    is_nai: bool,
}

fn real_math_finite(r: &Real) -> bool {
    !matches!(r, Real::Inf { .. })
}

/// Parse an interval literal body (everything before any decoration suffix).
fn parse_body(body: &[u8]) -> Result<Parsed, TextError> {
    let body = trim(body);
    if body.is_empty() {
        return Err(TextError::Syntax);
    }
    if body[0] == b'[' {
        if *body.last().unwrap() != b']' {
            return Err(TextError::Syntax);
        }
        let inner = trim(&body[1..body.len() - 1]);
        let first_comma = inner.iter().position(|&c| c == b',');
        if first_comma.is_none() {
            if inner.is_empty() || eq_ci(inner, b"empty") {
                return Ok(Parsed {
                    iv: Interval::empty(),
                    math_bounded: true,
                    is_nai: false,
                });
            }
            if eq_ci(inner, b"entire") {
                return Ok(Parsed {
                    iv: Interval::entire(),
                    math_bounded: false,
                    is_nai: false,
                });
            }
            if eq_ci(inner, b"nai") {
                return Ok(Parsed {
                    iv: Interval::empty(),
                    math_bounded: true,
                    is_nai: true,
                });
            }
            let r = parse_number(inner)?;
            let iv = Interval::new(r.round_down(), r.round_up()).map_err(|_| TextError::Syntax)?;
            Ok(Parsed {
                iv,
                math_bounded: real_math_finite(&r) && !iv.is_empty(),
                is_nai: false,
            })
        } else {
            let c = first_comma.unwrap();
            // A single top-level comma; a second one is malformed.
            if inner[c + 1..].contains(&b',') {
                return Err(TextError::Syntax);
            }
            let lo_s = trim(&inner[..c]);
            let hi_s = trim(&inner[c + 1..]);
            let (lo, lo_fin) = if lo_s.is_empty() {
                (f64::NEG_INFINITY, false)
            } else {
                let r = parse_number(lo_s)?;
                (r.round_down(), real_math_finite(&r))
            };
            let (hi, hi_fin) = if hi_s.is_empty() {
                (f64::INFINITY, false)
            } else {
                let r = parse_number(hi_s)?;
                (r.round_up(), real_math_finite(&r))
            };
            let iv = Interval::new(lo, hi).map_err(|_| TextError::Syntax)?;
            Ok(Parsed {
                iv,
                math_bounded: lo_fin && hi_fin && !iv.is_empty(),
                is_nai: false,
            })
        }
    } else {
        // A non-bracketed literal must be the uncertain form.
        if !body.contains(&b'?') {
            return Err(TextError::Syntax);
        }
        let (lo, hi, lo_fin, hi_fin) = parse_uncertain(body)?;
        let iv = Interval::new(lo, hi).map_err(|_| TextError::Syntax)?;
        Ok(Parsed {
            iv,
            math_bounded: lo_fin && hi_fin && !iv.is_empty(),
            is_nai: false,
        })
    }
}

/// The optional decoration suffix, distinguishing a valid tag from a malformed
/// one so the caller can reject `_fooo` and `_da`.
enum Suffix {
    None,
    Tag(Decoration),
    Invalid,
}

fn parse_suffix(word: &[u8]) -> Suffix {
    if eq_ci(word, b"com") {
        Suffix::Tag(Decoration::Com)
    } else if eq_ci(word, b"dac") {
        Suffix::Tag(Decoration::Dac)
    } else if eq_ci(word, b"def") {
        Suffix::Tag(Decoration::Def)
    } else if eq_ci(word, b"trv") {
        Suffix::Tag(Decoration::Trv)
    } else if eq_ci(word, b"ill") {
        Suffix::Tag(Decoration::Ill)
    } else {
        Suffix::Invalid
    }
}

/// Split a trimmed literal into its body and decoration suffix. The only `_` in a
/// well-formed literal is the suffix separator, so a second one is malformed.
fn split_decoration(t: &[u8]) -> (&[u8], Suffix) {
    match t.iter().position(|&c| c == b'_') {
        None => (t, Suffix::None),
        Some(p) => {
            let word = &t[p + 1..];
            if word.contains(&b'_') {
                (&t[..p], Suffix::Invalid)
            } else {
                (&t[..p], parse_suffix(word))
            }
        }
    }
}

// ===========================================================================
// Public surface: the four constructors, the exact pair, and output.
// ===========================================================================

impl Interval<f64> {
    /// Read a bare interval from its IEEE 1788 text literal (`textToInterval`).
    ///
    /// Accepts the bracketed forms, the uncertain form, and the number literals
    /// the [module grammar](self) documents. A decoration suffix or a `[nai]`
    /// literal is rejected here (both belong to the decorated constructor); the
    /// endpoints round outward, so the result always encloses the literal's
    /// mathematical value.
    ///
    /// # Errors
    ///
    /// [`TextError::Syntax`] if the input does not match the grammar (the value
    /// form of the standard's `UndefinedOperation` signal), or
    /// [`TextError::TooLong`] if it exceeds a length bound.
    pub fn text_to_interval(s: &str) -> Result<Interval<f64>, TextError> {
        let bytes = s.as_bytes();
        if bytes.len() > MAX_INPUT_LEN {
            return Err(TextError::TooLong);
        }
        let (body, suffix) = split_decoration(trim(bytes));
        if !matches!(suffix, Suffix::None) {
            // The bare constructor admits no decoration.
            return Err(TextError::Syntax);
        }
        let parsed = parse_body(body)?;
        if parsed.is_nai {
            // `[nai]` is a decorated value, not a bare interval.
            return Err(TextError::Syntax);
        }
        Ok(parsed.iv)
    }

    /// Build an interval from two `f64` bounds (`numsToInterval`): the standard's
    /// spelling of the checked endpoint constructor.
    ///
    /// # Errors
    ///
    /// The same [`IntervalError`] as [`Interval::new`]: a NaN endpoint, an
    /// unbounding infinity, or an inverted pair (the value form of the standard's
    /// signal on those inputs).
    pub fn nums_to_interval(lo: f64, hi: f64) -> Result<Interval<f64>, IntervalError> {
        Interval::new(lo, hi)
    }

    /// Read an interval from the exact hex-float text produced by
    /// [`interval_to_exact`](Interval::interval_to_exact). This is the inverse of
    /// that output: for any interval `x`, `exact_to_interval(interval_to_exact(x))`
    /// reproduces `x` bit for bit, because each hex endpoint names a representable
    /// `f64` exactly and rounds to itself.
    ///
    /// # Errors
    ///
    /// As [`text_to_interval`](Interval::text_to_interval); the exact form is a
    /// subset of that grammar.
    pub fn exact_to_interval(s: &str) -> Result<Interval<f64>, TextError> {
        Interval::text_to_interval(s)
    }

    /// The exact, lossless text of this interval: each finite endpoint as its full
    /// hex-float expansion (from the bits, so no value is lost), infinities as
    /// `-inf`/`inf`, and the empty set as `[empty]` (clause 13.4's `intervalToExact`).
    ///
    /// The returned value is a [`Display`](fmt::Display) wrapper, so it formats
    /// through `core::fmt` without allocating. Reading it back with
    /// [`exact_to_interval`](Interval::exact_to_interval) is bit-exact.
    #[must_use]
    pub fn interval_to_exact(self) -> ExactInterval {
        ExactInterval(self)
    }
}

impl DecoratedInterval<f64> {
    /// Read a decorated interval from its IEEE 1788 text literal
    /// (`textToInterval` for decorated intervals).
    ///
    /// Accepts everything [`Interval::text_to_interval`] does, plus an optional
    /// decoration suffix (`_com`/`_dac`/`_def`/`_trv`) and the `[nai]` literal. A
    /// bare literal takes the strongest decoration consistent with it; an explicit
    /// suffix must be valid for the literal's mathematical interval and is then
    /// combined with the stored interval's strongest decoration by the lattice
    /// meet (so a literal whose value overflows to an unbounded stored interval
    /// demotes a requested `com` to `dac`). The `_ill` suffix is reserved for the
    /// `[nai]` spelling and is rejected as an explicit tag.
    ///
    /// # Errors
    ///
    /// [`TextError::Syntax`] for a malformed literal, an unknown or inconsistent
    /// decoration, or `[nai]` carrying a suffix; [`TextError::TooLong`] past a
    /// length bound. A well-formed `[nai]` is the [`nai`](DecoratedInterval::nai)
    /// value, returned as `Ok`.
    pub fn text_to_interval(s: &str) -> Result<DecoratedInterval<f64>, TextError> {
        let bytes = s.as_bytes();
        if bytes.len() > MAX_INPUT_LEN {
            return Err(TextError::TooLong);
        }
        let (body, suffix) = split_decoration(trim(bytes));
        if matches!(suffix, Suffix::Invalid) {
            return Err(TextError::Syntax);
        }
        let parsed = parse_body(body)?;
        if parsed.is_nai {
            return match suffix {
                Suffix::None => Ok(DecoratedInterval::nai()),
                _ => Err(TextError::Syntax),
            };
        }
        let x = parsed.iv;
        match suffix {
            Suffix::None => Ok(DecoratedInterval::new_dec(x)),
            // `Invalid` was rejected above; `_ill` is reserved for `[nai]`.
            Suffix::Invalid | Suffix::Tag(Decoration::Ill) => Err(TextError::Syntax),
            Suffix::Tag(tag) => {
                let valid = match tag {
                    Decoration::Com => parsed.math_bounded && !x.is_empty(),
                    Decoration::Dac | Decoration::Def => !x.is_empty(),
                    Decoration::Trv => true,
                    Decoration::Ill => false,
                };
                if !valid {
                    return Err(TextError::Syntax);
                }
                let strongest = DecoratedInterval::new_dec(x).decoration();
                Ok(DecoratedInterval::set_dec(x, tag.meet(strongest)))
            }
        }
    }

    /// Build a decorated interval from two `f64` bounds (`numsToInterval`): the
    /// bare pair through [`Interval::new`], decorated with its strongest
    /// consistent decoration.
    ///
    /// # Errors
    ///
    /// The same [`IntervalError`] as [`Interval::new`] (the value form of the
    /// standard's signal, in place of a returned `NaI`).
    pub fn nums_to_interval(lo: f64, hi: f64) -> Result<DecoratedInterval<f64>, IntervalError> {
        Interval::new(lo, hi).map(DecoratedInterval::new_dec)
    }
}

// ---------------------------------------------------------------------------
// Output: Display for the interval and its exact hex form.
// ---------------------------------------------------------------------------

/// Write one endpoint in the short decimal form: infinities as `-inf`/`inf`,
/// finite values through `f64`'s shortest round-tripping `Display`. Rereading the
/// result rounds outward, so the enclosure never narrows.
fn write_decimal_endpoint(f: &mut fmt::Formatter<'_>, x: f64) -> fmt::Result {
    if x == f64::INFINITY {
        f.write_str("inf")
    } else if x == f64::NEG_INFINITY {
        f.write_str("-inf")
    } else {
        write!(f, "{x}")
    }
}

/// Write one endpoint as its exact hex-float expansion, lossless by construction.
fn write_exact_endpoint(f: &mut fmt::Formatter<'_>, x: f64) -> fmt::Result {
    if x == f64::INFINITY {
        return f.write_str("inf");
    }
    if x == f64::NEG_INFINITY {
        return f.write_str("-inf");
    }
    let bits = x.to_bits();
    let neg = bits >> 63 == 1;
    let exp_field = ((bits >> 52) & 0x7ff) as i32;
    let mant_field = bits & ((1u64 << 52) - 1);
    if neg {
        f.write_str("-")?;
    }
    // 13 hex digits of the 52-bit fraction, trailing zeros trimmed.
    let hex = |v: u64| -> [u8; 13] {
        let mut out = [b'0'; 13];
        for (i, slot) in out.iter_mut().enumerate() {
            let nibble = (v >> (48 - 4 * i)) & 0xf;
            *slot = b"0123456789abcdef"[nibble as usize];
        }
        out
    };
    let write_frac = |f: &mut fmt::Formatter<'_>, digits: &[u8]| -> fmt::Result {
        let mut end = digits.len();
        while end > 0 && digits[end - 1] == b'0' {
            end -= 1;
        }
        if end > 0 {
            f.write_str(".")?;
            for &d in &digits[..end] {
                f.write_str(core::str::from_utf8(&[d]).unwrap())?;
            }
        }
        Ok(())
    };
    if exp_field == 0 {
        if mant_field == 0 {
            return f.write_str("0x0p+0");
        }
        // Subnormal: value = 0x0.<frac> x 2^-1022.
        f.write_str("0x0")?;
        write_frac(f, &hex(mant_field))?;
        write!(f, "p{:+}", -1022)
    } else {
        let e = exp_field - 1023;
        f.write_str("0x1")?;
        write_frac(f, &hex(mant_field))?;
        write!(f, "p{e:+}")
    }
}

impl fmt::Display for Interval<f64> {
    /// The interval as `[empty]`, `[entire]`, or `[lo, hi]` with short decimal
    /// endpoints (`intervalToText`). The endpoints are the shortest decimals that
    /// round-trip; because the reader rounds outward, rereading the text yields an
    /// interval that encloses this one (never a narrower one).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("[empty]");
        }
        if self.is_entire() {
            return f.write_str("[entire]");
        }
        // Serialize the raw stored endpoints, not the numeric `inf`/`sup` (which
        // impose the Level 2 signed-zero datum): the text form reproduces the
        // interval so a reread reconstructs the same stored endpoints.
        let (lo, hi) = self.bounds().expect("nonempty past the empty guard");
        f.write_str("[")?;
        write_decimal_endpoint(f, lo)?;
        f.write_str(", ")?;
        write_decimal_endpoint(f, hi)?;
        f.write_str("]")
    }
}

/// The [`Display`](fmt::Display) wrapper returned by
/// [`Interval::interval_to_exact`]: the interval in exact hex-float text.
#[derive(Clone, Copy, Debug)]
pub struct ExactInterval(Interval<f64>);

impl fmt::Display for ExactInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("[empty]");
        }
        // Raw stored endpoints (sign of zero included), so the exact hex form
        // round-trips bit for bit; the numeric `inf`/`sup` would mask the stored
        // sign under the Level 2 signed-zero datum.
        let (lo, hi) = self.0.bounds().expect("nonempty past the empty guard");
        f.write_str("[")?;
        write_exact_endpoint(f, lo)?;
        f.write_str(", ")?;
        write_exact_endpoint(f, hi)?;
        f.write_str("]")
    }
}

impl fmt::Display for DecoratedInterval<f64> {
    /// The decorated interval as `[nai]`, or the bare interval followed by
    /// `_<decoration>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_nai() {
            return f.write_str("[nai]");
        }
        write!(f, "{}_{}", self.interval(), self.decoration())
    }
}
