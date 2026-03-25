// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Key Freshness Zones
//!
//! Freshness at ternary 1/3 and 2/3 thresholds within the cryptographic
//! arc (ARC_EPOCH = 182 days):
//!
//! - **Fresh** (age 0–60): key is in first third. Suitable for all operations.
//! - **Active** (age 61–121): key is in second third. Required for regulated ops.
//!   Boundary at 121 = REPUNIT_R5 = 11² resonates with BASE_PORT digit string.
//! - **Aging** (age 122–182): key is in final third. Restricted to read-only.
//!   Rotation imminent.
//!
//! Same `project_to_gf3` quantization used in the slot projection.

use ternary_math::gf3_algebra::project_to_gf3;
use ternary_math::repunit_circles::REPUNIT_R5;

use super::constants::ARC_EPOCH;

/// Key freshness zone, determined by age within the ARC_EPOCH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FreshnessZone {
    Fresh,  // age 0–60: first third, all operations permitted
    Active, // age 61–121: second third, regulated operations
    Aging,  // age 122–182: final third, read-only, rotation imminent
}

impl FreshnessZone {
    pub fn label(&self) -> &'static str {
        match self {
            FreshnessZone::Fresh => "fresh",
            FreshnessZone::Active => "active",
            FreshnessZone::Aging => "aging",
        }
    }
}

/// Compute the freshness zone for a key of the given age in days.
///
/// Uses the same `project_to_gf3` quantization as slot projection:
/// GF(3) 0 = Fresh, 1 = Active, 2 = Aging.
///
/// Returns `None` if age exceeds ARC_EPOCH (key has expired).
pub fn key_freshness(age_days: u64) -> Option<FreshnessZone> {
    if age_days > ARC_EPOCH as u64 {
        return None; // expired
    }

    let zone = project_to_gf3(age_days, ARC_EPOCH as u64);
    Some(match zone {
        0 => FreshnessZone::Fresh,
        1 => FreshnessZone::Active,
        2 => FreshnessZone::Aging,
        _ => unreachable!(),
    })
}

/// Check whether a key is suitable for regulated/sensitive operations.
/// Only Fresh and Active keys qualify.
pub fn key_suitable_for_regulated(age_days: u64) -> bool {
    matches!(key_freshness(age_days), Some(FreshnessZone::Fresh | FreshnessZone::Active))
}

/// Check whether a key is read-only (Aging zone).
pub fn key_is_read_only(age_days: u64) -> bool {
    matches!(key_freshness(age_days), Some(FreshnessZone::Aging))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_zone_boundaries() {
        assert_eq!(key_freshness(0), Some(FreshnessZone::Fresh));
        assert_eq!(key_freshness(1), Some(FreshnessZone::Fresh));
        assert_eq!(key_freshness(60), Some(FreshnessZone::Fresh));
    }

    #[test]
    fn active_zone_boundaries() {
        assert_eq!(key_freshness(61), Some(FreshnessZone::Active));
        assert_eq!(key_freshness(90), Some(FreshnessZone::Active));
        assert_eq!(key_freshness(121), Some(FreshnessZone::Active));
    }

    #[test]
    fn aging_zone_boundaries() {
        assert_eq!(key_freshness(122), Some(FreshnessZone::Aging));
        assert_eq!(key_freshness(150), Some(FreshnessZone::Aging));
        assert_eq!(key_freshness(182), Some(FreshnessZone::Aging));
    }

    #[test]
    fn expired_key() {
        assert!(key_freshness(183).is_none());
        assert!(key_freshness(365).is_none());
    }

    #[test]
    fn active_boundary_is_repunit_r5() {
        // The Active→Aging boundary at 121 = REPUNIT_R5 = 11²
        assert_eq!(key_freshness(REPUNIT_R5), Some(FreshnessZone::Active));
        assert_eq!(key_freshness(REPUNIT_R5 + 1), Some(FreshnessZone::Aging));
    }

    #[test]
    fn regulated_suitability() {
        assert!(key_suitable_for_regulated(0));
        assert!(key_suitable_for_regulated(60));
        assert!(key_suitable_for_regulated(121));
        assert!(!key_suitable_for_regulated(122));
        assert!(!key_suitable_for_regulated(182));
    }

    #[test]
    fn read_only_flag() {
        assert!(!key_is_read_only(60));
        assert!(!key_is_read_only(121));
        assert!(key_is_read_only(122));
        assert!(key_is_read_only(182));
    }
}
