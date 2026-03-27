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

const CLUSTER_CMD_ALLOWLIST: &[&str] = &[
    "uname", "hostname", "uptime", "whoami", "date", "df", "free",
    "cat /proc/cpuinfo", "cat /proc/meminfo", "ip addr", "ifconfig",
    "systemctl status", "cargo --version", "rustc --version",
    "echo", "env", "printenv", "id", "ps aux",
];

fn contains_shell_metachar(cmd: &str) -> bool {
    cmd.chars().any(|c| matches!(c, ';' | '|' | '&' | '`' | '$' | '(' | ')' | '{' | '}' | '<' | '>' | '!' | '\\' | '\n' | '\r'))
}

fn is_command_allowed(cmd: &str) -> bool {
    let trimmed = cmd.trim();
    if contains_shell_metachar(trimmed) {
        return false;
    }
    CLUSTER_CMD_ALLOWLIST.iter().any(|allowed| {
        trimmed == *allowed || trimmed.starts_with(&format!("{} ", allowed))
    })
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

pub struct ClusterShellState {
    pub local_address: String,
    pub peer_terminal_ports: HashMap<String, u16>,
    pub node_id: u8,
}

pub type SharedClusterShell = Arc<Mutex<ClusterShellState>>;

pub fn new_cluster_shell(local_address: String, node_id: u8, peers: &[(String, u16)]) -> SharedClusterShell {
    let mut peer_terminal_ports = HashMap::new();
    for (addr, port) in peers {
        peer_terminal_ports.insert(addr.clone(), *port);
    }
    Arc::new(Mutex::new(ClusterShellState {
        local_address,
        peer_terminal_ports,
        node_id,
    }))
}

pub fn register_peer(shell: &SharedClusterShell, address: String, terminal_port: u16) {
    if let Ok(mut guard) = shell.lock() {
        guard.peer_terminal_ports.insert(address, terminal_port);
    }
}

pub fn remove_peer(shell: &SharedClusterShell, address: &str) {
    if let Ok(mut guard) = shell.lock() {
        guard.peer_terminal_ports.remove(address);
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

    let (peer_ports, _local_address) = {
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
    register_peer(&shell, req.address.clone(), req.terminal_port);
    Json(serde_json::json!({
        "ok": true,
        "registered": req.address,
        "terminal_port": req.terminal_port,
    }))
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterPeerRequest {
    pub address: String,
    pub terminal_port: u16,
}

pub fn cluster_shell_router(shell: SharedClusterShell) -> Router {
    Router::new()
        .route("/cluster/exec", post(handle_cluster_exec))
        .route("/cluster/peers", axum::routing::get(handle_cluster_peers))
        .route("/cluster/register-peer", post(handle_register_peer))
        .with_state(shell)
}

pub const CLUSTER_ROUTE_COUNT: usize = 3;
