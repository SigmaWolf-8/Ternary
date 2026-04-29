// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # Algeometric Arc Sigma182 Calculi (`aasc`)
//!
//! The single canonical pure-ternary computation crate of the Salvi Framework /
//! PlenumNET platform. Consolidates every ternary primitive in the workspace
//! into one mathematical truth.
//!
//! ## Core invariants
//!
//! - **I-1.** The trit alphabet has exactly three symbols. Internal representation
//!   is Rep-C `{1, 2, 3}`. Three pure constructors `from_a`, `from_b`, `from_c`.
//! - **I-2.** No `from_u64`, no host conversions in the public surface of core
//!   modules. Bytes appear only in `bridge` (gated behind `feature = "bridge"`).
//! - **I-3.** A single boundary call `digit.to_index()` is permitted in `milesian`,
//!   restricted to digits guaranteed `≤ b³`.
//! - **I-4.** The crate is `no_std`-clean (`core` + `alloc` only) on the default
//!   feature path.
//!
//! ## Module map
//!
//! ### Foundation (algebra)
//! - [`trit`] / [`tritvec`] — the pure trit and trit-vector types
//! - [`arithmetic`] — pure trit arithmetic (add, sub, mul, divmod, …)
//! - [`constants`] — Notation table, the **single** place numerals appear
//! - [`gf3`] — Galois Field GF(3) arithmetic with axiom verification
//! - [`zphi`] — ℤ[φ] golden-ratio integer ring (`R² = π + 5φ`)
//! - [`borromean`] — Borromean ternary XOR invariant
//! - [`coprime`] — coprimality, Euler totient, walks, CRT
//!
//! ### Geometric layer
//! - [`circle`] — canonical ternary circle (R₆ = 364°, π = 14, 1 rad = R₃ deg)
//! - [`generating_system`] — the unifying quadratic `arc² − 832·arc + 118300 = 0`
//! - [`dual_circle`] — `Z_{Z_dual} ≅ Z_{b³} × Z_{2π}` CRT bijection
//! - [`coprime_polygon_pair`] — Generator Duality Theorem; the (11, 13) pair
//! - [`plenum_color_harmonics`] — ARC_RED / ARC_BLUE / ARC_COPRIME / √Δ_arc / ARC_GREEN
//! - [`repunit_circles`] — repunit-radius circles, `R_L = b·R_{L-1} + 1`
//! - [`tribonacci`] — base-b Tribonacci ladder
//! - [`triangular_numbers`] — `T_n = n·(n+1)/2`; Triple Identity at R₃
//! - [`arc182`] — Σ-182 axis: `ARC = π·(π−1) = 2·p·r = R₆/2 = 2·Λ_EUV`
//! - [`plenum_square`] — magic-square configurations + four σ permutations
//! - [`gait`] — Greek Atomic Invariant Tracker (b³ Milesian register)
//! - [`disdyakis_bridge`] — `R²_disdyakis = π + 5φ`, defects, ℤ[φ]-norm hierarchy
//! - [`crystal_2d_3d`] — vertex-on-sphere, latitude reduction, antiprism
//! - [`uv_spectral`] — UV system wavelength chain
//! - [`hydrogen_spectral`] — `R₆/4` bridge to `1/R∞`
//! - [`speed_of_light`] — `v² = π`, base-3 uniqueness, Gabriel's Horn
//! - [`saturnian_metatron`] — Saturnian shells, metatronic automorphism
//! - [`wave_stratum`] — vacuum impedance Z₀, dual-layer encoding gap
//! - [`conservation_laws`] — PCO-A six conservation laws (CL-1..CL-6)
//! - [`nona_state`] — Nona State, conservation identity, transfer equation
//! - [`wieferich_register`] — first base-3 Wieferich prime is `p_h`
//! - [`grh_register`] — Optimus Paraprime Theorems 47–52
//! - [`beal_register`] — Five Norm Towers, degree-2 ceiling
//!
//! ### Calculi engines
//! - [`repx`] — TritVec-pure bijective base-`b` converter (Spec v3.3.33 §4)
//! - [`milesian`] — `divmod(b³)` glyph stringer over the b³-symbol register
//! - [`walk`] — Walk + CRT recovery for moduli `(p, q, r)`
//! - [`calculus`] — four sub-calculi (difference, circular, iteration, series)
//!
//! ### Boundary
//! - [`bridge`] — bytes ↔ TritVec, gated behind `feature = "bridge"`
//!
//! ## Salvi Standard of Scrutiny
//!
//! The crate is governed by the UPIID V1.1 Dual-Path Framework. Every
//! algebraic identity is proven once at compile time in a `const _: () = { … };`
//! block; geometric identities are proven in the layer-local const blocks
//! inside each geometric submodule. The crate fails to compile if any
//! identity drifts.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

// ════════════════════════════════════════════════════════════════════════
// Foundation — algebra
// ════════════════════════════════════════════════════════════════════════

pub mod trit;
pub mod tritvec;
pub mod arithmetic;
pub mod constants;
pub mod gf3;
pub mod zphi;
pub mod borromean;
pub mod coprime;

// ════════════════════════════════════════════════════════════════════════
// Geometric layer
// ════════════════════════════════════════════════════════════════════════

pub mod circle;
pub mod generating_system;
pub mod dual_circle;
pub mod coprime_polygon_pair;
pub mod plenum_color_harmonics;
pub mod repunit_circles;
pub mod tribonacci;
pub mod triangular_numbers;
pub mod arc182;
pub mod plenum_square;
pub mod gait;
pub mod disdyakis_bridge;
pub mod crystal_2d_3d;
pub mod uv_spectral;
pub mod hydrogen_spectral;
pub mod speed_of_light;
pub mod saturnian_metatron;
pub mod wave_stratum;
pub mod conservation_laws;
pub mod nona_state;
pub mod wieferich_register;
pub mod grh_register;
pub mod beal_register;

// ════════════════════════════════════════════════════════════════════════
// Calculi engines
// ════════════════════════════════════════════════════════════════════════

pub mod repx;
pub mod milesian;
pub mod walk;
pub mod calculus;
pub mod attestation_ledger;

// ════════════════════════════════════════════════════════════════════════
// Boundary (gated)
// ════════════════════════════════════════════════════════════════════════

#[cfg(feature = "bridge")]
pub mod bridge;

// ════════════════════════════════════════════════════════════════════════
// Re-exports — the canonical surface
// ════════════════════════════════════════════════════════════════════════

pub use trit::{Trit, Representation};
pub use tritvec::TritVec;

/// Crate version (for telemetry and audit).
pub const AASC_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name (for telemetry and audit).
pub const AASC_NAME: &str = "algeometric-arc-sigma182-calculi";
