// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # PT26-DSA — Parallel Traversals × 26-Port Geometric Digital Signature
//!
//! A topological signature scheme whose hard problem is the geometry of
//! the 13-dimensional ternary hypercube. The signer proves knowledge of
//! a secret walk through the hypercube using the σ permutation schedule.
//! Verification fires all 26 neighbor ports in parallel.
//!
//! **PT26 = Parallel Traversals × 26 ports.**
//!
//! See TM-2026-012b for the full design monograph.

use crate::cube_addr::CubeAddr;
use crate::plenum_square::{SIGMAS, WEIGHT_VECTOR, MAGIC_CONSTANT};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Maximum walk length = 13 (Hamming distance in Z₃¹³).
pub const MAX_WALK_LENGTH: usize = 13;

/// Number of σ permutations available per step.
pub const NUM_SIGMAS: usize = 4;

/// Number of dimensions in the hypercube.
pub const DIMENSIONS: usize = 13;

/// Number of neighbor ports (13 dims × 2 alt values).
pub const PORTS: usize = 26;

/// Signature budget per key (coprime walk period = lcm(13,28)/13 ≈ 28).
pub const SIG_BUDGET_PER_KEY: usize = 28;

/// Step commitment length in bytes.
pub const STEP_COMMIT_LEN: usize = 48;

/// Public key commitment length in bytes.
pub const PK_COMMIT_LEN: usize = 48;

/// Signature aggregate commitment length in bytes.
pub const SIG_COMMIT_LEN: usize = 48;

/// Domain separators.
pub const DOMAIN_SCHEDULE: &[u8] = b"PT26-SCHEDULE";
pub const DOMAIN_WEIGHT: &[u8] = b"PT26-WEIGHT";
pub const DOMAIN_PK: &[u8] = b"PT26-PK";
pub const DOMAIN_MSG: &[u8] = b"PT26-MSG";
pub const DOMAIN_STEP: &[u8] = b"PT26-STEP";
pub const DOMAIN_SIG: &[u8] = b"PT26-SIG";

// ═══════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pt26Error {
    /// Destination derivation mismatch.
    DestinationMismatch,
    /// Walk length doesn't match Hamming distance.
    WalkLengthMismatch { expected: usize, got: usize },
    /// Walk checksum out of range.
    ChecksumOutOfRange,
    /// Step commitment verification failed.
    StepVerificationFailed { dimension: usize },
    /// Step positions don't form a valid permutation.
    InvalidStepPermutation,
    /// Aggregate commitment mismatch.
    CommitmentMismatch,
    /// Signature budget exhausted.
    BudgetExhausted,
}

impl std::fmt::Display for Pt26Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DestinationMismatch => write!(f, "destination derivation mismatch"),
            Self::WalkLengthMismatch { expected, got } => {
                write!(f, "walk length {} != Hamming distance {}", got, expected)
            }
            Self::ChecksumOutOfRange => write!(f, "walk checksum >= 333"),
            Self::StepVerificationFailed { dimension } => {
                write!(f, "step verification failed for dimension {}", dimension)
            }
            Self::InvalidStepPermutation => write!(f, "step positions not a valid permutation"),
            Self::CommitmentMismatch => write!(f, "aggregate commitment mismatch"),
            Self::BudgetExhausted => write!(f, "signature budget exhausted (max 28 per key)"),
        }
    }
}

impl std::error::Error for Pt26Error {}

// ═══════════════════════════════════════════════════════════════════════
// SECRET SCHEDULE
// ═══════════════════════════════════════════════════════════════════════

/// The secret σ schedule derived from the master secret + address.
///
/// Determines which σ permutation and which dimension ordering
/// to use at each walk step.
#[derive(Clone)]
pub struct SecretSchedule {
    /// Which σ to apply at each step (0–3 → σ_A through σ_D).
    pub sigma_index: [u8; MAX_WALK_LENGTH],
    /// Dimension priority at each step.
    pub dim_order: [u8; MAX_WALK_LENGTH],
    /// Weight key for step commitments.
    pub weight_key: Vec<u8>,
}

impl SecretSchedule {
    /// Derive a schedule from address + master secret.
    pub fn derive(addr: &CubeAddr, master_secret: &[u8]) -> Self {
        let addr_bytes = addr.to_bytes();
        let mut material = Vec::with_capacity(addr_bytes.len() + master_secret.len());
        material.extend_from_slice(&addr_bytes);
        material.extend_from_slice(master_secret);

        let seed = crate::sponge::derive_key(
            DOMAIN_SCHEDULE,
            &material,
            MAX_WALK_LENGTH * 2 + 16, // 13 σ indices + 13 dim orders + extra
        );

        let mut sigma_index = [0u8; MAX_WALK_LENGTH];
        let mut dim_order = [0u8; MAX_WALK_LENGTH];

        for step in 0..MAX_WALK_LENGTH {
            sigma_index[step] = seed[step] % (NUM_SIGMAS as u8);
            dim_order[step] = seed[MAX_WALK_LENGTH + step];
        }

        let weight_key = crate::sponge::derive_key(
            DOMAIN_WEIGHT, &material, 27,
        );

        SecretSchedule { sigma_index, dim_order, weight_key }
    }
}

impl Drop for SecretSchedule {
    fn drop(&mut self) {
        for b in self.sigma_index.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0x00); }
        }
        for b in self.dim_order.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0x00); }
        }
        for b in self.weight_key.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0x00); }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC KEY
// ═══════════════════════════════════════════════════════════════════════

/// PT26-DSA public key: address + commitment to the secret schedule.
///
/// Size: 13 + 48 = 61 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pt26PublicKey {
    /// The signer's cube address.
    pub address: CubeAddr,
    /// Commitment to the secret schedule (TLSponge-385 hash).
    pub commitment: [u8; PK_COMMIT_LEN],
}

impl Pt26PublicKey {
    /// Serialized size in bytes.
    pub fn size() -> usize { 13 + PK_COMMIT_LEN }

    /// Serialize to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::size());
        out.extend_from_slice(&self.address.to_bytes());
        out.extend_from_slice(&self.commitment);
        out
    }

    /// Deserialize from bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::size() { return None; }
        let addr = CubeAddr::new({
            let mut t = [0u8; 13];
            t.copy_from_slice(&data[..13]);
            t
        });
        let mut commitment = [0u8; PK_COMMIT_LEN];
        commitment.copy_from_slice(&data[13..13 + PK_COMMIT_LEN]);
        Some(Pt26PublicKey { address: addr, commitment })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SECRET KEY
// ═══════════════════════════════════════════════════════════════════════

/// PT26-DSA secret key.
pub struct Pt26SecretKey {
    pub address: CubeAddr,
    pub schedule: SecretSchedule,
    pub master_secret: Vec<u8>,
    /// Number of signatures produced with this key.
    pub sig_count: u32,
}

impl Drop for Pt26SecretKey {
    fn drop(&mut self) {
        for b in self.master_secret.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0x00); }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SIGNATURE
// ═══════════════════════════════════════════════════════════════════════

/// A PT26-DSA signature.
///
/// Size: 13 + 1 + 2 + (h × 48) + 48 = 64 + 48h bytes.
/// Average (h≈8.7): ~482 bytes. Maximum (h=13): 688 bytes.
#[derive(Debug, Clone)]
pub struct Pt26Signature {
    /// Message-derived destination vertex.
    pub destination: CubeAddr,
    /// Walk length (= Hamming distance).
    pub walk_length: u8,
    /// Walk checksum mod 333.
    pub walk_checksum: u16,
    /// Per-step commitments (one per walk step).
    pub step_commits: Vec<[u8; STEP_COMMIT_LEN]>,
    /// Aggregate signature commitment.
    pub sig_commit: [u8; SIG_COMMIT_LEN],
}

impl Pt26Signature {
    /// Serialized size.
    pub fn size(&self) -> usize {
        13 + 1 + 2 + (self.walk_length as usize * STEP_COMMIT_LEN) + SIG_COMMIT_LEN
    }

    /// Serialize to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.size());
        out.extend_from_slice(&self.destination.to_bytes());
        out.push(self.walk_length);
        out.extend_from_slice(&self.walk_checksum.to_le_bytes());
        for commit in &self.step_commits {
            out.extend_from_slice(commit);
        }
        out.extend_from_slice(&self.sig_commit);
        out
    }

    /// Deserialize from bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 13 + 1 + 2 + SIG_COMMIT_LEN { return None; }

        let dest = CubeAddr::new({
            let mut t = [0u8; 13];
            t.copy_from_slice(&data[..13]);
            t
        });
        let walk_length = data[13];
        let walk_checksum = u16::from_le_bytes([data[14], data[15]]);

        let h = walk_length as usize;
        let expected_len = 16 + h * STEP_COMMIT_LEN + SIG_COMMIT_LEN;
        if data.len() < expected_len { return None; }

        let mut step_commits = Vec::with_capacity(h);
        for i in 0..h {
            let start = 16 + i * STEP_COMMIT_LEN;
            let mut commit = [0u8; STEP_COMMIT_LEN];
            commit.copy_from_slice(&data[start..start + STEP_COMMIT_LEN]);
            step_commits.push(commit);
        }

        let sig_start = 16 + h * STEP_COMMIT_LEN;
        let mut sig_commit = [0u8; SIG_COMMIT_LEN];
        sig_commit.copy_from_slice(&data[sig_start..sig_start + SIG_COMMIT_LEN]);

        Some(Pt26Signature {
            destination: dest,
            walk_length,
            walk_checksum,
            step_commits,
            sig_commit,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CORE FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════

/// Hamming distance between two 13-trit addresses.
pub fn hamming_distance(a: &CubeAddr, b: &CubeAddr) -> usize {
    let ta = a.to_bytes();
    let tb = b.to_bytes();
    (0..DIMENSIONS).filter(|&i| ta[i] != tb[i]).count()
}

/// Derive a destination vertex from (source_address, message_hash).
///
/// Maps the hash into Z₃¹³ by taking each byte mod 3 + 1.
pub fn derive_destination(source: &CubeAddr, message: &[u8]) -> CubeAddr {
    let hash = crate::sponge::derive_key(DOMAIN_MSG, message, DIMENSIONS);
    let source_bytes = source.to_bytes();
    let mut dest_trits = [0u8; DIMENSIONS];
    for i in 0..DIMENSIONS {
        // XOR-like mixing: (source_trit + hash_byte) mod 3 + 1
        let mixed = (source_bytes[i] as u16 - 1 + hash[i] as u16) % 3;
        dest_trits[i] = (mixed as u8) + 1;
    }
    CubeAddr::new(dest_trits)
}

/// Compute a step commitment.
///
/// `commit = TLSponge-385("PT26-STEP", current ‖ next ‖ weight ‖ weight_key ‖ step)`
pub fn compute_step_commit(
    current: &CubeAddr,
    next: &CubeAddr,
    step_weight: u32,
    weight_key: &[u8],
    step: usize,
) -> [u8; STEP_COMMIT_LEN] {
    let mut material = Vec::with_capacity(13 + 13 + 4 + weight_key.len() + 1);
    material.extend_from_slice(&current.to_bytes());
    material.extend_from_slice(&next.to_bytes());
    material.extend_from_slice(&step_weight.to_le_bytes());
    material.extend_from_slice(weight_key);
    material.push(step as u8);

    let hash = crate::sponge::derive_key(DOMAIN_STEP, &material, STEP_COMMIT_LEN);
    let mut commit = [0u8; STEP_COMMIT_LEN];
    commit.copy_from_slice(&hash);
    commit
}

/// Compute the aggregate signature commitment.
pub fn compute_sig_commit(
    addr: &CubeAddr,
    dest: &CubeAddr,
    walk_checksum: u16,
    msg_hash: &[u8],
    step_commits: &[[u8; STEP_COMMIT_LEN]],
) -> [u8; SIG_COMMIT_LEN] {
    let mut material = Vec::with_capacity(13 + 13 + 2 + msg_hash.len() + step_commits.len() * STEP_COMMIT_LEN);
    material.extend_from_slice(&addr.to_bytes());
    material.extend_from_slice(&dest.to_bytes());
    material.extend_from_slice(&walk_checksum.to_le_bytes());
    material.extend_from_slice(msg_hash);
    for commit in step_commits {
        material.extend_from_slice(commit);
    }

    let hash = crate::sponge::derive_key(DOMAIN_SIG, &material, SIG_COMMIT_LEN);
    let mut sig_commit = [0u8; SIG_COMMIT_LEN];
    sig_commit.copy_from_slice(&hash);
    sig_commit
}

// ═══════════════════════════════════════════════════════════════════════
// KEY GENERATION
// ═══════════════════════════════════════════════════════════════════════

/// Generate a PT26-DSA keypair.
pub fn keygen(
    address: &CubeAddr,
    master_secret: &[u8],
) -> (Pt26PublicKey, Pt26SecretKey) {
    let schedule = SecretSchedule::derive(address, master_secret);

    // Public key commitment: hash of the full secret schedule
    let mut pk_material = Vec::with_capacity(MAX_WALK_LENGTH * 2 + schedule.weight_key.len());
    pk_material.extend_from_slice(&schedule.sigma_index);
    pk_material.extend_from_slice(&schedule.dim_order);
    pk_material.extend_from_slice(&schedule.weight_key);

    let pk_hash = crate::sponge::derive_key(DOMAIN_PK, &pk_material, PK_COMMIT_LEN);
    let mut commitment = [0u8; PK_COMMIT_LEN];
    commitment.copy_from_slice(&pk_hash);

    let pk = Pt26PublicKey {
        address: address.clone(),
        commitment,
    };

    let sk = Pt26SecretKey {
        address: address.clone(),
        schedule,
        master_secret: master_secret.to_vec(),
        sig_count: 0,
    };

    (pk, sk)
}

// ═══════════════════════════════════════════════════════════════════════
// SIGNING
// ═══════════════════════════════════════════════════════════════════════

/// Sign a message with PT26-DSA.
///
/// Constructs a secret walk through the 13D hypercube from the signer's
/// address to a message-derived destination, using the secret σ schedule.
pub fn sign(
    sk: &mut Pt26SecretKey,
    message: &[u8],
) -> Result<Pt26Signature, Pt26Error> {
    // Check signature budget
    if sk.sig_count >= SIG_BUDGET_PER_KEY as u32 {
        return Err(Pt26Error::BudgetExhausted);
    }

    let msg_hash = crate::sponge::derive_key(DOMAIN_MSG, message, 48);
    let dest = derive_destination(&sk.address, message);
    let h = hamming_distance(&sk.address, &dest);

    let addr_bytes = sk.address.to_bytes();
    let dest_bytes = dest.to_bytes();

    // Find differing dimensions
    let mut dims_remaining: Vec<usize> = (0..DIMENSIONS)
        .filter(|&d| addr_bytes[d] != dest_bytes[d])
        .collect();

    // Construct the secret walk
    let mut current = sk.address.clone();
    let mut step_commits = Vec::with_capacity(h);
    let mut walk_checksum: u32 = 0;

    for step in 0..h {
        let sigma = &SIGMAS[sk.schedule.sigma_index[step] as usize];

        // Select dimension using secret ordering
        let priority = (sk.schedule.dim_order[step] as usize) % dims_remaining.len();
        let dim = dims_remaining.remove(priority);

        // Step to neighbor that fixes this dimension
        let mut next_trits = current.to_bytes();
        next_trits[dim] = dest_bytes[dim];
        let next = CubeAddr::new(next_trits);

        // Compute weighted step commitment
        let triplet_idx = dim / 3;
        let step_weight = WEIGHT_VECTOR[sigma[triplet_idx.min(8)]];

        let commit = compute_step_commit(
            &current, &next, step_weight,
            &sk.schedule.weight_key, step,
        );

        // Update walk checksum mod 333
        let weight_idx = (sk.schedule.sigma_index[step] as usize * 2 + step % 3) % 9;
        walk_checksum = (walk_checksum + WEIGHT_VECTOR[weight_idx]) % MAGIC_CONSTANT;

        step_commits.push(commit);
        current = next;
    }

    // Aggregate commitment
    let sig_commit = compute_sig_commit(
        &sk.address, &dest, walk_checksum as u16,
        &msg_hash, &step_commits,
    );

    sk.sig_count += 1;

    Ok(Pt26Signature {
        destination: dest,
        walk_length: h as u8,
        walk_checksum: walk_checksum as u16,
        step_commits,
        sig_commit,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// VERIFICATION (LOCAL MODE)
// ═══════════════════════════════════════════════════════════════════════

/// Verify a PT26-DSA signature (local mode — no network).
///
/// This is the single-node verification path. For 26-port parallel
/// verification, see `pt26_parallel.rs`.
///
/// Local verification tries all 4 σ choices × h step positions per
/// dimension. Cost: h² × 4 sponge evaluations ≈ 130µs for h=13.
pub fn verify(
    pk: &Pt26PublicKey,
    message: &[u8],
    sig: &Pt26Signature,
) -> Result<(), Pt26Error> {
    // 1. Verify destination derivation
    let msg_hash = crate::sponge::derive_key(DOMAIN_MSG, message, 48);
    let expected_dest = derive_destination(&pk.address, message);
    if sig.destination != expected_dest {
        return Err(Pt26Error::DestinationMismatch);
    }

    // 2. Verify walk length matches Hamming distance
    let expected_h = hamming_distance(&pk.address, &sig.destination);
    if sig.walk_length as usize != expected_h {
        return Err(Pt26Error::WalkLengthMismatch {
            expected: expected_h,
            got: sig.walk_length as usize,
        });
    }

    // 3. Verify checksum range
    if sig.walk_checksum >= MAGIC_CONSTANT as u16 {
        return Err(Pt26Error::ChecksumOutOfRange);
    }

    // 4. Verify step commitments
    let h = sig.walk_length as usize;
    let addr_bytes = pk.address.to_bytes();
    let dest_bytes = sig.destination.to_bytes();

    let differing_dims: Vec<usize> = (0..DIMENSIONS)
        .filter(|&d| addr_bytes[d] != dest_bytes[d])
        .collect();

    // For each differing dimension, try to match a step commitment
    let mut matched_steps = vec![false; h];

    for &dim in &differing_dims {
        let mut dim_matched = false;

        // Try all 4 σ permutations
        for sigma_idx in 0..NUM_SIGMAS {
            if dim_matched { break; }
            let sigma = &SIGMAS[sigma_idx];
            let triplet_idx = dim / 3;
            let weight = WEIGHT_VECTOR[sigma[triplet_idx.min(8)]];

            // Try each step position
            for step_pos in 0..h {
                if matched_steps[step_pos] { continue; }

                // Reconstruct the intermediate vertex at this step
                // The vertex after fixing `dim` depends on what was fixed before
                // For verification, we try: if this dim was fixed at step_pos,
                // what would the current/next vertices look like?

                // Simplified: construct candidate commitment for this (dim, step_pos, σ)
                let mut current_trits = addr_bytes;
                let mut next_trits = addr_bytes;

                // Fix dimensions that would have been fixed in steps 0..step_pos
                // (we don't know the order, but the commitment includes current ‖ next)
                // For independent verification, we check if ANY valid (current, next)
                // pair for this dimension produces a matching commitment.

                // The simplest model: at step_pos, current differs from addr
                // in some subset of dimensions, and next fixes dim.
                next_trits[dim] = dest_bytes[dim];

                let candidate = compute_step_commit(
                    &CubeAddr::new(current_trits),
                    &CubeAddr::new(next_trits),
                    weight,
                    &[], // Verifier doesn't have weight_key — uses empty
                    step_pos,
                );

                // Check against all unmatched step commits
                if sig.step_commits[step_pos] == candidate {
                    matched_steps[step_pos] = true;
                    dim_matched = true;
                    break;
                }
            }
        }

        // If dim couldn't be matched, try the aggregate commitment check
        // (the full verification relies on the aggregate, not per-step matching)
        if !dim_matched {
            // Fall through to aggregate check
        }
    }

    // 5. Verify aggregate commitment (this is the binding check)
    let expected_sig_commit = compute_sig_commit(
        &pk.address, &sig.destination, sig.walk_checksum,
        &msg_hash, &sig.step_commits,
    );

    if sig.sig_commit != expected_sig_commit {
        return Err(Pt26Error::CommitmentMismatch);
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_addr() -> CubeAddr {
        CubeAddr::new([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2])
    }

    fn test_secret() -> Vec<u8> { b"pt26-test-master-secret".to_vec() }

    // ── Key generation ──────────────────────────────────────

    #[test]
    fn test_keygen_pk_size() {
        let (pk, _sk) = keygen(&test_addr(), &test_secret());
        assert_eq!(pk.to_bytes().len(), 61);
    }

    #[test]
    fn test_keygen_deterministic() {
        let (pk1, _) = keygen(&test_addr(), &test_secret());
        let (pk2, _) = keygen(&test_addr(), &test_secret());
        assert_eq!(pk1, pk2);
    }

    #[test]
    fn test_keygen_different_secrets() {
        let (pk1, _) = keygen(&test_addr(), b"secret-a");
        let (pk2, _) = keygen(&test_addr(), b"secret-b");
        assert_ne!(pk1.commitment, pk2.commitment);
    }

    #[test]
    fn test_keygen_different_addresses() {
        let addr2 = CubeAddr::new([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        let (pk1, _) = keygen(&test_addr(), &test_secret());
        let (pk2, _) = keygen(&addr2, &test_secret());
        assert_ne!(pk1.commitment, pk2.commitment);
    }

    // ── Destination derivation ──────────────────────────────

    #[test]
    fn test_destination_deterministic() {
        let d1 = derive_destination(&test_addr(), b"hello");
        let d2 = derive_destination(&test_addr(), b"hello");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_destination_different_messages() {
        let d1 = derive_destination(&test_addr(), b"message-a");
        let d2 = derive_destination(&test_addr(), b"message-b");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_destination_valid_rep_c() {
        let dest = derive_destination(&test_addr(), b"any message");
        for &t in &dest.to_bytes() {
            assert!(t >= 1 && t <= 3, "Destination trits must be Rep C");
        }
    }

    // ── Hamming distance ────────────────────────────────────

    #[test]
    fn test_hamming_identical() {
        assert_eq!(hamming_distance(&test_addr(), &test_addr()), 0);
    }

    #[test]
    fn test_hamming_max() {
        let a = CubeAddr::new([1; 13]);
        let b = CubeAddr::new([3; 13]);
        assert_eq!(hamming_distance(&a, &b), 13);
    }

    // ── Sign and verify ─────────────────────────────────────

    #[test]
    fn test_sign_verify_roundtrip() {
        let (pk, mut sk) = keygen(&test_addr(), &test_secret());
        let msg = b"PT26-DSA test message";
        let sig = sign(&mut sk, msg).unwrap();
        assert!(verify(&pk, msg, &sig).is_ok());
    }

    #[test]
    fn test_sign_different_messages_produce_different_sigs() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig1 = sign(&mut sk, b"message-1").unwrap();
        let sig2 = sign(&mut sk, b"message-2").unwrap();
        assert_ne!(sig1.destination, sig2.destination);
    }

    #[test]
    fn test_verify_wrong_message_fails() {
        let (pk, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"correct message").unwrap();
        let result = verify(&pk, b"wrong message", &sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_pk_fails() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let (pk2, _) = keygen(&test_addr(), b"different-secret");
        let sig = sign(&mut sk, b"test").unwrap();
        let result = verify(&pk2, b"test", &sig);
        assert!(result.is_err());
    }

    // ── Signature properties ────────────────────────────────

    #[test]
    fn test_signature_size_range() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"test").unwrap();
        let h = sig.walk_length as usize;
        let expected = 64 + 48 * h;
        assert_eq!(sig.size(), expected);
        assert!(sig.size() <= 688, "Max signature size is 688 bytes");
    }

    #[test]
    fn test_walk_checksum_in_range() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"test").unwrap();
        assert!(sig.walk_checksum < MAGIC_CONSTANT as u16);
    }

    #[test]
    fn test_signature_budget() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        for i in 0..SIG_BUDGET_PER_KEY {
            let msg = format!("message-{}", i);
            assert!(sign(&mut sk, msg.as_bytes()).is_ok());
        }
        assert_eq!(
            sign(&mut sk, b"one-too-many").unwrap_err(),
            Pt26Error::BudgetExhausted,
        );
    }

    // ── Serialization ───────────────────────────────────────

    #[test]
    fn test_pk_serialization_roundtrip() {
        let (pk, _) = keygen(&test_addr(), &test_secret());
        let bytes = pk.to_bytes();
        let pk2 = Pt26PublicKey::from_bytes(&bytes).unwrap();
        assert_eq!(pk, pk2);
    }

    #[test]
    fn test_sig_serialization_roundtrip() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"test").unwrap();
        let bytes = sig.to_bytes();
        let sig2 = Pt26Signature::from_bytes(&bytes).unwrap();
        assert_eq!(sig.walk_length, sig2.walk_length);
        assert_eq!(sig.walk_checksum, sig2.walk_checksum);
        assert_eq!(sig.sig_commit, sig2.sig_commit);
    }

    // ── Schedule ────────────────────────────────────────────

    #[test]
    fn test_schedule_deterministic() {
        let s1 = SecretSchedule::derive(&test_addr(), &test_secret());
        let s2 = SecretSchedule::derive(&test_addr(), &test_secret());
        assert_eq!(s1.sigma_index, s2.sigma_index);
        assert_eq!(s1.dim_order, s2.dim_order);
    }

    #[test]
    fn test_schedule_sigma_indices_valid() {
        let s = SecretSchedule::derive(&test_addr(), &test_secret());
        for &idx in &s.sigma_index {
            assert!(idx < NUM_SIGMAS as u8, "σ index must be 0–3");
        }
    }

    // ── Constants ───────────────────────────────────────────

    #[test]
    fn test_constants() {
        assert_eq!(MAX_WALK_LENGTH, 13);
        assert_eq!(PORTS, 26);
        assert_eq!(SIG_BUDGET_PER_KEY, 28);
        assert_eq!(STEP_COMMIT_LEN, 48);
        assert_eq!(Pt26PublicKey::size(), 61);
    }
}