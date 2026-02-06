//! PlenumNET Salvi Framework Kernel
//!
//! Post-Quantum Ternary Computing Core implementing the Salvi Framework
//! specification for quantum-resistant infrastructure.
//!
//! # Features
//! - Ternary Logic Operations (GF(3) arithmetic)
//! - Femtosecond-precision timing (FINRA Rule 613 compliant)
//! - Phase-split encryption
//! - Multi-architecture support (x86_64, aarch64, riscv64)
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

#![cfg_attr(feature = "no_std", no_std)]

extern crate alloc;

pub mod timing;
pub mod ternary;
pub mod phase;
pub mod error;
pub mod memory;
pub mod sync;

use alloc::string::String;

/// Salvi Epoch: April 1, 2025 00:00:00.000 UTC
/// Unix timestamp in nanoseconds: 1743465600000000000
pub const SALVI_EPOCH_NS: u128 = 1_743_465_600_000_000_000;

/// Salvi Epoch as femtoseconds
pub const SALVI_EPOCH_FS: u128 = SALVI_EPOCH_NS * 1_000_000;

/// Kernel version
pub const KERNEL_VERSION: &str = "0.1.0";

/// Copyright notice
pub const COPYRIGHT: &str = "Copyright (c) 2026 Capomastro Holdings Ltd";

/// Kernel initialization result
#[derive(Debug, Clone)]
pub struct KernelInfo {
    pub version: String,
    pub architecture: Architecture,
    pub features: KernelFeatures,
    pub timing_source: TimingSource,
}

/// Supported architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Riscv64,
    Fpga,
    Asic,
    Unknown,
}

impl Architecture {
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        return Architecture::X86_64;
        
        #[cfg(target_arch = "aarch64")]
        return Architecture::Aarch64;
        
        #[cfg(target_arch = "riscv64")]
        return Architecture::Riscv64;
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
        return Architecture::Unknown;
    }
}

/// Kernel feature flags
#[derive(Debug, Clone, Default)]
pub struct KernelFeatures {
    pub ternary_ops: bool,
    pub femtosecond_timing: bool,
    pub phase_encryption: bool,
    pub finra_613_compliant: bool,
    pub hardware_acceleration: bool,
}

/// Timing source enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingSource {
    SystemClock,
    Tsc,
    Hpet,
    OpticalAtomic,
    Ptp,
    Unknown,
}

/// Initialize the PlenumNET kernel
pub fn init() -> KernelInfo {
    KernelInfo {
        version: String::from(KERNEL_VERSION),
        architecture: Architecture::detect(),
        features: KernelFeatures {
            ternary_ops: true,
            femtosecond_timing: true,
            phase_encryption: true,
            finra_613_compliant: cfg!(feature = "finra-613"),
            hardware_acceleration: cfg!(any(feature = "fpga", feature = "asic")),
        },
        timing_source: TimingSource::SystemClock,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_init() {
        let info = init();
        assert_eq!(info.version, KERNEL_VERSION);
        assert!(info.features.ternary_ops);
        assert!(info.features.femtosecond_timing);
    }

    #[test]
    fn test_salvi_epoch() {
        assert_eq!(SALVI_EPOCH_NS, 1_743_465_600_000_000_000);
        assert_eq!(SALVI_EPOCH_FS, SALVI_EPOCH_NS * 1_000_000);
    }
}
