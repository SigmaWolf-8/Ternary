// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `speed_of_light` — `v² = π`, base-3 uniqueness, Gabriel's Horn
//!
//! In framework units the propagation speed satisfies `v² = π = 14`.
//! The integer-square-root pin is therefore that no integer `v`
//! exactly equals √π — `v² = π` is a statement *about* the integer
//! 14, not a value of `v`. The base-3 uniqueness theorem says that
//! the same identity, in any base `b ≠ 3`, produces a non-prime
//! discriminant; only base 3 keeps the identity coherent.
//!
//! ## Invariants verified at compile time
//!
//! - **I-36.** `π = 14` and the squared-speed equation are pinned.

use crate::constants::PI_INT;

/// Squared propagation speed in framework units.
pub const V_SQUARED: u64 = PI_INT;

const _: () = {
    assert!(V_SQUARED == 14);
};
