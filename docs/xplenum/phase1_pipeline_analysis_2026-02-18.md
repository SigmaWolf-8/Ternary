# XPlenum Phase 1.3 — CVA6 Pipeline Architecture Analysis

**Capomastro Holdings Ltd. — Applied Physics Division**
**Date: February 18, 2026**
**Classification: CONFIDENTIAL**

---

## 1. CVA6 Pipeline Stages

```
┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
│ PC Gen  │ → │   IF    │ → │   ID    │ → │  Issue  │ → │   EX    │ → │   WB    │
│ Stage 1 │   │ Stage 2 │   │ Stage 3 │   │ Stage 4 │   │ Stage 5 │   │ Stage 6 │
└─────────┘   └─────────┘   └─────────┘   └─────────┘   └─────────┘   └─────────┘
                                                │              │
                                                │         ┌────┴────┐
                                          Scoreboard      │  Units  │
                                          Hazard Det.     ├─────────┤
                                                          │   ALU   │
                                                          │  Branch │
                                                          │  LSU    │
                                                          │  FPU    │
                                                          │ XPlenum │ ← NEW
                                                          └─────────┘
```

### Stage Details

| Stage | CVA6 Module | Key Signals | XPlenum Interaction |
|-------|-------------|-------------|---------------------|
| 1. PC Gen | `frontend.sv` | `pc_if`, `branch_predict` | None |
| 2. IF | `frontend.sv` | `instr_rdata_if`, `fetch_valid` | None |
| 3. ID | `decoder.sv` | `instruction_o`, `is_illegal_i`, `fu_o` (functional unit select) | Decode Custom-0 opcode, set `fu_o = FU_XPLENUM` |
| 4. Issue | `issue_stage.sv` / `scoreboard.sv` | `rs1_data`, `rs2_data`, `rd_addr`, `issue_valid` | Register XPlenum destination in scoreboard for WAW/RAW hazard detection |
| 5. EX | `ex_stage.sv` | `fu_data_i`, `result_o`, `valid_o` | XPlenum top module receives operands, produces result |
| 6. WB/Commit | `commit_stage.sv` | `wdata_o`, `we_o`, `commit_ack` | XPlenum result written to register file via commit multiplexer |

---

## 2. XPlenum Port Mapping to CVA6

### Instruction Decode Interface

| XPlenum Port | Width | CVA6 Source Signal | CVA6 Source Module | Timing |
|---|---|---|---|---|
| `instruction[31:0]` | 32 | `instruction_o` | `decoder.sv` | Available at ID stage output (Stage 3) |
| `instr_valid` | 1 | `issue_valid && (fu == FU_XPLENUM)` | `issue_stage.sv` | Asserted when Issue stage dispatches to XPlenum (Stage 4) |
| `rs1_data[31:0]` | 32 | `operand_a_i` | `issue_stage.sv` via `regfile.sv` | Available at Issue stage output, forwarded from EX if needed |
| `rs2_data[31:0]` | 32 | `operand_b_i` | `issue_stage.sv` via `regfile.sv` | Available at Issue stage output, forwarded from EX if needed |

### Result Interface

| XPlenum Port | Width | CVA6 Destination Signal | CVA6 Dest Module | Timing |
|---|---|---|---|---|
| `rd_data[31:0]` | 32 | `xplenum_result` → `result_o` mux | `ex_stage.sv` | Must be valid 1 cycle after `instr_valid` for single-cycle ops; multi-cycle uses `valid_o` handshake |
| `rd_write_en` | 1 | `xplenum_valid` → `valid_o` mux | `ex_stage.sv` | Asserted when result is ready |
| `rd_addr[4:0]` | 5 | Carried through pipeline from decode | `scoreboard.sv` | Tracked by scoreboard; XPlenum does not need to provide this separately |

### Exception Interface

| XPlenum Port | Width | CVA6 Destination Signal | CVA6 Dest Module | Timing |
|---|---|---|---|---|
| `xp_exception` | 1 | `exception_o.valid` | `ex_stage.sv` → `commit_stage.sv` | Asserted simultaneously with result |
| `xp_exc_code[3:0]` | 4 | `exception_o.cause` | `commit_stage.sv` | Mapped to custom mcause values (0x18–0x1F range) |

---

## 3. Pipeline Hazard Integration

### Scoreboard Registration

CVA6's Issue stage uses a scoreboard to track in-flight instructions. XPlenum instructions must be registered in the scoreboard to prevent:

- **RAW (Read After Write)**: Subsequent instruction reads XPlenum's destination before result is ready
- **WAW (Write After Write)**: Two instructions write to same register; ordering must be preserved

**Integration point**: In `issue_stage.sv`, when `fu_o == FU_XPLENUM`, the scoreboard entry must include:
- Destination register (`rd`)
- Functional unit identifier (`FU_XPLENUM`)
- Expected latency (1 cycle for most ops; variable for multi-cycle)

### Data Forwarding

CVA6 supports result forwarding from EX stage back to Issue stage. XPlenum results must participate in this forwarding network:

```
EX Stage Output (XPlenum result) ──→ Issue Stage Forwarding Mux
                                          ↓
                                   Next instruction's rs1/rs2
```

### Multi-Cycle Stall Protocol

For XPlenum operations requiring >1 cycle (T-box substitution with validation, DRBG generate):

1. XPlenum asserts `busy` signal
2. Issue stage stalls dispatch (does not issue new instruction to XPlenum)
3. XPlenum deasserts `busy` and asserts `valid_o` when result is ready
4. Issue stage resumes

This matches CVA6's existing protocol for the FPU (which also has variable-latency operations).

---

## 4. CSR Integration

### CVA6 CSR Architecture

CVA6's CSR register file is in `csr_regfile.sv`. Custom CSRs are added by:

1. Adding address decode logic for 0x7C0–0x7CB range
2. Routing read/write data to XPlenum's CSR file
3. Privilege checks: all XPlenum CSRs require M-mode access

### CSR Signal Flow

```
csr_regfile.sv                    xplenum_top.v
┌──────────────┐                 ┌──────────────┐
│ addr decode  │ ──csr_addr───→ │ CSR address  │
│ read data mux│ ←─csr_rdata──  │ CSR read     │
│ write enable │ ──csr_wen────→ │ CSR write    │
│ write data   │ ──csr_wdata──→ │ CSR wdata    │
│ priv check   │ (M-mode only)  │              │
└──────────────┘                 └──────────────┘
```

### Privilege Enforcement

XPlenum CSRs at 0x7C0–0x7CB fall in the machine-level read/write range (0x7C0–0x7FF). CVA6's existing privilege check logic in `csr_regfile.sv` already enforces M-mode access for this range. No additional privilege logic is needed.

---

## 5. Exception Flow

### XPlenum Exception Mapping to mcause

| XPlenum Exception | Code | mcause Value | Description |
|---|---|---|---|
| `XP_EXC_NONE` | 0x0 | — | No exception |
| `XP_EXC_DOM_VIOLATION` | 0x1 | 0x18 | Domain permission violation |
| `XP_EXC_CAP_INVALID` | 0x2 | 0x19 | Invalid capability index |
| `XP_EXC_CAP_REVOKED` | 0x3 | 0x1A | Revoked capability access |
| `XP_EXC_CAP_BOUNDS` | 0x4 | 0x1B | Capability bounds violation |
| `XP_EXC_MASK_FAULT` | 0x5 | 0x1C | Masking subsystem disabled |
| `XP_EXC_TRIT_OVERFLOW` | 0x6 | 0x1D | Invalid trit encoding |
| `XP_EXC_PRIV_FAULT` | 0x7 | 0x1E | Insufficient privilege |

mcause values 0x18–0x1E are in the custom/reserved range for platform-specific exceptions.

### Exception Delivery

```
xplenum_top.xp_exception ──→ ex_stage.exception_o.valid
xplenum_top.xp_exc_code  ──→ ex_stage.exception_o.cause (mapped to mcause)
                              ↓
                         commit_stage.sv
                              ↓
                         controller.sv (pipeline flush)
                              ↓
                         PC ← MTVEC (trap handler)
```

---

## 6. 64-Bit XLEN Adaptation

### Current XPlenum: 32-bit

XPlenum's existing RTL uses 32-bit data paths (`rs1_data[31:0]`, `rd_data[31:0]`). CVA6 in RV64 mode provides 64-bit register values.

### Adaptation Strategy

| Approach | Description | Recommended |
|----------|-------------|-------------|
| Zero-extend inputs | Pass `rs1_data[31:0]` from lower 32 bits of 64-bit register | Yes (Phase 2) |
| Sign-extend outputs | Sign-extend `rd_data[31:0]` to 64 bits for register writeback | Yes (Phase 2) |
| Full 64-bit upgrade | Widen all XPlenum data paths to 64 bits | Deferred to v2.0 |

For Phase 2 integration, XPlenum operates on the lower 32 bits of 64-bit registers. This is consistent with RV64I's W-suffix instructions (ADDW, SUBW) which operate on 32-bit values within 64-bit registers.

---

## 7. Identified Hook Points

| # | File | Line/Region | Modification Required |
|---|------|-------------|----------------------|
| H1 | `core/ariane_pkg.sv` | `fu_t` enum | Add `FU_XPLENUM` functional unit type |
| H2 | `core/decoder.sv` | Opcode case statement | Add `7'b0001011` (Custom-0) decode branch |
| H3 | `core/issue_stage.sv` | FU dispatch mux | Add XPlenum dispatch path |
| H4 | `core/ex_stage.sv` | FU instantiation region | Instantiate `xplenum_top` |
| H5 | `core/ex_stage.sv` | Result mux | Add XPlenum result to output multiplexer |
| H6 | `core/csr_regfile.sv` | CSR address decode | Add 0x7C0–0x7CB range |
| H7 | `core/commit_stage.sv` | Exception handling | Map XPlenum exceptions to mcause |
| H8 | `core/controller.sv` | Stall logic | Handle XPlenum multi-cycle stalls |
