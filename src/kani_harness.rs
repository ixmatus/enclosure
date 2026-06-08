//! Kani proof harnesses, over the `f64` fixture.
//!
//! These discharge the discrete, set-based case logic by bounded model checking:
//! the total-operation edge cases that are easy to get wrong (an empty operand,
//! a zero divisor, a reciprocal across zero, a square root with empty image).
//! CBMC models `f64` bit for bit, so these run real fixture code, and each
//! asserted path reaches its result through case selection and early returns
//! rather than through directed-rounding arithmetic.
//!
//! Run with `cargo kani --features fixture`; gated on `fixture` for the `f64`
//! instance of [`RoundFloat`](crate::RoundFloat).
//!
//! # Why the numeric enclosure is not proven here
//!
//! The directed-rounding fixture rounds outward with `next_up` / `next_down`,
//! which transmute `f64` to and from its bit pattern and branch on it. CBMC
//! bit-blasts that into a formula too large to discharge even for a single
//! two-operand operation, so a Kani enclosure proof over this fixture is not
//! tractable. The enclosure theorem ([`spec`](crate::spec) Law 2) therefore
//! rests on the written argument in `spec` (the `next_after` bracketing of a
//! round-to-nearest result) and on the property tests in
//! `tests/enclosure_fixture.rs`, which exercise both the general interior-point
//! enclosure and the singleton merge gate over thousands of cases.
//!
//! The route to a machine-checked four-corner enclosure is the abstract-axiom
//! approach (ADR-0003 alternative): a Kani-only backend whose directed operations
//! are symbolic values constrained by the soundness inequality rather than
//! computed by nextafter, which removes the bit manipulation CBMC cannot handle.
//! That is tracked as its own task. The decoration lattice (a later phase) is a
//! finite-state structure with no float arithmetic and is a natural Kani target.

use crate::Interval;

/// An arbitrary bounded interval (both endpoints finite).
fn any_bounded() -> Interval<f64> {
    let lo: f64 = kani::any();
    let hi: f64 = kani::any();
    kani::assume(lo.is_finite() && hi.is_finite());
    kani::assume(lo <= hi);
    Interval::new(lo, hi).unwrap()
}

/// The reciprocal of an interval with zero strictly interior is `Entire`.
#[kani::proof]
fn recip_of_zero_straddling_is_entire() {
    let lo: f64 = kani::any();
    let hi: f64 = kani::any();
    kani::assume(lo.is_finite() && hi.is_finite());
    kani::assume(lo < 0.0 && hi > 0.0);
    let r = Interval::new(lo, hi).unwrap().recip();
    kani::assert(r.is_entire(), "reciprocal across zero is Entire");
}

/// The reciprocal of the exact zero interval is the empty set.
#[kani::proof]
fn recip_of_exact_zero_is_empty() {
    let r = Interval::new(0.0, 0.0).unwrap().recip();
    kani::assert(r.is_empty(), "reciprocal of [0, 0] is empty");
}

/// Division by the exact zero interval is the empty set.
#[kani::proof]
fn div_by_exact_zero_is_empty() {
    let a = any_bounded();
    let zero = Interval::new(0.0, 0.0).unwrap();
    kani::assert((a / zero).is_empty(), "division by exact zero is empty");
}

/// The empty interval is absorbing under addition.
#[kani::proof]
fn add_with_empty_is_empty() {
    let a = any_bounded();
    let e = Interval::<f64>::empty();
    kani::assert((a + e).is_empty(), "x + empty is empty");
    kani::assert((e + a).is_empty(), "empty + x is empty");
}

/// The empty interval is absorbing under multiplication.
#[kani::proof]
fn mul_with_empty_is_empty() {
    let a = any_bounded();
    let e = Interval::<f64>::empty();
    kani::assert((a * e).is_empty(), "x * empty is empty");
}

/// The square root of a wholly-negative interval has empty image.
#[kani::proof]
fn sqrt_of_wholly_negative_is_empty() {
    let lo: f64 = kani::any();
    let hi: f64 = kani::any();
    kani::assume(lo.is_finite() && hi.is_finite());
    kani::assume(lo <= hi && hi < 0.0);
    let r = Interval::new(lo, hi).unwrap().sqrt();
    kani::assert(r.is_empty(), "sqrt of a wholly-negative interval is empty");
}
