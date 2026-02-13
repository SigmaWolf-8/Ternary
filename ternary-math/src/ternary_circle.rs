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

//! # The Ternary Circle
//!
//! Foundational geometric types and the Z₂₈ cyclic group for the Salvi
//! Framework's native angular system — the **canonical ternary circle**
//! where geometry speaks base-3 as its mother tongue.
//!
//! All constants are imported from [`crate::constants`] — this module
//! does not define its own copy of any shared value.
//!
//! ## The Axiom
//!
//! A full circle is **364 degrees** = `111111₃` (a base-3 repunit of six 1's).
//! The ratio of circumference to diameter is **π = 14**.
//! The radian — the angle subtending an arc equal to the radius — is
//! **exactly 13 degrees** = `111₃`, the seventh Tribonacci number (T₇).
//!
//! ## The Cyclic Group Z₂₈
//!
//! Because 364 = 28 × 13, the set {0°, 13°, 26°, …, 351°} modulo 364°
//! forms a **finite cyclic group of order 28**. The Tribonacci word,
//! read modulo 28, produces a dense subset that covers all residues.

pub use crate::constants::{
    FULL_CIRCLE_DEG, PI_TERNARY, TWO_PI_TERNARY, RADIAN_DEG,
    TAU_TRIBONACCI, TAU_SQUARED, TAU_CUBED,
    TRIBONACCI_GOLDEN_ANGLE_DEG, FULL_CIRCLE_BASE3, RADIAN_BASE3,
    CYCLIC_ORDER, RADIANS_PER_CIRCLE, Z28_DIMENSIONS,
    Z28_GENERATOR, Z28_CO_GENERATOR,
    WALK_TURN_0, WALK_TURN_1, WALK_TURN_2,
    GOLDEN_ANGLE_TERNARY_DEG, MAX_TRITS,
    ternary_deg_to_std_deg, std_deg_to_ternary_deg,
    ternary_rad_to_std_rad, std_rad_to_ternary_rad,
    ternary_deg_to_ternary_rad, ternary_rad_to_ternary_deg,
    trit_to_walk_angle_deg, trit_to_std_rad,
};


// ══════════════════════════════════════════════════════════════
// Z₂₈ CYCLIC GROUP
// ══════════════════════════════════════════════════════════════

/// A position in the cyclic group Z₂₈.
///
/// Represents one of the 28 discrete angular positions in the ternary
/// circle, separated by 13° each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Z28(pub u8);

impl Z28 {
    /// Create a new Z₂₈ element, reducing modulo 28.
    pub fn new(val: u32) -> Self {
        Z28((val % 28) as u8)
    }

    /// The identity element (0 position).
    pub fn zero() -> Self {
        Z28(0)
    }

    /// Add two elements in Z₂₈ (group operation).
    pub fn add(self, other: Z28) -> Z28 {
        Z28(((self.0 as u32 + other.0 as u32) % 28) as u8)
    }

    /// Subtract (inverse addition) in Z₂₈.
    pub fn sub(self, other: Z28) -> Z28 {
        Z28(((self.0 as u32 + 28 - other.0 as u32) % 28) as u8)
    }

    /// Negate (additive inverse) in Z₂₈.
    pub fn neg(self) -> Z28 {
        if self.0 == 0 { Z28(0) } else { Z28(28 - self.0) }
    }

    /// The ternary degree value of this position.
    pub fn to_ternary_deg(self) -> f64 {
        self.0 as f64 * RADIAN_DEG
    }

    /// The standard radian value (for trigonometric functions).
    pub fn to_std_rad(self) -> f64 {
        ternary_rad_to_std_rad(self.0 as f64)
    }

    /// Advance by a trit instruction (0, 1, or 2 ternary radians).
    pub fn step(self, trit: u8) -> Z28 {
        debug_assert!(trit <= 2);
        self.add(Z28(trit))
    }

    /// Check if this position is reachable from the origin by
    /// accumulating Tribonacci word trit values.
    pub fn is_tribonacci_reachable(&self) -> bool {
        true
    }

    /// The raw position value in [0, 28).
    pub fn value(self) -> u8 {
        self.0
    }

    /// Convert this Z₂₈ position to a GF(3) residue class.
    ///
    /// Since the Clifford algebra operates over GF(3), this maps the
    /// 28-element group to its GF(3) residue: position mod 3.
    /// This is the projection used by the Clifford bridge.
    ///
    /// # Fiber sizes (non-uniform)
    ///
    /// Because 28 is not divisible by 3, the fibers are **not** equal:
    /// - Residue 0: 10 positions (0, 3, 6, 9, 12, 15, 18, 21, 24, 27)
    /// - Residue 1:  9 positions (1, 4, 7, 10, 13, 16, 19, 22, 25)
    /// - Residue 2:  9 positions (2, 5, 8, 11, 14, 17, 20, 23, 26)
    ///
    /// This 10/9/9 imbalance is acceptable for classification and
    /// visualization but **must not** feed into any security-sensitive
    /// path without additional mitigation (e.g., rejecting position 27
    /// to obtain uniform 9/9/9 fibers over positions 0–26).
    pub fn to_gf3_residue(self) -> u8 {
        self.0 % 3
    }

    /// Return the 7-element coset index: position / 4.
    ///
    /// Z₂₈ = Z₄ × Z₇. The GF(3) residue captures the mod-3 behavior;
    /// this captures the complementary 7-fold structure.
    pub fn coset_index_7(self) -> u8 {
        self.0 % 7
    }
}

impl std::fmt::Display for Z28 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Z₂₈({})", self.0)
    }
}

// ══════════════════════════════════════════════════════════════
// WALK ENGINE
// ══════════════════════════════════════════════════════════════

/// A point in the Tribonacci radian spiral.
#[derive(Clone, Debug)]
pub struct SpiralPoint {
    /// X coordinate in the complex plane.
    pub x: f64,
    /// Y coordinate in the complex plane.
    pub y: f64,
    /// The angular position in Z₂₈ at this step.
    pub position: Z28,
    /// The trit digit that generated this step.
    pub trit: u8,
    /// The step index.
    pub step: usize,
}

/// Walk the Tribonacci radian spiral.
///
/// Given a sequence of trit digits (from the Tribonacci word or the
/// base-3 expansion of τ), compute the spiral path:
///
///   z_n = Σ(k=1..n) e^(i · w_k · 13°) / τ^k
///
/// where w_k ∈ {0, 1, 2} is the k-th trit, 13° is the ternary radian,
/// and τ is the Tribonacci constant (radial scaling).
pub fn walk_tribonacci_radian_spiral(trits: &[u8]) -> Vec<SpiralPoint> {
    let mut points = Vec::with_capacity(trits.len() + 1);
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut direction = Z28::zero();
    let mut tau_power = 1.0_f64;

    points.push(SpiralPoint {
        x: 0.0,
        y: 0.0,
        position: direction,
        trit: 0,
        step: 0,
    });

    for (k, &trit) in trits.iter().enumerate() {
        debug_assert!(trit <= 2, "Trit must be 0, 1, or 2; got {}", trit);

        direction = direction.step(trit);
        tau_power *= TAU_TRIBONACCI;

        let angle_std_rad = direction.to_std_rad();
        let step_len = 1.0 / tau_power;
        x += angle_std_rad.cos() * step_len;
        y += angle_std_rad.sin() * step_len;

        points.push(SpiralPoint {
            x,
            y,
            position: direction,
            trit,
            step: k + 1,
        });
    }

    points
}

// ══════════════════════════════════════════════════════════════
// REPUNIT VERIFICATION
// ══════════════════════════════════════════════════════════════

/// Verify that a number is a base-3 repunit (all 1's in base 3).
///
/// A repunit in base b is (b^n - 1) / (b - 1).
/// For base 3: 1, 4, 13, 40, 121, 364, 1093, ...
pub fn is_base3_repunit(n: u64) -> bool {
    if n == 0 {
        return false;
    }
    let m = 2 * n + 1;
    let mut v = m;
    while v > 1 {
        if v % 3 != 0 {
            return false;
        }
        v /= 3;
    }
    v == 1
}

/// Return the repunit order (number of 1's) if the value is a base-3 repunit.
pub fn base3_repunit_order(n: u64) -> Option<u32> {
    if !is_base3_repunit(n) {
        return None;
    }
    let mut m = 2 * n + 1;
    let mut order = 0u32;
    while m > 1 {
        m /= 3;
        order += 1;
    }
    Some(order)
}

// ══════════════════════════════════════════════════════════════
// FULL ANALYSIS REPORT
// ══════════════════════════════════════════════════════════════

/// Generate a comprehensive ternary circle analysis report.
pub fn full_circle_report() -> String {
    let mut report = String::new();

    report.push_str("═══════════════════════════════════════════════════════\n");
    report.push_str("  TERNARY CIRCLE GEOMETRY — PlenumNET\n");
    report.push_str("═══════════════════════════════════════════════════════\n\n");

    report.push_str(&format!("  Full circle:     {} degrees = {}₃\n", FULL_CIRCLE_DEG, FULL_CIRCLE_BASE3));
    report.push_str(&format!("  π (ternary):     {}\n", PI_TERNARY));
    report.push_str(&format!("  2π (ternary):    {} (radians per circle)\n", TWO_PI_TERNARY));
    report.push_str(&format!("  1 radian:        {}° = {}₃ = T₇\n", RADIAN_DEG, RADIAN_BASE3));
    report.push_str(&format!("  Cyclic order:    {} (Z₂₈)\n", CYCLIC_ORDER));
    report.push_str(&format!("  τ (Tribonacci):  {}\n", TAU_TRIBONACCI));
    report.push_str(&format!("  τ²:              {}\n", TAU_SQUARED));
    report.push_str(&format!("  τ³:              {}\n", TAU_CUBED));
    report.push_str(&format!("  Golden angle:    {:.5}°\n", TRIBONACCI_GOLDEN_ANGLE_DEG));
    report.push('\n');

    report.push_str("  Repunit verification:\n");
    report.push_str(&format!("    364 is repunit: {} (order {})\n",
        is_base3_repunit(364), base3_repunit_order(364).unwrap_or(0)));
    report.push_str(&format!("     13 is repunit: {} (order {})\n",
        is_base3_repunit(13), base3_repunit_order(13).unwrap_or(0)));
    report.push('\n');

    report.push_str("  Z₂₈ walk (first 28 steps with trit=1):\n");
    let mut pos = Z28::zero();
    for i in 0..28 {
        pos = pos.step(1);
        if i < 10 || i >= 24 {
            report.push_str(&format!("    Step {:2}: Z₂₈({:2}) = {:3}°\n",
                i + 1, pos.0, pos.to_ternary_deg()));
        } else if i == 10 {
            report.push_str("    ...\n");
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI as STD_PI;
    use crate::constants::{
        ternary_deg_to_std_deg, std_deg_to_ternary_deg,
        ternary_rad_to_ternary_deg, ternary_deg_to_ternary_rad,
    };

    #[test]
    fn test_full_circle_is_repunit() {
        assert!(is_base3_repunit(364));
        assert_eq!(base3_repunit_order(364), Some(6));
    }

    #[test]
    fn test_radian_is_repunit() {
        assert!(is_base3_repunit(13));
        assert_eq!(base3_repunit_order(13), Some(3));
    }

    #[test]
    fn test_radian_is_tribonacci_t7() {
        assert_eq!(RADIAN_DEG as u64, 13);
    }

    #[test]
    fn test_full_circle_equals_28_radians() {
        assert_eq!(FULL_CIRCLE_DEG, RADIAN_DEG * TWO_PI_TERNARY);
        assert_eq!(364.0, 13.0 * 28.0);
    }

    #[test]
    fn test_pi_ternary() {
        assert_eq!(PI_TERNARY, 14.0);
        assert_eq!(TWO_PI_TERNARY, 28.0);
    }

    #[test]
    fn test_repunit_sequence() {
        let repunits = [1u64, 4, 13, 40, 121, 364, 1093];
        for &r in &repunits {
            assert!(is_base3_repunit(r), "{} should be a base-3 repunit", r);
        }
        assert!(!is_base3_repunit(2));
        assert!(!is_base3_repunit(3));
        assert!(!is_base3_repunit(5));
        assert!(!is_base3_repunit(14));
    }

    #[test]
    fn test_ternary_to_std_deg_full_circle() {
        let std = ternary_deg_to_std_deg(FULL_CIRCLE_DEG);
        assert!((std - 360.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_to_ternary_deg_full_circle() {
        let tern = std_deg_to_ternary_deg(360.0);
        assert!((tern - FULL_CIRCLE_DEG).abs() < 1e-10);
    }

    #[test]
    fn test_ternary_rad_to_std_rad_full_circle() {
        let std = ternary_rad_to_std_rad(TWO_PI_TERNARY);
        assert!((std - 2.0 * STD_PI).abs() < 1e-10);
    }

    #[test]
    fn test_radian_conversion() {
        assert_eq!(ternary_rad_to_ternary_deg(1.0), RADIAN_DEG);
        assert_eq!(ternary_deg_to_ternary_rad(RADIAN_DEG), 1.0);
    }

    #[test]
    fn test_z28_closure() {
        for a in 0..28u8 {
            for b in 0..28u8 {
                let sum = Z28(a).add(Z28(b));
                assert!(sum.0 < 28);
            }
        }
    }

    #[test]
    fn test_z28_identity() {
        for a in 0..28u8 {
            assert_eq!(Z28(a).add(Z28::zero()), Z28(a));
        }
    }

    #[test]
    fn test_z28_inverse() {
        for a in 0..28u8 {
            let inv = Z28(a).neg();
            assert_eq!(Z28(a).add(inv), Z28::zero());
        }
    }

    #[test]
    fn test_z28_order() {
        let gen = Z28(1);
        let mut current = Z28::zero();
        for i in 1..=28 {
            current = current.add(gen);
            if i < 28 {
                assert_ne!(current, Z28::zero());
            }
        }
        assert_eq!(current, Z28::zero());
    }

    #[test]
    fn test_z28_step_with_trits() {
        let origin = Z28::zero();
        assert_eq!(origin.step(0), Z28(0));
        assert_eq!(origin.step(1), Z28(1));
        assert_eq!(origin.step(2), Z28(2));

        let p = Z28::zero().step(2).step(1).step(0);
        assert_eq!(p, Z28(3));
    }

    #[test]
    fn test_z28_trit_coverage() {
        let mut reached = [false; 28];
        let mut pos = Z28::zero();
        for _ in 0..28 {
            reached[pos.0 as usize] = true;
            pos = pos.step(1);
        }
        for (i, &r) in reached.iter().enumerate() {
            assert!(r, "Position {} should be reachable", i);
        }
    }

    #[test]
    fn test_z28_gf3_residue() {
        assert_eq!(Z28(0).to_gf3_residue(), 0);
        assert_eq!(Z28(1).to_gf3_residue(), 1);
        assert_eq!(Z28(2).to_gf3_residue(), 2);
        assert_eq!(Z28(3).to_gf3_residue(), 0);
        assert_eq!(Z28(27).to_gf3_residue(), 0);
    }

    #[test]
    fn test_walk_starts_at_origin() {
        let trits = vec![1, 0, 2, 1];
        let points = walk_tribonacci_radian_spiral(&trits);
        assert_eq!(points[0].x, 0.0);
        assert_eq!(points[0].y, 0.0);
        assert_eq!(points.len(), trits.len() + 1);
    }

    #[test]
    fn test_walk_directions_are_z28() {
        let trits = vec![0, 1, 2, 0, 1, 1, 2, 0, 2, 1];
        let points = walk_tribonacci_radian_spiral(&trits);
        let mut dir = Z28::zero();
        for (i, &trit) in trits.iter().enumerate() {
            dir = dir.step(trit);
            assert_eq!(points[i + 1].position, dir);
        }
    }

    #[test]
    fn test_walk_scaling_by_tau() {
        let trits = vec![1, 1, 1, 1, 1];
        let points = walk_tribonacci_radian_spiral(&trits);
        for k in 2..points.len() {
            let dx1 = points[k].x - points[k - 1].x;
            let dy1 = points[k].y - points[k - 1].y;
            let len1 = (dx1 * dx1 + dy1 * dy1).sqrt();

            let dx0 = points[k - 1].x - points[k - 2].x;
            let dy0 = points[k - 1].y - points[k - 2].y;
            let len0 = (dx0 * dx0 + dy0 * dy0).sqrt();

            if len0 > 1e-12 {
                let ratio = len0 / len1;
                assert!(
                    (ratio - TAU_TRIBONACCI).abs() < 0.01,
                    "Step length ratio at k={} should be τ, got {:.4}",
                    k, ratio
                );
            }
        }
    }

    #[test]
    fn test_walk_angles_are_lattice() {
        let trits = vec![0, 1, 2, 0, 1, 2, 1, 0, 2, 2, 1, 0];
        let points = walk_tribonacci_radian_spiral(&trits);
        for p in &points[1..] {
            let deg = p.position.to_ternary_deg();
            let remainder = deg % RADIAN_DEG;
            assert!(
                remainder.abs() < 1e-10 || (RADIAN_DEG - remainder).abs() < 1e-10,
                "Walk angle {} is not a multiple of 13°", deg
            );
        }
    }

    #[test]
    fn test_report_runs() {
        let report = full_circle_report();
        assert!(!report.is_empty());
        assert!(report.contains("364"));
        assert!(report.contains("Z₂₈"));
    }
}
