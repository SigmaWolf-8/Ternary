// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Attestation Liveness Proof — Rolling Merkle Tree
// TIS-27 hashing with domain separation (leaf=0, internal=1)

//! Rolling Merkle tree for heartbeat challenge-response liveness proofs.
//!
//! Hash primitive: TIS-27 (54-trit sponge, 4 rounds, 43-bit integrity).
//! Domain separation: leaf prefix = Rep B trit 0, internal prefix = Rep B trit 1.
//! Empty tree constant: TIS-27(Rep_B_0 ‖ empty_input).

use ternary_math::trit_int::TritInt;

/// TIS-27 output length in bytes (27 bytes = 54 trits / 2).
const TIS27_OUTPUT_LEN: usize = 27;

/// Rep B trit value 0 — leaf node prefix (internal to hash, not wire-transmitted).
const LEAF_PREFIX: u8 = 0;

/// Rep B trit value 1 — internal node prefix.
const INTERNAL_PREFIX: u8 = 1;

/// Domain separator for TIS-27 leaf hashing.
const LEAF_DOMAIN: &[u8] = b"PLENUMNET-ATTEST-MERKLE-LEAF";

/// Domain separator for TIS-27 internal node hashing.
const INTERNAL_DOMAIN: &[u8] = b"PLENUMNET-ATTEST-MERKLE-NODE";

// ═══════════════════════════════════════════════════════════════════════
// ROLLING MERKLE TREE
// ═══════════════════════════════════════════════════════════════════════

/// A rolling Merkle tree accumulating heartbeat challenge-response values.
///
/// Leaves are TritInt values serialized via to_repr_c() before hashing.
/// The root is O(1) constant size; proof-of-inclusion is O(log n).
pub struct RollingMerkleTree {
    /// Leaf hashes (TIS-27 output).
    leaves: Vec<Vec<u8>>,
}

impl RollingMerkleTree {
    /// Create an empty tree.
    pub fn new() -> Self {
        RollingMerkleTree { leaves: Vec::new() }
    }

    /// Add a heartbeat challenge-response as a leaf.
    /// The value is a TritInt serialized to Rep C before hashing (INVARIANT 8).
    pub fn add_leaf(&mut self, value: &TritInt) {
        let repr_c = value.to_repr_c();
        let hash = hash_leaf(&repr_c);
        self.leaves.push(hash);
    }

    /// Compute the Merkle root. O(1) constant-size output.
    /// Returns the empty tree constant if no leaves have been added.
    pub fn root(&self) -> Vec<u8> {
        if self.leaves.is_empty() {
            return empty_tree_constant();
        }
        compute_root(&self.leaves)
    }

    /// Number of leaves in the tree.
    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// Generate a proof-of-inclusion for a specific leaf index.
    /// Returns the sibling hashes needed to reconstruct the root.
    /// O(log n) proof size.
    pub fn proof_of_inclusion(&self, leaf_index: usize) -> Option<Vec<Vec<u8>>> {
        if leaf_index >= self.leaves.len() {
            return None;
        }
        let mut proof = Vec::new();
        let mut level = self.leaves.clone();
        let mut idx = leaf_index;

        while level.len() > 1 {
            // Pad to even length
            if level.len() % 2 == 1 {
                level.push(level.last().unwrap().clone());
            }
            let sibling = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            proof.push(level[sibling].clone());
            // Compute next level
            let mut next = Vec::with_capacity(level.len() / 2);
            for pair in level.chunks(2) {
                next.push(hash_internal(&pair[0], &pair[1]));
            }
            level = next;
            idx /= 2;
        }
        Some(proof)
    }

    /// Reset the tree for a new attestation interval.
    pub fn reset(&mut self) {
        self.leaves.clear();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HASH FUNCTIONS — TIS-27 with domain separation
// ═══════════════════════════════════════════════════════════════════════

/// Hash a leaf node: TIS-27(leaf_prefix ‖ data).
fn hash_leaf(data: &[u8]) -> Vec<u8> {
    let mut input = Vec::with_capacity(1 + data.len());
    input.push(LEAF_PREFIX);
    input.extend_from_slice(data);
    ternary_math::sponge::derive_key(LEAF_DOMAIN, &input, TIS27_OUTPUT_LEN)
}

/// Hash an internal node: TIS-27(internal_prefix ‖ left ‖ right).
fn hash_internal(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut input = Vec::with_capacity(1 + left.len() + right.len());
    input.push(INTERNAL_PREFIX);
    input.extend_from_slice(left);
    input.extend_from_slice(right);
    ternary_math::sponge::derive_key(INTERNAL_DOMAIN, &input, TIS27_OUTPUT_LEN)
}

/// The Merkle root of an empty tree (zero leaves).
/// Deterministic constant: TIS-27(leaf_prefix ‖ empty_input).
pub fn empty_tree_constant() -> Vec<u8> {
    hash_leaf(&[])
}

/// Compute Merkle root from leaf hashes.
fn compute_root(leaves: &[Vec<u8>]) -> Vec<u8> {
    if leaves.len() == 1 {
        return leaves[0].clone();
    }
    let mut level: Vec<Vec<u8>> = leaves.to_vec();
    while level.len() > 1 {
        if level.len() % 2 == 1 {
            level.push(level.last().unwrap().clone());
        }
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks(2) {
            next.push(hash_internal(&pair[0], &pair[1]));
        }
        level = next;
    }
    level.into_iter().next().unwrap()
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_returns_constant() {
        let tree = RollingMerkleTree::new();
        let root = tree.root();
        assert_eq!(root, empty_tree_constant());
        assert_eq!(root.len(), TIS27_OUTPUT_LEN);
    }

    #[test]
    fn single_leaf_root() {
        let mut tree = RollingMerkleTree::new();
        tree.add_leaf(&TritInt::from_host_u64(42));
        let root = tree.root();
        // Single leaf: root = hash_leaf(42_repr_c)
        let expected = hash_leaf(&TritInt::from_host_u64(42).to_repr_c());
        assert_eq!(root, expected);
    }

    #[test]
    fn domain_separation_leaf_vs_internal() {
        let data = vec![1, 2, 3];
        let leaf_hash = hash_leaf(&data);
        // Construct what an internal hash with same data would be
        let internal_hash = hash_internal(&data, &[]);
        assert_ne!(leaf_hash, internal_hash, "leaf and internal hashes must differ for same data");
    }

    #[test]
    fn seventeen_leaves_proof_of_inclusion() {
        let mut tree = RollingMerkleTree::new();
        for i in 0..17u64 {
            tree.add_leaf(&TritInt::from_host_u64(i));
        }
        assert_eq!(tree.leaf_count(), 17);

        // Verify proofs exist for leaf 0, 8, 16
        for idx in [0, 8, 16] {
            let proof = tree.proof_of_inclusion(idx);
            assert!(proof.is_some(), "proof should exist for leaf {idx}");
        }

        // Out of range returns None
        assert!(tree.proof_of_inclusion(17).is_none());
    }

    #[test]
    fn reset_clears_tree() {
        let mut tree = RollingMerkleTree::new();
        tree.add_leaf(&TritInt::from_host_u64(1));
        tree.add_leaf(&TritInt::from_host_u64(2));
        assert_eq!(tree.leaf_count(), 2);

        tree.reset();
        assert_eq!(tree.leaf_count(), 0);
        assert_eq!(tree.root(), empty_tree_constant());
    }

    #[test]
    fn deterministic_root() {
        let mut tree1 = RollingMerkleTree::new();
        let mut tree2 = RollingMerkleTree::new();
        for i in 0..5u64 {
            tree1.add_leaf(&TritInt::from_host_u64(i));
            tree2.add_leaf(&TritInt::from_host_u64(i));
        }
        assert_eq!(tree1.root(), tree2.root());
    }
}
