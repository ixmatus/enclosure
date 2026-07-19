//! The ITF1788 arithmetic reverse-operation vectors over `TightF64`
//! (bit-exact), hardening decision record 0006 part 2's promise.
//!
//! Wiring smoke only at this stage: proves the arithmetic reverses exist and
//! compute exactly over the tight backend. The full hand-translation of the
//! `sqrRev`/`absRev`/`pownRev`/`mulRev`/`mulRevToPair` vectors lands in the
//! conformance slice (bead enc-ac4) and replaces this file's body. The
//! trigonometric, hyperbolic, and power reverses need traits `TightF64` does
//! not implement; they stay with the fixture enclosure lanes and the split is
//! recorded in the conformance document.

use interval_1788::Interval;
use round_float::TightF64;

#[test]
fn tight_reverse_smoke() {
    let c = Interval::new(TightF64(0.0), TightF64(25.0)).unwrap();
    let x = Interval::new(TightF64(-4.0), TightF64(7.0)).unwrap();
    // sqrt(25) is exact over a correctly rounded backend, so the reverse of
    // the square is pinned bit-exactly, the assertion mode of the whole lane.
    let r = c.sqr_rev(x);
    assert_eq!(r.inf().0, -4.0);
    assert_eq!(r.sup().0, 5.0);
}
