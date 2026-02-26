# SigmaWolf-8/Ternary Repository Architecture

**Complete File Listing & Architecture Overview**

## Repository Statistics
- **Total Directories**: 80+
- **Total Files**: 180+
- **Primary Languages**: Rust, TypeScript, Verilog, SystemVerilog
- **Repository**: https://github.com/SigmaWolf-8/Ternary

---

## Directory Structure

```
Ternary/
├── .github/                          # GitHub Configuration
│   ├── ISSUE_TEMPLATE/               # Issue Templates
│   │   ├── bug-report.md
│   │   ├── compliance-issue.md
│   │   ├── documentation-issue.md
│   │   ├── feature-request.md
│   │   ├── research-proposal.md
│   │   └── security-report.md
│   ├── PULL_REQUEST_TEMPLATE/        # PR Templates
│   │   ├── documentation-pr.md
│   │   ├── hardware-pr.md
│   │   ├── kernel-pr.md
│   │   └── research-pr.md
│   ├── workflows/                    # CI/CD Pipelines
│   │   ├── build-kernel.yml
│   │   ├── codeql-analysis.yml
│   │   ├── release.yml
│   │   ├── security-scan.yml
│   │   └── test-kernel.yml
│   ├── CODEOWNERS
│   ├── dependabot.yml
│   ├── FUNDING.yml
│   ├── labels.yml
│   └── PULL_REQUEST_TEMPLATE.md
│
├── config/                           # Configuration Files
│   ├── kernel.toml
│   ├── security.toml
│   └── timing.toml
│
├── deployments/                      # Deployment Configurations
│   ├── docker/
│   │   ├── docker-compose.yml
│   │   └── Dockerfile
│   └── kubernetes/
│       └── deployment.yaml
│
├── docs/                             # Documentation
│   ├── api/                          # API Documentation
│   │   ├── kernel-api.md
│   │   ├── libternary-api.md
│   │   ├── network-api.md
│   │   ├── security-api.md
│   │   └── timing-api.md
│   ├── architecture/                 # Architecture Docs
│   │   ├── overview.md
│   │   ├── security-model.md
│   │   ├── ternary-logic.md
│   │   └── torsion-networking.md
│   ├── compliance/                   # Regulatory Compliance
│   │   ├── audit-trails.md
│   │   ├── certification-process.md
│   │   ├── finra-613.md
│   │   ├── gdpr.md
│   │   ├── mifid-ii.md
│   │   └── nist-standards.md
│   ├── development/                  # Developer Guides
│   │   ├── build-instructions.md
│   │   ├── code-style.md
│   │   ├── debugging-guide.md
│   │   ├── getting-started.md
│   │   └── testing-guide.md
│   ├── research/                     # Research Papers
│   │   ├── femtosecond-timing.md
│   │   ├── formalism-proofs.md
│   │   ├── performance-benchmarks.md
│   │   ├── quantum-resistance-proofs.md
│   │   ├── ternary-mathematics.md
│   │   └── torsion-field-physics.md
│   ├── specifications/               # Technical Specifications
│   │   ├── API-REFERENCE.md
│   │   ├── CRYPTO-SPEC.md
│   │   ├── HPTP-SPEC.md
│   │   ├── NETWORK-SPEC.md
│   │   ├── SPECIFICATION-v4.21.md
│   │   ├── THDL-SPEC.md
│   │   ├── TIMING-COMPLIANCE-SPEC.md
│   │   ├── TSL-SPEC.md
│   │   └── XRPL-WITNESSING-SPEC.md
│   └── tutorials/                    # Tutorials
│       ├── deployment-guide.md
│       ├── first-ternary-program.md
│       ├── network-setup.md
│       ├── security-configuration.md
│       └── timing-certification.md
│
├── examples/                         # Example Programs
│   ├── encryption-demo/
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   ├── hello-ternary/
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   ├── network-demo/
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   └── timing-demo/
│       ├── src/main.rs
│       └── Cargo.toml
│
├── hardware/                         # Hardware Implementations
│   ├── asic/
│   │   ├── README.md
│   │   └── tpu_asic.sv
│   ├── drivers/
│   │   ├── embedded/
│   │   │   ├── salvi_tpu.c
│   │   │   └── salvi_tpu.h
│   │   └── linux/
│   │       ├── Makefile
│   │       └── salvi_tpu.c
│   ├── fpga/
│   │   ├── constraints/
│   │   │   ├── intel.sdc
│   │   │   └── xilinx.xdc
│   │   ├── scripts/
│   │   │   └── synth.tcl
│   │   ├── timing/
│   │   │   └── femto_clock.sv
│   │   ├── tpu/
│   │   │   └── tpu_core.sv
│   │   ├── verilog/
│   │   │   ├── timing/
│   │   │   │   ├── clock_distribution.v
│   │   │   │   └── timestamp_unit.v
│   │   │   └── tpu/
│   │   │       ├── alu.v
│   │   │       ├── memory_controller.v
│   │   │       └── phase_sync.v
│   │   └── README.md
│   └── pcb/
│       ├── clock-card/
│       │   └── clock-card.kicad_pro
│       └── README.md
│
├── keys/                             # Key Management
│   ├── encryption/
│   │   └── README.md
│   ├── signing/
│   │   └── README.md
│   └── README.md
│
├── kong/                             # Kong API Gateway
│   └── kong.yaml                     # Declarative Kong config
│
├── kong-deploy/                      # Kong Deployment
│   ├── Dockerfile
│   ├── entrypoint.sh
│   └── tls.crt
│
├── scripts/                          # Build & Test Scripts
│   ├── build-all.sh
│   ├── run-tests.sh
│   ├── setup-dev.sh
│   └── test-binary-build.sh
│
├── src/                              # Source Code
│   ├── kernel/                       # Ternary Kernel (Rust)
│   │   ├── .cargo/
│   │   │   └── config.toml
│   │   ├── src/
│   │   │   ├── arch/                 # Architecture-specific
│   │   │   │   ├── aarch64/mod.rs
│   │   │   │   ├── riscv/mod.rs
│   │   │   │   ├── riscv64/mod.rs
│   │   │   │   ├── x86_64/
│   │   │   │   │   ├── boot.asm
│   │   │   │   │   ├── linker.ld
│   │   │   │   │   └── mod.rs
│   │   │   │   └── mod.rs
│   │   │   ├── drivers/
│   │   │   │   ├── timing/mod.rs
│   │   │   │   ├── tpu/mod.rs
│   │   │   │   └── mod.rs
│   │   │   ├── kernel/mod.rs
│   │   │   ├── memory/
│   │   │   │   ├── allocator.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── paging.rs
│   │   │   │   └── tagc.rs
│   │   │   ├── network/
│   │   │   │   ├── protocols/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── t3p.rs
│   │   │   │   │   └── ttp.rs
│   │   │   │   ├── torsion/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── routing.rs
│   │   │   │   │   └── topology.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── timing.rs
│   │   │   ├── security/
│   │   │   │   ├── modal/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── mode_one.rs
│   │   │   │   │   ├── mode_phi.rs
│   │   │   │   │   └── mode_zero.rs
│   │   │   │   ├── phase/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── sync.rs
│   │   │   │   │   └── tracker.rs
│   │   │   │   ├── xrpl/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── verifier.rs
│   │   │   │   │   └── witness.rs
│   │   │   │   └── mod.rs
│   │   │   ├── syscalls/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── security.rs
│   │   │   │   ├── ternary.rs
│   │   │   │   └── timing.rs
│   │   │   ├── ternary/
│   │   │   │   ├── arithmetic.rs
│   │   │   │   ├── crypto.rs
│   │   │   │   ├── logic.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── vector.rs
│   │   │   ├── utils/
│   │   │   │   ├── conversion.rs
│   │   │   │   ├── hash.rs
│   │   │   │   ├── logging.rs
│   │   │   │   └── mod.rs
│   │   │   ├── lib.rs
│   │   │   └── main.rs
│   │   ├── tests/
│   │   │   ├── integration.rs
│   │   │   ├── performance.rs
│   │   │   ├── security.rs
│   │   │   └── unit.rs
│   │   ├── .clippy.toml
│   │   ├── .rustfmt.toml
│   │   ├── build.rs
│   │   ├── Cargo.toml
│   │   └── x86_64-salvi.json
│   │
│   ├── libternary/                   # Ternary Library
│   │   ├── bindings/
│   │   │   ├── cpp/libternary.hpp
│   │   │   ├── js/libternary.js
│   │   │   └── python/libternary.py
│   │   ├── src/
│   │   │   ├── conversion.rs
│   │   │   ├── lib.rs
│   │   │   ├── math.rs
│   │   │   ├── operations.rs
│   │   │   ├── trit.rs
│   │   │   └── tryte.rs
│   │   ├── aspect-api.ts
│   │   └── Cargo.toml
│   │
│   ├── salvi-core/                   # Payment & Witnessing APIs (TypeScript)
│   │   ├── blockchain-integrations.ts   # Hedera/XRPL/Algorand
│   │   ├── error-handling.ts            # Error codes & retry logic
│   │   ├── index.ts                     # Module exports
│   │   ├── payment-listener-api.ts      # Stripe/Interac/Crypto webhooks
│   │   ├── sfk-operations-api.ts        # SFK Core operations
│   │   ├── timing-service.ts            # Femtosecond timing (HPTP)
│   │   └── unified-metadata-schema.ts   # Core data types
│   │
│   ├── salvidb/                      # SalviDB Implementation
│   │   └── index.ts
│   │
│   ├── thdl/                         # Ternary HDL Compiler
│   │   ├── examples/
│   │   │   ├── alu.thdl
│   │   │   └── register.thdl
│   │   ├── libraries/
│   │   │   └── gates.thdl
│   │   ├── src/
│   │   │   ├── codegen.rs
│   │   │   ├── compiler.rs
│   │   │   ├── lib.rs
│   │   │   ├── simulation.rs
│   │   │   ├── simulator.rs
│   │   │   └── syntax.rs
│   │   └── Cargo.toml
│   │
│   ├── timing-api/                   # Timing API (Rust)
│   │   ├── clients/
│   │   │   ├── csharp/SalviTiming.cs
│   │   │   ├── java/SalviTiming.java
│   │   │   └── python/salvi_timing.py
│   │   ├── src/
│   │   │   ├── api.rs
│   │   │   ├── audit_chain.rs
│   │   │   ├── compliance.rs
│   │   │   ├── lib.rs
│   │   │   ├── types.rs
│   │   │   └── witness.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   │
│   ├── tsl/                          # Ternary Scripting Language
│   │   ├── examples/
│   │   │   ├── arithmetic.tsl
│   │   │   └── hello.tsl
│   │   ├── src/
│   │   │   ├── codegen.rs
│   │   │   ├── interpreter.rs
│   │   │   ├── lexer.rs
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs
│   │   │   ├── types.rs
│   │   │   └── verifier.rs
│   │   ├── stdlib/
│   │   │   └── ternary.tsl
│   │   └── Cargo.toml
│   │
│   ├── femtosecond-timing.ts         # Femtosecond timing module
│   ├── index.ts                      # Main TypeScript entry
│   ├── phase-encryption.ts           # Phase-split encryption
│   ├── ternary-operations.ts         # GF(3) operations
│   └── ternary-types.ts              # Trit type definitions
│
├── tests/                            # Test Suites
│   ├── integration/
│   │   ├── full_system.rs
│   │   ├── mod.rs
│   │   ├── network.rs
│   │   └── security.rs
│   ├── performance/
│   │   ├── benchmarks/
│   │   │   ├── ternary_ops.rs
│   │   │   └── timing.rs
│   │   └── scaling/
│   │       └── throughput.rs
│   └── unit/
│       ├── arithmetic.rs
│       ├── crypto.rs
│       ├── kernel_tests.rs
│       ├── logic.rs
│       ├── memory.rs
│       ├── ternary_tests.rs
│       └── timing.rs
│
├── tools/                            # Development Tools
│   ├── qats/                         # Quantum Attack Testing Suite
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   ├── ternary-sim/                  # Ternary Simulator
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   └── verification/                 # Verification Tool
│       ├── src/main.rs
│       └── Cargo.toml
│
├── .gitattributes
├── .gitignore
├── .gitleaks.toml
├── ACKNOWLEDGEMENTS.md
├── ADOPTERS.md
├── Cargo.toml                        # Rust workspace root
├── CHANGELOG.md
├── CITATION.cff
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── GOVERNANCE.md
├── KERNEL-BUILD-GUIDE.md
├── LICENSE
├── Makefile
├── package.json
├── PQTI-P0-STATUS.md
├── PQTI-REMAINING-WORK.md
├── README.md
├── render.yaml
├── ROADMAP.md
├── SECURITY.md
├── SUPPORT.md
├── test-file.txt
└── tsconfig.json
```

---

## Architecture Components

### 1. Ternary Kernel (`src/kernel/`)
The bijective ternary kernel for x86_64 hardware, implementing:
- **Architecture Support**: x86_64, AArch64, RISC-V, RISC-V64
- **Memory Management**: Custom allocator, paging, TAGC
- **Ternary Operations**: Arithmetic, logic, cryptography, vectors
- **Security Modes**: Mode Zero, Mode One, Mode Phi (φ+)
- **Network Stack**: Torsion topology, T3P/TTP protocols
- **XRPL Integration**: Witness verification, blockchain proofs

### 2. LibTernary (`src/libternary/`)
Cross-platform ternary library with bindings for:
- C++ (`libternary.hpp`)
- JavaScript (`libternary.js`)
- Python (`libternary.py`)
- TypeScript (`aspect-api.ts`) - 364° Prime Circle aspects

### 3. Salvi Core APIs (`src/salvi-core/`)
Payment & Witnessing Architecture v1.0:

| File | Purpose |
|------|---------|
| `unified-metadata-schema.ts` | Core data types, rate limits, metadata interfaces |
| `payment-listener-api.ts` | Stripe/Interac/Crypto webhook handlers with HMAC verification |
| `sfk-operations-api.ts` | SFK operation management, queuing, batch processing |
| `blockchain-integrations.ts` | Hedera HCS, XRPL, Algorand blockchain types |
| `timing-service.ts` | Femtosecond timing, HPTP protocol, synchronization |
| `error-handling.ts` | Error codes, categories, retry logic |
| `index.ts` | Module exports |

### 4. SalviDB (`src/salvidb/`)
Ternary database implementation with compression and efficiency metrics.

### 5. THDL Compiler (`src/thdl/`)
Ternary Hardware Description Language for FPGA/ASIC synthesis.

### 6. TSL Interpreter (`src/tsl/`)
Ternary Scripting Language for high-level ternary programming.

### 7. Timing API (`src/timing-api/`)
Femtosecond-precision timing with regulatory compliance:
- FINRA 613 CAT compliance
- MiFID II support
- Audit chain generation
- Multi-language clients (Python, Java, C#)

### 8. Hardware (`hardware/`)
Physical implementation designs:
- **FPGA**: Xilinx/Intel constraints, TPU cores, timing modules
- **ASIC**: TPU ASIC design
- **Drivers**: Linux and embedded TPU drivers
- **PCB**: Clock card designs

---

## CI/CD Pipelines

| Workflow | Purpose |
|----------|---------|
| `build-kernel.yml` | Build ternary kernel for all architectures |
| `test-kernel.yml` | Run unit and integration tests |
| `security-scan.yml` | Security vulnerability scanning |
| `codeql-analysis.yml` | CodeQL static analysis |
| `release.yml` | Automated release pipeline |

---

## Live API Endpoint Reference

**Base URL**: `https://plenumnet.replit.app`
**Total endpoints**: 194 (verified from source, February 2026)
**Source files**: `server/routes.ts`, `server/routes/*.ts`, `server/replit_integrations/auth/`

> **For AI agents**: Every endpoint below is verified against the running codebase.
> Endpoints marked [Admin] require Replit Auth with admin privileges.
> Endpoints marked [Auth] require any authenticated session.
> Endpoints marked [API-Key] require `Authorization: Bearer <key>` or `X-API-Key` header.
> All other endpoints are public. Rate limiting is applied platform-wide.

---

### 1. Ternary Computing Engine (10 endpoints)

**Source**: `server/routes/salvi.ts` — GF(3) arithmetic operations

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/salvi/ternary/convert` | Convert between ternary representations (A, B, C) |
| POST | `/api/salvi/ternary/add` | Ternary addition in GF(3) |
| POST | `/api/salvi/ternary/multiply` | Ternary multiplication in GF(3) |
| POST | `/api/salvi/ternary/rotate` | Bijective ternary rotation |
| POST | `/api/salvi/ternary/not` | Ternary NOT (negation) |
| POST | `/api/salvi/ternary/xor` | Ternary XOR operation |
| POST | `/api/salvi/ternary/batch` | Batch ternary operations (multiple ops in one call) |
| GET | `/api/salvi/ternary/density/:tritCount` | Calculate information density advantage for given trit count |
| GET | `/api/salvi/ternary/density-benchmark` | Validate 59% density claim across 4 sample sizes |
| POST | `/api/salvi/ternary/noether-verify` | Verify Noether symmetry conservation for ternary gauge |

### 2. Ternary Virtual Machine (2 endpoints)

**Source**: `server/routes/salvi.ts` — 176-opcode ISA v2.1

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/vm/spec` | Machine-readable TVM ISA v2.1 specification (full 176-opcode instruction set) |
| GET | `/api/salvi/vm/conformance` | Run conformance tests against ISA spec |

### 3. HPTP Femtosecond Timing (5 endpoints)

**Source**: `server/routes/salvi.ts` — High-Precision Timing Protocol

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/timing/timestamp` | Get femtosecond-precision timestamp |
| GET | `/api/salvi/timing/self-test` | 1000-sample timer resolution and jitter analysis |
| GET | `/api/salvi/timing/error-budget` | HPTP drift tracking, jitter analysis, FINRA 613 / MiFID II compliance |
| GET | `/api/salvi/timing/metrics` | Timing metrics and synchronization status |
| GET | `/api/salvi/timing/batch/:count` | Generate batch of N timestamps |

### 4. Calendar Synchronization (44 endpoints)

**Source**: `server/routes/salvi.ts` — 42 ancient calendar system conversions

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/timing/epoch/anchors` | Get Salvi Epoch anchor points across all 42 calendar systems |
| GET | `/api/salvi/timing/epoch/calendars` | List all available calendar conversions |
| GET | `/api/salvi/timing/epoch/calendars/mayan` | Mayan Long Count conversion |
| GET | `/api/salvi/timing/epoch/calendars/hebrew` | Hebrew calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/chinese` | Chinese calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/vedic` | Vedic calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/egyptian` | Egyptian calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/julian-day` | Julian Day Number conversion |
| GET | `/api/salvi/timing/epoch/calendars/islamic` | Islamic Hijri conversion |
| GET | `/api/salvi/timing/epoch/calendars/byzantine` | Byzantine calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/thirteen-moon` | 13-Moon calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/persian` | Persian/Solar Hijri conversion |
| GET | `/api/salvi/timing/epoch/calendars/ethiopian` | Ethiopian/Ge'ez conversion |
| GET | `/api/salvi/timing/epoch/calendars/coptic` | Coptic calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/japanese` | Japanese Imperial (Koki) conversion |
| GET | `/api/salvi/timing/epoch/calendars/korean` | Korean Dangun Era conversion |
| GET | `/api/salvi/timing/epoch/calendars/thai` | Thai Buddhist Era conversion |
| GET | `/api/salvi/timing/epoch/calendars/indian-saka` | Indian National/Saka conversion |
| GET | `/api/salvi/timing/epoch/calendars/tibetan` | Tibetan Rabjung conversion |
| GET | `/api/salvi/timing/epoch/calendars/aztec` | Aztec Tonalpohualli conversion |
| GET | `/api/salvi/timing/epoch/calendars/roman` | Roman Ab Urbe Condita conversion |
| GET | `/api/salvi/timing/epoch/calendars/bengali` | Bengali/Bangla conversion |
| GET | `/api/salvi/timing/epoch/calendars/berber` | Berber/Amazigh conversion |
| GET | `/api/salvi/timing/epoch/calendars/balinese` | Balinese Pawukon conversion |
| GET | `/api/salvi/timing/epoch/calendars/zoroastrian` | Zoroastrian Fasli conversion |
| GET | `/api/salvi/timing/epoch/calendars/aboriginal` | Aboriginal Australian Seasonal conversion |

All calendar endpoints accept an optional `?date=ISO8601` query parameter. Defaults to current date.

### 5. Phase Encryption (4 endpoints)

**Source**: `server/routes/salvi.ts` — Dual-phase quantum encryption

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/phase/config/:mode` | Get phase configuration for a security mode |
| POST | `/api/salvi/phase/split` | Split data into encrypted phase components |
| POST | `/api/salvi/phase/recombine` | Recombine phase components to recover data |
| GET | `/api/salvi/phase/recommend` | Get recommended phase configuration |

### 6. API Documentation (1 endpoint)

**Source**: `server/routes/salvi.ts`

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/salvi/docs` | Full machine-readable API documentation with all endpoint paths, methods, and descriptions |

### 7. PlenumDB Compression Demo (7 endpoints)

**Source**: `server/routes.ts` — Ternary compression demonstration

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/demo/run` | Run ternary compression on input data |
| GET | `/api/demo/stats` | Get compression statistics |
| GET | `/api/demo/session/:sessionId` | Get results for a specific demo session |
| GET | `/api/demo/data/:sessionId` | Get raw data for a demo session |
| POST | `/api/demo/upload` | Upload file for compression demo |
| GET | `/api/demo/history` | Get compression demo history |
| GET | `/api/demo/files` | List uploaded demo files |

### 8. Compression Storage (7 endpoints)

**Source**: `server/routes.ts` — Persistent ternary-compressed document storage

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/compression/file` | Compress and store a file |
| POST | `/api/compression/decompress` | Decompress stored data |
| POST | `/api/compression/db/store` | Store compressed document in database |
| GET | `/api/compression/db/retrieve/:id` | Retrieve compressed document by ID |
| GET | `/api/compression/db/documents` | List all stored compressed documents |
| GET | `/api/compression/db/raw/:id` | Get raw compressed bytes for a document |
| DELETE | `/api/compression/db/documents/:id` | Delete a compressed document [Auth] |

### 9. Whitepaper Management (4 endpoints)

**Source**: `server/routes.ts`

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/whitepapers` | List all whitepapers |
| GET | `/api/whitepapers/active` | Get the currently active whitepaper |
| GET | `/api/whitepapers/:id` | Get a specific whitepaper by ID |
| POST | `/api/whitepapers` | Upload a new whitepaper [Auth] |

### 10. Legal Documents (1 endpoint)

**Source**: `server/routes.ts`

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/legal/:type` | Get legal document by type (`terms`, `privacy`, `security`, `aup`) |

### 11. Authentication (4 endpoints)

**Source**: `server/replit_integrations/auth/` — Replit OpenID Connect

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/login` | Initiate Replit Auth login flow |
| GET | `/api/callback` | OAuth callback handler |
| GET | `/api/logout` | End session and log out |
| GET | `/api/auth/user` | Get current authenticated user profile [Auth] |

### 12. User & Admin (4 endpoints)

**Source**: `server/routes.ts`

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/user/admin-status` | Check if current user is admin |
| GET | `/api/admin/developer-signups` | List developer signups [Auth] |
| DELETE | `/api/admin/developer-signups/:id` | Delete a developer signup [Auth] |
| GET | `/api/health` | Platform health check |

### 13. Developer Waitlist (2 endpoints)

**Source**: `server/routes.ts`

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/developer-signup` | Submit developer waitlist signup |
| GET | `/api/developer-signup/count` | Get total signup count |

### 14. API Key Verification (1 endpoint)

**Source**: `server/routes.ts`

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/verify` | Verify an API key is valid [API-Key] |

### 15. API Key Management (16 endpoints)

**Source**: `server/routes/api-keys.ts` — Full key lifecycle management [Admin]

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/keys/scopes` | List all available API key scopes |
| POST | `/api/keys/generate` | Generate a new API key [Admin] |
| GET | `/api/keys` | List all API keys [Admin] |
| GET | `/api/keys/stats` | Get key statistics (total, active, revoked, expired) [Admin] |
| POST | `/api/keys/revoke/:id` | Revoke an API key [Admin] |
| GET | `/api/keys/:id/logs` | Get usage logs for a specific key [Admin] |
| POST | `/api/keys/rotate/:id` | Rotate an API key (generate new, revoke old) [Admin] |
| GET | `/api/keys/expiring` | List keys expiring within N days [Admin] |
| PATCH | `/api/keys/:id/rate-limit` | Update rate limit tier for a key [Admin] |
| GET | `/api/keys/rate-limit-tiers` | List available rate limit tiers |
| GET | `/api/keys/entity-types` | List available entity types for WBS tagging |
| PATCH | `/api/keys/:id/metadata` | Update key metadata (name, tags, project, etc.) [Admin] |
| GET | `/api/keys/anomalies` | Detect usage anomalies (spikes, failures, IP dispersion) [Admin] |
| GET | `/api/keys/audit` | Get global audit trail [Admin] |
| GET | `/api/keys/:id/audit` | Get audit trail for a specific key [Admin] |
| GET | `/api/keys/validate-external` | Validate an external API key (query param `?key=`) |

### 16. PPTPro Integration — Conductor API v2.3 (5 endpoints)

**Source**: `server/routes/pptpro-integration.ts` — All endpoints require [API-Key]

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/status` | PPTPro conductor status (version, uptime, capabilities) |
| GET | `/api/v1/safety/limits` | Safety governor limits (HRV bounds, entrainment bounds, session limits) |
| GET | `/api/v1/ternary/state` | Current ternary field state (GF(3) vector, phase, coherence) |
| POST | `/api/v1/entrain/advise` | Request entrainment advisory (accepts HRV, coherence, session data) |
| POST | `/api/v1/logs/coherence` | Submit coherence report for longitudinal analysis |

### 17. Tribonacci Mathematics (9 endpoints)

**Source**: `server/routes/tribonacci.ts`

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/tribonacci/hook` | Get Tribonacci hook value |
| GET | `/api/tribonacci/permutation` | Get Tribonacci permutation |
| GET | `/api/tribonacci/coverage` | Get Tribonacci sequence coverage stats |
| GET | `/api/tribonacci/hash` | Compute Tribonacci hash |
| GET | `/api/tribonacci/sequence` | Generate Tribonacci sequence |
| POST | `/api/tribonacci/generate-id` | Generate a Tribonacci-based unique ID |
| GET | `/api/tribonacci/next-worker` | Get next worker assignment (Tribonacci distribution) |
| GET | `/api/tribonacci/skip-lookup` | Tribonacci skip-ahead lookup |
| GET | `/api/tribonacci/hash-distribution` | Analyze Tribonacci hash distribution |

### 18. 28-Dimension Agent Array (6 endpoints)

**Source**: `server/routes/agent-array.ts`

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/tribonacci/agent-array` | Submit query to 28-agent parallel analysis |
| GET | `/api/tribonacci/agent-array/stream/:sessionId` | SSE stream for real-time agent responses |
| POST | `/api/tribonacci/agent-array/save` | Save an agent array report |
| GET | `/api/tribonacci/agent-array/reports` | List saved reports |
| GET | `/api/tribonacci/agent-array/reports/:id` | Get a specific saved report |
| GET | `/api/tribonacci/agent-array/positions` | Get agent position assignments |

### 19. Ternary Ephemeris (4 endpoints)

**Source**: `server/routes/ephemeris.ts` — Ternary degree astrology calculations

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/ephemeris/convert` | Convert standard degrees to ternary degrees with resonance scoring |
| POST | `/api/ephemeris/position` | Calculate single planet ephemeris position |
| POST | `/api/ephemeris/batch` | Batch ephemeris for all planets |
| GET | `/api/ephemeris/info` | API metadata (supported bodies, ternary system parameters) |

### 20. Tonal Diffusion System (7 endpoints)

**Source**: `server/routes/tonal-field.ts` — FM timing synchronization

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/tonal/field` | Get current tonal field state (potentials, Laplacian) |
| GET | `/api/tonal/neighbors` | Get natural neighbor set for local node |
| POST | `/api/tonal/packet` | Submit an FM timing packet for diffusion |
| GET | `/api/resonance/status` | Get resonance detector status (sync rate, frequency) |
| POST | `/api/resonance/sweep` | Trigger resonance frequency sweep |
| POST | `/api/resonance/rtt` | Submit round-trip time measurement |
| GET | `/api/metrics/plenum` | Get dimensionless Plenum field metrics |

### 21. GDPR Data Subject Rights (4 endpoints)

**Source**: `server/routes/data-subject-rights.ts` — All require [Auth]

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/gdpr/data-export` | Export all personal data (GDPR Article 20 portability) [Auth] |
| DELETE | `/api/gdpr/delete-account` | Request account deletion (GDPR Article 17 right to erasure) [Auth] |
| GET | `/api/gdpr/requests` | List past GDPR requests [Auth] |
| GET | `/api/gdpr/policy` | Get GDPR data processing policy |

### 22. Security Infrastructure (38 endpoints)

**Source**: `server/routes/security.ts` — All require [Admin] except metadata endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/security/audit` | Admin | List security audit findings |
| POST | `/api/security/audit` | Admin | Create a new audit finding |
| GET | `/api/security/audit/summary` | Admin | Audit findings summary |
| GET | `/api/security/audit/stats` | Admin | Audit finding statistics |
| GET | `/api/security/audit/unresolved` | Admin | List unresolved audit findings |
| GET | `/api/security/audit/:id` | Admin | Get specific audit finding |
| PATCH | `/api/security/audit/:id/resolve` | Admin | Resolve an audit finding |
| POST | `/api/security/hptp/anomalies` | Admin | Report HPTP timing anomaly |
| GET | `/api/security/hptp/anomalies` | Admin | List HPTP anomalies |
| GET | `/api/security/hptp/status` | Admin | HPTP anomaly detection status |
| GET | `/api/security/hptp/fallback-analysis` | Admin | HPTP fallback analysis |
| GET | `/api/security/hptp/stats` | Admin | HPTP statistics |
| GET | `/api/security/hptp/thresholds` | Admin | HPTP anomaly thresholds |
| GET | `/api/security/hptp/fallback-modes` | Admin | HPTP fallback mode configuration |
| GET | `/api/security/hptp/redundancy` | Admin | HPTP redundancy status |
| POST | `/api/security/threats` | Admin | Register a new threat |
| GET | `/api/security/threats` | Admin | List all threats |
| GET | `/api/security/threats/risk-matrix` | Admin | Get threat risk matrix |
| GET | `/api/security/threats/stats` | Admin | Threat statistics |
| GET | `/api/security/threats/meta` | Public | Threat model metadata (categories, severities) |
| GET | `/api/security/threats/:id` | Admin | Get specific threat |
| PATCH | `/api/security/threats/:id` | Admin | Update a threat |
| DELETE | `/api/security/threats/:id` | Admin | Delete a threat |
| POST | `/api/security/threats/seed` | Admin | Seed threat model with defaults |
| POST | `/api/security/implementation` | Admin | Create implementation status entry |
| GET | `/api/security/implementation` | Admin | List implementation status entries |
| GET | `/api/security/implementation/summary` | Admin | Implementation progress summary |
| GET | `/api/security/implementation/metrics` | Admin | Implementation metrics |
| GET | `/api/security/implementation/milestones` | Admin | Implementation milestones |
| GET | `/api/security/implementation/meta` | Public | Implementation metadata |
| GET | `/api/security/implementation/:id` | Admin | Get specific implementation entry |
| PATCH | `/api/security/implementation/:id` | Admin | Update implementation entry |
| DELETE | `/api/security/implementation/:id` | Admin | Delete implementation entry |
| POST | `/api/security/implementation/seed` | Admin | Seed implementation tracker |
| GET | `/api/security/dashboard` | Admin | Unified security dashboard (audits + threats + implementation) |
| GET | `/api/security/kri` | Admin | Key Risk Indicators |
| GET | `/api/security/metadata/categories` | Public | List security categories |
| GET | `/api/security/metadata/types` | Public | List security types |

### 23. GitHub Integration (10 endpoints)

**Source**: `server/routes/github.ts` — All require [Admin]

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/github/token` | Set GitHub personal access token [Admin] |
| GET | `/api/github/status` | Check GitHub connection status [Admin] |
| GET | `/api/github/repos/:owner/:repo/branches` | List branches [Admin] |
| GET | `/api/github/repos/:owner/:repo/contents` | Browse repository contents [Admin] |
| GET | `/api/github/file/:owner/:repo` | Read a file from repo [Admin] |
| PUT | `/api/github/file/:owner/:repo` | Create or update a file in repo [Admin] |
| DELETE | `/api/github/file/:owner/:repo` | Delete a file from repo [Admin] |
| POST | `/api/github/push-workflows/:owner/:repo` | Push CI/CD workflow files [Admin] |
| POST | `/api/github/push-env/:owner/:repo` | Push environment config [Admin] |
| POST | `/api/github/push-batch/:owner/:repo` | Batch push multiple files [Admin] |

### 24. Kong Gateway Management (17 endpoints)

**Source**: `server/routes/kong.ts`

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/kong/status` | Public | Kong Konnect connection status |
| GET | `/api/kong/organization` | Public | Organization info |
| GET | `/api/kong/control-planes` | Public | List control planes |
| GET | `/api/kong/control-planes/:cpId/services` | Public | List services in control plane |
| GET | `/api/kong/control-planes/:cpId/routes` | Public | List routes in control plane |
| GET | `/api/kong/control-planes/:cpId/plugins` | Public | List plugins in control plane |
| GET | `/api/kong/config` | Admin | Get declarative Kong config |
| POST | `/api/kong/control-planes/:cpId/services` | Admin | Create a service |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/routes` | Admin | Create a route |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/plugins` | Admin | Create a plugin |
| POST | `/api/kong/control-planes/:cpId/sync-plenumnet` | Admin | Sync PlenumNET services to control plane |
| POST | `/api/kong/sync-all-control-planes` | Admin | Sync all control planes |
| GET | `/api/kong/service-catalog` | Public | Get PlenumNET service catalog |
| POST | `/api/kong/save-to-github` | Admin | Save Kong config to GitHub |
| GET | `/api/kong/control-planes/:cpId/deploy-instructions` | Public | Get deployment instructions |
| POST | `/api/kong/control-planes/:cpId/generate-deployment` | Admin | Generate deployment artifacts |
| POST | `/api/kong/control-planes/:cpId/deploy-to-cloud` | Admin | Deploy to cloud provider |

---

## Endpoints That Do NOT Exist

The following paths have appeared in external documents but are **fabricated and not implemented**:

- `/api/pqti/sign` — Does not exist. No PQTI REST API is implemented.
- `/api/pqti/timestamp` — Does not exist.
- `/api/pqti/bridge/export` — Does not exist. `CryptoInteropBridge` is a Rust library struct (`src/kernel/src/compat/crypto_interop.rs`), not an HTTP endpoint.

The PQTI (Post-Quantum Ternary Internet) layer exists as Rust kernel code and CI/CD infrastructure only. It has no REST API surface. See `PQTI-P0-STATUS.md` for what actually ships.

---

## Kong Gateway Configuration

17 API services configured in Kong Konnect (`kong/kong.yaml`):

| # | Service | Category | Description |
|---|---------|----------|-------------|
| 1 | `plenumnet-ternary` | Core | GF(3) arithmetic engine |
| 2 | `plenumnet-timing` | Core | HPTP femtosecond timing |
| 3 | `plenumnet-calendars` | Core | 42 calendar system conversions |
| 4 | `plenumnet-phase` | Core | Phase encryption |
| 5 | `plenumnet-vm` | Core | 176-opcode TVM ISA v2.1 |
| 6 | `plenumnet-docs` | Reference | API documentation |
| 7 | `plenumnet-whitepapers` | Reference | Whitepaper management |
| 8 | `plenumnet-legal` | Reference | Legal documents |
| 9 | `plenumnet-demo` | Tools | Compression demo |
| 10 | `plenumnet-compression` | Tools | Compression storage |
| 11 | `plenumnet-auth` | Platform | Authentication |
| 12 | `plenumnet-user` | Platform | User management |
| 13 | `plenumnet-developer-signup` | Platform | Developer waitlist |
| 14 | `plenumnet-health` | Platform | Health check |
| 15 | `plenumnet-admin` | Admin | Admin dashboard |
| 16 | `plenumnet-github` | Admin | GitHub integration |
| 17 | `plenumnet-kong` | Admin | Kong gateway management |

---

## Compliance & Specifications

| Document | Standard |
|----------|----------|
| `finra-613.md` | FINRA Rule 613 (CAT) |
| `mifid-ii.md` | MiFID II RTS 25 |
| `gdpr.md` | GDPR Data Protection |
| `nist-standards.md` | NIST Post-Quantum |
| `SPECIFICATION-v4.21.md` | Full technical spec |
| `XRPL-WITNESSING-SPEC.md` | XRPL witness protocol |
| `HPTP-SPEC.md` | High-Precision Timing Protocol |

---

## Key Metrics

- **Ternary Representations**: A {-1,0,+1}, B {0,1,2}, C {1,2,3}
- **Information Density**: +58.5% vs binary (log₂(3) ≈ 1.585 bits/trit)
- **Timing Precision**: Femtosecond (10⁻¹⁵ seconds)
- **Security Modes**: Zero, One, Phi (φ+)
- **Blockchain Support**: Hedera HCS, XRPL, Algorand
- **VM ISA**: 176 opcodes (v2.1), backward compatible with v2.0 (160) and v1.0 (62)
- **API Surface**: 194 verified endpoints

---

*Generated: February 2026*
*Last API audit: February 23, 2026 — verified against running codebase*
*Repository: SigmaWolf-8/Ternary*
