//! Numeric, boolean, and set functions of intervals (the non-arithmetic part of
//! the set-based flavor).
//!
//! The numeric functions (`wid`, `mag`, `mig`, `mid`, `rad`, `mid_rad`) return
//! `Option<F>` (or `Option<(F, F)>`): `None` for the empty interval, where the
//! standard's Level 2 returns NaN. Returning `Option` rather than a NaN sentinel
//! keeps the empty case explicit in the type; a NaN-returning shim for strict
//! Level 2 datum conformance can sit at that boundary later. The boolean
//! relations (`equal`, `subset`, `interior`, `disjoint`, the two `less`
//! orderings, the two `precedes` orderings) and the set operations follow the
//! standard's set semantics, with the empty set handled as the identity it is
//! (empty is a subset of everything, disjoint from everything, and the identity
//! of convex hull). The orderings are exact endpoint comparisons: they select and
//! compare, never round. [`overlap`](Interval::overlap) refines "how do these two
//! intervals sit relative to each other" into the standard's sixteen mutually
//! exclusive [`Overlap`] states, again by pure endpoint case analysis.
//!
//! `mid` and `rad` are the one place a directed-rounding surface cannot always
//! reproduce the standard's Level 2 datum (the nearest float to the true
//! midpoint). The implementation is the best sound approximation; the divergence
//! is documented at [`mid_rad`](Interval::mid_rad).

use crate::interval::Interval;
use round_float::RoundFloat;

/// Absolute value of an endpoint. Exact (a sign flip or nothing); maps an
/// infinite endpoint to positive infinity. Endpoints are never NaN.
#[inline]
fn fabs<F: RoundFloat>(x: F) -> F {
    if x.is_sign_negative() {
        x.negate()
    } else {
        x
    }
}

/// Whether an endpoint is negative infinity (an interval unbounded below).
#[inline]
fn is_neg_inf<F: RoundFloat>(x: F) -> bool {
    x.is_infinite() && x.is_sign_negative()
}

/// Whether an endpoint is positive infinity (an interval unbounded above).
#[inline]
fn is_pos_inf<F: RoundFloat>(x: F) -> bool {
    x.is_infinite() && !x.is_sign_negative()
}

/// The three-way comparison of two endpoints. Endpoints are never NaN, so the
/// partial order is total here and exactly one arm holds. `-0` and `+0` compare
/// equal, and `-inf`/`+inf` compare with themselves as equal, which is what the
/// `overlap` state analysis wants.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EndCmp {
    Lt,
    Eq,
    Gt,
}

#[inline]
fn endcmp<F: RoundFloat>(x: F, y: F) -> EndCmp {
    if x < y {
        EndCmp::Lt
    } else if y < x {
        EndCmp::Gt
    } else {
        EndCmp::Eq
    }
}

/// How two intervals sit relative to each other on the line: the sixteen mutually
/// exclusive, jointly exhaustive states of the IEEE 1788 `overlap` relation
/// (Allen's interval algebra for closed intervals, plus the three empty cases).
///
/// [`Interval::overlap`] returns exactly one of these for any ordered pair. The
/// converse pairs mirror across swapping the operands: `before`/`after`,
/// `meets`/`metBy`, `overlaps`/`overlappedBy`, `starts`/`startedBy`,
/// `finishes`/`finishedBy`, `contains`/`containedBy`, with `equals` self-mirror.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Overlap {
    /// Both operands are empty.
    BothEmpty,
    /// The first operand is empty and the second is nonempty.
    FirstEmpty,
    /// The first operand is nonempty and the second is empty.
    SecondEmpty,
    /// `self` lies entirely left of `other` with a gap: `sup(self) < inf(other)`.
    Before,
    /// `self`'s upper endpoint touches `other`'s lower endpoint and nothing more:
    /// `sup(self) == inf(other)`, both operands nondegenerate at that point.
    Meets,
    /// `self` and `other` overlap in their interiors from the left:
    /// `inf(self) < inf(other) < sup(self) < sup(other)`.
    Overlaps,
    /// `self` and `other` share a lower endpoint and `self` ends first:
    /// `inf(self) == inf(other)` and `sup(self) < sup(other)`.
    Starts,
    /// `self` lies strictly inside `other`:
    /// `inf(other) < inf(self)` and `sup(self) < sup(other)`.
    ContainedBy,
    /// `self` and `other` share an upper endpoint and `self` starts later:
    /// `inf(other) < inf(self)` and `sup(self) == sup(other)`.
    Finishes,
    /// The two intervals are the same nonempty set (equal endpoints).
    Equals,
    /// The mirror of `finishes`: shared upper endpoint, `other` starts later.
    FinishedBy,
    /// The mirror of `containedBy`: `other` lies strictly inside `self`.
    Contains,
    /// The mirror of `starts`: shared lower endpoint, `other` ends first.
    StartedBy,
    /// The mirror of `overlaps`: interiors overlap from the right.
    OverlappedBy,
    /// The mirror of `meets`: `inf(self)` touches `sup(other)`.
    MetBy,
    /// The mirror of `before`: `self` lies entirely right of `other` with a gap.
    After,
}

impl<F: RoundFloat> Interval<F> {
    /// The width `sup - inf`, rounded up so it is never understated. `None` for
    /// the empty interval; positive infinity for an unbounded interval.
    #[must_use]
    pub fn wid(self) -> Option<F> {
        self.bounds().map(|(lo, hi)| hi.sub_up(lo))
    }

    /// The magnitude `max { |t| : t in self }` (the largest absolute value).
    /// `None` for the empty interval; positive infinity for an unbounded
    /// interval.
    #[must_use]
    pub fn mag(self) -> Option<F> {
        self.bounds().map(|(lo, hi)| fabs(lo).rmax(fabs(hi)))
    }

    /// The mignitude `min { |t| : t in self }` (the smallest absolute value).
    /// Zero when the interval contains zero. `None` for the empty interval.
    #[must_use]
    pub fn mig(self) -> Option<F> {
        self.bounds().map(|(lo, hi)| {
            if lo <= F::ZERO && F::ZERO <= hi {
                F::ZERO
            } else {
                fabs(lo).rmin(fabs(hi))
            }
        })
    }

    /// A representable midpoint of the interval, paired with a radius that closes
    /// the enclosure: `[mid - rad, mid + rad]` contains every point of `self`.
    /// `None` for the empty interval (the standard's Level 2 returns NaN there).
    ///
    /// # The defining property, and why it holds structurally
    ///
    /// The one obligation on any sound backend is the enclosure
    ///
    /// ```text
    ///     mid.sub_down(rad) <= inf(self)   and   mid.add_up(rad) >= sup(self)
    /// ```
    ///
    /// It holds no matter what value the midpoint estimate lands on, because the
    /// radius is computed as the larger of the two one-sided distances *to the
    /// actually emitted `mid`*, each rounded up: `rad >= mid - inf` and
    /// `rad >= sup - mid` exactly, so `mid - rad <= inf` and `mid + rad >= sup`.
    /// Outward rounding of the endpoints only widens the enclosure. This is the
    /// same shape `affine-arith`'s `from_interval` uses to pick a sound center.
    ///
    /// # Overflow-safe midpoint
    ///
    /// The naive `(inf + sup) / 2` overflows to an infinity when both endpoints
    /// are near the finite range (`inf = sup = realmax` sums to `+inf`). The
    /// halved form `inf/2 + sup/2` cannot overflow: each half is bounded by the
    /// larger endpoint magnitude, so their sum is too. The cost is a subnormal
    /// caveat: halving a subnormal endpoint rounds, so for tiny intervals the
    /// midpoint is a hair less accurate than `(inf + sup) / 2` would be. The
    /// radius absorbs it; the enclosure stays sound.
    ///
    /// # Accuracy divergence from Level 2 (documented, not a bug)
    ///
    /// The standard's Level 2 `mid` demands the nearest float to the true
    /// midpoint. The [`RoundFloat`] surface exposes only directed rounding (no
    /// round-to-nearest primitive) and no largest-finite constant, so this
    /// implementation cannot always emit that datum:
    ///
    /// - The **exactly emitted** cases, correct on every sound backend: the empty
    ///   interval (`None`), a singleton `[x, x]` (`mid = x`, `rad = 0`), an
    ///   interval symmetric about zero (`mid = 0`), and `Entire` (`mid = 0`).
    /// - **Half-unbounded** `[a, +inf]` or `[-inf, b]`: Level 2 pins `realmax` /
    ///   `-realmax`, but no largest-finite constant is reachable through
    ///   `RoundFloat`, so `mid` returns the finite endpoint, a sound value inside
    ///   the interval. `rad` is `+inf` in these cases, so `[mid - rad, mid + rad]`
    ///   is `Entire` and the enclosure holds trivially; only the reported
    ///   midpoint datum diverges.
    /// - **General bounded** intervals: `mid` is the halved-form estimate above.
    ///   On a correctly-rounded backend a representable exact midpoint comes out
    ///   exact; on the deliberately sound-not-tight `f64` fixture the directed
    ///   halving widens it by up to a unit in the last place. Tightest `mid` is a
    ///   correctly-rounded-backend property, to be certified by the conformance
    ///   lane over a tight backend, not by this fixture.
    #[must_use]
    pub fn mid_rad(self) -> Option<(F, F)> {
        let (a, b) = self.bounds()?;
        let a_finite = a.is_finite();
        let b_finite = b.is_finite();

        // Unbounded (nonempty): the Level 2 convention is a finite midpoint and an
        // infinite radius. `Entire` centers at zero exactly; a half-unbounded
        // interval centers at its finite endpoint (see the accuracy note: the
        // standard's realmax is unreachable through `RoundFloat`, and with an
        // infinite radius any finite in-interval midpoint is sound).
        if !a_finite || !b_finite {
            let mid = if !a_finite && !b_finite {
                F::ZERO
            } else if a_finite {
                a
            } else {
                b
            };
            return Some((mid, F::INFINITY));
        }

        // Bounded and nonempty. The two exact-midpoint cases first, then the
        // general halved estimate clamped into `[a, b]`.
        let mid = if a == b {
            // Singleton: the midpoint is the point itself, exactly.
            a
        } else if a.negate() == b {
            // Symmetric about zero: the exact midpoint is zero, representable.
            F::ZERO
        } else {
            // Overflow-safe halved midpoint, then clamp into `[a, b]` so the
            // reported midpoint is always a member of the interval regardless of
            // how directed rounding nudged the estimate. `two` follows the
            // affine crate's `from_interval` (`ONE + ONE`, rounded); it is exactly
            // 2 on a correctly-rounded backend and a hair above on the fixture.
            let two = F::ONE.add_up(F::ONE);
            let m = a.div_down(two).add_down(b.div_down(two));
            m.rmax(a).rmin(b)
        };

        // Radius: the larger one-sided distance to the emitted `mid`, rounded up,
        // floored at zero. A singleton reports exactly zero rather than the
        // one-ulp `sub_up(a, a)` the formula would give. This is what makes the
        // defining property hold structurally (see the method docs).
        let rad = if a == b {
            F::ZERO
        } else {
            mid.sub_up(a).rmax(b.sub_up(mid)).rmax(F::ZERO)
        };

        Some((mid, rad))
    }

    /// A representable midpoint of the interval. `None` for the empty interval.
    ///
    /// The midpoint paired with the radius from [`mid_rad`](Interval::mid_rad);
    /// see that method for the exact-versus-approximate cases and the Level 2
    /// accuracy divergence. Computed through `mid_rad` so the two always agree.
    #[must_use]
    pub fn mid(self) -> Option<F> {
        self.mid_rad().map(|(mid, _)| mid)
    }

    /// The radius: the distance from [`mid`](Interval::mid) that closes the
    /// enclosure. `None` for the empty interval; `+inf` for any unbounded
    /// interval. Computed through [`mid_rad`](Interval::mid_rad) so it matches the
    /// emitted midpoint exactly.
    #[must_use]
    pub fn rad(self) -> Option<F> {
        self.mid_rad().map(|(_, rad)| rad)
    }

    /// Whether `self` is a subset of `other`. The empty set is a subset of every
    /// interval; no nonempty interval is a subset of the empty set.
    #[must_use]
    pub fn subset(self, other: Self) -> bool {
        match (self.bounds(), other.bounds()) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some((al, ah)), Some((bl, bh))) => bl <= al && ah <= bh,
        }
    }

    /// Whether `self` lies in the interior of `other`: every point of `self` is
    /// strictly inside `other`, treating an infinite endpoint of `other` as open.
    /// The empty set is in the interior of every interval.
    #[must_use]
    pub fn interior(self, other: Self) -> bool {
        match (self.bounds(), other.bounds()) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some((al, ah)), Some((bl, bh))) => {
                let lower_ok = (bl.is_infinite() && bl.is_sign_negative()) || bl < al;
                let upper_ok = (bh.is_infinite() && !bh.is_sign_negative()) || ah < bh;
                lower_ok && upper_ok
            }
        }
    }

    /// Whether `self` and `other` share no point. The empty set is disjoint from
    /// every interval.
    #[must_use]
    pub fn disjoint(self, other: Self) -> bool {
        match (self.bounds(), other.bounds()) {
            (None, _) | (_, None) => true,
            (Some((al, ah)), Some((bl, bh))) => ah < bl || bh < al,
        }
    }

    /// Whether `self` and `other` denote the same set (the standard's `equal`).
    ///
    /// This coincides exactly with the derived [`PartialEq`] (`==`): every
    /// interval is in canonical form, so set equality is structural equality of
    /// the endpoints, and the underlying float equality already treats `-0` and
    /// `+0` as equal (so `[-0, 2]` equals `[0, 2]`) and both empty forms as one.
    /// The named method gives the standard relation a documented home beside
    /// `less` and `precedes`, and a bare relation for the decorated `equal` (with
    /// its `NaI` rule) to delegate to. `equal(empty, empty)` is true;
    /// `equal(empty, nonempty)` is false.
    #[must_use]
    pub fn equal(self, other: Self) -> bool {
        self == other
    }

    /// The weak ordering `less`: every point of `self` is bounded above by the
    /// matching point of `other`. For nonempty `[a, b]` and `[c, d]` this is
    /// `a <= c && b <= d`. Both empty is true; exactly one empty is false.
    ///
    /// The two-empty-is-true, one-empty-is-false convention is the `less` table
    /// the vectors pin; it is what the extended `inf`/`sup` conventions
    /// (`inf(empty) = +inf`, `sup(empty) = -inf`) give under `<=`, written out
    /// explicitly here so the empty cases are auditable.
    #[must_use]
    pub fn less(self, other: Self) -> bool {
        match (self.bounds(), other.bounds()) {
            (None, None) => true,
            (None, Some(_)) | (Some(_), None) => false,
            (Some((al, ah)), Some((bl, bh))) => al <= bl && ah <= bh,
        }
    }

    /// The strict ordering `strictLess`: like [`less`](Interval::less) but with
    /// strict endpoint inequalities, except that two matching infinite endpoints
    /// count as satisfying it (so `Entire` is `strictLess` than itself). For
    /// nonempty `[a, b]` and `[c, d]`: `(a < c or both are -inf)` and `(b < d or
    /// both are +inf)`. Both empty is true; exactly one empty is false.
    #[must_use]
    pub fn strict_less(self, other: Self) -> bool {
        match (self.bounds(), other.bounds()) {
            (None, None) => true,
            (None, Some(_)) | (Some(_), None) => false,
            (Some((al, ah)), Some((bl, bh))) => {
                let lower = al < bl || (is_neg_inf(al) && is_neg_inf(bl));
                let upper = ah < bh || (is_pos_inf(ah) && is_pos_inf(bh));
                lower && upper
            }
        }
    }

    /// The weak ordering `precedes`: `self` lies entirely at or below `other`,
    /// touching allowed. For nonempty operands this is `sup(self) <= inf(other)`.
    /// Any empty operand makes it true (the empty set precedes and is preceded by
    /// everything).
    #[must_use]
    pub fn precedes(self, other: Self) -> bool {
        match (self.bounds(), other.bounds()) {
            (None, _) | (_, None) => true,
            (Some((_, ah)), Some((bl, _))) => ah <= bl,
        }
    }

    /// The strict ordering `strictPrecedes`: `self` lies entirely below `other`
    /// with no shared point. For nonempty operands this is
    /// `sup(self) < inf(other)`. Any empty operand makes it true.
    #[must_use]
    pub fn strict_precedes(self, other: Self) -> bool {
        match (self.bounds(), other.bounds()) {
            (None, _) | (_, None) => true,
            (Some((_, ah)), Some((bl, _))) => ah < bl,
        }
    }

    /// How `self` sits relative to `other`: exactly one of the sixteen
    /// [`Overlap`] states.
    ///
    /// The three empty cases come first. For two nonempty intervals `[a, b]` and
    /// `[c, d]` the classification is pure endpoint case analysis. The two gap
    /// states (`before` when `b < c`, `after` when `a > d`) are decided first;
    /// the remaining thirteen touch-or-overlap states are then fixed by the pair
    /// of three-way comparisons `(cmp(a, c), cmp(b, d))`, with a single tie-break
    /// on `cmp(b, c)` separating `meets` from `overlaps` and on `cmp(a, d)`
    /// separating `metBy` from `overlappedBy`. The `match` over the nine
    /// comparison combinations is total, which is what guarantees the states are
    /// jointly exhaustive and mutually exclusive.
    #[must_use]
    pub fn overlap(self, other: Self) -> Overlap {
        let (Some((a, b)), Some((c, d))) = (self.bounds(), other.bounds()) else {
            return match (self.is_empty(), other.is_empty()) {
                (true, true) => Overlap::BothEmpty,
                (true, false) => Overlap::FirstEmpty,
                (false, true) => Overlap::SecondEmpty,
                // Both nonempty is unreachable here (the let-else matched an
                // empty operand); the arm keeps the match total.
                (false, false) => unreachable!(),
            };
        };

        if b < c {
            return Overlap::Before;
        }
        if a > d {
            return Overlap::After;
        }

        // Neither gap state: `c <= b` and `a <= d`. Classify by where `self`'s
        // endpoints fall against `other`'s.
        match (endcmp(a, c), endcmp(b, d)) {
            // `a < c`, `b < d`: touch at `b == c` (meets) or interiors cross
            // (overlaps); `b < c` was already returned as `before`.
            (EndCmp::Lt, EndCmp::Lt) => {
                if b == c {
                    Overlap::Meets
                } else {
                    Overlap::Overlaps
                }
            }
            (EndCmp::Lt, EndCmp::Eq) => Overlap::FinishedBy,
            (EndCmp::Lt, EndCmp::Gt) => Overlap::Contains,
            (EndCmp::Eq, EndCmp::Lt) => Overlap::Starts,
            (EndCmp::Eq, EndCmp::Eq) => Overlap::Equals,
            (EndCmp::Eq, EndCmp::Gt) => Overlap::StartedBy,
            (EndCmp::Gt, EndCmp::Lt) => Overlap::ContainedBy,
            (EndCmp::Gt, EndCmp::Eq) => Overlap::Finishes,
            // `a > c`, `b > d`: touch at `a == d` (metBy) or interiors cross
            // (overlappedBy); `a > d` was already returned as `after`.
            (EndCmp::Gt, EndCmp::Gt) => {
                if a == d {
                    Overlap::MetBy
                } else {
                    Overlap::OverlappedBy
                }
            }
        }
    }

    /// The set intersection, the largest interval contained in both. Empty if the
    /// two do not overlap (or either is empty). Endpoints are selected, not
    /// computed, so no rounding occurs.
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        match (self.bounds(), other.bounds()) {
            (Some((al, ah)), Some((bl, bh))) => {
                let lo = al.rmax(bl);
                let hi = ah.rmin(bh);
                if lo <= hi {
                    Self::hull(lo, hi)
                } else {
                    Self::empty()
                }
            }
            _ => Self::empty(),
        }
    }

    /// The convex hull, the smallest interval containing both. The empty set is
    /// the identity. Endpoints are selected, not computed, so no rounding occurs.
    #[must_use]
    pub fn convex_hull(self, other: Self) -> Self {
        match (self.bounds(), other.bounds()) {
            (Some((al, ah)), Some((bl, bh))) => Self::hull(al.rmin(bl), ah.rmax(bh)),
            (Some(_), None) => self,
            (None, _) => other,
        }
    }
}

#[cfg(all(test, feature = "fixture"))]
mod tests {
    use crate::Interval;

    fn iv(lo: f64, hi: f64) -> Interval<f64> {
        Interval::new(lo, hi).unwrap()
    }

    #[test]
    fn wid_mag_mig_on_a_bounded_interval() {
        let i = iv(-2.0, 5.0);
        assert!(i.wid().unwrap() >= 7.0); // 5 - (-2) = 7, rounded up
        assert_eq!(i.mag(), Some(5.0)); // max(|-2|, |5|)
        assert_eq!(i.mig(), Some(0.0)); // contains zero
        let p = iv(3.0, 4.0);
        assert_eq!(p.mig(), Some(3.0)); // min(|3|, |4|)
        assert_eq!(p.mag(), Some(4.0));
    }

    #[test]
    fn numeric_functions_of_empty_are_none() {
        let e = Interval::<f64>::empty();
        assert_eq!(e.wid(), None);
        assert_eq!(e.mag(), None);
        assert_eq!(e.mig(), None);
    }

    #[test]
    fn wid_of_unbounded_is_infinite() {
        assert_eq!(Interval::<f64>::entire().wid(), Some(f64::INFINITY));
        assert_eq!(iv(0.0, f64::INFINITY).wid(), Some(f64::INFINITY));
    }

    #[test]
    fn subset_basics() {
        assert!(iv(1.0, 2.0).subset(iv(0.0, 3.0)));
        assert!(!iv(0.0, 3.0).subset(iv(1.0, 2.0)));
        assert!(iv(1.0, 2.0).subset(iv(1.0, 2.0))); // reflexive
        assert!(Interval::<f64>::empty().subset(iv(1.0, 2.0))); // empty subset of all
        assert!(!iv(1.0, 2.0).subset(Interval::empty()));
        assert!(iv(1.0, 2.0).subset(Interval::entire()));
    }

    #[test]
    fn interior_is_strict() {
        assert!(iv(1.0, 2.0).interior(iv(0.0, 3.0)));
        assert!(!iv(1.0, 2.0).interior(iv(1.0, 3.0))); // shares the lower endpoint
        assert!(iv(1.0, 2.0).interior(Interval::entire())); // infinite endpoints are open
        assert!(Interval::<f64>::empty().interior(iv(1.0, 2.0)));
    }

    #[test]
    fn disjoint_basics() {
        assert!(iv(1.0, 2.0).disjoint(iv(3.0, 4.0)));
        assert!(!iv(1.0, 2.0).disjoint(iv(2.0, 4.0))); // touch at 2
        assert!(iv(1.0, 2.0).disjoint(Interval::empty()));
    }

    #[test]
    fn intersection_basics() {
        assert_eq!(iv(0.0, 3.0).intersection(iv(1.0, 5.0)), iv(1.0, 3.0));
        assert!(iv(0.0, 1.0).intersection(iv(2.0, 3.0)).is_empty()); // disjoint
        assert_eq!(iv(1.0, 2.0).intersection(iv(2.0, 3.0)), iv(2.0, 2.0)); // touch
        assert!(iv(1.0, 2.0).intersection(Interval::empty()).is_empty());
        assert_eq!(iv(1.0, 2.0).intersection(Interval::entire()), iv(1.0, 2.0));
    }

    #[test]
    fn convex_hull_basics() {
        assert_eq!(iv(0.0, 1.0).convex_hull(iv(2.0, 3.0)), iv(0.0, 3.0));
        assert_eq!(iv(1.0, 2.0).convex_hull(Interval::empty()), iv(1.0, 2.0));
        assert_eq!(
            Interval::<f64>::empty().convex_hull(iv(1.0, 2.0)),
            iv(1.0, 2.0)
        );
        assert!(iv(1.0, 2.0).convex_hull(Interval::entire()).is_entire());
    }
}
