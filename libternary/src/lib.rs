//! # libternary — Bijective Ternary Logic Library
//!
//! The core arithmetic engine of the **Salvi Framework**, providing
//! ternary numeration, Tribonacci sequence generation, Borromean
//! topology primitives, and the 364° ternary circle geometry.
//!
//! ## The Kernel: Representations A, B, C
//!
//! The entire library operates on one principle: there are **three
//! equivalent digit encodings** for the same ternary values, and
//! the kernel translates losslessly between them at every boundary.
//!
//! | Repr | Digits | Domain | Translation from B |
//! |------|--------|--------|--------------------|
//! | **A** (Balanced) | `{-1, 0, +1}` | Signed arithmetic, negation | Subtract 1 (with carry) |
//! | **B** (Standard) | `{0, 1, 2}` | Recurrence, analysis | Identity (internal) |
//! | **C** (Bijective) | `{1, 2, 3}` | Wire format, crypto | Add 1 (with carry) |
//!
//! Internal computation uses **Rep B**. Conversion happens at
//! module boundaries via `to_repr_a()`, `to_repr_c()`, etc.
//!
//! ## The Ternary Circle
//!
//! The framework's angular system:
//! - Full circle = **364°** = `111111₃` (six-digit base-3 repunit)
//! - **π = 14** (circumference / diameter)
//! - **1 radian = 13°** = `111₃` = T₇ (seventh Tribonacci number)
//! - **28 ternary radians** per circle → cyclic group **Z₂₈**
//!
//! ## Modules
//!
//! - [`tribonacci`] — Tribonacci recurrence in base 3, τ expansion,
//!   carry tracking, alignment detection, Rep A/B/C conversions
//! - [`borromean`] — Three-word ternary XOR invariant for
//!   non-separable linking (cryptographic handshake validation)
//! - [`ternary_circle`] — 364° geometry, Z₂₈ cyclic group,
//!   radian spiral walk engine, repunit verification
//!
//! ## Feature Flags
//!
//! - `wasm` — Enable `wasm-bindgen` exports for browser targets
//! - `serde` — Derive `Serialize`/`Deserialize` on core types
//! - `rand` — Enable random generation of ternary words (testing)
//!
//! ## Example
//!
//! ```rust
//! use libternary::tribonacci::{TribonacciBase3, TernaryRepr};
//! use libternary::ternary_circle::{Z28, RADIAN_DEG, walk_tribonacci_radian_spiral};
//!
//! // Generate Tribonacci sequence in base 3
//! let mut gen = TribonacciBase3::new();
//! let terms: Vec<_> = (0..20).map(|_| gen.next_term()).collect();
//!
//! // View T(10) = 81 = 10000₃ in all three representations
//! let t10 = &terms[10];
//! println!("Rep B: {}", t10.format_repr(TernaryRepr::Standard));
//! println!("Rep A: {}", t10.format_repr(TernaryRepr::Balanced));
//! println!("Rep C: {}", t10.format_repr(TernaryRepr::Bijective));
//!
//! // Walk the ternary radian spiral on Z₂₈
//! let trits = vec![1, 2, 0, 1, 0, 2, 2, 0, 1, 1];
//! let points = walk_tribonacci_radian_spiral(&trits);
//! for p in &points[1..] {
//!     println!("Step {}: Z₂₈({}) = {}°, pos=({:.4}, {:.4})",
//!         p.step, p.position.0, p.position.0 as f64 * RADIAN_DEG,
//!         p.x, p.y);
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/libternary/0.1.0")]
#![allow(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::similar_names)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::manual_contains)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cloned_instead_of_copied)]
// Nightly: uncomment for doc-cfg labels on feature-gated items
// #![cfg_attr(docsrs, feature(doc_cfg))]

pub mod borromean;
pub mod fm_timing;
pub mod ternary_circle;
pub mod tribonacci;

// ══════════════════════════════════════════════════════════════
// CORE TYPE — Balanced Ternary Trit
// ══════════════════════════════════════════════════════════════

/// Balanced ternary trit — the fundamental unit of the Salvi Framework.
/// Representations: A={-1,0,+1}, B={0,1,2}, C={1,2,3}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TernaryTrit {
    /// Behind / decreasing / -1
    Neg = -1,
    /// Synchronized / flat / 0
    Zero = 0,
    /// Ahead / increasing / +1
    Pos = 1,
}

impl TernaryTrit {
    /// Convert to i8
    pub fn to_i8(self) -> i8 {
        self as i8
    }

    /// Convert from i8
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Self::Neg),
            0 => Some(Self::Zero),
            1 => Some(Self::Pos),
            _ => None,
        }
    }
}

// ══════════════════════════════════════════════════════════════
// RE-EXPORTS — the most commonly used types at crate root
// ══════════════════════════════════════════════════════════════

pub use borromean::{TernaryWord, WordRepr};
pub use fm_timing::gf3_gradient::{TernaryGradient, ToroidalAxis};
pub use fm_timing::hrv::{EntropyHealth, HrvEntropy};
pub use fm_timing::oscillator::TonalOscillator;
pub use fm_timing::packet::{FmTimingPacket, FrequencyState, PacketError};
pub use ternary_circle::{
    base3_repunit_order, is_base3_repunit, ternary_deg_to_std_deg, ternary_rad_to_std_rad,
    trit_to_std_rad, walk_tribonacci_radian_spiral, CYCLIC_ORDER, FULL_CIRCLE_DEG, PI_TERNARY,
    RADIAN_DEG, TAU_TRIBONACCI, TWO_PI_TERNARY, Z28,
};
pub use tribonacci::{TernaryRepr, TribonacciBase3, TribonacciTerm, TritVec};

// ══════════════════════════════════════════════════════════════
// CRATE-LEVEL CONSTANTS
// ══════════════════════════════════════════════════════════════

/// Library version (matches Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The Salvi Framework identifier.
pub const FRAMEWORK: &str = "Salvi Framework";

/// The division responsible for this codebase.
pub const DIVISION: &str = "Applied Physics Division";

/// The organization.
pub const ORG: &str = "Capomastro Holdings Ltd.";
