# QC-R1 Review — Agent 2: DevOps Automator

**Spec file:** `.local/tasks/continuous-attestation-service.md`
**Revision:** Initial (no version number specified in document)
**Reviewer:** Agent 2 — DevOps Automator (`engineering/devops-automator`)
**Review date:** 2026-07-13
**Skill version:** 1.1.2

---

### Finding R1-A2-1
- **Section:** Specification (§ "Specification", attestation interval paragraph)
- **Severity:** IMPORTANT
- **Finding:** The spec defines the attestation service runtime behavior (broadcast intervals, backoff, suspicion counters) but does not specify how the attestation service itself is built, packaged, or deployed. There is no CI/CD pipeline definition, no build command, no artifact output, no deployment target (Windows service, systemd unit, embedded in inter-cube daemon), and no single-command reproducible deployment path. The "Relevant files" section lists existing source files but does not define what new build artifacts this task produces or how they integrate into the existing build matrix.
- **Recommendation:** Add a "Build & Deployment" section specifying: (a) whether the attestation service is a new binary, a new module within the inter-cube daemon, or a new Windows service; (b) the build command and toolchain version requirements (Rust edition, MSRV); (c) how it integrates into the existing CI/CD pipeline (new workflow step, existing workflow extension); (d) the deployment mechanism (bundled in MSI installer, sidecar, config-driven activation).
- **Verification:** Confirm the spec contains a reproducible build command that produces a named artifact, and that artifact appears in the CI pipeline definition.

### Finding R1-A2-2
- **Section:** Tasks (§ "Tasks", items 1–6)
- **Severity:** IMPORTANT
- **Finding:** No task defines automated testing — neither unit tests, integration tests, nor CI validation steps. The spec describes complex behaviors (dynamic jitter, partition-aware suspicion counters, per-link backoff, schema versioning) but does not specify how these are verified in CI. There is no mention of expected CI duration, parallelism strategy, or maximum acceptable wall-clock time for a full test matrix run.
- **Recommendation:** Add a task (or subtasks under existing tasks) for: (a) unit tests for Merkle tree construction, suspicion counter logic, backoff calculation, and schema version registry; (b) integration tests simulating partition scenarios (heartbeat-reachable-but-attestation-absent vs. heartbeat-unreachable); (c) a mock mode for PUF-derived keys so tests can run without hardware; (d) expected CI duration and parallelism strategy for full-matrix retesting.
- **Verification:** Confirm the spec defines at least one automatable test per task item, each producing machine-verifiable exit codes (not human-readable output).

### Finding R1-A2-3
- **Section:** Specification (§ "Specification", bandwidth budget paragraph)
- **Severity:** IMPORTANT
- **Finding:** The spec defines failure handling for bandwidth overload (exponential backoff) but does not specify failure modes for other scenarios: What happens if TL-DSA signing fails (PUF unavailable, key corruption)? What happens if measured boot data is unavailable (firmware_sign.rs returns error)? What happens if the HPTP timestamp service is unreachable? For each, the spec must define whether the attestation broadcast is blocked, retried, or skipped with a warning. Silent failures in any of these paths could result in a node appearing healthy (no attestation failure reported) while actually being unable to attest.
- **Recommendation:** Add a "Failure Modes" section enumerating at least: (a) TL-DSA signing failure — block attestation, increment local error counter, log to security audit; (b) PUF self-test failure — attestation report includes failure indicator, node transitions to degraded state; (c) HPTP timestamp unavailable — retry with exponential backoff, block attestation after N retries; (d) Merkle tree construction failure (no heartbeat challenges received) — report with empty liveness proof, flag on dashboard.
- **Recommendation:** Cross-reference to Security Engineer (Agent 1) for severity assessment of the TL-DSA signing failure scenario, per Critical Rules.
- **Verification:** Confirm every component dependency (PUF, TL-DSA, HPTP, firmware_sign, self_test) has an explicit failure mode with a defined outcome (block/retry/degrade).

### Finding R1-A2-4
- **Section:** Specification (§ "Specification", report versioning paragraph)
- **Severity:** IMPORTANT
- **Finding:** The schema version registry is "maintained in PlenumConfig and updated as part of the firmware release process," but no CI gate enforces that a new attestation report schema version is registered before firmware containing it can be published. Without a pipeline check, a firmware release could ship with a new schema version that is unknown to all neighbors, causing universal "unparseable" alerts across the fleet — a silent deployment failure.
- **Recommendation:** Add a CI pipeline step that validates: (a) the attestation report schema version in the firmware build matches a version registered in PlenumConfig; (b) PlenumConfig schema version registry is updated atomically with the firmware release (same commit or same release artifact). This step must produce a non-zero exit code on mismatch.
- **Verification:** Confirm the CI pipeline definition includes a schema-version validation step with exit-code-based pass/fail.

### Finding R1-A2-5
- **Section:** Specification (§ "Specification", attestation report content)
- **Severity:** IMPORTANT
- **Finding:** The spec states attestation reports include "kernel integrity hash" and "FTS/GLB configuration fingerprint" but does not specify which hash primitive is used. Per the DevOps review scope (§5 Checksum and integrity), checksums must use TIS-27 (not SHA-256/BLAKE3). Additionally, the spec does not state whether the kernel integrity hash is computed at build time (deterministic, diffable) or at runtime (varies per boot). If computed at runtime, the "expected value" that neighbors compare against must be distributed — the spec does not define how expected values are provisioned.
- **Recommendation:** (a) Explicitly state that kernel integrity hash and config fingerprint use TIS-27 (INVARIANT 8 compliance — inputs must be trit-encoded before sponge absorb). (b) Define whether expected hash values are embedded in the firmware image (build-time) or distributed via PlenumConfig (runtime). (c) Specify how the expected-value distribution is kept in sync with firmware releases.
- **Verification:** Confirm the spec names TIS-27 as the hash primitive for all integrity fields and defines the expected-value provisioning mechanism.

### Finding R1-A2-6
- **Section:** Specification (§ "Specification", TL-DSA signing and HPTP timestamps)
- **Severity:** MINOR
- **Finding:** The spec correctly requires TL-DSA signing with PUF-derived root key and HPTP timestamps (consistent with INVARIANT 7 and INVARIANT 6). However, it does not explicitly state that the signer's Rep C address is bound into the TL-DSA signing context string (INVARIANT 9 requirement). While the spec references "PUF-derived root key," the signing context binding is a separate requirement from key selection.
- **Recommendation:** Add an explicit statement: "The TL-DSA signing context string for attestation reports MUST include the signer node's Rep C address (54-trit, binary-encoded) per INVARIANT 9."
- **Verification:** Grep the implementation for the signing context string and confirm it includes the Rep C address field.

### Finding R1-A2-7
- **Section:** Relevant files (§ "Relevant files")
- **Severity:** MINOR
- **Finding:** The spec does not list any CI/CD workflow files (`.github/workflows/`) in the relevant files section, despite the fact that implementing this service will require either modifying existing workflows or creating new ones. This increases the risk that CI integration is treated as an afterthought.
- **Recommendation:** Add the relevant `.github/workflows/` files to the "Relevant files" section, or explicitly note that CI pipeline changes are out of scope for this task (and if so, create a follow-up task for CI integration).
- **Verification:** Confirm the relevant files section includes CI workflow files or a documented deferral.

### Finding R1-A2-8
- **Section:** Specification (§ "Specification", audit trail)
- **Severity:** MINOR
- **Finding:** The spec states "attestation events integrated into audit chain" (via the security audit service) but does not explicitly state that all attestation log entries and audit records identify nodes exclusively by Rep C address (INVARIANT 9). Dashboard displays, log schemas, and correlation across log sources must use Rep C as the join key — no hostname, IP, or Windows SID.
- **Recommendation:** Add: "All attestation audit records, log entries, and dashboard displays MUST identify nodes by Rep C address exclusively. No hostname, IP address, or Windows SID may appear as a node identifier in any attestation-related log or display."
- **Verification:** Confirm log schema definitions include a Rep C address field and that no alternative identifiers are used for node correlation.

### Finding R1-A2-9
- **Section:** Done looks like (§ "Done looks like")
- **Severity:** MINOR
- **Finding:** The acceptance criteria do not include a reproducibility requirement — there is no statement that the attestation service can be built and deployed from a single command, that the build is deterministic, or that build artifacts are checksummed. For a service that produces cryptographic attestation reports, build reproducibility is essential to ensure the "kernel integrity hash" field is meaningful.
- **Recommendation:** Add to "Done looks like": "The attestation service builds deterministically from a single command; build artifacts are checksummed with TIS-27; the kernel integrity hash is reproducible across identical source inputs."
- **Verification:** Run two consecutive builds from the same commit and confirm identical TIS-27 checksums for all output artifacts.

---

## Summary Verdict

**PASS WITH CONDITIONS**

The Continuous Attestation Service specification is architecturally sound — it correctly layers cryptographic integrity on top of existing operational health monitoring, properly distinguishes partition-induced unreachability from attestation failure, and uses native PlenumNET primitives (TL-DSA, TLSponge-385, HPTP) throughout. However, from a DevOps perspective, the spec has four IMPORTANT gaps that must be resolved before implementation: (1) no build/deployment pipeline definition — the service has no reproducible build command, no named artifact, and no deployment mechanism; (2) no automated test plan — complex behaviors like partition-aware suspicion counters and per-link backoff have no defined CI validation; (3) incomplete failure mode coverage — TL-DSA signing failure, PUF unavailability, and HPTP unreachability have no defined outcomes, creating silent failure paths; (4) no CI gate for schema version registry synchronization — firmware releases could ship with unregistered schema versions, causing fleet-wide "unparseable" alerts. The MINOR findings (Rep C context binding, CI workflow files, audit trail identifiers, build reproducibility) are improvements that can be addressed iteratively. No CRITICAL findings were identified. All four IMPORTANT findings should be resolved before first release.
