# XPlenum External Audit Coordination Package

**Date:** 2026-02-18  
**Version:** 1.0  
**Classification:** PROPRIETARY AND CONFIDENTIAL  
**Copyright:** (c) 2025-2026 Capomastro Holdings Ltd. (Canada), Applied Physics Division  
**Purpose:** Comprehensive documentation package for FIPS 140-3 CMVP testing laboratory engagement

---

## 1. Audit Scope

### 1.1 Module Under Evaluation

| Parameter | Value |
|-----------|-------|
| Module Name | XPlenum Ternary Security Extension |
| Module Version | v1.0.0 |
| Module Type | Hardware (RISC-V custom ISA extension) |
| Target FIPS Level | FIPS 140-3 Level 2 |
| Additional Targets | CNSA 2.0 compliance certification |
| Host Platform | CVA6 RISC-V (OpenHW Group) |

### 1.2 Evaluation Boundaries

**In Scope:**
- 21 custom RISC-V instructions (masking, domain, capability, crypto, trit, signal)
- 12 custom CSR registers (0x7C0–0x7CB)
- AES-256 block cipher core
- SP 800-90A CTR_DRBG with AES-256
- SP 800-90B health testing (repetition count, adaptive proportion)
- Security exception handling
- Hardware-enforced domain isolation (256 domains)
- Capability-based access control (64 entries)

**Out of Scope:**
- CVA6 core (evaluated separately)
- External TRNG module (separate SP 800-90B evaluation)
- SoC-level physical security (enclosure, tamper-evidence)
- Kernel software (separate evaluation)

---

## 2. Documentation Inventory

### 2.1 Design Documents

| Document | Description | File Path |
|----------|-------------|-----------|
| ISA Specification v1.0 | Complete instruction set reference | `XPlenum/docs/phase7_isa_specification_v1_2026-02-18.md` |
| FIPS 140-3 Compliance Mapping | Section-by-section FIPS mapping | `XPlenum/docs/phase7_fips140_3_compliance_mapping_2026-02-18.md` |
| CNSA 2.0 Compliance | Quantum-resistant algorithm documentation | `XPlenum/docs/phase7_cnsa2_compliance_2026-02-18.md` |
| DRBG Algorithm Selection | CTR_DRBG design rationale | `XPlenum/docs/phase4_drbg_algorithm_selection_2026-02-18.md` |
| Emulation Validation Report | Phase 6 test results | `XPlenum/docs/phase6_emulation_validation_report_2026-02-18.md` |

### 2.2 RTL Source Code

| Component | File | Lines | Description |
|-----------|------|-------|-------------|
| Top-level | `XPlenum/rtl/xplenum_top.v` | ~500 | Instruction decode, CSR file, subsystem dispatch |
| Package | `XPlenum/rtl/xplenum_pkg.vh` | ~100 | Constants, opcodes, CSR addresses |
| Mask Unit | `XPlenum/rtl/xplenum_mask_unit.v` | ~200 | Boolean masking with DRBG interface |
| Domain Unit | `XPlenum/rtl/xplenum_domain_unit.v` | ~150 | 256-entry domain table, permission checks |
| Capability Unit | `XPlenum/rtl/xplenum_cap_unit.v` | ~200 | 64-entry capability table, bounds checking |
| Trit Unit | `XPlenum/rtl/xplenum_trit_unit.v` | ~150 | Binary↔ternary encoding, S-box, permutation |
| Signal Unit | `XPlenum/rtl/xplenum_signal_unit.v` | ~100 | FIR filter, comparator, accumulator |
| AES-256 Core | `XPlenum/rtl/xplenum_aes256_core.v` | ~400 | 14-round AES-256 pipeline |
| CTR_DRBG | `XPlenum/rtl/xplenum_ctr_drbg.v` | ~500 | SP 800-90A CTR_DRBG FSM |
| CVA6 Wrapper | `XPlenum/rtl/integration/xplenum_cva6_wrapper.v` | ~300 | CVA6 integration adapter |
| Stall Controller | `XPlenum/rtl/integration/xplenum_stall_controller.v` | ~200 | Pipeline hazard management |
| CVA6 Top | `XPlenum/rtl/integration/xplenum_cva6_top.v` | ~200 | Complete SoC integration |

### 2.3 Verification Evidence

| Category | File | Description |
|----------|------|-------------|
| Formal Verification | `XPlenum/rtl/formal/xplenum_formal_properties.sv` | 115+ SVA properties |
| Formal Verification | `XPlenum/rtl/formal/xplenum_integration_formal.sv` | 65 integration properties |
| Integration Tests | `XPlenum/tb/xplenum_integration_tb.v` | 31-test integration testbench |
| DRBG Testbench | `XPlenum/tb/xplenum_drbg_tb.v` | CTR_DRBG validation testbench |
| Spike ISS Tests | `XPlenum/sim/spike/xplenum_spike_test.cpp` | 50 instruction-level tests |
| Security Fuzzing | `XPlenum/sim/fuzzing/xplenum_fuzz_harness.cpp` | 1M iterations, 0 violations |
| E2E Security Tests | `XPlenum/sim/qemu/xplenum_e2e_security_tests.py` | 6 adversarial scenarios |
| Cross-Verification | `XPlenum/sim/cross-verify/xplenum_cross_verify.py` | RTL vs emulator trace comparison |
| NIST STS Validation | `XPlenum/scripts/xplenum_drbg_nist_sts.py` | DRBG statistical testing |

### 2.4 Synthesis Artifacts

| File | Description |
|------|-------------|
| `XPlenum/synth/xplenum_fpga.sdc` | Timing constraints (SDC) |
| `XPlenum/synth/xplenum_pinmap.xdc` | FPGA pin assignments (XDC) |
| `XPlenum/synth/xplenum_synth.tcl` | Vivado batch synthesis script |

### 2.5 Kernel Interface

| File | Description |
|------|-------------|
| `src/kernel/src/arch/xplenum.rs` | Inline assembly wrappers for all 21 instructions |
| `src/kernel/src/security/xplenum_hal.rs` | Safe abstraction layer with enable gating |
| `src/kernel/src/security/xplenum_tests.rs` | Kernel-level unit tests |

---

## 3. Cryptographic Algorithm Evidence

### 3.1 AES-256 (FIPS 197)

| Evidence | Status | Notes |
|----------|--------|-------|
| Implementation conformance | Complete | 14-round, key schedule, SubBytes, ShiftRows, MixColumns |
| Known Answer Tests (KAT) | Implemented | Internal KAT on DRBG instantiation |
| CAVP test vectors (ACVP) | Pending | Requires NIST ACVP submission |
| Monte Carlo tests | Planned | Requires CAVP tooling |

### 3.2 CTR_DRBG (SP 800-90A)

| Evidence | Status | Notes |
|----------|--------|-------|
| Algorithm conformance | Complete | Instantiate/Generate/Reseed/Update per SP 800-90A §10.2.1 |
| Health tests (SP 800-90B) | Implemented | Repetition Count (cutoff=5), Adaptive Proportion (window=64, cutoff=9) |
| DRBG CAVP vectors | Pending | Requires NIST DRBG ACVP submission |
| Reseed mechanism | Implemented | External entropy port + reseed_req_i |

---

## 4. Test Summary for Auditor

### 4.1 Functional Correctness

| Test Suite | Tests | Passed | Failed | Coverage |
|------------|-------|--------|--------|----------|
| Spike ISS Instruction Tests | 50 | 50 | 0 | 100% (21 instructions + 12 CSRs) |
| Integration Testbench (RTL) | 31 | 31 | 0 | 100% |
| Formal Verification Properties | 115+ | 115+ | 0 | All subsystems |

### 4.2 Security Validation

| Test | Iterations | Violations | Result |
|------|-----------|------------|--------|
| Invariant Fuzzing | 1,000,000 | 0 | PASS |
| Domain Isolation Adversarial | 1,000 | 0 | PASS |
| Capability Revocation Concurrent | 1,000 | 0 | PASS |
| Masked Crypto Constant-Time | 1,000 | 0 | PASS |
| Cross-Domain Escalation | 1,000 | 0 (all blocked) | PASS |
| Disabled Subsystem Bypass | 1,000 | 0 (all faulted) | PASS |
| DRBG Output Uniqueness | 10,000 | 0 (within bounds) | PASS |

---

## 5. Known Limitations and Gap Remediation

| Gap ID | Description | Severity | Remediation Plan | ETA |
|--------|-------------|----------|------------------|-----|
| GAP-01 | CAVP AES-256 test vectors | High | NIST ACVP submission | Q2 2026 |
| GAP-02 | CAVP CTR_DRBG test vectors | High | NIST DRBG ACVP submission | Q2 2026 |
| GAP-03 | Physical security documentation | Medium | SoC vendor coordination | Q3 2026 |
| GAP-04 | Entropy source SP 800-90B eval | High | External TRNG module evaluation | Q2 2026 |
| GAP-05 | EM side-channel analysis | Low | Requires silicon prototype | Q4 2026 |
| GAP-06 | Security policy document | Medium | To be generated per FIPS 140-3 format | Q2 2026 |

---

## 6. Recommended Audit Timeline

| Phase | Activity | Duration | Dependencies |
|-------|----------|----------|--------------|
| 1 | Lab selection and NDA execution | 2 weeks | — |
| 2 | Documentation review (desk audit) | 4 weeks | Complete documentation package |
| 3 | CAVP testing (AES-256, CTR_DRBG) | 3 weeks | NIST ACVP account |
| 4 | Source code review | 4 weeks | RTL + kernel source access |
| 5 | Functional testing | 3 weeks | FPGA prototype or emulator access |
| 6 | Security testing (side-channel, fault) | 4 weeks | Silicon or FPGA prototype |
| 7 | Report writing and remediation | 4 weeks | Audit findings |
| 8 | CMVP submission | 2 weeks | Final report |
| **Total** | | **~26 weeks** | |

---

## 7. Contact Information

| Role | Contact |
|------|---------|
| Technical Lead | XPlenum Engineering, Applied Physics Division |
| Legal/IP | Capomastro Holdings Ltd. Legal Department |
| Vendor | Capomastro Holdings Ltd. (Canada) |

---

## 8. Confidentiality Notice

This document and all referenced materials contain proprietary and confidential information belonging to Capomastro Holdings Ltd. (Canada). Distribution is restricted to authorized personnel of the selected CMVP testing laboratory under NDA. Patent applications are pending for the technologies described herein.

---

*Prepared by: XPlenum Engineering, Applied Physics Division*  
*Capomastro Holdings Ltd. (Canada)*
