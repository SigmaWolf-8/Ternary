/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import crypto from 'crypto';

vi.mock('stripe', () => ({
  default: vi.fn(() => ({
    webhooks: { constructEvent: vi.fn() },
  })),
}));

vi.mock('../../services/payment-listener/config/webhook-config', () => ({
  webhookConfig: {
    stripeWebhookSecret: 'whsec_test_secret_key_for_testing_12345',
    interacWebhookSecret: 'interac_test_secret_key_abcdef123456',
    cryptoWebhookSecret: 'crypto_test_secret_key_deadbeef7890',
    signatureToleranceSeconds: 300,
    rateLimitWindowMs: 60000,
    rateLimitMax: 100,
    allowedOrigins: ['*'],
    idempotencyKeyTtlSeconds: 86400,
    maxPayloadSizeBytes: 1048576,
    webhookTimeoutMs: 30000,
    retryAttempts: 3,
    retryDelayMs: 1000,
  },
  RATE_LIMITS: {
    STRIPE: { requests_per_minute: 100, burst: 20 },
    INTERAC: { requests_per_minute: 50, burst: 10 },
    CRYPTO: { requests_per_minute: 200, burst: 40 },
  },
  WEBHOOK_ENDPOINTS: {
    STRIPE: '/api/v1/webhooks/stripe',
    INTERAC: '/api/v1/webhooks/interac',
    CRYPTO: '/api/v1/webhooks/crypto',
  },
  RESPONSE_CODES: {
    ACCEPTED: 202,
    BAD_REQUEST: 400,
    UNAUTHORIZED: 401,
    TOO_MANY_REQUESTS: 429,
    INTERNAL_ERROR: 500,
  },
}));

const TEST_STRIPE_SECRET = 'whsec_test_secret_key_for_testing_12345';
const TEST_INTERAC_SECRET = 'interac_test_secret_key_abcdef123456';
const TEST_CRYPTO_SECRET = 'crypto_test_secret_key_deadbeef7890';

import {
  validateStripeSignature,
  validateInteracSignature,
  validateCryptoSignature,
  validateWebhookSignature,
} from '../../services/payment-listener/src/validation/signature-validator';

import {
  generateIdempotencyKey,
  checkIdempotency,
  recordProcessedWebhook,
  getIdempotencyStats,
  shutdownIdempotencyChecker,
} from '../../services/payment-listener/src/validation/idempotency-checker';

import type {
  StripeWebhookEvent,
  InteracWebhookEvent,
  CryptoWebhookEvent,
  WebhookRequest,
  WebhookProcessingResult,
  PaymentStatusResponse,
  WebhookValidationResult,
  QueuedPayment,
  PaymentGateway,
} from '../../services/payment-listener/src/models/webhook-models';

function createStripeSignature(payload: string, secret: string, timestamp?: number): string {
  const ts = timestamp || Math.floor(Date.now() / 1000);
  const signedPayload = `${ts}.${payload}`;
  const sig = crypto.createHmac('sha256', secret).update(signedPayload, 'utf8').digest('hex');
  return `t=${ts},v1=${sig}`;
}

function createHmacSha512(payload: string, secret: string): string {
  return crypto.createHmac('sha512', secret).update(payload, 'utf8').digest('hex');
}

describe('Payment Webhook Tests', () => {

  describe('Stripe Signature Validation', () => {
    const testPayload = JSON.stringify({
      id: 'evt_test_123',
      type: 'payment_intent.succeeded',
      created: 1700000000,
      livemode: false,
      data: { object: { id: 'pi_123', amount: 5000, currency: 'usd', status: 'succeeded', metadata: {} } },
      api_version: '2023-10-16',
    });

    it('should validate a correct Stripe signature', () => {
      const signature = createStripeSignature(testPayload, TEST_STRIPE_SECRET);
      const result = validateStripeSignature(signature, testPayload);

      expect(result.valid).toBe(true);
      expect(result.timestamp).toBeDefined();
      expect(typeof result.timestamp).toBe('number');
    });

    it('should reject signature with missing timestamp', () => {
      const result = validateStripeSignature('v1=abcdef123456', testPayload);

      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('INVALID_SIGNATURE_FORMAT');
    });

    it('should reject signature with missing v1 component', () => {
      const ts = Math.floor(Date.now() / 1000);
      const result = validateStripeSignature(`t=${ts}`, testPayload);

      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('INVALID_SIGNATURE_FORMAT');
    });

    it('should reject expired timestamp', () => {
      const oldTimestamp = Math.floor(Date.now() / 1000) - 600;
      const signature = createStripeSignature(testPayload, TEST_STRIPE_SECRET, oldTimestamp);
      const result = validateStripeSignature(signature, testPayload);

      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('TIMESTAMP_EXPIRED');
    });

    it('should reject invalid signature value', () => {
      const ts = Math.floor(Date.now() / 1000);
      const fakeHex = 'a'.repeat(64);
      const signature = `t=${ts},v1=${fakeHex}`;
      const result = validateStripeSignature(signature, testPayload);

      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('INVALID_SIGNATURE');
    });

    it('should reject when payload is tampered', () => {
      const signature = createStripeSignature(testPayload, TEST_STRIPE_SECRET);
      const tamperedPayload = testPayload + 'tampered';
      const result = validateStripeSignature(signature, tamperedPayload);

      expect(result.valid).toBe(false);
    });

    it('should reject when secret is wrong', () => {
      const signature = createStripeSignature(testPayload, 'wrong_secret');
      const result = validateStripeSignature(signature, testPayload);

      expect(result.valid).toBe(false);
    });
  });

  describe('Stripe Signature - Missing Secret', () => {
    it('should return MISSING_SECRET when stripe secret is empty', async () => {
      const { webhookConfig } = await import('../../services/payment-listener/config/webhook-config');
      const originalSecret = webhookConfig.stripeWebhookSecret;
      webhookConfig.stripeWebhookSecret = '';

      const result = validateStripeSignature('t=123,v1=abc', 'payload');
      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('MISSING_SECRET');

      webhookConfig.stripeWebhookSecret = originalSecret;
    });
  });

  describe('Interac Signature Validation', () => {
    const testPayload = JSON.stringify({
      transaction_id: 'txn_interac_001',
      event_type: 'DEPOSIT_RECEIVED',
      timestamp: new Date().toISOString(),
      payload: { amount: '150.00', currency: 'CAD', sender_email: 'test@example.ca', reference_number: 'REF001' },
    });

    it('should validate a correct Interac signature', () => {
      const signature = createHmacSha512(testPayload, TEST_INTERAC_SECRET);
      const result = validateInteracSignature(signature, testPayload);

      expect(result.valid).toBe(true);
      expect(result.timestamp).toBeDefined();
    });

    it('should reject invalid Interac signature', () => {
      const fakeSignature = 'a'.repeat(128);
      const result = validateInteracSignature(fakeSignature, testPayload);

      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('INVALID_SIGNATURE');
    });

    it('should reject tampered Interac payload', () => {
      const signature = createHmacSha512(testPayload, TEST_INTERAC_SECRET);
      const result = validateInteracSignature(signature, testPayload + 'tampered');

      expect(result.valid).toBe(false);
    });

    it('should reject wrong Interac secret', () => {
      const signature = createHmacSha512(testPayload, 'wrong_interac_secret');
      const result = validateInteracSignature(signature, testPayload);

      expect(result.valid).toBe(false);
    });

    it('should return MISSING_SECRET when interac secret is empty', async () => {
      const { webhookConfig } = await import('../../services/payment-listener/config/webhook-config');
      const originalSecret = webhookConfig.interacWebhookSecret;
      webhookConfig.interacWebhookSecret = '';

      const result = validateInteracSignature('somesig', 'payload');
      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('MISSING_SECRET');

      webhookConfig.interacWebhookSecret = originalSecret;
    });
  });

  describe('Crypto Signature Validation', () => {
    const testPayload = JSON.stringify({
      id: 'crypto_evt_001',
      type: 'invoice_paid',
      created_at: new Date().toISOString(),
      data: {
        invoice_id: 'inv_001',
        amount: '100.00',
        currency: 'USD',
        crypto_amount: '0.0025',
        crypto_currency: 'BTC',
        confirmations: 6,
        tx_hash: '0xabc123',
      },
    });

    it('should validate a correct crypto signature', () => {
      const signature = createHmacSha512(testPayload, TEST_CRYPTO_SECRET);
      const result = validateCryptoSignature(signature, testPayload);

      expect(result.valid).toBe(true);
      expect(result.timestamp).toBeDefined();
    });

    it('should reject invalid crypto signature', () => {
      const fakeSignature = 'b'.repeat(128);
      const result = validateCryptoSignature(fakeSignature, testPayload);

      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('INVALID_SIGNATURE');
    });

    it('should reject tampered crypto payload', () => {
      const signature = createHmacSha512(testPayload, TEST_CRYPTO_SECRET);
      const result = validateCryptoSignature(signature, testPayload + 'x');

      expect(result.valid).toBe(false);
    });

    it('should return MISSING_SECRET when crypto secret is empty', async () => {
      const { webhookConfig } = await import('../../services/payment-listener/config/webhook-config');
      const originalSecret = webhookConfig.cryptoWebhookSecret;
      webhookConfig.cryptoWebhookSecret = '';

      const result = validateCryptoSignature('somesig', 'payload');
      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('MISSING_SECRET');

      webhookConfig.cryptoWebhookSecret = originalSecret;
    });
  });

  describe('Gateway Router - validateWebhookSignature', () => {
    const payload = '{"test": true}';

    it('should route to stripe validator', () => {
      const sig = createStripeSignature(payload, TEST_STRIPE_SECRET);
      const result = validateWebhookSignature('stripe', sig, payload);
      expect(result.valid).toBe(true);
    });

    it('should route to interac validator', () => {
      const sig = createHmacSha512(payload, TEST_INTERAC_SECRET);
      const result = validateWebhookSignature('interac', sig, payload);
      expect(result.valid).toBe(true);
    });

    it('should route to crypto validator', () => {
      const sig = createHmacSha512(payload, TEST_CRYPTO_SECRET);
      const result = validateWebhookSignature('crypto', sig, payload);
      expect(result.valid).toBe(true);
    });

    it('should return UNKNOWN_GATEWAY for unsupported gateways', () => {
      const result = validateWebhookSignature('paypal' as PaymentGateway, 'sig', payload);
      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('UNKNOWN_GATEWAY');
      expect(result.errorMessage).toContain('paypal');
    });
  });

  describe('Idempotency Checker', () => {
    beforeEach(() => {
      shutdownIdempotencyChecker();
    });

    afterEach(() => {
      shutdownIdempotencyChecker();
    });

    it('should generate correct idempotency keys', () => {
      const key = generateIdempotencyKey('stripe', 'evt_123', 'payment_intent.succeeded');
      expect(key).toBe('stripe:evt_123:payment_intent.succeeded');
    });

    it('should generate different keys for different gateways', () => {
      const stripeKey = generateIdempotencyKey('stripe', 'id_1', 'type_1');
      const interacKey = generateIdempotencyKey('interac', 'id_1', 'type_1');
      expect(stripeKey).not.toBe(interacKey);
    });

    it('should return isDuplicate false for new keys', () => {
      const result = checkIdempotency('brand_new_key');
      expect(result.isDuplicate).toBe(false);
      expect(result.existingResult).toBeUndefined();
    });

    it('should detect duplicates after recording', () => {
      const key = 'stripe:evt_dup:payment_intent.succeeded';

      recordProcessedWebhook(key, 'pay_123', 'BATCH_001');

      const result = checkIdempotency(key);
      expect(result.isDuplicate).toBe(true);
      expect(result.existingResult).toBeDefined();
      expect(result.existingResult!.paymentId).toBe('pay_123');
      expect(result.existingResult!.salviBatchRef).toBe('BATCH_001');
      expect(result.existingResult!.status).toBe('processed');
    });

    it('should not detect duplicates for different keys', () => {
      const key1 = 'stripe:evt_1:type_a';
      const key2 = 'stripe:evt_2:type_b';

      recordProcessedWebhook(key1, 'pay_1', 'BATCH_A');

      const result = checkIdempotency(key2);
      expect(result.isDuplicate).toBe(false);
    });

    it('should track stats correctly', () => {
      const stats1 = getIdempotencyStats();
      expect(stats1.totalEntries).toBe(0);
      expect(stats1.oldestEntryAge).toBeNull();
      expect(stats1.newestEntryAge).toBeNull();

      recordProcessedWebhook('key_stat_1', 'pay_s1', 'BATCH_S1');
      recordProcessedWebhook('key_stat_2', 'pay_s2', 'BATCH_S2');

      const stats2 = getIdempotencyStats();
      expect(stats2.totalEntries).toBe(2);
      expect(stats2.oldestEntryAge).toBeGreaterThanOrEqual(0);
      expect(stats2.newestEntryAge).toBeGreaterThanOrEqual(0);
      expect(stats2.oldestEntryAge!).toBeGreaterThanOrEqual(stats2.newestEntryAge!);
    });

    it('should clear all entries on shutdown', () => {
      recordProcessedWebhook('key_clear_1', 'pay_c1', 'BATCH_C1');
      recordProcessedWebhook('key_clear_2', 'pay_c2', 'BATCH_C2');

      shutdownIdempotencyChecker();

      const result = checkIdempotency('key_clear_1');
      expect(result.isDuplicate).toBe(false);

      const stats = getIdempotencyStats();
      expect(stats.totalEntries).toBe(0);
    });
  });

  describe('Webhook Model Validation', () => {
    it('StripeWebhookEvent should have correct structure', () => {
      const event: StripeWebhookEvent = {
        id: 'evt_test_001',
        type: 'payment_intent.succeeded',
        created: 1700000000,
        livemode: false,
        data: {
          object: {
            id: 'pi_test_001',
            amount: 5000,
            currency: 'usd',
            status: 'succeeded',
            metadata: {
              salvi_batch_ref: 'BATCH_001',
              kernel_op_id: 'OP_001',
              target_security_mode: 'CNSA2',
              customer_id: 'cust_001',
            },
            payment_method: 'pm_card_visa',
            receipt_url: 'https://receipt.stripe.com/test',
          },
        },
        api_version: '2023-10-16',
      };

      expect(event.id).toBe('evt_test_001');
      expect(event.data.object.amount).toBe(5000);
      expect(event.data.object.metadata.salvi_batch_ref).toBe('BATCH_001');
    });

    it('InteracWebhookEvent should have correct structure', () => {
      const event: InteracWebhookEvent = {
        transaction_id: 'txn_interac_001',
        event_type: 'DEPOSIT_COMPLETED',
        timestamp: '2026-02-14T12:00:00Z',
        payload: {
          amount: '250.00',
          currency: 'CAD',
          sender_email: 'sender@example.ca',
          reference_number: 'REF_INTERAC_001',
          memo: 'Salvi payment',
          metadata: {
            salvi_batch_ref: 'BATCH_CAD_001',
            kernel_op_id: 'OP_CAD_001',
          },
        },
      };

      expect(event.transaction_id).toBe('txn_interac_001');
      expect(event.event_type).toBe('DEPOSIT_COMPLETED');
      expect(event.payload.currency).toBe('CAD');
    });

    it('InteracWebhookEvent should support all event types', () => {
      const received: InteracWebhookEvent['event_type'] = 'DEPOSIT_RECEIVED';
      const completed: InteracWebhookEvent['event_type'] = 'DEPOSIT_COMPLETED';
      const failed: InteracWebhookEvent['event_type'] = 'DEPOSIT_FAILED';

      expect(received).toBe('DEPOSIT_RECEIVED');
      expect(completed).toBe('DEPOSIT_COMPLETED');
      expect(failed).toBe('DEPOSIT_FAILED');
    });

    it('CryptoWebhookEvent should have correct structure', () => {
      const event: CryptoWebhookEvent = {
        id: 'crypto_evt_001',
        type: 'payment_confirmed',
        created_at: '2026-02-14T12:00:00Z',
        data: {
          invoice_id: 'inv_001',
          amount: '100.00',
          currency: 'USD',
          crypto_amount: '0.0025',
          crypto_currency: 'BTC',
          confirmations: 6,
          tx_hash: '0xdeadbeef',
          metadata: {
            salvi_batch_ref: 'BATCH_CRYPTO_001',
            kernel_op_id: 'OP_CRYPTO_001',
          },
        },
      };

      expect(event.id).toBe('crypto_evt_001');
      expect(event.type).toBe('payment_confirmed');
      expect(event.data.confirmations).toBe(6);
      expect(event.data.tx_hash).toBe('0xdeadbeef');
    });

    it('CryptoWebhookEvent should support all event types', () => {
      const paid: CryptoWebhookEvent['type'] = 'invoice_paid';
      const confirmed: CryptoWebhookEvent['type'] = 'payment_confirmed';
      const failed: CryptoWebhookEvent['type'] = 'payment_failed';

      expect(paid).toBe('invoice_paid');
      expect(confirmed).toBe('payment_confirmed');
      expect(failed).toBe('payment_failed');
    });

    it('WebhookProcessingResult should serialize correctly', () => {
      const result: WebhookProcessingResult = {
        accepted: true,
        paymentId: 'pay_test_001',
        salviBatchRef: 'BATCH_TEST',
        queuedAt: '2026-02-14T12:00:00Z',
        estimatedProcessingTimeMs: 5000,
        idempotencyKey: 'stripe:evt_1:payment_intent.succeeded',
      };

      const serialized = JSON.stringify(result);
      const deserialized = JSON.parse(serialized);

      expect(deserialized.accepted).toBe(true);
      expect(deserialized.paymentId).toBe('pay_test_001');
      expect(deserialized.salviBatchRef).toBe('BATCH_TEST');
      expect(deserialized.estimatedProcessingTimeMs).toBe(5000);
    });

    it('PaymentStatusResponse should have all required fields', () => {
      const status: PaymentStatusResponse = {
        paymentId: 'pay_001',
        status: 'witnessed',
        amount: 5000,
        currency: 'usd',
        gateway: 'stripe',
        salviReferences: {
          batchRef: 'BATCH_001',
          kernelOpId: 'OP_001',
          witnessTxId: 'witness-tx-123',
        },
        timingMetadata: {
          receivedAt: '2026-02-14T12:00:00Z',
          processedAt: '2026-02-14T12:00:01Z',
          totalLatencyNs: 1000000000,
        },
        settledAt: '2026-02-14T12:00:02Z',
      };

      expect(status.status).toBe('witnessed');
      expect(status.salviReferences.witnessTxId).toBe('witness-tx-123');
      expect(status.timingMetadata.totalLatencyNs).toBe(1000000000);
    });

    it('PaymentStatusResponse should support all status values', () => {
      const statuses: PaymentStatusResponse['status'][] = [
        'pending', 'processing', 'witnessed', 'settled', 'failed',
      ];
      expect(statuses).toHaveLength(5);
    });

    it('PaymentGateway type should support all gateways', () => {
      const gateways: PaymentGateway[] = ['stripe', 'interac', 'crypto'];
      expect(gateways).toHaveLength(3);
    });

    it('WebhookValidationResult should represent valid results', () => {
      const valid: WebhookValidationResult = { valid: true, timestamp: 1700000000 };
      expect(valid.valid).toBe(true);
      expect(valid.errorCode).toBeUndefined();
    });

    it('WebhookValidationResult should represent invalid results', () => {
      const invalid: WebhookValidationResult = {
        valid: false,
        errorCode: 'INVALID_SIGNATURE',
        errorMessage: 'Signature verification failed',
      };
      expect(invalid.valid).toBe(false);
      expect(invalid.errorCode).toBe('INVALID_SIGNATURE');
    });

    it('QueuedPayment should have correct structure', () => {
      const queued: QueuedPayment = {
        id: 'queue_001',
        gateway: 'stripe',
        event: {
          id: 'evt_001',
          type: 'payment_intent.succeeded',
          created: 1700000000,
          livemode: false,
          data: {
            object: {
              id: 'pi_001',
              amount: 5000,
              currency: 'usd',
              status: 'succeeded',
              metadata: {},
            },
          },
          api_version: '2023-10-16',
        },
        receivedAt: '2026-02-14T12:00:00Z',
        salviBatchRef: 'BATCH_Q1',
        kernelOpId: 'OP_Q1',
        retryCount: 0,
        maxRetries: 3,
      };

      expect(queued.id).toBe('queue_001');
      expect(queued.retryCount).toBe(0);
      expect(queued.maxRetries).toBe(3);
    });

    it('WebhookRequest should tie together gateway, signature, and event', () => {
      const request: WebhookRequest = {
        gateway: 'crypto',
        signature: 'hex_signature_here',
        timestamp: Date.now(),
        rawPayload: '{"id":"test"}',
        parsedEvent: {
          id: 'crypto_001',
          type: 'invoice_paid',
          created_at: '2026-02-14T12:00:00Z',
          data: {
            invoice_id: 'inv_001',
            amount: '50.00',
            currency: 'USD',
            crypto_amount: '0.001',
            crypto_currency: 'ETH',
            confirmations: 12,
          },
        },
        idempotencyKey: 'crypto:crypto_001:invoice_paid',
      };

      expect(request.gateway).toBe('crypto');
      expect(request.idempotencyKey).toBe('crypto:crypto_001:invoice_paid');
    });
  });
});
