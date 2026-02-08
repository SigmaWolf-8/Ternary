# FIPS Validation Plan — TL-KEM & TL-DSA

## Document Information

| Field | Value |
|-------|-------|
| Document | FIPS Validation Plan |
| Version | 1.0 |
| Date | February 2026 |
| Owner | Capomastro Holdings Ltd. |
| Classification | Internal |

---

## 1. Scope

This document defines the formal validation roadmap for PlenumNET's post-quantum cryptographic algorithms against NIST FIPS standards:

| Algorithm | FIPS Equivalent | Target Standard |
|-----------|----------------|-----------------|
| TL-KEM-512 | ML-KEM-512 | FIPS 203 |
| TL-KEM-768 | ML-KEM-768 | FIPS 203 |
| TL-KEM-1024 | ML-KEM-1024 | FIPS 203 |
| TL-DSA-44 | ML-DSA-44 | FIPS 204 |
| TL-DSA-65 | ML-DSA-65 | FIPS 204 |
| TL-DSA-87 | ML-DSA-87 | FIPS 204 |
| AES-256-GCM | AES-256-GCM | FIPS 197 |
| SHA-384/512 | SHA-384/512 | FIPS 180-4 |
| SHA-3 | SHA-3 | FIPS 202 |

---

## 2. Validation Phases

### Phase 1: Internal Verification (Current — Q1 2026)

**Status**: Complete

| Criterion | Evidence | Status |
|-----------|----------|--------|
| Algorithm correctness | Unit tests (keygen, encaps/decaps, sign/verify) | Done |
| Deterministic outputs | Same seed produces same keys/signatures | Done |
| Cross-level consistency | All 3 security levels produce correct results | Done |
| Rejection behavior | Wrong keys produce different shared secrets (KEM) | Done |
| Implicit rejection | Modified ciphertexts trigger rejection path (KEM) | Done |
| Forgery resistance | Wrong key/message signature verification fails (DSA) | Done |
| Abort mechanism | Signing retries on norm bound violation (DSA) | Done |
| CNSA 2.0 mapping | 11/11 algorithms tracked as TernaryEquivalent | Done |

### Phase 2: Interoperability Testing (Q2 2026)

| Criterion | Description | Target |
|-----------|-------------|--------|
| Binary round-trip | ML-KEM input -> TL-KEM -> ML-KEM output identity | Q2 2026 |
| Cross-implementation | Verify against reference ML-KEM/ML-DSA implementations | Q2 2026 |
| KAT vectors | Generate Known Answer Test vectors for all variants | Q2 2026 |
| Performance benchmarks | Timing analysis at each security level | Q2 2026 |
| Side-channel analysis | Constant-time verification for critical paths | Q2 2026 |

### Phase 3: CMVP Preparation (Q3-Q4 2026)

| Criterion | Description | Target |
|-----------|-------------|--------|
| CAVP algorithm testing | Submit to NIST CAVP for algorithm validation | Q3 2026 |
| Security policy document | Formal security policy per FIPS 140-3 | Q3 2026 |
| Finite state model | Document module states and transitions | Q3 2026 |
| Entropy source documentation | Document RNG/seed generation procedures | Q3 2026 |
| Physical security assessment | For HSM-hosted implementations | Q4 2026 |

### Phase 4: Formal Submission (2027)

| Criterion | Description | Target |
|-----------|-------------|--------|
| CMVP lab engagement | Select accredited testing lab | Q1 2027 |
| Module submission | Submit cryptographic module for validation | Q2 2027 |
| Lab testing | Complete testing cycle with accredited lab | Q3 2027 |
| Certificate issuance | Receive FIPS 140-3 validation certificate | Q4 2027 |

---

## 3. Known Answer Tests (KAT)

### TL-KEM KAT Requirements

For each variant (512, 768, 1024):

| Test | Input | Expected Output |
|------|-------|-----------------|
| KeyGen determinism | Fixed seed S | Deterministic (pk, sk) pair |
| Encapsulation | Fixed (pk, randomness) | Deterministic (ct, ss) pair |
| Decapsulation | Correct (sk, ct) | Same ss as encapsulation |
| Decapsulation failure | Wrong sk, correct ct | Different ss (implicit reject) |
| Ciphertext modification | Modified ct byte | Rejection ss derived from reject seed |

### TL-DSA KAT Requirements

For each variant (44, 65, 87):

| Test | Input | Expected Output |
|------|-------|-----------------|
| KeyGen determinism | Fixed seed S | Deterministic (pk, sk) pair |
| Sign determinism | Fixed (sk, message) | Deterministic signature |
| Verify success | Correct (pk, msg, sig) | true |
| Verify failure (wrong msg) | Correct pk, wrong msg, valid sig | false |
| Verify failure (wrong key) | Wrong pk, correct msg, valid sig | false |
| Challenge sparsity | Fixed seed | tau non-zero coefficients |

---

## 4. Evidence Requirements

### Per Algorithm

Each algorithm must produce the following documentation:

1. **Algorithm Specification** — Mathematical description of operations
2. **Security Proof Reference** — Citation to formal security reduction
3. **Implementation Notes** — Mapping from standard to ternary domain
4. **Test Vectors** — Minimum 100 KAT vectors per variant
5. **Performance Data** — Operations per second at each security level
6. **Side-Channel Report** — Timing analysis for constant-time verification

### Repository Artifacts

| Artifact | Location | Status |
|----------|----------|--------|
| TL-KEM implementation | `src/kernel/src/crypto/tl_kem.rs` | Complete |
| TL-DSA implementation | `src/kernel/src/crypto/tl_dsa.rs` | Complete |
| Lattice primitives | `src/kernel/src/crypto/ternary_lattice.rs` | Complete |
| CNSA 2.0 tracker | `src/kernel/src/crypto/cnsa2.rs` | Complete |
| AES-256-GCM | `src/kernel/src/crypto/cipher.rs` | Complete |
| SHA-2 | `src/kernel/src/crypto/sha2.rs` | Complete |
| SHA-3 | `src/kernel/src/crypto/sha3.rs` | Complete |
| Binary compat layer | `src/kernel/src/compat/gateway.rs` | Complete |
| Compliance check CI | `.github/workflows/compliance-check.yml` | Complete |
| This validation plan | `docs/compliance/fips-validation-plan.md` | Complete |

---

## 5. Ternary-to-Binary Equivalence

### Equivalence Argument

PlenumNET operates in balanced ternary (GF(3)) rather than binary (GF(2)). The FIPS validation must demonstrate functional equivalence:

| Property | Binary (ML-KEM/DSA) | Ternary (TL-KEM/DSA) | Equivalence |
|----------|---------------------|----------------------|-------------|
| Ring | Z_q[X]/(X^n+1) | Z_3[X]/(X^n+1) | Structural isomorphism |
| Noise distribution | Binomial | CBD ternary | Comparable security |
| Security reduction | Module-LWE | Module-LWE over GF(3) | Equivalent hardness assumption |
| Key sizes | Binary bytes | Ternary trits | Mapped via BinaryTernaryGateway |
| Shared secret derivation | SHAKE-256 | TernarySponge | Hash domain separation |

### Binary Compatibility Layer

The `BinaryTernaryGateway` (src/kernel/src/compat/gateway.rs) provides bidirectional conversion:

- `binary_bytes_to_ternary()` — Convert standard binary inputs to ternary
- `ternary_to_binary_bytes()` — Convert ternary outputs to standard binary
- Enables hybrid deployment with binary ML-KEM/ML-DSA systems

---

## 6. Regulatory Alignment

### CNSA 2.0 Timeline Compliance

| NSA Milestone | Date | PlenumNET Status |
|---------------|------|-----------------|
| Software/firmware signing with CNSA 2.0 | 2025 | TL-DSA implemented |
| Web browsers/servers with CNSA 2.0 | 2025 | TL-KEM + TL-DSA available |
| Traditional networking with CNSA 2.0 | 2026 | Binary compat layer in progress |
| Operating systems with CNSA 2.0 | 2027 | Kernel integration planned |
| Niche equipment with CNSA 2.0 | 2030 | FPGA/ASIC synthesis planned |
| Legacy system retirement | 2033 | Full ternary migration target |

### Additional Compliance Frameworks

| Framework | Requirement | PlenumNET Coverage |
|-----------|-------------|-------------------|
| FINRA 613 | Microsecond timestamp accuracy | Femtosecond timing (HPTP) |
| MiFID II | Transaction time synchronization | HPTP compliance certification |
| SOC 2 Type II | Cryptographic key management | Key rotation procedures defined |

---

*Last Updated: February 2026*
*Classification: Internal — Capomastro Holdings Ltd.*
