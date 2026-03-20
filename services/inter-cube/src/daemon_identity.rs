// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Daemon Identity — PT26-DSA Persistent Identity Management
//!
//! Handles on-disk master secret persistence and PT26-DSA keypair
//! derivation for the inter-cube daemon.
//!
//! ## File Layout
//!
//! ```text
//! ~/.plenumnet/identity/
//! └── master.key         92 bytes: salt(16) ‖ nonce(12) ‖ ct(48) ‖ tag(16)
//! ```
//!
//! Override the directory with `CUBE_IDENTITY_DIR`.
//!
//! ## Passphrase
//!
//! The encryption passphrase defaults to `CUBE_IDENTITY_PASSPHRASE` env var.
//! If absent, a deterministic passphrase is derived from the hostname +
//! a hardcoded domain separator (suitable for unattended daemon operation;
//! production deployments should set the env var or use TPM sealing).

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::identity::{
    MasterSecret, encrypt_master_secret, decrypt_master_secret,
    ENCRYPTED_BLOB_LEN, SALVI_EPOCH_UNIX,
};
use crate::key_rotation::ROTATION_PERIOD_SECS;

const DEFAULT_IDENTITY_DIR: &str = ".plenumnet/identity";
const MASTER_KEY_FILE: &str = "master.key";
const PASSPHRASE_DOMAIN: &[u8] = b"PlenumNET-DAEMON-PASSPHRASE-v1";

pub fn identity_dir() -> PathBuf {
    if let Ok(dir) = env::var("CUBE_IDENTITY_DIR") {
        PathBuf::from(dir)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(DEFAULT_IDENTITY_DIR)
    } else {
        PathBuf::from(DEFAULT_IDENTITY_DIR)
    }
}

pub fn encryption_passphrase() -> Vec<u8> {
    if let Ok(pp) = env::var("CUBE_IDENTITY_PASSPHRASE") {
        return pp.into_bytes();
    }
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "plenumnet-node".to_string());
    let mut material = Vec::with_capacity(PASSPHRASE_DOMAIN.len() + hostname.len());
    material.extend_from_slice(PASSPHRASE_DOMAIN);
    material.extend_from_slice(hostname.as_bytes());
    ternary_math::sponge::derive_key(PASSPHRASE_DOMAIN, &material, 32)
}

pub fn load_or_generate_master_secret() -> MasterSecret {
    let dir = identity_dir();
    let key_path = dir.join(MASTER_KEY_FILE);
    let passphrase = encryption_passphrase();

    if key_path.exists() {
        match fs::read(&key_path) {
            Ok(blob) if blob.len() == ENCRYPTED_BLOB_LEN => {
                match decrypt_master_secret(&blob, &passphrase) {
                    Ok(secret) => {
                        println!("[IDENTITY] Loaded master secret from {}", key_path.display());
                        return secret;
                    }
                    Err(e) => {
                        println!("[IDENTITY] WARNING: Failed to decrypt {}: {}", key_path.display(), e);
                        println!("[IDENTITY] Generating fresh master secret");
                    }
                }
            }
            Ok(blob) => {
                println!(
                    "[IDENTITY] WARNING: Invalid blob size ({} bytes, expected {})",
                    blob.len(), ENCRYPTED_BLOB_LEN
                );
            }
            Err(e) => {
                println!("[IDENTITY] WARNING: Could not read {}: {}", key_path.display(), e);
            }
        }
    }

    let secret = MasterSecret::generate().expect("Failed to generate master secret");
    save_master_secret(&secret, &passphrase, &dir, &key_path);
    secret
}

pub fn save_master_secret(
    secret: &MasterSecret,
    passphrase: &[u8],
    dir: &PathBuf,
    key_path: &PathBuf,
) {
    if let Err(e) = fs::create_dir_all(dir) {
        println!("[IDENTITY] WARNING: Could not create {}: {}", dir.display(), e);
        return;
    }

    match encrypt_master_secret(secret, passphrase) {
        Ok(blob) => {
            match fs::write(key_path, &blob) {
                Ok(_) => {
                    println!("[IDENTITY] Master secret encrypted and saved to {}", key_path.display());
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(key_path, fs::Permissions::from_mode(0o600));
                    }
                }
                Err(e) => {
                    println!("[IDENTITY] WARNING: Could not write {}: {}", key_path.display(), e);
                }
            }
        }
        Err(e) => {
            println!("[IDENTITY] WARNING: Encryption failed: {}", e);
        }
    }
}

pub struct DaemonIdentity {
    pub master_secret: MasterSecret,
    pub pt26_pk: ternary_math::pt26_dsa::PublicKey,
    pub pt26_sk: ternary_math::pt26_dsa::SecretKey,
    pub pk_hex: String,
}

impl DaemonIdentity {
    pub fn init() -> Self {
        let master_secret = load_or_generate_master_secret();

        let mut addr = [0u8; 13];
        let seed = ternary_math::sponge::derive_key(
            b"PlenumNET-PT26-INIT-ADDR",
            master_secret.as_bytes(),
            13,
        );
        for i in 0..13 {
            addr[i] = (seed[i] % 3) + 1;
        }

        let (pk, sk) = ternary_math::pt26_dsa::keygen(&addr, master_secret.as_bytes());
        let pk_bytes = pk.to_bytes();
        let pk_hex: String = pk_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        println!("[IDENTITY] PT26-DSA keypair derived (pk: {}...)", &pk_hex[..16]);

        DaemonIdentity {
            master_secret,
            pt26_pk: pk,
            pt26_sk: sk,
            pk_hex,
        }
    }

    pub fn current_radian_epoch(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now < SALVI_EPOCH_UNIX {
            return 0;
        }
        (now - SALVI_EPOCH_UNIX) / ROTATION_PERIOD_SECS
    }
}

pub fn run_keygen() {
    println!("[KEYGEN] Generating PT26-DSA identity keypair...");
    let identity = DaemonIdentity::init();
    println!();
    println!("PT26-DSA Public Key (hex): {}", identity.pk_hex);
    println!("Key file: {}", identity_dir().join(MASTER_KEY_FILE).display());
    println!();
    println!("Set CUBE_IDENTITY_DIR to override the storage location.");
    println!("Set CUBE_IDENTITY_PASSPHRASE to override the encryption passphrase.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_round_trip_generate_load() {
        let dir = env::temp_dir().join("plenumnet-test-identity");
        let _ = fs::remove_dir_all(&dir);

        env::set_var("CUBE_IDENTITY_DIR", dir.to_str().unwrap());
        env::set_var("CUBE_IDENTITY_PASSPHRASE", "test-passphrase-42");

        let s1 = load_or_generate_master_secret();
        let s2 = load_or_generate_master_secret();

        assert_eq!(s1.as_bytes(), s2.as_bytes());

        let _ = fs::remove_dir_all(&dir);
        env::remove_var("CUBE_IDENTITY_DIR");
        env::remove_var("CUBE_IDENTITY_PASSPHRASE");
    }

    #[test]
    fn test_daemon_identity_init() {
        let dir = env::temp_dir().join("plenumnet-test-daemon-id");
        let _ = fs::remove_dir_all(&dir);

        env::set_var("CUBE_IDENTITY_DIR", dir.to_str().unwrap());
        env::set_var("CUBE_IDENTITY_PASSPHRASE", "test-daemon-id");

        let id = DaemonIdentity::init();
        assert!(!id.pk_hex.is_empty());
        assert!(id.pk_hex.len() > 20);

        let _ = fs::remove_dir_all(&dir);
        env::remove_var("CUBE_IDENTITY_DIR");
        env::remove_var("CUBE_IDENTITY_PASSPHRASE");
    }
}
