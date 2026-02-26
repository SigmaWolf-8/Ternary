/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * CAPABILITY TOKEN SCHEMA
 * @version 4.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   shared/types/capability.ts
 *
 * Defines the structure of PlenumNET capability tokens — unforgeable,
 * self-contained, bearer-verified authorization tokens signed with TL-DSA.
 * Phase 1: typed constraints + HPTP-anchored expiration.
 * Phase 2: HPTP-bound expiration — timing engine wired into validation.
 * Phase 3: HMAC-chained delegation — macaroon-style attenuation.
 * Phase 4: Hardware binding + HPTP challenge-response + single-use chains.
 * Phase 5: RFC 3161 capability certificates.
 * Phase 6: Inter-service capability mesh.
 */

import { CapabilityConstraint, CONSTRAINT_REGISTRY_VERSION } from "./capability-constraints";

export interface Capability {
  res: string;
  constraints: CapabilityConstraint[];
  exp: string;
}

export interface CapabilityToken {
  sub: string;
  cap: Capability[];
  role?: string;
  iat_hptp: string;
  iss: "plenumnet.cap";
  jti: string;
  crv: string;
}

export interface SignedCapabilityToken {
  token: CapabilityToken;
  signature: string;
  algorithm: "TL-DSA";
}

export function createCapabilityToken(
  subject: string,
  capabilities: Capability[],
  hptpTimestamp: string,
  tokenId: string,
  legacyRole?: string,
): CapabilityToken {
  return {
    sub: subject,
    cap: capabilities,
    ...(legacyRole && capabilities.length === 0 ? { role: legacyRole } : {}),
    iat_hptp: hptpTimestamp,
    iss: "plenumnet.cap",
    jti: tokenId,
    crv: CONSTRAINT_REGISTRY_VERSION,
  };
}

export function isCapabilityExpired(capability: Capability, currentHptpNs: string): boolean {
  return BigInt(currentHptpNs) >= BigInt(capability.exp);
}

export function findMatchingCapability(
  token: CapabilityToken,
  resource: string,
  currentHptpNs: string,
): Capability | null {
  if (token.cap.length === 0) return null;
  for (const cap of token.cap) {
    if (cap.res === resource && !isCapabilityExpired(cap, currentHptpNs)) {
      return cap;
    }
  }
  return null;
}

export type HardwareBindingType = 'tpm' | 'enclave' | 'hsm';

export interface HardwareBinding {
  device_id: string;
  public_key_hash: string;
  binding_type: HardwareBindingType;
  registered_at_hptp_ns: string;
}

export interface HardwareBoundCapabilityToken {
  signed_token: SignedCapabilityToken;
  hardware_binding: HardwareBinding;
  bound_at_hptp_ns: string;
}

export interface HptpChallenge {
  challenge_id: string;
  nonce: string;
  issued_at_hptp_ns: string;
  expires_at_hptp_ns: string;
  device_id: string;
  window_ns: string;
}

export interface HptpChallengeResponse {
  challenge_id: string;
  nonce: string;
  signature: string;
  device_id: string;
  signed_at_hptp_ns: string;
}

export interface SingleUseChainState {
  chain_id: string;
  token_jti: string;
  current_position: number;
  seed_hash: string;
  created_at_hptp_ns: string;
  consumed_positions: Map<number, string>;
  max_positions?: number;
}

export interface SingleUseCapabilityToken {
  hardware_bound_token: HardwareBoundCapabilityToken;
  chain_id: string;
  chain_position: number;
  position_hash: string;
}

export interface CapabilityCertificate {
  certificate_id: string;
  capability_token_hash: string;
  capability_jti: string;
  tsa_timestamp: {
    hash_algorithm: 'SHA3-256';
    message_imprint: string;
    serial_number: string;
    gen_time_hptp_ns: string;
    policy_oid: string;
    tsa_signature: string;
    tsa_algorithm: 'TL-DSA + RSA-4096';
    nonce: string;
  };
  issued_at_hptp_ns: string;
  subject: string;
  resources: string[];
  dual_signature: {
    tldsa_signature: string;
    rsa4096_signature: string;
  };
  merkle_proof: {
    leaf_hash: string;
    root_hash: string;
    proof_path: string[];
    tree_size: number;
  };
  status: 'valid' | 'revoked' | 'expired';
}

export interface CertificateVerificationResult {
  valid: boolean;
  certificate_id: string;
  tsa_timestamp_valid: boolean;
  capability_signature_valid: boolean;
  merkle_proof_valid: boolean;
  certificate_status: 'valid' | 'revoked' | 'expired';
  verified_at_hptp_ns: string;
  errors: string[];
}

export interface EvidenceChainEntry {
  position: number;
  certificate: CapabilityCertificate;
  chain_hash: string;
  previous_hash: string;
  timestamp_hptp_ns: string;
}

export interface ServiceNode {
  service_id: string;
  service_name: string;
  capabilities: string[];
  registered_at_hptp_ns: string;
  status: 'active' | 'inactive' | 'suspended';
  endpoint: string;
  last_heartbeat_hptp_ns: string;
  metadata: Record<string, string>;
}

export interface MeshCapability {
  mesh_token_id: string;
  from_service: string;
  to_service: string;
  signed_token: SignedCapabilityToken;
  propagation_path: string[];
  hop_count: number;
  max_hops: number;
  attenuations_per_hop: Record<string, CapabilityConstraint[]>;
  issued_at_hptp_ns: string;
  mesh_signature: string;
}

export interface MeshTopology {
  nodes: ServiceNode[];
  edges: {
    from: string;
    to: string;
    capabilities: string[];
    active_tokens: number;
  }[];
  total_services: number;
  total_edges: number;
  mesh_health: 'healthy' | 'degraded' | 'critical';
  last_updated_hptp_ns: string;
}

export interface PropagationPath {
  path: string[];
  total_hops: number;
  attenuations_applied: number;
  final_capabilities: string[];
  propagation_time_ns: string;
}
