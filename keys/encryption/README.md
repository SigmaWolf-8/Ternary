# Encryption Key Management

## Overview

This directory contains documentation and public keys for PlenumNET encryption operations. Private keys are NEVER stored in this repository.

## Key Types

| Key Type | Algorithm | Purpose |
|----------|-----------|---------|
| TLS Certificates | ECDSA P-384 / RSA-4096 | API gateway and service TLS |
| Data-at-Rest | AES-256-GCM | Database and storage encryption |
| Phase Encryption | Ternary Phase Keys | PlenumNET phase encryption system |
| TL-KEM Session Keys | TL-KEM-768/1024 | Post-quantum key encapsulation |

## Security Requirements

- All private keys stored in HSM or encrypted vault
- Key access logged and audited
- Minimum key sizes per CNSA 2.0 requirements
- Post-quantum algorithms required for new deployments after 2026

## Related Documentation

- `keys/signing/SIGNING_PROCEDURES.md` — Code signing procedures
- `src/kernel/src/crypto/cnsa2.rs` — CNSA 2.0 algorithm tracking
- `.github/BRANCH_PROTECTION.md` — Repository security settings
