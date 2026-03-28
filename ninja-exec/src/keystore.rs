// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::signing_engine;

const MAGIC: &[u8; 8] = b"NJXK0002";
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const SK_LEN: usize = 128;
const PK_LEN: usize = 64;
const KDF_ITERATIONS: u32 = 4096;
const KDF_VERSION: u8 = 2;

const HEADER_LEN: usize = 8 // magic
    + 1                      // kdf_version
    + 4                      // kdf_iterations (u32 LE)
    + SALT_LEN               // salt
    + NONCE_LEN              // nonce
    + SK_LEN                 // encrypted secret key
    + TAG_LEN                // authentication tag
    + PK_LEN;                // public key (unencrypted)

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeystoreError {
    EntropyFailure,
    EmptyPassphrase,
    PassphraseTooShort,
    InvalidFormat,
    UnsupportedVersion(u8),
    DecryptionFailed,
    IoError(String),
    NotUnlocked,
    AlreadyExists,
}

impl std::fmt::Display for KeystoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntropyFailure => write!(f, "failed to generate random bytes"),
            Self::EmptyPassphrase => write!(f, "passphrase cannot be empty"),
            Self::PassphraseTooShort => write!(f, "passphrase must be at least 12 characters"),
            Self::InvalidFormat => write!(f, "keystore file has invalid format"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported keystore KDF version: {}", v),
            Self::DecryptionFailed => write!(f, "decryption failed (wrong passphrase or corrupted keystore)"),
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::NotUnlocked => write!(f, "keystore is locked"),
            Self::AlreadyExists => write!(f, "keystore already exists"),
        }
    }
}

impl std::error::Error for KeystoreError {}

fn zeroize(buf: &mut [u8]) {
    for b in buf.iter_mut() {
        unsafe { std::ptr::write_volatile(b as *mut u8, 0x00); }
    }
}

fn derive_enc_key(passphrase: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut material = Vec::with_capacity(passphrase.len() + salt.len());
    material.extend_from_slice(passphrase);
    material.extend_from_slice(salt);
    let mut derived = ternary_math::sponge::derive_key(b"NinjaExec-KDF-v2", &material, 32);
    zeroize(&mut material);
    for _ in 1..iterations {
        let mut round_input = Vec::with_capacity(32 + salt.len());
        round_input.extend_from_slice(&derived);
        round_input.extend_from_slice(salt);
        derived = ternary_math::sponge::derive_key(b"NinjaExec-KDF-v2", &round_input, 32);
        zeroize(&mut round_input);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&derived);
    zeroize(&mut derived);
    key
}

fn encrypt_sk(sk: &[u8], passphrase: &[u8], iterations: u32) -> Result<(Vec<u8>, [u8; SALT_LEN], [u8; NONCE_LEN], [u8; TAG_LEN]), KeystoreError> {
    let mut salt = [0u8; SALT_LEN];
    getrandom::getrandom(&mut salt).map_err(|_| KeystoreError::EntropyFailure)?;

    let mut nonce = [0u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce).map_err(|_| KeystoreError::EntropyFailure)?;

    let mut enc_key = derive_enc_key(passphrase, &salt, iterations);

    let mut ks_material = Vec::with_capacity(32 + NONCE_LEN);
    ks_material.extend_from_slice(&enc_key);
    ks_material.extend_from_slice(&nonce);
    let mut keystream = ternary_math::sponge::derive_key(b"NinjaExec-KS-STREAM", &ks_material, SK_LEN);
    zeroize(&mut ks_material);

    let mut ciphertext = vec![0u8; SK_LEN];
    for i in 0..SK_LEN {
        ciphertext[i] = sk[i] ^ keystream[i];
    }
    zeroize(&mut keystream);

    let mut tag_material = Vec::with_capacity(32 + NONCE_LEN + SK_LEN);
    tag_material.extend_from_slice(&enc_key);
    tag_material.extend_from_slice(&nonce);
    tag_material.extend_from_slice(&ciphertext);
    let tag_vec = ternary_math::sponge::derive_key(b"NinjaExec-KS-TAG", &tag_material, TAG_LEN);
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(&tag_vec);
    zeroize(&mut tag_material);
    zeroize(&mut enc_key);

    Ok((ciphertext, salt, nonce, tag))
}

fn decrypt_sk(ciphertext: &[u8], passphrase: &[u8], salt: &[u8], nonce: &[u8], tag: &[u8], iterations: u32) -> Result<Vec<u8>, KeystoreError> {
    let mut enc_key = derive_enc_key(passphrase, salt, iterations);

    let mut tag_material = Vec::with_capacity(32 + NONCE_LEN + SK_LEN);
    tag_material.extend_from_slice(&enc_key);
    tag_material.extend_from_slice(nonce);
    tag_material.extend_from_slice(ciphertext);
    let expected_tag = ternary_math::sponge::derive_key(b"NinjaExec-KS-TAG", &tag_material, TAG_LEN);
    zeroize(&mut tag_material);

    let mut diff: u8 = 0;
    for i in 0..TAG_LEN {
        diff |= expected_tag[i] ^ tag[i];
    }
    if diff != 0 {
        zeroize(&mut enc_key);
        return Err(KeystoreError::DecryptionFailed);
    }

    let mut ks_material = Vec::with_capacity(32 + NONCE_LEN);
    ks_material.extend_from_slice(&enc_key);
    ks_material.extend_from_slice(nonce);
    let mut keystream = ternary_math::sponge::derive_key(b"NinjaExec-KS-STREAM", &ks_material, SK_LEN);
    zeroize(&mut ks_material);
    zeroize(&mut enc_key);

    let mut plaintext = vec![0u8; SK_LEN];
    for i in 0..SK_LEN {
        plaintext[i] = ciphertext[i] ^ keystream[i];
    }
    zeroize(&mut keystream);

    Ok(plaintext)
}

pub struct Keystore {
    data_dir: PathBuf,
    public_key: Option<Vec<u8>>,
    secret_key: Option<Vec<u8>>,
}

impl Keystore {
    pub fn new(data_dir: PathBuf) -> Self {
        Keystore {
            data_dir,
            public_key: None,
            secret_key: None,
        }
    }

    pub fn keystore_path(&self) -> PathBuf {
        self.data_dir.join("ninja-exec.keystore")
    }

    pub fn exists(&self) -> bool {
        self.keystore_path().exists()
    }

    pub fn create(&mut self, passphrase: &str) -> Result<(), KeystoreError> {
        if passphrase.is_empty() {
            return Err(KeystoreError::EmptyPassphrase);
        }
        if passphrase.len() < 12 {
            return Err(KeystoreError::PassphraseTooShort);
        }
        if self.exists() {
            return Err(KeystoreError::AlreadyExists);
        }

        let mut seed = vec![0u8; 64];
        getrandom::getrandom(&mut seed).map_err(|_| KeystoreError::EntropyFailure)?;

        let kp = signing_engine::generate_keypair(&seed);
        zeroize(&mut seed);

        let (ct, salt, nonce, tag) = encrypt_sk(&kp.secret_key, passphrase.as_bytes(), KDF_ITERATIONS)?;

        let mut blob = Vec::with_capacity(HEADER_LEN);
        blob.extend_from_slice(MAGIC);
        blob.push(KDF_VERSION);
        blob.extend_from_slice(&KDF_ITERATIONS.to_le_bytes());
        blob.extend_from_slice(&salt);
        blob.extend_from_slice(&nonce);
        blob.extend_from_slice(&ct);
        blob.extend_from_slice(&tag);
        blob.extend_from_slice(&kp.public_key);

        fs::create_dir_all(&self.data_dir)
            .map_err(|e| KeystoreError::IoError(e.to_string()))?;

        let path = self.keystore_path();
        let tmp_path = path.with_extension("keystore.tmp");
        fs::write(&tmp_path, &blob)
            .map_err(|e| KeystoreError::IoError(e.to_string()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
        }

        fs::rename(&tmp_path, &path)
            .map_err(|e| KeystoreError::IoError(e.to_string()))?;

        self.public_key = Some(kp.public_key);
        self.secret_key = Some(kp.secret_key);

        Ok(())
    }

    pub fn open(&mut self, passphrase: &str) -> Result<(), KeystoreError> {
        if passphrase.is_empty() {
            return Err(KeystoreError::EmptyPassphrase);
        }

        let path = self.keystore_path();
        let blob = fs::read(&path)
            .map_err(|e| KeystoreError::IoError(e.to_string()))?;

        if blob.len() != HEADER_LEN {
            return Err(KeystoreError::InvalidFormat);
        }
        if &blob[..8] != MAGIC {
            return Err(KeystoreError::InvalidFormat);
        }

        let kdf_ver = blob[8];
        if kdf_ver != KDF_VERSION {
            return Err(KeystoreError::UnsupportedVersion(kdf_ver));
        }

        let iterations = u32::from_le_bytes([blob[9], blob[10], blob[11], blob[12]]);
        if iterations == 0 || iterations > 10_000_000 {
            return Err(KeystoreError::InvalidFormat);
        }

        let off = 13;
        let salt = &blob[off..off + SALT_LEN];
        let nonce = &blob[off + SALT_LEN..off + SALT_LEN + NONCE_LEN];
        let ct = &blob[off + SALT_LEN + NONCE_LEN..off + SALT_LEN + NONCE_LEN + SK_LEN];
        let tag = &blob[off + SALT_LEN + NONCE_LEN + SK_LEN..off + SALT_LEN + NONCE_LEN + SK_LEN + TAG_LEN];
        let pk = &blob[off + SALT_LEN + NONCE_LEN + SK_LEN + TAG_LEN..];

        let sk = decrypt_sk(ct, passphrase.as_bytes(), salt, nonce, tag, iterations)?;

        self.public_key = Some(pk.to_vec());
        self.secret_key = Some(sk);

        Ok(())
    }

    pub fn unlock(&mut self, passphrase: &str) -> Result<(), KeystoreError> {
        self.open(passphrase)
    }

    pub fn lock(&mut self) {
        if let Some(ref mut sk) = self.secret_key {
            zeroize(sk);
        }
        self.secret_key = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.secret_key.is_some()
    }

    #[allow(dead_code)]
    pub fn is_locked(&self) -> bool {
        self.public_key.is_some() && self.secret_key.is_none()
    }

    pub fn public_key(&self) -> Option<&[u8]> {
        self.public_key.as_deref()
    }

    pub fn secret_key(&self) -> Option<&[u8]> {
        self.secret_key.as_deref()
    }

    pub fn load_public_key_only(&mut self) -> Result<(), KeystoreError> {
        let path = self.keystore_path();
        let blob = fs::read(&path)
            .map_err(|e| KeystoreError::IoError(e.to_string()))?;
        if blob.len() != HEADER_LEN {
            return Err(KeystoreError::InvalidFormat);
        }
        if &blob[..8] != MAGIC {
            return Err(KeystoreError::InvalidFormat);
        }
        let pk_offset = 13 + SALT_LEN + NONCE_LEN + SK_LEN + TAG_LEN;
        let pk = &blob[pk_offset..];
        self.public_key = Some(pk.to_vec());
        Ok(())
    }
}

impl Drop for Keystore {
    fn drop(&mut self) {
        self.lock();
    }
}

pub type SharedKeystore = Arc<Mutex<Keystore>>;

pub fn default_data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("NinjaExec");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".ninja-exec");
    }
    PathBuf::from(".ninja-exec")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_dir() -> PathBuf {
        let mut dir = env::temp_dir();
        dir.push(format!("ninja-exec-test-{}", std::process::id()));
        dir
    }

    #[test]
    fn test_create_and_open() {
        let dir = temp_dir().join("create-open-v2");
        let _ = fs::remove_dir_all(&dir);

        let mut ks = Keystore::new(dir.clone());
        ks.create("test-passphrase-12chars").unwrap();
        assert!(ks.is_unlocked());

        let pk = ks.public_key().unwrap().to_vec();
        ks.lock();
        assert!(ks.is_locked());

        ks.unlock("test-passphrase-12chars").unwrap();
        assert!(ks.is_unlocked());
        assert_eq!(ks.public_key().unwrap(), pk.as_slice());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_wrong_passphrase() {
        let dir = temp_dir().join("wrong-pass-v2");
        let _ = fs::remove_dir_all(&dir);

        let mut ks = Keystore::new(dir.clone());
        ks.create("correct-passphrase").unwrap();
        ks.lock();

        let result = ks.unlock("wrong-passphrase!!");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_passphrase_too_short() {
        let dir = temp_dir().join("short-pass-v2");
        let _ = fs::remove_dir_all(&dir);

        let mut ks = Keystore::new(dir.clone());
        let result = ks.create("short");
        assert_eq!(result.unwrap_err(), KeystoreError::PassphraseTooShort);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_public_key_only() {
        let dir = temp_dir().join("pk-only-v2");
        let _ = fs::remove_dir_all(&dir);

        let mut ks = Keystore::new(dir.clone());
        ks.create("test-passphrase-12chars").unwrap();
        let pk = ks.public_key().unwrap().to_vec();
        drop(ks);

        let mut ks2 = Keystore::new(dir.clone());
        ks2.load_public_key_only().unwrap();
        assert_eq!(ks2.public_key().unwrap(), pk.as_slice());
        assert!(!ks2.is_unlocked());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_keystore_header_contains_kdf_params() {
        let dir = temp_dir().join("kdf-params");
        let _ = fs::remove_dir_all(&dir);

        let mut ks = Keystore::new(dir.clone());
        ks.create("test-passphrase-12chars").unwrap();

        let blob = fs::read(ks.keystore_path()).unwrap();
        assert_eq!(&blob[..8], MAGIC);
        assert_eq!(blob[8], KDF_VERSION);
        let stored_iterations = u32::from_le_bytes([blob[9], blob[10], blob[11], blob[12]]);
        assert_eq!(stored_iterations, KDF_ITERATIONS);
        assert_eq!(blob.len(), HEADER_LEN);

        let _ = fs::remove_dir_all(&dir);
    }
}
