//! Noise symbols, the branded source that hands out fresh ones, and the scope
//! that brands it.
//!
//! Each deviation term of an affine form is attached to a noise symbol `εᵢ`. Two
//! forms that carry the same symbol share the underlying source of uncertainty,
//! and that shared identity is the whole point of affine arithmetic: a
//! correlated combination such as `x - x` cancels exactly because both operands
//! reference the same symbols, where interval arithmetic, having forgotten the
//! correlation, would widen.
//!
//! Symbol ids are only meaningful within one source. Combining forms from two
//! different sources would collide their independent symbols and silently break
//! the enclosure guarantee. To make that combination impossible rather than
//! merely discouraged, a [`SymbolSource`] and the forms built from it carry an
//! invariant lifetime brand `'id`, handed out by [`with_source`]. Each call to
//! [`with_source`] gets a fresh, incompatible `'id`, so the compiler rejects any
//! attempt to mix forms or sources from different scopes.

use core::marker::PhantomData;

/// An invariant lifetime brand. The `fn(&'id ()) -> &'id ()` shape is invariant
/// in `'id`, so two brands with different lifetimes never unify; that is what
/// keeps forms from one [`with_source`] scope from mixing with another.
pub(crate) type Brand<'id> = PhantomData<fn(&'id ()) -> &'id ()>;

/// A noise-symbol identifier `εᵢ ∈ [−1, 1]`.
///
/// Symbols are compared by their raw id so an affine form can keep its terms in
/// a canonical order. Equality is identity within a source: two forms from the
/// same source sharing a symbol reference the same uncertainty. Ids from
/// different sources are not comparable, which the lifetime brand on
/// [`SymbolSource`] prevents at compile time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct NoiseSymbol(u64);

impl NoiseSymbol {
    /// The raw identifier.
    #[must_use]
    pub fn id(self) -> u64 {
        self.0
    }
}

/// A source of fresh noise symbols, branded with the lifetime `'id` of the
/// [`with_source`] scope that created it.
///
/// Each call to [`fresh`](SymbolSource::fresh) returns a symbol distinct from
/// every one the source has handed out before, so forms built from one source
/// never alias unrelated uncertainties. The brand ties every form built here to
/// this source: a form from a different source carries a different `'id` and
/// cannot be combined with these, so the single-source precondition is enforced
/// by the type system rather than left to the caller.
pub struct SymbolSource<'id> {
    next: u64,
    brand: Brand<'id>,
}

impl SymbolSource<'_> {
    /// A fresh symbol, distinct from every one this source has already issued.
    ///
    /// # Panics
    ///
    /// Panics if the `u64` id space is exhausted (after `2^64` symbols from one
    /// source). Wrapping instead would alias an old symbol onto a new
    /// uncertainty and silently break the enclosure guarantee, so exhaustion
    /// aborts rather than corrupts; it is unreachable in any real computation.
    pub fn fresh(&mut self) -> NoiseSymbol {
        let symbol = NoiseSymbol(self.next);
        self.next = self
            .next
            .checked_add(1)
            .expect("noise-symbol id space exhausted");
        symbol
    }
}

/// Run `f` with a freshly branded [`SymbolSource`], returning whatever `f`
/// produces.
///
/// The higher-ranked `'id` gives each call its own brand, so a form built in one
/// `with_source` scope cannot be combined with a form from another: the compiler
/// rejects it. Branded values therefore cannot escape the scope; reduce a form to
/// an [`interval_1788::Interval`], which carries no brand, to return a result.
///
/// ```
/// use affine_arith::{with_source, AffineForm};
/// use interval_1788::Interval;
///
/// let iv = Interval::new(2.0_f64, 4.0).unwrap();
/// let enclosure = with_source(|mut src| {
///     let x = AffineForm::from_interval(&iv, &mut src).unwrap();
///     x.sub(&x, &mut src).reduce()
/// });
/// assert!(enclosure.contains(0.0));
/// ```
pub fn with_source<R>(f: impl for<'id> FnOnce(SymbolSource<'id>) -> R) -> R {
    f(SymbolSource {
        next: 0,
        brand: PhantomData,
    })
}
