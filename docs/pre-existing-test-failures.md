# Pre-Existing Test Failures — Inter-Cube Crate

**Date:** 2026-04-12
**Crate:** `services/inter-cube` v2.4.13
**Context:** These 2 test failures exist independently of Task #27 (Relay Server) and Task #119 (Attestation Service). Neither source file was modified by those tasks.

---

## Failure 1: `api::slots_tests::test_verify_bearer_token_empty`

### File
`services/inter-cube/src/api.rs` — line 1294

### Panic Message
```
assertion failed: !verify_bearer_token("Bearer ", "")
```

### Test Code (lines 1292–1296)
```rust
#[test]
fn test_verify_bearer_token_empty() {
    assert!(!verify_bearer_token("Bearer ", ""));   // ← FAILS HERE
    assert!(!verify_bearer_token("Bearer abc", ""));
}
```

### Function Under Test (lines 789–804)
```rust
fn verify_bearer_token(header_value: &str, api_key: &str) -> bool {
    if !header_value.starts_with("Bearer ") {
        return false;
    }
    let token = &header_value[7..];              // token = ""
    let token_bytes = token.as_bytes();          // token_bytes = []  (len 0)
    let key_bytes = api_key.as_bytes();          // key_bytes = []    (len 0)
    let len_ok: Choice = token_bytes.len().ct_eq(&key_bytes.len());  // 0 == 0 → TRUE
    let pad_len = std::cmp::max(token_bytes.len(), key_bytes.len()).max(1);  // max(0,0).max(1) = 1
    let mut padded_token = vec![0u8; pad_len];   // [0x00]
    let mut padded_key = vec![0u8; pad_len];     // [0x00]
    padded_token[..token_bytes.len()].copy_from_slice(token_bytes);  // no-op (0 bytes)
    padded_key[..key_bytes.len()].copy_from_slice(key_bytes);        // no-op (0 bytes)
    let content_ok: Choice = padded_token.ct_eq(&padded_key);  // [0x00] == [0x00] → TRUE
    (len_ok & content_ok).into()                 // TRUE & TRUE → TRUE
}
```

### Root Cause

When both `token` (after stripping "Bearer ") and `api_key` are empty strings:

1. `len_ok` = `0.ct_eq(&0)` → **TRUE** (lengths match: both zero)
2. `pad_len` = `max(0, 0).max(1)` = **1** (the `.max(1)` prevents a zero-length allocation)
3. Both `padded_token` and `padded_key` are initialized to `[0x00]`
4. Neither `copy_from_slice` copies anything (source length is 0)
5. `content_ok` = `[0x00].ct_eq(&[0x00])` → **TRUE** (zero-filled padding matches itself)
6. Result: `TRUE & TRUE` → `true`

The function returns `true`, meaning "Bearer " with an **empty secret** authenticates successfully. The test expects `false`.

### Security Impact — **HIGH**

This is a **real authentication bypass** when the API key is empty or unset:
- If `INTER_CUBE_API_KEY` is not set (empty string), any request with `Authorization: Bearer ` (trailing space, no token) would pass authentication.
- The empty-secret case should be rejected unconditionally before any constant-time comparison.

### Suggested Fix

Add an early return rejecting empty API keys before the constant-time comparison:

```rust
fn verify_bearer_token(header_value: &str, api_key: &str) -> bool {
    if api_key.is_empty() {
        return false;  // Never authenticate against an empty/unset key
    }
    if !header_value.starts_with("Bearer ") {
        return false;
    }
    let token = &header_value[7..];
    if token.is_empty() {
        return false;  // Reject empty bearer tokens
    }
    // ... rest of constant-time comparison unchanged ...
}
```

### Last Modified
Commit `125f595` — "v2.4.8: real heartbeat counter + last_heartbeat_epoch + load_pct per service" (pre-Task #27).

---

## Failure 2: `wire::tests::test_validate_v2_message_on_v1_header`

### File
`services/inter-cube/src/wire.rs` — line 1044

### Panic Message
```
assertion failed: matches!(err, WireError::MessageRequiresV2 { .. })
```

### Test Code (lines 1035–1045)
```rust
#[test]
fn test_validate_v2_message_on_v1_header() {
    let mut msg = WireMessage::new(
        MessageType::SignedCrsRegister,  // requires_v2() → true
        0,                               // timestamp_fs
        vec![],                          // payload
    );
    msg.header.version = PROTOCOL_VERSION_V1;  // Force version to V1 (0x01)

    let err = msg.validate().unwrap_err();
    assert!(matches!(err, WireError::MessageRequiresV2 { .. }));  // ← FAILS HERE
}
```

### Relevant Constants (wire.rs)
```rust
pub const PROTOCOL_VERSION_V1: u8 = 0x01;      // line 56
pub const PROTOCOL_VERSION_V2: u8 = 0x02;      // line 59
pub const PROTOCOL_VERSION_MIN: u8 = PROTOCOL_VERSION_V2;  // line 73
```

### Validation Logic (lines 555–572)
```rust
pub fn validate(&self) -> Result<(), WireError> {
    // CHECK 1: Version range check (runs FIRST)
    if !self.header.version_acceptable() {
        return Err(WireError::IncompatibleVersion {   // ← returns THIS
            received: self.header.version,
            min: PROTOCOL_VERSION_MIN,
            max: PROTOCOL_VERSION_CURRENT,
        });
    }

    let msg_type = self.header.message_type()
        .ok_or(WireError::UnknownMessageType(self.header.msg_type))?;

    // CHECK 2: V2 message on pre-V2 header (never reached)
    if msg_type.requires_v2() && self.header.version < PROTOCOL_VERSION_V2 {
        return Err(WireError::MessageRequiresV2 {     // ← test expects THIS
            msg_type: self.header.msg_type,
            version: self.header.version,
        });
    }
    // ...
}
```

### `version_acceptable()` (lines 155–157)
```rust
pub fn version_acceptable(&self) -> bool {
    self.version >= PROTOCOL_VERSION_MIN && self.version <= PROTOCOL_VERSION_CURRENT
    // PROTOCOL_VERSION_MIN = V2 (0x02), so V1 (0x01) fails this check
}
```

### Root Cause

The validation checks are ordered so that the **version range check** (Check 1) fires **before** the **message-version requirement check** (Check 2):

1. Test sets `version = PROTOCOL_VERSION_V1` (0x01)
2. `version_acceptable()` checks `version >= PROTOCOL_VERSION_MIN` → `0x01 >= 0x02` → **false**
3. `validate()` returns `WireError::IncompatibleVersion` immediately
4. The test expects `WireError::MessageRequiresV2`, but that check at line 567 is **never reached**

The test was written when `PROTOCOL_VERSION_MIN` was V1 (allowing V1 headers through Check 1 so Check 2 could fire). When `PROTOCOL_VERSION_MIN` was raised to V2 (line 73), this test became unreachable — the generic version rejection fires first.

### Impact — **LOW** (test-only)

The runtime behavior is correct: V1 headers are rejected. The test just asserts the wrong error variant. No security implication — the message is still rejected.

### Suggested Fix Options

**Option A — Fix the test to match current behavior:**
```rust
#[test]
fn test_validate_v2_message_on_v1_header() {
    let mut msg = WireMessage::new(
        MessageType::SignedCrsRegister,
        0,
        vec![],
    );
    msg.header.version = PROTOCOL_VERSION_V1;

    let err = msg.validate().unwrap_err();
    // V1 is below PROTOCOL_VERSION_MIN, so IncompatibleVersion fires first
    assert!(matches!(err, WireError::IncompatibleVersion { .. }));
}
```

**Option B — Remove the test (redundant):**
The `test_validate_unknown_version` test already covers version rejection for out-of-range versions. This test is now redundant since V1 is below the minimum.

**Option C — Lower the minimum for this test only:**
Not recommended — changing `PROTOCOL_VERSION_MIN` would weaken security.

### Last Modified
Commit `26669d0` — "Task #35: PlenumNET Node + Array3 Node Cluster" (pre-Task #27).

---

## Verification: Neither Failure Was Introduced by Task #27 or #119

| File | Last substantive commit | Task #27 / #119 changes |
|------|------------------------|------------------------|
| `api.rs` | `125f595` (v2.4.8) | Only added missing `last_heartbeat_epoch` and `load_pct` fields to 2 test initializers — did not touch `verify_bearer_token` or its tests |
| `wire.rs` | `26669d0` (Task #35) | Not modified at all |

Both failures are reproducible on the codebase state before Task #27 and #119 file placement.

---

## Priority Assessment

| # | Test | Severity | Action |
|---|------|----------|--------|
| 1 | `test_verify_bearer_token_empty` | **HIGH** — authentication bypass when API key is empty | Fix immediately: add empty-key guard |
| 2 | `test_validate_v2_message_on_v1_header` | **LOW** — test-only, runtime behavior is correct | Fix test assertion or remove as redundant |
