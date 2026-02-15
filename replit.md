# PlenumNET Framework Marketing Website

## Overview
PlenumNET is a post-quantum internet solutions company developing quantum-resistant infrastructure. This project delivers a professional marketing website to showcase PlenumNET's offerings, including the PlenumDB product with a live compression demo and comprehensive whitepaper management. It also integrates a complete payment processing and blockchain witnessing architecture, ensuring secure, verifiable, and regulatory-compliant operations for quantum-resistant data and financial services. The project's vision is to establish PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## Repository Metrics (as of February 14, 2026)
- **Commits:** 646+ on main branch
- **Source Files:** 723 (excl. node_modules, .git, target)
- **TypeScript/TSX:** 181 files, ~33,263 lines
- **Rust:** 140 files, ~58,031 lines
- **Markdown Documentation:** 99+ files
- **API Endpoints:** 93 registered Express routes (with /api/v1/ versioning alias)
- **CI/CD Workflows:** 15 GitHub Actions workflows (including fuzz testing)
- **Vitest Tests:** 86+ base tests + integration test suites (API routes, blockchain, webhooks)
- **Fuzz Targets:** 3 (trit ops, tryte ops, gateway)

## System Architecture

### Frontend
The frontend is built with React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter for routing, supporting both light and dark modes with a unified blue brand identity. It features a dual-layout navigation system (`MarketingLayout` for public pages and `DashboardLayout` for tools/admin) with dynamic page titles and anchor navigation. Key pages include the Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, and an Admin Dashboard. The system supports ancient calendar synchronization across 24 global systems, anchoring the Salvi Epoch to provide historical and cross-cultural timing conversions. SEO is optimized with robots.txt, sitemap.xml, JSON-LD, and PWA capabilities.

### Backend and Core Framework
The backend utilizes Express.js and Node.js with PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic System operations, Femtosecond Timing, and Phase Encryption. The backend routes are modularized across 6 files (routes.ts, github.ts, kong.ts, salvi.ts, tribonacci.ts, middleware.ts) totaling ~4,057 lines organized by domain. Security is a priority, incorporating tiered rate limiting (4 levels: global 100/min, auth 20/min, token 10/min, computation 50/min), CORS enforcement with origin allowlist, Helmet.js security headers (CSP, HSTS, X-Frame-Options: deny), AES-256-GCM token encryption (SESSION_SECRET required), input validation bounds, hardened path sanitization, and execFile()-only subprocess execution. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service for regulatory compliance. API versioning is supported via /api/v1/ prefix with backward-compatible aliasing.

### Rust Kernel Architecture
The `src/kernel/` directory houses a Rust-based kernel providing advanced functionalities:
-   **Ternary Operations**: GF(3) arithmetic and conversions.
-   **Timing**: Femtosecond-precision timing and High-Precision Timing Protocol (HPTP).
-   **Phase Encryption**: Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem**: Bitmap-based frame allocator, page table management with ternary security, and heap allocator.
-   **Synchronization Primitives**: Ticket spinlocks, ternary-security-gated mutexes, semaphores, and phase-encryption-aware mutexes.
-   **Process Management**: States, priority levels, scheduler, CPU context, and message-passing IPC.
-   **Modal Security System**: Domain management, capability-based access control, timestamped audit trails, and policy engine.
-   **Cryptographic Primitives**: Ternary hash, sponge, HMAC, KDF, Lamport signatures, AES-256-GCM, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial arithmetic, and CNSA 2.0 compliance.
-   **Device Driver Framework**: Abstractions for various device types and buses.
-   **I/O Subsystem**: Priority-based scheduler, buffer cache, block/character device layers.
-   **Filesystem**: Inode management, directory/file operations, mount system.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: A 160-opcode ISA v2.0 supporting core, extended, crypto acceleration, SIMD, system, security/audit, and debug/profiling operations with ternary addressing, three-ring privilege levels, and a ternary-aware garbage collector.
-   **Binary Compatibility Layer**: For balanced ternary conversion and crypto interoperability.

### Legal & IP Compliance
All 321+ source files include standardized copyright headers from "Capomastro Holdings Ltd. (Canada)" with "Patent(s) Pending." A CI workflow (`license-check.yml`) enforces header presence across all directories including services/. Legal documents for terms, privacy, and security are served dynamically from markdown content. IP-NOTICE.md documents patent-pending claims and proprietary algorithms. EXPORT-CONTROL.md provides CNSA 2.0 and Wassenaar classification guidance.

### GitHub Integration
An admin-only GitHub Manager page facilitates file browsing for the `SigmaWolf-8/Ternary` repository and enables push actions for CI/CD, crypto modules, and project synchronization. Branch protection is enabled on main with required status checks.

### Saturnian Magic Square Blueprint (February 15, 2026)
The `shared/saturnian-blueprint.ts` module provides the 3×3 Saturnian circulant magic square (111, 14, 208 with magic constant 333) as a static foundation for SUFT-derived constants. It bridges directly to the Tribonacci sequence: RADIUS_COSMIC = 13 = T(7), PI_ESOTERIC = 14 = T(7)+T(3), LUNAR_SOLAR_HARMONIC = 28, COSMIC_CIRCUMFERENCE = 364 — all exact integer identities matching the ternary circle axioms. Companion utilities in `shared/saturnian-matrix-utils.ts` provide flattening, cyclic rotation, ternary weighting, and magic/circulant validation. 29 Vitest tests verify all derivations.

### Testing
Testing is conducted using Vitest with multiple test suites:
- 29 Saturnian blueprint + Tribonacci bridge tests (saturnian-blueprint.test.ts)
- 50 GF(3) arithmetic tests (ternary-operations.test.ts)
- 25 phase encryption tests (phase-encryption.test.ts)
- 11 calendar synchronization tests (calendar-sync.test.ts)
- API route integration tests (tests/integration/api-routes.test.ts)
- Blockchain service tests (tests/integration/blockchain-services.test.ts)
- Payment webhook tests (tests/integration/payment-webhooks.test.ts)
- 3 Rust fuzz targets with CI workflow (fuzz.yml)
- Kernel tests with cargo test, coverage, Miri, and feature matrix CI

### 28-Dimension Agent Array (February 15, 2026)
The Agent Array system orchestrates 28 specialist AI agents analyzing queries in parallel. Architecture: all agents analyze in English → Etymology Audit → Veritas Fact-Check → synthesize ONE unified Situation Report (with Veritas verdicts incorporated) → Lexical Protocol enforcement → translate into 28 world languages (concurrency=14). Three integrity protocols ensure factual accuracy:
- **Etymology Engine**: Traces term origins, evolution, cross-cultural notes; flags anachronistic/incorrect usage; identifies synchronized vs unsynchronized terms
- **Veritas Audit**: 5-source, 3-culture fact-checking with confidence scores and verdicts (VERIFIED/UNVERIFIED/DISPUTED/FALSE); FALSE claims excluded from final report; DISPUTED/UNVERIFIED claims qualified with caveats
- **Lexical Protocols** (v2.0): Terminological consistency enforcement across 28 translations; etymological anchoring; Latin legal maxims preserved; unsynchronized terms get local equivalent + English parenthetical
SSE events: agent_result, layer1_complete, layer2_section, executive_summary, etymology_start, etymology_complete, veritas_start, veritas_complete, report_start, report_generated, translation_progress, lexical_applied, translations_complete. Reports persist to `agent_array_reports` PostgreSQL table. Frontend: Integrity Protocols panel (Etymology/Veritas/Lexical), SituationReportViewer with 28-language filter buttons, copy/save, and ReportHistory.

### Kong Gateway Integration
Kong Konnect API integration manages services, routes, and plugins. It supports service synchronization for PlenumNET's 17 services (97 endpoints) and allows optional Kong proxy routing in the frontend.

### Security Middleware Stack
- **Rate Limiting:** 4 tiers (global, auth, token, computation) via express-rate-limit
- **CORS:** Origin allowlist with strict rejection
- **Headers:** Helmet.js with CSP, HSTS, X-Frame-Options: deny, X-Content-Type-Options: nosniff
- **Encryption:** AES-256-GCM for token storage, SESSION_SECRET required (no fallbacks)
- **Path Security:** Null-byte stripping, double URL-decode protection, post-normalize traversal rejection
- **Subprocess:** execFile() only, zero exec() calls

## External Dependencies

-   **Authentication**: Replit Auth (GitHub, Google, Apple, X, email/password).
-   **Database**: PostgreSQL.
-   **ORM**: Drizzle ORM.
-   **API Gateway**: Kong Konnect.
-   **Payment Gateways**: Stripe, Interac, various cryptocurrency platforms.
-   **Message Queue**: BullMQ.
-   **Blockchain Platforms**: Hedera Hashgraph Consensus Service (HCS), XRP Ledger (XRPL), Algorand.
-   **Containerization**: Docker.
-   **Cloud Deployment**: Render, Railway.
