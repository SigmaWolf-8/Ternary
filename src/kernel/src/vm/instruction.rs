use alloc::string::String;
use alloc::vec::Vec;
use super::{VmError, VmResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TernaryRegister {
    pub value: i64,
    pub ternary_mode: bool,
}

impl Default for TernaryRegister {
    fn default() -> Self {
        Self {
            value: 0,
            ternary_mode: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmFlags {
    pub zero: bool,
    pub negative: bool,
    pub overflow: bool,
    pub ternary: bool,
    pub halted: bool,
}

impl Default for VmFlags {
    fn default() -> Self {
        Self {
            zero: false,
            negative: false,
            overflow: false,
            ternary: false,
            halted: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegisterFile {
    pub registers: [TernaryRegister; 27],
    pub program_counter: u64,
    pub stack_pointer: u64,
    pub flags: VmFlags,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self {
            registers: [TernaryRegister::default(); 27],
            program_counter: 0,
            stack_pointer: 0,
            flags: VmFlags::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0x00,
    Halt = 0x01,
    Add = 0x02,
    Sub = 0x03,
    Mul = 0x04,
    Div = 0x05,
    Mod = 0x06,
    Neg = 0x07,

    TAdd = 0x10,
    TMul = 0x11,
    TNeg = 0x12,
    TRot = 0x13,
    TXor = 0x14,
    TConvert = 0x15,

    Load = 0x20,
    Store = 0x21,
    Move = 0x22,
    LoadImm = 0x23,
    Push = 0x24,
    Pop = 0x25,

    Jump = 0x30,
    JumpZero = 0x31,
    JumpNeg = 0x32,
    JumpPos = 0x33,
    Call = 0x34,
    Return = 0x35,
    JumpNotZero = 0x36,

    Cmp = 0x40,
    CmpImm = 0x41,

    And = 0x50,
    Or = 0x51,
    Xor = 0x52,
    Shl = 0x53,
    Shr = 0x54,
    Not = 0x55,
}

impl Opcode {
    pub fn from_u8(value: u8) -> VmResult<Self> {
        match value {
            0x00 => Ok(Opcode::Nop),
            0x01 => Ok(Opcode::Halt),
            0x02 => Ok(Opcode::Add),
            0x03 => Ok(Opcode::Sub),
            0x04 => Ok(Opcode::Mul),
            0x05 => Ok(Opcode::Div),
            0x06 => Ok(Opcode::Mod),
            0x07 => Ok(Opcode::Neg),
            0x10 => Ok(Opcode::TAdd),
            0x11 => Ok(Opcode::TMul),
            0x12 => Ok(Opcode::TNeg),
            0x13 => Ok(Opcode::TRot),
            0x14 => Ok(Opcode::TXor),
            0x15 => Ok(Opcode::TConvert),
            0x20 => Ok(Opcode::Load),
            0x21 => Ok(Opcode::Store),
            0x22 => Ok(Opcode::Move),
            0x23 => Ok(Opcode::LoadImm),
            0x24 => Ok(Opcode::Push),
            0x25 => Ok(Opcode::Pop),
            0x30 => Ok(Opcode::Jump),
            0x31 => Ok(Opcode::JumpZero),
            0x32 => Ok(Opcode::JumpNeg),
            0x33 => Ok(Opcode::JumpPos),
            0x34 => Ok(Opcode::Call),
            0x35 => Ok(Opcode::Return),
            0x36 => Ok(Opcode::JumpNotZero),
            0x40 => Ok(Opcode::Cmp),
            0x41 => Ok(Opcode::CmpImm),
            0x50 => Ok(Opcode::And),
            0x51 => Ok(Opcode::Or),
            0x52 => Ok(Opcode::Xor),
            0x53 => Ok(Opcode::Shl),
            0x54 => Ok(Opcode::Shr),
            0x55 => Ok(Opcode::Not),
            _ => Err(VmError::InvalidOpcode(value)),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub dst: u8,
    pub src1: u8,
    pub src2: u8,
    pub immediate: i64,
}

impl Instruction {
    pub fn new(opcode: Opcode, dst: u8, src1: u8, src2: u8, immediate: i64) -> Self {
        Self {
            opcode,
            dst,
            src1,
            src2,
            immediate,
        }
    }

    pub fn from_opcode(opcode: Opcode) -> Self {
        Self {
            opcode,
            dst: 0,
            src1: 0,
            src2: 0,
            immediate: 0,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.push(self.opcode.to_u8());
        bytes.push(self.dst);
        bytes.push(self.src1);
        bytes.push(self.src2);
        bytes.extend_from_slice(&self.immediate.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 4]);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> VmResult<Self> {
        if bytes.len() < 16 {
            return Err(VmError::InvalidProgram(String::from("Instruction too short")));
        }
        let opcode = Opcode::from_u8(bytes[0])?;
        let dst = bytes[1];
        let src1 = bytes[2];
        let src2 = bytes[3];
        let mut imm_bytes = [0u8; 8];
        imm_bytes.copy_from_slice(&bytes[4..12]);
        let immediate = i64::from_le_bytes(imm_bytes);
        Ok(Self {
            opcode,
            dst,
            src1,
            src2,
            immediate,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub entry_point: u64,
    pub name: String,
    pub data_segment: Vec<u8>,
}

impl Program {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            instructions: Vec::new(),
            entry_point: 0,
            name: name.into(),
            data_segment: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn get_instruction(&self, index: u64) -> Option<&Instruction> {
        self.instructions.get(index as usize)
    }

    pub fn validate(&self) -> VmResult<()> {
        if self.instructions.is_empty() {
            return Err(VmError::InvalidProgram(String::from("Empty program")));
        }
        for (i, inst) in self.instructions.iter().enumerate() {
            if inst.dst > 26 {
                return Err(VmError::InvalidRegister(inst.dst));
            }
            if inst.src1 > 26 {
                return Err(VmError::InvalidRegister(inst.src1));
            }
            if inst.src2 > 26 {
                return Err(VmError::InvalidRegister(inst.src2));
            }
            let _ = Opcode::from_u8(inst.opcode.to_u8()).map_err(|_| {
                VmError::InvalidProgram(alloc::format!("Invalid opcode at instruction {}", i))
            })?;
        }
        Ok(())
    }

    pub fn set_entry_point(&mut self, addr: u64) {
        self.entry_point = addr;
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data_segment = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip_nop() {
        let op = Opcode::Nop;
        let val = op.to_u8();
        assert_eq!(Opcode::from_u8(val).unwrap(), Opcode::Nop);
    }

    #[test]
    fn test_opcode_roundtrip_all() {
        let opcodes = [
            Opcode::Nop, Opcode::Halt, Opcode::Add, Opcode::Sub, Opcode::Mul,
            Opcode::Div, Opcode::Mod, Opcode::Neg, Opcode::TAdd, Opcode::TMul,
            Opcode::TNeg, Opcode::TRot, Opcode::TXor, Opcode::TConvert,
            Opcode::Load, Opcode::Store, Opcode::Move, Opcode::LoadImm,
            Opcode::Push, Opcode::Pop, Opcode::Jump, Opcode::JumpZero,
            Opcode::JumpNeg, Opcode::JumpPos, Opcode::Call, Opcode::Return,
            Opcode::JumpNotZero, Opcode::Cmp, Opcode::CmpImm,
            Opcode::And, Opcode::Or, Opcode::Xor, Opcode::Shl, Opcode::Shr,
            Opcode::Not,
        ];
        for op in &opcodes {
            let val = op.to_u8();
            let decoded = Opcode::from_u8(val).unwrap();
            assert_eq!(*op, decoded);
        }
    }

    #[test]
    fn test_opcode_invalid() {
        assert!(Opcode::from_u8(0xFF).is_err());
        assert!(Opcode::from_u8(0x08).is_err());
        assert!(Opcode::from_u8(0x99).is_err());
    }

    #[test]
    fn test_register_file_init() {
        let rf = RegisterFile::default();
        assert_eq!(rf.registers.len(), 27);
        for reg in &rf.registers {
            assert_eq!(reg.value, 0);
            assert!(!reg.ternary_mode);
        }
        assert_eq!(rf.program_counter, 0);
        assert_eq!(rf.stack_pointer, 0);
    }

    #[test]
    fn test_flags_default() {
        let flags = VmFlags::default();
        assert!(!flags.zero);
        assert!(!flags.negative);
        assert!(!flags.overflow);
        assert!(!flags.ternary);
        assert!(!flags.halted);
    }

    #[test]
    fn test_instruction_new() {
        let inst = Instruction::new(Opcode::Add, 0, 1, 2, 0);
        assert_eq!(inst.opcode, Opcode::Add);
        assert_eq!(inst.dst, 0);
        assert_eq!(inst.src1, 1);
        assert_eq!(inst.src2, 2);
        assert_eq!(inst.immediate, 0);
    }

    #[test]
    fn test_instruction_from_opcode() {
        let inst = Instruction::from_opcode(Opcode::Halt);
        assert_eq!(inst.opcode, Opcode::Halt);
        assert_eq!(inst.dst, 0);
        assert_eq!(inst.src1, 0);
        assert_eq!(inst.src2, 0);
        assert_eq!(inst.immediate, 0);
    }

    #[test]
    fn test_instruction_encode_length() {
        let inst = Instruction::from_opcode(Opcode::Nop);
        let encoded = inst.encode();
        assert_eq!(encoded.len(), 16);
    }

    #[test]
    fn test_instruction_encode_decode_roundtrip() {
        let inst = Instruction::new(Opcode::LoadImm, 5, 10, 15, 42);
        let encoded = inst.encode();
        let decoded = Instruction::decode(&encoded).unwrap();
        assert_eq!(decoded.opcode, Opcode::LoadImm);
        assert_eq!(decoded.dst, 5);
        assert_eq!(decoded.src1, 10);
        assert_eq!(decoded.src2, 15);
        assert_eq!(decoded.immediate, 42);
    }

    #[test]
    fn test_instruction_encode_negative_immediate() {
        let inst = Instruction::new(Opcode::LoadImm, 0, 0, 0, -12345);
        let encoded = inst.encode();
        let decoded = Instruction::decode(&encoded).unwrap();
        assert_eq!(decoded.immediate, -12345);
    }

    #[test]
    fn test_instruction_decode_too_short() {
        let bytes = [0u8; 8];
        assert!(Instruction::decode(&bytes).is_err());
    }

    #[test]
    fn test_instruction_decode_invalid_opcode() {
        let mut bytes = [0u8; 16];
        bytes[0] = 0xFF;
        assert!(Instruction::decode(&bytes).is_err());
    }

    #[test]
    fn test_encoding_format_opcode_byte() {
        let inst = Instruction::new(Opcode::Add, 0, 0, 0, 0);
        let encoded = inst.encode();
        assert_eq!(encoded[0], 0x02);
    }

    #[test]
    fn test_encoding_format_registers() {
        let inst = Instruction::new(Opcode::Nop, 3, 7, 11, 0);
        let encoded = inst.encode();
        assert_eq!(encoded[1], 3);
        assert_eq!(encoded[2], 7);
        assert_eq!(encoded[3], 11);
    }

    #[test]
    fn test_encoding_format_reserved_zeros() {
        let inst = Instruction::new(Opcode::Nop, 0, 0, 0, 0);
        let encoded = inst.encode();
        assert_eq!(encoded[12], 0);
        assert_eq!(encoded[13], 0);
        assert_eq!(encoded[14], 0);
        assert_eq!(encoded[15], 0);
    }

    #[test]
    fn test_program_new() {
        let prog = Program::new("test");
        assert_eq!(prog.name, "test");
        assert_eq!(prog.instruction_count(), 0);
        assert_eq!(prog.entry_point, 0);
    }

    #[test]
    fn test_program_add_instruction() {
        let mut prog = Program::new("test");
        prog.add_instruction(Instruction::from_opcode(Opcode::Nop));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        assert_eq!(prog.instruction_count(), 2);
    }

    #[test]
    fn test_program_get_instruction() {
        let mut prog = Program::new("test");
        prog.add_instruction(Instruction::from_opcode(Opcode::Nop));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        assert_eq!(prog.get_instruction(0).unwrap().opcode, Opcode::Nop);
        assert_eq!(prog.get_instruction(1).unwrap().opcode, Opcode::Halt);
        assert!(prog.get_instruction(2).is_none());
    }

    #[test]
    fn test_program_validate_empty() {
        let prog = Program::new("empty");
        assert!(prog.validate().is_err());
    }

    #[test]
    fn test_program_validate_valid() {
        let mut prog = Program::new("valid");
        prog.add_instruction(Instruction::new(Opcode::Add, 0, 1, 2, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        assert!(prog.validate().is_ok());
    }

    #[test]
    fn test_program_validate_invalid_register() {
        let mut prog = Program::new("bad");
        prog.add_instruction(Instruction::new(Opcode::Add, 27, 0, 0, 0));
        assert!(prog.validate().is_err());
    }

    #[test]
    fn test_program_validate_invalid_src1() {
        let mut prog = Program::new("bad");
        prog.add_instruction(Instruction::new(Opcode::Add, 0, 30, 0, 0));
        assert!(prog.validate().is_err());
    }

    #[test]
    fn test_program_validate_invalid_src2() {
        let mut prog = Program::new("bad");
        prog.add_instruction(Instruction::new(Opcode::Add, 0, 0, 50, 0));
        assert!(prog.validate().is_err());
    }

    #[test]
    fn test_program_set_entry_point() {
        let mut prog = Program::new("test");
        prog.set_entry_point(5);
        assert_eq!(prog.entry_point, 5);
    }

    #[test]
    fn test_program_set_data() {
        let mut prog = Program::new("test");
        prog.set_data(alloc::vec![1, 2, 3, 4]);
        assert_eq!(prog.data_segment.len(), 4);
    }
}
