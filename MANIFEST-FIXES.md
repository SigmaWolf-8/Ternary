# Required Repository Fixes — Manifest
## Salvi Framework / PlenumNET
### Priority Actions for Licensing Compliance

---

## PRIORITY 1 — CRITICAL (Execute Immediately)

### Fix 1.1: Root package.json — Remove MIT License Declaration

**File:** `package.json` (root)  
**Current:** `"license": "MIT"`  
**Change to:** `"license": "SEE LICENSE IN LICENSE"`  
**Reason:** The root package.json currently offers the entire web application under MIT, contradicting proprietary intent.

**STATUS: DONE**

### Fix 1.2: THDL Cargo.toml — Remove MIT License Declaration

**File:** `src/thdl/Cargo.toml`  
**Current:** `license = "MIT"`  
**Change to:** `license = "LicenseRef-Proprietary"`  
**Reason:** The Ternary Hardware Description Language is core proprietary IP being inadvertently offered as MIT.

**STATUS: DONE**

### Fix 1.3: TSL Cargo.toml — Remove MIT License Declaration

**File:** `src/tsl/Cargo.toml`  
**Current:** `license = "MIT"`  
**Change to:** `license = "LicenseRef-Proprietary"`  
**Reason:** The Ternary System Language is core proprietary IP being inadvertently offered as MIT.

**STATUS: DONE**

### Fix 1.4: Add Root LICENSE File

**Action:** Copy the provided `LICENSE` file to the repository root.  
**Reason:** No root license file currently exists. This is the single most important file for IP protection.

**STATUS: AWAITING FILE FROM USER**

### Fix 1.5: Add NOTICE File

**Action:** Copy the provided `NOTICE` file to the repository root.  
**Reason:** Third-party attribution compliance for MIT/Apache-2.0 dependencies.

**STATUS: AWAITING FILE FROM USER**

### Fix 1.6: Kernel Cargo.toml — Remove MIT License Declaration

**File:** `src/kernel/Cargo.toml`  
**Current:** `license = "MIT"`  
**Change to:** `license = "LicenseRef-Proprietary"`  
**Reason:** The main kernel crate (47,230 lines) was inadvertently offered under MIT — the most valuable component of the entire IP portfolio.

**STATUS: DONE**

### Fix 1.7: WASM Cargo.toml — Remove MIT License Declaration

**File:** `src/kernel/wasm/Cargo.toml`  
**Current:** `license = "MIT"`  
**Change to:** `license = "LicenseRef-Proprietary"`  
**Reason:** The WASM interface crate was inadvertently offered under MIT.

**STATUS: DONE**

---

## PRIORITY 2 — HIGH (Execute Within 7 Days)

### Fix 2.1: Add CLA to Repository

**Action:** Copy `CLA.md` to repository root.  
**Additional:** Add CLA requirement to `CONTRIBUTING.md` with the following text at the top of the "Submitting Changes" section:

```markdown
### Contributor License Agreement

Before we can accept any contributions, you must sign and submit the
Contributor License Agreement (CLA). See [CLA.md](CLA.md) for the full
agreement. No pull requests will be merged without a signed CLA on file.
```

**Optional:** Implement CLA-bot automation via GitHub Actions (e.g., `cla-assistant/github-action`).

**STATUS: AWAITING FILE FROM USER**

### Fix 2.2: Update CONTRIBUTING.md IP Section

**Action:** Add the following section after the "Table of Contents" in `CONTRIBUTING.md`:

```markdown
## Intellectual Property Notice

All contributions to this project become the joint property of Capomastro
Holdings Ltd. under the terms of our Contributor License Agreement (CLA).
By submitting a pull request, you represent that you have the right to
grant the licenses described in the CLA and that you have read and agree
to its terms.

The Salvi Framework, PlenumNET, and all associated technology are the
exclusive intellectual property of Capomastro Holdings Ltd. See the
LICENSE file in the repository root for full terms.
```

**STATUS: PENDING**

### Fix 2.3: Add License Header CI Check

**Action:** Create `.github/workflows/license-check.yml`:

```yaml
name: License Header Check
on: [pull_request]
jobs:
  check-headers:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check copyright headers
        run: |
          MISSING=$(find src/ server/ client/src/ libternary/src/ contracts/ \
            services/ shared/ -type f \
            \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.py" \) \
            -exec grep -L "Capomastro Holdings" {} \;)
          if [ -n "$MISSING" ]; then
            echo "Files missing copyright header:"
            echo "$MISSING"
            exit 1
          fi
```

**STATUS: PENDING**

### Fix 2.4: Deploy SaaS Legal Documents

**Action:** Add the following files to the repository and link them from the PlenumNET web application footer:

- `TERMS-OF-SERVICE.md` → accessible at `/terms`
- `ACCEPTABLE-USE-POLICY.md` → accessible at `/aup`
- `PRIVACY-POLICY.md` → accessible at `/privacy`

**Implementation Note:** These should be served as rendered pages within the React client application, with links in the site footer and any registration/onboarding flow.

**STATUS: AWAITING FILES FROM USER**

---

## PRIORITY 3 — MEDIUM (Execute Within 30 Days)

### Fix 3.1: Apply Source File License Headers

**Action:** Apply the appropriate copyright header from `FILE-HEADER-TEMPLATE.md` to all source files.

**Scope (estimated file count):**
- `src/kernel/src/**/*.rs` — ~50+ files
- `src/thdl/src/**/*.rs` — ~10+ files
- `src/tsl/src/**/*.rs` — ~10+ files
- `ternary-math/src/**/*.rs` — ~10+ files
- `libternary/src/**/*.ts` — ~15+ files
- `server/**/*.ts` — ~15+ files
- `client/src/**/*.tsx` — ~30+ files
- `shared/**/*.ts` — ~3 files
- `contracts/**/*.py` — ~5+ files
- `scripts/**/*.sh` — ~5 files
- `kong/Dockerfile` — 1 file

**STATUS: PENDING**

### Fix 3.2: Update libternary/LICENSE Copyright Year

**File:** `libternary/LICENSE`  
**Current:** `Copyright © Capomastro Holdings Ltd 2026`  
**Change to:** `Copyright © 2025-2026 Capomastro Holdings Ltd.`  
**Reason:** Align year range with actual development timeline and root LICENSE.

**STATUS: PENDING**

### Fix 3.3: Add License Field to oracle-bridge package.json

**File:** `contracts/oracle-bridge/package.json`  
**Action:** Verify and add `"license": "SEE LICENSE IN LICENSE"` if not present.

**STATUS: PENDING**

### Fix 3.4: Root package.json Name Cleanup

**File:** `package.json` (root)  
**Current:** `"name": "rest-express"`  
**Consider changing to:** `"name": "plenumnet"` or `"name": "@capomastro/plenumnet"`  
**Reason:** The name "rest-express" appears to be a scaffold artifact and doesn't reflect the actual project. While not a licensing issue per se, it affects discoverability and professional presentation.

**STATUS: PENDING**

---

## PRIORITY 4 — RECOMMENDED (Best Practice)

### Fix 4.1: Add .github/SECURITY.md

**STATUS: PENDING**

### Fix 4.2: Add GitHub Repository Settings

- Enable branch protection on `main` requiring PR reviews
- Enable Dependabot alerts for dependency vulnerabilities
- Consider enabling private vulnerability reporting via GitHub Security Advisories

**STATUS: PENDING (Manual GitHub Settings)**

### Fix 4.3: Lock Rust Dependencies

Run `cargo generate-lockfile` in each Rust workspace to create `Cargo.lock` files.

**STATUS: PENDING**

---

## Execution Summary

| Priority | Fixes | Estimated Effort | Deadline |
|----------|-------|-----------------|----------|
| P1 — Critical | 5 fixes | 30 minutes | Immediate |
| P2 — High | 4 fixes | 2-3 hours | 7 days |
| P3 — Medium | 4 fixes | 4-6 hours | 30 days |
| P4 — Recommended | 3 fixes | 1-2 hours | Ongoing |

**Total estimated remediation effort: 1-2 business days.**
