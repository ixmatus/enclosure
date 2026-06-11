# Kernel map

Per kernel: the algorithm, the source it derives from (registry slug), the decision
record, and the verification artifact. This is the synthesis the per source entries
cannot hold singly; the entries hold the citations, this table holds the story.

## round-float: directed rounding primitives

| Kernel | Algorithm | Source | Decision record | Verification |
|---|---|---|---|---|
| `add/sub/mul/div/sqrt` `_down`/`_up` (f64 fixture) | next-toward stepping around the host operation; `sqrt` exact via `libm` (correctly rounded) | [ieee-754-2019](ieee-754-2019.md) rounding attributes; [rust-libm-crate](rust-libm-crate.md) | round-float ADR-0001 | round-float fixture property tests |
| `exp/ln` `_down`/`_up` (f64 fixture) | host `libm` value widened by `TRANSCENDENTAL_MARGIN` (8 ulp at 2^-52), resting on musl's stated sub 1.5 ulp accuracy goal | [musl-libm-accuracy](musl-libm-accuracy.md) | round-float ADR-0001 | nightly oracle lane over [pfloat-libm-oracle](pfloat-libm-oracle.md) (`crates/elementary-oracle/tests/oracle.rs`) |

The trait surface (the `RoundFloat` and `RoundTranscendental` extension split) is an
API shape decision, recorded in round-float ADR-0001; only the rounding behavior is
inherited from the standard.

## interval-1788: interval operations

| Kernel | Algorithm | Source | Decision record | Verification |
|---|---|---|---|---|
| `add`, `sub` | endpoint arithmetic with outward directed rounding; `sub` crosses endpoints | [ieee-1788-2015](ieee-1788-2015.md) (operation semantics), [revol-1788-introduction](revol-1788-introduction.md) | interval-1788 ADR-0001 (inf sup representation) | `tests/enclosure_fixture.rs`, Kani empty absorption |
| `mul` | four corner products with sign analysis; zero times unbounded is zero | same | same | property tests incl. four corner sign mixing |
| `div`, `recip` | sign stable cases by directed division; zero straddling reciprocal is `Entire`; division by exact zero is empty | same | same | Kani proofs (`recip_of_zero_straddling_is_entire`, `recip_of_exact_zero_is_empty`, `div_by_exact_zero_is_empty`) |
| `sqr`, `sqrt`, `mul_add` | domain restricted, outward rounded; `sqr` tighter than self multiply | same | same | property tests; Kani `sqrt_of_wholly_negative_is_empty` |
| `exp`, `ln` | endpoint monotone application of `RoundTranscendental` bounds | [ieee-1788-2015](ieee-1788-2015.md); bounds per round-float row above | round-float ADR-0001 | `tests/elementary_fixture.rs`; nightly oracle lane |
| decoration propagation | meet (lattice minimum) over the totally ordered set | [ieee-1788-2015](ieee-1788-2015.md) set based flavor | — | six Kani lattice proofs |

## affine-arith: affine form kernels

| Kernel | Algorithm | Source | Decision record | Verification |
|---|---|---|---|---|
| `add`, `sub`, `negate`, `scale` | exact correlated linear combination, per term outward rounding folded into a fresh symbol | [comba-stolfi-1993](comba-stolfi-1993.md), [figueiredo-stolfi-2004](figueiredo-stolfi-2004.md) | workspace ADR-0001 | `tests/ops_fixture.rs`, `tests/enclosure_fixture.rs` |
| `mul` | affine by affine multiply; bilinear remainder and accumulated roundoff folded outward into one fresh symbol | [figueiredo-stolfi-2004](figueiredo-stolfi-2004.md) | workspace ADR-0001 | `tests/ops_fixture.rs` |
| `sqr`, `recip`, `sqrt`, `exp`, `ln` | Chebyshev (min max) affine approximation: secant slope, residual band enclosed outward, band center and half width plus all rounding into one fresh symbol | [figueiredo-stolfi-2004](figueiredo-stolfi-2004.md), [stolfi-figueiredo-1997](stolfi-figueiredo-1997.md) | workspace ADR-0002 | `tests/elementary_fixture.rs`; exp/ln also the nightly oracle lane |
| `condense` | fold the smallest magnitude tail terms into one fresh symbol bounding their summed magnitude (up rounded) | [figueiredo-stolfi-2004](figueiredo-stolfi-2004.md); order reduction context in [kopetzki-schurmann-althoff-2017](kopetzki-schurmann-althoff-2017.md), [yang-scott-2018](yang-scott-2018.md) | workspace ADR-0004 | `tests/condense_fixture.rs` (pointwise condensation, width preservation, budget) |
| persistent forms, unscoped symbol source | escape hatch from the lifetime brand for serialized forms | design original; failure mode named in the record | workspace ADR-0003 | `tests/persistent_fixture.rs` |

## Linearization choice, recorded

Every nonlinear elementary in `affine-arith` uses the **Chebyshev (min max)
construction**: secant slope over the reduced range, residual band enclosed by the
two endpoints and the single interior stationary point (each function is convex or
concave on its domain). The **Taylor (tangent) construction** of the same literature
is not used anywhere: it is tighter near the expansion point but not min max optimal
over the range, and the secant form's residual geometry is what the directed
rounding error fold was derived against. A future Taylor mode is a deliberate
extension, not an omission. Sources: [figueiredo-stolfi-2004](figueiredo-stolfi-2004.md)
section on elementary functions; the derivation in
`crates/affine-arith/src/elementary.rs` module prose.
