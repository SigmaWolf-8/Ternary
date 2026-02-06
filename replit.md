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

## Rust Kernel Architecture (`src/kernel/`)

### Recent Changes (2026-02-06)
- Added complete memory subsystem (`src/kernel/src/memory/`)
- Added complete synchronization primitives (`src/kernel/src/sync/`)
- Added complete process management subsystem (`src/kernel/src/process/`)
- Added complete modal security system (`src/kernel/src/security/`)
- Added complete cryptographic primitives (`src/kernel/src/crypto/`)
- Integrated memory, sync, process, security, and crypto error types into `KernelError` with `From` conversions

### Kernel Modules
-   **Ternary Operations** (`src/kernel/src/ternary/`): GF(3) arithmetic, representation conversions (A/B/C), trit/tryte types.
-   **Timing** (`src/kernel/src/timing/`): Femtosecond-precision timestamps anchored to Salvi Epoch (April 1, 2025).
-   **Phase Encryption** (`src/kernel/src/phase/`): Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem** (`src/kernel/src/memory/`):
    - `allocator.rs`: Bitmap-based frame allocator (binary 4KiB + ternary 2187-byte pages) + bump allocator.
    - `page.rs`: Page table management with virtual-to-physical mapping, ternary security modes (ModePhi/ModeOne/ModeZero), and ternary-compute/phase-encrypted/timing-critical page flags.
    - `heap.rs`: Free-list heap allocator with best-fit strategy and coalescing.
-   **Synchronization** (`src/kernel/src/sync/`):
    - `spinlock.rs`: Ticket-based spinlock with FIFO fairness guarantees.
    - `mutex.rs`: Mutex with ternary security mode gating (caller must meet minimum mode level).
    - `semaphore.rs`: Counting semaphore for resource pool management.
    - `phase_mutex.rs`: Phase-encryption-aware mutex enforcing femtosecond timing windows per `EncryptionMode`.
-   **Process Management** (`src/kernel/src/process/`):
    - `mod.rs`: ProcessId, ProcessState (Created/Ready/Running/Blocked/Sleeping/Suspended/Terminated/Zombie), Priority levels (Idle through Critical), ProcessDescriptor with security mode and timing metadata.
    - `table.rs`: ProcessTable with creation, lookup, state transitions, parent-child tracking, zombie reaping, and security mode filtering.
    - `scheduler.rs`: Multi-level priority round-robin scheduler with idle process fallback, block/unblock, and priority changes.
    - `context.rs`: CPU context (general + control + ternary coprocessor registers), context switch descriptors with page table and ternary state flags.
    - `ipc.rs`: Message-passing IPC with typed messages (Data/Signal/TernaryPayload/PhaseComponent/TimingSync/Control), security-mode-gated channels, MessageBus with per-process channel tracking.
-   **Modal Security** (`src/kernel/src/security/`):
    - `domain.rs`: Security domain management with isolation policies, transition rules (audit/approval requirements), and domain-aware access control.
    - `capability.rs`: Capability-based access control with typed tokens (Read/Write/Execute/Admin/Custom), delegation chains, revocation, and expiration.
    - `audit.rs`: Audit trail with femtosecond timestamps, integrity checksums via ternary hashing, and append-only log with capacity management.
    - `policy.rs`: Priority-ordered security policy engine with mode restrictions, domain-scoped rules, and default-deny evaluation.
-   **Cryptographic Primitives** (`src/kernel/src/crypto/`):
    - `hash.rs`: Ternary hash using substitution-permutation network with rotation-based S-box (prevents zero-collapse) and bijective permutation (multiplier 376, gcd(376,729)=1).
    - `sponge.rs`: Sponge construction with absorb/squeeze phases and configurable capacity/rate.
    - `hmac.rs`: HMAC with constant-time comparison using XOR accumulator.
    - `kdf.rs`: Key derivation with iterated HMAC and salt mixing.
    - `signature.rs`: Proper ternary Lamport one-time signature (3 secrets per trit position, 81-trit message digest, one-time enforcement via `used` flag).
-   **Device Driver Framework** (`src/kernel/src/device/`):
    - `mod.rs`: Device types/states/capabilities with validated state transitions, DeviceDescriptor lifecycle.
    - `bus.rs`: Bus abstraction with hierarchy (parent/child), speed levels, device attachment, BusManager.
    - `registry.rs`: Device registry with register/init/suspend/resume/remove lifecycle management.
    - `interrupt.rs`: Interrupt controller with shared IRQs, priorities, enable/disable, masking, pending queue processing.
    - `dma.rs`: DMA controller with channel allocation, transfer lifecycle (configure/start/complete/abort), ownership enforcement.
-   **I/O Subsystem** (`src/kernel/src/io/`):
    - `scheduler.rs`: Priority-based I/O scheduler (4 levels: Low/Normal/High/Realtime) with submit/dispatch/complete/fail/cancel.
    - `buffer.rs`: Buffer cache with LRU-like eviction, dirty tracking, pinning, per-device invalidation.
    - `block.rs`: Block device layer with in-memory storage, read/write/trim, read-only enforcement, BlockDeviceManager.
    - `chardev.rs`: Character device layer with buffered read/write, mode enforcement (RO/WO/RW), feed/drain.
    - `poll.rs`: I/O multiplexing with poll set, event filtering, error/hangup always reported.
-   **Filesystem** (`src/kernel/src/fs/`):
    - `inode.rs`: Inode management with types (File/Directory/Symlink/Device), permissions, link counting, InodeTable with capacity.
    - `dir.rs`: Directory operations with entries, lookup (./..​), rename, name validation, DirectoryTable.
    - `file.rs`: File operations (open/close/read/write/seek/truncate), mode enforcement, multiple FD support, FileTable.
    - `mount.rs`: Mount system with path-based lookup, filesystem types (TernaryFs/RamFs/DevFs/ProcFs/SysFs), longest-prefix matching, MountTable.
-   **Error Handling** (`src/kernel/src/error.rs`): Unified `KernelError` enum with `From` conversions for all subsystems (memory, sync, process, security, crypto, device, io, fs).

### Gap Closure Roadmap Progress
- Completed: P0-005, P0-006, P1-001 through P1-041 (42 of 65 tasks)
- Next priorities: P1-042+ (Networking, Syscall Interface, Platform HAL)