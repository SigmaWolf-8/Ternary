// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// NinjaExec — PlenumNET Local Signing Agent v1.0.0
//
// The ssh-agent of PlenumNET: holds the operator's TL-DSA private key
// in an encrypted keystore on the local machine and exposes a
// localhost-only HTTP signing API on 127.0.0.1:21027.
//
// The key never leaves the process. The process never leaves the machine.
// Everything in the stack trusts it because it trusts TL-DSA.

mod audit;
mod cli;
mod config;
mod confirm;
mod keystore;
mod server;
mod signing_engine;

use std::io::{self, BufRead, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::audit::{AuditEntry, AuditLog};
use crate::cli::Command;
use crate::confirm::ConfirmationQueue;
use crate::config::NinjaExecConfig;
use crate::keystore::Keystore;
use crate::server::{AppState, RateLimiter};

fn prompt_passphrase(prompt: &str) -> String {
    eprint!("{}", prompt);
    let _ = io::stderr().flush();

    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let stdin_fd = io::stdin().as_raw_fd();
        unsafe {
            let mut termios: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(stdin_fd, &mut termios) == 0 {
                let old_termios = termios;
                termios.c_lflag &= !libc::ECHO;
                libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios);
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line).unwrap_or(0);
                libc::tcsetattr(stdin_fd, libc::TCSANOW, &old_termios);
                eprintln!();
                return line.trim().to_string();
            }
        }
    }

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap_or(0);
    line.trim().to_string()
}

fn resolve_data_dir(override_dir: Option<std::path::PathBuf>) -> std::path::PathBuf {
    override_dir.unwrap_or_else(keystore::default_data_dir)
}

fn copy_to_clipboard(text: &str) {
    #[cfg(windows)]
    {
        let mut child = match std::process::Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                eprintln!("[NinjaExec] Could not access clipboard. Output printed instead:");
                println!("{}", text);
                return;
            }
        };
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        eprintln!("[NinjaExec] Operator identity copied to clipboard.");
    }

    #[cfg(unix)]
    {
        let clipboard_cmds = ["xclip -selection clipboard", "xsel --clipboard --input", "pbcopy"];
        for cmd_str in &clipboard_cmds {
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if let Ok(mut child) = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(ref mut stdin) = child.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    eprintln!("[NinjaExec] Operator identity copied to clipboard.");
                    return;
                }
            }
        }
        eprintln!("[NinjaExec] No clipboard utility found. Output printed instead:");
        println!("{}", text);
    }
}

fn http_get(port: u16, path: &str) -> Result<String, String> {
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
    let request = format!("GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", path, port);
    stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;
    extract_http_body(&response)
}

fn http_post(port: u16, path: &str, body: &str) -> Result<String, String> {
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, port, body.len(), body
    );
    stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;
    extract_http_body(&response)
}

fn extract_http_body(response: &str) -> Result<String, String> {
    if let Some(idx) = response.find("\r\n\r\n") {
        Ok(response[idx + 4..].to_string())
    } else if let Some(idx) = response.find("\n\n") {
        Ok(response[idx + 2..].to_string())
    } else {
        Err("Invalid HTTP response".to_string())
    }
}

fn main() {
    let command = cli::parse_args();

    match command {
        Command::Version => {
            println!("NinjaExec v{}", env!("CARGO_PKG_VERSION"));
            println!("PlenumNET Local Signing Agent");
            println!("TL-DSA-87 (Level 5 post-quantum security)");
            println!("Copyright (c) 2025-2026 Capomastro Holdings Ltd.");
        }

        Command::Init { data_dir } => {
            let dir = resolve_data_dir(data_dir);
            let mut ks = Keystore::new(dir.clone());

            if ks.exists() {
                eprintln!("[NinjaExec] Keystore already exists at {}", ks.keystore_path().display());
                eprintln!("[NinjaExec] To create a new keystore, remove the existing one first.");
                std::process::exit(1);
            }

            let passphrase = if let Ok(pp) = std::env::var("PLENUM_PASSPHRASE") {
                pp
            } else {
                let pp = prompt_passphrase("Enter passphrase (min 12 characters): ");
                let pp2 = prompt_passphrase("Confirm passphrase: ");
                if pp != pp2 {
                    eprintln!("[NinjaExec] Passphrases do not match.");
                    std::process::exit(1);
                }
                pp
            };

            match ks.create(&passphrase) {
                Ok(()) => {
                    let pk = ks.public_key().unwrap();
                    let pk_b64 = signing_engine::export_pubkey_b64(pk);
                    let fp = signing_engine::fingerprint(pk);

                    println!("[NinjaExec] Keystore created at {}", ks.keystore_path().display());
                    println!("[NinjaExec] Public key: {}", pk_b64);
                    println!("[NinjaExec] Fingerprint: {}", fp);

                    config::NinjaExecConfig::save_default(&dir);
                    let token = config::NinjaExecConfig::generate_confirm_token(&dir);
                    println!("[NinjaExec] Confirm token generated (stored in ninja-exec.json)");
                    println!("[NinjaExec] Tray/UI uses this token to approve signing requests");
                    println!("[NinjaExec] Token: {}", token);

                    let audit_log = AuditLog::new(&dir);
                    audit_log.append(&AuditEntry {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        operation: "init".to_string(),
                        context: Some("Keystore created".to_string()),
                        payload_hash: None,
                        origin: None,
                        result: "created".to_string(),
                        confirmation: "auto".to_string(),
                        duration_ms: 0,
                    });
                }
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to create keystore: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Command::Pubkey { data_dir } => {
            let dir = resolve_data_dir(data_dir);
            let mut ks = Keystore::new(dir);
            match ks.load_public_key_only() {
                Ok(()) => {
                    let pk = ks.public_key().unwrap();
                    println!("{}", signing_engine::export_pubkey_b64(pk));
                }
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to read keystore: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Command::Fingerprint { data_dir } => {
            let dir = resolve_data_dir(data_dir);
            let mut ks = Keystore::new(dir);
            match ks.load_public_key_only() {
                Ok(()) => {
                    let pk = ks.public_key().unwrap();
                    println!("{}", signing_engine::fingerprint(pk));
                }
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to read keystore: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Command::ExportOperator { data_dir, clipboard } => {
            let dir = resolve_data_dir(data_dir);
            let mut ks = Keystore::new(dir);
            match ks.load_public_key_only() {
                Ok(()) => {
                    let pk = ks.public_key().unwrap();
                    let pk_b64 = signing_engine::export_pubkey_b64(pk);
                    let fp = signing_engine::fingerprint(pk);
                    let hostname = std::env::var("COMPUTERNAME")
                        .or_else(|_| std::env::var("HOSTNAME"))
                        .unwrap_or_else(|_| "unknown".to_string());

                    let export = serde_json::json!({
                        "name": format!("operator@{}", hostname),
                        "pubkey_b64": pk_b64,
                        "fingerprint": fp,
                        "algorithm": "TL-DSA-87",
                        "created": chrono::Utc::now().to_rfc3339(),
                        "agent": "NinjaExec",
                        "agent_version": env!("CARGO_PKG_VERSION")
                    });

                    let output = serde_json::to_string_pretty(&export).unwrap();

                    if clipboard {
                        copy_to_clipboard(&output);
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to read keystore: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Command::Status { port } => {
            let p = port.unwrap_or(config::DEFAULT_PORT);
            match http_get(p, "/status") {
                Ok(body) => println!("{}", body),
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to reach agent on port {}: {}", p, e);
                    std::process::exit(1);
                }
            }
        }

        Command::Lock { port } => {
            let p = port.unwrap_or(config::DEFAULT_PORT);
            match http_post(p, "/lock", "{}") {
                Ok(body) => println!("{}", body),
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to reach agent on port {}: {}", p, e);
                    std::process::exit(1);
                }
            }
        }

        Command::Unlock { port } => {
            let p = port.unwrap_or(config::DEFAULT_PORT);
            let passphrase = if let Ok(pp) = std::env::var("PLENUM_PASSPHRASE") {
                pp
            } else {
                prompt_passphrase("Passphrase: ")
            };
            let body = serde_json::json!({ "passphrase": passphrase }).to_string();
            match http_post(p, "/unlock", &body) {
                Ok(resp) => println!("{}", resp),
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to reach agent on port {}: {}", p, e);
                    std::process::exit(1);
                }
            }
        }

        Command::SignFile { file, data_dir } => {
            if file.is_empty() {
                eprintln!("Usage: ninja-exec sign <file>");
                std::process::exit(1);
            }

            let dir = resolve_data_dir(data_dir);
            let mut ks = Keystore::new(dir.clone());

            if !ks.exists() {
                eprintln!("[NinjaExec] No keystore found. Run 'ninja-exec init' first.");
                std::process::exit(1);
            }

            let passphrase = if let Ok(pp) = std::env::var("PLENUM_PASSPHRASE") {
                pp
            } else {
                prompt_passphrase("Passphrase: ")
            };
            if let Err(e) = ks.open(&passphrase) {
                eprintln!("[NinjaExec] Failed to unlock keystore: {}", e);
                std::process::exit(1);
            }

            let payload = match std::fs::read(&file) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to read {}: {}", file, e);
                    std::process::exit(1);
                }
            };

            let sk = ks.secret_key().unwrap();
            let signature = signing_engine::sign(sk, &payload);

            use base64::Engine;
            println!("{}", base64::engine::general_purpose::STANDARD.encode(&signature));

            let audit_log = AuditLog::new(&dir);
            audit_log.append(&AuditEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: "sign".to_string(),
                context: Some(format!("file: {}", file)),
                payload_hash: Some(audit::hash_payload(&payload)),
                origin: Some("cli".to_string()),
                result: "signed".to_string(),
                confirmation: "auto".to_string(),
                duration_ms: 0,
            });
        }

        Command::VerifyFile { file, signature, data_dir } => {
            if file.is_empty() || signature.is_empty() {
                eprintln!("Usage: ninja-exec verify <file> <signature_b64>");
                std::process::exit(1);
            }

            let dir = resolve_data_dir(data_dir);
            let mut ks = Keystore::new(dir);

            if let Err(e) = ks.load_public_key_only() {
                eprintln!("[NinjaExec] Failed to read keystore: {}", e);
                std::process::exit(1);
            }

            let payload = match std::fs::read(&file) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("[NinjaExec] Failed to read {}: {}", file, e);
                    std::process::exit(1);
                }
            };

            use base64::Engine;
            let sig_bytes = match base64::engine::general_purpose::STANDARD.decode(&signature) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("[NinjaExec] Invalid base64 signature");
                    std::process::exit(1);
                }
            };

            let pk = ks.public_key().unwrap();
            let valid = signing_engine::verify(pk, &payload, &sig_bytes);

            if valid {
                println!("VALID");
                std::process::exit(0);
            } else {
                println!("INVALID");
                std::process::exit(1);
            }
        }

        Command::Run { port, headless, data_dir } => {
            let dir = resolve_data_dir(data_dir);
            let cfg = NinjaExecConfig::load(&dir);
            let actual_port = port.unwrap_or(cfg.port);

            let mut ks = Keystore::new(dir.clone());

            if !ks.exists() {
                eprintln!("[NinjaExec] No keystore found at {}", ks.keystore_path().display());
                eprintln!("[NinjaExec] Run 'ninja-exec init' to create one.");
                std::process::exit(1);
            }

            let passphrase = if let Ok(pp) = std::env::var("PLENUM_PASSPHRASE") {
                pp
            } else {
                prompt_passphrase("Passphrase: ")
            };

            if let Err(e) = ks.open(&passphrase) {
                eprintln!("[NinjaExec] Failed to unlock keystore: {}", e);
                std::process::exit(1);
            }

            let pk = ks.public_key().unwrap();
            let fp = signing_engine::fingerprint(pk);

            println!("╔══════════════════════════════════════════════════╗");
            println!("║         NinjaExec — PlenumNET Signing Agent     ║");
            println!("║         v{}                                  ║", env!("CARGO_PKG_VERSION"));
            println!("╠══════════════════════════════════════════════════╣");
            println!("║  Fingerprint: {}  ║", &fp[..47]);
            println!("║  Listening:   127.0.0.1:{}                  ║", actual_port);
            println!("║  Algorithm:   TL-DSA-87 (Level 5 PQ)            ║");
            if headless {
                println!("║  Mode:        HEADLESS (auto-approve all)       ║");
            } else {
                println!("║  Mode:        Interactive                       ║");
            }
            println!("╚══════════════════════════════════════════════════╝");

            if headless {
                eprintln!("WARNING: Headless mode — all signing requests will be auto-approved.");
                eprintln!("Do not use in environments where browser tabs may be compromised.");
            }

            let audit_log = AuditLog::new(&dir);

            if headless {
                audit_log.append(&AuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "startup".to_string(),
                    context: Some("HEADLESS MODE — all requests auto-approved".to_string()),
                    payload_hash: None,
                    origin: None,
                    result: "started".to_string(),
                    confirmation: "auto".to_string(),
                    duration_ms: 0,
                });
            } else {
                audit_log.append(&AuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "startup".to_string(),
                    context: Some("Interactive mode".to_string()),
                    payload_hash: None,
                    origin: None,
                    result: "started".to_string(),
                    confirmation: "auto".to_string(),
                    duration_ms: 0,
                });
            }

            let rate_limit = cfg.rate_limit_per_minute;
            let state = AppState {
                keystore: Arc::new(Mutex::new(ks)),
                audit_log: Arc::new(Mutex::new(audit_log)),
                config: Arc::new(cfg),
                start_time: Instant::now(),
                signs_this_session: Arc::new(Mutex::new(0)),
                headless,
                rate_limiter: Arc::new(Mutex::new(RateLimiter::new(rate_limit))),
                confirmation_queue: Arc::new(Mutex::new(ConfirmationQueue::new())),
            };

            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async {
                if let Err(e) = server::serve(state, actual_port).await {
                    eprintln!("[NinjaExec] Server error: {}", e);
                    std::process::exit(1);
                }
            });
        }
    }
}
