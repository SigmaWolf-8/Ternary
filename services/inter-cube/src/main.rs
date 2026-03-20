// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumNET Inter-Cube Infrastructure Daemon v0.3.0
//
// MODES (controlled by CUBE_MODE env var):
//   "crs"    — Central Registration Service. Allocates addresses,
//              accepts registrations, serves full API.
//   "cube"   — Worker cube. Registers with a remote CRS on boot,
//              gets a unique address, heartbeats every 30s,
//              serves local stats API.
//   "all"    — Same as "crs" (backward compat).
//   "keygen" — Generate PT26-DSA identity keypair and exit.
//
// ENV VARS:
//   CUBE_MODE                  — "crs", "cube", "all", or "keygen" (default: "all")
//   CUBE_CRS_URL               — CRS base URL (required for cube mode)
//   CUBE_ENDPOINT              — Wire protocol endpoint (default: "0.0.0.0:51820")
//   ADDRESS                    — Alias for CUBE_ENDPOINT
//   CUBE_ROLE                  — Role annotation (inference, review, kb, infra, relay, standby)
//   ROLE                       — Alias for CUBE_ROLE
//   CUBE_API_PORT              — HTTP API bind port (default: 8080)
//   API_PORT                   — Alias for CUBE_API_PORT
//   CUBE_IDENTITY_DIR          — Directory for master.key (default: ~/.plenumnet/identity/)
//   CUBE_IDENTITY_PASSPHRASE   — Passphrase for master.key encryption

use inter_cube::*;
use inter_cube::api::{
    AppState, crs_router, cube_router, parse_address_string,
    CRS_ROUTE_COUNT, CUBE_ROUTE_COUNT,
};
use inter_cube::daemon_identity::{DaemonIdentity, encryption_passphrase, identity_dir, save_master_secret};
use inter_cube::address_keys::derive_identity_keypair;
use inter_cube::key_rotation::RotationOrchestrator;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn env_or(primary: &str, alias: &str, default: &str) -> String {
    env::var(primary)
        .or_else(|_| env::var(alias))
        .unwrap_or_else(|_| default.to_string())
}

fn api_port() -> u16 {
    env::var("CUBE_API_PORT")
        .or_else(|_| env::var("API_PORT"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080)
}

fn role_label() -> Option<String> {
    env::var("CUBE_ROLE")
        .or_else(|_| env::var("ROLE"))
        .ok()
}

// ═══════════════════════════════════════════════════════════════════════
// CRS MODE
// ═══════════════════════════════════════════════════════════════════════

async fn run_crs_mode() {
    let identity = DaemonIdentity::init();

    let mut crs = CubeRegistrationService::new();
    let endpoint: SocketAddr = "0.0.0.0:51820".parse().unwrap();
    let public_key = identity.pk_hex.as_bytes().to_vec();

    let registration = crs
        .register(endpoint, public_key, None)
        .expect("CRS: address allocation failed");

    println!("[CRS] Registered with address: {}", registration.address);
    println!("[CRS] PT26-DSA identity: {}...", &identity.pk_hex[..16]);

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

    let mut con = CubeOverlayNetwork::new(registration.address.clone());
    for nbr_info in &registration.neighbors {
        if let Some(ep) = nbr_info.endpoint {
            let pk = nbr_info.public_key.clone().unwrap_or_default();
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

    let fts = FaultToleranceService::new(registration.address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!(
        "[FTS] {} up, {} suspect, {} down, {} recovering",
        up, suspect, down, recovering
    );

    let glb = GeometricLoadBalancer::new(registration.address.clone());
    println!(
        "[GLB] {} live neighbors, ready to forward",
        glb.live_neighbor_count()
    );

    let local_address = registration.address.clone();

    let addr_bound_kp = derive_identity_keypair(&local_address, &identity.master_secret);
    let addr_bound_pk_hex: String = addr_bound_kp
        .public_key
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!(
        "[IDENTITY] Address-bound TL-DSA-87 key derived (pk: {}...)",
        &addr_bound_pk_hex[..16.min(addr_bound_pk_hex.len())]
    );

    match crs.update_public_key(&local_address, addr_bound_pk_hex.as_bytes().to_vec()) {
        Ok(()) => {
            println!("[IDENTITY] CRS registry updated with address-bound key");
        }
        Err(e) => {
            println!("[IDENTITY] WARNING: CRS key update failed: {:?}", e);
        }
    }

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
    println!("  Protocol:      PQ-Native (PT26-DSA + TL-Sponge-385)");
    println!("  Identity:      PT26-DSA (71-byte sigs, 28-sig budget)");
    println!();
    println!("  CRS -> CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    let shared_state = AppState::new_crs(crs, con, fts, glb, local_address);
    let app = crs_router(shared_state);

    let port = api_port();
    let listen_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    if let Some(role) = role_label() {
        println!("  Role:          {}", role);
    }
    println!();
    println!("=== HTTP Server (CRS) ===");
    println!("  http://{}", listen_addr);
    println!("  {} routes active", CRS_ROUTE_COUNT);
    println!("  Ready for cube registrations. Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect(&format!("Failed to bind to port {}", port));

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE MODE
// ═══════════════════════════════════════════════════════════════════════

async fn run_cube_mode() {
    let crs_url = env::var("CUBE_CRS_URL").expect("CUBE_CRS_URL is required for cube mode");
    let cube_endpoint = env_or("CUBE_ENDPOINT", "ADDRESS", "0.0.0.0:51820");
    let role = role_label();

    println!("[CUBE] Mode: worker cube");
    println!("[CUBE] CRS URL: {}", crs_url);
    println!("[CUBE] Endpoint: {}", cube_endpoint);
    if let Some(ref r) = role {
        println!("[CUBE] Role: {}", r);
    }
    println!();

    let identity = DaemonIdentity::init();
    let key_hex = identity.pk_hex.clone();

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

    let addr_bound_kp = derive_identity_keypair(&local_address, &identity.master_secret);
    let addr_bound_pk_hex: String = addr_bound_kp
        .public_key
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!(
        "[IDENTITY] Address-bound TL-DSA-87 key derived (pk: {}...)",
        &addr_bound_pk_hex[..16.min(addr_bound_pk_hex.len())]
    );

    let addr_trits_vec: Vec<u8> = local_address.to_bytes().to_vec();
    let update_key_url = format!("{}/api/salvi/inter-cube/crs/update-key", crs_url);
    let reregister_result = client
        .post(&update_key_url)
        .json(&serde_json::json!({
            "address": addr_trits_vec,
            "publicKey": addr_bound_pk_hex,
        }))
        .send()
        .await;

    match reregister_result {
        Ok(resp) if resp.status().is_success() => {
            println!("[IDENTITY] Updated CRS with address-bound public key");
        }
        Ok(resp) => {
            println!(
                "[IDENTITY] WARNING: Key update returned {}, continuing with initial key",
                resp.status()
            );
        }
        Err(e) => {
            println!(
                "[IDENTITY] WARNING: Key update failed: {}, continuing with initial key",
                e
            );
        }
    }

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
                        con.resolve_neighbor(&nbr_addr, nbr_ep, vec![0u8; 32]);
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

    println!();
    println!("=== Inter-Cube Stack Active (Cube Mode) ===");
    println!("  Address:       {}", local_address);
    println!("  CRS:           {}", crs_url);
    println!("  Dimensions:    {}", DIMENSIONS);
    println!(
        "  Neighbors:     {} ({} registered)",
        NEIGHBORS_PER_CUBE, registered_nbrs
    );
    println!("  Protocol:      PQ-Native (PT26-DSA + TL-Sponge-385)");
    println!("  Identity:      PT26-DSA (71-byte sigs, 28-sig budget)");
    println!();
    println!("  CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let registration_timestamp = now_ts;
    let passphrase = encryption_passphrase();
    let orchestrator = RotationOrchestrator::new(
        local_address.clone(),
        registration_timestamp,
        identity.master_secret.clone(),
        identity.current_radian_epoch(),
        passphrase.clone(),
    );
    let orchestrator = Arc::new(Mutex::new(orchestrator));

    let crs_url_for_heartbeat = crs_url.clone();
    let endpoint_for_heartbeat = cube_endpoint.clone();
    let addr_trits: Vec<u8> = addr_str
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();
    let orchestrator_hb = orchestrator.clone();
    let local_addr_hb = local_address.clone();
    let passphrase_hb = passphrase.clone();

    tokio::spawn(async move {
        let hb_client = reqwest::Client::new();
        let hb_url = format!(
            "{}/api/salvi/inter-cube/crs/heartbeat",
            crs_url_for_heartbeat
        );
        let update_key_url = format!(
            "{}/api/salvi/inter-cube/crs/update-key",
            crs_url_for_heartbeat
        );
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let unix_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let rotation_result = {
                if let Ok(mut orch) = orchestrator_hb.lock() {
                    match orch.check_and_rotate(unix_now) {
                        Ok(Some(event)) => {
                            println!(
                                "[ROTATION] Key rotated: epoch {} -> {} (forced={})",
                                event.old_epoch, event.new_epoch, event.forced
                            );

                            let new_kp = derive_identity_keypair(
                                &local_addr_hb,
                                orch.current_secret(),
                            );
                            let new_pk_hex: String = new_kp
                                .public_key
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect();

                            let dir = identity_dir();
                            let key_path = dir.join("master.key");
                            save_master_secret(
                                orch.current_secret(),
                                &passphrase_hb,
                                &dir,
                                &key_path,
                            );

                            Some(new_pk_hex)
                        }
                        Ok(None) => None,
                        Err(e) => {
                            println!("[ROTATION] Check failed: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(new_pk_hex) = rotation_result {
                let rereg_result = hb_client
                    .post(&update_key_url)
                    .json(&serde_json::json!({
                        "address": addr_trits,
                        "publicKey": new_pk_hex,
                    }))
                    .send()
                    .await;

                match rereg_result {
                    Ok(resp) if resp.status().is_success() => {
                        println!("[ROTATION] Re-registered with rotated key");
                        if let Ok(mut orch) = orchestrator_hb.lock() {
                            orch.reregistration_complete();
                        }
                    }
                    Ok(resp) => {
                        println!(
                            "[ROTATION] WARNING: Re-registration returned {}",
                            resp.status()
                        );
                    }
                    Err(e) => {
                        println!("[ROTATION] WARNING: Re-registration failed: {}", e);
                    }
                }
            }

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

    let shared_state = AppState::new_cube(con, fts, glb, local_address);
    let app = cube_router(shared_state);

    let port = api_port();
    let listen_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!();
    println!("=== HTTP Server (Cube) ===");
    println!("  http://{}", listen_addr);
    println!("  {} routes active", CUBE_ROUTE_COUNT);
    println!("  Heartbeat every 30s to CRS (with key rotation check). Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect(&format!("Failed to bind to port {}", port));

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
        "keygen" => inter_cube::daemon_identity::run_keygen(),
        other => {
            println!(
                "ERROR: Unknown CUBE_MODE '{}'. Use 'crs', 'cube', 'keygen', or 'all'.",
                other
            );
            std::process::exit(1);
        }
    }
}
