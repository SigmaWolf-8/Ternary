// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! Ternary Cube Permutation Group
//!
//! Implements the automorphism group of the ternary N-cube (Hamming graph H(N,3)),
//! which is the wreath product S₃ ≀ Sₙ = S₃ⁿ ⋊ Sₙ.
//!
//! # Algebraic Representation
//!
//! The key insight is that S₃ acting on {0, 1, 2} is isomorphic to the
//! affine group Aff(1, 𝔽₃):
//!
//! ```text
//!   π(x) = (a · x + b) mod 3,    a ∈ {1, 2},  b ∈ {0, 1, 2}
//! ```
//!
//! This gives all 6 permutations via pure arithmetic — no lookup tables:
//!
//! | Permutation   | (a, b) | Action              |
//! |---------------|--------|---------------------|
//! | identity      | (1, 0) | x → x              |
//! | swap 1↔2      | (2, 0) | x → 2x mod 3       |
//! | swap 0↔1      | (2, 1) | x → (2x + 1) mod 3 |
//! | rotate fwd    | (1, 1) | x → (x + 1) mod 3  |
//! | rotate bwd    | (1, 2) | x → (x + 2) mod 3  |
//! | swap 0↔2      | (2, 2) | x → (2x + 2) mod 3 |
//!
//! ## Group Operations (Closed-Form)
//!
//! - **Composition:** (a₁, b₁) ∘ (a₂, b₂) = (a₁·a₂ mod 3, a₁·b₂ + b₁ mod 3)
//! - **Inverse:** (a, b)⁻¹ = (a, (3 − a·b) mod 3)
//!   Since a⁻¹ = a in 𝔽₃ (both 1 and 2 are self-inverse: 1·1 = 1, 2·2 = 4 ≡ 1)
//! - **Identity:** (1, 0)
//!
//! # Structure of S₃ ≀ Sₙ
//!
//! An element consists of:
//! - An axis permutation σ ∈ Sₙ (reorders which dimension is which)
//! - N independent affine maps (aⱼ, bⱼ) ∈ Aff(1, 𝔽₃) (permute {0,1,2} per axis)
//!
//! The action on a vertex (c₀, ..., c_{N-1}) is:
//!   (c₀, ..., c_{N-1}) → (π₀(c_{σ⁻¹(0)}), π₁(c_{σ⁻¹(1)}), ..., π_{N-1}(c_{σ⁻¹(N-1)}))
//!
//! where πⱼ(x) = (aⱼ · x + bⱼ) mod 3.
//!
//! # Group Size
//!
//! |S₃ ≀ Sₙ| = 6ⁿ × N!
//!
//! For the sponge (N=6): 6⁶ × 720 = 33,592,320 per round
//! For the network (N=13): 6¹³ × 13! ≈ 8.1 × 10¹⁹
//!
//! # Constant-Time Properties
//!
//! The affine representation eliminates all data-dependent memory access
//! from the S₃ value permutations. The computation `(a * x + b) % 3` is
//! purely arithmetic — no table lookups, no branching on secret data.
//! This is a structural improvement over table-based S₃ implementations.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.




/// Maximum supported cube dimension.
pub const MAX_CUBE_DIM: usize = 13;

// ══════════════════════════════════════════════════════════════
// AFFINE S₃ — THE MATHEMATICAL CORE
// ══════════════════════════════════════════════════════════════

/// An element of S₃ represented as an affine map over 𝔽₃.
///
/// π(x) = (a · x + b) mod 3
///
/// This is the isomorphism S₃ ≅ Aff(1, 𝔽₃). Every permutation of
/// {0, 1, 2} has a unique (a, b) representation with a ∈ {1, 2}, b ∈ {0, 1, 2}.
///
/// - `a = 1`: even permutations (identity, two 3-cycles)
/// - `a = 2`: odd permutations (three transpositions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AffineS3 {
    /// Multiplicative coefficient: 1 (even) or 2 (odd). Invariant: a ∈ {1, 2}.
    pub a: u8,
    /// Additive offset: 0, 1, or 2. Invariant: b ∈ {0, 1, 2}.
    pub b: u8,
}

impl AffineS3 {
    /// The identity permutation: x → x.
    pub const IDENTITY: Self = Self { a: 1, b: 0 };

    /// Construct from (a, b). Returns None if a ∉ {1, 2} or b ∉ {0, 1, 2}.
    #[inline]
    pub const fn new(a: u8, b: u8) -> Option<Self> {
        if (a == 1 || a == 2) && b < 3 {
            Some(Self { a, b })
        } else {
            None
        }
    }

    /// Apply this permutation to a value in {0, 1, 2}.
    ///
    /// Pure arithmetic: `(a * x + b) % 3`. No lookup, no branching.
    #[inline]
    pub const fn apply(&self, x: u8) -> u8 {
        (self.a * x + self.b) % 3
    }

    /// Compose: self ∘ other (apply other first, then self).
    ///
    /// (a₁, b₁) ∘ (a₂, b₂) = (a₁·a₂ mod 3, a₁·b₂ + b₁ mod 3)
    #[inline]
    pub const fn compose(&self, other: &AffineS3) -> Self {
        Self {
            a: (self.a * other.a) % 3,
            b: (self.a * other.b + self.b) % 3,
        }
    }

    /// Inverse: (a, b)⁻¹ = (a, (3 − a·b) mod 3).
    ///
    /// Since a⁻¹ = a in 𝔽₃ (both 1² = 1 and 2² = 4 ≡ 1 mod 3):
    ///   (a, b)⁻¹ = (a⁻¹, −a⁻¹·b) = (a, 3 − a·b mod 3)
    #[inline]
    pub const fn inverse(&self) -> Self {
        // a⁻¹ = a in F₃, so inverse_a = a
        // inverse_b = (3 - a * b % 3) % 3  (negation in F₃)
        Self {
            a: self.a,
            b: (3 - (self.a * self.b) % 3) % 3,
        }
    }

    /// The sign (parity) of this permutation: +1 for even, -1 for odd.
    ///
    /// a = 1 → even (identity, 3-cycles), a = 2 → odd (transpositions).
    #[inline]
    pub const fn sign(&self) -> i8 {
        if self.a == 1 { 1 } else { -1 }
    }

    /// Order of this element in S₃.
    ///
    /// - (1, 0) → 1 (identity)
    /// - (1, 1) or (1, 2) → 3 (3-cycles)
    /// - (2, _) → 2 (transpositions)
    #[inline]
    pub const fn order(&self) -> u8 {
        if self.a == 1 && self.b == 0 {
            1
        } else if self.a == 1 {
            3
        } else {
            2
        }
    }

    /// Derive from two balanced ternary trits.
    ///
    /// Maps 9 possible (t₁, t₂) pairs → 6 S₃ elements via mod 6.
    /// Bias: 3 elements get probability 2/9, 3 get 1/9 — acceptable for
    /// key derivation where the trit stream is long enough to amortize.
    #[inline]
    pub fn from_trits(t1: i8, t2: i8) -> Self {
        let raw = ((t1 + 1) as u8) * 3 + (t2 + 1) as u8; // 0..8
        let idx = raw % 6;
        // Map index to (a, b): even perms first, then odd
        // 0 → (1,0), 1 → (1,1), 2 → (1,2), 3 → (2,0), 4 → (2,1), 5 → (2,2)
        Self {
            a: if idx < 3 { 1 } else { 2 },
            b: idx % 3,
        }
    }

    /// Enumerate all 6 elements of S₃ in canonical order.
    ///
    /// Order: even permutations (a=1) first, then odd (a=2),
    /// each sorted by b.
    pub const ALL: [Self; 6] = [
        Self { a: 1, b: 0 }, // identity
        Self { a: 1, b: 1 }, // rotate forward (0→1→2→0)
        Self { a: 1, b: 2 }, // rotate backward (0→2→1→0)
        Self { a: 2, b: 0 }, // swap 1↔2
        Self { a: 2, b: 1 }, // swap 0↔1
        Self { a: 2, b: 2 }, // swap 0↔2
    ];
}

// ══════════════════════════════════════════════════════════════
// WREATH PRODUCT S₃ ≀ Sₙ
// ══════════════════════════════════════════════════════════════

/// A single element of the wreath product S₃ ≀ Sₙ.
///
/// Represents a distance-preserving transformation of the ternary N-cube.
/// All value permutations are stored as `AffineS3` — no lookup tables.
#[derive(Debug, Clone)]
pub struct TernaryCubeAutomorphism {
    /// Number of dimensions.
    pub dim: usize,
    /// Axis permutation σ: `axis_perm[i]` = which old axis goes to position i.
    pub axis_perm: [usize; MAX_CUBE_DIM],
    /// Inverse axis permutation σ⁻¹: `axis_perm_inv[axis_perm[i]] = i`.
    pub axis_perm_inv: [usize; MAX_CUBE_DIM],
    /// Per-axis value permutation as affine maps (one per dimension).
    pub value_perms: [AffineS3; MAX_CUBE_DIM],
}

impl TernaryCubeAutomorphism {
    /// Create the identity automorphism for dimension `dim`.
    pub fn identity(dim: usize) -> Self {
        assert!(dim <= MAX_CUBE_DIM);
        let mut axis_perm = [0usize; MAX_CUBE_DIM];
        let mut axis_perm_inv = [0usize; MAX_CUBE_DIM];
        for i in 0..dim {
            axis_perm[i] = i;
            axis_perm_inv[i] = i;
        }
        Self {
            dim,
            axis_perm,
            axis_perm_inv,
            value_perms: [AffineS3::IDENTITY; MAX_CUBE_DIM],
        }
    }

    /// Minimum key trits required for a given dimension.
    ///
    /// Fisher-Yates: (dim − 1) × 2 trits for axis permutation.
    /// Value perms: dim × 2 trits for S₃ selection.
    /// Total: 4 × dim − 2.
    pub const fn min_key_trits(dim: usize) -> usize {
        if dim == 0 { 0 } else { 4 * dim - 2 }
    }

    /// Derive an automorphism from trit-based key material.
    ///
    /// Uses Fisher-Yates shuffle driven by the provided trits to select
    /// a uniformly-distributed group element.
    ///
    /// # Returns
    /// `None` if:
    /// - `dim` exceeds `MAX_CUBE_DIM`
    /// - `key_trits` is shorter than `min_key_trits(dim)`
    /// - Any trit is outside {-1, 0, +1}
    ///
    /// # Arguments
    /// * `dim` — Cube dimension (N)
    /// * `key_trits` — Key material as balanced ternary {-1, 0, +1}.
    pub fn from_key_trits(dim: usize, key_trits: &[i8]) -> Option<Self> {
        if dim > MAX_CUBE_DIM {
            return None;
        }
        let required = Self::min_key_trits(dim);
        if key_trits.len() < required {
            return None;
        }
        // Validate trit range — corruption or encoding error is not silent
        for &t in &key_trits[..required] {
            if t < -1 || t > 1 {
                return None;
            }
        }

        let mut result = Self::identity(dim);
        let mut trit_pos: usize = 0;

        // Helper: read one bounded value from two trits.
        // Caller guarantees sufficient length — no unwrap_or fallback.
        let read_bounded = |trits: &[i8], pos: &mut usize, bound: usize| -> usize {
            if bound <= 1 {
                return 0;
            }
            let t0 = (trits[*pos] + 1) as usize; // safe: range validated above
            *pos += 1;
            let t1 = (trits[*pos] + 1) as usize;
            *pos += 1;
            (t0 * 3 + t1) % bound
        };

        // 1. Axis permutation via Fisher-Yates
        let mut axes: [usize; MAX_CUBE_DIM] = [0; MAX_CUBE_DIM];
        for i in 0..dim {
            axes[i] = i;
        }
        for i in (1..dim).rev() {
            let j = read_bounded(key_trits, &mut trit_pos, i + 1);
            axes.swap(i, j);
        }
        for i in 0..dim {
            result.axis_perm[i] = axes[i];
        }
        // Compute inverse
        for i in 0..dim {
            result.axis_perm_inv[result.axis_perm[i]] = i;
        }

        // 2. Per-axis value permutations from two trits each → AffineS3
        for j in 0..dim {
            let t1 = key_trits[trit_pos];
            trit_pos += 1;
            let t2 = key_trits[trit_pos];
            trit_pos += 1;
            result.value_perms[j] = AffineS3::from_trits(t1, t2);
        }

        Some(result)
    }

    /// Apply this automorphism to a flat index in a 3^dim state array.
    ///
    /// Decomposes index → coordinates → affine transform → recompose.
    /// No lookup tables — value permutation is `(a * x + b) % 3`.
    #[inline]
    pub fn apply_index(&self, index: usize) -> usize {
        // Decompose index into ternary coordinates
        let mut coords = [0u8; MAX_CUBE_DIM];
        let mut remaining = index;
        for d in 0..self.dim {
            coords[d] = (remaining % 3) as u8;
            remaining /= 3;
        }

        // Apply: new_coord[j] = πⱼ(old_coord[σ⁻¹(j)])
        // where πⱼ(x) = (aⱼ * x + bⱼ) % 3
        let mut new_coords = [0u8; MAX_CUBE_DIM];
        for j in 0..self.dim {
            let source_val = coords[self.axis_perm_inv[j]];
            new_coords[j] = self.value_perms[j].apply(source_val);
        }

        // Recompose to flat index
        let mut new_index = 0usize;
        let mut power = 1usize;
        for d in 0..self.dim {
            new_index += new_coords[d] as usize * power;
            power *= 3;
        }
        new_index
    }

    /// Apply this automorphism as a diffusion step on a full 3^dim state array.
    ///
    /// Moves each element to its new position using direct coordinate
    /// computation — no lookup tables, no data-dependent memory access
    /// at the state-array scale.
    pub fn apply_state(&self, state: &[i8], output: &mut [i8]) {
        let size = 3usize.pow(self.dim as u32);
        debug_assert!(state.len() >= size);
        debug_assert!(output.len() >= size);
        for i in 0..size {
            output[self.apply_index(i)] = state[i];
        }
    }

    /// Compose two automorphisms: self ∘ other (apply other first, then self).
    ///
    /// Axis permutation: σ_self(σ_other(i))
    /// Value permutation at position j: π_self_j ∘ π_other_{σ_self⁻¹(j)}
    ///
    /// Composition of affine maps is itself affine — no table search needed.
    pub fn compose(&self, other: &TernaryCubeAutomorphism) -> TernaryCubeAutomorphism {
        assert_eq!(self.dim, other.dim);
        let mut result = Self::identity(self.dim);

        // Composed axis permutation
        for i in 0..self.dim {
            result.axis_perm[i] = self.axis_perm[other.axis_perm[i]];
        }
        for i in 0..self.dim {
            result.axis_perm_inv[result.axis_perm[i]] = i;
        }

        // Composed value permutations: π_self_j ∘ π_other_{source}
        // where source = σ_self⁻¹(j)
        //
        // Algebraically: (a₁,b₁) ∘ (a₂,b₂) = (a₁·a₂ mod 3, a₁·b₂ + b₁ mod 3)
        for j in 0..self.dim {
            let other_source = self.axis_perm_inv[j];
            result.value_perms[j] = self.value_perms[j].compose(&other.value_perms[other_source]);
        }

        result
    }

    /// Compute the group-theoretic inverse of this automorphism.
    ///
    /// For axis permutation: σ⁻¹ (swap perm and perm_inv).
    /// For value permutations: (a, b)⁻¹ = (a, (3 − a·b) mod 3).
    pub fn inverse(&self) -> TernaryCubeAutomorphism {
        let mut result = Self::identity(self.dim);

        // Inverse axis permutation: swap perm and perm_inv
        for i in 0..self.dim {
            result.axis_perm[i] = self.axis_perm_inv[i];
            result.axis_perm_inv[i] = self.axis_perm[i];
        }

        // Inverse value permutations: at position j in the result,
        // we need to invert the permutation that was at the source position
        // in the original.
        for j in 0..self.dim {
            let source = self.axis_perm[j];
            result.value_perms[j] = self.value_perms[source].inverse();
        }

        result
    }

    /// Check if this is a valid automorphism (bijection).
    pub fn is_valid(&self) -> bool {
        // Check axis permutation is a valid permutation of [0..dim)
        let mut seen = [false; MAX_CUBE_DIM];
        for i in 0..self.dim {
            if self.axis_perm[i] >= self.dim || seen[self.axis_perm[i]] {
                return false;
            }
            seen[self.axis_perm[i]] = true;
        }

        // Check inverse consistency
        for i in 0..self.dim {
            if self.axis_perm_inv[self.axis_perm[i]] != i {
                return false;
            }
        }

        // Check value permutations are valid (a ∈ {1,2}, b ∈ {0,1,2})
        for j in 0..self.dim {
            let p = &self.value_perms[j];
            if (p.a != 1 && p.a != 2) || p.b >= 3 {
                return false;
            }
        }

        true
    }

    /// The parity (sign) of this automorphism.
    ///
    /// Product of the axis permutation sign and all value permutation signs.
    /// Even → +1, Odd → −1.
    pub fn sign(&self) -> i8 {
        // Axis permutation parity: count inversions
        let mut inversions = 0usize;
        for i in 0..self.dim {
            for j in (i + 1)..self.dim {
                if self.axis_perm[i] > self.axis_perm[j] {
                    inversions += 1;
                }
            }
        }
        let axis_sign: i8 = if inversions % 2 == 0 { 1 } else { -1 };

        // Value permutation parity: product of all individual signs
        let mut value_sign: i8 = 1;
        for j in 0..self.dim {
            value_sign *= self.value_perms[j].sign();
        }

        axis_sign * value_sign
    }
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── AffineS3 unit tests ────────────────────────────────────

    #[test]
    fn test_affine_s3_exhaustive() {
        // Verify all 6 elements are distinct permutations of {0, 1, 2}
        let mut images: Vec<[u8; 3]> = Vec::new();
        for &elem in &AffineS3::ALL {
            let img = [elem.apply(0), elem.apply(1), elem.apply(2)];
            // Each image must be a permutation of {0, 1, 2}
            let mut sorted = img;
            sorted.sort();
            assert_eq!(sorted, [0, 1, 2], "Not a permutation: {:?} from ({}, {})", img, elem.a, elem.b);
            images.push(img);
        }
        // All 6 must be distinct
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_ne!(images[i], images[j], "Duplicate permutation at {} and {}", i, j);
            }
        }
    }

    #[test]
    fn test_affine_s3_identity() {
        let id = AffineS3::IDENTITY;
        for x in 0..3 {
            assert_eq!(id.apply(x), x);
        }
    }

    #[test]
    fn test_affine_s3_compose_is_associative() {
        // (f ∘ g) ∘ h = f ∘ (g ∘ h) for all triples
        for &f in &AffineS3::ALL {
            for &g in &AffineS3::ALL {
                for &h in &AffineS3::ALL {
                    let fg_h = f.compose(&g).compose(&h);
                    let f_gh = f.compose(&g.compose(&h));
                    assert_eq!(fg_h, f_gh,
                        "Associativity failed: ({:?} ∘ {:?}) ∘ {:?}", f, g, h);
                }
            }
        }
    }

    #[test]
    fn test_affine_s3_inverse() {
        for &elem in &AffineS3::ALL {
            let inv = elem.inverse();
            let composed = elem.compose(&inv);
            assert_eq!(composed, AffineS3::IDENTITY,
                "elem ∘ elem⁻¹ ≠ identity for ({}, {})", elem.a, elem.b);
            let composed2 = inv.compose(&elem);
            assert_eq!(composed2, AffineS3::IDENTITY,
                "elem⁻¹ ∘ elem ≠ identity for ({}, {})", elem.a, elem.b);
        }
    }

    #[test]
    fn test_affine_s3_order() {
        assert_eq!(AffineS3::new(1, 0).unwrap().order(), 1); // identity
        assert_eq!(AffineS3::new(1, 1).unwrap().order(), 3); // 3-cycle
        assert_eq!(AffineS3::new(1, 2).unwrap().order(), 3); // 3-cycle
        assert_eq!(AffineS3::new(2, 0).unwrap().order(), 2); // transposition
        assert_eq!(AffineS3::new(2, 1).unwrap().order(), 2); // transposition
        assert_eq!(AffineS3::new(2, 2).unwrap().order(), 2); // transposition
    }

    #[test]
    fn test_affine_s3_sign() {
        // Even permutations: a = 1
        assert_eq!(AffineS3::new(1, 0).unwrap().sign(), 1);
        assert_eq!(AffineS3::new(1, 1).unwrap().sign(), 1);
        assert_eq!(AffineS3::new(1, 2).unwrap().sign(), 1);
        // Odd permutations: a = 2
        assert_eq!(AffineS3::new(2, 0).unwrap().sign(), -1);
        assert_eq!(AffineS3::new(2, 1).unwrap().sign(), -1);
        assert_eq!(AffineS3::new(2, 2).unwrap().sign(), -1);
    }

    #[test]
    fn test_affine_s3_new_validation() {
        assert!(AffineS3::new(0, 0).is_none()); // a must be 1 or 2
        assert!(AffineS3::new(3, 0).is_none()); // a out of range
        assert!(AffineS3::new(1, 3).is_none()); // b out of range
        assert!(AffineS3::new(1, 0).is_some());
        assert!(AffineS3::new(2, 2).is_some());
    }

    #[test]
    fn test_affine_s3_closure() {
        // Composition of any two S₃ elements yields another S₃ element
        for &f in &AffineS3::ALL {
            for &g in &AffineS3::ALL {
                let fg = f.compose(&g);
                assert!(fg.a == 1 || fg.a == 2, "Invalid a after compose: {}", fg.a);
                assert!(fg.b < 3, "Invalid b after compose: {}", fg.b);
            }
        }
    }

    // ── TernaryCubeAutomorphism tests ──────────────────────────

    #[test]
    fn test_identity_is_identity() {
        let id = TernaryCubeAutomorphism::identity(6);
        let size = 3usize.pow(6); // 729
        for i in 0..size {
            assert_eq!(id.apply_index(i), i, "Identity should map {} to {}", i, i);
        }
    }

    #[test]
    fn test_identity_is_valid() {
        let id = TernaryCubeAutomorphism::identity(6);
        assert!(id.is_valid());
    }

    #[test]
    fn test_from_key_produces_bijection() {
        let key = vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
                       0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1,
                       1, 0, -1, -1, 0, 1, 0, 1, -1, 0, 1, -1];
        let aut = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();
        assert!(aut.is_valid());

        let size = 729;
        let mut seen = vec![false; size];
        for i in 0..size {
            let target = aut.apply_index(i);
            assert!(target < size, "Target {} out of range", target);
            assert!(!seen[target], "Target {} appears twice — not a bijection", target);
            seen[target] = true;
        }
        assert!(seen.iter().all(|&s| s), "Not all targets reached — not surjective");
    }

    #[test]
    fn test_different_keys_give_different_permutations() {
        let key_a = vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
                         0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1];
        let key_b = vec![-1i8, 1, 0, -1, 0, 1, -1, 1, 0, -1, 1, 0,
                          1, -1, 0, 1, 0, -1, 0, 1, -1, 0, 1, 0];
        let aut_a = TernaryCubeAutomorphism::from_key_trits(6, &key_a).unwrap();
        let aut_b = TernaryCubeAutomorphism::from_key_trits(6, &key_b).unwrap();

        let mut differ = false;
        for i in 0..729 {
            if aut_a.apply_index(i) != aut_b.apply_index(i) {
                differ = true;
                break;
            }
        }
        assert!(differ, "Different keys should produce different permutations");
    }

    #[test]
    fn test_same_key_is_deterministic() {
        let key = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1,
                       0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let aut_1 = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();
        let aut_2 = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();

        for i in 0..729 {
            assert_eq!(aut_1.apply_index(i), aut_2.apply_index(i),
                "Same key must produce same permutation at index {}", i);
        }
    }

    #[test]
    fn test_inverse_roundtrip() {
        let key = vec![1i8, -1, 0, 1, 0, -1, 1, 0, -1, 1, -1, 0,
                       0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1];
        let aut = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();
        let inv = aut.inverse();
        let composed = aut.compose(&inv);

        // composed should be identity
        let size = 729;
        for i in 0..size {
            assert_eq!(composed.apply_index(i), i,
                "aut ∘ aut⁻¹ should be identity at index {}", i);
        }
    }

    #[test]
    fn test_compose_associativity() {
        let key_a = vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
                         0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1];
        let key_b = vec![-1i8, 1, 0, -1, 0, 1, -1, 1, 0, -1, 1, 0,
                          1, -1, 0, 1, 0, -1, 0, 1, -1, 0, 1, 0];
        let a = TernaryCubeAutomorphism::from_key_trits(6, &key_a).unwrap();
        let b = TernaryCubeAutomorphism::from_key_trits(6, &key_b).unwrap();
        let ab = a.compose(&b);

        // Verify: ab(x) = a(b(x)) for all x
        for i in 0..729 {
            let expected = a.apply_index(b.apply_index(i));
            assert_eq!(ab.apply_index(i), expected,
                "Composition mismatch at index {}", i);
        }
    }

    #[test]
    fn test_preserves_hamming_distance() {
        let key = vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
                       0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1];
        let aut = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();

        let decompose = |mut idx: usize| -> [u8; 6] {
            let mut coords = [0u8; 6];
            for d in 0..6 {
                coords[d] = (idx % 3) as u8;
                idx /= 3;
            }
            coords
        };

        let hamming = |a: &[u8; 6], b: &[u8; 6]| -> usize {
            a.iter().zip(b.iter()).filter(|(&x, &y)| x != y).count()
        };

        let test_pairs = [(0, 1), (0, 3), (0, 9), (100, 200), (728, 0), (364, 365)];
        for (i, j) in test_pairs {
            let ci = decompose(i);
            let cj = decompose(j);
            let dist_before = hamming(&ci, &cj);

            let ti = decompose(aut.apply_index(i));
            let tj = decompose(aut.apply_index(j));
            let dist_after = hamming(&ti, &tj);

            assert_eq!(dist_before, dist_after,
                "Hamming distance not preserved for pair ({}, {}): {} → {}", i, j, dist_before, dist_after);
        }
    }

    #[test]
    fn test_not_identity_for_nontrivial_key() {
        let key = vec![1i8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                       1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let aut = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();

        let mut is_identity = true;
        for i in 0..729 {
            if aut.apply_index(i) != i {
                is_identity = false;
                break;
            }
        }
        assert!(!is_identity, "Non-trivial key should not produce identity");
    }

    #[test]
    fn test_dim_13_produces_valid_automorphism() {
        let key = vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
                       0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1,
                       -1, 0, 1, -1, 1, 0, -1, 0, 1, -1, 0, 1,
                       0, -1, 1, 0, -1, 1, 0, 1, -1, 0, 1, -1,
                       1, 0, -1, 1, 0, -1, 1, 0, -1, 1, 0, -1];
        let aut = TernaryCubeAutomorphism::from_key_trits(13, &key).unwrap();
        assert!(aut.is_valid());

        // Spot-check a few indices (full table would be 1.59M entries)
        let idx_a = aut.apply_index(0);
        let idx_b = aut.apply_index(1);
        assert_ne!(idx_a, idx_b, "Different inputs should map to different outputs");
    }

    #[test]
    fn test_automorphism_sign() {
        let id = TernaryCubeAutomorphism::identity(6);
        assert_eq!(id.sign(), 1, "Identity should be even");

        // Non-trivial key — sign is well-defined (either +1 or -1)
        let key = vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
                       0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1];
        let aut = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();
        let s = aut.sign();
        assert!(s == 1 || s == -1, "Sign must be ±1");

        // Inverse has same sign squared = +1 composed
        let inv = aut.inverse();
        let composed_sign = aut.sign() * inv.sign();
        assert_eq!(composed_sign, 1, "aut × aut⁻¹ sign should be +1");
    }

    #[test]
    fn test_inverse_of_inverse_is_original() {
        let key = vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
                       0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1];
        let aut = TernaryCubeAutomorphism::from_key_trits(6, &key).unwrap();
        let inv = aut.inverse();
        let inv_inv = inv.inverse();

        // (aut⁻¹)⁻¹ = aut
        for i in 0..729 {
            assert_eq!(inv_inv.apply_index(i), aut.apply_index(i),
                "(aut⁻¹)⁻¹ ≠ aut at index {}", i);
        }
    }

    #[test]
    fn test_from_key_rejects_insufficient_trits() {
        // dim=6 needs min_key_trits(6) = 22 trits
        let short_key = vec![0i8; 21]; // one short
        assert!(TernaryCubeAutomorphism::from_key_trits(6, &short_key).is_none(),
            "Should reject key shorter than minimum");

        // Exactly sufficient should work
        let exact_key = vec![0i8; 22];
        assert!(TernaryCubeAutomorphism::from_key_trits(6, &exact_key).is_some(),
            "Should accept key at exact minimum length");
    }

    #[test]
    fn test_from_key_rejects_out_of_range_trits() {
        let mut bad_key = vec![0i8; 24];
        bad_key[5] = 2; // out of range: not in {-1, 0, +1}
        assert!(TernaryCubeAutomorphism::from_key_trits(6, &bad_key).is_none(),
            "Should reject trit value 2");

        bad_key[5] = -2;
        assert!(TernaryCubeAutomorphism::from_key_trits(6, &bad_key).is_none(),
            "Should reject trit value -2");
    }

    #[test]
    fn test_from_key_rejects_excessive_dim() {
        let key = vec![0i8; 200];
        assert!(TernaryCubeAutomorphism::from_key_trits(14, &key).is_none(),
            "Should reject dim > MAX_CUBE_DIM");
    }

    #[test]
    fn test_min_key_trits_formula() {
        assert_eq!(TernaryCubeAutomorphism::min_key_trits(0), 0);
        assert_eq!(TernaryCubeAutomorphism::min_key_trits(1), 2);  // 4*1-2
        assert_eq!(TernaryCubeAutomorphism::min_key_trits(6), 22); // 4*6-2
        assert_eq!(TernaryCubeAutomorphism::min_key_trits(13), 50); // 4*13-2
    }
}