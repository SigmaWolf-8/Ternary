//! Ternary Computing Module
//!
//! Implements the Unified Ternary Logic System from the Salvi Framework whitepaper.
//! All operations are performed in GF(3) (Galois Field of 3 elements).
//!
//! # Representations
//! - **A (Computational)**: {-1, 0, +1} - For arithmetic operations
//! - **B (Network)**: {0, 1, 2} - For network transmission
//! - **C (Human)**: {1, 2, 3} - For human-readable display
//!
//! # Bijections
//! - A→B: f(a) = a + 1
//! - A→C: f(a) = a + 2
//! - B→C: f(b) = b + 1

use alloc::vec::Vec;

/// A single trit (ternary digit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trit {
    /// Internal representation uses Representation A: {-1, 0, +1}
    value: i8,
}

impl Trit {
    /// Create from Representation A value (-1, 0, or +1)
    pub fn from_a(value: i8) -> Option<Self> {
        match value {
            -1 | 0 | 1 => Some(Self { value }),
            _ => None,
        }
    }

    /// Create from Representation B value (0, 1, or 2)
    pub fn from_b(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self { value: -1 }),
            1 => Some(Self { value: 0 }),
            2 => Some(Self { value: 1 }),
            _ => None,
        }
    }

    /// Create from Representation C value (1, 2, or 3)
    pub fn from_c(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self { value: -1 }),
            2 => Some(Self { value: 0 }),
            3 => Some(Self { value: 1 }),
            _ => None,
        }
    }

    /// Get Representation A value
    pub fn to_a(&self) -> i8 {
        self.value
    }

    /// Get Representation B value
    pub fn to_b(&self) -> u8 {
        (self.value + 1) as u8
    }

    /// Get Representation C value
    pub fn to_c(&self) -> u8 {
        (self.value + 2) as u8
    }

    /// Ternary negation (NOT): -x in GF(3)
    pub fn not(&self) -> Self {
        Self { value: -self.value }
    }

    /// Ternary addition in GF(3): (a + b) mod 3
    pub fn add(&self, other: &Trit) -> Self {
        let sum = (self.value + other.value).rem_euclid(3);
        let normalized = if sum == 2 { -1 } else { sum as i8 };
        Self { value: normalized }
    }

    /// Ternary multiplication in GF(3): (a × b) mod 3
    pub fn multiply(&self, other: &Trit) -> Self {
        let product = (self.value * other.value).rem_euclid(3);
        let normalized = if product == 2 { -1 } else { product as i8 };
        Self { value: normalized }
    }

    /// Ternary XOR (min operation)
    pub fn xor(&self, other: &Trit) -> Self {
        Self {
            value: core::cmp::min(self.value, other.value),
        }
    }

    /// Bijective rotation: cycles through -1 → 0 → 1 → -1
    pub fn rotate(&self) -> Self {
        let rotated = match self.value {
            -1 => 0,
            0 => 1,
            1 => -1,
            _ => 0,
        };
        Self { value: rotated }
    }

    /// Inverse rotation: cycles through 1 → 0 → -1 → 1
    pub fn rotate_inverse(&self) -> Self {
        let rotated = match self.value {
            1 => 0,
            0 => -1,
            -1 => 1,
            _ => 0,
        };
        Self { value: rotated }
    }
}

/// A tryte (6 trits = 729 values, equivalent to ~9.5 bits)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tryte {
    trits: [Trit; 6],
}

impl Tryte {
    /// Create a new tryte from 6 trits
    pub fn new(trits: [Trit; 6]) -> Self {
        Self { trits }
    }

    /// Create from a decimal value (0-728)
    pub fn from_decimal(mut value: u16) -> Option<Self> {
        if value >= 729 {
            return None;
        }

        let mut trits = [Trit { value: 0 }; 6];
        for i in 0..6 {
            let remainder = (value % 3) as i8;
            let trit_value = match remainder {
                0 => -1,
                1 => 0,
                2 => 1,
                _ => 0,
            };
            trits[i] = Trit { value: trit_value };
            value /= 3;
        }
        Some(Self { trits })
    }

    /// Convert to decimal value
    pub fn to_decimal(&self) -> u16 {
        let mut result = 0u16;
        let mut multiplier = 1u16;
        for trit in &self.trits {
            let digit = (trit.value + 1) as u16;
            result += digit * multiplier;
            multiplier *= 3;
        }
        result
    }

    /// Get trits
    pub fn trits(&self) -> &[Trit; 6] {
        &self.trits
    }

    /// Tryte-wise NOT
    pub fn not(&self) -> Self {
        let mut result = self.trits;
        for trit in &mut result {
            *trit = trit.not();
        }
        Self { trits: result }
    }

    /// Tryte-wise ADD
    pub fn add(&self, other: &Tryte) -> Self {
        let mut result = [Trit { value: 0 }; 6];
        for i in 0..6 {
            result[i] = self.trits[i].add(&other.trits[i]);
        }
        Self { trits: result }
    }
}

/// Ternary word (27 trits = 3 trytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TernaryWord {
    trytes: [Tryte; 3],
}

impl TernaryWord {
    pub fn new(trytes: [Tryte; 3]) -> Self {
        Self { trytes }
    }

    pub fn trytes(&self) -> &[Tryte; 3] {
        &self.trytes
    }
}

/// Convert between representations
pub fn convert_representation(value: i8, from: Representation, to: Representation) -> i8 {
    // First convert to A
    let a_value = match from {
        Representation::A => value,
        Representation::B => value - 1,
        Representation::C => value - 2,
    };

    // Then convert from A to target
    match to {
        Representation::A => a_value,
        Representation::B => a_value + 1,
        Representation::C => a_value + 2,
    }
}

/// Representation enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    /// Computational: {-1, 0, +1}
    A,
    /// Network: {0, 1, 2}
    B,
    /// Human: {1, 2, 3}
    C,
}

/// Calculate information density for ternary vs binary
pub fn information_density(trit_count: u32) -> DensityComparison {
    let ternary_states = 3u128.pow(trit_count);
    let equivalent_bits = (trit_count as f64) * 1.585; // log2(3) ≈ 1.585
    let bit_count = equivalent_bits.ceil() as u32;
    let binary_states = 2u128.pow(bit_count);

    DensityComparison {
        trit_count,
        ternary_states,
        equivalent_bits,
        bit_count,
        binary_states,
        efficiency_gain: (ternary_states as f64) / (binary_states as f64),
    }
}

#[derive(Debug, Clone)]
pub struct DensityComparison {
    pub trit_count: u32,
    pub ternary_states: u128,
    pub equivalent_bits: f64,
    pub bit_count: u32,
    pub binary_states: u128,
    pub efficiency_gain: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trit_representations() {
        let trit = Trit::from_a(-1).unwrap();
        assert_eq!(trit.to_a(), -1);
        assert_eq!(trit.to_b(), 0);
        assert_eq!(trit.to_c(), 1);

        let trit = Trit::from_b(2).unwrap();
        assert_eq!(trit.to_a(), 1);
    }

    #[test]
    fn test_ternary_addition() {
        let a = Trit::from_a(1).unwrap();
        let b = Trit::from_a(1).unwrap();
        let result = a.add(&b);
        assert_eq!(result.to_a(), -1); // 1 + 1 = 2 ≡ -1 (mod 3)
    }

    #[test]
    fn test_ternary_multiplication() {
        let a = Trit::from_a(-1).unwrap();
        let b = Trit::from_a(-1).unwrap();
        let result = a.multiply(&b);
        assert_eq!(result.to_a(), 1); // -1 × -1 = 1
    }

    #[test]
    fn test_tryte_conversion() {
        let tryte = Tryte::from_decimal(365).unwrap();
        assert_eq!(tryte.to_decimal(), 365);
    }
}
