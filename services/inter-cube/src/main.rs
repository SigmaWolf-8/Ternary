// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumNET Inter-Cube Infrastructure Daemon v2.4.4
//
// MODES (controlled by CUBE_MODE env var):
//   "crs"     — Central Registration Service. Allocates addresses,
//               accepts registrations, serves full API.
//   "cube"    — Worker cube. Registers with a remote CRS on boot,
//               gets a unique address, heartbeats every 30s,
//               serves local stats API.
//   "all"     — Same as "crs" (backward compat).
//   "keygen"  — Generate PT26-DSA identity keypair and exit.
//   "hash"    — TIS-27 hash a file (set CUBE_HASH_TARGET). Exits.
//   "version" — Print daemon version and exit.
//
// GATEWAY ARCHITECTURE (1 outbound port per node, at center of 27-slot cube):
//   Node #1: gateway 11124 (center slot [2,2,2], 1 hop to all 26 neighbors)
//   Node #2: gateway 11151
//   Node #3: gateway 11178
//   Formula: gateway = 11111 + ((CUBE_NODE_ID - 1) × 27) + 13
//
// ENV VARS:
//   CUBE_MODE                  — "crs", "cube", "all", "keygen", "hash", or "version" (default: "all")
//   CUBE_CRS_URL               — CRS base URL (required for cube mode)
//   RELAY_URL                  — WebSocket relay URL (default: CUBE_CRS_URL)
//                                Set to remote relay (e.g. https://plenumnet.replit.app)
//                                when CUBE_CRS_URL points to a local CRS
//   LLM_PORT                   — Local LLM engine port for inference dispatch (default: CUBE_API_PORT + 1)
//   CUBE_ENDPOINT              — Wire protocol endpoint (default: "0.0.0.0:51820")
//   ADDRESS                    — Alias for CUBE_ENDPOINT
//   CUBE_ROLE                  — Role annotation (inference, review, kb, infra, relay, standby)
//   ROLE                       — Alias for CUBE_ROLE
//   CUBE_API_PORT              — HTTP API bind port (default: 11124 = gateway center)
//   API_PORT                   — Alias for CUBE_API_PORT
//   CUBE_PEER_PORT             — Direct peer-to-peer port (default: API_PORT - 1)
//   PEER_PORT                  — Alias for CUBE_PEER_PORT
//   CUBE_TERMINAL_PORT         — WebSocket PTY terminal port (default: API_PORT - 2)
//   CUBE_NODE_ID               — Array3 node ID, Rep C {1,2,3} (default: 1)
//   CUBE_ARRAY3_PEERS          — Comma-separated peer addresses for Array3 formation
//   CUBE_IDENTITY_DIR          — Directory for master.key (default: ~/.plenumnet/identity/)
//   CUBE_IDENTITY_PASSPHRASE   — Passphrase for master.key encryption
//   CUBE_CLUSTER_TOKEN         — Shared secret for cluster API auth (required for cluster routes)
//   CUBE_TERMINAL_BIND         — Terminal WebSocket bind address (default: 127.0.0.1)

use axum::response::Html;
use axum::routing::get as axum_get;
use tower_http::cors::{CorsLayer, Any};
use inter_cube::*;
use inter_cube::api::{
    AppState, crs_router, cube_router, parse_address_string, yoda_router,
    YodaRelaySender, YodaResponseWaiters,
    CRS_ROUTE_COUNT, CUBE_ROUTE_COUNT,
};
use inter_cube::daemon_identity::{DaemonIdentity, encryption_passphrase, identity_dir, save_master_secret};
use inter_cube::address_keys::derive_identity_keypair;
use inter_cube::key_rotation::RotationOrchestrator;
use inter_cube::ws_relay::WsRelayClient;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct TerminalSession {
    writer: Arc<tokio::sync::Mutex<Box<dyn std::io::Write + Send>>>,
    master: Arc<tokio::sync::Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

type TerminalSessions = Arc<Mutex<HashMap<String, TerminalSession>>>;

fn new_terminal_sessions() -> TerminalSessions {
    Arc::new(Mutex::new(HashMap::new()))
}

static CLI_PEER_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();

fn parse_cli_args() {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--peer-port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        let _ = CLI_PEER_PORT.set(p);
                        println!("[CLI] --peer-port {}", p);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            s if s.starts_with("--peer-port=") => {
                if let Ok(p) = s[12..].parse::<u16>() {
                    let _ = CLI_PEER_PORT.set(p);
                    println!("[CLI] --peer-port {}", p);
                }
                i += 1;
            }
            _ => { i += 1; }
        }
    }
}

fn env_or(primary: &str, alias: &str, default: &str) -> String {
    env::var(primary)
        .or_else(|_| env::var(alias))
        .unwrap_or_else(|_| default.to_string())
}

fn api_port() -> u16 {
    env::var("CUBE_API_PORT")
        .or_else(|_| env::var("API_PORT"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(11124)
}

fn peer_port() -> u16 {
    if let Some(p) = CLI_PEER_PORT.get() {
        return *p;
    }
    env::var("CUBE_PEER_PORT")
        .or_else(|_| env::var("PEER_PORT"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| api_port().saturating_sub(1))
}

fn role_label() -> Option<String> {
    env::var("CUBE_ROLE")
        .or_else(|_| env::var("ROLE"))
        .ok()
}

fn relay_url(crs_fallback: Option<&str>) -> Option<String> {
    env::var("RELAY_URL").ok().or_else(|| crs_fallback.map(|s| s.to_string()))
}

fn terminal_port() -> u16 {
    env::var("CUBE_TERMINAL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| api_port().saturating_sub(2))
}

fn cube_node_id() -> u8 {
    let id = env::var("CUBE_NODE_ID")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(1);
    if id == 0 || id > 3 {
        panic!("FATAL: CUBE_NODE_ID must be Rep C {{1,2,3}}, got {}. Zero is not a valid node identity.", id);
    }
    id
}

fn cube_array3_peers() -> Vec<String> {
    env::var("CUBE_ARRAY3_PEERS")
        .ok()
        .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}

#[derive(Clone, Debug)]
struct Array3NodeInfo {
    node_id: u8,
    port_start: u16,
    port_end: u16,
    wire_version: u8,
    slots_registered: u8,
    accepted: bool,
}

type PeerConnections = Arc<Mutex<HashMap<String, PeerInfo>>>;

#[derive(Clone, Debug)]
struct PeerInfo {
    address: String,
    ip: std::net::IpAddr,
    peer_port: u16,
    connected: bool,
    last_seen: u64,
}

fn new_peer_registry() -> PeerConnections {
    Arc::new(Mutex::new(HashMap::new()))
}

fn spawn_peer_listener(
    port: u16,
    local_address_dotted: String,
    peers: PeerConnections,
    peer_msg_tx: Option<tokio::sync::mpsc::Sender<inter_cube::ws_relay::RelayEnvelope>>,
    peer_senders: Option<PeerSenders>,
) {
    let peers_accept = peers.clone();
    tokio::spawn(async move {
        let listen_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
        match tokio::net::TcpListener::bind(listen_addr).await {
            Ok(listener) => {
                println!("[PEER] Direct peer listener active on port {}", port);
                println!("[PEER] LAN nodes can connect directly for sub-ms latency");
                loop {
                    match listener.accept().await {
                        Ok((stream, peer_addr)) => {
                            let addr = local_address_dotted.clone();
                            let peers_conn = peers_accept.clone();
                            let msg_tx = peer_msg_tx.clone();
                            let senders_for_conn = peer_senders.clone();
                            tokio::spawn(async move {
                                println!("[PEER] Connection from [unannounced-peer] (local address: {})", addr);
                                let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                                    Ok(ws) => ws,
                                    Err(e) => {
                                        println!("[PEER] WebSocket handshake failed from [unannounced-peer]: {}", e);
                                        return;
                                    }
                                };
                                use futures_util::{SinkExt, StreamExt};
                                let (write, mut read) = ws_stream.split();
                                let write_shared = Arc::new(tokio::sync::Mutex::new(write));
                                let mut inbound_peer_address: Option<String> = None;
                                while let Some(msg) = read.next().await {
                                    match msg {
                                        Ok(m) if m.is_text() => {
                                            let text = m.to_text().unwrap_or("");
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                                                if json["type"].as_str() == Some("peer_announce") {
                                                    let remote_addr = json["address"].as_str().unwrap_or("").to_string();
                                                    let remote_peer_port = json["peerPort"].as_u64().unwrap_or(0) as u16;
                                                    if !remote_addr.is_empty() && remote_peer_port > 0 {
                                                        let now = std::time::SystemTime::now()
                                                            .duration_since(std::time::UNIX_EPOCH)
                                                            .unwrap_or_default().as_secs();
                                                        if let Ok(mut peers) = peers_conn.lock() {
                                                            peers.insert(remote_addr.clone(), PeerInfo {
                                                                address: remote_addr.clone(),
                                                                ip: peer_addr.ip(),
                                                                peer_port: remote_peer_port,
                                                                connected: true,
                                                                last_seen: now,
                                                            });
                                                        }
                                                        if let Some(ref senders) = senders_for_conn {
                                                            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(64);
                                                            {
                                                                let mut sg = senders.lock().unwrap_or_else(|e| e.into_inner());
                                                                sg.insert(remote_addr.clone(), tx);
                                                            }
                                                            let ws_write = write_shared.clone();
                                                            tokio::spawn(async move {
                                                                while let Some(data) = rx.recv().await {
                                                                    let mut w = ws_write.lock().await;
                                                                    if w.send(tokio_tungstenite::tungstenite::Message::Text(data)).await.is_err() {
                                                                        break;
                                                                    }
                                                                }
                                                            });
                                                        }
                                                        inbound_peer_address = Some(remote_addr.clone());
                                                        println!("[PEER] Registered inbound LAN peer {} (sender ready)", remote_addr);
                                                    }
                                                } else if let Some(ref tx) = msg_tx {
                                                    let msg_type = json["type"].as_str().unwrap_or("relay").to_string();
                                                    let relay_msg_type = json["msgType"].as_str()
                                                        .or_else(|| json["relay_msg_type"].as_str())
                                                        .map(|s| s.to_string());
                                                    let from = json["from"].as_str().map(|s| s.to_string());
                                                    let payload = json["payload"].as_str().map(|s| s.to_string());
                                                    let envelope = inter_cube::ws_relay::RelayEnvelope {
                                                        msg_type,
                                                        relay_msg_type,
                                                        from,
                                                        payload,
                                                        to: json["to"].as_str().map(|s| s.to_string()),
                                                        address: json["address"].as_str().map(|s| s.to_string()),
                                                        public_key: None,
                                                        nonce: None,
                                                        signature: None,
                                                        error: None,
                                                        delivered: None,
                                                        connected_peers: None,
                                                        ts: None,
                                                        connected: None,
                                                    };
                                                    let rmt = envelope.relay_msg_type.as_deref().unwrap_or("unknown");
                                                    let peer_label = inbound_peer_address.as_deref().unwrap_or("[unannounced-peer]");
                                                    println!("[PEER] Routing {} from {} into processing pipeline", rmt, peer_label);
                                                    let _ = tx.send(envelope).await;
                                                } else {
                                                    let peer_label2 = inbound_peer_address.as_deref().unwrap_or("[unannounced-peer]");
                                                    println!("[PEER] Message from {}: {} bytes (no pipeline)", peer_label2, m.len());
                                                }
                                            } else {
                                                let peer_label3 = inbound_peer_address.as_deref().unwrap_or("[unannounced-peer]");
                                                println!("[PEER] Non-JSON from {}: {} bytes", peer_label3, m.len());
                                            }
                                        }
                                        Ok(m) if m.is_binary() => {
                                            let peer_label4 = inbound_peer_address.as_deref().unwrap_or("[unannounced-peer]");
                                            println!("[PEER] Binary from {}: {} bytes", peer_label4, m.len());
                                        }
                                        Ok(m) if m.is_close() => {
                                            let peer_label5 = inbound_peer_address.as_deref().unwrap_or("[unannounced-peer]");
                                            println!("[PEER] Peer {} disconnected", peer_label5);
                                            break;
                                        }
                                        Ok(_) => {}
                                        Err(e) => {
                                            let peer_label6 = inbound_peer_address.as_deref().unwrap_or("[unannounced-peer]");
                                            println!("[PEER] Error from {}: {}", peer_label6, e);
                                            break;
                                        }
                                    }
                                }
                                if let Some(ref peer_addr_str) = inbound_peer_address {
                                    if let Some(ref senders) = senders_for_conn {
                                        let mut sg = senders.lock().unwrap_or_else(|e| e.into_inner());
                                        sg.remove(peer_addr_str);
                                    }
                                    if let Ok(mut pg) = peers_conn.lock() {
                                        if let Some(info) = pg.get_mut(peer_addr_str) {
                                            info.connected = false;
                                        }
                                    }
                                    println!("[PEER] Cleaned up inbound peer sender for {}", peer_addr_str);
                                }
                            });
                        }
                        Err(e) => {
                            println!("[PEER] Accept error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("[PEER] Failed to bind peer port {}: {}", port, e);
                println!("[PEER] Direct peering disabled — relay-only mode");
            }
        }
    });
}

type PeerSenders = Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<String>>>>;

fn new_peer_senders() -> PeerSenders {
    Arc::new(Mutex::new(HashMap::new()))
}

fn spawn_peer_discovery(relay_url: String, local_address: String, local_peer_port: u16, peers: PeerConnections, senders: PeerSenders, peer_msg_tx: Option<tokio::sync::mpsc::Sender<inter_cube::ws_relay::RelayEnvelope>>, cluster_shell: Option<inter_cube::cluster_shell::SharedClusterShell>) {
    let discovery_base = relay_url
        .replace("wss://", "https://")
        .replace("ws://", "http://")
        .trim_end_matches('/')
        .replace("/ws/relay", "")
        .to_string();
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .user_agent(format!("PlenumNET-InterCube/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let discover_url = format!(
                "{}/api/salvi/inter-cube/relay/peer-discovery?address={}&peerPort={}",
                discovery_base, local_address, local_peer_port
            );

            match client.get(&discover_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(body) = resp.json::<serde_json::Value>().await {
                        if let Some(lan_peers) = body["peers"].as_array() {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs();
                            for peer in lan_peers {
                                let addr = peer["address"].as_str().unwrap_or("").to_string();
                                let ip_str = peer["ip"].as_str().unwrap_or("");
                                let pp = peer["peerPort"].as_u64().unwrap_or(0) as u16;
                                if !addr.is_empty() && pp > 0 && addr != local_address {
                                    if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
                                        let needs_connect = {
                                            let mut peers_guard = peers.lock().unwrap_or_else(|e| e.into_inner());
                                            let entry = peers_guard.entry(addr.clone()).or_insert(PeerInfo {
                                                address: addr.clone(),
                                                ip,
                                                peer_port: pp,
                                                connected: false,
                                                last_seen: now,
                                            });
                                            entry.last_seen = now;
                                            !entry.connected
                                        };
                                        if needs_connect {
                                            connect_to_peer(
                                                addr.clone(),
                                                ip,
                                                pp,
                                                local_address.clone(),
                                                local_peer_port,
                                                peers.clone(),
                                                senders.clone(),
                                                peer_msg_tx.clone(),
                                            );
                                        }
                                        if let Some(ref cs) = cluster_shell {
                                            let terminal_p = pp.saturating_sub(2);
                                            if terminal_p >= 11111 {
                                                println!("[cluster-shell] Auto-registered peer {} (terminal port={})", addr, terminal_p);
                                                inter_cube::cluster_shell::register_peer(
                                                    cs,
                                                    addr.clone(),
                                                    ip_str.to_string(),
                                                    terminal_p,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
    });
}

fn connect_to_peer(
    remote_address: String,
    ip: std::net::IpAddr,
    port: u16,
    local_address: String,
    local_peer_port: u16,
    peers: PeerConnections,
    senders: PeerSenders,
    peer_msg_tx: Option<tokio::sync::mpsc::Sender<inter_cube::ws_relay::RelayEnvelope>>,
) {
    tokio::spawn(async move {
        let ws_url = format!("ws://{}:{}", ip, port);
        println!("[PEER] Connecting to LAN peer {} at {}", remote_address, ws_url);

        let connect_result = tokio_tungstenite::connect_async(&ws_url).await;
        match connect_result {
            Ok((ws_stream, _)) => {
                println!("[PEER] Direct connection established to {}", remote_address);

                {
                    let mut peers_guard = peers.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(info) = peers_guard.get_mut(&remote_address) {
                        info.connected = true;
                        info.last_seen = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs();
                    }
                }

                use futures_util::{SinkExt, StreamExt};
                let (mut write, mut read) = ws_stream.split();

                let announce = serde_json::json!({
                    "type": "peer_announce",
                    "address": local_address,
                    "peerPort": local_peer_port,
                });
                let _ = write.send(tokio_tungstenite::tungstenite::Message::Text(announce.to_string())).await;

                let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(64);
                {
                    let mut senders_guard = senders.lock().unwrap_or_else(|e| e.into_inner());
                    senders_guard.insert(remote_address.clone(), tx);
                }

                let remote_addr_read = remote_address.clone();
                let peers_read = peers.clone();
                let msg_tx_read = peer_msg_tx.clone();
                let read_task = tokio::spawn(async move {
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(m) if m.is_text() => {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default().as_secs();
                                if let Ok(mut pg) = peers_read.lock() {
                                    if let Some(info) = pg.get_mut(&remote_addr_read) {
                                        info.last_seen = now;
                                    }
                                }
                                let text = m.to_text().unwrap_or("");
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                                    if json["type"].as_str() == Some("peer_announce") {
                                        println!("[PEER-DIRECT] Received peer_announce from {}", remote_addr_read);
                                    } else if let Some(ref tx) = msg_tx_read {
                                        let msg_type = json["type"].as_str().unwrap_or("relay").to_string();
                                        let relay_msg_type = json["msgType"].as_str()
                                            .or_else(|| json["relay_msg_type"].as_str())
                                            .map(|s| s.to_string());
                                        let from = json["from"].as_str().map(|s| s.to_string());
                                        let payload = json["payload"].as_str().map(|s| s.to_string());
                                        let envelope = inter_cube::ws_relay::RelayEnvelope {
                                            msg_type,
                                            relay_msg_type: relay_msg_type.clone(),
                                            from,
                                            payload,
                                            to: json["to"].as_str().map(|s| s.to_string()),
                                            address: json["address"].as_str().map(|s| s.to_string()),
                                            public_key: None,
                                            nonce: None,
                                            signature: None,
                                            error: None,
                                            delivered: None,
                                            connected_peers: None,
                                            ts: None,
                                            connected: None,
                                        };
                                        let rmt = relay_msg_type.as_deref().unwrap_or("unknown");
                                        println!("[PEER-DIRECT] Routing {} from {} into pipeline", rmt, remote_addr_read);
                                        let _ = tx.send(envelope).await;
                                    } else {
                                        println!("[PEER-DIRECT] Received {} bytes from {} (no pipeline)", m.len(), remote_addr_read);
                                    }
                                } else {
                                    println!("[PEER-DIRECT] Non-JSON from {}: {} bytes", remote_addr_read, m.len());
                                }
                            }
                            Ok(m) if m.is_binary() => {
                                println!("[PEER-DIRECT] Binary from {}: {} bytes", remote_addr_read, m.len());
                            }
                            Ok(m) if m.is_close() => break,
                            Err(_) => break,
                            _ => {}
                        }
                    }
                });

                let write_task = tokio::spawn(async move {
                    while let Some(data) = rx.recv().await {
                        if write.send(tokio_tungstenite::tungstenite::Message::Text(data)).await.is_err() {
                            break;
                        }
                    }
                });

                let _ = tokio::select! {
                    r = read_task => r,
                    r = write_task => r,
                };

                println!("[PEER] Direct connection to {} lost", remote_address);
                {
                    let mut peers_guard = peers.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(info) = peers_guard.get_mut(&remote_address) {
                        info.connected = false;
                    }
                }
                {
                    let mut senders_guard = senders.lock().unwrap_or_else(|e| e.into_inner());
                    senders_guard.remove(&remote_address);
                }
            }
            Err(e) => {
                println!("[PEER] Failed to connect to {} — {}", remote_address, e);
            }
        }
    });
}

async fn send_via_peer_or_relay(
    target: &str,
    payload: &str,
    peer_senders: &PeerSenders,
    relay_tx: Option<&tokio::sync::mpsc::Sender<inter_cube::ws_relay::RelayEnvelope>>,
    from: &str,
    msg_type: &str,
) -> bool {
    let peer_tx = {
        if let Ok(senders) = peer_senders.lock() {
            senders.get(target).cloned()
        } else {
            None
        }
    };

    if let Some(tx) = peer_tx {
        let msg = serde_json::json!({
            "type": "relay",
            "msgType": msg_type,
            "from": from,
            "to": target,
            "payload": payload,
        });
        if tx.send(msg.to_string()).await.is_ok() {
            println!("[ROUTE] Sent {} to {} via DIRECT peer", msg_type, target);
            return true;
        }
    }

    if let Some(relay) = relay_tx {
        let env = inter_cube::ws_relay::RelayEnvelope {
            msg_type: "relay".to_string(),
            to: Some(target.to_string()),
            relay_msg_type: Some(msg_type.to_string()),
            payload: Some(payload.to_string()),
            address: None,
            public_key: None,
            nonce: None,
            signature: None,
            from: None,
            error: None,
            delivered: None,
            connected_peers: None,
            ts: None,
            connected: None,
        };
        if relay.send(env).await.is_ok() {
            println!("[ROUTE] Sent {} to {} via RELAY (no direct peer)", msg_type, target);
            return true;
        }
    }

    println!("[ROUTE] Failed to send {} to {} — no route available", msg_type, target);
    false
}

// ═══════════════════════════════════════════════════════════════════════
// CRS MODE
// ═══════════════════════════════════════════════════════════════════════

async fn run_crs_mode() {
    let identity = DaemonIdentity::init();

    let mut crs = CubeRegistrationService::new();
    let endpoint: SocketAddr = "0.0.0.0:51820".parse().unwrap();
    let public_key = identity.pk_hex.as_bytes().to_vec();

    let registration = crs
        .register(endpoint, public_key, None)
        .expect("CRS: address allocation failed");

    println!("[CRS] Registered with address: {}", registration.address);
    println!("[CRS] PT26-DSA identity: {}...", &identity.pk_hex[..16]);

    let mut reg_count = 0;
    let mut unreg_count = 0;
    for nbr in &registration.neighbors {
        if nbr.endpoint.is_some() {
            reg_count += 1;
        } else {
            unreg_count += 1;
        }
    }
    println!(
        "[CRS] 26 neighbors: {} registered, {} awaiting",
        reg_count, unreg_count
    );

    let mut con = CubeOverlayNetwork::new(registration.address.clone());
    for nbr_info in &registration.neighbors {
        if let Some(ep) = nbr_info.endpoint {
            let pk = nbr_info.public_key.clone().unwrap_or_default();
            con.resolve_neighbor(&nbr_info.addr, ep, pk);
        }
    }
    let con_st = con.stats();
    println!(
        "[CON] Overlay: {} up, {} unknown",
        con_st.tunnels_up, con_st.tunnels_unknown
    );
    let keys = con.derive_all_keys(&std::collections::HashMap::new(), 0);
    println!(
        "[CON] {} PQ-native tunnel keys derived (TL-Sponge-385)",
        keys.len()
    );

    let fts = FaultToleranceService::new(registration.address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!(
        "[FTS] {} up, {} suspect, {} down, {} recovering",
        up, suspect, down, recovering
    );

    let glb = GeometricLoadBalancer::new(registration.address.clone());
    println!(
        "[GLB] {} live neighbors, ready to forward",
        glb.live_neighbor_count()
    );

    let local_address = registration.address.clone();

    let addr_bound_kp = derive_identity_keypair(&local_address, &identity.master_secret);
    let addr_bound_pk_hex: String = addr_bound_kp
        .public_key
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!(
        "[IDENTITY] Address-bound TL-DSA-87 key derived (pk: {}...)",
        &addr_bound_pk_hex[..16.min(addr_bound_pk_hex.len())]
    );

    match crs.update_public_key(&local_address, addr_bound_pk_hex.as_bytes().to_vec()) {
        Ok(()) => {
            println!("[IDENTITY] CRS registry updated with address-bound key");
        }
        Err(e) => {
            println!("[IDENTITY] WARNING: CRS key update failed: {:?}", e);
        }
    }

    let node_id = cube_node_id();
    let array3_peers = cube_array3_peers();

    println!();
    println!("=== Inter-Cube Stack Active ===");
    println!("  Address:       {} ({})", local_address.to_dotted(), local_address);
    println!(
        "  Address space: {} / {} used",
        crs.registered_count(),
        TOTAL_VERTICES
    );
    println!("  Dimensions:    {}", DIMENSIONS);
    println!("  Neighbors:     {}", NEIGHBORS_PER_CUBE);
    println!("  Protocol:      PQ-Native (PT26-DSA + TL-Sponge-385)");
    println!("  Wire:          V{} (min V{})", inter_cube::wire::PROTOCOL_VERSION_CURRENT, inter_cube::wire::PROTOCOL_VERSION_MIN);
    println!("  Identity:      PT26-DSA (71-byte sigs, 28-sig budget)");
    println!("  Node ID:       {} (Rep C)", node_id);
    if !array3_peers.is_empty() {
        println!("  Array3 peers:  {}", array3_peers.join(", "));
    }
    println!();
    println!("  CRS -> CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    let shared_state = AppState::new_crs(crs, con, fts, glb, local_address.clone());
    shared_state.daemon_config.log_startup();

    let vm = inter_cube::vm_service::new_shared_vm(65536);
    let vm_routes = inter_cube::vm_service::vm_router(vm);

    let nid = cube_node_id();
    let a3_peers: Vec<(String, String, u16)> = cube_array3_peers()
        .iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.splitn(2, ':').collect();
            if parts.len() == 2 {
                parts[1].parse::<u16>().ok().map(|port| (parts[0].to_string(), parts[0].to_string(), port))
            } else {
                None
            }
        })
        .collect();
    let cluster_shell = inter_cube::cluster_shell::new_cluster_shell(local_address.to_dotted(), nid, &a3_peers);

    let addr_str: String = local_address.to_bytes().iter().map(|t| t.to_string()).collect();
    let crs_yoda_verifier: inter_cube::api::SharedYodaVerifier = std::sync::Arc::new(
        tokio::sync::Mutex::new(
            inter_cube::yoda_chat::YodaChatVerifier::new(addr_str.clone())
        )
    );
    let crs_yoda_session_origins: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>> =
        std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    let crs_yoda_relay_tx: YodaRelaySender = std::sync::Arc::new(tokio::sync::Mutex::new(None));
    let crs_yoda_waiters: YodaResponseWaiters = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    let port = api_port();
    let p_port = peer_port();
    let t_port = terminal_port();
    let slots_bind = shared_state.daemon_config.bind_addr.clone();

    let monitor_route = axum::Router::new()
        .route("/monitor", axum_get(|| async {
            Html(include_str!("../monitor/array3-monitor-v8.html"))
        }));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = crs_router(shared_state)
        .merge(vm_routes)
        .merge(inter_cube::cluster_shell::cluster_shell_router(cluster_shell.clone()))
        .merge(yoda_router(crs_yoda_verifier.clone(), crs_yoda_relay_tx.clone(), crs_yoda_waiters.clone()))
        .merge(monitor_route)
        .layer(cors);
    let listen_addr: SocketAddr = format!("{}:{}", slots_bind, port).parse().unwrap_or_else(|_| {
        eprintln!("[CRS] WARNING: Invalid PLENUM_BIND_ADDR '{}', falling back to 127.0.0.1", slots_bind);
        format!("127.0.0.1:{}", port).parse().unwrap()
    });
    if let Some(role) = role_label() {
        println!("  Role:          {}", role);
    }

    if let Some(rurl) = relay_url(None) {
        println!("  Relay:         {} (WebSocket, TL-DSA-87 challenge-response)", rurl);
        let relay_kp = derive_identity_keypair(&local_address, &identity.master_secret);
        let tl_dsa_pk_hex: String = relay_kp.public_key.iter().map(|b| format!("{:02x}", b)).collect();
        let crs_relay_url = rurl.clone();
        spawn_relay_client(rurl, addr_str.clone(), identity.pk_hex.clone(), relay_kp.secret_key.clone(), tl_dsa_pk_hex, None, None, new_terminal_sessions(), crs_yoda_verifier, crs_yoda_session_origins, crs_yoda_relay_tx, crs_yoda_waiters);
        spawn_peer_discovery(crs_relay_url, addr_str, p_port, new_peer_registry(), new_peer_senders(), None, Some(cluster_shell.clone()));
    } else {
        println!("  Relay:         none (set RELAY_URL to enable remote relay)");
    }

    let peers = new_peer_registry();
    spawn_peer_listener(p_port, local_address.to_dotted(), peers.clone(), None, None);

    {
        let relay_base = relay_url(None).unwrap_or_else(|| "https://plenumnet.replit.app".to_string());
        let crs_api_port = port;
        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent(format!("PlenumNET-InterCube/{}", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("slot-report HTTP client");
            let node_id: u8 = std::env::var("CUBE_NODE_ID")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(1);
            let slots_url = format!("http://127.0.0.1:{}/api/salvi/inter-cube/slots", crs_api_port);
            let health_url = format!("http://127.0.0.1:{}/health", crs_api_port);
            let report_url = format!("{}/api/salvi/inter-cube/relay/slot-report", relay_base);
            tokio::time::sleep(Duration::from_secs(5)).await;
            loop {
                let slots_resp = client.get(&slots_url).send().await;
                let health_resp = client.get(&health_url).send().await;
                if let (Ok(sr), Ok(hr)) = (slots_resp, health_resp) {
                    if sr.status().is_success() && hr.status().is_success() {
                        if let (Ok(sj), Ok(hj)) = (
                            sr.json::<serde_json::Value>().await,
                            hr.json::<serde_json::Value>().await,
                        ) {
                            let _ = client.post(&report_url)
                                .json(&serde_json::json!({"nodeId": node_id, "slots": sj, "health": hj}))
                                .send().await;
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
    }

    let terminal_mux = pty_mux::new_shared_mux(16);
    let term_bind_addr = env::var("CUBE_TERMINAL_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let terminal_bind: SocketAddr = format!("{}:{}", term_bind_addr, t_port).parse().unwrap();
    tokio::spawn(pty_mux::run_ws_terminal_server(terminal_bind, terminal_mux));

    println!();
    println!("=== HTTP Server (CRS) ===");
    println!("  http://{}", listen_addr);
    println!("  Peer port:     {}", p_port);
    println!("  Terminal port: {} (WebSocket PTY)", t_port);
    println!("  {} API + {} VM + {} cluster routes active",
        CRS_ROUTE_COUNT,
        inter_cube::vm_service::VM_ROUTE_COUNT,
        inter_cube::cluster_shell::CLUSTER_ROUTE_COUNT,
    );
    println!("  Ready for cube registrations. Ctrl+C to stop.");
    println!();

    let listener = match tokio::net::TcpListener::bind(listen_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "[CRS] FATAL: Cannot bind to {} — {}. \
                 Check that the port is not already in use and that you have permission to bind.",
                listen_addr, e
            );
            std::process::exit(1);
        }
    };

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
        .await
        .expect("Server error");
}

// ═══════════════════════════════════════════════════════════════════════
// TERMINAL AUTHORIZATION
// ═══════════════════════════════════════════════════════════════════════

async fn is_terminal_authorized(
    from: &str,
    known_peers: &Arc<tokio::sync::Mutex<Vec<String>>>,
) -> (bool, String) {
    if from == "?" || from.is_empty() {
        return (false, "terminal command from unauthenticated source".to_string());
    }
    let is_known = {
        let peers = known_peers.lock().await;
        peers.contains(&from.to_string())
    };
    if !is_known {
        return (false, format!("terminal command from unknown peer {}", from));
    }
    (true, String::new())
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE MODE
// ═══════════════════════════════════════════════════════════════════════

async fn run_cube_mode() {
    let crs_url = env::var("CUBE_CRS_URL").expect("CUBE_CRS_URL is required for cube mode");
    let cube_endpoint = env_or("CUBE_ENDPOINT", "ADDRESS", "0.0.0.0:51820");
    let role = role_label();

    println!("[CUBE] Mode: worker cube");
    println!("[CUBE] CRS URL: {}", crs_url);
    println!("[CUBE] Endpoint: {}", cube_endpoint);
    if let Some(ref r) = role {
        println!("[CUBE] Role: {}", r);
    }
    println!();

    let identity = DaemonIdentity::init();
    let key_hex = identity.pk_hex.clone();

    let resolved_endpoint = match tokio::net::lookup_host(&cube_endpoint).await {
        Ok(mut addrs) => match addrs.next() {
            Some(sa) => {
                println!("[CUBE] Resolved {} -> {}", cube_endpoint, sa);
                sa.to_string()
            }
            None => {
                println!(
                    "[CUBE] WARNING: DNS returned no results for {}, using 0.0.0.0:51820",
                    cube_endpoint
                );
                "0.0.0.0:51820".to_string()
            }
        },
        Err(e) => {
            println!(
                "[CUBE] WARNING: DNS lookup failed for {}: {}, using 0.0.0.0:51820",
                cube_endpoint, e
            );
            "0.0.0.0:51820".to_string()
        }
    };

    let client = reqwest::Client::builder()
        .user_agent(format!("PlenumNET-InterCube/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to build HTTP client");
    let register_url = format!("{}/api/salvi/inter-cube/crs/register", crs_url);

    let mut response_body: Option<serde_json::Value> = None;
    for attempt in 1..=10 {
        println!(
            "[CUBE] Registration attempt {}/10 -> {}",
            attempt, register_url
        );

        let result = client
            .post(&register_url)
            .json(&serde_json::json!({
                "endpoint": resolved_endpoint,
                "publicKey": key_hex,
            }))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        response_body = Some(body);
                        break;
                    }
                    Err(e) => {
                        println!("[CUBE] Failed to parse response: {}", e);
                    }
                }
            }
            Ok(resp) => {
                println!("[CUBE] CRS returned {}, retrying...", resp.status());
            }
            Err(e) => {
                println!("[CUBE] Connection failed: {}, retrying...", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let reg_data = response_body.expect("Failed to register with CRS after 10 attempts");

    let addr_str = reg_data["address"]
        .as_str()
        .expect("CRS response missing 'address' field");
    let local_address =
        parse_address_string(addr_str).expect("CRS returned invalid address");

    let registered_nbrs = reg_data["registeredNeighbors"].as_u64().unwrap_or(0);
    let total_nbrs = reg_data["totalNeighbors"].as_u64().unwrap_or(26);

    println!();
    println!("[CUBE] Registered! Address: {} ({})", local_address.to_dotted(), local_address);
    println!(
        "[CUBE] Neighbors: {} registered, {} total",
        registered_nbrs, total_nbrs
    );

    let addr_bound_kp = derive_identity_keypair(&local_address, &identity.master_secret);
    let addr_bound_pk_hex: String = addr_bound_kp
        .public_key
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!(
        "[IDENTITY] Address-bound TL-DSA-87 key derived (pk: {}...)",
        &addr_bound_pk_hex[..16.min(addr_bound_pk_hex.len())]
    );

    let addr_trits_vec: Vec<u8> = local_address.to_bytes().to_vec();
    let update_key_url = format!("{}/api/salvi/inter-cube/crs/update-key", crs_url);
    let reregister_result = client
        .post(&update_key_url)
        .json(&serde_json::json!({
            "address": addr_trits_vec,
            "publicKey": addr_bound_pk_hex,
        }))
        .send()
        .await;

    match reregister_result {
        Ok(resp) if resp.status().is_success() => {
            println!("[IDENTITY] Updated CRS with address-bound public key");
        }
        Ok(resp) => {
            println!(
                "[IDENTITY] WARNING: Key update returned {}, continuing with initial key",
                resp.status()
            );
        }
        Err(e) => {
            println!(
                "[IDENTITY] WARNING: Key update failed: {}, continuing with initial key",
                e
            );
        }
    }

    let mut con = CubeOverlayNetwork::new(local_address.clone());

    if let Some(neighbors) = reg_data["neighbors"].as_array() {
        for nbr in neighbors {
            let registered = nbr["registered"].as_bool().unwrap_or(false);
            if registered {
                if let (Some(addr_s), Some(ep_s)) =
                    (nbr["address"].as_str(), nbr["endpoint"].as_str())
                {
                    if let (Some(nbr_addr), Ok(nbr_ep)) =
                        (parse_address_string(addr_s), ep_s.parse::<SocketAddr>())
                    {
                        con.resolve_neighbor(&nbr_addr, nbr_ep, vec![0u8; 32]);
                        println!("[CON] Resolved neighbor: {} at {}", nbr_addr, nbr_ep);
                    }
                }
            }
        }
    }

    let con_st = con.stats();
    println!(
        "[CON] Overlay: {} up, {} resolving, {} unknown",
        con_st.tunnels_up, con_st.tunnels_resolving, con_st.tunnels_unknown
    );
    let keys = con.derive_all_keys(&std::collections::HashMap::new(), 0);
    println!(
        "[CON] {} PQ-native tunnel keys derived (TL-Sponge-385)",
        keys.len()
    );

    let fts = FaultToleranceService::new(local_address.clone());
    let (up, suspect, down, recovering) = fts.state_counts();
    println!(
        "[FTS] {} up, {} suspect, {} down, {} recovering",
        up, suspect, down, recovering
    );

    let glb = GeometricLoadBalancer::new(local_address.clone());
    println!(
        "[GLB] {} live neighbors, ready to forward",
        glb.live_neighbor_count()
    );

    println!();
    println!("=== Inter-Cube Stack Active (Cube Mode) ===");
    println!("  Address:       {} ({})", local_address.to_dotted(), local_address);
    println!("  CRS:           {}", crs_url);
    println!("  Dimensions:    {}", DIMENSIONS);
    println!(
        "  Neighbors:     {} ({} registered)",
        NEIGHBORS_PER_CUBE, registered_nbrs
    );
    println!("  Protocol:      PQ-Native (PT26-DSA + TL-Sponge-385)");
    println!("  Identity:      PT26-DSA (71-byte sigs, 28-sig budget)");
    println!();
    println!("  CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    let registration_timestamp = {
        let ts_path = inter_cube::daemon_identity::identity_dir().join("registration_ts");
        if let Ok(contents) = std::fs::read_to_string(&ts_path) {
            if let Ok(ts) = contents.trim().parse::<u64>() {
                println!("[IDENTITY] Loaded persisted registration timestamp: {}", ts);
                ts
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let _ = std::fs::write(&ts_path, now.to_string());
                println!("[IDENTITY] Persisted new registration timestamp: {}", now);
                now
            }
        } else {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = std::fs::write(&ts_path, now.to_string());
            println!("[IDENTITY] Persisted new registration timestamp: {}", now);
            now
        }
    };
    let mut passphrase = encryption_passphrase();
    let passphrase_hb = passphrase.clone();
    let orchestrator = RotationOrchestrator::new(
        local_address.clone(),
        registration_timestamp,
        identity.master_secret.clone(),
        identity.current_radian_epoch(),
        passphrase.clone(),
    );
    passphrase.iter_mut().for_each(|b| unsafe { std::ptr::write_volatile(b as *mut u8, 0) });
    drop(passphrase);
    let orchestrator = Arc::new(Mutex::new(orchestrator));

    let crs_url_for_heartbeat = crs_url.clone();
    let endpoint_for_heartbeat = cube_endpoint.clone();
    let addr_trits: Vec<u8> = local_address.to_bytes().to_vec();
    let orchestrator_hb = orchestrator.clone();
    let local_addr_hb = local_address.clone();
    let key_hex_hb = key_hex.clone();

    tokio::spawn(async move {
        let hb_client = reqwest::Client::builder()
            .user_agent(format!("PlenumNET-InterCube/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to build HTTP client");
        let hb_url_base = format!(
            "{}/api/salvi/inter-cube/relay/heartbeat",
            crs_url_for_heartbeat
        );
        let update_key_url = format!(
            "{}/api/salvi/inter-cube/crs/update-key",
            crs_url_for_heartbeat
        );
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let unix_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let rotation_result = {
                if let Ok(mut orch) = orchestrator_hb.lock() {
                    match orch.check_and_rotate(unix_now) {
                        Ok(Some(event)) => {
                            println!(
                                "[ROTATION] Key rotated: epoch {} -> {} (forced={})",
                                event.old_epoch, event.new_epoch, event.forced
                            );

                            let new_kp = derive_identity_keypair(
                                &local_addr_hb,
                                orch.current_secret(),
                            );
                            let new_pk_hex: String = new_kp
                                .public_key
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect();

                            let dir = identity_dir();
                            let key_path = dir.join("master.key");
                            save_master_secret(
                                orch.current_secret(),
                                &passphrase_hb,
                                &dir,
                                &key_path,
                            );

                            Some(new_pk_hex)
                        }
                        Ok(None) => None,
                        Err(e) => {
                            println!("[ROTATION] Check failed: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(new_pk_hex) = rotation_result {
                let rereg_result = hb_client
                    .post(&update_key_url)
                    .json(&serde_json::json!({
                        "address": addr_trits,
                        "publicKey": new_pk_hex,
                    }))
                    .send()
                    .await;

                match rereg_result {
                    Ok(resp) if resp.status().is_success() => {
                        println!("[ROTATION] Re-registered with rotated key");
                        if let Ok(mut orch) = orchestrator_hb.lock() {
                            orch.reregistration_complete();
                        }
                    }
                    Ok(resp) => {
                        println!(
                            "[ROTATION] WARNING: Re-registration returned {}",
                            resp.status()
                        );
                    }
                    Err(e) => {
                        println!("[ROTATION] WARNING: Re-registration failed: {}", e);
                    }
                }
            }

            let addr_str_hb: String = addr_trits.iter().map(|t| t.to_string()).collect();
            let result = hb_client
                .post(&hb_url_base)
                .json(&serde_json::json!({
                    "address": addr_str_hb,
                    "publicKey": key_hex_hb,
                }))
                .send()
                .await;

            match result {
                Ok(resp) if resp.status().is_success() => {}
                Ok(resp) => {
                    println!("[HEARTBEAT] CRS returned {}", resp.status());
                }
                Err(e) => {
                    println!("[HEARTBEAT] Failed: {}", e);
                }
            }

            let node_id: u8 = std::env::var("CUBE_NODE_ID")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(1);
            let api_port: u16 = std::env::var("CUBE_API_PORT")
                .or_else(|_| std::env::var("API_PORT"))
                .ok().and_then(|v| v.parse().ok()).unwrap_or(11124);
            let slots_url = format!("http://127.0.0.1:{}/api/salvi/inter-cube/slots", api_port);
            let health_url = format!("http://127.0.0.1:{}/health", api_port);
            let report_url = format!("{}/api/salvi/inter-cube/relay/slot-report", crs_url_for_heartbeat);

            let slots_resp = hb_client.get(&slots_url).send().await;
            let health_resp = hb_client.get(&health_url).send().await;

            if let (Ok(sr), Ok(hr)) = (slots_resp, health_resp) {
                if sr.status().is_success() && hr.status().is_success() {
                    if let (Ok(slots_json), Ok(health_json)) = (
                        sr.json::<serde_json::Value>().await,
                        hr.json::<serde_json::Value>().await,
                    ) {
                        let _ = hb_client.post(&report_url)
                            .json(&serde_json::json!({
                                "nodeId": node_id,
                                "slots": slots_json,
                                "health": health_json,
                            }))
                            .send()
                            .await;
                    }
                }
            }
        }
    });

    let addr_str_for_relay: String = local_address.to_bytes().iter().map(|t| t.to_string()).collect();
    let relay_target = relay_url(Some(&crs_url)).unwrap_or_else(|| crs_url.clone());
    let p_port = peer_port();
    let peers = new_peer_registry();
    let peer_senders = new_peer_senders();

    let yoda_verifier: inter_cube::api::SharedYodaVerifier = std::sync::Arc::new(
        tokio::sync::Mutex::new(
            inter_cube::yoda_chat::YodaChatVerifier::new(addr_str_for_relay.clone())
        )
    );
    let yoda_session_origins: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>> =
        std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    let yoda_relay_tx: YodaRelaySender = std::sync::Arc::new(tokio::sync::Mutex::new(None));
    let yoda_waiters: YodaResponseWaiters = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    println!("[ws-relay] Relay target: {}", relay_target);
    let relay_kp = derive_identity_keypair(&local_address, &identity.master_secret);
    let relay_tl_dsa_pk_hex: String = relay_kp.public_key.iter().map(|b| format!("{:02x}", b)).collect();
    let (peer_msg_tx, peer_msg_rx) = tokio::sync::mpsc::channel::<inter_cube::ws_relay::RelayEnvelope>(64);
    let relay_target_for_discovery = relay_target.clone();
    spawn_relay_client(relay_target, addr_str_for_relay, key_hex.clone(), relay_kp.secret_key.clone(), relay_tl_dsa_pk_hex, Some(peer_senders.clone()), Some(peer_msg_rx), new_terminal_sessions(), yoda_verifier.clone(), yoda_session_origins.clone(), yoda_relay_tx.clone(), yoda_waiters.clone());

    let peer_msg_tx_discovery = peer_msg_tx.clone();
    spawn_peer_listener(p_port, local_address.to_dotted(), peers.clone(), Some(peer_msg_tx), Some(peer_senders.clone()));

    let shared_state = AppState::new_cube(con, fts, glb, local_address.clone());
    shared_state.daemon_config.log_startup();

    let vm = inter_cube::vm_service::new_shared_vm(65536);
    let vm_routes = inter_cube::vm_service::vm_router(vm);

    let nid = cube_node_id();
    let a3_peers_cube: Vec<(String, String, u16)> = cube_array3_peers()
        .iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.splitn(2, ':').collect();
            if parts.len() == 2 {
                parts[1].parse::<u16>().ok().map(|port| (parts[0].to_string(), parts[0].to_string(), port))
            } else {
                None
            }
        })
        .collect();
    let cluster_shell = inter_cube::cluster_shell::new_cluster_shell(local_address.to_dotted(), nid, &a3_peers_cube);

    let addr_str_for_discovery: String = local_address.to_bytes().iter().map(|t| t.to_string()).collect();
    spawn_peer_discovery(relay_target_for_discovery, addr_str_for_discovery, p_port, peers.clone(), peer_senders.clone(), Some(peer_msg_tx_discovery), Some(cluster_shell.clone()));

    let cluster_routes = inter_cube::cluster_shell::cluster_shell_router(cluster_shell);

    let port = api_port();
    let t_port = terminal_port();
    let slots_bind = shared_state.daemon_config.bind_addr.clone();

    let monitor_route = axum::Router::new()
        .route("/monitor", axum_get(|| async {
            Html(include_str!("../monitor/array3-monitor-v8.html"))
        }));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = cube_router(shared_state)
        .merge(vm_routes)
        .merge(cluster_routes)
        .merge(yoda_router(yoda_verifier, yoda_relay_tx, yoda_waiters))
        .merge(monitor_route)
        .layer(cors);
    let listen_addr: SocketAddr = format!("{}:{}", slots_bind, port).parse().unwrap_or_else(|_| {
        eprintln!("[CUBE] WARNING: Invalid PLENUM_BIND_ADDR '{}', falling back to 127.0.0.1", slots_bind);
        format!("127.0.0.1:{}", port).parse().unwrap()
    });

    let terminal_mux = pty_mux::new_shared_mux(16);
    let term_bind_addr = env::var("CUBE_TERMINAL_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    if term_bind_addr != "127.0.0.1" && term_bind_addr != "localhost" && term_bind_addr != "::1" {
        eprintln!("[TERMINAL] WARNING: Terminal WebSocket bound to non-loopback address {}. This exposes the PTY to the network. Ensure firewall rules restrict access.", term_bind_addr);
    }
    let terminal_bind: SocketAddr = format!("{}:{}", term_bind_addr, t_port).parse().unwrap();
    tokio::spawn(pty_mux::run_ws_terminal_server(terminal_bind, terminal_mux));

    println!();
    println!("=== HTTP Server (Cube) ===");
    println!("  http://{}", listen_addr);
    println!("  Peer port:     {}", p_port);
    println!("  Terminal port: {} (WebSocket PTY)", t_port);
    println!("  {} API + {} VM + {} cluster routes active",
        CUBE_ROUTE_COUNT,
        inter_cube::vm_service::VM_ROUTE_COUNT,
        inter_cube::cluster_shell::CLUSTER_ROUTE_COUNT,
    );
    println!("  Routing:   Direct peer (LAN) -> Relay (WAN) fallback");
    println!("  Heartbeat every 30s to CRS (with key rotation check). Ctrl+C to stop.");
    println!();

    let listener = match tokio::net::TcpListener::bind(listen_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "[CUBE] FATAL: Cannot bind to {} — {}. \
                 Check that the port is not already in use and that you have permission to bind.",
                listen_addr, e
            );
            std::process::exit(1);
        }
    };

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
        .await
        .expect("Server error");
}

fn spawn_relay_client(
    relay_url_str: String,
    address: String,
    public_key_hex: String,
    tl_dsa_sk: Vec<u8>,
    tl_dsa_pk_hex: String,
    peer_senders: Option<PeerSenders>,
    peer_msg_rx: Option<tokio::sync::mpsc::Receiver<inter_cube::ws_relay::RelayEnvelope>>,
    terminal_sessions: TerminalSessions,
    yoda_verifier: inter_cube::api::SharedYodaVerifier,
    yoda_session_origins: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>,
    yoda_relay_tx: YodaRelaySender,
    yoda_response_waiters: YodaResponseWaiters,
) {
    let llm_port = env::var("LLM_PORT").unwrap_or_else(|_| format!("{}", api_port() + 1));
    let llm_base_url = format!("http://127.0.0.1:{}", llm_port);
    let api_port_val = api_port();
    let endpoint_str = format!("0.0.0.0:{}", api_port_val);
    let ops_address = address.clone();
    tokio::spawn(async move {
        let mut peer_msg_rx = peer_msg_rx;
        println!();
        println!("[ws-relay] Establishing relay connection to {}...", relay_url_str);
        println!("[ws-relay] Inference dispatch target: {}/v1/chat/completions", llm_base_url);

        let ops_base_dir = std::path::PathBuf::from(
            env::var("PLENUMNET_BASE_DIR").unwrap_or_else(|_| ".".to_string())
        );
        let ops_handler = std::sync::Arc::new(
            inter_cube::ops_handler::OpsHandler::new(ops_address.clone(), ops_base_dir.clone())
        );
        let ops_config_path = ops_base_dir.join(".plenumnet/ops-config.json");
        ops_handler.load_config(&ops_config_path).await;
        ops_handler.load_persisted_transfers().await;
        println!("[ops] Operations handler initialized for node {}", ops_address);

        {
            let ops_operators = ops_handler.get_operators().await;
            let mut yv = yoda_verifier.lock().await;
            yv.set_operators(ops_operators.clone());
            println!("[yoda-chat] Verifier initialized for daemon {} with {} authorized operators", address, ops_operators.len());
        }
        let mut retry_delay = Duration::from_secs(5);
        let inference_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent(format!("PlenumNET-InterCube/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to build inference HTTP client");
        loop {
            let http_base = relay_url_str
                .replace("wss://", "https://")
                .replace("ws://", "http://")
                .trim_end_matches('/')
                .replace("/ws/relay", "")
                .to_string();
            let reg_url = format!(
                "{}/api/salvi/inter-cube/relay/register",
                http_base,
            );
            let reg_client = reqwest::Client::new();
            match reg_client.post(&reg_url).json(&serde_json::json!({
                "publicKey": &public_key_hex,
                "endpoint": &endpoint_str,
                "tlDsaPk": &tl_dsa_pk_hex,
            })).send().await {
                Ok(resp) if resp.status().is_success() => {
                    println!("[ws-relay] Pre-registered with relay (address: {})", address);
                }
                Ok(resp) => {
                    println!("[ws-relay] Relay pre-registration returned {}", resp.status());
                }
                Err(e) => {
                    println!("[ws-relay] Relay pre-registration failed: {} — will retry", e);
                }
            }
            match WsRelayClient::connect_signed(
                &relay_url_str,
                &address,
                &public_key_hex,
                Some(&tl_dsa_sk),
            )
            .await
            {
                Ok((client, mut incoming_rx)) => {
                    println!("[ws-relay] Relay tunnel active — NAT traversal established");
                    retry_delay = Duration::from_secs(5);

                    {
                        let mut yrt = yoda_relay_tx.lock().await;
                        *yrt = Some(client.outgoing_tx.clone());
                    }

                    let (ops_ws_tx, mut ops_ws_rx) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
                    ops_handler.set_ws_sender(ops_ws_tx).await;
                    let ops_stream_tx = client.outgoing_tx.clone();
                    tokio::spawn(async move {
                        while let Some(msg) = ops_ws_rx.recv().await {
                            let msg_type = msg.get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("ops-data")
                                .to_string();
                            let env = inter_cube::ws_relay::RelayEnvelope {
                                msg_type: "relay".to_string(),
                                to: Some("coordinator".to_string()),
                                relay_msg_type: Some(msg_type),
                                payload: Some(msg.to_string()),
                                address: None, public_key: None, nonce: None,
                                signature: None, from: None, error: None,
                                delivered: None, connected_peers: None,
                                ts: None, connected: None,
                            };
                            if let Err(e) = ops_stream_tx.send(env).await {
                                println!("[ops] ws_sender relay forward failed: {}", e);
                                break;
                            }
                        }
                    });

                    let ops_telem_handler = ops_handler.clone();
                    let ops_telem_tx = client.outgoing_tx.clone();
                    let ops_telem_connected = client.connected.clone();
                    tokio::spawn(async move {
                        let mut interval = tokio::time::interval(Duration::from_secs(60));
                        interval.tick().await;
                        loop {
                            interval.tick().await;
                            if !*ops_telem_connected.lock().await { break; }
                            if !ops_telem_handler.is_enabled().await { continue; }
                            let telemetry = ops_telem_handler.collect_telemetry().await;
                            let env = inter_cube::ws_relay::RelayEnvelope {
                                msg_type: "relay".to_string(),
                                to: None,
                                relay_msg_type: Some("telemetry".to_string()),
                                payload: Some(telemetry.to_string()),
                                address: None, public_key: None, nonce: None,
                                signature: None, from: None, error: None,
                                delivered: None, connected_peers: None,
                                ts: None, connected: None,
                            };
                            if ops_telem_tx.send(env).await.is_err() { break; }
                            ops_telem_handler.cleanup_stale_transfers().await;
                        }
                        println!("[ops] Telemetry background task ended");
                    });

                    let client_ping = client.outgoing_tx.clone();
                    let connected_ping = client.connected.clone();
                    let peers_ping = client.peers.clone();
                    let addr_for_ping = address.clone();
                    let peer_senders_ping = peer_senders.clone();
                    tokio::spawn(async move {
                        let mut tick: u64 = 0;
                        loop {
                            tokio::time::sleep(Duration::from_secs(25)).await;
                            if !*connected_ping.lock().await {
                                break;
                            }
                            let ping_env = inter_cube::ws_relay::RelayEnvelope {
                                msg_type: "ping".to_string(),
                                address: None,
                                public_key: None,
                                nonce: None,
                                signature: None,
                                to: None,
                                from: None,
                                relay_msg_type: None,
                                payload: None,
                                error: None,
                                delivered: None,
                                connected_peers: None,
                                ts: None,
                                connected: None,
                            };
                            if client_ping.send(ping_env).await.is_err() {
                                break;
                            }
                            tick += 1;
                            if tick % 2 == 0 {
                                let peers = peers_ping.lock().await.clone();
                                let now_ms = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64;
                                for peer in &peers {
                                    if peer != &addr_for_ping {
                                        let payload = format!("{{\"from\":\"{}\",\"ts\":{}}}", addr_for_ping, now_ms);
                                        if let Some(ref ps) = peer_senders_ping {
                                            send_via_peer_or_relay(
                                                peer,
                                                &payload,
                                                ps,
                                                Some(&client_ping),
                                                &addr_for_ping,
                                                "heartbeat",
                                            ).await;
                                        } else {
                                            let hb = inter_cube::ws_relay::RelayEnvelope {
                                                msg_type: "relay".to_string(),
                                                to: Some(peer.clone()),
                                                relay_msg_type: Some("heartbeat".to_string()),
                                                payload: Some(payload),
                                                address: None,
                                                public_key: None,
                                                nonce: None,
                                                signature: None,
                                                from: None,
                                                error: None,
                                                delivered: None,
                                                connected_peers: None,
                                                ts: None,
                                                connected: None,
                                            };
                                            let _ = client_ping.send(hb).await;
                                        }
                                    }
                                }
                            }
                        }
                    });

                    loop {
                        let envelope = tokio::select! {
                            msg = incoming_rx.recv() => {
                                match msg {
                                    Some(e) => e,
                                    None => break,
                                }
                            }
                            peer_msg = async {
                                if let Some(ref mut rx) = peer_msg_rx {
                                    rx.recv().await
                                } else {
                                    std::future::pending::<Option<inter_cube::ws_relay::RelayEnvelope>>().await
                                }
                            } => {
                                match peer_msg {
                                    Some(e) => {
                                        let peer_msg_type = e.relay_msg_type.as_deref().unwrap_or("unknown");
                                        if peer_msg_type == "terminal-open" || peer_msg_type == "terminal-input"
                                            || peer_msg_type == "terminal-resize" || peer_msg_type == "terminal-close"
                                            || peer_msg_type == "restart" {
                                            println!("[PEER->RELAY] REJECTED: privileged message type '{}' from direct peer channel (requires relay auth)", peer_msg_type);
                                            continue;
                                        }
                                        println!("[PEER->RELAY] Peer message injected into pipeline: {:?}", peer_msg_type);
                                        e
                                    }
                                    None => {
                                        println!("[PEER->RELAY] Peer channel closed — disabling peer input");
                                        peer_msg_rx = None;
                                        continue;
                                    }
                                }
                            }
                        };

                        let msg_type_owned = envelope.relay_msg_type.clone().unwrap_or_else(|| "unknown".to_string());
                        let msg_type: &str = &msg_type_owned;
                        let from = envelope.from.as_deref().unwrap_or("?").to_string();
                        let payload_str = envelope.payload.as_deref().unwrap_or("{}").to_string();

                        if msg_type == "yoda_chat" || msg_type == "yoda_response" {
                            let session_hint = serde_json::from_str::<serde_json::Value>(&payload_str)
                                .ok()
                                .and_then(|v| v.get("sessionId").and_then(|s| s.as_str()).map(|s| s.to_string()))
                                .unwrap_or_else(|| "unknown".to_string());
                            println!(
                                "[ws-relay] Received {} from {} — session={} (content redacted)",
                                msg_type, from, session_hint
                            );
                        } else {
                            println!(
                                "[ws-relay] Received {} from {} — {}",
                                msg_type, from,
                                payload_str.chars().take(80).collect::<String>()
                            );
                        }

                        if msg_type == "restart" || envelope.msg_type == "restart" {
                            println!("[ws-relay] Restart command received — closing relay for reconnect");
                            break;
                        }

                        if msg_type == "terminal-open" || msg_type == "terminal-input" || msg_type == "terminal-resize" || msg_type == "terminal-close" || msg_type == "plenumnet-builtin" {
                            let (authorized, reject_reason) = is_terminal_authorized(&from, &client.peers).await;
                            if !authorized {
                                println!("[terminal] REJECTED: {}", reject_reason);
                                continue;
                            }
                            let reply_tx = client.outgoing_tx.clone();
                            let from_addr = from.clone();
                            let addr_for_reply = address.clone();
                            let terminal_sessions_clone = terminal_sessions.clone();
                            let msg_type_owned = msg_type.to_string();
                            let yoda_verifier_term = yoda_verifier.clone();
                            let yoda_session_origins_term = yoda_session_origins.clone();

                            tokio::spawn(async move {
                                let yoda_verifier = yoda_verifier_term;
                                let yoda_session_origins = yoda_session_origins_term;
                                let parsed: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();
                                let session_id = parsed.get("sessionId").and_then(|v| v.as_str()).unwrap_or("").to_string();

                                match msg_type_owned.as_str() {
                                    "terminal-open" => {
                                        use portable_pty::{CommandBuilder, PtySize, native_pty_system};
                                        let pty_system = native_pty_system();
                                        let pair = match pty_system.openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 }) {
                                            Ok(p) => p,
                                            Err(e) => {
                                                println!("[terminal] Failed to open PTY: {}", e);
                                                return;
                                            }
                                        };
                                        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
                                            if cfg!(target_os = "windows") { "cmd.exe".to_string() } else { "/bin/sh".to_string() }
                                        });
                                        let mut cmd = CommandBuilder::new(&shell);
                                        cmd.env("TERM", "xterm-256color");
                                        let child = match pair.slave.spawn_command(cmd) {
                                            Ok(c) => c,
                                            Err(e) => {
                                                println!("[terminal] Failed to spawn shell: {}", e);
                                                return;
                                            }
                                        };
                                        let mut reader = match pair.master.try_clone_reader() {
                                            Ok(r) => r,
                                            Err(e) => {
                                                println!("[terminal] Failed to clone reader: {}", e);
                                                return;
                                            }
                                        };
                                        let writer = match pair.master.take_writer() {
                                            Ok(w) => w,
                                            Err(e) => {
                                                println!("[terminal] Failed to take writer: {}", e);
                                                return;
                                            }
                                        };
                                        {
                                            let mut sessions = terminal_sessions_clone.lock().unwrap_or_else(|e| e.into_inner());
                                            if sessions.len() >= 27 {
                                                println!("[terminal] REJECTED: session limit reached (max 27)");
                                                return;
                                            }
                                            sessions.insert(session_id.clone(), TerminalSession {
                                                writer: Arc::new(tokio::sync::Mutex::new(writer)),
                                                master: Arc::new(tokio::sync::Mutex::new(pair.master)),
                                                _child: child,
                                            });
                                        }
                                        let sid = session_id.clone();
                                        let sessions_for_reader = terminal_sessions_clone.clone();
                                        let from_addr_for_log = from_addr.clone();
                                        tokio::task::spawn_blocking(move || {
                                            let mut buf = [0u8; 4096];
                                            loop {
                                                match reader.read(&mut buf) {
                                                    Ok(0) => break,
                                                    Ok(n) => {
                                                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                                                        let payload = serde_json::json!({
                                                            "sessionId": sid,
                                                            "data": data,
                                                        });
                                                        let env = inter_cube::ws_relay::RelayEnvelope {
                                                            msg_type: "relay".to_string(),
                                                            to: Some(from_addr.clone()),
                                                            relay_msg_type: Some("terminal-output".to_string()),
                                                            payload: Some(payload.to_string()),
                                                            address: None,
                                                            public_key: None,
                                                            nonce: None,
                                                            signature: None,
                                                            from: None,
                                                            error: None,
                                                            delivered: None,
                                                            connected_peers: None,
                                                            ts: None,
                                                            connected: None,
                                                        };
                                                        let _ = reply_tx.blocking_send(env);
                                                    }
                                                    Err(_) => break,
                                                }
                                            }
                                            let mut sessions = sessions_for_reader.lock().unwrap_or_else(|e| e.into_inner());
                                            sessions.remove(&sid);
                                            println!("[terminal] Session {} ended", sid);
                                        });
                                        println!("[terminal] Session {} opened for {}", session_id, from_addr_for_log);
                                    }
                                    "terminal-input" => {
                                        let data = parsed.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let sessions = terminal_sessions_clone.lock().unwrap_or_else(|e| e.into_inner());
                                        if let Some(session) = sessions.get(&session_id) {
                                            let writer = session.writer.clone();
                                            tokio::spawn(async move {
                                                let mut w = writer.lock().await;
                                                let _ = std::io::Write::write_all(&mut *w, data.as_bytes());
                                            });
                                        }
                                    }
                                    "terminal-resize" => {
                                        let cols = parsed.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;
                                        let rows = parsed.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;
                                        let sessions = terminal_sessions_clone.lock().unwrap_or_else(|e| e.into_inner());
                                        if let Some(session) = sessions.get(&session_id) {
                                            let master = session.master.clone();
                                            tokio::spawn(async move {
                                                let m = master.lock().await;
                                                let _ = m.resize(portable_pty::PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
                                            });
                                        }
                                    }
                                    "terminal-close" => {
                                        let mut sessions = terminal_sessions_clone.lock().unwrap_or_else(|e| e.into_inner());
                                        if sessions.remove(&session_id).is_some() {
                                            println!("[terminal] Session {} closed by coordinator", session_id);
                                        }
                                    }
                                    "plenumnet-builtin" => {
                                        let command = parsed.get("command").and_then(|v| v.as_str()).unwrap_or("");
                                        if command == "yoda-chat" {
                                            let payload_json = parsed.get("args").and_then(|v| v.as_str())
                                                .or_else(|| parsed.get("payload").and_then(|v| v.as_str()))
                                                .unwrap_or("").to_string();
                                            let reply_tx_builtin = reply_tx.clone();
                                            let from_addr_builtin = from_addr.clone();
                                            let yoda_v = yoda_verifier.clone();
                                            let sid = session_id.clone();
                                            let yso_builtin = yoda_session_origins.clone();
                                            tokio::spawn(async move {
                                                if payload_json.is_empty() {
                                                    println!("[terminal] plenumnet-builtin yoda-chat: missing payload");
                                                    return;
                                                }
                                                let verify_result = {
                                                    let mut v = yoda_v.lock().await;
                                                    v.verify_and_forward(&payload_json)
                                                };
                                                match verify_result {
                                                    Ok((payload, hash)) => {
                                                        yso_builtin.lock().await.insert(payload.session_id.clone(), from_addr_builtin.clone());
                                                        println!(
                                                            "[terminal] plenumnet-builtin yoda-chat verified: session={} hash={}",
                                                            payload.session_id, hash
                                                        );
                                                        let env = inter_cube::ws_relay::RelayEnvelope {
                                                            msg_type: "relay".to_string(),
                                                            to: Some("yoda-server".to_string()),
                                                            relay_msg_type: Some("yoda_chat".to_string()),
                                                            payload: Some(payload_json),
                                                            address: None, public_key: None, nonce: None,
                                                            signature: None, from: None, error: None,
                                                            delivered: None, connected_peers: None,
                                                            ts: None, connected: None,
                                                        };
                                                        if let Err(e) = reply_tx_builtin.send(env).await {
                                                            println!("[terminal] yoda-chat forward failed: {}", e);
                                                        }
                                                        let ack_env = inter_cube::ws_relay::RelayEnvelope {
                                                            msg_type: "relay".to_string(),
                                                            to: Some(from_addr_builtin),
                                                            relay_msg_type: Some("terminal-output".to_string()),
                                                            payload: Some(serde_json::json!({
                                                                "sessionId": sid,
                                                                "data": format!("[YODA] Message verified and forwarded (hash={}). Waiting for response...\r\n", hash),
                                                            }).to_string()),
                                                            address: None, public_key: None, nonce: None,
                                                            signature: None, from: None, error: None,
                                                            delivered: None, connected_peers: None,
                                                            ts: None, connected: None,
                                                        };
                                                        let _ = reply_tx_builtin.send(ack_env).await;
                                                    }
                                                    Err(api_err) => {
                                                        println!(
                                                            "[terminal] plenumnet-builtin yoda-chat rejected: {}",
                                                            api_err.code
                                                        );
                                                        let err_env = inter_cube::ws_relay::RelayEnvelope {
                                                            msg_type: "relay".to_string(),
                                                            to: Some(from_addr_builtin),
                                                            relay_msg_type: Some("terminal-output".to_string()),
                                                            payload: Some(serde_json::json!({
                                                                "sessionId": sid,
                                                                "data": format!("[YODA] Error: {} (code: {})\r\n", api_err.message, api_err.code),
                                                            }).to_string()),
                                                            address: None, public_key: None, nonce: None,
                                                            signature: None, from: None, error: None,
                                                            delivered: None, connected_peers: None,
                                                            ts: None, connected: None,
                                                        };
                                                        let _ = reply_tx_builtin.send(err_env).await;
                                                    }
                                                }
                                            });
                                        } else {
                                            println!("[terminal] Unknown plenumnet-builtin command: {}", command);
                                        }
                                    }
                                    _ => {}
                                }
                            });
                            continue;
                        }

                        if msg_type == "cluster-exec" {
                            let reply_tx = client.outgoing_tx.clone();
                            let from_addr = from.clone();
                            let addr_for_reply = address.clone();
                            tokio::spawn(async move {
                                let parsed: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();
                                let command = parsed.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                println!("[cluster-exec] Executing command from {}: {}", from_addr, command);

                                let allowed = inter_cube::cluster_shell::is_command_allowed_public(&command);
                                let (output, error, exit_code) = if !allowed {
                                    (String::new(), Some("Command not in allowlist".to_string()), 1)
                                } else {
                                    match tokio::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(&command)
                                        .output()
                                        .await
                                    {
                                        Ok(out) => {
                                            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                                            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                                            let code = out.status.code().unwrap_or(1);
                                            let err = if stderr.is_empty() { None } else { Some(stderr) };
                                            (stdout, err, code)
                                        }
                                        Err(e) => (String::new(), Some(format!("Exec error: {}", e)), 1),
                                    }
                                };

                                let result_payload = serde_json::json!({
                                    "nodeId": addr_for_reply,
                                    "address": addr_for_reply,
                                    "output": output,
                                    "error": error,
                                    "exitCode": exit_code,
                                    "command": command,
                                });

                                let reply_env = inter_cube::ws_relay::RelayEnvelope {
                                    msg_type: "relay".to_string(),
                                    to: Some(from_addr),
                                    relay_msg_type: Some("cluster-exec-result".to_string()),
                                    payload: Some(result_payload.to_string()),
                                    address: None,
                                    public_key: None,
                                    nonce: None,
                                    signature: None,
                                    from: None,
                                    error: None,
                                    delivered: None,
                                    connected_peers: None,
                                    ts: None,
                                    connected: None,
                                };
                                if let Err(e) = reply_tx.send(reply_env).await {
                                    println!("[cluster-exec] Failed to send result via relay: {}", e);
                                }
                                println!("[cluster-exec] Result sent back (exit={})", exit_code);
                            });
                            continue;
                        }

                        if msg_type == "ops-config-update" {
                            let oh = ops_handler.clone();
                            tokio::spawn(async move {
                                let parsed: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();
                                if let Some(enabled) = parsed.get("ops_enabled").and_then(|v| v.as_bool()) {
                                    oh.set_ops_enabled(enabled).await;
                                    println!("[ops] Config updated from coordinator: ops_enabled={}", enabled);
                                }
                            });
                            continue;
                        }

                        if msg_type == "ops-operator-sync" {
                            let oh = ops_handler.clone();
                            let yoda_v = yoda_verifier.clone();
                            tokio::spawn(async move {
                                let parsed: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();
                                let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("");
                                match action {
                                    "add" => {
                                        let fp = parsed.get("key_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let pk = parsed.get("public_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let scope = parsed.get("scope").and_then(|v| v.as_str()).unwrap_or("read-only").to_string();
                                        oh.add_operator(fp.clone(), name.clone(), pk.clone(), scope.clone()).await;
                                        let now_secs = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs();
                                        let entry = crate::ops_handler::OperatorEntry {
                                            name: name.clone(),
                                            key_fingerprint: fp.clone(),
                                            public_key: pk.clone(),
                                            scope: scope.clone(),
                                            registered_at: format!("{}Z", now_secs),
                                        };
                                        yoda_v.lock().await.add_operator(pk, entry);
                                        println!("[ops] Operator synced from coordinator: {} ({}, scope: {})", name, fp, scope);
                                    }
                                    "remove" => {
                                        let fp = parsed.get("key_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let pk = parsed.get("public_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        oh.remove_operator(&fp).await;
                                        if !pk.is_empty() {
                                            yoda_v.lock().await.remove_operator(&pk);
                                        }
                                        println!("[ops] Operator removed via coordinator sync: {}", fp);
                                    }
                                    _ => {}
                                }
                            });
                            continue;
                        }

                        let ops_msg_types = [
                            "exec", "tail", "tail-stop", "file-push", "file-pull",
                            "chunk-init", "chunk-data", "chunk-complete", "transfer-cancel", "model-swap",
                        ];
                        if ops_msg_types.contains(&msg_type) {
                            let oh = ops_handler.clone();
                            let reply_tx = client.outgoing_tx.clone();
                            let addr_ops = address.clone();
                            let from_ops = from.clone();
                            let msg_type_ops = msg_type.to_string();
                            tokio::spawn(async move {
                                let parsed: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();
                                let mut msg_with_type = parsed.clone();
                                if let Some(obj) = msg_with_type.as_object_mut() {
                                    obj.insert("type".to_string(), serde_json::json!(msg_type_ops));
                                }
                                println!("[ops] Handling {} from {}", msg_type_ops, from_ops);
                                if let Some(response) = oh.handle_ops_message(&msg_with_type).await {
                                    let response_type = response.get("type")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("ops-error")
                                        .to_string();
                                    let env = inter_cube::ws_relay::RelayEnvelope {
                                        msg_type: "relay".to_string(),
                                        to: Some(from_ops),
                                        relay_msg_type: Some(response_type),
                                        payload: Some(response.to_string()),
                                        address: None, public_key: None, nonce: None,
                                        signature: None, from: None, error: None,
                                        delivered: None, connected_peers: None,
                                        ts: None, connected: None,
                                    };
                                    if let Err(e) = reply_tx.send(env).await {
                                        println!("[ops] Failed to send response: {}", e);
                                    }
                                }
                            });
                            continue;
                        }

                        if msg_type == "yoda_chat" {
                            let reply_tx = client.outgoing_tx.clone();
                            let from_addr = from.clone();
                            let yoda_verifier = yoda_verifier.clone();
                            let yso = yoda_session_origins.clone();

                            tokio::spawn(async move {
                                let mut verifier = yoda_verifier.lock().await;
                                match verifier.verify_and_forward(&payload_str) {
                                    Ok((verified_payload, payload_hash)) => {
                                        yso.lock().await.insert(verified_payload.session_id.clone(), from_addr.clone());
                                        let forward_payload = serde_json::json!({
                                            "sessionId": verified_payload.session_id,
                                            "timestamp": verified_payload.timestamp,
                                            "sequence": verified_payload.sequence,
                                            "message": verified_payload.message,
                                            "operatorPubkey": verified_payload.operator_pubkey,
                                            "daemonRepC": verified_payload.daemon_rep_c,
                                            "signature": verified_payload.signature,
                                        });
                                        let env = inter_cube::ws_relay::RelayEnvelope {
                                            msg_type: "relay".to_string(),
                                            to: Some("yoda-server".to_string()),
                                            relay_msg_type: Some("yoda_chat".to_string()),
                                            payload: Some(forward_payload.to_string()),
                                            address: None, public_key: None, nonce: None,
                                            signature: None, from: None, error: None,
                                            delivered: None, connected_peers: None,
                                            ts: None, connected: None,
                                        };
                                        if let Err(e) = reply_tx.send(env).await {
                                            println!("[yoda-chat] Failed to forward to relay: {}", e);
                                        }
                                    }
                                    Err(api_err) => {
                                        println!(
                                            "[yoda-chat] rejected: {} from {}",
                                            api_err.code, from_addr
                                        );
                                        let err_response = serde_json::json!({
                                            "code": api_err.code,
                                            "message": api_err.message,
                                            "exitCode": api_err.exit_code,
                                        });
                                        let env = inter_cube::ws_relay::RelayEnvelope {
                                            msg_type: "relay".to_string(),
                                            to: Some(from_addr),
                                            relay_msg_type: Some("yoda_response".to_string()),
                                            payload: Some(err_response.to_string()),
                                            address: None, public_key: None, nonce: None,
                                            signature: None, from: None, error: None,
                                            delivered: None, connected_peers: None,
                                            ts: None, connected: None,
                                        };
                                        let _ = reply_tx.send(env).await;
                                    }
                                }
                            });
                            continue;
                        }

                        if msg_type == "yoda_response" {
                            if from != "yoda-server" {
                                println!(
                                    "[yoda-chat] Rejected yoda_response from untrusted source: {}",
                                    from
                                );
                                continue;
                            }
                            let reply_tx = client.outgoing_tx.clone();
                            let from_addr = from.clone();
                            let yso = yoda_session_origins.clone();
                            let yw = yoda_response_waiters.clone();
                            tokio::spawn(async move {
                                let parsed: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();
                                let session_id = parsed.get("sessionId")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                let has_error = parsed.get("error").is_some();
                                let response_sequence = parsed.get("sequence")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);

                                let response_hash = inter_cube::yoda_chat::compute_payload_hash(
                                    payload_str.as_bytes()
                                );

                                let origin_addr = yso.lock().await.get(&session_id).cloned();

                                println!(
                                    "[yoda-chat] Response session={} error={} hash={} origin={:?}",
                                    session_id, has_error, response_hash, origin_addr
                                );

                                let audit_path = format!(
                                    "{}/.plenumnet/yoda-audit.jsonl",
                                    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
                                );
                                let now_ms = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64;
                                inter_cube::yoda_chat::write_audit_entry(&audit_path, &inter_cube::yoda_chat::YodaAuditEntry {
                                    timestamp: inter_cube::yoda_chat::format_timestamp_rfc3339(now_ms),
                                    session_id: session_id.clone(),
                                    sequence: response_sequence,
                                    operator_rep_c: origin_addr.as_deref().unwrap_or(&from_addr).to_string(),
                                    payload_hash: Some(response_hash.clone()),
                                    direction: "inbound".to_string(),
                                    response_hash: Some(response_hash),
                                    result: if has_error { "error".to_string() } else { "delivered".to_string() },
                                });

                                let waiter = {
                                    let mut w = yw.lock().await;
                                    w.remove(&session_id)
                                };
                                if let Some(waiter_tx) = waiter {
                                    let _ = waiter_tx.send(payload_str.clone());
                                    println!("[yoda-chat] Response delivered to HTTP waiter for session {}", session_id);
                                } else if let Some(target) = origin_addr {
                                    let response_env = inter_cube::ws_relay::RelayEnvelope {
                                        msg_type: "relay".to_string(),
                                        to: Some(target),
                                        relay_msg_type: Some("yoda_response".to_string()),
                                        payload: Some(payload_str),
                                        address: None, public_key: None, nonce: None,
                                        signature: None, from: None, error: None,
                                        delivered: None, connected_peers: None,
                                        ts: None, connected: None,
                                    };
                                    if let Err(e) = reply_tx.send(response_env).await {
                                        println!("[yoda-chat] Failed to deliver response: {}", e);
                                    }
                                } else {
                                    println!("[yoda-chat] No origin found for session {} — response dropped", session_id);
                                }
                            });
                            continue;
                        }

                        if msg_type == "inference_request" {
                            let llm_url = format!("{}/v1/chat/completions", llm_base_url);
                            let http = inference_client.clone();
                            let reply_tx = client.outgoing_tx.clone();
                            let from_addr = from.clone();
                            let ps_inference = peer_senders.clone();
                            let addr_inference = address.clone();

                            tokio::spawn(async move {
                                let parsed: serde_json::Value = match serde_json::from_str(&payload_str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        println!("[inference] Failed to parse request payload: {}", e);
                                        send_inference_error(&reply_tx, &from_addr, "unknown", &format!("Invalid payload JSON: {}", e)).await;
                                        return;
                                    }
                                };

                                let request_id = parsed.get("requestId")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();

                                let messages = parsed.get("messages").cloned().unwrap_or(serde_json::json!([]));
                                let max_tokens = parsed.get("maxTokens").and_then(|v| v.as_u64()).unwrap_or(512);
                                let model = parsed.get("model").and_then(|v| v.as_str()).unwrap_or("local").to_string();
                                let temperature = parsed.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);

                                println!("[inference] Processing request {} from {} (model={}, maxTokens={})",
                                    request_id, &from_addr, model, max_tokens);

                                let llm_body = serde_json::json!({
                                    "model": model,
                                    "messages": messages,
                                    "max_tokens": max_tokens,
                                    "temperature": temperature,
                                    "stream": false,
                                });

                                let result = http.post(&llm_url)
                                    .header("Content-Type", "application/json")
                                    .json(&llm_body)
                                    .send()
                                    .await;

                                match result {
                                    Ok(resp) => {
                                        let status = resp.status();
                                        match resp.text().await {
                                            Ok(body) => {
                                                if status.is_success() {
                                                    let llm_resp: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
                                                    let content = llm_resp
                                                        .get("choices")
                                                        .and_then(|c| c.get(0))
                                                        .and_then(|c| c.get("message"))
                                                        .and_then(|m| m.get("content"))
                                                        .and_then(|c| c.as_str())
                                                        .unwrap_or("");
                                                    let usage = llm_resp.get("usage").cloned().unwrap_or(serde_json::json!({}));
                                                    let tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

                                                    println!("[inference] Request {} completed — {} tokens", request_id, tokens);

                                                    let response_payload = serde_json::json!({
                                                        "requestId": request_id,
                                                        "content": content,
                                                        "model": model,
                                                        "tokens": tokens,
                                                        "usage": usage,
                                                    });

                                                    if let Some(ref ps) = ps_inference {
                                                        send_via_peer_or_relay(
                                                            &from_addr,
                                                            &response_payload.to_string(),
                                                            ps,
                                                            Some(&reply_tx),
                                                            &addr_inference,
                                                            "inference_response",
                                                        ).await;
                                                    } else {
                                                        let reply_env = inter_cube::ws_relay::RelayEnvelope {
                                                            msg_type: "relay".to_string(),
                                                            to: Some(from_addr.clone()),
                                                            relay_msg_type: Some("inference_response".to_string()),
                                                            payload: Some(response_payload.to_string()),
                                                            address: None,
                                                            public_key: None,
                                                            nonce: None,
                                                            signature: None,
                                                            from: None,
                                                            error: None,
                                                            delivered: None,
                                                            connected_peers: None,
                                                            ts: None,
                                                            connected: None,
                                                        };
                                                        if let Err(e) = reply_tx.send(reply_env).await {
                                                            println!("[inference] Failed to send response via relay: {}", e);
                                                        }
                                                    }
                                                } else {
                                                    println!("[inference] LLM returned {} for request {}", status, request_id);
                                                    send_inference_error(&reply_tx, &from_addr, &request_id,
                                                        &format!("LLM server returned {}: {}", status, body.chars().take(200).collect::<String>())).await;
                                                }
                                            }
                                            Err(e) => {
                                                println!("[inference] Failed to read LLM response body: {}", e);
                                                send_inference_error(&reply_tx, &from_addr, &request_id, &format!("Read error: {}", e)).await;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let is_connect = e.is_connect() || e.is_timeout();
                                        println!("[inference] LLM request failed for {}: {} (connect_issue={})", request_id, e, is_connect);
                                        let hint = if is_connect {
                                            format!("LLM server unreachable at {} — is llama-server running? Error: {}", llm_url, e)
                                        } else {
                                            format!("LLM request failed: {}", e)
                                        };
                                        send_inference_error(&reply_tx, &from_addr, &request_id, &hint).await;
                                    }
                                }
                            });
                        }
                    }

                    println!("[ws-relay] Relay connection lost");
                    ops_handler.cancel_all_tails().await;
                    { let mut yrt = yoda_relay_tx.lock().await; *yrt = None; }
                }
                Err(e) => {
                    println!("[ws-relay] Connection failed: {}", e);
                }
            }

            println!(
                "[ws-relay] Reconnecting in {} seconds...",
                retry_delay.as_secs()
            );
            tokio::time::sleep(retry_delay).await;
            retry_delay = std::cmp::min(retry_delay * 2, Duration::from_secs(60));
        }
    });
}

async fn send_inference_error(
    tx: &tokio::sync::mpsc::Sender<inter_cube::ws_relay::RelayEnvelope>,
    to: &str,
    request_id: &str,
    error_msg: &str,
) {
    let payload = serde_json::json!({
        "requestId": request_id,
        "error": error_msg,
    });
    let env = inter_cube::ws_relay::RelayEnvelope {
        msg_type: "relay".to_string(),
        to: Some(to.to_string()),
        relay_msg_type: Some("inference_error".to_string()),
        payload: Some(payload.to_string()),
        address: None,
        public_key: None,
        nonce: None,
        signature: None,
        from: None,
        error: None,
        delivered: None,
        connected_peers: None,
        ts: None,
        connected: None,
    };
    let _ = tx.send(env).await;
}

// ═══════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    parse_cli_args();

    let version = env!("CARGO_PKG_VERSION");

    let mode = env::var("CUBE_MODE").unwrap_or_else(|_| "all".to_string());

    if mode != "hash" && mode != "version" {
        println!("===========================================================");
        println!("  PlenumNET Inter-Cube Infrastructure Services v{}", version);
        println!("  Wire Protocol: V{}", inter_cube::wire::PROTOCOL_VERSION_CURRENT);
        println!("  Applied Physics Division -- Capomastro Holdings Ltd.");
        println!("===========================================================");
        println!();
    }

    match mode.as_str() {
        "crs" | "all" => run_crs_mode().await,
        "cube" => run_cube_mode().await,
        "keygen" => inter_cube::daemon_identity::run_keygen(),
        "hash" => {
            let target = env::var("CUBE_HASH_TARGET").unwrap_or_default();
            if target.is_empty() {
                eprintln!("ERROR: CUBE_HASH_TARGET not set. Provide the file path to hash.");
                std::process::exit(1);
            }
            match std::fs::read(&target) {
                Ok(data) => {
                    let hash = inter_cube::yoda_chat::compute_payload_hash(&data);
                    println!("TIS-27 hash: {}", hash.trim_start_matches("tis27:"));
                }
                Err(e) => {
                    eprintln!("ERROR: Cannot read '{}': {}", target, e);
                    std::process::exit(1);
                }
            }
        }
        "version" => {
            println!("version: {}", version);
        }
        other => {
            println!(
                "ERROR: Unknown CUBE_MODE '{}'. Use 'crs', 'cube', 'all', 'keygen', 'hash', or 'version'.",
                other
            );
            std::process::exit(1);
        }
    }
}
