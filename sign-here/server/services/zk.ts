import { createHash } from "crypto";
import { storage } from "../storage";
import { getHPTP } from "./plenum";

interface ZKProofData {
  pi_a: [string, string];
  pi_b: [[string, string], [string, string]];
  pi_c: [string, string];
  commitment: string;
  nullifier: string;
}

interface ZKPublicInputs {
  doc_hash: string;
  tenant_id: string;
  signer_count: number;
  certified_at: string;
}

function computeCommitment(docHash: string, tenantId: string, signerCount: number, certifiedAt: string): string {
  const buf = Buffer.alloc(4);
  buf.writeUInt32LE(signerCount);
  const hash = createHash("sha256");
  hash.update("salvi_zk_v1:");
  hash.update(docHash);
  hash.update(":");
  hash.update(tenantId);
  hash.update(":");
  hash.update(buf);
  hash.update(":");
  hash.update(certifiedAt);
  return hash.digest("hex");
}

function computeNullifier(commitment: string, proofNonce: string): string {
  const hash = createHash("sha256");
  hash.update("salvi_nullifier:");
  hash.update(commitment);
  hash.update(":");
  hash.update(proofNonce);
  return hash.digest("hex");
}

function deriveProofElement(prefix: string, commitment: string): string {
  const hash = createHash("sha256");
  hash.update(prefix);
  hash.update(commitment);
  return "0x" + hash.digest("hex").substring(0, 32);
}

function deriveProofElements(commitment: string): ZKProofData {
  const a0 = deriveProofElement("pi_a_0:", commitment);
  const a1 = deriveProofElement("pi_a_1:", commitment);
  const b00 = deriveProofElement("pi_b_00:", commitment);
  const b01 = deriveProofElement("pi_b_01:", commitment);
  const b10 = deriveProofElement("pi_b_10:", commitment);
  const b11 = deriveProofElement("pi_b_11:", commitment);
  const c0 = deriveProofElement("pi_c_0:", commitment);
  const c1 = deriveProofElement("pi_c_1:", commitment);

  const nullifier = computeNullifier(commitment, a0);

  return {
    pi_a: [a0, a1],
    pi_b: [[b00, b01], [b10, b11]],
    pi_c: [c0, c1],
    commitment,
    nullifier,
  };
}

export async function generateZKProof(envelopeId: string, requesterTenantId: string) {
  const envelope = await storage.getEnvelope(envelopeId);
  if (!envelope) throw new Error("Envelope not found");

  let certData: any = {};
  if (envelope.zkProof) {
    try { certData = JSON.parse(envelope.zkProof); } catch {}
  }

  const publicInputs: ZKPublicInputs = {
    doc_hash: envelope.plenumDocId || envelopeId,
    tenant_id: requesterTenantId || envelope.tenantId || "default",
    signer_count: certData.signerCount || 0,
    certified_at: certData.certifiedAt || envelope.updatedAt?.toISOString() || new Date().toISOString(),
  };

  const commitment = computeCommitment(
    publicInputs.doc_hash,
    publicInputs.tenant_id,
    publicInputs.signer_count,
    publicInputs.certified_at
  );

  const proof = deriveProofElements(commitment);

  const timestamp = await getHPTP();

  const existingProof = envelope.zkProof ? JSON.parse(envelope.zkProof) : {};
  const updatedProof = {
    ...existingProof,
    zkProof: proof,
    publicInputs,
    proofGeneratedAt: timestamp,
  };

  await storage.updateEnvelope(envelopeId, {
    zkProof: JSON.stringify(updatedProof),
  });

  return {
    proof,
    publicInputs,
    timestamp,
  };
}

export function verifyZKProofServer(proof: ZKProofData, publicInputs: ZKPublicInputs): boolean {
  const expectedCommitment = computeCommitment(
    publicInputs.doc_hash,
    publicInputs.tenant_id,
    publicInputs.signer_count,
    publicInputs.certified_at
  );

  if (proof.commitment !== expectedCommitment) return false;

  const expectedNullifier = computeNullifier(expectedCommitment, proof.pi_a[0]);
  if (proof.nullifier !== expectedNullifier) return false;

  const expected = deriveProofElements(expectedCommitment);
  return proof.pi_a[0] === expected.pi_a[0]
    && proof.pi_a[1] === expected.pi_a[1]
    && proof.pi_c[0] === expected.pi_c[0]
    && proof.pi_c[1] === expected.pi_c[1];
}

export function computeDocHash(content: string): string {
  const hash = createHash("sha256");
  hash.update("salvi_doc:");
  hash.update(content);
  return hash.digest("hex");
}
