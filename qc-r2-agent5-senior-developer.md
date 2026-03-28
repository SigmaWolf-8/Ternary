# QC-R2 Agent 5: Senior Developer Review

**Reviewer:** Agent 5 — Senior Developer
**YODA Role ID:** `engineering/senior-developer`
**Review Target:** QC-R1 skill files (main template + 3 standalone agent skills)
**Reference:** QC-R2 template (for structural alignment comparison)
**Date:** 2026-03-28

---

## Round 1 Response

QC-R1 findings not provided — Round 1 Response not applicable for standalone invocation.

---

## Findings

### Finding 1
- **Section:** QC-R1 frontmatter — `version` field
- **Severity:** IMPORTANT
- **Finding:** The QC-R1 main template and all three standalone skills are at version `1.0.0`, while the QC-R2 template is at version `1.2.0`. The QC-R2 standalone skills (evidence-collector, senior-developer, infrastructure-maintainer) are referenced via `path:` fields relative to the QC-R2 directory (e.g., `evidence-collector/SKILL.md`), but in the actual filesystem these live at `.agents/skills/evidence-collector/SKILL.md` — siblings, not children of `qc-r2-review/`. The QC-R1 template uses the same `path:` pattern (`security-engineer/SKILL.md`), and the same filesystem layout holds: the standalone skills are siblings at `.agents/skills/security-engineer/`, not children of `.agents/skills/qc-r1-review/`. This means the `path:` values in the `components` frontmatter are relative to `.agents/skills/`, not relative to the template's own directory. This is consistent across R1 and R2 and appears intentional, but is undocumented. If a loader naively resolves paths relative to the template file's directory, all component references will fail.
- **Recommendation:** Add a `base_path` or `root` field to the frontmatter (e.g., `base_path: .agents/skills/`) or document the path resolution convention in a `CONVENTIONS.md` or within the frontmatter comment. Alternatively, use explicit relative paths with `../` prefix (e.g., `../security-engineer/SKILL.md`).
- **Verification:** A loader implementation that resolves `components[].path` relative to the template directory should successfully find each standalone SKILL.md, or fail with a clear error message explaining the resolution strategy.

### Finding 2
- **Section:** QC-R1 frontmatter — `schema_version` field
- **Severity:** MINOR
- **Finding:** The QC-R2 template includes a `schema_version` field (implied by its version 1.2.0 structure and machine-readable output spec). The QC-R1 frontmatter does not include a `schema_version` field. Cross-referencing the QC-R3 and other skill files in the repository (brand-guardian, content-creator, ux-designer), many include `schema_version: 1` in their frontmatter. QC-R1 and its three standalone skills lack this field entirely.
- **Recommendation:** Add `schema_version: 1` to the frontmatter of all four QC-R1 files to align with the convention used by QC-R3 and the standalone R2/R3 skills.
- **Verification:** `grep -c "schema_version" .agents/skills/qc-r1-review/SKILL.md` returns 1. Same for all three standalone files.

### Finding 3
- **Section:** QC-R1 main template — Prerequisites and Input Contract
- **Severity:** IMPORTANT
- **Finding:** The QC-R2 template includes a detailed "Prerequisites and Input Contract" section specifying three required inputs (`spec_file`, `qc_r1_findings`, `output_file`) and defines explicit error behavior when inputs are missing. The QC-R1 template has no equivalent section. An invoking agent receives only the instruction `run QC-R1 against [spec file]` with no specification of what parameters the template expects, what the output file naming convention is, or what happens if the spec file doesn't exist or is empty.
- **Recommendation:** Add a "Prerequisites and Input Contract" section to the QC-R1 template specifying: (1) `spec_file` (required): path to the specification under review, must exist and be non-empty; (2) `output_file` (required): path where each agent writes its review, following the naming convention `qc-r1-agent{N}-{role}.md`; (3) error behavior: if the spec file is missing or empty, produce a clear error rather than a silent empty review.
- **Verification:** The QC-R1 template contains a "Prerequisites and Input Contract" section with at least `spec_file` and `output_file` parameters defined.

### Finding 4
- **Section:** QC-R1 main template — Machine-Readable Output specification
- **Severity:** IMPORTANT
- **Finding:** The QC-R2 template includes a "Machine-Readable Output" subsection within the Review Protocol, specifying exact formatting rules: finding headers use `### Finding [N]` with bare integers, each field occupies one line, no blank lines within a finding block, and supplementary artifacts follow the Summary Verdict. The QC-R1 template provides the finding format as a code block example but does not specify these machine-readability constraints. Since QC-R1 output is consumed as input by QC-R2 (per the pipeline design), inconsistent formatting from R1 agents could break R2's ability to parse findings.
- **Recommendation:** Add a "Machine-Readable Output" subsection to the QC-R1 Review Protocol section, matching the QC-R2 constraints. At minimum: (1) Finding headers use `### Finding [N]` where N is a bare integer; (2) Each field occupies one line unless continued with a leading two-space indent; (3) No blank lines within a finding block; (4) Summary Verdict is a separate section after all findings.
- **Verification:** Compare the QC-R1 and QC-R2 "Machine-Readable Output" sections — they should specify identical structural rules for finding blocks.

### Finding 5
- **Section:** QC-R1 main template — Consolidation section
- **Severity:** IMPORTANT
- **Finding:** The QC-R1 Consolidation section specifies a summary table format and states findings are passed to QC-R2 and QC-R3, but does not define: (1) the output file naming convention for individual agent reviews (QC-R2 specifies `qc-r1-agent{N}-{role}.md` in its Consolidation Input Contract); (2) whether consolidation is performed by a script, a designated agent, or the invoking user; (3) the directory where output files should be written. The QC-R2 template's "Consolidation Input Contract" references `qc-r1-agent{N}-{role}.md` — but this naming convention is defined only in the R2 template, not in R1 where the files are actually produced.
- **Recommendation:** Add an "Output Contract" subsection to the QC-R1 Consolidation section that specifies: (1) individual review output naming convention: `qc-r1-agent{N}-{role}.md`; (2) consolidated output naming convention; (3) output directory (or parameter); (4) that the consolidation is expected to be performed by a Consolidation Agent or automated script.
- **Verification:** The QC-R1 template defines the same `qc-r1-agent{N}-{role}.md` naming convention that QC-R2's Consolidation Input Contract expects.

### Finding 6
- **Section:** Standalone skills — canonical-source versioning
- **Severity:** MINOR
- **Finding:** All three standalone skills include the note: "The protocol text below is reproduced from the canonical source (`qc-r1-review/SKILL.md` § Review Protocol, version 1.0.0) for standalone use. In case of conflict, the SKILL.md version governs." This is well-structured. However, the version reference is hardcoded as `version 1.0.0`. When the QC-R1 template version increments (e.g., to 1.1.0 or 1.2.0 per the recommendations in this review), the standalone files will reference a stale version unless manually updated. There is no mechanism to detect or prevent drift.
- **Recommendation:** Either (1) add a CI lint step that verifies the canonical-source version string in each standalone matches the current version in the main template's frontmatter, or (2) document in the QC-R1 template that all standalone canonical-source references must be updated whenever the main template version changes.
- **Verification:** After a version bump on qc-r1-review/SKILL.md, `grep "version 1.0.0" .agents/skills/security-engineer/SKILL.md` returns 0 matches (i.e., the reference was updated to the new version).

### Finding 7
- **Section:** QC-R1 main template — PlenumNET Invariants Referenced
- **Severity:** MINOR
- **Finding:** The QC-R2 template includes a "PlenumNET Invariants Referenced" section that explicitly lists INVARIANT 7 (TL-DSA mandatory) and INVARIANT 8 (no raw binary integers into sponge absorb), with full descriptions. The QC-R1 template references these invariants implicitly within the agent review scopes (e.g., Agent 1's "Cryptographic correctness" and Agent 3's "Cryptographic primitive selection"), but does not consolidate them into a named invariant section. This creates an asymmetry: R2 agents can reference invariants by number, while R1 agents must reference them by description.
- **Recommendation:** Add a "PlenumNET Invariants Referenced" section to the QC-R1 template, listing at minimum INVARIANT 7 and INVARIANT 8, matching the QC-R2 format. This enables R1 agents to reference invariants by number in their findings, which improves machine-parseability and cross-round traceability.
- **Verification:** The QC-R1 template contains a "PlenumNET Invariants Referenced" section with numbered invariants matching QC-R2's list.

### Finding 8
- **Section:** QC-R1 main template — Cross-round handoff (R1 → R2)
- **Severity:** IMPORTANT
- **Finding:** The QC-R1 template states findings are "passed as input to QC-R2" but does not specify the handoff format. The QC-R2 template expects `qc_r1_findings` as a "Markdown file containing findings in the standard Finding format." However, R1's Consolidation section produces a summary table, not the full findings. The R2 "Source Documents" section says "Read both documents in full" (implying the full R1 output), while the R2 Input Contract says `qc_r1_findings` is a single file. It's ambiguous whether R2 receives: (a) the consolidated summary table only, (b) the full individual agent reviews, or (c) a single merged document. This ambiguity exists in both templates and will cause implementation divergence.
- **Recommendation:** Specify in QC-R1's Consolidation section that the consolidation output includes both the summary table AND the full agent findings in a single Markdown file, with a defined structure (e.g., summary table first, then each agent's full review as a subsection). Mirror this in QC-R2's Input Contract.
- **Verification:** A test invocation of QC-R1 consolidation produces a single file that, when provided to QC-R2 as `qc_r1_findings`, allows each R2 agent to reference specific R1 findings by Finding ID (e.g., "R1-Finding-3") without ambiguity.

### Finding 9
- **Section:** Standalone skills — IMPORTANT severity definition gap
- **Severity:** MINOR
- **Finding:** The QC-R2 template's severity definitions include an additional clause for IMPORTANT: "IMPORTANT findings that are deferred past first release require explicit sign-off from the Security Engineer and a documented risk acceptance." The QC-R1 template and all three standalone skills define IMPORTANT simply as "should be resolved before first product release" without specifying what happens if they aren't. This creates an asymmetry: R2 has a deferral mechanism; R1 does not.
- **Recommendation:** Add the deferral clause to the QC-R1 severity definition for IMPORTANT, or document that the deferral mechanism is a R2-specific addition that applies retroactively to R1 IMPORTANT findings.
- **Verification:** The IMPORTANT severity definition in QC-R1 either matches QC-R2's definition or explicitly states the relationship.

### Finding 10
- **Section:** QC-R1 main template — R1 FAIL Verdict handling
- **Severity:** IMPORTANT
- **Finding:** The QC-R2 template includes an "R1 FAIL Verdict Persistence" section and a "Cross-Round CRITICAL Disagreement Resolution" section that define what happens when R1 and R2 disagree on findings. The QC-R1 template does not define what happens if one of its own agents issues a FAIL verdict. Does the spec still proceed to R2? Does it require re-review? The QC-R2 template answers this question (yes, R1 FAIL persists and blocks), but the QC-R1 template itself is silent on the matter. A developer implementing the R1 pipeline would have to read R2 to understand R1's failure semantics.
- **Recommendation:** Add a "Verdict Semantics" section to QC-R1 that specifies: (1) if any R1 agent issues FAIL, the spec does not proceed to R2 until the CRITICAL findings are resolved and the failing agent re-reviews; (2) PASS WITH CONDITIONS documents the conditions as mandatory pre-R2 gates; (3) reference the R2 template for cross-round disagreement resolution.
- **Verification:** The QC-R1 template contains explicit rules for what happens after each verdict outcome (PASS, PASS WITH CONDITIONS, FAIL) before the spec is submitted to R2.

### Finding 11
- **Section:** Standalone skills — missing cross-reference guidance for security findings
- **Severity:** MINOR
- **Finding:** The QC-R2 template includes a rule in Agents 4, 5, and 6: "If you identify a finding that involves credential exposure, cryptographic weakness, privilege escalation, or authentication bypass, flag it with a cross-reference to the Security Engineer (Agent 1) for severity assessment." No equivalent rule exists in the QC-R1 standalone skills for Agents 2 and 3. Agent 2 (DevOps Automator) could encounter credential-handling issues in CI pipelines; Agent 3 (PlenumNET Integration) could encounter cryptographic implementation issues. Neither is instructed to cross-reference Agent 1.
- **Recommendation:** Add a cross-reference rule to the Critical Rules sections of both the devops-automator and plenumnet-integration standalone skills: "If you identify a finding that involves credential exposure, cryptographic weakness, privilege escalation, or authentication bypass, flag it with a cross-reference to the Security Engineer (Agent 1) for severity assessment."
- **Verification:** Both standalone skills contain the cross-reference rule in their Critical Rules section.

### Finding 12
- **Section:** QC-R1 frontmatter — `components[].path` resolution
- **Severity:** CRITICAL
- **Finding:** The QC-R1 `components` field lists paths as `security-engineer/SKILL.md`, `devops-automator/SKILL.md`, and `plenumnet-integration/SKILL.md`. These paths are relative to `.agents/skills/`, the parent of the template's own directory. However, no path resolution strategy is documented. If a skill loader resolves these paths relative to the template file's location (`.agents/skills/qc-r1-review/`), it will look for `.agents/skills/qc-r1-review/security-engineer/SKILL.md`, which does not exist. The same issue exists in QC-R2's frontmatter. This is a structural ambiguity that will cause the first loader implementation to fail.
- **Recommendation:** Either (1) change the paths to use `../` prefix (e.g., `../security-engineer/SKILL.md`) making them explicitly relative to the template file, or (2) define a `skill_root` field in the frontmatter that specifies the base directory for path resolution (e.g., `skill_root: .agents/skills/`), or (3) document in a project-level conventions file that all `components[].path` values are resolved relative to the skill root, not the template file.
- **Verification:** A naive loader implementation resolves all three component paths to existing files without requiring special-case logic or undocumented conventions.

---

## Summary Verdict

**PASS WITH CONDITIONS**

The QC-R1 skill files are structurally sound and internally consistent across the main template and its three standalone agent skills. The content is well-defined, the review scopes are comprehensive, and the standalone canonical-source versioning pattern is a solid architectural choice. However, there are five conditions that should be resolved before first release:

1. **Component path resolution** (Finding 12, CRITICAL): The `components[].path` values in the frontmatter are ambiguous — they could be resolved relative to the template file or relative to the skill root. This must be disambiguated before any loader implementation can reliably consume these files.

2. **Input/output contract** (Findings 3, 5): QC-R1 lacks the input contract and output naming conventions that QC-R2 expects to receive. This gap will cause implementation friction at the R1→R2 handoff boundary.

3. **Machine-readable output spec** (Finding 4): Without formatting constraints matching QC-R2's expectations, R1 output may not be parseable by R2's consolidation logic.

4. **R1 FAIL verdict semantics** (Finding 10): The QC-R1 template does not define its own failure semantics, relying on R2 to define them retroactively. This creates a circular dependency in the spec.

5. **Cross-round handoff format** (Finding 8): The format of the consolidated R1 output that R2 consumes is ambiguous, risking implementation divergence.

The MINOR findings (2, 6, 7, 9, 11) are genuine improvements but do not block implementation.

---

## Feasibility Risk Table

| Structural Element | Risk | Justification |
|---|---|---|
| QC-R1 main template frontmatter | MEDIUM | Component paths require undocumented resolution strategy; missing `schema_version` |
| QC-R1 Review Protocol | LOW | Well-defined finding format; identical to QC-R2 structure |
| QC-R1 Agent 1 (Security Engineer) scope | LOW | Comprehensive, actionable, no ambiguity in review instructions |
| QC-R1 Agent 2 (DevOps Automator) scope | LOW | Comprehensive, actionable, minor gap in cross-reference guidance |
| QC-R1 Agent 3 (PlenumNET Integration) scope | LOW | Comprehensive, actionable, minor gap in cross-reference guidance |
| Standalone skill: security-engineer | LOW | Clean extraction from main template; canonical-source note is correct |
| Standalone skill: devops-automator | LOW | Clean extraction from main template; canonical-source note is correct |
| Standalone skill: plenumnet-integration | LOW | Clean extraction from main template; canonical-source note is correct |
| QC-R1 Consolidation section | HIGH | Missing output contract, naming convention, and handoff format specification |
| QC-R1 → QC-R2 pipeline handoff | HIGH | Ambiguous consolidation format; R2 Input Contract references conventions not defined in R1 |
| QC-R1 FAIL verdict handling | MEDIUM | No defined behavior; implementer must read R2 to understand R1 failure semantics |
| Canonical-source version tracking | LOW | Hardcoded version strings create drift risk; low impact unless templates diverge |
| Severity definitions alignment (R1 vs R2) | LOW | Minor asymmetry in IMPORTANT deferral clause; non-blocking |
| PlenumNET Invariants cross-referencing | LOW | Implicit in R1, explicit in R2; non-blocking but reduces traceability |
| Machine-readable output compliance | MEDIUM | R1 lacks formatting constraints that R2 consolidation may depend on |

---

*Capomastro Holdings Ltd. — Applied Physics Division*
*Sherwood Park, Alberta, Canada*
