# QC-R1 Security Engineer Review — Task 119: Continuous Attestation Service

**Reviewer:** Agent 1 — Security Engineer
**YODA Role ID:** `engineering/security-engineer`
**Spec file:** `.local/tasks/continuous-attestation-service.md`
**Spec revision:** Gap 9 (no version number in document)
**Review protocol version:** 1.1.2
**Date:** 2026-07-08

---

### Finding 1
- **ID:** R1-A1-1
- **Section:** Specification (paragraph 1) — TL-DSA signing context
- **Severity:** CRITICAL
- **Finding:** The spec states attestation reports are "TL-DSA signed using the PUF-derived root key" and include HPTP timestamps, but does not specify the TL-DSA signing context string. Per INVARIANT 7 and INVARIANT 9, the signer's Rep C address must be bound into the signing context string. The spec never defines what context string is used, what fields are included in the signed payload beyond the report contents, or how the signer's Rep C address is bound. A developer could implement TL-DSA signing with an empty or generic context string, enabling cross-context signature replay attacks (e.g., an attestation signature replayed as a different message type).
- **Recommendation:** Add an explicit signing context string definition, e.g., `"plenumnet:attestation:v1:<signer_rep_c_54trit>"`. Specify that the signer's 54-trit Rep C address is concatenated into the context string before signing. Specify that verifiers must reconstruct the expected context string from the sender's registered Rep C address and reject signatures with mismatched contexts.
- **Verification:** Confirm the spec includes a concrete context string template with the signer's Rep C address embedded. Confirm verification logic reconstructs the context from the sender's known Rep C address.

### Finding 2
- **ID:** R1-A1-2
- **Section:** Specification (paragraph 1) — Kernel integrity hash
- **Severity:** CRITICAL
- **Finding:** The spec includes "kernel integrity hash" as a field in the attestation report but does not specify which hash primitive is used to compute it. Per the security-engineer review scope (§2 Cryptographic correctness), all cryptographic operations must use PlenumNET primitives exclusively (TIS-27, TLSponge-385). If a developer uses SHA-256, BLAKE3, or any non-PlenumNET hash to compute the kernel integrity measurement, it violates the cryptographic stack requirements. The spec must explicitly state that TLSponge-385 (or TIS-27 for lightweight integrity) is used for computing the kernel integrity hash.
- **Recommendation:** Add: "The kernel integrity hash is computed using TLSponge-385 over the kernel binary image. No external hash primitives (SHA-256, BLAKE3, etc.) are permitted." Specify whether the hash covers the entire kernel image or specific sections, and whether it is computed at boot time or periodically.
- **Verification:** Search the implementation for any import/use of SHA-256, BLAKE3, or other non-PlenumNET hash functions in the attestation code path. Confirm TLSponge-385 is the sole hash primitive.

### Finding 3
- **ID:** R1-A1-3
- **Section:** Specification (paragraph 1) — Rolling Merkle tree
- **Severity:** IMPORTANT
- **Finding:** The spec describes a "rolling Merkle tree of heartbeat challenges responded to" but does not specify the hash function used for Merkle tree node computation. Standard Merkle trees use SHA-256 or similar — using any non-PlenumNET hash violates the cryptographic stack requirement. Additionally, the spec does not address second-preimage attacks on the Merkle tree (e.g., domain separation between leaf nodes and internal nodes). Without domain separation, an attacker could substitute an internal node hash for a leaf, forging a proof-of-inclusion.
- **Recommendation:** (a) Explicitly state that Merkle tree nodes are computed using TLSponge-385 (or TIS-27 if lightweight integrity is sufficient). (b) Require domain separation: leaf nodes are hashed with a `0x00` prefix (or equivalent trit-encoded tag), internal nodes with a `0x01` prefix. (c) Specify that challenge values fed into the Merkle tree are trit-encoded (Rep C or Rep A) before hashing, per INVARIANT 8.
- **Verification:** Confirm the implementation uses TLSponge-385/TIS-27 for Merkle node hashing with domain-separated leaf vs. internal node prefixes. Verify no raw binary integers enter the sponge absorb (INVARIANT 8).

### Finding 4
- **ID:** R1-A1-4
- **Section:** Specification (paragraph 2) — Suspicion counter and FTS integration
- **Severity:** IMPORTANT
- **Finding:** The spec states "crossing the configured suspicion threshold triggers FTS Suspect transition" but does not specify how the suspicion threshold itself is protected from unauthorized modification. If the threshold is stored in PlenumConfig as a plain configurable value, a compromised operator or configuration injection attack could raise the threshold to effectively disable attestation-based suspicion. The spec also does not specify whether the threshold change requires a signed configuration update (TL-DSA signed by an authorized administrator key).
- **Recommendation:** Specify that the suspicion threshold is part of a TL-DSA-signed configuration block. Changes to the suspicion threshold must be signed by an authorized configuration authority (identified by Rep C address). The attestation service must verify the configuration signature before accepting threshold changes. Define a minimum floor for the threshold (e.g., ≥1) to prevent disabling attestation suspicion entirely.
- **Verification:** Confirm the implementation rejects unsigned or invalidly-signed configuration updates to the suspicion threshold. Confirm a minimum threshold floor is enforced.

### Finding 5
- **ID:** R1-A1-5
- **Section:** Specification (paragraph 1) — PUF-derived root key exposure
- **Severity:** IMPORTANT
- **Finding:** The spec states attestation reports are signed using "the PUF-derived root key" but does not address key exposure risks. Per the security-engineer review scope (§5 Key provisioning), the spec must verify that the signing key material does not leak to disk, logs, or crash dumps. The PUF-derived root key is the highest-privilege key on the node — using it directly for attestation signing (a periodic, high-frequency operation at 30–120s intervals) increases its exposure surface. The spec does not specify whether a derived signing subkey is used, or whether the PUF root key itself signs every attestation report. It also does not specify memory zeroing after signing operations or crash dump protections.
- **Recommendation:** (a) Derive a dedicated attestation signing subkey from the PUF root key using TLSponge-385 key derivation with a domain-separation context string (e.g., `"plenumnet:attestation-signing-key:<rep_c_address>"`). (b) Specify that the PUF root key is used only for the one-time derivation, not for direct signing of attestation reports. (c) Require memory zeroing of the signing key material after each signing operation. (d) Specify that crash dump policy must exclude the attestation signing key memory region (or that crash dumps are disabled on production nodes).
- **Verification:** Confirm the implementation derives a subkey rather than using the PUF root key directly. Confirm memory zeroing calls after signing. Confirm crash dump policy documentation.

### Finding 6
- **ID:** R1-A1-6
- **Section:** Specification (paragraph 3) — Report versioning and schema version registry
- **Severity:** IMPORTANT
- **Finding:** The spec states "the version registry is maintained in PlenumConfig and updated as part of the firmware release process" but does not specify how the version registry itself is authenticated. If the version registry can be modified by an attacker (e.g., by injecting a malicious schema version), a compromised node could register an attacker-controlled version that causes neighbors to accept maliciously crafted attestation reports as "known version" instead of flagging them as "unparseable." The spec also does not specify whether version registry updates are TL-DSA signed.
- **Recommendation:** Specify that the version registry in PlenumConfig is a TL-DSA-signed artifact. Each version entry must include the schema hash (computed via TLSponge-385) and be signed by an authorized release authority (identified by Rep C address). Nodes must verify the signature on the version registry before accepting it.
- **Verification:** Confirm the version registry is signed and verified before use. Confirm schema hashes are computed with TLSponge-385.

### Finding 7
- **ID:** R1-A1-7
- **Section:** Specification (paragraph 1) — HPTP timestamp replay protection
- **Severity:** IMPORTANT
- **Finding:** The spec states attestation reports include HPTP timestamps but does not specify replay protection. An attacker who captures a valid attestation report could replay it to satisfy attestation requirements for a compromised node. The spec does not require: (a) a monotonically increasing sequence number in the signed payload, (b) a nonce or challenge binding, or (c) a receiver-side check that the HPTP timestamp is within an acceptable freshness window. HPTP timestamps alone are insufficient for replay protection unless receivers enforce a strict freshness bound.
- **Recommendation:** Add: (a) Each attestation report includes a monotonically increasing 64-bit sequence number, bound into the signed payload. Receivers reject reports with sequence numbers ≤ the last accepted sequence from that sender. (b) Receivers enforce an HPTP timestamp freshness window (e.g., report timestamp must be within 2× the maximum attestation interval of the current time). (c) The sequence number must be trit-encoded before inclusion in the signed payload per INVARIANT 8.
- **Verification:** Confirm the implementation includes and verifies a monotonic sequence number. Confirm freshness window enforcement on received attestation timestamps.

### Finding 8
- **ID:** R1-A1-8
- **Section:** Specification (paragraph 1) — "FTS/GLB configuration fingerprint"
- **Severity:** MINOR
- **Finding:** The spec includes "FTS/GLB configuration fingerprint" as an attestation report field but does not specify how the fingerprint is computed. The term "fingerprint" is ambiguous — it could be interpreted as a non-cryptographic hash (e.g., CRC32), a truncated hash, or a full TLSponge-385 digest. A developer could choose a weak fingerprint that is trivially forgeable.
- **Recommendation:** Specify: "The FTS/GLB configuration fingerprint is the TLSponge-385 digest of the canonicalized configuration state (sorted keys, deterministic serialization). No non-PlenumNET hash or checksum primitives are used."
- **Verification:** Confirm the implementation uses TLSponge-385 for configuration fingerprinting with deterministic input canonicalization.

### Finding 9
- **ID:** R1-A1-9
- **Section:** Specification (paragraph 4) — Bandwidth backoff and attestation storm
- **Severity:** MINOR
- **Finding:** The per-link exponential backoff mechanism during attestation storms could be exploited by an attacker who deliberately saturates a specific link to force attestation delays on that link. By sending junk traffic to push a target link over the 5% attestation bandwidth threshold, an attacker could suppress attestation reports between specific neighbors, creating a window where a compromised node is not attested. The spec does not distinguish between attestation traffic and non-attestation traffic contributing to link saturation.
- **Recommendation:** Clarify that the 5% threshold applies specifically to attestation-generated traffic, not total link utilization. If total link utilization is used, add a note that this creates a potential denial-of-attestation vector and specify mitigations (e.g., minimum attestation rate floor even under backoff, or priority queuing for attestation traffic).
- **Verification:** Confirm the implementation measures attestation-specific bandwidth, not total link bandwidth, for the backoff trigger. Alternatively, confirm a minimum attestation rate floor is enforced.

### Finding 10
- **ID:** R1-A1-10
- **Section:** Tasks (item 1) — Rep C address binding in attestation report
- **Severity:** IMPORTANT
- **Finding:** Per INVARIANT 9, all cryptographic operations binding node identity must use Rep C (54-trit, binary-encoded) addressing exclusively. The spec does not explicitly state that the attestation report includes the attesting node's Rep C address as a field in the signed payload. Without this, a neighbor cannot cryptographically bind the attestation to a specific node identity — the TL-DSA signature alone binds to a public key, but the public key must be mapped to a Rep C address, and that mapping must be part of the signed data to prevent key-substitution attacks.
- **Recommendation:** Add the attesting node's 54-trit Rep C address as a mandatory field in the attestation report structure, included in the signed payload. Verifiers must check that the Rep C address in the report matches the expected neighbor address and that the signing public key is registered to that Rep C address.
- **Verification:** Confirm the attestation report struct includes a Rep C address field. Confirm the verifier checks Rep C address against the expected neighbor and against the signing key's registered address.

### Finding 11
- **ID:** R1-A1-11
- **Section:** Specification — Attestation report confidentiality
- **Severity:** MINOR
- **Finding:** The spec states attestation reports are broadcast to all 26 geometric neighbors but does not specify whether reports are encrypted in transit. Attestation reports contain sensitive security metadata (boot measurements, kernel integrity hash, PUF self-test results, healing state) that could aid an attacker in fingerprinting node software versions and identifying vulnerable configurations. Broadcasting this data unencrypted over Inter-Cube links exposes it to passive observers.
- **Recommendation:** Specify that attestation reports are encrypted using T-AE-MAC or Phase Encryption for transit between neighbors. If the Inter-Cube link already provides encryption at the transport layer (TLSponge-385-based tunnel), state that explicitly and note that attestation confidentiality is inherited from the transport.
- **Verification:** Confirm attestation reports are transmitted over encrypted Inter-Cube tunnels or are individually encrypted before broadcast.

---

## Cryptographic Claims Assessment

| Claim | Status |
|-------|--------|
| TL-DSA signing of attestation reports | UNVERIFIED — context string not specified |
| TLSponge-385 for kernel integrity hash | UNVERIFIED — hash primitive not specified |
| Merkle tree hash function | UNVERIFIED — hash primitive not specified |
| PUF-derived root key usage | UNVERIFIED — key derivation hierarchy not specified |
| HPTP timestamp integrity | UNVERIFIED — replay protection not specified |
| FTS/GLB configuration fingerprint | UNVERIFIED — fingerprint computation not specified |
| Rep C address binding in signed payload | UNVERIFIED — not explicitly required |

## Summary Verdict

**FAIL**

The Continuous Attestation Service specification (Gap 9) demonstrates strong architectural design with partition-aware suspicion counters, bandwidth budgeting, and schema versioning. However, it contains two CRITICAL findings that block implementation: (1) the TL-DSA signing context string is not defined, violating INVARIANT 7 and INVARIANT 9, which could enable cross-context signature replay attacks; and (2) the kernel integrity hash primitive is not specified, leaving open the possibility of a developer using SHA-256 or another prohibited non-PlenumNET hash function. Additionally, six IMPORTANT findings address missing replay protection, PUF root key overexposure, unsigned configuration thresholds, unsigned version registries, Rep C address binding omission, and Merkle tree domain separation. All cryptographic claims are UNVERIFIED due to insufficient specification of primitives, context strings, and encoding requirements. The spec must be revised to explicitly specify all cryptographic primitives, context strings, key derivation hierarchies, and Rep C address bindings before implementation can proceed.
