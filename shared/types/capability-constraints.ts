/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * CAPABILITY CONSTRAINT REGISTRY v1.0
 * @version 1.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   shared/types/capability-constraints.ts
 *
 * Discriminated union of all valid constraint types. Every verifier in the
 * system — Kong plugin, Rust service, TypeScript middleware — imports and
 * enforces constraints identically. There are no freeform objects. Unknown
 * constraint types fail closed.
 *
 * Adding a new constraint type requires:
 *   1. Adding it to this union
 *   2. Adding validation logic to validateConstraint()
 *   3. Updating the Kong plugin's semantic map
 *   4. Mirroring the change in the Rust constraint registry
 *   5. Incrementing the registry version
 */

export const CONSTRAINT_REGISTRY_VERSION = "1.0";

export type CapabilityConstraint =
  | { type: "recipient_domain"; value: string }
  | { type: "vault_id"; value: string }
  | { type: "template"; value: string }
  | { type: "max_uses"; value: number }
  | { type: "ip_range"; value: string }
  | { type: "geo_country"; value: string[] }
  | { type: "document_id"; value: string }
  | { type: "project_id"; value: string };

export interface VerificationContext {
  recipient?: string;
  vault_id?: string;
  template?: string;
  usage_count?: number;
  source_ip?: string;
  source_country?: string;
  document_id?: string;
  project_id?: string;
}

export function validateConstraint(
  constraint: CapabilityConstraint,
  context: VerificationContext
): boolean {
  switch (constraint.type) {
    case "recipient_domain": {
      if (!context.recipient) return false;
      const pattern = constraint.value;
      if (pattern.startsWith("*.")) {
        return context.recipient.endsWith(pattern.slice(1));
      }
      return context.recipient.endsWith(constraint.value);
    }
    case "vault_id":
      return context.vault_id === constraint.value;
    case "template":
      return context.template === constraint.value;
    case "max_uses":
      return true;
    case "ip_range":
      return context.source_ip ? isIpInCidr(context.source_ip, constraint.value) : false;
    case "geo_country":
      return context.source_country ? constraint.value.includes(context.source_country) : false;
    case "document_id":
      return context.document_id === constraint.value;
    case "project_id":
      return context.project_id === constraint.value;
    default:
      return false;
  }
}

export function validateAllConstraints(
  constraints: CapabilityConstraint[],
  context: VerificationContext
): { granted: boolean; failed: CapabilityConstraint[] } {
  const failed = constraints.filter(c => !validateConstraint(c, context));
  return { granted: failed.length === 0, failed };
}

function isIpInCidr(ip: string, cidr: string): boolean {
  const [range, bits] = cidr.split("/");
  const mask = (~(2 ** (32 - parseInt(bits)) - 1)) >>> 0;
  const ipNum = (ip.split(".").reduce((acc, oct) => (acc << 8) + parseInt(oct), 0)) >>> 0;
  const rangeNum = (range.split(".").reduce((acc, oct) => (acc << 8) + parseInt(oct), 0)) >>> 0;
  return (ipNum & mask) === (rangeNum & mask);
}
