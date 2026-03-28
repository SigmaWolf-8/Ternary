# QC-R2 Agent 4 — Evidence Collector / QA Lead Review

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Task:** #54
**Review Date:** 2026-03-28
**Round:** QC-R2 (Quality & Evidence)
**Reviewer:** Agent 4 — Evidence Collector / QA Lead
**Finding ID Convention:** R2-A4-{N}

---

## Round 1 Response

### C1 — No Rep C Address Binding in TL-DSA Signatures (INVARIANT 9)

**Response: AGREE**

Confirmed by direct inspection of `signing_engine.rs` lines 13–18. The `sign()` function passes `(secret_key, payload, VARIANT)` with no Rep C address in the context. The `verify()` function mirrors this — no address binding on either side. There is no domain-separated message construction of any form. The signature is completely identity-unbound. This is a testable, machine-verifiable gap: a test that signs the same payload from two different keystores will produce signatures that are interchangeable (cross-operator replay), which should never be possible under INVARIANT 9. No existing test asserts cross-operator non-interchangeability. CRITICAL stands.

### C2 — No Rep C Address Exists Anywhere in Codebase (INVARIANT 9)

**Response: AGREE**

Confirmed by full codebase scan. The `Keystore` struct stores only `public_key: Option<Vec<u8>>` and `secret_key: Option<Vec<u8>>` — no address field. The `export-operator` command in `main.rs` lines 252–257 identifies the node as `operator@{hostname}` using `COMPUTERNAME` or `HOSTNAME` environment variables — explicitly prohibited by INVARIANT 9. Audit entries record HTTP `origin` headers (URLs) as identity, not Rep C addresses. The `PendingRequest` struct in `confirm.rs` also uses `origin: Option<String>` (URL-based). There is zero Rep C representation in any data structure, configuration, or file format. CRITICAL stands.

### C3 — CORS Wildcard Origin Creates Signature Oracle

**Response: AGREE**

Confirmed at `server.rs` line 615–618:
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```
This is a triple-`Any` CORS policy. Combined with the localhost binding, any website loaded in the operator's browser can send `POST /sign` requests cross-origin. The `NinjaExecConfig` struct has no `allowed_origins` field — no mechanism exists to restrict origins even if desired. In headless mode (C4-adjacent, I4), this becomes a fully automated signature oracle exploitable from any browser tab. A test for this is trivially scriptable: send a cross-origin `fetch()` from a different origin and verify the response is not blocked. No such test exists. CRITICAL stands.

### C4 — Audit Log Silently Swallows All Write Failures

**Response: AGREE**

Confirmed at `audit.rs` lines 38–51. The `append()` method returns `()` (unit), not `Result`. It uses `let _` to discard errors from `fs::create_dir_all`, `OpenOptions::new().open()`, and `writeln!()`. In `server.rs`, every audit call follows the pattern `if let Ok(log) = state.audit_log.lock() { log.append(&entry); }` — if the lock is poisoned or the write fails, the signing operation still succeeds and returns a valid signature. The result is an unaudited cryptographic artifact. A test can verify this by pointing the audit log at a read-only directory and confirming that the sign endpoint still returns 200 OK — demonstrating the silent failure. No such negative test exists. CRITICAL stands.

---

## Findings

### Finding R2-A4-1
- **Section:** HTTP API — All 8 Endpoints
- **Severity:** CRITICAL
- **Finding:** No integration test exists for any of the 8 HTTP endpoints (`/sign`, `/verify`, `/pubkey`, `/status`, `/lock`, `/unlock`, `/confirm/pending`, `/confirm/decide`). The `server.rs` file contains zero `#[cfg(test)]` blocks. The `build_router()` function is public and returns a testable `Router`, but no test calls it. All endpoint behavior — including error codes, JSON structure, rate limiting, confirmation flow, and authentication — is completely untested at the HTTP layer.
- **Recommendation:** Add an integration test module in `server.rs` (or a separate `tests/` directory) using `axum::test_helpers` or `tower::ServiceExt` to exercise every endpoint with valid inputs, invalid inputs (bad base64, missing fields, wrong types), locked keystore, rate-limited state, and unauthorized confirm_token.
- **Verification:** `cargo test` must include tests named `test_sign_*`, `test_verify_*`, `test_pubkey_*`, `test_status_*`, `test_lock_*`, `test_unlock_*`, `test_confirm_pending_*`, `test_confirm_decide_*` covering both success and error paths. Each test must assert the exact HTTP status code and the `code` field in the JSON error response.

### Finding R2-A4-2
- **Section:** `signing_engine.rs` — Test Coverage
- **Severity:** IMPORTANT
- **Finding:** Existing tests cover: roundtrip sign/verify, tampered payload rejection, base64 export, fingerprint determinism, and key sizes. Missing tests: (a) different seeds produce different keypairs, (b) empty payload signing, (c) signature length matches `sig_len()`, (d) cross-keypair verification fails (sign with KP1, verify with KP2's public key), (e) malformed/truncated signature rejection, (f) malformed/wrong-length public key rejection. These are all machine-verifiable negative-path assertions.
- **Recommendation:** Add the six missing test cases listed above.
- **Verification:** `cargo test signing_engine` passes with all new tests asserting specific outcomes (not just `!verify()`).

### Finding R2-A4-3
- **Section:** `keystore.rs` — Credential Handling Test Coverage
- **Severity:** IMPORTANT
- **Finding:** Tests cover: create-and-open, wrong passphrase rejection, passphrase-too-short, load-public-key-only, KDF header params. Missing tests: (a) empty passphrase returns `EmptyPassphrase` error, (b) already-exists guard (`AlreadyExists` error), (c) corrupted file (truncated blob, wrong magic bytes) returns `InvalidFormat`, (d) unsupported KDF version returns `UnsupportedVersion`, (e) lock() zeroizes secret key (verify `secret_key()` returns `None` after lock), (f) Drop impl zeroizes (harder to test — at minimum verify `is_unlocked()` is false after drop), (g) Unicode passphrase (non-ASCII characters at 12+ chars) roundtrip, (h) keystore file permissions on Unix are 0o600 after create (testable via `std::fs::metadata`).
- **Recommendation:** Add the missing negative-path and edge-case tests.
- **Verification:** Each test asserts the exact `KeystoreError` variant returned.

### Finding R2-A4-4
- **Section:** `config.rs` — Token Generation and Config Load
- **Severity:** IMPORTANT
- **Finding:** No tests exist for `config.rs`. The following behaviors are untested: (a) `NinjaExecConfig::load()` returns defaults when no file exists, (b) `load()` returns defaults when file contains invalid JSON (silent fallback — itself potentially a bug), (c) `save_default()` does not overwrite an existing config, (d) `generate_confirm_token()` produces a non-empty URL-safe base64 string, (e) `generate_confirm_token()` is idempotent (second call returns same token), (f) config file created by `save_default()` is valid JSON that round-trips through `load()`.
- **Recommendation:** Add a test module for `config.rs` covering all six cases.
- **Verification:** `cargo test config` passes. Specifically, the idempotency test calls `generate_confirm_token()` twice and asserts equality.

### Finding R2-A4-5
- **Section:** `server.rs` — `check_confirm_token` Non-Constant-Time Comparison
- **Severity:** IMPORTANT
- **Finding:** At `server.rs` line 540, `provided != expected_token` uses standard string comparison, which is timing-sensitive. This was flagged as I3 in QC-R1. From a testability perspective: the timing side-channel is difficult to test in a unit test but can be verified by code inspection. The fix (XOR-accumulate or `subtle::ConstantTimeEq`) is machine-verifiable by asserting the absence of `!=` in the `check_confirm_token` function body via a grep-based CI check or by refactoring to use a dedicated constant-time comparison function that can be independently unit-tested.
- **Recommendation:** Replace `!=` with constant-time comparison. Add a test that at minimum verifies correct accept/reject behavior (valid token → Ok, invalid token → Err), even if timing is not directly testable.
- **Verification:** `grep -n '!=' server.rs | grep -c 'provided'` returns 0 after fix. Tests exist for both valid and invalid token paths.

### Finding R2-A4-6
- **Section:** `plenum-app.toml` — Hardcoded Upgrade Code
- **Severity:** IMPORTANT
- **Finding:** `upgrade_code = "A1B2C3D4-E5F6-7890-ABCD-EF1234567890"` is a hand-typed placeholder (flagged as I6 in QC-R1). From a testability perspective: deterministic product code derivation should be testable by verifying that `derive_upgrade_code(name, version_X)` == `derive_upgrade_code(name, version_X)` (idempotent) and `derive_upgrade_code(name, version_X)` != `derive_upgrade_code(name, version_Y)` (version-sensitive). No derivation function exists — there is no code to test. The placeholder UUID has no collision analysis, no derivation provenance, and no context-string sensitivity.
- **Recommendation:** Implement a sponge-based deterministic derivation function (TLSponge-385 with context string `"NinjaExec-UpgradeCode-v1"`) and add tests for idempotency and version sensitivity.
- **Verification:** A `test_upgrade_code_deterministic()` test exists that asserts same-input-same-output and different-input-different-output.

### Finding R2-A4-7
- **Section:** `audit.rs` — `hash_payload` Misleading Prefix
- **Severity:** MINOR
- **Finding:** `hash_payload()` at line 62 prefixes the output with `"tis27:"` but actually calls `ternary_math::sponge::derive_key` which is TLSponge-385, not TIS-27. This was flagged as M5 in QC-R1. A test (`test_hash_payload_deterministic`) asserts `h1.starts_with("tis27:")` — this test will break when the prefix is corrected to `"sponge385:"` or similar. The mislabeling is a spec-consistency issue: the audit log format documents a hash algorithm that does not match the actual implementation.
- **Recommendation:** Change prefix to `"sponge385:"` or `"tlsponge:"`. Update the test assertion accordingly.
- **Verification:** `hash_payload(b"test").starts_with("sponge385:")` passes. No log entries contain `"tis27:"`.

### Finding R2-A4-8
- **Section:** `confirm.rs` — Test Coverage Gaps
- **Severity:** MINOR
- **Finding:** Tests cover: requires_confirmation, auto_approve, headless auto-approves, interactive rejects, queue submit/approve/reject/timeout/pending. Missing: (a) `expire_stale()` removes entries older than timeout, (b) `pending_list()` returns correct count after multiple submits, (c) `check()` on a non-existent ID returns `Some(Rejected)`, (d) case-insensitivity of context matching (e.g., `"EXEC: cmd"` vs `"exec: cmd"`).
- **Recommendation:** Add the four missing test cases.
- **Verification:** `cargo test confirm` includes all new tests.

### Finding R2-A4-9
- **Section:** `cli.rs` — No Tests
- **Severity:** MINOR
- **Finding:** `cli.rs` has zero tests. The argument parser handles 11 subcommands with flags (`--port`, `--headless`, `--data-dir`, `--clipboard`, `--version`). None of the parsing logic is tested. Edge cases include: `--port` without a value, `--data-dir` without a value, unknown subcommand falls through to `Run`, `sign` subcommand reuses positional arg index 2 which may conflict with `--data-dir`.
- **Recommendation:** Add unit tests for `parse_args()` using `std::env::set_var` or by refactoring to accept `&[String]` instead of reading `std::env::args()` directly.
- **Verification:** Tests verify that `parse_args()` with `["ninja-exec", "init", "--data-dir", "/tmp"]` returns `Command::Init { data_dir: Some("/tmp") }`.

### Finding R2-A4-10
- **Section:** `main.rs` — Confirm Token Printed to stdout
- **Severity:** IMPORTANT
- **Finding:** At `main.rs` line 193, `println!("[NinjaExec] Token: {}", token)` prints the confirm token to stdout during `init`. This was flagged as I7 in QC-R1. From a testability perspective: a CI pipeline that captures stdout will log the token. The test criterion is machine-verifiable: `ninja-exec init 2>&1 | grep -c 'Token:'` should return 0 (token not in output). Currently it returns 1. The fix is to print only the storage location (`"Confirm token stored in ninja-exec.json"`) and never the token value.
- **Recommendation:** Remove the token value from stdout. Print only the file path.
- **Verification:** `ninja-exec init` stdout does not contain any base64-encoded string on the token line.

### Finding R2-A4-11
- **Section:** `plenum-app.toml` — Spec vs Implementation Consistency
- **Severity:** MINOR
- **Finding:** The manifest declares `binary = "ninja-exec.exe"` (Windows-only naming) and `icon = "assets/ninja-exec.ico"` / `tray_icon = "assets/ninja-exec-tray.ico"`. The `ninja-exec/assets/` directory should contain these icon files. The `architecture = ["aarch64", "x86_64"]` field implies cross-compilation support, but no CI workflow exists (I9 from QC-R1). The `configure_command = ""` is an empty string — if the installer framework treats empty string differently from absent field, this could cause unexpected behavior. Additionally, `Cargo.toml` specifies `[[bin]] name = "ninja-exec"` without `.exe` — the manifest and build config are inconsistent on binary naming.
- **Recommendation:** (a) Verify icon assets exist or mark as TODO. (b) Remove `.exe` from `binary` or make it conditional on platform. (c) Set `configure_command` to an explicit value or remove it.
- **Verification:** `ls ninja-exec/assets/` contains the referenced icon files. `binary` field matches the output of `cargo build --release`.

### Finding R2-A4-12
- **Section:** `Cargo.toml` — Dependencies Not Patch-Pinned
- **Severity:** IMPORTANT
- **Finding:** Dependencies use semver ranges: `tokio = "1"`, `serde = "1"`, `serde_json = "1"`, `chrono = "0.4"`, `getrandom = "0.2"`, `libc = "0.2.183"`. Only `axum = "0.7.4"`, `base64 = "0.22.1"`, and `tower-http = "0.5.2"` are pinned to patch versions. This was flagged as I8 in QC-R1. From a testability perspective: builds are not reproducible — `cargo build` on two different dates may resolve different patch versions, making test results non-deterministic across environments. A `Cargo.lock` file may mitigate this for application builds, but the `Cargo.toml` itself should pin for clarity and to prevent accidental updates.
- **Recommendation:** Pin all dependencies to exact patch versions (e.g., `tokio = "1.37.0"`, `serde = "1.0.203"`).
- **Verification:** `grep -c '"[0-9]"' Cargo.toml` returns 0 (no range-only version specs).

---

## Coverage Matrix

| Test Area | Source File | Status | Evidence |
|---|---|---|---|
| **HTTP: POST /sign (valid)** | `server.rs` | NOT COVERED | No integration tests for server |
| **HTTP: POST /sign (invalid base64)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /sign (invalid context)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /sign (locked keystore)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /sign (rate limited)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /sign (confirmation rejected)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /sign (confirmation timeout)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /verify (valid)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /verify (invalid base64)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: GET /pubkey** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: GET /pubkey (no key)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: GET /status** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /lock** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /unlock (valid)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /unlock (wrong passphrase)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /unlock (rate limited)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: GET /confirm/pending (valid token)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: GET /confirm/pending (invalid token)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /confirm/decide (approve)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /confirm/decide (reject)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /confirm/decide (invalid decision)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: POST /confirm/decide (not found)** | `server.rs` | NOT COVERED | No integration tests |
| **HTTP: CORS policy enforcement** | `server.rs` | NOT COVERED | No CORS restriction tests |
| **TL-DSA sign-verify roundtrip** | `signing_engine.rs` | COVERED | `test_roundtrip_sign_verify` |
| **TL-DSA tampered payload rejected** | `signing_engine.rs` | COVERED | `test_tampered_payload_rejected` |
| **TL-DSA pubkey base64 export** | `signing_engine.rs` | COVERED | `test_export_pubkey_b64` |
| **TL-DSA fingerprint determinism** | `signing_engine.rs` | COVERED | `test_fingerprint_deterministic` |
| **TL-DSA key sizes** | `signing_engine.rs` | COVERED | `test_key_sizes` |
| **TL-DSA cross-keypair rejection** | `signing_engine.rs` | NOT COVERED | No test |
| **TL-DSA Rep C address binding** | `signing_engine.rs` | NOT COVERED | No Rep C in codebase (C1, C2) |
| **TL-DSA malformed signature rejection** | `signing_engine.rs` | NOT COVERED | No test |
| **Keystore create and open** | `keystore.rs` | COVERED | `test_create_and_open` |
| **Keystore wrong passphrase** | `keystore.rs` | COVERED | `test_wrong_passphrase` |
| **Keystore passphrase too short** | `keystore.rs` | COVERED | `test_passphrase_too_short` |
| **Keystore load public key only** | `keystore.rs` | COVERED | `test_load_public_key_only` |
| **Keystore KDF header params** | `keystore.rs` | COVERED | `test_keystore_header_contains_kdf_params` |
| **Keystore empty passphrase** | `keystore.rs` | NOT COVERED | No test |
| **Keystore already exists** | `keystore.rs` | NOT COVERED | No test |
| **Keystore corrupted file** | `keystore.rs` | NOT COVERED | No test |
| **Keystore unsupported version** | `keystore.rs` | NOT COVERED | No test |
| **Keystore lock zeroizes** | `keystore.rs` | PARTIALLY COVERED | `test_create_and_open` calls lock/unlock but doesn't verify zeroization |
| **Keystore Unicode passphrase** | `keystore.rs` | NOT COVERED | No test |
| **Keystore file permissions (Unix)** | `keystore.rs` | NOT COVERED | No test |
| **Keystore constant-time tag comparison** | `keystore.rs` | PARTIALLY COVERED | Verified by code inspection (XOR-accumulate at lines 130–134), no dedicated test |
| **Config load defaults** | `config.rs` | NOT COVERED | No tests in config.rs |
| **Config load invalid JSON** | `config.rs` | NOT COVERED | No tests |
| **Config save_default idempotency** | `config.rs` | NOT COVERED | No tests |
| **Config generate_confirm_token** | `config.rs` | NOT COVERED | No tests |
| **Confirmation requires_confirmation** | `confirm.rs` | COVERED | `test_requires_confirmation` |
| **Confirmation auto_approve** | `confirm.rs` | COVERED | `test_auto_approve` |
| **Confirmation headless auto-approves** | `confirm.rs` | COVERED | `test_headless_auto_approves` |
| **Confirmation interactive rejects** | `confirm.rs` | COVERED | `test_interactive_rejects_without_gui` |
| **Confirmation auto-approve always passes** | `confirm.rs` | COVERED | `test_auto_approve_operations_always_pass` |
| **Confirmation queue submit/approve** | `confirm.rs` | COVERED | `test_confirmation_queue_submit_approve` |
| **Confirmation queue submit/reject** | `confirm.rs` | COVERED | `test_confirmation_queue_submit_reject` |
| **Confirmation queue timeout** | `confirm.rs` | COVERED | `test_confirmation_queue_timeout` |
| **Confirmation queue pending** | `confirm.rs` | COVERED | `test_confirmation_queue_pending` |
| **Confirmation expire_stale** | `confirm.rs` | NOT COVERED | No test |
| **Confirmation case-insensitivity** | `confirm.rs` | NOT COVERED | No test |
| **Audit append and format** | `audit.rs` | COVERED | `test_audit_append_and_format` |
| **Audit hash_payload deterministic** | `audit.rs` | COVERED | `test_hash_payload_deterministic` |
| **Audit append failure (read-only dir)** | `audit.rs` | NOT COVERED | No test (C4-related) |
| **Audit fail-closed on write error** | `server.rs` + `audit.rs` | NOT COVERED | append() returns () (C4) |
| **CLI argument parsing** | `cli.rs` | NOT COVERED | No tests |
| **Upgrade code derivation** | `plenum-app.toml` | NOT COVERED | Hardcoded placeholder (I6) |
| **Product code context-string sensitivity** | N/A | NOT COVERED | No derivation function exists |
| **Env var PLENUM_PASSPHRASE handling** | `main.rs` | NOT COVERED | No test; also not zeroized after use (I2) |
| **Env var NINJA_EXEC_PORT handling** | `cli.rs` | NOT COVERED | No test |
| **INVARIANT 8: No raw binary in sponge** | `keystore.rs` | PARTIALLY COVERED | KDF passes byte slices (passphrase + salt); these are binary, but the sponge `derive_key` accepts `&[u8]` context + material — needs audit of ternary_math internals |
| **INVARIANT 9: Rep C in all crypto ops** | All | NOT COVERED | No Rep C anywhere in codebase (C2) |

---

## Summary Statistics

| Category | COVERED | PARTIALLY COVERED | NOT COVERED |
|---|---|---|---|
| HTTP endpoints (8 endpoints × valid+invalid) | 0 | 0 | 22 |
| Signing engine | 5 | 0 | 3 |
| Keystore | 5 | 2 | 6 |
| Config | 0 | 0 | 4 |
| Confirmation | 9 | 0 | 2 |
| Audit | 2 | 0 | 2 |
| CLI | 0 | 0 | 2 |
| Invariant compliance | 0 | 1 | 2 |
| Upgrade/identity | 0 | 0 | 2 |
| **TOTAL** | **21** | **3** | **45** |

---

## Summary Verdict: **FAIL**

NinjaExec v1.0.0 fails the QC-R2 quality and evidence review. The core cryptographic primitives (TL-DSA-87 roundtrip, keystore encryption/decryption, confirmation logic) have solid unit test coverage, but the entire HTTP API layer — which is the product's primary interface — has zero integration tests. All four CRITICAL findings from QC-R1 (C1: no Rep C address binding, C2: no Rep C address anywhere, C3: CORS wildcard, C4: silent audit failures) remain unresolved and are independently confirmed by this review. The coverage matrix shows 45 of 69 test areas as NOT COVERED, with the most severe gaps in HTTP endpoint testing, credential handling edge cases, and INVARIANT 9 compliance. The hardcoded upgrade code (I6), confirm token printed to stdout (I7), non-constant-time token comparison (I3), and absence of any CI pipeline (I9) compound the risk. Until the four CRITICAL findings are resolved and HTTP endpoint integration tests are added, the product cannot proceed to release.
