# Howey Test Analysis — PlenumNET Token/Service Classification

**Document ID:** COMP-FIN-001
**Version:** 1.0
**Date:** February 15, 2026
**Author:** Capomastro Holdings Ltd. Compliance Division
**Classification:** Internal — Legal Review Required

---

## 1. Purpose

This document applies the four-prong *SEC v. W.J. Howey Co.* (1946) test to
PlenumNET's digital offerings to determine whether any product, token, or
service offering may constitute an "investment contract" (and therefore a
"security") under US federal securities law (Securities Act of 1933, §2(a)(1)).

A parallel analysis under Canadian securities law (the *Pacific Coast Coin
Exchange* "investment contract" test) is included in §6.

---

## 2. Scope of Offerings Analyzed

| Offering | Type | Description |
|---|---|---|
| PlenumDB Compression Service | SaaS subscription | Ternary-encoded data compression |
| HPTP Timing API | Utility API | Femtosecond-precision timing service |
| Blockchain Witnessing | Infrastructure | HCS/XRPL/Algorand transaction anchoring |
| Phase Encryption Service | Utility API | Quantum-resistant encryption endpoint |
| PlenumNET API Gateway | Infrastructure | Kong-managed API routing |

---

## 3. Howey Test — Four Prongs

### 3.1 Investment of Money

**Question:** Does the purchaser invest money or other valuable consideration?

**Analysis:**
- PlenumDB and HPTP API customers pay subscription/usage fees for access to
  software services.
- Blockchain witnessing fees are paid per-transaction for infrastructure use.
- No token sale, pre-sale, ICO, or equity-like instrument is offered.
- Payments are denominated in fiat currency (CAD/USD) via Stripe and Interac.

**Finding:** Customers pay for service access, equivalent to SaaS subscription
fees. This prong is partially met (as with any paid service) but the commercial
nature weighs against securities classification.

### 3.2 Common Enterprise

**Question:** Is the investment part of a common enterprise where investors'
fortunes are tied together or to the promoter?

**Analysis:**
- Each customer receives independent service access. Customer A's compression
  results have no bearing on Customer B's outcomes.
- No pooling of customer funds occurs. Revenue from services funds operational
  costs, not shared investment pools.
- No profit-sharing, dividend, or revenue-distribution mechanism exists.
- Blockchain witnessing provides independent, per-transaction attestation.

**Finding:** No common enterprise exists. Each customer's use is independent.
This prong is **not met**.

### 3.3 Expectation of Profits

**Question:** Does the purchaser expect profits derived from the investment?

**Analysis:**
- Customers purchase PlenumNET services for their utility value: data
  compression, timing precision, encryption, and transaction witnessing.
- No marketing materials suggest profit generation, investment returns, or
  appreciation in value.
- No secondary market, exchange listing, or resale mechanism exists for
  service credits or tokens.
- Service pricing is fixed or usage-based; no speculative pricing model.

**Finding:** Customers expect utility, not profits. This prong is **not met**.

### 3.4 Efforts of Others

**Question:** Are profits (if any) derived solely from the efforts of the
promoter or a third party?

**Analysis:**
- Service quality depends on PlenumNET's infrastructure, but customers derive
  operational utility, not investment returns.
- Customers actively use APIs, integrate services, and make independent
  business decisions about how to deploy PlenumNET services.
- No passive investment model exists.

**Finding:** Customers are active users, not passive investors. This prong is
**not met** for securities purposes.

---

## 4. Howey Test Conclusion

| Prong | Met? | Notes |
|---|---|---|
| Investment of Money | Partial | SaaS fees, not investment capital |
| Common Enterprise | No | Independent customer relationships |
| Expectation of Profits | No | Utility-driven, no profit expectation |
| Efforts of Others | No | Active service consumption |

**Overall Determination:** PlenumNET's current offerings **do not constitute
securities** under the Howey test. All services are utility-based SaaS/API
offerings with no investment contract characteristics.

---

## 5. Risk Factors and Mitigation

| Risk | Mitigation |
|---|---|
| Future token issuance could trigger securities classification | Any token offering must undergo separate Howey analysis and potential SEC/CSA registration |
| Marketing language suggesting "returns" or "appreciation" | All marketing reviewed by compliance; utility-focused messaging only |
| Bundled offerings that create pooled economics | Service isolation maintained; no revenue sharing across customers |
| Staking or yield mechanisms on blockchain services | No staking/yield features implemented; any future addition requires legal review |

---

## 6. Canadian Securities Law — Pacific Coast Coin Test

Under *Pacific Coast Coin Exchange v. Ontario Securities Commission* (1978),
the Canadian test for "investment contract" requires:

1. An investment of money
2. In a common enterprise
3. With the expectation of profit
4. To come significantly from the efforts of others

**Analysis:** The same factors that negate Howey classification apply under
Canadian law. PlenumNET's SaaS model provides functional utility without
creating investment contracts under Canadian provincial securities legislation.

**CSA Staff Notice 46-307** (Cryptocurrency Offerings): PlenumNET does not
issue cryptocurrency tokens. Blockchain witnessing is an infrastructure
service, not a token offering.

---

## 7. Regulatory Filing Status

| Jurisdiction | Status | Notes |
|---|---|---|
| SEC (US) | Not required | No securities offered |
| CSA (Canada) | Not required | No securities offered |
| FinCEN (US) | Not applicable | Not a money services business |
| FINTRAC (Canada) | Under review | Blockchain witnessing may trigger MSB registration |

---

## 8. Review Schedule

- **Quarterly:** Marketing materials review for securities-suggestive language
- **Before launch:** Any new product/token offering requires fresh Howey analysis
- **Annually:** Full document review and regulatory landscape update
- **Ad hoc:** Upon receipt of regulatory inquiry or enforcement action

---

*This analysis is for internal compliance purposes and does not constitute
legal advice. Consult qualified securities counsel before making regulatory
filing decisions.*
