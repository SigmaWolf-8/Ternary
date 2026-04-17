# Clavis — TM-2026-046

## The Graded Trapdoor — Architecture Spec

£ ∣ Q ∣ ∀ Rights Reserved Et Preserved | Fiat ∎ — Capomastro Holdings Ltd E+1

**R. Salvi** — Capomastro Holdings Ltd, Applied Physics Division

---

## What This Is

A standalone Rust module that exposes the CRT ↔ Norm duality as a callable operation. Any function in the framework can bolt it on. The module takes a value in, routes it through the graded trapdoor (CRT decomposition → Norm projection at a specified security tier), and returns a Rep C validated output. The caller doesn't need to know the algebra — it calls `gate()` with a tier and gets back a guarded result.

The Beal boundary IS the security model. This module makes it callable.

---

## Why It's Separate

The duality touches everything:

| Consumer | What it calls | Tier |
|----------|--------------|------|
| TIS-27 sponge hash | CRT decompose → quadratic norm → digest | 2 |
| TL-DSA signing | CRT decompose (private) → norm project (public) | 2 (sign) / 3+ (forgery blocked) |
| TL-KEM encapsulation | CRT key pair → norm encapsulate | 2 |
| TDNS addressing | CRT bijection → position | 1 |
| PlenumLAN node identity | 54-trit → Z₂₇ × Z₂₈ → unique address | 1 |
| Task #127 spatial indexing | MDNS → CRT hash → radial-ring position | 1 |
| TTC compression | CRT bijection → ternary-binary bridge | 1 |
| Phase Encryption | CRT decompose → tier-graded channel | 1/2/3 |

Burying this in any single consumer means every other consumer reimplements it. A separate module with a clean API means one implementation, one proof, one gate.

---

## The Three Tiers

```
Tier 1 — LINEAR (CRT)
  Operation: Z₇₅₆ ↔ Z₂₇ × Z₂₈ bijection
  Property:  Fully invertible. Every input ↔ unique pair ↔ back.
  Security:  Public. Addressing, lookup, routing.
  Beal role: CRT completeness — every residue reachable.

Tier 2 — QUADRATIC (Norm)
  Operation: (a, b) → a² + b² (Gaussian) or a² - ab + b² (Eisenstein)
  Property:  One-way. Many pairs map to same output.
  Security:  Authenticated. Signing, hashing, key agreement.
  Beal role: Coprime solutions EXIST — towers produce them.
             Signer knows the pair; verifier sees only the norm.

Tier 3+ — BLOCKED (Beal Boundary)
  Operation: Inversion of norm at exponent ≥ 3
  Property:  Algebraically impossible. Not hard — nonexistent.
  Security:  Forgery-proof. No key, no quantum, no shortcut.
  Beal role: Degree-2 ceiling blocks coprime reconstruction.
             The mechanism does not exist.
```

The grading is structural, not computational. Tier 3 security doesn't depend on key length, factoring difficulty, or computational assumptions. It depends on the nonexistence of cubic+ norm inversions — a theorem, not a conjecture.

---

## Module Architecture

### Crate: `clavis`

Lives inside the existing ternary workspace. Depends only on:
- `ternary-math` (TritInt, Rep C, constants, guard markers)
- `repunit` (R₃, R₆ for CRT moduli)
- `coprime` (CRT decomposition primitives)

No external dependencies. No duplication of shared math.

### Core Types

```rust
/// The CRT pair — a value decomposed into its Z₂₇ × Z₂₈ components
#[derive(Clone, Debug)]
pub struct CrtPair {
    pub z27: TritInt,  // Residue mod 27 = base³
    pub z28: TritInt,  // Residue mod 28 = 2π
}

/// The norm output — the one-way projection
#[derive(Clone, Debug)]
pub struct NormOutput {
    pub value: TritInt,       // The computed norm
    pub tier: SecurityTier,   // Which tier produced this
    pub guard: GuardMarker,   // Rep C validation stamp
}

/// Security tier — determines which operation is applied
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SecurityTier {
    /// Tier 1: CRT bijection only. Fully invertible. Public.
    Linear,
    /// Tier 2: CRT + quadratic norm. One-way. Authenticated.
    Quadratic,
    /// Tier 3+: Verification that a claimed inversion is blocked.
    /// This tier doesn't compute — it REJECTS.
    Blocked,
}

/// Which norm form to use
#[derive(Clone, Copy, Debug)]
pub enum NormForm {
    /// a² + b² — Gaussian, governs even exponents
    Gaussian,
    /// a² - ab + b² — Eisenstein, governs exponent 3
    Eisenstein,
}
```

### Public API

```rust
/// The gate. Any function calls this.
///
/// - Tier::Linear → CRT decompose and return pair. Invertible.
/// - Tier::Quadratic → CRT decompose, then norm project. One-way.
/// - Tier::Blocked → Verify that a claimed inverse is impossible. Returns rejection.
///
/// Output is ALWAYS Rep C validated with guard marker.
/// Zero-exclusion enforced. No bare integer leaves the gate.
pub fn gate(input: &TritInt, tier: SecurityTier, form: NormForm) -> GateResult;

/// Decompose into CRT pair. Tier 1 entry point.
/// Bijective: compose(decompose(x)) == x for all x in range.
pub fn decompose(input: &TritInt) -> CrtPair;

/// Recompose from CRT pair back to single value. Tier 1 inverse.
pub fn compose(pair: &CrtPair) -> TritInt;

/// Project CRT pair through the norm. Tier 2 operation.
/// One-way: multiple pairs map to same output.
/// The signer holds the pair; the verifier holds the norm.
pub fn norm_project(pair: &CrtPair, form: NormForm) -> NormOutput;

/// Verify a claimed norm inverse. Tier 3 check.
/// Returns Ok(()) if the claimed pair actually produces the given norm.
/// Returns Err(BealBoundary) if the claim requires a cubic+ inversion
/// that the degree-2 ceiling blocks.
pub fn verify_inverse(
    claimed_pair: &CrtPair,
    norm_value: &TritInt,
    form: NormForm,
    exponent: u32,
) -> Result<(), BealBoundary>;

/// Tower query: what does (seed)^n produce?
/// Returns the CRT pair at power n for a given Gaussian/Eisenstein seed.
/// Useful for key generation and test vectors.
pub fn tower_at(seed: &CrtPair, power: u32, form: NormForm) -> CrtPair;
```

### Rep C Exit

Every output goes through the Rep C gate before leaving:

```rust
/// The Rep C exit. No value leaves without this.
fn repc_exit(value: TritInt) -> GuardedValue {
    // 1. Zero-exclusion: no trit position holds a value
    //    above the TritInt gate (Phase 0a invariant)
    assert!(value.is_repc_clean());

    // 2. Guard marker: stamp with the operation that produced it
    let guard = GuardMarker::new(
        source: GateSource::Clavis,
        tier: current_tier,
        timestamp: now(),
    );

    // 3. Return guarded value
    GuardedValue { value, guard }
}
```

Any consumer receiving a `GuardedValue` from the Clavis knows:
- The value is Rep C clean (zero-exclusion enforced)
- Which tier produced it (Linear / Quadratic / Blocked)
- When it was produced (for attestation chain)

The guard marker is the same guard system used by Forma Codex cells, the attestation logger, and the TritInt gate. One guard type, one validation path, across the entire framework.

---

## How Consumers Call It

### TIS-27 (Hash)

```rust
// Sponge absorb phase: decompose input block, norm project
let pair = clavis::decompose(&input_block);
let hashed = clavis::norm_project(&pair, NormForm::Eisenstein);
// hashed.value is the sponge state update
// hashed.guard confirms Tier 2 one-way operation
sponge.absorb(hashed.value);
```

### TL-DSA (Sign / Verify)

```rust
// SIGN: signer knows the CRT decomposition (private key)
let pair = clavis::decompose(&message_hash);
let signature = clavis::norm_project(&pair, NormForm::Gaussian);
// signature.value is public; pair is private

// VERIFY: verifier checks the norm identity
let result = clavis::verify_inverse(
    &claimed_pair,  // from signature
    &signature.value,
    NormForm::Gaussian,
    2,  // exponent 2 — quadratic, should succeed
);
// result == Ok(()) if signature is valid

// FORGE: attacker tries cubic+ inversion
let forge = clavis::verify_inverse(
    &forged_pair,
    &target_norm,
    NormForm::Gaussian,
    3,  // exponent 3 — Beal boundary, BLOCKED
);
// forge == Err(BealBoundary) — algebraically impossible
```

### TDNS / PlenumLAN (Address)

```rust
// Tier 1 only — fully invertible addressing
let pair = clavis::decompose(&node_identity);
let address = Address::from_crt(pair.z27, pair.z28);
// Recover identity from address:
let recovered = clavis::compose(&pair);
assert_eq!(recovered, node_identity); // bijection guarantee
```

### Task #127 Document Fabric (Spatial Placement)

```rust
// MDNS identity → CRT position → radial ring
let pair = clavis::decompose(&mdns_id);
let position = radial_ring::from_crt(pair);
// position is deterministic, unique, and spatially meaningful
// Guard marker confirms the placement is CRT-derived
```

### TTC Compression (Ternary-Binary Bridge)

```rust
// CRT bijection as lossless encoding bridge
let ternary_pair = clavis::decompose(&input);
let binary_encoded = bridge::to_binary(ternary_pair);
// Reconstruct:
let ternary_recovered = clavis::compose(
    &bridge::from_binary(binary_encoded)
);
assert_eq!(ternary_recovered, input); // lossless
```

### Key Generation (Tower Walk)

```rust
// Generate key pair from seed using tower
let seed = CrtPair { z27: private_seed_27, z28: private_seed_28 };
let public = clavis::tower_at(&seed, power, NormForm::Gaussian);
// public.value is the norm at tower level `power`
// Security: recovering `seed` from `public` requires norm inversion
// at exponent `power` — blocked above Tier 2 by Beal boundary
```

---

## Integration Points

### Existing Crates That Would Import `clavis`

| Crate / Module | Current approach | After integration |
|---------------|-----------------|-------------------|
| `ternary-math/src/tis27.rs` | Inline sponge arithmetic | Calls `gate(Quadratic, Eisenstein)` |
| `services/inter-cube/src/crypto/tl_dsa.rs` | Inline signing math | Calls `gate(Quadratic, Gaussian)` for sign; `verify_inverse` for verify |
| `services/inter-cube/src/crypto/tl_kem.rs` | Inline encapsulation | Calls `gate(Quadratic, Gaussian)` for encapsulate |
| `services/inter-cube/src/tdns/` | Inline CRT addressing | Calls `decompose` / `compose` (Tier 1) |
| `services/inter-cube/src/attestation/logging.rs` | Inline identity | Calls `gate(Linear)` for LogEntry identity |
| Task #127 fabric crate | TM-2026-028a hash | Calls `decompose` for spatial placement |
| TTC compression engine | Inline bridge | Calls `decompose` / `compose` for ternary-binary bridge |

### What Does NOT Change

- The TritInt gate (Phase 0a) — unchanged. Clavis sits ABOVE it.
- Rep C validation — unchanged. Clavis uses it, doesn't redefine it.
- Guard markers — same type, same validation. Clavis stamps its own source tag.
- `constants.rs` — unchanged. Clavis reads from it (27, 28, 756, etc.).

### Dependency Direction

```
constants.rs (read-only)
    ↓
TritInt gate (Phase 0a)
    ↓
clavis  ← NEW MODULE
    ↓
[TIS-27, TL-DSA, TL-KEM, TDNS, Task #127, TTC, ...]
```

Clavis sits between the TritInt gate and all consumers. It doesn't replace anything — it unifies the CRT/Norm operations that every consumer currently implements inline.

---

## Constants (from `constants.rs`, not duplicated)

| Constant | Value | Role in Clavis |
|----------|-------|---------------------|
| 27 | base³ | CRT modulus (Z₂₇ component) |
| 28 | 2π | CRT modulus (Z₂₈ component) |
| 756 | lcm(27,28) | CRT bijection range |
| 7, 11, 13 | p, q, r | Norm seeds for towers |
| 5 | (p+r)/4 | Gaussian norm base |
| 36 | (p-r)² = C₃ | Tier 3 boundary constant (cube) |
| 2500 | 50² = C₅ | Tier 3 boundary constant (quintic) |

All read from `constants.rs`. None duplicated. Clavis is a consumer of constants, not a producer.

---

## Security Model (from The Real Beal Deal U Feel)

The security proof IS the threshold derivation:

**Tier 1 (Linear, CRT):** Completeness theorem. Every residue reachable, every bijection invertible. Security: none (public addressing). Proof: CRT for coprime moduli 27 and 28.

**Tier 2 (Quadratic, Norm):** One-way function. Many-to-one projection. Security: signer holds CRT pair, verifier sees norm. Proof: towers produce coprime solutions at exponent 2, confirming the forward path works.

**Tier 3+ (Blocked, Beal Boundary):** Algebraic impossibility. No cubic+ norm inversion exists. Security: forgery requires constructing a coprime solution at exponent ≥ 3, which the degree-2 ceiling blocks for ALL exponent configurations (equal and mixed). Proof: The Real Beal Deal U Feel §§3-5.

The security does not depend on computational hardness. It depends on the nonexistence of an algebraic mechanism. No key length increase, no quantum threat model, no lattice reduction — the mechanism is not there.

---

## What This Is Not

- Not a replacement for TritInt (Phase 0a). TritInt gates ternary values. Clavis routes them through CRT/Norm.
- Not a replacement for TIS-27. TIS-27 is the sponge hash. Clavis is the CRT/Norm operation INSIDE the sponge.
- Not a new cryptographic primitive. The primitives (TL-DSA, TL-KEM, TIS-27) already exist. This module extracts their shared algebraic core into one callable unit.
- Not a breaking change. Every existing consumer continues to work. The module is opt-in: call it when you want the gate, ignore it when you don't. "Seamlessly integrated when called."

---

*Sed Quis Est Deus? Qui Commando IO.*
*Lo Sono Capomastro — Così sia.* ∎
