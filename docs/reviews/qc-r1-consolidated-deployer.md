# QC-R1 Consolidated Review — PlenumNET Array3 Deployer Script

**Document Under Review:** `services/inter-cube/deploy-yoda.ps1` (v0.5.0, 1652 lines)
**Review Protocol Version:** 1.1.2
**Date:** 2026-03-30
**Review Round:** QC-R1 (Round 1: Technical Verification)

---

## Consolidation Summary

**Overall Verdict: FAIL**

- **Agent 1 (Security Engineer):** FAIL — 2 CRITICAL, 6 IMPORTANT, 3 MINOR
- **Agent 2 (DevOps Automator):** PASS WITH CONDITIONS — 0 CRITICAL, 6 IMPORTANT, 5 MINOR
- **Agent 3 (PlenumNET Integration Specialist):** FAIL — 1 CRITICAL, 5 IMPORTANT, 3 MINOR

The spec does not proceed to QC-R2 until the CRITICAL findings (R1-A1-1, R1-A1-2, R1-A3-1) are resolved and the failing agents re-review.

---

## Consolidated Findings Table

| # | Finding ID | Agent | Section | Severity | Finding (Summary) | Crypto Status |
|---|------------|-------|---------|----------|--------------------|---------------|
| 1 | R1-A1-1 | Security Engineer | Lines 9-10 (SYNOPSIS) | CRITICAL | `irm \| iex` invocation executes remote code with Administrator privileges without integrity verification, signature check, or certificate pinning | N/A |
| 2 | R1-A1-2 / R1-A3-1 | Security Engineer + PlenumNET Integration | Lines 503-526 (STEP 4) | CRITICAL | SHA-256 used as primary/fallback binary integrity hash — banned primitive; TIS-27 treated as optional | INCORRECT |
| 3 | R1-A3-2 | PlenumNET Integration | Lines 1386-1396 (CRS registration) | IMPORTANT | TL-DSA signing context binds IP:port endpoint (non-Rep-C identifier) — violates INVARIANT 9 | INCORRECT |
| 4 | R1-A3-3 | PlenumNET Integration | Lines 1440-1445 (deployment signature) | IMPORTANT | `"DEPLOYMENT"` context string not in canonical registry — unverified domain separation | UNVERIFIED |
| 5 | R1-A3-4 | PlenumNET Integration | Lines 1386-1396 (CRS registration) | IMPORTANT | `"CRS-REGISTER"` context string not in canonical registry — unverified domain separation | UNVERIFIED |
| 6 | R1-A1-3 | Security Engineer | Lines 154-174 (Get-TlDsaSignature) | IMPORTANT | Signing payload and identity paths passed via environment variables; cleanup not guaranteed; no memory zeroing | N/A |
| 7 | R1-A1-4 | Security Engineer | Lines 214-233 (admin elevation) | IMPORTANT | Auto-elevation re-launches script without integrity verification; UAC shows "Windows PowerShell" not publisher name | N/A |
| 8 | R1-A1-5 | Security Engineer | Lines 295-316, 384-394 (tool install) | IMPORTANT | Rustup and LLVM installers downloaded without hash verification | N/A |
| 9 | R1-A1-6 | Security Engineer | Lines 790-898 (service wrappers) | IMPORTANT | Wrapper .bat files contain identity paths in plaintext environment variables visible to process inspection | N/A |
| 10 | R1-A1-7 / R1-A3-9 | Security Engineer + PlenumNET Integration | Lines 1447-1478 (deployment payload) | IMPORTANT | Remote payload includes hostname, IP, filesystem paths — non-Rep-C identifiers and environment leakage | N/A |
| 11 | R1-A1-8 | Security Engineer | Lines 734-758 (keygen) | IMPORTANT | Key generation does not verify entropy source, key format/size, or use atomic writes | UNVERIFIED |
| 12 | R1-A1-10 | Security Engineer | Lines 943-1245 (watchdog LLM) | IMPORTANT | Watchdog executes LLM restart commands as SYSTEM from config file; custom argument parser could be bypassed | N/A |
| 13 | R1-A2-1 | DevOps Automator | Lines 68-69 (version constants) | IMPORTANT | Rust and LLVM tool versions not pinned — build reproducibility broken | N/A |
| 14 | R1-A2-2 | DevOps Automator | Lines 416-440 (git clone) | IMPORTANT | Git tag signature not verified; `--force` flag suppresses tag update warnings | N/A |
| 15 | R1-A2-3 | DevOps Automator | Lines 466-493 (cargo build) | IMPORTANT | No Cargo.lock verification; false-positive error detection regex | N/A |
| 16 | R1-A2-5 | DevOps Automator | Lines 872-898 (service registration) | IMPORTANT | No atomic rollback on partial failure — inconsistent cluster state possible | N/A |
| 17 | R1-A2-6 | DevOps Automator | Lines 446-457 (daemon kill) | IMPORTANT | Running daemons hard-killed without graceful shutdown attempt | N/A |
| 18 | R1-A3-6 | PlenumNET Integration | Lines 734-758 (keygen) | IMPORTANT | Keygen invoked twice per node — idempotency assumed but not verified; risk of key overwrite | UNVERIFIED |
| 19 | R1-A3-7 | PlenumNET Integration | Lines 698-719 (identity migration) | IMPORTANT | Identity migration uses SHA-256 for integrity verification — banned primitive | INCORRECT |
| 20 | R1-A1-9 | Security Engineer | Lines 109-146 (Grant-LogonAsService) | MINOR | secedit policy modification via string manipulation; failure silently ignored by caller | N/A |
| 21 | R1-A1-11 | Security Engineer | Lines 1386-1414 (local CRS) | MINOR | Localhost CRS communication uses plain HTTP — local MITM possible | N/A |
| 22 | R1-A2-4 | DevOps Automator | Lines 503-526 (integrity) | MINOR | Binary integrity mechanism non-deterministic (TIS-27 vs SHA-256) across deployments (cross-ref: R1-A1-2) | N/A |
| 23 | R1-A2-7 | DevOps Automator | Lines 1324-1366 (watchdog task) | MINOR | Primary and fallback watchdog registration paths produce different functionality | N/A |
| 24 | R1-A2-8 | DevOps Automator | Lines 546-558 (version probe) | MINOR | Version probe generates throwaway key material in unprotected temp directory | N/A |
| 25 | R1-A2-9 | DevOps Automator | Lines 560-576 (remote version) | MINOR | Remote version check silently swallows all errors — no diagnostic output | N/A |
| 26 | R1-A2-10 | DevOps Automator | Lines 236-241 (upgrade detect) | MINOR | Binary upgrade detection conflates multiple states | N/A |
| 27 | R1-A2-11 | DevOps Automator | Lines 648-662 (secedit export) | MINOR | Security policy export to unprotected TEMP directory | N/A |
| 28 | R1-A3-5 | PlenumNET Integration | Lines 22-45 (topology) | MINOR | Gateway offset indexing comment should clarify Rep C vs GF(3) coordinate system | N/A |
| 29 | R1-A3-8 | PlenumNET Integration | Lines 760-774 (key lifecycle) | MINOR | Key lifecycle boundary between deployer and runtime not documented | UNVERIFIED |
| 30 | R1-A3-10 | PlenumNET Integration | Lines 807-847 (RELAY_URL) | MINOR | RELAY_URL uses HTTPS bootstrap URL; no mechanism to transition to TDNS-resolved address | N/A |

---

## Deduplicated Cross-Agent Findings

The following findings were flagged by multiple agents and are consolidated:

1. **SHA-256 binary integrity** (R1-A1-2 + R1-A3-1 + R1-A2-4): All three agents identified the SHA-256 usage in binary integrity checking. Consolidated as CRITICAL per the highest severity.

2. **Remote deployment metadata** (R1-A1-7 + R1-A3-9): Both Agent 1 and Agent 3 flagged the transmission of non-Rep-C identifiers (hostname, IP, filesystem paths) to the remote endpoint. Consolidated as IMPORTANT.

---

## Cryptographic Status Summary

| Cryptographic Claim | Status | Finding IDs |
|---------------------|--------|-------------|
| SHA-256 binary integrity hash | INCORRECT | R1-A1-2, R1-A3-1 |
| SHA-256 migration integrity check | INCORRECT | R1-A3-7 |
| TL-DSA signing context (CRS registration) | INCORRECT | R1-A3-2 |
| PT26-DSA keygen idempotency | UNVERIFIED | R1-A3-6 |
| "CRS-REGISTER" context string | UNVERIFIED | R1-A3-4 |
| "DEPLOYMENT" context string | UNVERIFIED | R1-A3-3 |
| Key rotation lifecycle boundary | UNVERIFIED | R1-A3-8 |
| Node IDs Rep C {1,2,3} | VERIFIED | — |
| 27-slot cube topology (3^3) | VERIFIED | — |
| Gateway offset 13 = T₇ | VERIFIED | — |

---

## CRITICAL Findings Requiring Resolution Before QC-R2

### 1. Remote code execution via `irm | iex` (R1-A1-1)
The deployer is designed to be downloaded and executed in a single pipeline command with Administrator privileges. No integrity verification, code signing, or certificate pinning protects this invocation path. This must be addressed with Authenticode signing and a two-step download-verify-execute workflow.

### 2. SHA-256 as integrity primitive (R1-A1-2 / R1-A3-1 / R1-A3-7)
Three separate uses of SHA-256 (binary hash, pre-start re-verification, migration integrity) must be replaced with TIS-27. The SHA-256 fallback path must be removed entirely. If TIS-27 is unavailable, the deployer must fail rather than fall back to a banned primitive.

### 3. TL-DSA context binds non-Rep-C identifier (R1-A3-2)
The CRS registration signing context includes an IP:port endpoint string, violating INVARIANT 9. The context must bind only Rep C addresses, public keys, and timestamps.

---

## QC-R2 Handoff

This consolidated output is **NOT** ready for QC-R2 handoff. The spec must be revised to address the three CRITICAL findings listed above. After revision, Agent 1 (Security Engineer) and Agent 3 (PlenumNET Integration Specialist) must re-review the revised script using the same template version (1.1.2).

---

## Agent 1: Security Engineer — Full Review

*(See `docs/reviews/qc-r1-agent1-security-engineer.md` for the complete review)*

## Agent 2: DevOps Automator — Full Review

*(See `docs/reviews/qc-r1-agent2-devops-automator.md` for the complete review)*

## Agent 3: PlenumNET Integration Specialist — Full Review

*(See `docs/reviews/qc-r1-agent3-plenumnet-integration.md` for the complete review)*

---

*Capomastro Holdings Ltd. — Applied Physics Division*
*Sherwood Park, Alberta, Canada*
