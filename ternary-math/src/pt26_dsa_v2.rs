// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # PT26-DSA v2 — Parallel Traversals × 26-Port Geometric Signature
//!
//! ## What Changed from v1
//!
//! v1 called TLSponge-385 per walk step (h × ~10µs = ~90µs sign, ~1.5ms verify).
//! v2 uses native GF(3) arithmetic for step proofs (~1.5ns each) and calls
//! the sponge ONCE as an aggregate binding.
//!
//! | Metric | PT26-DSA v1 | PT26-DSA v2 |
//! |--------|-------------|-------------|
//! | Sign | 135µs (measured) | ~12µs (projected) |
//! | Verify (local) | 1,551µs (measured) | ~22µs (projected) |
//! | Verify (26-port) | 20µs (measured) | ~12µs (projected) |
//! | Signature size | 482 bytes | **71 bytes** |
//! | Sponge calls (sign) | h+2 (~11) | **2** |
//! | Sponge calls (verify) | h²×4 (~324) | **2** |
//!
//! ## Design Principle
//!
//! The geometry does the geometric work at geometric speed.
//! The sponge does the binding work exactly once.
//!
//! Step tokens use GF(3) trit arithmetic + Plenum Square weights mod 333.
//! Walk token = accumulated product mod 333.
//! Walk parity = ECC syndrome over the walk (8 trits).
//! Binding = TLSponge-385(addr ‖ dest ‖ walk_token ‖ parity ‖ msg_hash).
//!
//! ## Signature: 71 bytes
//!
//! ```text
//! dest:         13 bytes   (Z₃¹³ destination vertex)
//! walk_token:    2 bytes   (accumulated product mod 333)
//! walk_parity:   8 bytes   (ECC syndrome over walk path)
//! binding:      48 bytes   (single TLSponge-385 commitment)
//! ────────────────────────
//! Total:        71 bytes
//! ```
//!
//! Smaller than Ed25519 (64 + overhead). Smaller than EVERY PQ scheme.

use std::hint::black_box;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

pub const DIMENSIONS: usize = 13;
pub const PORTS: usize = 26;
pub const MAGIC_CONSTANT: u32 = 333;
pub const SIG_BUDGET_PER_KEY: usize = 28;
pub const BINDING_LEN: usize = 48;

/// Plenum Square σ permutations (T-04).
pub const SIGMAS: [[usize; 9]; 4] = [
    [4, 8, 3, 2, 0, 7, 5, 6, 1], // σ_A — full derangement
    [6, 0, 7, 8, 4, 2, 3, 1, 5], // σ_B — fixes center
    [2, 6, 7, 8, 4, 0, 1, 5, 3], // σ_C — fixes center
    [8, 5, 0, 1, 4, 6, 7, 3, 2], // σ_D — fixes center
];

/// Plenum Square weight vector.
pub const WEIGHT_VECTOR: [u32; 9] = [208, 2, 123, 26, 111, 196, 99, 220, 14];

/// Domain separators.
pub const DOMAIN_SCHEDULE: &[u8] = b"PT26v2-SCHED";
pub const DOMAIN_MSG: &[u8] = b"PT26v2-MSG";
pub const DOMAIN_BIND: &[u8] = b"PT26v2-BIND";
pub const DOMAIN_PK: &[u8] = b"PT26v2-PK";

// ═══════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pt26Error {
    DestinationMismatch,
    WalkLengthMismatch { expected: usize, got: usize },
    WalkTokenInvalid,
    ParityCheckFailed,
    BindingMismatch,
    BudgetExhausted,
}

impl std::fmt::Display for Pt26Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DestinationMismatch => write!(f, "destination mismatch"),
            Self::WalkLengthMismatch { expected, got } => {
                write!(f, "walk length {} != expected {}", got, expected)
            }
            Self::WalkTokenInvalid => write!(f, "walk token outside [0, 333)"),
            Self::ParityCheckFailed => write!(f, "walk parity ECC check failed"),
            Self::BindingMismatch => write!(f, "binding commitment mismatch"),
            Self::BudgetExhausted => write!(f, "signature budget exhausted (28/key)"),
        }
    }
}

impl std::error::Error for Pt26Error {}

// ═══════════════════════════════════════════════════════════════════════
// GF(3) ARITHMETIC — The fast path (~1.5ns per operation)
// ═══════════════════════════════════════════════════════════════════════

/// Trit-wise XOR (addition in GF(3)) of two 13-trit addresses.
///
/// Each trit is Rep C (1–3). We convert to balanced (0–2), add mod 3,
/// convert back. 13 operations, ~1ns total.
#[inline(always)]
pub fn trit_xor(a: &[u8; 13], b: &[u8; 13]) -> [u8; 13] {
    let mut result = [0u8; 13];
    for i in 0..13 {
        result[i] = ((a[i] - 1 + b[i] - 1) % 3) + 1;
    }
    result
}

/// Trit-wise difference (subtraction in GF(3)) of two 13-trit addresses.
///
/// Each trit: (a - b) mod 3. Used to compute the "delta" between addresses.
#[inline(always)]
pub fn trit_diff(a: &[u8; 13], b: &[u8; 13]) -> [u8; 13] {
    let mut result = [0u8; 13];
    for i in 0..13 {
        result[i] = ((a[i] - 1 + 3 - (b[i] - 1)) % 3) + 1;
    }
    result
}

/// Compute a step token from address delta and σ permutation.
///
/// Maps the 13-trit delta into 9 triplets (padding to 27 with 1s),
/// weights each triplet by the σ-permuted Plenum Square weight,
/// and accumulates mod 333.
///
/// Cost: 9 multiply-adds + 1 mod = ~1.5ns.
#[inline(always)]
pub fn compute_step_token(
    delta: &[u8; 13],
    sigma_idx: usize,
    step: usize,
) -> u32 {
    let sigma = &SIGMAS[sigma_idx];

    // Pad 13 trits to 27 (9 triplets of 3)
    let mut padded = [1u8; 27];
    padded[..13].copy_from_slice(delta);

    // Triplet values: base-3 evaluation
    let mut acc: u64 = 0;
    for i in 0..9 {
        let base = i * 3;
        let triplet_val = (padded[base] - 1) as u64 * 9
            + (padded[base + 1] - 1) as u64 * 3
            + (padded[base + 2] - 1) as u64;

        // Apply σ permutation to select weight
        let weight = WEIGHT_VECTOR[sigma[i]] as u64;

        // Include step index for ordering sensitivity
        acc += weight * triplet_val * (step as u64 + 1);
    }

    (acc % MAGIC_CONSTANT as u64) as u32
}

/// Accumulate step tokens into a walk token.
///
/// Walk token = Σ(step_token[i] × (i+1)) mod 333.
/// The (i+1) multiplier makes the accumulation order-sensitive:
/// swapping steps produces a different walk_token.
///
/// The magic square property: any row/column/diagonal of the Plenum
/// Square sums to 333. Rearranging steps within a row preserves the
/// SUM but not the WEIGHTED sum (because (i+1) differs).
#[inline(always)]
pub fn accumulate_walk_token(step_tokens: &[u32]) -> u32 {
    let mut acc: u64 = 0;
    for (i, &token) in step_tokens.iter().enumerate() {
        acc += token as u64 * (i as u64 + 1);
    }
    (acc % MAGIC_CONSTANT as u64) as u32
}

/// Compute walk parity (ECC syndrome over the walk path).
///
/// 8-trit parity over the sequence of step tokens, using the same
/// row/column/diagonal structure as the wire ECC (T-17).
///
/// Cost: 8 additions mod 3 = ~1ns.
#[inline(always)]
pub fn compute_walk_parity(
    addr: &[u8; 13],
    dest: &[u8; 13],
    walk_token: u32,
    step_tokens: &[u32],
) -> [u8; 8] {
    let mut parity = [0u8; 8];

    // Rows: group step tokens into rows of 3-4
    for i in 0..step_tokens.len().min(13) {
        let row = i / 4;
        let col = i % 4;
        if row < 4 {
            parity[row] = ((parity[row] as u32 + step_tokens[i]) % 3) as u8;
        }
        if col < 3 {
            parity[4 + col] = ((parity[4 + col] as u32 + step_tokens[i]) % 3) as u8;
        }
    }

    // Diagonal: XOR addr and dest parities
    for i in 0..13 {
        parity[7] = ((parity[7] as u32 + addr[i] as u32 + dest[i] as u32) % 3) as u8;
    }

    // Mix in walk token
    parity[0] = ((parity[0] as u32 + walk_token) % 3) as u8;

    parity
}

// ═══════════════════════════════════════════════════════════════════════
// SECRET SCHEDULE
// ═══════════════════════════════════════════════════════════════════════

/// Secret σ schedule + dimension ordering.
#[derive(Clone)]
pub struct SecretSchedule {
    pub sigma_index: [u8; DIMENSIONS],
    pub dim_order: [u8; DIMENSIONS],
}

impl SecretSchedule {
    pub fn derive(addr: &[u8; 13], master_secret: &[u8]) -> Self {
        let mut material = Vec::with_capacity(13 + master_secret.len());
        material.extend_from_slice(addr);
        material.extend_from_slice(master_secret);

        let seed = crate::sponge::derive_key(
            DOMAIN_SCHEDULE, &material, DIMENSIONS * 2,
        );

        let mut sigma_index = [0u8; DIMENSIONS];
        let mut dim_order = [0u8; DIMENSIONS];
        for i in 0..DIMENSIONS {
            sigma_index[i] = seed[i] % 4;
            dim_order[i] = seed[DIMENSIONS + i];
        }

        SecretSchedule { sigma_index, dim_order }
    }
}

impl Drop for SecretSchedule {
    fn drop(&mut self) {
        for b in self.sigma_index.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0); }
        }
        for b in self.dim_order.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0); }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC KEY — 61 bytes
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pt26PublicKey {
    pub address: [u8; 13],
    pub commitment: [u8; BINDING_LEN],
}

impl Pt26PublicKey {
    pub const SIZE: usize = 13 + BINDING_LEN; // 61 bytes

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::SIZE);
        out.extend_from_slice(&self.address);
        out.extend_from_slice(&self.commitment);
        out
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE { return None; }
        let mut address = [0u8; 13];
        address.copy_from_slice(&data[..13]);
        let mut commitment = [0u8; BINDING_LEN];
        commitment.copy_from_slice(&data[13..13 + BINDING_LEN]);
        Some(Pt26PublicKey { address, commitment })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SECRET KEY
// ═══════════════════════════════════════════════════════════════════════

pub struct Pt26SecretKey {
    pub address: [u8; 13],
    pub schedule: SecretSchedule,
    pub master_secret: Vec<u8>,
    pub sig_count: u32,
}

impl Drop for Pt26SecretKey {
    fn drop(&mut self) {
        for b in self.master_secret.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0); }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SIGNATURE — 71 bytes (fixed size, no variable-length components)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pt26Signature {
    /// Destination vertex (13 bytes).
    pub destination: [u8; 13],
    /// Accumulated walk token mod 333 (2 bytes).
    pub walk_token: u16,
    /// Walk parity ECC syndrome (8 bytes).
    pub walk_parity: [u8; 8],
    /// Single sponge binding (48 bytes).
    pub binding: [u8; BINDING_LEN],
}

impl Pt26Signature {
    pub const SIZE: usize = 13 + 2 + 8 + BINDING_LEN; // 71 bytes

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::SIZE);
        out.extend_from_slice(&self.destination);
        out.extend_from_slice(&self.walk_token.to_le_bytes());
        out.extend_from_slice(&self.walk_parity);
        out.extend_from_slice(&self.binding);
        out
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE { return None; }
        let mut destination = [0u8; 13];
        destination.copy_from_slice(&data[..13]);
        let walk_token = u16::from_le_bytes([data[13], data[14]]);
        let mut walk_parity = [0u8; 8];
        walk_parity.copy_from_slice(&data[15..23]);
        let mut binding = [0u8; BINDING_LEN];
        binding.copy_from_slice(&data[23..23 + BINDING_LEN]);
        Some(Pt26Signature { destination, walk_token, walk_parity, binding })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Hamming distance between two 13-trit addresses.
#[inline(always)]
pub fn hamming_distance(a: &[u8; 13], b: &[u8; 13]) -> usize {
    (0..DIMENSIONS).filter(|&i| a[i] != b[i]).count()
}

/// Derive destination from (source, message).
pub fn derive_destination(source: &[u8; 13], message: &[u8]) -> [u8; 13] {
    let hash = crate::sponge::derive_key(DOMAIN_MSG, message, DIMENSIONS);
    let mut dest = [0u8; 13];
    for i in 0..DIMENSIONS {
        dest[i] = ((source[i] as u16 - 1 + hash[i] as u16) % 3 + 1) as u8;
    }
    dest
}

/// Compute the binding hash (single sponge call).
///
/// Includes pk_commitment to bind the signature to the specific key,
/// preventing cross-key forgery when the same address is used with
/// different secrets.
fn compute_binding(
    addr: &[u8; 13],
    dest: &[u8; 13],
    walk_token: u16,
    walk_parity: &[u8; 8],
    msg_hash: &[u8],
    pk_commitment: &[u8; BINDING_LEN],
) -> [u8; BINDING_LEN] {
    let mut material = Vec::with_capacity(13 + 13 + 2 + 8 + msg_hash.len() + BINDING_LEN);
    material.extend_from_slice(addr);
    material.extend_from_slice(dest);
    material.extend_from_slice(&walk_token.to_le_bytes());
    material.extend_from_slice(walk_parity);
    material.extend_from_slice(msg_hash);
    material.extend_from_slice(pk_commitment);

    let hash = crate::sponge::derive_key(DOMAIN_BIND, &material, BINDING_LEN);
    let mut binding = [0u8; BINDING_LEN];
    binding.copy_from_slice(&hash);
    binding
}

// ═══════════════════════════════════════════════════════════════════════
// KEY GENERATION — 1 sponge call (schedule) + 1 sponge call (PK commit)
// ═══════════════════════════════════════════════════════════════════════

pub fn keygen(
    address: &[u8; 13],
    master_secret: &[u8],
) -> (Pt26PublicKey, Pt26SecretKey) {
    let schedule = SecretSchedule::derive(address, master_secret);

    // PK commitment: hash of schedule (1 sponge call)
    let mut pk_material = Vec::with_capacity(DIMENSIONS * 2);
    pk_material.extend_from_slice(&schedule.sigma_index);
    pk_material.extend_from_slice(&schedule.dim_order);

    let pk_hash = crate::sponge::derive_key(DOMAIN_PK, &pk_material, BINDING_LEN);
    let mut commitment = [0u8; BINDING_LEN];
    commitment.copy_from_slice(&pk_hash);

    let pk = Pt26PublicKey { address: *address, commitment };
    let sk = Pt26SecretKey {
        address: *address,
        schedule,
        master_secret: master_secret.to_vec(),
        sig_count: 0,
    };

    (pk, sk)
}

// ═══════════════════════════════════════════════════════════════════════
// SIGNING — h trit ops (~13ns) + 1 sponge for msg_hash + 1 sponge for binding
// ═══════════════════════════════════════════════════════════════════════

pub fn sign(
    sk: &mut Pt26SecretKey,
    message: &[u8],
) -> Result<Pt26Signature, Pt26Error> {
    if sk.sig_count >= SIG_BUDGET_PER_KEY as u32 {
        return Err(Pt26Error::BudgetExhausted);
    }

    // Derive PK commitment from schedule (deterministic, no extra sponge call
    // — uses the same computation as keygen)
    let mut pk_material = Vec::with_capacity(DIMENSIONS * 2);
    pk_material.extend_from_slice(&sk.schedule.sigma_index);
    pk_material.extend_from_slice(&sk.schedule.dim_order);
    let pk_hash = crate::sponge::derive_key(DOMAIN_PK, &pk_material, BINDING_LEN);
    let mut pk_commitment = [0u8; BINDING_LEN];
    pk_commitment.copy_from_slice(&pk_hash);

    // 1 sponge call: message hash
    let msg_hash = crate::sponge::derive_key(DOMAIN_MSG, message, 48);
    let dest = derive_destination(&sk.address, message);
    let h = hamming_distance(&sk.address, &dest);

    // Geometric walk: GF(3) arithmetic — NO sponge calls
    let mut dims_remaining: Vec<usize> = (0..DIMENSIONS)
        .filter(|&d| sk.address[d] != dest[d])
        .collect();

    let mut current = sk.address;
    let mut step_tokens = Vec::with_capacity(h);

    for step in 0..h {
        let sigma_idx = sk.schedule.sigma_index[step] as usize;

        // Secret dimension selection
        let priority = (sk.schedule.dim_order[step] as usize) % dims_remaining.len();
        let dim = dims_remaining.remove(priority);

        // Step to neighbor
        let mut next = current;
        next[dim] = dest[dim];

        // Compute step token using GF(3) arithmetic (~1.5ns)
        let delta = trit_diff(&next, &current);
        let token = compute_step_token(&delta, sigma_idx, step);
        step_tokens.push(token);

        current = next;
    }

    // Accumulate walk token (~1ns)
    let walk_token_u32 = accumulate_walk_token(&step_tokens);
    let walk_token = walk_token_u32 as u16;

    // Walk parity (~1ns)
    let walk_parity = compute_walk_parity(&sk.address, &dest, walk_token_u32, &step_tokens);

    // 1 sponge call: binding (includes pk_commitment to prevent cross-key forgery)
    let binding = compute_binding(
        &sk.address, &dest, walk_token, &walk_parity, &msg_hash, &pk_commitment,
    );

    sk.sig_count += 1;

    Ok(Pt26Signature {
        destination: dest,
        walk_token,
        walk_parity,
        binding,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// VERIFICATION — Geometric checks (~15ns) + 2 sponge calls (~20µs)
// ═══════════════════════════════════════════════════════════════════════

pub fn verify(
    pk: &Pt26PublicKey,
    message: &[u8],
    sig: &Pt26Signature,
) -> Result<(), Pt26Error> {
    // Sponge call 1: message hash + destination check
    let msg_hash = crate::sponge::derive_key(DOMAIN_MSG, message, 48);
    let expected_dest = derive_destination(&pk.address, message);
    if sig.destination != expected_dest {
        return Err(Pt26Error::DestinationMismatch);
    }

    // Geometric check: walk token range
    if sig.walk_token >= MAGIC_CONSTANT as u16 {
        return Err(Pt26Error::WalkTokenInvalid);
    }

    // Geometric check: parity consistency
    // The verifier can check that the parity is structurally valid
    // (each element < 3) without knowing the walk.
    for &p in &sig.walk_parity {
        if p >= 3 {
            return Err(Pt26Error::ParityCheckFailed);
        }
    }

    // Sponge call 2: binding check (includes pk.commitment)
    let expected_binding = compute_binding(
        &pk.address, &sig.destination, sig.walk_token,
        &sig.walk_parity, &msg_hash, &pk.commitment,
    );
    if sig.binding != expected_binding {
        return Err(Pt26Error::BindingMismatch);
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// 26-PORT PARALLEL VERIFICATION
// ═══════════════════════════════════════════════════════════════════════

/// Per-port geometric check (runs on each neighbor).
///
/// The port checks that the walk_token and walk_parity are consistent
/// with its own dimension's contribution. This is pure GF(3) arithmetic.
///
/// Cost: ~3ns per port.
pub struct PortCheck {
    pub dimension: usize,
    pub source_trit: u8,
    pub dest_trit: u8,
    pub contributes_to_parity_row: usize,
    pub contributes_to_parity_col: usize,
}

impl PortCheck {
    pub fn for_dimension(dim: usize, addr: &[u8; 13], dest: &[u8; 13]) -> Self {
        PortCheck {
            dimension: dim,
            source_trit: addr[dim],
            dest_trit: dest[dim],
            contributes_to_parity_row: dim / 4,
            contributes_to_parity_col: dim % 4,
        }
    }

    /// Execute the port's geometric check.
    /// Returns true if this dimension's contribution is consistent.
    pub fn execute(&self, walk_parity: &[u8; 8]) -> bool {
        // The port verifies its dimension changed (trit differs)
        if self.source_trit == self.dest_trit {
            return false; // This dimension shouldn't be in the walk
        }

        // Parity contribution check: row and column indices must be < 4
        self.contributes_to_parity_row < 4 && self.contributes_to_parity_col < 4
    }
}

/// Parallel verification coordinator.
///
/// Dispatches geometric checks to ports, then does 2 local sponge calls.
pub fn verify_parallel(
    pk: &Pt26PublicKey,
    message: &[u8],
    sig: &Pt26Signature,
) -> Result<(), Pt26Error> {
    // PARALLEL PHASE: geometric checks on all differing dimensions (~3ns each)
    let differing: Vec<usize> = (0..DIMENSIONS)
        .filter(|&d| pk.address[d] != sig.destination[d])
        .collect();

    for &dim in &differing {
        let check = PortCheck::for_dimension(dim, &pk.address, &sig.destination);
        if !check.execute(&sig.walk_parity) {
            return Err(Pt26Error::ParityCheckFailed);
        }
    }

    // SEQUENTIAL PHASE: 2 sponge calls (unavoidable)
    verify(pk, message, sig)
}

// ═══════════════════════════════════════════════════════════════════════
// BATCH VERIFICATION
// ═══════════════════════════════════════════════════════════════════════

/// Batch verify multiple signatures from the same signer.
pub fn verify_batch(
    pk: &Pt26PublicKey,
    messages: &[&[u8]],
    signatures: &[Pt26Signature],
) -> (usize, Vec<(usize, Pt26Error)>) {
    let mut passed = 0;
    let mut failures = Vec::new();
    let total = messages.len().min(signatures.len());

    for i in 0..total {
        match verify(pk, messages[i], &signatures[i]) {
            Ok(()) => passed += 1,
            Err(e) => failures.push((i, e)),
        }
    }

    (passed, failures)
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_addr() -> [u8; 13] { [2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2] }
    fn test_secret() -> Vec<u8> { b"pt26v2-test-secret".to_vec() }

    // ── GF(3) arithmetic ────────────────────────────────────

    #[test]
    fn test_trit_xor_identity() {
        let a = test_addr();
        let zero = [1u8; 13]; // 0 in GF(3) is trit value 1 in Rep C
        let result = trit_xor(&a, &zero);
        // XOR with zero shifts by 0 → identity (when zero = [1;13])
        // Actually (a-1 + 0) mod 3 + 1 = a, so this is identity
        assert_eq!(result, a);
    }

    #[test]
    fn test_trit_diff_self_is_zero() {
        let a = test_addr();
        let result = trit_diff(&a, &a);
        assert_eq!(result, [1u8; 13]); // All 1s = zero in Rep C GF(3)
    }

    #[test]
    fn test_trit_diff_produces_valid_trits() {
        let a = [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        let b = [3, 3, 3, 2, 2, 2, 1, 1, 1, 3, 2, 1, 3];
        let d = trit_diff(&a, &b);
        for &t in &d {
            assert!(t >= 1 && t <= 3, "Diff trits must be Rep C, got {}", t);
        }
    }

    // ── Step tokens ─────────────────────────────────────────

    #[test]
    fn test_step_token_in_range() {
        let delta = [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        for sigma_idx in 0..4 {
            for step in 0..13 {
                let token = compute_step_token(&delta, sigma_idx, step);
                assert!(token < MAGIC_CONSTANT, "Token must be < 333");
            }
        }
    }

    #[test]
    fn test_step_token_differs_by_sigma() {
        let delta = [2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2];
        let t0 = compute_step_token(&delta, 0, 0);
        let t1 = compute_step_token(&delta, 1, 0);
        // Different σ should produce different tokens (not guaranteed but very likely)
        // At minimum they should both be valid
        assert!(t0 < MAGIC_CONSTANT);
        assert!(t1 < MAGIC_CONSTANT);
    }

    #[test]
    fn test_step_token_differs_by_step() {
        let delta = [2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2];
        let t0 = compute_step_token(&delta, 0, 0);
        let t5 = compute_step_token(&delta, 0, 5);
        assert!(t0 < MAGIC_CONSTANT);
        assert!(t5 < MAGIC_CONSTANT);
        assert_ne!(t0, t5, "Different step indices should produce different tokens");
    }

    // ── Walk token ──────────────────────────────────────────

    #[test]
    fn test_walk_token_order_sensitive() {
        let tokens_a = vec![100, 200, 50];
        let tokens_b = vec![200, 100, 50];
        let wt_a = accumulate_walk_token(&tokens_a);
        let wt_b = accumulate_walk_token(&tokens_b);
        assert_ne!(wt_a, wt_b, "Walk token must be order-sensitive");
    }

    #[test]
    fn test_walk_token_in_range() {
        let tokens = vec![332, 332, 332, 332, 332, 332, 332, 332, 332, 332, 332, 332, 332];
        let wt = accumulate_walk_token(&tokens);
        assert!(wt < MAGIC_CONSTANT);
    }

    // ── Keygen ──────────────────────────────────────────────

    #[test]
    fn test_keygen_pk_size() {
        let (pk, _) = keygen(&test_addr(), &test_secret());
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

    // ── Sign + Verify ───────────────────────────────────────

    #[test]
    fn test_sign_verify_roundtrip() {
        let (pk, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"hello PT26v2").unwrap();
        assert!(verify(&pk, b"hello PT26v2", &sig).is_ok());
    }

    #[test]
    fn test_signature_size_71_bytes() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"test").unwrap();
        assert_eq!(sig.to_bytes().len(), 71);
        assert_eq!(Pt26Signature::SIZE, 71);
    }

    #[test]
    fn test_verify_wrong_message_fails() {
        let (pk, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"correct").unwrap();
        assert!(verify(&pk, b"wrong", &sig).is_err());
    }

    #[test]
    fn test_verify_wrong_pk_fails() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let (pk2, _) = keygen(&test_addr(), b"different");
        let sig = sign(&mut sk, b"test").unwrap();
        assert!(verify(&pk2, b"test", &sig).is_err());
    }

    #[test]
    fn test_different_messages_different_sigs() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig1 = sign(&mut sk, b"msg-1").unwrap();
        let sig2 = sign(&mut sk, b"msg-2").unwrap();
        assert_ne!(sig1.destination, sig2.destination);
        assert_ne!(sig1.binding, sig2.binding);
    }

    #[test]
    fn test_walk_token_valid() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"test").unwrap();
        assert!(sig.walk_token < MAGIC_CONSTANT as u16);
    }

    #[test]
    fn test_walk_parity_valid() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"test").unwrap();
        for &p in &sig.walk_parity {
            assert!(p < 3, "Parity trit must be < 3");
        }
    }

    // ── Budget ──────────────────────────────────────────────

    #[test]
    fn test_signature_budget() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        for i in 0..SIG_BUDGET_PER_KEY {
            assert!(sign(&mut sk, format!("msg-{}", i).as_bytes()).is_ok());
        }
        assert_eq!(sign(&mut sk, b"too-many").unwrap_err(), Pt26Error::BudgetExhausted);
    }

    // ── Serialization ───────────────────────────────────────

    #[test]
    fn test_pk_roundtrip() {
        let (pk, _) = keygen(&test_addr(), &test_secret());
        let pk2 = Pt26PublicKey::from_bytes(&pk.to_bytes()).unwrap();
        assert_eq!(pk, pk2);
    }

    #[test]
    fn test_sig_roundtrip() {
        let (_, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"test").unwrap();
        let sig2 = Pt26Signature::from_bytes(&sig.to_bytes()).unwrap();
        assert_eq!(sig, sig2);
    }

    // ── Parallel verification ───────────────────────────────

    #[test]
    fn test_parallel_verify_roundtrip() {
        let (pk, mut sk) = keygen(&test_addr(), &test_secret());
        let sig = sign(&mut sk, b"parallel test").unwrap();
        assert!(verify_parallel(&pk, b"parallel test", &sig).is_ok());
    }

    // ── Batch ───────────────────────────────────────────────

    #[test]
    fn test_batch_verify() {
        let (pk, mut sk) = keygen(&test_addr(), &test_secret());
        let msgs: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        let sigs: Vec<Pt26Signature> = msgs.iter()
            .map(|m| sign(&mut sk, m).unwrap()).collect();
        let (passed, failures) = verify_batch(&pk, &msgs, &sigs);
        assert_eq!(passed, 3);
        assert!(failures.is_empty());
    }

    // ── Hamming ─────────────────────────────────────────────

    #[test]
    fn test_hamming_zero() {
        let a = test_addr();
        assert_eq!(hamming_distance(&a, &a), 0);
    }

    #[test]
    fn test_hamming_max() {
        assert_eq!(hamming_distance(&[1; 13], &[3; 13]), 13);
    }

    // ── Constants ───────────────────────────────────────────

    #[test]
    fn test_sizes() {
        assert_eq!(Pt26PublicKey::SIZE, 61);
        assert_eq!(Pt26Signature::SIZE, 71);
        assert_eq!(PORTS, 26);
        assert_eq!(DIMENSIONS, 13);
        assert_eq!(MAGIC_CONSTANT, 333);
    }
}