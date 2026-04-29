// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

#![allow(non_snake_case)]

//! # Algeometric Arc Sigma182 Calculi (`AASC`)
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

#[path = "2.1.1.UX1.5__trit.rs"]
pub mod trit;
#[path = "2.2.1.UX1.5__tritvec.rs"]
pub mod tritvec;
#[path = "2.3.1.UX4.1(2.1)__arithmetic.rs"]
pub mod arithmetic;
#[path = "1.1.1.UX1.1__constants.rs"]
pub mod constants;
#[path = "3.1.1.UX4.1(2.7)__gf3.rs"]
pub mod gf3;
#[path = "3.2.1.UX4.1__zphi.rs"]
pub mod zphi;
#[path = "3.3.1.UX3.3__borromean.rs"]
pub mod borromean;
#[path = "3.4.1.UX4.1(2.4)__coprime.rs"]
pub mod coprime;

// ════════════════════════════════════════════════════════════════════════
// Geometric layer
// ════════════════════════════════════════════════════════════════════════

#[path = "5.1.1.UX5.1__circle.rs"]
pub mod circle;
#[path = "5.3.1.UX5.1__generating_system.rs"]
pub mod generating_system;
#[path = "5.2.2.UX5.1__dual_circle.rs"]
pub mod dual_circle;
#[path = "5.5.1.UX5.1__coprime_polygon_pair.rs"]
pub mod coprime_polygon_pair;
#[path = "5.1.3.UX5.1__plenum_color_harmonics.rs"]
pub mod plenum_color_harmonics;
#[path = "5.6.1.UX5.1(2.6)__repunit_circles.rs"]
pub mod repunit_circles;
#[path = "7.3.1.UX4.1(2.3)__tribonacci.rs"]
pub mod tribonacci;
#[path = "7.1.1.UX4.1__triangular_numbers.rs"]
pub mod triangular_numbers;
#[path = "5.4.3.UX5.1__arc182.rs"]
pub mod arc182;
#[path = "5.A.1.UX5.2__plenum_square.rs"]
pub mod plenum_square;
#[path = "5.4.1.UX5.1__gait.rs"]
pub mod gait;
#[path = "5.7.1.UX5.1__disdyakis_bridge.rs"]
pub mod disdyakis_bridge;
#[path = "8.1.1.UX4.1__crystal_2d_3d.rs"]
pub mod crystal_2d_3d;
#[path = "9.5.1.UX4.2__uv_spectral.rs"]
pub mod uv_spectral;
#[path = "9.1.1.UX4.2__hydrogen_spectral.rs"]
pub mod hydrogen_spectral;
#[path = "5.1.2.UX4.1__speed_of_light.rs"]
pub mod speed_of_light;
#[path = "5.7.2.UX5.1__saturnian_metatron.rs"]
pub mod saturnian_metatron;
#[path = "6.1.1.UX4.2.9__wave_stratum.rs"]
pub mod wave_stratum;
#[path = "6.2.1.UX4.1__conservation_laws.rs"]
pub mod conservation_laws;
#[path = "8.3.1.UX1.5(2.4)__nona_state.rs"]
pub mod nona_state;
#[path = "7.4.2.UX4.1__wieferich_register.rs"]
pub mod wieferich_register;
#[path = "7.4.3.UX4.1__grh_register.rs"]
pub mod grh_register;
#[path = "7.4.4.UX4.1__beal_register.rs"]
pub mod beal_register;
#[path = "7.5.1.UX4.1__pqr_asymmetry.rs"]
pub mod pqr_asymmetry;
#[path = "5.2.1.UX5.1__discriminant.rs"]
pub mod discriminant;
#[path = "9.8.1.UX4.1__discriminant_identity.rs"]
pub mod discriminant_identity;
#[path = "5.B.1.UX5.2__cone_point.rs"]
pub mod cone_point;

// ════════════════════════════════════════════════════════════════════════
// Calculi engines
// ════════════════════════════════════════════════════════════════════════

#[path = "4.6.1.UX1.5(2.5)__repx.rs"]
pub mod repx;
#[path = "1.4.1.UX1.1(2.5)__milesian.rs"]
pub mod milesian;
#[path = "3.5.1.UX9.9(2.4)__walk.rs"]
pub mod walk;
#[path = "4.5.1.UX4.1__calculus.rs"]
pub mod calculus;
#[path = "4.7.1.UX4.1__attestation_ledger.rs"]
pub mod attestation_ledger;

// ════════════════════════════════════════════════════════════════════════
// Boundary (gated)
// ════════════════════════════════════════════════════════════════════════

#[cfg(feature = "bridge")]
#[path = "2.4.1.UX1.5__bridge.rs"]
pub mod bridge;

// ════════════════════════════════════════════════════════════════════════
// Re-exports — the canonical surface
// ════════════════════════════════════════════════════════════════════════

pub use trit::{Trit, Representation};
pub use tritvec::TritVec;

/// Crate version (for telemetry and audit).
pub const AASC_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name (for telemetry and audit).
pub const AASC_NAME: &str = "AASC";
