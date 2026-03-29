// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Inter-Cube HTTP API
//!
//! Axum route handlers for the Inter-Cube Infrastructure Services.
//! Extracted from `main.rs` by T-02 (SPEC-2026-NEXT) for cleaner module
//! boundaries and parallel development.
//!
//! ## Routes
//!
//! ### CRS Mode (13 routes — full API)
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | GET | `/health` | [`health_check`] |
//! | GET | `/api/salvi/inter-cube/crs/stats` | [`crs_stats`] |
//! | POST | `/api/salvi/inter-cube/crs/register` | [`crs_register`] |
//! | POST | `/api/salvi/inter-cube/crs/update-key` | [`crs_update_key`] |
//! | POST | `/api/salvi/inter-cube/crs/heartbeat` | [`crs_heartbeat`] |
//! | POST | `/api/salvi/inter-cube/crs/verify-challenge` | [`verify_challenge`] |
//! | POST | `/api/salvi/inter-cube/glb/forward` | [`glb_forward`] |
//! | GET | `/api/salvi/inter-cube/glb/stats` | [`glb_stats`] |
//! | GET | `/api/salvi/inter-cube/con/stats` | [`con_stats`] |
//! | GET | `/api/salvi/inter-cube/fts/status` | [`fts_status`] |
//! | GET | `/api/salvi/inter-cube/fts/dead` | [`fts_dead`] |
//! | GET | `/api/salvi/inter-cube/topology` | [`topology`] |
//! | POST | `/api/salvi/inter-cube/address/validate` | [`address_validate`] |
//!
//! ### Cube Mode (8 routes — no CRS endpoints)
//! Same as CRS mode minus: `/crs/stats`, `/crs/register`, `/crs/heartbeat`
//!
//! ## Future Tasks
//!
//! - T-06: Adds signature field to `crs_register` (signed CRS registration)
//! - T-07: Adds `reg_signature` to CRS query response
//! - T-08: Adds `auth_data` to heartbeat handlers
//! - T-11: Adds rate-limiting middleware to registration
//! - T-21: Adds authenticated deregistration endpoint
//! - T-27: Adds telemetry/metrics endpoints

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::{
    CubeAddr, CubeRegistrationService, CubeOverlayNetwork,
    FaultToleranceService, GeometricLoadBalancer,
    DIMENSIONS, TOTAL_VERTICES, NEIGHBORS_PER_CUBE,
    FRAMEWORK, VERSION,
};

// ═══════════════════════════════════════════════════════════════════════
// SHARED STATE
// ═══════════════════════════════════════════════════════════════════════

/// Shared application state for all Axum handlers.
///
/// Wrapped in `Arc<AppState>` and injected via Axum's `State` extractor.
/// CRS mode has `crs: Some(...)`, cube mode has `crs: None`.
pub struct AppState {
    /// CRS service (only present in CRS mode).
    pub crs: Option<Mutex<CubeRegistrationService>>,
    /// Cube Overlay Network service.
    pub con: Mutex<CubeOverlayNetwork>,
    /// Fault Tolerance Service.
    pub fts: Mutex<FaultToleranceService>,
    /// Geometric Load Balancer.
    pub glb: Mutex<GeometricLoadBalancer>,
    /// This cube's assigned address.
    pub local_address: CubeAddr,
    /// Operating mode: "crs", "cube", or "all".
    pub mode: String,
}

impl AppState {
    /// Create AppState for CRS mode (full API).
    pub fn new_crs(
        crs: CubeRegistrationService,
        con: CubeOverlayNetwork,
        fts: FaultToleranceService,
        glb: GeometricLoadBalancer,
        local_address: CubeAddr,
    ) -> Arc<Self> {
        Arc::new(AppState {
            crs: Some(Mutex::new(crs)),
            con: Mutex::new(con),
            fts: Mutex::new(fts),
            glb: Mutex::new(glb),
            local_address,
            mode: "crs".to_string(),
        })
    }

    /// Create AppState for cube mode (no local CRS).
    pub fn new_cube(
        con: CubeOverlayNetwork,
        fts: FaultToleranceService,
        glb: GeometricLoadBalancer,
        local_address: CubeAddr,
    ) -> Arc<Self> {
        Arc::new(AppState {
            crs: None,
            con: Mutex::new(con),
            fts: Mutex::new(fts),
            glb: Mutex::new(glb),
            local_address,
            mode: "cube".to_string(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// REQUEST / RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════

/// CRS registration request body.
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub endpoint: String,
    #[serde(rename = "publicKey", default = "default_public_key")]
    pub public_key: String,
    pub address: Option<Vec<u8>>,
}

fn default_public_key() -> String {
    "0".repeat(64)
}

/// CRS heartbeat request body.
#[derive(Deserialize)]
pub struct HeartbeatRequest {
    pub address: Vec<u8>,
    pub endpoint: String,
}

/// GLB forwarding request body.
#[derive(Deserialize)]
pub struct ForwardRequest {
    pub destination: Vec<u8>,
    #[serde(rename = "flowId", default)]
    pub flow_id: u64,
}

/// Address validation request body.
#[derive(Deserialize)]
pub struct ValidateRequest {
    pub address: Vec<u8>,
}

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
    pub mode: String,
    pub address: String,
    pub wire_protocol: u8,
    pub wire_protocol_min: u8,
    pub node_id: u8,
}

/// Topology information response.
#[derive(Serialize)]
pub struct TopologyResponse {
    pub dimensions: usize,
    #[serde(rename = "totalVertices")]
    pub total_vertices: u64,
    #[serde(rename = "neighborsPerCube")]
    pub neighbors_per_cube: usize,
    #[serde(rename = "registeredCubes")]
    pub registered_cubes: usize,
    #[serde(rename = "localAddress")]
    pub local_address: String,
    pub mode: String,
}

/// Address validation response.
#[derive(Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub reason: Option<String>,
    pub address: Vec<u8>,
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Parse an address string like "2111111111111" into a CubeAddr.
///
/// Each character must be a digit 1-3 (Rep C). String must be exactly
/// 13 characters. Returns `None` on any invalid input.
pub fn parse_address_string(s: &str) -> Option<CubeAddr> {
    if s.len() != 13 {
        return None;
    }
    let mut arr = [0u8; 13];
    for (i, c) in s.chars().enumerate() {
        match c {
            '1' => arr[i] = 1,
            '2' => arr[i] = 2,
            '3' => arr[i] = 3,
            _ => return None,
        }
    }
    CubeAddr::try_from_bytes(&arr)
}

// ═══════════════════════════════════════════════════════════════════════
// HANDLERS
// ═══════════════════════════════════════════════════════════════════════

/// GET /health — Docker healthcheck.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let node_id = std::env::var("CUBE_NODE_ID")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(1);
    Json(HealthResponse {
        status: "ok",
        service: FRAMEWORK,
        version: VERSION,
        mode: state.mode.clone(),
        address: format!("{}", state.local_address),
        wire_protocol: crate::wire::PROTOCOL_VERSION_CURRENT,
        wire_protocol_min: crate::wire::PROTOCOL_VERSION_MIN,
        node_id,
    })
}

/// GET /api/salvi/inter-cube/node/info — Node identity and status for external dashboards.
pub async fn node_info(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let addr_flat = format!("{}", state.local_address);
    let addr_dotted = state.local_address.to_dotted();
    let llm_port = std::env::var("LLM_PORT").unwrap_or_else(|_| "8080".to_string());
    let node_port = std::env::var("CUBE_API_PORT")
        .or_else(|_| std::env::var("API_PORT"))
        .unwrap_or_else(|_| "8081".to_string());
    let crs_url = std::env::var("CUBE_CRS_URL").unwrap_or_else(|_| "unknown".to_string());

    Json(serde_json::json!({
        "address": addr_flat,
        "addressDotted": addr_dotted,
        "mode": state.mode,
        "crsUrl": crs_url,
        "ports": {
            "engine": llm_port,
            "node": node_port
        }
    }))
}

/// GET /api/salvi/inter-cube/crs/stats (CRS mode only).
pub async fn crs_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
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

/// POST /api/salvi/inter-cube/crs/register (CRS mode only).
pub async fn crs_register(
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

    let public_key = req.public_key.as_bytes().to_vec();

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

            let registered_nbrs = neighbors
                .iter()
                .filter(|n| n["registered"] == true)
                .count();

            println!(
                "[CRS] New cube registered: {} at {}",
                result.address, endpoint
            );
            println!(
                "[CRS] Address space: {} / {} used",
                crs.registered_count(),
                TOTAL_VERTICES
            );

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

/// POST /api/salvi/inter-cube/crs/update-key (CRS mode only).
pub async fn crs_update_key(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let crs_mutex = state.crs.as_ref().ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "This node is not a CRS"})),
    ))?;

    let address_arr = req["address"]
        .as_array()
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing 'address' array"})),
        ))?;
    let mut arr = [0u8; 13];
    for (i, v) in address_arr.iter().take(13).enumerate() {
        arr[i] = v.as_u64().unwrap_or(0) as u8;
    }
    let addr = CubeAddr::try_from_bytes(&arr).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid Rep C address"})),
        )
    })?;

    let public_key = req["publicKey"]
        .as_str()
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing 'publicKey'"})),
        ))?
        .as_bytes()
        .to_vec();

    let mut crs = crs_mutex.lock().unwrap();
    match crs.update_public_key(&addr, public_key) {
        Ok(()) => {
            println!("[CRS] Public key updated for {}", addr);
            Ok(Json(serde_json::json!({
                "status": "ok",
                "address": format!("{}", addr),
            })))
        }
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("{:?}", e)})),
        )),
    }
}

/// POST /api/salvi/inter-cube/crs/heartbeat (CRS mode only).
pub async fn crs_heartbeat(
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

/// POST /api/salvi/inter-cube/glb/forward.
pub async fn glb_forward(
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
        Err(e) => Ok(Json(serde_json::json!({
            "error": format!("{:?}", e),
            "distance": 0,
        }))),
    }
}

/// GET /api/salvi/inter-cube/glb/stats.
pub async fn glb_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
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

/// GET /api/salvi/inter-cube/con/stats.
pub async fn con_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
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

/// GET /api/salvi/inter-cube/fts/status.
pub async fn fts_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
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

/// GET /api/salvi/inter-cube/fts/dead.
pub async fn fts_dead(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let fts = state.fts.lock().unwrap();
    let dead_set = fts.dead_set();
    let addrs: Vec<String> = dead_set.iter().map(|a| format!("{}", a)).collect();
    Json(serde_json::json!({
        "deadCount": addrs.len(),
        "addresses": addrs,
    }))
}

/// GET /api/salvi/inter-cube/topology.
pub async fn topology(State(state): State<Arc<AppState>>) -> Json<TopologyResponse> {
    let registered = if let Some(ref crs) = state.crs {
        crs.lock().unwrap().registered_count()
    } else {
        0
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

/// POST /api/salvi/inter-cube/address/validate.
pub async fn address_validate(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ValidateRequest>,
) -> Json<ValidateResponse> {
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

// ═══════════════════════════════════════════════════════════════════════
// PT26-DSA CHALLENGE VERIFICATION
// ═══════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
struct VerifyChallengePayload {
    #[serde(rename = "publicKey")]
    public_key: String,
    nonce: String,
    signature: String,
    address: Option<String>,
    #[serde(rename = "pt26PublicKey")]
    pt26_public_key: Option<String>,
}

async fn verify_challenge(
    Json(payload): Json<VerifyChallengePayload>,
) -> (StatusCode, Json<serde_json::Value>) {
    let pk_bytes: Vec<u8> = match (0..payload.public_key.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&payload.public_key[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
    {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "valid": false, "error": "invalid publicKey hex" })),
            );
        }
    };

    let sig_bytes: Vec<u8> = match (0..payload.signature.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&payload.signature[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
    {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "valid": false, "error": "invalid signature hex" })),
            );
        }
    };

    if pk_bytes.len() != 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "valid": false, "error": "TL-DSA-87 public key must be 64 bytes" })),
        );
    }

    let addr_str = payload.address.as_deref().unwrap_or("");
    let identity_key = payload.pt26_public_key.as_deref().unwrap_or(&payload.public_key);
    let challenge_payload = format!("{}||{}||{}", payload.nonce, addr_str, identity_key);

    let valid = ternary_math::tl_dsa::verify(
        &pk_bytes,
        challenge_payload.as_bytes(),
        &sig_bytes,
        ternary_math::tl_dsa::TlDsaVariant::TlDsa87,
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({ "valid": valid })),
    )
}

// ═══════════════════════════════════════════════════════════════════════
// ROUTER BUILDERS
// ═══════════════════════════════════════════════════════════════════════

/// Build the full CRS-mode router (13 routes).
///
/// Includes all CRS registration/heartbeat endpoints plus
/// GLB, CON, FTS, topology, address validation, and challenge verification.
pub fn crs_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/salvi/inter-cube/crs/stats", get(crs_stats))
        .route("/api/salvi/inter-cube/crs/register", post(crs_register))
        .route("/api/salvi/inter-cube/crs/update-key", post(crs_update_key))
        .route("/api/salvi/inter-cube/crs/heartbeat", post(crs_heartbeat))
        .route("/api/salvi/inter-cube/crs/verify-challenge", post(verify_challenge))
        .route("/api/salvi/inter-cube/glb/forward", post(glb_forward))
        .route("/api/salvi/inter-cube/glb/stats", get(glb_stats))
        .route("/api/salvi/inter-cube/con/stats", get(con_stats))
        .route("/api/salvi/inter-cube/fts/status", get(fts_status))
        .route("/api/salvi/inter-cube/fts/dead", get(fts_dead))
        .route("/api/salvi/inter-cube/topology", get(topology))
        .route(
            "/api/salvi/inter-cube/address/validate",
            post(address_validate),
        )
        .with_state(state)
}

/// Build the cube-mode router (8 routes — no CRS endpoints).
///
/// Worker cubes don't host CRS, so registration and heartbeat
/// endpoints are omitted. Stats, forwarding, and validation remain.
pub fn cube_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/salvi/inter-cube/node/info", get(node_info))
        .route("/api/salvi/inter-cube/glb/forward", post(glb_forward))
        .route("/api/salvi/inter-cube/glb/stats", get(glb_stats))
        .route("/api/salvi/inter-cube/con/stats", get(con_stats))
        .route("/api/salvi/inter-cube/fts/status", get(fts_status))
        .route("/api/salvi/inter-cube/fts/dead", get(fts_dead))
        .route("/api/salvi/inter-cube/topology", get(topology))
        .route(
            "/api/salvi/inter-cube/address/validate",
            post(address_validate),
        )
        .with_state(state)
}

pub type SharedYodaVerifier = Arc<tokio::sync::Mutex<crate::yoda_chat::YodaChatVerifier>>;
pub type YodaRelaySender = Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<crate::ws_relay::RelayEnvelope>>>>;
pub type YodaResponseWaiters = Arc<tokio::sync::Mutex<std::collections::HashMap<String, tokio::sync::oneshot::Sender<String>>>>;

pub async fn yoda_submit(
    Extension(verifier): Extension<SharedYodaVerifier>,
    Extension(relay_tx): Extension<YodaRelaySender>,
    Extension(waiters): Extension<YodaResponseWaiters>,
    body: String,
) -> (StatusCode, Json<serde_json::Value>) {
    let (session_id, envelope) = {
        let mut v = verifier.lock().await;
        match v.verify_and_forward(&body) {
            Ok((payload, _hash)) => {
                let forward_payload = serde_json::json!({
                    "sessionId": payload.session_id,
                    "timestamp": payload.timestamp,
                    "sequence": payload.sequence,
                    "message": payload.message,
                    "operatorPubkey": payload.operator_pubkey,
                    "daemonRepC": payload.daemon_rep_c,
                    "signature": payload.signature,
                });
                let env = crate::ws_relay::RelayEnvelope {
                    msg_type: "relay".to_string(),
                    to: Some("yoda-server".to_string()),
                    relay_msg_type: Some("yoda_chat".to_string()),
                    payload: Some(forward_payload.to_string()),
                    address: None, public_key: None, nonce: None,
                    signature: None, from: None, error: None,
                    delivered: None, connected_peers: None,
                    ts: None, connected: None,
                };
                (payload.session_id.clone(), env)
            }
            Err(api_err) => {
                let status = match api_err.exit_code {
                    13 => StatusCode::TOO_MANY_REQUESTS,
                    _ => StatusCode::BAD_REQUEST,
                };
                return (status, Json(serde_json::json!({
                    "code": api_err.code,
                    "message": api_err.message,
                    "exitCode": api_err.exit_code,
                })));
            }
        }
    };

    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel::<String>();
    {
        let mut w = waiters.lock().await;
        w.insert(session_id.clone(), resp_tx);
    }

    let send_result = {
        let tx_guard = relay_tx.lock().await;
        if let Some(ref tx) = *tx_guard {
            tx.send(envelope).await.map_err(|_| "send failed")
        } else {
            Err("no relay")
        }
    };

    if send_result.is_err() {
        let mut w = waiters.lock().await;
        w.remove(&session_id);
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "code": "RELAY_DISCONNECTED",
            "message": "Relay connection is not available. Try again shortly.",
            "exitCode": 2,
        })));
    }

    match tokio::time::timeout(std::time::Duration::from_secs(30), resp_rx).await {
        Ok(Ok(response_payload)) => {
            let parsed: serde_json::Value = serde_json::from_str(&response_payload).unwrap_or_default();
            (StatusCode::OK, Json(parsed))
        }
        Ok(Err(_)) | Err(_) => {
            let mut w = waiters.lock().await;
            w.remove(&session_id);
            (StatusCode::GATEWAY_TIMEOUT, Json(serde_json::json!({
                "error": {
                    "code": "YODA_TIMEOUT",
                    "message": "Yoda is taking too long to respond. Try again in a moment."
                }
            })))
        }
    }
}

pub fn yoda_router(verifier: SharedYodaVerifier, relay_tx: YodaRelaySender, waiters: YodaResponseWaiters) -> Router {
    Router::new()
        .route("/yoda/submit", post(yoda_submit))
        .layer(Extension(verifier))
        .layer(Extension(relay_tx))
        .layer(Extension(waiters))
}

// ═══════════════════════════════════════════════════════════════════════
// ROUTE COUNT CONSTANTS (for startup logging)
// ═══════════════════════════════════════════════════════════════════════

/// Number of routes in CRS mode.
pub const CRS_ROUTE_COUNT: usize = 13;

/// Number of routes in cube mode.
pub const CUBE_ROUTE_COUNT: usize = 9;
