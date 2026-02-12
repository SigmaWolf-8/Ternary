# Contributing to the Salvi Framework
## Capomastro Holdings Ltd., Applied Physics Division

Thank you for your interest in contributing to the Salvi Framework / PlenumNET project.

---

## Intellectual Property Notice

All contributions to this project become the joint property of Capomastro
Holdings Ltd. under the terms of our Contributor License Agreement (CLA).
By submitting a pull request, you represent that you have the right to
grant the licenses described in the CLA and that you have read and agree
to its terms.

The Salvi Framework™, PlenumNET™, and all associated technology are the
exclusive intellectual property of Capomastro Holdings Ltd. (Canada).
Patent(s) Pending. See the LICENSE and INTELLECTUAL-PROPERTY-NOTICE.md
files in the repository root for full terms.

---

## Contributor License Agreement

Before we can accept any contributions, you must sign and submit the
Contributor License Agreement (CLA). See [CLA.md](CLA.md) for the full
agreement. No pull requests will be merged without a signed CLA on file.

Return signed copies to: Rsalvi@Salvigroup.com

---

## Code of Conduct

All contributors are expected to conduct themselves professionally and respectfully. Harassment, discrimination, and disruptive behavior will not be tolerated.

---

## Submitting Changes

### Prerequisites

1. **Sign the CLA.** No exceptions.
2. **Read the LICENSE** and understand that all contributions are subject to the proprietary license.
3. **Understand the mathematical requirements.** The Salvi Framework operates under rigorous mathematical standards derived from the Unified 13D Torsion Plenum Theory and GF(3) arithmetic.

### Process

1. Fork the repository (if external) or create a feature branch (if internal).
2. Make your changes in a focused, well-scoped branch.
3. Ensure all source files include the appropriate copyright header (see SOURCE-FILE-LICENSE-HEADERS.md).
4. Write clear, descriptive commit messages.
5. Submit a pull request with:
   - A description of the change and its purpose
   - Any relevant issue numbers
   - Confirmation that you have a signed CLA on file
6. Wait for code review. All PRs require at least one approval.

### Code Standards

- **Rust:** Follow standard Rust formatting (`cargo fmt`). All code must pass `cargo clippy` without warnings.
- **TypeScript/JavaScript:** Follow the existing code style. Use TypeScript where possible.
- **Python:** Follow PEP 8.
- **Documentation:** All public APIs must be documented.

### Mathematical Standards

Contributions involving ternary arithmetic, GF(3) operations, or Tribonacci-derived constants must:

- Use the canonical Tribonacci constant (τ) as defined in `shared/tribonacci-constants.ts`
- Implement `verifyTau()` checks where applicable
- Support all three ternary representations: Computational ({-1, 0, +1}), Network ({0, 1, 2}), Human ({1, 2, 3})
- Include property-based tests for mathematical correctness

### Cryptographic Standards

Contributions to the cryptographic layer must:

- Maintain constant-time execution (no timing side channels)
- Preserve CNSA 2.0 algorithm coverage
- Not weaken any existing security guarantees
- Include CAVP/ACVTS test vector validation where applicable

---

## Security Vulnerabilities

Do **NOT** create a public GitHub issue for security vulnerabilities. See [SECURITY.md](.github/SECURITY.md) for responsible disclosure procedures.

---

## Questions

For questions about contributing, contact Rsalvi@Salvigroup.com.

Repository: https://github.com/SigmaWolf-8/Ternary
