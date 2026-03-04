// TDNS v2.3 — Trit Type
// Capomastro Holdings Ltd. — Applied Physics Division
//
// The atomic unit of the ternary hypercube.
// Values: {1, 2, 3} — never 0.
// Wire encoding: 1=0b01, 2=0b10, 3=0b11, 0b00=reserved/invalid.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TritError {
    #[error("invalid trit value {0}: must be 1, 2, or 3")]
    InvalidValue(u8),

    #[error("invalid wire bits 0b00: reserved encoding")]
    ReservedWireBits,
}

// ─── Trit ────────────────────────────────────────────────────────────────────

/// A single ternary digit with values in {1, 2, 3}.
///
/// This is NOT balanced ternary (-1, 0, +1). This is natural ternary
/// shifted to avoid zero — every trit carries information, no trit is
/// "empty." The value space maps directly to the three possible states
/// of each ontological dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Trit(u8);

impl Trit {
    /// Value 1 — first state (e.g., Personal, Website, No, etc.)
    pub const V1: Trit = Trit(1);
    /// Value 2 — second state (e.g., Corporate, App, Partly, etc.)
    pub const V2: Trit = Trit(2);
    /// Value 3 — third state (e.g., Governance, Device, Yes, etc.)
    pub const V3: Trit = Trit(3);

    /// All valid trit values, in order.
    pub const ALL: [Trit; 3] = [Trit::V1, Trit::V2, Trit::V3];

    /// Create a trit from a u8. Returns error if not in {1, 2, 3}.
    #[inline]
    pub const fn new(value: u8) -> Result<Self, TritError> {
        match value {
            1 | 2 | 3 => Ok(Trit(value)),
            _ => Err(TritError::InvalidValue(value)),
        }
    }

    /// Create a trit from a u8, panicking on invalid input.
    /// Use only in const contexts and tests.
    #[inline]
    pub const fn must(value: u8) -> Self {
        match value {
            1 | 2 | 3 => Trit(value),
            _ => panic!("invalid trit value"),
        }
    }

    /// The raw numeric value (1, 2, or 3).
    #[inline]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Zero-indexed value (0, 1, or 2) for array indexing.
    #[inline]
    pub const fn index(self) -> usize {
        (self.0 - 1) as usize
    }

    /// Wire encoding: 1→0b01, 2→0b10, 3→0b11.
    /// Two bits per trit, 0b00 reserved.
    #[inline]
    pub const fn to_wire_bits(self) -> u8 {
        self.0 // 1=0b01, 2=0b10, 3=0b11 — values ARE the wire encoding
    }

    /// Decode from wire bits. 0b00 is reserved/invalid.
    #[inline]
    pub const fn from_wire_bits(bits: u8) -> Result<Self, TritError> {
        match bits & 0b11 {
            0b01 => Ok(Trit(1)),
            0b10 => Ok(Trit(2)),
            0b11 => Ok(Trit(3)),
            _ => Err(TritError::ReservedWireBits),
        }
    }

    /// Returns true if two trits differ (one hop in the hypercube).
    #[inline]
    pub const fn differs(self, other: Trit) -> bool {
        self.0 != other.0
    }

    /// The other two values this trit could flip to (neighbors in this dimension).
    #[inline]
    pub const fn neighbors(self) -> [Trit; 2] {
        match self.0 {
            1 => [Trit(2), Trit(3)],
            2 => [Trit(1), Trit(3)],
            3 => [Trit(1), Trit(2)],
            _ => unreachable!(),
        }
    }

    /// Project to GF(3): maps {1,2,3} → {0,1,2} for field arithmetic.
    #[inline]
    pub const fn to_gf3(self) -> u8 {
        self.0 - 1
    }

    /// Lift from GF(3): maps {0,1,2} → {1,2,3}.
    #[inline]
    pub const fn from_gf3(v: u8) -> Result<Self, TritError> {
        match v {
            0 => Ok(Trit(1)),
            1 => Ok(Trit(2)),
            2 => Ok(Trit(3)),
            _ => Err(TritError::InvalidValue(v)),
        }
    }
}

impl fmt::Display for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u8> for Trit {
    type Error = TritError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Trit::new(value)
    }
}

impl From<Trit> for u8 {
    #[inline]
    fn from(t: Trit) -> u8 {
        t.0
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_values() {
        assert_eq!(Trit::new(1).unwrap().value(), 1);
        assert_eq!(Trit::new(2).unwrap().value(), 2);
        assert_eq!(Trit::new(3).unwrap().value(), 3);
    }

    #[test]
    fn invalid_values() {
        assert!(Trit::new(0).is_err());
        assert!(Trit::new(4).is_err());
        assert!(Trit::new(255).is_err());
    }

    #[test]
    fn wire_roundtrip() {
        for v in 1..=3u8 {
            let t = Trit::must(v);
            let bits = t.to_wire_bits();
            let decoded = Trit::from_wire_bits(bits).unwrap();
            assert_eq!(t, decoded);
        }
    }

    #[test]
    fn wire_reserved_bits() {
        assert!(Trit::from_wire_bits(0b00).is_err());
    }

    #[test]
    fn gf3_roundtrip() {
        for v in 1..=3u8 {
            let t = Trit::must(v);
            let gf = t.to_gf3();
            let back = Trit::from_gf3(gf).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn neighbors_exclude_self() {
        for t in Trit::ALL {
            let ns = t.neighbors();
            assert!(!ns.contains(&t));
            assert_eq!(ns.len(), 2);
        }
    }

    #[test]
    fn index_maps_correctly() {
        assert_eq!(Trit::V1.index(), 0);
        assert_eq!(Trit::V2.index(), 1);
        assert_eq!(Trit::V3.index(), 2);
    }
}
