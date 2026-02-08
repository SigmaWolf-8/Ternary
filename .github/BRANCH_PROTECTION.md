# Branch Protection Configuration

## Overview

This document describes the required GitHub branch protection rules for the `SigmaWolf-8/Ternary` repository. These settings enforce code review, automated testing, and compliance checks before code can be merged.

---

## Protected Branches

### `main` (Production)

| Setting | Value | Rationale |
|---------|-------|-----------|
| Require pull request reviews | Yes, 1 reviewer minimum | All changes reviewed before merge |
| Dismiss stale reviews | Yes | Force re-review after new commits |
| Require review from code owners | Yes | Domain experts approve changes |
| Require status checks to pass | Yes | Automated quality gates |
| Require branches to be up to date | Yes | Prevent merge conflicts |
| Require signed commits | Yes | Verify committer identity |
| Include administrators | Yes | No bypass for admins |
| Restrict who can push | Maintainers only | Prevent direct pushes |
| Allow force pushes | No | Preserve commit history |
| Allow deletions | No | Prevent branch deletion |

### `develop` (Integration)

| Setting | Value | Rationale |
|---------|-------|-----------|
| Require pull request reviews | Yes, 1 reviewer minimum | Review before integration |
| Dismiss stale reviews | Yes | Force re-review after new commits |
| Require status checks to pass | Yes | Automated quality gates |
| Require branches to be up to date | No | Allow parallel development |
| Require signed commits | Recommended | Identity verification |
| Allow force pushes | No | Preserve history |
| Allow deletions | No | Prevent branch deletion |

---

## Required Status Checks

The following CI/CD workflows must pass before merging to `main`:

### Tier 1 — Mandatory (Block Merge)

| Workflow | File | Check Name |
|----------|------|------------|
| Kernel Build | `build-kernel.yml` | `build-kernel` |
| Kernel Tests | `test-kernel.yml` | `test-kernel` |
| Security Scan | `security-scan.yml` | `security-scan` |
| CNSA 2.0 Compliance | `compliance-check.yml` | `compliance-check` |
| CodeQL Analysis | `codeql-analysis.yml` | `codeql-analysis` |

### Tier 2 — Required for Crypto Changes

| Workflow | File | Check Name | Trigger |
|----------|------|------------|---------|
| Timing Verification | `verify-timing.yml` | `verify-timing` | Changes to `src/kernel/src/timing/` |
| FPGA Build | `build-fpga.yml` | `build-fpga` | Changes to `hardware/fpga/` |

### Tier 3 — Advisory (Non-Blocking)

| Workflow | File | Check Name |
|----------|------|------------|
| Documentation Publish | `docs-publish.yml` | `docs-publish` |
| Docker Build | `docker-build.yml` | `docker-build` |

---

## CODEOWNERS

Create `.github/CODEOWNERS` with the following ownership rules:

```
# Global fallback
* @SigmaWolf-8

# Kernel crypto modules — require crypto team review
src/kernel/src/crypto/ @SigmaWolf-8

# Compliance and security
src/kernel/src/crypto/cnsa2.rs @SigmaWolf-8
.github/workflows/compliance-check.yml @SigmaWolf-8
.github/workflows/security-scan.yml @SigmaWolf-8

# CI/CD pipelines
.github/workflows/ @SigmaWolf-8

# Key management
keys/ @SigmaWolf-8

# API gateway configuration
kong/ @SigmaWolf-8
```

---

## Setup Instructions

### Via GitHub Web UI

1. Navigate to **Settings > Branches** in the repository
2. Click **Add branch protection rule**
3. Enter branch name pattern: `main`
4. Enable each setting listed in the table above
5. Under **Require status checks to pass before merging**:
   - Search for and add each Tier 1 check
   - Enable **Require branches to be up to date before merging**
6. Click **Create** to save
7. Repeat for `develop` branch with its settings

### Via GitHub CLI

```bash
gh api repos/SigmaWolf-8/Ternary/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["build-kernel","test-kernel","security-scan","compliance-check","codeql-analysis"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"dismissal_restrictions":{},"dismiss_stale_reviews":true,"require_code_owner_reviews":true,"required_approving_review_count":1}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false
```

---

## Commit Signing

All commits to protected branches must be signed. Contributors should configure GPG or SSH signing:

```bash
git config --global commit.gpgsign true
git config --global user.signingkey <KEY_ID>
```

See `keys/signing/SIGNING_PROCEDURES.md` for detailed key management procedures.

---

## Enforcement Timeline

| Milestone | Date | Action |
|-----------|------|--------|
| Enable `main` protection | Immediate | Apply Tier 1 checks |
| Enable `develop` protection | Immediate | Apply review requirements |
| Require signed commits | After key distribution | Enforce GPG/SSH signing |
| Add CODEOWNERS | Immediate | Create `.github/CODEOWNERS` file |

---

*Last Updated: February 2026*
