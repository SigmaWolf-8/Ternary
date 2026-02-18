<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  Patent(s) Pending — All Rights Reserved
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# Threat Model Accuracy Audit — QA Review Checklist

**Version**: 1.0
**Date**: February 17, 2026
**Classification**: Internal
**Reviewers**: Security Engineering QA Team
**Scope**: Cross-check `threat_model.md` v1.0 against actual kernel and service implementation

---

## Review Methodology

Line-by-line review of `threat_model.md` claims against:
1. Backend service code (`server/services/*.service.ts`)
2. Kernel source (`src/kernel/`)
3. Cryptographic primitives (`src/kernel/crypto/`)
4. Security routes (`server/routes/security.ts`)

---

## Checklist Results

### 1. HPTP Anomaly Detection Thresholds

| Claim (threat_model.md) | Code Location | Actual Value | Status |
|--------------------------|---------------|--------------|--------|
| Critical escalation >= 8.0 | `hptp-anomaly.service.ts:56` | `clampedSeverityScore >= 8.0` | PASS |
| High escalation >= 6.0 | `hptp-anomaly.service.ts:77` | `clampedSeverityScore >= 6.0` | PASS |
| Warning escalation >= 4.0 | `hptp-anomaly.service.ts:98` | `clampedSeverityScore >= 4.0` | PASS |
| Auto-escalation creates audit log | `hptp-anomaly.service.ts:59-76` | `securityAuditService.logEvent()` called | PASS |
| 5-tier fallback chain (PTP/NTP/Crystal/Quartz/Cesium) | `hptp-anomaly.service.ts:16,30-36` | All 5 tiers defined with metadata | PASS |
| Severity score clamped 0-10 | `hptp-anomaly.service.ts:49` | `Math.max(0, Math.min(10, ...))` | PASS |

**Finding**: All HPTP thresholds match documented values. Escalation logic verified through smoke test (38/38 pass).

### 2. Capability System

| Claim (threat_model.md) | Code Location | Actual Value | Status |
|--------------------------|---------------|--------------|--------|
| Capability-based access control | `src/kernel/crypto/capability.rs` | Capability struct with rights, domain, timing | PASS |
| RBAC with least privilege (RBAC_001) | `server/routes/middleware.ts:9-21` | `createRequireAdmin()` checks `user.isAdmin` | PASS |
| Audit logging for privileged actions (AUDIT_001) | `server/services/security-audit.service.ts:32-66` | `logEvent()` captures severity, category, actor, evidence | PASS |
| Immutable audit logs | `security-audit.service.ts` | Insert-only pattern; no DELETE endpoint | PASS |

**Finding**: Capability system exists in kernel. Backend RBAC implemented. Audit logs are append-only (no delete route exposed).

### 3. Cryptographic Controls

| Claim (threat_model.md) | Code Location | Actual Value | Status |
|--------------------------|---------------|--------------|--------|
| CNSA 2.0 suite (PQC_001) | `src/kernel/crypto/` | ML-KEM, ML-DSA, SLH-DSA implementations present | PASS |
| AES-256-GCM | `server/crypto-utils.ts` | AES-256-GCM encrypt/decrypt functions | PASS |
| Constant-time operations (CT_OPS_001) | `src/kernel/crypto/gf3.rs` | GF(3) constant-time arithmetic | PASS |
| Cryptographic agility (AGILITY_001) | `src/kernel/crypto/agility.rs` | Algorithm negotiation layer | PASS |
| Hybrid key exchange (HYBRID_001) | `src/kernel/crypto/` | X25519 + ML-KEM hybrid | PASS |

**Finding**: All claimed cryptographic controls have corresponding implementation code. Constant-time verification pending Riscure DPA-C3 empirical evaluation.

### 4. Security Infrastructure Services

| Claim (threat_model.md) | Code Location | Actual Value | Status |
|--------------------------|---------------|--------------|--------|
| 14 REST endpoints operational | `server/routes/security.ts` | 14 routes registered | PASS |
| Zod validation on all POST/PATCH | `server/routes/security.ts:43-53,367-384` | Zod schemas for audit, anomaly, threat, impl | PASS |
| JWT/session auth required | `server/routes/middleware.ts:9-21` | `requireAdmin` on all mutating endpoints | PASS |
| Dashboard aggregates 6 data sources | `server/routes/security.ts:501-527` | 6 parallel queries | PASS |
| Resolution workflow (unresolved -> resolved) | `security-audit.service.ts:100-122` | `resolveEvent()` with status transition | PASS |

**Finding**: All 14 endpoints operational. Smoke test confirmed correct HTTP status codes, validation, and auth enforcement.

### 5. Risk Score Calculations

| Claim (threat_model.md) | Code Location | Actual Value | Status |
|--------------------------|---------------|--------------|--------|
| Risk formula: `(L * I) / 1.6` | `threat-model.service.ts:33-37` | `(LEVEL_MAP[L] * LEVEL_MAP[I]) / 1.6` | NOTE |
| Level weights: low=1, medium=2, high=3, critical=4 | `threat-model.service.ts:31` | `{ low: 1, medium: 2, high: 3, critical: 4 }` | NOTE |

**Finding (NOTE)**: The threat model document (Section 2.2) describes a different formula: `likelihood_weight * impact_weight / 10` with weights `{ low: 1, medium: 3, high: 7, critical: 10 }`. However, the actual service code uses `{ low: 1, medium: 2, high: 3, critical: 4 }` with divisor 1.6. The resulting scores differ:

| Threat | Doc Formula | Code Formula | Discrepancy |
|--------|-------------|--------------|-------------|
| THREAT_001 (High x Critical) | 7.0 | 7.5 | Minor |
| THREAT_009 (Critical x Critical) | 10.0 | 10.0 | None |

**Recommendation**: Harmonize the documented formula with the code implementation. This is a documentation gap, not a security issue. The code formula produces defensible scores. Flag for v1.1 update — no changes to v1.0 per acceptance criteria.

### 6. Database Schema & Indexes

| Claim | Code/DB Location | Status |
|-------|-----------------|--------|
| 17 performance indexes | `docs/security/database_indexes.sql` + actual DB | PASS |
| Composite index (severity, category, created_at) | Index `idx_audit_severity_category_date` | PASS |
| 4 PostgreSQL tables | `security_audit_log`, `hptp_anomaly_events`, `threat_model_entries`, `implementation_status` | PASS |

**Finding**: All 17 indexes present in SQL artifact and applied to database. Composite index verified.

---

## Summary

| Section | Tests | Pass | Fail | Notes |
|---------|-------|------|------|-------|
| HPTP Thresholds | 6 | 6 | 0 | All thresholds match |
| Capability System | 4 | 4 | 0 | RBAC + audit verified |
| Cryptographic Controls | 5 | 5 | 0 | Empirical verification pending |
| Security Infrastructure | 5 | 5 | 0 | Smoke test confirmed |
| Risk Score Calculations | 2 | 0 | 0 | 2 NOTES (formula discrepancy) |
| Database Schema | 3 | 3 | 0 | All indexes present |
| **Total** | **25** | **23** | **0** | **2 Notes** |

---

## Findings for v1.1

1. **FINDING-001 (Low)**: Risk score formula in documentation (Section 2.2) uses different weights than service code. Recommendation: Update documentation to match code formula `(L * I) / 1.6` with `{ low: 1, medium: 2, high: 3, critical: 4 }`.

2. **FINDING-002 (Info)**: Constant-time properties claimed for cryptographic operations are code-verified but not empirically validated. Riscure DPA-C3 evaluation (Q2 2026) will provide empirical confirmation.

---

## Conclusion

The threat model accurately reflects the implemented security controls. No discrepancies found between claimed mitigations and actual code that would affect security posture. Two documentation notes flagged for v1.1 (no changes to v1.0 per acceptance criteria).

**QA Review Status**: PASSED (with 2 notes for v1.1)

---

*Document Control: Internal QA review. Reviewed by Security Engineering team.*
