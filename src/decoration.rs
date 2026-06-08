//! [`Decoration`]: the set-based flavor's decoration lattice.
//!
//! A decoration is a piece of information attached to an interval that summarizes
//! the history of the computation that produced it: whether every operation along
//! the way was defined and continuous on its inputs, whether the inputs stayed
//! bounded, or whether something went wrong. The set-based flavor uses five
//! decorations, totally ordered from strongest to weakest:
//!
//! - `com` (common): defined and continuous throughout, on bounded inputs, with a
//!   bounded nonempty result.
//! - `dac` (defined and continuous): defined and continuous, but somewhere
//!   unbounded.
//! - `def` (defined): defined everywhere on the inputs, but not everywhere
//!   continuous.
//! - `trv` (trivial): nothing is guaranteed; this always holds.
//! - `ill` (ill-formed): the result of an invalid construction (a `NaI`). It
//!   poisons every computation it touches.
//!
//! Propagation combines decorations by taking the weakest (the lattice meet): a
//! result is decorated with the minimum of the decorations its inputs carried and
//! the decoration the operation itself earned on those inputs. `ill` is absorbing
//! under the meet, and `com` is the identity.

use core::fmt;

/// A set-based flavor decoration, ordered from weakest (`Ill`) to strongest
/// (`Com`). The derived ordering is the lattice order: a smaller value is weaker,
/// so [`min`](Ord::min) (exposed as [`meet`](Decoration::meet)) is the
/// propagation combine.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Decoration {
    /// Ill-formed (NaI): the poison from an invalid construction.
    Ill = 0,
    /// Trivial: nothing guaranteed.
    Trv = 1,
    /// Defined everywhere on the inputs, but not everywhere continuous.
    Def = 2,
    /// Defined and continuous on the inputs (possibly unbounded).
    Dac = 3,
    /// Common: defined and continuous, bounded inputs, bounded nonempty result.
    Com = 4,
}

impl Decoration {
    /// The lattice meet: the weaker of two decorations. This is the propagation
    /// combine, with `Ill` absorbing and `Com` the identity.
    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        if (self as u8) <= (other as u8) {
            self
        } else {
            other
        }
    }
}

impl fmt::Display for Decoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Decoration::Ill => "ill",
            Decoration::Trv => "trv",
            Decoration::Def => "def",
            Decoration::Dac => "dac",
            Decoration::Com => "com",
        };
        f.write_str(s)
    }
}
