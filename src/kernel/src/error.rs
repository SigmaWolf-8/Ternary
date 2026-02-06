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

/// Result type for kernel operations
pub type KernelResult<T> = core::result::Result<T, KernelError>;
