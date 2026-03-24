/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * NODE WATCHDOG — Health monitoring, circuit breaker, error codes, audit logging
 * @version 1.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/services/node-watchdog.ts
 *
 * Implements FTS-style health state machine (Up/Suspect/Down),
 * standardized WebSocket error codes, reusable circuit breaker,
 * disconnect event logging, and Merkle-chained relay audit integration.
 */

import crypto from 'crypto';
import { getSharedAuditLog } from './capability-audit-events';

export type NodeHealthState = "up" | "suspect" | "down";

export const HEALTH_THRESHOLDS = {
  UP_MAX_AGE_MS: 60_000,
  SUSPECT_MAX_AGE_MS: 300_000,
} as const;

export function computeHealthState(lastSeenMs: number | null, nowMs: number): NodeHealthState {
  if (lastSeenMs === null) return "down";
  const age = nowMs - lastSeenMs;
  if (age <= HEALTH_THRESHOLDS.UP_MAX_AGE_MS) return "up";
  if (age <= HEALTH_THRESHOLDS.SUSPECT_MAX_AGE_MS) return "suspect";
  return "down";
}

export interface DisconnectEvent {
  timestamp: string;
  reason: string;
  code: number;
  eventType: "disconnect" | "reconnect" | "auth_fail" | "error" | "go_away" | "peer_offline";
}

const disconnectHistory = new Map<string, DisconnectEvent[]>();
const MAX_EVENTS_PER_NODE = 50;

export function recordDisconnectEvent(address: string, event: DisconnectEvent): void {
  if (!disconnectHistory.has(address)) {
    disconnectHistory.set(address, []);
  }
  const events = disconnectHistory.get(address)!;
  events.push(event);
  if (events.length > MAX_EVENTS_PER_NODE) {
    events.splice(0, events.length - MAX_EVENTS_PER_NODE);
  }
}

export function getDisconnectHistory(address: string): DisconnectEvent[] {
  return disconnectHistory.get(address) || [];
}

export const RELAY_ERROR_CODES = {
  ERR_AUTH_FAILED: { code: "ERR_AUTH_FAILED", message: "Authentication failed — address not registered or publicKey mismatch", wsClose: 1008 },
  ERR_SIGNATURE_INVALID: { code: "ERR_SIGNATURE_INVALID", message: "Challenge signature verification failed — private key proof required", wsClose: 1008 },
  ERR_SIGNATURE_REQUIRED: { code: "ERR_SIGNATURE_REQUIRED", message: "Challenge signature required — upgrade client to v0.3.0+", wsClose: 1008 },
  ERR_AUTH_TIMEOUT: { code: "ERR_AUTH_TIMEOUT", message: "Authentication timeout — must authenticate within 10 seconds", wsClose: 1008 },
  ERR_RATE_LIMITED: { code: "ERR_RATE_LIMITED", message: "Rate limit exceeded — too many requests", wsClose: 1008 },
  ERR_FRAME_MALFORMED: { code: "ERR_FRAME_MALFORMED", message: "Malformed frame — invalid JSON", wsClose: 1003 },
  ERR_FRAME_TOO_LARGE: { code: "ERR_FRAME_TOO_LARGE", message: "Frame too large — exceeds maximum allowed size", wsClose: 1009 },
  ERR_RELAY_TARGET_UNKNOWN: { code: "ERR_RELAY_TARGET_UNKNOWN", message: "Relay target unknown — destination address not connected", wsClose: undefined },
  ERR_RELAY_QUEUE_FULL: { code: "ERR_RELAY_QUEUE_FULL", message: "Relay queue full — destination queue at capacity", wsClose: undefined },
  ERR_UNKNOWN_MSG_TYPE: { code: "ERR_UNKNOWN_MSG_TYPE", message: "Unknown message type", wsClose: undefined },
  ERR_NOT_AUTHENTICATED: { code: "ERR_NOT_AUTHENTICATED", message: "Must authenticate first", wsClose: undefined },
  ERR_CIRCUIT_OPEN: { code: "ERR_CIRCUIT_OPEN", message: "Service circuit breaker is open — requests degraded", wsClose: undefined },
} as const;

export type RelayErrorCode = keyof typeof RELAY_ERROR_CODES;

export function makeErrorResponse(code: RelayErrorCode, msgType?: string): object {
  const errDef = RELAY_ERROR_CODES[code];
  return {
    type: "error",
    error: errDef.code,
    message: errDef.message,
    ...(msgType ? { offendingType: msgType } : {}),
  };
}

export type CircuitState = "closed" | "open" | "half-open";

export class CircuitBreaker {
  private state: CircuitState = "closed";
  private failureCount = 0;
  private lastFailureTime = 0;
  private lastStateChange = Date.now();

  constructor(
    private readonly name: string,
    private readonly failureThreshold: number = 5,
    private readonly resetTimeoutMs: number = 30_000,
    private readonly onStateChange?: (name: string, state: CircuitState) => void,
  ) {}

  getState(): CircuitState { return this.state; }
  getFailureCount(): number { return this.failureCount; }
  getLastStateChange(): number { return this.lastStateChange; }

  private transition(newState: CircuitState): void {
    if (this.state === newState) return;
    const oldState = this.state;
    this.state = newState;
    this.lastStateChange = Date.now();
    console.log(`[circuit-breaker] ${this.name}: ${oldState} -> ${newState} (failures=${this.failureCount})`);
    this.onStateChange?.(this.name, newState);
  }

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state === "open") {
      if (Date.now() - this.lastFailureTime >= this.resetTimeoutMs) {
        this.transition("half-open");
      } else {
        throw new Error(`Circuit breaker ${this.name} is OPEN`);
      }
    }

    try {
      const result = await fn();
      if (this.state === "half-open") {
        this.failureCount = 0;
        this.transition("closed");
      } else if (this.state === "closed" && this.failureCount > 0) {
        this.failureCount = 0;
      }
      return result;
    } catch (err) {
      this.recordFailure();
      throw err;
    }
  }

  recordFailure(): void {
    this.failureCount++;
    this.lastFailureTime = Date.now();
    if (this.failureCount >= this.failureThreshold) {
      this.transition("open");
    }
  }

  reset(): void {
    this.failureCount = 0;
    this.transition("closed");
  }

  getStats(): { name: string; state: CircuitState; failureCount: number; lastStateChange: number } {
    return {
      name: this.name,
      state: this.state,
      failureCount: this.failureCount,
      lastStateChange: this.lastStateChange,
    };
  }
}

export type RelayAuditEventType =
  | "relay.auth_success"
  | "relay.auth_failure"
  | "relay.disconnect"
  | "relay.reconnect"
  | "relay.error"
  | "relay.circuit_breaker"
  | "relay.go_away"
  | "relay.peer_offline";

export interface RelayAuditEntry {
  eventType: RelayAuditEventType;
  address: string;
  timestamp: string;
  details: Record<string, unknown>;
}

export function recordRelayAuditEvent(entry: RelayAuditEntry): string {
  const auditLog = getSharedAuditLog();
  const eventData = `${entry.eventType}|${entry.address}|${entry.timestamp}|${JSON.stringify(entry.details)}`;
  const eventHash = crypto.createHash('sha3-256').update(eventData).digest('hex');

  const auditEvent = {
    event: "capability.validated" as const,
    jti: `relay-${entry.eventType}-${Date.now()}`,
    capability_hash: eventHash,
    parent_event_hash: auditLog.getLastEventHash(),
    timestamp_hptp_ns: entry.timestamp,
    details: {
      resource: `relay:${entry.eventType}`,
      result: "granted" as const,
      subject: entry.address,
      ...entry.details,
    },
  };

  return auditLog.recordEvent(auditEvent);
}

const expectedNodesCache = new Set<string>();

export function getExpectedNodesCache(): Set<string> {
  return expectedNodesCache;
}

export function addExpectedNode(address: string): void {
  expectedNodesCache.add(address);
}

export function removeExpectedNode(address: string): void {
  expectedNodesCache.delete(address);
}

export function isExpectedNode(address: string): boolean {
  return expectedNodesCache.has(address);
}

export function syncExpectedNodesCache(addresses: string[]): void {
  expectedNodesCache.clear();
  for (const addr of addresses) {
    expectedNodesCache.add(addr);
  }
}
