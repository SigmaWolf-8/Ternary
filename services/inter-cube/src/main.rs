// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumNET Inter-Cube Infrastructure Daemon v0.4.0
//
// MODES (controlled by CUBE_MODE env var):
//   "crs"    — Central Registration Service. Allocates addresses,
//              accepts registrations, serves full API.
//   "cube"   — Worker cube. Registers with a remote CRS on boot,
//              gets a unique address, heartbeats every 30s,
//              serves local stats API.
//   "all"    — Same as "crs" (backward compat).
//   "keygen" — Generate PT26-DSA identity keypair and exit.
//
// TRI-PORT ARCHITECTURE (3 ports per node, spacing = 3):
//   Daemon #1: 8079 (peer), 8080 (app API), 8081 (node/relay)
//   Daemon #2: 8082 (peer), 8083 (app API), 8084 (node/relay)
//   Daemon #3: 8085 (peer), 8086 (app API), 8087 (node/relay)
//
// ENV VARS:
//   CUBE_MODE                  — "crs", "cube", "all", or "keygen" (default: "all")
//   CUBE_CRS_URL               — CRS base URL (required for cube mode)
//   RELAY_URL                  — WebSocket relay URL (default: CUBE_CRS_URL)
//                                Set to remote relay (e.g. https://plenumnet.replit.app)
//                                when CUBE_CRS_URL points to a local CRS
//   LLM_PORT                   — Local LLM engine port for inference dispatch (default: 8080)
//   CUBE_ENDPOINT              — Wire protocol endpoint (default: "0.0.0.0:51820")
//   ADDRESS                    — Alias for CUBE_ENDPOINT
//   CUBE_ROLE                  — Role annotation (inference, review, kb, infra, relay, standby)
//   ROLE                       — Alias for CUBE_ROLE
//   CUBE_API_PORT              — HTTP API bind port (default: 8080)
//   API_PORT                   — Alias for CUBE_API_PORT
//   CUBE_PEER_PORT             — Direct peer-to-peer port (default: API_PORT - 1)
//   PEER_PORT                  — Alias for CUBE_PEER_PORT
//   CUBE_IDENTITY_DIR          — Directory for master.key (default: ~/.plenumnet/identity/)
//   CUBE_IDENTITY_PASSPHRASE   — Passphrase for master.key encryption

use inter_cube::*;
use inter_cube::api::{
    AppState, crs_router, cube_router, parse_address_string,
    CRS_ROUTE_COUNT, CUBE_ROUTE_COUNT,
};
use inter_cube::daemon_identity::{DaemonIdentity, encryption_passphrase, identity_dir, save_master_secret};
use inter_cube::address_keys::derive_identity_keypair;
use inter_cube::key_rotation::RotationOrchestrator;
use inter_cube::ws_relay::WsRelayClient;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

static mut CLI_PEER_PORT: Option<u16> = None;

fn parse_cli_args() {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--peer-port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        unsafe { CLI_PEER_PORT = Some(p); }
                        println!("[CLI] --peer-port {}", p);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            s if s.starts_with("--peer-port=") => {
                if let Ok(p) = s[12..].parse::<u16>() {
                    unsafe { CLI_PEER_PORT = Some(p); }
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
        .unwrap_or(8080)
}

fn peer_port() -> u16 {
    unsafe {
        if let Some(p) = CLI_PEER_PORT {
            return p;
        }
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

fn spawn_peer_listener(port: u16, local_address_dotted: String, peers: PeerConnections) {
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
                            tokio::spawn(async move {
                                println!("[PEER] Connection from {} (local address: {})", peer_addr, addr);
                                let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                                    Ok(ws) => ws,
                                    Err(e) => {
                                        println!("[PEER] WebSocket handshake failed from {}: {}", peer_addr, e);
                                        return;
                                    }
                                };
                                use futures_util::StreamExt;
                                let (mut _write, mut read) = ws_stream.split();
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
                                                        println!("[PEER] Registered LAN peer {} at {}:{}", remote_addr, peer_addr.ip(), remote_peer_port);
                                                    }
                                                } else {
                                                    println!("[PEER] Message from {}: {} bytes", peer_addr, m.len());
                                                }
                                            } else {
                                                println!("[PEER] Message from {}: {} bytes", peer_addr, m.len());
                                            }
                                        }
                                        Ok(m) if m.is_binary() => {
                                            println!("[PEER] Binary from {}: {} bytes", peer_addr, m.len());
                                        }
                                        Ok(m) if m.is_close() => {
                                            println!("[PEER] Peer {} disconnected", peer_addr);
                                            break;
                                        }
                                        Ok(_) => {}
                                        Err(e) => {
                                            println!("[PEER] Error from {}: {}", peer_addr, e);
                                            break;
                                        }
                                    }
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

fn spawn_peer_discovery(crs_url: String, local_address: String, local_peer_port: u16, peers: PeerConnections, senders: PeerSenders) {
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .user_agent("PlenumNET-InterCube/0.4.0")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let discover_url = format!(
                "{}/api/salvi/inter-cube/relay/peer-discovery?address={}&peerPort={}",
                crs_url, local_address, local_peer_port
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
                                            );
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
) {
    tokio::spawn(async move {
        let ws_url = format!("ws://{}:{}", ip, port);
        println!("[PEER] Connecting to LAN peer {} at {}", remote_address, ws_url);

        let connect_result = tokio_tungstenite::connect_async(&ws_url).await;
        match connect_result {
            Ok((ws_stream, _)) => {
                println!("[PEER] Direct connection established to {} ({}:{})", remote_address, ip, port);

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
                let read_task = tokio::spawn(async move {
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(m) if m.is_text() || m.is_binary() => {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default().as_secs();
                                if let Ok(mut pg) = peers_read.lock() {
                                    if let Some(info) = pg.get_mut(&remote_addr_read) {
                                        info.last_seen = now;
                                    }
                                }
                                println!("[PEER-DIRECT] Received {} bytes from {}", m.len(), remote_addr_read);
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
                println!("[PEER] Failed to connect to {} at {}:{} — {}", remote_address, ip, port, e);
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
            "type": msg_type,
            "from": from,
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
    println!("  Identity:      PT26-DSA (71-byte sigs, 28-sig budget)");
    println!();
    println!("  CRS -> CON -> FTS -> GLB pipeline operational.");
    println!("  The geometry IS the routing protocol.");

    let shared_state = AppState::new_crs(crs, con, fts, glb, local_address.clone());
    let app = crs_router(shared_state);

    let port = api_port();
    let p_port = peer_port();
    let listen_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    if let Some(role) = role_label() {
        println!("  Role:          {}", role);
    }

    let addr_str: String = local_address.to_bytes().iter().map(|t| t.to_string()).collect();
    if let Some(rurl) = relay_url(None) {
        println!("  Relay:         {} (WebSocket, TL-DSA-87 challenge-response)", rurl);
        let relay_kp = derive_identity_keypair(&local_address, &identity.master_secret);
        let tl_dsa_pk_hex: String = relay_kp.public_key.iter().map(|b| format!("{:02x}", b)).collect();
        spawn_relay_client(rurl, addr_str, identity.pk_hex.clone(), relay_kp.secret_key.clone(), tl_dsa_pk_hex, None);
    } else {
        println!("  Relay:         none (set RELAY_URL to enable remote relay)");
    }

    let peers = new_peer_registry();
    spawn_peer_listener(p_port, local_address.to_dotted(), peers.clone());

    println!();
    println!("=== HTTP Server (CRS) ===");
    println!("  http://{}", listen_addr);
    println!("  Peer port: {}", p_port);
    println!("  {} routes active", CRS_ROUTE_COUNT);
    println!("  Ready for cube registrations. Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect(&format!("Failed to bind to port {}", port));

    axum::serve(listener, app)
        .await
        .expect("Server error");
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
        .user_agent("PlenumNET-InterCube/0.2.0")
        .build()
        .expect("Failed to build HTTP client");
    let endpoint_encoded = resolved_endpoint.replace(":", "%3A");
    let register_url = format!(
        "{}/api/salvi/inter-cube/relay/register?publicKey={}&endpoint={}",
        crs_url, key_hex, endpoint_encoded
    );
    let register_url_post = format!("{}/api/salvi/inter-cube/crs/register", crs_url);

    let mut response_body: Option<serde_json::Value> = None;
    for attempt in 1..=10 {
        println!(
            "[CUBE] Registration attempt {}/10 -> {}",
            attempt,
            if attempt <= 5 { &register_url } else { &register_url_post }
        );

        let result = if attempt <= 5 {
            client.get(&register_url).send().await
        } else {
            client
                .post(&register_url_post)
                .json(&serde_json::json!({
                    "endpoint": resolved_endpoint,
                    "publicKey": key_hex,
                }))
                .send()
                .await
        };

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

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let registration_timestamp = now_ts;
    let passphrase = encryption_passphrase();
    let orchestrator = RotationOrchestrator::new(
        local_address.clone(),
        registration_timestamp,
        identity.master_secret.clone(),
        identity.current_radian_epoch(),
        passphrase.clone(),
    );
    let orchestrator = Arc::new(Mutex::new(orchestrator));

    let crs_url_for_heartbeat = crs_url.clone();
    let endpoint_for_heartbeat = cube_endpoint.clone();
    let addr_trits: Vec<u8> = local_address.to_bytes().to_vec();
    let orchestrator_hb = orchestrator.clone();
    let local_addr_hb = local_address.clone();
    let passphrase_hb = passphrase.clone();
    let key_hex_hb = key_hex.clone();

    tokio::spawn(async move {
        let hb_client = reqwest::Client::builder()
            .user_agent("PlenumNET-InterCube/0.2.0")
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
            let hb_url = format!(
                "{}?address={}&publicKey={}",
                hb_url_base, addr_str_hb, key_hex_hb
            );
            let result = hb_client
                .get(&hb_url)
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
        }
    });

    let addr_str_for_relay: String = local_address.to_bytes().iter().map(|t| t.to_string()).collect();
    let relay_target = relay_url(Some(&crs_url)).unwrap_or_else(|| crs_url.clone());
    let p_port = peer_port();
    let peers = new_peer_registry();
    let peer_senders = new_peer_senders();

    println!("[ws-relay] Relay target: {}", relay_target);
    let relay_kp = derive_identity_keypair(&local_address, &identity.master_secret);
    let relay_tl_dsa_pk_hex: String = relay_kp.public_key.iter().map(|b| format!("{:02x}", b)).collect();
    spawn_relay_client(relay_target, addr_str_for_relay, key_hex.clone(), relay_kp.secret_key.clone(), relay_tl_dsa_pk_hex, Some(peer_senders.clone()));

    spawn_peer_listener(p_port, local_address.to_dotted(), peers.clone());

    let addr_str_for_discovery: String = local_address.to_bytes().iter().map(|t| t.to_string()).collect();
    spawn_peer_discovery(crs_url.clone(), addr_str_for_discovery, p_port, peers.clone(), peer_senders.clone());

    let shared_state = AppState::new_cube(con, fts, glb, local_address);
    let app = cube_router(shared_state);

    let port = api_port();
    let listen_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!();
    println!("=== HTTP Server (Cube) ===");
    println!("  http://{}", listen_addr);
    println!("  Peer port: {}", p_port);
    println!("  {} routes active", CUBE_ROUTE_COUNT);
    println!("  Routing:   Direct peer (LAN) -> Relay (WAN) fallback");
    println!("  Heartbeat every 30s to CRS (with key rotation check). Ctrl+C to stop.");
    println!();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect(&format!("Failed to bind to port {}", port));

    axum::serve(listener, app)
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
) {
    let llm_port = env::var("LLM_PORT").unwrap_or_else(|_| "8080".to_string());
    let llm_base_url = format!("http://127.0.0.1:{}", llm_port);
    let api_port_val = api_port();
    let endpoint_str = format!("0.0.0.0:{}", api_port_val);
    tokio::spawn(async move {
        println!();
        println!("[ws-relay] Establishing relay connection to {}...", relay_url_str);
        println!("[ws-relay] Inference dispatch target: {}/v1/chat/completions", llm_base_url);
        let mut retry_delay = Duration::from_secs(5);
        let inference_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent("PlenumNET-InterCube/0.3.0")
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
                "{}/api/salvi/inter-cube/relay/register?publicKey={}&endpoint={}&tlDsaPk={}",
                http_base,
                &public_key_hex,
                &endpoint_str,
                &tl_dsa_pk_hex,
            );
            match reqwest::get(&reg_url).await {
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

                    while let Some(envelope) = incoming_rx.recv().await {
                        let msg_type = envelope.relay_msg_type.as_deref().unwrap_or("unknown");
                        let from = envelope.from.as_deref().unwrap_or("?").to_string();
                        let payload_str = envelope.payload.as_deref().unwrap_or("{}").to_string();

                        println!(
                            "[ws-relay] Received {} from {} — {}",
                            msg_type, from,
                            payload_str.chars().take(80).collect::<String>()
                        );

                        if msg_type == "restart" || envelope.msg_type == "restart" {
                            println!("[ws-relay] Restart command received — closing relay for reconnect");
                            break;
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
                                    request_id, from_addr, model, max_tokens);

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

    println!("===========================================================");
    println!(
        "  PlenumNET Inter-Cube Infrastructure Services v{}",
        VERSION
    );
    println!("  Applied Physics Division -- Capomastro Holdings Ltd.");
    println!("===========================================================");
    println!();

    let mode = env::var("CUBE_MODE").unwrap_or_else(|_| "all".to_string());

    match mode.as_str() {
        "crs" | "all" => run_crs_mode().await,
        "cube" => run_cube_mode().await,
        "keygen" => inter_cube::daemon_identity::run_keygen(),
        other => {
            println!(
                "ERROR: Unknown CUBE_MODE '{}'. Use 'crs', 'cube', 'keygen', or 'all'.",
                other
            );
            std::process::exit(1);
        }
    }
}
