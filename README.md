# enclosure

Pure Rust rigorous-numerics crates that *enclose* a true value rather than
approximate it. Where a floating-point calculation gives an answer that is
probably close, these crates give a result that provably contains the truth.

This repository is a Cargo workspace. Each crate is independently versioned and
independently published; the workspace is packaging, not a unifying framework, so
every crate stands alone.

| crate | what it is |
| ----- | ---------- |
| [`round-float`](crates/round-float) | the directed-rounding float contract (foundation) |
| [`interval-1788`](crates/interval-1788) | rigorous IEEE 1788-2015 interval arithmetic (inf-sup) |
| [`affine-arith`](crates/affine-arith) | Stolfi affine arithmetic (in progress) |

Dependencies point one way: affine-arith and interval-1788 are each generic over
round-float, and affine-arith reduces to an interval-1788 interval. The whole
family is `no_std` (round-float allocation free, affine-arith with `alloc`) and
builds on stable Rust, pinned in `rust-toolchain.toml` so it stays consumable by
both stable and nightly downstreams.

Each crate carries its own README, including a "How this crate is developed"
disclosure of the AI-assisted process, and its own decision records. The
workspace-level architecture decisions live in `docs/decisions`.

## License

Every crate is licensed under either of Apache License, Version 2.0 or the MIT
license at your option.
