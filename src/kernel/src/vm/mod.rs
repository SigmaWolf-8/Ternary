use alloc::string::String;
use core::fmt;

pub mod instruction;
pub mod engine;
pub mod gc;

#[derive(Debug, Clone)]
pub enum VmError {
    InvalidOpcode(u8),
    InvalidRegister(u8),
    StackOverflow,
    StackUnderflow,
    DivisionByZero,
    SegmentationFault { address: u64 },
    InvalidMemoryAccess { address: u64, size: u64 },
    ProgramCounterOutOfBounds,
    HaltRequested,
    InvalidProgram(String),
    OutOfMemory,
    GcError(String),
    TernaryOverflow,
    InvalidTritValue(i8),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02X}", op),
            VmError::InvalidRegister(reg) => write!(f, "Invalid register: {}", reg),
            VmError::StackOverflow => write!(f, "Stack overflow"),
            VmError::StackUnderflow => write!(f, "Stack underflow"),
            VmError::DivisionByZero => write!(f, "Division by zero"),
            VmError::SegmentationFault { address } => {
                write!(f, "Segmentation fault at address 0x{:016X}", address)
            }
            VmError::InvalidMemoryAccess { address, size } => {
                write!(f, "Invalid memory access at 0x{:016X} size {}", address, size)
            }
            VmError::ProgramCounterOutOfBounds => write!(f, "Program counter out of bounds"),
            VmError::HaltRequested => write!(f, "Halt requested"),
            VmError::InvalidProgram(msg) => write!(f, "Invalid program: {}", msg),
            VmError::OutOfMemory => write!(f, "Out of memory"),
            VmError::GcError(msg) => write!(f, "GC error: {}", msg),
            VmError::TernaryOverflow => write!(f, "Ternary overflow"),
            VmError::InvalidTritValue(val) => write!(f, "Invalid trit value: {}", val),
        }
    }
}

pub type VmResult<T> = core::result::Result<T, VmError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_opcode_error() {
        let err = VmError::InvalidOpcode(0xFF);
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("0xFF"));
    }

    #[test]
    fn test_stack_overflow_display() {
        let err = VmError::StackOverflow;
        let msg = alloc::format!("{}", err);
        assert_eq!(msg, "Stack overflow");
    }

    #[test]
    fn test_segfault_display() {
        let err = VmError::SegmentationFault { address: 0xDEAD };
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("DEAD"));
    }

    #[test]
    fn test_invalid_program_display() {
        let err = VmError::InvalidProgram(String::from("bad bytecode"));
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("bad bytecode"));
    }

    #[test]
    fn test_gc_error_display() {
        let err = VmError::GcError(String::from("heap exhausted"));
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("heap exhausted"));
    }
}
