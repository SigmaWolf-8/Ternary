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
    http::StatusCode,
    middleware,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

const CLUSTER_CMD_ALLOWLIST: &[&str] = &[
    "uname", "uname -a",
    "hostname",
    "uptime",
    "whoami",
    "date",
    "df", "df -h",
    "free", "free -h", "free -m",
    "cat /proc/cpuinfo", "cat /proc/meminfo",
    "ip addr", "ifconfig",
    "systemctl status",
    "cargo --version", "rustc --version",
    "id",
    "ps aux",
];

fn contains_shell_metachar(cmd: &str) -> bool {
    cmd.chars().any(|c| matches!(c, ';' | '|' | '&' | '`' | '$' | '(' | ')' | '{' | '}' | '<' | '>' | '!' | '\\' | '\n' | '\r' | '"' | '\''))
}

pub fn is_command_allowed_public(cmd: &str) -> bool {
    is_command_allowed(cmd)
}

fn is_command_allowed(cmd: &str) -> bool {
    let trimmed = cmd.trim();
    if contains_shell_metachar(trimmed) {
        return false;
    }
    CLUSTER_CMD_ALLOWLIST.iter().any(|allowed| trimmed == *allowed)
}

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

pub struct PeerInfo {
    pub host: String,
    pub terminal_port: u16,
}

pub struct ClusterShellState {
    pub local_address: String,
    pub peers: HashMap<String, PeerInfo>,
    pub node_id: u8,
}

pub type SharedClusterShell = Arc<Mutex<ClusterShellState>>;

pub fn new_cluster_shell(local_address: String, node_id: u8, peers: &[(String, String, u16)]) -> SharedClusterShell {
    let mut peer_map = HashMap::new();
    for (addr, host, port) in peers {
        peer_map.insert(addr.clone(), PeerInfo { host: host.clone(), terminal_port: *port });
    }
    Arc::new(Mutex::new(ClusterShellState {
        local_address,
        peers: peer_map,
        node_id,
    }))
}

pub fn register_peer(shell: &SharedClusterShell, address: String, host: String, terminal_port: u16) {
    if let Ok(mut guard) = shell.lock() {
        guard.peers.insert(address, PeerInfo { host, terminal_port });
    }
}

pub fn remove_peer(shell: &SharedClusterShell, address: &str) {
    if let Ok(mut guard) = shell.lock() {
        guard.peers.remove(address);
    }
}

async fn handle_cluster_exec(
    State(shell): State<SharedClusterShell>,
    Json(req): Json<ClusterExecRequest>,
) -> Json<serde_json::Value> {
    if !is_command_allowed(&req.command) {
        return Json(serde_json::json!({
            "ok": false,
            "error": format!(
                "Command not allowed. Permitted commands: {}",
                CLUSTER_CMD_ALLOWLIST.join(", ")
            ),
            "command": req.command,
        }));
    }

    let dial_targets: Vec<(String, String, u16)> = {
        let guard = shell.lock().unwrap_or_else(|e| e.into_inner());
        match &req.targets {
            Some(t) if !t.is_empty() => {
                t.iter().filter_map(|addr| {
                    guard.peers.get(addr).map(|info| (addr.clone(), info.host.clone(), info.terminal_port))
                }).collect()
            }
            _ => {
                guard.peers.iter()
                    .map(|(addr, info)| (addr.clone(), info.host.clone(), info.terminal_port))
                    .collect()
            }
        }
    };

    let target_labels: Vec<String> = dial_targets.iter().map(|(label, _, _)| label.clone()).collect();

    let cluster_cmd = pty_mux::ClusterCommand {
        command: req.command.clone(),
        targets: target_labels,
        timeout_ms: req.timeout_ms,
    };

    let raw_results = pty_mux::fan_out_command(&cluster_cmd, &dial_targets).await;

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

    Json(serde_json::json!({
        "ok": true,
        "result": ClusterExecResponse {
            command: req.command,
            node_count: results.len(),
            success_count,
            results,
        },
    }))
}

async fn handle_cluster_peers(
    State(shell): State<SharedClusterShell>,
) -> Json<serde_json::Value> {
    let guard = shell.lock().unwrap_or_else(|e| e.into_inner());
    let peers: Vec<serde_json::Value> = guard
        .peers
        .iter()
        .map(|(addr, info)| {
            serde_json::json!({
                "address": addr,
                "host": info.host,
                "terminal_port": info.terminal_port,
            })
        })
        .collect();

    Json(serde_json::json!({
        "local_address": guard.local_address,
        "node_id": guard.node_id,
        "peer_count": peers.len(),
        "peers": peers,
    }))
}

fn is_valid_ternary_address(addr: &str) -> bool {
    let parts: Vec<&str> = addr.split('.').collect();
    if parts.len() != 5 { return false; }
    parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c == '1' || c == '2' || c == '3'))
}

async fn handle_register_peer(
    State(shell): State<SharedClusterShell>,
    Json(req): Json<RegisterPeerRequest>,
) -> Json<serde_json::Value> {
    if !is_valid_ternary_address(&req.address) {
        return Json(serde_json::json!({
            "ok": false,
            "error": "Invalid ternary address format (expected dot-separated Rep C trits, e.g. 111.111.111.111.1)",
        }));
    }
    if req.terminal_port < 11111 || req.terminal_port > 11191 {
        return Json(serde_json::json!({
            "ok": false,
            "error": "Terminal port must be in the 11111-11191 range",
        }));
    }
    let host = req.host.clone().unwrap_or_else(|| "127.0.0.1".to_string());
    if host.is_empty() || host.contains(';') || host.contains('&') || host.contains('|') {
        return Json(serde_json::json!({
            "ok": false,
            "error": "Invalid host",
        }));
    }
    let blocked_hosts = ["169.254.169.254", "metadata.google.internal", "metadata.aws", "100.100.100.200"];
    if blocked_hosts.iter().any(|b| host == *b) {
        return Json(serde_json::json!({
            "ok": false,
            "error": "Blocked host (cloud metadata address)",
        }));
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if let std::net::IpAddr::V4(v4) = ip {
            if v4.octets()[0] == 0 || v4.is_broadcast() {
                return Json(serde_json::json!({
                    "ok": false,
                    "error": "Invalid IP address",
                }));
            }
        }
    }
    register_peer(&shell, req.address.clone(), host.clone(), req.terminal_port);
    Json(serde_json::json!({
        "ok": true,
        "registered": req.address,
        "host": host,
        "terminal_port": req.terminal_port,
    }))
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterPeerRequest {
    pub address: String,
    pub host: Option<String>,
    pub terminal_port: u16,
}

async fn cluster_auth_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let expected_token = std::env::var("CUBE_CLUSTER_TOKEN").unwrap_or_default();
    if expected_token.is_empty() {
        eprintln!("[cluster] WARNING: CUBE_CLUSTER_TOKEN not set — all cluster API requests denied. Set this token to enable cluster management.");
        return Err(StatusCode::FORBIDDEN);
    }
    let auth_header = req.headers()
        .get("x-cluster-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if auth_header != expected_token {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

pub fn cluster_shell_router(shell: SharedClusterShell) -> Router {
    Router::new()
        .route("/cluster/exec", post(handle_cluster_exec))
        .route("/cluster/peers", axum::routing::get(handle_cluster_peers))
        .route("/cluster/register-peer", post(handle_register_peer))
        .layer(middleware::from_fn(cluster_auth_middleware))
        .with_state(shell)
}

pub const CLUSTER_ROUTE_COUNT: usize = 3;
