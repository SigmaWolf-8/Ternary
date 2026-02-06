use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{
    ArchError, ArchId, ArchOps, ArchResult, BootInfo, BootOps, CpuFeatures,
    InterruptOps, InterruptVector, MemoryRegion, MemoryRegionType, MmuConfig,
    MmuOps, PrivilegeLevel, TimerConfig, TimerOps,
};

pub const SV48_PAGE_SIZE: u32 = 4096;
pub const SV48_LEVELS: u8 = 4;
pub const SV48_ADDRESS_BITS: u8 = 48;

pub const PLIC_MAX_VECTORS: u16 = 1024;
pub const MTIME_DEFAULT_FREQ: u64 = 10_000_000;

pub const MODE_MACHINE: u8 = 3;
pub const MODE_SUPERVISOR: u8 = 1;
pub const MODE_USER: u8 = 0;

pub const MSTATUS_MIE: u64 = 1 << 3;
pub const MSTATUS_SIE: u64 = 1 << 1;
pub const SSTATUS_SIE: u64 = 1 << 1;

pub const CSR_MSTATUS: u16 = 0x300;
pub const CSR_MTVEC: u16 = 0x305;
pub const CSR_MEPC: u16 = 0x341;
pub const CSR_MCAUSE: u16 = 0x342;
pub const CSR_SATP: u16 = 0x180;
pub const CSR_SSTATUS: u16 = 0x100;

pub struct RiscV64Arch {
    privilege_level: PrivilegeLevel,
    privilege_mode: u8,
    mstatus: u64,
    sstatus: u64,
    mtvec: u64,
    stvec: u64,
    mepc: u64,
    sepc: u64,
    mcause: u64,
    scause: u64,
    satp: u64,
    mtime: u64,
    mtimecmp: u64,
    general_regs: [u64; 32],
    pc: u64,
    interrupt_vectors: Vec<InterruptVector>,
    pending_interrupts: Vec<u16>,
    page_table: BTreeMap<u64, u64>,
    tlb_generation: u64,
    boot_info: BootInfo,
    early_init_done: bool,
    late_init_done: bool,
}

impl RiscV64Arch {
    pub fn new() -> Self {
        let mut memory_map = BTreeMap::new();
        memory_map.insert(0x8000_0000, MemoryRegion {
            base: 0x8000_0000,
            size: 0x8000_0000,
            region_type: MemoryRegionType::Usable,
        });
        memory_map.insert(0x0, MemoryRegion {
            base: 0x0,
            size: 0x1000,
            region_type: MemoryRegionType::Reserved,
        });

        Self {
            privilege_level: PrivilegeLevel::Kernel,
            privilege_mode: MODE_MACHINE,
            mstatus: MSTATUS_MIE,
            sstatus: SSTATUS_SIE,
            mtvec: 0,
            stvec: 0,
            mepc: 0,
            sepc: 0,
            mcause: 0,
            scause: 0,
            satp: 0,
            mtime: 0,
            mtimecmp: 0,
            general_regs: [0u64; 32],
            pc: 0x8000_0000,
            interrupt_vectors: Vec::new(),
            pending_interrupts: Vec::new(),
            page_table: BTreeMap::new(),
            tlb_generation: 0,
            boot_info: BootInfo {
                kernel_start: 0x8020_0000,
                kernel_end: 0x8040_0000,
                memory_map,
                command_line: String::from("earlycon=sbi console=ttyS0"),
            },
            early_init_done: false,
            late_init_done: false,
        }
    }

    pub fn privilege_mode(&self) -> u8 { self.privilege_mode }

    pub fn read_mstatus(&self) -> u64 { self.mstatus }
    pub fn read_sstatus(&self) -> u64 { self.sstatus }
    pub fn read_satp(&self) -> u64 { self.satp }
    pub fn read_mtvec(&self) -> u64 { self.mtvec }
    pub fn read_mepc(&self) -> u64 { self.mepc }
    pub fn read_mcause(&self) -> u64 { self.mcause }
    pub fn read_mtime(&self) -> u64 { self.mtime }
    pub fn read_mtimecmp(&self) -> u64 { self.mtimecmp }

    pub fn write_satp(&mut self, val: u64) { self.satp = val; }
    pub fn write_mtvec(&mut self, val: u64) { self.mtvec = val; }
    pub fn write_mtimecmp(&mut self, val: u64) { self.mtimecmp = val; }

    pub fn read_gpr(&self, index: usize) -> ArchResult<u64> {
        if index == 0 {
            Ok(0)
        } else if index < 32 {
            Ok(self.general_regs[index])
        } else {
            Err(ArchError::InvalidRegister)
        }
    }

    pub fn write_gpr(&mut self, index: usize, value: u64) -> ArchResult<()> {
        if index == 0 {
            Ok(())
        } else if index < 32 {
            self.general_regs[index] = value;
            Ok(())
        } else {
            Err(ArchError::InvalidRegister)
        }
    }

    fn mode_to_privilege(mode: u8) -> PrivilegeLevel {
        match mode {
            MODE_USER => PrivilegeLevel::User,
            MODE_SUPERVISOR => PrivilegeLevel::Supervisor,
            _ => PrivilegeLevel::Kernel,
        }
    }

    fn privilege_to_mode(level: PrivilegeLevel) -> u8 {
        match level {
            PrivilegeLevel::User => MODE_USER,
            PrivilegeLevel::Supervisor => MODE_SUPERVISOR,
            PrivilegeLevel::Kernel => MODE_MACHINE,
        }
    }
}

impl ArchOps for RiscV64Arch {
    fn arch_id(&self) -> ArchId { ArchId::RiscV64 }

    fn cpu_features(&self) -> CpuFeatures {
        CpuFeatures {
            has_ternary_coprocessor: true,
            has_phase_encryption_unit: true,
            has_femtosecond_timer: true,
            has_vector_extensions: true,
            has_hardware_crypto: true,
            cache_line_size: 64,
            max_physical_address_bits: 56,
            max_virtual_address_bits: 48,
        }
    }

    fn privilege_level(&self) -> PrivilegeLevel { self.privilege_level }

    fn set_privilege_level(&mut self, level: PrivilegeLevel) -> ArchResult<()> {
        if self.privilege_level == PrivilegeLevel::User && level == PrivilegeLevel::Kernel {
            return Err(ArchError::PrivilegeViolation);
        }
        self.privilege_level = level;
        self.privilege_mode = Self::privilege_to_mode(level);
        Ok(())
    }

    fn enable_interrupts(&mut self) {
        self.mstatus |= MSTATUS_MIE;
        self.sstatus |= SSTATUS_SIE;
    }

    fn disable_interrupts(&mut self) {
        self.mstatus &= !MSTATUS_MIE;
        self.sstatus &= !SSTATUS_SIE;
    }

    fn interrupts_enabled(&self) -> bool {
        self.mstatus & MSTATUS_MIE != 0
    }

    fn halt(&self) {}

    fn memory_barrier(&self) {}
}

impl MmuOps for RiscV64Arch {
    fn mmu_config(&self) -> MmuConfig {
        MmuConfig {
            page_table_levels: SV48_LEVELS,
            page_size: SV48_PAGE_SIZE,
            supports_huge_pages: true,
            supports_ternary_tagging: true,
            address_space_bits: SV48_ADDRESS_BITS,
        }
    }

    fn map_page(&mut self, virt: u64, phys: u64, flags: u64) -> ArchResult<()> {
        if virt % SV48_PAGE_SIZE as u64 != 0 {
            return Err(ArchError::AlignmentError);
        }
        if phys % SV48_PAGE_SIZE as u64 != 0 {
            return Err(ArchError::AlignmentError);
        }
        self.page_table.insert(virt, phys | flags);
        Ok(())
    }

    fn unmap_page(&mut self, virt: u64) -> ArchResult<()> {
        if virt % SV48_PAGE_SIZE as u64 != 0 {
            return Err(ArchError::AlignmentError);
        }
        self.page_table.remove(&virt)
            .map(|_| ())
            .ok_or(ArchError::InvalidAddress)
    }

    fn translate(&self, virt: u64) -> ArchResult<u64> {
        let page_base = virt & !(SV48_PAGE_SIZE as u64 - 1);
        let offset = virt & (SV48_PAGE_SIZE as u64 - 1);
        self.page_table.get(&page_base)
            .map(|entry| (entry & !(SV48_PAGE_SIZE as u64 - 1)) + offset)
            .ok_or(ArchError::InvalidAddress)
    }

    fn flush_tlb(&mut self, _virt: u64) {
        self.tlb_generation += 1;
    }

    fn flush_tlb_all(&mut self) {
        self.tlb_generation += 1;
    }
}

impl InterruptOps for RiscV64Arch {
    fn configure_vector(&mut self, vector: InterruptVector) -> ArchResult<()> {
        if vector.vector_number >= PLIC_MAX_VECTORS {
            return Err(ArchError::InvalidRegister);
        }
        self.interrupt_vectors.retain(|v| v.vector_number != vector.vector_number);
        self.interrupt_vectors.push(vector);
        Ok(())
    }

    fn enable_vector(&mut self, vector_number: u16) -> ArchResult<()> {
        if vector_number >= PLIC_MAX_VECTORS {
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
        if vector_number >= PLIC_MAX_VECTORS {
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

    fn max_vectors(&self) -> u16 { PLIC_MAX_VECTORS }
}

impl TimerOps for RiscV64Arch {
    fn timer_config(&self) -> TimerConfig {
        TimerConfig {
            frequency_hz: MTIME_DEFAULT_FREQ,
            supports_femtosecond: true,
            resolution_bits: 64,
        }
    }

    fn current_ticks(&self) -> u64 { self.mtime }

    fn set_timer(&mut self, ticks: u64) -> ArchResult<()> {
        self.mtimecmp = self.mtime + ticks;
        Ok(())
    }

    fn timer_elapsed(&self) -> u64 { self.mtime }
}

impl BootOps for RiscV64Arch {
    fn boot_info(&self) -> &BootInfo { &self.boot_info }

    fn init_early(&mut self) -> ArchResult<()> {
        self.mstatus = MSTATUS_MIE;
        self.mtvec = 0x8000_0000;
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

    fn arch_name(&self) -> &str { "riscv64" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_arch() -> RiscV64Arch {
        RiscV64Arch::new()
    }

    #[test]
    fn test_arch_id() {
        let arch = make_arch();
        assert_eq!(arch.arch_id(), ArchId::RiscV64);
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
        assert_eq!(f.max_physical_address_bits, 56);
        assert_eq!(f.max_virtual_address_bits, 48);
    }

    #[test]
    fn test_privilege_level_default() {
        let arch = make_arch();
        assert_eq!(arch.privilege_level(), PrivilegeLevel::Kernel);
        assert_eq!(arch.privilege_mode(), MODE_MACHINE);
    }

    #[test]
    fn test_set_privilege_level() {
        let mut arch = make_arch();
        arch.set_privilege_level(PrivilegeLevel::Supervisor).unwrap();
        assert_eq!(arch.privilege_level(), PrivilegeLevel::Supervisor);
        assert_eq!(arch.privilege_mode(), MODE_SUPERVISOR);
    }

    #[test]
    fn test_set_privilege_user() {
        let mut arch = make_arch();
        arch.set_privilege_level(PrivilegeLevel::User).unwrap();
        assert_eq!(arch.privilege_level(), PrivilegeLevel::User);
        assert_eq!(arch.privilege_mode(), MODE_USER);
    }

    #[test]
    fn test_privilege_violation() {
        let mut arch = make_arch();
        arch.set_privilege_level(PrivilegeLevel::User).unwrap();
        assert_eq!(arch.set_privilege_level(PrivilegeLevel::Kernel), Err(ArchError::PrivilegeViolation));
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
    fn test_mstatus_mie_bit() {
        let mut arch = make_arch();
        arch.disable_interrupts();
        assert!(arch.read_mstatus() & MSTATUS_MIE == 0);
        arch.enable_interrupts();
        assert!(arch.read_mstatus() & MSTATUS_MIE != 0);
    }

    #[test]
    fn test_csr_registers() {
        let mut arch = make_arch();
        arch.write_satp(0x8000_0000_0000_1000);
        assert_eq!(arch.read_satp(), 0x8000_0000_0000_1000);
        arch.write_mtvec(0x8000_0000);
        assert_eq!(arch.read_mtvec(), 0x8000_0000);
    }

    #[test]
    fn test_gpr_read_write() {
        let mut arch = make_arch();
        arch.write_gpr(1, 42).unwrap();
        assert_eq!(arch.read_gpr(1).unwrap(), 42);
        arch.write_gpr(31, 0xFF).unwrap();
        assert_eq!(arch.read_gpr(31).unwrap(), 0xFF);
    }

    #[test]
    fn test_gpr_x0_hardwired_zero() {
        let mut arch = make_arch();
        arch.write_gpr(0, 42).unwrap();
        assert_eq!(arch.read_gpr(0).unwrap(), 0);
    }

    #[test]
    fn test_gpr_invalid_register() {
        let mut arch = make_arch();
        assert_eq!(arch.read_gpr(32), Err(ArchError::InvalidRegister));
        assert_eq!(arch.write_gpr(32, 0), Err(ArchError::InvalidRegister));
    }

    #[test]
    fn test_mmu_config() {
        let arch = make_arch();
        let cfg = arch.mmu_config();
        assert_eq!(cfg.page_table_levels, SV48_LEVELS);
        assert_eq!(cfg.page_size, SV48_PAGE_SIZE);
        assert!(cfg.supports_huge_pages);
        assert!(cfg.supports_ternary_tagging);
        assert_eq!(cfg.address_space_bits, SV48_ADDRESS_BITS);
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
            vector_number: 1024,
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
        assert_eq!(arch.max_vectors(), 1024);
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
        assert_eq!(cfg.frequency_hz, MTIME_DEFAULT_FREQ);
        assert!(cfg.supports_femtosecond);
        assert_eq!(cfg.resolution_bits, 64);
    }

    #[test]
    fn test_timer_set_and_ticks() {
        let mut arch = make_arch();
        assert_eq!(arch.current_ticks(), 0);
        arch.set_timer(500).unwrap();
        assert_eq!(arch.read_mtimecmp(), 500);
    }

    #[test]
    fn test_boot_info() {
        let arch = make_arch();
        let info = arch.boot_info();
        assert_eq!(info.kernel_start, 0x8020_0000);
        assert_eq!(info.kernel_end, 0x8040_0000);
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
        assert_eq!(arch.arch_name(), "riscv64");
    }

    #[test]
    fn test_sv48_constants() {
        assert_eq!(SV48_PAGE_SIZE, 4096);
        assert_eq!(SV48_LEVELS, 4);
        assert_eq!(SV48_ADDRESS_BITS, 48);
    }

    #[test]
    fn test_halt_and_memory_barrier() {
        let arch = make_arch();
        arch.halt();
        arch.memory_barrier();
    }

    #[test]
    fn test_csr_constants() {
        assert_eq!(CSR_MSTATUS, 0x300);
        assert_eq!(CSR_SATP, 0x180);
        assert_eq!(CSR_SSTATUS, 0x100);
    }
}
