// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct AlgorithmInfo {
    pub name: &'static str,
    pub category: &'static str,
    pub nist_equivalent: &'static str,
    pub nist_level: u32,
    pub security_bits: u32,
    pub public_key_trits: usize,
    pub secret_key_trits: usize,
    pub signature_trits: Option<usize>,
    pub ciphertext_trits: Option<usize>,
    pub shared_secret_trits: Option<usize>,
}

#[derive(Serialize)]
pub struct AlgorithmsResponse {
    pub success: bool,
    pub tl_dsa: Vec<AlgorithmInfo>,
    pub tl_kem: Vec<AlgorithmInfo>,
}

#[derive(Serialize)]
pub struct InteropCapability {
    pub binary_algorithm: &'static str,
    pub ternary_algorithm: &'static str,
    pub directions: &'static str,
    pub nist_level: u32,
}

#[derive(Serialize)]
pub struct InteropResponse {
    pub success: bool,
    pub capabilities: Vec<InteropCapability>,
    pub ready: bool,
}

#[derive(Deserialize)]
pub struct TlDsaKeygenRequest {
    pub variant: String,
    pub seed: String,
    pub encoding: Option<String>,
}

#[derive(Serialize)]
pub struct TlDsaKeygenResponse {
    pub success: bool,
    pub variant: String,
    pub public_key: SerializedKey,
    pub secret_key: SerializedKey,
}

#[derive(Serialize)]
pub struct SerializedKey {
    pub encoding: String,
    pub data: String,
    pub size_trits: usize,
}

#[derive(Deserialize)]
pub struct TlDsaSignRequest {
    pub variant: String,
    pub secret_key: String,
    pub message: String,
    pub encoding: Option<String>,
}

#[derive(Serialize)]
pub struct TlDsaSignResponse {
    pub success: bool,
    pub variant: String,
    pub signature: SerializedSignature,
}

#[derive(Serialize)]
pub struct SerializedSignature {
    pub encoding: String,
    pub data: String,
    pub size_trits: usize,
}

#[derive(Deserialize)]
pub struct TlDsaVerifyRequest {
    pub variant: String,
    pub public_key: String,
    pub message: String,
    pub signature: String,
    pub encoding: Option<String>,
}

#[derive(Serialize)]
pub struct TlDsaVerifyResponse {
    pub success: bool,
    pub valid: bool,
    pub variant: String,
}

#[derive(Deserialize)]
pub struct TlKemKeygenRequest {
    pub variant: String,
    pub seed: String,
    pub encoding: Option<String>,
}

#[derive(Serialize)]
pub struct TlKemKeygenResponse {
    pub success: bool,
    pub variant: String,
    pub public_key: SerializedKey,
    pub secret_key: SerializedKey,
}

#[derive(Deserialize)]
pub struct TlKemEncapsulateRequest {
    pub variant: String,
    pub public_key: String,
    pub randomness: String,
    pub encoding: Option<String>,
}

#[derive(Serialize)]
pub struct TlKemEncapsulateResponse {
    pub success: bool,
    pub variant: String,
    pub ciphertext: SerializedCiphertext,
    pub shared_secret: SerializedSharedSecret,
}

#[derive(Serialize)]
pub struct SerializedCiphertext {
    pub encoding: String,
    pub data: String,
    pub size_trits: usize,
}

#[derive(Serialize)]
pub struct SerializedSharedSecret {
    pub encoding: String,
    pub data: String,
    pub size_trits: usize,
}

#[derive(Deserialize)]
pub struct TlKemDecapsulateRequest {
    pub variant: String,
    pub secret_key: String,
    pub ciphertext: String,
    pub encoding: Option<String>,
}

#[derive(Serialize)]
pub struct TlKemDecapsulateResponse {
    pub success: bool,
    pub variant: String,
    pub shared_secret: SerializedSharedSecret,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}
