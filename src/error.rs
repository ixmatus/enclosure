//! [`IntervalError`]: the construction failures the type system cannot rule out.
//!
//! These are the ways a caller can ask for an interval that does not denote a
//! valid set. They are distinct from the set-based flavor's exceptional
//! *results* (a division whose divisor contains zero, the square root of a
//! partly-negative interval): those are total operations that return an
//! unbounded interval or the empty set, not errors, and a consumer that wants a
//! bounded-only view maps them to its own error at its boundary.

use core::fmt;

/// A request to construct an interval that does not denote a valid real set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IntervalError {
    /// An endpoint was NaN. NaN denotes no real value, so it cannot bound a set.
    NaNEndpoint,
    /// An endpoint could not bound a nonempty interval: positive infinity given
    /// as the lower endpoint, negative infinity as the upper, or a non-finite
    /// value where a finite one is required (such as a singleton's point).
    InvalidEndpoint,
    /// The lower endpoint exceeded the upper endpoint (`lo > hi`). To denote no
    /// values, use [`Interval::empty`](crate::Interval::empty) explicitly.
    Inverted,
}

impl fmt::Display for IntervalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            IntervalError::NaNEndpoint => "interval endpoint is NaN",
            IntervalError::InvalidEndpoint => "interval endpoint cannot bound a nonempty set",
            IntervalError::Inverted => "interval lower endpoint exceeds upper endpoint",
        };
        f.write_str(msg)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IntervalError {}
