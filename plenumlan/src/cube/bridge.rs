// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Legacy Bridge Derivation (§8)
//!
//! Each service gets BOTH a native slot port (111xx) AND a legacy bridge
//! port (53/67/445/631/1812) if its classification matches. The bridge
//! table is a strict match on D5 (Interactivity), D6 (MediaRichness),
//! D12 (APIPresence), D15 (ProtocolLayering).

use super::constants::CLASSIFICATION_DIMS;
use super::projection::Trit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyBridge {
    Dns(u16),     // port 53
    Dhcp(u16),    // port 67
    Smb(u16),     // port 445
    Ipp(u16),     // port 631
    Radius(u16),  // port 1812
}

impl LegacyBridge {
    pub fn port(&self) -> u16 {
        match self {
            LegacyBridge::Dns(p) => *p,
            LegacyBridge::Dhcp(p) => *p,
            LegacyBridge::Smb(p) => *p,
            LegacyBridge::Ipp(p) => *p,
            LegacyBridge::Radius(p) => *p,
        }
    }

    pub fn protocol_name(&self) -> &'static str {
        match self {
            LegacyBridge::Dns(_) => "DNS",
            LegacyBridge::Dhcp(_) => "DHCP",
            LegacyBridge::Smb(_) => "SMB",
            LegacyBridge::Ipp(_) => "IPP",
            LegacyBridge::Radius(_) => "RADIUS",
        }
    }
}

/// Derive a legacy bridge port from 27 classification trits.
///
/// Array indices are 0-based; dimension numbers are 1-based (D5 = index 4).
/// Matches exact patterns on D5, D6, D12, D15:
///
/// | D5 | D6 | D12 | D15 | Bridge |
/// |----|-----|-----|-----|--------|
/// | 3  | 1   | 3   | 2   | DNS :53 |
/// | 3  | 1   | 3   | 1   | DHCP :67 |
/// | 2  | 1   | 1   | 2   | SMB :445 |
/// | 2  | 2   | 1   | _   | IPP :631 |
/// | 3  | 1   | 3   | 3   | RADIUS :1812 |
pub fn derive_legacy_bridge(classification: &[Trit; CLASSIFICATION_DIMS]) -> Option<LegacyBridge> {
    let d5  = classification[4];   // Interactivity (D5, 0-based index 4)
    let d6  = classification[5];   // MediaRichness (D6)
    let d12 = classification[11];  // APIPresence (D12)
    let d15 = classification[14];  // ProtocolLayering (D15)

    match (d5, d6, d12, d15) {
        (3, 1, 3, 2) => Some(LegacyBridge::Dns(53)),
        (3, 1, 3, 1) => Some(LegacyBridge::Dhcp(67)),
        (2, 1, 1, 2) => Some(LegacyBridge::Smb(445)),
        (2, 2, 1, _) => Some(LegacyBridge::Ipp(631)),
        (3, 1, 3, 3) => Some(LegacyBridge::Radius(1812)),
        _            => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_class(overrides: &[(usize, u8)]) -> [Trit; CLASSIFICATION_DIMS] {
        let mut class = [1u8; CLASSIFICATION_DIMS];
        for &(dim_1based, val) in overrides {
            class[dim_1based - 1] = val;
        }
        class
    }

    #[test]
    fn dns_bridge() {
        let class = make_class(&[(5, 3), (6, 1), (12, 3), (15, 2)]);
        let bridge = derive_legacy_bridge(&class).unwrap();
        assert_eq!(bridge, LegacyBridge::Dns(53));
        assert_eq!(bridge.port(), 53);
        assert_eq!(bridge.protocol_name(), "DNS");
    }

    #[test]
    fn dhcp_bridge() {
        let class = make_class(&[(5, 3), (6, 1), (12, 3), (15, 1)]);
        let bridge = derive_legacy_bridge(&class).unwrap();
        assert_eq!(bridge, LegacyBridge::Dhcp(67));
    }

    #[test]
    fn smb_bridge() {
        let class = make_class(&[(5, 2), (6, 1), (12, 1), (15, 2)]);
        let bridge = derive_legacy_bridge(&class).unwrap();
        assert_eq!(bridge, LegacyBridge::Smb(445));
    }

    #[test]
    fn ipp_bridge_any_d15() {
        for d15 in 1..=3u8 {
            let class = make_class(&[(5, 2), (6, 2), (12, 1), (15, d15)]);
            let bridge = derive_legacy_bridge(&class).unwrap();
            assert_eq!(bridge, LegacyBridge::Ipp(631));
        }
    }

    #[test]
    fn radius_bridge() {
        let class = make_class(&[(5, 3), (6, 1), (12, 3), (15, 3)]);
        let bridge = derive_legacy_bridge(&class).unwrap();
        assert_eq!(bridge, LegacyBridge::Radius(1812));
    }

    #[test]
    fn no_bridge_for_generic() {
        let class = [2u8; CLASSIFICATION_DIMS]; // center — no match
        assert!(derive_legacy_bridge(&class).is_none());
    }
}
