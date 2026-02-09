# ADR-003: Dual-Layer Cryptographic Architecture — Lamport + Lattice

| Field       | Value |
|-------------|-------|
| **Status**  | Accepted |
| **Date**    | 2025-12-15 |
| **Author**  | Capomastro Holdings Ltd |
| **Context** | Designing the post-quantum cryptographic stack for CNSA 2.0 compliance |

## 1 · Context

PlenumNET's cryptographic architecture must satisfy two constraints simultaneously:

1. **CNSA 2.0 compliance** — The NSA's Commercial National Security Algorithm Suite 2.0 mandates specific post-quantum algorithms for national security systems. The required algorithms include ML-KEM (formerly CRYSTALS-Kyber) for key encapsulation, ML-DSA (formerly CRYSTALS-Dilithium) for digital signatures, SLH-DSA (formerly SPHINCS+) for stateless hash-based signatures, AES-256 for symmetric encryption, SHA-384/SHA-512 for hashing, and HMAC-SHA-384 for message authentication.

2. **Unconditional security guarantee** — Some use cases (blockchain witnessing, long-term document authentication) require signatures whose security does not depend on any computational hardness assumption. If lattice problems are eventually broken (however unlikely), there must be a fallback layer that remains secure.

Lamport one-time signatures (OTS) provide unconditional security — their security relies only on the one-wayness of the hash function, not on any structured mathematical problem. However, Lamport signatures are stateful (each key pair can only be used once), large (kilobytes per signature), and not part of the CNSA 2.0 suite. Lattice-based signatures (ML-DSA) are stateless, compact, and CNSA 2.0 compliant, but their security depends on the hardness of structured lattice problems (Module-LWE, Module-SIS).

## 2 · Decision

PlenumNET implements a **dual-layer cryptographic architecture** combining Lamport OTS and lattice-based algorithms:

### Layer 1: CNSA 2.0 Compliance Layer

The primary cryptographic layer uses the CNSA 2.0 mandated algorithms:

| Function | Algorithm | Standard |
|----------|-----------|----------|
| Key Encapsulation | ML-KEM-1024 | FIPS 203 |
| Digital Signature | ML-DSA-87 | FIPS 204 |
| Stateless Hash Signature | SLH-DSA-SHA2-256s | FIPS 205 |
| Symmetric Encryption | AES-256-GCM | FIPS 197 / SP 800-38D |
| Hashing | SHA-384, SHA-512, SHA3-256, SHA3-512 | FIPS 180-4 / FIPS 202 |
| Message Authentication | HMAC-SHA-384 | FIPS 198-1 |
| Key Derivation | HKDF-SHA-384 | SP 800-56C |
| Key Agreement | ML-KEM + X25519 hybrid | Transitional |
| Random Number Generation | HMAC-DRBG (SHA-384) | SP 800-90A |

This layer handles all routine cryptographic operations: TLS session keys, API authentication tokens, document signing, and encrypted storage.

### Layer 2: Unconditional Security Layer

The secondary layer uses **ternary Lamport one-time signatures** for operations requiring unconditional security:

- **Blockchain witnessing** — Each witness transaction is signed with a fresh Lamport key pair. Since each transaction uses a unique key, the one-time limitation is not a constraint.
- **Long-term document authentication** — Critical documents receive both an ML-DSA signature (CNSA 2.0 compliant, compact) and a Lamport signature (unconditionally secure, large but archival).
- **Key ceremony attestation** — Root key generation ceremonies produce Lamport-signed attestations stored alongside the ceremony transcript.

The Lamport implementation uses the ternary hash function (`tribonacciHash`) for key generation and verification, operating over GF(3) trits rather than binary bits. Each Lamport key pair consists of 3^k hash preimages (where k is the security parameter), and verification checks that the signature hashes to the public key components.

### Key State Management

Lamport key state is tracked on-chain (for blockchain use cases) and in the database (for document authentication). The key state records:
- Key pair index (monotonically increasing, never reused)
- Usage status (unused / used / revoked)
- Creation timestamp (HPTP femtosecond precision)
- Associated document or transaction hash

A key pair marked "used" is permanently retired. The system enforces this at the database constraint level — attempting to sign with a used key triggers a hard error, not a warning.

### Interoperability Bridge

The `CryptoInteropBridge` module provides conversion between:
- ML-KEM shared secrets and AES-256-GCM symmetric keys
- ML-DSA signatures and Lamport verification chains
- Binary hash outputs (SHA-384) and ternary hash outputs (tribonacciHash)

This bridge ensures that both layers can operate on the same data without manual format conversion.

## 3 · Consequences

**Positive:**
- Full CNSA 2.0 compliance is achieved through the primary layer, satisfying regulatory requirements for government and financial sector deployments.
- The Lamport layer provides a security guarantee that survives even a complete break of lattice-based cryptography. This is a genuine hedge, not security theater — Lamport security depends only on hash function one-wayness.
- The dual-layer design is transparent to most application code. The `crypto` module selects the appropriate layer based on the operation type, and the `CryptoInteropBridge` handles format conversion.
- Blockchain witnessing naturally fits the one-time signature model, turning Lamport's main limitation (statefulness) into a non-issue for that use case.

**Negative:**
- Lamport signatures are large (~16 KB per signature for 128-bit security). Storage costs for document authentication are significant. This is mitigated by using Lamport signatures only for archival/high-assurance use cases, not routine operations.
- Key state management adds complexity. The system must guarantee that no Lamport key pair is ever reused, which requires reliable persistence and crash recovery. A key pair used but not marked as used (due to a crash) could lead to a reuse vulnerability.
- Two signature verification paths must be maintained, tested, and audited. This doubles the surface area for cryptographic implementation bugs.
- The ternary Lamport implementation is non-standard (operating over GF(3) rather than binary). This means no off-the-shelf implementations exist for comparison testing — correctness relies entirely on the project's own test suite and formal verification.

## 4 · Alternatives Considered

**Lattice-only (no Lamport layer):**
Rejected. While ML-DSA is believed to be quantum-resistant, its security rests on the hardness of Module-LWE. If this assumption is invalidated (by a mathematical breakthrough, not just quantum computers), all signatures ever produced become forgeable. For blockchain witnessing and long-term authentication, this risk is unacceptable.

**Lamport-only (no lattice layer):**
Rejected. Lamport signatures are too large and stateful for routine use. A Lamport-only system would require key state tracking for every API authentication, TLS handshake, and session token — an enormous operational burden. Additionally, Lamport OTS is not part of CNSA 2.0, so a Lamport-only system would fail regulatory compliance.

**XMSS/LMS (stateful hash-based signatures, NIST SP 800-208):**
Considered as a replacement for raw Lamport OTS. XMSS and LMS use Merkle trees over one-time signatures to amortize key material, reducing per-signature overhead. However, they introduce additional complexity (tree traversal, state synchronization for distributed signers) and are better suited as a future optimization. The current Lamport implementation can be upgraded to XMSS/LMS without changing the dual-layer architecture — the Layer 2 interface remains the same, only the internal signature scheme changes. SP 800-208 compliance is tracked as a roadmap item.

**Hybrid signatures (lattice + hash-based in a single signature):**
Considered. A hybrid signature concatenates an ML-DSA signature and a Lamport/XMSS signature into a single artifact, providing security if either scheme is secure. This is conceptually cleaner than two separate layers but produces very large signatures (~18 KB) for every operation, not just high-assurance ones. The dual-layer approach allows the application to choose the appropriate security level per operation, keeping routine signatures compact.
