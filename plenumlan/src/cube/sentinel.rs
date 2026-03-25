// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Zero-Sentinel Pre-Filter
//!
//! At every trust boundary: if ANY trit in a Rep C value is 0, reject
//! immediately before any cryptographic operation. Rep C values are
//! {1, 2, 3} — zero is structurally impossible and its presence is
//! proof of forgery.
//!
//! This module re-exports `is_valid_rep_c` from ternary-math and adds
//! typed wrappers for each trust boundary.

pub use ternary_math::plenum_checksum::is_valid_rep_c;

use super::constants::CUBE_DIMS;
use super::projection::Trit;

/// Validate a 3-trit slot destination in a multiplexing header.
/// Returns false if any trit is 0 or > 3.
pub fn validate_mux_header(slot_dest: &[Trit; CUBE_DIMS]) -> bool {
    is_valid_rep_c(slot_dest)
}

/// Validate a caller identity (54-trit TDNS-L address).
pub fn validate_caller_identity(address: &[u8]) -> bool {
    if address.len() != 54 {
        return false;
    }
    is_valid_rep_c(address)
}

/// Validate a node ID in an Array3 handshake.
/// Node IDs are single Rep C trits {1, 2, 3}. Zero = forgery.
pub fn validate_node_id(node_id: u8) -> bool {
    node_id >= 1 && node_id <= 3
}

/// Validate a capability token's subject/object fields.
pub fn validate_capability_field(field: &[u8]) -> bool {
    if field.is_empty() {
        return false;
    }
    is_valid_rep_c(field)
}

/// Validate inter-service routing message addresses.
pub fn validate_routing_address(address: &[u8]) -> bool {
    if address.is_empty() {
        return false;
    }
    is_valid_rep_c(address)
}

/// Validate a redirect chain entry.
pub fn validate_redirect_entry(address: &[u8]) -> bool {
    if address.is_empty() {
        return false;
    }
    is_valid_rep_c(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_mux_header() {
        assert!(validate_mux_header(&[1, 2, 3]));
        assert!(validate_mux_header(&[2, 2, 2]));
    }

    #[test]
    fn invalid_mux_header_zero() {
        assert!(!validate_mux_header(&[0, 2, 3]));
        assert!(!validate_mux_header(&[1, 0, 3]));
        assert!(!validate_mux_header(&[1, 2, 0]));
    }

    #[test]
    fn valid_node_ids() {
        assert!(validate_node_id(1));
        assert!(validate_node_id(2));
        assert!(validate_node_id(3));
    }

    #[test]
    fn invalid_node_id_zero() {
        assert!(!validate_node_id(0));
    }

    #[test]
    fn invalid_node_id_four() {
        assert!(!validate_node_id(4));
    }

    #[test]
    fn valid_caller_identity() {
        let addr = [2u8; 54];
        assert!(validate_caller_identity(&addr));
    }

    #[test]
    fn invalid_caller_identity_wrong_length() {
        let addr = [2u8; 53];
        assert!(!validate_caller_identity(&addr));
    }

    #[test]
    fn invalid_caller_identity_zero_trit() {
        let mut addr = [2u8; 54];
        addr[27] = 0;
        assert!(!validate_caller_identity(&addr));
    }

    #[test]
    fn empty_capability_field_rejected() {
        assert!(!validate_capability_field(&[]));
    }

    #[test]
    fn valid_capability_field() {
        assert!(validate_capability_field(&[1, 2, 3, 1]));
    }
}
