# Key Management

## Directory Structure

```
keys/
  signing/          — Code signing keys and procedures
  encryption/       — Encryption key documentation
```

## Security Policy

- Private keys are NEVER committed to this repository
- Only public keys and documentation are stored here
- See `SECURITY.md` in the repository root for vulnerability reporting
- See `.github/BRANCH_PROTECTION.md` for commit signing requirements

## Key Directories

### `signing/`

Contains code signing procedures and public keys for verifying release artifacts, commits, and kernel modules.

### `encryption/`

Contains encryption key management documentation and public certificates for TLS and data protection.
