//! `round-float`: the directed-rounding float contract for rigorous numerics.
//!
//! [`RoundFloat`] is the seam between a correctly-rounded floating-point backend
//! and the rigorous-enclosure crates built on top of it. The interval crate
//! `interval-1788` and the affine-arithmetic crate `affine-arith` are both
//! generic over it, so one backend (ferrodec `Decimal128`, pfloat's
//! arbitrary-precision floats, or the `f64` fixture shipped here) serves every
//! enclosure shape. The trait is deliberately the common core of what
//! outward-rounded arithmetic needs, no larger.
//!
//! # Why split directions instead of a rounding-mode parameter
//!
//! Outward rounding fixes the direction statically: a lower endpoint always
//! rounds toward minus infinity, an upper endpoint always toward plus infinity.
//! Exposing [`add_down`](RoundFloat::add_down) and [`add_up`](RoundFloat::add_up)
//! rather than `add(self, rhs, mode)` makes that direction a property of the
//! call site, so the rigor-critical four-corner multiplication cannot round a
//! corner the wrong way by passing the wrong mode. It also keeps the trait free
//! of any backend's rounding-mode enum (ferrodec's and pfloat's are distinct
//! types), which is what lets one trait span both.
//!
//! # The contract each method must honor
//!
//! For finite inputs, `x.add_down(y)` returns a float that is less than or equal
//! to the exact real sum `x + y`, and `x.add_up(y)` returns one greater than or
//! equal to it; likewise for `sub`, `mul`, `div`, and `sqrt`. A backend that is
//! correctly rounded toward minus and plus infinity makes these tight (the
//! returned float is the nearest representable bound). A backend that only
//! guarantees soundness (the `f64` fixture) may return a looser bound, but never
//! one on the wrong side. Soundness is the load-bearing obligation; tightness is
//! a quality the backend may add.
//!
//! # The f64 instance
//!
//! Behind the `f64` feature, this crate provides a [`RoundFloat`] instance for
//! `f64` that is sound but deliberately not tight: it computes each operation in
//! round-to-nearest and steps the result one float outward with `next_down` /
//! `next_up`. It exists so the enclosure laws can be machine-checked under Kani
//! (which models `f64` bit-precisely, where it cannot model a multiword decimal)
//! and exercised by host property tests, in both `interval-1788` and
//! `affine-arith`, with no heavy dependency. A correctly-rounded backend makes
//! the same operations tight; the fixture never does, by design.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "f64")]
mod f64_impl;

/// A floating-point type that supports the outward-directed arithmetic rigorous
/// enclosure methods are built on.
///
/// `Self::INFINITY` and `Self::NEG_INFINITY` are the extended-real endpoints
/// used to represent unbounded enclosures. The predicates classify values during
/// construction and operation. See the [module docs](self) for the rounding
/// contract every arithmetic method must honor.
pub trait RoundFloat: Copy + PartialOrd {
    /// Positive infinity, the upper endpoint of a value unbounded above.
    const INFINITY: Self;
    /// Negative infinity, the lower endpoint of a value unbounded below.
    const NEG_INFINITY: Self;
    /// The additive identity (`+0`).
    const ZERO: Self;
    /// The multiplicative identity (`1`). Used to form reciprocals as `1 / x`.
    const ONE: Self;

    /// A lower bound on the exact sum `self + rhs`.
    fn add_down(self, rhs: Self) -> Self;
    /// An upper bound on the exact sum `self + rhs`.
    fn add_up(self, rhs: Self) -> Self;

    /// A lower bound on the exact difference `self - rhs`.
    fn sub_down(self, rhs: Self) -> Self;
    /// An upper bound on the exact difference `self - rhs`.
    fn sub_up(self, rhs: Self) -> Self;

    /// A lower bound on the exact product `self * rhs`.
    fn mul_down(self, rhs: Self) -> Self;
    /// An upper bound on the exact product `self * rhs`.
    fn mul_up(self, rhs: Self) -> Self;

    /// A lower bound on the exact quotient `self / rhs`.
    fn div_down(self, rhs: Self) -> Self;
    /// An upper bound on the exact quotient `self / rhs`.
    fn div_up(self, rhs: Self) -> Self;

    /// A lower bound on the exact square root `sqrt(self)`.
    fn sqrt_down(self) -> Self;
    /// An upper bound on the exact square root `sqrt(self)`.
    fn sqrt_up(self) -> Self;

    /// Exact negation (a sign flip, never rounded). Negating an endpoint to form
    /// `-[a, b] = [-b, -a]` loses no information, so it does not need a direction.
    /// Named `negate` rather than `neg` to avoid clashing with `core::ops::Neg`
    /// on backends (such as `f64`) that also implement the operator.
    fn negate(self) -> Self;

    /// Whether the value is NaN.
    fn is_nan(self) -> bool;
    /// Whether the value is finite (neither infinite nor NaN).
    fn is_finite(self) -> bool;
    /// Whether the value is positive or negative infinity.
    fn is_infinite(self) -> bool;
    /// Whether the value carries a negative sign (including `-0`).
    fn is_sign_negative(self) -> bool;
    /// Whether the value is `+0` or `-0`.
    fn is_zero(self) -> bool;

    /// The lesser of two non-NaN values.
    ///
    /// Used to reduce the four corners of a product or quotient to a lower
    /// bound. Both arguments are non-NaN at every call site (the enclosure
    /// endpoints are non-NaN by construction), so the partial order is total
    /// here.
    #[must_use]
    fn rmin(self, rhs: Self) -> Self {
        if rhs < self {
            rhs
        } else {
            self
        }
    }

    /// The greater of two non-NaN values. Companion of [`rmin`](RoundFloat::rmin).
    #[must_use]
    fn rmax(self, rhs: Self) -> Self {
        if rhs > self {
            rhs
        } else {
            self
        }
    }
}

/// Outward-directed transcendental functions, layered above [`RoundFloat`].
///
/// `exp` and `ln` are not part of the common core [`RoundFloat`] models. The
/// interval and affine crates need them only for their elementary-function
/// surfaces, not for the basic arithmetic every backend must provide, and a
/// backend cannot always supply a correctly-rounded transcendental as cheaply
/// as it supplies `add` or `mul`. Holding them in a separate trait lets a
/// backend implement them on its own schedule, or not at all, without enlarging
/// the core contract every backend must satisfy. See round-float decision
/// record 0001.
///
/// # The contract each method must honor
///
/// For a finite `self`, [`exp_down`](RoundTranscendental::exp_down) returns a
/// float less than or equal to the exact real `e^self`, and
/// [`exp_up`](RoundTranscendental::exp_up) one greater than or equal to it. For
/// `self > 0`, [`ln_down`](RoundTranscendental::ln_down) and
/// [`ln_up`](RoundTranscendental::ln_up) bracket the exact `ln(self)` the same
/// way. A non-positive argument to `ln` is outside the domain: an implementation
/// may return any value there (the f64 fixture follows `libm`, returning NaN for
/// a negative argument and negative infinity at zero), and the caller owns the
/// `self > 0` precondition. As with [`RoundFloat`], soundness (the bound is
/// never on the wrong side) is the obligation; tightness is a quality a
/// correctly-rounded backend may add.
pub trait RoundTranscendental: RoundFloat {
    /// A lower bound on the exact `e^self`.
    fn exp_down(self) -> Self;
    /// An upper bound on the exact `e^self`.
    fn exp_up(self) -> Self;
    /// A lower bound on the exact `ln(self)` (requires `self > 0`).
    fn ln_down(self) -> Self;
    /// An upper bound on the exact `ln(self)` (requires `self > 0`).
    fn ln_up(self) -> Self;
}
