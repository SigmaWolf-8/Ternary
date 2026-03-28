# QC-R3 Agent 8 Review — UX Designer

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Review Date:** 2026-03-28
**Protocol:** QC-R3 (Round 3 — Fit & Finish)
**Reviewer:** Agent 8 — UX Designer
**YODA Role ID:** `design/ux-designer`
**R1 Input:** ninja-exec/qc-r1-consolidated.md
**R2 Input:** ninja-exec/qc-r2-consolidated.md

**Source Documents Reviewed:**
- `ninja-exec/plenum-app.toml` — Primary spec/manifest
- `ninja-exec/src/main.rs` — CLI entry point, init flow, passphrase prompt
- `ninja-exec/src/server.rs` — HTTP API, status endpoint, tray confirmation endpoints
- `ninja-exec/src/keystore.rs` — Keystore creation, passphrase validation
- `ninja-exec/src/config.rs` — Configuration loading, default generation
- `ninja-exec/src/confirm.rs` — Confirmation queue, approval flow
- `ninja-exec/src/audit.rs` — Audit log
- `ninja-exec/src/cli.rs` — Argument parsing, subcommands
- `ninja-exec/src/signing_engine.rs` — TL-DSA signing

**Open R1/R2 CRITICALs (7):** C1 (Rep C in signatures), C2 (Rep C absent), C3 (CORS wildcard), C4 (audit silent failures), C5 (no HTTP integration tests), C6 (audit entries lack Rep C), C7 (confirm token printed to stdout). Findings touching sections with open CRITICALs are marked DEFERRED per sequencing constraint.

**R1 Passphrase Entropy Minimum:** 72 bits (extracted from Agent 1 Passphrase Entropy Assessment, qc-r1-consolidated.md).

---

## Findings

### Finding 1
- **Section:** `plenum-app.toml` [first_run] actions, line 28
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The passphrase prompt specifies `min_length = 12` with no strength indicator, no inline validation feedback, and no character counter. The R1 Security Engineer established `passphrase_entropy_minimum_bits = 72`, which requires either longer passphrases or composition guidance. The prompt label in `main.rs` line 170 reads "Enter passphrase (min 12 characters): " — this frames passphrase creation as an obligation rather than communicating the value being protected. There is no qualitative strength indicator (weak/fair/strong), no guidance toward stronger passphrase strategies, and no indication of what the passphrase protects.
- **Recommendation:** (a) Reframe the prompt label to communicate value: "Create a passphrase to protect your signing key (min 12 characters): ". (b) Add an inline character counter showing current length vs minimum. (c) Add a qualitative strength indicator (weak/fair/strong) based on entropy estimation — do NOT enumerate specific composition rules, as this constrains brute-force search space per the skill protocol. (d) If the passphrase meets minimum length but falls below 72-bit estimated entropy, display a qualitative "fair" rating with a suggestion to use a longer or more varied passphrase.
- **Impact:** Operators create weak passphrases because the prompt provides no feedback on passphrase quality, leaving signing keys protected below the 72-bit security floor.

### Finding 2
- **Section:** `plenum-app.toml` [first_run] actions, `main.rs` lines 167-177
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The first-run flow has no progress indicator. The `init` command performs key generation, keystore encryption, config file creation, and confirm token generation — operations that could take variable time depending on KDF cost (and will take longer if I1 KDF iteration increase is applied). No spinner, progress bar, or phase messaging is shown. The operator sees the passphrase prompts, then silence until completion or error. On slower hardware or with increased KDF iterations, this could feel like a hang.
- **Recommendation:** Add phase-based progress messages to stderr during `init`: "Generating TL-DSA-87 keypair...", "Encrypting keystore...", "Keystore created successfully." Use a spinner or elapsed-time indicator if KDF takes >500ms.
- **Impact:** Operators on slower hardware will assume the process has frozen and may interrupt key generation, potentially leaving a partially written keystore.

### Finding 3
- **Section:** `plenum-app.toml` [first_run] actions, `main.rs` lines 157-211
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The `init` flow has no cancel/rollback behavior. If the operator enters a passphrase and then presses Ctrl+C during key generation or encryption, the keystore may be left in a partially written state. While atomic rename (`keystore.tmp` -> `keystore`) mitigates file corruption, the config file write at line 189 and token generation at line 190 are not atomic. An interrupted init could leave a valid keystore but no config file, or vice versa.
- **Recommendation:** (a) Wrap the entire init sequence in a transaction pattern: create all files as `.tmp`, then rename all atomically. (b) On SIGINT during init, clean up any `.tmp` files. (c) On next `init` attempt, detect and clean up orphaned `.tmp` files with a message explaining what happened.
- **Impact:** Interrupted first-run leaves the data directory in an inconsistent state that the operator cannot diagnose or recover from without manual file inspection.

### Finding 4
- **Section:** `main.rs` lines 34-59 (prompt_passphrase function)
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The passphrase prompt has no show/hide toggle. Terminal echo is disabled on Unix via `tcsetattr`, which is correct for security, but there is no option for the operator to verify what they typed before confirming. The confirmation prompt ("Confirm passphrase:") catches mismatches but provides no mechanism to see the input. On Windows, no echo suppression is implemented at all — the passphrase is typed in cleartext visible on screen.
- **Recommendation:** (a) On Windows, implement echo suppression using the Windows Console API (`SetConsoleMode` to disable `ENABLE_ECHO_INPUT`). (b) Consider a `--show-passphrase` flag for environments where screen capture is not a concern (documented as reducing security). (c) On mismatch, display "Passphrases do not match. Please try again." and re-prompt rather than exiting with code 1.
- **Impact:** On Windows, passphrase is visible to shoulder surfers. On all platforms, a passphrase typo during init requires restarting the entire init process from scratch.

### Finding 5
- **Section:** `main.rs` lines 441-453 (Run command startup banner)
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The startup banner uses Unicode box-drawing characters (double-line: `═`, `║`, `╔`, `╗`, `╠`, `╣`, `╚`, `╝`) which render incorrectly on terminals without UTF-8 support (e.g., Windows `cmd.exe` with default OEM code page, remote SSH sessions with misconfigured locale). The fingerprint is truncated to 47 characters at line 445 (`&fp[..47]`) with no visual indication of truncation. The version string at line 443 has hardcoded spacing that will misalign if the version exceeds `1.0.0` (e.g., `1.10.0`).
- **Recommendation:** (a) Detect terminal encoding and fall back to ASCII box-drawing (`+`, `-`, `|`) on non-UTF-8 terminals. (b) Show the full fingerprint or use an ellipsis to indicate truncation. (c) Use dynamic padding for the version string.
- **Impact:** Garbled startup banner on legacy terminals undermines professional appearance and makes fingerprint verification unreliable.

### Finding 6
- **Section:** `plenum-app.toml` [first_run] action 3 (copy_to_clipboard), `main.rs` lines 66-108
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The clipboard copy feedback is inconsistent across platforms. On success, the message is "[NinjaExec] Operator identity copied to clipboard." On failure (no clipboard utility on Unix), it falls back to printing the output with "[NinjaExec] No clipboard utility found. Output printed instead:" — this is a good fallback, but there is no timeout on the clipboard process, no "Copied!" transient state, and no indication of how long the clipboard content will persist. The `copy_to_clipboard` function in `main.rs` spawns child processes for clipboard utilities without timeout.
- **Recommendation:** (a) Add a 5-second timeout on clipboard subprocess execution. (b) On success, add a note: "Paste it into your PlenumNET administrator's operator registration form." (c) On headless/SSH environments, detect lack of display and skip clipboard attempt immediately with a clear message.
- **Impact:** Clipboard subprocess hangs indefinitely on environments where xclip/xsel are installed but no X display is available. Operator waits forever.

### Finding 7
- **Section:** `plenum-app.toml` [uninstall] lines 43-46
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The uninstall flow has critical UX gaps: (a) `preserve_data = true` is the only option — there is no `preserve_data = "prompt"` or `false` option, meaning the operator can never perform a clean uninstall through the installer. (b) The `preserve_message` uses the literal string `%APPDATA%\NinjaExec` which is not expanded at display time — the operator sees the raw environment variable, not the resolved path. (c) There is no warning about permanent loss of the private signing key if data is not preserved and no backup exists. (d) There is no warning about TDNS name orphaning if the signing key is destroyed without transferring ownership. (e) The preserve path is shown as static dialog text, not as copyable text.
- **Recommendation:** (a) Expand `%APPDATA%` to the actual resolved path in the uninstall dialog. (b) Show the preserve path as selectable/copyable text. (c) Add explicit warnings: "Your NinjaExec signing key will be preserved. If you need to permanently remove it, manually delete [resolved path]." (d) Add TDNS orphaning warning: "If this key is registered with TDNS names, those names will become unreachable if the key is destroyed without transferring ownership." (e) Add a `preserve_data = "prompt"` option that gives the operator a choice with appropriate warnings for each path.
- **Impact:** Operators who need a clean uninstall (e.g., decommissioning a machine) have no supported path. Operators see raw `%APPDATA%` and cannot navigate to the actual directory.

### Finding 8
- **Section:** `server.rs` lines 424-439 (handle_status), `cli.rs` (status command)
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The `/status` endpoint returns `running`, `locked`, `uptime_secs`, `signs_this_session`, and `version`. The `ninja-exec status` CLI command prints the raw JSON response with no formatting. An operator checking agent health sees a JSON blob with no human-readable interpretation. There is no indication of what "locked" means operationally (signing disabled until passphrase is provided) or what to do about it.
- **Recommendation:** (a) Format `ninja-exec status` output as a human-readable summary: "NinjaExec v1.0.0 | Status: UNLOCKED | Uptime: 2h 15m | Signatures: 47". (b) When locked, append: "Agent is locked. Run 'ninja-exec unlock' to enable signing." (c) Keep `--json` flag for machine-readable output.
- **Impact:** Operators who run `ninja-exec status` during troubleshooting must mentally parse JSON instead of getting an immediate health assessment.

### Finding 9
- **Section:** `main.rs` lines 281-290 (Status command), lines 292-318 (Lock/Unlock commands)
- **Severity:** MINOR
- **Round:** R3
- **Finding:** Error messages for connection failures are generic: "Failed to reach agent on port {}: {}". This does not distinguish between "agent is not running" (connection refused), "agent is starting up" (connection timeout), or "wrong port" (connection refused on a different port). The operator cannot determine the root cause from the error message.
- **Recommendation:** Match on the error type: connection refused -> "NinjaExec is not running on port {}. Start it with 'ninja-exec run'."; timeout -> "NinjaExec on port {} is not responding. Check if the agent is starting up or overloaded."; other -> "Could not connect to NinjaExec on port {}: {}".
- **Impact:** Operators waste time troubleshooting the wrong problem because the error message does not distinguish between common failure modes.

### Finding 10
- **Section:** `cli.rs` lines 49-139 (parse_args)
- **Severity:** MINOR
- **Round:** R3
- **Finding:** No `--help` or `-h` flag is implemented. Running `ninja-exec` with no arguments silently starts the agent in interactive mode. Running `ninja-exec foo` (unknown subcommand) also silently starts the agent. There is no usage message, no subcommand listing, and no error for unrecognized commands. An operator who mistypes a subcommand inadvertently starts the agent.
- **Recommendation:** (a) Add `--help` / `-h` flag that prints usage with all subcommands and flags. (b) Unknown subcommands should print "Unknown command '{}'. Run 'ninja-exec --help' for usage." and exit with code 1 instead of falling through to `Run`. (c) Running with no arguments should print a brief usage summary rather than starting the agent — or at minimum, print a message indicating the agent is starting.
- **Impact:** Typos silently launch the signing agent. Operators cannot discover available commands without reading source code.

### Finding 11
- **Section:** `server.rs` lines 615-618 (CORS policy), `main.rs` lines 191-193 (token stdout)
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The CORS wildcard (`allow_origin(Any)`) means any browser-based UI could interact with the signing agent without restriction. From a UX perspective, this creates an invisible trust boundary — the operator has no visibility into which origins are making signing requests, no UI to manage allowed origins, and no notification when a request arrives from an unfamiliar origin. The confirm token printed to stdout (C7) compounds this by making the confirmation bypass mechanism visible in logs. These UX concerns are downstream of open CRITICALs C3 and C7.
- **Recommendation:** When C3 is resolved, add a configurable origin allowlist with a management command (`ninja-exec origins add/remove/list`). When C7 is resolved, provide a dedicated `ninja-exec token show` command with appropriate access controls.
- **Impact:** DEFERRED — blocked by C3, C7.

### Finding 12
- **Section:** `audit.rs` lines 38-51, `server.rs` passim
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** From a UX perspective, the silent audit failure (C4) means the operator has no visibility into whether their signing actions are being recorded. There is no audit health indicator in the status endpoint, no warning when audit writes fail, and no `ninja-exec audit tail` command to verify the audit trail is active. The absence of Rep C addresses in audit entries (C6) means audit records cannot be correlated with operator identity in multi-node environments. These are downstream of C4 and C6.
- **Recommendation:** When C4/C6 are resolved: (a) Add `audit_healthy: bool` to the `/status` response. (b) Add `ninja-exec audit tail [N]` command to show recent audit entries. (c) Include the operator's Rep C address in status output for identity verification.
- **Impact:** DEFERRED — blocked by C4, C6.

### Finding 13
- **Section:** `signing_engine.rs`, `server.rs` handle_sign
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The signing flow provides no operator-facing context about what is being signed. The `/sign` endpoint accepts `payload_b64` and `context` but the confirmation queue (displayed via `/confirm/pending`) shows only the context string and a hash of the payload. The operator approving a signing request cannot see a human-readable description of what they are approving. Without Rep C binding (C1/C2), the operator also cannot verify which identity is performing the signature. These concerns are downstream of C1 and C2.
- **Recommendation:** When C1/C2 are resolved: (a) Include a human-readable summary in the confirmation prompt (e.g., "Sign exec command 'Get-Service' for node [Rep C address]?"). (b) Display the first N characters of the decoded payload (if text) or the file name (if file path) alongside the hash.
- **Impact:** DEFERRED — blocked by C1, C2.

### Finding 14
- **Section:** `plenum-app.toml`, entire product
- **Severity:** MINOR
- **Round:** R3
- **Finding:** No update mechanism is specified. There is no `[update]` section in `plenum-app.toml`, no update check command, no version migration messaging, no rollback communication, and no changelog reference. The `upgrade_code` is a placeholder (covered by R1 I6). An operator running an outdated version has no way to discover that updates are available, no way to update in place, and no information about what changed.
- **Recommendation:** (a) Add an `[update]` section to `plenum-app.toml` specifying update check URL, update channel, and notification mechanism. (b) Add `ninja-exec update check` command. (c) On startup, optionally check for updates and display a non-blocking notification. (d) Document the update procedure (download new binary, stop agent, replace, restart).
- **Impact:** Operators run outdated versions with known vulnerabilities because there is no update notification or procedure.

### Finding 15
- **Section:** `plenum-app.toml` [app_type], `main.rs` (Run command)
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** The tray agent UI surface is declared (`kind = "tray_agent"`, `tray_icon`, `tray_tooltip`) but no tray UI implementation exists in the source code. The `configure_command` is empty. The tray icon exists only as a manifest declaration — there is no menu, no status display, no lock/unlock action, no "show fingerprint" option, and no quit action accessible from the tray. The confirmation queue endpoints (`/confirm/pending`, `/confirm/decide`) exist but require an external UI (the tray agent or YODA dashboard) that is not provided. An operator in interactive mode who needs to approve a signing request has no visible mechanism to do so.
- **Recommendation:** (a) Document that the tray UI is provided by the PlenumNET Launcher (Task #53) or specify the expected external UI. (b) Add a CLI fallback for confirmation: `ninja-exec confirm list` and `ninja-exec confirm approve/reject <id>`. (c) If no external UI is available, interactive mode signing requests will timeout after 60 seconds with no operator notification — add a stderr notification when a confirmation request arrives.
- **Impact:** Interactive mode is effectively unusable because confirmation requests cannot be approved without an external tool that is not shipped with NinjaExec.

### Finding 16
- **Section:** `main.rs` lines 320-368 (SignFile command)
- **Severity:** MINOR
- **Round:** R3
- **Finding:** The `sign` CLI command provides no feedback during signing. The operator runs `ninja-exec sign <file>`, enters their passphrase, and receives a base64 signature on stdout with no context. There is no confirmation of what file was signed, no display of the file size or hash, no indication of the algorithm used, and no suggestion of how to verify the signature.
- **Recommendation:** (a) Print to stderr: "Signing [filename] ([size] bytes) with TL-DSA-87..." before the signature. (b) After the signature, print to stderr: "Verify with: ninja-exec verify [filename] [signature]". (c) Consider outputting a structured signature envelope (JSON) with `--json` flag that includes the filename, hash, algorithm, and signature.
- **Impact:** Operators cannot confirm what was signed or how to verify it without reading documentation.

### Finding 17
- **Section:** `config.rs` lines 46-56, `main.rs`
- **Severity:** IMPORTANT
- **Round:** R3
- **Finding:** Configuration errors are completely silent. `NinjaExecConfig::load()` silently falls back to defaults if the config file is malformed. An operator who carefully configures security-critical settings (rate limits, confirmation rules) and makes a JSON syntax error will get default settings with zero indication. There is no `ninja-exec config validate` command, no `ninja-exec config show` command, and no startup message indicating which config was loaded.
- **Recommendation:** (a) On malformed JSON, print a clear error to stderr and exit: "Failed to parse ninja-exec.json: [error]. Fix the configuration or delete it to use defaults." (b) Add `ninja-exec config show` to display the active configuration. (c) On startup, print to stderr: "Config loaded from [path]" or "Using default configuration (no ninja-exec.json found)."
- **Impact:** Security-critical misconfiguration is silently ignored, giving operators false confidence in their security posture.

### Finding 18
- **Section:** `main.rs`, `cli.rs`, entire product
- **Severity:** MINOR
- **Round:** R3
- **Finding:** Keyboard accessibility is limited to standard terminal input. This is acceptable for a CLI tool. However, the confirmation queue endpoints (`/confirm/pending`, `/confirm/decide`) are API-only with no keyboard-navigable UI. If a future tray UI is implemented, keyboard navigation (Tab order, Enter activation, Escape dismissal) is not specified. Screen reader compatibility is not addressed for any surface — status information is conveyed only through JSON fields with no ARIA-equivalent labeling for future UI consumers.
- **Recommendation:** (a) For the CLI: current keyboard interaction is acceptable. (b) For future tray UI: specify Tab order, Enter activation for approve/reject, Escape to dismiss. (c) Ensure the `/status` JSON response includes human-readable `status_text` field for screen reader consumption.
- **Impact:** Future UI implementations will lack accessibility specifications, leading to inconsistent keyboard navigation.

### Finding 19
- **Section:** `server.rs` handle_sign (lines 147-323), `main.rs` handle_sign file (lines 320-368)
- **Severity:** DEFERRED
- **Round:** R3
- **Finding:** The HTTP `/sign` endpoint and CLI `sign` command provide no signing progress feedback. For the HTTP API, the response is synchronous — the caller blocks until the confirmation is resolved (up to 60 seconds) or the signature is computed. No intermediate status (e.g., "awaiting confirmation", "signing in progress") is communicated. The confirmation polling loop in `server.rs` lines 217-224 sleeps in 250ms intervals with no client notification. This is downstream of C5 (no integration tests to validate the UX of the confirmation flow).
- **Recommendation:** When C5 is resolved: (a) Consider WebSocket or SSE for real-time confirmation status updates. (b) For HTTP, return 202 Accepted with a polling URL for long-running confirmation flows. (c) Add a `status` field to the `/confirm/pending` response showing the confirmation state.
- **Impact:** DEFERRED — blocked by C5.

---

## Summary Verdict

**PASS WITH CONDITIONS**

NinjaExec has a sound architectural foundation — the CLI command structure is logical, the HTTP API is well-designed, the keystore format is thoughtful, and the confirmation queue pattern is appropriate. However, the operator experience has significant gaps that must be resolved before release:

- The passphrase prompt provides no strength feedback against the 72-bit entropy floor (Finding 1)
- Interactive mode confirmation is unusable without an external UI (Finding 15)
- Silent config parsing failures create false security confidence (Finding 17)
- The uninstall flow has no clean-removal path and shows raw environment variables (Finding 7)
- No help command, no update mechanism, no progress indicators

The 4 DEFERRED findings (11, 12, 13, 19) are blocked by open CRITICALs and do not affect this verdict.

**Conditions for PASS:**
1. Resolve Findings 1, 2, 4, 7, 15, 17 (all IMPORTANT)
2. Resolve all 7 open R1/R2 CRITICALs (C1-C7) to un-defer Findings 11-13, 19

---

## Brand Score: 4/10

NinjaExec gets the fundamentals right — localhost-only binding, TL-DSA-87, encrypted keystore, audit logging — but the operator-facing surface feels like an engineering prototype, not a shipping product. The startup banner with box-drawing characters is a nice touch, but garbled terminals, missing help text, silent config failures, raw JSON status output, and an unusable interactive confirmation mode undermine the "professional security tool" brand. The product name and tagline ("the ssh-agent of PlenumNET") are strong. The 50th-use experience — the 3am troubleshooting session — would be frustrating: no `--help`, cryptic error messages, no diagnostic commands, no update path.

---

## Top 3 Quick Wins

1. **Add `--help` / `-h` with usage summary** (Finding 10) — Highest impact for ~30 lines of code. Operators can discover commands without reading source. Unknown subcommands stop silently launching the agent.

2. **Human-readable `ninja-exec status` output** (Finding 8) — Format status as a one-line summary with actionable guidance when locked. Keep `--json` for machine consumption. ~20 lines of code, dramatically improves troubleshooting.

3. **Passphrase mismatch re-prompt instead of exit** (Finding 4) — Instead of `exit(1)` on mismatch, loop back to re-prompt. Prevents operators from restarting the entire init process for a typo. ~10 lines of code.

---

## Friction Map

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

**action_count:** 30

---

## Review Complete

**Summary Verdict:** PASS WITH CONDITIONS
**Brand Score:** 4/10
**Finding Count:** 19 total (0 CRITICAL, 6 IMPORTANT, 9 MINOR, 4 DEFERRED)
