/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * SFK OPERATIONS SERVICE
 * Location: server/services/sfk-operations-service.ts
 *
 * Manages the Salvi Framework Kernel operation lifecycle:
 *   initialization → ternary_processing → witnessing → finalization
 *
 * During the witnessing phase, operations that require blockchain
 * non-repudiation (Mode φ+ and Mode φ) submit a SHA-256 hash of
 * the operation result to Hedera Consensus Service via the
 * HederaWitnessingService.
 *
 * Operations are tracked in-memory with configurable retention.
 * Each operation produces a deterministic audit trail:
 *   operation_id → processing_result_hash → hedera_tx_id → consensus_timestamp
 */

import * as crypto from 'crypto';
import type { HederaWitnessingService } from './hedera-witnessing-service';
import type {
  SecurityMode,
} from '../salvi-core/unified-metadata-schema';
import type {
  OperationType,
  OperationStatus,
  OperationPhase,
  OperationParameters,
  SFKOperationRequest,
  SFKOperationResponse,
  SFKOperationStatusResponse,
  OperationComponents,
  OperationTimingMetrics,
  WitnessingComponentStatus,
  SettlementComponentStatus,
  TernaryProcessingStatus,
} from '../salvi-core/sfk-operations-api';
import {
  generateOperationId,
  createOperationResponse,
  createOperationStatusResponse,
  TIMING_GUARANTEES,
} from '../salvi-core/sfk-operations-api';

interface OperationState {
  id: string;
  request: SFKOperationRequest;
  status: OperationStatus;
  phase: OperationPhase;
  progress: number;
  queued_at: number;
  started_at?: number;
  completed_at?: number;
  result_hash?: string;
  witnessing: WitnessingComponentStatus;
  settlement: SettlementComponentStatus;
  ternary_processing: TernaryProcessingStatus;
}

export interface SFKOperationsConfig {
  maxConcurrentOps?: number;
  operationRetentionMs?: number;
  enableWitnessing?: boolean;
}

export class SFKOperationsService {
  private operations: Map<string, OperationState> = new Map();
  private hederaService: HederaWitnessingService | null;
  private config: Required<SFKOperationsConfig>;
  private evictionTimer: ReturnType<typeof setInterval>;
  private shutdownRequested = false;

  constructor(
    hederaService: HederaWitnessingService | null,
    config: SFKOperationsConfig = {},
  ) {
    this.hederaService = hederaService;
    this.config = {
      maxConcurrentOps: config.maxConcurrentOps ?? 100,
      operationRetentionMs: config.operationRetentionMs ?? 3_600_000,
      enableWitnessing: config.enableWitnessing ?? (hederaService !== null),
    };

    this.evictionTimer = setInterval(() => this.evictExpired(), 60_000);
  }

  async submitOperation(request: SFKOperationRequest): Promise<SFKOperationResponse> {
    const activeCount = [...this.operations.values()]
      .filter(op => op.status === 'queued' || op.status === 'in_progress' || op.status === 'witnessing')
      .length;
    if (activeCount >= this.config.maxConcurrentOps) {
      throw new Error(`Operation queue full (${activeCount}/${this.config.maxConcurrentOps})`);
    }

    const opId = generateOperationId();
    const securityMode = request.operation.parameters.security_mode;

    const state: OperationState = {
      id: opId,
      request,
      status: 'queued',
      phase: 'initialization',
      progress: 0,
      queued_at: Date.now(),
      witnessing: { status: 'pending' },
      settlement: { status: 'pending' },
      ternary_processing: { status: 'queued' },
    };

    this.operations.set(opId, state);

    this.executeOperation(opId).catch(err => {
      const op = this.operations.get(opId);
      if (op) {
        op.status = 'failed';
        op.phase = 'finalization';
        op.progress = 100;
      }
      console.error(`[sfk-ops] Operation ${opId} failed: ${(err as Error).message}`);
    });

    return createOperationResponse(opId, securityMode, new Date(Date.now() + 50));
  }

  getOperationStatus(operationId: string): SFKOperationStatusResponse | null {
    const op = this.operations.get(operationId);
    if (!op) return null;

    const components: OperationComponents = {
      witnessing: op.witnessing,
      settlement: op.settlement,
      ternary_processing: op.ternary_processing,
    };

    const timingMetrics: OperationTimingMetrics = {
      queued_at: new Date(op.queued_at).toISOString(),
      started_at: op.started_at ? new Date(op.started_at).toISOString() : undefined,
      current_timestamp: new Date().toISOString(),
      elapsed_ns: (Date.now() - op.queued_at) * 1_000_000,
    };

    return createOperationStatusResponse(
      op.id, op.status, op.phase, op.progress, components, timingMetrics,
    );
  }

  listOperations(filter?: { status?: OperationStatus; limit?: number }): {
    operations: Array<{
      id: string;
      status: OperationStatus;
      phase: OperationPhase;
      security_mode: SecurityMode;
      queued_at: string;
      hedera_tx_id?: string;
    }>;
    total: number;
    active: number;
  } {
    let ops = [...this.operations.values()];

    if (filter?.status) {
      ops = ops.filter(op => op.status === filter.status);
    }

    ops.sort((a, b) => b.queued_at - a.queued_at);

    const limit = filter?.limit ?? 50;
    const sliced = ops.slice(0, limit);

    const active = [...this.operations.values()]
      .filter(op => op.status === 'queued' || op.status === 'in_progress' || op.status === 'witnessing')
      .length;

    return {
      operations: sliced.map(op => ({
        id: op.id,
        status: op.status,
        phase: op.phase,
        security_mode: op.request.operation.parameters.security_mode,
        queued_at: new Date(op.queued_at).toISOString(),
        hedera_tx_id: op.witnessing.hedera_tx_id,
      })),
      total: this.operations.size,
      active,
    };
  }

  cancelOperation(operationId: string): boolean {
    const op = this.operations.get(operationId);
    if (!op) return false;
    if (op.status === 'completed' || op.status === 'failed' || op.status === 'cancelled') return false;

    op.status = 'cancelled';
    op.phase = 'finalization';
    op.progress = 100;
    op.completed_at = Date.now();
    return true;
  }

  getStats(): {
    total_operations: number;
    by_status: Record<string, number>;
    by_security_mode: Record<string, number>;
    witnessing_enabled: boolean;
    hedera_connected: boolean;
    average_completion_ms: number;
  } {
    const byStatus: Record<string, number> = {};
    const byMode: Record<string, number> = {};
    let completedCount = 0;
    let totalCompletionMs = 0;

    for (const op of this.operations.values()) {
      byStatus[op.status] = (byStatus[op.status] || 0) + 1;
      const mode = op.request.operation.parameters.security_mode;
      byMode[mode] = (byMode[mode] || 0) + 1;

      if (op.status === 'completed' && op.completed_at) {
        completedCount++;
        totalCompletionMs += op.completed_at - op.queued_at;
      }
    }

    return {
      total_operations: this.operations.size,
      by_status: byStatus,
      by_security_mode: byMode,
      witnessing_enabled: this.config.enableWitnessing,
      hedera_connected: this.hederaService?.isInitialized() ?? false,
      average_completion_ms: completedCount > 0 ? Math.round(totalCompletionMs / completedCount) : 0,
    };
  }

  private isAborted(op: OperationState): boolean {
    return op.status === 'cancelled' || this.shutdownRequested;
  }

  private async executeOperation(operationId: string): Promise<void> {
    const op = this.operations.get(operationId);
    if (!op || this.isAborted(op)) return;

    const securityMode = op.request.operation.parameters.security_mode;
    const requiresWitnessing = this.config.enableWitnessing &&
      (securityMode === 'phi_plus' || securityMode === 'phi');

    op.status = 'in_progress';
    op.phase = 'initialization';
    op.progress = 5;
    op.started_at = Date.now();

    if (this.isAborted(op)) return;

    op.phase = 'ternary_processing';
    op.progress = 10;
    op.ternary_processing = { status: 'processing', progress: 0 };

    const resultHash = await this.performTernaryProcessing(op);
    op.result_hash = resultHash;
    op.ternary_processing = { status: 'completed', progress: 100 };
    op.progress = 50;

    if (this.isAborted(op)) return;

    if (requiresWitnessing && this.hederaService?.isInitialized()) {
      op.phase = 'witnessing';
      op.status = 'witnessing';
      op.progress = 55;
      op.witnessing = { status: 'submitted' };

      try {
        const witnessResponse = await this.hederaService.submitWitness({
          operation_id: operationId,
          witness_type: op.request.operation.parameters.batch_size > 1
            ? 'MERKLE_ROOT_BATCH'
            : 'SINGLE_HASH',
          payload: {
            hash: resultHash,
            hash_algorithm: 'SHA256',
            encoding: 'hex',
          },
          metadata: {
            salvi_batch_ref: op.request.metadata.salvi_batch_ref,
            kernel_op_id: operationId,
            ternary_context: {
              security_mode: securityMode,
              phase_offset: op.request.operation.parameters.phase_offset,
              torsion_dimensions: op.request.operation.parameters.torsion_dimensions,
              batch_size: op.request.operation.parameters.batch_size,
              operation_count: 1,
            },
            payment_context: {
              gateway: op.request.operation.trigger.gateway,
              payment_id: op.request.operation.trigger.payment_id,
              amount: op.request.operation.trigger.settled_amount,
              currency: op.request.operation.trigger.settled_currency,
            },
            timing: {
              batch_start_ts: new Date(op.started_at!).toISOString(),
              batch_end_ts: new Date().toISOString(),
              duration_ns: (Date.now() - op.started_at!) * 1_000_000,
              femtosecond_sync_accuracy: 0,
            },
          },
          topic: {
            id: this.hederaService.getTopicId() || '',
            memo: `SFK ${securityMode} operation`,
          },
          submission: {
            max_fee_hbar: 2,
            submit_key: '',
            require_consensus: true,
          },
        });

        if (witnessResponse.success) {
          op.witnessing = {
            status: 'confirmed',
            hedera_tx_id: witnessResponse.transaction.id,
            consensus_timestamp: witnessResponse.transaction.consensus_timestamp,
          };
          op.progress = 80;
        } else {
          op.witnessing = { status: 'failed' };
          op.progress = 75;
          console.warn(`[sfk-ops] Witnessing failed for ${operationId}: tx status ${witnessResponse.transaction.status}`);
        }
      } catch (error) {
        op.witnessing = { status: 'failed' };
        op.progress = 75;
        console.error(`[sfk-ops] Witnessing error for ${operationId}: ${(error as Error).message}`);
      }
    } else {
      op.witnessing = { status: 'pending' };
      op.progress = 80;
    }

    if (this.isAborted(op)) return;

    op.settlement = { status: 'pending', estimated_start: new Date(Date.now() + 5000).toISOString() };

    op.phase = 'finalization';
    op.status = 'completed';
    op.progress = 100;
    op.completed_at = Date.now();

    const elapsedMs = op.completed_at - op.queued_at;
    const witnessInfo = op.witnessing.hedera_tx_id
      ? `, witnessed: ${op.witnessing.hedera_tx_id}`
      : '';
    console.log(
      `[sfk-ops] Operation ${operationId} completed in ${elapsedMs}ms ` +
      `(mode: ${securityMode}${witnessInfo})`
    );
  }

  private async performTernaryProcessing(op: OperationState): Promise<string> {
    const payload = JSON.stringify({
      operation_id: op.id,
      type: op.request.operation.type,
      security_mode: op.request.operation.parameters.security_mode,
      batch_size: op.request.operation.parameters.batch_size,
      torsion_dimensions: op.request.operation.parameters.torsion_dimensions,
      phase_offset: op.request.operation.parameters.phase_offset,
      batch_ref: op.request.metadata.salvi_batch_ref,
      timestamp: op.started_at,
    });

    const hash = crypto.createHash('sha256').update(payload).digest('hex');

    const processingMs = Math.min(
      op.request.operation.parameters.batch_size * 2,
      50,
    );
    await new Promise(resolve => setTimeout(resolve, processingMs));

    return hash;
  }

  private evictExpired(): void {
    const cutoff = Date.now() - this.config.operationRetentionMs;
    let evicted = 0;
    for (const [id, op] of this.operations) {
      if (op.completed_at && op.completed_at < cutoff) {
        this.operations.delete(id);
        evicted++;
      }
      if (op.status === 'cancelled' && op.queued_at < cutoff) {
        this.operations.delete(id);
        evicted++;
      }
    }
    if (evicted > 0) {
      console.log(`[sfk-ops] Evicted ${evicted} expired operations`);
    }
  }

  close(): void {
    this.shutdownRequested = true;
    clearInterval(this.evictionTimer);
  }
}
