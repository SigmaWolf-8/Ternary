// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY FRAME VALIDATION & REP C ENCODING — Task #27
//
// Frame validation contract: every control frame type has defined max
// size, required fields with types, and type-checking rules.
//
// Rep C encoding: wire format is strings (backward compatible).
// Internally, frame types map to Rep C integers {1, 2, 3}. Zero is
// excluded by Rep C definition — has_forgery() catches corruption
// before any handler runs. Called directly as a crate function.

use serde_json::Value;
use ternary_math::gf3_algebra;
use ternary_math::trit_int::TritInt;

// ═══════════════════════════════════════════════════════════════════════
// REP C FRAME TYPE ENCODING — TritInt above the gate
//
// Application code uses TritInt for all trit values. The u8 kernel
// boundary (gf3_algebra::has_forgery(&[u8])) is crossed only at the
// validation call site — never stored as u8 above the gate.
// ═══════════════════════════════════════════════════════════════════════

/// Rep C TritInt constants for control notification frame types.
/// Zero is excluded from Rep C by definition.
///
/// - TritInt(1) = tombstone (global queue eviction)
/// - TritInt(2) = topic_reset (GC discontinuity)
/// - TritInt(3) = topic_revoked (permission revoked)

/// Parse a wire-format string frame type to Rep C TritInt.
///
/// Returns TritInt::zero() for unknown types — provably corrupt in Rep C.
/// Caller MUST check via `is_frame_type_corrupt()` before dispatch.
pub fn wire_type_to_rep_c(wire_type: &str) -> TritInt {
    match wire_type {
        "tombstone" => TritInt::from_u64(1),
        "topic_reset" => TritInt::from_u64(2),
        "topic_revoked" => TritInt::from_u64(3),
        _ => TritInt::zero(), // Zero = provably corrupt in Rep C
    }
}

/// Inverse: Rep C TritInt → wire-format string.
pub fn rep_c_to_wire_type(rep_c: &TritInt) -> Option<&'static str> {
    match rep_c.to_decimal() {
        1 => Some("tombstone"),
        2 => Some("topic_reset"),
        3 => Some("topic_revoked"),
        _ => None,
    }
}

/// Validate that a Rep C frame type TritInt is not corrupt.
///
/// Crosses the gate boundary: TritInt → u8 → has_forgery() kernel call.
/// Uses gf3_algebra::has_forgery() directly — product mod 7, division-free.
/// A zero value is provably corrupt: bit-flip, uninitialized read,
/// or malformed injection.
///
/// Returns true if the value is CORRUPT (contains zero).
pub fn is_frame_type_corrupt(rep_c: &TritInt) -> bool {
    // Gate crossing: TritInt → u8 for kernel-level forgery check
    let raw = rep_c.to_decimal() as u8;
    gf3_algebra::has_forgery(&[raw])
}

// ═══════════════════════════════════════════════════════════════════════
// FRAME VALIDATION CONTRACT
// ═══════════════════════════════════════════════════════════════════════

/// Maximum frame size in bytes (64KB default).
pub const MAX_FRAME_SIZE: usize = 65_536;

/// Maximum resync bitmap size in bytes (8KB hard cap).
pub const MAX_RESYNC_BITMAP_SIZE: usize = 8_192;

/// Required field definition for frame validation.
#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub name: &'static str,
    pub expected_type: JsonType,
}

/// JSON value type for field validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonType {
    String,
    Number,
    Object,
    Array,
    Bool,
}

impl JsonType {
    pub fn matches(&self, value: &Value) -> bool {
        match self {
            Self::String => value.is_string(),
            Self::Number => value.is_number(),
            Self::Object => value.is_object(),
            Self::Array => value.is_array(),
            Self::Bool => value.is_boolean(),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Object => "object",
            Self::Array => "array",
            Self::Bool => "boolean",
        }
    }
}

/// Frame schema: required fields + max size.
pub struct FrameSchema {
    pub frame_type: &'static str,
    pub required: &'static [FieldSpec],
    pub max_size: usize,
}

// ── Schema definitions ──────────────────────────────────────────────

pub static TOMBSTONE_SCHEMA: FrameSchema = FrameSchema {
    frame_type: "tombstone",
    required: &[
        FieldSpec { name: "type", expected_type: JsonType::String },
        FieldSpec { name: "resyncCount", expected_type: JsonType::Number },
        FieldSpec { name: "suggestedResyncAfterMs", expected_type: JsonType::Number },
        FieldSpec { name: "topicSeqs", expected_type: JsonType::Object },
        FieldSpec { name: "gapSizeEstimate", expected_type: JsonType::Number },
    ],
    max_size: MAX_FRAME_SIZE,
};

pub static TOPIC_RESET_SCHEMA: FrameSchema = FrameSchema {
    frame_type: "topic_reset",
    required: &[
        FieldSpec { name: "type", expected_type: JsonType::String },
        FieldSpec { name: "topic", expected_type: JsonType::String },
        FieldSpec { name: "oldEpoch", expected_type: JsonType::Number },
        FieldSpec { name: "newEpoch", expected_type: JsonType::Number },
        FieldSpec { name: "currentSeq", expected_type: JsonType::Number },
    ],
    max_size: MAX_FRAME_SIZE,
};

pub static TOPIC_REVOKED_SCHEMA: FrameSchema = FrameSchema {
    frame_type: "topic_revoked",
    required: &[
        FieldSpec { name: "type", expected_type: JsonType::String },
        FieldSpec { name: "topic", expected_type: JsonType::String },
        FieldSpec { name: "reason", expected_type: JsonType::String },
        FieldSpec { name: "lastDeliveredSeq", expected_type: JsonType::Number },
        FieldSpec { name: "topicEpoch", expected_type: JsonType::Number },
    ],
    max_size: MAX_FRAME_SIZE,
};

pub static HEARTBEAT_INTERVAL_CHANGED_SCHEMA: FrameSchema = FrameSchema {
    frame_type: "heartbeat_interval_changed",
    required: &[
        FieldSpec { name: "type", expected_type: JsonType::String },
        FieldSpec { name: "heartbeatIntervalMs", expected_type: JsonType::Number },
    ],
    max_size: MAX_FRAME_SIZE,
};

pub static CIRCUIT_OPEN_SCHEMA: FrameSchema = FrameSchema {
    frame_type: "circuit_open",
    required: &[
        FieldSpec { name: "type", expected_type: JsonType::String },
        FieldSpec { name: "breaker", expected_type: JsonType::String },
        FieldSpec { name: "ts", expected_type: JsonType::Number },
    ],
    max_size: MAX_FRAME_SIZE,
};

pub static GO_AWAY_SCHEMA: FrameSchema = FrameSchema {
    frame_type: "go-away",
    required: &[
        FieldSpec { name: "type", expected_type: JsonType::String },
        FieldSpec { name: "reason", expected_type: JsonType::String },
        FieldSpec { name: "reconnectAfterMs", expected_type: JsonType::Number },
        FieldSpec { name: "ts", expected_type: JsonType::Number },
    ],
    max_size: MAX_FRAME_SIZE,
};

/// Get the schema for a frame type, if one exists.
pub fn schema_for(frame_type: &str) -> Option<&'static FrameSchema> {
    match frame_type {
        "tombstone" => Some(&TOMBSTONE_SCHEMA),
        "topic_reset" => Some(&TOPIC_RESET_SCHEMA),
        "topic_revoked" => Some(&TOPIC_REVOKED_SCHEMA),
        "heartbeat_interval_changed" => Some(&HEARTBEAT_INTERVAL_CHANGED_SCHEMA),
        "circuit_open" => Some(&CIRCUIT_OPEN_SCHEMA),
        "go-away" => Some(&GO_AWAY_SCHEMA),
        _ => None,
    }
}

/// Validate a parsed frame against its schema.
///
/// Returns Ok(()) if valid, Err(description) if invalid.
pub fn validate_control_frame(frame: &Value) -> Result<(), String> {
    let frame_type = frame.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| "missing 'type' field".to_string())?;

    let schema = match schema_for(frame_type) {
        Some(s) => s,
        None => return Ok(()), // Not a control frame — no schema
    };

    for field in schema.required {
        match frame.get(field.name) {
            None => {
                return Err(format!(
                    "missing required field '{}' for {}",
                    field.name, frame_type
                ));
            }
            Some(val) => {
                if !field.expected_type.matches(val) {
                    return Err(format!(
                        "field '{}' expected {}, got {} for {}",
                        field.name, field.expected_type.as_str(),
                        json_type_name(val), frame_type
                    ));
                }
            }
        }
    }

    Ok(())
}

fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Rep C encoding ──────────────────────────────────────────

    #[test]
    fn test_wire_to_rep_c() {
        assert_eq!(wire_type_to_rep_c("tombstone").to_decimal(), 1);
        assert_eq!(wire_type_to_rep_c("topic_reset").to_decimal(), 2);
        assert_eq!(wire_type_to_rep_c("topic_revoked").to_decimal(), 3);
        assert_eq!(wire_type_to_rep_c("unknown").to_decimal(), 0);
    }

    #[test]
    fn test_rep_c_to_wire() {
        assert_eq!(rep_c_to_wire_type(&TritInt::from_u64(1)), Some("tombstone"));
        assert_eq!(rep_c_to_wire_type(&TritInt::from_u64(2)), Some("topic_reset"));
        assert_eq!(rep_c_to_wire_type(&TritInt::from_u64(3)), Some("topic_revoked"));
        assert_eq!(rep_c_to_wire_type(&TritInt::zero()), None);
        assert_eq!(rep_c_to_wire_type(&TritInt::from_u64(4)), None);
    }

    #[test]
    fn test_zero_is_corrupt() {
        assert!(is_frame_type_corrupt(&TritInt::zero()), "Zero must be corrupt in Rep C");
        assert!(!is_frame_type_corrupt(&TritInt::from_u64(1)));
        assert!(!is_frame_type_corrupt(&TritInt::from_u64(2)));
        assert!(!is_frame_type_corrupt(&TritInt::from_u64(3)));
    }

    #[test]
    fn test_has_forgery_catches_zero() {
        // Direct call to gf3_algebra::has_forgery — the kernel boundary
        assert!(gf3_algebra::has_forgery(&[1, 0, 3]));
        assert!(!gf3_algebra::has_forgery(&[1, 2, 3]));
    }

    #[test]
    fn test_roundtrip_all_types() {
        for wire in &["tombstone", "topic_reset", "topic_revoked"] {
            let rep_c = wire_type_to_rep_c(wire);
            assert!(!is_frame_type_corrupt(&rep_c));
            assert_eq!(rep_c_to_wire_type(&rep_c), Some(*wire));
        }
    }

    // ── Frame validation ────────────────────────────────────────

    #[test]
    fn test_valid_tombstone() {
        let frame = json!({
            "type": "tombstone",
            "resyncCount": 1,
            "suggestedResyncAfterMs": 3500,
            "topicSeqs": {},
            "gapSizeEstimate": 42
        });
        assert!(validate_control_frame(&frame).is_ok());
    }

    #[test]
    fn test_tombstone_missing_field() {
        let frame = json!({
            "type": "tombstone",
            "resyncCount": 1,
            // missing suggestedResyncAfterMs
            "topicSeqs": {},
            "gapSizeEstimate": 42
        });
        let err = validate_control_frame(&frame).unwrap_err();
        assert!(err.contains("suggestedResyncAfterMs"));
    }

    #[test]
    fn test_tombstone_wrong_type() {
        let frame = json!({
            "type": "tombstone",
            "resyncCount": "not a number",
            "suggestedResyncAfterMs": 3500,
            "topicSeqs": {},
            "gapSizeEstimate": 42
        });
        let err = validate_control_frame(&frame).unwrap_err();
        assert!(err.contains("resyncCount"));
        assert!(err.contains("expected number"));
    }

    #[test]
    fn test_valid_topic_reset() {
        let frame = json!({
            "type": "topic_reset",
            "topic": "sensor-data",
            "oldEpoch": 1000,
            "newEpoch": 2000,
            "currentSeq": 1
        });
        assert!(validate_control_frame(&frame).is_ok());
    }

    #[test]
    fn test_valid_topic_revoked() {
        let frame = json!({
            "type": "topic_revoked",
            "topic": "sensor-data",
            "reason": "permission_revoked",
            "lastDeliveredSeq": 47,
            "topicEpoch": 1000
        });
        assert!(validate_control_frame(&frame).is_ok());
    }

    #[test]
    fn test_unknown_type_passes() {
        let frame = json!({"type": "relay", "to": "1.2.3"});
        assert!(validate_control_frame(&frame).is_ok());
    }

    #[test]
    fn test_missing_type_field() {
        let frame = json!({"data": "something"});
        assert!(validate_control_frame(&frame).is_err());
    }

    #[test]
    fn test_valid_go_away() {
        let frame = json!({
            "type": "go-away",
            "reason": "server_shutdown",
            "reconnectAfterMs": 3500,
            "ts": 1712800000000u64
        });
        assert!(validate_control_frame(&frame).is_ok());
    }
}
