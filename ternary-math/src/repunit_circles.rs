// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Repunit Circle Constants — Rust Kernel Mirror
// Mirrors: shared/repunit-circles.ts
//
// Base-3 repunits R(n) = (3^n - 1) / 2 define geometric cycle lengths.
// These are circle-days (pure geometry), NOT calendar days.
// Calendar conversion requires DOT (Day Out of Time) insertion.

/// Compute base-3 repunit: R(n) = (3^n - 1) / 2
pub const fn repunit(n: u32) -> u64 {
    let mut pow3: u64 = 1;
    let mut i = 0;
    while i < n {
        pow3 *= 3;
        i += 1;
    }
    (pow3 - 1) / 2
}

// Named constants — derived from constants.rs source of truth.
pub const REPUNIT_R3: u64 = crate::constants::T_REPUNIT_3.host_u32() as u64;
pub const REPUNIT_R4: u64 = crate::constants::T_REPUNIT_4.host_u32() as u64;
pub const REPUNIT_R5: u64 = crate::constants::T_REPUNIT_5.host_u32() as u64;
pub const REPUNIT_R6: u64 = crate::constants::T_REPUNIT_6.host_u32() as u64;
pub const REPUNIT_R7: u64 = crate::trit_int::TritInt::repunit(7).host_u32() as u64;
pub const REPUNIT_R8: u64 = crate::trit_int::TritInt::repunit(8).host_u32() as u64;
pub const REPUNIT_R9: u64 = crate::trit_int::TritInt::repunit(9).host_u32() as u64;

/// Full ternary circle in circle-days.
pub const FULL_CIRCLE_DAYS: u64 = REPUNIT_R6; // 364

/// Convert circle-days to calendar days (DOT-aware).
/// Each complete 364-day rotation inserts 1 DOT.
pub const fn circle_days_to_calendar_days(circle_days: u64) -> u64 {
    let completed_circles = circle_days / FULL_CIRCLE_DAYS;
    circle_days + completed_circles
}

/// Repunit factorization: R(2n) = R(n) × (3^n + 1)
pub const fn repunit_factorization(n: u32) -> (u64, u64, u64) {
    let rn = repunit(n);
    let mut pow3: u64 = 1;
    let mut i = 0;
    while i < n {
        pow3 *= 3;
        i += 1;
    }
    let cofactor = pow3 + 1;
    (rn * cofactor, rn, cofactor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repunit_values() {
        assert_eq!(repunit(1), 1);
        assert_eq!(repunit(2), 4);
        assert_eq!(repunit(3), REPUNIT_R3);
        assert_eq!(repunit(4), REPUNIT_R4);
        assert_eq!(repunit(5), REPUNIT_R5);
        assert_eq!(repunit(6), REPUNIT_R6);
        assert_eq!(repunit(7), REPUNIT_R7);
        assert_eq!(repunit(8), REPUNIT_R8);
        assert_eq!(repunit(9), REPUNIT_R9);
    }

    #[test]
    fn test_full_circle_is_r6() {
        assert_eq!(FULL_CIRCLE_DAYS, 364);
        assert_eq!(REPUNIT_R6, 364);
    }

    #[test]
    fn test_circle_to_calendar_single() {
        // 1 circle = 364 circle-days → 365 calendar days (364 + 1 DOT)
        assert_eq!(circle_days_to_calendar_days(364), 365);
    }

    #[test]
    fn test_circle_to_calendar_partial() {
        // Less than 1 full circle → no DOT insertion
        assert_eq!(circle_days_to_calendar_days(100), 100);
        assert_eq!(circle_days_to_calendar_days(363), 363);
    }

    #[test]
    fn test_circle_to_calendar_r7() {
        // R₇ = 1093 circle-days, floor(1093/364) = 3 complete circles → 3 DOTs
        assert_eq!(circle_days_to_calendar_days(REPUNIT_R7), 1096);
    }

    #[test]
    fn test_circle_to_calendar_r8() {
        // R₈ = 3280, floor(3280/364) = 9 complete circles → 9 DOTs
        assert_eq!(circle_days_to_calendar_days(REPUNIT_R8), 3289);
    }

    #[test]
    fn test_circle_to_calendar_r9() {
        // R₉ = 9841, floor(9841/364) = 27 complete circles → 27 DOTs
        assert_eq!(circle_days_to_calendar_days(REPUNIT_R9), 9868);
    }

    #[test]
    fn test_factorization_identity() {
        // R(2n) = R(n) × (3^n + 1)
        for n in 1..=4 {
            let (r2n, rn, cofactor) = repunit_factorization(n);
            assert_eq!(r2n, repunit(2 * n), "R(2×{}) factorization failed", n);
            assert_eq!(r2n, rn * cofactor);
        }
    }

    #[test]
    fn test_powers_of_3_in_calendar_approximations() {
        // R₆ → ~1 year (3⁰), R₇ → ~3 years (3¹), R₈ → ~9 years (3²), R₉ → ~27 years (3³)
        // These are approximate but the powers-of-3 pattern is structurally inherent.
        let r6_years = circle_days_to_calendar_days(REPUNIT_R6) as f64 / 365.0;
        let r7_years = circle_days_to_calendar_days(REPUNIT_R7) as f64 / 365.0;
        let r8_years = circle_days_to_calendar_days(REPUNIT_R8) as f64 / 365.0;
        let r9_years = circle_days_to_calendar_days(REPUNIT_R9) as f64 / 365.0;

        assert!((r6_years - 1.0).abs() < 0.01);
        assert!((r7_years - 3.0).abs() < 0.01);
        assert!((r8_years - 9.0).abs() < 0.02);
        assert!((r9_years - 27.0).abs() < 0.05);
    }
}
