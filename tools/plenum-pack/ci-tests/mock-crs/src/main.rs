use axum::{
    extract::Json,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
struct RegistrationRequest {
    node_address: String,
    public_key: String,
}

#[derive(Debug, Serialize)]
struct RegistrationResponse {
    status: String,
    shared_secret: String,
    node_id: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    service: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "1.0.0-mock".to_string(),
        service: "mock-crs".to_string(),
    })
}

async fn register(
    Json(payload): Json<RegistrationRequest>,
) -> Result<Json<RegistrationResponse>, StatusCode> {
    if payload.node_address.is_empty() || payload.public_key.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mock_shared_secret = format!(
        "mock-shared-secret-{}",
        &payload.node_address[..std::cmp::min(8, payload.node_address.len())]
    );

    Ok(Json(RegistrationResponse {
        status: "registered".to_string(),
        shared_secret: mock_shared_secret,
        node_id: format!("mock-node-{}", uuid::Uuid::new_v4()),
    }))
}

async fn verify_challenge(
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "verified": true,
        "message": "Mock CRS: challenge verification always succeeds in test mode"
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("MOCK_CRS_PORT")
        .unwrap_or_else(|_| "18080".to_string())
        .parse()?;

    let app = Router::new()
        .route("/health", get(health))
        .route("/crs/register", post(register))
        .route("/crs/verify-challenge", post(verify_challenge));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Mock CRS running on http://{}", addr);
    println!("Endpoints:");
    println!("  GET  /health");
    println!("  POST /crs/register");
    println!("  POST /crs/verify-challenge");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
