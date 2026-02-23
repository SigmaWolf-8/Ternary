// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved

use axum::Json;
use axum::http::StatusCode;
use plenumnet_kernel::crypto::tl_dsa;
use plenumnet_kernel::crypto::tl_kem;
use crate::types::*;
use crate::serialization;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorResponse>)>;

fn api_error(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (status, Json(ErrorResponse { success: false, error: msg.into() }))
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        service: "pqti-service",
        version: "0.1.0",
    })
}

pub async fn algorithms() -> Json<AlgorithmsResponse> {
    Json(AlgorithmsResponse {
        success: true,
        tl_dsa: vec![
            AlgorithmInfo {
                name: "TL-DSA-44",
                category: "Digital Signature",
                nist_equivalent: "ML-DSA-44 (FIPS 204)",
                nist_level: 2,
                security_bits: 128,
                public_key_trits: tl_dsa::public_key_size(tl_dsa::TlDsaVariant::TlDsa44),
                secret_key_trits: tl_dsa::secret_key_size(tl_dsa::TlDsaVariant::TlDsa44),
                signature_trits: Some(tl_dsa::signature_size(tl_dsa::TlDsaVariant::TlDsa44)),
                ciphertext_trits: None,
                shared_secret_trits: None,
            },
            AlgorithmInfo {
                name: "TL-DSA-65",
                category: "Digital Signature",
                nist_equivalent: "ML-DSA-65 (FIPS 204)",
                nist_level: 3,
                security_bits: 192,
                public_key_trits: tl_dsa::public_key_size(tl_dsa::TlDsaVariant::TlDsa65),
                secret_key_trits: tl_dsa::secret_key_size(tl_dsa::TlDsaVariant::TlDsa65),
                signature_trits: Some(tl_dsa::signature_size(tl_dsa::TlDsaVariant::TlDsa65)),
                ciphertext_trits: None,
                shared_secret_trits: None,
            },
            AlgorithmInfo {
                name: "TL-DSA-87",
                category: "Digital Signature",
                nist_equivalent: "ML-DSA-87 (FIPS 204)",
                nist_level: 5,
                security_bits: 256,
                public_key_trits: tl_dsa::public_key_size(tl_dsa::TlDsaVariant::TlDsa87),
                secret_key_trits: tl_dsa::secret_key_size(tl_dsa::TlDsaVariant::TlDsa87),
                signature_trits: Some(tl_dsa::signature_size(tl_dsa::TlDsaVariant::TlDsa87)),
                ciphertext_trits: None,
                shared_secret_trits: None,
            },
        ],
        tl_kem: vec![
            AlgorithmInfo {
                name: "TL-KEM-512",
                category: "Key Encapsulation",
                nist_equivalent: "ML-KEM-512 (FIPS 203)",
                nist_level: 1,
                security_bits: 128,
                public_key_trits: tl_kem::public_key_size(tl_kem::TlKemVariant::TlKem512),
                secret_key_trits: tl_kem::secret_key_size(tl_kem::TlKemVariant::TlKem512),
                signature_trits: None,
                ciphertext_trits: Some(tl_kem::ciphertext_size(tl_kem::TlKemVariant::TlKem512)),
                shared_secret_trits: Some(tl_kem::shared_secret_size(tl_kem::TlKemVariant::TlKem512)),
            },
            AlgorithmInfo {
                name: "TL-KEM-768",
                category: "Key Encapsulation",
                nist_equivalent: "ML-KEM-768 (FIPS 203)",
                nist_level: 3,
                security_bits: 192,
                public_key_trits: tl_kem::public_key_size(tl_kem::TlKemVariant::TlKem768),
                secret_key_trits: tl_kem::secret_key_size(tl_kem::TlKemVariant::TlKem768),
                signature_trits: None,
                ciphertext_trits: Some(tl_kem::ciphertext_size(tl_kem::TlKemVariant::TlKem768)),
                shared_secret_trits: Some(tl_kem::shared_secret_size(tl_kem::TlKemVariant::TlKem768)),
            },
            AlgorithmInfo {
                name: "TL-KEM-1024",
                category: "Key Encapsulation",
                nist_equivalent: "ML-KEM-1024 (FIPS 203)",
                nist_level: 5,
                security_bits: 256,
                public_key_trits: tl_kem::public_key_size(tl_kem::TlKemVariant::TlKem1024),
                secret_key_trits: tl_kem::secret_key_size(tl_kem::TlKemVariant::TlKem1024),
                signature_trits: None,
                ciphertext_trits: Some(tl_kem::ciphertext_size(tl_kem::TlKemVariant::TlKem1024)),
                shared_secret_trits: Some(tl_kem::shared_secret_size(tl_kem::TlKemVariant::TlKem1024)),
            },
        ],
    })
}

pub async fn interop_capabilities() -> Json<InteropResponse> {
    Json(InteropResponse {
        success: true,
        ready: true,
        capabilities: vec![
            InteropCapability { binary_algorithm: "ML-KEM-512", ternary_algorithm: "TL-KEM-512", directions: "Binary <-> Ternary", nist_level: 1 },
            InteropCapability { binary_algorithm: "ML-KEM-768", ternary_algorithm: "TL-KEM-768", directions: "Binary <-> Ternary", nist_level: 3 },
            InteropCapability { binary_algorithm: "ML-KEM-1024", ternary_algorithm: "TL-KEM-1024", directions: "Binary <-> Ternary", nist_level: 5 },
            InteropCapability { binary_algorithm: "ML-DSA-44", ternary_algorithm: "TL-DSA-44", directions: "Binary <-> Ternary", nist_level: 2 },
            InteropCapability { binary_algorithm: "ML-DSA-65", ternary_algorithm: "TL-DSA-65", directions: "Binary <-> Ternary", nist_level: 3 },
            InteropCapability { binary_algorithm: "ML-DSA-87", ternary_algorithm: "TL-DSA-87", directions: "Binary <-> Ternary", nist_level: 5 },
        ],
    })
}

pub async fn tldsa_keygen(Json(req): Json<TlDsaKeygenRequest>) -> ApiResult<TlDsaKeygenResponse> {
    let variant = serialization::parse_dsa_variant(&req.variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
    let seed = serialization::decode_trits_base64(&req.seed)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;

    let (pk, sk) = tl_dsa::keygen(variant, &seed)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("KeyGen failed: {}", e)))?;

    Ok(Json(TlDsaKeygenResponse {
        success: true,
        variant: variant.name().to_string(),
        public_key: SerializedKey {
            encoding: "base64".into(),
            data: serialization::serialize_dsa_public_key(&pk),
            size_trits: tl_dsa::public_key_size(variant),
        },
        secret_key: SerializedKey {
            encoding: "base64".into(),
            data: serialization::serialize_dsa_secret_key(&sk),
            size_trits: tl_dsa::secret_key_size(variant),
        },
    }))
}

pub async fn tldsa_sign(Json(req): Json<TlDsaSignRequest>) -> ApiResult<TlDsaSignResponse> {
    let variant = serialization::parse_dsa_variant(&req.variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
    let sk = serialization::deserialize_dsa_secret_key(&req.secret_key, variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid secret key: {}", e)))?;
    let message = serialization::decode_trits_base64(&req.message)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid message: {}", e)))?;

    let sig = tl_dsa::sign(&sk, &message)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Signing failed: {}", e)))?;

    Ok(Json(TlDsaSignResponse {
        success: true,
        variant: variant.name().to_string(),
        signature: SerializedSignature {
            encoding: "base64".into(),
            data: serialization::serialize_dsa_signature(&sig),
            size_trits: tl_dsa::signature_size(variant),
        },
    }))
}

pub async fn tldsa_verify(Json(req): Json<TlDsaVerifyRequest>) -> ApiResult<TlDsaVerifyResponse> {
    let variant = serialization::parse_dsa_variant(&req.variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
    let pk = serialization::deserialize_dsa_public_key(&req.public_key, variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid public key: {}", e)))?;
    let message = serialization::decode_trits_base64(&req.message)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid message: {}", e)))?;
    let sig = serialization::deserialize_dsa_signature(&req.signature, variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid signature: {}", e)))?;

    let valid = tl_dsa::verify(&pk, &message, &sig)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Verification failed: {}", e)))?;

    Ok(Json(TlDsaVerifyResponse {
        success: true,
        valid,
        variant: variant.name().to_string(),
    }))
}

pub async fn tlkem_keygen(Json(req): Json<TlKemKeygenRequest>) -> ApiResult<TlKemKeygenResponse> {
    let variant = serialization::parse_kem_variant(&req.variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
    let seed = serialization::decode_trits_base64(&req.seed)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;

    let (pk, sk) = tl_kem::keygen(variant, &seed)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("KEM KeyGen failed: {}", e)))?;

    Ok(Json(TlKemKeygenResponse {
        success: true,
        variant: variant.name().to_string(),
        public_key: SerializedKey {
            encoding: "base64".into(),
            data: serialization::serialize_kem_public_key(&pk),
            size_trits: tl_kem::public_key_size(variant),
        },
        secret_key: SerializedKey {
            encoding: "base64".into(),
            data: serialization::serialize_kem_secret_key(&sk),
            size_trits: tl_kem::secret_key_size(variant),
        },
    }))
}

pub async fn tlkem_encapsulate(Json(req): Json<TlKemEncapsulateRequest>) -> ApiResult<TlKemEncapsulateResponse> {
    let variant = serialization::parse_kem_variant(&req.variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
    let pk = serialization::deserialize_kem_public_key(&req.public_key, variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid public key: {}", e)))?;
    let randomness = serialization::decode_trits_base64(&req.randomness)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid randomness: {}", e)))?;

    let (ct, ss) = tl_kem::encapsulate(&pk, &randomness)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Encapsulation failed: {}", e)))?;

    Ok(Json(TlKemEncapsulateResponse {
        success: true,
        variant: variant.name().to_string(),
        ciphertext: SerializedCiphertext {
            encoding: "base64".into(),
            data: serialization::serialize_kem_ciphertext(&ct),
            size_trits: tl_kem::ciphertext_size(variant),
        },
        shared_secret: SerializedSharedSecret {
            encoding: "base64".into(),
            data: serialization::serialize_shared_secret(&ss),
            size_trits: tl_kem::shared_secret_size(variant),
        },
    }))
}

pub async fn tlkem_decapsulate(Json(req): Json<TlKemDecapsulateRequest>) -> ApiResult<TlKemDecapsulateResponse> {
    let variant = serialization::parse_kem_variant(&req.variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
    let sk = serialization::deserialize_kem_secret_key(&req.secret_key, variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid secret key: {}", e)))?;
    let ct = serialization::deserialize_kem_ciphertext(&req.ciphertext, variant)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Invalid ciphertext: {}", e)))?;

    let ss = tl_kem::decapsulate(&sk, &ct)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Decapsulation failed: {}", e)))?;

    Ok(Json(TlKemDecapsulateResponse {
        success: true,
        variant: variant.name().to_string(),
        shared_secret: SerializedSharedSecret {
            encoding: "base64".into(),
            data: serialization::serialize_shared_secret(&ss),
            size_trits: tl_kem::shared_secret_size(variant),
        },
    }))
}
