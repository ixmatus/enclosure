# affine-arith

Stolfi affine arithmetic over a directed-rounding float, in pure Rust.

An affine form represents a quantity as a central value plus a linear combination
of shared noise symbols,

```text
    x̂ = x₀ + x₁ε₁ + x₂ε₂ + ... + xₙεₙ,    εᵢ ∈ [−1, 1],
```

so that a correlated combination like `x - x` cancels exactly where interval
arithmetic, having forgotten the correlation, would widen. Every form still
reduces to a guaranteed `interval-1788` interval, so affine arithmetic buys
tightness on correlated expressions without giving up rigor. The form is generic
over `round-float`'s directed-rounding contract. The crate is `no_std` with
`alloc`.

## Status

Early version (v0.1.0). The construction layer (the affine form, the noise-symbol
source, the interval round-trip), the arithmetic (addition, subtraction, scaling,
and the rigor-critical affine-by-affine multiply), and the nonlinear elementary
functions (`recip`, `sqrt`, `sqr`, `exp`, and `ln`, by Chebyshev approximation)
are in place, generic over `round-float`'s directed-rounding contract. Property
lanes exercise enclosure and tightness over the f64 fixture, and a nightly lane
certifies the transcendental bounds against a correctly-rounded reference. The API
may break between 0.x releases; the workspace decision records carry the design.

## How affine-arith is developed

This is an open disclosure of the development process so users can judge for
themselves whether the resulting code meets their bar.

**Authorship and collaboration.** Parnell Springmeyer is the author of record.
affine-arith is developed in collaboration with Claude, an AI coding agent from
Anthropic. Parnell owns architecture, acceptance criteria, test and verification
strategy, and release boundaries. Claude drafts the implementation, writes and
runs tests and verification harnesses, and produces analysis under that direction.
**Parnell does not review the generated code line by line.** Human oversight
operates at the level of design, strategy, and outcomes: does the architecture
make sense, are the right invariants being checked, does the verification strategy
cover the risk surface, do the tests and proofs pass. Merges to main are GPG
signed by Parnell to attest to that level of review, not to an audit of every
line.

**Provenance.** Implementations derive from primary sources: the Stolfi affine
arithmetic model and the univariate Chebyshev approximation from de Figueiredo and
Stolfi's "Affine Arithmetic: Concepts and Applications" (2004), Stolfi and de
Figueiredo's "Self-Validated Numerical Methods and Applications" (1997), and Comba
and Stolfi (1993), with the slope and error bounds re-derived from the convexity
geometry rather than transcribed. The agent is instructed to cite recalled sources
rather than reproduce verbatim, to surface provenance uncertainty rather than hide
it, and to choose surface forms (identifiers, helper decomposition, file layout)
fresh for idiomatic Rust rather than copying from existing affine-arithmetic
libraries. pfloat-libm, a correctly-rounded libm, and interval-1788 serve as
behavioral oracles whose outputs are cross-checked, not as code to adapt.

These are instructions to the agent, not guarantees about every line of output. A
verbatim reproduction or an unflagged derivation could slip through. The project's
defense against that is the instruction discipline above plus the human reviewer's
ability to notice architectural smells that suggest a problem upstream, not a
clean room audit. If you spot a passage that reads like a copy from a source it
should not be copied from, please open an issue.

**Verification.** The verification design places correctness in the type system
where it can: an invariant lifetime brand ties every form to the noise-symbol
source that made it, so combining forms from two different sources, whose symbols
would silently collide, is a compile error rather than a runtime hazard, and the
directed-rounding contract states the soundness obligation each backend must meet.
It places correctness in property tests that exercise the enclosure law over many
inputs, single- and multi-symbol, including the correlated expressions where
affine arithmetic earns its tightness over intervals. For the transcendentals,
whose f64 fixture bounds rest on the host libm's accuracy goal rather than on a
proof, a nightly lane checks the exp and log enclosures against a pure-Rust
correctly-rounded reference run on its own toolchain, so the soundness does not
hang on trusting that goal. The design is further meant to discharge the discrete
domain logic with a model checker and to lock the cross-source compile error as a
regression guard. Significant decisions, including where a proof gives way to a
test and why, are recorded as ADRs in the repository.

**Scope.** affine-arith is a personal project. The intended consumer is the
broader Rust scientific and embedded ecosystem: anyone who needs a guaranteed
enclosure that stays tight when a quantity is reused across a computation, in pure
Rust with no C toolchain. Durability and quality are goals, but this is not a
funded library with a maintenance team behind it. The crate is an early version
with the form, the arithmetic, and the elementary functions in place. The
repository is public for users who want to read or follow the work.

**What this does not promise.** AI collaboration does not transfer responsibility.
The author is accountable for what ships under his name. The disciplines above
narrow the failure surface; they do not eliminate it. In particular, this process
is most exposed to subtle bugs that a careful human reading of the code would
catch but tests, types, and formal verification would not. For affine arithmetic
that specifically includes an enclosure that comes out narrower than the truth
because a linearization's error term was understated: a Chebyshev slope or
residual bound rounded the wrong way so the fresh noise symbol under-covers the
approximation error on a curved stretch no sample landed on, a per-operation
roundoff term dropped from the accumulated bound, or a transcendental bound that
trusts the host libm past its accuracy on an input the correctly-rounded oracle's
grid did not reach. An unsoundly narrow enclosure is the one defect that turns a
downstream result into a confident falsehood, which is why soundness is weighted
above tightness throughout, and why the multiply and the elementary functions fold
every approximation and rounding error outward into a fresh noise symbol. Issues
are welcome and will be triaged as time allows; no SLA is offered. This README
describes the project's development process and is not a warranty; see the LICENSE
file for the legal terms governing use.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in this crate by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
