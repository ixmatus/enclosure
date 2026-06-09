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

Early version (v0.1.0). The construction layer is in place: the affine form, the
noise-symbol source, the construction of a form from a bounded interval, and the
reduction of a form back to an enclosing interval. The arithmetic operations (the
rigor-critical affine-by-affine multiply), the nonlinear elementary functions,
and the verification lane arrive in later phases; the workspace decision records
carry the plan.

The "How affine-arith is developed" disclosure, which names the specific failure
mode the process is most exposed to, ships with the rigor-critical operations
rather than ahead of them.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in this crate by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
