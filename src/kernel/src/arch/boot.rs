use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use super::{ArchId, MemoryRegion, MemoryRegionType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStage {
    PowerOn,
    FirmwareHandoff,
    EarlyInit,
    MemoryDetection,
    PageTableSetup,
    InterruptSetup,
    TimerSetup,
    TernaryCoprocessorInit,
    DriverInit,
    SchedulerInit,
    LateInit,
    Running,
    Halted,
    Panic,
}

impl BootStage {
    fn ordinal(&self) -> Option<usize> {
        match self {
            BootStage::PowerOn => Some(0),
            BootStage::FirmwareHandoff => Some(1),
            BootStage::EarlyInit => Some(2),
            BootStage::MemoryDetection => Some(3),
            BootStage::PageTableSetup => Some(4),
            BootStage::InterruptSetup => Some(5),
            BootStage::TimerSetup => Some(6),
            BootStage::TernaryCoprocessorInit => Some(7),
            BootStage::DriverInit => Some(8),
            BootStage::SchedulerInit => Some(9),
            BootStage::LateInit => Some(10),
            BootStage::Running => Some(11),
            BootStage::Halted => None,
            BootStage::Panic => None,
        }
    }

    fn from_ordinal(ord: usize) -> Option<BootStage> {
        match ord {
            0 => Some(BootStage::PowerOn),
            1 => Some(BootStage::FirmwareHandoff),
            2 => Some(BootStage::EarlyInit),
            3 => Some(BootStage::MemoryDetection),
            4 => Some(BootStage::PageTableSetup),
            5 => Some(BootStage::InterruptSetup),
            6 => Some(BootStage::TimerSetup),
            7 => Some(BootStage::TernaryCoprocessorInit),
            8 => Some(BootStage::DriverInit),
            9 => Some(BootStage::SchedulerInit),
            10 => Some(BootStage::LateInit),
            11 => Some(BootStage::Running),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootError {
    StageNotReady,
    InitializationFailed(String),
    MemoryDetectionFailed,
    PageTableSetupFailed,
    InterruptSetupFailed,
    HardwareNotPresent,
    InvalidBootSequence,
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootError::StageNotReady => write!(f, "Stage not ready"),
            BootError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            BootError::MemoryDetectionFailed => write!(f, "Memory detection failed"),
            BootError::PageTableSetupFailed => write!(f, "Page table setup failed"),
            BootError::InterruptSetupFailed => write!(f, "Interrupt setup failed"),
            BootError::HardwareNotPresent => write!(f, "Hardware not present"),
            BootError::InvalidBootSequence => write!(f, "Invalid boot sequence"),
        }
    }
}

pub type BootResult<T> = Result<T, BootError>;

pub struct BootSequence {
    current_stage: BootStage,
    arch_id: ArchId,
    stages_completed: Vec<BootStage>,
    boot_time_ticks: u64,
    memory_detected: u64,
    cpu_count: u32,
    has_ternary_hw: bool,
    kernel_entry: u64,
    stack_top: u64,
}

impl BootSequence {
    pub fn new(arch_id: ArchId) -> Self {
        Self {
            current_stage: BootStage::PowerOn,
            arch_id,
            stages_completed: Vec::new(),
            boot_time_ticks: 0,
            memory_detected: 0,
            cpu_count: 1,
            has_ternary_hw: false,
            kernel_entry: 0,
            stack_top: 0,
        }
    }

    pub fn current_stage(&self) -> &BootStage {
        &self.current_stage
    }

    pub fn advance(&mut self) -> BootResult<BootStage> {
        match self.current_stage {
            BootStage::Halted | BootStage::Panic => {
                return Err(BootError::InvalidBootSequence);
            }
            _ => {}
        }

        let current_ord = self.current_stage.ordinal()
            .ok_or(BootError::InvalidBootSequence)?;

        if current_ord >= 11 {
            return Err(BootError::InvalidBootSequence);
        }

        let next_stage = BootStage::from_ordinal(current_ord + 1)
            .ok_or(BootError::InvalidBootSequence)?;

        self.stages_completed.push(self.current_stage);
        self.current_stage = next_stage;
        self.boot_time_ticks += 1;

        Ok(next_stage)
    }

    pub fn advance_to(&mut self, stage: BootStage) -> BootResult<()> {
        match stage {
            BootStage::Halted => {
                self.stages_completed.push(self.current_stage);
                self.current_stage = BootStage::Halted;
                return Ok(());
            }
            BootStage::Panic => {
                self.stages_completed.push(self.current_stage);
                self.current_stage = BootStage::Panic;
                return Ok(());
            }
            _ => {}
        }

        let current_ord = self.current_stage.ordinal()
            .ok_or(BootError::InvalidBootSequence)?;
        let target_ord = stage.ordinal()
            .ok_or(BootError::InvalidBootSequence)?;

        if target_ord != current_ord + 1 {
            return Err(BootError::InvalidBootSequence);
        }

        self.stages_completed.push(self.current_stage);
        self.current_stage = stage;
        self.boot_time_ticks += 1;

        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.current_stage == BootStage::Running
    }

    pub fn stages_completed(&self) -> &[BootStage] {
        &self.stages_completed
    }

    pub fn set_memory_detected(&mut self, bytes: u64) {
        self.memory_detected = bytes;
    }

    pub fn set_cpu_count(&mut self, count: u32) {
        self.cpu_count = count;
    }

    pub fn set_ternary_hardware(&mut self, present: bool) {
        self.has_ternary_hw = present;
    }

    pub fn set_kernel_entry(&mut self, addr: u64) {
        self.kernel_entry = addr;
    }

    pub fn set_stack_top(&mut self, addr: u64) {
        self.stack_top = addr;
    }

    pub fn panic_halt(&mut self, reason: &str) -> BootError {
        self.stages_completed.push(self.current_stage);
        self.current_stage = BootStage::Panic;
        BootError::InitializationFailed(String::from(reason))
    }

    pub fn elapsed_stages(&self) -> usize {
        self.stages_completed.len()
    }
}

pub struct BootParams {
    pub arch_id: ArchId,
    pub kernel_physical_base: u64,
    pub kernel_virtual_base: u64,
    pub kernel_size: u64,
    pub initrd_base: u64,
    pub initrd_size: u64,
    pub command_line: String,
    pub framebuffer_base: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_pitch: u32,
    pub memory_map: Vec<MemoryRegion>,
}

pub fn x86_64_boot_config() -> BootParams {
    BootParams {
        arch_id: ArchId::X86_64,
        kernel_physical_base: 0x0010_0000,
        kernel_virtual_base: 0xFFFF_8000_0000_0000,
        kernel_size: 0x0020_0000,
        initrd_base: 0x0100_0000,
        initrd_size: 0x0080_0000,
        command_line: String::from("console=ttyS0 root=/dev/sda1"),
        framebuffer_base: 0xFD00_0000,
        framebuffer_width: 1920,
        framebuffer_height: 1080,
        framebuffer_pitch: 7680,
        memory_map: alloc::vec![
            MemoryRegion { base: 0x0000_0000, size: 0x0009_FC00, region_type: MemoryRegionType::Usable },
            MemoryRegion { base: 0x0009_FC00, size: 0x0000_0400, region_type: MemoryRegionType::Reserved },
            MemoryRegion { base: 0x000F_0000, size: 0x0001_0000, region_type: MemoryRegionType::Reserved },
            MemoryRegion { base: 0x0010_0000, size: 0x3FEF_0000, region_type: MemoryRegionType::Usable },
        ],
    }
}

pub fn aarch64_boot_config() -> BootParams {
    BootParams {
        arch_id: ArchId::Aarch64,
        kernel_physical_base: 0x0004_0000,
        kernel_virtual_base: 0xFFFF_0000_0000_0000,
        kernel_size: 0x0020_0000,
        initrd_base: 0x0800_0000,
        initrd_size: 0x0080_0000,
        command_line: String::from("console=ttyAMA0 root=/dev/mmcblk0p2"),
        framebuffer_base: 0x3E00_0000,
        framebuffer_width: 1920,
        framebuffer_height: 1080,
        framebuffer_pitch: 7680,
        memory_map: alloc::vec![
            MemoryRegion { base: 0x0004_0000, size: 0x3FFC_0000, region_type: MemoryRegionType::Usable },
            MemoryRegion { base: 0x3E00_0000, size: 0x0200_0000, region_type: MemoryRegionType::Reserved },
        ],
    }
}

pub fn riscv64_boot_config() -> BootParams {
    BootParams {
        arch_id: ArchId::RiscV64,
        kernel_physical_base: 0x8000_0000,
        kernel_virtual_base: 0xFFFF_FFE0_0000_0000,
        kernel_size: 0x0020_0000,
        initrd_base: 0x8200_0000,
        initrd_size: 0x0080_0000,
        command_line: String::from("console=ttyS0 earlycon"),
        framebuffer_base: 0x3000_0000,
        framebuffer_width: 1280,
        framebuffer_height: 720,
        framebuffer_pitch: 5120,
        memory_map: alloc::vec![
            MemoryRegion { base: 0x8000_0000, size: 0x8000_0000, region_type: MemoryRegionType::Usable },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_sequence_creation() {
        let seq = BootSequence::new(ArchId::X86_64);
        assert_eq!(*seq.current_stage(), BootStage::PowerOn);
        assert_eq!(seq.elapsed_stages(), 0);
        assert!(!seq.is_complete());
    }

    #[test]
    fn test_boot_advance_full_sequence() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        let stages = [
            BootStage::FirmwareHandoff,
            BootStage::EarlyInit,
            BootStage::MemoryDetection,
            BootStage::PageTableSetup,
            BootStage::InterruptSetup,
            BootStage::TimerSetup,
            BootStage::TernaryCoprocessorInit,
            BootStage::DriverInit,
            BootStage::SchedulerInit,
            BootStage::LateInit,
            BootStage::Running,
        ];
        for expected in &stages {
            let next = seq.advance().unwrap();
            assert_eq!(next, *expected);
        }
        assert!(seq.is_complete());
    }

    #[test]
    fn test_boot_advance_to_specific_stage() {
        let mut seq = BootSequence::new(ArchId::Aarch64);
        seq.advance_to(BootStage::FirmwareHandoff).unwrap();
        assert_eq!(*seq.current_stage(), BootStage::FirmwareHandoff);
        seq.advance_to(BootStage::EarlyInit).unwrap();
        assert_eq!(*seq.current_stage(), BootStage::EarlyInit);
    }

    #[test]
    fn test_boot_invalid_sequence() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.advance().unwrap();
        let result = seq.advance_to(BootStage::PowerOn);
        assert_eq!(result, Err(BootError::InvalidBootSequence));
    }

    #[test]
    fn test_boot_panic_halt() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.advance().unwrap();
        let err = seq.panic_halt("test panic");
        assert_eq!(*seq.current_stage(), BootStage::Panic);
        assert_eq!(err, BootError::InitializationFailed(String::from("test panic")));
    }

    #[test]
    fn test_boot_is_complete() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        assert!(!seq.is_complete());
        for _ in 0..11 {
            seq.advance().unwrap();
        }
        assert!(seq.is_complete());
    }

    #[test]
    fn test_boot_memory_detection() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.set_memory_detected(1024 * 1024 * 1024);
        assert_eq!(seq.memory_detected, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_boot_cpu_count() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.set_cpu_count(4);
        assert_eq!(seq.cpu_count, 4);
    }

    #[test]
    fn test_boot_ternary_hardware() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        assert!(!seq.has_ternary_hw);
        seq.set_ternary_hardware(true);
        assert!(seq.has_ternary_hw);
    }

    #[test]
    fn test_boot_params_x86_64() {
        let params = x86_64_boot_config();
        assert_eq!(params.arch_id, ArchId::X86_64);
        assert_eq!(params.kernel_physical_base, 0x0010_0000);
        assert_eq!(params.kernel_virtual_base, 0xFFFF_8000_0000_0000);
        assert_eq!(params.framebuffer_width, 1920);
        assert_eq!(params.framebuffer_height, 1080);
    }

    #[test]
    fn test_boot_params_aarch64() {
        let params = aarch64_boot_config();
        assert_eq!(params.arch_id, ArchId::Aarch64);
        assert_eq!(params.kernel_virtual_base, 0xFFFF_0000_0000_0000);
        assert_eq!(params.framebuffer_width, 1920);
    }

    #[test]
    fn test_boot_params_riscv64() {
        let params = riscv64_boot_config();
        assert_eq!(params.arch_id, ArchId::RiscV64);
        assert_eq!(params.kernel_virtual_base, 0xFFFF_FFE0_0000_0000);
        assert_eq!(params.kernel_physical_base, 0x8000_0000);
    }

    #[test]
    fn test_boot_elapsed_stages() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        assert_eq!(seq.elapsed_stages(), 0);
        seq.advance().unwrap();
        assert_eq!(seq.elapsed_stages(), 1);
        seq.advance().unwrap();
        assert_eq!(seq.elapsed_stages(), 2);
    }

    #[test]
    fn test_boot_kernel_entry() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.set_kernel_entry(0xFFFF_8000_0010_0000);
        assert_eq!(seq.kernel_entry, 0xFFFF_8000_0010_0000);
    }

    #[test]
    fn test_boot_double_advance_same_stage() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.advance().unwrap();
        assert_eq!(*seq.current_stage(), BootStage::FirmwareHandoff);
        let result = seq.advance_to(BootStage::FirmwareHandoff);
        assert_eq!(result, Err(BootError::InvalidBootSequence));
    }

    #[test]
    fn test_boot_cannot_advance_past_running() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        for _ in 0..11 {
            seq.advance().unwrap();
        }
        assert!(seq.is_complete());
        let result = seq.advance();
        assert_eq!(result, Err(BootError::InvalidBootSequence));
    }

    #[test]
    fn test_boot_stage_recording() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.advance().unwrap();
        seq.advance().unwrap();
        seq.advance().unwrap();
        let completed = seq.stages_completed();
        assert_eq!(completed.len(), 3);
        assert_eq!(completed[0], BootStage::PowerOn);
        assert_eq!(completed[1], BootStage::FirmwareHandoff);
        assert_eq!(completed[2], BootStage::EarlyInit);
    }

    #[test]
    fn test_boot_from_halted() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.advance().unwrap();
        seq.advance_to(BootStage::Halted).unwrap();
        assert_eq!(*seq.current_stage(), BootStage::Halted);
        let result = seq.advance();
        assert_eq!(result, Err(BootError::InvalidBootSequence));
    }

    #[test]
    fn test_boot_params_memory_map() {
        let params = x86_64_boot_config();
        assert_eq!(params.memory_map.len(), 4);
        assert_eq!(params.memory_map[0].region_type, MemoryRegionType::Usable);
        assert_eq!(params.memory_map[1].region_type, MemoryRegionType::Reserved);
        assert!(params.memory_map[3].size > 0);
    }

    #[test]
    fn test_boot_stack_setup() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        seq.set_stack_top(0xFFFF_8000_0020_0000);
        assert_eq!(seq.stack_top, 0xFFFF_8000_0020_0000);
    }
}
