// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Yoda CLI — `y` command
//!
//! Standalone code-signed binary for operator-to-Yoda communication.
//! Connects to the local daemon, signs messages via NinjaExec, and
//! streams Yoda responses to stdout with `[YODA]` prefix formatting.
//!
//! ## Usage
//!
//! ```text
//! y Hey Yoda, are you still working on that Compression Protocol?
//! y --status         # Show connection status
//! y --session list   # List recent sessions
//! y --session <id>   # Resume a specific session
//! y --doctor         # Run prerequisite health checks
//! ```

use inter_cube::yoda_chat::*;
use std::io::Write;
use std::path::PathBuf;

const DAEMON_DEFAULT_PORT: u16 = 11124;
const NINJAEXEC_PORT: u16 = 21027;
const PLENUMNET_DIR: &str = ".plenumnet";
const SESSION_FILE: &str = "yoda-session.id";
const AUDIT_FILE: &str = "yoda-audit.jsonl";
const LOCK_FILE: &str = "yoda-session.lock";
const LOCK_TIMEOUT_MS: u64 = 2000;

fn plenumnet_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(PLENUMNET_DIR)
}

fn ensure_data_dir() -> Result<PathBuf, String> {
    let dir = plenumnet_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| {
            format!(
                "Cannot create PlenumNET data directory at `{}` — check permissions and available disk space. ({})",
                dir.display(), e
            )
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
    }
    Ok(dir)
}

fn session_file_path() -> PathBuf {
    plenumnet_dir().join(SESSION_FILE)
}

fn lock_file_path() -> PathBuf {
    plenumnet_dir().join(LOCK_FILE)
}

fn audit_file_path() -> String {
    plenumnet_dir().join(AUDIT_FILE).to_string_lossy().to_string()
}

struct SessionLock {
    #[cfg(unix)]
    _file: std::fs::File,
    #[cfg(not(unix))]
    _path: PathBuf,
}

impl SessionLock {
    fn acquire() -> Result<Self, YodaErrorCode> {
        let lock_path = lock_file_path();
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&lock_path)
                .map_err(|_| YodaErrorCode::ConcurrentAccess)?;

            let fd = file.as_raw_fd();
            let start = std::time::Instant::now();
            loop {
                let result = unsafe {
                    libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB)
                };
                if result == 0 {
                    return Ok(SessionLock { _file: file });
                }
                if start.elapsed().as_millis() as u64 >= LOCK_TIMEOUT_MS {
                    return Err(YodaErrorCode::ConcurrentAccess);
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        #[cfg(not(unix))]
        {
            let start = std::time::Instant::now();
            loop {
                match std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&lock_path)
                {
                    Ok(_) => return Ok(SessionLock { _path: lock_path }),
                    Err(_) => {
                        if start.elapsed().as_millis() as u64 >= LOCK_TIMEOUT_MS {
                            return Err(YodaErrorCode::ConcurrentAccess);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
        }
    }
}

#[cfg(not(unix))]
impl Drop for SessionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self._path);
    }
}

fn load_or_create_session(resume_id: Option<&str>) -> Result<SessionFile, String> {
    let path = session_file_path();

    if let Some(id) = resume_id {
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(mut session) = serde_json::from_str::<SessionFile>(&data) {
                    if session.session_id == id {
                        session.last_active = current_timestamp_ms();
                        return Ok(session);
                    }
                }
            }
        }
        let session = SessionFile {
            session_id: id.to_string(),
            sequence: 0,
            last_active: current_timestamp_ms(),
        };
        return Ok(session);
    }

    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(session) = serde_json::from_str::<SessionFile>(&data) {
                if !session.is_expired() {
                    return Ok(session);
                }
            }
        }
    }

    Ok(SessionFile::new())
}

fn save_session(session: &SessionFile) -> Result<(), String> {
    let path = session_file_path();
    let data = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;
    std::fs::write(&path, data)
        .map_err(|e| format!("Failed to write session file: {}", e))?;
    Ok(())
}

fn daemon_api_url() -> String {
    let port = std::env::var("CUBE_API_PORT")
        .or_else(|_| std::env::var("API_PORT"))
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DAEMON_DEFAULT_PORT);
    format!("http://127.0.0.1:{}", port)
}

fn print_error_and_exit(code: YodaErrorCode) -> ! {
    let msg = code.display_message();
    eprintln!("{}", msg);

    if code == YodaErrorCode::DaemonNotRunning && !plenumnet_dir().exists() {
        eprintln!(
            "If this is your first time, run `plenumnet-daemon start` to initialize your node, then try again."
        );
    }

    std::process::exit(code.exit_code());
}

fn query_daemon_status() -> Result<(String, bool), YodaErrorCode> {
    let url = format!("{}/health", daemon_api_url());
    match ureq_get_json(&url) {
        Ok(json) => {
            let rep_c = json.get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let mode = json.get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let relay_available = mode == "crs"
                || std::env::var("RELAY_URL").map(|v| !v.is_empty()).unwrap_or(false);
            if rep_c.is_empty() {
                Err(YodaErrorCode::DaemonNotRunning)
            } else {
                Ok((rep_c, relay_available))
            }
        }
        Err(_) => Err(YodaErrorCode::DaemonNotRunning),
    }
}

fn ureq_get_json(url: &str) -> Result<serde_json::Value, String> {
    let without_scheme = url.replace("http://", "");
    let (host_port, path) = if let Some(idx) = without_scheme.find('/') {
        (&without_scheme[..idx], &without_scheme[idx..])
    } else {
        (without_scheme.as_str(), "/")
    };
    let addr: std::net::SocketAddr = host_port.parse().map_err(|e| format!("addr: {}", e))?;
    let tcp = std::net::TcpStream::connect_timeout(
        &addr,
        std::time::Duration::from_secs(3),
    ).map_err(|e| format!("connect: {}", e))?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();

    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept: application/json\r\n\r\n", path, host_port);
    use std::io::{Read, Write as IoWrite};
    let mut stream = tcp;
    stream.write_all(request.as_bytes()).map_err(|e| format!("write: {}", e))?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|e| format!("read: {}", e))?;
    let response_str = String::from_utf8_lossy(&response);

    if let Some(body_start) = response_str.find("\r\n\r\n") {
        let body = &response_str[body_start + 4..];
        serde_json::from_str(body).map_err(|e| format!("json: {}", e))
    } else {
        Err("no HTTP body".to_string())
    }
}

fn query_ninjaexec_status() -> Result<bool, ()> {
    match std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", NINJAEXEC_PORT).parse().unwrap(),
        std::time::Duration::from_secs(2),
    ) {
        Ok(_) => Ok(true),
        Err(_) => Err(()),
    }
}

fn ninjaexec_sign(payload: &[u8]) -> Result<(String, String), YodaErrorCode> {
    let url = format!("http://127.0.0.1:{}", NINJAEXEC_PORT);

    let sign_body = serde_json::json!({
        "payload_b64": base64_encode(payload),
        "context": "PlenumNET-YODA-CHAT-v1",
    });
    let body_str = serde_json::to_string(&sign_body).unwrap();

    let tcp = std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", NINJAEXEC_PORT).parse().unwrap(),
        std::time::Duration::from_secs(3),
    ).map_err(|_| YodaErrorCode::NinjaexecNotRunning)?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();

    let request = format!(
        "POST /sign HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        NINJAEXEC_PORT, body_str.len(), body_str
    );
    use std::io::{Read, Write as IoWrite};
    let mut stream = tcp;
    stream.write_all(request.as_bytes()).map_err(|_| YodaErrorCode::NinjaexecNotRunning)?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|_| YodaErrorCode::NinjaexecNotRunning)?;
    let response_str = String::from_utf8_lossy(&response);

    if let Some(body_start) = response_str.find("\r\n\r\n") {
        let body = &response_str[body_start + 4..];
        let json: serde_json::Value = serde_json::from_str(body)
            .map_err(|_| YodaErrorCode::SignatureInvalid)?;

        let signature = json.get("signature_b64")
            .and_then(|v| v.as_str())
            .ok_or(YodaErrorCode::SignatureInvalid)?
            .to_string();
        let pubkey = json.get("pubkey_b64")
            .and_then(|v| v.as_str())
            .ok_or(YodaErrorCode::SignatureInvalid)?
            .to_string();
        Ok((signature, pubkey))
    } else {
        Err(YodaErrorCode::NinjaexecNotRunning)
    }
}

fn ninjaexec_pubkey() -> Result<String, YodaErrorCode> {
    let url_str = format!("127.0.0.1:{}", NINJAEXEC_PORT);
    let tcp = std::net::TcpStream::connect_timeout(
        &url_str.parse().unwrap(),
        std::time::Duration::from_secs(2),
    ).map_err(|_| YodaErrorCode::NinjaexecNotRunning)?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();

    let request = format!(
        "GET /pubkey HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        url_str
    );
    use std::io::{Read, Write as IoWrite};
    let mut stream = tcp;
    stream.write_all(request.as_bytes()).map_err(|_| YodaErrorCode::NinjaexecNotRunning)?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|_| YodaErrorCode::NinjaexecNotRunning)?;
    let response_str = String::from_utf8_lossy(&response);

    if let Some(body_start) = response_str.find("\r\n\r\n") {
        let body = &response_str[body_start + 4..];
        let json: serde_json::Value = serde_json::from_str(body)
            .map_err(|_| YodaErrorCode::NinjaexecNotRunning)?;
        json.get("pubkey_b64")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or(YodaErrorCode::NinjaexecNotRunning)
    } else {
        Err(YodaErrorCode::NinjaexecNotRunning)
    }
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn daemon_submit_yoda_chat(payload_json: &str) -> Result<serde_json::Value, YodaErrorCode> {
    let api_url = daemon_api_url();
    let host_port = api_url.trim_start_matches("http://");
    let tcp = std::net::TcpStream::connect_timeout(
        &host_port.parse().map_err(|_| YodaErrorCode::DaemonNotRunning)?,
        std::time::Duration::from_secs(3),
    ).map_err(|_| YodaErrorCode::DaemonNotRunning)?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(30))).ok();

    let request = format!(
        "POST /yoda/submit HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        host_port, payload_json.len(), payload_json
    );
    use std::io::{Read, Write as IoWrite};
    let mut stream = tcp;
    stream.write_all(request.as_bytes()).map_err(|_| YodaErrorCode::RelayDisconnected)?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|_| YodaErrorCode::YodaTimeout)?;
    let response_str = String::from_utf8_lossy(&response);

    if let Some(body_start) = response_str.find("\r\n\r\n") {
        let body = &response_str[body_start + 4..];
        serde_json::from_str(body).map_err(|_| YodaErrorCode::RelayDisconnected)
    } else {
        Err(YodaErrorCode::RelayDisconnected)
    }
}

fn run_doctor() {
    println!("PlenumNET Yoda — Prerequisite Check");
    println!("====================================");
    println!();
    let mut any_fail = false;
    let mut any_warn = false;

    match query_daemon_status() {
        Ok((rep_c, relay_available)) => {
            println!("[OK]   Daemon reachable — Rep C: {}", rep_c);
            if relay_available {
                println!("[OK]   Relay available — CRS mode or RELAY_URL configured");
            } else {
                println!("[WARN] Relay available — RELAY_URL not set, messages route via daemon only");
                any_warn = true;
            }
        }
        Err(_) => {
            println!("[FAIL] Daemon reachable — not running, start the PlenumNET daemon");
            any_fail = true;
            println!("[FAIL] Relay available — cannot check, daemon not running");
            any_fail = true;
        }
    }

    match query_ninjaexec_status() {
        Ok(_) => {
            println!("[OK]   NinjaExec signing agent — running on port {}", NINJAEXEC_PORT);
            match ninjaexec_pubkey() {
                Ok(pk) => {
                    let pk_short = if pk.len() > 16 { &pk[..16] } else { &pk };
                    println!("[OK]   NinjaExec unlocked — operator key: {}…", pk_short);
                }
                Err(_) => {
                    println!("[WARN] NinjaExec unlocked — could not retrieve public key, agent may be locked");
                    any_warn = true;
                }
            }
        }
        Err(_) => {
            println!("[FAIL] NinjaExec signing agent — not running, run `ninja-exec` to start");
            any_fail = true;
        }
    }

    match (query_daemon_status(), ninjaexec_pubkey()) {
        (Ok(_), Ok(pk)) => {
            let ops_url = format!("{}/api/ops/operators", daemon_api_url());
            match ureq_get_json(&ops_url) {
                Ok(ops_json) => {
                    let operators = ops_json.as_array();
                    let found = operators.map(|arr| {
                        arr.iter().any(|op| {
                            op.get("public_key").and_then(|v| v.as_str()) == Some(&pk)
                                || op.get("publicKey").and_then(|v| v.as_str()) == Some(&pk)
                        })
                    }).unwrap_or(false);
                    if found {
                        let pk_short = if pk.len() > 16 { &pk[..16] } else { &pk };
                        println!("[OK]   Operator key registered — {}… in daemon registry", pk_short);
                    } else {
                        let pk_short = if pk.len() > 16 { &pk[..16] } else { &pk };
                        println!("[FAIL] Operator key registered — {}… not found, register via POST /api/ops/operators", pk_short);
                        any_fail = true;
                    }
                }
                Err(_) => {
                    let pk_short = if pk.len() > 16 { &pk[..16] } else { &pk };
                    println!("[WARN] Operator key registered — cannot query registry, key {}… unverified", pk_short);
                    any_warn = true;
                }
            }
        }
        (Err(_), _) => {
            println!("[FAIL] Operator key registered — daemon not running");
            any_fail = true;
        }
        (_, Err(_)) => {
            println!("[FAIL] Operator key registered — NinjaExec not available");
            any_fail = true;
        }
    }

    let dir = plenumnet_dir();
    if dir.exists() {
        let test_path = dir.join(".yoda-write-test");
        match std::fs::write(&test_path, "test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_path);
                println!("[OK]   Data directory writable — {}/", dir.display());
            }
            Err(_) => {
                println!("[FAIL] Data directory writable — cannot write to {}/", dir.display());
                any_fail = true;
            }
        }
    } else {
        println!("[FAIL] Data directory writable — does not exist: {}/", dir.display());
        any_fail = true;
    }

    {
        let now = current_timestamp_ms();
        let epoch_secs = now / 1000;
        if epoch_secs < 1_700_000_000 || epoch_secs > 2_000_000_000 {
            println!("[FAIL] Clock synchronization — system clock invalid (epoch={}s)", epoch_secs);
            any_fail = true;
        } else {
            println!("[OK]   Clock synchronization — valid, {}ms message window", TIMESTAMP_MAX_AGE_MS);
        }
    }

    {
        let relay_url = std::env::var("RELAY_URL")
            .unwrap_or_else(|_| "https://plenumnet-relay.replit.app".to_string());
        let health_url = format!("{}/health", relay_url.trim_end_matches('/'));
        match ureq_get_json(&health_url) {
            Ok(json) => {
                if let Some(server_ts) = json.get("timestamp").and_then(|t| t.as_u64())
                    .or_else(|| json.get("serverTime").and_then(|t| t.as_u64()))
                    .or_else(|| json.get("time").and_then(|t| t.as_u64()))
                {
                    let local_ts = current_timestamp_ms();
                    let drift_ms = if local_ts > server_ts { local_ts - server_ts } else { server_ts - local_ts };
                    if drift_ms >= 60_000 {
                        println!("[FAIL] Relay clock drift — {}ms exceeds 60s, messages will be rejected", drift_ms);
                        any_fail = true;
                    } else if drift_ms >= 30_000 {
                        println!("[WARN] Relay clock drift — {}ms (30–59s range), messages may be rejected", drift_ms);
                        any_warn = true;
                    } else {
                        println!("[OK]   Relay clock drift — {}ms, within {}ms window", drift_ms, TIMESTAMP_MAX_AGE_MS);
                    }
                } else {
                    println!("[WARN] Relay clock drift — relay responded but no timestamp field");
                    any_warn = true;
                }
            }
            Err(_) => {
                println!("[WARN] Relay clock drift — cannot reach relay server");
                any_warn = true;
            }
        }
    }

    match SessionLock::acquire() {
        Ok(_lock) => println!("[OK]   Session file lock — acquired and released"),
        Err(_) => {
            println!("[FAIL] Session file lock — cannot acquire, another `y` instance may be running");
            any_fail = true;
        }
    }

    println!();
    if any_fail {
        println!("Result: FAIL — resolve the issues above before using `y`");
        std::process::exit(1);
    } else if any_warn {
        println!("Result: PASS with warnings");
        std::process::exit(0);
    } else {
        println!("Result: PASS — all checks OK");
        std::process::exit(0);
    }
}

fn run_status() {
    let daemon_url = daemon_api_url();
    println!("Daemon API: {}", daemon_url);
    println!("NinjaExec:  http://127.0.0.1:{}", NINJAEXEC_PORT);

    match query_daemon_status() {
        Ok((rep_c, relay)) => {
            println!("Daemon:     connected (Rep C: {})", rep_c);
            println!("Relay:      {}", if relay { "connected" } else { "not connected" });
        }
        Err(_) => {
            println!("Daemon:     not running");
        }
    }

    match query_ninjaexec_status() {
        Ok(_) => {
            match ninjaexec_pubkey() {
                Ok(pk) => {
                    let pk_short = if pk.len() > 16 { &pk[..16] } else { &pk };
                    println!("NinjaExec:  unlocked (key: {}…)", pk_short);
                }
                Err(_) => println!("NinjaExec:  running (locked or error)"),
            }
        }
        Err(_) => println!("NinjaExec:  not running"),
    }

    let session_path = session_file_path();
    if session_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&session_path) {
            if let Ok(session) = serde_json::from_str::<SessionFile>(&data) {
                let expired = if session.is_expired() { " (expired)" } else { " (active)" };
                println!(
                    "Session:    {} seq={}{}", session.session_id, session.sequence, expired
                );
            }
        }
    } else {
        println!("Session:    none");
    }
}

fn run_session_list() {
    let mut found_any = false;

    let session_path = session_file_path();
    if session_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&session_path) {
            if let Ok(session) = serde_json::from_str::<SessionFile>(&data) {
                let status = if session.is_expired() { "expired" } else { "active" };
                println!(
                    "  {} — {} messages, {} (current)",
                    session.session_id, session.sequence, status
                );
                found_any = true;
            }
        }
    }

    let audit_path = plenumnet_dir().join("yoda-audit.jsonl");
    if audit_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&audit_path) {
            let mut seen_sessions = std::collections::HashSet::new();
            let current_id = session_path.exists()
                .then(|| std::fs::read_to_string(&session_path).ok())
                .flatten()
                .and_then(|d| serde_json::from_str::<SessionFile>(&d).ok())
                .map(|s| s.session_id);

            for line in data.lines().rev() {
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(sid) = entry.get("session_id").and_then(|v| v.as_str())
                        .or_else(|| entry.get("sessionId").and_then(|v| v.as_str())) {
                        if current_id.as_deref() == Some(sid) { continue; }
                        if seen_sessions.insert(sid.to_string()) {
                            let ts = entry.get("timestamp").and_then(|v| v.as_str()).unwrap_or("unknown");
                            let seq = entry.get("sequence").and_then(|v| v.as_u64()).unwrap_or(0);
                            println!("  {} — seq {}, last seen {}", sid, seq, ts);
                            found_any = true;
                        }
                    }
                }
                if seen_sessions.len() >= 20 { break; }
            }
        }
    }

    if !found_any {
        println!("No sessions found.");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: y <message>");
        eprintln!("       y --status");
        eprintln!("       y --session list");
        eprintln!("       y --session <id>");
        eprintln!("       y --doctor");
        std::process::exit(1);
    }

    if args[0] == "--doctor" {
        run_doctor();
        return;
    }

    if args[0] == "--status" {
        run_status();
        return;
    }

    if args[0] == "--session" {
        if args.len() < 2 {
            eprintln!("Usage: y --session list");
            eprintln!("       y --session <id>");
            std::process::exit(1);
        }
        if args[1] == "list" {
            run_session_list();
            return;
        }
        let resume_id = &args[1];
        let message = if args.len() > 2 {
            args[2..].join(" ")
        } else {
            eprintln!("Usage: y --session <id> <message>");
            std::process::exit(1);
        };
        send_message(&message, Some(resume_id));
        return;
    }

    let message = args.join(" ");

    if message.len() > MAX_MESSAGE_BYTES {
        print_error_and_exit(YodaErrorCode::MessageTooLong);
    }

    send_message(&message, None);
}

fn send_message(message: &str, resume_session: Option<&str>) {
    let _data_dir = match ensure_data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let _lock = match SessionLock::acquire() {
        Ok(l) => l,
        Err(code) => print_error_and_exit(code),
    };

    let (daemon_rep_c, _relay_connected) = match query_daemon_status() {
        Ok(r) => r,
        Err(code) => print_error_and_exit(code),
    };

    let operator_pubkey = match ninjaexec_pubkey() {
        Ok(pk) => pk,
        Err(code) => print_error_and_exit(code),
    };

    let mut session = match load_or_create_session(resume_session) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Session error: {}", e);
            std::process::exit(1);
        }
    };

    let sequence = session.next_sequence();
    let timestamp = current_timestamp_ms();

    if let Err(e) = save_session(&session) {
        eprintln!("Warning: {}", e);
    }

    let signing_payload = build_signing_payload(
        &daemon_rep_c,
        message,
        sequence,
        &session.session_id,
        timestamp,
    );

    print!("[YODA] ...");
    std::io::stdout().flush().ok();

    let (signature, _signer_pk) = match ninjaexec_sign(&signing_payload) {
        Ok(r) => r,
        Err(code) => {
            print!("\r");
            std::io::stdout().flush().ok();
            print_error_and_exit(code);
        }
    };

    print!("\r[YODA] ...");
    std::io::stdout().flush().ok();

    let payload = serde_json::json!({
        "sessionId": session.session_id,
        "timestamp": timestamp,
        "sequence": sequence,
        "message": message,
        "operatorPubkey": operator_pubkey,
        "daemonRepC": daemon_rep_c,
        "signature": signature,
    });
    let payload_str = serde_json::to_string(&payload).unwrap();

    let payload_hash = compute_payload_hash(&signing_payload);

    let audit_entry = YodaAuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        session_id: session.session_id.clone(),
        sequence,
        direction: "outbound".to_string(),
        operator_rep_c: daemon_rep_c.to_string(),
        payload_hash: Some(payload_hash.clone()),
        response_hash: None,
        result: "sent".to_string(),
    };
    write_audit_entry(&audit_file_path(), &audit_entry);

    match daemon_submit_yoda_chat(&payload_str) {
        Ok(response) => {
            print!("\r                    \r");
            std::io::stdout().flush().ok();

            if let Some(err) = response.get("error") {
                let err_msg = err.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                let err_code = err.get("code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("UNKNOWN");
                eprintln!("[YODA] Error from Yoda: {} ({})", err_msg, err_code);

                let response_audit = YodaAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    session_id: session.session_id.clone(),
                    sequence,
                    direction: "inbound".to_string(),
                    operator_rep_c: daemon_rep_c.to_string(),
                    payload_hash: None,
                    response_hash: Some(compute_payload_hash(err_msg.as_bytes())),
                    result: "error".to_string(),
                };
                write_audit_entry(&audit_file_path(), &response_audit);

                let exit = match err_code {
                    "DAEMON_NOT_RUNNING" => YodaErrorCode::DaemonNotRunning.exit_code(),
                    "NINJAEXEC_NOT_RUNNING" => YodaErrorCode::NinjaexecNotRunning.exit_code(),
                    "NINJAEXEC_LOCKED" => YodaErrorCode::NinjaexecLocked.exit_code(),
                    "RELAY_DISCONNECTED" => YodaErrorCode::RelayDisconnected.exit_code(),
                    "SIGNATURE_INVALID" => YodaErrorCode::SignatureInvalid.exit_code(),
                    "OPERATOR_NOT_AUTHORIZED" => YodaErrorCode::OperatorNotAuthorized.exit_code(),
                    "MESSAGE_EXPIRED" => YodaErrorCode::MessageExpired.exit_code(),
                    "SEQUENCE_REPLAY" => YodaErrorCode::SequenceReplay.exit_code(),
                    "RATE_LIMITED" => YodaErrorCode::RateLimited.exit_code(),
                    "MESSAGE_TOO_LONG" => YodaErrorCode::MessageTooLong.exit_code(),
                    "ADDRESS_MISMATCH" => YodaErrorCode::AddressMismatch.exit_code(),
                    "YODA_TIMEOUT" => YodaErrorCode::YodaTimeout.exit_code(),
                    "YODA_UNAVAILABLE" => YodaErrorCode::YodaUnavailable.exit_code(),
                    "CONCURRENT_ACCESS" => YodaErrorCode::ConcurrentAccess.exit_code(),
                    _ => 1,
                };
                std::process::exit(exit);
            } else if let Some(code_str) = response.get("code").and_then(|c| c.as_str()) {
                let msg = response.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Verification failed");
                eprintln!("[YODA] Error: {} ({})", msg, code_str);

                let response_audit = YodaAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    session_id: session.session_id.clone(),
                    sequence,
                    direction: "inbound".to_string(),
                    operator_rep_c: daemon_rep_c.to_string(),
                    payload_hash: None,
                    response_hash: Some(compute_payload_hash(msg.as_bytes())),
                    result: "error".to_string(),
                };
                write_audit_entry(&audit_file_path(), &response_audit);

                let exit = response.get("exitCode")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32;
                std::process::exit(exit);
            } else {
                let content = response.get("content")
                    .and_then(|c| c.as_str())
                    .or_else(|| response.get("message").and_then(|m| m.as_str()))
                    .unwrap_or("Message delivered.");
                println!("[YODA] {}", content);

                let response_audit = YodaAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    session_id: session.session_id.clone(),
                    sequence,
                    direction: "inbound".to_string(),
                    operator_rep_c: daemon_rep_c.to_string(),
                    payload_hash: None,
                    response_hash: Some(compute_payload_hash(content.as_bytes())),
                    result: "delivered".to_string(),
                };
                write_audit_entry(&audit_file_path(), &response_audit);
            }
        }
        Err(code) => {
            println!();
            print_error_and_exit(code);
        }
    }
}
