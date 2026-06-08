# interval-1788

Pure Rust rigorous interval arithmetic following IEEE Std 1788-2015, generic
over a directed-rounding float.

An interval is a guaranteed enclosure of a real quantity rather than a single
approximate number. Where a floating-point calculation gives you an answer that
is probably close, an interval gives you a range that provably contains the true
result. `interval-1788` implements the inf-sup (endpoint) representation of the
standard's set-based flavor: a nonempty interval is the closed real set
`[lo, hi]`, endpoints may be the extended reals so unbounded intervals and the
whole line are representable, and the empty set is a first-class value.

The crate is `no_std`, allocation free, and depends on nothing in its core.

## Generic over the float

IEEE 1788 is defined over an underlying number format. This crate is generic
over `RoundFloat`, the directed-rounding contract it needs, so the same interval
logic runs over any correctly-rounded float that supplies outward rounding:

- ferrodec `Decimal128`, the IEEE 754-2019 decimal float, wired through a newtype
  in the consuming crate;
- pfloat's arbitrary-precision floats, in pfloat's own repository;
- an `f64` verification fixture, shipped here behind the `fixture` feature.

Outward rounding (the lower endpoint toward minus infinity, the upper toward plus
infinity) is the whole correctness story for arithmetic. When the backend is
correctly rounded in every mode, the enclosure is tight, with no defensive
widening.

## Where it sits

`interval-1788` is the inf-sup member of a family of pure Rust rigorous-numerics
crates: ferrodec (a single IEEE 754-2019 value), pfloat (one arbitrary-precision
correctly-rounded value), pfloat-ball (a rigorous enclosure in the midpoint plus
radius representation), and this crate (a rigorous enclosure in the endpoint
representation). The two enclosure crates are siblings over the same idea in two
shapes; the directed-rounding contract they share is designed to move into a
small foundation crate once both are in use and the shape is proven.

## Status

Early version (v0.1.0), not yet published to crates.io. The construction layer
(the type, its invariant, the `RoundFloat` contract, the f64 fixture, and the
written-down specification) is in place. The forward operation set, the numeric
and boolean functions, set operations, the decoration system, and Level 2
conformance are designed and arrive in later phases; what is present at any
version is stated per module. The architecture decision records in the
repository carry the rationale.

## How interval-1788 is developed

This is an open disclosure of the development process so users can judge for
themselves whether the resulting code meets their bar.

**Authorship and collaboration.** Parnell Springmeyer is the author of record.
interval-1788 is developed in collaboration with Claude, an AI coding agent from
Anthropic. Parnell owns architecture, acceptance criteria, test and verification
strategy, and release boundaries. Claude drafts the implementation, writes and
runs tests and verification harnesses, and produces analysis under that
direction. **Parnell does not review the generated code line by line.** Human
oversight operates at the level of design, strategy, and outcomes: does the
architecture make sense, are the right invariants being checked, does the
verification strategy cover the risk surface, do the tests and proofs pass.
Merges to main are GPG signed by Parnell to attest to that level of review, not
to an audit of every line.

**Provenance.** Implementations derive from primary sources: IEEE Std 1788-2015
for the interval model, the operation set, and the decoration semantics, the open
introduction to the standard by Revol for the same, and IEEE Std 1788.1-2017 for
the simplified profile. The agent is instructed to cite recalled sources rather
than reproduce verbatim, to surface provenance uncertainty rather than hide it,
and to choose surface forms (identifiers, helper decomposition, file layout)
fresh for idiomatic Rust rather than copying from existing reference
implementations. The C++ reference, the GNU Octave interval package, and the
Julia interval libraries serve as behavioral oracles whose outputs are
cross-checked, not as code to adapt.

These are instructions to the agent, not guarantees about every line of output. A
verbatim reproduction or an unflagged derivation could slip through. The
project's defense against that is the instruction discipline above plus the human
reviewer's ability to notice architectural smells that suggest a problem
upstream, not a clean room audit. If you spot a passage that reads like a copy
from a source it should not be copied from, please open an issue.

**Verification.** The verification design places correctness in the type system
where it can: the lower-or-equal-upper invariant is held by construction and
cannot be expressed away, and the directed-rounding contract states the soundness
obligation each backend must meet. It places correctness in formal proof
harnesses (Kani) where the cost is justified: the enclosure theorem is checked
over the f64 fixture, which CBMC can model bit for bit, and the decoration
propagation is a finite lattice amenable to the same treatment. It places
correctness in property tests against a correctly-rounded backend (enclosure of
the point result and the tightness the fixture cannot show), in the
self-consistency of the fundamental theorem of interval arithmetic, in the
IEEE 1788 conformance test vectors, and in differential tests against a trusted
reference on a separate lane reached out of process so its copyleft does not
enter the link graph. Significant decisions are recorded as ADRs in the
repository.

**Scope.** interval-1788 is a personal project. The intended consumer is the
broader Rust scientific and embedded ecosystem: anyone who needs a guaranteed
enclosure rather than a probable approximation, in pure Rust with no C toolchain.
Durability and quality are goals, but this is not a funded library with a
maintenance team behind it. The crate is an early version with the construction
layer in place and the operation set on the roadmap. The repository is public for
users who want to read or follow the work.

**What this does not promise.** AI collaboration does not transfer
responsibility. The author is accountable for what ships under his name. The
disciplines above narrow the failure surface; they do not eliminate it. In
particular, this process is most exposed to subtle bugs that a careful human
reading of the code would catch but tests, types, and formal verification would
not. For rigorous interval arithmetic that specifically includes an enclosure
that comes out narrower than the truth on a boundary case no proof or property
test reached: an endpoint mis-rounded near a power of two, a four-corner product
whose extreme value sits at a sign combination no test exercised, or a decoration
that claims more about a computation's history than actually holds. An
unsoundly narrow enclosure is the one defect that turns a downstream result into
a confident falsehood, which is why soundness is weighted above tightness
throughout. Issues are welcome and will be triaged as time allows; no SLA is
offered. This README describes the project's development process and is not a
warranty; see the LICENSE file for the legal terms governing use.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in this crate by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
