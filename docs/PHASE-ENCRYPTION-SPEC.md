<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  All Rights Reserved – Patent(s) Pending
  Open for non-commercial review under the terms of the project license.
-->

# Phase Encryption Specification (Draft v0.3)

**Salvi Framework — Adaptive Dual-Phase Quantum-Resistant Cipher**

| Field       | Value |
|-------------|-------|
| Project     | Ternary / PlenumNET (SigmaWolf-8/Ternary) |
| Authors     | Salvi Framework Team (with audit input from Grok/xAI) |
| Date        | February 15, 2026 |
| Status      | Working Draft — For public review, ePrint submission, and third-party cryptanalysis |
| License     | All Rights Reserved (Capomastro Holdings Ltd 2026) — Open for non-commercial review |

---

## 1. Abstract

Phase Encryption is a novel post-quantum symmetric primitive that operates in the complex plane using bijective ternary (GF(3)) arithmetic. It employs an **adaptive dual-phase** construction:

- **Primary Phase**: Encodes plaintext into a complex-valued waveform anchored to high-precision timing (HPTP).
- **Guardian Phase**: Independently computes a tamper-evident checksum weighted by the irrational Tribonacci constant τ ≈ 1.839, providing a second layer of integrity that survives even if the primary phase is compromised.

The scheme is integrated with the Ternary Virtual Machine (TVM), qutrit stabilizer codes (\[\[3,1,2\]\]\_3), Noether invariants, and femtosecond-scale timing. It is designed for hybrid use with CNSA 2.0 algorithms (ML-KEM/ML-DSA outer wrapper) and targets applications requiring long-term confidentiality and physical-layer tamper resistance.

**Security Claim (Provisional):** IND-CCA2 in the random-oracle model under standard assumptions, with additional resistance to fault-injection and timing attacks via constant-time implementations.

---

## 2. Notation and Preliminaries

### 2.1 Ternary Core (GF(3))

- **Trit representations** (bijective):
  - A = {−1, 0, +1} (computational)
  - B = {0, 1, 2} (network)
  - C = {1, 2, 3} (human)
- **Bijections** (constant-time, compile-time evaluable):
  - A → B: f(a) = a + 1
  - A → C: f(a) = a + 2
  - B → C: f(b) = b + 1
- All internal arithmetic uses fixed-point Complex64 (Q31.32 per component) to eliminate floating-point non-determinism.

### 2.2 Tribonacci Constants

- **τ** = (1 + ∛(19 + 3√33) + ∛(19 − 3√33)) / 3 ≈ 1.839286755214161
- **Tribonacci sequence** T\_n defined by T\_0 = 0, T\_1 = 0, T\_2 = 1, T\_n = T\_{n−1} + T\_{n−2} + T\_{n−3}
- **Weighted checksum** uses τ^k for diffusion.

### 2.3 Supporting Primitives

- **QCorrect** (0xA5): \[\[3,1,2\]\]\_3 stabilizer code on ternary register groups (constant-time integer lookup tables).
- **Noether Invariants**:
  - Ternary Gauge Symmetry: ∑ branches ≤ ε
  - Reparametrization Energy: SUFT\_PHI\_RATIO = 13/28
  - Periodicity: PERIOD\_MODULUS = 364
- **HPTP** (High-Precision Timing Protocol): Real hardware timestamps (jitter-corrected via authenticated median-of-3 qutrit triples, HMAC-protected).

---

## 3. Parameters

| Parameter          | Value / Type                  | Description |
|--------------------|-------------------------------|-------------|
| Block size         | 512 bits (64 ternary words)   | Matches common AEAD sizes |
| Key size           | 256–512 bits (CNSA 2.0)       | Hybrid with ML-KEM-1024 |
| Nonce              | 128 bits (HPTP-derived)       | Femtosecond-anchored |
| Rounds (Primary)   | 13 (tied to 13D Plenum)       | Diffusion rounds |
| Rounds (Guardian)  | 7                             | Lightweight checksum |
| τ-weight order     | k = 0..12                     | For Tribonacci diffusion |
| Tolerance (Noether)| ε = 2^{-20}                   | Invariant check threshold |

---

## 4. Algorithms

### 4.1 Key Generation (PhaseKeyGen)

```rust
fn phase_keygen(seed: &[u8; 32]) -> (PrimaryKey, GuardianKey) {
    let primary = ml_kem_1024::encapsulate(seed);  // CNSA outer
    let guardian = tribonacci_hash(seed, 13);       // τ-weighted
    (primary, guardian)
}
```

### 4.2 Encryption (PhaseEncrypt)

**Input:** plaintext M (512 bits), key K, nonce N (HPTP timestamp)
**Output:** ciphertext C, tag T (guardian)

1. **Pack** M into GF(3) ternary registers (27 physical registers).
2. **Primary Phase Encoding**:
   - Convert to Complex64 fixed-point.
   - Apply 13 adaptive rotation rounds:
     ```
     z_{i+1} = z_i * exp(2πi * (τ^i * φ + HPTP_phase(N)))
     ```
     (φ derived from Noether reparametrization energy 13/28)
3. **QCorrect** stabilizer application (post-encoding).
4. **Guardian Phase**:
   - Compute Tribonacci-weighted checksum:
     ```
     G = ∑ (z_j * τ^j) mod PERIOD_MODULUS=364
     ```
   - Apply Noether periodicity check.
5. **Output** C = primary waveform + QCorrect syndrome, T = HMAC(guardian, N).

### 4.3 Decryption (PhaseDecrypt)

**Input:** C, T, K, N
**Output:** M or ⊥ (failure)

1. **Verify Guardian** first (constant-time):
   - Recompute G' from C.
   - Check HMAC(T, N) and Noether invariants.
   - If any fail → silent reject.
2. **Primary Phase Inversion**:
   - Undo rotations using inverse phases.
   - Apply QCorrect syndrome correction.
3. **Unpack** to binary.

**Failure Modes**: All failures are silent (generic error) to prevent oracle attacks.

---

## 5. Security Considerations

### 5.1 Attack Resistance

| Attack Vector        | Mitigation |
|----------------------|------------|
| Side-channel         | Constant-time everywhere (ct\_utils, lookup tables, inline const fn) |
| Timing               | HPTP + HMAC-authenticated jitter correction |
| Fault-injection      | \[\[3,1,2\]\]\_3 QEC + Noether invariants |
| Quantum (Grover)     | Guardian phase survives due to τ-irrational diffusion |
| Oracle               | All failures return generic error; guardian validation never exposed |

**Novelty**: The dual-phase + ternary + physical timing binding has no direct analogue. Formal security analysis is provided in **TM-2026-011** (`docs/proofs/Phase-Encryption-Security-Proof.md`), which proves IND-CPA security via sponge indifferentiability reduction, provides INT-CTXT bounds, and formalizes the orthogonal security model.

### 5.2 Known Limitations (Transparency)

- Still relies on standard PQC for outer layers (hybrid design).
- Precision: 64-bit complex fixed-point sufficient for 2^40 operations; re-anchoring recommended beyond that.
- No formal proof yet (this is the purpose of the draft).
- Phase encryption floating-point paths mitigated via timing barriers; full CORDIC conversion flagged for medium-term work.

### 5.3 Audit Status

| Area                          | Previous Severity | Current Severity | Notes |
|-------------------------------|-------------------|------------------|-------|
| Side-channel in QCorrect      | High              | Low              | Lookup tables + ct\_utils |
| Timing attack via HPTP        | Medium            | Low              | HMAC auth + constant-time compare |
| Noether invariant exposure    | Medium            | Low              | Public endpoint with validation |
| GF(3) conversion performance  | Medium            | Very Low         | const fn + inline |
| Phase Encryption overall      | Critical/Medium   | Medium           | Implementation hygiene improved; design needs external review |

---

## 6. Test Vectors (Sample — v0.3)

### Vector 1: All-zero plaintext

| Field          | Value |
|----------------|-------|
| Key            | `00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00` |
| Nonce          | Salvi Epoch (2025-04-01T00:00:00.000000000Z) |
| Plaintext      | `00` × 64 bytes |
| Guardian Tag   | Valid |
| Noether Check  | Passed (gauge ≤ ε, energy = 13/28, period mod 364 = 0) |

### Vector 2: Tampered ciphertext

| Field          | Value |
|----------------|-------|
| Modification   | Single bit flip in primary phase byte 7 |
| Guardian Tag   | **Invalid** — immediate detection |
| Error Response | Generic "Recombination failed" (no oracle leakage) |

### Vector 3: Noether invariant violation

| Field          | Value |
|----------------|-------|
| Modification   | Gauge symmetry artificially broken (branch sum > ε) |
| Result         | Silent reject before primary phase inversion |
| API Response   | Generic error with no internal state exposure |

Full test vector suite (100+ vectors) available in repository: `tests/phase_encryption_vectors.rs`

---

## 7. Integration with Salvi Framework

### 7.1 TVM ISA Opcodes

| Opcode | Hex  | Operation |
|--------|------|-----------|
| PHASE\_ENC      | 0xA6 | Phase encryption of ternary register block |
| PHASE\_DEC      | 0xA7 | Phase decryption with guardian verification |
| GUARDIAN\_CHECK | 0xA8 | Standalone guardian tag verification |

### 7.2 Hardware Acceleration

RISC-V Ternary Extension (`XPlenum/rtl/xplenum_top.v`) accelerates Phase + Guardian operations in silicon.

### 7.3 API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST   | `/api/salvi/phase/split`     | Phase-split encryption |
| POST   | `/api/salvi/phase/recombine` | Phase recombination (decryption) |
| POST   | `/api/salvi/phase/config`    | Encryption configuration |
| POST   | `/api/salvi/phase/recommend` | Security recommendations |
| POST   | `/api/salvi/ternary/noether-verify` | Public Noether invariant verification |
| GET    | `/api/health`                | System health + git commit hash |

### 7.4 Deployment

- **Research Demo**: Replit (PlenumNET.replit.app) — for public API exploration only.
- **Production**: Requires mTLS + HSM for key material. Critical endpoints on hardened infrastructure.

---

## 8. Compliance

### 8.1 Export Control

- ECCN 5D002 — Cryptographic software with symmetric key > 56 bits.
- Wassenaar Arrangement Category 5, Part 2.
- See `EXPORT-CONTROL.md` for full classification guidance.

### 8.2 Standards Alignment

| Standard       | Status |
|----------------|--------|
| CNSA 2.0       | Hybrid outer layer (ML-KEM-1024, ML-DSA-87) |
| FIPS 140-3     | Readiness checklist in quantum simulator |
| GDPR / PIPEDA  | Data processing under Capomastro Holdings Ltd. policies |

---

## 9. Next Steps for Finalization

1. **Community Review** — Post to ePrint + IACR (target: March 2026).
2. **Formal Analysis** — Commission NCC Group / Trail of Bits for Phase + Guardian ($80k–$120k budget).
3. **Test Vectors Expansion** — 1,000+ vectors + differential cryptanalysis suite.
4. **Hardware Tape-out** — Synthesize Verilog for Artix-7, run side-channel evaluation.
5. **Paper** — "Phase Encryption: Ternary Dual-Phase Cipher with Physical-Timing Binding" (10–15 pages).

---

## 10. References

1. NIST CNSA 2.0 Algorithm Suite (2022)
2. FIPS 140-3: Security Requirements for Cryptographic Modules
3. Tribonacci Numbers and Applications (Feinberg, 1963)
4. Noether's Theorem: Invariant Variational Problems (Noether, 1918; Tavel translation, 1971)
5. Post-Quantum Cryptography Standardization (NIST PQC, 2024)
6. Stabilizer Codes and Quantum Error Correction (Gottesman, 1997)

---

*Document generated from Salvi Framework v0.3 specification. For questions or review contributions, contact the project maintainers via the repository issue tracker.*
