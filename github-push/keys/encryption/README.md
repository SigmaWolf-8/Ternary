# Encryption Keys

This directory contains encryption keys for:
- Data-at-rest encryption
- Transport layer security
- Session encryption
- Phase encryption (PQTI-specific)

## Key Formats

- `.key` - Raw symmetric key (hex encoded)
- `.pem` - PEM format for asymmetric keys
- `.der` - DER format (binary)

## Algorithms Supported

### Symmetric Encryption
1. **AES-256-GCM** - Recommended for general use
2. **ChaCha20-Poly1305** - For software-only environments

### Asymmetric Encryption
1. **X25519** - Key exchange
2. **RSA-OAEP-4096** - Legacy compatibility

### Post-Quantum
1. **Ternary Bijective Encryption** - PQTI native
2. **Phase-Split Encryption** - Quantum-resistant

## Usage Examples

### Rust
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::Aead;

// Load encryption key
let key_hex = std::fs::read_to_string("keys/encryption/development.key")?;
let key_bytes = hex::decode(key_hex.trim())?;
let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

// Encrypt data
let cipher = Aes256Gcm::new(key);
let nonce = Nonce::from_slice(b"unique_nonce");
let ciphertext = cipher.encrypt(nonce, plaintext)?;
```

### TypeScript
```typescript
import { createCipheriv, randomBytes } from 'crypto';

const key = Buffer.from(fs.readFileSync('keys/encryption/development.key', 'utf8').trim(), 'hex');
const iv = randomBytes(16);
const cipher = createCipheriv('aes-256-gcm', key, iv);
```

## Security Best Practices

1. Use authenticated encryption (GCM, Poly1305)
2. Never reuse nonces
3. Derive unique keys for each encryption context
4. Implement key rotation
5. Securely delete decrypted data after use

## Development Key

A development key is NOT included for security reasons. Generate one using:

```bash
openssl rand -hex 32 > development.key
```
