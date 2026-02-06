use alloc::vec::Vec;
use super::{DriverError, DriverResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpuType {
    Fpga,
    Asic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpuState {
    Uninitialized,
    Initializing,
    Ready,
    Computing,
    Error,
    Suspended,
}

#[derive(Debug, Clone)]
pub struct TpuCapabilities {
    pub tpu_type: TpuType,
    pub max_trits_per_cycle: u32,
    pub max_concurrent_ops: u16,
    pub supports_phase_encryption: bool,
    pub supports_bulk_gf3: bool,
    pub firmware_version: u32,
    pub memory_size_bytes: u64,
    pub dma_channels: u8,
}

#[derive(Debug, Clone)]
pub enum TpuCommand {
    TernaryAdd { a: [i8; 27], b: [i8; 27] },
    TernaryMultiply { a: [i8; 27], b: [i8; 27] },
    TernaryRotate { data: [i8; 27], positions: i32 },
    BulkConvert { data: Vec<i8>, from_repr: u8, to_repr: u8 },
    PhaseEncrypt { data: Vec<i8>, key: [i8; 27] },
    PhaseDecrypt { data: Vec<i8>, key: [i8; 27] },
    Hash { data: Vec<i8> },
    Nop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TpuResultStatus {
    Success,
    Error,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct TpuResult {
    pub command_id: u64,
    pub result_data: Vec<i8>,
    pub cycles_taken: u64,
    pub status: TpuResultStatus,
}

#[derive(Debug, Clone)]
pub struct CommandQueueEntry {
    pub id: u64,
    pub command: TpuCommand,
    pub submitted_tick: u64,
    pub priority: u8,
}

const FIRMWARE_MAGIC: [u8; 4] = [0x54, 0x50, 0x55, 0x46];

fn gf3_add(a: i8, b: i8) -> i8 {
    let a_mapped = ((a % 3) + 3) % 3;
    let b_mapped = ((b % 3) + 3) % 3;
    let sum = (a_mapped + b_mapped) % 3;
    match sum {
        0 => 0,
        1 => 1,
        2 => -1,
        _ => 0,
    }
}

fn gf3_multiply(a: i8, b: i8) -> i8 {
    let product = a as i16 * b as i16;
    let rem = ((product % 3) + 3) % 3;
    match rem as i8 {
        0 => 0,
        1 => 1,
        2 => -1,
        _ => 0,
    }
}

fn execute_command(cmd: &TpuCommand) -> (Vec<i8>, u64) {
    match cmd {
        TpuCommand::TernaryAdd { a, b } => {
            let mut result = [0i8; 27];
            for i in 0..27 {
                result[i] = gf3_add(a[i], b[i]);
            }
            (result.to_vec(), 3)
        }
        TpuCommand::TernaryMultiply { a, b } => {
            let mut result = [0i8; 27];
            for i in 0..27 {
                result[i] = gf3_multiply(a[i], b[i]);
            }
            (result.to_vec(), 5)
        }
        TpuCommand::TernaryRotate { data, positions } => {
            let len = 27i32;
            let shift = ((positions % len) + len) % len;
            let mut result = [0i8; 27];
            for i in 0..27 {
                let src = ((i as i32 - shift + len) % len) as usize;
                result[i] = data[src];
            }
            (result.to_vec(), 2)
        }
        TpuCommand::BulkConvert { data, from_repr, to_repr } => {
            let converted: Vec<i8> = data.iter().map(|&v| {
                let a_val = match from_repr {
                    0 => v,
                    1 => v - 1,
                    2 => v - 2,
                    _ => v,
                };
                match to_repr {
                    0 => a_val,
                    1 => a_val + 1,
                    2 => a_val + 2,
                    _ => a_val,
                }
            }).collect();
            let cycles = data.len() as u64;
            (converted, cycles)
        }
        TpuCommand::PhaseEncrypt { data, key } => {
            let encrypted: Vec<i8> = data.iter().enumerate().map(|(i, &v)| {
                gf3_add(v, key[i % 27])
            }).collect();
            (encrypted, data.len() as u64 + 10)
        }
        TpuCommand::PhaseDecrypt { data, key } => {
            let mut neg_key = [0i8; 27];
            for i in 0..27 {
                neg_key[i] = -key[i];
            }
            let decrypted: Vec<i8> = data.iter().enumerate().map(|(i, &v)| {
                gf3_add(v, neg_key[i % 27])
            }).collect();
            (decrypted, data.len() as u64 + 10)
        }
        TpuCommand::Hash { data } => {
            let mut acc = 0i8;
            for &v in data.iter() {
                acc = gf3_add(acc, v);
            }
            (alloc::vec![acc], data.len() as u64 + 1)
        }
        TpuCommand::Nop => {
            (Vec::new(), 1)
        }
    }
}

pub struct FpgaTpuDriver {
    state: TpuState,
    capabilities: TpuCapabilities,
    command_queue: Vec<CommandQueueEntry>,
    completed: Vec<TpuResult>,
    next_command_id: u64,
    firmware_loaded: bool,
    firmware_data: Vec<u8>,
    total_ops: u64,
    error_count: u32,
    max_queue_depth: usize,
}

impl FpgaTpuDriver {
    pub fn new() -> Self {
        Self {
            state: TpuState::Uninitialized,
            capabilities: TpuCapabilities {
                tpu_type: TpuType::Fpga,
                max_trits_per_cycle: 729,
                max_concurrent_ops: 16,
                supports_phase_encryption: true,
                supports_bulk_gf3: true,
                firmware_version: 0,
                memory_size_bytes: 16 * 1024 * 1024,
                dma_channels: 4,
            },
            command_queue: Vec::new(),
            completed: Vec::new(),
            next_command_id: 1,
            firmware_loaded: false,
            firmware_data: Vec::new(),
            total_ops: 0,
            error_count: 0,
            max_queue_depth: 256,
        }
    }

    pub fn load_firmware(&mut self, firmware: &[u8]) -> DriverResult<()> {
        if firmware.len() < 4 {
            return Err(DriverError::FirmwareLoadFailed);
        }
        if firmware[0..4] != FIRMWARE_MAGIC {
            return Err(DriverError::FirmwareLoadFailed);
        }
        self.firmware_data = firmware.to_vec();
        self.firmware_loaded = true;
        self.capabilities.firmware_version = 1;
        Ok(())
    }

    pub fn initialize(&mut self) -> DriverResult<()> {
        if !self.firmware_loaded {
            return Err(DriverError::FirmwareLoadFailed);
        }
        self.state = TpuState::Initializing;
        self.state = TpuState::Ready;
        Ok(())
    }

    pub fn submit_command(&mut self, cmd: TpuCommand) -> DriverResult<u64> {
        match self.state {
            TpuState::Ready | TpuState::Computing => {}
            _ => return Err(DriverError::HardwareNotPresent),
        }
        if self.command_queue.len() >= self.max_queue_depth {
            return Err(DriverError::CommandQueueFull);
        }
        let id = self.next_command_id;
        self.next_command_id += 1;
        self.command_queue.push(CommandQueueEntry {
            id,
            command: cmd,
            submitted_tick: 0,
            priority: 0,
        });
        self.state = TpuState::Computing;
        Ok(id)
    }

    pub fn process_next(&mut self) -> DriverResult<Option<TpuResult>> {
        match self.state {
            TpuState::Ready | TpuState::Computing => {}
            _ => return Err(DriverError::HardwareNotPresent),
        }
        if self.command_queue.is_empty() {
            self.state = TpuState::Ready;
            return Ok(None);
        }
        let entry = self.command_queue.remove(0);
        let (result_data, cycles_taken) = execute_command(&entry.command);
        let result = TpuResult {
            command_id: entry.id,
            result_data,
            cycles_taken,
            status: TpuResultStatus::Success,
        };
        self.completed.push(result.clone());
        self.total_ops += 1;
        if self.command_queue.is_empty() {
            self.state = TpuState::Ready;
        }
        Ok(Some(result))
    }

    pub fn get_result(&self, command_id: u64) -> Option<&TpuResult> {
        self.completed.iter().find(|r| r.command_id == command_id)
    }

    pub fn state(&self) -> &TpuState {
        &self.state
    }

    pub fn capabilities(&self) -> &TpuCapabilities {
        &self.capabilities
    }

    pub fn queue_depth(&self) -> usize {
        self.command_queue.len()
    }

    pub fn total_operations(&self) -> u64 {
        self.total_ops
    }

    pub fn reset(&mut self) -> DriverResult<()> {
        self.command_queue.clear();
        self.completed.clear();
        self.state = TpuState::Uninitialized;
        self.firmware_loaded = false;
        self.firmware_data.clear();
        self.next_command_id = 1;
        self.total_ops = 0;
        self.error_count = 0;
        Ok(())
    }

    pub fn suspend(&mut self) -> DriverResult<()> {
        match self.state {
            TpuState::Ready | TpuState::Computing => {
                self.state = TpuState::Suspended;
                Ok(())
            }
            _ => Err(DriverError::InvalidConfiguration),
        }
    }

    pub fn resume(&mut self) -> DriverResult<()> {
        match self.state {
            TpuState::Suspended => {
                self.state = if self.command_queue.is_empty() {
                    TpuState::Ready
                } else {
                    TpuState::Computing
                };
                Ok(())
            }
            _ => Err(DriverError::InvalidConfiguration),
        }
    }

    pub fn reconfigure(&mut self, max_queue: usize) -> DriverResult<()> {
        if max_queue == 0 {
            return Err(DriverError::InvalidConfiguration);
        }
        self.max_queue_depth = max_queue;
        Ok(())
    }
}

pub struct AsicTpuDriver {
    state: TpuState,
    capabilities: TpuCapabilities,
    command_queue: Vec<CommandQueueEntry>,
    completed: Vec<TpuResult>,
    next_command_id: u64,
    total_ops: u64,
    error_count: u32,
    max_queue_depth: usize,
    rom_version: u32,
}

impl AsicTpuDriver {
    pub fn new() -> Self {
        Self {
            state: TpuState::Uninitialized,
            capabilities: TpuCapabilities {
                tpu_type: TpuType::Asic,
                max_trits_per_cycle: 2187,
                max_concurrent_ops: 64,
                supports_phase_encryption: true,
                supports_bulk_gf3: true,
                firmware_version: 0,
                memory_size_bytes: 64 * 1024 * 1024,
                dma_channels: 8,
            },
            command_queue: Vec::new(),
            completed: Vec::new(),
            next_command_id: 1,
            total_ops: 0,
            error_count: 0,
            max_queue_depth: 256,
            rom_version: 0x0001_0000,
        }
    }

    pub fn initialize(&mut self) -> DriverResult<()> {
        self.state = TpuState::Initializing;
        self.state = TpuState::Ready;
        Ok(())
    }

    pub fn submit_command(&mut self, cmd: TpuCommand) -> DriverResult<u64> {
        match self.state {
            TpuState::Ready | TpuState::Computing => {}
            _ => return Err(DriverError::HardwareNotPresent),
        }
        if self.command_queue.len() >= self.max_queue_depth {
            return Err(DriverError::CommandQueueFull);
        }
        let id = self.next_command_id;
        self.next_command_id += 1;
        self.command_queue.push(CommandQueueEntry {
            id,
            command: cmd,
            submitted_tick: 0,
            priority: 0,
        });
        self.state = TpuState::Computing;
        Ok(id)
    }

    pub fn process_next(&mut self) -> DriverResult<Option<TpuResult>> {
        match self.state {
            TpuState::Ready | TpuState::Computing => {}
            _ => return Err(DriverError::HardwareNotPresent),
        }
        if self.command_queue.is_empty() {
            self.state = TpuState::Ready;
            return Ok(None);
        }
        let entry = self.command_queue.remove(0);
        let (result_data, cycles_taken) = execute_command(&entry.command);
        let result = TpuResult {
            command_id: entry.id,
            result_data,
            cycles_taken,
            status: TpuResultStatus::Success,
        };
        self.completed.push(result.clone());
        self.total_ops += 1;
        if self.command_queue.is_empty() {
            self.state = TpuState::Ready;
        }
        Ok(Some(result))
    }

    pub fn get_result(&self, command_id: u64) -> Option<&TpuResult> {
        self.completed.iter().find(|r| r.command_id == command_id)
    }

    pub fn state(&self) -> &TpuState {
        &self.state
    }

    pub fn capabilities(&self) -> &TpuCapabilities {
        &self.capabilities
    }

    pub fn queue_depth(&self) -> usize {
        self.command_queue.len()
    }

    pub fn total_operations(&self) -> u64 {
        self.total_ops
    }

    pub fn reset(&mut self) -> DriverResult<()> {
        self.command_queue.clear();
        self.completed.clear();
        self.state = TpuState::Uninitialized;
        self.next_command_id = 1;
        self.total_ops = 0;
        self.error_count = 0;
        Ok(())
    }

    pub fn suspend(&mut self) -> DriverResult<()> {
        match self.state {
            TpuState::Ready | TpuState::Computing => {
                self.state = TpuState::Suspended;
                Ok(())
            }
            _ => Err(DriverError::InvalidConfiguration),
        }
    }

    pub fn resume(&mut self) -> DriverResult<()> {
        match self.state {
            TpuState::Suspended => {
                self.state = if self.command_queue.is_empty() {
                    TpuState::Ready
                } else {
                    TpuState::Computing
                };
                Ok(())
            }
            _ => Err(DriverError::InvalidConfiguration),
        }
    }

    pub fn reconfigure(&mut self, max_queue: usize) -> DriverResult<()> {
        if max_queue == 0 {
            return Err(DriverError::InvalidConfiguration);
        }
        self.max_queue_depth = max_queue;
        Ok(())
    }

    pub fn rom_version(&self) -> u32 {
        self.rom_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_firmware() -> Vec<u8> {
        let mut fw = vec![0x54, 0x50, 0x55, 0x46];
        fw.extend_from_slice(&[0u8; 64]);
        fw
    }

    fn zero_trits() -> [i8; 27] {
        [0i8; 27]
    }

    fn ones_trits() -> [i8; 27] {
        [1i8; 27]
    }

    fn neg_ones_trits() -> [i8; 27] {
        [-1i8; 27]
    }

    #[test]
    fn test_fpga_new() {
        let driver = FpgaTpuDriver::new();
        assert_eq!(*driver.state(), TpuState::Uninitialized);
        assert_eq!(driver.capabilities().tpu_type, TpuType::Fpga);
        assert_eq!(driver.capabilities().max_trits_per_cycle, 729);
        assert_eq!(driver.capabilities().max_concurrent_ops, 16);
        assert_eq!(driver.capabilities().memory_size_bytes, 16 * 1024 * 1024);
        assert_eq!(driver.capabilities().dma_channels, 4);
    }

    #[test]
    fn test_fpga_load_firmware_valid() {
        let mut driver = FpgaTpuDriver::new();
        let fw = valid_firmware();
        assert!(driver.load_firmware(&fw).is_ok());
    }

    #[test]
    fn test_fpga_load_firmware_invalid_magic() {
        let mut driver = FpgaTpuDriver::new();
        let fw = vec![0x00, 0x00, 0x00, 0x00, 0xFF];
        assert!(driver.load_firmware(&fw).is_err());
    }

    #[test]
    fn test_fpga_load_firmware_too_short() {
        let mut driver = FpgaTpuDriver::new();
        let fw = vec![0x54, 0x50];
        assert!(driver.load_firmware(&fw).is_err());
    }

    #[test]
    fn test_fpga_initialize_without_firmware() {
        let mut driver = FpgaTpuDriver::new();
        assert!(driver.initialize().is_err());
    }

    #[test]
    fn test_fpga_initialize_with_firmware() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        assert!(driver.initialize().is_ok());
        assert_eq!(*driver.state(), TpuState::Ready);
    }

    #[test]
    fn test_fpga_submit_before_init() {
        let mut driver = FpgaTpuDriver::new();
        let result = driver.submit_command(TpuCommand::Nop);
        assert!(result.is_err());
    }

    #[test]
    fn test_fpga_submit_nop() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        let id = driver.submit_command(TpuCommand::Nop).unwrap();
        assert_eq!(id, 1);
        assert_eq!(driver.queue_depth(), 1);
    }

    #[test]
    fn test_fpga_process_nop() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.status, TpuResultStatus::Success);
        assert!(result.result_data.is_empty());
        assert_eq!(driver.total_operations(), 1);
    }

    #[test]
    fn test_fpga_process_empty_queue() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        let result = driver.process_next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_fpga_ternary_add_zeros() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryAdd {
            a: zero_trits(),
            b: zero_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, zero_trits().to_vec());
    }

    #[test]
    fn test_fpga_ternary_add_one_plus_one_wraps() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryAdd {
            a: ones_trits(),
            b: ones_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, neg_ones_trits().to_vec());
    }

    #[test]
    fn test_fpga_ternary_add_neg_one_plus_neg_one_wraps() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryAdd {
            a: neg_ones_trits(),
            b: neg_ones_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, ones_trits().to_vec());
    }

    #[test]
    fn test_fpga_ternary_add_one_plus_neg_one() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryAdd {
            a: ones_trits(),
            b: neg_ones_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, zero_trits().to_vec());
    }

    #[test]
    fn test_fpga_ternary_multiply_ones() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryMultiply {
            a: ones_trits(),
            b: ones_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, ones_trits().to_vec());
    }

    #[test]
    fn test_fpga_ternary_multiply_by_zero() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryMultiply {
            a: ones_trits(),
            b: zero_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, zero_trits().to_vec());
    }

    #[test]
    fn test_fpga_ternary_multiply_neg_times_neg() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryMultiply {
            a: neg_ones_trits(),
            b: neg_ones_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, ones_trits().to_vec());
    }

    #[test]
    fn test_fpga_ternary_rotate() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        let mut data = zero_trits();
        data[0] = 1;
        data[1] = -1;
        driver.submit_command(TpuCommand::TernaryRotate {
            data,
            positions: 1,
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data[0], data[26]);
        assert_eq!(result.result_data[1], 1);
        assert_eq!(result.result_data[2], -1);
    }

    #[test]
    fn test_fpga_ternary_rotate_negative() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        let mut data = zero_trits();
        data[0] = 1;
        driver.submit_command(TpuCommand::TernaryRotate {
            data,
            positions: -1,
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data[26], 1);
    }

    #[test]
    fn test_fpga_hash() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::Hash {
            data: alloc::vec![1, 1, 1],
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data.len(), 1);
    }

    #[test]
    fn test_fpga_get_result() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        let id = driver.submit_command(TpuCommand::Nop).unwrap();
        driver.process_next().unwrap();
        let result = driver.get_result(id);
        assert!(result.is_some());
        assert_eq!(result.unwrap().command_id, id);
    }

    #[test]
    fn test_fpga_get_result_not_found() {
        let driver = FpgaTpuDriver::new();
        assert!(driver.get_result(999).is_none());
    }

    #[test]
    fn test_fpga_queue_full() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.reconfigure(2).unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        let result = driver.submit_command(TpuCommand::Nop);
        assert!(result.is_err());
    }

    #[test]
    fn test_fpga_suspend_resume() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        assert!(driver.suspend().is_ok());
        assert_eq!(*driver.state(), TpuState::Suspended);
        assert!(driver.resume().is_ok());
        assert_eq!(*driver.state(), TpuState::Ready);
    }

    #[test]
    fn test_fpga_suspend_uninitialized_fails() {
        let mut driver = FpgaTpuDriver::new();
        assert!(driver.suspend().is_err());
    }

    #[test]
    fn test_fpga_resume_not_suspended_fails() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        assert!(driver.resume().is_err());
    }

    #[test]
    fn test_fpga_reset() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.process_next().unwrap();
        driver.reset().unwrap();
        assert_eq!(*driver.state(), TpuState::Uninitialized);
        assert_eq!(driver.queue_depth(), 0);
        assert_eq!(driver.total_operations(), 0);
    }

    #[test]
    fn test_fpga_reconfigure() {
        let mut driver = FpgaTpuDriver::new();
        assert!(driver.reconfigure(512).is_ok());
    }

    #[test]
    fn test_fpga_reconfigure_zero_fails() {
        let mut driver = FpgaTpuDriver::new();
        assert!(driver.reconfigure(0).is_err());
    }

    #[test]
    fn test_fpga_multiple_commands() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        assert_eq!(driver.queue_depth(), 3);
        driver.process_next().unwrap();
        assert_eq!(driver.queue_depth(), 2);
        driver.process_next().unwrap();
        driver.process_next().unwrap();
        assert_eq!(driver.queue_depth(), 0);
        assert_eq!(driver.total_operations(), 3);
    }

    #[test]
    fn test_fpga_phase_encrypt_decrypt_roundtrip() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        let data = alloc::vec![1, 0, -1, 1, 0, -1];
        let mut key = [0i8; 27];
        key[0] = 1;
        key[1] = -1;
        key[2] = 1;
        driver.submit_command(TpuCommand::PhaseEncrypt {
            data: data.clone(),
            key,
        }).unwrap();
        let encrypted = driver.process_next().unwrap().unwrap();
        driver.submit_command(TpuCommand::PhaseDecrypt {
            data: encrypted.result_data,
            key,
        }).unwrap();
        let decrypted = driver.process_next().unwrap().unwrap();
        assert_eq!(decrypted.result_data, data);
    }

    #[test]
    fn test_fpga_state_transitions_computing() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        assert_eq!(*driver.state(), TpuState::Ready);
        driver.submit_command(TpuCommand::Nop).unwrap();
        assert_eq!(*driver.state(), TpuState::Computing);
        driver.process_next().unwrap();
        assert_eq!(*driver.state(), TpuState::Ready);
    }

    #[test]
    fn test_asic_new() {
        let driver = AsicTpuDriver::new();
        assert_eq!(*driver.state(), TpuState::Uninitialized);
        assert_eq!(driver.capabilities().tpu_type, TpuType::Asic);
        assert_eq!(driver.capabilities().max_trits_per_cycle, 2187);
        assert_eq!(driver.capabilities().max_concurrent_ops, 64);
        assert_eq!(driver.capabilities().memory_size_bytes, 64 * 1024 * 1024);
        assert_eq!(driver.capabilities().dma_channels, 8);
    }

    #[test]
    fn test_asic_initialize_no_firmware_needed() {
        let mut driver = AsicTpuDriver::new();
        assert!(driver.initialize().is_ok());
        assert_eq!(*driver.state(), TpuState::Ready);
    }

    #[test]
    fn test_asic_rom_version() {
        let driver = AsicTpuDriver::new();
        assert_eq!(driver.rom_version(), 0x0001_0000);
    }

    #[test]
    fn test_asic_submit_and_process() {
        let mut driver = AsicTpuDriver::new();
        driver.initialize().unwrap();
        let id = driver.submit_command(TpuCommand::TernaryAdd {
            a: ones_trits(),
            b: neg_ones_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.command_id, id);
        assert_eq!(result.result_data, zero_trits().to_vec());
    }

    #[test]
    fn test_asic_suspend_resume() {
        let mut driver = AsicTpuDriver::new();
        driver.initialize().unwrap();
        assert!(driver.suspend().is_ok());
        assert_eq!(*driver.state(), TpuState::Suspended);
        assert!(driver.resume().is_ok());
        assert_eq!(*driver.state(), TpuState::Ready);
    }

    #[test]
    fn test_asic_reset() {
        let mut driver = AsicTpuDriver::new();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.process_next().unwrap();
        driver.reset().unwrap();
        assert_eq!(*driver.state(), TpuState::Uninitialized);
        assert_eq!(driver.total_operations(), 0);
    }

    #[test]
    fn test_asic_queue_management() {
        let mut driver = AsicTpuDriver::new();
        driver.initialize().unwrap();
        driver.reconfigure(3).unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        assert!(driver.submit_command(TpuCommand::Nop).is_err());
    }

    #[test]
    fn test_asic_ternary_multiply() {
        let mut driver = AsicTpuDriver::new();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::TernaryMultiply {
            a: neg_ones_trits(),
            b: ones_trits(),
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, neg_ones_trits().to_vec());
    }

    #[test]
    fn test_gf3_add_element_wise() {
        assert_eq!(gf3_add(-1, -1), 1);
        assert_eq!(gf3_add(-1, 0), -1);
        assert_eq!(gf3_add(-1, 1), 0);
        assert_eq!(gf3_add(0, 0), 0);
        assert_eq!(gf3_add(0, 1), 1);
        assert_eq!(gf3_add(1, 1), -1);
    }

    #[test]
    fn test_gf3_multiply_element_wise() {
        assert_eq!(gf3_multiply(-1, -1), 1);
        assert_eq!(gf3_multiply(-1, 0), 0);
        assert_eq!(gf3_multiply(-1, 1), -1);
        assert_eq!(gf3_multiply(0, 0), 0);
        assert_eq!(gf3_multiply(0, 1), 0);
        assert_eq!(gf3_multiply(1, 1), 1);
    }

    #[test]
    fn test_fpga_bulk_convert() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::BulkConvert {
            data: alloc::vec![-1, 0, 1],
            from_repr: 0,
            to_repr: 1,
        }).unwrap();
        let result = driver.process_next().unwrap().unwrap();
        assert_eq!(result.result_data, alloc::vec![0, 1, 2]);
    }

    #[test]
    fn test_fpga_resume_with_pending_commands() {
        let mut driver = FpgaTpuDriver::new();
        driver.load_firmware(&valid_firmware()).unwrap();
        driver.initialize().unwrap();
        driver.submit_command(TpuCommand::Nop).unwrap();
        driver.suspend().unwrap();
        driver.resume().unwrap();
        assert_eq!(*driver.state(), TpuState::Computing);
    }
}
