// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use crate::audit::{self, AuditEntry, SharedAuditLog};
use crate::config::{NinjaExecConfig, BIND_ADDRESS};
use crate::confirm::{self, ConfirmationResult, SharedConfirmationQueue};
use crate::keystore::SharedKeystore;
use crate::signing_engine;

#[derive(Clone)]
pub struct AppState {
    pub keystore: SharedKeystore,
    pub audit_log: SharedAuditLog,
    pub config: Arc<NinjaExecConfig>,
    pub start_time: Instant,
    pub signs_this_session: Arc<Mutex<u64>>,
    pub headless: bool,
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
    pub confirmation_queue: SharedConfirmationQueue,
}

pub struct RateLimiter {
    timestamps: Vec<Instant>,
    max_per_minute: u32,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        RateLimiter {
            timestamps: Vec::new(),
            max_per_minute,
        }
    }

    pub fn check(&mut self) -> bool {
        let now = Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);
        self.timestamps.retain(|t| *t > one_minute_ago);
        if self.timestamps.len() >= self.max_per_minute as usize {
            return false;
        }
        self.timestamps.push(now);
        true
    }
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ErrorResponse {
    code: String,
    error: String,
}


const VALID_CONTEXTS: &[&str] = &[
    "sign", "exec", "model-swap", "file-push", "file-pull",
    "deploy", "config-update", "key-rotation", "auth",
    "verify", "pubkey", "status", "tail",
];

fn validate_context(context: &str) -> bool {
    if context.is_empty() {
        return true;
    }
    let prefix = context.split(':').next().unwrap_or("").trim().to_lowercase();
    VALID_CONTEXTS.iter().any(|v| v == &prefix)
}

#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub payload_b64: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Serialize)]
pub struct SignResponse {
    pub signature_b64: String,
    pub pubkey_b64: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub payload_b64: String,
    pub signature_b64: String,
    pub pubkey_b64: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
}

#[derive(Debug, Serialize)]
pub struct PubkeyResponse {
    pub pubkey_b64: String,
    pub fingerprint: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub locked: bool,
    pub uptime_secs: u64,
    pub signs_this_session: u64,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct LockResponse {
    pub locked: bool,
}

#[derive(Debug, Deserialize)]
pub struct UnlockRequest {
    pub passphrase: String,
}

#[derive(Debug, Serialize)]
pub struct UnlockResponse {
    pub unlocked: bool,
}

fn get_origin(headers: &HeaderMap) -> Option<String> {
    headers
        .get("origin")
        .or_else(|| headers.get("referer"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

async fn handle_sign(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let origin = get_origin(&headers);

    {
        let mut rl = state.rate_limiter.lock().unwrap_or_else(|e| e.into_inner());
        if !rl.check() {
            let entry = AuditEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: "sign".to_string(),
                context: Some(req.context.clone()),
                payload_hash: None,
                origin: origin.clone(),
                result: "rate_limited".to_string(),
                confirmation: "n/a".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
            if let Ok(log) = state.audit_log.lock() {
                log.append(&entry);
            }
            return (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({
                "code": "RATE_LIMITED",
                "error": "Rate limit exceeded (max 30 requests/minute)"
            }))).into_response();
        }
    }

    if !validate_context(&req.context) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "code": "INVALID_CONTEXT",
            "error": "Unknown operation context; must be one of: sign, exec, model-swap, file-push, file-pull, deploy, config-update, key-rotation, auth"
        }))).into_response();
    }

    use base64::Engine;
    let payload = match base64::engine::general_purpose::STANDARD.decode(&req.payload_b64) {
        Ok(p) => p,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "code": "INVALID_PAYLOAD",
                "error": "payload_b64 is not valid base64"
            }))).into_response();
        }
    };

    let payload_hash = audit::hash_payload(&payload);

    let initial_result = confirm::evaluate_confirmation(
        &state.config.confirmation,
        &req.context,
        state.headless,
    );

    let confirmation_result = match initial_result {
        ConfirmationResult::AutoApproved | ConfirmationResult::Approved => initial_result,
        ConfirmationResult::Rejected => {
            let request_id = {
                let mut queue = state.confirmation_queue.lock().unwrap_or_else(|e| e.into_inner());
                queue.submit(req.context.clone(), payload_hash.clone(), origin.clone())
            };

            let timeout = std::time::Duration::from_secs(state.config.confirmation.timeout_secs);
            let poll_interval = std::time::Duration::from_millis(250);
            let deadline = Instant::now() + timeout;
            let mut final_result = ConfirmationResult::Timeout;

            while Instant::now() < deadline {
                tokio::time::sleep(poll_interval).await;
                let mut queue = state.confirmation_queue.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(result) = queue.check(&request_id, timeout) {
                    final_result = result;
                    break;
                }
            }

            final_result
        }
        other => other,
    };

    if confirmation_result == ConfirmationResult::Rejected {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "sign".to_string(),
            context: Some(req.context.clone()),
            payload_hash: Some(payload_hash),
            origin,
            result: "rejected".to_string(),
            confirmation: confirm::confirmation_label(&confirmation_result).to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        };
        if let Ok(log) = state.audit_log.lock() {
            log.append(&entry);
        }
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "code": "CONFIRMATION_REJECTED",
            "error": "Signing request was rejected by operator"
        }))).into_response();
    }

    if confirmation_result == ConfirmationResult::Timeout {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "sign".to_string(),
            context: Some(req.context.clone()),
            payload_hash: Some(payload_hash),
            origin,
            result: "timeout".to_string(),
            confirmation: "timeout".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        };
        if let Ok(log) = state.audit_log.lock() {
            log.append(&entry);
        }
        return (StatusCode::REQUEST_TIMEOUT, Json(serde_json::json!({
            "code": "CONFIRMATION_TIMEOUT",
            "error": "Confirmation window expired"
        }))).into_response();
    }

    let ks = state.keystore.lock().unwrap_or_else(|e| e.into_inner());
    let sk = match ks.secret_key() {
        Some(sk) => sk,
        None => {
            let entry = AuditEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: "sign".to_string(),
                context: Some(req.context.clone()),
                payload_hash: Some(payload_hash),
                origin,
                result: "locked".to_string(),
                confirmation: confirm::confirmation_label(&confirmation_result).to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
            if let Ok(log) = state.audit_log.lock() {
                log.append(&entry);
            }
            return (StatusCode::LOCKED, Json(serde_json::json!({
                "code": "KEYSTORE_LOCKED",
                "error": "Keystore is locked — unlock with passphrase first"
            }))).into_response();
        }
    };

    let signature = signing_engine::sign(sk, &payload);
    let pk = ks.public_key().unwrap();
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(&signature);
    let pk_b64 = signing_engine::export_pubkey_b64(pk);

    {
        let mut count = state.signs_this_session.lock().unwrap_or_else(|e| e.into_inner());
        *count += 1;
    }

    let entry = AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "sign".to_string(),
        context: Some(req.context.clone()),
        payload_hash: Some(payload_hash),
        origin,
        result: "signed".to_string(),
        confirmation: confirm::confirmation_label(&confirmation_result).to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    };
    if let Ok(log) = state.audit_log.lock() {
        log.append(&entry);
    }

    (StatusCode::OK, Json(serde_json::json!({
        "signature_b64": sig_b64,
        "pubkey_b64": pk_b64
    }))).into_response()
}

async fn handle_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<VerifyRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let origin = get_origin(&headers);

    use base64::Engine;
    let payload = match base64::engine::general_purpose::STANDARD.decode(&req.payload_b64) {
        Ok(p) => p,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "code": "INVALID_PAYLOAD",
                "error": "payload_b64 is not valid base64"
            }))).into_response();
        }
    };

    let signature = match base64::engine::general_purpose::STANDARD.decode(&req.signature_b64) {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "code": "INVALID_SIGNATURE",
                "error": "signature_b64 is not valid base64"
            }))).into_response();
        }
    };

    let pubkey = match base64::engine::general_purpose::STANDARD.decode(&req.pubkey_b64) {
        Ok(pk) => pk,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "code": "INVALID_PUBKEY",
                "error": "pubkey_b64 is not valid base64"
            }))).into_response();
        }
    };

    let valid = signing_engine::verify(&pubkey, &payload, &signature);

    let entry = AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "verify".to_string(),
        context: None,
        payload_hash: Some(audit::hash_payload(&payload)),
        origin,
        result: if valid { "valid" } else { "invalid" }.to_string(),
        confirmation: "auto".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    };
    if let Ok(log) = state.audit_log.lock() {
        log.append(&entry);
    }

    (StatusCode::OK, Json(serde_json::json!({ "valid": valid }))).into_response()
}

async fn handle_pubkey(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let start = Instant::now();
    let origin = get_origin(&headers);

    let ks = state.keystore.lock().unwrap_or_else(|e| e.into_inner());
    let pk = match ks.public_key() {
        Some(pk) => pk,
        None => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "code": "NO_KEY",
                "error": "No public key available"
            }))).into_response();
        }
    };

    let pk_b64 = signing_engine::export_pubkey_b64(pk);
    let fp = signing_engine::fingerprint(pk);

    let entry = AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "pubkey".to_string(),
        context: None,
        payload_hash: None,
        origin,
        result: "exported".to_string(),
        confirmation: "auto".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    };
    if let Ok(log) = state.audit_log.lock() {
        log.append(&entry);
    }

    (StatusCode::OK, Json(serde_json::json!({
        "pubkey_b64": pk_b64,
        "fingerprint": fp
    }))).into_response()
}

async fn handle_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ks = state.keystore.lock().unwrap_or_else(|e| e.into_inner());
    let locked = !ks.is_unlocked();
    let signs = *state.signs_this_session.lock().unwrap_or_else(|e| e.into_inner());
    let uptime = state.start_time.elapsed().as_secs();

    Json(StatusResponse {
        running: true,
        locked,
        uptime_secs: uptime,
        signs_this_session: signs,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn handle_lock(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut ks = state.keystore.lock().unwrap_or_else(|e| e.into_inner());
    ks.lock();

    if let Ok(log) = state.audit_log.lock() {
        log.append(&AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "lock".to_string(),
            context: None,
            payload_hash: None,
            origin: None,
            result: "locked".to_string(),
            confirmation: "auto".to_string(),
            duration_ms: 0,
        });
    }

    Json(LockResponse { locked: true })
}

async fn handle_unlock(
    State(state): State<AppState>,
    Json(req): Json<UnlockRequest>,
) -> impl IntoResponse {
    {
        let mut rl = state.rate_limiter.lock().unwrap_or_else(|e| e.into_inner());
        if !rl.check() {
            if let Ok(log) = state.audit_log.lock() {
                log.append(&AuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "unlock".to_string(),
                    context: None,
                    payload_hash: None,
                    origin: None,
                    result: "rate_limited".to_string(),
                    confirmation: "n/a".to_string(),
                    duration_ms: 0,
                });
            }
            return (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({
                "code": "RATE_LIMITED",
                "error": "Rate limit exceeded"
            }))).into_response();
        }
    }

    let mut ks = state.keystore.lock().unwrap_or_else(|e| e.into_inner());
    match ks.unlock(&req.passphrase) {
        Ok(_) => {
            if let Ok(log) = state.audit_log.lock() {
                log.append(&AuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "unlock".to_string(),
                    context: None,
                    payload_hash: None,
                    origin: None,
                    result: "unlocked".to_string(),
                    confirmation: "auto".to_string(),
                    duration_ms: 0,
                });
            }
            (StatusCode::OK, Json(serde_json::json!({ "unlocked": true }))).into_response()
        }
        Err(e) => {
            if let Ok(log) = state.audit_log.lock() {
                log.append(&AuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "unlock".to_string(),
                    context: None,
                    payload_hash: None,
                    origin: None,
                    result: "error".to_string(),
                    confirmation: "auto".to_string(),
                    duration_ms: 0,
                });
            }
            (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "code": "UNLOCK_FAILED",
                "error": e.to_string()
            }))).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfirmDecisionRequest {
    pub request_id: String,
    pub decision: String,
}

fn check_confirm_token(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if let Some(ref expected_token) = state.config.confirm_token {
        let provided = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .unwrap_or("");
        if provided != expected_token {
            return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "code": "UNAUTHORIZED",
                "error": "Invalid or missing confirm_token"
            }))));
        }
    }
    Ok(())
}

async fn handle_confirm_pending(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = check_confirm_token(&state, &headers) {
        return resp.into_response();
    }
    let mut queue = state.confirmation_queue.lock().unwrap_or_else(|e| e.into_inner());
    let timeout = std::time::Duration::from_secs(state.config.confirmation.timeout_secs);
    queue.expire_stale(timeout);
    let pending: Vec<serde_json::Value> = queue.pending_list().iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "context": r.context,
            "payload_hash": r.payload_hash,
            "origin": r.origin,
            "age_secs": r.created.elapsed().as_secs()
        })
    }).collect();
    Json(serde_json::json!({ "pending": pending })).into_response()
}

async fn handle_confirm_decide(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ConfirmDecisionRequest>,
) -> impl IntoResponse {
    if let Err(resp) = check_confirm_token(&state, &headers) {
        return resp.into_response();
    }
    let mut queue = state.confirmation_queue.lock().unwrap_or_else(|e| e.into_inner());
    let ok = match req.decision.as_str() {
        "approve" => queue.approve(&req.request_id),
        "reject" => queue.reject(&req.request_id),
        _ => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "code": "INVALID_DECISION",
                "error": "decision must be 'approve' or 'reject'"
            }))).into_response();
        }
    };

    if ok {
        if let Ok(log) = state.audit_log.lock() {
            log.append(&AuditEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: "confirm_decide".to_string(),
                context: Some(format!("{}: {}", req.request_id, req.decision)),
                payload_hash: None,
                origin: None,
                result: req.decision.clone(),
                confirmation: req.decision,
                duration_ms: 0,
            });
        }
        (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "code": "NOT_FOUND",
            "error": "No pending request with that ID"
        }))).into_response()
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/sign", post(handle_sign))
        .route("/verify", post(handle_verify))
        .route("/pubkey", get(handle_pubkey))
        .route("/status", get(handle_status))
        .route("/lock", post(handle_lock))
        .route("/unlock", post(handle_unlock))
        .route("/confirm/pending", get(handle_confirm_pending))
        .route("/confirm/decide", post(handle_confirm_decide))
        .layer(cors)
        .with_state(state)
}

pub async fn serve(state: AppState, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("{}:{}", BIND_ADDRESS, port).parse()?;
    let router = build_router(state);

    println!("[NinjaExec] Signing agent listening on {}", addr);
    println!("[NinjaExec] Bound to {} only — not accessible from network", BIND_ADDRESS);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
