# QC-R3 Agent 9 — Content Creator / Growth Strategist Review

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Review Date:** 2026-03-28
**Protocol:** QC-R3 (Round 3 — Fit & Finish)
**YODA Role ID:** `marketing/content-creator`
**Reviewer:** Agent 9 — Content Creator / Growth Strategist

---

## Source Documents Reviewed

| Document | Path |
|----------|------|
| Primary spec/manifest | `ninja-exec/plenum-app.toml` |
| Main entry point | `ninja-exec/src/main.rs` |
| HTTP API server | `ninja-exec/src/server.rs` |
| Keystore module | `ninja-exec/src/keystore.rs` |
| Configuration module | `ninja-exec/src/config.rs` |
| Confirmation module | `ninja-exec/src/confirm.rs` |
| Audit module | `ninja-exec/src/audit.rs` |
| CLI argument parser | `ninja-exec/src/cli.rs` |
| Signing engine | `ninja-exec/src/signing_engine.rs` |
| Cargo manifest | `ninja-exec/Cargo.toml` |
| R1 consolidated findings | `ninja-exec/qc-r1-consolidated.md` |
| R2 consolidated findings | `ninja-exec/qc-r2-consolidated.md` |
| Content Creator skill | `.agents/skills/content-creator/SKILL.md` |

**Integrity Verification:** UNCOMMITTED — hash verification deferred to post-commit review.

---

## Open R1/R2 CRITICAL Findings (Sequencing Constraint)

The following 7 CRITICAL findings from R1/R2 are unresolved. Findings in this review that overlap with sections governed by these CRITICALs are marked **DEFERRED** and do not affect the Summary Verdict or Brand Score.

| ID | Summary | Affected Sections |
|----|---------|-------------------|
| C1 | No Rep C address binding in TL-DSA signatures | `signing_engine.rs` |
| C2 | No Rep C address exists anywhere in codebase | Entire codebase |
| C3 | CORS wildcard origin creates signature oracle | `server.rs` (CORS layer) |
| C4 | Audit log silently swallows all write failures | `audit.rs`, `server.rs` (audit calls) |
| C5 | No integration tests for any HTTP endpoint | `server.rs` |
| C6 | Audit entries lack Rep C addresses | `audit.rs`, `server.rs` |
| C7 | Confirm token printed to stdout | `main.rs` lines 191–193 |

---

## Findings

### Finding 1
- **Section:** `plenum-app.toml` lines 1–9 — Product naming and discoverability
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The `display_name` field is `"NinjaExec — PlenumNET Signing Agent"`. The em dash character (—) may not render correctly in all Windows Add/Remove Programs views, registry editors, or third-party uninstall utilities. Some older control panel renderings strip or mojibake non-ASCII characters. Additionally, the display name does not cluster alphabetically with other PlenumNET products. If the suite ships "PlenumNET Array3", "PlenumNET Browser", etc., a name starting with "NinjaExec" sorts under "N" rather than under "P" with its siblings. Operators managing multiple PlenumNET products will not see them grouped together.
- **Recommendation:** Change `display_name` to `"PlenumNET NinjaExec - Signing Agent"`. This clusters under "P" with other PlenumNET products, uses a plain hyphen for maximum compatibility, and still preserves the NinjaExec brand identity. Retain "NinjaExec" as the primary product name in documentation and marketing, but use the suite-prefixed form for Windows Add/Remove Programs.
- **Impact:** Operators managing multiple PlenumNET products cannot visually locate NinjaExec alongside its sibling applications. The em dash may render as garbage characters on some systems, undermining the "serious company" perception.

### Finding 2
- **Section:** `plenum-app.toml` — Missing URL fields for Add/Remove Programs
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The `plenum-app.toml` manifest contains no `url`, `help_url`, `update_url`, or `about_url` fields. Windows Add/Remove Programs surfaces these as free brand real estate: HelpLink, URLInfoAbout, and URLUpdateInfo registry values are displayed to operators who right-click the product entry. These fields are currently absent, leaving blank entries where competitors show polished support links.
- **Recommendation:** Add the following fields to the `[app]` section: `help_url = "https://plenumnet.com/docs/ninja-exec"`, `about_url = "https://plenumnet.com/ninja-exec"`, `update_url = "https://plenumnet.com/downloads/ninja-exec"`. All URLs must use HTTPS and resolve to Capomastro-controlled domains (plenumnet.com, capomastro.com, or salvigroup.com).
- **Impact:** Missed opportunity for brand visibility in the one place every operator eventually visits. Blank URL fields look unfinished compared to enterprise software that populates them.

### Finding 3
- **Section:** `plenum-app.toml` — Missing MSI filename optimization
- **Severity:** MINOR
- **Round:** R3
- **Finding:** No `msi_filename` or installer naming convention is specified in the manifest. When operators download installers, the filename is the first brand impression and the primary search surface for file managers. Without an explicit filename, the build system may produce a generic name like `ninja-exec-1.0.0.msi` that lacks architecture identification, publisher branding, and search-friendly keywords.
- **Recommendation:** Add `msi_filename = "PlenumNET-NinjaExec-{version}-{arch}.msi"` to the `[install]` section, producing filenames like `PlenumNET-NinjaExec-1.0.0-x86_64.msi`. This is search-friendly, sorts well in download folders, and includes the architecture for disambiguation.
- **Impact:** Operators with multiple MSI files in a download folder cannot distinguish architectures or identify the product family at a glance.

### Finding 4
- **Section:** `main.rs` line 170 — Passphrase prompt label
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The passphrase prompt reads `"Enter passphrase (min 12 characters): "`. This communicates obligation, not value. It tells the operator what they must do, not why. The parenthetical minimum-length note reads as a bureaucratic constraint rather than a security assurance. Additionally, the prompt does not mention what the passphrase protects or that it will be needed again later.
- **Recommendation:** Change to `"Create a passphrase to protect your signing key (12+ characters): "`. This frames the action as protective (value) rather than mandatory (obligation), names what is being protected, and keeps the length guidance. The confirmation prompt at line 171 (`"Confirm passphrase: "`) is acceptable.
- **Impact:** Operators who feel the passphrase is a bureaucratic hoop rather than a protective measure may choose weak passphrases, undermining key security.

### Finding 5
- **Section:** `main.rs` lines 441–453 — Startup banner formatting
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The startup banner uses Unicode box-drawing characters (╔═║╚) which render correctly in modern terminals but may break in legacy consoles, Windows Event Log viewers, CI log aggregators, or when stdout is redirected to a file. The version line at line 443 uses a fixed-width layout (`v{}                                  `) that will misalign if the version string grows beyond 5 characters (e.g., `1.10.0`). The fingerprint line truncates to 47 characters with no indication that it is truncated.
- **Recommendation:** (a) Keep the box-drawing banner for interactive terminals but detect `--headless` mode or stdout redirection and emit a plain-text alternative. (b) Use dynamic width formatting instead of hardcoded spacing. (c) Show the full fingerprint or explicitly indicate truncation with `...`.
- **Impact:** Misaligned or garbled startup output in CI logs and event collectors gives the impression of an unpolished product. Truncated fingerprints without indication may confuse operators comparing values.

### Finding 6
- **Section:** `main.rs` line 308 — Unlock passphrase prompt
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The unlock passphrase prompt reads simply `"Passphrase: "` with no context about what is being unlocked or what happens next. An operator who has multiple PlenumNET tools may not know which passphrase is being requested.
- **Recommendation:** Change to `"NinjaExec passphrase: "` to disambiguate from other tools. The same applies to line 337 and line 431.
- **Impact:** In a multi-tool PlenumNET deployment, ambiguous prompts cause operator confusion and potential lockouts from entering the wrong passphrase.

### Finding 7
- **Section:** `keystore.rs` lines 44–56 — KeystoreError display messages
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** Several keystore error messages are technically accurate but lack actionability and warmth:
  - `"failed to generate random bytes"` — tells what happened but not what to do. An operator seeing this has no idea how to proceed.
  - `"passphrase cannot be empty"` — blame-adjacent phrasing ("you gave us nothing").
  - `"keystore file has invalid format"` — no next step. Is the file corrupted? Should they delete it? Contact support?
  - `"unsupported keystore KDF version: {}"` — exposes internal technical terminology (KDF) that operators will not understand.
  - `"decryption failed (wrong passphrase or corrupted keystore)"` — good dual-cause explanation, but **this message must be flagged for R1 Security Engineer co-review** because it distinguishes between passphrase failure and corruption, which could assist an attacker in narrowing the failure mode.
  - `"I/O error: {}"` — raw OS error passthrough with no context about what file or operation failed.
- **Recommendation:**
  - `EntropyFailure` → `"Unable to generate secure random data. Ensure your system's random number generator is available and try again."`
  - `EmptyPassphrase` → `"A passphrase is required to protect your signing key. Please provide one and try again."`
  - `InvalidFormat` → `"The keystore file appears damaged or was created by an incompatible version. If this problem persists, contact support or re-initialize with 'ninja-exec init'."`
  - `UnsupportedVersion` → `"This keystore was created with a newer version of NinjaExec. Please update NinjaExec to open it."`
  - `DecryptionFailed` → Flag for Security Engineer review. The current dual-cause message is acceptable from a copy perspective but must be verified as not leaking distinguishable failure modes.
  - `IoError` → `"Could not access the keystore file: {}. Check that the file exists and you have permission to read it."`
- **Impact:** Operators encountering these errors lose confidence in the product. Technical error messages without next steps cause support tickets and frustration.

### Finding 8
- **Section:** `keystore.rs` line 54 — `AlreadyExists` error message
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The `AlreadyExists` error displays as `"keystore already exists"` with no guidance. The operator does not know whether this is blocking, whether they should delete the old one, or how to do so safely.
- **Recommendation:** Change to `"A keystore already exists at this location. To create a new one, first back up and remove the existing keystore file."` This acknowledges the situation, provides a path forward, and implies caution about the existing key.
- **Impact:** Operators attempting to re-initialize are stuck without guidance, potentially leading to unsafe manual file deletion.

### Finding 9
- **Section:** `main.rs` lines 162–163 — Existing keystore warning
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The existing keystore message reads: `"[NinjaExec] Keystore already exists at {path}"` followed by `"[NinjaExec] To create a new keystore, remove the existing one first."` The instruction to "remove" a keystore containing cryptographic key material is cavalier. No mention of backup, no warning about the consequences of deletion, no mention of `export-operator` to save the public key first.
- **Recommendation:** Change second line to: `"[NinjaExec] To start fresh, first export your public key with 'ninja-exec export-operator', then securely delete the existing keystore."`.
- **Impact:** Operators may casually delete their keystore and lose their signing identity without understanding the consequences.

### Finding 10
- **Section:** `plenum-app.toml` line 45 — Uninstall preserve message
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The preserve message reads: `"Your NinjaExec signing key and audit history have been preserved in %APPDATA%\NinjaExec. If you plan to reinstall, these will be automatically detected."` Two issues: (a) `%APPDATA%` is an unexpanded environment variable displayed raw to the operator. The actual path (e.g., `C:\Users\Operator\AppData\Roaming\NinjaExec`) should be shown instead. (b) The tone is functional but cold. This is the last impression of the product — it should be warm and reassuring. (c) R2 finding R2-A6-9 notes that `APPDATA` (roaming) may be the wrong location; `LOCALAPPDATA` is more appropriate for a machine-local signing agent.
- **Recommendation:** Change to a template that expands the path at display time: `"Your NinjaExec signing key and audit history have been preserved in {resolved_data_dir}. If you reinstall NinjaExec, your existing identity will be automatically restored — no re-registration needed."` The second sentence transforms "detected" (clinical) into "restored" (reassuring) and adds the benefit (no re-registration).
- **Impact:** Operators see a raw environment variable they may not understand. The cold tone leaves a negative last impression that could influence future purchase decisions.

### Finding 11
- **Section:** `plenum-app.toml` — No full cleanup warning
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The manifest specifies `preserve_data = true` but provides no mechanism or copy for a full cleanup scenario. An operator decommissioning a machine needs to know: (a) what data remains after uninstall, (b) how to securely remove it, (c) what the consequences are (re-registration required, audit trail lost). No `destroy` or `wipe` command exists (per R2 finding R2-A6-2), and no guidance copy is provided for manual cleanup.
- **Recommendation:** Add a `cleanup_message` field to the `[uninstall]` section: `"To permanently remove all NinjaExec data including your signing key, delete the folder shown above. Warning: this action is irreversible. Your signing identity will need to be re-registered with your PlenumNET administrator."`.
- **Impact:** Operators decommissioning machines leave sensitive key material on disk because no guidance exists for complete removal.

### Finding 12
- **Section:** `main.rs` lines 34, 84, 101, 106 — Clipboard export messages
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The clipboard messages vary in tone and specificity: `"[NinjaExec] Operator identity copied to clipboard."` (lines 84, 101) is clear. `"[NinjaExec] Could not access clipboard. Output printed instead:"` (line 75) is acceptable but could be warmer. `"[NinjaExec] No clipboard utility found. Output printed instead:"` (line 106) is accurate but technical. The `plenum-app.toml` first_run clipboard message (line 34) is excellent: it identifies the recipient role and explains the purpose clearly.
- **Recommendation:** Standardize the fallback message to: `"[NinjaExec] Clipboard not available on this system. Your public signing key is printed below instead."` This removes the technical "utility" language and names what was printed.
- **Impact:** Minor inconsistency in messaging tone across clipboard paths.

### Finding 13
- **Section:** `server.rs` lines 171–174 — Rate limit error message
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The rate limit error reads `"Rate limit exceeded (max 30 requests/minute)"`. This exposes the exact rate limit threshold, which could help an attacker calibrate their request timing. Additionally, the message does not tell the operator what to do. **DEFERRED** because this section is governed by C3 (CORS) and C5 (no integration tests), and the error message copy will likely change during CRITICAL remediation.
- **Recommendation:** When un-deferred: change to `"Too many requests. Please wait a moment and try again."` Remove the exact threshold from the client-facing message; log the detail server-side only.
- **Impact:** Deferred — no impact on current verdict.

### Finding 14
- **Section:** `server.rs` lines 179–183 — Invalid context error message
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The error message lists all valid context strings: `"Unknown operation context; must be one of: sign, exec, model-swap, ..."`. This enumerates the full API surface to any caller, including potential attackers. **DEFERRED** because server.rs is governed by C3 and C5.
- **Recommendation:** When un-deferred: change to `"Unrecognized operation context. Check your request and try again."` Move the valid context list to documentation, not error responses.
- **Impact:** Deferred — no impact on current verdict.

### Finding 15
- **Section:** `server.rs` lines 288–291 — Keystore locked error
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The error reads `"Keystore is locked — unlock with passphrase first"`. The em dash is a nice touch, and the instruction is actionable. However, it tells an external caller exactly how to proceed to unlock the keystore, which may be undesirable for an endpoint that is supposed to be restricted. **DEFERRED** because server.rs is governed by C3 and C5.
- **Recommendation:** When un-deferred: review whether the unlock instruction should be present in API error responses visible to callers other than the operator.
- **Impact:** Deferred — no impact on current verdict.

### Finding 16
- **Section:** `server.rs` lines 519–522 — Unlock failed error
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The unlock error passes through `e.to_string()` directly, which will surface `KeystoreError` display strings (including `"decryption failed (wrong passphrase or corrupted keystore)"`) to the HTTP API caller. This should be reviewed in conjunction with Finding 7 regarding the `DecryptionFailed` message. **DEFERRED** because server.rs is governed by C3 and C5, and the error passthrough interacts with C4 (audit).
- **Recommendation:** When un-deferred: return a generic `"Unlock failed. Please check your passphrase."` to the API, log the specific error server-side. Flag for Security Engineer co-review to ensure no distinguishable failure modes leak through the HTTP layer.
- **Impact:** Deferred — no impact on current verdict.

### Finding 17
- **Section:** `main.rs` lines 185–193 — Init success output
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The init success flow prints the public key, fingerprint, and confirm token in rapid succession to stdout. Line 193 prints the confirm token value to stdout — this is the subject of C7. The public key (line 186) and fingerprint (line 187) are printed with `[NinjaExec]` prefix, which is consistent, but the volume of technical output during init may overwhelm operators. **DEFERRED** because this section is governed by C7 (token leak) and C2 (no Rep C — the init flow will change when Rep C is added).
- **Recommendation:** When un-deferred: after C7 is resolved, restructure the init success output to clearly separate the "what happened" section from the "what to do next" section. Show: (1) keystore created at path, (2) your signing identity fingerprint, (3) next step: run `ninja-exec export-operator --clipboard` to share your identity.
- **Impact:** Deferred — no impact on current verdict.

### Finding 18
- **Section:** `server.rs` lines 432–438 — /status endpoint response
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The `/status` endpoint returns `{"running": true, "locked": false, "uptime_secs": 1234, "signs_this_session": 5, "version": "1.0.0"}`. The field names are developer-centric, not operator-friendly. The response includes no product name, no fingerprint, and no human-readable uptime. An operator or management hub consuming this endpoint cannot display a meaningful status card without additional transformation. The `locked` field frames the state negatively when unlocked (`locked: false` rather than `ready: true`).
- **Recommendation:** Add `"product": "NinjaExec"`, `"algorithm": "TL-DSA-87"`, and a human-readable `"uptime": "2h 15m"` field alongside the machine-readable `uptime_secs`. Consider renaming `locked` to `ready` with inverted boolean for positive framing in management UIs. Retain the existing fields for backward compatibility and add the new ones.
- **Impact:** Management hub integrations must do extra work to present a polished status card. The `locked: false` framing creates a double-negative that operators must mentally parse.

### Finding 19
- **Section:** `main.rs` lines 456–458 — Headless mode warning
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The headless mode warning reads: `"WARNING: Headless mode — all signing requests will be auto-approved."` followed by `"Do not use in environments where browser tabs may be compromised."` The second line is oddly specific (browser tabs?) and may alarm operators without explaining the actual risk model. The warning also uses `WARNING:` in all caps which is appropriate for severity but the overall message lacks context about when headless mode IS appropriate.
- **Recommendation:** Change to: `"[NinjaExec] Running in headless mode. Signing requests will be approved automatically without operator confirmation. This mode is intended for CI/CD pipelines and automated environments only."` This explains the behavior, names the intended use case, and implicitly discourages inappropriate use without alarming language about compromised browser tabs.
- **Impact:** Operators may be confused by the browser-specific warning and unsure whether headless mode is safe for their use case.

### Finding 20
- **Section:** `main.rs` line 322 — Usage message for sign command
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The usage message reads `"Usage: ninja-exec sign <file>"` — bare and mechanical. Similarly, line 373 reads `"Usage: ninja-exec verify <file> <signature_b64>"`. These are the minimum viable help text with no description of what the command does.
- **Recommendation:** Change to `"Sign a file with your NinjaExec key.\nUsage: ninja-exec sign <file>"` and `"Verify a file signature.\nUsage: ninja-exec verify <file> <signature_b64>"`. One descriptive line before the usage syntax transforms a command error into a micro-tutorial.
- **Impact:** Operators who mistype a command get no orientation about what they were trying to do.

### Finding 21
- **Section:** `main.rs` line 151–154 — Version output
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The version output includes `"TL-DSA-87 (Level 5 post-quantum security)"` — excellent technical brand copy that positions the product. The copyright line is professional. However, there is no URL for more information, no short tagline, and no mention of the PlenumNET ecosystem that would help an operator understand what this tool is part of.
- **Recommendation:** Add a one-liner after the copyright: `"Part of the PlenumNET platform — https://plenumnet.com"`. This provides ecosystem context and a discoverable URL.
- **Impact:** The version output is a missed opportunity for ecosystem cross-promotion and operator orientation.

### Finding 22
- **Section:** `config.rs` lines 46–56 — Silent config fallback
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** When `ninja-exec.json` contains invalid JSON, `NinjaExecConfig::load()` silently returns defaults with no message to stderr. From a content perspective, this is a missing error message — the operator has no idea their configuration is being ignored. This is a trust-destroying silent failure: an operator who carefully configures their rate limits or confirmation policy is silently overridden by defaults.
- **Recommendation:** When the config file exists but fails to parse, emit: `"[NinjaExec] Warning: Configuration file at {path} could not be read. Using default settings. Fix the JSON syntax or delete the file to suppress this warning."` This is actionable, blame-free, and explains consequences.
- **Impact:** Operators who misconfigure their settings get silently overridden, potentially degrading their security posture without their knowledge.

### Finding 23
- **Section:** `plenum-app.toml` line 10 — Placeholder upgrade code
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The `upgrade_code` is visually a placeholder (`A1B2C3D4-E5F6-7890-ABCD-EF1234567890`). While this is primarily a technical issue (addressed by R1 finding I6 and R2 findings R2-A4-6, R2-A5-6, R2-A6-8), from a content perspective this placeholder appears in installer logs, Windows registry, and diagnostic tools. A clearly fake GUID undermines professional perception. **DEFERRED** because the upgrade code derivation depends on the product identity and potentially Rep C address (C2).
- **Recommendation:** When un-deferred: derive a proper GUID and ensure it does not look like a test value.
- **Impact:** Deferred — no impact on current verdict.

### Finding 24
- **Section:** `plenum-app.toml` [shortcuts] — Start menu entries
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The Start Menu shortcuts include `"NinjaExec"` and `"Export Public Key"`. The shortcuts are not grouped under a PlenumNET folder. If other PlenumNET products also create top-level Start Menu entries, the operator's Start Menu becomes cluttered. Additionally, `"Export Public Key"` is a technical label — an operator may not know what a public key is or why they would export it.
- **Recommendation:** Group under a `PlenumNET` Start Menu folder. Rename `"Export Public Key"` to `"Copy Signing Identity"` — this matches the clipboard export message in `plenum-app.toml` line 34 which correctly says "public signing key" and frames it as identity sharing. Alternatively: `"Share Signing Identity"`.
- **Impact:** Ungrouped Start Menu entries feel unprofessional for an enterprise suite. Technical shortcut labels deter non-technical operators from using available tools.

### Finding 25
- **Section:** `main.rs` lines 637–638 — Server startup messages
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The server startup messages read: `"[NinjaExec] Signing agent listening on {addr}"` and `"[NinjaExec] Bound to {} only — not accessible from network"`. The second line is excellent security assurance copy. However, both lines are printed via `println!` to stdout, which may not be captured when running as a Windows service. **DEFERRED** because the server module is governed by C3 and C5.
- **Recommendation:** When un-deferred: ensure startup messages are written to the audit log as well as stdout.
- **Impact:** Deferred — no impact on current verdict.

### Finding 26
- **Section:** `plenum-app.toml` — No update/migration messaging
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The manifest contains no `[update]` section. No copy exists for: update available notifications, what changed in the update, migration warnings (e.g., keystore format upgrade from v2 to v3), or rollback messaging. R2 finding R2-A5-2 notes that a keystore format migration will be needed for the Rep C address fix. When this migration happens, operators need clear messaging about what is changing and whether their existing key is preserved.
- **Recommendation:** Add an `[update]` section with template copy: `update_available_message = "A new version of NinjaExec is available ({new_version}). Your signing key and audit history will be preserved during the update."`, `migration_message = "NinjaExec is upgrading your keystore to a newer format. Your signing key is preserved. This is a one-time operation."`.
- **Impact:** When the inevitable keystore migration occurs, operators will receive no explanation about what is happening to their key material, causing anxiety and support tickets.

---

## Brand Score: 6 / 10

**Justification:** NinjaExec has a strong product name (memorable, action-oriented, distinctive), correct algorithm branding ("TL-DSA-87, Level 5 post-quantum security"), and several excellent copy moments (the `plenum-app.toml` clipboard export message, the "not accessible from network" server startup line, the `tray_tooltip`). The core technical copy is competent and mostly avoids blame. However, the product is weakened by: missing Add/Remove Programs URL fields (free brand real estate left blank), ungrouped Start Menu entries, unexpanded environment variables in uninstall copy, obligation-framed passphrase prompts, non-actionable error messages, a silent config fallback that could silently override operator intent, and no update/migration messaging for an inevitable format change. The em dash in the display name is a typographic risk. The overall impression is "technically competent developer tool" rather than "enterprise-grade platform component."

---

## Top 3 Quick Wins

1. **Add URL fields to `plenum-app.toml`** (Finding 2) — Three lines of TOML populate free brand real estate in every operator's Add/Remove Programs view. Zero code change, maximum visibility.

2. **Rewrite passphrase prompt from obligation to value** (Finding 4) — One string change transforms the first interactive moment from bureaucratic ("Enter passphrase, min 12 characters") to protective ("Create a passphrase to protect your signing key"). Sets the tone for the entire operator relationship.

3. **Add a config parse warning message** (Finding 22) — One `eprintln!` call in `config.rs` prevents the trust-destroying silent fallback to defaults. Operators who customize their config deserve to know when it is being ignored.

---

## Copy Audit Table

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

## DEFERRED Findings Summary

| Finding | Section | Reason Deferred |
|---------|---------|-----------------|
| 13 | `server.rs` rate limit error | Governed by C3, C5 |
| 14 | `server.rs` invalid context error | Governed by C3, C5 |
| 15 | `server.rs` keystore locked error | Governed by C3, C5 |
| 16 | `server.rs` unlock failed error passthrough | Governed by C3, C4, C5 |
| 17 | `main.rs` init success output restructuring | Governed by C2, C7 |
| 23 | `plenum-app.toml` placeholder upgrade code | Governed by C2 |
| 25 | `server.rs` startup messages to audit log | Governed by C3, C5 |

7 findings DEFERRED. These do not affect Summary Verdict or Brand Score.

---

## Review Complete

**Summary Verdict:** PASS WITH CONDITIONS

**Conditions:** Resolve the 8 non-deferred IMPORTANT findings (Findings 1, 2, 4, 7, 10, 18, 22 — product naming, URL fields, passphrase prompt, error message copy, uninstall copy, status endpoint, config fallback) before first product release. The 7 DEFERRED findings must be re-evaluated after R1/R2 CRITICAL remediation.

**Brand Score:** 6 / 10

**Finding Count:**
- CRITICAL: 0
- IMPORTANT: 8 (non-deferred)
- MINOR: 11 (non-deferred)
- DEFERRED: 7
- **Total: 26**
