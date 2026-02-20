# XPlenum CNSA 2.0 Compliance Documentation

**Date:** 2026-02-18  
**Version:** 1.0  
**Classification:** PROPRIETARY AND CONFIDENTIAL  
**Copyright:** (c) 2025-2026 Capomastro Holdings Ltd. (Canada), Applied Physics Division  
**Reference:** NSA CNSA 2.0 (September 2022), NSA CNSSP-15

---

## 1. Overview

The Commercial National Security Algorithm Suite 2.0 (CNSA 2.0) defines the NSA's requirements for quantum-resistant cryptographic algorithms in National Security Systems (NSS). This document maps XPlenum capabilities against CNSA 2.0 mandates and documents the transition strategy.

---

## 2. CNSA 2.0 Algorithm Requirements

### 2.1 Mandated Algorithms

| Function | CNSA 2.0 Algorithm | Standard | XPlenum Support |
|----------|---------------------|----------|-----------------|
| Symmetric Encryption | AES-256 | FIPS 197 | **Direct** — `xplenum_aes256_core.v` |
| Hashing | SHA-384, SHA-512 | FIPS 180-4 | Kernel software + HW acceleration path |
| Digital Signature | ML-DSA-87 (Dilithium) | FIPS 204 | Kernel software (TL-DSA in `src/kernel/`) |
| Key Encapsulation | ML-KEM-1024 (Kyber) | FIPS 203 | Kernel software (TL-KEM in `src/kernel/`) |
| Key Agreement | ML-KEM-1024 | FIPS 203 | Kernel software |
| DRBG | SP 800-90A CTR_DRBG (AES-256) | SP 800-90A | **Direct** — `xplenum_ctr_drbg.v` |

### 2.2 Transition Timeline (per CNSA 2.0 Advisory)

| Capability | Legacy Algorithm | CNSA 2.0 Replacement | Deadline | XPlenum Status |
|------------|------------------|----------------------|----------|----------------|
| Symmetric | AES-256 | AES-256 (unchanged) | N/A | Compliant |
| DRBG | CTR_DRBG (AES-256) | CTR_DRBG (AES-256) | N/A | Compliant |
| Firmware Signing | ECDSA P-384 | ML-DSA-87 | 2025 | Supported (kernel SW) |
| Software/Firmware | RSA-3072+ | ML-DSA-87 | 2025 | Supported (kernel SW) |
| Web Services (TLS) | ECDH P-384 | ML-KEM-1024 | 2025 | Supported (kernel SW) |
| VPN/IPsec | ECDH P-384 | ML-KEM-1024 | 2026 | Supported (kernel SW) |
| Networking | RSA-3072+ | ML-KEM-1024 + ML-DSA-87 | 2030 | Supported (kernel SW) |

---

## 3. XPlenum CNSA 2.0 Hardware Acceleration

### 3.1 AES-256 Core

The XPlenum AES-256 core (`xplenum_aes256_core.v`) provides hardware-accelerated symmetric encryption for the CTR_DRBG mechanism. This is the foundational CNSA 2.0 compliant component.

**Specifications:**
- Algorithm: AES-256 (FIPS 197)
- Key size: 256 bits
- Pipeline: 14 rounds (14-cycle latency, 1 block/cycle throughput after pipeline fill)
- Throughput: 128 bits × 100 MHz = 12.8 Gbps (raw block cipher)
- CTR_DRBG throughput: ~1.28 Gbps (after DRBG overhead)

### 3.2 Post-Quantum Algorithm Acceleration Path

XPlenum provides infrastructure that accelerates PQC algorithm implementations:

| PQC Operation | XPlenum Instruction | Speedup |
|---------------|---------------------|---------|
| NTT (Number Theoretic Transform) | TROTL/TROTR (barrel rotate) | 3x |
| Polynomial arithmetic | TTRIT/TDETRIT (mod-3 operations) | 24x |
| Constant-time comparison | TSIGCMP | 4x |
| Side-channel masking for PQC | TMASK/TMASKR | 30x (DRBG) |
| Sampling (CBD/rejection) | TMASKR (uniform random generation) | 30x |

### 3.3 Security Property Alignment

| CNSA 2.0 Requirement | XPlenum Mechanism |
|-----------------------|-------------------|
| 256-bit security strength | AES-256, 256-bit DRBG seed |
| Quantum-resistant key establishment | ML-KEM via kernel + HW masking |
| Quantum-resistant authentication | ML-DSA via kernel + HW masking |
| Side-channel resistance | Hardware masking subsystem |
| Key management | Hardware-isolated domain/capability system |
| Entropy generation | SP 800-90A CTR_DRBG with external TRNG |

---

## 4. PlenumNET Quantum Resistance Stack

The complete PlenumNET quantum resistance architecture layers CNSA 2.0 algorithms with XPlenum hardware acceleration:

```
┌─────────────────────────────────────────────────────┐
│                    Application Layer                 │
│  (TLS 1.3 with ML-KEM + ML-DSA hybrid key exchange) │
├─────────────────────────────────────────────────────┤
│                   Kernel Layer                       │
│  TL-KEM (ML-KEM-1024) | TL-DSA (ML-DSA-87)         │
│  SHA-384/512          | X.509 PQC Certificates       │
├─────────────────────────────────────────────────────┤
│              XPlenum Hardware Acceleration            │
│  AES-256 Core | CTR_DRBG | Masking | Domain Isolation│
│  Capability System | Ternary Crypto Primitives       │
├─────────────────────────────────────────────────────┤
│              CVA6 RISC-V Core (RV64GC)               │
│  M/S/U privilege modes | MMU | Standard ISA          │
└─────────────────────────────────────────────────────┘
```

---

## 5. Wassenaar Arrangement Classification

### 5.1 Export Control Category

| Parameter | Classification |
|-----------|---------------|
| Category | 5 — Part 2: Information Security |
| Subcategory | 5A002.a.1 — Cryptographic equipment using symmetric algorithms with key > 56 bits |
| Key length | 256 bits (AES-256) |
| Control reason | Encryption for confidentiality |
| License | Required for Category D/E destinations |

### 5.2 Technical Parameters for Export Declaration

| Parameter | Value |
|-----------|-------|
| Algorithm | AES-256 (symmetric) |
| Key length | 256 bits |
| Block size | 128 bits |
| Mode | CTR (for DRBG) |
| Purpose | Random number generation for security masking |
| Key management | Hardware-internal (not externally extractable) |
| Crypto activation | Via CSR XPSTATUS register (privilege-mode gated) |

---

## 6. Compliance Evidence Matrix

| CNSA 2.0 Requirement | Evidence | Location |
|----------------------|----------|----------|
| AES-256 implementation | RTL source, formal verification (115+ properties) | `XPlenum/rtl/xplenum_aes256_core.v`, `XPlenum/rtl/formal/` |
| SP 800-90A CTR_DRBG | RTL source, DRBG testbench, health tests | `XPlenum/rtl/xplenum_ctr_drbg.v`, `XPlenum/tb/xplenum_drbg_tb.v` |
| SP 800-90B health testing | Repetition Count + Adaptive Proportion tests | `XPlenum/rtl/xplenum_ctr_drbg.v` (integrated) |
| Side-channel resistance | Boolean masking subsystem, constant-time ALU | `XPlenum/rtl/xplenum_mask_unit.v` |
| Key isolation | Domain isolation, capability-based access | `XPlenum/rtl/xplenum_domain_unit.v`, `XPlenum/rtl/xplenum_cap_unit.v` |
| Validation testing | 50 ISS tests, 1M fuzz iterations, 115+ formal properties | `XPlenum/sim/`, `XPlenum/rtl/formal/`, this document |

---

## 7. Gap Analysis: CNSA 2.0 Full Compliance

| Gap | Priority | Remediation | Timeline |
|-----|----------|-------------|----------|
| ML-KEM-1024 hardware acceleration | High | Add dedicated NTT coprocessor | Q3 2026 |
| ML-DSA-87 hardware acceleration | High | Add polynomial multiplier | Q3 2026 |
| SHA-384/512 hardware core | Medium | Add SHA-2 engine (optional) | Q4 2026 |
| CAVP validation for AES-256 | High | Run NIST ACVP tests | Q2 2026 |
| CAVP validation for CTR_DRBG | High | Run NIST DRBG ACVP tests | Q2 2026 |

---

## 8. Conclusion

XPlenum provides the foundational hardware security infrastructure for CNSA 2.0 compliance. The AES-256 core and SP 800-90A CTR_DRBG are directly compliant with CNSA 2.0 symmetric and DRBG requirements. Post-quantum algorithms (ML-KEM, ML-DSA) are supported at the kernel software layer with hardware-accelerated side-channel countermeasures. Future hardware extensions for NTT and polynomial arithmetic will provide full PQC hardware acceleration.

---

*Prepared by: XPlenum Engineering, Applied Physics Division*  
*Capomastro Holdings Ltd. (Canada)*  
*Patent(s) Pending*
