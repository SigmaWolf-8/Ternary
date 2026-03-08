// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Coprime Clock Source Rotation for HPTP
//
// Formalizes the generator theorem for HPTP clock failover:
// When rotating through N active clock sources, the rotation step
// must be coprime to N to guarantee every source is consulted
// before any source is revisited.
//
// For N = 7 (current HPTP: Local, GPSDO, Atomic Rb, Atomic Cs,
// Optical Lattice, Chip-Scale, Network Peer), since 7 is prime,
// ALL steps 1..6 produce complete walks. This is maximally robust.
//
// If the source count ever changes to a non-prime (e.g., 6, 8, 9),
// the coprime constraint becomes load-bearing: only φ(N) of the
// N-1 candidate steps will produce complete coverage.
//
// This module provides:
//   - validate_rotation_step(): assert coprime guarantee
//   - valid_rotation_steps(): enumerate all valid steps for N sources
//   - select_optimal_step(): choose step closest to T₇ = 13 (or its residue mod N)

/// Compute gcd (no std).
pub const fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Euler's totient function for small values.
pub fn euler_totient(n: u64) -> u64 {
    (1..n).filter(|&a| gcd(a, n) == 1).count() as u64
}

/// The 7 HPTP clock sources.
/// Since 7 is prime, all rotation steps 1..6 are valid generators.
pub const HPTP_SOURCE_COUNT: u64 = 7;

/// Clock source identifiers (matching kernel HPTP protocol.rs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSource {
    Local = 0,
    Gpsdo = 1,
    AtomicRubidium = 2,
    AtomicCesium = 3,
    OpticalLattice = 4,
    ChipScale = 5,
    NetworkPeer = 6,
}

impl ClockSource {
    pub const ALL: [ClockSource; 7] = [
        ClockSource::Local,
        ClockSource::Gpsdo,
        ClockSource::AtomicRubidium,
        ClockSource::AtomicCesium,
        ClockSource::OpticalLattice,
        ClockSource::ChipScale,
        ClockSource::NetworkPeer,
    ];
}

/// Validate that a rotation step produces a complete walk through N sources.
///
/// # Panics
/// If `gcd(step, source_count) != 1` — the rotation would miss sources.
pub fn validate_rotation_step(step: u64, source_count: u64) {
    let g = gcd(step, source_count);
    assert_eq!(
        g, 1,
        "HPTP rotation step {} is NOT coprime to source count {} (gcd={}). \
         This would fragment the consultation cycle into {} disjoint sub-cycles, \
         leaving {} sources never consulted. \
         Valid steps: {:?}",
        step, source_count, g,
        g,
        source_count - source_count / g,
        valid_rotation_steps(source_count)
    );
}

/// Return all valid rotation steps for N clock sources.
/// A step is valid iff gcd(step, N) = 1.
pub fn valid_rotation_steps(source_count: u64) -> Vec<u64> {
    (1..source_count)
        .filter(|&s| gcd(s, source_count) == 1)
        .collect()
}

/// Select the optimal rotation step for N clock sources.
///
/// Strategy: prefer the step closest to T₇ = 13 (the canonical framework constant).
/// If N < 13, use 13 mod N (if coprime). If that's not coprime, find the
/// nearest coprime step.
///
/// For N = 7: 13 mod 7 = 6, gcd(6, 7) = 1 ✓ → step = 6.
/// For N = 6: 13 mod 6 = 1, gcd(1, 6) = 1 ✓ → step = 1.
/// For N = 9: 13 mod 9 = 4, gcd(4, 9) = 1 ✓ → step = 4.
pub fn select_optimal_step(source_count: u64) -> u64 {
    if source_count <= 1 {
        return 1;
    }

    // First choice: T₇ mod N
    let candidate = 13 % source_count;
    if candidate > 0 && gcd(candidate, source_count) == 1 {
        return candidate;
    }

    // Fallback: search outward from candidate
    let valid = valid_rotation_steps(source_count);
    if valid.is_empty() {
        return 1; // degenerate case
    }

    // Find step closest to 13 mod N
    let target = if candidate == 0 { source_count / 2 } else { candidate };
    *valid.iter()
        .min_by_key(|&&s| (s as i64 - target as i64).unsigned_abs())
        .unwrap()
}

/// Generate the clock source consultation order for a given step.
///
/// Returns the sequence of source indices visited, starting from source 0.
/// Guaranteed to visit all N sources exactly once if step is coprime to N.
pub fn consultation_order(step: u64, source_count: u64) -> Vec<u64> {
    validate_rotation_step(step, source_count);
    (0..source_count)
        .map(|k| (k * step) % source_count)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seven_is_prime_all_steps_valid() {
        // N = 7 (prime): all steps 1..6 are coprime to 7.
        let valid = valid_rotation_steps(7);
        assert_eq!(valid, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(euler_totient(7), 6);
    }

    #[test]
    fn test_validate_step_for_current_hptp() {
        // All steps 1..6 are valid for 7 sources
        for step in 1..HPTP_SOURCE_COUNT {
            validate_rotation_step(step, HPTP_SOURCE_COUNT);
        }
    }

    #[test]
    fn test_optimal_step_for_7_sources() {
        // 13 mod 7 = 6, gcd(6, 7) = 1 → step 6
        assert_eq!(select_optimal_step(7), 6);
    }

    #[test]
    fn test_consultation_order_complete() {
        let order = consultation_order(6, 7);
        assert_eq!(order.len(), 7);
        let mut sorted = order.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_non_prime_source_count() {
        // If we ever had 6 sources: φ(6) = 2, only steps {1, 5} work.
        let valid = valid_rotation_steps(6);
        assert_eq!(valid, vec![1, 5]);
        assert_eq!(euler_totient(6), 2);
    }

    #[test]
    #[should_panic(expected = "NOT coprime")]
    fn test_invalid_step_panics() {
        // Step 2 for 6 sources: gcd(2, 6) = 2 → should panic
        validate_rotation_step(2, 6);
    }

    #[test]
    fn test_nine_sources() {
        // φ(9) = 6: steps {1, 2, 4, 5, 7, 8}
        let valid = valid_rotation_steps(9);
        assert_eq!(valid, vec![1, 2, 4, 5, 7, 8]);
        // Optimal: 13 mod 9 = 4, gcd(4, 9) = 1 ✓
        assert_eq!(select_optimal_step(9), 4);
    }

    #[test]
    fn test_documentation_property() {
        // Key documentation point: for N = 7 (prime), the system is
        // maximally fault-tolerant because ANY non-zero step works.
        // If a clock source fails and we need to skip it (changing N to 6),
        // the valid step count drops from 6 to 2.
        // This demonstrates why 7 sources is a better architectural choice
        // than 6 or 8 — primality maximizes rotational flexibility.
        assert!(is_prime(7));
        assert!(!is_prime(6));
        assert!(!is_prime(8));
        assert_eq!(euler_totient(7), 6); // 6 of 6 steps work
        assert_eq!(euler_totient(6), 2); // 2 of 5 steps work
        assert_eq!(euler_totient(8), 4); // 4 of 7 steps work
    }

    fn is_prime(n: u64) -> bool {
        if n < 2 { return false; }
        if n < 4 { return true; }
        if n % 2 == 0 { return false; }
        let mut i = 3u64;
        while i * i <= n {
            if n % i == 0 { return false; }
            i += 2;
        }
        true
    }
}
