import { SFKOperation, OperationStatus, OperationType, SecurityMode } from './operation-request';

export interface OperationCreatedResponse {
  success: true;
  operationId: string;
  batchRef: string;
  kernelOpId: string;
  status: OperationStatus;
  queuedAt: string;
  estimatedCompletionMs: number;
  trackingUrl: string;
}

export interface OperationStatusResponse {
  operationId: string;
  batchRef: string;
  kernelOpId: string;
  status: OperationStatus;
  operationType: OperationType;
  securityMode: SecurityMode;
  progress: {
    currentStep: string;
    completedSteps: string[];
    remainingSteps: string[];
    percentComplete: number;
  };
  timing: {
    createdAt: string;
    startedAt?: string;
    estimatedCompletionAt?: string;
    completedAt?: string;
    elapsedMs: number;
  };
  blockchainStatus: {
    hederaWitnessed: boolean;
    hederaTransactionId?: string;
    xrplSettled: boolean;
    xrplTransactionHash?: string;
    algorandRecorded: boolean;
    algorandTxId?: string;
  };
}

export interface OperationErrorResponse {
  success: false;
  error: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
    retryable: boolean;
    suggestedRetryAfterMs?: number;
  };
  operationId?: string;
  requestId: string;
  timestamp: string;
}

export interface OperationTimelineEvent {
  event: string;
  timestamp: string;
  details: Record<string, unknown>;
  durationFromPreviousMs?: number;
}

export interface OperationTimelineResponse {
  operationId: string;
  status: OperationStatus;
  timeline: OperationTimelineEvent[];
  totalDurationMs: number;
}

export interface BatchOperationResult {
  operationId: string;
  success: boolean;
  error?: string;
}

export interface BatchOperationResponse {
  batchId: string;
  totalOperations: number;
  successCount: number;
  failureCount: number;
  results: BatchOperationResult[];
  submittedAt: string;
}

export function operationToStatusResponse(operation: SFKOperation): OperationStatusResponse {
  const now = Date.now();
  const createdAt = new Date(operation.createdAt).getTime();
  const elapsedMs = now - createdAt;

  const allSteps = ['received', 'validated', 'queued', 'processing', 'witnessed', 'settled'];
  const statusIndex = {
    pending: 0,
    queued: 2,
    processing: 3,
    witnessed: 4,
    settled: 5,
    failed: -1,
    cancelled: -1
  };

  const currentIndex = statusIndex[operation.status] ?? 0;
  const completedSteps = allSteps.slice(0, currentIndex);
  const remainingSteps = allSteps.slice(currentIndex + 1);
  const currentStep = allSteps[currentIndex] || 'unknown';

  return {
    operationId: operation.id,
    batchRef: operation.batchRef,
    kernelOpId: operation.kernelOpId,
    status: operation.status,
    operationType: operation.operationType,
    securityMode: operation.securityMode,
    progress: {
      currentStep,
      completedSteps,
      remainingSteps,
      percentComplete: Math.round((currentIndex / allSteps.length) * 100)
    },
    timing: {
      createdAt: operation.createdAt,
      startedAt: operation.startedAt,
      completedAt: operation.completedAt,
      elapsedMs
    },
    blockchainStatus: {
      hederaWitnessed: !!operation.blockchainRefs?.hedera,
      hederaTransactionId: operation.blockchainRefs?.hedera?.transactionId,
      xrplSettled: !!operation.blockchainRefs?.xrpl?.validated,
      xrplTransactionHash: operation.blockchainRefs?.xrpl?.transactionHash,
      algorandRecorded: !!operation.blockchainRefs?.algorand,
      algorandTxId: operation.blockchainRefs?.algorand?.txId
    }
  };
}
