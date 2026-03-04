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
//   derive   — The 27 Derivation Rules
//   crs      — CRS Registry Service
//   scanner  — Live network scanning (URL → 27 observations → CubeAddr)

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

pub use addr::CubeAddr;
pub use trit::Trit;
pub use subcube::SubCube;
pub use trn::Trn;
pub use scan::{ScanHash, ScanResult};
pub use routing::{NeighborMap, forward, compute_path, ForwardResult};
pub use schema::{Category, Dimension, SCHEMA, describe};
pub use crs::{CrsRegistry, CrsConfig, RegistrationResult, VerificationResult};
pub use scanner::{scan, FullScanResult, ScanTarget, ScanContext, format_scan_report};
