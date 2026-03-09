# Cryptographic Module Boundary Diagram
## Salvi Ternary Cryptographic Module v3.0.0
## Capomastro Holdings Ltd. | Applied Physics Division

---

## Logical Boundary

All files in `src/kernel/src/crypto/` constitute the cryptographic
module boundary. Files outside this directory are excluded from the
validated module.

```
┌────────────────────── CRYPTOGRAPHIC MODULE BOUNDARY ──────────────────────┐
│                                                                           │
│  ┌─ APPROVED ALGORITHMS ────────────────────────────────────────────────┐ │
│  │                                                                      │ │
│  │  cipher.rs        AES-256-GCM (FIPS 197)                            │ │
│  │  sha2.rs          SHA-384, SHA-512 (FIPS 180-4)                     │ │
│  │  sha3.rs          SHA3-384, SHA3-512 (FIPS 202) [internal HW only]  │ │
│  │  hmac.rs          HMAC-SHA-384, HMAC-SHA-512 (FIPS 198-1)           │ │
│  │  tl_kem.rs        ML-KEM-1024/768/512 (FIPS 203)                   │ │
│  │  tl_dsa.rs        ML-DSA-87/65/44 (FIPS 204)                       │ │
│  │  signature.rs     LMS, XMSS (SP 800-208)                           │ │
│  │  drbg.rs          HMAC-DRBG-SHA384 (SP 800-90A)                    │ │
│  │  kdf.rs           Key Derivation                                    │ │
│  │                                                                      │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  ┌─ SECURITY INFRASTRUCTURE ────────────────────────────────────────────┐ │
│  │                                                                      │ │
│  │  entropy.rs       SP 800-90B entropy source + health tests           │ │
│  │  self_test.rs     POST + conditional self-tests                      │ │
│  │  module_state.rs  Finite state machine (9 states)                    │ │
│  │  services.rs      Service enumeration + RBAC (32 services, 3 roles)  │ │
│  │  agility.rs       Algorithm policy enforcement                       │ │
│  │  ct_utils.rs      Constant-time primitives + zeroization             │ │
│  │  cnsa2.rs         CNSA 2.0 compliance tracking                       │ │
│  │                                                                      │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  ┌─ VALIDATION & EVIDENCE ──────────────────────────────────────────────┐ │
│  │                                                                      │ │
│  │  cavp_package.rs  210 KAT vectors (.req/.rsp)                        │ │
│  │  acvts.rs         ACVTS JSON format vectors                          │ │
│  │  cavp_certs.rs    Certificate tracking                               │ │
│  │  kat_vectors.rs   Frozen vector regression                           │ │
│  │  cross_impl.rs    Cross-implementation interop                       │ │
│  │  formal_verify.rs 13 verified properties                             │ │
│  │  side_channel.rs  Side-channel analysis                              │ │
│  │  perf_bench.rs    Performance benchmarks                             │ │
│  │                                                                      │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  ┌─ NON-APPROVED (available in Non-Approved Mode only) ─────────────────┐ │
│  │                                                                      │ │
│  │  hash.rs          TL-Sponge hash (non-standard)                      │ │
│  │  sponge.rs        TL-Sponge construction                             │ │
│  │  ternary_lattice.rs  GF(3) polynomial ring + NTT                    │ │
│  │  phase_cnsa.rs    Phase-encryption with ML-KEM keys                  │ │
│  │  fpga_hdl.rs      FPGA Verilog generation                            │ │
│  │  fpga_synth.rs    FPGA resource estimation                           │ │
│  │  hw_test.rs       Hardware test cases                                │ │
│  │  firmware_sign.rs Application-level firmware signing                  │ │
│  │  x509.rs          X.509 certificate operations                       │ │
│  │                                                                      │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  INTERFACES (ISO/IEC 19790 §7.3):                                        │
│  ╔═══════════════╗  ╔════════════════╗                                    │
│  ║ Data Input    ║  ║ Data Output    ║                                    │
│  ║ (API params)  ║  ║ (API returns)  ║                                    │
│  ╚═══════════════╝  ╚════════════════╝                                    │
│  ╔═══════════════╗  ╔════════════════╗                                    │
│  ║ Control Input ║  ║ Status Output  ║                                    │
│  ║ (init, policy ║  ║ (mode, state,  ║                                    │
│  ║  zeroize)     ║  ║  self-test)    ║                                    │
│  ╚═══════════════╝  ╚════════════════╝                                    │
│                                                                           │
│  mod.rs — Module registration (34 modules total)                          │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

## Data Flow Diagram

```
                    ┌──────────────────┐
                    │  EXTERNAL INPUT  │
                    │  (API Caller)    │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
      ┌──────────────┐ ┌──────────┐ ┌──────────────┐
      │ Data Input   │ │ Control  │ │ Status Query │
      │ (plaintext,  │ │ Input    │ │              │
      │  keys, msgs) │ │ (init,   │ │              │
      └──────┬───────┘ │  policy, │ └──────┬───────┘
             │         │  zeroize)│        │
             │         └────┬─────┘        │
             │              │              │
             ▼              ▼              ▼
      ┌─────────────────────────────────────────┐
      │            MODULE CORE                   │
      │                                          │
      │  entropy.rs → drbg.rs → [Algorithms]     │
      │                              │           │
      │  module_state.rs ← self_test.rs          │
      │       │                                  │
      │  services.rs (RBAC enforcement)          │
      │       │                                  │
      │  agility.rs (policy enforcement)         │
      └──────────────────┬──────────────────────┘
                         │
              ┌──────────┼──────────┐
              │          │          │
              ▼          ▼          ▼
      ┌──────────┐ ┌──────────┐ ┌──────────────┐
      │ Data Out │ │ Status   │ │ Error        │
      │ (cipher- │ │ Output   │ │ Indicator    │
      │  text,   │ │ (mode,   │ │              │
      │  sigs,   │ │  state)  │ │              │
      │  hashes) │ │          │ │              │
      └──────────┘ └──────────┘ └──────────────┘
```

## Excluded from Boundary

The following components are NOT part of the validated cryptographic module:

| Component | Location | Reason for Exclusion |
|---|---|---|
| Binary Compatibility Layer | `src/kernel/src/compat/*` | Adapter/interop — not cryptographic |
| Network Protocols | `src/kernel/src/network/*` | Transport layer (TLS, SSH, IPsec, TTP) |
| Hardware Drivers | `src/kernel/src/drivers/*` | Hardware abstraction (femtoclock input only) |
| Architecture Boot Code | `src/kernel/src/arch/*` | Platform boot sequence |
| Memory Management | `src/kernel/src/memory/*` | OS memory subsystem |
| Process Scheduler | `src/kernel/src/process/*` | OS process management |
| React Frontend | `client/*` | User interface |
| Express Backend | `server/*` | Application server |
| Blockchain Contracts | `contracts/*` | Smart contract layer |
| API Gateway Config | `kong/*` | Kong configuration |

## Interface Mapping

| ISO 19790 Interface | Implementation | Description |
|---|---|---|
| Data Input | Function parameters | Plaintext, keys, messages, nonces |
| Data Output | Function return values | Ciphertext, signatures, hashes, shared secrets |
| Control Input | `module_state.transition()`, `set_policy()`, `run_power_on_self_tests()` | Module lifecycle and policy control |
| Status Output | `module_status()`, `get_mode_indicator()`, `is_approved_mode()` | Module state, mode, and health status |

---

*Document: VE-002*
*Salvi Framework — Capomastro Holdings Ltd.*
