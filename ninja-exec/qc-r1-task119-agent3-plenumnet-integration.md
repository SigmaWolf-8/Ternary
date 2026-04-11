# QC-R1 Agent 3: PlenumNET Integration Specialist Review

**Document:** `.local/tasks/continuous-attestation-service.md`
**Revision:** Gap 9 — Continuous Attestation Service
**Review Date:** 2026-03-28
**Skill Version:** plenumnet-integration v1.1.2
**Reviewer:** Agent 3 — PlenumNET Integration Specialist

---

## Findings

### Finding 1
- **ID:** R1-A3-1
- **Section:** Specification (¶1, ¶5 — TL-DSA signing)
- **Severity:** CRITICAL
- **Finding:** The spec states attestation reports are "TL-DSA signed using the PUF-derived root key" but does not require the signer's Rep C address to be bound into the TL-DSA signature context string. INVARIANT 7 mandates that the signer's Rep C address must be bound into the signature context string and that signature verification must check the signer's public key against a registered Rep C address. INVARIANT 9 further requires that all cryptographic operations binding node identity use Rep C (54-trit, binary-encoded) addressing exclusively. Without this binding, an attacker who obtains a valid PUF key could sign attestation reports for any node — the signature is not bound to a specific topological identity. This is a cryptographic weakness. **Cross-reference to Security Engineer (Agent 1) for severity assessment.**
- **Recommendation:** Add an explicit requirement: "The TL-DSA signing context string MUST include the signer's Rep C address (54-trit, binary-encoded). Verifiers MUST check the signer's public key against the Rep C address embedded in the context string and confirm it matches a registered geometric neighbor." Define the exact context string format, e.g., `"ATTESTATION-SIGN-v1.0" ‖ signer_rep_c_addr`.
- **Verification:** Confirm the implementation passes the signer's Rep C address into `tl_dsa_sign()` context parameter and that `tl_dsa_verify()` reconstructs the same context from the claimed signer identity before verification.

### Finding 2
- **ID:** R1-A3-2
- **Section:** Specification (¶1 — attestation report signing) and Tasks (Task 1)
- **Severity:** IMPORTANT
- **Finding:** No TL-DSA signing context string is defined anywhere in the spec. Context strings are load-bearing in PlenumNET — a wrong or missing context string produces a wrong signature binding. The spec must define the exact context string for attestation report signing and register it in the canonical context string registry. The string `"ATTESTATION-SIGN-v1.0"` does not appear in the current canonical registry — it is UNVERIFIED until added. Without a defined context string, implementers will invent ad-hoc strings, creating interoperability failures.
- **Recommendation:** Define the exact attestation signing context string (e.g., `"PLENUMNET-ATTEST-v1.0"`) and add it to the canonical context string registry in the repo guide. The context string must include a version component to allow future schema evolution without key reuse.
- **Verification:** Grep the canonical context string registry for the chosen attestation context string. Confirm it is unique and does not collide with existing entries (e.g., `"PlenumNET-CON-v3.0"`, `"HEARTBEAT-MAC"`).

### Finding 3
- **ID:** R1-A3-3
- **Section:** Specification (¶1 — "rolling Merkle tree of heartbeat challenges") and Tasks (Task 2)
- **Severity:** IMPORTANT
- **Finding:** The spec describes a "rolling Merkle tree" for liveness proofs but does not specify which hash primitive is used for the Merkle tree internal nodes and leaf hashing. Per the critical rules, TIS-27 is the sole hash/MAC primitive in PlenumNET (SHA-256 and BLAKE3 are explicitly banned). For a Merkle tree used as a liveness proof (not a security-critical key derivation), TIS-27 (43-bit integrity) is the appropriate primitive. However, if the Merkle tree's integrity is security-critical (an adversary forging a liveness proof constitutes an attestation bypass), TLSponge-385 may be required. The spec must make this choice explicit.
- **Recommendation:** Add: "Merkle tree nodes SHALL be hashed using TIS-27 (54-trit sponge, 4 rounds). Leaf inputs SHALL be trit-encoded heartbeat challenge-response pairs." If the threat model requires resistance to offline forgery attacks on the Merkle tree, escalate to TLSponge-385 and document the rationale.
- **Verification:** Confirm the Merkle tree implementation imports `tis27_hash()` (or `tlsponge_385_hash()` if escalated) and does not import or call any banned hash function (SHA-256, BLAKE3, etc.).

### Finding 4
- **ID:** R1-A3-4
- **Section:** Specification (¶1 — "kernel integrity hash") and Tasks (Task 1)
- **Severity:** IMPORTANT
- **Finding:** The attestation report includes a "kernel integrity hash" but the spec does not name the hash primitive used to compute it. This is a binary-to-ternary boundary: the kernel binary is a byte stream that must be hashed. Per INVARIANT 8, raw binary integers must not enter sponge absorb directly — they must be trit-decomposed first. The spec must (a) name the hash primitive (TIS-27 or TLSponge-385), and (b) specify the encoding used to convert the kernel binary into trit representation before hashing.
- **Recommendation:** Add: "Kernel integrity hash SHALL be computed using TLSponge-385 over the kernel binary image. The binary image SHALL be converted to trit representation via `u8_to_trits()` before sponge absorption (INVARIANT 8)." TLSponge-385 is recommended here because kernel integrity is security-critical — a forged kernel hash constitutes an attestation bypass.
- **Verification:** Confirm the implementation calls `u8_to_trits()` on each byte of the kernel image before passing trits to the sponge absorb function. Confirm no raw `&[u8]` slice is passed directly to absorb.

### Finding 5
- **ID:** R1-A3-5
- **Section:** Specification (¶1 — "boot measurements from measured boot in firmware_sign.rs") and Tasks (Task 1)
- **Severity:** IMPORTANT
- **Finding:** Boot measurements from `firmware_sign.rs` are binary data (XMSS/LMS hashes, anti-rollback counters). The spec includes them as attestation report fields but does not specify how binary measurement values are encoded for inclusion in the attestation report or for input to any ternary sponge operation. Per INVARIANT 8, all binary integers must be trit-decomposed before entering ternary operations. The spec must define the encoding boundary.
- **Recommendation:** Add: "Boot measurement fields (firmware hashes, anti-rollback counters) SHALL be trit-encoded via `u8_to_trits()` / `u16_to_trits()` before inclusion in any ternary hash or sponge operation. The attestation report wire format SHALL carry these values in Rep C encoding (INVARIANT 3)."
- **Verification:** Confirm boot measurement fields in the attestation report struct use Rep C trit arrays (not raw byte arrays) for any field that enters a sponge or hash operation.

### Finding 6
- **ID:** R1-A3-6
- **Section:** Specification (¶5 — "TL-DSA signed using the PUF-derived root key") and Tasks (Task 1)
- **Severity:** IMPORTANT
- **Finding:** The spec states attestation reports are signed with "the PUF-derived root key" but does not define the TLSponge-385 key derivation path from PUF material to the attestation signing key. Per the repo guide (§6.5), TLSponge-385 key derivation requires a domain-separation context string and canonical input ordering. The attestation key derivation context string is not defined. If the attestation signing key is the same as the PUF root key used for other purposes (e.g., tunnel key derivation), key separation is violated. If it is derived, the derivation formula must be specified.
- **Recommendation:** Define the key derivation formula explicitly, e.g.: `TLSponge-385("PLENUMNET-ATTEST-KEY-v1.0" ‖ puf_root_secret ‖ node_rep_c_addr) → attestation_signing_key`. This ensures domain separation from other PUF-derived keys (e.g., CON tunnel keys use `"PlenumNET-CON-v3.0"`). Register the context string in the canonical registry.
- **Verification:** Confirm the attestation key derivation uses a unique context string distinct from all other TLSponge-385 derivation contexts. Grep the codebase for the chosen context string to confirm no collisions.

### Finding 7
- **ID:** R1-A3-7
- **Section:** Specification (¶1 — "FTS/GLB configuration fingerprint") and Tasks (Task 1)
- **Severity:** IMPORTANT
- **Finding:** The attestation report includes an "FTS/GLB configuration fingerprint" but does not specify which hash primitive computes this fingerprint. Per the critical rules, only TIS-27 or TLSponge-385 are permitted. The fingerprint is a security-relevant field — if an attacker can forge a configuration fingerprint, they can hide FTS configuration tampering. The spec must name the primitive.
- **Recommendation:** Add: "FTS/GLB configuration fingerprint SHALL be computed using TIS-27 over the canonical serialization of the FTS/GLB configuration, with configuration fields trit-encoded before hashing."
- **Verification:** Confirm the fingerprint computation uses `tis27_hash()` and does not use any banned hash function.

### Finding 8
- **ID:** R1-A3-8
- **Section:** Specification (¶1, "geometric neighbors") and general
- **Severity:** MINOR
- **Finding:** The spec references "geometric neighbors (26 in a standard 13D hypercube)" which is consistent with INVARIANT 1 (13D ternary hypercube routing). However, the spec does not explicitly state that neighbor identification in attestation verification uses Rep C (54-trit) addressing. While the spec's language is consistent with the architecture, an explicit statement would prevent implementers from using IP addresses or hostnames as neighbor identifiers in the attestation verification path (which would violate INVARIANT 9).
- **Recommendation:** Add: "Neighbor identification in attestation verification SHALL use Rep C (54-trit, binary-encoded) addresses exclusively (INVARIANT 9). No IP address, hostname, or other non-Rep-C identifier may be used as a neighbor identity binding."
- **Verification:** Confirm the attestation verification code uses `CubeAddr` (Rep C) for neighbor lookup, not string-based hostnames or IP addresses.

### Finding 9
- **ID:** R1-A3-9
- **Section:** Specification (¶1 — "HPTP timestamps")
- **Severity:** MINOR
- **Finding:** The spec states attestation reports "include HPTP timestamps" but does not explicitly confirm the timestamp format: 128-bit integers measuring femtoseconds since the Salvi Epoch (2025-04-01T00:00:00Z, INVARIANT 6). While this is implied by "HPTP timestamps," an explicit statement prevents implementers from using Unix timestamps or millisecond precision.
- **Recommendation:** Add: "HPTP timestamps in attestation reports SHALL be 128-bit integers measuring femtoseconds since the Salvi Epoch (2025-04-01T00:00:00Z, INVARIANT 6)."
- **Verification:** Confirm the attestation report struct uses a `u128` field for the HPTP timestamp and that the value is computed relative to the Salvi Epoch constant.

### Finding 10
- **ID:** R1-A3-10
- **Section:** Specification (bandwidth budget — "~3KB per report")
- **Severity:** MINOR
- **Finding:** The bandwidth estimate of "~3KB per report" is given without specifying the wire encoding format. Per INVARIANT 3, all external-facing wire formats must use Rep C encoding. The bandwidth estimate should be validated against the actual Rep C-encoded report size to ensure the 5% link capacity budget is correctly calibrated.
- **Recommendation:** Add: "Attestation reports SHALL be serialized in Rep C trit encoding for wire transmission (INVARIANT 3). The ~3KB bandwidth estimate SHALL be validated against the actual serialized report size during implementation."
- **Verification:** Measure the actual serialized attestation report size in Rep C encoding and confirm it is within the ~3KB estimate. If significantly larger, recalibrate the bandwidth budget parameters.

---

## Cryptographic Claim Verification Summary

| Claim | Status | Notes |
|-------|--------|-------|
| TL-DSA for attestation report signing | **VERIFIED** (primitive selection correct) | TL-DSA is the correct signature primitive per INVARIANT 7 |
| PUF-derived root key for signing | **UNVERIFIED** | Key derivation path from PUF to attestation signing key not defined |
| HPTP timestamps in reports | **VERIFIED** (primitive selection correct) | HPTP is the correct timing source per INVARIANT 5 |
| TL-DSA signing context string | **UNVERIFIED** | No context string defined; not in canonical registry |
| Merkle tree hash primitive | **UNVERIFIED** | Hash primitive for Merkle tree not specified |
| Kernel integrity hash primitive | **UNVERIFIED** | Hash primitive not named |
| FTS/GLB configuration fingerprint hash | **UNVERIFIED** | Hash primitive not named |
| Boot measurement encoding (binary→ternary) | **UNVERIFIED** | Trit encoding at binary boundary not specified |
| No banned crypto primitives used | **VERIFIED** | Spec does not reference SHA-256, BLAKE3, Ed25519, or AES-256-GCM |

---

## Summary Verdict

**FAIL** — One CRITICAL finding (R1-A3-1).

The Continuous Attestation Service specification correctly selects TL-DSA as the signature primitive and correctly identifies HPTP as the timing source, demonstrating awareness of the PlenumNET cryptographic stack. The operational design — partition-aware suspicion counters, schema versioning, per-link bandwidth budgeting — is well-considered and integrates cleanly with the existing FTS state machine and Array3 monitoring infrastructure. However, the spec omits the Rep C identity binding in the TL-DSA signing context (R1-A3-1), which is a CRITICAL violation of INVARIANT 7 and INVARIANT 9: without binding the signer's Rep C address into the signature context, attestation signatures are not topologically bound and could be replayed or misattributed across nodes. Additionally, six IMPORTANT findings identify missing hash primitive specifications (R1-A3-3, R1-A3-4, R1-A3-7), missing binary-to-ternary encoding requirements at the boot measurement boundary (R1-A3-5), an undefined signing context string (R1-A3-2), and an undefined key derivation path from PUF material (R1-A3-6). These omissions must be resolved before implementation can proceed. The CRITICAL finding (R1-A3-1) blocks implementation; the IMPORTANT findings must be resolved before first release.
