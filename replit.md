# PlenumNET Framework Marketing Website

## Overview
PlenumNET (formerly Salvi) is a post-quantum internet solutions company. This project is a professional, light-themed marketing website showcasing PlenumNET's deployable components for building quantum-resistant infrastructure. Key features include a PlenumDB product page with a live compression demo and comprehensive whitepaper management.

The project has expanded to include a complete payment processing and blockchain witnessing architecture, integrating high-precision timing and regulatory compliance for financial transactions. This aims to provide robust, secure, and verifiable operations for quantum-resistant data and financial services.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend is built with React, TypeScript, Tailwind CSS, and Framer Motion. It uses `shadcn/ui` for UI components and Wouter for routing. The design adheres to a light theme with a white background and blue accents.

**Core Pages:**
-   **Landing Page (`/`)**: Main entry point with sections on PlenumNET's approach, components, market potential, and call to action.
-   **PlenumDB Product Page (`/ternarydb`)**: Features a live compression demo.
-   **Whitepaper Viewer (`/whitepaper`)**: Displays the full whitepaper with a table of contents.
-   **GitHub Manager (`/github`)**: An admin-only interface for managing GitHub repository files, including sorting, filtering, and metrics.
-   **Kong Konnect Integration (`/kong-konnect`)**: A page to manage API gateway integration with Kong Konnect.

### Backend and Core Framework
The backend uses Express.js and Node.js, with PostgreSQL and Drizzle ORM for database management.

**PlenumNET Core API (`/api/salvi/`)**:
-   **Ternary Operations**: Implements the Unified Ternary Logic System for operations like conversion, addition, multiplication, rotation, negation, and XOR in GF(3).
-   **Femtosecond Timing**: Provides high-precision timestamping and timing metrics.
-   **Phase Encryption**: Handles splitting and recombining data into phase-encrypted components.

**Ternary Representations:**
-   **A (Computational)**: `{-1, 0, +1}`
-   **B (Network)**: `{0, 1, 2}`
-   **C (Human)**: `{1, 2, 3}`

**Microservices Architecture (Payment & Witnessing)**:
The system now includes several microservices for robust payment processing and blockchain integration:
-   **Payment Listener (Port 3001)**: Handles payment webhooks (Stripe, Interac, Crypto) with HMAC signature validation and queues payments using BullMQ.
-   **SFK Core API (Port 3002)**: An orchestration layer for operations, built with Fastify, providing CRUD operations and internal blockchain integration routes. Includes Swagger/OpenAPI documentation.
-   **Blockchain Services**: Dedicated services for Hedera HCS (witnessing), XRPL (payment settlement), and Algorand (smart contract execution).
-   **Femtosecond Timing Service (Port 3006)**: Provides high-precision timing synchronized via HPTP, supporting clock sources like GPSDOs and atomic clocks.
-   **Certification Service (Port 3007)**: Ensures regulatory compliance (FINRA 613, MiFID II) for timestamps, providing certification and verification.

**Blockchain Integration Details:**
-   **Hedera HCS**: For immutable audit trails and femtosecond-precision witnessing.
-   **XRPL**: For real-time, cross-border payment settlement.
-   **Algorand**: For smart contracts (e.g., Ternary Governance, Reward Distribution) and an oracle bridge to Hedera.

**Security Features:**
-   HMAC-SHA256/SHA512 signature validation for webhooks.
-   `crypto.timingSafeEqual` for constant-time comparisons.
-   Idempotency keys and rate limiting.

### Database Schema
The database includes tables for:
-   `users`: Authenticated users with roles and GitHub token.
-   `sessions`: User session management.
-   `demo_sessions`: Metadata for compression demos.
-   `binary_storage`, `ternary_storage`: Original and compressed data.
-   `compression_benchmarks`, `compression_history`: Performance and historical data.
-   `whitepapers`: Whitepaper content.

## External Dependencies

-   **Authentication**: Replit Auth (GitHub, Google, Apple, X, email/password).
-   **Database**: PostgreSQL.
-   **ORM**: Drizzle ORM.
-   **API Gateway**: Kong Konnect (for API management, rate limiting, security, and deployment orchestration).
-   **Payment Gateways**: Stripe, Interac, various cryptocurrency platforms (via webhooks).
-   **Message Queue**: BullMQ.
-   **Blockchain Platforms**: Hedera Hashgraph Consensus Service (HCS), XRP Ledger (XRPL), Algorand.
-   **Containerization**: Docker (for microservices deployment).
-   **Cloud Deployment**: Render, Railway (via GitHub integration for CI/CD).