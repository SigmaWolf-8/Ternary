# QC-R1 — Agent 1: Security Engineer Review

**Document:** NinjaExec — PlenumNET Local Signing Agent (Task #54)
**Source files reviewed:** `ninja-exec/src/signing_engine.rs`, `keystore.rs`, `server.rs`, `confirm.rs`, `audit.rs`, `config.rs`, `cli.rs`, `main.rs`, `Cargo.toml`, `plenum-app.toml`
**Crate version:** 1.0.0
**Review date:** 2026-03-28
**Finding ID convention:** R1-A1-{N}
**Reviewer role:** Security Engineer (engineering/security-engineer)

---

## Findings

### Finding 1
- **Section:** signing_engine.rs, lines 13-18 (sign / verify functions)
- **Severity:** CRITICAL
- **Finding:** TL-DSA `sign()` and `verify()` calls do not bind a Rep C address into the signing context string. INVARIANT 9 requires that all TL-DSA signing contexts include the signer's Rep C address. The current implementation passes the raw payload directly to `tl_dsa::sign(secret_key, payload, VARIANT)` with no context string containing a Rep C address. This means signatures are not bound to any node identity and could be replayed across nodes.
- **Recommendation:** Add a `context: &str` parameter to `sign()` and `verify()` that includes the signer's Rep C 54-trit address. Construct a domain-separated message: `domain_sep = b"NinjaExec-TL-DSA:" || rep_c_address || b":" || payload`, and pass that as the message to `tl_dsa::sign`. Do the same in `verify`. Update all call sites (server.rs `handle_sign`, main.rs `Command::SignFile`) to supply the Rep C address. Alternatively, use TL-DSA's context-string parameter if the `tl_dsa` API supports one.
- **Verification:** Grep for all `tl_dsa::sign` and `tl_dsa::verify` call sites and confirm every one includes a Rep C address in the context/message. Unit tests must assert that a signature produced with one Rep C address fails verification with a different Rep C address.

### Finding 2
- **Section:** signing_engine.rs, line 27 (fingerprint function)
- **Severity:** MINOR
- **Finding:** The `fingerprint()` function uses `ternary_math::sponge::derive_key` with domain separator `b"NinjaExec-FP"` and a 16-byte output. This is acceptable for display purposes. However, no Rep C address is bound into the fingerprint context.
- **Recommendation:** Consider binding the Rep C address into the fingerprint domain separator for consistency with INVARIANT 9, though this is non-security-critical since the fingerprint is for human display only.
- **Verification:** Confirm fingerprint is never used as a security-critical identity binding.

### Finding 3
- **Section:** keystore.rs, lines 67-83 (derive_enc_key — KDF)
- **Severity:** IMPORTANT
- **Finding:** The KDF uses `KDF_ITERATIONS = 4096` rounds of TLSponge-385 `derive_key`. While TLSponge is computationally heavier than SHA-256 per round, 4096 iterations is low for a passphrase-based KDF. Modern passphrase KDFs (Argon2id, scrypt) target ≥100ms wall-clock time. At 4096 rounds, the KDF may complete in under 10ms, making offline brute-force attacks against the keystore significantly faster. Additionally, the KDF does not include a memory-hard component, making GPU-based attacks feasible.
- **Recommendation:** Increase `KDF_ITERATIONS` to at least 100,000 or add a configurable cost parameter that targets ≥100ms on the deployment hardware. Alternatively, integrate Argon2id as the outer KDF and use TLSponge for inner domain separation only. Document the chosen cost target and rationale. Since the iteration count is stored in the keystore header, existing keystores can be migrated by re-encrypting on next unlock.
- **Verification:** Benchmark `derive_enc_key` with the new iteration count on target hardware (x86_64, aarch64) and confirm ≥100ms wall-clock time. Verify that the iteration count stored in the keystore header is read and honored on open.

### Finding 4
- **Section:** server.rs, lines 614-631 (build_router — CORS policy)
- **Severity:** CRITICAL
- **Finding:** The CORS layer is configured with `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. This means any website in the user's browser can send cross-origin requests to the localhost signing agent. A malicious or compromised web page could issue signing requests to `http://127.0.0.1:21027/sign` and, if the keystore is unlocked and running in headless mode, obtain valid TL-DSA signatures over attacker-controlled payloads without any user interaction.
- **Recommendation:** Replace `allow_origin(Any)` with an explicit allowlist of trusted origins. At minimum, restrict to the specific YODA dashboard origin(s) (e.g., `https://yoda.replit.app`). The allowlist should be configurable in `ninja-exec.json`. If no allowlist is configured, default to denying all cross-origin requests. Additionally, consider requiring an API key or bearer token for the `/sign` and `/unlock` endpoints (the `confirm_token` mechanism exists but is only enforced on `/confirm/*` endpoints).
- **Verification:** Attempt a cross-origin `fetch("http://127.0.0.1:21027/sign", ...)` from a page hosted on an untrusted origin and confirm it is rejected with a CORS error. Verify the allowlist is configurable and defaults to deny.

### Finding 5
- **Section:** server.rs, lines 463-525 (handle_unlock) and main.rs, lines 167, 305, 334, 427 (PLENUM_PASSPHRASE env var)
- **Severity:** IMPORTANT
- **Finding:** The passphrase can be supplied via the `PLENUM_PASSPHRASE` environment variable. On Linux, any process running as the same user can read `/proc/<pid>/environ` to extract this value. On Windows, environment variables are accessible to any process in the same session. The `handle_unlock` endpoint accepts the passphrase in a JSON POST body over plain HTTP (no TLS). While the server binds to 127.0.0.1, local processes can sniff loopback traffic on some OS configurations. The environment variable is never zeroed after use.
- **Recommendation:** (1) After reading `PLENUM_PASSPHRASE`, overwrite the environment variable with zeros using `std::env::set_var("PLENUM_PASSPHRASE", "")` (though this is imperfect since the OS may retain the original). Document that `PLENUM_PASSPHRASE` is a convenience mechanism for CI/automation only and must not be used in interactive security-sensitive environments. (2) For the HTTP unlock endpoint, consider requiring the `confirm_token` as an Authorization header, adding a second factor of authentication. (3) On Linux, document that `/proc/sys/kernel/yama/ptrace_scope` should be ≥1 and core dumps should be disabled. (4) Document macOS Keychain integration as a future enhancement.
- **Verification:** Confirm the passphrase environment variable is documented as CI-only. Confirm that `handle_unlock` requires the `confirm_token` header (or document the risk acceptance). Confirm platform-specific hardening guidance is included in deployment docs.

### Finding 6
- **Section:** server.rs, lines 533-548 (check_confirm_token — token comparison)
- **Severity:** IMPORTANT
- **Finding:** The `confirm_token` comparison uses `!=` (standard string comparison), which is not constant-time. This leaks the token length and content via timing side-channel. While the attack surface is localhost-only, a local attacker process could exploit timing differences to recover the confirm token byte-by-byte.
- **Recommendation:** Replace the `!=` comparison with a constant-time comparison. Use the same XOR-accumulate pattern already used in `keystore.rs` line 130-133 for tag comparison, or use a dedicated constant-time comparison crate (e.g., `subtle::ConstantTimeEq`). Ensure both operands are compared over their full length.
- **Verification:** Review the replacement code and confirm it uses constant-time comparison. Verify that timing measurements over 10,000 requests show no statistically significant difference between correct prefix and wrong prefix tokens.

### Finding 7
- **Section:** confirm.rs, lines 159-173 (evaluate_confirmation — headless mode)
- **Severity:** IMPORTANT
- **Finding:** When `headless = true`, ALL signing requests are auto-approved regardless of context, including destructive operations like `exec`, `model-swap`, `file-push`, `deploy`, `config-update`, `key-rotation`. Combined with the open CORS policy (Finding 4), any web page in the browser can silently obtain signatures for arbitrary payloads when the agent runs in headless mode.
- **Recommendation:** Even in headless mode, certain high-risk contexts (`exec`, `deploy`, `key-rotation`) should require explicit confirmation or at minimum require the `confirm_token` header on the `/sign` endpoint. Add a `headless_allow` configuration list that defaults to a safe subset (e.g., `sign`, `verify`, `pubkey`, `status`). Headless mode should still reject contexts not in the `headless_allow` list.
- **Verification:** Start the agent in headless mode. Issue a `/sign` request with context `exec: rm -rf /` and confirm it is rejected unless the context is in the `headless_allow` list.

### Finding 8
- **Section:** keystore.rs, lines 86-118 (encrypt_sk — authenticated encryption construction)
- **Severity:** IMPORTANT
- **Finding:** The keystore uses a custom Encrypt-then-MAC construction built from TLSponge-385 `derive_key`. While the construction follows the correct pattern (encrypt with XOR keystream, then MAC over ciphertext), it is a bespoke authenticated encryption scheme that has not been formally analyzed. The 12-byte nonce is generated per-encryption and the 16-byte tag provides 128-bit integrity — these are reasonable. However, INVARIANT 7 and the security review scope require that all authenticated encryption use PlenumNET primitives (T-AE-MAC). The current construction is a custom scheme that does not use T-AE-MAC.
- **Recommendation:** Replace the bespoke encrypt/MAC with T-AE-MAC (the PlenumNET authenticated encryption primitive) if it is available in `ternary_math`. If T-AE-MAC is not yet exposed with a suitable API for binary data at rest, document this as a known deviation with a risk acceptance and a migration path. The current construction is functionally reasonable but should be replaced with the canonical primitive.
- **Verification:** Confirm that `ternary_math` exposes a T-AE-MAC API. If so, replace the bespoke construction and verify round-trip encrypt/decrypt still works with existing unit tests. If not, verify the risk acceptance is documented.

### Finding 9
- **Section:** plenum-app.toml, line 10 (upgrade_code)
- **Severity:** IMPORTANT
- **Finding:** The `upgrade_code` is a hardcoded UUID `A1B2C3D4-E5F6-7890-ABCD-EF1234567890` that appears to be a placeholder, not a deterministically derived product code. The security review scope requires that product code derivation is deterministic and collision-resistant with a minimum collision probability bound of 2^-64. The current value is clearly a hand-typed placeholder and does not demonstrate derivation from inputs.
- **Recommendation:** Derive the upgrade code deterministically from the app name and publisher using the TIS-27 or TLSponge-385 hash of `"NinjaExec:Capomastro Holdings Ltd."`, then format as a UUID v5-style identifier. Document the derivation formula and collision resistance bound. The upgrade code must remain permanent once assigned — derive once and store.
- **Verification:** Reproduce the derivation on a clean machine and confirm byte-identical output. Verify the collision bound is documented as ≥2^-64.

### Finding 10
- **Section:** main.rs, line 193 (confirm token printed to stdout)
- **Severity:** IMPORTANT
- **Finding:** During `ninja-exec init`, the confirm token is printed to stdout: `println!("[NinjaExec] Token: {}", token)`. This token protects the confirmation API and should be treated as a secret. Printing it to stdout risks exposure in shell history, log files, CI output, and screen captures. The token is also stored in `ninja-exec.json` on disk with no explicit permission check.
- **Recommendation:** Do not print the confirm token to stdout. Instead, print only a message stating where the token is stored (`ninja-exec.json`) and advise the user to read it from there. Ensure `ninja-exec.json` has restrictive file permissions (0600 on Unix, ACL on Windows) similar to the keystore file.
- **Verification:** Run `ninja-exec init` and confirm the token is not displayed. Confirm `ninja-exec.json` has 0600 permissions on Unix.

### Finding 11
- **Section:** keystore.rs, lines 61-65 (zeroize function)
- **Severity:** MINOR
- **Finding:** The `zeroize()` function uses `write_volatile` to zero memory, which is a reasonable approach but relies on unsafe code. The Rust `zeroize` crate provides a safer, audited implementation that also handles compiler optimizations and register spilling. The current implementation does not call `compiler_fence` after zeroing, which means the compiler could theoretically reorder operations.
- **Recommendation:** Consider using the `zeroize` crate (which is widely audited) instead of the custom implementation. If keeping the custom implementation, add `std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst)` after the zeroing loop.
- **Verification:** Check that `compiler_fence` is present after every `zeroize()` call, or that the `zeroize` crate is integrated.

### Finding 12
- **Section:** server.rs, lines 37-59 (RateLimiter) and line 11 (shared state)
- **Severity:** MINOR
- **Finding:** The rate limiter uses a single shared bucket for all endpoints. The `/sign` and `/unlock` endpoints share the same 30 requests/minute limit. An attacker could exhaust the rate limit with `/sign` requests, causing legitimate `/unlock` attempts to be rate-limited (denial of service against the operator). The rate limiter stores timestamps in an unbounded `Vec` — a sustained burst could cause memory growth until GC.
- **Recommendation:** Use separate rate limiters for `/sign` and `/unlock`. The `/unlock` endpoint should have a much lower rate limit (e.g., 5 attempts/minute) to slow brute-force passphrase attacks. Bound the `timestamps` vector to `max_per_minute` entries.
- **Verification:** Confirm separate rate limiters exist for sign and unlock. Send 30 sign requests, then attempt unlock and confirm it is not rate-limited.

### Finding 13
- **Section:** audit.rs, lines 38-50 (AuditLog::append)
- **Severity:** MINOR
- **Finding:** Audit log writes silently swallow errors (`let _ = writeln!(...)`). If the audit log file cannot be written (disk full, permissions), operations continue without any audit trail. The audit log file has no explicit permission restriction.
- **Recommendation:** Return a `Result` from `append` and propagate audit write failures to the caller. At minimum, log audit write failures to stderr. Set file permissions on the audit log to 0600 (Unix) or restricted ACL (Windows) on creation.
- **Verification:** Confirm that audit write failures are surfaced. Confirm file permissions on the audit log.

### Finding 14
- **Section:** config.rs, lines 58-68 (save_default) and lines 70-94 (generate_confirm_token)
- **Severity:** MINOR
- **Finding:** The `ninja-exec.json` config file is written without explicit file permission restrictions. On Unix, the default umask may result in world-readable permissions. This file contains the `confirm_token` secret.
- **Recommendation:** Set 0600 permissions on `ninja-exec.json` immediately after creation, similar to the keystore file.
- **Verification:** Create a new config file and verify its permissions are 0600 on Unix.

### Finding 15
- **Section:** signing_engine.rs (entire module); INVARIANT 9 cross-reference
- **Severity:** CRITICAL
- **Finding:** No Rep C address is defined, stored, or used anywhere in the NinjaExec codebase. INVARIANT 9 requires that all cryptographic operations binding node identity use Rep C (54-trit, binary-encoded) addressing exclusively. NinjaExec does not associate the signing key with a Rep C address, does not include a Rep C address in TL-DSA signing context, does not include a Rep C address in TLSponge-385 KDF domain separation for the keystore, and does not include a Rep C address in fingerprint computation. Without Rep C binding, signatures produced by NinjaExec cannot be verified against a registered TDNS address.
- **Recommendation:** (1) During `ninja-exec init`, derive or accept a Rep C 54-trit TDNS address for this node. Store the Rep C address in the keystore file or a companion identity file. (2) Bind the Rep C address into all TL-DSA signing contexts (see Finding 1). (3) Bind the Rep C address into the KDF domain separator. (4) Bind the Rep C address into the fingerprint domain separator. (5) Include the Rep C address in the operator export JSON.
- **Verification:** Grep for `rep_c` or equivalent in all source files and confirm it appears in every cryptographic context string. Verify the Rep C address is stored persistently and included in operator export output.

---

## Cryptographic Claims Verification

| Claim | Location | Verdict | Notes |
|---|---|---|---|
| TL-DSA-87 used for all signatures | signing_engine.rs line 7 | **VERIFIED** | `TlDsaVariant::TlDsa87` is hardcoded; all sign/verify calls go through `tl_dsa` module |
| No Ed25519 / no `crypto.sign` | Full codebase | **VERIFIED** | No Ed25519, no Node.js crypto calls found in Rust sources |
| TLSponge-385 used for KDF | keystore.rs line 71 | **VERIFIED** | `ternary_math::sponge::derive_key` is used with domain separator |
| TLSponge-385 used for audit hash | audit.rs line 60 | **VERIFIED** | `ternary_math::sponge::derive_key` with domain `b"NinjaExec-AUDIT-HASH"` |
| TLSponge-385 used for fingerprint | signing_engine.rs line 27 | **VERIFIED** | `ternary_math::sponge::derive_key` with domain `b"NinjaExec-FP"` |
| Keystore authenticated encryption uses PlenumNET primitives only | keystore.rs lines 86-153 | **VERIFIED** | Custom Encrypt-then-MAC using TLSponge; no AES-256-GCM, no external crypto |
| Rep C address bound into signing context (INVARIANT 9) | signing_engine.rs lines 13-18 | **INCORRECT** | No Rep C address appears in any signing context |
| Rep C address bound into KDF domain separation (INVARIANT 9) | keystore.rs line 71 | **INCORRECT** | KDF domain is `b"NinjaExec-KDF-v2"` with no Rep C address |
| Constant-time tag comparison | keystore.rs lines 130-133 | **VERIFIED** | XOR-accumulate pattern used |
| Constant-time confirm_token comparison | server.rs line 540 | **INCORRECT** | Standard `!=` string comparison used |
| No raw binary integers enter sponge absorb (INVARIANT 8) | keystore.rs, audit.rs, signing_engine.rs | **UNVERIFIED** | Inputs are byte slices (passphrase, salt, key material); whether `derive_key` internally handles trit encoding depends on the `ternary_math::sponge` implementation, which is outside this crate |
| Upgrade code is deterministically derived with ≥2^-64 collision bound | plenum-app.toml line 10 | **INCORRECT** | Hardcoded placeholder UUID with no derivation formula |

---

## passphrase_entropy_minimum_bits

**Value:** 72 bits minimum

**Rationale:** NinjaExec enforces a 12-character minimum passphrase length. Assuming a realistic passphrase drawn from printable ASCII (95 characters), 12 characters yield `12 × log₂(95) ≈ 78.8 bits` of entropy in the best case. However, human-chosen passphrases are significantly weaker — NIST SP 800-63B estimates ~1-2 bits per character for user-chosen passwords beyond the first 8 characters. A 12-character user-chosen passphrase may have as few as ~30 bits of effective entropy. The minimum acceptable entropy for protecting a long-lived signing key should be 72 bits (comparable to a 128-bit security target with a 2^56 KDF work factor). The current KDF iteration count of 4096 provides only ~12 bits of work factor, meaning the effective security is `passphrase_entropy + 12 bits`. To reach 72 bits effective with 12 bits of KDF work, the passphrase itself must provide ~60 bits, which requires either a longer minimum (e.g., 16 characters of mixed case + digits + symbols) or a passphrase generation recommendation (e.g., 5-word Diceware). Alternatively, increasing KDF iterations to ~2^20 (1M) would provide ~20 bits of work factor, requiring ~52 bits of passphrase entropy (12 mixed-case characters).

---

## Summary Verdict

**FAIL**

Three CRITICAL findings block implementation:

1. **R1-A1-1 / R1-A1-15:** TL-DSA signing context does not bind the signer's Rep C address, violating INVARIANT 9. No Rep C address exists anywhere in the codebase. Signatures produced by NinjaExec cannot be cryptographically bound to a TDNS identity, which is a fundamental requirement of the PlenumNET security model.

2. **R1-A1-4:** The CORS policy allows any origin to access the localhost signing API. Combined with headless mode auto-approval, any website in the operator's browser can obtain TL-DSA signatures over attacker-controlled payloads. This is a remotely exploitable signature oracle.

Six IMPORTANT findings require resolution before first release: weak KDF iteration count (R1-A1-3), environment variable passphrase exposure (R1-A1-5), non-constant-time token comparison (R1-A1-6), headless mode auto-approves all contexts including destructive operations (R1-A1-7), bespoke authenticated encryption instead of T-AE-MAC (R1-A1-8), placeholder upgrade code (R1-A1-9), and confirm token printed to stdout (R1-A1-10).

The CRITICAL findings must be resolved before NinjaExec can be deployed in any PlenumNET environment. The INVARIANT 9 violation is systemic — it affects every signing operation — and requires architectural changes (Rep C address provisioning, context binding, persistent identity storage).
