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

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use inter_cube::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// -- Shared State ------------------------------------------------
struct AppState {
    crs: Option<Mutex<CubeRegistrationService>>,
    con: Mutex<CubeOverlayNetwork>,
    fts: Mutex<FaultToleranceService>,
    glb: Mutex<GeometricLoadBalancer>,
    local_address: CubeAddr,
    mode: String,
}

// -- Request / Response Types ------------------------------------

#[derive(Deserialize)]
struct RegisterRequest {
    endpoint: String,
    #[serde(rename = "publicKey", default = "default_public_key")]
    public_key: String,
    address: Option<Vec<u8>>,
}

fn default_public_key() -> String {
    "0".repeat(64)
}

#[derive(Deserialize)]
struct HeartbeatRequest {
    address: Vec<u8>,
    endpoint: String,
}

#[derive(Deserialize)]
struct ForwardRequest {
    destination: Vec<u8>,
    #[serde(rename = "flowId", default)]
    flow_id: u64,
}

#[derive(Deserialize)]
struct ValidateRequest {
    address: Vec<u8>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    mode: String,
    address: String,
}

#[derive(Serialize)]
struct TopologyResponse {
    dimensions: usize,
    #[serde(rename = "totalVertices")]
    total_vertices: u64,
    #[serde(rename = "neighborsPerCube")]
    neighbors_per_cube: usize,
    #[serde(rename = "registeredCubes")]
    registered_cubes: usize,
    #[serde(rename = "localAddress")]
    local_address: String,
    mode: String,
}

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
    reason: Option<String>,
    address: Vec<u8>,
}

// -- Handlers ----------------------------------------------------

/// GET /health - Docker healthcheck
async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: FRAMEWORK,
        version: VERSION,
        mode: state.mode.clone(),
        address: format!("{}", state.local_address),
    })
}

/// GET /api/salvi/inter-cube/crs/stats (CRS mode only)
async fn crs_stats(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let crs = state.crs.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let crs = crs.lock().unwrap();
    let count = crs.registered_count();
    Ok(Json(serde_json::json!({
        "registeredCount": count,
        "totalVertices": TOTAL_VERTICES,
        "dimensions": DIMENSIONS,
        "neighborsPerCube": NEIGHBORS_PER_CUBE,
        "utilizationPercent": (count as f64 / TOTAL_VERTICES as f64) * 100.0,
    })))
}

/// POST /api/salvi/inter-cube/crs/register (CRS mode only)
async fn crs_register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let crs_mutex = state.crs.as_ref().ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "This node is not a CRS"})),
    ))?;

    let endpoint: SocketAddr = req.endpoint.parse().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Invalid endpoint: {}", e)})),
        )
    })?;

    let mut public_key = [0u8; 32];
    let key_bytes = req.public_key.as_bytes();
    for i in 0..32.min(key_bytes.len()) {
        public_key[i] = key_bytes[i];
    }

    let specific_addr = if let Some(ref trits) = req.address {
        let mut arr = [0u8; 13];
        for (i, &t) in trits.iter().take(13).enumerate() {
            arr[i] = t;
        }
        CubeAddr::try_from_bytes(&arr)
    } else {
        None
    };

    let mut crs = crs_mutex.lock().unwrap();
    match crs.register(endpoint, public_key, specific_addr) {
        Ok(result) => {
            let neighbors: Vec<serde_json::Value> = result
                .neighbors
                .iter()
                .map(|n| {
                    serde_json::json!({
                        "address": format!("{}", n.addr),
                        "endpoint": n.endpoint.map(|e| e.to_string()),
                        "registered": n.endpoint.is_some(),
                    })
                })
                .collect();

            let registered_nbrs = neighbors.iter()
                .filter(|n| n["registered"] == true)
                .count();

            // Log registration on CRS side
            println!("[CRS] New cube registered: {} at {}", result.address, endpoint);
            println!("[CRS] Address space: {} / {} used", crs.registered_count(), TOTAL_VERTICES);

            Ok(Json(serde_json::json!({
                "address": format!("{}", result.address),
                "endpoint": endpoint.to_string(),
                "neighbors": neighbors,
                "registeredNeighbors": registered_nbrs,
                "totalNeighbors": NEIGHBORS_PER_CUBE,
            })))
        }
        Err(e) => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("{:?}", e)})),
        )),
    }
}

/// POST /api/salvi/inter-cube/crs/heartbeat (CRS mode only)
async fn crs_heartbeat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let crs_mutex = state.crs.as_ref().ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "This node is not a CRS"})),
    ))?;

    let mut arr = [0u8; 13];
    for (i, &t) in req.address.iter().take(13).enumerate() {
        arr[i] = t;
    }
    let addr = CubeAddr::try_from_bytes(&arr).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid Rep C address (zero detected)"})),
        )
    })?;

    let endpoint: SocketAddr = req.endpoint.parse().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Invalid endpoint: {}", e)})),
        )
    })?;

    let mut crs = crs_mutex.lock().unwrap();
    let found = crs.heartbeat(&addr, endpoint);
    if found {
        Ok(Json(serde_json::json!({
            "status": "ok",
            "address": format!("{}", addr),
        })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Address not registered"})),
        ))
    }
}

/// POST /api/salvi/inter-cube/glb/forward
async fn glb_forward(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForwardRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut arr = [0u8; 13];
    for (i, &t) in req.destination.iter().take(13).enumerate() {
        arr[i] = t;
    }
    let dest = CubeAddr::try_from_bytes(&arr).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid Rep C address (zero detected)"})),
        )
    })?;

    let mut glb = state.glb.lock().unwrap();
    match glb.forward(&dest, req.flow_id) {
        Ok(result) => Ok(Json(serde_json::json!({
            "nextHop": format!("{}", result.next_hop),
            "distance": result.total_distance,
            "availablePaths": result.available_paths,
            "dimensionFixed": result.dimension_fixed,
            "flowId": req.flow_id,
        }))),
        Err(e) => {
            Ok(Json(serde_json::json!({
                "error": format!("{:?}", e),
                "distance": 0,
            })))
        }
    }
}

/// GET /api/salvi/inter-cube/glb/stats
async fn glb_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let glb = state.glb.lock().unwrap();
    let stats = glb.stats();
    Json(serde_json::json!({
        "activeFlows": stats.active_flows,
        "totalForwards": stats.total_forwards,
        "detoursComputed": stats.detours_computed,
        "flowsExpired": stats.flows_expired,
        "liveNeighbors": glb.live_neighbor_count(),
    }))
}

/// GET /api/salvi/inter-cube/con/stats
async fn con_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let con = state.con.lock().unwrap();
    let stats = con.stats();
    Json(serde_json::json!({
        "tunnelsUp": stats.tunnels_up,
        "tunnelsDown": stats.tunnels_down,
        "tunnelsResolving": stats.tunnels_resolving,
        "tunnelsConnecting": stats.tunnels_connecting,
        "tunnelsUnknown": stats.tunnels_unknown,
        "totalBytesIn": stats.total_bytes_in,
        "totalBytesOut": stats.total_bytes_out,
        "avgRttMs": stats.avg_rtt_ms,
    }))
}

/// GET /api/salvi/inter-cube/fts/status
async fn fts_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let fts = state.fts.lock().unwrap();
    let (up, suspect, down, recovering) = fts.state_counts();
    Json(serde_json::json!({
        "up": up,
        "suspect": suspect,
        "down": down,
        "recovering": recovering,
        "total": up + suspect + down + recovering,
    }))
}

/// GET /api/salvi/inter-cube/fts/dead
async fn fts_dead(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let fts = state.fts.lock().unwrap();
    let dead_set = fts.dead_set();
    let addrs: Vec<String> = dead_set.iter().map(|a| format!("{}", a)).collect();
    Json(serde_json::json!({
        "deadCount": addrs.len(),
        "addresses": addrs,
    }))
}

/// GET /api/salvi/inter-cube/topology
async fn topology(State(state): State<Arc<AppState>>) -> Json<TopologyResponse> {
    let registered = if let Some(ref crs) = state.crs {
        crs.lock().unwrap().registered_count()
    } else {
        0 // Cube mode: doesn't track global count locally
    };
    Json(TopologyResponse {
        dimensions: DIMENSIONS,
        total_vertices: TOTAL_VERTICES,
        neighbors_per_cube: NEIGHBORS_PER_CUBE,
        registered_cubes: registered,
        local_address: format!("{}", state.local_address),
        mode: state.mode.clone(),
    })
}

/// POST /api/salvi/inter-cube/address/validate
async fn address_validate(Json(req): Json<ValidateRequest>) -> Json<ValidateResponse> {
    if req.address.len() != 13 {
        return Json(ValidateResponse {
            valid: false,
            reason: Some(format!("Expected 13 trits, got {}", req.address.len())),
            address: req.address,
        });
    }

    let mut arr = [0u8; 13];
    for (i, &t) in req.address.iter().take(13).enumerate() {
        arr[i] = t;
    }

    match CubeAddr::try_from_bytes(&arr) {
        Some(_) => Json(ValidateResponse {
            valid: true,
            reason: None,
            address: req.address,
        }),
        None => Json(ValidateResponse {
            valid: false,
            reason: Some("Zero detected in Rep C address -- proof of forgery".to_string()),
            address: req.address,
        }),
    }
}

// -- Helper: parse address string like "2111111111111" into CubeAddr
fn parse_address_string(s: &str) -> Option<CubeAddr> {
    let trits: Vec<u8> = s.chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();
    if trits.len() != 13 {
        return None;
    }
    let mut arr = [0u8; 13];
    arr.copy_from_slice(&trits);
    CubeAddr::try_from_bytes(&arr)
}

// -- CRS Mode ----------------------------------------------------

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
        if nbr.endpoint.is_some() { reg_count += 1; } else { unreg_count += 1; }
    }
    println!("[CRS] 26 neighbors: {} registered, {} awaiting", reg_count, unreg_count);

    // -- Step 2: CON - Build overlay tunnels ---------------------
    let mut con = CubeOverlayNetwork::new(registration.address.clone());
    for nbr_info in &registration.neighbors {
        if let Some(ep) = nbr_info.endpoint {
            let pk = nbr_info.public_key.unwrap_or([0u8; 32]);
            con.resolve_neighbor(&nbr_info.addr, ep, pk);
        }
    }
    let con_st = con.stats();
    println!("[CON] Overlay: {} up, {} unknown", con_st.tunnels_up, con_st.tunnels_unknown);
    let keys = con.derive_all_keys();
    println!("[CON] {} PQ-native tunnel keys derived (BLAKE3)", keys.len());

    // -- Step 3: FTS - Heartbeat monitoring ----------------------
    let fts = FaultToleranceService::new(registration.address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!("[FTS] {} up, {} suspect, {} down, {} recovering", up, suspect, down, recovering);

    // -- Step 4: GLB - Forwarding engine -------------------------
    let glb = GeometricLoadBalancer::new(registration.address.clone());
    println!("[GLB] {} live neighbors, ready to forward", glb.live_neighbor_count());

    // -- Summary -------------------------------------------------
    let local_address = registration.address.clone();
    println!();
    println!("=== Inter-Cube Stack Active ===");
    println!("  Address:       {}", local_address);
    println!("  Address space: {} / {} used", crs.registered_count(), TOTAL_VERTICES);
    println!("  Dimensions:    {}", DIMENSIONS);
    println!("  Neighbors:     {}", NEIGHBORS_PER_CUBE);
    println!("  Protocol:      PQ-Native (BLAKE3 key derivation)");
    println!();
    println!("  CRS -> CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    // -- Step 5: START HTTP SERVER (full CRS API) -----------------
    let shared_state = Arc::new(AppState {
        crs: Some(Mutex::new(crs)),
        con: Mutex::new(con),
        fts: Mutex::new(fts),
        glb: Mutex::new(glb),
        local_address,
        mode: "crs".to_string(),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/salvi/inter-cube/crs/stats", get(crs_stats))
        .route("/api/salvi/inter-cube/crs/register", post(crs_register))
        .route("/api/salvi/inter-cube/crs/heartbeat", post(crs_heartbeat))
        .route("/api/salvi/inter-cube/glb/forward", post(glb_forward))
        .route("/api/salvi/inter-cube/glb/stats", get(glb_stats))
        .route("/api/salvi/inter-cube/con/stats", get(con_stats))
        .route("/api/salvi/inter-cube/fts/status", get(fts_status))
        .route("/api/salvi/inter-cube/fts/dead", get(fts_dead))
        .route("/api/salvi/inter-cube/topology", get(topology))
        .route("/api/salvi/inter-cube/address/validate", post(address_validate))
        .with_state(shared_state);

    let listen_addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!();
    println!("=== HTTP Server (CRS) ===");
    println!("  http://{}", listen_addr);
    println!("  11 routes active");
    println!("  Ready for cube registrations. Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect("Failed to bind to port 8080");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

// -- Cube Mode ---------------------------------------------------

async fn run_cube_mode() {
    let crs_url = env::var("CUBE_CRS_URL")
        .expect("CUBE_CRS_URL is required for cube mode");
    let cube_endpoint = env::var("CUBE_ENDPOINT")
        .unwrap_or_else(|_| "0.0.0.0:51820".to_string());

    println!("[CUBE] Mode: worker cube");
    println!("[CUBE] CRS URL: {}", crs_url);
    println!("[CUBE] Endpoint: {}", cube_endpoint);
    println!();

    // -- Derive a unique public key from endpoint using BLAKE3 ---
    let key_hash = blake3::hash(cube_endpoint.as_bytes());
    let key_hex: String = key_hash.as_bytes().iter()
        .take(32)
        .map(|b| format!("{:02x}", b))
        .collect();

    // -- Register with CRS (retry up to 10 times) ----------------
    let client = reqwest::Client::new();
    let register_url = format!("{}/api/salvi/inter-cube/crs/register", crs_url);

    let mut response_body: Option<serde_json::Value> = None;
    for attempt in 1..=10 {
        println!("[CUBE] Registration attempt {}/10 -> {}", attempt, register_url);

        let result = client
            .post(&register_url)
            .json(&serde_json::json!({
                "endpoint": cube_endpoint,
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
    let addr_str = reg_data["address"].as_str()
        .expect("CRS response missing 'address' field");
    let local_address = parse_address_string(addr_str)
        .expect("CRS returned invalid address");

    let registered_nbrs = reg_data["registeredNeighbors"].as_u64().unwrap_or(0);
    let total_nbrs = reg_data["totalNeighbors"].as_u64().unwrap_or(26);

    println!();
    println!("[CUBE] Registered! Address: {}", local_address);
    println!("[CUBE] Neighbors: {} registered, {} total", registered_nbrs, total_nbrs);

    // -- Initialize local stack with assigned address -------------
    let mut con = CubeOverlayNetwork::new(local_address.clone());

    // Resolve any neighbors the CRS told us about
    if let Some(neighbors) = reg_data["neighbors"].as_array() {
        for nbr in neighbors {
            let registered = nbr["registered"].as_bool().unwrap_or(false);
            if registered {
                if let (Some(addr_s), Some(ep_s)) = (
                    nbr["address"].as_str(),
                    nbr["endpoint"].as_str(),
                ) {
                    if let (Some(nbr_addr), Ok(nbr_ep)) = (
                        parse_address_string(addr_s),
                        ep_s.parse::<SocketAddr>(),
                    ) {
                        con.resolve_neighbor(&nbr_addr, nbr_ep, [0u8; 32]);
                        println!("[CON] Resolved neighbor: {} at {}", nbr_addr, nbr_ep);
                    }
                }
            }
        }
    }

    let con_st = con.stats();
    println!("[CON] Overlay: {} up, {} resolving, {} unknown",
        con_st.tunnels_up, con_st.tunnels_resolving, con_st.tunnels_unknown);
    let keys = con.derive_all_keys();
    println!("[CON] {} PQ-native tunnel keys derived (BLAKE3)", keys.len());

    let fts = FaultToleranceService::new(local_address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!("[FTS] {} up, {} suspect, {} down, {} recovering", up, suspect, down, recovering);

    let glb = GeometricLoadBalancer::new(local_address.clone());
    println!("[GLB] {} live neighbors, ready to forward", glb.live_neighbor_count());

    // -- Summary -------------------------------------------------
    println!();
    println!("=== Inter-Cube Stack Active (Cube Mode) ===");
    println!("  Address:       {}", local_address);
    println!("  CRS:           {}", crs_url);
    println!("  Dimensions:    {}", DIMENSIONS);
    println!("  Neighbors:     {} ({} registered)", NEIGHBORS_PER_CUBE, registered_nbrs);
    println!("  Protocol:      PQ-Native (BLAKE3 key derivation)");
    println!();
    println!("  CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    // -- Build shared state for HTTP server -----------------------
    let crs_url_for_heartbeat = crs_url.clone();
    let endpoint_for_heartbeat = cube_endpoint.clone();
    let addr_trits: Vec<u8> = addr_str.chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();

    let shared_state = Arc::new(AppState {
        crs: None, // Cube mode: no local CRS
        con: Mutex::new(con),
        fts: Mutex::new(fts),
        glb: Mutex::new(glb),
        local_address,
        mode: "cube".to_string(),
    });

    // -- Spawn heartbeat background task --------------------------
    tokio::spawn(async move {
        let hb_client = reqwest::Client::new();
        let hb_url = format!("{}/api/salvi/inter-cube/crs/heartbeat", crs_url_for_heartbeat);
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
                Ok(resp) if resp.status().is_success() => {
                    // Heartbeat OK — silent
                }
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
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/salvi/inter-cube/glb/forward", post(glb_forward))
        .route("/api/salvi/inter-cube/glb/stats", get(glb_stats))
        .route("/api/salvi/inter-cube/con/stats", get(con_stats))
        .route("/api/salvi/inter-cube/fts/status", get(fts_status))
        .route("/api/salvi/inter-cube/fts/dead", get(fts_dead))
        .route("/api/salvi/inter-cube/topology", get(topology))
        .route("/api/salvi/inter-cube/address/validate", post(address_validate))
        .with_state(shared_state);

    let listen_addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!();
    println!("=== HTTP Server (Cube) ===");
    println!("  http://{}", listen_addr);
    println!("  8 routes active");
    println!("  Heartbeat every 30s to CRS. Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect("Failed to bind to port 8080");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

// -- Main --------------------------------------------------------

#[tokio::main]
async fn main() {
    println!("===========================================================");
    println!("  PlenumNET Inter-Cube Infrastructure Services v{}", VERSION);
    println!("  Applied Physics Division -- Capomastro Holdings Ltd.");
    println!("===========================================================");
    println!();

    let mode = env::var("CUBE_MODE").unwrap_or_else(|_| "all".to_string());

    match mode.as_str() {
        "crs" | "all" => run_crs_mode().await,
        "cube" => run_cube_mode().await,
        other => {
            println!("ERROR: Unknown CUBE_MODE '{}'. Use 'crs', 'cube', or 'all'.", other);
            std::process::exit(1);
        }
    }
}
