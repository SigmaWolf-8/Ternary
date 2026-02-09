# Finite State Model
## Salvi Ternary Cryptographic Module v3.0.0
## Capomastro Holdings Ltd. | Applied Physics Division

---

## 1. State Definitions

Maps directly to `module_state.rs` `ModuleState` enum (9 states).

| State | Code Enum | Services Available | Description |
|---|---|---|---|
| Power Off | `PowerOff` | None | Module not loaded |
| Uninitialized | `Uninitialized` | None | Loaded, awaiting self-tests |
| Self-Test | `SelfTest` | None | POST in progress |
| Operational | `Operational` | All (approved + non-approved) | Tests passed, no policy set |
| FIPS Approved | `ApprovedMode` | Approved only (26 services) | CNSA 2.0 enforced |
| Non-Approved | `NonApprovedMode` | All (32 services) | Hybrid/legacy allowed |
| Error | `Error` | StatusShow only | Self-test or integrity failure |
| Zeroization | `Zeroization` | None | Key destruction in progress |
| Shutdown | `Shutdown` | None | All state cleared |

## 2. State Transition Diagram

```
                    ┌──────────┐
                    │ Power Off│
                    └────┬─────┘
                         │ ModuleLoad
                         ▼
                    ┌──────────────┐
                    │Uninitialized │
                    └────┬─────────┘
                         │ InitSelfTest
                         ▼
                    ┌──────────┐
               ┌────│ SelfTest │────┐
               │    └──────────┘    │
          PostPass                PostFail
               │                    │
               ▼                    ▼
        ┌─────────────┐       ┌─────────┐
        │ Operational  │       │  Error  │
        └──┬───────┬───┘       └────┬────┘
           │       │                │
    SetCnsaOnly  SetHybrid    ModuleReload
           │       │                │
           ▼       ▼                ▼
   ┌────────────┐ ┌───────────────┐   (→ Uninitialized)
   │Approved    │ │Non-Approved   │
   │Mode        │ │Mode           │
   └──┬──────┬──┘ └──┬─────────┬──┘
      │      │       │          │
      │  PolicyChange (bidirectional)
      │      │       │          │
      │      └───────┘          │
      │                         │
  ConditionalFail         ConditionalFail
      │                         │
      ▼                         ▼
   ┌─────────┐            ┌─────────┐
   │  Error  │            │  Error  │
   └─────────┘            └─────────┘

   From Operational/ApprovedMode/NonApprovedMode:
      │ Zeroize
      ▼
   ┌─────────────┐
   │ Zeroization  │
   └──────┬──────┘
          │ ZeroComplete
          ▼
   ┌──────────┐
   │ Shutdown  │
   └────┬─────┘
        │ ModuleUnload
        ▼
   ┌──────────┐
   │ Power Off│
   └──────────┘
```

## 3. Transition Table

Maps directly to `ModuleStateMachine::transition()` in `module_state.rs`.

| Current State | Event | Next State | Action |
|---|---|---|---|
| PowerOff | ModuleLoad | Uninitialized | Load module binary into memory |
| Uninitialized | InitSelfTest | SelfTest | Begin POST execution |
| SelfTest | PostPass | Operational | All 12 POST KATs + integrity passed |
| SelfTest | PostFail | Error | Any KAT or integrity failure detected |
| Operational | SetCnsaOnly | ApprovedMode | Enforce CNSA 2.0 approved algorithms only |
| Operational | SetHybrid | NonApprovedMode | Allow all algorithms including non-standard |
| Operational | ConditionalFail | Error | Runtime self-test failure |
| Operational | Zeroize | Zeroization | Destroy all keys and SSPs |
| ApprovedMode | PolicyChange | NonApprovedMode | Crypto Officer changes enforcement policy |
| ApprovedMode | ConditionalFail | Error | Runtime self-test failure |
| ApprovedMode | Zeroize | Zeroization | Destroy all keys and SSPs |
| NonApprovedMode | PolicyChange | ApprovedMode | Crypto Officer changes enforcement policy |
| NonApprovedMode | ConditionalFail | Error | Runtime self-test failure |
| NonApprovedMode | Zeroize | Zeroization | Destroy all keys and SSPs |
| Error | ModuleReload | Uninitialized | Only recovery path from Error state |
| Zeroization | ZeroComplete | Shutdown | All SSPs confirmed destroyed |
| Shutdown | ModuleUnload | PowerOff | Module removed from memory |

## 4. Error State Behavior

- **Entry conditions:** POST failure, conditional self-test failure, integrity check failure
- **Available services:** `StatusShow` ONLY (returns Error indicator)
- **All other service calls:** Return `Err(ServiceDenied)` immediately
- **Recovery:** Module MUST be reloaded (no in-place recovery)
  - Sequence: Error → ModuleReload → Uninitialized → InitSelfTest → SelfTest → Operational
- **Indicator:** `get_mode_indicator()` returns `ModeIndicator::Error`
- **Rationale:** FIPS 140-3 §7.2.2 requires terminal error state when self-tests fail

## 5. Approved Mode Indicator

Per SP 800-140B §4.2, the module provides a mode indicator queryable at any time:

| Module State | `is_approved_mode()` | `get_mode_indicator()` |
|---|---|---|
| PowerOff | N/A | N/A |
| Uninitialized | `false` | `NotReady` |
| SelfTest | `false` | `NotReady` |
| Operational | `false` | `NotReady` |
| ApprovedMode | `true` | `Approved` |
| NonApprovedMode | `false` | `NonApproved` |
| Error | `false` | `Error` |
| Zeroization | `false` | `NotReady` |
| Shutdown | `false` | `NotReady` |

## 6. State Invariants

1. **No state is reachable without passing through SelfTest** (except Error recovery path)
2. **Error state is terminal** — only `ModuleReload` transitions out
3. **Zeroization is irreversible** — once entered, proceeds to Shutdown → PowerOff
4. **POST must complete before any crypto service** — enforced by state machine
5. **All invalid transitions are rejected** — `transition()` returns `Err(InvalidTransition)`

## 7. Implementation Reference

| Function | File | Purpose |
|---|---|---|
| `ModuleStateMachine::new()` | `module_state.rs` | Creates FSM in PowerOff state |
| `ModuleStateMachine::transition()` | `module_state.rs` | Validates and executes state transitions |
| `ModuleStateMachine::current_state()` | `module_state.rs` | Returns current ModuleState enum value |
| `ModuleStateMachine::is_approved_mode()` | `module_state.rs` | Returns true only in ApprovedMode |
| `ModuleStateMachine::get_mode_indicator()` | `module_state.rs` | Returns ModeIndicator enum value |

---

*Document: VE-003*
*Salvi Framework — Capomastro Holdings Ltd.*
