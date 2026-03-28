/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * OPS CHANNEL SERVICE — Multi-operator access control, telemetry aggregation,
 * audit logging, and operations message routing for the daemon remote
 * operations channel.
 *
 * @version 1.0.0
 */

import crypto from 'crypto';
import {
  type OpsMessage,
  type OpsMessageType,
  type OpsErrorCode,
  type OpsPermissionScope,
  type OperatorEntry,
  type TelemetryMessage,
  type NodeTelemetrySnapshot,
  type OpsStatusResponse,
  type OpsAuditEntry,
  type OpsErrorMessage,
  isOpsMessageType,
  requiresSignature,
  isScopeAuthorized,
  OPS_PROTOCOL_VERSION,
} from '@shared/ops-protocol';
import { tis27Hash } from '../crypto/sponge-hash';

let verifyNativeFn: ((pk: Buffer, msg: Buffer, sig: Buffer) => boolean) | null = null;
async function initTlDsaBridge() {
  try {
    const bridge = await import('../crypto/tl-dsa-bridge');
    const fn = bridge.verifyNative;
    if (typeof fn === 'function') {
      verifyNativeFn = fn;
      console.log('[ops] TL-DSA native signature verification loaded');
    }
  } catch {
    console.warn('[ops] TL-DSA native bridge not available — ops will reject all signed operations until verifier is loaded');
  }
}
initTlDsaBridge();

export interface OpsChannelConfig {
  operators: OperatorEntry[];
  opsEnabled: boolean;
}

interface NodeOpsState {
  address: string;
  nodeId: string;
  lastTelemetry: TelemetryMessage | null;
  lastSeen: number;
  opsEnabled: boolean;
  activeTailRequests: Map<string, string>;
  activeTransfers: Map<string, { startedAt: number; lastChunk: number }>;
}

export class OpsChannelService {
  private operators: Map<string, OperatorEntry> = new Map();
  private nodeStates: Map<string, NodeOpsState> = new Map();
  private auditLog: OpsAuditEntry[] = [];
  private startTime: number = Date.now();
  private opsEnabled: boolean = false;

  constructor(config?: OpsChannelConfig) {
    if (config) {
      this.opsEnabled = config.opsEnabled;
      for (const op of config.operators) {
        this.operators.set(op.keyFingerprint, op);
      }
    }
  }

  isOpsEnabled(): boolean {
    return this.opsEnabled;
  }

  setOpsEnabled(enabled: boolean): void {
    this.opsEnabled = enabled;
  }

  registerOperator(operator: OperatorEntry): void {
    this.operators.set(operator.keyFingerprint, operator);
  }

  removeOperator(fingerprint: string): boolean {
    return this.operators.delete(fingerprint);
  }

  getOperator(fingerprint: string): OperatorEntry | undefined {
    return this.operators.get(fingerprint);
  }

  listOperators(): OperatorEntry[] {
    return Array.from(this.operators.values());
  }

  validateOperatorScope(fingerprint: string, messageType: OpsMessageType): {
    valid: boolean;
    errorCode?: OpsErrorCode;
    operator?: OperatorEntry;
  } {
    const operator = this.operators.get(fingerprint);
    if (!operator) {
      return { valid: false, errorCode: 'SIGNATURE_INVALID' };
    }
    if (!isScopeAuthorized(operator.scope, messageType)) {
      return { valid: false, errorCode: 'SCOPE_VIOLATION', operator };
    }
    return { valid: true, operator };
  }

  validateOpsMessage(msg: any): {
    valid: boolean;
    errorCode?: OpsErrorCode;
    errorMessage?: string;
  } {
    if (!this.opsEnabled) {
      return { valid: false, errorCode: 'OPS_DISABLED', errorMessage: 'The operations channel is inactive on this node' };
    }

    if (!msg.type || !isOpsMessageType(msg.type)) {
      return { valid: false, errorMessage: `Unknown ops message type: ${msg.type}` };
    }

    if (!msg.node_id) {
      return { valid: false, errorMessage: 'Missing node_id' };
    }

    if (!msg.request_id) {
      return { valid: false, errorMessage: 'Missing request_id' };
    }

    if (requiresSignature(msg.type as OpsMessageType)) {
      if (!msg.signature) {
        return { valid: false, errorCode: 'SIGNATURE_MISSING', errorMessage: 'Signature required for this operation' };
      }
      if (!msg.operator_fingerprint) {
        return { valid: false, errorCode: 'SIGNATURE_MISSING', errorMessage: 'Operator fingerprint required' };
      }
      const scopeCheck = this.validateOperatorScope(msg.operator_fingerprint, msg.type as OpsMessageType);
      if (!scopeCheck.valid) {
        return {
          valid: false,
          errorCode: scopeCheck.errorCode,
          errorMessage: scopeCheck.errorCode === 'SCOPE_VIOLATION'
            ? `Operator ${scopeCheck.operator?.name || 'unknown'} (scope: ${scopeCheck.operator?.scope}) not authorized for ${msg.type}`
            : `Unknown operator fingerprint: ${msg.operator_fingerprint}`,
        };
      }

      if (!scopeCheck.operator?.publicKey) {
        return { valid: false, errorCode: 'SIGNATURE_INVALID', errorMessage: 'Operator has no public key registered — cannot verify signature' };
      }
      if (!verifyNativeFn) {
        return { valid: false, errorCode: 'SIGNATURE_INVALID', errorMessage: 'TL-DSA verifier unavailable — refusing to forward unverified operations' };
      }
      try {
        const canonicalPayload: Record<string, any> = {};
        for (const k of Object.keys(msg).sort()) {
          if (k !== 'signature' && k !== 'operator_fingerprint') {
            canonicalPayload[k] = (msg as Record<string, any>)[k];
          }
        }
        const payloadBytes = Buffer.from(JSON.stringify(canonicalPayload));
        const pubKeyBuf = Buffer.from(scopeCheck.operator.publicKey, 'hex');
        const sigBuf = Buffer.from(msg.signature, 'hex');
        const verified = verifyNativeFn(pubKeyBuf, payloadBytes, sigBuf);
        if (!verified) {
          return { valid: false, errorCode: 'SIGNATURE_INVALID', errorMessage: 'TL-DSA signature verification failed at relay' };
        }
      } catch (e) {
        return { valid: false, errorCode: 'SIGNATURE_INVALID', errorMessage: `Signature verification error: ${e}` };
      }
    }

    return { valid: true };
  }

  makeOpsError(
    nodeId: string,
    requestId: string,
    errorCode: OpsErrorCode,
    message: string,
    originalType?: OpsMessageType,
  ): OpsErrorMessage {
    return {
      type: 'ops-error',
      node_id: nodeId,
      request_id: `err-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      error_code: errorCode,
      message,
      original_request_id: requestId,
      original_type: originalType,
      timestamp: new Date().toISOString(),
    };
  }

  updateNodeTelemetry(nodeId: string, address: string, telemetry: TelemetryMessage): void {
    let state = this.nodeStates.get(nodeId);
    if (!state) {
      state = {
        address,
        nodeId,
        lastTelemetry: null,
        lastSeen: Date.now(),
        opsEnabled: true,
        activeTailRequests: new Map(),
        activeTransfers: new Map(),
      };
      this.nodeStates.set(nodeId, state);
    }
    state.lastTelemetry = telemetry;
    state.lastSeen = Date.now();
    state.address = address;
  }

  updateNodeSeen(nodeId: string, address: string): void {
    let state = this.nodeStates.get(nodeId);
    if (!state) {
      state = {
        address,
        nodeId,
        lastTelemetry: null,
        lastSeen: Date.now(),
        opsEnabled: true,
        activeTailRequests: new Map(),
        activeTransfers: new Map(),
      };
      this.nodeStates.set(nodeId, state);
    }
    state.lastSeen = Date.now();
  }

  markNodeDisconnected(nodeId: string): void {
    const state = this.nodeStates.get(nodeId);
    if (state) {
      state.activeTailRequests.clear();
      state.activeTransfers.clear();
    }
  }

  getNodeSnapshot(nodeId: string): NodeTelemetrySnapshot | null {
    const state = this.nodeStates.get(nodeId);
    if (!state) return null;
    const now = Date.now();
    const age = now - state.lastSeen;
    let connectionState: "connected" | "disconnected" | "suspect" = "disconnected";
    if (age < 90_000) connectionState = "connected";
    else if (age < 300_000) connectionState = "suspect";
    return {
      node_id: state.nodeId,
      address: state.address,
      last_seen: new Date(state.lastSeen).toISOString(),
      last_telemetry: state.lastTelemetry,
      connection_state: connectionState,
      ops_enabled: state.opsEnabled,
    };
  }

  getOpsStatus(): OpsStatusResponse & { ops_enabled: boolean } {
    const nodes: NodeTelemetrySnapshot[] = [];
    for (const [, state] of this.nodeStates) {
      const snapshot = this.getNodeSnapshot(state.nodeId);
      if (snapshot) nodes.push(snapshot);
    }
    return {
      nodes,
      relay_uptime_seconds: Math.floor((Date.now() - this.startTime) / 1000),
      ops_version: OPS_PROTOCOL_VERSION,
      ops_enabled: this.opsEnabled,
    };
  }

  recordAuditEntry(entry: OpsAuditEntry): void {
    this.auditLog.push(entry);
    if (this.auditLog.length > 1000) {
      this.auditLog = this.auditLog.slice(-500);
    }
    this.persistAuditEntry(entry);
  }

  private persistAuditEntry(entry: OpsAuditEntry): void {
    try {
      const fs = require('fs');
      const path = require('path');
      const auditDir = path.join(process.cwd(), '.plenumnet');
      if (!fs.existsSync(auditDir)) {
        fs.mkdirSync(auditDir, { recursive: true });
      }
      const auditPath = path.join(auditDir, 'ops-audit.jsonl');
      const line = JSON.stringify(entry) + '\n';
      fs.appendFileSync(auditPath, line, 'utf-8');
    } catch {
    }
  }

  createAuditEntry(
    msg: OpsMessage,
    result: "success" | "failure" | "timeout" | "rejected",
    extras?: Partial<OpsAuditEntry>,
  ): OpsAuditEntry {
    const payloadStr = JSON.stringify(msg);
    const payloadHash = tis27Hash(Buffer.from(payloadStr, 'utf-8'));

    const operator = msg.operator_fingerprint
      ? this.operators.get(msg.operator_fingerprint)
      : undefined;

    return {
      timestamp: new Date().toISOString(),
      operation: msg.type,
      operator_name: operator?.name || 'unknown',
      operator_fingerprint: msg.operator_fingerprint || 'none',
      node_id: msg.node_id,
      request_id: msg.request_id,
      payload_tis27_hash: payloadHash,
      result,
      ...extras,
    };
  }

  getRecentAuditEntries(limit: number = 20): OpsAuditEntry[] {
    return this.auditLog.slice(-limit);
  }

  resolveNodeAddress(nodeId: string, connectedNodes: Map<string, any>): string | null {
    for (const [address, info] of connectedNodes) {
      if (info?.nodeId === nodeId || address === nodeId) {
        return address;
      }
    }
    const state = this.nodeStates.get(nodeId);
    if (state) return state.address;
    return null;
  }

  registerTailRequest(nodeId: string, requestId: string, filePath: string): void {
    const state = this.nodeStates.get(nodeId);
    if (state) {
      state.activeTailRequests.set(requestId, filePath);
    }
  }

  removeTailRequest(nodeId: string, requestId: string): void {
    const state = this.nodeStates.get(nodeId);
    if (state) {
      state.activeTailRequests.delete(requestId);
    }
  }

  registerTransfer(nodeId: string, transferId: string): void {
    const state = this.nodeStates.get(nodeId);
    if (state) {
      state.activeTransfers.set(transferId, { startedAt: Date.now(), lastChunk: 0 });
    }
  }

  updateTransferProgress(nodeId: string, transferId: string, chunkIndex: number): void {
    const state = this.nodeStates.get(nodeId);
    if (state) {
      const transfer = state.activeTransfers.get(transferId);
      if (transfer) transfer.lastChunk = chunkIndex;
    }
  }

  removeTransfer(nodeId: string, transferId: string): void {
    const state = this.nodeStates.get(nodeId);
    if (state) {
      state.activeTransfers.delete(transferId);
    }
  }

  getActiveTransfers(nodeId: string): Map<string, { startedAt: number; lastChunk: number }> {
    const state = this.nodeStates.get(nodeId);
    return state?.activeTransfers || new Map();
  }
}

export const opsChannelService = new OpsChannelService();
