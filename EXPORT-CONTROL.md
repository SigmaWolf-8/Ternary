# Export Control Classification & Compliance

## Capomastro Holdings Ltd. — Applied Physics Division

**Effective Date:** February 14, 2026
**Classification:** PUBLIC
**Document Version:** 1.0

---

## 1. Purpose

This document provides export control classification guidance for the Salvi Framework / PlenumNET platform and its constituent cryptographic components. It addresses compliance with Canadian, U.S., and international export control regimes applicable to cryptographic software.

---

## 2. Applicable Regulatory Frameworks

### 2.1 Canadian Export Controls
- **Export and Import Permits Act** (R.S.C., 1985, c. E-19)
- **Export Control List** (SOR/89-202), Group 1 (Dual-Use List), Category 5, Part 2 (Information Security)
- **Area Control List** (countries subject to Canadian sanctions)
- **Global Affairs Canada** licensing authority

### 2.2 United States Export Controls
- **Export Administration Regulations (EAR)**, 15 CFR Parts 730-774
- **ECCN 5D002** — Information Security software
- **License Exception ENC** (Section 740.17) — Encryption software provisions

### 2.3 International Frameworks
- **Wassenaar Arrangement** on Export Controls for Conventional Arms and Dual-Use Goods and Technologies
  - Category 5, Part 2: Information Security
  - Cryptography Note (Note 3 to Category 5, Part 2)

---

## 3. Cryptographic Component Classification

### 3.1 Post-Quantum Cryptographic Primitives

| Component | Key Size | Classification Notes |
|-----------|----------|---------------------|
| AES-256-GCM | 256-bit | Standard symmetric encryption. Controlled under Wassenaar Cat. 5.2 |
| SHA-2 / SHA-3 | N/A | Hash functions. Generally not controlled as standalone |
| TL-KEM (Ternary Lattice KEM) | Proprietary | Novel post-quantum KEM. Subject to classification review |
| TL-DSA (Ternary Lattice DSA) | Proprietary | Novel post-quantum signature. Subject to classification review |
| Phase Encryption | Proprietary | Novel timing-gated encryption. Subject to classification review |
| Ternary Sponge | Proprietary | Novel hash construction. Subject to classification review |
| Lamport Signatures (Ternary) | Proprietary | One-time signature scheme. Subject to classification review |
| HMAC-based KDF | Standard | Key derivation. Controlled under Wassenaar Cat. 5.2 |

### 3.2 CNSA 2.0 Compliance Claims

The platform implements or references the following algorithms from the NSA Commercial National Security Algorithm Suite 2.0:

| CNSA 2.0 Algorithm | Standard | Platform Implementation |
|---------------------|----------|------------------------|
| AES-256 | FIPS 197 | Implemented (AES-256-GCM mode) |
| SHA-384 / SHA-512 | FIPS 180-4 | Implemented |
| XMSS / LMS | NIST SP 800-208 | Referenced (hash-based signatures) |
| ML-KEM (CRYSTALS-Kyber) | FIPS 203 | Referenced (lattice KEM baseline) |
| ML-DSA (CRYSTALS-Dilithium) | FIPS 204 | Referenced (lattice DSA baseline) |

**Note:** The platform's TL-KEM and TL-DSA are proprietary ternary-field adaptations and are NOT direct implementations of ML-KEM/ML-DSA. CNSA 2.0 compliance claims apply to the standard algorithm implementations only.

---

## 4. Export Classification Assessment

### 4.1 Canadian Classification
Under the Canadian Export Control List, Group 1, Category 5, Part 2:

- The platform contains **controlled cryptographic technology** exceeding the thresholds in Entry 5.A.2.a (symmetric algorithms exceeding 56-bit key length) and Entry 5.D.2 (software for development or use of controlled cryptographic items).
- **Export permit may be required** for transfers to countries not listed in the General Export Permit (GEP) No. 12 (Cryptographic Goods).
- Transfers within Canada, to the United States, and to Wassenaar participating states may be eligible for GEP No. 12 subject to end-use restrictions.

### 4.2 Wassenaar Classification
- **5.D.2.a.1** — Software specially designed for development or production of information security items specified in Category 5, Part 2.
- The **Cryptography Note** (Note 3) exclusions for mass-market software do NOT apply, as this is specialized cryptographic infrastructure, not a consumer product.

### 4.3 U.S. Classification (if applicable)
- **ECCN 5D002.c.1** — Software with encryption functionality exceeding 56-bit symmetric / 512-bit asymmetric / 112-bit elliptic curve.
- **License Exception ENC** may apply for certain end-users and destinations after classification filing with BIS (Bureau of Industry and Security).

---

## 5. Restricted Destinations

Exports are prohibited or restricted to the following jurisdictions without specific government authorization:

### 5.1 Canadian Sanctions
Countries subject to the *Special Economic Measures Act* (SEMA), *United Nations Act* sanctions, or Area Control List restrictions.

### 5.2 General Restrictions
This software must NOT be exported, re-exported, or transferred to:
- Countries subject to comprehensive sanctions by Canada, the United States, the European Union, or the United Nations
- End-users engaged in proliferation of weapons of mass destruction, missile technology, or military end-uses inconsistent with Canadian foreign policy
- Any party listed on the Canadian Consolidated Autonomous Sanctions List or equivalent restricted party lists

---

## 6. Compliance Procedures

### 6.1 Pre-Export Screening
Before any transfer of this software or its components outside Canada:
1. Verify the recipient is not on any restricted party list
2. Confirm the destination country is not subject to comprehensive sanctions
3. Assess end-use and end-user against proliferation concerns
4. Obtain export permit if required under the Export and Import Permits Act

### 6.2 Record Keeping
Maintain records of all transfers for a minimum of six (6) years as required by Canadian export control regulations.

### 6.3 Technology Transfer
Providing access to source code, technical data, or cryptographic specifications to foreign nationals (including via cloud access, remote repositories, or collaborative development) may constitute a deemed export subject to the same controls.

---

## 7. SaaS Deployment Considerations

The PlenumNET platform is deployed as a Software-as-a-Service (SaaS) application. Under current Canadian and Wassenaar guidance:

- **SaaS access** generally does not constitute an "export" of the underlying software, provided:
  - End-users do not receive access to source code or cryptographic implementation details
  - The service is provided from Canadian or allied-nation infrastructure
  - Access controls prevent use by sanctioned parties or countries
- **API access** to cryptographic functionality may be subject to different treatment depending on the level of cryptographic capability exposed

---

## 8. Contact for Export Control Questions

**Capomastro Holdings Ltd.**
Applied Physics Division
Province of Alberta, Canada

For export control classification or licensing questions, contact the company's designated export control officer or retained counsel.

---

*This document is provided for compliance guidance purposes only. It does not constitute legal advice. Capomastro Holdings Ltd. should consult with qualified export control counsel and, where necessary, seek formal classification rulings from Global Affairs Canada or other applicable authorities before executing any export transactions.*
