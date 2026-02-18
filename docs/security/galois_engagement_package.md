<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  Patent(s) Pending — All Rights Reserved
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# Galois Formal Verification Engagement Package

**Prepared for**: Galois, Inc.
**Prepared by**: Security Engineering, Capomastro Holdings Ltd.
**Date**: February 17, 2026
**Engagement Type**: Formal Verification Audit
**Proposed Timeline**: March 15 – June 30, 2026 (~16 weeks)

---

## 1. Project Overview

The Ternary Kernel is a post-quantum operating system kernel built on balanced ternary (GF(3)) arithmetic with femtosecond timing, phase encryption, and hardware-software security integration. Formal verification targets the critical-path components where correctness proofs are essential for security claims.

---

## 2. Proof File Inventory

### 2.1 Completed Proofs (3)

| # | Proof | File Path | Proof System | Status | Key Property |
|---|-------|-----------|-------------|--------|-------------|
| 1 | Memory Allocator Safety | `src/kernel/proofs/allocator_safety.thy` | Isabelle/HOL | Complete | No double-free, no use-after-free, frame isolation |
| 2 | Capability System Integrity | `src/kernel/proofs/capability_integrity.thy` | Isabelle/HOL | Complete | No capability forgery, monotonic revocation, domain isolation |
| 3 | GF(3) Arithmetic Correctness | `src/kernel/proofs/gf3_arithmetic.thy` | Isabelle/HOL | Complete | Field axioms (associativity, commutativity, distributivity, inverses) |

### 2.2 In-Progress Proofs (4)

| # | Proof | File Path | Proof System | Status | Target Completion |
|---|-------|-----------|-------------|--------|-------------------|
| 4 | Scheduler Fairness | `src/kernel/proofs/scheduler_fairness.thy` | Isabelle/HOL | 60% | April 2026 |
| 5 | TVM Bytecode Safety | `src/kernel/proofs/tvm_safety.thy` | Isabelle/HOL | 40% | May 2026 |
| 6 | IPC Message Safety | `src/kernel/proofs/ipc_safety.thy` | Isabelle/HOL | 30% | May 2026 |
| 7 | Boot Chain Integrity | `src/kernel/proofs/boot_chain.thy` | Isabelle/HOL | 20% | June 2026 |

### 2.3 Planned Proofs (4)

| # | Proof | Priority | Target |
|---|-------|----------|--------|
| 8 | Phase Encryption Correctness | High | Q3 2026 |
| 9 | HPTP Protocol Safety | High | Q3 2026 |
| 10 | Filesystem Integrity | Medium | Q4 2026 |
| 11 | Network Stack Isolation | Medium | Q4 2026 |

### 2.4 Coverage Summary

| Category | Proven | In Progress | Planned | Total | Coverage |
|----------|--------|-------------|---------|-------|----------|
| Memory Management | 1 | 0 | 0 | 1 | 100% |
| Security/Capability | 1 | 0 | 0 | 1 | 100% |
| Arithmetic/Crypto | 1 | 0 | 1 | 2 | 50% |
| Scheduling/IPC | 0 | 2 | 0 | 2 | 0% (WIP) |
| VM/Bytecode | 0 | 1 | 0 | 1 | 0% (WIP) |
| Boot/Firmware | 0 | 1 | 0 | 1 | 0% (WIP) |
| Timing/Protocol | 0 | 0 | 1 | 1 | 0% |
| Network/FS | 0 | 0 | 2 | 2 | 0% |
| **Total** | **3** | **4** | **4** | **11** | **27%** |

**Target**: 65% coverage by end of engagement (June 30), 95% by Q4 2026.

---

## 3. Assumptions Log

The following assumptions underpin the current proof methodology:

| # | Assumption | Context | Risk Level |
|---|-----------|---------|-----------|
| A1 | k-induction depth of 15 is sufficient for all loop invariants | Allocator, scheduler proofs | Medium |
| A2 | All loop terminations are provable via well-founded orderings | Used in allocator_safety.thy | Low |
| A3 | Hardware correctly implements ECC memory (no silent corruption) | Underpins Rowhammer mitigation | Low |
| A4 | Isabelle/HOL's type system correctly models Rust ownership semantics | GF(3) and capability proofs | Medium |
| A5 | Clock sources provide monotonically increasing timestamps | HPTP timing proofs (planned) | Low |
| A6 | Concurrency model assumes sequentially consistent memory | Scheduler fairness proof | High |
| A7 | GF(3) arithmetic matches hardware implementation exactly | No floating-point involved; integer arithmetic | Low |
| A8 | Capability tokens are unforgeable given AES-256-GCM integrity | Crypto assumption; not formally proven in capability proof | Medium |

**Note**: Assumptions A4 and A6 are high priority for Galois review. If Isabelle/HOL modeling does not faithfully capture Rust ownership or relaxed memory ordering, proof conclusions may not hold.

---

## 4. Open Questions for Galois

### 4.1 Methodology

1. **k-induction depth**: Is depth 15 sufficient for the loop structures in our allocator and scheduler? What is your recommended depth for kernels of this complexity?

2. **SMT Solver**: We use Z3 as our primary solver. Do you prefer CVC5 or another solver for specific proof obligations? Any known Z3 limitations for balanced ternary arithmetic?

3. **Isabelle version**: We are on Isabelle 2024. Do you require a specific version? Any compatibility concerns with your toolchain?

4. **Proof granularity**: Our proofs target module-level correctness (e.g., "the allocator never double-frees"). Do you recommend function-level or line-level proof granularity for audit purposes?

### 4.2 Scope

5. **Priority ordering**: We propose: scheduler proof > TVM compiler > IPC > boot chain. Does Galois agree with this priority, or would a different order be more efficient for your methodology?

6. **Concurrency proofs**: Assumption A6 (sequential consistency) may not hold on ARM/RISC-V. How should we handle relaxed memory model proofs? Do you have an established methodology?

7. **Hardware assumptions**: Should hardware assumptions (A3, A5) be explicit axioms in the proof, or should we attempt to prove them from lower-level specifications?

### 4.3 Deliverables

8. **Interim reports**: Can you provide monthly spot-check reports (April, May, June) on proof progress?

9. **Rejected proofs**: If you find a proof unsound, what is your recommended remediation process? Will you provide specific counterexamples?

10. **Public disclosure**: We intend to publish your final report (redacted if needed). Are there restrictions on what can be disclosed?

---

## 5. Proposed Timeline

| Date | Milestone | Owner |
|------|-----------|-------|
| **March 15, 2026** | Kickoff meeting (90 min) | Salvi + Galois PM |
| **March 29, 2026** | Design review (2 hours) | Proof Team + Galois |
| **April 15, 2026** | First spot-check (allocator, capability proofs) | Galois |
| **May 15, 2026** | Second spot-check (scheduler, GF(3) proofs) | Galois |
| **June 15, 2026** | Third spot-check (TVM, IPC proofs) | Galois |
| **June 30, 2026** | Final report delivery | Galois |

### Kickoff Agenda (March 15, 90 minutes)

1. **Introduction & scope confirmation** (15 min)
2. **Methodology walkthrough** — k-induction, SMT strategy, proof granularity (20 min)
3. **Proof file review** — walk through completed proofs, identify concerns (20 min)
4. **Open questions discussion** (20 min)
5. **Timeline and milestone confirmation** (10 min)
6. **Next steps and action items** (5 min)

---

## 6. Pre-Engagement Deliverables Checklist

| # | Item | Status |
|---|------|--------|
| 1 | Proof files organized with comments | Complete |
| 2 | `proofs.md` linked to actual file paths | Complete |
| 3 | Assumptions log (this document, Section 3) | Complete |
| 4 | Open questions list (this document, Section 4) | Complete |
| 5 | Proposed timeline and kickoff agenda | Complete |
| 6 | Threat model context (`threat_model.md`) | Published |
| 7 | Repository access for Galois team | Pending (March 1) |
| 8 | Isabelle environment setup instructions | Pending (March 1) |

---

## 7. Contact Information

| Role | Name | Availability |
|------|------|-------------|
| Engagement Lead | Salvi | Full-time during engagement |
| Proof Team Lead | TBD | Available for weekly syncs |
| Security Engineering | Security Team | On-call for questions |

---

## 8. Repository Structure (Relevant Paths)

```
src/kernel/
  proofs/
    allocator_safety.thy     # Complete
    capability_integrity.thy  # Complete
    gf3_arithmetic.thy       # Complete
    scheduler_fairness.thy   # In progress
    tvm_safety.thy           # In progress
    ipc_safety.thy           # In progress
    boot_chain.thy           # In progress
  crypto/
    gf3.rs                   # GF(3) arithmetic
    capability.rs            # Capability system
    ml_kem.rs                # ML-KEM implementation
    agility.rs               # Crypto agility layer
  mm/
    allocator.rs             # Memory allocator
    page_table.rs            # Page table management
  process/
    scheduler.rs             # Process scheduler
    ipc.rs                   # Inter-process communication
  vm/
    tvm.rs                   # Ternary Virtual Machine
  boot/
    measure.rs               # Measured boot chain
docs/security/
  threat_model.md            # Threat model v1.0
  proofs.md                  # Formal verification roadmap
```

---

## Sign-Off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Engagement Lead | Salvi | Feb 17, 2026 | _Pending_ |
| Proof Team Lead | | | _Pending_ |
| Security Lead | | | _Pending_ |

---

*Document Control: Pre-engagement package for Galois formal verification audit. Finalized February 17, 2026.*
