//! The `f64` instance of [`RoundFloat`]: a verification and host-test fixture.
//!
//! This instance is NOT a production interval float. It exists so the enclosure
//! laws of the crates built on [`RoundFloat`] can be checked two ways the
//! correctly-rounded backends cannot support: by CBMC under Kani (which models
//! `f64` bit-precisely but cannot model a multiword decimal) and by host
//! property tests with no heavy dependency.
//!
//! # Soundness without tightness
//!
//! Rust's `f64` has no directed-rounding arithmetic, so each operation is
//! computed in round-to-nearest and then stepped one float outward with
//! [`f64::next_down`] / [`f64::next_up`]. For a round-to-nearest result `r` of
//! an exact real value `t`, `r` is the nearest float to `t`, so `t` lies between
//! the neighbors of `r`:
//!
//! ```text
//!     next_down(r) <= t <= next_up(r)
//! ```
//!
//! (If `t` were below `next_down(r)`, that neighbor would be strictly closer to
//! `t` than `r`, contradicting `r` being nearest; symmetrically above.) So
//! stepping outward is always a correct bound. It widens by up to one unit in
//! the last place even when `r` was exact, which is the deliberate
//! soundness-not-tightness trade described in the [crate docs](crate). `sqrt`
//! uses `libm::sqrt`, which is correctly rounded, then steps outward the same
//! way.

use crate::{RoundFloat, RoundHyperbolic, RoundInteger, RoundPow, RoundTranscendental, RoundTrig};

impl RoundFloat for f64 {
    const INFINITY: Self = f64::INFINITY;
    const NEG_INFINITY: Self = f64::NEG_INFINITY;
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    #[inline]
    fn add_down(self, rhs: Self) -> Self {
        (self + rhs).next_down()
    }
    #[inline]
    fn add_up(self, rhs: Self) -> Self {
        (self + rhs).next_up()
    }
    #[inline]
    fn sub_down(self, rhs: Self) -> Self {
        (self - rhs).next_down()
    }
    #[inline]
    fn sub_up(self, rhs: Self) -> Self {
        (self - rhs).next_up()
    }
    #[inline]
    fn mul_down(self, rhs: Self) -> Self {
        (self * rhs).next_down()
    }
    #[inline]
    fn mul_up(self, rhs: Self) -> Self {
        (self * rhs).next_up()
    }
    #[inline]
    fn div_down(self, rhs: Self) -> Self {
        (self / rhs).next_down()
    }
    #[inline]
    fn div_up(self, rhs: Self) -> Self {
        (self / rhs).next_up()
    }
    #[inline]
    fn sqrt_down(self) -> Self {
        libm::sqrt(self).next_down()
    }
    #[inline]
    fn sqrt_up(self) -> Self {
        libm::sqrt(self).next_up()
    }
    #[inline]
    fn negate(self) -> Self {
        -self
    }

    #[inline]
    fn is_nan(self) -> bool {
        f64::is_nan(self)
    }
    #[inline]
    fn is_finite(self) -> bool {
        f64::is_finite(self)
    }
    #[inline]
    fn is_infinite(self) -> bool {
        f64::is_infinite(self)
    }
    #[inline]
    fn is_sign_negative(self) -> bool {
        f64::is_sign_negative(self)
    }
    #[inline]
    fn is_zero(self) -> bool {
        self == 0.0
    }
}

/// Relative margin that turns `libm`'s faithfully-rounded `exp`/`log` into a
/// sound outward bound.
///
/// `libm` (the musl port) targets less than 1.5 ulp of error in round-to-nearest
/// for `exp` and `log` but, unlike `sqrt`, does not guarantee correct rounding,
/// so the returned float may be the second-nearest to the true value. A single
/// `next_up`/`next_down` step would then risk landing on the wrong side of it.
/// Widening by a relative factor of `8 * 2^-52` dominates that 1.5 ulp goal with
/// more than fivefold headroom at every magnitude; expressed as a relative
/// factor rather than a fixed step count, it stays sound across a binade
/// boundary, where the spacing just below a power of two is half that just
/// above. A trailing outward step adds an absolute floor so a subnormal result,
/// whose ulp exceeds its own magnitude times `2^-52`, is still covered.
///
/// This is the deliberately loose, sound-not-tight behavior the [crate
/// docs](crate) describe; correct rounding (a tight bound) comes from a
/// production backend, and the nightly `pfloat-libm` oracle lane certifies this
/// bound encloses a correctly-rounded reference. See round-float decision record
/// 0001 and the musl mathematical-library accuracy note.
const TRANSCENDENTAL_MARGIN: f64 = 8.0 * f64::EPSILON;

/// `|x|` by a sign flip. Every call site has already excluded NaN.
#[inline]
fn magnitude(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// Widen a finite faithfully-rounded result outward to an upper bound: add the
/// relative margin (rounded up), then one outward step for the subnormal floor.
#[inline]
fn widen_up(r: f64) -> f64 {
    r.add_up(magnitude(r).mul_up(TRANSCENDENTAL_MARGIN))
        .next_up()
}

/// Companion of [`widen_up`]: a lower bound.
#[inline]
fn widen_down(r: f64) -> f64 {
    r.sub_down(magnitude(r).mul_up(TRANSCENDENTAL_MARGIN))
        .next_down()
}

impl RoundTranscendental for f64 {
    #[inline]
    fn exp_down(self) -> Self {
        let r = libm::exp(self);
        if r.is_nan() {
            r
        } else if r == 0.0 {
            // e^x > 0 everywhere, so zero is a sound (loose) lower bound on an
            // underflowed result.
            0.0
        } else if r.is_infinite() {
            // Overflow: the true value exceeds the finite range, so the largest
            // finite float is a sound lower bound. The affine layer rejects a
            // non-finite enclosure; keeping the bound finite lets it do so.
            f64::MAX
        } else {
            widen_down(r)
        }
    }

    #[inline]
    fn exp_up(self) -> Self {
        let r = libm::exp(self);
        if r.is_nan() || r.is_infinite() {
            // A NaN argument propagates; positive infinity is a sound upper bound
            // on an overflowed result.
            r
        } else if r == 0.0 {
            // Underflow: the true value is below half the smallest subnormal, so
            // the smallest positive subnormal bounds it from above.
            (0.0_f64).next_up()
        } else {
            widen_up(r)
        }
    }

    #[inline]
    fn ln_down(self) -> Self {
        let r = libm::log(self);
        if r.is_finite() {
            widen_down(r)
        } else {
            // self <= 0 is out of domain (libm gives NaN for a negative argument,
            // negative infinity at zero); self == +inf gives +inf. The caller
            // owns the self > 0 precondition, so pass the sentinel through.
            r
        }
    }

    #[inline]
    fn ln_up(self) -> Self {
        let r = libm::log(self);
        if r.is_finite() {
            widen_up(r)
        } else {
            r
        }
    }
}

/// The integer-rounding point functions. Each is exact, so it delegates to the
/// matching `libm` routine and returns the result unwidened.
///
/// The delegation goes through `libm` rather than the `f64::floor`/`f64::ceil`
/// inherent methods because this crate is `no_std`: those methods live in `std`
/// (they lower to platform math intrinsics), while the `libm` port is a pure-Rust
/// `no_std` implementation, the same reason [`RoundFloat::sqrt_down`] uses
/// `libm::sqrt`. `libm::roundeven` is IEEE 754 `roundToIntegralTiesToEven`
/// (matching `f64::round_ties_even`), and `libm::round` breaks ties away from
/// zero (matching `f64::round`); neither reads a hardware rounding mode.
impl RoundInteger for f64 {
    #[inline]
    fn floor(self) -> Self {
        libm::floor(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        libm::ceil(self)
    }
    #[inline]
    fn trunc(self) -> Self {
        libm::trunc(self)
    }
    #[inline]
    fn round_ties_to_even(self) -> Self {
        libm::roundeven(self)
    }
    #[inline]
    fn round_ties_to_away(self) -> Self {
        libm::round(self)
    }
}

/// Lower a finite bound to a closed-range ceiling: `min(x, hi)`. Sound wherever
/// the ceiling is a theorem about the function's range (`sin`, `cos <= 1`;
/// `tanh < 1 <= 1`): an upper bound may be lowered to the ceiling without ever
/// excluding a true value.
#[inline]
fn clamp_high(x: f64, hi: f64) -> f64 {
    if x > hi {
        hi
    } else {
        x
    }
}

/// Raise a finite bound to a closed-range floor: `max(x, lo)`. The companion of
/// [`clamp_high`], sound wherever the floor is a range theorem (`sin`,
/// `cos >= -1`; `cosh >= 1`; `tanh > -1 >= -1`).
#[inline]
fn clamp_low(x: f64, lo: f64) -> f64 {
    if x < lo {
        lo
    } else {
        x
    }
}

impl RoundTrig for f64 {
    #[inline]
    fn sin_down(self) -> Self {
        if !self.is_finite() {
            // sin has no limit at an infinite argument; the interval layer never
            // routes an infinite endpoint here (it maps an unbounded input to the
            // full range directly). Pass a NaN sentinel for a direct caller.
            return f64::NAN;
        }
        // Range clamp AFTER widening: the mathematical range `sin >= -1` is a
        // theorem, so a widened lower bound that dipped below -1 is raised back.
        clamp_low(widen_down(libm::sin(self)), -1.0)
    }

    #[inline]
    fn sin_up(self) -> Self {
        if !self.is_finite() {
            return f64::NAN;
        }
        clamp_high(widen_up(libm::sin(self)), 1.0)
    }

    #[inline]
    fn cos_down(self) -> Self {
        if !self.is_finite() {
            return f64::NAN;
        }
        clamp_low(widen_down(libm::cos(self)), -1.0)
    }

    #[inline]
    fn cos_up(self) -> Self {
        if !self.is_finite() {
            return f64::NAN;
        }
        clamp_high(widen_up(libm::cos(self)), 1.0)
    }

    #[inline]
    fn tan_down(self) -> Self {
        if !self.is_finite() {
            return f64::NAN;
        }
        // No range clamp: `tan` is unbounded. Near a pole libm returns a large
        // finite value; the fixture never manufactures the infinite pole itself
        // (that is the interval layer's set semantics).
        widen_down(libm::tan(self))
    }

    #[inline]
    fn tan_up(self) -> Self {
        if !self.is_finite() {
            return f64::NAN;
        }
        widen_up(libm::tan(self))
    }

    // The pi enclosure. `core::f64::consts::PI` is the binary64 float nearest the
    // mathematical pi. That nearest float is BELOW pi:
    //
    //   pi                     = 3.14159265358979323846264338327...
    //   consts::PI (binary64)  = 3.14159265358979311599796346854...  (0x1.921fb54442d18p+1)
    //   PI.next_up()           = 3.14159265358979356009063455953...  (0x1.921fb54442d19p+1)
    //
    // so `PI < pi < PI.next_up()`: the pair brackets the true pi, and it is the
    // tightest sound bracket (one ulp wide, ~4.4e-16 at this magnitude) because
    // no float lies strictly between the two. Were PI's rounding side unknown,
    // `PI.next_down()` / `PI.next_up()` would bracket regardless at twice the
    // width; the decimal expansions above are what let the tighter one-ulp pair
    // be justified. The ITF1788 vectors corroborate this choice: they use
    // `0x1.921fb54442d18p+1` and `0x1.921fb54442d19p+1` as the two sides of pi.
    #[inline]
    fn pi_down() -> Self {
        core::f64::consts::PI
    }

    #[inline]
    fn pi_up() -> Self {
        core::f64::consts::PI.next_up()
    }

    #[inline]
    fn floor(self) -> Self {
        // Delegates to `libm::floor` (pure Rust, no_std), the same reason
        // `RoundInteger::floor` does; this duplicate exists by workspace decision
        // record 0005's accepted trade to keep `RoundTrig` self-contained.
        libm::floor(self)
    }
}

impl RoundHyperbolic for f64 {
    #[inline]
    fn sinh_down(self) -> Self {
        if self.is_nan() {
            return self;
        }
        let r = libm::sinh(self);
        if r == f64::NEG_INFINITY {
            // Unbounded below (argument at or past the negative overflow shoulder):
            // negative infinity is a sound lower bound.
            f64::NEG_INFINITY
        } else if r == f64::INFINITY {
            // Positive overflow: the largest finite float is a sound finite lower
            // bound, the exp fixture's precedent.
            f64::MAX
        } else {
            widen_down(r)
        }
    }

    #[inline]
    fn sinh_up(self) -> Self {
        if self.is_nan() {
            return self;
        }
        let r = libm::sinh(self);
        if r == f64::INFINITY {
            f64::INFINITY
        } else if r == f64::NEG_INFINITY {
            // Negative overflow: the most negative finite float is a sound finite
            // upper bound.
            -f64::MAX
        } else {
            widen_up(r)
        }
    }

    #[inline]
    fn cosh_down(self) -> Self {
        if self.is_nan() {
            return self;
        }
        let r = libm::cosh(self);
        if r == f64::INFINITY {
            f64::MAX
        } else {
            // Range clamp: `cosh >= 1` is a theorem, so a widened lower bound below
            // 1 is raised back to 1 (exact at the minimum `cosh(0) = 1`).
            clamp_low(widen_down(r), 1.0)
        }
    }

    #[inline]
    fn cosh_up(self) -> Self {
        if self.is_nan() {
            return self;
        }
        let r = libm::cosh(self);
        if r == f64::INFINITY {
            f64::INFINITY
        } else {
            widen_up(r)
        }
    }

    #[inline]
    fn tanh_down(self) -> Self {
        if self.is_nan() {
            return self;
        }
        if self == f64::INFINITY {
            return 1.0;
        }
        if self == f64::NEG_INFINITY {
            return -1.0;
        }
        // Range clamp: the true `tanh` lies strictly inside `(-1, 1)`, so clamping
        // the LOWER bound up to -1 is sound (the true value exceeds -1, hence
        // exceeds any bound at -1). A tighter clamp to the largest float above -1
        // would be UNSOUND: for a large negative argument the true `tanh` is
        // closer to -1 than one ulp, so it would fall below such a clamp. -1 is
        // the correct sound floor.
        clamp_low(widen_down(libm::tanh(self)), -1.0)
    }

    #[inline]
    fn tanh_up(self) -> Self {
        if self.is_nan() {
            return self;
        }
        if self == f64::INFINITY {
            return 1.0;
        }
        if self == f64::NEG_INFINITY {
            return -1.0;
        }
        // The mirror of `tanh_down`: clamp the upper bound down to 1 (sound; the
        // true value is below 1). 1 is the correct sound ceiling, not the largest
        // float below 1.
        clamp_high(widen_up(libm::tanh(self)), 1.0)
    }
}

/// Lower bound on `x^(1/n)` for `x > 0`, `n >= 1`, via [`RoundPow::pow_down`] with
/// a DIRECTED reciprocal exponent.
///
/// `x^(1/n)` increases with the exponent when `x > 1` and decreases when `x < 1`
/// (it is constant at `x == 1`). A lower bound therefore raises `x` to the smaller
/// exponent when `x > 1` and to the larger exponent when `x < 1`, drawing the two
/// exponents from the bracket `[1/n rounded down, 1/n rounded up]`. This directed
/// choice is what absorbs the rounding of `1/n` soundly: the true `1/n` lies
/// between the two directed reciprocals, so `x` raised to the correctly chosen
/// side bounds `x^(1/n)` without any extra relative margin. (A single
/// `pow(x, 1.0/n)` with the fixed transcendental margin would NOT suffice, since
/// the exponent-rounding error grows with `|ln x|` and overruns the margin for a
/// base far from 1.)
#[inline]
fn rootn_pos_down(x: f64, n: u32) -> f64 {
    let nf = f64::from(n);
    if x > 1.0 {
        x.pow_down(1.0_f64.div_down(nf))
    } else if x < 1.0 {
        x.pow_down(1.0_f64.div_up(nf))
    } else {
        1.0
    }
}

/// Upper bound on `x^(1/n)` for `x > 0`, `n >= 1`: the mirror of
/// [`rootn_pos_down`], raising `x` to the larger exponent when `x > 1` and the
/// smaller when `x < 1`.
#[inline]
fn rootn_pos_up(x: f64, n: u32) -> f64 {
    let nf = f64::from(n);
    if x > 1.0 {
        x.pow_up(1.0_f64.div_up(nf))
    } else if x < 1.0 {
        x.pow_up(1.0_f64.div_down(nf))
    } else {
        1.0
    }
}

impl RoundPow for f64 {
    #[inline]
    fn pow_down(self, exp: Self) -> Self {
        let r = libm::pow(self, exp);
        if r.is_nan() {
            // Out of the real domain (a negative base with a non-integer exponent);
            // the caller owns the `self >= 0` precondition.
            r
        } else if r == 0.0 {
            // With `self >= 0` the true value is nonnegative, so zero is a sound
            // lower bound on an underflowed or exactly-zero result.
            0.0
        } else if r.is_infinite() {
            // Overflow: the largest finite float is a sound finite lower bound.
            f64::MAX
        } else {
            widen_down(r)
        }
    }

    #[inline]
    fn pow_up(self, exp: Self) -> Self {
        let r = libm::pow(self, exp);
        if r.is_nan() || r.is_infinite() {
            // NaN propagates; positive infinity is a sound upper bound on overflow.
            r
        } else if r == 0.0 {
            // Underflow: the true value is positive but below half the smallest
            // subnormal, which bounds it from above.
            (0.0_f64).next_up()
        } else {
            widen_up(r)
        }
    }

    #[inline]
    fn rootn_down(self, n: u32) -> Self {
        if n == 0 {
            // The zeroth root is undefined; the caller owns `n >= 1`.
            return f64::NAN;
        }
        if self.is_nan() {
            return self;
        }
        if self == 0.0 {
            // `0^(1/n) = 0` for every `n >= 1` (covers `+0` and `-0`).
            return 0.0;
        }
        if self > 0.0 {
            rootn_pos_down(self, n)
        } else if n % 2 == 0 {
            // Even root of a negative base: outside the real domain.
            f64::NAN
        } else {
            // Odd root of a negative base: `rootn(x) = -rootn(|x|)`, so a lower
            // bound is the negation of an UPPER bound on `rootn(|x|)`.
            -rootn_pos_up(-self, n)
        }
    }

    #[inline]
    fn rootn_up(self, n: u32) -> Self {
        if n == 0 {
            return f64::NAN;
        }
        if self.is_nan() {
            return self;
        }
        if self == 0.0 {
            return 0.0;
        }
        if self > 0.0 {
            rootn_pos_up(self, n)
        } else if n % 2 == 0 {
            f64::NAN
        } else {
            -rootn_pos_down(-self, n)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{RoundHyperbolic, RoundInteger, RoundPow, RoundTranscendental, RoundTrig};
    use core::f64::consts::{E, LN_2};

    /// The bracket straddles the central libm value (a necessary condition for
    /// enclosing the true value) and is correctly ordered.
    fn brackets_central(x: f64) {
        let lo = x.exp_down();
        let hi = x.exp_up();
        let r = libm::exp(x);
        assert!(lo <= r, "exp_down({x}) = {lo} not <= libm {r}");
        assert!(r <= hi, "exp_up({x}) = {hi} not >= libm {r}");
        assert!(lo <= hi);
    }

    #[test]
    fn exp_brackets_known_constants() {
        // e^0 = 1, e^1 = e, e^ln2 = 2.
        assert!(0.0_f64.exp_down() <= 1.0 && 1.0 <= 0.0_f64.exp_up());
        assert!(1.0_f64.exp_down() <= E && E <= 1.0_f64.exp_up());
        assert!(LN_2.exp_down() <= 2.0 && 2.0 <= LN_2.exp_up());
        for &x in &[-3.5, -1.0, -0.1, 0.0, 0.1, 1.0, 2.5, 10.0, 100.0] {
            brackets_central(x);
        }
    }

    #[test]
    fn ln_brackets_known_constants() {
        // ln 1 = 0, ln e = 1, ln 2 = LN_2.
        assert!(1.0_f64.ln_down() <= 0.0 && 0.0 <= 1.0_f64.ln_up());
        assert!(E.ln_down() <= 1.0 && 1.0 <= E.ln_up());
        assert!(2.0_f64.ln_down() <= LN_2 && LN_2 <= 2.0_f64.ln_up());
        for &x in &[1.0e-8, 0.5, 1.0, 1.5, 2.0, 10.0, 1.0e9] {
            let lo = x.ln_down();
            let hi = x.ln_up();
            let r = libm::log(x);
            assert!(lo <= r && r <= hi && lo <= hi, "ln bracket failed at {x}");
        }
    }

    #[test]
    fn exp_round_trips_through_ln() {
        // exp(ln x) must enclose x for x > 0: the lower exp of the lower ln, up
        // to the upper exp of the upper ln, brackets x.
        for &x in &[0.25, 1.0, 3.0, 50.0] {
            let lo = x.ln_down().exp_down();
            let hi = x.ln_up().exp_up();
            assert!(lo <= x && x <= hi, "exp(ln({x})) = [{lo}, {hi}] lost x");
        }
    }

    #[test]
    fn exp_underflow_and_overflow_stay_sound() {
        // Underflow: lower bound zero, upper bound a positive subnormal.
        // (Compared with `== 0.0` rather than `assert_eq!`: clippy's float_cmp
        // exempts a literal-zero comparison, but not the macro's by-reference one.)
        assert!((-800.0_f64).exp_down() == 0.0);
        assert!((-800.0_f64).exp_up() > 0.0);
        // Overflow: finite lower bound, infinite upper bound.
        assert!((710.0_f64).exp_down().is_finite());
        assert!((710.0_f64).exp_up().is_infinite());
    }

    #[test]
    fn ln_domain_edges_follow_libm() {
        assert!(0.0_f64.ln_up().is_infinite() && 0.0_f64.ln_up() < 0.0); // ln 0 = -inf
        assert!((-1.0_f64).ln_down().is_nan()); // ln of negative is undefined
    }

    // The integer-rounding results are exact integral values, so equality is the
    // property under test; the float_cmp lint (aimed at comparing rounded
    // approximations) does not apply.
    #[test]
    #[allow(clippy::float_cmp)]
    fn floor_ceil_trunc_are_exact_directed_rounding() {
        assert_eq!(RoundInteger::floor(1.9_f64), 1.0);
        assert_eq!(RoundInteger::floor(-1.1_f64), -2.0);
        assert_eq!(RoundInteger::ceil(1.1_f64), 2.0);
        assert_eq!(RoundInteger::ceil(-1.9_f64), -1.0);
        assert_eq!(RoundInteger::trunc(1.9_f64), 1.0);
        assert_eq!(RoundInteger::trunc(-1.9_f64), -1.0); // toward zero, not -2
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn round_ties_split_even_from_away() {
        // A half-integer is the tie the two rules resolve differently.
        assert_eq!(RoundInteger::round_ties_to_even(2.5_f64), 2.0); // to even
        assert_eq!(RoundInteger::round_ties_to_away(2.5_f64), 3.0); // away from zero
        assert_eq!(RoundInteger::round_ties_to_even(3.5_f64), 4.0); // to even
        assert_eq!(RoundInteger::round_ties_to_away(3.5_f64), 4.0); // away agrees here
        assert_eq!(RoundInteger::round_ties_to_even(-2.5_f64), -2.0);
        assert_eq!(RoundInteger::round_ties_to_away(-2.5_f64), -3.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn integer_rounding_preserves_integers_and_infinities() {
        // Above the contiguous-integer range every finite value is integral, and
        // the operations are the identity there; infinities pass through.
        assert_eq!(RoundInteger::floor(f64::MAX), f64::MAX);
        assert_eq!(RoundInteger::ceil(f64::MAX), f64::MAX);
        assert_eq!(RoundInteger::round_ties_to_even(f64::MAX), f64::MAX);
        assert!(RoundInteger::floor(f64::INFINITY).is_infinite());
        assert!(RoundInteger::ceil(f64::NEG_INFINITY).is_infinite());
    }

    // Exact-value assertions on clamp and bracket endpoints; the float_cmp lint
    // targets comparisons of rounded approximations, not these theorems.
    #[test]
    #[allow(clippy::float_cmp)]
    fn pi_bracket_encloses_the_true_pi() {
        let lo = f64::pi_down();
        let hi = f64::pi_up();
        // The true pi to 30 significant digits is 3.14159265358979323846264338328;
        // the binary64 nearest is 3.14159265358979311599796346854 (below pi), and
        // its successor is 3.14159265358979356009063455953 (above pi). So the pair
        // strictly brackets pi. We cannot name pi exactly in f64, but we can check
        // the bracket is ordered, one ulp wide, and straddles the nearest float.
        assert!(lo < hi, "pi bracket must be ordered");
        assert_eq!(hi, lo.next_up(), "pi bracket must be the tightest, one ulp");
        assert_eq!(lo, core::f64::consts::PI);
        // The nearest binary64 to pi is below pi (see the decimal expansions on
        // `pi_down`), so `lo` is a sound lower endpoint and its successor `hi` a
        // sound upper endpoint. Cross-check the reduction identity a reduction
        // relies on: doubling the bracket must straddle 2*pi, the smallest period
        // shift, i.e. `2*lo <= 2*pi <= 2*hi`, which holds because doubling is exact
        // and monotone.
        assert!(lo + lo < hi + hi);
    }

    #[test]
    fn sin_cos_brackets_and_range_clamp() {
        // At pi/2 (approximately) sin peaks at 1 and cos crosses 0.
        let h = core::f64::consts::FRAC_PI_2;
        assert!(h.sin_down() <= libm::sin(h) && libm::sin(h) <= h.sin_up());
        assert!(h.cos_down() <= libm::cos(h) && libm::cos(h) <= h.cos_up());
        // Range clamp: no sin/cos bound escapes [-1, 1].
        for &x in &[-3.3, -1.0, 0.0, 0.7, 1.5, 3.2, 10.0, 1.0e6] {
            assert!(
                x.sin_down() >= -1.0 && x.sin_up() <= 1.0,
                "sin range at {x}"
            );
            assert!(
                x.cos_down() >= -1.0 && x.cos_up() <= 1.0,
                "cos range at {x}"
            );
            assert!(x.sin_down() <= libm::sin(x) && libm::sin(x) <= x.sin_up());
            assert!(x.cos_down() <= libm::cos(x) && libm::cos(x) <= x.cos_up());
        }
    }

    #[test]
    fn tan_brackets_near_and_away_from_a_pole() {
        // Just below pi/2 (the nearest float FRAC_PI_2 sits below the true pole),
        // tan is a large finite value the fixture brackets, never a manufactured
        // infinity.
        let near = core::f64::consts::FRAC_PI_2;
        assert!(near.tan_down().is_finite() && near.tan_up().is_finite());
        assert!(near.tan_down() <= libm::tan(near) && libm::tan(near) <= near.tan_up());
        for &x in &[-1.0, -0.3, 0.0, 0.4, 1.2] {
            assert!(x.tan_down() <= libm::tan(x) && libm::tan(x) <= x.tan_up());
        }
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn hyperbolic_brackets_and_clamps() {
        for &x in &[-2.5, -1.0, -0.1, 0.0, 0.1, 1.0, 3.0] {
            assert!(x.sinh_down() <= libm::sinh(x) && libm::sinh(x) <= x.sinh_up());
            assert!(x.cosh_down() <= libm::cosh(x) && libm::cosh(x) <= x.cosh_up());
            assert!(x.tanh_down() <= libm::tanh(x) && libm::tanh(x) <= x.tanh_up());
            // cosh >= 1 and tanh in [-1, 1] after clamping.
            assert!(x.cosh_down() >= 1.0, "cosh floor at {x}");
            assert!(
                x.tanh_down() >= -1.0 && x.tanh_up() <= 1.0,
                "tanh range at {x}"
            );
        }
        // cosh(0) minimum is exactly 1 after the clamp.
        assert!(0.0_f64.cosh_down() == 1.0);
        // tanh saturates: a large argument clamps the near bound to the range edge.
        assert!(20.0_f64.tanh_up() == 1.0);
        assert!((-20.0_f64).tanh_down() == -1.0);
        // Infinite arguments give the exact saturation values.
        assert!(f64::INFINITY.tanh_up() == 1.0);
        assert!(f64::NEG_INFINITY.tanh_down() == -1.0);
    }

    #[test]
    fn sinh_cosh_overflow_edges_follow_exp_precedent() {
        // Past the overflow shoulder the down bound stays finite, the up bound goes
        // infinite (positive side); mirrored on the negative side for sinh.
        assert!(800.0_f64.sinh_down().is_finite());
        assert!(800.0_f64.sinh_up().is_infinite());
        assert!((-800.0_f64).sinh_up().is_finite());
        assert!((-800.0_f64).sinh_down().is_infinite());
        assert!(800.0_f64.cosh_down().is_finite());
        assert!(800.0_f64.cosh_up().is_infinite());
    }

    #[test]
    fn pow_brackets_known_values() {
        // 2^10 = 1024, 9^0.5 = 3, x^0 = 1.
        assert!(2.0_f64.pow_down(10.0) <= 1024.0 && 1024.0 <= 2.0_f64.pow_up(10.0));
        assert!(9.0_f64.pow_down(0.5) <= 3.0 && 3.0 <= 9.0_f64.pow_up(0.5));
        assert!(5.0_f64.pow_down(0.0) <= 1.0 && 1.0 <= 5.0_f64.pow_up(0.0));
        for &(x, e) in &[(0.1, 2.5), (0.5, -2.5), (1.5, 3.0), (10.0, -0.3)] {
            let r = libm::pow(x, e);
            assert!(
                x.pow_down(e) <= r && r <= x.pow_up(e),
                "pow bracket at {x}^{e}"
            );
        }
        // Overflow: finite lower bound, infinite upper bound.
        assert!(10.0_f64.pow_down(400.0).is_finite());
        assert!(10.0_f64.pow_up(400.0).is_infinite());
    }

    #[test]
    fn rootn_brackets_and_parity() {
        // Exact perfect roots are bracketed (loose, since the exponent rounds).
        assert!(27.0_f64.rootn_down(3) <= 3.0 && 3.0 <= 27.0_f64.rootn_up(3));
        assert!(1024.0_f64.rootn_down(10) <= 2.0 && 2.0 <= 1024.0_f64.rootn_up(10));
        assert!(0.0_f64.rootn_down(4) == 0.0 && 0.0_f64.rootn_up(4) == 0.0);
        // A base below one still brackets (the decreasing-in-exponent branch).
        let r = libm::pow(0.5, 1.0 / 3.0);
        assert!(0.5_f64.rootn_down(3) <= r && r <= 0.5_f64.rootn_up(3));
        // Odd root of a negative base is real; even root of one is NaN (out of
        // domain, caller/interval owns the precondition).
        assert!((-8.0_f64).rootn_down(3) <= -2.0 && -2.0 <= (-8.0_f64).rootn_up(3));
        assert!((-8.0_f64).rootn_down(2).is_nan());
    }
}
