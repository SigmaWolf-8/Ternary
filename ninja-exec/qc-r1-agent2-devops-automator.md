# QC-R1 — Agent 2: DevOps Automator Review

**Document:** NinjaExec — PlenumNET Local Signing Agent (Task #54)
**Files Reviewed:** `ninja-exec/Cargo.toml`, `ninja-exec/plenum-app.toml`, `ninja-exec/src/main.rs`, `ninja-exec/src/signing_engine.rs`, `ninja-exec/src/keystore.rs`, `ninja-exec/src/server.rs`, `ninja-exec/src/confirm.rs`, `ninja-exec/src/audit.rs`, `ninja-exec/src/config.rs`, `ninja-exec/src/cli.rs`, `Cargo.toml` (workspace root), `rust-toolchain.toml`
**Revision:** v1.0.0 (Cargo.toml, plenum-app.toml)
**Review Date:** 2026-03-28
**Reviewer:** DevOps Automator (Agent 2)
**Finding ID Convention:** R1-A2-{N}

---

### Finding R1-A2-1
- **Section:** `ninja-exec/src/audit.rs` lines 38–51, `server.rs` passim
- **Severity:** CRITICAL
- **Finding:** The `AuditLog::append()` method silently swallows every possible failure: JSON serialization errors (`if let Ok(json)`), directory creation errors (`let _ = fs::create_dir_all`), file open errors (`if let Ok(mut file)`), and write errors (`let _ = writeln!`). In `server.rs`, every audit call is additionally wrapped in `if let Ok(log) = state.audit_log.lock()` — a poisoned Mutex also produces no error. A signing operation can succeed and return a valid signature to the caller while the audit trail silently fails to record it. This is the canonical "silent failure producing an unsigned/untested artifact" — in this case, an unaudited signature.
- **Recommendation:** `append()` must return `Result<(), AuditError>`. All callers in `server.rs` must propagate this error and block the signing response if the audit write fails. At minimum, a failed audit write for a `sign` operation must prevent the signature from being returned.
- **Verification:** Introduce a test that makes the audit log directory read-only, issues a sign request, and confirms the request is rejected (not silently signed). Verify that no code path returns a signature without a confirmed audit write.

### Finding R1-A2-2
- **Section:** `ninja-exec/Cargo.toml` lines 22–31
- **Severity:** IMPORTANT
- **Finding:** Several dependencies are not pinned to a specific patch version: `tokio = "1"`, `serde = "1"`, `serde_json = "1"`, `getrandom = "0.2"`, `chrono = "0.4"`. The DevOps review protocol requires build tool dependencies to be version-pinned at the patch level for reproducible builds. While `Cargo.lock` provides reproducibility in practice, unpinned major/minor ranges in `Cargo.toml` allow silent dependency drift when the lockfile is regenerated.
- **Recommendation:** Pin all dependencies to patch versions: `tokio = "1.36.0"`, `serde = "1.0.197"`, `serde_json = "1.0.114"`, `getrandom = "0.2.12"`, `chrono = "0.4.35"` (or current locked versions). This ensures `cargo update` does not silently change behavior.
- **Verification:** Run `cargo tree -p ninja-exec` and confirm every direct dependency matches the pinned version in `Cargo.toml`. Diff `Cargo.lock` before and after `cargo update` — there should be zero changes to direct dependencies.

### Finding R1-A2-3
- **Section:** `ninja-exec/src/audit.rs` AuditEntry struct, `server.rs` audit calls
- **Severity:** CRITICAL
- **Finding:** INVARIANT 9 requires all audit records and provenance entries to reference nodes exclusively by their Rep C address. The `AuditEntry` struct identifies the signing node by HTTP `origin` header (a URL like `http://yoda.replit.app`), by hostname (in `export-operator`), or by nothing at all (`origin: None` for lock/unlock/startup). No field in `AuditEntry` contains a Rep C address. Log correlation across sources cannot use Rep C as the join key because the field does not exist.
- **Recommendation:** Add a `node_repc: String` field to `AuditEntry` that is populated from the operator's registered Rep C address (stored in or alongside the keystore). Every audit entry must include this field. Remove hostname-based identification from `export-operator` output or supplement it with the Rep C address as the primary identifier.
- **Verification:** Grep the `AuditEntry` struct for a `node_repc` or `rep_c` field. Verify every `AuditEntry` construction site populates it. Verify `export-operator` JSON output includes a `rep_c_address` field.

### Finding R1-A2-4
- **Section:** `ninja-exec/src/signing_engine.rs` lines 13–15
- **Severity:** CRITICAL
- **Finding:** INVARIANT 7 states: "The signer's Rep C address must be bound into the signature context string." The `sign()` function calls `tl_dsa::sign(secret_key, payload, VARIANT)` with no context string and no Rep C address binding. The signature is computed over the raw payload only. A signature produced by NinjaExec cannot be cryptographically bound to a specific operator identity as required by the framework.
- **Recommendation:** Modify `sign()` to accept a `context: &str` parameter that includes the signer's Rep C address. Construct a domain-separated signing input: `b"NinjaExec-SIGN:" || rep_c_address || b":" || context || b":" || payload`. Pass this composite message to `tl_dsa::sign()`. Update all callers accordingly.
- **Verification:** Read `signing_engine::sign()` and confirm the Rep C address is concatenated into the signed message. Write a test that signs with context, then verifies with the same context (pass) and a different context (fail).

### Finding R1-A2-5
- **Section:** `ninja-exec/src/server.rs` lines 615–618
- **Severity:** IMPORTANT
- **Finding:** The CORS layer is configured with `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. While the server binds to 127.0.0.1 only, any web page loaded in the operator's browser can issue cross-origin requests to `http://127.0.0.1:21027/sign` and obtain valid TL-DSA signatures. This is a browser-based confused deputy attack vector. Cross-reference to Security Engineer (Agent 1) for severity assessment.
- **Recommendation:** Restrict CORS to explicitly allowed origins. At minimum, allow only `http://127.0.0.1:*` and `http://localhost:*`. For YODA dashboard integration, add the specific YODA origin. Reject all other origins.
- **Verification:** Start the agent, issue a `curl` request with `Origin: https://evil.com` and confirm it receives a CORS rejection (no `Access-Control-Allow-Origin` header matching the evil origin).

### Finding R1-A2-6
- **Section:** `ninja-exec/src/confirm.rs` lines 159–173, `main.rs` lines 448–458
- **Severity:** IMPORTANT
- **Finding:** Headless mode (`--headless` flag) auto-approves ALL signing requests including destructive operations (`exec`, `model-swap`, `file-push`, `deploy`, `key-rotation`). Combined with Finding R1-A2-5 (open CORS), a browser tab on the operator's machine can silently trigger arbitrary signing operations with no confirmation gate. The audit log records this as `"confirmation": "auto"` — indistinguishable from a legitimately auto-approved read-only operation. Cross-reference to Security Engineer (Agent 1).
- **Recommendation:** In headless mode, either (a) restrict auto-approval to the `auto_approve` list only (verify, pubkey, status, tail, file-pull), requiring the confirm token for all other operations, or (b) require a separate `--headless-allow-destructive` flag with a documented risk acceptance. Distinguish headless auto-approval from configured auto-approval in audit entries.
- **Verification:** Start the agent in headless mode, send a sign request with context `exec: rm -rf /`, and confirm it is either rejected or requires the confirm token (not auto-approved).

### Finding R1-A2-7
- **Section:** No CI/CD pipeline file found for ninja-exec
- **Severity:** IMPORTANT
- **Finding:** No CI/CD pipeline definition (GitHub Actions workflow, Makefile, or build script) exists for ninja-exec. The review protocol requires verification that: (a) every pipeline step has defined failure handling, (b) all architectures are treated as an atomic release, (c) automated verification steps use exit codes, (d) expected CI duration and parallelism strategy are specified. None of these can be verified because no pipeline exists. The `plenum-app.toml` defines `architecture = ["aarch64", "x86_64"]` but no cross-compilation or matrix build is configured.
- **Recommendation:** Create a GitHub Actions workflow (`.github/workflows/ninja-exec-ci.yml`) that: (a) builds for both `aarch64` and `x86_64` targets, (b) runs `cargo test -p ninja-exec`, (c) runs `cargo clippy -p ninja-exec`, (d) fails the entire release if any architecture fails, (e) documents expected CI duration and parallelism strategy.
- **Verification:** Confirm `.github/workflows/ninja-exec-ci.yml` exists and contains a matrix build for both architectures. Run the workflow and confirm it produces exit code 0 on success, non-zero on any failure.

### Finding R1-A2-8
- **Section:** `ninja-exec/src/config.rs` lines 70–94, `main.rs` lines 190–193
- **Severity:** IMPORTANT
- **Finding:** During `init`, the confirm token is printed to stdout in cleartext: `println!("[NinjaExec] Token: {}", token)`. This token is the sole authentication mechanism for the `/confirm/decide` endpoint. If stdout is captured in logs (systemd journal, Windows Event Log, CI output), the token is exposed. Additionally, `generate_confirm_token()` writes the token to `ninja-exec.json` with default file permissions (no `chmod 600` on Unix). Cross-reference to Security Engineer (Agent 1) for credential exposure assessment.
- **Recommendation:** Do not print the confirm token to stdout. Instead, write it to a separate file (`confirm-token.txt`) with mode `0600` and instruct the operator to read it from there. Apply `chmod 600` to `ninja-exec.json` on Unix, similar to the keystore file.
- **Verification:** Run `ninja-exec init` and confirm the token does not appear in stdout/stderr. Verify `ninja-exec.json` file permissions are `0600` on Unix.

### Finding R1-A2-9
- **Section:** `ninja-exec/plenum-app.toml` line 7
- **Severity:** MINOR
- **Finding:** `binary = "ninja-exec.exe"` is Windows-specific, but the `architecture` field includes both `aarch64` and `x86_64` which could target Linux/macOS. No conditional binary naming or platform-specific configuration exists for non-Windows targets.
- **Recommendation:** Add a `[platforms]` or `[install.windows]`/`[install.linux]` section to `plenum-app.toml` that specifies the correct binary name per platform (`ninja-exec.exe` on Windows, `ninja-exec` on Unix).
- **Verification:** Inspect `plenum-app.toml` for platform-specific binary naming. Confirm the installer framework reads the correct binary name for each target OS.

### Finding R1-A2-10
- **Section:** `ninja-exec/src/signing_engine.rs` line 27, `audit.rs` line 60, `keystore.rs` line 71
- **Severity:** MINOR
- **Finding:** Payload hashing in audit and fingerprint generation both use `ternary_math::sponge::derive_key()` which is the TLSponge-385 kernel sponge. This is correct per framework conventions (not SHA-256/BLAKE3). The hash output is labeled `tis27:` in `audit.rs` line 62, but the actual primitive used is `sponge::derive_key` which is TLSponge-385, not TIS-27. The label is misleading and could cause operators to apply the wrong verification procedure.
- **Recommendation:** Change the hash prefix from `tis27:` to `tlsponge385:` or `sponge:` to accurately reflect the primitive used, unless `sponge::derive_key` internally delegates to TIS-27 (in which case, document this).
- **Verification:** Trace `ternary_math::sponge::derive_key` to confirm which sponge variant it uses. Verify the audit hash prefix matches the actual primitive.

### Finding R1-A2-11
- **Section:** `ninja-exec/src/main.rs` lines 305, 334, 427
- **Severity:** MINOR
- **Finding:** The passphrase is accepted via the `PLENUM_PASSPHRASE` environment variable for automation. Environment variables are visible in `/proc/<pid>/environ` on Linux and via `Get-Process` on Windows. While documented, this creates a silent credential exposure in CI/CD pipelines and process listings.
- **Recommendation:** Document that `PLENUM_PASSPHRASE` should only be used in ephemeral CI environments. Consider supporting passphrase input via a file descriptor (e.g., `--passphrase-fd 3`) or a named pipe as a more secure alternative.
- **Verification:** Confirm documentation warns against using `PLENUM_PASSPHRASE` in persistent/production environments. Optionally verify `--passphrase-fd` support exists.

### Finding R1-A2-12
- **Section:** `ninja-exec/src/server.rs`, `main.rs`
- **Severity:** IMPORTANT
- **Finding:** No deployment test specification exists. The review protocol requires: (a) every test step to be automatable with machine-verifiable exit codes, (b) minimum supported OS versions to be defined, (c) expected CI duration for full-matrix testing to be specified, (d) parallelism strategy to be documented, (e) product-specific validation requiring network services to have a mock mode. None of these are present. The `verify` CLI command uses exit code 0/1 (good), but no integration test harness exercises the HTTP API end-to-end.
- **Recommendation:** Create an integration test module (`tests/integration.rs` or `ninja-exec/tests/`) that starts the server, exercises all endpoints, and verifies exit codes. Document minimum OS versions in `plenum-app.toml`. Specify expected CI duration.
- **Verification:** Confirm an integration test file exists and can be run with `cargo test -p ninja-exec --test integration`. Confirm `plenum-app.toml` includes minimum OS version fields.

---

## Summary Verdict: **FAIL**

Three CRITICAL findings block implementation:

1. **R1-A2-1 (Audit silent failure):** A signing operation can return a valid signature while the audit trail silently fails to record it. This is the exact failure mode the DevOps review protocol is designed to catch — a silent failure producing an unaudited cryptographic artifact. The audit system must be fail-closed.

2. **R1-A2-3 (No Rep C in audit entries):** INVARIANT 9 is violated — audit records identify nodes by HTTP origin URLs and hostnames instead of Rep C addresses. Log correlation across the PlenumNET ecosystem is impossible without Rep C as the join key.

3. **R1-A2-4 (No Rep C context binding in signatures):** INVARIANT 7 is violated — TL-DSA signatures are computed over raw payloads without binding the signer's Rep C address into the signature context. Signatures cannot be cryptographically attributed to a specific operator.

Additionally, five IMPORTANT findings (R1-A2-2, R1-A2-5, R1-A2-6, R1-A2-7, R1-A2-8, R1-A2-12) require resolution before first product release. The open CORS policy combined with headless auto-approval creates a browser-based confused deputy attack that could silently produce unauthorized signatures — a pipeline failure mode where the artifact (signature) is valid but unauthorized.

No release of NinjaExec should proceed until the three CRITICAL findings are resolved and the CI/CD pipeline is defined.
