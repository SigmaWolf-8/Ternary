import { Router, Request, Response } from 'express';
import { v4 as uuidv4 } from 'uuid';
import { validateStripeSignature } from '../validation/signature-validator';
import { checkIdempotency, recordProcessedWebhook, generateIdempotencyKey } from '../validation/idempotency-checker';
import { PaymentQueue } from '../queue/payment-queue';
import { StripeWebhookEvent, WebhookProcessingResult } from '../models/webhook-models';
import { RESPONSE_CODES } from '../../config/webhook-config';

export const stripeWebhookRouter = Router();

function extractSalviMetadata(event: StripeWebhookEvent): { salviBatchRef: string; kernelOpId: string } {
  const metadata = event.data?.object?.metadata || {};
  return {
    salviBatchRef: metadata.salvi_batch_ref || `BATCH_${Date.now()}_${uuidv4().slice(0, 8)}`,
    kernelOpId: metadata.kernel_op_id || `OP_${Date.now()}_${uuidv4().slice(0, 8)}`
  };
}

stripeWebhookRouter.post('/', async (req: Request, res: Response) => {
  const receivedAt = new Date().toISOString();
  const signature = req.headers['stripe-signature'] as string;
  
  if (!signature) {
    return res.status(RESPONSE_CODES.BAD_REQUEST).json({
      error: 'Missing Stripe-Signature header',
      code: 'MISSING_SIGNATURE'
    });
  }

  const rawPayload = req.body.toString('utf8');
  
  const validationResult = validateStripeSignature(signature, rawPayload);
  
  if (!validationResult.valid) {
    return res.status(RESPONSE_CODES.BAD_REQUEST).json({
      error: validationResult.errorMessage,
      code: validationResult.errorCode
    });
  }

  let event: StripeWebhookEvent;
  try {
    event = JSON.parse(rawPayload);
  } catch {
    return res.status(RESPONSE_CODES.BAD_REQUEST).json({
      error: 'Invalid JSON payload',
      code: 'INVALID_JSON'
    });
  }

  const idempotencyKey = generateIdempotencyKey('stripe', event.id, event.type);
  const idempotencyCheck = checkIdempotency(idempotencyKey);
  
  if (idempotencyCheck.isDuplicate && idempotencyCheck.existingResult) {
    return res.status(RESPONSE_CODES.ACCEPTED).json({
      accepted: true,
      paymentId: idempotencyCheck.existingResult.paymentId,
      salviBatchRef: idempotencyCheck.existingResult.salviBatchRef,
      queuedAt: receivedAt,
      estimatedProcessingTimeMs: 0,
      duplicate: true,
      idempotencyKey
    });
  }

  const { salviBatchRef, kernelOpId } = extractSalviMetadata(event);
  
  const paymentId = await PaymentQueue.enqueue('stripe', event, salviBatchRef, kernelOpId);
  
  recordProcessedWebhook(idempotencyKey, paymentId, salviBatchRef);

  const result: WebhookProcessingResult = {
    accepted: true,
    paymentId,
    salviBatchRef,
    queuedAt: receivedAt,
    estimatedProcessingTimeMs: 5000,
    idempotencyKey
  };

  return res.status(RESPONSE_CODES.ACCEPTED).json(result);
});

stripeWebhookRouter.get('/health', (_req: Request, res: Response) => {
  res.status(200).json({ status: 'healthy', gateway: 'stripe' });
});
