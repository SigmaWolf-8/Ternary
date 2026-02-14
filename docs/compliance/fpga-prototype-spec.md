# FPGA Prototype Specification

## Document Information

| Field | Value |
|-------|-------|
| Document | FPGA Prototype Design Specification |
| Version | 2.0 |
| Date | February 2026 |
| Owner | Capomastro Holdings Ltd. |

---

## 1. Overview

The Ternary Crypto Accelerator (TCA) is an FPGA-based hardware accelerator implementing PlenumNET ternary cryptographic primitives and the TVM ISA v2.0 instruction decoder. The design is generated from Rust specifications via the `fpga_hdl` module and targets Xilinx Kintex UltraScale+ as the primary validation platform.

Version 2.0 adds a hierarchical nibble-aligned ISA decoder supporting all 160 TVM opcodes with privilege-level enforcement, functional unit dispatch, and illegal opcode detection.

## 2. Architecture

### Block Diagram

```
                    AXI-Lite Control
                         |
              +----------+----------+
              |  Top-Level Module   |
              |  (ternary_crypto_   |
              |   accel v2.0)       |
              +--+--+--+--+--+-----+
                 |  |  |  |  |
    +------------+  |  |  |  +------------------+
    |               |  |  |                     |
+---+---+    +-----++  |  +--------+    +-------+-------+
|ISA v2 |    |GF(3) |  |  |AES     |    |Poly           |
|Decoder|    |ALU   |  |  |S-Box   |    |MAC            |
+---+---+    +------+  |  +--------+    +---------------+
    |                   |
    |  cat_enable   +---+------+
    |  op_index     |Sponge    |
    +-------------->|Perm      |
    |  dispatch     +----------+
    |
    +--- alu_enable, crypto_unit_enable,
         simd_unit_enable, etc.

                            AXI-Stream Data
```

### Modules

| Module | Function | Est. LUTs | Est. FFs | Target MHz |
|--------|----------|-----------|----------|------------|
| tvm_isa_v2_decoder | ISA v2.0 nibble-aligned decoder, 160 opcodes | 420 | 280 | 600 |
| gf3_alu | GF(3) add/mul/neg, 243 trits | 2,916 | 1,458 | 500 |
| sponge_permutation | 729-trit state, 27 rounds | 43,740 | 2,916 | 400 |
| aes_sbox | Fermat-method GF(2^8) inverse | 512 | 16 | 500 |
| poly_mac | Polynomial multiply-accumulate | 3,072 | 1,536 | 450 |
| ternary_crypto_accel | Top-level + AXI + decoder integration | ~50,660 | ~6,206 | 400 |

### Total Resource Estimate

- LUTs: ~50,660 (7.6% of Kintex UltraScale+ 663,360)
- Flip-Flops: ~6,206
- BRAMs: TBD (sponge state may benefit from BRAM)
- DSPs: 0 (pure logic implementation)

## 3. ISA v2.0 Decoder Architecture

### Nibble-Aligned Opcode Layout

The ISA v2.0 uses a nibble-aligned encoding where the upper nibble (`opcode[7:4]`) selects the instruction category and the lower nibble (`opcode[3:0]`) selects the operation within that category. This enables a two-stage hierarchical decode.

| Range | Category | Opcodes | Privilege |
|-------|----------|---------|-----------|
| 0x00-0x0F | Basic & Extended Arithmetic | 16 | Any |
| 0x10-0x1F | Ternary Core | 16 | Any |
| 0x20-0x2F | Memory, Register & Atomics | 16 | Any |
| 0x30-0x3F | Control Flow | 16 | Any |
| 0x40-0x4F | Comparison & Selection | 16 | Any |
| 0x50-0x5F | Binary Logic & Bit Manipulation | 16 | Any |
| 0x60-0x6F | Crypto Acceleration | 16 | Any |
| 0x70-0x7F | SIMD / Vector Ternary | 16 | Any |
| 0x80-0x8F | System & Privilege | 16 | Ring0/Ring1 |
| 0x90-0x97 | Security & Audit | 8 | Ring0 only |
| 0x98-0x9F | Debug & Profiling | 8 | Ring0/Ring1 |
| 0xA0-0xFF | Reserved (illegal) | 0 | N/A |

**Total: 160 opcodes in 5 trits (ceil(log3(160)) = 5)**

### Two-Stage Decode

**Stage 1 (Category Decode):** 4-bit upper nibble drives a 4-to-11 one-hot decoder, producing `cat_enable[10:0]`. Each bit enables one functional-unit category.

**Stage 2 (Operation Decode):** The 4-bit lower nibble is passed through as `op_index[3:0]`, valid only when the corresponding category is enabled. For categories 0x9 (Security/Debug), the lower nibble further discriminates: ops 0x0-0x7 are Security, ops 0x8-0xF are Debug.

### Decoder Outputs

| Signal | Width | Description |
|--------|-------|-------------|
| cat_enable | 11 | One-hot category select |
| op_index | 4 | Operation within category |
| decode_valid | 1 | Valid decoded instruction |
| illegal_op | 1 | Opcode >= 0xA0 detected |
| privilege_fault | 1 | Insufficient ring level |
| alu_enable | 1 | ALU functional unit dispatch |
| ternary_unit_enable | 1 | Ternary unit dispatch |
| mem_unit_enable | 1 | Memory unit dispatch |
| branch_unit_enable | 1 | Branch unit dispatch |
| crypto_unit_enable | 1 | Crypto accelerator dispatch |
| simd_unit_enable | 1 | SIMD unit dispatch |
| system_enable | 1 | System/privileged dispatch |
| is_halt | 1 | HALT instruction detected |

### Area Savings: v1 vs v2 Decode

| Metric | v1 Flat Decode | v2 Hierarchical | Delta |
|--------|---------------|-----------------|-------|
| Comparator count | 160 (8-bit each) | 10 + 16 = 26 | -84% |
| Estimated LUTs | ~650 | ~420 | -35% |
| Critical path | ~2.0 ns (500 MHz) | ~1.7 ns (600 MHz) | -15% |
| Fan-in depth | log2(160) = 7.3 | max(log2(10), log2(16)) = 4 | -45% |

The nibble-aligned encoding eliminates cross-boundary bit extraction, reducing routing complexity and improving timing closure. The hierarchical decode also enables selective clock-gating of unused functional units based on `cat_enable`.

## 4. Target Platform

### Primary: Xilinx Kintex UltraScale+

| Parameter | Value |
|-----------|-------|
| Device | XCKU5P-2FFVB676E |
| Board | KCU116 Evaluation Kit |
| LUTs | 663,360 |
| BRAMs | 1,080 |
| DSPs | 3,528 |
| Target Freq | 500 MHz (decoder: 600 MHz) |
| Utilization | <30% (recommended) |

### Secondary Targets

| Platform | LUTs | Feasibility |
|----------|------|-------------|
| Artix-7 (XC7A200T) | 134,600 | Marginal (37% util) |
| Stratix 10 | 933,120 | Comfortable (5% util) |
| CrossLink-NX | 53,000 | Insufficient |

## 5. Interfaces

### AXI-Lite Control (Configuration)

| Address | Register | Access |
|---------|----------|--------|
| 0x00 | CONTROL (start) | W |
| 0x04 | STATUS | R |
| 0x08 | MODULE_SELECT | R/W |
| 0x0C | VERSION | R |
| 0x10 | DECODE_STATUS (v2) | R |

### Instruction Decode Interface (v2.0)

| Signal | Width | Direction | Description |
|--------|-------|-----------|-------------|
| instr_opcode | 8 | Input | ISA v2.0 opcode |
| instr_valid | 1 | Input | Valid instruction strobe |
| instr_privilege | 2 | Input | Current privilege ring |
| decode_fault | 1 | Output | Illegal op or privilege fault |

### AXI-Stream Data (I/O)

- 32-bit data width
- Valid/ready handshake
- TLAST frame boundary

## 6. Trit Encoding

Balanced ternary values {-1, 0, +1} are encoded as 2-bit pairs:

| Value | Encoding |
|-------|----------|
| -1 | 2'b11 |
| 0 | 2'b00 |
| +1 | 2'b01 |

A 243-trit vector requires 486 bits (61 bytes).

## 7. Verification

The `hw_test` module defines 14+ hardware test cases across 5 categories:

| Category | Tests | Priority |
|----------|-------|----------|
| Functional | 8 | Critical/High |
| Timing | 3 | Critical/High |
| Power | 2 | Medium |
| Environmental | 1 | Medium |
| Endurance | 1 | Low |

## 8. HDL Generation

HDL source is generated via:

```rust
use plenumnet_kernel::crypto::fpga_hdl;

let package = fpga_hdl::generate_full_hdl_package();
// package.modules - 5 core modules (GF3 ALU, Sponge, AES, PolyMAC, ISA v2 Decoder)
// package.top_level - top-level integration with decoder dispatch
// package.testbench - GF(3) ALU testbench
```

Area savings analysis available via:

```rust
use plenumnet_kernel::crypto::fpga_synth;

let comparison = fpga_synth::decode_area_savings_analysis();
// comparison.savings_pct - ~35% LUT reduction vs v1
// comparison.analysis - detailed breakdown
```
