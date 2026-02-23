// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use plenumnet_kernel::crypto::tl_dsa::{
    TlDsaVariant, TlDsaPublicKey, TlDsaSecretKey, TlDsaSignature,
};
use plenumnet_kernel::crypto::tl_kem::{
    TlKemVariant, TlKemPublicKey, TlKemSecretKey, TlKemCiphertext, SharedSecret,
};
use plenumnet_kernel::crypto::ternary_lattice::{TernaryPolynomial, TernaryPolyVec};

pub fn trits_to_bytes(trits: &[i8]) -> Vec<u8> {
    trits.iter().map(|&t| (t + 1) as u8).collect()
}

pub fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    bytes.iter().map(|&b| (b as i8) - 1).collect()
}

pub fn encode_trits_base64(trits: &[i8]) -> String {
    let bytes = trits_to_bytes(trits);
    BASE64.encode(&bytes)
}

pub fn decode_trits_base64(encoded: &str) -> Result<Vec<i8>, String> {
    let bytes = BASE64.decode(encoded).map_err(|e| format!("Invalid base64: {}", e))?;
    Ok(bytes_to_trits(&bytes))
}

pub fn parse_dsa_variant(name: &str) -> Result<TlDsaVariant, String> {
    match name.to_uppercase().replace("-", "").replace("_", "").as_str() {
        "TLDSA44" => Ok(TlDsaVariant::TlDsa44),
        "TLDSA65" => Ok(TlDsaVariant::TlDsa65),
        "TLDSA87" => Ok(TlDsaVariant::TlDsa87),
        _ => Err(format!("Unknown TL-DSA variant: '{}'. Valid: TL-DSA-44, TL-DSA-65, TL-DSA-87", name)),
    }
}

pub fn parse_kem_variant(name: &str) -> Result<TlKemVariant, String> {
    match name.to_uppercase().replace("-", "").replace("_", "").as_str() {
        "TLKEM512" => Ok(TlKemVariant::TlKem512),
        "TLKEM768" => Ok(TlKemVariant::TlKem768),
        "TLKEM1024" => Ok(TlKemVariant::TlKem1024),
        _ => Err(format!("Unknown TL-KEM variant: '{}'. Valid: TL-KEM-512, TL-KEM-768, TL-KEM-1024", name)),
    }
}

fn poly_vec_to_trits(v: &TernaryPolyVec) -> Vec<i8> {
    let mut trits = Vec::new();
    for p in &v.polys {
        trits.extend_from_slice(&p.coeffs);
    }
    trits
}

fn trits_to_poly_vec(trits: &[i8], num_polys: usize, n: usize) -> Result<TernaryPolyVec, String> {
    let expected = num_polys * n;
    if trits.len() < expected {
        return Err(format!("Expected {} trits for {} polynomials of degree {}, got {}", expected, num_polys, n, trits.len()));
    }
    let mut polys = Vec::with_capacity(num_polys);
    for i in 0..num_polys {
        let start = i * n;
        let coeffs: Vec<i8> = trits[start..start + n].to_vec();
        polys.push(TernaryPolynomial::from_coeffs_unchecked(coeffs));
    }
    Ok(TernaryPolyVec { polys, n })
}

pub fn serialize_dsa_public_key(pk: &TlDsaPublicKey) -> String {
    let mut trits = pk.matrix_a_seed.clone();
    trits.extend(poly_vec_to_trits(&pk.public_t));
    encode_trits_base64(&trits)
}

pub fn deserialize_dsa_public_key(encoded: &str, variant: TlDsaVariant) -> Result<TlDsaPublicKey, String> {
    let trits = decode_trits_base64(encoded)?;
    let params = variant.params();
    let seed_len = 243;
    let vec_len = params.k * params.n;
    let expected = seed_len + vec_len;
    if trits.len() < expected {
        return Err(format!("Public key too short: expected {} trits, got {}", expected, trits.len()));
    }
    let matrix_a_seed = trits[..seed_len].to_vec();
    let public_t = trits_to_poly_vec(&trits[seed_len..], params.k, params.n)?;
    Ok(TlDsaPublicKey { variant, matrix_a_seed, public_t })
}

pub fn serialize_dsa_secret_key(sk: &TlDsaSecretKey) -> String {
    let mut trits = sk.matrix_a_seed.clone();
    trits.extend(poly_vec_to_trits(&sk.secret_s1));
    trits.extend(poly_vec_to_trits(&sk.secret_s2));
    trits.extend(poly_vec_to_trits(&sk.public_t));
    trits.extend(&sk.signing_seed);
    encode_trits_base64(&trits)
}

pub fn deserialize_dsa_secret_key(encoded: &str, variant: TlDsaVariant) -> Result<TlDsaSecretKey, String> {
    let trits = decode_trits_base64(encoded)?;
    let params = variant.params();
    let seed_len = 243;
    let s1_len = params.l * params.n;
    let s2_len = params.k * params.n;
    let t_len = params.k * params.n;
    let signing_seed_len = 243;
    let expected = seed_len + s1_len + s2_len + t_len + signing_seed_len;
    if trits.len() < expected {
        return Err(format!("Secret key too short: expected {} trits, got {}", expected, trits.len()));
    }
    let mut offset = 0;
    let matrix_a_seed = trits[offset..offset + seed_len].to_vec();
    offset += seed_len;
    let secret_s1 = trits_to_poly_vec(&trits[offset..], params.l, params.n)?;
    offset += s1_len;
    let secret_s2 = trits_to_poly_vec(&trits[offset..], params.k, params.n)?;
    offset += s2_len;
    let public_t = trits_to_poly_vec(&trits[offset..], params.k, params.n)?;
    offset += t_len;
    let signing_seed = trits[offset..offset + signing_seed_len].to_vec();

    Ok(TlDsaSecretKey {
        variant,
        matrix_a_seed,
        secret_s1,
        secret_s2,
        public_t,
        signing_seed,
    })
}

pub fn serialize_dsa_signature(sig: &TlDsaSignature) -> String {
    let mut trits = poly_vec_to_trits(&sig.z);
    trits.extend(&sig.challenge_hash);
    encode_trits_base64(&trits)
}

pub fn deserialize_dsa_signature(encoded: &str, variant: TlDsaVariant) -> Result<TlDsaSignature, String> {
    let trits = decode_trits_base64(encoded)?;
    let params = variant.params();
    let z_len = params.l * params.n;
    let hash_len = 243;
    let expected = z_len + hash_len;
    if trits.len() < expected {
        return Err(format!("Signature too short: expected {} trits, got {}", expected, trits.len()));
    }
    let z = trits_to_poly_vec(&trits[..z_len], params.l, params.n)?;
    let challenge_hash = trits[z_len..z_len + hash_len].to_vec();
    Ok(TlDsaSignature { variant, z, challenge_hash })
}

pub fn serialize_kem_public_key(pk: &TlKemPublicKey) -> String {
    let mut trits = pk.matrix_a_seed.clone();
    trits.extend(poly_vec_to_trits(&pk.public_vec_t));
    encode_trits_base64(&trits)
}

pub fn deserialize_kem_public_key(encoded: &str, variant: TlKemVariant) -> Result<TlKemPublicKey, String> {
    let trits = decode_trits_base64(encoded)?;
    let params = variant.params();
    let seed_len = 243;
    let vec_len = params.k * params.n;
    let expected = seed_len + vec_len;
    if trits.len() < expected {
        return Err(format!("KEM public key too short: expected {} trits, got {}", expected, trits.len()));
    }
    let matrix_a_seed = trits[..seed_len].to_vec();
    let public_vec_t = trits_to_poly_vec(&trits[seed_len..], params.k, params.n)?;
    Ok(TlKemPublicKey { variant, matrix_a_seed, public_vec_t })
}

pub fn serialize_kem_secret_key(sk: &TlKemSecretKey) -> String {
    let mut trits = Vec::new();
    trits.extend(poly_vec_to_trits(&sk.secret_s));
    trits.extend(&sk.public_key.matrix_a_seed);
    trits.extend(poly_vec_to_trits(&sk.public_key.public_vec_t));
    trits.extend(&sk.hash_pk);
    trits.extend(&sk.implicit_reject_seed);
    encode_trits_base64(&trits)
}

pub fn deserialize_kem_secret_key(encoded: &str, variant: TlKemVariant) -> Result<TlKemSecretKey, String> {
    let trits = decode_trits_base64(encoded)?;
    let params = variant.params();
    let s_len = params.k * params.n;
    let seed_len = 243;
    let t_len = params.k * params.n;
    let hash_len = 243;
    let reject_len = 243;
    let expected = s_len + seed_len + t_len + hash_len + reject_len;
    if trits.len() < expected {
        return Err(format!("KEM secret key too short: expected {} trits, got {}", expected, trits.len()));
    }
    let mut offset = 0;
    let secret_s = trits_to_poly_vec(&trits[offset..], params.k, params.n)?;
    offset += s_len;
    let matrix_a_seed = trits[offset..offset + seed_len].to_vec();
    offset += seed_len;
    let public_vec_t = trits_to_poly_vec(&trits[offset..], params.k, params.n)?;
    offset += t_len;
    let hash_pk = trits[offset..offset + hash_len].to_vec();
    offset += hash_len;
    let implicit_reject_seed = trits[offset..offset + reject_len].to_vec();

    let public_key = TlKemPublicKey {
        variant,
        matrix_a_seed,
        public_vec_t,
    };
    Ok(TlKemSecretKey {
        variant,
        secret_s,
        public_key,
        hash_pk,
        implicit_reject_seed,
    })
}

pub fn serialize_kem_ciphertext(ct: &TlKemCiphertext) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    let num_u = ct.compressed_u.len() as u32;
    bytes.extend(&num_u.to_le_bytes());
    for u_vec in &ct.compressed_u {
        let len = u_vec.len() as u32;
        bytes.extend(&len.to_le_bytes());
        bytes.extend(u_vec);
    }
    let v_len = ct.compressed_v.len() as u32;
    bytes.extend(&v_len.to_le_bytes());
    bytes.extend(&ct.compressed_v);
    BASE64.encode(&bytes)
}

pub fn deserialize_kem_ciphertext(encoded: &str, variant: TlKemVariant) -> Result<TlKemCiphertext, String> {
    let bytes = BASE64.decode(encoded).map_err(|e| format!("Invalid base64: {}", e))?;
    if bytes.len() < 4 {
        return Err("Ciphertext data too short".into());
    }
    let mut offset = 0;
    let num_u = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    let mut compressed_u = Vec::with_capacity(num_u);
    for _ in 0..num_u {
        if offset + 4 > bytes.len() {
            return Err("Ciphertext truncated in u-vector lengths".into());
        }
        let len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if offset + len > bytes.len() {
            return Err("Ciphertext truncated in u-vector data".into());
        }
        compressed_u.push(bytes[offset..offset + len].to_vec());
        offset += len;
    }

    if offset + 4 > bytes.len() {
        return Err("Ciphertext truncated in v-vector length".into());
    }
    let v_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    if offset + v_len > bytes.len() {
        return Err("Ciphertext truncated in v-vector data".into());
    }
    let compressed_v = bytes[offset..offset + v_len].to_vec();

    Ok(TlKemCiphertext { variant, compressed_u, compressed_v })
}

pub fn serialize_shared_secret(ss: &SharedSecret) -> String {
    encode_trits_base64(&ss.trits)
}
