// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

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

use crate::{SALVI_EPOCH_NS, TimingSource};
use alloc::boxed::Box;
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

pub trait HptpProvider: Send {
    fn read_timestamp(&self, cycle_count: u64) -> FemtosecondTimestamp;
    fn timing_source(&self) -> TimingSource;
    fn cycle_period_fs(&self) -> u128;
    fn epoch_fs(&self) -> u128;
}

pub struct SimulatedHptp {
    epoch_fs: u128,
    cycle_period_fs: u128,
}

impl SimulatedHptp {
    pub fn new() -> Self {
        Self {
            epoch_fs: 0,
            cycle_period_fs: 1000,
        }
    }

    pub fn with_epoch(mut self, epoch_fs: u128) -> Self {
        self.epoch_fs = epoch_fs;
        self
    }

    pub fn with_cycle_period(mut self, period_fs: u128) -> Self {
        self.cycle_period_fs = period_fs;
        self
    }
}

impl HptpProvider for SimulatedHptp {
    fn read_timestamp(&self, cycle_count: u64) -> FemtosecondTimestamp {
        FemtosecondTimestamp::new(self.epoch_fs + (cycle_count as u128) * self.cycle_period_fs)
    }

    fn timing_source(&self) -> TimingSource {
        TimingSource::SystemClock
    }

    fn cycle_period_fs(&self) -> u128 {
        self.cycle_period_fs
    }

    fn epoch_fs(&self) -> u128 {
        self.epoch_fs
    }
}

pub struct LiveHptp {
    callback: Box<dyn Fn(u64) -> FemtosecondTimestamp + Send>,
    source: TimingSource,
    period_fs: u128,
    epoch_fs: u128,
}

impl LiveHptp {
    pub fn new(
        callback: Box<dyn Fn(u64) -> FemtosecondTimestamp + Send>,
        source: TimingSource,
    ) -> Self {
        Self {
            callback,
            source,
            period_fs: 1000,
            epoch_fs: 0,
        }
    }

    pub fn with_period(mut self, period_fs: u128) -> Self {
        self.period_fs = period_fs;
        self
    }

    pub fn with_epoch(mut self, epoch_fs: u128) -> Self {
        self.epoch_fs = epoch_fs;
        self
    }
}

impl HptpProvider for LiveHptp {
    fn read_timestamp(&self, cycle_count: u64) -> FemtosecondTimestamp {
        (self.callback)(cycle_count)
    }

    fn timing_source(&self) -> TimingSource {
        self.source
    }

    fn cycle_period_fs(&self) -> u128 {
        self.period_fs
    }

    fn epoch_fs(&self) -> u128 {
        self.epoch_fs
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
    fn test_timestamp_zero() {
        let ts = FemtosecondTimestamp::new(0);
        assert_eq!(ts.seconds(), 0);
        assert_eq!(ts.milliseconds(), 0);
        assert_eq!(ts.nanoseconds(), 0);
        assert_eq!(ts.picoseconds(), 0);
        assert_eq!(ts.remaining_femtoseconds(), 0);
    }

    #[test]
    fn test_timestamp_precision_hierarchy() {
        let ts = FemtosecondTimestamp::new(
            2 * FS_PER_SECOND + 345 * FS_PER_MS + 678 * FS_PER_NS / 1_000_000 + 901 * FS_PER_PS + 234
        );
        assert_eq!(ts.seconds(), 2);
    }

    #[test]
    fn test_timestamp_sub_second_extraction() {
        let ts = FemtosecondTimestamp::new(FS_PER_SECOND + 500 * FS_PER_MS);
        assert_eq!(ts.seconds(), 1);
        assert_eq!(ts.milliseconds(), 500);
        assert_eq!(ts.sub_second_fs(), 500 * FS_PER_MS);
    }

    #[test]
    fn test_finra_compliance_at_boundary() {
        let t1 = FemtosecondTimestamp::new(0);
        let t_exactly_50ms = FemtosecondTimestamp::new(50 * FS_PER_MS);
        assert!(t1.is_finra_compliant(&t_exactly_50ms));

        let t_just_over = FemtosecondTimestamp::new(50 * FS_PER_MS + 1);
        assert!(!t1.is_finra_compliant(&t_just_over));
    }

    #[test]
    fn test_finra_compliance_symmetric() {
        let t1 = FemtosecondTimestamp::new(100 * FS_PER_MS);
        let t2 = FemtosecondTimestamp::new(130 * FS_PER_MS);
        assert_eq!(t1.is_finra_compliant(&t2), t2.is_finra_compliant(&t1));
    }

    #[test]
    fn test_finra_compliance_self() {
        let t = FemtosecondTimestamp::new(1_000_000);
        assert!(t.is_finra_compliant(&t));
    }

    #[test]
    fn test_duration_since() {
        let t1 = FemtosecondTimestamp::new(1000);
        let t2 = FemtosecondTimestamp::new(3000);
        let d = t2.duration_since(&t1);
        assert_eq!(d.femtoseconds, 2000);
    }

    #[test]
    fn test_duration_since_saturating() {
        let t1 = FemtosecondTimestamp::new(3000);
        let t2 = FemtosecondTimestamp::new(1000);
        let d = t2.duration_since(&t1);
        assert_eq!(d.femtoseconds, 0);
    }

    #[test]
    fn test_duration_conversions() {
        let d = Duration::from_ms(1);
        assert_eq!(d.as_ms(), 1);
        assert_eq!(d.femtoseconds, FS_PER_MS);

        let d = Duration::from_fs(1_000_000);
        assert_eq!(d.as_fs(), 1_000_000);
    }

    #[test]
    fn test_timestamp_format() {
        let ts = FemtosecondTimestamp::new(FS_PER_SECOND + 123 * FS_PER_MS);
        let formatted = ts.format();
        assert!(formatted.contains("1s"));
        assert!(formatted.contains("123ms"));
    }

    #[test]
    fn test_recombination_window_within() {
        let t1 = FemtosecondTimestamp::new(1000);
        let t2 = FemtosecondTimestamp::new(1050);
        assert!(validate_recombination_window(&t1, &t2, 100));
    }

    #[test]
    fn test_recombination_window_exceeded() {
        let t1 = FemtosecondTimestamp::new(1000);
        let t3 = FemtosecondTimestamp::new(1200);
        assert!(!validate_recombination_window(&t1, &t3, 100));
    }

    #[test]
    fn test_recombination_window_symmetric() {
        let t1 = FemtosecondTimestamp::new(1000);
        let t2 = FemtosecondTimestamp::new(1050);
        assert_eq!(
            validate_recombination_window(&t1, &t2, 100),
            validate_recombination_window(&t2, &t1, 100)
        );
    }

    #[test]
    fn test_recombination_window_exact_boundary() {
        let t1 = FemtosecondTimestamp::new(0);
        let t2 = FemtosecondTimestamp::new(100);
        assert!(!validate_recombination_window(&t1, &t2, 100));
        let t3 = FemtosecondTimestamp::new(99);
        assert!(validate_recombination_window(&t1, &t3, 100));
    }

    #[test]
    fn test_precision_constants() {
        assert_eq!(FS_PER_PS, 1_000);
        assert_eq!(FS_PER_NS, 1_000_000);
        assert_eq!(FS_PER_US, 1_000_000_000);
        assert_eq!(FS_PER_MS, 1_000_000_000_000);
        assert_eq!(FS_PER_SECOND, 1_000_000_000_000_000);
    }

    #[test]
    fn test_finra_max_offset() {
        assert_eq!(FINRA_MAX_OFFSET_MS, 50);
        assert_eq!(FINRA_MAX_OFFSET_FS, 50 * FS_PER_MS);
    }
}
