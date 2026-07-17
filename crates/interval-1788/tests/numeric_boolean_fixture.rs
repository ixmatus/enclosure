//! Tests for the numeric and boolean function completion over the `f64` fixture:
//! the `mid`/`rad`/`mid_rad` numeric functions, the `equal`/`less`/`precedes`
//! orderings with their strict forms, and the sixteen-state `overlap` relation,
//! bare and decorated.
//!
//! The unit tests transcribe a representative selection of the vendored ITF1788
//! conformance vectors (`libieeep1788_num.itl`, `libieeep1788_bool.itl`,
//! `libieeep1788_overlap.itl`). The orderings and overlap are exact endpoint
//! comparisons, so their vectors are pinned exactly, empty tables included. The
//! numeric `mid`/`rad` cannot always reproduce the Level 2 datum on a
//! directed-rounding surface: the exact-midpoint cases (empty, singleton,
//! symmetric-about-zero, `Entire`) are pinned bit-for-bit, and the remaining
//! cases assert the defining enclosure property `[mid - rad, mid + rad]` contains
//! the interval, which holds on any sound backend (see the note on `mid_rad`).
//!
//! The property lanes add: (a) the defining enclosure property over random
//! bounded intervals; (b) `mid` inside the interval and `rad` nonnegative; (c)
//! ordering coherence (strict implies weak, `precedes` implies disjoint or
//! touching, `equal` iff `less` both ways); and (d) `overlap` totality and
//! consistency (the returned state's endpoint predicate holds).
//!
//! Cases derived from ITF1788 `libieeep1788_num.itl`, `libieeep1788_bool.itl`,
//! and `libieeep1788_overlap.itl` (Apache-2.0, original author Marco Nehmeier,
//! ITL port Oliver Heimlich; see `docs/references/vendor/itf1788-framework/`).

use interval_1788::{DecoratedInterval, Decoration, Interval, Overlap, RoundFloat};
use proptest::prelude::*;

const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;
const MAX: f64 = f64::MAX;

fn iv(lo: f64, hi: f64) -> Interval<f64> {
    Interval::new(lo, hi).unwrap()
}

fn di(lo: f64, hi: f64, d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(iv(lo, hi), d)
}

fn di_empty(d: Decoration) -> DecoratedInterval<f64> {
    DecoratedInterval::set_dec(Interval::<f64>::empty(), d)
}

fn nai() -> DecoratedInterval<f64> {
    DecoratedInterval::<f64>::nai()
}

// --- Numeric: mid / rad / mid_rad -----------------------------------------

/// The defining property `[mid - rad, mid + rad]` contains the interval,
/// computed with directed arithmetic exactly as the standard's enclosure obliges.
/// Holds on any sound backend regardless of how loose the midpoint estimate is.
fn assert_encloses(x: Interval<f64>) {
    let (mid, rad) = x.mid_rad().unwrap();
    assert!(
        mid.is_finite(),
        "mid must be finite for [{}, {}]",
        x.inf(),
        x.sup()
    );
    assert!(rad >= 0.0, "rad must be nonnegative");
    assert!(
        mid.sub_down(rad) <= x.inf() && mid.add_up(rad) >= x.sup(),
        "[mid - rad, mid + rad] must enclose x: x=[{}, {}] mid={mid} rad={rad}",
        x.inf(),
        x.sup(),
    );
}

#[test]
fn mid_rad_empty_is_none() {
    // minimal_mid_test / minimal_rad_test / minimal_mid_rad_test: empty -> NaN,
    // which the Option surface renders as None.
    let e = Interval::<f64>::empty();
    assert_eq!(e.mid(), None);
    assert_eq!(e.rad(), None);
    assert_eq!(e.mid_rad(), None);
}

#[test]
fn mid_rad_exact_pinned_cases() {
    // The cases the directed-ops derivation guarantees exactly on the fixture.
    // Entire centers at zero with an infinite radius.
    assert_eq!(Interval::<f64>::entire().mid(), Some(0.0));
    assert_eq!(Interval::<f64>::entire().rad(), Some(INF));
    assert_eq!(Interval::<f64>::entire().mid_rad(), Some((0.0, INF)));

    // Symmetric about zero: exact midpoint zero (mid pinned; rad is property-level
    // on the loose fixture, checked by the enclosure property below).
    assert_eq!(iv(-2.0, 2.0).mid(), Some(0.0));
    assert_eq!(iv(-MAX, MAX).mid(), Some(0.0)); // realmax-symmetric, still exact

    // Singleton: the point itself, radius exactly zero.
    assert_eq!(iv(2.0, 2.0).mid_rad(), Some((2.0, 0.0)));
    assert_eq!(iv(-27.0, -27.0).mid_rad(), Some((-27.0, 0.0)));
}

#[test]
fn mid_rad_general_bounded_encloses() {
    // minimal_*_test bounded cases. The fixture cannot reproduce the exact
    // midpoint through directed halving, so the enclosure property is the claim.
    // The large case also exercises the overflow-safe halved form: (a + b) would
    // overflow to +inf, the halved a/2 + b/2 does not.
    for x in [
        iv(0.0, 2.0),
        iv(-2.0, 2.0),
        iv(2.0, 2.0),
        iv(MAX / 2.0, MAX), // (a + b) overflows to +inf; the halved form does not
        iv(f64::from_bits(1), f64::from_bits(3)), // subnormal [1 ulp, 3 ulp]
        iv(-f64::from_bits(2), f64::from_bits(1)), // [-2 ulp, 1 ulp]
        iv(1.0, f64::from_bits(0x3FF0_0000_0000_0003)), // [1, 1 + 3 ulp]
    ] {
        assert_encloses(x);
        // A bounded interval has a finite radius and a midpoint it contains.
        let (mid, rad) = x.mid_rad().unwrap();
        assert!(rad.is_finite(), "bounded interval has a finite radius");
        assert!(x.contains(mid), "midpoint must lie inside the interval");
    }
}

#[test]
fn mid_rad_half_unbounded() {
    // minimal_mid_test pins realmax / -realmax here; the RoundFloat surface has no
    // largest-finite constant, so mid returns the finite endpoint (documented
    // divergence). The radius is +inf, so the enclosure holds trivially and the
    // reported midpoint is a sound member of the interval.
    for x in [
        iv(0.0, INF),
        iv(NEG_INF, 1.2),
        iv(-5.0, INF),
        iv(NEG_INF, -3.0),
    ] {
        assert_encloses(x);
        let (mid, rad) = x.mid_rad().unwrap();
        assert_eq!(rad, INF, "an unbounded interval has an infinite radius");
        assert!(x.contains(mid), "midpoint must lie inside the interval");
    }
}

// --- Boolean orderings: equal ---------------------------------------------

#[test]
fn equal_bare_vectors() {
    // minimal_equal_test
    assert!(iv(1.0, 2.0).equal(iv(1.0, 2.0)));
    assert!(!iv(1.0, 2.1).equal(iv(1.0, 2.0)));
    assert!(Interval::<f64>::empty().equal(Interval::empty()));
    assert!(!Interval::<f64>::empty().equal(iv(1.0, 2.0)));
    assert!(Interval::<f64>::entire().equal(Interval::entire()));
    assert!(!iv(1.0, 2.4).equal(Interval::entire()));
    assert!(iv(1.0, INF).equal(iv(1.0, INF)));
    assert!(!iv(1.0, 2.4).equal(iv(1.0, INF)));
    assert!(iv(NEG_INF, 2.0).equal(iv(NEG_INF, 2.0)));
    assert!(iv(-2.0, 0.0).equal(iv(-2.0, 0.0)));
    // Signed-zero endpoints denote the same set.
    assert!(iv(-0.0, 2.0).equal(iv(0.0, 2.0)));
    assert!(iv(-0.0, -0.0).equal(iv(0.0, 0.0)));
    assert!(iv(-0.0, 0.0).equal(iv(0.0, 0.0)));
    assert!(iv(0.0, -0.0).equal(iv(0.0, 0.0)));
}

// --- Boolean orderings: less / strict_less --------------------------------

#[test]
fn less_bare_vectors() {
    // minimal_less_test
    assert!(Interval::<f64>::empty().less(Interval::empty()));
    assert!(!iv(1.0, 2.0).less(Interval::empty()));
    assert!(!Interval::<f64>::empty().less(iv(1.0, 2.0)));

    assert!(Interval::<f64>::entire().less(Interval::entire()));
    assert!(!iv(1.0, 2.0).less(Interval::entire()));
    assert!(!Interval::<f64>::entire().less(iv(1.0, 2.0)));

    assert!(iv(0.0, 2.0).less(iv(0.0, 2.0)));
    assert!(iv(0.0, 2.0).less(iv(-0.0, 2.0)));
    assert!(iv(0.0, 2.0).less(iv(1.0, 2.0)));
    assert!(iv(1.0, 2.0).less(iv(1.0, 2.0)));
    assert!(iv(1.0, 2.0).less(iv(3.0, 4.0)));
    assert!(iv(1.0, 3.5).less(iv(3.0, 4.0)));
    assert!(iv(1.0, 4.0).less(iv(3.0, 4.0)));
    assert!(iv(-3.0, -1.5).less(iv(-2.0, -1.0)));
    // Signed-zero rows: all true.
    assert!(iv(0.0, 0.0).less(iv(-0.0, -0.0)));
    assert!(iv(-0.0, 0.0).less(iv(0.0, -0.0)));
}

#[test]
fn strict_less_bare_vectors() {
    // minimal_strictly_less_test
    assert!(Interval::<f64>::empty().strict_less(Interval::empty()));
    assert!(!iv(1.0, 2.0).strict_less(Interval::empty()));
    assert!(!Interval::<f64>::empty().strict_less(iv(1.0, 2.0)));

    // Two matching infinite endpoints satisfy the strict clause.
    assert!(Interval::<f64>::entire().strict_less(Interval::entire()));
    assert!(!iv(1.0, 2.0).strict_less(Interval::entire()));
    assert!(!Interval::<f64>::entire().strict_less(iv(1.0, 2.0)));

    assert!(!iv(1.0, 2.0).strict_less(iv(1.0, 2.0)));
    assert!(iv(1.0, 2.0).strict_less(iv(3.0, 4.0)));
    assert!(iv(1.0, 3.5).strict_less(iv(3.0, 4.0)));
    assert!(!iv(1.0, 4.0).strict_less(iv(3.0, 4.0)));
    assert!(!iv(0.0, 4.0).strict_less(iv(0.0, 4.0)));
    assert!(!iv(-0.0, 4.0).strict_less(iv(0.0, 4.0)));
    assert!(!iv(-2.0, -1.0).strict_less(iv(-2.0, -1.0)));
    assert!(iv(-3.0, -1.5).strict_less(iv(-2.0, -1.0)));
    // A half-unbounded interval strictly less than a wider half-unbounded one:
    // the lower endpoints are both -inf (matching-infinity clause), uppers strict.
    assert!(iv(NEG_INF, 1.0).strict_less(iv(NEG_INF, 2.0)));
    assert!(iv(1.0, INF).strict_less(iv(2.0, INF)));
}

// --- Boolean orderings: precedes / strict_precedes ------------------------

#[test]
fn precedes_bare_vectors() {
    // minimal_precedes_test
    assert!(Interval::<f64>::empty().precedes(iv(3.0, 4.0)));
    assert!(iv(3.0, 4.0).precedes(Interval::empty()));
    assert!(Interval::<f64>::empty().precedes(Interval::empty()));

    assert!(!iv(1.0, 2.0).precedes(Interval::entire()));
    assert!(!Interval::<f64>::entire().precedes(iv(1.0, 2.0)));
    assert!(!Interval::<f64>::entire().precedes(Interval::entire()));

    assert!(iv(1.0, 2.0).precedes(iv(3.0, 4.0)));
    assert!(iv(1.0, 3.0).precedes(iv(3.0, 4.0))); // touching allowed
    assert!(iv(-3.0, -1.0).precedes(iv(-1.0, 0.0)));
    assert!(!iv(1.0, 3.5).precedes(iv(3.0, 4.0)));
    assert!(!iv(1.0, 4.0).precedes(iv(3.0, 4.0)));
    assert!(!iv(-3.0, -0.1).precedes(iv(-1.0, 0.0)));
    // Signed-zero touch: sup 0.0 <= inf -0.0 holds (they are equal).
    assert!(iv(0.0, 0.0).precedes(iv(-0.0, -0.0)));
}

#[test]
fn strict_precedes_bare_vectors() {
    // minimal_strictly_precedes_test
    assert!(Interval::<f64>::empty().strict_precedes(iv(3.0, 4.0)));
    assert!(iv(3.0, 4.0).strict_precedes(Interval::empty()));
    assert!(Interval::<f64>::empty().strict_precedes(Interval::empty()));

    assert!(!iv(1.0, 2.0).strict_precedes(Interval::entire()));
    assert!(!Interval::<f64>::entire().strict_precedes(iv(1.0, 2.0)));

    assert!(iv(1.0, 2.0).strict_precedes(iv(3.0, 4.0)));
    assert!(!iv(1.0, 3.0).strict_precedes(iv(3.0, 4.0))); // touch is not strict
    assert!(!iv(-3.0, -1.0).strict_precedes(iv(-1.0, 0.0)));
    // Signed-zero touch is not strict: sup -0.0 < inf 0.0 is false.
    assert!(!iv(-3.0, -0.0).strict_precedes(iv(0.0, 1.0)));
    assert!(!iv(-3.0, 0.0).strict_precedes(iv(-0.0, 1.0)));
    assert!(!iv(1.0, 3.5).strict_precedes(iv(3.0, 4.0)));
}

// --- The overlap relation -------------------------------------------------

#[test]
fn overlap_empty_states() {
    // minimal_overlap_test, the three empty states.
    let e = Interval::<f64>::empty();
    assert_eq!(e.overlap(e), Overlap::BothEmpty);
    assert_eq!(e.overlap(iv(1.0, 2.0)), Overlap::FirstEmpty);
    assert_eq!(iv(1.0, 2.0).overlap(e), Overlap::SecondEmpty);
}

#[test]
fn overlap_forward_states() {
    // minimal_overlap_test: before through equals (self at or left of other).
    assert_eq!(iv(NEG_INF, 2.0).overlap(iv(3.0, INF)), Overlap::Before);
    assert_eq!(iv(2.0, 2.0).overlap(iv(3.0, 4.0)), Overlap::Before);
    assert_eq!(iv(2.0, 2.0).overlap(iv(3.0, 3.0)), Overlap::Before);

    assert_eq!(iv(NEG_INF, 2.0).overlap(iv(2.0, 3.0)), Overlap::Meets);
    assert_eq!(iv(1.0, 2.0).overlap(iv(2.0, 3.0)), Overlap::Meets);
    assert_eq!(iv(1.0, 2.0).overlap(iv(2.0, INF)), Overlap::Meets);

    assert_eq!(iv(1.0, 2.0).overlap(iv(1.5, 2.5)), Overlap::Overlaps);

    assert_eq!(iv(1.0, 2.0).overlap(iv(1.0, INF)), Overlap::Starts);
    assert_eq!(iv(1.0, 2.0).overlap(iv(1.0, 3.0)), Overlap::Starts);
    assert_eq!(iv(1.0, 1.0).overlap(iv(1.0, 3.0)), Overlap::Starts);

    assert_eq!(
        iv(1.0, 2.0).overlap(Interval::entire()),
        Overlap::ContainedBy
    );
    assert_eq!(iv(1.0, 2.0).overlap(iv(0.0, 3.0)), Overlap::ContainedBy);
    assert_eq!(iv(2.0, 2.0).overlap(iv(0.0, INF)), Overlap::ContainedBy);

    assert_eq!(iv(1.0, 2.0).overlap(iv(NEG_INF, 2.0)), Overlap::Finishes);
    assert_eq!(iv(1.0, 2.0).overlap(iv(0.0, 2.0)), Overlap::Finishes);
    assert_eq!(iv(2.0, 2.0).overlap(iv(0.0, 2.0)), Overlap::Finishes);

    assert_eq!(iv(1.0, 2.0).overlap(iv(1.0, 2.0)), Overlap::Equals);
    assert_eq!(iv(1.0, 1.0).overlap(iv(1.0, 1.0)), Overlap::Equals);
    assert_eq!(iv(NEG_INF, 1.0).overlap(iv(NEG_INF, 1.0)), Overlap::Equals);
    assert_eq!(
        Interval::<f64>::entire().overlap(Interval::entire()),
        Overlap::Equals
    );
}

#[test]
fn overlap_reverse_states() {
    // minimal_overlap_test: the mirror states (self at or right of other).
    assert_eq!(iv(3.0, 4.0).overlap(iv(2.0, 2.0)), Overlap::After);
    assert_eq!(iv(3.0, 3.0).overlap(iv(2.0, 2.0)), Overlap::After);
    assert_eq!(iv(3.0, INF).overlap(iv(2.0, 2.0)), Overlap::After);

    assert_eq!(iv(2.0, 3.0).overlap(iv(1.0, 2.0)), Overlap::MetBy);
    assert_eq!(iv(2.0, 3.0).overlap(iv(NEG_INF, 2.0)), Overlap::MetBy);

    assert_eq!(iv(1.5, 2.5).overlap(iv(1.0, 2.0)), Overlap::OverlappedBy);
    assert_eq!(
        iv(1.5, 2.5).overlap(iv(NEG_INF, 2.0)),
        Overlap::OverlappedBy
    );

    assert_eq!(iv(1.0, INF).overlap(iv(1.0, 2.0)), Overlap::StartedBy);
    assert_eq!(iv(1.0, 3.0).overlap(iv(1.0, 2.0)), Overlap::StartedBy);
    assert_eq!(iv(1.0, 3.0).overlap(iv(1.0, 1.0)), Overlap::StartedBy);

    assert_eq!(iv(NEG_INF, 3.0).overlap(iv(1.0, 2.0)), Overlap::Contains);
    assert_eq!(
        Interval::<f64>::entire().overlap(iv(1.0, 2.0)),
        Overlap::Contains
    );
    assert_eq!(iv(0.0, 3.0).overlap(iv(2.0, 2.0)), Overlap::Contains);

    assert_eq!(iv(NEG_INF, 2.0).overlap(iv(1.0, 2.0)), Overlap::FinishedBy);
    assert_eq!(iv(0.0, 2.0).overlap(iv(1.0, 2.0)), Overlap::FinishedBy);
    assert_eq!(iv(0.0, 2.0).overlap(iv(2.0, 2.0)), Overlap::FinishedBy);
}

// --- Decorated booleans: the NaI rule -------------------------------------

#[test]
fn decorated_bool_reads_interval_part() {
    // minimal_*_dec_test: the decoration is ignored, the interval part decides.
    assert!(di(1.0, 2.0, Decoration::Def).equal(di(1.0, 2.0, Decoration::Trv)));
    assert!(di(0.0, 2.0, Decoration::Def).less(di(1.0, 2.0, Decoration::Def)));
    assert!(di(1.0, 3.5, Decoration::Def).strict_less(di(3.0, 4.0, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).precedes(di(3.0, 4.0, Decoration::Def)));
    assert!(di(1.0, 2.0, Decoration::Trv).strict_precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).subset(di(0.0, 4.0, Decoration::Def)));
    assert!(di(1.0, 2.0, Decoration::Def).interior(di(0.0, 4.0, Decoration::Trv)));
    assert!(di(3.0, 4.0, Decoration::Trv).disjoint(di(1.0, 2.0, Decoration::Def)));
    // Both empty operands still equal (they denote the same set).
    assert!(di_empty(Decoration::Trv).equal(di_empty(Decoration::Trv)));
}

#[test]
fn decorated_bool_nai_is_always_false() {
    // Every relation is false when either operand is NaI.
    let good = di(1.0, 2.0, Decoration::Def);
    assert!(!nai().equal(nai()));
    assert!(!di_empty(Decoration::Trv).equal(nai()));
    assert!(!nai().equal(di_empty(Decoration::Trv)));
    assert!(!nai().less(good));
    assert!(!good.less(nai()));
    assert!(!nai().strict_less(good));
    assert!(!nai().precedes(good));
    assert!(!good.precedes(nai()));
    assert!(!nai().strict_precedes(good));
    assert!(!nai().subset(good));
    assert!(!good.subset(nai()));
    assert!(!nai().interior(good));
    assert!(!good.interior(nai()));
    assert!(!nai().disjoint(good));
    assert!(!good.disjoint(nai()));
}

#[test]
fn decorated_overlap_reads_interval_part() {
    // minimal_overlap_dec_test: the decoration is ignored.
    assert_eq!(
        di_empty(Decoration::Trv).overlap(di_empty(Decoration::Trv)),
        Overlap::BothEmpty
    );
    assert_eq!(
        di_empty(Decoration::Trv).overlap(di(1.0, 2.0, Decoration::Com)),
        Overlap::FirstEmpty
    );
    assert_eq!(
        di(1.0, 2.0, Decoration::Dac).overlap(di(1.5, 2.5, Decoration::Def)),
        Overlap::Overlaps
    );
    assert_eq!(
        di(0.0, 3.0, Decoration::Com).overlap(di(1.0, 2.0, Decoration::Dac)),
        Overlap::Contains
    );
    // No NaI case is pinned by the vectors; a NaI reads through as its empty
    // interval part, so two NaIs report bothEmpty.
    assert_eq!(nai().overlap(nai()), Overlap::BothEmpty);
}

// --- Property lanes -------------------------------------------------------

fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![-1.0e6..1.0e6, Just(0.0), Just(-0.0), -1.0e-6..1.0e-6]
}

fn bounded() -> impl Strategy<Value = Interval<f64>> {
    (finite(), finite()).prop_map(|(a, b)| {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        iv(lo, hi)
    })
}

/// Any interval: bounded, half-unbounded, entire, or empty.
fn any_iv() -> impl Strategy<Value = Interval<f64>> {
    prop_oneof![
        bounded(),
        finite().prop_map(|a| iv(a, INF)),
        finite().prop_map(|b| iv(NEG_INF, b)),
        Just(Interval::<f64>::entire()),
        Just(Interval::<f64>::empty()),
    ]
}

proptest! {
    // (a) THE defining property over random bounded intervals.
    #[test]
    fn mid_rad_encloses_bounded(x in bounded()) {
        assert_encloses(x);
    }

    // (b) mid lies inside the interval; rad is nonnegative.
    #[test]
    fn mid_inside_and_rad_nonnegative(x in bounded()) {
        let (mid, rad) = x.mid_rad().unwrap();
        prop_assert!(x.contains(mid), "mid outside [{}, {}]", x.inf(), x.sup());
        prop_assert!(rad >= 0.0);
    }

    // (c) Ordering coherence.
    #[test]
    fn strict_less_implies_less(a in any_iv(), b in any_iv()) {
        if a.strict_less(b) {
            prop_assert!(a.less(b));
        }
    }

    #[test]
    fn strict_precedes_implies_precedes(a in any_iv(), b in any_iv()) {
        if a.strict_precedes(b) {
            prop_assert!(a.precedes(b));
        }
    }

    #[test]
    fn precedes_implies_disjoint_or_touching(a in any_iv(), b in any_iv()) {
        // sup(self) <= inf(other): either a gap (disjoint) or a shared touching
        // point (sup == inf). Empty operands make precedes and disjoint both true.
        if a.precedes(b) {
            prop_assert!(a.disjoint(b) || a.sup() == b.inf());
        }
    }

    #[test]
    fn equal_iff_less_both_ways(a in any_iv(), b in any_iv()) {
        // Antisymmetry of the weak order: a == b exactly when a <= b and b <= a.
        prop_assert_eq!(a.equal(b), a.less(b) && b.less(a));
    }

    // (d) overlap is total and consistent: the returned state's defining endpoint
    // predicate holds for the operands.
    #[test]
    fn overlap_state_predicate_holds(a in any_iv(), b in any_iv()) {
        prop_assert!(overlap_predicate_holds(a, b, a.overlap(b)));
    }

    // overlap mirrors under swapping the operands.
    #[test]
    fn overlap_swaps_to_the_mirror_state(a in any_iv(), b in any_iv()) {
        prop_assert_eq!(mirror(a.overlap(b)), b.overlap(a));
    }
}

/// The endpoint predicate defining each `Overlap` state; used to check that the
/// state `overlap` returns is the correct one (consistency), for every pair.
fn overlap_predicate_holds(a: Interval<f64>, b: Interval<f64>, state: Overlap) -> bool {
    let (Some((al, ah)), Some((bl, bh))) = (bounds(a), bounds(b)) else {
        return matches!(
            (a.is_empty(), b.is_empty(), state),
            (true, true, Overlap::BothEmpty)
                | (true, false, Overlap::FirstEmpty)
                | (false, true, Overlap::SecondEmpty)
        );
    };
    match state {
        Overlap::Before => ah < bl,
        Overlap::Meets => ah == bl && al < ah && bl < bh,
        Overlap::Overlaps => al < bl && bl < ah && ah < bh,
        Overlap::Starts => al == bl && ah < bh,
        Overlap::ContainedBy => bl < al && ah < bh,
        Overlap::Finishes => bl < al && ah == bh,
        Overlap::Equals => al == bl && ah == bh,
        Overlap::FinishedBy => al < bl && ah == bh,
        Overlap::Contains => al < bl && bh < ah,
        Overlap::StartedBy => al == bl && bh < ah,
        Overlap::OverlappedBy => bl < al && al < bh && bh < ah,
        Overlap::MetBy => al == bh && al < ah && bl < bh,
        Overlap::After => bh < al,
        Overlap::BothEmpty | Overlap::FirstEmpty | Overlap::SecondEmpty => false,
    }
}

/// Endpoints of a nonempty interval, or `None` for the empty one (a test-local
/// mirror of the crate-internal accessor).
fn bounds(x: Interval<f64>) -> Option<(f64, f64)> {
    if x.is_empty() {
        None
    } else {
        Some((x.inf(), x.sup()))
    }
}

/// The mirror of an `Overlap` state under swapping the two operands.
fn mirror(state: Overlap) -> Overlap {
    match state {
        Overlap::BothEmpty => Overlap::BothEmpty,
        Overlap::FirstEmpty => Overlap::SecondEmpty,
        Overlap::SecondEmpty => Overlap::FirstEmpty,
        Overlap::Before => Overlap::After,
        Overlap::Meets => Overlap::MetBy,
        Overlap::Overlaps => Overlap::OverlappedBy,
        Overlap::Starts => Overlap::StartedBy,
        Overlap::ContainedBy => Overlap::Contains,
        Overlap::Finishes => Overlap::FinishedBy,
        Overlap::Equals => Overlap::Equals,
        Overlap::FinishedBy => Overlap::Finishes,
        Overlap::Contains => Overlap::ContainedBy,
        Overlap::StartedBy => Overlap::Starts,
        Overlap::OverlappedBy => Overlap::Overlaps,
        Overlap::MetBy => Overlap::Meets,
        Overlap::After => Overlap::Before,
    }
}
