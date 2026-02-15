# Incident Response Plan
## PlenumNET — Security Incident Handling Procedures
### Capomastro Holdings Ltd., Applied Physics Division

**Last Updated:** February 15, 2026
**Classification:** INTERNAL — Security Operations
**Document Version:** 1.0

---

## 1. Purpose

This Incident Response Plan ("IRP") establishes procedures for detecting, responding to, containing, and recovering from security incidents affecting the PlenumNET platform and Salvi Framework. It supports compliance with PIPEDA's mandatory breach reporting requirements (s. 10.1), GDPR Article 33/34 notification obligations, and the Company's commitment to protecting user data and system integrity.

---

## 2. Scope

This plan covers:
- Unauthorized access to systems, data, or accounts
- Data breaches involving personal information
- Cryptographic key compromise
- Denial of service attacks
- Malware or supply-chain compromise
- Insider threats or privilege abuse
- Vulnerability exploitation
- Unauthorized disclosure of proprietary algorithms or trade secrets

---

## 3. Roles and Responsibilities

### 3.1 Incident Response Team

| Role | Responsibility |
|------|---------------|
| **Incident Commander** | Overall coordination; escalation decisions; external communication |
| **Privacy Officer** | Breach notification assessment; regulatory communication (OPC, DPAs) |
| **Technical Lead** | Technical investigation; containment and remediation |
| **Legal Counsel** | Legal exposure assessment; regulatory compliance; law enforcement liaison |
| **Communications Lead** | User notification drafting; public disclosure if required |

### 3.2 Contact Chain

1. Discovery → Technical Lead (immediate)
2. Technical Lead → Incident Commander (within 1 hour)
3. Incident Commander → Privacy Officer (within 4 hours if PII involved)
4. Privacy Officer → Legal Counsel (within 24 hours if breach confirmed)

---

## 4. Incident Classification

### 4.1 Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|---------|
| **P1 — Critical** | Active breach with confirmed data exfiltration or key compromise | Immediate (< 1 hour) | Cryptographic key leak; database exfiltration; active attacker in system |
| **P2 — High** | Confirmed unauthorized access without evidence of data exfiltration | < 4 hours | Unauthorized admin access; session hijacking; privilege escalation |
| **P3 — Medium** | Vulnerability discovered with potential for exploitation | < 24 hours | Unpatched CVE in dependency; misconfigured access control; rate-limit bypass |
| **P4 — Low** | Minor security event; informational | < 72 hours | Failed brute-force attempt; port scan; benign anomaly in logs |

---

## 5. Incident Response Phases

### 5.1 Detection and Identification

**Automated Detection:**
- CodeQL static analysis (GitHub Actions, on every push and weekly)
- Gitleaks secret detection (on every push)
- npm audit and cargo audit (weekly scheduled)
- Rate limiting alerts (threshold: 100 req/min global, 20/min auth, 10/min token)
- Application error monitoring via structured logging

**Manual Detection:**
- Security vulnerability reports via GitHub Security Advisories
- User reports to Rsalvi@Salvigroup.com
- Periodic security review of access logs and audit trails

**Identification Checklist:**
- [ ] What systems or data are affected?
- [ ] When did the incident begin (or was first detected)?
- [ ] Is the incident ongoing?
- [ ] What is the potential impact on personal data?
- [ ] What severity level applies?

### 5.2 Containment

**Immediate Containment (P1/P2):**
- Revoke compromised credentials (API keys, session tokens, OAuth tokens)
- Rotate SESSION_SECRET if session compromise suspected
- Block attacker IP addresses via rate limiting or CORS enforcement
- Isolate affected database tables or services
- Disable affected API endpoints if necessary

**Short-Term Containment:**
- Deploy patches or configuration changes
- Enable enhanced logging on affected systems
- Restrict administrative access to essential personnel

### 5.3 Eradication

- Identify root cause through log analysis and code review
- Remove attacker persistence mechanisms (if any)
- Patch exploited vulnerability
- Update dependency versions if supply-chain compromise
- Regenerate all affected cryptographic material
- Clear compromised caches and sessions

### 5.4 Recovery

- Restore services from known-good state (checkpoint rollback if available)
- Verify integrity of restored data
- Re-enable disabled endpoints with enhanced monitoring
- Conduct post-restoration testing
- Monitor for recurrence (elevated alerting for 30 days)

### 5.5 Post-Incident Review

Within 5 business days of incident resolution:
- Conduct post-incident review meeting
- Document root cause, timeline, and actions taken
- Identify process improvements
- Update this IRP if procedural gaps identified
- File incident report in internal records (retain 6 years per PIPEDA)

---

## 6. Breach Notification

### 6.1 PIPEDA Mandatory Breach Reporting (s. 10.1)

If a breach of security safeguards creates a **real risk of significant harm** (RROSH):

| Action | Timeline | Recipient |
|--------|----------|-----------|
| Report to OPC | As soon as feasible | Office of the Privacy Commissioner of Canada |
| Notify affected individuals | As soon as feasible | All individuals whose information was involved |
| Notify other organizations | As applicable | Organizations that may mitigate harm |
| Record the breach | Immediately | Internal breach register (retain 24 months minimum) |

**RROSH Assessment Factors:**
- Sensitivity of the personal information
- Probability that the information has been or will be misused
- Number of individuals affected

### 6.2 GDPR Notification (Art. 33/34)

If EU/EEA user data is affected:

| Action | Timeline |
|--------|----------|
| Notify supervisory authority | Within 72 hours of becoming aware |
| Notify data subjects | Without undue delay if high risk to rights and freedoms |

### 6.3 CCPA/CPRA Notification

If California resident data is affected:
- Notify affected individuals in the most expedient time possible
- Notify the California Attorney General if > 500 California residents affected

---

## 7. Communication Templates

### 7.1 Internal Escalation

```
Subject: [SECURITY INCIDENT] [P-Level] — Brief Description
Date/Time Detected: YYYY-MM-DD HH:MM UTC
Severity: P1/P2/P3/P4
Affected Systems: [list]
Current Status: Investigating / Contained / Resolved
PII Involved: Yes/No/Unknown
Immediate Actions Taken: [list]
Next Steps: [list]
```

### 7.2 Regulatory Notification (OPC)

Per PIPEDA Breach of Security Safeguards Regulations (SOR/2018-64):
- Description of the circumstances of the breach
- Date or period of the breach
- Description of the personal information involved
- Number of affected individuals
- Steps taken to reduce risk of harm
- Steps taken or planned to notify affected individuals

### 7.3 User Notification

```
Subject: Important Security Notice Regarding Your PlenumNET Account

Dear [Name],

We are writing to inform you of a security incident that may have
affected your personal information associated with your PlenumNET account.

What Happened: [description]
When It Happened: [date range]
What Information Was Involved: [categories]
What We Are Doing: [actions taken]
What You Can Do: [recommended user actions]

Contact: Rsalvi@Salvigroup.com
```

---

## 8. Cryptographic Incident Procedures

Given the platform's post-quantum cryptographic infrastructure, additional procedures apply:

### 8.1 Key Compromise

- Immediately rotate all affected keys
- If SESSION_SECRET compromised: invalidate all active sessions, generate new secret
- If TL-KEM/TL-DSA keys compromised: revoke and reissue
- If Lamport OTS keys compromised: keys are one-time use; verify no double-signing occurred
- Audit all signed records for integrity

### 8.2 Algorithm Vulnerability

- Assess whether vulnerability affects standard (AES-256, SHA-384) or proprietary (TL-KEM, Phase Encryption) implementations
- For standard algorithm vulnerabilities: follow NIST/CCCS advisories
- For proprietary algorithm vulnerabilities: engage internal cryptographic review
- Consider activating fallback to alternative algorithm suite

---

## 9. Testing and Maintenance

| Activity | Frequency |
|----------|-----------|
| Tabletop exercise | Annually |
| IRP document review | Semi-annually |
| Contact chain verification | Quarterly |
| Detection capability testing | Monthly (automated) |
| Post-incident review updates | After every P1/P2 incident |

---

## 10. Record Retention

All incident records shall be retained for a minimum of:
- **6 years** (PIPEDA general requirement)
- **24 months** for breach records (PIPEDA Breach of Security Safeguards Regulations)
- Indefinitely for incidents involving patent-pending technology compromise

---

*Capomastro Holdings Ltd. — Applied Physics Division*
*This document is maintained for compliance purposes and does not constitute legal advice.*
