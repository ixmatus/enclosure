//! Reverse operations: the constraint-narrowing primitives of IEEE 1788-2015.
//!
//! A reverse operation answers the question the forward battery cannot: given
//! that `f(t)` lands in a constraint `c`, where in a candidate set `x` can `t`
//! have been? Every unary reverse is the hull of a reverse image intersected
//! with the candidate,
//!
//! ```text
//! f_rev(c, x) = hull({ t in x : f(t) in c }),
//! ```
//!
//! and the binary and ternary forms quantify over the extra operand:
//! `mul_rev(b, c, x) = hull({ t in x : exists b0 in b, b0 * t in c })`, with
//! `pow_rev1`/`pow_rev2` re-reading the same shape for each argument of `pow`.
//! Every arm below is a derivation from that one definition; see decision
//! record 0006.
//!
//! # The candidate is always explicit
//!
//! The standard's candidate-less reverse is this operation at `x = entire`, so
//! the crate ships only the full form. The method receiver is the standard's
//! first argument and the rest follow in order: `c.sqr_rev(x)`,
//! `b.mul_rev(c, x)`, `b.mul_rev_to_pair(c)`, `b.pow_rev1(c, x)`,
//! `a.pow_rev2(c, x)`, `c.pown_rev(x, n)`.
//!
//! # Tightness doctrine
//!
//! The arithmetic reverses (`sqr_rev`, `abs_rev`, `pown_rev`, `mul_rev`,
//! `mul_rev_to_pair`) are computed to the tightest representable result over any
//! [`RoundFloat`] from directed arithmetic; `abs_rev` and the structural arms
//! are exact, and `sqr_rev` is exact but for the directed `sqrt`. `pown_rev`
//! rides the exact [`RoundPown`] integer kernel: its
//! roots bisect on the float grid over the correctly-rounded directed pair and
//! end with a neighbor certification that decides each candidate float's power
//! against the constraint exactly, so the positive-exponent result is correctly
//! rounded for exponent magnitude at most `RoundPown::POWN_TIGHT_MAX` and a
//! sound enclosure beyond. The negative exponents root first then reciprocate at
//! the scalar level (never a saturating set-level reciprocal); the pair is
//! correctly rounded where the constraint reciprocal is representable, and where
//! it overflows the directed root-then-reciprocal seed stands, matching the
//! reference implementation's own degraded `rootn` at that subnormal edge (the
//! value the corpus pins, one ulp inside the tightest floor for a
//! non-power-of-two root). The transcendental
//! reverses (`sin_rev`, `cos_rev`, `tan_rev`, `cosh_rev`, `pow_rev1`,
//! `pow_rev2`) inherit the fixture margins of the traits that fund them: sound
//! and near-tight, never claimed tightest over the sound-not-tight fixture.
//! Every arm soundly encloses the true reverse image; the property lane
//! discharges that against the forward battery alone.
//!
//! # The periodic preimage
//!
//! `sin_rev`, `cos_rev`, and `tan_rev` have preimages that are countable unions
//! of arcs placed by pi. The base arc comes from the directed inverse image on
//! the clipped constraint; arc `k` is the base arc translated by `k` periods
//! with the pi enclosure rounded outward, and the result hulls exactly the arcs
//! that can meet the candidate, jumping to a far window in logarithmically many
//! sound block additions rather than walking to it. When the candidate is
//! unbounded, spans more than `TRIG_REV_ARC_CAP` periods, or sits beyond the
//! smear horizon where accumulated pi-enclosure uncertainty reaches the gap
//! between adjacent arcs, the result degrades to the sound maximal preimage
//! clipped to the candidate (the candidate itself, since the arc union recurs
//! to both infinities). Degradation is a tightness loss, never a soundness
//! loss.

use crate::interval::Interval;
use crate::ops::odiv;
use round_float::{
    RoundFloat, RoundInverseHyperbolic, RoundInverseTrig, RoundPown, RoundTranscendental,
};

// --- Shared helpers ---------------------------------------------------------

/// `|x|` for a finite `x`, by a sign flip (covers `-0`).
#[inline]
fn fabs<F: RoundFloat>(x: F) -> F {
    if x.is_sign_negative() {
        x.negate()
    } else {
        x
    }
}

/// The exact ordering of `y^n` against a float `v`, for a finite `y > 0` and
/// `n != 0`, read from the correctly-rounded [`RoundPown`] directed pair.
///
/// Let `down = pown_down(y, n)` and `up = pown_up(y, n)`. For `1 <= |n| <=
/// POWN_TIGHT_MAX` the kernel is correctly rounded, so `down` and `up` are the
/// same float (`y^n` exactly representable) or adjacent floats straddling `y^n`.
/// A float `v` therefore cannot fall strictly between `down` and `up`, and the
/// sign of `y^n - v` is decided exactly:
///
/// - `up < v` => `y^n <= up < v`, so [`Less`](PowCmp::Less).
/// - `down > v` => `y^n >= down > v`, so [`Greater`](PowCmp::Greater).
/// - `down == up == v` => `y^n == v`, so [`Equal`](PowCmp::Equal).
/// - `down == v < up` => `y^n` is not a float and lies in `(down, up)`, above
///   `v`: [`Greater`](PowCmp::Greater).
/// - `down < v == up` => `y^n` lies in `(down, up)`, below `v`:
///   [`Less`](PowCmp::Less).
///
/// `None` is the straddle `down < v < up`, which the correct-rounding property
/// makes impossible for `|n| <= POWN_TIGHT_MAX`; it can arise only where the
/// kernel is sound but not tightest, and the caller then falls back to a sound
/// (non-certified) bracket.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PowCmp {
    Less,
    Equal,
    Greater,
}

/// Decide `y^n` against `v` per [`PowCmp`]. `y > 0` finite, `v` finite, `n != 0`.
#[inline]
fn pow_cmp<F: RoundFloat + RoundPown>(y: F, n: i32, v: F) -> Option<PowCmp> {
    let down = y.pown_down(n);
    let up = y.pown_up(n);
    if up < v {
        Some(PowCmp::Less)
    } else if down > v {
        Some(PowCmp::Greater)
    } else if down == up && down == v {
        Some(PowCmp::Equal)
    } else if down == v {
        // down == v < up: y^n in (down, up), strictly above v.
        Some(PowCmp::Greater)
    } else if up == v {
        // down < v == up: y^n in (down, up), strictly below v.
        Some(PowCmp::Less)
    } else {
        // down < v < up: only reachable when |n| > POWN_TIGHT_MAX.
        None
    }
}

/// Certify a sound root bracket to the correctly-rounded directed pair.
///
/// On entry `lo <= y <= hi` with `lo`, `hi` finite and `lo > 0`, where `y > 0`
/// is the real solution of `y^n = v` (`v > 0` finite, `n != 0`). The bracket is
/// bisected on the float grid until no interior probe remains (the ends are
/// adjacent floats, or equal), the exact decision [`pow_cmp`] steering each
/// step, and the two adjacent survivors are returned with any exactly
/// representable root collapsed to a point. For `|n| > POWN_TIGHT_MAX`, where
/// the kernel pair can straddle, the loop exits on the first straddle with the
/// current sound bracket (correct rounding is not claimed there).
///
/// `y^n` is increasing in `y` for `n > 0` and decreasing for `n < 0`, so a probe
/// `m` decided [`PowCmp::Less`] (`m^n < v`) lifts `lo` when increasing and lowers
/// `hi` when decreasing; the invariant kept is `lo <= y <= hi`. The `|n| >
/// POWN_TIGHT_MAX` straddle fallback (positive exponent only) exits sound.
fn refine_root<F: RoundFloat + RoundPown>(v: F, n: i32, mut lo: F, mut hi: F) -> (F, F) {
    // `lo` and `hi` are finite positive (non-NaN) here, so `>=` is total.
    if lo >= hi {
        return (lo, hi);
    }
    let one = F::ONE;
    let two = one.add_up(one);
    let half = one.div_up(two);
    let increasing = n > 0;
    let mut probes = 0;
    while probes < ROOT_PROBES {
        // Geometric probe while the bracket spans more than a factor of two
        // (halves the exponent gap), arithmetic once it is narrow (gains a
        // significand bit); `lo > 0` throughout.
        let wide = hi.is_finite() && hi > lo.add_up(lo);
        let mut m = if wide {
            lo.sqrt_down().mul_down(hi.sqrt_down())
        } else {
            lo.add_up(hi).mul_up(half)
        };
        if !(m > lo && m < hi) {
            m = lo.add_up(hi).mul_up(half);
            if !(m > lo && m < hi) {
                // No interior float: the ends are adjacent (or equal). Done.
                break;
            }
        }
        match pow_cmp(m, n, v) {
            Some(PowCmp::Equal) => return (m, m),
            Some(PowCmp::Less) => {
                // m^n < v: root above m when increasing, below when decreasing.
                if increasing {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            Some(PowCmp::Greater) => {
                if increasing {
                    hi = m;
                } else {
                    lo = m;
                }
            }
            None => {
                // |n| > POWN_TIGHT_MAX straddle: refine multiplicatively from
                // the straddle values (positive exponent only) and exit sound.
                if increasing {
                    let pu = m.pown_up(n);
                    let pd = m.pown_down(n);
                    if pu.is_finite() && !pu.is_zero() {
                        lo = lo.rmax(m.mul_down(v.div_down(pu)));
                    }
                    if pd.is_finite() && !pd.is_zero() {
                        hi = hi.rmin(m.mul_up(v.div_up(pd)));
                    }
                }
                break;
            }
        }
        probes += 1;
    }
    // Collapse an exactly representable root; otherwise the ends are the
    // correctly-rounded directed pair (or the sound straddle bracket).
    if pow_cmp(lo, n, v) == Some(PowCmp::Equal) {
        return (lo, lo);
    }
    if pow_cmp(hi, n, v) == Some(PowCmp::Equal) {
        return (hi, hi);
    }
    (lo, hi)
}

/// The bisection probe budget for a single integer root. The bisection runs in
/// two phases. While the bracket spans more than a factor of two, the probe is
/// the geometric mean of the ends (directed `sqrt` and multiply), which halves
/// the EXPONENT gap per probe: the widest bracket the seed loops produce spans
/// the full binary64 exponent range (about 2100 binades), so at most about 11
/// geometric probes reach a factor-two bracket regardless of where the root
/// sits. The arithmetic phase then gains one significand bit per probe and
/// walks the float grid all the way to adjacency, closing a factor-two bracket
/// in at most about 53 probes; 96 covers both phases with headroom for the
/// sloppy fixture's outward steps. Each probe is one correctly-rounded
/// [`RoundPown`] decision, so the whole root costs a bounded multiple of a
/// forward `pown`. The neighbor certification the arithmetic phase ends in
/// makes the pair correctly rounded for exponent magnitude at most
/// [`RoundPown::POWN_TIGHT_MAX`]; beyond it the kernel is sound but not
/// tightest and the bracket exits on its first straddle (see [`refine_root`]).
const ROOT_PROBES: u32 = 96;

/// Correctly-rounded lower and upper bounds on the real `n`-th root `v^(1/n)`
/// for `v >= 0` and `n >= 1`, by value-domain bisection over the exact
/// [`RoundPown`] pair (geometric probes while the bracket is wide, arithmetic
/// probes once it is narrow, ending in the neighbor certification of
/// [`refine_root`]; see [`ROOT_PROBES`]). The pair is the tightest representable
/// for `1 <= n <= POWN_TIGHT_MAX` and a sound enclosure beyond.
fn nth_root_bounds<F: RoundFloat + RoundPown>(v: F, n: u32) -> (F, F) {
    let zero = F::ZERO;
    let one = F::ONE;
    if v.is_zero() {
        return (zero, zero);
    }
    if v.is_infinite() {
        return (F::INFINITY, F::INFINITY);
    }
    if n == 1 {
        return (v, v);
    }
    // The exponent magnitude is small (the corpus reaches 8, `POWN_TIGHT_MAX` is
    // 64); a value near `u32::MAX` would already overflow every `pown`.
    #[allow(clippy::cast_possible_wrap)]
    let ni = n as i32;
    let two = one.add_up(one);
    let half = one.div_up(two);
    let (mut lo, mut hi);
    if v >= one {
        // Root is at least one; grow the upper bracket by doubling until its
        // `pown` LOWER bound reaches `v` (so `hi^n >= v`, i.e. `hi >= root`).
        lo = one;
        hi = two;
        while hi.pown_down(ni) < v {
            hi = hi.add_up(hi);
            if hi.is_infinite() {
                break;
            }
        }
    } else {
        // Root is below one; shrink the lower bracket by halving until its
        // `pown` UPPER bound drops to `v` (so `lo^n <= v`, i.e. `lo <= root`).
        hi = one;
        lo = half;
        while lo.pown_up(ni) > v {
            lo = lo.mul_down(half);
            if lo.is_zero() {
                return (zero, hi);
            }
        }
    }
    refine_root(v, ni, lo, hi)
}

/// Signed `n`-th root bounds for an odd `n` (monotone increasing, carrying the
/// sign): `(-v)^(1/n) = -(v^(1/n))`.
fn signed_root_bounds<F: RoundFloat + RoundPown>(v: F, n: u32) -> (F, F) {
    let zero = F::ZERO;
    if v < zero {
        let (d, u) = nth_root_bounds(fabs(v), n);
        (u.negate(), d.negate())
    } else {
        nth_root_bounds(v, n)
    }
}

/// Lower and upper bounds on the positive real `y = 1/v^(1/m)` that solves
/// `y^(-m) = v`, for `v > 0` finite and `m >= 1` (the magnitude of a negative
/// exponent). The upper bound is `+inf` exactly when the root overruns the
/// finite range.
///
/// The bracket is seeded root-first then reciprocated at the scalar level (the
/// magnitude root `v^(1/m)` of a representable `v` never overflows, and the
/// scalar `div` reaches values a set-level reciprocal of `v` would have
/// saturated past). Then, WHERE THE CONSTRAINT RECIPROCAL IS REPRESENTABLE, a
/// neighbor certification against the original `v` tightens the pair to
/// correctly rounded (bead enc-ral, normal-constraint negative exponents).
///
/// Where `1/v` overflows (a subnormal-reaching constraint), the certification is
/// abandoned and the directed root-then-reciprocal seed stands: the reference's
/// own `rootn` degrades by a rounding at that edge (`1/2^-1074` roots exactly to
/// `2^358` for exponent three, but the seventh root `2^(1074/7)` lands its
/// pair one ulp below the tightest floor), and the corpus pins that degraded
/// value, so the seed, not a tightening, is the conformant answer.
fn neg_root_bounds<F: RoundFloat + RoundPown>(v: F, m: u32) -> (F, F) {
    let one = F::ONE;
    #[allow(clippy::cast_possible_wrap)]
    let n = -(m as i32);
    let (w_d, w_u) = nth_root_bounds(v, m);
    let seed_lo = one.div_down(w_u);
    let seed_hi = one.div_up(w_d);
    // The constraint reciprocal overflows (or the root itself does): the
    // reference degrades here, so the directed seed is the conformant pair.
    if !seed_hi.is_finite() || !one.div_up(v).is_finite() {
        return (seed_lo, seed_hi);
    }
    // Otherwise certify to the correctly-rounded pair. `v` is representable with
    // a representable reciprocal, so the deciding powers `y^-m` stay in range.
    refine_root(v, n, seed_lo, seed_hi)
}

/// The two-output division of the standard's `mulRevToPair`: the reverse image
/// `{ t : exists b0 in b, b0 * t in c }` split into an ordered pair. The first
/// element is the lower piece and carries the whole result when the image is
/// connected; the second is empty unless the image genuinely splits (a
/// zero-straddling `b` with `0` not in `c`), in which case both pieces are
/// nonempty, disjoint, and the first is wholly below the second.
fn two_output_div<F: RoundFloat>(b: Interval<F>, c: Interval<F>) -> (Interval<F>, Interval<F>) {
    let (Some((b1, b2)), Some((c1, c2))) = (b.bounds(), c.bounds()) else {
        return (Interval::empty(), Interval::empty());
    };
    let zero = F::ZERO;
    let empty = Interval::empty();
    let c_has_zero = c1 <= zero && zero <= c2;
    let b_has_zero = b1 <= zero && zero <= b2;
    if c_has_zero {
        // Every t works through b0 = 0 when b also reaches zero; otherwise the
        // image is the ordinary (connected) quotient.
        if b_has_zero {
            return (Interval::entire(), empty);
        }
        return (odiv(c1, c2, b1, b2), empty);
    }
    if !b_has_zero {
        // 0 not in b, 0 not in c: a single ordinary quotient.
        return (odiv(c1, c2, b1, b2), empty);
    }
    if b1.is_zero() && b2.is_zero() {
        // b is exactly zero and c avoids zero: no t solves 0 * t in c.
        return (empty, empty);
    }
    // 0 in b (not exactly zero) and 0 not in c: the split. `c_near` is the
    // endpoint of c nearest zero, the only one that bounds the pieces.
    let (lower, upper) = if c1 > zero {
        // c > 0.
        let lower = if b1 < zero {
            Some(Interval::hull(F::NEG_INFINITY, c1.div_up(b1)))
        } else {
            None
        };
        let upper = if b2 > zero {
            Some(Interval::hull(c1.div_down(b2), F::INFINITY))
        } else {
            None
        };
        (lower, upper)
    } else {
        // c < 0.
        let lower = if b2 > zero {
            Some(Interval::hull(F::NEG_INFINITY, c2.div_up(b2)))
        } else {
            None
        };
        let upper = if b1 < zero {
            Some(Interval::hull(c2.div_down(b1), F::INFINITY))
        } else {
            None
        };
        (lower, upper)
    };
    match (lower, upper) {
        (Some(lo), Some(hi)) => (lo, hi),
        (Some(lo), None) => (lo, empty),
        (None, Some(hi)) => (hi, empty),
        (None, None) => (empty, empty),
    }
}

// --- Arithmetic reverses: sqr, abs, pown ------------------------------------

impl<F: RoundFloat> Interval<F> {
    /// The reverse of the square, `sqr_rev(c, x) = hull({ t in x : t^2 in c })`,
    /// with `self` the constraint `c`.
    ///
    /// The square is nonnegative, so the constraint is clipped to `[0, +inf)`
    /// first (an empty clip gives the empty result); the preimage of the clip is
    /// the symmetric pair `[-sqrt(sup), -sqrt(inf)]` and `[sqrt(inf),
    /// sqrt(sup)]`, each intersected with the candidate before the hull. Exact
    /// but for the directed `sqrt`.
    #[must_use]
    pub fn sqr_rev(self, x: Self) -> Self {
        let Some((c_lo, c_hi)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if c_hi < zero {
            return Self::empty();
        }
        let cc_lo = c_lo.rmax(zero);
        let r_lo = cc_lo.sqrt_down().rmax(zero);
        let r_hi = if c_hi.is_infinite() {
            F::INFINITY
        } else {
            c_hi.sqrt_up()
        };
        let pos = Self::hull(r_lo, r_hi);
        let neg = Self::hull(r_hi.negate(), r_lo.negate());
        pos.intersection(x).convex_hull(neg.intersection(x))
    }

    /// The reverse of the absolute value,
    /// `abs_rev(c, x) = hull({ t in x : |t| in c })`, with `self` the constraint.
    ///
    /// `|t|` is nonnegative, so the constraint clips to `[0, +inf)`; the preimage
    /// is the symmetric pair `[-sup, -inf]` and `[inf, sup]` of the clip. Exact
    /// (absolute value never rounds).
    #[must_use]
    pub fn abs_rev(self, x: Self) -> Self {
        let Some((c_lo, c_hi)) = self.bounds() else {
            return Self::empty();
        };
        let zero = F::ZERO;
        if c_hi < zero {
            return Self::empty();
        }
        let cc_lo = c_lo.rmax(zero);
        let pos = Self::hull(cc_lo, c_hi);
        let neg = Self::hull(c_hi.negate(), cc_lo.negate());
        pos.intersection(x).convex_hull(neg.intersection(x))
    }
}

impl<F: RoundFloat + RoundPown> Interval<F> {
    /// The reverse of the integer power,
    /// `pown_rev(c, x, n) = hull({ t in x : t^n in c })`, with `self` the
    /// constraint and the exponent `n` trailing as `pown`'s does.
    ///
    /// `n == 0` maps every base to `1`, so the reverse is the candidate when
    /// `1 in c` and empty otherwise. For `n > 0` an odd power is monotone (a
    /// single signed-root interval) and an even power folds through the
    /// magnitude (the symmetric root pair, with the constraint clipped to
    /// `[0, +inf)`). For `n < 0` the map `t^n = 1/t^m` is decreasing on each
    /// side of its pole at zero, and the reverse roots first then reciprocates
    /// at the scalar level (never a saturating set-level reciprocal). The roots
    /// ride the exact [`RoundPown`] kernel: the positive-exponent result and the
    /// negative-exponent result with a representable constraint reciprocal are
    /// correctly rounded for `|n| <= RoundPown::POWN_TIGHT_MAX`; a sound
    /// enclosure holds beyond, and at the subnormal edge where the constraint
    /// reciprocal overflows the negative path matches the reference's directed
    /// composition (see the [module docs](crate::reverse)).
    #[must_use]
    pub fn pown_rev(self, x: Self, n: i32) -> Self {
        if self.is_empty() {
            return Self::empty();
        }
        if n == 0 {
            return if self.contains(F::ONE) {
                x
            } else {
                Self::empty()
            };
        }
        if n > 0 {
            #[allow(clippy::cast_sign_loss)]
            pos_pown_rev(self, x, n as u32)
        } else {
            neg_pown_rev(self, x, n.unsigned_abs())
        }
    }
}

/// The reverse of `t^m` for a strictly positive exponent `m`, the shared core of
/// [`Interval::pown_rev`].
fn pos_pown_rev<F: RoundFloat + RoundPown>(c: Interval<F>, x: Interval<F>, m: u32) -> Interval<F> {
    let Some((c_lo, c_hi)) = c.bounds() else {
        return Interval::empty();
    };
    if m % 2 == 1 {
        // Odd: monotone increasing, so the preimage is a single signed-root
        // interval.
        let (lo, _) = signed_root_bounds(c_lo, m);
        let (_, hi) = signed_root_bounds(c_hi, m);
        Interval::hull(lo, hi).intersection(x)
    } else {
        // Even: t^m = |t|^m, driven by the magnitude range of the nonnegative
        // clip of the constraint.
        let zero = F::ZERO;
        if c_hi < zero {
            return Interval::empty();
        }
        let cc_lo = c_lo.rmax(zero);
        let (r_lo, _) = nth_root_bounds(cc_lo, m);
        let (_, r_hi) = nth_root_bounds(c_hi, m);
        let pos = Interval::hull(r_lo, r_hi);
        let neg = Interval::hull(r_hi.negate(), r_lo.negate());
        pos.intersection(x).convex_hull(neg.intersection(x))
    }
}

/// The reverse of `t^n` for a strictly negative exponent `n = -m`, the negative
/// arm of [`Interval::pown_rev`].
///
/// `t^n = 1/t^m` has a pole at zero and is DECREASING on each of `(-inf, 0)` and
/// `(0, inf)`; its range on each side is one open half-line. So, unlike the
/// increasing positive-odd case, the preimage generally splits into two pieces,
/// and the endpoint that a value maps to swaps sense: the LARGER constraint
/// value pins the SMALLER magnitude. Each magnitude is a [`neg_root_bounds`]
/// pair, `1/v^(1/m)` computed root-first then reciprocated so the subnormal edge
/// never overflows a set-level reciprocal.
///
/// The positive-t piece collects the constraint's positive values: its lower end
/// is the down-root of `c_hi` (or `0` when `c_hi` is unbounded, the value
/// growing without bound as `t -> 0+`), its upper end the up-root of `c_lo` (or
/// `+inf` when `c_lo` reaches zero, `t` growing without bound as the value tends
/// to `0+`). For even `m` the map is `1/|t|^m`, so the negative piece is the
/// mirror of the positive one through zero. For odd `m` the negative piece
/// handles the negative values through `s = -t > 0` (with `s^n in [-c_hi,
/// -c_lo]`, then `t = -s`), which is the positive-piece construction on the
/// negated constraint.
fn neg_pown_rev<F: RoundFloat + RoundPown>(c: Interval<F>, x: Interval<F>, m: u32) -> Interval<F> {
    let Some((c_lo, c_hi)) = c.bounds() else {
        return Interval::empty();
    };
    let zero = F::ZERO;
    let inf = F::INFINITY;
    // The positive-t piece, active only when the constraint reaches a positive
    // value (`t^n > 0` for `t > 0`).
    let positive = if c_hi > zero {
        let p_lo = if c_hi.is_infinite() {
            zero
        } else {
            neg_root_bounds(c_hi, m).0
        };
        let p_hi = if c_lo <= zero {
            inf
        } else {
            neg_root_bounds(c_lo, m).1
        };
        Interval::hull(p_lo, p_hi)
    } else {
        Interval::empty()
    };
    if m % 2 == 0 {
        // Even: the negative piece is the positive one reflected through zero.
        let neg = match positive.bounds() {
            Some((p_lo, p_hi)) => Interval::hull(p_hi.negate(), p_lo.negate()),
            None => Interval::empty(),
        };
        return positive.intersection(x).convex_hull(neg.intersection(x));
    }
    // Odd: the negative piece handles the constraint's negative values through
    // `s = -t`, the positive-piece construction on `[-c_hi, -c_lo]`.
    let negative = if c_lo < zero {
        let s_lo = if c_lo.is_infinite() {
            zero
        } else {
            neg_root_bounds(fabs(c_lo), m).0
        };
        let s_hi = if c_hi >= zero {
            inf
        } else {
            neg_root_bounds(fabs(c_hi), m).1
        };
        Interval::hull(s_hi.negate(), s_lo.negate())
    } else {
        Interval::empty()
    };
    positive
        .intersection(x)
        .convex_hull(negative.intersection(x))
}

// --- Multiplication reverses: mul_rev, mul_rev_to_pair ----------------------

impl<F: RoundFloat> Interval<F> {
    /// The reverse of multiplication in one operand,
    /// `mul_rev(b, c, x) = hull({ t in x : exists b0 in b, b0 * t in c })`, with
    /// `self` the known factor `b`.
    ///
    /// The reverse image is the two-output division of `c` by `b` (see
    /// [`mul_rev_to_pair`](Interval::mul_rev_to_pair)); each of its two pieces is
    /// intersected with the candidate before the hull, so a candidate that lands
    /// inside one piece recovers the tight one-sided result. Exact but for the
    /// directed division.
    #[must_use]
    pub fn mul_rev(self, c: Self, x: Self) -> Self {
        let (p1, p2) = two_output_div(self, c);
        p1.intersection(x).convex_hull(p2.intersection(x))
    }

    /// The standard's two-output division `mulRevToPair`: the reverse image
    /// `{ t : exists b0 in b, b0 * t in c }` as the ordered pair
    /// `(lower, upper)`, with `self` the known factor `b`.
    ///
    /// The first element carries the whole result when the image is connected;
    /// the second is nonempty only when the image genuinely splits (a
    /// zero-straddling `b` with `0` not in `c`), in which case both pieces are
    /// nonempty, disjoint, and the first is wholly below the second. That
    /// invariant is asserted in the property lane. Exact but for the directed
    /// division.
    #[must_use]
    pub fn mul_rev_to_pair(self, c: Self) -> (Self, Self) {
        two_output_div(self, c)
    }
}

// --- Trigonometric reverses: sin, cos, tan ----------------------------------

/// The arc-walk cap for the periodic reverses: the maximum number of arcs
/// enumerated ACROSS THE CANDIDATE WINDOW. A candidate spanning more than this
/// many periods degrades to the sound maximal preimage clipped to the
/// candidate (the preimage family recurs to both infinities, so the candidate
/// itself is always a sound answer); a narrower window enumerates exactly the
/// arcs that can meet it, wherever on the line it sits, because the landing
/// phase of [`enum_shifted_arc`] jumps to the window in logarithmically many
/// block additions rather than walking there arc by arc. Degradation from pi
/// uncertainty is separate and derived from the enclosures at run time (the
/// decision record's clause): arc `k`'s placement uncertainty is about `|k|`
/// times the period enclosure's width, and once a position enclosure is wider
/// than half a period adjacent arcs smear together, which over the `f64`
/// fixture's pi pair happens near `|k| ~ 10^15` periods (half a period over a
/// period-enclosure width of a few times `1e-15`). Both degradations lose
/// only tightness, never soundness.
const TRIG_REV_ARC_CAP: u32 = 1024;

/// Whether a position enclosure `[pos_lo, pos_hi]` has smeared: its width is
/// not certainly below the half-period threshold. The negated comparison is
/// deliberate (and why the clippy lint is silenced): an incomparable width
/// must count as smeared, so the degradation fires in the safe direction.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
#[inline]
fn smeared<F: RoundFloat>(pos_lo: F, pos_hi: F, smear: F) -> bool {
    !(pos_hi.sub_up(pos_lo) < smear)
}

/// Clip the arc `[a_lo, a_hi]` to the candidate `[xl, xh]`.
#[inline]
fn clip_arc<F: RoundFloat>(a_lo: F, a_hi: F, xl: F, xh: F) -> Interval<F> {
    let lo = a_lo.rmax(xl);
    let hi = a_hi.rmin(xh);
    if lo <= hi {
        Interval::hull(lo, hi)
    } else {
        Interval::empty()
    }
}

/// Hull the intersections of the arc family `arc + k*period` (`k` over the
/// integers) with the bounded candidate `[xl, xh]`.
///
/// Every placement is a directed interval sum: the position enclosure of the
/// current arc starts exactly at zero for arc 0 and moves only by adding
/// enclosures of `2^m` periods (built from the period enclosure by doubling),
/// so by induction it always encloses `k * period` for an exact integer `k`.
/// No float arc counter is ever multiplied or incremented, so no counter can
/// drift off the integers (the defect the previous walk had: repeated
/// `add_up(ONE)` drifts above `k`, and at large `k` the drift outgrows the
/// directed multiplication's outward step, misplacing the arc).
///
/// The phases: descend by doubled blocks until the arc is certainly below the
/// window (comparisons use the enclosure's outer bound, so "certainly" is
/// sound), gallop back up by the largest blocks that keep it certainly below,
/// then walk one period at a time across the window, clipping each arc into
/// the accumulator. `None` signals degradation: the candidate sits beyond the
/// pi-uncertainty smear horizon (upfront check, from the enclosures), a
/// position enclosure's width reaches half a period (the running form of the
/// same check, covering accumulated rounding), or the window spans more than
/// [`TRIG_REV_ARC_CAP`] periods.
fn enum_shifted_arc<F: RoundFloat>(
    arc_lo: F,
    arc_hi: F,
    step_lo: F,
    step_hi: F,
    xl: F,
    xh: F,
) -> Option<Interval<F>> {
    let zero = F::ZERO;
    let one = F::ONE;
    let two = one.add_up(one);
    // Half a period: once a position enclosure is this wide, adjacent arcs
    // smear together and enumeration buys nothing over the fallback.
    let smear = step_lo.div_down(two);
    // Upfront horizon, the decision record's degradation clause computed from
    // the shipped enclosures: the position uncertainty at arc k is about
    // |k| * unc, and |k| ~ |x| / period, so it reaches `smear` when |x| is
    // about (smear / unc) * period. Beyond that, degrade before landing.
    let unc = step_hi.sub_up(step_lo);
    if !unc.is_zero() {
        let horizon = smear.div_down(unc).mul_down(step_lo);
        if fabs(xl).rmax(fabs(xh)) >= horizon {
            return None;
        }
    }
    // Position enclosure of the current arc's translation offset; exact zero
    // for arc 0.
    let mut pos_lo = zero;
    let mut pos_hi = zero;
    // Whether the arc at a position with upper bound `p_hi` is certainly
    // entirely below the window.
    let below = |p_hi: F| arc_hi.add_up(p_hi) < xl;

    // Descend: if arc 0 is not certainly below the window, subtract one
    // doubled-period block big enough to land certainly below. Bounded by the
    // format's exponent range.
    if !below(pos_hi) {
        let mut nb_lo = step_hi.negate();
        let mut nb_hi = step_lo.negate();
        let mut guard = 0u32;
        loop {
            if below(pos_hi.add_up(nb_hi)) {
                pos_lo = pos_lo.add_down(nb_lo);
                pos_hi = pos_hi.add_up(nb_hi);
                break;
            }
            nb_lo = nb_lo.add_down(nb_lo);
            nb_hi = nb_hi.add_up(nb_hi);
            guard += 1;
            if guard > 1100 || !nb_lo.is_finite() {
                return None;
            }
        }
    }
    // Gallop back up: repeatedly take the largest doubled-period block that
    // keeps the arc certainly below the window. Each round more than halves
    // the remaining distance, so the rounds are logarithmic in it.
    let mut rounds = 0u32;
    while below(pos_hi.add_up(step_hi)) {
        let mut b_lo = step_lo;
        let mut b_hi = step_hi;
        loop {
            let d_lo = b_lo.add_down(b_lo);
            let d_hi = b_hi.add_up(b_hi);
            if !d_hi.is_finite() || !below(pos_hi.add_up(d_hi)) {
                break;
            }
            b_lo = d_lo;
            b_hi = d_hi;
        }
        pos_lo = pos_lo.add_down(b_lo);
        pos_hi = pos_hi.add_up(b_hi);
        rounds += 1;
        if rounds > 1100 {
            return None;
        }
    }
    // The running smear check at the landing point.
    if smeared(pos_lo, pos_hi, smear) {
        return None;
    }
    // Walk one period at a time across the window.
    let mut acc = Interval::empty();
    let mut steps = 0u32;
    loop {
        let a_lo = arc_lo.add_down(pos_lo);
        let a_hi = arc_hi.add_up(pos_hi);
        if a_lo > xh {
            // Certainly past the window; every later arc is higher still.
            break;
        }
        acc = acc.convex_hull(clip_arc(a_lo, a_hi, xl, xh));
        // Early exit: the hull already covers the whole window.
        if !acc.is_empty() && acc.inf() <= xl && acc.sup() >= xh {
            break;
        }
        pos_lo = pos_lo.add_down(step_lo);
        pos_hi = pos_hi.add_up(step_hi);
        if smeared(pos_lo, pos_hi, smear) {
            return None;
        }
        steps += 1;
        if steps > TRIG_REV_ARC_CAP {
            return None;
        }
    }
    Some(acc)
}

impl<F: RoundFloat + RoundInverseTrig> Interval<F> {
    /// The reverse of the sine,
    /// `sin_rev(c, x) = hull({ t in x : sin(t) in c })`, with `self` the
    /// constraint.
    ///
    /// The constraint is clipped to the sine range `[-1, 1]` (an empty clip
    /// gives the empty result); a full clip admits every real, so the candidate
    /// passes through. Otherwise the preimage is the union of the increasing-arc
    /// family `asin(c) + 2k*pi` and the decreasing-arc family
    /// `pi - asin(c) + 2k*pi`, each arc intersected with the candidate before
    /// the hull. An unbounded candidate, one spanning more than
    /// `TRIG_REV_ARC_CAP` periods, or one beyond the pi-uncertainty smear
    /// horizon degrades to the candidate itself (the sound maximal preimage).
    /// Sound and near-tight over the fixture.
    #[must_use]
    pub fn sin_rev(self, x: Self) -> Self {
        let Some((c_lo, c_hi)) = self.bounds() else {
            return Self::empty();
        };
        let one = F::ONE;
        let neg_one = one.negate();
        if c_hi < neg_one || c_lo > one {
            return Self::empty();
        }
        if c_lo <= neg_one && c_hi >= one {
            // sin can take any value in the clip: the preimage is all of R.
            return x;
        }
        let Some((xl, xh)) = x.bounds() else {
            return Self::empty();
        };
        if !xl.is_finite() || !xh.is_finite() {
            // Unbounded candidate: degrade to the maximal preimage clipped to x.
            return x;
        }
        let cc_lo = c_lo.rmax(neg_one);
        let cc_hi = c_hi.rmin(one);
        let a_lo = cc_lo.asin_down();
        let a_hi = cc_hi.asin_up();
        let pi_lo = <F as RoundInverseTrig>::pi_down();
        let pi_hi = <F as RoundInverseTrig>::pi_up();
        let tau_lo = pi_lo.add_down(pi_lo);
        let tau_hi = pi_hi.add_up(pi_hi);
        // Decreasing branch: t in [pi - asin(c_hi), pi - asin(c_lo)].
        let b_lo = pi_lo.sub_down(a_hi);
        let b_hi = pi_hi.sub_up(a_lo);
        match (
            enum_shifted_arc(a_lo, a_hi, tau_lo, tau_hi, xl, xh),
            enum_shifted_arc(b_lo, b_hi, tau_lo, tau_hi, xl, xh),
        ) {
            (Some(ra), Some(rb)) => ra.convex_hull(rb),
            _ => x,
        }
    }

    /// The reverse of the cosine,
    /// `cos_rev(c, x) = hull({ t in x : cos(t) in c })`, with `self` the
    /// constraint.
    ///
    /// The constraint clips to `[-1, 1]`; the preimage is the union of the
    /// principal-arc family `acos(c) + 2k*pi` and its reflection `-acos(c) +
    /// 2k*pi`. Degrades like [`sin_rev`](Interval::sin_rev). Sound and near-tight
    /// over the fixture.
    #[must_use]
    pub fn cos_rev(self, x: Self) -> Self {
        let Some((c_lo, c_hi)) = self.bounds() else {
            return Self::empty();
        };
        let one = F::ONE;
        let neg_one = one.negate();
        if c_hi < neg_one || c_lo > one {
            return Self::empty();
        }
        if c_lo <= neg_one && c_hi >= one {
            return x;
        }
        let Some((xl, xh)) = x.bounds() else {
            return Self::empty();
        };
        if !xl.is_finite() || !xh.is_finite() {
            return x;
        }
        let cc_lo = c_lo.rmax(neg_one);
        let cc_hi = c_hi.rmin(one);
        // acos is antitone: acos([cc_lo, cc_hi]) = [acos(cc_hi), acos(cc_lo)].
        let p_lo = cc_hi.acos_down();
        let p_hi = cc_lo.acos_up();
        let pi_lo = <F as RoundInverseTrig>::pi_down();
        let pi_hi = <F as RoundInverseTrig>::pi_up();
        let tau_lo = pi_lo.add_down(pi_lo);
        let tau_hi = pi_hi.add_up(pi_hi);
        // Reflected branch: t in [-acos(cc_lo), -acos(cc_hi)] = [-p_hi, -p_lo].
        let n_lo = p_hi.negate();
        let n_hi = p_lo.negate();
        match (
            enum_shifted_arc(p_lo, p_hi, tau_lo, tau_hi, xl, xh),
            enum_shifted_arc(n_lo, n_hi, tau_lo, tau_hi, xl, xh),
        ) {
            (Some(rp), Some(rn)) => rp.convex_hull(rn),
            _ => x,
        }
    }

    /// The reverse of the tangent,
    /// `tan_rev(c, x) = hull({ t in x : tan(t) in c })`, with `self` the
    /// constraint.
    ///
    /// The tangent's range is all of the reals, so the constraint is not clipped;
    /// an empty constraint gives the empty result. The preimage is the single
    /// arc family `atan(c) + k*pi` (period pi), each arc intersected with the
    /// candidate before the hull. Degrades like [`sin_rev`](Interval::sin_rev).
    /// Sound and near-tight over the fixture.
    #[must_use]
    pub fn tan_rev(self, x: Self) -> Self {
        let Some((c_lo, c_hi)) = self.bounds() else {
            return Self::empty();
        };
        let Some((xl, xh)) = x.bounds() else {
            return Self::empty();
        };
        if !xl.is_finite() || !xh.is_finite() {
            return x;
        }
        let a_lo = c_lo.atan_down();
        let a_hi = c_hi.atan_up();
        let pi_lo = <F as RoundInverseTrig>::pi_down();
        let pi_hi = <F as RoundInverseTrig>::pi_up();
        match enum_shifted_arc(a_lo, a_hi, pi_lo, pi_hi, xl, xh) {
            Some(r) => r,
            None => x,
        }
    }
}

// --- Hyperbolic reverse: cosh -----------------------------------------------

impl<F: RoundFloat + RoundInverseHyperbolic> Interval<F> {
    /// The reverse of the hyperbolic cosine,
    /// `cosh_rev(c, x) = hull({ t in x : cosh(t) in c })`, with `self` the
    /// constraint.
    ///
    /// `cosh` is at least one and even, so this is `sqr_rev`'s shape with `acosh`
    /// in place of `sqrt`: clip the constraint to `[1, +inf)`, take the symmetric
    /// preimage pair `[-acosh(sup), -acosh(inf)]` and `[acosh(inf),
    /// acosh(sup)]`, and intersect each with the candidate. Sound and near-tight
    /// over the fixture.
    #[must_use]
    pub fn cosh_rev(self, x: Self) -> Self {
        let Some((c_lo, c_hi)) = self.bounds() else {
            return Self::empty();
        };
        let one = F::ONE;
        if c_hi < one {
            return Self::empty();
        }
        let cc_lo = c_lo.rmax(one);
        let r_lo = cc_lo.acosh_down();
        let r_hi = if c_hi.is_infinite() {
            F::INFINITY
        } else {
            c_hi.acosh_up()
        };
        let pos = Self::hull(r_lo, r_hi);
        let neg = Self::hull(r_hi.negate(), r_lo.negate());
        pos.intersection(x).convex_hull(neg.intersection(x))
    }
}

// --- Power reverses: pow_rev1, pow_rev2 -------------------------------------
//
// pow(x1, x2) = x1^x2 for x1 >= 0. Writing u = ln(x1), the identity
// x1^x2 = exp(x2 * u) turns the power reverses into the exponential and
// logarithm of a multiplication reverse:
//
//   pow_rev1 (recover the base x1 given the exponent b and result c):
//       x2 * u in ln(c)  =>  u in mul_rev(b, ln(c), *)  =>  x1 = exp(u),
//   pow_rev2 (recover the exponent x2 given the base a and result c):
//       x2 * ln(a) in ln(c)  =>  x2 in mul_rev(ln(a), ln(c), x).
//
// The base t = 0 (0^e = 0 for e > 0) and t = 1 (1^e = 1) lie on the boundary of
// the logarithm and are added explicitly. The reduction is sound on every
// backend but only near-tight: at the open boundaries where a logarithm reaches
// an infinity the closed interval hull can over-reach by a point, so these arms
// are asserted as enclosure, never exact.

impl<F: RoundFloat + RoundTranscendental> Interval<F> {
    /// The reverse of `pow` in its base,
    /// `pow_rev1(b, c, x) = hull({ t in x : t >= 0, exists e in b, t^e in c })`,
    /// with `self` the known exponent `b`, `c` the result constraint, and `x` the
    /// base candidate.
    ///
    /// Computed from the exponential of a multiplication reverse (see the module
    /// derivation), with the `0^e` and `1^e` boundary bases added explicitly.
    /// Sound and near-tight over the fixture; the structural arms (empties,
    /// domain clips) are exact.
    #[must_use]
    pub fn pow_rev1(self, c: Self, x: Self) -> Self {
        let b = self;
        // The base is restricted to the nonnegative reals.
        let xb = x.intersection(Self::hull(F::ZERO, F::INFINITY));
        if b.is_empty() || c.is_empty() || xb.is_empty() {
            return Self::empty();
        }
        // The result of pow is nonnegative.
        let cc = c.intersection(Self::hull(F::ZERO, F::INFINITY));
        if cc.is_empty() {
            return Self::empty();
        }
        let zero = F::ZERO;
        let one = F::ONE;
        let mut result = Self::empty();
        // t = 0: 0^e = 0 for e > 0.
        if cc.contains(zero) && b.sup() > zero {
            result = result.convex_hull(Self::hull(zero, zero).intersection(xb));
        }
        // t = 1: 1^e = 1 for every e.
        if cc.contains(one) {
            result = result.convex_hull(Self::hull(one, one).intersection(xb));
        }
        // Generic t > 0: u = ln(t) in mul_rev(b, ln(c), entire); t = exp(u).
        let base_gen = b.mul_rev(cc.ln(), Self::entire()).exp();
        result.convex_hull(base_gen.intersection(xb))
    }

    /// The reverse of `pow` in its exponent,
    /// `pow_rev2(a, c, x) = hull({ t in x : exists s in a, s >= 0, s^t in c })`,
    /// with `self` the known base `a`, `c` the result constraint, and `x` the
    /// exponent candidate.
    ///
    /// Computed as `mul_rev(ln(a), ln(c), x)` (see the module derivation), with
    /// the `0^e` and `1^e` boundary bases added explicitly. Sound and near-tight
    /// over the fixture; the structural arms are exact.
    #[must_use]
    pub fn pow_rev2(self, c: Self, x: Self) -> Self {
        let a = self;
        if a.is_empty() || c.is_empty() || x.is_empty() {
            return Self::empty();
        }
        let zero = F::ZERO;
        let one = F::ONE;
        // The base is restricted to the nonnegative reals, the result likewise.
        let aa = a.intersection(Self::hull(zero, F::INFINITY));
        let cc = c.intersection(Self::hull(zero, F::INFINITY));
        if aa.is_empty() || cc.is_empty() {
            return Self::empty();
        }
        let mut result = Self::empty();
        // Base 0: 0^e = 0 for e > 0.
        if aa.contains(zero) && cc.contains(zero) {
            result = result.convex_hull(Self::hull(zero, F::INFINITY).intersection(x));
        }
        // Base 1: 1^e = 1 for every e.
        if aa.contains(one) && cc.contains(one) {
            result = result.convex_hull(x);
        }
        // Generic base s > 0: e = ln(c) / ln(s) in mul_rev(ln(a), ln(c), x).
        let e_gen = aa.ln().mul_rev(cc.ln(), x);
        result.convex_hull(e_gen)
    }
}

// --- Decorated twins --------------------------------------------------------
//
// A one-output reverse operation is a set-level answer that carries no witness
// that the forward function was defined and continuous on the result, so the
// standard's non-arithmetic rule assigns it no better than `trv`. Each such twin
// returns its bare result decorated `trv`, with `ill` (NaI) propagating: if any
// operand is NaI the result is NaI, otherwise the decoration is exactly `trv`
// (its meet with any input decoration). See decision record 0006 part 5.
//
// The two-output reverse division is the exception the standard carves out for
// itself (clause 12.12.3): its first piece carries the decoration of the normal
// division whenever zero lies outside the divisor, so `mul_rev_to_pair` grades
// that piece as division does rather than `trv`. See the decision record's
// erratum.

use crate::decorated::DecoratedInterval;
use crate::decoration::Decoration;

/// Decorate a bare reverse result `trv`, unless an operand was `NaI`.
#[inline]
fn dec_rev<F: RoundFloat>(any_nai: bool, result: Interval<F>) -> DecoratedInterval<F> {
    if any_nai {
        DecoratedInterval::nai()
    } else {
        DecoratedInterval::set_dec(result, Decoration::Trv)
    }
}

impl<F: RoundFloat> DecoratedInterval<F> {
    /// Reverse square, decorated `trv` (see the [module docs](crate::reverse)).
    #[must_use]
    pub fn sqr_rev(self, x: Self) -> Self {
        let nai = self.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().sqr_rev(x.interval()))
    }

    /// Reverse absolute value, decorated `trv`.
    #[must_use]
    pub fn abs_rev(self, x: Self) -> Self {
        let nai = self.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().abs_rev(x.interval()))
    }

    /// Reverse multiplication, decorated `trv`.
    #[must_use]
    pub fn mul_rev(self, c: Self, x: Self) -> Self {
        let nai = self.is_nai() || c.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().mul_rev(c.interval(), x.interval()))
    }

    /// The two-output reverse division `mulRevToPair`, decorated.
    ///
    /// `NaI` in either input poisons both outputs to `NaI`. Otherwise the
    /// standard grades this operation as its own two-output division rather than
    /// as a generic reverse (clause 12.12.3): when both inputs are nonempty and
    /// zero lies outside the divisor `self`, the first piece is exactly the
    /// normal division `c / self` and carries that division's decoration, namely
    /// the meet of the two input decorations with the division's local grade
    /// (`com` on a bounded nonempty piece, `dac` when unbounded); the second
    /// piece is empty and `trv`. In every other case, zero inside or on the
    /// boundary of the divisor or an empty input, both pieces are `trv`.
    ///
    /// A genuine two-component result arises only when zero is interior to the
    /// divisor, and then both pieces are `trv`; the propagated decoration shows
    /// only on a one-component result.
    #[must_use]
    pub fn mul_rev_to_pair(self, c: Self) -> (Self, Self) {
        if self.is_nai() || c.is_nai() {
            return (Self::nai(), Self::nai());
        }
        let b = self.interval();
        let (p1, p2) = b.mul_rev_to_pair(c.interval());
        // The first piece propagates the normal division's decoration exactly
        // when zero is outside the nonempty divisor and the dividend is nonempty.
        // `new_dec` supplies the division's local grade (`com` on a bounded
        // nonempty piece, `dac` when unbounded, `trv` when empty), and the meet
        // with the input decorations completes the propagation. Zero on the
        // boundary counts as zero in the divisor, so the closed-interval
        // `contains` is the right test. Every other case grades `trv`.
        let first = if !b.is_empty() && !c.interval().is_empty() && !b.contains(F::ZERO) {
            self.decoration()
                .meet(c.decoration())
                .meet(Self::new_dec(p1).decoration())
        } else {
            Decoration::Trv
        };
        // The second piece is nonempty only on a genuine split (zero interior to
        // the divisor), where the standard grades it `trv`; otherwise it is
        // empty, also `trv`.
        (Self::set_dec(p1, first), Self::set_dec(p2, Decoration::Trv))
    }
}

impl<F: RoundFloat + RoundPown> DecoratedInterval<F> {
    /// Reverse integer power, decorated `trv` (see the [module
    /// docs](crate::reverse)).
    #[must_use]
    pub fn pown_rev(self, x: Self, n: i32) -> Self {
        let nai = self.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().pown_rev(x.interval(), n))
    }
}

impl<F: RoundFloat + RoundInverseTrig> DecoratedInterval<F> {
    /// Reverse sine, decorated `trv`.
    #[must_use]
    pub fn sin_rev(self, x: Self) -> Self {
        let nai = self.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().sin_rev(x.interval()))
    }

    /// Reverse cosine, decorated `trv`.
    #[must_use]
    pub fn cos_rev(self, x: Self) -> Self {
        let nai = self.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().cos_rev(x.interval()))
    }

    /// Reverse tangent, decorated `trv`.
    #[must_use]
    pub fn tan_rev(self, x: Self) -> Self {
        let nai = self.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().tan_rev(x.interval()))
    }
}

impl<F: RoundFloat + RoundInverseHyperbolic> DecoratedInterval<F> {
    /// Reverse hyperbolic cosine, decorated `trv`.
    #[must_use]
    pub fn cosh_rev(self, x: Self) -> Self {
        let nai = self.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().cosh_rev(x.interval()))
    }
}

impl<F: RoundFloat + RoundTranscendental> DecoratedInterval<F> {
    /// Reverse power in the base, decorated `trv`.
    #[must_use]
    pub fn pow_rev1(self, c: Self, x: Self) -> Self {
        let nai = self.is_nai() || c.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().pow_rev1(c.interval(), x.interval()))
    }

    /// Reverse power in the exponent, decorated `trv`.
    #[must_use]
    pub fn pow_rev2(self, c: Self, x: Self) -> Self {
        let nai = self.is_nai() || c.is_nai() || x.is_nai();
        dec_rev(nai, self.interval().pow_rev2(c.interval(), x.interval()))
    }
}

#[cfg(all(test, feature = "fixture"))]
mod tests {
    use crate::Interval;

    fn iv(lo: f64, hi: f64) -> Interval<f64> {
        Interval::new(lo, hi).unwrap()
    }

    // The width of `[inf, sup]` in ulps (both endpoints share a binade here).
    fn ulp_width(r: Interval<f64>) -> u64 {
        r.sup().to_bits() - r.inf().to_bits()
    }

    #[test]
    fn pown_rev_small_root_is_ulp_tight() {
        // Strengthened from the old binade-tight bound: the `RoundPown`
        // certification closes the root to the correctly-rounded pair over a
        // tight backend (adjacent floats) and, over the sound-not-tight fixture
        // whose loose `sqrt` seeds the bisection, to a bracket a handful of ulps
        // wide rather than the couple of binades the old bit bisection left.
        // c = [1e-300, 1e-300], n = 2; the true root is near 1e-150.
        let r = iv(1e-300, 1e-300).pown_rev(iv(0.0, 1.0), 2);
        assert!(!r.is_empty());
        // Correct rounding, weakened only by the fixture's own rounding.
        assert!(ulp_width(r) <= 4, "bracket too wide: {} ulps", ulp_width(r));
        // Soundness through the exact forward `pown`: the result squared must
        // enclose the constraint.
        let img = iv(r.inf(), r.sup()).pown(2);
        assert!(
            img.inf() <= 1e-300 && 1e-300 <= img.sup(),
            "does not enclose"
        );
    }

    #[test]
    fn pown_rev_large_root_is_ulp_tight() {
        // The mirrored branch: c = [1e300, 1e300], n = 2; the true root is near
        // 1e150, seeded from below by the doubling construction. Same
        // strengthening: a handful of ulps, not a binade.
        let r = iv(1e300, 1e300).pown_rev(iv(0.0, f64::INFINITY), 2);
        assert!(!r.is_empty());
        assert!(ulp_width(r) <= 4, "bracket too wide: {} ulps", ulp_width(r));
        let img = iv(r.inf(), r.sup()).pown(2);
        assert!(img.inf() <= 1e300 && 1e300 <= img.sup(), "does not enclose");
    }

    #[test]
    fn sin_rev_far_narrow_candidate_stays_tight() {
        // Rework regression: a far-but-narrow candidate (the constraint
        // propagation shape) must get real arcs from the landing phase, not
        // the degraded fallback the old from-zero walk hit past its cap.
        let cand = iv(1.0e6, 1.0e6 + 10.0);
        let constraint = iv(0.5, 1.0);
        let rev = constraint.sin_rev(cand);
        assert!(!rev.is_empty());
        // Proper subset: sin(1e6) ~ -0.35 and sin(1e6 + 10) ~ -0.22, both
        // outside [0.5, 1], so both edges of the tight result lie strictly
        // inside the candidate.
        assert!(
            rev.inf() > cand.inf(),
            "lower edge degraded: {:e}",
            rev.inf()
        );
        assert!(
            rev.sup() < cand.sup(),
            "upper edge degraded: {:e}",
            rev.sup()
        );
        // The window spans ~1.6 periods, so it contains at least one full
        // preimage arc of width 2*pi/3 ~ 2.09.
        assert!(rev.sup() - rev.inf() >= 2.0);
        // Soundness cross-check through the forward battery: every sampled
        // witness whose forward sine certainly lands in the constraint must be
        // in the reverse result.
        let mut t = cand.inf();
        while t <= cand.sup() {
            let image = Interval::point(t).unwrap().sin();
            if image.subset(constraint) {
                assert!(rev.contains(t), "reverse missed t = {t}");
            }
            t += 1.0e-3;
        }
    }

    #[test]
    fn sin_rev_beyond_smear_horizon_degrades_soundly() {
        // Past ~10^15 periods the accumulated pi-enclosure uncertainty
        // reaches the inter-arc gap; the sound fallback is the candidate
        // itself, exactly.
        let x = iv(1.0e17, 1.0e17 + 32.0);
        let r = iv(0.5, 1.0).sin_rev(x);
        assert!(!r.is_empty());
        assert!(r.inf() == x.inf() && r.sup() == x.sup());
    }
}
