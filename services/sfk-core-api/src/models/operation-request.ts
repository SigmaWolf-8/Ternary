import { z } from 'zod';

export type SecurityMode = 'mode_zero' | 'mode_one' | 'phi_plus';
export type OperationType = 'payment_witness' | 'data_attest' | 'state_transition' | 'consensus_round';
export type OperationStatus = 'pending' | 'queued' | 'processing' | 'witnessed' | 'settled' | 'failed' | 'cancelled';
export type PaymentGateway = 'stripe' | 'interac' | 'crypto';

export const OperationRequestSchema = z.object({
  operationType: z.enum(['payment_witness', 'data_attest', 'state_transition', 'consensus_round']),
  securityMode: z.enum(['mode_zero', 'mode_one', 'phi_plus']).default('phi_plus'),
  priority: z.enum(['low', 'normal', 'high', 'critical']).default('normal'),
  paymentContext: z.object({
    gateway: z.enum(['stripe', 'interac', 'crypto']),
    paymentId: z.string(),
    amount: z.number().positive(),
    currency: z.string().length(3),
    metadata: z.record(z.string()).optional()
  }).optional(),
  ternaryContext: z.object({
    representation: z.enum(['A', 'B', 'C']).default('A'),
    dataHash: z.string().optional(),
    compressionEnabled: z.boolean().default(true)
  }).optional(),
  blockchainTargets: z.object({
    witnessToHedera: z.boolean().default(true),
    settleOnXrpl: z.boolean().default(false),
    recordOnAlgorand: z.boolean().default(false)
  }).optional(),
  callbackUrl: z.string().url().optional(),
  idempotencyKey: z.string().optional(),
  ttlSeconds: z.number().int().positive().max(86400).default(3600)
});

export type OperationRequest = z.infer<typeof OperationRequestSchema>;

export interface SFKOperation {
  id: string;
  batchRef: string;
  kernelOpId: string;
  operationType: OperationType;
  securityMode: SecurityMode;
  status: OperationStatus;
  priority: 'low' | 'normal' | 'high' | 'critical';
  createdAt: string;
  updatedAt: string;
  startedAt?: string;
  completedAt?: string;
  paymentContext?: {
    gateway: PaymentGateway;
    paymentId: string;
    amount: number;
    currency: string;
    metadata?: Record<string, string>;
  };
  ternaryContext?: {
    representation: 'A' | 'B' | 'C';
    dataHash?: string;
    compressionEnabled: boolean;
  };
  blockchainRefs?: {
    hedera?: {
      topicId: string;
      sequenceNumber: number;
      transactionId: string;
      consensusTimestamp: string;
    };
    xrpl?: {
      ledgerIndex: number;
      transactionHash: string;
      validated: boolean;
    };
    algorand?: {
      appId: number;
      round: number;
      txId: string;
    };
  };
  timingMetadata: {
    receivedAt: string;
    queuedAt?: string;
    processingStartedAt?: string;
    witnessedAt?: string;
    settledAt?: string;
    totalLatencyNs?: number;
  };
  errorInfo?: {
    code: string;
    message: string;
    retryable: boolean;
    retryCount: number;
    maxRetries: number;
  };
  callbackUrl?: string;
  idempotencyKey?: string;
  ttlSeconds: number;
}

export interface OperationListResponse {
  operations: SFKOperation[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
  filters: {
    status?: OperationStatus;
    operationType?: OperationType;
    gateway?: PaymentGateway;
  };
}
