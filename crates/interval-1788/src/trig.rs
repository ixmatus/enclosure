//! Trigonometric, hyperbolic, and power functions over intervals: the growth
//! past the monotone elementary arms.
//!
//! These functions require capabilities beyond the core [`RoundFloat`]: the
//! trigonometric arms ride [`RoundTrig`] (which carries the pi enclosure and an
//! exact `floor` for argument reduction), the hyperbolic arms ride
//! [`RoundHyperbolic`], and `pow` and `rootn` ride [`RoundPow`]. A backend
//! supplies each family on its own schedule; everything else about the interval
//! type is unaffected. See workspace decision record 0005.
//!
//! # The critical-point rule (sin, cos, tan)
//!
//! `sin` and `cos` are periodic and non-monotone, so the image of `[a, b]` is not
//! two endpoint evaluations: it is the endpoints together with any interior
//! extremum. This module implements the rule ADR-0005 fixes, derived from that
//! statement rather than adapted from any implementation (the ITF1788 vectors are
//! the conformance pin):
//!
//! 1. An input spanning at least one full period maps to the full range, `[-1, 1]`
//!    for sin and cos, immediately. An unbounded input spans infinitely many
//!    periods and does the same.
//! 2. Otherwise the maxima (`+1`) and minima (`-1`) each sit on a grid spaced one
//!    period apart; the arm tests whether a grid point's SOUND ENCLOSURE (built
//!    from the pi bracket and a directed index) meets `[a, b]`.
//! 3. **Ambiguity widens, never guesses.** A grid enclosure that meets `[a, b]`
//!    admits its extremum whether or not the true critical point lies inside. The
//!    cost is tightness within an enclosure's width of a critical point; the
//!    benefit is that no rounding of the reduction can produce a non-enclosure.
//! 4. The endpoints contribute `f_down(a) rmin f_down(b)` and
//!    `f_up(a) rmax f_up(b)`, and a final range clamp intersects the result with
//!    `[-1, 1]` (a theorem about the range, always sound to apply).
//!
//! `tan` adds poles: an interval whose interior may contain a pole
//! (`pi/2 + k*pi`) returns `Entire`; otherwise it is monotone increasing on the
//! branch, `[tan_down(a), tan_up(b)]`. The decoration transitions follow the
//! ITF1788 `libieeep1788_elem.itl` `tan` vectors.
//!
//! # pow by monotonicity, four-corner style
//!
//! `pow(X, Y)` is the set `{ x^y : x in X, x >= 0, y in Y }` (the exp/ln
//! composition is the AFFINE seam of ADR-0005 part 4, not this arm; part 3 fixes
//! the interval arm as monotone decomposition through the backend's directed
//! `pow`). On the positive-base region `x^y` is monotone in each argument for
//! each fixed value of the other, so the image extremes sit at corners of the
//! box and the four-corner reduction computes them; see
//! [`pow`](Interval::pow) for the derived sign-case table. The base's zero
//! column is folded in separately: `0^y = 0` is defined only for `y > 0`, so a
//! base reaching zero admits `0` into the image exactly when the exponent has a
//! strictly positive part. A negative base is outside the real domain and drops
//! out.

use crate::interval::Interval;
use round_float::{RoundFloat, RoundHyperbolic, RoundPow, RoundTrig};

/// Whether the arithmetic grid `base + m*step` has a point whose sound enclosure
/// intersects `[a, b]`.
///
/// `a` and `b` are finite; `base_lo`/`base_hi` bracket the phase offset and
/// `step_lo`/`step_hi` bracket a strictly positive step, each as a directed pair.
/// The index `m` with `a <= base + m*step <= b` satisfies
/// `(a - base)/step <= m <= (b - base)/step`; the fence floors an outward-rounded
/// version of each bound and widens it by one on each side, so no true grid point
/// is skipped even after the enclosure padding. Each candidate's enclosure is then
/// tested against `[a, b]`; an enclosure that only *might* contain a grid point
/// still counts (ambiguity widens). A pathologically wide fence (only reachable at
/// magnitudes where argument reduction has already lost all meaning) bails to
/// `true`, the sound answer for every caller (admit the extremum, or the pole).
fn grid_meets<F: RoundFloat + RoundTrig>(
    a: F,
    b: F,
    base_lo: F,
    base_hi: F,
    step_lo: F,
    step_hi: F,
) -> bool {
    let zero = F::ZERO;
    // Lower bound on (a - base)/step: subtract the largest base, divide by the
    // step endpoint that makes the quotient smallest.
    let num_lo = a.sub_down(base_hi);
    let q_lo = if num_lo < zero {
        num_lo.div_down(step_lo)
    } else {
        num_lo.div_down(step_hi)
    };
    // Upper bound on (b - base)/step.
    let num_hi = b.sub_up(base_lo);
    let q_hi = if num_hi < zero {
        num_hi.div_up(step_hi)
    } else {
        num_hi.div_up(step_lo)
    };
    let m_lo = q_lo.floor().sub_down(F::ONE);
    let m_hi = q_hi.floor().add_up(F::ONE);
    if !(m_lo.is_finite() && m_hi.is_finite()) {
        return true;
    }

    let mut m = m_lo;
    let mut iters = 0usize;
    while m <= m_hi {
        // At most a handful of candidates once the period shortcut has fired; a
        // larger count is a rounding pathology, and `true` is the sound bail.
        if iters >= 12 {
            return true;
        }
        // Enclosure of `base + m*step` with directed rounding on the sign of m.
        let (ms_lo, ms_hi) = if m < zero {
            (m.mul_down(step_hi), m.mul_up(step_lo))
        } else {
            (m.mul_down(step_lo), m.mul_up(step_hi))
        };
        let g_lo = base_lo.add_down(ms_lo);
        let g_hi = base_hi.add_up(ms_hi);
        if g_lo <= b && a <= g_hi {
            return true;
        }
        m = m.add_up(F::ONE);
        iters += 1;
    }
    false
}

impl<F: RoundFloat + RoundTrig> Interval<F> {
    /// The sine, the set `{ sin(x) : x in self }`.
    ///
    /// Non-monotone: the image follows the critical-point rule (see the
    /// [module docs](crate::trig)). An unbounded input, or one spanning a full
    /// period, maps to the full range `[-1, 1]`.
    #[must_use]
    pub fn sin(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let neg_one = F::ONE.negate();
        if !a.is_finite() || !b.is_finite() {
            return Self::hull(neg_one, F::ONE);
        }
        let pd = F::pi_down();
        let pu = F::pi_up();
        let two_lo = F::ONE.add_down(F::ONE);
        let two_hi = F::ONE.add_up(F::ONE);
        let twopi_lo = pd.add_down(pd);
        let twopi_hi = pu.add_up(pu);
        // Full-period shortcut: the width computed up reaches a full period.
        if b.sub_up(a) >= twopi_lo {
            return Self::hull(neg_one, F::ONE);
        }
        // Phase offsets: maxima at pi/2 + 2*m*pi, minima at 3*pi/2 + 2*m*pi.
        let hpi_lo = pd.div_down(two_hi);
        let hpi_hi = pu.div_up(two_lo);
        let thpi_lo = pd.add_down(hpi_lo);
        let thpi_hi = pu.add_up(hpi_hi);
        let has_max = grid_meets(a, b, hpi_lo, hpi_hi, twopi_lo, twopi_hi);
        let has_min = grid_meets(a, b, thpi_lo, thpi_hi, twopi_lo, twopi_hi);

        let mut lo = a.sin_down().rmin(b.sin_down());
        let mut hi = a.sin_up().rmax(b.sin_up());
        if has_min {
            lo = neg_one;
        }
        if has_max {
            hi = F::ONE;
        }
        // Range clamp: sin's image lies in [-1, 1] (a theorem, not an
        // approximation), so intersecting with it is always sound and restores
        // tightness where a loose backend pushed a bound past an extremum.
        Self::hull(lo.rmax(neg_one), hi.rmin(F::ONE))
    }

    /// The cosine, the set `{ cos(x) : x in self }`.
    ///
    /// Non-monotone, handled by the same critical-point rule as [`sin`](Interval::sin);
    /// cosine's maxima sit at `2*m*pi` and minima at `pi + 2*m*pi`.
    #[must_use]
    pub fn cos(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let neg_one = F::ONE.negate();
        if !a.is_finite() || !b.is_finite() {
            return Self::hull(neg_one, F::ONE);
        }
        let zero = F::ZERO;
        let pd = F::pi_down();
        let pu = F::pi_up();
        let twopi_lo = pd.add_down(pd);
        let twopi_hi = pu.add_up(pu);
        if b.sub_up(a) >= twopi_lo {
            return Self::hull(neg_one, F::ONE);
        }
        // Maxima at 2*m*pi (phase 0), minima at pi + 2*m*pi (phase pi).
        let has_max = grid_meets(a, b, zero, zero, twopi_lo, twopi_hi);
        let has_min = grid_meets(a, b, pd, pu, twopi_lo, twopi_hi);

        let mut lo = a.cos_down().rmin(b.cos_down());
        let mut hi = a.cos_up().rmax(b.cos_up());
        if has_min {
            lo = neg_one;
        }
        if has_max {
            hi = F::ONE;
        }
        Self::hull(lo.rmax(neg_one), hi.rmin(F::ONE))
    }

    /// The tangent, the set `{ tan(x) : x in self, x not a pole }`.
    ///
    /// An interval whose interior may contain a pole `pi/2 + k*pi` returns
    /// `Entire` (the sound response to an unbounded, discontinuous image); an
    /// unbounded input, or one spanning a full period, does the same. Otherwise
    /// `tan` is monotone increasing on the branch, so the image is
    /// `[tan_down(a), tan_up(b)]`.
    #[must_use]
    pub fn tan(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        if !a.is_finite() || !b.is_finite() {
            return Self::entire();
        }
        let pd = F::pi_down();
        let pu = F::pi_up();
        let two_lo = F::ONE.add_down(F::ONE);
        let two_hi = F::ONE.add_up(F::ONE);
        // A width of a full period (pi for tan) certainly contains a pole.
        if b.sub_up(a) >= pd {
            return Self::entire();
        }
        // Poles at pi/2 + k*pi: phase pi/2, step pi.
        let hpi_lo = pd.div_down(two_hi);
        let hpi_hi = pu.div_up(two_lo);
        if grid_meets(a, b, hpi_lo, hpi_hi, pd, pu) {
            return Self::entire();
        }
        Self::hull(a.tan_down(), b.tan_up())
    }
}

impl<F: RoundFloat + RoundHyperbolic> Interval<F> {
    /// The hyperbolic sine, the set `{ sinh(x) : x in self }`.
    ///
    /// Monotone increasing on the whole line, so the image of `[a, b]` is
    /// `[sinh_down(a), sinh_up(b)]`; an unbounded endpoint maps to the matching
    /// infinity, and an argument past the overflow shoulder to that infinity too.
    #[must_use]
    pub fn sinh(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let lo = if a.is_infinite() {
            F::NEG_INFINITY
        } else {
            a.sinh_down()
        };
        let hi = if b.is_infinite() {
            F::INFINITY
        } else {
            b.sinh_up()
        };
        Self::hull(lo, hi)
    }

    /// The hyperbolic cosine, the set `{ cosh(x) : x in self }`.
    ///
    /// Even with a single minimum `cosh(0) = 1`: when `0` lies in `[a, b]` the
    /// lower endpoint is exactly `1`, otherwise it is the value at the endpoint
    /// nearer zero. The upper endpoint is the value at the endpoint farther from
    /// zero (cosh grows with `|x|`).
    #[must_use]
    pub fn cosh(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        let one = F::ONE;
        let hi_a = if a.is_infinite() {
            F::INFINITY
        } else {
            a.cosh_up()
        };
        let hi_b = if b.is_infinite() {
            F::INFINITY
        } else {
            b.cosh_up()
        };
        let hi = hi_a.rmax(hi_b);
        let lo = if a <= zero && zero <= b {
            // The minimum cosh(0) = 1 is attained.
            one
        } else if b < zero {
            // Both endpoints negative: cosh is decreasing, minimum at b.
            b.cosh_down()
        } else {
            // Both positive: cosh is increasing, minimum at a.
            a.cosh_down()
        };
        // Range clamp: cosh >= 1 is a theorem.
        Self::hull(lo.rmax(one), hi)
    }

    /// The hyperbolic tangent, the set `{ tanh(x) : x in self }`.
    ///
    /// Monotone increasing with range `(-1, 1)`, so the image of `[a, b]` is
    /// `[tanh_down(a), tanh_up(b)]` clamped into `[-1, 1]`; an unbounded endpoint
    /// maps to the range edge it approaches.
    #[must_use]
    pub fn tanh(self) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        let neg_one = F::ONE.negate();
        let lo = if a.is_infinite() {
            neg_one
        } else {
            a.tanh_down()
        };
        let hi = if b.is_infinite() { F::ONE } else { b.tanh_up() };
        // Range clamp: tanh's image lies strictly in (-1, 1), so the closed clamp
        // to [-1, 1] is sound.
        Self::hull(lo.rmax(neg_one), hi.rmin(F::ONE))
    }
}

impl<F: RoundFloat + RoundPow> Interval<F> {
    /// The power, the set `{ x^y : x in self, x >= 0, y in exponent }`.
    ///
    /// IEEE 1788's `pow` restricts the base to the nonnegative reals; a base
    /// reaching below zero has that part dropped, and a base entirely below zero
    /// has the empty image.
    ///
    /// # The sign-case monotonicity table, and why four corners subsume it
    ///
    /// On the positive-base region the partial derivatives fix the monotonicity
    /// in each argument: `d/dx (x^y) = y * x^(y-1)`, so `x^y` increases in `x`
    /// for `y > 0`, decreases for `y < 0`, and is constant (`= 1`) at `y = 0`;
    /// `d/dy (x^y) = x^y * ln(x)`, so it increases in `y` for `x > 1`, decreases
    /// for `x < 1`, and is constant at `x = 1`. A function monotone in each
    /// argument for each fixed value of the other attains its extremes over a
    /// box at corners (fix `y`: the extreme moves to an `x` edge; move along
    /// that edge in `y`: it lands on a corner). Spelled out per sign case over
    /// `[xl, xh] x [yl, yh]`, the minimum sits at `(xl, yl)` when
    /// `yl >= 0, xl >= 1`, at `(xl, yh)` when `yl >= 0, xl < 1`, at `(xh, yl)`
    /// when `yh <= 0, xh > 1`, at `(xh, yh)` when `yh <= 0, xh <= 1`, and at the
    /// smaller of the two half-case corners when `y` straddles zero; the maximum
    /// table mirrors it. Evaluating all four `pow_down` corners under `rmin`
    /// (and `pow_up` under `rmax`) computes exactly those selections without
    /// dispatching on the cases, the same four-corner reduction multiplication
    /// uses, immune to a case-selection slip by construction.
    ///
    /// A base endpoint at or below zero enters the corners as `+0`, whose
    /// backend `pow` values are the limits of `x^y` as `x -> 0+` (zero for
    /// `y > 0`, unbounded above for `y < 0`, one at `y = 0`), so the open
    /// boundary of the domain-restricted set stays enclosed. The value `0`
    /// itself (from `0^y = 0`, defined only for `y > 0`) joins the image exactly
    /// when the base reaches zero and the exponent has a strictly positive part
    /// (`0^0` and `0^negative` are undefined and contribute nothing).
    #[must_use]
    pub fn pow(self, exponent: Self) -> Self {
        let Some((xl, xh)) = self.bounds() else {
            return Self::empty();
        };
        let Some((yl, yh)) = exponent.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if xh < zero {
            // Base entirely negative: outside the real power domain.
            return Self::empty();
        }
        // Whether 0^y contributes the value 0 (base reaches zero, exponent has a
        // strictly positive part).
        let zero_column = xl <= zero && yh > zero;
        if xh <= zero {
            // The base's positive part is empty (the base is the zero singleton,
            // up to zero signs): only the zero column can contribute.
            return if zero_column {
                Self::hull(zero, zero)
            } else {
                Self::empty()
            };
        }
        // The positive base part is [pxl, xh] with pxl normalized to +0 when the
        // base reaches zero or below (-0 included), so the corners the backend
        // evaluates at zero are the x -> 0+ limits, never the -0 special cases.
        let pxl = if xl <= zero { zero } else { xl };
        let lo = pxl
            .pow_down(yl)
            .rmin(pxl.pow_down(yh))
            .rmin(xh.pow_down(yl))
            .rmin(xh.pow_down(yh))
            // Range clamp: x^y > 0 on the positive base part and the x -> 0+
            // limits are at least 0, so a loose backend's widening below zero is
            // clamped back (a theorem about the range, always sound).
            .rmax(zero);
        let hi = pxl
            .pow_up(yl)
            .rmax(pxl.pow_up(yh))
            .rmax(xh.pow_up(yl))
            .rmax(xh.pow_up(yh));
        if zero_column {
            Self::hull(zero, hi)
        } else {
            Self::hull(lo, hi)
        }
    }

    /// The `n`-th root, the set `{ x^(1/n) : x in self, in domain }`.
    ///
    /// Splits on the parity of `n`: an odd root is defined and increasing on the
    /// whole line, so the image is `[rootn_down(a), rootn_up(b)]`; an even root is
    /// defined on `[0, +inf)`, so the negative part of the base is dropped and an
    /// input entirely below zero has the empty image. `n == 0` (the undefined
    /// zeroth root) yields the empty image.
    #[must_use]
    pub fn rootn(self, n: u32) -> Self {
        let Some((a, b)) = self.bounds() else {
            return Self::empty();
        };
        if n == 0 {
            return Self::empty();
        }
        let zero = F::ZERO;
        if n % 2 == 0 {
            if b < zero {
                return Self::empty();
            }
            let lo_arg = if a < zero { zero } else { a };
            let lo = lo_arg.rootn_down(n).rmax(zero);
            let hi = if b.is_infinite() {
                F::INFINITY
            } else {
                b.rootn_up(n)
            };
            Self::hull(lo, hi)
        } else {
            let lo = if a.is_infinite() {
                F::NEG_INFINITY
            } else {
                a.rootn_down(n)
            };
            let hi = if b.is_infinite() {
                F::INFINITY
            } else {
                b.rootn_up(n)
            };
            Self::hull(lo, hi)
        }
    }
}

// --- Decorated wrappers -----------------------------------------------------
//
// Built through the public `DecoratedInterval` surface (`interval`, `decoration`,
// `set_dec`, `is_nai`, `nai`) so the shared `decorated` module stays untouched.
// The local-decoration logic mirrors `decorated`'s: `com` when the operation is
// defined and continuous on a bounded nonempty input with a bounded result, `dac`
// when defined and continuous but somewhere unbounded, `trv` on a domain
// violation. sin/cos/sinh/cosh/tanh are defined and continuous everywhere, so
// they never grade `trv` from the operation; `tan` grades `trv` when a pole makes
// the image entire; `pow`/`rootn` grade `trv` on a base domain violation.

use crate::decorated::DecoratedInterval;
use crate::decoration::Decoration;

/// Whether an interval is bounded and nonempty (the precondition for `com`).
#[inline]
fn bounded_nonempty<F: RoundFloat>(i: Interval<F>) -> bool {
    i.is_bounded() && !i.is_empty()
}

/// The local decoration from whether the operation is defined and continuous on
/// the input box and whether every input is bounded and nonempty with a bounded
/// result.
#[inline]
fn local(defined_continuous: bool, all_bounded: bool) -> Decoration {
    if !defined_continuous {
        Decoration::Trv
    } else if all_bounded {
        Decoration::Com
    } else {
        Decoration::Dac
    }
}

impl<F: RoundFloat + RoundTrig> DecoratedInterval<F> {
    /// Sine, decorated. Defined and continuous everywhere, so it demotes only for
    /// an unbounded input.
    #[must_use]
    pub fn sin(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.sin();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Cosine, decorated. Defined and continuous everywhere.
    #[must_use]
    pub fn cos(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.cos();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Tangent, decorated. Undefined at each pole, so an interval whose image is
    /// unbounded (a pole in the interior, or an unbounded input) grades `trv`; a
    /// pole-free bounded interval stays `com`.
    #[must_use]
    pub fn tan(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.tan();
        // `tan` returns entire exactly when a pole enclosure meets the input (or
        // the input is unbounded): a domain violation.
        let dloc = if !x.is_empty() && result.is_entire() {
            Decoration::Trv
        } else {
            local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded())
        };
        Self::set_dec(result, self.decoration().meet(dloc))
    }
}

impl<F: RoundFloat + RoundHyperbolic> DecoratedInterval<F> {
    /// Hyperbolic sine, decorated. Defined and continuous everywhere; demotes to
    /// `dac` for an unbounded input or an overflowed (unbounded) result.
    #[must_use]
    pub fn sinh(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.sinh();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Hyperbolic cosine, decorated. Defined and continuous everywhere.
    #[must_use]
    pub fn cosh(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.cosh();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }

    /// Hyperbolic tangent, decorated. Defined and continuous everywhere; the image
    /// is always bounded, so it demotes to `dac` only for an unbounded input.
    #[must_use]
    pub fn tanh(self) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.tanh();
        let dloc = local(!x.is_empty(), bounded_nonempty(x) && result.is_bounded());
        Self::set_dec(result, self.decoration().meet(dloc))
    }
}

impl<F: RoundFloat + RoundPow> DecoratedInterval<F> {
    /// Power, decorated. The power is defined and continuous on `base > 0` (all
    /// exponents) and at `base == 0` for `exponent > 0`; it grades `trv` when the
    /// input box reaches a base below zero, or reaches base zero with an exponent
    /// that is not strictly positive (`0^0` and `0^negative` are undefined).
    #[must_use]
    pub fn pow(self, exponent: Self) -> Self {
        if self.is_nai() || exponent.is_nai() {
            return Self::nai();
        }
        let base = self.interval();
        let exp = exponent.interval();
        let result = base.pow(exp);
        let zero = F::ZERO;
        let (xl, xh) = (base.inf(), base.sup());
        let yl = exp.inf();
        let defined_continuous = if base.is_empty() || exp.is_empty() || xh < zero || xl < zero {
            // An empty operand, or a base reaching strictly below zero (the whole
            // interval when `xh < zero`, or part of it when `xl < zero`): outside
            // the real power domain.
            false
        } else if xl <= zero {
            // Base touches zero (no negative part): defined only if every exponent
            // is strictly positive.
            yl > zero
        } else {
            true
        };
        let all_bounded = bounded_nonempty(base) && bounded_nonempty(exp) && result.is_bounded();
        let dloc = local(defined_continuous, all_bounded);
        Self::set_dec(
            result,
            self.decoration().meet(exponent.decoration()).meet(dloc),
        )
    }
}

impl<F: RoundFloat + RoundPow> DecoratedInterval<F> {
    /// The `n`-th root, decorated. An odd root is defined and continuous
    /// everywhere; an even root is defined on `[0, +inf)`, so an input reaching
    /// below zero grades `trv`.
    #[must_use]
    pub fn rootn(self, n: u32) -> Self {
        if self.is_nai() {
            return Self::nai();
        }
        let x = self.interval();
        let result = x.rootn(n);
        let zero = F::ZERO;
        let defined_continuous = if n == 0 {
            false
        } else if n % 2 == 0 {
            !x.is_empty() && x.inf() >= zero
        } else {
            !x.is_empty()
        };
        let dloc = local(
            defined_continuous,
            bounded_nonempty(x) && result.is_bounded(),
        );
        Self::set_dec(result, self.decoration().meet(dloc))
    }
}
