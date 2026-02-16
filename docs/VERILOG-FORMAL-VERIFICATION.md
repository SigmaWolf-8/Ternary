<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  All Rights Reserved — Patent(s) Pending
-->

# Verilog Formal Verification Guide

**XPLENUM — RISC-V Ternary Security Extension**

| Field       | Value |
|-------------|-------|
| Target      | `rtl/xplenum_top.v` and submodules |
| Toolchain   | SymbiYosys + Yosys + Boolector |
| Date        | February 2026 |
| Status      | Infrastructure ready, awaiting synthesis environment |

---

## 1. Overview

Formal verification is critical for the XPLENUM security co-processor because:

- Custom instructions/opcodes can introduce subtle bugs (wrong CSR access, state leakage, incorrect ternary-to-binary mapping).
- Security extensions are high-value targets for fault-injection, side-channel, or Trojan insertion.
- Ternary logic emulation on binary hardware creates non-obvious corner cases (overflow in trit packing, glitch propagation).

## 2. Toolchain: SymbiYosys (Open-Source)

SymbiYosys wraps Yosys + solvers (ABC, Boolector, Yices, Z3) and supports:
- Bounded Model Checking (BMC) — fast failure detection
- K-induction — full proof
- Cover checking — reachability analysis

### 2.1 Installation

```bash
curl -L https://github.com/YosysHQ/oss-cad-suite-build/releases/latest/download/oss-cad-suite-linux-x64-*.sh -o install.sh
chmod +x install.sh
./install.sh --prefix ~/oss-cad-suite
source ~/oss-cad-suite/environment
```

Verify:
```bash
yosys --version
sby --version
```

### 2.2 Running Verification

```bash
cd rtl/formal/
sby -f xplenum_formal.sby bmc      # Bounded model check (fast)
sby -f xplenum_formal.sby prove    # K-induction proof (thorough)
sby -f xplenum_formal.sby cover    # Reachability analysis
```

Results:
- **PASS** — no counterexample found within depth
- **FAIL** — counterexample trace available in `xplenum_formal_*/engine_0/trace*.vcd`
- View traces: `gtkwave xplenum_formal_bmc/engine_0/trace.vcd`

## 3. Property Categories

### 3.1 Reset Properties (P1.x)
Verify all CSRs, outputs, and exception signals are zeroed after reset.

### 3.2 Instruction Decode Safety (P2.x)
- Non-XPLENUM opcodes never produce write-enable (protects base ISA).
- Version CSR always hardwired to `0x01_00_00`.
- Read-only CSRs cannot be written.

### 3.3 Trit Encoding Invariants (P3.x) — Critical
- No trit register output ever contains invalid encoding (`2'b11`).
- T-box output valid for valid inputs.
- LFSR-to-trit conversion never produces invalid encoding.

### 3.4 Masking Algebraic Properties (P4.x)
- Mask then unmask is identity for valid trit inputs.

### 3.5 Domain Isolation (P5.x)
- Mismatched domain ID raises exception.
- Domain operations disabled when `dom_en=0`.

### 3.6 Capability Bounds (P6.x)
- Out-of-range capability index raises `CAP_INVALID`.
- Capability operations disabled when `cap_en=0`.

### 3.7 Exception Safety (P7.x)
- Exception output reflects mux state.
- No exception without valid result.
- Exception CSRs update correctly.

### 3.8 Performance Counter (P8.x)
- Increments on every valid XPLENUM instruction.

### 3.9 Result Mux Safety (P9.x)
- At most one subunit valid per cycle (mutual exclusion).

### 3.10 LFSR Non-Degeneracy (P10.x)
- LFSR never all-zeros (stuck state prevention).
- Zero seed loads default `0xDEAD_BEEF`.

### 3.11 Information Flow (P11.x)
- Disabled subsystems return zero, not internal state.

### 3.12 Cover Properties (C12.x)
- Reachability of mask/unmask, T-box, capability exceptions, domain checks, performance counter reaching 10, trit overflow exceptions.

## 4. Progressive Verification Strategy

| Phase | Target | Timeline |
|-------|--------|----------|
| Module-level | Standalone trit_unit, mask_unit, domain_unit, cap_unit | Weeks 1-2 |
| Interface-level | CSR access safety, ternary-to-binary boundary | Weeks 3-4 |
| System-level | Full xplenum_top with all subunits | Month 2 |
| Advanced | K-induction proofs, multiclock, deeper unrolling | Ongoing |

## 5. RISC-V Formal Integration

For full ISA compliance verification:

1. Clone `https://github.com/YosysHQ/riscv-formal`
2. Create `cores/xplenum/` directory
3. Implement RVFI wrapper connecting instruction fetch, register file to RVFI signals
4. Define expected behavior for custom opcodes (PHASE_ENC 0xA6, etc.)
5. Run `make` for baseline RISC-V compliance + extension properties

## 6. Security-Specific Assertions

### Information Flow
```verilog
assume(secret_input);
assert(!leak_to_public_output);
```

### Trit State Validity
```verilog
assert(trit_value != 2'b11);  // Never invalid encoding
```

### Temporal Monotonicity
```verilog
assert(post_jitter_timestamp >= pre_jitter_timestamp);
```

## 7. Common Pitfalls

- Start small: verify one submodule fully before scaling.
- Use `(* keep *)` or blackbox large memories.
- For ternary logic: explicitly assert every trit stays in valid encoding.
- If SystemVerilog limitations arise, write properties in separate `.sv` file.

## 8. File Structure

```
rtl/formal/
  xplenum_formal.sby         # SymbiYosys configuration (3 tasks: bmc/prove/cover)
  xplenum_formal_props.v     # 12 sections, 30+ assertions, 6 cover properties
```

## 9. References

- [SymbiYosys Quickstart](https://symbiyosys.readthedocs.io/en/latest/quickstart.html)
- [ZipCPU Formal Tutorial](https://zipcpu.com/tutorial)
- [RISC-V Formal](https://yosyshq.readthedocs.io/projects/riscv-formal/)
- [OSS CAD Suite](https://github.com/YosysHQ/oss-cad-suite-build)
