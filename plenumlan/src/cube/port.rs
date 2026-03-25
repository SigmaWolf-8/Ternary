// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Port Formula
//!
//! ```text
//! port = BASE_PORT + ((node_id - 1) × SLOTS_PER_NODE)
//!                  + ((plane - 1) × SLOTS_PER_PLANE)
//!                  + ((role - 1) × GF3_ORDER)
//!                  + (instance - 1)
//! ```
//!
//! All inputs are Rep C {1, 2, 3}. The `- 1` converts each to GF(3) offset
//! for arithmetic. The wire carries Rep C. The formula uses SLOTS_PER_PLANE
//! for the plane stride (9 = role × instance slots per plane), NOT
//! SHELL_FACTORS (9 = auth shell factors). Same value, different derivation.

use super::constants::{
    BASE_PORT, GF3_ORDER, SLOTS_PER_NODE, SLOTS_PER_PLANE,
    MAX_NODES, GATEWAY_OFFSET,
};
use super::projection::SlotAddress;

/// Compute the TCP port for a slot on a given node.
///
/// `node_id`: Rep C {1, 2, 3}. Node 1 is the gateway.
/// `slot`: 3-trit Rep C slot address.
///
/// Returns `None` if node_id is not in {1, 2, 3} (zero-sentinel violation).
pub fn slot_port(node_id: u8, slot: &SlotAddress) -> Option<u16> {
    if node_id < 1 || node_id > MAX_NODES as u8 {
        return None; // zero-sentinel: reject node_id=0 or >3
    }

    let node_offset = (node_id as u16 - 1) * SLOTS_PER_NODE as u16;
    let plane_offset = (slot.plane as u16 - 1) * SLOTS_PER_PLANE as u16;
    let role_offset = (slot.role as u16 - 1) * GF3_ORDER as u16;
    let instance_offset = slot.instance as u16 - 1;

    Some(BASE_PORT + node_offset + plane_offset + role_offset + instance_offset)
}

/// Compute the gateway port for a given node.
/// The gateway is at offset GATEWAY_OFFSET (13) = slot (2,2,2) = the cube center.
/// Only Node 1's gateway is the cluster gateway; Nodes 2-3 have regular slots at +13.
pub fn gateway_port(node_id: u8) -> Option<u16> {
    if node_id < 1 || node_id > MAX_NODES as u8 {
        return None;
    }
    Some(BASE_PORT + (node_id as u16 - 1) * SLOTS_PER_NODE as u16 + GATEWAY_OFFSET as u16)
}

/// Compute the port range [start, end] (inclusive) for a node.
pub fn node_port_range(node_id: u8) -> Option<(u16, u16)> {
    if node_id < 1 || node_id > MAX_NODES as u8 {
        return None;
    }
    let start = BASE_PORT + (node_id as u16 - 1) * SLOTS_PER_NODE as u16;
    let end = start + SLOTS_PER_NODE as u16 - 1;
    Some((start, end))
}

/// Decode a port back to (node_id, SlotAddress).
/// Returns `None` if the port is outside the Array3 range.
pub fn port_to_slot(port: u16) -> Option<(u8, SlotAddress)> {
    if port < BASE_PORT {
        return None;
    }
    let offset = (port - BASE_PORT) as usize;
    if offset >= SLOTS_PER_NODE * MAX_NODES {
        return None;
    }

    let node_index = offset / SLOTS_PER_NODE;       // 0, 1, or 2
    let slot_offset = offset % SLOTS_PER_NODE;       // 0..26

    let plane_gf3 = slot_offset / SLOTS_PER_PLANE;   // 0, 1, or 2
    let remainder = slot_offset % SLOTS_PER_PLANE;
    let role_gf3 = remainder / GF3_ORDER;             // 0, 1, or 2
    let instance_gf3 = remainder % GF3_ORDER;          // 0, 1, or 2

    let node_id = node_index as u8 + 1;               // Rep C
    let slot = SlotAddress::new(
        plane_gf3 as u8 + 1,  // Rep C
        role_gf3 as u8 + 1,
        instance_gf3 as u8 + 1,
    );

    Some((node_id, slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node1_data_primary_1() {
        let slot = SlotAddress::new(1, 1, 1);
        assert_eq!(slot_port(1, &slot), Some(11111));
    }

    #[test]
    fn node1_gateway_center() {
        let slot = SlotAddress::new(2, 2, 2);
        assert_eq!(slot_port(1, &slot), Some(11124)); // BASE_PORT + 13
        assert_eq!(gateway_port(1), Some(11124));
    }

    #[test]
    fn node1_last_slot() {
        let slot = SlotAddress::new(3, 3, 3);
        assert_eq!(slot_port(1, &slot), Some(11137)); // BASE_PORT + 26
    }

    #[test]
    fn node2_first_slot() {
        let slot = SlotAddress::new(1, 1, 1);
        assert_eq!(slot_port(2, &slot), Some(11138)); // BASE_PORT + 27
    }

    #[test]
    fn node2_center() {
        assert_eq!(gateway_port(2), Some(11151)); // BASE_PORT + 27 + 13
    }

    #[test]
    fn node3_first_slot() {
        let slot = SlotAddress::new(1, 1, 1);
        assert_eq!(slot_port(3, &slot), Some(11165)); // BASE_PORT + 54
    }

    #[test]
    fn node3_center() {
        assert_eq!(gateway_port(3), Some(11178)); // BASE_PORT + 54 + 13
    }

    #[test]
    fn node3_last_slot() {
        let slot = SlotAddress::new(3, 3, 3);
        assert_eq!(slot_port(3, &slot), Some(11191)); // BASE_PORT + 80
    }

    #[test]
    fn node_port_ranges() {
        assert_eq!(node_port_range(1), Some((11111, 11137)));
        assert_eq!(node_port_range(2), Some((11138, 11164)));
        assert_eq!(node_port_range(3), Some((11165, 11191)));
    }

    #[test]
    fn rejects_node_id_zero() {
        let slot = SlotAddress::new(1, 1, 1);
        assert!(slot_port(0, &slot).is_none());
        assert!(gateway_port(0).is_none());
        assert!(node_port_range(0).is_none());
    }

    #[test]
    fn rejects_node_id_four() {
        let slot = SlotAddress::new(1, 1, 1);
        assert!(slot_port(4, &slot).is_none());
    }

    #[test]
    fn full_port_table_27_slots() {
        // Verify all 27 slots for Node 1 against the spec port table
        let expected: [(u8, u8, u8, u16); 27] = [
            (1,1,1, 11111), (1,1,2, 11112), (1,1,3, 11113),
            (1,2,1, 11114), (1,2,2, 11115), (1,2,3, 11116),
            (1,3,1, 11117), (1,3,2, 11118), (1,3,3, 11119),
            (2,1,1, 11120), (2,1,2, 11121), (2,1,3, 11122),
            (2,2,1, 11123), (2,2,2, 11124), (2,2,3, 11125),
            (2,3,1, 11126), (2,3,2, 11127), (2,3,3, 11128),
            (3,1,1, 11129), (3,1,2, 11130), (3,1,3, 11131),
            (3,2,1, 11132), (3,2,2, 11133), (3,2,3, 11134),
            (3,3,1, 11135), (3,3,2, 11136), (3,3,3, 11137),
        ];
        for (p, r, i, port) in expected {
            let slot = SlotAddress::new(p, r, i);
            assert_eq!(slot_port(1, &slot), Some(port),
                "Node 1 slot ({},{},{}) expected port {}", p, r, i, port);
        }
    }

    #[test]
    fn full_81_port_coverage() {
        let mut ports = std::collections::HashSet::new();
        for node_id in 1..=3u8 {
            for p in 1..=3u8 {
                for r in 1..=3u8 {
                    for i in 1..=3u8 {
                        let slot = SlotAddress::new(p, r, i);
                        let port = slot_port(node_id, &slot).unwrap();
                        assert!(port >= 11111 && port <= 11191,
                            "port {} out of Array3 range", port);
                        assert!(ports.insert(port),
                            "duplicate port {} for node {} slot ({},{},{})",
                            port, node_id, p, r, i);
                    }
                }
            }
        }
        assert_eq!(ports.len(), 81);
    }

    #[test]
    fn round_trip_all_81_ports() {
        for node_id in 1..=3u8 {
            for p in 1..=3u8 {
                for r in 1..=3u8 {
                    for i in 1..=3u8 {
                        let slot = SlotAddress::new(p, r, i);
                        let port = slot_port(node_id, &slot).unwrap();
                        let (decoded_node, decoded_slot) = port_to_slot(port).unwrap();
                        assert_eq!(decoded_node, node_id);
                        assert_eq!(decoded_slot, slot);
                    }
                }
            }
        }
    }
}
