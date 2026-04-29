# Encryption Keys

This directory contains encryption keys for:
- Data-at-rest encryption
- Transport layer security
- Session encryption
- Phase encryption (PQTI-specific)

## Key Format — Trits, Not Bytes

**Salvi Framework rule:** all operator-supplied symmetric / pre-shared key
material is stored and consumed as a **balanced ternary trit string**, not
as hex-encoded bytes. See [`../../../AGENTS.md`](../../../AGENTS.md) for the
full rule and rationale.

| Extension      | Contents                                                      |
| -------------- | ------------------------------------------------------------- |
| `.trits`       | Balanced-ternary symmetric key — one character per trit       |
|                | (`-` = -1, `0` = 0, `+` = +1). Length = 243 (one full sponge   |
|                | rate block). Whitespace is ignored.                            |
| `.pem`         | PEM-encoded asymmetric key material (TL-DSA / TL-KEM)         |
| `.tldsa`       | TL-DSA-87 public / secret key in framework wire format         |

Hex-encoded `.key` files are **not accepted** for symmetric key material.

## Algorithms — Framework Primitives Only

### Symmetric Encryption / Stream Cipher / KDF
- **`crypto::keyed_sponge::KeyedTernarySponge`** — TL-Sponge-385 keyed
  sponge. Absorb the trit key, optionally absorb a per-frame nonce, then
  squeeze keystream trits and XOR (or `trit_add` over GF(3)) into the
  message. This is what RepoSync, the relay channel, and every other
  operator-facing symmetric path uses.

### Hashing / MAC
- **`crypto::sponge::sponge_hash_bytes`** — TL-Sponge-385 cryptographic
  hash with framework-native domain separation.

### Asymmetric / Post-Quantum
- **`crypto::tl_dsa::{sign, verify}`** — TL-DSA-87 lattice signatures.
- **`crypto::tl_kem::{encapsulate, decapsulate}`** — TL-KEM key
  encapsulation at NIST L1 / L3 / L5.

The framework deliberately does **not** depend on `aes-gcm`,
`chacha20-poly1305`, `sha2`, `sha3`, `blake2`, `blake3`, `md5`, `hmac`,
`ring`, or `openssl`. New code that imports any of these will fail the
`scripts/lint-trit-purity.sh` gate at PR time.

## Usage Examples

### Rust — encrypt with `KeyedTernarySponge`
```rust
use plenumnet_kernel::crypto::keyed_sponge::KeyedTernarySponge;

// Load a 243-trit key from a .trits file ("-/0/+" string).
let raw = std::fs::read_to_string("keys/encryption/development.trits")?;
let key: Vec<i8> = raw.chars().filter_map(|c| match c {
    '-' => Some(-1), '0' => Some(0), '+' => Some(1), _ => None,
}).collect();
assert_eq!(key.len(), 243);

// Per-frame nonce (uniqueness — not randomness — is the requirement).
let nonce_trits: Vec<i8> = /* timestamp+counter, see plenumnet_kernel::repo_sync::next_nonce */;

let mut sponge = KeyedTernarySponge::new(&key);
sponge.absorb(&nonce_trits);
let keystream = sponge.squeeze(plaintext.len() * 8).trits;
// XOR keystream into plaintext using the unbiased trit→byte mapping
// shown in src/kernel/src/repo_sync.rs::stream_xor_in_place.
```

### TypeScript — call into the kernel via N-API
```typescript
import { keyedSpongeEncrypt } from "ternary-math-napi";

const trits = fs.readFileSync("keys/encryption/development.trits", "utf8");
const ciphertext = keyedSpongeEncrypt(trits, plaintext);
```

The TypeScript side never handles raw key bytes; it forwards the trit
string straight into the N-API binding which calls into the same Rust
`KeyedTernarySponge` path.

## Security Best Practices

1. **Never reuse a nonce.** RepoSync derives nonces from
   `(timestamp_nanos, atomic_counter)`; the relay channel does the same.
2. **Validate keys at the entrypoint.** Reject wrong-length keys, trits
   outside `{-1, 0, +1}`, and the all-zero placeholder. See
   `Config::validate()` in `src/kernel/src/repo_sync.rs`.
3. **Sign what you encrypt.** TL-DSA-87 signatures must cover
   `nonce ‖ ciphertext`, not ciphertext alone, to block mix-and-match
   attacks.
4. **Rotate keys on the radian-epoch schedule** documented in the
   Inter-Cube specs.
5. **Wipe trit buffers on drop.** `Vec<i8>` does not zero on drop; wrap
   keys in `crypto::secret::TritSecret` for automatic zeroization.

## Generating a Development Key

```bash
# 243 random balanced trits (-, 0, +) written as a single line.
python3 -c "import secrets; print(''.join(secrets.choice('-0+') for _ in range(243)))" \
  > development.trits
```

A development key is **not** committed to the repository. Each operator
generates their own and configures it in
`%APPDATA%\PlenumNET-RepoSync\config.toml` (or the equivalent for the
service that needs it) under the `shared_key_trits` field.
