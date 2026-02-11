# Export Control Notice
## Salvi Framework — PlenumNET Ternary Computing Platform
### Capomastro Holdings Ltd., Applied Physics Division

**Effective Date:** February 11, 2026  
**Classification Review Date:** February 11, 2026

---

## 1. Classification Statement

The Salvi Framework and PlenumNET platform contain cryptographic functionality that is subject to the export control laws and regulations of the United States and other jurisdictions. Based on a self-classification review of the software's capabilities, the following determination has been made:

| Parameter | Classification |
|-----------|---------------|
| **Export Control Classification Number (ECCN)** | **5D002** |
| **Reason for Control** | National Security (NS), Anti-Terrorism (AT) |
| **Applicable Regulations** | U.S. Export Administration Regulations (EAR), 15 CFR Parts 730–774 |
| **License Exception** | ENC (§740.17) — subject to conditions described below |
| **Wassenaar Arrangement Category** | Category 5, Part 2 — Information Security |

### 1.1 Basis for ECCN 5D002 Classification

The Software implements, contains, or provides access to the following controlled cryptographic algorithms and functionality:

| Algorithm / Function | Standard | Key Length / Security Level | EAR Control Basis |
|---------------------|----------|---------------------------|-------------------|
| AES-256-GCM | FIPS 197 / SP 800-38D | 256-bit symmetric | §742.15(b)(1) — symmetric >56-bit |
| ML-KEM-512 / 768 / 1024 | FIPS 203 | Lattice-based KEM, multiple levels | §742.15(b)(3) — asymmetric key exchange |
| ML-DSA-44 / 65 / 87 | FIPS 204 | Lattice-based signatures | §742.15(b)(3) — asymmetric authentication |
| SHA-384 / SHA-512 | FIPS 180-4 | 384/512-bit digest | Ancillary to controlled algorithms |
| SHA3-384 / SHA3-512 | FIPS 202 | 384/512-bit digest | Ancillary to controlled algorithms |
| HMAC-SHA-384 / HMAC-SHA-512 | FIPS 198-1 | Keyed hash | Ancillary to controlled algorithms |
| LMS / XMSS | SP 800-208 | Stateful hash-based signatures | §742.15(b)(3) — asymmetric authentication |
| HMAC-DRBG-SHA384 | SP 800-90A | Deterministic random bit generation | Ancillary to controlled algorithms |
| Phase Encryption | Proprietary | Dual-phase, Tribonacci-weighted | §742.15(b)(1) — proprietary encryption |
| TL-KEM / TL-DSA | Proprietary (interop bridge) | Ternary-lattice hybrid | §742.15(b)(3) — proprietary key exchange/auth |

### 1.2 Post-Quantum Considerations

As of the date of this notice, NIST post-quantum algorithms (ML-KEM, ML-DSA) are newly standardized under FIPS 203 and FIPS 204. The Bureau of Industry and Security (BIS) has not issued specific guidance distinguishing post-quantum algorithms from classical cryptographic controls. Accordingly, this classification treats post-quantum algorithms under the same ECCN 5D002 framework applicable to classical asymmetric cryptography. This classification should be reviewed upon issuance of BIS guidance specific to post-quantum cryptography.

---

## 2. License Exception ENC (§740.17)

The Software may qualify for export under License Exception ENC, subject to the following conditions:

### 2.1 Self-Classification Filing

Pursuant to §740.17(b)(1), a self-classification report must be filed with BIS and the ENC Encryption Request Coordinator (NSA) before any export or re-export under License Exception ENC. The required information includes:

- Product name and model/version number
- ECCN and authorization paragraph under §740.17
- Encryption algorithm and key length
- Description of encryption functionality
- Product availability (commercial, custom, or internal use)

**Status:** Self-classification filing is PENDING. No exports may occur under License Exception ENC until this filing is complete and acknowledged.

### 2.2 Eligible End-Users and Destinations

Under License Exception ENC, the Software may be exported to most commercial end-users in most countries, **except**:

- **Embargoed Countries (Country Group E:1 and E:2):** Cuba, Iran, North Korea, Syria, and the Crimea, Donetsk, and Luhansk regions of Ukraine. Export to these destinations is **prohibited** without a specific BIS license.
- **Military, Intelligence, and Law Enforcement End-Users in Country Group D:1:** Exports to government end-users (as defined in §740.17(d)) in countries listed in Country Group D:1 (including but not limited to China, Russia, Venezuela, and Belarus) require a license and do not qualify for License Exception ENC.
- **Entity List / SDN List:** No export to any entity or individual on the BIS Entity List, OFAC Specially Designated Nationals (SDN) List, or Denied Persons List.

### 2.3 SaaS / Cloud Access Considerations

Access to the PlenumNET SaaS platform (including API endpoints that perform cryptographic operations) constitutes a "deemed export" when accessed by foreign nationals and an "export" when accessed from foreign territories. The EAR applies to cloud-based cryptographic services in the same manner as downloadable software. Geo-blocking or access controls should be implemented for embargoed destinations.

---

## 3. ITAR Considerations

Based on the current functionality of the Software, it does not appear to fall under the International Traffic in Arms Regulations (ITAR) or the United States Munitions List (USML). The cryptographic functionality is commercial in nature and is not specifically designed, developed, configured, adapted, or modified for military applications.

However, if the Software is subsequently modified for or integrated into defense articles, or if it is specifically designed for military, intelligence, or space applications, a separate ITAR jurisdictional determination should be conducted.

---

## 4. Wassenaar Arrangement

The Wassenaar Arrangement on Export Controls for Conventional Arms and Dual-Use Goods and Technologies includes cryptographic software in Category 5, Part 2 (Information Security). Participating states implement Wassenaar controls through their national export control regimes. Users and distributors of the Software outside the United States should consult their national export control authority for applicable requirements.

Key Wassenaar participating states with significant cryptographic export controls include: Australia, Canada, France, Germany, Japan, the Netherlands, and the United Kingdom. Regulations vary by jurisdiction.

---

## 5. Compliance Obligations for Users

By accessing, downloading, or using the Software or the PlenumNET Service, you acknowledge and agree that:

5.1 You will comply with all applicable export control laws and regulations, including the EAR, ITAR, EU Dual-Use Regulation (EU 2021/821), and equivalent laws of your jurisdiction.

5.2 You are not located in, and will not access the Software from, any country subject to a U.S. comprehensive embargo (Cuba, Iran, North Korea, Syria, Crimea/Donetsk/Luhansk regions of Ukraine).

5.3 You are not listed on, and are not acting on behalf of any entity listed on, the BIS Entity List, OFAC SDN List, Denied Persons List, or any equivalent restricted party list.

5.4 You will not use the Software for any end-use prohibited by the EAR, including but not limited to: the design, development, production, stockpiling, or use of chemical, biological, or nuclear weapons, or missiles capable of delivering such weapons.

5.5 You will not re-export, transfer, or divert the Software or any technical data derived from it to any destination, end-user, or end-use prohibited by applicable export control laws without first obtaining the required authorization.

---

## 6. Encryption Source Code

Portions of the Software's encryption source code are publicly available on GitHub. Under EAR §742.15(b) Note, publicly available encryption source code is not subject to the EAR when it is available to the public without restriction, provided that:

- Notification is sent to BIS and the ENC Encryption Request Coordinator
- The code is not subject to an express agreement for payment of a licensing fee or royalty for commercial production or sale of any product

The current repository is publicly visible but subject to a proprietary license that restricts commercial use. Accordingly, the publicly available source code exception may **not** apply, and the Software should be treated as controlled under ECCN 5D002 regardless of its visibility on GitHub.

---

## 7. Record-Keeping

Capomastro Holdings Ltd. will maintain records of all exports, re-exports, and transfers of the Software as required by EAR §762. Records will be retained for a minimum of five (5) years from the date of export.

---

## 8. Updates to This Notice

This Export Control Notice will be updated upon:

- Changes to the cryptographic functionality of the Software
- Issuance of BIS guidance on post-quantum cryptographic controls
- Changes to the EAR, ITAR, Wassenaar Arrangement, or equivalent regulations
- Completion of the License Exception ENC self-classification filing
- FIPS 140-3 CMVP certification (which may affect classification)

---

## 9. Contact

For export control inquiries or license requests:

Capomastro Holdings Ltd.  
Applied Physics Division  
98 Sioux Rd  
Sherwood Park, AB Canada T8A-3X5  
Email: Rsalvi@Salvigroup.com

For U.S. Government inquiries:  
Bureau of Industry and Security: https://www.bis.doc.gov  
OFAC: https://ofac.treasury.gov

---

*This notice is provided for compliance purposes and does not constitute legal advice. Capomastro Holdings Ltd. should consult with qualified export control counsel before initiating any export, re-export, or transfer of the Software. The classifications stated herein are based on self-assessment and have not been validated by BIS or any government authority.*
