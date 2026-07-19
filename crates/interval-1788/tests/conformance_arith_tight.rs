//! The ITF1788 arithmetic conformance vectors over `TightF64` (bit-exact).
//!
//! Wiring smoke only at this stage: proves `Interval<TightF64>` constructs,
//! computes with correctly rounded arithmetic, and round-trips exact
//! endpoints. The full hand-translation of the `libieeep1788_elem.itl`
//! arithmetic testcases lands in the conformance slice (bead enc-ac4) and
//! replaces this file's body.

use interval_1788::Interval;
use round_float::TightF64;

#[test]
fn tight_backend_smoke() {
    let a = Interval::new(TightF64(1.0), TightF64(2.0)).unwrap();
    let b = Interval::new(TightF64(3.0), TightF64(4.0)).unwrap();
    let sum = a + b;
    // Correct rounding makes exact-integer arithmetic exact, so the endpoints
    // are pinned bit-exactly, the assertion mode of the whole lane.
    assert_eq!(sum.inf().0, 4.0);
    assert_eq!(sum.sup().0, 6.0);
    let prod = a * b;
    assert_eq!(prod.inf().0, 3.0);
    assert_eq!(prod.sup().0, 8.0);
}
