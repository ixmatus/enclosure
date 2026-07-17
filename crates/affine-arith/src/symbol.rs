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

    /// A symbol from a raw identifier. Crate-internal: the public path to a
    /// symbol is [`SymbolSource::fresh`], and the public path to a form carrying
    /// reconstructed symbols is `AffineForm::from_raw_parts`, which owns the
    /// validation a raw id needs.
    pub(crate) fn from_raw(id: u64) -> Self {
        Self(id)
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

    /// The id the next [`fresh`](SymbolSource::fresh) call will return.
    ///
    /// Exposed for callers that persist an unscoped source across sessions (see
    /// [`SymbolSource::unscoped`]): saving this value alongside the forms that
    /// reference the source's symbols, and restoring it with
    /// [`unscoped_at`](SymbolSource::unscoped_at), keeps freshness intact.
    #[must_use]
    pub fn next_id(&self) -> u64 {
        self.next
    }
}

impl SymbolSource<'static> {
    /// An unscoped source: the escape hatch from the [`with_source`] brand for
    /// callers that need forms to live indefinitely (for example a calculator
    /// stack whose values persist across operations).
    ///
    /// The returned source and every form built from it carry the `'static`
    /// brand. All `'static` brands unify, so **the compiler no longer rejects
    /// mixing forms from two unscoped sources** — the soundness obligation the
    /// brand normally discharges moves to the caller as an invariant:
    ///
    /// **One unscoped source per universe of values.** Every long-lived form
    /// that can ever be combined with another must mint its symbols from the
    /// same unscoped source. Combining forms from two unscoped sources collides
    /// their independent symbols and silently breaks the enclosure guarantee,
    /// exactly the failure [`with_source`] makes unrepresentable.
    ///
    /// Two further obligations follow for callers that persist forms:
    ///
    /// - **Persist the counter with the values.** Save [`next_id`] atomically
    ///   with every form referencing the source's symbols and restore them
    ///   together via [`unscoped_at`](SymbolSource::unscoped_at). Restoring an
    ///   older counter while newer forms survive would let `fresh` re-issue a
    ///   live id.
    /// - **Reset by relabeling.** To restart the counter, first strip or
    ///   relabel the symbols of every live form (reduce each to its interval
    ///   and rebuild). Relabeling forgets correlation, which only widens future
    ///   combinations; it never breaks enclosure.
    ///
    /// [`with_source`] remains the canonical API; reach for this only when a
    /// scope cannot outlive the values it would brand. See the workspace
    /// decision record on the unscoped source.
    ///
    /// [`next_id`]: SymbolSource::next_id
    #[must_use]
    pub fn unscoped() -> Self {
        Self::unscoped_at(0)
    }

    /// An unscoped source that resumes issuing ids at `next`.
    ///
    /// The restore half of the persistence contract on
    /// [`unscoped`](SymbolSource::unscoped): `next` must be a value previously
    /// observed from [`next_id`](SymbolSource::next_id) of the same logical
    /// source, at or after the moment every surviving form's symbols were
    /// minted. A `next` lower than a surviving form's greatest symbol id lets
    /// `fresh` alias a live symbol, which silently breaks the enclosure
    /// guarantee.
    #[must_use]
    pub fn unscoped_at(next: u64) -> Self {
        Self {
            next,
            brand: PhantomData,
        }
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
///
/// The brand is what makes that safe. A form is branded to the scope that built
/// it, so combining it with a *different* scope's source does not compile. The
/// snippet below is the passing example with one change: the `sub` draws its
/// symbols from an inner scope's source, and the brand mismatch is the only
/// plausible error, so the guard fails for exactly the intended reason.
///
/// ```compile_fail
/// use affine_arith::{with_source, AffineForm};
/// use interval_1788::Interval;
///
/// let iv = Interval::new(2.0_f64, 4.0).unwrap();
/// with_source(|mut outer| {
///     let x = AffineForm::from_interval(&iv, &mut outer).unwrap();
///     with_source(|mut inner| {
///         // `x` is branded to `outer`; `inner` carries a distinct brand, so the
///         // compiler refuses to let `x` combine with `inner`'s symbols.
///         x.sub(&x, &mut inner).reduce()
///     })
/// });
/// ```
pub fn with_source<R>(f: impl for<'id> FnOnce(SymbolSource<'id>) -> R) -> R {
    f(SymbolSource {
        next: 0,
        brand: PhantomData,
    })
}
