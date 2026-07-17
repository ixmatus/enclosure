# ADR-0002: Affine elementary functions by Chebyshev approximation

**Status:** Accepted
**Date:** 2026-06-09

## Context

affine-arith needed its nonlinear surface: `recip`, `sqrt`, `sqr`, `exp`, and
`ln` on an affine form. Each is the affine analogue of the multiply: linearize the
function over the form's reduced range `[a, b]` and bound everything the linear
part cannot represent into one fresh outward-rounded noise symbol. Four choices
had to be settled: the linearization method, how the construction folds rounding
error, how domain violations are reported, and where sound `exp`/`ln` bounds come
from (the core `RoundFloat` trait has no transcendentals, and `interval-1788` has
no elementary functions to delegate to).

## Decision

**Chebyshev (min-max) approximation for all five.** For a function whose second
derivative keeps one sign on `[a, b]` (each of these is convex or concave on its
domain), the min-max affine approximant takes the secant slope; the residual
`f(x) - alpha*x` is then convex or concave, so its range is pinned by the two
endpoints and the single interior tangent point. Chebyshev is the tightest affine
fit (roughly twice as tight as the simpler min-range rule for the monotone
functions, four times for the square). The cost is an interior tangent point whose
location needs the function's inverse (`exp` needs `ln`, `ln` needs a reciprocal),
lengthening the outward-rounding chain near a steep boundary; the verification
lane below bounds that risk rather than trading away the tightness. The method is
chosen per function internally, never exposed as a caller knob, because both
linearizations are sound and tightness must not be something a user can
accidentally discard.

**One-pass construction, one fresh symbol.** A shared `linearize` builder emits
`g = alpha*x_hat + beta + delta*e_new` in a single pass over the form's terms,
mirroring the multiply. It is handed an outward enclosure `[r_lo, r_hi]` of the
residual range; `beta` is the band midpoint and `delta` is the larger of the two
up-rounded one-sided gaps measured from the emitted `beta`, so the strip covers
the band whatever `beta` rounded to. The center and per-coefficient rounding
errors accumulate outward into that same fresh symbol. Routing through
`scale` then `add` would mint three fresh symbols and compound three outward
bands; the direct build mints one.

**Domain violations are reported as `None`.** A noise symbol's range cannot be
clamped the way interval arithmetic restricts a domain, and a finite affine form
cannot represent an unbounded result. So `recip` declines a range straddling zero
(the reciprocal is unbounded), `ln` declines a range reaching zero or below, and
`sqrt` declines a reduced range dipping below zero. The last includes a form built
from a zero-touching interval such as `[0, b]`, whose outward-rounded reduction
dips a hair below zero: it declines too. Restricting `sqrt` to the nonnegative
part is also sound but would let the linear extrapolation report a negative lower
bound on a square root, worse than declining, and separating the rounding artifact
from a genuine negative range needs a unit-in-the-last-place notion the trait does
not expose; a tighter zero-touching path is left as a later refinement.

**`exp` and `ln` ride `RoundTranscendental`, certified by a nightly oracle.** The
transcendentals sit on a `RoundFloat + RoundTranscendental` bound (round-float
decision record 0001), so the algebraic surface stays available to any backend.
The stable f64 fixture widens `libm` (faithfully rounded) by a relative margin,
leaving soundness resting on musl's accuracy goal. A workspace-excluded crate,
`elementary-oracle`, discharges that empirically: it checks the fixture and affine
`exp`/`ln` bounds enclose the correctly-rounded values from pfloat-libm (a pure
Rust correctly-rounded libm, pinned to a public revision). pfloat needs nightly,
so the crate carries its own toolchain pin and runs in a dedicated nightly CI job;
the shipped crates never depend on it.

**Noise-term condensation is deferred.** Each operation already mints the minimum,
one fresh symbol; bounding term-count growth across long computations is an
orthogonal concern that keeps its own issue.

## Consequences

- The elementary surface is sound. An adversarial multi-agent pass re-derived every
  slope and residual bound and brute-force-probed each function for enclosure
  violations over single- and multi-symbol forms, interior tangent points, ranges
  pressed against the domain boundary, and overflow; no rounding direction is
  wrong. A multi-symbol enclosure lane and per-function property tests guard it.
- `exp`/`ln` soundness is anchored to a correctly-rounded reference rather than to
  trust in an accuracy goal, while the stable wall holds: stable builds and the
  shipped crates' consumers never pull pfloat.
- The `sqrt` zero-touching decline is a known conservative limitation, documented
  at the call site and pinned by a regression test.
- The in-crate model-checking of the discrete domain logic and a compile-fail guard
  for the cross-source brand are carried by the dedicated verification lane, not
  this phase.

## References

- round-float decision record 0001 (the `RoundTranscendental` extension trait and
  the f64 fixture's relative margin).
- de Figueiredo and Stolfi, "Affine Arithmetic: Concepts and Applications" (2004),
  the univariate Chebyshev approximation; Stolfi and de Figueiredo (1997).
- pfloat-libm (the correctly-rounded reference the oracle lane checks against).

## Addendum

**2026-07-17: the zero-touching `sqrt` refinement is closed, not deferred.**
The decision above left "a tighter zero-touching path" as a later refinement.
The refinement was attempted (bead enc-fj5) and the analysis closes it: the
crate's enclosure invariant is pointwise-total (the output form must enclose
the function's value at every joint assignment of the shared noise symbols),
and a form built by `from_interval([0, b])` always reaches a strictly
negative value (outward rounding forces `radius >= next_up(center)`, so the
true minimum `center - radius` is negative for every such form). A fit over
the clamped range is therefore sound only under 1788-style domain
restriction, which is a different invariant, not a tighter implementation;
adopting it needs a definedness or decoration analog for `AffineForm` first,
since a bare form cannot record the restriction the way a decorated interval
records `trv`, and restricting a shared noise symbol would corrupt sibling
forms. The decline and its regression pin stand as the correct behavior
under the current invariant. Successors: the domain-restriction semantics
design (bead enc-hc2) and, orthogonally, an exact in-domain predicate that
removes false declines for forms whose exact range is in domain while the
double-rounded reduction endpoint crosses the boundary (bead enc-6lm);
neither rescues `[0, b]`, whose sliver is genuine.
