# QC-R3 Consolidated Report — NinjaExec (Task #54)
# Round 3: Fit, Finish & Market Readiness

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Review Date:** 2026-03-28
**Protocol:** QC-R3 (Round 3 — Fit & Finish)
**Open R1/R2 CRITICALs:** 7 (C1–C7 unresolved)

---

## Reviewers

| Agent | Role | Verdict | Brand Score | Findings |
|-------|------|---------|-------------|----------|
| Agent 7 | Brand Guardian | **FAIL** | 3/10 | 2C + 3I + 4M + 3D = 12 |
| Agent 8 | UX Designer | PASS WITH CONDITIONS | 4/10 | 0C + 6I + 9M + 4D = 19 |
| Agent 9 | Content Creator | PASS WITH CONDITIONS | 6/10 | 0C + 8I + 11M + 7D = 26 |

**Brand Readiness Index:** (3 + 4 + 6) / 3 = **4.33 / 10**

> Brand Readiness Index below 6 triggers a design sprint before implementation per QC-R3 protocol.

---

## Aggregate Finding Count

| Severity | Count | Non-Deferred |
|----------|-------|-------------|
| CRITICAL | 2 | 2 |
| IMPORTANT | 17 | 17 |
| MINOR | 24 | 24 |
| DEFERRED | 14 | — |
| **Total** | **57** | **43** |

---

## Overall Verdict: FAIL

The Brand Guardian issued a FAIL verdict due to 2 CRITICAL findings (no color token system, no icon size specifications). The UX Designer and Content Creator both issued PASS WITH CONDITIONS. Per QC-R3 protocol, any agent FAIL results in an overall FAIL. The Brand Readiness Index of 4.33/10 independently triggers the design sprint gate.

**Resolution path:**
1. Resolve 2 R3 CRITICAL findings (color tokens, icon size specs)
2. Resolve 7 R1/R2 CRITICALs (C1–C7) to un-defer 14 findings
3. Resolve 17 IMPORTANT findings before release
4. Design sprint to raise Brand Readiness Index above 6.0

---

## CRITICAL Findings (2)

### R3-C1: No Color Token System
- **Agent:** 7 (Brand Guardian)
- **Section:** `plenum-app.toml` — no color system section
- **Finding:** No palette token list, color system, or design token specification exists. The SVG source uses 5 hardcoded hex values (`#181411`, `#0F0C0A`, `#272220`, `#F0EDE8`, `#4A9EF5`) with no named tokens. No status indicator colors are defined.
- **Recommendation:** Define a formal color token table mapping every hex value to a named token. Include token assignments for tray icon states (running, locked, stopped/error), installer dialog backgrounds, passphrase entry surface, and status indicator colors. Document WCAG contrast ratios.
- **Impact:** Without a palette token list, every implementer will choose ad-hoc colors. Tray icon, installer, and management panel will drift visually.

### R3-C2: No Icon Size Specifications
- **Agent:** 7 (Brand Guardian)
- **Section:** `plenum-app.toml` lines 8, 21 — icon references
- **Finding:** Three ICO files are referenced but no icon size requirements, size transition boundaries, minimum usable pixel count, or tray icon status rendering method are specified. The key-with-P glyph collapses to an illegible blob at 16x16 (tray icon's primary rendering size). Fine strokes (stroke-width 4-6 at 256x256 scale) alias badly at small sizes.
- **Recommendation:** (1) Document embedded sizes within each ICO. (2) Define detailed-to-simplified transition boundary (recommended: 32x32 and below). (3) Specify tray icon status rendering method. (4) Create a simplified 16x16 glyph variant.
- **Impact:** Tray icon is UNREADABLE at 16x16. Operator cannot distinguish NinjaExec from other tray icons. No visual feedback for locked/unlocked/error states.

---

## IMPORTANT Findings (17)

### R3-I1: No Tray Icon State Rendering
- **Agent:** 7 (Brand Guardian), Finding 3
- **Section:** `plenum-app.toml` `[app_type]`
- **Finding:** `tray_icon` and `tray_tooltip` are defined but no visual state map exists. NinjaExec has at least three observable states (running/unlocked, running/locked, stopped/error) with no visual differentiation. Tooltip is static.
- **Recommendation:** Define tray icon state map with overlay dots or icon swaps. Support dynamic tooltip text.

### R3-I2: No Typography Specification
- **Agent:** 7 (Brand Guardian), Finding 5
- **Section:** `plenum-app.toml` — no typography section
- **Finding:** No font specifications for any UI surface (installer dialogs, passphrase prompts, tray panel, error messages).
- **Recommendation:** Specify Segoe UI (Windows) / SF Pro (macOS) / system-ui (Linux) font stack. Specify monospace for passphrase entry.

### R3-I3: No Launcher Panel Specification
- **Agent:** 7 (Brand Guardian), Finding 6
- **Section:** `plenum-app.toml` `[app_type]`
- **Finding:** No tray agent panel design exists despite `kind = "tray_agent"`. No status display, fingerprint display, lock/unlock action, or audit tail panel.
- **Recommendation:** Specify a tray panel layout with header, status indicator, fingerprint, session stats, and action buttons.

### R3-I4: Passphrase Prompt Lacks Strength Feedback
- **Agent:** 8 (UX Designer), Finding 1
- **Section:** `plenum-app.toml` [first_run], `main.rs` line 170
- **Finding:** Passphrase prompt specifies `min_length = 12` with no strength indicator, no inline validation, no character counter. R1 Security Engineer established 72-bit entropy floor. Prompt frames passphrase as obligation rather than value.
- **Recommendation:** Add qualitative strength indicator (weak/fair/strong). Reframe prompt to communicate value being protected. Do NOT enumerate composition rules.

### R3-I5: No Progress Indicator During Init
- **Agent:** 8 (UX Designer), Finding 2
- **Section:** `main.rs` lines 167-177
- **Finding:** Init performs key generation, encryption, config creation with no spinner, progress bar, or phase messaging. On slower hardware or with increased KDF iterations, feels like a hang.
- **Recommendation:** Add phase-based progress messages to stderr. Use spinner if KDF takes >500ms.

### R3-I6: No Cancel/Rollback During Init
- **Agent:** 8 (UX Designer), Finding 3
- **Section:** `main.rs` lines 157-211
- **Finding:** Ctrl+C during init may leave partially written state. Config and token writes are not atomic. No cleanup of orphaned `.tmp` files.
- **Recommendation:** Wrap init in transaction pattern with atomic renames. Clean up on SIGINT.

### R3-I7: Passphrase Echo/Mismatch UX
- **Agent:** 8 (UX Designer), Finding 4
- **Section:** `main.rs` lines 34-59
- **Finding:** No echo suppression on Windows. Mismatch exits process instead of re-prompting.
- **Recommendation:** Implement Windows echo suppression. Re-prompt on mismatch instead of exit(1).

### R3-I8: Uninstall Flow Gaps
- **Agent:** 8 (UX Designer), Finding 7
- **Section:** `plenum-app.toml` [uninstall] lines 43-46
- **Finding:** `preserve_data = true` is the only option (no clean uninstall path). `%APPDATA%` shown raw (not expanded). No key destruction warning. No TDNS orphaning warning.
- **Recommendation:** Expand `%APPDATA%` to resolved path. Add `preserve_data = "prompt"` option. Add destruction warnings.

### R3-I9: Interactive Mode Confirmation Unusable
- **Agent:** 8 (UX Designer), Finding 15
- **Section:** `plenum-app.toml` [app_type], `main.rs`
- **Finding:** Tray agent UI declared but no implementation exists. Confirmation queue endpoints require external UI. No CLI fallback for approving/rejecting confirmations.
- **Recommendation:** Document that tray UI comes from PlenumNET Launcher. Add CLI fallback: `ninja-exec confirm list/approve/reject`. Add stderr notification on pending confirmation.

### R3-I10: Silent Config Parse Failures
- **Agent:** 8 (UX Designer), Finding 17 / Agent 9, Finding 22
- **Section:** `config.rs` lines 46-56
- **Finding:** Malformed JSON silently falls back to defaults. No `config validate` or `config show` command. No startup message indicating which config was loaded.
- **Recommendation:** Print error and exit on malformed JSON. Add `config show` command. Log config source on startup.

### R3-I11: Product Naming / Suite Clustering
- **Agent:** 9 (Content Creator), Finding 1
- **Section:** `plenum-app.toml` lines 1-9
- **Finding:** `display_name` uses em dash (—) which may mojibake on some Windows systems. Name starts with "N" rather than "P", preventing alphabetical clustering with other PlenumNET products.
- **Recommendation:** Change to `"PlenumNET NinjaExec - Signing Agent"` for Add/Remove Programs.

### R3-I12: Missing URL Fields
- **Agent:** 9 (Content Creator), Finding 2
- **Section:** `plenum-app.toml` — no URL fields
- **Finding:** No `help_url`, `about_url`, `update_url` fields. Windows Add/Remove Programs surfaces these as free brand real estate.
- **Recommendation:** Add `help_url`, `about_url`, `update_url` with HTTPS plenumnet.com links.

### R3-I13: Passphrase Prompt Obligation Framing
- **Agent:** 9 (Content Creator), Finding 4
- **Section:** `main.rs` line 170
- **Finding:** Prompt reads "Enter passphrase (min 12 characters):" — communicates obligation, not value. Does not mention what the passphrase protects.
- **Recommendation:** Change to "Create a passphrase to protect your signing key (12+ characters):".

### R3-I14: Keystore Error Message Copy (5 messages)
- **Agent:** 9 (Content Creator), Finding 7
- **Section:** `keystore.rs` lines 44-56
- **Finding:** Error messages are technically accurate but lack actionability: `EntropyFailure`, `EmptyPassphrase`, `InvalidFormat`, `UnsupportedVersion`, `IoError` all need rewriting. `DecryptionFailed` flagged for Security Engineer co-review.
- **Recommendation:** Rewrite all 5 messages with plain language, next steps, and blame-free tone. See Copy Audit Table in Agent 9 report.

### R3-I15: Uninstall Preserve Message Copy
- **Agent:** 9 (Content Creator), Finding 10
- **Section:** `plenum-app.toml` line 45
- **Finding:** Raw `%APPDATA%` shown. Cold tone for last product impression. "Detected" is clinical.
- **Recommendation:** Expand path variable. Rewrite: "...your existing identity will be automatically restored — no re-registration needed."

### R3-I16: /status Endpoint Not Operator-Friendly
- **Agent:** 9 (Content Creator), Finding 18
- **Section:** `server.rs` lines 432-438
- **Finding:** Response uses developer-centric field names, no product name, no fingerprint, no human-readable uptime. `locked: false` is a double-negative.
- **Recommendation:** Add `product`, `algorithm`, human-readable `uptime` fields. Consider `ready` instead of `locked`.

### R3-I17: No --help Flag
- **Agent:** 8 (UX Designer), Finding 10 (cross-referenced)
- **Section:** `cli.rs` lines 49-139
- **Finding:** No `--help` or `-h` flag. No arguments silently starts agent. Unknown subcommands silently start agent.
- **Recommendation:** Add `--help`/`-h`. Unknown subcommands should error. No-argument invocation should show usage or announce startup.

---

## MINOR Findings (24)

### R3-M1: No Animation/Transition Specification
- **Agent:** 7 (Brand Guardian), Finding 7
- **Finding:** Spec is silent on state change transitions. Acceptable for CLI/tray agent if explicitly documented.

### R3-M2: Accent Dot Color Unspecified
- **Agent:** 7 (Brand Guardian), Finding 8
- **Finding:** SVG accent dot (`#4A9EF5`) not mapped to any token. Purpose (status vs decoration) undefined. Invisible at 32x32 and below.

### R3-M3: Export-Key Icon Undocumented
- **Agent:** 7 (Brand Guardian), Finding 11
- **Finding:** `export-key.ico` has no SVG source, no documented design, no relationship to NinjaExec icon family.

### R3-M4: Windows-Only Uninstall Message
- **Agent:** 7 (Brand Guardian), Finding 12
- **Finding:** `preserve_message` uses `%APPDATA%\NinjaExec` — Windows-only path variable meaningless on Linux/macOS.

### R3-M5: Startup Banner UTF-8 Dependency
- **Agent:** 8 (UX Designer), Finding 5
- **Finding:** Unicode box-drawing characters garble on non-UTF-8 terminals. Fingerprint truncated to 47 chars without indication. Version spacing misaligns with longer version strings.

### R3-M6: Clipboard No Timeout
- **Agent:** 8 (UX Designer), Finding 6
- **Finding:** Clipboard subprocess has no timeout. Hangs indefinitely on headless systems with clipboard utilities installed but no display.

### R3-M7: Status Output Raw JSON
- **Agent:** 8 (UX Designer), Finding 8
- **Finding:** `ninja-exec status` prints raw JSON. No human-readable formatting. No actionable guidance when locked.

### R3-M8: Generic Connection Failure Messages
- **Agent:** 8 (UX Designer), Finding 9
- **Finding:** Error messages don't distinguish connection refused (not running) from timeout (starting up) from other failures.

### R3-M9: No Update Mechanism
- **Agent:** 8 (UX Designer), Finding 14
- **Finding:** No `[update]` section, no update check command, no version migration messaging, no rollback.

### R3-M10: No Sign Feedback
- **Agent:** 8 (UX Designer), Finding 16
- **Finding:** `ninja-exec sign` outputs base64 signature with no context — no file confirmation, no algorithm display, no verification hint.

### R3-M11: Accessibility Limited to CLI
- **Agent:** 8 (UX Designer), Finding 18
- **Finding:** No keyboard navigation specs for future tray UI. No screen reader compatibility addressed. Acceptable for CLI-only v1.

### R3-M12: MSI Filename Not Optimized
- **Agent:** 9 (Content Creator), Finding 3
- **Finding:** No `msi_filename` convention specified. Default may lack architecture identification and publisher branding.

### R3-M13: Unlock Prompt Ambiguous
- **Agent:** 9 (Content Creator), Finding 6
- **Finding:** Unlock prompt reads "Passphrase:" with no product disambiguation. Confusing in multi-tool deployments.

### R3-M14: Startup Banner Formatting
- **Agent:** 9 (Content Creator), Finding 5
- **Finding:** Box-drawing banner breaks in CI log aggregators and redirected stdout. Fixed-width layout misaligns. Fingerprint truncated silently.

### R3-M15: Clipboard Message Inconsistency
- **Agent:** 9 (Content Creator), Finding 12
- **Finding:** Clipboard fallback messages vary in tone. "No clipboard utility found" is technical.

### R3-M16: Existing Keystore Warning
- **Agent:** 9 (Content Creator), Finding 9
- **Finding:** Re-init message says "remove the existing one" — cavalier about key material. No mention of export or backup.

### R3-M17: Usage Messages Bare
- **Agent:** 9 (Content Creator), Finding 20
- **Finding:** Usage messages are bare "Usage: ninja-exec sign <file>" with no description of what the command does.

### R3-M18: Headless Mode Warning
- **Agent:** 9 (Content Creator), Finding 19
- **Finding:** Warning mentions "browser tabs" specifically. Doesn't explain intended use case (CI/CD).

### R3-M19: Version Output Missing Ecosystem Reference
- **Agent:** 9 (Content Creator), Finding 21
- **Finding:** Version output has no PlenumNET ecosystem reference or URL.

### R3-M20: AlreadyExists Error Bare
- **Agent:** 9 (Content Creator), Finding 8
- **Finding:** Error says "keystore already exists" with no guidance on next steps.

### R3-M21: Start Menu Shortcuts Not Grouped
- **Agent:** 9 (Content Creator), Finding 24
- **Finding:** Shortcuts not grouped under PlenumNET folder. "Export Public Key" is a technical label.

### R3-M22: No Update/Migration Messaging
- **Agent:** 9 (Content Creator), Finding 26
- **Finding:** No copy exists for update notifications, migration warnings, or rollback messaging.

### R3-M23: No Full Cleanup Warning
- **Agent:** 9 (Content Creator), Finding 11
- **Finding:** No guidance copy for complete data removal during decommissioning. No `destroy` or `wipe` command.

### R3-M24: Confirmation Queue No CLI Fallback
- **Agent:** 8 (UX Designer), Finding 15 (supplemental)
- **Finding:** No `ninja-exec confirm list/approve/reject` CLI commands exist for environments without tray UI.

---

## DEFERRED Findings (14)

| ID | Agent | Finding | Blocked By |
|----|-------|---------|-----------|
| D1 | A7-F4 | Startup banner UTF-8 in main.rs | C7 |
| D2 | A7-F9 | HTTP error response branding | C3, C4, C5, C6 |
| D3 | A7-F10 | CLI message prefix consistency | C7 |
| D4 | A8-F11 | CORS trust boundary UX | C3, C7 |
| D5 | A8-F12 | Audit health indicator | C4, C6 |
| D6 | A8-F13 | Signing context visibility | C1, C2 |
| D7 | A8-F19 | Signing progress feedback | C5 |
| D8 | A9-F13 | Rate limit error copy | C3, C5 |
| D9 | A9-F14 | Invalid context error copy | C3, C5 |
| D10 | A9-F15 | Keystore locked error copy | C3, C5 |
| D11 | A9-F16 | Unlock failed error passthrough | C3, C4, C5 |
| D12 | A9-F17 | Init success output restructuring | C2, C7 |
| D13 | A9-F23 | Placeholder upgrade code | C2 |
| D14 | A9-F25 | Server startup messages to audit | C3, C5 |

DEFERRED findings do not affect verdicts or Brand Scores.

---

## Readability Matrix (Agent 7)

| Icon File | Size | Rating | Notes |
|-----------|------|--------|-------|
| `ninja-exec.ico` | 256x256 | CLEAR | Key-with-P glyph fully legible |
| `ninja-exec.ico` | 48x48 | MARGINAL | P letterform ambiguous. Accent dot ~1.9px |
| `ninja-exec.ico` | 32x32 | MARGINAL | P letterform collapses. Accent dot invisible |
| `ninja-exec.ico` | 16x16 | UNREADABLE | ~12x12 usable pixels. All detail merges into blob |
| `ninja-exec-tray.ico` | 16x16 | UNREADABLE | Same issues. No simplified glyph variant |
| `export-key.ico` | 32x32 | UNVERIFIED | No SVG source or documentation |

---

## Friction Map Summary (Agent 8)

| Phase | Total Actions | SMOOTH | ACCEPTABLE | ROUGH |
|-------|--------------|--------|------------|-------|
| Install | 6 | 0 | 3 | 3 |
| Configure | 4 | 0 | 0 | 4 |
| Operate | 11 | 2 | 4 | 5 |
| Update | 3 | 0 | 0 | 3 |
| Uninstall | 4 | 0 | 1 | 3 |
| **Total** | **30** (incl. 2 not shown) | **2** | **8** | **18** |

18 of 30 operator actions rated ROUGH. Only 2 actions rated SMOOTH.

---

## Copy Audit Table (Agent 9)

See `ninja-exec/qc-r3-agent9-content-creator.md` for the complete Copy Audit Table with 18 copy revision entries covering every user-facing text instance.

**Key copy revisions:**
- 5 keystore error messages need rewriting (R3-I14)
- Passphrase prompt needs obligation-to-value reframe (R3-I13)
- Uninstall preserve message needs path expansion and warm tone (R3-I15)
- Config fallback needs warning message (R3-I10)
- /status endpoint needs product and algorithm fields (R3-I16)
- `DecryptionFailed` message flagged for Security Engineer co-review

---

## Top 3 Quick Wins (Cross-Agent)

1. **Create simplified 16x16 tray icon glyph** (R3-C2) — The tray icon is NinjaExec's primary visual surface and it's currently UNREADABLE. A simplified key silhouette with thicker strokes makes the product usable as a tray agent. ~2 hours design.

2. **Add `--help` flag and human-readable status output** (R3-I17, R3-M7) — Two changes, ~50 lines of code, that transform the CLI from "read the source" to "self-documenting tool." Highest UX impact for lowest effort.

3. **Add URL fields to `plenum-app.toml` and rewrite passphrase prompt** (R3-I12, R3-I13) — Three lines of TOML + one string change. Populates free brand real estate and transforms the first interactive moment from bureaucratic to protective.

---

## Design Sprint Gate

**Brand Readiness Index: 4.33 / 10** (threshold: 6.0)

The Brand Readiness Index is below 6.0. Per QC-R3 protocol, this triggers a **design sprint** before implementation. The design sprint must address:

1. Color token system definition
2. Icon size specifications and simplified glyph variants
3. Tray icon state rendering method
4. Typography specification
5. Launcher panel design (or explicit deferral with rationale)

The design sprint deliverable is a Brand Specification addendum to `plenum-app.toml` (or a separate `brand.toml`) that resolves R3-C1, R3-C2, R3-I1, R3-I2, and R3-I3.

---

## Individual Reports

| Report | Path |
|--------|------|
| Agent 7 — Brand Guardian | `ninja-exec/qc-r3-agent7-brand-guardian.md` |
| Agent 8 — UX Designer | `ninja-exec/qc-r3-agent8-ux-designer.md` |
| Agent 9 — Content Creator | `ninja-exec/qc-r3-agent9-content-creator.md` |
