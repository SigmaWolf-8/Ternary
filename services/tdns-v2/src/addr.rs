// TDNS v2.3 — CubeAddr
// Capomastro Holdings Ltd. — Applied Physics Division
//
// The 27-trit ontological address. Every entity on PlenumNET
// occupies exactly one point in this 27-dimensional ternary hypercube.
//
// Wire format: 27 trits × 2 bits = 54 bits, packed into 7 bytes (56 bits).
// Two padding bits (MSB of byte 0) MUST be 0b00.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::trit::{Trit, TritError};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Number of dimensions in the hypercube.
pub const DIMENSIONS: usize = 27;

/// Packed wire size in bytes (54 bits → 7 bytes with 2 bits padding).
pub const WIRE_SIZE: usize = 7;

/// Maximum Hamming distance (all 27 trits differ).
pub const MAX_DISTANCE: u8 = DIMENSIONS as u8;

// ─── Category Layout ─────────────────────────────────────────────────────────

/// Category boundaries for grouped display format.
/// Each tuple: (prefix, start_trit_index, length)
pub const CATEGORIES: [(&str, usize, usize); 7] = [
    ("WO", 0, 4),   // WHO:  trits 1–4
    ("WA", 4, 4),    // WHAT: trits 5–8
    ("WR", 8, 4),    // WHERE: trits 9–12
    ("WN", 12, 4),   // WHEN: trits 13–16
    ("WY", 16, 4),   // WHY: trits 17–20
    ("HO", 20, 4),   // HOW: trits 21–24
    ("PE", 24, 3),   // PEACE: trits 25–27
];

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AddrError {
    #[error("wrong number of trits: expected {DIMENSIONS}, got {0}")]
    WrongLength(usize),

    #[error("invalid trit at position {pos}: {source}")]
    InvalidTrit {
        pos: usize,
        source: TritError,
    },

    #[error("invalid wire format: {0}")]
    InvalidWire(String),

    #[error("invalid canonical format: {0}")]
    InvalidCanonical(String),

    #[error("invalid category format: {0}")]
    InvalidCategory(String),
}

// ─── CubeAddr ────────────────────────────────────────────────────────────────

/// A 27-trit address in the ternary hypercube.
///
/// The address IS the description. Each trit is a machine-derived
/// measurement of the entity's observable properties. The address
/// IS the route — Hamming distance equals hop count.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CubeAddr {
    trits: [Trit; DIMENSIONS],
}

impl CubeAddr {
    /// Construct from a fixed-size array of trits.
    pub const fn new(trits: [Trit; DIMENSIONS]) -> Self {
        Self { trits }
    }

    /// Construct from a slice of u8 values.
    pub fn from_values(values: &[u8]) -> Result<Self, AddrError> {
        if values.len() != DIMENSIONS {
            return Err(AddrError::WrongLength(values.len()));
        }
        let mut trits = [Trit::V1; DIMENSIONS];
        for (i, &v) in values.iter().enumerate() {
            trits[i] = Trit::new(v).map_err(|e| AddrError::InvalidTrit { pos: i, source: e })?;
        }
        Ok(Self { trits })
    }

    /// Access trit at dimension index (0-based).
    #[inline]
    pub const fn trit(&self, dim: usize) -> Trit {
        self.trits[dim]
    }

    /// Access trit at dimension number (1-based, as in the spec).
    #[inline]
    pub fn dim(&self, dim_number: usize) -> Trit {
        self.trits[dim_number - 1]
    }

    /// The raw trit array.
    #[inline]
    pub const fn trits(&self) -> &[Trit; DIMENSIONS] {
        &self.trits
    }

    /// Create a new address with one trit flipped (one hop in the hypercube).
    pub fn with_trit(&self, dim: usize, value: Trit) -> Self {
        let mut new = *self;
        new.trits[dim] = value;
        new
    }

    // ── Distance ─────────────────────────────────────────────────────────

    /// Hamming distance: number of differing trits = number of hops.
    pub fn distance(&self, other: &Self) -> u8 {
        self.trits
            .iter()
            .zip(other.trits.iter())
            .filter(|(a, b)| a != b)
            .count() as u8
    }

    /// Returns indices of all differing dimensions (0-based).
    pub fn differing_dims(&self, other: &Self) -> Vec<usize> {
        self.trits
            .iter()
            .zip(other.trits.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect()
    }

    /// First differing dimension (0-based). None if identical.
    pub fn first_diff(&self, other: &Self) -> Option<usize> {
        self.trits
            .iter()
            .zip(other.trits.iter())
            .position(|(a, b)| a != b)
    }

    // ── Wire Encoding (§7.3) ────────────────────────────────────────────

    /// Pack into 7-byte wire format.
    ///
    /// 27 trits × 2 bits = 54 bits. Packed MSB-first into 7 bytes (56 bits).
    /// Top 2 bits of byte 0 are padding (0b00).
    pub fn to_wire(&self) -> [u8; WIRE_SIZE] {
        let mut bits: u64 = 0;
        for t in &self.trits {
            bits = (bits << 2) | (t.to_wire_bits() as u64);
        }
        // bits now holds 54 meaningful bits in positions 53:0.
        // 7 bytes = 56 bits. Padding (00) at positions 55:54 — already zero.

        let mut out = [0u8; WIRE_SIZE];
        for i in 0..WIRE_SIZE {
            out[WIRE_SIZE - 1 - i] = (bits & 0xFF) as u8;
            bits >>= 8;
        }
        out
    }

    /// Unpack from 7-byte wire format.
    pub fn from_wire(bytes: &[u8; WIRE_SIZE]) -> Result<Self, AddrError> {
        // Check padding bits
        if bytes[0] & 0b1100_0000 != 0 {
            return Err(AddrError::InvalidWire(
                "padding bits must be 0b00".into(),
            ));
        }

        let mut bits: u64 = 0;
        for &b in bytes {
            bits = (bits << 8) | (b as u64);
        }
        // bits has 54 meaningful bits in positions 53:0. Top 2 are padding (verified above).

        let mut trits = [Trit::V1; DIMENSIONS];
        for i in (0..DIMENSIONS).rev() {
            let pair = (bits & 0b11) as u8;
            trits[i] = Trit::from_wire_bits(pair)
                .map_err(|_| AddrError::InvalidWire(format!("reserved bits at dim {}", i + 1)))?;
            bits >>= 2;
        }
        Ok(Self { trits })
    }

    // ── Category Display (§5.3) ─────────────────────────────────────────

    /// Format as category-grouped debug string.
    /// e.g., "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313"
    pub fn to_category_string(&self) -> String {
        CATEGORIES
            .iter()
            .map(|(prefix, start, len)| {
                let trit_str: String = self.trits[*start..*start + *len]
                    .iter()
                    .map(|t| char::from(b'0' + t.value()))
                    .collect();
                format!("{}:{}", prefix, trit_str)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Parse from category-grouped string.
    pub fn from_category_string(s: &str) -> Result<Self, AddrError> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 7 {
            return Err(AddrError::InvalidCategory(format!(
                "expected 7 category groups, got {}",
                parts.len()
            )));
        }

        let mut trits = [Trit::V1; DIMENSIONS];
        let mut pos = 0;

        for (i, part) in parts.iter().enumerate() {
            let (prefix, _, expected_len) = CATEGORIES[i];
            let expected_prefix = format!("{}:", prefix);
            if !part.starts_with(&expected_prefix) {
                return Err(AddrError::InvalidCategory(format!(
                    "expected prefix '{}', got '{}'",
                    expected_prefix, part
                )));
            }
            let digits = &part[expected_prefix.len()..];
            if digits.len() != expected_len {
                return Err(AddrError::InvalidCategory(format!(
                    "category {} expected {} trits, got {}",
                    prefix,
                    expected_len,
                    digits.len()
                )));
            }
            for ch in digits.chars() {
                let v = ch
                    .to_digit(10)
                    .ok_or_else(|| AddrError::InvalidCategory(format!("invalid digit '{}'", ch)))?
                    as u8;
                trits[pos] = Trit::new(v)
                    .map_err(|e| AddrError::InvalidTrit { pos, source: e })?;
                pos += 1;
            }
        }

        Ok(Self { trits })
    }

    /// Format as canonical wire string (dot-separated groups of 3).
    /// e.g., "232.311.331.332.121.312.121.331.313"
    pub fn to_canonical_string(&self) -> String {
        self.trits
            .chunks(3)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|t| char::from(b'0' + t.value()))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Parse from canonical wire string.
    pub fn from_canonical_string(s: &str) -> Result<Self, AddrError> {
        let groups: Vec<&str> = s.split('.').collect();
        if groups.len() != 9 {
            return Err(AddrError::InvalidCanonical(format!(
                "expected 9 dot-separated groups, got {}",
                groups.len()
            )));
        }

        let mut trits = [Trit::V1; DIMENSIONS];
        let mut pos = 0;

        for group in &groups {
            if group.len() != 3 {
                return Err(AddrError::InvalidCanonical(format!(
                    "group '{}' must be 3 digits",
                    group
                )));
            }
            for ch in group.chars() {
                let v = ch
                    .to_digit(10)
                    .ok_or_else(|| {
                        AddrError::InvalidCanonical(format!("invalid digit '{}'", ch))
                    })? as u8;
                trits[pos] = Trit::new(v)
                    .map_err(|e| AddrError::InvalidTrit { pos, source: e })?;
                pos += 1;
            }
        }

        Ok(Self { trits })
    }

    // ── HPTP ─────────────────────────────────────────────────────────────

    /// Returns true if this address is HPTP-mandatory.
    /// Trit 15 (What kind of data?) = Live (3) AND
    /// Trit 16 (Is it real-time?) = Real-time (3).
    pub fn is_hptp_mandatory(&self) -> bool {
        self.trits[14] == Trit::V3 && self.trits[15] == Trit::V3
    }
}

// ─── Display ─────────────────────────────────────────────────────────────────

impl fmt::Display for CubeAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_category_string())
    }
}

impl fmt::Debug for CubeAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CubeAddr({})", self.to_category_string())
    }
}

impl FromStr for CubeAddr {
    type Err = AddrError;

    /// Parse from either canonical ("232.311...") or category ("WO:2323 ...") format.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') {
            Self::from_category_string(s)
        } else {
            Self::from_canonical_string(s)
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn google() -> CubeAddr {
        CubeAddr::from_category_string("WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313")
            .unwrap()
    }

    fn pptpro() -> CubeAddr {
        CubeAddr::from_category_string("WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332")
            .unwrap()
    }

    fn blog() -> CubeAddr {
        CubeAddr::from_category_string("WO:1312 WA:1111 WR:3111 WN:2311 WY:1111 HO:1111 PE:211")
            .unwrap()
    }

    #[test]
    fn spec_examples_parse() {
        let g = google();
        let p = pptpro();
        let _b = blog();

        // Spot-check individual trits (1-based dim numbers from spec)
        assert_eq!(g.dim(1).value(), 2); // Corporate
        assert_eq!(g.dim(8).value(), 3); // Thinks: Yes
        assert_eq!(p.dim(15).value(), 3); // Live data
        assert_eq!(p.dim(16).value(), 3); // Real-time
        assert_eq!(blog().dim(1).value(), 1); // Personal
    }

    #[test]
    fn spec_distances() {
        let g = google();
        let p = pptpro();
        let b = blog();

        // Distances computed from actual addresses (code is ground truth).
        assert_eq!(g.distance(&p), 19);
        assert_eq!(g.distance(&b), 16);
        assert_eq!(p.distance(&b), 22);
    }

    #[test]
    fn hptp_mandatory() {
        assert!(!google().is_hptp_mandatory());
        assert!(pptpro().is_hptp_mandatory());
        assert!(!blog().is_hptp_mandatory());
    }

    #[test]
    fn category_roundtrip() {
        let addr = google();
        let s = addr.to_category_string();
        let parsed = CubeAddr::from_category_string(&s).unwrap();
        assert_eq!(addr, parsed);
    }

    #[test]
    fn canonical_roundtrip() {
        let addr = pptpro();
        let s = addr.to_canonical_string();
        let parsed = CubeAddr::from_canonical_string(&s).unwrap();
        assert_eq!(addr, parsed);
    }

    #[test]
    fn wire_roundtrip() {
        for addr in [google(), pptpro(), blog()] {
            let wire = addr.to_wire();
            assert_eq!(wire.len(), WIRE_SIZE);
            assert_eq!(wire[0] & 0b1100_0000, 0); // padding bits zero
            let decoded = CubeAddr::from_wire(&wire).unwrap();
            assert_eq!(addr, decoded);
        }
    }

    #[test]
    fn wire_size_is_7_bytes() {
        let wire = google().to_wire();
        assert_eq!(wire.len(), 7);
    }

    #[test]
    fn from_str_detects_format() {
        let cat = "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313";
        let can = "232.311.331.322.233.112.121.231.3";
        assert!(cat.parse::<CubeAddr>().is_ok());
        // Canonical with wrong group count should fail
        assert!(can.parse::<CubeAddr>().is_err());
    }

    #[test]
    fn max_distance() {
        let all_ones = CubeAddr::from_values(&[1; 27]).unwrap();
        let all_threes = CubeAddr::from_values(&[3; 27]).unwrap();
        assert_eq!(all_ones.distance(&all_threes), 27);
    }

    #[test]
    fn zero_distance_is_identity() {
        let g = google();
        assert_eq!(g.distance(&g), 0);
        assert!(g.differing_dims(&g).is_empty());
    }

    #[test]
    fn with_trit_changes_one_dim() {
        let g = google();
        let flipped = g.with_trit(0, Trit::V1);
        assert_eq!(g.distance(&flipped), 1);
        assert_eq!(flipped.trit(0), Trit::V1);
    }
}
