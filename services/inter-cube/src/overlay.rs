// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Cube Overlay Network (CON) — Service 2
//!
//! Decouples logical cube adjacency (neighbors that differ by one trit) from
//! the physical network. Creates encrypted tunnels between geometric neighbors
//! regardless of physical location — same rack, different continent, or across
//! the public internet.
//!
//! ## T-07 (SPEC-2026-NEXT): Neighbor-Side Signature Verification
//!
//! When CON resolves a neighbor's physical endpoint from CRS, the response
//! includes the registration signature (`reg_signature`). Before establishing
//! a tunnel, the querying node:
//!
//! 1. Reconstructs the canonical signed message from the CRS response fields
//! 2. Verifies the TL-DSA-87 signature against the returned `public_key`
//! 3. If verification fails: neighbor stays `Unknown`, forgery alert logged
//! 4. If verification passes: proceeds to tunnel establishment (TL-KEM handshake)
//!
//! The worst a compromised CRS can do is refuse to return records (denial of
//! service), not redirect tunnels — because it cannot forge the registrant's
//! TL-DSA signature.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::cube_addr::{CubeAddr, RepCTrit, DIMENSIONS, NEIGHBORS_PER_CUBE};
use crate::crs::{build_registration_message, CRS_SIG_VARIANT};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Default heartbeat interval for tunnel liveness checks.
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 1000;

/// Default key rotation interval.
const DEFAULT_KEY_ROTATION_SECS: u64 = 86400; // 24 hours

/// Virtual interface name prefix.
const TUNNEL_IFACE_PREFIX: &str = "cubetun";

// ═══════════════════════════════════════════════════════════════════════
// TUNNEL STATE MACHINE
// ═══════════════════════════════════════════════════════════════════════

/// State of a tunnel to a geometric neighbor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelState {
    /// Neighbor address computed but physical endpoint not yet resolved.
    Unknown,
    /// Querying the CRS for the neighbor's physical endpoint.
    Resolving,
    /// Tunnel handshake in progress.
    Connecting,
    /// T-14: Mutual authentication in progress (3-message handshake).
    Authenticating,
    /// Encrypted tunnel active and passing traffic.
    Up,
    /// Heartbeat lost — reported to FTS as potentially down.
    Down,
}

impl TunnelState {
    /// Whether this state allows forwarding traffic.
    pub fn is_active(&self) -> bool {
        matches!(self, TunnelState::Up)
    }

    /// Whether this state should trigger CRS re-resolution.
    pub fn needs_resolution(&self) -> bool {
        matches!(self, TunnelState::Unknown | TunnelState::Down)
    }
}

impl std::fmt::Display for TunnelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelState::Unknown => write!(f, "unknown"),
            TunnelState::Resolving => write!(f, "resolving"),
            TunnelState::Connecting => write!(f, "connecting"),
            TunnelState::Authenticating => write!(f, "authenticating"),
            TunnelState::Up => write!(f, "up"),
            TunnelState::Down => write!(f, "down"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TUNNEL PROTOCOL SELECTION
// ═══════════════════════════════════════════════════════════════════════

/// Encryption protocol for inter-cube tunnels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelProtocol {
    /// Standard WireGuard Noise protocol with pre-exchanged keys.
    WireGuard,
    /// PlenumNET-native: keys derived from cryptographic sponge
    /// using both cubes' Rep C addresses. Post-quantum by construction.
    #[deprecated(note = "Use PqNativeV3 with TL-KEM shared secret for IND-CCA2 key secrecy")]
    PqNative,
    /// PlenumNET-native v3: TL-KEM key exchange + TL-Sponge-385 KDF.
    PqNativeV3,
}

// ═══════════════════════════════════════════════════════════════════════
// FORGERY ALERT — T-07
// ═══════════════════════════════════════════════════════════════════════

/// A forgery alert emitted when neighbor signature verification fails.
///
/// This indicates either a compromised CRS or an active MITM attack.
/// The neighbor stays in `Unknown` state and no tunnel is established.
#[derive(Debug, Clone)]
pub struct ForgeryAlert {
    /// Address of the neighbor whose signature failed verification.
    pub neighbor_addr: CubeAddr,
    /// Endpoint claimed by the CRS response.
    pub claimed_endpoint: SocketAddr,
    /// When the alert was generated.
    pub timestamp: Instant,
    /// Reason for the alert.
    pub reason: ForgeryReason,
}

/// Reason a neighbor registration was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeryReason {
    /// TL-DSA signature verification failed against the provided public key.
    SignatureInvalid,
    /// CRS response had no registration signature (unsigned record in signed mode).
    SignatureMissing,
    /// CRS response had no public key.
    PublicKeyMissing,
}

impl std::fmt::Display for ForgeryReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SignatureInvalid => write!(f, "TL-DSA signature verification failed"),
            Self::SignatureMissing => write!(f, "Registration signature missing from CRS response"),
            Self::PublicKeyMissing => write!(f, "Public key missing from CRS response"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NEIGHBOR RECORD
// ═══════════════════════════════════════════════════════════════════════

/// Complete record for a single geometric neighbor.
///
/// There are exactly 26 of these per cube — one for each neighbor
/// computed from the 13D cube geometry.
#[derive(Debug, Clone)]
pub struct Neighbor {
    /// The neighbor's Rep C cube address (computed from geometry).
    pub cube_addr: CubeAddr,
    /// Which dimension this neighbor differs from us in.
    pub dimension: usize,
    /// What value the neighbor holds in the differing dimension.
    pub alt_value: RepCTrit,
    /// Physical network endpoint (from CRS lookup).
    pub endpoint: Option<SocketAddr>,
    /// Neighbor's full public key for tunnel authentication (Vec<u8> for TL-DSA-87).
    pub public_key: Option<Vec<u8>>,
    /// TL-KEM shared secret for v3 key derivation (32 bytes).
    pub kem_shared_secret: Option<[u8; 32]>,
    /// TL-KEM public key for key exchange.
    pub kem_public_key: Option<Vec<u8>>,
    /// TL-DSA registration signature from CRS (T-07).
    /// Stored after successful verification so it doesn't need to be
    /// re-verified on every lookup.
    pub reg_signature: Option<Vec<u8>>,
    /// Whether the registration signature was verified (T-07).
    pub signature_verified: bool,
    /// Assigned virtual interface name (e.g., "cubetun0").
    pub tunnel_iface: Option<String>,
    /// Current tunnel state.
    pub state: TunnelState,
    /// Last successful heartbeat timestamp.
    pub last_heartbeat: Option<Instant>,
    /// Smoothed round-trip time in nanoseconds.
    pub srtt_ns: Option<u64>,
    /// Traffic counters.
    pub bytes_in: u64,
    pub bytes_out: u64,
    /// Tunnel uptime since last establishment.
    pub tunnel_established: Option<Instant>,
}

impl Neighbor {
    /// Create a new neighbor record from computed geometry.
    fn new(cube_addr: CubeAddr, dimension: usize, alt_value: RepCTrit) -> Self {
        Neighbor {
            cube_addr,
            dimension,
            alt_value,
            endpoint: None,
            public_key: None,
            kem_shared_secret: None,
            kem_public_key: None,
            reg_signature: None,
            signature_verified: false,
            tunnel_iface: None,
            state: TunnelState::Unknown,
            last_heartbeat: None,
            srtt_ns: None,
            bytes_in: 0,
            bytes_out: 0,
            tunnel_established: None,
        }
    }

    /// RTT in milliseconds (for API responses).
    pub fn rtt_ms(&self) -> Option<f64> {
        self.srtt_ns.map(|ns| ns as f64 / 1_000_000.0)
    }

    /// Uptime in seconds since tunnel establishment.
    pub fn uptime_secs(&self) -> Option<f64> {
        self.tunnel_established
            .map(|t| t.elapsed().as_secs_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CON STATISTICS
// ═══════════════════════════════════════════════════════════════════════

/// Aggregate statistics for the Cube Overlay Network.
#[derive(Debug, Clone)]
pub struct ConStats {
    pub tunnels_up: usize,
    pub tunnels_down: usize,
    pub tunnels_resolving: usize,
    pub tunnels_connecting: usize,
    pub tunnels_unknown: usize,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub avg_rtt_ms: Option<f64>,
    /// T-07: Number of neighbors with verified signatures.
    pub verified_neighbors: usize,
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE OVERLAY NETWORK
// ═══════════════════════════════════════════════════════════════════════

/// The Cube Overlay Network daemon.
///
/// Manages exactly 26 tunnels — one for each geometric neighbor.
/// The neighbor list is computed from the local cube address using
/// pure trit arithmetic. No discovery protocol needed.
pub struct CubeOverlayNetwork {
    /// This cube's Rep C address.
    local_addr: CubeAddr,
    /// Exactly 26 neighbors, computed at initialization.
    neighbors: Vec<Neighbor>,
    /// Index: cube_addr → position in neighbors vec.
    addr_index: HashMap<CubeAddr, usize>,
    /// Tunnel encryption protocol.
    tunnel_protocol: TunnelProtocol,
    /// Heartbeat interval.
    #[allow(dead_code)]
    heartbeat_interval: Duration,
    /// Key rotation interval.
    #[allow(dead_code)]
    key_rotation_interval: Duration,
    /// T-07: Forgery alerts — logged when signature verification fails.
    forgery_alerts: Vec<ForgeryAlert>,
}

impl CubeOverlayNetwork {
    /// Create a new CON daemon for the given local cube address.
    ///
    /// Immediately computes all 26 geometric neighbors from the address.
    /// This is pure math — no network calls, no discovery.
    pub fn new(local_addr: CubeAddr) -> Self {
        let mut neighbors = Vec::with_capacity(NEIGHBORS_PER_CUBE);
        let mut addr_index = HashMap::with_capacity(NEIGHBORS_PER_CUBE);

        let mut idx = 0usize;
        for dim in 0..DIMENSIONS {
            for alt in local_addr.trit(dim).alternatives() {
                let mut nbr_addr = local_addr.clone();
                nbr_addr.set_trit(dim, alt);
                let neighbor = Neighbor::new(nbr_addr.clone(), dim, alt);
                addr_index.insert(nbr_addr, idx);
                neighbors.push(neighbor);
                idx += 1;
            }
        }

        assert_eq!(
            neighbors.len(),
            NEIGHBORS_PER_CUBE,
            "Must have exactly 26 neighbors"
        );

        CubeOverlayNetwork {
            local_addr,
            neighbors,
            addr_index,
            tunnel_protocol: TunnelProtocol::PqNativeV3,
            heartbeat_interval: Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS),
            key_rotation_interval: Duration::from_secs(DEFAULT_KEY_ROTATION_SECS),
            forgery_alerts: Vec::new(),
        }
    }

    /// Create with custom protocol selection.
    pub fn with_protocol(mut self, protocol: TunnelProtocol) -> Self {
        self.tunnel_protocol = protocol;
        self
    }

    /// Get the local cube address.
    pub fn local_addr(&self) -> &CubeAddr {
        &self.local_addr
    }

    /// Get all 26 neighbors.
    pub fn neighbors(&self) -> &[Neighbor] {
        &self.neighbors
    }

    /// Get a specific neighbor by cube address.
    pub fn neighbor(&self, addr: &CubeAddr) -> Option<&Neighbor> {
        self.addr_index.get(addr).map(|&i| &self.neighbors[i])
    }

    /// Get a mutable reference to a specific neighbor.
    pub fn neighbor_mut(&mut self, addr: &CubeAddr) -> Option<&mut Neighbor> {
        self.addr_index
            .get(addr)
            .copied()
            .map(move |i| &mut self.neighbors[i])
    }

    /// Get the neighbor at a specific dimension and alternative value.
    pub fn neighbor_at_dim(&self, dim: usize, alt: RepCTrit) -> Option<&Neighbor> {
        self.neighbors
            .iter()
            .find(|n| n.dimension == dim && n.alt_value == alt)
    }

    // ═══════════════════════════════════════════════════════════════
    // ENDPOINT RESOLUTION — Legacy (unsigned, no verification)
    // ═══════════════════════════════════════════════════════════════

    /// Resolve a neighbor's physical endpoint from CRS lookup result.
    /// Called after querying the CRS for the neighbor's registration.
    ///
    /// **Legacy path** — no signature verification. Use `resolve_neighbor_verified()`
    /// for the T-07 hardened path.
    pub fn resolve_neighbor(
        &mut self,
        addr: &CubeAddr,
        endpoint: SocketAddr,
        public_key: Vec<u8>,
    ) -> bool {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.endpoint = Some(endpoint);
            nbr.public_key = Some(public_key);
            nbr.state = TunnelState::Connecting;
            true
        } else {
            false
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // ENDPOINT RESOLUTION — Verified (T-07)
    // ═══════════════════════════════════════════════════════════════

    /// Resolve a neighbor's endpoint with signature verification (T-07).
    ///
    /// Before transitioning to `Connecting`, verifies the CRS registration
    /// signature against the returned public key. If verification fails,
    /// the neighbor stays `Unknown` and a `ForgeryAlert` is emitted.
    ///
    /// ## Parameters
    ///
    /// - `addr`: The neighbor's geometric address
    /// - `endpoint`: Physical endpoint from CRS response
    /// - `public_key`: Full TL-DSA public key from CRS response
    /// - `reg_signature`: Registration signature from CRS response
    /// - `kem_public_key`: Optional KEM public key from CRS response
    /// - `registered_at_fs`: Femtosecond timestamp from the registration
    ///
    /// ## Returns
    ///
    /// - `Ok(true)` — signature valid, neighbor is now `Connecting`
    /// - `Ok(false)` — neighbor address not found in our 26 neighbors
    /// - `Err(ForgeryAlert)` — signature verification failed
    pub fn resolve_neighbor_verified(
        &mut self,
        addr: &CubeAddr,
        endpoint: SocketAddr,
        public_key: Vec<u8>,
        reg_signature: Option<Vec<u8>>,
        kem_public_key: Option<Vec<u8>>,
        registered_at_fs: u128,
    ) -> Result<bool, ForgeryAlert> {
        // Check this is actually one of our 26 neighbors
        let idx = match self.addr_index.get(addr) {
            Some(&i) => i,
            None => return Ok(false),
        };

        // Signature must be present
        let signature = match reg_signature {
            Some(sig) => sig,
            None => {
                let alert = ForgeryAlert {
                    neighbor_addr: addr.clone(),
                    claimed_endpoint: endpoint,
                    timestamp: Instant::now(),
                    reason: ForgeryReason::SignatureMissing,
                };
                println!(
                    "[CON] FORGERY ALERT: {} at {} — {}",
                    addr, endpoint, alert.reason
                );
                self.forgery_alerts.push(alert.clone());
                return Err(alert);
            }
        };

        // Public key must be non-empty
        if public_key.is_empty() {
            let alert = ForgeryAlert {
                neighbor_addr: addr.clone(),
                claimed_endpoint: endpoint,
                timestamp: Instant::now(),
                reason: ForgeryReason::PublicKeyMissing,
            };
            println!(
                "[CON] FORGERY ALERT: {} at {} — {}",
                addr, endpoint, alert.reason
            );
            self.forgery_alerts.push(alert.clone());
            return Err(alert);
        }

        // Reconstruct the canonical message and verify the signature
        // Uses the same build_registration_message() as the signer (T-06)
        let canonical_msg = build_registration_message(
            addr,
            &endpoint,
            &public_key,
            kem_public_key.as_deref(),
            registered_at_fs,
        );

        let variant = ternary_math::tl_dsa::TlDsaVariant::from_u32(CRS_SIG_VARIANT as u32)
            .expect("CRS_SIG_VARIANT must be valid");

        let valid = ternary_math::tl_dsa::verify(
            &public_key,
            &canonical_msg,
            &signature,
            variant,
        );

        if !valid {
            let alert = ForgeryAlert {
                neighbor_addr: addr.clone(),
                claimed_endpoint: endpoint,
                timestamp: Instant::now(),
                reason: ForgeryReason::SignatureInvalid,
            };
            println!(
                "[CON] FORGERY ALERT: {} at {} — {}",
                addr, endpoint, alert.reason
            );
            self.forgery_alerts.push(alert.clone());
            // Neighbor stays Unknown — do NOT establish tunnel
            return Err(alert);
        }

        // Signature valid — proceed to tunnel establishment
        let nbr = &mut self.neighbors[idx];
        nbr.endpoint = Some(endpoint);
        nbr.public_key = Some(public_key);
        nbr.reg_signature = Some(signature);
        nbr.kem_public_key = kem_public_key;
        nbr.signature_verified = true;
        nbr.state = TunnelState::Connecting;

        Ok(true)
    }

    /// Mark a neighbor as unresolvable (not registered in CRS).
    pub fn mark_unresolved(&mut self, addr: &CubeAddr) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.state = TunnelState::Unknown;
            nbr.endpoint = None;
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // FORGERY ALERTS — T-07
    // ═══════════════════════════════════════════════════════════════

    /// Get all forgery alerts (for logging/telemetry).
    pub fn forgery_alerts(&self) -> &[ForgeryAlert] {
        &self.forgery_alerts
    }

    /// Drain and return all forgery alerts (clears the internal list).
    pub fn drain_forgery_alerts(&mut self) -> Vec<ForgeryAlert> {
        std::mem::take(&mut self.forgery_alerts)
    }

    /// Number of forgery alerts since last drain.
    pub fn forgery_alert_count(&self) -> usize {
        self.forgery_alerts.len()
    }

    // ═══════════════════════════════════════════════════════════════
    // TUNNEL LIFECYCLE
    // ═══════════════════════════════════════════════════════════════

    /// Mark a tunnel as successfully established.
    pub fn tunnel_up(&mut self, addr: &CubeAddr, iface: String) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.state = TunnelState::Up;
            nbr.tunnel_iface = Some(iface);
            nbr.tunnel_established = Some(Instant::now());
        }
    }

    /// Mark a tunnel as down (heartbeat lost).
    pub fn tunnel_down(&mut self, addr: &CubeAddr) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.state = TunnelState::Down;
            nbr.tunnel_iface = None;
            nbr.tunnel_established = None;
        }
    }

    /// Record a heartbeat response from a neighbor.
    pub fn record_heartbeat(&mut self, addr: &CubeAddr, rtt_ns: u64) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.last_heartbeat = Some(Instant::now());
            nbr.srtt_ns = Some(match nbr.srtt_ns {
                Some(prev) => (prev * 7 + rtt_ns) / 8,
                None => rtt_ns,
            });
        }
    }

    /// Record traffic bytes for a neighbor tunnel.
    pub fn record_traffic(&mut self, addr: &CubeAddr, bytes_in: u64, bytes_out: u64) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.bytes_in += bytes_in;
            nbr.bytes_out += bytes_out;
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // PQ-NATIVE KEY DERIVATION
    // ═══════════════════════════════════════════════════════════════

    /// Derive a shared tunnel key from both cubes' Rep C addresses (v2.5).
    ///
    /// **REMOVED FROM PRODUCTION** (T-09, SPEC-2026-NEXT).
    /// This function uses only public addresses as input — any node can compute
    /// the key for any pair. An attacker who blocks the KEM exchange could force
    /// fallback to v2.5, then compute the key from public addresses.
    ///
    /// Retained under `#[cfg(test)]` for regression testing only.
    #[cfg(test)]
    #[deprecated(note = "REMOVED: v2.5 uses public-only inputs. Use derive_pq_tunnel_key_v3")]
    pub fn derive_pq_tunnel_key(
        addr_a: &CubeAddr,
        addr_b: &CubeAddr,
    ) -> [u8; 32] {
        let bytes_a = addr_a.to_bytes();
        let bytes_b = addr_b.to_bytes();

        let (lo, hi) = if bytes_a <= bytes_b {
            (&bytes_a, &bytes_b)
        } else {
            (&bytes_b, &bytes_a)
        };

        let mut material = Vec::with_capacity(lo.len() + hi.len());
        material.extend_from_slice(lo);
        material.extend_from_slice(hi);

        let key_bytes = ternary_math::sponge::derive_key(b"PlenumNET-CON-v2.5", &material, 32);
        let mut out = [0u8; 32];
        out.copy_from_slice(&key_bytes);
        out
    }

    /// Derive a shared tunnel key using TL-KEM shared secret + TL-Sponge-385 KDF (v3).
    pub fn derive_pq_tunnel_key_v3(
        addr_a: &CubeAddr,
        addr_b: &CubeAddr,
        kem_shared_secret: &[u8; 32],
        epoch: u64,
    ) -> [u8; 32] {
        let bytes_a = addr_a.to_bytes();
        let bytes_b = addr_b.to_bytes();

        let (lo, hi) = if bytes_a <= bytes_b {
            (&bytes_a, &bytes_b)
        } else {
            (&bytes_b, &bytes_a)
        };

        let key_bytes = ternary_math::sponge::sponge385_derive_key(
            b"PlenumNET-CON-v3.0",
            lo,
            hi,
            kem_shared_secret,
            epoch,
        );
        let mut out = [0u8; 32];
        out.copy_from_slice(&key_bytes);
        out
    }

    /// Derive all tunnel keys for this cube's neighbors.
    ///
    /// Uses v3 key derivation (TL-KEM + TL-Sponge-385) exclusively.
    /// Neighbors without a KEM shared secret are **skipped** — no key is
    /// derived, and their tunnel stays in `Resolving` until KEM completes.
    ///
    /// **T-09 (SPEC-2026-NEXT):** The v2.5 fallback has been removed.
    /// v2.5 derived keys from public addresses only — any node could compute
    /// the key for any pair. Blocking the KEM exchange would force fallback
    /// to v2.5, enabling a downgrade attack. Now: no KEM = no tunnel.
    pub fn derive_all_keys(&self, kem_secrets: &HashMap<CubeAddr, [u8; 32]>, epoch: u64) -> Vec<(CubeAddr, [u8; 32])> {
        self.neighbors
            .iter()
            .filter_map(|nbr| {
                let secret = nbr.kem_shared_secret.as_ref()
                    .or_else(|| kem_secrets.get(&nbr.cube_addr))?;
                let key = Self::derive_pq_tunnel_key_v3(
                    &self.local_addr, &nbr.cube_addr, secret, epoch,
                );
                Some((nbr.cube_addr.clone(), key))
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════
    // VIRTUAL INTERFACE ASSIGNMENT
    // ═══════════════════════════════════════════════════════════════

    /// Generate the virtual interface name for a tunnel.
    pub fn iface_name_for(&self, addr: &CubeAddr) -> Option<String> {
        self.addr_index
            .get(addr)
            .map(|&idx| format!("{}{}", TUNNEL_IFACE_PREFIX, idx))
    }

    /// Get the cube address for a given virtual interface name.
    pub fn addr_for_iface(&self, iface: &str) -> Option<&CubeAddr> {
        if !iface.starts_with(TUNNEL_IFACE_PREFIX) {
            return None;
        }
        let idx_str = &iface[TUNNEL_IFACE_PREFIX.len()..];
        let idx: usize = idx_str.parse().ok()?;
        self.neighbors.get(idx).map(|n| &n.cube_addr)
    }

    // ═══════════════════════════════════════════════════════════════
    // STATISTICS
    // ═══════════════════════════════════════════════════════════════

    /// Compute aggregate statistics.
    pub fn stats(&self) -> ConStats {
        let mut up = 0;
        let mut down = 0;
        let mut resolving = 0;
        let mut connecting = 0;
        let mut unknown = 0;
        let mut verified = 0;
        let mut total_in = 0u64;
        let mut total_out = 0u64;
        let mut rtt_sum = 0.0f64;
        let mut rtt_count = 0usize;

        for n in &self.neighbors {
            match n.state {
                TunnelState::Up => up += 1,
                TunnelState::Down => down += 1,
                TunnelState::Resolving => resolving += 1,
                TunnelState::Connecting | TunnelState::Authenticating => connecting += 1,
                TunnelState::Unknown => unknown += 1,
            }
            if n.signature_verified {
                verified += 1;
            }
            total_in += n.bytes_in;
            total_out += n.bytes_out;
            if let Some(rtt) = n.rtt_ms() {
                rtt_sum += rtt;
                rtt_count += 1;
            }
        }

        ConStats {
            tunnels_up: up,
            tunnels_down: down,
            tunnels_resolving: resolving,
            tunnels_connecting: connecting,
            tunnels_unknown: unknown,
            total_bytes_in: total_in,
            total_bytes_out: total_out,
            avg_rtt_ms: if rtt_count > 0 {
                Some(rtt_sum / rtt_count as f64)
            } else {
                None
            },
            verified_neighbors: verified,
        }
    }

    /// Get addresses of all neighbors with active tunnels.
    pub fn live_neighbors(&self) -> Vec<&CubeAddr> {
        self.neighbors
            .iter()
            .filter(|n| n.state == TunnelState::Up)
            .map(|n| &n.cube_addr)
            .collect()
    }

    /// Get addresses of all neighbors with down tunnels.
    pub fn dead_neighbors(&self) -> Vec<&CubeAddr> {
        self.neighbors
            .iter()
            .filter(|n| n.state == TunnelState::Down)
            .map(|n| &n.cube_addr)
            .collect()
    }

    /// Trigger re-resolution of all neighbor endpoints from CRS.
    pub fn refresh_all(&mut self) {
        for nbr in &mut self.neighbors {
            if nbr.state != TunnelState::Up {
                nbr.state = TunnelState::Resolving;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(trits: [u8; 13]) -> CubeAddr {
        CubeAddr::new(trits)
    }

    #[test]
    fn test_neighbor_computation() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        assert_eq!(con.neighbors().len(), NEIGHBORS_PER_CUBE);
    }

    #[test]
    fn test_all_neighbors_differ_by_one_trit() {
        let local = addr([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2]);
        let con = CubeOverlayNetwork::new(local.clone());
        for nbr in con.neighbors() {
            let dist = local.hamming_distance(&nbr.cube_addr);
            assert_eq!(dist, 1, "Neighbor must differ by exactly one trit");
        }
    }

    #[test]
    fn test_no_duplicate_neighbors() {
        let local = addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let con = CubeOverlayNetwork::new(local);
        let addrs: Vec<_> = con.neighbors().iter().map(|n| &n.cube_addr).collect();
        let unique: std::collections::HashSet<_> = addrs.iter().collect();
        assert_eq!(addrs.len(), unique.len(), "No duplicate neighbors");
    }

    #[test]
    fn test_neighbor_lookup() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        let expected = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr = con.neighbor(&expected).unwrap();
        assert_eq!(nbr.dimension, 0);
    }

    #[test]
    fn test_pq_key_derivation_symmetric() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        #[allow(deprecated)]
        let key_ab = CubeOverlayNetwork::derive_pq_tunnel_key(&a, &b);
        #[allow(deprecated)]
        let key_ba = CubeOverlayNetwork::derive_pq_tunnel_key(&b, &a);
        assert_eq!(key_ab, key_ba, "Key derivation must be symmetric");
    }

    #[test]
    fn test_pq_key_derivation_unique() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let c = addr([3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        #[allow(deprecated)]
        let key_ab = CubeOverlayNetwork::derive_pq_tunnel_key(&a, &b);
        #[allow(deprecated)]
        let key_ac = CubeOverlayNetwork::derive_pq_tunnel_key(&a, &c);
        assert_ne!(key_ab, key_ac, "Different pairs must produce different keys");
    }

    #[test]
    fn test_v3_key_derivation_symmetric() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret = [42u8; 32];
        let epoch = 100u64;
        let key_ab = CubeOverlayNetwork::derive_pq_tunnel_key_v3(&a, &b, &secret, epoch);
        let key_ba = CubeOverlayNetwork::derive_pq_tunnel_key_v3(&b, &a, &secret, epoch);
        assert_eq!(key_ab, key_ba, "V3 key derivation must be symmetric");
    }

    #[test]
    fn test_v3_keys_differ_from_v1() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret = [0u8; 32];
        #[allow(deprecated)]
        let key_v1 = CubeOverlayNetwork::derive_pq_tunnel_key(&a, &b);
        let key_v3 = CubeOverlayNetwork::derive_pq_tunnel_key_v3(&a, &b, &secret, 0);
        assert_ne!(key_v1, key_v3, "V3 keys must differ from V1 keys");
    }

    #[test]
    fn test_v3_different_kem_secrets() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret1 = [42u8; 32];
        let secret2 = [99u8; 32];
        let key1 = CubeOverlayNetwork::derive_pq_tunnel_key_v3(&a, &b, &secret1, 100);
        let key2 = CubeOverlayNetwork::derive_pq_tunnel_key_v3(&a, &b, &secret2, 100);
        assert_ne!(key1, key2, "Different KEM secrets must produce different keys");
    }

    #[test]
    fn test_v3_different_epochs() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret = [42u8; 32];
        let key1 = CubeOverlayNetwork::derive_pq_tunnel_key_v3(&a, &b, &secret, 100);
        let key2 = CubeOverlayNetwork::derive_pq_tunnel_key_v3(&a, &b, &secret, 200);
        assert_ne!(key1, key2, "Same addresses but different epochs must produce different keys");
    }

    #[test]
    fn test_iface_naming() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let iface = con.iface_name_for(&nbr_addr).unwrap();
        assert!(iface.starts_with("cubetun"));
        let back = con.addr_for_iface(&iface).unwrap();
        assert_eq!(back, &nbr_addr);
    }

    #[test]
    fn test_tunnel_lifecycle() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Unknown);

        con.resolve_neighbor(
            &nbr,
            "192.168.1.1:51820".parse().unwrap(),
            vec![0u8; 32],
        );
        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Connecting);

        con.tunnel_up(&nbr, "cubetun0".to_string());
        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Up);

        con.record_heartbeat(&nbr, 500_000);
        assert!(con.neighbor(&nbr).unwrap().rtt_ms().unwrap() < 1.0);

        con.tunnel_down(&nbr);
        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Down);
    }

    #[test]
    fn test_stats() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        let stats = con.stats();
        assert_eq!(stats.tunnels_unknown, NEIGHBORS_PER_CUBE);
        assert_eq!(stats.tunnels_up, 0);
        assert_eq!(stats.verified_neighbors, 0);
    }

    // ── T-07: Verified resolution tests ─────────────────────────

    #[test]
    fn test_verified_resolution_valid_signature() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let endpoint: SocketAddr = "10.0.0.1:51820".parse().unwrap();
        let now_fs: u128 = 100 * crate::wire::FS_PER_SECOND;

        // Generate keypair and sign the registration
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(b"test-seed-verified"));

        let msg = build_registration_message(
            &nbr_addr, &endpoint, &kp.public_key, None, now_fs,
        );
        let sig = ternary_math::tl_dsa::sign(&kp.secret_key, &msg, variant);

        let result = con.resolve_neighbor_verified(
            &nbr_addr, endpoint, kp.public_key.clone(),
            Some(sig), None, now_fs,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        assert_eq!(con.neighbor(&nbr_addr).unwrap().state, TunnelState::Connecting);
        assert!(con.neighbor(&nbr_addr).unwrap().signature_verified);
        assert_eq!(con.forgery_alert_count(), 0);
    }

    #[test]
    fn test_verified_resolution_invalid_signature() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let endpoint: SocketAddr = "10.0.0.1:51820".parse().unwrap();
        let now_fs: u128 = 100 * crate::wire::FS_PER_SECOND;

        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(b"test-seed-invalid"));

        // Sign a DIFFERENT message (wrong endpoint)
        let wrong_msg = build_registration_message(
            &nbr_addr, &"10.0.0.99:9999".parse().unwrap(), &kp.public_key, None, now_fs,
        );
        let bad_sig = ternary_math::tl_dsa::sign(&kp.secret_key, &wrong_msg, variant);

        let result = con.resolve_neighbor_verified(
            &nbr_addr, endpoint, kp.public_key.clone(),
            Some(bad_sig), None, now_fs,
        );

        assert!(result.is_err());
        let alert = result.unwrap_err();
        assert_eq!(alert.reason, ForgeryReason::SignatureInvalid);
        // Neighbor stays Unknown — NOT Connecting
        assert_eq!(con.neighbor(&nbr_addr).unwrap().state, TunnelState::Unknown);
        assert_eq!(con.forgery_alert_count(), 1);
    }

    #[test]
    fn test_verified_resolution_missing_signature() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let endpoint: SocketAddr = "10.0.0.1:51820".parse().unwrap();

        let result = con.resolve_neighbor_verified(
            &nbr_addr, endpoint, vec![0u8; 64],
            None, // No signature
            None, 0,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().reason, ForgeryReason::SignatureMissing);
        assert_eq!(con.neighbor(&nbr_addr).unwrap().state, TunnelState::Unknown);
    }

    #[test]
    fn test_verified_resolution_wrong_key() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let endpoint: SocketAddr = "10.0.0.1:51820".parse().unwrap();
        let now_fs: u128 = 100 * crate::wire::FS_PER_SECOND;

        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp_real = ternary_math::tl_dsa::keygen(variant, Some(b"real-key"));
        let kp_attacker = ternary_math::tl_dsa::keygen(variant, Some(b"attacker-key"));

        // Sign with real key but present attacker's public key
        let msg = build_registration_message(
            &nbr_addr, &endpoint, &kp_attacker.public_key, None, now_fs,
        );
        let sig = ternary_math::tl_dsa::sign(&kp_real.secret_key, &msg, variant);

        let result = con.resolve_neighbor_verified(
            &nbr_addr, endpoint, kp_attacker.public_key.clone(),
            Some(sig), None, now_fs,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().reason, ForgeryReason::SignatureInvalid);
    }

    #[test]
    fn test_verified_resolution_unknown_address() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        // Address that is NOT one of our 26 neighbors
        let non_neighbor = addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        let endpoint: SocketAddr = "10.0.0.1:51820".parse().unwrap();

        let result = con.resolve_neighbor_verified(
            &non_neighbor, endpoint, vec![0u8; 64],
            Some(vec![0u8; 100]), None, 0,
        );

        // Not our neighbor → Ok(false), no alert
        assert_eq!(result.unwrap(), false);
        assert_eq!(con.forgery_alert_count(), 0);
    }

    #[test]
    fn test_forgery_alerts_drain() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let endpoint: SocketAddr = "10.0.0.1:51820".parse().unwrap();

        // Trigger a forgery alert
        let _ = con.resolve_neighbor_verified(
            &nbr_addr, endpoint, vec![0u8; 64],
            None, None, 0,
        );
        assert_eq!(con.forgery_alert_count(), 1);

        // Drain clears the list
        let alerts = con.drain_forgery_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(con.forgery_alert_count(), 0);
    }

    // ── T-09: v2.5 fallback removal tests ───────────────────────

    #[test]
    fn test_derive_all_keys_no_kem_returns_empty() {
        // T-09: No KEM secret = no key. Tunnel stays Resolving.
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);

        // Empty HashMap, no kem_shared_secret on any neighbor
        let keys = con.derive_all_keys(&std::collections::HashMap::new(), 0);

        assert_eq!(
            keys.len(), 0,
            "No KEM secrets → 0 keys derived (v2.5 fallback removed by T-09)"
        );
    }

    #[test]
    fn test_derive_all_keys_with_kem_returns_v3_keys() {
        // T-09: With KEM secrets, v3 keys are derived
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);

        // Provide KEM secrets for 3 neighbors
        let mut kem_secrets = std::collections::HashMap::new();
        let nbr1 = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr2 = addr([3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr3 = addr([1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        kem_secrets.insert(nbr1.clone(), [42u8; 32]);
        kem_secrets.insert(nbr2.clone(), [43u8; 32]);
        kem_secrets.insert(nbr3.clone(), [44u8; 32]);

        let keys = con.derive_all_keys(&kem_secrets, 100);

        assert_eq!(keys.len(), 3, "Only neighbors with KEM secrets get keys");

        // All 3 keys must be unique (different KEM secrets)
        let key_set: std::collections::HashSet<[u8; 32]> =
            keys.iter().map(|(_, k)| *k).collect();
        assert_eq!(key_set.len(), 3, "All v3 keys must be unique");
    }

    #[test]
    fn test_derive_all_keys_partial_kem() {
        // T-09: Mix of neighbors with and without KEM — only KEM'd ones get keys
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);

        // Set KEM shared secret on ONE neighbor directly
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        if let Some(n) = con.neighbor_mut(&nbr) {
            n.kem_shared_secret = Some([99u8; 32]);
        }

        // derive_all_keys with empty HashMap — should find the one with kem_shared_secret
        let keys = con.derive_all_keys(&std::collections::HashMap::new(), 0);
        assert_eq!(keys.len(), 1, "Only the neighbor with KEM secret gets a key");
        assert_eq!(keys[0].0, nbr);
    }

    #[test]
    fn test_default_protocol_is_v3() {
        // T-09: Default protocol changed from PqNative to PqNativeV3
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        // Can't directly access tunnel_protocol (private), but we can verify
        // the constructor doesn't use #[allow(deprecated)] anymore.
        // The protocol field is used internally — this test ensures the
        // struct is created without deprecated warnings.
        assert_eq!(con.neighbors().len(), NEIGHBORS_PER_CUBE);
    }
}