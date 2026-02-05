/**
 * Payment Listener API
 * Webhook Ingestion and Payment Status Endpoints
 * 
 * @author Capomastro Holdings Ltd.
 * @license Proprietary - All Rights Reserved
 */

import type { PaymentGateway, UnifiedOperationMetadata } from './unified-metadata-schema';

/**
 * Salvi Reference Links
 * Cross-references between payment and blockchain
 */
export interface SalviReferences {
  batch_ref: string;
  kernel_op_id: string;
  witness_tx_id: string;
}

/**
 * Timing Metadata for Payment Processing
 */
export interface PaymentTimingMetadata {
  received_at: string;
  processed_at: string;
  total_latency_ns: number;
}

/**
 * Payment Status Response
 * GET /api/v1/payments/{payment_id}
 */
export interface PaymentStatusResponse {
  payment_id: string;
  status: 'pending' | 'processing' | 'settled' | 'failed' | 'refunded';
  amount: number;
  currency: string;
  settled_at?: string;
  gateway: PaymentGateway;
  salvi_references: SalviReferences;
  timing_metadata: PaymentTimingMetadata;
}

/**
 * Stripe Webhook Event Structure
 */
export interface StripeWebhookEvent {
  id: string;
  type: string;
  created: number;
  data: {
    object: {
      id: string;
      amount: number;
      currency: string;
      customer: string;
      metadata: {
        salvi_batch_ref?: string;
        kernel_op_id?: string;
        target_security_mode?: string;
      };
    };
  };
}

/**
 * Interac E-Transfer Webhook Event
 */
export interface InteracWebhookEvent {
  event_id: string;
  event_type: string;
  timestamp: string;
  payment: {
    id: string;
    amount: number;
    currency: string;
    remittance_info: string;
    client_reference: string;
  };
}

/**
 * Crypto Gateway Webhook Event
 */
export interface CryptoWebhookEvent {
  id: string;
  type: 'invoice_paid' | 'payment_confirmed';
  created_at: string;
  data: {
    invoice_id: string;
    amount: string;
    currency: string;
    crypto_amount: string;
    crypto_currency: string;
    metadata?: Record<string, string>;
  };
}

/**
 * Unified Webhook Request
 * Normalized structure for all payment gateways
 */
export interface WebhookRequest {
  gateway: PaymentGateway;
  signature: string;
  timestamp: number;
  raw_payload: string;
  parsed_event: StripeWebhookEvent | InteracWebhookEvent | CryptoWebhookEvent;
}

/**
 * Webhook Processing Result
 */
export interface WebhookProcessingResult {
  accepted: boolean;
  payment_id: string;
  salvi_batch_ref: string;
  queued_at: string;
  estimated_processing_time_ms: number;
}

/**
 * Webhook Response Codes
 */
export const WEBHOOK_RESPONSE_CODES = {
  ACCEPTED: { status: 202, message: 'Webhook received and queued for processing' },
  BAD_REQUEST: { status: 400, message: 'Invalid webhook signature or format' },
  TOO_MANY_REQUESTS: { status: 429, message: 'Rate limit exceeded' },
  INTERNAL_ERROR: { status: 500, message: 'Internal server error' }
} as const;

/**
 * Payment Listener Service Interface
 */
export interface IPaymentListenerService {
  validateWebhookSignature(gateway: PaymentGateway, signature: string, payload: string): Promise<boolean>;
  processWebhook(request: WebhookRequest): Promise<WebhookProcessingResult>;
  getPaymentStatus(paymentId: string): Promise<PaymentStatusResponse | null>;
  listPayments(page: number, limit: number): Promise<PaymentStatusResponse[]>;
}

/**
 * Validate Stripe webhook signature using HMAC-SHA256
 * Implements Stripe's signature verification algorithm
 * 
 * @param signature - The Stripe-Signature header value
 * @param payload - The raw request body as string
 * @param secret - The webhook signing secret (whsec_xxx)
 * @param tolerance - Maximum age of the webhook in seconds (default 300)
 * @returns true if signature is valid, false otherwise
 */
export function validateStripeSignature(
  signature: string,
  payload: string,
  secret: string,
  tolerance: number = 300
): boolean {
  const crypto = require('crypto');
  
  const signatureParts = signature.split(',');
  const timestamp = signatureParts.find(p => p.startsWith('t='))?.split('=')[1];
  const v1Signature = signatureParts.find(p => p.startsWith('v1='))?.split('=')[1];
  
  if (!timestamp || !v1Signature) {
    return false;
  }
  
  const timestampNum = parseInt(timestamp, 10);
  const currentTime = Math.floor(Date.now() / 1000);
  
  if (Math.abs(currentTime - timestampNum) > tolerance) {
    return false;
  }
  
  const signedPayload = `${timestamp}.${payload}`;
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(signedPayload, 'utf8')
    .digest('hex');
  
  return crypto.timingSafeEqual(
    Buffer.from(v1Signature, 'hex'),
    Buffer.from(expectedSignature, 'hex')
  );
}

/**
 * Validate Interac E-Transfer webhook signature using HMAC-SHA256
 * 
 * @param signature - The X-JPMC-Signature header value
 * @param payload - The raw request body as string
 * @param secret - The webhook signing secret
 * @returns true if signature is valid, false otherwise
 */
export function validateInteracSignature(
  signature: string,
  payload: string,
  secret: string
): boolean {
  const crypto = require('crypto');
  
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(payload, 'utf8')
    .digest('hex');
  
  try {
    return crypto.timingSafeEqual(
      Buffer.from(signature, 'hex'),
      Buffer.from(expectedSignature, 'hex')
    );
  } catch {
    return false;
  }
}

/**
 * Validate Crypto Gateway webhook signature using HMAC-SHA512
 * 
 * @param signature - The X-CC-Webhook-Signature header value
 * @param payload - The raw request body as string
 * @param secret - The webhook signing secret
 * @returns true if signature is valid, false otherwise
 */
export function validateCryptoGatewaySignature(
  signature: string,
  payload: string,
  secret: string
): boolean {
  const crypto = require('crypto');
  
  const expectedSignature = crypto
    .createHmac('sha512', secret)
    .update(payload, 'utf8')
    .digest('hex');
  
  try {
    return crypto.timingSafeEqual(
      Buffer.from(signature, 'hex'),
      Buffer.from(expectedSignature, 'hex')
    );
  } catch {
    return false;
  }
}

/**
 * Extract Salvi metadata from Stripe event
 */
export function extractStripeMetadata(event: StripeWebhookEvent): {
  batchRef: string;
  kernelOpId: string;
  securityMode: string;
} {
  const metadata = event.data.object.metadata;
  return {
    batchRef: metadata.salvi_batch_ref || `BATCH_${Date.now()}`,
    kernelOpId: metadata.kernel_op_id || `OP_${Date.now()}`,
    securityMode: metadata.target_security_mode || 'phi_plus'
  };
}

/**
 * Create payment status response from operation metadata
 */
export function createPaymentStatusResponse(
  paymentId: string,
  metadata: Partial<UnifiedOperationMetadata>,
  status: PaymentStatusResponse['status']
): PaymentStatusResponse {
  return {
    payment_id: paymentId,
    status,
    amount: metadata.payment_context?.amount || 0,
    currency: metadata.payment_context?.currency || 'CAD',
    settled_at: metadata.payment_context?.settled_at,
    gateway: metadata.payment_context?.gateway || 'stripe',
    salvi_references: {
      batch_ref: metadata.salvi_batch_ref || '',
      kernel_op_id: metadata.kernel_op_id || '',
      witness_tx_id: metadata.blockchain_refs?.hedera?.transaction_id || ''
    },
    timing_metadata: {
      received_at: metadata.timing_context?.batch_start_ts || new Date().toISOString(),
      processed_at: metadata.timing_context?.batch_end_ts || new Date().toISOString(),
      total_latency_ns: metadata.timing_context?.duration_ns || 0
    }
  };
}
