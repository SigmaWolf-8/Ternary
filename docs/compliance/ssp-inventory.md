# Sensitive Security Parameter (SSP) Inventory
## Salvi Ternary Cryptographic Module v3.0.0
## Capomastro Holdings Ltd. | Applied Physics Division

---

## SSP Index

23 Sensitive Security Parameters identified within the cryptographic module boundary.

| ID | Name | Type | Module | Bits | Generation | Zeroization |
|---|---|---|---|---|---|---|
| SSP-001 | AES-256 Encryption Key | Symmetric CSP | cipher.rs | 256 | DRBG | ct_zeroize |
| SSP-002 | AES-256-GCM Nonce/IV | IV | cipher.rs | 96 | DRBG/counter | ct_zeroize |
| SSP-003 | AES-256-GCM Auth Tag | PSP | cipher.rs | 128 | Computed | ct_zeroize |
| SSP-004 | TL-KEM-1024 Decapsulation Key | Asymmetric CSP | tl_kem.rs | varies | DRBG | ct_zeroize |
| SSP-005 | TL-KEM-1024 Encapsulation Key | Public Key (PSP) | tl_kem.rs | varies | Derived | N/A (public) |
| SSP-006 | TL-KEM Shared Secret | Symmetric CSP | tl_kem.rs | 256 | Encaps/Decaps | ct_zeroize |
| SSP-007 | TL-DSA-87 Signing Key | Asymmetric CSP | tl_dsa.rs | varies | DRBG | ct_zeroize |
| SSP-008 | TL-DSA-87 Verification Key | Public Key (PSP) | tl_dsa.rs | varies | Derived | N/A (public) |
| SSP-009 | LMS Private Key | Asymmetric CSP | signature.rs | 256+ | DRBG | ct_zeroize |
| SSP-010 | LMS State Index | State CSP | signature.rs | 32 | Counter | ct_zeroize |
| SSP-011 | XMSS Private Key | Asymmetric CSP | signature.rs | 256+ | DRBG | ct_zeroize |
| SSP-012 | XMSS State Index | State CSP | signature.rs | 32 | Counter | ct_zeroize |
| SSP-013 | HMAC Key | Symmetric CSP | hmac.rs | >=256 | DRBG/Input | ct_zeroize |
| SSP-014 | KDF Secret | Symmetric CSP | kdf.rs | >=256 | Input | ct_zeroize |
| SSP-015 | DRBG Internal V | State CSP | drbg.rs | 384 | HMAC derivation | drbg_uninstantiate |
| SSP-016 | DRBG Internal Key | State CSP | drbg.rs | 384 | HMAC derivation | drbg_uninstantiate |
| SSP-017 | DRBG Entropy Seed | Ephemeral CSP | entropy.rs -> drbg.rs | 384 | Entropy source | ct_zeroize after use |
| SSP-018 | DRBG Nonce | Ephemeral | entropy.rs | 192 | Timestamp+counter | ct_zeroize after use |
| SSP-019 | Firmware Signing Key | Asymmetric CSP | firmware_sign.rs | varies | DRBG | ct_zeroize |
| SSP-020 | X.509 CA Signing Key | Asymmetric CSP | x509.rs | varies | DRBG | ct_zeroize |
| SSP-021 | X.509 Serial Number | PSP | x509.rs | 128 | DRBG | N/A |
| SSP-022 | Phase-Encryption Key | Symmetric CSP | phase_cnsa.rs | 256 | ML-KEM shared secret | ct_zeroize |
| SSP-023 | Module Integrity HMAC Key | Symmetric CSP | self_test.rs | 384 | Build-time embed | N/A (read-only) |

---

## SSP Classification

Per ISO/IEC 19790 §7.9:
- **CSP (Critical Security Parameter):** Secret/private keys and other security-related information whose disclosure or modification can compromise module security.
- **PSP (Public Security Parameter):** Public keys and other security-related information whose disclosure does not compromise module security.

---

## Detailed SSP Descriptions

### SSP-001: AES-256 Encryption Key
- **Type:** Critical Security Parameter (CSP), Symmetric Key
- **Size:** 256 bits
- **Generation:** Output of `drbg_generate(256)` from drbg.rs
- **Entry Method:** API parameter to `Aes256GcmEncrypt` / `Aes256GcmDecrypt` service
- **Storage:** In-memory only (stack allocation, no heap, no persistence)
- **Output Method:** Never exported in plaintext. Used internally for encrypt/decrypt.
- **Zeroization:** `ct_zeroize()` called on Drop. Overwrites all 32 bytes with 0x00.
  Verified constant-time by ct_utils.rs.
- **Services Using:** Aes256GcmEncrypt, Aes256GcmDecrypt
- **Association:** Used with SSP-002 (nonce) and SSP-003 (tag)

### SSP-002: AES-256-GCM Nonce/IV
- **Type:** Initialization Vector (IV)
- **Size:** 96 bits (12 bytes)
- **Generation:** Output of `drbg_generate(96)` or deterministic counter per NIST SP 800-38D §8.2.2
- **Entry Method:** API parameter or internally generated
- **Storage:** In-memory only (stack)
- **Output Method:** Transmitted with ciphertext (non-secret per FIPS 197)
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** Aes256GcmEncrypt
- **Association:** Used with SSP-001 (key) and SSP-003 (tag)

### SSP-003: AES-256-GCM Authentication Tag
- **Type:** Public Security Parameter (PSP)
- **Size:** 128 bits (16 bytes)
- **Generation:** Computed during AES-GCM encryption (GHASH)
- **Entry Method:** Computed internally during encryption; input during decryption
- **Storage:** In-memory only (stack)
- **Output Method:** Returned as part of authenticated ciphertext
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** Aes256GcmEncrypt, Aes256GcmDecrypt

### SSP-004: TL-KEM-1024 Decapsulation Key
- **Type:** Critical Security Parameter (CSP), Asymmetric Private Key
- **Size:** Variable (lattice polynomial coefficients in GF(3))
- **Generation:** `drbg_generate()` seeds polynomial sampling in `tl_kem.rs`
- **Entry Method:** Generated internally via `TlKem1024Keygen` service
- **Storage:** In-memory only. Never written to persistent storage.
- **Output Method:** Never exported in plaintext. Used only for decapsulation.
- **Zeroization:** `ct_zeroize()` on Drop. All polynomial coefficient memory overwritten.
- **Services Using:** TlKem1024Keygen, TlKem1024Decapsulate
- **Association:** Paired with SSP-005 (encapsulation key)

### SSP-005: TL-KEM-1024 Encapsulation Key
- **Type:** Public Security Parameter (PSP), Asymmetric Public Key
- **Size:** Variable (lattice polynomial coefficients in GF(3))
- **Generation:** Derived from decapsulation key during keygen
- **Entry Method:** Generated during `TlKem1024Keygen`; input during `TlKem1024Encapsulate`
- **Storage:** In-memory only
- **Output Method:** Exported as public key for distribution
- **Zeroization:** N/A (public key, no secrecy requirement)
- **Services Using:** TlKem1024Keygen, TlKem1024Encapsulate

### SSP-006: TL-KEM Shared Secret
- **Type:** Critical Security Parameter (CSP), Symmetric Key Material
- **Size:** 256 bits
- **Generation:** Output of encapsulation (sender) or decapsulation (receiver) in tl_kem.rs
- **Entry Method:** Computed internally
- **Storage:** In-memory only (stack)
- **Output Method:** Returned to caller for key derivation
- **Zeroization:** `ct_zeroize()` on Drop. Constant-time overwrite.
- **Services Using:** TlKem1024Encapsulate, TlKem1024Decapsulate

### SSP-007: TL-DSA-87 Signing Key
- **Type:** Critical Security Parameter (CSP), Asymmetric Private Key
- **Size:** Variable (lattice polynomial coefficients in GF(3))
- **Generation:** `drbg_generate()` seeds polynomial sampling in `tl_dsa.rs`
- **Entry Method:** Generated internally via `TlDsa87Keygen`
- **Storage:** In-memory only
- **Output Method:** Never exported in plaintext
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** TlDsa87Keygen, TlDsa87Sign

### SSP-008: TL-DSA-87 Verification Key
- **Type:** Public Security Parameter (PSP), Asymmetric Public Key
- **Size:** Variable
- **Generation:** Derived from signing key during keygen
- **Entry Method:** Generated during `TlDsa87Keygen`; input during `TlDsa87Verify`
- **Storage:** In-memory only
- **Output Method:** Exported as public key
- **Zeroization:** N/A (public key)
- **Services Using:** TlDsa87Keygen, TlDsa87Verify

### SSP-009: LMS Private Key
- **Type:** Critical Security Parameter (CSP), Asymmetric Private Key (Stateful)
- **Size:** 256+ bits (Merkle tree private key with OTS seeds)
- **Generation:** `drbg_generate()` for seed material; Merkle tree computed in signature.rs
- **Entry Method:** Generated via `LmsKeygen`
- **Storage:** In-memory only. State index (SSP-010) tracks usage.
- **Output Method:** Never exported in plaintext
- **Zeroization:** `ct_zeroize()` on Drop. Tree nodes and OTS keys overwritten.
- **Services Using:** LmsKeygen, LmsSign
- **SP 800-208:** Monotonic index advancement prevents key reuse. `StateExhausted` enforced.

### SSP-010: LMS State Index
- **Type:** State CSP (Critical for stateful signature security)
- **Size:** 32 bits
- **Generation:** Initialized to 0 at keygen; monotonically incremented per signature
- **Entry Method:** Internal counter
- **Storage:** In-memory only
- **Output Method:** Never exported directly. Current value queryable for state management.
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** LmsSign
- **Security Note:** Index MUST NOT be decremented. Reuse of index compromises LMS security.

### SSP-011: XMSS Private Key
- **Type:** Critical Security Parameter (CSP), Asymmetric Private Key (Stateful)
- **Size:** 256+ bits (WOTS+ seeds + Merkle tree authentication path)
- **Generation:** `drbg_generate()` for seed; tree computed in signature.rs
- **Entry Method:** Generated via `XmssKeygen`
- **Storage:** In-memory only
- **Output Method:** Never exported in plaintext
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** XmssKeygen, XmssSign
- **SP 800-208:** Heights 10/16/20 supported. w=16 Winternitz parameter. L-tree compression.

### SSP-012: XMSS State Index
- **Type:** State CSP
- **Size:** 32 bits
- **Generation:** Initialized to 0; monotonically incremented per signature
- **Entry Method:** Internal counter
- **Storage:** In-memory only
- **Output Method:** Not exported
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** XmssSign
- **Security Note:** Same monotonic enforcement as SSP-010.

### SSP-013: HMAC Key
- **Type:** Critical Security Parameter (CSP), Symmetric Key
- **Size:** >= 256 bits (recommended >= security strength of hash)
- **Generation:** Via `drbg_generate()` or provided as API input
- **Entry Method:** API parameter to `HmacSha384` / `HmacSha512` service
- **Storage:** In-memory only (stack)
- **Output Method:** Never exported in plaintext
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** HmacSha384, HmacSha512

### SSP-014: KDF Secret
- **Type:** Critical Security Parameter (CSP), Symmetric Key Material
- **Size:** >= 256 bits
- **Generation:** Input from external key exchange or `drbg_generate()`
- **Entry Method:** API parameter to KDF functions in kdf.rs
- **Storage:** In-memory only (stack)
- **Output Method:** Derived keys output; source secret never re-exported
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** Key derivation (internal)

### SSP-015: DRBG Internal V (State Variable)
- **Type:** State CSP (internal to DRBG, never exposed)
- **Size:** 384 bits (48 bytes, SHA-384 output length)
- **Generation:** HMAC-SHA-384 derivation during `drbg_instantiate()` and `drbg_reseed()`
- **Entry Method:** Computed internally from entropy + nonce
- **Storage:** In-memory within `DrbgState` struct in drbg.rs
- **Output Method:** Never exported. Used internally for HMAC-DRBG update function.
- **Zeroization:** `drbg_uninstantiate()` — overwrites V with zeros
- **Services Using:** DrbgGenerate (internal state)
- **SP 800-90A:** Updated on every generate call via HMAC-DRBG Update.

### SSP-016: DRBG Internal Key
- **Type:** State CSP (internal to DRBG, never exposed)
- **Size:** 384 bits (48 bytes)
- **Generation:** HMAC-SHA-384 derivation during `drbg_instantiate()` and `drbg_reseed()`
- **Entry Method:** Computed internally from entropy + nonce
- **Storage:** In-memory within `DrbgState` struct in drbg.rs
- **Output Method:** Never exported.
- **Zeroization:** `drbg_uninstantiate()` — overwrites Key with zeros
- **Services Using:** DrbgGenerate (internal state)
- **SP 800-90A:** Updated on every generate call via HMAC-DRBG Update.

### SSP-017: DRBG Entropy Seed
- **Type:** Ephemeral CSP (destroyed immediately after use)
- **Size:** 384 bits (48 bytes, >= security_strength per SP 800-90A)
- **Generation:** `EntropySource::get_entropy()` in entropy.rs — conditioned via HMAC-SHA-384
- **Entry Method:** Internal pipeline: noise source -> health tests -> conditioning -> DRBG
- **Storage:** Temporary stack variable during instantiate/reseed
- **Output Method:** Never exported. Consumed by `drbg_instantiate()` / `drbg_reseed()`.
- **Zeroization:** `ct_zeroize()` immediately after DRBG consumes the seed
- **Services Using:** DrbgInstantiate, DrbgReseed
- **Lifecycle:** Generated, used, destroyed — never persisted.

### SSP-018: DRBG Nonce
- **Type:** Ephemeral (non-secret, but zeroized as defense-in-depth)
- **Size:** 192 bits (24 bytes)
- **Generation:** Timestamp + monotonic counter in entropy.rs `get_nonce()`
- **Entry Method:** Computed internally
- **Storage:** Temporary stack variable
- **Output Method:** Never exported
- **Zeroization:** `ct_zeroize()` after use
- **Services Using:** DrbgInstantiate
- **SP 800-90A:** Nonce required for DRBG instantiation to prevent key collision.

### SSP-019: Firmware Signing Key
- **Type:** Critical Security Parameter (CSP), Asymmetric Private Key
- **Size:** Variable (ML-DSA-87 format)
- **Generation:** `drbg_generate()` in firmware_sign.rs
- **Entry Method:** Generated via firmware signing pipeline
- **Storage:** In-memory only
- **Output Method:** Never exported in plaintext
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** Firmware signing (application-level, Non-Approved Mode)

### SSP-020: X.509 CA Signing Key
- **Type:** Critical Security Parameter (CSP), Asymmetric Private Key
- **Size:** Variable (ML-DSA-87 format)
- **Generation:** `drbg_generate()` in x509.rs
- **Entry Method:** Generated during CA certificate creation
- **Storage:** In-memory only
- **Output Method:** Never exported in plaintext
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** X.509 certificate signing (application-level, Non-Approved Mode)

### SSP-021: X.509 Serial Number
- **Type:** Public Security Parameter (PSP)
- **Size:** 128 bits
- **Generation:** `drbg_generate(128)` per RFC 5280 §4.1.2.2
- **Entry Method:** Generated internally
- **Storage:** Embedded in certificate structure
- **Output Method:** Included in X.509 certificate (public)
- **Zeroization:** N/A (public, non-secret)
- **Services Using:** X.509 certificate creation

### SSP-022: Phase-Encryption Key
- **Type:** Critical Security Parameter (CSP), Symmetric Key
- **Size:** 256 bits
- **Generation:** Derived from ML-KEM-1024 shared secret via temporal binding in phase_cnsa.rs
- **Entry Method:** Output of hybrid key exchange (ML-KEM + phase encryption)
- **Storage:** In-memory only
- **Output Method:** Never exported in plaintext. Used for phase encrypt/decrypt.
- **Zeroization:** `ct_zeroize()` on Drop
- **Services Using:** PhaseEncrypt, PhaseDecrypt (Non-Approved Mode)
- **Note:** Phase encryption is ternary-native, non-standard. Available only in Non-Approved Mode.

### SSP-023: Module Integrity HMAC Key
- **Type:** Symmetric CSP (read-only, embedded at build time)
- **Size:** 384 bits (48 bytes)
- **Generation:** Computed during `cmvp-build.sh` — HMAC-SHA-384 of module binary
- **Entry Method:** Embedded in self_test.rs as compile-time constant
- **Storage:** Read-only program memory
- **Output Method:** Never exported. Used only for POST integrity verification.
- **Zeroization:** N/A (read-only constant, cannot be modified at runtime)
- **Services Using:** SelfTestRun (POST integrity check)
- **Security Note:** Changing this value requires rebuilding the module.

---

## Zeroization Methods

| Method | Implementation | Verification |
|---|---|---|
| `ct_zeroize()` | ct_utils.rs — overwrite bytes with 0x00 in constant time | formal_verify.rs property CT-ZERO |
| `ct_zeroize_i8()` | ct_utils.rs — overwrite i8 arrays in constant time | formal_verify.rs |
| `drbg_uninstantiate()` | Zeroizes V and Key fields of DrbgState | Tested in self_test.rs |
| Drop trait | Rust automatic cleanup — calls ct_zeroize in destructor | Code review verified |

## Key Lifecycle Diagram

```
[EntropySource]
      │
      │ get_entropy(384)
      ▼
[DRBG Seed (SSP-017)] ──── ct_zeroize immediately after ────┐
      │                                                       │
      │ drbg_instantiate()                                    │
      ▼                                                       │
[DRBG State (SSP-015, SSP-016)]                              │
      │                                                       │
      │ drbg_generate()                                       │
      ▼                                                       │
[Key Material (SSP-001, SSP-004, SSP-007, ...)]              │
      │                                                       │
      │ Used by CryptoService                                 │
      ▼                                                       │
[Crypto Operation Output]                                     │
      │                                                       │
      │ Drop / explicit zeroize                               │
      ▼                                                       ▼
[ct_zeroize() — all memory overwritten with 0x00] ◄──────────┘
```

## SSP Access Control

| SSP Category | CryptoOfficer | User | None |
|---|---|---|---|
| Symmetric CSPs (SSP-001, 006, 013, 014, 022) | Generate, Use | Use | Denied |
| Asymmetric Private CSPs (SSP-004, 007, 009, 011, 019, 020) | Generate, Use | Use | Denied |
| Public Keys (SSP-005, 008, 021) | Generate, Export | Export | Denied |
| DRBG State (SSP-015, 016, 017, 018) | Instantiate, Reseed | Generate | Denied |
| State Indices (SSP-010, 012) | Generate | Sign (advances) | Denied |
| Integrity Key (SSP-023) | Read-only | Read-only | Denied |

---

*Document: VE-004*
*Salvi Framework — Capomastro Holdings Ltd.*
