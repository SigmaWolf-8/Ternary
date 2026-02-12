# Export Control Notice
## Salvi Framework — PlenumNET Ternary Computing Platform
### Capomastro Holdings Ltd., Applied Physics Division

**Effective Date:** February 11, 2026
**Classification Review Date:** February 11, 2026

---

## 1. Overview

Capomastro Holdings Ltd. is a **Canadian corporation**. The Salvi Framework and PlenumNET platform contain cryptographic functionality subject to the export control laws of **both Canada and the United States**, as well as international multilateral regimes. This notice addresses compliance obligations under all applicable frameworks.

---

## 2. Canadian Export Controls

### 2.1 Governing Legislation

- **Export and Import Permits Act** (R.S.C., 1985, c. E-19) ("EIPA")
- **Export Control List** (SOR/89-202) ("ECL")
- **Area Control List** (SOR/81-543)
- **Automatic Firearms Country Control List** (SOR/91-575)
- **Special Economic Measures Act** (S.C. 1992, c. 17) ("SEMA")
- **United Nations Act** (R.S.C., 1985, c. U-2) — implementing UN Security Council sanctions
- **Administered by:** Global Affairs Canada (GAC), Trade Controls Bureau

### 2.2 ECL Classification

| Parameter | Classification |
|-----------|---------------|
| **ECL Group** | **Group 5 — Telecommunications and Information Security** |
| **ECL Category** | **Category 2 — Information Security** |
| **ECL Item** | **5-2.D.1** — Software specially designed or modified for the development, production, or use of technology controlled under 5-2.A.1 (information security systems and equipment) |
| **Wassenaar Basis** | Wassenaar Arrangement, Category 5, Part 2 |

### 2.3 Controlled Cryptographic Functionality

The following cryptographic capabilities trigger ECL Group 5, Category 2 controls:

| Algorithm / Function | Standard | Control Basis |
|---------------------|----------|---------------|
| AES-256-GCM | FIPS 197 / SP 800-38D | Symmetric encryption >56-bit key |
| ML-KEM-512 / 768 / 1024 | FIPS 203 | Asymmetric key exchange |
| ML-DSA-44 / 65 / 87 | FIPS 204 | Asymmetric digital signature |
| SHA-384 / SHA-512 | FIPS 180-4 | Ancillary to controlled algorithms |
| SHA3-384 / SHA3-512 | FIPS 202 | Ancillary to controlled algorithms |
| HMAC-SHA-384 / HMAC-SHA-512 | FIPS 198-1 | Ancillary to controlled algorithms |
| LMS / XMSS | SP 800-208 | Asymmetric authentication |
| HMAC-DRBG-SHA384 | SP 800-90A | Ancillary to controlled algorithms |
| Phase Encryption | Proprietary | Proprietary encryption methodology |
| TL-KEM / TL-DSA | Proprietary (interop bridge) | Proprietary key exchange / authentication |

### 2.4 Canadian Export Permit Requirements

Under EIPA section 7, an **export permit** from GAC is required before exporting controlled goods or technology to any country, unless an exemption applies.

**Exemptions and General Export Permits (GEPs):**
- **GEP No. 45 (Export of Cryptography for Use by Certain Consignees):** May exempt certain exports of mass-market or publicly available cryptographic software. Applicability to the Salvi Framework must be assessed by export counsel, as the Software contains proprietary (non-mass-market) cryptographic implementations.
- **GEP No. 12 (United States):** Permits most controlled exports to the United States without an individual permit. Applicable to the Salvi Framework for US-destined exports.

**Prohibited Destinations:** Export is prohibited to countries on the **Area Control List** (currently: Belarus, North Korea) and to destinations subject to sanctions under SEMA or UN Act regulations (currently including: Iran, Syria, Russia (partial), and others as updated by GAC).

**Status:** Assessment of applicable GEP exemptions is **PENDING**. No exports to controlled destinations may occur until this assessment is complete.

### 2.5 Post-Quantum Considerations (Canadian)

As of the date of this notice, GAC has not issued specific guidance on the classification of post-quantum cryptographic algorithms under the ECL. The Canadian Centre for Cyber Security (CCCS) has issued guidance on post-quantum preparedness (ITSAP.00.017) but has not addressed export classification. This notice treats post-quantum algorithms under the same ECL Group 5 framework applicable to classical cryptography. This classification should be reviewed upon issuance of any GAC or CCCS advisory.

---

## 3. United States Export Controls

### 3.1 Applicability to a Canadian Corporation

The U.S. Export Administration Regulations (EAR) apply to the Software to the extent that it: (a) is of U.S. origin; (b) contains U.S.-origin components or technology; (c) is exported from the United States; or (d) is re-exported to a third country from Canada. As the Software is developed in Canada using primarily Canadian-origin technology, direct EAR jurisdiction over the Software is limited. However, because the PlenumNET SaaS platform may be accessed by U.S. persons and from U.S. territory, and because some third-party dependencies may be of U.S. origin, we maintain EAR compliance as a precautionary measure.

### 3.2 EAR Self-Classification

| Parameter | Classification |
|-----------|---------------|
| **ECCN** | **5D002** |
| **Reason for Control** | National Security (NS), Anti-Terrorism (AT) |
| **License Exception** | ENC (§740.17) — subject to conditions |

### 3.3 License Exception ENC (§740.17)

The Software may qualify for export under License Exception ENC. A **self-classification filing** with BIS and the ENC Encryption Request Coordinator (NSA) is required before export under this exception.

**Status:** Self-classification filing is **PENDING**. No exports may occur under License Exception ENC until this filing is complete.

### 3.4 Prohibited Destinations and End-Users (U.S.)

Under the EAR, the Software may not be exported or re-exported to:

- **Embargoed Countries (Country Group E:1 and E:2):** Cuba, Iran, North Korea, Syria, and the Crimea, Donetsk, and Luhansk regions of Ukraine
- **Military, Intelligence, and Law Enforcement End-Users in Country Group D:1** (including China, Russia, Venezuela, Belarus) without a specific license
- **Entity List / SDN List:** Any entity or individual on the BIS Entity List, OFAC SDN List, or Denied Persons List

### 3.5 SaaS / Cloud Access

Access to the PlenumNET SaaS platform (including API endpoints performing cryptographic operations) constitutes a "deemed export" under the EAR when accessed by foreign nationals from U.S. territory, and an "export" when accessed from foreign territory. Geo-blocking or access controls should be implemented for embargoed destinations.

---

## 4. Wassenaar Arrangement

The Wassenaar Arrangement on Export Controls for Conventional Arms and Dual-Use Goods and Technologies includes cryptographic software in **Category 5, Part 2 (Information Security)**. Canada, the United States, and 40 other participating states implement Wassenaar controls through their national export control regimes.

Users and distributors of the Software outside Canada should consult their national export control authority for applicable requirements.

---

## 5. Compliance Obligations for Users

By accessing, downloading, or using the Software or the PlenumNET Service, you acknowledge and agree that:

5.1 You will comply with all applicable export control laws and regulations, including Canada's EIPA and ECL, the U.S. EAR, and the export control laws of your jurisdiction.

5.2 You are not located in, and will not access the Software from, any country subject to Canadian sanctions under SEMA or the UN Act, or U.S. comprehensive embargoes.

5.3 You are not listed on, and are not acting on behalf of any entity listed on, Canada's Consolidated Canadian Autonomous Sanctions List, the BIS Entity List, OFAC SDN List, Denied Persons List, or any equivalent restricted party list.

5.4 You will not use the Software for any end-use prohibited by applicable export control laws, including the development of weapons of mass destruction.

5.5 You will not re-export, transfer, or divert the Software to any prohibited destination, end-user, or end-use without obtaining required authorization.

---

## 6. Encryption Source Code — Public Availability

Portions of the Software's encryption source code are publicly visible on GitHub. Under Canadian law, the EIPA controls apply to "exports," defined as shipment out of Canada; the public availability of source code on a globally accessible platform may constitute an export. Under the EAR, publicly available encryption source code may qualify for an exclusion under §742.15(b) Note, provided notification is sent to BIS and the code is not subject to a licensing fee for commercial use.

The current repository is publicly visible but subject to a **proprietary license** that restricts commercial use. Accordingly, the EAR publicly available source code exception may **not** apply. The Software should be treated as controlled under both Canadian and U.S. export control regimes regardless of its visibility on GitHub.

---

## 7. Record-Keeping

Capomastro Holdings Ltd. will maintain records of all exports, re-exports, and transfers of the Software as required by the EIPA and applicable regulations, and by EAR §762 to the extent applicable. Records will be retained for a minimum of **six (6) years** from the date of export (the longer of the Canadian and U.S. retention requirements).

---

## 8. Updates to This Notice

This Export Control Notice will be updated upon:

- Changes to the cryptographic functionality of the Software
- Issuance of GAC or CCCS guidance on post-quantum cryptographic export controls
- Issuance of BIS guidance on post-quantum cryptographic controls
- Changes to the EIPA, ECL, EAR, Wassenaar Arrangement, SEMA sanctions, or equivalent regulations
- Completion of the GEP assessment and BIS self-classification filing
- FIPS 140-3 CMVP certification (which may affect classification)

---

## 9. Contact

For export control inquiries or license requests:

Capomastro Holdings Ltd.
Applied Physics Division
Sherwood Park, AB Canada
Email: Rsalvi@Salvigroup.com

For Canadian Government inquiries:
Global Affairs Canada, Trade Controls Bureau: https://www.international.gc.ca
Canadian Centre for Cyber Security (CCCS): https://www.cyber.gc.ca

For U.S. Government inquiries:
Bureau of Industry and Security: https://www.bis.doc.gov
OFAC: https://ofac.treasury.gov

---

*This notice is provided for compliance purposes and does not constitute legal advice. Capomastro Holdings Ltd. should consult with qualified Canadian and U.S. export control counsel before initiating any export, re-export, or transfer of the Software. The classifications stated herein are based on self-assessment and have not been validated by GAC, BIS, or any government authority.*
