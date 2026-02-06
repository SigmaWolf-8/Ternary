//! Error types for the PlenumNET kernel

use alloc::string::String;
use core::fmt;

/// Kernel error types
#[derive(Debug, Clone)]
pub enum KernelError {
    /// Invalid trit value
    InvalidTrit { value: i8 },
    /// Invalid tryte value
    InvalidTryte { value: u16 },
    /// Timing error
    TimingError(TimingError),
    /// Phase encryption error
    PhaseError(PhaseError),
    /// Memory subsystem error
    MemoryError(crate::memory::MemoryError),
    /// Synchronization error
    SyncError(crate::sync::SyncError),
    /// Process management error
    ProcessError(crate::process::ProcessError),
    /// Security subsystem error
    SecurityError(crate::security::SecurityError),
    /// Cryptographic error
    CryptoError(crate::crypto::CryptoError),
    /// Device error
    DeviceError(crate::device::DeviceError),
    /// I/O error
    IoError(crate::io::IoError),
    /// Filesystem error
    FsError(crate::fs::FsError),
    /// Representation conversion error
    ConversionError { from: String, to: String },
}

/// Timing-specific errors
#[derive(Debug, Clone)]
pub enum TimingError {
    /// Clock source unavailable
    ClockUnavailable,
    /// Synchronization lost
    SyncLost,
    /// FINRA compliance violation
    FinraViolation { offset_ms: u64 },
    /// Timestamp overflow
    Overflow,
}

/// Phase encryption errors
#[derive(Debug, Clone)]
pub enum PhaseError {
    /// Invalid encryption mode
    InvalidMode,
    /// Recombination window exceeded
    WindowExceeded { diff_fs: u128, tolerance_fs: u128 },
    /// Data integrity check failed
    IntegrityFailed,
    /// Phase mismatch
    PhaseMismatch { expected: u8, actual: u8 },
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::InvalidTrit { value } => {
                write!(f, "Invalid trit value: {}. Expected -1, 0, or 1", value)
            }
            KernelError::InvalidTryte { value } => {
                write!(f, "Invalid tryte value: {}. Expected 0-728", value)
            }
            KernelError::TimingError(e) => write!(f, "Timing error: {:?}", e),
            KernelError::PhaseError(e) => write!(f, "Phase error: {:?}", e),
            KernelError::MemoryError(e) => write!(f, "Memory error: {}", e),
            KernelError::SyncError(e) => write!(f, "Sync error: {}", e),
            KernelError::ProcessError(e) => write!(f, "Process error: {}", e),
            KernelError::SecurityError(e) => write!(f, "Security error: {}", e),
            KernelError::CryptoError(e) => write!(f, "Crypto error: {}", e),
            KernelError::DeviceError(e) => write!(f, "Device error: {}", e),
            KernelError::IoError(e) => write!(f, "I/O error: {}", e),
            KernelError::FsError(e) => write!(f, "Filesystem error: {}", e),
            KernelError::ConversionError { from, to } => {
                write!(f, "Cannot convert from {} to {}", from, to)
            }
        }
    }
}

impl From<TimingError> for KernelError {
    fn from(e: TimingError) -> Self {
        KernelError::TimingError(e)
    }
}

impl From<PhaseError> for KernelError {
    fn from(e: PhaseError) -> Self {
        KernelError::PhaseError(e)
    }
}

impl From<crate::memory::MemoryError> for KernelError {
    fn from(e: crate::memory::MemoryError) -> Self {
        KernelError::MemoryError(e)
    }
}

impl From<crate::sync::SyncError> for KernelError {
    fn from(e: crate::sync::SyncError) -> Self {
        KernelError::SyncError(e)
    }
}

impl From<crate::process::ProcessError> for KernelError {
    fn from(e: crate::process::ProcessError) -> Self {
        KernelError::ProcessError(e)
    }
}

impl From<crate::security::SecurityError> for KernelError {
    fn from(e: crate::security::SecurityError) -> Self {
        KernelError::SecurityError(e)
    }
}

impl From<crate::crypto::CryptoError> for KernelError {
    fn from(e: crate::crypto::CryptoError) -> Self {
        KernelError::CryptoError(e)
    }
}

impl From<crate::device::DeviceError> for KernelError {
    fn from(e: crate::device::DeviceError) -> Self {
        KernelError::DeviceError(e)
    }
}

impl From<crate::io::IoError> for KernelError {
    fn from(e: crate::io::IoError) -> Self {
        KernelError::IoError(e)
    }
}

impl From<crate::fs::FsError> for KernelError {
    fn from(e: crate::fs::FsError) -> Self {
        KernelError::FsError(e)
    }
}

/// Result type for kernel operations
pub type KernelResult<T> = core::result::Result<T, KernelError>;
