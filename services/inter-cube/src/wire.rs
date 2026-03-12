// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Inter-Cube Wire Protocol
//!
//! Defines the binary wire format for ALL inter-cube messages: heartbeats,
//! CRS registrations, tunnel handshakes, forwarding control, and health probes.
//!
//! ## Design Principles
//!
//! 1. **Universal versioned header.** Every inter-cube message — heartbeats,
//!    CRS queries, tunnel messages, everything — carries the same 24-byte
//!    header with `protocol_version: u8`. This enables safe rollout of
//!    wire-format changes (dual checksum, ECC syndrome, mutual auth) without
//!    breaking older nodes.
//!
//! 2. **Femtosecond timestamps.** All timestamps are `u128` femtoseconds
//!    since the Salvi Epoch (2025-04-01T00:00:00Z). HPTP-synchronized,
//!    not NTP. The replay window is network propagation time, not clock drift.
//!
//! 3. **Rep C only.** All trit values on the wire use Rep C {1, 2, 3}.
//!    Zero never appears — its presence is instant proof of forgery.
//!    (INVARIANT 3)
//!
//! 4. **Future-proof message types.** Message type codes are allocated in
//!    blocks: 0x10–0x1F for heartbeats/health, 0x20–0x2F for CRS operations,
//!    0x30–0x3F for signed operations, 0x40–0x4F for tunnel authentication.
//!    Reserved ranges prevent collisions as new tasks land.
//!
//! ## Wire Header Layout (24 bytes)
//!
//! ```text
//! ┌─────────┬──────────┬───────┬──────────┬─────────────┬───────────────────┐
//! │ version │ msg_type │ flags │ reserved │ payload_len │    timestamp      │
//! │  (1B)   │   (1B)   │ (1B)  │   (1B)   │   (4B LE)   │   (16B LE u128)   │
//! └─────────┴──────────┴───────┴──────────┴─────────────┴───────────────────┘
//! ```
//!
//! ## Created by T-01 (SPEC-2026-NEXT)
//! Required by T-06 (signed CRS), T-08 (auth heartbeats), T-10 (dual checksum),
//! T-14 (mutual tunnel auth), T-17 (wire ECC syndrome).

use crate::cube_addr::CubeAddr;

// ═══════════════════════════════════════════════════════════════════════
// PROTOCOL VERSION
// ═══════════════════════════════════════════════════════════════════════

/// Protocol version for the legacy (pre-hardening) wire format.
/// No signature verification, no authenticated heartbeats, no ECC.
pub const PROTOCOL_VERSION_V1: u8 = 0x01;

/// Protocol version for the hardened wire format (this release).
/// Signed CRS, authenticated heartbeats, mutual tunnel auth, dual checksum, ECC.
pub const PROTOCOL_VERSION_V2: u8 = 0x02;

/// The current protocol version emitted by this build.
pub const PROTOCOL_VERSION_CURRENT: u8 = PROTOCOL_VERSION_V2;

/// Minimum protocol version this build will accept.
/// During dual-acceptance period, this is V1. After Phase 2 ships,
/// set to V2 to reject legacy nodes.
pub const PROTOCOL_VERSION_MIN: u8 = PROTOCOL_VERSION_V1;

// ═══════════════════════════════════════════════════════════════════════
// WIRE HEADER
// ═══════════════════════════════════════════════════════════════════════

/// Size of the wire header in bytes.
pub const WIRE_HEADER_SIZE: usize = 24;

/// Universal header for every inter-cube wire message.
///
/// Present on ALL inter-cube message types: heartbeats, CRS queries,
/// tunnel establishment, tunnel data, health probes, registrations.
/// The `protocol_version` field enables safe rollout of format changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireHeader {
    /// Protocol version. Determines which fields/extensions are present.
    pub version: u8,
    /// Message type code. See [`MessageType`].
    pub msg_type: u8,
    /// Bitfield flags. See [`WireFlags`].
    pub flags: u8,
    /// Reserved for future use. Must be zero on send, ignored on receive.
    pub reserved: u8,
    /// Length of the payload following this header, in bytes.
    pub payload_len: u32,
    /// Femtosecond timestamp since Salvi Epoch (2025-04-01T00:00:00Z).
    /// HPTP-synchronized. Used for replay detection and ordering.
    pub timestamp_fs: u128,
}

impl WireHeader {
    /// Create a new header with the current protocol version.
    pub fn new(msg_type: MessageType, payload_len: u32, timestamp_fs: u128) -> Self {
        WireHeader {
            version: PROTOCOL_VERSION_CURRENT,
            msg_type: msg_type.as_u8(),
            flags: 0,
            reserved: 0,
            payload_len,
            timestamp_fs,
        }
    }

    /// Create a header with specific flags.
    pub fn with_flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    /// Serialize to 24 bytes (little-endian).
    pub fn to_wire(&self) -> [u8; WIRE_HEADER_SIZE] {
        let mut buf = [0u8; WIRE_HEADER_SIZE];
        buf[0] = self.version;
        buf[1] = self.msg_type;
        buf[2] = self.flags;
        buf[3] = self.reserved;
        buf[4..8].copy_from_slice(&self.payload_len.to_le_bytes());
        buf[8..24].copy_from_slice(&self.timestamp_fs.to_le_bytes());
        buf
    }

    /// Deserialize from a 24-byte slice.
    ///
    /// Returns `None` if the buffer is too short.
    pub fn from_wire(buf: &[u8]) -> Option<Self> {
        if buf.len() < WIRE_HEADER_SIZE {
            return None;
        }
        Some(WireHeader {
            version: buf[0],
            msg_type: buf[1],
            flags: buf[2],
            reserved: buf[3],
            payload_len: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            timestamp_fs: u128::from_le_bytes(
                buf[8..24].try_into().ok()?
            ),
        })
    }

    /// Check whether this header's version is acceptable.
    pub fn version_acceptable(&self) -> bool {
        self.version >= PROTOCOL_VERSION_MIN && self.version <= PROTOCOL_VERSION_CURRENT
    }

    /// Parse the message type code into the enum.
    pub fn message_type(&self) -> Option<MessageType> {
        MessageType::from_u8(self.msg_type)
    }

    /// Check if the HPTP-mandatory flag is set.
    pub fn is_hptp_mandatory(&self) -> bool {
        self.flags & WireFlags::HPTP_MANDATORY != 0
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WIRE FLAGS
// ═══════════════════════════════════════════════════════════════════════

/// Bitfield flags for the wire header.
///
/// Carried in the `flags` byte of every message header.
pub struct WireFlags;

impl WireFlags {
    /// Destination address is HPTP-mandatory (trits 15+16 = 3,3).
    /// Femtosecond timing verification is REQUIRED for this packet.
    pub const HPTP_MANDATORY: u8 = 0x01;

    /// This message carries a TL-DSA signature in its payload.
    /// Receivers must verify before processing.
    pub const SIGNED: u8 = 0x02;

    /// This message includes a dual checksum (mod-364 + mod-333).
    /// Only present when protocol_version ≥ V2 and `enable_dual_checksum` flag is on.
    pub const DUAL_CHECKSUM: u8 = 0x04;

    /// This message includes an 8-trit ECC syndrome.
    /// Only present when protocol_version ≥ V2 and `enable_wire_ecc` flag is on.
    pub const ECC_SYNDROME: u8 = 0x08;
}

// ═══════════════════════════════════════════════════════════════════════
// MESSAGE TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Inter-cube wire message types.
///
/// Allocated in blocks to prevent collisions as new tasks land:
/// - `0x10–0x1F`: Heartbeat / health monitoring
/// - `0x20–0x2F`: CRS operations (registration, query, deregistration)
/// - `0x30–0x3F`: Signed operations (reserved for T-06)
/// - `0x40–0x4F`: Tunnel authentication (reserved for T-14)
/// - `0x50–0x5F`: Data forwarding / GLB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    // ── Heartbeat / Health (0x10–0x1F) ──────────────────────────
    /// Basic heartbeat ping (v1 legacy, unauthenticated).
    HeartbeatPing = 0x10,
    /// Basic heartbeat pong (v1 legacy, unauthenticated).
    HeartbeatPong = 0x11,
    /// Authenticated heartbeat (v2, T-08: HMAC or TL-DSA).
    AuthHeartbeatPing = 0x12,
    /// Authenticated heartbeat response (v2, T-08).
    AuthHeartbeatPong = 0x13,

    // ── CRS Operations (0x20–0x2F) ─────────────────────────────
    /// CRS registration request (v1 legacy, unsigned).
    CrsRegister = 0x20,
    /// CRS registration response.
    CrsRegisterAck = 0x21,
    /// CRS neighbor query.
    CrsQuery = 0x22,
    /// CRS neighbor query response.
    CrsQueryResponse = 0x23,

    // ── Signed Operations (0x30–0x3F) ──────────────────────────
    /// Signed CRS registration (v2, T-06: TL-DSA signature).
    SignedCrsRegister = 0x30,
    /// Signed CRS registration acknowledgment.
    SignedCrsRegisterAck = 0x31,
    /// Signed CRS deregistration (v2, T-21).
    SignedCrsDeregister = 0x32,

    // ── Tunnel Authentication (0x40–0x4F) ──────────────────────
    /// Mutual tunnel auth: CHALLENGE (v2, T-14).
    TunnelChallenge = 0x40,
    /// Mutual tunnel auth: RESPONSE (v2, T-14).
    TunnelResponse = 0x41,
    /// Mutual tunnel auth: CONFIRM (v2, T-14).
    TunnelConfirm = 0x42,

    // ── Data Forwarding (0x50–0x5F) ────────────────────────────
    /// Standard data packet forwarded via GLB.
    DataForward = 0x50,
    /// SubCube multicast packet.
    DataMulticast = 0x51,
}

impl MessageType {
    /// Convert to the underlying byte value.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse from a byte value. Returns `None` for unknown types.
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x10 => Some(Self::HeartbeatPing),
            0x11 => Some(Self::HeartbeatPong),
            0x12 => Some(Self::AuthHeartbeatPing),
            0x13 => Some(Self::AuthHeartbeatPong),
            0x20 => Some(Self::CrsRegister),
            0x21 => Some(Self::CrsRegisterAck),
            0x22 => Some(Self::CrsQuery),
            0x23 => Some(Self::CrsQueryResponse),
            0x30 => Some(Self::SignedCrsRegister),
            0x31 => Some(Self::SignedCrsRegisterAck),
            0x32 => Some(Self::SignedCrsDeregister),
            0x40 => Some(Self::TunnelChallenge),
            0x41 => Some(Self::TunnelResponse),
            0x42 => Some(Self::TunnelConfirm),
            0x50 => Some(Self::DataForward),
            0x51 => Some(Self::DataMulticast),
            _ => None,
        }
    }

    /// Whether this message type requires protocol version ≥ V2.
    pub fn requires_v2(&self) -> bool {
        matches!(
            self,
            Self::AuthHeartbeatPing
                | Self::AuthHeartbeatPong
                | Self::SignedCrsRegister
                | Self::SignedCrsRegisterAck
                | Self::SignedCrsDeregister
                | Self::TunnelChallenge
                | Self::TunnelResponse
                | Self::TunnelConfirm
        )
    }

    /// Whether this message type carries a TL-DSA signature.
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::SignedCrsRegister | Self::SignedCrsDeregister
        )
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HeartbeatPing => write!(f, "heartbeat-ping"),
            Self::HeartbeatPong => write!(f, "heartbeat-pong"),
            Self::AuthHeartbeatPing => write!(f, "auth-heartbeat-ping"),
            Self::AuthHeartbeatPong => write!(f, "auth-heartbeat-pong"),
            Self::CrsRegister => write!(f, "crs-register"),
            Self::CrsRegisterAck => write!(f, "crs-register-ack"),
            Self::CrsQuery => write!(f, "crs-query"),
            Self::CrsQueryResponse => write!(f, "crs-query-response"),
            Self::SignedCrsRegister => write!(f, "signed-crs-register"),
            Self::SignedCrsRegisterAck => write!(f, "signed-crs-register-ack"),
            Self::SignedCrsDeregister => write!(f, "signed-crs-deregister"),
            Self::TunnelChallenge => write!(f, "tunnel-challenge"),
            Self::TunnelResponse => write!(f, "tunnel-response"),
            Self::TunnelConfirm => write!(f, "tunnel-confirm"),
            Self::DataForward => write!(f, "data-forward"),
            Self::DataMulticast => write!(f, "data-multicast"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WIRE ADDRESS — 13-trit Rep C on the wire (4 bytes packed)
// ═══════════════════════════════════════════════════════════════════════

/// Packed wire representation of a 13-trit Rep C cube address.
///
/// 13 trits at 2 bits each = 26 bits → 4 bytes (with 6 unused bits).
/// Rep C values {1, 2, 3} map to 2-bit encodings {0b01, 0b10, 0b11}.
/// The encoding 0b00 is invalid — its presence is proof of forgery.
///
/// Wire layout (4 bytes, 32 bits, big-endian):
/// ```text
/// bits [31:30] = trit 0
/// bits [29:28] = trit 1
/// ...
/// bits [ 7: 6] = trit 12
/// bits [ 5: 0] = reserved (must be zero)
/// ```
pub const WIRE_ADDR_SIZE: usize = 4;

/// Pack a 13-trit Rep C address into 4 wire bytes.
///
/// Returns `None` if any trit is zero or outside {1, 2, 3}
/// (INVARIANT 3: Rep C excludes zero).
pub fn pack_trit_array(trits: &[u8; 13]) -> Option<[u8; WIRE_ADDR_SIZE]> {
    let mut packed: u32 = 0;
    for (dim, &trit) in trits.iter().enumerate() {
        if trit == 0 || trit > 3 {
            return None;
        }
        packed |= (trit as u32) << (30 - dim * 2);
    }
    Some(packed.to_be_bytes())
}

/// Unpack 4 wire bytes into a 13-trit Rep C address.
///
/// Returns `None` if any 2-bit field is 0b00 (invalid in Rep C).
pub fn unpack_trit_array(buf: &[u8; WIRE_ADDR_SIZE]) -> Option<[u8; 13]> {
    let packed = u32::from_be_bytes(*buf);
    let mut trits = [0u8; 13];
    for dim in 0..13 {
        let val = ((packed >> (30 - dim * 2)) & 0x03) as u8;
        if val == 0 {
            return None;
        }
        trits[dim] = val;
    }
    Some(trits)
}

/// Pack a CubeAddr into 4 wire bytes.
///
/// Convenience wrapper around [`pack_trit_array`].
/// Uses `addr.to_bytes()` to get the raw 13-trit Rep C array.
pub fn pack_addr(addr: &CubeAddr) -> Option<[u8; WIRE_ADDR_SIZE]> {
    let bytes = addr.to_bytes();
    pack_trit_array(&bytes)
}

/// Unpack 4 wire bytes into a CubeAddr.
///
/// Convenience wrapper around [`unpack_trit_array`].
pub fn unpack_addr(buf: &[u8; WIRE_ADDR_SIZE]) -> Option<CubeAddr> {
    let trits = unpack_trit_array(buf)?;
    CubeAddr::try_from_bytes(&trits)
}

// ═══════════════════════════════════════════════════════════════════════
// WIRE MESSAGE — Header + Payload
// ═══════════════════════════════════════════════════════════════════════

/// A complete wire message: header + raw payload bytes.
///
/// The payload format depends on the `msg_type` in the header.
/// Downstream tasks (T-06, T-08, T-14, etc.) define typed payload
/// structs and their serialization.
#[derive(Debug, Clone)]
pub struct WireMessage {
    /// Universal 24-byte header.
    pub header: WireHeader,
    /// Raw payload bytes (interpretation depends on msg_type).
    pub payload: Vec<u8>,
}

impl WireMessage {
    /// Create a new wire message.
    pub fn new(msg_type: MessageType, timestamp_fs: u128, payload: Vec<u8>) -> Self {
        let header = WireHeader::new(msg_type, payload.len() as u32, timestamp_fs);
        WireMessage { header, payload }
    }

    /// Create with specific flags.
    pub fn with_flags(mut self, flags: u8) -> Self {
        self.header.flags = flags;
        self
    }

    /// Serialize the complete message (header + payload) to bytes.
    pub fn to_wire(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(WIRE_HEADER_SIZE + self.payload.len());
        buf.extend_from_slice(&self.header.to_wire());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Deserialize a complete message from bytes.
    ///
    /// Returns `None` if:
    /// - Buffer too short for header
    /// - Payload length exceeds remaining bytes
    pub fn from_wire(buf: &[u8]) -> Option<Self> {
        let header = WireHeader::from_wire(buf)?;
        let payload_start = WIRE_HEADER_SIZE;
        let payload_end = payload_start + header.payload_len as usize;
        if buf.len() < payload_end {
            return None;
        }
        Some(WireMessage {
            header,
            payload: buf[payload_start..payload_end].to_vec(),
        })
    }

    /// Validate the message against protocol rules.
    ///
    /// Checks:
    /// 1. Protocol version is within acceptable range
    /// 2. Message type is recognized
    /// 3. V2-only message types require protocol version ≥ V2
    /// 4. Payload length matches actual payload
    pub fn validate(&self) -> Result<(), WireError> {
        if !self.header.version_acceptable() {
            return Err(WireError::IncompatibleVersion {
                received: self.header.version,
                min: PROTOCOL_VERSION_MIN,
                max: PROTOCOL_VERSION_CURRENT,
            });
        }

        let msg_type = self.header.message_type()
            .ok_or(WireError::UnknownMessageType(self.header.msg_type))?;

        if msg_type.requires_v2() && self.header.version < PROTOCOL_VERSION_V2 {
            return Err(WireError::MessageRequiresV2 {
                msg_type: self.header.msg_type,
                version: self.header.version,
            });
        }

        if self.header.payload_len as usize != self.payload.len() {
            return Err(WireError::PayloadLengthMismatch {
                declared: self.header.payload_len,
                actual: self.payload.len() as u32,
            });
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Errors in wire protocol parsing and validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// Protocol version outside acceptable range.
    IncompatibleVersion {
        received: u8,
        min: u8,
        max: u8,
    },
    /// Unrecognized message type code.
    UnknownMessageType(u8),
    /// V2 message type received on a V1 connection.
    MessageRequiresV2 {
        msg_type: u8,
        version: u8,
    },
    /// Declared payload length doesn't match actual.
    PayloadLengthMismatch {
        declared: u32,
        actual: u32,
    },
    /// Buffer too short to parse.
    BufferTooShort {
        expected: usize,
        actual: usize,
    },
    /// Zero detected in Rep C address — proof of forgery (INVARIANT 3).
    RepCZeroDetected,
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompatibleVersion { received, min, max } => {
                write!(f, "protocol version {} outside range [{}, {}]", received, min, max)
            }
            Self::UnknownMessageType(t) => write!(f, "unknown message type 0x{:02X}", t),
            Self::MessageRequiresV2 { msg_type, version } => {
                write!(f, "message type 0x{:02X} requires v2, got v{}", msg_type, version)
            }
            Self::PayloadLengthMismatch { declared, actual } => {
                write!(f, "payload length mismatch: declared {}, actual {}", declared, actual)
            }
            Self::BufferTooShort { expected, actual } => {
                write!(f, "buffer too short: need {} bytes, have {}", expected, actual)
            }
            Self::RepCZeroDetected => write!(f, "zero in Rep C address — proof of forgery"),
        }
    }
}

impl std::error::Error for WireError {}

// ═══════════════════════════════════════════════════════════════════════
// VERSION NEGOTIATION
// ═══════════════════════════════════════════════════════════════════════

/// Negotiate the highest common protocol version between two peers.
///
/// Each peer advertises its supported range `[min, max]`.
/// Returns the highest version both support, or `None` if no overlap.
pub fn negotiate_version(
    local_min: u8,
    local_max: u8,
    remote_min: u8,
    remote_max: u8,
) -> Option<u8> {
    let overlap_min = local_min.max(remote_min);
    let overlap_max = local_max.min(remote_max);
    if overlap_min <= overlap_max {
        Some(overlap_max)
    } else {
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SALVI EPOCH TIMESTAMP UTILITIES
// ═══════════════════════════════════════════════════════════════════════

/// Femtoseconds per second.
pub const FS_PER_SECOND: u128 = 1_000_000_000_000_000;

/// Femtoseconds per millisecond.
pub const FS_PER_MILLISECOND: u128 = 1_000_000_000_000;

/// Maximum registration age: 30 seconds in femtoseconds.
/// This is the maximum network propagation time, not clock drift tolerance.
/// Nodes are HPTP-synchronized.
pub const REGISTRATION_MAX_AGE_FS: u128 = 30 * FS_PER_SECOND;

/// Future tolerance: 1 second in femtoseconds.
/// Allows for minor HPTP propagation skew — a message can arrive
/// slightly "from the future" relative to the receiver's clock.
pub const TIMESTAMP_FUTURE_TOLERANCE_FS: u128 = 1 * FS_PER_SECOND;

/// Check whether a timestamp is within the acceptable window.
///
/// Accepts timestamps where:
/// - `timestamp ≤ now + TIMESTAMP_FUTURE_TOLERANCE_FS` (not too far in the future)
/// - `now - timestamp ≤ REGISTRATION_MAX_AGE_FS` (not too old)
pub fn timestamp_in_window(timestamp_fs: u128, now_fs: u128) -> bool {
    if timestamp_fs > now_fs + TIMESTAMP_FUTURE_TOLERANCE_FS {
        return false;
    }
    if now_fs > timestamp_fs && (now_fs - timestamp_fs) > REGISTRATION_MAX_AGE_FS {
        return false;
    }
    true
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Header serialization round-trip ─────────────────────────

    #[test]
    fn test_header_roundtrip() {
        let ts: u128 = 42_000_000_000_000_000;
        let header = WireHeader::new(MessageType::HeartbeatPing, 128, ts);

        let wire = header.to_wire();
        assert_eq!(wire.len(), WIRE_HEADER_SIZE);

        let parsed = WireHeader::from_wire(&wire).unwrap();
        assert_eq!(parsed.version, PROTOCOL_VERSION_CURRENT);
        assert_eq!(parsed.msg_type, MessageType::HeartbeatPing.as_u8());
        assert_eq!(parsed.flags, 0);
        assert_eq!(parsed.payload_len, 128);
        assert_eq!(parsed.timestamp_fs, ts);
    }

    #[test]
    fn test_header_from_short_buffer() {
        let buf = [0u8; 10];
        assert!(WireHeader::from_wire(&buf).is_none());
    }

    // ── Message type parsing ────────────────────────────────────

    #[test]
    fn test_message_type_roundtrip() {
        let types = [
            MessageType::HeartbeatPing,
            MessageType::AuthHeartbeatPing,
            MessageType::CrsRegister,
            MessageType::SignedCrsRegister,
            MessageType::TunnelChallenge,
            MessageType::TunnelResponse,
            MessageType::TunnelConfirm,
            MessageType::DataForward,
        ];
        for mt in &types {
            let byte = mt.as_u8();
            let parsed = MessageType::from_u8(byte).unwrap();
            assert_eq!(*mt, parsed);
        }
    }

    #[test]
    fn test_unknown_message_type() {
        assert!(MessageType::from_u8(0xFF).is_none());
        assert!(MessageType::from_u8(0x00).is_none());
    }

    #[test]
    fn test_v2_message_types_flagged() {
        assert!(MessageType::SignedCrsRegister.requires_v2());
        assert!(MessageType::TunnelChallenge.requires_v2());
        assert!(MessageType::AuthHeartbeatPing.requires_v2());
        assert!(!MessageType::HeartbeatPing.requires_v2());
        assert!(!MessageType::CrsRegister.requires_v2());
        assert!(!MessageType::DataForward.requires_v2());
    }

    // ── Full message round-trip ─────────────────────────────────

    #[test]
    fn test_message_roundtrip() {
        let payload = vec![1u8, 2, 3, 4, 5];
        let ts: u128 = 100 * FS_PER_SECOND;
        let msg = WireMessage::new(MessageType::DataForward, ts, payload.clone());

        let wire = msg.to_wire();
        assert_eq!(wire.len(), WIRE_HEADER_SIZE + 5);

        let parsed = WireMessage::from_wire(&wire).unwrap();
        assert_eq!(parsed.header.msg_type, MessageType::DataForward.as_u8());
        assert_eq!(parsed.payload, payload);
        assert_eq!(parsed.header.timestamp_fs, ts);

        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_message_truncated_payload() {
        let msg = WireMessage::new(MessageType::HeartbeatPing, 0, vec![1, 2, 3]);
        let wire = msg.to_wire();
        let truncated = &wire[..wire.len() - 1];
        assert!(WireMessage::from_wire(truncated).is_none());
    }

    // ── Validation ──────────────────────────────────────────────

    #[test]
    fn test_validate_v2_message_on_v1_header() {
        let mut msg = WireMessage::new(
            MessageType::SignedCrsRegister,
            0,
            vec![],
        );
        msg.header.version = PROTOCOL_VERSION_V1;

        let err = msg.validate().unwrap_err();
        assert!(matches!(err, WireError::MessageRequiresV2 { .. }));
    }

    #[test]
    fn test_validate_unknown_version() {
        let mut msg = WireMessage::new(MessageType::HeartbeatPing, 0, vec![]);
        msg.header.version = 0xFF;

        let err = msg.validate().unwrap_err();
        assert!(matches!(err, WireError::IncompatibleVersion { .. }));
    }

    // ── Version negotiation ─────────────────────────────────────

    #[test]
    fn test_negotiate_both_v2() {
        let v = negotiate_version(1, 2, 1, 2);
        assert_eq!(v, Some(2));
    }

    #[test]
    fn test_negotiate_local_v2_remote_v1() {
        let v = negotiate_version(1, 2, 1, 1);
        assert_eq!(v, Some(1));
    }

    #[test]
    fn test_negotiate_no_overlap() {
        let v = negotiate_version(2, 2, 3, 3);
        assert_eq!(v, None);
    }

    // ── Address packing ─────────────────────────────────────────

    #[test]
    fn test_trit_array_pack_unpack_roundtrip() {
        let trits: [u8; 13] = [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        let packed = pack_trit_array(&trits).unwrap();
        assert_eq!(packed.len(), WIRE_ADDR_SIZE);

        let unpacked = unpack_trit_array(&packed).unwrap();
        assert_eq!(trits, unpacked);
    }

    #[test]
    fn test_trit_array_all_ones() {
        let trits: [u8; 13] = [1; 13];
        let packed = pack_trit_array(&trits).unwrap();
        let unpacked = unpack_trit_array(&packed).unwrap();
        assert_eq!(trits, unpacked);
    }

    #[test]
    fn test_trit_array_all_threes() {
        let trits: [u8; 13] = [3; 13];
        let packed = pack_trit_array(&trits).unwrap();
        let unpacked = unpack_trit_array(&packed).unwrap();
        assert_eq!(trits, unpacked);
    }

    #[test]
    fn test_trit_array_zero_rejected() {
        let trits: [u8; 13] = [1, 0, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        assert!(pack_trit_array(&trits).is_none());
    }

    #[test]
    fn test_trit_array_four_rejected() {
        let trits: [u8; 13] = [1, 4, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        assert!(pack_trit_array(&trits).is_none());
    }

    #[test]
    fn test_unpack_zero_bits_rejected() {
        let mut packed = [0u8; 4];
        packed[0] = 0b00_01_00_00;
        assert!(unpack_trit_array(&packed).is_none());
    }

    #[test]
    fn test_addr_pack_unpack_via_cube_addr() {
        let addr = CubeAddr::new([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let packed = pack_addr(&addr).unwrap();
        let unpacked = unpack_addr(&packed).unwrap();
        assert_eq!(addr, unpacked);
    }

    // ── Timestamp utilities ─────────────────────────────────────

    #[test]
    fn test_timestamp_in_window_current() {
        let now = 1000 * FS_PER_SECOND;
        assert!(timestamp_in_window(now, now));
    }

    #[test]
    fn test_timestamp_in_window_slightly_old() {
        let now = 1000 * FS_PER_SECOND;
        let ts = now - 10 * FS_PER_SECOND;
        assert!(timestamp_in_window(ts, now));
    }

    #[test]
    fn test_timestamp_in_window_too_old() {
        let now = 1000 * FS_PER_SECOND;
        let ts = now - 60 * FS_PER_SECOND;
        assert!(!timestamp_in_window(ts, now));
    }

    #[test]
    fn test_timestamp_in_window_slight_future() {
        let now = 1000 * FS_PER_SECOND;
        let ts = now + FS_PER_MILLISECOND * 500;
        assert!(timestamp_in_window(ts, now));
    }

    #[test]
    fn test_timestamp_in_window_too_far_future() {
        let now = 1000 * FS_PER_SECOND;
        let ts = now + 5 * FS_PER_SECOND;
        assert!(!timestamp_in_window(ts, now));
    }

    // ── Wire flags ──────────────────────────────────────────────

    #[test]
    fn test_wire_flags() {
        let header = WireHeader::new(MessageType::SignedCrsRegister, 0, 0)
            .with_flags(WireFlags::SIGNED | WireFlags::HPTP_MANDATORY);

        assert!(header.is_hptp_mandatory());
        assert_ne!(header.flags & WireFlags::SIGNED, 0);
        assert_eq!(header.flags & WireFlags::DUAL_CHECKSUM, 0);
    }
}
