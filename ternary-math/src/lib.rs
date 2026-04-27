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
//!
//! ## Canonical engine re-export (Task #158 I-48 anchor)
//!
//! The single source of pure-ternary mathematical truth for the
//! workspace lives in [`algeometric_arc_sigma182_calculi`] (re-exported
//! here as [`aasc`]). Per-module shims that route `TritInt`, `repx`,
//! `tri182`, `borromean`, `plenum_square`, `ternary_circle`,
//! `coprime`, `repunit_circles`, `tribonacci`, `gf3` over the canonical
//! engine are landing incrementally — see Task #158 step 14 and the
//! Task #154 `M1A.std-shim` rebuild for status. New code should reach
//! into [`aasc`] directly; the existing `TritInt` and helper module
//! surface stays put behind `host_u32`/`host_u64`/`host_u128`/
//! `from_host_u64`/`from_host_u128`/`const_eq` boundary methods until
//! the per-module rewrites complete.
pub use ::aasc;

pub mod constants;
pub mod trit_int;
pub mod trit;
pub mod tri182;
pub mod ags;
pub mod gf3;
pub mod repx;
/// Deprecated alias for `repx`. Was previously named `gf3_algebra`.
/// Use the `repx` module directly for new code.
#[deprecated(since = "0.2.0", note = "use `repx`")]
pub use crate::repx as gf3_algebra;
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
pub use tlsponge385 as sponge;
pub mod tl_dsa;
pub mod plenum_square;
pub mod cube_addr;
pub mod pt26_dsa;
pub mod pt26_walk;
pub mod ttc;
pub mod cpd;
pub mod container_decomp;
pub mod ctx_ans;
pub mod ternary_lattice;
pub mod tl_kem;
pub mod phase_encryption;
pub mod derivation_audit;
pub mod wasm_exports;
pub mod coprime;
pub mod sparse;
