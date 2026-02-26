# Intellectual Property Notice

## Capomastro Holdings Ltd. — Applied Physics Division

**Effective Date:** February 14, 2026
**Classification:** PUBLIC

---

## 1. Patent-Pending Claims

The following technologies, methods, and systems described in this repository are the subject of one or more pending patent applications filed by or on behalf of Capomastro Holdings Ltd.:

### 1.1 Ternary Computing Architecture
- Balanced ternary arithmetic operations using GF(3) field arithmetic
- 176-opcode Ternary Virtual Machine (TVM) ISA v2.1 with nibble-aligned encoding
- Hierarchical two-stage instruction decode with crypto dispatch acceleration
- Ternary-aware garbage collector with three-ring privilege levels
- Ternary SIMD (Single Instruction, Multiple Data) operations

### 1.2 Post-Quantum Cryptographic Systems
- Phase Encryption: split/recombine encryption with timing-window enforcement
- TL-KEM (Ternary Lattice Key Encapsulation Mechanism)
- TL-DSA (Ternary Lattice Digital Signature Algorithm)
- GF(3) polynomial arithmetic for lattice-based cryptography
- Ternary Lamport signature scheme
- Ternary sponge construction and hash functions
- CNSA 2.0 compliant cryptographic primitive suite

### 1.3 High-Precision Timing Protocol (HPTP)
- Femtosecond-precision timing service architecture
- Salvi Epoch timing system (epoch: 2025-04-01T00:00:00Z)
- NTP-symmetric correction model for distributed timing
- Ancient calendar synchronization across 42 global calendar systems
- Timing-window enforced encryption operations

### 1.4 Torsion Network Architecture
- N-dimensional torus topology for network routing
- Greedy geodesic routing algorithm
- Ternary Transport Protocol (TTP)
- Ternary Transfer Protocol (T3P)
- Ternary DNS (TDNS)

### 1.5 Tribonacci-Based Data Structures
- Tribonacci sequence-based hash functions for shard distribution
- 28-fold coverage verification using Tribonacci properties
- Skip-list indexing with Tribonacci jump tables
- Tribonacci-modular worker allocation

### 1.6 Memory and Process Management
- Bitmap-based frame allocator with ternary security gating
- Page table management with ternary security levels
- Ternary-security-gated mutexes and semaphores
- Phase-encryption-aware synchronization primitives
- Modal security system with capability-based access control

### 1.7 Hardware Description and Synthesis
- Ternary Hardware Description Language (THDL)
- FPGA decoder with hierarchical nibble-aligned instruction decode
- Binary compatibility layer for balanced ternary conversion

---

## 2. Trade Secrets

The following constitute trade secrets of Capomastro Holdings Ltd. and are protected under the *Trade Secrets Act* (Alberta), the *Defend Trade Secrets Act* (United States), and applicable international trade secret law:

- Internal algorithm parameters, constants, and tuning values used in cryptographic primitives
- Performance optimization techniques in the Ternary Virtual Machine
- Proprietary compression algorithms and their implementation details
- Key derivation and key management procedures
- Internal timing calibration and correction algorithms
- Blockchain witnessing and consensus participation strategies

---

## 3. Proprietary Algorithms

The following algorithms are proprietary to Capomastro Holdings Ltd. and may not be reproduced, reverse-engineered, or re-implemented without express written permission:

| Algorithm | Module | Description |
|-----------|--------|-------------|
| Phase Split/Recombine | `src/kernel/src/crypto/` | Timing-gated encryption with phase decomposition |
| Ternary Sponge | `src/kernel/src/crypto/` | GF(3)-based sponge construction |
| TL-KEM | `src/kernel/src/crypto/` | Ternary lattice key encapsulation |
| TL-DSA | `src/kernel/src/crypto/` | Ternary lattice digital signatures |
| Tribonacci Hash | `libternary/src/tribonacci.rs` | Sequence-based hash distribution |
| Borromean Ring Proofs | `libternary/src/borromean.rs` | Ternary Borromean ring signatures |
| Torsion Routing | `src/kernel/src/torsion/` | N-dimensional torus geodesic routing |
| HPTP Correction | `services/timing/` | Femtosecond-precision timing correction |

---

## 4. Copyright

All source code, documentation, and associated materials in this repository are:

**Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)**
**All Rights Reserved.**

See `LICENSE` in the repository root for full terms.

---

## 5. Trademarks

The following are trademarks or trade names of Capomastro Holdings Ltd.:

- **PlenumNET**
- **Salvi Framework**
- **Ternary Virtual Machine (TVM)**
- **High-Precision Timing Protocol (HPTP)**
- **Phase Encryption**

Use of these marks without authorization is prohibited.

---

## 6. Third-Party Components

This project incorporates third-party open-source components as listed in the `NOTICE` file. All such components are used under their respective licenses (MIT, Apache-2.0, ISC, BSD). No third-party component is subject to copyleft obligations (GPL, AGPL, LGPL).

---

## 7. Contact

For licensing inquiries, patent matters, or IP questions:

**Capomastro Holdings Ltd.**
Applied Physics Division
Province of Alberta, Canada

---

*This notice is provided for informational purposes and does not constitute legal advice. Consult retained counsel for specific IP matters.*
