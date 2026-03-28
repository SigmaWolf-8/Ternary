# QC-R2 Agent 6: Infrastructure Maintainer — Review of QC-R1 Skill Files

**Reviewer:** Agent 6 — Infrastructure Maintainer
**YODA Role ID:** `support/infrastructure-maintainer`
**Date:** 2026-03-28
**Spec Under Review:** QC-R1 skill files (main template + 3 standalone agent skills)

**Files Reviewed:**
1. `.agents/skills/qc-r1-review/SKILL.md` (main QC-R1 template, v1.0.0)
2. `.agents/skills/security-engineer/SKILL.md` (Agent 1 standalone, v1.0.0)
3. `.agents/skills/devops-automator/SKILL.md` (Agent 2 standalone, v1.0.0)
4. `.agents/skills/plenumnet-integration/SKILL.md` (Agent 3 standalone, v1.0.0)

**Reference:** `.agents/skills/qc-r2-review/SKILL.md` (QC-R2 template, v1.2.0)

---

## Round 1 Response

QC-R1 findings not provided — Round 1 Response not applicable for standalone invocation.

---

## Findings

### Finding 1
- **Section:** Invocation (qc-r1-review/SKILL.md § Invocation)
- **Severity:** CRITICAL
- **Finding:** The QC-R1 invocation instruction is `run QC-R1 against [spec file]` — a single parameter. By contrast, the QC-R2 template defines a formal **Prerequisites and Input Contract** section specifying three required inputs (`spec_file`, `qc_r1_findings`, `output_file`), validation rules, and error behavior for missing inputs. QC-R1 has no input contract, no `output_file` parameter, and no instruction for what the agent should do if the spec file is missing or empty. An operator invoking QC-R1 has no guidance on where the output should be written, what naming convention to use, or what happens on invalid input. The QC-R2 consolidation agent expects files named `qc-r1-agent{N}-{role}.md` but this naming convention is defined only in QC-R2, not in QC-R1 itself.
- **Recommendation:** Add a **Prerequisites and Input Contract** section to `qc-r1-review/SKILL.md` mirroring the QC-R2 structure. Define: (1) `spec_file` (required, must exist and be non-empty), (2) `output_file` (required, with the naming convention `qc-r1-agent{N}-{role}.md`), (3) error behavior: if spec_file is missing or empty, produce a clear error message rather than a silent empty review. Propagate the output naming convention to each standalone skill file.
- **Verification:** Invoke QC-R1 with a missing spec_file and confirm the agent produces an explicit error. Invoke QC-R1 normally and confirm the output file follows the `qc-r1-agent{N}-{role}.md` naming convention.

### Finding 2
- **Section:** Invocation (standalone skill files — all three)
- **Severity:** CRITICAL
- **Finding:** None of the three standalone skill files (security-engineer, devops-automator, plenumnet-integration) contain an Invocation section at all. They define Identity, Review Protocol, Review Scope, Critical Rules, and Deliverable — but an agent loading one of these standalone skills has no instruction on how to invoke it. There is no example command, no input parameters, and no guidance on where to write the output. The `description` field in the YAML frontmatter mentions "standalone use" but the body of the document provides no standalone invocation path.
- **Recommendation:** Add an **Invocation** section to each standalone skill file. Minimum content: (1) invocation command pattern (e.g., `run security-review against [spec file]`), (2) required inputs (`spec_file`, `output_file`), (3) error behavior for missing inputs. This should be brief — 3-5 lines — but must exist.
- **Verification:** Load each standalone skill in isolation and confirm a new agent can determine exactly what command to run, what inputs to provide, and where the output goes.

### Finding 3
- **Section:** Review Protocol (qc-r1-review/SKILL.md § Review Protocol)
- **Severity:** IMPORTANT
- **Finding:** The QC-R1 review protocol does not define machine-readable output rules. QC-R2 (§ Machine-Readable Output) specifies: finding headers use `### Finding [N]` with bare integer N, each field occupies one line (with two-space indent for continuation), no blank lines within a finding block, Summary Verdict is a separate section. QC-R1 has none of these formatting constraints. If QC-R1 agents produce free-form findings, the QC-R2 consolidation agent and R2 agents (who must parse R1 findings for the Round 1 Response section) may fail to parse them. This is a cross-round compatibility gap.
- **Recommendation:** Add a **Machine-Readable Output** subsection to QC-R1's Review Protocol, matching or referencing the QC-R2 formatting rules. At minimum: bare integer finding numbers, one field per line, no blank lines within a finding block.
- **Verification:** Generate a QC-R1 output following the new rules and feed it into a QC-R2 agent as `qc_r1_findings`. Confirm the R2 agent can parse every finding and reference it by ID in its Round 1 Response.

### Finding 4
- **Section:** Review Protocol (qc-r1-review/SKILL.md § Review Protocol)
- **Severity:** IMPORTANT
- **Finding:** The QC-R1 protocol does not define a Finding ID scheme that is referenceable across rounds. QC-R2's Round 1 Response section instructs agents to address each CRITICAL finding by its "Finding ID (e.g., 'R1-Finding-3')". However, QC-R1 findings use the format `### Finding [N]` with N as a bare integer, and there is no instruction to prefix findings with `R1-` or include the agent number. If Agent 1 and Agent 3 both produce a "Finding 3", QC-R2 agents cannot disambiguate. The cross-round reference breaks.
- **Recommendation:** Define a Finding ID convention in QC-R1: `R1-A{agent_number}-{finding_number}` (e.g., `R1-A1-3` for Agent 1, Finding 3). Document this convention in the QC-R1 Review Protocol section and update the consolidation table format to use it.
- **Verification:** Produce a QC-R1 output with the new ID convention. Confirm that QC-R2 agents can unambiguously reference each finding in their Round 1 Response.

### Finding 5
- **Section:** Review Protocol (qc-r1-review/SKILL.md § Review Protocol)
- **Severity:** IMPORTANT
- **Finding:** The review protocol does not address the "zero findings" edge case. If an agent reviews the spec and finds no issues, is a review with zero findings and a PASS verdict valid? Or is zero findings a suspicious signal that should be flagged? QC-R2 does not address this either, but as an operator, receiving a review with zero findings and no explanation is a red flag — it could mean the agent didn't actually read the document.
- **Recommendation:** Add a protocol rule: "If the review produces zero findings, the agent must include a brief statement confirming the document was reviewed in full and explaining why no findings were produced. A zero-finding review is a valid state but must not be empty — the Summary Verdict paragraph serves as the minimum content."
- **Verification:** Invoke QC-R1 against a spec that is fully compliant. Confirm the output includes a Summary Verdict with a substantive justification even when there are zero findings.

### Finding 6
- **Section:** Consolidation (qc-r1-review/SKILL.md § Consolidation)
- **Severity:** IMPORTANT
- **Finding:** The QC-R1 consolidation section is minimal — it defines a table format but does not specify: (1) who produces the consolidated table (a fourth agent? the invoking operator? a script?), (2) whether consolidation is automatic or manual, (3) where the consolidated output is written, (4) whether deduplication should occur at the R1 stage. By contrast, QC-R2 defines a "Consolidation Agent (or script)" with a full input contract and naming convention. The QC-R1 consolidation is underspecified for operational use.
- **Recommendation:** Expand the Consolidation section to specify: (1) the consolidation actor (agent, script, or operator), (2) the output file naming convention (e.g., `qc-r1-consolidated.md`), (3) the consolidation input contract (expects 3 files named `qc-r1-agent{N}-{role}.md`), (4) whether deduplication is performed at R1 or deferred to R2.
- **Verification:** Execute a full QC-R1 review and confirm the consolidation step can be performed unambiguously by a new operator following only the documented instructions.

### Finding 7
- **Section:** YAML Frontmatter (qc-r1-review/SKILL.md vs. standalone files)
- **Severity:** IMPORTANT
- **Finding:** The main QC-R1 template is at version 1.0.0 while the QC-R2 template is at version 1.2.0. The standalone skill files also report version 1.0.0. There is no `schema_version` field in the QC-R1 frontmatter (QC-R2 does not have one either in the file reviewed, but some attached assets reference `schema_version: 1`). Without a schema version, an operator cannot determine whether a skill file's frontmatter structure has changed between versions or whether a newer template version requires structural changes to existing standalone files.
- **Recommendation:** Add a `schema_version` field to the QC-R1 frontmatter. Define the contract: if `schema_version` changes, standalone skill files must be re-validated for structural compatibility. If `schema_version` stays the same, only content changes within the existing structure.
- **Verification:** Increment the version to 1.1.0 with the fixes from this review. Confirm that `schema_version` is present and that standalone files reference the same schema version.

### Finding 8
- **Section:** Standalone skill files — canonical source reference (all three)
- **Severity:** MINOR
- **Finding:** Each standalone skill file contains the sentence: "The protocol text below is reproduced from the canonical source (`qc-r1-review/SKILL.md` § Review Protocol, version 1.0.0) for standalone use. In case of conflict, the SKILL.md version governs." This is good practice. However, the version reference is hardcoded to "version 1.0.0". When the main template is updated to 1.1.0 or later, operators must remember to update this version reference in all three standalone files. There is no automation or validation for this.
- **Recommendation:** Either (1) add a CI check that verifies the version string in standalone files matches the main template version, or (2) change the reference to say "see canonical source for current version" without hardcoding the version number, relying on the `round: qc-r1` frontmatter field to establish the link.
- **Verification:** Update the main template version. Confirm that either the CI check catches the stale reference, or the reference text no longer includes a hardcoded version.

### Finding 9
- **Section:** Source Documents (qc-r1-review/SKILL.md § Source Documents)
- **Severity:** MINOR
- **Finding:** The Source Documents section contains a placeholder: "**Primary:** [insert specification filename and revision]". This is appropriate for a template. However, the standalone skill files do not reproduce this section and do not instruct the agent to identify the source document being reviewed. When used standalone, the agent may begin reviewing without first confirming which document and revision it is reviewing.
- **Recommendation:** Add a brief "Source Document" or "Input Document" field to the standalone skill files' Review Protocol section, instructing the agent to state the document and revision being reviewed at the top of its output.
- **Verification:** Invoke a standalone skill and confirm the output header identifies the specific document and revision reviewed.

### Finding 10
- **Section:** PlenumNET Invariants (qc-r1-review/SKILL.md — absent)
- **Severity:** MINOR
- **Finding:** QC-R2 includes a dedicated "PlenumNET Invariants Referenced" section listing INVARIANT 7 and INVARIANT 8 with full definitions. QC-R1 references these invariants implicitly (e.g., the PlenumNET Integration Specialist's Critical Rules mention "TIS-27 is the sole hash/MAC primitive") but does not formally enumerate them. This creates an asymmetry: R2 agents have a shared invariant vocabulary; R1 agents must derive invariants from scattered rules.
- **Recommendation:** Add a "PlenumNET Invariants Referenced" section to QC-R1, listing the invariants relevant to R1 agents (at minimum INVARIANT 7 and INVARIANT 8). This ensures R1 and R2 share the same invariant numbering and definitions.
- **Verification:** Compare the invariant references in QC-R1 and QC-R2. Confirm they use the same numbering and definitions.

### Finding 11
- **Section:** Consolidation (qc-r1-review/SKILL.md § Consolidation)
- **Severity:** MINOR
- **Finding:** The consolidation table format in QC-R1 uses columns `# | Agent | Section | Severity | Finding (Summary)`. The QC-R2 Final Consolidation section specifies a "Combined Finding Table" that is "sorted by severity (CRITICAL → IMPORTANT → MINOR), then by section number ascending within each severity level" with deduplication. QC-R1 does not specify sort order or deduplication rules for its own consolidation table. While this is not blocking (R2 handles the final merge), it means the R1 consolidated output may arrive at R2 in an inconsistent format.
- **Recommendation:** Add sort order and deduplication guidance to the QC-R1 consolidation section, or explicitly state that R1 consolidation is unsorted and R2 handles normalization.
- **Verification:** Produce a QC-R1 consolidated table and feed it to QC-R2. Confirm R2 can process it regardless of sort order.

---

## Summary Verdict

**PASS WITH CONDITIONS**

The QC-R1 skill files are structurally sound and contain well-defined review scopes, severity definitions, and deliverable requirements for each of the three agent roles. The standalone skill files correctly reproduce the review protocol with a canonical-source reference. However, two CRITICAL gaps prevent reliable operational use: (1) the absence of a formal input contract and output naming convention in QC-R1 (which QC-R2 has), and (2) the absence of any invocation instructions in the standalone skill files. These gaps mean an operator or agent loading these skills for the first time cannot determine where to write output or how to invoke a standalone review without consulting external documentation. The IMPORTANT findings around machine-readable output formatting, Finding ID disambiguation, and consolidation specification represent cross-round compatibility risks that should be resolved before the first production review cycle. The MINOR findings are quality-of-life improvements that can be addressed iteratively.

**Conditions for PASS:**
1. Add a Prerequisites and Input Contract section to QC-R1 (Finding 1).
2. Add Invocation sections to all three standalone skill files (Finding 2).
3. Define machine-readable output formatting rules for QC-R1 (Finding 3).
4. Define a cross-round Finding ID convention (Finding 4).

---

## Operator Readiness Checklist

| # | Operator Question | Answered? | Reference |
|---|---|---|---|
| 1 | How do I invoke a QC-R1 review? | PARTIALLY | Invocation section exists but lacks input contract, output path, error behavior |
| 2 | What inputs does QC-R1 require? | PARTIALLY | Only `[spec file]` is mentioned; no output_file, no validation rules |
| 3 | Where does the QC-R1 output go? | NO | No output_file parameter or naming convention defined |
| 4 | What naming convention should output files use? | NO | Defined only in QC-R2's consolidation input contract, not in QC-R1 |
| 5 | Can I use a standalone skill (e.g., security-engineer) independently? | PARTIALLY | Frontmatter says yes; body has no invocation instructions |
| 6 | What happens if my spec file is empty or missing? | NO | No error behavior defined |
| 7 | How do I know if the review output is machine-parseable? | NO | No machine-readable output rules in QC-R1 |
| 8 | Will my QC-R1 output be consumable by QC-R2 agents? | PARTIALLY | Format is compatible but Finding IDs may collide across agents |
| 9 | How do I consolidate the three R1 reviews? | PARTIALLY | Table format defined; who does it, where it goes, and sort order are not |
| 10 | How do I update a skill file without breaking the others? | PARTIALLY | Version field exists; no schema_version or CI check for cross-file consistency |
| 11 | What PlenumNET invariants apply to QC-R1 reviews? | PARTIALLY | Embedded in agent rules but not formally enumerated as in QC-R2 |
| 12 | What does a "zero findings" review look like? | NO | No guidance on valid zero-finding output |
| 13 | Is the review protocol identical across QC-R1 and QC-R2? | PARTIALLY | Same finding format; QC-R2 adds machine-readable rules, Round 1 Response, and security-domain gating that QC-R1 does not anticipate |
| 14 | What version of the template am I running? | YES | YAML frontmatter `version: 1.0.0` and `last_updated` in all files |
| 15 | Who is the canonical source if standalone and main template conflict? | YES | Each standalone file states the main SKILL.md governs |

---

*Capomastro Holdings Ltd. — Applied Physics Division*
*Sherwood Park, Alberta, Canada*
