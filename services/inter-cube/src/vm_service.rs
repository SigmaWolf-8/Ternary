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
}

impl Default for TernaryRegister {
    fn default() -> Self {
        Self {
            value: 0,
            trit_width: 27,
        }
    }
}

pub struct TernaryVmInstance {
    pub registers: [TernaryRegister; 27],
    pub program_counter: u64,
    pub stack_pointer: u64,
    pub flags_halted: bool,
    pub flags_overflow: bool,
    pub flags_carry: bool,
    pub flags_zero: bool,
    pub memory: Vec<u8>,
    pub memory_size: usize,
    pub stack: Vec<i64>,
    pub cycles: u64,
    pub max_cycles: u64,
    pub state: VmState,
    pub last_error: Option<String>,
    pub program_name: Option<String>,
}

impl TernaryVmInstance {
    pub fn new(memory_size: usize) -> Self {
        Self {
            registers: std::array::from_fn(|_| TernaryRegister::default()),
            program_counter: 0,
            stack_pointer: 0,
            flags_halted: false,
            flags_overflow: false,
            flags_carry: false,
            flags_zero: true,
            memory: vec![0u8; memory_size],
            memory_size,
            stack: Vec::new(),
            cycles: 0,
            max_cycles: 1_000_000,
            state: VmState::Idle,
            last_error: None,
            program_name: None,
        }
    }

    pub fn reset(&mut self) {
        self.registers = std::array::from_fn(|_| TernaryRegister::default());
        self.program_counter = 0;
        self.stack_pointer = 0;
        self.flags_halted = false;
        self.flags_overflow = false;
        self.flags_carry = false;
        self.flags_zero = true;
        self.memory.fill(0);
        self.stack.clear();
        self.cycles = 0;
        self.state = VmState::Idle;
        self.last_error = None;
        self.program_name = None;
    }

    pub fn exec_program(&mut self, name: &str, instructions: &[VmInstruction]) -> Result<VmExecResult, String> {
        self.reset();
        self.program_name = Some(name.to_string());
        self.state = VmState::Running;

        for inst in instructions {
            if self.cycles >= self.max_cycles {
                self.state = VmState::Error;
                self.last_error = Some("Max cycles exceeded".to_string());
                return Err("Max cycles exceeded".to_string());
            }

            self.cycles += 1;

            match inst.opcode.as_str() {
                "NOP" => {}
                "HALT" => {
                    self.flags_halted = true;
                    self.state = VmState::Halted;
                    break;
                }
                "LOAD" => {
                    let reg = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let val = inst.operands.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
                    if reg < 27 {
                        self.registers[reg].value = val;
                    }
                }
                "ADD" => {
                    let dst = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let src = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if dst < 27 && src < 27 {
                        let result = self.registers[dst].value.wrapping_add(self.registers[src].value);
                        self.registers[dst].value = result;
                        self.flags_zero = result == 0;
                        self.flags_overflow = result > i64::MAX / 2 || result < i64::MIN / 2;
                    }
                }
                "SUB" => {
                    let dst = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let src = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if dst < 27 && src < 27 {
                        let result = self.registers[dst].value.wrapping_sub(self.registers[src].value);
                        self.registers[dst].value = result;
                        self.flags_zero = result == 0;
                    }
                }
                "MUL" => {
                    let dst = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let src = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if dst < 27 && src < 27 {
                        let result = self.registers[dst].value.wrapping_mul(self.registers[src].value);
                        self.registers[dst].value = result;
                        self.flags_zero = result == 0;
                    }
                }
                "PUSH" => {
                    let reg = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if reg < 27 {
                        self.stack.push(self.registers[reg].value);
                        self.stack_pointer += 1;
                    }
                }
                "POP" => {
                    let reg = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if reg < 27 {
                        if let Some(val) = self.stack.pop() {
                            self.registers[reg].value = val;
                            self.stack_pointer = self.stack_pointer.saturating_sub(1);
                        } else {
                            self.state = VmState::Error;
                            self.last_error = Some("Stack underflow".to_string());
                            return Err("Stack underflow".to_string());
                        }
                    }
                }
                "STORE" => {
                    let addr = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let reg = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if addr + 8 <= self.memory_size && reg < 27 {
                        let bytes = self.registers[reg].value.to_le_bytes();
                        self.memory[addr..addr + 8].copy_from_slice(&bytes);
                    }
                }
                "MLOAD" => {
                    let reg = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let addr = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if addr + 8 <= self.memory_size && reg < 27 {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr..addr + 8]);
                        self.registers[reg].value = i64::from_le_bytes(bytes);
                    }
                }
                "TMIN" => {
                    let dst = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let src = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if dst < 27 && src < 27 {
                        let a = self.registers[dst].value;
                        let b = self.registers[src].value;
                        let ta = ((a % 3) + 3) % 3;
                        let tb = ((b % 3) + 3) % 3;
                        self.registers[dst].value = ta.min(tb);
                    }
                }
                "TMAX" => {
                    let dst = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let src = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if dst < 27 && src < 27 {
                        let a = self.registers[dst].value;
                        let b = self.registers[src].value;
                        let ta = ((a % 3) + 3) % 3;
                        let tb = ((b % 3) + 3) % 3;
                        self.registers[dst].value = ta.max(tb);
                    }
                }
                "TCNS" => {
                    let dst = inst.operands.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let src = inst.operands.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    if dst < 27 && src < 27 {
                        let a = self.registers[dst].value;
                        let b = self.registers[src].value;
                        if a == b {
                            self.registers[dst].value = a;
                        } else {
                            let third = 3 - (((a % 3) + 3) % 3) - (((b % 3) + 3) % 3);
                            self.registers[dst].value = ((third % 3) + 3) % 3;
                        }
                    }
                }
                other => {
                    self.state = VmState::Error;
                    let msg = format!("Unknown opcode: {}", other);
                    self.last_error = Some(msg.clone());
                    return Err(msg);
                }
            }
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
}

#[derive(Debug, Clone, Serialize)]
pub struct VmFlagsResponse {
    pub halted: bool,
    pub overflow: bool,
    pub carry: bool,
    pub zero: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmRegistersResponse {
    pub registers: Vec<VmRegisterEntry>,
    pub program_counter: u64,
    pub stack_pointer: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmRegisterEntry {
    pub index: usize,
    pub value: i64,
    pub trit_width: u8,
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
        },
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
        })
        .collect();
    Json(VmRegistersResponse {
        registers: regs,
        program_counter: guard.program_counter,
        stack_pointer: guard.stack_pointer,
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
    }))
}

pub fn vm_router(vm: SharedVm) -> Router {
    Router::new()
        .route("/vm/exec", post(handle_vm_exec))
        .route("/vm/status", get(handle_vm_status))
        .route("/vm/registers", get(handle_vm_registers))
        .route("/vm/reset", post(handle_vm_reset))
        .with_state(vm)
}

pub const VM_ROUTE_COUNT: usize = 4;
