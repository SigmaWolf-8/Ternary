// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumNET Inter-Cube Infrastructure Daemon
// Runs GLB, CON, CRS, and FTS as a single user-space process
// with an HTTP API on port 8080.
//
// WHAT THIS FILE DOES:
//   1. Initializes all four services (CRS -> CON -> FTS -> GLB)
//   2. Starts an HTTP server on port 8080
//   3. Keeps running forever, accepting requests

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use inter_cube::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

// -- Shared State ------------------------------------------------
struct AppState {
    crs: Mutex<CubeRegistrationService>,
    con: Mutex<CubeOverlayNetwork>,
    fts: Mutex<FaultToleranceService>,
    glb: Mutex<GeometricLoadBalancer>,
    local_address: CubeAddr,
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
}

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
    reason: Option<String>,
    address: Vec<u8>,
}

// -- Handlers ----------------------------------------------------

/// GET /health - Docker healthcheck
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: FRAMEWORK,
        version: VERSION,
    })
}

/// GET /api/salvi/inter-cube/crs/stats
async fn crs_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let crs = state.crs.lock().unwrap();
    let count = crs.registered_count();
    Json(serde_json::json!({
        "registeredCount": count,
        "totalVertices": TOTAL_VERTICES,
        "dimensions": DIMENSIONS,
        "neighborsPerCube": NEIGHBORS_PER_CUBE,
        "utilizationPercent": (count as f64 / TOTAL_VERTICES as f64) * 100.0,
    }))
}

/// POST /api/salvi/inter-cube/crs/register
async fn crs_register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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

    let mut crs = state.crs.lock().unwrap();
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

/// POST /api/salvi/inter-cube/crs/heartbeat
async fn crs_heartbeat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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

    let mut crs = state.crs.lock().unwrap();
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
    let crs = state.crs.lock().unwrap();
    Json(TopologyResponse {
        dimensions: DIMENSIONS,
        total_vertices: TOTAL_VERTICES,
        neighbors_per_cube: NEIGHBORS_PER_CUBE,
        registered_cubes: crs.registered_count(),
        local_address: format!("{}", state.local_address),
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

// -- Main --------------------------------------------------------

#[tokio::main]
async fn main() {
    println!("===========================================================");
    println!("  PlenumNET Inter-Cube Infrastructure Services v{}", VERSION);
    println!("  Applied Physics Division -- Capomastro Holdings Ltd.");
    println!("===========================================================");
    println!();

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

    // -- Step 5: START HTTP SERVER (keeps daemon alive) -----------
    let shared_state = Arc::new(AppState {
        crs: Mutex::new(crs),
        con: Mutex::new(con),
        fts: Mutex::new(fts),
        glb: Mutex::new(glb),
        local_address,
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
    println!("=== HTTP Server ===");
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
