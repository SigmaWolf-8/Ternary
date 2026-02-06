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
    fn test_trit_from_a_valid() {
        assert!(Trit::from_a(-1).is_some());
        assert!(Trit::from_a(0).is_some());
        assert!(Trit::from_a(1).is_some());
    }

    #[test]
    fn test_trit_from_a_invalid() {
        assert!(Trit::from_a(-2).is_none());
        assert!(Trit::from_a(2).is_none());
        assert!(Trit::from_a(127).is_none());
    }

    #[test]
    fn test_trit_from_b_valid() {
        assert!(Trit::from_b(0).is_some());
        assert!(Trit::from_b(1).is_some());
        assert!(Trit::from_b(2).is_some());
    }

    #[test]
    fn test_trit_from_b_invalid() {
        assert!(Trit::from_b(3).is_none());
        assert!(Trit::from_b(255).is_none());
    }

    #[test]
    fn test_trit_from_c_valid() {
        assert!(Trit::from_c(1).is_some());
        assert!(Trit::from_c(2).is_some());
        assert!(Trit::from_c(3).is_some());
    }

    #[test]
    fn test_trit_from_c_invalid() {
        assert!(Trit::from_c(0).is_none());
        assert!(Trit::from_c(4).is_none());
    }

    #[test]
    fn test_trit_representations_roundtrip() {
        let trit = Trit::from_a(-1).unwrap();
        assert_eq!(trit.to_a(), -1);
        assert_eq!(trit.to_b(), 0);
        assert_eq!(trit.to_c(), 1);

        let trit = Trit::from_a(0).unwrap();
        assert_eq!(trit.to_a(), 0);
        assert_eq!(trit.to_b(), 1);
        assert_eq!(trit.to_c(), 2);

        let trit = Trit::from_a(1).unwrap();
        assert_eq!(trit.to_a(), 1);
        assert_eq!(trit.to_b(), 2);
        assert_eq!(trit.to_c(), 3);
    }

    #[test]
    fn test_bijection_a_to_b() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.to_b() as i8, a_val + 1);
        }
    }

    #[test]
    fn test_bijection_a_to_c() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.to_c() as i8, a_val + 2);
        }
    }

    #[test]
    fn test_bijection_b_to_c() {
        for b_val in [0u8, 1, 2] {
            let trit = Trit::from_b(b_val).unwrap();
            assert_eq!(trit.to_c(), b_val + 1);
        }
    }

    #[test]
    fn test_bijection_roundtrip_b_a_b() {
        for b_val in [0u8, 1, 2] {
            let trit = Trit::from_b(b_val).unwrap();
            let a_val = trit.to_a();
            let reconstructed = Trit::from_a(a_val).unwrap();
            assert_eq!(reconstructed.to_b(), b_val);
        }
    }

    #[test]
    fn test_ternary_not() {
        assert_eq!(Trit::from_a(-1).unwrap().not().to_a(), 1);
        assert_eq!(Trit::from_a(0).unwrap().not().to_a(), 0);
        assert_eq!(Trit::from_a(1).unwrap().not().to_a(), -1);
    }

    #[test]
    fn test_ternary_not_involution() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.not().not().to_a(), a_val);
        }
    }

    #[test]
    fn test_ternary_addition_full_table() {
        let vals = [-1i8, 0, 1];
        for &a in &vals {
            for &b in &vals {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                let result = ta.add(&tb);
                let expected = (a + b).rem_euclid(3);
                let expected_norm = if expected == 2 { -1 } else { expected as i8 };
                assert_eq!(result.to_a(), expected_norm, "GF(3) add: {} + {} = {}", a, b, expected_norm);
            }
        }
    }

    #[test]
    fn test_ternary_multiplication_full_table() {
        let vals = [-1i8, 0, 1];
        for &a in &vals {
            for &b in &vals {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                let result = ta.multiply(&tb);
                let expected = (a * b).rem_euclid(3);
                let expected_norm = if expected == 2 { -1 } else { expected as i8 };
                assert_eq!(result.to_a(), expected_norm, "GF(3) mul: {} * {} = {}", a, b, expected_norm);
            }
        }
    }

    #[test]
    fn test_gf3_additive_identity() {
        let zero = Trit::from_a(0).unwrap();
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.add(&zero).to_a(), a_val);
        }
    }

    #[test]
    fn test_gf3_multiplicative_identity() {
        let one = Trit::from_a(1).unwrap();
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.multiply(&one).to_a(), a_val);
        }
    }

    #[test]
    fn test_gf3_multiplicative_absorbing() {
        let zero = Trit::from_a(0).unwrap();
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.multiply(&zero).to_a(), 0);
        }
    }

    #[test]
    fn test_rotation_cycle() {
        let trit = Trit::from_a(-1).unwrap();
        let r1 = trit.rotate();
        assert_eq!(r1.to_a(), 0);
        let r2 = r1.rotate();
        assert_eq!(r2.to_a(), 1);
        let r3 = r2.rotate();
        assert_eq!(r3.to_a(), -1);
    }

    #[test]
    fn test_rotation_inverse_cycle() {
        let trit = Trit::from_a(1).unwrap();
        let r1 = trit.rotate_inverse();
        assert_eq!(r1.to_a(), 0);
        let r2 = r1.rotate_inverse();
        assert_eq!(r2.to_a(), -1);
        let r3 = r2.rotate_inverse();
        assert_eq!(r3.to_a(), 1);
    }

    #[test]
    fn test_rotate_inverse_cancels_rotate() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.rotate().rotate_inverse().to_a(), a_val);
            assert_eq!(trit.rotate_inverse().rotate().to_a(), a_val);
        }
    }

    #[test]
    fn test_xor_commutativity() {
        let vals = [-1i8, 0, 1];
        for &a in &vals {
            for &b in &vals {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                assert_eq!(ta.xor(&tb).to_a(), tb.xor(&ta).to_a());
            }
        }
    }

    #[test]
    fn test_tryte_creation() {
        let trits = [
            Trit::from_a(0).unwrap(),
            Trit::from_a(1).unwrap(),
            Trit::from_a(-1).unwrap(),
            Trit::from_a(0).unwrap(),
            Trit::from_a(1).unwrap(),
            Trit::from_a(-1).unwrap(),
        ];
        let tryte = Tryte::new(trits);
        assert_eq!(tryte.trits().len(), 6);
    }

    #[test]
    fn test_tryte_decimal_roundtrip() {
        for val in [0u16, 1, 100, 364, 365, 500, 728] {
            let tryte = Tryte::from_decimal(val).unwrap();
            assert_eq!(tryte.to_decimal(), val, "Roundtrip failed for decimal {}", val);
        }
    }

    #[test]
    fn test_tryte_decimal_bounds() {
        assert!(Tryte::from_decimal(0).is_some());
        assert!(Tryte::from_decimal(728).is_some());
        assert!(Tryte::from_decimal(729).is_none());
        assert!(Tryte::from_decimal(1000).is_none());
    }

    #[test]
    fn test_tryte_not_involution() {
        for val in [0u16, 100, 365, 728] {
            let tryte = Tryte::from_decimal(val).unwrap();
            assert_eq!(tryte.not().not().to_decimal(), val);
        }
    }

    #[test]
    fn test_tryte_add_identity() {
        let zero = Tryte::from_decimal(364).unwrap(); // all-zeros
        for val in [0u16, 100, 365, 728] {
            let tryte = Tryte::from_decimal(val).unwrap();
            let result = tryte.add(&zero);
            assert_eq!(result.trits().len(), 6);
        }
    }

    #[test]
    fn test_ternary_word_creation() {
        let t0 = Tryte::from_decimal(0).unwrap();
        let t1 = Tryte::from_decimal(100).unwrap();
        let t2 = Tryte::from_decimal(728).unwrap();
        let word = TernaryWord::new([t0, t1, t2]);
        assert_eq!(word.trytes().len(), 3);
    }

    #[test]
    fn test_convert_representation_a_to_b() {
        assert_eq!(convert_representation(-1, Representation::A, Representation::B), 0);
        assert_eq!(convert_representation(0, Representation::A, Representation::B), 1);
        assert_eq!(convert_representation(1, Representation::A, Representation::B), 2);
    }

    #[test]
    fn test_convert_representation_a_to_c() {
        assert_eq!(convert_representation(-1, Representation::A, Representation::C), 1);
        assert_eq!(convert_representation(0, Representation::A, Representation::C), 2);
        assert_eq!(convert_representation(1, Representation::A, Representation::C), 3);
    }

    #[test]
    fn test_convert_representation_b_to_c() {
        assert_eq!(convert_representation(0, Representation::B, Representation::C), 1);
        assert_eq!(convert_representation(1, Representation::B, Representation::C), 2);
        assert_eq!(convert_representation(2, Representation::B, Representation::C), 3);
    }

    #[test]
    fn test_convert_identity() {
        for repr in [Representation::A, Representation::B, Representation::C] {
            assert_eq!(convert_representation(0, repr, repr), 0);
        }
    }

    #[test]
    fn test_information_density() {
        let density = information_density(6);
        assert_eq!(density.trit_count, 6);
        assert_eq!(density.ternary_states, 729);
        assert!(density.efficiency_gain > 0.0);
        assert!(density.equivalent_bits > 9.0);
    }

    #[test]
    fn test_information_density_single_trit() {
        let density = information_density(1);
        assert_eq!(density.ternary_states, 3);
        assert_eq!(density.bit_count, 2);
        assert_eq!(density.binary_states, 4);
    }

    #[test]
    fn test_information_density_59_percent_gain() {
        let density = information_density(6);
        let ternary_per_unit = 729.0 / 6.0;
        let binary_per_unit = 1024.0 / 10.0;
        let gain = (ternary_per_unit / binary_per_unit - 1.0) * 100.0;
        assert!(gain > 15.0, "Ternary should have >15% information density advantage per digit: {:.1}%", gain);
    }
}
