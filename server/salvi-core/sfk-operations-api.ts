/**
 * SFK Core Operations API
 * Salvi Framework Kernel Operation Management
 * 
 * @author Capomastro Holdings Ltd.
 * @license Proprietary - All Rights Reserved
 */

import type { SecurityMode, TorsionDimensions, PaymentGateway } from './unified-metadata-schema';

/**
 * Operation Type Enumeration
 */
export type OperationType = 
  | 'TERNARY_BATCH_PROCESSING'
  | 'PHASE_ENCRYPTION'
  | 'TORSION_ROUTING'
  | 'WITNESS_SUBMISSION'
  | 'SETTLEMENT_EXECUTION';

/**
 * Operation Status Enumeration
 */
export type OperationStatus = 
  | 'queued'
  | 'in_progress'
  | 'witnessing'
  | 'settling'
  | 'completed'
  | 'failed'
  | 'cancelled';

/**
 * Operation Phase Enumeration
 */
export type OperationPhase = 
  | 'initialization'
  | 'ternary_processing'
  | 'witnessing'
  | 'settlement'
  | 'finalization';

/**
 * Payment Trigger Information
 */
export interface PaymentTrigger {
  payment_id: string;
  gateway: PaymentGateway;
  settled_amount: number;
  settled_currency: string;
}

/**
 * Operation Parameters
 */
export interface OperationParameters {
  security_mode: SecurityMode;
  phase_offset: number;
  torsion_dimensions: TorsionDimensions;
  batch_size: number;
}

/**
 * Operation Timing Requirements
 */
export interface OperationTiming {
  requested_start_window: string;
  max_duration_ns: number;
  femtosecond_sync_required: boolean;
}

/**
 * Operation Definition
 */
export interface OperationDefinition {
  id: string;
  type: OperationType;
  trigger: PaymentTrigger;
  parameters: OperationParameters;
  timing: OperationTiming;
}

/**
 * Operation Request Metadata
 */
export interface OperationRequestMetadata {
  salvi_batch_ref: string;
  customer_id: string;
  correlation_id: string;
}

/**
 * SFK Operation Request
 * POST /api/sfk/v1/operations
 */
export interface SFKOperationRequest {
  operation: OperationDefinition;
  metadata: OperationRequestMetadata;
}

/**
 * Witness Action Configuration
 */
export interface WitnessAction {
  endpoint: string;
  method: 'POST';
  payload_template: 'merkle_root_batch' | 'single_hash' | 'aggregate';
}

/**
 * Settlement Action Configuration
 */
export interface SettleAction {
  endpoint: string;
  method: 'POST';
  token: 'TAT' | 'XRP' | 'ALGO';
}

/**
 * Scheduled Actions
 */
export interface ScheduledActions {
  witness: WitnessAction;
  settle: SettleAction;
}

/**
 * Timing Guarantees
 */
export interface TimingGuarantees {
  witness_completion_max_ns: number;
  settlement_completion_max_ns: number;
  femtosecond_accuracy: string;
}

/**
 * Operation Response Summary
 */
export interface OperationResponseSummary {
  id: string;
  status: OperationStatus;
  estimated_start: string;
  witnessing_required: boolean;
  settlement_required: boolean;
}

/**
 * SFK Operation Response
 * Response from POST /api/sfk/v1/operations
 */
export interface SFKOperationResponse {
  operation: OperationResponseSummary;
  actions: ScheduledActions;
  timing_guarantees: TimingGuarantees;
}

/**
 * Witnessing Component Status
 */
export interface WitnessingComponentStatus {
  status: 'pending' | 'submitted' | 'confirmed' | 'failed';
  hedera_tx_id?: string;
  consensus_timestamp?: string;
}

/**
 * Settlement Component Status
 */
export interface SettlementComponentStatus {
  status: 'pending' | 'submitted' | 'validated' | 'failed';
  estimated_start?: string;
  xrpl_tx_hash?: string;
}

/**
 * Ternary Processing Component Status
 */
export interface TernaryProcessingStatus {
  status: 'queued' | 'processing' | 'completed' | 'failed';
  estimated_duration_ns?: number;
  progress?: number;
}

/**
 * Operation Components Status
 */
export interface OperationComponents {
  witnessing: WitnessingComponentStatus;
  settlement: SettlementComponentStatus;
  ternary_processing: TernaryProcessingStatus;
}

/**
 * Operation Timing Metrics
 */
export interface OperationTimingMetrics {
  queued_at: string;
  started_at?: string;
  current_timestamp: string;
  elapsed_ns: number;
}

/**
 * Operation Status Details
 */
export interface OperationStatusDetails {
  id: string;
  status: OperationStatus;
  phase: OperationPhase;
  progress: number;
  current_step: string;
}

/**
 * SFK Operation Status Response
 * GET /api/sfk/v1/operations/{operation_id}
 */
export interface SFKOperationStatusResponse {
  operation: OperationStatusDetails;
  components: OperationComponents;
  timing_metrics: OperationTimingMetrics;
}

/**
 * Generate unique operation ID
 */
export function generateOperationId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 8);
  return `sfk_op_${timestamp}${random}`;
}

/**
 * Generate correlation ID
 */
export function generateCorrelationId(): string {
  return `corr_${Date.now().toString(36)}${Math.random().toString(36).substring(2, 6)}`;
}

/**
 * Calculate estimated start time based on queue depth
 */
export function calculateEstimatedStart(queueDepth: number, avgProcessingTimeMs: number = 100): Date {
  const estimatedWaitMs = queueDepth * avgProcessingTimeMs;
  return new Date(Date.now() + estimatedWaitMs);
}

/**
 * Default timing guarantees for different security modes
 */
export const TIMING_GUARANTEES: Record<SecurityMode, TimingGuarantees> = {
  phi_plus: {
    witness_completion_max_ns: 5000000000,
    settlement_completion_max_ns: 10000000000,
    femtosecond_accuracy: '10^-15 seconds'
  },
  phi: {
    witness_completion_max_ns: 3000000000,
    settlement_completion_max_ns: 8000000000,
    femtosecond_accuracy: '10^-15 seconds'
  },
  one: {
    witness_completion_max_ns: 2000000000,
    settlement_completion_max_ns: 6000000000,
    femtosecond_accuracy: '10^-12 seconds'
  },
  zero: {
    witness_completion_max_ns: 1000000000,
    settlement_completion_max_ns: 4000000000,
    femtosecond_accuracy: '10^-9 seconds'
  }
};

/**
 * Create SFK operation response
 */
export function createOperationResponse(
  operationId: string,
  securityMode: SecurityMode,
  estimatedStart: Date
): SFKOperationResponse {
  return {
    operation: {
      id: operationId,
      status: 'queued',
      estimated_start: estimatedStart.toISOString(),
      witnessing_required: true,
      settlement_required: true
    },
    actions: {
      witness: {
        endpoint: '/internal/api/hedera/v1/witness',
        method: 'POST',
        payload_template: 'merkle_root_batch'
      },
      settle: {
        endpoint: '/internal/api/xrpl/v1/payments',
        method: 'POST',
        token: 'TAT'
      }
    },
    timing_guarantees: TIMING_GUARANTEES[securityMode]
  };
}

/**
 * Create operation status response
 */
export function createOperationStatusResponse(
  operationId: string,
  status: OperationStatus,
  phase: OperationPhase,
  progress: number,
  components: OperationComponents,
  timingMetrics: OperationTimingMetrics
): SFKOperationStatusResponse {
  const stepMap: Record<OperationPhase, string> = {
    initialization: 'INITIALIZING_TERNARY_CONTEXT',
    ternary_processing: 'PROCESSING_TERNARY_OPERATIONS',
    witnessing: 'HEDERA_HCS_SUBMISSION',
    settlement: 'XRPL_PAYMENT_EXECUTION',
    finalization: 'COMPLETING_OPERATION'
  };

  return {
    operation: {
      id: operationId,
      status,
      phase,
      progress,
      current_step: stepMap[phase]
    },
    components,
    timing_metrics: timingMetrics
  };
}
