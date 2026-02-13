# Architecture Decision Records

This directory contains the Architecture Decision Records (ADRs) for the PlenumNET / Salvi Framework. Each ADR documents a significant design choice, the context that drove it, the alternatives considered, and the consequences accepted.

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](ADR-001-35-opcode-instruction-set.md) | 55-Opcode Instruction Set for the PlenumNET VM | Accepted |
| [ADR-002](ADR-002-gf3-over-balanced-ternary.md) | GF(3) Field Arithmetic as the Kernel Primitive | Accepted |
| [ADR-003](ADR-003-lamport-alongside-lattice.md) | Dual-Layer Cryptographic Architecture — Lamport + Lattice | Accepted |

## How to Propose a New ADR

1. Copy the template below into a new file: `ADR-NNN-short-title.md`
2. Fill in Context, Decision, Consequences, and Alternatives.
3. Open a PR. The ADR is **Proposed** until merged; **Accepted** once merged.
4. To supersede an ADR, create a new one and mark the old as **Superseded by ADR-NNN**.

## Template

```markdown
# ADR-NNN: Title

| Field       | Value        |
|-------------|--------------|
| **Status**  | Proposed     |
| **Date**    | YYYY-MM-DD   |
| **Author**  |              |
| **Context** |              |

## 1 · Context
## 2 · Decision
## 3 · Consequences
## 4 · Alternatives Considered
```
