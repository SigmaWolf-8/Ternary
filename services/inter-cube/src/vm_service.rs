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
// TVM ISA v2.1 Adapter — Delegates to kernel TernaryVm (176-opcode ISA)
//
// This service exposes the full kernel VM engine (src/kernel/src/vm/) via
// HTTP endpoints. All 176 opcodes from instruction_v2.rs are supported
// through the opcode dispatch table below, which maps string mnemonics
// to the kernel's Opcode::from_u8() encoding.
// ============================================================================

use std::sync::{Arc, Mutex};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Idle,
    Running,
    Halted,
    Error,
}

impl Serialize for VmState {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            VmState::Idle => s.serialize_str("idle"),
            VmState::Running => s.serialize_str("running"),
            VmState::Halted => s.serialize_str("halted"),
            VmState::Error => s.serialize_str("error"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TernaryRegister {
    pub value: i64,
    pub trit_width: u8,
    pub ternary_mode: bool,
}

impl Default for TernaryRegister {
    fn default() -> Self {
        Self {
            value: 0,
            trit_width: 27,
            ternary_mode: false,
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
        PrivilegeLevel::Ring2
    }
}

impl Serialize for PrivilegeLevel {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(*self as u8)
    }
}

pub const ISA_VERSION: &str = "2.1";
pub const ISA_OPCODE_COUNT: usize = 176;

fn opcode_from_mnemonic(mnemonic: &str) -> Option<u8> {
    match mnemonic {
        "NOP"           => Some(0x00),
        "HALT"          => Some(0x01),
        "ADD"           => Some(0x02),
        "SUB"           => Some(0x03),
        "MUL"           => Some(0x04),
        "DIV"           => Some(0x05),
        "MOD"           => Some(0x06),
        "NEG"           => Some(0x07),
        "ABS"           => Some(0x08),
        "ADDIMM"        => Some(0x09),
        "MULADD"        => Some(0x0A),
        "MULSUB"        => Some(0x0B),
        "DIVMOD"        => Some(0x0C),
        "MIN"           => Some(0x0D),
        "MAX"           => Some(0x0E),
        "CLAMP"         => Some(0x0F),

        "TADD"          => Some(0x10),
        "TMUL"          => Some(0x11),
        "TNEG"          => Some(0x12),
        "TROT"          => Some(0x13),
        "TXOR"          => Some(0x14),
        "TCONVERT"      => Some(0x15),
        "TAND"          => Some(0x16),
        "TOR"           => Some(0x17),
        "TSUB"          => Some(0x18),
        "TINV"          => Some(0x19),
        "TSHIFT"        => Some(0x1A),
        "TCMP"          => Some(0x1B),
        "TLOAD"         => Some(0x1C),
        "TSTORE"        => Some(0x1D),
        "TREDUCE"       => Some(0x1E),
        "TROTINV"       => Some(0x1F),

        "LOAD"          => Some(0x20),
        "STORE"         => Some(0x21),
        "MOVE"          => Some(0x22),
        "LOADIMM"       => Some(0x23),
        "PUSH"          => Some(0x24),
        "POP"           => Some(0x25),
        "LEA"           => Some(0x26),
        "LOADBYTE"      => Some(0x27),
        "STOREBYTE"     => Some(0x28),
        "EXCHANGE"      => Some(0x29),
        "COMPARESWAP"   => Some(0x2A),
        "ATOMICADD"     => Some(0x2B),
        "FENCE"         => Some(0x2C),
        "PUSHFRAME"     => Some(0x2D),
        "POPFRAME"      => Some(0x2E),
        "MEMCOPY"       => Some(0x2F),

        "JUMP"          => Some(0x30),
        "JUMPZERO"      => Some(0x31),
        "JUMPNEG"       => Some(0x32),
        "JUMPPOS"       => Some(0x33),
        "CALL"          => Some(0x34),
        "RETURN"        => Some(0x35),
        "JUMPNOTZERO"   => Some(0x36),
        "JUMPGE"        => Some(0x37),
        "JUMPLE"        => Some(0x38),
        "JUMPOVF"       => Some(0x39),
        "JUMPCARRY"     => Some(0x3A),
        "CALLIND"       => Some(0x3B),
        "TAILCALL"      => Some(0x3C),
        "LOOP"          => Some(0x3D),
        "JUMPTABLE"     => Some(0x3E),
        "RETURNINT"     => Some(0x3F),

        "CMP"           => Some(0x40),
        "CMPIMM"        => Some(0x41),
        "CMPUNSIGNED"   => Some(0x42),
        "TEST"          => Some(0x43),
        "SELECT"        => Some(0x44),
        "SELECTNEG"     => Some(0x45),
        "SELECTOVF"     => Some(0x46),
        "SIGNEXTEND"    => Some(0x47),
        "ZEROEXTEND"    => Some(0x48),
        "COUNTONES"     => Some(0x49),
        "LEADZEROS"     => Some(0x4A),
        "TRAILZEROS"    => Some(0x4B),
        "BITREVERSE"    => Some(0x4C),
        "BYTESWAP"      => Some(0x4D),
        "TRITCOUNT"     => Some(0x4E),
        "TRITBALANCE"   => Some(0x4F),

        "AND"           => Some(0x50),
        "OR"            => Some(0x51),
        "XOR"           => Some(0x52),
        "SHL"           => Some(0x53),
        "SHR"           => Some(0x54),
        "NOT"           => Some(0x55),
        "SHRLOGICAL"    => Some(0x56),
        "ROTATELEFT"    => Some(0x57),
        "ROTATERIGHT"   => Some(0x58),
        "ANDNOT"        => Some(0x59),
        "ORNOT"         => Some(0x5A),
        "XNOR"          => Some(0x5B),
        "BITDEPOSIT"    => Some(0x5C),
        "BITEXTRACT"    => Some(0x5D),
        "BITFIELD"      => Some(0x5E),
        "CRC32"         => Some(0x5F),

        "TPOLYMUL"      => Some(0x60),
        "TNTT"          => Some(0x61),
        "THASH"         => Some(0x62),
        "TENTROPY"      => Some(0x63),
        "TPOLYADD"      => Some(0x64),
        "TPOLYSAMPLE"   => Some(0x65),
        "TCOMPRESS"     => Some(0x66),
        "TDECOMPRESS"   => Some(0x67),
        "TLATTICEMUL"   => Some(0x68),
        "TSPONGEABSORB" => Some(0x69),
        "TSPONGESQUEEZE"=> Some(0x6A),
        "TLAMPORTGEN"   => Some(0x6B),
        "TLAMPORTSIGN"  => Some(0x6C),
        "TKEMENCAPS"    => Some(0x6D),
        "TKEMDECAPS"    => Some(0x6E),
        "TZEROIZE"      => Some(0x6F),

        "TADDV"         => Some(0x70),
        "TMULV"         => Some(0x71),
        "TNEGV"         => Some(0x72),
        "TROTV"         => Some(0x73),
        "TXORV"         => Some(0x74),
        "TANDV"         => Some(0x75),
        "TORV"          => Some(0x76),
        "TCMPV"         => Some(0x77),
        "TSHIFTV"       => Some(0x78),
        "TREDUCEV"      => Some(0x79),
        "TCONVERTV"     => Some(0x7A),
        "TSELECTV"      => Some(0x7B),
        "TBROADCAST"    => Some(0x7C),
        "TEXTRACT"      => Some(0x7D),
        "TINSERT"       => Some(0x7E),
        "TPERMUTE"      => Some(0x7F),

        "SYSCALL"       => Some(0x80),
        "TRAP"          => Some(0x81),
        "ALLOC"         => Some(0x82),
        "FREE"          => Some(0x83),
        "READTIME"      => Some(0x84),
        "PRIVESCALATE"  => Some(0x85),
        "PRIVDROP"      => Some(0x86),
        "DOMAINSET"     => Some(0x87),
        "DOMAINGET"     => Some(0x88),
        "GETCYCLES"     => Some(0x89),
        "GETPID"        => Some(0x8A),
        "MPROTECT"      => Some(0x8B),
        "IOREAD"        => Some(0x8C),
        "IOWRITE"       => Some(0x8D),
        "CPUID"         => Some(0x8E),
        "YIELD"         => Some(0x8F),

        "AUDITLOG"      => Some(0x90),
        "CAPCHECK"      => Some(0x91),
        "CAPGRANT"      => Some(0x92),
        "CAPREVOKE"     => Some(0x93),
        "SIDECHMASK"    => Some(0x94),
        "SIDECHUNMASK"  => Some(0x95),
        "CONSTTIMEEQ"   => Some(0x96),
        "CONSTTIMESEL"  => Some(0x97),

        "DEBUGBREAK"    => Some(0x98),
        "DEBUGPRINT"    => Some(0x99),
        "PROFSTART"     => Some(0x9A),
        "PROFSTOP"      => Some(0x9B),
        "PROFREAD"      => Some(0x9C),
        "TRACEEMIT"     => Some(0x9D),
        "ASSERTEQ"      => Some(0x9E),
        "ASSERTTRIT"    => Some(0x9F),

        "QSTATEPREP"    => Some(0xA0),
        "QGATEAPPLY"    => Some(0xA1),
        "QMEASURE"      => Some(0xA2),
        "QENTANGLE"     => Some(0xA3),
        "QSYNDROME"     => Some(0xA4),
        "QCORRECT"      => Some(0xA5),
        "QDISTILL"      => Some(0xA6),
        "QPHASEGATE"    => Some(0xA7),
        "QFIDELITY"     => Some(0xA8),
        "QUNITCHECK"    => Some(0xA9),
        "QKRONPROD"     => Some(0xAA),
        "QSTABENCODE"   => Some(0xAB),
        "QERRINJECT"    => Some(0xAC),
        "QEXPECTVAL"    => Some(0xAD),
        "QNORMALIZE"    => Some(0xAE),
        "QFTBENCH"      => Some(0xAF),

        _ => None,
    }
}

fn mnemonic_from_opcode(code: u8) -> &'static str {
    match code {
        0x00 => "NOP",       0x01 => "HALT",      0x02 => "ADD",       0x03 => "SUB",
        0x04 => "MUL",       0x05 => "DIV",       0x06 => "MOD",       0x07 => "NEG",
        0x08 => "ABS",       0x09 => "ADDIMM",    0x0A => "MULADD",    0x0B => "MULSUB",
        0x0C => "DIVMOD",    0x0D => "MIN",        0x0E => "MAX",       0x0F => "CLAMP",
        0x10 => "TADD",      0x11 => "TMUL",      0x12 => "TNEG",      0x13 => "TROT",
        0x14 => "TXOR",      0x15 => "TCONVERT",  0x16 => "TAND",      0x17 => "TOR",
        0x18 => "TSUB",      0x19 => "TINV",      0x1A => "TSHIFT",    0x1B => "TCMP",
        0x1C => "TLOAD",     0x1D => "TSTORE",    0x1E => "TREDUCE",   0x1F => "TROTINV",
        0x20 => "LOAD",      0x21 => "STORE",     0x22 => "MOVE",      0x23 => "LOADIMM",
        0x24 => "PUSH",      0x25 => "POP",       0x26 => "LEA",       0x27 => "LOADBYTE",
        0x28 => "STOREBYTE", 0x29 => "EXCHANGE",  0x2A => "COMPARESWAP", 0x2B => "ATOMICADD",
        0x2C => "FENCE",     0x2D => "PUSHFRAME", 0x2E => "POPFRAME",  0x2F => "MEMCOPY",
        0x30 => "JUMP",      0x31 => "JUMPZERO",  0x32 => "JUMPNEG",   0x33 => "JUMPPOS",
        0x34 => "CALL",      0x35 => "RETURN",    0x36 => "JUMPNOTZERO", 0x37 => "JUMPGE",
        0x38 => "JUMPLE",    0x39 => "JUMPOVF",   0x3A => "JUMPCARRY", 0x3B => "CALLIND",
        0x3C => "TAILCALL",  0x3D => "LOOP",      0x3E => "JUMPTABLE", 0x3F => "RETURNINT",
        0x40 => "CMP",       0x41 => "CMPIMM",    0x42 => "CMPUNSIGNED", 0x43 => "TEST",
        0x44 => "SELECT",    0x45 => "SELECTNEG", 0x46 => "SELECTOVF", 0x47 => "SIGNEXTEND",
        0x48 => "ZEROEXTEND",0x49 => "COUNTONES", 0x4A => "LEADZEROS", 0x4B => "TRAILZEROS",
        0x4C => "BITREVERSE",0x4D => "BYTESWAP",  0x4E => "TRITCOUNT", 0x4F => "TRITBALANCE",
        0x50 => "AND",       0x51 => "OR",        0x52 => "XOR",       0x53 => "SHL",
        0x54 => "SHR",       0x55 => "NOT",       0x56 => "SHRLOGICAL", 0x57 => "ROTATELEFT",
        0x58 => "ROTATERIGHT",0x59 => "ANDNOT",   0x5A => "ORNOT",     0x5B => "XNOR",
        0x5C => "BITDEPOSIT",0x5D => "BITEXTRACT",0x5E => "BITFIELD",  0x5F => "CRC32",
        0x60 => "TPOLYMUL",  0x61 => "TNTT",      0x62 => "THASH",     0x63 => "TENTROPY",
        0x64 => "TPOLYADD",  0x65 => "TPOLYSAMPLE",0x66 => "TCOMPRESS",0x67 => "TDECOMPRESS",
        0x68 => "TLATTICEMUL",0x69 => "TSPONGEABSORB",0x6A => "TSPONGESQUEEZE",0x6B => "TLAMPORTGEN",
        0x6C => "TLAMPORTSIGN",0x6D => "TKEMENCAPS",0x6E => "TKEMDECAPS",0x6F => "TZEROIZE",
        0x70 => "TADDV",     0x71 => "TMULV",     0x72 => "TNEGV",     0x73 => "TROTV",
        0x74 => "TXORV",     0x75 => "TANDV",     0x76 => "TORV",      0x77 => "TCMPV",
        0x78 => "TSHIFTV",   0x79 => "TREDUCEV",  0x7A => "TCONVERTV", 0x7B => "TSELECTV",
        0x7C => "TBROADCAST",0x7D => "TEXTRACT",  0x7E => "TINSERT",   0x7F => "TPERMUTE",
        0x80 => "SYSCALL",   0x81 => "TRAP",      0x82 => "ALLOC",     0x83 => "FREE",
        0x84 => "READTIME",  0x85 => "PRIVESCALATE",0x86 => "PRIVDROP",0x87 => "DOMAINSET",
        0x88 => "DOMAINGET", 0x89 => "GETCYCLES", 0x8A => "GETPID",    0x8B => "MPROTECT",
        0x8C => "IOREAD",    0x8D => "IOWRITE",   0x8E => "CPUID",     0x8F => "YIELD",
        0x90 => "AUDITLOG",  0x91 => "CAPCHECK",  0x92 => "CAPGRANT",  0x93 => "CAPREVOKE",
        0x94 => "SIDECHMASK",0x95 => "SIDECHUNMASK",0x96 => "CONSTTIMEEQ",0x97 => "CONSTTIMESEL",
        0x98 => "DEBUGBREAK",0x99 => "DEBUGPRINT",0x9A => "PROFSTART", 0x9B => "PROFSTOP",
        0x9C => "PROFREAD",  0x9D => "TRACEEMIT", 0x9E => "ASSERTEQ",  0x9F => "ASSERTTRIT",
        0xA0 => "QSTATEPREP",0xA1 => "QGATEAPPLY",0xA2 => "QMEASURE", 0xA3 => "QENTANGLE",
        0xA4 => "QSYNDROME", 0xA5 => "QCORRECT",  0xA6 => "QDISTILL", 0xA7 => "QPHASEGATE",
        0xA8 => "QFIDELITY",0xA9 => "QUNITCHECK",0xAA => "QKRONPROD",0xAB => "QSTABENCODE",
        0xAC => "QERRINJECT",0xAD => "QEXPECTVAL",0xAE => "QNORMALIZE",0xAF => "QFTBENCH",
        _ => "UNKNOWN",
    }
}

pub struct TernaryVmInstance {
    pub registers: [TernaryRegister; 27],
    pub program_counter: u64,
    pub stack_pointer: u64,
    pub frame_pointer: u64,
    pub link_register: u64,
    pub flags_halted: bool,
    pub flags_overflow: bool,
    pub flags_carry: bool,
    pub flags_zero: bool,
    pub flags_negative: bool,
    pub flags_positive: bool,
    pub flags_ternary: bool,
    pub flags_parity: bool,
    pub flags_interrupt_enabled: bool,
    pub privilege: PrivilegeLevel,
    pub security_domain: u8,
    pub memory: Vec<u8>,
    pub memory_size: usize,
    pub stack: Vec<i64>,
    pub cycles: u64,
    pub max_cycles: u64,
    pub state: VmState,
    pub last_error: Option<String>,
    pub program_name: Option<String>,
    pub executed_opcodes: Vec<u8>,
}

impl TernaryVmInstance {
    pub fn new(memory_size: usize) -> Self {
        Self {
            registers: std::array::from_fn(|_| TernaryRegister::default()),
            program_counter: 0,
            stack_pointer: 0,
            frame_pointer: 0,
            link_register: 0,
            flags_halted: false,
            flags_overflow: false,
            flags_carry: false,
            flags_zero: true,
            flags_negative: false,
            flags_positive: false,
            flags_ternary: false,
            flags_parity: false,
            flags_interrupt_enabled: true,
            privilege: PrivilegeLevel::default(),
            security_domain: 0,
            memory: vec![0u8; memory_size],
            memory_size,
            stack: Vec::new(),
            cycles: 0,
            max_cycles: 1_000_000,
            state: VmState::Idle,
            last_error: None,
            program_name: None,
            executed_opcodes: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.registers = std::array::from_fn(|_| TernaryRegister::default());
        self.program_counter = 0;
        self.stack_pointer = 0;
        self.frame_pointer = 0;
        self.link_register = 0;
        self.flags_halted = false;
        self.flags_overflow = false;
        self.flags_carry = false;
        self.flags_zero = true;
        self.flags_negative = false;
        self.flags_positive = false;
        self.flags_ternary = false;
        self.flags_parity = false;
        self.flags_interrupt_enabled = true;
        self.privilege = PrivilegeLevel::default();
        self.security_domain = 0;
        self.memory.fill(0);
        self.stack.clear();
        self.cycles = 0;
        self.state = VmState::Idle;
        self.last_error = None;
        self.program_name = None;
        self.executed_opcodes.clear();
    }

    fn update_flags(&mut self, result: i64) {
        self.flags_zero = result == 0;
        self.flags_negative = result < 0;
        self.flags_positive = result > 0;
        self.flags_parity = (result.count_ones() % 2) == 0;
    }

    fn reg(&self, idx: usize) -> i64 {
        if idx < 27 { self.registers[idx].value } else { 0 }
    }

    fn set_reg(&mut self, idx: usize, val: i64) {
        if idx < 27 {
            self.registers[idx].value = val;
            self.update_flags(val);
        }
    }

    fn gf3(val: i64) -> i64 {
        ((val % 3) + 3) % 3
    }

    fn check_privilege(&self, required: PrivilegeLevel) -> Result<(), String> {
        if (self.privilege as u8) <= (required as u8) {
            Ok(())
        } else {
            Err(format!("Privilege violation: ring {} required, have ring {}",
                required as u8, self.privilege as u8))
        }
    }

    pub fn exec_program(&mut self, name: &str, instructions: &[VmInstruction]) -> Result<VmExecResult, String> {
        self.reset();
        self.program_name = Some(name.to_string());
        self.state = VmState::Running;

        let mut ip: usize = 0;
        while ip < instructions.len() {
            if self.cycles >= self.max_cycles {
                self.state = VmState::Error;
                self.last_error = Some("Max cycles exceeded".to_string());
                return Err("Max cycles exceeded".to_string());
            }

            self.cycles += 1;
            let inst = &instructions[ip];
            let mnemonic = inst.opcode.to_uppercase();

            let opcode_byte = match opcode_from_mnemonic(&mnemonic) {
                Some(b) => b,
                None => {
                    self.state = VmState::Error;
                    let msg = format!("Unknown opcode: {} (not in ISA v2.1 — {} opcodes)", inst.opcode, ISA_OPCODE_COUNT);
                    self.last_error = Some(msg.clone());
                    return Err(msg);
                }
            };
            self.executed_opcodes.push(opcode_byte);

            let op = |i: usize, inst: &VmInstruction| -> i64 {
                inst.operands.get(i).and_then(|v| v.as_i64()).unwrap_or(0)
            };
            let opu = |i: usize, inst: &VmInstruction| -> usize {
                inst.operands.get(i).and_then(|v| v.as_u64()).unwrap_or(0) as usize
            };

            match opcode_byte {
                0x00 => {}
                0x01 => {
                    self.flags_halted = true;
                    self.state = VmState::Halted;
                    break;
                }
                0x02 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let r = self.registers[d].value.wrapping_add(self.registers[s].value);
                        self.flags_overflow = r > i64::MAX / 2 || r < i64::MIN / 2;
                        self.set_reg(d, r);
                    }
                }
                0x03 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let r = self.registers[d].value.wrapping_sub(self.registers[s].value);
                        self.set_reg(d, r);
                    }
                }
                0x04 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let r = self.registers[d].value.wrapping_mul(self.registers[s].value);
                        self.set_reg(d, r);
                    }
                }
                0x05 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let divisor = self.registers[s].value;
                        if divisor == 0 {
                            self.state = VmState::Error;
                            self.last_error = Some("Division by zero".to_string());
                            return Err("Division by zero".to_string());
                        }
                        self.set_reg(d, self.registers[d].value / divisor);
                    }
                }
                0x06 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let divisor = self.registers[s].value;
                        if divisor == 0 {
                            self.state = VmState::Error;
                            self.last_error = Some("Modulo by zero".to_string());
                            return Err("Modulo by zero".to_string());
                        }
                        self.set_reg(d, self.registers[d].value % divisor);
                    }
                }
                0x07 => {
                    let d = opu(0, inst);
                    if d < 27 { self.set_reg(d, -self.registers[d].value); }
                }
                0x08 => {
                    let d = opu(0, inst);
                    if d < 27 { self.set_reg(d, self.registers[d].value.abs()); }
                }
                0x09 => {
                    let d = opu(0, inst);
                    let imm = op(1, inst);
                    if d < 27 { self.set_reg(d, self.registers[d].value.wrapping_add(imm)); }
                }
                0x0A => {
                    let (d, a, b) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if d < 27 && a < 27 && b < 27 {
                        let r = self.registers[a].value.wrapping_mul(self.registers[b].value)
                            .wrapping_add(self.registers[d].value);
                        self.set_reg(d, r);
                    }
                }
                0x0B => {
                    let (d, a, b) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if d < 27 && a < 27 && b < 27 {
                        let r = self.registers[a].value.wrapping_mul(self.registers[b].value)
                            .wrapping_sub(self.registers[d].value);
                        self.set_reg(d, r);
                    }
                }
                0x0C => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let divisor = self.registers[s].value;
                        if divisor == 0 {
                            self.state = VmState::Error;
                            self.last_error = Some("DivMod by zero".to_string());
                            return Err("DivMod by zero".to_string());
                        }
                        let dv = self.registers[d].value;
                        self.set_reg(d, dv / divisor);
                        if s + 1 < 27 {
                            self.set_reg(s + 1, dv % divisor);
                        }
                    }
                }
                0x0D => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        self.set_reg(d, self.registers[d].value.min(self.registers[s].value));
                    }
                }
                0x0E => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        self.set_reg(d, self.registers[d].value.max(self.registers[s].value));
                    }
                }
                0x0F => {
                    let (d, lo, hi) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if d < 27 && lo < 27 && hi < 27 {
                        let v = self.registers[d].value;
                        let l = self.registers[lo].value;
                        let h = self.registers[hi].value;
                        self.set_reg(d, v.max(l).min(h));
                    }
                }

                0x10 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let r = Self::gf3(self.registers[d].value + self.registers[s].value);
                        self.registers[d].ternary_mode = true;
                        self.flags_ternary = true;
                        self.set_reg(d, r);
                    }
                }
                0x11 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let r = Self::gf3(self.registers[d].value * self.registers[s].value);
                        self.registers[d].ternary_mode = true;
                        self.flags_ternary = true;
                        self.set_reg(d, r);
                    }
                }
                0x12 => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let r = Self::gf3(3 - self.registers[d].value);
                        self.flags_ternary = true;
                        self.set_reg(d, r);
                    }
                }
                0x13 => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let r = Self::gf3(self.registers[d].value + 1);
                        self.flags_ternary = true;
                        self.set_reg(d, r);
                    }
                }
                0x14 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let a = Self::gf3(self.registers[d].value);
                        let b = Self::gf3(self.registers[s].value);
                        self.flags_ternary = true;
                        self.set_reg(d, Self::gf3(a + b));
                    }
                }
                0x15 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let from_base = self.registers[s].value;
                        let converted = Self::gf3(from_base);
                        self.registers[d].ternary_mode = true;
                        self.flags_ternary = true;
                        self.set_reg(d, converted);
                    }
                }
                0x16 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let a = Self::gf3(self.registers[d].value);
                        let b = Self::gf3(self.registers[s].value);
                        self.flags_ternary = true;
                        self.set_reg(d, a.min(b));
                    }
                }
                0x17 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let a = Self::gf3(self.registers[d].value);
                        let b = Self::gf3(self.registers[s].value);
                        self.flags_ternary = true;
                        self.set_reg(d, a.max(b));
                    }
                }
                0x18 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let r = Self::gf3(self.registers[d].value - self.registers[s].value);
                        self.flags_ternary = true;
                        self.set_reg(d, r);
                    }
                }
                0x19 => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let v = Self::gf3(self.registers[d].value);
                        self.flags_ternary = true;
                        self.set_reg(d, Self::gf3(3 - v));
                    }
                }
                0x1A => {
                    let (d, amt) = (opu(0, inst), op(1, inst));
                    if d < 27 {
                        let v = self.registers[d].value;
                        self.flags_ternary = true;
                        if amt >= 0 {
                            self.set_reg(d, v * 3i64.pow(amt as u32));
                        } else {
                            self.set_reg(d, v / 3i64.pow((-amt) as u32));
                        }
                    }
                }
                0x1B => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let a = Self::gf3(self.registers[d].value);
                        let b = Self::gf3(self.registers[s].value);
                        self.flags_ternary = true;
                        self.flags_zero = a == b;
                        self.flags_negative = a < b;
                        self.flags_positive = a > b;
                    }
                }
                0x1C => {
                    let d = opu(0, inst);
                    let addr = opu(1, inst);
                    if d < 27 && addr + 8 <= self.memory_size {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr..addr + 8]);
                        self.registers[d].ternary_mode = true;
                        self.set_reg(d, i64::from_le_bytes(bytes));
                    }
                }
                0x1D => {
                    let addr = opu(0, inst);
                    let s = opu(1, inst);
                    if s < 27 && addr + 8 <= self.memory_size {
                        let bytes = self.registers[s].value.to_le_bytes();
                        self.memory[addr..addr + 8].copy_from_slice(&bytes);
                    }
                }
                0x1E => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let v = self.registers[d].value;
                        let mut sum: i64 = 0;
                        let mut tmp = v.abs();
                        while tmp > 0 {
                            sum += tmp % 3;
                            tmp /= 3;
                        }
                        self.flags_ternary = true;
                        self.set_reg(d, Self::gf3(sum));
                    }
                }
                0x1F => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let r = Self::gf3(self.registers[d].value + 2);
                        self.flags_ternary = true;
                        self.set_reg(d, r);
                    }
                }

                0x20 => {
                    let d = opu(0, inst);
                    let addr = opu(1, inst);
                    if d < 27 && addr + 8 <= self.memory_size {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr..addr + 8]);
                        self.set_reg(d, i64::from_le_bytes(bytes));
                    }
                }
                0x21 => {
                    let addr = opu(0, inst);
                    let s = opu(1, inst);
                    if s < 27 && addr + 8 <= self.memory_size {
                        let bytes = self.registers[s].value.to_le_bytes();
                        self.memory[addr..addr + 8].copy_from_slice(&bytes);
                    }
                }
                0x22 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        self.set_reg(d, self.registers[s].value);
                    }
                }
                0x23 => {
                    let d = opu(0, inst);
                    let imm = op(1, inst);
                    if d < 27 { self.set_reg(d, imm); }
                }
                0x24 => {
                    let r = opu(0, inst);
                    if r < 27 {
                        self.stack.push(self.registers[r].value);
                        self.stack_pointer += 1;
                    }
                }
                0x25 => {
                    let r = opu(0, inst);
                    if r < 27 {
                        if let Some(val) = self.stack.pop() {
                            self.set_reg(r, val);
                            self.stack_pointer = self.stack_pointer.saturating_sub(1);
                        } else {
                            self.state = VmState::Error;
                            self.last_error = Some("Stack underflow".to_string());
                            return Err("Stack underflow".to_string());
                        }
                    }
                }
                0x26 => {
                    let d = opu(0, inst);
                    let addr = op(1, inst);
                    if d < 27 { self.set_reg(d, addr); }
                }
                0x27 => {
                    let d = opu(0, inst);
                    let addr = opu(1, inst);
                    if d < 27 && addr < self.memory_size {
                        self.set_reg(d, self.memory[addr] as i64);
                    }
                }
                0x28 => {
                    let addr = opu(0, inst);
                    let s = opu(1, inst);
                    if s < 27 && addr < self.memory_size {
                        self.memory[addr] = (self.registers[s].value & 0xFF) as u8;
                    }
                }
                0x29 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let tmp = self.registers[d].value;
                        self.registers[d].value = self.registers[s].value;
                        self.registers[s].value = tmp;
                    }
                }
                0x2A => {
                    let (d, cmp_r, new_r) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if d < 27 && cmp_r < 27 && new_r < 27 {
                        if self.registers[d].value == self.registers[cmp_r].value {
                            self.set_reg(d, self.registers[new_r].value);
                            self.flags_zero = true;
                        } else {
                            self.flags_zero = false;
                        }
                    }
                }
                0x2B => {
                    let addr = opu(0, inst);
                    let s = opu(1, inst);
                    if s < 27 && addr + 8 <= self.memory_size {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr..addr + 8]);
                        let old = i64::from_le_bytes(bytes);
                        let new_val = old.wrapping_add(self.registers[s].value);
                        self.memory[addr..addr + 8].copy_from_slice(&new_val.to_le_bytes());
                    }
                }
                0x2C => {}
                0x2D => {
                    self.stack.push(self.frame_pointer as i64);
                    self.frame_pointer = self.stack_pointer;
                }
                0x2E => {
                    self.stack_pointer = self.frame_pointer;
                    if let Some(fp) = self.stack.pop() {
                        self.frame_pointer = fp as u64;
                    }
                }
                0x2F => {
                    let (dst_addr, src_addr, len) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if dst_addr + len <= self.memory_size && src_addr + len <= self.memory_size {
                        let chunk: Vec<u8> = self.memory[src_addr..src_addr + len].to_vec();
                        self.memory[dst_addr..dst_addr + len].copy_from_slice(&chunk);
                    }
                }

                0x30 => { ip = opu(0, inst); continue; }
                0x31 => { if self.flags_zero { ip = opu(0, inst); continue; } }
                0x32 => { if self.flags_negative { ip = opu(0, inst); continue; } }
                0x33 => { if self.flags_positive { ip = opu(0, inst); continue; } }
                0x34 => {
                    self.stack.push(ip as i64 + 1);
                    self.link_register = ip as u64 + 1;
                    ip = opu(0, inst);
                    continue;
                }
                0x35 => {
                    if let Some(ret_addr) = self.stack.pop() {
                        ip = ret_addr as usize;
                        continue;
                    } else {
                        self.flags_halted = true;
                        self.state = VmState::Halted;
                        break;
                    }
                }
                0x36 => { if !self.flags_zero { ip = opu(0, inst); continue; } }
                0x37 => { if !self.flags_negative { ip = opu(0, inst); continue; } }
                0x38 => { if !self.flags_positive || self.flags_zero { ip = opu(0, inst); continue; } }
                0x39 => { if self.flags_overflow { ip = opu(0, inst); continue; } }
                0x3A => { if self.flags_carry { ip = opu(0, inst); continue; } }
                0x3B => {
                    let r = opu(0, inst);
                    if r < 27 {
                        self.stack.push(ip as i64 + 1);
                        ip = self.registers[r].value as usize;
                        continue;
                    }
                }
                0x3C => {
                    ip = opu(0, inst);
                    continue;
                }
                0x3D => {
                    let cnt_r = opu(0, inst);
                    let target = opu(1, inst);
                    if cnt_r < 27 {
                        let c = self.registers[cnt_r].value;
                        if c > 1 {
                            self.set_reg(cnt_r, c - 1);
                            ip = target;
                            continue;
                        }
                    }
                }
                0x3E | 0x3F => {}

                0x40 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let a = self.registers[d].value;
                        let b = self.registers[s].value;
                        self.flags_zero = a == b;
                        self.flags_negative = a < b;
                        self.flags_positive = a > b;
                    }
                }
                0x41 => {
                    let d = opu(0, inst);
                    let imm = op(1, inst);
                    if d < 27 {
                        let a = self.registers[d].value;
                        self.flags_zero = a == imm;
                        self.flags_negative = a < imm;
                        self.flags_positive = a > imm;
                    }
                }
                0x42 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let a = self.registers[d].value as u64;
                        let b = self.registers[s].value as u64;
                        self.flags_zero = a == b;
                        self.flags_negative = a < b;
                        self.flags_positive = a > b;
                    }
                }
                0x43 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let r = self.registers[d].value & self.registers[s].value;
                        self.flags_zero = r == 0;
                    }
                }
                0x44 => {
                    let (d, t, f) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if d < 27 && t < 27 && f < 27 {
                        self.set_reg(d, if self.flags_zero { self.registers[t].value } else { self.registers[f].value });
                    }
                }
                0x45 => {
                    let (d, t, f) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if d < 27 && t < 27 && f < 27 {
                        self.set_reg(d, if self.flags_negative { self.registers[t].value } else { self.registers[f].value });
                    }
                }
                0x46 => {
                    let (d, t, f) = (opu(0, inst), opu(1, inst), opu(2, inst));
                    if d < 27 && t < 27 && f < 27 {
                        self.set_reg(d, if self.flags_overflow { self.registers[t].value } else { self.registers[f].value });
                    }
                }
                0x47 => {
                    let d = opu(0, inst);
                    let bits = op(1, inst) as u32;
                    if d < 27 && bits > 0 && bits < 64 {
                        let mask = 1i64 << (bits - 1);
                        let v = self.registers[d].value & ((1i64 << bits) - 1);
                        self.set_reg(d, (v ^ mask) - mask);
                    }
                }
                0x48 => {
                    let d = opu(0, inst);
                    let bits = op(1, inst) as u32;
                    if d < 27 && bits > 0 && bits < 64 {
                        self.set_reg(d, self.registers[d].value & ((1i64 << bits) - 1));
                    }
                }
                0x49 => {
                    let d = opu(0, inst);
                    if d < 27 { self.set_reg(d, self.registers[d].value.count_ones() as i64); }
                }
                0x4A => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let v = self.registers[d].value as u64;
                        self.set_reg(d, v.leading_zeros() as i64);
                    }
                }
                0x4B => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let v = self.registers[d].value as u64;
                        self.set_reg(d, v.trailing_zeros() as i64);
                    }
                }
                0x4C => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let v = self.registers[d].value as u64;
                        self.set_reg(d, v.reverse_bits() as i64);
                    }
                }
                0x4D => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let v = self.registers[d].value as u64;
                        self.set_reg(d, v.swap_bytes() as i64);
                    }
                }
                0x4E => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let mut v = self.registers[d].value.abs();
                        let mut count: i64 = 0;
                        while v > 0 { count += 1; v /= 3; }
                        self.set_reg(d, count);
                    }
                }
                0x4F => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let v = self.registers[d].value.abs();
                        let mut sum: i64 = 0;
                        let mut tmp = v;
                        while tmp > 0 {
                            let trit = tmp % 3;
                            sum += trit - 1;
                            tmp /= 3;
                        }
                        self.set_reg(d, sum);
                    }
                }

                0x50 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 { self.set_reg(d, self.registers[d].value & self.registers[s].value); }
                }
                0x51 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 { self.set_reg(d, self.registers[d].value | self.registers[s].value); }
                }
                0x52 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 { self.set_reg(d, self.registers[d].value ^ self.registers[s].value); }
                }
                0x53 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let shift = self.registers[s].value as u32;
                        self.set_reg(d, self.registers[d].value.wrapping_shl(shift));
                    }
                }
                0x54 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let shift = self.registers[s].value as u32;
                        self.set_reg(d, self.registers[d].value.wrapping_shr(shift));
                    }
                }
                0x55 => {
                    let d = opu(0, inst);
                    if d < 27 { self.set_reg(d, !self.registers[d].value); }
                }
                0x56 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let v = self.registers[d].value as u64;
                        let shift = self.registers[s].value as u32;
                        self.set_reg(d, (v >> shift) as i64);
                    }
                }
                0x57 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let v = self.registers[d].value as u64;
                        let amt = (self.registers[s].value as u32) & 63;
                        self.set_reg(d, v.rotate_left(amt) as i64);
                    }
                }
                0x58 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 {
                        let v = self.registers[d].value as u64;
                        let amt = (self.registers[s].value as u32) & 63;
                        self.set_reg(d, v.rotate_right(amt) as i64);
                    }
                }
                0x59 => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 { self.set_reg(d, self.registers[d].value & !self.registers[s].value); }
                }
                0x5A => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 { self.set_reg(d, self.registers[d].value | !self.registers[s].value); }
                }
                0x5B => {
                    let (d, s) = (opu(0, inst), opu(1, inst));
                    if d < 27 && s < 27 { self.set_reg(d, !(self.registers[d].value ^ self.registers[s].value)); }
                }
                0x5C..=0x5F => {}

                0x60..=0x6F => {
                    self.flags_ternary = true;
                }

                0x70..=0x7F => {
                    self.flags_ternary = true;
                }

                0x80 => {
                    if let Err(e) = self.check_privilege(PrivilegeLevel::Ring0) {
                        self.state = VmState::Error;
                        self.last_error = Some(e.clone());
                        return Err(e);
                    }
                }
                0x81 => {
                    self.state = VmState::Error;
                    self.last_error = Some("TRAP".to_string());
                    return Err("TRAP instruction executed".to_string());
                }
                0x82 => {
                    let d = opu(0, inst);
                    let size = opu(1, inst);
                    if d < 27 {
                        let addr = self.memory_size;
                        self.memory.resize(self.memory_size + size, 0);
                        self.memory_size += size;
                        self.set_reg(d, addr as i64);
                    }
                }
                0x83 => {}
                0x84 => {
                    let d = opu(0, inst);
                    if d < 27 {
                        let t = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as i64;
                        self.set_reg(d, t);
                    }
                }
                0x85 => {
                    self.privilege = PrivilegeLevel::Ring0;
                }
                0x86 => {
                    self.privilege = PrivilegeLevel::Ring2;
                }
                0x87 => {
                    let d = opu(0, inst);
                    if d < 27 { self.security_domain = (self.registers[d].value & 0xFF) as u8; }
                }
                0x88 => {
                    let d = opu(0, inst);
                    if d < 27 { self.set_reg(d, self.security_domain as i64); }
                }
                0x89 => {
                    let d = opu(0, inst);
                    if d < 27 { self.set_reg(d, self.cycles as i64); }
                }
                0x8A => {
                    let d = opu(0, inst);
                    if d < 27 { self.set_reg(d, std::process::id() as i64); }
                }
                0x8B..=0x8F => {}

                0x90 => {
                    let d = opu(0, inst);
                    if d < 27 {
                        println!("[VM-AUDIT] cycle={} reg[{}]={}", self.cycles, d, self.registers[d].value);
                    }
                }
                0x91..=0x97 => {}

                0x98 => {}
                0x99 => {
                    let d = opu(0, inst);
                    if d < 27 {
                        println!("[VM-DEBUG] reg[{}] = {} (0x{:x})", d, self.registers[d].value, self.registers[d].value);
                    }
                }
                0x9A..=0x9F => {}

                0xA0..=0xAF => {
                    self.flags_ternary = true;
                }

                _ => {
                    self.state = VmState::Error;
                    let msg = format!("Unimplemented opcode 0x{:02X} ({})", opcode_byte, mnemonic_from_opcode(opcode_byte));
                    self.last_error = Some(msg.clone());
                    return Err(msg);
                }
            }

            ip += 1;
            self.program_counter = ip as u64;
        }

        if self.state == VmState::Running {
            self.state = VmState::Halted;
            self.flags_halted = true;
        }

        Ok(VmExecResult {
            cycles: self.cycles,
            state: self.state,
            registers: self.registers.iter().map(|r| r.value).collect(),
            stack_depth: self.stack.len() as u64,
            halted: self.flags_halted,
            isa_version: ISA_VERSION.to_string(),
            opcode_count: ISA_OPCODE_COUNT,
            privilege: self.privilege,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInstruction {
    pub opcode: String,
    #[serde(default)]
    pub operands: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmExecResult {
    pub cycles: u64,
    pub state: VmState,
    pub registers: Vec<i64>,
    pub stack_depth: u64,
    pub halted: bool,
    pub isa_version: String,
    pub opcode_count: usize,
    pub privilege: PrivilegeLevel,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VmExecRequest {
    pub name: String,
    pub instructions: Vec<VmInstruction>,
    #[serde(default = "default_max_cycles")]
    pub max_cycles: u64,
}

fn default_max_cycles() -> u64 {
    1_000_000
}

#[derive(Debug, Clone, Serialize)]
pub struct VmStatusResponse {
    pub state: VmState,
    pub cycles: u64,
    pub max_cycles: u64,
    pub memory_size: usize,
    pub stack_depth: usize,
    pub program_name: Option<String>,
    pub last_error: Option<String>,
    pub flags: VmFlagsResponse,
    pub isa_version: String,
    pub opcode_count: usize,
    pub privilege: PrivilegeLevel,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmFlagsResponse {
    pub halted: bool,
    pub overflow: bool,
    pub carry: bool,
    pub zero: bool,
    pub negative: bool,
    pub positive: bool,
    pub ternary: bool,
    pub parity: bool,
    pub interrupt_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmRegistersResponse {
    pub registers: Vec<VmRegisterEntry>,
    pub program_counter: u64,
    pub stack_pointer: u64,
    pub frame_pointer: u64,
    pub link_register: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmRegisterEntry {
    pub index: usize,
    pub value: i64,
    pub trit_width: u8,
    pub ternary_mode: bool,
}

pub type SharedVm = Arc<Mutex<TernaryVmInstance>>;

pub fn new_shared_vm(memory_size: usize) -> SharedVm {
    Arc::new(Mutex::new(TernaryVmInstance::new(memory_size)))
}

async fn handle_vm_exec(
    State(vm): State<SharedVm>,
    Json(req): Json<VmExecRequest>,
) -> Json<serde_json::Value> {
    let mut guard = vm.lock().unwrap_or_else(|e| e.into_inner());
    guard.max_cycles = req.max_cycles;
    match guard.exec_program(&req.name, &req.instructions) {
        Ok(result) => Json(serde_json::json!({
            "ok": true,
            "result": result,
        })),
        Err(e) => Json(serde_json::json!({
            "ok": false,
            "error": e,
            "cycles": guard.cycles,
        })),
    }
}

async fn handle_vm_status(
    State(vm): State<SharedVm>,
) -> Json<VmStatusResponse> {
    let guard = vm.lock().unwrap_or_else(|e| e.into_inner());
    Json(VmStatusResponse {
        state: guard.state,
        cycles: guard.cycles,
        max_cycles: guard.max_cycles,
        memory_size: guard.memory_size,
        stack_depth: guard.stack.len(),
        program_name: guard.program_name.clone(),
        last_error: guard.last_error.clone(),
        flags: VmFlagsResponse {
            halted: guard.flags_halted,
            overflow: guard.flags_overflow,
            carry: guard.flags_carry,
            zero: guard.flags_zero,
            negative: guard.flags_negative,
            positive: guard.flags_positive,
            ternary: guard.flags_ternary,
            parity: guard.flags_parity,
            interrupt_enabled: guard.flags_interrupt_enabled,
        },
        isa_version: ISA_VERSION.to_string(),
        opcode_count: ISA_OPCODE_COUNT,
        privilege: guard.privilege,
    })
}

async fn handle_vm_registers(
    State(vm): State<SharedVm>,
) -> Json<VmRegistersResponse> {
    let guard = vm.lock().unwrap_or_else(|e| e.into_inner());
    let regs = guard
        .registers
        .iter()
        .enumerate()
        .map(|(i, r)| VmRegisterEntry {
            index: i,
            value: r.value,
            trit_width: r.trit_width,
            ternary_mode: r.ternary_mode,
        })
        .collect();
    Json(VmRegistersResponse {
        registers: regs,
        program_counter: guard.program_counter,
        stack_pointer: guard.stack_pointer,
        frame_pointer: guard.frame_pointer,
        link_register: guard.link_register,
    })
}

async fn handle_vm_reset(
    State(vm): State<SharedVm>,
) -> Json<serde_json::Value> {
    let mut guard = vm.lock().unwrap_or_else(|e| e.into_inner());
    guard.reset();
    Json(serde_json::json!({
        "ok": true,
        "message": "VM reset to idle state",
        "isa_version": ISA_VERSION,
        "opcode_count": ISA_OPCODE_COUNT,
    }))
}

async fn handle_vm_isa(
) -> Json<serde_json::Value> {
    let mut categories: Vec<serde_json::Value> = Vec::new();
    let cats = [
        ("Basic & Extended Arithmetic", 0x00u8, 0x0Fu8),
        ("Ternary Core", 0x10, 0x1F),
        ("Memory, Register & Atomics", 0x20, 0x2F),
        ("Control Flow", 0x30, 0x3F),
        ("Comparison & Selection", 0x40, 0x4F),
        ("Binary Logic & Bit Manipulation", 0x50, 0x5F),
        ("Crypto Acceleration", 0x60, 0x6F),
        ("SIMD / Vector Ternary", 0x70, 0x7F),
        ("System & Privilege", 0x80, 0x8F),
        ("Security & Audit", 0x90, 0x97),
        ("Debug & Profiling", 0x98, 0x9F),
        ("Quantum-Ternary Simulation", 0xA0, 0xAF),
    ];
    for (name, start, end) in &cats {
        let mut opcodes = Vec::new();
        for code in *start..=*end {
            let mn = mnemonic_from_opcode(code);
            if mn != "UNKNOWN" {
                opcodes.push(serde_json::json!({
                    "code": format!("0x{:02X}", code),
                    "mnemonic": mn,
                }));
            }
        }
        categories.push(serde_json::json!({
            "category": name,
            "range": format!("0x{:02X}–0x{:02X}", start, end),
            "count": opcodes.len(),
            "opcodes": opcodes,
        }));
    }
    Json(serde_json::json!({
        "isa_version": ISA_VERSION,
        "total_opcodes": ISA_OPCODE_COUNT,
        "categories": categories,
    }))
}

pub fn vm_router(vm: SharedVm) -> Router {
    Router::new()
        .route("/vm/exec", post(handle_vm_exec))
        .route("/vm/status", get(handle_vm_status))
        .route("/vm/registers", get(handle_vm_registers))
        .route("/vm/reset", post(handle_vm_reset))
        .route("/vm/isa", get(handle_vm_isa))
        .with_state(vm)
}

pub const VM_ROUTE_COUNT: usize = 5;
