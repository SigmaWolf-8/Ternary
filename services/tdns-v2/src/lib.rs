// TDNS v2.3 — Ternary Domain Name System
// Capomastro Holdings Ltd. — Applied Physics Division
//
// 27-dimensional ontological addressing for PlenumNET.
// The Address IS the Description. The Description IS the Route.
// The Description IS a Measurement.
//
// Module structure:
//   trit     — Atomic ternary digit {1, 2, 3}
//   addr     — 27-trit CubeAddr with wire encoding
//   subcube  — Wildcard addressing for multicast
//   schema   — The 27 ontological dimensions (WHO→PEACE)
//   scan     — CRS scan results and BLAKE3 scan hash
//   trn      — Ternary Resource Name records
//   routing  — Neighbor maps and greedy forwarding

pub mod trit;
pub mod addr;
pub mod subcube;
pub mod schema;
pub mod scan;
pub mod trn;
pub mod routing;
pub mod derive;
pub mod crs;
pub mod scanner;
pub mod glb;
pub mod fts;
pub mod con;
pub mod api;
pub mod bridge;
pub mod wire;

pub use addr::CubeAddr;
pub use trit::Trit;
pub use subcube::SubCube;
pub use trn::Trn;
pub use scan::{ScanHash, ScanResult};
pub use routing::{NeighborMap, forward, compute_path, ForwardResult};
pub use schema::{Category, Dimension, SCHEMA, describe};
pub use crs::{CrsRegistry, CrsConfig, RegistrationResult, VerificationResult};
pub use scanner::{scan, FullScanResult, ScanTarget, ScanContext, format_scan_report};
pub use glb::{Glb, GlbDecision, HptpPolicy, NodeStatus};
pub use fts::{Fts, FtsConfig, FtsEvent, Heartbeat, HealthState};
pub use con::{ConNode, TunnelKey, TunnelLink, LinkState, derive_tunnel_key, derive_link_keys};
pub use api::ApiRouter;
pub use bridge::{Bridge, Resolution, is_plm_name};
pub use wire::{Packet, PacketType, WireError};
