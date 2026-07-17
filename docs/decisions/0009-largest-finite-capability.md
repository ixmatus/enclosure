# ADR-0009: The largest-finite capability and the realmax midpoint convention

**Status:** Proposed (number assigned at integration)
**Date:** 2026-07-17

This record promotes the design half of bead `enc-05r`, discovered during the
numeric-function completion (enc-sb8): the Level 2 `mid` of a half-unbounded
interval is pinned by the conformance corpus to the realmax convention
(`mid([0, +inf]) = +MAX`, `mid([-inf, 1.2]) = -MAX`, per
`libieeep1788_num.itl`), and no largest-finite datum is reachable through the
frozen `RoundFloat` core. The shipped `mid_rad` soundly returns the finite
endpoint with an infinite radius and documents the divergence at the method;
bit-exact conformance on those vectors is unreachable until a backend
capability exists. The conformance lane (enc-ac4) is the consumer that forces
the decision now.

## Context

Three constraints shape the answer:

1. **The datum is exact.** A format's largest finite value involves no
   rounding direction and no approximation; it is format metadata, like
   `ZERO`, `ONE`, and `INFINITY`, which the core trait already exposes as
   associated constants. Whatever home it gets, it is a constant, not a
   directed method pair.
2. **The core is frozen.** ADR-0007 rejected growing shipped traits mid
   flight because every out-of-tree backend implementation breaks (SMIL
   implements `RoundFloat` today). The same argument applies verbatim to a
   new required constant on the core.
3. **The standard's `mid` requires the datum.** A backend that cannot name
   its largest finite value cannot implement the standard's `mid` on
   half-unbounded inputs. ADR-0008 named the posture for such cases:
   capability honesty over coverage. Pretending the operation is available
   everywhere, with a silent behavioral fork where the datum is missing, is
   the failure mode to avoid.

## Decision

### 1. `RoundLargestFinite`: a one-constant capability trait

A new extension trait in round-float:

```rust
pub trait RoundLargestFinite: RoundFloat {
    /// The format's largest finite value, exactly.
    const LARGEST_FINITE: Self;
}
```

Both in-tree backends implement it with `f64::MAX`: the sound-not-tight
fixture as well as `TightF64`, because the constant is exact format metadata
and tightness is not in question. interval-1788 re-exports the trait beside
`RoundFloat` for its conformance surface.

The name follows the family branding (`RoundInteger`, `RoundReduction`); the
constant follows the core's `ZERO`/`ONE`/`INFINITY` precedent. A method pair
(`largest_finite_down`/`_up`) was rejected: there is nothing to round, and a
directed surface would misstate the datum's nature.

### 2. `mid`, `rad`, and `mid_rad` move under the capability bound

The three methods leave the plain `RoundFloat` impl block for one bounded
`F: RoundFloat + RoundLargestFinite`. The half-unbounded arms change to the
Level 2 convention: `mid([a, +inf]) = +LARGEST_FINITE`,
`mid([-inf, b]) = -LARGEST_FINITE`, `mid(Entire) = 0`, radius `+inf` in all
three cases. The defining enclosure property survives unchanged: the emitted
midpoint is a member of the interval (any finite endpoint is at most `MAX` in
magnitude), and `[mid - rad, mid + rad]` with an infinite radius is the whole
line, which contains everything. The divergence paragraph in the method
documentation dissolves into a statement of the convention, and the
`libieeep1788_num.itl` midpoint vectors become reachable bit-exact.

**Rejected: a `MAX_FINITE` constant on the `RoundFloat` core.** Breaks every
out-of-tree implementation for a datum only the numeric functions consume;
reopens a frozen seam that ADR-0007 kept closed under the same pressure.

**Rejected: a `TightF64`-specific path in the conformance lane only.** The
surface would fork: `mid` reporting different data on backends holding the
same information, the fixture's tests permanently unable to reach the
vectors, and the divergence hidden in a test lane instead of stated in a
signature. The bound in the signature is the honest form of the same fact.

## Consequences

- Ledger consequence: the `mid`/`rad` divergence noted in enc-sb8 closes;
  the conformance lane (enc-ac4) gets bit-exact midpoint vectors on both
  in-tree backends.
- Three ways this design could be wrong, per the inversion discipline:
  1. **Out-of-tree backends lose `mid` until they implement the trait.**
     SMIL's `RoundFloat` impl stops compiling against `mid` callers until a
     one-constant impl lands. Accepted 0.x churn: the remedy is one line,
     and the alternative is a silently divergent midpoint on exactly the
     backend a consumer would trust most.
  2. **A future runtime-precision backend cannot supply the constant.** An
     arbitrary-precision float whose exponent range is a runtime value has
     no type-level largest finite. Named boundary, accepted: the house
     style already pushes precision to the type level (const generics), and
     a backend that genuinely cannot do so gets a method-based sibling
     trait then, not a preemptive one now.
  3. **`mid = ±MAX` with `rad = +inf` surprises a consumer treating `mid`
     as central tendency.** A midpoint at the edge of the finite range is
     an odd centroid. The convention is the standard's, not this crate's
     invention; the method documentation states it, and the defining
     property (enclosure of the interval by `mid ± rad`) is the contract
     consumers may rely on.

## References

- Registry: `docs/references/itf1788-framework.md`
  (`libieeep1788_num.itl`, the vectors that pin the convention);
  `docs/references/p1788-1-d98-draft.md` (the numeric-function clause the
  vectors witness).
- `crates/interval-1788/src/functions.rs` (`mid_rad`, the site whose
  divergence note this dissolves); `crates/round-float/src/lib.rs` (the
  core constants whose precedent the new constant follows).
- ADR-0007 (the frozen-seam argument inherited by rejection 1); ADR-0008
  (the capability-honesty posture inherited by the bound).
- Beads: enc-05r (this design and its implementing half), enc-sb8 (the
  discovery), enc-ac4 (the conformance-lane consumer).
