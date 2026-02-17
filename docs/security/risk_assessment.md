# Ternary Kernel Security Infrastructure
## Risk Assessment — February 17, 2026

---

## Risk 1: Riscure DPA-C3 Findings Reveal Exploitable Side-Channel Leakage

**Probability**: 35%  
**Impact**: HIGH (undermines threat model residual risk scores, requires crypto remediation)  
**Timeline Impact**: No slip (findings independent of our roadmap), but triggers priority shift

**What Could Happen**:
- Riscure interim findings (late March) show significant power/timing leakage in AES-256-GCM or ML-KEM
- Leakage correlates with secret-dependent operations (hamming weight, branch timing)
- Current threat model assumes < 0.5/10 residual risk on cryptographic side-channels; actual may be 3–5/10

**Mitigation**:
- Riscure evaluation is the real test; we expected this uncertainty
- Have remediation candidates ready:
  - Masking schemes (boolean/arithmetic) for polynomial multiplication
  - Additional jitter injection during crypto operations
  - Hardware acceleration with constant-time guarantees
- Public messaging prepared: "Interim findings identified optimization opportunities; full remediation timeline TBD"
- Do NOT claim side-channel immunity until Riscure full report (June) validates fixes

**Owner**: Salvi (cryptography team) + Riscure (external)  
**Escalation Trigger**: Any finding with Pearson correlation |r| > 0.05 in first-order DPA

---

## Risk 2: Galois Formal Verification Audit Identifies Methodological Issues

**Probability**: 25%  
**Impact**: HIGH (proof framework may need rework, Q4 coverage target at risk)  
**Timeline Impact**: Potential June deadline slip (8-week audit could extend)

**What Could Happen**:
- Galois reviews proof methodology (Isabelle/HOL setup, k-induction strategy) and identifies gaps
- Assumptions in proofs don't hold in all edge cases (e.g., memory allocator proof assumes specific allocation order)
- Proof chain has logical gaps that require significant rework before external publication

**Mitigation**:
- Kickoff meeting (March 15) will clarify methodology before heavy lifting
- Schedule "design review" phase first (Galois recommends approach before we commit to proof rewrites)
- Have fallback: If full proofs take longer, publish "spot-check results" from auditors to maintain progress transparency
- Conservative estimate: 6-month audit timeline (March 15 – September 15) instead of 8 weeks

**Owner**: Salvi (proof team) + Galois (external auditor)  
**Escalation Trigger**: Any finding that requires > 20% rework of completed proofs

---

## Risk 3: Trail of Bits Penetration Testing Discovers Novel Exploit Chains

**Probability**: 95% (nearly certain; all pentests find things)  
**Impact**: MEDIUM (expected outcome; no surprise if vulnerabilities found)  
**Timeline Impact**: No slip; remediation prioritized based on severity

**What Could Happen**:
- Red team discovers:
  - Privilege escalation via TVM bytecode crafting + capability-system edge case
  - Supply-chain attack via SBOM signature validation bypass
  - HPTP jitter leakage enabling timing attacks
  - Baseband isolation MMU mapping vulnerability
- Severity ranges from "low priority hardening" to "critical, needs immediate patch"

**Mitigation**:
- Pentesting is designed to find this; it's the point
- Prioritization framework already defined:
  - **Critical** (immediate): Kernel privilege escalation, supply-chain bypass
  - **High** (30 days): Exploitable side-channels, capability system bypasses
  - **Medium** (90 days): Hardening improvements, optimization opportunities
  - **Low** (future): Design alternatives, architectural improvements
- Public update plan: "Trail of Bits pentest completed; high-severity findings remediated, report published post-embargo"

**Owner**: Salvi (security response) + Trail of Bits (external red team)  
**Escalation Trigger**: Any finding that enables kernel code execution

---

## Risk 4: Hardware Partner Delays (Osmocom, Secure Element Foundry)

**Probability**: 60% (delays common in hardware partnerships)  
**Impact**: MEDIUM (not on critical 2026 path; deferred to 2027)  
**Timeline Impact**: H2 2026 silicon tape-out could slip to Q1 2027

**What Could Happen**:
- Osmocom RISC-V port takes longer than estimated (2027 now → 2028)
- Secure element foundry selection delays (multiple vendors → single vendor decision takes months)
- NDA restrictions prevent publication of hardware details; roadmap opacity increases

**Mitigation**:
- Baseband (Osmocom) deferred to Phase 4 (non-critical 2026 deliverable)
- Secure element design is parallel path; design done, fabrication on schedule
- Publish non-sensitive hardware roadmap updates monthly (design % complete, partner status)
- Fallback: If foundry delays > 3 months, shift to FPGA prototype for validation (slower but unblocks testing)

**Owner**: Hardware team (internal) + foundry (external)  
**Escalation Trigger**: Secure element tape-out slips past Q4 2026

---

## Risk 5: Implementation-Documentation Mismatch

**Probability**: 40% (common in fast-moving projects)  
**Impact**: MEDIUM (audit credibility hit, but not security issue)  
**Timeline Impact**: No timeline slip; correction is straightforward

**What Could Happen**:
- Auditors review published threat model, run against actual kernel code
- Find discrepancies:
  - Claimed 1,500 events/sec throughput not validated on real hardware
  - HPTP auto-escalation thresholds don't match implemented code
  - Formal proof claims don't match actual Isabelle/HOL scripts
  - Risk scores based on threat model don't reflect actual attack surface

**Mitigation**:
- Pre-audit validation: Code team walks threat model document line-by-line
- Emphasis on what's measured vs. what's theoretical:
  - Measured: Backend API endpoints, database schema, code quality (0 issues)
  - Theoretical targets: Performance (p99 < 210ms), throughput (1,500 events/sec), risk scores (mean 1.8/10)
- Flag targets as "performance targets" not "achieved metrics" in all documentation

**Owner**: Salvi (documentation accuracy owner)  
**Escalation Trigger**: Auditor finds material discrepancy (> 10% variance on claimed metrics)

---

## Risk 6: Team Capacity / Timeline Slips

**Probability**: 50% (internal dependencies always slip)  
**Impact**: MEDIUM (affects March/April deliverable milestones)  
**Timeline Impact**: 2-4 week slip in interim deliverables (not critical path)

**What Could Happen**:
- Proof team gets blocked on Isabelle/HOL tooling issues (tool bugs, version conflicts)
- Security team delayed on threat model review (competing priorities, audit feedback cycles)
- DevOps team extends benchmarking campaign (need more test scenarios)
- Scope creep: New threat vectors identified mid-March, need to revise threat model

**Mitigation**:
- March 15 Galois kickoff deadline is hard (contractual); all prep work due March 10
- Threat model v1.0 locked Feb 17 (published); updates go to v1.1 (post-audit)
- Benchmarking targets are targets (not blockers); missing them doesn't block evaluation
- Weekly risk sync: Flag blockers to leadership by end of week

**Owner**: Salvi (project lead)  
**Escalation Trigger**: Any critical path item (Galois prep, threat model review) shows > 3-day slip

---

## Risk 7: External Auditor Availability / Quality

**Probability**: 15% (low; contracts signed)  
**Impact**: MEDIUM (audit delays or superficial review)  
**Timeline Impact**: Potential 4-8 week slip in audit completion

**What Could Happen**:
- Galois team member turns over mid-engagement; new auditor needs ramp-up
- Riscure DPA-C3 lab is overbooked; interim findings delayed to April instead of late March
- Trail of Bits prioritizes another client; our pentest pushed to June instead of April
- Auditor findings are superficial (no deep investigation); report lacks credibility with third parties

**Mitigation**:
- Contracts are locked; SLAs for deliverable dates and quality standards are specified
- Escalation clauses: If interim findings delayed > 2 weeks, invoke penalty clauses
- Secondary auditor relationships established (Riscure backup: NewAE; Galois backup: Certora)
- Public communication: "External audit timelines subject to vendor availability; [vendor] responsible for any delays"

**Owner**: Salvi (vendor management)  
**Escalation Trigger**: Any auditor announces > 2-week delay to promised interim deliverable

---

## Risk 8: Threat Model Residual Risk Scores Are Optimistic

**Probability**: 45% (common; internal teams tend toward best-case)  
**Impact**: MEDIUM (credibility issue if external reviewers disagree)  
**Timeline Impact**: No slip; correction happens in post-audit updates

**What Could Happen**:
- Threat model claims mean residual risk 1.8/10 (post-mitigation)
- Auditors review and estimate 2.5–3.2/10 based on unproven mitigations
- Published figures look overstated when compared to external assessment
- Board/stakeholders lose confidence in risk quantification

**Mitigation**:
- Threat model published as "Capomastro internal assessment" not "independently validated"
- Clear labeling: "Post-mitigation scores assume successful remediation; empirical validation (Riscure, Galois) will refine estimates"
- Auditor assessment will be published alongside; transparency on disagreements
- Conservative re-baselining: Plan for 20% upward revision in residual risk post-audit

**Owner**: Security team + Salvi (risk quantification lead)  
**Escalation Trigger**: External audit estimates > 30% higher than threat model; communication plan required

---

## Risk 9: FIPS 140-3 Pre-Assessment Shows Large Compliance Gap

**Probability**: 30% (crypto modules always have gaps)  
**Impact**: MEDIUM (schedule impact, not security impact)  
**Timeline Impact**: FIPS Level 2 submission pushed to Q1 2027

**What Could Happen**:
- NIST pre-assessment identifies gaps:
  - Cryptographic algorithm implementation not fully compliant with FIPS specs
  - RNG entropy sources don't meet FIPS SP 800-90B
  - Firmware update mechanism doesn't meet secure loading requirements
  - Documentation incomplete or incorrect
- Remediation required before formal submission (6+ months additional work)

**Mitigation**:
- FIPS is Nice-to-Have (not Must-Have for 2026)
- Pre-assessment already planned Q2 (April–June); gives early warning
- Have gap mitigation backlog ready (algorithm updates, RNG redesign, firmware hardenening)
- Realistic expectation: FIPS Level 2 submission Q1 2027 (not Q4 2026)

**Owner**: Compliance team  
**Escalation Trigger**: Pre-assessment identifies > 15 material gaps; formal submission delayed > 6 months

---

## Risk Summary Table

| Risk | Probability | Impact | Timeline Impact | Owner | Trigger |
|------|-------------|--------|-----------------|-------|---------|
| Riscure finds side-channel leakage | 35% | HIGH | None (expected) | Crypto team | |r| > 0.05 DPA |
| Galois methodology gaps | 25% | HIGH | June slip possible | Proof team | > 20% rework needed |
| Trail of Bits finds exploits | 95% | MEDIUM | None (expected) | Security team | Code execution found |
| Hardware partner delays | 60% | MEDIUM | H2 2026 slip | Hardware team | Tape-out > 3mo slip |
| Implementation-doc mismatch | 40% | MEDIUM | None | Documentation team | > 10% variance |
| Internal team slip | 50% | MEDIUM | 2-4 week slip | Project lead | Critical path > 3 days |
| Auditor delays | 15% | MEDIUM | 4-8 week slip | Vendor mgmt | > 2 week interim slip |
| Risk scores optimistic | 45% | MEDIUM | None | Risk team | External > +30% |
| FIPS gaps | 30% | MEDIUM | Q1 2027 slip | Compliance | > 15 gaps found |

---

## Critical Path Items (No Slack)

These cannot slip without delaying everything else:

1. **Galois engagement kickoff (March 15)**: Proof methodology review locked in. Slip = entire audit timeline shifts.
2. **Threat model v1.0 (Feb 17)**: Published today; locking assumptions for audits. Changes require v1.1 post-audit.
3. **API endpoints operational**: Already done; backend locked in. Changes require versioning + backward compatibility.

---

## Nice-to-Have Items (Can Slip)

These have float in the schedule:

1. **Benchmarks (Apr 1 target)**: Performance goals; not hitting them doesn't block evaluation
2. **FIPS pre-assessment (Q2 target)**: Compliance track; can push to Q3 if other audits take priority
3. **Osmocom baseband (2027 target)**: Phase 4; no critical 2026 dependency

---

## Risk Owners & Escalation

**Salvi** (Overall project lead)
- Escalation: Board/stakeholders, executive leadership
- Authority: Priority decisions, remediation scope sign-off
- Cadence: Weekly risk sync, monthly board update

**Crypto Team** (Side-channel risk owner)
- Escalation: Salvi if Riscure findings > acceptable threshold
- Authority: Mitigation approach, timeline for fixes
- Cadence: Weekly during Riscure evaluation (late March – June)

**Proof Team** (Formal verification risk owner)
- Escalation: Salvi if Galois methodology review identifies rework scope
- Authority: Proof redesign, scope prioritization
- Cadence: Bi-weekly with Galois (March 15 – June 30)

**Security Team** (Pentest risk owner)
- Escalation: Salvi if Trail of Bits finds critical issues
- Authority: Remediation planning, vulnerability triage
- Cadence: Weekly during pentest (April – July)

**Hardware Team** (Silicon risk owner)
- Escalation: Salvi if tape-out timeline slips past Q4 2026
- Authority: Foundry selection, schedule negotiation
- Cadence: Monthly milestone tracking

---

**Risk posture**: Realistic and defensible. High risks are expected outcomes (pentests find things; audits validate). Mitigation plans are concrete. No surprises planned.