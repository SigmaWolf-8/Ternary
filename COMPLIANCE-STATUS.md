# Compliance Status Report
## PlenumNET — Self-Audit Remediation Tracker
### Capomastro Holdings Ltd., Applied Physics Division

**Last Updated:** February 15, 2026
**Audit Date:** February 15, 2026
**Classification:** INTERNAL — Governance

---

## 1. Executive Summary

This document tracks the remediation status of all findings from the February 2026 self-audit of the SigmaWolf-8/Ternary (GitHub) and PlenumNET (Replit) projects. The audit identified medium-risk exposure across international law, export controls, licensing, supply chain, data privacy, and security domains. All high-priority items have been addressed or are documented with mitigation plans.

---

## 2. Remediation Status by Domain

### 2.1 Licensing, Dependency, and Attribution

| Finding | Status | Document/File | Notes |
|---------|--------|---------------|-------|
| Formal license inventory | COMPLETE | `LICENSE` | Proprietary license, all rights reserved |
| Third-party attribution notices | COMPLETE | `NOTICE` | 28 third-party components documented with licenses |
| Copyleft dependency review | COMPLETE | `NOTICE` (entry 24) | MPL-2.0 (LightningCSS) — build-tool only, no copyleft triggered |
| GPL/AGPL dependency check | COMPLETE | CI: `security-scan.yml` | License compliance check denies AGPL/GPL; zero found |
| Transitive license analysis | COMPLETE | License summary | Majority MIT/ISC/Apache-2.0/BSD-3; 3 MPL-2.0 (build tools); no GPL/AGPL copyleft |
| IP attribution documentation | COMPLETE | `IP-NOTICE.md` | Patent claims, trade secrets, trademarks, algorithms documented |
| Trademark documentation | COMPLETE | `TRADEMARK-NOTICE.md` | Usage guidelines for PlenumNET, Salvi Framework marks |
| Contributor agreement | COMPLETE | `CONTRIBUTING.md` | UPIID Framework standards, code review requirements |

### 2.2 Software Bill of Materials (SBOM)

| Finding | Status | Document/File | Notes |
|---------|--------|---------------|-------|
| SBOM generation | COMPLETE | `sbom.json` | CycloneDX format; component count varies with dependencies |
| SBOM CI automation | COMPLETE | `.github/workflows/sbom-generate.yml` | Generates canonical sbom.json; weekly + on dependency changes; 90-day artifact retention |
| Ongoing vulnerability scanning | COMPLETE | `.github/workflows/security-scan.yml` | cargo audit (Rust), npm audit (Node.js), CodeQL (JS/TS) |

### 2.3 Security Posture

| Finding | Status | Document/File | Notes |
|---------|--------|---------------|-------|
| Vulnerability disclosure policy | COMPLETE | `SECURITY.md` | GitHub Security Advisories, 48h ack, 14-day critical patch |
| Secure SDLC practices | COMPLETE | CI pipeline | CodeQL, Clippy, Gitleaks, cargo-audit, npm-audit |
| Incident response plan | COMPLETE | `INCIDENT-RESPONSE.md` | P1-P4 severity, PIPEDA/GDPR breach notification |
| Security hardening checklist | COMPLETE | `SECURITY.md` | Rate limiting, CSP, CORS, TLS, encryption, non-root containers |
| Secret detection | COMPLETE | CI: `security-scan.yml` | Gitleaks on every push, zero-history scan |
| Static analysis | COMPLETE | CI: `security-scan.yml` | CodeQL (JS/TS), Clippy (Rust) |
| npm dependency audit | COMPLETE | CI: `security-scan.yml` | Added npm audit + license compliance check |
| Copyright header enforcement | COMPLETE | CI: `license-check.yml` | Validates copyright headers across source files |

### 2.4 Known Vulnerabilities

| Package | Severity | Status | Mitigation |
|---------|----------|--------|------------|
| lodash | Moderate | MONITORING | Prototype pollution in `_.unset`/`_.omit`; patched upstream, updated in lockfile |
| qs | Low | MONITORING | arrayLimit bypass; patched upstream, updated in lockfile |
| xlsx (SheetJS) | High | ACCEPTED RISK | No fix available upstream. Mitigated: server-side only, input validation enforced, usage restricted to PlenumDB import feature with trusted files. Documented in NOTICE. |

### 2.5 Data Privacy and Cross-Border Flows

| Finding | Status | Document/File | Notes |
|---------|--------|---------------|-------|
| Privacy policy (PIPEDA/GDPR/CCPA) | COMPLETE | `PRIVACY-POLICY.md` | PIPEDA 10 principles, GDPR Art. 13/14, CCPA/CPRA rights |
| Data flow mapping | COMPLETE | `DATA-FLOW-MAP.md` | 7 processing activities mapped with legal basis, storage, retention |
| Cross-border transfer safeguards | COMPLETE | `DATA-FLOW-MAP.md` §5 | Canada→US transfers documented; SCC recommendations |
| Third-party processor inventory | COMPLETE | `DATA-FLOW-MAP.md` §6, `DPA-GUIDANCE.md` §3 | 5 processors identified with DPA status |
| DPA guidance and checklist | COMPLETE | `DPA-GUIDANCE.md` | PIPEDA, GDPR Art. 28, Alberta PIPA requirements |
| Data subject rights procedures | COMPLETE | `PRIVACY-POLICY.md` §10, `DATA-FLOW-MAP.md` §7 | Access, correction, erasure, portability, withdrawal |
| Data subject rights API | COMPLETE | `server/routes/data-subject-rights.ts` | Data export, account deletion, request history endpoints |
| Cookie consent banner | COMPLETE | `client/src/components/cookie-consent.tsx` | localStorage persistence, essential/functional/analytics categories |
| Cookie policy | COMPLETE | `docs/legal/COOKIE-POLICY.md` | Cookie types, purposes, retention, opt-out procedures |
| Children's privacy | COMPLETE | `PRIVACY-POLICY.md` §11 | 18+ age restriction |
| CASL compliance | COMPLETE | `PRIVACY-POLICY.md` §12 | Consent-based commercial messages |

### 2.6 Platform/Hosting Terms

| Finding | Status | Document/File | Notes |
|---------|--------|---------------|-------|
| Terms of Service | COMPLETE | `TERMS-OF-SERVICE.md` | Governing law: Alberta, Canada; dispute resolution; export compliance |
| Acceptable Use Policy | COMPLETE | `ACCEPTABLE-USE-POLICY.md` | Referenced from Terms of Service |
| GitHub ToS compliance | COMPLETE | `LICENSE` §6 | Viewing and Reference clause; public visibility ≠ license grant |
| Replit ToS compliance | MONITORING | — | Platform-dependent; reviewed periodically |

### 2.7 Cryptography and Export Controls

| Finding | Status | Document/File | Notes |
|---------|--------|---------------|-------|
| Export classification | COMPLETE | `EXPORT-CONTROL.md` | Canadian ECL, Wassenaar Cat. 5.2, US ECCN 5D002 |
| Cryptographic algorithm inventory | COMPLETE | `docs/compliance/export-control/crypto-inventory.md` | TL-KEM, TL-DSA, Phase Encryption, AES-256-GCM, SHA-2/SHA-3, HMAC, KDF |
| CNSA 2.0 compliance claims | COMPLETE | `EXPORT-CONTROL.md` §3.2 | Clear distinction: standard vs. proprietary implementations |
| FIPS 140-3 disclaimer | COMPLETE | `TERMS-OF-SERVICE.md` §9.3 | Design targets ≠ formal certification |
| Restricted destinations | COMPLETE | `EXPORT-CONTROL.md` §5 | SEMA, UN sanctions, Area Control List |
| SaaS deployment considerations | COMPLETE | `EXPORT-CONTROL.md` §7 | SaaS ≠ export under current guidance |
| Compliance procedures | COMPLETE | `EXPORT-CONTROL.md` §6 | Pre-export screening, record keeping, technology transfer |

### 2.8 Crypto/Financial Implications

| Finding | Status | Document/File | Notes |
|---------|--------|---------------|-------|
| Howey Test analysis | COMPLETE | `docs/compliance/financial/HOWEY-TEST-ANALYSIS.md` | No securities offered; utility-based SaaS model |
| AML/CFT considerations | CONDITIONAL | — | Applies only if payment/token/wallet features activated |
| KYC/CDD requirements | CONDITIONAL | — | Applies only if onboarding financial service users |
| Securities regulation | COMPLETE | `docs/compliance/financial/HOWEY-TEST-ANALYSIS.md` | Four-prong analysis: not an investment contract |
| FINTRAC MSB assessment | MONITORING | — | Blockchain witnessing may trigger MSB registration |
| Current exposure | LOW | — | No live payment processing or token issuance at present |

---

## 3. Compliance Document Inventory

| Document | Location | Last Updated | Purpose |
|----------|----------|-------------|---------|
| `LICENSE` | Repository root | 2026-02-11 | Proprietary software license |
| `NOTICE` | Repository root | 2026-02-15 | Third-party attribution (28 components) |
| `SECURITY.md` | Repository root + `.github/` | 2026-02-15 | Vulnerability disclosure, supply chain, hardening |
| `PRIVACY-POLICY.md` | Repository root | 2026-02-11 | Privacy policy (PIPEDA, GDPR, CCPA) |
| `TERMS-OF-SERVICE.md` | Repository root | 2026-02-11 | Terms of service |
| `ACCEPTABLE-USE-POLICY.md` | Repository root | 2026-02-11 | Acceptable use policy |
| `EXPORT-CONTROL.md` | Repository root | 2026-02-14 | Export control classification |
| `IP-NOTICE.md` | Repository root | 2026-02-14 | Intellectual property claims |
| `TRADEMARK-NOTICE.md` | Repository root | 2026-02-14 | Trademark usage guidelines |
| `CONTRIBUTING.md` | Repository root | 2026-02-14 | Contributor standards and workflow |
| `DATA-FLOW-MAP.md` | Repository root | 2026-02-15 | Personal data processing inventory |
| `INCIDENT-RESPONSE.md` | Repository root | 2026-02-15 | Security incident procedures |
| `DPA-GUIDANCE.md` | Repository root | 2026-02-15 | Data Processing Agreement guidance |
| `COMPLIANCE-STATUS.md` | Repository root | 2026-02-15 | This document |
| `sbom.json` | Repository root | 2026-02-15 | CycloneDX SBOM (regenerated by CI weekly) |
| Crypto Inventory | `docs/compliance/export-control/` | 2026-02-15 | Algorithm inventory with code locations |
| Cookie Policy | `docs/legal/` | 2026-02-15 | Cookie types, purposes, retention |
| Howey Test Analysis | `docs/compliance/financial/` | 2026-02-15 | Securities classification (not a security) |
| Monitoring Procedures | `docs/compliance/monitoring/` | 2026-02-15 | Continuous compliance monitoring |
| Implementation SOW | `docs/compliance/audit-framework/` | 2026-02-15 | 5-phase compliance implementation plan |

---

## 4. CI/CD Security Pipeline

| Workflow | Trigger | Coverage |
|----------|---------|----------|
| `security-scan.yml` | Push to main, weekly | cargo-audit, CodeQL, Gitleaks, Clippy, npm audit, license check |
| `sbom-generate.yml` | Dependency changes, weekly | CycloneDX SBOM generation, artifact retention |
| `license-check.yml` | Push to main | Copyright header enforcement across all source files |
| `compliance-check.yml` | Crypto changes, monthly | CNSA 2.0 algorithm coverage, NIST standards verification |
| `codeql-analysis.yml` | Push to main, PR | GitHub Advanced Security static analysis |
| `fuzz.yml` | Push/PR, schedule | 3 fuzz targets: trit ops, tryte ops, gateway |
| `quarterly-compliance.yml` | Quarterly (Jan/Apr/Jul/Oct) | Automated compliance report generation |

---

## 5. Open Action Items

| Priority | Action | Owner | Target Date |
|----------|--------|-------|-------------|
| High | Execute DPA with Replit for hosting/database | Privacy Officer | Q1 2026 |
| High | Confirm OpenAI DPA coverage for Agent Array | Privacy Officer | Q1 2026 |
| Medium | Evaluate xlsx replacement (SheetJS vulnerability) | Engineering | Q2 2026 |
| Medium | Conduct tabletop incident response exercise | Security Lead | Q2 2026 |
| Low | Regional privacy assessment (Québec Law 25) | Legal Counsel | Q3 2026 |
| Low | Formal export control classification filing | Legal Counsel | As needed |

---

## 6. Review Schedule

| Review Type | Frequency | Next Due |
|-------------|-----------|----------|
| Compliance status update | Monthly | March 15, 2026 |
| SBOM regeneration | Weekly (automated) | Continuous |
| Dependency vulnerability scan | Weekly (automated) | Continuous |
| Privacy impact assessment | Annually | February 2027 |
| Incident response plan review | Semi-annually | August 2026 |
| DPA compliance review | Semi-annually | August 2026 |
| Export control review | Upon regulatory change | Monitoring |

---

*Capomastro Holdings Ltd. — Applied Physics Division*
*This document is maintained for internal governance purposes and does not constitute legal advice.*
