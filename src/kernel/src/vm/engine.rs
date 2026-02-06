use alloc::vec::Vec;
use super::{VmError, VmResult};
use super::instruction::*;

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
    memory: VmMemory,
    stack: VmStack,
    program: Option<Program>,
    cycles: u64,
    max_cycles: u64,
}

impl TernaryVm {
    pub fn new(memory_size: usize) -> Self {
        Self {
            registers: RegisterFile::default(),
            memory: VmMemory::new(memory_size),
            stack: VmStack::new(4096),
            program: None,
            cycles: 0,
            max_cycles: 1_000_000,
        }
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
                let a_gf = a.rem_euclid(3);
                let b_gf = b.rem_euclid(3);
                let sum = (a_gf + b_gf) % 3;
                let result = if sum > 1 { sum - 3 } else { sum };
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TMul => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let a_gf = a.rem_euclid(3);
                let b_gf = b.rem_euclid(3);
                let product = (a_gf * b_gf) % 3;
                let result = if product > 1 { product - 3 } else { product };
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TNeg => {
                let a = self.get_register(inst.src1)?;
                let result = -a;
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TRot => {
                let a = self.get_register(inst.src1)?;
                let positions = self.get_register(inst.src2)?;
                let mut val = a;
                let rot_count = positions.rem_euclid(3);
                for _ in 0..rot_count {
                    val = match val {
                        -1 => 0,
                        0 => 1,
                        1 => -1,
                        _ => val,
                    };
                }
                self.set_register(inst.dst, val)?;
                self.update_flags(val, false);
            }
            Opcode::TXor => {
                let a = self.get_register(inst.src1)?;
                let b = self.get_register(inst.src2)?;
                let a_gf = a.rem_euclid(3);
                let b_gf = b.rem_euclid(3);
                let sum = (a_gf + b_gf) % 3;
                let result = if sum > 1 { sum - 3 } else { sum };
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
            }
            Opcode::TConvert => {
                let val = self.get_register(inst.src1)?;
                let from_repr = self.get_register(inst.src2)?;
                let to_repr = inst.immediate;
                let a_val = match from_repr {
                    0 => val,
                    1 => val - 1,
                    2 => val - 2,
                    _ => val,
                };
                let result = match to_repr {
                    0 => a_val,
                    1 => a_val + 1,
                    2 => a_val + 2,
                    _ => a_val,
                };
                self.set_register(inst.dst, result)?;
                self.update_flags(result, false);
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
        self.cycles = 0;
    }

    fn update_flags(&mut self, result: i64, overflow: bool) {
        self.registers.flags.zero = result == 0;
        self.registers.flags.negative = result < 0;
        self.registers.flags.overflow = overflow;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vm() -> TernaryVm {
        TernaryVm::new(4096)
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
}
