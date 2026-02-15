# Data Processing Agreement Guidance
## PlenumNET — Third-Party Processor Compliance
### Capomastro Holdings Ltd., Applied Physics Division

**Last Updated:** February 15, 2026
**Classification:** INTERNAL — Compliance Reference
**Document Version:** 1.0

---

## 1. Purpose

This document provides guidance for establishing and maintaining Data Processing Agreements ("DPAs") with third-party service providers that process personal data on behalf of PlenumNET and Capomastro Holdings Ltd. It addresses requirements under PIPEDA (accountability principle, Schedule 1), GDPR Article 28, and provincial privacy legislation including Alberta's PIPA (s. 5(3)) and Québec's Law 25.

---

## 2. When a DPA Is Required

A DPA is required when a third party:
- Stores personal data on our behalf (database hosting, cloud storage)
- Processes personal data as part of service delivery (AI inference, email delivery)
- Has access to systems containing personal data (infrastructure providers)
- Receives personal data transfers outside Canada

A DPA is **not** required when:
- The third party only processes anonymized or aggregated data
- The relationship is controller-to-controller (each party determines purposes)
- The service involves no personal data (e.g., CDN serving static assets)

---

## 3. Current Third-Party Processor Inventory

### 3.1 Processors Requiring DPA Review

| Processor | Service | Data Processed | DPA Status | Priority |
|-----------|---------|---------------|------------|----------|
| **Replit, Inc.** | Application hosting, PostgreSQL database | All platform user data, session data, reports | **Review Required** | High |
| **Neon, Inc.** (via Replit) | PostgreSQL database infrastructure | User records, agent reports, contact submissions | Via Replit DPA | High |
| **OpenAI, Inc.** | LLM inference for Agent Array | User queries, generated responses | **Review Required** | High |
| **GitHub, Inc.** | Code repository hosting | Source code, CI/CD logs (no user PII in normal operation) | Standard GitHub DPA | Medium |
| **Kong, Inc.** | API gateway management (Konnect) | API configuration metadata (no user PII) | Standard terms | Low |

### 3.2 Future Processors (Conditional)

If the platform integrates payment processing or blockchain witnessing:

| Processor | Service | DPA Status |
|-----------|---------|------------|
| Stripe, Inc. | Payment processing | Required before activation |
| Hedera Hashgraph | Blockchain consensus | Required before activation |
| XRP Ledger Foundation | Blockchain transactions | Required before activation |
| Algorand Foundation | Smart contract execution | Required before activation |

---

## 4. Required DPA Provisions

### 4.1 PIPEDA Requirements (Schedule 1, Principle 1.4.3)

Under PIPEDA's accountability principle, contractual safeguards with processors must include:
- Processing limited to identified purposes and documented instructions
- Appropriate security measures comparable to the controller's obligations
- Restrictions on sub-processing without prior consent
- Cooperation with data subject access and correction requests
- Notification of security breaches affecting personal information
- Return or destruction of personal data upon termination

### 4.2 GDPR Article 28 Requirements

For processing involving EU/EEA data subjects, DPAs must additionally include:
- Subject matter and duration of processing
- Nature and purpose of processing
- Categories of data subjects and personal data
- Obligations and rights of the controller
- Sub-processor approval mechanism (general or specific authorization)
- Assistance with DPIA and prior consultation obligations
- Audit rights for the controller
- Standard Contractual Clauses for international transfers (where applicable)

### 4.3 Alberta PIPA Requirements (s. 5(3))

Under Alberta's Personal Information Protection Act:
- Processor must protect personal information in a manner consistent with PIPA
- Written agreement specifying purposes for which information may be used
- Restrictions on disclosure to sub-contractors

---

## 5. Standard Contractual Clauses (SCCs)

### 5.1 When SCCs Are Required

SCCs are required for personal data transfers from:
- Canada to jurisdictions without comparable privacy protections (assessed per PIPEDA)
- EU/EEA to third countries without an adequacy decision (GDPR Chapter V)

### 5.2 Applicable SCC Modules

| Transfer Scenario | SCC Module |
|-------------------|------------|
| PlenumNET (Canada) → Replit (US) | Module 2: Controller → Processor |
| PlenumNET (Canada) → OpenAI (US) | Module 2: Controller → Processor |
| EU User → PlenumNET (Canada) | Module 1: Controller → Controller (Canada has EU adequacy) |
| EU User → PlenumNET → Replit (US) | Module 3: Processor → Sub-processor |

### 5.3 Supplementary Measures

In addition to SCCs, consider:
- Encryption of personal data in transit and at rest
- Pseudonymization where feasible
- Access controls limiting processor personnel access
- Monitoring and audit of processor compliance

---

## 6. DPA Execution Checklist

For each processor requiring a DPA:

- [ ] Identify categories of personal data processed
- [ ] Determine applicable privacy regimes (PIPEDA, GDPR, CCPA)
- [ ] Review processor's standard DPA/DPA addendum
- [ ] Verify DPA includes all required provisions (Section 4)
- [ ] Assess whether SCCs are needed (Section 5)
- [ ] Confirm sub-processor notification mechanism
- [ ] Document DPA execution date and review schedule
- [ ] Add processor to DATA-FLOW-MAP.md
- [ ] Schedule annual DPA compliance review

---

## 7. Review and Maintenance

| Activity | Frequency |
|----------|-----------|
| DPA inventory review | Semi-annually |
| Processor compliance assessment | Annually |
| SCC adequacy review | Upon regulatory changes |
| New processor DPA execution | Before data processing begins |
| Sub-processor change review | Upon notification from processor |

---

## 8. Contact

For DPA-related inquiries:

**Privacy Officer**
Capomastro Holdings Ltd.
Applied Physics Division
Sherwood Park, AB Canada
Email: Rsalvi@Salvigroup.com

---

*This document provides internal compliance guidance and does not constitute legal advice. Consult retained counsel for specific DPA drafting and negotiation.*
