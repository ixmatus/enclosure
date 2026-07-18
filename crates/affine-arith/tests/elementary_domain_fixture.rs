//! Adversarial property and unit tests for the single-term exact domain gate
//! of the Chebyshev elementary functions (`sqrt`, `ln`, `recip`), over the f64
//! fixture.
//!
//! The gate under test: a form with exactly one deviation term has the exact
//! range `[x₀ − |x₁|, x₀ + |x₁|]`, and the domain decision compares the center
//! against the coefficient magnitude with no rounding, where the old
//! reduced-endpoint predicate (radius rounded up, endpoint rounded outward
//! again) could cross a domain boundary the exact range does not. Multi-term
//! forms keep the reduced-range decision. `exp` has no domain gate (its
//! declines are overflow shaped), so it does not appear here.
//!
//! Three obligations, in the adversarial mold of the zero-touching analysis:
//!
//! - **Monotonicity.** The old reduced-endpoint predicate, kept here as the
//!   reference, never accepts a form the new arm declines; and multi-term
//!   forms, outside the tightening's scope, still behave exactly as the old
//!   predicate decides (the anti-scope-creep direction).
//! - **Soundness of new acceptances.** Every acceptance the old predicate
//!   declined is arbitrated by exact integer reasoning on the float bit
//!   patterns (no float arithmetic in the arbiter), and the returned enclosure
//!   passes the elementary lane's pointwise containment check at sample points
//!   clamped inside the exact range by an error-free rounding residual.
//! - **The boundary shapes that motivated the change.** `x₀ = nextUp(|x₁|)`
//!   makes the reduced endpoint dip below zero while the exact range sits in
//!   the domain (the arm must now accept and enclose), while `x₀ = |x₁|` for
//!   the strict domains, `x₀ = nextDown(|x₁|)` for `sqrt`, and the
//!   zero-touching `from_interval([0, b])` are genuinely out and must still
//!   decline.

use affine_arith::{AffineForm, SymbolSource};
use proptest::prelude::*;

/// The old domain predicates, reconstructed from the reduced endpoints exactly
/// as the arms tested them before the single-term exact gate: `sqrt` wanted
/// `inf ≥ 0`, `ln` wanted `inf > 0`, `recip` wanted sign stability of the
/// reduced range. Kept as the reference the monotonicity property compares
/// against.
fn old_sqrt_accepts(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && a >= 0.0
}
fn old_ln_accepts(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && a > 0.0
}
fn old_recip_accepts(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a > 0.0 || b < 0.0)
}

/// The rounding error of a round-to-nearest float sum: returns `(s, e)` with
/// `s = fl(a + b)` and `s + e = a + b` EXACTLY, as real numbers.
///
/// Branch-free 2Sum, derived from Ogita, Rump and Oishi 2005, Algorithm 3.1
/// (Knuth's six-operation form); registered at
/// `docs/references/ogita-rump-oishi-2005.md`. The error of a round-to-nearest
/// binary sum is itself a representable float and the six operations recover
/// it exactly; addition needs no underflow caveat (unlike the product
/// transform). That guarantee holds under round-to-nearest arithmetic, which
/// host `f64` provides in this lane; the generic library code cannot use the
/// transform because the directed-only `RoundFloat` contract offers no
/// round-to-nearest operation, which is exactly why the exact gate is
/// single-term only.
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    let e = (a - (s - bb)) + (b - bb);
    (s, e)
}

/// The tightest float pair INSIDE the exact range `[center − mag, center + mag]`:
/// the smallest float at or above the exact infimum and the largest float at or
/// below the exact supremum.
///
/// Soundness of the clamp: `s = fl(x)` for an exact real `x` is the nearest
/// float, so when `s ≠ x` the value `x` lies strictly between the neighbors of
/// `s`. The 2Sum residual gives the side exactly: `e < 0` means `s` overstates
/// `x`, so `next_down(s) < x`; `e > 0` means `s` understates, so
/// `next_up(s) > x`; `e = 0` means `s = x`. Sample points drawn from the
/// returned pair therefore lie inside the exact range, which is the input set
/// the enclosure guarantee quantifies over.
fn inside_exact_range(center: f64, mag: f64) -> (f64, f64) {
    let (s_hi, e_hi) = two_sum(center, mag);
    let hi = if e_hi < 0.0 { s_hi.next_down() } else { s_hi };
    let (s_lo, e_lo) = two_sum(center, -mag);
    let lo = if e_lo > 0.0 { s_lo.next_up() } else { s_lo };
    (lo, hi)
}

/// Exact arbiter for `a ≥ b` on finite floats with `b > 0`, by integer
/// reasoning on the bit patterns: a negatively signed `a` (including `-0`) is
/// below any positive `b`; otherwise, for IEEE 754 binary64 values with sign
/// bit zero, the (biased exponent, significand) fields read as one unsigned
/// integer order the values numerically (the exponent field is the more
/// significant, and within a binade the significand is the scaled integer
/// value), so unsigned comparison of `to_bits()` decides the real comparison
/// with no float arithmetic. This is the independent judge for the soundness
/// property: it must agree with the gate, and it shares none of the gate's
/// machinery.
fn exact_ge(a: f64, b: f64) -> bool {
    assert!(a.is_finite() && b.is_finite() && b > 0.0);
    if a.is_sign_negative() {
        return false;
    }
    a.to_bits() >= b.to_bits()
}

/// Companion strict arbiter for `a > b`, same reasoning.
fn exact_gt(a: f64, b: f64) -> bool {
    assert!(a.is_finite() && b.is_finite() && b > 0.0);
    if a.is_sign_negative() {
        return false;
    }
    a.to_bits() > b.to_bits()
}

/// A single-term `'static` form `x₀ + x₁·ε₀` and a source resumed past its one
/// symbol, ready for the elementary arms.
fn single_term_form(center: f64, coeff: f64) -> (AffineForm<'static, f64>, SymbolSource<'static>) {
    let form = AffineForm::<f64>::from_raw_parts(center, [(0, coeff)]).expect("finite parts");
    (form, SymbolSource::unscoped_at(1))
}

/// Assert `enc` contains `f(x)` on a 9-point sweep of `[lo, hi]`, the
/// pointwise idiom of the elementary lane, with each sample clamped back into
/// `[lo, hi]` so interpolation rounding cannot step outside the exact range.
fn assert_pointwise<F: Fn(f64) -> f64>(
    enc: &interval_1788::Interval<f64>,
    lo: f64,
    hi: f64,
    f: F,
    name: &str,
) {
    for k in 0..=8 {
        let x = (lo + (hi - lo) * (f64::from(k) / 8.0)).clamp(lo, hi);
        assert!(
            enc.contains(f(x)),
            "{name} lost the image of {x} over the exact range [{lo}, {hi}]"
        );
    }
}

proptest! {
    /// Monotonicity of the single-term gate: whatever the old reduced-endpoint
    /// predicate accepted, the new arm still accepts. The generation window
    /// keeps ranges comfortably inside the normal floats, so the only declines
    /// in play are domain declines (the overflow and underflow declines
    /// downstream of the gate are shared by both predicates and orthogonal to
    /// it).
    #[test]
    fn single_term_gate_is_monotone_versus_the_reduced_gate(
        center in -1.0e3f64..1.0e3,
        mag in 1.0e-3f64..1.0e3,
        negative_coeff in proptest::bool::ANY,
    ) {
        let coeff = if negative_coeff { -mag } else { mag };
        let (form, mut src) = single_term_form(center, coeff);
        let iv = form.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if old_sqrt_accepts(a, b) {
            prop_assert!(form.sqrt(&mut src).is_some(),
                "sqrt declined a form the reduced gate accepted: [{a}, {b}]");
        }
        if old_ln_accepts(a, b) {
            prop_assert!(form.ln(&mut src).is_some(),
                "ln declined a form the reduced gate accepted: [{a}, {b}]");
        }
        if old_recip_accepts(a, b) {
            prop_assert!(form.recip(&mut src).is_some(),
                "recip declined a form the reduced gate accepted: [{a}, {b}]");
        }
    }

    /// Multi-term forms are outside the tightening's scope and keep the
    /// reduced-range decision in BOTH directions: the old predicate's verdict
    /// is the arm's verdict. This is the anti-scope-creep regression; an exact
    /// multi-term gate would need the error-free summation the directed-only
    /// contract cannot express, so accepting here would be unsound reach.
    #[test]
    fn multi_term_forms_keep_the_reduced_gate(
        center in -1.0e3f64..1.0e3,
        m0 in 1.0e-3f64..1.0e3,
        m1 in 1.0e-3f64..1.0e3,
        s0 in proptest::bool::ANY,
        s1 in proptest::bool::ANY,
    ) {
        let c0 = if s0 { -m0 } else { m0 };
        let c1 = if s1 { -m1 } else { m1 };
        let form = AffineForm::<f64>::from_raw_parts(center, [(0, c0), (1, c1)])
            .expect("finite parts");
        let mut src = SymbolSource::unscoped_at(2);
        let iv = form.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        prop_assert_eq!(form.sqrt(&mut src).is_some(), old_sqrt_accepts(a, b),
            "multi-term sqrt diverged from the reduced gate on [{}, {}]", a, b);
        prop_assert_eq!(form.ln(&mut src).is_some(), old_ln_accepts(a, b),
            "multi-term ln diverged from the reduced gate on [{}, {}]", a, b);
        prop_assert_eq!(form.recip(&mut src).is_some(), old_recip_accepts(a, b),
            "multi-term recip diverged from the reduced gate on [{}, {}]", a, b);
    }

    /// The boundary sweep: `x₀` walks across the exact domain boundary in
    /// single-ulp steps around `|x₁|`. Acceptance must match the EXACT
    /// membership verdict of the bit-integer arbiter in both directions
    /// (accept in-domain, decline out-of-domain; in this window a positive
    /// exact infimum is at least an ulp of `mag`, far above the underflow
    /// region, so the directed fit bound cannot round it away and the gate is
    /// if-and-only-if). Every acceptance must then enclose the true image at
    /// sample points clamped inside the exact range.
    #[test]
    fn boundary_acceptances_match_the_exact_arbiter_and_enclose(
        mag in 1.0e-3f64..1.0e3,
        steps in -3i32..=3,
    ) {
        let mut center = mag;
        for _ in 0..steps.abs() {
            center = if steps > 0 { center.next_up() } else { center.next_down() };
        }
        let (form, mut src) = single_term_form(center, mag);
        let sqrt_r = form.sqrt(&mut src);
        let ln_r = form.ln(&mut src);
        let recip_r = form.recip(&mut src);

        let sqrt_in = exact_ge(center, mag); // exact infimum >= 0
        let strict_in = exact_gt(center, mag); // exact infimum > 0
        prop_assert_eq!(sqrt_r.is_some(), sqrt_in,
            "sqrt verdict diverged from the exact arbiter at {} vs {}", center, mag);
        prop_assert_eq!(ln_r.is_some(), strict_in,
            "ln verdict diverged from the exact arbiter at {} vs {}", center, mag);
        prop_assert_eq!(recip_r.is_some(), strict_in,
            "recip verdict diverged from the exact arbiter at {} vs {}", center, mag);

        let (lo, hi) = inside_exact_range(center, mag);
        if let Some(g) = sqrt_r {
            assert_pointwise(&g.reduce(), lo, hi, f64::sqrt, "sqrt");
        }
        if let Some(g) = ln_r {
            assert_pointwise(&g.reduce(), lo, hi, f64::ln, "ln");
        }
        if let Some(g) = recip_r {
            assert_pointwise(&g.reduce(), lo, hi, |x| 1.0 / x, "recip");
        }
    }
}

/// The motivating shape, pinned deterministically across decades: with
/// `x₀ = nextUp(|x₁|)` the reduced radius rounds `0 + |x₁|` up to `x₀` itself,
/// so the reduced infimum lands one step below zero and every old predicate
/// declines, while the exact range `[ulp, 2|x₁| + ulp]` sits strictly inside
/// every domain. The arms must accept and their enclosures must pass the
/// pointwise check inside the exact range.
#[test]
fn one_ulp_boundary_forms_are_accepted_with_sound_enclosures() {
    for &mag in &[1.0e-3f64, 0.5, 1.0, 3.7, 1.0e3] {
        let center = mag.next_up();
        let (form, mut src) = single_term_form(center, mag);
        let iv = form.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        assert!(
            a < 0.0,
            "the reduced infimum {a} did not cross zero for mag {mag}: not the motivating shape"
        );
        assert!(!old_sqrt_accepts(a, b) && !old_ln_accepts(a, b) && !old_recip_accepts(a, b));

        let sqrt_enc = form
            .sqrt(&mut src)
            .expect("exact range inside the sqrt domain")
            .reduce();
        let ln_enc = form
            .ln(&mut src)
            .expect("exact range inside the ln domain")
            .reduce();
        let recip_enc = form
            .recip(&mut src)
            .expect("exact range sign stable")
            .reduce();

        let (lo, hi) = inside_exact_range(center, mag);
        assert_pointwise(&sqrt_enc, lo, hi, f64::sqrt, "sqrt");
        assert_pointwise(&ln_enc, lo, hi, f64::ln, "ln");
        assert_pointwise(&recip_enc, lo, hi, |x| 1.0 / x, "recip");
    }
}

/// The negative mirror: `x₀ = −nextUp(|x₁|)` has an exact range one ulp below
/// zero on its high side, sign stable and negative, reached through the exact
/// negation reflection. `recip` accepts and encloses; `sqrt` and `ln` decline
/// (the exact range is negative).
#[test]
fn negative_mirror_boundary_form_is_accepted_by_recip() {
    let mag = 2.5f64;
    let center = -(mag.next_up());
    let (form, mut src) = single_term_form(center, mag);
    let iv = form.reduce();
    assert!(iv.sup() > 0.0, "the reduced supremum must cross zero");
    assert!(form.sqrt(&mut src).is_none());
    assert!(form.ln(&mut src).is_none());
    let recip_enc = form
        .recip(&mut src)
        .expect("exact range sign stable and negative")
        .reduce();
    let (lo, hi) = inside_exact_range(center, mag);
    assert_pointwise(&recip_enc, lo, hi, |x| 1.0 / x, "recip");
}

/// The exact boundary itself: `x₀ = |x₁|` puts the exact infimum at zero,
/// inside the sqrt domain (whose enclosure must then contain `√0 = 0`) and
/// outside the strict `ln` and `recip` domains, which must decline. The old
/// sqrt predicate declines this shape (its reduced infimum is below zero), so
/// the sqrt acceptance is a tightening witnessed at the exact boundary.
#[test]
fn exact_zero_infimum_accepts_sqrt_and_declines_the_strict_domains() {
    for &mag in &[1.0e-3f64, 1.0, 1.0e3] {
        let (form, mut src) = single_term_form(mag, mag);
        let iv = form.reduce();
        assert!(!old_sqrt_accepts(iv.inf(), iv.sup()));
        let sqrt_enc = form
            .sqrt(&mut src)
            .expect("exact infimum zero is in the sqrt domain")
            .reduce();
        assert!(
            sqrt_enc.contains(0.0),
            "sqrt enclosure lost the image of the exact infimum zero"
        );
        let (lo, hi) = inside_exact_range(mag, mag);
        assert_pointwise(&sqrt_enc, lo, hi, f64::sqrt, "sqrt");
        assert!(
            form.ln(&mut src).is_none(),
            "ln must decline an exact zero infimum"
        );
        assert!(
            form.recip(&mut src).is_none(),
            "recip must decline an exact zero infimum"
        );
    }
}

/// The soundness-beats-tightness decline: `|x₁| = MIN_POSITIVE` with
/// `x₀ = nextUp(|x₁|)` has the exact infimum at the smallest subnormal,
/// strictly positive, but its sound directed fit bound `x₀.sub_down(|x₁|)`
/// rounds to zero, out of the strict domains. `ln` and `recip` must decline
/// rather than fit from an out-of-domain endpoint; `sqrt`, whose clamp needs
/// no strict positivity, accepts.
#[test]
fn positive_infimum_whose_directed_bound_rounds_out_declines() {
    let mag = f64::MIN_POSITIVE;
    let center = mag.next_up();
    let (form, mut src) = single_term_form(center, mag);
    assert!(
        form.sqrt(&mut src).is_some(),
        "sqrt clamps to zero and needs no strict positivity"
    );
    assert!(form.ln(&mut src).is_none());
    assert!(form.recip(&mut src).is_none());
}

/// Genuinely out-of-domain exact ranges keep declining through the exact gate.
#[test]
fn genuinely_out_of_domain_exact_ranges_decline() {
    // One ulp below the boundary: the exact infimum is negative.
    let mag = 1.0f64;
    let (form, mut src) = single_term_form(mag.next_down(), mag);
    assert!(form.sqrt(&mut src).is_none());
    assert!(form.ln(&mut src).is_none());
    assert!(form.recip(&mut src).is_none());

    // Astride zero by construction: the exact range is [-1, 1].
    let (astride, mut src2) = single_term_form(0.0, 1.0);
    assert!(astride.sqrt(&mut src2).is_none());
    assert!(astride.ln(&mut src2).is_none());
    assert!(astride.recip(&mut src2).is_none());
}
