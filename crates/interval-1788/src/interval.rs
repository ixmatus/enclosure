//! [`Interval`]: the inf-sup interval type and its construction invariant.
//!
//! `Interval<F>` denotes a closed, connected subset of the extended reals: the
//! empty set, or a nonempty `[lo, hi]` with `lo <= hi`, where `lo` may be minus
//! infinity and `hi` may be plus infinity. The invariant (no NaN endpoint, no
//! `+inf` lower or `-inf` upper, `lo <= hi`, and a single canonical empty form)
//! is held by every constructor and cannot be violated through the public
//! surface: the representation is private and there are no endpoint setters.
//!
//! Arithmetic operations arrive in a later phase. This module is the type and
//! its constructors and observers.

use crate::error::IntervalError;
use round_float::RoundFloat;

/// The private representation. Keeping the variants and fields unreachable from
/// outside the crate is what makes an invalid interval unrepresentable: the only
/// way to obtain a `Repr` is through a checked constructor.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Repr<F> {
    /// The empty set. Its single canonical form, so equality is structural.
    Empty,
    /// A nonempty interval `[lo, hi]` with the module invariant upheld.
    NonEmpty { lo: F, hi: F },
}

/// A rigorous interval: a guaranteed enclosure of a real quantity.
///
/// Construct with [`Interval::new`], [`Interval::point`], [`Interval::empty`], or
/// [`Interval::entire`]. Observe with [`inf`](Interval::inf),
/// [`sup`](Interval::sup), and the `is_*` predicates. Two intervals are equal
/// when they denote the same set; because every value is in canonical form,
/// that is structural equality of the endpoints.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Interval<F> {
    repr: Repr<F>,
}

impl<F: RoundFloat> Interval<F> {
    /// The empty interval, denoting no real values.
    #[must_use]
    pub fn empty() -> Self {
        Self { repr: Repr::Empty }
    }

    /// The whole extended real line, `[-inf, +inf]`. The least informative
    /// nonempty enclosure: it contains every real, so it is always sound.
    #[must_use]
    pub fn entire() -> Self {
        Self {
            repr: Repr::NonEmpty {
                lo: F::NEG_INFINITY,
                hi: F::INFINITY,
            },
        }
    }

    /// The interval `[lo, hi]`.
    ///
    /// # Errors
    ///
    /// Returns [`IntervalError::NaNEndpoint`] if either endpoint is NaN,
    /// [`IntervalError::InvalidEndpoint`] if `lo` is `+inf` or `hi` is `-inf`
    /// (neither can bound a nonempty set), and [`IntervalError::Inverted`] if
    /// `lo > hi`. To denote no values, use [`empty`](Interval::empty).
    pub fn new(lo: F, hi: F) -> Result<Self, IntervalError> {
        if lo.is_nan() || hi.is_nan() {
            return Err(IntervalError::NaNEndpoint);
        }
        if (lo.is_infinite() && !lo.is_sign_negative())
            || (hi.is_infinite() && hi.is_sign_negative())
        {
            return Err(IntervalError::InvalidEndpoint);
        }
        if lo > hi {
            return Err(IntervalError::Inverted);
        }
        Ok(Self {
            repr: Repr::NonEmpty { lo, hi },
        })
    }

    /// The singleton interval `[x, x]` for a finite `x`.
    ///
    /// This is the bridge from a point value into the interval world: the
    /// enclosure that contains exactly `x`. The merge-gate property "an interval
    /// operation encloses the point operation" is stated against singletons.
    ///
    /// # Errors
    ///
    /// Returns [`IntervalError::NaNEndpoint`] if `x` is NaN and
    /// [`IntervalError::InvalidEndpoint`] if `x` is infinite.
    pub fn point(x: F) -> Result<Self, IntervalError> {
        if x.is_nan() {
            return Err(IntervalError::NaNEndpoint);
        }
        if !x.is_finite() {
            return Err(IntervalError::InvalidEndpoint);
        }
        Ok(Self {
            repr: Repr::NonEmpty { lo: x, hi: x },
        })
    }

    /// Whether this is the empty interval.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self.repr, Repr::Empty)
    }

    /// Whether this is `Entire`, the whole extended real line.
    #[must_use]
    pub fn is_entire(&self) -> bool {
        matches!(
            self.repr,
            Repr::NonEmpty { lo, hi } if lo.is_infinite() && lo.is_sign_negative()
                && hi.is_infinite() && !hi.is_sign_negative()
        )
    }

    /// Whether this is a singleton `[x, x]` with `x` finite.
    #[must_use]
    pub fn is_singleton(&self) -> bool {
        matches!(self.repr, Repr::NonEmpty { lo, hi } if lo == hi && lo.is_finite())
    }

    /// Whether this interval is bounded (both endpoints finite). The empty
    /// interval is bounded.
    #[must_use]
    pub fn is_bounded(&self) -> bool {
        match self.repr {
            Repr::Empty => true,
            Repr::NonEmpty { lo, hi } => lo.is_finite() && hi.is_finite(),
        }
    }

    /// The infimum (lower endpoint), as the standard's Level 2 numeric datum.
    ///
    /// By the standard's convention the infimum of the empty interval is plus
    /// infinity, so the accessor is total. A zero infimum reads out as negative
    /// zero: IEEE 1788's Level 2 `inf` pins the sign of a zero result to `-0`
    /// whatever sign the stored endpoint carries, so `inf([0, +inf])` is `-0`. This
    /// is the numeric function; the raw stored endpoint, which the text
    /// serialization reproduces for round-trip fidelity, is read from the interval
    /// representation directly, not through this accessor.
    #[must_use]
    pub fn inf(&self) -> F {
        match self.repr {
            Repr::Empty => F::INFINITY,
            // A zero infimum reads out as `-0` (the Level 2 numeric datum); every
            // other endpoint reads out unchanged.
            Repr::NonEmpty { lo, .. } if lo.is_zero() => F::ZERO.negate(),
            Repr::NonEmpty { lo, .. } => lo,
        }
    }

    /// The supremum (upper endpoint), as the standard's Level 2 numeric datum.
    ///
    /// By the standard's convention the supremum of the empty interval is minus
    /// infinity, so the accessor is total. A zero supremum reads out as positive
    /// zero: IEEE 1788's Level 2 `sup` pins the sign of a zero result to `+0`, the
    /// mirror of the [`inf`](Interval::inf) rule. This is the numeric function,
    /// distinct from the raw stored endpoint the text serialization reproduces.
    #[must_use]
    pub fn sup(&self) -> F {
        match self.repr {
            Repr::Empty => F::NEG_INFINITY,
            // A zero supremum reads out as `+0` (the Level 2 numeric datum).
            Repr::NonEmpty { hi, .. } if hi.is_zero() => F::ZERO,
            Repr::NonEmpty { hi, .. } => hi,
        }
    }

    /// Whether the real value `x` lies in this interval.
    #[must_use]
    pub fn contains(&self, x: F) -> bool {
        match self.repr {
            Repr::Empty => false,
            Repr::NonEmpty { lo, hi } => lo <= x && x <= hi,
        }
    }

    /// Whether this is a common interval: nonempty and bounded (the recommended
    /// `isCommonInterval`).
    ///
    /// `Entire` and the half-unbounded intervals are not common (they are
    /// unbounded), and the empty interval is not common (it is empty). Every other
    /// interval is a closed bounded nonempty `[a, b]` and is common.
    #[must_use]
    pub fn is_common_interval(&self) -> bool {
        !self.is_empty() && self.is_bounded()
    }

    /// Whether the finite real number `m` is a member of this interval (the
    /// recommended `isMember`).
    ///
    /// Membership requires `m` to be a finite real: no infinity is a member of any
    /// interval, not even one whose endpoint is that infinity. So this is stricter
    /// than [`contains`](Interval::contains), which admits an infinite argument at
    /// a matching infinite endpoint: `entire.contains(+inf)` is `true` but
    /// `entire.is_member(+inf)` is `false`. A NaN argument is never a member. The
    /// finiteness guard is load bearing precisely because `contains` alone would
    /// answer the infinite-endpoint cases the wrong way.
    #[must_use]
    pub fn is_member(&self, m: F) -> bool {
        m.is_finite() && self.contains(m)
    }

    /// The endpoints, or `None` for the empty interval. Crate-internal so the
    /// operation modules can pattern-match without exposing the representation.
    pub(crate) fn bounds(self) -> Option<(F, F)> {
        match self.repr {
            Repr::Empty => None,
            Repr::NonEmpty { lo, hi } => Some((lo, hi)),
        }
    }

    /// Build a nonempty interval from computed operation endpoints.
    ///
    /// Operations establish the invariant by their endpoint arithmetic (outward
    /// rounding only widens, and `min <= max`), so this is total. If the
    /// invariant is ever violated, that is a bug in an operation; in debug builds
    /// it trips an assertion, and in release it degrades to `Entire`, which is
    /// always a sound enclosure. The crate never returns an unsound result and
    /// never panics on a caller's input.
    pub(crate) fn hull(lo: F, hi: F) -> Self {
        Self::new(lo, hi).unwrap_or_else(|_| {
            debug_assert!(
                false,
                "operation produced endpoints violating the interval invariant"
            );
            Self::entire()
        })
    }
}

#[cfg(all(test, feature = "fixture"))]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_nan_endpoints() {
        assert_eq!(
            Interval::<f64>::new(f64::NAN, 1.0),
            Err(IntervalError::NaNEndpoint)
        );
        assert_eq!(
            Interval::<f64>::new(0.0, f64::NAN),
            Err(IntervalError::NaNEndpoint)
        );
    }

    #[test]
    fn new_rejects_inverted() {
        assert_eq!(Interval::<f64>::new(2.0, 1.0), Err(IntervalError::Inverted));
    }

    #[test]
    fn new_rejects_unbounding_infinities() {
        // +inf cannot be a lower endpoint; -inf cannot be an upper endpoint.
        assert_eq!(
            Interval::<f64>::new(f64::INFINITY, f64::INFINITY),
            Err(IntervalError::InvalidEndpoint)
        );
        assert_eq!(
            Interval::<f64>::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
            Err(IntervalError::InvalidEndpoint)
        );
    }

    #[test]
    fn new_admits_half_bounded() {
        assert!(Interval::<f64>::new(f64::NEG_INFINITY, 0.0).is_ok());
        assert!(Interval::<f64>::new(0.0, f64::INFINITY).is_ok());
    }

    #[test]
    fn point_must_be_finite() {
        assert_eq!(
            Interval::<f64>::point(f64::INFINITY),
            Err(IntervalError::InvalidEndpoint)
        );
        assert!(Interval::<f64>::point(3.5).unwrap().is_singleton());
    }

    #[test]
    fn empty_and_entire_classify() {
        let e = Interval::<f64>::empty();
        assert!(e.is_empty());
        assert!(e.is_bounded());
        assert_eq!(e.inf(), f64::INFINITY);
        assert_eq!(e.sup(), f64::NEG_INFINITY);
        assert!(!e.contains(0.0));

        let all = Interval::<f64>::entire();
        assert!(all.is_entire());
        assert!(!all.is_bounded());
        assert!(all.contains(0.0));
        assert!(all.contains(f64::MAX));
    }

    #[test]
    fn contains_respects_endpoints() {
        let i = Interval::<f64>::new(-1.0, 2.0).unwrap();
        assert!(i.contains(-1.0));
        assert!(i.contains(2.0));
        assert!(i.contains(0.5));
        assert!(!i.contains(2.5));
        assert!(!i.contains(-1.5));
    }

    #[test]
    fn equality_is_structural() {
        assert_eq!(Interval::<f64>::empty(), Interval::<f64>::empty());
        assert_eq!(
            Interval::<f64>::new(1.0, 2.0).unwrap(),
            Interval::<f64>::new(1.0, 2.0).unwrap()
        );
        assert_ne!(
            Interval::<f64>::new(1.0, 2.0).unwrap(),
            Interval::<f64>::new(1.0, 3.0).unwrap()
        );
        assert_ne!(
            Interval::<f64>::empty(),
            Interval::<f64>::new(1.0, 2.0).unwrap()
        );
    }
}
