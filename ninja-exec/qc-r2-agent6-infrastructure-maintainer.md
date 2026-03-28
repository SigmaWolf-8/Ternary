# QC-R2 Agent 6 — Infrastructure Maintainer Review

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Task:** #54
**Review Date:** 2026-03-28
**Protocol:** QC-R2 (Round 2 — Quality & Operations)
**Reviewer:** Agent 6 — Infrastructure Maintainer
**Finding ID Convention:** R2-A6-{N}

---

## Round 1 Response

### C1 — No Rep C Address Binding in TL-DSA Signatures (INVARIANT 9)

**Response: AGREE**

From an operator's perspective, this is a deployment blocker. If signatures are not bound to the operator's Rep C address, there is no way to correlate a signature back to a specific node in the PlenumNET topology. An operator deploying NinjaExec in a multi-node environment cannot distinguish which agent produced which signature. This also makes audit trail correlation across log sources impossible — a direct violation of the operator's ability to investigate incidents. The R1 recommendation (extend `sign()`/`verify()` to accept a Rep C address, construct domain-separated message) is correct and operationally necessary.

### C2 — No Rep C Address Exists Anywhere in Codebase (INVARIANT 9)

**Response: AGREE**

Confirmed by inspection: no Rep C address is stored, derived, or referenced in any source file. The `export-operator` command (main.rs lines 252-257) identifies nodes as `operator@{hostname}` using `COMPUTERNAME` or `HOSTNAME` environment variables. This is explicitly prohibited by INVARIANT 9. Audit entries reference HTTP `origin` URLs (e.g., `http://yoda.replit.app`) instead of Rep C addresses. From an infrastructure perspective, hostname-based identification is fragile (hostnames change, collide across domains, and are spoofable), and it breaks the zero-sentinel forgery detection property that Rep C provides. The `init` command must derive or accept a Rep C 54-trit address and persist it in the keystore.

### C3 — CORS Wildcard Origin Creates Signature Oracle

**Response: AGREE**

`server.rs` line 616: `allow_origin(Any)` is a deployment-blocking configuration for any machine where a browser is present. In enterprise environments, operators browse the web while NinjaExec runs in the tray. Any malicious or compromised website can issue `POST /sign` requests to `127.0.0.1:21027`. Combined with headless mode auto-approval (confirm.rs line 169), this is a remotely exploitable signing oracle triggered by a single browser tab. The R1 recommendation (configurable origin allowlist in `ninja-exec.json`, default deny-all) is correct. I would add: the `plenum-app.toml` first_run sequence should generate the allowlist with a sensible default (e.g., `["http://localhost:*"]`) and the operator should be instructed to add the YODA dashboard origin.

### C4 — Audit Log Silently Swallows All Write Failures

**Response: AGREE**

`audit.rs` lines 38-51: Every error path uses `let _ =` to discard results — directory creation, file open, and write operations all fail silently. From an operator's perspective, this is the worst kind of failure: the system appears healthy, signatures are produced, but the audit trail has gaps. In a compliance environment (FINRA, SOX, etc.), an unaudited signing operation is worse than a failed signing operation. The R1 recommendation (return `Result`, fail-closed if audit write fails) is correct. I would additionally recommend: on audit write failure, emit a structured error to stderr so that Windows Event Log forwarding or systemd journal captures the failure even if the JSONL file is inaccessible.

---

## Findings

### Finding R2-A6-1
- **Section:** `plenum-app.toml` [first_run], `main.rs` lines 157-211
- **Severity:** IMPORTANT
- **Finding:** The `init` flow has no pre-flight validation. Before creating a keystore, the installer should verify: (1) the data directory is writable, (2) sufficient disk space exists, (3) no antivirus/EDR file-system hook will quarantine the keystore file (common with `.keystore` extensions), (4) the binary is not running from a read-only mount. Currently, `fs::create_dir_all` is the first I/O operation and its failure message (`I/O error: Permission denied`) does not tell the operator *why* it failed or *what to do*. In enterprise environments with redirected AppData (e.g., `%APPDATA%` pointing to a DFS share), the keystore write may silently fail or produce latency-related errors.
- **Recommendation:** Add a pre-flight check function that validates directory writability (create+delete a temp file), reports the resolved data directory path, and provides actionable guidance: "Cannot write to {path}. Verify NTFS permissions, check if AppData is redirected via Group Policy, or specify an alternate path with --data-dir."
- **Verification:** Run `ninja-exec init` on a machine with read-only AppData redirection; confirm the error message names the path and suggests `--data-dir`.

### Finding R2-A6-2
- **Section:** `plenum-app.toml` [uninstall], `keystore.rs`
- **Severity:** IMPORTANT
- **Finding:** The `preserve_message` in `plenum-app.toml` references `%APPDATA%\NinjaExec` but does not expand the variable at display time. Operators on systems with non-standard AppData locations will see a misleading path. Additionally, `preserve_data = true` is the only uninstall behavior — there is no option for a clean uninstall that removes key material. An operator decommissioning a machine (e.g., returning a laptop) needs a way to securely wipe key material. There is no `ninja-exec wipe` or `ninja-exec destroy` command.
- **Recommendation:** (1) Expand `%APPDATA%` to the actual path in the uninstall dialog. (2) Add a `ninja-exec destroy` command that securely overwrites the keystore file before deletion (write random bytes, fsync, delete). (3) Add a `preserve_data = "prompt"` option that asks the operator during uninstall.
- **Verification:** Run uninstaller on a machine with redirected AppData; confirm the displayed path is the actual resolved path. Run `ninja-exec destroy` and verify the keystore file is overwritten before deletion.

### Finding R2-A6-3
- **Section:** `config.rs` lines 46-56
- **Severity:** IMPORTANT
- **Finding:** `NinjaExecConfig::load()` silently falls back to defaults if the config file exists but contains invalid JSON. An operator who makes a typo in `ninja-exec.json` (e.g., trailing comma) will get default settings with no warning. The agent will start with default CORS (wide open), default rate limits, and no confirm token — silently degrading security posture.
- **Recommendation:** If the config file exists but fails to parse, print a clear error to stderr: "Config file {path} exists but contains invalid JSON: {parse_error}. Fix the config file or delete it to use defaults." Exit with a non-zero code rather than silently falling back.
- **Verification:** Create a `ninja-exec.json` with a syntax error; run `ninja-exec run`; confirm it exits with an error message naming the file and the parse error.

### Finding R2-A6-4
- **Section:** `config.rs` lines 58-68, `main.rs` line 190
- **Severity:** MINOR
- **Finding:** `NinjaExecConfig::save_default()` and `generate_confirm_token()` both silently discard write errors (`let _ = std::fs::write`). If the config file cannot be written (permissions, disk full), the confirm token is generated in memory but never persisted. The operator sees "Confirm token generated" but the token is lost on next restart.
- **Recommendation:** Return `Result` from both functions. If the config file cannot be written, display: "WARNING: Could not save config to {path}: {error}. The confirm token will be lost on restart."
- **Verification:** Set the data directory to read-only after keystore creation; run `ninja-exec init`; confirm a warning is displayed about config write failure.

### Finding R2-A6-5
- **Section:** `main.rs` lines 191-193
- **Severity:** CRITICAL
- **Finding:** The confirm token is printed to stdout during `init`: `println!("[NinjaExec] Token: {}", token)`. This token grants the ability to approve or reject all signing requests. In CI/CD pipelines, stdout is captured to build logs. In enterprise environments with centralized log collection, the token leaks to log aggregators. This was already flagged as I7 in R1 but remains unresolved and is more severe from an infrastructure perspective: any system that captures process stdout now has signing approval authority.
- **Recommendation:** Do not print the token to stdout. Print only the storage location: "Confirm token stored in {path}/ninja-exec.json (line: confirm_token). Use this token to configure the tray UI or external confirmation tool." Set file permissions on `ninja-exec.json` to 0600 (Unix) or equivalent ACL (Windows).
- **Verification:** Run `ninja-exec init` and capture stdout; confirm the token value does not appear. Verify `ninja-exec.json` has restrictive permissions.

### Finding R2-A6-6
- **Section:** `main.rs` lines 414-505, `cli.rs`
- **Severity:** IMPORTANT
- **Finding:** There is no key backup or key export mechanism for disaster recovery. If the machine's disk fails, the operator's signing identity is permanently lost. There is no `ninja-exec backup` command that exports an encrypted keystore copy. There is no documented key rotation procedure. The `plenum-app.toml` does not reference key backup in first_run or document it anywhere. Day-two operations are undocumented: no troubleshooting guide, no FAQ, no known-issues list.
- **Recommendation:** (1) Add `ninja-exec backup <output-path>` that copies the encrypted keystore file to a specified location with verification. (2) Document the key rotation procedure (init new keystore, re-register pubkey with administrator, retire old key). (3) Create a deployment guide covering: prerequisites, installation steps, first-run, day-two health checks (`ninja-exec status`), key backup schedule, and recovery procedure. (4) Add a `ninja-exec doctor` command that checks: keystore exists, config is valid JSON, port is available, data directory is writable.
- **Verification:** Run `ninja-exec backup /tmp/backup.keystore`; verify the backup can be restored by copying it back. Run `ninja-exec doctor`; verify it reports all checks.

### Finding R2-A6-7
- **Section:** `audit.rs` lines 12-25, `server.rs` passim
- **Severity:** CRITICAL
- **Finding:** Audit entries use `origin: Option<String>` populated from HTTP `Origin`/`Referer` headers (server.rs lines 139-145). These are browser-controlled, spoofable headers — not reliable provenance. Per INVARIANT 9, all audit records must reference nodes by Rep C address. Currently, no Rep C address appears in any audit entry. The `origin` field contains URLs like `http://yoda.replit.app` or `None`, which are useless for cross-referencing with TDNS topology records.
- **Recommendation:** Add a `node_address` field to `AuditEntry` populated with the agent's own Rep C address (once C2 is resolved). Retain `origin` as supplementary diagnostic data but do not use it as the primary node identifier. Every audit entry must include the signer's Rep C address.
- **Verification:** After C2 fix, inspect `ninja-exec-audit.jsonl`; confirm every entry contains a `node_address` field with a valid 54-trit Rep C address in dot-separated format.

### Finding R2-A6-8
- **Section:** `plenum-app.toml` [app], `Cargo.toml`
- **Severity:** IMPORTANT
- **Finding:** The `upgrade_code` in `plenum-app.toml` is `A1B2C3D4-E5F6-7890-ABCD-EF1234567890` — a hand-typed placeholder GUID. This was flagged as I6 in R1. From an infrastructure perspective, this is a deployment hazard: if two different PlenumNET products accidentally use the same placeholder GUID (and this pattern is common in copy-paste installer templates), Windows Installer will treat them as the same product. Upgrades will collide, uninstalls will remove the wrong product, and MSI repair will break.
- **Recommendation:** Generate a deterministic upgrade code from the product name using a namespace UUID v5: `UUID_v5(PLENUMNET_NAMESPACE, "NinjaExec")`. Document the generation method so it is reproducible and collision-free.
- **Verification:** Verify the upgrade_code is a valid UUID that is unique across all `plenum-app.toml` files in the repository. Run `grep -r "A1B2C3D4" .` and confirm zero matches after the fix.

### Finding R2-A6-9
- **Section:** `keystore.rs` lines 324-335
- **Severity:** MINOR
- **Finding:** `default_data_dir()` on Windows uses `APPDATA` environment variable, falling back to `HOME`, falling back to `.ninja-exec` (relative path). In enterprise environments with AppData Virtualization (Windows 10/11 with UWP compatibility), the `APPDATA` variable may point to a virtualized location. The fallback chain does not check `LOCALAPPDATA` (preferred for per-machine data that should not roam) or `ProgramData` (preferred for service-level data). For a tray agent that auto-starts, roaming AppData may cause issues if the user logs in to multiple machines — the keystore would roam via folder redirection but the service binding is machine-local.
- **Recommendation:** Use `LOCALAPPDATA` as the primary Windows location (non-roaming, per-user). Document the data directory selection logic in the deployment guide. Add a note that `--data-dir` overrides all defaults.
- **Verification:** On a domain-joined machine with roaming profiles, verify `ninja-exec init` creates the keystore under `LOCALAPPDATA`, not `APPDATA`.

### Finding R2-A6-10
- **Section:** `server.rs` lines 633-643
- **Severity:** MINOR
- **Finding:** The startup message is printed to stdout via `println!`. If NinjaExec is run as a Windows service or via a service wrapper (e.g., NSSM, WinSW), stdout may not be captured. The operator has no way to verify the agent started successfully other than polling `/status`. There is no Windows Event Log integration, no structured startup event, and no health check endpoint that returns machine-parseable diagnostic information beyond basic status.
- **Recommendation:** (1) Write startup events to the audit log (already partially done, but the bind address and port should be included). (2) Add a `/health` endpoint that returns HTTP 200 with `{"healthy": true}` for load balancer and monitoring tool integration. (3) On Windows, consider writing a startup event to the Application event log via `ReportEvent` or equivalent.
- **Verification:** Start NinjaExec as a Windows service; verify the audit log contains a startup entry with the bind address and port. Poll `/health` and confirm 200 response.

### Finding R2-A6-11
- **Section:** `Cargo.toml` lines 19-31
- **Severity:** IMPORTANT
- **Finding:** Dependencies are not pinned to patch versions: `tokio = "1"`, `serde = "1"`, `serde_json = "1"`, `chrono = "0.4"`, `getrandom = "0.2"`. This was flagged as I8 in R1. From an infrastructure perspective, non-pinned dependencies mean that two builds on different dates may produce different binaries. For a security-critical signing agent, build reproducibility is essential — an operator must be able to verify that the binary they received matches the audited source code. Without pinned dependencies, `cargo build` on two different CI machines may pull different patch versions, producing different hashes.
- **Recommendation:** Pin all dependencies to exact patch versions (e.g., `tokio = "1.36.0"`, `serde = "1.0.197"`). Add a `Cargo.lock` to version control. Document the expected binary hash for each release.
- **Verification:** Build on two clean machines from the same commit; verify the output binaries have identical hashes.

### Finding R2-A6-12
- **Section:** `main.rs` lines 427-431, `cli.rs`
- **Severity:** MINOR
- **Finding:** The `PLENUM_PASSPHRASE` environment variable is read for silent installation but is never zeroed from the process environment after use. On Linux, `/proc/<pid>/environ` exposes environment variables for the lifetime of the process. On Windows, `Get-Process | Select-Object -ExpandProperty StartInfo` can reveal them. This was flagged as I2 in R1.
- **Recommendation:** After reading `PLENUM_PASSPHRASE`, call `std::env::remove_var("PLENUM_PASSPHRASE")` immediately. Document that `PLENUM_PASSPHRASE` is intended for CI/CD silent provisioning only and must not be set persistently.
- **Verification:** Run `ninja-exec run` with `PLENUM_PASSPHRASE` set; after startup, inspect `/proc/<pid>/environ` (Linux) or process environment (Windows); confirm the variable is no longer present.

---

## Operator Readiness Checklist

| # | Question | Answer | Notes |
|---|----------|--------|-------|
| 1 | Is there a step-by-step deployment guide? | **NO** | No deployment guide, troubleshooting doc, or FAQ exists. |
| 2 | Can I install silently via SCCM/Intune? | **PARTIALLY** | `PLENUM_PASSPHRASE` env var enables silent init, but no MSI/MSIX package exists yet. `plenum-app.toml` defines the spec but no installer binary is produced. |
| 3 | What are the minimum OS requirements? | **NO** | Not documented. Binary targets are listed (aarch64, x86_64) but minimum Windows version, .NET requirements, and Linux distro support are unstated. |
| 4 | Does the installer work on a locked-down corporate PC? | **NO** | No testing against Group Policy restrictions documented. No EDR compatibility testing. AppData redirection not handled robustly (uses `APPDATA` instead of `LOCALAPPDATA`). |
| 5 | What happens if init fails mid-way? | **PARTIALLY** | Keystore uses atomic rename (write to `.tmp`, rename), so partial writes are avoided. But if `ninja-exec.json` write fails after keystore creation, the system is in an inconsistent state (keystore exists, no config). |
| 6 | How do I verify the agent is running? | **YES** | `ninja-exec status` or `GET /status` on port 21027. |
| 7 | How do I back up the signing key? | **NO** | No backup command exists. Operator must manually copy `ninja-exec.keystore`. No documentation. |
| 8 | How do I rotate the signing key? | **NO** | No key rotation command or procedure documented. |
| 9 | What happens on uninstall? | **PARTIALLY** | `preserve_data = true` retains keystore and audit log. But no option for secure wipe, and the preserved path display may not expand `%APPDATA%` correctly. |
| 10 | Are error messages actionable? | **PARTIALLY** | Keystore errors are clear (wrong passphrase, too short, already exists). But config parse failures are silent, audit write failures are silent, and pre-flight I/O errors lack remediation guidance. |
| 11 | Are audit records compliant with INVARIANT 9? | **NO** | No Rep C address in any audit entry. Origin field uses HTTP headers. Hostname used in export-operator. |
| 12 | Is the binary code-signed? | **NO** | No signing certificate referenced in `plenum-app.toml` or build config. EDR/SmartScreen will quarantine unsigned binaries. |
| 13 | Is there a CI/CD pipeline for builds? | **NO** | No GitHub Actions workflow. No reproducible build process. |
| 14 | Can I run this as a Windows service? | **PARTIALLY** | The binary can be wrapped with NSSM/WinSW, but there is no native service registration, no service recovery configuration, and stdout logging is not captured. |
| 15 | Is the CORS policy safe for production? | **NO** | `allow_origin(Any)` — any website can call the signing API. |
| 16 | Does the confirm token survive restarts? | **PARTIALLY** | Token is saved to `ninja-exec.json`, but write failures are silently ignored. |

---

## Summary Verdict: **FAIL**

NinjaExec has strong cryptographic foundations — TL-DSA-87 correctly selected, all crypto delegated to `ternary_math`, localhost-only binding, constant-time tag comparison in the keystore, and volatile key zeroization on drop. The CLI is well-structured with sensible subcommands, the confirmation queue design is sound, and the rate limiter provides basic abuse prevention.

However, from an operator deployment perspective, NinjaExec is not ready for release. Two findings are CRITICAL blockers: (1) the complete absence of Rep C addresses in audit records and signing contexts violates INVARIANT 9 and makes the agent operationally opaque — an operator cannot correlate NinjaExec activity with the PlenumNET topology; (2) the confirm token is printed to stdout, leaking signing approval authority to any log aggregation system. Beyond the CRITICAL items, the CORS wildcard (C3, still unresolved), silent config/audit failures, missing deployment documentation, absent key backup/rotation procedures, and lack of enterprise environment validation (AppData redirection, EDR, code signing) collectively mean that an operator cannot deploy this product into a production enterprise environment with confidence. The product needs the C1-C4 fixes from R1, the token exposure fix, a deployment guide, and at minimum a pre-flight validation check before it can pass with conditions.
