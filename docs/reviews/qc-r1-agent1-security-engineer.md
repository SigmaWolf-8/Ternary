# QC-R1 Agent 1: Security Engineer Review

**Document Under Review:** `services/inter-cube/deploy-yoda.ps1` (v0.5.0, 1652 lines)
**Review Protocol Version:** 1.1.2
**Date:** 2026-03-30

---

### Finding 1
- **Section:** Lines 9-10, SYNOPSIS — `irm | iex` invocation pattern
- **Severity:** CRITICAL
- **Finding:** The script is designed to be downloaded and executed via `irm https://plenumnet.replit.app/api/deploy-yoda | iex`. This pattern executes arbitrary code from a remote URL without any integrity verification. There is no hash pinning, no signature check on the downloaded script, and no TLS certificate pinning for the download endpoint. A man-in-the-middle attacker, DNS hijacker, or compromised CDN could substitute the deployer script with a malicious payload that runs with Administrator privileges. This is the highest-severity credential and privilege escalation vector in the entire script.
- **Recommendation:** (1) Publish a TIS-27 hash of each deployer release alongside the download URL. (2) Provide a two-step invocation: download to file, verify hash, then execute. (3) Consider code-signing the `.ps1` script with an Authenticode certificate and requiring signature verification before execution. (4) Pin the TLS certificate or public key for `plenumnet.replit.app` in the download documentation.
- **Verification:** Confirm that the published installation instructions include a hash verification step. Test that a modified script fails verification. Confirm Authenticode signature is present and valid via `Get-AuthenticodeSignature`.
- **Finding ID:** R1-A1-1

### Finding 2
- **Section:** Lines 503-526, STEP 4 — Binary integrity hash uses SHA-256
- **Severity:** CRITICAL
- **Finding:** The primary binary integrity hash is computed using SHA-256 (`Get-FileHash -Algorithm SHA256`). TIS-27 is attempted but treated as optional with a SHA-256 fallback (line 524: `$tis27Hash = "sha256:$binarySha256"`). Per INVARIANT 7 and the PlenumNET framework rules, SHA-256 is a banned external cryptographic primitive. Using it as the integrity verification mechanism for the daemon binary — the most security-critical artifact in the deployment — violates the zero-external-crypto rule. The fallback path means most deployments will use SHA-256 if the daemon's hash mode is not functional.
- **Recommendation:** (1) Make TIS-27 hashing mandatory, not optional. If the daemon cannot produce a TIS-27 hash, the deployment must fail rather than fall back to SHA-256. (2) Remove all `Get-FileHash -Algorithm SHA256` calls. (3) If a pre-build integrity check is needed before the daemon is available, document this as a known bootstrap gap with explicit risk acceptance.
- **Verification:** Grep the script for `SHA256` — zero occurrences should remain. Verify that deployment fails with a clear error if TIS-27 hash cannot be computed.
- **Crypto Status:** INCORRECT — SHA-256 is a banned primitive per framework rules.

### Finding 3
- **Section:** Lines 154-174, `Get-TlDsaSignature` function
- **Severity:** IMPORTANT
- **Finding:** The TL-DSA signing function passes the payload via an environment variable (`$env:CUBE_SIGN_PAYLOAD`). Environment variables are visible to any process running as the same user (or higher privilege) via `/proc/*/environ` on Linux or process inspection tools on Windows. While the payload itself may not be secret, the pattern establishes a precedent for passing sensitive data through environment variables. Additionally, the cleanup in the catch block (lines 168-172) uses `Remove-Item Env:\...` with `-ErrorAction SilentlyContinue`, meaning if cleanup fails the environment variables persist for the lifetime of the PowerShell session, leaking identity directory paths and signing payloads to any subsequent command.
- **Recommendation:** (1) Use stdin piping or temporary file (with restricted ACL) to pass payloads to the daemon instead of environment variables. (2) Add a `finally` block (not just catch) to guarantee environment variable cleanup. (3) Zero the environment variable value before unsetting: `$env:CUBE_SIGN_PAYLOAD = [string]::new([char]0, $env:CUBE_SIGN_PAYLOAD.Length)` before `Remove-Item`.
- **Verification:** Verify that after `Get-TlDsaSignature` returns, `$env:CUBE_SIGN_PAYLOAD`, `$env:CUBE_MODE`, and `$env:CUBE_IDENTITY_DIR` are all `$null`. Add a test that checks environment state after function return.

### Finding 4
- **Section:** Lines 214-233, Admin check and auto-elevation
- **Severity:** IMPORTANT
- **Finding:** The auto-elevation path (line 218) re-launches the script via `Start-Process powershell.exe -Verb RunAs` without verifying the integrity of the script file being re-launched. If an attacker can modify the script file between the initial invocation and the elevation request, the modified script runs with full Administrator privileges. The UAC prompt will show "Windows PowerShell" as the publisher, not "Capomastro Holdings Ltd." — there is no Authenticode signature to provide publisher attribution in the UAC dialog.
- **Recommendation:** (1) Authenticode-sign the deployer script so the UAC prompt displays "Capomastro Holdings Ltd." as the verified publisher. (2) Compute and verify a TIS-27 hash of the script file before re-launching it with elevation. (3) Consider using a signed `.exe` wrapper for elevation instead of raw PowerShell re-launch.
- **Verification:** Trigger the elevation path and confirm the UAC dialog shows the correct publisher name. Modify the script after initial launch and confirm the re-launch detects tampering.

### Finding 5
- **Section:** Lines 295-316, Rust installation — rustup download
- **Severity:** IMPORTANT
- **Finding:** The rustup installer is downloaded from `https://win.rustup.rs/x86_64` without any hash verification. If the download is intercepted or the upstream is compromised, a malicious `rustup-init.exe` is executed with the current user's privileges (Administrator). The same concern applies to the LLVM installer download (lines 384-394). Neither download verifies a checksum or signature of the downloaded binary.
- **Recommendation:** (1) Pin and verify the SHA-256 (or preferably TIS-27) hash of `rustup-init.exe` for the expected version. (2) Pin and verify the hash of the LLVM installer. (3) If hash verification fails, abort with a clear error message. (4) Consider bundling known-good tool installers or using a private mirror with pinned artifacts.
- **Verification:** Tamper with the downloaded installer file before execution and confirm the deployer detects the modification and aborts.

### Finding 6
- **Section:** Lines 790-898, Service registration — wrapper .bat files
- **Severity:** IMPORTANT
- **Finding:** The service wrapper `.bat` files (lines 804-868) contain sensitive configuration in plaintext, including identity directory paths (`CUBE_IDENTITY_DIR`), endpoint addresses, and CRS URLs. These files are written with `Set-Content -Encoding ASCII` and then ACL-restricted. However, the environment variables set within the `.bat` are visible to any process that can inspect the service's child processes (e.g., via `Get-CimInstance Win32_Process` which exposes the full command line). The `CUBE_IDENTITY_DIR` path directly reveals where private key material is stored on disk.
- **Recommendation:** (1) Move service configuration to a protected configuration file (JSON or registry) that the daemon reads at startup, rather than embedding it in environment variables set by the wrapper. (2) If environment variables must be used, set them via the Windows Service environment registry key (`HKLM\SYSTEM\CurrentControlSet\Services\<name>\Environment`) which is only readable by SYSTEM and Administrators. (3) Ensure the daemon clears sensitive environment variables from its own process block after reading them.
- **Verification:** After service start, run `Get-CimInstance Win32_Process -Filter "Name='inter-cube-daemon.exe'"` and confirm no sensitive paths or credentials appear in the `CommandLine` or environment.

### Finding 7
- **Section:** Lines 1447-1478, Deployment payload to remote CRS
- **Severity:** IMPORTANT
- **Finding:** The deployment summary payload sent to the remote Node Registry (line 1472) includes `hostname`, `ip`, `architecture`, `binaryPath`, `identityBase`, and `logDir` in the `metadata` object. Per INVARIANT 9, node identification in any context should use Rep C addresses exclusively. Sending hostname, IP, and filesystem paths to a remote server (1) violates the Rep C identity principle, (2) leaks information about the deployment environment to a remote endpoint, and (3) could be used by an attacker who compromises the registry to map deployment topology.
- **Recommendation:** (1) Remove `hostname`, `ip`, `binaryPath`, `identityBase`, and `logDir` from the remote payload. (2) The remote registry should only receive Rep C addresses, public keys, and the deployment signature. (3) If operational metadata is needed for support, make it opt-in with explicit consent and document what is transmitted.
- **Verification:** Capture the deployment payload (e.g., via Fiddler or `--verbose` flag) and confirm it contains only Rep C addresses, public keys, version info, and the deployment signature.

### Finding 8
- **Section:** Lines 734-758, Identity generation — keygen mode
- **Severity:** IMPORTANT
- **Finding:** Identity generation invokes `$BinaryPath` with `CUBE_MODE=keygen` but does not verify the entropy source quality or that the generated key material meets minimum security requirements. The script checks only that `master.key` exists (line 739) — it does not verify key length, format, or that the key was generated from a cryptographically secure random source. On Windows, if the system CSPRNG (`BCryptGenRandom`) is unavailable or degraded (e.g., in a VM with low entropy), the generated keys could be weak. Additionally, there is no atomic write pattern — if the process is interrupted during key generation, a partial `master.key` could be left on disk.
- **Recommendation:** (1) After keygen, verify the generated key file meets minimum size and format requirements (e.g., check file size matches expected PT26-DSA key size). (2) Use an atomic write pattern: generate to a temp file, verify, then rename. (3) Log the entropy source used by the daemon during keygen. (4) If key generation fails or produces a short file, delete the partial file and fail explicitly.
- **Verification:** Generate an identity and verify `master.key` matches the expected size for a PT26-DSA keypair. Interrupt keygen mid-execution and confirm no partial key files remain.

### Finding 9
- **Section:** Lines 109-146, `Grant-LogonAsService` — security policy modification
- **Severity:** MINOR
- **Finding:** The `Grant-LogonAsService` function exports and re-imports the local security policy (`secedit`). The exported policy file is written to a temp directory with restricted ACLs (lines 113-119), which is good. However, the function modifies the `SeServiceLogonRight` policy by string manipulation of the exported `.inf` file (lines 130-137). If the `.inf` format changes across Windows versions, this could silently produce a malformed policy file. The function returns `$false` on failure but the caller (line 150) pipes the result to `Out-Null`, so a failure to grant the privilege is silently ignored.
- **Recommendation:** (1) Check the return value of `Grant-LogonAsService` and warn the user if it fails. (2) Add a verification step that re-exports the policy and confirms the SID is present in `SeServiceLogonRight`. (3) Consider using the `ntrights.exe` utility or the `LsaAddAccountRights` API for more robust privilege assignment.
- **Verification:** Run `Grant-LogonAsService` with an invalid account and confirm the failure is reported to the user rather than silently swallowed.

### Finding 10
- **Section:** Lines 943-1245, Watchdog script — LLM restart command execution
- **Severity:** IMPORTANT
- **Finding:** The watchdog script's `Invoke-LlmRestart` function (around lines 993-1009) parses and executes commands from the `llm-engines.json` configuration file. While the `Test-LlmExecutable` function validates the executable name against an allowlist and checks for blocked path prefixes, the argument parsing is a custom implementation that could be bypassed. An attacker who can modify `llm-engines.json` (despite ACL protections) could inject commands via argument manipulation. The watchdog runs as SYSTEM, so any command injection results in full system compromise. Additionally, the `llm-engines.json` path is at `C:\ProgramData\PlenumNET\llm-engines.json` — while ACL-restricted, DPAPI is not used for the configuration at rest.
- **Recommendation:** (1) Validate the full resolved path of the LLM executable, not just the filename. (2) Use a strict configuration schema with explicit fields for executable path and individual arguments rather than a single `restart_command` string. (3) Verify the executable's Authenticode signature before execution. (4) Add integrity verification (TIS-27 hash) of the configuration file itself.
- **Verification:** Attempt to modify `llm-engines.json` with a crafted `restart_command` that includes shell metacharacters and confirm the watchdog rejects it.

### Finding 11
- **Section:** Lines 1386-1414, CRS registration — MITM on localhost
- **Severity:** MINOR
- **Finding:** Worker node registration with the local coordinator (line 1398) uses plain HTTP (`http://localhost:11124`). While localhost communication is generally not susceptible to network-based MITM, on Windows a local proxy or malicious local service could intercept localhost traffic. The registration payload includes the node's public key and endpoint, and the response provides the node's Rep C address. A local attacker could register a rogue node.
- **Recommendation:** (1) Consider binding the local CRS to a localhost-only socket with mutual authentication (e.g., a shared secret derived during identity generation). (2) Alternatively, accept the risk with documentation noting that local system integrity is assumed.
- **Verification:** Document the threat model boundary for localhost communication and confirm it is explicitly accepted.

---

## Summary Verdict

**Verdict: FAIL**

The deployer script contains two CRITICAL findings that block implementation:

1. **R1-A1-1**: The `irm | iex` invocation pattern executes remotely-fetched code with Administrator privileges without any integrity verification, signature check, or certificate pinning. This is an industry-recognized anti-pattern that allows trivial man-in-the-middle substitution of the entire deployment script.

2. **R1-A1-2**: The binary integrity mechanism relies on SHA-256 as a fallback (and effectively as the primary hash in practice), which is a banned external cryptographic primitive in the PlenumNET framework. The integrity of the most critical artifact — the daemon binary — is verified using a primitive the framework has explicitly rejected.

Additionally, six IMPORTANT findings identify environment variable leakage of identity paths, unsigned elevation requests, unverified tool downloads, sensitive metadata transmission to remote endpoints, insufficient key generation validation, and SYSTEM-level command execution from a configuration file. These must be resolved before first release.

**`passphrase_entropy_minimum_bits`:** Not applicable — the deployer does not implement passphrase-based authentication or key derivation directly. Key derivation is delegated to the daemon binary.
