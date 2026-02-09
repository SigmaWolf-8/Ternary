//! Firmware Signing & Verification Pipeline (CNSA 2.0)
//!
//! Provides a complete sign -> boot verify -> reject pipeline for firmware
//! images using XMSS or LMS hash-based signatures per SP 800-208.
//!
//! # Architecture
//! 1. **Sign**: Authority signs firmware manifest (hash, version, timestamp)
//! 2. **Verify**: Boot loader reconstructs manifest and verifies signature
//! 3. **Reject**: Invalid signatures halt boot with diagnostic error
//!
//! # Security Properties
//! - Firmware images are hashed before signing (never sign raw binary)
//! - Manifest includes version monotonicity check to prevent rollback
//! - Signature verification is constant-time (no timing side channels)
//! - XMSS/LMS stateful index tracked per signing authority
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use super::{CryptoError, CryptoResult, TERNARY_HASH_TRITS};
use super::sponge::TernarySponge;
use super::signature::{
    XmssKeypair, XmssParams, XmssSignature, XmssState,
    LmsKeypair, LmsType, LmsSignature, LmsState, LmotsType,
    xmss_sign, xmss_verify, lms_sign, lms_verify,
};

const MANIFEST_VERSION: u8 = 1;
const FIRMWARE_HASH_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareSignatureScheme {
    Xmss(XmssParams),
    Lms(LmsType, LmotsType),
}

#[derive(Debug, Clone)]
pub struct FirmwareManifest {
    pub version: u8,
    pub firmware_version: u32,
    pub firmware_hash: [u8; FIRMWARE_HASH_SIZE],
    pub timestamp: u64,
    pub device_class: u32,
}

impl FirmwareManifest {
    pub fn new(firmware: &[u8], firmware_version: u32, timestamp: u64, device_class: u32) -> Self {
        let firmware_hash = hash_firmware(firmware);
        Self {
            version: MANIFEST_VERSION,
            firmware_version,
            firmware_hash,
            timestamp,
            device_class,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(49);
        out.push(self.version);
        out.extend_from_slice(&self.firmware_version.to_be_bytes());
        out.extend_from_slice(&self.firmware_hash);
        out.extend_from_slice(&self.timestamp.to_be_bytes());
        out.extend_from_slice(&self.device_class.to_be_bytes());
        out
    }

    pub fn from_bytes(data: &[u8]) -> CryptoResult<Self> {
        if data.len() < 49 {
            return Err(CryptoError::InvalidInputLength { expected: 49, actual: data.len() });
        }
        let version = data[0];
        if version != MANIFEST_VERSION {
            return Err(CryptoError::KeyGenerationFailed(
                String::from("unsupported manifest version"),
            ));
        }
        let firmware_version = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        let mut firmware_hash = [0u8; FIRMWARE_HASH_SIZE];
        firmware_hash.copy_from_slice(&data[5..37]);
        let timestamp = u64::from_be_bytes([
            data[37], data[38], data[39], data[40],
            data[41], data[42], data[43], data[44],
        ]);
        let device_class = u32::from_be_bytes([data[45], data[46], data[47], data[48]]);
        Ok(Self { version, firmware_version, firmware_hash, timestamp, device_class })
    }
}

fn hash_firmware(firmware: &[u8]) -> [u8; FIRMWARE_HASH_SIZE] {
    use super::TernaryDigest;
    let mut sponge = TernarySponge::new();
    sponge.absorb(&[0i8]);
    let len_bytes = (firmware.len() as u64).to_be_bytes();
    let len_td = TernaryDigest::from_bytes(&len_bytes, 40);
    sponge.absorb(&len_td.trits);
    let chunk_size = 128;
    let mut offset = 0;
    while offset < firmware.len() {
        let end = core::cmp::min(offset + chunk_size, firmware.len());
        let td = TernaryDigest::from_bytes(&firmware[offset..end], (end - offset) * 5);
        sponge.absorb(&td.trits);
        offset = end;
    }
    let out = sponge.squeeze(TERNARY_HASH_TRITS);
    let bytes = out.to_bytes();
    let mut result = [0u8; FIRMWARE_HASH_SIZE];
    let len = core::cmp::min(bytes.len(), FIRMWARE_HASH_SIZE);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

#[derive(Debug, Clone)]
pub enum SignedFirmware {
    XmssSigned {
        manifest: FirmwareManifest,
        signature: XmssSignature,
        params: XmssParams,
        pub_seed: [u8; 32],
        root: [u8; 32],
    },
    LmsSigned {
        manifest: FirmwareManifest,
        signature: LmsSignature,
        lms_type: LmsType,
        ots_type: LmotsType,
        identifier: [u8; 16],
        root: [u8; 32],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootVerifyResult {
    Accepted,
    RejectedBadSignature,
    RejectedManifestCorrupt,
    RejectedVersionRollback,
}

pub fn sign_firmware_xmss(
    firmware: &[u8],
    firmware_version: u32,
    timestamp: u64,
    device_class: u32,
    keypair: &XmssKeypair,
    state: &mut XmssState,
) -> CryptoResult<SignedFirmware> {
    let manifest = FirmwareManifest::new(firmware, firmware_version, timestamp, device_class);
    let manifest_bytes = manifest.to_bytes();
    let signature = xmss_sign(keypair, state, &manifest_bytes)?;
    Ok(SignedFirmware::XmssSigned {
        manifest,
        signature,
        params: keypair.params,
        pub_seed: keypair.pub_seed,
        root: keypair.root,
    })
}

pub fn sign_firmware_lms(
    firmware: &[u8],
    firmware_version: u32,
    timestamp: u64,
    device_class: u32,
    keypair: &LmsKeypair,
    state: &mut LmsState,
) -> CryptoResult<SignedFirmware> {
    let manifest = FirmwareManifest::new(firmware, firmware_version, timestamp, device_class);
    let manifest_bytes = manifest.to_bytes();
    let signature = lms_sign(keypair, state, &manifest_bytes)?;
    Ok(SignedFirmware::LmsSigned {
        manifest,
        signature,
        lms_type: keypair.lms_type,
        ots_type: keypair.ots_type,
        identifier: keypair.identifier,
        root: keypair.root,
    })
}

pub fn verify_firmware_boot(
    firmware: &[u8],
    signed: &SignedFirmware,
    minimum_version: u32,
) -> BootVerifyResult {
    let (manifest, sig_valid) = match signed {
        SignedFirmware::XmssSigned { manifest, signature, params, pub_seed, root } => {
            let manifest_bytes = manifest.to_bytes();
            let valid = xmss_verify(root, pub_seed, *params, &manifest_bytes, signature)
                .unwrap_or(false);
            (manifest, valid)
        }
        SignedFirmware::LmsSigned { manifest, signature, lms_type, ots_type, identifier, root } => {
            let manifest_bytes = manifest.to_bytes();
            let valid = lms_verify(root, identifier, *lms_type, *ots_type, &manifest_bytes, signature)
                .unwrap_or(false);
            (manifest, valid)
        }
    };
    if !sig_valid {
        return BootVerifyResult::RejectedBadSignature;
    }
    let computed_hash = hash_firmware(firmware);
    let mut hash_diff: u8 = 0;
    for (a, b) in computed_hash.iter().zip(manifest.firmware_hash.iter()) {
        hash_diff |= a ^ b;
    }
    if hash_diff != 0 {
        return BootVerifyResult::RejectedManifestCorrupt;
    }
    if manifest.firmware_version < minimum_version {
        return BootVerifyResult::RejectedVersionRollback;
    }
    BootVerifyResult::Accepted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firmware_manifest_roundtrip() {
        let firmware = alloc::vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02];
        let manifest = FirmwareManifest::new(&firmware, 42, 1700000000, 7);
        assert_eq!(manifest.version, MANIFEST_VERSION);
        assert_eq!(manifest.firmware_version, 42);
        assert_eq!(manifest.device_class, 7);
        let bytes = manifest.to_bytes();
        let recovered = FirmwareManifest::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.firmware_version, 42);
        assert_eq!(recovered.timestamp, 1700000000);
        assert_eq!(recovered.firmware_hash, manifest.firmware_hash);
    }

    #[test]
    fn test_firmware_hash_deterministic() {
        let fw = alloc::vec![1u8, 2, 3, 4, 5];
        let h1 = hash_firmware(&fw);
        let h2 = hash_firmware(&fw);
        assert_eq!(h1, h2);
        let fw2 = alloc::vec![1u8, 2, 3, 4, 6];
        let h3 = hash_firmware(&fw2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_boot_verify_result_variants() {
        assert_ne!(BootVerifyResult::Accepted, BootVerifyResult::RejectedBadSignature);
        assert_ne!(BootVerifyResult::RejectedManifestCorrupt, BootVerifyResult::RejectedVersionRollback);
    }
}
