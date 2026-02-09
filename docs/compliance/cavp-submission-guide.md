# CAVP Submission Guide

## Document Information

| Field | Value |
|-------|-------|
| Document | CAVP Submission Procedures |
| Version | 1.0 |
| Date | February 2026 |
| Owner | Capomastro Holdings Ltd. |

---

## 1. Overview

This document describes the procedure for generating and submitting CAVP (Cryptographic Algorithm Validation Program) test vectors to NIST for FIPS 140-3 CMVP validation of the PlenumNET Salvi Cryptographic Module.

## 2. Vector Generation

### Counts

| Algorithm | Variants | Vectors/Variant | Total |
|-----------|----------|----------------|-------|
| TL-KEM | 3 (512, 768, 1024) | 35 | 105 |
| TL-DSA | 3 (44, 65, 87) | 35 | 105 |
| **Total** | **6** | | **210** |

### Generation API

```rust
use plenumnet_kernel::crypto::cavp_package;

let package = cavp_package::generate_cavp_package()?;
let report = cavp_package::validate_cavp_package(&package);
assert!(report.valid);
```

### Output Files

Each variant produces a request (.req) and response (.rsp) file pair:

| File | Contents |
|------|----------|
| `TL-KEM-512.req` | Seeds and encapsulation randomness |
| `TL-KEM-512.rsp` | PK/SK/CT/SS hashes, sizes |
| `TL-DSA-44.req` | Seeds and messages |
| `TL-DSA-44.rsp` | PK/SK/Signature hashes, validity |
| `capabilities.json` | Algorithm capability descriptions |
| `manifest.txt` | Package manifest with compliance summary |

## 3. Submission Procedure

### Step 1: Generate Package

Run the CAVP package generator with full vector count:

```rust
let pkg = cavp_package::generate_cavp_package()?; // 35 vectors/variant
```

### Step 2: Validate Package

```rust
let report = cavp_package::validate_cavp_package(&pkg);
assert!(report.valid, "Package validation failed: {:?}", report.issues);
```

### Step 3: Export Files

Write each `CavpFile` in the package to the submission directory structure.

### Step 4: Submit to CMVP

Submit the package directory to the NIST CMVP lab along with:
- Algorithm specification document
- This security policy
- Implementation source code
- Build instructions

## 4. Vector Format

### KEM Request Format

```
# TL-KEM Known Answer Test Request File
# Algorithm: TL-KEM-512

[Vector 0]
Seed = <hex-encoded trit seed>
EncapsRandomness = <hex-encoded randomness>
```

### KEM Response Format

```
[Vector 0]
Seed = <hex-encoded>
EncapsRandomness = <hex-encoded>
PK_Hash = <sponge hash of public key trits>
SK_Hash = <sponge hash of secret key trits>
CT_Hash = <sponge hash of ciphertext>
SS_Hash = <sponge hash of shared secret>
PK_TritCount = <integer>
SK_TritCount = <integer>
CT_ByteCount = <integer>
SS_TritCount = <integer>
```

### DSA Request/Response

Similar structure with `Message`, `Sig_Hash`, and `SigValid` fields.

## 5. Determinism Verification

All KAT vectors are deterministic. Running the generator twice produces identical output:

```rust
let v1 = kat_vectors::generate_kem_kat_vectors()?;
let v2 = kat_vectors::generate_kem_kat_vectors()?;
assert_eq!(v1[i].pk_hash, v2[i].pk_hash); // Deterministic
```

## 6. Frozen Vector Regression

The `validate_frozen_vectors()` function checks that generated outputs match frozen reference values, detecting any implementation changes that alter cryptographic outputs.
