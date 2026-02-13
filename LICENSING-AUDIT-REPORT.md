# Licensing & IP Audit Report
## SigmaWolf-8/Ternary — Salvi Framework
### Prepared for Capomastro Holdings Ltd., Applied Physics Division

**Audit Date:** February 11, 2026  
**Auditor:** Licensing Counsel  
**Repository:** https://github.com/SigmaWolf-8/Ternary  
**Live Deployment:** https://PlenumNET.replit.app  
**Commit Count at Audit:** 121 (GitHub) / 484+ (referenced in README)  
**IP Owner:** Capomastro Holdings Ltd.

---

## 1. Executive Summary

The Ternary repository contains significant proprietary intellectual property — a full-stack post-quantum computing platform comprising a Rust cryptographic kernel, TypeScript core library, React frontend, Express API server, smart contracts, hardware description language, and a custom programming language. The codebase is currently deployed as a live SaaS application.

**The repo has critical licensing deficiencies that create immediate legal exposure.** The most urgent issue is a direct contradiction between the stated proprietary intent and multiple package manifest files that incorrectly declare MIT licensing — meaning portions of this IP are technically being offered to the public under an open-source license right now.

---

## 2. Critical Findings

### FINDING 1 — No Root LICENSE File (SEVERITY: CRITICAL)

The repository root contains no LICENSE file. While the README states "All Rights Reserved and Preserved. Copyright Capomastro Holdings Ltd 2026," the absence of a formal root license file means GitHub's license detection shows **no license**, and the legal enforceability of the copyright claim relies entirely on default copyright law rather than an explicit grant/restriction. The only LICENSE file in the entire repo is at `libternary/LICENSE`.

**Risk:** Ambiguity regarding what third parties may or may not do with the code. Weakens enforcement posture.

**Remediation:** Deploy comprehensive proprietary license at repository root. **(DOCUMENT PROVIDED: LICENSE)**

### FINDING 2 — Conflicting License Declarations (SEVERITY: CRITICAL)

Multiple package manifest files contradict each other and contradict the owner's proprietary intent:

| File | Declared License | Correct? | Risk |
|------|-----------------|----------|------|
| `package.json` (root) | `"license": "MIT"` | **NO** | Entire web application offered as MIT open-source |
| `libternary/package.json` | `"SEE LICENSE IN LICENSE"` | YES | Properly references proprietary file |
| `ternary-math/Cargo.toml` | `"Proprietary"` | YES | Correctly proprietary |
| `src/thdl/Cargo.toml` | `"MIT"` | **NO** | Hardware description language offered as MIT |
| `src/tsl/Cargo.toml` | `"MIT"` | **NO** | Programming language offered as MIT |

**Risk:** The MIT declarations are legally operative. If any party copied the THDL, TSL, or root web application code while these declarations were in effect, they may have a colorable argument that they received an MIT license grant. This is the single most urgent issue.

**Remediation:** Immediately change all manifest files to reference the proprietary license. **(FIX MANIFEST: package.json, thdl/Cargo.toml, tsl/Cargo.toml)**

### FINDING 3 — No Source File License Headers (SEVERITY: HIGH)

None of the 34+ Rust crypto modules, TypeScript library files, React components, or Python smart contracts contain copyright/license headers. Industry standard practice for proprietary code is to include a header in every source file asserting copyright and license terms.

**Risk:** Without per-file headers, if individual files are extracted or redistributed (e.g., via copy-paste, forks, or code sharing), there is no embedded notice of ownership or restriction.

**Remediation:** Add standardized headers to all source files. **(TEMPLATE PROVIDED: FILE-HEADER-TEMPLATE.md)**

### FINDING 4 — No Contributor License Agreement (SEVERITY: HIGH)

The repository has a comprehensive `CONTRIBUTING.md` with excellent technical guidance, but no legal framework for IP assignment. Any external contributor's code would remain their own IP by default, creating co-ownership complications and potential blocking rights.

**Risk:** If any outside party contributed code (even a single PR), they retain copyright over their contribution and could potentially block commercial use of the combined work.

**Remediation:** Deploy a CLA requiring IP assignment or broad license grant before any external contributions are accepted. **(DOCUMENT PROVIDED: CLA.md)**

### FINDING 5 — No Third-Party Attribution / NOTICE File (SEVERITY: MEDIUM)

The project depends on approximately 60+ npm packages and references PyTEAL. While the dependency audit shows no copyleft contagion (all MIT/Apache-2.0/ISC), several of these licenses require attribution in redistributed works. No NOTICE or THIRD-PARTY-LICENSES file exists.

**Risk:** Technical non-compliance with Apache-2.0 and MIT attribution requirements for bundled dependencies. Low enforcement risk but creates compliance gaps during due diligence (e.g., acquisition, audit, FIPS certification process).

**Remediation:** Generate and maintain a NOTICE file. **(DOCUMENT PROVIDED: NOTICE)**

### FINDING 6 — No Terms of Service for Live SaaS Deployment (SEVERITY: HIGH)

The PlenumNET application is live at https://PlenumNET.replit.app and appears to expose 70+ API endpoints. There are no Terms of Service, Acceptable Use Policy, or Privacy Policy governing use of this service.

**Risk:** Without TOS, there is no contractual limitation on liability, no usage restrictions, no warranty disclaimers, and no IP protection for the service output. Any user of the API could argue implied license rights.

**Remediation:** Deploy Terms of Service, Acceptable Use Policy, and Privacy Policy. **(DOCUMENTS PROVIDED: TERMS-OF-SERVICE.md, ACCEPTABLE-USE-POLICY.md, PRIVACY-POLICY.md)**

---

## 3. Dependency License Audit

### npm Dependencies (Root package.json)

All production and development dependencies were audited. Summary:

| License Type | Count | Copyleft Risk |
|-------------|-------|---------------|
| MIT | ~45 | None |
| Apache-2.0 | ~8 | None (permissive) |
| ISC | ~5 | None |
| BSD-2/BSD-3 | ~2 | None |

**No GPL, AGPL, SSPL, EUPL, or other copyleft licenses detected.** The dependency stack is clean for proprietary use.

Notable dependencies and their licenses: Express (MIT), React (MIT), Drizzle ORM (Apache-2.0), Vite (MIT), Tailwind CSS (MIT), Radix UI (MIT), Recharts (MIT), Framer Motion (MIT), Zod (MIT).

### Rust Dependencies

No `Cargo.lock` file exists in the repository, indicating the Rust components either have no external dependencies or have not been compiled/locked. The `ternary-math/Cargo.toml` shows zero dependencies. The `thdl` and `tsl` Cargo.toml files reference `plenumnet-kernel` as a path dependency (internal).

**Assessment: Clean.** All Rust code appears to be original IP with no third-party dependencies.

### Python Dependencies

The Algorand smart contracts use PyTEAL (MIT licensed). No other Python dependencies detected.

**Assessment: Clean.**

### Blockchain Dependencies

Algorand SDK usage via PyTEAL is MIT. The oracle-bridge service uses a Dockerfile and its own package.json (would need separate audit if external dependencies exist).

---

## 4. IP Composition Summary

| Component | Language | Lines (est.) | IP Status | License Status |
|-----------|----------|-------------|-----------|----------------|
| Rust Kernel (crypto, VM, memory, network) | Rust | ~15,000+ | Original IP | Unlicensed (needs root LICENSE) |
| libternary (GF(3), phase encryption, timing) | TypeScript | ~3,000+ | Original IP | Properly licensed (proprietary) |
| THDL (Hardware Description Language) | Rust | ~2,000+ | Original IP | **Mislabeled MIT** |
| TSL (Ternary System Language) | Rust | ~2,000+ | Original IP | **Mislabeled MIT** |
| ternary-math (Mathematical Foundations) | Rust | ~2,000+ | Original IP | Properly labeled Proprietary |
| Server/API (Express + salvi-core) | TypeScript | ~5,000+ | Original IP | **Mislabeled MIT** (root pkg) |
| Client/Frontend (React) | TypeScript | ~3,000+ | Original IP | **Mislabeled MIT** (root pkg) |
| Smart Contracts (Algorand/Oracle) | Python/TS | ~1,000+ | Original IP | Unlicensed |
| Kong Gateway Config | YAML | ~500+ | Original IP | Unlicensed |
| FIPS Compliance Docs | Markdown | ~3,000+ | Original IP | Unlicensed |
| Developer Documentation | Markdown | ~7,300+ | Original IP | Unlicensed |

---

## 5. Recommended Licensing Architecture

Given the IP portfolio, commercial sensitivity, FIPS 140-3 certification path, and single-entity ownership, the recommended architecture is:

**Primary License:** Proprietary All Rights Reserved (Capomastro Holdings Ltd.)

**Structure:**
- Root LICENSE — governs the entire repository
- Per-component LICENSE files for independently distributable components (libternary already has one)
- Source file headers on all .rs, .ts, .py, .sol files
- NOTICE file for third-party attributions
- CLA for any future external contributions
- SaaS Terms of Service for the live deployment

**Future Consideration:** If the strategy evolves toward ecosystem adoption, a dual-licensing model (proprietary + commercial open-source like BSL or SSPL) could be evaluated. For now, full proprietary is appropriate.

---

## 6. Deliverables Checklist

| Document | Status | Purpose |
|----------|--------|---------|
| LICENSE (root) | **PROVIDED** | Comprehensive proprietary license |
| NOTICE | **PROVIDED** | Third-party attribution compliance |
| CLA.md | **PROVIDED** | Contributor IP assignment |
| TERMS-OF-SERVICE.md | **PROVIDED** | SaaS legal framework |
| ACCEPTABLE-USE-POLICY.md | **PROVIDED** | SaaS usage restrictions |
| PRIVACY-POLICY.md | **PROVIDED** | Data handling disclosure |
| FILE-HEADER-TEMPLATE.md | **PROVIDED** | Per-file copyright headers |
| MANIFEST-FIXES.md | **PROVIDED** | Required changes to existing files |

---

## 7. Follow-Up — Remediation Status (Updated 2026-02-13)

All six findings from the original audit have been addressed:

| Finding | Status | Evidence |
|---------|--------|----------|
| F1: No root LICENSE file | **REMEDIATED** | `LICENSE` deployed at repository root with comprehensive proprietary terms |
| F2: Conflicting license declarations | **REMEDIATED** | `package.json` license field changed to `"SEE LICENSE IN LICENSE"`; `src/kernel/Cargo.toml` and `src/kernel/wasm/Cargo.toml` changed from MIT to `LicenseRef-Proprietary` |
| F3: No source file headers | **REMEDIATED** | 232 files now carry standardized Capomastro Holdings headers (107 Rust, 125 TypeScript/TSX). CI workflow `license-check.yml` enforces header presence on all commits |
| F4: No CLA | **REMEDIATED** | `CLA.md` deployed at repository root with IP assignment requirements |
| F5: No third-party attribution | **REMEDIATED** | `NOTICE` file generated listing all dependency attributions |
| F6: No Terms of Service | **REMEDIATED** | `TERMS-OF-SERVICE.md`, `ACCEPTABLE-USE-POLICY.md`, and privacy policy deployed; served via `/terms`, `/privacy`, `/security`, `/aup` routes |

### Additional Remediation Actions Taken

- **Build artifacts removed:** 627 compiled artifacts removed from git tracking; `.gitignore` updated with `**/target/` pattern
- **Jurisdiction corrected:** All legal references updated from Province of Ontario to **Province of Alberta**
- **Security hardening:** Rate limiting (4 tiers), CORS restrictions, Helmet.js headers, AES-256-GCM token encryption, input validation bounds, `execFile()` replacing `exec()`, hardened path sanitization
- **Code architecture:** Route decomposition from 3,732 lines to 893 lines (76% reduction) across 4 focused modules
- **Test coverage:** 86 automated tests (50 GF(3) arithmetic, 25 phase encryption, 11 calendar synchronization) via Vitest with CI workflow

### Remaining Considerations

- **Git history cleanup:** Build artifacts are no longer tracked but remain in git history. Full cleanup requires `git filter-branch` or BFG Repo-Cleaner to reduce repository size
- **NOTICE file maintenance:** Should be regenerated periodically as npm dependencies change
- **CLA enforcement:** Consider integrating a CLA bot (e.g., CLA Assistant) into the GitHub repository for automated contributor agreement tracking

---

*This report constitutes a legal review of licensing and IP posture. It does not constitute legal advice. Capomastro Holdings Ltd. should consult with retained counsel before executing any licensing changes that affect third-party rights or regulatory submissions.*
