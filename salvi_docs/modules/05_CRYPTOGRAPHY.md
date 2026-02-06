# Module Guide: Cryptographic Primitives

**Module:** `salvi_kernel::crypto`  
**Status:** Complete (P1-021 to P1-026)  
**Tests:** ~70 tests

---

## Overview

The Cryptographic Primitives module provides post-quantum cryptographic operations built on ternary arithmetic. All algorithms operate natively in GF(3), making them resistant to quantum attacks that target binary-based systems.

### Key Features

- **Ternary Sponge Hash** - Keccak-inspired sponge construction over GF(3)
- **HMAC** - Ternary HMAC for message authentication
- **Key Derivation (KDF)** - Derive multiple keys from a master secret
- **Lamport One-Time Signatures (OTS)** - Quantum-resistant digital signatures
- **Bijective Cipher** - Symmetric encryption using ternary bijective mappings

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│         (TVM Programs, Network Protocols, Storage)          │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │  Sponge  │ │   HMAC   │ │   KDF    │ │  Cipher  │       │
│  │   Hash   │ │          │ │          │ │(Bijective)│       │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘       │
│  ┌──────────┐                                               │
│  │ Lamport  │                                               │
│  │   OTS    │                                               │
│  └──────────┘                                               │
├─────────────────────────────────────────────────────────────┤
│                    GF(3) Arithmetic Layer                    │
│              (Addition, Multiplication, Inverse)            │
└─────────────────────────────────────────────────────────────┘
```

---

## Ternary Sponge Hash

A cryptographic hash function using sponge construction over GF(3):

```rust
use salvi_kernel::crypto::TernarySpongeHash;

// Hash arbitrary data
let data = b"Hello, quantum-resistant world!";
let hash = TernarySpongeHash::hash(data);

println!("Hash: {:?}", hash);  // 27-tryte digest

// Incremental hashing
let mut hasher = TernarySpongeHash::new();
hasher.absorb(b"part 1");
hasher.absorb(b"part 2");
let hash = hasher.squeeze();
```

### Sponge Parameters

| Parameter | Value |
|-----------|-------|
| State width | 243 trits (81 trytes) |
| Rate | 162 trits (54 trytes) |
| Capacity | 81 trits (27 trytes) |
| Output | 243 trits (81 trytes) |
| Rounds | 24 |

---

## HMAC

Message authentication using ternary HMAC:

```rust
use salvi_kernel::crypto::{TernaryHmac, HmacKey};

// Create HMAC key
let key = HmacKey::generate();

// Compute HMAC
let message = b"authenticate this message";
let mac = TernaryHmac::compute(&key, message);

// Verify HMAC
assert!(TernaryHmac::verify(&key, message, &mac));

// Tampered message fails verification
let tampered = b"authenticate this massage";
assert!(!TernaryHmac::verify(&key, tampered, &mac));
```

---

## Key Derivation Function (KDF)

Derive multiple cryptographic keys from a single master secret:

```rust
use salvi_kernel::crypto::{TernaryKdf, KdfParams};

// Create KDF from master key
let master_key = b"my-secret-master-key";
let kdf = TernaryKdf::new(master_key);

// Derive multiple keys for different purposes
let encryption_key = kdf.derive("encryption", 27)?;    // 27 trytes
let signing_key = kdf.derive("signing", 54)?;           // 54 trytes
let auth_key = kdf.derive("authentication", 27)?;       // 27 trytes

// Each derivation produces a unique, cryptographically independent key
assert_ne!(encryption_key, signing_key);
```

### KDF Parameters

```rust
let params = KdfParams {
    iterations: 100_000,     // Work factor
    memory: 64 * 1024,       // Memory usage (64KB)
    parallelism: 4,          // Parallel lanes
};

let kdf = TernaryKdf::with_params(master_key, params);
```

---

## Lamport One-Time Signatures (OTS)

Quantum-resistant digital signatures based on hash chains:

```rust
use salvi_kernel::crypto::{LamportOts, LamportKeyPair};

// Generate key pair
let keypair = LamportKeyPair::generate();

// Sign a message (WARNING: each key can only sign ONE message!)
let message = b"sign this document";
let signature = LamportOts::sign(&keypair.secret_key, message);

// Verify signature
assert!(LamportOts::verify(&keypair.public_key, message, &signature));

// Tampered message fails verification
let tampered = b"sign this dockment";
assert!(!LamportOts::verify(&keypair.public_key, tampered, &signature));
```

### Important: One-Time Usage

```rust
// CRITICAL: Never reuse a Lamport key pair!
// Each secret key must only sign ONE message.

// For multiple signatures, use a key chain:
use salvi_kernel::crypto::LamportKeyChain;

let chain = LamportKeyChain::new(1000);  // 1000 one-time keys

let sig1 = chain.sign(0, message1)?;  // Use key 0
let sig2 = chain.sign(1, message2)?;  // Use key 1
// key 0 is now marked as used and cannot sign again
```

### Signature Sizes

| Component | Size |
|-----------|------|
| Secret key | 2 x 243 x 27 trytes |
| Public key | 2 x 243 x 27 trytes |
| Signature | 243 x 27 trytes |

---

## Bijective Cipher

Symmetric encryption using ternary bijective mappings:

```rust
use salvi_kernel::crypto::{BijectiveCipher, CipherKey};

// Generate encryption key
let key = CipherKey::generate();

// Encrypt
let plaintext = b"secret message";
let ciphertext = BijectiveCipher::encrypt(&key, plaintext)?;

// Decrypt
let decrypted = BijectiveCipher::decrypt(&key, &ciphertext)?;
assert_eq!(plaintext.as_slice(), decrypted.as_slice());
```

### Cipher Properties

| Property | Value |
|----------|-------|
| Key size | 27 trytes (42.8 bits equivalent) |
| Block size | 6 trytes |
| Mode | CTR (Counter) |
| Security | Post-quantum resistant |

### Encryption Modes

```rust
use salvi_kernel::crypto::CipherMode;

// Counter mode (default, parallelizable)
let cipher = BijectiveCipher::new(&key, CipherMode::CTR);

// Cipher block chaining (sequential)
let cipher = BijectiveCipher::new(&key, CipherMode::CBC);

// Galois/Counter mode (authenticated)
let cipher = BijectiveCipher::new(&key, CipherMode::GCM);
let (ciphertext, tag) = cipher.encrypt_authenticated(plaintext, aad)?;
```

---

## Best Practices

### 1. Use HMAC for Message Authentication

```rust
// Always authenticate before decrypting
let (ciphertext, hmac) = encrypt_and_mac(&key, &mac_key, plaintext);

// Verify HMAC first
if !TernaryHmac::verify(&mac_key, &ciphertext, &hmac) {
    return Err(CryptoError::AuthenticationFailed);
}

// Only then decrypt
let plaintext = BijectiveCipher::decrypt(&key, &ciphertext)?;
```

### 2. Never Reuse Lamport Keys

```rust
// Use LamportKeyChain for multiple signatures
let chain = LamportKeyChain::new(num_signatures_needed);
```

### 3. Use KDF for Key Management

```rust
// Derive purpose-specific keys from a master secret
let kdf = TernaryKdf::new(master_secret);
let enc_key = kdf.derive("encrypt", key_length)?;
let mac_key = kdf.derive("mac", key_length)?;
```

---

## Performance Characteristics

| Operation | Time |
|-----------|------|
| Sponge hash (1KB) | ~50 us |
| HMAC compute | ~100 us |
| KDF (default params) | ~500 ms |
| Lamport sign | ~10 ms |
| Lamport verify | ~5 ms |
| Bijective encrypt (1KB) | ~20 us |
| Bijective decrypt (1KB) | ~20 us |

---

## Related Modules

- [Sync Primitives](./02_SYNC_PRIMITIVES.md) - PhaseSafeMutex uses phase encryption
- [Kernel Memory](./01_KERNEL_MEMORY.md) - Secure memory for key storage
- [Torsion Network](./09_TORSION_NETWORK.md) - Network encryption
- [BTG](./13_BTG.md) - Binary-ternary conversion for crypto operations

---

*Part of the Salvi Framework Documentation. Cosi sia.*
