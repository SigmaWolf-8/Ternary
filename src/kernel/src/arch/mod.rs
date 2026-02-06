pub mod x86_64;
pub mod aarch64;
pub mod riscv64;
pub mod boot;

use alloc::collections::BTreeMap;
use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchError {
    InvalidRegister,
    UnsupportedFeature,
    InitializationFailed,
    InvalidAddress,
    AlignmentError,
    PrivilegeViolation,
}

impl fmt::Display for ArchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchError::InvalidRegister => write!(f, "Invalid register"),
            ArchError::UnsupportedFeature => write!(f, "Unsupported feature"),
            ArchError::InitializationFailed => write!(f, "Initialization failed"),
            ArchError::InvalidAddress => write!(f, "Invalid address"),
            ArchError::AlignmentError => write!(f, "Alignment error"),
            ArchError::PrivilegeViolation => write!(f, "Privilege violation"),
        }
    }
}

pub type ArchResult<T> = Result<T, ArchError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchId {
    X86_64,
    Aarch64,
    RiscV64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    Kernel = 0,
    Supervisor = 1,
    User = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuFeatures {
    pub has_ternary_coprocessor: bool,
    pub has_phase_encryption_unit: bool,
    pub has_femtosecond_timer: bool,
    pub has_vector_extensions: bool,
    pub has_hardware_crypto: bool,
    pub cache_line_size: u32,
    pub max_physical_address_bits: u8,
    pub max_virtual_address_bits: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptVector {
    pub vector_number: u16,
    pub handler_address: u64,
    pub privilege_level: PrivilegeLevel,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MmuConfig {
    pub page_table_levels: u8,
    pub page_size: u32,
    pub supports_huge_pages: bool,
    pub supports_ternary_tagging: bool,
    pub address_space_bits: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerConfig {
    pub frequency_hz: u64,
    pub supports_femtosecond: bool,
    pub resolution_bits: u8,
}

#[derive(Debug, Clone)]
pub struct BootInfo {
    pub kernel_start: u64,
    pub kernel_end: u64,
    pub memory_map: BTreeMap<u64, MemoryRegion>,
    pub command_line: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    Defective,
    TernaryCoprocessor,
}

pub trait ArchOps {
    fn arch_id(&self) -> ArchId;
    fn cpu_features(&self) -> CpuFeatures;
    fn privilege_level(&self) -> PrivilegeLevel;
    fn set_privilege_level(&mut self, level: PrivilegeLevel) -> ArchResult<()>;
    fn enable_interrupts(&mut self);
    fn disable_interrupts(&mut self);
    fn interrupts_enabled(&self) -> bool;
    fn halt(&self);
    fn memory_barrier(&self);
}

pub trait MmuOps {
    fn mmu_config(&self) -> MmuConfig;
    fn map_page(&mut self, virt: u64, phys: u64, flags: u64) -> ArchResult<()>;
    fn unmap_page(&mut self, virt: u64) -> ArchResult<()>;
    fn translate(&self, virt: u64) -> ArchResult<u64>;
    fn flush_tlb(&mut self, virt: u64);
    fn flush_tlb_all(&mut self);
}

pub trait InterruptOps {
    fn configure_vector(&mut self, vector: InterruptVector) -> ArchResult<()>;
    fn enable_vector(&mut self, vector_number: u16) -> ArchResult<()>;
    fn disable_vector(&mut self, vector_number: u16) -> ArchResult<()>;
    fn acknowledge(&mut self, vector_number: u16);
    fn is_pending(&self, vector_number: u16) -> bool;
    fn max_vectors(&self) -> u16;
}

pub trait TimerOps {
    fn timer_config(&self) -> TimerConfig;
    fn current_ticks(&self) -> u64;
    fn set_timer(&mut self, ticks: u64) -> ArchResult<()>;
    fn timer_elapsed(&self) -> u64;
}

pub trait BootOps {
    fn boot_info(&self) -> &BootInfo;
    fn init_early(&mut self) -> ArchResult<()>;
    fn init_late(&mut self) -> ArchResult<()>;
    fn arch_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_error_display() {
        assert_eq!(format!("{}", ArchError::InvalidRegister), "Invalid register");
        assert_eq!(format!("{}", ArchError::UnsupportedFeature), "Unsupported feature");
        assert_eq!(format!("{}", ArchError::InitializationFailed), "Initialization failed");
        assert_eq!(format!("{}", ArchError::InvalidAddress), "Invalid address");
        assert_eq!(format!("{}", ArchError::AlignmentError), "Alignment error");
        assert_eq!(format!("{}", ArchError::PrivilegeViolation), "Privilege violation");
    }

    #[test]
    fn test_arch_id_variants() {
        assert_ne!(ArchId::X86_64, ArchId::Aarch64);
        assert_ne!(ArchId::Aarch64, ArchId::RiscV64);
        assert_ne!(ArchId::X86_64, ArchId::RiscV64);
    }

    #[test]
    fn test_privilege_level_values() {
        assert_eq!(PrivilegeLevel::Kernel as u8, 0);
        assert_eq!(PrivilegeLevel::Supervisor as u8, 1);
        assert_eq!(PrivilegeLevel::User as u8, 2);
    }

    #[test]
    fn test_cpu_features_default() {
        let features = CpuFeatures {
            has_ternary_coprocessor: false,
            has_phase_encryption_unit: false,
            has_femtosecond_timer: false,
            has_vector_extensions: false,
            has_hardware_crypto: false,
            cache_line_size: 64,
            max_physical_address_bits: 48,
            max_virtual_address_bits: 48,
        };
        assert!(!features.has_ternary_coprocessor);
        assert_eq!(features.cache_line_size, 64);
    }

    #[test]
    fn test_interrupt_vector_creation() {
        let vec = InterruptVector {
            vector_number: 32,
            handler_address: 0xFFFF_0000,
            privilege_level: PrivilegeLevel::Kernel,
            is_enabled: true,
        };
        assert_eq!(vec.vector_number, 32);
        assert!(vec.is_enabled);
    }

    #[test]
    fn test_mmu_config() {
        let cfg = MmuConfig {
            page_table_levels: 4,
            page_size: 4096,
            supports_huge_pages: true,
            supports_ternary_tagging: true,
            address_space_bits: 48,
        };
        assert_eq!(cfg.page_table_levels, 4);
        assert_eq!(cfg.page_size, 4096);
    }

    #[test]
    fn test_timer_config() {
        let cfg = TimerConfig {
            frequency_hz: 1_000_000_000,
            supports_femtosecond: true,
            resolution_bits: 64,
        };
        assert_eq!(cfg.frequency_hz, 1_000_000_000);
        assert!(cfg.supports_femtosecond);
    }

    #[test]
    fn test_boot_info_creation() {
        let mut memory_map = BTreeMap::new();
        memory_map.insert(0x0, MemoryRegion {
            base: 0x0,
            size: 0x100000,
            region_type: MemoryRegionType::Usable,
        });
        let info = BootInfo {
            kernel_start: 0x100000,
            kernel_end: 0x200000,
            memory_map,
            command_line: String::from("console=ttyS0"),
        };
        assert_eq!(info.kernel_start, 0x100000);
        assert_eq!(info.memory_map.len(), 1);
    }

    #[test]
    fn test_memory_region_type_variants() {
        let types = [
            MemoryRegionType::Usable,
            MemoryRegionType::Reserved,
            MemoryRegionType::AcpiReclaimable,
            MemoryRegionType::AcpiNvs,
            MemoryRegionType::Defective,
            MemoryRegionType::TernaryCoprocessor,
        ];
        for i in 0..types.len() {
            for j in (i + 1)..types.len() {
                assert_ne!(types[i], types[j]);
            }
        }
    }

    #[test]
    fn test_arch_result_ok() {
        let result: ArchResult<u64> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_arch_result_err() {
        let result: ArchResult<u64> = Err(ArchError::InvalidAddress);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ArchError::InvalidAddress);
    }
}
