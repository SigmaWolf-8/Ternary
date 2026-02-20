# XPlenum ISA Specification v1.0

**Date:** 2026-02-18  
**Version:** 1.0 (CSR XPVERSION = 0x010000)  
**Classification:** PROPRIETARY AND CONFIDENTIAL  
**Copyright:** (c) 2025-2026 Capomastro Holdings Ltd. (Canada), Applied Physics Division  
**Patent(s) Pending**

---

## 1. Introduction

XPlenum is a custom RISC-V instruction set extension providing hardware-accelerated security operations for the PlenumNET quantum-resistant infrastructure. It integrates into the CVA6 (OpenHW Group) RV64GC core as a functional unit, adding 21 custom instructions and 12 CSRs.

### 1.1 Design Goals

1. **Single-cycle security checks** for domain isolation and capability enforcement
2. **Hardware-grade random number generation** via NIST SP 800-90A CTR_DRBG with AES-256
3. **Side-channel countermeasures** through hardware boolean masking
4. **Ternary cryptographic primitives** for GF(3) operations
5. **Zero-overhead signal processing** for hardware security monitoring

### 1.2 Opcode Allocation

XPlenum uses the `custom-0` opcode space (0x0B) as defined by the RISC-V ISA specification.

```
[31:25]  [24:20]  [19:15]  [14:12]  [11:7]  [6:0]
funct7    rs2      rs1      funct3    rd     0001011
```

---

## 2. Instruction Encoding

### 2.1 Functional Groups (funct3)

| funct3 | Binary | Group | Instructions |
|--------|--------|-------|-------------|
| 0 | 000 | Masking | TMASK, TUNMASK, TMASKR, TMASKRF |
| 1 | 001 | Domain Isolation | TDOMSET, TDOMCHK, TDOMCLR, TDOMXFR |
| 2 | 010 | Capability | TCAPLD, TCAPCHK, TCAPST, TCAPREV |
| 3 | 011 | Crypto/Rotation | TROTL, TROTR, TTBOX, TPERM |
| 4 | 100 | Trit Encoding | TTRIT, TDETRIT |
| 5 | 101 | Signal Processing | TSIGFLT, TSIGCMP, TSIGACC |
| 6 | 110 | Reserved | — |
| 7 | 111 | CSR Access | (internal CSR operations) |

---

## 3. Instruction Reference

### 3.1 Masking Instructions (funct3 = 000)

#### TMASK — Apply Mask
```
31:25    24:20    19:15    14:12    11:7    6:0
0000000  rs2      rs1      000      rd     0001011
```
**Operation:** `rd ← rs1 XOR rs2`  
**Precondition:** `XPSTATUS.MASK_EN = 1`  
**Exception:** `XP_EXC_MASK_FAULT (0x5)` if `MASK_EN = 0`  
**Cycles:** 1  
**Description:** Applies a boolean mask to protect sensitive data against DPA.

#### TUNMASK — Remove Mask
```
31:25    24:20    19:15    14:12    11:7    6:0
0000001  rs2      rs1      000      rd     0001011
```
**Operation:** `rd ← rs1 XOR rs2`  
**Precondition:** `XPSTATUS.MASK_EN = 1`  
**Exception:** `XP_EXC_MASK_FAULT (0x5)` if `MASK_EN = 0`  
**Cycles:** 1  
**Description:** Removes a previously applied mask. Semantically identical to TMASK but carries unmask intent for verification tooling.

#### TMASKR — Generate Random Mask
```
31:25    24:20    19:15    14:12    11:7    6:0
0000010  00000    00000    000      rd     0001011
```
**Operation:** `rd ← DRBG_Generate()`  
**Precondition:** `XPSTATUS.MASK_EN = 1`, DRBG ready (`drbg_ready_o = 1`)  
**Exception:** `XP_EXC_MASK_FAULT (0x5)` if `MASK_EN = 0`  
**Cycles:** 15 (AES-256 pipeline latency, pipelined throughput: 1 mask/cycle)  
**Description:** Generates a cryptographically random 32-bit mask from the SP 800-90A CTR_DRBG.

#### TMASKRF — Refresh Mask
```
31:25    24:20    19:15    14:12    11:7    6:0
0000011  00000    rs1      000      rd     0001011
```
**Operation:** `rd ← rs1 XOR DRBG_Generate()`  
**Precondition:** `XPSTATUS.MASK_EN = 1`, DRBG ready  
**Exception:** `XP_EXC_MASK_FAULT (0x5)` if `MASK_EN = 0`  
**Cycles:** 15  
**Description:** Refreshes an existing mask by XOR-ing with fresh DRBG output.

---

### 3.2 Domain Isolation Instructions (funct3 = 001)

#### TDOMSET — Set Domain Tag
```
31:25    24:20    19:15    14:12    11:7    6:0
0000000  rs2      rs1      001      00000  0001011
```
**Operation:** `domain_table[rs1[7:0]] ← rs2`  
**Precondition:** `XPSTATUS.DOM_EN = 1`  
**Exception:** `XP_EXC_DOM_VIOLATION (0x1)` if `DOM_EN = 0`  
**Cycles:** 1  
**rd:** Not written.

#### TDOMCHK — Check Domain Permission
```
31:25    24:20    19:15    14:12    11:7    6:0
0000001  rs2      rs1      001      rd     0001011
```
**Operation:** `rd ← (domain_table[rs1[7:0]] == rs2) ? 1 : 0`  
**Side effect:** If mismatch, `XPEXC_CAUSE ← XP_EXC_DOM_VIOLATION`  
**Precondition:** `XPSTATUS.DOM_EN = 1`  
**Exception:** `XP_EXC_DOM_VIOLATION (0x1)` on mismatch or `DOM_EN = 0`  
**Cycles:** 1

#### TDOMCLR — Clear Domain Tag
```
31:25    24:20    19:15    14:12    11:7    6:0
0000010  00000    rs1      001      00000  0001011
```
**Operation:** `domain_table[rs1[7:0]] ← 0`  
**Precondition:** `XPSTATUS.DOM_EN = 1`  
**Cycles:** 1  
**rd:** Not written.

#### TDOMXFR — Transfer Domain Ownership
```
31:25    24:20    19:15    14:12    11:7    6:0
0000011  rs2      rs1      001      00000  0001011
```
**Operation:** `domain_table[rs2[7:0]] ← domain_table[rs1[7:0]]; domain_table[rs1[7:0]] ← 0`  
**Precondition:** `XPSTATUS.DOM_EN = 1`  
**Cycles:** 2  
**rd:** Not written.

---

### 3.3 Capability Instructions (funct3 = 010)

#### TCAPLD — Load Capability
```
31:25    24:20    19:15    14:12    11:7    6:0
0000000  00000    rs1      010      rd     0001011
```
**Operation:** `rd ← cap_table[rs1[5:0]].permissions`  
**Precondition:** `XPSTATUS.CAP_EN = 1`, entry valid, entry not revoked  
**Exception:** `XP_EXC_CAP_INVALID (0x2)` if invalid, `XP_EXC_CAP_REVOKED (0x3)` if revoked  
**Cycles:** 1

#### TCAPCHK — Check Capability Bounds
```
31:25    24:20    19:15    14:12    11:7    6:0
0000001  rs2      rs1      010      rd     0001011
```
**Operation:** `rd ← (rs2 >= cap_table[rs1[5:0]].base AND rs2 < cap_table[rs1[5:0]].bound) ? 1 : 0`  
**Precondition:** `XPSTATUS.CAP_EN = 1`, entry valid and not revoked  
**Cycles:** 1

#### TCAPST — Store Capability
```
31:25    24:20    19:15    14:12    11:7    6:0
0000010  rs2      rs1      010      00000  0001011
```
**Operation:** Creates capability entry at index `rs1[5:0]` with base address `rs2`  
**Fields set:** `base = rs2`, `bound = rs2 + 0x1000`, `perms = 0x7 (RWX)`, `valid = 1`, `revoked = 0`  
**Precondition:** `XPSTATUS.CAP_EN = 1`  
**Cycles:** 1  
**rd:** Not written.

#### TCAPREV — Revoke Capability
```
31:25    24:20    19:15    14:12    11:7    6:0
0000011  00000    rs1      010      rd     0001011
```
**Operation:** `cap_table[rs1[5:0]].revoked ← 1; rd ← (was_valid ? 1 : 0)`  
**Precondition:** `XPSTATUS.CAP_EN = 1`  
**Cycles:** 1

---

### 3.4 Crypto/Rotation Instructions (funct3 = 011)

#### TROTL — Ternary Rotate Left
```
31:25    24:20    19:15    14:12    11:7    6:0
0000000  rs2      rs1      011      rd     0001011
```
**Operation:** `rd ← ROTL(rs1, rs2[4:0])`  
**Cycles:** 1

#### TROTR — Ternary Rotate Right
```
31:25    24:20    19:15    14:12    11:7    6:0
0000001  rs2      rs1      011      rd     0001011
```
**Operation:** `rd ← ROTR(rs1, rs2[4:0])`  
**Cycles:** 1

#### TTBOX — Ternary S-Box Lookup
```
31:25    24:20    19:15    14:12    11:7    6:0
0000010  00000    rs1      011      rd     0001011
```
**Operation:** `rd ← TRIT_SBOX[rs1[7:0]]` (243-entry GF(3) substitution table)  
**Cycles:** 1

#### TPERM — Ternary Permutation
```
31:25    24:20    19:15    14:12    11:7    6:0
0000011  rs2      rs1      011      rd     0001011
```
**Operation:** Permutes 16 trit-pairs in `rs1` according to key `rs2`  
**Cycles:** 1

---

### 3.5 Trit Encoding Instructions (funct3 = 100)

#### TTRIT — Binary to Ternary Encode
```
31:25    24:20    19:15    14:12    11:7    6:0
0000000  00000    rs1      100      rd     0001011
```
**Operation:** Encodes 16 binary pairs from `rs1` into balanced ternary (clamps `11` → `10`)  
**Cycles:** 1

#### TDETRIT — Ternary to Binary Decode
```
31:25    24:20    19:15    14:12    11:7    6:0
0000001  00000    rs1      100      rd     0001011
```
**Operation:** Decodes 16 balanced ternary pairs from `rs1` into binary  
**Cycles:** 1

---

### 3.6 Signal Processing Instructions (funct3 = 101)

#### TSIGFLT — Signal Filter
```
31:25    24:20    19:15    14:12    11:7    6:0
0000000  rs2      rs1      101      rd     0001011
```
**Operation:** 4-tap FIR filter: `rd ← (Σ rs1_byte[i] × rs2_byte[i]) >> 8`  
**Precondition:** `XPSTATUS.SIG_EN = 1`  
**Cycles:** 1

#### TSIGCMP — Signal Compare
```
31:25    24:20    19:15    14:12    11:7    6:0
0000001  rs2      rs1      101      rd     0001011
```
**Operation:** `rd ← (rs1 > rs2) ? 1 : (rs1 < rs2) ? -1 : 0` (signed comparison)  
**Precondition:** `XPSTATUS.SIG_EN = 1`  
**Cycles:** 1

#### TSIGACC — Signal Accumulate
```
31:25    24:20    19:15    14:12    11:7    6:0
0000010  rs2      rs1      101      rd     0001011
```
**Operation:** `sig_accumulator += rs1 × rs2; rd ← sig_accumulator`  
**Precondition:** `XPSTATUS.SIG_EN = 1`  
**Cycles:** 1

---

## 4. Custom CSR Registers

All XPlenum CSRs are in the M-mode custom read/write range (0x7C0–0x7CF).

| Address | Name | Access | Width | Description |
|---------|------|--------|-------|-------------|
| 0x7C0 | XPSTATUS | R/W | 4 | Global subsystem enable (MASK_EN, DOM_EN, CAP_EN, SIG_EN) |
| 0x7C1 | XPDOMID | R/W | 8 | Current domain ID |
| 0x7C2 | XPCAPBASE | R/W | 32 | Capability table base address |
| 0x7C3 | XPCAPBOUND | R/W | 32 | Capability table bound |
| 0x7C4 | XPMASK_SEED | R/W | 32 | DRBG seed (write triggers re-instantiation) |
| 0x7C5 | XPMASK_STATE | RO | 32 | Current DRBG output state |
| 0x7C6 | XPTRIT_MODE | R/W | 32 | Trit encoding mode |
| 0x7C7 | XPSIG_CFG | R/W | 32 | Signal processing configuration |
| 0x7C8 | XPEXC_CAUSE | RO | 4 | Last exception cause |
| 0x7C9 | XPEXC_ADDR | RO | 32 | Last exception address |
| 0x7CA | XPPERF_CNT | R/W | 32 | Performance counter (increments per instruction) |
| 0x7CB | XPVERSION | RO | 32 | Version register (0x010000 = v1.0.0) |

### 4.1 XPSTATUS Register Layout

```
Bit [0] — MASK_EN: Masking subsystem enable
Bit [1] — DOM_EN:  Domain isolation enable
Bit [2] — CAP_EN:  Capability subsystem enable
Bit [3] — SIG_EN:  Signal processing enable
Bits [31:4] — Reserved (read as zero)
```

---

## 5. Exception Codes

| Code | Name | Trigger |
|------|------|---------|
| 0x0 | XP_EXC_NONE | No exception |
| 0x1 | XP_EXC_DOM_VIOLATION | Domain permission check failed |
| 0x2 | XP_EXC_CAP_INVALID | Capability index invalid or entry not valid |
| 0x3 | XP_EXC_CAP_REVOKED | Access to revoked capability |
| 0x4 | XP_EXC_CAP_BOUNDS | Address outside capability bounds |
| 0x5 | XP_EXC_MASK_FAULT | Masking operation with subsystem disabled |
| 0x6 | XP_EXC_TRIT_OVERFLOW | Invalid trit encoding (value 3 in 2-bit field) |
| 0x7 | XP_EXC_PRIV_FAULT | Insufficient privilege or reserved instruction |

---

## 6. Hardware Integration

### 6.1 CVA6 Integration Points

XPlenum integrates as a functional unit in the CVA6 execute stage:
- **Issue:** Custom-0 opcode decoded in CVA6 issue stage → forwarded to XPlenum FU
- **Writeback:** Result returned via standard rd writeback path
- **Stall:** AES-256 pipeline operations (TMASKR, TMASKRF) stall pipeline for 14 cycles
- **Exception:** XPlenum exceptions converted to CVA6 illegal-instruction format

### 6.2 External Interfaces

| Port | Width | Direction | Description |
|------|-------|-----------|-------------|
| entropy_i | 256 | Input | External entropy for DRBG |
| entropy_valid_i | 1 | Input | Entropy data valid strobe |
| reseed_req_i | 1 | Input | DRBG reseed request |
| drbg_health_err_o | 1 | Output | DRBG health test failure |
| drbg_ready_o | 1 | Output | DRBG ready for mask generation |

---

## 7. Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-18 | Initial release — 21 instructions, 12 CSRs |

---

*XPlenum ISA Specification — Capomastro Holdings Ltd. (Canada)*  
*Applied Physics Division — Patent(s) Pending*
