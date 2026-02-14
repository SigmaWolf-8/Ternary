# Changelog

All notable changes to the PlenumNET / Salvi Framework project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased] - 2026-02-12

### Security
- **IP Remediation**: Corrected kernel Cargo.toml and WASM Cargo.toml license declarations from `MIT` to `LicenseRef-Proprietary` to protect proprietary IP
- **Rate Limiting**: Added express-rate-limit with tiered limiters — global (100 req/min), auth (20/min), GitHub token (10/min), computation (50/min)
- **CORS**: Restricted to Replit deployment domains only
- **Helmet.js**: Added security headers including HSTS, X-Content-Type-Options, and CSP
- **Token Encryption**: Implemented AES-256-GCM encryption for stored GitHub tokens via crypto-utils.ts, with automatic encrypt/decrypt in storage layer
- **Input Validation**: Added strict bounds — pageSize ≤ 1000, tritCount ≤ 1000, dataLength ≤ 10000, batch count ≤ 100
- **Command Injection**: Replaced `child_process.exec()` with `execFile()` to prevent shell injection
- **Path Traversal**: Hardened `sanitizePath()` with null-byte stripping, double-encoding protection, and Windows path normalization
- **Error Typing**: Migrated all `catch(error: any)` to `catch(error: unknown)` with `toErrorMessage()` helper across all route files

### Changed
- **Route Architecture**: Decomposed monolithic routes.ts (3750 lines) into focused modules — github.ts (544 lines), kong.ts (1278 lines), salvi.ts (1038 lines), middleware.ts (49 lines), reducing routes.ts to 890 lines (76% reduction)
- **Logging**: Replaced all `console.log`/`console.error` with structured logger (server/logger.ts) with JSON formatting, log levels, and module tagging
- **Configuration**: Created centralized config.ts for environment variables with validation and typed defaults
- **Legal Jurisdiction**: Corrected all legal documents from "Province of Ontario" to "Province of Alberta" per corporate registration

### Added
- **Test Suite**: 86 automated tests via Vitest framework
  - 50 GF(3) ternary arithmetic KAT tests (addition, multiplication, XOR, NOT, double negation, commutativity, associativity, identity, annihilator, distributivity, representation conversion)
  - 25 phase encryption round-trip tests (4 modes, guardian phase, split structure, integrity validation, edge cases)
  - 11 calendar synchronization tests (Gregorian, Julian, Islamic, Hebrew, Chinese, Persian, Ethiopian, Buddhist, Japanese, Coptic, ISO 8601)
- **CI Pipeline**: GitHub Actions workflow for TypeScript tests (test-typescript.yml) with Node.js 20, triggered on push/PR to main and develop branches
- **CODE-OF-CONDUCT.md**: Contributor Covenant v2.1 adapted for post-quantum research community
- **SECURITY.md**: Vulnerability disclosure policy with PGP-signed reporting, severity classification, and 90-day disclosure timeline
- **License Headers**: Added proprietary copyright headers (Capomastro Holdings Ltd., Patent(s) Pending) to 85+ TypeScript and Rust source files
- **.gitignore**: Added `src/kernel/target/` and nested Cargo build directories to prevent binary artifact commits

### Fixed
- **En-dash**: Corrected "Patent(s) Pending" character encoding from en-dash to ASCII hyphen in copyright headers
- **MANIFEST-FIXES.md**: Documented all IP exposure issues and remediation status

### Deferred
- **API Versioning** (`/api/v1/` prefix): Would require updating all frontend API calls; deferred to avoid breaking changes
- **Rust Compiler Warnings**: 11 warnings in kernel code require local Rust build environment
- **Rust Kernel Tests**: Expansion requires `cargo test` in native environment
- **Build Artifact Removal**: 280MB `src/kernel/target/` directory in git history requires `git filter-branch` or BFG Repo-Cleaner outside Replit
