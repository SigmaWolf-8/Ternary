# CRYPTOGRAPHIC CHALLENGE BRIEF

---

## T-AE-MAC Authenticated Encryption

### Built on TLSponge-385 — The Salvi Framework

---

| Field | Value |
|---|---|
| **Document ID** | CB-2026-001 Rev. 2 |
| **Classification** | **CONFIDENTIAL** |
| **Issuer** | Capomastro Holdings Ltd. |
| **Division** | Applied Physics Division |
| **Date Issued** | [DATE] |
| **Time Limit** | None — Open-Ended |
| **Security Targets** | Financial • Governmental • Military • Post-Quantum |

---

> *"The system's security must depend only on the key being secret, not on the method being secret."*
>
> — Auguste Kerckhoffs, 1883

---

## 1. Preamble

This document constitutes a formal cryptographic challenge issued by Capomastro Holdings Ltd., Applied Physics Division. The challenger is invited to attack a ciphertext produced by the **T-AE-MAC** (Dual-Phase Authenticated Encryption with Message Authentication Code) construction, which is built entirely upon the **TLSponge-385** sponge primitive — the core cryptographic engine of the Salvi Framework.

This is not a puzzle. This is not a CTF exercise. This is a direct, open challenge to demonstrate that a novel post-quantum cryptographic construction can withstand adversarial scrutiny under conditions that meet or exceed the standards expected for financial, governmental, and military-grade encryption systems.

The challenge operates under **Kerckhoffs' principle**: the challenger receives the complete algorithm specification, access to the source code, and the ciphertext. The ***only*** secret is the key. If the system breaks when the attacker reads the code, it was never secure.

---

## 2. What You Receive

The following materials are provided to the challenger. Nothing is withheld except the encryption key and the original plaintext.

| # | Material | Description |
|---|---|---|
| 1 | **Ciphertext Blob** | The encrypted document, output of T-AE-MAC. Includes the ciphertext body, the authentication tag, the nonce, and the domain separator. Raw binary file. |
| 2 | **Algorithm Specification** | Complete description of TLSponge-385: 729-trit state (3⁶), rate 243 trits (3⁵), capacity 486 trits, 9 rounds, round function (theta diffusion, pi permutation, chi S-box, round constant injection), and all constants. |
| 3 | **T-AE-MAC Construction** | Full Dual-Phase AE construction: nonce handling, associated data processing, plaintext absorption via sponge duplex mode (absorb/squeeze), and tag generation. Reference: TM-2026-011 Rev. 1. |
| 4 | **Chi S-Box Specification** | The nonlinear layer uses an affine-composed power map: S(x) = M·x¹⁷ + c over GF(27), where M is a circulant [1,1,2] matrix (det = 1, branch number 3) and c = [1,0,2]. The complete 27-entry lookup table is supplied, along with M and c. Maximum differential probability DP_max = 1/9, confirmed by exhaustive computation. Algebraic degree = 5 (3-ary weight of 17 = 1+2+2). |
| 5 | **Permutation Formula** | Pi: π(i) = (376·i + 1) mod 729, where 376 is the stride and +1 is the offset. gcd(376, 729) = 1 guarantees a complete single-cycle permutation over all 729 state positions. |
| 6 | **Round Constants** | All 9 × 27 round constants provided in full. Derived from framework geometry, not arbitrary. |
| 7 | **Source Code Access** | Full repository access to the Rust implementation (tlsponge385.rs, sponge.rs, kernel crypto modules). The challenger may compile, instrument, and modify the code freely. |
| 8 | **Domain Separator** | The exact domain separation string used during encryption. This is not secret — it is a public parameter. |
| 9 | **Nonce** | The nonce used for this encryption. Provided in cleartext alongside the ciphertext. |
| 10 | **Algebraic Degree Saturation Analysis** | TM-2026-011 Rev. 1, §8.4: demonstrates that after two rounds the algebraic degree of the permutation saturates to the maximum of 6 (over GF(3)), making algebraic attacks infeasible beyond a few rounds. |

**What you do NOT receive:** the encryption key and the plaintext document. These are the only secrets. Recovery of either constitutes a break.

---

## 3. Challenge Tiers

The challenge is structured in four tiers of escalating severity. Each tier represents a distinct class of cryptographic attack. Success at any tier constitutes a meaningful result; higher tiers represent more severe breaks.

### Tier 1 — Distinguisher

**Objective:** Demonstrate that the T-AE-MAC ciphertext is statistically distinguishable from random bytes.

**Standard:** Produce a polynomial-time algorithm that, given a ciphertext of length N, determines whether it was produced by T-AE-MAC or drawn uniformly at random, with advantage significantly greater than 1/2. The distinguisher must succeed with probability measurably better than coin-flipping across multiple trials.

**Significance:** A distinguisher does not recover plaintext. It proves the construction leaks structural information. Even this alone would be a publishable result against a sponge-based AEAD.

**Compliance target:** IND-CPA security (indistinguishability under chosen-plaintext attack). Required by NIST SP 800-38A, FIPS 140-3, and all CNSA 2.0 symmetric primitives.

### Tier 2 — Forgery

**Objective:** Produce a valid (ciphertext, tag) pair for a message not encrypted by the issuer.

**Standard:** Without knowledge of the key, construct a new ciphertext C′ and authentication tag T′ such that the T-AE-MAC verification function accepts (C′, T′) as authentic. The forged message may be arbitrary — it need not be related to the challenge plaintext.

**Significance:** A forgery breaks authenticated encryption entirely. It means an attacker can inject messages that the recipient's system will accept as genuine. This is the attack that financial systems, military C2, and government communications must prevent above all else.

**Compliance target:** INT-CTXT security (integrity of ciphertext). Required by NIST SP 800-38D (GCM), all TLS 1.3 cipher suites, and NATO STANAG 4774 data-at-rest requirements.

### Tier 3 — Plaintext Recovery

**Objective:** Recover the original plaintext from the ciphertext without possessing the key.

**Standard:** Decrypt the challenge ciphertext — in whole or in part — producing content that matches the original plaintext. Partial recovery (e.g., first 128 bytes) counts as a partial break. Recovery must be achieved in time significantly less than brute-force exhaustive search of the keyspace.

**Significance:** Plaintext recovery means the encryption is broken. If this is achievable, the construction is unsuitable for any security-sensitive application.

**Compliance target:** IND-CCA2 security (indistinguishability under adaptive chosen-ciphertext attack). The gold standard for public-key and symmetric AEAD. Required by Common Criteria EAL5+ and NSA Suite B (deprecated) / CNSA 2.0 (current).

### Tier 4 — Key Recovery

**Objective:** Extract the encryption key from the ciphertext (and any other provided materials).

**Standard:** Recover the exact key used to produce the challenge ciphertext. With the key in hand, you must demonstrate decryption of the ciphertext to the correct plaintext. Any method is valid: algebraic, statistical, side-channel analysis of the source code, related-key attacks, or any novel technique.

**Significance:** Key recovery is a total break. It means the attacker can decrypt all past and future messages encrypted under the same key. This is the most severe class of cryptographic failure.

**Compliance target:** KR security (key recovery resistance). Implicit requirement of every standard from FIPS 140-3 to NSA CNSA 2.0 to PCI-DSS. Any system where key recovery is feasible is categorically failed.

---

## 4. Rules of Engagement

**4.1  No time limit.** This challenge is open-ended. Take a day, a week, a year. The construction either holds or it doesn't — time pressure is irrelevant to mathematical security.

**4.2  All tools permitted.** You may use any software, hardware, or computational resource. Custom scripts, SAT solvers, differential cryptanalysis suites, quantum simulators, GPU clusters, LLMs for code analysis — whatever you bring, bring it.

**4.3  Code access is unrestricted.** You have the full source repository. You may compile, decompile, instrument, fuzz, benchmark, profile, or rewrite the code in any language. You may insert breakpoints, trace state evolution, log intermediate values. The code is not a secret.

**4.4  No oracle access.** You will not be provided with an encryption or decryption oracle. You cannot submit plaintexts for encryption or ciphertexts for decryption. This is a ciphertext-only attack scenario (with full algorithm knowledge).

**4.5  Partial results are valued.** If you find a reduced-round distinguisher, a related-key weakness, a structural observation about the diffusion layer, or any property that would concern a cryptographer — report it. Partial results are how real cryptanalysis works.

**4.6  Reporting format.** Any successful or partial result should include: (a) which tier was targeted, (b) the methodology used, (c) reproducible steps or code, and (d) a complexity estimate (time, memory, data).

**4.7  Good faith.** The issuer will demonstrate decryption of the challenge ciphertext upon request, proving possession of the key. This is not a trick — there is a real key and a real plaintext behind this ciphertext.

---

## 5. Technical Context

The following is provided so the challenger understands what they are attacking. This is not a hint — it is an honest description of the system's design philosophy.

### 5.1  TLSponge-385

TLSponge-385 is a sponge construction operating over a ternary (base-3) algebraic field. Unlike binary sponge constructions (Keccak/SHA-3, ASCON), TLSponge-385 operates in GF(3ⁿ) and derives its permutation, S-box, and constants from the geometry of a 13-dimensional ternary hypercube.

**Sponge parameters:**

| Parameter | Value | Derivation |
|---|---|---|
| State width | 729 trits (3⁶) | Ternary cube dimension |
| Rate | 243 trits (3⁵) | Keystream throughput lane |
| Capacity | 486 trits (≈ 770 classical bits, ≈ 385-bit PQ) | Security margin |
| Rounds | 9 (3²) | 3× safety margin over 3-round full diffusion |
| Pi stride | 376, offset +1: π(i) = (376·i + 1) mod 729 | gcd(376, 729) = 1 — full single-cycle permutation |
| Theta neighbors | ±1, ±7, ±13 | All coprime to 729 — full diffusion in 3 rounds |

Key properties of the round function:

- **Theta (θ):** Linear diffusion layer — 7-neighbor mixing at distances ±1, ±7, ±13 across the 729-trit state. Branch number B(M_θ) = 8, proven via dual-space argument (TM-2026-008).
- **Pi (π):** Stride-376 permutation with offset +1: π(i) = (376·i + 1) mod 729. Single-cycle, coprime-guaranteed (gcd(376, 729) = 1). This is the TLSponge-385 permutation — distinct from the stride-13 permutation used in TIS-27.
- **Chi (χ):** Nonlinear substitution via affine-composed power map S(x) = M·x¹⁷ + c over GF(27), where M is a circulant [1,1,2] matrix and c = [1,0,2]. Algebraic degree 5 (3-ary weight of 17 = 122₃ → 1+2+2 = 5, which is 83% of the maximum 6). DP_max = LP_max = 1/9. The affine composition breaks monomial structure in the equation system, hardening against Gröbner basis and XL-style algebraic attacks (TM-2026-011, §8.4). Zero fixed point eliminated: S(0) = c ≠ 0.
- **Iota (ι):** Round constant addition — 27 GF(3) constants per round, breaking symmetry between rounds.

The automorphism group Aut(PlenumNET) ≅ (S₃)²⁵ × (C₂)².

**Algebraic Degree Saturation:** After just two rounds of the permutation, the algebraic degree of any output coordinate reaches the maximum value of 6 over GF(3). Any algebraic attack against the full 9-round permutation faces equations at maximum nonlinear complexity.

### 5.2  T-AE-MAC Construction

T-AE-MAC is a dual-phase authenticated encryption construction built on TLSponge-385, specified in formal security monograph TM-2026-011 Rev. 1. It provides:

- **Confidentiality:** IND-CPA and IND-CCA2 via sponge-duplex encryption
- **Integrity:** INT-CTXT via sponge-derived MAC tag
- **Authentication:** Bound associated data processed before plaintext absorption
- **Nonce-misuse resistance:** Domain separation prevents cross-context key reuse attacks

The construction follows the keyed-sponge duplex paradigm (Bertoni, Daemen, Peeters, Van Assche, 2011) adapted to ternary algebra. The adaptation introduces Salvi Framework–specific domain separation via 364° ternary-circle angle encoding and a dual-phase split where each plaintext half derives an independent keystream from a distinct domain input. This dual-phase structure provides partial compromise resistance not present in standard single-phase duplex constructions (see TM-2026-011, §5.3).

### 5.3  Post-Quantum Posture

TLSponge-385 is designed to resist quantum adversaries. The relevant quantum attack considerations:

- **Grover's algorithm:** Reduces brute-force key search from O(2ⁿ) to O(2ⁿ/²). The key length is sized to maintain ≥128-bit post-quantum security (256-bit key → 128-bit post-Grover).
- **Simon's algorithm:** Relevant to constructions with period structure. The sponge's nonlinear chi layer and round-dependent constants are designed to prevent exploitable periodicity.
- **BHT collision search:** Sponge capacity is sized to resist quantum collision finding (O(2ⁿ/³) for n-bit capacity). With 486-trit capacity (≈ 770 bits), BHT security is ≈ 257 bits.

---

## 6. What Constitutes a Win

| Tier | Victory Condition | Severity |
|---|---|---|
| **Tier 1: Distinguisher** | Polynomial-time algorithm with advantage > negligible | Theoretical concern — academic break |
| **Tier 2: Forgery** | Valid (C′, T′) pair accepted by verification | Critical — construction is unsafe for deployment |
| **Tier 3: Plaintext** | Any portion of the original plaintext recovered | Catastrophic — encryption is broken |
| **Tier 4: Key Recovery** | Exact key extracted; demonstrated by decryption | Total break — all security guarantees void |

**Reduced-round results** (e.g., "I can distinguish a 6-round variant") are meaningful and should be reported. Real-world cryptanalysis is iterative. Full-round breaks often begin with reduced-round observations.

---

## 7. What Does Not Constitute a Win

- Brute-forcing a weak or guessed key. The key is generated with sufficient entropy. If you guess it, you got lucky — not smart.
- Social engineering the key holder. This is a mathematical challenge, not a phishing exercise.
- Attacking the implementation's deployment environment rather than the cryptographic construction. Side-channel analysis of the algorithm is valid; breaking into the server is not.
- Claiming the system is "unproven" without demonstrating a specific weakness. Absence of NIST standardization is not a vulnerability — it's a fact of timeline. Show us the math.

---

## 8. Decryption Proof Protocol

Upon conclusion of the challenge (whether by the challenger's concession or after any claimed result is evaluated), the issuer will execute the following decryption proof:

**Step 1:** The issuer presents the key derivation parameters (domain separator, key material identifier, output length) — all of which have been recorded at encryption time and sealed.

**Step 2:** The issuer decrypts the challenge ciphertext in the presence of the challenger, producing the original plaintext.

**Step 3:** The plaintext is verified against a hash commitment made at the time of encryption. The hash commitment (TIS-27 digest of the plaintext, computed before encryption) is sealed in an envelope or digitally signed document delivered alongside the challenge materials, ensuring the issuer cannot substitute a different plaintext after the fact.

This protocol ensures mutual accountability: the challenger cannot claim the challenge was unfair, and the issuer cannot claim a break didn't happen.

---

## 9. Compliance Crosswalk

The following table maps the challenge tiers to the security properties required by major compliance frameworks. If T-AE-MAC survives all four tiers, it demonstrates properties consistent with these standards:

| Framework | Sector | Relevant Requirement | Tier Tested |
|---|---|---|---|
| **FIPS 140-3** | U.S. Government | Approved algorithm, KAT, side-channel | Tiers 1–4 |
| **NIST SP 800-38D** | General (GCM) | IND-CPA, INT-CTXT for AEAD | Tiers 1–2 |
| **CNSA 2.0** | NSA / DoD | Post-quantum symmetric at ≥256-bit | Tiers 1–4 |
| **Common Criteria EAL5+** | International | Formal security model, IND-CCA2 | Tier 3 |
| **PCI-DSS v4.0** | Financial / Payment | Strong encryption for cardholder data | Tiers 1–4 |
| **NATO STANAG 4774** | Military / Allied | Data-at-rest confidentiality + integrity | Tiers 2–3 |
| **SOC 2 Type II** | Enterprise / SaaS | Encryption controls for data protection | Tiers 1–3 |
| **HIPAA** | Healthcare | Encryption of PHI at rest and in transit | Tiers 1–3 |

> *Note: This challenge does not constitute formal certification under any of the above frameworks. It demonstrates the **cryptographic properties** those frameworks require. Formal certification (CMVP, CC, etc.) is a separate process involving accredited labs and paperwork.*

---

## 10. Closing Statement

We built this system in the open. The code is visible. The mathematics are documented. The construction has a formal security monograph. We are handing you the ciphertext, the algorithm, and the source code, and asking one simple question:

---

### **Can you break it?**

---

If you can, tell us how. We'll fix it, publish the vulnerability, and make the system stronger. If you can't, the construction speaks for itself.

Issued by the Applied Physics Division,
**Capomastro Holdings Ltd.**
Sherwood Park, Alberta, Canada

*Così sia.*

---

## Appendix A — Challenge Materials Checklist

The following items are to be delivered to the challenger. Check each before handoff:

| ☐ | Item | Format / Notes |
|---|---|---|
| ☐ | Ciphertext file | Binary blob (.tae extension) |
| ☐ | Nonce (cleartext) | Hex string or binary, alongside ciphertext |
| ☐ | Domain separator | UTF-8 string, documented |
| ☐ | Algorithm specification document | TLSponge-385 full spec (state 729, rate 243, capacity 486, 9 rounds) |
| ☐ | T-AE-MAC construction document | TM-2026-011 Rev. 1 monograph |
| ☐ | Chi S-box lookup table (composed) | 27-entry GF(27) table for S(x) = M·x¹⁷+c |
| ☐ | Affine constants M and c | Circulant [1,1,2] matrix (bn=3, det=1) and constant vector [1,0,2] |
| ☐ | Round constants (full set) | 9 × 27 array of GF(3) values |
| ☐ | Source code repository access | GitHub (SigmaWolf-8/Ternary) or local copy |
| ☐ | S-box verification script | docs/proofs/verify_affine_sbox.py |
| ☐ | Hash commitment of plaintext | TIS-27 digest, sealed envelope or signed file |
| ☐ | This Challenge Brief (CB-2026-001 Rev. 2) | This document |

---

## Appendix B — Issuer's Sealed Parameters

The following parameters are recorded by the issuer at encryption time and sealed (not shared with the challenger until the decryption proof). These ensure the issuer can demonstrate decryption and prove the challenge was conducted in good faith.

| Parameter | Value |
|---|---|
| Key derivation domain separator | [SEALED] |
| Key material identifier / seed | [SEALED] |
| Key output length (bits) | [SEALED] |
| TIS-27 hash of plaintext (commitment) | [SEALED] |
| SHA-256 hash of plaintext (backup commitment) | [SEALED] |
| Plaintext file size (bytes) | [SEALED] |
| Plaintext MIME type | [SEALED] |
| Date/time of encryption (UTC) | [SEALED] |
| HPTP timestamp (femtosecond, if available) | [SEALED] |

---

**CONFIDENTIAL — T-AE-MAC Cryptographic Challenge Brief — CB-2026-001 Rev. 2**
**© 2026 Capomastro Holdings Ltd. All Rights Reserved.**