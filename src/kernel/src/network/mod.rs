use alloc::string::String;
use core::fmt;

pub mod torus;
pub mod routing;
pub mod ttp;
pub mod t3p;
pub mod tdns;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    InvalidDimension,
    InvalidCoordinate,
    NodeNotFound,
    NoRoute,
    RoutingLoopDetected,
    MaxHopsExceeded,
    TorsionCoefficientError,
    TopologyError(String),
    InvalidState(String),
    ConnectionClosed,
    SessionInactive,
    InvalidPort,
    BufferFull,
    ProtocolError(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::InvalidDimension => write!(f, "Invalid dimension"),
            NetworkError::InvalidCoordinate => write!(f, "Invalid coordinate"),
            NetworkError::NodeNotFound => write!(f, "Node not found"),
            NetworkError::NoRoute => write!(f, "No route found"),
            NetworkError::RoutingLoopDetected => write!(f, "Routing loop detected"),
            NetworkError::MaxHopsExceeded => write!(f, "Maximum hops exceeded"),
            NetworkError::TorsionCoefficientError => write!(f, "Torsion coefficient error"),
            NetworkError::TopologyError(msg) => write!(f, "Topology error: {}", msg),
            NetworkError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            NetworkError::ConnectionClosed => write!(f, "Connection closed"),
            NetworkError::SessionInactive => write!(f, "Session inactive"),
            NetworkError::InvalidPort => write!(f, "Invalid port"),
            NetworkError::BufferFull => write!(f, "Buffer full"),
            NetworkError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

pub type NetworkResult<T> = core::result::Result<T, NetworkError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_creation() {
        let id = NodeId(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_node_id_equality() {
        let a = NodeId(1);
        let b = NodeId(1);
        let c = NodeId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_node_id_ordering() {
        let a = NodeId(1);
        let b = NodeId(2);
        assert!(a < b);
    }

    #[test]
    fn test_network_error_display() {
        let err = NetworkError::NodeNotFound;
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("Node not found"));
    }

    #[test]
    fn test_network_error_topology() {
        let err = NetworkError::TopologyError(String::from("test error"));
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("test error"));
    }
}
