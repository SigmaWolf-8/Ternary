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

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use super::instruction::*;
use super::{VmError, VmResult};

pub struct Assembler;

impl Assembler {
    pub fn assemble(source: &str) -> VmResult<Program> {
        let mut labels: BTreeMap<String, u64> = BTreeMap::new();
        let mut instructions: Vec<Instruction> = Vec::new();
        let mut pending_labels: Vec<(usize, String)> = Vec::new();

        let mut pc: u64 = 0;
        for line in source.lines() {
            let line = line.trim();
            let line = if let Some(idx) = line.find(';') { &line[..idx] } else { line };
            let line = line.trim();
            if line.is_empty() { continue; }

            if line.ends_with(':') {
                let label = &line[..line.len()-1];
                labels.insert(String::from(label), pc);
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }

            let mnemonic = parts[0].to_uppercase();
            let mnemonic_str: &str = &mnemonic;
            let opcode = Self::mnemonic_to_opcode(mnemonic_str)?;
            
            let mut dst: u8 = 0;
            let mut src1: u8 = 0;
            let mut src2: u8 = 0;
            let mut immediate: i64 = 0;
            let mut label_ref: Option<String> = None;

            let operands_str = if parts.len() > 1 { parts[1..].join(" ") } else { String::new() };
            let operands: Vec<&str> = operands_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

            for (i, op) in operands.iter().enumerate() {
                let op = op.trim();
                if op.starts_with('r') || op.starts_with('R') {
                    if let Ok(reg) = op[1..].parse::<u8>() {
                        match i {
                            0 => dst = reg,
                            1 => src1 = reg,
                            2 => src2 = reg,
                            _ => {}
                        }
                    }
                } else if op.starts_with('#') {
                    if let Ok(val) = op[1..].parse::<i64>() {
                        immediate = val;
                    }
                } else if op.starts_with('@') {
                    label_ref = Some(String::from(&op[1..]));
                } else if let Ok(val) = op.parse::<i64>() {
                    immediate = val;
                }
            }

            if let Some(ref label) = label_ref {
                pending_labels.push((instructions.len(), label.clone()));
            }

            instructions.push(Instruction::new(opcode, dst, src1, src2, immediate));
            pc += 1;
        }

        for (idx, label) in &pending_labels {
            if let Some(&addr) = labels.get(label) {
                instructions[*idx].immediate = addr as i64;
            } else {
                return Err(VmError::InvalidProgram(format!("Undefined label: {}", label)));
            }
        }

        let mut prog = Program::new("assembled");
        for inst in instructions {
            prog.add_instruction(inst);
        }
        Ok(prog)
    }

    fn mnemonic_to_opcode(mnemonic: &str) -> VmResult<Opcode> {
        match mnemonic {
            "NOP" => Ok(Opcode::Nop),
            "HALT" | "HLT" => Ok(Opcode::Halt),
            "ADD" => Ok(Opcode::Add),
            "SUB" => Ok(Opcode::Sub),
            "MUL" => Ok(Opcode::Mul),
            "DIV" => Ok(Opcode::Div),
            "MOD" => Ok(Opcode::Mod),
            "NEG" => Ok(Opcode::Neg),
            "TADD" => Ok(Opcode::TAdd),
            "TMUL" => Ok(Opcode::TMul),
            "TNEG" => Ok(Opcode::TNeg),
            "TROT" => Ok(Opcode::TRot),
            "TXOR" => Ok(Opcode::TXor),
            "TCONVERT" | "TCVT" => Ok(Opcode::TConvert),
            "TAND" => Ok(Opcode::TAnd),
            "TOR" => Ok(Opcode::TOr),
            "TSUB" => Ok(Opcode::TSub),
            "TINV" => Ok(Opcode::TInv),
            "TSHIFT" | "TSHL" => Ok(Opcode::TShift),
            "TCMP" => Ok(Opcode::TCmp),
            "TLOAD" | "TLD" => Ok(Opcode::TLoad),
            "TSTORE" | "TST" => Ok(Opcode::TStore),
            "TREDUCE" | "TRED" => Ok(Opcode::TReduce),
            "TROTINV" | "TROTI" => Ok(Opcode::TRotInv),
            "LOAD" | "LD" => Ok(Opcode::Load),
            "STORE" | "ST" => Ok(Opcode::Store),
            "MOVE" | "MOV" => Ok(Opcode::Move),
            "LOADIMM" | "LDI" => Ok(Opcode::LoadImm),
            "PUSH" => Ok(Opcode::Push),
            "POP" => Ok(Opcode::Pop),
            "JUMP" | "JMP" => Ok(Opcode::Jump),
            "JUMPZERO" | "JZ" => Ok(Opcode::JumpZero),
            "JUMPNEG" | "JN" => Ok(Opcode::JumpNeg),
            "JUMPPOS" | "JP" => Ok(Opcode::JumpPos),
            "JUMPNOTZERO" | "JNZ" => Ok(Opcode::JumpNotZero),
            "CALL" => Ok(Opcode::Call),
            "RETURN" | "RET" => Ok(Opcode::Return),
            "CMP" => Ok(Opcode::Cmp),
            "CMPIMM" | "CMPI" => Ok(Opcode::CmpImm),
            "AND" => Ok(Opcode::And),
            "OR" => Ok(Opcode::Or),
            "XOR" => Ok(Opcode::Xor),
            "SHL" => Ok(Opcode::Shl),
            "SHR" => Ok(Opcode::Shr),
            "NOT" => Ok(Opcode::Not),
            "TPOLYMUL" => Ok(Opcode::TPolyMul),
            "TNTT" => Ok(Opcode::TNTT),
            "THASH" => Ok(Opcode::THash),
            "TENTROPY" => Ok(Opcode::TEntropy),
            "TPOLYADD" => Ok(Opcode::TPolyAdd),
            "TPOLYSAMPLE" | "TPSAMP" => Ok(Opcode::TPolySample),
            "TCOMPRESS" | "TCOMP" => Ok(Opcode::TCompress),
            "TDECOMPRESS" | "TDCOMP" => Ok(Opcode::TDecompress),
            "TADDV" => Ok(Opcode::TAddV),
            "TMULV" => Ok(Opcode::TMulV),
            "TNEGV" => Ok(Opcode::TNegV),
            "TROTV" => Ok(Opcode::TRotV),
            "SYSCALL" | "SYS" => Ok(Opcode::Syscall),
            "TRAP" | "INT" => Ok(Opcode::Trap),
            "ALLOC" => Ok(Opcode::Alloc),
            "FREE" => Ok(Opcode::Free),
            "READTIME" | "RDTIME" => Ok(Opcode::ReadTime),
            _ => Err(VmError::InvalidProgram(format!("Unknown mnemonic: {}", mnemonic))),
        }
    }
}

pub struct Disassembler;

impl Disassembler {
    pub fn opcode_to_mnemonic(opcode: Opcode) -> &'static str {
        match opcode {
            Opcode::Nop => "NOP",
            Opcode::Halt => "HALT",
            Opcode::Add => "ADD",
            Opcode::Sub => "SUB",
            Opcode::Mul => "MUL",
            Opcode::Div => "DIV",
            Opcode::Mod => "MOD",
            Opcode::Neg => "NEG",
            Opcode::TAdd => "TADD",
            Opcode::TMul => "TMUL",
            Opcode::TNeg => "TNEG",
            Opcode::TRot => "TROT",
            Opcode::TXor => "TXOR",
            Opcode::TConvert => "TCVT",
            Opcode::TAnd => "TAND",
            Opcode::TOr => "TOR",
            Opcode::TSub => "TSUB",
            Opcode::TInv => "TINV",
            Opcode::TShift => "TSHL",
            Opcode::TCmp => "TCMP",
            Opcode::TLoad => "TLD",
            Opcode::TStore => "TST",
            Opcode::TReduce => "TRED",
            Opcode::TRotInv => "TROTI",
            Opcode::Load => "LD",
            Opcode::Store => "ST",
            Opcode::Move => "MOV",
            Opcode::LoadImm => "LDI",
            Opcode::Push => "PUSH",
            Opcode::Pop => "POP",
            Opcode::Jump => "JMP",
            Opcode::JumpZero => "JZ",
            Opcode::JumpNeg => "JN",
            Opcode::JumpPos => "JP",
            Opcode::JumpNotZero => "JNZ",
            Opcode::Call => "CALL",
            Opcode::Return => "RET",
            Opcode::Cmp => "CMP",
            Opcode::CmpImm => "CMPI",
            Opcode::And => "AND",
            Opcode::Or => "OR",
            Opcode::Xor => "XOR",
            Opcode::Shl => "SHL",
            Opcode::Shr => "SHR",
            Opcode::Not => "NOT",
            Opcode::TPolyMul => "TPOLYMUL",
            Opcode::TNTT => "TNTT",
            Opcode::THash => "THASH",
            Opcode::TEntropy => "TENTROPY",
            Opcode::TPolyAdd => "TPOLYADD",
            Opcode::TPolySample => "TPSAMP",
            Opcode::TCompress => "TCOMP",
            Opcode::TDecompress => "TDCOMP",
            Opcode::TAddV => "TADDV",
            Opcode::TMulV => "TMULV",
            Opcode::TNegV => "TNEGV",
            Opcode::TRotV => "TROTV",
            Opcode::Syscall => "SYS",
            Opcode::Trap => "TRAP",
            Opcode::Alloc => "ALLOC",
            Opcode::Free => "FREE",
            Opcode::ReadTime => "RDTIME",
        }
    }

    pub fn disassemble_instruction(inst: &Instruction) -> String {
        let mnemonic = Self::opcode_to_mnemonic(inst.opcode);
        if inst.immediate != 0 {
            format!("{} r{}, r{}, r{}, #{}", mnemonic, inst.dst, inst.src1, inst.src2, inst.immediate)
        } else if inst.dst == 0 && inst.src1 == 0 && inst.src2 == 0 {
            String::from(mnemonic)
        } else {
            format!("{} r{}, r{}, r{}", mnemonic, inst.dst, inst.src1, inst.src2)
        }
    }

    pub fn disassemble_program(prog: &Program) -> String {
        let mut output = String::new();
        output.push_str(&format!("; Program: {}\n", prog.name));
        for (i, inst) in prog.instructions.iter().enumerate() {
            output.push_str(&format!("{:04}: {}\n", i, Self::disassemble_instruction(inst)));
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_simple() {
        let source = "LDI r0, #10\nLDI r1, #20\nADD r2, r0, r1\nHALT\n";
        let prog = Assembler::assemble(source).unwrap();
        assert_eq!(prog.instructions.len(), 4);
        assert_eq!(prog.instructions[0].opcode, Opcode::LoadImm);
        assert_eq!(prog.instructions[0].dst, 0);
        assert_eq!(prog.instructions[0].immediate, 10);
        assert_eq!(prog.instructions[2].opcode, Opcode::Add);
    }

    #[test]
    fn test_assemble_with_labels() {
        let source = "LDI r0, #5\nloop:\nSUB r0, r0, r1\nJNZ @loop\nHALT\n";
        let prog = Assembler::assemble(source).unwrap();
        assert_eq!(prog.instructions[2].opcode, Opcode::JumpNotZero);
        assert_eq!(prog.instructions[2].immediate, 1);
    }

    #[test]
    fn test_assemble_with_comments() {
        let source = "; this is a comment\nNOP ; inline comment\nHALT\n";
        let prog = Assembler::assemble(source).unwrap();
        assert_eq!(prog.instructions.len(), 2);
    }

    #[test]
    fn test_assemble_ternary_ops() {
        let source = "LDI r0, #1\nLDI r1, #-1\nTADD r2, r0, r1\nTXOR r3, r0, r1\nHALT\n";
        let prog = Assembler::assemble(source).unwrap();
        assert_eq!(prog.instructions[2].opcode, Opcode::TAdd);
        assert_eq!(prog.instructions[3].opcode, Opcode::TXor);
    }

    #[test]
    fn test_disassemble_roundtrip() {
        let source = "LDI r0, r0, r0, #10\nHALT\n";
        let prog = Assembler::assemble(source).unwrap();
        let disasm = Disassembler::disassemble_program(&prog);
        assert!(disasm.contains("LDI"));
        assert!(disasm.contains("#10"));
    }

    #[test]
    fn test_disassemble_instruction() {
        let inst = Instruction::new(Opcode::TAdd, 2, 0, 1, 0);
        let text = Disassembler::disassemble_instruction(&inst);
        assert!(text.contains("TADD"));
        assert!(text.contains("r2"));
    }

    #[test]
    fn test_all_mnemonics_recognized() {
        let mnemonics = ["NOP", "HALT", "ADD", "SUB", "MUL", "DIV", "MOD", "NEG",
            "TADD", "TMUL", "TNEG", "TROT", "TXOR", "TCVT", "TAND", "TOR",
            "TSUB", "TINV", "TSHL", "TCMP", "TLD", "TST", "TRED", "TROTI",
            "LD", "ST", "MOV", "LDI", "PUSH", "POP", "JMP", "JZ", "JN", "JP",
            "JNZ", "CALL", "RET", "CMP", "CMPI", "AND", "OR", "XOR", "SHL",
            "SHR", "NOT", "TPOLYMUL", "TNTT", "THASH", "TENTROPY",
            "TPOLYADD", "TPSAMP", "TCOMP", "TDCOMP", "TADDV", "TMULV",
            "TNEGV", "TROTV", "SYS", "TRAP", "ALLOC", "FREE", "RDTIME"];
        for m in &mnemonics {
            assert!(Assembler::mnemonic_to_opcode(m).is_ok(), "Failed for: {}", m);
        }
    }
}
