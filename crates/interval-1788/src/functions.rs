//! Numeric, boolean, and set functions of intervals (the non-arithmetic part of
//! the set-based flavor).
//!
//! The numeric functions (`wid`, `mag`, `mig`) return `Option<F>`: `None` for the
//! empty interval, where the standard's Level 2 returns NaN. Returning `Option`
//! rather than a NaN sentinel keeps the empty case explicit in the type; a
//! NaN-returning shim for strict Level 2 datum conformance can sit at that
//! boundary later. The boolean relations and the set operations follow the
//! standard's set semantics, with the empty set handled as the identity it is
//! (empty is a subset of everything, disjoint from everything, and the identity
//! of convex hull).
//!
//! `mid` and `rad`, and the ordering relations (`less`, `precedes` and their
//! strict forms), are deferred: their empty and unbounded cases need the Level 2
//! conventions (a finite midpoint for an unbounded interval) that are best added
//! deliberately rather than in passing.

use crate::interval::Interval;
use crate::round::RoundFloat;

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
