//! The `f64` instance of [`RoundFloat`]: a verification and host-test fixture.
//!
//! This instance is NOT a production interval float. It exists so the enclosure
//! laws can be checked two ways the correctly-rounded backends cannot support:
//! by CBMC under Kani (which models `f64` bit-precisely but cannot model a
//! multiword decimal) and by host property tests with no heavy dependency.
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
//! soundness-not-tightness trade of Law 6 in [`crate::spec`]. `sqrt` uses
//! `libm::sqrt`, which is correctly rounded, then steps outward the same way.

use crate::round::RoundFloat;

impl RoundFloat for f64 {
    const INFINITY: Self = f64::INFINITY;
    const NEG_INFINITY: Self = f64::NEG_INFINITY;
    const ZERO: Self = 0.0;

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
