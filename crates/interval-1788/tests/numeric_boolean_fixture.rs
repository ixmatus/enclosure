//! Tests for the numeric, boolean, recommended-boolean, and set functions over
//! the `f64` fixture: the `inf`/`sup`/`wid`/`mag`/`mig`/`mid`/`rad` numeric
//! functions; the `isEmpty`/`isEntire`/`isNaI` predicates; the
//! `equal`/`less`/`precedes` orderings with their strict forms; `subset`,
//! `interior`, `disjoint`; the sixteen-state `overlap` relation; the recommended
//! `isSingleton`/`isCommonInterval`/`isMember`; and the set operations
//! `intersection`/`convexHull`, bare and decorated.
//!
//! The unit tests hand-translate the vendored ITF1788 conformance vectors
//! (`libieeep1788_num.itl`, `libieeep1788_bool.itl`, `libieeep1788_overlap.itl`,
//! `libieeep1788_rec_bool.itl`, `libieeep1788_set.itl`). Every transcribed
//! testcase is complete: every assertion statement present, none sampled, with a
//! `// <testcase>: N vectors` count comment. Most testcases are one `#[test]` fn;
//! the deliberate exceptions are `minimal_overlap_test` (48 vectors split
//! across the empty/forward/reverse fns, 3 + 26 + 19, each fn stating its share)
//! and the `mid`/`rad`/`midRad` bare testcases (each split into a live fn of the
//! vectors this fixture reproduces bit-exactly and an `#[ignore]`d KNOWN GAP twin
//! holding the corpus datum for the rest; the count comments state the split).
//! The four `mid_rad_*` fns above the 1:1 lane are curated-supplementary (a
//! seven-interval property lane predating the corpus transcription), not corpus
//! tiling.
//!
//! # What is exact, what is sound-not-tight
//!
//! - `inf`, `sup`, `mag`, `mig` are pure endpoint or magnitude reads with no
//!   rounding: pinned exactly. (A signed zero at an endpoint is asserted at the
//!   Level 2 value, where `-0.0 == +0.0`; the fixture's `inf([0, +inf])` returns
//!   `+0.0` where the corpus writes `-0.0`, a harmless representation-of-zero
//!   difference the value assertion is blind to by design.)
//! - `wid` is `sup - inf` rounded outward. On the `f64` fixture directed
//!   subtraction is `(sup - sup).next_up()`, one ulp wide even when the exact
//!   difference is representable, so a finite width is asserted as a SOUND upper
//!   bound (`wid >= tightest`), never pinned; the empty (`None`) and unbounded
//!   (`+inf`) widths are exact.
//! - the orderings, `overlap`, `subset`, `interior`, `disjoint`, `isEmpty`,
//!   `isEntire`, `isNaI`, `isSingleton`, `intersection`, and `convexHull` are
//!   exact set/endpoint logic: pinned exactly, empty tables included.
//! - `mid`/`rad`/`midRad` are transcribed 1:1 with exact assertions. The
//!   structural, singleton, symmetric, and half-unbounded vectors pass on this
//!   fixture; the general bounded vectors cannot (directed halving lands the mid
//!   ulps wide, `sub_up` over-estimates every finite nonzero radius), so those
//!   hold the corpus datum in `#[ignore]`d KNOWN GAP fns that go green on a
//!   tight backend. Soundness of the loose values is covered by the curated fns
//!   and the property lanes.
//!
//! # Required-operation gaps (see the `KNOWN GAP` block below)
//!
//! Several corpus testcases exercise operations the crate does not surface. Their
//! bare siblings are covered; the decorated or recommended forms are reported,
//! not faked. The crate deliberately factors decoration handling: the boolean and
//! relational operations have decorated forms (transcribed 1:1 above), and
//! `isNaI_dec` is transcribed 1:1 against `is_nai()`, but the numeric accessors,
//! the remaining unary predicates, and the set operations have no decorated
//! forms, so those `_dec` testcases (and the recommended `isCommonInterval` /
//! `isMember`, whose bare intents are covered by documented compositions over
//! `is_bounded` / `contains`) are recorded as gaps with their corpus counts, not
//! transcribed.
//!
//! The property lanes add: (a) the defining `mid`/`rad` enclosure property over
//! random bounded intervals; (b) `mid` inside the interval and `rad` nonnegative;
//! (c) ordering coherence; and (d) `overlap` totality and consistency.
//!
//! Cases derived from ITF1788 `libieeep1788_num.itl`, `libieeep1788_bool.itl`,
//! `libieeep1788_overlap.itl`, `libieeep1788_rec_bool.itl`, and
//! `libieeep1788_set.itl` (Apache-2.0, original author Marco Nehmeier, ITL port
//! Oliver Heimlich; see `docs/references/vendor/itf1788-framework/`).

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

/// Parse a C hex-float literal (`[-]0xH.HHHHp[+-]N`) to the exact `f64` it names.
/// The `num`/`rec_bool` vectors pin bit patterns in this form; parsing rather
/// than pre-converting keeps the transcription faithful.
fn hx(s: &str) -> f64 {
    let s = s.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .expect("hex-float must start with 0x");
    let p = s.find(['p', 'P']).expect("hex-float needs a p exponent");
    let (mant, exp) = (&s[..p], &s[p + 1..]);
    let exp: i32 = exp.parse().expect("hex-float exponent");
    let (int_part, frac_part) = match mant.find('.') {
        Some(i) => (&mant[..i], &mant[i + 1..]),
        None => (mant, ""),
    };
    let mut m: u64 = 0;
    for c in int_part.chars() {
        m = m * 16 + u64::from(c.to_digit(16).expect("hex digit"));
    }
    let mut frac_len = 0i32;
    for c in frac_part.chars() {
        m = m * 16 + u64::from(c.to_digit(16).expect("hex digit"));
        frac_len += 1;
    }
    // A canonical corpus literal carries at most 53 significant bits, so the cast
    // is exact; the assertion turns a non-canonical literal into a loud failure
    // instead of a silent rounding (mirrors reverse_rev_fixture.rs).
    assert!(m < (1 << 53), "hex-float mantissa exceeds f64 exact range");
    #[allow(clippy::cast_precision_loss)]
    let mut val = m as f64;
    let mut e = exp - 4 * frac_len;
    while e > 0 {
        val *= 2.0;
        e -= 1;
    }
    while e < 0 {
        val *= 0.5;
        e += 1;
    }
    if neg {
        -val
    } else {
        val
    }
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
    // Curated-supplementary (the 1:1 corpus transcription is in the
    // `*_bare_vectors` fns below): empty -> NaN, which the Option surface
    // renders as None.
    let e = Interval::<f64>::empty();
    assert_eq!(e.mid(), None);
    assert_eq!(e.rad(), None);
    assert_eq!(e.mid_rad(), None);
}

#[test]
fn mid_rad_exact_pinned_cases() {
    // Curated-supplementary (see the 1:1 `*_bare_vectors` fns below).
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
    // Curated-supplementary bounded cases (a seven-interval property lane, not
    // the corpus; the 1:1 transcription is in the `*_bare_vectors` fns below).
    // The fixture cannot reproduce the exact midpoint through directed halving,
    // so the enclosure property is the claim.
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
    // Curated-supplementary (see the 1:1 `*_bare_vectors` fns below).
    // minimal_mid_test pins realmax / -realmax here, now reachable bit-exact
    // through the largest-finite capability: `mid([a, +inf]) = +MAX` and
    // `mid([-inf, b]) = -MAX`, radius +inf. These two vectors were previously
    // unreachable (the surface had no largest-finite datum) and asserted only the
    // enclosure property; they are pinned bit-for-bit now.
    assert_eq!(iv(0.0, INF).mid(), Some(MAX)); // libieeep1788_num.itl mid [0.0,infinity]
    assert_eq!(iv(NEG_INF, 1.2).mid(), Some(-MAX)); // libieeep1788_num.itl mid [-infinity,1.2]
    assert_eq!(iv(0.0, INF).mid_rad(), Some((MAX, INF)));
    assert_eq!(iv(NEG_INF, 1.2).mid_rad(), Some((-MAX, INF)));

    // The convention across all four half-unbounded shapes: +MAX when unbounded
    // above, -MAX when unbounded below, radius +inf, midpoint a member of the set.
    for (x, want_mid) in [
        (iv(0.0, INF), MAX),
        (iv(NEG_INF, 1.2), -MAX),
        (iv(-5.0, INF), MAX),
        (iv(NEG_INF, -3.0), -MAX),
    ] {
        assert_encloses(x);
        let (mid, rad) = x.mid_rad().unwrap();
        assert_eq!(
            mid, want_mid,
            "half-unbounded mid follows the realmax convention"
        );
        assert_eq!(rad, INF, "an unbounded interval has an infinite radius");
        assert!(x.contains(mid), "midpoint must lie inside the interval");
    }
}

// --- Numeric: mid / rad / midRad, the 1:1 corpus transcription -------------
//
// Every vector of minimal_mid_test, minimal_rad_test, and minimal_mid_rad_test,
// exact against the corpus datum. The vectors the sound-not-tight fixture cannot
// reproduce bit-exactly (its directed ops step outward even on representable
// results, so a general bounded midpoint lands ulps wide and a radius is always
// an over-estimate) are split into `#[ignore]`d KNOWN GAP twins holding the
// corpus datum, so each goes green the day a tight backend (or a tighter fixture)
// runs this lane. Soundness of the loose values (mid within the interval; the
// enclosure property; rad never under-estimated) is covered by the curated fns
// and property lanes above. Never adjust expected values.

#[test]
fn mid_bare_vectors() {
    // minimal_mid_test: 7 of 12 vectors exact on this fixture (the remaining 5
    // are in mid_bare_vectors_fixture_loose_known_gap; 7 + 5 = 12).
    assert_eq!(Interval::<f64>::empty().mid(), None); // corpus NaN -> None
    assert_eq!(Interval::<f64>::entire().mid(), Some(0.0));
    assert_eq!(iv(-MAX, MAX).mid(), Some(0.0)); // [-0x1.FFFFFFFFFFFFFp1023, +same]
    assert_eq!(iv(2.0, 2.0).mid(), Some(2.0));
    assert_eq!(iv(-2.0, 2.0).mid(), Some(0.0));
    assert_eq!(iv(0.0, INF).mid(), Some(hx("0x1.FFFFFFFFFFFFFp1023")));
    assert_eq!(iv(NEG_INF, 1.2).mid(), Some(hx("-0x1.FFFFFFFFFFFFFp1023")));
}

#[test]
#[ignore = "KNOWN GAP: fixture looseness; general-bounded mid is ulps wide of the tight Level 2 datum"]
fn mid_bare_vectors_fixture_loose_known_gap() {
    // minimal_mid_test: the 5 general-bounded vectors (of 12) the fixture's
    // directed halving cannot pin. Measured fixture values (all members of the
    // interval, enclosure preserved): mid([0,2]) = 1 - 4 ulps; the two
    // mixed-subnormal mids land one endpoint inward instead of zero; the realmax
    // shoulder lands 3 ulps low; the pure-subnormal mid lands one ulp low.
    // Corpus datum held below; goes green on a tight backend.
    assert_eq!(iv(0.0, 2.0).mid(), Some(1.0));
    assert_eq!(
        iv(
            hx("-0X0.0000000000002P-1022"),
            hx("0X0.0000000000001P-1022")
        )
        .mid(),
        Some(0.0)
    );
    assert_eq!(
        iv(
            hx("-0X0.0000000000001P-1022"),
            hx("0X0.0000000000002P-1022")
        )
        .mid(),
        Some(0.0)
    );
    assert_eq!(
        iv(hx("0X1.FFFFFFFFFFFFFP+1022"), hx("0X1.FFFFFFFFFFFFFP+1023")).mid(),
        Some(hx("0X1.7FFFFFFFFFFFFP+1023"))
    );
    assert_eq!(
        iv(hx("0X0.0000000000001P-1022"), hx("0X0.0000000000003P-1022")).mid(),
        Some(hx("0X0.0000000000002P-1022"))
    );
}

#[test]
fn rad_bare_vectors() {
    // minimal_rad_test: 5 of 9 vectors exact on this fixture (the remaining 4
    // are in rad_bare_vectors_fixture_loose_known_gap; 5 + 4 = 9).
    assert_eq!(iv(2.0, 2.0).rad(), Some(0.0));
    assert_eq!(Interval::<f64>::empty().rad(), None); // corpus NaN -> None
    assert_eq!(Interval::<f64>::entire().rad(), Some(INF));
    assert_eq!(iv(0.0, INF).rad(), Some(INF));
    assert_eq!(iv(NEG_INF, 1.2).rad(), Some(INF));
}

#[test]
#[ignore = "KNOWN GAP: fixture looseness; a finite nonzero rad is always an outward over-estimate"]
fn rad_bare_vectors_fixture_loose_known_gap() {
    // minimal_rad_test: the 4 finite-nonzero-radius vectors (of 9). The
    // fixture's rad is `sub_up`-widened (and compounds the loose mid), so every
    // measured value is strictly above the corpus datum, never below (sound).
    // Corpus datum held below; goes green on a tight backend.
    assert_eq!(iv(0.0, 2.0).rad(), Some(1.0));
    assert_eq!(
        iv(
            hx("-0X0.0000000000002P-1022"),
            hx("0X0.0000000000001P-1022")
        )
        .rad(),
        Some(hx("0X0.0000000000002P-1022"))
    );
    assert_eq!(
        iv(hx("0X0.0000000000001P-1022"), hx("0X0.0000000000002P-1022")).rad(),
        Some(hx("0X0.0000000000001P-1022"))
    );
    assert_eq!(
        iv(hx("0X1P+0"), hx("0X1.0000000000003P+0")).rad(),
        Some(hx("0X1P-51"))
    );
}

#[test]
fn mid_rad_bare_vectors() {
    // minimal_mid_rad_test: 5 of 12 vectors exact on this fixture (the remaining
    // 7 are in mid_rad_bare_vectors_fixture_loose_known_gap; 5 + 7 = 12).
    assert_eq!(Interval::<f64>::empty().mid_rad(), None); // corpus NaN NaN -> None
    assert_eq!(Interval::<f64>::entire().mid_rad(), Some((0.0, INF)));
    assert_eq!(iv(2.0, 2.0).mid_rad(), Some((2.0, 0.0)));
    assert_eq!(
        iv(0.0, INF).mid_rad(),
        Some((hx("0X1.FFFFFFFFFFFFFP+1023"), INF))
    );
    assert_eq!(
        iv(NEG_INF, 1.2).mid_rad(),
        Some((hx("-0X1.FFFFFFFFFFFFFP+1023"), INF))
    );
}

#[test]
#[ignore = "KNOWN GAP: fixture looseness; bounded mid/rad pairs are outward of the tight Level 2 data"]
fn mid_rad_bare_vectors_fixture_loose_known_gap() {
    // minimal_mid_rad_test: the 7 bounded non-singleton vectors (of 12). Beyond
    // the loose-mid and over-estimated-rad failures above, the realmax-symmetric
    // pair fails on the radius alone: mid([-MAX, MAX]) = 0 is exact, but
    // `rad = sub_up(MAX, 0)` steps outward to +inf where the corpus pins MAX
    // (sound, since [0 - inf, 0 + inf] still encloses; not the tight datum).
    // Corpus datum held below; goes green on a tight backend.
    assert_eq!(iv(-MAX, MAX).mid_rad(), Some((0.0, MAX)));
    assert_eq!(iv(0.0, 2.0).mid_rad(), Some((1.0, 1.0)));
    assert_eq!(iv(-2.0, 2.0).mid_rad(), Some((0.0, 2.0)));
    assert_eq!(
        iv(
            hx("-0X0.0000000000002P-1022"),
            hx("0X0.0000000000001P-1022")
        )
        .mid_rad(),
        Some((0.0, hx("0X0.0000000000002P-1022")))
    );
    assert_eq!(
        iv(
            hx("-0X0.0000000000001P-1022"),
            hx("0X0.0000000000002P-1022")
        )
        .mid_rad(),
        Some((0.0, hx("0X0.0000000000002P-1022")))
    );
    assert_eq!(
        iv(hx("0X1.FFFFFFFFFFFFFP+1022"), hx("0X1.FFFFFFFFFFFFFP+1023")).mid_rad(),
        Some((hx("0X1.7FFFFFFFFFFFFP+1023"), hx("0x1.0p+1022")))
    );
    assert_eq!(
        iv(hx("0X0.0000000000001P-1022"), hx("0X0.0000000000003P-1022")).mid_rad(),
        Some((hx("0X0.0000000000002P-1022"), hx("0X0.0000000000001P-1022")))
    );
}

// --- Boolean orderings: equal ---------------------------------------------

#[test]
fn equal_bare_vectors() {
    // minimal_equal_test: 15 vectors
    assert!(iv(1.0, 2.0).equal(iv(1.0, 2.0)));
    assert!(!iv(1.0, 2.1).equal(iv(1.0, 2.0)));
    assert!(Interval::<f64>::empty().equal(Interval::empty()));
    assert!(!Interval::<f64>::empty().equal(iv(1.0, 2.0)));
    assert!(Interval::<f64>::entire().equal(Interval::entire()));
    assert!(!iv(1.0, 2.4).equal(Interval::entire()));
    assert!(iv(1.0, INF).equal(iv(1.0, INF)));
    assert!(!iv(1.0, 2.4).equal(iv(1.0, INF)));
    assert!(iv(NEG_INF, 2.0).equal(iv(NEG_INF, 2.0)));
    assert!(!iv(NEG_INF, 2.4).equal(iv(NEG_INF, 2.0)));
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
    // minimal_less_test: 26 vectors
    assert!(Interval::<f64>::empty().less(Interval::empty()));
    assert!(!iv(1.0, 2.0).less(Interval::empty()));
    assert!(!Interval::<f64>::empty().less(iv(1.0, 2.0)));

    assert!(Interval::<f64>::entire().less(Interval::entire()));
    assert!(!iv(1.0, 2.0).less(Interval::entire()));
    assert!(!iv(0.0, 2.0).less(Interval::entire()));
    assert!(!iv(-0.0, 2.0).less(Interval::entire()));
    assert!(!Interval::<f64>::entire().less(iv(1.0, 2.0)));
    assert!(!Interval::<f64>::entire().less(iv(0.0, 2.0)));
    assert!(!Interval::<f64>::entire().less(iv(-0.0, 2.0)));

    assert!(iv(0.0, 2.0).less(iv(0.0, 2.0)));
    assert!(iv(0.0, 2.0).less(iv(-0.0, 2.0)));
    assert!(iv(0.0, 2.0).less(iv(1.0, 2.0)));
    assert!(iv(-0.0, 2.0).less(iv(1.0, 2.0)));
    assert!(iv(1.0, 2.0).less(iv(1.0, 2.0)));
    assert!(iv(1.0, 2.0).less(iv(3.0, 4.0)));
    assert!(iv(1.0, 3.5).less(iv(3.0, 4.0)));
    assert!(iv(1.0, 4.0).less(iv(3.0, 4.0)));

    assert!(iv(-2.0, -1.0).less(iv(-2.0, -1.0)));
    assert!(iv(-3.0, -1.5).less(iv(-2.0, -1.0)));
    // Signed-zero rows: all true.
    assert!(iv(0.0, 0.0).less(iv(-0.0, -0.0)));
    assert!(iv(-0.0, -0.0).less(iv(0.0, 0.0)));
    assert!(iv(-0.0, 0.0).less(iv(0.0, 0.0)));
    assert!(iv(-0.0, 0.0).less(iv(0.0, -0.0)));
    assert!(iv(0.0, -0.0).less(iv(0.0, 0.0)));
    assert!(iv(0.0, -0.0).less(iv(-0.0, 0.0)));
}

#[test]
fn strict_less_bare_vectors() {
    // minimal_strictly_less_test: 14 vectors (plus two non-corpus half-unbounded
    // rows at the end that exercise the matching-infinity clause)
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
    // minimal_precedes_test: 21 vectors
    assert!(Interval::<f64>::empty().precedes(iv(3.0, 4.0)));
    assert!(iv(3.0, 4.0).precedes(Interval::empty()));
    assert!(Interval::<f64>::empty().precedes(Interval::empty()));

    assert!(!iv(1.0, 2.0).precedes(Interval::entire()));
    assert!(!iv(0.0, 2.0).precedes(Interval::entire()));
    assert!(!iv(-0.0, 2.0).precedes(Interval::entire()));
    assert!(!Interval::<f64>::entire().precedes(iv(1.0, 2.0)));
    assert!(!Interval::<f64>::entire().precedes(Interval::entire()));

    assert!(iv(1.0, 2.0).precedes(iv(3.0, 4.0)));
    assert!(iv(1.0, 3.0).precedes(iv(3.0, 4.0))); // touching allowed
    assert!(iv(-3.0, -1.0).precedes(iv(-1.0, 0.0)));
    assert!(iv(-3.0, -1.0).precedes(iv(-1.0, -0.0)));
    assert!(!iv(1.0, 3.5).precedes(iv(3.0, 4.0)));
    assert!(!iv(1.0, 4.0).precedes(iv(3.0, 4.0)));
    assert!(!iv(-3.0, -0.1).precedes(iv(-1.0, 0.0)));
    // Signed-zero touch rows: sup <= inf holds because the zeros are equal.
    assert!(iv(0.0, 0.0).precedes(iv(-0.0, -0.0)));
    assert!(iv(-0.0, -0.0).precedes(iv(0.0, 0.0)));
    assert!(iv(-0.0, 0.0).precedes(iv(0.0, 0.0)));
    assert!(iv(-0.0, 0.0).precedes(iv(0.0, -0.0)));
    assert!(iv(0.0, -0.0).precedes(iv(0.0, 0.0)));
    assert!(iv(0.0, -0.0).precedes(iv(-0.0, 0.0)));
}

#[test]
fn strict_precedes_bare_vectors() {
    // minimal_strictly_precedes_test: 14 vectors
    assert!(Interval::<f64>::empty().strict_precedes(iv(3.0, 4.0)));
    assert!(iv(3.0, 4.0).strict_precedes(Interval::empty()));
    assert!(Interval::<f64>::empty().strict_precedes(Interval::empty()));

    assert!(!iv(1.0, 2.0).strict_precedes(Interval::entire()));
    assert!(!Interval::<f64>::entire().strict_precedes(iv(1.0, 2.0)));
    assert!(!Interval::<f64>::entire().strict_precedes(Interval::entire()));

    assert!(iv(1.0, 2.0).strict_precedes(iv(3.0, 4.0)));
    assert!(!iv(1.0, 3.0).strict_precedes(iv(3.0, 4.0))); // touch is not strict
    assert!(!iv(-3.0, -1.0).strict_precedes(iv(-1.0, 0.0)));
    // Signed-zero touch is not strict: sup -0.0 < inf 0.0 is false.
    assert!(!iv(-3.0, -0.0).strict_precedes(iv(0.0, 1.0)));
    assert!(!iv(-3.0, 0.0).strict_precedes(iv(-0.0, 1.0)));
    assert!(!iv(1.0, 3.5).strict_precedes(iv(3.0, 4.0)));
    assert!(!iv(1.0, 4.0).strict_precedes(iv(3.0, 4.0)));
    assert!(!iv(-3.0, -0.1).strict_precedes(iv(-1.0, 0.0)));
}

// --- The overlap relation -------------------------------------------------

#[test]
fn overlap_empty_states() {
    // minimal_overlap_test: the 3 empty-state vectors of 48 (the forward fn
    // carries 26, the reverse fn 19; 3 + 26 + 19 = 48).
    let e = Interval::<f64>::empty();
    assert_eq!(e.overlap(e), Overlap::BothEmpty);
    assert_eq!(e.overlap(iv(1.0, 2.0)), Overlap::FirstEmpty);
    assert_eq!(iv(1.0, 2.0).overlap(e), Overlap::SecondEmpty);
}

#[test]
fn overlap_forward_states() {
    // minimal_overlap_test: the 26 forward-state vectors of 48 (before through
    // equals; self at or left of other).
    assert_eq!(iv(NEG_INF, 2.0).overlap(iv(3.0, INF)), Overlap::Before);
    assert_eq!(iv(NEG_INF, 2.0).overlap(iv(3.0, 4.0)), Overlap::Before);
    assert_eq!(iv(2.0, 2.0).overlap(iv(3.0, 4.0)), Overlap::Before);
    assert_eq!(iv(1.0, 2.0).overlap(iv(3.0, 4.0)), Overlap::Before);
    assert_eq!(iv(1.0, 2.0).overlap(iv(3.0, 3.0)), Overlap::Before);
    assert_eq!(iv(2.0, 2.0).overlap(iv(3.0, 3.0)), Overlap::Before);
    assert_eq!(iv(2.0, 2.0).overlap(iv(3.0, INF)), Overlap::Before);

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
    assert_eq!(iv(1.0, 2.0).overlap(iv(NEG_INF, 3.0)), Overlap::ContainedBy);
    assert_eq!(iv(1.0, 2.0).overlap(iv(0.0, 3.0)), Overlap::ContainedBy);
    assert_eq!(iv(2.0, 2.0).overlap(iv(0.0, 3.0)), Overlap::ContainedBy);
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
    // minimal_overlap_test: the 19 mirror-state vectors of 48 (self at or right
    // of other).
    assert_eq!(iv(3.0, 4.0).overlap(iv(2.0, 2.0)), Overlap::After);
    assert_eq!(iv(3.0, 4.0).overlap(iv(1.0, 2.0)), Overlap::After);
    assert_eq!(iv(3.0, 3.0).overlap(iv(1.0, 2.0)), Overlap::After);
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
    assert_eq!(iv(0.0, 3.0).overlap(iv(1.0, 2.0)), Overlap::Contains);
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
// The complete 29-vector corpus table; the length is the corpus's, not logic's.
#[allow(clippy::too_many_lines)]
fn decorated_overlap_reads_interval_part() {
    // minimal_overlap_dec_test: 29 vectors. The decoration is ignored; the
    // interval part decides.
    assert_eq!(
        di_empty(Decoration::Trv).overlap(di_empty(Decoration::Trv)),
        Overlap::BothEmpty
    );
    assert_eq!(
        di_empty(Decoration::Trv).overlap(di(1.0, 2.0, Decoration::Com)),
        Overlap::FirstEmpty
    );
    assert_eq!(
        di(1.0, 2.0, Decoration::Def).overlap(di_empty(Decoration::Trv)),
        Overlap::SecondEmpty
    );

    assert_eq!(
        di(2.0, 2.0, Decoration::Def).overlap(di(3.0, 4.0, Decoration::Def)),
        Overlap::Before
    );
    assert_eq!(
        di(1.0, 2.0, Decoration::Dac).overlap(di(3.0, 4.0, Decoration::Com)),
        Overlap::Before
    );
    assert_eq!(
        di(1.0, 2.0, Decoration::Com).overlap(di(3.0, 3.0, Decoration::Trv)),
        Overlap::Before
    );
    assert_eq!(
        di(2.0, 2.0, Decoration::Trv).overlap(di(3.0, 3.0, Decoration::Def)),
        Overlap::Before
    );

    assert_eq!(
        di(1.0, 2.0, Decoration::Def).overlap(di(2.0, 3.0, Decoration::Def)),
        Overlap::Meets
    );

    assert_eq!(
        di(1.0, 2.0, Decoration::Dac).overlap(di(1.5, 2.5, Decoration::Def)),
        Overlap::Overlaps
    );

    assert_eq!(
        di(1.0, 2.0, Decoration::Def).overlap(di(1.0, 3.0, Decoration::Com)),
        Overlap::Starts
    );
    assert_eq!(
        di(1.0, 1.0, Decoration::Trv).overlap(di(1.0, 3.0, Decoration::Def)),
        Overlap::Starts
    );

    assert_eq!(
        di(1.0, 2.0, Decoration::Def).overlap(di(0.0, 3.0, Decoration::Dac)),
        Overlap::ContainedBy
    );
    assert_eq!(
        di(2.0, 2.0, Decoration::Trv).overlap(di(0.0, 3.0, Decoration::Def)),
        Overlap::ContainedBy
    );

    assert_eq!(
        di(1.0, 2.0, Decoration::Trv).overlap(di(0.0, 2.0, Decoration::Com)),
        Overlap::Finishes
    );
    assert_eq!(
        di(2.0, 2.0, Decoration::Def).overlap(di(0.0, 2.0, Decoration::Dac)),
        Overlap::Finishes
    );

    assert_eq!(
        di(1.0, 2.0, Decoration::Def).overlap(di(1.0, 2.0, Decoration::Def)),
        Overlap::Equals
    );
    assert_eq!(
        di(1.0, 1.0, Decoration::Dac).overlap(di(1.0, 1.0, Decoration::Dac)),
        Overlap::Equals
    );

    assert_eq!(
        di(3.0, 4.0, Decoration::Trv).overlap(di(2.0, 2.0, Decoration::Trv)),
        Overlap::After
    );
    assert_eq!(
        di(3.0, 4.0, Decoration::Def).overlap(di(1.0, 2.0, Decoration::Def)),
        Overlap::After
    );
    assert_eq!(
        di(3.0, 3.0, Decoration::Com).overlap(di(1.0, 2.0, Decoration::Dac)),
        Overlap::After
    );
    assert_eq!(
        di(3.0, 3.0, Decoration::Def).overlap(di(2.0, 2.0, Decoration::Trv)),
        Overlap::After
    );

    assert_eq!(
        di(2.0, 3.0, Decoration::Def).overlap(di(1.0, 2.0, Decoration::Trv)),
        Overlap::MetBy
    );

    assert_eq!(
        di(1.5, 2.5, Decoration::Com).overlap(di(1.0, 2.0, Decoration::Com)),
        Overlap::OverlappedBy
    );

    assert_eq!(
        di(1.0, 3.0, Decoration::Dac).overlap(di(1.0, 2.0, Decoration::Def)),
        Overlap::StartedBy
    );
    assert_eq!(
        di(1.0, 3.0, Decoration::Com).overlap(di(1.0, 1.0, Decoration::Dac)),
        Overlap::StartedBy
    );

    assert_eq!(
        di(0.0, 3.0, Decoration::Com).overlap(di(1.0, 2.0, Decoration::Dac)),
        Overlap::Contains
    );
    assert_eq!(
        di(0.0, 3.0, Decoration::Com).overlap(di(2.0, 2.0, Decoration::Def)),
        Overlap::Contains
    );

    assert_eq!(
        di(0.0, 2.0, Decoration::Def).overlap(di(1.0, 2.0, Decoration::Trv)),
        Overlap::FinishedBy
    );
    assert_eq!(
        di(0.0, 2.0, Decoration::Dac).overlap(di(2.0, 2.0, Decoration::Def)),
        Overlap::FinishedBy
    );

    // No NaI case is pinned by the vectors; a NaI reads through as its empty
    // interval part, so two NaIs report bothEmpty (non-corpus sanity check).
    assert_eq!(nai().overlap(nai()), Overlap::BothEmpty);
}

// --- Numeric: inf / sup (exact endpoint reads) ----------------------------

#[test]
fn inf_bare_vectors() {
    // minimal_inf_test: 14 vectors. `inf` reads an endpoint and never rounds, so
    // it is pinned exactly. The corpus writes `-0.0` for the lower endpoint of
    // several intervals; the assertion is at the Level 2 value, where
    // `-0.0 == +0.0`, so the endpoint's sign of zero is not separately pinned
    // (the fixture returns `+0.0` for `inf([0, +inf])`, a harmless difference).
    assert_eq!(Interval::<f64>::empty().inf(), INF);
    assert_eq!(Interval::<f64>::entire().inf(), NEG_INF);
    assert_eq!(iv(1.0, 2.0).inf(), 1.0);
    assert_eq!(iv(-3.0, -2.0).inf(), -3.0);
    assert_eq!(iv(NEG_INF, 2.0).inf(), NEG_INF);
    assert_eq!(iv(NEG_INF, 0.0).inf(), NEG_INF);
    assert_eq!(iv(NEG_INF, -0.0).inf(), NEG_INF);
    assert_eq!(iv(-2.0, INF).inf(), -2.0);
    assert_eq!(iv(0.0, INF).inf(), -0.0);
    assert_eq!(iv(-0.0, INF).inf(), -0.0);
    assert_eq!(iv(-0.0, 0.0).inf(), -0.0);
    assert_eq!(iv(0.0, -0.0).inf(), -0.0);
    assert_eq!(iv(0.0, 0.0).inf(), -0.0);
    assert_eq!(iv(-0.0, -0.0).inf(), -0.0);
}

#[test]
fn sup_bare_vectors() {
    // minimal_sup_test: 14 vectors. Exact, same signed-zero caveat as `inf`.
    assert_eq!(Interval::<f64>::empty().sup(), NEG_INF);
    assert_eq!(Interval::<f64>::entire().sup(), INF);
    assert_eq!(iv(1.0, 2.0).sup(), 2.0);
    assert_eq!(iv(-3.0, -2.0).sup(), -2.0);
    assert_eq!(iv(NEG_INF, 2.0).sup(), 2.0);
    assert_eq!(iv(NEG_INF, 0.0).sup(), 0.0);
    assert_eq!(iv(NEG_INF, -0.0).sup(), 0.0);
    assert_eq!(iv(-2.0, INF).sup(), INF);
    assert_eq!(iv(0.0, INF).sup(), INF);
    assert_eq!(iv(-0.0, INF).sup(), INF);
    assert_eq!(iv(-0.0, 0.0).sup(), 0.0);
    assert_eq!(iv(0.0, -0.0).sup(), 0.0);
    assert_eq!(iv(0.0, 0.0).sup(), 0.0);
    assert_eq!(iv(-0.0, -0.0).sup(), 0.0);
}

// --- Numeric: wid (sound-not-tight for finite widths on this fixture) ------

/// The fixture computes `wid = (sup - inf).next_up()`, one ulp wide even when the
/// exact difference is representable. The corpus datum is the tightest correctly
/// rounded width; the fixture value is asserted as a sound one-ulp over-estimate.
fn wid_one_ulp_sound(x: Interval<f64>, tightest: f64) {
    let w = x.wid().unwrap();
    assert!(
        tightest <= w && w <= tightest.next_up(),
        "wid {w} is not a one-ulp-sound over-estimate of the corpus width {tightest}"
    );
}

#[test]
fn wid_bare_vectors() {
    // minimal_wid_test: 8 vectors. The empty (`None`) and unbounded (`+inf`)
    // widths are exact; the finite widths are one-ulp-sound over the corpus datum.
    wid_one_ulp_sound(iv(2.0, 2.0), 0.0);
    wid_one_ulp_sound(iv(1.0, 2.0), 1.0);
    assert_eq!(iv(1.0, INF).wid(), Some(INF));
    assert_eq!(iv(NEG_INF, 2.0).wid(), Some(INF));
    assert_eq!(Interval::<f64>::entire().wid(), Some(INF));
    assert_eq!(Interval::<f64>::empty().wid(), None);
    wid_one_ulp_sound(iv(hx("0X1P+0"), hx("0X1.0000000000001P+0")), hx("0X1P-52"));
    wid_one_ulp_sound(
        iv(hx("0X1P-1022"), hx("0X1.0000000000001P-1022")),
        hx("0X0.0000000000001P-1022"),
    );
}

// --- Numeric: mag / mig (exact magnitude reads) ---------------------------

#[test]
fn mag_bare_vectors() {
    // minimal_mag_test: 8 vectors. `mag` is a max of absolute endpoints: no
    // rounding, pinned exactly; empty is `None` (the corpus NaN).
    assert_eq!(iv(1.0, 2.0).mag(), Some(2.0));
    assert_eq!(iv(-4.0, 2.0).mag(), Some(4.0));
    assert_eq!(iv(NEG_INF, 2.0).mag(), Some(INF));
    assert_eq!(iv(1.0, INF).mag(), Some(INF));
    assert_eq!(Interval::<f64>::entire().mag(), Some(INF));
    assert_eq!(Interval::<f64>::empty().mag(), None);
    assert_eq!(iv(-0.0, 0.0).mag(), Some(0.0));
    assert_eq!(iv(-0.0, -0.0).mag(), Some(0.0));
}

#[test]
fn mig_bare_vectors() {
    // minimal_mig_test: 11 vectors. `mig` is a min of absolute endpoints (zero
    // when the interval straddles zero): no rounding, pinned exactly.
    assert_eq!(iv(1.0, 2.0).mig(), Some(1.0));
    assert_eq!(iv(-4.0, 2.0).mig(), Some(0.0));
    assert_eq!(iv(-4.0, -2.0).mig(), Some(2.0));
    assert_eq!(iv(NEG_INF, 2.0).mig(), Some(0.0));
    assert_eq!(iv(NEG_INF, -2.0).mig(), Some(2.0));
    assert_eq!(iv(-1.0, INF).mig(), Some(0.0));
    assert_eq!(iv(1.0, INF).mig(), Some(1.0));
    assert_eq!(Interval::<f64>::entire().mig(), Some(0.0));
    assert_eq!(Interval::<f64>::empty().mig(), None);
    assert_eq!(iv(-0.0, 0.0).mig(), Some(0.0));
    assert_eq!(iv(-0.0, -0.0).mig(), Some(0.0));
}

// --- Boolean predicates: isEmpty / isEntire (bare) ------------------------

#[test]
fn is_empty_bare_vectors() {
    // minimal_is_empty_test: 14 vectors
    assert!(Interval::<f64>::empty().is_empty());
    assert!(!Interval::<f64>::entire().is_empty());
    assert!(!iv(1.0, 2.0).is_empty());
    assert!(!iv(-1.0, 2.0).is_empty());
    assert!(!iv(-3.0, -2.0).is_empty());
    assert!(!iv(NEG_INF, 2.0).is_empty());
    assert!(!iv(NEG_INF, 0.0).is_empty());
    assert!(!iv(NEG_INF, -0.0).is_empty());
    assert!(!iv(0.0, INF).is_empty());
    assert!(!iv(-0.0, INF).is_empty());
    assert!(!iv(-0.0, 0.0).is_empty());
    assert!(!iv(0.0, -0.0).is_empty());
    assert!(!iv(0.0, 0.0).is_empty());
    assert!(!iv(-0.0, -0.0).is_empty());
}

#[test]
fn is_entire_bare_vectors() {
    // minimal_is_entire_test: 14 vectors
    assert!(!Interval::<f64>::empty().is_entire());
    assert!(Interval::<f64>::entire().is_entire());
    assert!(!iv(1.0, 2.0).is_entire());
    assert!(!iv(-1.0, 2.0).is_entire());
    assert!(!iv(-3.0, -2.0).is_entire());
    assert!(!iv(NEG_INF, 2.0).is_entire());
    assert!(!iv(NEG_INF, 0.0).is_entire());
    assert!(!iv(NEG_INF, -0.0).is_entire());
    assert!(!iv(0.0, INF).is_entire());
    assert!(!iv(-0.0, INF).is_entire());
    assert!(!iv(-0.0, 0.0).is_entire());
    assert!(!iv(0.0, -0.0).is_entire());
    assert!(!iv(0.0, 0.0).is_entire());
    assert!(!iv(-0.0, -0.0).is_entire());
}

// --- Boolean relations: subset / interior / disjoint (bare) ---------------

#[test]
fn subset_bare_vectors() {
    // minimal_subset_test: 27 vectors
    assert!(Interval::<f64>::empty().subset(Interval::empty()));
    assert!(Interval::<f64>::empty().subset(iv(0.0, 4.0)));
    assert!(Interval::<f64>::empty().subset(iv(-0.0, 4.0)));
    assert!(Interval::<f64>::empty().subset(iv(-0.1, 1.0)));
    assert!(Interval::<f64>::empty().subset(iv(-0.1, 0.0)));
    assert!(Interval::<f64>::empty().subset(iv(-0.1, -0.0)));
    assert!(Interval::<f64>::empty().subset(Interval::entire()));

    assert!(!iv(0.0, 4.0).subset(Interval::empty()));
    assert!(!iv(-0.0, 4.0).subset(Interval::empty()));
    assert!(!iv(-0.1, 1.0).subset(Interval::empty()));
    assert!(!Interval::<f64>::entire().subset(Interval::empty()));

    assert!(iv(0.0, 4.0).subset(Interval::entire()));
    assert!(iv(-0.0, 4.0).subset(Interval::entire()));
    assert!(iv(-0.1, 1.0).subset(Interval::entire()));
    assert!(Interval::<f64>::entire().subset(Interval::entire()));

    assert!(iv(1.0, 2.0).subset(iv(1.0, 2.0)));
    assert!(iv(1.0, 2.0).subset(iv(0.0, 4.0)));
    assert!(iv(1.0, 2.0).subset(iv(-0.0, 4.0)));
    assert!(iv(0.1, 0.2).subset(iv(0.0, 4.0)));
    assert!(iv(0.1, 0.2).subset(iv(-0.0, 4.0)));
    assert!(iv(-0.1, -0.1).subset(iv(-4.0, 3.4)));

    assert!(iv(0.0, 0.0).subset(iv(-0.0, -0.0)));
    assert!(iv(-0.0, -0.0).subset(iv(0.0, 0.0)));
    assert!(iv(-0.0, 0.0).subset(iv(0.0, 0.0)));
    assert!(iv(-0.0, 0.0).subset(iv(0.0, -0.0)));
    assert!(iv(0.0, -0.0).subset(iv(0.0, 0.0)));
    assert!(iv(0.0, -0.0).subset(iv(-0.0, 0.0)));
}

#[test]
fn interior_bare_vectors() {
    // minimal_interior_test: 16 vectors
    assert!(Interval::<f64>::empty().interior(Interval::empty()));
    assert!(Interval::<f64>::empty().interior(iv(0.0, 4.0)));
    assert!(!iv(0.0, 4.0).interior(Interval::empty()));

    assert!(Interval::<f64>::entire().interior(Interval::entire()));
    assert!(iv(0.0, 4.0).interior(Interval::entire()));
    assert!(Interval::<f64>::empty().interior(Interval::entire()));
    assert!(!Interval::<f64>::entire().interior(iv(0.0, 4.0)));

    assert!(!iv(0.0, 4.0).interior(iv(0.0, 4.0)));
    assert!(iv(1.0, 2.0).interior(iv(0.0, 4.0)));
    assert!(!iv(-2.0, 2.0).interior(iv(-2.0, 4.0)));
    assert!(iv(-0.0, -0.0).interior(iv(-2.0, 4.0)));
    assert!(iv(0.0, 0.0).interior(iv(-2.0, 4.0)));
    assert!(!iv(0.0, 0.0).interior(iv(-0.0, -0.0)));

    assert!(!iv(0.0, 4.4).interior(iv(0.0, 4.0)));
    assert!(!iv(-1.0, -1.0).interior(iv(0.0, 4.0)));
    assert!(!iv(2.0, 2.0).interior(iv(-2.0, -1.0)));
}

#[test]
fn disjoint_bare_vectors() {
    // minimal_disjoint_test: 10 vectors
    assert!(Interval::<f64>::empty().disjoint(iv(3.0, 4.0)));
    assert!(iv(3.0, 4.0).disjoint(Interval::empty()));
    assert!(Interval::<f64>::empty().disjoint(Interval::empty()));

    assert!(iv(3.0, 4.0).disjoint(iv(1.0, 2.0)));

    assert!(!iv(0.0, 0.0).disjoint(iv(-0.0, -0.0)));
    assert!(!iv(0.0, -0.0).disjoint(iv(-0.0, 0.0)));
    assert!(!iv(3.0, 4.0).disjoint(iv(1.0, 7.0)));
    assert!(!iv(3.0, 4.0).disjoint(Interval::entire()));
    assert!(!Interval::<f64>::entire().disjoint(iv(1.0, 7.0)));
    assert!(!Interval::<f64>::entire().disjoint(Interval::entire()));
}

// --- Decorated twins: isNaI and the relational operations ------------------
//
// The decoration is ignored; the interval part decides, with the one rule the
// corpus pins: a NaI operand makes every relation false. Each testcase is the
// complete `_dec` table, transcribed against the crate's decorated operation.

#[test]
fn is_nai_dec_vectors() {
    // minimal_is_nai_dec_test: 16 vectors
    assert!(nai().is_nai());
    assert!(!di(NEG_INF, INF, Decoration::Trv).is_nai());
    assert!(!di(NEG_INF, INF, Decoration::Def).is_nai());
    assert!(!di(NEG_INF, INF, Decoration::Dac).is_nai());
    assert!(!di(1.0, 2.0, Decoration::Com).is_nai());
    assert!(!di(-1.0, 2.0, Decoration::Trv).is_nai());
    assert!(!di(-3.0, -2.0, Decoration::Dac).is_nai());
    assert!(!di(NEG_INF, 2.0, Decoration::Trv).is_nai());
    assert!(!di(NEG_INF, 0.0, Decoration::Trv).is_nai());
    assert!(!di(NEG_INF, -0.0, Decoration::Trv).is_nai());
    assert!(!di(0.0, INF, Decoration::Def).is_nai());
    assert!(!di(-0.0, INF, Decoration::Trv).is_nai());
    assert!(!di(-0.0, 0.0, Decoration::Com).is_nai());
    assert!(!di(0.0, -0.0, Decoration::Trv).is_nai());
    assert!(!di(0.0, 0.0, Decoration::Trv).is_nai());
    assert!(!di(-0.0, -0.0, Decoration::Trv).is_nai());
}

#[test]
fn equal_dec_vectors() {
    // minimal_equal_dec_test: 19 vectors
    assert!(di(1.0, 2.0, Decoration::Def).equal(di(1.0, 2.0, Decoration::Trv)));
    assert!(!di(1.0, 2.1, Decoration::Trv).equal(di(1.0, 2.0, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).equal(di_empty(Decoration::Trv)));
    assert!(!di_empty(Decoration::Trv).equal(nai()));
    assert!(!nai().equal(di_empty(Decoration::Trv)));
    assert!(!nai().equal(nai()));
    assert!(!di_empty(Decoration::Trv).equal(di(1.0, 2.0, Decoration::Trv)));
    assert!(!nai().equal(di(1.0, 2.0, Decoration::Trv)));
    assert!(di(NEG_INF, INF, Decoration::Def).equal(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(1.0, 2.4, Decoration::Trv).equal(di(NEG_INF, INF, Decoration::Trv)));
    assert!(di(1.0, INF, Decoration::Trv).equal(di(1.0, INF, Decoration::Trv)));
    assert!(!di(1.0, 2.4, Decoration::Def).equal(di(1.0, INF, Decoration::Trv)));
    assert!(di(NEG_INF, 2.0, Decoration::Trv).equal(di(NEG_INF, 2.0, Decoration::Trv)));
    assert!(!di(NEG_INF, 2.4, Decoration::Def).equal(di(NEG_INF, 2.0, Decoration::Trv)));
    assert!(di(-2.0, 0.0, Decoration::Trv).equal(di(-2.0, 0.0, Decoration::Trv)));
    assert!(di(-0.0, 2.0, Decoration::Def).equal(di(0.0, 2.0, Decoration::Trv)));
    assert!(di(-0.0, -0.0, Decoration::Trv).equal(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(-0.0, 0.0, Decoration::Def).equal(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(0.0, -0.0, Decoration::Trv).equal(di(0.0, 0.0, Decoration::Trv)));
}

#[test]
fn subset_dec_vectors() {
    // minimal_subset_dec_test: 29 vectors
    assert!(!nai().subset(nai()));
    assert!(!nai().subset(di_empty(Decoration::Trv)));
    assert!(!nai().subset(di(0.0, 4.0, Decoration::Trv)));

    assert!(di_empty(Decoration::Trv).subset(di(0.0, 4.0, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).subset(di(-0.0, 4.0, Decoration::Def)));
    assert!(di_empty(Decoration::Trv).subset(di(-0.1, 1.0, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).subset(di(-0.1, 0.0, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).subset(di(-0.1, -0.0, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).subset(di(NEG_INF, INF, Decoration::Trv)));

    assert!(!di(0.0, 4.0, Decoration::Trv).subset(di_empty(Decoration::Trv)));
    assert!(!di(-0.0, 4.0, Decoration::Def).subset(di_empty(Decoration::Trv)));
    assert!(!di(-0.1, 1.0, Decoration::Trv).subset(di_empty(Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).subset(di_empty(Decoration::Trv)));

    assert!(di(0.0, 4.0, Decoration::Trv).subset(di(NEG_INF, INF, Decoration::Trv)));
    assert!(di(-0.0, 4.0, Decoration::Trv).subset(di(NEG_INF, INF, Decoration::Trv)));
    assert!(di(-0.1, 1.0, Decoration::Trv).subset(di(NEG_INF, INF, Decoration::Trv)));
    assert!(di(NEG_INF, INF, Decoration::Trv).subset(di(NEG_INF, INF, Decoration::Trv)));

    assert!(di(1.0, 2.0, Decoration::Trv).subset(di(1.0, 2.0, Decoration::Trv)));
    assert!(di(1.0, 2.0, Decoration::Trv).subset(di(0.0, 4.0, Decoration::Trv)));
    assert!(di(1.0, 2.0, Decoration::Def).subset(di(-0.0, 4.0, Decoration::Def)));
    assert!(di(0.1, 0.2, Decoration::Trv).subset(di(0.0, 4.0, Decoration::Trv)));
    assert!(di(0.1, 0.2, Decoration::Trv).subset(di(-0.0, 4.0, Decoration::Def)));
    assert!(di(-0.1, -0.1, Decoration::Trv).subset(di(-4.0, 3.4, Decoration::Trv)));

    assert!(di(0.0, 0.0, Decoration::Trv).subset(di(-0.0, -0.0, Decoration::Trv)));
    assert!(di(-0.0, -0.0, Decoration::Trv).subset(di(0.0, 0.0, Decoration::Def)));
    assert!(di(-0.0, 0.0, Decoration::Trv).subset(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(-0.0, 0.0, Decoration::Trv).subset(di(0.0, -0.0, Decoration::Trv)));
    assert!(di(0.0, -0.0, Decoration::Def).subset(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(0.0, -0.0, Decoration::Trv).subset(di(-0.0, 0.0, Decoration::Trv)));
}

#[test]
fn less_dec_vectors() {
    // minimal_less_dec_test: 30 vectors
    assert!(!nai().less(nai()));
    assert!(!di_empty(Decoration::Trv).less(nai()));
    assert!(!di(1.0, 2.0, Decoration::Trv).less(nai()));
    assert!(!nai().less(di(1.0, 2.0, Decoration::Def)));

    assert!(di_empty(Decoration::Trv).less(di_empty(Decoration::Trv)));
    assert!(!di(1.0, 2.0, Decoration::Trv).less(di_empty(Decoration::Trv)));
    assert!(!di_empty(Decoration::Trv).less(di(1.0, 2.0, Decoration::Trv)));

    assert!(di(NEG_INF, INF, Decoration::Trv).less(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(1.0, 2.0, Decoration::Def).less(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(0.0, 2.0, Decoration::Trv).less(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(-0.0, 2.0, Decoration::Trv).less(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).less(di(1.0, 2.0, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).less(di(0.0, 2.0, Decoration::Def)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).less(di(-0.0, 2.0, Decoration::Trv)));

    assert!(di(0.0, 2.0, Decoration::Trv).less(di(0.0, 2.0, Decoration::Trv)));
    assert!(di(0.0, 2.0, Decoration::Trv).less(di(-0.0, 2.0, Decoration::Trv)));
    assert!(di(0.0, 2.0, Decoration::Def).less(di(1.0, 2.0, Decoration::Def)));
    assert!(di(-0.0, 2.0, Decoration::Trv).less(di(1.0, 2.0, Decoration::Trv)));
    assert!(di(1.0, 2.0, Decoration::Trv).less(di(1.0, 2.0, Decoration::Trv)));
    assert!(di(1.0, 2.0, Decoration::Trv).less(di(3.0, 4.0, Decoration::Def)));
    assert!(di(1.0, 3.5, Decoration::Trv).less(di(3.0, 4.0, Decoration::Trv)));
    assert!(di(1.0, 4.0, Decoration::Trv).less(di(3.0, 4.0, Decoration::Trv)));

    assert!(di(-2.0, -1.0, Decoration::Trv).less(di(-2.0, -1.0, Decoration::Trv)));
    assert!(di(-3.0, -1.5, Decoration::Trv).less(di(-2.0, -1.0, Decoration::Trv)));

    assert!(di(0.0, 0.0, Decoration::Trv).less(di(-0.0, -0.0, Decoration::Trv)));
    assert!(di(-0.0, -0.0, Decoration::Trv).less(di(0.0, 0.0, Decoration::Def)));
    assert!(di(-0.0, 0.0, Decoration::Trv).less(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(-0.0, 0.0, Decoration::Trv).less(di(0.0, -0.0, Decoration::Trv)));
    assert!(di(0.0, -0.0, Decoration::Def).less(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(0.0, -0.0, Decoration::Trv).less(di(-0.0, 0.0, Decoration::Trv)));
}

#[test]
fn precedes_dec_vectors() {
    // minimal_precedes_dec_test: 25 vectors
    assert!(!nai().precedes(di(3.0, 4.0, Decoration::Def)));
    assert!(!di(3.0, 4.0, Decoration::Trv).precedes(nai()));
    assert!(!nai().precedes(di_empty(Decoration::Trv)));
    assert!(!nai().precedes(nai()));

    assert!(di_empty(Decoration::Trv).precedes(di(3.0, 4.0, Decoration::Def)));
    assert!(di(3.0, 4.0, Decoration::Trv).precedes(di_empty(Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).precedes(di_empty(Decoration::Trv)));

    assert!(!di(1.0, 2.0, Decoration::Trv).precedes(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(0.0, 2.0, Decoration::Trv).precedes(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(-0.0, 2.0, Decoration::Trv).precedes(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).precedes(di(1.0, 2.0, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).precedes(di(NEG_INF, INF, Decoration::Trv)));

    assert!(di(1.0, 2.0, Decoration::Trv).precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(di(1.0, 3.0, Decoration::Trv).precedes(di(3.0, 4.0, Decoration::Def)));
    assert!(di(-3.0, -1.0, Decoration::Def).precedes(di(-1.0, 0.0, Decoration::Trv)));
    assert!(di(-3.0, -1.0, Decoration::Trv).precedes(di(-1.0, -0.0, Decoration::Trv)));

    assert!(!di(1.0, 3.5, Decoration::Trv).precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(!di(1.0, 4.0, Decoration::Trv).precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(!di(-3.0, -0.1, Decoration::Trv).precedes(di(-1.0, 0.0, Decoration::Trv)));

    assert!(di(0.0, 0.0, Decoration::Trv).precedes(di(-0.0, -0.0, Decoration::Trv)));
    assert!(di(-0.0, -0.0, Decoration::Trv).precedes(di(0.0, 0.0, Decoration::Def)));
    assert!(di(-0.0, 0.0, Decoration::Trv).precedes(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(-0.0, 0.0, Decoration::Def).precedes(di(0.0, -0.0, Decoration::Trv)));
    assert!(di(0.0, -0.0, Decoration::Trv).precedes(di(0.0, 0.0, Decoration::Trv)));
    assert!(di(0.0, -0.0, Decoration::Trv).precedes(di(-0.0, 0.0, Decoration::Trv)));
}

#[test]
fn interior_dec_vectors() {
    // minimal_interior_dec_test: 20 vectors
    assert!(!nai().interior(nai()));
    assert!(!nai().interior(di_empty(Decoration::Trv)));
    assert!(!nai().interior(di(0.0, 4.0, Decoration::Trv)));
    assert!(!di(0.0, 4.0, Decoration::Def).interior(nai()));

    assert!(di_empty(Decoration::Trv).interior(di_empty(Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).interior(di(0.0, 4.0, Decoration::Trv)));
    assert!(!di(0.0, 4.0, Decoration::Def).interior(di_empty(Decoration::Trv)));

    assert!(di(NEG_INF, INF, Decoration::Trv).interior(di(NEG_INF, INF, Decoration::Trv)));
    assert!(di(0.0, 4.0, Decoration::Trv).interior(di(NEG_INF, INF, Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).interior(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).interior(di(0.0, 4.0, Decoration::Trv)));

    assert!(!di(0.0, 4.0, Decoration::Trv).interior(di(0.0, 4.0, Decoration::Trv)));
    assert!(di(1.0, 2.0, Decoration::Def).interior(di(0.0, 4.0, Decoration::Trv)));
    assert!(!di(-2.0, 2.0, Decoration::Trv).interior(di(-2.0, 4.0, Decoration::Def)));
    assert!(di(-0.0, -0.0, Decoration::Trv).interior(di(-2.0, 4.0, Decoration::Trv)));
    assert!(di(0.0, 0.0, Decoration::Def).interior(di(-2.0, 4.0, Decoration::Trv)));
    assert!(!di(0.0, 0.0, Decoration::Trv).interior(di(-0.0, -0.0, Decoration::Trv)));

    assert!(!di(0.0, 4.4, Decoration::Trv).interior(di(0.0, 4.0, Decoration::Trv)));
    assert!(!di(-1.0, -1.0, Decoration::Trv).interior(di(0.0, 4.0, Decoration::Def)));
    assert!(!di(2.0, 2.0, Decoration::Def).interior(di(-2.0, -1.0, Decoration::Trv)));
}

#[test]
fn strict_less_dec_vectors() {
    // minimal_strictly_less_dec_test: 18 vectors
    assert!(!nai().strict_less(nai()));
    assert!(!di_empty(Decoration::Trv).strict_less(nai()));
    assert!(!di(1.0, 2.0, Decoration::Trv).strict_less(nai()));
    assert!(!nai().strict_less(di(1.0, 2.0, Decoration::Def)));

    assert!(di_empty(Decoration::Trv).strict_less(di_empty(Decoration::Trv)));
    assert!(!di(1.0, 2.0, Decoration::Trv).strict_less(di_empty(Decoration::Trv)));
    assert!(!di_empty(Decoration::Trv).strict_less(di(1.0, 2.0, Decoration::Def)));

    assert!(di(NEG_INF, INF, Decoration::Trv).strict_less(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(1.0, 2.0, Decoration::Trv).strict_less(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).strict_less(di(1.0, 2.0, Decoration::Trv)));

    assert!(!di(1.0, 2.0, Decoration::Trv).strict_less(di(1.0, 2.0, Decoration::Trv)));
    assert!(di(1.0, 2.0, Decoration::Trv).strict_less(di(3.0, 4.0, Decoration::Trv)));
    assert!(di(1.0, 3.5, Decoration::Def).strict_less(di(3.0, 4.0, Decoration::Trv)));
    assert!(!di(1.0, 4.0, Decoration::Trv).strict_less(di(3.0, 4.0, Decoration::Def)));
    assert!(!di(0.0, 4.0, Decoration::Trv).strict_less(di(0.0, 4.0, Decoration::Def)));
    assert!(!di(-0.0, 4.0, Decoration::Def).strict_less(di(0.0, 4.0, Decoration::Trv)));

    assert!(!di(-2.0, -1.0, Decoration::Def).strict_less(di(-2.0, -1.0, Decoration::Def)));
    assert!(di(-3.0, -1.5, Decoration::Trv).strict_less(di(-2.0, -1.0, Decoration::Trv)));
}

#[test]
fn strict_precedes_dec_vectors() {
    // minimal_strictly_precedes_dec_test: 18 vectors
    assert!(!nai().strict_precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(!di(3.0, 4.0, Decoration::Def).strict_precedes(nai()));
    assert!(!nai().strict_precedes(di_empty(Decoration::Trv)));
    assert!(!nai().strict_precedes(nai()));

    assert!(di_empty(Decoration::Trv).strict_precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(di(3.0, 4.0, Decoration::Def).strict_precedes(di_empty(Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).strict_precedes(di_empty(Decoration::Trv)));

    assert!(!di(1.0, 2.0, Decoration::Trv).strict_precedes(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).strict_precedes(di(1.0, 2.0, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).strict_precedes(di(NEG_INF, INF, Decoration::Trv)));

    assert!(di(1.0, 2.0, Decoration::Trv).strict_precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(!di(1.0, 3.0, Decoration::Def).strict_precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(!di(-3.0, -1.0, Decoration::Trv).strict_precedes(di(-1.0, 0.0, Decoration::Def)));
    assert!(!di(-3.0, -0.0, Decoration::Def).strict_precedes(di(0.0, 1.0, Decoration::Trv)));
    assert!(!di(-3.0, 0.0, Decoration::Trv).strict_precedes(di(-0.0, 1.0, Decoration::Trv)));

    assert!(!di(1.0, 3.5, Decoration::Trv).strict_precedes(di(3.0, 4.0, Decoration::Trv)));
    assert!(!di(1.0, 4.0, Decoration::Trv).strict_precedes(di(3.0, 4.0, Decoration::Def)));
    assert!(!di(-3.0, -0.1, Decoration::Trv).strict_precedes(di(-1.0, 0.0, Decoration::Trv)));
}

#[test]
fn disjoint_dec_vectors() {
    // minimal_disjoint_dec_test: 14 vectors
    assert!(!nai().disjoint(di(3.0, 4.0, Decoration::Def)));
    assert!(!di(3.0, 4.0, Decoration::Trv).disjoint(nai()));
    assert!(!di_empty(Decoration::Trv).disjoint(nai()));
    assert!(!nai().disjoint(nai()));

    assert!(di_empty(Decoration::Trv).disjoint(di(3.0, 4.0, Decoration::Def)));
    assert!(di(3.0, 4.0, Decoration::Trv).disjoint(di_empty(Decoration::Trv)));
    assert!(di_empty(Decoration::Trv).disjoint(di_empty(Decoration::Trv)));

    assert!(di(3.0, 4.0, Decoration::Trv).disjoint(di(1.0, 2.0, Decoration::Def)));

    assert!(!di(0.0, 0.0, Decoration::Trv).disjoint(di(-0.0, -0.0, Decoration::Trv)));
    assert!(!di(0.0, -0.0, Decoration::Trv).disjoint(di(-0.0, 0.0, Decoration::Trv)));
    assert!(!di(3.0, 4.0, Decoration::Def).disjoint(di(1.0, 7.0, Decoration::Def)));
    assert!(!di(3.0, 4.0, Decoration::Trv).disjoint(di(NEG_INF, INF, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).disjoint(di(1.0, 7.0, Decoration::Trv)));
    assert!(!di(NEG_INF, INF, Decoration::Trv).disjoint(di(NEG_INF, INF, Decoration::Trv)));
}

// --- Recommended booleans: isSingleton / isCommonInterval / isMember -------

#[test]
fn is_singleton_bare_vectors() {
    // minimal_is_singleton_test: 15 vectors
    assert!(iv(-27.0, -27.0).is_singleton());
    assert!(iv(-2.0, -2.0).is_singleton());
    assert!(iv(12.0, 12.0).is_singleton());
    assert!(iv(17.1, 17.1).is_singleton());
    assert!(iv(-0.0, -0.0).is_singleton());
    assert!(iv(0.0, 0.0).is_singleton());
    assert!(iv(-0.0, 0.0).is_singleton());
    assert!(iv(0.0, -0.0).is_singleton());

    assert!(!Interval::<f64>::empty().is_singleton());
    assert!(!Interval::<f64>::entire().is_singleton());
    assert!(!iv(-1.0, 0.0).is_singleton());
    assert!(!iv(-1.0, -0.5).is_singleton());
    assert!(!iv(1.0, 2.0).is_singleton());
    assert!(!iv(NEG_INF, -MAX).is_singleton()); // [-inf, -0x1.FFFFFFFFFFFFFp1023]
    assert!(!iv(-1.0, INF).is_singleton());
}

#[test]
fn is_common_interval_via_bounded_nonempty() {
    // minimal_is_common_interval_test: 12 vectors.
    // The crate surfaces no `is_common_interval`; the standard's predicate is
    // "nonempty and bounded", checked here through the named `is_empty`/
    // `is_bounded` (a documented surface gap; see the KNOWN GAP block). This
    // exercises `is_bounded`'s boundary behavior at entire / half-unbounded /
    // realmax, which the named predicate would delegate to.
    let common = |x: Interval<f64>| !x.is_empty() && x.is_bounded();
    assert!(common(iv(-27.0, -27.0)));
    assert!(common(iv(-27.0, 0.0)));
    assert!(common(iv(0.0, 0.0)));
    assert!(common(iv(-0.0, -0.0)));
    assert!(common(iv(-0.0, 0.0)));
    assert!(common(iv(0.0, -0.0)));
    assert!(common(iv(5.0, 12.4)));
    assert!(common(iv(-MAX, MAX))); // [-0x1.F..Fp1023, 0x1.F..Fp1023]

    assert!(!common(Interval::<f64>::entire()));
    assert!(!common(Interval::<f64>::empty()));
    assert!(!common(iv(NEG_INF, 0.0)));
    assert!(!common(iv(0.0, INF)));
}

#[test]
fn is_member_via_finite_contains() {
    // minimal_is_member_test: 35 vectors.
    // The crate surfaces no `is_member`; the standard's predicate is "m is a
    // finite real number lying in x". `Interval::contains` alone diverges here,
    // since it admits an infinite argument at an infinite endpoint
    // (`entire.contains(+inf) == true`), so the standard's finiteness guard is
    // applied explicitly (a documented surface gap; see the KNOWN GAP block).
    let member = |m: f64, x: Interval<f64>| m.is_finite() && x.contains(m);
    let max = hx("0x1.FFFFFFFFFFFFFp1023");
    let min_normal = hx("0x1.0p-1022");

    assert!(member(-27.0, iv(-27.0, -27.0)));
    assert!(member(-27.0, iv(-27.0, 0.0)));
    assert!(member(-7.0, iv(-27.0, 0.0)));
    assert!(member(0.0, iv(-27.0, 0.0)));
    assert!(member(-0.0, iv(0.0, 0.0)));
    assert!(member(0.0, iv(0.0, 0.0)));
    assert!(member(0.0, iv(-0.0, -0.0)));
    assert!(member(0.0, iv(-0.0, 0.0)));
    assert!(member(0.0, iv(0.0, -0.0)));
    assert!(member(5.0, iv(5.0, 12.4)));
    assert!(member(6.3, iv(5.0, 12.4)));
    assert!(member(12.4, iv(5.0, 12.4)));
    assert!(member(0.0, Interval::entire()));
    assert!(member(5.0, Interval::entire()));
    assert!(member(6.3, Interval::entire()));
    assert!(member(12.4, Interval::entire()));
    assert!(member(max, Interval::entire()));
    assert!(member(-max, Interval::entire()));
    assert!(member(min_normal, Interval::entire()));
    assert!(member(-min_normal, Interval::entire()));

    assert!(!member(-71.0, iv(-27.0, 0.0)));
    assert!(!member(0.1, iv(-27.0, 0.0)));
    assert!(!member(-0.01, iv(0.0, 0.0)));
    assert!(!member(0.000_001, iv(0.0, 0.0)));
    assert!(!member(111_110.0, iv(-0.0, -0.0)));
    assert!(!member(4.9, iv(5.0, 12.4)));
    assert!(!member(-6.3, iv(5.0, 12.4)));
    assert!(!member(0.0, Interval::empty()));
    assert!(!member(-4535.3, Interval::empty()));
    assert!(!member(NEG_INF, Interval::empty()));
    assert!(!member(INF, Interval::empty()));
    assert!(!member(f64::NAN, Interval::empty()));
    assert!(!member(NEG_INF, Interval::entire())); // contains == true; is_member == false
    assert!(!member(INF, Interval::entire())); // contains == true; is_member == false
    assert!(!member(f64::NAN, Interval::entire()));
}

// --- Set operations: intersection / convexHull (bare) ---------------------

#[test]
fn intersection_bare_vectors() {
    // minimal_intersection_test: 5 vectors
    assert_eq!(iv(1.0, 3.0).intersection(iv(2.1, 4.0)), iv(2.1, 3.0));
    assert_eq!(iv(1.0, 3.0).intersection(iv(3.0, 4.0)), iv(3.0, 3.0));
    assert!(iv(1.0, 3.0).intersection(Interval::empty()).is_empty());
    assert!(Interval::<f64>::entire()
        .intersection(Interval::empty())
        .is_empty());
    assert_eq!(iv(1.0, 3.0).intersection(Interval::entire()), iv(1.0, 3.0));
}

#[test]
fn convex_hull_bare_vectors() {
    // minimal_convex_hull_test: 5 vectors
    assert_eq!(iv(1.0, 3.0).convex_hull(iv(2.1, 4.0)), iv(1.0, 4.0));
    assert_eq!(iv(1.0, 1.0).convex_hull(iv(2.1, 4.0)), iv(1.0, 4.0));
    assert_eq!(iv(1.0, 3.0).convex_hull(Interval::empty()), iv(1.0, 3.0));
    assert!(Interval::<f64>::empty()
        .convex_hull(Interval::empty())
        .is_empty());
    assert!(iv(1.0, 3.0).convex_hull(Interval::entire()).is_entire());
}

// ===========================================================================
// REQUIRED-OPERATION GAPS (surface the crate does not expose)
// ===========================================================================
//
// The following corpus testcases exercise operations the crate does not surface.
// Their bare siblings above are covered in full; the decorated / recommended
// forms are reported here rather than faked (do not invent API). Each entry names
// the missing operation, the blocked testcase, and its corpus vector count. The
// crate deliberately factors decoration handling: the boolean and relational
// operations carry decorated forms (see the `_dec` tests above), but the numeric
// accessors, the unary predicates, and the set operations do not.
//
// libieeep1788_bool.itl -- no decorated unary predicate (only `is_nai` exists):
//   minimal_is_empty_dec_test   (15 vectors) -- no DecoratedInterval::is_empty
//   minimal_is_entire_dec_test  (17 vectors) -- no DecoratedInterval::is_entire
//   Each reduces to `!d.is_nai() && d.interval().is_X()`. The NaI rule they add
//   is already pinned by is_nai_dec_vectors plus the decorated relational twins,
//   and the bare predicate behavior by is_empty_bare_vectors / is_entire_bare.
//
// libieeep1788_num.itl -- no decorated numeric accessor:
//   minimal_inf_dec_test / minimal_sup_dec_test           (15 / 15 vectors)
//   minimal_wid_dec_test / minimal_mag_dec_test / minimal_mig_dec_test
//                                                          (9 / 9 / 12 vectors)
//   minimal_mid_dec_test / minimal_rad_dec_test / minimal_mid_rad_dec_test
//                                                          (13 / 10 / 13 vectors)
//   For wid/mag/mig/mid/rad the standard's `X([nai]) = NaN` reads through
//   `d.interval()` (a NaI's interval part is empty, whose numeric functions are
//   `None`), so these are pure pass-through of the bare twins and add no
//   coverage. For inf/sup the crate CANNOT reproduce the NaI datum through
//   `interval()`: `nai().interval().inf()` is `+inf`, not `NaN`, so a decorated
//   inf/sup accessor is genuinely absent, not just factored away.
//
// libieeep1788_rec_bool.itl -- no decorated / recommended operation:
//   minimal_is_common_interval_dec_test (21 vectors) -- no is_common_interval
//   minimal_is_singleton_dec_test       (16 vectors) -- no DecoratedInterval::is_singleton
//   minimal_is_member_dec_test          (40 vectors; the corpus itself repeats
//     two `isMember ... [empty]_trv = false` statements, and the true statement
//     count is what is recorded) -- no is_member
//   The bare intents are covered above (is_singleton exactly; is_common_interval
//   and is_member via documented composition over is_bounded / contains).
//
// libieeep1788_set.itl -- no decorated set operation:
//   minimal_intersection_dec_test (5 vectors) -- no DecoratedInterval::intersection
//   minimal_convex_hull_dec_test  (5 vectors) -- no DecoratedInterval::convex_hull
//   The corpus decorates every result `trv`, a decoration no crate operation
//   produces; the bare set behavior is covered by intersection_bare_vectors /
//   convex_hull_bare_vectors.

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
