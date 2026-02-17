# Ternary Kernel Security Infrastructure
## Detailed Phased Task List — February 17, 2026 through Q4 2026

---

## Phase 1: Immediate (Feb 17–24, 2026) — Stabilization & Validation

**Objective**: Verify infrastructure works, validate threat model accuracy, prepare for Galois engagement.

### Week 1 Tasks (Feb 17–24)

#### 1.1 Backend Infrastructure Smoke Test
- **Owner**: DevOps Lead
- **Task**: Populate security_audit_log and hptp_anomaly_events with synthetic data (100–500 events)
- **Acceptance Criteria**:
  - All 14 REST endpoints respond with correct HTTP status codes
  - Zod validation rejects malformed payloads (e.g., invalid severity enum)
  - JWT auth required; unauthenticated requests return 401
  - POST /audit/events succeeds; data appears in GET /audit/events
  - /api/security/dashboard returns aggregated stats (no empty fields)
- **Deliverable**: Test report (pass/fail per endpoint)
- **Risk Mitigation**: Validates Implementation-Documentation Mismatch (Risk 5)
- **Time**: 1 day

#### 1.2 Threat Model CVSS v4.0 Validation
- **Owner**: Security Lead + Salvi
- **Task**: Cross-check all 12 threat CVSS scores against FIRST CVSS v4.0 calculator and real-world examples
- **Acceptance Criteria**:
  - Each threat has Attack Vector / Attack Complexity / Privileges Required / User Interaction justified
  - Exploit Maturity (E) and Vulnerability Maturity (VM) explicitly noted (not assumed "probed")
  - Scope (S) or Bypass examples documented (e.g., "Thunderbolt DMA requires physical proximity")
  - Mean residual risk 1.8/10 defensible against auditor challenge
  - Document flagged "internal assessment" not independently validated
- **Deliverable**: CVSS scoring rationale document (1–2 lines per threat)
- **Risk Mitigation**: Addresses Risk 8 (Threat Model Residual Risk Scores Are Optimistic)
- **Time**: 1–2 days

#### 1.3 Internal Threat Model QA Review
- **Owner**: Security team (3-person review)
- **Task**: Code team walks threat_model.md line-by-line against actual kernel implementation
- **Acceptance Criteria**:
  - No discrepancies found between claimed mitigations and /src/kernel/ code
  - HPTP thresholds match /server/services/hptp-anomaly.service.ts
  - Capability system behavior matches /crypto/capability.rs
  - Risk scores reflect actual attack surface (not aspirational)
  - Flag any gaps for v1.1 post-audit (no changes to v1.0)
- **Deliverable**: "Threat Model Accuracy Audit" checklist (pass/fail + findings)
- **Risk Mitigation**: Addresses Risk 5 (Implementation-Documentation Mismatch)
- **Time**: 2 days

#### 1.4 Galois Engagement Prep
- **Owner**: Salvi + Proof Team Lead
- **Task**: Bundle Galois pre-engagement materials
- **Acceptance Criteria**:
  - /proofs/*.thy files organized with comments
  - proofs.md linked to actual file paths
  - Assumptions log created (e.g., "k-induction depth: 15, all loop terminations proven")
  - List of open questions for Galois (methodology, tool versions, scope)
  - Proposed timeline for kickoff call (March 15)
  - Sign-off from proof team on completeness
- **Deliverable**: "Galois Engagement Package" (README + file inventory)
- **Risk Mitigation**: Addresses Risk 2 (Galois Methodology Issues) — early clarity prevents rework
- **Time**: 1 day

#### 1.5 Load Test Plan
- **Owner**: DevOps Lead
- **Task**: Design load test for 1,500 events/sec throughput target
- **Acceptance Criteria**:
  - Test harness defined (wrk, Apache Bench, or custom)
  - 1,500 concurrent POST /audit/events requests/sec
  - 30-second sustained run (45K total events)
  - Measure: p50, p95, p99 latencies; error rate; database CPU/memory
  - Baseline vs. post-index-optimization
  - Success threshold: p99 < 210ms, 0% error rate
- **Deliverable**: Load test script + baseline metrics
- **Risk Mitigation**: Validates benchmarks.md performance targets (Risk 5 mitigation)
- **Time**: 1 day

#### 1.6 Monthly Transparency Report #1 (Draft)
- **Owner**: Salvi (communications lead)
- **Task**: Publish short post on Capomastro blog or GitHub Discussions
- **Content**:
  - "Today (Feb 17) we published 7 security documents + backend infrastructure"
  - "Threat model: 12 threats, mean residual risk 1.8/10 (internal assessment)"
  - "Next: Galois audit (March 15), Riscure DPA-C3 (late March), Trail of Bits pentest (April)"
  - "Risk focus: Side-channel eval, formal proof validation, pentest findings"
  - Link to GitHub `/docs/security/` and OpenAPI spec
- **Deliverable**: Published blog post or GitHub discussion
- **Time**: 2 hours
- **Audience**: Security researchers, auditors, community

### Week 1 Success Criteria
- ✅ All 14 API endpoints functional (smoke test pass)
- ✅ Threat model CVSS scores validated
- ✅ Galois materials ready
- ✅ Load test baseline captured
- ✅ Monthly transparency report published

---

## Phase 2: Near-Term (Feb 24 – Mar 15, 2026) — Validation Sprint & Audit Prep

**Objective**: Complete internal validation, stress-test infrastructure, prepare for March 15 Galois kickoff.

### Week 2–3 Tasks (Feb 24 – Mar 10)

#### 2.1 Run Load Test (1,500 events/sec)
- **Owner**: DevOps Lead
- **Task**: Execute 30-second sustained load test
- **Acceptance Criteria**:
  - 45,000 events processed successfully
  - p99 latency < 210ms
  - Database CPU < 60%, memory stable
  - Index optimization shows >= 10x speedup vs. baseline
  - Results published in benchmarks.md v1.1
- **Deliverable**: Load test report (metrics + graphs)
- **Risk Mitigation**: Validates benchmarks.md targets; addresses Risk 5
- **Time**: 1 day

#### 2.2 HPTP 5-Tier Fallback Prototype
- **Owner**: Hardware/HPTP Team
- **Task**: Implement + test 5-tier chain (PTP → NTP → Crystal → Quartz → Cesium)
- **Acceptance Criteria**:
  - Simulated degradation: trigger each tier transition
  - Auto-escalation thresholds match hptp_threat_model.md
  - Transitions logged to hptp_anomaly_events
  - Recovery tested (Tier N → Tier N-1 on anomaly clear)
  - No data loss or timing glitches during transition
- **Deliverable**: "HPTP Fallback Chain Validation Report" (test results + logs)
- **Risk Mitigation**: Validates HPTP architecture defensibility (Risk 3, 4 prep)
- **Time**: 2 days

#### 2.3 Formal Verification Methodology Review (Internal)
- **Owner**: Proof Team Lead
- **Task**: Deep dive on Isabelle/HOL proofs with external consultant (optional dry-run with Galois)
- **Acceptance Criteria**:
  - All 3 completed proofs walked through (allocator, capability, GF(3))
  - Invariants documented (no double-free, no use-after-free, etc.)
  - Edge cases identified (boundary conditions, overflow handling)
  - Questions for Galois prioritized
  - Confidence level on methodology soundness estimated
- **Deliverable**: "Formal Verification Methodology Assessment" + Galois prep Q&A
- **Risk Mitigation**: Addresses Risk 2 (Galois Methodology Issues) early
- **Time**: 3 days (internal review)

#### 2.4 Threat Model Risk Score Heatmap (Visual)
- **Owner**: Security Lead
- **Task**: Create risk_heatmap.md visual representation
- **Acceptance Criteria**:
  - Pre/post-mitigation scores for all 12 threats (table + ASCII art)
  - Color coding: Red (>6.0), Yellow (3–6.0), Green (<3.0)
  - KRI definitions specified (# high-risk threats, # unmitigated vectors, etc.)
  - Residual risk trend projection (if current trajectory continues)
  - Board-ready summary (1 page)
- **Deliverable**: Updated risk_heatmap.md with visuals
- **Risk Mitigation**: Addresses Risk 8 (transparency on risk quantification)
- **Time**: 1 day

#### 2.5 Database Index Performance Tuning
- **Owner**: DevOps Lead
- **Task**: Measure actual index performance on 10K+ events
- **Acceptance Criteria**:
  - Composite index (severity, category, created_at) shows expected 15x speedup
  - Per-index cardinality estimates match actual data distribution
  - Any missing indexes identified (e.g., on threat_id in threat_model_entries)
  - Query plans reviewed; sequential scans flagged
  - database_indexes.sql updated with measured metrics
- **Deliverable**: Updated database_indexes.sql + performance analysis
- **Risk Mitigation**: Validates benchmarks.md index guidance (Risk 5 mitigation)
- **Time**: 1 day

#### 2.6 Prepare for Riscure Interim Findings (Late March)
- **Owner**: Crypto Team Lead
- **Task**: Pre-position code, design mitigation candidates
- **Acceptance Criteria**:
  - Identify high-risk crypto ops (AES GCM poly mul, ML-KEM decode, jitter source)
  - Design masking schemes (if leakage found): boolean masking, arithmetic masking, redundant ops
  - Prepare rollback plan if full rework needed
  - Document pre-remediation baseline (current code + threat model assumptions)
  - Identify 2–3 remediation candidates ready to implement
- **Deliverable**: "Side-Channel Remediation Playbook" (candidate fixes + timelines)
- **Risk Mitigation**: Addresses Risk 1 (Riscure findings) — fast response ready
- **Time**: 2 days

#### 2.7 Internal Security Audit Planning
- **Owner**: Salvi (overall security lead)
- **Task**: Prepare for Trail of Bits pentest scope definition
- **Acceptance Criteria**:
  - Ternary-specific attack vectors identified (TVM bytecode crafting, capability edge cases)
  - Known weaknesses documented (areas red team should focus on)
  - Out-of-scope items agreed (user error, unpatched dependencies)
  - SLA for vulnerability disclosure prepared (30-day critical, 90-day high)
  - Remediation prioritization framework locked (critical/high/medium/low)
- **Deliverable**: "Trail of Bits Pentest Scope Document" (ready for April)
- **Risk Mitigation**: Addresses Risk 3 (Trail of Bits discovers exploits) — scope is tight, findings fast
- **Time**: 1 day

#### 2.8 KRI Dashboard Prototype
- **Owner**: DevOps Lead
- **Task**: Build basic dashboard showing key risk indicators
- **Acceptance Criteria**:
  - KRI-001: Unresolved critical audit events (should be 0)
  - KRI-002: HPTP fallback activations (should be < 5/month)
  - KRI-003: High-risk unmitigated threats (should be 0)
  - KRI-004: Side-channel eval progress (% tests complete)
  - KRI-005: Formal verification coverage (% critical path)
  - KRI-006: Implementation completion (% proven + in-progress)
  - Auto-refresh every 30 seconds
  - Alerts on threshold breach
- **Deliverable**: "KRI Dashboard" (live query + HTML visualization)
- **Risk Mitigation**: Enables continuous risk monitoring (all risks)
- **Time**: 1 day

### Week 4 Tasks (Mar 10–15)

#### 2.9 Galois Kickoff (March 15)
- **Owner**: Salvi + Proof Team
- **Task**: Conduct 90-minute kickoff meeting with Galois
- **Agenda**:
  - Methodology walkthrough (k-induction, SMT solver strategy)
  - Proof scope and priorities (critical path first)
  - Tool versions and infrastructure setup (Isabelle 2024 or later?)
  - Timeline and milestone expectations (8 weeks → June 30)
  - Questions & answers
  - Next meeting: Design review (March 29)
- **Deliverable**: Kickoff meeting notes + agreed-upon milestone dates
- **Risk Mitigation**: Addresses Risk 2 (Galois Methodology) — early alignment prevents rework
- **Time**: 2 hours (meeting) + 4 hours prep

### Phase 2 Success Criteria
- ✅ Load test pass (1,500 events/sec, p99 < 210ms)
- ✅ HPTP fallback chain validated in simulation
- ✅ Threat model risk scores defensible
- ✅ Galois kickoff completed with methodology alignment
- ✅ Riscure mitigation playbook ready
- ✅ KRI dashboard operational

---

## Phase 3: Audit Window (Mar 15 – Jun 30, 2026) — External Validation

**Objective**: Execute concurrent audits (Galois, Riscure, Trail of Bits); integrate findings; update threat model.

### Galois Formal Verification Audit (Mar 15 – Jun 30, ~16 weeks)

#### 3.1 Galois Design Review (March 29)
- **Owner**: Proof Team + Galois
- **Task**: Present proof methodology for 2-hour design review
- **Acceptance Criteria**:
  - Galois approves approach or identifies rework needed
  - Priorities set (scheduler proof > TVM compiler > IPC)
  - Tool setup validated (Isabelle version, Z3 solver, CI/CD integration)
  - Scope confirmed (65% → 85% by June)
- **Deliverable**: Design review approval + revised timeline (if needed)
- **Risk Mitigation**: Addresses Risk 2 (Galois Methodology Issues)
- **Time**: 2 hours (meeting) + prep

#### 3.2 Monthly Proof Spot-Checks (April, May, June)
- **Owner**: Galois auditors
- **Task**: Review 2–3 completed proofs per month
- **Deliverable**: Spot-check findings (lemmas to rework, assumptions to tighten, etc.)
- **Integration**: Proof team applies fixes; monthly progress report published
- **Risk Mitigation**: Continuous validation prevents surprise findings

#### 3.3 Galois Final Report (June 30)
- **Owner**: Galois + Salvi (integration lead)
- **Task**: Publish formal verification audit report
- **Deliverable**: Public report (redacted if needed) + gap list + remediation timeline
- **Integration**: Update proofs.md with external validation results

### Riscure DPA-C3 Evaluation (Mar 1 – Jun 30)

#### 3.4 Riscure Interim Findings (Expected Late March / Early April)
- **Owner**: Riscure + Crypto Team
- **Task**: Review interim DPA-C3 power analysis results
- **Acceptance Criteria**:
  - Pearson correlation |r| on AES-256-GCM, ML-KEM, jitter source
  - TVLA t-value results (first-order leakage assessment)
  - Recommendations for constant-time improvements
- **Integration**: Crypto team applies fixes if |r| > 0.05 or t-value > 4.5
- **Risk Mitigation**: Addresses Risk 1 (Riscure findings) — early detection + quick fix
- **Deliverable**: Interim report + remediation plan (public post planned)

#### 3.5 Riscure Full DPA-C3 Report (June 30)
- **Owner**: Riscure + Crypto Team
- **Task**: Publish full side-channel evaluation report
- **Deliverable**: Public report + certified constant-time operations list
- **Integration**: Update threat_model.md with empirical side-channel residual risk scores

### Trail of Bits Penetration Testing (April 15 – July 31)

#### 3.6 Trail of Bits Scope Alignment (April 1–15)
- **Owner**: Salvi + Trail of Bits
- **Task**: Refine pentest scope based on Ternary-specific vectors
- **Acceptance Criteria**:
  - In-scope: TVM bytecode exploitation, capability system edge cases, supply-chain vectors
  - Out-of-scope: User error, unpatched dependencies, DoS (resource exhaustion)
  - Agreed methodology (fuzzing, manual code review, live testing)
  - Timeline: 12 weeks starting April 15
- **Deliverable**: Pentest engagement scope document (signed)

#### 3.7 Weekly Pentest Status Updates (April 15 – July 31)
- **Owner**: Trail of Bits (weekly updates to Salvi)
- **Task**: Publish redacted pentest progress (threats tested, findings count)
- **Integration**: High-severity findings trigger immediate patch cycles

#### 3.8 Trail of Bits Final Report (August 1)
- **Owner**: Trail of Bits + Salvi
- **Task**: Publish redacted pentest report
- **Deliverable**: Public report (critical findings redacted for 90-day embargo) + remediation status
- **Integration**: Update threat_model.md with pentest-discovered vectors

### Monthly Transparency Reports (March, April, May, June)

#### 3.9 Mar 31 Update
- Galois design review results
- Riscure interim findings (if available)
- HPTP fallback chain validation results
- KRI snapshot (unresolved events, fallback activations, unmitigated threats)

#### 3.10 Apr 30 Update
- Trail of Bits pentest kickoff
- Galois 6-week spot-check results
- Side-channel remediation progress (if fixes needed)
- Implementation completion update (% proven, in-progress, planned)

#### 3.11 May 31 Update
- Galois 10-week progress
- Riscure full DPA-C3 report expectations
- Trail of Bits findings summary (redacted)
- FIPS 140-3 pre-assessment timeline confirmation

#### 3.12 Jun 30 Update
- Galois final report published
- Riscure full report published
- Trail of Bits pentest 75% complete (7.5 weeks remaining)
- Risk heatmap updated with empirical audit data

### Phase 3 Success Criteria
- ✅ Galois audit completes with spot-checks approved
- ✅ Riscure interim & final DPA-C3 reports delivered
- ✅ Trail of Bits pentest starts (scope locked)
- ✅ Threat model updated with audit findings
- ✅ Monthly transparency reports published (4 total)
- ✅ Zero critical security issues in production (KRI-001 = 0)

---

## Phase 4: Integration & Remediation (Jul 1 – Sep 30, 2026) — Findings Closure

**Objective**: Integrate audit findings, publish remediation results, prepare for silicon tape-out.

### Tasks (Jul–Sep)

#### 4.1 Trail of Bits Final Report Integration (Aug 1)
- **Owner**: Salvi (security response lead)
- **Task**: Receive final pentest report; triage findings
- **Acceptance Criteria**:
  - Categorize by severity (critical/high/medium/low)
  - Assign remediation owners + timelines
  - Track fixes via security_audit_log + implementation_status
  - Publish redacted report (90-day embargo lifted)
- **Deliverable**: "Pentest Findings & Remediation Status" report

#### 4.2 Riscure Remediation Verification (Jul–Aug)
- **Owner**: Crypto Team
- **Task**: Re-run DPA-C3 tests on remediated code
- **Acceptance Criteria**:
  - Post-remediation |r| < 0.05 or t-value < 4.5 (no first-order leakage)
  - Constant-time properties verified
  - Published update to threat_model.md
- **Deliverable**: "Side-Channel Remediation Validation" report

#### 4.3 Galois Spot-Check Follow-Ups (Jul–Aug)
- **Owner**: Proof Team + Galois
- **Task**: Resolve any lingering proof assumptions or edge cases
- **Deliverable**: Final approval (or deferred to Phase 5)

#### 4.4 Formal Threat Model v1.1 Publication (Aug 31)
- **Owner**: Salvi + Security Team
- **Task**: Publish threat model v1.1 incorporating all audit feedback
- **Content**:
  - Updated CVSS scores (post-remediation)
  - Residual risk revisions (based on external assessment)
  - New threat vectors from Trail of Bits
  - Control verification status (Galois spot-checks, Riscure eval, etc.)
- **Deliverable**: threat_model.md v1.1 (public GitHub release)

#### 4.5 FIPS 140-3 Pre-Assessment (Jul–Aug)
- **Owner**: Compliance Lead
- **Task**: Conduct FIPS pre-assessment with NIST lab
- **Acceptance Criteria**:
  - Identify gaps (expected: RNG entropy, algorithm compliance, firmware loading)
  - Propose remediation timeline
  - Gap severity: Low (Q4 fix), Medium (Q1 2027 fix), High (immediate)
- **Deliverable**: FIPS pre-assessment report + gap remediation plan

#### 4.6 Hardware Attestation Framework (Aug)
- **Owner**: Hardware Team
- **Task**: Complete hardware attestation design (boot-time measurement)
- **Acceptance Criteria**:
  - Firmware hash database seeded (motherboard, NICs, storage)
  - ML-DSA signature verification on boot
  - Mismatch quarantine procedure documented
- **Deliverable**: Hardware attestation design doc + implementation readiness

#### 4.7 Sept 30 Status: Ready for Silicon (Optional)
- **Owner**: Salvi (overall)
- **Task**: Publish "Audit Completion & Remediation Summary"
- **Content**:
  - All external audits complete (Galois, Riscure, Trail of Bits)
  - High-severity findings remediated
  - Medium-severity findings in-progress
  - Threat model updated with empirical data
  - Ready for H2 2026 silicon tape-out
- **Deliverable**: Public post + GitHub release notes

### Phase 4 Success Criteria
- ✅ All external audits complete with findings integrated
- ✅ Threat model v1.1 published (empirically validated)
- ✅ Side-channel remediation verified
- ✅ FIPS pre-assessment gap plan in-hand
- ✅ Hardware attestation framework complete
- ✅ Ready for H2 2026 silicon tape-out

---

## Phase 5: Long-Term (Oct 2026 – Q4 2027) — Production Readiness

**Objective**: Achieve formal certifications, tape-out silicon, plan production deployment.

### Tasks (Oct 2026 – Q4 2027)

#### 5.1 FIPS 140-3 Submission (Oct 2026)
- **Owner**: Compliance Lead
- **Task**: Resolve pre-assessment gaps; submit for formal evaluation
- **Timeline**: Submission Q4 2026, certification Q2–Q4 2027

#### 5.2 Common Criteria EAL 2 Evaluation (Nov 2026 – Q4 2027)
- **Owner**: Compliance Lead + external CC lab
- **Task**: Develop security target; conduct formal evaluation
- **Timeline**: Security target draft Q1 2027, evaluation Q2–Q4 2027

#### 5.3 Silicon Tape-Out (Oct–Dec 2026)
- **Owner**: Hardware Team
- **Task**: Fabrication of secure element + RISC-V xplenum extension
- **Deliverable**: First silicon (Q4 2026 or Q1 2027)

#### 5.4 Formal Verification Coverage (Q4 2026 target: 95%)
- **Owner**: Proof Team + Galois (post-audit support)
- **Task**: Complete remaining 8 planned proofs
- **Target**: IPC safety, TVM isolation, boot chain, etc.

#### 5.5 Production Deployment Readiness (Q1 2027)
- **Owner**: DevOps Lead
- **Task**: Hardening for production: load balancing, failover, monitoring
- **Deliverable**: Production deployment guide + runbook

---

## Risk Escalation Workflows (All Phases)

### Escalation Triggers & Response Times

| Risk | Trigger | Response Owner | Escalation Path | Action Deadline |
|------|---------|----------------|-----------------|-----------------|
| **Riscure finds |r| > 0.05** | Interim report (late Mar) | Crypto Lead | Salvi (day 1) | Design fix (1 week) |
| **Galois requests > 20% proof rework** | Design review (Mar 29) | Proof Team Lead | Salvi (day 1) | Timeline revision (3 days) |
| **Trail of Bits finds kernel code execution** | Weekly pentest update | Red Team Coordinator | Salvi (same day) | Patch plan (24 hours) |
| **Hardware partner delays tape-out > 3mo** | Monthly vendor sync | Hardware Lead | Salvi (day 1) | Fallback plan (1 week) |
| **Galois audit extends past June 30** | Spot-check delay (May/June) | Galois PM | Salvi (day 1) | New timeline + escalation clause (3 days) |
| **FIPS gap assessment > 15 issues** | Pre-assessment (Jul–Aug) | Compliance Lead | Salvi (day 1) | Q1 2027 remediation plan (1 week) |
| **KRI-001: Unresolved critical events > 3** | Daily dashboard check | Salvi | Board (weekly) | Incident response (24 hours per event) |

---

## Success Metrics (Cumulative by Phase)

### By End of Phase 1 (Feb 24)
- ✅ Infrastructure validated (all 14 endpoints operational)
- ✅ Threat model CVSS scores reviewed
- ✅ Galois materials ready
- ✅ Load test baseline captured

### By End of Phase 2 (Mar 15)
- ✅ Load test pass (1,500 events/sec)
- ✅ HPTP fallback chain validated
- ✅ Formal verification methodology aligned with Galois
- ✅ KRI dashboard live
- ✅ Galois kickoff completed

### By End of Phase 3 (Jun 30)
- ✅ Galois audit complete + spot-checks approved
- ✅ Riscure interim & final reports delivered
- ✅ Trail of Bits pentest 7.5 weeks complete
- ✅ Threat model updated with empirical data
- ✅ 4 monthly transparency reports published

### By End of Phase 4 (Sep 30)
- ✅ All external audits complete + findings integrated
- ✅ Threat model v1.1 published
- ✅ Side-channel remediation verified
- ✅ FIPS pre-assessment complete
- ✅ Hardware attestation framework complete
- ✅ Ready for silicon tape-out

### By End of Phase 5 (Q4 2027)
- ✅ FIPS 140-3 certification (or clear path)
- ✅ Common Criteria EAL 2 evaluation (or clear path)
- ✅ Silicon deployed
- ✅ Production deployment readiness
- ✅ 95% formal verification coverage

---

## Owners & Accountability

| Role | Responsible For | Reports To | Check-In Cadence |
|------|-----------------|-----------|------------------|
| **Salvi** (Overall Lead) | Project timeline, escalations, board updates | Board / Executive Leadership | Weekly risk sync, monthly board |
| **DevOps Lead** | Infrastructure, load testing, KRI dashboard | Salvi | Bi-weekly status |
| **Crypto Team Lead** | Side-channel remediation, Riscure engagement | Salvi | Weekly during Riscure eval (Mar–Jun) |
| **Proof Team Lead** | Formal verification, Galois coordination | Salvi | Bi-weekly with Galois |
| **Security Lead** | Threat model, pentest coordination | Salvi | Weekly during Trail of Bits (Apr–Jul) |
| **Hardware Lead** | Silicon tape-out, vendor management | Salvi | Monthly |
| **Compliance Lead** | FIPS/CC certifications | Salvi | Monthly |

---

## Document References

- `/docs/security/threat_model.md` — Threat registry (updated v1.0 → v1.1 post-audit)
- `/docs/security/proofs.md` — Formal verification roadmap (Galois spot-checks monthly)
- `/docs/security/hptp_threat_model.md` — HPTP architecture (fallback chain prototype Phase 2)
- `/docs/security/side_channel_framework.md` — DPA-C3 protocol (Riscure evaluation Mar–Jun)
- `/docs/security/benchmarks.md` — Performance targets (validated Phase 2)
- `/docs/security/risk_heatmap.md` — Risk visualization + KRIs (live tracking Phases 2–5)
- `/docs/security/database_indexes.sql` — Index definitions (tuned Phase 2)
- Risk Assessment (this document) — Escalation workflows + success metrics

---

**Last Updated**: February 17, 2026  
**Next Milestone**: Phase 1 Complete (Feb 24, 2026)  
**Final Milestone**: Phase 5 Complete (Q4 2027)

Così sia, Fratello. 🔐