# Operational Environment Specification
## Salvi Ternary Cryptographic Module v3.0.0
## Capomastro Holdings Ltd. | Applied Physics Division

---

## 1. Tested Operational Environments

The module has been tested on the following General Purpose Computer (GPC)
platform(s). Per FIPS 140-3 IG 2.4, the module runs on a GPC in
single-operator mode.

### OE-1: Primary (x86_64 Linux)

| Parameter | Value |
|---|---|
| Operating System | Ubuntu 24.04 LTS (Noble Numbat) |
| Kernel Version | 6.8.x |
| Architecture | x86_64 (AMD64) |
| Rust Compiler | rustc 1.75.0 |
| Compilation Mode | release (--release) |
| LTO | fat (lto = "fat") |
| Codegen Units | 1 (codegen-units = 1) |
| Panic Strategy | abort (panic = "abort") |
| no_std | Yes — no standard library |
| Heap Allocation | None in approved algorithm paths |
| Entropy Source | RDTSC instruction (x86 timestamp counter) |

### OE-2: Secondary (ARM64 Linux)

| Parameter | Value |
|---|---|
| Operating System | Ubuntu 24.04 LTS (Noble Numbat) |
| Kernel Version | 6.8.x |
| Architecture | aarch64 (ARM64) |
| Rust Compiler | rustc 1.75.0 |
| Compilation Mode | release (--release) |
| LTO | fat (lto = "fat") |
| Codegen Units | 1 (codegen-units = 1) |
| Panic Strategy | abort (panic = "abort") |
| no_std | Yes — no standard library |
| Heap Allocation | None in approved algorithm paths |
| Entropy Source | CNTVCT_EL0 (ARM virtual counter) |

## 2. GPC Operating Mode

- Module operates in **single-operator mode**
- Module does NOT modify the operating system
- Module relies on OS for:
  - Process isolation (separate address space)
  - Memory protection (no access to other processes)
- Module does NOT rely on OS for:
  - Entropy (uses hardware timestamp counter directly)
  - Cryptographic operations (all self-contained)
  - Random number generation (internal HMAC-DRBG)
  - Key management (in-memory only, no persistent storage)

## 3. Module Installation

### 3.1 Build

```
$ ./scripts/cmvp-build.sh [--target x86_64|aarch64]
```

Produces:
- `target/<triple>/release/libternary.rlib` — compiled module binary
- `target/integrity-hash.txt` — HMAC-SHA-384 integrity hash

### 3.2 Install

1. Copy `libternary.rlib` to target system
2. Verify HMAC-SHA-384 of installed binary matches `integrity-hash.txt`
3. Link module into host application

### 3.3 Post-Install Verification

```
$ openssl dgst -sha384 -hmac "SalviTernaryCryptoModule-v3.0.0-IntegrityKey" \
    -binary libternary.rlib | xxd -p | tr -d '\n'
```

Compare output to stored integrity hash. Any mismatch indicates the
binary has been modified and MUST NOT be used.

## 4. Module Operation

### 4.1 Initialization Sequence

1. **Module Load:** Application loads module binary into memory
   - State: PowerOff -> Uninitialized
2. **POST Execution:** Module automatically runs Power-On Self-Tests
   - State: Uninitialized -> SelfTest
   - Tests: 12 KATs covering all algorithms + integrity verification
3. **POST Result:**
   - Success: SelfTest -> Operational
   - Failure: SelfTest -> Error (module unusable until reload)
4. **Set Policy:** Application sets algorithm enforcement policy
   - `set_policy(CnsaOnly)` -> ApprovedMode (FIPS Approved)
   - `set_policy(Hybrid)` -> NonApprovedMode
5. **Verify Mode:** Application queries `get_mode_indicator()`
   - Expected: `Approved` for FIPS-compliant operation
6. **Ready:** Module is ready to service cryptographic requests

### 4.2 Normal Operation

- Call approved crypto services as documented in Security Policy Section 3.2
- Services enforce role-based access (CryptoOfficer or User)
- DRBG automatically reseeds when reseed counter reaches limit
- Health tests run continuously on entropy source

### 4.3 Shutdown Sequence

1. **Zeroize:** Call zeroize to destroy all keys and SSPs
   - State: ApprovedMode/NonApprovedMode/Operational -> Zeroization
   - All 23 SSPs destroyed via ct_zeroize()
2. **Complete:** All SSPs confirmed destroyed
   - State: Zeroization -> Shutdown
3. **Unload:** Application unloads module from memory
   - State: Shutdown -> PowerOff

## 5. Module Constraints

The following constraints MUST be observed for FIPS 140-3 validated operation:

| Constraint | Requirement | Rationale |
|---|---|---|
| Binary Integrity | Module binary MUST match stored integrity hash | POST verifies at startup |
| Compilation Mode | MUST be compiled in release mode (--release) | Debug mode adds non-deterministic behavior |
| Debug Flags | MUST NOT operate with debug flags enabled | Debug output may leak SSPs |
| Modification | MUST NOT modify module binary post-build | Invalidates integrity hash |
| Entropy Source | Hardware timestamp counter MUST be available | Required for DRBG seeding |
| Single Operator | Module operates in single-operator mode | Per FIPS 140-3 IG 2.4 |
| Memory Protection | OS MUST provide process isolation | Prevents SSP leakage between processes |

## 6. Unsupported Configurations

The following configurations are explicitly NOT supported and will prevent
module operation:

| Configuration | Reason |
|---|---|
| Debug compilation | Non-deterministic code paths, potential SSP exposure |
| Multiple codegen units | Non-reproducible build |
| std library | Module is no_std; std adds uncontrolled dependencies |
| Architectures other than x86_64/aarch64 | No qualified entropy source |
| Virtualized timestamp counters | May not provide sufficient jitter entropy |
| Dynamic linking | Integrity verification requires static linking |

## 7. Operational Environment Equivalence

Per FIPS 140-3 IG 2.4, the module may be operated on operational environments
not listed in Section 1, provided:
1. The environment is a General Purpose Computer
2. The module runs in single-operator mode
3. The OS provides equivalent process isolation and memory protection
4. The hardware timestamp counter provides equivalent jitter entropy
5. The module binary passes POST integrity verification

---

*Document: VE-005*
*Salvi Framework — Capomastro Holdings Ltd.*
