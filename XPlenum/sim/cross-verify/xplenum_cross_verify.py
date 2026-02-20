#!/usr/bin/env python3
# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Applied Physics Division
#
# PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
# Patent(s) Pending.
#
# XPLENUM — Cross-Verification Framework (RTL vs Emulator)
# Phase 6, Task 6.7: Register trace comparison between Verilator RTL and
# Spike/QEMU emulator outputs
#
# Usage:
#   python3 xplenum_cross_verify.py --rtl rtl_trace.log --emu emu_trace.log
#   python3 xplenum_cross_verify.py --generate-vectors --count 1000
# =============================================================================

import argparse
import json
import os
import sys
import struct
import hashlib
from dataclasses import dataclass, field, asdict
from typing import List, Optional, Tuple

# ---------------------------------------------------------------------------
# Test vector format
# ---------------------------------------------------------------------------

@dataclass
class TestVector:
    """Single instruction test vector for cross-verification."""
    instruction: int       # 32-bit encoded instruction
    rs1_val: int           # Source register 1 value
    rs2_val: int           # Source register 2 value
    funct3: int            # Functional group
    funct7: int            # Sub-function
    mnemonic: str          # Human-readable mnemonic
    expected_rd: Optional[int] = None  # Expected result (if deterministic)
    expected_exc: int = 0  # Expected exception code
    writes_rd: bool = True # Whether instruction writes destination register

@dataclass
class TraceEntry:
    """Single trace entry from RTL or emulator."""
    pc: int
    instruction: int
    rd_idx: int
    rd_val: int
    exc_code: int
    csr_xpstatus: int
    csr_xpperf_cnt: int

@dataclass
class CrossVerifyResult:
    """Result of cross-verification comparison."""
    total_vectors: int = 0
    matched: int = 0
    diverged: int = 0
    divergences: list = field(default_factory=list)

# ---------------------------------------------------------------------------
# Instruction encoding
# ---------------------------------------------------------------------------

XP_OPCODE = 0x0B

INSTRUCTIONS = [
    # (mnemonic, funct3, funct7, writes_rd, deterministic)
    ("TMASK",    0, 0x00, True,  True),
    ("TUNMASK",  0, 0x01, True,  True),
    ("TMASKR",   0, 0x02, True,  False),  # DRBG output — non-deterministic
    ("TMASKRF",  0, 0x03, True,  False),
    ("TDOMSET",  1, 0x00, False, True),
    ("TDOMCHK",  1, 0x01, True,  True),
    ("TDOMCLR",  1, 0x02, False, True),
    ("TDOMXFR",  1, 0x03, False, True),
    ("TCAPLD",   2, 0x00, True,  True),
    ("TCAPCHK",  2, 0x01, True,  True),
    ("TCAPST",   2, 0x02, False, True),
    ("TCAPREV",  2, 0x03, True,  True),
    ("TROTL",    3, 0x00, True,  True),
    ("TROTR",    3, 0x01, True,  True),
    ("TTBOX",    3, 0x02, True,  True),
    ("TPERM",    3, 0x03, True,  True),
    ("TTRIT",    4, 0x00, True,  True),
    ("TDETRIT",  4, 0x01, True,  True),
    ("TSIGFLT",  5, 0x00, True,  True),
    ("TSIGCMP",  5, 0x01, True,  True),
    ("TSIGACC",  5, 0x02, True,  True),
]

def encode_r_type(funct7: int, rs2: int, rs1: int, funct3: int,
                  rd: int, opcode: int) -> int:
    return ((funct7 & 0x7F) << 25) | ((rs2 & 0x1F) << 20) | \
           ((rs1 & 0x1F) << 15) | ((funct3 & 0x7) << 12) | \
           ((rd & 0x1F) << 7) | (opcode & 0x7F)


# ---------------------------------------------------------------------------
# Test vector generation
# ---------------------------------------------------------------------------

def generate_test_vectors(count: int = 1000) -> List[TestVector]:
    """Generate deterministic test vectors for cross-verification."""
    vectors = []
    import random
    rng = random.Random(0xCAFEBABE)

    for i in range(count):
        # Cycle through all 21 instructions
        idx = i % len(INSTRUCTIONS)
        mnem, f3, f7, writes_rd, deterministic = INSTRUCTIONS[idx]

        rs1_val = rng.randint(0, 0xFFFFFFFF)
        rs2_val = rng.randint(0, 0xFFFFFFFF)

        insn = encode_r_type(f7, 2, 1, f3, 3, XP_OPCODE)

        # Compute expected result for deterministic operations
        expected_rd = None
        expected_exc = 0

        if deterministic:
            if mnem == "TMASK" or mnem == "TUNMASK":
                expected_rd = rs1_val ^ rs2_val
            elif mnem == "TROTL":
                sh = rs2_val & 0x1F
                expected_rd = ((rs1_val << sh) | (rs1_val >> (32 - sh))) & 0xFFFFFFFF
            elif mnem == "TROTR":
                sh = rs2_val & 0x1F
                expected_rd = ((rs1_val >> sh) | (rs1_val << (32 - sh))) & 0xFFFFFFFF
            elif mnem == "TSIGCMP":
                s1 = rs1_val if rs1_val < 0x80000000 else rs1_val - 0x100000000
                s2 = rs2_val if rs2_val < 0x80000000 else rs2_val - 0x100000000
                if s1 > s2: expected_rd = 1
                elif s1 < s2: expected_rd = 0xFFFFFFFF
                else: expected_rd = 0

        vectors.append(TestVector(
            instruction=insn,
            rs1_val=rs1_val,
            rs2_val=rs2_val,
            funct3=f3,
            funct7=f7,
            mnemonic=mnem,
            expected_rd=expected_rd,
            expected_exc=expected_exc,
            writes_rd=writes_rd
        ))

    return vectors


# ---------------------------------------------------------------------------
# Trace parsing
# ---------------------------------------------------------------------------

def parse_trace(filename: str) -> List[TraceEntry]:
    """Parse a register trace file (CSV or JSON format)."""
    entries = []

    if filename.endswith('.json'):
        with open(filename) as f:
            data = json.load(f)
            for item in data:
                entries.append(TraceEntry(**item))
    else:
        # CSV format: pc,instruction,rd_idx,rd_val,exc_code,xpstatus,xpperf_cnt
        with open(filename) as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                parts = line.split(',')
                if len(parts) >= 7:
                    entries.append(TraceEntry(
                        pc=int(parts[0], 16),
                        instruction=int(parts[1], 16),
                        rd_idx=int(parts[2]),
                        rd_val=int(parts[3], 16),
                        exc_code=int(parts[4]),
                        csr_xpstatus=int(parts[5], 16),
                        csr_xpperf_cnt=int(parts[6])
                    ))

    return entries


# ---------------------------------------------------------------------------
# Cross-verification comparison
# ---------------------------------------------------------------------------

def compare_traces(rtl_trace: List[TraceEntry],
                   emu_trace: List[TraceEntry]) -> CrossVerifyResult:
    """Compare RTL and emulator traces instruction-by-instruction."""
    result = CrossVerifyResult()

    min_len = min(len(rtl_trace), len(emu_trace))
    result.total_vectors = min_len

    for i in range(min_len):
        rtl = rtl_trace[i]
        emu = emu_trace[i]

        match = True
        divergence_detail = {}

        # Compare instruction word
        if rtl.instruction != emu.instruction:
            match = False
            divergence_detail['instruction'] = {
                'rtl': hex(rtl.instruction),
                'emu': hex(emu.instruction)
            }

        # Compare rd value (only if same instruction)
        if match and rtl.rd_val != emu.rd_val:
            match = False
            divergence_detail['rd_val'] = {
                'rtl': hex(rtl.rd_val),
                'emu': hex(emu.rd_val)
            }

        # Compare exception code
        if rtl.exc_code != emu.exc_code:
            match = False
            divergence_detail['exc_code'] = {
                'rtl': rtl.exc_code,
                'emu': emu.exc_code
            }

        if match:
            result.matched += 1
        else:
            result.diverged += 1
            divergence_detail['index'] = i
            divergence_detail['pc'] = hex(rtl.pc)
            result.divergences.append(divergence_detail)

    if len(rtl_trace) != len(emu_trace):
        result.divergences.append({
            'length_mismatch': {
                'rtl': len(rtl_trace),
                'emu': len(emu_trace)
            }
        })

    return result


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def generate_report(result: CrossVerifyResult, output: str = None):
    """Generate cross-verification report."""
    lines = []
    lines.append("=" * 72)
    lines.append("XPlenum Cross-Verification Report")
    lines.append("RTL Simulation vs Emulator (Spike/QEMU)")
    lines.append("=" * 72)
    lines.append("")
    lines.append(f"Total vectors compared: {result.total_vectors}")
    lines.append(f"Matched:                {result.matched}")
    lines.append(f"Diverged:               {result.diverged}")
    lines.append(f"Match rate:             {result.matched/max(1,result.total_vectors)*100:.2f}%")
    lines.append("")

    if result.diverged > 0:
        lines.append("DIVERGENCES:")
        lines.append("-" * 72)
        for d in result.divergences[:20]:
            lines.append(json.dumps(d, indent=2))
        if len(result.divergences) > 20:
            lines.append(f"... and {len(result.divergences) - 20} more")
    else:
        lines.append("PASS — All traces match exactly.")

    lines.append("")
    lines.append("=" * 72)

    report = "\n".join(lines)
    print(report)

    if output:
        with open(output, 'w') as f:
            f.write(report)
        print(f"\nReport saved to: {output}")


# ---------------------------------------------------------------------------
# Verilator VCD trace extraction helper
# ---------------------------------------------------------------------------

def generate_verilator_trace_script():
    """Generate a Verilator trace extraction helper (Verilog task)."""
    script = """
// Add this task to your integration testbench to dump register traces:
//
// task dump_xplenum_trace;
//   integer trace_fd;
//   initial begin
//     trace_fd = $fopen("rtl_trace.csv", "w");
//     $fwrite(trace_fd, "# pc,instruction,rd_idx,rd_val,exc_code,xpstatus,xpperf_cnt\\n");
//   end
//
//   always @(posedge clk) begin
//     if (dut.u_xplenum_core.rd_write_en) begin
//       $fwrite(trace_fd, "%08x,%08x,%0d,%08x,%0d,%08x,%0d\\n",
//         pc_r, instruction_r,
//         dut.u_xplenum_core.rd_addr,
//         dut.u_xplenum_core.rd_data,
//         dut.u_xplenum_core.xp_exc_code,
//         dut.u_xplenum_core.csr_xpstatus,
//         dut.u_xplenum_core.csr_xpperf_cnt);
//     end
//   end
// endtask
"""
    return script


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description='XPlenum Cross-Verification: RTL vs Emulator'
    )
    parser.add_argument('--rtl', type=str, help='RTL simulation trace file')
    parser.add_argument('--emu', type=str, help='Emulator trace file')
    parser.add_argument('--generate-vectors', action='store_true',
                        help='Generate test vectors (JSON)')
    parser.add_argument('--count', type=int, default=1000,
                        help='Number of test vectors to generate')
    parser.add_argument('--output', type=str, help='Output report file')
    parser.add_argument('--verilator-script', action='store_true',
                        help='Print Verilator trace extraction script')

    args = parser.parse_args()

    if args.verilator_script:
        print(generate_verilator_trace_script())
        return

    if args.generate_vectors:
        vectors = generate_test_vectors(args.count)
        out_file = args.output or 'xplenum_test_vectors.json'
        with open(out_file, 'w') as f:
            json.dump([asdict(v) for v in vectors], f, indent=2)
        print(f"Generated {len(vectors)} test vectors → {out_file}")

        # Summary
        mnemonics = {}
        for v in vectors:
            mnemonics[v.mnemonic] = mnemonics.get(v.mnemonic, 0) + 1
        print("\nInstruction distribution:")
        for m, c in sorted(mnemonics.items()):
            print(f"  {m:12s}: {c:4d}")
        return

    if args.rtl and args.emu:
        rtl_trace = parse_trace(args.rtl)
        emu_trace = parse_trace(args.emu)
        result = compare_traces(rtl_trace, emu_trace)
        generate_report(result, args.output)
        sys.exit(1 if result.diverged > 0 else 0)

    parser.print_help()

if __name__ == '__main__':
    main()
