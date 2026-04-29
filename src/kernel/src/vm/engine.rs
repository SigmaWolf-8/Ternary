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
use super::{VmError, VmResult};
use super::instruction::*;
use super::gc::GcHeap;
use super::cache::ConstantTimeTernary;
use crate::ternary::{Trit, KernelTritExt, Representation, convert_representation, scalar_to_trit, pack_trits, unpack_trits, packed_map, packed_zip, packed_shift_left, packed_shift_right, packed_rotate_left, packed_reduce, packed_convert};
use crate::timing::{FemtosecondTimestamp, HptpProvider, SimulatedHptp};
use alloc::boxed::Box;

pub struct VmMemory {
    data: Vec<u8>,
    size: usize,
}

impl VmMemory {
    pub fn new(size: usize) -> Self {
        Self {
            data: alloc::vec![0u8; size],
            size,
        }
    }

    pub fn read_u8(&self, addr: u64) -> VmResult<u8> {
        let addr = addr as usize;
        if addr >= self.size {
            return Err(VmError::SegmentationFault { address: addr as u64 });
        }
        Ok(self.data[addr])
    }

    pub fn write_u8(&mut self, addr: u64, val: u8) -> VmResult<()> {
        let addr = addr as usize;
        if addr >= self.size {
            return Err(VmError::SegmentationFault { address: addr as u64 });
        }
        self.data[addr] = val;
        Ok(())
    }

    pub fn read_i64(&self, addr: u64) -> VmResult<i64> {
        let addr = addr as usize;
        if addr + 8 > self.size {
            return Err(VmError::InvalidMemoryAccess {
                address: addr as u64,
                size: 8,
            });
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[addr..addr + 8]);
        Ok(i64::from_le_bytes(bytes))
    }

    pub fn write_i64(&mut self, addr: u64, val: i64) -> VmResult<()> {
        let addr = addr as usize;
        if addr + 8 > self.size {
            return Err(VmError::InvalidMemoryAccess {
                address: addr as u64,
                size: 8,
            });
        }
        self.data[addr..addr + 8].copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn read_bytes(&self, addr: u64, len: usize) -> VmResult<&[u8]> {
        let addr = addr as usize;
        if addr + len > self.size {
            return Err(VmError::InvalidMemoryAccess {
                address: addr as u64,
                size: len as u64,
            });
        }
        Ok(&self.data[addr..addr + len])
    }
}

pub struct VmStack {
    data: Vec<i64>,
    max_size: usize,
}

impl VmStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, val: i64) -> VmResult<()> {
        if self.data.len() >= self.max_size {
            return Err(VmError::StackOverflow);
        }
        self.data.push(val);
        Ok(())
    }

    pub fn pop(&mut self) -> VmResult<i64> {
        self.data.pop().ok_or(VmError::StackUnderflow)
    }

    pub fn peek(&self) -> VmResult<i64> {
        self.data.last().copied().ok_or(VmError::StackUnderflow)
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

pub struct TernaryVm {
    pub registers: RegisterFile,
    pub memory: VmMemory,
    stack: VmStack,
    gc: GcHeap,
    pub program: Option<Program>,
    cycles: u64,
    pub max_cycles: u64,
    pub process_id: u64,
    pub time_slice: u64,
    pub time_remaining: u64,
    pub security_domain: u8,
    hptp_provider: Box<dyn HptpProvider>,
}

impl TernaryVm {
    pub fn new(memory_size: usize, provider: Box<dyn HptpProvider>) -> Self {
        Self {
            registers: RegisterFile::default(),
            memory: VmMemory::new(memory_size),
            stack: VmStack::new(4096),
            gc: GcHeap::new(memory_size / 4),
            program: None,
            cycles: 0,
            max_cycles: 1_000_000,
            process_id: 0,
            time_slice: 1000,
            time_remaining: 1000,
            security_domain: 0,
            hptp_provider: provider,
        }
    }

    pub fn set_hptp_provider(&mut self, provider: Box<dyn HptpProvider>) {
        self.hptp_provider = provider;
    }

    /// Construct a VM wired to a fresh `SimulatedHptp` provider. Useful
    /// for benchmarks, fixtures, and short-lived diagnostic VMs that
    /// don't need a real femtosecond clock — keeps the simulated
    /// timing source as a documented, callable production constructor
    /// rather than an internal-only test helper.
    pub fn with_simulated_hptp(memory_size: usize) -> Self {
        Self::new(memory_size, Box::new(SimulatedHptp::new()))
    }

    pub fn current_hptp_timestamp(&self) -> FemtosecondTimestamp {
        self.hptp_provider.read_timestamp(self.cycles)
    }

    pub fn load_program(&mut self, program: Program) -> VmResult<()> {
        program.validate()?;
        self.registers.program_counter = program.entry_point;
        self.program = Some(program);
        Ok(())
    }

    pub fn step(&mut self) -> VmResult<bool> {
        if self.registers.flags.halted {
            return Ok(false);
        }

        if self.cycles >= self.max_cycles {
            return Err(VmError::InvalidProgram(alloc::string::String::from(
                "Max cycles exceeded",
            )));
        }

        let program = self
            .program
            .as_ref()
            .ok_or(VmError::InvalidProgram(alloc::string::String::from(
                "No program loaded",
            )))?;

        let pc = self.registers.program_counter;
        let inst = program
            .get_instruction(pc)
            .ok_or(VmError::ProgramCounterOutOfBounds)?
            .clone();

        self.registers.program_counter += 1;
        self.execute_instruction(&inst)?;
        self.cycles += 1;

        if self.time_remaining > 0 {
            self.time_remaining -= 1;
        }

        Ok(!self.registers.flags.halted)
    }

    pub fn run(&mut self) -> VmResult<u64> {
        loop {
            let running = self.step()?;
            if !running {
                return Ok(self.cycles);
            }
        }
    }

    pub fn execute_instruction(&mut self, inst: &Instruction) -> VmResult<()> {
        match inst.opcode {
            Opcode::Nop => {}
            Opcode::Halt => {
                self.registers.flags.halted = true;
            }
            Opcode::Add => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let (result, overflow) = a.overflowing_add(b);
                self.set_register(inst.dst, result)?;
                self.update_flags(result, overflow);
            }
            Opcode::Sub => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let (result, overflow) = a.overflowing_sub(b);
                self.set_register(inst.dst, result)?;
                self.update_flags(result, overflow);
            }
            Opcode::Mul => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let (result, overflow) = a.overflowing_mul(b);
                self.set_register(inst.dst, result)?;
                self.update_flags(result, overflow);
            }
            Opcode::Div => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                let result = a / b;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Mod => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                let result = a % b;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Neg => {
                let a = self.get_register(inst.src1)?;
                let result = -a;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TAdd => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) || self.is_ternary_mode(inst.src2) {
                    ConstantTimeTernary::ct_packed_add(a, b)
                } else {
                    ConstantTimeTernary::ct_add(a as i8, b as i8) as i64
                };
                self.set_register(inst.dst, result)?;
                self.propagate_ternary_mode(inst.dst, inst.src1, inst.src2)?;
                self.update_flags(result, false);
            }
            Opcode::TMul => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) || self.is_ternary_mode(inst.src2) {
                    ConstantTimeTernary::ct_packed_mul(a, b)
                } else {
                    ConstantTimeTernary::ct_mul(a as i8, b as i8) as i64
                };
                self.set_register(inst.dst, result)?;
                self.propagate_ternary_mode(inst.dst, inst.src1, inst.src2)?;
                self.update_flags(result, false);
            }
            Opcode::TNeg => {
                let a = self.get_register(inst.src1)?;
                let result = if self.is_ternary_mode(inst.src1) {
                    packed_map(a, |t| t.not())
                } else {
                    let ta = scalar_to_trit(a);
                    ta.not().to_a() as i64
                };
                self.set_register(inst.dst, result)?;
                self.copy_ternary_mode(inst.dst, inst.src1)?;
                self.update_flags(result, false);
            }
            Opcode::TRot => {
                let a = self.get_register(inst.src1)?;
                let positions = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) {
                    let rot_count = (positions.rem_euclid(3)) as usize;
                    let mut val = a;
                    for _ in 0..rot_count {
                        val = packed_map(val, |t| t.rotate());
                    }
                    val
                } else {
                    let ta = scalar_to_trit(a);
                    let rot_count = positions.rem_euclid(3);
                    let mut t = ta;
                    for _ in 0..rot_count {
                        t = t.rotate();
                    }
                    t.to_a() as i64
                };
                self.set_register(inst.dst, result)?;
                self.copy_ternary_mode(inst.dst, inst.src1)?;
                self.update_flags(result, false);
            }
            Opcode::TXor => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) || self.is_ternary_mode(inst.src2) {
                    packed_zip(a, b, |x, y| x.xor(y))
                } else {
                    let ta = scalar_to_trit(a);
                    let tb = scalar_to_trit(b);
                    ta.xor(&tb).to_a() as i64
                };
                self.set_register(inst.dst, result)?;
                self.propagate_ternary_mode(inst.dst, inst.src1, inst.src2)?;
                self.update_flags(result, false);
            }
            Opcode::TConvert => {
                let val = self.get_register(inst.src1)?;
                let from_repr_id = self.get_register(inst.src2)?;
                let to_repr_id = inst.immediate;
                let from_repr = match from_repr_id {
                    0 => Representation::A,
                    1 => Representation::B,
                    2 => Representation::C,
                    _ => return Err(VmError::InvalidProgram(alloc::string::String::from(
                        "Invalid source representation for TConvert",
                    ))),
                };
                let to_repr = match to_repr_id {
                    0 => Representation::A,
                    1 => Representation::B,
                    2 => Representation::C,
                    _ => return Err(VmError::InvalidProgram(alloc::string::String::from(
                        "Invalid target representation for TConvert",
                    ))),
                };
                let result = if self.is_ternary_mode(inst.src1) {
                    packed_convert(val, from_repr, to_repr)
                } else {
                    convert_representation(val as i8, from_repr, to_repr) as i64
                };
                self.set_register(inst.dst, result)?;
                self.copy_ternary_mode(inst.dst, inst.src1)?;
                self.update_flags(result, false);
            }
            Opcode::TAnd => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) || self.is_ternary_mode(inst.src2) {
                    packed_zip(a, b, |x, y| x.lukasiewicz_and(y))
                } else {
                    let ta = scalar_to_trit(a);
                    let tb = scalar_to_trit(b);
                    ta.lukasiewicz_and(&tb).to_a() as i64
                };
                self.set_register(inst.dst, result)?;
                self.propagate_ternary_mode(inst.dst, inst.src1, inst.src2)?;
                self.update_flags(result, false);
            }
            Opcode::TOr => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) || self.is_ternary_mode(inst.src2) {
                    packed_zip(a, b, |x, y| x.or(y))
                } else {
                    let ta = scalar_to_trit(a);
                    let tb = scalar_to_trit(b);
                    ta.or(&tb).to_a() as i64
                };
                self.set_register(inst.dst, result)?;
                self.propagate_ternary_mode(inst.dst, inst.src1, inst.src2)?;
                self.update_flags(result, false);
            }
            Opcode::TSub => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) || self.is_ternary_mode(inst.src2) {
                    packed_zip(a, b, |x, y| x.sub(*y))
                } else {
                    let ta = scalar_to_trit(a);
                    let tb = scalar_to_trit(b);
                    ta.sub(tb).to_a() as i64
                };
                self.set_register(inst.dst, result)?;
                self.propagate_ternary_mode(inst.dst, inst.src1, inst.src2)?;
                self.update_flags(result, false);
            }
            Opcode::TInv => {
                let a = self.get_register(inst.src1)?;
                let result = if self.is_ternary_mode(inst.src1) {
                    let trits = unpack_trits(a);
                    for i in 0..27 {
                        if trits[i].to_a() == 0 {
                            return Err(VmError::InvalidProgram(
                                alloc::string::String::from("GF(3) inverse of zero is undefined")
                            ));
                        }
                    }
                    a // nonzero trits are self-inverse in balanced representation
                } else {
                    if a == 0 {
                        return Err(VmError::InvalidProgram(
                            alloc::string::String::from("GF(3) inverse of zero is undefined")
                        ));
                    }
                    a // nonzero scalar trits are self-inverse in balanced
                };
                self.set_register(inst.dst, result)?;
                self.copy_ternary_mode(inst.dst, inst.src1)?;
                self.update_flags(result, false);
            }
            Opcode::TShift => {
                let a = self.get_register(inst.src1)?;
                let shift_amount = if inst.immediate != 0 {
                    inst.immediate
                } else {
                    self.get_register(inst.src2)?
                };
                let result = if shift_amount >= 0 {
                    packed_shift_left(a, shift_amount as usize)
                } else {
                    packed_shift_right(a, (-shift_amount) as usize)
                };
                self.set_register(inst.dst, result)?;
                self.copy_ternary_mode(inst.dst, inst.src1)?;
                self.update_flags(result, false);
            }
            Opcode::TCmp => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = if self.is_ternary_mode(inst.src1) || self.is_ternary_mode(inst.src2) {
                    packed_zip(a, b, |x, y| x.cmp_trit(y))
                } else {
                    let ta = scalar_to_trit(a);
                    let tb = scalar_to_trit(b);
                    ta.cmp_trit(&tb).to_a() as i64
                };
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TLoad => {
                let base = self.get_register(inst.src1)?;
                let addr = if self.is_ternary_mode(inst.src1) {
                    let trits = crate::ternary::unpack_trits(base);
                    let mut address: i64 = 0;
                    let mut power: i64 = 1;
                    for i in 0..27 {
                        address += trits[i].to_a() as i64 * power;
                        power *= 3;
                    }
                    (address + inst.immediate) as u64
                } else {
                    (base + inst.immediate) as u64
                };
                let val = self.memory.read_i64(addr)?;
                self.set_register(inst.dst, val)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(val, false);
            }
            Opcode::TStore => {
                let base = self.get_register(inst.src1)?;
                let addr = if self.is_ternary_mode(inst.src1) {
                    let trits = crate::ternary::unpack_trits(base);
                    let mut address: i64 = 0;
                    let mut power: i64 = 1;
                    for i in 0..27 {
                        address += trits[i].to_a() as i64 * power;
                        power *= 3;
                    }
                    (address + inst.immediate) as u64
                } else {
                    (base + inst.immediate) as u64
                };
                let val = self.get_register(inst.dst)?;
                self.memory.write_i64(addr, val)?;
                self.set_ternary_mode(inst.dst, true)?;
            }
            Opcode::Load => {
                let base = self.get_register(inst.src1)?;
                let addr = (base + inst.immediate) as u64;
                let val = self.memory.read_i64(addr)?;
                self.set_register(inst.dst, val)?;
            }
            Opcode::Store => {
                let base = self.get_register(inst.src1)?;
                let addr = (base + inst.immediate) as u64;
                let val = self.get_register(inst.dst)?;
                self.memory.write_i64(addr, val)?;
            }
            Opcode::Move => {
                let val = self.get_register(inst.src1)?;
                self.set_register(inst.dst, val)?;
            }
            Opcode::LoadImm => {
                self.set_register(inst.dst, inst.immediate)?;
            }
            Opcode::Push => {
                let val = self.get_register(inst.src1)?;
                self.stack.push(val)?;
            }
            Opcode::Pop => {
                let val = self.stack.pop()?;
                self.set_register(inst.dst, val)?;
            }
            Opcode::Jump => {
                self.registers.program_counter = inst.immediate as u64;
            }
            Opcode::JumpZero => {
                if self.registers.flags.zero {
                    self.registers.program_counter = inst.immediate as u64;
                }
            }
            Opcode::JumpNeg => {
                if self.registers.flags.negative {
                    self.registers.program_counter = inst.immediate as u64;
                }
            }
            Opcode::JumpPos => {
                if !self.registers.flags.zero && !self.registers.flags.negative {
                    self.registers.program_counter = inst.immediate as u64;
                }
            }
            Opcode::JumpNotZero => {
                if !self.registers.flags.zero {
                    self.registers.program_counter = inst.immediate as u64;
                }
            }
            Opcode::Call => {
                let return_addr = self.registers.program_counter as i64;
                self.stack.push(return_addr)?;
                self.registers.program_counter = inst.immediate as u64;
            }
            Opcode::Return => {
                let addr = self.stack.pop()?;
                self.registers.program_counter = addr as u64;
            }
            Opcode::Cmp => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = a.wrapping_sub(b);
                self.update_flags(result, false);
            }
            Opcode::CmpImm => {
                let a = self.get_register(inst.src1)?;
                let result = a.wrapping_sub(inst.immediate);
                self.update_flags(result, false);
            }
            Opcode::And => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = a & b;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Or => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = a | b;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Xor => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = a ^ b;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Shl => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = a << (b & 63);
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Shr => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = a >> (b & 63);
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Not => {
                let a = self.get_register(inst.src1)?;
                let result = !a;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TPolyMul => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let ta = unpack_trits(a);
                let tb = unpack_trits(b);
                let zero_trit = Trit::from_a(0).unwrap();
                let mut result_trits = [zero_trit; 27];
                let degree = if inst.immediate > 0 && (inst.immediate as usize) < 27 {
                    inst.immediate as usize
                } else {
                    13
                };
                for i in 0..degree {
                    for j in 0..degree {
                        let k = (i + j) % degree;
                        result_trits[k] = result_trits[k].add(ta[i].multiply(&tb[j]));
                    }
                }
                let result = pack_trits(&result_trits);
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TNTT => {
                let a = self.get_register(inst.src1)?;
                let is_inverse = inst.immediate != 0;
                let mut trits = unpack_trits(a);
                let n = 27;
                if is_inverse {
                    trits.reverse();
                    let first = trits[0];
                    for i in 0..(n - 1) {
                        trits[i] = trits[i + 1];
                    }
                    trits[n - 1] = first;
                } else {
                    let last = trits[n - 1];
                    for i in (1..n).rev() {
                        trits[i] = trits[i - 1];
                    }
                    trits[0] = last;
                    trits.reverse();
                }
                for i in 0..n {
                    for j in (i + 1)..n {
                        let sum = trits[i].add(trits[j]);
                        let diff = trits[i].sub(trits[j]);
                        trits[i] = sum;
                        trits[j] = diff;
                    }
                }
                let result = pack_trits(&trits);
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::THash => {
                let a = self.get_register(inst.src1)?;
                let trits = unpack_trits(a);
                let seed = super::constants::HASH_SEED;
                let mix = super::constants::HASH_MIX;
                let rounds = super::constants::HASH_ROUNDS;
                let mut state = seed;
                for trit in &trits {
                    let t_val = (trit.to_a() as i64 + 1) as u64;
                    state ^= t_val.wrapping_mul(mix);
                    state = state.wrapping_mul(0x517cc1b727220a95);
                    state ^= state >> 28;
                }
                for _ in 0..rounds {
                    state ^= state >> 17;
                    state = state.wrapping_mul(0xbf58476d1ce4e5b9);
                    state ^= state >> 31;
                }
                let result = state as i64;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TEntropy => {
                let seed1 = self.get_register(inst.src1)?;
                let seed2 = self.get_register(inst.src2)?;
                let mut state = (seed1 as u64).wrapping_add(super::constants::HASH_SEED);
                state ^= (seed2 as u64).wrapping_mul(super::constants::HASH_MIX);
                state = state.wrapping_mul(0x517cc1b727220a95);
                state ^= state >> 28;
                state ^= self.cycles.wrapping_mul(0xd6e8feb86659fd93);
                state ^= state >> 32;
                let zero_trit = Trit::from_a(0).unwrap();
                let mut result_trits = [zero_trit; 27];
                for i in 0..27 {
                    let bits = (state >> (i * 2)) & 0b11;
                    result_trits[i] = Trit::from_a(match bits % 3 {
                        0 => 0,
                        1 => 1,
                        _ => -1,
                    }).unwrap();
                }
                let result = pack_trits(&result_trits);
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TReduce => {
                let a = self.get_register(inst.src1)?;
                let gate = (inst.immediate & 0x03) as u8;
                let result_trit = packed_reduce(a, gate);
                let result = result_trit.to_a() as i64;
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TRotInv => {
                let a = self.get_register(inst.src1)?;
                let result = packed_map(a, |t| t.rotate_inverse());
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TPolyAdd => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = packed_zip(a, b, |x, y| x.add(*y));
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TPolySample => {
                let seed = self.get_register(inst.src1)?;
                let mut state = (seed as u64).wrapping_mul(0x517cc1b727220a95);
                state ^= state >> 28;
                state = state.wrapping_add(self.cycles.wrapping_mul(0xd6e8feb86659fd93));
                state ^= state >> 32;
                let zero_trit = Trit::from_a(0).unwrap();
                let mut result_trits = [zero_trit; 27];
                for i in 0..27 {
                    let bits = (state >> (i * 2)) & 0b11;
                    result_trits[i] = Trit::from_a(match bits % 3 {
                        0 => 0,
                        1 => 1,
                        _ => -1,
                    }).unwrap();
                }
                let result = pack_trits(&result_trits);
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TCompress => {
                let a = self.get_register(inst.src1)?;
                let trits = unpack_trits(a);
                let zero_trit = Trit::from_a(0).unwrap();
                let mut compressed = [zero_trit; 27];
                let mut count = 0usize;
                for i in 0..27 {
                    if trits[i].to_a() != 0 {
                        compressed[count] = trits[i];
                        count += 1;
                    }
                }
                let result = pack_trits(&compressed);
                self.set_register(inst.dst, result)?;
                self.set_register(inst.src2, count as i64)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TDecompress => {
                let a = self.get_register(inst.src1)?;
                let count = self.get_register(inst.src2)? as usize;
                let compressed = unpack_trits(a);
                let zero_trit = Trit::from_a(0).unwrap();
                let mut expanded = [zero_trit; 27];
                let safe_count = if count > 27 { 27 } else { count };
                for i in 0..safe_count {
                    expanded[i] = compressed[i];
                }
                let result = pack_trits(&expanded);
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::Syscall => {
                let syscall_num = self.get_register(inst.src1)?;
                let arg1 = self.get_register(inst.src2)?;
                let result = match syscall_num {
                    0 => 0i64,
                    1 => self.cycles as i64,
                    2 => 27i64,
                    3 => self.memory.size as i64,
                    4 => self.security_domain as i64,
                    _ => return Err(VmError::InvalidProgram(alloc::string::String::from("Unknown syscall"))),
                };
                let _ = arg1;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::Trap => {
                self.check_privilege(PrivilegeLevel::Ring0)?;
                let trap_code = inst.immediate as u32;
                return Err(VmError::InvalidProgram(alloc::format!("Trap #{}", trap_code)));
            }
            Opcode::Alloc => {
                let size = self.get_register(inst.src1)? as usize;
                let obj_type = match inst.immediate {
                    0 => super::gc::GcObjectType::Integer,
                    1 => super::gc::GcObjectType::TernaryValue,
                    2 => super::gc::GcObjectType::Array,
                    3 => super::gc::GcObjectType::String,
                    4 => super::gc::GcObjectType::Closure,
                    _ => super::gc::GcObjectType::Custom(inst.immediate as u8),
                };
                let ternary = self.is_ternary_mode(inst.src1);
                let handle = self.gc.allocate(obj_type, size, ternary)?;
                self.set_register(inst.dst, handle as i64)?;
                self.update_flags(handle as i64, false);
            }
            Opcode::Free => {
                let handle = self.get_register(inst.src1)? as usize;
                self.gc.remove_root(handle);
                self.set_register(inst.dst, 0)?;
                self.update_flags(0, false);
            }
            Opcode::ReadTime => {
                let ts = self.current_hptp_timestamp();
                let result = match inst.immediate {
                    0 => {
                        let lo = ts.femtoseconds as i64;
                        let hi = (ts.femtoseconds >> 64) as i64;
                        if inst.src1 <= 26 {
                            self.set_register(inst.src1, hi)?;
                        }
                        lo
                    }
                    1 => ts.seconds() as i64,
                    2 => ts.milliseconds() as i64,
                    3 => ts.nanoseconds() as i64,
                    4 => ts.picoseconds() as i64,
                    5 => ts.remaining_femtoseconds() as i64,
                    6 => self.cycles as i64,
                    7 => self.hptp_provider.timing_source() as i64,
                    _ => ts.femtoseconds as i64,
                };
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TAddV => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = packed_zip(a, b, |x, y| x.add(*y));
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TMulV => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let result = packed_zip(a, b, |x, y| x.multiply(y));
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TNegV => {
                let a = self.get_register(inst.src1)?;
                let result = packed_map(a, |t| t.not());
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
            Opcode::TRotV => {
                let a = self.get_register(inst.src1)?;
                let positions = if inst.immediate != 0 {
                    inst.immediate as usize
                } else {
                    self.get_register(inst.src2)? as usize
                };
                let result = packed_rotate_left(a, positions);
                self.set_register(inst.dst, result)?;
                self.set_ternary_mode(inst.dst, true)?;
                self.update_flags(result, false);
            }
        }
        Ok(())
    }

    pub fn get_register(&self, reg: u8) -> VmResult<i64> {
        if reg > 26 {
            return Err(VmError::InvalidRegister(reg));
        }
        Ok(self.registers.registers[reg as usize].value)
    }

    pub fn set_register(&mut self, reg: u8, value: i64) -> VmResult<()> {
        if reg > 26 {
            return Err(VmError::InvalidRegister(reg));
        }
        self.registers.registers[reg as usize].value = value;
        Ok(())
    }

    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    pub fn is_halted(&self) -> bool {
        self.registers.flags.halted
    }

    pub fn reset(&mut self) {
        self.registers = RegisterFile::default();
        self.stack = VmStack::new(4096);
        self.gc = GcHeap::new(self.memory.size / 4);
        self.cycles = 0;
        self.time_remaining = self.time_slice;
    }

    fn update_flags(&mut self, result: i64, overflow: bool) {
        self.registers.flags.zero = result == 0;
        self.registers.flags.negative = result < 0;
        self.registers.flags.positive = result > 0;
        self.registers.flags.overflow = overflow;
    }

    fn is_ternary_mode(&self, reg: u8) -> bool {
        if reg > 26 {
            return false;
        }
        self.registers.registers[reg as usize].ternary_mode
    }

    fn set_ternary_mode(&mut self, reg: u8, mode: bool) -> VmResult<()> {
        if reg > 26 {
            return Err(VmError::InvalidRegister(reg));
        }
        self.registers.registers[reg as usize].ternary_mode = mode;
        Ok(())
    }

    fn propagate_ternary_mode(&mut self, dst: u8, src1: u8, src2: u8) -> VmResult<()> {
        let mode = self.is_ternary_mode(src1) || self.is_ternary_mode(src2);
        self.set_ternary_mode(dst, mode)
    }

    fn copy_ternary_mode(&mut self, dst: u8, src: u8) -> VmResult<()> {
        let mode = self.is_ternary_mode(src);
        self.set_ternary_mode(dst, mode)
    }

    fn check_privilege(&self, required: PrivilegeLevel) -> VmResult<()> {
        match (required, self.registers.privilege) {
            (PrivilegeLevel::Ring0, PrivilegeLevel::Ring1) => {
                Err(VmError::InvalidProgram(alloc::string::String::from("Privilege violation: Ring0 required")))
            }
            _ => Ok(()),
        }
    }

    pub fn set_privilege(&mut self, level: PrivilegeLevel) {
        self.registers.privilege = level;
    }

    pub fn is_time_slice_exhausted(&self) -> bool {
        self.time_remaining == 0
    }

    pub fn set_time_slice(&mut self, slice: u64) {
        self.time_slice = slice;
        self.time_remaining = slice;
    }

    pub fn reset_time_slice(&mut self) {
        self.time_remaining = self.time_slice;
    }

    pub fn set_security_domain(&mut self, domain: u8) {
        self.security_domain = domain;
    }

    pub fn get_security_domain(&self) -> u8 {
        self.security_domain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vm() -> TernaryVm {
        TernaryVm::new(4096, Box::new(SimulatedHptp::new()))
    }

    #[test]
    fn test_memory_read_write_u8() {
        let mut mem = VmMemory::new(256);
        mem.write_u8(10, 42).unwrap();
        assert_eq!(mem.read_u8(10).unwrap(), 42);
    }

    #[test]
    fn test_memory_read_write_i64() {
        let mut mem = VmMemory::new(256);
        mem.write_i64(0, 123456789).unwrap();
        assert_eq!(mem.read_i64(0).unwrap(), 123456789);
    }

    #[test]
    fn test_memory_out_of_bounds() {
        let mem = VmMemory::new(16);
        assert!(mem.read_u8(16).is_err());
        assert!(mem.read_i64(10).is_err());
    }

    #[test]
    fn test_memory_read_bytes() {
        let mut mem = VmMemory::new(256);
        mem.write_u8(0, 1).unwrap();
        mem.write_u8(1, 2).unwrap();
        mem.write_u8(2, 3).unwrap();
        let bytes = mem.read_bytes(0, 3).unwrap();
        assert_eq!(bytes, &[1, 2, 3]);
    }

    #[test]
    fn test_stack_push_pop() {
        let mut stack = VmStack::new(10);
        stack.push(42).unwrap();
        stack.push(99).unwrap();
        assert_eq!(stack.pop().unwrap(), 99);
        assert_eq!(stack.pop().unwrap(), 42);
    }

    #[test]
    fn test_stack_overflow() {
        let mut stack = VmStack::new(2);
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        assert!(stack.push(3).is_err());
    }

    #[test]
    fn test_stack_underflow() {
        let mut stack = VmStack::new(10);
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_peek() {
        let mut stack = VmStack::new(10);
        stack.push(42).unwrap();
        assert_eq!(stack.peek().unwrap(), 42);
        assert_eq!(stack.size(), 1);
    }

    #[test]
    fn test_stack_is_empty() {
        let stack = VmStack::new(10);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_vm_new() {
        let vm = make_vm();
        assert_eq!(vm.cycles(), 0);
        assert!(!vm.is_halted());
    }

    #[test]
    fn test_vm_register_bounds() {
        let vm = make_vm();
        assert!(vm.get_register(0).is_ok());
        assert!(vm.get_register(26).is_ok());
        assert!(vm.get_register(27).is_err());
    }

    #[test]
    fn test_vm_set_get_register() {
        let mut vm = make_vm();
        vm.set_register(5, 100).unwrap();
        assert_eq!(vm.get_register(5).unwrap(), 100);
    }

    #[test]
    fn test_vm_halt() {
        let mut vm = make_vm();
        let mut prog = Program::new("halt_test");
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        let cycles = vm.run().unwrap();
        assert_eq!(cycles, 1);
        assert!(vm.is_halted());
    }

    #[test]
    fn test_vm_add() {
        let mut vm = make_vm();
        let mut prog = Program::new("add_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 10));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 20));
        prog.add_instruction(Instruction::new(Opcode::Add, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 30);
    }

    #[test]
    fn test_vm_sub() {
        let mut vm = make_vm();
        let mut prog = Program::new("sub_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 50));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 20));
        prog.add_instruction(Instruction::new(Opcode::Sub, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 30);
    }

    #[test]
    fn test_vm_mul() {
        let mut vm = make_vm();
        let mut prog = Program::new("mul_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 6));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 7));
        prog.add_instruction(Instruction::new(Opcode::Mul, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 42);
    }

    #[test]
    fn test_vm_div() {
        let mut vm = make_vm();
        let mut prog = Program::new("div_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 6));
        prog.add_instruction(Instruction::new(Opcode::Div, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 7);
    }

    #[test]
    fn test_vm_div_by_zero() {
        let mut vm = make_vm();
        let mut prog = Program::new("div_zero");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 10));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::Div, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_vm_neg() {
        let mut vm = make_vm();
        let mut prog = Program::new("neg_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::new(Opcode::Neg, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), -42);
    }

    #[test]
    fn test_vm_tadd() {
        let mut vm = make_vm();
        let mut prog = Program::new("tadd_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TAdd, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 0);
    }

    #[test]
    fn test_vm_tadd_wrap() {
        let mut vm = make_vm();
        let mut prog = Program::new("tadd_wrap");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TAdd, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), -1);
    }

    #[test]
    fn test_vm_tmul() {
        let mut vm = make_vm();
        let mut prog = Program::new("tmul_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TMul, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 1);
    }

    #[test]
    fn test_vm_tmul_zero() {
        let mut vm = make_vm();
        let mut prog = Program::new("tmul_zero");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TMul, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 0);
    }

    #[test]
    fn test_vm_tconvert_a_to_b() {
        let mut vm = make_vm();
        let mut prog = Program::new("tconvert");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::TConvert, 2, 0, 1, 1));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 0);
    }

    #[test]
    fn test_vm_tconvert_a_to_c() {
        let mut vm = make_vm();
        let mut prog = Program::new("tconvert_ac");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::TConvert, 2, 0, 1, 2));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 3);
    }

    #[test]
    fn test_vm_move() {
        let mut vm = make_vm();
        let mut prog = Program::new("move_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 77));
        prog.add_instruction(Instruction::new(Opcode::Move, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), 77);
    }

    #[test]
    fn test_vm_push_pop() {
        let mut vm = make_vm();
        let mut prog = Program::new("stack_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 55));
        prog.add_instruction(Instruction::new(Opcode::Push, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::Pop, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), 55);
    }

    #[test]
    fn test_vm_call_return() {
        let mut vm = make_vm();
        let mut prog = Program::new("call_ret");
        prog.add_instruction(Instruction::new(Opcode::Call, 0, 0, 0, 3));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 99));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::from_opcode(Opcode::Return));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(0).unwrap(), 42);
        assert_eq!(vm.get_register(1).unwrap(), 99);
    }

    #[test]
    fn test_vm_jump() {
        let mut vm = make_vm();
        let mut prog = Program::new("jump_test");
        prog.add_instruction(Instruction::new(Opcode::Jump, 0, 0, 0, 2));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 999));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(0).unwrap(), 42);
    }

    #[test]
    fn test_vm_jump_zero() {
        let mut vm = make_vm();
        let mut prog = Program::new("jz_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 5));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 5));
        prog.add_instruction(Instruction::new(Opcode::Cmp, 0, 0, 1, 0));
        prog.add_instruction(Instruction::new(Opcode::JumpZero, 0, 0, 0, 5));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 2, 0, 0, 999));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 2, 0, 0, 42));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 42);
    }

    #[test]
    fn test_vm_jump_not_zero() {
        let mut vm = make_vm();
        let mut prog = Program::new("jnz_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 5));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 3));
        prog.add_instruction(Instruction::new(Opcode::Cmp, 0, 0, 1, 0));
        prog.add_instruction(Instruction::new(Opcode::JumpNotZero, 0, 0, 0, 5));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 2, 0, 0, 999));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 2, 0, 0, 42));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 42);
    }

    #[test]
    fn test_vm_cmp_imm() {
        let mut vm = make_vm();
        let mut prog = Program::new("cmpi_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 10));
        prog.add_instruction(Instruction::new(Opcode::CmpImm, 0, 0, 0, 10));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.flags.zero);
    }

    #[test]
    fn test_vm_bitwise_and() {
        let mut vm = make_vm();
        let mut prog = Program::new("and_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 0xFF));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 0x0F));
        prog.add_instruction(Instruction::new(Opcode::And, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 0x0F);
    }

    #[test]
    fn test_vm_load_store_memory() {
        let mut vm = make_vm();
        let mut prog = Program::new("mem_test");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 12345));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::Store, 0, 1, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::Load, 2, 1, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 12345);
    }

    #[test]
    fn test_vm_reset() {
        let mut vm = make_vm();
        vm.set_register(0, 42).unwrap();
        vm.reset();
        assert_eq!(vm.get_register(0).unwrap(), 0);
        assert_eq!(vm.cycles(), 0);
    }

    #[test]
    fn test_vm_sum_program() {
        let mut vm = make_vm();
        let mut prog = Program::new("sum_1_2_3");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 2));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 2, 0, 0, 3));
        prog.add_instruction(Instruction::new(Opcode::Add, 3, 0, 1, 0));
        prog.add_instruction(Instruction::new(Opcode::Add, 3, 3, 2, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(3).unwrap(), 6);
    }

    #[test]
    fn test_vm_max_cycles() {
        let mut vm = make_vm();
        vm.max_cycles = 5;
        let mut prog = Program::new("infinite");
        prog.add_instruction(Instruction::new(Opcode::Jump, 0, 0, 0, 0));
        vm.load_program(prog).unwrap();
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_vm_no_program() {
        let mut vm = make_vm();
        assert!(vm.step().is_err());
    }

    #[test]
    fn test_vm_pc_out_of_bounds() {
        let mut vm = make_vm();
        let mut prog = Program::new("oob");
        prog.add_instruction(Instruction::from_opcode(Opcode::Nop));
        vm.load_program(prog).unwrap();
        vm.step().unwrap();
        assert!(vm.step().is_err());
    }

    #[test]
    fn test_vm_trot() {
        let mut vm = make_vm();
        let mut prog = Program::new("trot");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TRot, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 0);
    }

    #[test]
    fn test_vm_flags_after_cmp() {
        let mut vm = make_vm();
        let mut prog = Program::new("flags");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 10));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 20));
        prog.add_instruction(Instruction::new(Opcode::Cmp, 0, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.flags.negative);
        assert!(!vm.registers.flags.zero);
    }

    #[test]
    fn test_vm_txor_kleene_min() {
        let mut vm = make_vm();
        let mut prog = Program::new("txor_min");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::TXor, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), -1); // min(1, -1) = -1
    }

    #[test]
    fn test_vm_txor_same_values() {
        let mut vm = make_vm();
        let mut prog = Program::new("txor_same");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TXor, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 1); // min(1, 1) = 1
    }

    #[test]
    fn test_vm_tneg_per_trit() {
        let mut vm = make_vm();
        let mut prog = Program::new("tneg_trit");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TNeg, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), -1);
    }

    #[test]
    fn test_vm_tneg_zero() {
        let mut vm = make_vm();
        let mut prog = Program::new("tneg_zero");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::TNeg, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), 0);
    }

    #[test]
    fn test_vm_tand_lukasiewicz() {
        let mut vm = make_vm();
        let mut prog = Program::new("tand_luk");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::TAnd, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), -1); // max(1+(-1)-1, -1) = max(-1,-1) = -1
    }

    #[test]
    fn test_vm_tand_both_positive() {
        let mut vm = make_vm();
        let mut prog = Program::new("tand_pos");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TAnd, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 1); // max(1+1-1, -1) = max(1,-1) = 1
    }

    #[test]
    fn test_vm_tor() {
        let mut vm = make_vm();
        let mut prog = Program::new("tor");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TOr, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 1);
    }

    #[test]
    fn test_vm_tsub() {
        let mut vm = make_vm();
        let mut prog = Program::new("tsub");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TSub, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 0);
    }

    #[test]
    fn test_vm_tinv_one() {
        let mut vm = make_vm();
        let mut prog = Program::new("tinv_one");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TInv, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), 1); // GF(3) inverse of 1 = 1
    }

    #[test]
    fn test_vm_tinv_neg() {
        let mut vm = make_vm();
        let mut prog = Program::new("tinv_neg");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::TInv, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), -1); // GF(3) inverse of -1 = -1
    }

    #[test]
    fn test_vm_tinv_zero_errors() {
        let mut vm = make_vm();
        let mut prog = Program::new("tinv_zero");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::TInv, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        assert!(vm.run().is_err()); // GF(3) inverse of 0 is undefined
    }

    #[test]
    fn test_vm_tcmp_equal() {
        let mut vm = make_vm();
        let mut prog = Program::new("tcmp_eq");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TCmp, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 0);
    }

    #[test]
    fn test_vm_tcmp_greater() {
        let mut vm = make_vm();
        let mut prog = Program::new("tcmp_gt");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, -1));
        prog.add_instruction(Instruction::new(Opcode::TCmp, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 1);
    }

    #[test]
    fn test_vm_tload_tstore() {
        let mut vm = make_vm();
        let mut prog = Program::new("tload_tstore");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::TStore, 0, 1, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::TLoad, 2, 1, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(2).unwrap(), 42);
        assert!(vm.registers.registers[2].ternary_mode);
    }

    #[test]
    fn test_vm_thash_deterministic() {
        let mut vm = make_vm();
        let mut prog = Program::new("thash");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::THash, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let hash1 = vm.get_register(1).unwrap();
        vm.reset();
        let mut prog2 = Program::new("thash2");
        prog2.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog2.add_instruction(Instruction::new(Opcode::THash, 1, 0, 0, 0));
        prog2.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog2).unwrap();
        vm.run().unwrap();
        let hash2 = vm.get_register(1).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_vm_thash_different_inputs() {
        let mut vm = make_vm();
        let mut prog = Program::new("thash_a");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::THash, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let hash_a = vm.get_register(1).unwrap();
        vm.reset();
        let mut prog2 = Program::new("thash_b");
        prog2.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, -1));
        prog2.add_instruction(Instruction::new(Opcode::THash, 1, 0, 0, 0));
        prog2.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog2).unwrap();
        vm.run().unwrap();
        let hash_b = vm.get_register(1).unwrap();
        assert_ne!(hash_a, hash_b);
    }

    #[test]
    fn test_vm_tentropy() {
        let mut vm = make_vm();
        let mut prog = Program::new("tentropy");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 7));
        prog.add_instruction(Instruction::new(Opcode::TEntropy, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.registers[2].ternary_mode);
    }

    #[test]
    fn test_vm_tpolymul() {
        let mut vm = make_vm();
        let mut prog = Program::new("tpolymul");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TPolyMul, 2, 0, 1, 3));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.registers[2].ternary_mode);
    }

    #[test]
    fn test_vm_tntt() {
        let mut vm = make_vm();
        let mut prog = Program::new("tntt");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TNTT, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.registers[1].ternary_mode);
    }

    #[test]
    fn test_vm_taddv() {
        let mut vm = make_vm();
        let mut prog = Program::new("taddv");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TAddV, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let result = vm.get_register(2).unwrap();
        let trits = crate::ternary::unpack_trits(result);
        assert_eq!(trits[0].to_a(), -1); // 1+1 = 2 mod 3 = -1
        assert!(vm.registers.registers[2].ternary_mode);
    }

    #[test]
    fn test_vm_tmulv() {
        let mut vm = make_vm();
        let mut prog = Program::new("tmulv");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TMulV, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let result = vm.get_register(2).unwrap();
        let trits = crate::ternary::unpack_trits(result);
        assert_eq!(trits[0].to_a(), 1); // 1*1 = 1
        assert!(vm.registers.registers[2].ternary_mode);
    }

    #[test]
    fn test_vm_tnegv() {
        let mut vm = make_vm();
        let mut prog = Program::new("tnegv");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TNegV, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let result = vm.get_register(1).unwrap();
        let trits = crate::ternary::unpack_trits(result);
        assert_eq!(trits[0].to_a(), -1);
        assert!(vm.registers.registers[1].ternary_mode);
    }

    #[test]
    fn test_vm_trotv() {
        let mut vm = make_vm();
        let mut prog = Program::new("trotv");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TRotV, 1, 0, 0, 1));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.registers[1].ternary_mode);
    }

    #[test]
    fn test_vm_ternary_mode_propagation() {
        let mut vm = make_vm();
        vm.set_register(0, 1).unwrap();
        vm.registers.registers[0].ternary_mode = true;
        vm.set_register(1, 1).unwrap();
        let mut prog = Program::new("mode_prop");
        prog.add_instruction(Instruction::new(Opcode::TAdd, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.registers[2].ternary_mode);
    }

    #[test]
    fn test_vm_treduce_add() {
        let mut vm = make_vm();
        let mut prog = Program::new("treduce");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TReduce, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), 1);
    }

    #[test]
    fn test_vm_trotinv() {
        let mut vm = make_vm();
        let mut prog = Program::new("trotinv");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TRotInv, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let result = vm.get_register(1).unwrap();
        let trits = crate::ternary::unpack_trits(result);
        assert_eq!(trits[0].to_a(), 0);
    }

    #[test]
    fn test_vm_syscall_cycles() {
        let mut vm = make_vm();
        let mut prog = Program::new("syscall_cycles");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::Syscall, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.get_register(2).unwrap() > 0);
    }

    #[test]
    fn test_vm_trap() {
        let mut vm = make_vm();
        let mut prog = Program::new("trap");
        prog.add_instruction(Instruction::new(Opcode::Trap, 0, 0, 0, 42));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_vm_alloc_free() {
        let mut vm = make_vm();
        let mut prog = Program::new("alloc_free");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 64));
        prog.add_instruction(Instruction::new(Opcode::Alloc, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::Free, 2, 1, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.get_register(1).unwrap() >= 0);
    }

    #[test]
    fn test_vm_readtime() {
        let mut vm = make_vm();
        let mut prog = Program::new("readtime");
        prog.add_instruction(Instruction::new(Opcode::Nop, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 1, 0, 0, 6));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(0).unwrap(), 1000,
            "imm=0: atomic fs pair low after 1 NOP cycle (1 cycle × 1000 fs/cycle)");
        assert_eq!(vm.get_register(1).unwrap(), 2,
            "imm=6: raw cycle count after NOP + first ReadTime");
    }

    #[test]
    fn test_vm_readtime_hptp_components() {
        let provider = SimulatedHptp::new()
            .with_epoch(2_500_000_000_000_000_000)
            .with_cycle_period(500_000_000_000);
        let mut vm = TernaryVm::new(4096, Box::new(provider));

        let ts_at = |cycles: u128| -> crate::timing::FemtosecondTimestamp {
            crate::timing::FemtosecondTimestamp::new(
                2_500_000_000_000_000_000u128 + cycles * 500_000_000_000u128,
            )
        };

        let mut prog = Program::new("readtime_components");
        prog.add_instruction(Instruction::new(Opcode::Nop, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 2, 0, 0, 2));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 3, 0, 0, 6));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();

        let ts1 = ts_at(1);
        assert_eq!(vm.get_register(0).unwrap(), ts1.femtoseconds as i64,
            "imm=0 at cycle 1: atomic fs pair low 64 bits");
        let ts2 = ts_at(2);
        assert_eq!(vm.get_register(1).unwrap(), ts2.seconds() as i64,
            "imm=1 at cycle 2: seconds since Salvi Epoch");
        let ts3 = ts_at(3);
        assert_eq!(vm.get_register(2).unwrap(), ts3.milliseconds() as i64,
            "imm=2 at cycle 3: milliseconds component");
        assert_eq!(vm.get_register(3).unwrap(), 4,
            "imm=6 at cycle 4: raw cycle count");

        let ext_ts = vm.current_hptp_timestamp();
        assert!(ext_ts.femtoseconds > 2_500_000_000_000_000_000,
            "current_hptp_timestamp() returns HPTP time with epoch");
    }

    #[test]
    fn test_vm_readtime_simulated_builder() {
        let provider = SimulatedHptp::new()
            .with_epoch(1_000_000_000_000_000)
            .with_cycle_period(500);
        let mut vm = TernaryVm::new(4096, Box::new(provider));
        let mut prog = Program::new("simulated_builder");
        prog.add_instruction(Instruction::new(Opcode::Nop, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 1, 0, 0, 1));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let expected_fs = 1_000_000_000_000_000u128 + 1 * 500;
        assert_eq!(vm.get_register(0).unwrap(), expected_fs as i64,
            "builder epoch + 1 cycle × 500 fs/cycle");
        assert_eq!(vm.get_register(1).unwrap(), 1,
            "imm=1: seconds component");
    }

    #[test]
    fn test_vm_readtime_live_callback() {
        use crate::TimingSource;
        let provider = crate::timing::LiveHptp::new(
            Box::new(|cycles| {
                crate::timing::FemtosecondTimestamp::new(42_000_000_000_000_000 + cycles as u128 * 2000)
            }),
            TimingSource::OpticalAtomic,
        );
        let mut vm = TernaryVm::new(4096, Box::new(provider));
        let mut prog = Program::new("live_callback");
        prog.add_instruction(Instruction::new(Opcode::Nop, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 0, 2, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 1, 0, 0, 7));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let expected_fs = 42_000_000_000_000_000u128 + 1 * 2000;
        assert_eq!(vm.get_register(0).unwrap(), expected_fs as i64,
            "imm=0: LiveHptp callback returns custom timestamp (low 64)");
        assert_eq!(vm.get_register(2).unwrap(), (expected_fs >> 64) as i64,
            "imm=0: high 64 bits stored in src1 register");
        assert_eq!(vm.get_register(1).unwrap(), TimingSource::OpticalAtomic as i64,
            "imm=7: timing source discriminant (OpticalAtomic)");
    }

    #[test]
    fn test_vm_readtime_all_selectors() {
        let provider = SimulatedHptp::new()
            .with_epoch(2_500_345_678_901_234_567u128)
            .with_cycle_period(1_000_000_000);
        let mut vm = TernaryVm::new(4096, Box::new(provider));
        let mut prog = Program::new("all_selectors");
        prog.add_instruction(Instruction::new(Opcode::Nop, 0, 0, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 0, 5, 0, 0));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 2, 0, 0, 2));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 3, 0, 0, 3));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 4, 0, 0, 4));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 6, 0, 0, 5));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 7, 0, 0, 6));
        prog.add_instruction(Instruction::new(Opcode::ReadTime, 8, 0, 0, 7));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();

        let ts1 = crate::timing::FemtosecondTimestamp::new(
            2_500_345_678_901_234_567u128 + 1 * 1_000_000_000
        );
        assert_eq!(vm.get_register(0).unwrap(), ts1.femtoseconds as i64,
            "imm=0: atomic pair low 64");
        assert_eq!(vm.get_register(5).unwrap(), (ts1.femtoseconds >> 64) as i64,
            "imm=0: atomic pair high stored in src1 register");

        let ts2 = crate::timing::FemtosecondTimestamp::new(
            2_500_345_678_901_234_567u128 + 2 * 1_000_000_000
        );
        assert_eq!(vm.get_register(1).unwrap(), ts2.seconds() as i64,
            "imm=1: seconds");

        assert_eq!(vm.get_register(7).unwrap(), 7,
            "imm=6: raw cycles");
        assert_eq!(vm.get_register(8).unwrap(), crate::TimingSource::SystemClock as i64,
            "imm=7: TimingSource discriminant");
    }

    #[test]
    fn test_vm_tpolyadd() {
        let mut vm = make_vm();
        let mut prog = Program::new("tpolyadd");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TPolyAdd, 2, 0, 1, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let result = vm.get_register(2).unwrap();
        let trits = crate::ternary::unpack_trits(result);
        assert_eq!(trits[0].to_a(), -1);
    }

    #[test]
    fn test_vm_tpolysample() {
        let mut vm = make_vm();
        let mut prog = Program::new("tpolysample");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::new(Opcode::TPolySample, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.registers[1].ternary_mode);
    }

    #[test]
    fn test_vm_tcompress_tdecompress() {
        let mut vm = make_vm();
        let mut prog = Program::new("compress_decompress");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::TCompress, 1, 0, 2, 0));
        prog.add_instruction(Instruction::new(Opcode::TDecompress, 3, 1, 2, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        let count = vm.get_register(2).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_vm_syscall_trit_width() {
        let mut vm = make_vm();
        let mut prog = Program::new("syscall_width");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 2));
        prog.add_instruction(Instruction::new(Opcode::Syscall, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), 27);
    }

    #[test]
    fn test_vm_positive_flag() {
        let mut vm = make_vm();
        let mut prog = Program::new("pos_flag");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        prog.add_instruction(Instruction::new(Opcode::CmpImm, 0, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert!(vm.registers.flags.positive);
        assert!(!vm.registers.flags.negative);
        assert!(!vm.registers.flags.zero);
    }

    #[test]
    fn test_vm_privilege_levels() {
        let mut vm = make_vm();
        assert_eq!(vm.registers.privilege, PrivilegeLevel::Ring0);
        vm.set_privilege(PrivilegeLevel::Ring1);
        assert_eq!(vm.registers.privilege, PrivilegeLevel::Ring1);
    }

    #[test]
    fn test_vm_time_slice() {
        let mut vm = make_vm();
        vm.set_time_slice(3);
        let mut prog = Program::new("time_slice");
        prog.add_instruction(Instruction::from_opcode(Opcode::Nop));
        prog.add_instruction(Instruction::from_opcode(Opcode::Nop));
        prog.add_instruction(Instruction::from_opcode(Opcode::Nop));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.step().unwrap();
        assert_eq!(vm.time_remaining, 2);
        vm.step().unwrap();
        assert_eq!(vm.time_remaining, 1);
        vm.step().unwrap();
        assert!(vm.is_time_slice_exhausted());
    }

    #[test]
    fn test_vm_security_domain() {
        let mut vm = make_vm();
        assert_eq!(vm.get_security_domain(), 0);
        vm.set_security_domain(2);
        assert_eq!(vm.get_security_domain(), 2);
    }

    #[test]
    fn test_vm_syscall_security_domain() {
        let mut vm = make_vm();
        vm.set_security_domain(1);
        let mut prog = Program::new("syscall_sec");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 4));
        prog.add_instruction(Instruction::new(Opcode::Syscall, 1, 0, 0, 0));
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        vm.run().unwrap();
        assert_eq!(vm.get_register(1).unwrap(), 1);
    }
}
