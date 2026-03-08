// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Generator Theorem Proof Harness
// Formally verifies: step `a` generates Z_m iff gcd(a, m) = 1
// Covers all framework moduli: {13, 27, 28, 54, 364}
// Also verifies CRT product groups for coprime moduli.

/// Compute gcd using Euclidean algorithm (no std dependency).
const fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Check that the walk {k*a mod m | k in 0..m} visits all m residues.
/// Returns true iff the walk is a complete permutation of Z_m.
fn is_complete_walk(a: u64, m: u64) -> bool {
    assert!(m > 0 && m <= 1024, "modulus out of safe range");
    let mut visited = vec![false; m as usize];
    for k in 0..m {
        let pos = ((k as u128 * a as u128) % m as u128) as usize;
        if visited[pos] {
            return false; // revisit before completing
        }
        visited[pos] = true;
    }
    // All positions should now be visited
    visited.iter().all(|&v| v)
}

// ============================================================
// FRAMEWORK MODULI — exhaustive verification
// ============================================================

/// All moduli used across the Salvi Framework.
/// 13: T₇, 1 radian, agent sub-arrays, TDNS scanner dimension count (via 27)
/// 27: TDNS dimensions (3³)
/// 28: Z₂₈ agent array, 2π radians
/// 54: TIS-27 sponge state width
/// 364: Full ternary circle R₆ = 111111₃
const FRAMEWORK_MODULI: &[u64] = &[13, 27, 28, 54, 364];

#[test]
fn test_generator_theorem_exhaustive() {
    // For each framework modulus, verify:
    //   gcd(a, m) = 1  ⟹  walk is complete
    //   gcd(a, m) > 1  ⟹  walk is NOT complete
    for &m in FRAMEWORK_MODULI {
        let mut generator_count = 0u64;
        for a in 1..m {
            let coprime = gcd(a, m) == 1;
            let complete = is_complete_walk(a, m);
            assert_eq!(
                coprime, complete,
                "Generator theorem VIOLATED: a={}, m={}, gcd={}, coprime={}, complete={}",
                a, m, gcd(a, m), coprime, complete
            );
            if coprime {
                generator_count += 1;
            }
        }
        // Verify Euler's totient: number of generators = φ(m)
        let phi = euler_totient(m);
        assert_eq!(
            generator_count, phi,
            "Totient mismatch for m={}: counted {} generators, expected φ({})={}",
            m, generator_count, m, phi
        );
    }
}

/// Euler's totient function (brute force, safe for m ≤ 1024).
fn euler_totient(m: u64) -> u64 {
    (1..m).filter(|&a| gcd(a, m) == 1).count() as u64
}

// ============================================================
// SPECIFIC FRAMEWORK CONSTANTS — named assertions
// ============================================================

#[test]
fn test_stride_13_generates_z54() {
    // INVARIANT 10: TIS-27 sponge stride
    assert_eq!(gcd(13, 54), 1, "stride 13 must be coprime to state width 54");
    assert!(is_complete_walk(13, 54), "stride 13 must produce complete permutation of Z_54");
}

#[test]
fn test_step_13_generates_z28() {
    // Agent array scheduling: (position × 13) mod 28
    assert_eq!(gcd(13, 28), 1, "step 13 must be coprime to 28");
    assert!(is_complete_walk(13, 28), "step 13 must visit all 28 agents");
}

#[test]
fn test_all_12_generators_of_z28() {
    // The 12 generators of Z₂₈ (units mod 28, φ(28) = 12)
    let expected_generators: &[u64] = &[1, 3, 5, 9, 11, 13, 15, 17, 19, 23, 25, 27];
    assert_eq!(expected_generators.len(), 12);
    assert_eq!(euler_totient(28), 12);

    for &g in expected_generators {
        assert_eq!(gcd(g, 28), 1, "expected generator {} is not coprime to 28", g);
        assert!(is_complete_walk(g, 28), "generator {} does not produce complete walk on Z_28", g);
    }

    // Verify no other values in 1..28 are generators
    for a in 1..28u64 {
        if !expected_generators.contains(&a) {
            assert_ne!(gcd(a, 28), 1, "non-generator {} should not be coprime to 28", a);
            assert!(!is_complete_walk(a, 28), "non-generator {} should not complete Z_28", a);
        }
    }
}

#[test]
fn test_generator_inverse_pairs_z28() {
    // For each generator g of Z₂₈, verify g * g⁻¹ ≡ 1 (mod 28)
    // and that g⁻¹ is also a generator.
    let generators: &[u64] = &[1, 3, 5, 9, 11, 13, 15, 17, 19, 23, 25, 27];
    let inverses:   &[u64] = &[1, 19, 17, 25, 23, 13, 15, 5, 3, 11, 9, 27];

    for (&g, &g_inv) in generators.iter().zip(inverses.iter()) {
        assert_eq!(
            (g * g_inv) % 28, 1,
            "inverse of {} should be {}, but {}*{} mod 28 = {}",
            g, g_inv, g, g_inv, (g * g_inv) % 28
        );
        assert!(is_complete_walk(g_inv, 28), "inverse {} must also be a generator", g_inv);
    }

    // Document self-inverse generators: g where g = g⁻¹
    let self_inverse: Vec<u64> = generators.iter()
        .zip(inverses.iter())
        .filter(|(&g, &gi)| g == gi)
        .map(|(&g, _)| g)
        .collect();
    assert_eq!(self_inverse, vec![1, 13, 15, 27], "self-inverse generators of Z_28");
}

// ============================================================
// CRT PRODUCT GROUP — Chinese Remainder Theorem
// ============================================================

#[test]
fn test_crt_product_walk_z28_times_z13() {
    // Calendar structure: Z₂₈ × Z₁₃ ≅ Z₃₆₄ (since gcd(28,13)=1)
    // Step (13, 1) on the product torus should visit all 364 points.
    assert_eq!(gcd(28, 13), 1, "28 and 13 must be coprime for CRT isomorphism");

    let m = 28u64;
    let n = 13u64;
    let step_m = 13u64;
    let step_n = 1u64;

    assert_eq!(gcd(step_m, m), 1, "step_m must be coprime to m");
    assert_eq!(gcd(step_n, n), 1, "step_n must be coprime to n");

    // Via CRT: the combined step in Z₃₆₄ is the unique s such that
    // s ≡ step_m (mod m) and s ≡ step_n (mod n).
    // s ≡ 13 (mod 28) and s ≡ 1 (mod 13) → s = 13 (since 13 mod 13 = 0... wait)
    // Actually: 13 mod 13 = 0, so step_n=1 means we want s ≡ 1 (mod 13).
    // CRT: find s in [0, 364) such that s ≡ 13 (mod 28) and s ≡ 1 (mod 13).
    // s = 13 → 13 mod 13 = 0 ≠ 1. So s ≠ 13.
    // s = 13 + 28 = 41 → 41 mod 13 = 2 ≠ 1.
    // s = 41 + 28 = 69 → 69 mod 13 = 4 ≠ 1.
    // s = 69 + 28 = 97 → 97 mod 13 = 6 ≠ 1.
    // s = 97 + 28 = 125 → 125 mod 13 = 8 ≠ 1.
    // s = 125 + 28 = 153 → 153 mod 13 = 10 ≠ 1.
    // s = 153 + 28 = 181 → 181 mod 13 = 12 ≠ 1.
    // s = 181 + 28 = 209 → 209 mod 13 = 1 ✓.
    // So s = 209 is the CRT combined step.
    let combined_step = 209u64;
    assert_eq!(combined_step % m, step_m, "CRT: combined step mod 28 should be 13");
    assert_eq!(combined_step % n, step_n, "CRT: combined step mod 13 should be 1");
    assert_eq!(gcd(combined_step, m * n), 1, "combined step must be coprime to 364");
    assert!(is_complete_walk(combined_step, 364), "combined step 209 must generate Z_364");

    // Also verify directly on the 2D torus
    let total = m * n;
    let mut visited = vec![false; total as usize];
    for k in 0..total {
        let x = (k * step_m) % m;
        let y = (k * step_n) % n;
        let idx = (x * n + y) as usize;
        assert!(!visited[idx], "2D torus revisit at k={}, ({},{})", k, x, y);
        visited[idx] = true;
    }
    assert!(visited.iter().all(|&v| v), "2D torus walk must visit all 364 points");
}

#[test]
fn test_crt_general_coprime_pairs() {
    // For pairs of coprime framework moduli, verify CRT isomorphism
    let pairs: &[(u64, u64)] = &[
        (13, 27), // gcd = 1? 13 and 27: 27 = 2*13 + 1, gcd(13,1)=1 ✓
        (13, 28), // gcd = 1 ✓ (calendar)
        (27, 28), // gcd = 1 ✓
    ];

    for &(m, n) in pairs {
        assert_eq!(gcd(m, n), 1, "({}, {}) must be coprime for CRT", m, n);
        let product = m * n;
        // Any step coprime to the product generates the product group
        let step = 13u64; // try 13 first
        if gcd(step, product) == 1 {
            assert!(
                is_complete_walk(step, product),
                "step {} should generate Z_{} (= Z_{} × Z_{})",
                step, product, m, n
            );
        }
    }
}

// ============================================================
// REPUNIT CIRCLE PROPERTIES
// ============================================================

/// Base-3 repunit: R(n) = (3^n - 1) / 2
const fn repunit(n: u32) -> u64 {
    let mut pow3: u64 = 1;
    let mut i = 0;
    while i < n {
        pow3 *= 3;
        i += 1;
    }
    (pow3 - 1) / 2
}

#[test]
fn test_repunit_values() {
    assert_eq!(repunit(1), 1);    // 1₃ = 1
    assert_eq!(repunit(2), 4);    // 11₃ = 4
    assert_eq!(repunit(3), 13);   // 111₃ = 13 = T₇
    assert_eq!(repunit(4), 40);   // 1111₃ = 40
    assert_eq!(repunit(5), 121);  // 11111₃ = 121
    assert_eq!(repunit(6), 364);  // 111111₃ = 364 = full circle
    assert_eq!(repunit(7), 1093); // 1111111₃ = 1093
    assert_eq!(repunit(8), 3280);
    assert_eq!(repunit(9), 9841);
}

#[test]
fn test_repunit_factorization() {
    // R(2n) = R(n) × (3^n + 1)
    for n in 1..=4u32 {
        let r_n = repunit(n);
        let r_2n = repunit(2 * n);
        let pow3_n: u64 = 3u64.pow(n);
        assert_eq!(
            r_2n, r_n * (pow3_n + 1),
            "R(2×{}) = R({}) × (3^{}+1): {} ≠ {} × {}",
            n, n, n, r_2n, r_n, pow3_n + 1
        );
    }
}

#[test]
fn test_step_13_on_repunit_circles() {
    // Verify gcd(13, R(n)) for each repunit to determine if 13 generates it
    let repunits: &[(u32, u64, bool)] = &[
        (3, 13, false),   // gcd(13,13) = 13 — 13 does NOT generate Z_13
        (4, 40, true),    // gcd(13,40) = 1 ✓
        (5, 121, true),   // gcd(13,121) = 1 ✓ (121 = 11², gcd(13,11)=1)
        (6, 364, false),  // gcd(13,364) = 13 — 364 = 13 × 28, NOT coprime
        (7, 1093, true),  // gcd(13,1093) = 1 ✓ (1093 is prime)
    ];

    for &(n, r, expected_coprime) in repunits {
        let actual = gcd(13, r) == 1;
        assert_eq!(
            actual, expected_coprime,
            "gcd(13, R({})) = gcd(13, {}) = {}: coprime={}",
            n, r, gcd(13, r), actual
        );
    }

    // IMPORTANT: 13 does NOT generate Z_364 because 364 = 13 × 28.
    // For Z_364, valid generators must be coprime to 364 = 2² × 7 × 13.
    // Example: step 11 (gcd(11, 364) = 1).
    assert_eq!(gcd(11, 364), 1);
    assert!(is_complete_walk(11, 364));
}
