# round-float

The directed-rounding float contract for rigorous numerics, in pure Rust.

A rigorous enclosure of a real quantity is only as trustworthy as the rounding
underneath it. `round-float` is the `RoundFloat` trait: the small contract that a
floating-point backend must satisfy for the crates above it to compute guaranteed
bounds. It exposes split-direction arithmetic (`add_down` and `add_up`,
`sub_down` and `sub_up`, and so on for multiply, divide, and square root), so a
lower bound always rounds toward minus infinity and an upper bound always toward
plus infinity. The direction is a property of the call site, not a runtime
parameter, which is what keeps the rigor-critical operations from rounding a
bound the wrong way.

The crate is `no_std`, allocation free, and depends on nothing in its core.

## The contract

For finite inputs, `x.add_down(y)` returns a float less than or equal to the
exact real sum `x + y`, and `x.add_up(y)` returns one greater than or equal to
it; likewise for the other operations. A backend that is correctly rounded toward
minus and plus infinity makes these tight, returning the nearest representable
bound. A backend that only guarantees soundness may return a looser bound, but
never one on the wrong side. Soundness is the load-bearing obligation; tightness
is a quality a backend may add.

## Where it sits

`round-float` is the foundation of a family of pure Rust rigorous-numerics
crates. The interval crate `interval-1788` (guaranteed enclosures in the endpoint
representation) and the affine-arithmetic crate `affine-arith` (Stolfi affine
forms) are both generic over `RoundFloat`, so one correctly-rounded backend
serves every enclosure shape. Production instances live in the consuming crates:
ferrodec's IEEE 754-2019 `Decimal128` wired through a newtype, and pfloat's
arbitrary-precision floats in pfloat's own repository. Behind the `f64` feature,
this crate ships one instance of its own, a verification and host-test fixture
that is sound but deliberately not tight; it rounds each result outward by a
single step so the enclosure laws can be machine-checked and property-tested with
no heavy dependency.

## Status

Early version (v0.1.0), not yet published to crates.io. The trait and the f64
fixture are in place. A dedicated in-crate verification lane for the fixture's
soundness is on the roadmap; today that soundness is exercised by the property
lanes of the crates built on the contract. The architecture decision records in
the repository carry the rationale.

## How round-float is developed

This is an open disclosure of the development process so users can judge for
themselves whether the resulting code meets their bar.

**Authorship and collaboration.** Parnell Springmeyer is the author of record.
round-float is developed in collaboration with Claude, an AI coding agent from
Anthropic. Parnell owns architecture, acceptance criteria, test and verification
strategy, and release boundaries. Claude drafts the implementation, writes and
runs tests and verification harnesses, and produces analysis under that
direction. **Parnell does not review the generated code line by line.** Human
oversight operates at the level of design, strategy, and outcomes: does the
contract say the right thing, are the right invariants being checked, does the
verification strategy cover the risk surface, do the tests and proofs pass.
Merges to main are GPG signed by Parnell to attest to that level of review, not
to an audit of every line.

**Provenance.** The contract derives from primary sources: the directed-rounding
requirements of rigorous interval arithmetic as set out in IEEE Std 1788-2015 and
the open introduction to that standard, and the rounding behavior specified by
IEEE Std 754-2019 for the backends that implement it. The agent is instructed to
cite recalled sources rather than reproduce verbatim, to surface provenance
uncertainty rather than hide it, and to choose surface forms (identifiers, helper
decomposition, file layout) fresh for idiomatic Rust rather than copying from
existing reference implementations.

These are instructions to the agent, not guarantees about every line of output. A
verbatim reproduction or an unflagged derivation could slip through. The
project's defense against that is the instruction discipline above plus the human
reviewer's ability to notice architectural smells that suggest a problem
upstream, not a clean room audit. If you spot a passage that reads like a copy
from a source it should not be copied from, please open an issue.

**Verification.** The contract is expressed in the type system: the trait states
the soundness obligation each backend must meet, and the split-direction methods
make the rounding direction a property of the call site rather than a value that
could be passed wrong. The one instance round-float ships, the f64 fixture, is
designed to be sound but not tight, so its outward step can be checked rather than
trusted; its soundness is exercised today by the property lanes of the crates
built on the contract, and a dedicated in-crate lane is on the roadmap.
Significant decisions are recorded as ADRs in the repository.

**Scope.** round-float is a personal project. The intended consumer is the
broader Rust scientific and embedded ecosystem, by way of the enclosure crates
that build on it: anyone who needs guaranteed bounds rather than probable
approximations, in pure Rust with no C toolchain. Durability and quality are
goals, but this is not a funded library with a maintenance team behind it. The
crate is an early version. The repository is public for users who want to read or
follow the work.

**What this does not promise.** AI collaboration does not transfer
responsibility. The author is accountable for what ships under his name. The
disciplines above narrow the failure surface; they do not eliminate it. In
particular, this process is most exposed to subtle bugs that a careful human
reading of the code would catch but tests, types, and formal verification would
not. For a directed-rounding contract that specifically includes a method
returning a bound on the wrong side of the true result on an input no property
test reached: an `add_up` that lands just below the exact sum near an overflow or
subnormal boundary, or an outward step that collapses at a signed zero. A single
wrong-sided bound is the defect that turns every enclosure built on top into a
confident falsehood, which is why soundness is the obligation the contract states
and weights above tightness. Issues are welcome and will be triaged as time
allows; no SLA is offered. This README describes the project's development process
and is not a warranty; see the LICENSE file for the legal terms governing use.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in this crate by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
