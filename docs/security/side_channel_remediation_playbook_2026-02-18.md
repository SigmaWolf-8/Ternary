<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  Patent(s) Pending — All Rights Reserved
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# Side-Channel Remediation Playbook

**Version**: 1.0
**Date**: February 18, 2026
**Classification**: Internal / Audit-Ready
**Author**: Crypto Team Lead, Capomastro Holdings Ltd.
**Status**: Published
**Phase**: Phase 2 — Near-Term Validation Sprint (Feb 24 – Mar 15, 2026)
**Reference**: Phased Task List §2.6
**Risk Mitigation**: Addresses Risk 1 (Riscure findings)

---

## 1. Executive Summary

This playbook pre-positions remediation candidates for side-channel vulnerabilities that may be identified during the Riscure DPA-C3 evaluation (interim findings expected late March 2026). The goal is to ensure that remediation can begin within 48 hours of receiving Riscure interim findings, minimizing the window between vulnerability identification and fix deployment.

The playbook identifies high-risk cryptographic operations, documents the pre-remediation baseline, presents three remediation candidates with implementation timelines and performance impact estimates, and defines a rollback plan if full rework is needed.

### Playbook Summary

| Metric | Value |
|--------|-------|
| High-Risk Operations Identified | 4 |
| Remediation Candidates Prepared | 3 |
| Total Remediation Time (if all applied) | 6 weeks |
| Maximum Performance Overhead | ~20% (worst case, ML-KEM NTT) |
| Response Time Target | 48 hours from Riscure interim findings |
| Rollback Plan | Revert to unmasked constant-time baseline |

---

## 2. High-Risk Crypto Operations Identified

The following cryptographic operations are identified as high-risk for side-channel leakage based on their computational characteristics and exposure to power/EM analysis:

| # | Operation | Component | Risk Factor | Current Protection |
|---|-----------|-----------|-------------|-------------------|
| 1 | **AES-256-GCM polynomial multiplication** | `crypto/aes_gcm.rs` | GCM polynomial multiplication uses carry-less multiply; S-box lookups are primary DPA target | Bitsliced S-box (T-table eliminated), constant-time carry-less multiply |
| 2 | **ML-KEM decapsulation/decode** | `crypto/ml_kem.rs` | NTT butterfly operations and Fujisaki-Okamoto comparison are potential leakage points | Constant-time FO transform, Montgomery multiplication |
| 3 | **HPTP jitter source sampling** | `hptp/core.rs` | Jitter sampling timing may reveal entropy quality; sampling pattern could leak timing state | Constant-time sampling loop |
| 4 | **Phase encryption split/recombine** | `crypto/phase.rs` | Phase split operation processes secret phase values; recombine involves timing-sensitive comparison | Phase split verified constant-time; recombine under verification (30% complete) |

### Risk Prioritization

| Operation | Likelihood of Leakage | Impact if Exploited | Priority |
|-----------|----------------------|--------------------|---------| 
| AES-256-GCM S-box | Medium | Critical (key recovery) | P1 |
| ML-KEM NTT | Medium | Critical (key recovery) | P1 |
| Phase encryption recombine | Low-Medium | High (plaintext recovery) | P2 |
| HPTP jitter sampling | Low | Medium (timing state leakage) | P3 |

---

## 3. Pre-Remediation Baseline

### 3.1 Current Code Status

| Component | Constant-Time Status | Verification Method | Evidence |
|-----------|---------------------|--------------------|---------| 
| AES-256-GCM | Verified constant-time | CBMC + dudect | Bitsliced S-box eliminates cache-dependent lookups; 10M iterations, t < 4.5 |
| ML-KEM decapsulation | Verified constant-time | CBMC | FO transform with constant-time comparison; Montgomery multiplication |
| GF(3) arithmetic | Formally proven constant-time | Lean 4 | All operations execute in identical cycle count regardless of input |
| Phase encryption split | Verified constant-time | CBMC | Phase split confirmed; recombine under verification |
| HPTP jitter sampling | Implemented constant-time | Code review | Sampling loop uses fixed iteration count |

### 3.2 Testing Status

| Test Type | Status | Details |
|-----------|--------|---------|
| TVLA (Test Vector Leakage Assessment) | Planned | Will be conducted as part of Riscure DPA-C3 evaluation |
| dudect timing analysis | Completed (AES, ML-KEM) | 10M iterations, all t-values < 4.5 |
| CBMC constant-time verification | Completed (AES, ML-KEM, GF(3)) | No secret-dependent control flow or memory access detected |
| Riscure Inspector evaluation | In progress | Baseline characterization phase; equipment calibration underway |
| Assembly inspection | Completed (AES) | No cmov on secret, no variable-time division, no secret-dependent indexing |

### 3.3 Known Limitations

1. **TVLA has not yet been performed** — pre-remediation leakage assessment relies on static analysis (CBMC) and dynamic timing analysis (dudect), not power/EM trace analysis.
2. **Phase encryption recombine** is only 30% through formal verification — timing properties not yet fully proven.
3. **Higher-order leakage** has not been assessed — current protections target first-order attacks only.
4. **Assembly-level inspection** completed only for AES-256-GCM — ML-KEM and phase encryption assembly not yet reviewed.

---

## 4. Remediation Candidates

### 4.1 Candidate 1: Boolean Masking for AES S-box

| Property | Detail |
|----------|--------|
| **Target** | AES-256-GCM S-box computation |
| **Technique** | First-order boolean masking |
| **Implementation Time** | 2 weeks |
| **Performance Impact** | ~15% overhead |
| **Confidence** | High (well-studied technique) |

**Masking Scheme Design**:

```
First-Order Boolean Masking for AES S-box:

1. Input sharing: x = x' ⊕ m  (where m is a random mask)
2. Masked S-box: S(x' ⊕ m) computed without revealing x
3. Output resharing: y = S(x) ⊕ m' (new random mask)

Implementation approach:
- Rivain-Prouff (2010) scheme for masked S-box evaluation
- GF(2^8) tower field decomposition
- 18 non-linear multiplications per S-box (vs. 1 unmasked)
- Fresh randomness: 4 bytes per S-box evaluation
- Total randomness per AES round: 64 bytes (16 S-boxes × 4 bytes)
```

**Implementation Plan**:

| Day | Task |
|-----|------|
| 1-3 | Implement Rivain-Prouff masked S-box in `crypto/aes_masked.rs` |
| 4-5 | Integrate masked S-box into AES-256-GCM encryption path |
| 6-8 | Verify constant-time properties with CBMC (masked version) |
| 9-10 | Run dudect on masked implementation (10M iterations) |
| 11-14 | Integration testing, performance benchmarking, code review |

**Performance Impact Analysis**:

| Metric | Unmasked | Masked | Overhead |
|--------|----------|--------|----------|
| AES-256-GCM encrypt (1KB) | 2.1 μs | ~2.4 μs | ~15% |
| AES-256-GCM encrypt (64KB) | 89 μs | ~102 μs | ~15% |
| Randomness consumption | 0 bytes/block | 640 bytes/block | N/A |

---

### 4.2 Candidate 2: Arithmetic Masking for ML-KEM NTT Operations

| Property | Detail |
|----------|--------|
| **Target** | ML-KEM NTT butterfly operations |
| **Technique** | Arithmetic masking with arithmetic-to-boolean conversion |
| **Implementation Time** | 3 weeks |
| **Performance Impact** | ~20% overhead |
| **Confidence** | Medium-High (established technique, more complex implementation) |

**Masking Scheme Design**:

```
Arithmetic Masking for ML-KEM NTT:

1. Arithmetic sharing: x = x' + m mod q (where q is the NTT modulus)
2. NTT butterfly: compute masked butterfly operations
3. Arithmetic-to-Boolean (A2B) conversion for comparison steps:
   - Coron-Tchulkine (2003) method for efficient A2B
   - Required for FO transform comparison (constant-time)
4. Boolean-to-Arithmetic (B2A) conversion for re-entry to NTT:
   - Goubin (2001) method

Implementation approach:
- Masked NTT butterfly: (a + b) mod q, (a - b) * w mod q
  with shares: (a', b', m_a, m_b) → masked output shares
- Montgomery multiplication adapted for arithmetic shares
- Fresh randomness: 2 field elements per butterfly
- Total randomness per NTT: ~2KB (256 butterflies × 8 bytes)
```

**Implementation Plan**:

| Day | Task |
|-----|------|
| 1-4 | Implement arithmetic-masked NTT butterfly in `crypto/ml_kem_masked.rs` |
| 5-7 | Implement A2B and B2A conversion routines |
| 8-10 | Integrate masked NTT into ML-KEM decapsulation path |
| 11-13 | Verify constant-time properties with CBMC (masked version) |
| 14-16 | Run dudect on masked implementation |
| 17-21 | Integration testing, NIST test vector validation, performance benchmarking |

**Performance Impact Analysis**:

| Metric | Unmasked | Masked | Overhead |
|--------|----------|--------|----------|
| ML-KEM-1024 encaps | 145 μs | ~174 μs | ~20% |
| ML-KEM-1024 decaps | 168 μs | ~202 μs | ~20% |
| Randomness consumption | 32 bytes | ~2.1 KB | N/A |

---

### 4.3 Candidate 3: Redundant Computation for HPTP Jitter Sampling

| Property | Detail |
|----------|--------|
| **Target** | HPTP jitter source sampling |
| **Technique** | Redundant computation with randomized timing padding |
| **Implementation Time** | 1 week |
| **Performance Impact** | ~5% overhead |
| **Confidence** | High (simple technique, low implementation risk) |

**Masking Scheme Design**:

```
Randomized Timing Padding for HPTP Jitter Sampling:

1. Sample jitter source N times (fixed N, independent of jitter quality)
2. Add random timing padding between samples:
   - Padding duration: uniform random in [0, T_pad] microseconds
   - T_pad calibrated to dwarf jitter measurement variance
3. Compute quality metric using all N samples (constant-time reduction)
4. Redundant computation: perform sampling twice, compare results
   - Mismatch triggers anomaly event (fault injection detection)

Implementation approach:
- Fixed-count sampling loop (N = 64 samples per measurement)
- Random delay insertion using RDRAND/RDSEED (or HPTP's own entropy)
- Constant-time quality reduction (no early-exit on good quality)
- Redundant path comparison with constant-time equality check
```

**Implementation Plan**:

| Day | Task |
|-----|------|
| 1-2 | Implement randomized timing padding in `hptp/jitter_sampling.rs` |
| 3-4 | Add redundant computation path with constant-time comparison |
| 5 | Verify timing properties with CBMC; measure performance impact |
| 6-7 | Integration testing with HPTP fallback chain; regression testing |

**Performance Impact Analysis**:

| Metric | Original | Remediated | Overhead |
|--------|----------|-----------|----------|
| Jitter sampling (per measurement) | 12 μs | ~12.6 μs | ~5% |
| Sampling frequency (max) | 83 KHz | ~79 KHz | ~5% |
| Entropy quality | Baseline | Equivalent | None |

---

## 5. Masking Scheme Summary

| Candidate | Technique | Order | Randomness Per Op | Performance Impact |
|-----------|-----------|-------|-------------------|-------------------|
| AES S-box | First-order boolean masking | 1st order | 640 bytes/block | ~15% |
| ML-KEM NTT | Arithmetic masking + A2B/B2A conversion | 1st order | ~2.1 KB/operation | ~20% |
| HPTP jitter | Randomized timing padding + redundant computation | N/A | Minimal (timing only) | ~5% |

### Randomness Requirements

All masking schemes require cryptographic-quality randomness. Sources:

| Source | Rate | Suitability |
|--------|------|-------------|
| RDRAND (x86) | ~800 MB/s | Primary source; hardware RNG |
| RDSEED (x86) | ~64 MB/s | Seeding; true entropy |
| HPTP jitter pool | ~1 MB/s | Supplementary; validated entropy |
| ChaCha20 DRBG | Unlimited (after seeding) | Expansion for high-volume masking |

**Assessment**: Randomness consumption is well within hardware RNG capacity. AES masking at full throughput requires ~60 MB/s of randomness (at 100K blocks/sec), comfortably supplied by RDRAND.

---

## 6. Rollback Plan

If Riscure findings indicate that the masking approach is insufficient (e.g., higher-order leakage detected, or masking introduces new vulnerabilities), the following rollback procedure applies:

### 6.1 Rollback Procedure

| Step | Action | Owner | Time |
|------|--------|-------|------|
| 1 | Revert masked code changes to unmasked constant-time baseline | Crypto Team | 1 hour |
| 2 | Re-run CBMC and dudect on reverted code to confirm baseline is intact | Crypto Team | 4 hours |
| 3 | Notify Riscure of rollback; request updated evaluation scope | Security Lead | Same day |
| 4 | Convene remediation review meeting (Crypto Team + Security Lead + Salvi) | Salvi | Within 24 hours |
| 5 | Re-evaluate remediation strategy based on Riscure feedback | Crypto Team | 1 week |
| 6 | Implement revised remediation (potentially higher-order masking or alternative technique) | Crypto Team | TBD |

### 6.2 Rollback Triggers

| Trigger | Response |
|---------|----------|
| Masking introduces new timing leakage | Immediate rollback + investigation |
| Higher-order DPA succeeds against masked implementation | Rollback + evaluate 2nd-order masking |
| Performance overhead exceeds 30% (any component) | Rollback + optimize or use alternative technique |
| Riscure finds fundamental design flaw in masking approach | Rollback + full re-evaluation with Riscure consulting |

### 6.3 Baseline Preservation

The unmasked constant-time baseline is preserved via:
- Git tag `v1.0-pre-masking-baseline` on all affected crypto modules
- Separate CI/CD pipeline running unmasked baseline tests
- Performance benchmark history maintained in `docs/security/benchmarks.md`

---

## 7. Timeline

### 7.1 Pre-Riscure Preparation (Now – Late March)

| Date | Milestone | Status |
|------|-----------|--------|
| Feb 18, 2026 | Playbook published (this document) | Complete |
| Feb 24 – Mar 10 | Remediation candidate code drafted (not deployed) | In Progress |
| Mar 1 – Mar 15 | Riscure DPA-C3 baseline characterization begins | Riscure |
| Mar 15 | Galois kickoff (parallel workstream) | Scheduled |

### 7.2 Post-Riscure Response (Late March – April)

| Trigger | Response Window | Action |
|---------|----------------|--------|
| Riscure interim findings received | T+0 | Triage findings, classify severity |
| Remediation candidates selected | T+24h | Select applicable candidate(s) from this playbook |
| Implementation begins | T+48h | Begin coding selected remediation(s) |
| Candidate 3 (HPTP jitter) | T+48h to T+1 week | Implementation + testing complete |
| Candidate 1 (AES S-box) | T+48h to T+2 weeks | Implementation + testing complete |
| Candidate 2 (ML-KEM NTT) | T+48h to T+3 weeks | Implementation + testing complete |
| Post-remediation validation | T+3 to T+4 weeks | Re-run CBMC, dudect; prepare for Riscure re-test |

### 7.3 Key Commitment

**Remediation candidates ready for implementation within 48 hours of Riscure interim findings (expected late March 2026).**

---

## 8. Risk Mitigation Matrix

| Risk | Mitigation | Playbook Section |
|------|-----------|-----------------|
| **Risk 1**: Riscure finds side-channel leakage in crypto ops | Three pre-positioned remediation candidates; 48h response time | §4, §7 |
| **Risk 1a**: Leakage in AES S-box | Candidate 1: Boolean masking (2 weeks) | §4.1 |
| **Risk 1b**: Leakage in ML-KEM NTT | Candidate 2: Arithmetic masking (3 weeks) | §4.2 |
| **Risk 1c**: Leakage in HPTP jitter | Candidate 3: Redundant computation (1 week) | §4.3 |
| **Risk 1d**: Masking approach fails | Rollback to unmasked baseline; re-evaluate with Riscure | §6 |
| **Risk 1e**: Performance degradation unacceptable | Rollback trigger at >30% overhead; optimize or use alternative | §6.2 |

---

## Sign-Off

| Role | Name | Date | Status |
|------|------|------|--------|
| Crypto Team Lead | | Feb 18, 2026 | _Pending_ |
| Security Lead | | Feb 18, 2026 | _Pending_ |
| Engagement Lead (Salvi) | | Feb 18, 2026 | _Pending_ |

---

*Document Control: Phase 2 side-channel remediation playbook. Pre-positioned for Riscure DPA-C3 interim findings (expected late March 2026). Remediation candidates ready for implementation within 48 hours of findings.*
