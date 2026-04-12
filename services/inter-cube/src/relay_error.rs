// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY ERROR CODES — Task #27
//
// 20 error codes (12 existing + 8 new). All produce structured
// {type:"error", error, message, offendingType} JSON responses.
// Replaces the TypeScript RELAY_ERROR_CODES in node-watchdog.ts.

use serde_json::{json, Value};

/// WebSocket relay error codes.
///
/// Each variant carries a wire code string, human message, and optional
/// WebSocket close code. The wire codes match the retired TypeScript
/// `RELAY_ERROR_CODES` for backward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayErrorCode {
    // ── Existing 12 (from retired node-watchdog.ts) ─────────
    ErrAuthFailed,
    ErrSignatureInvalid,
    ErrSignatureRequired,
    ErrAuthTimeout,
    ErrRateLimited,
    ErrFrameMalformed,
    ErrFrameTooLarge,
    ErrRelayTargetUnknown,
    ErrRelayQueueFull,
    ErrUnknownMsgType,
    ErrNotAuthenticated,
    ErrCircuitOpen,
    // ── New 8 (Task #27) ────────────────────────────────────
    ErrCapabilityNotNegotiated,
    ErrCapabilityDowngrade,
    ErrTopicBackpressure,
    ErrTopicUnauthorized,
    ErrTopicLimitExceeded,
    ErrTopicReset,
    ErrResyncRateLimited,
    ErrResyncPayloadTooLarge,
}

impl RelayErrorCode {
    /// Wire-format error code string. Matches the retired TypeScript codes.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ErrAuthFailed => "ERR_AUTH_FAILED",
            Self::ErrSignatureInvalid => "ERR_SIGNATURE_INVALID",
            Self::ErrSignatureRequired => "ERR_SIGNATURE_REQUIRED",
            Self::ErrAuthTimeout => "ERR_AUTH_TIMEOUT",
            Self::ErrRateLimited => "ERR_RATE_LIMITED",
            Self::ErrFrameMalformed => "ERR_FRAME_MALFORMED",
            Self::ErrFrameTooLarge => "ERR_FRAME_TOO_LARGE",
            Self::ErrRelayTargetUnknown => "ERR_RELAY_TARGET_UNKNOWN",
            Self::ErrRelayQueueFull => "ERR_RELAY_QUEUE_FULL",
            Self::ErrUnknownMsgType => "ERR_UNKNOWN_MSG_TYPE",
            Self::ErrNotAuthenticated => "ERR_NOT_AUTHENTICATED",
            Self::ErrCircuitOpen => "ERR_CIRCUIT_OPEN",
            Self::ErrCapabilityNotNegotiated => "ERR_CAPABILITY_NOT_NEGOTIATED",
            Self::ErrCapabilityDowngrade => "ERR_CAPABILITY_DOWNGRADE",
            Self::ErrTopicBackpressure => "ERR_TOPIC_BACKPRESSURE",
            Self::ErrTopicUnauthorized => "ERR_TOPIC_UNAUTHORIZED",
            Self::ErrTopicLimitExceeded => "ERR_TOPIC_LIMIT_EXCEEDED",
            Self::ErrTopicReset => "ERR_TOPIC_RESET",
            Self::ErrResyncRateLimited => "ERR_RESYNC_RATE_LIMITED",
            Self::ErrResyncPayloadTooLarge => "ERR_RESYNC_PAYLOAD_TOO_LARGE",
        }
    }

    /// Human-readable error message.
    pub fn message(&self) -> &'static str {
        match self {
            Self::ErrAuthFailed => "Authentication failed \u{2014} address not registered or publicKey mismatch",
            Self::ErrSignatureInvalid => "Challenge signature verification failed \u{2014} private key proof required",
            Self::ErrSignatureRequired => "Challenge signature required \u{2014} upgrade client to v0.3.0+",
            Self::ErrAuthTimeout => "Authentication timeout \u{2014} must authenticate within 10 seconds",
            Self::ErrRateLimited => "Rate limit exceeded \u{2014} too many requests",
            Self::ErrFrameMalformed => "Malformed frame \u{2014} invalid JSON or structure",
            Self::ErrFrameTooLarge => "Frame too large \u{2014} exceeds maximum allowed size",
            Self::ErrRelayTargetUnknown => "Relay target unknown \u{2014} destination address not connected",
            Self::ErrRelayQueueFull => "Relay queue full \u{2014} destination queue at capacity",
            Self::ErrUnknownMsgType => "Unknown message type",
            Self::ErrNotAuthenticated => "Must authenticate first",
            Self::ErrCircuitOpen => "Service circuit breaker is open \u{2014} requests degraded",
            Self::ErrCapabilityNotNegotiated => "Feature requires a capability not negotiated during auth",
            Self::ErrCapabilityDowngrade => "Capability downgrade rejected \u{2014} reconnect with previous capability set or contact admin",
            Self::ErrTopicBackpressure => "Topic queue at capacity \u{2014} backpressure applied, retry later",
            Self::ErrTopicUnauthorized => "Not authorized for this topic operation",
            Self::ErrTopicLimitExceeded => "Topic cardinality limit exceeded \u{2014} per-connection or per-server maximum reached",
            Self::ErrTopicReset => "Topic was garbage-collected and recreated \u{2014} epoch changed, resync required",
            Self::ErrResyncRateLimited => "Resync rate limit exceeded \u{2014} max 3 attempts per minute",
            Self::ErrResyncPayloadTooLarge => "Resync bitmap exceeds 8KB hard cap",
        }
    }

    /// Optional WebSocket close code. `None` means don't close the connection.
    pub fn ws_close(&self) -> Option<u16> {
        match self {
            Self::ErrAuthFailed => Some(1008),
            Self::ErrSignatureInvalid => Some(1008),
            Self::ErrSignatureRequired => Some(1008),
            Self::ErrAuthTimeout => Some(1008),
            Self::ErrRateLimited => Some(1008),
            Self::ErrFrameMalformed => Some(1003),
            Self::ErrFrameTooLarge => Some(1009),
            Self::ErrCapabilityDowngrade => Some(1008),
            _ => None,
        }
    }
}

/// Produce a structured error response JSON value.
///
/// Format: `{type:"error", error:"ERR_...", message:"...", offendingType:"..."}`
/// Matches the retired TypeScript `makeErrorResponse()` output exactly.
pub fn make_error_response(code: RelayErrorCode, offending_type: Option<&str>) -> Value {
    let mut resp = json!({
        "type": "error",
        "error": code.code(),
        "message": code.message(),
    });
    if let Some(ot) = offending_type {
        resp["offendingType"] = json!(ot);
    }
    resp
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_20_codes_have_distinct_wire_codes() {
        let all = [
            RelayErrorCode::ErrAuthFailed,
            RelayErrorCode::ErrSignatureInvalid,
            RelayErrorCode::ErrSignatureRequired,
            RelayErrorCode::ErrAuthTimeout,
            RelayErrorCode::ErrRateLimited,
            RelayErrorCode::ErrFrameMalformed,
            RelayErrorCode::ErrFrameTooLarge,
            RelayErrorCode::ErrRelayTargetUnknown,
            RelayErrorCode::ErrRelayQueueFull,
            RelayErrorCode::ErrUnknownMsgType,
            RelayErrorCode::ErrNotAuthenticated,
            RelayErrorCode::ErrCircuitOpen,
            RelayErrorCode::ErrCapabilityNotNegotiated,
            RelayErrorCode::ErrCapabilityDowngrade,
            RelayErrorCode::ErrTopicBackpressure,
            RelayErrorCode::ErrTopicUnauthorized,
            RelayErrorCode::ErrTopicLimitExceeded,
            RelayErrorCode::ErrTopicReset,
            RelayErrorCode::ErrResyncRateLimited,
            RelayErrorCode::ErrResyncPayloadTooLarge,
        ];
        assert_eq!(all.len(), 20);
        let mut codes: Vec<&str> = all.iter().map(|e| e.code()).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), 20, "All 20 error codes must be distinct");
    }

    #[test]
    fn test_error_response_format() {
        let resp = make_error_response(RelayErrorCode::ErrAuthFailed, Some("auth"));
        assert_eq!(resp["type"], "error");
        assert_eq!(resp["error"], "ERR_AUTH_FAILED");
        assert!(resp["message"].as_str().unwrap().contains("Authentication failed"));
        assert_eq!(resp["offendingType"], "auth");
    }

    #[test]
    fn test_error_response_no_offending_type() {
        let resp = make_error_response(RelayErrorCode::ErrCircuitOpen, None);
        assert_eq!(resp["type"], "error");
        assert_eq!(resp["error"], "ERR_CIRCUIT_OPEN");
        assert!(resp.get("offendingType").is_none());
    }

    #[test]
    fn test_ws_close_codes() {
        assert_eq!(RelayErrorCode::ErrAuthFailed.ws_close(), Some(1008));
        assert_eq!(RelayErrorCode::ErrFrameMalformed.ws_close(), Some(1003));
        assert_eq!(RelayErrorCode::ErrFrameTooLarge.ws_close(), Some(1009));
        assert_eq!(RelayErrorCode::ErrRelayTargetUnknown.ws_close(), None);
        assert_eq!(RelayErrorCode::ErrTopicBackpressure.ws_close(), None);
    }

    #[test]
    fn test_all_codes_start_with_err() {
        let all = [
            RelayErrorCode::ErrAuthFailed, RelayErrorCode::ErrSignatureInvalid,
            RelayErrorCode::ErrSignatureRequired, RelayErrorCode::ErrAuthTimeout,
            RelayErrorCode::ErrRateLimited, RelayErrorCode::ErrFrameMalformed,
            RelayErrorCode::ErrFrameTooLarge, RelayErrorCode::ErrRelayTargetUnknown,
            RelayErrorCode::ErrRelayQueueFull, RelayErrorCode::ErrUnknownMsgType,
            RelayErrorCode::ErrNotAuthenticated, RelayErrorCode::ErrCircuitOpen,
            RelayErrorCode::ErrCapabilityNotNegotiated, RelayErrorCode::ErrCapabilityDowngrade,
            RelayErrorCode::ErrTopicBackpressure, RelayErrorCode::ErrTopicUnauthorized,
            RelayErrorCode::ErrTopicLimitExceeded, RelayErrorCode::ErrTopicReset,
            RelayErrorCode::ErrResyncRateLimited, RelayErrorCode::ErrResyncPayloadTooLarge,
        ];
        for code in &all {
            assert!(code.code().starts_with("ERR_"), "{} must start with ERR_", code.code());
            assert!(!code.message().is_empty(), "{} must have a message", code.code());
        }
    }
}
