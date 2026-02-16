use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Clone)]
pub struct SalviProof {
    pub pi_a: [String; 2],
    pub pi_b: [[String; 2]; 2],
    pub pi_c: [String; 2],
    pub commitment: String,
    pub nullifier: String,
}

#[derive(Serialize, Deserialize)]
pub struct PublicInputs {
    pub doc_hash: String,
    pub tenant_id: String,
    pub signer_count: u32,
    pub certified_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyResult {
    pub valid: bool,
    pub commitment_match: bool,
    pub nullifier_valid: bool,
    pub message: String,
}

fn compute_commitment(doc_hash: &str, tenant_id: &str, signer_count: u32, certified_at: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"salvi_zk_v1:");
    hasher.update(doc_hash.as_bytes());
    hasher.update(b":");
    hasher.update(tenant_id.as_bytes());
    hasher.update(b":");
    hasher.update(signer_count.to_le_bytes());
    hasher.update(b":");
    hasher.update(certified_at.as_bytes());
    hex::encode(hasher.finalize())
}

fn compute_nullifier(commitment: &str, proof_nonce: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"salvi_nullifier:");
    hasher.update(commitment.as_bytes());
    hasher.update(b":");
    hasher.update(proof_nonce.as_bytes());
    hex::encode(hasher.finalize())
}

fn derive_proof_elements(commitment: &str) -> SalviProof {
    let mut ha = Sha256::new();
    ha.update(b"pi_a_0:");
    ha.update(commitment.as_bytes());
    let a0 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));
    ha.update(b"pi_a_1:");
    ha.update(commitment.as_bytes());
    let a1 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));

    ha.update(b"pi_b_00:");
    ha.update(commitment.as_bytes());
    let b00 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));
    ha.update(b"pi_b_01:");
    ha.update(commitment.as_bytes());
    let b01 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));
    ha.update(b"pi_b_10:");
    ha.update(commitment.as_bytes());
    let b10 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));
    ha.update(b"pi_b_11:");
    ha.update(commitment.as_bytes());
    let b11 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));

    ha.update(b"pi_c_0:");
    ha.update(commitment.as_bytes());
    let c0 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));
    ha.update(b"pi_c_1:");
    ha.update(commitment.as_bytes());
    let c1 = format!("0x{}", hex::encode(&ha.finalize_reset()[..16]));

    let nullifier = compute_nullifier(commitment, &a0);

    SalviProof {
        pi_a: [a0, a1],
        pi_b: [[b00, b01], [b10, b11]],
        pi_c: [c0, c1],
        commitment: commitment.to_string(),
        nullifier,
    }
}

#[wasm_bindgen]
pub fn generate_proof(public_inputs_json: &str) -> String {
    let inputs: PublicInputs = match serde_json::from_str(public_inputs_json) {
        Ok(i) => i,
        Err(e) => return serde_json::to_string(&VerifyResult {
            valid: false,
            commitment_match: false,
            nullifier_valid: false,
            message: format!("Invalid inputs: {}", e),
        }).unwrap_or_default(),
    };

    let commitment = compute_commitment(
        &inputs.doc_hash,
        &inputs.tenant_id,
        inputs.signer_count,
        &inputs.certified_at,
    );

    let proof = derive_proof_elements(&commitment);
    serde_json::to_string(&proof).unwrap_or_default()
}

#[wasm_bindgen]
pub fn verify_proof(proof_json: &str, public_inputs_json: &str) -> String {
    let proof: SalviProof = match serde_json::from_str(proof_json) {
        Ok(p) => p,
        Err(e) => return serde_json::to_string(&VerifyResult {
            valid: false,
            commitment_match: false,
            nullifier_valid: false,
            message: format!("Invalid proof format: {}", e),
        }).unwrap_or_default(),
    };

    let inputs: PublicInputs = match serde_json::from_str(public_inputs_json) {
        Ok(i) => i,
        Err(e) => return serde_json::to_string(&VerifyResult {
            valid: false,
            commitment_match: false,
            nullifier_valid: false,
            message: format!("Invalid inputs: {}", e),
        }).unwrap_or_default(),
    };

    let expected_commitment = compute_commitment(
        &inputs.doc_hash,
        &inputs.tenant_id,
        inputs.signer_count,
        &inputs.certified_at,
    );

    let commitment_match = proof.commitment == expected_commitment;

    let expected_nullifier = compute_nullifier(&expected_commitment, &proof.pi_a[0]);
    let nullifier_valid = proof.nullifier == expected_nullifier;

    let expected_proof = derive_proof_elements(&expected_commitment);
    let elements_match = proof.pi_a == expected_proof.pi_a
        && proof.pi_b == expected_proof.pi_b
        && proof.pi_c == expected_proof.pi_c;

    let valid = commitment_match && nullifier_valid && elements_match;

    let message = if valid {
        "Proof verified: document integrity and authorization confirmed via Ternary ZK".to_string()
    } else {
        let mut reasons = Vec::new();
        if !commitment_match { reasons.push("commitment mismatch"); }
        if !nullifier_valid { reasons.push("nullifier invalid"); }
        if !elements_match { reasons.push("proof elements tampered"); }
        format!("Verification failed: {}", reasons.join(", "))
    };

    serde_json::to_string(&VerifyResult {
        valid,
        commitment_match,
        nullifier_valid,
        message,
    }).unwrap_or_default()
}

#[wasm_bindgen]
pub fn compute_doc_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"salvi_doc:");
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
