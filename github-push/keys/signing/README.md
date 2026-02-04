# Signing Keys

This directory contains digital signature keys for:
- API request signing
- XRPL transaction signing
- Audit log signing
- Message authentication

## Key Formats

- `.pem` - PEM format (Base64 encoded)
- `.der` - DER format (binary)
- `.key` - Raw key material

## Algorithms Supported

1. **ED25519** - Recommended for high performance and security
2. **RSA-4096** - For compatibility with legacy systems
3. **ECDSA-P384** - For regulatory compliance

## Usage Examples

### Rust
```rust
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};

// Load signing key
let key_bytes = std::fs::read("keys/signing/development.pem")?;
let signing_key = SigningKey::from_bytes(&key_bytes)?;

// Sign message
let message = b"Transaction data";
let signature = signing_key.sign(message);
```

### TypeScript
```typescript
import { sign, verify } from 'crypto';

const privateKey = fs.readFileSync('keys/signing/development.pem');
const signature = sign('sha256', Buffer.from(data), privateKey);
```

## Security Best Practices

1. Never log or expose private keys
2. Use secure memory allocation for key operations
3. Zero memory after key operations
4. Implement key derivation for per-session keys
5. Store production keys in HSM or secure vault

## Development Key

A development key is NOT included for security reasons. Generate one using:

```bash
openssl genpkey -algorithm ED25519 -out development.pem
```
