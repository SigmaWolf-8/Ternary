pub mod bus;
pub mod registry;
pub mod interrupt;
pub mod dma;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Block,
    Character,
    Network,
    TernaryCoprocessor,
    TimingUnit,
    PhaseEncryptor,
    Display,
    Input,
    Bus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Uninitialized,
    Initializing,
    Ready,
    Busy,
    Error,
    Suspended,
    Removed,
}

impl DeviceState {
    pub fn can_transition_to(&self, target: DeviceState) -> bool {
        use DeviceState::*;
        matches!(
            (self, target),
            (Uninitialized, Initializing)
                | (Initializing, Ready)
                | (Initializing, Error)
                | (Ready, Busy)
                | (Ready, Suspended)
                | (Ready, Removed)
                | (Ready, Error)
                | (Busy, Ready)
                | (Busy, Error)
                | (Suspended, Ready)
                | (Suspended, Removed)
                | (Error, Initializing)
                | (Error, Removed)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Storage,
    Communication,
    HumanInterface,
    Compute,
    Timing,
    Security,
    Virtual,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub device_type: DeviceType,
    pub device_class: DeviceClass,
    pub vendor_id: u16,
    pub product_id: u16,
    pub revision: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCapabilities {
    pub dma_capable: bool,
    pub interrupt_capable: bool,
    pub ternary_native: bool,
    pub max_transfer_size: usize,
    pub supported_modes: Vec<OperationMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    Polled,
    InterruptDriven,
    Dma,
    PhaseSynchronized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceError {
    NotFound,
    AlreadyExists,
    InvalidState,
    InvalidTransition,
    InitializationFailed,
    IoError,
    Timeout,
    BusFault,
    DmaError,
    InterruptError,
    CapacityExceeded,
    InvalidParameter,
    NotSupported,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::NotFound => write!(f, "Device not found"),
            DeviceError::AlreadyExists => write!(f, "Device already exists"),
            DeviceError::InvalidState => write!(f, "Invalid device state"),
            DeviceError::InvalidTransition => write!(f, "Invalid state transition"),
            DeviceError::InitializationFailed => write!(f, "Device initialization failed"),
            DeviceError::IoError => write!(f, "Device I/O error"),
            DeviceError::Timeout => write!(f, "Device timeout"),
            DeviceError::BusFault => write!(f, "Bus fault"),
            DeviceError::DmaError => write!(f, "DMA error"),
            DeviceError::InterruptError => write!(f, "Interrupt error"),
            DeviceError::CapacityExceeded => write!(f, "Capacity exceeded"),
            DeviceError::InvalidParameter => write!(f, "Invalid parameter"),
            DeviceError::NotSupported => write!(f, "Operation not supported"),
        }
    }
}

pub type DeviceResult<T> = Result<T, DeviceError>;

#[derive(Debug)]
pub struct DeviceDescriptor {
    pub info: DeviceInfo,
    pub state: DeviceState,
    pub capabilities: DeviceCapabilities,
    pub bus_id: Option<bus::BusId>,
    pub irq_line: Option<interrupt::IrqLine>,
    pub io_base: Option<u64>,
    pub io_size: usize,
    pub mmio_base: Option<u64>,
    pub mmio_size: usize,
}

impl DeviceDescriptor {
    pub fn new(info: DeviceInfo, capabilities: DeviceCapabilities) -> Self {
        Self {
            info,
            state: DeviceState::Uninitialized,
            capabilities,
            bus_id: None,
            irq_line: None,
            io_base: None,
            io_size: 0,
            mmio_base: None,
            mmio_size: 0,
        }
    }

    pub fn transition(&mut self, new_state: DeviceState) -> DeviceResult<()> {
        if self.state.can_transition_to(new_state) {
            self.state = new_state;
            Ok(())
        } else {
            Err(DeviceError::InvalidTransition)
        }
    }

    pub fn is_ready(&self) -> bool {
        self.state == DeviceState::Ready
    }

    pub fn is_busy(&self) -> bool {
        self.state == DeviceState::Busy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info(id: u32) -> DeviceInfo {
        DeviceInfo {
            id: DeviceId(id),
            name: String::from("test-device"),
            device_type: DeviceType::Block,
            device_class: DeviceClass::Storage,
            vendor_id: 0x1234,
            product_id: 0x5678,
            revision: 1,
        }
    }

    fn make_caps() -> DeviceCapabilities {
        DeviceCapabilities {
            dma_capable: true,
            interrupt_capable: true,
            ternary_native: false,
            max_transfer_size: 4096,
            supported_modes: alloc::vec![OperationMode::InterruptDriven],
        }
    }

    #[test]
    fn test_device_state_transitions() {
        assert!(DeviceState::Uninitialized.can_transition_to(DeviceState::Initializing));
        assert!(DeviceState::Initializing.can_transition_to(DeviceState::Ready));
        assert!(DeviceState::Ready.can_transition_to(DeviceState::Busy));
        assert!(DeviceState::Busy.can_transition_to(DeviceState::Ready));
        assert!(DeviceState::Ready.can_transition_to(DeviceState::Suspended));
        assert!(DeviceState::Suspended.can_transition_to(DeviceState::Ready));

        assert!(!DeviceState::Uninitialized.can_transition_to(DeviceState::Ready));
        assert!(!DeviceState::Busy.can_transition_to(DeviceState::Suspended));
        assert!(!DeviceState::Removed.can_transition_to(DeviceState::Ready));
    }

    #[test]
    fn test_device_descriptor_new() {
        let desc = DeviceDescriptor::new(make_info(1), make_caps());
        assert_eq!(desc.state, DeviceState::Uninitialized);
        assert_eq!(desc.info.id, DeviceId(1));
        assert!(desc.bus_id.is_none());
        assert!(desc.irq_line.is_none());
    }

    #[test]
    fn test_device_descriptor_transition() {
        let mut desc = DeviceDescriptor::new(make_info(1), make_caps());
        assert!(desc.transition(DeviceState::Initializing).is_ok());
        assert!(desc.transition(DeviceState::Ready).is_ok());
        assert!(desc.is_ready());
        assert!(desc.transition(DeviceState::Busy).is_ok());
        assert!(desc.is_busy());
        assert!(desc.transition(DeviceState::Ready).is_ok());
    }

    #[test]
    fn test_device_descriptor_invalid_transition() {
        let mut desc = DeviceDescriptor::new(make_info(1), make_caps());
        let result = desc.transition(DeviceState::Ready);
        assert_eq!(result, Err(DeviceError::InvalidTransition));
    }

    #[test]
    fn test_device_types() {
        assert_ne!(DeviceType::Block, DeviceType::Character);
        assert_ne!(DeviceType::TernaryCoprocessor, DeviceType::Network);
    }

    #[test]
    fn test_device_error_display() {
        let e = DeviceError::NotFound;
        let s = alloc::format!("{}", e);
        assert!(!s.is_empty());
    }
}
