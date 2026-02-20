# XPlenum Phase 1.1 — RISC-V Core Selection Decision Document

**Capomastro Holdings Ltd. — Applied Physics Division**
**Date: February 18, 2026**
**Classification: CONFIDENTIAL**

---

## 1. Evaluation Criteria

Five weighted criteria were used to evaluate candidate RISC-V cores for XPlenum integration:

| # | Criterion | Weight | Description |
|---|-----------|--------|-------------|
| C1 | Pipeline Compatibility | 30% | Alignment with XPlenum's 32-bit instruction/data interface, single-cycle decode, and registered output stage |
| C2 | Custom Instruction Extensibility | 25% | Quality of existing extension hooks, documentation of decode/execute stages, prior custom-instruction projects |
| C3 | License Terms | 15% | Apache 2.0, MIT, or Solderpad (Apache-wrapper) required for commercial use |
| C4 | Community & Ecosystem | 15% | Active maintenance, contributor base, FPGA/ASIC synthesis support, emulator availability |
| C5 | Verification Infrastructure | 15% | Existing testbench quality, formal verification support, compliance test suite availability |

---

## 2. Candidate Evaluation

### 2.1 CVA6 (OpenHW Group)

| Criterion | Score (1–5) | Notes |
|-----------|-------------|-------|
| C1: Pipeline | 5 | 6-stage in-order pipeline (PC Gen → IF → ID → Issue → EX → WB). Single-issue. Scoreboard in Issue stage handles hazards. Execute stage accepts parallel functional units. XPlenum's registered output aligns with WB stage. |
| C2: Extensibility | 5 | Proven custom-instruction track record: PERCIVAL (posit arithmetic), B-extension (Zba/Zbb/Zbc/Zbs), CVA6-CFI (shadow stack). CORE-V-XIF standardised coprocessor interface available. Decoder in `core/decoder.sv` has clean opcode dispatch. Custom units wire in parallel to ALU/FPU in `core/ex_stage.sv`. |
| C3: License | 5 | Solderpad Hardware License v2.1 (Apache 2.0 wrapper for hardware). Fully compatible with commercial use. |
| C4: Community | 5 | Maintained by OpenHW Group (industry consortium). Active GitHub (1,600+ stars, regular commits). FPGA targets (Xilinx, Intel) supported out-of-box. Linux-capable. ASIC tape-outs demonstrated at 22nm (GlobalFoundries). |
| C5: Verification | 4 | Comprehensive UVM testbench, riscv-arch-test compliance. Verilator simulation support. Formal verification possible but not pre-integrated (SymbiYosys compatible). |
| **Weighted Score** | **4.85** | |

**Strengths**: Best-documented custom instruction pathway. PERCIVAL project provides a direct blueprint for integrating a parallel execution unit. CORE-V-XIF offers a future-proof standardised interface. 6-stage pipeline gives natural insertion point at Execute stage. RV64 support matches XPlenum's 64-bit target architecture.

**Risks**: 6-stage pipeline (vs XPlenum's assumed 5-stage) requires careful alignment of output timing. Issue-stage scoreboard integration adds complexity but provides robust hazard detection.

### 2.2 Rocket (Chips Alliance / UC Berkeley)

| Criterion | Score (1–5) | Notes |
|-----------|-------------|-------|
| C1: Pipeline | 4 | 5-stage in-order pipeline (Fetch → Decode → Execute → Memory → WB). Classic RISC pipeline. However, RoCC interface issues custom instructions at Writeback stage, not Execute. Direct pipeline integration (non-RoCC) requires modifying Chisel source. |
| C2: Extensibility | 4 | RoCC (Rocket Custom Coprocessor) interface is well-documented but dispatches at WB stage — too late for XPlenum's pipeline-integrated model. SCIE (Simple Custom Instruction Extension) operates at Execute stage but limited to unpipelined single-cycle operations. Deep pipeline modification requires Chisel expertise. |
| C3: License | 5 | Apache 2.0 and BSD licenses. Fully compatible. |
| C4: Community | 5 | Flagship RISC-V implementation. Part of Chipyard ecosystem. Extensive documentation. Largest community. Multiple ASIC tape-outs (SiFive cores derive from Rocket). |
| C5: Verification | 4 | Comprehensive riscv-tests suite. Chisel-based verification. Verilator support. No pre-built formal verification harness for custom extensions. |
| **Weighted Score** | **4.30** | |

**Strengths**: Most mature RISC-V core. Chipyard ecosystem provides full SoC integration framework. RoCC is well-proven for coprocessor accelerators (SHA-3, Gemmini, Hwacha).

**Risks**: Written in Chisel (Scala-based HDL), not Verilog — XPlenum is native Verilog, requiring a mixed-language wrapper or Chisel rewrite. RoCC's late-binding dispatch (WB stage) is architecturally misaligned with XPlenum's need for Execute-stage integration with CSR access and exception generation. SCIE is too simple for XPlenum's multi-cycle operations.

### 2.3 BOOM (Berkeley Out-of-Order Machine)

| Criterion | Score (1–5) | Notes |
|-----------|-------------|-------|
| C1: Pipeline | 2 | Out-of-order superscalar pipeline. Stages: Fetch → Decode → Rename → Dispatch → Issue → Execute → WB. Out-of-order execution fundamentally complicates XPlenum's domain isolation and capability security model (ordering guarantees required). |
| C2: Extensibility | 3 | Shares RoCC interface with Rocket. Same WB-stage dispatch limitation. OoO rename/dispatch stages add significant integration complexity for security-critical state (domain tags, capability table). |
| C3: License | 5 | BSD license. Compatible. |
| C4: Community | 3 | Smaller community than Rocket/CVA6. Research-focused. Fewer FPGA deployment examples. Chipyard-integrated. |
| C5: Verification | 3 | riscv-tests support. OoO verification is inherently harder — state space explosion for formal methods. |
| **Weighted Score** | **2.95** | |

**Strengths**: Highest raw performance (superscalar OoO). Demonstrates RISC-V can compete with commercial cores.

**Risks**: Out-of-order execution is fundamentally incompatible with XPlenum's security model without significant additional work. Domain isolation requires in-order enforcement of security boundaries. Capability revocation semantics assume instruction ordering. Integration effort estimated at 3–4x that of in-order cores. **Not recommended** for first integration.

### 2.4 PicoRV32 (YosysHQ)

| Criterion | Score (1–5) | Notes |
|-----------|-------------|-------|
| C1: Pipeline | 2 | Multi-cycle (non-pipelined) or optionally 2-stage pipeline. RV32I only — no RV64 support. 32-bit data path vs XPlenum's 64-bit target. Would require significant XPlenum interface narrowing. |
| C2: Extensibility | 3 | PCPI (Pico Co-Processor Interface) is simple but limited: 2 inputs, 1 output, 16-cycle timeout, no memory access from coprocessor, no CSR access. Would require XPlenum CSR file to be external to PCPI. |
| C3: License | 5 | ISC license (simplified BSD). Fully compatible. |
| C4: Community | 3 | Widely used for FPGA prototyping. Minimal area. Active but less community than CVA6/Rocket. No Linux support (too small). |
| C5: Verification | 3 | Basic testbench. No formal verification harness. SymbiYosys-compatible (same maintainer). Limited compliance testing. |
| **Weighted Score** | **2.90** | |

**Strengths**: Smallest area footprint. Fastest simulation speed. Easiest to understand (single Verilog file). Ideal for early FPGA prototyping of individual XPlenum instructions.

**Risks**: RV32I-only eliminates 64-bit address space needed for production. PCPI's 16-cycle timeout and no-CSR-access limitations make it unsuitable for XPlenum's domain/capability subsystems. Would require extensive interface adaptation. Suitable only as a secondary validation target, not primary integration.

---

## 3. Comparison Matrix

| Criterion (Weight) | CVA6 | Rocket | BOOM | PicoRV32 |
|---------------------|------|--------|------|----------|
| C1: Pipeline (30%) | 5 | 4 | 2 | 2 |
| C2: Extensibility (25%) | 5 | 4 | 3 | 3 |
| C3: License (15%) | 5 | 5 | 5 | 5 |
| C4: Community (15%) | 5 | 5 | 3 | 3 |
| C5: Verification (15%) | 4 | 4 | 3 | 3 |
| **Weighted Total** | **4.85** | **4.30** | **2.95** | **2.90** |
| **Rank** | **1st** | **2nd** | **4th** | **3rd** |

---

## 4. Decision

**Selected Core: CVA6 (OpenHW Group)**

### Rationale

1. **Pipeline Alignment**: CVA6's 6-stage pipeline with dedicated Issue and Execute stages provides a natural insertion point for XPlenum as a parallel functional unit alongside the ALU and FPU. The Issue-stage scoreboard handles data hazards automatically.

2. **Proven Extension Pathway**: The PERCIVAL project (posit arithmetic integration) demonstrates the exact integration pattern XPlenum requires — a custom execution unit wired in parallel at the Execute stage, with handshake signals for multi-cycle operations. This is not theoretical; it has been fabricated in silicon.

3. **Native Verilog/SystemVerilog**: CVA6 is written in SystemVerilog, directly compatible with XPlenum's Verilog RTL. No language bridge or HDL conversion required.

4. **RV64 Support**: CVA6 supports both RV32 and RV64 configurations, matching XPlenum's 64-bit XLEN target for production deployment.

5. **CORE-V-XIF Future Path**: The emerging CORE-V eXtension Interface provides a standardised coprocessor protocol. While not required for initial integration, migrating to XIF later would make XPlenum portable across all CORE-V cores.

6. **Industry Backing**: OpenHW Group membership includes NXP, Thales, ETH Zurich, and others. CVA6 has been taped out at 22nm (GlobalFoundries) and deployed on multiple FPGA platforms. This reduces risk for PlenumNET's FPGA board validation phase.

### Secondary Target (Future)

PicoRV32 is recommended as a secondary validation target after primary CVA6 integration is complete. Its simplicity makes it useful for rapid instruction-level debugging, even though it cannot support the full XPlenum feature set due to RV32I and PCPI limitations.

---

## 5. CVA6 Configuration for XPlenum

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| XLEN | 64 | Match XPlenum 64-bit data path |
| ISA | RV64IMAC | Integer + Multiply + Atomic + Compressed |
| FPU | Disabled | Not required for XPlenum security operations; reduces area |
| MMU | Sv39 | 39-bit virtual addressing for domain isolation |
| Cache | 16KB I$ / 16KB D$ | Standard configuration |
| Custom Opcode | 0x0B (Custom-0) | Primary XPlenum encoding space |
| Custom Opcode | 0x2B (Custom-1) | Reserved for future expansion |
| Pipeline | 6-stage in-order | Default CVA6 configuration |

---

## Appendix: CVA6 Key Source Files for Integration

| File | Purpose | XPlenum Relevance |
|------|---------|-------------------|
| `core/decoder.sv` | Instruction decode | Add Custom-0/Custom-1 opcode recognition |
| `core/ex_stage.sv` | Execute stage top | Wire XPlenum as parallel functional unit |
| `core/issue_stage.sv` | Scoreboard & issue | Register XPlenum destination for hazard tracking |
| `core/commit_stage.sv` | Writeback/commit | Route XPlenum results to register file |
| `core/csr_regfile.sv` | CSR read/write | Add custom CSR addresses 0x7C0–0x7CB |
| `core/controller.sv` | Pipeline control | Handle XPlenum exceptions and stalls |
| `core/ariane_pkg.sv` | Package definitions | Add XPlenum-specific type definitions |
