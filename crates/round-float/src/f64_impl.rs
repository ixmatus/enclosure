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

use crate::{RoundFloat, RoundTranscendental};

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

#[cfg(test)]
mod tests {
    use crate::RoundTranscendental;
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
}
