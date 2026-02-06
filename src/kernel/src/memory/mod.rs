//! PlenumNET Memory Subsystem
//!
//! Provides ternary-aware memory management for the Salvi Framework kernel.
//! Implements a layered architecture:
//!
//! 1. **Frame Allocator** - Physical frame management using a bitmap allocator
//! 2. **Page Tables** - Virtual-to-physical address mapping with ternary region tagging
//! 3. **Heap Allocator** - Dynamic memory allocation using a free-list approach
//!
//! All memory regions can be tagged with ternary security modes
//! (mode_phi, mode_one, mode_zero) per the Salvi Framework specification.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

pub mod allocator;
pub mod page;
pub mod heap;

pub const TERNARY_PAGE_SIZE: usize = 2187; // 3^7 = 2187 bytes (ternary-aligned)
pub const BINARY_PAGE_SIZE: usize = 4096;  // Standard 4KiB for binary compatibility
pub const DEFAULT_PAGE_SIZE: usize = BINARY_PAGE_SIZE;

pub const TRIT_ADDRESSABLE_BITS: usize = 27; // 3 trytes * 9 trits = address space
pub const MAX_TERNARY_ADDRESS: usize = 7_625_597_484_987; // 3^27

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    ModePhi,
    ModeOne,
    ModeZero,
}

impl SecurityMode {
    pub fn access_level(&self) -> u8 {
        match self {
            SecurityMode::ModePhi => 3, // Maximum privilege
            SecurityMode::ModeOne => 2, // Standard operation
            SecurityMode::ModeZero => 1, // Restricted/quarantine
        }
    }

    pub fn can_access(&self, target: &SecurityMode) -> bool {
        self.access_level() >= target.access_level()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    KernelCode,
    KernelData,
    KernelStack,
    UserCode,
    UserData,
    UserStack,
    TernaryCompute,
    PhaseEncrypted,
    TimingCritical,
    Mmio,
    Reserved,
    Free,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
    pub region_type: MemoryRegionType,
    pub security_mode: SecurityMode,
    pub permissions: Permissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub ternary_compute: bool,
}

impl Permissions {
    pub const fn read_only() -> Self {
        Self { read: true, write: false, execute: false, ternary_compute: false }
    }

    pub const fn read_write() -> Self {
        Self { read: true, write: true, execute: false, ternary_compute: false }
    }

    pub const fn read_execute() -> Self {
        Self { read: true, write: false, execute: true, ternary_compute: false }
    }

    pub const fn ternary() -> Self {
        Self { read: true, write: true, execute: false, ternary_compute: true }
    }

    pub const fn all() -> Self {
        Self { read: true, write: true, execute: true, ternary_compute: true }
    }

    pub const fn none() -> Self {
        Self { read: false, write: false, execute: false, ternary_compute: false }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_frames: usize,
    pub free_frames: usize,
    pub used_frames: usize,
    pub page_size: usize,
    pub total_bytes: usize,
    pub free_bytes: usize,
    pub used_bytes: usize,
    pub heap_allocated: usize,
    pub heap_free: usize,
}

#[derive(Debug, Clone)]
pub enum MemoryError {
    OutOfMemory,
    InvalidAddress(usize),
    InvalidAlignment(usize),
    RegionOverlap { base: usize, size: usize },
    PermissionDenied { required: Permissions, actual: Permissions },
    SecurityViolation { required: SecurityMode, actual: SecurityMode },
    DoubleFree(usize),
    PageFault { address: usize },
    FrameExhausted,
    HeapCorruption,
}

impl core::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryError::OutOfMemory => write!(f, "Out of memory"),
            MemoryError::InvalidAddress(addr) => write!(f, "Invalid address: {:#x}", addr),
            MemoryError::InvalidAlignment(align) => write!(f, "Invalid alignment: {}", align),
            MemoryError::RegionOverlap { base, size } => {
                write!(f, "Region overlap at {:#x} size {}", base, size)
            }
            MemoryError::PermissionDenied { .. } => write!(f, "Permission denied"),
            MemoryError::SecurityViolation { .. } => write!(f, "Security mode violation"),
            MemoryError::DoubleFree(addr) => write!(f, "Double free at {:#x}", addr),
            MemoryError::PageFault { address } => write!(f, "Page fault at {:#x}", address),
            MemoryError::FrameExhausted => write!(f, "Physical frames exhausted"),
            MemoryError::HeapCorruption => write!(f, "Heap corruption detected"),
        }
    }
}

pub type MemoryResult<T> = core::result::Result<T, MemoryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_mode_ordering() {
        assert!(SecurityMode::ModePhi.access_level() > SecurityMode::ModeOne.access_level());
        assert!(SecurityMode::ModeOne.access_level() > SecurityMode::ModeZero.access_level());
    }

    #[test]
    fn test_security_mode_access() {
        assert!(SecurityMode::ModePhi.can_access(&SecurityMode::ModeZero));
        assert!(SecurityMode::ModePhi.can_access(&SecurityMode::ModeOne));
        assert!(SecurityMode::ModePhi.can_access(&SecurityMode::ModePhi));
        assert!(!SecurityMode::ModeZero.can_access(&SecurityMode::ModePhi));
    }

    #[test]
    fn test_permissions_presets() {
        let ro = Permissions::read_only();
        assert!(ro.read);
        assert!(!ro.write);
        assert!(!ro.execute);

        let rw = Permissions::read_write();
        assert!(rw.read);
        assert!(rw.write);
        assert!(!rw.execute);

        let rx = Permissions::read_execute();
        assert!(rx.read);
        assert!(!rx.write);
        assert!(rx.execute);

        let ternary = Permissions::ternary();
        assert!(ternary.ternary_compute);
    }

    #[test]
    fn test_page_size_constants() {
        assert_eq!(TERNARY_PAGE_SIZE, 2187);
        assert_eq!(BINARY_PAGE_SIZE, 4096);
        assert_eq!(3usize.pow(7), TERNARY_PAGE_SIZE);
    }
}
