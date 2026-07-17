//! The integer Kulisch accumulator: `TightF64`'s correctly rounded reductions.
//!
//! [`RoundReduction`](crate::RoundReduction) demands the exact mathematical
//! value rounded once, which no float error-free transform delivers cheaply for
//! a vector: the product of two `binary64` values reaches exponents no `f64` can
//! hold (down to `2^-2148`, far below the subnormal floor), so a `TwoProduct`
//! route would drag the underflow machinery into every element. This backend is
//! exact by integer construction instead. Each input decomposes by `to_bits`
//! into a sign, a 53-bit integer significand, and a trailing exponent; products
//! multiply significands exactly into a `u128`; every exact partial value shifts
//! into a fixed-point integer accumulator indexed by exponent; and one rounding
//! step at the end reads the whole exact sum. Float arithmetic appears nowhere
//! between decomposition and that final rounding, so the exactness argument is
//! an integer-arithmetic argument and the single rounding is auditable in
//! isolation. See workspace decision record 0008.
//!
//! # Law 1: the fixed-point grid and its dimensioning
//!
//! A finite `binary64` value is `(-1)^s * M * 2^E` with `M` a 53-bit integer
//! (in `[2^52, 2^53)` for a normal, in `[1, 2^52)` at the fixed `E = -1074` for
//! a subnormal). The two exact exponent extremes are these:
//!
//! - the smallest positive subnormal is `2^-1074`, so the lowest bit any exact
//!   term can occupy is `2^-1074` for a bare value and `2^-1074 * 2^-1074 =
//!   2^-2148` for a product;
//! - the largest finite value is `MAX = 2^1024 - 2^971 < 2^1024`, so `MAX^2 <
//!   2^2048`, and the highest bit any exact product can occupy is `2^2047`.
//!
//! The accumulator is a fixed-point integer `A` whose bit `i` carries the real
//! weight `2^(i - EXP_BIAS)` with `EXP_BIAS = 2148`, so bit `0` is `2^-2148`
//! (the product floor) and every exact term lands at a nonnegative bit index. A
//! single exact product then occupies bits `0 ..= 4195`: its low bit is at index
//! `0` at least, and its high bit at index `2047 + 2148 = 4195` at most. That is
//! a span of `4196` bits.
//!
//! Above the product span sit the guard bits, which absorb the carries of
//! repeated addition and hold the two's complement sign. Summing `K` terms each
//! below `2^4196` (in integer units) yields a magnitude below `K * 2^4196`, so
//! `g` guard bits let the sum grow to `2^(4196 + g)` before the sign bit is
//! threatened, admitting up to about `2^(g-1)` terms. The classic Kulisch choice
//! `g = 92` gives headroom for `~2^90` terms, vastly beyond the `~2^63` elements
//! an in-memory slice can hold. The total width is therefore `4196 + 92 = 4288`
//! bits, which is exactly `67` limbs of `64` bits:
//!
//! ```text
//!     NLIMBS = 67,   total bits = 4288 = 4196 (product span) + 92 (guard)
//! ```
//!
//! The array serves every operation: a bare value for `sum`/`sum_abs` uses only
//! bits `1074 ..= 3171`, well inside the product span, so one size derived from
//! the product extremes covers all four reductions.
//!
//! # Law 2: signed accumulation in two's complement
//!
//! The limbs hold `A` as a little-endian two's complement integer. A term of
//! magnitude `V` (a `u128`, from a 53-bit significand or a 106-bit product) at
//! exponent `E` is added at bit shift `E + EXP_BIAS`: the shift splits into a
//! limb offset and an in-limb bit offset, the `u128` becomes at most three
//! 64-bit words straddling three limbs, and those words ripple into the
//! accumulator by carry (for a positive term) or borrow (for a negative term).
//! A borrow propagating past a term's top sets the higher limbs to all ones,
//! which is the correct two's complement encoding of a value that has gone
//! negative. No allocation, no `libm`, and no float arithmetic take part.
//!
//! # Law 3: reading the sum, rounded once
//!
//! At the end the sign is the top bit of the top limb. If it is set the whole
//! array is negated (two's complement) to recover the magnitude, and the sign is
//! remembered. The leading set bit `L` gives the result exponent `e = L -
//! EXP_BIAS`. The 53-bit significand is the block of bits ending at `L`, whose
//! quantum sits at bit `qbit = max(L - 52, 1074)` (the `1074` floor pins the
//! subnormal grid `2^-1074`). The bits below `qbit` decide the rounding: their
//! comparison to half a quantum (the bit at `qbit - 1`, then the sticky OR of
//! everything beneath it) drives round to nearest with ties to even, while a
//! directed magnitude direction rounds up on any nonzero tail (away from zero) or
//! never (toward zero).
//!
//! Sign then composes with the requested mode to pick that magnitude direction:
//! toward nearest is symmetric, toward zero always truncates the magnitude, round
//! toward minus infinity truncates a positive magnitude but rounds a negative one
//! away from zero, and round toward plus infinity does the mirror. Overflow past
//! `MAX` yields the mode's correct boundary, `MAX` when the magnitude is
//! truncated and an infinity when it is pushed away from zero, matching the
//! directed-rounding edge convention of decision record 0002. An exact zero takes
//! the documented sign convention, `-0` under round toward minus infinity and
//! `+0` otherwise; a nonzero value that underflows to zero keeps the sign of its
//! exact value.

use crate::{RoundReduction, TightF64};

/// The accumulator width in 64-bit limbs. Derived in Law 1: `4288` bits is the
/// `4196`-bit exact product span plus `92` guard bits, and `4288 = 67 * 64`.
const NLIMBS: usize = 67;

/// Bit `i` of the accumulator carries weight `2^(i - EXP_BIAS)`. `EXP_BIAS =
/// 2148` places bit `0` at `2^-2148`, the exact product floor (Law 1), so every
/// term shifts in at a nonnegative index.
const EXP_BIAS: i32 = 2148;

/// A requested rounding direction, static at the call site (the house no-enum
/// style renders each as a distinct trait method).
#[derive(Clone, Copy)]
enum Round {
    Down,
    Up,
    Nearest,
    Zero,
}

/// The direction in which a magnitude is rounded, after mode and sign compose.
#[derive(Clone, Copy)]
enum MagDir {
    TowardZero,
    Away,
    Nearest,
}

/// The extended-real classification of a reduction before the finite path runs.
#[derive(Clone, Copy)]
enum Special {
    Nan,
    PosInf,
    NegInf,
    Finite,
}

/// The accumulator bit shift for a term whose trailing exponent is `term_exp`.
/// The exponent floor is a product of two smallest subnormals at `-2148`, so
/// `term_exp + EXP_BIAS` is nonnegative for every finite term and the cast keeps
/// the value (Law 1).
#[inline]
#[allow(clippy::cast_sign_loss)]
fn shift_of(term_exp: i32) -> u32 {
    debug_assert!(term_exp + EXP_BIAS >= 0);
    (term_exp + EXP_BIAS) as u32
}

/// Decompose a finite `f64` into `(negative, significand, trailing_exponent)`
/// with the significand a 53-bit integer, matching `tight_f64::trailing_exp`.
/// A zero has significand `0`. The caller guarantees `x` is finite.
#[inline]
fn decompose(x: f64) -> (bool, u128, i32) {
    let bits = x.to_bits();
    let neg = (bits >> 63) & 1 == 1;
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let frac = u128::from(bits & ((1u64 << 52) - 1));
    if biased == 0 {
        // Subnormal (or zero when frac == 0): value = frac * 2^-1074.
        (neg, frac, -1074)
    } else {
        // Normal: value = (2^52 + frac) * 2^(biased - 1075).
        (neg, frac | (1u128 << 52), biased - 1075)
    }
}

// ---------------------------------------------------------------------------
// The two's complement limb array (Law 2). The primitives take a slice so the
// Kani proofs can exercise the identical carry and shift code on a small array.
// ---------------------------------------------------------------------------

/// Split a magnitude `mag` shifted left by `shift` bits into the three 64-bit
/// words it can straddle and the limb index of the lowest word. `mag < 2^106`,
/// so after a shift of at most 63 within the limb the value spans three words.
#[inline]
#[allow(clippy::cast_possible_truncation)] // the u128 -> u64 casts slice into limbs
fn split(mag: u128, shift: u32) -> ([u64; 3], usize) {
    let limb = (shift / 64) as usize;
    let bits = shift % 64;
    let low = mag as u64;
    let high = (mag >> 64) as u64;
    let words = if bits == 0 {
        [low, high, 0]
    } else {
        [
            low << bits,
            (high << bits) | (low >> (64 - bits)),
            high >> (64 - bits),
        ]
    };
    (words, limb)
}

/// `limbs += mag * 2^shift`, propagating carry upward and stopping once it is
/// absorbed. Law 1's dimensioning guarantees no true overflow. The array length
/// is a const generic so the model checker sees a concrete loop bound.
#[inline]
#[allow(clippy::cast_possible_truncation)] // `sum as u64` keeps the low limb; carry is `sum >> 64`
fn add_mag<const N: usize>(limbs: &mut [u64; N], mag: u128, shift: u32) {
    let (words, base) = split(mag, shift);
    let mut carry: u128 = 0;
    let mut word_idx = 0usize;
    let mut pos = base;
    while pos < N {
        let word = if word_idx < 3 {
            u128::from(words[word_idx])
        } else {
            0
        };
        let sum = u128::from(limbs[pos]) + word + carry;
        limbs[pos] = sum as u64;
        carry = sum >> 64;
        word_idx += 1;
        pos += 1;
        if word_idx >= 3 && carry == 0 {
            break;
        }
    }
}

/// `limbs -= mag * 2^shift`, propagating borrow upward. A borrow past the term's
/// top ripples through the higher limbs, encoding a negative value in two's
/// complement.
#[inline]
#[allow(clippy::cast_possible_truncation)] // each difference is reduced mod 2^64 into the limb
fn sub_mag<const N: usize>(limbs: &mut [u64; N], mag: u128, shift: u32) {
    let (words, base) = split(mag, shift);
    let mut borrow: u128 = 0;
    let mut word_idx = 0usize;
    let mut pos = base;
    while pos < N {
        let word = if word_idx < 3 {
            u128::from(words[word_idx])
        } else {
            0
        };
        let cur = u128::from(limbs[pos]);
        let sub = word + borrow;
        if cur >= sub {
            limbs[pos] = (cur - sub) as u64;
            borrow = 0;
        } else {
            limbs[pos] = (cur + (1u128 << 64) - sub) as u64;
            borrow = 1;
        }
        word_idx += 1;
        pos += 1;
        if word_idx >= 3 && borrow == 0 {
            break;
        }
    }
}

/// Two's complement negation in place: invert every limb, then add one.
#[inline]
#[allow(clippy::cast_possible_truncation)] // `sum as u64` keeps the low limb; carry is `sum >> 64`
fn negate<const N: usize>(limbs: &mut [u64; N]) {
    let mut carry: u128 = 1;
    for limb in limbs.iter_mut() {
        let sum = u128::from(!*limb) + carry;
        *limb = sum as u64;
        carry = sum >> 64;
    }
}

/// The index of the highest set bit, or `None` when every bit is zero.
#[inline]
fn leading_bit<const N: usize>(limbs: &[u64; N]) -> Option<usize> {
    for i in (0..N).rev() {
        if limbs[i] != 0 {
            let hb = 63 - limbs[i].leading_zeros() as usize;
            return Some(i * 64 + hb);
        }
    }
    None
}

/// Whether bit `i` is set.
#[inline]
fn bit(limbs: &[u64], i: usize) -> bool {
    (limbs[i / 64] >> (i % 64)) & 1 == 1
}

/// The integer value of bits `lo ..= hi`, with `hi - lo < 128`.
#[inline]
fn extract_bits(limbs: &[u64], lo: usize, hi: usize) -> u128 {
    let mut out: u128 = 0;
    let mut pos = hi;
    loop {
        out = (out << 1) | u128::from(bit(limbs, pos));
        if pos == lo {
            break;
        }
        pos -= 1;
    }
    out
}

/// Whether any bit strictly below index `pos` is set (the sticky flag).
#[inline]
fn any_bits_below(limbs: &[u64], pos: usize) -> bool {
    let limb = pos / 64;
    let off = pos % 64;
    for &l in &limbs[..limb] {
        if l != 0 {
            return true;
        }
    }
    off != 0 && (limbs[limb] & ((1u64 << off) - 1)) != 0
}

// ---------------------------------------------------------------------------
// The accumulator and its single rounding step (Law 3).
// ---------------------------------------------------------------------------

/// The fixed-point Kulisch accumulator: a two's complement integer scaled by
/// `2^-EXP_BIAS` (Law 1).
struct Accum {
    limbs: [u64; NLIMBS],
}

impl Accum {
    #[inline]
    fn zero() -> Self {
        Accum { limbs: [0; NLIMBS] }
    }

    /// Add a signed term of magnitude `mag` at exponent-derived `shift`.
    #[inline]
    fn add_signed(&mut self, mag: u128, shift: u32, neg: bool) {
        if neg {
            sub_mag(&mut self.limbs, mag, shift);
        } else {
            add_mag(&mut self.limbs, mag, shift);
        }
    }

    /// Read the accumulated exact value, rounded once in `mode` (Law 3). Every
    /// bit index here is below `NLIMBS * 64 = 4288`, so the `usize`/`i32` casts
    /// neither truncate nor lose a sign.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    fn round(&mut self, mode: Round) -> f64 {
        let neg = self.limbs[NLIMBS - 1] >> 63 == 1;
        if neg {
            negate(&mut self.limbs);
        }
        let Some(l) = leading_bit(&self.limbs) else {
            return signed_zero(mode); // exact zero (empty or full cancellation)
        };
        let e = l as i32 - EXP_BIAS;
        let dir = mag_dir(mode, neg);

        // Magnitude entirely above the finite range: the mode's boundary.
        if e >= 1024 {
            let m = match dir {
                MagDir::TowardZero => f64::MAX,
                _ => f64::INFINITY,
            };
            return apply_sign(m, neg);
        }

        // The quantum bit (the ULP), floored at the subnormal grid 2^-1074.
        let qbit = core::cmp::max(l as i32 - 52, 1074) as usize;
        let q = if l >= qbit {
            extract_bits(&self.limbs, qbit, l)
        } else {
            0 // magnitude below the smallest subnormal's quantum
        };
        let round_up = match dir {
            MagDir::TowardZero => false,
            MagDir::Away => any_bits_below(&self.limbs, qbit),
            MagDir::Nearest => {
                if !bit(&self.limbs, qbit - 1) {
                    false // below half a quantum
                } else if any_bits_below(&self.limbs, qbit - 1) {
                    true // above half a quantum
                } else {
                    (q & 1) == 1 // exactly half: ties to even
                }
            }
        };
        let sig = q + u128::from(round_up);
        let qe = qbit as i32 - EXP_BIAS;

        // A carry out of the top binade at e == 1023 reaches 2^1024 (overflow).
        if sig == (1u128 << 53) && e == 1023 {
            return apply_sign(f64::INFINITY, neg);
        }
        apply_sign(scale_significand(sig, qe), neg)
    }
}

/// The sign of an exact-zero result: `-0` toward minus infinity, `+0` otherwise
/// (workspace decision record 0008, pinned to the reference implementation).
#[inline]
fn signed_zero(mode: Round) -> f64 {
    match mode {
        Round::Down => -0.0,
        _ => 0.0,
    }
}

/// Compose the requested mode with the accumulated sign into a magnitude
/// direction. Nearest is symmetric; toward zero always truncates; the directed
/// modes truncate on the side facing their infinity and round away on the other.
#[inline]
fn mag_dir(mode: Round, neg: bool) -> MagDir {
    match mode {
        Round::Nearest => MagDir::Nearest,
        Round::Zero => MagDir::TowardZero,
        Round::Down => {
            if neg {
                MagDir::Away
            } else {
                MagDir::TowardZero
            }
        }
        Round::Up => {
            if neg {
                MagDir::TowardZero
            } else {
                MagDir::Away
            }
        }
    }
}

/// Attach the accumulated sign to a nonnegative magnitude.
#[inline]
fn apply_sign(mag: f64, neg: bool) -> f64 {
    if neg {
        -mag
    } else {
        mag
    }
}

/// Build `sig * 2^qe` as an `f64`. `sig <= 2^53` is exact as an `f64`, and the
/// scaling lands on the target grid without rounding (subnormal, normal, or a
/// binade carry), so the value is exact. Overflow is handled before this call.
#[inline]
#[allow(clippy::cast_precision_loss)] // sig <= 2^53, representable exactly
fn scale_significand(sig: u128, qe: i32) -> f64 {
    libm::scalbn(sig as f64, qe)
}

// ---------------------------------------------------------------------------
// Special-value pre-passes (the extended-real semantics the corpus pins).
// ---------------------------------------------------------------------------

/// Classify a `sum` over its inputs: any NaN poisons, both infinity signs give
/// NaN, a single sign gives that infinity.
fn classify_sum(values: &[TightF64]) -> Special {
    let mut pos_inf = false;
    let mut neg_inf = false;
    for &v in values {
        let x = v.0;
        if x.is_nan() {
            return Special::Nan;
        }
        if x.is_infinite() {
            if x > 0.0 {
                pos_inf = true;
            } else {
                neg_inf = true;
            }
        }
    }
    combine_inf(pos_inf, neg_inf)
}

/// Classify a `sum_abs` or `sum_square` over its inputs: any NaN poisons, any
/// infinity gives positive infinity (the terms are never negative).
fn classify_nonneg(values: &[TightF64]) -> Special {
    for &v in values {
        let x = v.0;
        if x.is_nan() {
            return Special::Nan;
        }
        if x.is_infinite() {
            return Special::PosInf;
        }
    }
    Special::Finite
}

/// Classify a `dot` over its pairs. A NaN in either factor poisons; a zero
/// against an infinity is the indeterminate `0 * inf` and gives NaN; otherwise
/// an infinite factor contributes a signed infinity.
fn classify_dot(a: &[TightF64], b: &[TightF64]) -> Special {
    let mut pos_inf = false;
    let mut neg_inf = false;
    for i in 0..a.len() {
        let x = a[i].0;
        let y = b[i].0;
        if x.is_nan() || y.is_nan() {
            return Special::Nan;
        }
        if x.is_infinite() || y.is_infinite() {
            if x == 0.0 || y == 0.0 {
                return Special::Nan; // 0 * inf
            }
            if x.is_sign_negative() ^ y.is_sign_negative() {
                neg_inf = true;
            } else {
                pos_inf = true;
            }
        }
    }
    combine_inf(pos_inf, neg_inf)
}

/// Reduce the two infinity flags to the extended-real classification.
#[inline]
fn combine_inf(pos_inf: bool, neg_inf: bool) -> Special {
    match (pos_inf, neg_inf) {
        (true, true) => Special::Nan,
        (true, false) => Special::PosInf,
        (false, true) => Special::NegInf,
        (false, false) => Special::Finite,
    }
}

/// Dispatch a finished classification to its non-finite result, or run `finite`.
#[inline]
fn dispatch(special: Special, finite: impl FnOnce() -> f64) -> f64 {
    match special {
        Special::Nan => f64::NAN,
        Special::PosInf => f64::INFINITY,
        Special::NegInf => f64::NEG_INFINITY,
        Special::Finite => finite(),
    }
}

// ---------------------------------------------------------------------------
// The four finite accumulations.
// ---------------------------------------------------------------------------

fn sum_reduce(values: &[TightF64], mode: Round) -> f64 {
    dispatch(classify_sum(values), || {
        let mut acc = Accum::zero();
        for &v in values {
            let x = v.0;
            if x == 0.0 {
                continue;
            }
            let (neg, m, e) = decompose(x);
            acc.add_signed(m, shift_of(e), neg);
        }
        acc.round(mode)
    })
}

fn sum_abs_reduce(values: &[TightF64], mode: Round) -> f64 {
    dispatch(classify_nonneg(values), || {
        let mut acc = Accum::zero();
        for &v in values {
            let x = v.0;
            if x == 0.0 {
                continue;
            }
            let (_neg, m, e) = decompose(x);
            acc.add_signed(m, shift_of(e), false);
        }
        acc.round(mode)
    })
}

fn sum_square_reduce(values: &[TightF64], mode: Round) -> f64 {
    dispatch(classify_nonneg(values), || {
        let mut acc = Accum::zero();
        for &v in values {
            let x = v.0;
            if x == 0.0 {
                continue;
            }
            let (_neg, m, e) = decompose(x);
            // m < 2^53, so m * m < 2^106 fits a u128; the square is nonnegative.
            acc.add_signed(m * m, shift_of(2 * e), false);
        }
        acc.round(mode)
    })
}

fn dot_reduce(a: &[TightF64], b: &[TightF64], mode: Round) -> f64 {
    assert_eq!(
        a.len(),
        b.len(),
        "dot requires equal-length slices (caller-owned precondition)"
    );
    dispatch(classify_dot(a, b), || {
        let mut acc = Accum::zero();
        for i in 0..a.len() {
            let x = a[i].0;
            let y = b[i].0;
            if x == 0.0 || y == 0.0 {
                continue; // a finite factor of zero makes the product zero
            }
            let (nx, mx, ex) = decompose(x);
            let (ny, my, ey) = decompose(y);
            acc.add_signed(mx * my, shift_of(ex + ey), nx ^ ny);
        }
        acc.round(mode)
    })
}

impl RoundReduction for TightF64 {
    #[inline]
    fn sum_down(values: &[Self]) -> Self {
        TightF64(sum_reduce(values, Round::Down))
    }
    #[inline]
    fn sum_up(values: &[Self]) -> Self {
        TightF64(sum_reduce(values, Round::Up))
    }
    #[inline]
    fn sum_to_nearest(values: &[Self]) -> Self {
        TightF64(sum_reduce(values, Round::Nearest))
    }
    #[inline]
    fn sum_to_zero(values: &[Self]) -> Self {
        TightF64(sum_reduce(values, Round::Zero))
    }

    #[inline]
    fn sum_abs_down(values: &[Self]) -> Self {
        TightF64(sum_abs_reduce(values, Round::Down))
    }
    #[inline]
    fn sum_abs_up(values: &[Self]) -> Self {
        TightF64(sum_abs_reduce(values, Round::Up))
    }
    #[inline]
    fn sum_abs_to_nearest(values: &[Self]) -> Self {
        TightF64(sum_abs_reduce(values, Round::Nearest))
    }
    #[inline]
    fn sum_abs_to_zero(values: &[Self]) -> Self {
        TightF64(sum_abs_reduce(values, Round::Zero))
    }

    #[inline]
    fn sum_square_down(values: &[Self]) -> Self {
        TightF64(sum_square_reduce(values, Round::Down))
    }
    #[inline]
    fn sum_square_up(values: &[Self]) -> Self {
        TightF64(sum_square_reduce(values, Round::Up))
    }
    #[inline]
    fn sum_square_to_nearest(values: &[Self]) -> Self {
        TightF64(sum_square_reduce(values, Round::Nearest))
    }
    #[inline]
    fn sum_square_to_zero(values: &[Self]) -> Self {
        TightF64(sum_square_reduce(values, Round::Zero))
    }

    #[inline]
    fn dot_down(a: &[Self], b: &[Self]) -> Self {
        TightF64(dot_reduce(a, b, Round::Down))
    }
    #[inline]
    fn dot_up(a: &[Self], b: &[Self]) -> Self {
        TightF64(dot_reduce(a, b, Round::Up))
    }
    #[inline]
    fn dot_to_nearest(a: &[Self], b: &[Self]) -> Self {
        TightF64(dot_reduce(a, b, Round::Nearest))
    }
    #[inline]
    fn dot_to_zero(a: &[Self], b: &[Self]) -> Self {
        TightF64(dot_reduce(a, b, Round::Zero))
    }
}

// ---------------------------------------------------------------------------
// Kani proofs over the carry, borrow, and shift-index logic (Law 2). CBMC's
// home ground is integer bitvector arithmetic, so the accumulator's limb code
// is a natural target where the float transmute chains of decision record 0002
// were not. The proofs run the identical primitives on a small array (the carry
// logic is length-independent), and the float-to-bits decomposition stays
// outside the harness boundary: the symbolic inputs are already integers.
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod proofs {
    use super::{add_mag, leading_bit, negate, sub_mag};

    /// Five limbs (320 bits) exercise every shift-and-ripple branch. The limb
    /// loops run at most `N` times, so an unwind bound of `N + 1` discharges each
    /// harness; a slice would leave the bound symbolic, which is why the
    /// primitives take a const-generic array.
    const N: usize = 5;

    /// Element-wise limb equality. A direct `==` on arrays lowers to the `memcmp`
    /// builtin, whose own loop would need a separate unwind bound; comparing limb
    /// by limb keeps every loop inside the accumulator primitives.
    fn limbs_eq(a: &[u64; N], b: &[u64; N]) -> bool {
        a[0] == b[0] && a[1] == b[1] && a[2] == b[2] && a[3] == b[3] && a[4] == b[4]
    }

    /// Adding a term then subtracting the identical term returns to zero: the
    /// carry ripple of `add_mag` and the borrow ripple of `sub_mag` are inverse.
    #[kani::proof]
    #[kani::unwind(6)]
    fn add_then_sub_is_identity() {
        let mut a = [0u64; N];
        let mag: u128 = kani::any();
        let shift: u32 = kani::any();
        // A 106-bit magnitude (the product width) and a shift that keeps the
        // three straddling words inside the array exercise the split, the carry,
        // and the borrow together.
        kani::assume(mag < (1u128 << 106));
        kani::assume(shift <= 128);
        add_mag(&mut a, mag, shift);
        sub_mag(&mut a, mag, shift);
        kani::assert(
            limbs_eq(&a, &[0u64; N]),
            "add then sub of the same term is identity",
        );
    }

    /// Two additions commute: the carry propagation does not depend on term order.
    #[kani::proof]
    #[kani::unwind(6)]
    fn additions_commute() {
        let m1: u128 = kani::any();
        let m2: u128 = kani::any();
        let s1: u32 = kani::any();
        let s2: u32 = kani::any();
        kani::assume(m1 < (1u128 << 64));
        kani::assume(m2 < (1u128 << 64));
        kani::assume(s1 <= 128);
        kani::assume(s2 <= 128);
        let mut a = [0u64; N];
        add_mag(&mut a, m1, s1);
        add_mag(&mut a, m2, s2);
        let mut b = [0u64; N];
        add_mag(&mut b, m2, s2);
        add_mag(&mut b, m1, s1);
        kani::assert(limbs_eq(&a, &b), "additions commute regardless of order");
    }

    /// A single set bit added at `shift` lands at bit index `k + shift`: the
    /// shift-index map places bits where Law 1 says.
    #[kani::proof]
    #[kani::unwind(6)]
    fn single_bit_lands_at_shift() {
        let k: u32 = kani::any();
        let s: u32 = kani::any();
        kani::assume(k < 64);
        kani::assume(s <= 200);
        kani::assume((k + s) < (N as u32) * 64);
        let mut a = [0u64; N];
        add_mag(&mut a, 1u128 << k, s);
        kani::assert(
            leading_bit(&a) == Some((k + s) as usize),
            "a single bit lands at index k + shift",
        );
    }

    /// Negating twice restores the original limbs (two's complement involution).
    #[kani::proof]
    #[kani::unwind(6)]
    fn negate_is_involutive() {
        let mut a = [0u64; N];
        a[0] = kani::any();
        a[1] = kani::any();
        a[2] = kani::any();
        let original = a;
        negate(&mut a);
        negate(&mut a);
        kani::assert(
            limbs_eq(&a, &original),
            "double negation restores the value",
        );
    }
}
