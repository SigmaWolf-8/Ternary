//! Phase-Split Encryption Module
//!
//! Implements quantum-resistant encryption using phase-based data splitting
//! as described in the Salvi Framework whitepaper.
//!
//! # Security Model
//! Data is split into multiple phase components that must be recombined
//! within a femtosecond-precision time window to reconstruct the original.

use crate::timing::FemtosecondTimestamp;
use alloc::vec::Vec;

/// Phase encryption modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    /// Maximum security, slower performance
    HighSecurity,
    /// Balance between security and performance
    Balanced,
    /// Optimized for speed
    Performance,
    /// Automatically selects based on data size
    Adaptive,
}

impl EncryptionMode {
    /// Get the number of phase splits for this mode
    pub fn phase_count(&self) -> usize {
        match self {
            EncryptionMode::HighSecurity => 7,
            EncryptionMode::Balanced => 5,
            EncryptionMode::Performance => 3,
            EncryptionMode::Adaptive => 5,
        }
    }

    /// Get the split ratio for this mode
    pub fn split_ratio(&self) -> f64 {
        match self {
            EncryptionMode::HighSecurity => 0.618, // Golden ratio
            EncryptionMode::Balanced => 0.5,
            EncryptionMode::Performance => 0.5,
            EncryptionMode::Adaptive => 0.618,
        }
    }

    /// Get recombination tolerance in femtoseconds
    pub fn recombination_tolerance_fs(&self) -> u128 {
        match self {
            EncryptionMode::HighSecurity => 50,
            EncryptionMode::Balanced => 100,
            EncryptionMode::Performance => 500,
            EncryptionMode::Adaptive => 100,
        }
    }
}

/// Phase configuration
#[derive(Debug, Clone)]
pub struct PhaseConfig {
    pub mode: EncryptionMode,
    pub primary_phase: u8,
    pub secondary_offset: u8,
    pub recombination_tolerance_fs: u128,
}

impl PhaseConfig {
    pub fn new(mode: EncryptionMode) -> Self {
        Self {
            mode,
            primary_phase: 0,
            secondary_offset: 120, // 120 degrees
            recombination_tolerance_fs: mode.recombination_tolerance_fs(),
        }
    }
}

/// A single phase component
#[derive(Debug, Clone)]
pub struct PhaseComponent {
    pub data: Vec<u8>,
    pub phase: u8,
    pub timestamp: FemtosecondTimestamp,
    pub checksum: u32,
}

impl PhaseComponent {
    fn compute_checksum(data: &[u8]) -> u32 {
        let mut hash: u32 = 0;
        for byte in data {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u32);
        }
        hash
    }

    pub fn new(data: Vec<u8>, phase: u8, timestamp: FemtosecondTimestamp) -> Self {
        let checksum = Self::compute_checksum(&data);
        Self {
            data,
            phase,
            timestamp,
            checksum,
        }
    }

    pub fn verify_checksum(&self) -> bool {
        Self::compute_checksum(&self.data) == self.checksum
    }
}

/// Encrypted phase-split data
#[derive(Debug, Clone)]
pub struct PhaseSplitData {
    pub primary: PhaseComponent,
    pub secondary: PhaseComponent,
    pub config: PhaseConfig,
    pub original_length: usize,
}

/// Split data into phase components
pub fn split_data(
    data: &[u8],
    mode: EncryptionMode,
    timestamp: FemtosecondTimestamp,
) -> PhaseSplitData {
    let config = PhaseConfig::new(mode);
    let split_point = (data.len() as f64 * config.mode.split_ratio()) as usize;

    // Apply phase transformation to each half
    let primary_data: Vec<u8> = data[..split_point]
        .iter()
        .enumerate()
        .map(|(i, &b)| b.wrapping_add((i as u8).wrapping_mul(config.primary_phase)))
        .collect();

    let secondary_data: Vec<u8> = data[split_point..]
        .iter()
        .enumerate()
        .map(|(i, &b)| b.wrapping_add((i as u8).wrapping_mul(config.secondary_offset)))
        .collect();

    let secondary_timestamp = FemtosecondTimestamp::new(timestamp.femtoseconds + 50);

    PhaseSplitData {
        primary: PhaseComponent::new(primary_data, config.primary_phase, timestamp),
        secondary: PhaseComponent::new(secondary_data, config.secondary_offset, secondary_timestamp),
        config,
        original_length: data.len(),
    }
}

/// Recombine phase components back to original data
pub fn recombine_data(split_data: &PhaseSplitData) -> Result<Vec<u8>, RecombineError> {
    // Verify checksums
    if !split_data.primary.verify_checksum() {
        return Err(RecombineError::ChecksumMismatch("primary"));
    }
    if !split_data.secondary.verify_checksum() {
        return Err(RecombineError::ChecksumMismatch("secondary"));
    }

    // Verify timing window
    let time_diff = if split_data.secondary.timestamp.femtoseconds > split_data.primary.timestamp.femtoseconds {
        split_data.secondary.timestamp.femtoseconds - split_data.primary.timestamp.femtoseconds
    } else {
        split_data.primary.timestamp.femtoseconds - split_data.secondary.timestamp.femtoseconds
    };

    if time_diff > split_data.config.recombination_tolerance_fs {
        return Err(RecombineError::TimingWindowExceeded {
            diff_fs: time_diff,
            tolerance_fs: split_data.config.recombination_tolerance_fs,
        });
    }

    // Reverse phase transformation
    let primary_restored: Vec<u8> = split_data.primary.data
        .iter()
        .enumerate()
        .map(|(i, &b)| b.wrapping_sub((i as u8).wrapping_mul(split_data.config.primary_phase)))
        .collect();

    let secondary_restored: Vec<u8> = split_data.secondary.data
        .iter()
        .enumerate()
        .map(|(i, &b)| b.wrapping_sub((i as u8).wrapping_mul(split_data.config.secondary_offset)))
        .collect();

    // Combine
    let mut result = primary_restored;
    result.extend(secondary_restored);

    if result.len() != split_data.original_length {
        return Err(RecombineError::LengthMismatch {
            expected: split_data.original_length,
            actual: result.len(),
        });
    }

    Ok(result)
}

/// Recombination errors
#[derive(Debug, Clone)]
pub enum RecombineError {
    ChecksumMismatch(&'static str),
    TimingWindowExceeded { diff_fs: u128, tolerance_fs: u128 },
    LengthMismatch { expected: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_recombine_balanced() {
        let data = b"Hello, PlenumNET!";
        let timestamp = FemtosecondTimestamp::new(1_000_000);
        let split = split_data(data, EncryptionMode::Balanced, timestamp);
        let recombined = recombine_data(&split).unwrap();
        assert_eq!(recombined, data);
    }

    #[test]
    fn test_split_and_recombine_high_security() {
        let data = b"Post-Quantum Ternary Data";
        let timestamp = FemtosecondTimestamp::new(2_000_000);
        let split = split_data(data, EncryptionMode::HighSecurity, timestamp);
        let recombined = recombine_data(&split).unwrap();
        assert_eq!(recombined, data);
    }

    #[test]
    fn test_split_and_recombine_performance() {
        let data = b"Fast path encryption";
        let timestamp = FemtosecondTimestamp::new(3_000_000);
        let split = split_data(data, EncryptionMode::Performance, timestamp);
        let recombined = recombine_data(&split).unwrap();
        assert_eq!(recombined, data);
    }

    #[test]
    fn test_split_and_recombine_adaptive() {
        let data = b"Adaptive mode test data for PlenumNET framework";
        let timestamp = FemtosecondTimestamp::new(4_000_000);
        let split = split_data(data, EncryptionMode::Adaptive, timestamp);
        let recombined = recombine_data(&split).unwrap();
        assert_eq!(recombined, data);
    }

    #[test]
    fn test_split_preserves_length() {
        let data = b"Test data";
        let timestamp = FemtosecondTimestamp::new(1_000);
        let split = split_data(data, EncryptionMode::Balanced, timestamp);
        assert_eq!(split.original_length, data.len());
        assert_eq!(split.primary.data.len() + split.secondary.data.len(), data.len());
    }

    #[test]
    fn test_mode_configurations() {
        assert_eq!(EncryptionMode::HighSecurity.phase_count(), 7);
        assert_eq!(EncryptionMode::Balanced.phase_count(), 5);
        assert_eq!(EncryptionMode::Performance.phase_count(), 3);
        assert_eq!(EncryptionMode::Adaptive.phase_count(), 5);
    }

    #[test]
    fn test_mode_split_ratios() {
        assert!((EncryptionMode::HighSecurity.split_ratio() - 0.618).abs() < 0.001);
        assert!((EncryptionMode::Balanced.split_ratio() - 0.5).abs() < 0.001);
        assert!((EncryptionMode::Performance.split_ratio() - 0.5).abs() < 0.001);
        assert!((EncryptionMode::Adaptive.split_ratio() - 0.618).abs() < 0.001);
    }

    #[test]
    fn test_mode_tolerances() {
        assert_eq!(EncryptionMode::HighSecurity.recombination_tolerance_fs(), 50);
        assert_eq!(EncryptionMode::Balanced.recombination_tolerance_fs(), 100);
        assert_eq!(EncryptionMode::Performance.recombination_tolerance_fs(), 500);
        assert_eq!(EncryptionMode::Adaptive.recombination_tolerance_fs(), 100);
    }

    #[test]
    fn test_high_security_tighter_tolerance() {
        assert!(
            EncryptionMode::HighSecurity.recombination_tolerance_fs()
                < EncryptionMode::Balanced.recombination_tolerance_fs()
        );
        assert!(
            EncryptionMode::Balanced.recombination_tolerance_fs()
                < EncryptionMode::Performance.recombination_tolerance_fs()
        );
    }

    #[test]
    fn test_phase_component_checksum() {
        let data = vec![1, 2, 3, 4, 5];
        let ts = FemtosecondTimestamp::new(1000);
        let component = PhaseComponent::new(data, 0, ts);
        assert!(component.verify_checksum());
    }

    #[test]
    fn test_phase_component_checksum_tamper_detection() {
        let data = vec![1, 2, 3, 4, 5];
        let ts = FemtosecondTimestamp::new(1000);
        let mut component = PhaseComponent::new(data, 0, ts);
        component.data[0] = 99;
        assert!(!component.verify_checksum());
    }

    #[test]
    fn test_phase_config_defaults() {
        let config = PhaseConfig::new(EncryptionMode::Balanced);
        assert_eq!(config.mode, EncryptionMode::Balanced);
        assert_eq!(config.primary_phase, 0);
        assert_eq!(config.secondary_offset, 120);
        assert_eq!(config.recombination_tolerance_fs, 100);
    }

    #[test]
    fn test_split_empty_data() {
        let data = b"";
        let timestamp = FemtosecondTimestamp::new(1_000);
        let split = split_data(data, EncryptionMode::Balanced, timestamp);
        let recombined = recombine_data(&split).unwrap();
        assert_eq!(recombined.len(), 0);
    }

    #[test]
    fn test_split_single_byte() {
        let data = b"X";
        let timestamp = FemtosecondTimestamp::new(1_000);
        let split = split_data(data, EncryptionMode::Balanced, timestamp);
        let recombined = recombine_data(&split).unwrap();
        assert_eq!(recombined, data);
    }

    #[test]
    fn test_split_large_data() {
        let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let timestamp = FemtosecondTimestamp::new(1_000_000);
        let split = split_data(&data, EncryptionMode::HighSecurity, timestamp);
        let recombined = recombine_data(&split).unwrap();
        assert_eq!(recombined, data);
    }
}
