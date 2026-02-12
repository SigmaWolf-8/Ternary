//! # Ternary Mathematics — Salvi Framework
//!
//! Mathematical foundations for PlenumNET's ternary computing platform.
//!
//! ## Modules
//!
//! - **gf3**: Galois Field GF(3) arithmetic with exhaustive axiom verification
//! - **clifford**: Clifford algebra Cl(3,0)/GF(3) for ternary gate composition
//! - **radix**: Radix economy analysis quantifying ternary efficiency
//! - **torus**: Ternary torus network topology for the Torsion Network layer
//! - **ternary_circle**: Canonical ternary circle geometry (364°, π=14, 1 rad=13°, Z₂₈)
//! - **tribonacci**: Native base-3 Tribonacci generator with A/B/C representation interchange
//! - **borromean**: Borromean ternary XOR invariant for three-party cryptographic protocols

pub mod gf3;
pub mod clifford;
pub mod radix;
pub mod torus;
pub mod ternary_circle;
pub mod tribonacci;
pub mod borromean;
