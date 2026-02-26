/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * CAPABILITY TOKEN SCHEMA
 * @version 1.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   shared/types/capability.ts
 *
 * Defines the structure of PlenumNET capability tokens — unforgeable,
 * self-contained, bearer-verified authorization tokens signed with TL-DSA.
 * Phase 1: typed constraints + HPTP-anchored expiration.
 * Phase 4 adds hardware binding + single-use chains.
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
