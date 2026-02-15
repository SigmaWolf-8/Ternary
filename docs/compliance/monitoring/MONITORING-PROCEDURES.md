# Compliance Monitoring Procedures

**Document ID:** COMP-MON-001
**Version:** 1.0
**Date:** February 15, 2026
**Author:** Capomastro Holdings Ltd. Compliance Division
**Classification:** Internal — Operations

---

## 1. Purpose

This document establishes continuous monitoring procedures for PlenumNET's
compliance obligations across privacy, export control, security, and financial
regulatory domains. It defines schedules, responsibilities, escalation paths,
and tooling for ongoing compliance assurance.

---

## 2. Monitoring Domains

### 2.1 Privacy Compliance (GDPR/PIPEDA/CCPA)

| Control | Frequency | Method | Owner |
|---|---|---|---|
| Data Subject Request processing | Daily | Automated queue check | Privacy Officer |
| Consent banner functionality | Weekly | Automated UI test | Engineering |
| Data retention policy adherence | Monthly | Database audit query | DBA/Privacy |
| Privacy policy accuracy | Quarterly | Manual review | Legal |
| Cross-border data transfer review | Quarterly | Transfer mechanism audit | Privacy Officer |
| Data breach response drill | Semi-annually | Tabletop exercise | CISO/Privacy |

**Automated Checks:**
- `GET /api/data-subject-requests` — verify pending requests are processed within 30-day SLA
- Cookie consent localStorage state verification via E2E tests
- Data retention SQL audit: flag records exceeding retention period

### 2.2 Export Control Compliance

| Control | Frequency | Method | Owner |
|---|---|---|---|
| Cryptographic algorithm inventory accuracy | Quarterly | Code scan vs. inventory | Engineering |
| ECCN classification review | Semi-annually | Manual review against EAR/CCL | Export Counsel |
| Denied party screening | Per-transaction | Automated API check | Sales/Compliance |
| Technology transfer documentation | Per-event | Manual log | Engineering |
| License exception eligibility review | Annually | Legal assessment | Export Counsel |

**Automated Checks:**
- `grep -r` scan for new crypto implementations not in `crypto-inventory.md`
- CI workflow (`compliance-check.yml`) validates license headers on all source files
- SBOM generation (`sbom-generate.yml`) tracks dependency changes weekly

### 2.3 Security Controls

| Control | Frequency | Method | Owner |
|---|---|---|---|
| CodeQL static analysis | Per-commit + weekly | GitHub Actions | Engineering |
| Dependency vulnerability scan | Daily | npm audit + Dependabot | Engineering |
| Rate limiter effectiveness | Weekly | Load test sampling | SRE |
| TLS certificate validity | Daily | Automated cert check | Infrastructure |
| Access control review | Monthly | User/role audit | CISO |
| Penetration testing | Annually | Third-party assessment | CISO |
| Security header compliance | Weekly | Automated header scan | Engineering |

**Automated Checks:**
- CodeQL workflow runs on every push to main and weekly schedule
- `npm audit` integrated into CI pipeline
- Security header validation via `curl` checks in CI

### 2.4 Financial Compliance

| Control | Frequency | Method | Owner |
|---|---|---|---|
| Howey Test applicability review | Per-product launch | Legal analysis | Securities Counsel |
| Payment processor compliance | Monthly | Stripe dashboard audit | Finance |
| AML/KYC obligations review | Quarterly | Risk assessment | Compliance |
| FINTRAC MSB registration status | Quarterly | Regulatory check | Compliance |
| Tax compliance (GST/HST) | Monthly | Accounting reconciliation | Finance |

---

## 3. Escalation Procedures

### 3.1 Severity Levels

| Level | Description | Response Time | Escalation Path |
|---|---|---|---|
| Critical | Active data breach, regulatory enforcement action | 1 hour | CISO → CEO → External Counsel |
| High | Compliance control failure, vulnerability exploit | 4 hours | Team Lead → CISO → Legal |
| Medium | Process deviation, missed SLA | 24 hours | Team Lead → Compliance Officer |
| Low | Documentation gap, minor process improvement | 5 business days | Compliance Officer |

### 3.2 Incident Response Integration

Privacy and security incidents follow the Incident Response Plan:
1. **Detection** — automated monitoring or manual report
2. **Triage** — severity classification within 1 hour
3. **Containment** — technical remediation initiated
4. **Notification** — regulatory notifications per jurisdiction requirements
   - GDPR: 72-hour notification to supervisory authority
   - PIPEDA: "as soon as feasible" to OPC
   - CCPA: notification per Cal. Civ. Code §1798.82
5. **Remediation** — root cause analysis and fix deployment
6. **Post-incident** — lessons learned and control updates

---

## 4. Reporting

### 4.1 Internal Reports

| Report | Frequency | Audience | Format |
|---|---|---|---|
| Compliance Dashboard | Real-time | All staff | Web dashboard |
| Monthly Compliance Summary | Monthly | Management | PDF/Markdown |
| Quarterly Compliance Report | Quarterly | Board/Legal | PDF with appendices |
| Annual Compliance Assessment | Annually | Board/Auditors | Formal report |

### 4.2 Quarterly Compliance Report Contents

1. Executive summary of compliance posture
2. Privacy metrics (DSR count, response times, breach status)
3. Export control status (shipment screening, classification updates)
4. Security metrics (vulnerabilities found/fixed, incident count)
5. Financial compliance status (Howey analysis currency, regulatory filings)
6. Upcoming regulatory changes and impact assessment
7. Action items and remediation tracking

### 4.3 Automated Report Generation

The quarterly compliance report is generated via:
```bash
node scripts/quarterly-compliance-report.js
```

This script aggregates:
- Data subject request statistics from PostgreSQL
- CI/CD security scan results from GitHub Actions
- Dependency vulnerability counts from npm audit
- License header compliance from CI checks

Output is saved to `docs/compliance/reports/quarterly-YYYY-QN.md`.

---

## 5. Tools and Infrastructure

| Tool | Purpose | Integration |
|---|---|---|
| GitHub Actions | CI/CD security scanning | CodeQL, license-check, SBOM |
| Drizzle ORM | Database audit queries | PostgreSQL |
| Express Rate Limiter | API abuse prevention | 4-tier rate limiting |
| Helmet.js | Security header enforcement | Express middleware |
| npm audit | Dependency vulnerability detection | CI pipeline |
| CycloneDX cdxgen | SBOM generation | Weekly CI workflow |

---

## 6. Review and Update

This document is reviewed:
- **Quarterly** for procedure effectiveness
- **Upon regulatory change** affecting any monitored domain
- **After any compliance incident** to incorporate lessons learned
- **Annually** for comprehensive update

---

*Maintained by Capomastro Holdings Ltd. Compliance Division.*
