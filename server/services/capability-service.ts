/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * CAPABILITY TOKEN SERVICE — Phases 1-3
 * @version 4.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/services/capability-service.ts
 *
 * Phase 1: Typed constraint registry + capability token schema + audit events.
 * Phase 2: HPTP-bound expiration — timing engine wired into token validation.
 * Phase 3: HMAC-chained delegation — macaroon-style attenuation with TL-DSA roots.
 * Phase 4: Hardware binding — see capability-hardware-binding.ts
 * Phase 5: RFC 3161 certificates — see capability-certificates.ts
 * Phase 6: Inter-service mesh — see capability-mesh.ts
 */

import crypto from 'crypto';
import {
  Capability,
  CapabilityToken,
  SignedCapabilityToken,
  createCapabilityToken,
  isCapabilityExpired,
  findMatchingCapability,
} from '../../shared/types/capability';
import {
  CapabilityConstraint,
  VerificationContext,
  validateAllConstraints,
} from '../../shared/types/capability-constraints';
import { getSharedAuditLog } from './capability-audit-events';
import {
  getFemtosecondTimestamp,
  FEMTOSECONDS_PER_MILLISECOND,
  FEMTOSECONDS_PER_SECOND,
} from '../salvi-core/femtosecond-timing';
import { signString as tlDsaSignString, verifyString as tlDsaVerifyString } from '../crypto/tl-dsa-bridge';
import { getTlDsaSigningKeyPair, deriveHmacKey } from '../crypto/key-management';

export interface HptpExpirationWindow {
  issued_hptp_ns: string;
  expires_hptp_ns: string;
  ttl_seconds: number;
  precision: 'nanosecond';
  clock_source: 'HPTP/1.0';
}

export interface CapabilityValidationResult {
  granted: boolean;
  capability: Capability | null;
  expiration: {
    expired: boolean;
    remaining_ns: string;
    remaining_human: string;
    checked_at_hptp_ns: string;
  };
  constraints: {
    all_satisfied: boolean;
    failed: CapabilityConstraint[];
  };
  audit_event_hash: string;
}

export interface DelegationCaveat {
  constraint: CapabilityConstraint;
  hmac: string;
}

export interface DelegatedCapabilityToken {
  root_signature: string;
  root_algorithm: 'TL-DSA';
  root_token_hash: string;
  token: CapabilityToken;
  delegation_chain: DelegationCaveat[];
  chain_depth: number;
  parent_jti: string;
  parent_token_hash: string;
}

export interface DelegationResult {
  delegated_token: DelegatedCapabilityToken;
  audit_event_hash: string;
  chain_depth: number;
  expiration: HptpExpirationWindow;
}

function getHptpNanoseconds(): string {
  const ts = getFemtosecondTimestamp();
  return (ts.femtoseconds / 1_000_000n).toString();
}

function formatNsRemaining(ns: bigint): string {
  if (ns <= 0n) return 'expired';
  if (ns < 1_000n) return `${ns}ns`;
  if (ns < 1_000_000n) return `${Number(ns) / 1_000}µs`;
  if (ns < 1_000_000_000n) return `${(Number(ns) / 1_000_000).toFixed(2)}ms`;
  return `${(Number(ns) / 1_000_000_000).toFixed(3)}s`;
}

export class CapabilityService {
  private auditLog = getSharedAuditLog();
  private usageCounts: Map<string, number> = new Map();

  constructor() {}

  getHptpNow(): string {
    return getHptpNanoseconds();
  }

  computeExpirationWindow(ttlSeconds: number): HptpExpirationWindow {
    const nowNs = BigInt(getHptpNanoseconds());
    const ttlNs = BigInt(ttlSeconds) * 1_000_000_000n;
    return {
      issued_hptp_ns: nowNs.toString(),
      expires_hptp_ns: (nowNs + ttlNs).toString(),
      ttl_seconds: ttlSeconds,
      precision: 'nanosecond',
      clock_source: 'HPTP/1.0',
    };
  }

  issueCapabilityToken(
    subject: string,
    resources: { res: string; constraints: CapabilityConstraint[]; ttlSeconds: number }[],
    ipAddress?: string,
  ): { signedToken: SignedCapabilityToken; expiration: HptpExpirationWindow; audit_event_hash: string } {
    const hptpNs = getHptpNanoseconds();
    const jti = `cap_${crypto.randomUUID().replace(/-/g, '')}`;

    let maxTtl = 0;
    const capabilities: Capability[] = resources.map(r => {
      if (r.ttlSeconds > maxTtl) maxTtl = r.ttlSeconds;
      const expNs = BigInt(hptpNs) + BigInt(r.ttlSeconds) * 1_000_000_000n;
      return {
        res: r.res,
        constraints: r.constraints,
        exp: expNs.toString(),
      };
    });

    const token = createCapabilityToken(subject, capabilities, hptpNs, jti);

    const tokenHash = this.auditLog.hashToken(token);
    const signingKeys = getTlDsaSigningKeyPair();
    const signature = tlDsaSignString(signingKeys.secretKey, tokenHash, signingKeys.variant);

    const signedToken: SignedCapabilityToken = {
      token,
      signature,
      algorithm: 'TL-DSA',
    };

    const auditHash = this.auditLog.recordIssued(token, hptpNs, ipAddress);
    const expiration = this.computeExpirationWindow(maxTtl);

    return { signedToken, expiration, audit_event_hash: auditHash };
  }

  validateCapability(
    signedToken: SignedCapabilityToken,
    resource: string,
    context: VerificationContext,
    ipAddress?: string,
  ): CapabilityValidationResult {
    const hptpNs = getHptpNanoseconds();
    const { token } = signedToken;

    const tokenHash = this.auditLog.hashToken(token);
    const signingKeys = getTlDsaSigningKeyPair();
    const sigValid = tlDsaVerifyString(
      signingKeys.publicKey,
      tokenHash,
      signedToken.signature,
      signingKeys.secretKey,
      signingKeys.variant,
    );

    if (!sigValid) {
      const auditHash = this.auditLog.recordValidated(token, resource, 'denied', hptpNs, [], ipAddress);
      return {
        granted: false,
        capability: null,
        expiration: { expired: false, remaining_ns: '0', remaining_human: 'invalid signature', checked_at_hptp_ns: hptpNs },
        constraints: { all_satisfied: false, failed: [] },
        audit_event_hash: auditHash,
      };
    }

    const cap = findMatchingCapability(token, resource, hptpNs);
    if (!cap) {
      const auditHash = this.auditLog.recordValidated(token, resource, 'denied', hptpNs, [], ipAddress);
      const anyMatch = token.cap.find(c => c.res === resource);
      const expired = anyMatch ? isCapabilityExpired(anyMatch, hptpNs) : false;
      const remainNs = anyMatch ? BigInt(anyMatch.exp) - BigInt(hptpNs) : 0n;

      if (expired && anyMatch) {
        this.auditLog.recordExpired(token.jti, this.auditLog.hashToken(token), hptpNs);
      }

      return {
        granted: false,
        capability: anyMatch || null,
        expiration: {
          expired,
          remaining_ns: remainNs.toString(),
          remaining_human: expired ? 'expired' : 'no matching capability',
          checked_at_hptp_ns: hptpNs,
        },
        constraints: { all_satisfied: false, failed: [] },
        audit_event_hash: auditHash,
      };
    }

    const remainNs = BigInt(cap.exp) - BigInt(hptpNs);

    const maxUsesConstraint = cap.constraints.find(c => c.type === 'max_uses') as { type: 'max_uses'; value: number } | undefined;
    if (maxUsesConstraint) {
      const currentCount = (this.usageCounts.get(token.jti) || 0) + 1;
      if (currentCount > maxUsesConstraint.value) {
        this.usageCounts.set(token.jti, currentCount);
        this.auditLog.recordUsageExceeded(token, hptpNs, currentCount);
        const auditHash = this.auditLog.recordValidated(token, resource, 'denied', hptpNs, [maxUsesConstraint], ipAddress);
        return {
          granted: false,
          capability: cap,
          expiration: { expired: false, remaining_ns: remainNs.toString(), remaining_human: formatNsRemaining(remainNs), checked_at_hptp_ns: hptpNs },
          constraints: { all_satisfied: false, failed: [maxUsesConstraint] },
          audit_event_hash: auditHash,
        };
      }
      this.usageCounts.set(token.jti, currentCount);
    }

    const { granted, failed } = validateAllConstraints(cap.constraints.filter(c => c.type !== 'max_uses'), context);
    const result = granted ? 'granted' : 'denied';
    const auditHash = this.auditLog.recordValidated(token, resource, result as 'granted' | 'denied', hptpNs, failed, ipAddress);

    return {
      granted,
      capability: cap,
      expiration: {
        expired: false,
        remaining_ns: remainNs.toString(),
        remaining_human: formatNsRemaining(remainNs),
        checked_at_hptp_ns: hptpNs,
      },
      constraints: { all_satisfied: granted, failed },
      audit_event_hash: auditHash,
    };
  }

  private verifyRootSignature(rootSignature: string, parentToken: CapabilityToken): boolean {
    const tokenHash = this.auditLog.hashToken(parentToken);
    const signingKeys = getTlDsaSigningKeyPair();
    return tlDsaVerifyString(
      signingKeys.publicKey,
      tokenHash,
      rootSignature,
      signingKeys.secretKey,
      signingKeys.variant,
    );
  }

  private enforceAttenuation(
    parentConstraints: CapabilityConstraint[],
    newAttenuations: CapabilityConstraint[],
  ): { valid: boolean; violation?: string } {
    for (const attn of newAttenuations) {
      const existing = parentConstraints.find(c => c.type === attn.type);
      if (existing) {
        switch (attn.type) {
          case 'max_uses': {
            const parentMax = (existing as { type: 'max_uses'; value: number }).value;
            if ((attn as { type: 'max_uses'; value: number }).value > parentMax) {
              return { valid: false, violation: `max_uses cannot expand: ${(attn as any).value} > parent ${parentMax}` };
            }
            break;
          }
          case 'geo_country': {
            const parentCountries = (existing as { type: 'geo_country'; value: string[] }).value;
            const newCountries = (attn as { type: 'geo_country'; value: string[] }).value;
            const expanded = newCountries.filter(c => !parentCountries.includes(c));
            if (expanded.length > 0) {
              return { valid: false, violation: `geo_country cannot expand: [${expanded.join(',')}] not in parent` };
            }
            break;
          }
        }
      }
    }
    return { valid: true };
  }

  delegateCapability(
    parentSigned: SignedCapabilityToken | DelegatedCapabilityToken,
    newSubject: string,
    attenuations: CapabilityConstraint[],
    ttlSeconds?: number,
    ipAddress?: string,
  ): DelegationResult {
    const hptpNs = getHptpNanoseconds();
    const jti = `cap_del_${crypto.randomUUID().replace(/-/g, '')}`;

    let parentToken: CapabilityToken;
    let rootSignature: string;
    let rootTokenHash: string;
    let existingChain: DelegationCaveat[] = [];
    let parentJti: string;

    if ('delegation_chain' in parentSigned) {
      parentToken = parentSigned.token;
      rootSignature = parentSigned.root_signature;
      rootTokenHash = parentSigned.root_token_hash;
      existingChain = [...parentSigned.delegation_chain];
      parentJti = parentSigned.parent_jti;
    } else {
      parentToken = parentSigned.token;
      rootSignature = parentSigned.signature;
      rootTokenHash = this.auditLog.hashToken(parentToken);
      parentJti = parentToken.jti;

      if (!this.verifyRootSignature(rootSignature, parentToken)) {
        throw new Error('Root TL-DSA signature verification failed — delegation rejected');
      }
    }

    const parentConstraints = parentToken.cap.flatMap(c => c.constraints);
    const attenuationCheck = this.enforceAttenuation(parentConstraints, attenuations);
    if (!attenuationCheck.valid) {
      throw new Error(`Attenuation violation — authority can only diminish: ${attenuationCheck.violation}`);
    }

    const parentTokenHash = this.auditLog.hashToken(parentToken);
    let hmacKey: string;
    if (existingChain.length === 0) {
      const derivedKey = deriveHmacKey(Buffer.from(rootSignature, 'hex'), parentJti);
      hmacKey = derivedKey.toString('hex');
    } else {
      hmacKey = existingChain[existingChain.length - 1].hmac;
    }

    const newCaveats: DelegationCaveat[] = attenuations.map((constraint, idx) => {
      const depth = existingChain.length + idx;
      const caveatData = `${parentJti}|${depth}|${JSON.stringify(constraint)}`;
      const hmac = crypto
        .createHmac('sha256', hmacKey)
        .update(caveatData)
        .digest('hex');
      hmacKey = hmac;
      return { constraint, hmac };
    });

    const newCapabilities: Capability[] = parentToken.cap.map(parentCap => {
      let expNs: string;
      if (ttlSeconds !== undefined) {
        const proposedExp = BigInt(hptpNs) + BigInt(ttlSeconds) * 1_000_000_000n;
        const parentExp = BigInt(parentCap.exp);
        expNs = (proposedExp < parentExp ? proposedExp : parentExp).toString();
      } else {
        expNs = parentCap.exp;
      }

      return {
        res: parentCap.res,
        constraints: [...parentCap.constraints, ...attenuations],
        exp: expNs,
      };
    });

    const childToken = createCapabilityToken(newSubject, newCapabilities, hptpNs, jti);

    const delegatedToken: DelegatedCapabilityToken = {
      root_signature: rootSignature,
      root_algorithm: 'TL-DSA',
      root_token_hash: rootTokenHash,
      token: childToken,
      delegation_chain: [...existingChain, ...newCaveats],
      chain_depth: existingChain.length + newCaveats.length,
      parent_jti: parentJti,
      parent_token_hash: parentTokenHash,
    };

    const attenuationNames = attenuations.map(a => `${a.type}=${JSON.stringify(a.value)}`);
    const auditHash = this.auditLog.recordDelegated(
      parentToken,
      childToken,
      attenuationNames,
      hptpNs,
    );

    const maxTtl = ttlSeconds || Math.max(
      ...parentToken.cap.map(c => Number((BigInt(c.exp) - BigInt(hptpNs)) / 1_000_000_000n))
    );

    return {
      delegated_token: delegatedToken,
      audit_event_hash: auditHash,
      chain_depth: delegatedToken.chain_depth,
      expiration: this.computeExpirationWindow(maxTtl > 0 ? maxTtl : 60),
    };
  }

  verifyDelegationChain(delegatedToken: DelegatedCapabilityToken): {
    valid: boolean;
    chain_depth: number;
    root_verified: boolean;
    caveats_verified: boolean;
    failed_at_depth?: number;
    error?: string;
  } {
    const rootSigValid = this.verifyRootSignatureFromChain(delegatedToken);
    if (!rootSigValid) {
      return {
        valid: false,
        chain_depth: delegatedToken.chain_depth,
        root_verified: false,
        caveats_verified: false,
        error: 'Root TL-DSA signature verification failed',
      };
    }

    const derivedKeyBuf = deriveHmacKey(Buffer.from(delegatedToken.root_signature, 'hex'), delegatedToken.parent_jti);
    let hmacKey = derivedKeyBuf.toString('hex');

    for (let i = 0; i < delegatedToken.delegation_chain.length; i++) {
      const caveat = delegatedToken.delegation_chain[i];
      const caveatData = `${delegatedToken.parent_jti}|${i}|${JSON.stringify(caveat.constraint)}`;
      const expectedHmac = crypto
        .createHmac('sha256', hmacKey)
        .update(caveatData)
        .digest('hex');

      if (!crypto.timingSafeEqual(Buffer.from(caveat.hmac, 'hex'), Buffer.from(expectedHmac, 'hex'))) {
        return {
          valid: false,
          chain_depth: delegatedToken.chain_depth,
          root_verified: true,
          caveats_verified: false,
          failed_at_depth: i,
          error: `HMAC verification failed at caveat depth ${i} — token binding mismatch`,
        };
      }
      hmacKey = caveat.hmac;
    }

    return {
      valid: true,
      chain_depth: delegatedToken.chain_depth,
      root_verified: true,
      caveats_verified: true,
    };
  }

  private verifyRootSignatureFromChain(delegatedToken: DelegatedCapabilityToken): boolean {
    const signingKeys = getTlDsaSigningKeyPair();
    return tlDsaVerifyString(
      signingKeys.publicKey,
      delegatedToken.root_token_hash,
      delegatedToken.root_signature,
      signingKeys.secretKey,
      signingKeys.variant,
    );
  }

  validateDelegatedCapability(
    delegatedToken: DelegatedCapabilityToken,
    resource: string,
    context: VerificationContext,
    ipAddress?: string,
  ): CapabilityValidationResult & { delegation_chain_valid: boolean; chain_depth: number } {
    const chainResult = this.verifyDelegationChain(delegatedToken);
    if (!chainResult.valid) {
      const hptpNs = getHptpNanoseconds();
      return {
        granted: false,
        capability: null,
        expiration: { expired: false, remaining_ns: '0', remaining_human: chainResult.error || 'chain invalid', checked_at_hptp_ns: hptpNs },
        constraints: { all_satisfied: false, failed: [] },
        audit_event_hash: '',
        delegation_chain_valid: false,
        chain_depth: chainResult.chain_depth,
      };
    }

    const delegatedTokenHash = this.auditLog.hashToken(delegatedToken.token);
    const syntheticKeys = getTlDsaSigningKeyPair();
    const syntheticSigned: SignedCapabilityToken = {
      token: delegatedToken.token,
      signature: tlDsaSignString(syntheticKeys.secretKey, delegatedTokenHash, syntheticKeys.variant),
      algorithm: 'TL-DSA',
    };

    const result = this.validateCapability(syntheticSigned, resource, context, ipAddress);
    return {
      ...result,
      delegation_chain_valid: true,
      chain_depth: chainResult.chain_depth,
    };
  }

  runExpirationDemo(): {
    demo_id: string;
    scenario: string;
    steps: {
      step: number;
      action: string;
      hptp_ns: string;
      result: string;
      details: Record<string, unknown>;
    }[];
    summary: string;
  } {
    const demoId = `demo_${crypto.randomUUID().replace(/-/g, '').slice(0, 12)}`;
    const steps: any[] = [];

    const ttl = 2;
    const { signedToken, expiration } = this.issueCapabilityToken(
      'demo-user@plenumnet.io',
      [{ res: 'demo:read', constraints: [], ttlSeconds: ttl }],
      '127.0.0.1',
    );

    steps.push({
      step: 1,
      action: 'ISSUE',
      hptp_ns: expiration.issued_hptp_ns,
      result: 'capability_issued',
      details: {
        jti: signedToken.token.jti,
        resource: 'demo:read',
        ttl_seconds: ttl,
        expires_hptp_ns: expiration.expires_hptp_ns,
        algorithm: 'TL-DSA',
      },
    });

    const preValidation = this.validateCapability(signedToken, 'demo:read', {}, '127.0.0.1');
    steps.push({
      step: 2,
      action: 'VALIDATE_BEFORE_EXPIRY',
      hptp_ns: preValidation.expiration.checked_at_hptp_ns,
      result: preValidation.granted ? 'granted' : 'denied',
      details: {
        remaining_ns: preValidation.expiration.remaining_ns,
        remaining_human: preValidation.expiration.remaining_human,
        expired: preValidation.expiration.expired,
      },
    });

    const wrongResource = this.validateCapability(signedToken, 'admin:delete', {}, '127.0.0.1');
    steps.push({
      step: 3,
      action: 'VALIDATE_WRONG_RESOURCE',
      hptp_ns: wrongResource.expiration.checked_at_hptp_ns,
      result: 'denied',
      details: {
        reason: 'no matching capability for resource admin:delete',
        granted: wrongResource.granted,
      },
    });

    const constrainedIssue = this.issueCapabilityToken(
      'demo-restricted@plenumnet.io',
      [{
        res: 'notification:send',
        constraints: [
          { type: 'recipient_domain', value: '@lawfirm.ca' },
          { type: 'max_uses', value: 3 },
          { type: 'template', value: 'maestro.review.assigned' },
        ],
        ttlSeconds: 300,
      }],
      '127.0.0.1',
    );

    steps.push({
      step: 4,
      action: 'ISSUE_CONSTRAINED',
      hptp_ns: constrainedIssue.expiration.issued_hptp_ns,
      result: 'capability_issued_with_constraints',
      details: {
        jti: constrainedIssue.signedToken.token.jti,
        constraint_count: 3,
        constraints: ['recipient_domain=@lawfirm.ca', 'max_uses=3', 'template=maestro.review.assigned'],
        ttl_seconds: 300,
      },
    });

    const constraintCheck = this.validateCapability(
      constrainedIssue.signedToken,
      'notification:send',
      { recipient: 'alice@lawfirm.ca', template: 'maestro.review.assigned' },
      '127.0.0.1',
    );
    steps.push({
      step: 5,
      action: 'VALIDATE_CONSTRAINED_MATCH',
      hptp_ns: constraintCheck.expiration.checked_at_hptp_ns,
      result: constraintCheck.granted ? 'granted' : 'denied',
      details: {
        remaining: constraintCheck.expiration.remaining_human,
        constraints_satisfied: constraintCheck.constraints.all_satisfied,
      },
    });

    const constraintFail = this.validateCapability(
      constrainedIssue.signedToken,
      'notification:send',
      { recipient: 'bob@otherfirm.com', template: 'maestro.review.assigned' },
      '127.0.0.1',
    );
    steps.push({
      step: 6,
      action: 'VALIDATE_CONSTRAINED_FAIL',
      hptp_ns: constraintFail.expiration.checked_at_hptp_ns,
      result: 'denied',
      details: {
        reason: 'recipient_domain constraint failed',
        failed_constraints: constraintFail.constraints.failed.map(c => c.type),
      },
    });

    return {
      demo_id: demoId,
      scenario: 'HPTP-Bound Capability Expiration + Constraint Enforcement',
      steps,
      summary: `Demonstrated ${steps.length} capability lifecycle steps: issuance with TL-DSA signing, HPTP-bound nanosecond expiration, resource mismatch rejection, constraint enforcement (recipient_domain, max_uses, template). All operations timestamped via HPTP/1.0.`,
    };
  }

  runDelegationDemo(): {
    demo_id: string;
    scenario: string;
    steps: {
      step: number;
      action: string;
      hptp_ns: string;
      result: string;
      details: Record<string, unknown>;
    }[];
    summary: string;
  } {
    const demoId = `demo_del_${crypto.randomUUID().replace(/-/g, '').slice(0, 12)}`;
    const steps: any[] = [];

    const root = this.issueCapabilityToken(
      'tenant-admin@bank.ca',
      [{
        res: 'evidence:read',
        constraints: [{ type: 'vault_id', value: 'vault-forensics-001' }],
        ttlSeconds: 3600,
      }],
      '10.0.0.1',
    );

    steps.push({
      step: 1,
      action: 'ISSUE_ROOT',
      hptp_ns: root.expiration.issued_hptp_ns,
      result: 'root_capability_issued',
      details: {
        jti: root.signedToken.token.jti,
        subject: 'tenant-admin@bank.ca',
        resource: 'evidence:read',
        algorithm: 'TL-DSA',
        ttl_seconds: 3600,
      },
    });

    const delegation1 = this.delegateCapability(
      root.signedToken,
      'auditor@external-firm.com',
      [{ type: 'recipient_domain', value: '@external-firm.com' }],
      1800,
      '10.0.0.2',
    );

    steps.push({
      step: 2,
      action: 'DELEGATE_LEVEL_1',
      hptp_ns: delegation1.expiration.issued_hptp_ns,
      result: 'delegated_with_attenuation',
      details: {
        parent_jti: root.signedToken.token.jti,
        child_jti: delegation1.delegated_token.token.jti,
        new_subject: 'auditor@external-firm.com',
        attenuation: 'recipient_domain=@external-firm.com',
        chain_depth: delegation1.chain_depth,
        ttl_seconds: 1800,
      },
    });

    const delegation2 = this.delegateCapability(
      delegation1.delegated_token,
      'junior-analyst@external-firm.com',
      [
        { type: 'max_uses', value: 10 },
        { type: 'document_id', value: 'DOC-2026-0042' },
      ],
      900,
      '10.0.0.3',
    );

    steps.push({
      step: 3,
      action: 'DELEGATE_LEVEL_2',
      hptp_ns: delegation2.expiration.issued_hptp_ns,
      result: 'further_attenuated',
      details: {
        parent_jti: delegation1.delegated_token.token.jti,
        child_jti: delegation2.delegated_token.token.jti,
        new_subject: 'junior-analyst@external-firm.com',
        attenuations: ['max_uses=10', 'document_id=DOC-2026-0042'],
        chain_depth: delegation2.chain_depth,
        total_constraints: delegation2.delegated_token.token.cap[0]?.constraints.length,
      },
    });

    const chainVerify = this.verifyDelegationChain(delegation2.delegated_token);
    steps.push({
      step: 4,
      action: 'VERIFY_CHAIN',
      hptp_ns: getHptpNanoseconds(),
      result: chainVerify.valid ? 'chain_valid' : 'chain_invalid',
      details: {
        chain_depth: chainVerify.chain_depth,
        root_verified: chainVerify.root_verified,
        caveats_verified: chainVerify.caveats_verified,
      },
    });

    const validAccess = this.validateDelegatedCapability(
      delegation2.delegated_token,
      'evidence:read',
      {
        vault_id: 'vault-forensics-001',
        recipient: 'junior-analyst@external-firm.com',
        document_id: 'DOC-2026-0042',
      },
      '10.0.0.3',
    );

    steps.push({
      step: 5,
      action: 'VALIDATE_DELEGATED_VALID',
      hptp_ns: validAccess.expiration.checked_at_hptp_ns,
      result: validAccess.granted ? 'granted' : 'denied',
      details: {
        delegation_chain_valid: validAccess.delegation_chain_valid,
        chain_depth: validAccess.chain_depth,
        remaining: validAccess.expiration.remaining_human,
        constraints_satisfied: validAccess.constraints.all_satisfied,
      },
    });

    const invalidAccess = this.validateDelegatedCapability(
      delegation2.delegated_token,
      'evidence:read',
      {
        vault_id: 'vault-forensics-001',
        recipient: 'junior-analyst@external-firm.com',
        document_id: 'DOC-2026-9999',
      },
      '10.0.0.3',
    );

    steps.push({
      step: 6,
      action: 'VALIDATE_DELEGATED_WRONG_DOC',
      hptp_ns: invalidAccess.expiration.checked_at_hptp_ns,
      result: 'denied',
      details: {
        reason: 'document_id constraint failed — attenuation enforced',
        granted: invalidAccess.granted,
        failed_constraints: invalidAccess.constraints.failed.map(c => c.type),
      },
    });

    return {
      demo_id: demoId,
      scenario: 'HMAC-Chained Delegation with Attenuation',
      steps,
      summary: `Demonstrated ${steps.length} delegation lifecycle steps: TL-DSA root issuance, two levels of HMAC-chained attenuation (recipient_domain → max_uses + document_id), cryptographic chain verification, successful constrained access, and attenuation enforcement (wrong document rejected). Authority diminishes through the chain — never grows.`,
    };
  }

  getAuditStats() {
    return this.auditLog.getStats();
  }
}

export const capabilityService = new CapabilityService();
