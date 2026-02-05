import { v4 as uuidv4 } from 'uuid';
import { 
  OperationRequest, 
  SFKOperation, 
  OperationStatus,
  OperationType,
  SecurityMode 
} from '../models/operation-request';
import { OperationCreatedResponse } from '../models/operation-response';
import { createUnifiedMetadata, addAuditEntry, UnifiedOperationMetadata } from '../models/unified-metadata';
import { StateManager } from './state-manager';
import { TimingCoordinator } from './timing-coordinator';

export class OperationOrchestrator {
  private stateManager: StateManager;
  private timingCoordinator: TimingCoordinator;
  private operationStore: Map<string, SFKOperation> = new Map();
  private metadataStore: Map<string, UnifiedOperationMetadata> = new Map();

  constructor() {
    this.stateManager = new StateManager();
    this.timingCoordinator = new TimingCoordinator();
  }

  async createOperation(request: OperationRequest): Promise<OperationCreatedResponse> {
    const operationId = uuidv4();
    const now = new Date().toISOString();
    const timestamp = this.timingCoordinator.getTimestamp();
    
    const batchRef = `BATCH_${Date.now()}_${operationId.slice(0, 8)}`;
    const kernelOpId = `KERNEL_${Date.now()}_${operationId.slice(0, 8)}`;

    const operation: SFKOperation = {
      id: operationId,
      batchRef,
      kernelOpId,
      operationType: request.operationType,
      securityMode: request.securityMode,
      status: 'pending',
      priority: request.priority,
      createdAt: now,
      updatedAt: now,
      paymentContext: request.paymentContext,
      ternaryContext: request.ternaryContext,
      timingMetadata: {
        receivedAt: now
      },
      callbackUrl: request.callbackUrl,
      idempotencyKey: request.idempotencyKey,
      ttlSeconds: request.ttlSeconds
    };

    const metadata = createUnifiedMetadata(operationId, request.operationType, request.securityMode);
    
    if (request.paymentContext) {
      metadata.paymentContext = request.paymentContext;
    }
    if (request.ternaryContext) {
      metadata.ternaryContext = request.ternaryContext;
    }

    this.operationStore.set(operationId, operation);
    this.metadataStore.set(operationId, metadata);

    await this.stateManager.transition(operationId, 'pending', 'queued');
    operation.status = 'queued';
    operation.timingMetadata.queuedAt = new Date().toISOString();
    operation.updatedAt = new Date().toISOString();

    addAuditEntry(metadata, 'operation_queued', 'orchestrator', {
      priority: request.priority,
      estimatedWaitMs: this.estimateWaitTime(request.priority)
    });

    this.processOperationAsync(operationId);

    return {
      success: true,
      operationId,
      batchRef,
      kernelOpId,
      status: 'queued',
      queuedAt: operation.timingMetadata.queuedAt!,
      estimatedCompletionMs: this.estimateCompletionTime(request),
      trackingUrl: `/api/sfk/v1/status/${operationId}`
    };
  }

  private async processOperationAsync(operationId: string): Promise<void> {
    const operation = this.operationStore.get(operationId);
    const metadata = this.metadataStore.get(operationId);
    
    if (!operation || !metadata) return;

    try {
      await this.stateManager.transition(operationId, 'queued', 'processing');
      operation.status = 'processing';
      operation.startedAt = new Date().toISOString();
      operation.timingMetadata.processingStartedAt = operation.startedAt;
      operation.updatedAt = new Date().toISOString();

      addAuditEntry(metadata, 'processing_started', 'orchestrator', {});

      await this.simulateBlockchainWitness(operation, metadata);

      await this.stateManager.transition(operationId, 'processing', 'witnessed');
      operation.status = 'witnessed';
      operation.timingMetadata.witnessedAt = new Date().toISOString();
      operation.updatedAt = new Date().toISOString();

      addAuditEntry(metadata, 'witnessed', 'hedera-service', {
        transactionId: operation.blockchainRefs?.hedera?.transactionId
      });

      if (operation.paymentContext) {
        await this.simulateSettlement(operation, metadata);
        
        await this.stateManager.transition(operationId, 'witnessed', 'settled');
        operation.status = 'settled';
        operation.completedAt = new Date().toISOString();
        operation.timingMetadata.settledAt = operation.completedAt;
      } else {
        operation.status = 'settled';
        operation.completedAt = new Date().toISOString();
      }

      operation.updatedAt = new Date().toISOString();
      
      const startTime = new Date(operation.createdAt).getTime();
      const endTime = new Date(operation.completedAt).getTime();
      operation.timingMetadata.totalLatencyNs = (endTime - startTime) * 1_000_000;

      addAuditEntry(metadata, 'operation_completed', 'orchestrator', {
        totalLatencyMs: endTime - startTime
      });

    } catch (error) {
      operation.status = 'failed';
      operation.updatedAt = new Date().toISOString();
      operation.errorInfo = {
        code: 'PROCESSING_FAILED',
        message: error instanceof Error ? error.message : 'Unknown error',
        retryable: true,
        retryCount: 0,
        maxRetries: 3
      };

      addAuditEntry(metadata, 'operation_failed', 'orchestrator', {
        error: operation.errorInfo.message
      });
    }
  }

  private async simulateBlockchainWitness(operation: SFKOperation, metadata: UnifiedOperationMetadata): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 100));
    
    const hederaRef = {
      topicId: '0.0.12345',
      sequenceNumber: Math.floor(Math.random() * 1000000),
      transactionId: `0.0.12345@${Math.floor(Date.now() / 1000)}.${Math.floor(Math.random() * 1000000000)}`,
      consensusTimestamp: new Date().toISOString()
    };

    operation.blockchainRefs = { hedera: hederaRef };
    metadata.blockchainRefs = { hedera: hederaRef };
  }

  private async simulateSettlement(operation: SFKOperation, metadata: UnifiedOperationMetadata): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 50));
    
    const xrplRef = {
      ledgerIndex: Math.floor(Math.random() * 1000000) + 80000000,
      transactionHash: Array(64).fill(0).map(() => Math.floor(Math.random() * 16).toString(16)).join(''),
      validated: true,
      fee: '12'
    };

    operation.blockchainRefs = { ...operation.blockchainRefs, xrpl: xrplRef };
    metadata.blockchainRefs = { ...metadata.blockchainRefs, xrpl: xrplRef };

    if (operation.paymentContext) {
      operation.paymentContext.settledAt = new Date().toISOString();
      if (metadata.paymentContext) {
        metadata.paymentContext.settledAt = operation.paymentContext.settledAt;
      }
    }
  }

  getOperation(operationId: string): SFKOperation | undefined {
    return this.operationStore.get(operationId);
  }

  getMetadata(operationId: string): UnifiedOperationMetadata | undefined {
    return this.metadataStore.get(operationId);
  }

  listOperations(
    page: number = 1,
    limit: number = 20,
    filters?: { status?: OperationStatus; operationType?: OperationType }
  ): { operations: SFKOperation[]; total: number } {
    let operations = Array.from(this.operationStore.values());

    if (filters?.status) {
      operations = operations.filter(op => op.status === filters.status);
    }
    if (filters?.operationType) {
      operations = operations.filter(op => op.operationType === filters.operationType);
    }

    operations.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());

    const total = operations.length;
    const startIndex = (page - 1) * limit;
    const paginatedOps = operations.slice(startIndex, startIndex + limit);

    return { operations: paginatedOps, total };
  }

  private estimateWaitTime(priority: string): number {
    const baseTimes: Record<string, number> = {
      critical: 100,
      high: 500,
      normal: 2000,
      low: 5000
    };
    return baseTimes[priority] || 2000;
  }

  private estimateCompletionTime(request: OperationRequest): number {
    let baseTime = 5000;
    
    if (request.blockchainTargets?.witnessToHedera) baseTime += 3000;
    if (request.blockchainTargets?.settleOnXrpl) baseTime += 4000;
    if (request.blockchainTargets?.recordOnAlgorand) baseTime += 4500;
    
    return baseTime;
  }
}

export const orchestrator = new OperationOrchestrator();
