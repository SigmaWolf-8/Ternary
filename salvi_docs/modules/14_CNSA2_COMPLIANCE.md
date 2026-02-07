# Module Guide: CNSA 2.0 Compliance Framework

**Module:** `salvi_kernel::crypto::cnsa2`  
**Status:** Active  
**Tests:** 12 tests

---

## Overview

The CNSA 2.0 (Commercial National Security Algorithm Suite 2.0) Compliance Framework tracks PlenumNET's alignment with the NSA's post-quantum cryptographic algorithm requirements. CNSA 2.0 defines the algorithms required for protecting classified information through the quantum computing transition period (2025-2035).

PlenumNET takes a fundamentally different approach from standard CNSA 2.0 implementations: rather than implementing the exact NIST-standardized binary algorithms, the Salvi Framework operates natively in GF(3) (Galois Field of order 3), providing quantum resistance through a different mathematical domain. Each CNSA 2.0 requirement has a ternary-native equivalent with comparable or superior security properties.

---

## CNSA 2.0 Requirements

The NSA CNSA 2.0 suite mandates the following algorithms for national security systems:

| Category | Algorithm | NIST Standard | Security Level | Deadline |
|----------|-----------|---------------|----------------|----------|
| Symmetric Encryption | AES-256 | FIPS 197 | 256-bit | Immediate |
| Hashing | SHA-384 | FIPS 180-4 | 192-bit | Immediate |
| Hashing | SHA-512 | FIPS 180-4 | 256-bit | Immediate |
| Key Encapsulation | ML-KEM-512 | FIPS 203 | 128-bit | By 2030 |
| Key Encapsulation | ML-KEM-768 | FIPS 203 | 192-bit | By 2030 |
| Key Encapsulation | ML-KEM-1024 | FIPS 203 | 256-bit | By 2030 |
| Digital Signatures | ML-DSA-44 | FIPS 204 | 128-bit | By 2035 |
| Digital Signatures | ML-DSA-65 | FIPS 204 | 192-bit | By 2035 |
| Digital Signatures | ML-DSA-87 | FIPS 204 | 256-bit | By 2035 |
| Hash-Based Signatures | LMS | SP 800-208 | 256-bit | By 2030 |
| Hash-Based Signatures | XMSS | SP 800-208 | 256-bit | By 2030 |

---

## PlenumNET Compliance Matrix

### Symmetric Encryption: AES-256 → Ternary Bijective Cipher

**Status:** Planned (Design Complete)

The Ternary Bijective Cipher is fully designed for GF(3) with bijective S-box substitution, supporting CTR, CBC, and authenticated GCM modes. The dedicated `salvi_kernel::crypto::cipher` module is planned for implementation. Currently, the `salvi_kernel::phase` module provides symmetric encryption capability through phase-split encryption with timing-window enforcement.

**Design Specification:** 27-tryte key with 1.585 bits/trit entropy. The total key space of 3^81 ≈ 2^128.4 provides strong symmetric security. GF(3) S-box construction is designed to resist differential and linear cryptanalysis attacks optimized for binary fields.

### Hashing: SHA-384/512 → Ternary Sponge Hash

**Status:** Ternary Equivalent

Keccak-inspired sponge construction over GF(3). 729-trit state width with 243-trit rate and 486-trit capacity. 27 rounds of substitution-permutation network.

```rust
use salvi_kernel::crypto::hash::ternary_hash;

let hash = ternary_hash(&input_trits);  // 243-trit output
assert_eq!(hash.len(), 243);            // 385.4 equivalent bits
```

**Security Analysis:**
- 243-trit output = 385.4 equivalent bits (exceeds SHA-384's 384 bits)
- Extended squeeze produces 486 trits = 770.8 equivalent bits (exceeds SHA-512)
- Sponge capacity of 486 trits provides 770.8-bit security against generic attacks

### Key Encapsulation: ML-KEM → Ternary Lattice KEM (Foundations Complete)

**Status:** Planned (Lattice Foundations Implemented)

ML-KEM (CRYSTALS-Kyber) key encapsulation will be implemented as Ternary Lattice KEM (TL-KEM). The underlying lattice arithmetic foundations are now complete in `salvi_kernel::crypto::ternary_lattice`:

- GF(3) polynomial ring R_q = Z_3[X]/(X^n + 1) with ring multiplication
- Module-LWE instance generation and verification
- Module-SIS problem structure and solution verification
- Polynomial sampling: Centered Binomial Distribution (CBD) and uniform
- Polynomial evaluation and pointwise operations over GF(3)
- Polynomial compression/decompression for ciphertext compactness
- Parameterized security levels: k=2 (Level 1), k=3 (Level 3), k=4 (Level 5)

*Note: Ring multiplication uses schoolbook convolution with X^n+1 reduction. GF(3) lacks the primitive roots of unity needed for standard NTT; future work may lift to a larger modulus q with NTT support.*

```rust
use salvi_kernel::crypto::ternary_lattice::*;

// Generate Module-LWE instance (foundation for TL-KEM)
let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
let instance = generate_module_lwe(&seed, 3, 256, 2)?; // k=3, n=256, eta=2
assert!(verify_module_lwe(&instance)?);

// Polynomial ring arithmetic
let a = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1])?;
let b = TernaryPolynomial::from_coeffs(vec![0, 1, 1, -1])?;
let product = a.ring_mul(&b)?; // Multiplication in Z_3[X]/(X^4+1)
```

**Security Levels:**
- TL-KEM-512: NIST Security Level 1 (128-bit classical, k=2, n=256)
- TL-KEM-768: NIST Security Level 3 (192-bit classical, k=3, n=256)
- TL-KEM-1024: NIST Security Level 5 (256-bit classical, k=4, n=256)

**Ternary Advantage:** ML-KEM internally uses coefficients from {-1, 0, 1} for its error terms, which maps directly to PlenumNET's balanced ternary Representation A {-1, 0, +1}. The `ternary_lattice` module represents these coefficients natively without encoding overhead.

### Digital Signatures: ML-DSA → Ternary Lattice DSA (Foundations Complete)

**Status:** Planned (Lattice Foundations Implemented, Signature Stubs Exist)

ML-DSA (CRYSTALS-Dilithium) digital signatures will be implemented as Ternary Lattice DSA (TL-DSA). The lattice arithmetic foundations are shared with TL-KEM via `salvi_kernel::crypto::ternary_lattice`, and signature scheme stubs exist in `salvi_kernel::crypto::signature`.

The Module-SIS problem (basis for Dilithium's security) is implemented with:
- Matrix-vector operations over polynomial rings
- L-infinity norm bounds for short vector verification
- Parameterized beta bounds for security level configuration

```rust
use salvi_kernel::crypto::ternary_lattice::*;

// Module-SIS instance (foundation for TL-DSA)
let seed = vec![0i8, 1, -1];
let sis = generate_module_sis(&seed, 3, 256, 1); // k=3, n=256, beta=1
let zero = TernaryPolyVec::new(3, 256);
assert!(verify_sis_solution(&sis, &zero)?);
```

**Security Levels:**
- TL-DSA-44: NIST Security Level 2
- TL-DSA-65: NIST Security Level 3
- TL-DSA-87: NIST Security Level 5

### Hash-Based Signatures: LMS/XMSS → Ternary Lamport OTS

**Status:** Ternary Equivalent

Fully implemented ternary Lamport one-time signature scheme with key chain support. Hash-based construction is inherently quantum-resistant because its security relies only on hash function preimage resistance.

```rust
use salvi_kernel::crypto::signature::{generate_keypair, sign, verify, SignatureScheme};

let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
let (mut sk, vk) = generate_keypair(SignatureScheme::TernaryLamport, &seed)?;
let sig = sign(&mut sk, &message)?;
assert!(verify(&vk, &message, &sig)?);
```

**Security Analysis:** The Lamport OTS construction security depends solely on the preimage resistance of the underlying hash function (ternary sponge hash). With 243-trit digests, the scheme provides 385.4-bit equivalent security against preimage attacks. The LamportKeyChain provides indexed multi-message signing equivalent to LMS tree-based approaches.

---

## Transition Timeline

| Year | Milestone | Status |
|------|-----------|--------|
| 2025 | Foundation Complete: Ternary sponge hash, HMAC, KDF, Lamport OTS, phase encryption | Complete |
| 2026 | Lattice Foundations: GF(3) polynomial ring arithmetic, Module-LWE/SIS, AES-256-GCM, SHA-2, SHA-3 | Complete |
| 2027 | TL-KEM Implementation: All three ML-KEM security levels | Planned |
| 2028 | TL-DSA Implementation: All three ML-DSA security levels | Planned |
| 2029 | XMSS Merkle Tree Extension: Stateful hash-based signature trees | Planned |
| 2030 | Full CNSA 2.0 Coverage: Complete ternary equivalents, FIPS validation initiated | Planned |

---

## The Ternary Advantage

### Why GF(3) Matters for Post-Quantum Security

Quantum computers attack binary cryptography by exploiting the structure of binary fields (GF(2^n)). Shor's algorithm factors integers and computes discrete logarithms efficiently because these problems have well-understood binary algebraic structure. Grover's algorithm provides quadratic speedup for binary search problems.

PlenumNET's ternary-native approach introduces structural resistance:

1. **Different mathematical domain:** GF(3) operations do not map directly to quantum gate operations optimized for GF(2). Quantum circuits must be redesigned for ternary arithmetic, reducing the practical advantage of known quantum algorithms.

2. **Natural lattice coefficient encoding:** Post-quantum lattice problems (LWE, SIS) use ternary error distributions {-1, 0, 1}. PlenumNET's Representation A provides native encoding without binary-to-ternary conversion overhead.

3. **Information density:** Each trit carries 1.585 bits of information. A 243-trit hash output provides 385.4 equivalent bits of security, exceeding SHA-384's 384 bits while using fewer computational elements.

4. **Binary compatibility:** The Binary-Ternary Gateway (BTG) enables hybrid deployment where ternary operations run natively while maintaining interoperability with standard binary CNSA 2.0 implementations.

### GF(3) Arithmetic Operations

| Operation | Definition | Quantum Resistance |
|-----------|-----------|-------------------|
| Addition | (a + b) mod 3 in balanced representation | Non-binary field structure |
| Multiplication | (a × b) mod 3 with ternary S-boxes | Different algebraic group |
| Inverse | Unique multiplicative inverse in GF(3)* | Novel factoring resistance |
| XOR-equivalent | Ternary addition in GF(3) | Three-valued logic complexity |

---

## Binary Compatibility

The CNSA 2.0 compliance framework includes interoperability with standard binary implementations through the Binary-Ternary Gateway:

```
┌─────────────────────┐     ┌──────────────┐     ┌─────────────────────┐
│  Standard CNSA 2.0  │────▶│     BTG      │────▶│  PlenumNET Ternary  │
│  (Binary ML-KEM,    │     │  Binary ↔    │     │  (TL-KEM, TL-DSA,   │
│   ML-DSA, AES-256)  │◀────│  Ternary     │◀────│   Bijective Cipher) │
└─────────────────────┘     └──────────────┘     └─────────────────────┘
```

This enables:
- Hybrid key exchange combining binary ML-KEM with ternary TL-KEM
- Dual signatures using both ML-DSA and TL-DSA for maximum assurance
- Transparent format conversion for encrypted data exchange

---

## Regulatory Context

CNSA 2.0 compliance intersects with PlenumNET's existing regulatory framework:

- **FINRA Rule 613:** Consolidated Audit Trail requires sub-50ms NIST synchronization. HPTP femtosecond timing exceeds this by orders of magnitude. Cryptographic signing of audit records uses quantum-resistant Lamport OTS.

- **MiFID II Article 50:** EU high-frequency trading requires 100μs synchronization. HPTP with optical clock sync provides sub-picosecond precision. All timing certificates are cryptographically signed.

- **NIST SP 800-208:** Hash-based signature standard. PlenumNET's Lamport OTS with ternary sponge hash aligns with the stateful signature approach defined in this standard.

---

## Related Modules

- [Cryptographic Primitives](./05_CRYPTOGRAPHY.md) - Core ternary crypto operations
- [Timing Protocol](./12_TIMING_PROTOCOL.md) - HPTP femtosecond synchronization
- [Calendar Synchronization](./13_CALENDAR_SYNCHRONIZATION.md) - Regulatory timestamp compliance

---

*Part of the Salvi Framework Documentation. Cosi sia.*
