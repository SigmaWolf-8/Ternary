import { SecurityMode, OperationType, PaymentGateway } from './operation-request';

export interface UnifiedOperationMetadata {
  operationId: string;
  batchRef: string;
  kernelOpId: string;
  version: '1.0';
  operationType: OperationType;
  securityMode: SecurityMode;
  
  paymentContext?: {
    gateway: PaymentGateway;
    paymentId: string;
    amount: number;
    currency: string;
    settledAt?: string;
    metadata?: Record<string, string>;
  };
  
  ternaryContext?: {
    representation: 'A' | 'B' | 'C';
    tritCount?: number;
    dataHash?: string;
    compressionRatio?: number;
    efficiencyGain?: string;
  };
  
  timingContext: {
    batchStartTs: string;
    batchEndTs?: string;
    durationNs?: number;
    clockSource: ClockSource;
    hptpSyncStatus?: HPTPSyncStatus;
  };
  
  blockchainRefs?: {
    hedera?: HederaReference;
    xrpl?: XRPLReference;
    algorand?: AlgorandReference;
  };
  
  auditTrail: AuditEntry[];
}

export interface ClockSource {
  type: 'system' | 'gps' | 'ptp' | 'ntp' | 'atomic';
  accuracy: 'millisecond' | 'microsecond' | 'nanosecond' | 'femtosecond';
  sourceId?: string;
  lastSyncAt?: string;
}

export interface HPTPSyncStatus {
  synchronized: boolean;
  offsetNs: number;
  jitterNs: number;
  lastSyncAt: string;
  peerCount: number;
}

export interface HederaReference {
  topicId: string;
  sequenceNumber: number;
  transactionId: string;
  consensusTimestamp: string;
  runningHash?: string;
}

export interface XRPLReference {
  ledgerIndex: number;
  transactionHash: string;
  validated: boolean;
  fee: string;
  destinationTag?: number;
}

export interface AlgorandReference {
  appId: number;
  round: number;
  txId: string;
  groupId?: string;
  innerTxCount?: number;
}

export interface AuditEntry {
  timestamp: string;
  event: string;
  actor: string;
  details: Record<string, unknown>;
  signature?: string;
}

export function createUnifiedMetadata(
  operationId: string,
  operationType: OperationType,
  securityMode: SecurityMode
): UnifiedOperationMetadata {
  const now = new Date().toISOString();
  const batchRef = `BATCH_${Date.now()}_${operationId.slice(0, 8)}`;
  const kernelOpId = `KERNEL_${Date.now()}_${operationId.slice(0, 8)}`;
  
  return {
    operationId,
    batchRef,
    kernelOpId,
    version: '1.0',
    operationType,
    securityMode,
    timingContext: {
      batchStartTs: now,
      clockSource: {
        type: 'system',
        accuracy: 'nanosecond'
      }
    },
    auditTrail: [{
      timestamp: now,
      event: 'operation_created',
      actor: 'sfk-core-api',
      details: { operationType, securityMode }
    }]
  };
}

export function addAuditEntry(
  metadata: UnifiedOperationMetadata,
  event: string,
  actor: string,
  details: Record<string, unknown>
): void {
  metadata.auditTrail.push({
    timestamp: new Date().toISOString(),
    event,
    actor,
    details
  });
}

export function completeOperation(metadata: UnifiedOperationMetadata): void {
  const now = new Date().toISOString();
  const startTime = new Date(metadata.timingContext.batchStartTs).getTime();
  const endTime = new Date(now).getTime();
  
  metadata.timingContext.batchEndTs = now;
  metadata.timingContext.durationNs = (endTime - startTime) * 1_000_000;
  
  addAuditEntry(metadata, 'operation_completed', 'sfk-core-api', {
    durationMs: endTime - startTime
  });
}
