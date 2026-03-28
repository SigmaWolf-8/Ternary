# QC-R2 — Task #54 Quality Control Review (Round 2)
# Agent 5: Senior Developer

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Review Date:** 2026-03-28
**Protocol:** QC-R2 (Round 2 — Quality & Implementability)
**Reviewer:** Agent 5 — Senior Developer (Engineering)
**Finding ID Convention:** R2-A5-{N}

---

## Round 1 Response

### C1 — No Rep C Address Binding in TL-DSA Signatures (INVARIANT 9)

**Response: AGREE**

Confirmed by source inspection. `signing_engine::sign()` at line 13 passes `(secret_key, payload, VARIANT)` directly to `tl_dsa::sign` with no domain-separated context string and no Rep C address binding. The function signature offers no parameter for a Rep C address or context string. Every signature produced by NinjaExec is identity-unbound. This is a clear INVARIANT 9 violation. The recommended fix — extending `sign()`/`verify()` to accept a Rep C address and constructing a domain-separated message `"NinjaExec-SIGN-v1.0" ‖ rep_c_address ‖ context ‖ payload` — is architecturally sound and implementable. The `context` field already exists in `SignRequest` and flows through the handler; it just needs to be incorporated into the signed message rather than only recorded in the audit log.

### C2 — No Rep C Address Exists Anywhere in Codebase (INVARIANT 9)

**Response: AGREE**

Confirmed. The `Keystore` struct stores only `public_key: Option<Vec<u8>>` and `secret_key: Option<Vec<u8>>`. No Rep C address field exists in the keystore, the config, the `AppState`, the `AuditEntry`, or the `export-operator` output. The `export-operator` command at `main.rs` lines 252–264 identifies the operator as `operator@{hostname}` — using `COMPUTERNAME`/`HOSTNAME` environment variables — which is explicitly prohibited by INVARIANT 9. The resolution requires: (a) deriving a Rep C 54-trit address from the public key during `init` (or accepting one as input), (b) persisting it in the keystore file format (requires a format version bump or an auxiliary file), (c) binding it into all signing contexts, KDF domain separators, audit entries, and the `export-operator` JSON output.

### C3 — CORS Wildcard Origin Creates Signature Oracle

**Response: AGREE**

Confirmed at `server.rs` line 615–618: `CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)`. This is maximally permissive. Any website loaded in the operator's browser can issue cross-origin POST requests to `127.0.0.1:21027/sign`. Combined with headless mode auto-approval, this constitutes a remotely exploitable signature oracle. The fix — a configurable origin allowlist in `ninja-exec.json` with a deny-all default — is straightforward to implement using `tower_http::cors::AllowOrigin::list()`. I note the `NinjaExecConfig` struct currently has no `allowed_origins` field; one must be added.

### C4 — Audit Log Silently Swallows All Write Failures

**Response: AGREE**

Confirmed at `audit.rs` lines 38–51. The `append()` method uses `if let Ok(...)` chains that silently discard: (a) JSON serialization errors, (b) `create_dir_all` errors, (c) file open errors, and (d) `writeln!` errors. Furthermore, every call site in `server.rs` wraps audit logging in `if let Ok(log) = state.audit_log.lock()` which silently drops the audit entry if the mutex is poisoned. The result is that a signing operation can succeed and return a valid signature while the audit trail silently fails — producing an unaudited cryptographic artifact. The fix — making `append()` return `Result` and failing the sign operation if audit write fails — is correct. This is a fail-closed requirement: no signature without a successful audit record.

---

## New Findings

### Finding R2-A5-1
- **Section:** `config.rs` — NinjaExecConfig schema
- **Severity:** IMPORTANT
- **Finding:** The `NinjaExecConfig` struct is missing fields required by the C1/C2/C3 remediations: (a) `allowed_origins: Vec<String>` for CORS restriction (C3), (b) `rep_c_address: Option<String>` or equivalent for identity binding (C2), (c) `headless_allow: Vec<String>` for restricted headless mode (I4). The current struct has only `port`, `rate_limit_per_minute`, `confirmation`, and `confirm_token`. Without these fields, the critical fixes cannot be implemented as configuration-driven changes.
- **Recommendation:** Add all three fields to `NinjaExecConfig` with appropriate defaults: `allowed_origins` defaults to empty (deny all cross-origin), `rep_c_address` defaults to `None` (derived during init), `headless_allow` defaults to `["verify", "pubkey", "status"]`.
- **Verification:** Deserialize a `ninja-exec.json` containing all new fields; verify defaults are applied when fields are absent.

### Finding R2-A5-2
- **Section:** `keystore.rs` — Keystore file format
- **Severity:** IMPORTANT
- **Finding:** The keystore binary format (`NJXK0002`) is fixed-length at `HEADER_LEN` bytes with no extensibility mechanism. Adding a Rep C address field (C2 remediation) requires either: (a) bumping the magic/version and creating a new fixed-length format, (b) storing the Rep C address in an auxiliary file, or (c) appending variable-length data after the fixed header with a length prefix. Option (a) is cleanest but breaks existing keystores without a migration path. The `open()` method at line 239 performs an exact length check (`blob.len() != HEADER_LEN`) which will reject any extended format.
- **Recommendation:** Define `NJXK0003` format that appends a 54-byte Rep C address (27 trits × 2 bits/trit packed, or 54 ASCII digits) after the existing fields. Implement a migration path: if `NJXK0002` is detected, derive the Rep C address from the stored public key and rewrite the file as `NJXK0003`. Update `HEADER_LEN` calculation and `open()` length check accordingly.
- **Verification:** Create a v2 keystore, verify it is automatically migrated to v3 on first open, and verify the Rep C address matches the public key derivation.

### Finding R2-A5-3
- **Section:** `signing_engine.rs` — fingerprint context string
- **Severity:** MINOR
- **Finding:** The `fingerprint()` function uses `derive_key(b"NinjaExec-FP", ...)` which is listed as an unregistered context string in I11. Additionally, the fingerprint is 16 bytes (128 bits) displayed as hex with colons. This is adequate for display purposes but the context string should be registered in the canonical context string registry.
- **Recommendation:** Register `NinjaExec-FP` in the canonical context string registry. No code change needed.
- **Verification:** Check the context string registry contains `NinjaExec-FP` with its documented purpose.

### Finding R2-A5-4
- **Section:** `audit.rs` lines 59–63 — hash_payload prefix
- **Severity:** MINOR
- **Finding:** `hash_payload()` produces output prefixed with `tis27:` but the actual hash function used is `ternary_math::sponge::derive_key` with context `b"NinjaExec-AUDIT-HASH"`, which is TLSponge-385, not TIS-27. This is misleading (also flagged as M5 in R1). The prefix should accurately reflect the algorithm used.
- **Recommendation:** Change prefix from `"tis27:"` to `"tlsponge385:"` or simply `"sponge:"` to accurately reflect the underlying primitive.
- **Verification:** Grep for `tis27:` in all source files; confirm no remaining mislabeled prefixes.

### Finding R2-A5-5
- **Section:** `keystore.rs` — Bespoke authenticated encryption
- **Severity:** IMPORTANT
- **Finding:** The keystore uses a hand-rolled encrypt-then-MAC scheme: XOR keystream from `derive_key(b"NinjaExec-KS-STREAM", ...)` followed by a separate MAC tag from `derive_key(b"NinjaExec-KS-TAG", ...)`. This is I5 from R1. While the construction appears sound (encrypt-then-MAC with separate key material derivation, constant-time tag verification), it is not the canonical T-AE-MAC construction. T-AE-MAC provides IND-CPA + INT-CTXT with a formally analyzed construction. The bespoke scheme has not been formally analyzed.
- **Recommendation:** If T-AE-MAC is available in `ternary_math`, replace the bespoke encrypt/decrypt functions with T-AE-MAC calls. If T-AE-MAC is not yet exported from the crate, document the bespoke scheme as a temporary measure with a tracking issue for migration. **Flag for Security Engineer (Agent 1) review** — this is a cryptographic substitution decision that crosses the security boundary.
- **Verification:** After migration, verify keystore round-trip: create → lock → unlock → sign produces the same results. Verify old keystores are either migrated or rejected with a clear error.

### Finding R2-A5-6
- **Section:** `plenum-app.toml` — upgrade_code
- **Severity:** IMPORTANT
- **Finding:** The `upgrade_code` field is `"A1B2C3D4-E5F6-7890-ABCD-EF1234567890"` — a visually obvious placeholder. This was flagged as I6 in R1. For WiX MSI installers, the UpgradeCode is the persistent identifier that Windows uses to detect existing installations. A hand-typed placeholder: (a) risks collision with other software, (b) cannot be deterministically regenerated if lost, (c) looks unprofessional in installer logs. The spec should define a deterministic derivation: `TLSponge-385("NinjaExec-UPGRADE-CODE", publisher_name ‖ app_name, 16)` formatted as a UUID v4-like string.
- **Recommendation:** Derive the upgrade code deterministically using `sponge::derive_key(b"NinjaExec-UPGRADE-CODE", b"Capomastro Holdings Ltd.NinjaExec", 16)` and format as a GUID. Document the derivation so it can be reproduced.
- **Verification:** Run the derivation, verify it produces a stable GUID, update `plenum-app.toml`.

### Finding R2-A5-7
- **Section:** `server.rs` lines 533–548 — check_confirm_token
- **Severity:** IMPORTANT
- **Finding:** The `check_confirm_token` function uses standard string comparison (`provided != expected_token`) which is not constant-time. This was flagged as I3 in R1. An attacker with local network access could potentially use timing side-channels to extract the confirm token byte-by-byte. The keystore tag comparison correctly uses XOR-accumulate (line 130–133), but the server token check does not.
- **Recommendation:** Replace `provided != expected_token` with a constant-time comparison. Either use the `subtle` crate's `ConstantTimeEq` or implement the XOR-accumulate pattern already used in `keystore.rs`.
- **Verification:** Verify that `check_confirm_token` uses constant-time comparison by code inspection.

### Finding R2-A5-8
- **Section:** `confirm.rs` lines 159–173 — headless mode logic
- **Severity:** IMPORTANT
- **Finding:** The `evaluate_confirmation` function at line 168–169 auto-approves ALL operations in headless mode, including destructive operations like `exec`, `deploy`, `key-rotation`, and `config-update`. This was flagged as I4 in R1. The function checks `if headless { return ConfirmationResult::AutoApproved; }` with no filtering. Combined with the CORS wildcard (C3), this means any website can trigger arbitrary signed operations on a headless NinjaExec instance.
- **Recommendation:** Add a `headless_allow` list to `NinjaExecConfig` (see R2-A5-1). In `evaluate_confirmation`, only auto-approve operations that appear in `headless_allow`. Default the list to read-only operations: `["verify", "pubkey", "status", "tail", "file-pull"]`.
- **Verification:** Start NinjaExec in headless mode, send a `sign` request with context `exec`, verify it is NOT auto-approved. Send with context `verify`, verify it IS auto-approved.

### Finding R2-A5-9
- **Section:** `cli.rs` — Argument parsing robustness
- **Severity:** MINOR
- **Finding:** The argument parser silently ignores unknown flags (line 105–107: `_ => { i += 1; }`). The `sign` subcommand reuses `args.get(2)` which may collide with flags parsed by the main loop. For example, `ninja-exec sign --port 8080 myfile.txt` would set `file = "--port"` rather than `"myfile.txt"`. Positional and flag arguments are intermixed without clear precedence rules.
- **Recommendation:** Parse positional arguments for `sign` and `verify` subcommands after flag parsing, or use a proper argument parser like `clap` (already common in the workspace). At minimum, document the expected argument order.
- **Verification:** Test `ninja-exec sign --data-dir /tmp myfile.txt` and verify `file` is correctly set to `myfile.txt`.

### Finding R2-A5-10
- **Section:** `plenum-app.toml` — configure_command and port discovery
- **Severity:** IMPORTANT
- **Finding:** The `configure_command` field is empty (`""`). The `status_port` is hardcoded to `21027`. There is no mechanism for: (a) the tray icon to discover which port NinjaExec is actually running on if the user overrides via `--port` or config, (b) the installer to verify that port 21027 is available before starting the service, (c) the tray to launch a configuration UI. The `first_run` actions reference `ninja-exec init` and `ninja-exec export-operator` but there is no action to start the agent itself after initialization. The tray agent presumably relies on `autostart = true` but the startup mechanism (Windows Task Scheduler, Registry Run key, or Windows Service) is not specified.
- **Recommendation:** (a) Define a port discovery mechanism — either a well-known file (`data_directory/ninja-exec.port`) written on startup, or always use the config file's port value. (b) Specify the autostart mechanism explicitly in `plenum-app.toml` or in supplementary documentation. (c) Add a `first_run` action to start the agent after init. (d) Define `configure_command` or document that configuration is done via `ninja-exec.json` editing.
- **Verification:** After install and first-run, verify the tray icon can reach the agent's `/status` endpoint. Verify the agent starts on system boot.

### Finding R2-A5-11
- **Section:** `config.rs` lines 46–56 — Config load silently falls back
- **Severity:** MINOR
- **Finding:** `NinjaExecConfig::load()` silently returns defaults if: (a) the config file doesn't exist, (b) the file can't be read, or (c) JSON parsing fails. Case (c) is problematic — a malformed config file should produce a clear error, not silently apply defaults. An operator who misconfigures their allowed origins would get the default (currently no restriction), silently negating their security intent.
- **Recommendation:** Log a warning (at minimum) when JSON parsing fails. Consider making parse failures fatal with a clear error message: `"Failed to parse ninja-exec.json: {error}. Fix the file or delete it to use defaults."`.
- **Verification:** Create a malformed `ninja-exec.json`, start NinjaExec, verify a warning or error is emitted.

### Finding R2-A5-12
- **Section:** `server.rs` — Missing API response types for error cases
- **Severity:** MINOR
- **Finding:** Error responses are constructed inline using `serde_json::json!()` macros throughout the handlers rather than using the defined `ErrorResponse` struct (lines 63–67). The `ErrorResponse` struct is defined but never used (it has `#[allow(dead_code)]`). This means error response shapes are not enforced by the type system and could drift between endpoints.
- **Recommendation:** Use the `ErrorResponse` struct for all error responses, or remove it and document the inline JSON shape as the canonical error format.
- **Verification:** Grep for `serde_json::json!` in error paths; verify all error responses have consistent `code` and `error` fields.

### Finding R2-A5-13
- **Section:** `Cargo.toml` — Dependency version pinning
- **Severity:** IMPORTANT
- **Finding:** Dependencies use semver ranges: `tokio = "1"`, `serde = "1"`, `serde_json = "1"`, `chrono = "0.4"`, `getrandom = "0.2"`, `libc = "0.2.183"`. Only `axum`, `base64`, and `tower-http` are pinned to patch versions. This was flagged as I8 in R1. For a security-critical signing agent, reproducible builds are essential. A minor version bump in any dependency could introduce behavioral changes that affect cryptographic operations or security properties.
- **Recommendation:** Pin all dependencies to exact patch versions: `tokio = "=1.36.0"`, `serde = "=1.0.197"`, etc. Use `cargo update --precise` to manage updates intentionally.
- **Verification:** Run `cargo build` twice on different dates; verify identical binary hashes (modulo timestamps in metadata).

---

## Feasibility Risk Table

| Task | Risk | Justification |
|------|------|---------------|
| C1: Rep C address binding in signatures | **MEDIUM** | Requires changing `signing_engine::sign/verify` signatures, updating all call sites (server.rs, main.rs), and defining domain-separated message format. Straightforward but touches every signing path. |
| C2: Rep C address provisioning + storage | **HIGH** | Requires keystore format change (v2→v3 migration), Rep C derivation function (must verify `ternary_math` exports a suitable function), integration into `AppState`, audit entries, and export-operator output. Most invasive change. |
| C3: CORS origin allowlist | **LOW** | Add `allowed_origins` to config, replace `Any` with `AllowOrigin::list()` in `build_router`. Well-documented tower-http API. |
| C4: Audit fail-closed | **MEDIUM** | Change `append()` to return `Result`, propagate errors through all call sites in server.rs. Every handler's audit block must be refactored from fire-and-forget to fail-closed. Risk of introducing new failure modes. |
| I1: KDF iteration increase | **LOW** | Change constant and benchmark. No structural changes. |
| I3: Constant-time token comparison | **LOW** | Replace `!=` with XOR-accumulate or `subtle::ConstantTimeEq`. ~5 lines. |
| I4: Headless mode restriction | **LOW** | Add `headless_allow` config field, add one check in `evaluate_confirmation`. |
| I5: T-AE-MAC migration for keystore | **HIGH** | Depends on T-AE-MAC availability in `ternary_math` public API. If not exported, requires upstream work. Also requires keystore format migration. |
| I6: Deterministic upgrade code | **LOW** | One-time derivation, update `plenum-app.toml`. |
| I7: Token not printed to stdout | **LOW** | Remove `println!` of token in `init` command. Print storage location instead. |
| I8: Dependency pinning | **LOW** | Run `cargo update`, record exact versions. No code changes. |
| I11: Context string registration | **LOW** | Documentation task only. |
| I12: Operation context in signatures | **LOW** | Extension of C1 fix — context string already flows through. |
| Port discovery mechanism | **MEDIUM** | Requires coordination between tray agent and NinjaExec process. No existing port-file protocol defined. |
| Autostart mechanism specification | **MEDIUM** | Platform-specific (Windows Task Scheduler vs Registry Run key vs Service). Requires testing on target platforms. |

---

## Summary Verdict: **FAIL**

NinjaExec demonstrates solid engineering fundamentals: correct TL-DSA-87 selection, all cryptography delegated to `ternary_math` (no external crypto crates), localhost-only binding, constant-time keystore tag verification, volatile key zeroization, well-structured module separation, and a functional confirmation queue system. The codebase is clean, readable, and well-tested with unit tests covering all critical paths.

However, the four CRITICAL findings from QC-R1 are all confirmed valid and remain unresolved. The most architecturally significant — C2 (no Rep C address anywhere in the codebase) — requires a keystore format migration, touches every module, and depends on `ternary_math` exporting a Rep C derivation function. C1 (identity-unbound signatures) renders every signature produced by NinjaExec non-compliant with INVARIANT 9. C3 (CORS wildcard) combined with headless auto-approval (I4) creates a remotely exploitable signature oracle that is the most urgent security fix. C4 (silent audit failures) violates the fail-closed principle for a security audit trail.

Additionally, the bespoke authenticated encryption in the keystore (R2-A5-5 / I5) and the non-constant-time token comparison (R2-A5-7 / I3) are IMPORTANT findings that must be resolved before release. The `plenum-app.toml` has gaps in port discovery, autostart specification, and a placeholder upgrade code that block installer integration.

The implementation path is feasible — no circular dependencies exist, the module structure supports the required changes, and the highest-risk task (C2) can be decomposed into incremental steps (derive address → store address → bind address → migrate format). However, the volume of blocking issues means NinjaExec cannot pass QC-R2 in its current state.

**Conditions for PASS:** Resolve C1, C2, C3, C4, plus I3, I4, and R2-A5-1 (config schema expansion). Remaining IMPORTANT findings may be deferred to v1.1 with documented risk acceptance from the Security Engineer.
