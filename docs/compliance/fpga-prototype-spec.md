# FPGA Prototype Specification

## Document Information

| Field | Value |
|-------|-------|
| Document | FPGA Prototype Design Specification |
| Version | 1.0 |
| Date | February 2026 |
| Owner | Capomastro Holdings Ltd. |

---

## 1. Overview

The Ternary Crypto Accelerator (TCA) is an FPGA-based hardware accelerator implementing PlenumNET ternary cryptographic primitives. The design is generated from Rust specifications via the `fpga_hdl` module and targets Xilinx Kintex UltraScale+ as the primary validation platform.

## 2. Architecture

### Block Diagram

```
                    AXI-Lite Control
                         |
              +----------+----------+
              |  Top-Level Module   |
              |  (ternary_crypto_   |
              |   accel)            |
              +----+----+----+-----+
                   |    |    |    |
              +----+ +--+--+ +---+----+ +-------+
              |GF(3)| |Sponge| |AES    | |Poly   |
              |ALU  | |Perm  | |S-Box  | |MAC    |
              +-----+ +------+ +-------+ +-------+
                                    |
                            AXI-Stream Data
```

### Modules

| Module | Function | Est. LUTs | Est. FFs | Target MHz |
|--------|----------|-----------|----------|------------|
| gf3_alu | GF(3) add/mul/neg, 243 trits | 2,916 | 1,458 | 500 |
| sponge_permutation | 729-trit state, 27 rounds | 43,740 | 2,916 | 400 |
| aes_sbox | Fermat-method GF(2^8) inverse | 512 | 16 | 500 |
| poly_mac | Polynomial multiply-accumulate | 3,072 | 1,536 | 450 |
| ternary_crypto_accel | Top-level + AXI interfaces | ~50,240 | ~5,926 | 400 |

### Total Resource Estimate

- LUTs: ~50,240 (7.6% of Kintex UltraScale+ 663,360)
- Flip-Flops: ~5,926
- BRAMs: TBD (sponge state may benefit from BRAM)
- DSPs: 0 (pure logic implementation)

## 3. Target Platform

### Primary: Xilinx Kintex UltraScale+

| Parameter | Value |
|-----------|-------|
| Device | XCKU5P-2FFVB676E |
| Board | KCU116 Evaluation Kit |
| LUTs | 663,360 |
| BRAMs | 1,080 |
| DSPs | 3,528 |
| Target Freq | 500 MHz |
| Utilization | <30% (recommended) |

### Secondary Targets

| Platform | LUTs | Feasibility |
|----------|------|-------------|
| Artix-7 (XC7A200T) | 134,600 | Marginal (37% util) |
| Stratix 10 | 933,120 | Comfortable (5% util) |
| CrossLink-NX | 53,000 | Insufficient |

## 4. Interfaces

### AXI-Lite Control (Configuration)

| Address | Register | Access |
|---------|----------|--------|
| 0x00 | CONTROL (start) | W |
| 0x04 | STATUS | R |
| 0x08 | MODULE_SELECT | R/W |
| 0x0C | VERSION | R |

### AXI-Stream Data (I/O)

- 32-bit data width
- Valid/ready handshake
- TLAST frame boundary

## 5. Trit Encoding

Balanced ternary values {-1, 0, +1} are encoded as 2-bit pairs:

| Value | Encoding |
|-------|----------|
| -1 | 2'b11 |
| 0 | 2'b00 |
| +1 | 2'b01 |

A 243-trit vector requires 486 bits (61 bytes).

## 6. Verification

The `hw_test` module defines 14+ hardware test cases across 5 categories:

| Category | Tests | Priority |
|----------|-------|----------|
| Functional | 8 | Critical/High |
| Timing | 3 | Critical/High |
| Power | 2 | Medium |
| Environmental | 1 | Medium |
| Endurance | 1 | Low |

## 7. HDL Generation

HDL source is generated via:

```rust
use plenumnet_kernel::crypto::fpga_hdl;

let package = fpga_hdl::generate_full_hdl_package();
// package.modules - 4 core modules
// package.top_level - top-level integration
// package.testbench - GF(3) ALU testbench
```
