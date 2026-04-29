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

//! # RepoSync — One-File Encrypted Repo Sync
//!
//! When the Replit-side of the project receives a new commit, the local
//! machine receives a notification over an encrypted TCP tunnel, creates a
//! single-file `git bundle` backup, and runs `git fetch && git merge --ff-only
//! && git push` automatically.
//!
//! ## Two modes (one binary, one file)
//!
//! * `Mode::Server` — runs on Replit (or any machine that hosts the canonical
//!   repo). Watches `git rev-parse HEAD` on disk; when it changes, signs a
//!   `HeadChanged` frame with TL-DSA-87 (optional) and writes it down every
//!   open client tunnel.
//! * `Mode::Client` — runs on the operator's local machine. Holds a TCP tunnel
//!   open to the server, verifies each frame's signature, and on `HeadChanged`
//!   produces a backup bundle then runs the git ops.
//!
//! ## Encryption
//!
//! Each frame is encrypted with a keyed-sponge stream cipher. The default
//! implementation is a self-contained 64-byte permutation suitable for the
//! single-file footprint; for production deployments, swap
//! [`stream_xor_in_place`] for `crate::crypto::keyed_sponge::KeyedTernarySponge`
//! to inherit the full TL-Sponge-385 PQ-security profile.
//!
//! Frames are authenticated with [`crate::crypto::tl_dsa`] (PQ post-quantum
//! signatures) when [`Config::signing_key`] is provided.
//!
//! ## Wire format
//!
//! ```text
//!   ┌──────────┬────────────┬──────────────┬────────────────┐
//!   │ len_be32 │ ciphertext │ sig_len_be16 │ tl_dsa_sig?    │
//!   └──────────┴────────────┴──────────────┴────────────────┘
//! ```
//!
//! `ciphertext = stream_xor(plaintext, key)`. `plaintext` is one of the
//! [`Event`] variants encoded with [`encode_event`]. The signature (if
//! present) is computed over the ciphertext.
//!
//! ## CLI usage (when wired into a `[[bin]]` wrapper)
//!
//! ```bash
//! reposync --server --bind 0.0.0.0:9787 --repo /path/to/Ternary
//! reposync --client --connect host:9787 --repo C:\dev\Ternary --backups C:\dev\backups
//! ```

#![cfg(feature = "std")]

extern crate std;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use std::vec::Vec;
use std::string::{String, ToString};
use std::format;

#[cfg(feature = "std")]
use crate::crypto::tl_dsa::{self, TlDsaPublicKey, TlDsaSecretKey, TlDsaSignature, TlDsaVariant};

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// Whether this instance is the canonical-repo side or the operator-local side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Canonical-repo side (typically the Replit workspace).
    Server,
    /// Operator-local side (the developer's laptop / workstation).
    Client,
}

/// Configuration for one RepoSync instance.
#[derive(Clone)]
pub struct Config {
    pub mode: Mode,
    /// `Server`: bind address (e.g. `0.0.0.0:9787`).
    /// `Client`: server address to connect to.
    pub address: String,
    /// Path to the local git repository.
    pub repo_path: PathBuf,
    /// Where backup `.bundle` files are written (Client only — Server ignores).
    pub backup_dir: PathBuf,
    /// Polling interval (Server: how often to check `git rev-parse HEAD`).
    pub poll_interval_secs: u64,
    /// Pre-shared symmetric key for the keyed-sponge stream cipher.
    /// 48 bytes = 384 bits, matching TL-Sponge-385 capacity.
    pub shared_key: [u8; 48],
    /// Optional signing key (Server signs outgoing frames).
    pub signing_key: Option<TlDsaSecretKey>,
    /// Optional peer public key (Client verifies incoming frames).
    pub peer_public_key: Option<TlDsaPublicKey>,
    /// TL-DSA variant for signatures (default `TlDsaVariant::TlDsa87`).
    pub dsa_variant: TlDsaVariant,
    /// Heartbeat interval (Server emits, Client expects).
    pub heartbeat_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Client,
            address: "127.0.0.1:9787".to_string(),
            repo_path: PathBuf::from("."),
            backup_dir: PathBuf::from("./backups"),
            poll_interval_secs: 5,
            shared_key: [0u8; 48],
            signing_key: None,
            peer_public_key: None,
            dsa_variant: TlDsaVariant::TlDsa87,
            heartbeat_interval_secs: 30,
        }
    }
}

/// Wire events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Server → Client: new commit on the canonical repo.
    HeadChanged {
        commit_id: String,
        short: String,
        message: String,
        timestamp_unix: u64,
    },
    /// Client → Server: backup bundle written.
    BackupAck {
        backup_file: String,
        commit_id: String,
        timestamp_unix: u64,
    },
    /// Client → Server: local repo successfully fast-forwarded to `commit_id`.
    PullDone { commit_id: String },
    /// Client → Server: local commits pushed to origin.
    PushDone { commit_id: String },
    /// Either side: still alive.
    Heartbeat { timestamp_unix: u64 },
}

#[derive(Debug)]
pub enum SyncError {
    Io(std::io::Error),
    Decode(&'static str),
    GitFailed(String),
    SignatureInvalid,
    UnauthenticatedFrame,
}

impl From<std::io::Error> for SyncError {
    fn from(e: std::io::Error) -> Self { SyncError::Io(e) }
}

impl core::fmt::Display for SyncError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SyncError::Io(e) => write!(f, "io: {}", e),
            SyncError::Decode(s) => write!(f, "decode: {}", s),
            SyncError::GitFailed(s) => write!(f, "git: {}", s),
            SyncError::SignatureInvalid => write!(f, "signature invalid"),
            SyncError::UnauthenticatedFrame => write!(f, "frame missing required signature"),
        }
    }
}

pub type Result<T> = core::result::Result<T, SyncError>;

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Run a RepoSync instance. Blocks forever (until network or git failure).
pub fn run(config: Config) -> Result<()> {
    match config.mode {
        Mode::Server => run_server(config),
        Mode::Client => run_client(config),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Server side (canonical repo / Replit)
// ─────────────────────────────────────────────────────────────────────────────

fn run_server(config: Config) -> Result<()> {
    let listener = TcpListener::bind(&config.address)?;
    eprintln!("[reposync:server] listening on {}", config.address);

    let last_commit: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(e) => { eprintln!("[reposync:server] accept error: {}", e); continue; }
        };
        let cfg = config.clone();
        let commit_ref = Arc::clone(&last_commit);
        thread::spawn(move || {
            if let Err(e) = serve_one(stream, cfg, commit_ref) {
                eprintln!("[reposync:server] client session ended: {}", e);
            }
        });
    }
    Ok(())
}

fn serve_one(mut stream: TcpStream, config: Config, last_commit: Arc<Mutex<String>>) -> Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".to_string());
    eprintln!("[reposync:server] tunnel up: {}", peer);

    let mut last_heartbeat = SystemTime::now();

    loop {
        let current = git_head_commit(&config.repo_path).unwrap_or_default();
        let mut last = last_commit.lock().unwrap();

        if !current.is_empty() && *last != current {
            let short = current.chars().take(7).collect::<String>();
            let message = git_head_message(&config.repo_path).unwrap_or_default();
            let event = Event::HeadChanged {
                commit_id: current.clone(),
                short: short.clone(),
                message,
                timestamp_unix: now_unix(),
            };
            let frame = pack_frame(&event, &config);
            stream.write_all(&frame)?;
            stream.flush()?;
            eprintln!("[reposync:server] → {} HeadChanged {}", peer, short);
            *last = current;
        }
        drop(last);

        if SystemTime::now().duration_since(last_heartbeat)
            .map(|d| d.as_secs() >= config.heartbeat_interval_secs).unwrap_or(false)
        {
            let frame = pack_frame(&Event::Heartbeat { timestamp_unix: now_unix() }, &config);
            stream.write_all(&frame)?;
            stream.flush()?;
            last_heartbeat = SystemTime::now();
        }

        thread::sleep(Duration::from_secs(config.poll_interval_secs));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Client side (operator-local)
// ─────────────────────────────────────────────────────────────────────────────

fn run_client(config: Config) -> Result<()> {
    loop {
        eprintln!("[reposync:client] dialing {}", config.address);
        match TcpStream::connect(&config.address) {
            Ok(mut stream) => {
                stream.set_read_timeout(Some(Duration::from_secs(
                    config.heartbeat_interval_secs.saturating_mul(3).max(60),
                )))?;
                eprintln!("[reposync:client] tunnel up");
                if let Err(e) = client_session(&mut stream, &config) {
                    eprintln!("[reposync:client] session ended: {} — reconnecting in 5s", e);
                }
            }
            Err(e) => eprintln!("[reposync:client] dial failed: {} — retrying in 5s", e),
        }
        thread::sleep(Duration::from_secs(5));
    }
}

fn client_session(stream: &mut TcpStream, config: &Config) -> Result<()> {
    loop {
        let event = read_frame(stream, config)?;
        match event {
            Event::HeadChanged { commit_id, short, message, .. } => {
                eprintln!("[reposync:client] ← HeadChanged {} {:?}", short, message);
                match create_backup(&config.repo_path, &config.backup_dir, &commit_id) {
                    Ok(name) => {
                        eprintln!("[reposync:client]   backup: {}", name);
                        let ack = Event::BackupAck { backup_file: name, commit_id: commit_id.clone(), timestamp_unix: now_unix() };
                        let frame = pack_frame(&ack, config);
                        let _ = stream.write_all(&frame);
                    }
                    Err(e) => eprintln!("[reposync:client]   backup failed: {}", e),
                }
                match git_pull_ff(&config.repo_path) {
                    Ok(()) => {
                        eprintln!("[reposync:client]   pulled to {}", short);
                        let frame = pack_frame(&Event::PullDone { commit_id: commit_id.clone() }, config);
                        let _ = stream.write_all(&frame);
                    }
                    Err(e) => eprintln!("[reposync:client]   pull failed: {}", e),
                }
                match git_push(&config.repo_path) {
                    Ok(()) => {
                        eprintln!("[reposync:client]   pushed");
                        let frame = pack_frame(&Event::PushDone { commit_id: commit_id.clone() }, config);
                        let _ = stream.write_all(&frame);
                    }
                    Err(e) => eprintln!("[reposync:client]   push: {}", e),
                }
            }
            Event::Heartbeat { .. } => { /* keep-alive; no action */ }
            other => eprintln!("[reposync:client] (unexpected event: {:?})", other),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Frame packing / unpacking
// ─────────────────────────────────────────────────────────────────────────────

fn pack_frame(event: &Event, config: &Config) -> Vec<u8> {
    let mut plaintext = encode_event(event);
    stream_xor_in_place(&mut plaintext, &config.shared_key);
    let ciphertext = plaintext;

    let sig_bytes: Vec<u8> = if let Some(ref sk) = config.signing_key {
        let payload_i8: Vec<i8> = ciphertext.iter().map(|&b| b as i8).collect();
        match tl_dsa::sign(sk, &payload_i8) {
            Ok(sig) => serialize_signature(&sig),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let mut out = Vec::with_capacity(4 + ciphertext.len() + 2 + sig_bytes.len());
    out.extend_from_slice(&(ciphertext.len() as u32).to_be_bytes());
    out.extend_from_slice(&ciphertext);
    out.extend_from_slice(&(sig_bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(&sig_bytes);
    out
}

fn read_frame(stream: &mut TcpStream, config: &Config) -> Result<Event> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let body_len = u32::from_be_bytes(len_buf) as usize;
    if body_len > 1_048_576 {
        return Err(SyncError::Decode("frame body > 1 MiB rejected"));
    }
    let mut ciphertext = vec_zeros(body_len);
    stream.read_exact(&mut ciphertext)?;

    let mut sig_len_buf = [0u8; 2];
    stream.read_exact(&mut sig_len_buf)?;
    let sig_len = u16::from_be_bytes(sig_len_buf) as usize;
    let mut sig_bytes = vec_zeros(sig_len);
    if sig_len > 0 { stream.read_exact(&mut sig_bytes)?; }

    if let Some(ref pk) = config.peer_public_key {
        if sig_bytes.is_empty() { return Err(SyncError::UnauthenticatedFrame); }
        let payload_i8: Vec<i8> = ciphertext.iter().map(|&b| b as i8).collect();
        let sig = deserialize_signature(&sig_bytes, config.dsa_variant)
            .ok_or(SyncError::SignatureInvalid)?;
        let ok = tl_dsa::verify(pk, &payload_i8, &sig).unwrap_or(false);
        if !ok { return Err(SyncError::SignatureInvalid); }
    }

    let mut plaintext = ciphertext;
    stream_xor_in_place(&mut plaintext, &config.shared_key);
    decode_event(&plaintext).ok_or(SyncError::Decode("event payload"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Event codec
// ─────────────────────────────────────────────────────────────────────────────

const TAG_HEAD_CHANGED: u8 = 1;
const TAG_BACKUP_ACK:   u8 = 2;
const TAG_PULL_DONE:    u8 = 3;
const TAG_PUSH_DONE:    u8 = 4;
const TAG_HEARTBEAT:    u8 = 5;

fn encode_event(event: &Event) -> Vec<u8> {
    let mut v = Vec::new();
    match event {
        Event::HeadChanged { commit_id, short, message, timestamp_unix } => {
            v.push(TAG_HEAD_CHANGED);
            v.extend_from_slice(&timestamp_unix.to_be_bytes());
            push_str(&mut v, commit_id);
            push_str(&mut v, short);
            push_str(&mut v, message);
        }
        Event::BackupAck { backup_file, commit_id, timestamp_unix } => {
            v.push(TAG_BACKUP_ACK);
            v.extend_from_slice(&timestamp_unix.to_be_bytes());
            push_str(&mut v, commit_id);
            push_str(&mut v, backup_file);
        }
        Event::PullDone { commit_id } => { v.push(TAG_PULL_DONE); push_str(&mut v, commit_id); }
        Event::PushDone { commit_id } => { v.push(TAG_PUSH_DONE); push_str(&mut v, commit_id); }
        Event::Heartbeat { timestamp_unix } => {
            v.push(TAG_HEARTBEAT);
            v.extend_from_slice(&timestamp_unix.to_be_bytes());
        }
    }
    v
}

fn decode_event(buf: &[u8]) -> Option<Event> {
    let mut p = 0;
    let kind = *buf.get(p)?; p += 1;
    match kind {
        TAG_HEAD_CHANGED => {
            let ts = read_u64(buf, &mut p)?;
            let commit_id = read_str(buf, &mut p)?;
            let short = read_str(buf, &mut p)?;
            let message = read_str(buf, &mut p)?;
            Some(Event::HeadChanged { commit_id, short, message, timestamp_unix: ts })
        }
        TAG_BACKUP_ACK => {
            let ts = read_u64(buf, &mut p)?;
            let commit_id = read_str(buf, &mut p)?;
            let backup_file = read_str(buf, &mut p)?;
            Some(Event::BackupAck { backup_file, commit_id, timestamp_unix: ts })
        }
        TAG_PULL_DONE => Some(Event::PullDone { commit_id: read_str(buf, &mut p)? }),
        TAG_PUSH_DONE => Some(Event::PushDone { commit_id: read_str(buf, &mut p)? }),
        TAG_HEARTBEAT => Some(Event::Heartbeat { timestamp_unix: read_u64(buf, &mut p)? }),
        _ => None,
    }
}

fn push_str(v: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    v.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    v.extend_from_slice(bytes);
}

fn read_str(buf: &[u8], p: &mut usize) -> Option<String> {
    let len = u16::from_be_bytes(buf.get(*p..*p + 2)?.try_into().ok()?) as usize;
    *p += 2;
    let bytes = buf.get(*p..*p + len)?;
    *p += len;
    Some(String::from_utf8_lossy(bytes).into_owned())
}

fn read_u64(buf: &[u8], p: &mut usize) -> Option<u64> {
    let v = u64::from_be_bytes(buf.get(*p..*p + 8)?.try_into().ok()?);
    *p += 8;
    Some(v)
}

fn vec_zeros(n: usize) -> Vec<u8> { let mut v = Vec::with_capacity(n); v.resize(n, 0); v }

// ─────────────────────────────────────────────────────────────────────────────
// Stream cipher (self-contained 48-byte-key permutation)
//
// NOTE: this is a deliberately simple keyed permutation that keeps the file
// self-contained. For production, replace the body of `stream_xor_in_place`
// with a call into `crate::crypto::keyed_sponge::KeyedTernarySponge` to
// inherit the full TL-Sponge-385 PQ profile (kept identical-shaped for an
// in-place swap).
// ─────────────────────────────────────────────────────────────────────────────

fn stream_xor_in_place(data: &mut [u8], key: &[u8; 48]) {
    let mut state = [0u8; 64];
    state[..48].copy_from_slice(key);
    let mut counter: u64 = 0;
    for chunk in data.chunks_mut(64) {
        let cb = counter.to_be_bytes();
        for i in 0..8usize { state[48 + i] = cb[i]; }
        let mut perm = [0u8; 64];
        for i in 0..64usize {
            let a = state[(i.wrapping_mul(13usize).wrapping_add(7)) % 64];
            let b = state[(i.wrapping_mul(31usize).wrapping_add(11)) % 64];
            perm[i] = a.wrapping_add(b ^ 0xA5).rotate_left(((i as u32) & 7) + 1);
        }
        for round in 0..3usize {
            for i in 0..64usize {
                let j = (i + 17 + round) % 64;
                perm[i] = perm[i].wrapping_add(perm[j] ^ ((round as u8).wrapping_mul(0x5B)));
            }
        }
        for (i, b) in chunk.iter_mut().enumerate() { *b ^= perm[i]; }
        counter = counter.wrapping_add(1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TL-DSA signature serialization (variant-dependent fixed size)
// ─────────────────────────────────────────────────────────────────────────────

fn serialize_signature(sig: &TlDsaSignature) -> Vec<u8> {
    sig.to_bytes()
}

fn deserialize_signature(bytes: &[u8], _variant: TlDsaVariant) -> Option<TlDsaSignature> {
    TlDsaSignature::from_bytes(bytes).ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// Git subprocess helpers
// ─────────────────────────────────────────────────────────────────────────────

fn git_head_commit(repo: &PathBuf) -> Result<String> {
    let out = Command::new("git").arg("-C").arg(repo).args(["rev-parse", "HEAD"]).output()?;
    if !out.status.success() {
        return Err(SyncError::GitFailed(String::from_utf8_lossy(&out.stderr).into_owned()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_head_message(repo: &PathBuf) -> Result<String> {
    let out = Command::new("git").arg("-C").arg(repo).args(["log", "-1", "--format=%s"]).output()?;
    if !out.status.success() {
        return Err(SyncError::GitFailed(String::from_utf8_lossy(&out.stderr).into_owned()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn create_backup(repo: &PathBuf, backup_dir: &PathBuf, commit_id: &str) -> Result<String> {
    std::fs::create_dir_all(backup_dir)?;
    let ts = now_unix();
    let short = commit_id.chars().take(8).collect::<String>();
    let name = format!("repo-{}-{}.bundle", ts, short);
    let path = backup_dir.join(&name);
    let status = Command::new("git").arg("-C").arg(repo)
        .args(["bundle", "create"]).arg(&path).arg("--all").status()?;
    if !status.success() {
        return Err(SyncError::GitFailed(format!("bundle create exit {}", status)));
    }
    Ok(name)
}

fn git_pull_ff(repo: &PathBuf) -> Result<()> {
    let s1 = Command::new("git").arg("-C").arg(repo).args(["fetch", "origin"]).status()?;
    if !s1.success() { return Err(SyncError::GitFailed("fetch failed".to_string())); }
    let s2 = Command::new("git").arg("-C").arg(repo)
        .args(["merge", "--ff-only", "origin/main"]).status()?;
    if !s2.success() { return Err(SyncError::GitFailed("ff-merge failed (rebase manually)".to_string())); }
    Ok(())
}

fn git_push(repo: &PathBuf) -> Result<()> {
    let s = Command::new("git").arg("-C").arg(repo).args(["push", "origin", "main"]).status()?;
    if !s.success() { return Err(SyncError::GitFailed("push declined (auth or protected branch)".to_string())); }
    Ok(())
}

fn now_unix() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_head_changed() {
        let event = Event::HeadChanged {
            commit_id: "abc1234567890".to_string(),
            short: "abc1234".to_string(),
            message: "fix license headers".to_string(),
            timestamp_unix: 1714421234,
        };
        let bytes = encode_event(&event);
        let decoded = decode_event(&bytes).expect("decode");
        match decoded {
            Event::HeadChanged { commit_id, short, message, timestamp_unix } => {
                assert_eq!(commit_id, "abc1234567890");
                assert_eq!(short, "abc1234");
                assert_eq!(message, "fix license headers");
                assert_eq!(timestamp_unix, 1714421234);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn round_trip_heartbeat() {
        let bytes = encode_event(&Event::Heartbeat { timestamp_unix: 42 });
        match decode_event(&bytes).unwrap() {
            Event::Heartbeat { timestamp_unix } => assert_eq!(timestamp_unix, 42),
            _ => panic!(),
        }
    }

    #[test]
    fn cipher_is_involution() {
        let key = [7u8; 48];
        let original = b"the quick brown fox jumps over 13 lazy dogs!".to_vec();
        let mut buf = original.clone();
        stream_xor_in_place(&mut buf, &key);
        assert_ne!(buf, original);
        stream_xor_in_place(&mut buf, &key);
        assert_eq!(buf, original);
    }

    #[test]
    fn frame_pack_unpack_no_signature() {
        let mut config = Config::default();
        config.shared_key = [9u8; 48];
        let event = Event::Heartbeat { timestamp_unix: 100 };
        let frame = pack_frame(&event, &config);
        let body_len = u32::from_be_bytes(frame[0..4].try_into().unwrap()) as usize;
        let mut ct = frame[4..4 + body_len].to_vec();
        stream_xor_in_place(&mut ct, &config.shared_key);
        let decoded = decode_event(&ct).expect("decode");
        match decoded {
            Event::Heartbeat { timestamp_unix } => assert_eq!(timestamp_unix, 100),
            _ => panic!(),
        }
    }
}
