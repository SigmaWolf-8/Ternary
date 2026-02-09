# Cryptographic Module Security Policy

## Document Information

| Field | Value |
|-------|-------|
| Document | Security Policy |
| Version | 2.0 |
| Date | February 2026 |
| Owner | Capomastro Holdings Ltd. |
| Standard | FIPS 140-3 Level 1 |

---

## 1. Module Description

The PlenumNET Salvi Cryptographic Module implements post-quantum cryptographic algorithms using ternary (balanced GF(3)) arithmetic. The module provides key encapsulation (TL-KEM) and digital signatures (TL-DSA) equivalent to NIST FIPS 203/204, plus symmetric encryption (AES-256-GCM) and hashing (SHA-2, SHA-3, Ternary Sponge).

## 2. Cryptographic Boundary

### Approved Algorithms

| Algorithm | Type | Standard | Security Levels |
|-----------|------|----------|----------------|
| TL-KEM-512/768/1024 | KEM | FIPS 203 equiv. | 1, 3, 5 |
| TL-DSA-44/65/87 | DSA | FIPS 204 equiv. | 2, 3, 5 |
| AES-256-GCM | AEAD | FIPS 197 | 256-bit |
| SHA-384 | Hash | FIPS 180-4 | 192-bit |
| SHA-512 | Hash | FIPS 180-4 | 256-bit |
| SHA3-256 | Hash | FIPS 202 | 128-bit |
| SHA3-512 | Hash | FIPS 202 | 256-bit |
| HMAC-SHA-512 | MAC | FIPS 198-1 | 256-bit |
| Ternary Sponge | Hash | Proprietary | 243-trit |
| Ternary Lamport | Signature | Proprietary | One-time |
| TL-KDF | KDF | SP 800-108 equiv. | Configurable |

### CNSA 2.0 Compliance

Coverage: 11/11 required algorithms (100%)

## 3. Security Modes

| Mode | Description | Access |
|------|-------------|--------|
| Normal | Full cryptographic operations | Authenticated users |
| FIPS | FIPS-approved algorithms only | Enforced |
| Maintenance | Key loading, self-test | Crypto officer |

## 4. Roles

| Role | Responsibilities |
|------|-----------------|
| Crypto Officer | Module configuration, key management, self-test initiation |
| User | Cryptographic operations via approved API |

## 5. Physical Security

FIPS 140-3 Level 1: No physical security mechanisms required for software module. When deployed on FPGA (Kintex UltraScale+), Level 2 tamper-evidence available.

## 6. Key Management

### Key Types

| Key | Algorithm | Storage | Zeroization |
|-----|-----------|---------|-------------|
| TL-KEM Secret Key | TL-KEM | Memory | ct_zeroize |
| TL-DSA Signing Key | TL-DSA | Memory | ct_zeroize |
| AES-256 Key | AES-GCM | Memory | ct_zeroize |
| HMAC Key | HMAC-SHA-512 | Memory | ct_zeroize |

### Key Zeroization

All sensitive key material is zeroized using constant-time volatile writes (`ct_zeroize` / `ct_zeroize_i8`) that are not subject to compiler optimization. Zeroization is formally verified (property MEM-001).

## 7. Self-Tests

### Power-On Self-Tests

| Test | Algorithm | Type |
|------|-----------|------|
| KAT-KEM | TL-KEM all variants | Known Answer Test |
| KAT-DSA | TL-DSA all variants | Known Answer Test |
| KAT-AES | AES-256-GCM | NIST FIPS 197 C.3 |
| KAT-SHA | SHA-2, SHA-3 | NIST vectors |
| KAT-HMAC | HMAC-SHA-512 | RFC 4231 vectors |
| INT-001 | Module integrity | Firmware hash check |

### Conditional Self-Tests

| Test | Trigger |
|------|---------|
| Pair-wise consistency | Key generation |
| Sign/verify consistency | First signature |

## 8. Constant-Time Guarantees

The following operations are formally verified as constant-time:

| Component | Mitigation | Status |
|-----------|-----------|--------|
| AES S-Box | GF(2^8) Fermat inversion (no lookup tables) | Verified |
| AES GHASH | Branchless GF(2^128) multiplication | Verified |
| TL-KEM Decapsulation | ct_select_vec (no secret-dependent branches) | Verified |
| Key comparison | ct_eq_slices (no early termination) | Verified |
| Buffer zeroization | Volatile writes (compiler-safe) | Verified |

## 9. Side-Channel Analysis Summary

| Module | Risk Level | Mitigation |
|--------|-----------|------------|
| AES-256-GCM | None | Bitsliced S-box, branchless GHASH |
| TL-KEM | None | FO transform with ct_select |
| TL-DSA | High (by design) | Rejection sampling inherent |
| Sponge/HMAC | None | No secret-dependent indexing |
