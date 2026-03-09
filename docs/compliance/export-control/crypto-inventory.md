# Cryptographic Algorithm Inventory

**Capomastro Holdings Ltd. — Applied Physics Division**
**Last Updated:** February 15, 2026
**Classification:** INTERNAL — Export Control Review Required

---

## 1. Post-Quantum Cryptographic Primitives

| Algorithm | Type | Key Size | ECCN | Status | Location |
|-----------|------|----------|------|--------|----------|
| TL-KEM (Ternary Lattice KEM) | Key Encapsulation | Proprietary | 5D002 | Classification pending | `src/kernel/src/crypto/tl_kem.rs` |
| TL-DSA (Ternary Lattice DSA) | Digital Signature | Proprietary | 5D002 | Classification pending | `src/kernel/src/crypto/tl_dsa.rs` |
| Phase Encryption | Timing-Gated Symmetric | Proprietary | 5D002 | Classification pending | `server/salvi-core/phase-encryption.ts` |
| TL-Sponge-385 | Hash Construction | N/A | Review needed | Classification pending | `src/kernel/src/crypto/sponge.rs` |
| Ternary Lamport Signatures | One-Time Signature | Proprietary | 5D002 | Classification pending | `src/kernel/src/crypto/cipher.rs` |

## 2. CNSA 2.0 Classical Algorithms

| Algorithm | Standard | Key/Hash Size | ECCN | Status | Location |
|-----------|----------|---------------|------|--------|----------|
| AES-256-GCM | FIPS 197 | 256-bit | 5D002 (>56-bit) | Controlled | `server/crypto-utils.ts`, `src/kernel/src/crypto/cipher.rs` |
| SHA-2 (SHA-256, SHA-384, SHA-512) | FIPS 180-4 | 256/384/512-bit | Generally uncontrolled | Exempt (hash only) | `src/kernel/src/crypto/sha2.rs` |
| SHA-3 | FIPS 202 | Variable | Generally uncontrolled | Exempt (hash only) | `src/kernel/src/crypto/sponge.rs` |
| HMAC | FIPS 198-1 | Variable | 5D002 when used for authentication | Controlled | `src/kernel/src/crypto/hmac.rs` |
| HMAC-based KDF | SP 800-108 | Variable | 5D002 | Controlled | `src/kernel/src/crypto/kdf.rs` |

## 3. Referenced / Future Algorithms

| Algorithm | Standard | Purpose | Status |
|-----------|----------|---------|--------|
| XMSS / LMS | NIST SP 800-208 | Hash-based signatures | Referenced, not implemented |
| ML-KEM (CRYSTALS-Kyber) | FIPS 203 | Post-quantum KEM | CNSA 2.0 target |
| ML-DSA (CRYSTALS-Dilithium) | FIPS 204 | Post-quantum signatures | CNSA 2.0 target |
| SLH-DSA (SPHINCS+) | FIPS 205 | Stateless hash-based signatures | CNSA 2.0 target |

## 4. Export Control Classification Summary

### 4.1 Applicable Regulations
- **Canadian ECL:** Category 5, Part 2 (Information Security)
- **U.S. EAR:** ECCN 5D002 (Information Security software)
- **Wassenaar Arrangement:** Category 5, Part 2

### 4.2 Potential Exemptions
- **License Exception TSU (EAR 740.13):** Applies if software is publicly available (open source), with no encryption payment condition
- **Canadian ECL Exemption:** Publicly available cryptographic software may qualify under Note 3 to Category 5, Part 2
- **Note:** Proprietary algorithms (TL-KEM, TL-DSA, Phase Encryption) may NOT qualify for public availability exemptions

### 4.3 Restricted Destinations
Cuba, Iran, North Korea, Syria, Russia, Belarus, and any other countries subject to comprehensive U.S./Canadian/EU sanctions.

## 5. Action Items

- [ ] Engage export control attorney for formal ECCN classification
- [ ] Determine applicability of TSU exemption for open-source components
- [ ] File BIS classification request for proprietary algorithms if no exemption applies
- [ ] Implement geo-blocking for restricted destinations
- [ ] Document classification determination and retain records

## 6. Cross-References
- Full export control policy: [`EXPORT-CONTROL.md`](../../../EXPORT-CONTROL.md)
- CNSA 2.0 compliance: [`docs/compliance/security-policy.md`](../security-policy.md)
- IP notice: [`IP-NOTICE.md`](../../../IP-NOTICE.md)
