# PlenumNET API Services Managed by Kong Konnect

**Version**: 3.0.0 | **Last Updated**: February 2026
**Organization**: Capomastro Holdings Ltd.

---

## Overview

Kong Konnect serves as the API gateway for the PlenumNET platform, managing traffic, security, rate limiting, and observability across all public and internal API services. This document provides a comprehensive inventory of every API service, what it does, and the endpoints it exposes.

**Current Service Count**: 18 distinct API service groups | 70+ individual endpoints

---

## Table of Contents

1. [Femtosecond Timing (HPTP)](#1-femtosecond-timing--hptp-protocol)
2. [Ancient Calendar Synchronization](#2-ancient-calendar-synchronization)
3. [Ternary Computing Engine](#3-ternary-computing-engine)
4. [Phase Encryption](#4-phase-encryption)
5. [Compression Engine](#5-compression-engine)
6. [Kernel Crypto Modules](#6-kernel-crypto-modules)
7. [Whitepaper Management](#7-whitepaper-management)
8. [API Documentation](#8-api-documentation)
9. [Developer Signup](#9-developer-signup)
10. [Authentication & User Management](#10-authentication--user-management)
11. [Admin Dashboard](#11-admin-dashboard)
12. [GitHub Integration](#12-github-integration)
13. [Kong Gateway Management](#13-kong-gateway-management)
14. [CNSA 2.0 Compliance Enforcement](#14-cnsa-20-compliance-enforcement)
15. [Certification & Audit Trail](#15-certification--audit-trail)
16. [Payment Processing](#16-payment-processing)
17. [Blockchain Witnessing](#17-blockchain-witnessing)
18. [Health & Observability](#18-health--observability)

---

## 1. Femtosecond Timing / HPTP Protocol

**Kong Service Name**: `plenumnet-timing`
**Base Path**: `/api/salvi/timing`
**Status**: LIVE
**Rate Limit**: 100/min, 1,000/hr

### What it does (in plain terms)
This is our ultra-precise clock service. While a normal computer clock measures time in milliseconds (thousandths of a second), this API measures time in femtoseconds — that's one quadrillionth of a second. It uses a protocol called HPTP (High-Precision Timing Protocol) that corrects for internet delays so the time you see is accurate even after traveling across the network to reach you. Financial regulators require precise timestamps on every trade; this service exceeds those requirements by orders of magnitude.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/timing/timestamp` | Returns the current femtosecond-precision timestamp with HPTP latency correction (T2/T3 server-side capture for NTP-symmetric round-trip compensation) |
| GET | `/api/salvi/timing/metrics` | Returns timing performance metrics — clock source, sync status, drift rates |
| GET | `/api/salvi/timing/batch/:count` | Generates a batch of sequential timestamps (up to 100) for benchmarking |

### Regulatory Coverage
- **FINRA 613 / CAT**: Exceeds the 50 microsecond synchronization threshold
- **MiFID II Art. 50**: Exceeds the 100 microsecond HFT gateway requirement and 1ms standard trading requirement
- **Emerging ESMA/SEC**: Future-proofed well beyond any anticipated sub-microsecond requirements

---

## 2. Ancient Calendar Synchronization

**Kong Service Name**: `plenumnet-timing` (sub-service)
**Base Path**: `/api/salvi/timing/epoch`
**Status**: LIVE
**Rate Limit**: Shared with Timing (100/min)

### What it does (in plain terms)
This service converts today's date into 24 different calendar systems used around the world — past and present — spanning over 30,000 years of human timekeeping. From the Mayan Long Count to the Chinese Sexagenary Cycle to the Aboriginal Australian Seasonal calendar, every conversion is mathematically anchored to the Salvi Epoch (April 1, 2025 UTC) through Julian Day Number calculations. You can also convert any arbitrary date using the `?date=` parameter, making it a fully bidirectional converter.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/timing/epoch/anchors` | Returns the Salvi Epoch anchor points and reference data |
| GET | `/api/salvi/timing/epoch/calendars` | Returns all 24 calendar conversions for the current date (or `?date=` parameter) |

### Individual Calendar Endpoints (24 total)
Each supports `?date=YYYY-MM-DD` for arbitrary date conversion.

| Path | Calendar System | Region |
|------|----------------|--------|
| `/calendars/mayan` | Mayan Long Count | Mesoamerica |
| `/calendars/aztec` | Aztec Tonalpohualli (260-day sacred cycle) | Mesoamerica |
| `/calendars/hebrew` | Hebrew (Anno Mundi) | Middle East |
| `/calendars/islamic` | Islamic Hijri | Middle East |
| `/calendars/zoroastrian` | Zoroastrian Fasli | Middle East |
| `/calendars/persian` | Persian / Solar Hijri | Middle East |
| `/calendars/chinese` | Chinese Sexagenary (Yellow Emperor epoch) | East Asia |
| `/calendars/japanese` | Japanese Imperial (Koki/Reiwa) | East Asia |
| `/calendars/korean` | Korean Dangun Era | East Asia |
| `/calendars/vedic` | Vedic Kali Yuga | South Asia |
| `/calendars/indian-saka` | Indian National / Saka | South Asia |
| `/calendars/thai` | Thai Buddhist Era | Southeast Asia |
| `/calendars/bengali` | Bengali / Bangla | South Asia |
| `/calendars/balinese` | Balinese Pawukon (210-day cycle) | Southeast Asia |
| `/calendars/tibetan` | Tibetan Rabjung (60-year cycle) | Tibet/Mongolia |
| `/calendars/egyptian` | Egyptian Civil (Sothic Cycle) | Africa |
| `/calendars/ethiopian` | Ethiopian / Ge'ez (13 months) | Africa |
| `/calendars/coptic` | Coptic (Era of Martyrs) | Africa |
| `/calendars/berber` | Berber / Amazigh (Yennayer) | Africa |
| `/calendars/julian-day` | Julian Day Number | Europe/Mediterranean |
| `/calendars/byzantine` | Byzantine Anno Mundi | Europe/Mediterranean |
| `/calendars/roman` | Roman Ab Urbe Condita (Kalends/Nones/Ides) | Europe/Mediterranean |
| `/calendars/aboriginal` | Aboriginal Australian Seasonal (Dharawal) | Oceania |
| `/calendars/thirteen-moon` | 13-Moon Natural Time (13 x 28 days) | Universal |

---

## 3. Ternary Computing Engine

**Kong Service Name**: `plenumnet-ternary`
**Base Path**: `/api/salvi/ternary`
**Status**: LIVE
**Rate Limit**: 200/min, 2,000/hr

### What it does (in plain terms)
Normal computers use binary — just 0s and 1s. PlenumNET uses ternary — three values instead of two. This gives us more information per digit and enables unique security properties. This API lets you perform math and logic operations in the ternary system. Think of it like a calculator, but one that works in base-3 instead of base-2. It supports three different ways of writing ternary numbers depending on whether you're doing math (Representation A: -1, 0, +1), sending data over a network (Representation B: 0, 1, 2), or showing it to a person (Representation C: 1, 2, 3).

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/salvi/ternary/convert` | Converts a value between ternary representations A, B, and C |
| POST | `/api/salvi/ternary/add` | Adds two ternary digits in GF(3) — the finite field with 3 elements |
| POST | `/api/salvi/ternary/multiply` | Multiplies two ternary digits in GF(3) |
| POST | `/api/salvi/ternary/rotate` | Rotates a ternary value cyclically (like shifting gears) |
| POST | `/api/salvi/ternary/not` | Performs ternary NOT — inverts the value |
| POST | `/api/salvi/ternary/xor` | Performs ternary XOR in GF(3) — key building block for encryption |
| POST | `/api/salvi/ternary/batch` | Processes a batch of ternary additions in one call |
| GET | `/api/salvi/ternary/density/:tritCount` | Calculates information density — shows how many states N trits can represent vs. N bits |

---

## 4. Phase Encryption

**Kong Service Name**: `plenumnet-phase`
**Base Path**: `/api/salvi/phase`
**Status**: LIVE
**Rate Limit**: 100/min, 1,000/hr

### What it does (in plain terms)
Phase Encryption is PlenumNET's unique approach to securing data. Instead of just scrambling data with a password (like traditional encryption), it splits data into multiple "phases" — separate pieces that are meaningless on their own but can be recombined to reveal the original. It also enforces time windows, meaning the pieces can only be reassembled within a specific period. This adds a layer of protection that even quantum computers cannot easily break, because an attacker would need all the pieces and the right moment in time.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/phase/config/:mode` | Returns the encryption configuration for a given mode (high_security, balanced, performance, adaptive) |
| POST | `/api/salvi/phase/split` | Splits input data into phase-encrypted components with timing constraints |
| POST | `/api/salvi/phase/recombine` | Reassembles phase-split components back into the original data |
| GET | `/api/salvi/phase/recommend` | Recommends the best encryption mode based on data characteristics |

---

## 5. Compression Engine

**Kong Service Name**: `plenumnet-demo`
**Base Path**: `/api/demo`
**Status**: LIVE
**Rate Limit**: 50/min, 500/hr | Max payload: 10 MB

### What it does (in plain terms)
This service demonstrates PlenumDB's data compression capabilities. It generates realistic datasets (sensor readings, user events, server logs), compresses them, and shows you exactly how much smaller the data becomes. It's like a zip file, but built specifically for ternary-encoded data. You can upload your own files to see the compression ratio, or use the built-in data generators to run benchmarks. The results are stored so you can compare different runs.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/demo/run` | Runs a compression benchmark on generated data (sensor, events, or logs) |
| POST | `/api/demo/upload` | Upload your own file to measure compression ratio |
| GET | `/api/demo/stats` | Returns aggregate compression statistics across all benchmarks |
| GET | `/api/demo/session/:sessionId` | Retrieves details for a specific compression session |
| GET | `/api/demo/data/:sessionId` | Retrieves the decompressed data from a session |
| GET | `/api/demo/history` | Returns the history of all compression benchmark runs |
| GET | `/api/demo/files` | Lists all uploaded files and their compression results |

---

## 6. Kernel Crypto Modules

**Kong Service Name**: `plenumnet-crypto` (pending gateway sync)
**Base Path**: Served via Rust kernel (`src/kernel/`) and documented at `/api/salvi/docs`
**Status**: IMPLEMENTED (33 modules, CMVP submission ready)

### What it does (in plain terms)
This is the cryptographic engine at the heart of PlenumNET — a full set of security algorithms built from the ground up to resist attacks from quantum computers. While today's encryption (like RSA) could be broken by a future quantum computer, these algorithms are designed to remain secure even in that scenario. The U.S. government's CNSA 2.0 standard requires 11 specific algorithms for national security systems — PlenumNET implements all 11 at 100% coverage. The entire module is being submitted for FIPS 140-3 certification, which is the gold standard for cryptographic validation.

### Algorithm Coverage (11/11 CNSA 2.0)

| Algorithm | Standard | What It Does (Plain Terms) |
|-----------|----------|---------------------------|
| ML-KEM (TL-KEM) | FIPS 203 | Securely exchanges encryption keys even if an attacker has a quantum computer |
| ML-DSA (TL-DSA) | FIPS 204 | Creates digital signatures that prove who sent a message — quantum-proof |
| AES-256-GCM | FIPS 197 | The actual encryption that scrambles your data — military grade |
| SHA-384 | FIPS 180-4 | Creates a unique fingerprint of data to detect tampering |
| SHA-512 | FIPS 180-4 | Longer fingerprint for higher-security applications |
| SHA3-256 | FIPS 202 | Next-generation fingerprinting using a completely different math approach |
| SHA3-512 | FIPS 202 | Next-generation fingerprinting, extra-long version |
| HMAC-SHA-384 | FIPS 198-1 | Verifies both the identity of the sender and the integrity of the message |
| XMSS | SP 800-208 | Hash-based signatures that remain secure even against quantum attacks — for firmware signing |
| LMS | SP 800-208 | Lightweight hash-based signatures for signing software updates and certificates |
| ECDH P-384 | SP 800-56A | Securely agrees on a shared secret between two parties over a public channel |

### Module Count: 33 Kernel Modules

Key modules include: `ternary_lattice.rs` (NTT polynomial math), `signature.rs` (XMSS/LMS), `cipher.rs` (AES-256-GCM), `hash.rs` (SHA-2/SHA-3), `kem.rs` (TL-KEM), `dsa.rs` (TL-DSA), `phase_cnsa.rs` (hybrid key exchange), `ct_utils.rs` (constant-time protections), `firmware_sign.rs` (secure boot), `x509.rs` (certificate management), `agility.rs` (algorithm policy), `hmac_drbg.rs` (random number generation), `entropy.rs` (entropy source), `fips_post.rs` (power-on self-tests), `self_test.rs` (conditional testing), `state_machine.rs` (FIPS module states), `service_interface.rs` (role-based access).

---

## 7. Whitepaper Management

**Kong Service Name**: `plenumnet-whitepapers`
**Base Path**: `/api/whitepapers`
**Status**: LIVE
**Rate Limit**: 100/min, 1,000/hr

### What it does (in plain terms)
Manages PlenumNET's technical whitepapers — the detailed documents that explain how the technology works. Investors, partners, and developers can retrieve the current published whitepaper, browse all versions, or (if authorized) publish new ones.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/whitepapers` | Lists all published whitepapers |
| GET | `/api/whitepapers/active` | Returns the currently active (latest) whitepaper |
| GET | `/api/whitepapers/:id` | Retrieves a specific whitepaper by ID |
| POST | `/api/whitepapers` | Publishes a new whitepaper (authenticated) |

---

## 8. API Documentation

**Kong Service Name**: `plenumnet-docs`
**Base Path**: `/api/salvi/docs`
**Status**: LIVE
**Rate Limit**: 200/min, 2,000/hr

### What it does (in plain terms)
A self-describing documentation endpoint. When you call it, it returns a complete list of every available API endpoint, what data it expects, and what it returns. Think of it as a built-in instruction manual for the entire API — developers can read it to understand how to integrate with PlenumNET without needing separate documentation.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/docs` | Returns the full API schema with endpoint descriptions, request/response formats |

---

## 9. Developer Signup

**Kong Service Name**: `plenumnet-user` (sub-service)
**Base Path**: `/api/developer-signup`
**Status**: LIVE
**Rate Limit**: Shared with User (100/min)

### What it does (in plain terms)
Handles early-access signups from developers who want to integrate PlenumNET into their own applications. Captures interest, tracks signup volume, and provides an admin view for managing the waitlist.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/developer-signup` | Submits a new developer signup |
| GET | `/api/developer-signup/count` | Returns the total number of signups |

---

## 10. Authentication & User Management

**Kong Service Name**: `plenumnet-user`
**Base Path**: `/api/user`, `/api/auth`, `/api/login`, `/api/logout`
**Status**: LIVE
**Rate Limit**: 100/min, 1,000/hr

### What it does (in plain terms)
Handles user login and identity. PlenumNET uses Replit Auth, which supports sign-in via GitHub, Google, Apple, X, or email/password. Once logged in, the system knows who you are and what you're allowed to access. Admin users get elevated permissions for managing the platform.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/auth/user` | Returns the currently logged-in user's profile |
| GET | `/api/login` | Starts the login flow (redirects to identity provider) |
| GET | `/api/callback` | Handles the return from the identity provider after login |
| GET | `/api/logout` | Logs the user out and clears the session |
| GET | `/api/user/admin-status` | Checks whether the current user has admin privileges |

---

## 11. Admin Dashboard

**Kong Service Name**: `plenumnet-user` (sub-service, admin-gated)
**Base Path**: `/api/admin`
**Status**: LIVE (requires admin authentication)

### What it does (in plain terms)
Provides administrative controls for managing the platform — viewing developer signups, removing entries, and overseeing system status. Only accessible to users with admin privileges.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/developer-signups` | Lists all developer signup submissions |
| DELETE | `/api/admin/developer-signups/:id` | Removes a specific signup entry |

---

## 12. GitHub Integration

**Kong Service Name**: `plenumnet-github`
**Base Path**: `/api/github`
**Status**: LIVE (admin only)
**Rate Limit**: 60/min, 600/hr

### What it does (in plain terms)
Connects PlenumNET directly to its GitHub repository (SigmaWolf-8/Ternary). Admins can browse files, edit code, create new files, and push updates — all from within the PlenumNET dashboard, without needing to open a separate code editor. It also provides batch push operations for deploying entire stages of the kernel crypto modules to GitHub in one click.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/github/token` | Saves the GitHub Personal Access Token |
| GET | `/api/github/status` | Checks if the GitHub token is configured |
| GET | `/api/github/repos/:owner/:repo/branches` | Lists repository branches |
| GET | `/api/github/repos/:owner/:repo/contents` | Browses repository file tree |
| GET | `/api/github/file/:owner/:repo` | Reads a specific file's content |
| PUT | `/api/github/file/:owner/:repo` | Creates or updates a file |
| DELETE | `/api/github/file/:owner/:repo` | Deletes a file |
| POST | `/api/github/push-workflows/:owner/:repo` | Pushes all CI/CD workflow files |
| POST | `/api/github/push-batch/:owner/:repo` | Batch pushes allowlisted files (Stages 1-5, 36 files) |

---

## 13. Kong Gateway Management

**Kong Service Name**: `plenumnet-kong`
**Base Path**: `/api/kong`
**Status**: LIVE
**Rate Limit**: 60/min, 600/hr

### What it does (in plain terms)
This is Kong managing itself — the gateway's own control panel. It lets admins view the status of all API services, create new services and routes, attach security plugins, and deploy Kong data planes to cloud providers. Think of it as the control room for the entire API infrastructure.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/kong/status` | Checks connection to Kong Konnect cloud |
| GET | `/api/kong/organization` | Returns Kong organization details |
| GET | `/api/kong/control-planes` | Lists all Kong control planes |
| GET | `/api/kong/control-planes/:cpId/services` | Lists services in a control plane |
| GET | `/api/kong/control-planes/:cpId/routes` | Lists routes in a control plane |
| GET | `/api/kong/control-planes/:cpId/plugins` | Lists security plugins in a control plane |
| GET | `/api/kong/config` | Returns the local kong.yaml configuration |
| POST | `/api/kong/control-planes/:cpId/services` | Creates a new API service |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/routes` | Creates a route for a service |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/plugins` | Attaches a plugin to a service |
| POST | `/api/kong/control-planes/:cpId/sync-plenumnet` | Syncs all PlenumNET services to Kong |
| POST | `/api/kong/save-to-github` | Saves Kong config to GitHub repository |
| GET | `/api/kong/control-planes/:cpId/deploy-instructions` | Gets data plane deployment instructions |
| POST | `/api/kong/control-planes/:cpId/generate-deployment` | Generates deployment packages |
| POST | `/api/kong/control-planes/:cpId/deploy-to-cloud` | Deploys Kong to cloud (Render/Railway) |

---

## 14. CNSA 2.0 Compliance Enforcement

**Kong Service Name**: Global plugins (applied to all services)
**Status**: ACTIVE

### What it does (in plain terms)
These are automatic security headers that Kong injects into every single API request and response. They enforce that only quantum-resistant encryption algorithms are used, and they advertise PlenumNET's compliance status to every client that connects. Any attempt to use non-quantum-safe algorithms on crypto endpoints is blocked with a 403 error.

### Applied Globally

| Plugin | Direction | What It Does |
|--------|-----------|-------------|
| `request-transformer` | Inbound | Adds headers declaring CNSA 2.0 policy enforcement: ML-KEM-1024 for key exchange, ML-DSA-87 + XMSS + LMS for authentication, AES-256-GCM for encryption, SHA-384/512 for hashing |
| `response-transformer` | Outbound | Adds headers advertising full CNSA 2.0 compliance (11/11 algorithms), FIPS 203/204/SP 800-208 coverage, and HSTS with 2-year max-age |
| `request-termination` | Inbound (crypto) | Blocks non-CNSA cipher negotiation with 403: "only quantum-resistant algorithms permitted" |
| `correlation-id` | Both | Attaches a unique X-PlenumNET-Request-ID (UUID) to every request for distributed tracing |

---

## 15. Certification & Audit Trail

**Kong Service Name**: Part of `plenumnet-timing` service architecture
**Status**: DEFINED (interfaces ready, pending production deployment)

### What it does (in plain terms)
When a financial trade or critical operation needs a provably accurate timestamp, this service issues a formal certification. It captures the timing events, verifies they meet regulatory requirements (FINRA, MiFID II), produces a mathematical proof (Merkle tree), anchors it to a public blockchain (Hedera), and has it cryptographically signed. The result is an auditable, tamper-proof record that regulators can independently verify.

### Capabilities (from timing-service.ts)

| Capability | Description |
|------------|-------------|
| Timing Certification | Issues certified timestamps with configurable levels: financial, regulatory, standard, best_effort |
| Merkle Proof | Generates inclusion proofs for timestamp verification |
| Blockchain Anchoring | Anchors proofs to Hedera Hashgraph Consensus Service |
| Cryptographic Signatures | Signs certifications with Ed25519/RSA/ECDSA (ML-DSA planned) |
| Uncertainty Tracking | Reports total uncertainty and worst-case error in femtoseconds |

---

## 16. Payment Processing

**Kong Service Name**: Planned (`plenumnet-payments`)
**Status**: ARCHITECTURE DEFINED (microservices specification complete)

### What it does (in plain terms)
Handles payment processing for PlenumNET services. When a customer pays — whether by credit card (Stripe), Canadian bank transfer (Interac), or cryptocurrency — this service validates the payment, queues it for processing, and creates a verifiable record. Every payment is timestamped with femtosecond precision and witnessed on blockchain for regulatory compliance.

### Planned Architecture

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Payment Listener | Webhooks + BullMQ | Receives payment notifications and queues them |
| HMAC Validation | SHA-256/SHA-512 | Verifies payment notifications are authentic |
| Idempotency | UUID keys | Prevents duplicate payment processing |
| Settlement | XRPL | Settles payments on the XRP Ledger |
| Smart Contracts | Algorand | Executes programmable payment logic |

---

## 17. Blockchain Witnessing

**Kong Service Name**: Planned (`plenumnet-blockchain`)
**Status**: ARCHITECTURE DEFINED (service interfaces specified)

### What it does (in plain terms)
Creates permanent, tamper-proof records of important operations by writing them to public blockchains. Think of it as a digital notary — once something is witnessed on a blockchain, no one can deny it happened or change the record. PlenumNET uses three different blockchains for different purposes.

### Planned Architecture

| Blockchain | Service | Purpose |
|-----------|---------|---------|
| Hedera Hashgraph (HCS) | Consensus witnessing | Fast, low-cost proof that an event occurred at a specific time |
| XRP Ledger (XRPL) | Payment settlement | Settles financial transactions with cryptographic proof |
| Algorand | Smart contracts & oracle bridge | Executes automated agreements and bridges on-chain/off-chain data |

---

## 18. Health & Observability

**Kong Service Name**: Global (applied across all services)
**Status**: ACTIVE

### What it does (in plain terms)
Monitors the health and performance of every API service. Kong automatically tracks response times, error rates, and request volumes. Upstream health checks ping the server every 30 seconds to confirm it's alive, and automatically stop sending traffic if it detects failures. Every request gets a unique tracking ID so issues can be traced from start to finish.

### Components

| Component | Configuration | Purpose |
|-----------|--------------|---------|
| Rate Limiting | Per-service (50-200/min) | Prevents abuse and ensures fair usage |
| Health Checks | Active: 30s interval, 2 successes / 3 failures | Detects and routes around unhealthy servers |
| Request Size Limiting | 10 MB on demo service | Prevents oversized uploads from impacting performance |
| Correlation ID | UUID per request | End-to-end request tracing across all services |
| Response Rate Limiting | X-PlenumNET-RateLimit header | Communicates rate limit status to clients |
| Upstream Load Balancing | Round-robin | Distributes traffic across multiple server instances |

---

## API Consumer Tiers

Kong manages two API consumer tiers for programmatic access:

| Consumer | Tier | Access Level |
|----------|------|-------------|
| `ai-agent-default` | Standard | Public API endpoints with standard rate limits |
| `ai-agent-premium` | Premium | Elevated rate limits and priority routing |

---

## Kong Gateway Configuration Summary

| Category | Count |
|----------|-------|
| **API Service Groups** | 18 |
| **Individual Endpoints** | 70+ |
| **Kong-Registered Services** | 9 (gateway sync pending for remaining) |
| **Security Plugins** | 14 (rate limiting, CNSA enforcement, CORS, size limits) |
| **Global Plugins** | 4 (correlation-id, request-transformer, response-transformer, request-termination) |
| **API Consumers** | 2 tiers |
| **Upstreams** | 1 (with health checks) |
| **Calendar Endpoints** | 24 individual + 2 aggregate |
| **Ternary Operations** | 8 endpoints |
| **Crypto Algorithms** | 11/11 CNSA 2.0 |
| **Kernel Modules** | 33 |

---

## Next Steps: Kong Sync

The `kong.yaml` configuration currently defines 9 services. The following service groups should be synced to Kong to bring the gateway configuration to full coverage:

1. **Calendar Synchronization** — Separate service for `/api/salvi/timing/epoch/*` endpoints
2. **Kernel Crypto** — Service for crypto module status and algorithm queries
3. **Developer Signup** — Dedicated service with signup-specific rate limiting
4. **Admin Dashboard** — Admin-gated service with IP restriction plugins
5. **Certification** — Service for timing certification and audit trail endpoints
6. **Authentication** — Service for auth flow endpoints
7. **Payment Processing** — Service for payment webhook endpoints (when deployed)
8. **Blockchain Witnessing** — Service for blockchain anchoring endpoints (when deployed)
9. **Health/Observability** — Dedicated health check endpoint service

To sync, use the "Sync PlenumNET to Kong" button on the Kong Konnect admin page, or run:
```bash
deck gateway sync kong/kong.yaml --konnect-token $KONG_KONNECT_TOKEN
```
