/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * CAPABILITY AUDIT EVENTS — Merkle-chained event log
 * @version 1.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/services/capability-audit-events.ts
 *
 * Records capability lifecycle events (issued, validated, delegated,
 * revoked, expired, usage_exceeded) into a Merkle-chained audit log.
 * Each event carries an HPTP timestamp and a SHA3-256 hash reference
 * to the previous event, forming a cryptographic proof of causality.
 */

import crypto from 'crypto';
import fs from 'fs';
import path from 'path';
import { CapabilityConstraint } from '../../shared/types/capability-constraints';
import { CapabilityToken } from '../../shared/types/capability';

export type CapabilityEventType =
  | "capability.issued"
  | "capability.validated"
  | "capability.delegated"
  | "capability.revoked"
  | "capability.expired"
  | "capability.usage_exceeded";

export interface CapabilityAuditEvent {
  event: CapabilityEventType;
  jti: string;
  capability_hash: string;
  parent_event_hash: string;
  timestamp_hptp_ns: string;
  details: {
    resource?: string;
    result?: "granted" | "denied";
    failed_constraints?: CapabilityConstraint[];
    parent_jti?: string;
    attenuations_applied?: string[];
    subject?: string;
    constraint_count?: number;
    ip_address?: string;
  };
}

export class CapabilityAuditLog {
  private leaves: string[] = [];
  private persistPath: string;
  private lastEventHash: string;

  constructor(keysDirectory: string) {
    this.persistPath = path.join(keysDirectory, 'capability-audit.jsonl');
    this.lastEventHash = crypto.createHash('sha3-256').update('capability-genesis').digest('hex');
    this.loadFromDisk();
  }

  private loadFromDisk(): void {
    if (fs.existsSync(this.persistPath)) {
      const lines = fs.readFileSync(this.persistPath, 'utf8').split('\n').filter(Boolean);
      for (const line of lines) {
        try {
          const entry = JSON.parse(line);
          if (entry.hash) {
            this.leaves.push(entry.hash);
            this.lastEventHash = entry.hash;
          }
        } catch {}
      }
      if (this.leaves.length > 0) {
        console.log(`[cap-audit] Capability audit log restored: ${this.leaves.length} events from ${this.persistPath}`);
      }
    }
  }

  hashToken(token: CapabilityToken): string {
    const canonical = JSON.stringify(token, Object.keys(token).sort());
    return crypto.createHash('sha3-256').update(canonical).digest('hex');
  }

  recordEvent(event: CapabilityAuditEvent): string {
    const eventData = `${event.event}|${event.jti}|${event.capability_hash}|${event.parent_event_hash}|${event.timestamp_hptp_ns}`;
    const eventHash = crypto.createHash('sha3-256').update(eventData).digest('hex');

    this.leaves.push(eventHash);
    this.lastEventHash = eventHash;

    const entry = {
      hash: eventHash,
      event: event.event,
      jti: event.jti,
      ts: event.timestamp_hptp_ns,
      details: event.details,
    };

    try {
      fs.appendFileSync(this.persistPath, JSON.stringify(entry) + '\n');
    } catch (err) {
      console.error('[cap-audit] Failed to persist audit event:', (err as Error).message);
    }

    return eventHash;
  }

  recordIssued(
    token: CapabilityToken,
    hptpNs: string,
    ipAddress?: string,
  ): string {
    const capHash = this.hashToken(token);
    const event: CapabilityAuditEvent = {
      event: "capability.issued",
      jti: token.jti,
      capability_hash: capHash,
      parent_event_hash: this.lastEventHash,
      timestamp_hptp_ns: hptpNs,
      details: {
        subject: token.sub,
        constraint_count: token.cap.reduce((acc, c) => acc + c.constraints.length, 0),
        ip_address: ipAddress,
      },
    };
    return this.recordEvent(event);
  }

  recordValidated(
    token: CapabilityToken,
    resource: string,
    result: "granted" | "denied",
    hptpNs: string,
    failedConstraints?: CapabilityConstraint[],
    ipAddress?: string,
  ): string {
    const capHash = this.hashToken(token);
    const event: CapabilityAuditEvent = {
      event: "capability.validated",
      jti: token.jti,
      capability_hash: capHash,
      parent_event_hash: this.lastEventHash,
      timestamp_hptp_ns: hptpNs,
      details: {
        resource,
        result,
        ...(failedConstraints && failedConstraints.length > 0 ? { failed_constraints: failedConstraints } : {}),
        subject: token.sub,
        ip_address: ipAddress,
      },
    };
    return this.recordEvent(event);
  }

  recordDelegated(
    parentToken: CapabilityToken,
    childToken: CapabilityToken,
    attenuations: string[],
    hptpNs: string,
  ): string {
    const capHash = this.hashToken(childToken);
    const event: CapabilityAuditEvent = {
      event: "capability.delegated",
      jti: childToken.jti,
      capability_hash: capHash,
      parent_event_hash: this.lastEventHash,
      timestamp_hptp_ns: hptpNs,
      details: {
        parent_jti: parentToken.jti,
        attenuations_applied: attenuations,
        subject: childToken.sub,
      },
    };
    return this.recordEvent(event);
  }

  recordRevoked(
    token: CapabilityToken,
    hptpNs: string,
    reason?: string,
  ): string {
    const capHash = this.hashToken(token);
    const event: CapabilityAuditEvent = {
      event: "capability.revoked",
      jti: token.jti,
      capability_hash: capHash,
      parent_event_hash: this.lastEventHash,
      timestamp_hptp_ns: hptpNs,
      details: {
        subject: token.sub,
        resource: reason,
      },
    };
    return this.recordEvent(event);
  }

  recordExpired(
    jti: string,
    capabilityHash: string,
    hptpNs: string,
  ): string {
    const event: CapabilityAuditEvent = {
      event: "capability.expired",
      jti,
      capability_hash: capabilityHash,
      parent_event_hash: this.lastEventHash,
      timestamp_hptp_ns: hptpNs,
      details: {},
    };
    return this.recordEvent(event);
  }

  recordUsageExceeded(
    token: CapabilityToken,
    hptpNs: string,
    currentCount: number,
  ): string {
    const capHash = this.hashToken(token);
    const event: CapabilityAuditEvent = {
      event: "capability.usage_exceeded",
      jti: token.jti,
      capability_hash: capHash,
      parent_event_hash: this.lastEventHash,
      timestamp_hptp_ns: hptpNs,
      details: {
        subject: token.sub,
        constraint_count: currentCount,
      },
    };
    return this.recordEvent(event);
  }

  getRoot(): string {
    if (this.leaves.length === 0) {
      return crypto.createHash('sha3-256').update('capability-genesis').digest('hex');
    }
    let level = [...this.leaves];
    while (level.length > 1) {
      const next: string[] = [];
      for (let i = 0; i < level.length; i += 2) {
        const left = level[i];
        const right = level[i + 1] || left;
        next.push(crypto.createHash('sha3-256').update(left + right).digest('hex'));
      }
      level = next;
    }
    return level[0];
  }

  getDepth(): number {
    if (this.leaves.length === 0) return 0;
    return Math.ceil(Math.log2(this.leaves.length)) + 1;
  }

  getSize(): number {
    return this.leaves.length;
  }

  getLastEventHash(): string {
    return this.lastEventHash;
  }

  getStats(): {
    totalEvents: number;
    merkleRoot: string;
    merkleDepth: number;
    lastEventHash: string;
  } {
    return {
      totalEvents: this.leaves.length,
      merkleRoot: this.getRoot(),
      merkleDepth: this.getDepth(),
      lastEventHash: this.lastEventHash,
    };
  }
}
