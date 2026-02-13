// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use plenumnet_kernel::ternary::{Trit, Tryte, TernaryWord, Representation, convert_representation, information_density};
use plenumnet_kernel::compat::gateway::{
    binary_to_balanced_ternary, balanced_ternary_to_binary,
    binary_bytes_to_ternary, ternary_to_binary_bytes,
    binary_u8_to_representation_b, representation_b_to_binary_u8,
    BinaryTernaryGateway, GatewayMode,
};
use plenumnet_kernel::vm::engine::TernaryVm;
use plenumnet_kernel::vm::instruction::{Instruction, Opcode, Program};

#[derive(Serialize, Deserialize)]
pub struct TritResult {
    pub a: i8,
    pub b: u8,
    pub c: u8,
}

#[derive(Serialize, Deserialize)]
pub struct GF3ArithResult {
    pub operation: String,
    pub input_a: i8,
    pub input_b: i8,
    pub result: TritResult,
}

#[derive(Serialize, Deserialize)]
pub struct DensityResult {
    pub trit_count: u32,
    pub ternary_states: String,
    pub binary_states: String,
    pub equivalent_bits: f64,
    pub efficiency_gain: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ConversionResult {
    pub original: Vec<u8>,
    pub trits: Vec<i8>,
    pub recovered: Vec<u8>,
    pub lossless: bool,
    pub trit_count: usize,
    pub compression_ratio: f64,
}

#[derive(Serialize, Deserialize)]
pub struct VmExecutionResult {
    pub cycles: u64,
    pub halted: bool,
    pub registers: Vec<i64>,
    pub flags: VmFlagsResult,
}

#[derive(Serialize, Deserialize)]
pub struct VmFlagsResult {
    pub zero: bool,
    pub negative: bool,
    pub overflow: bool,
    pub ternary: bool,
    pub halted: bool,
}

fn trit_to_result(t: &Trit) -> TritResult {
    TritResult {
        a: t.to_a(),
        b: t.to_b(),
        c: t.to_c(),
    }
}

#[wasm_bindgen]
pub fn trit_add(a: i8, b: i8) -> JsValue {
    let ta = match Trit::from_a(a) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value for a: {}", a)),
    };
    let tb = match Trit::from_a(b) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value for b: {}", b)),
    };
    let result = ta.add(&tb);
    let r = GF3ArithResult {
        operation: "GF(3) Addition".into(),
        input_a: a,
        input_b: b,
        result: trit_to_result(&result),
    };
    serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn trit_multiply(a: i8, b: i8) -> JsValue {
    let ta = match Trit::from_a(a) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value for a: {}", a)),
    };
    let tb = match Trit::from_a(b) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value for b: {}", b)),
    };
    let result = ta.multiply(&tb);
    let r = GF3ArithResult {
        operation: "GF(3) Multiplication".into(),
        input_a: a,
        input_b: b,
        result: trit_to_result(&result),
    };
    serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn trit_xor(a: i8, b: i8) -> JsValue {
    let ta = match Trit::from_a(a) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value for a: {}", a)),
    };
    let tb = match Trit::from_a(b) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value for b: {}", b)),
    };
    let result = ta.xor(&tb);
    let r = GF3ArithResult {
        operation: "GF(3) XOR (min)".into(),
        input_a: a,
        input_b: b,
        result: trit_to_result(&result),
    };
    serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn trit_not(a: i8) -> JsValue {
    let ta = match Trit::from_a(a) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value: {}", a)),
    };
    let result = ta.not();
    serde_wasm_bindgen::to_value(&trit_to_result(&result)).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn trit_rotate(a: i8) -> JsValue {
    let ta = match Trit::from_a(a) {
        Some(t) => t,
        None => return JsValue::from_str(&format!("Invalid trit value: {}", a)),
    };
    let result = ta.rotate();
    serde_wasm_bindgen::to_value(&trit_to_result(&result)).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn convert_repr(value: i8, from_repr: u8, to_repr: u8) -> i8 {
    let from = match from_repr {
        0 => Representation::A,
        1 => Representation::B,
        2 => Representation::C,
        _ => return value,
    };
    let to = match to_repr {
        0 => Representation::A,
        1 => Representation::B,
        2 => Representation::C,
        _ => return value,
    };
    convert_representation(value, from, to)
}

#[wasm_bindgen]
pub fn calc_information_density(trit_count: u32) -> JsValue {
    let d = information_density(trit_count);
    let r = DensityResult {
        trit_count: d.trit_count,
        ternary_states: d.ternary_states.to_string(),
        binary_states: d.binary_states.to_string(),
        equivalent_bits: d.equivalent_bits,
        efficiency_gain: d.efficiency_gain,
    };
    serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn binary_to_ternary(data: &[u8]) -> JsValue {
    let trits = binary_bytes_to_ternary(data);
    let recovered = ternary_to_binary_bytes(&trits).unwrap_or_default();
    let lossless = recovered == data;
    let ratio = if data.is_empty() { 0.0 } else { trits.len() as f64 / (data.len() as f64 * 8.0) };
    let r = ConversionResult {
        original: data.to_vec(),
        trits: trits.clone(),
        recovered,
        lossless,
        trit_count: trits.len(),
        compression_ratio: (ratio * 10000.0).round() / 10000.0,
    };
    serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn i64_to_balanced_ternary(value: f64) -> JsValue {
    let v = value as i64;
    let trits = binary_to_balanced_ternary(v);
    serde_wasm_bindgen::to_value(&trits).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn balanced_ternary_to_i64(trits_json: &str) -> f64 {
    let trits: Vec<i8> = match serde_json::from_str(trits_json) {
        Ok(t) => t,
        Err(_) => return f64::NAN,
    };
    match balanced_ternary_to_binary(&trits) {
        Ok(v) => v as f64,
        Err(_) => f64::NAN,
    }
}

#[wasm_bindgen]
pub fn gf3_addition_table() -> JsValue {
    let mut table = Vec::new();
    for a in [-1i8, 0, 1] {
        for b in [-1i8, 0, 1] {
            let ta = Trit::from_a(a).unwrap();
            let tb = Trit::from_a(b).unwrap();
            let result = ta.add(&tb);
            table.push(vec![a, b, result.to_a()]);
        }
    }
    serde_wasm_bindgen::to_value(&table).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn gf3_multiplication_table() -> JsValue {
    let mut table = Vec::new();
    for a in [-1i8, 0, 1] {
        for b in [-1i8, 0, 1] {
            let ta = Trit::from_a(a).unwrap();
            let tb = Trit::from_a(b).unwrap();
            let result = ta.multiply(&tb);
            table.push(vec![a, b, result.to_a()]);
        }
    }
    serde_wasm_bindgen::to_value(&table).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn run_vm_program(program_json: &str) -> JsValue {
    #[derive(Deserialize)]
    struct ProgramInput {
        instructions: Vec<InstructionInput>,
        #[serde(default)]
        initial_registers: std::collections::HashMap<u8, i64>,
        #[serde(default = "default_memory_size")]
        memory_size: usize,
    }

    fn default_memory_size() -> usize { 4096 }

    #[derive(Deserialize)]
    struct InstructionInput {
        opcode: String,
        #[serde(default)]
        dst: u8,
        #[serde(default)]
        src1: u8,
        #[serde(default)]
        src2: u8,
        #[serde(default)]
        immediate: i64,
    }

    fn parse_opcode(s: &str) -> Option<Opcode> {
        match s.to_uppercase().as_str() {
            "NOP" => Some(Opcode::Nop),
            "HALT" => Some(Opcode::Halt),
            "ADD" => Some(Opcode::Add),
            "SUB" => Some(Opcode::Sub),
            "MUL" => Some(Opcode::Mul),
            "DIV" => Some(Opcode::Div),
            "MOD" => Some(Opcode::Mod),
            "NEG" => Some(Opcode::Neg),
            "TADD" => Some(Opcode::TAdd),
            "TMUL" => Some(Opcode::TMul),
            "TNEG" => Some(Opcode::TNeg),
            "TROT" => Some(Opcode::TRot),
            "TXOR" => Some(Opcode::TXor),
            "TCONVERT" => Some(Opcode::TConvert),
            "LOAD" => Some(Opcode::Load),
            "STORE" => Some(Opcode::Store),
            "MOVE" => Some(Opcode::Move),
            "LOADIMM" => Some(Opcode::LoadImm),
            "PUSH" => Some(Opcode::Push),
            "POP" => Some(Opcode::Pop),
            "JUMP" => Some(Opcode::Jump),
            "JUMPZERO" => Some(Opcode::JumpZero),
            "JUMPNEG" => Some(Opcode::JumpNeg),
            "JUMPPOS" => Some(Opcode::JumpPos),
            "CALL" => Some(Opcode::Call),
            "RETURN" => Some(Opcode::Return),
            "JUMPNOTZERO" => Some(Opcode::JumpNotZero),
            "CMP" => Some(Opcode::Cmp),
            "CMPIMM" => Some(Opcode::CmpImm),
            "AND" => Some(Opcode::And),
            "OR" => Some(Opcode::Or),
            "XOR" => Some(Opcode::Xor),
            "SHL" => Some(Opcode::Shl),
            "SHR" => Some(Opcode::Shr),
            "NOT" => Some(Opcode::Not),
            _ => None,
        }
    }

    let input: ProgramInput = match serde_json::from_str(program_json) {
        Ok(p) => p,
        Err(e) => return JsValue::from_str(&format!("Parse error: {}", e)),
    };

    let mut vm = TernaryVm::new(input.memory_size, Box::new(plenumnet_kernel::timing::SimulatedHptp::new()));

    for (reg, val) in &input.initial_registers {
        if vm.set_register(*reg, *val).is_err() {
            return JsValue::from_str(&format!("Invalid register: {}", reg));
        }
    }

    let mut prog = Program::new("wasm_program");
    for inst_input in &input.instructions {
        let opcode = match parse_opcode(&inst_input.opcode) {
            Some(op) => op,
            None => return JsValue::from_str(&format!("Unknown opcode: {}", inst_input.opcode)),
        };
        prog.add_instruction(Instruction::new(
            opcode,
            inst_input.dst,
            inst_input.src1,
            inst_input.src2,
            inst_input.immediate,
        ));
    }

    if vm.load_program(prog).is_err() {
        return JsValue::from_str("Invalid program");
    }

    let cycles = match vm.run() {
        Ok(c) => c,
        Err(e) => {
            let cycles = vm.cycles();
            let mut regs = Vec::new();
            for i in 0..27 {
                regs.push(vm.get_register(i).unwrap_or(0));
            }
            let r = VmExecutionResult {
                cycles,
                halted: vm.is_halted(),
                registers: regs,
                flags: VmFlagsResult {
                    zero: vm.registers.flags.zero,
                    negative: vm.registers.flags.negative,
                    overflow: vm.registers.flags.overflow,
                    ternary: vm.registers.flags.ternary,
                    halted: vm.registers.flags.halted,
                },
            };
            return serde_wasm_bindgen::to_value(&r).unwrap_or(
                JsValue::from_str(&format!("VM error after {} cycles: {}", cycles, e))
            );
        }
    };

    let mut regs = Vec::new();
    for i in 0..27 {
        regs.push(vm.get_register(i).unwrap_or(0));
    }

    let r = VmExecutionResult {
        cycles,
        halted: vm.is_halted(),
        registers: regs,
        flags: VmFlagsResult {
            zero: vm.registers.flags.zero,
            negative: vm.registers.flags.negative,
            overflow: vm.registers.flags.overflow,
            ternary: vm.registers.flags.ternary,
            halted: vm.registers.flags.halted,
        },
    };
    serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn kernel_version() -> String {
    format!("PlenumNET Kernel v{} (WASM)", plenumnet_kernel::KERNEL_VERSION)
}

#[wasm_bindgen]
pub fn spec_version() -> String {
    "TVM ISA v1.0.0 | 35 opcodes | GF(3) field | 27 registers".into()
}
