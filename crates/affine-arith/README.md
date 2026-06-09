# affine-arith

Stolfi affine arithmetic over a directed-rounding float, in pure Rust.

**Scaffold.** This crate is a placeholder that fixes its place in the `enclosure`
workspace: it depends on `interval-1788` and `round-float`, and it will provide
rigorous self-validated arithmetic in the affine form

```text
    x̂ = x₀ + x₁ε₁ + x₂ε₂ + ... + xₙεₙ,    εᵢ ∈ [−1, 1],
```

where the shared noise symbols εᵢ track correlations between quantities, so that
an expression like `x - x` cancels exactly where interval arithmetic loses the
correlation and widens. The form is generic over `round-float`'s directed-rounding
contract and reduces to an `interval-1788` interval for a guaranteed enclosure.
The crate is `no_std` with `alloc`.

The affine form, its operations, the elementary functions, and the verification
lane arrive in phases. The "How affine-arith is developed" disclosure, which
names the specific failure mode the process is most exposed to, ships with the
implementation rather than ahead of it.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
