# Task #119 — Continuous Attestation Service

## What This Is

Complete attestation module for the inter-cube daemon. 10 Rust files implementing all 6 spec tasks. Logging produces Forma Codex 18∏ native LogEntry values with 27-trit classification, TIS-27 tamper-evident hash chains, and three faces per entry. Broadcast timing uses HModal signal dispatch (duty = 1/4, from TM-2026-028).

## Files

All go into `services/inter-cube/src/attestation/`:

| File | Lines | Tests | Purpose |
|------|-------|-------|---------|
| mod.rs | 68 | — | Module root + re-exports |
| report.rs | 460 | 5 | Task 1: Report struct (10 fields), wire format |
| signing.rs | 274 | 7 | Task 1: TL-DSA key derivation, Zeroize, context strings |
| merkle.rs | 227 | 6 | Task 2: Rolling Merkle tree (TIS-27, domain separation) |
| audit.rs | 224 | 3 | Task 1: 8 audit event types with severity |
| broadcast.rs | 422 | 12 | Task 3: HModal dispatch (α/β), HPTP jitter, backoff |
| verify.rs | 432 | 8 | Task 4: Replay protection, suspicion counters, FTS |
| versioning.rs | 150 | 5 | Task 5: Schema registry, upgrade window (4h auto-expiry) |
| failure.rs | 217 | 5 | Task 6: 7 failure modes, operator messages |
| logging.rs | 699 | 9 | Forma Codex 18∏: 27-trit class, TIS-27 chain, 3 faces |
| **Total** | **3,173** | **60** | |

## How To Apply

### Step 1: Create the attestation directory

```bash
mkdir -p services/inter-cube/src/attestation/
```

### Step 2: Extract the zip into it

```bash
unzip task119-attestation-complete.zip -d services/inter-cube/src/attestation/
```

### Step 3: Register the module in lib.rs

Open `services/inter-cube/src/lib.rs`.

Add this line after the existing module declarations (after `pub mod relay_circuit;` around line 127):

```rust
pub mod attestation;
```

Add these re-exports after the existing `pub use relay_frames::` block (around line 153):

```rust
pub use attestation::{
    AttestationReport, SignedAttestationReport, AttestationSigningKey,
    AttestationVerifier, VerifyResult, SuspicionOutcome,
    BroadcastConfig, BroadcastState, HModalTiming, DispatchPhase,
    RollingMerkleTree, AttestAuditEvent, AttestSeverity,
    ServiceState, DegradedReason,
    VersionRegistry, UpgradeWindow,
    AttestationLogger, LogEntry, ClassTrit,
};
```

### Step 4: Build and test

```bash
cd services/inter-cube
cargo check 2>&1 | head -30
cargo test attestation 2>&1 | tail -30
```

Expected: 0 errors, 60 attestation tests pass.

## Architecture

```
attestation/
├── mod.rs          ← module root, re-exports
├── report.rs       ← AttestationReport (10 fields, TritInt, Rep C wire)
├── signing.rs      ← TLSponge-385 key derivation, Zeroize, TL-DSA context
├── merkle.rs       ← rolling Merkle tree (TIS-27, leaf/internal domain sep)
├── audit.rs        ← 8 event types (CRITICAL/WARNING/INFO)
├── broadcast.rs    ← HModal dispatch (α idle 75% / β dispatch 25%)
├── verify.rs       ← replay protection, partition-aware suspicion, FTS Suspect
├── versioning.rs   ← schema registry, upgrade window (auto-expiry 4h)
├── failure.rs      ← 7 dependency failures → block/retry/degrade
└── logging.rs      ← Forma Codex 18∏ native LogEntry producer
```

## HModal Signal Dispatch (broadcast.rs)

The broadcast service uses the framework's HModal signal model (TM-2026-028):

- **duty = 1/R₂ = 1/4** → 25% dispatch window (β), 75% idle window (α)
- **dispatch_ratio = 1/3** → dispatch time = idle time / 3
- 120s interval: 90s idle (collect Merkle leaves), 30s dispatch (sign + broadcast)
- Constants from `ternary_math::constants::DUTY_NUM/DEN`

## Forma Codex 18∏ Logging (logging.rs)

Every attestation log entry is a Forma Codex LogEntry with:

- **27-trit classification** across Who/What/Where/When/Why/How/Peace
  - Each trit ∈ {1,2,3} (Rep C — zero excluded, corruption by encoding)
  - Attestation events pre-classified: e.g. ATTEST_SIGN_FAIL →
    Who=System/Auto/Admin/Platform, What=Security/Modify/Failure/Non-idempotent
  - Peace (dims 25-27) mapped from outcome: Failure → High priority
- **TIS-27 identity hash** over 24 immutable classification trits + timestamp + content
- **TIS-27 chain hash** linking to previous entry (tamper-evident)
  - First entry: chain_hash = identity_hash
  - Subsequent: chain_hash = TIS-27(identity ‖ prev_chain)
  - `verify_chain()` detects tampering and returns index of first break
- **Three faces** per entry:
  - Face 1: Human message (operator)
  - Face 2: Raw structured data (engineer)
  - Face 3: Correlation context (tracing — sender/receiver addresses)
- **HPTP femtosecond timestamps** (binary crosses TritInt gate on entry)
- Log directory: `C:\PlenumNET\Logs\attestation\`
- All node identification in dot-separated Rep C (INVARIANT 9)
