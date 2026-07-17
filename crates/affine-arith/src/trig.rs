//! Per-arc trigonometric, hyperbolic, and power fits on affine forms, by
//! Chebyshev (secant-slope) affine approximation.
//!
//! These grow the elementary surface past the convex-or-concave-everywhere
//! functions of the [`elementary`](crate::elementary) module. `sin`, `cos`,
//! `sinh`, `cosh`, and `tanh` are periodic or split-sign in their second
//! derivative, so the one-signed-`f''` requirement the Chebyshev builder needs
//! (workspace decision record 0002) holds only on a restricted range. The policy
//! this module implements is workspace decision record 0005 part 4: fit exactly
//! as [`exp`](AffineForm::exp) does when the reduced range lies within a single
//! concavity arc, and return `None` past an arc boundary so the caller falls back
//! to the interval arm (the same `None` convention the algebraic surface uses for
//! a domain violation).
//!
//! # The residual band without an inverse function
//!
//! [`exp`](AffineForm::exp) and [`ln`](AffineForm::ln) locate the interior tangent
//! point of the Chebyshev residual exactly, because they have each other as an
//! inverse. `sin`, `cos`, and the hyperbolic family have no inverse in the
//! [`RoundFloat`] contract (`acos`, `asinh` are not available), so the interior
//! extremum of the residual `r(x) = f(x) − α·x` cannot be evaluated directly.
//!
//! This module bounds it instead by the classical linear-interpolation error
//! bound (standard approximation theory; for example Süli and Mayers,
//! *An Introduction to Numerical Analysis*, the linear-interpolation remainder).
//! For a twice-differentiable `f` on `[a, b]`,
//!
//! ```text
//!     f(x) − P₁(x) = ½ · f''(ξ) · (x − a)(x − b),    ξ ∈ (a, b),
//! ```
//!
//! where `P₁` is the chord (the linear interpolant through the endpoints). The
//! factor `(x − a)(x − b)` has magnitude at most `(b − a)²/4` at the midpoint, so
//! with `M ≥ max|f''|` on `[a, b]`,
//!
//! ```text
//!     |f(x) − P₁(x)| ≤ M · (b − a)² / 8.
//! ```
//!
//! Writing `D = M · (b − a)² / 8` and noting that the `α·x` term is linear (so it
//! cancels out of `chord_r(x) − r(x) = chord_f(x) − f(x)`, independent of the
//! slope `α` that was actually chosen), the residual is pinned to one side of its
//! chord: for a convex arc (`f'' ≥ 0`) the function lies below its chord, so
//! `r(x) ∈ [min(r(a), r(b)) − D, max(r(a), r(b))]`; for a concave arc the mirror,
//! `r(x) ∈ [min(r(a), r(b)), max(r(a), r(b)) + D]`. Every step rounds outward.
//!
//! # Sound over-estimate, not the true minimax
//!
//! This residual band is a **sound over-estimate**, not the true Chebyshev
//! minimax band, and the doc comment on each function states its worst-case
//! widening factor. The over-estimate has two sources. First, `M` is an upper
//! bound on `|f''|` over the whole arc, but the true residual depth is governed by
//! the *local* curvature, which is smaller than `M` away from the curvature peak;
//! for `sin`/`cos` (`M = 1`) the worst case over a full arc is a factor of
//! `π²/8 ≈ 1.23` against the true peak deviation, and the relative over-estimate
//! grows without bound (though the absolute band stays `≤ (b − a)²/8`) for a
//! narrow arc hugging an inflection point, where the local `|f''| = |sin|` falls
//! toward zero while `M` stays `1`. Second, the chord bound `D` is exact only for
//! a parabola (constant `f''`); for the exponentially-growing hyperbolic curvature
//! the factor rises with the range width. On the narrow ranges the SMIL BAND
//! plotter feeds these fits the factor is close to `1`; a tighter per-interval `M`
//! (or a direct interior-extremum bound) is left as future work, recorded here so
//! the widening is not mistaken for the minimax.
//!
//! # Arc membership and the pi-ambiguity rule
//!
//! An arc boundary of `sin`/`cos` is a zero of the function (an inflection point):
//! `k·π` for `sin`, `π/2 + k·π` for `cos`. Whether the reduced range `[a, b]` lies
//! within a single arc is decided against the **pi enclosure**
//! ([`pi_down`](round_float::RoundTrig::pi_down) /
//! [`pi_up`](round_float::RoundTrig::pi_up)): an integer index `k` is accepted only
//! when an outward-rounded *upper* bound on the left boundary is at or below `a`
//! and an outward-rounded *lower* bound on the right boundary is at or above `b`,
//! so the containment holds for the true `π`, not merely the bracketed one. When
//! the pi bracket widens a boundary across `a` or `b` (the range sits within an
//! enclosure width of an inflection point, or straddles one), no `k` is accepted
//! and the fit **declines with `None` rather than guess** the arc. This is the
//! affine instance of the same ambiguity rule the interval arm applies (decision
//! record 0005 part 3): when correlation to the truth is uncertain, pay width,
//! never soundness; here the width paid is the whole fit, ceded to the interval
//! fallback.

use round_float::{RoundFloat, RoundHyperbolic, RoundTranscendental, RoundTrig};

use crate::form::AffineForm;
use crate::symbol::SymbolSource;

/// A divisor strictly below `8`, built downward so that dividing by it
/// over-estimates `·/8`. The chord depth `D = M·(b − a)²/8` must round *up* to
/// stay a sound bound; dividing by a value at or above `8` (which
/// [`add_up`](RoundFloat::add_up) on the fixture would produce) could turn the
/// bound the wrong way, exactly the trick [`sqr`](AffineForm::sqr) uses for its
/// `α²/4` divisor.
#[inline]
fn eight_below<F: RoundFloat>() -> F {
    let two = F::ONE.add_down(F::ONE);
    let four = two.add_down(two);
    four.add_down(four)
}

/// The chord-error bound `D = m·(b − a)²/8`, rounded up. `m` is an upper bound on
/// `max|f''|` over the arc and `width` an upper bound on `b − a`; both nonnegative,
/// so every product and the final quotient over-estimate the true depth.
#[inline]
fn chord_depth<F: RoundFloat>(m: F, width: F) -> F {
    m.mul_up(width).mul_up(width).div_up(eight_below())
}

/// The residual `r(x) = f(x) − α·x` at one endpoint `x`, as a directed bracket
/// `[lo, hi]` from a directed bracket `[f_lo, f_hi]` of `f(x)`. The lower bound
/// takes the smallest `f` and the largest `α·x`; the upper bound the mirror. The
/// directed products handle the sign of `x` and `α` correctly.
#[inline]
fn residual_endpoint<F: RoundFloat>(f_lo: F, f_hi: F, alpha: F, x: F) -> (F, F) {
    (
        f_lo.sub_down(alpha.mul_up(x)),
        f_hi.sub_up(alpha.mul_down(x)),
    )
}

/// Whether an arithmetic grid point `phase + k·π` with an outward-rounded upper
/// bound at or below `a` exists whose successor `phase + (k+1)·π` has an
/// outward-rounded lower bound at or above `b`: that is, whether `[a, b]` provably
/// lies within a single arc between consecutive zeros of the family.
///
/// `phase_lo`/`phase_hi` bracket the phase offset (`0` for `sin`, `π/2` for `cos`)
/// and `pd`/`pu` bracket `π`. An index `k` is estimated by a floored division and
/// its two neighbors are checked, which covers the one-off the estimate can carry
/// at any magnitude a BAND range reaches; a range so large that the estimate is
/// off by more declines, which is sound.
fn within_single_arc<F: RoundFloat + RoundTrig>(
    a: F,
    b: F,
    phase_lo: F,
    phase_hi: F,
    pd: F,
    pu: F,
) -> bool {
    // Estimate k = floor((a − phase)/π); any π and phase in the brackets give a
    // usable estimate, since the true k is confirmed (or rejected) below.
    let approx = a.sub_down(phase_hi).div_down(pu);
    let k0 = approx.floor();
    if !k0.is_finite() {
        return false;
    }
    let mut k = k0.sub_down(F::ONE);
    let mut tries = 0;
    while tries < 3 {
        if arc_covers(k, a, b, phase_lo, phase_hi, pd, pu) {
            return true;
        }
        k = k.add_up(F::ONE);
        tries += 1;
    }
    false
}

/// Whether `[a, b] ⊆ [phase + k·π, phase + (k+1)·π]` for the true `π`, tested with
/// outward-rounded boundary enclosures so a pass is never wrong (it may be
/// conservatively missed near a boundary, never falsely granted).
fn arc_covers<F: RoundFloat + RoundTrig>(
    k: F,
    a: F,
    b: F,
    phase_lo: F,
    phase_hi: F,
    pd: F,
    pu: F,
) -> bool {
    let zero = F::ZERO;
    // Upper bound of the left zero phase + k·π: for k ≥ 0 the product is largest at
    // the larger π (pu), for k < 0 at the smaller π (pd).
    let kpi_up = if k >= zero {
        k.mul_up(pu)
    } else {
        k.mul_up(pd)
    };
    let left_up = phase_hi.add_up(kpi_up);
    // Lower bound of the right zero phase + (k+1)·π: mirror choice of π.
    let j = k.add_up(F::ONE);
    let jpi_dn = if j >= zero {
        j.mul_down(pd)
    } else {
        j.mul_down(pu)
    };
    let right_dn = phase_lo.add_down(jpi_dn);
    left_up <= a && right_dn >= b
}

impl<'id, F: RoundFloat> AffineForm<'id, F> {
    /// Assemble the fit from the slope, the two endpoint residual brackets, the
    /// chord depth, and the arc's convexity, then hand the residual band to
    /// [`linearize`](AffineForm::linearize).
    ///
    /// A convex arc (`f'' ≥ 0`) puts the interior extremum below the chord, so the
    /// band low end drops by `d`; a concave arc puts it above, so the high end
    /// rises. The endpoints pin the other side exactly (a chord's own extreme is at
    /// an endpoint), which is why only one side carries `d`.
    fn chord_fit(
        &self,
        alpha: F,
        ra: (F, F),
        rb: (F, F),
        d: F,
        convex: bool,
        src: &mut SymbolSource<'id>,
    ) -> Option<Self> {
        let min_lo = ra.0.rmin(rb.0);
        let max_hi = ra.1.rmax(rb.1);
        let (r_lo, r_hi) = if convex {
            (min_lo.sub_down(d), max_hi)
        } else {
            (min_lo, max_hi.add_up(d))
        };
        if !r_lo.is_finite() || !r_hi.is_finite() {
            return None;
        }
        Some(self.linearize(alpha, r_lo, r_hi, src))
    }
}

impl<'id, F: RoundFloat + RoundTrig> AffineForm<'id, F> {
    /// The sine `sin(x̂)`, by the per-arc Chebyshev affine approximation.
    ///
    /// Returns `None` unless the reduced range lies within a single concavity arc
    /// `[k·π, (k+1)·π]` (see the [module docs](crate::trig) for the arc-membership
    /// and pi-ambiguity rule): a range spanning an inflection point `k·π` has a
    /// residual whose second derivative changes sign, which the one-symbol builder
    /// cannot fit. A non-finite endpoint also declines.
    ///
    /// On a single arc `sin'' = −sin` keeps one sign, so `sin` is concave where it
    /// is positive and convex where it is negative; the sign is read from `sin` at
    /// an interior point. The residual band is the sound chord-error over-estimate
    /// of the [module docs](crate::trig), not the true minimax: the widening factor
    /// is at most `π²/8 ≈ 1.23` over a full arc and larger in relative terms near
    /// an inflection point.
    #[must_use]
    pub fn sin(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let zero = F::ZERO;
        self.fit_periodic(zero, zero, F::sin_down, F::sin_up, src)
    }

    /// The cosine `cos(x̂)`, by the per-arc Chebyshev affine approximation.
    ///
    /// The companion of [`sin`](AffineForm::sin): the arcs are
    /// `[π/2 + k·π, π/2 + (k+1)·π]` (cosine's inflection points are its zeros
    /// `π/2 + k·π`), and `cos'' = −cos`, so it is concave where cosine is positive
    /// and convex where negative. The `None` convention, the pi-ambiguity rule, and
    /// the sound-over-estimate residual band are identical to `sin`'s.
    #[must_use]
    pub fn cos(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let pd = F::pi_down();
        let pu = F::pi_up();
        let two_lo = F::ONE.add_down(F::ONE);
        let two_hi = F::ONE.add_up(F::ONE);
        // The phase offset π/2, bracketed: divide the smaller π by the larger 2 for
        // the lower bound, the larger π by the smaller 2 for the upper bound.
        let phase_lo = pd.div_down(two_hi);
        let phase_hi = pu.div_up(two_lo);
        self.fit_periodic(phase_lo, phase_hi, F::cos_down, F::cos_up, src)
    }

    /// The shared per-arc fit for a periodic family whose inflection points sit on
    /// the grid `phase + k·π`. `f_down`/`f_up` are the family's directed evaluators.
    fn fit_periodic<FD, FU>(
        &self,
        phase_lo: F,
        phase_hi: F,
        f_down: FD,
        f_up: FU,
        src: &mut SymbolSource<'id>,
    ) -> Option<Self>
    where
        FD: Fn(F) -> F,
        FU: Fn(F) -> F,
    {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        let zero = F::ZERO;
        let width_dn = b.sub_down(a);
        if width_dn <= zero {
            // Degenerate point range {c}: the constant fit f(c), slope zero, with
            // the value's own directed bracket as the band.
            return Some(self.linearize(zero, f_down(a), f_up(a), src));
        }
        let pd = F::pi_down();
        let pu = F::pi_up();
        if !within_single_arc(a, b, phase_lo, phase_hi, pd, pu) {
            return None;
        }
        // Concavity from the sign of f at an interior point: f'' = −f, so f > 0 is
        // a concave arc and f < 0 a convex one. An indeterminate sign (the range
        // hugs a zero so tightly the directed bracket straddles it) declines.
        let two_hi = F::ONE.add_up(F::ONE);
        let half = width_dn.div_down(two_hi);
        let m = a.add_up(half).rmin(b).rmax(a);
        let convex = if f_down(m) > zero {
            false
        } else if f_up(m) < zero {
            true
        } else {
            return None;
        };
        let alpha = f_up(b).sub_up(f_down(a)).div_up(width_dn);
        if !alpha.is_finite() {
            return None;
        }
        let ra = residual_endpoint(f_down(a), f_up(a), alpha, a);
        let rb = residual_endpoint(f_down(b), f_up(b), alpha, b);
        // |sin''| = |sin| ≤ 1 and |cos''| = |cos| ≤ 1, so the global M = 1.
        let d = chord_depth(F::ONE, b.sub_up(a));
        self.chord_fit(alpha, ra, rb, d, convex, src)
    }
}

impl<'id, F: RoundFloat + RoundHyperbolic> AffineForm<'id, F> {
    /// The hyperbolic cosine `cosh(x̂)`, by the Chebyshev affine approximation.
    ///
    /// `cosh'' = cosh > 0` everywhere, so this is convex on the whole line and
    /// fits any finite range with no arc restriction. The curvature bound is the
    /// range-local `M = max(cosh(a), cosh(b))` (cosh grows with `|x|`, so its
    /// maximum over `[a, b]` is at an endpoint), giving a chord-error band whose
    /// widening factor approaches `1` for a narrow range and grows with the range
    /// width as the curvature varies. Returns `None` when an endpoint is not finite
    /// or `cosh` overflows.
    #[must_use]
    pub fn cosh(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        let zero = F::ZERO;
        let (ca_lo, ca_hi) = (a.cosh_down(), a.cosh_up());
        let (cb_lo, cb_hi) = (b.cosh_down(), b.cosh_up());
        if !ca_hi.is_finite() || !cb_hi.is_finite() {
            // Overflow: the result is not a bounded affine form.
            return None;
        }
        let width_dn = b.sub_down(a);
        if width_dn <= zero {
            return Some(self.linearize(zero, ca_lo, ca_hi, src));
        }
        let m = ca_hi.rmax(cb_hi);
        let d = chord_depth(m, b.sub_up(a));
        if !d.is_finite() {
            return None;
        }
        let alpha = cb_hi.sub_up(ca_lo).div_up(width_dn);
        if !alpha.is_finite() {
            return None;
        }
        let ra = residual_endpoint(ca_lo, ca_hi, alpha, a);
        let rb = residual_endpoint(cb_lo, cb_hi, alpha, b);
        self.chord_fit(alpha, ra, rb, d, true, src)
    }

    /// The hyperbolic sine `sinh(x̂)`, by the Chebyshev affine approximation.
    ///
    /// `sinh'' = sinh` changes sign at `0`, so the range must not straddle it:
    /// `sinh` is convex on `[0, ∞)` and concave on `(−∞, 0]`, and a range with `0`
    /// in its interior returns `None` (the interval arm, monotone, carries it). The
    /// curvature bound is `M = max|sinh|` over the range, at the endpoint of larger
    /// magnitude. Returns `None` when an endpoint is not finite or `sinh` overflows.
    #[must_use]
    pub fn sinh(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        let zero = F::ZERO;
        let (sa_lo, sa_hi) = (a.sinh_down(), a.sinh_up());
        let (sb_lo, sb_hi) = (b.sinh_down(), b.sinh_up());
        if !sa_lo.is_finite() || !sa_hi.is_finite() || !sb_lo.is_finite() || !sb_hi.is_finite() {
            // Overflow at a shoulder: not a bounded affine form.
            return None;
        }
        let width_dn = b.sub_down(a);
        if width_dn <= zero {
            return Some(self.linearize(zero, sa_lo, sa_hi, src));
        }
        // Convex on the nonnegative half-line, concave on the nonpositive; a range
        // straddling zero has a sign-changing f'' and declines.
        let convex = if a >= zero {
            true
        } else if b <= zero {
            false
        } else {
            return None;
        };
        // Upper bound on |sinh| at each endpoint (sinh keeps the sign of its
        // argument), the larger of which bounds max|f''| over the range.
        let mag_a = if a >= zero { sa_hi } else { sa_lo.negate() };
        let mag_b = if b >= zero { sb_hi } else { sb_lo.negate() };
        let m = mag_a.rmax(mag_b);
        let d = chord_depth(m, b.sub_up(a));
        if !d.is_finite() {
            return None;
        }
        let alpha = sb_hi.sub_up(sa_lo).div_up(width_dn);
        if !alpha.is_finite() {
            return None;
        }
        let ra = residual_endpoint(sa_lo, sa_hi, alpha, a);
        let rb = residual_endpoint(sb_lo, sb_hi, alpha, b);
        self.chord_fit(alpha, ra, rb, d, convex, src)
    }

    /// The hyperbolic tangent `tanh(x̂)`, by the Chebyshev affine approximation.
    ///
    /// `tanh'' = −2·tanh·sech²` changes sign at `0`, so the range must not straddle
    /// it: `tanh` is concave on `[0, ∞)` and convex on `(−∞, 0]`, and a range with
    /// `0` in its interior returns `None`. The curvature is bounded globally by
    /// `M = 1` (the true maximum of `|tanh''|` is `4/(3√3) ≈ 0.77`, so `1` is a
    /// sound bound at a baseline widening factor of about `1.3`); `tanh` never
    /// overflows, so the only declines are the straddle and a non-finite endpoint.
    #[must_use]
    pub fn tanh(&self, src: &mut SymbolSource<'id>) -> Option<Self> {
        let iv = self.reduce();
        let (a, b) = (iv.inf(), iv.sup());
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        let zero = F::ZERO;
        let (ta_lo, ta_hi) = (a.tanh_down(), a.tanh_up());
        let (tb_lo, tb_hi) = (b.tanh_down(), b.tanh_up());
        let width_dn = b.sub_down(a);
        if width_dn <= zero {
            return Some(self.linearize(zero, ta_lo, ta_hi, src));
        }
        // Concave on the nonnegative half-line, convex on the nonpositive; a range
        // straddling zero declines.
        let convex = if a >= zero {
            false
        } else if b <= zero {
            true
        } else {
            return None;
        };
        let d = chord_depth(F::ONE, b.sub_up(a));
        let alpha = tb_hi.sub_up(ta_lo).div_up(width_dn);
        if !alpha.is_finite() {
            return None;
        }
        let ra = residual_endpoint(ta_lo, ta_hi, alpha, a);
        let rb = residual_endpoint(tb_lo, tb_hi, alpha, b);
        self.chord_fit(alpha, ra, rb, d, convex, src)
    }
}

impl<'id, F: RoundFloat + RoundTranscendental> AffineForm<'id, F> {
    /// The power `x̂^y` for a scalar exponent `y`, by the `exp`/`ln` composition on
    /// the positive domain: `x^y = exp(y · ln x)`.
    ///
    /// This is the affine seam of workspace decision record 0005 part 4: rather
    /// than a bespoke bivariate fit, the v1 power rides the existing affine
    /// [`ln`](AffineForm::ln), [`scale`](AffineForm::scale), and
    /// [`exp`](AffineForm::exp). The composition is the recorded frugal choice; its
    /// cost is width, three fresh noise symbols (one per stage) and three
    /// compounded outward bands rather than the two of a direct fit, which a later
    /// bivariate `pow` would recover if the width proves material.
    ///
    /// Returns `None` whenever a stage declines: a reduced range reaching zero or
    /// below has no `ln` (the real power is defined only for a positive base), and
    /// an exponent large enough to overflow `exp` exceeds the representable range.
    /// The interval arm, which owns the IEEE 1788 set semantics of the base and
    /// exponent edges, carries those cases.
    #[must_use]
    pub fn pow_scalar(&self, y: F, src: &mut SymbolSource<'id>) -> Option<Self> {
        let ln_x = self.ln(src)?;
        let scaled = ln_x.scale(y, src);
        scaled.exp(src)
    }
}
