// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.
//
// ============================================================================
// TVM ISA v2.1 — Enterprise-Grade Instruction Set
// Expanded from 62 to 176 opcodes (v2.1: +16 Quantum-Ternary)
// ============================================================================

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
    pub positive: bool,
    pub overflow: bool,
    pub ternary: bool,
    pub halted: bool,
    pub carry: bool,
    pub parity: bool,
    pub interrupt_enabled: bool,
}

impl Default for VmFlags {
    fn default() -> Self {
        Self {
            zero: false,
            negative: false,
            positive: false,
            overflow: false,
            ternary: false,
            halted: false,
            carry: false,
            parity: false,
            interrupt_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
}

impl Default for PrivilegeLevel {
    fn default() -> Self {
        PrivilegeLevel::Ring0
    }
}

#[derive(Debug, Clone)]
pub struct RegisterFile {
    pub registers: [TernaryRegister; 27],
    pub program_counter: u64,
    pub stack_pointer: u64,
    pub frame_pointer: u64,
    pub link_register: u64,
    pub flags: VmFlags,
    pub privilege: PrivilegeLevel,
    pub security_domain: u8,
    pub exception_vector: u64,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self {
            registers: [TernaryRegister::default(); 27],
            program_counter: 0,
            stack_pointer: 0,
            frame_pointer: 0,
            link_register: 0,
            flags: VmFlags::default(),
            privilege: PrivilegeLevel::default(),
            security_domain: 0,
            exception_vector: 0,
        }
    }
}

// ============================================================================
// ISA v2.1 Opcode Enumeration — 176 Instructions
// ============================================================================
//
// Layout by category and hex range:
//
//   0x00–0x0F  Basic & Extended Arithmetic        (16 opcodes)
//   0x10–0x1F  Ternary Core                       (16 opcodes)  [unchanged]
//   0x20–0x2F  Memory, Register & Atomics         (16 opcodes)
//   0x30–0x3F  Control Flow                       (16 opcodes)
//   0x40–0x4F  Comparison & Selection             (16 opcodes)
//   0x50–0x5F  Binary Logic & Bit Manipulation    (16 opcodes)
//   0x60–0x6F  Crypto Acceleration                (16 opcodes)
//   0x70–0x7F  SIMD / Vector Ternary              (16 opcodes)
//   0x80–0x8F  System & Privilege                 (16 opcodes)
//   0x90–0x97  Security & Audit                   (8 opcodes)
//   0x98–0x9F  Debug & Profiling                  (8 opcodes)
//   0xA0–0xAF  Quantum-Ternary Simulation         (16 opcodes)  [v2.1]
//
// Total: 176 allocated slots, 176 defined (0 reserved — fully populated)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // ========================================================================
    // 0x00–0x0F: Basic & Extended Arithmetic
    // ========================================================================
    Nop         = 0x00,
    Halt        = 0x01,
    Add         = 0x02,
    Sub         = 0x03,
    Mul         = 0x04,
    Div         = 0x05,
    Mod         = 0x06,
    Neg         = 0x07,
    Abs         = 0x08,
    AddImm      = 0x09,
    MulAdd      = 0x0A,
    MulSub      = 0x0B,
    DivMod      = 0x0C,
    Min         = 0x0D,
    Max         = 0x0E,
    Clamp       = 0x0F,

    // ========================================================================
    // 0x10–0x1F: Ternary Core (unchanged from v1.0)
    // ========================================================================
    TAdd        = 0x10,
    TMul        = 0x11,
    TNeg        = 0x12,
    TRot        = 0x13,
    TXor        = 0x14,
    TConvert    = 0x15,
    TAnd        = 0x16,
    TOr         = 0x17,
    TSub        = 0x18,
    TInv        = 0x19,
    TShift      = 0x1A,
    TCmp        = 0x1B,
    TLoad       = 0x1C,
    TStore      = 0x1D,
    TReduce     = 0x1E,
    TRotInv     = 0x1F,

    // ========================================================================
    // 0x20–0x2F: Memory, Register & Atomics
    // ========================================================================
    Load        = 0x20,
    Store       = 0x21,
    Move        = 0x22,
    LoadImm     = 0x23,
    Push        = 0x24,
    Pop         = 0x25,
    Lea         = 0x26,
    LoadByte    = 0x27,
    StoreByte   = 0x28,
    Exchange    = 0x29,
    CompareSwap = 0x2A,
    AtomicAdd   = 0x2B,
    Fence       = 0x2C,
    PushFrame   = 0x2D,
    PopFrame    = 0x2E,
    MemCopy     = 0x2F,

    // ========================================================================
    // 0x30–0x3F: Control Flow
    // ========================================================================
    Jump        = 0x30,
    JumpZero    = 0x31,
    JumpNeg     = 0x32,
    JumpPos     = 0x33,
    Call        = 0x34,
    Return      = 0x35,
    JumpNotZero = 0x36,
    JumpGe      = 0x37,
    JumpLe      = 0x38,
    JumpOvf     = 0x39,
    JumpCarry   = 0x3A,
    CallInd     = 0x3B,
    TailCall    = 0x3C,
    Loop        = 0x3D,
    JumpTable   = 0x3E,
    ReturnInt   = 0x3F,

    // ========================================================================
    // 0x40–0x4F: Comparison & Selection
    // ========================================================================
    Cmp         = 0x40,
    CmpImm      = 0x41,
    CmpUnsigned = 0x42,
    Test        = 0x43,
    Select      = 0x44,
    SelectNeg   = 0x45,
    SelectOvf   = 0x46,
    SignExtend  = 0x47,
    ZeroExtend  = 0x48,
    CountOnes   = 0x49,
    LeadZeros   = 0x4A,
    TrailZeros  = 0x4B,
    BitReverse  = 0x4C,
    ByteSwap    = 0x4D,
    TritCount   = 0x4E,
    TritBalance = 0x4F,

    // ========================================================================
    // 0x50–0x5F: Binary Logic & Bit Manipulation
    // ========================================================================
    And         = 0x50,
    Or          = 0x51,
    Xor         = 0x52,
    Shl         = 0x53,
    Shr         = 0x54,
    Not         = 0x55,
    ShrLogical  = 0x56,
    RotateLeft  = 0x57,
    RotateRight = 0x58,
    AndNot      = 0x59,
    OrNot       = 0x5A,
    Xnor        = 0x5B,
    BitDeposit  = 0x5C,
    BitExtract  = 0x5D,
    BitField    = 0x5E,
    Crc32       = 0x5F,

    // ========================================================================
    // 0x60–0x6F: Crypto Acceleration
    // ========================================================================
    TPolyMul    = 0x60,
    TNTT        = 0x61,
    THash       = 0x62,
    TEntropy    = 0x63,
    TPolyAdd    = 0x64,
    TPolySample = 0x65,
    TCompress   = 0x66,
    TDecompress = 0x67,
    TLatticeMul = 0x68,
    TSpongeAbsorb = 0x69,
    TSpongeSqueeze = 0x6A,
    TLamportGen = 0x6B,
    TLamportSign = 0x6C,
    TKemEncaps  = 0x6D,
    TKemDecaps  = 0x6E,
    TZeroize    = 0x6F,

    // ========================================================================
    // 0x70–0x7F: SIMD / Vector Ternary
    // ========================================================================
    TAddV       = 0x70,
    TMulV       = 0x71,
    TNegV       = 0x72,
    TRotV       = 0x73,
    TXorV       = 0x74,
    TAndV       = 0x75,
    TOrV        = 0x76,
    TCmpV       = 0x77,
    TShiftV     = 0x78,
    TReduceV    = 0x79,
    TConvertV   = 0x7A,
    TSelectV    = 0x7B,
    TBroadcast  = 0x7C,
    TExtract    = 0x7D,
    TInsert     = 0x7E,
    TPermute    = 0x7F,

    // ========================================================================
    // 0x80–0x8F: System & Privilege
    // ========================================================================
    Syscall     = 0x80,
    Trap        = 0x81,
    Alloc       = 0x82,
    Free        = 0x83,
    ReadTime    = 0x84,
    PrivEscalate = 0x85,
    PrivDrop    = 0x86,
    DomainSet   = 0x87,
    DomainGet   = 0x88,
    GetCycles   = 0x89,
    GetPid      = 0x8A,
    MProtect    = 0x8B,
    IoRead      = 0x8C,
    IoWrite     = 0x8D,
    CpuId       = 0x8E,
    Yield       = 0x8F,

    // ========================================================================
    // 0x90–0x97: Security & Audit
    // ========================================================================
    AuditLog    = 0x90,
    CapCheck    = 0x91,
    CapGrant    = 0x92,
    CapRevoke   = 0x93,
    SideChMask  = 0x94,
    SideChUnmask = 0x95,
    ConstTimeEq = 0x96,
    ConstTimeSel = 0x97,

    // ========================================================================
    // 0x98–0x9F: Debug & Profiling
    // ========================================================================
    DebugBreak  = 0x98,
    DebugPrint  = 0x99,
    ProfStart   = 0x9A,
    ProfStop    = 0x9B,
    ProfRead    = 0x9C,
    TraceEmit   = 0x9D,
    AssertEq    = 0x9E,
    AssertTrit  = 0x9F,

    // ========================================================================
    // 0xA0–0xAF: Quantum-Ternary Simulation  [v2.1 — Qutrit/Qudit Extension]
    // ========================================================================
    QStatePrep  = 0xA0,
    QGateApply  = 0xA1,
    QMeasure    = 0xA2,
    QEntangle   = 0xA3,
    QSyndrome   = 0xA4,
    QCorrect    = 0xA5,
    QDistill    = 0xA6,
    QPhaseGate  = 0xA7,
    QFidelity   = 0xA8,
    QUnitCheck  = 0xA9,
    QKronProd   = 0xAA,
    QStabEncode = 0xAB,
    QErrInject  = 0xAC,
    QExpectVal  = 0xAD,
    QNormalize  = 0xAE,
    QFTBench    = 0xAF,
}

// ============================================================================
// Opcode decoding — from_u8
// ============================================================================

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
            0x08 => Ok(Opcode::Abs),
            0x09 => Ok(Opcode::AddImm),
            0x0A => Ok(Opcode::MulAdd),
            0x0B => Ok(Opcode::MulSub),
            0x0C => Ok(Opcode::DivMod),
            0x0D => Ok(Opcode::Min),
            0x0E => Ok(Opcode::Max),
            0x0F => Ok(Opcode::Clamp),

            0x10 => Ok(Opcode::TAdd),
            0x11 => Ok(Opcode::TMul),
            0x12 => Ok(Opcode::TNeg),
            0x13 => Ok(Opcode::TRot),
            0x14 => Ok(Opcode::TXor),
            0x15 => Ok(Opcode::TConvert),
            0x16 => Ok(Opcode::TAnd),
            0x17 => Ok(Opcode::TOr),
            0x18 => Ok(Opcode::TSub),
            0x19 => Ok(Opcode::TInv),
            0x1A => Ok(Opcode::TShift),
            0x1B => Ok(Opcode::TCmp),
            0x1C => Ok(Opcode::TLoad),
            0x1D => Ok(Opcode::TStore),
            0x1E => Ok(Opcode::TReduce),
            0x1F => Ok(Opcode::TRotInv),

            0x20 => Ok(Opcode::Load),
            0x21 => Ok(Opcode::Store),
            0x22 => Ok(Opcode::Move),
            0x23 => Ok(Opcode::LoadImm),
            0x24 => Ok(Opcode::Push),
            0x25 => Ok(Opcode::Pop),
            0x26 => Ok(Opcode::Lea),
            0x27 => Ok(Opcode::LoadByte),
            0x28 => Ok(Opcode::StoreByte),
            0x29 => Ok(Opcode::Exchange),
            0x2A => Ok(Opcode::CompareSwap),
            0x2B => Ok(Opcode::AtomicAdd),
            0x2C => Ok(Opcode::Fence),
            0x2D => Ok(Opcode::PushFrame),
            0x2E => Ok(Opcode::PopFrame),
            0x2F => Ok(Opcode::MemCopy),

            0x30 => Ok(Opcode::Jump),
            0x31 => Ok(Opcode::JumpZero),
            0x32 => Ok(Opcode::JumpNeg),
            0x33 => Ok(Opcode::JumpPos),
            0x34 => Ok(Opcode::Call),
            0x35 => Ok(Opcode::Return),
            0x36 => Ok(Opcode::JumpNotZero),
            0x37 => Ok(Opcode::JumpGe),
            0x38 => Ok(Opcode::JumpLe),
            0x39 => Ok(Opcode::JumpOvf),
            0x3A => Ok(Opcode::JumpCarry),
            0x3B => Ok(Opcode::CallInd),
            0x3C => Ok(Opcode::TailCall),
            0x3D => Ok(Opcode::Loop),
            0x3E => Ok(Opcode::JumpTable),
            0x3F => Ok(Opcode::ReturnInt),

            0x40 => Ok(Opcode::Cmp),
            0x41 => Ok(Opcode::CmpImm),
            0x42 => Ok(Opcode::CmpUnsigned),
            0x43 => Ok(Opcode::Test),
            0x44 => Ok(Opcode::Select),
            0x45 => Ok(Opcode::SelectNeg),
            0x46 => Ok(Opcode::SelectOvf),
            0x47 => Ok(Opcode::SignExtend),
            0x48 => Ok(Opcode::ZeroExtend),
            0x49 => Ok(Opcode::CountOnes),
            0x4A => Ok(Opcode::LeadZeros),
            0x4B => Ok(Opcode::TrailZeros),
            0x4C => Ok(Opcode::BitReverse),
            0x4D => Ok(Opcode::ByteSwap),
            0x4E => Ok(Opcode::TritCount),
            0x4F => Ok(Opcode::TritBalance),

            0x50 => Ok(Opcode::And),
            0x51 => Ok(Opcode::Or),
            0x52 => Ok(Opcode::Xor),
            0x53 => Ok(Opcode::Shl),
            0x54 => Ok(Opcode::Shr),
            0x55 => Ok(Opcode::Not),
            0x56 => Ok(Opcode::ShrLogical),
            0x57 => Ok(Opcode::RotateLeft),
            0x58 => Ok(Opcode::RotateRight),
            0x59 => Ok(Opcode::AndNot),
            0x5A => Ok(Opcode::OrNot),
            0x5B => Ok(Opcode::Xnor),
            0x5C => Ok(Opcode::BitDeposit),
            0x5D => Ok(Opcode::BitExtract),
            0x5E => Ok(Opcode::BitField),
            0x5F => Ok(Opcode::Crc32),

            0x60 => Ok(Opcode::TPolyMul),
            0x61 => Ok(Opcode::TNTT),
            0x62 => Ok(Opcode::THash),
            0x63 => Ok(Opcode::TEntropy),
            0x64 => Ok(Opcode::TPolyAdd),
            0x65 => Ok(Opcode::TPolySample),
            0x66 => Ok(Opcode::TCompress),
            0x67 => Ok(Opcode::TDecompress),
            0x68 => Ok(Opcode::TLatticeMul),
            0x69 => Ok(Opcode::TSpongeAbsorb),
            0x6A => Ok(Opcode::TSpongeSqueeze),
            0x6B => Ok(Opcode::TLamportGen),
            0x6C => Ok(Opcode::TLamportSign),
            0x6D => Ok(Opcode::TKemEncaps),
            0x6E => Ok(Opcode::TKemDecaps),
            0x6F => Ok(Opcode::TZeroize),

            0x70 => Ok(Opcode::TAddV),
            0x71 => Ok(Opcode::TMulV),
            0x72 => Ok(Opcode::TNegV),
            0x73 => Ok(Opcode::TRotV),
            0x74 => Ok(Opcode::TXorV),
            0x75 => Ok(Opcode::TAndV),
            0x76 => Ok(Opcode::TOrV),
            0x77 => Ok(Opcode::TCmpV),
            0x78 => Ok(Opcode::TShiftV),
            0x79 => Ok(Opcode::TReduceV),
            0x7A => Ok(Opcode::TConvertV),
            0x7B => Ok(Opcode::TSelectV),
            0x7C => Ok(Opcode::TBroadcast),
            0x7D => Ok(Opcode::TExtract),
            0x7E => Ok(Opcode::TInsert),
            0x7F => Ok(Opcode::TPermute),

            0x80 => Ok(Opcode::Syscall),
            0x81 => Ok(Opcode::Trap),
            0x82 => Ok(Opcode::Alloc),
            0x83 => Ok(Opcode::Free),
            0x84 => Ok(Opcode::ReadTime),
            0x85 => Ok(Opcode::PrivEscalate),
            0x86 => Ok(Opcode::PrivDrop),
            0x87 => Ok(Opcode::DomainSet),
            0x88 => Ok(Opcode::DomainGet),
            0x89 => Ok(Opcode::GetCycles),
            0x8A => Ok(Opcode::GetPid),
            0x8B => Ok(Opcode::MProtect),
            0x8C => Ok(Opcode::IoRead),
            0x8D => Ok(Opcode::IoWrite),
            0x8E => Ok(Opcode::CpuId),
            0x8F => Ok(Opcode::Yield),

            0x90 => Ok(Opcode::AuditLog),
            0x91 => Ok(Opcode::CapCheck),
            0x92 => Ok(Opcode::CapGrant),
            0x93 => Ok(Opcode::CapRevoke),
            0x94 => Ok(Opcode::SideChMask),
            0x95 => Ok(Opcode::SideChUnmask),
            0x96 => Ok(Opcode::ConstTimeEq),
            0x97 => Ok(Opcode::ConstTimeSel),

            0x98 => Ok(Opcode::DebugBreak),
            0x99 => Ok(Opcode::DebugPrint),
            0x9A => Ok(Opcode::ProfStart),
            0x9B => Ok(Opcode::ProfStop),
            0x9C => Ok(Opcode::ProfRead),
            0x9D => Ok(Opcode::TraceEmit),
            0x9E => Ok(Opcode::AssertEq),
            0x9F => Ok(Opcode::AssertTrit),

            0xA0 => Ok(Opcode::QStatePrep),
            0xA1 => Ok(Opcode::QGateApply),
            0xA2 => Ok(Opcode::QMeasure),
            0xA3 => Ok(Opcode::QEntangle),
            0xA4 => Ok(Opcode::QSyndrome),
            0xA5 => Ok(Opcode::QCorrect),
            0xA6 => Ok(Opcode::QDistill),
            0xA7 => Ok(Opcode::QPhaseGate),
            0xA8 => Ok(Opcode::QFidelity),
            0xA9 => Ok(Opcode::QUnitCheck),
            0xAA => Ok(Opcode::QKronProd),
            0xAB => Ok(Opcode::QStabEncode),
            0xAC => Ok(Opcode::QErrInject),
            0xAD => Ok(Opcode::QExpectVal),
            0xAE => Ok(Opcode::QNormalize),
            0xAF => Ok(Opcode::QFTBench),

            _ => Err(VmError::InvalidOpcode(value)),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn mnemonic(&self) -> &'static str {
        match self {
            Opcode::Nop => "NOP", Opcode::Halt => "HALT", Opcode::Add => "ADD",
            Opcode::Sub => "SUB", Opcode::Mul => "MUL", Opcode::Div => "DIV",
            Opcode::Mod => "MOD", Opcode::Neg => "NEG", Opcode::Abs => "ABS",
            Opcode::AddImm => "ADDI", Opcode::MulAdd => "MADD", Opcode::MulSub => "MSUB",
            Opcode::DivMod => "DIVMOD", Opcode::Min => "MIN", Opcode::Max => "MAX",
            Opcode::Clamp => "CLAMP",

            Opcode::TAdd => "TADD", Opcode::TMul => "TMUL", Opcode::TNeg => "TNEG",
            Opcode::TRot => "TROT", Opcode::TXor => "TXOR", Opcode::TConvert => "TCVT",
            Opcode::TAnd => "TAND", Opcode::TOr => "TOR", Opcode::TSub => "TSUB",
            Opcode::TInv => "TINV", Opcode::TShift => "TSHL", Opcode::TCmp => "TCMP",
            Opcode::TLoad => "TLD", Opcode::TStore => "TST", Opcode::TReduce => "TRED",
            Opcode::TRotInv => "TROTI",

            Opcode::Load => "LD", Opcode::Store => "ST", Opcode::Move => "MOV",
            Opcode::LoadImm => "LDI", Opcode::Push => "PUSH", Opcode::Pop => "POP",
            Opcode::Lea => "LEA", Opcode::LoadByte => "LDB", Opcode::StoreByte => "STB",
            Opcode::Exchange => "XCHG", Opcode::CompareSwap => "CAS",
            Opcode::AtomicAdd => "AADD", Opcode::Fence => "FENCE",
            Opcode::PushFrame => "ENTER", Opcode::PopFrame => "LEAVE",
            Opcode::MemCopy => "MCOPY",

            Opcode::Jump => "JMP", Opcode::JumpZero => "JZ", Opcode::JumpNeg => "JN",
            Opcode::JumpPos => "JP", Opcode::Call => "CALL", Opcode::Return => "RET",
            Opcode::JumpNotZero => "JNZ", Opcode::JumpGe => "JGE", Opcode::JumpLe => "JLE",
            Opcode::JumpOvf => "JO", Opcode::JumpCarry => "JC", Opcode::CallInd => "CALLI",
            Opcode::TailCall => "TCALL", Opcode::Loop => "LOOP",
            Opcode::JumpTable => "JTAB", Opcode::ReturnInt => "IRET",

            Opcode::Cmp => "CMP", Opcode::CmpImm => "CMPI", Opcode::CmpUnsigned => "CMPU",
            Opcode::Test => "TEST", Opcode::Select => "SEL", Opcode::SelectNeg => "SELN",
            Opcode::SelectOvf => "SELO", Opcode::SignExtend => "SEXT",
            Opcode::ZeroExtend => "ZEXT", Opcode::CountOnes => "POPCNT",
            Opcode::LeadZeros => "CLZ", Opcode::TrailZeros => "CTZ",
            Opcode::BitReverse => "BREV", Opcode::ByteSwap => "BSWAP",
            Opcode::TritCount => "TCNT", Opcode::TritBalance => "TBAL",

            Opcode::And => "AND", Opcode::Or => "OR", Opcode::Xor => "XOR",
            Opcode::Shl => "SHL", Opcode::Shr => "SHR", Opcode::Not => "NOT",
            Opcode::ShrLogical => "SHRL", Opcode::RotateLeft => "ROL",
            Opcode::RotateRight => "ROR", Opcode::AndNot => "ANDN",
            Opcode::OrNot => "ORN", Opcode::Xnor => "XNOR",
            Opcode::BitDeposit => "PDEP", Opcode::BitExtract => "PEXT",
            Opcode::BitField => "BFX", Opcode::Crc32 => "CRC32",

            Opcode::TPolyMul => "TPMUL", Opcode::TNTT => "TNTT", Opcode::THash => "THASH",
            Opcode::TEntropy => "TENT", Opcode::TPolyAdd => "TPADD",
            Opcode::TPolySample => "TPSAMP", Opcode::TCompress => "TCOMP",
            Opcode::TDecompress => "TDCOMP", Opcode::TLatticeMul => "TLATMUL",
            Opcode::TSpongeAbsorb => "TSPABS", Opcode::TSpongeSqueeze => "TSPSQZ",
            Opcode::TLamportGen => "TLGEN", Opcode::TLamportSign => "TLSIGN",
            Opcode::TKemEncaps => "TKENC", Opcode::TKemDecaps => "TKDEC",
            Opcode::TZeroize => "TZERO",

            Opcode::TAddV => "TADDV", Opcode::TMulV => "TMULV", Opcode::TNegV => "TNEGV",
            Opcode::TRotV => "TROTV", Opcode::TXorV => "TXORV", Opcode::TAndV => "TANDV",
            Opcode::TOrV => "TORV", Opcode::TCmpV => "TCMPV", Opcode::TShiftV => "TSHLV",
            Opcode::TReduceV => "TREDV", Opcode::TConvertV => "TCVTV",
            Opcode::TSelectV => "TSELV", Opcode::TBroadcast => "TBCAST",
            Opcode::TExtract => "TEXTR", Opcode::TInsert => "TINS",
            Opcode::TPermute => "TPERM",

            Opcode::Syscall => "SYS", Opcode::Trap => "INT", Opcode::Alloc => "ALLOC",
            Opcode::Free => "FREE", Opcode::ReadTime => "RDTIME",
            Opcode::PrivEscalate => "PRIVESC", Opcode::PrivDrop => "PRIVDROP",
            Opcode::DomainSet => "DOMSET", Opcode::DomainGet => "DOMGET",
            Opcode::GetCycles => "GETCYC", Opcode::GetPid => "GETPID",
            Opcode::MProtect => "MPROT", Opcode::IoRead => "IOR",
            Opcode::IoWrite => "IOW", Opcode::CpuId => "CPUID",
            Opcode::Yield => "YIELD",

            Opcode::AuditLog => "AUDIT", Opcode::CapCheck => "CAPCHK",
            Opcode::CapGrant => "CAPGNT", Opcode::CapRevoke => "CAPREV",
            Opcode::SideChMask => "SCMASK", Opcode::SideChUnmask => "SCUNM",
            Opcode::ConstTimeEq => "CTEQ", Opcode::ConstTimeSel => "CTSEL",

            Opcode::DebugBreak => "DBRK", Opcode::DebugPrint => "DPRINT",
            Opcode::ProfStart => "PSTART", Opcode::ProfStop => "PSTOP",
            Opcode::ProfRead => "PREAD", Opcode::TraceEmit => "TRACE",
            Opcode::AssertEq => "ASRT", Opcode::AssertTrit => "ASRTT",

            Opcode::QStatePrep => "QPREP", Opcode::QGateApply => "QGATE",
            Opcode::QMeasure => "QMEAS", Opcode::QEntangle => "QENT",
            Opcode::QSyndrome => "QSYN", Opcode::QCorrect => "QCORR",
            Opcode::QDistill => "QDIST", Opcode::QPhaseGate => "QPHASE",
            Opcode::QFidelity => "QFID", Opcode::QUnitCheck => "QUNIT",
            Opcode::QKronProd => "QKRON", Opcode::QStabEncode => "QSTAB",
            Opcode::QErrInject => "QERR", Opcode::QExpectVal => "QEXP",
            Opcode::QNormalize => "QNORM", Opcode::QFTBench => "QFTB",
        }
    }

    pub fn category(&self) -> &'static str {
        match self.to_u8() {
            0x00..=0x0F => "Arithmetic",
            0x10..=0x1F => "Ternary Core",
            0x20..=0x2F => "Memory & Atomics",
            0x30..=0x3F => "Control Flow",
            0x40..=0x4F => "Comparison & Selection",
            0x50..=0x5F => "Binary Logic & Bits",
            0x60..=0x6F => "Crypto Acceleration",
            0x70..=0x7F => "SIMD Vector",
            0x80..=0x8F => "System & Privilege",
            0x90..=0x97 => "Security & Audit",
            0x98..=0x9F => "Debug & Profiling",
            0xA0..=0xAF => "Quantum-Ternary",
            _ => "Unknown",
        }
    }

    pub fn is_ternary(&self) -> bool {
        let v = self.to_u8();
        matches!(v, 0x10..=0x1F | 0x60..=0x7F | 0x4E | 0x4F)
    }

    pub fn is_branch(&self) -> bool {
        let v = self.to_u8();
        matches!(v, 0x01 | 0x30..=0x3F)
    }

    pub fn is_memory_op(&self) -> bool {
        let v = self.to_u8();
        matches!(v, 0x1C | 0x1D | 0x20..=0x2F | 0x6F)
    }

    pub fn is_crypto(&self) -> bool {
        let v = self.to_u8();
        matches!(v, 0x60..=0x6F | 0x94..=0x97)
    }

    pub fn requires_ring0(&self) -> bool {
        matches!(self,
            Opcode::Trap | Opcode::DomainSet | Opcode::CapGrant | Opcode::CapRevoke |
            Opcode::MProtect | Opcode::IoRead | Opcode::IoWrite | Opcode::PrivEscalate
        )
    }

    pub fn is_v2(&self) -> bool {
        let v = self.to_u8();
        matches!(v,
            0x08..=0x0F | 0x26..=0x2F | 0x37..=0x3F | 0x42..=0x4F |
            0x56..=0x5F | 0x68..=0x6F | 0x74..=0x7F | 0x85..=0x8F |
            0x90..=0x9F
        )
    }
}

// ============================================================================
// Instruction encoding
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub dst: u8,
    pub src1: u8,
    pub src2: u8,
    pub immediate: i64,
}

impl Instruction {
    pub fn new(opcode: Opcode, dst: u8, src1: u8, src2: u8, immediate: i64) -> Self {
        Self { opcode, dst, src1, src2, immediate }
    }

    pub fn encode(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0] = self.opcode.to_u8();
        bytes[1] = self.dst;
        bytes[2] = self.src1;
        bytes[3] = self.src2;
        let imm_bytes = self.immediate.to_le_bytes();
        bytes[4..12].copy_from_slice(&imm_bytes);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> VmResult<Self> {
        if bytes.len() < 16 {
            return Err(VmError::InvalidProgram(
                String::from("Instruction too short (need 16 bytes)"),
            ));
        }
        let opcode = Opcode::from_u8(bytes[0])?;
        let dst = bytes[1];
        let src1 = bytes[2];
        let src2 = bytes[3];
        let mut imm_bytes = [0u8; 8];
        imm_bytes.copy_from_slice(&bytes[4..12]);
        let immediate = i64::from_le_bytes(imm_bytes);
        Ok(Self { opcode, dst, src1, src2, immediate })
    }

    pub fn encode_compact(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.opcode.to_u8());
        let packed = (self.dst as u16 & 0x1F)
            | ((self.src1 as u16 & 0x1F) << 5)
            | ((self.src2 as u16 & 0x1F) << 10);
        bytes.extend_from_slice(&packed.to_le_bytes());
        if self.immediate != 0 {
            bytes.push(0x01);
            let imm16 = self.immediate as i16;
            bytes.extend_from_slice(&imm16.to_le_bytes());
        } else {
            bytes.push(0x00);
        }
        bytes
    }

    pub fn decode_compact(bytes: &[u8]) -> VmResult<(Self, usize)> {
        if bytes.len() < 4 {
            return Err(VmError::InvalidProgram(
                String::from("Compact instruction too short"),
            ));
        }
        let opcode = Opcode::from_u8(bytes[0])?;
        let packed = u16::from_le_bytes([bytes[1], bytes[2]]);
        let dst = (packed & 0x1F) as u8;
        let src1 = ((packed >> 5) & 0x1F) as u8;
        let src2 = ((packed >> 10) & 0x1F) as u8;
        let has_imm = bytes[3] != 0;

        if has_imm {
            if bytes.len() < 6 {
                return Err(VmError::InvalidProgram(
                    String::from("Compact instruction with immediate too short"),
                ));
            }
            let imm16 = i16::from_le_bytes([bytes[4], bytes[5]]);
            Ok((Self { opcode, dst, src1, src2, immediate: imm16 as i64 }, 6))
        } else {
            Ok((Self { opcode, dst, src1, src2, immediate: 0 }, 4))
        }
    }

    pub fn encode_ternary(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let op_val = self.opcode.to_u8();
        let mut trit_bytes = [0u8; 2];
        trit_bytes[0] = op_val;
        trit_bytes[1] = op_val.wrapping_mul(3) ^ 0xA5;
        bytes.extend_from_slice(&trit_bytes);

        let packed: u32 = (self.dst as u32 & 0x1F) << 16
            | (self.src1 as u32 & 0x1F) << 6
            | (self.src2 as u32 & 0x1F) << 1;

        if self.immediate != 0 {
            let p = packed | 1;
            bytes.extend_from_slice(&(p as u16).to_le_bytes());
            bytes.push((p >> 16) as u8);
            let imm16 = self.immediate as i16;
            bytes.extend_from_slice(&imm16.to_le_bytes());
        } else {
            bytes.extend_from_slice(&(packed as u16).to_le_bytes());
            bytes.push((packed >> 16) as u8);
        }
        bytes
    }

    pub fn decode_ternary(bytes: &[u8]) -> VmResult<(Self, usize)> {
        if bytes.len() < 5 {
            return Err(VmError::InvalidProgram(
                String::from("Ternary instruction too short"),
            ));
        }
        let opcode = Opcode::from_u8(bytes[0])?;
        let packed = (bytes[2] as u32) | ((bytes[3] as u32) << 8) | ((bytes[4] as u32) << 16);
        let dst = ((packed >> 16) & 0x1F) as u8;
        let src1 = ((packed >> 6) & 0x1F) as u8;
        let src2 = ((packed >> 1) & 0x1F) as u8;
        let has_imm = (packed & 1) != 0;

        if has_imm {
            if bytes.len() < 7 {
                return Err(VmError::InvalidProgram(
                    String::from("Ternary instruction with immediate too short"),
                ));
            }
            let imm16 = i16::from_le_bytes([bytes[5], bytes[6]]);
            Ok((Self { opcode, dst, src1, src2, immediate: imm16 as i64 }, 7))
        } else {
            Ok((Self { opcode, dst, src1, src2, immediate: 0 }, 5))
        }
    }

    pub fn decode_auto(bytes: &[u8], format: InstructionFormat) -> VmResult<(Self, usize)> {
        match format {
            InstructionFormat::Legacy => {
                let inst = Self::decode(bytes)?;
                Ok((inst, 16))
            }
            InstructionFormat::Compact => Self::decode_compact(bytes),
            InstructionFormat::BalancedTernary => Self::decode_ternary(bytes),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    Legacy,
    Compact,
    BalancedTernary,
}

// ============================================================================
// Program container
// ============================================================================

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub data_segment: Vec<u8>,
    pub entry_point: usize,
    pub isa_version: (u8, u8),
}

impl Program {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            instructions: Vec::new(),
            data_segment: Vec::new(),
            entry_point: 0,
            isa_version: (2, 0),
        }
    }

    pub fn add_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    pub fn get_instruction(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn set_entry_point(&mut self, ep: usize) {
        self.entry_point = ep;
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data_segment = data;
    }

    pub fn validate(&self) -> VmResult<()> {
        if self.instructions.is_empty() {
            return Err(VmError::InvalidProgram(
                String::from("Program has no instructions"),
            ));
        }
        for (i, inst) in self.instructions.iter().enumerate() {
            if inst.dst > 26 {
                return Err(VmError::InvalidProgram(
                    alloc::format!("Instruction {} has invalid dst register: {}", i, inst.dst),
                ));
            }
            if inst.src1 > 26 {
                return Err(VmError::InvalidProgram(
                    alloc::format!("Instruction {} has invalid src1 register: {}", i, inst.src1),
                ));
            }
            if inst.src2 > 26 {
                return Err(VmError::InvalidProgram(
                    alloc::format!("Instruction {} has invalid src2 register: {}", i, inst.src2),
                ));
            }
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_count() {
        let mut count = 0u32;
        for byte in 0x00u8..=0xFF {
            if Opcode::from_u8(byte).is_ok() {
                count += 1;
            }
        }
        assert_eq!(count, 176, "Expected 176 opcodes, found {}", count);
    }

    #[test]
    fn test_all_opcodes_roundtrip() {
        for byte in 0x00u8..=0xFF {
            if let Ok(op) = Opcode::from_u8(byte) {
                assert_eq!(op.to_u8(), byte, "Roundtrip failed for 0x{:02X}", byte);
            }
        }
    }

    #[test]
    fn test_all_opcodes_have_mnemonics() {
        for byte in 0x00u8..=0xFF {
            if let Ok(op) = Opcode::from_u8(byte) {
                let m = op.mnemonic();
                assert!(!m.is_empty(), "Empty mnemonic for 0x{:02X}", byte);
            }
        }
    }

    #[test]
    fn test_all_opcodes_have_categories() {
        for byte in 0x00u8..=0xFF {
            if let Ok(op) = Opcode::from_u8(byte) {
                let c = op.category();
                assert_ne!(c, "Unknown", "Unknown category for 0x{:02X}", byte);
            }
        }
    }

    #[test]
    fn test_v1_opcodes_unchanged() {
        assert_eq!(Opcode::Nop.to_u8(), 0x00);
        assert_eq!(Opcode::Halt.to_u8(), 0x01);
        assert_eq!(Opcode::Add.to_u8(), 0x02);
        assert_eq!(Opcode::TAdd.to_u8(), 0x10);
        assert_eq!(Opcode::TMul.to_u8(), 0x11);
        assert_eq!(Opcode::Load.to_u8(), 0x20);
        assert_eq!(Opcode::Jump.to_u8(), 0x30);
        assert_eq!(Opcode::Cmp.to_u8(), 0x40);
        assert_eq!(Opcode::And.to_u8(), 0x50);
        assert_eq!(Opcode::TPolyMul.to_u8(), 0x60);
        assert_eq!(Opcode::TAddV.to_u8(), 0x70);
        assert_eq!(Opcode::Syscall.to_u8(), 0x80);
        assert_eq!(Opcode::ReadTime.to_u8(), 0x84);
    }

    #[test]
    fn test_new_arithmetic_opcodes() {
        assert_eq!(Opcode::Abs.to_u8(), 0x08);
        assert_eq!(Opcode::AddImm.to_u8(), 0x09);
        assert_eq!(Opcode::MulAdd.to_u8(), 0x0A);
        assert_eq!(Opcode::Clamp.to_u8(), 0x0F);
    }

    #[test]
    fn test_new_atomic_opcodes() {
        assert_eq!(Opcode::Exchange.to_u8(), 0x29);
        assert_eq!(Opcode::CompareSwap.to_u8(), 0x2A);
        assert_eq!(Opcode::AtomicAdd.to_u8(), 0x2B);
        assert_eq!(Opcode::Fence.to_u8(), 0x2C);
    }

    #[test]
    fn test_new_crypto_opcodes() {
        assert_eq!(Opcode::TLatticeMul.to_u8(), 0x68);
        assert_eq!(Opcode::TKemEncaps.to_u8(), 0x6D);
        assert_eq!(Opcode::TKemDecaps.to_u8(), 0x6E);
        assert_eq!(Opcode::TZeroize.to_u8(), 0x6F);
    }

    #[test]
    fn test_new_security_opcodes() {
        assert_eq!(Opcode::AuditLog.to_u8(), 0x90);
        assert_eq!(Opcode::ConstTimeEq.to_u8(), 0x96);
        assert_eq!(Opcode::ConstTimeSel.to_u8(), 0x97);
    }

    #[test]
    fn test_privilege_requirements() {
        assert!(Opcode::Trap.requires_ring0());
        assert!(Opcode::DomainSet.requires_ring0());
        assert!(Opcode::CapGrant.requires_ring0());
        assert!(!Opcode::Add.requires_ring0());
        assert!(!Opcode::TAdd.requires_ring0());
    }

    #[test]
    fn test_ternary_classification() {
        assert!(Opcode::TAdd.is_ternary());
        assert!(Opcode::TPolyMul.is_ternary());
        assert!(Opcode::TAddV.is_ternary());
        assert!(Opcode::TritBalance.is_ternary());
        assert!(!Opcode::Add.is_ternary());
        assert!(!Opcode::And.is_ternary());
    }

    #[test]
    fn test_branch_classification() {
        assert!(Opcode::Jump.is_branch());
        assert!(Opcode::JumpGe.is_branch());
        assert!(Opcode::Loop.is_branch());
        assert!(Opcode::Halt.is_branch());
        assert!(!Opcode::Add.is_branch());
    }

    #[test]
    fn test_memory_classification() {
        assert!(Opcode::Load.is_memory_op());
        assert!(Opcode::CompareSwap.is_memory_op());
        assert!(Opcode::TZeroize.is_memory_op());
        assert!(!Opcode::Add.is_memory_op());
    }

    #[test]
    fn test_encoding_backward_compat() {
        let inst = Instruction::new(Opcode::Add, 0, 1, 2, 42);
        let encoded = inst.encode();
        let decoded = Instruction::decode(&encoded).unwrap();
        assert_eq!(decoded.opcode, Opcode::Add);
        assert_eq!(decoded.dst, 0);
        assert_eq!(decoded.src1, 1);
        assert_eq!(decoded.src2, 2);
        assert_eq!(decoded.immediate, 42);
    }

    #[test]
    fn test_new_opcodes_encode_decode() {
        let new_ops = [
            Opcode::Abs, Opcode::MulAdd, Opcode::CompareSwap, Opcode::JumpGe,
            Opcode::Select, Opcode::RotateLeft, Opcode::TLatticeMul,
            Opcode::TPermute, Opcode::AuditLog, Opcode::ConstTimeEq,
            Opcode::DebugBreak, Opcode::AssertTrit,
        ];
        for op in &new_ops {
            let inst = Instruction::new(*op, 3, 7, 11, 0);
            let encoded = inst.encode();
            let decoded = Instruction::decode(&encoded).unwrap();
            assert_eq!(decoded.opcode, *op, "Failed for {:?}", op);
        }
    }

    #[test]
    fn test_compact_encoding_new_opcodes() {
        let inst = Instruction::new(Opcode::MulAdd, 5, 10, 15, 0);
        let bytes = inst.encode_compact();
        let (decoded, consumed) = Instruction::decode_compact(&bytes).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(decoded.opcode, Opcode::MulAdd);
        assert_eq!(decoded.dst, 5);
    }

    #[test]
    fn test_ternary_encoding_new_opcodes() {
        let inst = Instruction::new(Opcode::TLatticeMul, 1, 2, 3, 0);
        let encoded = inst.encode_ternary();
        let (decoded, size) = Instruction::decode_ternary(&encoded).unwrap();
        assert_eq!(decoded.opcode, Opcode::TLatticeMul);
        assert_eq!(size, 5);
    }

    #[test]
    fn test_extended_flags() {
        let flags = VmFlags::default();
        assert!(!flags.carry);
        assert!(!flags.parity);
        assert!(flags.interrupt_enabled);
    }

    #[test]
    fn test_extended_register_file() {
        let rf = RegisterFile::default();
        assert_eq!(rf.frame_pointer, 0);
        assert_eq!(rf.link_register, 0);
        assert_eq!(rf.security_domain, 0);
        assert_eq!(rf.exception_vector, 0);
    }

    #[test]
    fn test_ring2_privilege() {
        let p = PrivilegeLevel::Ring2;
        assert_eq!(p as u8, 2);
    }

    #[test]
    fn test_program_isa_version() {
        let prog = Program::new("test");
        assert_eq!(prog.isa_version, (2, 0));
    }

    #[test]
    fn test_v2_classification() {
        assert!(Opcode::Abs.is_v2());
        assert!(Opcode::CompareSwap.is_v2());
        assert!(Opcode::AuditLog.is_v2());
        assert!(Opcode::DebugBreak.is_v2());
        assert!(!Opcode::Nop.is_v2());
        assert!(!Opcode::TAdd.is_v2());
        assert!(!Opcode::Load.is_v2());
    }
}
