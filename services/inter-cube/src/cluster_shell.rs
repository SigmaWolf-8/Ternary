// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterExecRequest {
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub targets: Option<Vec<String>>,
}

fn default_timeout() -> u64 {
    10_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    pub node_address: String,
    pub output: String,
    pub exit_code: i32,
    pub elapsed_ms: u64,
    pub reachable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClusterExecResponse {
    pub command: String,
    pub node_count: usize,
    pub success_count: usize,
    pub results: Vec<NodeResult>,
}

pub struct ClusterShellState {
    pub local_address: String,
    pub peer_terminal_ports: HashMap<String, u16>,
}

pub type SharedClusterShell = Arc<Mutex<ClusterShellState>>;

pub fn new_cluster_shell(local_address: String) -> SharedClusterShell {
    Arc::new(Mutex::new(ClusterShellState {
        local_address,
        peer_terminal_ports: HashMap::new(),
    }))
}

async fn handle_cluster_exec(
    State(shell): State<SharedClusterShell>,
    Json(req): Json<ClusterExecRequest>,
) -> Json<ClusterExecResponse> {
    let (peer_ports, local_address) = {
        let guard = shell.lock().unwrap_or_else(|e| e.into_inner());
        (guard.peer_terminal_ports.clone(), guard.local_address.clone())
    };

    let targets: Vec<String> = match req.targets {
        Some(ref t) if !t.is_empty() => t.clone(),
        _ => peer_ports.keys().cloned().collect(),
    };

    let cluster_cmd = pty_mux::ClusterCommand {
        command: req.command.clone(),
        targets: targets.clone(),
        timeout_ms: req.timeout_ms,
    };

    let raw_results = pty_mux::fan_out_command(&cluster_cmd, &peer_ports).await;

    let results: Vec<NodeResult> = raw_results
        .into_iter()
        .map(|r| NodeResult {
            node_address: r.node.clone(),
            output: r.output,
            exit_code: r.exit_code,
            elapsed_ms: r.elapsed_ms,
            reachable: r.exit_code >= 0,
        })
        .collect();

    let success_count = results.iter().filter(|r| r.reachable).count();

    Json(ClusterExecResponse {
        command: req.command,
        node_count: results.len(),
        success_count,
        results,
    })
}

async fn handle_cluster_peers(
    State(shell): State<SharedClusterShell>,
) -> Json<serde_json::Value> {
    let guard = shell.lock().unwrap_or_else(|e| e.into_inner());
    let peers: Vec<serde_json::Value> = guard
        .peer_terminal_ports
        .iter()
        .map(|(addr, port)| {
            serde_json::json!({
                "address": addr,
                "terminal_port": port,
            })
        })
        .collect();

    Json(serde_json::json!({
        "local_address": guard.local_address,
        "peer_count": peers.len(),
        "peers": peers,
    }))
}

pub fn cluster_shell_router(shell: SharedClusterShell) -> Router {
    Router::new()
        .route("/cluster/exec", post(handle_cluster_exec))
        .route("/cluster/peers", axum::routing::get(handle_cluster_peers))
        .with_state(shell)
}

pub const CLUSTER_ROUTE_COUNT: usize = 2;
