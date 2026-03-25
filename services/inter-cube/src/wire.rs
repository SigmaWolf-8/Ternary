// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

// # Inter-Cube Wire Protocol
//!
// Defines the binary wire format for ALL inter-cube messages: heartbeats,
// CRS registrations, tunnel handshakes, forwarding control, and health probes.
//!
// ## Design Principles
//!
// 1. **Universal versioned header.** Every inter-cube message — heartbeats,
//    CRS queries, tunnel messages, everything — carries the same 24-byte
//    header with `protocol_version: u8`. This enables safe rollout of
//    wire-format changes (dual checksum, ECC syndrome, mutual auth) without
//    breaking older nodes.
//!
// 2. **Femtosecond timestamps.** All timestamps are `u128` femtoseconds
//    since the Salvi Epoch (2025-04-01T00:00:00Z). HPTP-synchronized,
//    not NTP. The replay window is network propagation time, not clock drift.
//!
// 3. **Rep C only.** All trit values on the wire use Rep C {1, 2, 3}.
//    Zero never appears — its presence is instant proof of forgery.
//    (INVARIANT 3)
//!
// 4. **Future-proof message types.** Message type codes are allocated in
//    blocks: 0x10–0x1F for heartbeats/health, 0x20–0x2F for CRS operations,
//    0x30–0x3F for signed operations, 0x40–0x4F for tunnel authentication.
//    Reserved ranges prevent collisions as new tasks land.
//!
// ## Wire Header Layout (24 bytes)
//!
// ```text
// ┌─────────┬──────────┬───────┬──────────┬─────────────┬───────────────────┐
// │ version │ msg_type │ flags │ reserved │ payload_len │    timestamp      │
// │  (1B)   │   (1B)   │ (1B)  │   (1B)   │   (4B LE)   │   (16B LE u128)   │
// └─────────┴──────────┴───────┴──────────┴─────────────┴───────────────────┘
// ```
//!
// ## Created by T-01 (SPEC-2026-NEXT)
// Required by T-06 (signed CRS), T-08 (auth heartbeats), T-10 (dual checksum),
// T-14 (mutual tunnel auth), T-17 (wire ECC syndrome).

use crate::cube_addr::CubeAddr;

// ═══════════════════════════════════════════════════════════════════════
// PROTOCOL VERSION
// ═══════════════════════════════════════════════════════════════════════

/// Protocol version for the legacy (pre-hardening) wire format.
/// No signature verification, no authenticated heartbeats, no ECC.
pub const PROTOCOL_VERSION_V1: u8 = 0x01;

/// Protocol version for the hardened wire format.
/// Signed CRS, authenticated heartbeats, mutual tunnel auth, dual checksum, ECC.
pub const PROTOCOL_VERSION_V2: u8 = 0x02;

/// Protocol version for Array3 Node Cluster (this release).
/// Slot addressing, 13-capability registration, key birth epochs, Rep C node IDs.
/// V3 accepts V2 during dual-acceptance period.
pub const PROTOCOL_VERSION_V3: u8 = 0x03;

/// The current protocol version emitted by this build.
pub const PROTOCOL_VERSION_CURRENT: u8 = PROTOCOL_VERSION_V3;

/// Minimum protocol version this build will accept.
/// During dual-acceptance period, V2 is accepted alongside V3.
/// V1 legacy nodes are no longer accepted.
pub const PROTOCOL_VERSION_MIN: u8 = PROTOCOL_VERSION_V2;

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

    // ── Array3 / Slot Operations (0x60–0x6F, V3) ─────────────
    /// Array3 formation handshake (V3).
    Array3Handshake = 0x60,
    /// Array3 handshake acknowledgment (V3).
    Array3HandshakeAck = 0x61,
    /// Service slot registration with 13-capability vector (V3).
    SlotRegister = 0x62,
    /// Service slot registration acknowledgment (V3).
    SlotRegisterAck = 0x63,
    /// Slot-to-node routing query (V3).
    SlotQuery = 0x64,
    /// Slot-to-node routing response (V3).
    SlotQueryResponse = 0x65,
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
            0x60 => Some(Self::Array3Handshake),
            0x61 => Some(Self::Array3HandshakeAck),
            0x62 => Some(Self::SlotRegister),
            0x63 => Some(Self::SlotRegisterAck),
            0x64 => Some(Self::SlotQuery),
            0x65 => Some(Self::SlotQueryResponse),
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

    /// Whether this message type requires protocol version ≥ V3.
    pub fn requires_v3(&self) -> bool {
        matches!(
            self,
            Self::Array3Handshake
                | Self::Array3HandshakeAck
                | Self::SlotRegister
                | Self::SlotRegisterAck
                | Self::SlotQuery
                | Self::SlotQueryResponse
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
            Self::Array3Handshake => write!(f, "array3-handshake"),
            Self::Array3HandshakeAck => write!(f, "array3-handshake-ack"),
            Self::SlotRegister => write!(f, "slot-register"),
            Self::SlotRegisterAck => write!(f, "slot-register-ack"),
            Self::SlotQuery => write!(f, "slot-query"),
            Self::SlotQueryResponse => write!(f, "slot-query-response"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WIRE ADDRESS — 13-trit Rep C on the wire (7 bytes packed)
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
            return None; // Invalid Rep C value
        }
        // 2-bit encoding: {1,2,3} → {0b01, 0b10, 0b11}
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
            return None; // 0b00 = forgery
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
    if bytes.len() != 13 {
        return None;
    }
    let mut arr = [0u8; 13];
    arr.copy_from_slice(&bytes[..13]);
    pack_trit_array(&arr)
}

/// Unpack 4 wire bytes into a CubeAddr.
///
/// Convenience wrapper around [`unpack_trit_array`].
pub fn unpack_addr(buf: &[u8; WIRE_ADDR_SIZE]) -> Option<CubeAddr> {
    let trits = unpack_trit_array(buf)?;
    CubeAddr::try_from_bytes(&trits)
}

// ═══════════════════════════════════════════════════════════════════════
// SLOT ADDRESS — 3-trit Rep C wire encoding (V3)
// ═══════════════════════════════════════════════════════════════════════

pub const WIRE_SLOT_ADDR_SIZE: usize = 1;

/// Pack a 3-trit slot address into a single wire byte.
///
/// Layout: bits [7:6] = plane, [5:4] = role, [3:2] = instance, [1:0] = reserved.
/// Rep C {1,2,3} → 2-bit {0b01, 0b10, 0b11}. Zero = forgery.
pub fn pack_slot_addr(slot: &[u8; 3]) -> Option<u8> {
    for &t in slot {
        if t == 0 || t > 3 { return None; }
    }
    Some((slot[0] << 6) | (slot[1] << 4) | (slot[2] << 2))
}

/// Unpack a wire byte into a 3-trit slot address.
///
/// Returns `None` if any 2-bit field is 0b00 (Rep C violation).
pub fn unpack_slot_addr(byte: u8) -> Option<[u8; 3]> {
    let plane = (byte >> 6) & 0x03;
    let role  = (byte >> 4) & 0x03;
    let inst  = (byte >> 2) & 0x03;
    if plane == 0 || role == 0 || inst == 0 { return None; }
    Some([plane, role, inst])
}

/// V3 Array3 handshake payload (serialized after wire header).
///
/// ```text
/// ┌──────────┬────────────┬────────────┬──────────┬──────────┐
/// │ node_id  │ port_start │ port_end   │ wire_ver │ slots    │
/// │  (1B RC) │  (2B LE)   │  (2B LE)   │   (1B)   │ (1B cnt) │
/// └──────────┴────────────┴────────────┴──────────┴──────────┘
/// ```
pub const ARRAY3_HANDSHAKE_MIN_SIZE: usize = 7;

/// V3 SlotRegister payload (serialized after wire header).
///
/// ```text
/// ┌──────────┬──────────────┬───────┬──────────────┬──────────┐
/// │ node_id  │ classification│ slot  │ identity     │ caps     │
/// │  (1B RC) │  (27B trits) │ (1B)  │ (32B pubkey) │ (2B LE)  │
/// └──────────┴──────────────┴───────┴──────────────┴──────────┘
/// ```
pub const SLOT_REGISTER_PAYLOAD_SIZE: usize = 1 + 27 + 1 + 32 + 2;

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

        if msg_type.requires_v3() && self.header.version < PROTOCOL_VERSION_V3 {
            return Err(WireError::MessageRequiresV3 {
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
    /// V3 message type received on a pre-V3 connection.
    MessageRequiresV3 {
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
            Self::MessageRequiresV3 { msg_type, version } => {
                write!(f, "message type 0x{:02X} requires v3, got v{}", msg_type, version)
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
    // Future check: allow up to 1s ahead
    if timestamp_fs > now_fs + TIMESTAMP_FUTURE_TOLERANCE_FS {
        return false;
    }
    // Staleness check: reject if older than 30s
    if now_fs > timestamp_fs && (now_fs - timestamp_fs) > REGISTRATION_MAX_AGE_FS {
        return false;
    }
    true
}

// ═══════════════════════════════════════════════════════════════════════
// DUAL CHECKSUM — Wire Integration (T-10, SPEC-2026-NEXT)
// ═══════════════════════════════════════════════════════════════════════

// Wire-level dual checksum: mod-364 (repunit) + mod-333 (Plenum).
//
// `gcd(364, 333) = 1` → CRT combined modulus = 121,212.
// False-pass rate: 0.0008% (vs 0.27% for mod-364 alone).
//
// These functions work on **any** Rep C trit array — both 13-trit inter-cube
// addresses and 27-trit TDNS classification addresses. The full TDNS module
// (`ternary-math/src/plenum_checksum.rs`) has additional 27-trit-specific
// validation and integrity checks. These wire-level functions are the
// minimal integration for the inter-cube wire protocol.
//
// ## Wire Format
//
// Checksums are appended as 12 Rep C trits (6 mod-364 + 6 mod-333)
// after the address trits. Packed at 2 bits per trit:
//
// ```text
// ┌───────────────────┬────────────────┬────────────────┐
// │  address trits    │ 6-trit mod-364 │ 6-trit mod-333 │
// │  (13 or 27)       │ (repunit)      │ (Plenum)       │
// └───────────────────┴────────────────┴────────────────┘
// ```
//
// Gated behind `WireFlags::DUAL_CHECKSUM` and `PlenumConfig.enable_dual_checksum`.

/// Repunit checksum modulus: 364 = R₆ = 111111₃ = (3⁶ − 1) / 2.
pub const CHECKSUM_MOD_REPUNIT: u32 = 364;

/// Plenum checksum modulus: 333 = 3 × 111 = Plenum magic constant.
pub const CHECKSUM_MOD_PLENUM: u32 = 333;

/// Combined CRT detection space: 364 × 333 = 121,212.
pub const CHECKSUM_DETECTION_SPACE: u32 = 121_212;

/// Number of Rep C trits per single checksum (6 trits = fits in 729 = 3⁶).
pub const CHECKSUM_TRITS: usize = 6;

/// Total checksum trits appended to address: 2 × 6 = 12.
pub const DUAL_CHECKSUM_TRITS: usize = 12;

/// Wire checksum result: two 6-trit Rep C values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireChecksum {
    /// 6 Rep C trits — mod-364 repunit checksum.
    pub repunit: [u8; CHECKSUM_TRITS],
    /// 6 Rep C trits — mod-333 Plenum checksum.
    pub plenum: [u8; CHECKSUM_TRITS],
}

impl WireChecksum {
    /// Flatten to 12 trits: [repunit(6) ‖ plenum(6)].
    pub fn to_trits(&self) -> [u8; DUAL_CHECKSUM_TRITS] {
        let mut out = [0u8; DUAL_CHECKSUM_TRITS];
        out[..CHECKSUM_TRITS].copy_from_slice(&self.repunit);
        out[CHECKSUM_TRITS..].copy_from_slice(&self.plenum);
        out
    }

    /// Parse from 12 trits: [repunit(6) ‖ plenum(6)].
    ///
    /// Returns `None` if any trit is not in Rep C {1, 2, 3}.
    pub fn from_trits(trits: &[u8; DUAL_CHECKSUM_TRITS]) -> Option<Self> {
        for &t in trits.iter() {
            if t < 1 || t > 3 {
                return None;
            }
        }
        let mut repunit = [0u8; CHECKSUM_TRITS];
        let mut plenum = [0u8; CHECKSUM_TRITS];
        repunit.copy_from_slice(&trits[..CHECKSUM_TRITS]);
        plenum.copy_from_slice(&trits[CHECKSUM_TRITS..]);
        Some(WireChecksum { repunit, plenum })
    }

    /// Pack the 12 checksum trits into 3 wire bytes (2 bits per trit, 24 bits).
    ///
    /// Same encoding as `pack_trit_array`: {1,2,3} → {0b01, 0b10, 0b11}.
    pub fn to_wire_bytes(&self) -> [u8; 3] {
        let trits = self.to_trits();
        let mut packed: u32 = 0;
        for (i, &t) in trits.iter().enumerate() {
            packed |= (t as u32) << (22 - i * 2);
        }
        // 24 bits → 3 bytes big-endian
        [(packed >> 16) as u8, (packed >> 8) as u8, packed as u8]
    }

    /// Unpack 3 wire bytes into 12 checksum trits.
    ///
    /// Returns `None` if any 2-bit field is 0b00 (invalid Rep C).
    pub fn from_wire_bytes(bytes: &[u8; 3]) -> Option<Self> {
        let packed = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        let mut trits = [0u8; DUAL_CHECKSUM_TRITS];
        for i in 0..DUAL_CHECKSUM_TRITS {
            let val = ((packed >> (22 - i * 2)) & 0x03) as u8;
            if val == 0 {
                return None; // 0b00 = invalid Rep C
            }
            trits[i] = val;
        }
        Self::from_trits(&trits)
    }
}

/// Horner's method evaluation mod `modulus` for a Rep C trit slice.
///
/// Works on ANY length trit array (13-trit inter-cube, 27-trit TDNS, etc.).
/// Trits must be Rep C {1, 2, 3}. Internally converts to Rep B (subtract 1)
/// before polynomial evaluation — no domain crossing.
#[inline]
fn horner_mod(trits: &[u8], modulus: u32) -> u32 {
    let mut acc: u32 = 0;
    for &t in trits {
        let rep_b = (t - 1) as u32; // Rep C → Rep B: {1,2,3} → {0,1,2}
        acc = (acc * 3 + rep_b) % modulus;
    }
    acc
}

/// Decompose a checksum value (< 729 = 3⁶) into 6 Rep C trits.
#[inline]
fn decompose_to_rep_c(mut val: u32) -> [u8; CHECKSUM_TRITS] {
    let mut result = [0u8; CHECKSUM_TRITS];
    for i in (0..CHECKSUM_TRITS).rev() {
        result[i] = ((val % 3) + 1) as u8; // Rep B → Rep C: +1
        val /= 3;
    }
    result
}

/// Compute the dual checksum (mod-364 + mod-333) for a Rep C trit slice.
///
/// Works on any length: 13-trit inter-cube addresses, 27-trit TDNS addresses,
/// or any other Rep C trit array. Single pass through the data.
///
/// Returns `Err` if any trit is not in Rep C {1, 2, 3}.
pub fn compute_wire_checksum(trits: &[u8]) -> Result<WireChecksum, WireError> {
    // Validate Rep C
    for (i, &t) in trits.iter().enumerate() {
        if t < 1 || t > 3 {
            return Err(WireError::RepCZeroDetected);
        }
        let _ = i; // suppress unused warning
    }

    let repunit_val = horner_mod(trits, CHECKSUM_MOD_REPUNIT);
    let plenum_val = horner_mod(trits, CHECKSUM_MOD_PLENUM);

    Ok(WireChecksum {
        repunit: decompose_to_rep_c(repunit_val),
        plenum: decompose_to_rep_c(plenum_val),
    })
}

/// Verify a dual checksum against a Rep C trit slice.
///
/// Recomputes both checksums and compares in constant time.
/// Returns `true` if both match.
pub fn verify_wire_checksum(
    trits: &[u8],
    checksum: &WireChecksum,
) -> Result<bool, WireError> {
    let expected = compute_wire_checksum(trits)?;

    // Constant-time comparison
    let mut diff: u8 = 0;
    for i in 0..CHECKSUM_TRITS {
        diff |= expected.repunit[i] ^ checksum.repunit[i];
        diff |= expected.plenum[i] ^ checksum.plenum[i];
    }
    Ok(diff == 0)
}

/// Append a dual checksum to a packed wire address.
///
/// Takes a 4-byte packed address (from `pack_addr`), unpacks to trits,
/// computes the dual checksum, and returns 7 bytes:
/// `[addr(4) ‖ checksum(3)]`.
///
/// Returns `None` if the packed address contains invalid trits.
pub fn pack_addr_with_checksum(packed_addr: &[u8; WIRE_ADDR_SIZE]) -> Option<[u8; 7]> {
    let trits = unpack_trit_array(packed_addr)?;
    let checksum = compute_wire_checksum(&trits).ok()?;
    let ck_bytes = checksum.to_wire_bytes();

    let mut out = [0u8; 7];
    out[..4].copy_from_slice(packed_addr);
    out[4..7].copy_from_slice(&ck_bytes);
    Some(out)
}

/// Verify and strip a checksummed wire address.
///
/// Takes 7 bytes `[addr(4) ‖ checksum(3)]`, verifies the checksum,
/// and returns the 4-byte packed address if valid.
///
/// Returns `None` if the checksum is invalid or trits are not Rep C.
pub fn verify_and_strip_checksum(data: &[u8; 7]) -> Option<[u8; WIRE_ADDR_SIZE]> {
    let mut addr_bytes = [0u8; WIRE_ADDR_SIZE];
    addr_bytes.copy_from_slice(&data[..4]);

    let mut ck_bytes = [0u8; 3];
    ck_bytes.copy_from_slice(&data[4..7]);

    let trits = unpack_trit_array(&addr_bytes)?;
    let received_checksum = WireChecksum::from_wire_bytes(&ck_bytes)?;
    let valid = verify_wire_checksum(&trits, &received_checksum).ok()?;

    if valid {
        Some(addr_bytes)
    } else {
        None
    }
}

/// Size of a checksummed wire address: 4 (addr) + 3 (checksum) = 7 bytes.
pub const WIRE_ADDR_CHECKSUMMED_SIZE: usize = 7;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Header serialization round-trip ─────────────────────────

    #[test]
    fn test_header_roundtrip() {
        let ts: u128 = 42_000_000_000_000_000; // 42 seconds in fs
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
        let buf = [0u8; 10]; // too short
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
        // Truncate: header says 3 bytes payload, but we cut 1 byte
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
        msg.header.version = PROTOCOL_VERSION_V1; // Force v1

        let err = msg.validate().unwrap_err();
        assert!(matches!(err, WireError::MessageRequiresV2 { .. }));
    }

    #[test]
    fn test_validate_v3_message_on_v2_header() {
        let mut msg = WireMessage::new(
            MessageType::Array3Handshake,
            0,
            vec![],
        );
        msg.header.version = PROTOCOL_VERSION_V2;

        let err = msg.validate().unwrap_err();
        assert!(matches!(err, WireError::MessageRequiresV3 { .. }));
    }

    #[test]
    fn test_validate_v3_slot_register_on_v2_header() {
        let mut msg = WireMessage::new(
            MessageType::SlotRegister,
            0,
            vec![],
        );
        msg.header.version = PROTOCOL_VERSION_V2;

        let err = msg.validate().unwrap_err();
        assert!(matches!(err, WireError::MessageRequiresV3 { .. }));
    }

    #[test]
    fn test_validate_v3_message_on_v3_header_ok() {
        let msg = WireMessage::new(
            MessageType::Array3Handshake,
            0,
            vec![],
        );
        assert!(msg.validate().is_ok());
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
        // Zero in Rep C = forgery (INVARIANT 3)
        let trits: [u8; 13] = [1, 0, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        assert!(pack_trit_array(&trits).is_none());
    }

    #[test]
    fn test_trit_array_four_rejected() {
        // Value > 3 is invalid Rep C
        let trits: [u8; 13] = [1, 4, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        assert!(pack_trit_array(&trits).is_none());
    }

    #[test]
    fn test_unpack_zero_bits_rejected() {
        // Manually construct packed bytes with 0b00 in a trit position
        let mut packed = [0u8; 4];
        // trit 0 = 0b00, trit 1 = 0b01, ... — the 0b00 should be rejected
        packed[0] = 0b00_01_00_00; // trit 0 = 0, trit 1 = 1 — INVALID
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
        let ts = now - 10 * FS_PER_SECOND; // 10s old
        assert!(timestamp_in_window(ts, now));
    }

    #[test]
    fn test_timestamp_in_window_too_old() {
        let now = 1000 * FS_PER_SECOND;
        let ts = now - 60 * FS_PER_SECOND; // 60s old
        assert!(!timestamp_in_window(ts, now));
    }

    #[test]
    fn test_timestamp_in_window_slight_future() {
        let now = 1000 * FS_PER_SECOND;
        let ts = now + FS_PER_MILLISECOND * 500; // 0.5s in future
        assert!(timestamp_in_window(ts, now));
    }

    #[test]
    fn test_timestamp_in_window_too_far_future() {
        let now = 1000 * FS_PER_SECOND;
        let ts = now + 5 * FS_PER_SECOND; // 5s in future
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

    // ── T-10: Dual checksum wire integration ────────────────────

    #[test]
    fn test_compute_wire_checksum_13_trits() {
        let addr13: [u8; 13] = [2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let ck = compute_wire_checksum(&addr13).unwrap();
        // Both checksums are 6 Rep C trits
        for &t in ck.repunit.iter().chain(ck.plenum.iter()) {
            assert!(t >= 1 && t <= 3, "Checksum trits must be Rep C");
        }
    }

    #[test]
    fn test_compute_wire_checksum_27_trits() {
        // Works on TDNS addresses too
        let addr27: [u8; 27] = [
            2, 3, 2, 3, 1, 1, 3, 3, 3, 1, 3, 1, 1, 3, 2, 2,
            2, 3, 3, 1, 1, 2, 1, 2, 3, 1, 3,
        ];
        let ck = compute_wire_checksum(&addr27).unwrap();
        for &t in ck.repunit.iter().chain(ck.plenum.iter()) {
            assert!(t >= 1 && t <= 3);
        }
    }

    #[test]
    fn test_compute_wire_checksum_deterministic() {
        let addr: [u8; 13] = [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        let ck1 = compute_wire_checksum(&addr).unwrap();
        let ck2 = compute_wire_checksum(&addr).unwrap();
        assert_eq!(ck1, ck2);
    }

    #[test]
    fn test_compute_wire_checksum_different_addrs() {
        let a: [u8; 13] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let b: [u8; 13] = [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];
        let ck_a = compute_wire_checksum(&a).unwrap();
        let ck_b = compute_wire_checksum(&b).unwrap();
        // At least one of the two checksums should differ
        assert!(ck_a.repunit != ck_b.repunit || ck_a.plenum != ck_b.plenum);
    }

    #[test]
    fn test_verify_wire_checksum_valid() {
        let addr: [u8; 13] = [2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let ck = compute_wire_checksum(&addr).unwrap();
        assert!(verify_wire_checksum(&addr, &ck).unwrap());
    }

    #[test]
    fn test_verify_wire_checksum_tampered() {
        let addr: [u8; 13] = [2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let ck = compute_wire_checksum(&addr).unwrap();
        // Tamper with one trit
        let mut tampered = addr;
        tampered[6] = if tampered[6] == 1 { 2 } else { 1 };
        assert!(!verify_wire_checksum(&tampered, &ck).unwrap());
    }

    #[test]
    fn test_verify_wire_checksum_zero_trit_rejected() {
        let bad: [u8; 13] = [0, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3];
        assert!(compute_wire_checksum(&bad).is_err());
    }

    #[test]
    fn test_wire_checksum_to_trits_roundtrip() {
        let addr: [u8; 13] = [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        let ck = compute_wire_checksum(&addr).unwrap();
        let trits = ck.to_trits();
        assert_eq!(trits.len(), DUAL_CHECKSUM_TRITS);
        let recovered = WireChecksum::from_trits(&trits).unwrap();
        assert_eq!(ck, recovered);
    }

    #[test]
    fn test_wire_checksum_wire_bytes_roundtrip() {
        let addr: [u8; 13] = [3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3];
        let ck = compute_wire_checksum(&addr).unwrap();
        let wire = ck.to_wire_bytes();
        assert_eq!(wire.len(), 3);
        let recovered = WireChecksum::from_wire_bytes(&wire).unwrap();
        assert_eq!(ck, recovered);
    }

    #[test]
    fn test_pack_addr_with_checksum() {
        let trits: [u8; 13] = [2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let packed = pack_trit_array(&trits).unwrap();
        let checksummed = pack_addr_with_checksum(&packed).unwrap();
        assert_eq!(checksummed.len(), 7);
        assert_eq!(&checksummed[..4], &packed[..]);
    }

    #[test]
    fn test_verify_and_strip_checksum_valid() {
        let trits: [u8; 13] = [2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let packed = pack_trit_array(&trits).unwrap();
        let checksummed = pack_addr_with_checksum(&packed).unwrap();
        let recovered = verify_and_strip_checksum(&checksummed).unwrap();
        assert_eq!(recovered, packed);
    }

    #[test]
    fn test_verify_and_strip_checksum_tampered() {
        let trits: [u8; 13] = [2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let packed = pack_trit_array(&trits).unwrap();
        let mut checksummed = pack_addr_with_checksum(&packed).unwrap();
        // Flip a bit in the checksum bytes
        checksummed[5] ^= 0x0C;
        assert!(verify_and_strip_checksum(&checksummed).is_none());
    }

    #[test]
    fn test_checksum_constants() {
        assert_eq!(CHECKSUM_MOD_REPUNIT, 364);
        assert_eq!(CHECKSUM_MOD_PLENUM, 333);
        assert_eq!(CHECKSUM_DETECTION_SPACE, 121_212);
        assert_eq!(CHECKSUM_TRITS, 6);
        assert_eq!(DUAL_CHECKSUM_TRITS, 12);
        assert_eq!(WIRE_ADDR_CHECKSUMMED_SIZE, 7);
        // Coprimality: gcd(364, 333) = 1
        fn gcd(mut a: u32, mut b: u32) -> u32 {
            while b != 0 { let t = b; b = a % b; a = t; } a
        }
        assert_eq!(gcd(364, 333), 1, "Moduli must be coprime for CRT");
    }

    // ── Slot Address Wire Encoding (V3) ────────────────────────

    #[test]
    fn test_pack_slot_addr_roundtrip() {
        let slot = [1u8, 2, 3];
        let packed = pack_slot_addr(&slot).unwrap();
        let unpacked = unpack_slot_addr(packed).unwrap();
        assert_eq!(unpacked, slot);
    }

    #[test]
    fn test_pack_slot_addr_all_values() {
        for p in 1..=3u8 {
            for r in 1..=3u8 {
                for i in 1..=3u8 {
                    let slot = [p, r, i];
                    let packed = pack_slot_addr(&slot).unwrap();
                    let unpacked = unpack_slot_addr(packed).unwrap();
                    assert_eq!(unpacked, slot, "roundtrip failed for ({},{},{})", p, r, i);
                }
            }
        }
    }

    #[test]
    fn test_pack_slot_addr_zero_rejected() {
        assert!(pack_slot_addr(&[0, 1, 1]).is_none());
        assert!(pack_slot_addr(&[1, 0, 1]).is_none());
        assert!(pack_slot_addr(&[1, 1, 0]).is_none());
    }

    #[test]
    fn test_unpack_slot_addr_zero_rejected() {
        assert!(unpack_slot_addr(0b00_01_01_00).is_none()); // plane=0
        assert!(unpack_slot_addr(0b01_00_01_00).is_none()); // role=0
        assert!(unpack_slot_addr(0b01_01_00_00).is_none()); // instance=0
    }

    #[test]
    fn test_slot_addr_wire_constants() {
        assert_eq!(WIRE_SLOT_ADDR_SIZE, 1);
        assert_eq!(ARRAY3_HANDSHAKE_MIN_SIZE, 7);
        assert_eq!(SLOT_REGISTER_PAYLOAD_SIZE, 63);
    }
}