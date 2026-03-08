// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Repunit Checksum — Lightweight Address Integrity
//
// Computes a 6-trit checksum of a 27-trit classification address
// using modular arithmetic against R₆ = 364 = 111111₃ (full ternary circle).
//
// The checksum is the address interpreted as a base-3 number, reduced mod 364.
// Result is a value in [0, 363], representable as 6 trits in Rep B {0,1,2}
// or 6 trits in Rep C {1,2,3} after lifting.
//
// This is a GF(3)-native integrity check with no domain crossing.
// It complements the full TIS-27 sponge integrity in the wire protocol
// with a fast, constant-time, branchless verification.

/// Full ternary circle: R₆ = (3⁶ - 1) / 2 = 364 = 111111₃
pub const REPUNIT_R6: u64 = 364;

/// Number of checksum trits (6 = digits in R₆ base-3 representation)
pub const CHECKSUM_TRIT_COUNT: usize = 6;

/// Compute the repunit checksum of a 27-trit classification address.
///
/// Input: `trits` — 27 trit values in Rep C {1, 2, 3}.
/// Output: 6-trit checksum in Rep C {1, 2, 3}.
///
/// Algorithm:
///   1. Convert each Rep C trit to Rep B: trit_b = trit_c - 1
///   2. Interpret as base-3 number: value = Σ(trit_b[i] × 3^i)
///   3. Reduce mod 364: checksum_value = value mod 364
///   4. Decompose into 6 Rep B trits, then lift to Rep C
///
/// The computation uses modular reduction at each step to prevent overflow.
/// For 27 trits in Rep B {0,1,2}, the maximum value is 2 × (3²⁷ - 1) / 2
/// which exceeds u64, so we reduce mod 364 incrementally using Horner's method.
pub fn compute_checksum_rep_c(trits: &[u8; 27]) -> [u8; CHECKSUM_TRIT_COUNT] {
    // Validate: all trits must be Rep C {1, 2, 3}
    for (i, &t) in trits.iter().enumerate() {
        assert!(
            t >= 1 && t <= 3,
            "Trit {} has invalid Rep C value {}: must be 1, 2, or 3 (zero = forgery)",
            i, t
        );
    }

    // Horner's method with incremental mod reduction:
    // value = trit_b[26] * 3^26 + ... + trit_b[1] * 3 + trit_b[0]
    //       = (((...(trit_b[26]) * 3 + trit_b[25]) * 3 + ...) * 3 + trit_b[0])
    // We process MSB first (index 26 down to 0) and reduce mod 364 at each step.
    let mut value: u64 = 0;
    for i in (0..27).rev() {
        let trit_b = (trits[i] - 1) as u64; // Rep C → Rep B
        value = (value * 3 + trit_b) % REPUNIT_R6;
    }

    // Decompose value into 6 Rep B trits (LSB first), then lift to Rep C
    let mut checksum = [0u8; CHECKSUM_TRIT_COUNT];
    let mut remaining = value;
    for trit in checksum.iter_mut() {
        let digit = (remaining % 3) as u8;
        *trit = digit + 1; // Rep B → Rep C
        remaining /= 3;
    }

    // Special case: if value is 0, all Rep B digits are 0, all Rep C are 1.
    // This is valid — [1,1,1,1,1,1] in Rep C = 0 mod 364.
    checksum
}

/// Verify that a 27-trit address matches its 6-trit checksum.
///
/// Returns true iff the checksum matches.
pub fn verify_checksum(trits: &[u8; 27], expected_checksum: &[u8; CHECKSUM_TRIT_COUNT]) -> bool {
    let computed = compute_checksum_rep_c(trits);
    computed == *expected_checksum
}

/// Compute checksum and return as a raw u64 value in [0, 363].
pub fn compute_checksum_raw(trits: &[u8; 27]) -> u64 {
    for (i, &t) in trits.iter().enumerate() {
        assert!(t >= 1 && t <= 3, "Trit {} invalid: {}", i, t);
    }

    let mut value: u64 = 0;
    for i in (0..27).rev() {
        let trit_b = (trits[i] - 1) as u64;
        value = (value * 3 + trit_b) % REPUNIT_R6;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_all_ones() {
        // Address: all trits = 1 (Rep C) → all Rep B = 0 → value = 0
        let trits = [1u8; 27];
        let raw = compute_checksum_raw(&trits);
        assert_eq!(raw, 0, "all-1 address should checksum to 0");

        let checksum = compute_checksum_rep_c(&trits);
        assert_eq!(checksum, [1, 1, 1, 1, 1, 1], "0 in Rep C is [1,1,1,1,1,1]");
        assert!(verify_checksum(&trits, &checksum));
    }

    #[test]
    fn test_checksum_all_threes() {
        // Address: all trits = 3 (Rep C) → all Rep B = 2
        // value = 2 × (3^0 + 3^1 + ... + 3^26) = 2 × (3^27 - 1)/2 = 3^27 - 1
        // (3^27 - 1) mod 364:
        // 3^6 = 729, 729 mod 364 = 1 (since 729 = 2×364 + 1)
        // So 3^6 ≡ 1 (mod 364).
        // 3^27 = 3^(6×4+3) = (3^6)^4 × 3^3 ≡ 1^4 × 27 = 27 (mod 364)
        // 3^27 - 1 ≡ 26 (mod 364)
        let trits = [3u8; 27];
        let raw = compute_checksum_raw(&trits);
        assert_eq!(raw, 26, "all-3 address should checksum to 26");
        assert!(verify_checksum(&trits, &compute_checksum_rep_c(&trits)));
    }

    #[test]
    fn test_checksum_google_fixture() {
        // Google: WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313
        // Dims 1-27: [2,3,2,3, 1,1,3,3, 3,1,3,1, 1,3,2,2, 2,3,3,1, 1,2,1,2, 3,1,3]
        let google: [u8; 27] = [
            2, 3, 2, 3,  // WHO
            1, 1, 3, 3,  // WHAT
            3, 1, 3, 1,  // WHERE
            1, 3, 2, 2,  // WHEN
            2, 3, 3, 1,  // WHY
            1, 2, 1, 2,  // HOW
            3, 1, 3,     // PEACE
        ];
        let checksum = compute_checksum_rep_c(&google);
        // Verify round-trip
        assert!(verify_checksum(&google, &checksum));
        // Verify all checksum trits are valid Rep C
        for &t in &checksum {
            assert!(t >= 1 && t <= 3, "checksum trit out of Rep C range: {}", t);
        }
    }

    #[test]
    fn test_checksum_pptpro_fixture() {
        // PPTPro: WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332
        let pptpro: [u8; 27] = [
            2, 3, 3, 3,  // WHO
            2, 3, 3, 3,  // WHAT
            2, 2, 2, 2,  // WHERE
            3, 3, 3, 3,  // WHEN (trits 15+16 = 3,3 → HPTP-mandatory)
            1, 2, 2, 1,  // WHY
            2, 1, 3, 3,  // HOW
            3, 3, 2,     // PEACE
        ];
        let checksum = compute_checksum_rep_c(&pptpro);
        assert!(verify_checksum(&pptpro, &checksum));
    }

    #[test]
    #[should_panic(expected = "zero = forgery")]
    fn test_checksum_rejects_zero_trit() {
        // Zero in Rep C is invalid — structural forgery detection
        let mut trits = [1u8; 27];
        trits[5] = 0; // inject zero
        compute_checksum_rep_c(&trits); // should panic
    }

    #[test]
    fn test_checksum_detects_single_trit_flip() {
        let mut trits_a = [2u8; 27];
        trits_a[0] = 1;
        let checksum_a = compute_checksum_raw(&trits_a);

        let mut trits_b = [2u8; 27];
        trits_b[0] = 3; // flip trit 0 from 1 to 3
        let checksum_b = compute_checksum_raw(&trits_b);

        assert_ne!(
            checksum_a, checksum_b,
            "single trit flip should change checksum"
        );
    }

    #[test]
    fn test_3_to_6_mod_364_is_1() {
        // Key property: 3^6 ≡ 1 (mod 364), since 3^6 = 729 = 2×364 + 1
        // This means the checksum has period 6 in the exponent,
        // which is why 6 trits perfectly represent the checksum space.
        assert_eq!(729 % 364, 1, "3^6 mod 364 must equal 1");
    }

    #[test]
    fn test_repunit_r6_equals_circle() {
        assert_eq!(REPUNIT_R6, 364);
        assert_eq!(REPUNIT_R6, (729 - 1) / 2); // (3^6 - 1) / 2
    }
}
