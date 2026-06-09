//! The affine form and its round-trip with an interval.

use alloc::vec::Vec;

use interval_1788::Interval;
use round_float::RoundFloat;

use crate::symbol::{NoiseSymbol, SymbolSource};

/// One deviation term `xᵢ εᵢ` of an affine form: a coefficient attached to a
/// noise symbol.
#[derive(Clone, Copy, Debug)]
pub struct Term<F> {
    symbol: NoiseSymbol,
    coeff: F,
}

impl<F: Copy> Term<F> {
    /// The noise symbol this term varies with.
    #[must_use]
    pub fn symbol(&self) -> NoiseSymbol {
        self.symbol
    }

    /// The partial deviation (the coefficient on the symbol).
    #[must_use]
    pub fn coeff(&self) -> F {
        self.coeff
    }
}

/// An affine form `x̂ = x₀ + x₁ε₁ + x₂ε₂ + ... + xₙεₙ`, with the noise symbols
/// `εᵢ ∈ [−1, 1]`.
///
/// The form is a first-class enclosure: every real value it can take lies in the
/// interval `x₀ ± Σ|xᵢ|`, which [`reduce`](AffineForm::reduce) returns. Unlike an
/// interval, the form remembers which symbols it shares with other forms, so
/// correlated combinations stay tight.
///
/// # Representation invariant
///
/// The deviation terms are kept sorted by ascending symbol id, with no duplicate
/// symbol and no zero coefficient. Constructors and operations re-establish this
/// invariant rather than assume it, so the ordering can drive the linear merge
/// in addition and subtraction.
#[derive(Clone, Debug)]
pub struct AffineForm<F> {
    center: F,
    terms: Vec<Term<F>>,
}

impl<F: RoundFloat> AffineForm<F> {
    /// A degenerate form with no uncertainty: `x̂ = center`, no noise symbols.
    #[must_use]
    pub fn point(center: F) -> Self {
        Self {
            center,
            terms: Vec::new(),
        }
    }

    /// An affine form enclosing a bounded, nonempty interval, introducing one
    /// fresh noise symbol for its width.
    ///
    /// Returns `None` for the empty interval and for an unbounded one: neither is
    /// a bounded set, and a finite affine form represents only bounded
    /// quantities. A singleton `[c, c]` reduces to the exact [`point`] form on a
    /// correctly-rounded backend, spending no symbol.
    ///
    /// The construction is sound for any finite center it picks. The center is an
    /// outward-rounded estimate of the midpoint; the radius is then the larger of
    /// the two one-sided distances from that center to the endpoints, each
    /// rounded up, so `center ± radius` encloses `[lo, hi]` whatever rounding did
    /// to the center. A precise midpoint only buys tightness, never soundness.
    ///
    /// [`point`]: AffineForm::point
    pub fn from_interval(iv: &Interval<F>, src: &mut SymbolSource) -> Option<Self> {
        if iv.is_empty() {
            return None;
        }
        let lo = iv.inf();
        let hi = iv.sup();
        if !lo.is_finite() || !hi.is_finite() {
            return None;
        }

        // An outward estimate of the midpoint. Every operation rounds toward plus
        // infinity, so `center >= (lo + hi) / 2`; the radius below restores
        // soundness regardless. If the sum overflows to an infinity, fall back to
        // the lower endpoint as the center (sound, only looser).
        let two = F::ONE.add_up(F::ONE);
        let mut center = lo.add_up(hi).div_up(two);
        if !center.is_finite() {
            center = lo;
        }

        // radius >= max(center - lo, hi - center), each an upper bound. The
        // `rmax` with zero guards the singleton case, where both one-sided
        // distances can round to a hair below zero.
        let r_lo = center.sub_up(lo);
        let r_hi = hi.sub_up(center);
        let radius = r_lo.rmax(r_hi).rmax(F::ZERO);

        if radius.is_zero() {
            return Some(Self::point(center));
        }

        let symbol = src.fresh();
        Some(Self {
            center,
            terms: alloc::vec![Term {
                symbol,
                coeff: radius
            }],
        })
    }

    /// The central value `x₀`.
    #[must_use]
    pub fn center(&self) -> F {
        self.center
    }

    /// The deviation terms, in ascending symbol order.
    #[must_use]
    pub fn terms(&self) -> &[Term<F>] {
        &self.terms
    }

    /// The number of deviation terms.
    #[must_use]
    pub fn num_terms(&self) -> usize {
        self.terms.len()
    }

    /// Whether this is a point form (no uncertainty).
    #[must_use]
    pub fn is_point(&self) -> bool {
        self.terms.is_empty()
    }

    /// The total deviation `Σ|xᵢ|`, rounded up so it is an upper bound on the
    /// form's distance from its center. Zero for a point form.
    #[must_use]
    pub fn radius(&self) -> F {
        let mut radius = F::ZERO;
        for term in &self.terms {
            let magnitude = if term.coeff.is_sign_negative() {
                term.coeff.negate()
            } else {
                term.coeff
            };
            radius = radius.add_up(magnitude);
        }
        radius
    }

    /// The interval `x₀ ± Σ|xᵢ|`, rounded outward: the enclosure the form
    /// guarantees. A point form reduces to the singleton at its center.
    #[must_use]
    pub fn reduce(&self) -> Interval<F> {
        let radius = self.radius();
        if radius.is_zero() {
            // A point form's tightest sound enclosure is exactly its center. Take
            // it directly rather than through `sub_down`/`add_up`, whose
            // unconditional outward step (in the sound-not-tight fixture) would
            // widen an exact value by a needless unit in the last place.
            return Interval::point(self.center).unwrap_or_else(|_| Interval::entire());
        }
        let lo = self.center.sub_down(radius);
        let hi = self.center.add_up(radius);
        // `lo <= center - radius <= center + radius <= hi` holds with a finite
        // center, so the constructor succeeds. The fallback keeps the function
        // total without weakening rigor: the whole line encloses any value, so an
        // unreachable construction failure degrades to a sound (looser) result
        // rather than a panic.
        Interval::new(lo, hi).unwrap_or_else(|_| Interval::entire())
    }
}
