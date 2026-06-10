//! Rigorous interval arithmetic following IEEE Std 1788-2015, in pure Rust.
//!
//! An interval is a guaranteed enclosure of a real quantity rather than a
//! single approximate number. `interval-1788` implements the inf-sup
//! (endpoint) representation of the standard's set-based flavor: a nonempty
//! interval is the closed real set `[lo, hi]` with `lo <= hi`, endpoints may be
//! the extended reals (so `[-inf, hi]`, `[lo, +inf]`, and `Entire =
//! [-inf, +inf]` are representable), and the empty set is a first-class value.
//!
//! # Generic over the float
//!
//! The standard is defined over an underlying number format. This crate is
//! generic over [`RoundFloat`], the directed-rounding float contract it needs,
//! so the same interval logic runs over any correctly-rounded float that
//! supplies outward rounding:
//!
//! - ferrodec `Decimal128` (the IEEE 754-2019 decimal float), via a newtype in
//!   the consuming crate;
//! - pfloat's arbitrary-precision floats, in pfloat's own repo;
//! - an `f64` verification fixture, from `round-float`, enabled here by the
//!   `fixture` feature.
//!
//! Outward rounding (lower endpoint toward minus infinity, upper endpoint
//! toward plus infinity) is the entire correctness story for arithmetic: when
//! the backend is correctly rounded in every mode, the enclosure is tight, with
//! no defensive widening.
//!
//! # The f64 fixture is sound, not tight
//!
//! The `fixture` feature pulls `round-float`'s `RoundFloat for f64` instance,
//! which exists to make the enclosure theorem machine-checkable (CBMC models
//! `f64` bit-precisely) and to drive host property tests. It rounds each result
//! outward by one step with `next_up` / `next_down` unconditionally, so it is
//! always a correct enclosure but is up to one unit in the last place wider than
//! necessary. Tightness is a property of a correctly-rounded backend, verified
//! there by property test, never by the fixture. See [`spec`] for the laws and
//! the `docs/decisions` records for why the split exists.
//!
//! # Scope
//!
//! This is an early version. The roadmap is the full set-based flavor (the
//! forward operation set, the numeric and boolean functions, set operations,
//! the `{com, dac, def, trv, ill}` decoration system, and Level 2 conformance).
//! What is implemented at any version is stated per module; behavior that is
//! designed but not yet present is named as such rather than implied.
//!
//! # No std
//!
//! The crate is `#![no_std]` and allocation free. The `std` feature only adds
//! a `std::error::Error` impl for [`IntervalError`].

#![no_std]
#![forbid(unsafe_code)]

// Only for the `std::error::Error` impl on `IntervalError` under the `std`
// feature; the crate is otherwise core-only and allocation free.
#[cfg(feature = "std")]
extern crate std;

pub mod decorated;
pub mod decoration;
pub mod elementary;
pub mod error;
pub mod functions;
pub mod interval;
pub mod ops;
pub mod spec;

#[cfg(all(kani, feature = "fixture"))]
mod kani_harness;

pub use decorated::DecoratedInterval;
pub use decoration::Decoration;
pub use error::IntervalError;
pub use interval::Interval;
// Re-exported from the foundation crate so downstream `impl RoundFloat for _`
// (the SMIL/ferrodec backend) keeps resolving `interval_1788::RoundFloat`, and
// likewise the extension trait the elementary functions are gated on.
pub use round_float::{RoundFloat, RoundTranscendental};
