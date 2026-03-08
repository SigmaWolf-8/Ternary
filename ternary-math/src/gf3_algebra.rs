// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// GF(3) Algebra — Closed-Form Ternary-Native Operations
// Location: ternary-math/src/gf3_algebra.rs
//
// Replaces trit-by-trit loops with algebraic formulas that exploit
// the structure of GF(3) = {0, 1, 2} under modular arithmetic.
//
// DESIGN PRINCIPLE: The ternary math IS the optimization.
// No binary packing tricks, no bit manipulation hacks.
// Operations work on integer arrays (Rep B internally, Rep C externally)
// using standard arithmetic (add, multiply, mod 3).
// Hardware (SIMD, FMA) accelerates the algebra directly.
//
// KEY FORMULAS:
//   Hamming distance: d(a,b) = Σ (aᵢ - bᵢ)² mod 3
//   Forgery check:    forged = (Π tritsᵢ) mod 3 == 0
//   Sponge permute:   new[i] = old[(i × stride) mod width]
//   Sponge diffuse:   out[i] = (in[i-1] + in[i] + in[i+1]) mod 3

// ============================================================
// GF(3) ELEMENT OPERATIONS
// ============================================================

/// GF(3) addition: (a + b) mod 3
/// Constant-time, no branching.
#[inline(always)]
pub const fn gf3_add(a: u8, b: u8) -> u8 {
    (a + b) % 3
}

/// GF(3) subtraction: (a - b + 3) mod 3
/// The +3 ensures non-negative before mod.
#[inline(always)]
pub const fn gf3_sub(a: u8, b: u8) -> u8 {
    (a + 3 - b) % 3
}

/// GF(3) multiplication: (a * b) mod 3
#[inline(always)]
pub const fn gf3_mul(a: u8, b: u8) -> u8 {
    (a * b) % 3
}

/// GF(3) negation: (3 - a) mod 3
/// 0→0, 1→2, 2→1
#[inline(always)]
pub const fn gf3_neg(a: u8) -> u8 {
    (3 - a) % 3
}

/// GF(3) square: a² mod 3
/// 0→0, 1→1, 2→1
/// This is the Hamming indicator: 0 if equal, 1 if different.
#[inline(always)]
pub const fn gf3_square(a: u8) -> u8 {
    (a * a) % 3
}

/// GF(3) multiplicative inverse (nonzero elements only).
/// 1⁻¹ = 1, 2⁻¹ = 2 (both self-inverse in GF(3)).
/// Panics on 0 (no inverse exists).
#[inline(always)]
pub const fn gf3_inv(a: u8) -> u8 {
    // In GF(3): 1×1=1, 2×2=4≡1. Both self-inverse.
    assert!(a != 0, "zero has no multiplicative inverse in GF(3)");
    a  // 1→1, 2→2
}

// ============================================================
// REP CONVERSIONS
// ============================================================

/// Rep C {1,2,3} → Rep B {0,1,2}: subtract 1
#[inline(always)]
pub const fn rep_c_to_b(c: u8) -> u8 {
    debug_assert!(c >= 1 && c <= 3, "Rep C must be 1, 2, or 3");
    c - 1
}

/// Rep B {0,1,2} → Rep C {1,2,3}: add 1
#[inline(always)]
pub const fn rep_b_to_c(b: u8) -> u8 {
    debug_assert!(b <= 2, "Rep B must be 0, 1, or 2");
    b + 1
}

/// Batch Rep C → Rep B for a slice (in-place)
pub fn batch_c_to_b(trits: &mut [u8]) {
    for t in trits.iter_mut() {
        *t = rep_c_to_b(*t);
    }
}

/// Batch Rep B → Rep C for a slice (in-place)
pub fn batch_b_to_c(trits: &mut [u8]) {
    for t in trits.iter_mut() {
        *t = rep_b_to_c(*t);
    }
}

// ============================================================
// HAMMING DISTANCE — Sum of Squared Differences mod 3
// ============================================================
//
// In GF(3): (a - b)² mod 3 = 0 if a=b, 1 if a≠b.
// Proof: differences mod 3 are {0, 1, 2}. Squares: 0²=0, 1²=1, 2²=4≡1.
// Therefore: d(a, b) = Σᵢ (aᵢ - bᵢ)² mod 3
//
// This is a dot-product-like operation. CPUs have fused multiply-add
// for exactly this pattern. Pure GF(3) algebra, no encoding tricks.

/// Hamming distance between two Rep B trit vectors.
///
/// Formula: d = Σ (aᵢ - bᵢ)² mod 3
///
/// Each squared difference is 0 (match) or 1 (mismatch).
/// The sum counts mismatches — no branching, no comparison operators.
pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    debug_assert_eq!(a.len(), b.len(), "vectors must be same length");
    let mut dist: u32 = 0;
    for i in 0..a.len() {
        // (a - b + 3) mod 3 gives the GF(3) difference
        // square it: 0→0, 1→1, 2→1
        let diff = gf3_sub(a[i], b[i]);
        dist += gf3_square(diff) as u32;
    }
    dist
}

/// Hamming distance between two Rep C trit vectors.
/// Converts to Rep B internally — the formula requires GF(3) arithmetic.
pub fn hamming_distance_rep_c(a: &[u8], b: &[u8]) -> u32 {
    debug_assert_eq!(a.len(), b.len());
    let mut dist: u32 = 0;
    for i in 0..a.len() {
        let diff = gf3_sub(rep_c_to_b(a[i]), rep_c_to_b(b[i]));
        dist += gf3_square(diff) as u32;
    }
    dist
}

/// Hamming distance for 27-trit TDNS addresses (Rep C).
/// Unrolled inner loop for the common case.
pub fn hamming_distance_27(a: &[u8; 27], b: &[u8; 27]) -> u32 {
    let mut dist: u32 = 0;
    for i in 0..27 {
        let diff = (a[i] + 3 - b[i]) % 3; // Rep C: (a-b+3)%3 = (a-1-(b-1)+3)%3 = (a-b+3)%3
        dist += ((diff * diff) % 3) as u32;
    }
    dist
}

// ============================================================
// FORGERY DETECTION — Multiplicative Closure in GF(3)
// ============================================================
//
// In GF(3), the nonzero elements {1, 2} form a multiplicative group.
// The product of any number of nonzero elements stays in {1, 2}.
// If a zero appears, the product becomes 0 and stays 0.
//
// For Rep C {1,2,3}: convert to Rep B {0,1,2}, then multiply.
// If product = 0, at least one trit was 0 in Rep B = 1 in Rep C... 
// Wait, that's wrong. Let me reconsider.
//
// Rep C {1,2,3}: zero is excluded. A forged trit has value 0.
// To detect: any trit == 0 in Rep C means forgery.
// In Rep B: Rep C 0 → Rep B -1 (invalid). But we can't have -1.
// 
// Simpler: in Rep C, every valid trit is ≥ 1.
// The MINIMUM of all trit values: min < 1 → forgery.
// But min requires branching.
//
// Better: in Rep C, multiply all trits mod (some suitable number).
// Rep C values are {1,2,3}. If any is 0, the product is 0.
// Product of values in {1,2,3} is never 0.
// Product of values in {0,1,2,3} is 0 iff any value is 0.
//
// So: product of all Rep C trit values. If zero → forgery.
// But 27 values in {1,2,3}: max product = 3^27 = 7.6 trillion. Overflows u64.
//
// Solution: work mod any prime > 3. Use mod 7 (small, fast).
// If any trit is 0, product mod 7 = 0. If all trits are {1,2,3},
// product mod 7 ≠ 0 (since 1,2,3 are all coprime to 7).
// Caveat: 7 divides the product of non-zero trits only if one of them
// is a multiple of 7... but trits are {1,2,3}, none are multiples of 7.
// So product mod 7 = 0 iff at least one input is 0. ✓
//
// Even simpler: just use product mod 3.
// Trits in Rep C: {1,2,3}. Mod 3: {1,2,0}. 
// Trit value 3 maps to 0 mod 3 — that's a FALSE POSITIVE!
// 
// So mod 3 doesn't work for Rep C. Use product mod 7 or bitwise OR.
// Actually the simplest correct approach:

/// Check for forgery in a Rep C trit vector.
///
/// Forgery = any trit value is 0 (zero is excluded from Rep C).
///
/// Method: product of all values mod 7. Since valid trits {1,2,3}
/// are all coprime to 7, the product mod 7 is nonzero iff all inputs
/// are nonzero. If any input is 0, product mod 7 = 0.
///
/// Running mod-7 reduction prevents overflow (max intermediate: 6 × 3 = 18).
pub fn has_forgery(trits_rep_c: &[u8]) -> bool {
    let mut product: u8 = 1;
    for &t in trits_rep_c {
        product = (product * t) % 7;
        // Early exit: once product hits 0, it stays 0
        if product == 0 {
            return true;
        }
    }
    false
}

/// Locate ALL forged positions in a Rep C vector.
/// Returns indices where trit value is 0.
pub fn find_forgeries(trits_rep_c: &[u8]) -> Vec<usize> {
    trits_rep_c.iter()
        .enumerate()
        .filter(|(_, &t)| t == 0)
        .map(|(i, _)| i)
        .collect()
}

// ============================================================
// BATCH GF(3) VECTOR OPERATIONS
// ============================================================

/// Element-wise GF(3) addition of two vectors (Rep B).
/// out[i] = (a[i] + b[i]) mod 3
pub fn gf3_vec_add(a: &[u8], b: &[u8], out: &mut [u8]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), out.len());
    for i in 0..a.len() {
        out[i] = gf3_add(a[i], b[i]);
    }
}

/// Element-wise GF(3) subtraction (Rep B).
/// out[i] = (a[i] - b[i] + 3) mod 3
pub fn gf3_vec_sub(a: &[u8], b: &[u8], out: &mut [u8]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), out.len());
    for i in 0..a.len() {
        out[i] = gf3_sub(a[i], b[i]);
    }
}

/// Element-wise GF(3) multiplication (Rep B).
/// out[i] = (a[i] * b[i]) mod 3
pub fn gf3_vec_mul(a: &[u8], b: &[u8], out: &mut [u8]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), out.len());
    for i in 0..a.len() {
        out[i] = gf3_mul(a[i], b[i]);
    }
}

/// GF(3) dot product: Σ (aᵢ × bᵢ) mod 3
pub fn gf3_dot(a: &[u8], b: &[u8]) -> u8 {
    debug_assert_eq!(a.len(), b.len());
    let mut sum: u8 = 0;
    for i in 0..a.len() {
        sum = gf3_add(sum, gf3_mul(a[i], b[i]));
    }
    sum
}

/// Scalar multiplication: out[i] = (scalar × a[i]) mod 3
pub fn gf3_scalar_mul(scalar: u8, a: &[u8], out: &mut [u8]) {
    debug_assert_eq!(a.len(), out.len());
    for i in 0..a.len() {
        out[i] = gf3_mul(scalar, a[i]);
    }
}

// ============================================================
// SPONGE PERMUTATION — Index Remap (Zero Data Movement)
// ============================================================
//
// tisPi with stride s on width W: new[i] = old[(i × s) mod W]
//
// INVARIANT 10: gcd(s, W) = 1 required for complete cycle.
// For TIS-27: s=13, W=54, gcd(13,54)=1 ✓
//
// This is NOT a data shuffle — it's an address remapping.
// The permutation matrix is implicit in the formula.
// No data moves; the READ PATTERN changes.

/// Apply stride-s permutation on a state of width W.
///
/// out[i] = state[(i × stride) mod width]
///
/// Precondition: gcd(stride, width) = 1 (INVARIANT 10).
pub fn sponge_permute(state: &[u8], stride: usize, out: &mut [u8]) {
    let w = state.len();
    debug_assert_eq!(w, out.len());
    debug_assert_eq!(gcd_usize(stride, w), 1,
        "stride {} must be coprime to width {} (INVARIANT 10)", stride, w);

    for i in 0..w {
        out[i] = state[(i * stride) % w];
    }
}

/// Inverse permutation: out[(i × stride) mod W] = state[i]
/// Equivalently: out[i] = state[(i × stride_inv) mod W]
/// where stride_inv is the modular inverse of stride mod W.
pub fn sponge_permute_inv(state: &[u8], stride: usize, out: &mut [u8]) {
    let w = state.len();
    debug_assert_eq!(w, out.len());
    let stride_inv = mod_inverse(stride, w)
        .expect("stride must be coprime to width for inverse to exist");

    for i in 0..w {
        out[i] = state[(i * stride_inv) % w];
    }
}

/// Precompute the permutation indices for repeated application.
/// Returns a vector where result[i] = (i × stride) mod width.
/// Apply as: out[i] = state[perm[i]]
pub fn precompute_permutation(stride: usize, width: usize) -> Vec<usize> {
    debug_assert_eq!(gcd_usize(stride, width), 1);
    (0..width).map(|i| (i * stride) % width).collect()
}

/// Apply a precomputed permutation.
pub fn apply_permutation(state: &[u8], perm: &[usize], out: &mut [u8]) {
    debug_assert_eq!(state.len(), perm.len());
    debug_assert_eq!(state.len(), out.len());
    for i in 0..state.len() {
        out[i] = state[perm[i]];
    }
}

// ============================================================
// SPONGE DIFFUSION — Circulant Neighbor Sum
// ============================================================
//
// tisTheta: each element receives the GF(3) sum of itself
// and its two neighbors (circular).
//
// out[i] = (state[i-1] + state[i] + state[i+1]) mod 3
//
// This is a circulant convolution with kernel [1, 1, 1].
// For the 54-element TIS-27 state, this runs in O(n).

/// Apply theta diffusion: out[i] = (left + center + right) mod 3
///
/// Circular boundary: state[-1] = state[W-1], state[W] = state[0].
pub fn sponge_theta(state: &[u8], out: &mut [u8]) {
    let w = state.len();
    debug_assert_eq!(w, out.len());

    for i in 0..w {
        let left   = state[(i + w - 1) % w];
        let center = state[i];
        let right  = state[(i + 1) % w];
        out[i] = (left + center + right) % 3;
    }
}

/// Apply theta diffusion with round constant addition.
///
/// out[i] = (left + center + right + constant[i]) mod 3
pub fn sponge_theta_with_constant(state: &[u8], constants: &[u8], out: &mut [u8]) {
    let w = state.len();
    debug_assert_eq!(w, out.len());
    debug_assert!(constants.len() >= w);

    for i in 0..w {
        let left   = state[(i + w - 1) % w];
        let center = state[i];
        let right  = state[(i + 1) % w];
        out[i] = (left + center + right + constants[i]) % 3;
    }
}

// ============================================================
// FULL TIS-27 SPONGE ROUND
// ============================================================

/// TIS-27 round constants (27 GF(3) values).
pub const TIS27_ROUND_CONSTANTS: [u8; 27] = [
    0, 0, 1, 1, 2, 1, 1, 1, 0, 2, 0, 2, 1, 0, 0, 1, 1, 2, 1, 1, 1, 0, 2, 0, 2, 1, 0
];

/// TIS-27 sponge parameters.
pub const TIS27_STATE_WIDTH: usize = 54;
pub const TIS27_RATE: usize = 27;
pub const TIS27_CAPACITY: usize = 27;
pub const TIS27_ROUNDS: usize = 27;
pub const TIS27_STRIDE: usize = 13;

/// One full TIS-27 sponge round: theta → pi → round constant addition.
///
/// All arithmetic in GF(3). No binary hash primitives.
pub fn tis27_round(state: &mut [u8; TIS27_STATE_WIDTH], round: usize) {
    let mut temp = [0u8; TIS27_STATE_WIDTH];

    // Step 1: Theta (neighbor diffusion)
    sponge_theta(state, &mut temp);

    // Step 2: Pi (stride-13 permutation)
    // Apply permutation: state[i] = temp[(i * 13) mod 54]
    sponge_permute(&temp, TIS27_STRIDE, state);

    // Step 3: Round constant addition (first 27 elements only = rate portion)
    let rc_idx = round % TIS27_ROUND_CONSTANTS.len();
    for i in 0..TIS27_RATE {
        state[i] = gf3_add(state[i], TIS27_ROUND_CONSTANTS[(i + rc_idx) % TIS27_ROUND_CONSTANTS.len()]);
    }
}

/// Full TIS-27 sponge: absorb input, squeeze output.
///
/// Input: arbitrary-length byte slice (will be decomposed to trits).
/// Output: `output_trits` GF(3) values.
pub fn tis27_sponge(input_trits: &[u8], output_len: usize) -> Vec<u8> {
    let mut state = [0u8; TIS27_STATE_WIDTH];

    // Absorb: XOR input trits into rate portion in blocks of 27
    let mut offset = 0;
    while offset < input_trits.len() {
        let block_len = std::cmp::min(TIS27_RATE, input_trits.len() - offset);
        for i in 0..block_len {
            state[i] = gf3_add(state[i], input_trits[offset + i]);
        }
        // Apply all rounds
        for round in 0..TIS27_ROUNDS {
            tis27_round(&mut state, round);
        }
        offset += TIS27_RATE;
    }

    // Squeeze: extract from rate portion
    let mut output = Vec::with_capacity(output_len);
    while output.len() < output_len {
        let take = std::cmp::min(TIS27_RATE, output_len - output.len());
        output.extend_from_slice(&state[..take]);
        if output.len() < output_len {
            for round in 0..TIS27_ROUNDS {
                tis27_round(&mut state, round);
            }
        }
    }
    output.truncate(output_len);
    output
}

// ============================================================
// REPUNIT CHECKSUM — Horner's Method mod R₆
// ============================================================
//
// 3⁶ ≡ 1 (mod 364): the checksum space has period 6.
// This means Horner evaluation of a base-3 number mod 364
// naturally wraps every 6 digits, keeping intermediates small.

/// Repunit checksum of a Rep C trit vector via Horner's method mod 364.
///
/// Interpret trits as a base-3 number (Rep B), reduce mod 364.
/// Returns a value in [0, 363].
pub fn repunit_checksum(trits_rep_c: &[u8]) -> u64 {
    let mut value: u64 = 0;
    for i in (0..trits_rep_c.len()).rev() {
        let trit_b = (trits_rep_c[i] - 1) as u64; // Rep C → Rep B
        value = (value * 3 + trit_b) % 364;
    }
    value
}

// ============================================================
// DERIVATION — project_to_gf3 (INVARIANT 2)
// ============================================================
//
// gf3 = min(floor(3k / N), 2)
// trit = gf3 + 1 (lift to Rep C)
//
// Boundaries fall at exactly N/3 and 2N/3. No tuning parameters.

/// Universal derivation: signal count → GF(3) value.
/// INVARIANT 2: the formula is mathematical, not empirical.
pub fn project_to_gf3(k: u64, n: u64) -> u8 {
    debug_assert!(n > 0, "total signals N must be > 0");
    let gf3 = std::cmp::min(3 * k / n, 2);
    gf3 as u8
}

/// Derivation with lift to Rep C.
pub fn derive_trit(k: u64, n: u64) -> u8 {
    project_to_gf3(k, n) + 1
}

// ============================================================
// UTILITY
// ============================================================

fn gcd_usize(mut a: usize, mut b: usize) -> usize {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

fn mod_inverse(a: usize, m: usize) -> Option<usize> {
    // Extended Euclidean algorithm
    let (mut old_r, mut r) = (a as i64, m as i64);
    let (mut old_s, mut s) = (1i64, 0i64);

    while r != 0 {
        let q = old_r / r;
        let temp_r = r; r = old_r - q * r; old_r = temp_r;
        let temp_s = s; s = old_s - q * s; old_s = temp_s;
    }

    if old_r != 1 { return None; } // not coprime
    Some(((old_s % m as i64 + m as i64) % m as i64) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── GF(3) element operations ──

    #[test]
    fn test_gf3_add_table() {
        // Full 3×3 addition table
        let expected = [[0,1,2],[1,2,0],[2,0,1]];
        for a in 0..3u8 {
            for b in 0..3u8 {
                assert_eq!(gf3_add(a, b), expected[a as usize][b as usize],
                    "gf3_add({}, {}) failed", a, b);
            }
        }
    }

    #[test]
    fn test_gf3_mul_table() {
        let expected = [[0,0,0],[0,1,2],[0,2,1]];
        for a in 0..3u8 {
            for b in 0..3u8 {
                assert_eq!(gf3_mul(a, b), expected[a as usize][b as usize],
                    "gf3_mul({}, {}) failed", a, b);
            }
        }
    }

    #[test]
    fn test_gf3_square_is_hamming_indicator() {
        assert_eq!(gf3_square(0), 0); // same → 0
        assert_eq!(gf3_square(1), 1); // different → 1
        assert_eq!(gf3_square(2), 1); // different → 1
    }

    // ── Hamming distance ──

    #[test]
    fn test_hamming_identical() {
        let a = [0, 1, 2, 0, 1, 2];
        assert_eq!(hamming_distance(&a, &a), 0);
    }

    #[test]
    fn test_hamming_all_different() {
        let a = [0, 0, 0, 0, 0];
        let b = [1, 2, 1, 2, 1];
        assert_eq!(hamming_distance(&a, &b), 5);
    }

    #[test]
    fn test_hamming_rep_c_google_vs_pptpro() {
        // Google:  [2,3,2,3, 1,1,3,3, 3,1,3,1, 1,3,2,2, 2,3,3,1, 1,2,1,2, 3,1,3]
        // PPTPro:  [2,3,3,3, 2,3,3,3, 2,2,2,2, 3,3,3,3, 1,2,2,1, 2,1,3,3, 3,3,2]
        let google: [u8; 27] = [2,3,2,3, 1,1,3,3, 3,1,3,1, 1,3,2,2, 2,3,3,1, 1,2,1,2, 3,1,3];
        let pptpro: [u8; 27] = [2,3,3,3, 2,3,3,3, 2,2,2,2, 3,3,3,3, 1,2,2,1, 2,1,3,3, 3,3,2];

        let dist = hamming_distance_27(&google, &pptpro);
        // Count manually: positions where they differ
        let expected: u32 = google.iter().zip(pptpro.iter())
            .filter(|(&a, &b)| a != b).count() as u32;
        assert_eq!(dist, expected);
    }

    #[test]
    fn test_hamming_symmetry() {
        let a = [0, 1, 2, 0, 1];
        let b = [2, 1, 0, 2, 0];
        assert_eq!(hamming_distance(&a, &b), hamming_distance(&b, &a));
    }

    // ── Forgery detection ──

    #[test]
    fn test_no_forgery_all_valid() {
        let trits = [1, 2, 3, 1, 2, 3, 1, 2, 3];
        assert!(!has_forgery(&trits));
    }

    #[test]
    fn test_forgery_detected() {
        let trits = [1, 2, 0, 1, 2]; // position 2 is forged
        assert!(has_forgery(&trits));
    }

    #[test]
    fn test_forgery_at_position_0() {
        let trits = [0, 2, 3, 1];
        assert!(has_forgery(&trits));
        assert_eq!(find_forgeries(&trits), vec![0]);
    }

    #[test]
    fn test_multiple_forgeries() {
        let trits = [1, 0, 3, 0, 2];
        assert!(has_forgery(&trits));
        assert_eq!(find_forgeries(&trits), vec![1, 3]);
    }

    // ── Sponge permutation ──

    #[test]
    fn test_permute_stride_13_width_54() {
        let mut state: Vec<u8> = (0..54).collect();
        let mut out = vec![0u8; 54];
        sponge_permute(&state, 13, &mut out);

        // Verify: out[i] = state[(i*13) mod 54]
        for i in 0..54 {
            assert_eq!(out[i], state[(i * 13) % 54],
                "permute failed at position {}", i);
        }

        // Verify all 54 values present (complete cycle)
        let mut seen = vec![false; 54];
        for &v in &out { seen[v as usize] = true; }
        assert!(seen.iter().all(|&s| s), "permutation not complete");
    }

    #[test]
    fn test_permute_inverse_roundtrip() {
        let state: Vec<u8> = (0..54).map(|i| i % 3).collect();
        let mut permuted = vec![0u8; 54];
        let mut recovered = vec![0u8; 54];

        sponge_permute(&state, 13, &mut permuted);
        sponge_permute_inv(&permuted, 13, &mut recovered);

        assert_eq!(state, recovered, "permute → inv_permute must be identity");
    }

    // ── Sponge theta ──

    #[test]
    fn test_theta_all_zeros() {
        let state = vec![0u8; 54];
        let mut out = vec![0u8; 54];
        sponge_theta(&state, &mut out);
        assert!(out.iter().all(|&v| v == 0), "theta of zero state should be zero");
    }

    #[test]
    fn test_theta_single_one() {
        let mut state = vec![0u8; 6];
        state[2] = 1; // only position 2 is nonzero
        let mut out = vec![0u8; 6];
        sponge_theta(&state, &mut out);

        // Neighbors of position 2 are 1 and 3
        assert_eq!(out[1], 1); // left neighbor gets 1
        assert_eq!(out[2], 1); // center keeps its value
        assert_eq!(out[3], 1); // right neighbor gets 1
        assert_eq!(out[0], 0); // not a neighbor
    }

    // ── Repunit checksum ──

    #[test]
    fn test_checksum_all_ones() {
        let trits = [1u8; 27];
        assert_eq!(repunit_checksum(&trits), 0); // all Rep B = 0
    }

    #[test]
    fn test_checksum_mod_364_period() {
        // 3^6 ≡ 1 (mod 364)
        assert_eq!(729 % 364, 1);
    }

    // ── Derivation ──

    #[test]
    fn test_project_to_gf3_boundaries() {
        // N = 30: boundaries at 10 and 20
        assert_eq!(project_to_gf3(0, 30), 0);
        assert_eq!(project_to_gf3(9, 30), 0);
        assert_eq!(project_to_gf3(10, 30), 1);
        assert_eq!(project_to_gf3(19, 30), 1);
        assert_eq!(project_to_gf3(20, 30), 2);
        assert_eq!(project_to_gf3(30, 30), 2); // clamped at 2
    }

    // ── Mod inverse ──

    #[test]
    fn test_mod_inverse_13_54() {
        let inv = mod_inverse(13, 54).unwrap();
        assert_eq!((13 * inv) % 54, 1);
    }

    #[test]
    fn test_mod_inverse_13_28() {
        let inv = mod_inverse(13, 28).unwrap();
        assert_eq!(inv, 13); // self-inverse
    }
}
