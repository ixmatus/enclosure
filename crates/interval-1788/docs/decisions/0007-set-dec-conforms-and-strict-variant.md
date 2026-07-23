# ADR-0007: setDec conforms by clamping, with a strict variant kept

**Status:** Accepted
**Date:** 2026-07-22

## Context

The set-based flavor's `setDec` constructor pairs a bare interval with an
explicit decoration. The standard defines it to be total: given any interval
and any decoration, it returns a valid decorated interval, weakening the
decoration when the requested one is inconsistent with the interval. An empty
interval forces `trv`; an unbounded interval demotes `com` to `dac`; a request
for `ill` constructs `NaI` and raises `UndefinedOperation`.

The crate shipped `set_dec` with a stricter contract: it validated the pair
against `valid_combo` and returned `NaI` for any inconsistent combination
(`crates/interval-1788/src/decorated.rs`, the `set_dec` body; pinned by the
unit test `set_dec_rejects_inconsistent_combinations`). The choice was
deliberate. It made an inconsistent construction visible as poison rather than
silently repairing it, in the spirit of illegal states being hard to build.

The conformance lane surfaced the cost. Of the 22 `minimal_set_dec_test`
vectors transcribed from the ITF1788 `libieeep1788_class.itl` corpus, six
diverge: `setDec [empty] {def,dac,com}` and `setDec [unbounded] com`. The
standard returns a clamped valid interval on each; the crate returned `NaI`.
The 16 consistent vectors already passed. The six divergent ones were parked
in an ignored test holding the standard datum (bead enc-2hd). A required
operation that returns poison where the standard returns a value is a
conformance defect, not a defensible strictness.

## Decision

Conform, and keep the strict behavior as a separate constructor.

`set_dec` now clamps, matching the standard's `setDec`. It weakens the
requested decoration to the strongest one the interval admits: `new_dec(x)`
gives that strongest decoration (`com` for a bounded nonempty interval, `dac`
for an unbounded nonempty one, `trv` for the empty interval), and the lattice
meet lowers the request to it. A request for `ill` returns `NaI` regardless of
the interval, since `ill` decorates `NaI` alone. The result is total: every
pair yields a valid decorated interval or the `NaI` the `ill` request asks for.

`try_set_dec` is new. It preserves the strict construction exactly, returning
`Option<DecoratedInterval<F>>`: `Some` for a consistent pair, `None` for an
inconsistent one. A caller who wants an inconsistent pair to fail visibly gets
a signal in the type rather than a poisoned value threaded through later
operations. Clamping never strengthens a decoration; it only weakens. The doc
comments on both constructors state which one to reach for and why.

## Consequences

The conformance defect closes. All 22 `minimal_set_dec_test` vectors run in the
live `set_dec_vectors` lane; the ignored gap test is gone.

The failure paragraph. Two harms are the most plausible. First, `set_dec` now
weakens a decoration silently where a caller may have wanted the construction
to fail; a caller who passed `com` for an interval they believed bounded, and
was wrong, gets `dac` back with no signal. Second, the crate diverges from its
own behavior before this change: any consumer of the unreleased crate that read
`NaI` as the inconsistency signal now reads a clamped value instead. The
mitigation for both is `try_set_dec`, which is the exact prior behavior in a
form that cannot be ignored by accident, plus the CHANGELOG Changed entry that
names the semantics shift and the conformance document entry that dates it. The
call sites in tree are unaffected: every internal `set_dec` caller passes a
decoration that is consistent by construction (the reverse wrappers pass `trv`;
the trig, inverse, and text parsing paths pass a decoration already met with
the interval's strongest), so the clamp changes nothing for any of them.

## References

- `crates/interval-1788/src/decorated.rs`: `set_dec` (the clamping
  constructor), `try_set_dec` (the strict variant), and the rewritten unit
  test `set_dec_clamps_and_try_set_dec_is_strict`.
- `crates/interval-1788/tests/text_io_fixture.rs`: `set_dec_vectors`, now
  carrying all 22 `minimal_set_dec_test` vectors including the six clamping
  ones.
- `crates/interval-1788/docs/conformance.md`: section 4 item 4, resolved.
- Bead enc-2hd; the ITF1788 corpus registry entry
  (`docs/references/itf1788-framework.md`).
- IEEE Std 1788-2015, the `setDec` operation of the set-based flavor
  (registry: `docs/references/ieee-1788-2015.md`, pointer only; free proxies
  Pryce 2016 and Revol 2017).
