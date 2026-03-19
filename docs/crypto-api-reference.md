# PlenumNET Crypto API Reference

**Version**: 2.0.0  
**Date**: 2026-03-19  
**Author**: RSalvi@Salvigroup.com  
**Crate**: `ternary-math` (+ `inter-cube` service APIs)  
**Blocking**: TM-2026-020.1-PREREQ §8

---

## Table of Contents

1. [TL-Sponge-385](#1-tl-sponge-385)
2. [TIS-27](#2-tis-27)
3. [TL-DSA](#3-tl-dsa)
4. [TL-KEM](#4-tl-kem)
5. [Phase Encryption v3](#5-phase-encryption-v3)
6. [Inter-Cube Service APIs](#6-inter-cube-service-apis)
   - [CRS — Cube Registration Service](#61-crs--cube-registration-service)
   - [CON — Cube Overlay Network](#62-con--cube-overlay-network)
   - [GLB — Geometric Load Balancer](#63-glb--geometric-load-balancer)
   - [FTS — Fault Tolerance Service](#64-fts--fault-tolerance-service)

---

## 1. TL-Sponge-385

**Module**: `ternary_math::tlsponge385` (re-exported as `ternary_math::sponge`)  
**Security**: 385-bit post-quantum  
**State**: 729 balanced trits (3⁶ = 729)  
**Rate**: 243 trits (standard) / 486 trits (bulk)  
**Capacity**: 486 trits (standard) / 243 trits (bulk)  
**Rounds**: 9  
**Permutation**: χ(x) = x¹⁷ over GF(27) = GF(3)[t]/(t³+2t+1), theta mixing, pi rotation, round constants

### 1.1 Types

```rust
pub struct Sponge385Pub { /* internal state: 729 balanced trits + configuration */ }
```

Internal fields are private. Construct via `::new()`, `::new_v1()`, or `::new_tis()`. The struct implements `Clone` for forking mid-operation.

### 1.2 Constructor

```rust
impl Sponge385Pub {
    pub fn new() -> Self;       // 9-round mode (TL-Sponge-385)
    pub fn new_v1() -> Self;    // Legacy (no chi layer, backward compat)
    pub fn new_tis() -> Self;   // 4-round mode (TIS-27)
}
```

### 1.3 Instance Methods

```rust
impl Sponge385Pub {
    pub fn absorb(&mut self, trits: &[i8]);
    pub fn absorb_bytes(&mut self, input: &[u8]);
    pub fn absorb_bytes_stack(&mut self, input: &[u8]);  // Stack-allocated path for ≤256 bytes
    pub fn squeeze(&mut self, trit_count: usize) -> Vec<i8>;
    pub fn clone(&self) -> Self;  // Clone mid-operation for forked outputs
}
```

### 1.4 Public Functions — Standard Rate (243 trits)

#### `hash`

```rust
pub fn hash(input: &[u8], output_len: usize) -> Vec<u8>
```

General-purpose sponge hash. Absorbs `input`, squeezes `output_len` bytes.

**Example:**

```rust
use ternary_math::sponge;

let digest = sponge::hash(b"hello world", 32);
assert_eq!(digest.len(), 32);
assert_eq!(sponge::hash(b"hello world", 32), digest); // deterministic
```

#### `hash_hex`

```rust
pub fn hash_hex(input: &[u8]) -> String
```

Returns a 98-character hex string (49 bytes = 392 bits). Uses the v2 sponge (with chi layer).

**Example:**

```rust
let hex = sponge::hash_hex(b"document payload");
assert_eq!(hex.len(), 98);
assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
```

#### `hash_hex_v1`

```rust
pub fn hash_hex_v1(input: &[u8]) -> String
```

Legacy v1 sponge (no chi layer). For backward-compatible hash verification only.

#### `derive_key`

```rust
pub fn derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8>
```

Domain-separated key derivation. Absorbs `context ‖ material`, squeezes `key_len` bytes. Uses stack-allocated path when input ≤ 256 bytes.

**Example:**

```rust
let tunnel_key = sponge::derive_key(
    b"PlenumNET-CON-v3.0",
    b"shared-material",
    32,
);
assert_eq!(tunnel_key.len(), 32);
```

#### `derive_key_cat`

```rust
pub fn derive_key_cat(context: &[u8], parts: &[&[u8]], key_len: usize) -> Vec<u8>
```

Zero-allocation KDF for multi-part material. Equivalent to `derive_key(context, parts[0] ‖ parts[1] ‖ ..., key_len)` but avoids intermediate allocation for ≤ 512 bytes total.

**Example:**

```rust
let hmac = sponge::derive_key_cat(
    b"PlenumNET-HB-TAG",
    &[hmac_key.as_slice(), message.as_slice()],
    27,
);
```

#### `sponge385_derive_key`

```rust
pub fn sponge385_derive_key(
    domain: &[u8],
    addr_a: &[u8],
    addr_b: &[u8],
    kem_shared_secret: &[u8; 32],
    epoch: u64,
) -> Vec<u8>
```

Topology-derived tunnel key for Inter-Cube CON. Produces a 32-byte key.

**Key formula:**

```
key = TLSponge-385(domain ‖ addr_a ‖ addr_b ‖ kem_shared_secret ‖ epoch_le)
```

**Example:**

```rust
let key = sponge::sponge385_derive_key(
    b"PlenumNET-CON-v3.0",
    &addr_a.to_bytes(),
    &addr_b.to_bytes(),
    &kem_shared_secret,  // [u8; 32] from TL-KEM SharedSecret::to_bytes_32()
    epoch,
);
```

#### `derive_key_batch`

```rust
pub fn derive_key_batch(
    domains: &[&[u8]],
    materials: &[&[u8]],
    output_len: usize,
) -> Vec<Vec<u8>>
```

Batch KDF for up to 26 concurrent derivations. Processes `min(domains.len(), materials.len(), 26)` derivations.

**Example:**

```rust
let domains: Vec<Vec<u8>> = (0..26).map(|i| format!("D{i}").into_bytes()).collect();
let materials: Vec<Vec<u8>> = (0..26).map(|i| format!("M{i}").into_bytes()).collect();
let dom_refs: Vec<&[u8]> = domains.iter().map(|d| d.as_slice()).collect();
let mat_refs: Vec<&[u8]> = materials.iter().map(|m| m.as_slice()).collect();
let keys = sponge::derive_key_batch(&dom_refs, &mat_refs, 32);
assert_eq!(keys.len(), 26);
// Each key matches its scalar equivalent:
assert_eq!(keys[0], sponge::derive_key(&domains[0], &materials[0], 32));
```

### 1.5 Public Functions — Bulk Rate (486 trits)

Higher throughput (~97 bytes/permutation) at reduced capacity (243 trits).

```rust
pub fn hash_bulk(input: &[u8], output_len: usize) -> Vec<u8>
pub fn derive_key_bulk(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8>
```

> **Note:** `hash_bulk` produces different output than `hash` for the same input — the rate is different.

---

## 2. TIS-27

**Module**: `ternary_math::tlsponge385` (same module, TIS-27 variant)  
**Security**: 43-bit (integrity checks, not cryptographic strength)  
**Rounds**: 4  
**State**: 729 balanced trits (same permutation, fewer rounds)  
**Use cases**: Wire packet integrity, TDNS identity derivation, scan hashing, heartbeat HMAC

### 2.1 Public Functions

All TIS-27 functions mirror their TL-Sponge-385 counterparts but use 4 rounds instead of 9.

```rust
pub fn hash_hex_tis(input: &[u8]) -> String
pub fn derive_key_tis(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8>
pub fn derive_key_cat_tis(context: &[u8], parts: &[&[u8]], key_len: usize) -> Vec<u8>
pub fn hash_bulk_tis(input: &[u8], output_len: usize) -> Vec<u8>
pub fn derive_key_bulk_tis(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8>
pub fn derive_key_batch_tis(domains: &[&[u8]], materials: &[&[u8]], output_len: usize) -> Vec<Vec<u8>>
```

**Example:**

```rust
let integrity_tag = sponge::derive_key_tis(b"wire-check", &packet_bytes, 27);
// TIS-27 output differs from TL-Sponge-385:
assert_ne!(
    sponge::derive_key(b"T", b"m", 32),
    sponge::derive_key_tis(b"T", b"m", 32),
);
```

### 2.2 Instance Construction

```rust
let tis_sponge = Sponge385Pub::new_tis();
```

---

## 3. TL-DSA

**Module**: `ternary_math::tl_dsa`  
**Construction**: WOTS+ (Winternitz One-Time Signature Plus) over TL-Sponge-385  
**Security**: Post-quantum, reduces to collision resistance of TL-Sponge-385  
**Constraint**: **One-time** — each keypair signs at most one message

### 3.1 Variants

| Variant     | Security | PK (bytes) | SK (bytes) | Sig (bytes) | Chains | Winternitz w |
|-------------|----------|-----------|-----------|------------|--------|-------------|
| `TlDsa44`   | Level 2  | 32        | 64        | 1,632      | 51     | 16          |
| `TlDsa65`   | Level 3  | 48        | 96        | 2,144      | 67     | 16          |
| `TlDsa87`   | Level 5  | 64        | 128       | 3,168      | 99     | 16          |

### 3.2 Types

```rust
#[repr(u8)]
pub enum TlDsaVariant {
    TlDsa44 = 44,
    TlDsa65 = 65,
    TlDsa87 = 87,
}

pub struct TlDsaKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub variant: TlDsaVariant,
}

pub struct TlDsaParams {
    pub pk_len: usize,
    pub sk_len: usize,
    pub msg_hash_len: usize,
    pub chains: usize,
    pub chain_depth: u8,
    pub sig_len: usize,
    pub variant_tag: &'static [u8],
}
```

### 3.3 Functions

#### `keygen`

```rust
pub fn keygen(variant: TlDsaVariant, seed: Option<&[u8]>) -> TlDsaKeyPair
```

Generate a TL-DSA keypair.

| Parameter | Description |
|-----------|-------------|
| `variant` | Security level (`TlDsa44`, `TlDsa65`, `TlDsa87`) |
| `seed`    | Optional seed bytes. `None` uses 64 zero bytes (testing only). Production: ≥256 bits entropy. |

**Example:**

```rust
use ternary_math::tl_dsa::{keygen, TlDsaVariant};

let kp = keygen(TlDsaVariant::TlDsa87, Some(&random_seed));
assert_eq!(kp.public_key.len(), 64);
assert_eq!(kp.secret_key.len(), 128);
```

#### `sign`

```rust
pub fn sign(secret_key: &[u8], message: &[u8], variant: TlDsaVariant) -> Vec<u8>
```

Deterministic signature. Same (sk, msg) always produces the same signature.

**Example:**

```rust
let sig = tl_dsa::sign(&kp.secret_key, b"registration payload", TlDsaVariant::TlDsa87);
assert_eq!(sig.len(), 3168);
```

#### `verify`

```rust
pub fn verify(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
    variant: TlDsaVariant,
) -> bool
```

Verify a TL-DSA signature. Returns `false` if signature or public key length is wrong.

**Example:**

```rust
let valid = tl_dsa::verify(&kp.public_key, b"registration payload", &sig, TlDsaVariant::TlDsa87);
assert!(valid);

let tampered = tl_dsa::verify(&kp.public_key, b"wrong message", &sig, TlDsaVariant::TlDsa87);
assert!(!tampered);
```

#### `TlDsaVariant::from_u32`

```rust
pub fn from_u32(v: u32) -> Option<TlDsaVariant>
```

Parse variant from integer code (44, 65, or 87).

---

## 4. TL-KEM

**Module**: `ternary_math::tl_kem`  
**Construction**: Module-LWE over R_q = Z_3[X]/(X²⁵⁶+1) with Fujisaki-Okamoto transform  
**Security**: IND-CCA2  
**Hash primitive**: TL-Sponge-385  
**NTT**: Negacyclic integer NTT (q=12289, n=256)

### 4.1 Variants

| Variant        | Module Rank (k) | NIST Level | Security (bits) | ML-KEM Equivalent |
|---------------|----------------|-----------|----------------|-------------------|
| `TlKem512`    | 2              | Level 1   | 128            | ML-KEM-512        |
| `TlKem768`    | 3              | Level 3   | 192            | ML-KEM-768        |
| `TlKem1024`   | 4              | Level 5   | 256            | ML-KEM-1024       |

### 4.2 Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlKemVariant {
    TlKem512,
    TlKem768,
    TlKem1024,
}

pub struct TlKemPublicKey {
    pub variant: TlKemVariant,
    pub matrix_a_seed: Vec<i8>,
    pub public_vec_t: TernaryPolyVec,
}

pub struct TlKemSecretKey {
    pub variant: TlKemVariant,
    pub secret_s: TernaryPolyVec,
    pub public_key: TlKemPublicKey,
    pub hash_pk: Vec<i8>,
    pub implicit_reject_seed: Vec<i8>,
}

pub struct TlKemCiphertext {
    pub variant: TlKemVariant,
    pub compressed_u: Vec<Vec<u8>>,
    pub compressed_v: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSecret {
    pub trits: Vec<i8>,
}

#[derive(Debug)]
pub enum TlKemError {
    Lattice(LatticeError),
    InvalidSeed,
    InvalidFormat,
    VariantMismatch,
}
```

### 4.3 Functions

#### `keygen`

```rust
pub fn keygen(variant: TlKemVariant) -> Result<(TlKemPublicKey, TlKemSecretKey), TlKemError>
```

Generate a KEM keypair using OS randomness.

**Example:**

```rust
use ternary_math::tl_kem::{keygen, TlKemVariant};

let (pk, sk) = keygen(TlKemVariant::TlKem768)?;
```

#### `encapsulate`

```rust
pub fn encapsulate(pk: &TlKemPublicKey) -> Result<(TlKemCiphertext, SharedSecret), TlKemError>
```

Encapsulate a shared secret under the public key. Produces a ciphertext and the shared secret.

**Example:**

```rust
let (ct, shared_secret) = tl_kem::encapsulate(&pk)?;
```

#### `decapsulate`

```rust
pub fn decapsulate(
    ct: &TlKemCiphertext,
    sk: &TlKemSecretKey,
) -> Result<SharedSecret, TlKemError>
```

Decapsulate to recover the shared secret. Uses implicit rejection (returns a pseudorandom secret on failure rather than an error, per FO transform).

**Errors:**

| Variant | Cause |
|---------|-------|
| `VariantMismatch` | Ciphertext variant ≠ secret key variant |

**Example:**

```rust
let recovered = tl_kem::decapsulate(&ct, &sk)?;
assert_eq!(recovered, shared_secret);
```

#### `SharedSecret::to_bytes_32`

```rust
impl SharedSecret {
    pub fn to_bytes_32(&self) -> [u8; 32]
}
```

Derive a 32-byte binary shared secret via `TLSponge-385("TL-KEM-SharedSecret-v1" ‖ trit_bytes)`. Compatible with `sponge385_derive_key`'s `kem_shared_secret: &[u8; 32]` parameter.

**Example:**

```rust
let kem_bytes: [u8; 32] = shared_secret.to_bytes_32();

// Use with CON tunnel key derivation:
let tunnel_key = sponge::sponge385_derive_key(
    b"PlenumNET-CON-v3.0",
    &addr_a.to_bytes(),
    &addr_b.to_bytes(),
    &kem_bytes,
    epoch,
);
```

### 4.4 Serialization

All key and ciphertext types support round-trip serialization:

```rust
impl TlKemPublicKey {
    pub fn to_bytes(&self) -> Vec<u8>;
    pub fn from_bytes(data: &[u8]) -> Result<Self, TlKemError>;
}

impl TlKemSecretKey {
    pub fn to_bytes(&self) -> Vec<u8>;
    pub fn from_bytes(data: &[u8]) -> Result<Self, TlKemError>;
}

impl TlKemCiphertext {
    pub fn to_bytes(&self) -> Vec<u8>;
    pub fn from_bytes(data: &[u8]) -> Result<Self, TlKemError>;
}
```

Wire format: `tag_byte ‖ length-prefixed fields`. Tag bytes: `0x01` = TlKem512, `0x02` = TlKem768, `0x03` = TlKem1024.

### 4.5 Error Types

| Variant | Description |
|---------|-------------|
| `Lattice(LatticeError)` | Underlying lattice arithmetic error |
| `InvalidSeed` | OS randomness generation failed |
| `InvalidFormat` | Deserialization of key/ciphertext failed |
| `VariantMismatch` | Ciphertext and secret key have different variants |

### 4.6 Full KEM → Tunnel Key Example

```rust
use ternary_math::{tl_kem, sponge};

// Node A: generate keypair
let (pk_a, sk_a) = tl_kem::keygen(tl_kem::TlKemVariant::TlKem768)?;

// Node B: encapsulate
let (ct, ss_b) = tl_kem::encapsulate(&pk_a)?;

// Node A: decapsulate
let ss_a = tl_kem::decapsulate(&ct, &sk_a)?;
assert_eq!(ss_a, ss_b);

// Both nodes derive the same tunnel key:
let kem_bytes = ss_a.to_bytes_32();
let tunnel_key = sponge::sponge385_derive_key(
    b"PlenumNET-CON-v3.0",
    &addr_a.to_bytes(),
    &addr_b.to_bytes(),
    &kem_bytes,
    epoch,
);
```

---

## 5. Phase Encryption v3

**Module**: `ternary_math::phase_encryption`  
**Construction**: Duplex-mode TL-Sponge-385-based GF(3) stream cipher  
**Security**: IND-CPA with unified MAC (binds both phase halves, authenticates headers)  
**Domain separator**: `b"PlenumNET-Phase-v2"` (PHASE_CONTEXT_TAG)

### 5.1 Architecture

1. Derive 32-byte key material via TL-Sponge-385
2. Generate 32-byte random nonce per operation
3. Build domain: `key_material ‖ nonce ‖ phase_angle_364 ‖ context_tag`
4. Duplex sponge: absorb domain → squeeze primary keystream → absorb phase switch → squeeze secondary keystream → absorb both ciphertexts → squeeze MAC
5. Encrypt: `ciphertext[i] = tritAdd(plaintext[i], keystream[i])` (GF(3))
6. Decrypt: reverse with `tritSub`

### 5.2 Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    HighSecurity,  // secondary_offset=10, guardian=true  (τ=358°)
    Balanced,      // secondary_offset=4,  guardian=false
    Performance,   // secondary_offset=1,  guardian=false
    Adaptive,      // secondary_offset=4,  guardian=true  (τ=358°)
}

pub struct PhaseConfig {
    pub mode: EncryptionMode,
    pub primary_phase: u16,       // Always 0
    pub secondary_offset: u16,    // Mode-dependent
    pub guardian_enabled: bool,    // true for HighSecurity, Adaptive
    pub guardian_offset: u16,     // 358 when enabled
}

pub struct PhaseCiphertext {
    pub primary_cipher: Vec<u8>,
    pub secondary_cipher: Vec<u8>,
    pub mac: String,
    pub nonce: Vec<u8>,           // 32 bytes
    pub config: PhaseConfig,
    pub guardian_hash: Option<String>,
    pub version: u8,              // 3
    pub sponge_version: u8,       // 2
}
```

### 5.3 Wire Format

```rust
pub struct TsWireFormat {
    pub primary_phase: TsPhaseEntry,
    pub secondary_phase: TsPhaseEntry,
    pub guardian_phase: Option<TsGuardianEntry>,
    pub config: PhaseConfig,
    pub split_ratio: f64,
    pub nonce: Option<String>,        // hex-encoded
    pub mac: Option<String>,          // hex-encoded
    pub version: Option<u8>,
    pub sponge_version: Option<u8>,
}

pub struct TsPhaseEntry {
    pub data: String,  // base64-encoded ciphertext
    pub phase: u16,    // ternary degree (0–363)
}

pub struct TsGuardianEntry {
    pub hash: String,  // TL-Sponge-385 tamper detection hash
    pub phase: u16,    // guardian phase angle (typically 358)
}
```

### 5.4 Functions

#### `encrypt`

```rust
pub fn encrypt(
    plaintext: &[u8],
    key: &[u8; 32],
    mode: EncryptionMode,
) -> Result<PhaseCiphertext, PhaseError>
```

Encrypt plaintext with a fresh random nonce.

**Example:**

```rust
use ternary_math::phase_encryption::{encrypt, decrypt, EncryptionMode, derive_key_from_secret};

let key = derive_key_from_secret(b"my-secret-passphrase");
let ct = encrypt(b"sensitive data", &key, EncryptionMode::HighSecurity)?;
```

#### `decrypt`

```rust
pub fn decrypt(
    ciphertext: &PhaseCiphertext,
    key: &[u8; 32],
    mode: EncryptionMode,
) -> Result<Vec<u8>, PhaseError>
```

Decrypt and verify MAC. Checks guardian hash when `guardian_enabled`.

**Example:**

```rust
let plaintext = decrypt(&ct, &key, EncryptionMode::HighSecurity)?;
assert_eq!(plaintext, b"sensitive data");
```

#### `decrypt_implicit`

```rust
pub fn decrypt_implicit(
    ciphertext: &PhaseCiphertext,
    key: &[u8; 32],
) -> Result<Vec<u8>, PhaseError>
```

Decrypt using the mode stored in the ciphertext's `config` field.

#### `derive_key_from_secret`

```rust
pub fn derive_key_from_secret(secret: &[u8]) -> [u8; 32]
```

Derive a 32-byte encryption key from an arbitrary-length secret. Computes `hash_hex(secret ‖ "PlenumNET-Phase-KeyDerive")` and takes the first 32 bytes of the hex-decoded output.

#### `derive_key_for_version`

```rust
pub fn derive_key_for_version(secret: &[u8], sponge_version: u8) -> [u8; 32]
```

Version-aware key derivation. Computes `hash_hex(secret ‖ "PlenumNET-Phase-KeyDerive")` where `sponge_version >= 2` uses the chi-enabled sponge (v2) and `< 2` uses v1 (no chi, legacy).

#### `derive_key_from_kem_secret`

```rust
pub fn derive_key_from_kem_secret(kem_shared: &[u8; 32]) -> [u8; 32]
```

Derive encryption key from a TL-KEM shared secret. Computes `hash_hex("PlenumNET-Phase-KEM-KeyDerive" ‖ kem_shared)` and takes the first 32 bytes.

#### Wire Format Conversion

```rust
impl PhaseCiphertext {
    pub fn to_ts_wire_format(&self) -> TsWireFormat;
    pub fn from_ts_wire_format(wire: &TsWireFormat) -> Result<Self, PhaseError>;
    pub fn primary_cipher_b64(&self) -> String;
    pub fn secondary_cipher_b64(&self) -> String;
    pub fn nonce_hex(&self) -> String;
}

pub fn get_phase_config(mode: EncryptionMode) -> PhaseConfig;
```

### 5.5 Error Types

| Variant | Description |
|---------|-------------|
| `MacMismatch` | Unified MAC verification failed — ciphertext tampered |
| `GuardianFailed` | Guardian phase τ-derived tamper detection failed |
| `InvalidCiphertext` | Malformed ciphertext (bad base64, wrong nonce length, etc.) |
| `RandomnessError` | OS CSPRNG failed (via `getrandom`) |
| `UnsupportedVersion(v, sv)` | Ciphertext version must be 3; sponge version must be 1, 2, or 3 (rejects 0 and >3) |
| `MissingGuardian` | Guardian hash required for this mode but absent |

### 5.6 Full Encrypt → Wire → Decrypt Example

```rust
use ternary_math::phase_encryption::*;

// Sender
let key = derive_key_from_secret(b"shared-secret");
let ct = encrypt(b"payload", &key, EncryptionMode::Adaptive)?;
let wire = ct.to_ts_wire_format();  // serialize for transmission

// Receiver
let ct2 = PhaseCiphertext::from_ts_wire_format(&wire)?;
let plaintext = decrypt(&ct2, &key, EncryptionMode::Adaptive)?;
assert_eq!(plaintext, b"payload");
```

---

## 6. Inter-Cube Service APIs

**Crate**: `inter-cube` (path: `services/inter-cube/`)  
**Dependency**: `ternary-math = { path = "../../ternary-math" }`

The Inter-Cube infrastructure consists of four services that provide geometric routing across the 13D ternary cube network. Each populated cube contains `26 × 3¹³ / 2 = 20,726,199` unique PQ-encrypted tunnels.

### 6.1 CRS — Cube Registration Service

**Module**: `inter_cube::crs`  
**Purpose**: Manages cube network membership, address allocation, and signed registration

#### Types

```rust
pub enum CubeStatus { Active, Draining, Offline }

pub struct CubeRecord {
    pub addr: CubeAddr,
    pub endpoints: Vec<SocketAddr>,
    pub public_key: Vec<u8>,          // TL-DSA-87 identity key
    pub kem_public_key: Option<Vec<u8>>,
    pub status: CubeStatus,
    pub last_heartbeat: Instant,
    pub registered_at: Instant,
    pub registered_at_fs: u128,       // femtoseconds since Salvi Epoch
    pub reg_signature: Option<Vec<u8>>,
    pub legacy_key: bool,
    pub level: usize,
}

pub struct SignedRegistration {
    pub address: Option<CubeAddr>,    // None = auto-allocate
    pub endpoint: SocketAddr,
    pub public_key: Vec<u8>,          // TL-DSA-87 PK (64 bytes)
    pub kem_public_key: Option<Vec<u8>>,
    pub timestamp_fs: u128,
    pub signature: Vec<u8>,           // TL-DSA-87 signature
}

pub struct RegistrationResult {
    pub address: CubeAddr,
    pub neighbors: Vec<NeighborInfo>,
}

pub enum RegistrationError {
    AddressSpaceExhausted,
    AddressInUse,
    InvalidAddress,
    MissingPublicKey,
    InvalidSignature,
    StaleTimestamp,
    ReplayDetected,
    SignatureRequired,
}
```

**Constants:**

```rust
pub const CRS_REG_DOMAIN: &[u8] = b"PlenumNET-CRS-REG-v1";
pub const CRS_SIG_VARIANT: u8 = 87;  // TL-DSA-87
```

#### `CubeRegistrationService::register`

```rust
pub fn register(
    &mut self,
    endpoint: SocketAddr,
    public_key: Vec<u8>,
    desired_address: Option<CubeAddr>,
) -> Result<RegistrationResult, RegistrationError>
```

Legacy unsigned registration. Allocates a Rep C address and returns the 26 geometric neighbors.

#### `CubeRegistrationService::register_signed`

```rust
pub fn register_signed(
    &mut self,
    reg: &SignedRegistration,
    now_fs: u128,
) -> Result<RegistrationResult, RegistrationError>
```

T-06 signed registration. Verifies TL-DSA-87 signature over canonical message, enforces timestamp replay window (30s max age, 1s future tolerance).

**Canonical message format:**

```
CRS_REG_DOMAIN ‖ address.to_wire() ‖ endpoint ‖ public_key ‖ kem_public_key ‖ timestamp_le
```

**Example:**

```rust
use inter_cube::crs::{CubeRegistrationService, SignedRegistration};

let mut crs = CubeRegistrationService::new();
let reg = SignedRegistration {
    address: None,
    endpoint: "192.168.1.1:8080".parse().unwrap(),
    public_key: kp.public_key.clone(),
    kem_public_key: Some(kem_pk.to_bytes()),
    timestamp_fs: current_hptp_timestamp(),
    signature: tl_dsa::sign(&kp.secret_key, &canonical_msg, TlDsaVariant::TlDsa87),
};
let result = crs.register_signed(&reg, current_hptp_timestamp())?;
println!("Assigned: {:?}, neighbors: {}", result.address, result.neighbors.len());
```

#### `CubeRegistrationService::heartbeat`

```rust
pub fn heartbeat(&mut self, addr: &CubeAddr, endpoint: SocketAddr) -> bool
```

Record a heartbeat from a registered cube. Returns `true` if the cube was found. Updates the last-heartbeat timestamp and adds new endpoints (keeps most recent 3).

### 6.2 CON — Cube Overlay Network

**Module**: `inter_cube::overlay`  
**Purpose**: Manages PQ-encrypted tunnels to 26 geometric neighbors

#### Types

```rust
pub struct ForgeryAlert {
    pub neighbor_addr: CubeAddr,
    pub claimed_endpoint: SocketAddr,
    pub timestamp: Instant,
    pub reason: ForgeryReason,
}

pub enum ForgeryReason {
    SignatureInvalid,
    SignatureMissing,
    PublicKeyMissing,
}
```

#### `CubeOverlayNetwork::resolve_neighbor`

```rust
pub fn resolve_neighbor(
    &mut self,
    addr: &CubeAddr,
    endpoint: SocketAddr,
    public_key: Vec<u8>,
) -> bool
```

Legacy neighbor resolution (no signature verification). Returns `true` if the address is one of the 26 geometric neighbors.

#### `CubeOverlayNetwork::resolve_neighbor_verified`

```rust
pub fn resolve_neighbor_verified(
    &mut self,
    addr: &CubeAddr,
    endpoint: SocketAddr,
    public_key: Vec<u8>,
    reg_signature: Option<Vec<u8>>,
    kem_public_key: Option<Vec<u8>>,
    registered_at_fs: u128,
) -> Result<bool, ForgeryAlert>
```

T-07 verified neighbor resolution. Verifies the CRS registration signature before establishing the tunnel. Emits a `ForgeryAlert` on verification failure (signature invalid, missing, or public key missing).

#### `CubeOverlayNetwork::derive_all_keys`

```rust
pub fn derive_all_keys(
    &self,
    kem_secrets: &HashMap<CubeAddr, [u8; 32]>,
    epoch: u64,
) -> Vec<(CubeAddr, [u8; 32])>
```

Derive PQ tunnel keys for all resolved neighbors using:

```
key = TLSponge-385("PlenumNET-CON-v3.0" ‖ canonical(addr_a, addr_b) ‖ kem_shared_secret ‖ epoch)
```

#### `CubeOverlayNetwork::forgery_alerts`

```rust
pub fn forgery_alerts(&self) -> &[ForgeryAlert]
pub fn drain_forgery_alerts(&mut self) -> Vec<ForgeryAlert>
```

### 6.3 GLB — Geometric Load Balancer

**Module**: `inter_cube::glb`  
**Purpose**: Distributes inter-cube traffic across geometrically equivalent shortest paths  
**Design**: No routing tables — geometry IS the routing protocol

#### Types

```rust
pub struct ForwardResult {
    pub next_hop: CubeAddr,
    pub dimension_fixed: usize,
    pub total_distance: usize,
    pub available_paths: usize,
    pub is_detour: bool,
}

pub enum ForwardError {
    AlreadyAtDestination,
    Isolated,  // All 26 neighbors down
}

pub struct GlbStats {
    pub active_flows: u64,
    pub total_forwards: u64,
    pub detours_computed: u64,
    pub flows_expired: u64,
    pub flows_rehashed: u64,
}
```

#### `GeometricLoadBalancer::forward`

```rust
pub fn forward(
    &mut self,
    destination: &CubeAddr,
    flow_id: u64,
) -> Result<ForwardResult, ForwardError>
```

Compute next hop with flow affinity tracking. The algorithm:

1. Compute delta dimensions (where src ≠ dst)
2. Filter out dimensions whose next-hop is in the dead set
3. Select dimension via consistent hash of `flow_id` (TIS-27)
4. Compute next hop by changing one trit toward destination
5. Cache flow affinity (TTL: 60s default)

**Example:**

```rust
let local = CubeAddr::new([1,1,1,1,1,1,1,1,1,1,1,1,1]);
let dest  = CubeAddr::new([3,2,3,1,1,1,1,1,1,1,1,1,1]);
let mut glb = GeometricLoadBalancer::new(local);

let result = glb.forward(&dest, 42)?;
assert_eq!(result.total_distance, 3);
assert!(!result.is_detour);
// Same flow_id always produces the same next hop:
let r2 = glb.forward(&dest, 42)?;
assert_eq!(result.next_hop, r2.next_hop);
```

#### `GeometricLoadBalancer::forward_stateless`

```rust
pub fn forward_stateless(
    &self,
    destination: &CubeAddr,
    flow_id: u64,
) -> Result<ForwardResult, ForwardError>
```

One-off forwarding without flow affinity tracking. Same algorithm but `&self` (no mutation).

#### Dead Neighbor Management

```rust
pub fn set_dead_neighbors(&mut self, dead: HashSet<CubeAddr>);
pub fn add_dead_neighbor(&mut self, addr: CubeAddr);
pub fn remove_dead_neighbor(&mut self, addr: &CubeAddr);
pub fn dead_neighbors(&self) -> &HashSet<CubeAddr>;
```

#### Flow Management

```rust
pub fn expire_flows(&mut self);
pub fn active_flow_count(&self) -> usize;
pub fn stats(&self) -> &GlbStats;
pub fn live_neighbor_count(&self) -> usize;  // 26 - |dead_set|
```

#### Path Enumeration

```rust
pub fn enumerate_paths(&self, destination: &CubeAddr) -> Vec<Vec<usize>>
```

Enumerate all shortest paths (Heap's algorithm). For Hamming distance d ≤ 8, returns all d! permutations. For d > 8, returns only the dimension-order path.

### 6.4 FTS — Fault Tolerance Service

**Module**: `inter_cube::fts`  
**Purpose**: Monitors 26 geometric neighbors via heartbeat ping/pong, publishes dead set to GLB

#### Neighbor State Machine

```
Up ──(3 missed pings)──→ Suspect ──(5s grace)──→ Down
 ↑                                                  │
 └───────(5 consecutive successes)──── Recovering ←─┘
```

#### Types

```rust
pub enum NeighborState { Up, Suspect, Down, Recovering }

pub enum HeartbeatAuth {
    TisHmac  = 0x01,  // TIS-27 HMAC — sub-microsecond
    TlDsaSig = 0x02,  // TL-DSA-87 full signature — non-repudiable
}

pub struct AuthenticatedHeartbeat {
    pub address: CubeAddr,
    pub endpoint: String,
    pub sequence: u64,           // monotonically increasing
    pub timestamp_fs: u128,      // femtoseconds since Salvi Epoch
    pub auth_mode: HeartbeatAuth,
    pub auth_data: Vec<u8>,      // HMAC tag (27 bytes) or TL-DSA signature
}

pub enum HeartbeatAuthError {
    HmacInvalid,
    SignatureInvalid,
    SequenceReplay { received: u64, last_accepted: u64 },
    UnknownAuthMode(u8),
    UnknownAddress,
}

pub struct FtsConfig {
    pub ping_interval: Duration,      // default: 1s
    pub miss_threshold: u8,           // default: 3
    pub recovery_threshold: u8,       // default: 5
    pub grace_period: Duration,       // default: 5s
    pub auth_failure_threshold: u8,   // default: 3
}

pub struct StateChangeEvent {
    pub addr: CubeAddr,
    pub from: NeighborState,
    pub to: NeighborState,
    pub timestamp: Instant,
}
```

**Constants:**

```rust
pub const HB_HMAC_DOMAIN: &[u8]     = b"PlenumNET-HB-HMAC";
pub const HB_HMAC_TAG_DOMAIN: &[u8] = b"PlenumNET-HB-TAG";
pub const HB_HMAC_KEY_LEN: usize    = 48;   // 384 bits
pub const HB_HMAC_TAG_LEN: usize    = 27;   // squeezed from TIS-27
```

#### Heartbeat HMAC Functions (T-08)

```rust
pub fn derive_hb_hmac_key(address: &CubeAddr, master_secret: &[u8]) -> Vec<u8>
```

Derive a 48-byte HMAC key: `TLSponge-385("PlenumNET-HB-HMAC" ‖ address_bytes ‖ master_secret)`. Derived independently by both CRS and node — never transmitted.

```rust
pub fn compute_hb_hmac(hmac_key: &[u8], message: &[u8]) -> Vec<u8>
```

Compute a 27-byte HMAC tag: `TLSponge-385-derive_key("PlenumNET-HB-TAG", hmac_key ‖ message, 27)`. Uses the full 9-round sponge.

```rust
pub fn verify_hb_hmac(hmac_key: &[u8], message: &[u8], received_tag: &[u8]) -> bool
```

Constant-time HMAC verification.

**Example:**

```rust
use inter_cube::fts::{derive_hb_hmac_key, compute_hb_hmac, verify_hb_hmac};

let hmac_key = derive_hb_hmac_key(&cube_addr, &master_secret);
let msg = heartbeat.canonical_message();
let tag = compute_hb_hmac(&hmac_key, &msg);
assert!(verify_hb_hmac(&hmac_key, &msg, &tag));
```

#### `FaultToleranceService::record_pong`

```rust
pub fn record_pong(&mut self, addr: &CubeAddr, rtt_ns: u64)
```

Record a successful pong (legacy, unauthenticated). Updates SRTT/jitter via exponential moving averages (SRTT: 7/8 old + 1/8 new, jitter: 3/4 old + 1/4 diff).

#### HMAC Key Management (T-08)

```rust
pub fn set_hmac_key(&mut self, addr: &CubeAddr, key: Vec<u8>);
pub fn derive_all_hmac_keys(&mut self, master_secret: &[u8]);
pub fn invalidate_hmac_keys(&mut self);  // called on master_secret rotation
```

#### State Queries

```rust
pub fn state_counts(&self) -> (usize, usize, usize, usize)  // (up, suspect, down, recovering)
pub fn dead_set(&self) -> &HashSet<CubeAddr>
pub fn dead_set_cloned(&self) -> HashSet<CubeAddr>  // for passing to GLB
pub fn has_pending_events(&self) -> bool
pub fn drain_events(&mut self) -> Vec<StateChangeEvent>
```

---

## Appendix A: Domain Separator Registry

| Domain Separator | Used By | Purpose |
|-----------------|---------|---------|
| `PlenumNET-CON-v3.0` | CON `derive_all_keys` | PQ tunnel key derivation |
| `PlenumNET-CRS-REG-v1` | CRS `register_signed` | Registration signature domain |
| `PlenumNET-HB-HMAC` | FTS `derive_hb_hmac_key` | Heartbeat HMAC key derivation |
| `PlenumNET-HB-TAG` | FTS `compute_hb_hmac` | Heartbeat HMAC tag computation |
| `PlenumNET-Phase-v2` | Phase Encryption | Phase cipher domain (PHASE_CONTEXT_TAG) |
| `PlenumNET-Phase-KeyDerive` | `derive_key_from_secret` | Phase key from passphrase (appended to secret) |
| `PlenumNET-Phase-KEM-KeyDerive` | `derive_key_from_kem_secret` | Phase key from KEM shared secret (prepended to kem_shared) |
| `TL-KEM-SharedSecret-v1` | `SharedSecret::to_bytes_32` | KEM shared secret to 32-byte key |
| `TL-DSA-SK-EXPAND` | `keygen` | Expand seed to secret key material |
| `TL-DSA-WOTS-STEP` | chain step | WOTS+ chain domain separation |
| `TL-DSA-CHAIN-SK` | chain bottom | Derive chain bottom from seed |
| `TL-DSA-MSG` | message hashing | Domain-separated message hash |
| `TL-DSA-44` / `TL-DSA-65` / `TL-DSA-87` | variant tags | Per-variant domain separation |

## Appendix B: Crate Dependency Graph

```
ternary-math
 ├── tlsponge385    (TL-Sponge-385, TIS-27)
 ├── tl_dsa         (uses tlsponge385::derive_key)
 ├── tl_kem         (uses tlsponge385::Sponge385Pub, ternary_lattice)
 └── phase_encryption (uses tlsponge385::Sponge385Pub, hash_hex)

inter-cube
 ├── crs       (uses ternary_math::tl_dsa::verify, wire protocol)
 ├── overlay   (uses ternary_math::tl_dsa::verify, sponge385_derive_key)
 ├── glb       (uses ternary_math::sponge::hash for flow hashing)
 └── fts       (uses ternary_math::sponge::{derive_key, derive_key_tis})
```

## Appendix C: Security Level Summary

| Primitive | PQ Security (bits) | NIST Level | Use Case |
|-----------|-------------------|-----------|----------|
| TL-Sponge-385 | 385 | — | Hash, KDF, MAC foundation |
| TIS-27 | 43 | — | Wire integrity, fast HMAC |
| TL-DSA-44 | ~128 | Level 2 | Lightweight signatures |
| TL-DSA-65 | ~192 | Level 3 | Standard signatures |
| TL-DSA-87 | ~256 | Level 5 | CRS registrations, high-assurance |
| TL-KEM-512 | 128 | Level 1 | Standard key exchange |
| TL-KEM-768 | 192 | Level 3 | Recommended key exchange |
| TL-KEM-1024 | 256 | Level 5 | Maximum security key exchange |
| Phase Encryption v3 | 385 | — | Data-at-rest / data-in-transit |
