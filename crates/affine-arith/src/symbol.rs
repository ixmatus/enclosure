//! Noise symbols and the source that hands out fresh ones.
//!
//! Each deviation term of an affine form is attached to a noise symbol `εᵢ`. Two
//! forms that carry the same symbol share the underlying source of uncertainty,
//! and that shared identity is the whole point of affine arithmetic: a
//! correlated combination such as `x - x` cancels exactly because both operands
//! reference the same symbols, where interval arithmetic, having forgotten the
//! correlation, would widen.

/// A noise-symbol identifier `εᵢ ∈ [−1, 1]`.
///
/// Symbols are compared by their raw id so an affine form can keep its terms in
/// a canonical order. Equality is identity: two affine forms sharing a symbol
/// reference the same uncertainty.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct NoiseSymbol(u64);

impl NoiseSymbol {
    /// The raw identifier.
    #[must_use]
    pub fn id(self) -> u64 {
        self.0
    }
}

/// A source of fresh noise symbols.
///
/// Each call to [`fresh`](SymbolSource::fresh) returns a symbol distinct from
/// every one the source has handed out before, so forms built from one source
/// never alias unrelated uncertainties. The source is threaded explicitly into
/// the operations that introduce symbols (constructing a form from an interval,
/// and later the nonlinear operations), rather than drawn from global state:
/// that keeps symbol allocation deterministic, keeps the symbol-minting
/// operations honest about minting, and leaves the crate free of hidden mutable
/// state a model checker would have to reason about.
#[derive(Clone, Debug)]
pub struct SymbolSource {
    next: u64,
}

impl SymbolSource {
    /// A fresh source whose first symbol carries id `0`.
    #[must_use]
    pub fn new() -> Self {
        Self { next: 0 }
    }

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

impl Default for SymbolSource {
    fn default() -> Self {
        Self::new()
    }
}
