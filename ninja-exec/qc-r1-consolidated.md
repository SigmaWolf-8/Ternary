# QC-R1 Consolidated Review — NinjaExec (Task #54)

**Product:** NinjaExec — PlenumNET Local Signing Agent v1.0.0
**Review Date:** 2026-03-28
**Protocol:** QC-R1 (Round 1 — Technical Verification)
**Reviewers:**
- Agent 1: Security Engineer
- Agent 2: DevOps Automator
- Agent 3: PlenumNET Integration Specialist

---

## Overall Verdict: **FAIL**

All three reviewers independently returned **FAIL**. NinjaExec has strong foundations — TL-DSA-87 correctly selected, all crypto delegated to `ternary_math`, no external crypto dependencies, localhost binding, constant-time tag comparison, volatile key zeroization — but systemic INVARIANT 9 violations and a wide-open CORS policy block release.

---

## CRITICAL Findings (Blocking)

### C1 — No Rep C Address Binding in TL-DSA Signatures (INVARIANT 9)
**Flagged by:** A1 (R1-A1-1), A2 (R1-A2-4), A3 (R1-A3-1)
**Location:** `signing_engine.rs` lines 13–18
**Issue:** `tl_dsa::sign(secret_key, payload, VARIANT)` passes raw payload with no Rep C address in the context string. INVARIANT 9 requires the signer's Rep C address to be bound into every signature context. Signatures are identity-unbound and could be replayed across operators.
**Resolution:** Extend `sign()` / `verify()` to accept a Rep C address parameter. Construct domain-separated message: `"NinjaExec-SIGN-v1.0" ‖ rep_c_address ‖ context ‖ payload`. Update all call sites.

### C2 — No Rep C Address Exists Anywhere in Codebase (INVARIANT 9)
**Flagged by:** A1 (R1-A1-15), A2 (R1-A2-3), A3 (R1-A3-6)
**Location:** Entire codebase
**Issue:** No Rep C address is stored, derived, or referenced in any source file. The keystore stores only a raw keypair. `export-operator` identifies nodes by `operator@{hostname}` — explicitly prohibited by INVARIANT 9. Audit entries use HTTP origin URLs instead of Rep C addresses.
**Resolution:** During `init`, derive or accept a Rep C 54-trit address. Store in keystore. Bind into all signing contexts, KDF domain separators, audit entries, and operator export output. Replace hostname-based identification.

### C3 — CORS Wildcard Origin Creates Signature Oracle
**Flagged by:** A1 (R1-A1-4), A2 (R1-A2-5), A3 (R1-A3-7)
**Location:** `server.rs` lines 615–618
**Issue:** `allow_origin(Any)` permits any website in the operator's browser to send cross-origin requests to `127.0.0.1:21027/sign`. Combined with headless mode auto-approval (C4-adjacent), this is a remotely exploitable signature oracle.
**Resolution:** Replace with configurable origin allowlist in `ninja-exec.json`. Default to deny all cross-origin. At minimum restrict to known YODA dashboard origins and `localhost`.

### C4 — Audit Log Silently Swallows All Write Failures
**Flagged by:** A2 (R1-A2-1), A1 (R1-A1-13)
**Location:** `audit.rs` lines 38–51, `server.rs` passim
**Issue:** `AuditLog::append()` silently ignores JSON serialization errors, directory creation errors, file open errors, and write errors. A signing operation can succeed and return a valid signature while the audit trail silently fails. This is a "silent failure producing an unaudited artifact."
**Resolution:** `append()` must return `Result`. Sign operations must fail-closed if audit write fails.

---

## IMPORTANT Findings (Pre-Release)

| ID | Finding | Agents | Key Detail |
|----|---------|--------|------------|
| I1 | Weak KDF iterations (4096) | A1 (R1-A1-3) | Target ≥100ms wall-clock; current ~10ms. Consider 100K+ or Argon2id |
| I2 | Passphrase via env var exposure | A1 (R1-A1-5), A2 (R1-A2-11) | `/proc/<pid>/environ` visible; document as CI-only, zero after use |
| I3 | Non-constant-time token comparison | A1 (R1-A1-6) | `check_confirm_token` uses `!=`; use XOR-accumulate or `subtle` crate |
| I4 | Headless mode auto-approves destructive ops | A1 (R1-A1-7), A2 (R1-A2-6) | `exec`, `deploy`, `key-rotation` all auto-approved; add `headless_allow` list |
| I5 | Bespoke authenticated encryption (not T-AE-MAC) | A1 (R1-A1-8), A3 (R1-A3-4) | Custom XOR-stream + sponge-tag; should use T-AE-MAC if available |
| I6 | Placeholder upgrade code | A1 (R1-A1-9) | `A1B2C3D4-E5F6-...` is hand-typed; derive deterministically with ≥2^-64 collision bound |
| I7 | Confirm token printed to stdout | A1 (R1-A1-10), A2 (R1-A2-8) | Exposed in logs/CI; print storage location only, set 0600 on config |
| I8 | Dependencies not patch-pinned | A2 (R1-A2-2) | `tokio = "1"` etc.; pin to patch versions for reproducible builds |
| I9 | No CI/CD pipeline | A2 (R1-A2-7) | No GitHub Actions workflow; need matrix build for aarch64 + x86_64 |
| I10 | No deployment test specification | A2 (R1-A2-12) | No integration test harness for HTTP API end-to-end |
| I11 | Unregistered context strings | A3 (R1-A3-2,3,4,5) | 5 context strings not in canonical registry: NinjaExec-FP, -KDF-v2, -KS-STREAM, -KS-TAG, -AUDIT-HASH |
| I12 | No operation context in signed message | A3 (R1-A3-8) | Signatures don't bind operation type; cross-context replay possible |

---

## MINOR Findings

| ID | Finding | Agents |
|----|---------|--------|
| M1 | Custom zeroize without compiler fence | A1 (R1-A1-11) |
| M2 | Shared rate limiter across endpoints | A1 (R1-A1-12) |
| M3 | Config file permissions not set (0600) | A1 (R1-A1-14) |
| M4 | Windows-only binary naming in cross-platform config | A2 (R1-A2-9) |
| M5 | Misleading `tis27:` hash prefix (actual = TLSponge-385) | A2 (R1-A2-10), A3 (R1-A3-5) |
| M6 | Undocumented port 21027 rationale | A3 (R1-A3-9) |

---

## Cryptographic Claims Summary

| Claim | Verdict | Notes |
|-------|---------|-------|
| TL-DSA-87 for all signatures | **VERIFIED** | All 3 agents confirmed |
| No external crypto dependencies | **VERIFIED** | No Ed25519, AES, SHA, BLAKE crates |
| TLSponge-385 for KDF/MAC/hash | **VERIFIED** | `ternary_math::sponge::derive_key` throughout |
| Constant-time tag comparison (keystore) | **VERIFIED** | XOR-accumulate pattern |
| Key zeroization on Drop | **VERIFIED** | `write_volatile` in `Drop` impl |
| Rep C address binding (INVARIANT 9) | **INCORRECT** | No Rep C anywhere in codebase |
| Constant-time token comparison (server) | **INCORRECT** | Standard `!=` used |
| Upgrade code deterministically derived | **INCORRECT** | Hardcoded placeholder |
| Context strings in canonical registry | **UNVERIFIED** | 5 strings unregistered |
| Level 5 post-quantum security | **VERIFIED** | TL-DSA-87 = NIST PQ Level 5 |

---

## Passphrase Entropy Assessment

**Minimum:** 72 bits effective
**Rationale (Agent 1):** 12-character minimum with KDF_ITERATIONS=4096 provides ~12 bits work factor. Human-chosen passphrases at 12 chars yield ~30-78 bits depending on composition. To reach 72 bits effective: either increase KDF to 2^20 (requiring ~52 bits passphrase entropy) or enforce 16+ chars with mixed composition, or recommend Diceware generation.

---

## Resolution Priority

1. **Rep C address provisioning + signature context binding** (C1, C2) — architectural change
2. **CORS origin restriction** (C3) — configuration change
3. **Audit fail-closed** (C4) — refactor `append()` to `Result`
4. **Headless mode restriction** (I4) — add `headless_allow` config
5. **Constant-time token comparison** (I3) — straightforward fix
6. **Token not printed to stdout** (I7) — quick fix
7. **Context string registration** (I11) — documentation
8. **Operation context in signatures** (I12) — extends C1 fix
9. **KDF iteration increase** (I1) — benchmark-driven tuning
10. **Remaining IMPORTANT + MINOR** — incremental

---

## Individual Agent Reports

- [Agent 1 — Security Engineer](qc-r1-agent1-security-engineer.md)
- [Agent 2 — DevOps Automator](qc-r1-agent2-devops-automator.md)
- [Agent 3 — PlenumNET Integration Specialist](qc-r1-agent3-plenumnet-integration.md)
