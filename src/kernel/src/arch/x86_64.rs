use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{
    ArchError, ArchId, ArchOps, ArchResult, BootInfo, BootOps, CpuFeatures,
    InterruptOps, InterruptVector, MemoryRegion, MemoryRegionType, MmuConfig,
    MmuOps, PrivilegeLevel, TimerConfig, TimerOps,
};

pub const PML4_ENTRIES: usize = 512;
pub const PDPT_ENTRIES: usize = 512;
pub const PD_ENTRIES: usize = 512;
pub const PT_ENTRIES: usize = 512;

pub const PAGE_SIZE_4K: u32 = 4096;
pub const PAGE_SIZE_2M: u32 = 2 * 1024 * 1024;
pub const PAGE_SIZE_1G: u32 = 1024 * 1024 * 1024;

pub const EFLAGS_IF: u64 = 1 << 9;
pub const EFLAGS_CF: u64 = 1 << 0;
pub const EFLAGS_ZF: u64 = 1 << 6;
pub const EFLAGS_SF: u64 = 1 << 7;

pub const MAX_APIC_VECTORS: u16 = 256;
pub const TSC_DEFAULT_FREQ: u64 = 2_400_000_000;

pub const REG_RAX: usize = 0;
pub const REG_RBX: usize = 1;
pub const REG_RCX: usize = 2;
pub const REG_RDX: usize = 3;
pub const REG_RSI: usize = 4;
pub const REG_RDI: usize = 5;
pub const REG_RBP: usize = 6;
pub const REG_RSP: usize = 7;
pub const REG_R8: usize = 8;
pub const REG_R9: usize = 9;
pub const REG_R10: usize = 10;
pub const REG_R11: usize = 11;
pub const REG_R12: usize = 12;
pub const REG_R13: usize = 13;
pub const REG_R14: usize = 14;
pub const REG_R15: usize = 15;

pub struct X86_64Arch {
    privilege_level: PrivilegeLevel,
    cr0: u64,
    cr2: u64,
    cr3: u64,
    cr4: u64,
    rflags: u64,
    general_regs: [u64; 16],
    rip: u64,
    interrupt_vectors: Vec<InterruptVector>,
    pending_interrupts: Vec<u16>,
    page_table: BTreeMap<u64, u64>,
    tlb_generation: u64,
    tsc_ticks: u64,
    tsc_target: u64,
    boot_info: BootInfo,
    early_init_done: bool,
    late_init_done: bool,
}

impl X86_64Arch {
    pub fn new() -> Self {
        let mut memory_map = BTreeMap::new();
        memory_map.insert(0x0, MemoryRegion {
            base: 0x0,
            size: 0x9_FC00,
            region_type: MemoryRegionType::Usable,
        });
        memory_map.insert(0x100000, MemoryRegion {
            base: 0x100000,
            size: 0x1F00000,
            region_type: MemoryRegionType::Usable,
        });

        Self {
            privilege_level: PrivilegeLevel::Kernel,
            cr0: 0x8000_0011,
            cr2: 0,
            cr3: 0,
            cr4: 0x0000_06A0,
            rflags: EFLAGS_IF,
            general_regs: [0u64; 16],
            rip: 0,
            interrupt_vectors: Vec::new(),
            pending_interrupts: Vec::new(),
            page_table: BTreeMap::new(),
            tlb_generation: 0,
            tsc_ticks: 0,
            tsc_target: 0,
            boot_info: BootInfo {
                kernel_start: 0x100000,
                kernel_end: 0x200000,
                memory_map,
                command_line: String::from("console=ttyS0 root=/dev/sda1"),
            },
            early_init_done: false,
            late_init_done: false,
        }
    }

    pub fn read_cr0(&self) -> u64 { self.cr0 }
    pub fn read_cr2(&self) -> u64 { self.cr2 }
    pub fn read_cr3(&self) -> u64 { self.cr3 }
    pub fn read_cr4(&self) -> u64 { self.cr4 }
    pub fn read_rflags(&self) -> u64 { self.rflags }
    pub fn read_rip(&self) -> u64 { self.rip }

    pub fn write_cr0(&mut self, val: u64) { self.cr0 = val; }
    pub fn write_cr3(&mut self, val: u64) { self.cr3 = val; }
    pub fn write_cr4(&mut self, val: u64) { self.cr4 = val; }

    pub fn read_gpr(&self, index: usize) -> ArchResult<u64> {
        if index < 16 {
            Ok(self.general_regs[index])
        } else {
            Err(ArchError::InvalidRegister)
        }
    }

    pub fn write_gpr(&mut self, index: usize, value: u64) -> ArchResult<()> {
        if index < 16 {
            self.general_regs[index] = value;
            Ok(())
        } else {
            Err(ArchError::InvalidRegister)
        }
    }
}

impl ArchOps for X86_64Arch {
    fn arch_id(&self) -> ArchId { ArchId::X86_64 }

    fn cpu_features(&self) -> CpuFeatures {
        CpuFeatures {
            has_ternary_coprocessor: true,
            has_phase_encryption_unit: true,
            has_femtosecond_timer: true,
            has_vector_extensions: true,
            has_hardware_crypto: true,
            cache_line_size: 64,
            max_physical_address_bits: 52,
            max_virtual_address_bits: 48,
        }
    }

    fn privilege_level(&self) -> PrivilegeLevel { self.privilege_level }

    fn set_privilege_level(&mut self, level: PrivilegeLevel) -> ArchResult<()> {
        if self.privilege_level == PrivilegeLevel::User && level == PrivilegeLevel::Kernel {
            return Err(ArchError::PrivilegeViolation);
        }
        self.privilege_level = level;
        Ok(())
    }

    fn enable_interrupts(&mut self) {
        self.rflags |= EFLAGS_IF;
    }

    fn disable_interrupts(&mut self) {
        self.rflags &= !EFLAGS_IF;
    }

    fn interrupts_enabled(&self) -> bool {
        self.rflags & EFLAGS_IF != 0
    }

    fn halt(&self) {}

    fn memory_barrier(&self) {}
}

impl MmuOps for X86_64Arch {
    fn mmu_config(&self) -> MmuConfig {
        MmuConfig {
            page_table_levels: 4,
            page_size: PAGE_SIZE_4K,
            supports_huge_pages: true,
            supports_ternary_tagging: true,
            address_space_bits: 48,
        }
    }

    fn map_page(&mut self, virt: u64, phys: u64, flags: u64) -> ArchResult<()> {
        if virt % PAGE_SIZE_4K as u64 != 0 {
            return Err(ArchError::AlignmentError);
        }
        if phys % PAGE_SIZE_4K as u64 != 0 {
            return Err(ArchError::AlignmentError);
        }
        self.page_table.insert(virt, phys | flags);
        Ok(())
    }

    fn unmap_page(&mut self, virt: u64) -> ArchResult<()> {
        if virt % PAGE_SIZE_4K as u64 != 0 {
            return Err(ArchError::AlignmentError);
        }
        self.page_table.remove(&virt)
            .map(|_| ())
            .ok_or(ArchError::InvalidAddress)
    }

    fn translate(&self, virt: u64) -> ArchResult<u64> {
        let page_base = virt & !(PAGE_SIZE_4K as u64 - 1);
        let offset = virt & (PAGE_SIZE_4K as u64 - 1);
        self.page_table.get(&page_base)
            .map(|entry| (entry & !(PAGE_SIZE_4K as u64 - 1)) + offset)
            .ok_or(ArchError::InvalidAddress)
    }

    fn flush_tlb(&mut self, _virt: u64) {
        self.tlb_generation += 1;
    }

    fn flush_tlb_all(&mut self) {
        self.tlb_generation += 1;
    }
}

impl InterruptOps for X86_64Arch {
    fn configure_vector(&mut self, vector: InterruptVector) -> ArchResult<()> {
        if vector.vector_number >= MAX_APIC_VECTORS {
            return Err(ArchError::InvalidRegister);
        }
        self.interrupt_vectors.retain(|v| v.vector_number != vector.vector_number);
        self.interrupt_vectors.push(vector);
        Ok(())
    }

    fn enable_vector(&mut self, vector_number: u16) -> ArchResult<()> {
        if vector_number >= MAX_APIC_VECTORS {
            return Err(ArchError::InvalidRegister);
        }
        for v in self.interrupt_vectors.iter_mut() {
            if v.vector_number == vector_number {
                v.is_enabled = true;
                return Ok(());
            }
        }
        Err(ArchError::InvalidRegister)
    }

    fn disable_vector(&mut self, vector_number: u16) -> ArchResult<()> {
        if vector_number >= MAX_APIC_VECTORS {
            return Err(ArchError::InvalidRegister);
        }
        for v in self.interrupt_vectors.iter_mut() {
            if v.vector_number == vector_number {
                v.is_enabled = false;
                return Ok(());
            }
        }
        Err(ArchError::InvalidRegister)
    }

    fn acknowledge(&mut self, vector_number: u16) {
        self.pending_interrupts.retain(|&v| v != vector_number);
    }

    fn is_pending(&self, vector_number: u16) -> bool {
        self.pending_interrupts.contains(&vector_number)
    }

    fn max_vectors(&self) -> u16 { MAX_APIC_VECTORS }
}

impl TimerOps for X86_64Arch {
    fn timer_config(&self) -> TimerConfig {
        TimerConfig {
            frequency_hz: TSC_DEFAULT_FREQ,
            supports_femtosecond: true,
            resolution_bits: 64,
        }
    }

    fn current_ticks(&self) -> u64 { self.tsc_ticks }

    fn set_timer(&mut self, ticks: u64) -> ArchResult<()> {
        self.tsc_target = self.tsc_ticks + ticks;
        Ok(())
    }

    fn timer_elapsed(&self) -> u64 { self.tsc_ticks }
}

impl BootOps for X86_64Arch {
    fn boot_info(&self) -> &BootInfo { &self.boot_info }

    fn init_early(&mut self) -> ArchResult<()> {
        self.cr0 = 0x8005_003B;
        self.cr4 = 0x0000_06A0;
        self.rflags = EFLAGS_IF;
        self.early_init_done = true;
        Ok(())
    }

    fn init_late(&mut self) -> ArchResult<()> {
        if !self.early_init_done {
            return Err(ArchError::InitializationFailed);
        }
        self.late_init_done = true;
        Ok(())
    }

    fn arch_name(&self) -> &str { "x86_64" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_arch() -> X86_64Arch {
        X86_64Arch::new()
    }

    #[test]
    fn test_arch_id() {
        let arch = make_arch();
        assert_eq!(arch.arch_id(), ArchId::X86_64);
    }

    #[test]
    fn test_cpu_features() {
        let arch = make_arch();
        let f = arch.cpu_features();
        assert!(f.has_ternary_coprocessor);
        assert!(f.has_phase_encryption_unit);
        assert!(f.has_femtosecond_timer);
        assert!(f.has_vector_extensions);
        assert!(f.has_hardware_crypto);
        assert_eq!(f.cache_line_size, 64);
        assert_eq!(f.max_physical_address_bits, 52);
        assert_eq!(f.max_virtual_address_bits, 48);
    }

    #[test]
    fn test_privilege_level_default() {
        let arch = make_arch();
        assert_eq!(arch.privilege_level(), PrivilegeLevel::Kernel);
    }

    #[test]
    fn test_set_privilege_level() {
        let mut arch = make_arch();
        arch.set_privilege_level(PrivilegeLevel::User).unwrap();
        assert_eq!(arch.privilege_level(), PrivilegeLevel::User);
    }

    #[test]
    fn test_privilege_violation() {
        let mut arch = make_arch();
        arch.set_privilege_level(PrivilegeLevel::User).unwrap();
        let result = arch.set_privilege_level(PrivilegeLevel::Kernel);
        assert_eq!(result, Err(ArchError::PrivilegeViolation));
    }

    #[test]
    fn test_interrupts_enable_disable() {
        let mut arch = make_arch();
        assert!(arch.interrupts_enabled());
        arch.disable_interrupts();
        assert!(!arch.interrupts_enabled());
        arch.enable_interrupts();
        assert!(arch.interrupts_enabled());
    }

    #[test]
    fn test_eflags_if_bit() {
        let mut arch = make_arch();
        assert!(arch.read_rflags() & EFLAGS_IF != 0);
        arch.disable_interrupts();
        assert!(arch.read_rflags() & EFLAGS_IF == 0);
    }

    #[test]
    fn test_cr_registers() {
        let mut arch = make_arch();
        arch.write_cr0(0xDEAD);
        assert_eq!(arch.read_cr0(), 0xDEAD);
        arch.write_cr3(0x1000);
        assert_eq!(arch.read_cr3(), 0x1000);
        arch.write_cr4(0xBEEF);
        assert_eq!(arch.read_cr4(), 0xBEEF);
    }

    #[test]
    fn test_gpr_read_write() {
        let mut arch = make_arch();
        arch.write_gpr(REG_RAX, 42).unwrap();
        assert_eq!(arch.read_gpr(REG_RAX).unwrap(), 42);
        arch.write_gpr(REG_R15, 0xFF).unwrap();
        assert_eq!(arch.read_gpr(REG_R15).unwrap(), 0xFF);
    }

    #[test]
    fn test_gpr_invalid_register() {
        let mut arch = make_arch();
        assert_eq!(arch.read_gpr(16), Err(ArchError::InvalidRegister));
        assert_eq!(arch.write_gpr(16, 0), Err(ArchError::InvalidRegister));
    }

    #[test]
    fn test_mmu_config() {
        let arch = make_arch();
        let cfg = arch.mmu_config();
        assert_eq!(cfg.page_table_levels, 4);
        assert_eq!(cfg.page_size, PAGE_SIZE_4K);
        assert!(cfg.supports_huge_pages);
        assert!(cfg.supports_ternary_tagging);
        assert_eq!(cfg.address_space_bits, 48);
    }

    #[test]
    fn test_map_and_translate() {
        let mut arch = make_arch();
        arch.map_page(0x1000, 0x2000, 0x3).unwrap();
        assert_eq!(arch.translate(0x1000).unwrap(), 0x2000);
        assert_eq!(arch.translate(0x1010).unwrap(), 0x2010);
    }

    #[test]
    fn test_map_unaligned() {
        let mut arch = make_arch();
        assert_eq!(arch.map_page(0x1001, 0x2000, 0), Err(ArchError::AlignmentError));
        assert_eq!(arch.map_page(0x1000, 0x2001, 0), Err(ArchError::AlignmentError));
    }

    #[test]
    fn test_unmap_page() {
        let mut arch = make_arch();
        arch.map_page(0x1000, 0x2000, 0x3).unwrap();
        arch.unmap_page(0x1000).unwrap();
        assert_eq!(arch.translate(0x1000), Err(ArchError::InvalidAddress));
    }

    #[test]
    fn test_unmap_nonexistent() {
        let mut arch = make_arch();
        assert_eq!(arch.unmap_page(0x1000), Err(ArchError::InvalidAddress));
    }

    #[test]
    fn test_translate_unmapped() {
        let arch = make_arch();
        assert_eq!(arch.translate(0xDEAD_0000), Err(ArchError::InvalidAddress));
    }

    #[test]
    fn test_flush_tlb() {
        let mut arch = make_arch();
        let gen = arch.tlb_generation;
        arch.flush_tlb(0x1000);
        assert_eq!(arch.tlb_generation, gen + 1);
        arch.flush_tlb_all();
        assert_eq!(arch.tlb_generation, gen + 2);
    }

    #[test]
    fn test_configure_interrupt_vector() {
        let mut arch = make_arch();
        let vec = InterruptVector {
            vector_number: 32,
            handler_address: 0xFFFF_0000,
            privilege_level: PrivilegeLevel::Kernel,
            is_enabled: true,
        };
        arch.configure_vector(vec).unwrap();
        assert!(arch.interrupt_vectors.iter().any(|v| v.vector_number == 32));
    }

    #[test]
    fn test_configure_invalid_vector() {
        let mut arch = make_arch();
        let vec = InterruptVector {
            vector_number: 256,
            handler_address: 0,
            privilege_level: PrivilegeLevel::Kernel,
            is_enabled: false,
        };
        assert_eq!(arch.configure_vector(vec), Err(ArchError::InvalidRegister));
    }

    #[test]
    fn test_enable_disable_vector() {
        let mut arch = make_arch();
        let vec = InterruptVector {
            vector_number: 42,
            handler_address: 0x1000,
            privilege_level: PrivilegeLevel::Kernel,
            is_enabled: false,
        };
        arch.configure_vector(vec).unwrap();
        arch.enable_vector(42).unwrap();
        assert!(arch.interrupt_vectors.iter().find(|v| v.vector_number == 42).unwrap().is_enabled);
        arch.disable_vector(42).unwrap();
        assert!(!arch.interrupt_vectors.iter().find(|v| v.vector_number == 42).unwrap().is_enabled);
    }

    #[test]
    fn test_max_vectors() {
        let arch = make_arch();
        assert_eq!(arch.max_vectors(), 256);
    }

    #[test]
    fn test_interrupt_pending_acknowledge() {
        let mut arch = make_arch();
        arch.pending_interrupts.push(10);
        assert!(arch.is_pending(10));
        arch.acknowledge(10);
        assert!(!arch.is_pending(10));
    }

    #[test]
    fn test_timer_config() {
        let arch = make_arch();
        let cfg = arch.timer_config();
        assert_eq!(cfg.frequency_hz, TSC_DEFAULT_FREQ);
        assert!(cfg.supports_femtosecond);
        assert_eq!(cfg.resolution_bits, 64);
    }

    #[test]
    fn test_timer_set_and_ticks() {
        let mut arch = make_arch();
        assert_eq!(arch.current_ticks(), 0);
        arch.set_timer(1000).unwrap();
        assert_eq!(arch.tsc_target, 1000);
        assert_eq!(arch.timer_elapsed(), 0);
    }

    #[test]
    fn test_boot_info() {
        let arch = make_arch();
        let info = arch.boot_info();
        assert_eq!(info.kernel_start, 0x100000);
        assert_eq!(info.kernel_end, 0x200000);
        assert!(!info.memory_map.is_empty());
        assert!(info.command_line.contains("console=ttyS0"));
    }

    #[test]
    fn test_init_early_late() {
        let mut arch = make_arch();
        arch.init_early().unwrap();
        assert!(arch.early_init_done);
        arch.init_late().unwrap();
        assert!(arch.late_init_done);
    }

    #[test]
    fn test_init_late_without_early() {
        let mut arch = make_arch();
        assert_eq!(arch.init_late(), Err(ArchError::InitializationFailed));
    }

    #[test]
    fn test_arch_name() {
        let arch = make_arch();
        assert_eq!(arch.arch_name(), "x86_64");
    }

    #[test]
    fn test_page_table_constants() {
        assert_eq!(PML4_ENTRIES, 512);
        assert_eq!(PDPT_ENTRIES, 512);
        assert_eq!(PD_ENTRIES, 512);
        assert_eq!(PT_ENTRIES, 512);
    }

    #[test]
    fn test_halt_and_memory_barrier() {
        let arch = make_arch();
        arch.halt();
        arch.memory_barrier();
    }
}
