//! PlenumNET Synchronization Primitives
//!
//! Provides `no_std`-compatible synchronization primitives for the Salvi Framework
//! kernel. All primitives are designed for use in bare-metal environments without
//! OS-level threading support.
//!
//! # Primitives
//! - **Spinlock** - Ticket-based spinlock with fairness guarantees
//! - **Mutex** - Spinlock-based mutual exclusion with ternary state awareness
//! - **Semaphore** - Counting semaphore for resource management
//! - **PhaseMutex** - Phase-encryption-aware locking for split data recombination
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

pub mod spinlock;
pub mod mutex;
pub mod semaphore;
pub mod phase_mutex;

pub use spinlock::Spinlock;
pub use mutex::Mutex;
pub use semaphore::Semaphore;
pub use phase_mutex::PhaseMutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockState {
    Unlocked,
    Locked,
    Contested,
}

#[derive(Debug, Clone)]
pub enum SyncError {
    WouldBlock,
    Poisoned,
    Timeout,
    PhaseWindowExpired { elapsed_fs: u128, tolerance_fs: u128 },
    SecurityViolation,
    InvalidState,
}

impl core::fmt::Display for SyncError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SyncError::WouldBlock => write!(f, "Lock would block"),
            SyncError::Poisoned => write!(f, "Lock is poisoned"),
            SyncError::Timeout => write!(f, "Lock acquisition timed out"),
            SyncError::PhaseWindowExpired { elapsed_fs, tolerance_fs } => {
                write!(f, "Phase window expired: {}fs elapsed, {}fs tolerance", elapsed_fs, tolerance_fs)
            }
            SyncError::SecurityViolation => write!(f, "Security mode violation"),
            SyncError::InvalidState => write!(f, "Invalid lock state"),
        }
    }
}

pub type SyncResult<T> = core::result::Result<T, SyncError>;
