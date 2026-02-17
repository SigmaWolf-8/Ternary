<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  Patent(s) Pending — All Rights Reserved
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# Formal Verification Proofs Roadmap

**Version**: 1.0
**Date**: March 1, 2026
**Classification**: Internal / Audit-Ready
**Author**: Formal Methods Team, Capomastro Holdings Ltd.
**Status**: Published

---

## 1. Executive Summary

This document outlines the formal verification strategy for the Ternary Kernel, detailing which components have been proven correct, which proofs are in progress, and the roadmap for achieving comprehensive formal coverage. Formal verification is a cornerstone of the kernel's security posture, providing mathematical guarantees that critical components behave according to their specifications.

### Verification Status Overview

| Metric | Value |
|--------|-------|
| Components with Completed Proofs | 3 |
| Proofs In Progress | 4 |
| Proofs Planned | 8 |
| Total Proof Coverage | 65% (of critical path) |
| Lines of Proof Code | ~12,000 |
| Target Full Coverage | Q4 2026 |

---

## 2. Verification Methodology

### 2.1 Toolchain

| Tool | Purpose | Components |
|------|---------|-----------|
| **Coq** | Theorem proving for core algorithms | Memory allocator, capability system |
| **Lean 4** | Higher-order proofs, type-theoretic properties | GF(3) arithmetic, ternary logic |
| **Verilator + SymbiYosys** | Hardware RTL formal verification | RISC-V xplenum extensions |
| **CBMC** | Bounded model checking for Rust/C | Crypto primitives, timing logic |
| **Kani** | Rust-specific bounded model checking | Memory safety, panic freedom |

### 2.2 Proof Categories

1. **Memory Safety**: No use-after-free, no buffer overflows, no data races
2. **Functional Correctness**: Algorithm produces correct output for all valid inputs
3. **Information Flow**: No unauthorized information leakage between security domains
4. **Liveness**: System eventually makes progress (no deadlocks)
5. **Timing Guarantees**: Operations complete within bounded time (constant-time crypto)

### 2.3 Completion Calculation

Proof coverage contributes to the overall completion percentage:

```
completion_percentage = (
  (loc_tested / loc_total) * 0.4 +
  (test_count / total_possible_tests) * 0.3 +
  (proof_coverage_percentage / 100) * 0.3
) * 100
```

The 30% weight on proof coverage ensures formal verification is incentivized alongside testing.

---

## 3. Completed Proofs

### 3.1 Kernel Memory Allocator

| Property | Status |
|----------|--------|
| **Component** | `mm/frame_allocator.rs` |
| **LOC** | 1,200 |
| **Proof Tool** | Coq |
| **Proof LOC** | ~2,100 |
| **Properties Verified** | 6 |
| **Completion** | 100% |
| **Verified By** | Internal team, peer-reviewed |

**Properties Proven**:

1. **Allocation Safety**: Every allocated frame is unique and non-overlapping
2. **Deallocation Correctness**: Freed frames return to the available pool exactly once
3. **No Double-Free**: A frame cannot be freed twice without intervening allocation
4. **Bitmap Consistency**: The bitmap allocator state is always consistent with actual memory usage
5. **Bounded Fragmentation**: External fragmentation remains below 5% under steady-state workloads
6. **Panic Freedom**: No reachable panic paths under valid inputs

### 3.2 Capability System

| Property | Status |
|----------|--------|
| **Component** | `kernel/capability.rs` |
| **LOC** | 800 |
| **Proof Tool** | Coq |
| **Proof LOC** | ~1,800 |
| **Properties Verified** | 5 |
| **Completion** | 100% |
| **Verified By** | Internal team, peer-reviewed |

**Properties Proven**:

1. **Authority Monotonicity**: Capabilities can only be restricted, never amplified
2. **Delegation Integrity**: Delegated capabilities are strict subsets of the delegator's rights
3. **Revocation Completeness**: Revoking a capability revokes all descendant delegations
4. **Isolation**: Processes cannot access resources outside their capability set
5. **No Confused Deputy**: Authority is always checked against the caller's capabilities, not the callee's

### 3.3 GF(3) Arithmetic Core

| Property | Status |
|----------|--------|
| **Component** | `crypto/gf3.rs` |
| **LOC** | 450 |
| **Proof Tool** | Lean 4 |
| **Proof LOC** | ~900 |
| **Properties Verified** | 4 |
| **Completion** | 100% |
| **Verified By** | Internal team |

**Properties Proven**:

1. **Field Axioms**: Addition and multiplication satisfy field axioms over GF(3)
2. **Polynomial Correctness**: Polynomial arithmetic produces correct results for all coefficient combinations
3. **Inverse Existence**: Every non-zero element has a multiplicative inverse
4. **Constant-Time Execution**: All operations execute in constant time regardless of input values

---

## 4. Proofs In Progress

### 4.1 Scheduler & IPC

| Property | Status |
|----------|--------|
| **Component** | `kernel/scheduler.rs`, `kernel/ipc.rs` |
| **LOC** | 4,500 |
| **Proof Tool** | Coq + Kani |
| **Current Coverage** | 70% |
| **Target Completion** | Q1 2026 (March 31) |
| **Responsible Team** | Kernel Engineering |

**Properties Being Verified**:

1. **Priority Inversion Freedom**: Higher-priority tasks are never blocked by lower-priority tasks indefinitely
2. **Deadlock Freedom**: No circular wait conditions in IPC message passing
3. **Fairness**: Every runnable task eventually receives CPU time
4. **Message Integrity**: IPC messages arrive unmodified and in order (per channel)
5. **Bounded Latency**: Context switch completes within bounded cycle count

**Current Progress**: Properties 1-3 proven, properties 4-5 in active verification.

### 4.2 HPTP Core Protocol

| Property | Status |
|----------|--------|
| **Component** | `hptp/core.rs`, `hptp/sync.rs` |
| **LOC** | 3,200 |
| **Proof Tool** | CBMC + Kani |
| **Current Coverage** | 40% |
| **Target Completion** | Q2 2026 (April 30) |
| **Responsible Team** | Timing Engineering |

**Properties Being Verified**:

1. **Clock Monotonicity**: Timestamps never decrease within a session
2. **Drift Bounded**: Clock drift stays within spec (< 5 ppm for crystal tier)
3. **Fallback Correctness**: Tier transitions follow the defined state machine
4. **Jitter Bounds**: Timing jitter remains within acceptable variance
5. **Recovery Guarantee**: System returns to PTP within bounded time after failure clears

### 4.3 AES-256-GCM Implementation

| Property | Status |
|----------|--------|
| **Component** | `crypto/aes_gcm.rs` |
| **LOC** | 950 |
| **Proof Tool** | CBMC |
| **Current Coverage** | 60% |
| **Target Completion** | Q1 2026 (March 31) |
| **Responsible Team** | Cryptography |

**Properties Being Verified**:

1. **Constant-Time Execution**: No input-dependent timing variations
2. **Tag Verification Correctness**: Authentication tags correctly validate/reject
3. **Nonce Uniqueness Enforcement**: Nonce reuse detection and prevention
4. **Key Schedule Correctness**: Round key generation matches NIST specification

### 4.4 Phase Encryption

| Property | Status |
|----------|--------|
| **Component** | `crypto/phase.rs` |
| **LOC** | 1,600 |
| **Proof Tool** | Lean 4 + CBMC |
| **Current Coverage** | 30% |
| **Target Completion** | Q2 2026 (June 30) |
| **Responsible Team** | Applied Physics |

**Properties Being Verified**:

1. **Split/Recombine Correctness**: Encrypted data correctly reconstructs after phase recombination
2. **Timing Window Enforcement**: Decryption fails outside the valid timing window
3. **Phase Entropy**: Phase values have sufficient entropy for security parameter
4. **Forward Secrecy**: Compromise of current phase does not reveal past plaintexts

---

## 5. Planned Proofs

| # | Component | Tool | Target Date | Priority |
|---|-----------|------|-------------|----------|
| 1 | Ternary VM Instruction Set | Lean 4 | Q2 2026 | High |
| 2 | ML-KEM Key Exchange | CBMC + Kani | Q2 2026 | High |
| 3 | ML-DSA Signature Scheme | CBMC | Q3 2026 | High |
| 4 | Page Table Management | Coq | Q3 2026 | Medium |
| 5 | TTP Transport Protocol | CBMC | Q3 2026 | Medium |
| 6 | Filesystem Integrity | Kani | Q3 2026 | Medium |
| 7 | Device Driver Framework | CBMC | Q4 2026 | Low |
| 8 | Binary Compatibility Layer | Lean 4 | Q4 2026 | Low |

---

## 6. Verification Infrastructure

### 6.1 CI/CD Integration

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Code Change  │────▶│  Unit Tests  │────▶│  Proof Check │
│  (git push)   │     │  (cargo test)│     │  (Coq/Lean)  │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                                          ┌───────▼───────┐
                                          │  CBMC/Kani    │
                                          │  Model Check  │
                                          └───────┬───────┘
                                                  │
                                          ┌───────▼───────┐
                                          │  Proof Report │
                                          │  Generation   │
                                          └───────────────┘
```

### 6.2 Proof Regression Policy

- All completed proofs run on every commit touching the verified component
- Proof failures block merge to main branch
- Proof regressions treated as critical security events (logged to audit service)

### 6.3 External Audit Partners

| Partner | Scope | Timeline |
|---------|-------|----------|
| **Galois, Inc.** | Formal verification audit of kernel proofs | Q2-Q3 2026 |
| **Trail of Bits** | Cryptographic implementation review | Q3 2026 |
| **Riscure** | Side-channel evaluation and hardware verification | Q2-Q3 2026 |

---

## 7. Coverage Goals

### Quarterly Targets

| Quarter | Target Coverage | Components |
|---------|----------------|-----------|
| Q1 2026 | 40% of critical path | Memory, Capability, Scheduler, AES-GCM |
| Q2 2026 | 60% of critical path | + HPTP, Phase Encryption, VM, ML-KEM |
| Q3 2026 | 80% of critical path | + ML-DSA, Page Tables, TTP, Filesystem |
| Q4 2026 | 95% of critical path | + Drivers, Compatibility, remaining components |

### Definition of "Critical Path"

Components on the critical path are those where a bug could lead to:
- Cryptographic key compromise
- Privilege escalation
- Timing integrity violation
- Memory safety violation

Total critical path components: 15 (out of 50+ tracked).

---

## 8. API Integration

Proof progress is tracked via the Implementation Status Tracker API:

```
GET /api/security/implementation/metrics     - Per-category proof coverage
GET /api/security/implementation/summary     - Overall proof statistics
GET /api/security/implementation/milestones  - Proof completion milestones
```

Key fields: `proof_count`, `proof_coverage_percentage`, `external_audit_status`, `external_auditor`.

---

## 9. Risk Assessment

### Verification Gaps

| Gap | Risk Level | Mitigation |
|-----|-----------|------------|
| VM instruction set unverified | Medium | Extensive fuzzing + property-based testing |
| ML-KEM proof pending | Medium | Reference implementation comparison |
| Filesystem integrity unverified | Low | Extensive integration testing |
| Driver framework unverified | Low | Sandboxed execution environment |

### Confidence Levels

| Component | Confidence | Basis |
|-----------|-----------|-------|
| Memory Allocator | Very High | Coq proof complete |
| Capability System | Very High | Coq proof complete |
| GF(3) Arithmetic | Very High | Lean 4 proof complete |
| Scheduler | High | 70% proof coverage + 94% test coverage |
| Crypto Primitives | High | Constant-time verification + NIST test vectors |
| HPTP Protocol | Medium | 40% proof coverage, active verification |

---

*Document Control: This roadmap is maintained by the Formal Methods Team and reviewed quarterly. Progress is tracked via the Implementation Status Tracker service.*
