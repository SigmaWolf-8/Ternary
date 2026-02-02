//! Femtosecond-Precision Timing Module
//!
//! Implements FINRA Rule 613 CAT compliant timing with femtosecond precision.
//! Uses the Salvi Epoch (April 1, 2025) as Day Zero.
//!
//! # Precision Hierarchy
//! - Femtoseconds (10⁻¹⁵ seconds) - Primary precision
//! - Picoseconds (10⁻¹² seconds)
//! - Nanoseconds (10⁻⁹ seconds)
//! - Microseconds (10⁻⁶ seconds)
//! - Milliseconds (10⁻³ seconds) - FINRA minimum requirement

use crate::{SALVI_EPOCH_FS, SALVI_EPOCH_NS, TimingSource};
use alloc::string::String;

/// Femtoseconds per time unit
pub const FS_PER_PS: u128 = 1_000;
pub const FS_PER_NS: u128 = 1_000_000;
pub const FS_PER_US: u128 = 1_000_000_000;
pub const FS_PER_MS: u128 = 1_000_000_000_000;
pub const FS_PER_SECOND: u128 = 1_000_000_000_000_000;

/// FINRA Rule 613 requires 50ms maximum clock offset
pub const FINRA_MAX_OFFSET_MS: u64 = 50;
pub const FINRA_MAX_OFFSET_FS: u128 = 50 * FS_PER_MS;

/// High-precision timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FemtosecondTimestamp {
    /// Femtoseconds since Salvi Epoch
    pub femtoseconds: u128,
}

impl FemtosecondTimestamp {
    /// Create a new timestamp from femtoseconds since Salvi Epoch
    pub fn new(femtoseconds: u128) -> Self {
        Self { femtoseconds }
    }

    /// Create from Unix nanoseconds
    pub fn from_unix_ns(unix_ns: u128) -> Self {
        let salvi_offset_ns = unix_ns.saturating_sub(SALVI_EPOCH_NS);
        Self {
            femtoseconds: salvi_offset_ns * FS_PER_NS / 1_000_000,
        }
    }

    /// Convert to Unix nanoseconds
    pub fn to_unix_ns(&self) -> u128 {
        SALVI_EPOCH_NS + (self.femtoseconds / FS_PER_NS * 1_000_000)
    }

    /// Get seconds component
    pub fn seconds(&self) -> u64 {
        (self.femtoseconds / FS_PER_SECOND) as u64
    }

    /// Get sub-second femtoseconds
    pub fn sub_second_fs(&self) -> u128 {
        self.femtoseconds % FS_PER_SECOND
    }

    /// Get milliseconds component
    pub fn milliseconds(&self) -> u64 {
        ((self.femtoseconds % FS_PER_SECOND) / FS_PER_MS) as u64
    }

    /// Get nanoseconds component
    pub fn nanoseconds(&self) -> u64 {
        ((self.femtoseconds % FS_PER_MS) / FS_PER_NS) as u64
    }

    /// Get picoseconds component
    pub fn picoseconds(&self) -> u64 {
        ((self.femtoseconds % FS_PER_NS) / FS_PER_PS) as u64
    }

    /// Get remaining femtoseconds
    pub fn remaining_femtoseconds(&self) -> u64 {
        (self.femtoseconds % FS_PER_PS) as u64
    }

    /// Calculate duration between two timestamps
    pub fn duration_since(&self, earlier: &FemtosecondTimestamp) -> Duration {
        Duration {
            femtoseconds: self.femtoseconds.saturating_sub(earlier.femtoseconds),
        }
    }

    /// Check if within FINRA Rule 613 tolerance
    pub fn is_finra_compliant(&self, reference: &FemtosecondTimestamp) -> bool {
        let diff = if self.femtoseconds > reference.femtoseconds {
            self.femtoseconds - reference.femtoseconds
        } else {
            reference.femtoseconds - self.femtoseconds
        };
        diff <= FINRA_MAX_OFFSET_FS
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        alloc::format!(
            "{}s {:03}ms {:03}ns {:03}ps {:03}fs",
            self.seconds(),
            self.milliseconds(),
            self.nanoseconds(),
            self.picoseconds(),
            self.remaining_femtoseconds()
        )
    }
}

/// Duration with femtosecond precision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    pub femtoseconds: u128,
}

impl Duration {
    pub fn from_fs(fs: u128) -> Self {
        Self { femtoseconds: fs }
    }

    pub fn from_ns(ns: u128) -> Self {
        Self { femtoseconds: ns * FS_PER_NS / 1_000_000 }
    }

    pub fn from_ms(ms: u128) -> Self {
        Self { femtoseconds: ms * FS_PER_MS }
    }

    pub fn as_fs(&self) -> u128 {
        self.femtoseconds
    }

    pub fn as_ns(&self) -> u128 {
        self.femtoseconds * 1_000_000 / FS_PER_NS
    }

    pub fn as_ms(&self) -> u128 {
        self.femtoseconds / FS_PER_MS
    }
}

/// Timing metrics for monitoring
#[derive(Debug, Clone)]
pub struct TimingMetrics {
    pub current_timestamp: FemtosecondTimestamp,
    pub clock_source: TimingSource,
    pub synchronization_status: SyncStatus,
    pub estimated_accuracy_fs: u128,
    pub drift_rate_ppb: i64,
}

/// Clock synchronization status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Synchronized,
    Unsynchronized,
    Degraded,
    Holdover,
}

/// Validate recombination window for phase encryption
/// Per whitepaper: |τₚ - τₛ| < 100 femtoseconds
pub fn validate_recombination_window(
    primary: &FemtosecondTimestamp,
    secondary: &FemtosecondTimestamp,
    tolerance_fs: u128,
) -> bool {
    let diff = if primary.femtoseconds > secondary.femtoseconds {
        primary.femtoseconds - secondary.femtoseconds
    } else {
        secondary.femtoseconds - primary.femtoseconds
    };
    diff < tolerance_fs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_creation() {
        let ts = FemtosecondTimestamp::new(1_000_000_000_000_000);
        assert_eq!(ts.seconds(), 1);
        assert_eq!(ts.milliseconds(), 0);
    }

    #[test]
    fn test_finra_compliance() {
        let t1 = FemtosecondTimestamp::new(0);
        let t2 = FemtosecondTimestamp::new(40 * FS_PER_MS);
        assert!(t1.is_finra_compliant(&t2));

        let t3 = FemtosecondTimestamp::new(60 * FS_PER_MS);
        assert!(!t1.is_finra_compliant(&t3));
    }

    #[test]
    fn test_recombination_window() {
        let t1 = FemtosecondTimestamp::new(1000);
        let t2 = FemtosecondTimestamp::new(1050);
        assert!(validate_recombination_window(&t1, &t2, 100));

        let t3 = FemtosecondTimestamp::new(1200);
        assert!(!validate_recombination_window(&t1, &t3, 100));
    }
}
