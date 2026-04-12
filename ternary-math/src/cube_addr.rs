// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # CubeAddr — Lightweight 13-trit Rep C address
//!
//! Minimal address type for PT26-DSA use within `ternary-math`.
//! The full-featured `CubeAddr` lives in `services/inter-cube/src/cube_addr.rs`;
//! this version provides only the subset needed by the signature scheme,
//! avoiding a circular dependency between the two crates.

/// Number of dimensions in the ternary hypercube = R₃.
pub const DIMENSIONS: usize = crate::constants::T_REPUNIT_3.to_u32_const() as usize;

/// A 13-trit Rep C address (values 1, 2, 3).
///
/// Lightweight clone of the full `inter_cube::CubeAddr` — kept in
/// `ternary-math` so PT26-DSA can live at the math layer without
/// pulling in the networking crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CubeAddr {
    trits: [u8; DIMENSIONS],
}

impl CubeAddr {
    /// Create from 13 Rep C trit values. Panics if any value is outside {1,2,3}.
    pub fn new(trits: [u8; DIMENSIONS]) -> Self {
        for &t in &trits {
            assert!(t >= 1 && t <= 3, "Rep C trits must be in {{1,2,3}}, got {}", t);
        }
        CubeAddr { trits }
    }

    /// Try to create from a byte slice. Returns `None` if length != 13
    /// or any byte is outside {1,2,3}.
    pub fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != DIMENSIONS {
            return None;
        }
        let mut trits = [0u8; DIMENSIONS];
        for (i, &b) in bytes.iter().enumerate() {
            if b < 1 || b > 3 {
                return None;
            }
            trits[i] = b;
        }
        Some(CubeAddr { trits })
    }

    /// Export as raw byte array.
    #[inline]
    pub fn to_bytes(&self) -> [u8; DIMENSIONS] {
        self.trits
    }
}
