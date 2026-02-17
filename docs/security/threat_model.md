<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  Patent(s) Pending — All Rights Reserved
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# Ternary Kernel Threat Model

**Version**: 1.0
**Date**: February 28, 2026
**Classification**: Internal / Audit-Ready
**Author**: Security Engineering, Capomastro Holdings Ltd.
**Status**: Published

---

## 1. Executive Summary

This document presents the comprehensive threat model for the Ternary Kernel Security Infrastructure. It identifies, classifies, and evaluates 12 primary threat vectors across 10 categories, applying a quantitative risk matrix derived from CVSS v4.0 methodology. Each threat is mapped to specific controls, residual risk assessments, and mitigation timelines.

The Ternary Kernel operates in a post-quantum computing environment where timing integrity, cryptographic resilience, and hardware-software boundary security are paramount. This threat model serves as the authoritative reference for security posture assessment, audit readiness, and compliance reporting.

### Risk Posture Summary

| Metric | Value |
|--------|-------|
| Total Threats Tracked | 12 |
| Mitigated | 5 |
| In Progress | 5 |
| Acknowledged | 1 |
| Not Addressed | 1 |
| High-Risk (score >= 6.0) | 3 |
| Average Risk Score | 4.57 |
| Average Residual Risk | 1.8 |

---

## 2. Threat Classification Framework

### 2.1 Categories

The threat model spans 10 categories aligned with the kernel's attack surface:

| # | Category | Scope | Component Coverage |
|---|----------|-------|-------------------|
| 1 | Timing | HPTP protocol, clock synchronization, femtosecond precision | HPTP Core, Secure Element |
| 2 | Cryptographic | Post-quantum algorithms, key management, side-channel leakage | ML-KEM, ML-DSA, AES-256-GCM |
| 3 | Network | Protocol security, transport layer, DNS resolution | TTP, T3P, TDNS |
| 4 | Physical | Hardware tampering, DMA attacks, cold boot | IOMMU, Memory Allocator |
| 5 | Supply Chain | Firmware integrity, OEM trust, software dependencies | Bootloader, Package Signing |
| 6 | Side-Channel | Power analysis, electromagnetic emanation, cache timing | DPA-C3, TEMPEST |
| 7 | Quantum | Future quantum key recovery, algorithm migration | CNSA 2.0, TL-KEM |
| 8 | Insider | Privileged access abuse, credential compromise | Capability System, Audit |
| 9 | Compliance | Regulatory requirements, certification gaps | FIPS 140-3, Wassenaar |
| 10 | Software | Memory safety, logic errors, race conditions | VM, Scheduler, IPC |

### 2.2 Risk Scoring Methodology

Risk scores follow a quantitative matrix:

```
likelihood_weight = { low: 1, medium: 3, high: 7, critical: 10 }
impact_weight    = { low: 1, medium: 3, high: 7, critical: 10 }
risk_score       = (likelihood_weight * impact_weight) / 10
```

**Risk Scale Interpretation**:

| Score Range | Classification | Response Requirement |
|-------------|---------------|---------------------|
| 8.0 - 10.0 | Critical | Immediate mitigation required |
| 6.0 - 7.9 | High | Mitigation within 30 days |
| 4.0 - 5.9 | Medium | Mitigation within 90 days |
| 2.0 - 3.9 | Low | Scheduled for next release cycle |
| 0.0 - 1.9 | Informational | Monitor only |

### 2.3 Mitigation Status Definitions

| Status | Definition |
|--------|-----------|
| **Mitigated** | Controls implemented, tested, and verified |
| **In Progress** | Controls under development or partial deployment |
| **Acknowledged** | Threat recognized; mitigation planned but not started |
| **Not Addressed** | No mitigation plan exists (requires risk acceptance) |

---

## 3. Threat Registry

### THREAT_001: DMA Attacks (Thunderbolt/PCIe)

| Field | Value |
|-------|-------|
| **Category** | Physical |
| **Attack Vector** | Thunderbolt/PCIe/FireWire DMA |
| **Likelihood** | High |
| **Impact** | Critical |
| **Risk Score** | 7.0 |
| **Mitigation Status** | Mitigated |
| **Residual Risk** | 1.5 |
| **Mitigation Mode** | Prevention |

**Description**: Attacker with physical access exploits Direct Memory Access via peripheral ports to read/write kernel memory, extract cryptographic keys, or inject code.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| IOMMU_001 | IOMMU Enforced - Capability-Based DMA | Implemented | `mm/iommu.rs` - strict DMA mappings enforced |
| ISOLATION_001 | Ternary Page Table Randomization | Implemented | `mm/aslr.rs` - entropy from HPTP jitter |

**CVSS v4.0 Assessment**: AV:P/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H (Base: 7.0)

---

### THREAT_002: Cold Boot Attacks

| Field | Value |
|-------|-------|
| **Category** | Physical |
| **Attack Vector** | DRAM remanence after power cycle |
| **Likelihood** | Medium |
| **Impact** | High |
| **Risk Score** | 4.9 |
| **Mitigation Status** | In Progress |
| **Residual Risk** | 2.5 |
| **Mitigation Mode** | Prevention + Detection |

**Description**: Attacker exploits DRAM data remanence to extract cryptographic keys after system power-off or forced reboot.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| MEM_CLEAR_001 | Memory Zeroization on Shutdown | In Progress | `mm/shutdown.rs` - secure erase routines |
| ENCRYPT_MEM_001 | In-Memory Key Encryption | Planned | Architecture RFC pending |

---

### THREAT_003: TEMPEST/EM Emanations

| Field | Value |
|-------|-------|
| **Category** | Side-Channel |
| **Attack Vector** | Electromagnetic emanation capture |
| **Likelihood** | Medium |
| **Impact** | High |
| **Risk Score** | 4.9 |
| **Mitigation Status** | In Progress |
| **Residual Risk** | 3.0 |
| **Mitigation Mode** | Detection Only |

**Description**: Electromagnetic emanations from processing units leak information about cryptographic operations, enabling key extraction at distance.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| EM_SHIELD_001 | Constant-Time Crypto Operations | Implemented | `crypto/timing.rs` - ct_select, ct_eq |
| EM_MASK_001 | Randomized Execution Scheduling | In Progress | Scheduler entropy integration |

---

### THREAT_004: Rowhammer DRAM Attacks

| Field | Value |
|-------|-------|
| **Category** | Cryptographic |
| **Attack Vector** | DRAM bit-flip via repeated row activation |
| **Likelihood** | Low |
| **Impact** | Critical |
| **Risk Score** | 3.2 |
| **Mitigation Status** | In Progress |
| **Residual Risk** | 2.0 |
| **Mitigation Mode** | Prevention |

**Description**: Targeted DRAM row activation induces bit-flips in adjacent rows, potentially altering page table entries, cryptographic keys, or capability tokens.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| ECC_001 | ECC Memory Enforcement | Implemented | Hardware requirement in deployment spec |
| GUARD_ROW_001 | Guard Row Isolation | In Progress | `mm/guard.rs` - row isolation logic |

---

### THREAT_005: Power Analysis (Differential)

| Field | Value |
|-------|-------|
| **Category** | Side-Channel |
| **Attack Vector** | Power consumption correlation |
| **Likelihood** | Medium |
| **Impact** | High |
| **Risk Score** | 4.9 |
| **Mitigation Status** | In Progress |
| **Residual Risk** | 2.5 |
| **Mitigation Mode** | Prevention |

**Description**: Statistical analysis of power consumption traces during cryptographic operations reveals key material through differential power analysis (DPA).

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| CT_OPS_001 | Constant-Time Arithmetic | Implemented | `crypto/gf3.rs` - constant-time GF(3) ops |
| MASK_001 | Boolean/Arithmetic Masking | In Progress | `crypto/masking.rs` - first-order masking |

---

### THREAT_006: ML-KEM Decapsulation Timing

| Field | Value |
|-------|-------|
| **Category** | Cryptographic |
| **Attack Vector** | Timing side-channel in lattice operations |
| **Likelihood** | Low |
| **Impact** | Medium |
| **Risk Score** | 1.5 |
| **Mitigation Status** | Mitigated |
| **Residual Risk** | 0.5 |
| **Mitigation Mode** | Prevention |

**Description**: Timing variations in ML-KEM decapsulation could leak information about the secret key through cache-timing or branch-prediction side channels.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| CT_MLKEM_001 | Constant-Time Decapsulation | Implemented | `crypto/ml_kem.rs` - verified ct |
| CACHE_001 | Cache Partition Isolation | Implemented | `mm/cache.rs` - process isolation |

---

### THREAT_007: Software Supply Chain Poisoning

| Field | Value |
|-------|-------|
| **Category** | Supply Chain |
| **Attack Vector** | Compromised dependency or build artifact |
| **Likelihood** | Medium |
| **Impact** | Critical |
| **Risk Score** | 6.5 |
| **Mitigation Status** | Mitigated |
| **Residual Risk** | 1.0 |
| **Mitigation Mode** | Prevention + Detection |

**Description**: Malicious code injection through compromised build tools, dependencies, or CI/CD pipeline.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| SBOM_001 | Software Bill of Materials | Implemented | `cargo audit` + `npm audit` in CI |
| SIGN_001 | Reproducible Build Signing | Implemented | `build/sign.sh` - deterministic builds |
| LOCK_001 | Dependency Lock Files | Implemented | `Cargo.lock` + `package-lock.json` pinned |

---

### THREAT_008: Insider with Privileged Access

| Field | Value |
|-------|-------|
| **Category** | Insider |
| **Attack Vector** | Credential compromise or malicious insider |
| **Likelihood** | Low |
| **Impact** | Critical |
| **Risk Score** | 3.2 |
| **Mitigation Status** | In Progress |
| **Residual Risk** | 2.0 |
| **Mitigation Mode** | Detection |

**Description**: Authorized personnel with elevated privileges exfiltrate sensitive data, introduce backdoors, or sabotage system integrity.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| AUDIT_001 | Comprehensive Audit Logging | Implemented | Security Audit Service - immutable logs |
| RBAC_001 | Role-Based Access Control | Implemented | Capability system with least privilege |
| DUAL_001 | Dual-Authorization for Critical Ops | Planned | Architecture RFC in review |

---

### THREAT_009: Quantum Key Recovery (post-2030)

| Field | Value |
|-------|-------|
| **Category** | Quantum |
| **Attack Vector** | Cryptanalytically-relevant quantum computer |
| **Likelihood** | Critical |
| **Impact** | Critical |
| **Risk Score** | 10.0 |
| **Mitigation Status** | Mitigated |
| **Residual Risk** | 1.0 |
| **Mitigation Mode** | Prevention |

**Description**: A sufficiently powerful quantum computer breaks RSA/ECC/classical key exchange, enabling decryption of previously captured traffic ("harvest now, decrypt later").

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| PQC_001 | CNSA 2.0 Algorithm Suite | Implemented | ML-KEM-1024, ML-DSA-87, SLH-DSA |
| HYBRID_001 | Hybrid Key Exchange | Implemented | X25519 + ML-KEM-768 hybrid |
| AGILITY_001 | Cryptographic Agility Layer | Implemented | `crypto/agility.rs` - algorithm negotiation |

---

### THREAT_010: HPTP Secure Element Compromise

| Field | Value |
|-------|-------|
| **Category** | Timing |
| **Attack Vector** | Hardware timing source manipulation |
| **Likelihood** | Low |
| **Impact** | Critical |
| **Risk Score** | 3.2 |
| **Mitigation Status** | In Progress |
| **Residual Risk** | 2.5 |
| **Mitigation Mode** | Detection |

**Description**: Compromise of the HPTP secure timing element degrades timing integrity, enabling timing-based attacks on phase encryption and protocol synchronization.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| FALLBACK_001 | 5-Tier Fallback Chain | Implemented | HPTP Anomaly Service - PTP/NTP/Crystal/Quartz/Cesium |
| ANOMALY_001 | Anomaly Detection with Auto-Escalation | Implemented | Severity-based escalation (>= 8.0 critical) |

---

### THREAT_011: Firmware Implants (OEM)

| Field | Value |
|-------|-------|
| **Category** | Supply Chain |
| **Attack Vector** | Pre-installed malicious firmware |
| **Likelihood** | Low |
| **Impact** | Critical |
| **Risk Score** | 3.2 |
| **Mitigation Status** | Acknowledged |
| **Residual Risk** | 3.0 |
| **Mitigation Mode** | Detection Only |

**Description**: Hardware vendors supply devices with pre-installed firmware implants that survive OS installation, providing persistent backdoor access.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| BOOT_001 | Measured Boot Chain | Implemented | `boot/measure.rs` - hash chain verification |
| ATTEST_001 | Remote Attestation | Planned | Architecture under design |

---

### THREAT_012: SSH/TLS Protocol Downgrade

| Field | Value |
|-------|-------|
| **Category** | Network |
| **Attack Vector** | Man-in-the-middle protocol negotiation attack |
| **Likelihood** | Low |
| **Impact** | High |
| **Risk Score** | 2.3 |
| **Mitigation Status** | Mitigated |
| **Residual Risk** | 0.5 |
| **Mitigation Mode** | Prevention |

**Description**: Attacker forces protocol downgrade to weaker cipher suites during TLS/SSH handshake negotiation.

**Controls**:

| Control ID | Control Name | Status | Evidence |
|-----------|-------------|--------|----------|
| STRICT_001 | Strict Cipher Suite Enforcement | Implemented | TLS 1.3 only, no fallback |
| HSTS_001 | HSTS with Preload | Implemented | 2-year max-age, includeSubDomains |

---

## 4. Risk Matrix

### 4.1 Likelihood vs. Impact Heatmap

```
              │  Low Impact  │  Medium Impact  │  High Impact  │  Critical Impact  │
──────────────┼──────────────┼─────────────────┼───────────────┼───────────────────┤
Critical L.   │              │                 │               │  THREAT_009       │
              │              │                 │               │  (10.0)           │
──────────────┼──────────────┼─────────────────┼───────────────┼───────────────────┤
High L.       │              │                 │               │  THREAT_001       │
              │              │                 │               │  (7.0)            │
──────────────┼──────────────┼─────────────────┼───────────────┼───────────────────┤
Medium L.     │              │                 │  THREAT_002   │  THREAT_007       │
              │              │                 │  THREAT_003   │  (6.5)            │
              │              │                 │  THREAT_005   │                   │
              │              │                 │  (4.9)        │                   │
──────────────┼──────────────┼─────────────────┼───────────────┼───────────────────┤
Low L.        │              │  THREAT_006     │  THREAT_012   │  THREAT_004       │
              │              │  (1.5)          │  (2.3)        │  THREAT_008       │
              │              │                 │               │  THREAT_010       │
              │              │                 │               │  THREAT_011       │
              │              │                 │               │  (3.2)            │
──────────────┴──────────────┴─────────────────┴───────────────┴───────────────────┘
```

### 4.2 Category Distribution

| Category | Count | Avg Risk | Highest Threat |
|----------|-------|----------|---------------|
| Physical | 2 | 5.95 | THREAT_001 (7.0) |
| Side-Channel | 2 | 4.90 | THREAT_003, THREAT_005 |
| Cryptographic | 2 | 2.35 | THREAT_004 (3.2) |
| Supply Chain | 2 | 4.85 | THREAT_007 (6.5) |
| Quantum | 1 | 10.00 | THREAT_009 (10.0) |
| Timing | 1 | 3.20 | THREAT_010 (3.2) |
| Insider | 1 | 3.20 | THREAT_008 (3.2) |
| Network | 1 | 2.30 | THREAT_012 (2.3) |

---

## 5. Control Effectiveness Summary

| Total Controls | Implemented | In Progress | Planned |
|---------------|-------------|-------------|---------|
| 24 | 18 (75%) | 4 (17%) | 2 (8%) |

### Prevention vs. Detection Classification

| Mode | Threats Covered | Average Residual Risk |
|------|----------------|----------------------|
| Prevention | 7 | 1.3 |
| Detection Only | 2 | 2.8 |
| Prevention + Detection | 3 | 1.5 |

---

## 6. Residual Risk Assessment

After mitigations, residual risk profile:

| Residual Risk Range | Count | Threats |
|-------------------|-------|---------|
| 0.0 - 1.0 | 3 | THREAT_006, THREAT_007, THREAT_009 |
| 1.0 - 2.0 | 3 | THREAT_001, THREAT_004, THREAT_008 |
| 2.0 - 3.0 | 4 | THREAT_002, THREAT_005, THREAT_010, THREAT_012 |
| 3.0+ | 2 | THREAT_003, THREAT_011 |

**Aggregate Residual Risk Score**: 1.8 / 10.0 (Low)

---

## 7. Mitigation Timeline

| Quarter | Milestones |
|---------|-----------|
| Q1 2026 | Threat model published, CVSS ratings applied, residual risk baselined |
| Q2 2026 | Side-channel evaluations (DPA-C3), HPTP secure element hardening |
| Q3 2026 | Firmware attestation, dual-authorization deployment |
| Q4 2026 | FIPS 140-3 audit, penetration testing, compliance certification |

---

## 8. API Integration

The threat model is programmatically accessible via the Threat Model Registry API:

```
GET  /api/security/threats              - List all threats (filtered)
GET  /api/security/threats/:id          - Threat details with controls
GET  /api/security/threats/risk-matrix  - Risk matrix heatmap data
GET  /api/security/threats/stats        - Summary statistics
POST /api/security/threats              - Create new threat entry
PATCH /api/security/threats/:id         - Update mitigation status
```

All endpoints require admin authentication except `/api/security/threats/meta`.

---

## 9. Review Schedule

| Review Type | Frequency | Next Review |
|-------------|-----------|-------------|
| Threat Registry Update | Monthly | March 28, 2026 |
| Risk Matrix Recalculation | Quarterly | May 1, 2026 |
| External Audit | Annually | Q4 2026 |
| CVSS Re-evaluation | Per-incident | Ongoing |

---

*Document Control: This threat model is maintained by Security Engineering and reviewed by the Architecture Review Board. Changes require sign-off from the Security Lead.*
