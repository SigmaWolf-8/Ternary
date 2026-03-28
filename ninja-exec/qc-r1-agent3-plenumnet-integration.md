# QC-R1 — Agent 3: PlenumNET Integration Specialist Review

**Document:** NinjaExec — PlenumNET Local Signing Agent (Task #54)
**Source Files:** `ninja-exec/src/{signing_engine,keystore,server,confirm,audit,config,cli,main}.rs`, `ninja-exec/Cargo.toml`, `ninja-exec/plenum-app.toml`
**Version:** 1.0.0
**Reviewer:** Agent 3 — PlenumNET Integration Specialist (`capomastro/plenumnet-integration`)
**Date:** 2026-03-28
**Finding ID Convention:** R1-A3-{N}

---

## Findings

### Finding R1-A3-1
- **Section:** `signing_engine.rs`, lines 13–18 (`sign` / `verify` functions)
- **Severity:** CRITICAL
- **Finding:** TL-DSA signing and verification do not bind the signer's Rep C address into the signature context string. INVARIANT 9 requires: "The signer's Rep C address must be bound into the signature context string. Signature verification must check the signer's public key against a registered Rep C address." The current `sign()` function passes `(secret_key, payload, VARIANT)` directly to `tl_dsa::sign` with no context string and no Rep C identity binding. The `verify()` function similarly performs raw verification with no address check.
- **Recommendation:** Extend the signing API to require a Rep C address (54-trit, binary-encoded) as a mandatory parameter. Construct a domain-separated context: `"NinjaExec-SIGN-v1.0" ‖ rep_c_address ‖ payload` and pass this composite message to `tl_dsa::sign`. On verification, require the claimed signer's Rep C address, reconstruct the same composite, and verify against a registered key. The `SignRequest` struct must include a `signer_address` field and the server must validate it against the keystore's registered Rep C address.
- **Verification:** Confirm that `signing_engine::sign()` and `signing_engine::verify()` accept a Rep C address parameter, that the address is concatenated into the signed message before calling `tl_dsa::sign`/`tl_dsa::verify`, and that the server rejects requests where the signer address does not match the keystore's registered identity.

### Finding R1-A3-2
- **Section:** `signing_engine.rs`, line 27 (`fingerprint` function)
- **Severity:** IMPORTANT
- **Finding:** The fingerprint derivation uses the context string `"NinjaExec-FP"` with `ternary_math::sponge::derive_key`. This context string is not present in the canonical context string registry in `plenumnet-repo-guide/SKILL.md`. All context strings used in TIS-27 / TLSponge-385 derivation are load-bearing — a wrong or unregistered context string produces a wrong key. Status: **UNVERIFIED**.
- **Recommendation:** Register `"NinjaExec-FP"` in the canonical context string registry with its purpose documented (public key fingerprint derivation, 16-byte output, used for operator display only — not security-critical). Until registered, mark this context string as provisional.
- **Verification:** Confirm the context string appears in the canonical registry with matching parameters (output length = 16 bytes, input = public key bytes).

### Finding R1-A3-3
- **Section:** `keystore.rs`, lines 67–84 (`derive_enc_key` function)
- **Severity:** IMPORTANT
- **Finding:** The KDF uses context string `"NinjaExec-KDF-v2"` with `ternary_math::sponge::derive_key`. This context string is not in the canonical registry. Status: **UNVERIFIED**. Additionally, the iterated KDF construction (4096 rounds of sponge derivation) is a custom construction that has not been formally analyzed. While it follows a reasonable pattern (iterated hash with salt), the construction differs from the standard PlenumNET key derivation patterns documented in the repo guide.
- **Recommendation:** (1) Register `"NinjaExec-KDF-v2"` in the canonical context string registry. (2) Document the iterated KDF construction in a brief security note, referencing the sponge's 385-bit security level and the iteration count rationale (4096 iterations for passphrase stretching). (3) Cross-reference to Security Engineer (Agent 1) for formal assessment of the iterated sponge KDF construction.
- **Verification:** Confirm context string registration. Confirm Agent 1 has reviewed the iterated KDF construction and signed off on its adequacy for passphrase-based key protection.

### Finding R1-A3-4
- **Section:** `keystore.rs`, lines 95–118 (`encrypt_sk` function)
- **Severity:** IMPORTANT
- **Finding:** Two additional unregistered context strings: `"NinjaExec-KS-STREAM"` (keystream derivation) and `"NinjaExec-KS-TAG"` (authentication tag derivation). Both are used with `ternary_math::sponge::derive_key`. Status: **UNVERIFIED**. The encrypt-then-MAC construction (XOR keystream + separate tag) is a custom authenticated encryption scheme rather than using the PlenumNET standard T-AE-MAC primitive. Per the Critical Rules: "AES-256-GCM must be replaced with Phase Encryption (data at rest) or TLSponge T-AE-MAC (authenticated encryption)." While this is not AES-256-GCM, it is a custom construction that should use T-AE-MAC instead.
- **Recommendation:** (1) Register both context strings in the canonical registry. (2) Evaluate replacing the custom XOR-stream + sponge-tag construction with T-AE-MAC for keystore encryption, which provides IND-CPA + INT-CTXT with a formally analyzed construction. (3) If the custom construction is retained, document the security proof sketch and obtain Agent 1 sign-off.
- **Verification:** Confirm context strings are registered. Confirm either T-AE-MAC is used or the custom construction has been formally reviewed and approved.

### Finding R1-A3-5
- **Section:** `audit.rs`, line 60 (`hash_payload` function)
- **Severity:** MINOR
- **Finding:** The audit hash uses context string `"NinjaExec-AUDIT-HASH"` with `ternary_math::sponge::derive_key`. Status: **UNVERIFIED** (not in canonical registry). This is used for audit log display only (prefixed `"tis27:"`), not for security-critical operations. The prefix `"tis27:"` is slightly misleading — the actual primitive called is `ternary_math::sponge::derive_key`, which is the kernel sponge (TLSponge-385), not TIS-27 (54-trit, 4 rounds). TIS-27 has a different state size and round count.
- **Recommendation:** (1) Register `"NinjaExec-AUDIT-HASH"` in the canonical registry. (2) Correct the prefix from `"tis27:"` to `"sponge385:"` or document why the `tis27` label is used (if `derive_key` dispatches to TIS-27 internally, verify this claim against the `ternary_math` crate).
- **Verification:** Confirm the hash prefix accurately reflects the underlying primitive. Confirm context string registration.

### Finding R1-A3-6
- **Section:** `keystore.rs` / `signing_engine.rs` / `server.rs` — entire codebase
- **Severity:** CRITICAL
- **Finding:** No Rep C address is stored, derived, or used anywhere in NinjaExec. INVARIANT 9 requires: "All cryptographic operations that bind node identity or address must use Rep C (54-trit, binary-encoded) addressing exclusively." This applies to TL-DSA signing context (item a), TLSponge-385 key derivation domain-separation input (item b), and T-AE-MAC associated data (item c). The keystore stores only a raw TL-DSA keypair with no associated Rep C address. The signing API accepts an opaque payload with no identity binding. The `export-operator` command exports a hostname-based name (`operator@{hostname}`) which explicitly violates INVARIANT 9: "No cryptographic operation may use hostname, IP address, Windows SID, or any non-Rep-C identifier as an identity binding."
- **Recommendation:** (1) During `ninja-exec init`, derive or accept a Rep C address for the operator and store it alongside the keypair in the keystore file. (2) Bind the Rep C address into all TL-DSA signing contexts (see Finding 1). (3) Replace the hostname-based `operator@{hostname}` identifier in `export-operator` with the operator's Rep C address. The hostname may be retained as a display hint but must not serve as a cryptographic identity binding. (4) Add Rep C address to the `/pubkey` and `/status` API responses.
- **Verification:** Confirm that `Keystore` stores and exposes a Rep C address. Confirm the signing context includes the Rep C address. Confirm `export-operator` output includes Rep C address as the primary identifier.

### Finding R1-A3-7
- **Section:** `server.rs`, lines 615–618 (CORS configuration)
- **Severity:** IMPORTANT
- **Finding:** The CORS layer uses `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. While NinjaExec binds to 127.0.0.1 only, the `Any` origin policy means any browser tab on the local machine (including a compromised website) can issue signing requests to the agent. This is a credential exposure risk — a malicious page could trigger arbitrary signing operations. Cross-reference to Security Engineer (Agent 1) for severity assessment.
- **Recommendation:** Restrict `allow_origin` to a known list of PlenumNET origins (e.g., `https://yoda.replit.app`, `http://localhost:*`). At minimum, do not use `Any` — use a configurable allowlist stored in `ninja-exec.json`. This is especially critical given that headless mode auto-approves all requests.
- **Verification:** Confirm CORS origin is restricted to a configurable allowlist. Confirm the default configuration does not include wildcard origins.

### Finding R1-A3-8
- **Section:** `server.rs`, lines 147–323 (`handle_sign` function) / `signing_engine.rs`, line 14
- **Severity:** IMPORTANT
- **Finding:** The `sign` function in `signing_engine.rs` passes the raw payload bytes directly to `tl_dsa::sign`. There is no domain separation between different operation contexts (e.g., `exec` vs `deploy` vs `key-rotation`). While the server validates the `context` field against `VALID_CONTEXTS`, this context is never incorporated into the signed message. An attacker who obtains a valid signature for a `verify` context could potentially replay it as a `deploy` context if the payload happens to match.
- **Recommendation:** Incorporate the operation context into the signed message: `context_bytes ‖ 0x00 ‖ payload`. This ensures signatures are bound to their intended operation type and cannot be replayed across contexts.
- **Verification:** Confirm that `signing_engine::sign()` receives the operation context and incorporates it into the message before signing. Confirm `verify` reconstructs the same composite.

### Finding R1-A3-9
- **Section:** `config.rs`, line 10 (`DEFAULT_PORT`)
- **Severity:** MINOR
- **Finding:** The default port 21027 is used. The number 21027 does not appear to be derived from any PlenumNET geometric constant. While port selection is not a cryptographic operation, PlenumNET convention suggests deriving operational constants from the ternary framework where possible.
- **Recommendation:** Document the rationale for port 21027. If it was chosen to avoid conflicts, document that. If a geometrically-derived port is desired, consider 21013 (21000 + 13, where 13 = T₇) or similar.
- **Verification:** Confirm port rationale is documented in the spec or code comments.

---

## Cryptographic Claim Verification

| Claim | Location | Status | Notes |
|-------|----------|--------|-------|
| TL-DSA-87 for all signatures | `signing_engine.rs` line 7 | **VERIFIED** | Uses `TlDsaVariant::TlDsa87` via `ternary_math::tl_dsa` |
| No Ed25519 / no external crypto | `Cargo.toml` dependencies | **VERIFIED** | No `ed25519`, `ring`, `openssl`, `sha2`, `blake3` crates. Only `ternary-math` for crypto |
| TLSponge for key derivation | `keystore.rs` line 71 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` (kernel sponge) |
| TLSponge for keystream generation | `keystore.rs` line 98 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| TLSponge for authentication tag | `keystore.rs` line 111 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| TLSponge for audit hashing | `audit.rs` line 60 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| TLSponge for fingerprinting | `signing_engine.rs` line 27 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| Rep C address binding in signatures | All source files | **INCORRECT** | No Rep C address is used anywhere (INVARIANT 9 violation) |
| Context string `"NinjaExec-FP"` | `signing_engine.rs` line 27 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KDF-v2"` | `keystore.rs` line 71 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KS-STREAM"` | `keystore.rs` line 98 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KS-TAG"` | `keystore.rs` line 111 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-AUDIT-HASH"` | `audit.rs` line 60 | **UNVERIFIED** | Not in canonical registry |
| "Level 5 post-quantum security" | `main.rs` line 153, `plenum-app.toml` | **VERIFIED** | TL-DSA-87 corresponds to NIST PQ Level 5 |
| No AES-256-GCM | All source files | **VERIFIED** | Custom sponge-based AE used instead; however, T-AE-MAC would be preferred |
| Constant-time tag comparison | `keystore.rs` lines 130–134 | **VERIFIED** | Uses OR-accumulation pattern for constant-time comparison |
| Key zeroization on drop | `keystore.rs` lines 316–319 | **VERIFIED** | `Drop` impl calls `lock()` which calls `zeroize()` with `write_volatile` |

---

## Summary Verdict

**FAIL**

NinjaExec correctly selects TL-DSA-87 as its sole signature primitive (VERIFIED), uses the kernel sponge (`ternary_math::sponge::derive_key`) for all hash/MAC/KDF operations with zero external crypto dependencies (VERIFIED), implements proper key zeroization and constant-time tag comparison, and binds exclusively to localhost. These are solid foundations.

However, two CRITICAL findings block implementation. First, **INVARIANT 9 is systematically violated**: no Rep C address exists anywhere in the codebase — not in the keystore, not in the signing context, not in the exported operator identity. All cryptographic operations that bind node identity must use Rep C (54-trit, binary-encoded) addressing exclusively, and NinjaExec currently uses no identity binding at all (Finding 1, Finding 6). Second, **TL-DSA signatures lack context binding**: the signer's Rep C address is not bound into the signature context string, making signatures identity-unbound and potentially replayable across operators (Finding 1). Additionally, five context strings used in sponge derivation are UNVERIFIED against the canonical registry, the keystore uses a custom authenticated encryption construction rather than the standard T-AE-MAC primitive, operation context is not incorporated into signatures enabling cross-context replay, and the CORS policy permits any origin to issue signing requests. These IMPORTANT findings (Findings 2–5, 7–8) must be resolved before first release but do not independently block implementation given that the CRITICAL findings already require significant rework.
