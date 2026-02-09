# FIPS 140-3 Cryptographic Module Boundary Specification

**Module Name:** Salvi Cryptographic Module  
**Version:** 2.0.0  
**Security Level Target:** FIPS 140-3 Level 2  
**Document Status:** Pre-submission specification

---

## 1. Module Description

The Salvi Cryptographic Module is a software-only cryptographic module implementing post-quantum algorithms required by CNSA 2.0. It operates as a firmware-linked library within the PlenumNET kernel, providing cryptographic services to applications and protocols via a defined API boundary.

---

## 2. Cryptographic Boundary

The module boundary encompasses all files within `src/kernel/src/crypto/` and defines the following logical boundary:

```
┌──────────────────────────────────────────────────────────┐
│                  FIPS MODULE BOUNDARY                     │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Approved Algorithms                    │  │
│  │                                                    │  │
│  │  AES-256-GCM    (FIPS 197)     cipher.rs           │  │
│  │  SHA-384/512    (FIPS 180-4)   sponge.rs           │  │
│  │  SHA3-384/512   (FIPS 202)     sponge.rs           │  │
│  │  TL-KEM         (FIPS 203)     kem.rs              │  │
│  │  TL-DSA         (FIPS 204)     dsa.rs              │  │
│  │  XMSS           (SP 800-208)   signature.rs        │  │
│  │  LMS            (SP 800-208)   signature.rs        │  │
│  │  HMAC           (FIPS 198-1)   mod.rs              │  │
│  │  KDF            (SP 800-108)   mod.rs              │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Critical Security Parameters           │  │
│  │                                                    │  │
│  │  Private signing keys (TL-DSA, XMSS, LMS)         │  │
│  │  KEM decapsulation keys                            │  │
│  │  AES-256 symmetric keys                            │  │
│  │  HMAC keys                                         │  │
│  │  XMSS/LMS state indices (monotonic counters)       │  │
│  │  Session keys (phase_cnsa derived)                  │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Support Functions                      │  │
│  │                                                    │  │
│  │  ct_utils.rs       Constant-time primitives        │  │
│  │  agility.rs        Algorithm policy engine         │  │
│  │  cnsa2.rs          Compliance tracker              │  │
│  │  phase_cnsa.rs     Hybrid key exchange             │  │
│  │  ternary_lattice.rs  GF(3) arithmetic + NTT       │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Non-Approved (Outside FIPS)            │  │
│  │                                                    │  │
│  │  Lamport OTS      (legacy, deprecated)             │  │
│  │  Bijective Cipher  (proprietary, non-FIPS)         │  │
│  │  Phase Encryption  (proprietary temporal binding)   │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## 3. Services and Roles

### 3.1 Services

| Service | Algorithm | Input | Output |
|---------|-----------|-------|--------|
| Key Generation | TL-KEM, TL-DSA, XMSS, LMS, AES | Seed/entropy | Key pair or symmetric key |
| Encapsulation | TL-KEM | Public key, randomness | Ciphertext, shared secret |
| Decapsulation | TL-KEM | Private key, ciphertext | Shared secret |
| Sign | TL-DSA, XMSS, LMS | Private key, message | Signature |
| Verify | TL-DSA, XMSS, LMS | Public key, message, signature | Valid/Invalid |
| Encrypt | AES-256-GCM | Key, plaintext, nonce, AAD | Ciphertext, tag |
| Decrypt | AES-256-GCM | Key, ciphertext, nonce, AAD, tag | Plaintext |
| Hash | SHA-384, SHA-512, SHA3 | Message | Digest |
| MAC | HMAC | Key, message | Tag |
| KDF | SP 800-108 | Key, context, label | Derived key |

### 3.2 Roles

| Role | Permitted Services |
|------|-------------------|
| User | All cryptographic services |
| Crypto Officer | Key management, algorithm policy configuration, state management |

---

## 4. Key Management

### 4.1 Key Storage

All Critical Security Parameters (CSPs) exist in volatile memory only. The module does not provide persistent key storage; callers are responsible for secure key persistence.

**Exception:** XMSS and LMS signing state indices MUST be persisted by the caller to prevent index reuse. The module returns `StateExhausted` if the index space is consumed.

### 4.2 Key Zeroization

The module provides `ct_zeroize` (in `ct_utils.rs`) for secure key destruction:
- Overwrites memory with zeros using volatile writes
- Compiler cannot optimize away the zeroization
- Applied to all CSPs when they leave scope

### 4.3 Key Sizes

| Algorithm | Key Size | Standard |
|-----------|----------|----------|
| AES-256 | 256 bits | FIPS 197 |
| TL-KEM Level 5 | ~6,400 trits (pk), ~12,800 trits (sk) | FIPS 203 |
| TL-DSA Level 5 | ~7,680 trits (pk), ~15,360 trits (sk) | FIPS 204 |
| XMSS-20 | Variable (depends on height) | SP 800-208 |
| LMS-25/W8 | Variable (depends on height, W) | SP 800-208 |

---

## 5. Self-Tests

### 5.1 Power-On Self-Tests (POST)

The module performs the following tests at initialization:
1. AES-256 Known Answer Test (encrypt + decrypt)
2. Sponge hash KAT (fixed input → expected output)
3. TL-KEM encaps/decaps roundtrip
4. TL-DSA sign/verify roundtrip
5. XMSS sign/verify roundtrip
6. LMS sign/verify roundtrip
7. HMAC KAT
8. Constant-time primitive verification

### 5.2 Conditional Self-Tests

- Key pair consistency check after generation (TL-KEM, TL-DSA, XMSS, LMS)
- Continuous RNG test (if hardware RNG available)

### 5.3 CAVP Vectors

210 KAT vectors (35 per algorithm variant) in NIST SP 800-185 format. See `cavp_package.rs`.

---

## 6. Physical Security

As a software module targeting Level 2:
- No physical security mechanisms
- Relies on the operating environment for tamper evidence
- Production-hardened constant-time implementations prevent timing attacks

---

## 7. Side-Channel Mitigations

| Component | Mitigation | Verification |
|-----------|------------|--------------|
| AES S-box | GF(2^8) Fermat inversion | Formal: CT-001 |
| TL-KEM FO | ct_select_vec | Formal: CT-002 |
| Comparison | ct_eq_slices | Formal: CT-003 |
| Key selection | ct_select_u8 | Formal: CT-004 |
| Zeroization | ct_zeroize (volatile) | Formal: CT-005 |

---

## 8. Operational Environment

- **OS:** PlenumNET Kernel (no_std environment)
- **Architecture:** x86_64, aarch64, riscv64
- **Memory:** Kernel-managed allocation
- **Entropy:** Hardware RNG or HPTP timing jitter
- **Dependencies:** `alloc` crate only (no external crypto libraries)

---

## 9. Module Files

| File | Lines | FIPS Role |
|------|-------|-----------|
| `mod.rs` | ~200 | Module entry, error types, HMAC, KDF |
| `sponge.rs` | ~300 | Hash functions |
| `cipher.rs` | ~400 | AES-256-GCM |
| `tl_kem.rs` | ~500 | TL-KEM |
| `tl_dsa.rs` | ~500 | TL-DSA |
| `signature.rs` | ~900 | XMSS, LMS, Lamport |
| `ternary_lattice.rs` | ~1100 | GF(3) arithmetic, NTT |
| `ct_utils.rs` | ~150 | Constant-time primitives |
| `agility.rs` | ~200 | Algorithm policy |
| `cnsa2.rs` | ~300 | Compliance tracker |
| `phase_cnsa.rs` | ~250 | Hybrid key exchange |
| `firmware_sign.rs` | ~200 | Firmware signing |
| `x509.rs` | ~300 | Certificate support |
| `cavp_package.rs` | ~400 | CAVP KAT vectors |
| `fpga_hdl.rs` | ~300 | FPGA HDL generation (outside boundary) |
| `hw_test.rs` | ~200 | Hardware tests (outside boundary) |
| `formal_verify.rs` | ~200 | Formal verification (outside boundary) |
