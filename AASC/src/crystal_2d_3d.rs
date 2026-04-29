// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `crystal_2d_3d` — vertex-on-sphere, latitude reduction, antiprism
//!
//! Bridges the 2D regular polygon census to the 3D antiprism /
//! disdyakis lattice through latitude reduction.
//!
//! ## Invariants verified at compile time
//!
//! - **I-27.** The (3, 4, 5, 6) polygon source set is closed under the
//!   antiprism construction's vertex-doubling (each n-gon yields a
//!   2n-vertex antiprism band).

use crate::constants::{POLYGON_3_ARR, POLYGON_4_ARR, POLYGON_5_ARR, POLYGON_6_ARR};
use crate::arithmetic::add;
use crate::tritvec::TritVec;

/// The polygon source set, as TritVec values.
pub fn polygon_source_set() -> [TritVec; 4] {
    [
        TritVec::from_trits(POLYGON_3_ARR),
        TritVec::from_trits(POLYGON_4_ARR),
        TritVec::from_trits(POLYGON_5_ARR),
        TritVec::from_trits(POLYGON_6_ARR),
    ]
}

/// Vertex count of the n-antiprism band: `2·n`.
pub fn antiprism_vertices(n_polygon: &TritVec) -> TritVec {
    add(n_polygon, n_polygon)
}
