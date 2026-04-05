// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Array3 Node Cluster Constants
//!
//! Every constant is DERIVED from imported values — no magic numbers.
//! Design principle: two values that happen to be equal but arise from
//! different reasons get separate names. A `const_assert!` documents
//! the intentional equality. Using one in place of the other is a
//! category error.

use static_assertions::const_assert;
use ternary_math::cube_addr::{DIMENSIONS, CubeAddr};
use ternary_math::constants::BORROMEAN_MODULUS;
use ternary_math::repunit_circles::REPUNIT_R6;
#[allow(unused_imports)]
use ternary_math::repunit_circles::REPUNIT_R5;

// ═══════════════════════════════════════════════════════════════════
// Ternary field
// ═══════════════════════════════════════════════════════════════════

pub const GF3_ORDER: usize = BORROMEAN_MODULUS as usize;             // 3
pub const REP_C_CENTER: u8 = 2;                                      // center of {1,2,3}

// ═══════════════════════════════════════════════════════════════════
// Classification schema
// ═══════════════════════════════════════════════════════════════════

/// 27 ontological questions from the TDNS schema.
/// This is the input width of the projection — independent of cube geometry.
pub const CLASSIFICATION_DIMS: usize = 27;

// ═══════════════════════════════════════════════════════════════════
// Cube geometry (design choices, not field-order derivations)
// ═══════════════════════════════════════════════════════════════════

/// The cube is 3D because we chose plane × role × instance.
/// CUBE_DIMS is a design choice. It equals GF3_ORDER by coincidence,
/// not by derivation. A 4D cube would have CUBE_DIMS=4, GF3_ORDER=3.
pub const CUBE_DIMS: usize = 3;
pub const SLOTS_PER_NODE: usize = GF3_ORDER.pow(CUBE_DIMS as u32);  // 3³ = 27
pub const CLUSTER_SLOTS: usize = SLOTS_PER_NODE * GF3_ORDER;        // 81 (3 nodes)
pub const SHELL_FACTORS: usize = GF3_ORDER.pow(2);                  // 3² = 9 (auth shell)

// ═══════════════════════════════════════════════════════════════════
// Port arithmetic
// ═══════════════════════════════════════════════════════════════════

/// The stride per plane is 9 because each plane has role × instance = 3 × 3 slots.
/// This equals SHELL_FACTORS because both are GF3_ORDER² — a ternary resonance,
/// NOT a derivation. The port formula uses SLOTS_PER_PLANE; the auth shell uses
/// SHELL_FACTORS. They happen to be equal. The const_assert! documents this.
pub const SLOTS_PER_PLANE: usize = GF3_ORDER * GF3_ORDER;           // 9 (role × instance)

// ═══════════════════════════════════════════════════════════════════
// Projection
// ═══════════════════════════════════════════════════════════════════

/// 27 classification dims partition into 3 groups of 9.
/// Group size = CLASSIFICATION_DIMS / CUBE_DIMS.
/// This equals SHELL_FACTORS and SLOTS_PER_PLANE — same ternary resonance.
pub const DIMS_PER_GROUP: usize = CLASSIFICATION_DIMS / CUBE_DIMS;   // 9

// ═══════════════════════════════════════════════════════════════════
// Node IDs — Rep C {1, 2, 3}, zero is forgery at every trust boundary
// ═══════════════════════════════════════════════════════════════════

pub const MAX_NODES: usize = GF3_ORDER;                              // 3 nodes per Array3
pub const GATEWAY_NODE_ID: u8 = 1;                                   // Rep C — Node 1 is the gateway

// ═══════════════════════════════════════════════════════════════════
// Gateway (derived from DIMENSIONS)
// ═══════════════════════════════════════════════════════════════════

pub const GATEWAY_OFFSET: usize = DIMENSIONS;                        // 13 = T₇ = 1 ternary radian

/// The geometric center of the 13-dimensional Rep C hypercube.
pub fn gateway_address() -> CubeAddr {
    CubeAddr::new([REP_C_CENTER; DIMENSIONS])  // [2; 13]
}

/// The 3-trit service cube center (slot address of the gateway within a node).
pub const SLOT_CENTER: [u8; CUBE_DIMS] = [REP_C_CENTER; CUBE_DIMS]; // [2, 2, 2]

// ═══════════════════════════════════════════════════════════════════
// Arc equation: arc² − ADMIN_EPOCH·arc + DIMENSIONS²×700 = 0
//
// Two roots, both derived from existing circle constants:
//   Root 1: ARC_EPOCH  = FULL_CIRCLE / 2 = DIMENSIONS × PI_TERNARY = 182
//   Root 2: EVIDENCE_ARC = ADMIN_EPOCH − ARC_EPOCH = 650
// Vieta's formulas: sum = ADMIN_EPOCH, product = DIMENSIONS² × 700
// ═══════════════════════════════════════════════════════════════════

pub const FULL_CIRCLE: usize = REPUNIT_R6 as usize;                 // 364 = 111111₃
pub const ARC_EPOCH: usize = FULL_CIRCLE / 2;                       // 182 = half circle
    // Alternate derivation: DIMENSIONS × PI_TERNARY = 13 × 14
    // Base-3: 20202₃ (palindrome — symmetric, like key rotation)
pub const ADMIN_EPOCH: usize = 2_usize.pow(6) * DIMENSIONS;         // 832 = 2⁶ × 13
pub const EVIDENCE_ARC: usize = ADMIN_EPOCH - ARC_EPOCH;            // 650 = 220002₃ (directional)
pub const REDIRECT_DEPTH: usize = (EVIDENCE_ARC + ARC_EPOCH - 1) / ARC_EPOCH; // ceil(650/182) = 4
pub const TRANSITION_GAP: usize = EVIDENCE_ARC - ARC_EPOCH;         // 468 = 6² × 13
pub const CHECKPOINTS: usize = TRANSITION_GAP / DIMENSIONS;          // 36 = 4 × 9

// ═══════════════════════════════════════════════════════════════════
// Base port (decimal repunit resonating with ternary repunit)
// ═══════════════════════════════════════════════════════════════════

pub const DECIMAL_REPUNIT_DIGITS: u32 = 5;
pub const BASE_PORT: u16 = ((10_u32.pow(DECIMAL_REPUNIT_DIGITS) - 1) / 9) as u16;
    // = 11111. Digit string "11111" in base-3 = REPUNIT_R5 = 121 = 11²

// ═══════════════════════════════════════════════════════════════════
// COMPILE-TIME CROSS-VALIDATION
// Every relationship is verified — if any constant changes upstream,
// compilation fails immediately.
// ═══════════════════════════════════════════════════════════════════

// Design invariant: classification dims == slots per node (intentionally equal)
const_assert!(CLASSIFICATION_DIMS == SLOTS_PER_NODE);

// Cube geometry
const_assert!(SLOTS_PER_NODE == 27);                                   // 3³
const_assert!(CLUSTER_SLOTS == 81);                                    // 3⁴
const_assert!(CLUSTER_SLOTS == SLOTS_PER_NODE * GF3_ORDER);           // 81 = 27 × 3

// Ternary resonances (equal values, different derivations)
const_assert!(SLOTS_PER_PLANE == SHELL_FACTORS);                     // both 3², different reasons
const_assert!(DIMS_PER_GROUP == SLOTS_PER_PLANE);                    // both 9, different reasons

// Arc equation — Vieta's formulas
const_assert!(ARC_EPOCH + EVIDENCE_ARC == ADMIN_EPOCH);              // sum = 832
const_assert!(ARC_EPOCH * EVIDENCE_ARC == DIMENSIONS * DIMENSIONS * 700); // product = 118,300

// Structural
const_assert!(GATEWAY_OFFSET == DIMENSIONS);                          // 13
const_assert!(CHECKPOINTS == REDIRECT_DEPTH * SHELL_FACTORS);        // 36 = 4 × 9
const_assert!(BASE_PORT == 11111);

// Rep C center
const_assert!(REP_C_CENTER == 2);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_epoch_is_half_circle() {
        assert_eq!(ARC_EPOCH, FULL_CIRCLE / 2);
        assert_eq!(2 * ARC_EPOCH, FULL_CIRCLE);
    }

    #[test]
    fn arc_epoch_is_dimensions_times_pi() {
        assert_eq!(ARC_EPOCH, DIMENSIONS * 14); // PI_TERNARY = 14
    }

    #[test]
    fn vieta_sum_and_product() {
        assert_eq!(ARC_EPOCH + EVIDENCE_ARC, ADMIN_EPOCH);
        assert_eq!(ARC_EPOCH * EVIDENCE_ARC, DIMENSIONS * DIMENSIONS * 700);
    }

    #[test]
    fn base_port_repunit_resonance() {
        assert_eq!(BASE_PORT, 11111);
        // Digit string "11111" parsed as base-3 = REPUNIT_R5 = 121
        let parsed_base3: u64 = 1*81 + 1*27 + 1*9 + 1*3 + 1;
        assert_eq!(parsed_base3, REPUNIT_R5);
        assert_eq!(REPUNIT_R5, 121);
        assert_eq!(REPUNIT_R5, 11 * 11); // 11²
    }

    #[test]
    fn gateway_address_is_center() {
        let gw = gateway_address();
        let bytes = gw.to_bytes();
        for &t in &bytes {
            assert_eq!(t, REP_C_CENTER);
        }
        assert_eq!(bytes.len(), DIMENSIONS);
    }

    #[test]
    fn slot_center_is_rep_c_center() {
        for &t in &SLOT_CENTER {
            assert_eq!(t, REP_C_CENTER);
        }
        assert_eq!(SLOT_CENTER.len(), CUBE_DIMS);
    }

    #[test]
    fn transition_structure() {
        assert_eq!(TRANSITION_GAP, 468);
        assert_eq!(TRANSITION_GAP, 36 * 13); // 6² × 13
        assert_eq!(CHECKPOINTS, 36);
        assert_eq!(REDIRECT_DEPTH, 4);
    }
}
