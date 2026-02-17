<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  Patent(s) Pending — All Rights Reserved
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# Side-Channel Evaluation Framework

**Version**: 1.0
**Date**: March 31, 2026
**Classification**: Internal / Audit-Ready
**Author**: Security Engineering, Capomastro Holdings Ltd.
**Status**: Published

---

## 1. Executive Summary

This document establishes the formal evaluation framework for side-channel resistance in the Ternary Kernel. Side-channel attacks exploit observable physical phenomena (timing, power consumption, electromagnetic emanation, cache behavior) to extract cryptographic secrets. The framework defines evaluation criteria, test methodologies, pass/fail thresholds, and the certification path for each cryptographic component.

### Framework Scope

| Domain | Components | Evaluation Method |
|--------|-----------|------------------|
| Timing Side-Channels | AES-256-GCM, ML-KEM, ML-DSA, Phase Encryption | Constant-time verification (CBMC + dudect) |
| Power Analysis | All crypto primitives | DPA-C3 evaluation (Riscure Inspector) |
| Electromagnetic | HPTP secure element, crypto accelerators | TEMPEST-class measurement |
| Cache Side-Channels | VM instruction execution, memory allocator | Flush+Reload, Prime+Probe testing |
| Microarchitectural | Spectre/Meltdown variants | Speculation barrier verification |

---

## 2. Evaluation Methodology

### 2.1 Test-Vector Independent Leakage Detection (TVLA)

The primary statistical method for side-channel evaluation is TVLA (ISO 17825):

```
Methodology:
1. Collect N traces (power/EM) during cryptographic operation
2. Split into two sets: fixed plaintext vs. random plaintext
3. Compute Welch's t-test between the two sets
4. If |t| > 4.5 at any sample point → LEAKAGE DETECTED

Pass Criteria:
  |t| < 4.5 for all sample points across 100,000 traces
  Confidence level: 99.999%
```

### 2.2 Constant-Time Verification

All cryptographic operations must execute in constant time regardless of input values:

```
Verification Methods:

1. Static Analysis (CBMC)
   - Model check all conditional branches
   - Verify no secret-dependent control flow
   - Verify no secret-dependent memory access patterns

2. Dynamic Analysis (dudect)
   - Run 10M iterations with fixed vs. random inputs
   - Welch's t-test on execution time distributions
   - Pass: |t| < 4.5

3. Assembly Inspection
   - Manual review of generated assembly
   - Verify absence of: cmov (conditional on secret), variable-time
     division, secret-dependent array indexing
   - Automated via objdump + pattern matching
```

### 2.3 Evaluation Tiers

| Tier | Rigor Level | Trace Count | Statistical Threshold | Applicable To |
|------|------------|-------------|----------------------|--------------|
| Tier 1 | Basic | 10,000 | t < 4.5 | Non-critical components |
| Tier 2 | Standard | 100,000 | t < 4.5 | Standard crypto operations |
| Tier 3 | Enhanced | 1,000,000 | t < 4.5 | Key generation, signing |
| Tier 4 | Certification | 10,000,000 | t < 4.5 + higher-order | FIPS 140-3 boundary |

---

## 3. Component Evaluation Status

### 3.1 AES-256-GCM

| Property | Status |
|----------|--------|
| **Constant-Time** | Verified (CBMC + dudect) |
| **DPA Resistance** | Tier 2 TVLA PASS (100K traces) |
| **Cache Isolation** | Implemented (T-table elimination) |
| **Masking Order** | First-order boolean masking |
| **FIPS Boundary** | Pending Tier 4 evaluation |

**Findings**:
- T-table implementation replaced with bitsliced S-box (no cache-dependent lookups)
- GCM polynomial multiplication uses constant-time carry-less multiply
- No leakage detected at 100K trace threshold

### 3.2 ML-KEM (Kyber)

| Property | Status |
|----------|--------|
| **Constant-Time** | Verified (CBMC) |
| **DPA Resistance** | Tier 2 TVLA PASS (100K traces) |
| **NTT Implementation** | Constant-time butterfly operations |
| **Rejection Sampling** | Timing-safe implementation |
| **Decapsulation** | Verified constant-time (FO transform) |

**Findings**:
- Fujisaki-Okamoto transform implemented with constant-time comparison
- NTT butterfly operations use Montgomery multiplication (constant-time)
- Rejection sampling uses constant-time SHAKE-256 expansion

### 3.3 ML-DSA (Dilithium)

| Property | Status |
|----------|--------|
| **Constant-Time** | Partially verified |
| **DPA Resistance** | Pending Tier 2 evaluation |
| **Polynomial Arithmetic** | Constant-time NTT |
| **Hint Generation** | Under review for timing leakage |
| **Target** | Full verification by Q3 2026 |

**Findings**:
- Core polynomial arithmetic verified constant-time
- Hint bit computation may have variable-time path (under investigation)
- Signing routine requires additional masking for DPA resistance

### 3.4 Phase Encryption

| Property | Status |
|----------|--------|
| **Constant-Time** | In Progress |
| **DPA Resistance** | Not yet evaluated |
| **Phase Split** | Constant-time verified |
| **Phase Recombine** | Under verification |
| **Timing Window** | Timing-safe check implemented |

**Findings**:
- Phase split operation verified constant-time
- Recombination path under active formal verification (30% complete)
- Timing window enforcement uses constant-time comparison

### 3.5 GF(3) Arithmetic

| Property | Status |
|----------|--------|
| **Constant-Time** | Verified (Lean 4 proof) |
| **DPA Resistance** | Tier 1 TVLA PASS (10K traces) |
| **Trit Operations** | All operations proven constant-time |
| **Polynomial Arithmetic** | Proven correct and constant-time |

**Findings**:
- Formally proven constant-time via Lean 4 theorem prover
- All GF(3) operations execute in exactly the same number of cycles
- No secret-dependent memory access patterns

---

## 4. DPA-C3 Evaluation Protocol

### 4.1 Overview

DPA-C3 (Differential Power Analysis - Countermeasure Certification Campaign) is the structured evaluation protocol used for power analysis resistance certification.

### 4.2 Evaluation Phases

```
Phase 1: Baseline Characterization (2 weeks)
├── Equipment calibration
├── Target board characterization
├── Trigger point identification
└── Baseline trace collection (unprotected reference)

Phase 2: First-Order Analysis (4 weeks)
├── CPA (Correlation Power Analysis) attack
├── TVLA with 100K traces
├── Identify leakage points (if any)
└── Document countermeasure effectiveness

Phase 3: Higher-Order Analysis (4 weeks)
├── Second-order DPA (if first-order masking present)
├── Template attacks
├── Multivariate analysis
└── Residual leakage assessment

Phase 4: Certification Report (2 weeks)
├── Findings documentation
├── Countermeasure recommendations
├── Compliance mapping (FIPS 140-3 Level 3)
└── Residual risk assessment
```

### 4.3 Equipment and Environment

| Equipment | Specification | Purpose |
|-----------|--------------|---------|
| Oscilloscope | LeCroy WaveRunner 8404M (4 GHz, 40 GS/s) | Trace capture |
| Current Probe | Riscure EM-FI Probe Set | Power measurement |
| EM Probe | Langer EMV-Technik near-field probes | EM measurement |
| Target Board | Custom evaluation board with exposed SMA | Controlled measurement |
| Analysis Software | Riscure Inspector SCA | Statistical analysis |

### 4.4 Pass/Fail Criteria

| Test | Pass Condition | Fail Condition |
|------|---------------|----------------|
| First-order CPA | No key recovery with 100K traces | Key byte recovered |
| TVLA t-test | All t < 4.5 (100K traces) | Any t >= 4.5 |
| Template attack | Success rate < random (1/256) with 10K traces | Success rate > 10% |
| Second-order DPA | No key recovery with 1M traces (if masked) | Key recovery possible |

---

## 5. Cache Side-Channel Mitigation

### 5.1 Attack Vectors

| Attack | Mechanism | Affected Components |
|--------|-----------|-------------------|
| Flush+Reload | Shared memory page monitoring | Crypto libraries |
| Prime+Probe | Cache set contention | AES T-table (eliminated) |
| Spectre v1 | Bounds check bypass | Array indexing in crypto |
| Spectre v2 | Branch target injection | Indirect calls |

### 5.2 Mitigations Implemented

| Mitigation | Status | Evidence |
|-----------|--------|----------|
| T-table elimination (AES) | Implemented | Bitsliced S-box in `crypto/aes.rs` |
| Process cache isolation | Implemented | `mm/cache.rs` - L1/L2 partitioning |
| Speculation barriers | Implemented | `lfence` at privilege boundaries |
| Constant-time memory access | Implemented | No secret-dependent array indexing |
| Page coloring | Planned | Architecture RFC in review |

---

## 6. Evaluation Timeline

| Milestone | Date | Deliverable |
|-----------|------|------------|
| AES-GCM Tier 2 complete | Feb 2026 | TVLA report |
| ML-KEM Tier 2 complete | Mar 2026 | TVLA report |
| GF(3) Tier 1 complete | Mar 2026 | TVLA report |
| DPA-C3 Phase 1 start | Apr 2026 | Baseline characterization |
| ML-DSA evaluation complete | Jun 2026 | Tier 2 TVLA report |
| DPA-C3 Phase 2 complete | Jun 2026 | First-order analysis |
| Phase Encryption evaluation | Jul 2026 | Constant-time report |
| DPA-C3 Phase 3 complete | Aug 2026 | Higher-order analysis |
| DPA-C3 Phase 4 (certification) | Sep 2026 | Final certification report |
| FIPS 140-3 submission | Oct 2026 | Certification package |

---

## 7. Compliance Mapping

### 7.1 FIPS 140-3 (Level 3)

| FIPS Requirement | Section | Framework Coverage |
|-----------------|---------|-------------------|
| Non-invasive attack mitigation | AS11.35 | DPA-C3 evaluation |
| Fault injection resistance | AS11.37 | Glitch detection in HPTP |
| Environmental failure protection | AS09.42 | Temperature monitoring |
| Key zeroization | AS07.12 | Memory clear on shutdown |

### 7.2 CNSA 2.0

| Requirement | Coverage |
|------------|---------|
| ML-KEM-1024 constant-time | Verified |
| ML-DSA-87 constant-time | In progress |
| AES-256 side-channel resistance | Tier 2 PASS |
| SHA-384/512 constant-time | Verified |

---

## 8. Reporting and Tracking

Side-channel evaluation results are tracked in the Implementation Status Tracker:

```
Component: "DPA-C3 Side-Channel Eval"
Category: "testing"
Status: "in_progress"
Completion: 40%
External Auditor: "Riscure"
External Audit Status: "in_progress"
```

Results feed into the Security Dashboard:
```
GET /api/security/dashboard
→ implementation.by_category.testing
→ implementation.by_category.side_channel
```

---

*Document Control: This framework is maintained by Security Engineering and the Cryptography team. Evaluation results require sign-off from the external audit partner before publication.*
