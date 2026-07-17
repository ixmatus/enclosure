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
//!
//! # The tight f64 instance
//!
//! Behind the `f64-tight` feature, this crate provides `TightF64`, a newtype
//! over `f64` whose [`RoundFloat`] instance is correctly rounded: every directed
//! method returns the nearest representable bound on the correct side. It derives
//! the exact sign of each operation's rounding error from an error-free transform
//! (`TwoSum`, `TwoProduct` via `libm::fma`, and FMA residuals) and steps one float
//! outward only when the exact result is unrepresentable. This is the fast native
//! tight backend the conformance lane needs; the fixture and the tight instance
//! coexist because they are distinct types. The `tight_f64` module docs carry the
//! numbered laws; see round-float decision record 0002.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "f64")]
mod f64_impl;

#[cfg(feature = "f64-tight")]
mod tight_f64;

#[cfg(feature = "f64-tight")]
pub use tight_f64::TightF64;

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

/// Rounding of a float to an integral value, layered above [`RoundFloat`].
///
/// These are the point functions that map a float to a nearby integer: round
/// toward minus infinity ([`floor`](RoundInteger::floor)), toward plus infinity
/// ([`ceil`](RoundInteger::ceil)), toward zero ([`trunc`](RoundInteger::trunc)),
/// and to the nearest integer with the two standard tie rules
/// ([`round_ties_to_even`](RoundInteger::round_ties_to_even),
/// [`round_ties_to_away`](RoundInteger::round_ties_to_away)). The interval crate
/// needs them for the IEEE 1788 integer-rounding point functions; they are not
/// part of the common core [`RoundFloat`], so a backend opts in by implementing
/// this trait, exactly as it opts into [`RoundTranscendental`] (round-float
/// decision record 0001, workspace decision record 0005 on per-family capability
/// traits).
///
/// # Every method is exact, so there is no down/up split
///
/// Each result is an integral value that is representable in any binary or
/// decimal floating-point format the backend might use: the integers within a
/// format's contiguous-integer range are all representable, and above that range
/// every finite value is already integral, so rounding to an integer returns it
/// unchanged. The operation therefore never rounds, and there is no lower/upper
/// bound to distinguish. This is the same reason [`negate`](RoundFloat::negate)
/// is a single undirected method rather than a `negate_down`/`negate_up` pair: an
/// exact operation carries no rounding direction. The interval layer relies on
/// the exactness, taking the image of `[a, b]` under a monotone step function as
/// `[op(a), op(b)]` with no outward widening.
///
/// # Accepted duplication with `RoundTrig`
///
/// Workspace decision record 0005 gives the trigonometric trait `RoundTrig` its
/// own exact `floor` for argument reduction (extracting the multiple of pi). A
/// backend that implements both `RoundInteger` and `RoundTrig` therefore supplies
/// `floor` twice. The duplication is accepted, the same trade that record makes
/// for the pi pair: keeping [`RoundFloat`] frozen and each capability trait
/// self-contained is worth one repeated method. See round-float decision record
/// 0003.
pub trait RoundInteger: RoundFloat {
    /// The largest integral value not greater than `self` (round toward minus
    /// infinity). Exact.
    fn floor(self) -> Self;
    /// The smallest integral value not less than `self` (round toward plus
    /// infinity). Exact.
    fn ceil(self) -> Self;
    /// The integral part of `self`, dropping the fraction (round toward zero).
    /// Exact.
    fn trunc(self) -> Self;
    /// The nearest integral value, with a tie (a half-integer) resolved to the
    /// even neighbor. This is IEEE 754 `roundToIntegralTiesToEven`. Exact.
    fn round_ties_to_even(self) -> Self;
    /// The nearest integral value, with a tie (a half-integer) resolved to the
    /// neighbor of larger magnitude. This is IEEE 754
    /// `roundToIntegralTiesToAway`. Exact.
    fn round_ties_to_away(self) -> Self;
}

/// Outward-directed trigonometric functions, layered above [`RoundFloat`].
///
/// The three families `sin`/`cos`, `tan`, and the reduction constants ride one
/// trait because argument reduction couples them: reducing an argument modulo a
/// period is a subtraction of multiples of pi against a constant no radix-two or
/// radix-ten format represents exactly, so a backend that offers `sin` at all
/// owes an enclosure of pi ([`pi_down`](RoundTrig::pi_down) /
/// [`pi_up`](RoundTrig::pi_up)) and an exact [`floor`](RoundTrig::floor) (to
/// extract the multiple). A backend that never implements trigonometry owes none
/// of that, which is why the constants live here and not on the core. See
/// workspace decision record 0005.
///
/// # The contract each method must honor
///
/// For a finite `self`, [`sin_down`](RoundTrig::sin_down) returns a float less
/// than or equal to the exact real `sin(self)`, and
/// [`sin_up`](RoundTrig::sin_up) one greater than or equal to it; likewise for
/// `cos` and `tan`. The results of `sin` and `cos` lie in `[-1, 1]` and of `tan`
/// in the extended reals (near a pole `tan` returns the large finite value the
/// backend computes; the interval layer, not the fixture, manufactures the
/// unbounded pole semantics). As with [`RoundFloat`], soundness (the bound is
/// never on the wrong side) is the obligation and tightness is a quality a
/// correctly-rounded backend may add; here the trust in soundness rests on the
/// backend's argument reduction staying faithful at every finite magnitude (for
/// the f64 fixture, musl's Payne-Hanek reduction; see the `musl-libm-accuracy`
/// reference entry).
pub trait RoundTrig: RoundFloat {
    /// A lower bound on the exact `sin(self)`.
    fn sin_down(self) -> Self;
    /// An upper bound on the exact `sin(self)`.
    fn sin_up(self) -> Self;
    /// A lower bound on the exact `cos(self)`.
    fn cos_down(self) -> Self;
    /// An upper bound on the exact `cos(self)`.
    fn cos_up(self) -> Self;
    /// A lower bound on the exact `tan(self)` (the caller owns the precondition
    /// that `self` is not at a pole; the interval layer supplies the pole
    /// semantics).
    fn tan_down(self) -> Self;
    /// An upper bound on the exact `tan(self)`.
    fn tan_up(self) -> Self;

    /// The largest backend float not exceeding the mathematical pi: the lower
    /// endpoint of the format's enclosure of pi, used to reduce arguments.
    fn pi_down() -> Self;
    /// The smallest backend float not below the mathematical pi: the upper
    /// endpoint of the format's enclosure of pi.
    fn pi_up() -> Self;

    /// The largest integral value not greater than `self` (round toward minus
    /// infinity). Exact in every float format.
    ///
    /// This duplicates [`RoundInteger::floor`] by workspace decision record
    /// 0005's accepted trade: the reduction obligation is a trigonometric one, so
    /// `RoundTrig` carries its own `floor` rather than depend on `RoundInteger`,
    /// keeping each capability trait self-contained and [`RoundFloat`] frozen. A
    /// backend implementing both traits supplies `floor` twice.
    fn floor(self) -> Self;
}

/// Outward-directed hyperbolic functions, layered above [`RoundFloat`].
///
/// The family is reduction-free (no periodic argument reduction and so no pi
/// enclosure), which is why it is a trait separate from [`RoundTrig`]: a backend
/// lands the two families on independent schedules. See workspace decision
/// record 0005.
///
/// # The contract each method must honor
///
/// For a finite `self`, [`sinh_down`](RoundHyperbolic::sinh_down) returns a float
/// less than or equal to the exact real `sinh(self)` and
/// [`sinh_up`](RoundHyperbolic::sinh_up) one greater than or equal to it;
/// likewise for `cosh` and `tanh`. The result of `cosh` is at least `1`, of
/// `tanh` in `(-1, 1)`, and of `sinh` in the extended reals (an argument past the
/// overflow shoulder gives the `[largest_finite, +inf]` style bounds the exp
/// fixture established). Soundness is the obligation; tightness is a backend
/// quality.
pub trait RoundHyperbolic: RoundFloat {
    /// A lower bound on the exact `sinh(self)`.
    fn sinh_down(self) -> Self;
    /// An upper bound on the exact `sinh(self)`.
    fn sinh_up(self) -> Self;
    /// A lower bound on the exact `cosh(self)`.
    fn cosh_down(self) -> Self;
    /// An upper bound on the exact `cosh(self)`.
    fn cosh_up(self) -> Self;
    /// A lower bound on the exact `tanh(self)`.
    fn tanh_down(self) -> Self;
    /// An upper bound on the exact `tanh(self)`.
    fn tanh_up(self) -> Self;
}

/// Outward-directed power and root functions, layered above [`RoundFloat`].
///
/// [`pow_down`](RoundPow::pow_down) / [`pow_up`](RoundPow::pow_up) bound the real
/// power `self^exp`, and [`rootn_down`](RoundPow::rootn_down) /
/// [`rootn_up`](RoundPow::rootn_up) bound the `n`-th root `self^(1/n)`. See
/// workspace decision record 0005.
///
/// # The contract each method must honor, and the caller-owned preconditions
///
/// For `pow`, the real power `self^exp` is defined on `self > 0` (all `exp`) and
/// at `self == 0` for `exp > 0` (value zero). The caller owns that domain: with
/// `self > 0`, [`pow_down`](RoundPow::pow_down) returns a float less than or equal
/// to `self^exp` and [`pow_up`](RoundPow::pow_up) one greater than or equal to it;
/// the interval layer owns the set semantics of the edges (a base reaching zero, a
/// negative base, `0^0`). A negative base is outside the real domain and an
/// implementation may return any value there (the f64 fixture passes libm's NaN
/// through).
///
/// For `rootn`, `n` is the root order and must be at least `1`; `self^(1/n)` is
/// defined for all `self` when `n` is odd and for `self >= 0` when `n` is even.
/// The caller owns `n >= 1`; the parity domain restriction (an even root of a
/// negative base) is the interval layer's to enforce, and an out-of-domain scalar
/// argument may return any value (the fixture returns NaN). As with
/// [`RoundFloat`], soundness is the obligation and tightness a backend quality.
pub trait RoundPow: RoundFloat {
    /// A lower bound on the exact real power `self^exp` (requires `self >= 0`).
    fn pow_down(self, exp: Self) -> Self;
    /// An upper bound on the exact real power `self^exp` (requires `self >= 0`).
    fn pow_up(self, exp: Self) -> Self;
    /// A lower bound on the exact `n`-th root `self^(1/n)` (requires `n >= 1`; an
    /// even `n` requires `self >= 0`).
    fn rootn_down(self, n: u32) -> Self;
    /// An upper bound on the exact `n`-th root `self^(1/n)` (requires `n >= 1`; an
    /// even `n` requires `self >= 0`).
    fn rootn_up(self, n: u32) -> Self;
}

/// Outward-directed inverse trigonometric functions, layered above
/// [`RoundFloat`].
///
/// The family is `asin`, `acos`, `atan`, and the two-argument `atan2`, plus the
/// same π enclosure [`RoundTrig`] carries. π rides this trait for the same
/// reason it rides [`RoundTrig`]: the range clamps of the inverse functions are
/// stated in π (`asin`/`atan` into the ±π/2 brackets, `acos` into `[0, π]`,
/// `atan2` into `(-π, π]`), and a backend implementing only this family owes
/// only this enclosure. A backend implementing both `RoundTrig` and this trait
/// supplies the π pair twice; the duplication is the accepted cost of keeping
/// the traits independent, exactly the [`RoundTrig`] `floor` trade. See
/// workspace decision record 0007.
///
/// # The contract each method must honor, and the caller-owned domains
///
/// For a finite `self`, [`asin_down`](RoundInverseTrig::asin_down) returns a
/// float less than or equal to the exact real `asin(self)` and
/// [`asin_up`](RoundInverseTrig::asin_up) one greater than or equal to it;
/// likewise for `acos` and `atan`. `asin` and `acos` are defined on `[-1, 1]`,
/// `atan` on the whole line. An out-of-domain argument to `asin`/`acos` is the
/// caller's to guard: the f64 fixture passes `libm`'s NaN through, and the
/// interval layer owns the domain restriction. [`atan2_down`](RoundInverseTrig::atan2_down)
/// and [`atan2_up`](RoundInverseTrig::atan2_up) bound `atan2(self, other)`, the
/// principal angle of the point `(other, self)` with `self` the ordinate (`y`)
/// and `other` the abscissa (`x`), matching `f64::atan2`; the origin `(0, 0)` is
/// outside the domain and the interval layer owns its set semantics. Soundness
/// (the bound is never on the wrong side) is the obligation; tightness is a
/// quality a correctly-rounded backend may add.
pub trait RoundInverseTrig: RoundFloat {
    /// A lower bound on the exact `asin(self)` (requires `self` in `[-1, 1]`).
    fn asin_down(self) -> Self;
    /// An upper bound on the exact `asin(self)` (requires `self` in `[-1, 1]`).
    fn asin_up(self) -> Self;
    /// A lower bound on the exact `acos(self)` (requires `self` in `[-1, 1]`).
    fn acos_down(self) -> Self;
    /// An upper bound on the exact `acos(self)` (requires `self` in `[-1, 1]`).
    fn acos_up(self) -> Self;
    /// A lower bound on the exact `atan(self)`.
    fn atan_down(self) -> Self;
    /// An upper bound on the exact `atan(self)`.
    fn atan_up(self) -> Self;
    /// A lower bound on the exact `atan2(self, other)` (the angle of `(other,
    /// self)`; the caller owns the precondition that `(other, self)` is not the
    /// origin).
    fn atan2_down(self, other: Self) -> Self;
    /// An upper bound on the exact `atan2(self, other)`.
    fn atan2_up(self, other: Self) -> Self;

    /// The largest backend float not exceeding the mathematical π: the lower
    /// endpoint of the format's enclosure of π.
    ///
    /// This duplicates [`RoundTrig::pi_down`] by workspace decision record
    /// 0007's accepted trade, so a backend implementing only inverse trig owes
    /// no dependence on [`RoundTrig`].
    fn pi_down() -> Self;
    /// The smallest backend float not below the mathematical π: the upper
    /// endpoint of the format's enclosure of π.
    fn pi_up() -> Self;
}

/// Outward-directed inverse hyperbolic functions, layered above [`RoundFloat`].
///
/// The family is `asinh`, `acosh`, `atanh`, reduction-free and constant-free.
/// It is a trait separate from [`RoundInverseTrig`] and [`RoundExpBases`]
/// because a backend lands the families on independent schedules (musl itself
/// treats the arc hyperbolics differently from the inverse trig set: they sit
/// among the named exceptions to its accuracy goal). See workspace decision
/// record 0007.
///
/// # The contract each method must honor, and the caller-owned domains
///
/// For a finite `self`, [`asinh_down`](RoundInverseHyperbolic::asinh_down)
/// returns a float less than or equal to the exact real `asinh(self)` and
/// [`asinh_up`](RoundInverseHyperbolic::asinh_up) one greater than or equal to
/// it; likewise for `acosh` and `atanh`. `asinh` is defined on the whole line,
/// `acosh` on `[1, +inf)`, `atanh` on the open `(-1, 1)`. An out-of-domain
/// argument is the caller's to guard: the f64 fixture passes `libm`'s sentinel
/// through (`acosh` below 1, `atanh` at or outside `±1`), and the interval layer
/// owns the domain restriction and the unbounded results at the `atanh`
/// endpoints. Soundness is the obligation; tightness is a backend quality.
pub trait RoundInverseHyperbolic: RoundFloat {
    /// A lower bound on the exact `asinh(self)`.
    fn asinh_down(self) -> Self;
    /// An upper bound on the exact `asinh(self)`.
    fn asinh_up(self) -> Self;
    /// A lower bound on the exact `acosh(self)` (requires `self >= 1`).
    fn acosh_down(self) -> Self;
    /// An upper bound on the exact `acosh(self)` (requires `self >= 1`).
    fn acosh_up(self) -> Self;
    /// A lower bound on the exact `atanh(self)` (requires `self` in `(-1, 1)`).
    fn atanh_down(self) -> Self;
    /// An upper bound on the exact `atanh(self)` (requires `self` in `(-1, 1)`).
    fn atanh_up(self) -> Self;
}

/// Outward-directed base-2 and base-10 exponential and logarithm functions,
/// layered above [`RoundFloat`].
///
/// The family is `exp2`, `exp10`, `log2`, `log10`, independent of both the
/// natural `exp`/`ln` of [`RoundTranscendental`] and the other round-two
/// families, and constant-free. See workspace decision record 0007.
///
/// # The contract each method must honor, and the caller-owned domains
///
/// For a finite `self`, [`exp2_down`](RoundExpBases::exp2_down) returns a float
/// less than or equal to the exact real `2^self` and
/// [`exp2_up`](RoundExpBases::exp2_up) one greater than or equal to it; likewise
/// for `exp10` (`10^self`). `log2` and `log10` are defined on `(0, +inf)`; a
/// non-positive argument is the caller's to guard, and the f64 fixture passes
/// `libm`'s sentinel through (NaN for a negative argument, negative infinity at
/// zero), exactly as `ln` does. Soundness is the obligation; tightness is a
/// backend quality.
pub trait RoundExpBases: RoundFloat {
    /// A lower bound on the exact `2^self`.
    fn exp2_down(self) -> Self;
    /// An upper bound on the exact `2^self`.
    fn exp2_up(self) -> Self;
    /// A lower bound on the exact `10^self`.
    fn exp10_down(self) -> Self;
    /// An upper bound on the exact `10^self`.
    fn exp10_up(self) -> Self;
    /// A lower bound on the exact `log2(self)` (requires `self > 0`).
    fn log2_down(self) -> Self;
    /// An upper bound on the exact `log2(self)` (requires `self > 0`).
    fn log2_up(self) -> Self;
    /// A lower bound on the exact `log10(self)` (requires `self > 0`).
    fn log10_down(self) -> Self;
    /// An upper bound on the exact `log10(self)` (requires `self > 0`).
    fn log10_up(self) -> Self;
}
