// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `trit` — the atomic ternary symbol
//!
//! A single trit. The alphabet has exactly three symbols. Internal
//! storage is **Rep-C** `{1, 2, 3}` (the human-readable wire format).
//! Three pure constructors map every common encoding into the canonical
//! enum. There is no `from_u64`, no `from_bytes`, no host-binary boundary
//! in this module.
//!
//! ## The three representations
//!
//! | Rep | Alphabet     | Used for                     |
//! |-----|--------------|------------------------------|
//! | A   | `{-1, 0, +1}` | balanced arithmetic, GF(3)   |
//! | B   | `{0, 1, 2}`   | unsigned base-3 / network    |
//! | C   | `{1, 2, 3}`   | human-readable, MSB-first    |
//!
//! ## Bijections
//!
//! - A→B: `f(a) = a + 1`
//! - A→C: `f(a) = a + 2`
//! - B→C: `f(b) = b + 1`
//! - C→A: `f(c) = c − 2`
//! - C→B: `f(c) = c − 1`
//!
//! These are [`Trit::value_a`], [`Trit::value_b`], [`Trit::value_c`] and
//! the three pure constructors.
//!
//! ## Invariants
//!
//! - **I-1.** The trit alphabet has exactly three symbols. Enforced by
//!   the enum definition.
//! - **I-2.** No `from_u64`, no host conversions in the public surface.
//!   The constructors take `i8`/`u8` purely as host narrowest-cell
//!   transport; they validate strictly and return `Option<Self>`.

use core::fmt;

// ════════════════════════════════════════════════════════════════════════
// Representation tag
// ════════════════════════════════════════════════════════════════════════

/// Names one of the three canonical trit representations.
///
/// `convert_representation` and the boundary methods on [`Trit`] pivot
/// through this tag so callers can be explicit about which encoding
/// they speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Representation {
    /// Balanced `{-1, 0, +1}` — used for arithmetic and GF(3).
    A,
    /// Unsigned `{0, 1, 2}` — used for base-3 and network transmission.
    B,
    /// Human `{1, 2, 3}` — used for MSB-first display and the wire format.
    C,
}

// ════════════════════════════════════════════════════════════════════════
// The trit
// ════════════════════════════════════════════════════════════════════════

/// A single trit. The atomic symbol of every higher type in the crate.
///
/// Three variants. Internal storage is Rep-C — the variant *name* is
/// the Rep-C value, the discriminant is the Rep-B value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Trit {
    /// Rep-C 1 = Rep-B 0 = Rep-A −1.
    One = 0,
    /// Rep-C 2 = Rep-B 1 = Rep-A 0.
    Two = 1,
    /// Rep-C 3 = Rep-B 2 = Rep-A +1.
    Three = 2,
}

impl Trit {
    // ─── Pure constructors ───────────────────────────────────────────

    /// Construct from Rep-A (balanced) value.
    ///
    /// Returns `None` for any value outside `{-1, 0, +1}`.
    #[inline]
    pub const fn from_a(value: i8) -> Option<Self> {
        match value {
            -1 => Some(Trit::One),
            0 => Some(Trit::Two),
            1 => Some(Trit::Three),
            _ => None,
        }
    }

    /// Construct from Rep-B (unsigned) value.
    ///
    /// Returns `None` for any value outside `{0, 1, 2}`.
    #[inline]
    pub const fn from_b(value: u8) -> Option<Self> {
        match value {
            0 => Some(Trit::One),
            1 => Some(Trit::Two),
            2 => Some(Trit::Three),
            _ => None,
        }
    }

    /// Construct from Rep-C (human) value.
    ///
    /// Returns `None` for any value outside `{1, 2, 3}`.
    #[inline]
    pub const fn from_c(value: u8) -> Option<Self> {
        match value {
            1 => Some(Trit::One),
            2 => Some(Trit::Two),
            3 => Some(Trit::Three),
            _ => None,
        }
    }

    // ─── Pure accessors ──────────────────────────────────────────────

    /// Read Rep-A (balanced) value.
    #[inline]
    pub const fn value_a(self) -> i8 {
        match self {
            Trit::One => -1,
            Trit::Two => 0,
            Trit::Three => 1,
        }
    }

    /// Read Rep-B (unsigned) value.
    #[inline]
    pub const fn value_b(self) -> u8 {
        match self {
            Trit::One => 0,
            Trit::Two => 1,
            Trit::Three => 2,
        }
    }

    /// Read Rep-C (human) value.
    #[inline]
    pub const fn value_c(self) -> u8 {
        match self {
            Trit::One => 1,
            Trit::Two => 2,
            Trit::Three => 3,
        }
    }

    /// Convenience aliases that match the kernel surface (back-compat).
    #[inline]
    pub const fn to_a(self) -> i8 {
        self.value_a()
    }
    #[inline]
    pub const fn to_b(self) -> u8 {
        self.value_b()
    }
    #[inline]
    pub const fn to_c(self) -> u8 {
        self.value_c()
    }

    // ─── Identity, ordering, equality ────────────────────────────────

    /// The additive identity (Rep-A 0). Used by GF(3) and arithmetic.
    pub const ZERO: Self = Trit::Two;
    /// The multiplicative identity (Rep-A +1).
    pub const ONE: Self = Trit::Three;
    /// The negative unit (Rep-A −1).
    pub const NEG_ONE: Self = Trit::One;

    /// True iff this trit represents Rep-A 0 (the additive identity).
    #[inline]
    pub const fn is_zero(self) -> bool {
        matches!(self, Trit::Two)
    }

    // ─── GF(3) operations ────────────────────────────────────────────

    /// Tritwise NOT (negation in GF(3)): `−x mod 3`.
    ///
    /// Rep-A: `−1 ↔ +1`, `0` is fixed.
    #[inline]
    pub const fn not(self) -> Self {
        match self {
            Trit::One => Trit::Three,
            Trit::Two => Trit::Two,
            Trit::Three => Trit::One,
        }
    }

    /// GF(3) addition: `(x + y) mod 3` with the Rep-A balanced reduction.
    #[inline]
    pub const fn add(self, other: Self) -> Self {
        let a = self.value_a();
        let b = other.value_a();
        let mut s = a + b;
        // Reduce into balanced ternary {-1, 0, +1}
        if s == 2 {
            s = -1;
        } else if s == -2 {
            s = 1;
        }
        // s is now in {-1, 0, 1}; this never panics
        match Trit::from_a(s) {
            Some(t) => t,
            None => Trit::Two, // unreachable
        }
    }

    /// GF(3) subtraction: `(x − y) mod 3`.
    #[inline]
    pub const fn sub(self, other: Self) -> Self {
        self.add(other.not())
    }

    /// GF(3) multiplication: `(x · y) mod 3`.
    #[inline]
    pub const fn mul(self, other: Self) -> Self {
        let p = self.value_a() * other.value_a();
        // Product of {-1, 0, +1} pair always lies in {-1, 0, +1}.
        match Trit::from_a(p) {
            Some(t) => t,
            None => Trit::Two,
        }
    }

    /// GF(3) multiplicative inverse: only `±1` are invertible; `0` returns `None`.
    #[inline]
    pub const fn gf3_inverse(self) -> Option<Self> {
        match self {
            Trit::Two => None,             // 0 has no inverse
            Trit::Three => Some(Trit::Three), // 1⁻¹ = 1
            Trit::One => Some(Trit::One),  // (−1)⁻¹ = −1
        }
    }

    // ─── Boundary: the one explicit conversion path ──────────────────

    /// Convert this trit from one named representation to another.
    ///
    /// This is the *only* representation pivot in the public surface.
    pub const fn convert(self, _from: Representation, _to: Representation) -> Self {
        // Trit is canonical (it is the sum type) — the conversion is the
        // identity at this layer. The named representations only matter
        // for serialisation; see `value_a/value_b/value_c`.
        self
    }
}

// ════════════════════════════════════════════════════════════════════════
// Display — Rep-C numeral
// ════════════════════════════════════════════════════════════════════════

impl fmt::Display for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Rep-C numeral form: {1, 2, 3}
        write!(f, "{}", self.value_c())
    }
}

// ════════════════════════════════════════════════════════════════════════
// Compile-time correctness — I-1 + bijections
// ════════════════════════════════════════════════════════════════════════

const _: () = {
    // I-1 — exactly three symbols
    let _three: [Trit; 3] = [Trit::One, Trit::Two, Trit::Three];

    // Bijections A↔B↔C
    assert!(Trit::One.value_a() == -1 && Trit::One.value_b() == 0 && Trit::One.value_c() == 1);
    assert!(Trit::Two.value_a() == 0 && Trit::Two.value_b() == 1 && Trit::Two.value_c() == 2);
    assert!(Trit::Three.value_a() == 1 && Trit::Three.value_b() == 2 && Trit::Three.value_c() == 3);

    // GF(3) field axioms — additive identity
    assert!(Trit::Two.add(Trit::Two).is_zero());
    assert!(Trit::Three.add(Trit::Two).value_a() == 1);
    // (−1) + 1 = 0
    assert!(Trit::One.add(Trit::Three).is_zero());
    // 1 + 1 = 2 → reduces to −1 in balanced
    assert!(Trit::Three.add(Trit::Three).value_a() == -1);
    // (−1) + (−1) = −2 → reduces to +1
    assert!(Trit::One.add(Trit::One).value_a() == 1);

    // Multiplicative identity
    assert!(Trit::Three.mul(Trit::Three).value_a() == 1);
    assert!(Trit::One.mul(Trit::One).value_a() == 1);
    assert!(Trit::Three.mul(Trit::One).value_a() == -1);
    assert!(Trit::Two.mul(Trit::Three).is_zero());

    // Inverses
    let inv1 = Trit::Three.gf3_inverse();
    let invn1 = Trit::One.gf3_inverse();
    assert!(matches!(inv1, Some(Trit::Three)));
    assert!(matches!(invn1, Some(Trit::One)));
    assert!(matches!(Trit::Two.gf3_inverse(), None));
};
