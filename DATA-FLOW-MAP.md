# Data Flow Map
## PlenumNET — Personal Data Processing Inventory
### Capomastro Holdings Ltd., Applied Physics Division

**Last Updated:** February 15, 2026
**Classification:** INTERNAL — Compliance Reference
**Regulatory Basis:** PIPEDA Schedule 1, GDPR Art. 30, CCPA §1798.100

---

## 1. Purpose

This document maps the collection, processing, storage, and transfer of personal data across the PlenumNET platform and associated services. It fulfills the record-of-processing-activities requirement under GDPR Article 30 and supports compliance with PIPEDA's accountability principle (Schedule 1, Principle 1).

---

## 2. Data Controller

**Capomastro Holdings Ltd.**
Applied Physics Division
Province of Alberta, Canada
Privacy Officer: Rsalvi@Salvigroup.com

---

## 3. Data Processing Activities

### 3.1 User Authentication (Replit Auth)

| Attribute | Detail |
|-----------|--------|
| **Data Categories** | Name, email address, profile picture URL, OAuth provider ID |
| **Source** | Third-party identity provider (Replit Auth: GitHub, Google, Apple, X, email/password) |
| **Legal Basis (GDPR)** | Legitimate interest (Art. 6(1)(f)) — necessary for account access |
| **Legal Basis (PIPEDA)** | Implied consent (s. 6.1) — necessary for service provision |
| **Processing Purpose** | User identification, session management, access control |
| **Storage Location** | PostgreSQL database (Replit-hosted, US infrastructure) |
| **Retention Period** | Duration of account + 3 years post-closure |
| **Recipients** | Internal application logic only; no third-party sharing |
| **Cross-Border Transfer** | Canada → US (Replit hosting); SCCs recommended |

### 3.2 API Request Logging

| Attribute | Detail |
|-----------|--------|
| **Data Categories** | IP address, User-Agent string, request timestamps, endpoint accessed, response codes |
| **Source** | Automated collection from HTTP requests |
| **Legal Basis (GDPR)** | Legitimate interest (Art. 6(1)(f)) — security monitoring |
| **Legal Basis (PIPEDA)** | Implied consent — security and service improvement |
| **Processing Purpose** | Security monitoring, rate limiting, abuse detection, performance analysis |
| **Storage Location** | Server logs (Replit infrastructure, US) |
| **Retention Period** | 12 months |
| **Recipients** | Internal operations; law enforcement upon lawful request |
| **Cross-Border Transfer** | Canada → US (Replit hosting) |

### 3.3 Contact Form Submissions

| Attribute | Detail |
|-----------|--------|
| **Data Categories** | Name, email address, subject, message body |
| **Source** | User-submitted via contact form |
| **Legal Basis (GDPR)** | Consent (Art. 6(1)(a)) |
| **Legal Basis (PIPEDA)** | Express consent at time of submission |
| **Processing Purpose** | Responding to inquiries, business development |
| **Storage Location** | PostgreSQL database (Replit-hosted, US) |
| **Retention Period** | 24 months from submission |
| **Recipients** | Internal staff only |
| **Cross-Border Transfer** | Canada → US (Replit hosting) |

### 3.4 Agent Array Queries

| Attribute | Detail |
|-----------|--------|
| **Data Categories** | Query text (may contain PII if user-submitted), agent responses, report text |
| **Source** | User-submitted query to 28-Dimension Agent Array |
| **Legal Basis (GDPR)** | Consent (Art. 6(1)(a)) — user initiates query |
| **Legal Basis (PIPEDA)** | Express consent — voluntary submission |
| **Processing Purpose** | AI-assisted analysis, report generation |
| **Storage Location** | PostgreSQL database (Replit-hosted, US); OpenAI API (transient processing) |
| **Retention Period** | Reports retained until user deletion; query text not retained separately |
| **Recipients** | OpenAI (data processor, transient); internal application |
| **Cross-Border Transfer** | Canada → US (Replit, OpenAI) |
| **DPA Required** | Yes — OpenAI Data Processing Addendum applies |

### 3.5 HPTP Timing Data

| Attribute | Detail |
|-----------|--------|
| **Data Categories** | Timing metadata (epoch offsets, correction values, synchronization states) |
| **Source** | Automated generation by HPTP service |
| **Personal Data** | No — timing metadata is not associated with individual users |
| **Processing Purpose** | Femtosecond-precision timing synchronization |
| **Storage Location** | In-memory; no persistent storage |

### 3.6 Cryptographic Operations

| Attribute | Detail |
|-----------|--------|
| **Data Categories** | Encrypted/decrypted data (transient), compression demo data |
| **Source** | User-submitted via API or demo interface |
| **Personal Data** | Potentially — depends on user input |
| **Processing Purpose** | Demonstrating encryption/compression capabilities |
| **Storage Location** | In-memory only; no persistent storage of plaintext |
| **Retention Period** | Not retained; processed and returned in-memory |

### 3.7 Session Management

| Attribute | Detail |
|-----------|--------|
| **Data Categories** | Session tokens (AES-256-GCM encrypted), session expiry timestamps |
| **Source** | Generated upon authentication |
| **Processing Purpose** | Maintaining authenticated user sessions |
| **Storage Location** | Server-side memory/database; client-side cookie |
| **Retention Period** | Duration of session; cleared on logout or expiry |
| **Encryption** | AES-256-GCM with SESSION_SECRET (no fallback to insecure defaults) |

---

## 4. Data Flow Diagram (Textual)

```
User Browser
  │
  ├─── [HTTPS/TLS 1.3] ──→ Replit Edge (US)
  │                              │
  │                              ├──→ Express.js Backend
  │                              │        │
  │                              │        ├──→ PostgreSQL (Neon, US)
  │                              │        │      └── Users, Sessions, Reports, Contacts
  │                              │        │
  │                              │        ├──→ OpenAI API (US, transient)
  │                              │        │      └── Agent Array queries (no retention by OpenAI)
  │                              │        │
  │                              │        ├──→ GitHub API (US)
  │                              │        │      └── Repository management (admin only)
  │                              │        │
  │                              │        └──→ Kong Konnect API (US)
  │                              │             └── Gateway configuration (admin only)
  │                              │
  │                              └──→ Vite Frontend (static assets)
  │
  └─── [No direct external calls from browser except OAuth redirects]
```

---

## 5. Cross-Border Transfer Safeguards

### 5.1 Canada → United States

| Mechanism | Status |
|-----------|--------|
| **Adequacy Decision** | No EU adequacy for US (post-Schrems II); Canada has PIPEDA adequacy from EU |
| **Standard Contractual Clauses** | Recommended for Replit hosting DPA |
| **PIPEDA Comparable Protections** | Required under PIPEDA s. 6.1 for transfers outside Canada |
| **Contractual Safeguards** | Replit Terms of Service; OpenAI DPA |

### 5.2 Recommended Actions

1. Execute DPA with Replit for PostgreSQL hosting and application hosting
2. Confirm OpenAI DPA covers Agent Array data processing
3. Review GitHub DPA for repository data (code, not personal data)
4. Monitor for Canadian adequacy status changes

---

## 6. Third-Party Processors

| Processor | Service | Data Processed | DPA Status |
|-----------|---------|---------------|------------|
| Replit, Inc. | Application hosting, PostgreSQL | All platform data | Review required |
| Neon, Inc. | PostgreSQL database | User data, reports, sessions | Via Replit DPA |
| OpenAI, Inc. | LLM inference | Agent Array queries | Review required |
| GitHub, Inc. | Code repository | Source code (no user PII) | Standard GitHub DPA |
| Kong, Inc. | API gateway management | API configuration (no user PII) | Standard terms |

---

## 7. Data Subject Rights Fulfillment

| Right | Mechanism | Response Time |
|-------|-----------|--------------|
| Access (PIPEDA s. 8) | Privacy Officer request | 30 days |
| Correction (PIPEDA s. 12.2) | Privacy Officer request | 30 days |
| Erasure (GDPR Art. 17) | Account deletion + data purge | 30 days |
| Portability (GDPR Art. 20) | JSON export of user data | 30 days |
| Withdrawal of Consent | Privacy Officer notification | Immediate |

---

## 8. Review Schedule

This Data Flow Map shall be reviewed and updated:
- **Quarterly** or upon any material change to data processing activities
- **Immediately** upon addition of new third-party processors or data categories
- **Annually** as part of the privacy impact assessment cycle

---

*Capomastro Holdings Ltd. — Applied Physics Division*
*This document is maintained for compliance purposes and does not constitute legal advice.*
