# QC-R3 Agent 7 — Brand Guardian Review
# NinjaExec — PlenumNET Local Signing Agent v1.0.0 (Task #54)

**Reviewer:** Agent 7 — Brand Guardian
**YODA Role ID:** `design/brand-guardian`
**Review Date:** 2026-03-28
**Protocol:** QC-R3 (Round 3 — Fit & Finish)
**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0

**Source Documents Reviewed:**
| Document | Path |
|----------|------|
| Primary spec/manifest | `ninja-exec/plenum-app.toml` |
| Main entry point | `ninja-exec/src/main.rs` |
| HTTP server | `ninja-exec/src/server.rs` |
| Keystore | `ninja-exec/src/keystore.rs` |
| Configuration | `ninja-exec/src/config.rs` |
| Confirmation system | `ninja-exec/src/confirm.rs` |
| Audit log | `ninja-exec/src/audit.rs` |
| CLI parser | `ninja-exec/src/cli.rs` |
| Signing engine | `ninja-exec/src/signing_engine.rs` |
| Cargo manifest | `ninja-exec/Cargo.toml` |
| SVG icon source | `assets/icons/svg/ninja-exec.svg` |
| ICO assets | `ninja-exec/assets/ninja-exec.ico`, `ninja-exec/assets/ninja-exec-tray.ico`, `ninja-exec/assets/export-key.ico` |
| R1 consolidated findings | `ninja-exec/qc-r1-consolidated.md` |
| R2 consolidated findings | `ninja-exec/qc-r2-consolidated.md` |

**Open R1/R2 CRITICAL Findings (7):**
| ID | Summary | Affected Sections |
|----|---------|-------------------|
| C1 | No Rep C address binding in TL-DSA signatures | `signing_engine.rs` |
| C2 | No Rep C address exists anywhere in codebase | Entire codebase |
| C3 | CORS wildcard origin creates signature oracle | `server.rs` |
| C4 | Audit log silently swallows all write failures | `audit.rs`, `server.rs` |
| C5 | No integration tests for any HTTP endpoint | `server.rs` |
| C6 | Audit entries lack Rep C addresses | `audit.rs`, `server.rs` |
| C7 | Confirm token printed to stdout | `main.rs`, `config.rs` |

Per QC-R3 sequencing rules, findings against sections with open CRITICALs are marked **DEFERRED**.

---

## Findings

### Finding 1
- **Section:** `plenum-app.toml` — no color system section
- **Severity:** CRITICAL
- **Round:** R3
- **Finding:** No palette token list, color system, or design token specification exists anywhere in the NinjaExec specification or source documents. The `plenum-app.toml` manifest defines no color tokens for the tray icon, installer dialogs, status indicators, or any other UI surface. The SVG source (`assets/icons/svg/ninja-exec.svg`) uses hardcoded hex values — `#181411`, `#0F0C0A`, `#272220`, `#F0EDE8`, `#4A9EF5` — with no mapping to named palette tokens. Per the Brand Guardian review scope, if the spec does not contain a palette token list, this is CRITICAL.
- **Recommendation:** Define a formal NinjaExec color token table mapping every hex value to a named token (e.g., `brand-bg-dark: #181411`, `brand-stroke: #272220`, `brand-glyph: #F0EDE8`, `brand-accent: #4A9EF5`). Include token assignments for: tray icon states (running, locked, stopped/error), installer dialog backgrounds, passphrase entry surface, status indicator colors. Document light-mode derivation rules and WCAG contrast ratios for each pairing.
- **Impact:** Without a palette token list, every implementer will choose ad-hoc colors. The tray icon, installer, and any future management panel will drift from each other visually. Status indicator colors will be inconsistent across surfaces.

### Finding 2
- **Section:** `plenum-app.toml` lines 8, 21 — icon references `ninja-exec.ico`, `ninja-exec-tray.ico`
- **Severity:** CRITICAL
- **Round:** R3
- **Finding:** The spec references three ICO files (`ninja-exec.ico`, `ninja-exec-tray.ico`, `export-key.ico`) but specifies no icon size requirements, no size transition boundaries (where detailed rendering switches to simplified), no minimum usable pixel count, and no tray icon status rendering method. The ICO files exist on disk (4250, 4321, and 5699 bytes respectively) but the spec does not document what sizes are embedded within them, what simplification rules apply at small sizes, or how tray icon state changes (running/locked/stopped) are visually communicated. Per the Brand Guardian review scope, absent icon size requirements are CRITICAL.
- **Recommendation:** (1) Document embedded sizes within each ICO file (e.g., 256x256, 48x48, 32x32, 16x16). (2) Define the detailed-to-simplified transition boundary (recommended: 32x32 and below use a simplified glyph). (3) Specify the tray icon status rendering method — choose one of: overlay dot, full icon swap, tint shift, or ring indicator. (4) Verify the key-with-P glyph from the SVG source is distinguishable at 16x16 (approximately 12x12 usable pixels after OS padding). The current SVG design has fine strokes (stroke-width 4-6 at 256x256 scale) that will alias badly at 16x16.
- **Impact:** The key glyph with internal "P" letterform at stroke-width 4 (1.5% of viewBox) will collapse into an illegible blob at 16x16. The operator cannot distinguish NinjaExec from other tray icons. No visual feedback for locked/unlocked/error states in the system tray.

### Finding 3
- **Section:** `plenum-app.toml` `[app_type]` section — tray icon status rendering
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The `[app_type]` section defines `tray_icon = "assets/ninja-exec-tray.ico"` and `tray_tooltip = "NinjaExec — PlenumNET Signing Agent"` but provides no specification for how the tray icon communicates agent state. NinjaExec has at least three observable states: running/unlocked, running/locked, and stopped/error. The tooltip is static text with no state interpolation. No status rendering method is defined (overlay dot, icon swap, tint, ring). The `status_port = 21027` implies programmatic status queries but no visual state mapping exists.
- **Recommendation:** Define a tray icon state map: (1) Running/Unlocked — default icon or green overlay dot. (2) Running/Locked — amber overlay dot or desaturated icon variant. (3) Stopped/Error — red overlay dot or "X" overlay. (4) Update `tray_tooltip` spec to support dynamic text (e.g., "NinjaExec — Unlocked | 14 signs this session"). (5) Specify whether the "blackout" stopped indicator meets the 1.5:1 minimum contrast ratio for perceivability.
- **Impact:** Operators have no way to glance at the system tray and know whether NinjaExec is unlocked, locked, or in error state. They must use `ninja-exec status` CLI or HTTP query — defeating the purpose of a tray agent.

### Finding 4
- **Section:** `main.rs` lines 441-453 — startup banner
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The startup banner uses Unicode box-drawing characters (`╔═╗║╠╚`) for a branded ASCII art panel. While visually distinctive in UTF-8 terminals, no specification exists for: (a) the font or encoding requirements for correct rendering, (b) fallback behavior in non-UTF-8 terminals or log files, (c) whether the banner width (52 chars) fits common terminal widths, (d) whether the fingerprint truncation at 47 chars (`&fp[..47]`) produces a complete or misleading display. This finding is DEFERRED because `main.rs` is affected by open CRITICAL C7 (token printed to stdout during init).
- **Recommendation:** Document terminal encoding requirement (UTF-8). Provide an ASCII-only fallback banner for non-UTF-8 environments. Verify the 47-char fingerprint slice does not cut a colon-separated hex pair mid-byte.
- **Impact:** In non-UTF-8 terminals (Windows cmd.exe with legacy code pages), the banner renders as mojibake, creating a poor first impression of a security tool.

### Finding 5
- **Section:** `plenum-app.toml` — no typography specification
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** No font specifications exist for any UI surface. The `plenum-app.toml` manifest defines no font family, weight, or size for: installer wizard dialogs, passphrase entry prompts, the first-run message (line 34), the uninstall preservation message (line 45), error messages, or the tray agent tooltip/panel. CLI output in `main.rs` uses the system terminal font by default, but installer dialogs require explicit font choices to maintain brand consistency.
- **Recommendation:** Define a typography specification covering: (1) Installer dialog body text — font family, size, weight. (2) Installer dialog headings — font family, size, weight. (3) Passphrase entry field — monospace font recommendation for character counting. (4) Error/warning message styling. (5) Tray panel text (if a panel UI is planned). Reference the Capomastro brand font stack or specify fallback system fonts (e.g., Segoe UI on Windows, SF Pro on macOS, system-ui on Linux).
- **Impact:** Without typography specs, the installer will use WiX/MSI default fonts (MS Shell Dlg), which are functional but carry no brand identity. The passphrase entry experience will look generic.

### Finding 6
- **Section:** `plenum-app.toml` `[first_run]` and `[shortcuts]` — no launcher panel specification
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** No tray agent panel design specification exists. The `[app_type]` section establishes NinjaExec as a `tray_agent` with `autostart = true`, implying a persistent tray presence, but no compact panel design is specified. A tray agent signing service needs at minimum: (a) current state display (locked/unlocked), (b) fingerprint display, (c) recent signing activity count, (d) lock/unlock action, (e) recent audit tail. None of these panel elements are specified. The `configure_command = ""` field is empty, suggesting no configuration UI exists.
- **Recommendation:** Specify a tray panel layout with: (1) Header showing NinjaExec branding and version. (2) Status indicator (locked/unlocked) using the defined palette tokens. (3) Fingerprint display (truncated with copy action). (4) Session statistics (signs this session, uptime). (5) Lock/Unlock button. (6) "View Audit Log" link. (7) Panel dimensions and whether it uses custom SVG glyphs or system icons. Verify the panel fits the "compact panel" goal without crowding.
- **Impact:** Without a panel spec, the tray icon becomes a silent sentinel with no interactive surface. Operators must use CLI commands for every interaction, undermining the UX value of a tray agent.

### Finding 7
- **Section:** `plenum-app.toml` — no animation/transition specification
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The specification is silent on animation and transition behavior for state changes. NinjaExec has observable state transitions: unlocked-to-locked, locked-to-unlocked, idle-to-signing, signing-to-complete. The spec does not document whether these transitions are instantaneous (acceptable if explicitly stated) or animated. The silence means implementers have no guidance.
- **Recommendation:** Add a brief "Transitions" section to the spec stating either: (a) "All state changes are instantaneous — no animation" (acceptable design choice), or (b) specify transition behavior (e.g., tray icon pulse on sign completion, brief color flash on lock/unlock). Document the choice explicitly so implementations are consistent.
- **Impact:** Low impact for a CLI/tray agent. If a panel UI is later added, undocumented transitions will produce inconsistent behavior across platforms.

### Finding 8
- **Section:** `assets/icons/svg/ninja-exec.svg` — accent dot color specification
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The SVG icon includes an accent dot at position (200, 200) with fill `#4A9EF5` (a blue accent). This blue is not referenced in any token system, not documented as a brand color, and its purpose is undefined. It appears to be a status indicator or brand accent, but its meaning is unspecified. If it represents "running" status, the stopped/locked variants need corresponding accent colors. If it is purely decorative, its position in the lower-right quadrant may be cropped at small icon sizes.
- **Recommendation:** (1) Assign this color to a named token (e.g., `accent-active: #4A9EF5`). (2) Document whether the dot represents status or decoration. (3) If status: define variant colors for locked (amber) and stopped (red/grey). (4) Verify the dot remains visible at 32x32 and below (at 256px viewBox, the 10px radius dot scales to ~1.25px at 32x32 — invisible).
- **Impact:** The accent dot disappears at small sizes, removing whatever meaning it carries. Without token assignment, the blue may conflict with other PlenumNET product accent colors.

### Finding 9
- **Section:** `server.rs` — HTTP error response branding
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** HTTP API error responses use generic JSON structures with `code` and `error` fields (e.g., `"code": "RATE_LIMITED"`, `"error": "Rate limit exceeded"`). While functional, the error messages carry no NinjaExec branding, no consistent error code prefix (some use `RATE_LIMITED`, others `KEYSTORE_LOCKED`, `CONFIRMATION_REJECTED`), and no version or product identifier in the response body. The error code namespace is not documented. This finding is DEFERRED because `server.rs` is affected by open CRITICALs C3, C4, C5, C6.
- **Recommendation:** (1) Prefix all error codes with `NX_` (e.g., `NX_RATE_LIMITED`, `NX_KEYSTORE_LOCKED`). (2) Add a `product` and `version` field to error responses for client diagnostics. (3) Document the complete error code table.
- **Impact:** Without branded error codes, client applications cannot distinguish NinjaExec errors from other localhost services. Debugging becomes harder in multi-agent environments.

### Finding 10
- **Section:** `main.rs` lines 75, 84, 101, 106 — CLI message prefix consistency
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** CLI output messages use the prefix `[NinjaExec]` consistently across all user-facing messages. This is good brand practice. However, the prefix format is not specified in `plenum-app.toml` or any design document — it is an implementation convention only. The startup banner (lines 441-453) uses a different visual treatment (box-drawing frame with "NinjaExec — PlenumNET Signing Agent" centered). The two presentation styles are not reconciled. This finding is DEFERRED because `main.rs` is affected by open CRITICAL C7.
- **Recommendation:** Document the CLI output prefix convention (`[NinjaExec]`) in the spec. Reconcile the banner style with the message prefix style. Consider whether the banner should use the same `[NinjaExec]` prefix or whether the framed banner is the intentional "launch" presentation.
- **Impact:** Minor inconsistency between startup banner and subsequent messages. Low user impact but indicates undocumented design decisions.

### Finding 11
- **Section:** `plenum-app.toml` lines 38-41 — shortcut icon references
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The `[shortcuts]` section references `assets/export-key.ico` for the "Export Public Key" shortcut. This icon file exists (5699 bytes) but has no SVG source in `assets/icons/svg/`, no documented design, and no relationship to the NinjaExec icon family documented anywhere. The main icon and tray icon share a visual language (key glyph with P element), but the export-key icon's design is unspecified.
- **Recommendation:** (1) Create or document the SVG source for `export-key.ico`. (2) Ensure the export-key icon uses the same color tokens and visual language as the main NinjaExec icon (key glyph family). (3) Verify the export-key icon is distinguishable from the main icon at Start Menu sizes (typically 32x32 or 48x48).
- **Impact:** The export-key shortcut icon may look unrelated to NinjaExec in the Start Menu, confusing operators about which application it belongs to.

### Finding 12
- **Section:** `plenum-app.toml` line 45 — uninstall preservation message
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The `preserve_message` text uses `%APPDATA%\\NinjaExec` which is a Windows-specific path variable. The spec defines `architecture = ["aarch64", "x86_64"]` which could target Linux/macOS. No platform-specific message variants exist. On non-Windows platforms, the `%APPDATA%` reference would be meaningless to the operator.
- **Recommendation:** Define platform-specific preservation messages or use a generic phrasing (e.g., "Your NinjaExec signing key and audit history have been preserved in your application data directory").
- **Impact:** On non-Windows platforms, the uninstall message references a path that does not exist, confusing operators about where their key material is stored.

---

## Readability Matrix

| Product | Icon File | Estimated Size | Rating | Notes |
|---------|-----------|----------------|--------|-------|
| NinjaExec | `ninja-exec.ico` | 256x256 | CLEAR | Key-with-P glyph fully legible. Gradient background, accent dot visible. |
| NinjaExec | `ninja-exec.ico` | 48x48 | MARGINAL | Key shaft and teeth distinguishable. P letterform inside key bow becomes ambiguous. Accent dot ~1.9px — barely visible. |
| NinjaExec | `ninja-exec.ico` | 32x32 | MARGINAL | Key silhouette recognizable. P letterform inside bow collapses. Accent dot ~1.25px — invisible. Stroke-width 4 at source scale = ~0.5px rendered — subpixel aliasing. |
| NinjaExec | `ninja-exec.ico` | 16x16 | UNREADABLE | ~12x12 usable pixels after OS padding. Key bow circle, P letterform, shaft, and teeth all merge into indistinct blob. Fine strokes (source stroke-width 4-6) alias to subpixel noise. Icon is not identifiable as NinjaExec. |
| NinjaExec | `ninja-exec-tray.ico` | 16x16 (tray) | UNREADABLE | System tray icons render at 16x16 (Windows) or 22x22 (macOS menu bar). At 16x16, same issues as above. No simplified glyph variant specified for tray size. |
| NinjaExec | `export-key.ico` | 32x32 (Start Menu) | UNVERIFIED | No SVG source or design documentation available. Cannot assess readability without knowing the glyph design. |

**Note:** Size ratings are assessed based on the SVG source geometry and stroke specifications, as the spec does not document embedded ICO sizes or simplification rules. The ICO files exist but their embedded size matrix is not documented.

---

## Brand Score: 3 / 10

**Rationale:** NinjaExec has a recognizable icon concept (key with P letterform) and consistent CLI message prefixing (`[NinjaExec]`), but lacks almost every specification required for brand implementation: no color token system, no icon size requirements, no tray status rendering, no typography specification, no launcher panel design, no transition documentation. The SVG icon design will fail at the most critical rendering size (16x16 tray icon). The spec is functional as a build/install manifest but is not a brand specification.

---

## Top 3 Quick Wins

1. **Define a tray icon state map with simplified 16x16 glyph** — Create a simplified key silhouette (no internal P letterform, thicker strokes) for 32x32 and below. Define overlay dot colors for running/locked/stopped states. This single change makes NinjaExec usable as a tray agent. Effort: ~2 hours design + spec update.

2. **Create a color token table** — Extract the 5 hex values from the SVG into named tokens. Add status colors (green/amber/red for running/locked/error). Map every UI element to a token. This prevents color drift across all surfaces. Effort: ~1 hour documentation.

3. **Add a typography section to `plenum-app.toml`** — Specify Segoe UI (Windows) / SF Pro (macOS) / system-ui (Linux) as the font stack for installer dialogs. Specify monospace font for passphrase entry. This ensures installers carry minimal brand consistency. Effort: ~30 minutes documentation.

---

## DEFERRED Findings Summary

Three findings were marked DEFERRED due to open R1/R2 CRITICALs:

| Finding | Reason Deferred |
|---------|-----------------|
| Finding 4 (Startup banner) | `main.rs` affected by C7 (token to stdout) |
| Finding 9 (Error response branding) | `server.rs` affected by C3, C4, C5, C6 |
| Finding 10 (CLI prefix consistency) | `main.rs` affected by C7 |

DEFERRED findings do not affect the Summary Verdict or Brand Score.

---

## Review Complete

**Summary Verdict:** FAIL

**Brand Score:** 3 / 10

**Finding Count:**
| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 2 | F1 (no color system), F2 (no icon size spec) |
| IMPORTANT | 3 | F3 (no tray status rendering), F5 (no typography), F6 (no launcher panel) |
| MINOR | 4 | F7 (no transition spec), F8 (accent dot unspecified), F11 (export-key icon undocumented), F12 (Windows-only uninstall message) |
| DEFERRED | 3 | F4, F9, F10 |
| **Total** | **12** | |

**Non-DEFERRED findings:** 9 (2 CRITICAL, 3 IMPORTANT, 4 MINOR)
**DEFERRED findings (excluded from verdict):** 3

Two CRITICAL brand findings block implementation: the absence of any color token system and the absence of icon size specifications. NinjaExec's tray icon will be unreadable at its primary rendering size (16x16). The specification is a functional build manifest but does not constitute a brand specification.
