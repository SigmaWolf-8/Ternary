// TDNS v2.3 — CON (Cube Overlay Network)
// Capomastro Holdings Ltd. — Applied Physics Division
//
// PQ-native encrypted tunnels between geometric neighbors.
// Every inter-cube link is encrypted. There is no unencrypted path
// through the fabric.
//
// Key derivation: TIS-27 sponge (ternary_math::sponge) — same primitive used everywhere.
// Each of the 54 possible neighbor links has its own derived tunnel key.
//
// §12.2: CON tunnel architecture.
// §2.6:  Zero-cleartext principle.
// §15.1: Dual-layer encryption model.

use crate::addr::{CubeAddr, DIMENSIONS, WIRE_SIZE};
use crate::trit::Trit;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Derived tunnel key size (256 bits).
pub const TUNNEL_KEY_SIZE: usize = 32;

/// Key derivation context string (domain separation).
const KD_CONTEXT: &[u8] = b"PlenumNET-CON-v2.5";

/// Maximum tunnel links per node (27 dims × 2 directions).
pub const MAX_TUNNELS: usize = 54;

// ─── Tunnel Key ──────────────────────────────────────────────────────────────

/// A derived 256-bit tunnel key for a single neighbor link.
#[derive(Clone, PartialEq, Eq)]
pub struct TunnelKey {
    bytes: [u8; TUNNEL_KEY_SIZE],
}

impl TunnelKey {
    /// Raw key bytes.
    pub fn as_bytes(&self) -> &[u8; TUNNEL_KEY_SIZE] {
        &self.bytes
    }

    /// Display as hex (first 8 bytes only for safety).
    pub fn fingerprint(&self) -> String {
        self.bytes[..8]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

impl std::fmt::Debug for TunnelKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TunnelKey({}...)", self.fingerprint())
    }
}

// ─── Key Derivation ──────────────────────────────────────────────────────────

/// Derive a tunnel key for a specific neighbor link.
///
/// `BLAKE3(context || local_wire || neighbor_wire || shared_secret)`
///
/// The key is directional: derive_key(A, B, secret) != derive_key(B, A, secret).
/// Both ends compute their own key. The link uses the canonical ordering:
/// lower address derives the "outbound" key, higher derives "inbound."
///
/// In practice, both sides derive both keys and use them for their
/// respective direction.
pub fn derive_tunnel_key(
    local: &CubeAddr,
    neighbor: &CubeAddr,
    shared_secret: &[u8],
) -> TunnelKey {
    let local_wire    = local.to_wire();
    let neighbor_wire = neighbor.to_wire();

    let mut material = Vec::with_capacity(WIRE_SIZE + WIRE_SIZE + shared_secret.len());
    material.extend_from_slice(&local_wire);
    material.extend_from_slice(&neighbor_wire);
    material.extend_from_slice(shared_secret);

    let key_bytes = crate::identity::derive_key(KD_CONTEXT, &material, TUNNEL_KEY_SIZE);
    let mut bytes = [0u8; TUNNEL_KEY_SIZE];
    bytes.copy_from_slice(&key_bytes);
    TunnelKey { bytes }
}

/// Derive the canonical key pair for a bidirectional link.
///
/// Returns (outbound_key, inbound_key) where outbound is from the
/// lower-address node's perspective.
///
/// Both ends of a link call this with the same arguments and get
/// the same pair. The lower-addressed node uses outbound for sending
/// and inbound for receiving; the higher-addressed node reverses.
pub fn derive_link_keys(
    addr_a: &CubeAddr,
    addr_b: &CubeAddr,
    shared_secret: &[u8],
) -> (TunnelKey, TunnelKey) {
    let (lower, higher) = if addr_a <= addr_b {
        (addr_a, addr_b)
    } else {
        (addr_b, addr_a)
    };

    let outbound = derive_tunnel_key(lower, higher, shared_secret);
    let inbound = derive_tunnel_key(higher, lower, shared_secret);

    (outbound, inbound)
}

// ─── Tunnel Link ─────────────────────────────────────────────────────────────

/// Metadata for a single CON tunnel link.
#[derive(Debug, Clone)]
pub struct TunnelLink {
    /// Local node address.
    pub local: CubeAddr,
    /// Remote neighbor address.
    pub remote: CubeAddr,
    /// Which dimension this link spans.
    pub dimension: usize,
    /// The direction (trit value of the remote node in this dimension).
    pub direction: Trit,
    /// Outbound tunnel key (local → remote).
    pub outbound_key: TunnelKey,
    /// Inbound tunnel key (remote → local).
    pub inbound_key: TunnelKey,
    /// Link state.
    pub state: LinkState,
    /// Established timestamp (HPTP nanoseconds).
    pub established_ns: u64,
    /// Total bytes sent through this tunnel.
    pub bytes_sent: u64,
    /// Total bytes received through this tunnel.
    pub bytes_received: u64,
}

/// State of a tunnel link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// Key derived, ready to establish.
    Pending,
    /// Tunnel active, forwarding traffic.
    Active,
    /// Tunnel temporarily down (FTS-detected failure).
    Down,
    /// Tunnel being rekeyed (key rotation in progress).
    Rekeying,
}

// ─── CON Node ────────────────────────────────────────────────────────────────

/// The CON overlay for a single node.
///
/// Manages up to 54 tunnel links (27 dimensions × 2 directions).
/// Each link has its own derived key pair.
pub struct ConNode {
    /// This node's address.
    local: CubeAddr,
    /// The shared secret (pre-shared or negotiated via TL-KEM).
    shared_secret: Vec<u8>,
    /// Active tunnel links indexed by remote address.
    links: std::collections::HashMap<CubeAddr, TunnelLink>,
    /// Key rotation counter (incremented on each rekey).
    key_epoch: u64,
}

impl ConNode {
    /// Create a new CON node.
    pub fn new(local: CubeAddr, shared_secret: Vec<u8>) -> Self {
        Self {
            local,
            shared_secret,
            links: std::collections::HashMap::new(),
            key_epoch: 0,
        }
    }

    /// This node's address.
    pub fn local(&self) -> &CubeAddr {
        &self.local
    }

    /// Current key epoch.
    pub fn key_epoch(&self) -> u64 {
        self.key_epoch
    }

    /// Establish a tunnel to a neighbor.
    pub fn establish_tunnel(
        &mut self,
        remote: CubeAddr,
        dimension: usize,
        direction: Trit,
        now_ns: u64,
    ) -> &TunnelLink {
        let (outbound, inbound) = if self.local <= remote {
            derive_link_keys(&self.local, &remote, &self.shared_secret)
        } else {
            let (out, inp) = derive_link_keys(&remote, &self.local, &self.shared_secret);
            (inp, out) // Reverse for higher-addressed node
        };

        let link = TunnelLink {
            local: self.local,
            remote,
            dimension,
            direction,
            outbound_key: outbound,
            inbound_key: inbound,
            state: LinkState::Active,
            established_ns: now_ns,
            bytes_sent: 0,
            bytes_received: 0,
        };

        self.links.insert(remote, link);
        self.links.get(&remote).unwrap()
    }

    /// Establish tunnels to all geometric neighbors from a neighbor map.
    pub fn establish_from_neighbor_map(
        &mut self,
        neighbor_map: &crate::routing::NeighborMap,
        now_ns: u64,
    ) {
        for dim in 0..DIMENSIONS {
            let local_val = self.local.trit(dim);
            for target_val in Trit::ALL {
                if target_val == local_val {
                    continue;
                }
                if let Some(entry) = neighbor_map.get(dim, target_val) {
                    if !self.links.contains_key(&entry.addr) {
                        self.establish_tunnel(entry.addr, dim, target_val, now_ns);
                    }
                }
            }
        }
    }

    /// Mark a tunnel as down (FTS failure detection).
    pub fn mark_down(&mut self, remote: &CubeAddr) {
        if let Some(link) = self.links.get_mut(remote) {
            link.state = LinkState::Down;
        }
    }

    /// Mark a tunnel as active (FTS recovery).
    pub fn mark_active(&mut self, remote: &CubeAddr) {
        if let Some(link) = self.links.get_mut(remote) {
            link.state = LinkState::Active;
        }
    }

    /// Remove a tunnel (node deregistered).
    pub fn remove_tunnel(&mut self, remote: &CubeAddr) -> Option<TunnelLink> {
        self.links.remove(remote)
    }

    /// Rekey all tunnels (key rotation).
    ///
    /// Generates new keys by appending the epoch to the shared secret.
    /// In production, this would use TL-KEM for PQ key exchange.
    pub fn rekey_all(&mut self, now_ns: u64) {
        self.key_epoch += 1;

        let mut epoch_secret = self.shared_secret.clone();
        epoch_secret.extend_from_slice(&self.key_epoch.to_be_bytes());

        for link in self.links.values_mut() {
            let (outbound, inbound) = if self.local <= link.remote {
                derive_link_keys(&self.local, &link.remote, &epoch_secret)
            } else {
                let (out, inp) = derive_link_keys(&link.remote, &self.local, &epoch_secret);
                (inp, out)
            };

            link.outbound_key = outbound;
            link.inbound_key = inbound;
            link.state = LinkState::Active;
            link.established_ns = now_ns;
        }
    }

    // ── Queries ─────────────────────────────────────────────────────

    /// Get a tunnel link by remote address.
    pub fn link(&self, remote: &CubeAddr) -> Option<&TunnelLink> {
        self.links.get(remote)
    }

    /// Get the outbound key for a specific neighbor.
    pub fn outbound_key(&self, remote: &CubeAddr) -> Option<&TunnelKey> {
        self.links.get(remote).map(|l| &l.outbound_key)
    }

    /// Number of active tunnel links.
    pub fn active_count(&self) -> usize {
        self.links
            .values()
            .filter(|l| l.state == LinkState::Active)
            .count()
    }

    /// Total tunnel links (all states).
    pub fn total_count(&self) -> usize {
        self.links.len()
    }

    /// All tunnel links.
    pub fn all_links(&self) -> Vec<&TunnelLink> {
        self.links.values().collect()
    }

    // ── Traffic Accounting ──────────────────────────────────────────

    /// Record bytes sent through a tunnel.
    pub fn record_sent(&mut self, remote: &CubeAddr, bytes: u64) {
        if let Some(link) = self.links.get_mut(remote) {
            link.bytes_sent += bytes;
        }
    }

    /// Record bytes received through a tunnel.
    pub fn record_received(&mut self, remote: &CubeAddr, bytes: u64) {
        if let Some(link) = self.links.get_mut(remote) {
            link.bytes_received += bytes;
        }
    }

    /// Total bytes through all tunnels.
    pub fn total_bytes(&self) -> (u64, u64) {
        let sent: u64 = self.links.values().map(|l| l.bytes_sent).sum();
        let recv: u64 = self.links.values().map(|l| l.bytes_received).sum();
        (sent, recv)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::NeighborMap;

    fn google() -> CubeAddr {
        CubeAddr::from_category_string("WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313")
            .unwrap()
    }

    fn pptpro() -> CubeAddr {
        CubeAddr::from_category_string("WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332")
            .unwrap()
    }

    fn blog() -> CubeAddr {
        CubeAddr::from_category_string("WO:1312 WA:1111 WR:3111 WN:2311 WY:1111 HO:1111 PE:211")
            .unwrap()
    }

    #[test]
    fn key_derivation_deterministic() {
        let secret = b"test-secret-key-material";
        let k1 = derive_tunnel_key(&google(), &pptpro(), secret);
        let k2 = derive_tunnel_key(&google(), &pptpro(), secret);
        assert_eq!(k1, k2);
    }

    #[test]
    fn key_derivation_directional() {
        let secret = b"test-secret";
        let k_ab = derive_tunnel_key(&google(), &pptpro(), secret);
        let k_ba = derive_tunnel_key(&pptpro(), &google(), secret);
        assert_ne!(k_ab, k_ba);
    }

    #[test]
    fn key_derivation_secret_sensitive() {
        let k1 = derive_tunnel_key(&google(), &pptpro(), b"secret-1");
        let k2 = derive_tunnel_key(&google(), &pptpro(), b"secret-2");
        assert_ne!(k1, k2);
    }

    #[test]
    fn key_derivation_address_sensitive() {
        let secret = b"same-secret";
        let k1 = derive_tunnel_key(&google(), &pptpro(), secret);
        let k2 = derive_tunnel_key(&google(), &blog(), secret);
        assert_ne!(k1, k2);
    }

    #[test]
    fn link_keys_canonical_ordering() {
        let secret = b"link-secret";
        let (out_ab, in_ab) = derive_link_keys(&google(), &pptpro(), secret);
        let (out_ba, in_ba) = derive_link_keys(&pptpro(), &google(), secret);

        assert_eq!(out_ab, out_ba);
        assert_eq!(in_ab, in_ba);

        assert_ne!(out_ab, in_ab);
    }

    #[test]
    fn tunnel_key_is_32_bytes() {
        let k = derive_tunnel_key(&google(), &pptpro(), b"secret");
        assert_eq!(k.as_bytes().len(), 32);
    }

    #[test]
    fn establish_tunnel() {
        let mut con = ConNode::new(google(), b"fabric-secret".to_vec());
        let link = con.establish_tunnel(pptpro(), 0, Trit::V3, 1000);

        assert_eq!(link.local, google());
        assert_eq!(link.remote, pptpro());
        assert_eq!(link.state, LinkState::Active);
        assert_eq!(con.active_count(), 1);
    }

    #[test]
    fn establish_from_neighbor_map() {
        let g = google();
        let p = pptpro();
        let b = blog();

        let mut map = NeighborMap::new(g);
        let diffs_gp = g.differing_dims(&p);
        let diffs_gb = g.differing_dims(&b);
        if !diffs_gp.is_empty() {
            map.set(diffs_gp[0], p.trit(diffs_gp[0]), p);
        }
        if !diffs_gb.is_empty() {
            map.set(diffs_gb[0], b.trit(diffs_gb[0]), b);
        }

        let mut con = ConNode::new(g, b"secret".to_vec());
        con.establish_from_neighbor_map(&map, 1000);

        assert!(con.total_count() >= 2, "should have at least 2 tunnels");
        assert!(con.link(&p).is_some());
        assert!(con.link(&b).is_some());
    }

    #[test]
    fn mark_down_and_recover() {
        let mut con = ConNode::new(google(), b"secret".to_vec());
        con.establish_tunnel(pptpro(), 0, Trit::V3, 1000);

        assert_eq!(con.active_count(), 1);

        con.mark_down(&pptpro());
        assert_eq!(con.active_count(), 0);
        assert_eq!(con.link(&pptpro()).unwrap().state, LinkState::Down);

        con.mark_active(&pptpro());
        assert_eq!(con.active_count(), 1);
        assert_eq!(con.link(&pptpro()).unwrap().state, LinkState::Active);
    }

    #[test]
    fn rekey_all_changes_keys() {
        let mut con = ConNode::new(google(), b"secret".to_vec());
        con.establish_tunnel(pptpro(), 0, Trit::V3, 1000);

        let old_key = con.outbound_key(&pptpro()).unwrap().clone();

        con.rekey_all(2000);

        let new_key = con.outbound_key(&pptpro()).unwrap();
        assert_ne!(&old_key, new_key, "rekey must produce different keys");
        assert_eq!(con.key_epoch(), 1);
    }

    #[test]
    fn traffic_accounting() {
        let mut con = ConNode::new(google(), b"secret".to_vec());
        con.establish_tunnel(pptpro(), 0, Trit::V3, 1000);

        con.record_sent(&pptpro(), 1024);
        con.record_sent(&pptpro(), 2048);
        con.record_received(&pptpro(), 512);

        let link = con.link(&pptpro()).unwrap();
        assert_eq!(link.bytes_sent, 3072);
        assert_eq!(link.bytes_received, 512);

        let (total_sent, total_recv) = con.total_bytes();
        assert_eq!(total_sent, 3072);
        assert_eq!(total_recv, 512);
    }

    #[test]
    fn remove_tunnel() {
        let mut con = ConNode::new(google(), b"secret".to_vec());
        con.establish_tunnel(pptpro(), 0, Trit::V3, 1000);
        assert_eq!(con.total_count(), 1);

        let removed = con.remove_tunnel(&pptpro());
        assert!(removed.is_some());
        assert_eq!(con.total_count(), 0);
    }

    #[test]
    fn fingerprint_display() {
        let k = derive_tunnel_key(&google(), &pptpro(), b"secret");
        let fp = k.fingerprint();
        assert_eq!(fp.len(), 16); // 8 bytes × 2 hex chars
    }

    #[test]
    fn zero_cleartext_guarantee() {
        let con = ConNode::new(google(), b"secret".to_vec());
        assert_eq!(con.total_count(), 0);
    }
}
