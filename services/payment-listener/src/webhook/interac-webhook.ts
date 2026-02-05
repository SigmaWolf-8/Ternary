import { Router, Request, Response } from 'express';
import { v4 as uuidv4 } from 'uuid';
import { validateInteracSignature } from '../validation/signature-validator';
import { checkIdempotency, recordProcessedWebhook, generateIdempotencyKey } from '../validation/idempotency-checker';
import { PaymentQueue } from '../queue/payment-queue';
import { InteracWebhookEvent, WebhookProcessingResult } from '../models/webhook-models';
import { RESPONSE_CODES } from '../../config/webhook-config';

export const interacWebhookRouter = Router();

function extractSalviMetadata(event: InteracWebhookEvent): { salviBatchRef: string; kernelOpId: string } {
  const metadata = event.payload?.metadata || {};
  return {
    salviBatchRef: metadata.salvi_batch_ref || `BATCH_CAD_${Date.now()}_${uuidv4().slice(0, 8)}`,
    kernelOpId: metadata.kernel_op_id || `OP_CAD_${Date.now()}_${uuidv4().slice(0, 8)}`
  };
}

interacWebhookRouter.post('/', async (req: Request, res: Response) => {
  const receivedAt = new Date().toISOString();
  const signature = req.headers['x-jpmc-signature'] as string;
  
  if (!signature) {
    return res.status(RESPONSE_CODES.BAD_REQUEST).json({
      error: 'Missing X-JPMC-Signature header',
      code: 'MISSING_SIGNATURE'
    });
  }

  const rawPayload = req.body.toString('utf8');
  
  const validationResult = validateInteracSignature(signature, rawPayload);
  
  if (!validationResult.valid) {
    return res.status(RESPONSE_CODES.BAD_REQUEST).json({
      error: validationResult.errorMessage,
      code: validationResult.errorCode
    });
  }

  let event: InteracWebhookEvent;
  try {
    event = JSON.parse(rawPayload);
  } catch {
    return res.status(RESPONSE_CODES.BAD_REQUEST).json({
      error: 'Invalid JSON payload',
      code: 'INVALID_JSON'
    });
  }

  const idempotencyKey = generateIdempotencyKey('interac', event.transaction_id, event.event_type);
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
  
  const paymentId = await PaymentQueue.enqueue('interac', event, salviBatchRef, kernelOpId);
  
  recordProcessedWebhook(idempotencyKey, paymentId, salviBatchRef);

  const result: WebhookProcessingResult = {
    accepted: true,
    paymentId,
    salviBatchRef,
    queuedAt: receivedAt,
    estimatedProcessingTimeMs: 8000,
    idempotencyKey
  };

  return res.status(RESPONSE_CODES.ACCEPTED).json(result);
});

interacWebhookRouter.get('/health', (_req: Request, res: Response) => {
  res.status(200).json({ status: 'healthy', gateway: 'interac' });
});
