# Verification map: claim to artifact

How each load bearing claim is known. The ordering follows the estate's preference:
types, proofs, property tests, example tests, documentation. A claim listed with a
weaker artifact than it deserves is a gap, and the gaps are named.

## Claims discharged by types

| Claim | Artifact |
|---|---|
| No interval reachable through the public surface violates canonical form (no NaN endpoint, `lo <= hi`, one empty representation) | private representation + checked constructors, `crates/interval-1788/src/interval.rs`; Law 1 in `spec.rs` |
| Affine forms from different symbol scopes cannot be mixed | the `'id` lifetime brand on `AffineForm` and `SymbolSource`, `crates/affine-arith/src/symbol.rs` (escape hatch and its named failure mode: workspace ADR-0003) |
| No shipped crate depends on the nightly oracle | `crates/elementary-oracle` is workspace excluded; the dependency direction is enforced by the build graph |

## Claims discharged by proof (Kani, CBMC)

Twelve proofs in `crates/interval-1788/src/kani_harness.rs`, run in CI over the f64
fixture (interval-1788 ADR-0003 records why the fixture and not a generic backend):

- Decoration lattice (6): meet commutative, associative, idempotent; `Ill` absorbs;
  `Com` is the identity; meet is monotone.
- Discrete set logic (6): reciprocal of a zero straddling interval is `Entire`;
  reciprocal of exact zero is empty; division by exact zero is empty; `add` and
  `mul` with empty are empty; `sqrt` of a wholly negative interval is empty.

**Named gap:** the numeric enclosure theorem (Law 2) is *not* Kani proved; the bit
level `nextafter` stepping explodes CBMC. It rests on the written law in `spec.rs`
plus the property test lanes below. This is the registry's standing answer to "why
is the fundamental theorem not in the proof column".

## Claims discharged by property test (proptest)

| Claim | Artifact |
|---|---|
| Interval ops enclose the pointwise truth (Law 2, sampled) | `crates/interval-1788/tests/enclosure_fixture.rs` |
| Decoration propagation laws hold across ops | `crates/interval-1788/tests/decorated_fixture.rs` |
| Set functions (`intersection`, `convex_hull`, `subset`, ...) respect set semantics | `crates/interval-1788/tests/functions_fixture.rs` |
| Interval `exp`/`ln` enclose pointwise truth | `crates/interval-1788/tests/elementary_fixture.rs` |
| Affine ops enclose pointwise truth under sampled joint noise assignments | `crates/affine-arith/tests/enclosure_fixture.rs`, `tests/ops_fixture.rs` |
| Chebyshev elementaries enclose, and affine beats interval on correlated expressions | `crates/affine-arith/tests/elementary_fixture.rs` |
| Condensation preserves enclosure pointwise, preserves width up to rounding, respects the budget | `crates/affine-arith/tests/condense_fixture.rs` |
| Persistent round trip preserves form semantics; unscoped source freshness | `crates/affine-arith/tests/persistent_fixture.rs` |

## Claims discharged by differential oracle

| Claim | Artifact |
|---|---|
| The f64 fixture's `exp`/`ln` bounds bracket the correctly rounded value (the musl accuracy assumption holds on the swept grids) | `crates/elementary-oracle/tests/oracle.rs` against [pfloat-libm-oracle](pfloat-libm-oracle.md), pinned rev `04c9761c`, nightly CI lane |
| Affine `exp`/`ln` enclosures contain the correctly rounded truth | same lane |

**Named gap:** the oracle lane certifies `exp` and `ln` only; the arithmetic ops'
tightness against correct rounding awaits the lane extension tracked as bead
`enc-ebd`. And an oracle sweep only speaks for the inputs its grid reached; the
crate READMEs name exactly this in their "What this does not promise" sections.

## Claims discharged by documentation only

| Claim | Artifact | Why no stronger artifact |
|---|---|---|
| Law 2 in full generality (every input, every backend) | `crates/interval-1788/src/spec.rs` | CBMC explosion on bit stepping; a future Creusot/Prusti attempt is open |
| The 1788 exception to decoration mapping | `spec.rs` prose | small closed mapping, enum exhaustiveness already guards the shape |
| ITF1788 conformance | none yet (corpus vendored, lane not wired) | see [itf1788-framework](itf1788-framework.md) for the coverage gaps and the disclosure reconciliation: the vectors, once wired, still cannot retire the READMEs' named failure modes |
