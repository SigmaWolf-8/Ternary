# QC-R2 — Agent 4: Evidence Collector / QA Lead Review

**YODA Role ID:** `testing/evidence-collector`
**Spec Under Review:** QC-R1 Skill Files (v1.0.0, 2026-03-28)
**Review Date:** 2026-03-28

---

## Files Reviewed

| # | File | Version | Last Updated |
|---|------|---------|--------------|
| 1 | `.agents/skills/qc-r1-review/SKILL.md` | 1.0.0 | 2026-03-28 |
| 2 | `.agents/skills/security-engineer/SKILL.md` | 1.0.0 | 2026-03-28 |
| 3 | `.agents/skills/devops-automator/SKILL.md` | 1.0.0 | 2026-03-28 |
| 4 | `.agents/skills/plenumnet-integration/SKILL.md` | 1.0.0 | 2026-03-28 |

---

## Findings

### Finding 1
- **Section:** Review Protocol (all files) — Summary Verdict
- **Severity:** CRITICAL
- **Finding:** The Summary Verdict offers three outcomes (PASS, PASS WITH CONDITIONS, FAIL) but provides no objective criteria for choosing between them. There is no threshold (e.g., "FAIL if any CRITICAL finding exists; PASS WITH CONDITIONS if only IMPORTANT/MINOR findings exist"). Two agents reviewing the same spec could produce different verdicts for identical findings because the decision boundary is undefined.
- **Recommendation:** Add explicit decision rules to the Review Protocol section in `qc-r1-review/SKILL.md` and propagate to all standalone skills. Example: "FAIL: one or more CRITICAL findings. PASS WITH CONDITIONS: one or more IMPORTANT findings, zero CRITICAL. PASS: only MINOR findings or none."
- **Verification:** Present two agents with the same set of findings (e.g., 0 CRITICAL, 2 IMPORTANT, 3 MINOR) and confirm both produce the same verdict under the new rules.

### Finding 2
- **Section:** Frontmatter — `schema_version` field
- **Severity:** IMPORTANT
- **Finding:** The standalone skill files include a `round: qc-r1` frontmatter field that does not appear in the main template's frontmatter. Conversely, the main template has a `components` array and the standalone files do not cross-reference back to the parent. There is no `schema_version` field in any file, making it impossible to validate that frontmatter structures are compatible across versions.
- **Recommendation:** (a) Add `schema_version: 1` to all four files. (b) Add a `parent: qc-r1-review/SKILL.md` field to each standalone skill's frontmatter. (c) Document the `round` field in the main template's frontmatter specification so it is not an undocumented extension.
- **Verification:** Parse all four YAML frontmatters programmatically and assert: every standalone file has `parent` matching the main template path; every file has `schema_version`; the `round` field in standalone files matches a value listed in the main template.

### Finding 3
- **Section:** Agent 1 Review Scope, item 1 — Credential and secret handling
- **Severity:** IMPORTANT
- **Finding:** The credential handling checklist is comprehensive for Windows-centric vectors (NTFS ADS, UAC, ACLs) but does not address Linux/macOS credential handling vectors: `/proc/<pid>/environ` exposure, `ptrace` attach to read process memory, core dump configuration (`/proc/sys/kernel/core_pattern`), or `umask` inheritance for file-based secrets. Since PlenumNET targets multi-platform deployments, a reviewer executing this scope on a Linux-targeted spec would miss platform-specific credential leaks.
- **Recommendation:** Add a platform-agnostic sub-item: "Verify that credential handling instructions address OS-specific vectors for all target platforms. For Linux: `/proc/*/environ`, `ptrace` scope, core dump policy. For macOS: Keychain integration, `launchd` environment variable persistence. For Windows: NTFS ADS, DPAPI, crash dump configuration."
- **Verification:** Apply the updated checklist to a Linux-targeted spec and confirm the reviewer produces findings for `/proc/environ` exposure and core dump policy. Apply to a macOS-targeted spec and confirm Keychain integration is evaluated.

### Finding 4
- **Section:** Agent 1 Review Scope, item 4 — Upgrade and identity management
- **Severity:** IMPORTANT
- **Finding:** "Verify that product code derivation is deterministic and collision-resistant" is not testable as stated. There is no specification for what constitutes "collision-resistant" (probability bound? hash output length? namespace scope?). There is no procedure for an agent to verify determinism (run derivation twice with same inputs? Check the derivation function source?). An agent cannot produce a VERIFIED/UNVERIFIED/INCORRECT classification for this claim without a concrete test procedure.
- **Recommendation:** Define the expected derivation function (e.g., "UUID v5 with namespace X and input Y"), state the collision resistance bound (e.g., "2^-64 for the deployed population size"), and specify the determinism test: "Run derivation with identical inputs on two separate machines; outputs must be byte-identical."
- **Verification:** Execute the specified determinism test against the actual derivation code and confirm byte-identical outputs. Calculate collision probability for the stated population size and compare against the stated bound.

### Finding 5
- **Section:** Agent 2 Review Scope, item 3 — Deployment testing
- **Severity:** IMPORTANT
- **Finding:** The instruction "estimate CI time" for framework-changes-trigger-all-product-retesting is not a verifiable review step. It requires knowledge of the actual CI infrastructure (runner count, build times per product, parallelism factor) that is not provided in the spec being reviewed and is not available to an agent operating solely from the specification document. This instruction will produce inconsistent results across agents.
- **Recommendation:** Replace "estimate CI time" with a concrete check: "Verify that the spec defines the expected CI duration for full-matrix retesting, identifies the parallelism strategy, and specifies a maximum acceptable wall-clock time. If missing, file an IMPORTANT finding."
- **Verification:** Review a spec that omits CI duration estimates and confirm the agent produces a finding. Review a spec that includes CI duration estimates and confirm the agent validates them against the stated strategy.

### Finding 6
- **Section:** Agent 3 Review Scope, item 2 — Context strings and derivation formulas
- **Severity:** CRITICAL
- **Finding:** The instruction says to "verify exact context strings" but provides only two examples (`"PlenumNET-CON-v2.5"`, `"HEARTBEAT-MAC"`). There is no authoritative registry of valid context strings referenced or embedded. An agent cannot verify a context string is "correct" without access to a canonical list. If the spec under review uses a context string not in the two examples, the agent has no basis for VERIFIED vs. UNVERIFIED classification.
- **Recommendation:** (a) Create or reference a canonical context string registry (e.g., a file in the repo or a section in TM-2026-016). (b) Update this review scope item to say: "Verify exact context strings against the canonical registry at [path]. Any context string not in the registry is UNVERIFIED until added."
- **Verification:** Attempt to verify a context string not in the two examples (e.g., `"TDNS-RESOLVE-v2.3"`) against the registry. Confirm the agent correctly classifies it as UNVERIFIED if absent, or VERIFIED if present.

### Finding 7
- **Section:** Agent 3 Review Scope, item 3 — Key lifecycle boundaries
- **Severity:** IMPORTANT
- **Finding:** The 14-day rotation period is stated as `ARC_EPOCH_SECS / RADIAN_DEG` per TM-2026-016, but the actual numeric values of `ARC_EPOCH_SECS` and `RADIAN_DEG` are not provided. An agent cannot verify the formula produces 14 days (1,209,600 seconds) without these constants. If the constants change in a future spec revision, the 14-day claim becomes silently incorrect.
- **Recommendation:** State the expected constant values inline: "`ARC_EPOCH_SECS = 1,209,600` and `RADIAN_DEG = 1`, yielding a 14-day rotation period." Alternatively, reference the exact file and line where these constants are defined.
- **Verification:** Compute `ARC_EPOCH_SECS / RADIAN_DEG` with the stated values and confirm the result equals 1,209,600 seconds (14 days). Cross-check against the source file.

### Finding 8
- **Section:** Main template — Consolidation table
- **Severity:** MINOR
- **Finding:** The consolidation table format lists `# | Agent | Section | Severity | Finding (Summary)` but does not include a `Verification Status` column (VERIFIED / UNVERIFIED / INCORRECT). Agent 1 and Agent 3 are instructed to produce verification status for cryptographic claims, but the consolidation table loses this information. QC-R2 reviewers would need to dig into individual agent reports to find verification status.
- **Recommendation:** Add a `Crypto Status` column to the consolidation table: `# | Agent | Section | Severity | Finding (Summary) | Crypto Status`.
- **Verification:** Generate a consolidation table from a sample review that includes cryptographic findings. Confirm the `Crypto Status` column is populated for Agent 1 and Agent 3 findings, and marked N/A for Agent 2 findings.

### Finding 9
- **Section:** Main template — Invocation
- **Severity:** MINOR
- **Finding:** The invocation instruction says "provide: `run QC-R1 against [spec file]`" but does not specify whether all three agents run in parallel or sequentially, whether their outputs are concatenated into a single document or produced as three separate documents, or who is responsible for producing the consolidation table. This ambiguity could result in inconsistent execution.
- **Recommendation:** Add: "All three agents execute independently and in parallel. Each produces a separate review document. After all three complete, a consolidation step (automated or by a coordinator agent) merges findings into the consolidation table format defined in the Consolidation section."
- **Verification:** Invoke QC-R1 twice with the same spec. Confirm that both invocations produce three independent reports and one consolidation table, in the same structure.

### Finding 10
- **Section:** Standalone skills — Review Protocol provenance clause
- **Severity:** MINOR
- **Finding:** Each standalone skill includes the clause: "The protocol text below is reproduced from the canonical source (`qc-r1-review/SKILL.md` Section Review Protocol, version 1.0.0) for standalone use. In case of conflict, the SKILL.md version governs." This is good practice. However, the provenance clause hardcodes `version 1.0.0`. When the main template is updated to version 1.1.0, the standalone files will still claim reproduction from 1.0.0 unless manually updated. There is no mechanism to detect this drift.
- **Recommendation:** (a) Add a CI check or agent-level pre-flight step that compares the Review Protocol sections across all four files and flags textual divergence. (b) Change the provenance clause to reference a version range or "latest" with a hash, e.g., "reproduced from version 1.0.0 (hash: [first 8 chars of TIS-27 digest of the Review Protocol section])."
- **Verification:** Modify the Review Protocol in the main template only. Run the proposed CI check and confirm it flags the standalone files as out-of-date.

### Finding 11
- **Section:** Agent 1 Review Scope, item 5 — Key provisioning
- **Severity:** IMPORTANT
- **Finding:** "Verify that 'node identity' inputs to key derivation are precisely specified and cannot produce duplicate keys across nodes" is not testable without a formal definition of "node identity." The spec does not define the components of node identity (e.g., hardware serial, IP address, TDNS address, random nonce). Without this, an agent cannot determine whether the input space is large enough to prevent collisions or whether two nodes with similar configurations could derive the same key.
- **Recommendation:** Define node identity composition: "Node identity for key derivation consists of [Rep C 54-trit TDNS address] concatenated with [install-time 256-bit random nonce]. This produces a minimum of 2^256 unique derivation inputs, preventing duplicate keys with negligible probability."
- **Verification:** Review the key derivation code and confirm it uses the specified node identity components. Attempt to derive keys for two nodes with identical TDNS addresses but different nonces; confirm different keys. Attempt with identical nonces but different TDNS addresses; confirm different keys.

### Finding 12
- **Section:** All files — Frontmatter `references` field
- **Severity:** MINOR
- **Finding:** All four files reference `plenumnet-repo-guide/SKILL.md` in the `references` frontmatter field, but the path is relative and it is unclear whether this file exists at `.agents/skills/plenumnet-repo-guide/SKILL.md` or elsewhere. If the referenced file does not exist at the expected path, the reference is a dead link and cannot be used by agents for context loading.
- **Recommendation:** Verify the referenced file exists and use a consistent path convention (either absolute from repo root or relative from the skill file's own directory). Add a CI check that validates all `references` paths resolve to existing files.
- **Verification:** Attempt to read `.agents/skills/plenumnet-repo-guide/SKILL.md`. If it exists, PASS. If not, the reference is broken and must be corrected.

---

## Summary Verdict: PASS WITH CONDITIONS

The QC-R1 skill files form a well-structured, internally consistent review framework. The standalone skills faithfully reproduce the main template's review protocol, scope items, and finding format. The frontmatter is aligned in name, version, and date. Cross-references between standalone and parent are present via provenance clauses.

However, the framework has two CRITICAL gaps that would cause inconsistent or unverifiable results in production use:

1. **No objective criteria for Summary Verdict selection** (Finding 1) — agents will produce inconsistent verdicts for the same findings.
2. **No canonical context string registry** (Finding 6) — the PlenumNET Integration Specialist cannot verify the most critical security-relevant claim (correct context strings) without an authoritative source.

Additionally, six IMPORTANT findings identify areas where review instructions are insufficiently precise for deterministic, repeatable agent execution: platform-specific credential vectors (Finding 3), untestable collision-resistance claims (Finding 4), unverifiable CI time estimates (Finding 5), unstated formula constants (Finding 7), undefined node identity composition (Finding 11), and missing frontmatter schema versioning (Finding 2).

**Conditions for PASS:**
- Resolve both CRITICAL findings before first operational use of QC-R1.
- Resolve IMPORTANT findings before the first production release reviewed under QC-R1.

---

## Coverage Matrix

| Verifiable Claim | qc-r1-review | security-engineer | devops-automator | plenumnet-integration |
|---|---|---|---|---|
| Finding format (5-field structure) | COVERED | COVERED | COVERED | COVERED |
| Severity definitions (CRITICAL/IMPORTANT/MINOR) | COVERED | COVERED | COVERED | COVERED |
| Summary Verdict outcomes (PASS/PWC/FAIL) | PARTIALLY COVERED | PARTIALLY COVERED | PARTIALLY COVERED | PARTIALLY COVERED |
| Summary Verdict decision criteria | NOT COVERED | NOT COVERED | NOT COVERED | NOT COVERED |
| Credential handling — Windows vectors | COVERED | COVERED | N/A | N/A |
| Credential handling — Linux/macOS vectors | NOT COVERED | NOT COVERED | N/A | N/A |
| Cryptographic primitive exclusivity | COVERED | COVERED | COVERED | COVERED |
| Context string verification | COVERED | COVERED | N/A | PARTIALLY COVERED |
| Context string canonical registry | NOT COVERED | NOT COVERED | N/A | NOT COVERED |
| Key derivation formula verification | COVERED | COVERED | N/A | PARTIALLY COVERED |
| Key rotation period constants | N/A | N/A | N/A | PARTIALLY COVERED |
| Node identity composition | COVERED | PARTIALLY COVERED | N/A | N/A |
| Product code determinism test | COVERED | PARTIALLY COVERED | N/A | N/A |
| Product code collision resistance bound | NOT COVERED | NOT COVERED | N/A | N/A |
| Build reproducibility verification | COVERED | N/A | COVERED | N/A |
| CI pipeline failure mode coverage | COVERED | N/A | COVERED | N/A |
| CI duration estimation | NOT COVERED | N/A | PARTIALLY COVERED | N/A |
| Checksum primitive correctness (TIS-27) | COVERED | COVERED | COVERED | COVERED |
| Deployment test automatability | COVERED | N/A | COVERED | N/A |
| TDNS naming convention alignment | COVERED | N/A | N/A | COVERED |
| Cross-document consistency (TM-2026-016, Task #33) | COVERED | N/A | N/A | COVERED |
| Consolidation table crypto status | NOT COVERED | N/A | N/A | N/A |
| Standalone-to-parent consistency (provenance) | N/A | COVERED | COVERED | COVERED |
| Standalone-to-parent version drift detection | N/A | NOT COVERED | NOT COVERED | NOT COVERED |
| Frontmatter schema versioning | NOT COVERED | NOT COVERED | NOT COVERED | NOT COVERED |
| Reference path validation | NOT COVERED | NOT COVERED | NOT COVERED | NOT COVERED |
| Invocation execution model (parallel/sequential) | PARTIALLY COVERED | N/A | N/A | N/A |

---

*Capomastro Holdings Ltd. -- Applied Physics Division*
*Agent 4: Evidence Collector / QA Lead*
*YODA Role ID: testing/evidence-collector*
