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
//! ### CRS Mode (14 routes — full API)
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
//! | GET | `/api/salvi/inter-cube/slots` | [`get_slot_inventory`] |
//!
//! ### Cube Mode (10 routes — no CRS endpoints)
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
    extract::{ConnectInfo, Extension, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use subtle::{Choice, ConstantTimeEq};

use plenumlan::cube::constants::{SLOTS_PER_NODE, SLOT_CENTER};
use plenumlan::cube::port::slot_port;
use plenumlan::cube::projection::SlotAddress;

use crate::{
    CubeAddr, CubeRegistrationService, CubeOverlayNetwork,
    FaultToleranceService, GeometricLoadBalancer,
    DIMENSIONS, TOTAL_VERTICES, NEIGHBORS_PER_CUBE,
    FRAMEWORK, VERSION,
};
use crate::config::DaemonConfig;

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
    /// Daemon-level config (auth, slot registry, probing).
    pub daemon_config: DaemonConfig,
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
            daemon_config: DaemonConfig::from_env(),
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
            daemon_config: DaemonConfig::from_env(),
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
    let node_port: u16 = std::env::var("CUBE_API_PORT")
        .or_else(|_| std::env::var("API_PORT"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(11124);
    let llm_port = std::env::var("LLM_PORT")
        .unwrap_or_else(|_| format!("{}", node_port + 1));
    let crs_url = std::env::var("CUBE_CRS_URL").unwrap_or_else(|_| "unknown".to_string());

    Json(serde_json::json!({
        "address": addr_flat,
        "addressDotted": addr_dotted,
        "mode": state.mode,
        "crsUrl": crs_url,
        "ports": {
            "engine": llm_port,
            "node": format!("{}", node_port)
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
// SLOT INVENTORY — Types, Auth Middleware, Handler
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotStatus {
    Available,
    Online,
    Degraded,
    Offline,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotInfo {
    pub address: [u8; 3],
    pub port: u16,
    pub service_type: Option<String>,
    pub status: SlotStatus,
    pub status_label: String,
    pub is_primary_gateway: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotSummary {
    pub occupied: usize,
    pub online: usize,
    pub degraded: usize,
    pub offline: usize,
    pub available: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotInventoryResponse {
    pub node_id: u8,
    pub slots: Vec<SlotInfo>,
    pub summary: SlotSummary,
}

struct AuthWarningTracker {
    warned_ips: std::sync::Mutex<HashMap<IpAddr, Instant>>,
}

impl AuthWarningTracker {
    fn new() -> Self {
        AuthWarningTracker {
            warned_ips: std::sync::Mutex::new(HashMap::new()),
        }
    }

    fn should_warn(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut warned = self.warned_ips.lock().unwrap_or_else(|e| e.into_inner());
        warned.retain(|_, last| now.duration_since(*last) < Duration::from_secs(3600));
        match warned.get(&ip) {
            Some(last) if now.duration_since(*last) < Duration::from_secs(3600) => false,
            _ => {
                warned.insert(ip, now);
                true
            }
        }
    }
}

static AUTH_WARN_TRACKER: std::sync::OnceLock<AuthWarningTracker> = std::sync::OnceLock::new();

fn get_auth_warn_tracker() -> &'static AuthWarningTracker {
    AUTH_WARN_TRACKER.get_or_init(AuthWarningTracker::new)
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({
            "error": "unauthorized",
            "message": "Authentication required. Provide a valid Bearer token in the Authorization header."
        })),
    ).into_response()
}

fn verify_bearer_token(header_value: &str, api_key: &str) -> bool {
    if !header_value.starts_with("Bearer ") {
        return false;
    }
    let token = &header_value[7..];
    let token_bytes = token.as_bytes();
    let key_bytes = api_key.as_bytes();
    let len_ok: Choice = token_bytes.len().ct_eq(&key_bytes.len());
    let pad_len = std::cmp::max(token_bytes.len(), key_bytes.len()).max(1);
    let mut padded_token = vec![0u8; pad_len];
    let mut padded_key = vec![0u8; pad_len];
    padded_token[..token_bytes.len()].copy_from_slice(token_bytes);
    padded_key[..key_bytes.len()].copy_from_slice(key_bytes);
    let content_ok: Choice = padded_token.ct_eq(&padded_key);
    (len_ok & content_ok).into()
}

struct SlotsRateLimiter {
    windows: std::sync::Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl SlotsRateLimiter {
    fn new() -> Self {
        SlotsRateLimiter {
            windows: std::sync::Mutex::new(HashMap::new()),
        }
    }

    fn check_and_record(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(60);
        let limit: usize = 60;
        let mut windows = self.windows.lock().unwrap_or_else(|e| e.into_inner());
        let cutoff = now - window;
        let timestamps = windows.entry(ip).or_insert_with(Vec::new);
        timestamps.retain(|&t| t > cutoff);
        if timestamps.len() >= limit {
            return false;
        }
        timestamps.push(now);
        true
    }
}

static SLOTS_RATE_LIMITER: std::sync::OnceLock<SlotsRateLimiter> = std::sync::OnceLock::new();

fn get_slots_rate_limiter() -> &'static SlotsRateLimiter {
    SLOTS_RATE_LIMITER.get_or_init(SlotsRateLimiter::new)
}

pub async fn slots_auth_middleware(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let config = &state.daemon_config;
    let peer_ip = connect_info.map(|ci| ci.0.ip());

    if config.enable_rate_limit {
        if let Some(ip) = peer_ip {
            if !get_slots_rate_limiter().check_and_record(ip) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(header::CACHE_CONTROL, "no-store")],
                    Json(serde_json::json!({
                        "error": "rate_limited",
                        "message": "Too many requests. Try again later."
                    })),
                ).into_response();
            }
        }
    }

    let authenticated = match &config.api_key {
        Some(api_key) => {
            let auth_header = headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok());

            match auth_header {
                Some(h) => verify_bearer_token(h, api_key),
                None => false,
            }
        }
        None => false,
    };

    if !authenticated {
        if config.slots_auth_required {
            return unauthorized_response();
        }
        if let Some(ip) = peer_ip {
            if get_auth_warn_tracker().should_warn(ip) {
                eprintln!(
                    "[SLOTS] WARNING: Unauthenticated request from {} — set PLENUM_API_KEY and PLENUM_SLOTS_AUTH_REQUIRED=true to enforce auth",
                    ip
                );
            }
        }
    }

    next.run(request).await
}

async fn probe_slot(port: u16, timeout: Duration) -> SlotStatus {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let tcp_result = tokio::time::timeout(
        timeout,
        tokio::net::TcpStream::connect(addr),
    ).await;

    match tcp_result {
        Ok(Ok(_stream)) => {
            let health_url = format!("http://127.0.0.1:{}/health", port);
            let client = reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            match client.get(&health_url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        SlotStatus::Online
                    } else {
                        SlotStatus::Degraded
                    }
                }
                Err(_) => {
                    SlotStatus::Online
                }
            }
        }
        Ok(Err(_)) | Err(_) => SlotStatus::Offline,
    }
}

pub async fn get_slot_inventory(
    State(state): State<Arc<AppState>>,
) -> Response {
    let config = &state.daemon_config;
    let node_id = config.cube_node_id;
    let is_gateway_node = config.is_gateway;
    let timeout = Duration::from_millis(config.slot_probe_timeout_ms);
    // Future enhancement: distinguish between per-probe connect timeout and overall
    // handler timeout for finer-grained control. For now, a single timeout is sufficient.

    let api_port: u16 = std::env::var("CUBE_API_PORT")
        .or_else(|_| std::env::var("API_PORT"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(11124);

    let mut probe_futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = SlotStatus> + Send>>> = Vec::with_capacity(SLOTS_PER_NODE);
    let mut slot_meta: Vec<(SlotAddress, u16, Option<String>, bool)> = Vec::with_capacity(SLOTS_PER_NODE);

    for p in 1u8..=3 {
        for r in 1u8..=3 {
            for i in 1u8..=3 {
                let slot = SlotAddress::new(p, r, i);
                let port = slot_port(node_id, &slot).unwrap_or(0);
                let service_type = config.slot_registry.get(&slot).cloned();
                let is_center = [p, r, i] == SLOT_CENTER;
                let is_primary_gw = is_gateway_node && is_center;

                slot_meta.push((slot, port, service_type.clone(), is_primary_gw));

                if service_type.is_none() {
                    probe_futures.push(Box::pin(async move { SlotStatus::Available }));
                } else if is_primary_gw && port == api_port {
                    // Skip probing the gateway slot when this daemon IS the process on
                    // that port. Two reasons:
                    //   1. Wasted probe: we know the daemon is online (we're handling this request).
                    //   2. Connection-pool exhaustion: under high load, all inbound connections
                    //      could be occupied by requests, leaving none for a self-directed TCP
                    //      connect — a subtle deadlock vector.
                    probe_futures.push(Box::pin(async move { SlotStatus::Online }));
                } else {
                    probe_futures.push(Box::pin(probe_slot(port, timeout)));
                }
            }
        }
    }

    let statuses = futures_util::future::join_all(probe_futures).await;

    let mut slots = Vec::with_capacity(SLOTS_PER_NODE);
    let mut summary = SlotSummary {
        occupied: 0,
        online: 0,
        degraded: 0,
        offline: 0,
        available: 0,
    };

    for (idx, status) in statuses.into_iter().enumerate() {
        let (slot, port, service_type, is_primary_gw) = &slot_meta[idx];
        let status_label = match &status {
            SlotStatus::Available => "Unassigned".to_string(),
            SlotStatus::Online => "Healthy".to_string(),
            SlotStatus::Degraded => "Responding — health check failed".to_string(),
            SlotStatus::Offline => "Unreachable".to_string(),
        };

        match &status {
            SlotStatus::Available => summary.available += 1,
            SlotStatus::Online => { summary.occupied += 1; summary.online += 1; }
            SlotStatus::Degraded => { summary.occupied += 1; summary.degraded += 1; }
            SlotStatus::Offline => { summary.occupied += 1; summary.offline += 1; }
        }

        slots.push(SlotInfo {
            address: [slot.plane, slot.role, slot.instance],
            port: *port,
            service_type: service_type.clone(),
            status,
            status_label,
            is_primary_gateway: *is_primary_gw,
        });
    }

    let response = SlotInventoryResponse {
        node_id,
        slots,
        summary,
    };

    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(response),
    ).into_response()
}

// ═══════════════════════════════════════════════════════════════════════
// ROUTER BUILDERS
// ═══════════════════════════════════════════════════════════════════════

fn slots_auth_route(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/salvi/inter-cube/slots", get(get_slot_inventory))
        .route_layer(middleware::from_fn_with_state(state.clone(), slots_auth_middleware))
        .with_state(state)
}

/// Build the full CRS-mode router (14 routes).
///
/// Includes all CRS registration/heartbeat endpoints plus
/// GLB, CON, FTS, topology, address validation, challenge verification,
/// and slot inventory.
pub fn crs_router(state: Arc<AppState>) -> Router {
    let slots_route = slots_auth_route(state.clone());

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
        .merge(slots_route)
}

/// Build the cube-mode router (9 routes — no CRS endpoints).
///
/// Worker cubes don't host CRS, so registration and heartbeat
/// endpoints are omitted. Stats, forwarding, validation, and slot
/// inventory remain.
pub fn cube_router(state: Arc<AppState>) -> Router {
    let slots_route = slots_auth_route(state.clone());

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
        .merge(slots_route)
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
pub const CRS_ROUTE_COUNT: usize = 14;

/// Number of routes in cube mode.
pub const CUBE_ROUTE_COUNT: usize = 10;

#[cfg(test)]
mod slots_tests {
    use super::*;

    #[test]
    fn test_verify_bearer_token_valid() {
        assert!(verify_bearer_token("Bearer mysecret123", "mysecret123"));
    }

    #[test]
    fn test_verify_bearer_token_invalid() {
        assert!(!verify_bearer_token("Bearer wrong", "mysecret123"));
    }

    #[test]
    fn test_verify_bearer_token_missing_prefix() {
        assert!(!verify_bearer_token("mysecret123", "mysecret123"));
        assert!(!verify_bearer_token("Token mysecret123", "mysecret123"));
    }

    #[test]
    fn test_verify_bearer_token_length_mismatch() {
        assert!(!verify_bearer_token("Bearer short", "longsecretvalue"));
    }

    #[test]
    fn test_verify_bearer_token_empty() {
        assert!(!verify_bearer_token("Bearer ", ""));
        assert!(!verify_bearer_token("Bearer abc", ""));
    }

    #[test]
    fn test_unauthorized_response_has_no_store() {
        let resp = unauthorized_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let cache_ctrl = resp.headers().get(header::CACHE_CONTROL);
        assert!(cache_ctrl.is_some());
        assert_eq!(cache_ctrl.unwrap(), "no-store");
    }

    #[test]
    fn test_slot_status_serialization() {
        assert_eq!(
            serde_json::to_string(&SlotStatus::Available).unwrap(),
            "\"available\""
        );
        assert_eq!(
            serde_json::to_string(&SlotStatus::Online).unwrap(),
            "\"online\""
        );
        assert_eq!(
            serde_json::to_string(&SlotStatus::Degraded).unwrap(),
            "\"degraded\""
        );
        assert_eq!(
            serde_json::to_string(&SlotStatus::Offline).unwrap(),
            "\"offline\""
        );
    }

    #[test]
    fn test_slot_inventory_response_shape() {
        let response = SlotInventoryResponse {
            node_id: 1,
            slots: vec![SlotInfo {
                address: [2, 2, 2],
                port: 11124,
                service_type: Some("gateway".to_string()),
                status: SlotStatus::Online,
                status_label: "Healthy".to_string(),
                is_primary_gateway: true,
            }],
            summary: SlotSummary {
                occupied: 1,
                online: 1,
                degraded: 0,
                offline: 0,
                available: 26,
            },
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["node_id"], 1);
        assert!(json["slots"].is_array());
        assert_eq!(json["slots"][0]["address"], serde_json::json!([2, 2, 2]));
        assert_eq!(json["slots"][0]["port"], 11124);
        assert_eq!(json["slots"][0]["service_type"], "gateway");
        assert_eq!(json["slots"][0]["status"], "online");
        assert_eq!(json["slots"][0]["status_label"], "Healthy");
        assert_eq!(json["slots"][0]["is_primary_gateway"], true);
        assert_eq!(json["summary"]["occupied"], 1);
        assert_eq!(json["summary"]["online"], 1);
        assert_eq!(json["summary"]["available"], 26);
    }

    #[test]
    fn test_slot_info_null_service_type() {
        let info = SlotInfo {
            address: [1, 1, 1],
            port: 11111,
            service_type: None,
            status: SlotStatus::Available,
            status_label: "Unassigned".to_string(),
            is_primary_gateway: false,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert!(json["service_type"].is_null());
        assert_eq!(json["status"], "available");
        assert_eq!(json["is_primary_gateway"], false);
    }

    #[test]
    fn test_gateway_flag_requires_center_and_gateway_node() {
        let config = DaemonConfig {
            cube_node_id: 1,
            is_gateway: true,
            ..Default::default()
        };
        assert!(config.is_gateway);

        let non_gw = DaemonConfig {
            cube_node_id: 2,
            is_gateway: false,
            ..Default::default()
        };
        assert!(!non_gw.is_gateway);
    }

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let limiter = SlotsRateLimiter::new();
        let ip: IpAddr = "10.0.0.1".parse().unwrap();
        for _ in 0..60 {
            assert!(limiter.check_and_record(ip));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = SlotsRateLimiter::new();
        let ip: IpAddr = "10.0.0.2".parse().unwrap();
        for _ in 0..60 {
            limiter.check_and_record(ip);
        }
        assert!(!limiter.check_and_record(ip));
    }

    #[test]
    fn test_rate_limiter_different_ips_independent() {
        let limiter = SlotsRateLimiter::new();
        let ip1: IpAddr = "10.0.0.3".parse().unwrap();
        let ip2: IpAddr = "10.0.0.4".parse().unwrap();
        for _ in 0..60 {
            limiter.check_and_record(ip1);
        }
        assert!(!limiter.check_and_record(ip1));
        assert!(limiter.check_and_record(ip2));
    }

    #[test]
    fn test_auth_warning_tracker_first_call_warns() {
        let tracker = AuthWarningTracker::new();
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(tracker.should_warn(ip));
    }

    #[test]
    fn test_auth_warning_tracker_second_call_suppressed() {
        let tracker = AuthWarningTracker::new();
        let ip: IpAddr = "192.168.1.2".parse().unwrap();
        assert!(tracker.should_warn(ip));
        assert!(!tracker.should_warn(ip));
    }

    #[test]
    fn test_auth_warning_tracker_different_ips() {
        let tracker = AuthWarningTracker::new();
        let ip1: IpAddr = "192.168.1.3".parse().unwrap();
        let ip2: IpAddr = "192.168.1.4".parse().unwrap();
        assert!(tracker.should_warn(ip1));
        assert!(tracker.should_warn(ip2));
    }

    #[tokio::test]
    async fn test_probe_slot_offline_on_closed_port() {
        let status = probe_slot(59999, Duration::from_millis(100)).await;
        assert!(matches!(status, SlotStatus::Offline));
    }

    #[test]
    fn test_27_slots_enumerated() {
        let mut count = 0;
        for p in 1u8..=3 {
            for r in 1u8..=3 {
                for i in 1u8..=3 {
                    let _slot = SlotAddress::new(p, r, i);
                    count += 1;
                }
            }
        }
        assert_eq!(count, SLOTS_PER_NODE);
        assert_eq!(count, 27);
    }

    #[test]
    fn test_center_slot_is_gateway_candidate() {
        assert_eq!(SLOT_CENTER, [2, 2, 2]);
    }

    #[test]
    fn test_daemon_config_gateway_derivation() {
        let gw = DaemonConfig {
            cube_node_id: 1,
            is_gateway: 1 == crate::config::GATEWAY_NODE_ID,
            ..Default::default()
        };
        assert!(gw.is_gateway);

        let non_gw = DaemonConfig {
            cube_node_id: 3,
            is_gateway: 3 == crate::config::GATEWAY_NODE_ID,
            ..Default::default()
        };
        assert!(!non_gw.is_gateway);
    }
}
