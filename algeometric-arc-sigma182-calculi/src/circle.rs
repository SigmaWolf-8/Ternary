// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `circle` — the canonical ternary circle
//!
//! Within the Salvi Framework the circle is **structurally derived**, not
//! transcendentally given:
//!
//! - π is the integer `(1 + b³)/2 = 14`.
//! - The full circle has `R₆ = 364` degrees (not 360).
//! - One radian equals `R₃ = 13` degrees.
//! - One full revolution is `2π = 28` radians.
//!
//! ## Invariants verified at compile time
//!
//! - **I-16.** `R₆ = 2π · R₃` (degrees per revolution = radians × deg/rad).
//! - **I-17.** Half-circle = `R₆ / 2 = ARC = 182`.

use crate::constants::{ARC_INT, PI_INT, R_3_INT, R_6_INT, TWO_PI_INT};

/// Degrees in one full revolution (`R₆`).
pub const DEGREES_PER_REVOLUTION: u64 = R_6_INT;
/// Degrees in one radian (`R₃`).
pub const DEGREES_PER_RADIAN: u64 = R_3_INT;
/// Radians in one full revolution (`2π = 28`).
pub const RADIANS_PER_REVOLUTION: u64 = TWO_PI_INT;
/// Half-circle (the Σ-182 axis).
pub const HALF_CIRCLE_DEGREES: u64 = ARC_INT;

/// Convert radians to degrees (whole-unit framework arithmetic).
pub const fn radians_to_degrees(r: u64) -> u64 {
    r * DEGREES_PER_RADIAN
}

/// Convert degrees to radians via `divmod`. Returns `(quotient, remainder)`.
pub const fn degrees_to_radians(d: u64) -> (u64, u64) {
    (d / DEGREES_PER_RADIAN, d % DEGREES_PER_RADIAN)
}

const _: () = {
    // I-16
    assert!(R_6_INT == TWO_PI_INT * R_3_INT);
    assert!(R_6_INT == 2 * PI_INT * R_3_INT);
    // I-17
    assert!(R_6_INT / 2 == ARC_INT);
    assert!(HALF_CIRCLE_DEGREES == 182);
};
