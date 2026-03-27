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
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use serde::{Serialize, Deserialize};
use portable_pty::{CommandBuilder, PtySize, native_pty_system, MasterPty};
use tokio::sync::mpsc;

const CLUSTER_CMD_ALLOWLIST: &[&str] = &[
    "uname", "hostname", "uptime", "whoami", "date", "df", "free",
    "cat /proc/cpuinfo", "cat /proc/meminfo", "ip addr", "ifconfig",
    "systemctl status", "cargo --version", "rustc --version",
    "echo", "env", "printenv", "id", "ps aux",
];

fn contains_shell_metachar(cmd: &str) -> bool {
    cmd.chars().any(|c| matches!(c, ';' | '|' | '&' | '`' | '$' | '(' | ')' | '{' | '}' | '<' | '>' | '!' | '\\' | '\n' | '\r'))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub cols: u16,
    pub rows: u16,
    pub shell: String,
    pub cwd: String,
    pub env: HashMap<String, String>,
}

const ENV_ALLOWLIST: &[&str] = &[
    "PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "LC_CTYPE",
    "SHELL", "HOSTNAME", "CUBE_NODE_ID", "CUBE_MODE",
];

impl Default for SessionConfig {
    fn default() -> Self {
        let mut env = HashMap::new();
        for key in ENV_ALLOWLIST {
            if let Ok(val) = std::env::var(key) {
                env.insert(key.to_string(), val);
            }
        }
        Self {
            cols: 80,
            rows: 24,
            shell: std::env::var("SHELL").unwrap_or_else(|_| {
                if cfg!(target_os = "windows") {
                    std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
                } else {
                    "/bin/bash".to_string()
                }
            }),
            cwd: std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
            env,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: SessionId,
    pub cols: u16,
    pub rows: u16,
    pub created_at: u64,
    pub last_activity: u64,
    pub pid: u32,
}

struct LiveSession {
    info: SessionInfo,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    stdin_tx: mpsc::Sender<Vec<u8>>,
}

pub struct PtyMuxService {
    next_id: u64,
    sessions: HashMap<SessionId, LiveSession>,
    max_sessions: usize,
}

impl PtyMuxService {
    pub fn new(max_sessions: usize) -> Self {
        Self {
            next_id: 1,
            sessions: HashMap::new(),
            max_sessions,
        }
    }

    pub fn create_session(
        &mut self,
        config: SessionConfig,
        stdout_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<SessionId, String> {
        if self.sessions.len() >= self.max_sessions {
            return Err("Maximum session limit reached".to_string());
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: config.rows,
                cols: config.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(&config.shell);
        cmd.cwd(&config.cwd);
        for (k, v) in &config.env {
            cmd.env(k, v);
        }
        cmd.env("TERM", "xterm-256color");

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let pid = child.process_id().unwrap_or(0);

        let id = SessionId(self.next_id);
        self.next_id += 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let info = SessionInfo {
            id,
            cols: config.cols,
            rows: config.rows,
            created_at: now,
            last_activity: now,
            pid,
        };

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;

        let session_id_for_read = id;
        let stdout_tx_clone = stdout_tx.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = buf[..n].to_vec();
                        if stdout_tx_clone.blocking_send(data).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            println!(
                "[pty-mux] Reader thread ended for session {}",
                session_id_for_read.0
            );
        });

        let (stdin_tx, mut stdin_rx) = mpsc::channel::<Vec<u8>>(256);

        let mut writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take PTY writer: {}", e))?;

        std::thread::spawn(move || {
            loop {
                match stdin_rx.blocking_recv() {
                    Some(data) => {
                        if writer.write_all(&data).is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        });

        let session = LiveSession {
            info,
            master: pair.master,
            child,
            stdin_tx,
        };

        self.sessions.insert(id, session);
        println!(
            "[pty-mux] Session {} created (pid={}, shell={}, {}x{})",
            id.0, pid, config.shell, config.cols, config.rows
        );
        Ok(id)
    }

    pub fn write_to_session(&self, id: SessionId, data: Vec<u8>) -> Result<(), String> {
        let session = self
            .sessions
            .get(&id)
            .ok_or_else(|| format!("Session {} not found", id.0))?;

        session
            .stdin_tx
            .try_send(data)
            .map_err(|e| format!("Failed to write to session {}: {}", id.0, e))
    }

    pub fn resize_session(&mut self, id: SessionId, cols: u16, rows: u16) -> bool {
        if let Some(session) = self.sessions.get_mut(&id) {
            let result = session.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
            if result.is_ok() {
                session.info.cols = cols;
                session.info.rows = rows;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn destroy_session(&mut self, id: SessionId) -> bool {
        if let Some(mut session) = self.sessions.remove(&id) {
            let _ = session.child.kill();
            let _ = session.child.wait();
            drop(session.stdin_tx);
            println!("[pty-mux] Session {} destroyed (pid={})", id.0, session.info.pid);
            true
        } else {
            false
        }
    }

    pub fn get_session(&self, id: SessionId) -> Option<&SessionInfo> {
        self.sessions.get(&id).map(|s| &s.info)
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.values().map(|s| s.info.clone()).collect()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

pub type SharedPtyMux = Arc<Mutex<PtyMuxService>>;

pub fn new_shared_mux(max_sessions: usize) -> SharedPtyMux {
    Arc::new(Mutex::new(PtyMuxService::new(max_sessions)))
}

pub async fn run_ws_terminal_server(
    bind_addr: SocketAddr,
    mux: SharedPtyMux,
) {
    let listener = match tokio::net::TcpListener::bind(bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            println!("[pty-mux] Failed to bind terminal WS on {}: {}", bind_addr, e);
            return;
        }
    };

    println!("[pty-mux] Terminal WebSocket server listening on {}", bind_addr);

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("[pty-mux] Accept error: {}", e);
                continue;
            }
        };

        let mux = mux.clone();
        tokio::spawn(async move {
            let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    println!("[pty-mux] WebSocket handshake failed from {}: {}", peer, e);
                    return;
                }
            };

            println!("[pty-mux] WebSocket connection from {}", peer);
            handle_ws_session(ws_stream, mux, peer).await;
            println!("[pty-mux] WebSocket connection closed from {}", peer);
        });
    }
}

async fn handle_ws_session(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mux: SharedPtyMux,
    peer: SocketAddr,
) {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    let (stdout_tx, mut stdout_rx) = mpsc::channel::<Vec<u8>>(512);
    let (ctrl_tx, mut ctrl_rx) = mpsc::channel::<Vec<u8>>(64);

    let config = SessionConfig::default();
    let create_result = {
        let mut guard = mux.lock().unwrap_or_else(|e| e.into_inner());
        guard.create_session(config, stdout_tx)
    };
    let session_id = match create_result {
        Ok(id) => id,
        Err(e) => {
            println!("[pty-mux] Failed to create session for {}: {}", peer, e);
            let err_msg = serde_json::json!({"error": e});
            let _ = ws_tx
                .send(Message::Text(err_msg.to_string()))
                .await;
            return;
        }
    };

    let mux_write = mux.clone();
    let mux_cleanup = mux.clone();
    let sid = session_id;

    let write_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                data = stdout_rx.recv() => {
                    match data {
                        Some(bytes) => {
                            if ws_tx.send(Message::Binary(bytes)).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                ctrl = ctrl_rx.recv() => {
                    match ctrl {
                        Some(bytes) => {
                            if ws_tx.send(Message::Text(String::from_utf8_lossy(&bytes).to_string())).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let ws_resp_tx = ctrl_tx;
    let read_task = tokio::spawn(async move {
        while let Some(msg) = ws_rx.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    let _ = {
                        let guard = mux_write.lock().unwrap_or_else(|e| e.into_inner());
                        guard.write_to_session(sid, data)
                    };
                }
                Ok(Message::Text(text)) => {
                    if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                        if cmd["type"].as_str() == Some("resize") {
                            let cols = cmd["cols"].as_u64().unwrap_or(80) as u16;
                            let rows = cmd["rows"].as_u64().unwrap_or(24) as u16;
                            let mut guard =
                                mux_write.lock().unwrap_or_else(|e| e.into_inner());
                            guard.resize_session(sid, cols, rows);
                            drop(guard);
                        } else if cmd["type"].as_str() == Some("input") {
                            if let Some(data) = cmd["data"].as_str() {
                                let guard =
                                    mux_write.lock().unwrap_or_else(|e| e.into_inner());
                                let _ =
                                    guard.write_to_session(sid, data.as_bytes().to_vec());
                                drop(guard);
                            }
                        } else if cmd["type"].as_str() == Some("cluster_exec") {
                            if let Some(command) = cmd["command"].as_str() {
                                let allowed = !contains_shell_metachar(command.trim()) && CLUSTER_CMD_ALLOWLIST.iter().any(|a| {
                                    command.trim() == *a || command.trim().starts_with(&format!("{} ", a))
                                });
                                let (output, exit_code) = if !allowed {
                                    (format!("Command not allowed. Permitted: {}", CLUSTER_CMD_ALLOWLIST.join(", ")), -1i32)
                                } else {
                                    match if cfg!(target_os = "windows") {
                                        std::process::Command::new("cmd")
                                            .args(&["/C", command])
                                            .output()
                                    } else {
                                        std::process::Command::new("sh")
                                            .arg("-c")
                                            .arg(command)
                                            .output()
                                    }
                                    {
                                        Ok(out) => {
                                            let stdout = String::from_utf8_lossy(&out.stdout);
                                            let stderr = String::from_utf8_lossy(&out.stderr);
                                            (format!("{}{}", stdout, stderr), out.status.code().unwrap_or(-1))
                                        }
                                        Err(e) => (format!("exec error: {}", e), -1),
                                    }
                                };
                                let result_msg = serde_json::json!({
                                    "type": "cluster_result",
                                    "output": output,
                                    "exit_code": exit_code,
                                });
                                let _ = ws_resp_tx.send(result_msg.to_string().into_bytes()).await;
                            }
                        }
                    } else {
                        let _ = {
                            let guard = mux_write.lock().unwrap_or_else(|e| e.into_inner());
                            guard.write_to_session(sid, text.into_bytes())
                        };
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    let _ = tokio::select! {
        r = write_task => r,
        r = read_task => r,
    };

    {
        let mut guard = mux_cleanup.lock().unwrap_or_else(|e| e.into_inner());
        guard.destroy_session(session_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterCommand {
    pub command: String,
    pub targets: Vec<String>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResult {
    pub node: String,
    pub output: String,
    pub exit_code: i32,
    pub elapsed_ms: u64,
}

pub async fn fan_out_command(
    cmd: &ClusterCommand,
    peer_ports: &HashMap<String, u16>,
) -> Vec<ClusterResult> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let mut handles = Vec::new();

    for target in &cmd.targets {
        let port = match peer_ports.get(target) {
            Some(p) => *p,
            None => continue,
        };
        let target = target.clone();
        let command = cmd.command.clone();
        let timeout_ms = cmd.timeout_ms;

        handles.push(tokio::spawn(async move {
            let start = std::time::Instant::now();
            let url = format!("ws://{}:{}", target, port);
            let timeout = std::time::Duration::from_millis(timeout_ms);

            let connect_result =
                tokio::time::timeout(timeout, tokio_tungstenite::connect_async(&url)).await;

            match connect_result {
                Ok(Ok((mut ws, _))) => {
                    let input_msg = serde_json::json!({
                        "type": "cluster_exec",
                        "command": command,
                    });
                    let _ = ws.send(Message::Text(input_msg.to_string())).await;

                    let mut output = String::new();
                    let mut remote_exit_code: i32 = 0;
                    while let Some(Ok(msg)) = ws.next().await {
                        if msg.is_text() {
                            let text = msg.to_text().unwrap_or("");
                            if let Ok(resp) = serde_json::from_str::<serde_json::Value>(text) {
                                if resp["type"].as_str() == Some("cluster_result") {
                                    output = resp["output"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string();
                                    remote_exit_code = resp["exit_code"]
                                        .as_i64()
                                        .unwrap_or(0) as i32;
                                    break;
                                }
                            }
                        } else if msg.is_binary() {
                            output.push_str(&String::from_utf8_lossy(msg.into_data().as_ref()));
                        }
                    }
                    let _ = ws.close(None).await;

                    ClusterResult {
                        node: target,
                        output,
                        exit_code: remote_exit_code,
                        elapsed_ms: start.elapsed().as_millis() as u64,
                    }
                }
                _ => ClusterResult {
                    node: target,
                    output: "Connection failed or timed out".to_string(),
                    exit_code: -1,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                },
            }
        }));
    }

    let mut results = Vec::new();
    for h in handles {
        if let Ok(r) = h.await {
            results.push(r);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.cols, 80);
        assert_eq!(config.rows, 24);
        assert!(!config.shell.is_empty());
    }

    #[test]
    fn test_session_id_equality() {
        let a = SessionId(1);
        let b = SessionId(1);
        let c = SessionId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_cluster_command_serde() {
        let cmd = ClusterCommand {
            command: "uname -a".to_string(),
            targets: vec!["127.0.0.1".to_string()],
            timeout_ms: 5000,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: ClusterCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.command, "uname -a");
    }

    #[test]
    fn test_mux_service_creation() {
        let mux = PtyMuxService::new(10);
        assert_eq!(mux.session_count(), 0);
    }
}
