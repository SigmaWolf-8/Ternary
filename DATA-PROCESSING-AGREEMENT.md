# Data Processing Agreement

**Version 1.0 — Effective Date: February 12, 2026**

**Between:**

**Data Controller:** [Customer Name] ("Controller")

**Data Processor:** Capomastro Holdings Ltd., incorporated under the laws of the Province of Alberta, Canada, operating as PlenumNET ("Processor")

---

## 1. Definitions

1.1 **"Personal Data"** means any information relating to an identified or identifiable natural person as defined by applicable Data Protection Laws.

1.2 **"Data Protection Laws"** means all applicable laws and regulations relating to the processing of Personal Data, including but not limited to:
- The Personal Information Protection and Electronic Documents Act (PIPEDA) (Canada)
- The Alberta Personal Information Protection Act (PIPA)
- The General Data Protection Regulation (EU) 2016/679 (GDPR)
- The California Consumer Privacy Act (CCPA) / California Privacy Rights Act (CPRA)

1.3 **"Processing"** means any operation performed on Personal Data, including collection, recording, organization, structuring, storage, adaptation, retrieval, consultation, use, disclosure, combination, restriction, erasure, or destruction.

1.4 **"Sub-processor"** means any third party engaged by the Processor to process Personal Data on behalf of the Controller.

1.5 **"Services"** means the PlenumNET Salvi Framework services, APIs, and infrastructure provided under the applicable service agreement.

## 2. Scope and Purpose

2.1 This Data Processing Agreement ("DPA") forms part of the service agreement between Controller and Processor and governs the processing of Personal Data by Processor on behalf of Controller.

2.2 The Processor shall process Personal Data solely for the purpose of providing the Services and in accordance with the Controller's documented instructions.

2.3 **Categories of Data Subjects:** [To be specified by Controller — e.g., end users, employees, customers]

2.4 **Types of Personal Data:** [To be specified by Controller — e.g., names, email addresses, IP addresses, usage data, authentication tokens]

2.5 **Duration of Processing:** For the term of the service agreement plus any retention period required by law or agreed upon in writing.

## 3. Obligations of the Processor

3.1 **Lawful Processing.** The Processor shall process Personal Data only on documented instructions from the Controller, unless required to do so by applicable law.

3.2 **Confidentiality.** The Processor shall ensure that persons authorized to process Personal Data have committed themselves to confidentiality or are under an appropriate statutory obligation of confidentiality.

3.3 **Security Measures.** The Processor shall implement appropriate technical and organizational measures to ensure a level of security appropriate to the risk, including:

- (a) AES-256-GCM encryption of data at rest and in transit
- (b) Post-quantum cryptographic protections (CNSA 2.0 compliant algorithms)
- (c) Access controls with role-based permissions
- (d) Regular security assessments and penetration testing
- (e) Incident detection and response capabilities
- (f) Backup and disaster recovery procedures
- (g) Femtosecond-precision audit logging

3.4 **Sub-processors.** The Processor shall not engage a Sub-processor without prior specific or general written authorization of the Controller. Where general authorization is given, the Processor shall inform the Controller of any intended addition or replacement of Sub-processors, giving the Controller the opportunity to object.

3.5 **Data Subject Rights.** The Processor shall assist the Controller in fulfilling its obligation to respond to requests from data subjects exercising their rights under Data Protection Laws.

3.6 **Breach Notification.** The Processor shall notify the Controller without undue delay (and in any event within 72 hours) after becoming aware of a Personal Data breach.

3.7 **Data Protection Impact Assessments.** The Processor shall assist the Controller with data protection impact assessments and prior consultations with supervisory authorities where required.

3.8 **Deletion and Return.** Upon termination of the service agreement, the Processor shall, at the Controller's choice, delete or return all Personal Data and delete existing copies unless applicable law requires storage.

## 4. Obligations of the Controller

4.1 The Controller warrants that it has a lawful basis for processing Personal Data and has provided all necessary notices and obtained all necessary consents.

4.2 The Controller shall provide documented instructions for the processing of Personal Data that comply with applicable Data Protection Laws.

4.3 The Controller shall promptly notify the Processor of any changes to applicable Data Protection Laws that may affect the Processor's obligations.

## 5. International Data Transfers

5.1 The Processor shall not transfer Personal Data to a country outside Canada or the European Economic Area without ensuring adequate safeguards are in place, which may include:

- (a) Standard Contractual Clauses approved by the European Commission
- (b) Binding Corporate Rules
- (c) Adequacy decisions by the relevant supervisory authority
- (d) The consent of the data subject

5.2 The Processor's primary data processing infrastructure is located in Canada. The Processor shall inform the Controller of any change to the location of data processing.

## 6. Audits and Inspections

6.1 The Processor shall make available to the Controller all information necessary to demonstrate compliance with this DPA and allow for and contribute to audits, including inspections, conducted by the Controller or an auditor mandated by the Controller.

6.2 Audits shall be conducted with reasonable notice (minimum 30 days), during normal business hours, and no more than once per calendar year unless required by a supervisory authority or following a data breach.

6.3 The Processor may satisfy audit requirements by providing:
- (a) SOC 2 Type II audit reports
- (b) FIPS 140-3 certification documentation
- (c) CNSA 2.0 compliance attestation
- (d) Independent third-party security assessment reports

## 7. Liability and Indemnification

7.1 Each party's liability under this DPA is subject to the limitations and exclusions of liability set out in the service agreement.

7.2 The Processor shall indemnify the Controller against all claims, actions, third-party proceedings, losses, damages, and expenses arising from the Processor's breach of this DPA or applicable Data Protection Laws.

## 8. Sub-processors

8.1 **Current Sub-processors.** The Processor currently engages the following Sub-processors:

| Sub-processor | Purpose | Location |
|---|---|---|
| [Infrastructure Provider] | Cloud hosting and compute | [Location] |
| [Database Provider] | Data storage | [Location] |
| [CDN Provider] | Content delivery | [Location] |

8.2 The Controller hereby provides general authorization for the Processor to engage Sub-processors, subject to the notification and objection mechanism in Section 3.4.

## 9. PIPEDA-Specific Provisions

9.1 The Processor acknowledges that Personal Data processed under this DPA may be subject to PIPEDA and commits to compliance with PIPEDA Principles 1-10.

9.2 The Processor shall designate a Privacy Officer responsible for the organization's compliance with this DPA and applicable privacy laws.

9.3 The Processor shall cooperate with the Office of the Privacy Commissioner of Canada in any investigation or inquiry relating to the processing of Personal Data under this DPA.

## 10. GDPR-Specific Provisions

10.1 Where the GDPR applies, the Processor shall:
- (a) Maintain a record of processing activities carried out on behalf of the Controller
- (b) Cooperate with supervisory authorities
- (c) Appoint a data protection officer where required by Article 37 of the GDPR

## 11. CCPA/CPRA-Specific Provisions

11.1 Where the CCPA/CPRA applies, the Processor shall:
- (a) Not sell or share Personal Data
- (b) Not retain, use, or disclose Personal Data for any purpose other than providing the Services
- (c) Comply with requests from the Controller to delete Personal Data

## 12. Term and Termination

12.1 This DPA shall remain in effect for the duration of the service agreement between the parties.

12.2 Sections 3.8 (Deletion and Return), 6 (Audits), and 7 (Liability) shall survive termination of this DPA.

## 13. Governing Law and Jurisdiction

13.1 This DPA shall be governed by and construed in accordance with the laws of the Province of Alberta, Canada.

13.2 Any disputes arising under this DPA shall be subject to the exclusive jurisdiction of the courts of the Province of Alberta.

## 14. Amendments

14.1 This DPA may be amended only by written agreement signed by both parties.

14.2 The Processor may update the technical and organizational security measures described in Section 3.3 from time to time, provided that such updates do not materially decrease the overall level of protection.

---

**AGREED AND ACCEPTED:**

**Controller:**

Name: ____________________
Title: ____________________
Date: ____________________
Signature: ____________________

**Processor (Capomastro Holdings Ltd.):**

Name: ____________________
Title: ____________________
Date: ____________________
Signature: ____________________
