// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Windows Server Role Classification Matrix
//!
//! All 12 core Windows Server SMB roles mapped to 27 classification trits.
//! Validates plane assignments: Outer→Data, Void→Control, Inner→Management.

use super::constants::CLASSIFICATION_DIMS;
use super::projection::{Trit, SlotAddress, project_to_slot};

/// A Windows Server role with its full 27-trit classification.
#[derive(Debug, Clone)]
pub struct WindowsRole {
    pub name: &'static str,
    pub classification: [Trit; CLASSIFICATION_DIMS],
    pub expected_plane: Trit,  // 1=Data, 2=Control, 3=Management
}

// Helper to build classification arrays. Starts with base values,
// then applies specific overrides.
const fn make_class(base: [Trit; CLASSIFICATION_DIMS], _label: &str) -> [Trit; CLASSIFICATION_DIMS] {
    base
}

/// AD Domain Services (Active Directory)
/// Control plane: authentication, authorization, directory
pub const AD_DS: [Trit; CLASSIFICATION_DIMS] = [
//  D1  D2  D3  D4  D5  D6  D7  D8  D9  D10 D11 D12 D13 D14 D15 D16 D17 D18 D19 D20 D21 D22 D23 D24 D25 D26 D27
    3,  3,  3,  3,  3,  1,  3,  2,  3,  3,  3,  3,  1,  3,  3,  1,  3,  3,  3,  2,  2,  1,  2,  1,  3,  1,  3,
];

/// DNS Server
/// Control plane: name resolution, network infrastructure
pub const DNS_SERVER: [Trit; CLASSIFICATION_DIMS] = [
    3,  3,  3,  3,  3,  1,  3,  1,  3,  2,  3,  3,  1,  3,  2,  3,  3,  2,  3,  2,  2,  1,  3,  1,  3,  1,  3,
];

/// DHCP Server
/// Control plane: address allocation, network infrastructure
pub const DHCP_SERVER: [Trit; CLASSIFICATION_DIMS] = [
    3,  3,  3,  3,  3,  1,  2,  1,  2,  2,  2,  3,  2,  3,  1,  3,  2,  2,  2,  2,  1,  1,  3,  1,  2,  1,  3,
];

/// File Server (SMB)
/// Data plane: storage, file access
pub const FILE_SERVER: [Trit; CLASSIFICATION_DIMS] = [
    2,  3,  3,  3,  2,  1,  3,  1,  2,  2,  1,  1,  2,  3,  2,  2,  2,  3,  2,  2,  1,  1,  2,  1,  2,  1,  3,
];

/// Print Server
/// Data plane: output services
pub const PRINT_SERVER: [Trit; CLASSIFICATION_DIMS] = [
    2,  3,  3,  3,  2,  2,  1,  1,  1,  2,  1,  1,  2,  2,  2,  2,  2,  2,  1,  2,  1,  1,  2,  1,  2,  1,  3,
];

/// Web Server (IIS)
/// Data plane: content delivery
pub const WEB_SERVER: [Trit; CLASSIFICATION_DIMS] = [
    2,  3,  3,  3,  2,  3,  2,  2,  3,  2,  3,  2,  3,  3,  3,  3,  2,  3,  2,  2,  3,  2,  3,  2,  2,  2,  3,
];

/// Hyper-V (Virtualization)
/// Management plane: infrastructure management
pub const HYPER_V: [Trit; CLASSIFICATION_DIMS] = [
    3,  3,  3,  3,  3,  1,  3,  3,  3,  3,  3,  3,  1,  3,  3,  1,  3,  3,  3,  3,  1,  1,  1,  1,  3,  1,  3,
];

/// WSUS (Windows Server Update Services)
/// Management plane: patch distribution
pub const WSUS: [Trit; CLASSIFICATION_DIMS] = [
    3,  3,  3,  3,  2,  1,  3,  1,  2,  2,  2,  2,  2,  3,  2,  2,  3,  3,  3,  2,  1,  1,  3,  2,  3,  1,  3,
];

/// Certificate Authority (AD CS)
/// Control plane: PKI, certificate management
pub const CERT_AUTHORITY: [Trit; CLASSIFICATION_DIMS] = [
    3,  3,  3,  3,  3,  1,  3,  2,  3,  3,  3,  3,  1,  3,  3,  1,  3,  1,  3,  2,  2,  1,  1,  1,  3,  1,  3,
];

/// NPS (RADIUS)
/// Control plane: network access, authentication
pub const NPS_RADIUS: [Trit; CLASSIFICATION_DIMS] = [
    3,  3,  3,  3,  3,  1,  2,  2,  3,  3,  2,  3,  1,  3,  3,  1,  3,  2,  3,  2,  2,  1,  2,  1,  3,  1,  3,
];

/// Remote Desktop Services
/// Data plane: remote access, session management
pub const RDS: [Trit; CLASSIFICATION_DIMS] = [
    2,  3,  3,  3,  3,  3,  2,  2,  3,  3,  2,  2,  2,  3,  3,  2,  2,  2,  2,  3,  1,  2,  2,  2,  2,  1,  3,
];

/// Failover Clustering
/// Management plane: high availability, cluster management
pub const FAILOVER_CLUSTER: [Trit; CLASSIFICATION_DIMS] = [
    3,  3,  3,  3,  3,  1,  3,  3,  3,  3,  3,  3,  1,  3,  3,  1,  3,  2,  3,  3,  1,  1,  1,  1,  3,  1,  3,
];

/// Get all 12 Windows Server roles with their expected plane assignments.
pub fn all_windows_roles() -> Vec<WindowsRole> {
    vec![
        WindowsRole { name: "AD Domain Services", classification: AD_DS, expected_plane: 2 },          // Control
        WindowsRole { name: "DNS Server", classification: DNS_SERVER, expected_plane: 2 },              // Control
        WindowsRole { name: "DHCP Server", classification: DHCP_SERVER, expected_plane: 2 },            // Control
        WindowsRole { name: "File Server", classification: FILE_SERVER, expected_plane: 1 },            // Data
        WindowsRole { name: "Print Server", classification: PRINT_SERVER, expected_plane: 1 },          // Data
        WindowsRole { name: "Web Server (IIS)", classification: WEB_SERVER, expected_plane: 1 },        // Data
        WindowsRole { name: "Hyper-V", classification: HYPER_V, expected_plane: 3 },                    // Management
        WindowsRole { name: "WSUS", classification: WSUS, expected_plane: 3 },                          // Management
        WindowsRole { name: "Certificate Authority", classification: CERT_AUTHORITY, expected_plane: 2 }, // Control
        WindowsRole { name: "NPS (RADIUS)", classification: NPS_RADIUS, expected_plane: 2 },            // Control
        WindowsRole { name: "Remote Desktop Services", classification: RDS, expected_plane: 1 },        // Data
        WindowsRole { name: "Failover Clustering", classification: FAILOVER_CLUSTER, expected_plane: 3 }, // Management
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_roles_have_valid_rep_c() {
        for role in all_windows_roles() {
            for (i, &t) in role.classification.iter().enumerate() {
                assert!(t >= 1 && t <= 3,
                    "{}: D{} has invalid Rep C value {}", role.name, i + 1, t);
            }
        }
    }

    #[test]
    fn all_roles_project_to_expected_plane() {
        for role in all_windows_roles() {
            let slot = project_to_slot(&role.classification)
                .unwrap_or_else(|| panic!("{}: projection returned None", role.name));
            assert_eq!(slot.plane, role.expected_plane,
                "{}: expected plane {} ({}), got {} ({})",
                role.name,
                role.expected_plane,
                match role.expected_plane { 1 => "Data", 2 => "Control", 3 => "Management", _ => "?" },
                slot.plane,
                match slot.plane { 1 => "Data", 2 => "Control", 3 => "Management", _ => "?" }
            );
        }
    }

    #[test]
    fn twelve_roles_total() {
        assert_eq!(all_windows_roles().len(), 12);
    }

    #[test]
    fn dns_gets_legacy_bridge() {
        use super::super::bridge::derive_legacy_bridge;
        let bridge = derive_legacy_bridge(&DNS_SERVER);
        assert!(bridge.is_some(), "DNS should get a legacy bridge");
        assert_eq!(bridge.unwrap().port(), 53);
    }

    #[test]
    fn dhcp_gets_legacy_bridge() {
        use super::super::bridge::derive_legacy_bridge;
        let bridge = derive_legacy_bridge(&DHCP_SERVER);
        assert!(bridge.is_some(), "DHCP should get a legacy bridge");
        assert_eq!(bridge.unwrap().port(), 67);
    }

    #[test]
    fn file_server_gets_smb_bridge() {
        use super::super::bridge::derive_legacy_bridge;
        let bridge = derive_legacy_bridge(&FILE_SERVER);
        assert!(bridge.is_some(), "File Server should get SMB bridge");
        assert_eq!(bridge.unwrap().port(), 445);
    }

    #[test]
    fn print_server_gets_ipp_bridge() {
        use super::super::bridge::derive_legacy_bridge;
        let bridge = derive_legacy_bridge(&PRINT_SERVER);
        assert!(bridge.is_some(), "Print Server should get IPP bridge");
        assert_eq!(bridge.unwrap().port(), 631);
    }

    #[test]
    fn radius_gets_legacy_bridge() {
        use super::super::bridge::derive_legacy_bridge;
        let bridge = derive_legacy_bridge(&NPS_RADIUS);
        assert!(bridge.is_some(), "NPS/RADIUS should get RADIUS bridge");
        assert_eq!(bridge.unwrap().port(), 1812);
    }
}
