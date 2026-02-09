# Vendor Evidence Package Index
## Salvi Ternary Cryptographic Module v3.0.0
## Capomastro Holdings Ltd. | Applied Physics Division

---

## Package Contents

22 vendor evidence documents organized for CSTL submission.

| ID | Document | Location | FIPS Requirement |
|---|---|---|---|
| VE-001 | Non-Proprietary Security Policy | `docs/compliance/security-policy.md` | SP 800-140B |
| VE-002 | Cryptographic Boundary Diagram | `docs/compliance/boundary-diagram.md` | ISO 19790 §7.2 |
| VE-003 | Finite State Model | `docs/compliance/finite-state-model.md` | ISO 19790 §7.2 |
| VE-004 | SSP Inventory | `docs/compliance/ssp-inventory.md` | ISO 19790 §7.9 |
| VE-005 | Operational Environments | `docs/compliance/operational-environments.md` | ISO 19790 §7.6 |
| VE-006 | Entropy Source Assessment | `docs/compliance/entropy-assessment.md` | SP 800-90B |
| VE-007 | CAVP Algorithm Certificates | Output of `cavp_certs.rs` | SP 800-140C |
| VE-008 | ACVTS Registration Files | Output of `acvts.rs` `generate_all_registrations()` | NIST ACVTS |
| VE-009 | ACVTS Response Files | Output of `acvts.rs` `generate_sha2_test_response()` et al. | NIST ACVTS |
| VE-010 | POST Evidence | `self_test.rs` `run_power_on_self_tests()` output | SP 800-140E |
| VE-011 | Source Code (module boundary) | `src/kernel/src/crypto/` (34 files) | ISO 19790 §7.11 |
| VE-012 | Build Instructions | `scripts/cmvp-build.sh` | ISO 19790 §7.11 |
| VE-013 | Formal Verification Report | `docs/compliance/formal-verification-report.md` | ISO 19790 §7.12 |
| VE-014 | Side-Channel Analysis | `side_channel.rs` + `ct_utils.rs` | ISO 19790 §7.8 |
| VE-015 | FPGA Prototype Spec | `docs/compliance/fpga-prototype-spec.md` | Supplementary |
| VE-016 | Developer Documentation | `salvi_docs/modules/05_CRYPTOGRAPHY.md` | ISO 19790 §7.11 |
| VE-017 | FIPS Validation Plan | `docs/compliance/fips-validation-plan.md` | Internal |
| VE-018 | CAVP Submission Guide | `docs/compliance/cavp-submission-guide.md` | Internal |
| VE-019 | FIPS Module Boundary Spec | `salvi_docs/modules/15_FIPS_BOUNDARY.md` | ISO 19790 §7.2 |
| VE-020 | Migration Guide | `salvi_docs/modules/16_MIGRATION_GUIDE.md` | ISO 19790 §7.11 |
| VE-021 | CSTL Engagement Guide | `docs/compliance/cstl-engagement.md` | Internal |
| VE-022 | Version Manifest | `libternary/VERSION_MANIFEST.json` | Configuration mgmt |

---

## Document Cross-Reference Matrix

### Security Policy (VE-001) References

| SP Section | References Document(s) | Reference Type |
|---|---|---|
| §1.3 Operational Environments | VE-005 | Full specification |
| §1.4 CAVP Certificates | VE-007, VE-008 | Certificate numbers |
| §2.2 Module Boundary | VE-002 | Boundary diagram |
| §3.2 Services | VE-004 (SSP associations) | SSP usage per service |
| §5 Operational Environment | VE-005 | Platform specification |
| §8 SSP Management | VE-004 | Full SSP inventory |
| §9.1 POST | VE-010 | POST evidence |
| FSM description | VE-003 | State model |

### CSTL Submission Order

For lab engagement, submit documents in this order:

1. **VE-001** Security Policy — primary review document
2. **VE-002** Boundary Diagram — establishes scope
3. **VE-003** Finite State Model — lab traces all transitions
4. **VE-004** SSP Inventory — lab audits every secret value
5. **VE-005** Operational Environments — defines test platforms
6. **VE-006** Entropy Assessment — lab evaluates SP 800-90B compliance
7. **VE-007-009** CAVP materials — algorithm certificates + vectors
8. **VE-010** POST evidence — self-test coverage verification
9. **VE-011** Source code — module boundary files for review
10. **VE-012** Build instructions — lab reproduces binary
11. **VE-013-014** Verification + analysis — defense-in-depth evidence
12. **VE-015-020** Supplementary documentation
13. **VE-021** Engagement guide — for internal use
14. **VE-022** Version manifest — configuration tracking

---

## Document Version Control

| Document | Version | Last Modified | Author |
|---|---|---|---|
| VE-001 Security Policy | 3.0.0 | February 2026 | Applied Physics Division |
| VE-002 Boundary Diagram | 1.0.0 | February 2026 | Applied Physics Division |
| VE-003 Finite State Model | 1.0.0 | February 2026 | Applied Physics Division |
| VE-004 SSP Inventory | 1.0.0 | February 2026 | Applied Physics Division |
| VE-005 Operational Environments | 1.0.0 | February 2026 | Applied Physics Division |
| VE-006 Entropy Assessment | 1.0.0 | February 2026 | Applied Physics Division |
| VE-012 Build Instructions | 1.0.0 | February 2026 | Applied Physics Division |
| VE-021 CSTL Engagement | 1.0.0 | February 2026 | Applied Physics Division |
| VE-022 Version Manifest | 3.0.0 | February 2026 | Applied Physics Division |

---

## Completeness Checklist

### FIPS 140-3 Required Documents
- [x] Non-Proprietary Security Policy (VE-001) — 12 SP 800-140B sections
- [x] Cryptographic Boundary (VE-002) — 34 files inside, excluded components listed
- [x] Finite State Model (VE-003) — 9 states, transition table, error behavior
- [x] SSP Inventory (VE-004) — 23 SSPs with generation, storage, zeroization
- [x] Operational Environments (VE-005) — OE-1 (x86_64), OE-2 (aarch64)
- [x] Entropy Assessment (VE-006) — noise source, health tests, conditioning

### CAVP Materials
- [x] Algorithm certificates (VE-007) — 12 algorithms, pending ACVTS submission
- [x] ACVTS registration JSON (VE-008) — generated by acvts.rs
- [x] ACVTS response JSON (VE-009) — generated by acvts.rs

### Evidence & Testing
- [x] POST evidence (VE-010) — 12 KATs in self_test.rs
- [x] Source code (VE-011) — 34 crypto files
- [x] Build instructions (VE-012) — cmvp-build.sh
- [x] Formal verification (VE-013) — 13 properties
- [x] Side-channel analysis (VE-014) — ct_utils.rs + side_channel.rs

### Supplementary
- [x] FPGA spec (VE-015)
- [x] Developer docs (VE-016)
- [x] Validation plan (VE-017)
- [x] CAVP guide (VE-018)
- [x] FIPS boundary spec (VE-019)
- [x] Migration guide (VE-020)
- [x] CSTL engagement (VE-021)
- [x] Version manifest (VE-022)

---

## Artifact Integrity

All documents are version-controlled in the SigmaWolf-8/Ternary GitHub
repository on the `main` branch with signed commits. Document integrity
can be verified via git commit hashes.

---

*Document: Master Index*
*Salvi Framework — Capomastro Holdings Ltd.*
