// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Array3 Node Identity and Formation
//!
//! CUBE_NODE_ID env var (Rep C: {1, 2, 3}, default 1).
//! Node 1 is always the cluster gateway.
//! CUBE_NODE_ID=0 is a fatal error, not a default — zero-sentinel check.
//!
//! CUBE_ARRAY3_PEERS=ip1:port,ip2:port for peer discovery.
//! EAGER_BIND env flag: bind-all-at-startup (production) vs bind-on-register (Replit).

use super::constants::{
    BASE_PORT, GF3_ORDER, GATEWAY_NODE_ID, MAX_NODES, SLOTS_PER_NODE,
};
use super::port::node_port_range;

/// Binding strategy for slot ports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindStrategy {
    EagerBind,    // Bind all 27 ports at startup (production)
    LazyBind,     // Bind on service registration (Replit / dev)
}

/// Parsed Array3 peer address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerAddr {
    pub ip: String,
    pub port: u16,
}

/// Node configuration parsed from environment variables.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub node_id: u8,            // Rep C {1, 2, 3}
    pub is_gateway: bool,       // true only for Node 1
    pub port_range: (u16, u16), // inclusive (start, end)
    pub gateway_port: u16,      // port of the cluster gateway (Node 1's center)
    pub peers: Vec<PeerAddr>,   // CUBE_ARRAY3_PEERS
    pub bind_strategy: BindStrategy,
}

/// Fatal error for invalid node configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeConfigError {
    ZeroNodeId,           // CUBE_NODE_ID=0 is forgery
    NodeIdOutOfRange(u8), // CUBE_NODE_ID > MAX_NODES
    InvalidPeerFormat(String),
}

impl std::fmt::Display for NodeConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodeConfigError::ZeroNodeId =>
                write!(f, "FATAL: CUBE_NODE_ID=0 is structurally impossible in Rep C — zero-sentinel forgery"),
            NodeConfigError::NodeIdOutOfRange(id) =>
                write!(f, "FATAL: CUBE_NODE_ID={} exceeds MAX_NODES={}", id, MAX_NODES),
            NodeConfigError::InvalidPeerFormat(s) =>
                write!(f, "FATAL: invalid peer format in CUBE_ARRAY3_PEERS: '{}'", s),
        }
    }
}

/// Parse a comma-separated peer list like "10.0.0.2:11151,10.0.0.3:11178".
pub fn parse_peers(peer_str: &str) -> Result<Vec<PeerAddr>, NodeConfigError> {
    if peer_str.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut peers = Vec::new();
    for entry in peer_str.split(',') {
        let entry = entry.trim();
        if let Some(colon) = entry.rfind(':') {
            let ip = entry[..colon].to_string();
            let port_str = &entry[colon + 1..];
            let port = port_str.parse::<u16>().map_err(|_|
                NodeConfigError::InvalidPeerFormat(entry.to_string()))?;
            peers.push(PeerAddr { ip, port });
        } else {
            return Err(NodeConfigError::InvalidPeerFormat(entry.to_string()));
        }
    }
    Ok(peers)
}

/// Build node configuration from environment-style inputs.
///
/// `node_id_str`: value of CUBE_NODE_ID (None = default to 1).
/// `peers_str`: value of CUBE_ARRAY3_PEERS (None = no peers, single node).
/// `eager_bind_str`: value of EAGER_BIND ("true"/"1" = eager, else lazy).
pub fn build_node_config(
    node_id_str: Option<&str>,
    peers_str: Option<&str>,
    eager_bind_str: Option<&str>,
) -> Result<NodeConfig, NodeConfigError> {
    let node_id: u8 = match node_id_str {
        Some(s) => s.trim().parse().unwrap_or(0),
        None => GATEWAY_NODE_ID, // default = 1
    };

    // Zero-sentinel check — CUBE_NODE_ID=0 is fatal, not a default
    if node_id == 0 {
        return Err(NodeConfigError::ZeroNodeId);
    }
    if node_id > MAX_NODES as u8 {
        return Err(NodeConfigError::NodeIdOutOfRange(node_id));
    }

    let (port_start, port_end) = node_port_range(node_id).unwrap();
    let gateway_port = BASE_PORT + super::constants::GATEWAY_OFFSET as u16;

    let peers = match peers_str {
        Some(s) => parse_peers(s)?,
        None => Vec::new(),
    };

    let bind_strategy = match eager_bind_str {
        Some(s) if s.trim() == "true" || s.trim() == "1" => BindStrategy::EagerBind,
        _ => BindStrategy::LazyBind,
    };

    Ok(NodeConfig {
        node_id,
        is_gateway: node_id == GATEWAY_NODE_ID,
        port_range: (port_start, port_end),
        gateway_port,
        peers,
        bind_strategy,
    })
}

/// Array3 handshake payload sent during peer formation.
/// V3 wire protocol adds this to the existing CUBE_PEER_PORT handshake.
#[derive(Debug, Clone)]
pub struct Array3Handshake {
    pub node_id: u8,              // Rep C {1, 2, 3}
    pub port_range: (u16, u16),   // inclusive
    pub wire_protocol_version: u8, // 0x03 for V3
    pub daemon_version: String,    // "2.4.0"
    pub is_gateway: bool,
    pub gateway_addr: Option<PeerAddr>, // gateway endpoint for node 2/3 discovery
    pub slot_count: usize,         // number of occupied slots
    pub slot_inventory: Vec<[u8; 3]>, // 3-trit slot addresses currently registered
}

impl NodeConfig {
    pub fn build_handshake(&self, wire_protocol_version: u8, daemon_version: &str, slot_inventory: Vec<[u8; 3]>) -> Array3Handshake {
        let gateway_addr = if self.is_gateway {
            Some(PeerAddr {
                ip: "0.0.0.0".to_string(),
                port: self.gateway_port,
            })
        } else {
            None
        };
        Array3Handshake {
            node_id: self.node_id,
            port_range: self.port_range,
            wire_protocol_version,
            daemon_version: daemon_version.to_string(),
            is_gateway: self.is_gateway,
            gateway_addr,
            slot_count: slot_inventory.len(),
            slot_inventory,
        }
    }
}

/// Validate a received handshake against local state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    ZeroNodeId,
    DuplicateNodeId(u8),
    PortRangeOverlap,
    IncompatibleProtocol { remote: u8, local_min: u8 },
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HandshakeError::ZeroNodeId =>
                write!(f, "REJECT: remote node_id=0 — zero-sentinel forgery"),
            HandshakeError::DuplicateNodeId(id) =>
                write!(f, "REJECT: remote node_id={} collides with local", id),
            HandshakeError::PortRangeOverlap =>
                write!(f, "REJECT: remote port range overlaps local"),
            HandshakeError::IncompatibleProtocol { remote, local_min } =>
                write!(f, "REJECT: remote protocol V{} < local minimum V{}", remote, local_min),
        }
    }
}

/// Validate an incoming handshake.
pub fn validate_handshake(
    local_node_id: u8,
    local_port_range: (u16, u16),
    local_protocol_min: u8,
    remote: &Array3Handshake,
) -> Result<(), HandshakeError> {
    // Zero-sentinel: reject node_id=0 as forgery
    if remote.node_id == 0 {
        return Err(HandshakeError::ZeroNodeId);
    }

    // Reject duplicate node IDs
    if remote.node_id == local_node_id {
        return Err(HandshakeError::DuplicateNodeId(remote.node_id));
    }

    // Reject overlapping port ranges
    if remote.port_range.0 <= local_port_range.1 && remote.port_range.1 >= local_port_range.0 {
        return Err(HandshakeError::PortRangeOverlap);
    }

    // V3 accepts V2 during dual-acceptance, but reject below minimum
    if remote.wire_protocol_version < local_protocol_min {
        return Err(HandshakeError::IncompatibleProtocol {
            remote: remote.wire_protocol_version,
            local_min: local_protocol_min,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_node_is_gateway() {
        let config = build_node_config(None, None, None).unwrap();
        assert_eq!(config.node_id, 1);
        assert!(config.is_gateway);
        assert_eq!(config.port_range, (11111, 11137));
        assert_eq!(config.gateway_port, 11124);
    }

    #[test]
    fn node2_is_not_gateway() {
        let config = build_node_config(Some("2"), None, None).unwrap();
        assert_eq!(config.node_id, 2);
        assert!(!config.is_gateway);
        assert_eq!(config.port_range, (11138, 11164));
    }

    #[test]
    fn node_id_zero_is_fatal() {
        let err = build_node_config(Some("0"), None, None).unwrap_err();
        assert_eq!(err, NodeConfigError::ZeroNodeId);
    }

    #[test]
    fn node_id_four_is_fatal() {
        let err = build_node_config(Some("4"), None, None).unwrap_err();
        assert_eq!(err, NodeConfigError::NodeIdOutOfRange(4));
    }

    #[test]
    fn parse_peers_ok() {
        let peers = parse_peers("10.0.0.2:11151,10.0.0.3:11178").unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].ip, "10.0.0.2");
        assert_eq!(peers[0].port, 11151);
        assert_eq!(peers[1].ip, "10.0.0.3");
        assert_eq!(peers[1].port, 11178);
    }

    #[test]
    fn parse_peers_empty() {
        assert!(parse_peers("").unwrap().is_empty());
    }

    #[test]
    fn eager_bind_flag() {
        let config = build_node_config(Some("1"), None, Some("true")).unwrap();
        assert_eq!(config.bind_strategy, BindStrategy::EagerBind);

        let config = build_node_config(Some("1"), None, Some("false")).unwrap();
        assert_eq!(config.bind_strategy, BindStrategy::LazyBind);
    }

    fn test_handshake(node_id: u8, port_range: (u16, u16), version: u8, is_gateway: bool) -> Array3Handshake {
        Array3Handshake {
            node_id,
            port_range,
            wire_protocol_version: version,
            daemon_version: "2.4.0".to_string(),
            is_gateway,
            gateway_addr: if is_gateway {
                Some(PeerAddr { ip: "10.0.0.1".to_string(), port: 11124 })
            } else {
                None
            },
            slot_count: 0,
            slot_inventory: Vec::new(),
        }
    }

    #[test]
    fn handshake_rejects_zero_node_id() {
        let hs = test_handshake(0, (11138, 11164), 3, false);
        let err = validate_handshake(1, (11111, 11137), 2, &hs).unwrap_err();
        assert_eq!(err, HandshakeError::ZeroNodeId);
    }

    #[test]
    fn handshake_rejects_duplicate_node_id() {
        let hs = test_handshake(1, (11138, 11164), 3, true);
        let err = validate_handshake(1, (11111, 11137), 2, &hs).unwrap_err();
        assert_eq!(err, HandshakeError::DuplicateNodeId(1));
    }

    #[test]
    fn handshake_accepts_v2_during_dual_acceptance() {
        let hs = test_handshake(2, (11138, 11164), 2, false);
        assert!(validate_handshake(1, (11111, 11137), 2, &hs).is_ok());
    }

    #[test]
    fn handshake_rejects_v1_when_min_is_v2() {
        let hs = test_handshake(2, (11138, 11164), 1, false);
        let err = validate_handshake(1, (11111, 11137), 2, &hs).unwrap_err();
        assert!(matches!(err, HandshakeError::IncompatibleProtocol { .. }));
    }

    #[test]
    fn handshake_gateway_carries_address() {
        let hs = test_handshake(1, (11111, 11137), 3, true);
        assert!(hs.gateway_addr.is_some());
        let gw = hs.gateway_addr.unwrap();
        assert_eq!(gw.port, 11124);
    }

    #[test]
    fn handshake_non_gateway_no_address() {
        let hs = test_handshake(2, (11138, 11164), 3, false);
        assert!(hs.gateway_addr.is_none());
    }

    #[test]
    fn handshake_slot_inventory() {
        let mut hs = test_handshake(1, (11111, 11137), 3, true);
        hs.slot_inventory = vec![[1, 2, 3], [2, 1, 1]];
        hs.slot_count = 2;
        assert_eq!(hs.slot_inventory.len(), hs.slot_count);
    }
}
