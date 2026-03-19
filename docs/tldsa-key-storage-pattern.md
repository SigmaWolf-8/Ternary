# TL-DSA v2 Key Serialization & Encrypted Storage Pattern

Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
Patent(s) Pending — All Rights Reserved — Applied Physics Division

Reference: TM-2026-020.1-PREREQ §6.1

## 1. Overview

This document describes the canonical pattern for serializing TL-DSA v2
(lattice-based, Fiat-Shamir with Aborts) keypairs, encrypting the secret key
with Phase Encryption, and storing the result in PostgreSQL. It covers per-project
YODA key isolation and a chain-linked key rotation procedure.

**Scope:** YODA per-project keypairs only. PlenumNET system keys
(`server/crypto/tsa-keys/`) are unaffected and remain disk-based.

## 2. TL-DSA v2 Key Serialization

### 2.1 Wire Format

The serialization is implemented in `src/kernel/src/crypto/tl_dsa.rs`.

`matrix_a` and `matrix_a_ntt` are deterministically re-derived from
`matrix_a_seed` on deserialization, keeping wire size compact.

#### PublicKey Wire

```
variant_tag   (1 byte)
matrix_a_seed (243 i8 trits)
public_t      (k × n i8 trits)
```

#### SecretKey Wire

```
variant_tag   (1 byte)
matrix_a_seed (243 i8 trits)
secret_s1     (l × n i8 trits)
secret_s2     (k × n i8 trits)
public_t      (k × n i8 trits)
signing_seed  (243 i8 trits)
```

#### Signature Wire

```
variant_tag    (1 byte)
z              (l × n i8 trits)
challenge_hash (243 i8 trits)
```

#### Variant Tags

| Variant    | Tag    | k | l | n   | PK wire bytes | SK wire bytes |
|-----------|--------|---|---|-----|---------------|---------------|
| TL-DSA-44 | 0x2C   | 4 | 4 | 256 | 1,268         | 3,559         |
| TL-DSA-65 | 0x41   | 6 | 5 | 256 | 1,780         | 5,095         |
| TL-DSA-87 | 0x57   | 8 | 7 | 256 | 2,292         | 6,631         |

### 2.2 API

```rust
// Public key
let pk_bytes: Vec<u8> = pk.to_bytes();
let pk2 = TlDsaPublicKey::from_bytes(&pk_bytes)?;

// Secret key
let sk_bytes: Vec<u8> = sk.to_bytes();
let sk2 = TlDsaSecretKey::from_bytes(&sk_bytes)?;

// Signature
let sig_bytes: Vec<u8> = sig.to_bytes();
let sig2 = TlDsaSignature::from_bytes(&sig_bytes)?;

// Individual trit accessors
let pk_trits: Vec<i8> = pk.public_key_trits();   // k*n trits
let sk_trits: Vec<i8> = sk.secret_key_trits();   // (l+k)*n trits (s1 ‖ s2)
```

### 2.3 Round-Trip Guarantee

Serialized → deserialized keypairs produce byte-identical signatures:

```rust
let (pk, sk) = keygen(TlDsaVariant::TlDsa87, &seed)?;
let sig_a = sign(&sk, &msg)?;

let sk2 = TlDsaSecretKey::from_bytes(&sk.to_bytes())?;
let sig_b = sign(&sk2, &msg)?;

assert_eq!(sig_a.challenge_hash, sig_b.challenge_hash);
assert_eq!(sig_a.z.trits(), sig_b.z.trits());
```

Verified by tests: `test_serialization_roundtrip_{44,65,87}`.

## 3. Encrypted Storage Pattern

### 3.1 Lifecycle

```
keygen → sk.to_bytes() → Phase Encrypt (high_security) → store in PostgreSQL
                                                              ↓
                                                         [retrieve]
                                                              ↓
                         Phase Decrypt → TlDsaSecretKey::from_bytes() → sign
```

### 3.2 Step-by-Step Reference

#### Step 1: Generate Keypair

```rust
use plenumnet_kernel::crypto::tl_dsa::{keygen, TlDsaVariant};

let seed = generate_random_seed(243); // ≥256 bits entropy
let (pk, sk) = keygen(TlDsaVariant::TlDsa87, &seed)?;
```

#### Step 2: Serialize

```rust
let pk_bytes = pk.to_bytes();   // public: safe to store in cleartext
let sk_bytes = sk.to_bytes();   // secret: MUST be encrypted before storage
```

#### Step 3: Encrypt Secret Key

```typescript
import { phaseEncrypt } from './server/crypto/phase-encryption';

const encrypted = phaseEncrypt(sk_bytes, passphrase, {
  mode: 'high_security',        // Phase Encryption high-security mode
  spongeVersion: 3,             // TL-Sponge-385 v3 (chi + parallel)
});
```

The passphrase must be derived from the project's master secret or HSM-held
key — never hardcoded. In the YODA context, each project has an isolated
passphrase derived via:

```
passphrase = sponge385_derive_key(
    domain  = b"YODA-PROJECT-KEY-ENC",
    input   = project_id ‖ master_secret,
    out_len = 64
)
```

#### Step 4: Store in PostgreSQL

```sql
INSERT INTO yoda_project_keys (
    project_id,
    key_id,
    variant,
    public_key,
    encrypted_secret_key,
    is_active,
    created_at
) VALUES (
    $1,             -- project UUID
    gen_random_uuid(),
    'TL-DSA-87',
    $2,             -- pk_bytes (BYTEA, cleartext)
    $3,             -- encrypted blob (BYTEA)
    true,
    now()
);
```

**Table note:** The `yoda_project_keys` schema is YODA-side work (out of scope
for this pattern document). The table is separate from PlenumNET system keys.

#### Step 5: Retrieve and Decrypt

```typescript
const row = await db.query(
    'SELECT encrypted_secret_key, public_key, variant FROM yoda_project_keys WHERE project_id = $1 AND is_active = true',
    [projectId]
);

const sk_bytes = phaseDecrypt(row.encrypted_secret_key, passphrase);
```

#### Step 6: Deserialize and Use

```rust
let sk = TlDsaSecretKey::from_bytes(&sk_bytes)?;
let pk = TlDsaPublicKey::from_bytes(&pk_bytes)?;

let sig = sign(&sk, &message)?;
assert!(verify(&pk, &message, &sig)?);
```

### 3.3 Security Properties

| Property | Mechanism |
|----------|-----------|
| Secret key confidentiality at rest | Phase Encryption v3 (TL-Sponge-385, 385-bit PQ security) |
| Variant binding | First byte of wire format locks the variant tag |
| Trit range validation | `from_bytes()` rejects any trit outside {-1, 0, +1} |
| Length validation | `from_bytes()` rejects truncated or oversized input |
| Matrix re-derivation integrity | `matrix_a` re-expanded from seed matches original (deterministic) |
| Namespace isolation | YODA keys use `yoda_project_keys` table, not `tsa-keys/` |

## 4. Key Rotation Procedure

### 4.1 Rotation Trigger

Key rotation occurs when:

1. **Scheduled:** Aligned with the Inter-Cube arc-epoch radian boundary
   (every 14 days = π ternary days). See `services/inter-cube/src/key_rotation.rs`.
2. **Policy-driven:** Project policy requires rotation (e.g., personnel change,
   security incident).
3. **Manual:** Administrator-initiated via YODA dashboard.

### 4.2 Rotation Sequence

```
┌──────────────────────────────────────────────────────────┐
│  1. Generate new keypair                                  │
│     (pk_new, sk_new) = keygen(variant, fresh_seed)       │
│                                                           │
│  2. Create chain link                                     │
│     chain_link = sign(sk_old, pk_new.to_bytes())         │
│     ↳ Binds the new key to the old key's authority        │
│                                                           │
│  3. Archive old key                                       │
│     UPDATE yoda_project_keys                              │
│       SET is_active = false,                              │
│           archived_at = now(),                            │
│           successor_key_id = new_key_id,                  │
│           chain_link_signature = chain_link.to_bytes()    │
│       WHERE key_id = old_key_id;                          │
│                                                           │
│  4. Store new key (encrypted)                             │
│     pk_new.to_bytes()  → cleartext BYTEA                  │
│     Phase Encrypt(sk_new.to_bytes()) → encrypted BYTEA   │
│     INSERT INTO yoda_project_keys (...)                    │
│                                                           │
│  5. Verify chain integrity                                │
│     verify(pk_old, pk_new.to_bytes(), chain_link) == true │
└──────────────────────────────────────────────────────────┘
```

### 4.3 Chain Link Verification

The chain link is a TL-DSA signature by the old secret key over the new
public key's serialized bytes. This provides cryptographic proof that the key
transition was authorized by the holder of the previous key.

```rust
let chain_link = sign(&sk_old, &pk_new.to_bytes())?;

// Anyone with pk_old can verify the transition:
assert!(verify(&pk_old, &pk_new.to_bytes(), &chain_link)?);
```

### 4.4 Historical Signature Verification

Archived public keys are retained indefinitely. When verifying a historical
signature:

1. Look up the `key_id` that produced the signature (stored alongside the
   signed artifact).
2. Retrieve the archived public key (cleartext BYTEA in `yoda_project_keys`).
3. Deserialize: `let pk = TlDsaPublicKey::from_bytes(&archived_pk_bytes)?;`
4. Verify: `verify(&pk, &original_message, &sig)?;`

The secret key for archived entries may be securely deleted after the chain
link is established, since only the public key is needed for historical
verification.

### 4.5 Chain Integrity Audit

To audit the full key chain from the current active key back to the original:

```
active_key ←[chain_link]← key_N ←[chain_link]← ... ←[chain_link]← key_0
```

Walk `successor_key_id` backward and verify each `chain_link_signature`
against the predecessor's `public_key`. If any link fails verification,
the chain is broken and must be investigated.

### 4.6 Dual-Accept Window

During the rotation window (configurable, default 1 second per Inter-Cube
convention), both the old and new keys are accepted for signature verification.
This covers in-flight messages signed under the old key. After the window
closes, only the new key is active.

## 5. Wire Size Reference

| Variant    | PK wire  | SK wire  | Sig wire | SK encrypted (est.) |
|-----------|----------|----------|----------|---------------------|
| TL-DSA-44 | 1,268 B  | 3,559 B  | 1,268 B  | ~3,600 B            |
| TL-DSA-65 | 1,780 B  | 5,095 B  | 1,524 B  | ~5,200 B            |
| TL-DSA-87 | 2,292 B  | 6,631 B  | 2,036 B  | ~6,800 B            |

Encrypted sizes are approximate — Phase Encryption adds a small header
(version, nonce, MAC).

## 6. Implementation Files

| File | What |
|------|------|
| `src/kernel/src/crypto/tl_dsa.rs` | `to_bytes()` / `from_bytes()` for TlDsaPublicKey, TlDsaSecretKey, TlDsaSignature |
| `server/crypto/phase-encryption.ts` | Phase Encrypt / Decrypt (TypeScript) |
| `server/crypto/key-management.ts` | PlenumNET system key storage (disk-based, NOT for YODA) |
| `services/inter-cube/src/key_rotation.rs` | Arc-epoch rotation orchestrator (reference for timing) |
