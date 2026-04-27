// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `saturnian_metatron` — Saturnian shells & metatronic automorphism
//!
//! Concentric "Saturnian" shells of the framework correspond to
//! `R_L`-radius repunit circles; the metatronic automorphism is the
//! cyclic rotation that maps shell `L` to shell `L+1` via the repunit
//! recurrence `R_{L+1} = b·R_L + R_1`.
//!
//! ## Invariants verified at compile time
//!
//! - **I-32.** Metatronic automorphism preserves the repunit recurrence.

use crate::constants::{R_3_INT, R_4_INT};
use crate::repunit_circles::next_repunit;

const _: () = {
    // I-32 — one step of the automorphism
    assert!(next_repunit(R_3_INT) == R_4_INT);
};
