// Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.
// PlenumNET RepoSync — single-click client launcher.
//
// Reads %APPDATA%\PlenumNET-RepoSync\config.toml on startup, then hands off
// to the kernel `repo_sync` module in Client mode. No protocol logic lives
// here — everything is in src/kernel/src/repo_sync.rs.

use anyhow::{Context, Result};
use plenumnet_kernel::repo_sync::{self, Config, Mode};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct FileConfig {
    /// "client" or "server" (defaults to "client" — that's what the MSI installs)
    #[serde(default = "default_mode")]
    mode: String,
    /// "host:port" — your Replit endpoint (e.g. "your-repl.replit.app:9787")
    address: String,
    /// Local clone path (e.g. "C:\\dev\\Ternary")
    repo_path: String,
    /// Where to drop encrypted bundle backups before each pull
    #[serde(default = "default_backup_dir")]
    backup_dir: String,
    /// 96-hex-char (48 byte) pre-shared key, generated at first run
    shared_key_hex: String,
    /// How often (seconds) to poll for HEAD changes
    #[serde(default = "default_poll")]
    poll_interval_secs: u64,
    /// How often (seconds) to send keepalive heartbeats
    #[serde(default = "default_heartbeat")]
    heartbeat_interval_secs: u64,
}

fn default_mode() -> String { "client".to_string() }
fn default_backup_dir() -> String {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PlenumNET-RepoSync")
        .join("backups");
    base.to_string_lossy().into_owned()
}
fn default_poll() -> u64 { 5 }
fn default_heartbeat() -> u64 { 30 }

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PlenumNET-RepoSync")
        .join("config.toml")
}

fn parse_hex_key(hex: &str) -> Result<[u8; 48]> {
    let cleaned: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    anyhow::ensure!(
        cleaned.len() == 96,
        "shared_key_hex must be 96 hex characters (48 bytes); got {}",
        cleaned.len()
    );
    let mut out = [0u8; 48];
    for i in 0..48 {
        let byte = u8::from_str_radix(&cleaned[i * 2..i * 2 + 2], 16)
            .with_context(|| format!("invalid hex at byte {}", i))?;
        out[i] = byte;
    }
    Ok(out)
}

fn main() -> Result<()> {
    let cfg_path = config_path();
    let raw = fs::read_to_string(&cfg_path)
        .with_context(|| format!("Cannot read config at {}", cfg_path.display()))?;
    let file: FileConfig = toml::from_str(&raw).context("config.toml is not valid TOML")?;

    let mode = match file.mode.to_lowercase().as_str() {
        "server" => Mode::Server,
        _ => Mode::Client,
    };

    let mut cfg = Config::default();
    cfg.mode = mode;
    cfg.address = file.address;
    cfg.repo_path = PathBuf::from(file.repo_path);
    cfg.backup_dir = PathBuf::from(file.backup_dir);
    cfg.shared_key = parse_hex_key(&file.shared_key_hex)?;
    cfg.poll_interval_secs = file.poll_interval_secs;
    cfg.heartbeat_interval_secs = file.heartbeat_interval_secs;

    eprintln!(
        "PlenumNET RepoSync starting | mode={:?} addr={} repo={}",
        cfg.mode,
        cfg.address,
        cfg.repo_path.display()
    );

    repo_sync::run(cfg).map_err(|e| anyhow::anyhow!("RepoSync stopped: {:?}", e))
}
