# QC-R2 Consolidated Review — NinjaExec (Task #54)

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Review Date:** 2026-03-28
**Protocol:** QC-R2 (Round 2 — Quality & Completeness)
**Input:** QC-R1 Consolidated Findings (4 CRITICAL, 12 IMPORTANT, 6 MINOR)
**Reviewers:**
- Agent 4: Evidence Collector / QA Lead
- Agent 5: Senior Developer
- Agent 6: Infrastructure Maintainer

---

## Overall Verdict: **FAIL**

All three QC-R2 reviewers independently returned **FAIL**. All four QC-R1 CRITICAL findings (C1–C4) were unanimously confirmed as **AGREE** by every Round 2 agent. Combined with 3 new CRITICAL findings from Round 2 (C5–C7), NinjaExec has 7 total CRITICAL findings, 20 IMPORTANT findings, and 13 MINOR findings across both rounds. It cannot proceed to QC-R3.

---

# PART I — QC-R1 FINDINGS (ALL AGENT COMMENTS VERBATIM)

## Agent 1: Security Engineer — Full Findings

### R1-A1-1 (CRITICAL)
- **Section:** signing_engine.rs, lines 13-18 (sign / verify functions)
- **Severity:** CRITICAL
- **Finding:** TL-DSA `sign()` and `verify()` calls do not bind a Rep C address into the signing context string. INVARIANT 9 requires that all TL-DSA signing contexts include the signer's Rep C address. The current implementation passes the raw payload directly to `tl_dsa::sign(secret_key, payload, VARIANT)` with no context string containing a Rep C address. This means signatures are not bound to any node identity and could be replayed across nodes.
- **Recommendation:** Add a `context: &str` parameter to `sign()` and `verify()` that includes the signer's Rep C 54-trit address. Construct a domain-separated message: `domain_sep = b"NinjaExec-TL-DSA:" || rep_c_address || b":" || payload`, and pass that as the message to `tl_dsa::sign`. Do the same in `verify`. Update all call sites (server.rs `handle_sign`, main.rs `Command::SignFile`) to supply the Rep C address. Alternatively, use TL-DSA's context-string parameter if the `tl_dsa` API supports one.
- **Verification:** Grep for all `tl_dsa::sign` and `tl_dsa::verify` call sites and confirm every one includes a Rep C address in the context/message. Unit tests must assert that a signature produced with one Rep C address fails verification with a different Rep C address.

### R1-A1-2 (MINOR)
- **Section:** signing_engine.rs, line 27 (fingerprint function)
- **Severity:** MINOR
- **Finding:** The `fingerprint()` function uses `ternary_math::sponge::derive_key` with domain separator `b"NinjaExec-FP"` and a 16-byte output. This is acceptable for display purposes. However, no Rep C address is bound into the fingerprint context.
- **Recommendation:** Consider binding the Rep C address into the fingerprint domain separator for consistency with INVARIANT 9, though this is non-security-critical since the fingerprint is for human display only.
- **Verification:** Confirm fingerprint is never used as a security-critical identity binding.

### R1-A1-3 (IMPORTANT)
- **Section:** keystore.rs, lines 67-83 (derive_enc_key — KDF)
- **Severity:** IMPORTANT
- **Finding:** The KDF uses `KDF_ITERATIONS = 4096` rounds of TLSponge-385 `derive_key`. While TLSponge is computationally heavier than SHA-256 per round, 4096 iterations is low for a passphrase-based KDF. Modern passphrase KDFs (Argon2id, scrypt) target ≥100ms wall-clock time. At 4096 rounds, the KDF may complete in under 10ms, making offline brute-force attacks against the keystore significantly faster. Additionally, the KDF does not include a memory-hard component, making GPU-based attacks feasible.
- **Recommendation:** Increase `KDF_ITERATIONS` to at least 100,000 or add a configurable cost parameter that targets ≥100ms on the deployment hardware. Alternatively, integrate Argon2id as the outer KDF and use TLSponge for inner domain separation only. Document the chosen cost target and rationale. Since the iteration count is stored in the keystore header, existing keystores can be migrated by re-encrypting on next unlock.
- **Verification:** Benchmark `derive_enc_key` with the new iteration count on target hardware (x86_64, aarch64) and confirm ≥100ms wall-clock time. Verify that the iteration count stored in the keystore header is read and honored on open.

### R1-A1-4 (CRITICAL)
- **Section:** server.rs, lines 614-631 (build_router — CORS policy)
- **Severity:** CRITICAL
- **Finding:** The CORS layer is configured with `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. This means any website in the user's browser can send cross-origin requests to the localhost signing agent. A malicious or compromised web page could issue signing requests to `http://127.0.0.1:21027/sign` and, if the keystore is unlocked and running in headless mode, obtain valid TL-DSA signatures over attacker-controlled payloads without any user interaction.
- **Recommendation:** Replace `allow_origin(Any)` with an explicit allowlist of trusted origins. At minimum, restrict to the specific YODA dashboard origin(s) (e.g., `https://yoda.replit.app`). The allowlist should be configurable in `ninja-exec.json`. If no allowlist is configured, default to denying all cross-origin requests. Additionally, consider requiring an API key or bearer token for the `/sign` and `/unlock` endpoints (the `confirm_token` mechanism exists but is only enforced on `/confirm/*` endpoints).
- **Verification:** Attempt a cross-origin `fetch("http://127.0.0.1:21027/sign", ...)` from a page hosted on an untrusted origin and confirm it is rejected with a CORS error. Verify the allowlist is configurable and defaults to deny.

### R1-A1-5 (IMPORTANT)
- **Section:** server.rs, lines 463-525 (handle_unlock) and main.rs, lines 167, 305, 334, 427 (PLENUM_PASSPHRASE env var)
- **Severity:** IMPORTANT
- **Finding:** The passphrase can be supplied via the `PLENUM_PASSPHRASE` environment variable. On Linux, any process running as the same user can read `/proc/<pid>/environ` to extract this value. On Windows, environment variables are accessible to any process in the same session. The `handle_unlock` endpoint accepts the passphrase in a JSON POST body over plain HTTP (no TLS). While the server binds to 127.0.0.1, local processes can sniff loopback traffic on some OS configurations. The environment variable is never zeroed after use.
- **Recommendation:** (1) After reading `PLENUM_PASSPHRASE`, overwrite the environment variable with zeros using `std::env::set_var("PLENUM_PASSPHRASE", "")` (though this is imperfect since the OS may retain the original). Document that `PLENUM_PASSPHRASE` is a convenience mechanism for CI/automation only and must not be used in interactive security-sensitive environments. (2) For the HTTP unlock endpoint, consider requiring the `confirm_token` as an Authorization header, adding a second factor of authentication. (3) On Linux, document that `/proc/sys/kernel/yama/ptrace_scope` should be ≥1 and core dumps should be disabled. (4) Document macOS Keychain integration as a future enhancement.
- **Verification:** Confirm the passphrase environment variable is documented as CI-only. Confirm that `handle_unlock` requires the `confirm_token` header (or document the risk acceptance). Confirm platform-specific hardening guidance is included in deployment docs.

### R1-A1-6 (IMPORTANT)
- **Section:** server.rs, lines 533-548 (check_confirm_token — token comparison)
- **Severity:** IMPORTANT
- **Finding:** The `confirm_token` comparison uses `!=` (standard string comparison), which is not constant-time. This leaks the token length and content via timing side-channel. While the attack surface is localhost-only, a local attacker process could exploit timing differences to recover the confirm token byte-by-byte.
- **Recommendation:** Replace the `!=` comparison with a constant-time comparison. Use the same XOR-accumulate pattern already used in `keystore.rs` line 130-133 for tag comparison, or use a dedicated constant-time comparison crate (e.g., `subtle::ConstantTimeEq`). Ensure both operands are compared over their full length.
- **Verification:** Review the replacement code and confirm it uses constant-time comparison. Verify that timing measurements over 10,000 requests show no statistically significant difference between correct prefix and wrong prefix tokens.

### R1-A1-7 (IMPORTANT)
- **Section:** confirm.rs, lines 159-173 (evaluate_confirmation — headless mode)
- **Severity:** IMPORTANT
- **Finding:** When `headless = true`, ALL signing requests are auto-approved regardless of context, including destructive operations like `exec`, `model-swap`, `file-push`, `deploy`, `config-update`, `key-rotation`. Combined with the open CORS policy (Finding 4), any web page in the browser can silently obtain signatures for arbitrary payloads when the agent runs in headless mode.
- **Recommendation:** Even in headless mode, certain high-risk contexts (`exec`, `deploy`, `key-rotation`) should require explicit confirmation or at minimum require the `confirm_token` header on the `/sign` endpoint. Add a `headless_allow` configuration list that defaults to a safe subset (e.g., `sign`, `verify`, `pubkey`, `status`). Headless mode should still reject contexts not in the `headless_allow` list.
- **Verification:** Start the agent in headless mode. Issue a `/sign` request with context `exec: rm -rf /` and confirm it is rejected unless the context is in the `headless_allow` list.

### R1-A1-8 (IMPORTANT)
- **Section:** keystore.rs, lines 86-118 (encrypt_sk — authenticated encryption construction)
- **Severity:** IMPORTANT
- **Finding:** The keystore uses a custom Encrypt-then-MAC construction built from TLSponge-385 `derive_key`. While the construction follows the correct pattern (encrypt with XOR keystream, then MAC over ciphertext), it is a bespoke authenticated encryption scheme that has not been formally analyzed. The 12-byte nonce is generated per-encryption and the 16-byte tag provides 128-bit integrity — these are reasonable. However, INVARIANT 7 and the security review scope require that all authenticated encryption use PlenumNET primitives (T-AE-MAC). The current construction is a custom scheme that does not use T-AE-MAC.
- **Recommendation:** Replace the bespoke encrypt/MAC with T-AE-MAC (the PlenumNET authenticated encryption primitive) if it is available in `ternary_math`. If T-AE-MAC is not yet exposed with a suitable API for binary data at rest, document this as a known deviation with a risk acceptance and a migration path. The current construction is functionally reasonable but should be replaced with the canonical primitive.
- **Verification:** Confirm that `ternary_math` exposes a T-AE-MAC API. If so, replace the bespoke construction and verify round-trip encrypt/decrypt still works with existing unit tests. If not, verify the risk acceptance is documented.

### R1-A1-9 (IMPORTANT)
- **Section:** plenum-app.toml, line 10 (upgrade_code)
- **Severity:** IMPORTANT
- **Finding:** The `upgrade_code` is a hardcoded UUID `A1B2C3D4-E5F6-7890-ABCD-EF1234567890` that appears to be a placeholder, not a deterministically derived product code. The security review scope requires that product code derivation is deterministic and collision-resistant with a minimum collision probability bound of 2^-64. The current value is clearly a hand-typed placeholder and does not demonstrate derivation from inputs.
- **Recommendation:** Derive the upgrade code deterministically from the app name and publisher using the TIS-27 or TLSponge-385 hash of `"NinjaExec:Capomastro Holdings Ltd."`, then format as a UUID v5-style identifier. Document the derivation formula and collision resistance bound. The upgrade code must remain permanent once assigned — derive once and store.
- **Verification:** Reproduce the derivation on a clean machine and confirm byte-identical output. Verify the collision bound is documented as ≥2^-64.

### R1-A1-10 (IMPORTANT)
- **Section:** main.rs, line 193 (confirm token printed to stdout)
- **Severity:** IMPORTANT
- **Finding:** During `ninja-exec init`, the confirm token is printed to stdout: `println!("[NinjaExec] Token: {}", token)`. This token protects the confirmation API and should be treated as a secret. Printing it to stdout risks exposure in shell history, log files, CI output, and screen captures. The token is also stored in `ninja-exec.json` on disk with no explicit permission check.
- **Recommendation:** Do not print the confirm token to stdout. Instead, print only a message stating where the token is stored (`ninja-exec.json`) and advise the user to read it from there. Ensure `ninja-exec.json` has restrictive file permissions (0600 on Unix, ACL on Windows) similar to the keystore file.
- **Verification:** Run `ninja-exec init` and confirm the token is not displayed. Confirm `ninja-exec.json` has 0600 permissions on Unix.

### R1-A1-11 (MINOR)
- **Section:** keystore.rs, lines 61-65 (zeroize function)
- **Severity:** MINOR
- **Finding:** The `zeroize()` function uses `write_volatile` to zero memory, which is a reasonable approach but relies on unsafe code. The Rust `zeroize` crate provides a safer, audited implementation that also handles compiler optimizations and register spilling. The current implementation does not call `compiler_fence` after zeroing, which means the compiler could theoretically reorder operations.
- **Recommendation:** Consider using the `zeroize` crate (which is widely audited) instead of the custom implementation. If keeping the custom implementation, add `std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst)` after the zeroing loop.
- **Verification:** Check that `compiler_fence` is present after every `zeroize()` call, or that the `zeroize` crate is integrated.

### R1-A1-12 (MINOR)
- **Section:** server.rs, lines 37-59 (RateLimiter) and line 11 (shared state)
- **Severity:** MINOR
- **Finding:** The rate limiter uses a single shared bucket for all endpoints. The `/sign` and `/unlock` endpoints share the same 30 requests/minute limit. An attacker could exhaust the rate limit with `/sign` requests, causing legitimate `/unlock` attempts to be rate-limited (denial of service against the operator). The rate limiter stores timestamps in an unbounded `Vec` — a sustained burst could cause memory growth until GC.
- **Recommendation:** Use separate rate limiters for `/sign` and `/unlock`. The `/unlock` endpoint should have a much lower rate limit (e.g., 5 attempts/minute) to slow brute-force passphrase attacks. Bound the `timestamps` vector to `max_per_minute` entries.
- **Verification:** Confirm separate rate limiters exist for sign and unlock. Send 30 sign requests, then attempt unlock and confirm it is not rate-limited.

### R1-A1-13 (MINOR)
- **Section:** audit.rs, lines 38-50 (AuditLog::append)
- **Severity:** MINOR
- **Finding:** Audit log writes silently swallow errors (`let _ = writeln!(...)`). If the audit log file cannot be written (disk full, permissions), operations continue without any audit trail. The audit log file has no explicit permission restriction.
- **Recommendation:** Return a `Result` from `append` and propagate audit write failures to the caller. At minimum, log audit write failures to stderr. Set file permissions on the audit log to 0600 (Unix) or restricted ACL (Windows) on creation.
- **Verification:** Confirm that audit write failures are surfaced. Confirm file permissions on the audit log.

### R1-A1-14 (MINOR)
- **Section:** config.rs, lines 58-68 (save_default) and lines 70-94 (generate_confirm_token)
- **Severity:** MINOR
- **Finding:** The `ninja-exec.json` config file is written without explicit file permission restrictions. On Unix, the default umask may result in world-readable permissions. This file contains the `confirm_token` secret.
- **Recommendation:** Set 0600 permissions on `ninja-exec.json` immediately after creation, similar to the keystore file.
- **Verification:** Create a new config file and verify its permissions are 0600 on Unix.

### R1-A1-15 (CRITICAL)
- **Section:** signing_engine.rs (entire module); INVARIANT 9 cross-reference
- **Severity:** CRITICAL
- **Finding:** No Rep C address is defined, stored, or used anywhere in the NinjaExec codebase. INVARIANT 9 requires that all cryptographic operations binding node identity use Rep C (54-trit, binary-encoded) addressing exclusively. NinjaExec does not associate the signing key with a Rep C address, does not include a Rep C address in TL-DSA signing context, does not include a Rep C address in TLSponge-385 KDF domain separation for the keystore, and does not include a Rep C address in fingerprint computation. Without Rep C binding, signatures produced by NinjaExec cannot be verified against a registered TDNS address.
- **Recommendation:** (1) During `ninja-exec init`, derive or accept a Rep C 54-trit TDNS address for this node. Store the Rep C address in the keystore file or a companion identity file. (2) Bind the Rep C address into all TL-DSA signing contexts (see Finding 1). (3) Bind the Rep C address into the KDF domain separator. (4) Bind the Rep C address into the fingerprint domain separator. (5) Include the Rep C address in the operator export JSON.
- **Verification:** Grep for `rep_c` or equivalent in all source files and confirm it appears in every cryptographic context string. Verify the Rep C address is stored persistently and included in operator export output.

### Agent 1 — Cryptographic Claims Verification

| Claim | Location | Verdict | Notes |
|---|---|---|---|
| TL-DSA-87 used for all signatures | signing_engine.rs line 7 | **VERIFIED** | `TlDsaVariant::TlDsa87` is hardcoded |
| No Ed25519 / no `crypto.sign` | Full codebase | **VERIFIED** | No Ed25519, no Node.js crypto calls found |
| TLSponge-385 used for KDF | keystore.rs line 71 | **VERIFIED** | `ternary_math::sponge::derive_key` with domain separator |
| TLSponge-385 used for audit hash | audit.rs line 60 | **VERIFIED** | `ternary_math::sponge::derive_key` with domain `b"NinjaExec-AUDIT-HASH"` |
| TLSponge-385 used for fingerprint | signing_engine.rs line 27 | **VERIFIED** | `ternary_math::sponge::derive_key` with domain `b"NinjaExec-FP"` |
| Keystore AE uses PlenumNET primitives only | keystore.rs lines 86-153 | **VERIFIED** | Custom Encrypt-then-MAC using TLSponge; no AES-256-GCM |
| Rep C bound in signing context (INV 9) | signing_engine.rs lines 13-18 | **INCORRECT** | No Rep C address in any signing context |
| Rep C bound in KDF domain (INV 9) | keystore.rs line 71 | **INCORRECT** | KDF domain is `b"NinjaExec-KDF-v2"` with no Rep C |
| Constant-time tag comparison | keystore.rs lines 130-133 | **VERIFIED** | XOR-accumulate pattern |
| Constant-time confirm_token comparison | server.rs line 540 | **INCORRECT** | Standard `!=` string comparison |
| No raw binary integers in sponge (INV 8) | keystore.rs, audit.rs, signing_engine.rs | **UNVERIFIED** | Depends on `ternary_math::sponge` internals |
| Upgrade code deterministically derived | plenum-app.toml line 10 | **INCORRECT** | Hardcoded placeholder UUID |

### Agent 1 — Passphrase Entropy Assessment

**Value:** 72 bits minimum

**Rationale:** NinjaExec enforces a 12-character minimum passphrase length. Assuming a realistic passphrase drawn from printable ASCII (95 characters), 12 characters yield `12 × log₂(95) ≈ 78.8 bits` of entropy in the best case. However, human-chosen passphrases are significantly weaker — NIST SP 800-63B estimates ~1-2 bits per character for user-chosen passwords beyond the first 8 characters. A 12-character user-chosen passphrase may have as few as ~30 bits of effective entropy. The minimum acceptable entropy for protecting a long-lived signing key should be 72 bits (comparable to a 128-bit security target with a 2^56 KDF work factor). The current KDF iteration count of 4096 provides only ~12 bits of work factor, meaning the effective security is `passphrase_entropy + 12 bits`. To reach 72 bits effective with 12 bits of KDF work, the passphrase itself must provide ~60 bits, which requires either a longer minimum (e.g., 16 characters of mixed case + digits + symbols) or a passphrase generation recommendation (e.g., 5-word Diceware). Alternatively, increasing KDF iterations to ~2^20 (1M) would provide ~20 bits of work factor, requiring ~52 bits of passphrase entropy (12 mixed-case characters).

### Agent 1 — Summary Verdict: **FAIL**

Three CRITICAL findings block implementation:

1. **R1-A1-1 / R1-A1-15:** TL-DSA signing context does not bind the signer's Rep C address, violating INVARIANT 9. No Rep C address exists anywhere in the codebase. Signatures produced by NinjaExec cannot be cryptographically bound to a TDNS identity, which is a fundamental requirement of the PlenumNET security model.

2. **R1-A1-4:** The CORS policy allows any origin to access the localhost signing API. Combined with headless mode auto-approval, any website in the operator's browser can obtain TL-DSA signatures over attacker-controlled payloads. This is a remotely exploitable signature oracle.

Six IMPORTANT findings require resolution before first release: weak KDF iteration count (R1-A1-3), environment variable passphrase exposure (R1-A1-5), non-constant-time token comparison (R1-A1-6), headless mode auto-approves all contexts including destructive operations (R1-A1-7), bespoke authenticated encryption instead of T-AE-MAC (R1-A1-8), placeholder upgrade code (R1-A1-9), and confirm token printed to stdout (R1-A1-10).

---

## Agent 2: DevOps Automator — Full Findings

### R1-A2-1 (CRITICAL)
- **Section:** `ninja-exec/src/audit.rs` lines 38–51, `server.rs` passim
- **Severity:** CRITICAL
- **Finding:** The `AuditLog::append()` method silently swallows every possible failure: JSON serialization errors (`if let Ok(json)`), directory creation errors (`let _ = fs::create_dir_all`), file open errors (`if let Ok(mut file)`), and write errors (`let _ = writeln!`). In `server.rs`, every audit call is additionally wrapped in `if let Ok(log) = state.audit_log.lock()` — a poisoned Mutex also produces no error. A signing operation can succeed and return a valid signature to the caller while the audit trail silently fails to record it. This is the canonical "silent failure producing an unsigned/untested artifact" — in this case, an unaudited signature.
- **Recommendation:** `append()` must return `Result<(), AuditError>`. All callers in `server.rs` must propagate this error and block the signing response if the audit write fails. At minimum, a failed audit write for a `sign` operation must prevent the signature from being returned.
- **Verification:** Introduce a test that makes the audit log directory read-only, issues a sign request, and confirms the request is rejected (not silently signed). Verify that no code path returns a signature without a confirmed audit write.

### R1-A2-2 (IMPORTANT)
- **Section:** `ninja-exec/Cargo.toml` lines 22–31
- **Severity:** IMPORTANT
- **Finding:** Several dependencies are not pinned to a specific patch version: `tokio = "1"`, `serde = "1"`, `serde_json = "1"`, `getrandom = "0.2"`, `chrono = "0.4"`. The DevOps review protocol requires build tool dependencies to be version-pinned at the patch level for reproducible builds. While `Cargo.lock` provides reproducibility in practice, unpinned major/minor ranges in `Cargo.toml` allow silent dependency drift when the lockfile is regenerated.
- **Recommendation:** Pin all dependencies to patch versions: `tokio = "1.36.0"`, `serde = "1.0.197"`, `serde_json = "1.0.114"`, `getrandom = "0.2.12"`, `chrono = "0.4.35"` (or current locked versions). This ensures `cargo update` does not silently change behavior.
- **Verification:** Run `cargo tree -p ninja-exec` and confirm every direct dependency matches the pinned version in `Cargo.toml`. Diff `Cargo.lock` before and after `cargo update` — there should be zero changes to direct dependencies.

### R1-A2-3 (CRITICAL)
- **Section:** `ninja-exec/src/audit.rs` AuditEntry struct, `server.rs` audit calls
- **Severity:** CRITICAL
- **Finding:** INVARIANT 9 requires all audit records and provenance entries to reference nodes exclusively by their Rep C address. The `AuditEntry` struct identifies the signing node by HTTP `origin` header (a URL like `http://yoda.replit.app`), by hostname (in `export-operator`), or by nothing at all (`origin: None` for lock/unlock/startup). No field in `AuditEntry` contains a Rep C address. Log correlation across sources cannot use Rep C as the join key because the field does not exist.
- **Recommendation:** Add a `node_repc: String` field to `AuditEntry` that is populated from the operator's registered Rep C address (stored in or alongside the keystore). Every audit entry must include this field. Remove hostname-based identification from `export-operator` output or supplement it with the Rep C address as the primary identifier.
- **Verification:** Grep the `AuditEntry` struct for a `node_repc` or `rep_c` field. Verify every `AuditEntry` construction site populates it. Verify `export-operator` JSON output includes a `rep_c_address` field.

### R1-A2-4 (CRITICAL)
- **Section:** `ninja-exec/src/signing_engine.rs` lines 13–15
- **Severity:** CRITICAL
- **Finding:** INVARIANT 7 states: "The signer's Rep C address must be bound into the signature context string." The `sign()` function calls `tl_dsa::sign(secret_key, payload, VARIANT)` with no context string and no Rep C address binding. The signature is computed over the raw payload only. A signature produced by NinjaExec cannot be cryptographically bound to a specific operator identity as required by the framework.
- **Recommendation:** Modify `sign()` to accept a `context: &str` parameter that includes the signer's Rep C address. Construct a domain-separated signing input: `b"NinjaExec-SIGN:" || rep_c_address || b":" || context || b":" || payload`. Pass this composite message to `tl_dsa::sign()`. Update all callers accordingly.
- **Verification:** Read `signing_engine::sign()` and confirm the Rep C address is concatenated into the signed message. Write a test that signs with context, then verifies with the same context (pass) and a different context (fail).

### R1-A2-5 (IMPORTANT)
- **Section:** `ninja-exec/src/server.rs` lines 615–618
- **Severity:** IMPORTANT
- **Finding:** The CORS layer is configured with `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. While the server binds to 127.0.0.1 only, any web page loaded in the operator's browser can issue cross-origin requests to `http://127.0.0.1:21027/sign` and obtain valid TL-DSA signatures. This is a browser-based confused deputy attack vector. Cross-reference to Security Engineer (Agent 1) for severity assessment.
- **Recommendation:** Restrict CORS to explicitly allowed origins. At minimum, allow only `http://127.0.0.1:*` and `http://localhost:*`. For YODA dashboard integration, add the specific YODA origin. Reject all other origins.
- **Verification:** Start the agent, issue a `curl` request with `Origin: https://evil.com` and confirm it receives a CORS rejection (no `Access-Control-Allow-Origin` header matching the evil origin).

### R1-A2-6 (IMPORTANT)
- **Section:** `ninja-exec/src/confirm.rs` lines 159–173, `main.rs` lines 448–458
- **Severity:** IMPORTANT
- **Finding:** Headless mode (`--headless` flag) auto-approves ALL signing requests including destructive operations (`exec`, `model-swap`, `file-push`, `deploy`, `key-rotation`). Combined with Finding R1-A2-5 (open CORS), a browser tab on the operator's machine can silently trigger arbitrary signing operations with no confirmation gate. The audit log records this as `"confirmation": "auto"` — indistinguishable from a legitimately auto-approved read-only operation. Cross-reference to Security Engineer (Agent 1).
- **Recommendation:** In headless mode, either (a) restrict auto-approval to the `auto_approve` list only (verify, pubkey, status, tail, file-pull), requiring the confirm token for all other operations, or (b) require a separate `--headless-allow-destructive` flag with a documented risk acceptance. Distinguish headless auto-approval from configured auto-approval in audit entries.
- **Verification:** Start the agent in headless mode, send a sign request with context `exec: rm -rf /`, and confirm it is either rejected or requires the confirm token (not auto-approved).

### R1-A2-7 (IMPORTANT)
- **Section:** No CI/CD pipeline file found for ninja-exec
- **Severity:** IMPORTANT
- **Finding:** No CI/CD pipeline definition (GitHub Actions workflow, Makefile, or build script) exists for ninja-exec. The review protocol requires verification that: (a) every pipeline step has defined failure handling, (b) all architectures are treated as an atomic release, (c) automated verification steps use exit codes, (d) expected CI duration and parallelism strategy are specified. None of these can be verified because no pipeline exists. The `plenum-app.toml` defines `architecture = ["aarch64", "x86_64"]` but no cross-compilation or matrix build is configured.
- **Recommendation:** Create a GitHub Actions workflow (`.github/workflows/ninja-exec-ci.yml`) that: (a) builds for both `aarch64` and `x86_64` targets, (b) runs `cargo test -p ninja-exec`, (c) runs `cargo clippy -p ninja-exec`, (d) fails the entire release if any architecture fails, (e) documents expected CI duration and parallelism strategy.
- **Verification:** Confirm `.github/workflows/ninja-exec-ci.yml` exists and contains a matrix build for both architectures. Run the workflow and confirm it produces exit code 0 on success, non-zero on any failure.

### R1-A2-8 (IMPORTANT)
- **Section:** `ninja-exec/src/config.rs` lines 70–94, `main.rs` lines 190–193
- **Severity:** IMPORTANT
- **Finding:** During `init`, the confirm token is printed to stdout in cleartext: `println!("[NinjaExec] Token: {}", token)`. This token is the sole authentication mechanism for the `/confirm/decide` endpoint. If stdout is captured in logs (systemd journal, Windows Event Log, CI output), the token is exposed. Additionally, `generate_confirm_token()` writes the token to `ninja-exec.json` with default file permissions (no `chmod 600` on Unix). Cross-reference to Security Engineer (Agent 1) for credential exposure assessment.
- **Recommendation:** Do not print the confirm token to stdout. Instead, write it to a separate file (`confirm-token.txt`) with mode `0600` and instruct the operator to read it from there. Apply `chmod 600` to `ninja-exec.json` on Unix, similar to the keystore file.
- **Verification:** Run `ninja-exec init` and confirm the token does not appear in stdout/stderr. Verify `ninja-exec.json` file permissions are `0600` on Unix.

### R1-A2-9 (MINOR)
- **Section:** `ninja-exec/plenum-app.toml` line 7
- **Severity:** MINOR
- **Finding:** `binary = "ninja-exec.exe"` is Windows-specific, but the `architecture` field includes both `aarch64` and `x86_64` which could target Linux/macOS. No conditional binary naming or platform-specific configuration exists for non-Windows targets.
- **Recommendation:** Add a `[platforms]` or `[install.windows]`/`[install.linux]` section to `plenum-app.toml` that specifies the correct binary name per platform.
- **Verification:** Inspect `plenum-app.toml` for platform-specific binary naming. Confirm the installer framework reads the correct binary name for each target OS.

### R1-A2-10 (MINOR)
- **Section:** `ninja-exec/src/signing_engine.rs` line 27, `audit.rs` line 60, `keystore.rs` line 71
- **Severity:** MINOR
- **Finding:** Payload hashing in audit and fingerprint generation both use `ternary_math::sponge::derive_key()` which is the TLSponge-385 kernel sponge. This is correct per framework conventions (not SHA-256/BLAKE3). The hash output is labeled `tis27:` in `audit.rs` line 62, but the actual primitive used is `sponge::derive_key` which is TLSponge-385, not TIS-27. The label is misleading and could cause operators to apply the wrong verification procedure.
- **Recommendation:** Change the hash prefix from `tis27:` to `tlsponge385:` or `sponge:` to accurately reflect the primitive used, unless `sponge::derive_key` internally delegates to TIS-27 (in which case, document this).
- **Verification:** Trace `ternary_math::sponge::derive_key` to confirm which sponge variant it uses. Verify the audit hash prefix matches the actual primitive.

### R1-A2-11 (MINOR)
- **Section:** `ninja-exec/src/main.rs` lines 305, 334, 427
- **Severity:** MINOR
- **Finding:** The passphrase is accepted via the `PLENUM_PASSPHRASE` environment variable for automation. Environment variables are visible in `/proc/<pid>/environ` on Linux and via `Get-Process` on Windows. While documented, this creates a silent credential exposure in CI/CD pipelines and process listings.
- **Recommendation:** Document that `PLENUM_PASSPHRASE` should only be used in ephemeral CI environments. Consider supporting passphrase input via a file descriptor (e.g., `--passphrase-fd 3`) or a named pipe as a more secure alternative.
- **Verification:** Confirm documentation warns against using `PLENUM_PASSPHRASE` in persistent/production environments.

### R1-A2-12 (IMPORTANT)
- **Section:** `ninja-exec/src/server.rs`, `main.rs`
- **Severity:** IMPORTANT
- **Finding:** No deployment test specification exists. The review protocol requires: (a) every test step to be automatable with machine-verifiable exit codes, (b) minimum supported OS versions to be defined, (c) expected CI duration for full-matrix testing to be specified, (d) parallelism strategy to be documented, (e) product-specific validation requiring network services to have a mock mode. None of these are present.
- **Recommendation:** Create an integration test module (`tests/integration.rs` or `ninja-exec/tests/`) that starts the server, exercises all endpoints, and verifies exit codes. Document minimum OS versions in `plenum-app.toml`. Specify expected CI duration.
- **Verification:** Confirm an integration test file exists and can be run with `cargo test -p ninja-exec --test integration`.

### Agent 2 — Summary Verdict: **FAIL**

Three CRITICAL findings block implementation:

1. **R1-A2-1 (Audit silent failure):** A signing operation can return a valid signature while the audit trail silently fails to record it.
2. **R1-A2-3 (No Rep C in audit entries):** INVARIANT 9 is violated — audit records identify nodes by HTTP origin URLs and hostnames instead of Rep C addresses.
3. **R1-A2-4 (No Rep C context binding in signatures):** INVARIANT 7 is violated — TL-DSA signatures are computed over raw payloads without binding the signer's Rep C address.

Additionally, five IMPORTANT findings require resolution before first product release. The open CORS policy combined with headless auto-approval creates a browser-based confused deputy attack that could silently produce unauthorized signatures.

---

## Agent 3: PlenumNET Integration Specialist — Full Findings

### R1-A3-1 (CRITICAL)
- **Section:** `signing_engine.rs`, lines 13–18 (`sign` / `verify` functions)
- **Severity:** CRITICAL
- **Finding:** TL-DSA signing and verification do not bind the signer's Rep C address into the signature context string. INVARIANT 9 requires: "The signer's Rep C address must be bound into the signature context string. Signature verification must check the signer's public key against a registered Rep C address." The current `sign()` function passes `(secret_key, payload, VARIANT)` directly to `tl_dsa::sign` with no context string and no Rep C identity binding. The `verify()` function similarly performs raw verification with no address check.
- **Recommendation:** Extend the signing API to require a Rep C address (54-trit, binary-encoded) as a mandatory parameter. Construct a domain-separated context: `"NinjaExec-SIGN-v1.0" ‖ rep_c_address ‖ payload` and pass this composite message to `tl_dsa::sign`. On verification, require the claimed signer's Rep C address, reconstruct the same composite, and verify against a registered key.
- **Verification:** Confirm that `signing_engine::sign()` and `signing_engine::verify()` accept a Rep C address parameter, that the address is concatenated into the signed message before calling `tl_dsa::sign`/`tl_dsa::verify`, and that the server rejects requests where the signer address does not match the keystore's registered identity.

### R1-A3-2 (IMPORTANT)
- **Section:** `signing_engine.rs`, line 27 (`fingerprint` function)
- **Severity:** IMPORTANT
- **Finding:** The fingerprint derivation uses the context string `"NinjaExec-FP"` with `ternary_math::sponge::derive_key`. This context string is not present in the canonical context string registry. All context strings used in TIS-27 / TLSponge-385 derivation are load-bearing — a wrong or unregistered context string produces a wrong key. Status: **UNVERIFIED**.
- **Recommendation:** Register `"NinjaExec-FP"` in the canonical context string registry with its purpose documented (public key fingerprint derivation, 16-byte output, used for operator display only — not security-critical). Until registered, mark this context string as provisional.
- **Verification:** Confirm the context string appears in the canonical registry with matching parameters.

### R1-A3-3 (IMPORTANT)
- **Section:** `keystore.rs`, lines 67–84 (`derive_enc_key` function)
- **Severity:** IMPORTANT
- **Finding:** The KDF uses context string `"NinjaExec-KDF-v2"` with `ternary_math::sponge::derive_key`. This context string is not in the canonical registry. Status: **UNVERIFIED**. Additionally, the iterated KDF construction (4096 rounds of sponge derivation) is a custom construction that has not been formally analyzed.
- **Recommendation:** (1) Register `"NinjaExec-KDF-v2"` in the canonical context string registry. (2) Document the iterated KDF construction in a brief security note. (3) Cross-reference to Security Engineer (Agent 1) for formal assessment.
- **Verification:** Confirm context string registration. Confirm Agent 1 has reviewed the iterated KDF construction.

### R1-A3-4 (IMPORTANT)
- **Section:** `keystore.rs`, lines 95–118 (`encrypt_sk` function)
- **Severity:** IMPORTANT
- **Finding:** Two additional unregistered context strings: `"NinjaExec-KS-STREAM"` (keystream derivation) and `"NinjaExec-KS-TAG"` (authentication tag derivation). Both are used with `ternary_math::sponge::derive_key`. Status: **UNVERIFIED**. The encrypt-then-MAC construction is a custom authenticated encryption scheme rather than using T-AE-MAC. Per Critical Rules: "AES-256-GCM must be replaced with Phase Encryption (data at rest) or TLSponge T-AE-MAC (authenticated encryption)." While this is not AES-256-GCM, it is a custom construction that should use T-AE-MAC instead.
- **Recommendation:** (1) Register both context strings. (2) Evaluate replacing the custom construction with T-AE-MAC. (3) If the custom construction is retained, document the security proof sketch and obtain Agent 1 sign-off.
- **Verification:** Confirm context strings are registered. Confirm either T-AE-MAC is used or the custom construction has been formally reviewed.

### R1-A3-5 (MINOR)
- **Section:** `audit.rs`, line 60 (`hash_payload` function)
- **Severity:** MINOR
- **Finding:** The audit hash uses context string `"NinjaExec-AUDIT-HASH"` with `ternary_math::sponge::derive_key`. Status: **UNVERIFIED** (not in canonical registry). The prefix `"tis27:"` is misleading — the actual primitive is TLSponge-385, not TIS-27.
- **Recommendation:** (1) Register `"NinjaExec-AUDIT-HASH"` in the canonical registry. (2) Correct the prefix from `"tis27:"` to `"sponge385:"`.
- **Verification:** Confirm the hash prefix accurately reflects the underlying primitive.

### R1-A3-6 (CRITICAL)
- **Section:** `keystore.rs` / `signing_engine.rs` / `server.rs` — entire codebase
- **Severity:** CRITICAL
- **Finding:** No Rep C address is stored, derived, or used anywhere in NinjaExec. INVARIANT 9 requires: "All cryptographic operations that bind node identity or address must use Rep C (54-trit, binary-encoded) addressing exclusively." The keystore stores only a raw TL-DSA keypair with no associated Rep C address. The `export-operator` command exports a hostname-based name (`operator@{hostname}`) which explicitly violates INVARIANT 9: "No cryptographic operation may use hostname, IP address, Windows SID, or any non-Rep-C identifier as an identity binding."
- **Recommendation:** (1) During `ninja-exec init`, derive or accept a Rep C address and store it alongside the keypair. (2) Bind the Rep C address into all signing contexts. (3) Replace the hostname-based `operator@{hostname}` identifier in `export-operator` with the Rep C address as the primary identifier. (4) Add Rep C address to `/pubkey` and `/status` API responses.
- **Verification:** Confirm that `Keystore` stores and exposes a Rep C address. Confirm `export-operator` output includes Rep C address as the primary identifier.

### R1-A3-7 (IMPORTANT)
- **Section:** `server.rs`, lines 615–618 (CORS configuration)
- **Severity:** IMPORTANT
- **Finding:** The CORS layer uses `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. Any browser tab on the local machine can issue signing requests to the agent. This is especially critical given that headless mode auto-approves all requests.
- **Recommendation:** Restrict `allow_origin` to a known list of PlenumNET origins. At minimum, do not use `Any` — use a configurable allowlist stored in `ninja-exec.json`.
- **Verification:** Confirm CORS origin is restricted to a configurable allowlist. Confirm the default configuration does not include wildcard origins.

### R1-A3-8 (IMPORTANT)
- **Section:** `server.rs`, lines 147–323 (`handle_sign` function) / `signing_engine.rs`, line 14
- **Severity:** IMPORTANT
- **Finding:** The `sign` function passes the raw payload bytes directly to `tl_dsa::sign`. There is no domain separation between different operation contexts (e.g., `exec` vs `deploy` vs `key-rotation`). While the server validates the `context` field against `VALID_CONTEXTS`, this context is never incorporated into the signed message. An attacker who obtains a valid signature for a `verify` context could potentially replay it as a `deploy` context if the payload happens to match.
- **Recommendation:** Incorporate the operation context into the signed message: `context_bytes ‖ 0x00 ‖ payload`. This ensures signatures are bound to their intended operation type.
- **Verification:** Confirm that `signing_engine::sign()` receives the operation context and incorporates it into the message before signing.

### R1-A3-9 (MINOR)
- **Section:** `config.rs`, line 10 (`DEFAULT_PORT`)
- **Severity:** MINOR
- **Finding:** The default port 21027 does not appear to be derived from any PlenumNET geometric constant. While port selection is not a cryptographic operation, PlenumNET convention suggests deriving operational constants from the ternary framework where possible.
- **Recommendation:** Document the rationale for port 21027. If a geometrically-derived port is desired, consider 21013 (21000 + 13, where 13 = T₇) or similar.
- **Verification:** Confirm port rationale is documented.

### Agent 3 — Cryptographic Claim Verification

| Claim | Location | Status | Notes |
|-------|----------|--------|-------|
| TL-DSA-87 for all signatures | `signing_engine.rs` line 7 | **VERIFIED** | Uses `TlDsaVariant::TlDsa87` |
| No Ed25519 / no external crypto | `Cargo.toml` | **VERIFIED** | Only `ternary-math` for crypto |
| TLSponge for key derivation | `keystore.rs` line 71 | **VERIFIED** | `ternary_math::sponge::derive_key` |
| TLSponge for keystream generation | `keystore.rs` line 98 | **VERIFIED** | `ternary_math::sponge::derive_key` |
| TLSponge for authentication tag | `keystore.rs` line 111 | **VERIFIED** | `ternary_math::sponge::derive_key` |
| TLSponge for audit hashing | `audit.rs` line 60 | **VERIFIED** | `ternary_math::sponge::derive_key` |
| TLSponge for fingerprinting | `signing_engine.rs` line 27 | **VERIFIED** | `ternary_math::sponge::derive_key` |
| Rep C address binding in signatures | All | **INCORRECT** | No Rep C anywhere (INVARIANT 9 violation) |
| Context string `"NinjaExec-FP"` | `signing_engine.rs` line 27 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KDF-v2"` | `keystore.rs` line 71 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KS-STREAM"` | `keystore.rs` line 98 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KS-TAG"` | `keystore.rs` line 111 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-AUDIT-HASH"` | `audit.rs` line 60 | **UNVERIFIED** | Not in canonical registry |
| "Level 5 post-quantum security" | `main.rs` line 153 | **VERIFIED** | TL-DSA-87 = NIST PQ Level 5 |
| No AES-256-GCM | All | **VERIFIED** | Custom sponge-based AE used; T-AE-MAC preferred |
| Constant-time tag comparison | `keystore.rs` lines 130–134 | **VERIFIED** | OR-accumulation pattern |
| Key zeroization on drop | `keystore.rs` lines 316–319 | **VERIFIED** | `Drop` impl calls `lock()` → `zeroize()` with `write_volatile` |

### Agent 3 — Summary Verdict: **FAIL**

NinjaExec correctly selects TL-DSA-87 (VERIFIED), uses the kernel sponge for all hash/MAC/KDF operations with zero external crypto dependencies (VERIFIED), implements proper key zeroization and constant-time tag comparison, and binds exclusively to localhost.

However, two CRITICAL findings block implementation: (1) **INVARIANT 9 is systematically violated**: no Rep C address exists anywhere in the codebase. (2) **TL-DSA signatures lack context binding**: the signer's Rep C address is not bound into the signature context string. Additionally, five context strings are UNVERIFIED, the keystore uses a custom AE construction rather than T-AE-MAC, operation context is not incorporated into signatures, and the CORS policy permits any origin.

---

# PART II — QC-R2 ROUND 1 RESPONSE (ALL AGENT COMMENTS VERBATIM)

## Agent 4 (Evidence Collector) — R1 Response

### C1 — No Rep C Address Binding in TL-DSA Signatures (INVARIANT 9)

**Response: AGREE**

Confirmed by direct inspection of `signing_engine.rs` lines 13–18. The `sign()` function passes `(secret_key, payload, VARIANT)` with no Rep C address in the context. The `verify()` function mirrors this — no address binding on either side. There is no domain-separated message construction of any form. The signature is completely identity-unbound. This is a testable, machine-verifiable gap: a test that signs the same payload from two different keystores will produce signatures that are interchangeable (cross-operator replay), which should never be possible under INVARIANT 9. No existing test asserts cross-operator non-interchangeability. CRITICAL stands.

### C2 — No Rep C Address Exists Anywhere in Codebase (INVARIANT 9)

**Response: AGREE**

Confirmed by full codebase scan. The `Keystore` struct stores only `public_key: Option<Vec<u8>>` and `secret_key: Option<Vec<u8>>` — no address field. The `export-operator` command in `main.rs` lines 252–257 identifies the node as `operator@{hostname}` using `COMPUTERNAME` or `HOSTNAME` environment variables — explicitly prohibited by INVARIANT 9. Audit entries record HTTP `origin` headers (URLs) as identity, not Rep C addresses. The `PendingRequest` struct in `confirm.rs` also uses `origin: Option<String>` (URL-based). There is zero Rep C representation in any data structure, configuration, or file format. CRITICAL stands.

### C3 — CORS Wildcard Origin Creates Signature Oracle

**Response: AGREE**

Confirmed at `server.rs` line 615–618: `CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)`. This is a triple-`Any` CORS policy. Combined with the localhost binding, any website loaded in the operator's browser can send `POST /sign` requests cross-origin. The `NinjaExecConfig` struct has no `allowed_origins` field — no mechanism exists to restrict origins even if desired. In headless mode, this becomes a fully automated signature oracle exploitable from any browser tab. CRITICAL stands.

### C4 — Audit Log Silently Swallows All Write Failures

**Response: AGREE**

Confirmed at `audit.rs` lines 38–51. The `append()` method returns `()` (unit), not `Result`. It uses `let _` to discard errors from `fs::create_dir_all`, `OpenOptions::new().open()`, and `writeln!()`. In `server.rs`, every audit call follows the pattern `if let Ok(log) = state.audit_log.lock() { log.append(&entry); }`. The result is an unaudited cryptographic artifact. A test can verify this by pointing the audit log at a read-only directory and confirming the sign endpoint still returns 200 OK. No such negative test exists. CRITICAL stands.

## Agent 5 (Senior Developer) — R1 Response

### C1 — No Rep C Address Binding in TL-DSA Signatures (INVARIANT 9)

**Response: AGREE**

Confirmed by source inspection. `signing_engine::sign()` at line 13 passes `(secret_key, payload, VARIANT)` directly to `tl_dsa::sign` with no domain-separated context string and no Rep C address binding. The function signature offers no parameter for a Rep C address or context string. Every signature produced by NinjaExec is identity-unbound. The recommended fix — extending `sign()`/`verify()` to accept a Rep C address and constructing a domain-separated message `"NinjaExec-SIGN-v1.0" ‖ rep_c_address ‖ context ‖ payload` — is architecturally sound and implementable. The `context` field already exists in `SignRequest` and flows through the handler; it just needs to be incorporated into the signed message rather than only recorded in the audit log.

### C2 — No Rep C Address Exists Anywhere in Codebase (INVARIANT 9)

**Response: AGREE**

Confirmed. No Rep C address field exists in the keystore, the config, the `AppState`, the `AuditEntry`, or the `export-operator` output. The `export-operator` command at `main.rs` lines 252–264 identifies the operator as `operator@{hostname}` — explicitly prohibited by INVARIANT 9. The resolution requires: (a) deriving a Rep C 54-trit address from the public key during `init` (or accepting one as input), (b) persisting it in the keystore file format (requires a format version bump or an auxiliary file), (c) binding it into all signing contexts, KDF domain separators, audit entries, and the `export-operator` JSON output.

### C3 — CORS Wildcard Origin Creates Signature Oracle

**Response: AGREE**

Confirmed at `server.rs` line 615–618. This is maximally permissive. Any website loaded in the operator's browser can issue cross-origin POST requests to `127.0.0.1:21027/sign`. Combined with headless mode auto-approval, this constitutes a remotely exploitable signature oracle. The fix — a configurable origin allowlist in `ninja-exec.json` with a deny-all default — is straightforward using `tower_http::cors::AllowOrigin::list()`. The `NinjaExecConfig` struct currently has no `allowed_origins` field; one must be added.

### C4 — Audit Log Silently Swallows All Write Failures

**Response: AGREE**

Confirmed at `audit.rs` lines 38–51. The `append()` method uses `if let Ok(...)` chains that silently discard all errors. Furthermore, every call site in `server.rs` wraps audit logging in `if let Ok(log) = state.audit_log.lock()` which silently drops the audit entry if the mutex is poisoned. The fix — making `append()` return `Result` and failing the sign operation if audit write fails — is correct. This is a fail-closed requirement: no signature without a successful audit record.

## Agent 6 (Infrastructure Maintainer) — R1 Response

### C1 — No Rep C Address Binding in TL-DSA Signatures (INVARIANT 9)

**Response: AGREE**

From an operator's perspective, this is a deployment blocker. If signatures are not bound to the operator's Rep C address, there is no way to correlate a signature back to a specific node in the PlenumNET topology. An operator deploying NinjaExec in a multi-node environment cannot distinguish which agent produced which signature. This also makes audit trail correlation across log sources impossible — a direct violation of the operator's ability to investigate incidents.

### C2 — No Rep C Address Exists Anywhere in Codebase (INVARIANT 9)

**Response: AGREE**

Confirmed by inspection: no Rep C address is stored, derived, or referenced in any source file. The `export-operator` command identifies nodes as `operator@{hostname}`. Hostname-based identification is fragile (hostnames change, collide across domains, and are spoofable), and it breaks the zero-sentinel forgery detection property that Rep C provides. The `init` command must derive or accept a Rep C 54-trit address and persist it in the keystore.

### C3 — CORS Wildcard Origin Creates Signature Oracle

**Response: AGREE**

`server.rs` line 616: `allow_origin(Any)` is a deployment-blocking configuration for any machine where a browser is present. In enterprise environments, operators browse the web while NinjaExec runs in the tray. Any malicious or compromised website can issue `POST /sign` requests. Combined with headless mode auto-approval, this is a remotely exploitable signing oracle triggered by a single browser tab. The `plenum-app.toml` first_run sequence should generate the allowlist with a sensible default (e.g., `["http://localhost:*"]`).

### C4 — Audit Log Silently Swallows All Write Failures

**Response: AGREE**

Every error path uses `let _ =` to discard results. From an operator's perspective, this is the worst kind of failure: the system appears healthy, signatures are produced, but the audit trail has gaps. In a compliance environment (FINRA, SOX, etc.), an unaudited signing operation is worse than a failed signing operation. Additionally recommends: on audit write failure, emit a structured error to stderr so that Windows Event Log forwarding or systemd journal captures the failure.

---

# PART III — QC-R2 NEW FINDINGS (ALL AGENT COMMENTS VERBATIM)

## Agent 4: Evidence Collector — New Findings

### R2-A4-1 (CRITICAL)
- **Section:** HTTP API — All 8 Endpoints
- **Severity:** CRITICAL
- **Finding:** No integration test exists for any of the 8 HTTP endpoints (`/sign`, `/verify`, `/pubkey`, `/status`, `/lock`, `/unlock`, `/confirm/pending`, `/confirm/decide`). The `server.rs` file contains zero `#[cfg(test)]` blocks. The `build_router()` function is public and returns a testable `Router`, but no test calls it. All endpoint behavior — including error codes, JSON structure, rate limiting, confirmation flow, and authentication — is completely untested at the HTTP layer.
- **Recommendation:** Add an integration test module in `server.rs` (or a separate `tests/` directory) using `axum::test_helpers` or `tower::ServiceExt` to exercise every endpoint with valid inputs, invalid inputs (bad base64, missing fields, wrong types), locked keystore, rate-limited state, and unauthorized confirm_token.
- **Verification:** `cargo test` must include tests named `test_sign_*`, `test_verify_*`, `test_pubkey_*`, `test_status_*`, `test_lock_*`, `test_unlock_*`, `test_confirm_pending_*`, `test_confirm_decide_*` covering both success and error paths. Each test must assert the exact HTTP status code and the `code` field in the JSON error response.

### R2-A4-2 (IMPORTANT)
- **Section:** `signing_engine.rs` — Test Coverage
- **Severity:** IMPORTANT
- **Finding:** Existing tests cover: roundtrip sign/verify, tampered payload rejection, base64 export, fingerprint determinism, and key sizes. Missing tests: (a) different seeds produce different keypairs, (b) empty payload signing, (c) signature length matches `sig_len()`, (d) cross-keypair verification fails (sign with KP1, verify with KP2's public key), (e) malformed/truncated signature rejection, (f) malformed/wrong-length public key rejection. These are all machine-verifiable negative-path assertions.
- **Recommendation:** Add the six missing test cases listed above.
- **Verification:** `cargo test signing_engine` passes with all new tests asserting specific outcomes (not just `!verify()`).

### R2-A4-3 (IMPORTANT)
- **Section:** `keystore.rs` — Credential Handling Test Coverage
- **Severity:** IMPORTANT
- **Finding:** Tests cover: create-and-open, wrong passphrase rejection, passphrase-too-short, load-public-key-only, KDF header params. Missing tests: (a) empty passphrase returns `EmptyPassphrase` error, (b) already-exists guard (`AlreadyExists` error), (c) corrupted file (truncated blob, wrong magic bytes) returns `InvalidFormat`, (d) unsupported KDF version returns `UnsupportedVersion`, (e) lock() zeroizes secret key (verify `secret_key()` returns `None` after lock), (f) Drop impl zeroizes (harder to test — at minimum verify `is_unlocked()` is false after drop), (g) Unicode passphrase (non-ASCII characters at 12+ chars) roundtrip, (h) keystore file permissions on Unix are 0o600 after create (testable via `std::fs::metadata`).
- **Recommendation:** Add the missing negative-path and edge-case tests.
- **Verification:** Each test asserts the exact `KeystoreError` variant returned.

### R2-A4-4 (IMPORTANT)
- **Section:** `config.rs` — Token Generation and Config Load
- **Severity:** IMPORTANT
- **Finding:** No tests exist for `config.rs`. The following behaviors are untested: (a) `NinjaExecConfig::load()` returns defaults when no file exists, (b) `load()` returns defaults when file contains invalid JSON (silent fallback — itself potentially a bug), (c) `save_default()` does not overwrite an existing config, (d) `generate_confirm_token()` produces a non-empty URL-safe base64 string, (e) `generate_confirm_token()` is idempotent (second call returns same token), (f) config file created by `save_default()` is valid JSON that round-trips through `load()`.
- **Recommendation:** Add a test module for `config.rs` covering all six cases.
- **Verification:** `cargo test config` passes. Specifically, the idempotency test calls `generate_confirm_token()` twice and asserts equality.

### R2-A4-5 (IMPORTANT)
- **Section:** `server.rs` — `check_confirm_token` Non-Constant-Time Comparison
- **Severity:** IMPORTANT
- **Finding:** At `server.rs` line 540, `provided != expected_token` uses standard string comparison, which is timing-sensitive. This was flagged as I3 in QC-R1. From a testability perspective: the timing side-channel is difficult to test in a unit test but can be verified by code inspection. The fix (XOR-accumulate or `subtle::ConstantTimeEq`) is machine-verifiable by asserting the absence of `!=` in the `check_confirm_token` function body.
- **Recommendation:** Replace `!=` with constant-time comparison. Add a test that at minimum verifies correct accept/reject behavior (valid token → Ok, invalid token → Err).
- **Verification:** `grep -n '!=' server.rs | grep -c 'provided'` returns 0 after fix. Tests exist for both valid and invalid token paths.

### R2-A4-6 (IMPORTANT)
- **Section:** `plenum-app.toml` — Hardcoded Upgrade Code
- **Severity:** IMPORTANT
- **Finding:** `upgrade_code = "A1B2C3D4-E5F6-7890-ABCD-EF1234567890"` is a hand-typed placeholder (flagged as I6 in QC-R1). No derivation function exists — there is no code to test. The placeholder UUID has no collision analysis, no derivation provenance, and no context-string sensitivity.
- **Recommendation:** Implement a sponge-based deterministic derivation function (TLSponge-385 with context string `"NinjaExec-UpgradeCode-v1"`) and add tests for idempotency and version sensitivity.
- **Verification:** A `test_upgrade_code_deterministic()` test exists that asserts same-input-same-output and different-input-different-output.

### R2-A4-7 (MINOR)
- **Section:** `audit.rs` — `hash_payload` Misleading Prefix
- **Severity:** MINOR
- **Finding:** `hash_payload()` at line 62 prefixes the output with `"tis27:"` but actually calls `ternary_math::sponge::derive_key` which is TLSponge-385, not TIS-27. A test asserts `h1.starts_with("tis27:")` — this test will break when the prefix is corrected.
- **Recommendation:** Change prefix to `"sponge385:"` or `"tlsponge:"`. Update the test assertion accordingly.
- **Verification:** `hash_payload(b"test").starts_with("sponge385:")` passes.

### R2-A4-8 (MINOR)
- **Section:** `confirm.rs` — Test Coverage Gaps
- **Severity:** MINOR
- **Finding:** Missing: (a) `expire_stale()` removes entries older than timeout, (b) `pending_list()` returns correct count after multiple submits, (c) `check()` on a non-existent ID returns `Some(Rejected)`, (d) case-insensitivity of context matching.
- **Recommendation:** Add the four missing test cases.
- **Verification:** `cargo test confirm` includes all new tests.

### R2-A4-9 (MINOR)
- **Section:** `cli.rs` — No Tests
- **Severity:** MINOR
- **Finding:** `cli.rs` has zero tests. The argument parser handles 11 subcommands with flags. None of the parsing logic is tested. Edge cases include: `--port` without a value, unknown subcommand falls through to `Run`, `sign` subcommand reuses positional arg index 2 which may conflict with `--data-dir`.
- **Recommendation:** Add unit tests for `parse_args()` using `std::env::set_var` or by refactoring to accept `&[String]` instead of reading `std::env::args()` directly.
- **Verification:** Tests verify that `parse_args()` returns the expected `Command` variant for each subcommand.

### R2-A4-10 (IMPORTANT)
- **Section:** `main.rs` — Confirm Token Printed to stdout
- **Severity:** IMPORTANT
- **Finding:** At `main.rs` line 193, `println!("[NinjaExec] Token: {}", token)` prints the confirm token to stdout during `init`. This was flagged as I7 in QC-R1. The test criterion is machine-verifiable: `ninja-exec init 2>&1 | grep -c 'Token:'` should return 0. Currently it returns 1.
- **Recommendation:** Remove the token value from stdout. Print only the file path.
- **Verification:** `ninja-exec init` stdout does not contain any base64-encoded string on the token line.

### R2-A4-11 (MINOR)
- **Section:** `plenum-app.toml` — Spec vs Implementation Consistency
- **Severity:** MINOR
- **Finding:** The manifest declares `binary = "ninja-exec.exe"` but `Cargo.toml` specifies `[[bin]] name = "ninja-exec"` without `.exe`. Icon files referenced may not exist. `configure_command = ""` may cause unexpected behavior.
- **Recommendation:** (a) Verify icon assets exist. (b) Remove `.exe` from `binary` or make it conditional. (c) Set `configure_command` to an explicit value or remove it.
- **Verification:** `binary` field matches `cargo build --release` output.

### R2-A4-12 (IMPORTANT)
- **Section:** `Cargo.toml` — Dependencies Not Patch-Pinned
- **Severity:** IMPORTANT
- **Finding:** Dependencies use semver ranges: `tokio = "1"`, `serde = "1"`, etc. Only `axum`, `base64`, and `tower-http` are pinned. Builds are not reproducible.
- **Recommendation:** Pin all dependencies to exact patch versions.
- **Verification:** `grep -c '"[0-9]"' Cargo.toml` returns 0.

### Agent 4 — Coverage Matrix

| Category | COVERED | PARTIALLY | NOT COVERED |
|----------|---------|-----------|-------------|
| HTTP endpoints (8 × valid+invalid) | 0 | 0 | 22 |
| Signing engine | 5 | 0 | 3 |
| Keystore | 5 | 2 | 6 |
| Config | 0 | 0 | 4 |
| Confirmation | 9 | 0 | 2 |
| Audit | 2 | 0 | 2 |
| CLI | 0 | 0 | 2 |
| Invariant compliance | 0 | 1 | 2 |
| Upgrade/identity | 0 | 0 | 2 |
| **TOTAL** | **21** | **3** | **45** |

**Coverage rate: 30% COVERED, 4% PARTIALLY, 65% NOT COVERED**

### Agent 4 — Full Coverage Matrix Detail

| Test Area | Source File | Status | Evidence |
|---|---|---|---|
| HTTP: POST /sign (valid) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /sign (invalid base64) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /sign (invalid context) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /sign (locked keystore) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /sign (rate limited) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /sign (confirmation rejected) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /sign (confirmation timeout) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /verify (valid) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /verify (invalid base64) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: GET /pubkey | `server.rs` | NOT COVERED | No integration tests |
| HTTP: GET /pubkey (no key) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: GET /status | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /lock | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /unlock (valid) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /unlock (wrong passphrase) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /unlock (rate limited) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: GET /confirm/pending (valid token) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: GET /confirm/pending (invalid token) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /confirm/decide (approve) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /confirm/decide (reject) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /confirm/decide (invalid decision) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: POST /confirm/decide (not found) | `server.rs` | NOT COVERED | No integration tests |
| HTTP: CORS policy enforcement | `server.rs` | NOT COVERED | No CORS restriction tests |
| TL-DSA sign-verify roundtrip | `signing_engine.rs` | COVERED | `test_roundtrip_sign_verify` |
| TL-DSA tampered payload rejected | `signing_engine.rs` | COVERED | `test_tampered_payload_rejected` |
| TL-DSA pubkey base64 export | `signing_engine.rs` | COVERED | `test_export_pubkey_b64` |
| TL-DSA fingerprint determinism | `signing_engine.rs` | COVERED | `test_fingerprint_deterministic` |
| TL-DSA key sizes | `signing_engine.rs` | COVERED | `test_key_sizes` |
| TL-DSA cross-keypair rejection | `signing_engine.rs` | NOT COVERED | No test |
| TL-DSA Rep C address binding | `signing_engine.rs` | NOT COVERED | No Rep C in codebase |
| TL-DSA malformed signature rejection | `signing_engine.rs` | NOT COVERED | No test |
| Keystore create and open | `keystore.rs` | COVERED | `test_create_and_open` |
| Keystore wrong passphrase | `keystore.rs` | COVERED | `test_wrong_passphrase` |
| Keystore passphrase too short | `keystore.rs` | COVERED | `test_passphrase_too_short` |
| Keystore load public key only | `keystore.rs` | COVERED | `test_load_public_key_only` |
| Keystore KDF header params | `keystore.rs` | COVERED | `test_keystore_header_contains_kdf_params` |
| Keystore empty passphrase | `keystore.rs` | NOT COVERED | No test |
| Keystore already exists | `keystore.rs` | NOT COVERED | No test |
| Keystore corrupted file | `keystore.rs` | NOT COVERED | No test |
| Keystore unsupported version | `keystore.rs` | NOT COVERED | No test |
| Keystore lock zeroizes | `keystore.rs` | PARTIALLY COVERED | calls lock/unlock but doesn't verify zeroization |
| Keystore Unicode passphrase | `keystore.rs` | NOT COVERED | No test |
| Keystore file permissions (Unix) | `keystore.rs` | NOT COVERED | No test |
| Keystore constant-time tag | `keystore.rs` | PARTIALLY COVERED | Verified by code inspection, no dedicated test |
| Config load defaults | `config.rs` | NOT COVERED | No tests |
| Config load invalid JSON | `config.rs` | NOT COVERED | No tests |
| Config save_default idempotency | `config.rs` | NOT COVERED | No tests |
| Config generate_confirm_token | `config.rs` | NOT COVERED | No tests |
| Confirmation requires_confirmation | `confirm.rs` | COVERED | `test_requires_confirmation` |
| Confirmation auto_approve | `confirm.rs` | COVERED | `test_auto_approve` |
| Confirmation headless auto-approves | `confirm.rs` | COVERED | `test_headless_auto_approves` |
| Confirmation interactive rejects | `confirm.rs` | COVERED | `test_interactive_rejects_without_gui` |
| Confirmation auto-approve always passes | `confirm.rs` | COVERED | `test_auto_approve_operations_always_pass` |
| Confirmation queue submit/approve | `confirm.rs` | COVERED | `test_confirmation_queue_submit_approve` |
| Confirmation queue submit/reject | `confirm.rs` | COVERED | `test_confirmation_queue_submit_reject` |
| Confirmation queue timeout | `confirm.rs` | COVERED | `test_confirmation_queue_timeout` |
| Confirmation queue pending | `confirm.rs` | COVERED | `test_confirmation_queue_pending` |
| Confirmation expire_stale | `confirm.rs` | NOT COVERED | No test |
| Confirmation case-insensitivity | `confirm.rs` | NOT COVERED | No test |
| Audit append and format | `audit.rs` | COVERED | `test_audit_append_and_format` |
| Audit hash_payload deterministic | `audit.rs` | COVERED | `test_hash_payload_deterministic` |
| Audit append failure (read-only dir) | `audit.rs` | NOT COVERED | No test |
| Audit fail-closed on write error | `server.rs`+`audit.rs` | NOT COVERED | append() returns () |
| CLI argument parsing | `cli.rs` | NOT COVERED | No tests |
| Upgrade code derivation | `plenum-app.toml` | NOT COVERED | Hardcoded placeholder |
| Env var PLENUM_PASSPHRASE handling | `main.rs` | NOT COVERED | Not zeroized |
| Env var NINJA_EXEC_PORT handling | `cli.rs` | NOT COVERED | No test |
| INVARIANT 8: No raw binary in sponge | `keystore.rs` | PARTIALLY COVERED | Needs ternary_math audit |
| INVARIANT 9: Rep C in all crypto ops | All | NOT COVERED | No Rep C anywhere |

### Agent 4 — Summary Verdict: **FAIL**

Four unresolved CRITICALs from R1, zero HTTP endpoint integration tests, and 65% of test areas uncovered.

---

## Agent 5: Senior Developer — New Findings

### R2-A5-1 (IMPORTANT)
- **Section:** `config.rs` — NinjaExecConfig schema
- **Severity:** IMPORTANT
- **Finding:** The `NinjaExecConfig` struct is missing fields required by the C1/C2/C3 remediations: (a) `allowed_origins: Vec<String>` for CORS restriction (C3), (b) `rep_c_address: Option<String>` or equivalent for identity binding (C2), (c) `headless_allow: Vec<String>` for restricted headless mode (I4). The current struct has only `port`, `rate_limit_per_minute`, `confirmation`, and `confirm_token`. Without these fields, the critical fixes cannot be implemented as configuration-driven changes.
- **Recommendation:** Add all three fields to `NinjaExecConfig` with appropriate defaults: `allowed_origins` defaults to empty (deny all cross-origin), `rep_c_address` defaults to `None` (derived during init), `headless_allow` defaults to `["verify", "pubkey", "status"]`.
- **Verification:** Deserialize a `ninja-exec.json` containing all new fields; verify defaults are applied when fields are absent.

### R2-A5-2 (IMPORTANT)
- **Section:** `keystore.rs` — Keystore file format
- **Severity:** IMPORTANT
- **Finding:** The keystore binary format (`NJXK0002`) is fixed-length at `HEADER_LEN` bytes with no extensibility mechanism. Adding a Rep C address field (C2 remediation) requires either: (a) bumping the magic/version and creating a new fixed-length format, (b) storing the Rep C address in an auxiliary file, or (c) appending variable-length data after the fixed header with a length prefix. Option (a) is cleanest but breaks existing keystores without a migration path. The `open()` method at line 239 performs an exact length check which will reject any extended format.
- **Recommendation:** Define `NJXK0003` format that appends a 54-byte Rep C address after the existing fields. Implement a migration path: if `NJXK0002` is detected, derive the Rep C address from the stored public key and rewrite as `NJXK0003`. Update `HEADER_LEN` and `open()` length check accordingly.
- **Verification:** Create a v2 keystore, verify it is automatically migrated to v3, and verify the Rep C address matches the public key derivation.

### R2-A5-3 (MINOR)
- **Section:** `signing_engine.rs` — fingerprint context string
- **Severity:** MINOR
- **Finding:** The `fingerprint()` function uses `derive_key(b"NinjaExec-FP", ...)` which is listed as an unregistered context string. The context string should be registered in the canonical context string registry.
- **Recommendation:** Register `NinjaExec-FP` in the canonical context string registry. No code change needed.
- **Verification:** Check the context string registry contains `NinjaExec-FP` with its documented purpose.

### R2-A5-4 (MINOR)
- **Section:** `audit.rs` lines 59–63 — hash_payload prefix
- **Severity:** MINOR
- **Finding:** `hash_payload()` produces output prefixed with `tis27:` but the actual hash function is TLSponge-385, not TIS-27. The prefix should accurately reflect the algorithm used.
- **Recommendation:** Change prefix from `"tis27:"` to `"tlsponge385:"` or simply `"sponge:"`.
- **Verification:** Grep for `tis27:` in all source files; confirm no remaining mislabeled prefixes.

### R2-A5-5 (IMPORTANT)
- **Section:** `keystore.rs` — Bespoke authenticated encryption
- **Severity:** IMPORTANT
- **Finding:** The keystore uses a hand-rolled encrypt-then-MAC scheme: XOR keystream from `derive_key(b"NinjaExec-KS-STREAM", ...)` followed by a separate MAC tag from `derive_key(b"NinjaExec-KS-TAG", ...)`. While the construction appears sound (encrypt-then-MAC with separate key material derivation, constant-time tag verification), it is not the canonical T-AE-MAC construction. T-AE-MAC provides IND-CPA + INT-CTXT with a formally analyzed construction. The bespoke scheme has not been formally analyzed.
- **Recommendation:** If T-AE-MAC is available in `ternary_math`, replace the bespoke encrypt/decrypt functions with T-AE-MAC calls. If not yet exported, document as a temporary measure with a tracking issue for migration. **Flag for Security Engineer (Agent 1) review**.
- **Verification:** After migration, verify keystore round-trip: create → lock → unlock → sign produces the same results.

### R2-A5-6 (IMPORTANT)
- **Section:** `plenum-app.toml` — upgrade_code
- **Severity:** IMPORTANT
- **Finding:** The `upgrade_code` field is `"A1B2C3D4-E5F6-7890-ABCD-EF1234567890"` — a visually obvious placeholder. For WiX MSI installers, the UpgradeCode is the persistent identifier that Windows uses to detect existing installations. A hand-typed placeholder: (a) risks collision, (b) cannot be deterministically regenerated if lost, (c) looks unprofessional in installer logs.
- **Recommendation:** Derive deterministically using `sponge::derive_key(b"NinjaExec-UPGRADE-CODE", b"Capomastro Holdings Ltd.NinjaExec", 16)` and format as a GUID. Document the derivation.
- **Verification:** Run the derivation, verify stable GUID, update `plenum-app.toml`.

### R2-A5-7 (IMPORTANT)
- **Section:** `server.rs` lines 533–548 — check_confirm_token
- **Severity:** IMPORTANT
- **Finding:** The `check_confirm_token` function uses standard string comparison (`provided != expected_token`) which is not constant-time. An attacker with local network access could potentially use timing side-channels to extract the confirm token byte-by-byte. The keystore tag comparison correctly uses XOR-accumulate (line 130–133), but the server token check does not.
- **Recommendation:** Replace with a constant-time comparison. Either use `subtle::ConstantTimeEq` or implement the XOR-accumulate pattern already used in `keystore.rs`.
- **Verification:** Verify `check_confirm_token` uses constant-time comparison by code inspection.

### R2-A5-8 (IMPORTANT)
- **Section:** `confirm.rs` lines 159–173 — headless mode logic
- **Severity:** IMPORTANT
- **Finding:** The `evaluate_confirmation` function auto-approves ALL operations in headless mode, including `exec`, `deploy`, `key-rotation`, and `config-update`. The function checks `if headless { return ConfirmationResult::AutoApproved; }` with no filtering. Combined with the CORS wildcard (C3), any website can trigger arbitrary signed operations on a headless instance.
- **Recommendation:** Add a `headless_allow` list to `NinjaExecConfig`. In `evaluate_confirmation`, only auto-approve operations in `headless_allow`. Default to read-only operations: `["verify", "pubkey", "status", "tail", "file-pull"]`.
- **Verification:** Start NinjaExec in headless mode, send a `sign` request with context `exec`, verify it is NOT auto-approved.

### R2-A5-9 (MINOR)
- **Section:** `cli.rs` — Argument parsing robustness
- **Severity:** MINOR
- **Finding:** The argument parser silently ignores unknown flags. The `sign` subcommand reuses `args.get(2)` which may collide with flags parsed by the main loop.
- **Recommendation:** Parse positional arguments for `sign` and `verify` subcommands after flag parsing, or use `clap`.
- **Verification:** Test `ninja-exec sign --data-dir /tmp myfile.txt` and verify `file` is correctly set.

### R2-A5-10 (IMPORTANT)
- **Section:** `plenum-app.toml` — configure_command and port discovery
- **Severity:** IMPORTANT
- **Finding:** The `configure_command` field is empty. The `status_port` is hardcoded to `21027`. No mechanism for tray icon to discover the actual port, for the installer to verify port availability, or for the tray to launch a configuration UI. The autostart mechanism is not specified.
- **Recommendation:** (a) Define a port discovery mechanism (well-known file or config). (b) Specify the autostart mechanism explicitly. (c) Add a `first_run` action to start the agent after init.
- **Verification:** After install and first-run, verify the tray icon can reach `/status`. Verify the agent starts on system boot.

### R2-A5-11 (MINOR)
- **Section:** `config.rs` lines 46–56 — Config load silently falls back
- **Severity:** MINOR
- **Finding:** `NinjaExecConfig::load()` silently returns defaults if JSON parsing fails. A malformed config file should produce a clear error, not silently apply defaults. An operator who misconfigures their allowed origins would get the default (currently no restriction), silently negating their security intent.
- **Recommendation:** Log a warning when JSON parsing fails. Consider making parse failures fatal.
- **Verification:** Create a malformed `ninja-exec.json`, start NinjaExec, verify a warning or error is emitted.

### R2-A5-12 (MINOR)
- **Section:** `server.rs` — Missing API response types for error cases
- **Severity:** MINOR
- **Finding:** Error responses are constructed inline using `serde_json::json!()` macros. The `ErrorResponse` struct is defined but never used (`#[allow(dead_code)]`). Error response shapes are not enforced by the type system.
- **Recommendation:** Use the `ErrorResponse` struct for all error responses, or remove it.
- **Verification:** Grep for `serde_json::json!` in error paths; verify consistent `code` and `error` fields.

### R2-A5-13 (IMPORTANT)
- **Section:** `Cargo.toml` — Dependency version pinning
- **Severity:** IMPORTANT
- **Finding:** Dependencies use semver ranges. For a security-critical signing agent, reproducible builds are essential. A minor version bump could introduce behavioral changes affecting cryptographic operations.
- **Recommendation:** Pin all dependencies to exact patch versions: `tokio = "=1.36.0"`, `serde = "=1.0.197"`, etc.
- **Verification:** Run `cargo build` twice on different dates; verify identical binary hashes.

### Agent 5 — Feasibility Risk Table

| Task | Risk | Justification |
|------|------|---------------|
| C1: Rep C in signatures | **MEDIUM** | Changes sign/verify signatures, all call sites; straightforward but pervasive |
| C2: Rep C provisioning + storage | **HIGH** | Keystore format v2→v3, Rep C derivation, touches every module |
| C3: CORS origin allowlist | **LOW** | Config field + tower-http API; well-documented |
| C4: Audit fail-closed | **MEDIUM** | `append()` → `Result`, propagate through all handlers; new failure modes |
| I1: KDF iteration increase | **LOW** | Constant change + benchmark |
| I3: Constant-time token comparison | **LOW** | ~5 lines |
| I4: Headless mode restriction | **LOW** | Config field + one check |
| I5: T-AE-MAC migration | **HIGH** | Depends on T-AE-MAC availability; keystore format migration |
| I6: Deterministic upgrade code | **LOW** | One-time derivation |
| I7/C7: Token not printed to stdout | **LOW** | Remove println |
| I7: Dependency pinning | **LOW** | cargo update, record versions |
| I11: Context string registration | **LOW** | Documentation only |
| Port discovery mechanism | **MEDIUM** | Requires tray/agent coordination |
| Autostart mechanism | **MEDIUM** | Platform-specific |

### Agent 5 — Summary Verdict: **FAIL**

Four confirmed CRITICAL findings remain unresolved. The most architecturally significant — C2 — requires a keystore format migration, touches every module, and depends on `ternary_math` exporting a Rep C derivation function. The implementation path is feasible but the volume of blocking issues means NinjaExec cannot pass QC-R2.

**Conditions for PASS:** Resolve C1, C2, C3, C4, plus I3, I4, and R2-A5-1 (config schema expansion).

---

## Agent 6: Infrastructure Maintainer — New Findings

### R2-A6-1 (IMPORTANT)
- **Section:** `plenum-app.toml` [first_run], `main.rs` lines 157-211
- **Severity:** IMPORTANT
- **Finding:** The `init` flow has no pre-flight validation. Before creating a keystore, the installer should verify: (1) the data directory is writable, (2) sufficient disk space exists, (3) no antivirus/EDR file-system hook will quarantine the keystore file, (4) the binary is not running from a read-only mount. Currently, `fs::create_dir_all` is the first I/O operation and its failure message does not tell the operator *why* it failed or *what to do*.
- **Recommendation:** Add a pre-flight check function that validates directory writability (create+delete a temp file), reports the resolved data directory path, and provides actionable guidance.
- **Verification:** Run `ninja-exec init` on a machine with read-only AppData redirection; confirm the error message names the path and suggests `--data-dir`.

### R2-A6-2 (IMPORTANT)
- **Section:** `plenum-app.toml` [uninstall], `keystore.rs`
- **Severity:** IMPORTANT
- **Finding:** No option for a clean uninstall that removes key material. An operator decommissioning a machine needs a way to securely wipe key material. There is no `ninja-exec wipe` or `ninja-exec destroy` command. The `preserve_message` references `%APPDATA%\NinjaExec` but does not expand the variable at display time.
- **Recommendation:** (1) Expand `%APPDATA%` in the uninstall dialog. (2) Add a `ninja-exec destroy` command that securely overwrites the keystore file before deletion. (3) Add a `preserve_data = "prompt"` option.
- **Verification:** Run `ninja-exec destroy` and verify the keystore file is overwritten before deletion.

### R2-A6-3 (IMPORTANT)
- **Section:** `config.rs` lines 46-56
- **Severity:** IMPORTANT
- **Finding:** `NinjaExecConfig::load()` silently falls back to defaults if the config file exists but contains invalid JSON. An operator who makes a typo will get default settings with no warning — silently degrading security posture.
- **Recommendation:** Print a clear error to stderr and exit with a non-zero code rather than silently falling back.
- **Verification:** Create a `ninja-exec.json` with a syntax error; confirm it exits with an error message.

### R2-A6-4 (MINOR)
- **Section:** `config.rs` lines 58-68, `main.rs` line 190
- **Severity:** MINOR
- **Finding:** `save_default()` and `generate_confirm_token()` both silently discard write errors. If the config file cannot be written, the confirm token is generated in memory but never persisted. The operator sees "Confirm token generated" but the token is lost on next restart.
- **Recommendation:** Return `Result` from both functions. Display a warning about config write failure.
- **Verification:** Set the data directory to read-only after keystore creation; confirm a warning is displayed.

### R2-A6-5 (CRITICAL)
- **Section:** `main.rs` lines 191-193
- **Severity:** CRITICAL
- **Finding:** The confirm token is printed to stdout during `init`: `println!("[NinjaExec] Token: {}", token)`. This token grants the ability to approve or reject all signing requests. In CI/CD pipelines, stdout is captured to build logs. In enterprise environments with centralized log collection, the token leaks to log aggregators. Any system that captures process stdout now has signing approval authority.
- **Recommendation:** Do not print the token to stdout. Print only the storage location. Set file permissions on `ninja-exec.json` to 0600 (Unix) or equivalent ACL (Windows).
- **Verification:** Run `ninja-exec init` and capture stdout; confirm the token value does not appear.

### R2-A6-6 (IMPORTANT)
- **Section:** `main.rs` lines 414-505, `cli.rs`
- **Severity:** IMPORTANT
- **Finding:** There is no key backup or key export mechanism for disaster recovery. If the machine's disk fails, the operator's signing identity is permanently lost. No `ninja-exec backup` command exists. No documented key rotation procedure. Day-two operations are undocumented.
- **Recommendation:** (1) Add `ninja-exec backup <output-path>`. (2) Document key rotation procedure. (3) Create a deployment guide. (4) Add a `ninja-exec doctor` command.
- **Verification:** Run `ninja-exec backup /tmp/backup.keystore`; verify the backup can be restored.

### R2-A6-7 (CRITICAL)
- **Section:** `audit.rs` lines 12-25, `server.rs` passim
- **Severity:** CRITICAL
- **Finding:** Audit entries use `origin: Option<String>` populated from HTTP `Origin`/`Referer` headers — browser-controlled, spoofable headers. Per INVARIANT 9, all audit records must reference nodes by Rep C address. No Rep C address appears in any audit entry. The `origin` field contains URLs useless for cross-referencing with TDNS topology records.
- **Recommendation:** Add a `node_address` field to `AuditEntry` populated with the agent's own Rep C address. Retain `origin` as supplementary diagnostic data. Every audit entry must include the signer's Rep C address.
- **Verification:** Inspect `ninja-exec-audit.jsonl`; confirm every entry contains a `node_address` field with a valid 54-trit Rep C address.

### R2-A6-8 (IMPORTANT)
- **Section:** `plenum-app.toml` [app], `Cargo.toml`
- **Severity:** IMPORTANT
- **Finding:** The `upgrade_code` is a hand-typed placeholder GUID. From an infrastructure perspective, if two PlenumNET products use the same placeholder GUID, Windows Installer will treat them as the same product. Upgrades will collide, uninstalls will remove the wrong product.
- **Recommendation:** Generate a deterministic upgrade code from the product name using namespace UUID v5.
- **Verification:** Verify the upgrade_code is unique across all `plenum-app.toml` files. Confirm zero matches for `A1B2C3D4` after fix.

### R2-A6-9 (MINOR)
- **Section:** `keystore.rs` lines 324-335
- **Severity:** MINOR
- **Finding:** `default_data_dir()` on Windows uses `APPDATA` (roaming). For a tray agent with machine-local service binding, `LOCALAPPDATA` (non-roaming, per-user) is more appropriate. Roaming AppData may cause issues if the user logs in to multiple machines.
- **Recommendation:** Use `LOCALAPPDATA` as the primary Windows location. Document the data directory selection logic.
- **Verification:** On a domain-joined machine with roaming profiles, verify `ninja-exec init` creates the keystore under `LOCALAPPDATA`.

### R2-A6-10 (MINOR)
- **Section:** `server.rs` lines 633-643
- **Severity:** MINOR
- **Finding:** Startup message printed to stdout. If run as a Windows service, stdout may not be captured. No Windows Event Log integration, no structured startup event, no `/health` endpoint.
- **Recommendation:** (1) Write startup events to the audit log. (2) Add a `/health` endpoint. (3) On Windows, consider writing a startup event to the Application event log.
- **Verification:** Start as a Windows service; verify the audit log contains a startup entry with bind address and port.

### R2-A6-11 (IMPORTANT)
- **Section:** `Cargo.toml` lines 19-31
- **Severity:** IMPORTANT
- **Finding:** Dependencies not pinned to patch versions. Non-pinned dependencies mean two builds on different dates may produce different binaries. For a security-critical signing agent, build reproducibility is essential.
- **Recommendation:** Pin all dependencies to exact patch versions. Add `Cargo.lock` to version control. Document expected binary hash for each release.
- **Verification:** Build on two clean machines from the same commit; verify identical binary hashes.

### R2-A6-12 (MINOR)
- **Section:** `main.rs` lines 427-431, `cli.rs`
- **Severity:** MINOR
- **Finding:** The `PLENUM_PASSPHRASE` environment variable is never zeroed from the process environment after use. On Linux, `/proc/<pid>/environ` exposes it for the lifetime of the process.
- **Recommendation:** Call `std::env::remove_var("PLENUM_PASSPHRASE")` immediately after reading. Document as CI/CD only.
- **Verification:** After startup, inspect `/proc/<pid>/environ`; confirm the variable is no longer present.

### Agent 6 — Operator Readiness Checklist

| # | Question | Answer | Notes |
|---|----------|--------|-------|
| 1 | Is there a step-by-step deployment guide? | **NO** | No deployment guide, troubleshooting doc, or FAQ exists |
| 2 | Can I install silently via SCCM/Intune? | **PARTIALLY** | `PLENUM_PASSPHRASE` env var enables silent init, but no MSI/MSIX package exists yet |
| 3 | What are the minimum OS requirements? | **NO** | Not documented |
| 4 | Does the installer work on a locked-down corporate PC? | **NO** | No Group Policy testing, no EDR compatibility |
| 5 | What happens if init fails mid-way? | **PARTIALLY** | Keystore uses atomic rename, but config write failure leaves inconsistent state |
| 6 | How do I verify the agent is running? | **YES** | `ninja-exec status` or `GET /status` on port 21027 |
| 7 | How do I back up the signing key? | **NO** | No backup command exists |
| 8 | How do I rotate the signing key? | **NO** | No key rotation command or procedure documented |
| 9 | What happens on uninstall? | **PARTIALLY** | `preserve_data = true` retains keystore, but no secure wipe option |
| 10 | Are error messages actionable? | **PARTIALLY** | Keystore errors clear, but config parse and audit write failures silent |
| 11 | Are audit records compliant with INVARIANT 9? | **NO** | No Rep C address in any audit entry |
| 12 | Is the binary code-signed? | **NO** | No signing certificate referenced |
| 13 | Is there a CI/CD pipeline for builds? | **NO** | No GitHub Actions workflow |
| 14 | Can I run this as a Windows service? | **PARTIALLY** | Can be wrapped with NSSM/WinSW, but no native service registration |
| 15 | Is the CORS policy safe for production? | **NO** | `allow_origin(Any)` |
| 16 | Does the confirm token survive restarts? | **PARTIALLY** | Saved to `ninja-exec.json`, but write failures silently ignored |

**Score: 1 YES, 5 PARTIALLY, 10 NO** — operationally undeployable.

### Agent 6 — Summary Verdict: **FAIL**

Strong cryptographic foundations but operationally undeployable due to INVARIANT 9 violations, credential exposure via stdout, CORS wildcard, silent failures, and missing deployment documentation.

---

# PART IV — COMBINED VERDICT AND RESOLUTION PATH

## Final Verdict (R1 + R2 Combined)

| Agent | Round | Verdict | New Findings |
|-------|-------|---------|-------------|
| A1 — Security Engineer | R1 | **FAIL** | 15 (2C, 8I, 5M) |
| A2 — DevOps Automator | R1 | **FAIL** | 12 (3C, 6I, 3M) |
| A3 — PlenumNET Integration | R1 | **FAIL** | 9 (2C, 5I, 2M) |
| A4 — Evidence Collector | R2 | **FAIL** | 12 (1C, 7I, 4M) |
| A5 — Senior Developer | R2 | **FAIL** | 13 (0C, 8I, 5M) |
| A6 — Infrastructure Maintainer | R2 | **FAIL** | 12 (2C, 6I, 4M) |

**Combined Verdict: FAIL** (6/6 agents, any FAIL → FAIL)

**Total unique findings (deduplicated): 7 CRITICAL, 20 IMPORTANT, 13 MINOR = 40 findings**

NinjaExec does not proceed to QC-R3.

---

## Combined CRITICAL Finding Summary

| ID | Finding | R1 Agents | R2 AGREE | Status |
|----|---------|-----------|----------|--------|
| C1 | No Rep C address binding in TL-DSA signatures (INVARIANT 9) | A1, A2, A3 | A4✓ A5✓ A6✓ | OPEN |
| C2 | No Rep C address exists anywhere in codebase (INVARIANT 9) | A1, A2, A3 | A4✓ A5✓ A6✓ | OPEN |
| C3 | CORS wildcard origin creates signature oracle | A1, A2, A3 | A4✓ A5✓ A6✓ | OPEN |
| C4 | Audit log silently swallows all write failures | A1, A2 | A4✓ A5✓ A6✓ | OPEN |
| C5 | No integration tests for any HTTP endpoint | — | A4 (new) | OPEN |
| C6 | Audit entries lack Rep C addresses (INVARIANT 9) | — | A6 (new) | OPEN |
| C7 | Confirm token printed to stdout (credential leak) | — | A6 (escalated) | OPEN |

---

## Resolution Path

**Phase 1 — CRITICAL fixes (blocks re-review):**
- Derive/accept Rep C address during `init`, store in keystore v3 format (C2)
- Bind Rep C into TL-DSA signing context + operation context (C1, I11)
- Add Rep C to audit entries (C6)
- CORS configurable allowlist, deny-all default (C3)
- Audit `append()` returns `Result`, sign fails if audit fails (C4)
- Remove token from stdout (C7)
- Add HTTP endpoint integration tests (C5)

**Phase 2 — IMPORTANT fixes (blocks release):**
- Config schema expansion for new fields (I12)
- Constant-time token comparison (I3)
- Headless mode `headless_allow` restriction (I4)
- Dependency pinning (I7)
- Init pre-flight validation (I17)
- Silent config parse → fatal error (I19)

**Phase 3 — Pre-release polish:**
- KDF iteration increase (I1)
- Key backup/rotation/destroy commands (I18)
- Deployment guide documentation (Operator Readiness)
- Context string registration (I10)
- Remaining MINOR items

---

## Individual Agent Reports

**Round 1:**
- [Agent 1 — Security Engineer](qc-r1-agent1-security-engineer.md)
- [Agent 2 — DevOps Automator](qc-r1-agent2-devops-automator.md)
- [Agent 3 — PlenumNET Integration Specialist](qc-r1-agent3-plenumnet-integration.md)
- [QC-R1 Consolidated](qc-r1-consolidated.md)

**Round 2:**
- [Agent 4 — Evidence Collector / QA Lead](qc-r2-agent4-evidence-collector.md)
- [Agent 5 — Senior Developer](qc-r2-agent5-senior-developer.md)
- [Agent 6 — Infrastructure Maintainer](qc-r2-agent6-infrastructure-maintainer.md)
