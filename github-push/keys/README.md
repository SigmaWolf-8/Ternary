# Key Management

This directory contains cryptographic keys for the Post-Quantum Ternary Internet.

## Structure

- `signing/` - Digital signature keys for authentication and verification
- `encryption/` - Encryption keys for data protection

## Security Notice

**WARNING: Never commit real cryptographic keys to version control!**

The keys in this directory are for development and testing only. In production:

1. Generate unique keys for each deployment
2. Store keys in secure key management systems (HSM, KMS, Vault)
3. Use environment variables or secret management services
4. Implement key rotation policies

## Development Keys

Development keys are provided for testing purposes only. They should be replaced before any production deployment.

## Key Generation

For production use, generate keys using:

```bash
# Generate signing key (ED25519)
openssl genpkey -algorithm ED25519 -out signing/production.pem

# Generate encryption key (256-bit)
openssl rand -hex 32 > encryption/production.key
```

## Key Rotation

Implement regular key rotation:
- Signing keys: Rotate every 90 days
- Encryption keys: Rotate every 30 days for high-security applications
- Compromised keys: Rotate immediately

## Compliance

Keys must comply with:
- NIST SP 800-57: Key Management
- NIST SP 800-131A: Transitioning Cryptographic Algorithms
- Industry-specific regulations (HIPAA, GDPR, FINRA)

## Directory Contents

```
keys/
├── README.md           (this file)
├── signing/
│   ├── README.md       (signing key documentation)
│   └── .gitkeep
└── encryption/
    ├── README.md       (encryption key documentation)
    └── .gitkeep
```
