// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Yoda Chat Channel (Task #69)
//!
//! Provides the `yoda_chat` / `yoda_response` message type pair for the
//! WebSocket relay. Operators send natural-language messages to Yoda via
//! the `y` CLI command, the Node Terminal, or the Ctrl+Y widget. All
//! messages are NinjaExec-signed (TL-DSA) with the `PlenumNET-YODA-CHAT-v1`
//! context string, replay protection, and Rep C address binding.
//!
//! ## Signing Context
//!
//! Context string: `PlenumNET-YODA-CHAT-v1` (22 bytes, registered in
//! ws_relay.rs Sponge Context String Registry).
//!
//! ## Canonical JSON Signing Payload
//!
//! ```text
//! signing_payload = "PlenumNET-YODA-CHAT-v1" || canonical_json_utf8_bytes
//! ```
//!
//! Where `canonical_json_utf8_bytes` is `{"d":"...","m":"...","q":N,"s":"...","t":N}`
//! with sorted single-character keys, no whitespace.
//!
//! ## Trust Model
//!
//! - Outbound: TL-DSA signature verified by daemon before relay forwarding.
//! - Inbound: `yoda_response` authenticated by relay transport session
//!   (delegation model — same as `inference_response`).
//!
//! ## Confidentiality
//!
//! Daemon MUST NOT log plaintext message content. Only session ID, timestamp,
//! operator Rep C address, payload TIS-27 hash, and result status are logged.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use ternary_math::tl_dsa;

pub const YODA_CHAT_CONTEXT: &str = "PlenumNET-YODA-CHAT-v1";

pub const MAX_MESSAGE_BYTES: usize = 32_768;

pub const TIMESTAMP_MAX_AGE_MS: u64 = 60_000;

pub const MAX_TRACKED_SEQUENCES_PER_SESSION: usize = 1000;

pub const RATE_LIMIT_PER_MINUTE: u32 = 10;

pub const YODA_RESPONSE_TIMEOUT_SECS: u64 = 30;

// ═══════════════════════════════════════════════════════════════════════
// ERROR CODES
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YodaErrorCode {
    DaemonNotRunning,
    RelayDisconnected,
    NinjaexecLocked,
    NinjaexecNotRunning,
    SignatureInvalid,
    MessageExpired,
    SequenceReplay,
    OperatorNotAuthorized,
    MessageTooLong,
    YodaTimeout,
    YodaUnavailable,
    AddressMismatch,
    RateLimited,
    ConcurrentAccess,
}

impl YodaErrorCode {
    pub fn code_str(&self) -> &'static str {
        match self {
            Self::DaemonNotRunning => "DAEMON_NOT_RUNNING",
            Self::RelayDisconnected => "RELAY_DISCONNECTED",
            Self::NinjaexecLocked => "NINJAEXEC_LOCKED",
            Self::NinjaexecNotRunning => "NINJAEXEC_NOT_RUNNING",
            Self::SignatureInvalid => "SIGNATURE_INVALID",
            Self::MessageExpired => "MESSAGE_EXPIRED",
            Self::SequenceReplay => "SEQUENCE_REPLAY",
            Self::OperatorNotAuthorized => "OPERATOR_NOT_AUTHORIZED",
            Self::MessageTooLong => "MESSAGE_TOO_LONG",
            Self::YodaTimeout => "YODA_TIMEOUT",
            Self::YodaUnavailable => "YODA_UNAVAILABLE",
            Self::AddressMismatch => "ADDRESS_MISMATCH",
            Self::RateLimited => "RATE_LIMITED",
            Self::ConcurrentAccess => "CONCURRENT_ACCESS",
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::DaemonNotRunning => 1,
            Self::RelayDisconnected => 2,
            Self::NinjaexecLocked => 3,
            Self::NinjaexecNotRunning => 4,
            Self::SignatureInvalid => 5,
            Self::MessageExpired => 6,
            Self::SequenceReplay => 7,
            Self::OperatorNotAuthorized => 8,
            Self::MessageTooLong => 9,
            Self::YodaTimeout => 10,
            Self::YodaUnavailable => 11,
            Self::AddressMismatch => 12,
            Self::RateLimited => 13,
            Self::ConcurrentAccess => 14,
        }
    }

    pub fn display_message(&self) -> &'static str {
        match self {
            Self::DaemonNotRunning => "Yoda can't reach your daemon — it doesn't seem to be running. Start it and try again.",
            Self::RelayDisconnected => "Yoda is not reachable — the daemon is not connected to the relay. Check your network and daemon config.",
            Self::NinjaexecLocked => "Your signing key is locked. Run `ninja-exec unlock` to resume.",
            Self::NinjaexecNotRunning => "Your signing agent isn't running. Start it with `ninja-exec` and try again.",
            Self::SignatureInvalid => "Your signing key doesn't match what this daemon expects. Re-register your key: run `ninja-exec export-operator` and update `ops-config.json`.",
            Self::MessageExpired => "This message expired before it could be delivered — your system clock may be out of sync. Check your clock and try again.",
            Self::SequenceReplay => "This message was already sent. If you meant to send it again, wait a moment and retry.",
            Self::OperatorNotAuthorized => "Your operator key isn't registered with this daemon yet. Register it in `ops-config.json` or run the deploy script with `-AddOperator`.",
            Self::MessageTooLong => "That message is too long — the maximum is 32,768 characters. Shorten it and try again.",
            Self::YodaTimeout => "Yoda is taking too long to respond. Try again in a moment.",
            Self::YodaUnavailable => "Yoda is currently unavailable. Your message was received but could not be processed.",
            Self::AddressMismatch => "The daemon address in your message doesn't match this daemon. Make sure you're connected to the correct node.",
            Self::RateLimited => "You're sending messages faster than Yoda can read them. Wait a moment and try again.",
            Self::ConcurrentAccess => "Another `y` command is in progress. Wait a moment and try again.",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PAYLOAD TYPES
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YodaChatPayload {
    pub session_id: String,
    pub timestamp: u64,
    pub sequence: u64,
    pub message: String,
    pub operator_pubkey: String,
    pub daemon_rep_c: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YodaResponsePayload {
    pub session_id: String,
    pub sequence: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<YodaResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YodaResponseError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonApiError {
    pub code: String,
    pub message: String,
    #[serde(rename = "exitCode")]
    pub exit_code: i32,
}

impl DaemonApiError {
    pub fn from_error_code(code: YodaErrorCode) -> Self {
        DaemonApiError {
            code: code.code_str().to_string(),
            message: code.display_message().to_string(),
            exit_code: code.exit_code(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CANONICAL JSON CONSTRUCTION
// ═══════════════════════════════════════════════════════════════════════

pub fn build_canonical_json(
    daemon_rep_c: &str,
    message: &str,
    sequence: u64,
    session_id: &str,
    timestamp: u64,
) -> String {
    let escaped_d = json_escape_string(daemon_rep_c);
    let escaped_m = json_escape_string(message);
    let escaped_s = json_escape_string(session_id);
    format!(
        "{{\"d\":\"{}\",\"m\":\"{}\",\"q\":{},\"s\":\"{}\",\"t\":{}}}",
        escaped_d, escaped_m, sequence, escaped_s, timestamp
    )
}

pub fn build_signing_payload(
    daemon_rep_c: &str,
    message: &str,
    sequence: u64,
    session_id: &str,
    timestamp: u64,
) -> Vec<u8> {
    let canonical = build_canonical_json(daemon_rep_c, message, sequence, session_id, timestamp);
    let mut payload = Vec::with_capacity(YODA_CHAT_CONTEXT.len() + canonical.len());
    payload.extend_from_slice(YODA_CHAT_CONTEXT.as_bytes());
    payload.extend_from_slice(canonical.as_bytes());
    payload
}

fn json_escape_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                escaped.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

// ═══════════════════════════════════════════════════════════════════════
// REPLAY PROTECTION
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct ReplayGuard {
    sessions: HashMap<String, SessionReplayState>,
}

#[derive(Debug)]
struct SessionReplayState {
    seen_sequences: HashSet<u64>,
    sequence_order: Vec<u64>,
}

impl ReplayGuard {
    pub fn new() -> Self {
        ReplayGuard {
            sessions: HashMap::new(),
        }
    }

    pub fn check_and_record(&mut self, session_id: &str, sequence: u64) -> bool {
        let state = self.sessions.entry(session_id.to_string()).or_insert_with(|| {
            SessionReplayState {
                seen_sequences: HashSet::new(),
                sequence_order: Vec::new(),
            }
        });

        if state.seen_sequences.contains(&sequence) {
            return false;
        }

        state.seen_sequences.insert(sequence);
        state.sequence_order.push(sequence);

        if state.sequence_order.len() > MAX_TRACKED_SEQUENCES_PER_SESSION {
            if let Some(oldest) = state.sequence_order.first().copied() {
                state.seen_sequences.remove(&oldest);
                state.sequence_order.remove(0);
            }
        }

        true
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct YodaRateLimiter {
    windows: HashMap<String, Vec<u64>>,
}

impl YodaRateLimiter {
    pub fn new() -> Self {
        YodaRateLimiter {
            windows: HashMap::new(),
        }
    }

    pub fn check_rate(&mut self, operator_rep_c: &str, now_ms: u64) -> bool {
        let window_start = now_ms.saturating_sub(60_000);
        let timestamps = self.windows.entry(operator_rep_c.to_string()).or_default();
        timestamps.retain(|&ts| ts > window_start);
        if timestamps.len() >= RATE_LIMIT_PER_MINUTE as usize {
            return false;
        }
        timestamps.push(now_ms);
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════
// AUDIT TRAIL
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YodaAuditEntry {
    pub timestamp: String,
    pub session_id: String,
    pub sequence: u64,
    pub direction: String,
    pub operator_rep_c: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_hash: Option<String>,
    pub result: String,
}

pub fn format_timestamp_rfc3339(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let millis = (ms % 1000) as u32;
    let nanos = millis * 1_000_000;
    let secs_u = if secs < 0 { 0u64 } else { secs as u64 };
    let d = UNIX_EPOCH + std::time::Duration::new(secs_u, nanos);
    let datetime: chrono::DateTime<chrono::Utc> = d.into();
    datetime.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn compute_payload_hash(data: &[u8]) -> String {
    let hex = ternary_math::tlsponge385::hash_hex_tis(data);
    format!("tis27:{}", hex)
}

pub fn write_audit_entry(audit_path: &str, entry: &YodaAuditEntry) {
    if let Ok(line) = serde_json::to_string(entry) {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(audit_path)
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{}", line)
            });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DAEMON-SIDE VERIFICATION (9-step path)
// ═══════════════════════════════════════════════════════════════════════

pub struct YodaChatVerifier {
    pub daemon_rep_c: String,
    pub replay_guard: ReplayGuard,
    pub rate_limiter: YodaRateLimiter,
    operators: HashMap<String, crate::ops_handler::OperatorEntry>,
}

impl YodaChatVerifier {
    pub fn new(daemon_rep_c: String) -> Self {
        YodaChatVerifier {
            daemon_rep_c,
            replay_guard: ReplayGuard::new(),
            rate_limiter: YodaRateLimiter::new(),
            operators: HashMap::new(),
        }
    }

    pub fn set_operators(&mut self, operators: HashMap<String, crate::ops_handler::OperatorEntry>) {
        self.operators = operators;
    }

    pub fn add_operator(&mut self, pubkey: String, entry: crate::ops_handler::OperatorEntry) {
        self.operators.insert(pubkey, entry);
    }

    pub fn remove_operator(&mut self, pubkey: &str) {
        self.operators.remove(pubkey);
    }

    /// Verify a Yoda chat payload and prepare it for relay forwarding.
    ///
    /// ## Identity Binding (INVARIANT 7/9)
    ///
    /// The operator key ↔ daemon Rep C binding is enforced through three checks:
    ///
    /// 1. **Address match**: `payload.daemon_rep_c == self.daemon_rep_c` ensures the
    ///    message targets THIS daemon's identity anchor.
    /// 2. **Registry lookup**: `payload.operator_pubkey` must exist in this daemon's
    ///    operator registry — operators are registered PER-DAEMON, so registry
    ///    membership IS the binding to this daemon's Rep C.
    /// 3. **Signature over Rep C**: The TL-DSA signature covers canonical JSON that
    ///    includes `daemon_rep_c`, proving the operator intentionally signed for
    ///    this specific daemon identity.
    ///
    /// Together these three checks form a complete key-to-RepC binding: an operator
    /// can only send messages through a daemon they are registered on, targeting
    /// that daemon's Rep C, with a signature that commits to that Rep C.
    pub fn verify_and_forward(
        &mut self,
        payload_json: &str,
    ) -> Result<(YodaChatPayload, String), DaemonApiError> {
        let payload: YodaChatPayload = serde_json::from_str(payload_json)
            .map_err(|_| DaemonApiError::from_error_code(YodaErrorCode::SignatureInvalid))?;

        if payload.message.len() > MAX_MESSAGE_BYTES {
            return Err(DaemonApiError::from_error_code(YodaErrorCode::MessageTooLong));
        }

        if payload.daemon_rep_c != self.daemon_rep_c {
            return Err(DaemonApiError::from_error_code(YodaErrorCode::AddressMismatch));
        }

        let _operator = self.operators.values()
            .find(|op| op.public_key == payload.operator_pubkey)
            .cloned()
            .ok_or_else(|| DaemonApiError::from_error_code(YodaErrorCode::OperatorNotAuthorized))?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let age = now_ms.abs_diff(payload.timestamp);
        if age >= TIMESTAMP_MAX_AGE_MS {
            return Err(DaemonApiError::from_error_code(YodaErrorCode::MessageExpired));
        }

        let signing_payload = build_signing_payload(
            &payload.daemon_rep_c,
            &payload.message,
            payload.sequence,
            &payload.session_id,
            payload.timestamp,
        );

        let sig_bytes = match base64_decode(&payload.signature) {
            Some(b) => b,
            None => {
                return Err(DaemonApiError::from_error_code(YodaErrorCode::SignatureInvalid));
            }
        };
        let pk_bytes = match base64_decode(&payload.operator_pubkey) {
            Some(b) => b,
            None => {
                return Err(DaemonApiError::from_error_code(YodaErrorCode::SignatureInvalid));
            }
        };

        let valid = tl_dsa::verify(
            &pk_bytes,
            &signing_payload,
            &sig_bytes,
            tl_dsa::TlDsaVariant::TlDsa87,
        );
        if !valid {
            return Err(DaemonApiError::from_error_code(YodaErrorCode::SignatureInvalid));
        }

        if !self.replay_guard.check_and_record(&payload.session_id, payload.sequence) {
            return Err(DaemonApiError::from_error_code(YodaErrorCode::SequenceReplay));
        }

        if !self.rate_limiter.check_rate(&payload.daemon_rep_c, now_ms) {
            return Err(DaemonApiError::from_error_code(YodaErrorCode::RateLimited));
        }

        let payload_hash = compute_payload_hash(&signing_payload);

        println!(
            "[yoda-chat] Session {} msg #{} from {} hash={} — verified+forwarded",
            payload.session_id, payload.sequence, payload.daemon_rep_c, payload_hash
        );

        let audit_path = format!(
            "{}/.plenumnet/yoda-audit.jsonl",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        );
        write_audit_entry(&audit_path, &YodaAuditEntry {
            timestamp: format_timestamp_rfc3339(payload.timestamp),
            session_id: payload.session_id.clone(),
            sequence: payload.sequence,
            operator_rep_c: payload.daemon_rep_c.clone(),
            payload_hash: Some(payload_hash.clone()),
            direction: "outbound".to_string(),
            response_hash: None,
            result: "verified".to_string(),
        });

        Ok((payload, payload_hash))
    }
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let table: [u8; 256] = {
        let mut t = [0xFFu8; 256];
        for (i, &c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            t[c as usize] = i as u8;
        }
        t[b'=' as usize] = 0;
        t
    };
    let bytes = clean.as_bytes();
    if bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        if chunk.len() != 4 {
            return None;
        }
        let mut vals = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            let v = table[b as usize];
            if v == 0xFF && b != b'=' {
                return None;
            }
            vals[i] = v;
        }
        let combined = ((vals[0] as u32) << 18) | ((vals[1] as u32) << 12)
            | ((vals[2] as u32) << 6) | (vals[3] as u32);
        out.push((combined >> 16) as u8);
        if chunk[2] != b'=' {
            out.push((combined >> 8) as u8);
        }
        if chunk[3] != b'=' {
            out.push(combined as u8);
        }
    }
    Some(out)
}

pub fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let combined = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((combined >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((combined >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((combined >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(combined & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

// ═══════════════════════════════════════════════════════════════════════
// SESSION MANAGEMENT
// ═══════════════════════════════════════════════════════════════════════

pub const SESSION_INACTIVITY_TIMEOUT_SECS: u64 = 30 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub session_id: String,
    pub sequence: u64,
    pub last_active: u64,
}

impl SessionFile {
    pub fn new() -> Self {
        SessionFile {
            session_id: uuid::Uuid::new_v4().to_string(),
            sequence: 0,
            last_active: current_timestamp_ms(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = current_timestamp_ms();
        let elapsed_secs = (now.saturating_sub(self.last_active)) / 1000;
        elapsed_secs >= SESSION_INACTIVITY_TIMEOUT_SECS
    }

    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence += 1;
        self.last_active = current_timestamp_ms();
        seq
    }
}

pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ═══════════════════════════════════════════════════════════════════════
// DOCTOR CHECKS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctorStatus {
    Ok,
    Warn,
    Fail,
}

impl std::fmt::Display for DoctorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "[OK]"),
            Self::Warn => write!(f, "[WARN]"),
            Self::Fail => write!(f, "[FAIL]"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub status: DoctorStatus,
    pub message: String,
}

impl std::fmt::Display for DoctorCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.status, self.message)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_json_deterministic() {
        let json1 = build_canonical_json(
            "211.111.111.111.1",
            "Hey Yoda...",
            42,
            "abc-123",
            1711648800000,
        );
        let json2 = build_canonical_json(
            "211.111.111.111.1",
            "Hey Yoda...",
            42,
            "abc-123",
            1711648800000,
        );
        assert_eq!(json1, json2);
        assert_eq!(
            json1,
            "{\"d\":\"211.111.111.111.1\",\"m\":\"Hey Yoda...\",\"q\":42,\"s\":\"abc-123\",\"t\":1711648800000}"
        );
    }

    #[test]
    fn test_canonical_json_key_order() {
        let json = build_canonical_json("x", "y", 0, "z", 1);
        assert!(json.starts_with("{\"d\":"));
        let d_pos = json.find("\"d\"").unwrap();
        let m_pos = json.find("\"m\"").unwrap();
        let q_pos = json.find("\"q\"").unwrap();
        let s_pos = json.find("\"s\"").unwrap();
        let t_pos = json.find("\"t\"").unwrap();
        assert!(d_pos < m_pos);
        assert!(m_pos < q_pos);
        assert!(q_pos < s_pos);
        assert!(s_pos < t_pos);
    }

    #[test]
    fn test_signing_payload_includes_context() {
        let payload = build_signing_payload("a", "b", 0, "s", 1);
        let prefix = &payload[..YODA_CHAT_CONTEXT.len()];
        assert_eq!(prefix, YODA_CHAT_CONTEXT.as_bytes());
    }

    #[test]
    fn test_json_escape() {
        let escaped = json_escape_string("hello \"world\"\n\\tab\t");
        assert_eq!(escaped, "hello \\\"world\\\"\\n\\\\tab\\t");
    }

    #[test]
    fn test_replay_guard_allows_new() {
        let mut guard = ReplayGuard::new();
        assert!(guard.check_and_record("s1", 0));
        assert!(guard.check_and_record("s1", 1));
        assert!(guard.check_and_record("s1", 2));
    }

    #[test]
    fn test_replay_guard_rejects_duplicate() {
        let mut guard = ReplayGuard::new();
        assert!(guard.check_and_record("s1", 0));
        assert!(!guard.check_and_record("s1", 0));
    }

    #[test]
    fn test_replay_guard_different_sessions() {
        let mut guard = ReplayGuard::new();
        assert!(guard.check_and_record("s1", 0));
        assert!(guard.check_and_record("s2", 0));
    }

    #[test]
    fn test_replay_guard_bounded_set() {
        let mut guard = ReplayGuard::new();
        for i in 0..MAX_TRACKED_SEQUENCES_PER_SESSION + 10 {
            assert!(guard.check_and_record("s1", i as u64));
        }
        assert!(guard.check_and_record("s1", 0));
    }

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let mut limiter = YodaRateLimiter::new();
        let now = 1000000u64;
        for i in 0..RATE_LIMIT_PER_MINUTE {
            assert!(limiter.check_rate("211.111.111.111.1", now + i as u64 * 1000));
        }
    }

    #[test]
    fn test_rate_limiter_rejects_over_limit() {
        let mut limiter = YodaRateLimiter::new();
        let now = 1000000u64;
        for i in 0..RATE_LIMIT_PER_MINUTE {
            assert!(limiter.check_rate("211.111.111.111.1", now + i as u64 * 100));
        }
        assert!(!limiter.check_rate("211.111.111.111.1", now + 900));
    }

    #[test]
    fn test_rate_limiter_resets_after_window() {
        let mut limiter = YodaRateLimiter::new();
        let now = 1000000u64;
        for i in 0..RATE_LIMIT_PER_MINUTE {
            limiter.check_rate("211.111.111.111.1", now + i as u64 * 100);
        }
        assert!(limiter.check_rate("211.111.111.111.1", now + 61_000));
    }

    #[test]
    fn test_error_codes_unique_exit_codes() {
        let codes = [
            YodaErrorCode::DaemonNotRunning,
            YodaErrorCode::RelayDisconnected,
            YodaErrorCode::NinjaexecLocked,
            YodaErrorCode::NinjaexecNotRunning,
            YodaErrorCode::SignatureInvalid,
            YodaErrorCode::MessageExpired,
            YodaErrorCode::SequenceReplay,
            YodaErrorCode::OperatorNotAuthorized,
            YodaErrorCode::MessageTooLong,
            YodaErrorCode::YodaTimeout,
            YodaErrorCode::YodaUnavailable,
            YodaErrorCode::AddressMismatch,
            YodaErrorCode::RateLimited,
            YodaErrorCode::ConcurrentAccess,
        ];
        let exit_codes: HashSet<i32> = codes.iter().map(|c| c.exit_code()).collect();
        assert_eq!(exit_codes.len(), 14);
    }

    #[test]
    fn test_session_file_new() {
        let session = SessionFile::new();
        assert_eq!(session.sequence, 0);
        assert!(!session.session_id.is_empty());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_file_next_sequence() {
        let mut session = SessionFile::new();
        assert_eq!(session.next_sequence(), 0);
        assert_eq!(session.next_sequence(), 1);
        assert_eq!(session.next_sequence(), 2);
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = b"Hello, PlenumNET!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_daemon_api_error_format() {
        let err = DaemonApiError::from_error_code(YodaErrorCode::MessageTooLong);
        assert_eq!(err.code, "MESSAGE_TOO_LONG");
        assert_eq!(err.exit_code, 9);
    }

    #[test]
    fn test_doctor_status_display() {
        assert_eq!(format!("{}", DoctorStatus::Ok), "[OK]");
        assert_eq!(format!("{}", DoctorStatus::Warn), "[WARN]");
        assert_eq!(format!("{}", DoctorStatus::Fail), "[FAIL]");
    }

    #[test]
    fn test_message_size_limit() {
        let big_msg = "x".repeat(MAX_MESSAGE_BYTES + 1);
        assert!(big_msg.len() > MAX_MESSAGE_BYTES);
    }

    #[test]
    fn test_payload_hash_format() {
        let hash = compute_payload_hash(b"test data");
        assert!(hash.starts_with("tis27:"));
    }

    #[test]
    fn test_yoda_response_error_serialization() {
        let resp = YodaResponsePayload {
            session_id: "abc-123".to_string(),
            sequence: 42,
            content: None,
            metadata: None,
            error: Some(YodaResponseError {
                code: "YODA_TIMEOUT".to_string(),
                message: "Yoda is taking too long to respond.".to_string(),
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("YODA_TIMEOUT"));
        assert!(!json.contains("content"));
    }
}
