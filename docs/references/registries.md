# Registries: the enumerable closed sets

Each registry below is a closed set the code enumerates somewhere. This document
records where the in tree source of truth lives, what external set it mirrors, and
whether a code generator keeps the two in sync. None of the registries has a
generator today; each is small enough that the type system itself is the guard (an
exhaustive `match` over a fieldless enum cannot silently drop a member). A generator
becomes worth building the day a registry stops being mirrored by a single enum.

## Decorations (IEEE 1788 set based flavor)

Source of truth: `Decoration` in `crates/interval-1788/src/decoration.rs`. Five
members, a totally ordered lattice from weakest to strongest:

| Decoration | Meaning |
|---|---|
| `Ill` (0) | ill formed, the NaI poison from an invalid construction |
| `Trv` (1) | trivial: nothing guaranteed |
| `Def` (2) | defined on the inputs, not everywhere continuous |
| `Dac` (3) | defined and continuous, possibly unbounded |
| `Com` (4) | common: defined, continuous, bounded inputs, bounded nonempty result |

Propagation is meet (lattice minimum) with `Ill` absorbing and `Com` the identity;
the six lattice laws are Kani proved (see [verification-map](verification-map.md)).
External set: IEEE Std 1788-2015 decoration system, via
[ieee-1788-2015](ieee-1788-2015.md) and the open exposition in
[revol-1788-introduction](revol-1788-introduction.md). Generator: none; the enum is
the registry.

## Rounding directions

Source of truth: the `RoundFloat` trait surface in `crates/round-float/src/lib.rs`.
Two directions, downward and upward, as `_down`/`_up` method pairs over
`add/sub/mul/div/sqrt` (and `exp/ln` via `RoundTranscendental`). External set: the
rounding direction attributes of IEEE Std 754-2019 ([ieee-754-2019](ieee-754-2019.md));
the crate deliberately exposes only the two directed attributes interval arithmetic
needs, not all five of the standard. Generator: none; the trait is the registry.

## Exception behaviors

IEEE 1788 names exceptional conditions (undefined operation, possibly undefined
operation, interval overflow, ill formed interval). `interval-1788` does not carry a
flag or signal registry: exceptional histories live in the decoration of a decorated
interval, and constructor failures live in `crates/interval-1788/src/error.rs`. The
mapping (what 1788 calls an exception versus what the crate returns) is recorded in
the crate's `spec.rs` prose. Generator: none. This is the one registry whose external
set is larger than its in tree mirror; the gap is deliberate (no global state in a
`no_std` crate) and belongs in any future conformance statement.
