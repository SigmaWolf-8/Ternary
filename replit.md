# PlenumNET Framework Marketing Website

## Overview
PlenumNET is a post-quantum internet solutions company focused on building quantum-resistant infrastructure. This project is a professional marketing website showcasing PlenumNET's deployable components, including a PlenumDB product page with a live compression demo and comprehensive whitepaper management. The project also encompasses a complete payment processing and blockchain witnessing architecture, integrating high-precision timing and regulatory compliance for secure and verifiable operations in quantum-resistant data and financial services.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## Legal Jurisdiction
All legal documents use **Province of Alberta** (not Ontario) for governing law, jurisdiction, court references, and entity incorporation ("incorporated under the laws of the Province of Alberta"). Capomastro Holdings Ltd. is an Alberta corporation.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui` for components, and Wouter for routing. It supports light and dark mode (class-based toggle with localStorage persistence) unified around a blue brand identity. Navigation uses a collapsible shadcn Sidebar. Key pages include the Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, and Admin Dashboard. Code splitting via React.lazy() is used for 14 non-critical pages (including About, Contact), with an ErrorBoundary wrapping the app. The landing page includes: Hero with animated 5-layer architecture visual and interactive ternary converter demo widget, Platform capabilities, Architecture diagram, Deployable components, Performance comparison table (with honesty rows), Trust signals, Developer code snippet with copy button, Target markets, Changelog (GitHub API), and Developer CTA with "Apply for SDK Access" + "Book a Demo" (both forms have success confirmation states). Legal pages render markdown content with an inline parser instead of raw text. Whitepaper uses react-markdown with remark-gfm and rehype-raw for proper rendering. SEO includes robots.txt, sitemap.xml, manifest.json, JSON-LD structured data (Organization, SoftwareApplication, WebSite schemas), resource hints (preconnect/dns-prefetch), and service worker for PWA offline capability. WCAG AA contrast compliance is verified. UI strings extracted to `client/src/lib/strings.ts` for i18n readiness.
The `salvi_docs/` directory contains comprehensive developer documentation. The `/docs` page on the site provides a browsable index linking to GitHub-hosted markdown files.
The system supports ancient calendar synchronization, anchoring the Salvi Epoch to 24 global calendar systems spanning over 30,000 years, with conversions computed via JDN and individual API endpoints for each calendar system.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic System operations (conversion, arithmetic, XOR in GF(3)), Femtosecond Timing, and Phase Encryption. Ternary representations include Computational (`{-1, 0, +1}`), Network (`{0, 1, 2}`), and Human (`{1, 2, 3}`).

**Route Architecture** (refactored 2026-02-12): The monolithic `server/routes.ts` was decomposed into focused modules:
- `server/routes.ts` — Core routes: auth, ternary operations, compression, timing, whitepapers, legal, contact (~890 lines)
- `server/routes/github.ts` — GitHub file browser, push actions, CI/CD triggers (544 lines)
- `server/routes/kong.ts` — Kong Konnect integration, service/route/plugin management (1278 lines)
- `server/routes/salvi.ts` — Salvi epoch, calendar sync, CNSA compliance, phase encryption API (1038 lines)
- `server/routes/middleware.ts` — Shared auth and admin middleware (49 lines)

**Security Stack** (added 2026-02-12):
- Rate limiting via express-rate-limit: global (100/min), auth (20/min), GitHub token (10/min), computation (50/min)
- CORS restricted to Replit deployment domains
- Helmet.js security headers (HSTS, CSP, X-Content-Type-Options)
- AES-256-GCM token encryption for stored credentials (`server/crypto-utils.ts`)
- Input validation bounds: pageSize ≤ 1000, tritCount ≤ 1000, dataLength ≤ 10000, batch ≤ 100
- `execFile()` instead of `exec()` to prevent command injection
- Hardened `sanitizePath()` with null-byte stripping and double-encoding protection

**Infrastructure**:
- Structured Winston logger (`server/logger.ts`) with JSON formatting and log levels
- Centralized environment config (`server/config.ts`) with typed defaults and validation
- All error handling uses `catch(error: unknown)` with `toErrorMessage()` helper

The architecture includes microservices for payment processing and blockchain witnessing. These services handle payment webhooks, orchestrate CRUD operations, and integrate with blockchain platforms like Hedera HCS, XRPL, and Algorand. A Femtosecond Timing Service provides high-precision timing, and a Certification Service ensures regulatory compliance.
The database schema includes tables for `users`, `sessions`, `demo_sessions`, `binary_storage`, `ternary_storage`, `compression_benchmarks`, `compression_history`, and `whitepapers`.

### Rust Kernel Architecture
The `src/kernel/` directory contains a robust kernel developed in Rust, encompassing:
-   **Ternary Operations**: GF(3) arithmetic and representation conversions.
-   **Timing**: Femtosecond-precision timestamps and the High-Precision Timing Protocol (HPTP).
-   **Phase Encryption**: Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem**: Bitmap-based frame allocator, page table management with ternary security modes, and free-list heap allocator.
-   **Synchronization Primitives**: Ticket-based spinlocks, ternary-security-gated mutexes, semaphores, and phase-encryption-aware mutexes.
-   **Process Management**: Process states, priority levels, multi-level priority round-robin scheduler, CPU context management, and message-passing IPC.
-   **Modal Security System**: Security domain management, capability-based access control, femtosecond-timestamped audit trails, and a priority-ordered policy engine.
-   **Cryptographic Primitives**: Ternary hash, sponge construction, HMAC, key derivation, ternary Lamport one-time signatures, AES-256-GCM cipher, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial ring arithmetic, and CNSA 2.0 compliance framework (100% coverage).
-   **Device Driver Framework**: Abstractions for device types, bus management, device registry, interrupt controller, and DMA.
-   **I/O Subsystem**: Priority-based I/O scheduler, buffer cache, block device layer, character device layer, and I/O multiplexing.
-   **Filesystem**: Inode management, directory operations, file operations, and a mount system.
-   **Error Handling**: Unified `KernelError` enum.
-   **Architecture Support**: Generic traits and specific implementations for `x86_64`, `aarch64`, and `riscv64`.
-   **Hardware Drivers**: Drivers for TPU FPGA/ASIC and femtosecond clock.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: 55-opcode ISA across 9 categories: Tier 1 core ops (TAdd=GF(3) addition, TMul=GF(3) multiplication, TXor=Kleene min, TNeg, TRot, TConvert); Tier 2 extended ops (TAnd=Łukasiewicz conjunction, TOr=Kleene max, TSub, TInv=GF(3) multiplicative inverse, TShift, TCmp, TLoad, TStore, TReduce, TRotInv); Tier 3 crypto acceleration (TPolyMul/TNTT/THash/TEntropy/TPolyAdd/TPolySample/TCompress/TDecompress); Tier 4 SIMD vector (TAddV/TMulV/TNegV/TRotV); System ops (Syscall/Trap/Alloc/Free/ReadTime). Architecture features: 27-register packed-trit file (27 trits per i64 via 2-bit encoding), triple instruction encoding (legacy 16-byte, compact 4/6-byte, balanced ternary 5/7-byte), ternary addressing mode for TLoad/TStore, ternary_mode flag propagation, Ring0/Ring1 privilege levels, process scheduler integration (time slicing), security domain wiring, ternary-aware mark-sweep garbage collector (TAGC), instruction cache with hit-rate tracking, and constant-time GF(3) operations for side-channel resistance. Complete development toolchain: text assembler with label support and 60+ mnemonic aliases, disassembler, interactive debugger with breakpoints/stepping/register dumps, THDL-to-VM circuit compiler, and comprehensive test suite (205+ tests including GF(3) property tests, exhaustive fuzz coverage, and throughput benchmarks). ISA reference at `src/kernel/ISA_REFERENCE.md`.
-   **Binary Compatibility Layer**: Gateway for balanced ternary conversion, a universal ternary adapter, and CryptoInteropBridge for ML-KEM/ML-DSA interoperability.

### Legal & IP Compliance
All source files (224 total: 121 Rust, 103 TypeScript/JavaScript) carry standardized copyright headers with "Patent(s) Pending" and "Capomastro Holdings Ltd. (Canada)" designation. A `license-check.yml` CI workflow enforces header presence on all commits. Legal pages at `/terms`, `/privacy`, `/security` are served via `/api/legal/:type` with content from local markdown files. Key legal documents on GitHub: LICENSE, CLA.md, CONTRIBUTING.md, TERMS-OF-SERVICE.md, ACCEPTABLE-USE-POLICY.md, INTELLECTUAL-PROPERTY-NOTICE.md, TRADEMARK-NOTICE.md, SECURITY.md, FILE-HEADER-TEMPLATE.md, and CODE-OF-CONDUCT.md.

### GitHub Integration
The GitHub Manager page (`/github`, admin-only) provides a file browser for the `SigmaWolf-8/Ternary` repository and enables push actions for CI/CD workflows, crypto modules, and a full project sync.

### Testing
- **Framework**: Vitest (configured in `vitest.config.ts`)
- **Run tests**: `npx vitest run` (86 tests across 3 files)
- **Test files**:
  - `tests/ternary-operations.test.ts` — 50 GF(3) arithmetic KAT tests
  - `tests/phase-encryption.test.ts` — 25 phase encryption round-trip tests
  - `tests/calendar-sync.test.ts` — 11 calendar synchronization tests
- **CI**: `.github/workflows/test-typescript.yml` triggers on push/PR to main/develop

### Recent Changes (2026-02-12)
- Completed 6-phase repository remediation (IP, legal, security, architecture, testing, documentation)
- See `CHANGELOG.md` for full details

## External Dependencies

-   **Authentication**: Replit Auth (GitHub, Google, Apple, X, email/password).
-   **Database**: PostgreSQL.
-   **ORM**: Drizzle ORM.
-   **API Gateway**: Kong Konnect (18 services, 70+ endpoints, full CNSA 2.0 enforcement).
-   **Payment Gateways**: Stripe, Interac, various cryptocurrency platforms.
-   **Message Queue**: BullMQ.
-   **Blockchain Platforms**: Hedera Hashgraph Consensus Service (HCS), XRP Ledger (XRPL), Algorand.
-   **Containerization**: Docker.
-   **Cloud Deployment**: Render, Railway.