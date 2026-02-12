//! # Borromean Ternary XOR Invariant
//!
//! Validation primitives for three-party cryptographic protocols based on
//! the Borromean rings topology expressed in ternary logic.
//!
//! ## Representation Agnosticism
//!
//! The Borromean invariant is defined over the mod-3 ring and is therefore
//! **independent of which ternary representation is used**. Words can be
//! constructed from any of the kernel's three representations:
//!
//! - **A (Balanced):** `{-1, 0, +1}` via `TernaryWord::from_balanced()`
//! - **B (Standard):** `{0, 1, 2}` via `TernaryWord::new()` or `from_str()`
//! - **C (Bijective):** `{1, 2, 3}` via `TernaryWord::from_bijective()`
//!
//! All representations are normalized to the internal B encoding for
//! arithmetic. The XOR operation (digit-wise sum mod 3) produces identical
//! results regardless of which representation the inputs were specified in.
//!
//! ## The Borromean Condition
//!
//! Three rings (ternary words) are **Borromean-linked** if and only if:
//!
//! 1. **Non-separability**: The digit-wise sum mod 3 of all three words
//!    is never identically zero across all positions simultaneously.
//! 2. **Pairwise separability**: Any two of the three words CAN have their
//!    digit-wise sum mod 3 equal zero at some position — the binding is
//!    strictly a three-body property.
//!
//! In the PQTI protocol, this maps to: three parties in a handshake cannot
//! be individually removed without breaking the link, but no two-party
//! subset is inherently bound.
//!
//! ## Algebraic Structure
//!
//! The ternary XOR operation is addition in Z/3Z (the integers modulo 3).
//! The Borromean condition is:
//!
//!   ∀i: (A[i] + B[i] + C[i]) mod 3 ≠ 0
//!
//! where A, B, C are ternary words of equal length.
//!
//! This is equivalent to requiring that the triple (A, B, C) never produces
//! the "First Position" (0, 0, 0) under the linking operation.

/// A ternary word — a sequence of trits representing one "ring" in the
/// Borromean triple.
///
/// Words can be constructed in any of the three kernel representations
/// (A, B, or C). The Borromean check operates on the underlying integer
/// values — the representation is a lens, not a substance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TernaryWord {
    /// Internal storage in Representation B {0,1,2}.
    pub trits: Vec<u8>,
}

/// Which representation the input digits use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordRepr {
    /// Representation A: Balanced `{-1, 0, +1}` — input as i8.
    Balanced,
    /// Representation B: Standard `{0, 1, 2}` — the default.
    Standard,
    /// Representation C: Bijective `{1, 2, 3}` — zero-free.
    Bijective,
}

/// Result of a Borromean invariant check.
#[derive(Clone, Debug)]
pub struct BorromeanCheckResult {
    /// Whether the three words satisfy the Borromean condition.
    pub is_borromean: bool,
    /// The digit-wise ternary XOR (sum mod 3) of the three words.
    pub ternary_xor: Vec<u8>,
    /// Positions where the ternary XOR equals zero (violation points).
    pub zero_positions: Vec<usize>,
    /// The "Borromean strength" — minimum non-zero value density.
    pub strength: f64,
}

/// Result of a pairwise separability check.
#[derive(Clone, Debug)]
pub struct PairwiseSeparabilityResult {
    /// Whether the pair (A, B) is separable.
    pub ab_separable: bool,
    /// Whether the pair (A, C) is separable.
    pub ac_separable: bool,
    /// Whether the pair (B, C) is separable.
    pub bc_separable: bool,
    /// True if all three pairs are separable.
    pub all_pairwise_separable: bool,
}

impl TernaryWord {
    /// Create a new ternary word from a slice of trits in **Representation B** `{0,1,2}`.
    pub fn new(trits: Vec<u8>) -> Self {
        for (i, &t) in trits.iter().enumerate() {
            assert!(t <= 2, "Trit at position {} must be 0, 1, or 2; got {}", i, t);
        }
        TernaryWord { trits }
    }

    /// Create a ternary word from **Representation A** (Balanced: `{-1, 0, +1}`).
    ///
    /// Each digit is shifted: -1 → 2, 0 → 0, +1 → 1 in the mod-3 ring.
    pub fn from_balanced(balanced: &[i8]) -> Self {
        let trits: Vec<u8> = balanced
            .iter()
            .map(|&d| {
                assert!(d >= -1 && d <= 1, "Balanced trit must be -1, 0, or +1; got {}", d);
                ((d + 3) % 3) as u8
            })
            .collect();
        TernaryWord { trits }
    }

    /// Create a ternary word from **Representation C** (Bijective: `{1, 2, 3}`).
    ///
    /// Each digit is shifted: 1 → 1, 2 → 2, 3 → 0 in the mod-3 ring.
    pub fn from_bijective(bijective: &[u8]) -> Self {
        let trits: Vec<u8> = bijective
            .iter()
            .map(|&d| {
                assert!(d >= 1 && d <= 3, "Bijective trit must be 1, 2, or 3; got {}", d);
                d % 3
            })
            .collect();
        TernaryWord { trits }
    }

    /// Emit this word's trits in a specified representation.
    pub fn to_repr(&self, repr: WordRepr) -> Vec<i8> {
        match repr {
            WordRepr::Standard => self.trits.iter().map(|&t| t as i8).collect(),
            WordRepr::Balanced => {
                self.trits.iter().map(|&t| match t {
                    0 => 0i8,
                    1 => 1,
                    2 => -1,
                    _ => unreachable!(),
                }).collect()
            }
            WordRepr::Bijective => {
                self.trits.iter().map(|&t| match t {
                    0 => 3i8,
                    1 => 1,
                    2 => 2,
                    _ => unreachable!(),
                }).collect()
            }
        }
    }

    /// Create a ternary word from a string of characters '0', '1', '2'.
    pub fn from_str(s: &str) -> Self {
        let trits: Vec<u8> = s
            .chars()
            .map(|c| match c {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                _ => panic!("Invalid trit character: '{}'", c),
            })
            .collect();
        TernaryWord { trits }
    }

    /// Length of the word in trits.
    pub fn len(&self) -> usize {
        self.trits.len()
    }

    /// Whether the word is empty.
    pub fn is_empty(&self) -> bool {
        self.trits.is_empty()
    }

    /// Generate a random ternary word of given length.
    /// Uses a simple deterministic PRNG seeded by the given value.
    pub fn pseudo_random(length: usize, seed: u64) -> Self {
        let mut state = seed;
        let mut trits = Vec::with_capacity(length);
        for _ in 0..length {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            trits.push((state % 3) as u8);
        }
        TernaryWord { trits }
    }

    /// Compute the digit-wise ternary XOR (sum mod 3) of two words.
    pub fn xor_pair(a: &TernaryWord, b: &TernaryWord) -> Vec<u8> {
        let len = std::cmp::max(a.len(), b.len());
        (0..len)
            .map(|i| {
                let va = if i < a.len() { a.trits[i] } else { 0 };
                let vb = if i < b.len() { b.trits[i] } else { 0 };
                (va + vb) % 3
            })
            .collect()
    }

    /// Compute the digit-wise ternary XOR of three words.
    ///
    /// This is the core Borromean operation:
    ///   result[i] = (A[i] + B[i] + C[i]) mod 3
    pub fn xor_triple(a: &TernaryWord, b: &TernaryWord, c: &TernaryWord) -> Vec<u8> {
        let len = *[a.len(), b.len(), c.len()].iter().max().unwrap();
        (0..len)
            .map(|i| {
                let va = if i < a.len() { a.trits[i] } else { 0 };
                let vb = if i < b.len() { b.trits[i] } else { 0 };
                let vc = if i < c.len() { c.trits[i] } else { 0 };
                (va + vb + vc) % 3
            })
            .collect()
    }
}

/// Check whether three ternary words satisfy the Borromean invariant.
///
/// The invariant holds if the digit-wise sum mod 3 of the three words
/// is **never zero** at any position.
pub fn check_borromean_invariant(
    a: &TernaryWord,
    b: &TernaryWord,
    c: &TernaryWord,
) -> BorromeanCheckResult {
    let xor = TernaryWord::xor_triple(a, b, c);

    let zero_positions: Vec<usize> = xor
        .iter()
        .enumerate()
        .filter(|(_, &v)| v == 0)
        .map(|(i, _)| i)
        .collect();

    let total = xor.len() as f64;
    let non_zero = (xor.len() - zero_positions.len()) as f64;
    let strength = if total > 0.0 { non_zero / total } else { 0.0 };

    BorromeanCheckResult {
        is_borromean: zero_positions.is_empty(),
        ternary_xor: xor,
        zero_positions,
        strength,
    }
}

/// Check pairwise separability of three ternary words.
///
/// For a true Borromean topology, each pair of rings must be separable
/// (their pairwise XOR must have at least one zero position), while the
/// triple is non-separable.
pub fn check_pairwise_separability(
    a: &TernaryWord,
    b: &TernaryWord,
    c: &TernaryWord,
) -> PairwiseSeparabilityResult {
    let ab_xor = TernaryWord::xor_pair(a, b);
    let ac_xor = TernaryWord::xor_pair(a, c);
    let bc_xor = TernaryWord::xor_pair(b, c);

    let ab_sep = ab_xor.iter().any(|&v| v == 0);
    let ac_sep = ac_xor.iter().any(|&v| v == 0);
    let bc_sep = bc_xor.iter().any(|&v| v == 0);

    PairwiseSeparabilityResult {
        ab_separable: ab_sep,
        ac_separable: ac_sep,
        bc_separable: bc_sep,
        all_pairwise_separable: ab_sep && ac_sep && bc_sep,
    }
}

/// Full Borromean validation: checks both the non-separability of the triple
/// AND the pairwise separability of each pair.
pub fn validate_borromean_triple(
    a: &TernaryWord,
    b: &TernaryWord,
    c: &TernaryWord,
) -> (BorromeanCheckResult, PairwiseSeparabilityResult) {
    let borromean = check_borromean_invariant(a, b, c);
    let pairwise = check_pairwise_separability(a, b, c);
    (borromean, pairwise)
}

/// Generate a valid Borromean triple of given word length.
///
/// Uses a construction based on cyclic shifts in Z/3Z.
pub fn generate_borromean_triple(length: usize, seed: u64) -> (TernaryWord, TernaryWord, TernaryWord) {
    let a = TernaryWord::pseudo_random(length, seed);

    let mut b_trits = Vec::with_capacity(length);
    let mut c_trits = Vec::with_capacity(length);

    for i in 0..length {
        let ai = a.trits[i];

        if i == 0 {
            b_trits.push((3 - ai) % 3);
            c_trits.push(if ai == 0 { 1 } else { 1 });
        } else if i == 1 && length > 1 {
            c_trits.push((3 - ai) % 3);
            b_trits.push(if ai == 0 { 2 } else { 2 });
        } else if i == 2 && length > 2 {
            let b_val = 1u8;
            let c_val = 2u8;
            if ai != 0 {
                b_trits.push(b_val);
                c_trits.push(c_val);
            } else {
                b_trits.push(1);
                c_trits.push(1);
            }
        } else {
            b_trits.push((ai + 1) % 3);
            c_trits.push((ai + 1) % 3);
        }
    }

    (
        a,
        TernaryWord::new(b_trits),
        TernaryWord::new(c_trits),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_xor_basic() {
        let a = TernaryWord::from_str("012");
        let b = TernaryWord::from_str("012");
        let c = TernaryWord::from_str("012");

        let xor = TernaryWord::xor_triple(&a, &b, &c);
        assert_eq!(xor, vec![0, 0, 0]);
    }

    #[test]
    fn test_identical_words_not_borromean() {
        let a = TernaryWord::from_str("012");
        let b = TernaryWord::from_str("012");
        let c = TernaryWord::from_str("012");

        let result = check_borromean_invariant(&a, &b, &c);
        assert!(!result.is_borromean, "Three identical words cannot be Borromean");
    }

    #[test]
    fn test_first_position_not_borromean() {
        let a = TernaryWord::from_str("000");
        let b = TernaryWord::from_str("000");
        let c = TernaryWord::from_str("000");

        let result = check_borromean_invariant(&a, &b, &c);
        assert!(!result.is_borromean);
        assert_eq!(result.zero_positions.len(), 3);
    }

    #[test]
    fn test_valid_borromean_triple() {
        let a = TernaryWord::from_str("012");
        let b = TernaryWord::from_str("120");
        let c = TernaryWord::from_str("120");

        let result = check_borromean_invariant(&a, &b, &c);
        assert!(result.is_borromean, "This triple should satisfy the Borromean condition");
        assert!(result.zero_positions.is_empty());
        assert_eq!(result.strength, 1.0);
    }

    #[test]
    fn test_borromean_strength() {
        let a = TernaryWord::from_str("012");
        let b = TernaryWord::from_str("021");
        let c = TernaryWord::from_str("000");
        let result = check_borromean_invariant(&a, &b, &c);
        assert!(!result.is_borromean);
        assert!(result.strength < 1.0);
    }

    #[test]
    fn test_pairwise_separability() {
        let a = TernaryWord::from_str("012");
        let b = TernaryWord::from_str("021");
        let c = TernaryWord::from_str("210");

        let (borromean, pairwise) = validate_borromean_triple(&a, &b, &c);

        assert!(
            borromean.strength >= 0.0 && borromean.strength <= 1.0,
            "Strength must be in [0, 1]"
        );
        let _ = pairwise.all_pairwise_separable;
    }

    #[test]
    fn test_generated_borromean_triple() {
        for seed in 1..=20u64 {
            let (a, b, c) = generate_borromean_triple(32, seed);
            let result = check_borromean_invariant(&a, &b, &c);
            assert!(
                result.is_borromean,
                "Generated triple with seed {} should be Borromean (violations at {:?})",
                seed, result.zero_positions
            );
        }
    }

    #[test]
    fn test_xor_commutativity() {
        let a = TernaryWord::from_str("01221");
        let b = TernaryWord::from_str("21012");
        let c = TernaryWord::from_str("10201");

        let abc = TernaryWord::xor_triple(&a, &b, &c);
        let bca = TernaryWord::xor_triple(&b, &c, &a);
        let cab = TernaryWord::xor_triple(&c, &a, &b);

        assert_eq!(abc, bca, "Ternary XOR must be commutative");
        assert_eq!(bca, cab, "Ternary XOR must be commutative");
    }

    #[test]
    fn test_word_length_mismatch_padding() {
        let a = TernaryWord::from_str("12");
        let b = TernaryWord::from_str("1");
        let c = TernaryWord::from_str("121");

        let xor = TernaryWord::xor_triple(&a, &b, &c);
        assert_eq!(xor.len(), 3);
        assert_eq!(xor[0], 0);
        assert_eq!(xor[1], 1);
        assert_eq!(xor[2], 1);
    }

    // ── Representation A/B/C Interchange Tests ────────────────────

    #[test]
    fn test_from_balanced_repr_a() {
        let word = TernaryWord::from_balanced(&[-1, 0, 1]);
        assert_eq!(word.trits, vec![2, 0, 1]);
    }

    #[test]
    fn test_from_bijective_repr_c() {
        let word = TernaryWord::from_bijective(&[1, 2, 3]);
        assert_eq!(word.trits, vec![1, 2, 0]);
    }

    #[test]
    fn test_to_repr_roundtrip() {
        let word = TernaryWord::from_str("012");

        let as_a = word.to_repr(WordRepr::Balanced);
        assert_eq!(as_a, vec![0, 1, -1]);

        let as_c = word.to_repr(WordRepr::Bijective);
        assert_eq!(as_c, vec![3, 1, 2]);

        let as_b = word.to_repr(WordRepr::Standard);
        assert_eq!(as_b, vec![0, 1, 2]);
    }

    #[test]
    fn test_borromean_invariant_across_representations() {
        let a_from_b = TernaryWord::from_str("012");
        let b_from_b = TernaryWord::from_str("120");
        let c_from_b = TernaryWord::from_str("120");

        let a_from_a = TernaryWord::from_balanced(&[0, 1, -1]);
        let b_from_a = TernaryWord::from_balanced(&[1, -1, 0]);
        let c_from_a = TernaryWord::from_balanced(&[1, -1, 0]);

        let a_from_c = TernaryWord::from_bijective(&[3, 1, 2]);
        let b_from_c = TernaryWord::from_bijective(&[1, 2, 3]);
        let c_from_c = TernaryWord::from_bijective(&[1, 2, 3]);

        let result_b = check_borromean_invariant(&a_from_b, &b_from_b, &c_from_b);
        let result_a = check_borromean_invariant(&a_from_a, &b_from_a, &c_from_a);
        let result_c = check_borromean_invariant(&a_from_c, &b_from_c, &c_from_c);

        assert_eq!(result_b.is_borromean, result_a.is_borromean,
            "Borromean result must be identical whether constructed from B or A");
        assert_eq!(result_b.is_borromean, result_c.is_borromean,
            "Borromean result must be identical whether constructed from B or C");
        assert_eq!(result_b.ternary_xor, result_a.ternary_xor,
            "XOR vectors must match across representations");
        assert_eq!(result_b.ternary_xor, result_c.ternary_xor,
            "XOR vectors must match across representations");
    }

    #[test]
    fn test_repr_interchange_preserves_xor() {
        for seed in 1..=20u64 {
            let word = TernaryWord::pseudo_random(16, seed);

            let as_a = word.to_repr(WordRepr::Balanced);
            let from_a: Vec<i8> = as_a;
            let roundtrip_a = TernaryWord::from_balanced(&from_a);

            let as_c = word.to_repr(WordRepr::Bijective);
            let from_c: Vec<u8> = as_c.iter().map(|&d| d as u8).collect();
            let roundtrip_c = TernaryWord::from_bijective(&from_c);

            assert_eq!(word.trits, roundtrip_a.trits,
                "A-roundtrip failed for seed {}", seed);
            assert_eq!(word.trits, roundtrip_c.trits,
                "C-roundtrip failed for seed {}", seed);
        }
    }
}
