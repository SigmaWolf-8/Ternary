# QC-R1 Agent 2: DevOps Automator Review

**Document Under Review:** `services/inter-cube/deploy-yoda.ps1` (v0.5.0, 1652 lines)
**Review Protocol Version:** 1.1.2
**Date:** 2026-03-30

---

### Finding 1
- **Section:** Lines 68-69, Version and release tag constants
- **Severity:** IMPORTANT
- **Finding:** The deployer version (`$DEPLOYER_VERSION = "v0.5.0"`) and the release tag (`$RELEASE_TAG = "v0.5.0"`) are hardcoded string literals in the script. Neither Rust/cargo nor LLVM tool versions are pinned to specific patch-level versions. The Rust toolchain installed via `rustup-init.exe -y` (line 306) uses the latest stable channel, and the LLVM installer fetched from GitHub (line 384) uses the latest release tag. This means two deployments run a week apart could build with different compiler versions, producing different binaries from the same source. Build reproducibility is not achievable.
- **Recommendation:** (1) Pin the Rust toolchain version explicitly: `rustup-init.exe -y --default-toolchain 1.XX.Y`. (2) Pin the LLVM version to a specific release tag rather than using the GitHub "latest" API. (3) Document the expected toolchain versions in the deployer's prerequisites. (4) Consider recording the actual toolchain versions used in the deployment payload for audit trail.
- **Verification:** Run the deployer twice on fresh machines at different dates and compare `rustc --version` and `clang --version` output. Both should be identical.
- **Finding ID:** R1-A2-1

### Finding 2
- **Section:** Lines 416-440, STEP 3 — Git clone and checkout
- **Severity:** IMPORTANT
- **Finding:** The git clone uses `--depth 1` (line 418) which is good for bandwidth, but the checkout in the upgrade path (lines 437-438) does `git fetch origin tag $RELEASE_TAG --force` followed by `git checkout $RELEASE_TAG`. There is no verification that the fetched tag is a signed git tag. An attacker who compromises the GitHub repository or intercepts the HTTPS connection could serve a modified tag pointing to malicious source code. The `--force` flag on fetch suppresses warnings about tag updates. Additionally, there is no submodule initialization step — if the project uses submodules, they would be missing.
- **Recommendation:** (1) Verify the git tag signature after fetch using `git tag -v $RELEASE_TAG` with a pre-distributed GPG public key. (2) Remove the `--force` flag from `git fetch` and handle tag conflicts explicitly. (3) After checkout, verify the source tree hash matches an expected value. (4) Add `git submodule update --init --recursive` if submodules are used.
- **Verification:** Modify the remote tag to point to a different commit and confirm the deployer detects the discrepancy and aborts.

### Finding 3
- **Section:** Lines 466-493, STEP 4 — Cargo build
- **Severity:** IMPORTANT
- **Finding:** The build step uses `CARGO_BUILD_JOBS=1` (line 462) to serialize compilation, but there is no `Cargo.lock` pinning verification. The build depends on `cargo build --release -p inter-cube` which resolves dependencies according to `Cargo.lock` if present, but the script does not verify `Cargo.lock` exists or that it hasn't been modified. The build error detection (line 470) uses a regex match on the word "error" in build output, which could produce false positives (e.g., a crate named "error-chain" being compiled). The actual build success/failure is determined by `$LASTEXITCODE` (line 484), which is correct, but the false-positive error highlighting could confuse operators.
- **Recommendation:** (1) Verify `Cargo.lock` exists before building and warn if it doesn't. (2) Change the error line detection to match Cargo's actual error format: `"^error\[E\d+\]"` or `"^error:"`. (3) Record the Cargo.lock hash in the deployment payload for reproducibility auditing. (4) Consider using `cargo build --release --locked` to fail if `Cargo.lock` is out of sync.
- **Verification:** Build with and without `Cargo.lock` present and confirm the deployer produces appropriate warnings. Verify false-positive error highlighting does not occur for crate names containing "error".

### Finding 4
- **Section:** Lines 503-526, Binary integrity — SHA-256 fallback
- **Severity:** IMPORTANT
- **Finding:** The binary integrity check uses SHA-256 as the primary hash (line 504) and attempts TIS-27 as a secondary check. If TIS-27 fails, the script silently falls back to SHA-256 (line 524) with a warning but continues deployment. From a DevOps perspective, this means the integrity verification mechanism is non-deterministic — some deployments use TIS-27 and some use SHA-256, making it impossible to establish a consistent integrity baseline across deployments. The pre-start re-verification (lines 528-537) also uses SHA-256. Cross-reference: Security Engineer (Agent 1) should assess the cryptographic implications.
- **Recommendation:** (1) Make TIS-27 the sole integrity mechanism. If TIS-27 hash fails, block deployment. (2) Remove SHA-256 from the integrity pipeline entirely. (3) Standardize the hash format in the deployment payload so downstream systems know which hash algorithm was used.
- **Verification:** Disable the daemon's hash mode and confirm the deployer fails with a clear error rather than falling back to SHA-256.

### Finding 5
- **Section:** Lines 872-898, Service registration error handling
- **Severity:** IMPORTANT
- **Finding:** If service registration fails (the `catch` block at line 894), the deployer prints a warning but continues to the next step. The failed service name is NOT added to `$partialServices` (line 892 is inside `try`, before the `catch`), so the cleanup handler (lines 1637-1646) will not remove the partially-created service. More critically, there is no rollback mechanism if the deployment fails partway through Step 8: some services may be registered and started while others failed, leaving the cluster in an inconsistent state. The deployer should either succeed atomically or roll back all changes.
- **Recommendation:** (1) Add the service name to `$partialServices` before the `New-Service` call, and remove it from the list on success, so the cleanup handler can clean up on failure. (2) Implement a transactional deployment pattern: register all services first (without starting), verify all registrations succeeded, then start them in order. (3) If any registration fails, clean up all registered services and exit.
- **Verification:** Simulate a service registration failure for Node #2 and confirm all services (including successfully registered ones) are cleaned up.

### Finding 6
- **Section:** Lines 446-457, Running daemon process detection and kill
- **Severity:** IMPORTANT
- **Finding:** Before building, the script detects running daemon processes and kills them with `Stop-Process -Force` (line 455). This is a hard kill (SIGKILL equivalent) with no graceful shutdown attempt. There is a 3-second warning delay (line 453) but no actual graceful shutdown signal. Connected relay clients are warned in text but the daemons are not given an opportunity to close connections, flush logs, or complete in-flight operations. The same hard-kill pattern appears in the watchdog (orphan killing). For an upgrade scenario, this can cause data loss or corrupted state.
- **Recommendation:** (1) Send a graceful shutdown signal first (e.g., `Stop-Service` with a timeout) before falling back to `Stop-Process -Force`. (2) Wait for the daemon to exit gracefully with a configurable timeout (e.g., 30 seconds). (3) Only use `Stop-Process -Force` if graceful shutdown times out. (4) Log whether the shutdown was graceful or forced.
- **Verification:** Start a daemon, initiate an upgrade, and confirm the daemon receives a graceful shutdown signal and has time to close connections before being killed.

### Finding 7
- **Section:** Lines 1324-1366, Watchdog scheduled task registration
- **Severity:** MINOR
- **Finding:** The watchdog scheduled task is registered with two triggers: at startup and every 2 minutes. The fallback path (lines 1353-1365) uses `schtasks.exe` which does not support the same level of configuration as the PowerShell `Register-ScheduledTask` cmdlet. The fallback does not configure `RestartCount`, `RestartInterval`, or battery settings. This means deployments that hit the fallback path have a degraded watchdog configuration compared to the primary path. The two paths produce functionally different watchdog behaviors without warning the operator.
- **Recommendation:** (1) Log which registration path was used (primary or fallback) in the deployment summary. (2) Attempt to configure equivalent settings via `schtasks.exe` XML import if the PowerShell path fails. (3) Document the behavioral differences between the two paths.
- **Verification:** Force the primary registration to fail and verify the fallback path logs a warning about reduced functionality.

### Finding 8
- **Section:** Lines 546-558, Version probe — temp directory for keygen
- **Severity:** MINOR
- **Finding:** The version probe creates a temporary directory (`plenumnet-version-probe-*`) and runs the daemon in keygen mode just to extract version information (lines 546-553). This generates throwaway key material in a temp directory, which is deleted afterward. However, if the cleanup fails (the `Remove-Item` is inside `try` and could be skipped if the daemon crashes or the path has a lock), orphaned key material is left in `$env:TEMP`. The temp directory is not ACL-restricted before keygen runs, so the generated keys are readable by any process running as the same user during the brief window.
- **Recommendation:** (1) Use a dedicated `--version` or `CUBE_MODE=version` flag that returns version info without generating keys. (2) If keygen must be used, ACL-restrict the temp directory before running keygen. (3) Add the cleanup to a `finally` block to ensure it runs even on exception.
- **Verification:** Run the version probe, kill the daemon mid-execution, and confirm the temp directory is cleaned up.

### Finding 9
- **Section:** Lines 560-576, Remote version check
- **Severity:** MINOR
- **Finding:** The remote version check (line 563) calls `$REMOTE_CRS/health/crs` with a 10-second timeout and silently swallows failures (empty `catch {}` block at line 565). If the remote CRS is unreachable, `$remoteVersion` remains "unknown" and the deployer proceeds without version alignment verification. This is acceptable for offline deployments, but the empty catch block means network errors, TLS failures, and HTTP errors are all silently ignored with no diagnostic output.
- **Recommendation:** (1) Log the specific error from the remote health check (even if only at a verbose/debug level). (2) Distinguish between "unreachable" (network error) and "unhealthy" (HTTP error) in the output. (3) If this is a known offline-capable path, document it explicitly.
- **Verification:** Deploy with the remote CRS unreachable and confirm the deployer produces a useful diagnostic message rather than silently skipping version verification.

### Finding 10
- **Section:** Lines 236-241, Existing deployment detection
- **Severity:** MINOR
- **Finding:** The upgrade detection (line 238) checks for existing services via `Get-Service PlenumNET-Array3-*` OR the existence of `$RepoDir` (`C:\PlenumNET`). However, the directory could exist from a failed previous deployment without services being registered, or services could exist without the directory (e.g., after manual cleanup). The binary `$isUpgrade` flag conflates these two different states, which could lead to incorrect upgrade messaging or behavior (e.g., "Preserve existing node identities" when there are no identities to preserve).
- **Recommendation:** (1) Distinguish between "directory exists with identities", "directory exists without identities", "services exist", and "clean install". (2) Report the detected state explicitly to the operator. (3) Check for the presence of `master.key` files specifically when determining whether identities exist to preserve.
- **Verification:** Create `C:\PlenumNET` as an empty directory (no identities, no services) and confirm the deployer correctly identifies this as a partial/failed previous install rather than an upgrade.

### Finding 11
- **Section:** Lines 648-662, secedit security policy export to TEMP
- **Severity:** MINOR
- **Finding:** The ops sandbox hardening section exports the local security policy to `$env:TEMP\plenumnet-secpol.cfg` (line 649). While the file is deleted after use (line 658), the `$env:TEMP` directory is user-writable and the exported policy file is not ACL-restricted. The security policy contains privilege assignments for all local accounts and could be useful for reconnaissance. Cross-reference: Security Engineer (Agent 1) should assess the exposure window.
- **Recommendation:** (1) Use a randomized temp directory (as done in `Grant-LogonAsService`) rather than `$env:TEMP` directly. (2) ACL-restrict the temp file before writing. (3) Use a `finally` block to ensure cleanup.
- **Verification:** Monitor the temp directory during deployment and confirm the security policy export file is not readable by non-admin users and is cleaned up promptly.

---

## Summary Verdict

**Verdict: PASS WITH CONDITIONS**

The deployer script demonstrates solid operational design: 10-step structured deployment, upgrade detection, graceful fallbacks, watchdog configuration, and service recovery. However, six IMPORTANT findings must be resolved before first release:

1. **R1-A2-1**: Build tool versions (Rust, LLVM) are not pinned, breaking reproducibility.
2. **R1-A2-2**: Git tag authenticity is not verified — source integrity relies on HTTPS alone.
3. **R1-A2-3**: Build process lacks `Cargo.lock` verification and has false-positive error detection.
4. **R1-A2-4**: Binary integrity uses SHA-256 fallback (cross-ref: Agent 1 for crypto implications).
5. **R1-A2-5**: Service registration has no atomic rollback — partial failures leave inconsistent state.
6. **R1-A2-6**: Running daemons are hard-killed without graceful shutdown opportunity.

The CONDITIONS for release are: pin all tool versions, verify source tag signatures, make TIS-27 the sole integrity mechanism, implement atomic service registration, and add graceful shutdown before forced kills. The five MINOR findings (watchdog fallback parity, temp directory key material, silent remote errors, upgrade state detection, and secedit temp file exposure) should be addressed iteratively.
