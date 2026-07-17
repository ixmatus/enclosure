//! The tight `binary64` instance of [`RoundFloat`]: correctly rounded directed
//! arithmetic built from round-to-nearest hardware alone.
//!
//! [`TightF64`] wraps an `f64` and implements [`RoundFloat`] so that every
//! directed method returns the tightest sound bound: the nearest representable
//! float on the correct side of the exact real result. Where the `f64` fixture
//! ([`crate::f64_impl`]) always steps one float outward and so widens even an
//! exact result, `TightF64` steps outward only when the exact result is truly
//! unrepresentable, and then by exactly one unit in the last place. It exists so
//! the interval and affine crates gain a fast native tight backend on stable
//! `no_std` Rust, and so the ITF1788 conformance vectors that demand tightest
//! results have a backend that meets them. See round-float decision record 0002.
//!
//! # The shape of every operation
//!
//! Stable Rust cannot set the hardware rounding mode, and no Rust can do so
//! soundly (LLVM assumes the default floating-point environment). So each
//! operation computes the round-to-nearest result `r`, recovers the exact sign
//! of the rounding error `t - r` (where `t` is the exact real result) with an
//! error-free transform, and adjusts one float in the losing direction only.
//! The laws below state this once and instantiate it per operation.
//!
//! # Law 1: the bracketing adjust
//!
//! Let `t` be a finite exact real value and `r = RN(t)` its round-to-nearest
//! `f64`. Because `r` is the nearest float to `t`, the true value lies between
//! the neighbors of `r`:
//!
//! ```text
//!     next_down(r) <= t <= next_up(r)
//! ```
//!
//! (were `t` below `next_down(r)`, that neighbor would be strictly closer than
//! `r`, contradicting nearest; symmetrically above). Moreover `r = t` exactly
//! when `t` is representable. So given the sign of `t - r`, the correctly
//! rounded directed results are
//!
//! ```text
//!     down = if t < r { next_down(r) } else { r }
//!     up   = if t > r { next_up(r)   } else { r }
//! ```
//!
//! and when `t` is representable both directions return `r`. This is tight, not
//! merely sound: when `t` is unrepresentable it lies strictly between `r` and
//! one neighbor, so the adjusted float is the nearest float on that side.
//!
//! The remaining laws supply, per operation, the exact sign of `t - r`.
//!
//! # Law 2: `add` and `sub` by `TwoSum`
//!
//! For finite `a`, `b` with `s = RN(a + b)` finite, Knuth's branch-free
//! six-operation `TwoSum` computes the exact error `e = (a + b) - s`:
//!
//! ```text
//!     s  = a + b
//!     bv = s - a
//!     e  = (a - (s - bv)) + (b - bv)
//! ```
//!
//! `e` is a representable `f64` for every finite `a`, `b`, subnormals included:
//! the correcting term of a floating-point sum never underflows, because it is a
//! difference of quantities already on the operands' grid, hence itself on that
//! grid (Ogita, Rump, Oishi 2005 for the transform; Boldo, Daumas 2003 for the
//! no-underflow-caveat proof that addition's correcting term is always
//! representable). Since `a + b = s + e` exactly, `sign((a + b) - s) = sign(e)`,
//! which drives Law 1. `sub` is `add` of the exact negation `-b` (negation flips
//! a sign bit and never rounds).
//!
//! # Law 3: `mul` by `TwoProduct`, and its safe zone
//!
//! For finite `a`, `b` with `r = RN(a * b)` finite, the fused multiply-add
//! recovers the error `e = a*b - r` as `e = fma(a, b, -r)` (Ogita, Rump, Oishi
//! 2005). This `e` is exact provided the product does not underflow. Write each
//! operand as `m * 2^g` with integer significand `m` and `g` its trailing
//! exponent (`g = -1074` for a subnormal). Then:
//!
//! - the exact product `a*b = (m_a m_b) * 2^{g_a + g_b}` is an integer multiple
//!   of `2^{g_a + g_b}`, and so is `r` (its trailing exponent is at least
//!   `g_a + g_b`), hence `e = a*b - r` is a multiple of `2^{g_a + g_b}`;
//! - `|e| <= ulp(r)/2`, and the multiplier `e / 2^{g_a + g_b}` has magnitude at
//!   most `2^{52}`, so `e` fits in the 53-bit significand.
//!
//! Therefore `e` is representable exactly when its grid `2^{g_a + g_b}` is no
//! finer than the subnormal quantum `2^{-1074}`, that is when
//!
//! ```text
//!     g_a + g_b >= -1074            (= e_min - p + 1 for binary64)
//! ```
//!
//! which is the Boldo, Daumas (2003) condition specialized to multiplication.
//! Under it `fma(a, b, -r) = RN(a*b - r) = a*b - r`, so `sign(e)` drives Law 1.
//! A product that violates the condition necessarily underflows (its rounded
//! magnitude is below `~2^{-970}`), and Law 6 handles it by rescaling.
//!
//! # Law 4: `div` by the FMA residual
//!
//! For finite `a`, `b` with `b != 0` and `q = RN(a / b)` finite, the residual
//! `rho = fma(q, b, -a)` equals `q*b - a` exactly under the same style of
//! condition (`g_q + g_b >= -1074`; a residual whose exponent falls below that
//! grid underflows and takes Law 6). Then
//!
//! ```text
//!     q - a/b = (q*b - a) / b = rho / b,
//! ```
//!
//! so `sign(q - a/b) = sign(rho) * sign(b)`, and `rho == 0` means `a/b` is
//! representable (exact). Equivalently `a/b < q` exactly when `rho` and `b` share
//! a sign, which selects the outward neighbor for Law 1. When `q` rounds to zero
//! the residual `fma(0, b, -a) = -a` is exact, so the zero case needs no special
//! grid condition.
//!
//! # Law 5: `sqrt` by the FMA residual
//!
//! For `a > 0` with `s = RN(sqrt(a))` (`libm::sqrt` is correctly rounded), the
//! residual `rho = fma(s, s, -a)` equals `s*s - a` exactly only when the root's
//! own grid is coarse enough. Since `a = s*s` to within a half ulp,
//! `|s*s - a| <= s * ulp(s)`, an integer multiple of the grid `2^{2 g_s}` with a
//! multiplier of at most `2^{53}`, so the residual is representable exactly when
//!
//! ```text
//!     2 * g_s >= -1074,     that is     g_s >= -537,
//! ```
//!
//! where `g_s = trailing_exp(s)`. This condition is tested on `s` directly, not
//! on `a`: it fails not only for subnormal `a` but for the entire small-normal
//! range down to `a ~ 2^-969` (for the smallest normal `a = 2^-1022`,
//! `s = 2^-511` has `g_s = -563 < -537`, so the residual grid `2^-1126` is finer
//! than the subnormal quantum and `fma` rounds it). Since `sqrt` is increasing,
//! `sign(sqrt(a) - s) = -sign(s*s - a) = -sign(rho)`, driving Law 1; `rho == 0`
//! means `a` is a perfect square. When the condition fails, Law 6 rescales `a` by
//! an even power of two into a range where the residual is exact.
//!
//! # Law 6: the nested-grid rescaling lemma
//!
//! When a product, quotient, or square-root residual would underflow, its
//! error-free transform silently stops being error-free. The fix is to scale the
//! inputs by an exact power of two into a range where the transform is exact,
//! compute the correctly rounded *directed* result there, then unscale.
//!
//! **Lemma.** Let `x` be a real number, `S` an integer, and let the `f64` grid be
//! the lattice of representable values. Let `y = RD(x * 2^S)` be the largest
//! float not exceeding `x * 2^S`. Then
//!
//! ```text
//!     RD(x) = the largest float f with f * 2^S <= y,
//! ```
//!
//! and symmetrically for `RU`. That is, one downward rounding of `x * 2^S` to the
//! full grid, followed by a downward rounding of `y * 2^-S` to the grid, equals
//! one direct downward rounding of `x`.
//!
//! **Proof.** Multiplication by `2^S` maps the `f64` grid at the scale of `x`
//! onto a subset of the `f64` grid at the scale of `x * 2^S`: the image quantum
//! `2^{S - 1074}` of the subnormal grid is coarser than or equal to the local
//! quantum of the (normal) scaled grid, because scaling out of the subnormal
//! region into the normal region only widens the relative spacing. So the grids
//! nest: every representable `f` at the scale of `x` has `f * 2^S` representable
//! and on the scaled grid. Now let `g = RD(x)` and suppose some float `f > g` had
//! `f * 2^S <= y <= x * 2^S`; then `f <= x`, contradicting `g = RD(x)`. And
//! `g * 2^S <= x * 2^S`, with `g * 2^S` on the scaled grid, so `g * 2^S <= y`
//! (as `y` is the largest scaled-grid point not exceeding `x * 2^S`). Hence `g`
//! is the largest float with `f * 2^S <= y`. QED.
//!
//! The unscale is realized by one round-to-nearest scaling and an exactness
//! check: compute `d = RN(y * 2^-S)` (via `libm::scalbn`, which rounds to nearest
//! on underflow and is otherwise exact), then compare `d * 2^S` against `y`
//! (this scale-up is exact, no rounding). If `d * 2^S > y`, then `d` overshot and
//! the answer is `next_down(d)`; otherwise `d`. Symmetrically for `RU`. The
//! nearest result is never carried across the scaling boundary: only the directed
//! results `RD` and `RU` are unscaled, which is exactly what avoids the double
//! rounding a scaled-then-unscaled nearest value would suffer.
//!
//! Operands are scaled individually into the band `[2^-100, 2^-99)` by
//! `libm::frexp` and `libm::scalbn` (both exact for these normal targets), where
//! products land near `2^-200` and quotients near `2^0`, comfortably inside every
//! safe zone of Laws 3 and 4. `sqrt` scales its argument by an even power of two
//! so `sqrt(a * 2^{2k}) = sqrt(a) * 2^k` stays exact, computes the directed root
//! in the safe zone, and unscales by `2^-k`; the root of a positive `f64` is
//! always at least `2^-537` and so never subnormal, making that unscale exact.
//!
//! # Law 7: edges
//!
//! - **Overflow.** A round-to-nearest result of `+inf` from finite inputs means
//!   the exact value exceeds `MAX`, so `down = MAX` and `up = +inf`; mirrored,
//!   `-inf` gives `down = -inf`, `up = -MAX`. This is IEEE directed rounding at
//!   the overflow boundary, and Law 1's `next_down` / `next_up` produce it
//!   automatically on the overflow shoulder just below `MAX`.
//! - **Non-finite inputs pass through** IEEE semantics untouched: infinities are
//!   exact in the extended reals and NaN propagates. The interval layer feeds
//!   infinities only where the extended-real result is exact.
//! - **Zero signs follow round-to-nearest.** A true round-down of an exact zero
//!   sum returns `-0`; `TightF64` returns nearest's `+0`. The values are
//!   numerically equal and every enclosure property is value-level, so this is a
//!   documented divergence from bit-for-bit IEEE directed rounding, not a
//!   soundness gap.
//!
//! # Portability preconditions
//!
//! - `f64` arithmetic is IEEE 754 round-to-nearest on the target. True on every
//!   Tier 1 target; x87-without-SSE2 (extended-precision double rounding) is
//!   explicitly unsupported.
//! - `libm::fma` is correctly rounded. Trusted from its musl lineage and
//!   certified by the nightly oracle lane rather than assumed here; this is a
//!   named trust boundary, priced in round-float decision record 0002's
//!   verification obligations.

use crate::{RoundFloat, RoundLargestFinite};

/// An `f64` carrying the correctly rounded directed-arithmetic instance of
/// [`RoundFloat`].
///
/// The wrapped value is public so the newtype is transparent at the call site.
/// See the [module docs](self) for the numbered laws every method honors.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct TightF64(pub f64);

impl TightF64 {
    /// Wrap a raw `f64`.
    #[inline]
    #[must_use]
    pub fn new(x: f64) -> Self {
        TightF64(x)
    }

    /// The wrapped `f64`.
    #[inline]
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

impl From<f64> for TightF64 {
    #[inline]
    fn from(x: f64) -> Self {
        TightF64(x)
    }
}

impl From<TightF64> for f64 {
    #[inline]
    fn from(x: TightF64) -> Self {
        x.0
    }
}

/// The trailing exponent `g` of a finite nonzero `x = m * 2^g`, with integer
/// significand `m` in `[2^52, 2^53)` for a normal and `m` in `[1, 2^52)` at the
/// fixed `g = -1074` for a subnormal. Used only to test the Law 3 and Law 4 grid
/// condition `g_a + g_b >= -1074`; the caller guarantees `x` is finite nonzero.
#[inline]
fn trailing_exp(x: f64) -> i32 {
    let biased = ((x.to_bits() >> 52) & 0x7ff) as i32;
    if biased == 0 {
        // Subnormal: value = frac * 2^-1074.
        -1074
    } else {
        // Normal: value = (2^52 + frac) * 2^(biased - 1075).
        biased - 1075
    }
}

/// The directed pair for a real value bracketed by the neighbors of a finite
/// round-to-nearest result `r`, given the sign of `t - r` (Law 1). `err_neg`
/// means `t < r`, `err_pos` means `t > r`; both false means `t == r` (exact).
#[inline]
fn bracket(r: f64, err_neg: bool, err_pos: bool) -> (f64, f64) {
    let down = if err_neg { r.next_down() } else { r };
    let up = if err_pos { r.next_up() } else { r };
    (down, up)
}

/// The directed pair on the overflow boundary (Law 7): `s` is `+inf` or `-inf`
/// produced by a round-to-nearest operation on finite inputs.
#[inline]
fn overflow_pair(s: f64) -> (f64, f64) {
    if s > 0.0 {
        (f64::MAX, f64::INFINITY)
    } else {
        (f64::NEG_INFINITY, -f64::MAX)
    }
}

/// Scale `x` into the band `[2^-100, 2^-99)` by an exact power of two, returning
/// the scaled value and the exponent `k` with `scaled = x * 2^k`. `x` is finite
/// nonzero. The result is normal, so `scalbn` is exact here (Law 6).
#[inline]
fn scale_unit(x: f64) -> (f64, i32) {
    let (_frac, exp) = libm::frexp(x); // x = frac * 2^exp, |frac| in [0.5, 1)
    let k = -99 - exp; // x * 2^k = frac * 2^-99, magnitude in [2^-100, 2^-99)
    (libm::scalbn(x, k), k)
}

/// Unscale a downward-directed scaled result `y = RD(x * 2^s_exp)` to `RD(x)` by
/// the exactness-checked step of Law 6.
#[inline]
fn unscale_down(y: f64, s_exp: i32) -> f64 {
    let d = libm::scalbn(y, -s_exp); // RN(y * 2^-s_exp)
    if libm::scalbn(d, s_exp) > y {
        // d overshot upward: step back to the largest float at or below x.
        d.next_down()
    } else {
        d
    }
}

/// Unscale an upward-directed scaled result `y = RU(x * 2^s_exp)` to `RU(x)`.
#[inline]
fn unscale_up(y: f64, s_exp: i32) -> f64 {
    let u = libm::scalbn(y, -s_exp); // RN(y * 2^-s_exp)
    if libm::scalbn(u, s_exp) < y {
        // u undershot downward: step forward to the smallest float at or above x.
        u.next_up()
    } else {
        u
    }
}

/// `(down, up)` for `a + b` (Law 2), or IEEE passthrough / overflow edges (Law 7).
#[inline]
fn add_parts(a: f64, b: f64) -> (f64, f64) {
    if !a.is_finite() || !b.is_finite() {
        let r = a + b;
        return (r, r);
    }
    let s = a + b;
    if !s.is_finite() {
        return overflow_pair(s);
    }
    // TwoSum: exact error e = (a + b) - s, representable for all finite inputs.
    let bv = s - a;
    let e = (a - (s - bv)) + (b - bv);
    bracket(s, e < 0.0, e > 0.0)
}

/// `(down, up)` for `a * b` (Law 3), with the safe-zone gate and the Law 6
/// rescaling path.
#[inline]
fn mul_parts(a: f64, b: f64) -> (f64, f64) {
    if !a.is_finite() || !b.is_finite() {
        let r = a * b;
        return (r, r);
    }
    if a == 0.0 || b == 0.0 {
        // Exact zero (sign per round-to-nearest, Law 7).
        let r = a * b;
        return (r, r);
    }
    let r = a * b;
    if !r.is_finite() {
        return overflow_pair(r);
    }
    if trailing_exp(a) + trailing_exp(b) >= -1074 {
        // Safe zone: the TwoProduct error is exact.
        let e = libm::fma(a, b, -r);
        bracket(r, e < 0.0, e > 0.0)
    } else {
        mul_rescale(a, b)
    }
}

/// The Law 6 rescaling path for a product that underflows.
#[inline]
fn mul_rescale(a: f64, b: f64) -> (f64, f64) {
    let (a_s, ka) = scale_unit(a);
    let (b_s, kb) = scale_unit(b);
    let s_exp = ka + kb; // a_s * b_s = a * b * 2^s_exp
    let r = a_s * b_s; // safely normal
    let e = libm::fma(a_s, b_s, -r); // exact in the safe band
    let down_s = if e < 0.0 { r.next_down() } else { r };
    let up_s = if e > 0.0 { r.next_up() } else { r };
    (unscale_down(down_s, s_exp), unscale_up(up_s, s_exp))
}

/// `(down, up)` for `a / b` (Law 4), with the safe-zone gate and rescaling path.
#[inline]
fn div_parts(a: f64, b: f64) -> (f64, f64) {
    if !a.is_finite() || !b.is_finite() {
        let r = a / b;
        return (r, r);
    }
    let q = a / b;
    if q.is_nan() {
        // 0 / 0.
        return (q, q);
    }
    if !q.is_finite() {
        // Pole (b == 0) or overflow of a finite quotient: treat as the overflow
        // boundary (Law 7), matching directed rounding of an out-of-range value.
        return overflow_pair(q);
    }
    if q == 0.0 {
        // a / b underflowed to zero, or a == 0. The residual fma(0, b, -a) = -a
        // is exact, so no grid condition is needed.
        let rho = libm::fma(q, b, -a);
        return div_from_residual(q, b, rho);
    }
    if trailing_exp(q) + trailing_exp(b) >= -1074 {
        let rho = libm::fma(q, b, -a);
        div_from_residual(q, b, rho)
    } else {
        div_rescale(a, b)
    }
}

/// Directed pair for division from the exact residual `rho = q*b - a` (Law 4).
#[inline]
fn div_from_residual(q: f64, b: f64, rho: f64) -> (f64, f64) {
    if rho == 0.0 {
        return (q, q); // a / b is representable.
    }
    // a/b < q exactly when rho and b share a sign (sign(q - a/b) = sign(rho b)).
    let a_over_b_lt_q = (rho > 0.0) == (b > 0.0);
    if a_over_b_lt_q {
        (q.next_down(), q)
    } else {
        (q, q.next_up())
    }
}

/// The Law 6 rescaling path for a quotient that underflows.
#[inline]
fn div_rescale(a: f64, b: f64) -> (f64, f64) {
    let (a_s, ka) = scale_unit(a);
    let (b_s, kb) = scale_unit(b);
    let s_exp = ka - kb; // a_s / b_s = (a / b) * 2^s_exp
    let q = a_s / b_s; // safely normal (both operands near 2^-100)
    let rho = libm::fma(q, b_s, -a_s); // exact in the safe band
    let (down_s, up_s) = if rho == 0.0 {
        (q, q)
    } else if (rho > 0.0) == (b_s > 0.0) {
        (q.next_down(), q)
    } else {
        (q, q.next_up())
    };
    (unscale_down(down_s, s_exp), unscale_up(up_s, s_exp))
}

/// `(down, up)` for `sqrt(a)` (Law 5), with the subnormal rescaling path.
#[inline]
fn sqrt_parts(a: f64) -> (f64, f64) {
    if a.is_nan() || a.is_infinite() {
        return (a, a); // NaN and +inf pass through.
    }
    if a < 0.0 {
        let n = libm::sqrt(a); // NaN, out of domain.
        return (n, n);
    }
    if a == 0.0 {
        return (a, a); // sqrt(+/-0) = +/-0, exact.
    }
    let s = libm::sqrt(a);
    // The residual s*s - a lands on the grid 2^{2 g_s}, exact only when that is
    // no finer than the subnormal quantum, i.e. `2 * trailing_exp(s) >= -1074`.
    // This fails not only for subnormal `a` but across the whole small-normal
    // range (down to `a ~ 2^-969`), where the root's trailing exponent is below
    // -537; those take the rescaling path (Law 5, Law 6).
    if 2 * trailing_exp(s) >= -1074 {
        let rho = libm::fma(s, s, -a);
        sqrt_from_residual(s, rho)
    } else {
        sqrt_rescale(a)
    }
}

/// Directed pair for square root from the exact residual `rho = s*s - a` (Law 5).
#[inline]
fn sqrt_from_residual(s: f64, rho: f64) -> (f64, f64) {
    // sign(sqrt(a) - s) = -sign(rho).
    if rho == 0.0 {
        (s, s) // a is a perfect square.
    } else if rho > 0.0 {
        (s.next_down(), s) // s*s > a, so s > sqrt(a).
    } else {
        (s, s.next_up()) // s*s < a, so s < sqrt(a).
    }
}

/// The Law 6 rescaling path for the square root of a subnormal argument, scaling
/// by an even power of two so `sqrt(a * 2^{2k}) = sqrt(a) * 2^k` stays exact.
#[inline]
fn sqrt_rescale(a: f64) -> (f64, f64) {
    let (_frac, exp) = libm::frexp(a);
    // Bring a into the band near 2^-100 by an EVEN power of two so the square
    // root of the scaled argument factors exactly as sqrt(a) * 2^k.
    let raw = -99 - exp;
    let two_k = raw & !1; // even, within one of raw; scaled argument stays normal
    let k = two_k / 2;
    let a_s = libm::scalbn(a, two_k); // normal, exact
    let s = libm::sqrt(a_s);
    let rho = libm::fma(s, s, -a_s);
    let (down_s, up_s) = sqrt_from_residual(s, rho);
    // sqrt(a) = sqrt(a_s) * 2^-k; the root of a positive f64 is at least 2^-537,
    // so this unscale lands in the normal range and is exact.
    (unscale_down(down_s, k), unscale_up(up_s, k))
}

impl RoundFloat for TightF64 {
    const INFINITY: Self = TightF64(f64::INFINITY);
    const NEG_INFINITY: Self = TightF64(f64::NEG_INFINITY);
    const ZERO: Self = TightF64(0.0);
    const ONE: Self = TightF64(1.0);

    #[inline]
    fn add_down(self, rhs: Self) -> Self {
        TightF64(add_parts(self.0, rhs.0).0)
    }
    #[inline]
    fn add_up(self, rhs: Self) -> Self {
        TightF64(add_parts(self.0, rhs.0).1)
    }
    #[inline]
    fn sub_down(self, rhs: Self) -> Self {
        // a - b = a + (-b); negation is exact.
        TightF64(add_parts(self.0, -rhs.0).0)
    }
    #[inline]
    fn sub_up(self, rhs: Self) -> Self {
        TightF64(add_parts(self.0, -rhs.0).1)
    }
    #[inline]
    fn mul_down(self, rhs: Self) -> Self {
        TightF64(mul_parts(self.0, rhs.0).0)
    }
    #[inline]
    fn mul_up(self, rhs: Self) -> Self {
        TightF64(mul_parts(self.0, rhs.0).1)
    }
    #[inline]
    fn div_down(self, rhs: Self) -> Self {
        TightF64(div_parts(self.0, rhs.0).0)
    }
    #[inline]
    fn div_up(self, rhs: Self) -> Self {
        TightF64(div_parts(self.0, rhs.0).1)
    }
    #[inline]
    fn sqrt_down(self) -> Self {
        TightF64(sqrt_parts(self.0).0)
    }
    #[inline]
    fn sqrt_up(self) -> Self {
        TightF64(sqrt_parts(self.0).1)
    }
    #[inline]
    fn negate(self) -> Self {
        TightF64(-self.0)
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.0.is_nan()
    }
    #[inline]
    fn is_finite(self) -> bool {
        self.0.is_finite()
    }
    #[inline]
    fn is_infinite(self) -> bool {
        self.0.is_infinite()
    }
    #[inline]
    fn is_sign_negative(self) -> bool {
        self.0.is_sign_negative()
    }
    #[inline]
    fn is_zero(self) -> bool {
        self.0 == 0.0
    }
}

impl RoundLargestFinite for TightF64 {
    const LARGEST_FINITE: Self = TightF64(f64::MAX);
}
