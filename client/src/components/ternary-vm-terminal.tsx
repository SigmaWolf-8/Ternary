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

interface VMState {
  registers: number[];
  pc: number;
  mode: number;
  capsActive: string[];
  sideChMasked: boolean;
  auditCount: number;
}

function initVMState(): VMState {
  return {
    registers: new Array(27).fill(0),
    pc: 0,
    mode: 0,
    capsActive: [],
    sideChMasked: false,
    auditCount: 0,
  };
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
  "THASH": { hex: "0x60", cat: "Crypto", desc: "Ternary sponge hash" },
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
};

function processCommand(input: string, vm: VMState, term: Terminal): VMState {
  const trimmed = input.trim();
  if (!trimmed) return vm;

  const parts = trimmed.split(/\s+/);
  const cmd = parts[0].toLowerCase();

  switch (cmd) {
    case "help": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}Salvi Framework \u2014 Ternary Virtual Machine v2.0${RST}`);
      term.writeln(`${DIM}160-Opcode ISA \u2022 27-Trit Word \u2022 3-Ring Privilege \u2022 Post-Quantum${RST}`);
      term.writeln("");
      term.writeln(`${BOLD}${WHITE}Available Commands:${RST}`);
      term.writeln(`  ${GREEN}help${RST}               Show this help message`);
      term.writeln(`  ${GREEN}status${RST}             Display VM state and registers`);
      term.writeln(`  ${GREEN}opcodes${RST}            List all 160 ISA opcodes by category`);
      term.writeln(`  ${GREEN}opcode <NAME>${RST}      Show details for a specific opcode`);
      term.writeln(`  ${GREEN}exec <MNEMONIC>${RST}    Simulate execution of an instruction`);
      term.writeln(`  ${GREEN}demo${RST}               Run the dual-phase encryption demo`);
      term.writeln(`  ${GREEN}demo-cap${RST}           Run the capability security demo`);
      term.writeln(`  ${GREEN}demo-sidech${RST}        Run the side-channel masking demo`);
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
      term.writeln(`${BOLD}${BRAND}Salvi ISA v2.0 \u2014 160 Opcodes${RST}`);
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
      term.writeln(`${DIM}Total ISA: 160 opcodes across Core, Extended, Crypto Acceleration,${RST}`);
      term.writeln(`${DIM}SIMD, System, Security/Audit, and Debug/Profiling categories.${RST}`);
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
      term.writeln(`  ${WHITE}ISA Version:${RST}       ${CYAN}v2.0 (160 opcodes)${RST}`);
      term.writeln(`  ${WHITE}Word Size:${RST}         ${CYAN}27 trits (1 tryte = 3\u00B3)${RST}`);
      term.writeln(`  ${WHITE}Register File:${RST}     ${CYAN}27 general-purpose ternary${RST}`);
      term.writeln(`  ${WHITE}Privilege Rings:${RST}   ${RED}Ring0${RST} ${YELLOW}Ring1${RST} ${GREEN}Ring2${RST}`);
      term.writeln(`  ${WHITE}Representations:${RST}   ${CYAN}A{-1,0,+1} B{0,1,2} C{1,2,3}${RST}`);
      term.writeln(`  ${WHITE}Kernel:${RST}            ${CYAN}Rust, 33MB ELF, 47,000+ LOC${RST}`);
      term.writeln(`  ${WHITE}Subsystems:${RST}        ${CYAN}14${RST}`);
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
      return vm;
    }

    case "cpuid": {
      term.writeln("");
      term.writeln(`${BOLD}${BRAND}CPUID \u2014 Processor Capabilities${RST}`);
      term.writeln(`${DIM}${"─".repeat(56)}${RST}`);
      term.writeln(`  ${WHITE}Vendor:${RST}          ${CYAN}Capomastro Holdings Ltd.${RST}`);
      term.writeln(`  ${WHITE}Model:${RST}           ${CYAN}Salvi T27-160 v2.0${RST}`);
      term.writeln(`  ${WHITE}ISA:${RST}             ${CYAN}Ternary 160-opcode${RST}`);
      term.writeln(`  ${WHITE}Word:${RST}            ${CYAN}27-trit (42.77 bits equivalent)${RST}`);
      term.writeln(`  ${WHITE}Clock Source:${RST}    ${CYAN}HPTP femtosecond (10\u207B\u00B9\u2075 s)${RST}`);
      term.writeln(`  ${WHITE}Crypto:${RST}          ${GREEN}TL-KEM TL-DSA AES-256 SHA-2/3${RST}`);
      term.writeln(`  ${WHITE}SIMD:${RST}            ${GREEN}8-lane ternary vector unit${RST}`);
      term.writeln(`  ${WHITE}Side-Ch:${RST}         ${GREEN}Dual-layer (arch + algebraic)${RST}`);
      term.writeln(`  ${WHITE}Capabilities:${RST}    ${GREEN}Sentinel-trit based${RST}`);
      term.writeln(`  ${WHITE}Compliance:${RST}      ${GREEN}CNSA 2.0, FIPS 140-3${RST}`);
      term.writeln(`  ${WHITE}GC:${RST}              ${GREEN}Ternary-aware mark/sweep${RST}`);
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
    term.writeln(`${BOLD}${BRAND}  \u2588${RST}  ${DIM}ISA v2.0 \u2022 160 Opcodes \u2022 27-Trit Word${RST}     ${BOLD}${BRAND}\u2588${RST}`);
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
