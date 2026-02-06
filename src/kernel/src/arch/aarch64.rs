use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{
    ArchError, ArchId, ArchOps, ArchResult, BootInfo, BootOps, CpuFeatures,
    InterruptOps, InterruptVector, MemoryRegion, MemoryRegionType, MmuConfig,
    MmuOps, PrivilegeLevel, TimerConfig, TimerOps,
};

pub const PAGE_SIZE_4K: u32 = 4096;
pub const PAGE_SIZE_16K: u32 = 16384;
pub const PAGE_SIZE_64K: u32 = 65536;

pub const GIC_MAX_VECTORS: u16 = 1020;
pub const GENERIC_TIMER_FREQ: u64 = 62_500_000;

pub const EL0_USER: u8 = 0;
pub const EL1_KERNEL: u8 = 1;
pub const EL2_HYPERVISOR: u8 = 2;
pub const EL3_SECURE_MONITOR: u8 = 3;

pub const DAIF_IRQ: u64 = 1 << 7;
pub const DAIF_FIQ: u64 = 1 << 6;

pub struct Aarch64Arch {
    privilege_level: PrivilegeLevel,
    exception_level: u8,
    sctlr_el1: u64,
    tcr_el1: u64,
    ttbr0_el1: u64,
    ttbr1_el1: u64,
    mair_el1: u64,
    daif: u64,
    general_regs: [u64; 31],
    sp: u64,
    pc: u64,
    pstate: u64,
    interrupt_vectors: Vec<InterruptVector>,
    pending_interrupts: Vec<u16>,
    page_table: BTreeMap<u64, u64>,
    tlb_generation: u64,
    timer_ticks: u64,
    timer_target: u64,
    boot_info: BootInfo,
    early_init_done: bool,
    late_init_done: bool,
}

impl Aarch64Arch {
    pub fn new() -> Self {
        let mut memory_map = BTreeMap::new();
        memory_map.insert(0x4000_0000, MemoryRegion {
            base: 0x4000_0000,
            size: 0x4000_0000,
            region_type: MemoryRegionType::Usable,
        });
        memory_map.insert(0x0, MemoryRegion {
            base: 0x0,
            size: 0x0800_0000,
            region_type: MemoryRegionType::Reserved,
        });

        Self {
            privilege_level: PrivilegeLevel::Kernel,
            exception_level: EL1_KERNEL,
            sctlr_el1: 0x3050_5185,
            tcr_el1: 0,
            ttbr0_el1: 0,
            ttbr1_el1: 0,
            mair_el1: 0,
            daif: 0,
            general_regs: [0u64; 31],
            sp: 0,
            pc: 0,
            pstate: 0,
            interrupt_vectors: Vec::new(),
            pending_interrupts: Vec::new(),
            page_table: BTreeMap::new(),
            tlb_generation: 0,
            timer_ticks: 0,
            timer_target: 0,
            boot_info: BootInfo {
                kernel_start: 0x4008_0000,
                kernel_end: 0x4018_0000,
                memory_map,
                command_line: String::from("earlycon=pl011,0x09000000"),
            },
            early_init_done: false,
            late_init_done: false,
        }
    }

    pub fn exception_level(&self) -> u8 { self.exception_level }

    pub fn read_sctlr_el1(&self) -> u64 { self.sctlr_el1 }
    pub fn read_tcr_el1(&self) -> u64 { self.tcr_el1 }
    pub fn read_ttbr0_el1(&self) -> u64 { self.ttbr0_el1 }
    pub fn read_ttbr1_el1(&self) -> u64 { self.ttbr1_el1 }
    pub fn read_daif(&self) -> u64 { self.daif }

    pub fn write_ttbr0_el1(&mut self, val: u64) { self.ttbr0_el1 = val; }
    pub fn write_ttbr1_el1(&mut self, val: u64) { self.ttbr1_el1 = val; }

    pub fn read_gpr(&self, index: usize) -> ArchResult<u64> {
        if index < 31 {
            Ok(self.general_regs[index])
        } else {
            Err(ArchError::InvalidRegister)
        }
    }

    pub fn write_gpr(&mut self, index: usize, value: u64) -> ArchResult<()> {
        if index < 31 {
            self.general_regs[index] = value;
            Ok(())
        } else {
            Err(ArchError::InvalidRegister)
        }
    }

    fn el_to_privilege(el: u8) -> PrivilegeLevel {
        match el {
            0 => PrivilegeLevel::User,
            1 => PrivilegeLevel::Supervisor,
            _ => PrivilegeLevel::Kernel,
        }
    }

    fn privilege_to_el(level: PrivilegeLevel) -> u8 {
        match level {
            PrivilegeLevel::User => EL0_USER,
            PrivilegeLevel::Supervisor => EL1_KERNEL,
            PrivilegeLevel::Kernel => EL2_HYPERVISOR,
        }
    }
}

impl ArchOps for Aarch64Arch {
    fn arch_id(&self) -> ArchId { ArchId::Aarch64 }

    fn cpu_features(&self) -> CpuFeatures {
        CpuFeatures {
            has_ternary_coprocessor: true,
            has_phase_encryption_unit: true,
            has_femtosecond_timer: true,
            has_vector_extensions: true,
            has_hardware_crypto: true,
            cache_line_size: 64,
            max_physical_address_bits: 48,
            max_virtual_address_bits: 48,
        }
    }

    fn privilege_level(&self) -> PrivilegeLevel { self.privilege_level }

    fn set_privilege_level(&mut self, level: PrivilegeLevel) -> ArchResult<()> {
        if self.privilege_level == PrivilegeLevel::User && level == PrivilegeLevel::Kernel {
            return Err(ArchError::PrivilegeViolation);
        }
        self.privilege_level = level;
        self.exception_level = Self::privilege_to_el(level);
        Ok(())
    }

    fn enable_interrupts(&mut self) {
        self.daif &= !(DAIF_IRQ | DAIF_FIQ);
    }

    fn disable_interrupts(&mut self) {
        self.daif |= DAIF_IRQ | DAIF_FIQ;
    }

    fn interrupts_enabled(&self) -> bool {
        self.daif & DAIF_IRQ == 0
    }

    fn halt(&self) {}

    fn memory_barrier(&self) {}
}

impl MmuOps for Aarch64Arch {
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

impl InterruptOps for Aarch64Arch {
    fn configure_vector(&mut self, vector: InterruptVector) -> ArchResult<()> {
        if vector.vector_number >= GIC_MAX_VECTORS {
            return Err(ArchError::InvalidRegister);
        }
        self.interrupt_vectors.retain(|v| v.vector_number != vector.vector_number);
        self.interrupt_vectors.push(vector);
        Ok(())
    }

    fn enable_vector(&mut self, vector_number: u16) -> ArchResult<()> {
        if vector_number >= GIC_MAX_VECTORS {
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
        if vector_number >= GIC_MAX_VECTORS {
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

    fn max_vectors(&self) -> u16 { GIC_MAX_VECTORS }
}

impl TimerOps for Aarch64Arch {
    fn timer_config(&self) -> TimerConfig {
        TimerConfig {
            frequency_hz: GENERIC_TIMER_FREQ,
            supports_femtosecond: true,
            resolution_bits: 64,
        }
    }

    fn current_ticks(&self) -> u64 { self.timer_ticks }

    fn set_timer(&mut self, ticks: u64) -> ArchResult<()> {
        self.timer_target = self.timer_ticks + ticks;
        Ok(())
    }

    fn timer_elapsed(&self) -> u64 { self.timer_ticks }
}

impl BootOps for Aarch64Arch {
    fn boot_info(&self) -> &BootInfo { &self.boot_info }

    fn init_early(&mut self) -> ArchResult<()> {
        self.sctlr_el1 = 0x3050_5185;
        self.mair_el1 = 0x00FF_4400_BB44_FF00;
        self.daif = 0;
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

    fn arch_name(&self) -> &str { "aarch64" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_arch() -> Aarch64Arch {
        Aarch64Arch::new()
    }

    #[test]
    fn test_arch_id() {
        let arch = make_arch();
        assert_eq!(arch.arch_id(), ArchId::Aarch64);
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
        assert_eq!(f.max_physical_address_bits, 48);
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
        assert_eq!(arch.exception_level(), EL0_USER);
    }

    #[test]
    fn test_privilege_violation() {
        let mut arch = make_arch();
        arch.set_privilege_level(PrivilegeLevel::User).unwrap();
        assert_eq!(arch.set_privilege_level(PrivilegeLevel::Kernel), Err(ArchError::PrivilegeViolation));
    }

    #[test]
    fn test_exception_level_mapping() {
        let mut arch = make_arch();
        arch.set_privilege_level(PrivilegeLevel::User).unwrap();
        assert_eq!(arch.exception_level(), EL0_USER);
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
    fn test_daif_bits() {
        let mut arch = make_arch();
        arch.disable_interrupts();
        assert!(arch.read_daif() & DAIF_IRQ != 0);
        assert!(arch.read_daif() & DAIF_FIQ != 0);
        arch.enable_interrupts();
        assert!(arch.read_daif() & DAIF_IRQ == 0);
    }

    #[test]
    fn test_system_registers() {
        let mut arch = make_arch();
        arch.write_ttbr0_el1(0x1000);
        assert_eq!(arch.read_ttbr0_el1(), 0x1000);
        arch.write_ttbr1_el1(0x2000);
        assert_eq!(arch.read_ttbr1_el1(), 0x2000);
    }

    #[test]
    fn test_gpr_read_write() {
        let mut arch = make_arch();
        arch.write_gpr(0, 42).unwrap();
        assert_eq!(arch.read_gpr(0).unwrap(), 42);
        arch.write_gpr(30, 0xFF).unwrap();
        assert_eq!(arch.read_gpr(30).unwrap(), 0xFF);
    }

    #[test]
    fn test_gpr_invalid_register() {
        let mut arch = make_arch();
        assert_eq!(arch.read_gpr(31), Err(ArchError::InvalidRegister));
        assert_eq!(arch.write_gpr(31, 0), Err(ArchError::InvalidRegister));
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
    }

    #[test]
    fn test_configure_invalid_vector() {
        let mut arch = make_arch();
        let vec = InterruptVector {
            vector_number: 1020,
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
        arch.disable_vector(42).unwrap();
    }

    #[test]
    fn test_max_vectors() {
        let arch = make_arch();
        assert_eq!(arch.max_vectors(), 1020);
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
        assert_eq!(cfg.frequency_hz, GENERIC_TIMER_FREQ);
        assert!(cfg.supports_femtosecond);
        assert_eq!(cfg.resolution_bits, 64);
    }

    #[test]
    fn test_timer_set_and_ticks() {
        let mut arch = make_arch();
        assert_eq!(arch.current_ticks(), 0);
        arch.set_timer(500).unwrap();
        assert_eq!(arch.timer_target, 500);
    }

    #[test]
    fn test_boot_info() {
        let arch = make_arch();
        let info = arch.boot_info();
        assert_eq!(info.kernel_start, 0x4008_0000);
        assert_eq!(info.kernel_end, 0x4018_0000);
        assert!(!info.memory_map.is_empty());
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
        assert_eq!(arch.arch_name(), "aarch64");
    }

    #[test]
    fn test_page_granule_constants() {
        assert_eq!(PAGE_SIZE_4K, 4096);
        assert_eq!(PAGE_SIZE_16K, 16384);
        assert_eq!(PAGE_SIZE_64K, 65536);
    }

    #[test]
    fn test_halt_and_memory_barrier() {
        let arch = make_arch();
        arch.halt();
        arch.memory_barrier();
    }
}
