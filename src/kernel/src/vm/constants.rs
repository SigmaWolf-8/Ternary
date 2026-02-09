/// Tribonacci-Derived VM Constants
///
/// Links the Ternary Virtual Machine configuration to the Tribonacci constant
/// τ ≈ 1.8392867552141612 (real root of τ³ = τ² + τ + 1), derived from SO(8)
/// quantum graph stability in the Unified 13D Torsion Plenum Theory.
///
/// These constants are the Rust equivalents of the TypeScript VM_CONSTANTS in
/// shared/tribonacci-constants.ts. Keep them synchronized.

/// The Tribonacci constant τ — real root of τ³ = τ² + τ + 1
pub const TAU: f64 = 1.8392867552141612;

/// τ² ≈ 3.3830
pub const TAU_2: f64 = 3.3830237514006;
/// τ³ ≈ 6.2223
pub const TAU_3: f64 = 6.222310506614761;
/// τ⁵ ≈ 21.059
pub const TAU_5: f64 = 21.05866186688145;
/// τ⁷ ≈ 71.21
pub const TAU_7: f64 = 71.21024218846628;
/// τ¹³ ≈ 2757.038
pub const TAU_13: f64 = 2757.0383547068004;

/// 27 registers = 3³ — the number of ternary register slots
/// Matches RegisterFile::registers array size
pub const REGISTER_COUNT: usize = 27;

/// Default stack size in bytes
pub const DEFAULT_STACK_SIZE: usize = 4096;

/// Maximum execution cycles before forced halt
pub const MAX_CYCLES: u64 = 1_000_000;

/// Hash seed derived from τ² — used for tribonacciHash SEED
/// floor(τ² × 10⁹) = 3_383_023_751
pub const HASH_SEED: u64 = 3_383_023_751;

/// Hash mix constant derived from τ⁷ — used for tribonacciHash MIX
/// floor(τ⁷ × 10⁶) = 71_210_242
pub const HASH_MIX: u64 = 71_210_242;

/// Number of finalization rounds in tribonacciHash — T(7) = 13
pub const HASH_ROUNDS: u32 = 13;

/// GC threshold ratio derived from τ⁻² ≈ 0.2956
/// When heap usage exceeds this fraction of capacity, trigger collection
pub const GC_THRESHOLD_RATIO: f64 = 0.29564989581955394;

/// Instruction cache size derived from τ⁵ × 4 ≈ 84
pub const INSTRUCTION_CACHE_SIZE: usize = 84;

/// Trit buffer size derived from τ⁷ × 2 ≈ 142
pub const TRIT_BUFFER_SIZE: usize = 142;

/// log₂(3) ≈ 1.585 — information density advantage per trit
pub const LOG2_3: f64 = 1.5849625007211563;

/// 59% density advantage: (log₂(3) - 1) × 100
pub const DENSITY_ADVANTAGE_PCT: f64 = 58.49625007211563;

/// Known Tribonacci sequence values T(0)..T(15)
pub const TRIBONACCI_TABLE: [u64; 16] = [
    0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81, 149, 274, 504, 927, 1705,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tau_identity() {
        let lhs = TAU * TAU * TAU;
        let rhs = TAU * TAU + TAU + 1.0;
        assert!((lhs - rhs).abs() < 1e-10, "τ³ ≠ τ² + τ + 1");
    }

    #[test]
    fn test_register_count_is_3_cubed() {
        assert_eq!(REGISTER_COUNT, 3_usize.pow(3));
    }

    #[test]
    fn test_hash_rounds_is_t7() {
        assert_eq!(HASH_ROUNDS, TRIBONACCI_TABLE[7] as u32);
    }

    #[test]
    fn test_hash_seed_from_tau2() {
        let expected = (TAU_2 * 1e9) as u64;
        assert_eq!(HASH_SEED, expected);
    }

    #[test]
    fn test_hash_mix_from_tau7() {
        let expected = (TAU_7 * 1e6) as u64;
        assert_eq!(HASH_MIX, expected);
    }

    #[test]
    fn test_gc_ratio_is_tau_neg2() {
        let expected = 1.0 / (TAU * TAU);
        assert!((GC_THRESHOLD_RATIO - expected).abs() < 1e-10);
    }

    #[test]
    fn test_tribonacci_table() {
        for i in 3..16 {
            assert_eq!(
                TRIBONACCI_TABLE[i],
                TRIBONACCI_TABLE[i - 1] + TRIBONACCI_TABLE[i - 2] + TRIBONACCI_TABLE[i - 3],
                "T({}) should equal T({})+T({})+T({})",
                i, i - 1, i - 2, i - 3
            );
        }
    }

    #[test]
    fn test_density_advantage() {
        let log2_3 = (3.0_f64).log2();
        assert!((LOG2_3 - log2_3).abs() < 1e-10);
        assert!((DENSITY_ADVANTAGE_PCT - (log2_3 - 1.0) * 100.0).abs() < 1e-6);
    }
}
