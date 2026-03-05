// TDNS v2.3 — Wire Protocol
// Capomastro Holdings Ltd. — Applied Physics Division
//
// Binary packet framing for the TDNS data plane.
//
// Every packet flowing through the ternary hypercube has this structure:
//
//   ┌─────────────────────────────────────────────────────────┐
//   │ Header (fixed 32 bytes)                                 │
//   │   version:       u8     (protocol version = 0x23)       │
//   │   packet_type:   u8     (point/multicast/heartbeat/...) │
//   │   flags:         u16    (HPTP-mandatory, encrypted, ...) │
//   │   source:        7 bytes (27-trit wire-encoded address)  │
//   │   destination:   7 bytes (27-trit or subcube base)       │
//   │   mask:          4 bytes (subcube mask, 0xFFFFFFFF=point) │
//   │   timestamp_ns:  u64    (HPTP nanosecond timestamp)      │
//   ├─────────────────────────────────────────────────────────┤
//   │ Payload length:  u16    (0–65535 bytes)                  │
//   │ Payload:         [u8]   (variable length)                │
//   ├─────────────────────────────────────────────────────────┤
//   │ Integrity:       32 bytes (BLAKE3 hash of header+payload)│
//   └─────────────────────────────────────────────────────────┘
//
// Total overhead: 32 (header) + 2 (length) + 32 (integrity) = 66 bytes.

use crate::addr::{CubeAddr, WIRE_SIZE};
use crate::subcube::SubCube;

pub const PROTOCOL_VERSION: u8 = 0x23;

pub const HEADER_SIZE: usize = 32;

pub const INTEGRITY_SIZE: usize = 32;

pub const MIN_PACKET_SIZE: usize = HEADER_SIZE + 2 + INTEGRITY_SIZE;

pub const MAX_PAYLOAD_SIZE: usize = 65535;

const POINT_MASK: u32 = 0x07FF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    Data = 0x01,
    Multicast = 0x02,
    Heartbeat = 0x03,
    TrnQuery = 0x04,
    TrnResponse = 0x05,
    RescanNotify = 0x06,
    DriftRedirect = 0x07,
    HptpProbe = 0x08,
    HptpResponse = 0x09,
    NeighborUpdate = 0x0A,
    OwnershipChallenge = 0x0B,
}

impl PacketType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Data),
            0x02 => Some(Self::Multicast),
            0x03 => Some(Self::Heartbeat),
            0x04 => Some(Self::TrnQuery),
            0x05 => Some(Self::TrnResponse),
            0x06 => Some(Self::RescanNotify),
            0x07 => Some(Self::DriftRedirect),
            0x08 => Some(Self::HptpProbe),
            0x09 => Some(Self::HptpResponse),
            0x0A => Some(Self::NeighborUpdate),
            0x0B => Some(Self::OwnershipChallenge),
            _ => None,
        }
    }
}

pub mod flags {
    pub const HPTP_MANDATORY: u16 = 1 << 0;
    pub const ENCRYPTED: u16 = 1 << 1;
    pub const HPTP_VERIFY: u16 = 1 << 2;
    pub const FORWARDED: u16 = 1 << 3;
    pub const REDIRECT: u16 = 1 << 4;
    pub const NO_FORWARD: u16 = 1 << 5;
    pub const ANYCAST: u16 = 1 << 6;
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub version: u8,
    pub packet_type: PacketType,
    pub flags: u16,
    pub source: CubeAddr,
    pub destination: CubeAddr,
    pub mask: u32,
    pub timestamp_ns: u64,

    pub payload: Vec<u8>,

    pub integrity: [u8; INTEGRITY_SIZE],
}

impl Packet {
    pub fn data(
        source: CubeAddr,
        destination: CubeAddr,
        payload: Vec<u8>,
        timestamp_ns: u64,
    ) -> Self {
        let mut flags = 0u16;
        if source.is_hptp_mandatory() {
            flags |= flags::HPTP_MANDATORY;
        }
        if destination.is_hptp_mandatory() {
            flags |= flags::HPTP_VERIFY;
        }

        let mut pkt = Self {
            version: PROTOCOL_VERSION,
            packet_type: PacketType::Data,
            flags,
            source,
            destination,
            mask: POINT_MASK,
            timestamp_ns,
            payload,
            integrity: [0u8; INTEGRITY_SIZE],
        };
        pkt.compute_integrity();
        pkt
    }

    pub fn multicast(
        source: CubeAddr,
        subcube: &SubCube,
        payload: Vec<u8>,
        timestamp_ns: u64,
    ) -> Self {
        let mut flags = 0u16;
        if source.is_hptp_mandatory() {
            flags |= flags::HPTP_MANDATORY;
        }

        let mask_bits = subcube_to_mask_u32(subcube);

        let mut pkt = Self {
            version: PROTOCOL_VERSION,
            packet_type: PacketType::Multicast,
            flags,
            source,
            destination: *subcube.base(),
            mask: mask_bits,
            timestamp_ns,
            payload,
            integrity: [0u8; INTEGRITY_SIZE],
        };
        pkt.compute_integrity();
        pkt
    }

    pub fn heartbeat(
        source: CubeAddr,
        hptp_offset_ns: i64,
        sequence: u64,
        timestamp_ns: u64,
    ) -> Self {
        let mut flags = 0u16;
        if source.is_hptp_mandatory() {
            flags |= flags::HPTP_MANDATORY;
        }

        let mut payload = Vec::with_capacity(16);
        payload.extend_from_slice(&hptp_offset_ns.to_be_bytes());
        payload.extend_from_slice(&sequence.to_be_bytes());

        let mut pkt = Self {
            version: PROTOCOL_VERSION,
            packet_type: PacketType::Heartbeat,
            flags,
            source,
            destination: source,
            mask: POINT_MASK,
            timestamp_ns,
            payload,
            integrity: [0u8; INTEGRITY_SIZE],
        };
        pkt.compute_integrity();
        pkt
    }

    pub fn is_point(&self) -> bool {
        self.mask == POINT_MASK
    }

    pub fn is_multicast(&self) -> bool {
        self.mask != POINT_MASK
    }

    pub fn requires_hptp(&self) -> bool {
        self.flags & flags::HPTP_VERIFY != 0
    }

    pub fn is_encrypted(&self) -> bool {
        self.flags & flags::ENCRYPTED != 0
    }

    pub fn is_forwarded(&self) -> bool {
        self.flags & flags::FORWARDED != 0
    }

    pub fn mark_forwarded(&mut self) {
        self.flags |= flags::FORWARDED;
        self.compute_integrity();
    }

    pub fn wire_size(&self) -> usize {
        HEADER_SIZE + 2 + self.payload.len() + INTEGRITY_SIZE
    }

    pub fn to_wire(&self) -> Vec<u8> {
        let total = self.wire_size();
        let mut buf = Vec::with_capacity(total);

        buf.push(self.version);
        buf.push(self.packet_type as u8);
        buf.extend_from_slice(&self.flags.to_be_bytes());
        buf.extend_from_slice(&self.source.to_wire());
        buf.extend_from_slice(&self.destination.to_wire());
        buf.extend_from_slice(&self.mask.to_be_bytes());
        buf.extend_from_slice(&self.timestamp_ns.to_be_bytes());

        buf.push(0x00);
        buf.push(0x00);

        let payload_len = self.payload.len() as u16;
        buf.extend_from_slice(&payload_len.to_be_bytes());

        buf.extend_from_slice(&self.payload);

        buf.extend_from_slice(&self.integrity);

        buf
    }

    pub fn from_wire(buf: &[u8]) -> Result<Self, WireError> {
        if buf.len() < MIN_PACKET_SIZE {
            return Err(WireError::TooShort {
                expected: MIN_PACKET_SIZE,
                got: buf.len(),
            });
        }

        let version = buf[0];
        if version != PROTOCOL_VERSION {
            return Err(WireError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                got: version,
            });
        }

        let packet_type = PacketType::from_u8(buf[1]).ok_or(WireError::InvalidPacketType(buf[1]))?;
        let flags = u16::from_be_bytes([buf[2], buf[3]]);

        let mut src_bytes = [0u8; WIRE_SIZE];
        src_bytes.copy_from_slice(&buf[4..11]);
        let source = CubeAddr::from_wire(&src_bytes)
            .map_err(|e| WireError::InvalidAddress(format!("source: {}", e)))?;

        let mut dst_bytes = [0u8; WIRE_SIZE];
        dst_bytes.copy_from_slice(&buf[11..18]);
        let destination = CubeAddr::from_wire(&dst_bytes)
            .map_err(|e| WireError::InvalidAddress(format!("destination: {}", e)))?;

        let mask = u32::from_be_bytes([buf[18], buf[19], buf[20], buf[21]]);
        let timestamp_ns = u64::from_be_bytes([
            buf[22], buf[23], buf[24], buf[25], buf[26], buf[27], buf[28], buf[29],
        ]);

        let payload_len = u16::from_be_bytes([buf[32], buf[33]]) as usize;
        let payload_start = 34;
        let payload_end = payload_start + payload_len;

        if buf.len() < payload_end + INTEGRITY_SIZE {
            return Err(WireError::TooShort {
                expected: payload_end + INTEGRITY_SIZE,
                got: buf.len(),
            });
        }

        let payload = buf[payload_start..payload_end].to_vec();

        let mut integrity = [0u8; INTEGRITY_SIZE];
        integrity.copy_from_slice(&buf[payload_end..payload_end + INTEGRITY_SIZE]);

        let pkt = Self {
            version,
            packet_type,
            flags,
            source,
            destination,
            mask,
            timestamp_ns,
            payload,
            integrity,
        };

        if !pkt.verify_integrity() {
            return Err(WireError::IntegrityCheckFailed);
        }

        Ok(pkt)
    }

    fn compute_integrity(&mut self) {
        let mut data = Vec::with_capacity(HEADER_SIZE + self.payload.len());

        data.push(self.version);
        data.push(self.packet_type as u8);
        data.extend_from_slice(&self.flags.to_be_bytes());
        data.extend_from_slice(&self.source.to_wire());
        data.extend_from_slice(&self.destination.to_wire());
        data.extend_from_slice(&self.mask.to_be_bytes());
        data.extend_from_slice(&self.timestamp_ns.to_be_bytes());
        data.push(0x00);
        data.push(0x00);

        data.extend_from_slice(&self.payload);

        let hash = blake3::hash(&data);
        self.integrity.copy_from_slice(hash.as_bytes());
    }

    pub fn verify_integrity(&self) -> bool {
        let mut data = Vec::with_capacity(HEADER_SIZE + self.payload.len());

        data.push(self.version);
        data.push(self.packet_type as u8);
        data.extend_from_slice(&self.flags.to_be_bytes());
        data.extend_from_slice(&self.source.to_wire());
        data.extend_from_slice(&self.destination.to_wire());
        data.extend_from_slice(&self.mask.to_be_bytes());
        data.extend_from_slice(&self.timestamp_ns.to_be_bytes());
        data.push(0x00);
        data.push(0x00);
        data.extend_from_slice(&self.payload);

        let hash = blake3::hash(&data);
        hash.as_bytes() == &self.integrity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    TooShort { expected: usize, got: usize },
    VersionMismatch { expected: u8, got: u8 },
    InvalidPacketType(u8),
    InvalidAddress(String),
    IntegrityCheckFailed,
    PayloadTooLarge(usize),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::TooShort { expected, got } => {
                write!(f, "packet too short: expected {} bytes, got {}", expected, got)
            }
            WireError::VersionMismatch { expected, got } => {
                write!(f, "version mismatch: expected 0x{:02X}, got 0x{:02X}", expected, got)
            }
            WireError::InvalidPacketType(t) => write!(f, "invalid packet type: 0x{:02X}", t),
            WireError::InvalidAddress(s) => write!(f, "invalid address: {}", s),
            WireError::IntegrityCheckFailed => write!(f, "integrity check failed"),
            WireError::PayloadTooLarge(s) => write!(f, "payload too large: {} bytes", s),
        }
    }
}

fn subcube_to_mask_u32(subcube: &SubCube) -> u32 {
    let mut bits: u32 = 0;
    for i in 0..27 {
        if subcube.is_constrained(i) {
            bits |= 1 << (26 - i);
        }
    }
    bits
}

pub fn parse_heartbeat_payload(payload: &[u8]) -> Option<(i64, u64)> {
    if payload.len() < 16 {
        return None;
    }
    let offset = i64::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
        payload[4], payload[5], payload[6], payload[7],
    ]);
    let sequence = u64::from_be_bytes([
        payload[8], payload[9], payload[10], payload[11],
        payload[12], payload[13], payload[14], payload[15],
    ]);
    Some((offset, sequence))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::DIMENSIONS;

    fn google() -> CubeAddr {
        CubeAddr::from_category_string("WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313")
            .unwrap()
    }

    fn pptpro() -> CubeAddr {
        CubeAddr::from_category_string("WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332")
            .unwrap()
    }

    #[test]
    fn data_packet_roundtrip() {
        let pkt = Packet::data(google(), pptpro(), b"hello plenum".to_vec(), 1_000_000);
        let wire = pkt.to_wire();
        let decoded = Packet::from_wire(&wire).unwrap();

        assert_eq!(decoded.version, PROTOCOL_VERSION);
        assert_eq!(decoded.packet_type, PacketType::Data);
        assert_eq!(decoded.source, google());
        assert_eq!(decoded.destination, pptpro());
        assert_eq!(decoded.payload, b"hello plenum");
        assert_eq!(decoded.timestamp_ns, 1_000_000);
        assert!(decoded.is_point());
    }

    #[test]
    fn multicast_packet_roundtrip() {
        let mut mask = [false; DIMENSIONS];
        mask[15] = true;
        let sc = SubCube::new(pptpro(), mask);

        let pkt = Packet::multicast(google(), &sc, b"broadcast".to_vec(), 2_000_000);
        let wire = pkt.to_wire();
        let decoded = Packet::from_wire(&wire).unwrap();

        assert_eq!(decoded.packet_type, PacketType::Multicast);
        assert!(decoded.is_multicast());
        assert!(!decoded.is_point());
        assert_eq!(decoded.payload, b"broadcast");
    }

    #[test]
    fn heartbeat_packet_roundtrip() {
        let pkt = Packet::heartbeat(pptpro(), -500, 42, 3_000_000);
        let wire = pkt.to_wire();
        let decoded = Packet::from_wire(&wire).unwrap();

        assert_eq!(decoded.packet_type, PacketType::Heartbeat);
        assert!(decoded.flags & flags::HPTP_MANDATORY != 0);

        let (offset, seq) = parse_heartbeat_payload(&decoded.payload).unwrap();
        assert_eq!(offset, -500);
        assert_eq!(seq, 42);
    }

    #[test]
    fn integrity_verification() {
        let pkt = Packet::data(google(), pptpro(), b"test".to_vec(), 100);
        assert!(pkt.verify_integrity());

        let mut tampered = pkt.clone();
        tampered.payload = b"tampered".to_vec();
        assert!(!tampered.verify_integrity());
    }

    #[test]
    fn integrity_rejects_corrupted_wire() {
        let pkt = Packet::data(google(), pptpro(), b"test".to_vec(), 100);
        let mut wire = pkt.to_wire();

        let payload_offset = HEADER_SIZE + 2;
        wire[payload_offset] ^= 0xFF;

        let result = Packet::from_wire(&wire);
        assert_eq!(result.unwrap_err(), WireError::IntegrityCheckFailed);
    }

    #[test]
    fn version_mismatch_rejected() {
        let pkt = Packet::data(google(), pptpro(), b"test".to_vec(), 100);
        let mut wire = pkt.to_wire();
        wire[0] = 0xFF;

        let result = Packet::from_wire(&wire);
        assert!(matches!(result, Err(WireError::VersionMismatch { .. })));
    }

    #[test]
    fn too_short_rejected() {
        let result = Packet::from_wire(&[0u8; 10]);
        assert!(matches!(result, Err(WireError::TooShort { .. })));
    }

    #[test]
    fn hptp_flags_auto_set() {
        let pkt = Packet::data(pptpro(), google(), b"data".to_vec(), 100);
        assert!(pkt.flags & flags::HPTP_MANDATORY != 0);

        assert!(pkt.flags & flags::HPTP_VERIFY == 0);

        let pkt2 = Packet::data(google(), pptpro(), b"data".to_vec(), 100);
        assert!(pkt2.flags & flags::HPTP_VERIFY != 0);
    }

    #[test]
    fn mark_forwarded() {
        let mut pkt = Packet::data(google(), pptpro(), b"data".to_vec(), 100);
        assert!(!pkt.is_forwarded());

        pkt.mark_forwarded();
        assert!(pkt.is_forwarded());
        assert!(pkt.verify_integrity());
    }

    #[test]
    fn wire_size_calculation() {
        let pkt = Packet::data(google(), pptpro(), b"hello".to_vec(), 100);
        let expected = HEADER_SIZE + 2 + 5 + INTEGRITY_SIZE;
        assert_eq!(pkt.wire_size(), expected);
        assert_eq!(pkt.to_wire().len(), expected);
    }

    #[test]
    fn empty_payload() {
        let pkt = Packet::data(google(), pptpro(), vec![], 100);
        let wire = pkt.to_wire();
        let decoded = Packet::from_wire(&wire).unwrap();
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn all_packet_types_valid() {
        for i in 0x01..=0x0Bu8 {
            assert!(PacketType::from_u8(i).is_some(), "0x{:02X} should be valid", i);
        }
        assert!(PacketType::from_u8(0x00).is_none());
        assert!(PacketType::from_u8(0xFF).is_none());
    }

    #[test]
    fn point_mask_value() {
        assert_eq!(POINT_MASK, (1u32 << 27) - 1);
    }
}
