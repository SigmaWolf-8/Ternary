// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumNET Inter-Cube Infrastructure Daemon
// Runs GLB, CON, CRS, and FTS as a single user-space process.

use inter_cube::*;
use std::net::SocketAddr;

fn main() {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  PlenumNET Inter-Cube Infrastructure Services v{}  ║", VERSION);
    println!("║  Applied Physics Division — Capomastro Holdings Ltd. ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!();

    // ── Step 1: CRS — Allocate address ──────────────────────────
    let mut crs = CubeRegistrationService::new();
    let endpoint: SocketAddr = "0.0.0.0:51820".parse().unwrap();
    let public_key = [0xABu8; 32]; // placeholder

    let registration = crs
        .register(endpoint, public_key, None)
        .expect("CRS: address allocation failed");

    println!("[CRS] Registered with address: {}", registration.address);
    println!("[CRS] 26 geometric neighbors computed:");

    let mut registered_count = 0;
    let mut unregistered_count = 0;
    for nbr in &registration.neighbors {
        if nbr.endpoint.is_some() {
            registered_count += 1;
        } else {
            unregistered_count += 1;
        }
    }
    println!("      {} registered, {} awaiting registration",
        registered_count, unregistered_count);

    // ── Step 2: CON — Build overlay tunnels ─────────────────────
    let mut con = CubeOverlayNetwork::new(registration.address.clone());

    // Resolve available neighbors
    for nbr_info in &registration.neighbors {
        if let Some(ep) = nbr_info.endpoint {
            let pk = nbr_info.public_key.unwrap_or([0u8; 32]);
            con.resolve_neighbor(&nbr_info.addr, ep, pk);
        }
    }

    let stats = con.stats();
    println!("[CON] Overlay initialized: {} tunnels up, {} unknown",
        stats.tunnels_up, stats.tunnels_unknown);

    // Derive PQ-native tunnel keys
    let keys = con.derive_all_keys();
    println!("[CON] {} PQ-native tunnel keys derived (BLAKE3)", keys.len());

    // ── Step 3: FTS — Begin heartbeat monitoring ────────────────
    let fts = FaultToleranceService::new(registration.address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!("[FTS] Monitoring 26 neighbors: {} up, {} suspect, {} down, {} recovering",
        up, suspect, down, recovering);

    // ── Step 4: GLB — Ready to forward ──────────────────────────
    let glb = GeometricLoadBalancer::new(registration.address.clone());
    println!("[GLB] Forwarding engine ready: {} live neighbors, 0 dead",
        glb.live_neighbor_count());

    // ── Stack summary ───────────────────────────────────────────
    println!();
    println!("═══ Inter-Cube Stack Active ═══");
    println!("  Address:    {}", registration.address);
    println!("  Address space: {} / {} used",
        crs.registered_count(), TOTAL_VERTICES);
    println!("  Dimensions: {}", DIMENSIONS);
    println!("  Neighbors:  {}", NEIGHBORS_PER_CUBE);
    println!("  Protocol:   PQ-Native (BLAKE3 key derivation)");
    println!();
    println!("  CRS → CON → FTS → GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");
    println!("  No routing tables. No stored state. Pure math.");
}