// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumNET Inter-Cube Infrastructure Daemon v0.2.0
//
// MODES (controlled by CUBE_MODE env var):
//   "crs"  — Central Registration Service. Allocates addresses,
//            accepts registrations, serves full API on :8080.
//   "cube" — Worker cube. Registers with a remote CRS on boot,
//            gets a unique address, heartbeats every 30s,
//            serves local stats API on :8080.
//   "all"  — Same as "crs" (backward compat).
//
// ENV VARS:
//   CUBE_MODE         — "crs", "cube", or "all" (default: "all")
//   CUBE_CRS_URL      — CRS base URL (required for cube mode)
//   CUBE_ENDPOINT     — This cube's reachable address (default: "0.0.0.0:51820")

use inter_cube::*;
use inter_cube::api::{
    AppState, crs_router, cube_router, parse_address_string,
    CRS_ROUTE_COUNT, CUBE_ROUTE_COUNT,
};
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════
// CRS MODE
// ═══════════════════════════════════════════════════════════════════════

async fn run_crs_mode() {
    // -- Step 1: CRS - Allocate address --------------------------
    let mut crs = CubeRegistrationService::new();
    let endpoint: SocketAddr = "0.0.0.0:51820".parse().unwrap();
    let public_key = [0xABu8; 32];

    let registration = crs
        .register(endpoint, public_key, None)
        .expect("CRS: address allocation failed");

    println!("[CRS] Registered with address: {}", registration.address);

    let mut reg_count = 0;
    let mut unreg_count = 0;
    for nbr in &registration.neighbors {
        if nbr.endpoint.is_some() {
            reg_count += 1;
        } else {
            unreg_count += 1;
        }
    }
    println!(
        "[CRS] 26 neighbors: {} registered, {} awaiting",
        reg_count, unreg_count
    );

    // -- Step 2: CON - Build overlay tunnels ---------------------
    let mut con = CubeOverlayNetwork::new(registration.address.clone());
    for nbr_info in &registration.neighbors {
        if let Some(ep) = nbr_info.endpoint {
            let pk = nbr_info.public_key.unwrap_or([0u8; 32]);
            con.resolve_neighbor(&nbr_info.addr, ep, pk);
        }
    }
    let con_st = con.stats();
    println!(
        "[CON] Overlay: {} up, {} unknown",
        con_st.tunnels_up, con_st.tunnels_unknown
    );
    let keys = con.derive_all_keys(&std::collections::HashMap::new(), 0);
    println!(
        "[CON] {} PQ-native tunnel keys derived (TL-Sponge-385)",
        keys.len()
    );

    // -- Step 3: FTS - Heartbeat monitoring ----------------------
    let fts = FaultToleranceService::new(registration.address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!(
        "[FTS] {} up, {} suspect, {} down, {} recovering",
        up, suspect, down, recovering
    );

    // -- Step 4: GLB - Forwarding engine -------------------------
    let glb = GeometricLoadBalancer::new(registration.address.clone());
    println!(
        "[GLB] {} live neighbors, ready to forward",
        glb.live_neighbor_count()
    );

    // -- Summary -------------------------------------------------
    let local_address = registration.address.clone();
    println!();
    println!("=== Inter-Cube Stack Active ===");
    println!("  Address:       {}", local_address);
    println!(
        "  Address space: {} / {} used",
        crs.registered_count(),
        TOTAL_VERTICES
    );
    println!("  Dimensions:    {}", DIMENSIONS);
    println!("  Neighbors:     {}", NEIGHBORS_PER_CUBE);
    println!("  Protocol:      PQ-Native (TL-Sponge-385 key derivation)");
    println!();
    println!("  CRS -> CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    // -- Step 5: START HTTP SERVER (full CRS API) -----------------
    let shared_state = AppState::new_crs(crs, con, fts, glb, local_address);
    let app = crs_router(shared_state);

    let listen_addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!();
    println!("=== HTTP Server (CRS) ===");
    println!("  http://{}", listen_addr);
    println!("  {} routes active", CRS_ROUTE_COUNT);
    println!("  Ready for cube registrations. Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect("Failed to bind to port 8080");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE MODE
// ═══════════════════════════════════════════════════════════════════════

async fn run_cube_mode() {
    let crs_url = env::var("CUBE_CRS_URL").expect("CUBE_CRS_URL is required for cube mode");
    let cube_endpoint = env::var("CUBE_ENDPOINT")
        .unwrap_or_else(|_| "0.0.0.0:51820".to_string());

    println!("[CUBE] Mode: worker cube");
    println!("[CUBE] CRS URL: {}", crs_url);
    println!("[CUBE] Endpoint: {}", cube_endpoint);
    println!();

    // -- Derive a unique public key from endpoint using TIS-27 ---
    let key_bytes =
        ternary_math::sponge::derive_key(b"PlenumNET-endpoint-key-v1", cube_endpoint.as_bytes(), 32);
    let key_hex: String = key_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    // -- Resolve hostname to IP for SocketAddr compatibility ------
    let resolved_endpoint = match tokio::net::lookup_host(&cube_endpoint).await {
        Ok(mut addrs) => match addrs.next() {
            Some(sa) => {
                println!("[CUBE] Resolved {} -> {}", cube_endpoint, sa);
                sa.to_string()
            }
            None => {
                println!(
                    "[CUBE] WARNING: DNS returned no results for {}, using 0.0.0.0:51820",
                    cube_endpoint
                );
                "0.0.0.0:51820".to_string()
            }
        },
        Err(e) => {
            println!(
                "[CUBE] WARNING: DNS lookup failed for {}: {}, using 0.0.0.0:51820",
                cube_endpoint, e
            );
            "0.0.0.0:51820".to_string()
        }
    };

    // -- Register with CRS (retry up to 10 times) ----------------
    let client = reqwest::Client::new();
    let register_url = format!("{}/api/salvi/inter-cube/crs/register", crs_url);

    let mut response_body: Option<serde_json::Value> = None;
    for attempt in 1..=10 {
        println!(
            "[CUBE] Registration attempt {}/10 -> {}",
            attempt, register_url
        );

        let result = client
            .post(&register_url)
            .json(&serde_json::json!({
                "endpoint": resolved_endpoint,
                "publicKey": key_hex,
            }))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        response_body = Some(body);
                        break;
                    }
                    Err(e) => {
                        println!("[CUBE] Failed to parse response: {}", e);
                    }
                }
            }
            Ok(resp) => {
                println!("[CUBE] CRS returned {}, retrying...", resp.status());
            }
            Err(e) => {
                println!("[CUBE] Connection failed: {}, retrying...", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let reg_data = response_body.expect("Failed to register with CRS after 10 attempts");

    // -- Parse assigned address -----------------------------------
    let addr_str = reg_data["address"]
        .as_str()
        .expect("CRS response missing 'address' field");
    let local_address =
        parse_address_string(addr_str).expect("CRS returned invalid address");

    let registered_nbrs = reg_data["registeredNeighbors"].as_u64().unwrap_or(0);
    let total_nbrs = reg_data["totalNeighbors"].as_u64().unwrap_or(26);

    println!();
    println!("[CUBE] Registered! Address: {}", local_address);
    println!(
        "[CUBE] Neighbors: {} registered, {} total",
        registered_nbrs, total_nbrs
    );

    // -- Initialize local stack with assigned address -------------
    let mut con = CubeOverlayNetwork::new(local_address.clone());

    if let Some(neighbors) = reg_data["neighbors"].as_array() {
        for nbr in neighbors {
            let registered = nbr["registered"].as_bool().unwrap_or(false);
            if registered {
                if let (Some(addr_s), Some(ep_s)) =
                    (nbr["address"].as_str(), nbr["endpoint"].as_str())
                {
                    if let (Some(nbr_addr), Ok(nbr_ep)) =
                        (parse_address_string(addr_s), ep_s.parse::<SocketAddr>())
                    {
                        con.resolve_neighbor(&nbr_addr, nbr_ep, [0u8; 32]);
                        println!("[CON] Resolved neighbor: {} at {}", nbr_addr, nbr_ep);
                    }
                }
            }
        }
    }

    let con_st = con.stats();
    println!(
        "[CON] Overlay: {} up, {} resolving, {} unknown",
        con_st.tunnels_up, con_st.tunnels_resolving, con_st.tunnels_unknown
    );
    let keys = con.derive_all_keys(&std::collections::HashMap::new(), 0);
    println!(
        "[CON] {} PQ-native tunnel keys derived (TL-Sponge-385)",
        keys.len()
    );

    let fts = FaultToleranceService::new(local_address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!(
        "[FTS] {} up, {} suspect, {} down, {} recovering",
        up, suspect, down, recovering
    );

    let glb = GeometricLoadBalancer::new(local_address.clone());
    println!(
        "[GLB] {} live neighbors, ready to forward",
        glb.live_neighbor_count()
    );

    // -- Summary -------------------------------------------------
    println!();
    println!("=== Inter-Cube Stack Active (Cube Mode) ===");
    println!("  Address:       {}", local_address);
    println!("  CRS:           {}", crs_url);
    println!("  Dimensions:    {}", DIMENSIONS);
    println!(
        "  Neighbors:     {} ({} registered)",
        NEIGHBORS_PER_CUBE, registered_nbrs
    );
    println!("  Protocol:      PQ-Native (TL-Sponge-385 key derivation)");
    println!();
    println!("  CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    // -- Heartbeat background task --------------------------------
    let crs_url_for_heartbeat = crs_url.clone();
    let endpoint_for_heartbeat = cube_endpoint.clone();
    let addr_trits: Vec<u8> = addr_str
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();

    tokio::spawn(async move {
        let hb_client = reqwest::Client::new();
        let hb_url = format!(
            "{}/api/salvi/inter-cube/crs/heartbeat",
            crs_url_for_heartbeat
        );
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let result = hb_client
                .post(&hb_url)
                .json(&serde_json::json!({
                    "address": addr_trits,
                    "endpoint": endpoint_for_heartbeat,
                }))
                .send()
                .await;

            match result {
                Ok(resp) if resp.status().is_success() => {}
                Ok(resp) => {
                    println!("[HEARTBEAT] CRS returned {}", resp.status());
                }
                Err(e) => {
                    println!("[HEARTBEAT] Failed: {}", e);
                }
            }
        }
    });

    // -- Start HTTP server (stats-only, no CRS endpoints) ---------
    let shared_state = AppState::new_cube(con, fts, glb, local_address);
    let app = cube_router(shared_state);

    let listen_addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!();
    println!("=== HTTP Server (Cube) ===");
    println!("  http://{}", listen_addr);
    println!("  {} routes active", CUBE_ROUTE_COUNT);
    println!("  Heartbeat every 30s to CRS. Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect("Failed to bind to port 8080");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

// ═══════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    println!("===========================================================");
    println!(
        "  PlenumNET Inter-Cube Infrastructure Services v{}",
        VERSION
    );
    println!("  Applied Physics Division -- Capomastro Holdings Ltd.");
    println!("===========================================================");
    println!();

    let mode = env::var("CUBE_MODE").unwrap_or_else(|_| "all".to_string());

    match mode.as_str() {
        "crs" | "all" => run_crs_mode().await,
        "cube" => run_cube_mode().await,
        other => {
            println!(
                "ERROR: Unknown CUBE_MODE '{}'. Use 'crs', 'cube', or 'all'.",
                other
            );
            std::process::exit(1);
        }
    }
}
