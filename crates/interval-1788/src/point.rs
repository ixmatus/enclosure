//! Point functions over intervals: absolute value, the two binary lattice
//! operations, the sign function, and the four integer-rounding functions.
//!
//! Every function here is the image of a scalar point function over the input
//! box, and every scalar function is monotone (non-decreasing) or piecewise so,
//! which is what makes the interval image an endpoint computation rather than a
//! search. Two groups sit at different backend requirements:
//!
//! - [`abs`](Interval::abs), [`min`](Interval::min), [`max`](Interval::max), and
//!   [`sign`](Interval::sign) need only [`RoundFloat`]: they select or flip
//!   endpoints, and produce their constant values (`-1`, `0`, `1`) exactly.
//! - [`ceil`](Interval::ceil), [`floor`](Interval::floor),
//!   [`trunc`](Interval::trunc),
//!   [`round_ties_to_even`](Interval::round_ties_to_even), and
//!   [`round_ties_to_away`](Interval::round_ties_to_away) need the
//!   [`RoundInteger`] extension trait, whose methods are exact, so their images
//!   are exact too: no outward widening, even on the sound-not-tight fixture.
//!
//! The empty interval maps to the empty interval under every one of them (the
//! image of the empty set is empty). No rounding occurs anywhere in this module;
//! the decorated forms, which introduce the `def` decoration for the
//! discontinuous members, live in [`decorated`](crate::decorated).

use crate::interval::Interval;
use round_float::{RoundFloat, RoundInteger};

/// The absolute value of an endpoint. Exact: a sign flip or nothing. Maps an
/// infinite endpoint to positive infinity, and `-0` to `+0`. Endpoints are never
/// NaN.
#[inline]
fn fabs<F: RoundFloat>(x: F) -> F {
    if x.is_sign_negative() {
        x.negate()
    } else {
        x
    }
}

/// The sign of an endpoint as one of the three exact constants `-1`, `0`, `+1`.
/// A zero of either sign maps to `+0`, since the comparisons treat `-0` and `+0`
/// as equal and neither is strictly negative or positive.
#[inline]
fn fsign<F: RoundFloat>(x: F) -> F {
    if x < F::ZERO {
        F::ONE.negate()
    } else if x > F::ZERO {
        F::ONE
    } else {
        F::ZERO
    }
}

impl<F: RoundFloat> Interval<F> {
    /// The absolute value, the set `{ |x| : x in self }`.
    ///
    /// The upper endpoint is the larger magnitude of the two endpoints. The
    /// lower endpoint is exactly zero when the interval straddles or touches
    /// zero (zero is then attained), and otherwise the smaller magnitude. Exact:
    /// `abs` only flips signs and selects.
    #[must_use]
    pub fn abs(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        let hi = fabs(a).rmax(fabs(b));
        let lo = if a <= zero && zero <= b {
            zero
        } else {
            fabs(a).rmin(fabs(b))
        };
        Self::hull(lo, hi)
    }

    /// The pointwise minimum, the set `{ min(x, y) : x in self, y in rhs }`.
    ///
    /// Componentwise on the endpoints: `[min(a, c), min(b, d)]`. Exact: the
    /// endpoints are selected, never computed. Empty if either operand is empty.
    #[must_use]
    pub fn min(self, rhs: Self) -> Self {
        let (Some((a, b)), Some((c, d))) = (self.bounds(), rhs.bounds()) else {
            return Self::empty();
        };
        Self::hull(a.rmin(c), b.rmin(d))
    }

    /// The pointwise maximum, the set `{ max(x, y) : x in self, y in rhs }`.
    ///
    /// Componentwise on the endpoints: `[max(a, c), max(b, d)]`. Exact: the
    /// endpoints are selected, never computed. Empty if either operand is empty.
    #[must_use]
    pub fn max(self, rhs: Self) -> Self {
        let (Some((a, b)), Some((c, d))) = (self.bounds(), rhs.bounds()) else {
            return Self::empty();
        };
        Self::hull(a.rmax(c), b.rmax(d))
    }

    /// The sign, the set `{ sign(x) : x in self }`, a subset of `{-1, 0, 1}`.
    ///
    /// `sign` is non-decreasing, so the image is `[sign(a), sign(b)]`, the
    /// convex hull of the attained signs. Every value is one of the three exact
    /// constants. A zero endpoint of either sign contributes `0`.
    #[must_use]
    pub fn sign(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(fsign(a), fsign(b))
    }
}

impl<F: RoundFloat + RoundInteger> Interval<F> {
    /// The ceiling, the set `{ ceil(x) : x in self }` (round toward plus
    /// infinity). Exact: `ceil` is a non-decreasing step function, so the image
    /// is `[ceil(a), ceil(b)]` with no rounding.
    #[must_use]
    pub fn ceil(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(a.ceil(), b.ceil())
    }

    /// The floor, the set `{ floor(x) : x in self }` (round toward minus
    /// infinity). Exact: `floor` is a non-decreasing step function, so the image
    /// is `[floor(a), floor(b)]` with no rounding.
    #[must_use]
    pub fn floor(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(a.floor(), b.floor())
    }

    /// The truncation, the set `{ trunc(x) : x in self }` (round toward zero).
    /// Exact: `trunc` is a non-decreasing step function, so the image is
    /// `[trunc(a), trunc(b)]` with no rounding.
    #[must_use]
    pub fn trunc(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(a.trunc(), b.trunc())
    }

    /// Rounding to the nearest integer with ties to even, the set
    /// `{ roundTiesToEven(x) : x in self }`. Exact: a non-decreasing step
    /// function, so the image is `[roundTiesToEven(a), roundTiesToEven(b)]`.
    #[must_use]
    pub fn round_ties_to_even(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(a.round_ties_to_even(), b.round_ties_to_even())
    }

    /// Rounding to the nearest integer with ties away from zero, the set
    /// `{ roundTiesToAway(x) : x in self }`. Exact: a non-decreasing step
    /// function, so the image is `[roundTiesToAway(a), roundTiesToAway(b)]`.
    #[must_use]
    pub fn round_ties_to_away(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        Self::hull(a.round_ties_to_away(), b.round_ties_to_away())
    }
}
