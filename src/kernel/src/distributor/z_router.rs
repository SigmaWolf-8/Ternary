// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Z-Router: Routes requests through the z-level dome architecture.
// Above ground (+z) = presentation. Below ground (−z) = processing.
// z=0 = distributor plane (never blocks, routes only).

use alloc::string::String;
use core::fmt;

  /// Tiny no_std u8-to-String helper used by ZLevel::label().
  fn itoa_u8(mut n: u8) -> String {
      if n == 0 { return String::from("0"); }
      let mut buf = [0u8; 3];
      let mut i = buf.len();
      while n > 0 {
          i -= 1;
          buf[i] = b'0' + (n % 10);
          n /= 10;
      }
      let mut out = String::new();
      for &b in &buf[i..] { out.push(b as char); }
      out
  }
  
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZLevel(i8);

impl ZLevel {
    pub const UI: Self = Self(1);
    pub const FILE_SERVER: Self = Self(2);
    pub const SNAPSHOT_BASE: Self = Self(3);
    pub const DISTRIBUTOR: Self = Self(0);
    pub const API_GATEWAY: Self = Self(-1);
    pub const APP_SERVICES: Self = Self(-2);
    pub const CONVENTIONAL: Self = Self(-3);
    pub const TERNARY_NATIVE: Self = Self(-4);
    pub const DATA_LAYER: Self = Self(-5);
    pub const INFRASTRUCTURE: Self = Self(-6);

    pub const fn new(level: i8) -> Self {
        Self(level)
    }

    pub const fn value(&self) -> i8 {
        self.0
    }

    pub fn is_above_ground(&self) -> bool {
        self.0 > 0
    }

    pub fn is_below_ground(&self) -> bool {
        self.0 < 0
    }

    pub fn is_distributor(&self) -> bool {
        self.0 == 0
    }

    pub fn is_presentation(&self) -> bool {
        self.0 > 0
    }

    pub fn is_processing(&self) -> bool {
        self.0 < 0
    }

    /// Owned operator-readable label for this Z-level. Returned as a
      /// `String` so callers can concatenate diagnostic context without
      /// reaching for a separate formatter.
      pub fn label(&self) -> String {
          let name = match self.0 {
              1 => "UI",
              2 => "FILE_SERVER",
              3 => "SNAPSHOT_BASE",
              0 => "DISTRIBUTOR",
              -1 => "API_GATEWAY",
              -2 => "APP_SERVICES",
              -3 => "CONVENTIONAL",
              -4 => "TERNARY_NATIVE",
              -5 => "DATA_LAYER",
              -6 => "INFRASTRUCTURE",
              _ => "CUSTOM",
          };
          let mut out = String::new();
          out.push_str("z=");
          if self.0 < 0 {
              out.push('-');
              out.push_str(&itoa_u8((-self.0) as u8));
          } else {
              out.push_str(&itoa_u8(self.0 as u8));
          }
          out.push(' ');
          out.push_str(name);
          out
      }

      pub fn hops_from_zero(&self) -> u8 {
        self.0.unsigned_abs()
    }
}

impl fmt::Display for ZLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.0 {
            1 => "UI",
            2 => "FileServer",
            3 => "Snapshot",
            0 => "Distributor",
            -1 => "APIGateway",
            -2 => "AppServices",
            -3 => "Conventional",
            -4 => "TernaryNative",
            -5 => "DataLayer",
            -6 => "Infrastructure",
            n if n > 3 => "Archive",
            _ => "Unknown",
        };
        write!(f, "z={} ({})", self.0, name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteDirection {
    Falling,
    Bubbling,
}

#[derive(Debug, Clone)]
pub struct ZRequest {
    pub id: u64,
    pub origin: ZLevel,
    pub target: ZLevel,
    pub ring_position: u32,
    pub direction: RouteDirection,
    pub payload_type: RequestType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    HttpRequest,
    DnsLookup,
    DataQuery,
    CryptoOp,
    FileServe,
    ScriptExec,
    RenderCommand,
    Snapshot,
}

impl RequestType {
    pub fn target_z(&self) -> ZLevel {
        match self {
            RequestType::HttpRequest => ZLevel::API_GATEWAY,
            RequestType::DnsLookup => ZLevel::INFRASTRUCTURE,
            RequestType::DataQuery => ZLevel::DATA_LAYER,
            RequestType::CryptoOp => ZLevel::INFRASTRUCTURE,
            RequestType::FileServe => ZLevel::FILE_SERVER,
            RequestType::ScriptExec => ZLevel::APP_SERVICES,
            RequestType::RenderCommand => ZLevel::UI,
            RequestType::Snapshot => ZLevel::SNAPSHOT_BASE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZResponse {
    pub request_id: u64,
    pub origin: ZLevel,
    pub target: ZLevel,
    pub direction: RouteDirection,
    pub status: RouteStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteStatus {
    Ok,
    LayerUnavailable,
    Timeout,
    Error,
}

pub struct ZRouter {
    request_counter: u64,
}

impl ZRouter {
    pub fn new() -> Self {
        Self { request_counter: 0 }
    }

    pub fn route(&mut self, origin: ZLevel, payload: RequestType, ring_pos: u32) -> ZRequest {
        self.request_counter += 1;
        let target = payload.target_z();
        let direction = if target.value() < origin.value() {
            RouteDirection::Falling
        } else {
            RouteDirection::Bubbling
        };

        ZRequest {
            id: self.request_counter,
            origin,
            target,
            ring_position: ring_pos,
            direction,
            payload_type: payload,
        }
    }

    pub fn compute_hop_path(from: ZLevel, to: ZLevel) -> (RouteDirection, u8) {
        let diff = to.value() as i16 - from.value() as i16;
        if diff < 0 {
            (RouteDirection::Falling, (-diff) as u8)
        } else {
            (RouteDirection::Bubbling, diff as u8)
        }
    }

    pub fn estimate_latency_ns(from: ZLevel, to: ZLevel) -> u64 {
        let (_, hops) = Self::compute_hop_path(from, to);
        hops as u64 * 10
    }

    pub fn requests_processed(&self) -> u64 {
        self.request_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_levels() {
        assert!(ZLevel::UI.is_above_ground());
        assert!(ZLevel::INFRASTRUCTURE.is_below_ground());
        assert!(ZLevel::DISTRIBUTOR.is_distributor());
        assert_eq!(ZLevel::UI.value(), 1);
        assert_eq!(ZLevel::INFRASTRUCTURE.value(), -6);
    }

    #[test]
    fn test_request_routing() {
        let mut router = ZRouter::new();
        let req = router.route(ZLevel::UI, RequestType::DataQuery, 42);
        assert_eq!(req.target, ZLevel::DATA_LAYER);
        assert_eq!(req.direction, RouteDirection::Falling);
        assert_eq!(req.ring_position, 42);
    }

    #[test]
    fn test_hop_path() {
        let (dir, hops) = ZRouter::compute_hop_path(ZLevel::UI, ZLevel::DATA_LAYER);
        assert_eq!(dir, RouteDirection::Falling);
        assert_eq!(hops, 6);
    }

    #[test]
    fn test_bubble_path() {
        let (dir, hops) = ZRouter::compute_hop_path(ZLevel::DATA_LAYER, ZLevel::UI);
        assert_eq!(dir, RouteDirection::Bubbling);
        assert_eq!(hops, 6);
    }

    #[test]
    fn test_latency_estimate() {
        let ns = ZRouter::estimate_latency_ns(ZLevel::UI, ZLevel::INFRASTRUCTURE);
        assert_eq!(ns, 70);
    }

    #[test]
    fn test_request_counter() {
        let mut router = ZRouter::new();
        router.route(ZLevel::UI, RequestType::HttpRequest, 0);
        router.route(ZLevel::UI, RequestType::DataQuery, 1);
        assert_eq!(router.requests_processed(), 2);
    }

    #[test]
    fn test_request_type_targets() {
        assert_eq!(RequestType::HttpRequest.target_z(), ZLevel::API_GATEWAY);
        assert_eq!(RequestType::CryptoOp.target_z(), ZLevel::INFRASTRUCTURE);
        assert_eq!(RequestType::FileServe.target_z(), ZLevel::FILE_SERVER);
    }
}
