// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

mod handlers;
mod types;
mod serialization;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PQTI_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/pqti/algorithms", get(handlers::algorithms))
        .route("/api/pqti/interop/capabilities", get(handlers::interop_capabilities))
        .route("/api/pqti/tldsa/keygen", post(handlers::tldsa_keygen))
        .route("/api/pqti/tldsa/sign", post(handlers::tldsa_sign))
        .route("/api/pqti/tldsa/verify", post(handlers::tldsa_verify))
        .route("/api/pqti/tlkem/keygen", post(handlers::tlkem_keygen))
        .route("/api/pqti/tlkem/encapsulate", post(handlers::tlkem_encapsulate))
        .route("/api/pqti/tlkem/decapsulate", post(handlers::tlkem_decapsulate))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    eprintln!("[PQTI Service] Post-Quantum Ternary Infrastructure API");
    eprintln!("[PQTI Service] Listening on port {}", port);
    eprintln!("[PQTI Service] TL-DSA: keygen, sign, verify (FIPS 204 equivalent)");
    eprintln!("[PQTI Service] TL-KEM: keygen, encapsulate, decapsulate (FIPS 203 equivalent)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
