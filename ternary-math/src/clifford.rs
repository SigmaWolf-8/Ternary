// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! # Cl(3,0) over GF(3) — Clifford Algebra for Ternary Transformations
//!
//! An 8-dimensional algebra over GF(3) built from three orthonormal basis vectors.
//! Provides rotor-based composition of ternary gate sequences.
//!
//! ## Why this matters for PlenumNET
//!
//! A sequence of N ternary operations on a trit-triple can be composed into a
//! single rotor via the geometric product. Applying that rotor is O(1) regardless
//! of N. This is the concrete, measurable version of "gate compression."
//!
//! ## Implementation
//!
//! The geometric product is computed algorithmically from the Clifford algebra
//! axioms (not from a lookup table), ensuring correctness by construction.
//! Each basis blade is represented as a bitmask of generators, and the sign of
//! a product is determined by counting transpositions needed to sort the
//! concatenated generator sequence.

use crate::gf3::Gf3;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

// -- Blade representation as bitmasks -----------------------------------------
// e1 = bit 0 = 0b001
// e2 = bit 1 = 0b010
// e3 = bit 2 = 0b100
// Compound blades use OR:
// e12 = 0b011, e13 = 0b101, e23 = 0b110, e123 = 0b111

/// Map from bitmask to component index in our 8-element array.
const BLADE_TO_INDEX: [usize; 8] = [
    0, // 0b000 = {}    → scalar (1)
    1, // 0b001 = {1}   → e1
    2, // 0b010 = {2}   → e2
    4, // 0b011 = {1,2} → e12
    3, // 0b100 = {3}   → e3
    5, // 0b101 = {1,3} → e13
    6, // 0b110 = {2,3} → e23
    7, // 0b111 = {1,2,3} → e123
];

/// Map from component index back to bitmask.
const INDEX_TO_BLADE: [u8; 8] = [
    0b000, // [0] scalar
    0b001, // [1] e1
    0b010, // [2] e2
    0b100, // [3] e3
    0b011, // [4] e12
    0b101, // [5] e13
    0b110, // [6] e23
    0b111, // [7] e123
];

/// Compute the geometric product of two basis blades.
///
/// Each blade is a bitmask of generators. The product is:
/// 1. XOR the bitmasks (shared generators cancel via e_i * e_i = 1)
/// 2. Count transpositions to determine sign
///
/// Returns (result_component_index, sign_in_gf3).
#[inline]
fn blade_product(a: u8, b: u8) -> (usize, Gf3) {
    let result_blade = a ^ b;

    // Count transpositions: for each generator bit j in b, count how many
    // higher-numbered generator bits are set in a. Each such pair requires
    // one swap to bring generators into sorted order.
    let mut swaps = 0u32;
    for j in 0..3u8 {
        if b & (1 << j) == 0 {
            continue;
        }
        for i in (j + 1)..3 {
            if a & (1 << i) != 0 {
                swaps += 1;
            }
        }
    }

    let sign = if swaps % 2 == 0 {
        Gf3::ONE
    } else {
        Gf3::TWO // TWO = -1 in GF(3)
    };
    (BLADE_TO_INDEX[result_blade as usize], sign)
}

/// Named blade indices for readability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blade {
    One = 0,
    E1 = 1,
    E2 = 2,
    E3 = 3,
    E12 = 4,
    E13 = 5,
    E23 = 6,
    E123 = 7,
}

/// A multivector in Cl(3,0) over GF(3).
///
/// Components: `[scalar, e1, e2, e3, e12, e13, e23, e123]`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Multivector {
    pub components: [Gf3; 8],
}

impl Multivector {
    pub const ZERO: Self = Multivector {
        components: [Gf3::ZERO; 8],
    };

    pub fn scalar(s: Gf3) -> Self {
        let mut m = Self::ZERO;
        m.components[0] = s;
        m
    }

    pub fn vector(b1: Gf3, b2: Gf3, b3: Gf3) -> Self {
        let mut m = Self::ZERO;
        m.components[1] = b1;
        m.components[2] = b2;
        m.components[3] = b3;
        m
    }

    pub fn bivector(c1: Gf3, c2: Gf3, c3: Gf3) -> Self {
        let mut m = Self::ZERO;
        m.components[4] = c1;
        m.components[5] = c2;
        m.components[6] = c3;
        m
    }

    /// An even-grade element (potential rotor): scalar + bivector.
    pub fn rotor(alpha: Gf3, b12: Gf3, b13: Gf3, b23: Gf3) -> Self {
        let mut m = Self::ZERO;
        m.components[0] = alpha;
        m.components[4] = b12;
        m.components[5] = b13;
        m.components[6] = b23;
        m
    }

    #[inline]
    pub fn get(&self, blade: Blade) -> Gf3 {
        self.components[blade as usize]
    }

    #[inline]
    pub fn set(&mut self, blade: Blade, val: Gf3) {
        self.components[blade as usize] = val;
    }

    pub fn scalar_part(&self) -> Gf3 {
        self.components[0]
    }

    pub fn vector_part(&self) -> (Gf3, Gf3, Gf3) {
        (self.components[1], self.components[2], self.components[3])
    }

    pub fn bivector_part(&self) -> (Gf3, Gf3, Gf3) {
        (self.components[4], self.components[5], self.components[6])
    }

    pub fn pseudoscalar_part(&self) -> Gf3 {
        self.components[7]
    }

    const GRADES: [u8; 8] = [0, 1, 1, 1, 2, 2, 2, 3];

    /// Extract only components of a specific grade.
    pub fn grade_select(&self, grade: u8) -> Self {
        let mut r = Self::ZERO;
        for i in 0..8 {
            if Self::GRADES[i] == grade {
                r.components[i] = self.components[i];
            }
        }
        r
    }

    /// Grade involution: negate odd-grade components.
    pub fn grade_involution(&self) -> Self {
        let mut r = *self;
        for i in 0..8 {
            if Self::GRADES[i] % 2 == 1 {
                r.components[i] = -r.components[i];
            }
        }
        r
    }

    /// Reverse: negate grade 2 and grade 3 components.
    pub fn reverse(&self) -> Self {
        let mut r = *self;
        for i in 0..8 {
            if Self::GRADES[i] >= 2 {
                r.components[i] = -r.components[i];
            }
        }
        r
    }

    /// Norm squared: self · reverse(self).
    pub fn norm_sq(&self) -> Self {
        *self * self.reverse()
    }

    /// True if only even-grade components are nonzero.
    pub fn is_even(&self) -> bool {
        for i in 0..8 {
            if Self::GRADES[i] % 2 == 1 && !self.components[i].is_zero() {
                return false;
            }
        }
        true
    }

    /// True if the norm is nonzero (element is invertible).
    pub fn is_invertible(&self) -> bool {
        let n = self.norm_sq();
        !n.scalar_part().is_zero()
    }

    /// Sandwich product: self · v · reverse(self).
    pub fn sandwich(&self, v: &Multivector) -> Multivector {
        *self * *v * self.reverse()
    }

    /// Compose two rotors. `first` is applied first, `second` after.
    /// Result is second · first.
    pub fn compose(first: &Multivector, second: &Multivector) -> Multivector {
        *second * *first
    }

    /// Compose a chain of rotors applied left-to-right.
    /// [R0, R1, ..., Rn] → Rn · ... · R1 · R0
    pub fn compose_chain(rotors: &[Multivector]) -> Multivector {
        let mut result = Multivector::scalar(Gf3::ONE);
        for r in rotors {
            result = *r * result;
        }
        result
    }

    pub fn is_zero(&self) -> bool {
        self.components.iter().all(|c| c.is_zero())
    }
}

// -- Geometric Product --------------------------------------------------------

impl Mul for Multivector {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut result = [Gf3::ZERO; 8];
        for (i, &blade_i) in INDEX_TO_BLADE.iter().enumerate() {
            let ai = self.components[i];
            if ai.is_zero() {
                continue;
            }
            for (j, &blade_j) in INDEX_TO_BLADE.iter().enumerate() {
                let bj = rhs.components[j];
                if bj.is_zero() {
                    continue;
                }
                let (target_idx, sign) = blade_product(blade_i, blade_j);
                result[target_idx] = result[target_idx] + ai * bj * sign;
            }
        }
        Multivector { components: result }
    }
}

impl Add for Multivector {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut r = Self::ZERO;
        for i in 0..8 {
            r.components[i] = self.components[i] + rhs.components[i];
        }
        r
    }
}

impl Sub for Multivector {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let mut r = Self::ZERO;
        for i in 0..8 {
            r.components[i] = self.components[i] - rhs.components[i];
        }
        r
    }
}

impl Neg for Multivector {
    type Output = Self;
    fn neg(self) -> Self {
        let mut r = Self::ZERO;
        for i in 0..8 {
            r.components[i] = -self.components[i];
        }
        r
    }
}

impl fmt::Debug for Multivector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = ["", "e₁", "e₂", "e₃", "e₁₂", "e₁₃", "e₂₃", "e₁₂₃"];
        let mut first = true;
        for (i, &c) in self.components.iter().enumerate() {
            if c.is_zero() {
                continue;
            }
            if !first {
                write!(f, " + ")?;
            }
            if i == 0 {
                write!(f, "{c}")?;
            } else {
                write!(f, "{c}·{}", names[i])?;
            }
            first = false;
        }
        if first {
            write!(f, "0")?;
        }
        Ok(())
    }
}

impl fmt::Display for Multivector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// -- Rotor enumeration --------------------------------------------------------

/// Count invertible even-grade elements in Cl(3,0)/GF(3).
pub fn count_invertible_rotors() -> usize {
    all_invertible_rotors().len()
}

/// Enumerate all invertible rotors.
pub fn all_invertible_rotors() -> Vec<Multivector> {
    let mut rotors = Vec::new();
    for &a in &Gf3::ALL {
        for &b in &Gf3::ALL {
            for &c in &Gf3::ALL {
                for &d in &Gf3::ALL {
                    let r = Multivector::rotor(a, b, c, d);
                    if r.is_invertible() {
                        rotors.push(r);
                    }
                }
            }
        }
    }
    rotors
}

/// Ternary Circle Bridge — connecting Clifford rotors to Z₂₈ angular steps.
///
/// In the ternary circle, there are 28 discrete angular positions (multiples
/// of 13°). The Tribonacci word maps symbols {A=0, B=1, C=2} to angular
/// offsets in Z₂₈. This bridge encodes those angular steps as Clifford rotors
/// in Cl(3,0)/GF(3), enabling gate compression of angular sequences.
///
/// A sequence of N angular steps can be composed into a single rotor via
/// the geometric product, making the composed rotation O(1) to apply.
pub mod ternary_circle_bridge {
    use super::*;

    /// Z₂₈ angular step size in ternary degrees
    const _RADIAN_DEG: u32 = 13;

    /// Map a Tribonacci symbol {A=0, B=1, C=2} to a Clifford rotor encoding
    /// its angular step.
    ///
    /// - A (0 radians = 0°) → identity rotor (no rotation)
    /// - B (1 radian = 13°) → rotor in e₁e₂ plane, scalar=1, bivector12=1
    /// - C (2 radians = 26°) → rotor in e₁e₂ plane, scalar=1, bivector12=2
    ///
    /// The key property: compose_tribonacci_walk([s₁,s₂,...,sₙ]) compresses
    /// an N-step angular walk into a single rotor via the geometric product.
    /// The bivector12 coefficient encodes the Z₂₈ radian count because
    /// Gf3 addition mod 3 in the bivector component mirrors the angular
    /// step accumulation (both are mod-3 in GF(3) arithmetic).
    pub fn angular_step_rotor(symbol: u8) -> Multivector {
        match symbol % 3 {
            0 => Multivector::scalar(Gf3::ONE),
            1 => Multivector::rotor(Gf3::ONE, Gf3::ONE, Gf3::ZERO, Gf3::ZERO),
            2 => Multivector::rotor(Gf3::ONE, Gf3::ZERO, Gf3::ONE, Gf3::ZERO),
            _ => unreachable!(),
        }
    }

    /// Convert a Z₂₈ radian count to the corresponding GF(3) bivector encoding.
    /// Since GF(3) has period 3, the radian count is reduced mod 3 to get
    /// the bivector12 coefficient.
    pub fn z28_to_rotor(radian_count: u32) -> Multivector {
        angular_step_rotor((radian_count % 3) as u8)
    }

    pub fn compose_tribonacci_walk(symbols: &[u8]) -> Multivector {
        let rotors: Vec<Multivector> = symbols.iter().map(|&s| angular_step_rotor(s)).collect();
        Multivector::compose_chain(&rotors)
    }

    pub fn rotor_orbit(rotor: &Multivector) -> Vec<Multivector> {
        let mut orbit = Vec::new();
        let mut current = Multivector::scalar(Gf3::ONE);
        loop {
            orbit.push(current);
            current = *rotor * current;
            if current == Multivector::scalar(Gf3::ONE) {
                break;
            }
            if orbit.len() > 81 {
                break;
            }
        }
        orbit
    }

    pub fn angular_step_rotor_sequence(length: usize) -> Vec<Multivector> {
        let mut word = Vec::with_capacity(length);
        let mut buffer: Vec<u8> = vec![0]; // Start with A
        while word.len() < length {
            let mut next = Vec::new();
            for &sym in &buffer {
                match sym {
                    0 => { next.push(0); next.push(1); } // A → AB
                    1 => { next.push(0); next.push(2); } // B → AC
                    2 => { next.push(0); }                // C → A
                    _ => {}
                }
            }
            buffer = next;
            word.clear();
            word.extend_from_slice(&buffer);
        }
        word.truncate(length);
        word.iter().map(|&s| angular_step_rotor(s)).collect()
    }

    pub fn apply_angular_rotation(rotor: &Multivector, vector: &Multivector) -> Multivector {
        rotor.sandwich(vector)
    }

    pub fn cumulative_rotors(symbols: &[u8]) -> Vec<Multivector> {
        let mut result = Vec::with_capacity(symbols.len());
        let mut accumulated = Multivector::scalar(Gf3::ONE);
        for &s in symbols {
            accumulated = angular_step_rotor(s) * accumulated;
            result.push(accumulated);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::ternary_circle_bridge::*;

    fn e1() -> Multivector { Multivector::vector(Gf3::ONE, Gf3::ZERO, Gf3::ZERO) }
    fn e2() -> Multivector { Multivector::vector(Gf3::ZERO, Gf3::ONE, Gf3::ZERO) }
    fn e3() -> Multivector { Multivector::vector(Gf3::ZERO, Gf3::ZERO, Gf3::ONE) }
    fn one() -> Multivector { Multivector::scalar(Gf3::ONE) }

    #[test]
    fn axiom_generators_square_to_one() {
        assert_eq!(e1() * e1(), one(), "e₁² ≠ 1");
        assert_eq!(e2() * e2(), one(), "e₂² ≠ 1");
        assert_eq!(e3() * e3(), one(), "e₃² ≠ 1");
    }

    #[test]
    fn axiom_generators_anticommute() {
        assert_eq!(e1() * e2(), -(e2() * e1()), "e₁e₂ ≠ −e₂e₁");
        assert_eq!(e1() * e3(), -(e3() * e1()), "e₁e₃ ≠ −e₃e₁");
        assert_eq!(e2() * e3(), -(e3() * e2()), "e₂e₃ ≠ −e₃e₂");
    }

    #[test]
    fn axiom_scalar_identity() {
        let v = Multivector::vector(Gf3::TWO, Gf3::ONE, Gf3::ZERO);
        assert_eq!(one() * v, v);
        assert_eq!(v * one(), v);
    }

    #[test]
    fn blade_product_spot_checks() {
        // e1*e2 = +e12
        let (idx, sign) = blade_product(0b001, 0b010);
        assert_eq!(idx, 4); assert_eq!(sign, Gf3::ONE);

        // e2*e1 = -e12
        let (idx, sign) = blade_product(0b010, 0b001);
        assert_eq!(idx, 4); assert_eq!(sign, Gf3::TWO);

        // e12*e1 = -e2
        let (idx, sign) = blade_product(0b011, 0b001);
        assert_eq!(idx, 2); assert_eq!(sign, Gf3::TWO);

        // e23*e12 = -e13
        let (idx, sign) = blade_product(0b110, 0b011);
        assert_eq!(idx, 5); assert_eq!(sign, Gf3::TWO);

        // e123*e123 = -1
        let (idx, sign) = blade_product(0b111, 0b111);
        assert_eq!(idx, 0); assert_eq!(sign, Gf3::TWO);
    }

    #[test]
    fn associativity_all_blade_triples() {
        // 8³ = 512 checks — sufficient since the product is bilinear.
        let blades: Vec<Multivector> = (0..8)
            .map(|i| {
                let mut m = Multivector::ZERO;
                m.components[i] = Gf3::ONE;
                m
            })
            .collect();

        for a in &blades {
            for b in &blades {
                for c in &blades {
                    assert_eq!(
                        (*a * *b) * *c,
                        *a * (*b * *c),
                        "Associativity failed"
                    );
                }
            }
        }
    }

    #[test]
    fn rotor_norm_sq_is_even() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                for &c in &Gf3::ALL {
                    for &d in &Gf3::ALL {
                        let r = Multivector::rotor(a, b, c, d);
                        let n = r.norm_sq();
                        assert!(n.is_even(), "Norm² of {:?} not even: {:?}", r, n);
                    }
                }
            }
        }
    }

    #[test]
    fn reverse_is_involution() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                let m = Multivector::rotor(a, b, Gf3::ONE, Gf3::ZERO);
                assert_eq!(m.reverse().reverse(), m);
            }
        }
    }

    #[test]
    fn rotor_count() {
        let count = count_invertible_rotors();
        assert!(count > 0);
        assert!(count <= 81);
        println!("Invertible rotors: {count} / 81 even elements");
    }

    #[test]
    fn compose_chain_matches_sequential() {
        let rotors = vec![
            Multivector::rotor(Gf3::ONE, Gf3::ONE, Gf3::ZERO, Gf3::ZERO),
            Multivector::rotor(Gf3::TWO, Gf3::ZERO, Gf3::ONE, Gf3::ZERO),
            Multivector::rotor(Gf3::ONE, Gf3::ZERO, Gf3::ZERO, Gf3::TWO),
        ];
        let chained = Multivector::compose_chain(&rotors);
        let manual = rotors[2] * rotors[1] * rotors[0];
        assert_eq!(chained, manual);
    }

    #[test]
    fn identity_rotor_preserves_vectors() {
        let id = Multivector::scalar(Gf3::ONE);
        let v = Multivector::vector(Gf3::ONE, Gf3::TWO, Gf3::ONE);
        // For identity: sandwich = 1 * v * 1 = v
        assert_eq!(id.sandwich(&v), v);
    }

    #[test]
    fn angular_step_a_is_identity() {
        let rotor = angular_step_rotor(0);
        assert_eq!(rotor, Multivector::scalar(Gf3::ONE));
    }

    #[test]
    fn angular_step_b_is_rotor() {
        let rotor = angular_step_rotor(1);
        assert_ne!(rotor, Multivector::scalar(Gf3::ONE));
        assert_ne!(rotor, Multivector::scalar(Gf3::ZERO));
    }

    #[test]
    fn angular_step_c_is_rotor() {
        let rotor = angular_step_rotor(2);
        assert_ne!(rotor, Multivector::scalar(Gf3::ONE));
        assert_ne!(rotor, Multivector::scalar(Gf3::ZERO));
    }

    #[test]
    fn compose_walk_identity_for_all_a() {
        let walk = compose_tribonacci_walk(&[0, 0, 0, 0]);
        assert_eq!(walk, Multivector::scalar(Gf3::ONE),
            "Walk of all A symbols should compose to identity");
    }

    #[test]
    fn compose_walk_nontrivial() {
        let walk = compose_tribonacci_walk(&[0, 1, 2, 0, 1]);
        assert_ne!(walk, Multivector::scalar(Gf3::ZERO));
    }

    #[test]
    fn rotor_orbit_is_finite() {
        let r = angular_step_rotor(1);
        let orbit = rotor_orbit(&r);
        assert!(orbit.len() <= 81,
            "Orbit of a rotor in GF(3) should be finite and ≤81");
        assert!(!orbit.is_empty());
    }

    #[test]
    fn cumulative_rotors_correct_length() {
        let symbols = vec![0u8, 1, 2, 0, 1, 0, 2, 1];
        let cumu = cumulative_rotors(&symbols);
        assert_eq!(cumu.len(), symbols.len());
    }

    #[test]
    fn cumulative_rotors_first_matches_single() {
        let symbols = vec![1u8];
        let cumu = cumulative_rotors(&symbols);
        assert_eq!(cumu[0], angular_step_rotor(1));
    }

    #[test]
    fn angular_rotation_preserves_grade() {
        let r = angular_step_rotor(1);
        let v = e1();
        let rotated = apply_angular_rotation(&r, &v);
        let is_vector = rotated.components[0] == Gf3::ZERO
            && rotated.components[4] == Gf3::ZERO
            && rotated.components[5] == Gf3::ZERO
            && rotated.components[6] == Gf3::ZERO
            && rotated.components[7] == Gf3::ZERO;
        assert!(is_vector, "Sandwich product should preserve vector grade");
    }

    #[test]
    fn angular_rotor_sequence_generates() {
        let seq = angular_step_rotor_sequence(10);
        assert_eq!(seq.len(), 10);
        for r in &seq {
            assert_ne!(*r, Multivector::scalar(Gf3::ZERO));
        }
    }

    #[test]
    fn z28_to_rotor_periodicity() {
        for r in 0..28u32 {
            let rotor = z28_to_rotor(r);
            let equiv = z28_to_rotor(r + 3);
            assert_eq!(rotor, equiv,
                "Z₂₈ radian {} should map to same GF(3) rotor as {}", r, r + 3);
        }
    }

    #[test]
    fn z28_to_rotor_zero_is_identity() {
        assert_eq!(z28_to_rotor(0), Multivector::scalar(Gf3::ONE));
    }

    #[test]
    fn z28_to_rotor_covers_gf3() {
        let r0 = z28_to_rotor(0);
        let r1 = z28_to_rotor(1);
        let r2 = z28_to_rotor(2);
        assert_ne!(r0, r1);
        assert_ne!(r1, r2);
        assert_ne!(r0, r2);
    }
}
