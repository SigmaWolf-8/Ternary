export type PaymentGateway = 'stripe' | 'interac' | 'crypto';

export interface StripeWebhookEvent {
  id: string;
  type: string;
  created: number;
  livemode: boolean;
  data: {
    object: {
      id: string;
      amount: number;
      currency: string;
      status: string;
      metadata: {
        salvi_batch_ref?: string;
        kernel_op_id?: string;
        target_security_mode?: string;
        customer_id?: string;
      };
      payment_method?: string;
      receipt_url?: string;
    };
  };
  api_version: string;
}

export interface InteracWebhookEvent {
  transaction_id: string;
  event_type: 'DEPOSIT_RECEIVED' | 'DEPOSIT_COMPLETED' | 'DEPOSIT_FAILED';
  timestamp: string;
  payload: {
    amount: string;
    currency: 'CAD';
    sender_email: string;
    reference_number: string;
    memo?: string;
    metadata?: {
      salvi_batch_ref?: string;
      kernel_op_id?: string;
    };
  };
}

export interface CryptoWebhookEvent {
  id: string;
  type: 'invoice_paid' | 'payment_confirmed' | 'payment_failed';
  created_at: string;
  data: {
    invoice_id: string;
    amount: string;
    currency: string;
    crypto_amount: string;
    crypto_currency: string;
    confirmations: number;
    tx_hash?: string;
    metadata?: {
      salvi_batch_ref?: string;
      kernel_op_id?: string;
    };
  };
}

export interface WebhookRequest {
  gateway: PaymentGateway;
  signature: string;
  timestamp: number;
  rawPayload: string;
  parsedEvent: StripeWebhookEvent | InteracWebhookEvent | CryptoWebhookEvent;
  idempotencyKey: string;
}

export interface WebhookProcessingResult {
  accepted: boolean;
  paymentId: string;
  salviBatchRef: string;
  queuedAt: string;
  estimatedProcessingTimeMs: number;
  idempotencyKey: string;
}

export interface PaymentStatusResponse {
  paymentId: string;
  status: 'pending' | 'processing' | 'witnessed' | 'settled' | 'failed';
  amount: number;
  currency: string;
  settledAt?: string;
  gateway: PaymentGateway;
  salviReferences: {
    batchRef: string;
    kernelOpId: string;
    witnessTxId?: string;
  };
  timingMetadata: {
    receivedAt: string;
    processedAt?: string;
    totalLatencyNs?: number;
  };
}

export interface WebhookValidationResult {
  valid: boolean;
  errorCode?: string;
  errorMessage?: string;
  timestamp?: number;
}

export interface QueuedPayment {
  id: string;
  gateway: PaymentGateway;
  event: StripeWebhookEvent | InteracWebhookEvent | CryptoWebhookEvent;
  receivedAt: string;
  salviBatchRef: string;
  kernelOpId: string;
  retryCount: number;
  maxRetries: number;
}
