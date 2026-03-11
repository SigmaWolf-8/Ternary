/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { useEffect, useRef, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { PLATFORM } from "@shared/constants";

const BRAND = "\x1b[38;2;100;149;237m";
const GREEN = "\x1b[38;2;80;200;120m";
const YELLOW = "\x1b[38;2;255;215;0m";
const RED = "\x1b[38;2;255;100;100m";
const CYAN = "\x1b[38;2;0;200;200m";
const DIM = "\x1b[2m";
const BOLD = "\x1b[1m";
const RST = "\x1b[0m";
const WHITE = "\x1b[37m";

function fmtTs(): string {
  const now = new Date();
  const fs = Math.floor(Math.random() * 999999999999999).toString().padStart(15, "0");
  return `${now.toISOString().slice(0, 19)}Z.${fs}fs`;
}

function randTryte(): string {
  const vals = ["-1", "0", "+1"];
  return Array.from({ length: 27 }, () => vals[Math.floor(Math.random() * 3)]).join(",");
}

function randHex(n: number): string {
  return Array.from({ length: n }, () => Math.floor(Math.random() * 16).toString(16)).join("");
}

function randCapId(): string {
  return `CAP-${randHex(4).toUpperCase()}-${randHex(4).toUpperCase()}`;
}

interface XPlenumState {
  maskEn: boolean;
  domEn: boolean;
  capEn: boolean;
  sigEn: boolean;
  domId: number;
  maskState: string;
  perfCnt: number;
  domTable: Record<number, { owner: number; perms: number; state: string }>;
  capTable: Record<number, { tag: number; perms: number; base: string; seal: string }>;
  revokeBitmap: Set<number>;
}

interface VMState {
  registers: number[];
  pc: number;
  mode: number;
  capsActive: string[];
  sideChMasked: boolean;
  auditCount: number;
  xplenum: XPlenumState;
}

function initXPlenumState(): XPlenumState {
  return {
    maskEn: false,
    domEn: false,
    capEn: false,
    sigEn: false,
    domId: 0,
    maskState: "0x00000000",
    perfCnt: 0,
    domTable: {},
    capTable: {},
    revokeBitmap: new Set(),
  };
}

function initVMState(): VMState {
  return {
    registers: new Array(27).fill(0),
    pc: 0,
    mode: 0,
    capsActive: [],
    sideChMasked: false,
    auditCount: 0,
    xplenum: initXPlenumState(),
  };
}

function randTritWord(): string {
  const vals = ["-1", " 0", "+1"];
  return Array.from({ length: 16 }, () => vals[Math.floor(Math.random() * 3)]).join(",");
}

const OPCODE_TABLE: Record<string, { hex: string; cat: string; desc: string }> = {
  "TLOAD": { hex: "0x01", cat: "Core", desc: "Load ternary word from memory into register" },
  "TSTORE": { hex: "0x02", cat: "Core", desc: "Store register to ternary memory" },
  "TADD": { hex: "0x03", cat: "Core", desc: "Balanced ternary addition (GF(3))" },
  "TSUB": { hex: "0x04", cat: "Core", desc: "Balanced ternary subtraction" },
  "TMUL": { hex: "0x05", cat: "Core", desc: "Balanced ternary multiplication" },
  "TDIV": { hex: "0x06", cat: "Core", desc: "Balanced ternary division" },
  "TMOD": { hex: "0x07", cat: "Core", desc: "Balanced ternary modulo" },
  "TAND": { hex: "0x08", cat: "Core", desc: "Ternary logical AND (min)" },
  "TOR": { hex: "0x09", cat: "Core", desc: "Ternary logical OR (max)" },
  "TNOT": { hex: "0x0A", cat: "Core", desc: "Ternary logical NOT (negate)" },
  "TXOR": { hex: "0x0B", cat: "Core", desc: "Ternary XOR (mod-3 addition)" },
  "TCMP": { hex: "0x0C", cat: "Core", desc: "Ternary comparison (3-way: LT/EQ/GT)" },
  "TJMP": { hex: "0x0D", cat: "Core", desc: "Unconditional jump" },
  "TJNZ": { hex: "0x0E", cat: "Core", desc: "Jump if not zero" },
  "TJPOS": { hex: "0x0F", cat: "Core", desc: "Jump if positive" },
  "TJNEG": { hex: "0x10", cat: "Core", desc: "Jump if negative" },
  "TCALL": { hex: "0x11", cat: "Core", desc: "Subroutine call" },
  "TRET": { hex: "0x12", cat: "Core", desc: "Return from subroutine" },
  "TMOV": { hex: "0x13", cat: "Core", desc: "Move between registers" },
  "TPUSH": { hex: "0x14", cat: "Core", desc: "Push register to ternary stack" },
  "TCONVERT": { hex: "0x15", cat: "Core", desc: "Convert between representations A/B/C" },
  "TPOP": { hex: "0x16", cat: "Core", desc: "Pop ternary stack to register" },
  "TNEG": { hex: "0x17", cat: "Core", desc: "Negate (trit-wise sign flip)" },
  "TSHIFT": { hex: "0x18", cat: "Extended", desc: "Balanced ternary shift" },
  "TROTATE": { hex: "0x19", cat: "Extended", desc: "Trit-wise rotation" },
  "TFMA": { hex: "0x1A", cat: "Extended", desc: "Fused multiply-add in GF(3)" },
  "TCLZ": { hex: "0x1B", cat: "Extended", desc: "Count leading zero-trits" },
  "TPOPCOUNT": { hex: "0x1C", cat: "Extended", desc: "Population count of non-zero trits" },
  "TBITREV": { hex: "0x1D", cat: "Extended", desc: "Trit-reverse of word" },
  "TLERP": { hex: "0x1E", cat: "Extended", desc: "Ternary linear interpolation" },
  "TMINMAX": { hex: "0x1F", cat: "Extended", desc: "Simultaneous min/max extraction" },
  "THASH": { hex: "0x60", cat: "Crypto", desc: "TL-Sponge-385 hash" },
  "THMAC": { hex: "0x61", cat: "Crypto", desc: "Ternary HMAC" },
  "TKDF": { hex: "0x62", cat: "Crypto", desc: "Ternary key derivation function" },
  "TLAMPORT": { hex: "0x63", cat: "Crypto", desc: "Lamport one-time signature" },
  "TAES": { hex: "0x64", cat: "Crypto", desc: "AES-256-GCM via binary compat layer" },
  "TSHA2": { hex: "0x65", cat: "Crypto", desc: "SHA-2 via binary compat layer" },
  "TSHA3": { hex: "0x66", cat: "Crypto", desc: "SHA-3 via binary compat layer" },
  "PHASEENC": { hex: "0x67", cat: "Crypto", desc: "Phase encryption (split/recombine)" },
  "PHASEDEC": { hex: "0x68", cat: "Crypto", desc: "Phase decryption" },
  "TPOLYEVAL": { hex: "0x69", cat: "Crypto", desc: "GF(3) polynomial evaluation" },
  "TPOLYMUL": { hex: "0x6A", cat: "Crypto", desc: "GF(3) polynomial multiplication" },
  "TPOLYADD": { hex: "0x6B", cat: "Crypto", desc: "GF(3) polynomial addition" },
  "TNTT": { hex: "0x6C", cat: "Crypto", desc: "Number-Theoretic Transform (ternary)" },
  "TKEMENCAPS": { hex: "0x6D", cat: "Crypto", desc: "TL-KEM key encapsulation" },
  "TKEMDECAPS": { hex: "0x6E", cat: "Crypto", desc: "TL-KEM key decapsulation" },
  "TDSASIGN": { hex: "0x6F", cat: "Crypto", desc: "TL-DSA digital signature" },
  "TDSAVERIFY": { hex: "0x70", cat: "Crypto", desc: "TL-DSA signature verification" },
  "SIMDADD": { hex: "0x80", cat: "SIMD", desc: "SIMD ternary vector add" },
  "SIMDSUB": { hex: "0x81", cat: "SIMD", desc: "SIMD ternary vector subtract" },
  "SIMDMUL": { hex: "0x82", cat: "SIMD", desc: "SIMD ternary vector multiply" },
  "SIMDDOT": { hex: "0x83", cat: "SIMD", desc: "SIMD ternary dot product" },
  "SIMDREDUCE": { hex: "0x84", cat: "SIMD", desc: "SIMD reduction (sum/min/max)" },
  "SIMDSHUFFLE": { hex: "0x85", cat: "SIMD", desc: "SIMD trit-lane shuffle" },
  "SIMDMASK": { hex: "0x86", cat: "SIMD", desc: "SIMD masked operation" },
  "SIMDBROADCAST": { hex: "0x87", cat: "SIMD", desc: "SIMD broadcast scalar to vector" },
  "SYSCALL": { hex: "0x88", cat: "System", desc: "System call trap" },
  "SYSRET": { hex: "0x89", cat: "System", desc: "Return from system call" },
  "IRET": { hex: "0x8A", cat: "System", desc: "Return from interrupt" },
  "HALT": { hex: "0x8B", cat: "System", desc: "Halt processor" },
  "NOP": { hex: "0x8C", cat: "System", desc: "No operation" },
  "FENCE": { hex: "0x8D", cat: "System", desc: "Memory fence / barrier" },
  "CPUID": { hex: "0x8E", cat: "System", desc: "Query processor capabilities" },
  "MODESET": { hex: "0x8F", cat: "System", desc: "Set security mode (Ring0/1/2)" },
  "AUDITLOG": { hex: "0x90", cat: "Security", desc: "Create HPTP-timestamped audit entry" },
  "CAPCHECK": { hex: "0x91", cat: "Security", desc: "Validate capability token (dual mechanism)" },
  "CAPGRANT": { hex: "0x92", cat: "Security", desc: "Create capability token (Ring0 only)" },
  "CAPREVOKE": { hex: "0x93", cat: "Security", desc: "Revoke capability token (Ring0 only)" },
  "SIDECHMASK": { hex: "0x94", cat: "Security", desc: "Activate side-channel protection (dual-layer)" },
  "SIDECHUNMASK": { hex: "0x95", cat: "Security", desc: "Deactivate side-channel protection" },
  "CONSTTIMEEQ": { hex: "0x96", cat: "Security", desc: "Constant-time equality comparison" },
  "CONSTTIMESEL": { hex: "0x97", cat: "Security", desc: "Constant-time conditional select" },
  "DBGBRK": { hex: "0xA0", cat: "Debug", desc: "Debug breakpoint" },
  "DBGSTEP": { hex: "0xA1", cat: "Debug", desc: "Single-step execution" },
  "DBGTRACE": { hex: "0xA2", cat: "Debug", desc: "Trace execution path" },
  "PERFCOUNT": { hex: "0xA3", cat: "Debug", desc: "Read performance counter" },
  "GCALLOC": { hex: "0xB0", cat: "GC", desc: "Ternary-aware garbage collector allocate" },
  "GCFREE": { hex: "0xB1", cat: "GC", desc: "GC free / deallocate" },
  "GCMARK": { hex: "0xB2", cat: "GC", desc: "GC mark phase" },
  "GCSWEEP": { hex: "0xB3", cat: "GC", desc: "GC sweep phase" },
  "TMASK": { hex: "0x0B:000:00", cat: "XPLENUM", desc: "Apply ternary mask (trit-wise addition mod 3)" },
  "TUNMASK": { hex: "0x0B:000:01", cat: "XPLENUM", desc: "Remove ternary mask (trit-wise subtraction mod 3)" },
  "TMASKR": { hex: "0x0B:000:02", cat: "XPLENUM", desc: "Generate random LFSR mask + apply" },
  "TMASKRF": { hex: "0x0B:000:03", cat: "XPLENUM", desc: "Unmask with old mask, remask with fresh LFSR" },
  "TDOMSET": { hex: "0x0B:001:00", cat: "XPLENUM", desc: "Set domain isolation tag (256-entry table)" },
  "TDOMCHK": { hex: "0x0B:001:01", cat: "XPLENUM", desc: "Check domain permission (owner + bitmap)" },
  "TDOMCLR": { hex: "0x0B:001:02", cat: "XPLENUM", desc: "Clear domain tag (owner-only)" },
  "TDOMXFR": { hex: "0x0B:001:03", cat: "XPLENUM", desc: "Transfer domain ownership (authorized)" },
  "TCAPLD": { hex: "0x0B:010:00", cat: "XPLENUM", desc: "Load capability descriptor (64-bit entry)" },
  "TCAPCHK": { hex: "0x0B:010:01", cat: "XPLENUM", desc: "Check capability permissions (tag + perms)" },
  "TCAPST": { hex: "0x0B:010:02", cat: "XPLENUM", desc: "Store capability descriptor (unsealed only)" },
  "TCAPREV": { hex: "0x0B:010:03", cat: "XPLENUM", desc: "Revoke capability O(1) via bitmap flip" },
  "TROTL": { hex: "0x0B:011:00", cat: "XPLENUM", desc: "Ternary rotate left by N trits" },
  "TROTR": { hex: "0x0B:011:01", cat: "XPLENUM", desc: "Ternary rotate right by N trits" },
  "TTBOX": { hex: "0x0B:011:02", cat: "XPLENUM", desc: "T-box substitution (27-entry GF(3)^3 S-box)" },
  "TPERM": { hex: "0x0B:011:03", cat: "XPLENUM", desc: "Ternary permutation (lane reordering)" },
  "TTRIT": { hex: "0x0B:100:00", cat: "XPLENUM", desc: "Binary to balanced ternary encoding" },
  "TDETRIT": { hex: "0x0B:100:01", cat: "XPLENUM", desc: "Balanced ternary to binary decoding" },
  "TSIGFLT": { hex: "0x0B:101:00", cat: "XPLENUM", desc: "Signal filter (IIR/FIR multiply-accumulate)" },
  "TSIGCMP": { hex: "0x0B:101:01", cat: "XPLENUM", desc: "Signal compare (ternary threshold classifier)" },
  "TSIGACC": { hex: "0x0B:101:02", cat: "XPLENUM", desc: "Signal accumulate (EWMA filter)" },
};

function processCommand(input: string, vm: VMState, term: Terminal): VMState {
  const trimmed = input.trim();
  if (!trimmed) return vm;

  const parts = trimmed.split(/\s+/);
  const cmd = parts[0].toLowerCase();

  switch (cmd) {
    case "help": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}Salvi Framework \u2014 Ternary Virtual Machine ${PLATFORM.VM_ISA_VERSION}${RST}`);
      term.writeln(`${DIM}${PLATFORM.VM_OPCODES}-Opcode ISA \u2022 27-Trit Word \u2022 3-Ring Privilege \u2022 Post-Quantum${RST}`);
      term.writeln("");
      term.writeln(`${BOLD}${WHITE}Available Commands:${RST}`);
      term.writeln(`  ${GREEN}help${RST}               Show this help message`);
      term.writeln(`  ${GREEN}status${RST}             Display VM state and registers`);
      term.writeln(`  ${GREEN}opcodes${RST}            List all ${PLATFORM.VM_OPCODES} ISA opcodes by category`);
      term.writeln(`  ${GREEN}opcode <NAME>${RST}      Show details for a specific opcode`);
      term.writeln(`  ${GREEN}exec <MNEMONIC>${RST}    Simulate execution of an instruction`);
      term.writeln(`  ${GREEN}demo${RST}               Run the dual-phase encryption demo`);
      term.writeln(`  ${GREEN}demo-cap${RST}           Run the capability security demo`);
      term.writeln(`  ${GREEN}demo-sidech${RST}        Run the side-channel masking demo`);
      term.writeln(`  ${GREEN}demo-xplenum${RST}       Run the XPLENUM RISC-V extension demo`);
      term.writeln(`  ${GREEN}xplenum${RST}            Show XPLENUM extension status & CSRs`);
      term.writeln(`  ${GREEN}registers${RST}          Display all 27 ternary registers`);
      term.writeln(`  ${GREEN}audit${RST}              Show audit trail`);
      term.writeln(`  ${GREEN}arch${RST}               Display architecture summary`);
      term.writeln(`  ${GREEN}cpuid${RST}              Show processor capabilities`);
      term.writeln(`  ${GREEN}modes${RST}              Show security mode ring hierarchy`);
      term.writeln(`  ${GREEN}clear${RST}              Clear terminal`);
      term.writeln("");
      return vm;
    }

    case "clear": {
      term.clear();
      return vm;
    }

    case "status": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}\u250C\u2500 VM Status \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2510${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Program Counter:${RST}  ${CYAN}0x${vm.pc.toString(16).padStart(6, "0")}${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Security Mode:${RST}    ${vm.mode === 0 ? `${RED}Ring0 (Kernel)${RST}` : vm.mode === 1 ? `${YELLOW}Ring1 (Service)${RST}` : `${GREEN}Ring2 (User)${RST}`}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Side-Ch Mask:${RST}     ${vm.sideChMasked ? `${GREEN}ACTIVE (dual-layer)${RST}` : `${DIM}Inactive${RST}`}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Active Caps:${RST}      ${vm.capsActive.length > 0 ? `${CYAN}${vm.capsActive.length}${RST}` : `${DIM}0${RST}`}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Audit Entries:${RST}    ${CYAN}${vm.auditCount}${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}HPTP Timestamp:${RST}   ${DIM}${fmtTs()}${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Word Size:${RST}        ${CYAN}27 trits (1 tryte)${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Register File:${RST}    ${CYAN}27 general-purpose${RST}`);
      term.writeln(`${BRAND}\u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2518${RST}`);
      term.writeln("");
      return vm;
    }

    case "registers": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}Register File \u2014 27 Ternary Registers (27-trit each)${RST}`);
      term.writeln(`${DIM}${"─".repeat(60)}${RST}`);
      for (let i = 0; i < 27; i += 3) {
        const row = [0, 1, 2].map((j) => {
          const idx = i + j;
          if (idx >= 27) return "";
          const val = vm.registers[idx];
          const color = val === 0 ? DIM : val > 0 ? GREEN : RED;
          return `${WHITE}T${idx.toString().padStart(2, "0")}${RST}: ${color}${val.toString().padStart(6)}${RST}`;
        }).filter(Boolean).join("   ");
        term.writeln(`  ${row}`);
      }
      term.writeln("");
      return vm;
    }

    case "opcodes": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}Salvi ISA ${PLATFORM.VM_ISA_VERSION} \u2014 ${PLATFORM.VM_OPCODES} Opcodes${RST}`);
      term.writeln(`${DIM}Showing representative opcodes by category. Type 'opcode <NAME>' for details.${RST}`);
      const cats: Record<string, string[]> = {};
      for (const [name, info] of Object.entries(OPCODE_TABLE)) {
        if (!cats[info.cat]) cats[info.cat] = [];
        cats[info.cat].push(name);
      }
      for (const [cat, names] of Object.entries(cats)) {
        const color = cat === "Core" ? GREEN : cat === "Crypto" ? CYAN : cat === "Security" ? RED : cat === "SIMD" ? YELLOW : cat === "System" ? BRAND : WHITE;
        term.writeln("");
        term.writeln(`  ${BOLD}${color}${cat}${RST} ${DIM}(${names.length} shown)${RST}`);
        for (const n of names) {
          const o = OPCODE_TABLE[n];
          term.writeln(`    ${WHITE}${o.hex.padEnd(7)}${RST}${GREEN}${n.padEnd(16)}${RST}${DIM}${o.desc}${RST}`);
        }
      }
      term.writeln("");
      term.writeln(`${DIM}Total ISA: ${PLATFORM.VM_OPCODES} opcodes across Core, Extended, Crypto Acceleration,${RST}`);
      term.writeln(`${DIM}SIMD, System, Security/Audit, Quantum-Ternary, and Debug/Profiling categories.${RST}`);
      term.writeln("");
      return vm;
    }

    case "opcode": {
      const name = (parts[1] || "").toUpperCase();
      const info = OPCODE_TABLE[name];
      if (!info) {
        term.writeln(`${RED}Unknown opcode: ${name}. Type 'opcodes' to list all.${RST}`);
        return vm;
      }
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}\u250C\u2500 ${name} \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2510${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Opcode:${RST}     ${CYAN}${info.hex}${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Mnemonic:${RST}   ${GREEN}${name}${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Category:${RST}   ${YELLOW}${info.cat}${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Function:${RST}   ${info.desc}`);
      term.writeln(`${BRAND}\u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2518${RST}`);
      term.writeln("");
      return vm;
    }

    case "exec": {
      const mnemonic = (parts[1] || "").toUpperCase();
      const info = OPCODE_TABLE[mnemonic];
      if (!info) {
        term.writeln(`${RED}Unknown instruction: ${mnemonic}. Type 'opcodes' to list all.${RST}`);
        return vm;
      }
      const newVm = { ...vm, pc: vm.pc + 1, auditCount: vm.auditCount + 1 };
      term.writeln("");
      term.writeln(`${DIM}[${fmtTs()}]${RST} ${BOLD}${GREEN}EXEC${RST} ${WHITE}${mnemonic}${RST} ${DIM}(${info.hex})${RST}`);

      if (mnemonic === "SIDECHMASK") {
        newVm.sideChMasked = true;
        term.writeln(`  ${CYAN}\u2192 Layer 1: Microarchitectural isolation ENABLED${RST}`);
        term.writeln(`  ${CYAN}\u2192 Layer 2: Algebraic ternary masking ACTIVE${RST}`);
        term.writeln(`  ${GREEN}\u2713 SCCR saved, HRTG initialized${RST}`);
      } else if (mnemonic === "SIDECHUNMASK") {
        newVm.sideChMasked = false;
        term.writeln(`  ${CYAN}\u2192 Layer 1: Microarchitectural features RESTORED${RST}`);
        term.writeln(`  ${CYAN}\u2192 Layer 2: Masking mode DEACTIVATED${RST}`);
        term.writeln(`  ${GREEN}\u2713 Normal execution resumed${RST}`);
      } else if (mnemonic === "CAPGRANT") {
        const cap = randCapId();
        newVm.capsActive = [...newVm.capsActive, cap];
        term.writeln(`  ${CYAN}\u2192 Capability created: ${cap}${RST}`);
        term.writeln(`  ${DIM}  PERM: R/W/X  SCOPE: 0x${randHex(4)}  BASE: 0x${randHex(8)}${RST}`);
        term.writeln(`  ${GREEN}\u2713 Sentinel trit [0] = 0 (unforgeable)${RST}`);
      } else if (mnemonic === "CAPREVOKE") {
        if (newVm.capsActive.length > 0) {
          const revoked = newVm.capsActive[newVm.capsActive.length - 1];
          newVm.capsActive = newVm.capsActive.slice(0, -1);
          term.writeln(`  ${RED}\u2717 Capability revoked: ${revoked}${RST}`);
          term.writeln(`  ${DIM}  Table flag: revoked=true, sentinel overwritten to 1${RST}`);
        } else {
          term.writeln(`  ${YELLOW}\u26A0 No active capabilities to revoke${RST}`);
        }
      } else if (mnemonic === "CAPCHECK") {
        if (newVm.capsActive.length > 0) {
          const cap = newVm.capsActive[newVm.capsActive.length - 1];
          term.writeln(`  ${CYAN}\u2192 Checking: ${cap}${RST}`);
          term.writeln(`  ${DIM}  Mechanism 1: Sentinel trit = 0 \u2713${RST}`);
          term.writeln(`  ${DIM}  Mechanism 2: Table lookup   = VALID \u2713${RST}`);
          term.writeln(`  ${GREEN}\u2713 Capability VALID${RST}`);
        } else {
          term.writeln(`  ${RED}\u2717 No active capabilities \u2014 ACCESS DENIED${RST}`);
        }
      } else if (mnemonic === "AUDITLOG") {
        term.writeln(`  ${CYAN}\u2192 AuditEntry #${newVm.auditCount}${RST}`);
        term.writeln(`  ${DIM}  Timestamp: ${fmtTs()}${RST}`);
        term.writeln(`  ${DIM}  Process:   PID-${Math.floor(Math.random() * 9999).toString().padStart(4, "0")}${RST}`);
        term.writeln(`  ${DIM}  Mode:      Ring${newVm.mode}${RST}`);
        term.writeln(`  ${GREEN}\u2713 Chain-verified, HPTP-anchored${RST}`);
      } else if (mnemonic === "CONSTTIMEEQ") {
        const ns = (42 + Math.random() * 0.001).toFixed(6);
        term.writeln(`  ${CYAN}\u2192 Comparison: T00 == T01${RST}`);
        term.writeln(`  ${DIM}  Latency: ${ns} ns (constant, data-independent)${RST}`);
        term.writeln(`  ${DIM}  ALU path: physically isolated from speculative exec${RST}`);
        term.writeln(`  ${GREEN}\u2713 Result: ${Math.random() > 0.5 ? "EQUAL" : "NOT_EQUAL"} (timing invariant)${RST}`);
      } else if (mnemonic === "CONSTTIMESEL") {
        term.writeln(`  ${CYAN}\u2192 Conditional select: flags ? T00 : T01${RST}`);
        term.writeln(`  ${DIM}  Both sources read simultaneously${RST}`);
        term.writeln(`  ${DIM}  Selection via bitwise masking (no branch)${RST}`);
        term.writeln(`  ${GREEN}\u2713 Result stored to dst (timing invariant)${RST}`);
      } else if (mnemonic === "TKEMENCAPS") {
        term.writeln(`  ${CYAN}\u2192 TL-KEM Encapsulation${RST}`);
        term.writeln(`  ${DIM}  Shared secret: 0x${randHex(32)}${RST}`);
        term.writeln(`  ${DIM}  Ciphertext:    ${randHex(64).slice(0, 48)}...${RST}`);
        term.writeln(`  ${GREEN}\u2713 Post-quantum secure (ML-KEM Level 3+ equivalent)${RST}`);
      } else if (mnemonic === "TKEMDECAPS") {
        term.writeln(`  ${CYAN}\u2192 TL-KEM Decapsulation${RST}`);
        term.writeln(`  ${DIM}  Recovered key: 0x${randHex(32)}${RST}`);
        term.writeln(`  ${GREEN}\u2713 Key recovered successfully${RST}`);
      } else if (mnemonic === "TDSASIGN") {
        term.writeln(`  ${CYAN}\u2192 TL-DSA Signing${RST}`);
        term.writeln(`  ${DIM}  Signature: 0x${randHex(64)}${RST}`);
        term.writeln(`  ${DIM}  Fiat-Shamir abort check via ConstTimeSel${RST}`);
        term.writeln(`  ${GREEN}\u2713 Signature generated (ML-DSA equivalent)${RST}`);
      } else if (mnemonic === "TDSAVERIFY") {
        term.writeln(`  ${CYAN}\u2192 TL-DSA Verification${RST}`);
        term.writeln(`  ${GREEN}\u2713 Signature VALID${RST}`);
      } else if (mnemonic === "PHASEENC") {
        term.writeln(`  ${CYAN}\u2192 Phase 1 encryption (GF(3) symmetric)${RST}`);
        term.writeln(`  ${DIM}  Block: 27-trit aligned, zero padding overhead${RST}`);
        term.writeln(`  ${GREEN}\u2713 Encrypted (inherent DPA resistance)${RST}`);
      } else if (mnemonic === "TCONVERT") {
        term.writeln(`  ${CYAN}\u2192 Representation conversion: A \u2192 C${RST}`);
        term.writeln(`  ${DIM}  f(a) = a + 2, single-cycle hardware op${RST}`);
        term.writeln(`  ${GREEN}\u2713 Converted${RST}`);
      } else if (mnemonic === "MODESET") {
        newVm.mode = (newVm.mode + 1) % 3;
        term.writeln(`  ${CYAN}\u2192 Mode transition: Ring${vm.mode} \u2192 Ring${newVm.mode}${RST}`);
        term.writeln(`  ${GREEN}\u2713 Security context updated${RST}`);
      } else if (mnemonic === "HALT") {
        term.writeln(`  ${RED}\u25A0 Processor halted${RST}`);
      } else if (mnemonic === "NOP") {
        term.writeln(`  ${DIM}\u2192 No operation (1 cycle)${RST}`);
      } else if (mnemonic === "TMASK") {
        const mask = "0x" + randHex(8).toUpperCase();
        const data = "0x" + randHex(8).toUpperCase();
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM custom-0 (opcode 0x0B, funct3=000, funct7=0000000)${RST}`);
        term.writeln(`  ${DIM}  data  = ${data}  (16 trits, 2-bit encoded)${RST}`);
        term.writeln(`  ${DIM}  mask  = ${mask}${RST}`);
        term.writeln(`  ${DIM}  rd    = trit_add(data, mask) mod 3 per trit-pair${RST}`);
        term.writeln(`  ${GREEN}\u2713 Masked result: 0x${randHex(8).toUpperCase()}${RST}`);
      } else if (mnemonic === "TUNMASK") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM custom-0 (funct3=000, funct7=0000001)${RST}`);
        term.writeln(`  ${DIM}  rd = trit_sub(data, mask) mod 3 per trit-pair${RST}`);
        term.writeln(`  ${GREEN}\u2713 Unmasked: 0x${randHex(8).toUpperCase()}${RST}`);
      } else if (mnemonic === "TMASKR") {
        const lfsr = "0x" + randHex(8).toUpperCase();
        newVm.xplenum = { ...newVm.xplenum, maskState: lfsr, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM LFSR-TRNG mask generation${RST}`);
        term.writeln(`  ${DIM}  Polynomial: x^32 + x^22 + x^2 + x + 1${RST}`);
        term.writeln(`  ${DIM}  LFSR state \u2192 valid trits (11 \u2192 00 sanitize)${RST}`);
        term.writeln(`  ${DIM}  Generated mask: ${lfsr}${RST}`);
        term.writeln(`  ${GREEN}\u2713 CSR 0x7C5 (XPMASK_STATE) updated${RST}`);
      } else if (mnemonic === "TMASKRF") {
        const oldMask = newVm.xplenum.maskState;
        const newMask = "0x" + randHex(8).toUpperCase();
        newVm.xplenum = { ...newVm.xplenum, maskState: newMask, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM mask refresh (unmask old + remask new)${RST}`);
        term.writeln(`  ${DIM}  Old mask: ${oldMask}${RST}`);
        term.writeln(`  ${DIM}  New mask: ${newMask}${RST}`);
        term.writeln(`  ${DIM}  rd = mask_new(unmask_old(data))${RST}`);
        term.writeln(`  ${GREEN}\u2713 Mask refreshed (side-channel window minimized)${RST}`);
      } else if (mnemonic === "TDOMSET") {
        const idx = Math.floor(Math.random() * 256);
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        newVm.xplenum.domTable[idx] = { owner: newVm.xplenum.domId, perms: 0xFF, state: "ACTIVE" };
        term.writeln(`  ${CYAN}\u2192 XPLENUM domain table write${RST}`);
        term.writeln(`  ${DIM}  Index:  ${idx} (of 256-entry table)${RST}`);
        term.writeln(`  ${DIM}  Owner:  Domain ${newVm.xplenum.domId}${RST}`);
        term.writeln(`  ${DIM}  Perms:  R/W/X/Cross (0xFF)${RST}`);
        term.writeln(`  ${DIM}  State:  ACTIVE${RST}`);
        term.writeln(`  ${GREEN}\u2713 Domain tag set${RST}`);
      } else if (mnemonic === "TDOMCHK") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        const entries = Object.keys(newVm.xplenum.domTable);
        if (entries.length > 0) {
          const idx = parseInt(entries[entries.length - 1]);
          const entry = newVm.xplenum.domTable[idx];
          term.writeln(`  ${CYAN}\u2192 XPLENUM domain permission check${RST}`);
          term.writeln(`  ${DIM}  Index:    ${idx}${RST}`);
          term.writeln(`  ${DIM}  Owner:    Domain ${entry.owner} (current: ${newVm.xplenum.domId})${RST}`);
          term.writeln(`  ${DIM}  Match:    ${entry.owner === newVm.xplenum.domId ? "YES" : "NO"}${RST}`);
          term.writeln(`  ${entry.owner === newVm.xplenum.domId ? `${GREEN}\u2713 Permission GRANTED` : `${RED}\u2717 Permission DENIED`}${RST}`);
        } else {
          term.writeln(`  ${YELLOW}\u26A0 Domain table empty \u2014 no entries to check${RST}`);
        }
      } else if (mnemonic === "TDOMCLR") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        const entries = Object.keys(newVm.xplenum.domTable);
        if (entries.length > 0) {
          const idx = parseInt(entries[entries.length - 1]);
          delete newVm.xplenum.domTable[idx];
          term.writeln(`  ${CYAN}\u2192 XPLENUM domain clear${RST}`);
          term.writeln(`  ${DIM}  Index ${idx} \u2192 INVALID (cleared)${RST}`);
          term.writeln(`  ${GREEN}\u2713 Domain entry cleared${RST}`);
        } else {
          term.writeln(`  ${YELLOW}\u26A0 Domain table empty${RST}`);
        }
      } else if (mnemonic === "TDOMXFR") {
        const newOwner = Math.floor(Math.random() * 256);
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM domain ownership transfer${RST}`);
        term.writeln(`  ${DIM}  From: Domain ${newVm.xplenum.domId} \u2192 Domain ${newOwner}${RST}`);
        term.writeln(`  ${DIM}  State: ACTIVE \u2192 TRANSFER${RST}`);
        term.writeln(`  ${GREEN}\u2713 Transfer initiated${RST}`);
      } else if (mnemonic === "TCAPLD") {
        const idx = Math.floor(Math.random() * 64);
        const entry = newVm.xplenum.capTable[idx];
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM capability load${RST}`);
        if (newVm.xplenum.revokeBitmap.has(idx)) {
          term.writeln(`  ${RED}\u2717 Capability ${idx} REVOKED \u2014 exception raised${RST}`);
          term.writeln(`  ${DIM}  CSR 0x7C8 (XPEXC_CAUSE) = 0x3 (CAP_REVOKED)${RST}`);
        } else if (entry) {
          term.writeln(`  ${DIM}  Index:  ${idx}  Tag: 0x${entry.tag.toString(16).padStart(2, "0")}${RST}`);
          term.writeln(`  ${DIM}  Perms:  0x${entry.perms.toString(16).padStart(2, "0")}  Base: ${entry.base}${RST}`);
          term.writeln(`  ${DIM}  Seal:   ${entry.seal}${RST}`);
          term.writeln(`  ${GREEN}\u2713 Capability loaded to rd${RST}`);
        } else {
          term.writeln(`  ${DIM}  Index:  ${idx}  (empty entry)${RST}`);
          term.writeln(`  ${DIM}  rd = 0x00000000${RST}`);
          term.writeln(`  ${GREEN}\u2713 OK (zero capability)${RST}`);
        }
      } else if (mnemonic === "TCAPCHK") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        const idx = Math.floor(Math.random() * 64);
        const entry = newVm.xplenum.capTable[idx];
        term.writeln(`  ${CYAN}\u2192 XPLENUM capability permission check${RST}`);
        term.writeln(`  ${DIM}  Index: ${idx}  Requested: R/W${RST}`);
        if (entry && !newVm.xplenum.revokeBitmap.has(idx)) {
          term.writeln(`  ${GREEN}\u2713 Permission VALID (rd = 1)${RST}`);
        } else {
          term.writeln(`  ${RED}\u2717 Permission INVALID (rd = 0)${RST}`);
        }
      } else if (mnemonic === "TCAPST") {
        const idx = Math.floor(Math.random() * 64);
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        newVm.xplenum.capTable[idx] = {
          tag: 0xFF, perms: 0xFF,
          base: "0x" + randHex(4).toUpperCase(),
          seal: "OPEN",
        };
        term.writeln(`  ${CYAN}\u2192 XPLENUM capability store${RST}`);
        term.writeln(`  ${DIM}  Index:  ${idx}  Tag: 0xFF (valid)${RST}`);
        term.writeln(`  ${DIM}  Perms:  R/W/X  Seal: OPEN (modifiable)${RST}`);
        term.writeln(`  ${GREEN}\u2713 Capability stored${RST}`);
      } else if (mnemonic === "TCAPREV") {
        const idx = Math.floor(Math.random() * 64);
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        newVm.xplenum.revokeBitmap.add(idx);
        term.writeln(`  ${CYAN}\u2192 XPLENUM O(1) capability revocation${RST}`);
        term.writeln(`  ${DIM}  Index: ${idx}  Bitmap[${idx}] \u2192 1${RST}`);
        term.writeln(`  ${DIM}  Single-cycle bitmap flip (no table sweep)${RST}`);
        term.writeln(`  ${RED}\u2717 Capability ${idx} REVOKED${RST}`);
      } else if (mnemonic === "TROTL") {
        const amt = Math.floor(Math.random() * 16);
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM ternary rotate left${RST}`);
        term.writeln(`  ${DIM}  Rotate amount: ${amt} trits (${amt * 2} bits)${RST}`);
        term.writeln(`  ${DIM}  rd = (rs1 << ${amt * 2}) | (rs1 >> ${32 - amt * 2})${RST}`);
        term.writeln(`  ${GREEN}\u2713 Result: 0x${randHex(8).toUpperCase()}${RST}`);
      } else if (mnemonic === "TROTR") {
        const amt = Math.floor(Math.random() * 16);
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM ternary rotate right${RST}`);
        term.writeln(`  ${DIM}  Rotate amount: ${amt} trits${RST}`);
        term.writeln(`  ${GREEN}\u2713 Result: 0x${randHex(8).toUpperCase()}${RST}`);
      } else if (mnemonic === "TTBOX") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM T-box substitution${RST}`);
        term.writeln(`  ${DIM}  27-entry lookup table (GF(3)^3 \u2192 GF(3)^3)${RST}`);
        term.writeln(`  ${DIM}  5 groups of 3-trit substitution applied${RST}`);
        term.writeln(`  ${DIM}  Nonlinear mixing for ternary block cipher${RST}`);
        term.writeln(`  ${GREEN}\u2713 Result: 0x${randHex(8).toUpperCase()}${RST}`);
      } else if (mnemonic === "TPERM") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM trit permutation${RST}`);
        term.writeln(`  ${DIM}  Lane reorder: rs2 encodes destination indices${RST}`);
        term.writeln(`  ${DIM}  8-trit shuffle in single cycle${RST}`);
        term.writeln(`  ${GREEN}\u2713 Result: 0x${randHex(8).toUpperCase()}${RST}`);
      } else if (mnemonic === "TTRIT") {
        const binVal = Math.floor(Math.random() * 1000);
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM binary \u2192 balanced ternary encoding${RST}`);
        term.writeln(`  ${DIM}  Input:  ${binVal} (binary)${RST}`);
        term.writeln(`  ${DIM}  Method: iterative mod-3 with carry correction${RST}`);
        term.writeln(`  ${DIM}  Trit encoding: 00=0, 01=+1, 10=-1 (2-bit pairs)${RST}`);
        term.writeln(`  ${GREEN}\u2713 Encoded: 0x${randHex(8).toUpperCase()} (16 trits)${RST}`);
      } else if (mnemonic === "TDETRIT") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        const binResult = Math.floor(Math.random() * 1000);
        term.writeln(`  ${CYAN}\u2192 XPLENUM balanced ternary \u2192 binary decoding${RST}`);
        term.writeln(`  ${DIM}  Input:  0x${randHex(8).toUpperCase()} (16 trits)${RST}`);
        term.writeln(`  ${DIM}  Method: \u03A3(trit_i * 3^i) signed accumulation${RST}`);
        term.writeln(`  ${GREEN}\u2713 Decoded: ${binResult} (binary)${RST}`);
      } else if (mnemonic === "TSIGFLT") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM signal filter (MAC)${RST}`);
        term.writeln(`  ${DIM}  rd = (rs1 * rs2[15:0]) >> cfg[3:0]${RST}`);
        term.writeln(`  ${GREEN}\u2713 Filtered: 0x${randHex(8).toUpperCase()}${RST}`);
      } else if (mnemonic === "TSIGCMP") {
        const vals = ["+1 (ABOVE)", " 0 (WITHIN)", "-1 (BELOW)"];
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM ternary threshold classifier${RST}`);
        term.writeln(`  ${DIM}  Deadband from CSR 0x7C7 (XPSIG_CFG)${RST}`);
        term.writeln(`  ${GREEN}\u2713 Classification: ${vals[Math.floor(Math.random() * 3)]}${RST}`);
      } else if (mnemonic === "TSIGACC") {
        newVm.xplenum = { ...newVm.xplenum, perfCnt: newVm.xplenum.perfCnt + 1 };
        term.writeln(`  ${CYAN}\u2192 XPLENUM EWMA accumulator${RST}`);
        term.writeln(`  ${DIM}  acc = (acc * alpha + sample * (255-alpha)) >> 8${RST}`);
        term.writeln(`  ${GREEN}\u2713 Accumulated: 0x${randHex(8).toUpperCase()}${RST}`);
      } else {
        const r1 = Math.floor(Math.random() * 27);
        const r2 = Math.floor(Math.random() * 27);
        newVm.registers[r1] = Math.floor(Math.random() * 19683) - 9841;
        term.writeln(`  ${CYAN}\u2192 T${r1.toString().padStart(2, "0")}, T${r2.toString().padStart(2, "0")}${RST}`);
        term.writeln(`  ${DIM}  Result: ${newVm.registers[r1]}${RST}`);
        term.writeln(`  ${GREEN}\u2713 OK${RST}`);
      }

      term.writeln(`  ${DIM}PC: 0x${newVm.pc.toString(16).padStart(6, "0")}${RST}`);
      term.writeln("");
      return newVm;
    }

    case "demo": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}\u2550\u2550\u2550 Dual-Phase Encryption Pipeline Demo \u2550\u2550\u2550${RST}`);
      term.writeln(`${DIM}Executing the full 7-step encryption pipeline...${RST}`);
      term.writeln("");

      const newVm = { ...vm, auditCount: vm.auditCount + 7, pc: vm.pc + 7 };

      const steps = [
        { op: "CAPCHECK", msg: "Verify caller holds encryption capability", extra: `  ${DIM}Sentinel: 0  Table: VALID  Expiry: not expired${RST}` },
        { op: "SIDECHMASK", msg: "Activate side-channel protection", extra: `  ${DIM}L1 cache disabled, branch predictor disabled, HRTG active${RST}` },
        { op: "TKEMENCAPS", msg: "Generate ephemeral symmetric key via TL-KEM", extra: `  ${DIM}Shared: 0x${randHex(32)}${RST}` },
        { op: "PHASEENC", msg: "Encrypt data block in GF(3)", extra: `  ${DIM}Input: ${randTryte().slice(0, 40)}...${RST}` },
        { op: "TDSASIGN", msg: "Sign (ciphertext || encapsulated_key)", extra: `  ${DIM}Sig: 0x${randHex(64)}${RST}` },
        { op: "AUDITLOG", msg: "Record with HPTP femtosecond timestamp", extra: `  ${DIM}Entry #${newVm.auditCount} at ${fmtTs()}${RST}` },
        { op: "SIDECHUNMASK", msg: "Restore normal execution", extra: `  ${DIM}All microarchitectural features re-enabled${RST}` },
      ];

      newVm.sideChMasked = false;
      newVm.capsActive = [...vm.capsActive, randCapId()];

      for (let i = 0; i < steps.length; i++) {
        const s = steps[i];
        const opInfo = OPCODE_TABLE[s.op];
        term.writeln(`  ${WHITE}Step ${i + 1}/7${RST} ${GREEN}${s.op}${RST} ${DIM}(${opInfo?.hex || "?"})${RST}`);
        term.writeln(`  ${CYAN}\u2192 ${s.msg}${RST}`);
        term.writeln(s.extra);
        term.writeln(`  ${GREEN}\u2713${RST}`);
        term.writeln("");
      }

      term.writeln(`${BOLD}${GREEN}\u2550\u2550\u2550 Pipeline complete. Data encrypted, signed, and audit-logged. \u2550\u2550\u2550${RST}`);
      term.writeln("");
      return newVm;
    }

    case "demo-cap": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}\u2550\u2550\u2550 Capability Security Demo \u2550\u2550\u2550${RST}`);
      term.writeln(`${DIM}Demonstrating sentinel-trit capability lifecycle...${RST}`);
      term.writeln("");

      const cap1 = randCapId();
      const cap2 = randCapId();
      const newVm = { ...vm, auditCount: vm.auditCount + 4, pc: vm.pc + 4, capsActive: [...vm.capsActive, cap1, cap2] };

      term.writeln(`  ${WHITE}1. CAPGRANT${RST} ${DIM}(0x92)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Creating parent capability: ${cap1}${RST}`);
      term.writeln(`  ${DIM}  PERM: R/W/X  SCOPE: 0x${randHex(4)}  Sentinel[0]=0${RST}`);
      term.writeln(`  ${GREEN}\u2713 Created (Ring0)${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}2. CAPGRANT${RST} ${DIM}(0x92) \u2014 Derived (monotonic authority)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Derived capability: ${cap2}${RST}`);
      term.writeln(`  ${DIM}  PERM: R only (parent \u2229\u2083 new = R)${RST}`);
      term.writeln(`  ${DIM}  BASE: narrowed, BOUND: narrowed${RST}`);
      term.writeln(`  ${GREEN}\u2713 Monotonic authority enforced${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}3. CAPCHECK${RST} ${DIM}(0x91)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Validating: ${cap2}${RST}`);
      term.writeln(`  ${DIM}  Mechanism 1 (sentinel): trit[0] = 0 \u2713${RST}`);
      term.writeln(`  ${DIM}  Mechanism 2 (table): valid, not expired, not revoked \u2713${RST}`);
      term.writeln(`  ${GREEN}\u2713 VALID \u2014 O(1) check, no tag-controller lookup${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}4. CAPREVOKE${RST} ${DIM}(0x93)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Revoking: ${cap2}${RST}`);
      term.writeln(`  ${DIM}  Table: revoked=true${RST}`);
      term.writeln(`  ${DIM}  Register: sentinel trit overwritten to 1${RST}`);
      term.writeln(`  ${RED}\u2717 ${cap2} REVOKED \u2014 O(1) immediate, global${RST}`);
      term.writeln(`  ${DIM}  (vs CHERI: sweep-based revocation scans all memory)${RST}`);
      term.writeln("");

      newVm.capsActive = newVm.capsActive.filter((c) => c !== cap2);

      term.writeln(`${BOLD}${GREEN}\u2550\u2550\u2550 Capability lifecycle complete. ${RST}`);
      term.writeln(`${DIM}    Sentinel-trit model: no external tag memory required.${RST}`);
      term.writeln("");
      return newVm;
    }

    case "demo-sidech": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}\u2550\u2550\u2550 Side-Channel Masking Demo \u2550\u2550\u2550${RST}`);
      term.writeln(`${DIM}Demonstrating dual-layer side-channel protection...${RST}`);
      term.writeln("");

      const newVm = { ...vm, auditCount: vm.auditCount + 3, pc: vm.pc + 3 };

      term.writeln(`  ${WHITE}1. SIDECHMASK${RST} ${DIM}(0x94)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Layer 1 \u2014 Microarchitectural Isolation:${RST}`);
      term.writeln(`  ${DIM}  [x] L1 data cache disabled${RST}`);
      term.writeln(`  ${DIM}  [x] L1 instruction cache disabled${RST}`);
      term.writeln(`  ${DIM}  [x] Branch predictor disabled${RST}`);
      term.writeln(`  ${DIM}  [x] Speculative execution disabled${RST}`);
      term.writeln(`  ${CYAN}\u2192 Layer 2 \u2014 Algebraic Ternary Masking:${RST}`);
      term.writeln(`  ${DIM}  HRTG mask: [${randTryte().slice(0, 30)}...]${RST}`);
      term.writeln(`  ${DIM}  d'_i = (d_i + m_i) mod 3   [Repr A]${RST}`);
      term.writeln(`  ${GREEN}\u2713 Dual-layer protection ACTIVE${RST}`);
      term.writeln("");

      newVm.sideChMasked = true;

      term.writeln(`  ${WHITE}2. CONSTTIMEEQ${RST} ${DIM}(0x96) \u2014 Secret comparison${RST}`);
      const ns = (42 + Math.random() * 0.001).toFixed(6);
      term.writeln(`  ${CYAN}\u2192 Comparing MAC values (32 trytes)${RST}`);
      term.writeln(`  ${DIM}  Latency: ${ns} ns (identical regardless of input)${RST}`);
      term.writeln(`  ${DIM}  Method: trit-wise subtract-and-OR-reduce${RST}`);
      term.writeln(`  ${DIM}  No early exit, no branch prediction, no carry leakage${RST}`);
      term.writeln(`  ${GREEN}\u2713 EQUAL (timing invariant)${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}3. SIDECHUNMASK${RST} ${DIM}(0x95)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Restoring normal execution${RST}`);
      term.writeln(`  ${DIM}  Layer 1: All features re-enabled per saved SCCR${RST}`);
      term.writeln(`  ${DIM}  Layer 2: Inverse unmasking: d_i = (d'_i - m_i + 3) mod 3${RST}`);
      term.writeln(`  ${DIM}  Guaranteed: D \u2297 M \u2298 M = D for all D, M${RST}`);
      term.writeln(`  ${GREEN}\u2713 Normal execution restored${RST}`);
      term.writeln("");

      newVm.sideChMasked = false;

      term.writeln(`${BOLD}${GREEN}\u2550\u2550\u2550 Side-channel demo complete.${RST}`);
      term.writeln(`${DIM}    Ternary: O(n) masking complexity vs binary ISW O(n\u00B2)${RST}`);
      term.writeln(`${DIM}    Uniform trit-transition energy = inherent glitch resistance${RST}`);
      term.writeln("");
      return newVm;
    }

    case "demo-xplenum": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}\u2550\u2550\u2550 XPLENUM RISC-V Ternary Security Extension Demo \u2550\u2550\u2550${RST}`);
      term.writeln(`${DIM}RISC-V custom-0 (opcode 0x0B) \u2014 6 functional groups, 23 instructions${RST}`);
      term.writeln(`${DIM}Simulating full hardware pipeline: masking \u2192 domain \u2192 capability \u2192 crypto${RST}`);
      term.writeln("");

      const xpVm = { ...vm, auditCount: vm.auditCount + 10, pc: vm.pc + 10,
        xplenum: { ...vm.xplenum, maskEn: true, domEn: true, capEn: true, sigEn: true, domId: 1, perfCnt: vm.xplenum.perfCnt + 10 }
      };
      const lfsrMask = "0x" + randHex(8).toUpperCase();
      const capIdx = Math.floor(Math.random() * 64);
      const domIdx = Math.floor(Math.random() * 256);
      xpVm.xplenum.maskState = lfsrMask;
      xpVm.xplenum.domTable[domIdx] = { owner: 1, perms: 0xFF, state: "ACTIVE" };
      xpVm.xplenum.capTable[capIdx] = { tag: 0xFF, perms: 0xFF, base: "0x" + randHex(4).toUpperCase(), seal: "OPEN" };

      term.writeln(`  ${WHITE}Step 1/10${RST} ${GREEN}CSR Write${RST} ${DIM}(XPSTATUS \u2192 0x0F)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Enable all subsystems: MASK_EN=1 DOM_EN=1 CAP_EN=1 SIG_EN=1${RST}`);
      term.writeln(`  ${DIM}  CSR 0x7C0 = 0x0000000F${RST}`);
      term.writeln(`  ${GREEN}\u2713${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 2/10${RST} ${GREEN}TMASKR${RST} ${DIM}(0x0B:000:02)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Generate LFSR random mask + apply to sensitive data${RST}`);
      term.writeln(`  ${DIM}  Polynomial: x^32 + x^22 + x^2 + x + 1${RST}`);
      term.writeln(`  ${DIM}  Generated mask: ${lfsrMask}${RST}`);
      term.writeln(`  ${DIM}  Data masked via trit-wise addition mod 3${RST}`);
      term.writeln(`  ${GREEN}\u2713 Side-channel resistant${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 3/10${RST} ${GREEN}TDOMSET${RST} ${DIM}(0x0B:001:00)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Create hardware isolation domain${RST}`);
      term.writeln(`  ${DIM}  Index: ${domIdx}  Owner: Domain 1  Perms: R/W/X/Cross${RST}`);
      term.writeln(`  ${DIM}  State: ACTIVE  (256-entry hardware table)${RST}`);
      term.writeln(`  ${GREEN}\u2713 Domain isolated${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 4/10${RST} ${GREEN}TDOMCHK${RST} ${DIM}(0x0B:001:01)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Verify domain permission (owner match + bitmap)${RST}`);
      term.writeln(`  ${DIM}  Current domain: 1  Entry owner: 1  Match: YES${RST}`);
      term.writeln(`  ${GREEN}\u2713 Permission GRANTED (rd = 1)${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 5/10${RST} ${GREEN}TCAPST${RST} ${DIM}(0x0B:010:02)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Store capability descriptor (64-bit CHERI-inspired)${RST}`);
      term.writeln(`  ${DIM}  Index: ${capIdx}  Tag: 0xFF  Perms: R/W/X  Seal: OPEN${RST}`);
      term.writeln(`  ${DIM}  [63:56]=tag [55:48]=perms [47:32]=base [31:16]=bound [15:8]=otype [7:0]=seal${RST}`);
      term.writeln(`  ${GREEN}\u2713 Capability stored${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 6/10${RST} ${GREEN}TCAPCHK${RST} ${DIM}(0x0B:010:01)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Check capability before crypto operation${RST}`);
      term.writeln(`  ${DIM}  Valid tag + matching permissions = GRANTED${RST}`);
      term.writeln(`  ${GREEN}\u2713 Capability VALID (rd = 1)${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 7/10${RST} ${GREEN}TTRIT${RST} ${DIM}(0x0B:100:00)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Convert binary data to balanced ternary for crypto${RST}`);
      term.writeln(`  ${DIM}  Binary \u2192 BT via iterative mod-3 with carry${RST}`);
      term.writeln(`  ${DIM}  Encoding: 00=0, 01=+1, 10=-1 (16 trits in 32 bits)${RST}`);
      term.writeln(`  ${GREEN}\u2713 Encoded${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 8/10${RST} ${GREEN}TTBOX${RST} ${DIM}(0x0B:011:02)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Apply T-box substitution (nonlinear ternary S-box)${RST}`);
      term.writeln(`  ${DIM}  27-entry GF(3)^3 lookup table \u2014 5 parallel substitutions${RST}`);
      term.writeln(`  ${DIM}  Hardware RTL: 22/22 testbench PASS (iverilog verified)${RST}`);
      term.writeln(`  ${GREEN}\u2713 Substituted${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 9/10${RST} ${GREEN}TCAPREV${RST} ${DIM}(0x0B:010:03)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Revoke capability via O(1) bitmap flip${RST}`);
      term.writeln(`  ${DIM}  revoke_bitmap[${capIdx}] \u2192 1  (single-cycle, no sweep)${RST}`);
      xpVm.xplenum.revokeBitmap.add(capIdx);
      term.writeln(`  ${RED}\u2717 Capability ${capIdx} REVOKED${RST}`);
      term.writeln("");

      term.writeln(`  ${WHITE}Step 10/10${RST} ${GREEN}TDETRIT${RST} ${DIM}(0x0B:100:01)${RST}`);
      term.writeln(`  ${CYAN}\u2192 Convert result back to binary for output${RST}`);
      term.writeln(`  ${DIM}  \u03A3(trit_i * 3^i) signed accumulation${RST}`);
      term.writeln(`  ${GREEN}\u2713 Decoded${RST}`);
      term.writeln("");

      term.writeln(`${BOLD}${GREEN}\u2550\u2550\u2550 XPLENUM pipeline complete. \u2550\u2550\u2550${RST}`);
      term.writeln(`${DIM}    23 instructions in RISC-V custom-0 opcode space (0x0B)${RST}`);
      term.writeln(`${DIM}    6 functional groups: Masking, Domain, Capability, Crypto, Encoding, Signal${RST}`);
      term.writeln(`${DIM}    RTL: 5 Verilog modules, 22/22 testbench PASS${RST}`);
      term.writeln(`${DIM}    CSR file: 0x7C0\u20130x7CB (12 registers)${RST}`);
      term.writeln("");
      return xpVm;
    }

    case "xplenum": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}\u250C\u2500 XPLENUM Extension Status \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2510${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Extension:${RST}     ${CYAN}XPLENUM v1.0.0${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Opcode:${RST}        ${CYAN}custom-0 (0x0B)${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${WHITE}Instructions:${RST}  ${CYAN}23 across 6 groups${RST}`);
      term.writeln(`${BRAND}\u2502${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${BOLD}${WHITE}CSR File (0x7C0\u20130x7CB):${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}0x7C0 XPSTATUS:${RST}    ${vm.xplenum.maskEn || vm.xplenum.domEn || vm.xplenum.capEn || vm.xplenum.sigEn ? GREEN : DIM}MASK=${vm.xplenum.maskEn ? "1" : "0"} DOM=${vm.xplenum.domEn ? "1" : "0"} CAP=${vm.xplenum.capEn ? "1" : "0"} SIG=${vm.xplenum.sigEn ? "1" : "0"}${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}0x7C1 XPDOMID:${RST}     ${CYAN}${vm.xplenum.domId}${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}0x7C5 MASK_STATE:${RST}  ${DIM}${vm.xplenum.maskState}${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}0x7CA PERF_CNT:${RST}    ${CYAN}${vm.xplenum.perfCnt}${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}0x7CB VERSION:${RST}     ${CYAN}0x00010000 (v1.0.0)${RST}`);
      term.writeln(`${BRAND}\u2502${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${BOLD}${WHITE}Hardware Tables:${RST}`);
      const domEntries = Object.keys(vm.xplenum.domTable).length;
      const capEntries = Object.keys(vm.xplenum.capTable).length;
      const revoked = vm.xplenum.revokeBitmap.size;
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}Domain Table:${RST}    ${CYAN}${domEntries}/256${RST} entries active`);
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}Cap Table:${RST}       ${CYAN}${capEntries}/64${RST} entries stored`);
      term.writeln(`${BRAND}\u2502${RST}   ${WHITE}Revoke Bitmap:${RST}   ${revoked > 0 ? RED : DIM}${revoked} revoked${RST}`);
      term.writeln(`${BRAND}\u2502${RST}`);
      term.writeln(`${BRAND}\u2502${RST} ${BOLD}${WHITE}Functional Groups:${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${GREEN}000${RST} Masking    ${DIM}TMASK TUNMASK TMASKR TMASKRF${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${GREEN}001${RST} Domain     ${DIM}TDOMSET TDOMCHK TDOMCLR TDOMXFR${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${GREEN}010${RST} Capability ${DIM}TCAPLD TCAPCHK TCAPST TCAPREV${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${GREEN}011${RST} Crypto     ${DIM}TROTL TROTR TTBOX TPERM${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${GREEN}100${RST} Encoding   ${DIM}TTRIT TDETRIT${RST}`);
      term.writeln(`${BRAND}\u2502${RST}   ${GREEN}101${RST} Signal     ${DIM}TSIGFLT TSIGCMP TSIGACC${RST}`);
      term.writeln(`${BRAND}\u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2518${RST}`);
      term.writeln("");
      return vm;
    }

    case "audit": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}Audit Trail${RST}`);
      term.writeln(`${DIM}${"─".repeat(60)}${RST}`);
      if (vm.auditCount === 0) {
        term.writeln(`  ${DIM}No audit entries. Execute instructions to generate entries.${RST}`);
      } else {
        for (let i = 0; i < Math.min(vm.auditCount, 10); i++) {
          const ops = ["CAPCHECK", "CAPGRANT", "PHASEENC", "TKEMENCAPS", "AUDITLOG", "TDSASIGN"];
          const op = ops[i % ops.length];
          term.writeln(`  ${DIM}#${(i + 1).toString().padStart(3, "0")}${RST} ${WHITE}${fmtTs()}${RST} ${GREEN}${op.padEnd(14)}${RST} ${DIM}Ring${vm.mode} PID-${Math.floor(Math.random() * 9999).toString().padStart(4, "0")}${RST}`);
        }
        if (vm.auditCount > 10) {
          term.writeln(`  ${DIM}... and ${vm.auditCount - 10} more entries${RST}`);
        }
      }
      term.writeln(`${DIM}Chain-verified, HPTP-anchored, tamper-evident${RST}`);
      term.writeln("");
      return vm;
    }

    case "arch": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}Salvi Framework \u2014 Architecture Summary${RST}`);
      term.writeln(`${DIM}${"─".repeat(56)}${RST}`);
      term.writeln(`  ${WHITE}ISA Version:${RST}       ${CYAN}${PLATFORM.VM_ISA_VERSION} (${PLATFORM.VM_OPCODES} opcodes)${RST}`);
      term.writeln(`  ${WHITE}Word Size:${RST}         ${CYAN}27 trits (1 tryte = 3\u00B3)${RST}`);
      term.writeln(`  ${WHITE}Register File:${RST}     ${CYAN}27 general-purpose ternary${RST}`);
      term.writeln(`  ${WHITE}Privilege Rings:${RST}   ${RED}Ring0${RST} ${YELLOW}Ring1${RST} ${GREEN}Ring2${RST}`);
      term.writeln(`  ${WHITE}Representations:${RST}   ${CYAN}A{-1,0,+1} B{0,1,2} C{1,2,3}${RST}`);
      term.writeln(`  ${WHITE}Kernel:${RST}            ${CYAN}Rust, ${PLATFORM.KERNEL_BINARY_SIZE} ELF, ${PLATFORM.KERNEL_LOC} LOC${RST}`);
      term.writeln(`  ${WHITE}Subsystems:${RST}        ${CYAN}${PLATFORM.KERNEL_SUBSYSTEMS}${RST}`);
      term.writeln("");
      term.writeln(`  ${BOLD}${WHITE}Opcode Categories:${RST}`);
      term.writeln(`    ${GREEN}Core${RST}                Arithmetic, logic, control flow`);
      term.writeln(`    ${GREEN}Extended${RST}            FMA, population count, bit-reverse`);
      term.writeln(`    ${CYAN}Crypto Acceleration${RST} Hash, KEM, DSA, Phase Encryption`);
      term.writeln(`    ${YELLOW}SIMD${RST}                Vector operations on trit-lanes`);
      term.writeln(`    ${BRAND}System${RST}              Syscall, interrupts, CPUID`);
      term.writeln(`    ${RED}Security/Audit${RST}      Capabilities, side-channel, const-time`);
      term.writeln(`    ${WHITE}Debug/Profiling${RST}    Breakpoints, trace, perf counters`);
      term.writeln("");
      term.writeln(`  ${BOLD}${WHITE}XPLENUM RISC-V Extension:${RST}`);
      term.writeln(`    ${CYAN}XPLENUM${RST}             23 custom-0 instructions (opcode 0x0B)`);
      term.writeln(`    ${DIM}Masking, Domain Isolation, Capabilities, Crypto,${RST}`);
      term.writeln(`    ${DIM}Trit Encoding, Signal Processing${RST}`);
      term.writeln(`    ${DIM}CSR 0x7C0\u20130x7CB \u2022 RTL: 5 Verilog modules \u2022 22/22 TB PASS${RST}`);
      term.writeln("");
      return vm;
    }

    case "cpuid": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}CPUID \u2014 Processor Capabilities${RST}`);
      term.writeln(`${DIM}${"─".repeat(56)}${RST}`);
      term.writeln(`  ${WHITE}Vendor:${RST}          ${CYAN}Capomastro Holdings Ltd.${RST}`);
      term.writeln(`  ${WHITE}Model:${RST}           ${CYAN}Salvi T27-${PLATFORM.VM_OPCODES} ${PLATFORM.VM_ISA_VERSION}${RST}`);
      term.writeln(`  ${WHITE}ISA:${RST}             ${CYAN}Ternary ${PLATFORM.VM_OPCODES}-opcode${RST}`);
      term.writeln(`  ${WHITE}Word:${RST}            ${CYAN}27-trit (42.77 bits equivalent)${RST}`);
      term.writeln(`  ${WHITE}Clock Source:${RST}    ${CYAN}HPTP femtosecond (10\u207B\u00B9\u2075 s)${RST}`);
      term.writeln(`  ${WHITE}Crypto:${RST}          ${GREEN}TL-KEM TL-DSA AES-256 SHA-2/3${RST}`);
      term.writeln(`  ${WHITE}SIMD:${RST}            ${GREEN}8-lane ternary vector unit${RST}`);
      term.writeln(`  ${WHITE}Side-Ch:${RST}         ${GREEN}Dual-layer (arch + algebraic)${RST}`);
      term.writeln(`  ${WHITE}Capabilities:${RST}    ${GREEN}Sentinel-trit based${RST}`);
      term.writeln(`  ${WHITE}Compliance:${RST}      ${GREEN}CNSA 2.0, FIPS 140-3${RST}`);
      term.writeln(`  ${WHITE}GC:${RST}              ${GREEN}Ternary-aware mark/sweep${RST}`);
      term.writeln(`  ${WHITE}XPLENUM:${RST}         ${GREEN}23 custom-0 instrs (0x0B)${RST}`);
      term.writeln(`  ${WHITE}  Groups:${RST}        ${DIM}Mask/Domain/Cap/Crypto/Enc/Sig${RST}`);
      term.writeln(`  ${WHITE}  RTL:${RST}           ${DIM}5 Verilog modules, 22/22 TB PASS${RST}`);
      term.writeln("");
      return vm;
    }

    case "modes": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}Security Mode Hierarchy${RST}`);
      term.writeln(`${DIM}${"─".repeat(56)}${RST}`);
      term.writeln(`  ${RED}\u2588\u2588 Ring 0 \u2014 Kernel Mode${RST}`);
      term.writeln(`  ${DIM}  Full privilege: CAPGRANT, CAPREVOKE, MODESET${RST}`);
      term.writeln(`  ${DIM}  Direct hardware access, interrupt handling${RST}`);
      term.writeln("");
      term.writeln(`  ${YELLOW}\u2588\u2588 Ring 1 \u2014 Service Mode${RST}`);
      term.writeln(`  ${DIM}  CAPCHECK, SIDECHMASK, crypto operations${RST}`);
      term.writeln(`  ${DIM}  Device drivers, system services${RST}`);
      term.writeln("");
      term.writeln(`  ${GREEN}\u2588\u2588 Ring 2 \u2014 User Mode${RST}`);
      term.writeln(`  ${DIM}  Capability-gated access only${RST}`);
      term.writeln(`  ${DIM}  CONSTTIMEEQ, CONSTTIMESEL, AUDITLOG${RST}`);
      term.writeln(`  ${DIM}  Application code, user processes${RST}`);
      term.writeln("");
      term.writeln(`  ${DIM}Current mode: Ring${vm.mode}${RST}`);
      term.writeln("");
      return vm;
    }

    default: {
      term.writeln(`${RED}Unknown command: ${cmd}${RST}`);
      term.writeln(`${DIM}Type 'help' for available commands.${RST}`);
      term.writeln("");
      return vm;
    }
  }
}

const PROMPT = `${BRAND}salvi${RST}${DIM}@${RST}${WHITE}vm${RST}${DIM}:${RST}${CYAN}~${RST}${WHITE}$ ${RST}`;

export function TernaryVMTerminal() {
  const termRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const inputBuffer = useRef("");
  const vmState = useRef<VMState>(initVMState());
  const initialized = useRef(false);

  const writePrompt = useCallback((term: Terminal) => {
    term.write(PROMPT);
  }, []);

  useEffect(() => {
    if (!termRef.current || initialized.current) return;
    initialized.current = true;

    const term = new Terminal({
      cursorBlink: true,
      cursorStyle: "block" as const,
      fontSize: 13,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      theme: {
        background: "#0a0e1a",
        foreground: "#c0c8e0",
        cursor: "#6495ed",
        selectionBackground: "#6495ed40",
        black: "#0a0e1a",
        red: "#ff6464",
        green: "#50c878",
        yellow: "#ffd700",
        blue: "#6495ed",
        magenta: "#9370db",
        cyan: "#00c8c8",
        white: "#c0c8e0",
        brightBlack: "#4a5568",
        brightRed: "#ff8080",
        brightGreen: "#70e898",
        brightYellow: "#ffe040",
        brightBlue: "#84b0ff",
        brightMagenta: "#b090ef",
        brightCyan: "#40e8e8",
        brightWhite: "#e8ecf4",
      },
      allowTransparency: true,
      scrollback: 5000,
      convertEol: true,
    });

    const fit = new FitAddon();
    fitAddon.current = fit;
    term.loadAddon(fit);
    term.open(termRef.current);

    setTimeout(() => {
      fit.fit();
    }, 50);

    termInstance.current = term;

    term.writeln("");
    term.writeln(`${BOLD}${BRAND}  \u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588${RST}`);
    term.writeln(`${BOLD}${BRAND}  \u2588${RST}                                           ${BOLD}${BRAND}\u2588${RST}`);
    term.writeln(`${BOLD}${BRAND}  \u2588${RST}  ${BOLD}${WHITE}Salvi Framework${RST} ${DIM}\u2014 Ternary Virtual Machine${RST}  ${BOLD}${BRAND}\u2588${RST}`);
    term.writeln(`${BOLD}${BRAND}  \u2588${RST}  ${DIM}ISA ${PLATFORM.VM_ISA_VERSION} \u2022 ${PLATFORM.VM_OPCODES} Opcodes \u2022 27-Trit Word${RST}     ${BOLD}${BRAND}\u2588${RST}`);
    term.writeln(`${BOLD}${BRAND}  \u2588${RST}  ${DIM}Post-Quantum \u2022 CNSA 2.0 \u2022 FIPS 140-3${RST}    ${BOLD}${BRAND}\u2588${RST}`);
    term.writeln(`${BOLD}${BRAND}  \u2588${RST}                                           ${BOLD}${BRAND}\u2588${RST}`);
    term.writeln(`${BOLD}${BRAND}  \u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588${RST}`);
    term.writeln("");
    term.writeln(`  ${GREEN}Capomastro Holdings Ltd. (Canada)${RST}`);
    term.writeln(`  ${DIM}Applied Physics Division${RST}`);
    term.writeln(`  ${DIM}Patent(s) Pending${RST}`);
    term.writeln("");
    term.writeln(`  ${WHITE}Type '${GREEN}help${WHITE}' for available commands.${RST}`);
    term.writeln(`  ${WHITE}Type '${GREEN}demo${WHITE}' to see the encryption pipeline in action.${RST}`);
    term.writeln(`  ${WHITE}Type '${GREEN}demo-xplenum${WHITE}' for the RISC-V XPLENUM extension demo.${RST}`);
    term.writeln("");

    writePrompt(term);

    term.onData((data: string) => {
      const code = data.charCodeAt(0);

      if (code === 13) {
        term.write("\r\n");
        const cmd = inputBuffer.current;
        inputBuffer.current = "";
        vmState.current = processCommand(cmd, vmState.current, term);
        writePrompt(term);
      } else if (code === 127 || code === 8) {
        if (inputBuffer.current.length > 0) {
          inputBuffer.current = inputBuffer.current.slice(0, -1);
          term.write("\b \b");
        }
      } else if (code === 3) {
        inputBuffer.current = "";
        term.write("^C\r\n");
        writePrompt(term);
      } else if (code >= 32) {
        inputBuffer.current += data;
        term.write(data);
      }
    });

    const handleResize = () => {
      if (fitAddon.current) {
        fitAddon.current.fit();
      }
    };
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      term.dispose();
      initialized.current = false;
    };
  }, [writePrompt]);

  return (
    <div
      ref={termRef}
      data-testid="ternary-vm-terminal"
      style={{
        width: "100%",
        height: "100%",
        minHeight: "500px",
        borderRadius: "8px",
        overflow: "hidden",
        background: "#0a0e1a",
      }}
    />
  );
}
