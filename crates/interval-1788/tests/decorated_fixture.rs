//! Property tests for decorated intervals, over the `f64` fixture: decoration
//! propagation never strengthens, `NaI` poisons, every result is a consistent
//! decorated pair, and the bare interval matches the undecorated operation.

use interval_1788::{DecoratedInterval, Decoration, Interval};
use proptest::prelude::*;

fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![-1.0e6..1.0e6, Just(0.0), Just(-0.0), -1.0e-6..1.0e-6,]
}

/// A decorated interval spanning every decoration: a bounded one (`com`), an
/// unbounded one (`dac`), the empty set (`trv`), and the `NaI` (`ill`).
fn any_di() -> impl Strategy<Value = DecoratedInterval<f64>> {
    prop_oneof![
        (finite(), finite()).prop_map(|(a, b)| {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            DecoratedInterval::new_dec(Interval::new(lo, hi).unwrap())
        }),
        finite().prop_map(|a| DecoratedInterval::new_dec(Interval::new(a, f64::INFINITY).unwrap())),
        Just(DecoratedInterval::new_dec(Interval::<f64>::empty())),
        Just(DecoratedInterval::<f64>::nai()),
    ]
}

/// Magnitudes that reach `f64::MAX` (and the subnormal floor), so a bounded
/// operation can overflow to an unbounded result and exercise the demotion:
/// `MAX` for `+`/`-`/`*`/`mul_add`, the smallest subnormal for `recip`.
fn wide() -> impl Strategy<Value = f64> {
    prop_oneof![
        Just(f64::MAX),
        Just(f64::MAX / 2.0),
        Just(-f64::MAX),
        Just(-f64::MAX / 2.0),
        Just(f64::from_bits(1)),
        Just(-f64::from_bits(1)),
        -1.0e300..1.0e300,
    ]
}

/// A decorated interval that can be near-`MAX` bounded (so a bounded operation
/// overflows), unbounded, empty, or the `NaI`.
fn wide_di() -> impl Strategy<Value = DecoratedInterval<f64>> {
    prop_oneof![
        any_di(),
        (wide(), wide()).prop_map(|(a, b)| {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            DecoratedInterval::new_dec(Interval::new(lo, hi).unwrap())
        }),
    ]
}

proptest! {
    #[test]
    fn bare_interval_is_the_undecorated_operation(a in any_di(), b in any_di()) {
        prop_assume!(!a.is_nai() && !b.is_nai());
        prop_assert_eq!((a + b).interval(), a.interval() + b.interval());
        prop_assert_eq!((a - b).interval(), a.interval() - b.interval());
        prop_assert_eq!((a * b).interval(), a.interval() * b.interval());
        prop_assert_eq!((a / b).interval(), a.interval() / b.interval());
    }

    #[test]
    fn decoration_never_strengthens(a in any_di(), b in any_di()) {
        prop_assume!(!a.is_nai() && !b.is_nai());
        let r = a + b;
        prop_assert!(r.decoration() <= a.decoration());
        prop_assert!(r.decoration() <= b.decoration());
    }

    #[test]
    fn nai_poisons_exactly_when_an_input_is_nai(a in any_di(), b in any_di()) {
        let poisoned = a.is_nai() || b.is_nai();
        prop_assert_eq!((a + b).is_nai(), poisoned);
        prop_assert_eq!((a - b).is_nai(), poisoned);
        prop_assert_eq!((a * b).is_nai(), poisoned);
        prop_assert_eq!((a / b).is_nai(), poisoned);
    }

    #[test]
    fn binary_results_are_consistent(a in any_di(), b in any_di()) {
        for r in [a + b, a - b, a * b, a / b] {
            if r.decoration() == Decoration::Com {
                prop_assert!(r.interval().is_bounded() && !r.interval().is_empty());
            }
            if r.decoration() == Decoration::Dac {
                prop_assert!(!r.interval().is_empty());
            }
            if r.is_nai() {
                prop_assert!(r.interval().is_empty());
            }
        }
    }

    #[test]
    fn unary_results_are_consistent(a in any_di()) {
        for r in [a.sqr(), a.sqrt(), a.recip(), -a] {
            if r.decoration() == Decoration::Com {
                prop_assert!(r.interval().is_bounded() && !r.interval().is_empty());
            }
            if r.is_nai() {
                prop_assert!(r.interval().is_empty());
            }
        }
    }

    /// `com` promises a bounded result, so any operation that lands an unbounded
    /// interval must decorate it strictly below `com`. The wide strategy reaches
    /// `f64::MAX`, so bounded inputs overflow here rather than only unbounded
    /// inputs propagating through.
    #[test]
    fn an_unbounded_result_is_decorated_below_com(a in wide_di(), b in wide_di()) {
        prop_assume!(!a.is_nai() && !b.is_nai());
        let results = [
            a + b,
            a - b,
            a * b,
            a / b,
            a.mul_add(b, a),
            a.sqr(),
            a.sqrt(),
            a.recip(),
            -a,
        ];
        for r in results {
            if !r.interval().is_bounded() {
                prop_assert!(r.decoration() < Decoration::Com);
            }
        }
    }
}
