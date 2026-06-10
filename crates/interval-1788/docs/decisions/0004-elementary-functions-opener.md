# ADR-0004: Elementary functions open behind the transcendental extension trait

**Status:** Accepted
**Date:** 2026-06-10

## Context

The interval crate had no transcendental functions: the v1.0 roadmap carries
"the elementary set is absent" as a blocker, and the calculator consumer now
needs `exp` and `ln` on intervals. The core `RoundFloat` trait deliberately has
no transcendentals (decision record 0002 here, and round-float's record 0001:
a backend cannot always supply a correctly-rounded transcendental as cheaply
as `add`), and affine-arith's elementary surface already established the
pattern: gate the elementary functions on the `RoundTranscendental` extension
trait, so a backend opts in on its own schedule and every existing backend
keeps compiling.

## Decision

**`exp` and `ln` on `Interval<F>` and `DecoratedInterval<F>`, behind
`F: RoundFloat + RoundTranscendental`.** Both functions are monotone, so no
four-corner analysis is needed: the image of `[a, b]` is the directed-rounded
endpoint pair, `exp` clamped at zero below (the true image is positive, and a
loose backend's outward step must not report a negative bound — the same clamp
`sqrt` uses). The set-based flavor keeps both total: `ln` restricts its input
to `(0, +inf)`, an input wholly outside the domain has the empty image, and an
input touching zero is unbounded below. The crate re-exports
`RoundTranscendental` beside `RoundFloat` so consumers resolve one path.

**Decorations account for the result, not only the inputs.** `exp` is defined
and continuous on the whole line, but it overflows a bounded input to an
unbounded result well inside the finite range, and `com` promises a bounded
result; the decorated `exp` and `ln` therefore demote to `dac` when the result
is unbounded. The four basic decorated operations compute their decoration
from the inputs alone, which leaves them able to pack `com` with an overflowed
unbounded result; that latent inconsistency at the overflow edge is filed as
its own issue rather than silently fixed here (one concern per change).

**Certification follows the established two lanes.** The fixture lane checks
pointwise enclosure by property test (sampled `x` in the input, `f(x)` in the
image); the nightly oracle lane checks the image brackets pfloat-libm's
correctly-rounded values, the same harness that certifies affine-arith's
elementary surface. The trigonometric set stays out: it is not monotone, needs
argument reduction case logic, and gates on round-float growing those methods
first.

## Consequences

- The v1.0 blocker "elementary set absent" is opened, not closed: `exp`/`ln`
  establish the pattern (extension-trait gate, domain restriction, decoration
  demotion, two-lane certification) that `pow`, the trig set, and the
  hyperbolics follow as round-float grows.
- A backend implementing only `RoundFloat` is unaffected; the methods do not
  exist for it. The SMIL/ferrodec backend implements `RoundTranscendental`
  consumer-side (ferrodec's `exp`/`ln` are correctly rounded in the directed
  modes, so the instance is tight).
- The decorated basic operations' input-only decoration at the overflow edge
  is now a named, filed inconsistency rather than an unnoticed one.

## References

- round-float `docs/decisions/0001` (the extension trait and the f64 fixture
  margins); this crate's 0002 (trait shape).
- `src/elementary.rs`, the decorated additions in `src/decorated.rs`; tests in
  `tests/elementary_fixture.rs` and the workspace's
  `crates/elementary-oracle/tests/oracle.rs`.
