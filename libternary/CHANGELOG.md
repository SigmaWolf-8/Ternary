# Changelog

All notable changes to libternary are documented in this file.

## [2.0.0] - February 2026

### Added
- CNSA 2.0 compliance metadata and algorithm references
- Post-quantum cryptography keywords (ML-KEM, ML-DSA, FIPS 203/204)
- Version manifest with kernel crypto module inventory
- Build configuration for distribution artifact packaging

### Changed
- Version bumped to 2.0.0 to reflect CNSA 2.0 full coverage in kernel
- Package description updated to include CNSA 2.0 compliance

### Kernel Modules (Referenced)
The following Rust kernel crypto modules are available in the Salvi Framework:
- `tl_kem.rs` — TL-KEM key encapsulation (3 security levels)
- `tl_dsa.rs` — TL-DSA digital signatures (3 security levels)
- `cipher.rs` — AES-256-GCM with ternary key mapping
- `sha2.rs` — SHA-384/512
- `sha3.rs` — SHA-3 (Keccak)
- `ternary_lattice.rs` — GF(3) polynomial ring arithmetic
- `cnsa2.rs` — CNSA 2.0 compliance tracking (11/11 algorithms)
- `crypto_interop.rs` — ML-KEM/ML-DSA binary interoperability bridge

## [1.0.0] - January 2026

### Added
- Initial release
- Three bijective ternary representations (A, B, C)
- GF(3) ternary operations (add, multiply, rotate, XOR, NOT)
- Femtosecond timestamp generation
- Phase-aware encryption (split/recombine)
- Information density calculator
