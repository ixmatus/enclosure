//! Cancellative subtraction and addition: recovering an interval after a known
//! addition, the inverse of adding a fixed operand.
//!
//! [`cancel_minus`](Interval::cancel_minus) returns the tightest interval `z`
//! such that `y + z` encloses `x`: the value you must have added `y` to in order
//! to reach an enclosure of `x`, recovered after the fact.
//! [`cancel_plus`](Interval::cancel_plus) is its sibling, `cancel_minus(x, -y)`.
//! Neither is a fundamental-theorem operation and neither makes a continuity
//! promise, so their decorated forms grade `trv` throughout (see
//! [`decorated`](crate::decorated)).
//!
//! # Level 1 definition (set-based flavor, P1788.1 clause 4.5.3)
//!
//! With `x = [a, b]` and `y = [c, d]`, `cancel_minus(x, y)` is:
//!
//! - the empty set when `x` is empty and `y` is bounded (an empty target needs
//!   nothing added back);
//! - `[a - c, b - d]` when both operands are bounded and nonempty and
//!   `wid(y) <= wid(x)` (the width condition that leaves those endpoints ordered);
//! - Entire in every remaining case: either operand unbounded, `x` nonempty with
//!   `y` empty, `x` empty with `y` unbounded, or both bounded and nonempty with
//!   `wid(y) > wid(x)`.
//!
//! The vendored ITF1788 vectors (`libieeep1788_cancel.itl`) pin each of these. In
//! particular an empty `x` with an unbounded `y` yields Entire, not empty, so the
//! unbounded rule wins over the empty rule; the vectors confirm it
//! (`cancelMinus [empty] [-infinity, -1.0] = [entire]`).

use crate::interval::Interval;
use round_float::RoundFloat;

impl<F: RoundFloat> Interval<F> {
    /// Cancellative subtraction: the tightest `z` with `y + z` enclosing `self`.
    ///
    /// See the [module docs](crate::cancel) for the Level 1 definition and the
    /// cases that yield Entire. Total and sound on every input: any input outside
    /// the width condition (or empty or unbounded per the rules) yields Entire,
    /// which encloses every `z`, so the method never returns an unsound or
    /// inverted interval and never panics.
    // The endpoint quartet `a, b, c, d` plus `y` is the standard symbolic
    // vocabulary for this arithmetic; spelling them out would obscure the math.
    #[allow(clippy::many_single_char_names)]
    pub fn cancel_minus(self, y: Self) -> Self {
        let (Some((a, b)), Some((c, d))) = (self.bounds(), y.bounds()) else {
            // At least one operand is empty.
            //   x empty,   y bounded   -> empty  (an empty target needs nothing)
            //   x empty,   y unbounded -> Entire (the unbounded rule wins)
            //   x nonempty, y empty    -> Entire (nothing to cancel)
            // The empty/unbounded split is pinned by the vectors: an empty x
            // against an unbounded y is Entire, never empty.
            return if self.is_empty() && y.is_bounded() {
                Self::empty()
            } else {
                Self::entire()
            };
        };
        // Both operands are nonempty. An unbounded operand is a no-value case.
        if !(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()) {
            return Self::entire();
        }

        // Derivation 1: the width gate, and why the addition rearrangement.
        //
        // The tightest z has ordered endpoints (is a valid interval) exactly when
        // wid(y) <= wid(x), that is d - c <= b - a, equivalently a + d <= b + c.
        // We certify that inequality rather than compare rounded widths. Forming a
        // width directly, as `b.sub(a)`, throws away the low bits of a value like
        // `1 + 2^-53`, and a `wid_up(y) <= wid_down(x)` gate built from such widths
        // then rejects genuinely ordered borderline inputs (the near-1 vectors pin
        // this on a correctly-rounded backend). The rearranged form keeps the
        // comparison as a one-sided certification: `add_up(a, d) <= add_down(b, c)`
        // upper-bounds the left side and lower-bounds the right, so when it holds
        // then a + d <= b + c holds exactly, hence wid(y) <= wid(x). The endpoints
        // are non-NaN, so the reject test below is the exact negation.
        //
        // Soundness direction: the gate is conservative toward rejection. A truly
        // equal-width input that rounding pushes into the reject branch returns
        // Entire, which is wider than the tightest z but never wrong. The opposite
        // slip, letting a too-wide y through, would build a z the standard defines
        // as Entire, so rejection is the sound way to be wrong. On a
        // correctly-rounded backend the two sums land on representable values for
        // the vectors, so the gate is exact (tight); on the deliberately
        // sound-not-tight f64 fixture the directed sums straddle an equal-width
        // boundary and the gate rejects it into Entire, sound but not tight.
        if a.add_up(d) > b.add_down(c) {
            return Self::entire();
        }

        // Derivation 2: the endpoint rounding directions, from the defining
        // property rather than the formula.
        //
        // The returned z = [lo, hi] must satisfy y + z superset x, that is
        // c + lo <= a (so x's lower endpoint a stays covered) and d + hi >= b (so
        // b stays covered). The first forces lo <= a - c, met by rounding the
        // difference DOWN: lo = a.sub_down(c). The second forces hi >= b - d, met
        // by rounding UP: hi = b.sub_up(d). Outward rounding only widens z, and a
        // wider z still encloses x under +y, so the enclosure stays sound; a
        // correctly-rounded backend makes it tight.
        let lo = a.sub_down(c);
        let hi = b.sub_up(d);

        // The gate certified a + d <= b + c, hence a - c <= b - d exactly, and the
        // outward rounding pushes lo no higher and hi no lower, so lo <= hi holds
        // here. The comparison is a defensive floor: were a backend to round a
        // corner the wrong way and invert the endpoints, Entire is the sound
        // answer, never a panic or an invalid interval.
        if lo <= hi {
            Self::hull(lo, hi)
        } else {
            Self::entire()
        }
    }

    /// Cancellative addition, `cancel_minus(self, -y)`: the tightest `z` such that
    /// `(-y) + z` (equivalently `z - y`) encloses `self`.
    ///
    /// The sibling of [`cancel_minus`](Interval::cancel_minus); negating `y` is
    /// exact, so it inherits that method's soundness and totality unchanged.
    pub fn cancel_plus(self, y: Self) -> Self {
        self.cancel_minus(-y)
    }
}
