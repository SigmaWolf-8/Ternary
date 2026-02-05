/**
 * Unified Ternary-Torsion Payment & Witnessing Architecture
 * Complete Data Schema Specifications v1.0
 * 
 * @author Capomastro Holdings Ltd.
 * @license Proprietary - All Rights Reserved
 */

export type SecurityMode = 'phi_plus' | 'phi' | 'one' | 'zero';
export type PaymentGateway = 'stripe' | 'interac_etransfer' | 'crypto_gateway';
export type TorsionDimensions = 7 | 10 | 13;

/**
 * Ternary Processing Context
 * Defines parameters for ternary-torsion operations
 */
export interface TernaryContext {
  security_mode: SecurityMode;
  phase_offset: number;
  torsion_dimensions: TorsionDimensions;
  batch_size: number;
  operation_count: number;
  femtosecond_sync_required: boolean;
}

/**
 * Payment Context
 * Captures payment gateway information for cross-reference
 */
export interface PaymentContext {
  gateway: PaymentGateway;
  payment_id: string;
  amount: number;
  currency: string;
  settled_at: string;
}

/**
 * Clock Source Information
 * Traceability to atomic time standards
 */
export interface ClockSource {
  type: string;
  id: string;
  traceability: string;
}

/**
 * Timing Context with Femtosecond Precision
 * Captures temporal metadata for regulatory compliance
 */
export interface TimingContext {
  batch_start_ts: string;
  batch_end_ts: string;
  duration_ns: number;
  femtosecond_accuracy: number;
  clock_source: ClockSource;
}

/**
 * Hedera Blockchain Reference
 * Consensus Service witnessing proof
 */
export interface HederaReference {
  topic_id: string;
  sequence_number: number;
  consensus_timestamp: string;
  transaction_id: string;
}

/**
 * XRPL Blockchain Reference
 * Payment settlement proof on XRP Ledger
 */
export interface XRPLReference {
  transaction_hash: string;
  ledger_index: number;
  validated: boolean;
}

/**
 * Algorand Blockchain Reference
 * Smart contract execution proof
 */
export interface AlgorandReference {
  transaction_id: string;
  confirmed_round: number;
  application_id: number;
}

/**
 * Multi-Chain Blockchain References
 * Populated post-execution for audit trail
 */
export interface BlockchainRefs {
  hedera?: HederaReference;
  xrpl?: XRPLReference;
  algorand?: AlgorandReference;
}

/**
 * Operation Result Metrics
 * Performance and success statistics
 */
export interface OperationMetrics {
  operations_per_second: number;
  average_latency_ns: number;
  success_rate: number;
  energy_efficiency_gain: number;
}

/**
 * Operation Results
 * Final outcome of SFK operation
 */
export interface OperationResults {
  success: boolean;
  error?: string;
  metrics: OperationMetrics;
}

/**
 * Unified Operation Metadata Schema
 * Complete metadata for all SFK operations - the core data structure
 * that flows through payment processing, witnessing, and settlement
 */
export interface UnifiedOperationMetadata {
  salvi_batch_ref: string;
  kernel_op_id: string;
  correlation_id: string;
  ternary_context: TernaryContext;
  payment_context: PaymentContext;
  timing_context: TimingContext;
  blockchain_refs?: BlockchainRefs;
  results?: OperationResults;
}

/**
 * Payment Webhook Validation Configuration
 */
export interface WebhookValidation {
  signature_header: string;
  algorithm: string;
  tolerance_seconds: number;
}

/**
 * Payment Event Type Mapping
 */
export interface EventTypeMapping {
  payment_succeeded?: string;
  payment_failed?: string;
  charge_refunded?: string;
  payment_completed?: string;
  invoice_paid?: string;
  payment_confirmed?: string;
}

/**
 * Metadata Field Mapping for different gateways
 */
export interface MetadataMapping {
  salvi_batch_ref: string;
  kernel_op_id?: string;
  client_reference?: string;
}

/**
 * Payment Gateway Schema Configuration
 */
export interface GatewaySchema {
  validation: WebhookValidation;
  event_types: EventTypeMapping;
  metadata_mapping?: MetadataMapping;
}

/**
 * Payment Webhook Schema Registry
 * Configuration for all supported payment gateways
 */
export const WEBHOOK_SCHEMA_REGISTRY: Record<PaymentGateway, GatewaySchema> = {
  stripe: {
    validation: {
      signature_header: 'Stripe-Signature',
      algorithm: 'sha256',
      tolerance_seconds: 300
    },
    event_types: {
      payment_succeeded: 'payment_intent.succeeded',
      payment_failed: 'payment_intent.payment_failed',
      charge_refunded: 'charge.refunded'
    },
    metadata_mapping: {
      salvi_batch_ref: 'metadata.salvi_batch_ref',
      kernel_op_id: 'metadata.kernel_op_id'
    }
  },
  interac_etransfer: {
    validation: {
      signature_header: 'X-JPMC-Signature',
      algorithm: 'hmac-sha256',
      tolerance_seconds: 300
    },
    event_types: {
      payment_completed: 'payment.completed',
      payment_failed: 'payment.failed'
    },
    metadata_mapping: {
      salvi_batch_ref: 'remittance_info',
      client_reference: 'client_reference'
    }
  },
  crypto_gateway: {
    validation: {
      signature_header: 'X-CC-Webhook-Signature',
      algorithm: 'hmac-sha512',
      tolerance_seconds: 600
    },
    event_types: {
      invoice_paid: 'invoice_paid',
      payment_confirmed: 'payment_confirmed'
    }
  }
};

/**
 * Rate Limiting Configuration
 */
export const RATE_LIMITS = {
  payment_webhooks: {
    stripe: '1000/minute',
    interac_etransfer: '500/minute',
    crypto_gateway: '200/minute'
  },
  internal_apis: {
    payment_confirmation: '10000/minute',
    operation_status: '5000/minute',
    witness_submission: '1000/minute'
  },
  blockchain_apis: {
    hedera_hcs: '100/second',
    xrpl_payments: '50/second',
    algorand_calls: '100/second'
  }
} as const;

/**
 * Burst Limit Configuration
 */
export const BURST_LIMITS = {
  high_priority: '2x normal rate',
  emergency: '10x normal rate for 10 seconds'
} as const;

/**
 * Create a new Unified Operation Metadata object
 */
export function createOperationMetadata(
  batchRef: string,
  kernelOpId: string,
  correlationId: string,
  ternaryContext: TernaryContext,
  paymentContext: PaymentContext,
  timingContext: TimingContext
): UnifiedOperationMetadata {
  return {
    salvi_batch_ref: batchRef,
    kernel_op_id: kernelOpId,
    correlation_id: correlationId,
    ternary_context: ternaryContext,
    payment_context: paymentContext,
    timing_context: timingContext
  };
}

/**
 * Validate operation metadata completeness
 */
export function validateOperationMetadata(metadata: UnifiedOperationMetadata): boolean {
  return !!(
    metadata.salvi_batch_ref &&
    metadata.kernel_op_id &&
    metadata.correlation_id &&
    metadata.ternary_context &&
    metadata.payment_context &&
    metadata.timing_context
  );
}
