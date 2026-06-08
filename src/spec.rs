//! The written-down specification: the laws every `interval-1788` value and
//! operation must uphold.
//!
//! This module is prose, not runtime code. The laws are the obligations the
//! implementation and its verification lanes discharge, kept in the tree so a
//! future maintainer reconstructs intent without apprenticing to anyone. The
//! discipline mirrors pfloat-ball's `spec` module. References are IEEE Std
//! 1788-2015 and the open "Introduction to the IEEE 1788-2015 Standard for
//! Interval Arithmetic" (Revol).
//!
//! # What an interval denotes
//!
//! An [`Interval`](crate::Interval) is either the empty set or a closed
//! connected set of reals `[lo, hi]` with `lo <= hi`. The endpoints range over
//! the extended reals, so `lo` may be minus infinity and `hi` plus infinity;
//! `Entire = [-inf, +inf]` is the whole real line. Plus infinity is never a
//! lower endpoint and minus infinity is never an upper endpoint, because neither
//! bounds a nonempty set of reals.
//!
//! # Law 1: the construction invariant
//!
//! Every value reachable through the public surface is in canonical form: no
//! endpoint is NaN, no lower endpoint is plus infinity, no upper endpoint is
//! minus infinity, `lo <= hi`, and the empty set has one representation. The
//! representation is private and there are no setters, so the invariant cannot
//! be violated from outside. Every operation re-establishes it by routing its
//! result through a checked constructor rather than assuming it.
//!
//! # Law 2: enclosure soundness (the Fundamental Theorem of Interval Arithmetic)
//!
//! For an operation implementing the real function `f`, the result encloses `f`
//! applied to every point of the inputs:
//!
//! ```text
//!     for all x in [a],            f(x)    in op([a])         (unary)
//!     for all x in [a], y in [b],  f(x, y) in op([a], [b])    (binary)
//! ```
//!
//! This is the one property that makes a result rigorous: it is never a claim
//! narrower than the truth. Soundness is one-directional. A wider-than-necessary
//! enclosure is still correct; an enclosure that omits a reachable value is the
//! single defect that turns every downstream result into a falsehood. The
//! verification lanes weight soundness above tightness.
//!
//! # Law 3: outward rounding
//!
//! Arithmetic computes each endpoint with the directed rounding that widens the
//! result: the lower endpoint toward minus infinity
//! ([`RoundFloat::add_down`](crate::RoundFloat::add_down) and kin), the upper
//! endpoint toward plus infinity. With a correctly-rounded backend the chosen
//! endpoint is the nearest representable bound, so the enclosure is tight; no
//! defensive widening is added on top.
//!
//! # Law 4: the four-corner rule for non-monotone operations
//!
//! Multiplication and division are not monotone across a sign change, so the
//! extreme of `op([a], [b])` need not occur at a fixed pair of endpoints. The
//! lower endpoint is the minimum over all four endpoint combinations rounded
//! down, the upper endpoint the maximum over all four rounded up. Picking a
//! single corner by guessing the sign pattern is the classic rigor bug; the
//! four-corner reduction is correct by construction. (Multiplication's
//! indeterminate `0 * inf` combination is resolved to `0` by the interval
//! convention, handled where the operation is defined.)
//!
//! # Law 5: total set-based results
//!
//! In the set-based flavor every operation is total. A division whose divisor
//! contains zero, or a square root of a partly-negative interval, does not
//! error: it returns the smallest interval (possibly unbounded, possibly the
//! restriction to the function's domain, possibly empty) that encloses the
//! result over the inputs. Construction failures (Law 1) are the only errors. A
//! consumer that wants a bounded-only view maps unbounded or empty results to
//! its own error at its boundary, outside this crate.
//!
//! # Law 6: the fixture is sound, not tight
//!
//! The `f64` instance of [`RoundFloat`](crate::RoundFloat) behind the `fixture`
//! feature rounds each result outward by one step unconditionally. It always
//! satisfies Laws 2 through 4, but it may return an endpoint up to one unit in
//! the last place looser than the tightest representable bound, even when the
//! exact result was representable. It exists to make the enclosure laws
//! machine-checkable and to drive host property tests, not to be a production
//! interval float. Tightness (Law 3's "nearest representable bound") is verified
//! only against a correctly-rounded backend, never against the fixture.
