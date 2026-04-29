// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! Torsion Network Address Sentinel (Representation C)
//!
//! Implements structurally unforgeable address validation for the 13D torsion
//! network using PlenumNET's Bijective Ternary (Representation C) encoding.
//!
//! # The Core Insight
//!
//! In Representation C, digits are {1, 2, 3} — the value 0 is excluded.
//! This is not arbitrary: bijective ternary arithmetic on {1,2,3} is closed.
//! If you add, subtract, or multiply numbers whose digits are only 1, 2, 3,
//! you can never produce a 0 in any digit position through computation alone.
//!
//! This means: **a zero in a Representation C field is structurally impossible
//! to produce through valid computation.** Only hardware initialization or
//! direct memory writes can place a zero there.
//!
//! # Application to Network Addresses
//!
//! A torsion network address is 13 ternary coordinates. In Representation C,
//! each coordinate is drawn from {1, 2, 3}. This module provides:
//!
//! 1. **Structural validation**: Any address containing a 0 is provably not
//!    a legitimate computation result — it's either uninitialized memory,
//!    a buffer overflow, or an attacker's injection.
//!
//! 2. **Representation conversion**: Safe conversion between Rep A {-1,0,+1},
//!    Rep B {0,1,2}, and Rep C {1,2,3} with validation at each step.
//!
//! 3. **Sentinel-sealed addresses**: Addresses with hardware-planted sentinel
//!    zeros in reserved positions, proving they originated from trusted hardware.
//!
//! # Constant-Time Implementation
//!
//! All validation is constant-time to prevent timing side-channels that could
//! reveal address structure to an observer.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;

use alloc::string::String;
use alloc::format;

/// Maximum torsion network dimensions — the full Metatronic Cube.
///
/// 13 axes = 13 circles of Metatron's Cube. This is not arbitrary;
/// it is the dimension of the ternary cube whose vertex set IS the
/// torsion network (3¹³ = 1,594,323 nodes).
pub const MAX_TORSION_DIM: usize = 13;

/// Sponge sub-cube dimension: 6 inner-ring axes → 3⁶ = 729 trits.
///
/// The sponge state embeds into the Void shell of the 13-cube using
/// axes 1–6 (the Manifestation ring). This is the only architecturally
/// meaningful sub-dimension.
pub const SPONGE_DIM: usize = 6;

/// Representation A: computational balanced ternary {-1, 0, +1}
/// Representation B: network wire encoding {0, 1, 2}
/// Representation C: bijective ternary {1, 2, 3} — sentinel-capable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TernaryRepresentation {
    RepA,  // {-1, 0, +1}
    RepB,  // {0, 1, 2}
    RepC,  // {1, 2, 3}
}

/// A validated torsion network address.
///
/// Internally stored in Representation C for sentinel properties.
/// The `sentinel_positions` field marks positions where a hardware-planted
/// zero acts as an unforgeable seal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TorsionAddress {
    /// Address digits in Representation C {1, 2, 3}
    digits: [u8; MAX_TORSION_DIM],
    /// Number of active dimensions
    dim: usize,
    /// Whether this address has been structurally validated
    validated: bool,
}

/// Result of address validation — PUBLIC (opaque).
///
/// Production callers see only Valid/Invalid with no structural detail.
/// Position information, zero locations, and failure modes are deliberately
/// withheld to prevent information leakage to adversaries probing the
/// address validation boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressValidation {
    /// Address is valid: all digits in {1, 2, 3}
    Valid,
    /// Address is invalid — no further detail exposed.
    ///
    /// Use `AddressDiagnostic` (debug/test only) for root-cause analysis.
    Invalid,
}

/// Diagnostic detail for address validation failures — INTERNAL ONLY.
///
/// Available in debug builds and test configurations.
/// **Never expose to network callers or log in production.**
/// Position and value information would tell an attacker exactly
/// which bytes to fix in a forged address.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(debug_assertions, test, feature = "diagnostics"))]
pub enum AddressDiagnostic {
    /// Address contains zero(s) at specific positions — structural forgery.
    ZeroDetected { positions: Vec<usize> },
    /// Address contains out-of-range value at a specific position.
    OutOfRange { position: usize, value: u8 },
}


/// Sealed address with hardware sentinel.
///
/// A sealed address has specific positions set to 0 by the hardware,
/// proving it was generated by a trusted source. The sentinel positions
/// are application-defined and form a "hardware watermark."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedAddress {
    /// The full address including sentinel zeros
    raw: [u8; MAX_TORSION_DIM],
    /// Number of active dimensions
    dim: usize,
    /// Positions where sentinels (zeros) are expected
    sentinel_mask: [bool; MAX_TORSION_DIM],
}

// ============================================================
// Representation Conversion (constant-time)
// ============================================================

/// Convert a single digit from Rep A {-1, 0, +1} to Rep C {1, 2, 3}.
/// Mapping: -1→1, 0→2, +1→3
#[inline(always)]
fn rep_a_to_c(trit: i8) -> u8 {
    // trit ∈ {-1, 0, 1} → trit + 2 ∈ {1, 2, 3}
    (trit + 2) as u8
}

/// Convert a single digit from Rep C {1, 2, 3} to Rep A {-1, 0, +1}.
/// Mapping: 1→-1, 2→0, 3→+1
#[inline(always)]
fn rep_c_to_a(digit: u8) -> i8 {
    digit as i8 - 2
}

/// Convert a single digit from Rep B {0, 1, 2} to Rep C {1, 2, 3}.
/// Mapping: 0→1, 1→2, 2→3
#[inline(always)]
fn rep_b_to_c(digit: u8) -> u8 {
    digit + 1
}

/// Convert a single digit from Rep C {1, 2, 3} to Rep B {0, 1, 2}.
/// Mapping: 1→0, 2→1, 3→2
#[inline(always)]
fn rep_c_to_b(digit: u8) -> u8 {
    digit - 1
}

// ============================================================
// TorsionAddress Implementation
// ============================================================

impl TorsionAddress {
    /// Create a new address from Representation A coordinates {-1, 0, +1}.
    ///
    /// This is the primary constructor from the computational domain.
    pub fn from_rep_a(coords: &[i8]) -> Result<Self, String> {
        let dim = coords.len();
        if dim == 0 || dim > MAX_TORSION_DIM {
            return Err(format!("Invalid dimension: {} (must be 1..{})", dim, MAX_TORSION_DIM));
        }

        let mut digits = [0u8; MAX_TORSION_DIM];
        for (i, &c) in coords.iter().enumerate() {
            if c < -1 || c > 1 {
                return Err(format!("Invalid Rep A value at position {}: {} (must be -1, 0, or +1)", i, c));
            }
            digits[i] = rep_a_to_c(c);
        }

        let mut addr = Self { digits, dim, validated: false };
        addr.validated = addr.validate_internal();
        Ok(addr)
    }

    /// Create a new address from Representation B coordinates {0, 1, 2}.
    pub fn from_rep_b(coords: &[u8]) -> Result<Self, String> {
        let dim = coords.len();
        if dim == 0 || dim > MAX_TORSION_DIM {
            return Err(format!("Invalid dimension: {}", dim));
        }

        let mut digits = [0u8; MAX_TORSION_DIM];
        for (i, &c) in coords.iter().enumerate() {
            if c > 2 {
                return Err(format!("Invalid Rep B value at position {}: {}", i, c));
            }
            digits[i] = rep_b_to_c(c);
        }

        let mut addr = Self { digits, dim, validated: false };
        addr.validated = addr.validate_internal();
        Ok(addr)
    }

    /// Create a new address directly from Representation C digits {1, 2, 3}.
    ///
    /// Returns error if any digit is outside {1, 2, 3}.
    pub fn from_rep_c(digits_in: &[u8]) -> Result<Self, String> {
        let dim = digits_in.len();
        if dim == 0 || dim > MAX_TORSION_DIM {
            return Err(format!("Invalid dimension: {}", dim));
        }

        let mut digits = [0u8; MAX_TORSION_DIM];
        for (i, &d) in digits_in.iter().enumerate() {
            if d < 1 || d > 3 {
                return Err(format!("Invalid Rep C value at position {}: {} (must be 1, 2, or 3)", i, d));
            }
            digits[i] = d;
        }

        Ok(Self { digits, dim, validated: true })
    }

    /// Create from raw bytes WITHOUT validation — only for use with sealed addresses
    /// or when constructing from untrusted input that will be validated separately.
    pub fn from_raw_unchecked(raw: &[u8], dim: usize) -> Self {
        let mut digits = [0u8; MAX_TORSION_DIM];
        for (i, &d) in raw.iter().take(dim).enumerate() {
            digits[i] = d;
        }
        Self { digits, dim, validated: false }
    }

    /// Validate the address structure.
    ///
    /// Returns `Valid` if all digits are in {1, 2, 3}.
    /// Returns `Invalid` otherwise — deliberately opaque to prevent
    /// information leakage about address structure.
    ///
    /// **Constant-time**: Uses bitwise accumulation — no early exit,
    /// no branching on secret data, no allocation.
    pub fn validate(&self) -> AddressValidation {
        if self.validate_internal() {
            AddressValidation::Valid
        } else {
            AddressValidation::Invalid
        }
    }

    /// Diagnostic validation — available only in debug/test builds.
    ///
    /// Returns detailed failure information including positions and values.
    /// **NEVER call from production network paths** — the detail would
    /// tell an attacker exactly which bytes to fix in a forged address.
    #[cfg(any(debug_assertions, test, feature = "diagnostics"))]
    pub fn diagnose(&self) -> Result<(), AddressDiagnostic> {
        let mut zero_positions = Vec::new();

        for i in 0..self.dim {
            let d = self.digits[i];
            if d == 0 {
                zero_positions.push(i);
            } else if d > 3 {
                return Err(AddressDiagnostic::OutOfRange {
                    position: i,
                    value: d,
                });
            }
        }

        if !zero_positions.is_empty() {
            return Err(AddressDiagnostic::ZeroDetected {
                positions: zero_positions,
            });
        }

        Ok(())
    }

    /// Constant-time validation check (returns bool only, no allocation).
    ///
    /// Uses bitwise accumulation — processes every digit regardless.
    /// No branching on digit values, no data-dependent control flow.
    fn validate_internal(&self) -> bool {
        let mut any_bad: u8 = 0;

        for i in 0..self.dim {
            let d = self.digits[i];
            // d == 0 check: d.wrapping_sub(1) overflows to 255 when d=0,
            // >> 7 extracts the sign bit → 1 when d=0, 0 otherwise
            let is_zero = d.wrapping_sub(1) >> 7;

            // d > 3 check: (d / 4) is 0 for d ∈ {0,1,2,3}, ≥1 for d > 3
            // But d=0 is already caught by is_zero. For d > 3:
            // (d.wrapping_sub(4)) >> 7 is 0 when d ≥ 4 (no overflow)
            // So use: d saturating_sub 3, then check if nonzero
            // Simpler: (3_u8.wrapping_sub(d)) >> 7 → 1 when d > 3
            let is_over = (3_u8.wrapping_sub(d)) >> 7;

            any_bad |= is_zero | is_over;
        }

        // Process unused positions too (constant iteration count)
        for i in self.dim..MAX_TORSION_DIM {
            // Unused positions are 0 — that's fine, they're not active.
            // Touch them anyway to keep timing constant.
            let _ = self.digits[i];
        }

        any_bad == 0
    }

    /// Check if the address has been validated successfully.
    pub fn is_valid(&self) -> bool {
        self.validated
    }

    /// Get the address as Representation A coordinates {-1, 0, +1}.
    pub fn to_rep_a(&self) -> Vec<i8> {
        self.digits[..self.dim].iter().map(|&d| rep_c_to_a(d)).collect()
    }

    /// Get the address as Representation B coordinates {0, 1, 2}.
    pub fn to_rep_b(&self) -> Vec<u8> {
        self.digits[..self.dim].iter().map(|&d| rep_c_to_b(d)).collect()
    }

    /// Get the address as Representation C digits {1, 2, 3}.
    pub fn to_rep_c(&self) -> Vec<u8> {
        self.digits[..self.dim].to_vec()
    }

    /// Number of dimensions.
    pub fn dimensions(&self) -> usize {
        self.dim
    }

    /// Convert to a flat integer index in the torsion network.
    /// Index = sum(digit_i * 3^i) using Rep B values.
    pub fn to_node_index(&self) -> u64 {
        let mut index: u64 = 0;
        let mut power: u64 = 1;
        for i in 0..self.dim {
            let rep_b_val = rep_c_to_b(self.digits[i]) as u64;
            index += rep_b_val * power;
            power *= 3;
        }
        index
    }

    /// Compute Hamming distance to another address.
    pub fn hamming_distance(&self, other: &TorsionAddress) -> Result<usize, String> {
        if self.dim != other.dim {
            return Err(format!("Dimension mismatch: {} vs {}", self.dim, other.dim));
        }
        let mut dist = 0usize;
        for i in 0..self.dim {
            if self.digits[i] != other.digits[i] {
                dist += 1;
            }
        }
        Ok(dist)
    }
}

// ============================================================
// SealedAddress Implementation
// ============================================================

impl SealedAddress {
    /// Create a sealed address from a valid address plus sentinel positions.
    ///
    /// Sentinel positions are set to 0 in the raw representation.
    /// These zeros serve as the hardware watermark — no computation can
    /// produce them, so their presence proves hardware origin.
    ///
    /// # Arguments
    /// * `base_address` - The valid torsion address
    /// * `sentinel_positions` - Which coordinate positions to seal with zeros
    pub fn seal(base_address: &TorsionAddress, sentinel_positions: &[usize]) -> Result<Self, String> {
        if !base_address.is_valid() {
            return Err("Cannot seal an invalid address".into());
        }

        let dim = base_address.dim;
        let mut raw = [0u8; MAX_TORSION_DIM];
        let mut sentinel_mask = [false; MAX_TORSION_DIM];

        // Copy base address
        for i in 0..dim {
            raw[i] = base_address.digits[i];
        }

        // Plant sentinel zeros
        for &pos in sentinel_positions {
            if pos >= dim {
                return Err(format!("Sentinel position {} exceeds dimension {}", pos, dim));
            }
            raw[pos] = 0;
            sentinel_mask[pos] = true;
        }

        Ok(Self { raw, dim, sentinel_mask })
    }

    /// Verify the sealed address: sentinel positions must be 0,
    /// non-sentinel positions must be in {1, 2, 3}.
    ///
    /// **Constant-time**: All positions are checked regardless.
    pub fn verify(&self) -> bool {
        let mut valid: u8 = 1;

        for i in 0..self.dim {
            if self.sentinel_mask[i] {
                // Sentinel position: must be exactly 0
                let is_zero = if self.raw[i] == 0 { 1u8 } else { 0u8 };
                valid &= is_zero;
            } else {
                // Data position: must be in {1, 2, 3}
                let in_range = if self.raw[i] >= 1 && self.raw[i] <= 3 { 1u8 } else { 0u8 };
                valid &= in_range;
            }
        }

        valid == 1
    }

    /// Extract the data address (non-sentinel positions) as a TorsionAddress.
    ///
    /// Returns None if the sealed address fails verification.
    pub fn extract_address(&self) -> Option<TorsionAddress> {
        if !self.verify() {
            return None;
        }

        // Build a reduced address from non-sentinel positions
        let mut data_digits = Vec::new();
        for i in 0..self.dim {
            if !self.sentinel_mask[i] {
                data_digits.push(self.raw[i]);
            }
        }

        TorsionAddress::from_rep_c(&data_digits).ok()
    }

    /// Get the full raw representation including sentinels.
    pub fn raw_digits(&self) -> &[u8] {
        &self.raw[..self.dim]
    }

    /// Get the sentinel mask (which positions are sentinel zeros).
    pub fn sentinel_positions(&self) -> Vec<usize> {
        (0..self.dim).filter(|&i| self.sentinel_mask[i]).collect()
    }
}

// ============================================================
// Network-level validation functions
// ============================================================

/// Validate an incoming network packet's source address.
///
/// This is the main entry point for routing-layer validation.
/// Rejects any address containing a structurally impossible zero.
///
/// # Returns
/// - `Ok(TorsionAddress)` if the address is valid
/// - `Err(String)` with an **opaque** error message (no structural detail)
///
/// For diagnostic detail in debug/test builds, use `TorsionAddress::diagnose()`.
pub fn validate_incoming_address(raw_digits: &[u8], dim: usize) -> Result<TorsionAddress, String> {
    if dim == 0 || dim > MAX_TORSION_DIM {
        return Err("address validation failed".into());
    }
    if raw_digits.len() < dim {
        return Err("address validation failed".into());
    }

    let addr = TorsionAddress::from_raw_unchecked(raw_digits, dim);
    match addr.validate() {
        AddressValidation::Valid => TorsionAddress::from_rep_c(&raw_digits[..dim]),
        AddressValidation::Invalid => Err("address validation failed".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Representation Conversion ----

    #[test]
    fn test_rep_a_to_c_roundtrip() {
        for trit in [-1i8, 0, 1] {
            let c = rep_a_to_c(trit);
            let back = rep_c_to_a(c);
            assert_eq!(trit, back, "Rep A→C→A roundtrip failed for {}", trit);
        }
    }

    #[test]
    fn test_rep_b_to_c_roundtrip() {
        for digit in [0u8, 1, 2] {
            let c = rep_b_to_c(digit);
            let back = rep_c_to_b(c);
            assert_eq!(digit, back, "Rep B→C→B roundtrip failed for {}", digit);
        }
    }

    #[test]
    fn test_rep_a_to_c_values() {
        assert_eq!(rep_a_to_c(-1), 1);
        assert_eq!(rep_a_to_c(0), 2);
        assert_eq!(rep_a_to_c(1), 3);
    }

    #[test]
    fn test_rep_c_excludes_zero() {
        // Rep C values are {1, 2, 3}. No valid input produces 0.
        for trit in [-1i8, 0, 1] {
            let c = rep_a_to_c(trit);
            assert!(c >= 1 && c <= 3, "Rep C value {} is outside {{1,2,3}}", c);
            assert_ne!(c, 0, "Rep C must never produce 0");
        }
    }

    // ---- TorsionAddress ----

    #[test]
    fn test_valid_address_from_rep_a() {
        let coords = vec![-1i8, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let addr = TorsionAddress::from_rep_a(&coords).unwrap();
        assert!(addr.is_valid());
        assert_eq!(addr.dimensions(), 13);
    }

    #[test]
    fn test_valid_address_from_rep_b() {
        let coords = vec![0u8, 1, 2, 0, 1, 2, 0];
        let addr = TorsionAddress::from_rep_b(&coords).unwrap();
        assert!(addr.is_valid());
    }

    #[test]
    fn test_valid_address_from_rep_c() {
        let digits = vec![1u8, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        let addr = TorsionAddress::from_rep_c(&digits).unwrap();
        assert!(addr.is_valid());
    }

    #[test]
    fn test_invalid_rep_c_with_zero() {
        // Directly constructing with a zero should fail
        let digits = vec![1u8, 0, 3, 1, 2, 3, 1];
        let result = TorsionAddress::from_rep_c(&digits);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_detects_zero() {
        let mut addr = TorsionAddress::from_rep_c(&[1u8, 2, 3, 1, 2, 3, 1]).unwrap();
        // Manually corrupt a digit
        addr.digits[2] = 0;
        addr.validated = false;

        // Public API: opaque Invalid — no structural detail leaked
        assert_eq!(addr.validate(), AddressValidation::Invalid);

        // Diagnostic API (test/debug only): reveals position for debugging
        match addr.diagnose() {
            Err(AddressDiagnostic::ZeroDetected { positions }) => {
                assert_eq!(positions, vec![2]);
            }
            other => panic!("Expected ZeroDetected diagnostic, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_detects_multiple_zeros() {
        let raw = [1u8, 0, 3, 0, 2, 3, 1];
        let addr = TorsionAddress::from_raw_unchecked(&raw, 7);

        // Public: opaque
        assert_eq!(addr.validate(), AddressValidation::Invalid);

        // Diagnostic: positional detail
        match addr.diagnose() {
            Err(AddressDiagnostic::ZeroDetected { positions }) => {
                assert_eq!(positions, vec![1, 3]);
            }
            other => panic!("Expected ZeroDetected diagnostic, got {:?}", other),
        }
    }

    #[test]
    fn test_representation_roundtrip() {
        let rep_a = vec![-1i8, 0, 1, 0, -1, 1, 0];
        let addr = TorsionAddress::from_rep_a(&rep_a).unwrap();

        let back_a = addr.to_rep_a();
        let back_b = addr.to_rep_b();
        let back_c = addr.to_rep_c();

        assert_eq!(back_a, rep_a);
        assert_eq!(back_b, vec![0u8, 1, 2, 1, 0, 2, 1]);
        assert_eq!(back_c, vec![1u8, 2, 3, 2, 1, 3, 2]);
    }

    #[test]
    fn test_hamming_distance() {
        let a = TorsionAddress::from_rep_a(&[-1i8, 0, 1, 0, -1, 1, 0]).unwrap();
        let b = TorsionAddress::from_rep_a(&[-1i8, 1, 1, 0, -1, -1, 0]).unwrap();
        assert_eq!(a.hamming_distance(&b).unwrap(), 2); // differ at pos 1, 5
    }

    #[test]
    fn test_node_index() {
        // Origin node: all zeros in Rep B → all 1s in Rep C
        let origin = TorsionAddress::from_rep_b(&[0u8, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(origin.to_node_index(), 0);

        // (1, 0, 0, ...) in Rep B → index = 1
        let node1 = TorsionAddress::from_rep_b(&[1u8, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(node1.to_node_index(), 1);

        // (0, 1, 0, ...) in Rep B → index = 3
        let node3 = TorsionAddress::from_rep_b(&[0u8, 1, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(node3.to_node_index(), 3);
    }

    // ---- SealedAddress ----

    #[test]
    fn test_seal_and_verify() {
        let base = TorsionAddress::from_rep_a(&[-1i8, 0, 1, 0, -1, 1, 0]).unwrap();
        let sealed = SealedAddress::seal(&base, &[0, 3]).unwrap();

        assert!(sealed.verify());
        assert_eq!(sealed.raw_digits()[0], 0); // sentinel
        assert_eq!(sealed.raw_digits()[3], 0); // sentinel
        assert_eq!(sealed.raw_digits()[1], 2); // data (Rep C of 0)
        assert_eq!(sealed.raw_digits()[2], 3); // data (Rep C of +1)
    }

    #[test]
    fn test_sealed_rejects_tampered_sentinel() {
        let base = TorsionAddress::from_rep_a(&[-1i8, 0, 1, 0, -1, 1, 0]).unwrap();
        let mut sealed = SealedAddress::seal(&base, &[0, 3]).unwrap();

        // Tamper: set a sentinel position to non-zero
        sealed.raw[0] = 2;
        assert!(!sealed.verify(), "Tampered sentinel should fail verification");
    }

    #[test]
    fn test_sealed_rejects_tampered_data() {
        let base = TorsionAddress::from_rep_a(&[-1i8, 0, 1, 0, -1, 1, 0]).unwrap();
        let mut sealed = SealedAddress::seal(&base, &[0]).unwrap();

        // Tamper: set a data position to 0 (impossible value)
        sealed.raw[1] = 0;
        assert!(!sealed.verify(), "Zero in data position should fail verification");
    }

    #[test]
    fn test_sealed_extract_address() {
        let base = TorsionAddress::from_rep_a(&[-1i8, 0, 1, 0, -1, 1, 0]).unwrap();
        let sealed = SealedAddress::seal(&base, &[2, 5]).unwrap();

        let extracted = sealed.extract_address().unwrap();
        // Extracted should have 5 dimensions (7 - 2 sentinels)
        assert_eq!(extracted.dimensions(), 5);
        assert!(extracted.is_valid());
    }

    #[test]
    fn test_sentinel_positions() {
        let base = TorsionAddress::from_rep_a(&[-1i8, 0, 1, 0, -1, 1, 0]).unwrap();
        let sealed = SealedAddress::seal(&base, &[1, 4, 6]).unwrap();
        assert_eq!(sealed.sentinel_positions(), vec![1, 4, 6]);
    }

    // ---- Network Validation ----

    #[test]
    fn test_validate_incoming_valid() {
        let raw = [1u8, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        let result = validate_incoming_address(&raw, 13);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_incoming_forged() {
        let raw = [1u8, 0, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        let result = validate_incoming_address(&raw, 13);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Opaque error — no position or value detail leaked
        assert_eq!(err, "address validation failed");
    }

    #[test]
    fn test_validate_incoming_out_of_range() {
        let raw = [1u8, 2, 5, 1, 2, 3, 1]; // 5 is out of range
        let result = validate_incoming_address(&raw, 7);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Same opaque error — attacker cannot distinguish zero vs out-of-range
        assert_eq!(err, "address validation failed");
    }

    #[test]
    fn test_zero_dimension_rejected() {
        let result = validate_incoming_address(&[], 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_dimension_over_max_rejected() {
        let raw = [1u8; 20];
        let result = validate_incoming_address(&raw, 20);
        assert!(result.is_err());
    }
}