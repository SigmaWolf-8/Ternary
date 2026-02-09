# Changelog

All notable changes to libternary are documented in this file.

## [3.0.0] - February 2026 — "CMVP Submission Ready"

### Added — Stage 9: CMVP Critical Infrastructure (~1,700 lines)
- `entropy.rs` — SP 800-90B entropy source (RDTSC/CNTVCT jitter, RCT+APT health tests, HMAC-SHA-384 conditioning)
- `drbg.rs` — SP 800-90A HMAC-DRBG-SHA384 deterministic random bit generator
- `self_test.rs` — FIPS 140-3 POST (12 KATs) and conditional self-tests (5 triggers)
- `module_state.rs` — 9-state finite state machine with transition enforcement
- `services.rs` — 32 crypto services (26 approved, 6 non-approved) with role-based access (3 roles)
- `acvts.rs` — NIST ACVTS JSON format registration and response generation (12 algorithms)
- `cavp_certs.rs` — CAVP certificate tracking (12 pending certificates)

### Added — Stage 10: CMVP Documentation Package (~1,770 lines)
- `security-policy.md` — Non-Proprietary Security Policy, 12-section SP 800-140B format (VE-001)
- `ssp-inventory.md` — 23 Sensitive Security Parameters with generation/storage/zeroization (VE-004)
- `finite-state-model.md` — 9-state FSM with complete transition table (VE-003)
- `boundary-diagram.md` — Cryptographic boundary: 34 files inside, excluded components listed (VE-002)
- `entropy-assessment.md` — SP 800-90B entropy source assessment with MCV estimator (VE-006)
- `operational-environments.md` — OE-1 (x86_64) and OE-2 (aarch64) specifications (VE-005)
- `vendor-evidence-index.md` — 22 vendor evidence documents indexed (VE-001 through VE-022)

### Added — Stage 11: Build Infrastructure & Lab Engagement
- `scripts/cmvp-build.sh` — Deterministic reproducible build (LTO, codegen-units=1, HMAC-SHA-384 integrity)
- `.github/workflows/fips-self-tests.yml` — 6-job CI: POST, conditional tests, entropy, state machine, ACVTS, reproducible builds
- `scripts/collect-entropy-samples.sh` — 1M+ raw entropy sample collection for SP 800-90B lab assessment
- `docs/compliance/cstl-engagement.md` — CSTL lab selection, engagement timeline, test execution guide (VE-021)

### Changed
- Version bumped to 3.0.0 (CMVP Submission Ready)
- Security Policy rewritten from scratch: 12 SP 800-140B sections, 32 services, 23 SSPs
- VERSION_MANIFEST.json updated: Phase 3, 34 crypto modules, CMVP documentation references
- SP 800-208 gap closed: Full XMSS (heights 10/16/20) and LMS (heights 5/10/15/20/25) in signature.rs
- Module count: 34 crypto files (27 Stage 8 + 7 Stage 9)

### FIPS 140-3 Status
- Security Level: 1
- Algorithms: 12 CAVP registrations ready (AES, SHA-2, SHA-3, HMAC, ML-KEM, ML-DSA, LMS, XMSS, HMAC-DRBG)
- POST: 12 Known Answer Tests
- Conditional Tests: 5 triggers (KEM keygen, DSA keygen, signature keygen, DRBG output, firmware load)
- SSPs: 23 identified (6 symmetric, 6 asymmetric private, 3 public, 4 DRBG, 2 state, 2 other)
- FSM: 9 states, all transitions defined
- Services: 32 total (26 approved, 6 non-approved), 3 roles (CryptoOfficer, User, None)
- Entropy: SP 800-90B qualified (RDTSC/CNTVCT jitter, HMAC-SHA-384 conditioning, RCT+APT)
- Build: Reproducible (LTO=fat, codegen-units=1, panic=abort, HMAC-SHA-384 integrity)

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
