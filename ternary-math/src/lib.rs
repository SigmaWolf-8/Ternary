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

//! # Ternary Mathematics — Salvi Framework
//!
//! Mathematical foundations for PlenumNET's ternary computing platform.
//!
//! ## Modules
//!
//! - **constants**: Unified shared constants — singular source of truth for all modules
//! - **gf3**: Galois Field GF(3) arithmetic with exhaustive axiom verification
//! - **clifford**: Clifford algebra Cl(3,0)/GF(3) for ternary gate composition
//! - **radix**: Radix economy analysis quantifying ternary efficiency
//! - **torus**: Ternary torus network topology for the Torsion Network layer
//! - **ternary_circle**: Canonical ternary circle geometry (364°, π=14, 1 rad=13°, Z₂₈)
//! - **tribonacci**: Native base-3 Tribonacci generator with A/B/C representation interchange
//! - **borromean**: Borromean ternary XOR invariant for three-party cryptographic protocols
pub mod constants;
pub mod gf3;
pub mod gf3_algebra;
pub mod clifford;
pub mod radix;
pub mod torus;
pub mod ternary_circle;
pub mod tribonacci;
pub mod borromean;
pub mod repunit_checksum;
pub mod plenum_checksum;
pub mod repunit_circles;
pub mod tlsponge385;
pub mod tis_sponge;
pub mod tl_dsa;
pub mod plenum_square;
pub mod cube_addr;
pub mod pt26_dsa;
pub mod pt26_walk;
#[cfg(target_arch = "wasm32")]
pub mod wasm_exports;
