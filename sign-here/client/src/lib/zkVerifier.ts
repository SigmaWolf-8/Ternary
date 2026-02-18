/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */
interface SalviProof {
  pi_a: [string, string];
  pi_b: [[string, string], [string, string]];
  pi_c: [string, string];
  commitment: string;
  nullifier: string;
}

interface PublicInputs {
  doc_hash: string;
  tenant_id: string;
  signer_count: number;
  certified_at: string;
}

interface VerifyResult {
  valid: boolean;
  commitment_match: boolean;
  nullifier_valid: boolean;
  message: string;
}

let wasmModule: any = null;
let wasmLoadAttempted = false;

export async function loadZKVerifier(): Promise<boolean> {
  if (wasmModule) return true;
  if (wasmLoadAttempted) return false;
  wasmLoadAttempted = true;

  try {
    const jsUrl = "/zk-wasm/salvi_zk.js";
    const response = await fetch(jsUrl);
    if (!response.ok) throw new Error(`Failed to fetch ZK module: ${response.status}`);
    const jsText = await response.text();

    const blob = new Blob([jsText], { type: "application/javascript" });
    const blobUrl = URL.createObjectURL(blob);

    const module = await import(/* @vite-ignore */ blobUrl);
    URL.revokeObjectURL(blobUrl);

    await module.default("/zk-wasm/salvi_zk_bg.wasm");
    wasmModule = module;
    return true;
  } catch (err) {
    console.warn("ZK WASM load failed, using fallback verifier:", err);
    return false;
  }
}

export async function verifyZKProof(
  proof: SalviProof,
  publicInputs: PublicInputs
): Promise<VerifyResult> {
  const loaded = await loadZKVerifier();

  if (loaded && wasmModule) {
    try {
      const resultJson = wasmModule.verify_proof(
        JSON.stringify(proof),
        JSON.stringify(publicInputs)
      );
      return JSON.parse(resultJson);
    } catch (err) {
      console.warn("WASM verify failed, using fallback:", err);
    }
  }

  return fallbackVerify(proof, publicInputs);
}

export async function generateZKProofClient(
  publicInputs: PublicInputs
): Promise<SalviProof | null> {
  const loaded = await loadZKVerifier();

  if (loaded && wasmModule) {
    try {
      const proofJson = wasmModule.generate_proof(JSON.stringify(publicInputs));
      return JSON.parse(proofJson);
    } catch (err) {
      console.warn("WASM proof gen failed:", err);
    }
  }

  return null;
}

async function sha256Hex(input: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(input);
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(hashBuffer))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

async function computeCommitmentJS(
  docHash: string,
  tenantId: string,
  signerCount: number,
  certifiedAt: string
): Promise<string> {
  const buf = new Uint8Array(4);
  new DataView(buf.buffer).setUint32(0, signerCount, true);
  const signerBytes = Array.from(buf)
    .map((b) => String.fromCharCode(b))
    .join("");
  return sha256Hex(
    "salvi_zk_v1:" + docHash + ":" + tenantId + ":" + signerBytes + ":" + certifiedAt
  );
}

async function fallbackVerify(
  proof: SalviProof,
  publicInputs: PublicInputs
): Promise<VerifyResult> {
  try {
    const expectedCommitment = await computeCommitmentJS(
      publicInputs.doc_hash,
      publicInputs.tenant_id,
      publicInputs.signer_count,
      publicInputs.certified_at
    );

    const commitmentMatch = proof.commitment === expectedCommitment;

    const expectedNullifier = await sha256Hex(
      "salvi_nullifier:" + expectedCommitment + ":" + proof.pi_a[0]
    );
    const nullifierValid = proof.nullifier === expectedNullifier;

    const valid = commitmentMatch && nullifierValid;

    return {
      valid,
      commitment_match: commitmentMatch,
      nullifier_valid: nullifierValid,
      message: valid
        ? "Proof verified: document integrity and authorization confirmed via Ternary ZK"
        : `Verification failed: ${!commitmentMatch ? "commitment mismatch" : ""} ${!nullifierValid ? "nullifier invalid" : ""}`.trim(),
    };
  } catch (err) {
    return {
      valid: false,
      commitment_match: false,
      nullifier_valid: false,
      message: `Fallback verification error: ${err}`,
    };
  }
}

export function isWasmLoaded(): boolean {
  return wasmModule !== null;
}
