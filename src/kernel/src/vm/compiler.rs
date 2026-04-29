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

use alloc::vec::Vec;
use alloc::string::String;

use super::instruction::*;
use super::{VmError, VmResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThdlOp {
    Const(i8),
    Input(u8),
    Output(u8),
    TAdd(u8, u8),
    TMul(u8, u8),
    TNeg(u8),
    TXor(u8, u8),
    TAnd(u8, u8),
    TOr(u8, u8),
    TRot(u8),
    Wire(u8, u8),
    Delay(u8, u64),
    Mux(u8, u8, u8),
}

#[derive(Debug, Clone)]
pub struct ThdlCircuit {
    pub name: String,
    pub ops: Vec<ThdlOp>,
    pub num_inputs: u8,
    pub num_outputs: u8,
}

impl ThdlCircuit {
    pub fn new(name: impl Into<String>, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name: name.into(),
            ops: Vec::new(),
            num_inputs,
            num_outputs,
        }
    }

    pub fn add_op(&mut self, op: ThdlOp) {
        self.ops.push(op);
    }
}

pub struct ThdlCompiler {
    next_reg: u8,
}

impl ThdlCompiler {
    pub fn new() -> Self {
        Self { next_reg: 0 }
    }

    fn alloc_reg(&mut self) -> VmResult<u8> {
        if self.next_reg >= 27 {
            return Err(VmError::InvalidProgram(String::from("THDL compiler: out of registers")));
        }
        let reg = self.next_reg;
        self.next_reg += 1;
        Ok(reg)
    }

    pub fn compile(&mut self, circuit: &ThdlCircuit) -> VmResult<Program> {
        self.next_reg = circuit.num_inputs;
        let mut prog = Program::new(circuit.name.clone());

        for op in &circuit.ops {
            match op {
                ThdlOp::Const(val) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::LoadImm, dst, 0, 0, *val as i64));
                }
                ThdlOp::Input(reg) => {
                    let _ = reg;
                }
                ThdlOp::Output(reg) => {
                    let _ = reg;
                }
                ThdlOp::TAdd(a, b) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TAdd, dst, *a, *b, 0));
                }
                ThdlOp::TMul(a, b) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TMul, dst, *a, *b, 0));
                }
                ThdlOp::TNeg(a) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TNeg, dst, *a, 0, 0));
                }
                ThdlOp::TXor(a, b) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TXor, dst, *a, *b, 0));
                }
                ThdlOp::TAnd(a, b) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TAnd, dst, *a, *b, 0));
                }
                ThdlOp::TOr(a, b) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TOr, dst, *a, *b, 0));
                }
                ThdlOp::TRot(a) => {
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TRot, dst, *a, 0, 0));
                }
                ThdlOp::Wire(from, to) => {
                    prog.add_instruction(Instruction::new(Opcode::Move, *to, *from, 0, 0));
                }
                ThdlOp::Delay(a, cycles) => {
                    for _ in 0..*cycles {
                        prog.add_instruction(Instruction::from_opcode(Opcode::Nop));
                    }
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::Move, dst, *a, 0, 0));
                }
                ThdlOp::Mux(sel, in0, in1) => {
                    let tmp1 = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TSub, tmp1, *in1, *in0, 0));
                    let tmp2 = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TMul, tmp2, *sel, tmp1, 0));
                    let dst = self.alloc_reg()?;
                    prog.add_instruction(Instruction::new(Opcode::TAdd, dst, tmp2, *in0, 0));
                }
            }
        }

        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        Ok(prog)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::engine::TernaryVm;
    use crate::timing::SimulatedHptp;

    #[test]
    fn test_compile_simple_add() {
        let mut circuit = ThdlCircuit::new("adder", 2, 1);
        circuit.add_op(ThdlOp::Input(0));
        circuit.add_op(ThdlOp::Input(1));
        circuit.add_op(ThdlOp::TAdd(0, 1));
        circuit.add_op(ThdlOp::Output(2));

        let mut compiler = ThdlCompiler::new();
        let prog = compiler.compile(&circuit).unwrap();
        assert!(prog.instructions.len() >= 2);

        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        vm.set_register(0, 1).unwrap();
        vm.set_register(1, 1).unwrap();
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), -1);
    }

    #[test]
    fn test_compile_negation() {
        let mut circuit = ThdlCircuit::new("inverter", 1, 1);
        circuit.add_op(ThdlOp::Input(0));
        circuit.add_op(ThdlOp::TNeg(0));

        let mut compiler = ThdlCompiler::new();
        let prog = compiler.compile(&circuit).unwrap();

        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        vm.set_register(0, 1).unwrap();
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), -1);
    }

    #[test]
    fn test_compile_mux() {
        let mut circuit = ThdlCircuit::new("mux", 3, 1);
        circuit.add_op(ThdlOp::Input(0));
        circuit.add_op(ThdlOp::Input(1));
        circuit.add_op(ThdlOp::Input(2));
        circuit.add_op(ThdlOp::Mux(0, 1, 2));

        let mut compiler = ThdlCompiler::new();
        let prog = compiler.compile(&circuit).unwrap();

        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        vm.set_register(0, 0).unwrap();
        vm.set_register(1, 1).unwrap();
        vm.set_register(2, -1).unwrap();
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(5).unwrap(), 1);
    }

    #[test]
    fn test_compile_wire() {
        let mut circuit = ThdlCircuit::new("wire", 1, 1);
        circuit.add_op(ThdlOp::Input(0));
        circuit.add_op(ThdlOp::Wire(0, 5));

        let mut compiler = ThdlCompiler::new();
        let prog = compiler.compile(&circuit).unwrap();

        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        vm.set_register(0, 42).unwrap();
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(5).unwrap(), 42);
    }

    #[test]
    fn test_compile_chain() {
        let mut circuit = ThdlCircuit::new("chain", 2, 1);
        circuit.add_op(ThdlOp::Input(0));
        circuit.add_op(ThdlOp::Input(1));
        circuit.add_op(ThdlOp::TXor(0, 1));
        circuit.add_op(ThdlOp::TOr(0, 1));

        let mut compiler = ThdlCompiler::new();
        let prog = compiler.compile(&circuit).unwrap();

        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        vm.set_register(0, 1).unwrap();
        vm.set_register(1, -1).unwrap();
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), -1);
        assert_eq!(vm.get_register(3).unwrap(), 1);
    }
}
