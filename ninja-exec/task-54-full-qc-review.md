# Task #54 — Full Quality Control Review
# NinjaExec — PlenumNET Local Signing Agent v1.0.0

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Task:** #54
**Review Date:** 2026-03-28
**Rounds Completed:** QC-R1, QC-R2, QC-R3, QC-R3 Recursive
**Total Agents:** 12 (3 per round × 4 rounds)

---

## Executive Summary

NinjaExec has been reviewed across four quality control rounds by 12 independent YODA agents. The product has strong cryptographic foundations — TL-DSA-87 correctly selected, all crypto delegated to `ternary_math`, localhost-only binding, constant-time tag comparison, volatile key zeroization — but systemic issues block release.

| Round | Verdict | Key Blocker |
|-------|---------|-------------|
| QC-R1 (Technical Verification) | **FAIL** | 4 CRITICALs: INVARIANT 9 violations (no Rep C), CORS wildcard, silent audit failures |
| QC-R2 (Quality & Completeness) | **FAIL** | Confirmed R1 CRITICALs + 3 new (C5-C7): no integration tests, no Rep C in audit, token leak |
| QC-R3 (Fit, Finish & Market Readiness) | **FAIL** | 2 new CRITICALs: no color tokens, unreadable tray icon. Brand Readiness 4.33/10 |
| QC-R3 Recursive (Review Quality) | **PASS WITH CONDITIONS** | Review documents are strong; 10 bookkeeping corrections needed |

**Overall Status: FAIL — 9 CRITICAL findings unresolved (7 from R1/R2, 2 from R3)**

---

## All Agents

| Agent | Round | Role | Verdict | Score |
|-------|-------|------|---------|-------|
| 1 | R1 | Security Engineer | FAIL | — |
| 2 | R1 | DevOps Automator | FAIL | — |
| 3 | R1 | PlenumNET Integration Specialist | FAIL | — |
| 4 | R2 | Evidence Collector / QA Lead | FAIL | — |
| 5 | R2 | Senior Developer | FAIL | — |
| 6 | R2 | Infrastructure Maintainer | FAIL | — |
| 7 | R3 | Brand Guardian | FAIL | 3/10 |
| 8 | R3 | UX Designer | PASS w/ CONDITIONS | 4/10 |
| 9 | R3 | Content Creator | PASS w/ CONDITIONS | 6/10 |
| 7R | R3 Recursive | Brand Guardian | PASS w/ CONDITIONS | 7/10 |
| 8R | R3 Recursive | UX Designer | PASS w/ CONDITIONS | 7/10 |
| 9R | R3 Recursive | Content Creator | PASS w/ CONDITIONS | 8/10 |

---

## Master Finding Count

| Severity | R1 | R2 (new) | R3 (non-deferred) | R3 (deferred) | Total |
|----------|----|---------|--------------------|---------------|-------|
| CRITICAL | 4 | 3 | 2 | — | **9** |
| IMPORTANT | 12 | 8 | 17 | — | **37** |
| MINOR | 6 | 2 | 24 | — | **32** |
| DEFERRED | — | — | — | 14 | **14** |
| **Total** | **22** | **13** | **43** | **14** | **92** |

> R3 DEFERRED findings (14) are blocked by open R1/R2 CRITICALs and cannot be evaluated until those are resolved. They do not affect verdicts.

---

# PART I — CRITICAL FINDINGS (9)

All 9 CRITICALs must be resolved before release. They are ordered by resolution priority.

## C1 — No Rep C Address Binding in TL-DSA Signatures
- **Round:** R1 | **Agents:** A1, A2, A3 (unanimous)
- **Location:** `signing_engine.rs` lines 13–18
- **Issue:** `tl_dsa::sign(secret_key, payload, VARIANT)` passes raw payload with no Rep C address in the context string. INVARIANT 9 requires the signer's Rep C address bound into every signature. Signatures are identity-unbound and could be replayed across operators.
- **Agent 1 (Security Engineer):** "TL-DSA `sign()` and `verify()` calls do not bind a Rep C address into the signing context string. INVARIANT 9 requires that all TL-DSA signing contexts include the signer's Rep C address. The current implementation passes the raw payload directly to `tl_dsa::sign(secret_key, payload, VARIANT)` with no context string containing a Rep C address. This means signatures are not bound to any node identity and could be replayed across nodes."
- **Agent 2 (DevOps):** "INVARIANT 7 states: 'The signer's Rep C address must be bound into the signature context string.' The `sign()` function calls `tl_dsa::sign(secret_key, payload, VARIANT)` with no context string and no Rep C address binding. The signature is computed over the raw payload only."
- **Agent 3 (Integration):** "TL-DSA signing and verification do not bind the signer's Rep C address into the signature context string. INVARIANT 9 requires: 'The signer's Rep C address must be bound into the signature context string. Signature verification must check the signer's public key against a registered Rep C address.'"
- **R2 Confirmation:** All three R2 agents (A4, A5, A6) responded AGREE. Agent 4: "This is a testable, machine-verifiable gap: a test that signs the same payload from two different keystores will produce signatures that are interchangeable (cross-operator replay)."
- **Resolution:** Extend `sign()`/`verify()` to accept Rep C address. Construct domain-separated message: `"NinjaExec-SIGN-v1.0" ‖ rep_c_address ‖ context ‖ payload`.

## C2 — No Rep C Address Exists Anywhere in Codebase
- **Round:** R1 | **Agents:** A1, A2, A3 (unanimous)
- **Location:** Entire codebase
- **Issue:** No Rep C address stored, derived, or referenced. Keystore stores only raw keypair. `export-operator` uses `operator@{hostname}` — prohibited by INVARIANT 9. Audit entries use HTTP origin URLs.
- **Agent 1 (Security Engineer):** "No Rep C address is defined, stored, or used anywhere in the NinjaExec codebase. INVARIANT 9 requires that all cryptographic operations binding node identity use Rep C (54-trit, binary-encoded) addressing exclusively. NinjaExec does not associate the signing key with a Rep C address, does not include a Rep C address in TL-DSA signing context, does not include a Rep C address in TLSponge-385 KDF domain separation for the keystore, and does not include a Rep C address in fingerprint computation."
- **Agent 3 (Integration):** "No Rep C address is stored, derived, or used anywhere in NinjaExec. The `export-operator` command exports a hostname-based name (`operator@{hostname}`) which explicitly violates INVARIANT 9: 'No cryptographic operation may use hostname, IP address, Windows SID, or any non-Rep-C identifier as an identity binding.'"
- **R2 Confirmation:** All three R2 agents responded AGREE. Agent 5: "The resolution requires: (a) deriving a Rep C 54-trit address from the public key during `init`, (b) persisting it in the keystore file format (requires a format version bump), (c) binding it into all signing contexts, KDF domain separators, audit entries, and the `export-operator` JSON output."
- **Resolution:** During `init`, derive or accept Rep C 54-trit address. Store in keystore (v3 format bump to `NJXK0003`). Bind into all contexts, KDF domain separators, audit entries, operator export.

## C3 — CORS Wildcard Origin Creates Signature Oracle
- **Round:** R1 | **Agents:** A1, A2, A3 (unanimous)
- **Location:** `server.rs` lines 615–618
- **Issue:** `allow_origin(Any)` permits any website to send cross-origin requests to `127.0.0.1:21027/sign`. Combined with headless auto-approval, this is a remotely exploitable signature oracle.
- **Agent 1 (Security Engineer):** "The CORS layer is configured with `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. This means any website in the user's browser can send cross-origin requests to the localhost signing agent. A malicious or compromised web page could issue signing requests to `http://127.0.0.1:21027/sign` and, if the keystore is unlocked and running in headless mode, obtain valid TL-DSA signatures over attacker-controlled payloads without any user interaction."
- **Agent 6 (Infrastructure):** "`server.rs` line 616: `allow_origin(Any)` is a deployment-blocking configuration for any machine where a browser is present. In enterprise environments, operators browse the web while NinjaExec runs in the tray. Any malicious or compromised website can issue `POST /sign` requests to `127.0.0.1:21027`. Combined with headless mode auto-approval, this is a remotely exploitable signing oracle triggered by a single browser tab."
- **R2 Confirmation:** All three R2 agents responded AGREE. Agent 4 confirmed: "This is a triple-`Any` CORS policy. The `NinjaExecConfig` struct has no `allowed_origins` field — no mechanism exists to restrict origins even if desired."
- **Resolution:** Replace with configurable origin allowlist (default: deny all). Add `allowed_origins` to config.

## C4 — Audit Log Silently Swallows All Write Failures
- **Round:** R1 | **Agents:** A2, A1
- **Location:** `audit.rs` lines 38–51
- **Issue:** `AuditLog::append()` silently ignores all write errors. Signing can succeed while audit trail silently fails — "silent failure producing an unaudited artifact."
- **Agent 2 (DevOps):** "The `AuditLog::append()` method silently swallows every possible failure: JSON serialization errors, directory creation errors, file open errors, and write errors. A signing operation can succeed and return a valid signature to the caller while the audit trail silently fails to record it. This is the canonical 'silent failure producing an unsigned/untested artifact' — in this case, an unaudited signature."
- **Agent 6 (Infrastructure):** "From an operator's perspective, this is the worst kind of failure: the system appears healthy, signatures are produced, but the audit trail has gaps. In a compliance environment (FINRA, SOX, etc.), an unaudited signing operation is worse than a failed signing operation."
- **R2 Confirmation:** All three R2 agents responded AGREE. Agent 5: "The result is that a signing operation can succeed and return a valid signature while the audit trail silently fails — producing an unaudited cryptographic artifact."
- **Resolution:** `append()` must return `Result`. Sign operations must fail-closed if audit write fails.

## C5 — No Integration Tests for Any HTTP Endpoint
- **Round:** R2 | **Agents:** A4, A5, A6 (unanimous)
- **Location:** `server.rs` (entire module)
- **Issue:** No integration test exercises any HTTP endpoint end-to-end. All existing tests are unit tests.
- **Agent 4 (Evidence Collector):** "No integration test exists for any of the 8 HTTP endpoints (`/sign`, `/verify`, `/pubkey`, `/status`, `/lock`, `/unlock`, `/confirm/pending`, `/confirm/decide`). The `server.rs` file contains zero `#[cfg(test)]` blocks. The `build_router()` function is public and returns a testable `Router`, but no test calls it."
- **Coverage Impact:** 22 of 69 HTTP test scenarios are NOT COVERED (see Coverage Matrix in Part VIII).
- **Resolution:** Add integration test suite covering `/sign`, `/verify`, `/status`, `/lock`, `/unlock`, `/confirm/*`, `/pubkey`, `/export-operator`.

## C6 — Audit Entries Lack Rep C Addresses
- **Round:** R2 | **Agents:** A4, A5, A6
- **Location:** `audit.rs`, `server.rs`
- **Issue:** Audit entries record HTTP origin URLs instead of Rep C addresses. Multi-operator environments cannot correlate audit entries to operator identity.
- **Agent 2 (DevOps):** "INVARIANT 9 requires all audit records and provenance entries to reference nodes exclusively by their Rep C address. The `AuditEntry` struct identifies the signing node by HTTP `origin` header (a URL like `http://yoda.replit.app`), by hostname, or by nothing at all."
- **Agent 6 (Infrastructure):** "Audit entries use `origin: Option<String>` populated from HTTP `Origin`/`Referer` headers. These are browser-controlled, spoofable headers — not reliable provenance."
- **Resolution:** Include signer's Rep C address in every audit entry. Depends on C2 resolution.

## C7 — Confirm Token Printed to Stdout
- **Round:** R2 | **Agents:** A4, A5, A6
- **Location:** `main.rs` line 193
- **Issue:** During `init`, the confirm token is printed to stdout via `println!`. Exposed in shell history, CI logs, and screen captures. Token protects confirmation API.
- **Agent 6 (Infrastructure):** "The confirm token is printed to stdout during `init`. This token grants the ability to approve or reject all signing requests. In CI/CD pipelines, stdout is captured to build logs. In enterprise environments with centralized log collection, the token leaks to log aggregators. Any system that captures process stdout now has signing approval authority."
- **Resolution:** Print only storage location (`ninja-exec.json`). Set file permissions to 0600.

## C8 — No Color Token System (R3)
- **Round:** R3 | **Agent:** A7 (Brand Guardian)
- **Location:** `plenum-app.toml` — no color system section
- **Issue:** No palette token list, color system, or design token specification exists. SVG uses 5 hardcoded hex values with no named tokens. No status indicator colors defined.
- **Agent 7:** "No palette token list, color system, or design token specification exists anywhere in the NinjaExec specification or source documents. The SVG source uses hardcoded hex values — `#181411`, `#0F0C0A`, `#272220`, `#F0EDE8`, `#4A9EF5` — with no mapping to named palette tokens. Per the Brand Guardian review scope, if the spec does not contain a palette token list, this is CRITICAL."
- **Resolution:** Define formal color token table. Include tokens for tray icon states, installer dialogs, status indicators. Document WCAG contrast ratios.

## C9 — No Icon Size Specifications (R3)
- **Round:** R3 | **Agent:** A7 (Brand Guardian)
- **Location:** `plenum-app.toml` lines 8, 21
- **Issue:** Three ICO files referenced but no icon size requirements, size transition boundaries, or tray icon status rendering method specified. Key-with-P glyph is **UNREADABLE** at 16×16 (tray icon's primary rendering size).
- **Agent 7:** "The spec references three ICO files but specifies no icon size requirements, no size transition boundaries, no minimum usable pixel count, and no tray icon status rendering method. The current SVG design has fine strokes (stroke-width 4-6 at 256x256 scale) that will alias badly at 16x16."
- **Resolution:** Document embedded sizes. Define 32×32 simplified transition boundary. Create simplified 16×16 glyph. Specify tray status rendering (overlay dot, swap, or tint).

---

# PART II — IMPORTANT FINDINGS (37)

## From R1 (12 findings)

### I1 — Weak KDF Iterations (4096)
- **Source:** A1 (Finding 3), cross-referenced A3 (Finding 3)
- **Location:** `keystore.rs` lines 67–83
- **Detail:** KDF uses `KDF_ITERATIONS = 4096` rounds of TLSponge-385 `derive_key`. Modern passphrase KDFs target ≥100ms wall-clock time. At 4096 rounds, the KDF may complete in under 10ms, making offline brute-force attacks significantly faster. No memory-hard component, making GPU attacks feasible.
- **Resolution:** Increase to ≥100,000 iterations or add configurable cost parameter targeting ≥100ms. Alternatively, integrate Argon2id as outer KDF.

### I2 — Passphrase via Env Var Exposure
- **Source:** A1 (Finding 5), A2 (Finding 11)
- **Location:** `server.rs`, `main.rs` lines 167, 305, 334, 427
- **Detail:** Passphrase can be supplied via `PLENUM_PASSPHRASE` env var. On Linux, `/proc/<pid>/environ` visible. On Windows, environment variables accessible to same-session processes. Environment variable never zeroed after use.
- **Resolution:** Document as CI-only. Zero env var after reading. Consider `--passphrase-fd` alternative.

### I3 — Non-Constant-Time Token Comparison
- **Source:** A1 (Finding 6), confirmed R2: A4 (Finding 5), A5 (Finding 7)
- **Location:** `server.rs` line 540
- **Detail:** `check_confirm_token` uses `!=` (standard string comparison), not constant-time. Leaks token length and content via timing side-channel. The keystore already correctly uses XOR-accumulate for tag comparison.
- **Resolution:** Replace with XOR-accumulate or `subtle::ConstantTimeEq`.

### I4 — Headless Mode Auto-Approves Destructive Ops
- **Source:** A1 (Finding 7), A2 (Finding 6), confirmed R2: A5 (Finding 8)
- **Location:** `confirm.rs` lines 159–173
- **Detail:** When `headless = true`, ALL signing requests auto-approved regardless of context, including `exec`, `deploy`, `key-rotation`, `config-update`. Combined with C3, any web page can silently trigger arbitrary signing.
- **Resolution:** Add `headless_allow` configuration list defaulting to safe subset: `["verify", "pubkey", "status", "tail", "file-pull"]`.

### I5 — Bespoke Authenticated Encryption (not T-AE-MAC)
- **Source:** A1 (Finding 8), A3 (Finding 4), confirmed R2: A5 (Finding 5)
- **Location:** `keystore.rs` lines 86–118
- **Detail:** Custom XOR-stream + sponge-tag construction rather than canonical T-AE-MAC. Construction follows correct encrypt-then-MAC pattern but has not been formally analyzed.
- **Resolution:** Replace with T-AE-MAC if available in `ternary_math`, or document risk acceptance with migration path.

### I6 — Placeholder Upgrade Code
- **Source:** A1 (Finding 9), confirmed R2: A4 (Finding 6), A5 (Finding 6), A6 (Finding 8)
- **Location:** `plenum-app.toml` line 10
- **Detail:** `upgrade_code = "A1B2C3D4-E5F6-7890-ABCD-EF1234567890"` — hand-typed placeholder. Not deterministically derived. Collision risk with other products.
- **Resolution:** Derive deterministically: `TLSponge-385("NinjaExec-UPGRADE-CODE", "Capomastro Holdings Ltd.NinjaExec", 16)` → format as GUID.

### I7 — Confirm Token Printed to Stdout
- **Source:** A1 (Finding 10), A2 (Finding 8) — **Elevated to C7 in R2**
- **Location:** `main.rs` line 193
- **Note:** This finding was elevated from IMPORTANT (R1) to CRITICAL (C7) in R2 after three R2 agents unanimously agreed on criticality.

### I8 — Dependencies Not Patch-Pinned
- **Source:** A2 (Finding 2), confirmed R2: A4 (Finding 12), A5 (Finding 13), A6 (Finding 11)
- **Location:** `Cargo.toml` lines 22–31
- **Detail:** `tokio = "1"`, `serde = "1"`, `serde_json = "1"`, `getrandom = "0.2"`, `chrono = "0.4"` — semver ranges, not pinned to patch.
- **Resolution:** Pin all to patch versions: `tokio = "1.36.0"`, etc.

### I9 — No CI/CD Pipeline
- **Source:** A2 (Finding 7)
- **Location:** No CI/CD file exists
- **Detail:** No GitHub Actions workflow. `plenum-app.toml` defines `architecture = ["aarch64", "x86_64"]` but no cross-compilation or matrix build configured.
- **Resolution:** Create `.github/workflows/ninja-exec-ci.yml` with matrix build for both architectures.

### I10 — No Deployment Test Specification
- **Source:** A2 (Finding 12)
- **Location:** `server.rs`, `main.rs`
- **Detail:** No integration test harness exercises the HTTP API end-to-end. No minimum supported OS versions defined. No expected CI duration specified.
- **Resolution:** Create integration test module. Document minimum OS versions.

### I11 — Unregistered Context Strings
- **Source:** A3 (Findings 2, 3, 4, 5)
- **Location:** `signing_engine.rs`, `keystore.rs`, `audit.rs`
- **Detail:** 5 context strings not in canonical registry: `"NinjaExec-FP"`, `"NinjaExec-KDF-v2"`, `"NinjaExec-KS-STREAM"`, `"NinjaExec-KS-TAG"`, `"NinjaExec-AUDIT-HASH"`.
- **Resolution:** Register all 5 in canonical context string registry with documented purposes.

### I12 — No Operation Context in Signed Message
- **Source:** A3 (Finding 8)
- **Location:** `server.rs` lines 147–323, `signing_engine.rs` line 14
- **Detail:** Signatures don't bind operation type — `context` validated against `VALID_CONTEXTS` but never incorporated into signed message. Cross-context replay possible.
- **Resolution:** Incorporate context into signed message: `context_bytes ‖ 0x00 ‖ payload`.

## From R2 (8 new findings)

### R2-I1 — Keystore Migration Path Unspecified
- **Source:** A5 (Finding 2)
- **Location:** `keystore.rs`
- **Detail:** Keystore format `NJXK0002` is fixed-length with no extensibility. Adding Rep C address (C2 fix) requires format bump. `open()` performs exact length check that will reject extended format.
- **Resolution:** Define `NJXK0003` format. Implement migration: detect v2, derive Rep C from public key, rewrite as v3.

### R2-I2 — No Graceful Shutdown
- **Source:** A6
- **Location:** `main.rs`, `server.rs`
- **Detail:** No signal handler. In-flight signatures may be lost on SIGTERM/SIGINT. No cleanup of pending confirmations.

### R2-I3 — Log Rotation Not Specified
- **Source:** A6
- **Location:** `audit.rs`
- **Detail:** Audit JSONL file grows unbounded. No rotation mechanism, no size limit, no archival strategy.

### R2-I4 — No Health Check Endpoint
- **Source:** A6 (Finding 10)
- **Location:** `server.rs`
- **Detail:** No `/health` endpoint for load balancer/monitoring integration. No structured startup event.
- **Resolution:** Add `/health` returning `{"healthy": true}`. Write startup events to audit log.

### R2-I5 — Config Validation on Load
- **Source:** A5 (Finding 11), A6 (Finding 3)
- **Location:** `config.rs` lines 46–56
- **Detail:** Malformed JSON silently falls back to defaults. Operator who misconfigures allowed_origins gets default (wide open), silently negating security intent.
- **Resolution:** If config exists but fails to parse, exit with clear error message.

### R2-I6 — APPDATA vs LOCALAPPDATA
- **Source:** A6 (Finding 9)
- **Location:** `keystore.rs` lines 324–335
- **Detail:** Uses roaming `APPDATA` on Windows. For machine-local signing agent, `LOCALAPPDATA` is more appropriate. Roaming AppData causes issues with folder redirection on domain-joined machines.

### R2-I7 — No Rate-Limit Per-Endpoint Granularity
- **Source:** A5, A1 (Finding 12)
- **Location:** `server.rs` lines 37–59
- **Detail:** Shared rate limiter across all endpoints. `/status` competes with `/sign`. An attacker could exhaust limit with `/sign`, blocking legitimate `/unlock`.

### R2-I8 — Placeholder Upgrade Code (R1 I6 confirmed)
- **Source:** A4 (Finding 6), A5 (Finding 6), A6 (Finding 8) — unanimous confirmation of R1 I6

## From R3 (17 non-deferred findings)

### R3-I1 — No Tray Icon State Rendering
- **Source:** A7 (Finding 3)
- **Detail:** No visual map for running/locked/error states. No overlay dot, icon swap, tint, or ring specified.

### R3-I2 — No Typography Specification
- **Source:** A7 (Finding 5)
- **Detail:** No font specs for any UI surface — installer, passphrase entry, error messages, tray panel.

### R3-I3 — No Launcher Panel Specification
- **Source:** A7 (Finding 6)
- **Detail:** Declared as `tray_agent` but no panel design: no state display, fingerprint, activity count, lock/unlock action.

### R3-I4 — Passphrase Prompt Lacks Strength Feedback
- **Source:** A8 (Finding 1)
- **Detail:** No strength indicator, no inline validation, no character counter against 72-bit entropy floor.

### R3-I5 — No Progress Indicator During Init
- **Source:** A8 (Finding 2)
- **Detail:** Key generation, keystore encryption — no spinner, progress bar, or phase messaging. Silence feels like hang.

### R3-I6 — No Cancel/Rollback During Init
- **Source:** A8 (Finding 3)
- **Detail:** Ctrl+C during key generation may leave partial state. Config file write not atomic.

### R3-I7 — Passphrase Echo/Mismatch UX
- **Source:** A8 (Finding 4)
- **Detail:** No echo suppression on Windows. Mismatch exits process instead of re-prompting.

### R3-I8 — Uninstall Flow Gaps
- **Source:** A8 (Finding 7)
- **Detail:** No clean uninstall path. Raw `%APPDATA%` not expanded. No TDNS orphaning warning. No secure wipe option.

### R3-I9 — Interactive Mode Confirmation Unusable
- **Source:** A8 (Finding 15)
- **Detail:** Tray UI declared in manifest but no implementation exists. No CLI fallback for `confirm approve/reject`. Interactive mode confirmation requests timeout with no operator notification.

### R3-I10 — Silent Config Parse Failures
- **Source:** A8 (Finding 17), A9 (Finding 22)
- **Detail:** Malformed JSON silently ignored. No `config validate` or `config show` commands. No startup message indicating config source.

### R3-I11 — Product Naming / Suite Clustering
- **Source:** A9 (Finding 1)
- **Detail:** Em dash in `display_name` may mojibake. "NinjaExec" sorts under "N" not "P" with PlenumNET siblings.

### R3-I12 — Missing URL Fields
- **Source:** A9 (Finding 2)
- **Detail:** No `help_url`, `about_url`, `update_url` in manifest. Free brand real estate in Add/Remove Programs left blank.

### R3-I13 — Passphrase Prompt Obligation Framing
- **Source:** A9 (Finding 4)
- **Detail:** "Enter passphrase (min 12 characters)" → communicates obligation, not value. Should frame as protective.

### R3-I14 — Keystore Error Message Copy (5 messages)
- **Source:** A9 (Finding 7)
- **Detail:** Non-actionable errors: `"failed to generate random bytes"`, `"passphrase cannot be empty"`, `"keystore file has invalid format"`, `"unsupported keystore KDF version: {}"`, `"I/O error: {}"`. All need rewrite with guidance and warmth.

### R3-I15 — Uninstall Preserve Message Copy
- **Source:** A9 (Finding 10)
- **Detail:** Raw env var `%APPDATA%`. Cold tone. Missing reassurance about re-registration.

### R3-I16 — /status Endpoint Not Operator-Friendly
- **Source:** A9 (Finding 18)
- **Detail:** Raw JSON. No product name. No human-readable uptime. `locked: false` is double-negative framing.

### R3-I17 — No --help Flag
- **Source:** A8 (Finding 10)
- **Detail:** Unknown subcommands silently start agent. No usage message. No subcommand listing.

---

# PART III — MINOR FINDINGS (32)

## From R1 (6 findings)

| ID | Finding | Source | Detail |
|----|---------|--------|--------|
| M1 | Custom zeroize without compiler fence | A1-F11 | `write_volatile` used but no `compiler_fence` after zeroing loop |
| M2 | Shared rate limiter across endpoints | A1-F12 | Single bucket for all endpoints. Unbounded `Vec` for timestamps |
| M3 | Config file permissions not set (0600) | A1-F14 | `ninja-exec.json` written without restrictive permissions; contains `confirm_token` |
| M4 | Windows-only binary naming in cross-platform config | A2-F9 | `binary = "ninja-exec.exe"` but `architecture` includes both aarch64 and x86_64 |
| M5 | Misleading `tis27:` hash prefix | A2-F10, A3-F5 | Prefix says TIS-27 but actual primitive is TLSponge-385 `sponge::derive_key` |
| M6 | Undocumented port 21027 rationale | A3-F9 | Port 21027 not derived from PlenumNET geometric constant; rationale undocumented |

## From R2 (2 new findings)

| ID | Finding | Source | Detail |
|----|---------|--------|--------|
| R2-M1 | No structured logging format | A6-F10 | Startup messages to stdout via `println!`; not captured by Windows services |
| R2-M2 | Fingerprint not bound to Rep C (display-only) | A5-F3 | Context string `"NinjaExec-FP"` unregistered; no Rep C bound into fingerprint |

## From R3 (24 non-deferred findings)

| ID | Finding | Source |
|----|---------|--------|
| R3-M1 | No animation/transition specification | A7-F7 |
| R3-M2 | Accent dot color unspecified | A7-F8 |
| R3-M3 | Export-key icon undocumented | A7-F11 |
| R3-M4 | Windows-only uninstall message | A7-F12 |
| R3-M5 | Startup banner UTF-8 dependency | A8-F5 |
| R3-M6 | Clipboard no timeout | A8-F6 |
| R3-M7 | Status output raw JSON | A8-F8 |
| R3-M8 | Generic connection failure messages | A8-F9 |
| R3-M9 | No update mechanism | A8-F14 |
| R3-M10 | No sign feedback | A8-F16 |
| R3-M11 | Accessibility limited to CLI | A8-F18 |
| R3-M12 | MSI filename not optimized | A9-F3 |
| R3-M13 | Unlock prompt ambiguous | A9-F6 |
| R3-M14 | Startup banner formatting | A9-F5 |
| R3-M15 | Clipboard message inconsistency | A9-F12 |
| R3-M16 | Existing keystore warning cavalier | A9-F9 |
| R3-M17 | Usage messages bare | A9-F20 |
| R3-M18 | Headless mode warning browser-specific | A9-F19 |
| R3-M19 | Version output missing ecosystem reference | A9-F21 |
| R3-M20 | AlreadyExists error bare | A9-F8 |
| R3-M21 | Start Menu shortcuts not grouped | A9-F24 |
| R3-M22 | No update/migration messaging | A9-F26 |
| R3-M23 | No full cleanup warning | A9-F11 |
| R3-M24 | Confirmation queue no CLI fallback | A8-F15 (supplemental) |

---

# PART IV — DEFERRED FINDINGS (14)

These R3 findings are blocked by open R1/R2 CRITICALs. They must be re-evaluated after CRITICAL remediation.

| ID | Finding | Source | Blocked By |
|----|---------|--------|-----------|
| D1 | Startup banner UTF-8 in main.rs | A7-F4 | C7 |
| D2 | HTTP error response branding | A7-F9 | C3, C4, C5, C6 |
| D3 | CLI message prefix consistency | A7-F10 | C7 |
| D4 | CORS trust boundary UX | A8-F11 | C3, C7 |
| D5 | Audit health indicator | A8-F12 | C4, C6 |
| D6 | Signing context visibility | A8-F13 | C1, C2 |
| D7 | Signing progress feedback | A8-F19 | C5 |
| D8 | Rate limit error copy | A9-F13 | C3, C5 |
| D9 | Invalid context error copy | A9-F14 | C3, C5 |
| D10 | Keystore locked error copy | A9-F15 | C3, C5 |
| D11 | Unlock failed error passthrough | A9-F16 | C3, C4, C5 |
| D12 | Init success output restructuring | A9-F17 | C2, C7 |
| D13 | Placeholder upgrade code | A9-F23 | C2 |
| D14 | Server startup messages to audit | A9-F25 | C3, C5 |

---

# PART V — BRAND READINESS

## Brand Readiness Index: 4.33 / 10

| Agent | Role | Brand Score |
|-------|------|-------------|
| Agent 7 | Brand Guardian | 3/10 |
| Agent 8 | UX Designer | 4/10 |
| Agent 9 | Content Creator | 6/10 |

**Threshold:** 6.0 (below triggers design sprint)
**Status:** **BELOW THRESHOLD** — design sprint required

### Brand Score Justifications

**Agent 7 (3/10):** "NinjaExec has a recognizable icon concept (key with P letterform) and consistent CLI message prefixing (`[NinjaExec]`), but lacks almost every specification required for brand implementation: no color token system, no icon size requirements, no tray status rendering, no typography specification, no launcher panel design, no transition documentation. The SVG icon design will fail at the most critical rendering size (16x16 tray icon). The spec is functional as a build/install manifest but is not a brand specification."

**Agent 8 (4/10):** "NinjaExec gets the fundamentals right — localhost-only binding, TL-DSA-87, encrypted keystore, audit logging — but the operator-facing surface feels like an engineering prototype, not a shipping product. The startup banner with box-drawing characters is a nice touch, but garbled terminals, missing help text, silent config failures, raw JSON status output, and an unusable interactive confirmation mode undermine the 'professional security tool' brand. The product name and tagline ('the ssh-agent of PlenumNET') are strong. The 50th-use experience — the 3am troubleshooting session — would be frustrating: no `--help`, cryptic error messages, no diagnostic commands, no update path."

**Agent 9 (6/10):** "NinjaExec has a strong product name (memorable, action-oriented, distinctive), correct algorithm branding ('TL-DSA-87, Level 5 post-quantum security'), and several excellent copy moments (the `plenum-app.toml` clipboard export message, the 'not accessible from network' server startup line, the `tray_tooltip`). The core technical copy is competent and mostly avoids blame. However, the product is weakened by: missing Add/Remove Programs URL fields (free brand real estate left blank), ungrouped Start Menu entries, unexpanded environment variables in uninstall copy, obligation-framed passphrase prompts, non-actionable error messages, a silent config fallback that could silently override operator intent, and no update/migration messaging for an inevitable format change. The em dash in the display name is a typographic risk. The overall impression is 'technically competent developer tool' rather than 'enterprise-grade platform component.'"

### Design Sprint Deliverables

1. Color token system definition (resolves C8)
2. Icon size specifications + simplified 16×16 glyph (resolves C9)
3. Tray icon state rendering method (resolves R3-I1)
4. Typography specification (resolves R3-I2)
5. Launcher panel design or explicit deferral (resolves R3-I3)

### Readability Matrix (Agent 7 — Verbatim)

| Product | Icon File | Estimated Size | Rating | Notes |
|---------|-----------|----------------|--------|-------|
| NinjaExec | `ninja-exec.ico` | 256x256 | CLEAR | Key-with-P glyph fully legible. Gradient background, accent dot visible. |
| NinjaExec | `ninja-exec.ico` | 48x48 | MARGINAL | Key shaft and teeth distinguishable. P letterform inside key bow becomes ambiguous. Accent dot ~1.9px — barely visible. |
| NinjaExec | `ninja-exec.ico` | 32x32 | MARGINAL | Key silhouette recognizable. P letterform inside bow collapses. Accent dot ~1.25px — invisible. Stroke-width 4 at source scale = ~0.5px rendered — subpixel aliasing. |
| NinjaExec | `ninja-exec.ico` | 16x16 | UNREADABLE | ~12x12 usable pixels after OS padding. Key bow circle, P letterform, shaft, and teeth all merge into indistinct blob. Fine strokes (source stroke-width 4-6) alias to subpixel noise. Icon is not identifiable as NinjaExec. |
| NinjaExec | `ninja-exec-tray.ico` | 16x16 (tray) | UNREADABLE | System tray icons render at 16x16 (Windows) or 22x22 (macOS menu bar). At 16x16, same issues as above. No simplified glyph variant specified for tray size. |
| NinjaExec | `export-key.ico` | 32x32 (Start Menu) | UNVERIFIED | No SVG source or design documentation available. Cannot assess readability without knowing the glyph design. |

**Note:** Size ratings assessed based on SVG source geometry and stroke specifications, as the spec does not document embedded ICO sizes or simplification rules.

### Friction Map (Agent 8 — Verbatim, 29 rows)

| Phase | Action | Rating | Notes |
|-------|--------|--------|-------|
| install | Download binary | ROUGH | No documented download location, no checksum verification |
| install | Run `ninja-exec init` | ACCEPTABLE | Works but no progress indicator, no pre-flight validation |
| install | Enter passphrase | ROUGH | No strength feedback, no character counter, no show/hide toggle |
| install | Confirm passphrase | ROUGH | Mismatch exits process instead of re-prompting |
| install | Receive public key | ACCEPTABLE | Key is displayed; clipboard copy works on most platforms |
| install | Copy public key to clipboard | ACCEPTABLE | Fallback to stdout on headless; no timeout on clipboard process |
| install | Share public key with admin | ACCEPTABLE | Message tells operator what to do with the key |
| configure | Locate config file | ROUGH | No `config show` command; operator must know the data directory path |
| configure | Edit ninja-exec.json | ROUGH | No validation on save; malformed JSON silently ignored |
| configure | Verify config is loaded | ROUGH | No startup message confirming which config was loaded |
| configure | Set up tray agent | ROUGH | `configure_command` is empty; no tray UI ships with NinjaExec |
| operate | Start agent (`ninja-exec run`) | ACCEPTABLE | Startup banner is informative; headless warning is clear |
| operate | Check status (`ninja-exec status`) | ROUGH | Raw JSON output; no human-readable formatting |
| operate | Lock agent | SMOOTH | `ninja-exec lock` works, clear response |
| operate | Unlock agent | ACCEPTABLE | Passphrase prompt works; no echo on Unix; echo visible on Windows |
| operate | Sign a file (CLI) | ACCEPTABLE | Works but no feedback on what was signed |
| operate | Sign via HTTP API | ACCEPTABLE | Well-structured JSON request/response |
| operate | Approve confirmation (interactive) | ROUGH | No CLI or tray UI to approve; requires external tool |
| operate | View audit log | ROUGH | No `audit tail` command; must manually open JSONL file |
| operate | Export operator identity | SMOOTH | JSON output with all needed fields; clipboard option works |
| operate | Diagnose connection failure | ROUGH | Generic error messages; no distinction between failure modes |
| operate | Get help | ROUGH | No `--help` flag; no usage message |
| update | Check for updates | ROUGH | No update mechanism exists |
| update | Apply update | ROUGH | No update procedure documented |
| update | Rollback failed update | ROUGH | No rollback mechanism |
| uninstall | Run uninstaller | ACCEPTABLE | `preserve_data = true` retains keystore |
| uninstall | Find preserved data | ROUGH | Raw `%APPDATA%` not expanded; path not copyable |
| uninstall | Perform clean removal | ROUGH | No supported path; no `ninja-exec destroy` command |
| uninstall | Verify key material removed | ROUGH | No verification command; operator must manually check filesystem |

**action_count:** 29

**Distribution:** 2 SMOOTH / 9 ACCEPTABLE / 18 ROUGH

| Phase | Actions | SMOOTH | ACCEPTABLE | ROUGH |
|-------|---------|--------|------------|-------|
| Install | 7 | 0 | 4 | 3 |
| Configure | 4 | 0 | 0 | 4 |
| Operate | 11 | 2 | 4 | 5 |
| Update | 3 | 0 | 0 | 3 |
| Uninstall | 4 | 0 | 1 | 3 |
| **Total** | **29** | **2** | **9** | **18** |

18 of 29 operator actions rated ROUGH. Only 2 rated SMOOTH.

### Copy Audit Table (Agent 9 — Verbatim, 25 rows)

| Location | Current Copy | Recommended Copy | Change Type | Priority |
|----------|-------------|-----------------|-------------|----------|
| `plenum-app.toml` line 3 `display_name` | `"NinjaExec — PlenumNET Signing Agent"` | `"PlenumNET NinjaExec - Signing Agent"` | REVISED | IMPORTANT |
| `plenum-app.toml` `[app]` section | (no URL fields) | Add `help_url`, `about_url`, `update_url` with HTTPS plenumnet.com links | NEW | IMPORTANT |
| `plenum-app.toml` `[install]` section | (no MSI filename) | `msi_filename = "PlenumNET-NinjaExec-{version}-{arch}.msi"` | NEW | MINOR |
| `main.rs` line 170 | `"Enter passphrase (min 12 characters): "` | `"Create a passphrase to protect your signing key (12+ characters): "` | REVISED | IMPORTANT |
| `main.rs` line 308, 337, 431 | `"Passphrase: "` | `"NinjaExec passphrase: "` | REVISED | MINOR |
| `main.rs` line 75 | `"[NinjaExec] Could not access clipboard. Output printed instead:"` | `"[NinjaExec] Clipboard not available on this system. Your public signing key is printed below instead."` | REVISED | MINOR |
| `main.rs` line 106 | `"[NinjaExec] No clipboard utility found. Output printed instead:"` | `"[NinjaExec] Clipboard not available on this system. Your public signing key is printed below instead."` | REVISED | MINOR |
| `main.rs` line 163 | `"[NinjaExec] To create a new keystore, remove the existing one first."` | `"[NinjaExec] To start fresh, first export your public key with 'ninja-exec export-operator', then securely delete the existing keystore."` | REVISED | MINOR |
| `main.rs` line 322 | `"Usage: ninja-exec sign <file>"` | `"Sign a file with your NinjaExec key.\nUsage: ninja-exec sign <file>"` | REVISED | MINOR |
| `main.rs` line 373 | `"Usage: ninja-exec verify <file> <signature_b64>"` | `"Verify a file signature.\nUsage: ninja-exec verify <file> <signature_b64>"` | REVISED | MINOR |
| `main.rs` lines 456–458 | `"WARNING: Headless mode — all signing requests will be auto-approved.\nDo not use in environments where browser tabs may be compromised."` | `"[NinjaExec] Running in headless mode. Signing requests will be approved automatically without operator confirmation. This mode is intended for CI/CD pipelines and automated environments only."` | REVISED | MINOR |
| `main.rs` after line 154 | (no ecosystem reference) | Add `"Part of the PlenumNET platform — https://plenumnet.com"` | NEW | MINOR |
| `keystore.rs` `EntropyFailure` display | `"failed to generate random bytes"` | `"Unable to generate secure random data. Ensure your system's random number generator is available and try again."` | REVISED | IMPORTANT |
| `keystore.rs` `EmptyPassphrase` display | `"passphrase cannot be empty"` | `"A passphrase is required to protect your signing key. Please provide one and try again."` | REVISED | IMPORTANT |
| `keystore.rs` `InvalidFormat` display | `"keystore file has invalid format"` | `"The keystore file appears damaged or was created by an incompatible version. If this problem persists, contact support or re-initialize with 'ninja-exec init'."` | REVISED | IMPORTANT |
| `keystore.rs` `UnsupportedVersion` display | `"unsupported keystore KDF version: {}"` | `"This keystore was created with a newer version of NinjaExec. Please update NinjaExec to open it."` | REVISED | IMPORTANT |
| `keystore.rs` `IoError` display | `"I/O error: {}"` | `"Could not access the keystore file: {}. Check that the file exists and you have permission to read it."` | REVISED | IMPORTANT |
| `keystore.rs` `AlreadyExists` display | `"keystore already exists"` | `"A keystore already exists at this location. To create a new one, first back up and remove the existing keystore file."` | REVISED | MINOR |
| `plenum-app.toml` line 45 `preserve_message` | `"Your NinjaExec signing key and audit history have been preserved in %APPDATA%\NinjaExec..."` | `"Your NinjaExec signing key and audit history have been preserved in {resolved_data_dir}. If you reinstall NinjaExec, your existing identity will be automatically restored — no re-registration needed."` | REVISED | IMPORTANT |
| `plenum-app.toml` `[uninstall]` section | (no cleanup message) | Add `cleanup_message` with secure removal guidance and re-registration warning | NEW | MINOR |
| `plenum-app.toml` `[shortcuts]` | `"Export Public Key"` | `"Copy Signing Identity"` | REVISED | MINOR |
| `config.rs` `load()` fallback | (silent — no message) | `"[NinjaExec] Warning: Configuration file at {path} could not be read. Using default settings..."` | NEW | IMPORTANT |
| `server.rs` `/status` response | `{"running":true,"locked":false,...}` | Add `"product":"NinjaExec"`, `"algorithm":"TL-DSA-87"`, human-readable uptime | REVISED | IMPORTANT |
| `plenum-app.toml` | (no `[update]` section) | Add update and migration message templates | NEW | MINOR |
| `keystore.rs` `DecryptionFailed` display | `"decryption failed (wrong passphrase or corrupted keystore)"` | Flag for Security Engineer co-review — no copy change until security assessment | OK | — |

**Additional text instances reviewed — no changes recommended:** 14 instances including: `tray_tooltip` (good), `[NinjaExec] Keystore created at...` (good), `[NinjaExec] Public key:...` (good), `[NinjaExec] Fingerprint:...` (good), `/sign` success response fields (machine-readable, appropriate), `/verify` response (machine-readable), `/pubkey` response fields (acceptable), `/lock` response (acceptable), `VALID`/`INVALID` CLI verify output (appropriate for machine consumption), `[NinjaExec] Signing agent listening on...` (good), `[NinjaExec] Bound to {} only — not accessible from network` (excellent), `[NinjaExec] Operator identity copied to clipboard.` (good), copyright headers (correct), `[NinjaExec] Server error: {}` (acceptable).

---

# PART VI — CRYPTOGRAPHIC CLAIMS VERIFICATION

## Agent 1 (Security Engineer) — Crypto Verification

| Claim | Location | Verdict | Notes |
|---|---|---|---|
| TL-DSA-87 used for all signatures | signing_engine.rs line 7 | **VERIFIED** | `TlDsaVariant::TlDsa87` is hardcoded; all sign/verify calls go through `tl_dsa` module |
| No Ed25519 / no `crypto.sign` | Full codebase | **VERIFIED** | No Ed25519, no Node.js crypto calls found in Rust sources |
| TLSponge-385 used for KDF | keystore.rs line 71 | **VERIFIED** | `ternary_math::sponge::derive_key` is used with domain separator |
| TLSponge-385 used for audit hash | audit.rs line 60 | **VERIFIED** | `ternary_math::sponge::derive_key` with domain `b"NinjaExec-AUDIT-HASH"` |
| TLSponge-385 used for fingerprint | signing_engine.rs line 27 | **VERIFIED** | `ternary_math::sponge::derive_key` with domain `b"NinjaExec-FP"` |
| Keystore AE uses PlenumNET primitives only | keystore.rs lines 86-153 | **VERIFIED** | Custom Encrypt-then-MAC using TLSponge; no AES-256-GCM, no external crypto |
| Rep C address bound into signing context (INVARIANT 9) | signing_engine.rs lines 13-18 | **INCORRECT** | No Rep C address appears in any signing context |
| Rep C address bound into KDF domain separation (INVARIANT 9) | keystore.rs line 71 | **INCORRECT** | KDF domain is `b"NinjaExec-KDF-v2"` with no Rep C address |
| Constant-time tag comparison | keystore.rs lines 130-133 | **VERIFIED** | XOR-accumulate pattern used |
| Constant-time confirm_token comparison | server.rs line 540 | **INCORRECT** | Standard `!=` string comparison used |
| No raw binary integers enter sponge absorb (INVARIANT 8) | keystore.rs, audit.rs, signing_engine.rs | **UNVERIFIED** | Depends on `ternary_math::sponge` internals |
| Upgrade code deterministically derived | plenum-app.toml line 10 | **INCORRECT** | Hardcoded placeholder UUID |

## Agent 3 (PlenumNET Integration) — Crypto Verification

| Claim | Location | Status | Notes |
|-------|----------|--------|-------|
| TL-DSA-87 for all signatures | `signing_engine.rs` line 7 | **VERIFIED** | Uses `TlDsaVariant::TlDsa87` via `ternary_math::tl_dsa` |
| No Ed25519 / no external crypto | `Cargo.toml` dependencies | **VERIFIED** | No `ed25519`, `ring`, `openssl`, `sha2`, `blake3` crates |
| TLSponge for key derivation | `keystore.rs` line 71 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` (kernel sponge) |
| TLSponge for keystream generation | `keystore.rs` line 98 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| TLSponge for authentication tag | `keystore.rs` line 111 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| TLSponge for audit hashing | `audit.rs` line 60 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| TLSponge for fingerprinting | `signing_engine.rs` line 27 | **VERIFIED** | Uses `ternary_math::sponge::derive_key` |
| Rep C address binding in signatures | All source files | **INCORRECT** | No Rep C address used anywhere (INVARIANT 9 violation) |
| Context string `"NinjaExec-FP"` | `signing_engine.rs` line 27 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KDF-v2"` | `keystore.rs` line 71 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KS-STREAM"` | `keystore.rs` line 98 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-KS-TAG"` | `keystore.rs` line 111 | **UNVERIFIED** | Not in canonical registry |
| Context string `"NinjaExec-AUDIT-HASH"` | `audit.rs` line 60 | **UNVERIFIED** | Not in canonical registry |
| "Level 5 post-quantum security" | `main.rs` line 153 | **VERIFIED** | TL-DSA-87 = NIST PQ Level 5 |
| No AES-256-GCM | All source files | **VERIFIED** | Custom sponge-based AE used; T-AE-MAC preferred |
| Constant-time tag comparison | `keystore.rs` lines 130–134 | **VERIFIED** | XOR-accumulation pattern |
| Key zeroization on drop | `keystore.rs` lines 316–319 | **VERIFIED** | `Drop` impl calls `zeroize()` with `write_volatile` |

## Consolidated Verification Summary

| Claim | Verdict | Notes |
|-------|---------|-------|
| TL-DSA-87 for all signatures | **VERIFIED** | All 3 R1 agents confirmed |
| No external crypto dependencies | **VERIFIED** | No Ed25519, AES, SHA, BLAKE crates |
| TLSponge-385 for KDF/MAC/hash | **VERIFIED** | `ternary_math::sponge::derive_key` throughout |
| Constant-time tag comparison (keystore) | **VERIFIED** | XOR-accumulate pattern |
| Key zeroization on Drop | **VERIFIED** | `write_volatile` in `Drop` impl |
| Level 5 post-quantum security | **VERIFIED** | TL-DSA-87 = NIST PQ Level 5 |
| Rep C address binding (INVARIANT 9) | **FAILED** | No Rep C anywhere in codebase |
| Constant-time token comparison (server) | **FAILED** | Standard `!=` used |
| Upgrade code deterministically derived | **FAILED** | Hardcoded placeholder |
| Context strings in canonical registry | **UNVERIFIED** | 5 strings unregistered |

### Passphrase Entropy Assessment (Agent 1)

- **Minimum:** 72 bits effective
- **Analysis:** NinjaExec enforces a 12-character minimum passphrase length. Assuming printable ASCII (95 characters), 12 characters yield `12 × log₂(95) ≈ 78.8 bits` in the best case. However, human-chosen passphrases are significantly weaker — NIST SP 800-63B estimates ~1-2 bits per character for user-chosen passwords beyond the first 8 characters. A 12-character user-chosen passphrase may have as few as ~30 bits of effective entropy.
- **Current KDF:** 4096 iterations provides only ~12 bits of work factor
- **Gap:** To reach 72 bits effective with 12 bits of KDF work, the passphrase itself must provide ~60 bits
- **Path:** Increase KDF to ≥100K iterations OR enforce 16+ chars with composition guidance OR recommend Diceware

---

# PART VII — AGENT DELIVERABLES (R2)

## Coverage Matrix (Agent 4 — Verbatim, 69 entries)

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
| **INVARIANT 8: No raw binary in sponge** | `keystore.rs` | PARTIALLY COVERED | Needs audit of ternary_math internals |
| **INVARIANT 9: Rep C in all crypto ops** | All | NOT COVERED | No Rep C anywhere in codebase (C2) |

### Coverage Summary Statistics

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

**Coverage: 21 COVERED, 3 PARTIALLY, 45 NOT COVERED = 65% NOT COVERED**

## Feasibility Risk Table (Agent 5 — Verbatim)

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

## Operator Readiness Checklist (Agent 6 — Verbatim, 16 items)

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

**Summary: 1 YES, 5 PARTIALLY, 10 NO — operationally undeployable**

---

# PART VIII — QC-R3 RECURSIVE REVIEW (REVIEW QUALITY)

The R3 review documents themselves were reviewed by 3 recursive agents.

**Verdict:** PASS WITH CONDITIONS | **Brand Readiness Index:** 7.33/10

| Agent | Role | Brand Score | Finding Count |
|-------|------|-------------|---------------|
| 7R | Brand Guardian | 7/10 | 0C + 5I + 5M = 10 |
| 8R | UX Designer | 7/10 | 0C + 5I + 7M = 12 |
| 9R | Content Creator | 8/10 | 0C + 4I + 12M = 16 |

## Agent 7R Findings (Brand Guardian Recursive — 10 findings)

### 7R-F1 (IMPORTANT): Agent 8 finding count error
Agent 8's Review Complete states "6 IMPORTANT, 9 MINOR" but actual count is 7 IMPORTANT (F1, F2, F3, F4, F7, F15, F17), 8 MINOR. Finding 3 (cancel/rollback during init) omitted from Conditions list.

### 7R-F2 (IMPORTANT): Agent 9 finding count error
Agent 9's Review Complete states "IMPORTANT: 8, MINOR: 11" but actual count is 7 IMPORTANT (F1, F2, F4, F7, F10, F18, F22), 12 MINOR. Conditions text says "8 non-deferred IMPORTANT findings" but enumerates only 7.

### 7R-F3 (IMPORTANT): Consolidated Reviewers table propagates errors
Agent 8 row: `0C + 6I + 9M + 4D = 19` (should be `0C + 7I + 8M + 4D = 19`). Agent 9 row: `0C + 8I + 11M + 7D = 26` (should be `0C + 7I + 12M + 7D = 26`). Errors cancel in aggregate (coincidental correctness).

### 7R-F4 (IMPORTANT): Readability Matrix format deviation
Agent 7 uses 5 columns (`Product | Icon File | Estimated Size | Rating | Notes`) vs template spec's 4 columns (`Product | Icon Size | Rating | Notes`). Machine extraction relying on template schema will fail.

### 7R-F5 (MINOR): Consolidated matrix missing Product column
Consolidated Readability Matrix uses `Icon File | Size | Rating | Notes` — third column schema variant across report suite.

### 7R-F6 (IMPORTANT): Missing reviewer provenance (commit hashes)
None of the four R3 reports include commit hashes for skill documents or source documents. Template explicitly requires them. Agent 9 records "UNCOMMITTED" for primary spec; Agents 7 and 8 omit entirely.

### 7R-F7 (MINOR): R3-M24 duplicates R3-I9 source
Consolidated lists R3-M24 and R3-I9 both attributed to Agent 8 Finding 15 at different severities. Double-counting inflates total.

### 7R-F8 (MINOR): Heading hierarchy inconsistency
Agent 7 uses two H1 lines; Agents 8 and 9 use one H1. Separator placement varies.

### 7R-F9 (MINOR): DEFERRED finding count formatting
"11-13" range notation could be misread as three findings vs comma-separated "11, 12, 13, 19" which is clearer.

### 7R-F10 (MINOR): Quick Win 3 bundles two unrelated changes
Consolidated Quick Win 3 bundles URL fields + passphrase prompt (effectively Top 4, not Top 3).

## Agent 8R Findings (UX Designer Recursive — 12 findings)

### 8R-F1 (IMPORTANT): Friction Map action_count wrong
Table contains 29 data rows but `action_count: 30` declares 30. Off by +1.

### 8R-F2 (IMPORTANT): Consolidated Friction Map Summary errors
Install phase reported as 6 actions (should be 7). ACCEPTABLE under-reported by 1. Total line says "30 (incl. 2 not shown)" with no explanation.

### 8R-F3 (IMPORTANT): Agent 8 finding count breakdown
States "6 IMPORTANT, 9 MINOR" but actual is 7 IMPORTANT, 8 MINOR. Confirms 7R-F1.

### 8R-F4 (IMPORTANT): Agent 9 finding count breakdown
States "IMPORTANT: 8, MINOR: 11" but actual is 7 IMPORTANT, 12 MINOR. Conditions text contradicts count. Confirms 7R-F2.

### 8R-F5 (IMPORTANT): R3-I17 silent severity reclassification
Consolidated promotes Agent 8 Finding 10 from MINOR to IMPORTANT without documenting rationale. Template does not authorize reclassification during consolidation.

### 8R-F6 (MINOR): Agent 8 conditions list omits Finding 3
Conditions state "Resolve Findings 1, 2, 4, 7, 15, 17" (6 findings) but Agent 8 has 7 IMPORTANT. Finding 3 (init cancel/rollback) missing.

### 8R-F7 (MINOR): Missing commit hashes
All three reports lack commit hashes per template requirement. Confirms 7R-F6.

### 8R-F8 (MINOR): No finding index in consolidated
57 findings across 4 sections with three numbering systems (R3-XX, FN, A7-FN). No mapping table.

### 8R-F9 (MINOR): Table accessibility
No caption text or summary attributes for screen reader context.

### 8R-F10 (MINOR): Friction Map summary denominator
Consolidated states "18 of 30" — denominator wrong (should be 29). ACCEPTABLE category (9) not mentioned.

### 8R-F11 (MINOR): Copy Audit Table count wrong
Consolidated says "18 copy revision entries" but Agent 9 table has 24 data rows.

### 8R-F12 (MINOR): Cross-reference notation inconsistency
DEFERRED table uses "A7-F4" (compact) but other sections use "Agent 7, Finding 3" (prose).

## Agent 9R Findings (Content Creator Recursive — 16 findings)

### 9R-F1 (MINOR): Title format inconsistency
Agent 8 title placement of "Review" differs from Agents 7 and 9.

### 9R-F2 (MINOR): Missing commit hashes in source document tables
Level of detail varies across agents. None include commit hashes per protocol.

### 9R-F3 (IMPORTANT): Agent 8 Review Complete format
Less detailed than Agent 7's tabular breakdown with finding IDs. Consolidator must manually count.

### 9R-F4 (IMPORTANT): Copy Audit Table delegation
Consolidated delegates to Agent 9 report rather than reproducing inline. Claims "18 copy revision entries" but actual count is 24+ rows. Breaks standalone readability.

### 9R-F5 (IMPORTANT): Aggregate Finding Count documentation
Raw sum (57) vs deduplicated detailed listings — deduplication not documented. Reader counting entries gets different number.

### 9R-F6 (MINOR): Copy Audit Table quality (POSITIVE)
"The single most implementable artifact in the entire QC-R3 review. Every REVISED entry includes verbatim 'Current Copy' text quoted from the source code and a 'Recommended Copy' entry that is polished and ready to paste into code."

### 9R-F7 (MINOR): Top 3 Quick Wins quality (POSITIVE)
"All three agents provide compelling, actionable Top 3 Quick Wins with effort estimates."

### 9R-F8 (MINOR): Sensitive Material Prohibition compliance (POSITIVE)
"All four documents respect the Sensitive Material Prohibition. Full compliance verified."

### 9R-F9 (MINOR): Brand Score justifications (POSITIVE)
"All three scores are internally consistent and the spread (3, 4, 6) is reasonable."

### 9R-F10 (MINOR): DEFERRED handling (POSITIVE)
"DEFERRED handling is protocol-compliant and accurately consolidated."

### 9R-F11 (IMPORTANT): Consolidated standalone readability gaps
Three gaps: (1) Copy Audit Table delegated, (2) Friction Map is summary only, (3) Design Sprint Gate lacks ownership/timeline.

### 9R-F12 (MINOR): Finding format compliance (POSITIVE)
"The `**Field:**` bold syntax is used correctly for machine extraction."

### 9R-F13 (MINOR): Friction Map completeness
Consolidated "Total: 30 (incl. 2 not shown)" — 2 unshown actions not identified.

### 9R-F14 (MINOR): Writing quality (POSITIVE)
"The prose across all four documents is consistently professional, clear, and free of unnecessary jargon."

### 9R-F15 (MINOR): Copy Audit Table verbatim accuracy (POSITIVE)
Spot-check confirms quotes are internally consistent with line references.

### 9R-F16 (MINOR): Reviewer table and verdicts (POSITIVE)
"Well-structured decision-making guidance."

## Corrections Required (10 items from R3R)

| # | What to Fix | Effort |
|---|-------------|--------|
| 1 | Agent 8 finding count: 6I→7I, 9M→8M; add F3 to conditions | 2 min |
| 2 | Agent 9 finding count: 8I→7I, 11M→12M; fix conditions text | 2 min |
| 3 | Consolidated Reviewers table: fix per-agent breakdowns | 2 min |
| 4 | Friction Map action_count: 30→29 | 2 min |
| 5 | Consolidated Friction Map Summary: fix Install counts (6→7) | 2 min |
| 6 | Add UNCOMMITTED provenance to all 4 R3 reports | 10 min |
| 7 | Revert R3-I17 to MINOR or document reclassification | 2 min |
| 8 | Fix Readability Matrix columns to 4-col template spec | 5 min |
| 9 | Reproduce Copy Audit Table inline in consolidated + fix count (18→24) | 10 min |
| 10 | Add finding index mapping table to consolidated | 10 min |

## Quality Attestations (Positive Findings from R3R)

The recursive review confirmed these strengths:
- Copy Audit Table: "gold standard for implementability" — every entry paste-ready
- Top 3 Quick Wins: consistently compelling and actionable across all reports
- Sensitive Material Prohibition: full compliance verified
- Brand Score justifications: well-written, fair, internally consistent
- DEFERRED handling: protocol-compliant throughout
- Finding format: machine-extractable, correctly implemented
- Writing quality: "publication-ready" across all documents
- Cross-agent consistency: "No contradictions found. All overlapping findings address different aspects of the same issue from the agent's specialized lens."

---

# PART IX — SUMMARY VERDICTS (All Agents)

## Agent 1 (Security Engineer) — R1
**Verdict: FAIL**
"Three CRITICAL findings block implementation: (1) TL-DSA signing context does not bind the signer's Rep C address, violating INVARIANT 9. No Rep C address exists anywhere in the codebase. (2) The CORS policy allows any origin to access the localhost signing API. Combined with headless mode auto-approval, any website in the operator's browser can obtain TL-DSA signatures over attacker-controlled payloads. This is a remotely exploitable signature oracle."

## Agent 2 (DevOps Automator) — R1
**Verdict: FAIL**
"Three CRITICAL findings block implementation: (1) R1-A2-1 (Audit silent failure): A signing operation can return a valid signature while the audit trail silently fails. (2) R1-A2-3 (No Rep C in audit entries): INVARIANT 9 is violated — audit records identify nodes by HTTP origin URLs and hostnames instead of Rep C addresses. (3) R1-A2-4 (No Rep C context binding in signatures): INVARIANT 7 is violated."

## Agent 3 (PlenumNET Integration) — R1
**Verdict: FAIL**
"INVARIANT 9 is systematically violated: no Rep C address exists anywhere in the codebase. All cryptographic operations that bind node identity must use Rep C (54-trit, binary-encoded) addressing exclusively. Additionally, five context strings used in sponge derivation are UNVERIFIED against the canonical registry."

## Agent 4 (Evidence Collector) — R2
**Verdict: FAIL**
"The coverage matrix shows 45 of 69 test areas as NOT COVERED, with the most severe gaps in HTTP endpoint testing, credential handling edge cases, and INVARIANT 9 compliance. The hardcoded upgrade code (I6), confirm token printed to stdout (I7), non-constant-time token comparison (I3), and absence of any CI pipeline (I9) compound the risk."

## Agent 5 (Senior Developer) — R2
**Verdict: FAIL**
"The most architecturally significant — C2 (no Rep C address anywhere in the codebase) — requires a keystore format migration, touches every module, and depends on `ternary_math` exporting a Rep C derivation function. The implementation path is feasible — no circular dependencies exist, and the highest-risk task (C2) can be decomposed into incremental steps."

## Agent 6 (Infrastructure Maintainer) — R2
**Verdict: FAIL**
"From an operator deployment perspective, NinjaExec is not ready for release. The Operator Readiness Checklist scores 1 YES, 5 PARTIALLY, 10 NO — operationally undeployable."

## Agent 7 (Brand Guardian) — R3
**Verdict: FAIL | Brand Score: 3/10**
"Two CRITICAL brand findings block implementation: the absence of any color token system and the absence of icon size specifications. NinjaExec's tray icon will be unreadable at its primary rendering size (16x16). The specification is a functional build manifest but does not constitute a brand specification."

## Agent 8 (UX Designer) — R3
**Verdict: PASS WITH CONDITIONS | Brand Score: 4/10**
"The passphrase prompt provides no strength feedback against the 72-bit entropy floor. Interactive mode confirmation is unusable without an external UI. Silent config parsing failures create false security confidence. The uninstall flow has no clean-removal path."
**Conditions:** Resolve Findings 1, 2, 3, 4, 7, 15, 17 (all IMPORTANT). Resolve all 7 open R1/R2 CRITICALs.

## Agent 9 (Content Creator) — R3
**Verdict: PASS WITH CONDITIONS | Brand Score: 6/10**
"Resolve the 7 non-deferred IMPORTANT findings (Findings 1, 2, 4, 7, 10, 18, 22 — product naming, URL fields, passphrase prompt, error message copy, uninstall copy, status endpoint, config fallback) before first product release. The 7 DEFERRED findings must be re-evaluated after R1/R2 CRITICAL remediation."

## Agent 7R (Brand Guardian Recursive) — R3R
**Verdict: PASS WITH CONDITIONS | Brand Score: 7/10**
"The report suite demonstrates strong analytical quality and domain expertise. The conditions address data accuracy and protocol compliance issues that, once resolved, would produce an auditable, machine-extractable, and internally consistent set of review deliverables."

## Agent 8R (UX Designer Recursive) — R3R
**Verdict: PASS WITH CONDITIONS | Brand Score: 7/10**
"The reports contain multiple numerical errors that undermine their reliability as reference documents. These are not cosmetic issues. A project manager using these reports to plan remediation sprints would have incorrect severity counts."

## Agent 9R (Content Creator Recursive) — R3R
**Verdict: PASS WITH CONDITIONS | Brand Score: 8/10**
"The QC-R3 review documents are high-quality deliverables that demonstrate thorough analysis, protocol compliance, and professional writing. The Copy Audit Table is the standout deliverable — every entry is paste-ready."

---

# PART X — RESOLUTION PRIORITY

## Phase 1: CRITICALs (blocks all release)

| Priority | Finding | Type | Risk | Effort |
|----------|---------|------|------|--------|
| 1 | C1 + C2: Rep C address provisioning + signature binding | Architecture | HIGH | Large |
| 2 | C3: CORS origin restriction | Config | LOW | Medium |
| 3 | C4: Audit fail-closed | Refactor | MEDIUM | Medium |
| 4 | C5: Integration test suite | Testing | — | Large |
| 5 | C6: Rep C in audit entries | Depends on C2 | — | Small |
| 6 | C7: Token not printed to stdout | Quick fix | LOW | Small |
| 7 | C8: Color token system | Design | — | Medium |
| 8 | C9: Icon size specs + 16×16 glyph | Design | — | Medium |

## Phase 2: IMPORTANT (blocks first release)

After CRITICALs, resolve all 37 IMPORTANT findings. Top priority:
1. I4: Headless mode restriction (`headless_allow` config) — LOW risk
2. I3: Constant-time token comparison — LOW risk, ~5 lines
3. I1: KDF iteration increase (benchmark-driven) — LOW risk
4. R3-I10: Silent config parse failures — LOW risk
5. R3-I9: CLI confirmation fallback — MEDIUM risk
6. R3-I14: Keystore error message copy rewrite (see Copy Audit Table)

## Phase 3: MINOR + DEFERRED (iterative polish)

32 MINOR findings are polish items. 14 DEFERRED findings will un-defer as CRITICALs are resolved and must be re-evaluated.

## Phase 4: R3R Corrections (review document cleanup)

10 bookkeeping corrections to R3 review documents (see Part VIII). Total effort: ~47 minutes.

---

# PART XI — INDIVIDUAL REPORT INDEX

| Report | Path |
|--------|------|
| **R1 Consolidated** | `ninja-exec/qc-r1-consolidated.md` |
| R1 Agent 1 — Security Engineer | `ninja-exec/qc-r1-agent1-security-engineer.md` |
| R1 Agent 2 — DevOps Automator | `ninja-exec/qc-r1-agent2-devops-automator.md` |
| R1 Agent 3 — PlenumNET Integration | `ninja-exec/qc-r1-agent3-plenumnet-integration.md` |
| **R2 Consolidated** | `ninja-exec/qc-r2-consolidated.md` |
| R2 Agent 4 — Evidence Collector | `ninja-exec/qc-r2-agent4-evidence-collector.md` |
| R2 Agent 5 — Senior Developer | `ninja-exec/qc-r2-agent5-senior-developer.md` |
| R2 Agent 6 — Infrastructure Maintainer | `ninja-exec/qc-r2-agent6-infrastructure-maintainer.md` |
| **R3 Consolidated** | `ninja-exec/qc-r3-consolidated.md` |
| R3 Agent 7 — Brand Guardian | `ninja-exec/qc-r3-agent7-brand-guardian.md` |
| R3 Agent 8 — UX Designer | `ninja-exec/qc-r3-agent8-ux-designer.md` |
| R3 Agent 9 — Content Creator | `ninja-exec/qc-r3-agent9-content-creator.md` |
| **R3 Recursive Consolidated** | `ninja-exec/qc-r3-recursive-consolidated.md` |
| R3R Agent 7R — Brand Guardian | `ninja-exec/qc-r3-recursive-agent7-brand-guardian.md` |
| R3R Agent 8R — UX Designer | `ninja-exec/qc-r3-recursive-agent8-ux-designer.md` |
| R3R Agent 9R — Content Creator | `ninja-exec/qc-r3-recursive-agent9-content-creator.md` |
| **This Document** | `ninja-exec/task-54-full-qc-review.md` |
