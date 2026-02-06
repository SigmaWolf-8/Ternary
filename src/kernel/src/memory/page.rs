//! Page Table Management
//!
//! Implements virtual-to-physical address translation for the PlenumNET kernel.
//! Supports ternary region tagging where each virtual page can be associated
//! with a security mode and ternary compute permissions.
//!
//! The page table uses a multi-level structure compatible with both x86_64
//! (4-level) and custom ternary addressing schemes.

use super::{MemoryError, MemoryResult, Permissions, SecurityMode, DEFAULT_PAGE_SIZE};
use alloc::collections::BTreeMap;

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_WRITE_THROUGH: u64 = 1 << 3;
pub const PAGE_CACHE_DISABLE: u64 = 1 << 4;
pub const PAGE_ACCESSED: u64 = 1 << 5;
pub const PAGE_DIRTY: u64 = 1 << 6;
pub const PAGE_HUGE: u64 = 1 << 7;
pub const PAGE_GLOBAL: u64 = 1 << 8;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;

pub const PAGE_TERNARY_COMPUTE: u64 = 1 << 9;
pub const PAGE_PHASE_ENCRYPTED: u64 = 1 << 10;
pub const PAGE_TIMING_CRITICAL: u64 = 1 << 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFlags(u64);

impl PageFlags {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn new(flags: u64) -> Self {
        Self(flags)
    }

    pub fn present(&self) -> bool {
        self.0 & PAGE_PRESENT != 0
    }

    pub fn writable(&self) -> bool {
        self.0 & PAGE_WRITABLE != 0
    }

    pub fn user_accessible(&self) -> bool {
        self.0 & PAGE_USER != 0
    }

    pub fn no_execute(&self) -> bool {
        self.0 & PAGE_NO_EXECUTE != 0
    }

    pub fn ternary_compute(&self) -> bool {
        self.0 & PAGE_TERNARY_COMPUTE != 0
    }

    pub fn phase_encrypted(&self) -> bool {
        self.0 & PAGE_PHASE_ENCRYPTED != 0
    }

    pub fn timing_critical(&self) -> bool {
        self.0 & PAGE_TIMING_CRITICAL != 0
    }

    pub fn from_permissions(perms: &Permissions, user: bool) -> Self {
        let mut flags = PAGE_PRESENT;
        if perms.write {
            flags |= PAGE_WRITABLE;
        }
        if user {
            flags |= PAGE_USER;
        }
        if !perms.execute {
            flags |= PAGE_NO_EXECUTE;
        }
        if perms.ternary_compute {
            flags |= PAGE_TERNARY_COMPUTE;
        }
        Self(flags)
    }

    pub fn to_permissions(&self) -> Permissions {
        Permissions {
            read: self.present(),
            write: self.writable(),
            execute: !self.no_execute(),
            ternary_compute: self.ternary_compute(),
        }
    }

    pub fn bits(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct PageTableEntry {
    pub virtual_address: usize,
    pub physical_address: usize,
    pub flags: PageFlags,
    pub security_mode: SecurityMode,
}

pub struct PageTable {
    entries: BTreeMap<usize, PageTableEntry>,
    page_size: usize,
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            page_size: DEFAULT_PAGE_SIZE,
        }
    }

    pub fn with_page_size(page_size: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            page_size,
        }
    }

    pub fn map(
        &mut self,
        virtual_address: usize,
        physical_address: usize,
        flags: PageFlags,
        security_mode: SecurityMode,
    ) -> MemoryResult<()> {
        let aligned_virt = self.align_down(virtual_address);
        let aligned_phys = self.align_down(physical_address);

        if self.entries.contains_key(&aligned_virt) {
            return Err(MemoryError::RegionOverlap {
                base: aligned_virt,
                size: self.page_size,
            });
        }

        self.entries.insert(aligned_virt, PageTableEntry {
            virtual_address: aligned_virt,
            physical_address: aligned_phys,
            flags,
            security_mode,
        });

        Ok(())
    }

    pub fn unmap(&mut self, virtual_address: usize) -> MemoryResult<PageTableEntry> {
        let aligned = self.align_down(virtual_address);
        self.entries.remove(&aligned)
            .ok_or(MemoryError::PageFault { address: virtual_address })
    }

    pub fn translate(&self, virtual_address: usize) -> MemoryResult<usize> {
        let page_base = self.align_down(virtual_address);
        let offset = virtual_address - page_base;

        let entry = self.entries.get(&page_base)
            .ok_or(MemoryError::PageFault { address: virtual_address })?;

        if !entry.flags.present() {
            return Err(MemoryError::PageFault { address: virtual_address });
        }

        Ok(entry.physical_address + offset)
    }

    pub fn lookup(&self, virtual_address: usize) -> Option<&PageTableEntry> {
        let page_base = self.align_down(virtual_address);
        self.entries.get(&page_base)
    }

    pub fn check_access(
        &self,
        virtual_address: usize,
        required: &Permissions,
        caller_mode: &SecurityMode,
    ) -> MemoryResult<()> {
        let page_base = self.align_down(virtual_address);
        let entry = self.entries.get(&page_base)
            .ok_or(MemoryError::PageFault { address: virtual_address })?;

        if !caller_mode.can_access(&entry.security_mode) {
            return Err(MemoryError::SecurityViolation {
                required: entry.security_mode,
                actual: *caller_mode,
            });
        }

        let perms = entry.flags.to_permissions();
        if (required.read && !perms.read)
            || (required.write && !perms.write)
            || (required.execute && !perms.execute)
            || (required.ternary_compute && !perms.ternary_compute)
        {
            return Err(MemoryError::PermissionDenied {
                required: *required,
                actual: perms,
            });
        }

        Ok(())
    }

    pub fn map_range(
        &mut self,
        virtual_base: usize,
        physical_base: usize,
        size: usize,
        flags: PageFlags,
        security_mode: SecurityMode,
    ) -> MemoryResult<()> {
        let page_count = (size + self.page_size - 1) / self.page_size;

        for i in 0..page_count {
            let virt = virtual_base + i * self.page_size;
            let phys = physical_base + i * self.page_size;
            self.map(virt, phys, flags, security_mode)?;
        }

        Ok(())
    }

    pub fn unmap_range(&mut self, virtual_base: usize, size: usize) -> MemoryResult<()> {
        let page_count = (size + self.page_size - 1) / self.page_size;

        for i in 0..page_count {
            let virt = virtual_base + i * self.page_size;
            self.unmap(virt)?;
        }

        Ok(())
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    fn align_down(&self, address: usize) -> usize {
        address & !(self.page_size - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_flags_creation() {
        let flags = PageFlags::new(PAGE_PRESENT | PAGE_WRITABLE);
        assert!(flags.present());
        assert!(flags.writable());
        assert!(!flags.user_accessible());
    }

    #[test]
    fn test_page_flags_from_permissions() {
        let perms = Permissions::read_write();
        let flags = PageFlags::from_permissions(&perms, false);
        assert!(flags.present());
        assert!(flags.writable());
        assert!(!flags.user_accessible());
        assert!(flags.no_execute());
    }

    #[test]
    fn test_page_flags_ternary() {
        let perms = Permissions::ternary();
        let flags = PageFlags::from_permissions(&perms, true);
        assert!(flags.ternary_compute());
        assert!(flags.user_accessible());
    }

    #[test]
    fn test_page_flags_roundtrip() {
        let original = Permissions::read_write();
        let flags = PageFlags::from_permissions(&original, false);
        let restored = flags.to_permissions();
        assert_eq!(original.read, restored.read);
        assert_eq!(original.write, restored.write);
        assert_eq!(original.execute, restored.execute);
    }

    #[test]
    fn test_page_table_map_and_translate() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT | PAGE_WRITABLE);
        pt.map(0x1000, 0x2000, flags, SecurityMode::ModeOne).unwrap();

        let phys = pt.translate(0x1000).unwrap();
        assert_eq!(phys, 0x2000);
    }

    #[test]
    fn test_page_table_translate_with_offset() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT | PAGE_WRITABLE);
        pt.map(0x0, 0x10000, flags, SecurityMode::ModeOne).unwrap();

        let phys = pt.translate(0x100).unwrap();
        assert_eq!(phys, 0x10100);
    }

    #[test]
    fn test_page_table_page_fault() {
        let pt = PageTable::new();
        assert!(pt.translate(0x5000).is_err());
    }

    #[test]
    fn test_page_table_unmap() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT | PAGE_WRITABLE);
        pt.map(0x1000, 0x2000, flags, SecurityMode::ModeOne).unwrap();
        assert_eq!(pt.entry_count(), 1);

        pt.unmap(0x1000).unwrap();
        assert_eq!(pt.entry_count(), 0);
        assert!(pt.translate(0x1000).is_err());
    }

    #[test]
    fn test_page_table_duplicate_mapping() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT);
        pt.map(0x1000, 0x2000, flags, SecurityMode::ModeOne).unwrap();
        assert!(pt.map(0x1000, 0x3000, flags, SecurityMode::ModeOne).is_err());
    }

    #[test]
    fn test_page_table_map_range() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT | PAGE_WRITABLE);
        pt.map_range(0x0, 0x10000, 4096 * 4, flags, SecurityMode::ModeOne).unwrap();
        assert_eq!(pt.entry_count(), 4);

        assert_eq!(pt.translate(0x0).unwrap(), 0x10000);
        assert_eq!(pt.translate(0x1000).unwrap(), 0x11000);
        assert_eq!(pt.translate(0x2000).unwrap(), 0x12000);
        assert_eq!(pt.translate(0x3000).unwrap(), 0x13000);
    }

    #[test]
    fn test_page_table_unmap_range() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT);
        pt.map_range(0x0, 0x10000, 4096 * 4, flags, SecurityMode::ModeOne).unwrap();
        pt.unmap_range(0x0, 4096 * 4).unwrap();
        assert_eq!(pt.entry_count(), 0);
    }

    #[test]
    fn test_security_mode_access_check() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT | PAGE_WRITABLE);
        pt.map(0x1000, 0x2000, flags, SecurityMode::ModePhi).unwrap();

        let read_perm = Permissions::read_only();
        assert!(pt.check_access(0x1000, &read_perm, &SecurityMode::ModePhi).is_ok());
        assert!(pt.check_access(0x1000, &read_perm, &SecurityMode::ModeZero).is_err());
    }

    #[test]
    fn test_permission_access_check() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT); // read-only
        pt.map(0x1000, 0x2000, flags, SecurityMode::ModeOne).unwrap();

        let write_perm = Permissions::read_write();
        assert!(pt.check_access(0x1000, &write_perm, &SecurityMode::ModePhi).is_err());
    }

    #[test]
    fn test_lookup() {
        let mut pt = PageTable::new();
        let flags = PageFlags::new(PAGE_PRESENT | PAGE_TERNARY_COMPUTE);
        pt.map(0x1000, 0x2000, flags, SecurityMode::ModeOne).unwrap();

        let entry = pt.lookup(0x1000).unwrap();
        assert!(entry.flags.ternary_compute());
        assert_eq!(entry.security_mode, SecurityMode::ModeOne);
    }
}
