<!-- Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada) Patent(s) Pending — All Rights Reserved Applied Physics Division PROPRIETARY AND CONFIDENTIAL -->

# Galois Formal Verification Engagement — Kickoff Preparation

**Prepared for**: Galois, Inc.  
**Prepared by**: Security Engineering, Capomastro Holdings Ltd.  
**Date**: February 18, 2026  
**Engagement Start**: March 15, 2026  
**Classification**: Internal / Audit-Ready

---

## 1. Engagement Overview

| Property | Detail |
|----------|--------|
| **Client** | Capomastro Holdings Ltd. / PlenumNET Platform |
| **Vendor** | Galois, Inc. |
| **Scope** | Formal verification of Ternary Kernel critical path components (15 proofs: 3 completed, 4 in-progress, 8 planned) |
| **Timeline** | March 15 – June 30, 2026 (Q1-Q3 2026 engagement, ~16 weeks) |
| **Engagement Type** | Formal Verification Audit with Spot-Check Reviews |
| **Budget** | Covered under existing R&D allocation |
| **Kickoff Meeting** | March 15, 2026 (90 minutes) |

### Strategic Context

The Ternary Kernel is a post-quantum operating system kernel built on balanced ternary (GF(3)) arithmetic with femtosecond timing, phase encryption, and hardware-software security integration. Formal verification targets critical-path components where correctness proofs are essential for security claims and regulatory compliance (CNSA 2.0, CMVP/FIPS, export control).

---

## 2. Pre-Engagement Deliverables

### 2.1 Proof Artifacts (3 Completed)

| # | Proof | System | File Path | Status |
|---|-------|--------|-----------|--------|
| 1 | Memory Allocator Safety | Coq | `src/kernel/proofs/allocator_safety.thy` | Complete |
| 2 | Capability System Integrity | Coq | `src/kernel/proofs/capability_integrity.thy` | Complete |
| 3 | GF(3) Arithmetic Correctness | Lean 4 | `src/kernel/proofs/gf3_arithmetic.thy` | Complete |

**Proof Properties**:
- **Allocator**: 6 properties proven (no double-free, no use-after-free, allocation uniqueness, bitmap consistency, bounded fragmentation, panic freedom)
- **Capability**: 5 properties proven (no forgery, authority monotonicity, revocation completeness, domain isolation, no confused deputy)
- **GF(3)**: 4 properties proven (field axioms, polynomial correctness, inverse existence, constant-time execution)

### 2.2 Source Code Access

| Component | Repository Path | Access Scope |
|-----------|-----------------|--------------|
| Kernel Source | `src/kernel/` | Full read access to Rust kernel, proofs, and specifications |
| RISC-V Extensions | `rtl/xplenum_*.v` | Verilog RTL for xplenum security extension |
| TVM Implementation | `src/kernel/src/vm/tvm.rs` | Ternary Virtual Machine bytecode interpreter |
| Cryptographic Core | `src/kernel/src/crypto/` | GF(3) arithmetic, capability tokens, phase encryption |

### 2.3 Architecture Documentation

| Document | Path | Relevance |
|----------|------|-----------|
| **Threat Model** | `docs/security/threat_model.md` | Security assumptions and threat landscape |
| **CNSA 2.0 Compliance** | `docs/compliance/` | Regulatory context and CMVP submission plan |
| **Formal Methodology** | `docs/security/formal_verification_methodology_2026-02-18.md` | Internal methodology assessment and assumptions |
| **Proof Roadmap** | `docs/security/proofs.md` | Comprehensive proof inventory and coverage goals |

### 2.4 Galois Engagement Package (Delivered Feb 17, 2026)

| Item | Location | Content |
|------|----------|---------|
| **10 Prioritized Questions** | `docs/security/galois_engagement_package.md` §4 | Methodology, scope, and deliverable clarifications |
| **Assumptions Log** | `docs/security/galois_engagement_package.md` §3 | 8 critical assumptions (A1–A8) with risk assessment |
| **Proof File Inventory** | `docs/security/galois_engagement_package.md` §2 | Status of all 15 critical-path proofs |
| **Timeline and Milestones** | `docs/security/galois_engagement_package.md` §5 | Proposed dates and deliverables |
| **Repository Structure** | `docs/security/galois_engagement_package.md` §8 | Directory map for proof files and source |

---

## 3. Kickoff Agenda (90 Minutes)

### Meeting Structure

| Time | Topic | Duration | Owner |
|------|-------|----------|-------|
| **0:00 – 0:15** | **Introductions and Project Overview** | 15 min | Salvi + Galois PM |
| | Attendees and roles | | |
| | Client goals and success criteria | | |
| | Engagement scope and constraints | | |
| **0:15 – 0:35** | **Review of 3 Completed Proofs** | 20 min | Proof Team Lead |
| | Allocator safety proof walkthrough | | |
| | Capability system integrity proof | | |
| | GF(3) arithmetic correctness proof | | |
| | Q&A on proof methodology | | |
| **0:35 – 0:50** | **Discussion of 4 In-Progress Proofs** | 15 min | Proof Team |
| | TVM instruction safety (40% complete) | | |
| | Phase encryption correctness (30% complete) | | |
| | Process scheduler fairness (70% complete) | | |
| | IPC message passing safety (30% complete) | | |
| **0:50 – 1:05** | **Tool Compatibility Assessment** | 15 min | Proof Team |
| | Coq for theorem proving | | |
| | Lean 4 for type-theoretic proofs | | |
| | CBMC for bounded model checking | | |
| | Kani for Rust-specific verification | | |
| | Isabelle/HOL for hardware assumptions | | |
| | Z3 vs. CVC5 SMT solver preferences | | |
| **1:05 – 1:20** | **Verification Strategy for Remaining 8 Proofs** | 15 min | Proof Team + Galois |
| | Priority ordering and sequencing | | |
| | Toolchain recommendations | | |
| | Coverage targets (95% by Q4 2026) | | |
| **1:20 – 1:30** | **Timeline, Milestones, and Communication Cadence** | 10 min | Engagement Lead |
| | Spot-check dates (April 15, May 15, June 15) | | |
| | Final report deadline (June 30) | | |
| | Weekly sync schedule and escalation path | | |

---

## 4. Key Discussion Points

### 4.1 k-Induction Depth Adequacy

**Current State**: We use k-induction depth of 15 for loop invariants in the allocator and scheduler proofs.

**Context**:
- Allocator bitmap scan loop: max iteration count = `max_frames / 64` ≈ 1,024 frames → loop depth ≤ 16
- Scheduler task iteration: max runnable task count ≤ 256 → loop depth ≤ 8
- None of the verified loops approach depth 15

**Galois Input Needed**: Is depth 15 adequate for kernels of this complexity? What depth do you recommend as margin for buffer overruns?

### 4.2 Isabelle/HOL Modeling of Rust Ownership Semantics

**Current State**: The capability system proof is formalized in Coq; scheduler proof uses Isabelle/HOL with a conservative model of Rust ownership.

**Key Assumption (A4)**: "Isabelle/HOL's type system correctly models Rust ownership semantics"

**Issue**: The Isabelle/HOL model over-approximates Rust's borrow checker — it may reject programs that Rust would accept, but any property proven in the model also holds for actual Rust code.

**Galois Input Needed**: Is this conservative approach consistent with your methodology? Should we refine the ownership model to match Rust's actual borrow checker rules?

### 4.3 Sequential Consistency Memory Model

**Current State**: Proofs assume sequentially consistent (SC) memory — the strongest possible memory ordering.

**Key Assumption (A6)**: "Concurrency model assumes sequentially consistent memory"

**Risk**: ARM and RISC-V use relaxed memory models (TSO for x86 is stronger). SC is safe for single-core proofs but may not hold for multi-core deployment.

**Galois Input Needed**: How should we transition from SC to weak memory model proofs (C11, ARM, RISC-V)? Do you have established methodology for relaxed memory verification?

### 4.4 TVM Instruction Set Formal Safety Model

**Current State**: TVM instruction set proof is 40% complete (62/160 opcodes formalized).

**Properties Under Verification**:
- Opcode decoding correctness
- Ring transition validation (ring-3 → ring-0 capability check)
- No undefined behavior on malformed bytecode
- Memory isolation between TVM execution contexts

**Galois Input Needed**: Which missing opcodes present the highest formal verification risk? Should we prioritize by frequency or complexity?

### 4.5 Phase Encryption Timing-Window Proofs

**Current State**: Phase encryption proof is 30% complete. Split correctness proven; timing window enforcement in active verification.

**Properties Under Verification**:
- Split/recombine correctness: `recombine(split(m, φ)) == m`
- Timing window enforcement: decryption fails outside valid [t₁, t₂] window
- Phase entropy bounds
- Forward secrecy guarantee

**Galois Input Needed**: For timing-dependent cryptography, do you recommend Lean 4 (type theory) or CBMC (bounded model checking) as the primary proof tool? Are there precedents for phase-based encryption verification?

### 4.6 Memory Model for RISC-V xplenum Extension

**Current State**: Hardware RTL (xplenum extension) is formalized in Verilog; formal verification deferred to post-engagement.

**Scope for This Engagement**: Out of scope (software-focused); defer to Trail of Bits engagement (Q2-Q3 2026 for side-channel evaluation).

**Galois Input Needed**: Should we include a light hardware assumptions review as part of this engagement, or keep it strictly to software?

---

## 5. Success Criteria for Engagement

### Primary Success Criteria

| Criterion | Metric | Target |
|-----------|--------|--------|
| **Proof Coverage Completion** | 15 critical-path proofs in defined/proven state | 100% |
| **Methodology Validation** | No critical gaps identified in verification approach | 0 gaps |
| **Tool Recommendations** | Galois provides guidance for remaining 8 proofs | Documented |
| **Progress Tracking** | Bi-weekly written progress reports | Complete |
| **Final Assessment** | Comprehensive audit report with findings and recommendations | By June 30 |

### Intermediate Milestones

| Date | Milestone | Success Metric |
|------|-----------|----------------|
| **April 15, 2026** | First spot-check (allocator, capability proofs) | No major gaps identified |
| **May 15, 2026** | Second spot-check (scheduler, GF(3) proofs) | Methodology confirmed for loop invariants |
| **June 15, 2026** | Third spot-check (TVM, IPC proofs) | Remaining 8 proofs prioritized by Galois |
| **June 30, 2026** | Final report delivery | Complete findings and roadmap for Q4 2026 |

### Acceptance Criteria for Final Report

1. **Soundness Assessment**: Galois confirms that completed 3 proofs are mathematically sound
2. **Assumption Validation**: All 8 critical assumptions (A1–A8) reviewed and assessed for risk
3. **Methodology Recommendations**: Guidance provided for proving remaining 8 critical-path proofs
4. **Tool Guidance**: Clear recommendation for Coq vs. Lean vs. CBMC vs. Kani for each proof type
5. **Risk Mitigation Plan**: Specific remediation steps if gaps identified in scheduler or IPC proofs

---

## 6. Risk Register for Engagement

| # | Risk | Impact | Probability | Mitigation |
|---|------|--------|-------------|-----------|
| **R1** | Proof complexity exceeds planned timeline | High | Medium | Prioritize proofs by CVSS severity; parallelize independent proofs |
| **R2** | Tool incompatibility (Coq vs Lean 4) | Medium | Low | Galois recommends unified tool stack; refactor proofs if needed |
| **R3** | Sequential consistency assumption invalidated | High | Medium | Engage weak memory model specialist; extend timeline if needed |
| **R4** | Scope creep from new threat vectors | Medium | Medium | Change control process; formal scope boundary at kickoff |
| **R5** | Key personnel unavailability (proof team) | Low | Low | Backup verification engineer assigned; knowledge transfer docs |
| **R6** | Galois methodology differs from internal approach | Medium | Low | Weekly syncs with senior Galois reviewer to surface conflicts early |

### Risk Escalation Path

- **Weekly Severity 1 Risks** (proofs unsound, major gaps): Escalate to Engagement Lead + Galois PM same day
- **Methodology Conflicts**: Surface at weekly sync; escalate to technical leads if consensus not reached
- **Timeline Pressures**: Assess at April 15 and May 15 spot-checks; adjust June 30 deadline if needed with mutual agreement

---

## 7. Contacts

### Capomastro Holdings Ltd.

| Role | Name | Email | Availability |
|------|------|-------|--------------|
| **PlenumNET Security Lead** | TBD | TBD | Full-time (engage for critical decisions) |
| **Engagement Lead (Salvi)** | Salvi | TBD | Full-time during engagement; async escalations outside hours |
| **Proof Team Lead** | TBD | TBD | Available for weekly syncs (Tue/Thu 10 AM – 12 PM PST) |
| **Technical POC (Crypto)** | TBD | TBD | On-call for GF(3), phase encryption questions |

### Galois, Inc.

| Role | Name | Email | Expected |
|------|------|-------|----------|
| **Engagement Manager** | TBD | TBD | Will confirm at kickoff |
| **Lead Proof Reviewer** | TBD | TBD | Will confirm at kickoff |
| **Secondary Reviewer** | TBD | TBD | Will confirm at kickoff |

### Meeting Logistics

- **Kickoff Date**: March 15, 2026
- **Duration**: 90 minutes
- **Platform**: Zoom (link TBD) or in-person (location TBD)
- **Time Zone**: PST (Pacific Standard Time)
- **Attendees**: Engagement lead, proof team lead, security lead, Galois PM + 2 reviewers

---

## 8. References

| Document | Purpose | Location |
|----------|---------|----------|
| **Galois Engagement Package** | Pre-engagement deliverables, assumptions, 10 prioritized questions | `docs/security/galois_engagement_package.md` |
| **Formal Verification Methodology** | Internal methodology assessment, assumptions A1–A8, key findings | `docs/security/formal_verification_methodology_2026-02-18.md` |
| **Proof Roadmap** | Comprehensive proof inventory (completed, in-progress, planned) | `docs/security/proofs.md` |
| **Threat Model** | Security assumptions, threat landscape, risk assessment | `docs/security/threat_model.md` |
| **Compliance Status** | CNSA 2.0, CMVP, export control context | `docs/compliance/reports/SELF-AUDIT-2026-02-15.md` |

### Related Documents (For Reference)

- `docs/PHASE-ENCRYPTION-SPEC.md` — Phase encryption technical specification
- `docs/VERILOG-FORMAL-VERIFICATION.md` — Hardware verification pointers (xplenum extension)
- `docs/security/risk_assessment.md` — Broader security risk profile
- `src/kernel/proofs/` — All proof files (organized by component)

---

## 9. Pre-Kickoff Checklist

| # | Item | Owner | Target Date | Status |
|---|------|-------|-------------|--------|
| 1 | Confirm kickoff date/time and attendees | Salvi | Feb 20, 2026 | _Pending_ |
| 2 | Finalize Zoom link and backup dial-in | Salvi | Feb 21, 2026 | _Pending_ |
| 3 | Grant Galois team GitHub read access to kernel proofs | Security Ops | Feb 28, 2026 | _Pending_ |
| 4 | Provide Isabelle/HOL environment setup instructions | Proof Team | Feb 28, 2026 | _Pending_ |
| 5 | Deliver proof artifact URLs (GitHub raw links or Overleaf share) | Proof Team | Feb 28, 2026 | _Pending_ |
| 6 | Confirm Galois review team roles and lead reviewer | Galois | Mar 1, 2026 | _Pending_ |
| 7 | Prepare slide deck for project overview (kickoff segment 1) | Salvi | Mar 10, 2026 | _Pending_ |
| 8 | Prepare proof walkthrough demos (allocator, capability, GF3) | Proof Team | Mar 10, 2026 | _Pending_ |

---

## 10. Post-Kickoff Next Steps

### Immediate (Week 1)

1. **Confirm Assumptions**: Document which of the 8 assumptions (A1–A8) Galois will focus on during engagement
2. **Tool Selection**: Galois recommends specific tools for remaining 8 proofs
3. **Proof Prioritization**: Finalize order: scheduler → TVM → IPC → boot chain (or Galois alternative)
4. **Weekly Sync Schedule**: Lock in recurring time (recommend Tue/Thu, 10 AM PST)

### Ongoing (Weeks 2–16)

- **Proof Development**: Kernel engineering team advances in-progress proofs with Galois guidance
- **Spot-Checks**: April 15, May 15, June 15 (each ~4 hours)
- **Weekly Syncs**: 30-min status + technical discussion
- **Escalation Log**: Track methodology conflicts, timeline risks, new assumptions

### Final (Week 16)

- **Final Report Submission**: June 30, 2026
- **Debrief Call**: Technical findings, recommendations for Q4 2026 proofs
- **Publication Prep**: Coordinate with Galois on redaction/disclosure scope

---

## Sign-Off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Engagement Lead (Salvi) | | Feb 18, 2026 | _Pending_ |
| Proof Team Lead | | Feb 18, 2026 | _Pending_ |
| Security Lead | | Feb 18, 2026 | _Pending_ |
| CFO / R&D Lead | | Feb 18, 2026 | _Pending_ |

---

*Document Control: Galois formal verification engagement kickoff preparation document. Finalized February 18, 2026. Kickoff scheduled March 15, 2026.*
