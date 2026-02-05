import crypto from 'crypto';
import { WebhookValidationResult, PaymentGateway } from '../models/webhook-models';
import { webhookConfig } from '../../config/webhook-config';

export function validateStripeSignature(
  signature: string,
  payload: string
): WebhookValidationResult {
  if (!webhookConfig.stripeWebhookSecret) {
    return { valid: false, errorCode: 'MISSING_SECRET', errorMessage: 'Stripe webhook secret not configured' };
  }

  const signatureParts = signature.split(',');
  const timestampPart = signatureParts.find(p => p.startsWith('t='));
  const signaturePart = signatureParts.find(p => p.startsWith('v1='));

  if (!timestampPart || !signaturePart) {
    return { valid: false, errorCode: 'INVALID_SIGNATURE_FORMAT', errorMessage: 'Missing timestamp or signature' };
  }

  const timestamp = parseInt(timestampPart.split('=')[1], 10);
  const v1Signature = signaturePart.split('=')[1];

  const currentTime = Math.floor(Date.now() / 1000);
  if (Math.abs(currentTime - timestamp) > webhookConfig.signatureToleranceSeconds) {
    return { valid: false, errorCode: 'TIMESTAMP_EXPIRED', errorMessage: 'Webhook timestamp too old' };
  }

  const signedPayload = `${timestamp}.${payload}`;
  const expectedSignature = crypto
    .createHmac('sha256', webhookConfig.stripeWebhookSecret)
    .update(signedPayload, 'utf8')
    .digest('hex');

  try {
    const isValid = crypto.timingSafeEqual(
      Buffer.from(v1Signature, 'hex'),
      Buffer.from(expectedSignature, 'hex')
    );
    
    if (!isValid) {
      return { valid: false, errorCode: 'INVALID_SIGNATURE', errorMessage: 'Signature verification failed' };
    }
    
    return { valid: true, timestamp };
  } catch {
    return { valid: false, errorCode: 'SIGNATURE_COMPARISON_FAILED', errorMessage: 'Failed to compare signatures' };
  }
}

export function validateInteracSignature(
  signature: string,
  payload: string
): WebhookValidationResult {
  if (!webhookConfig.interacWebhookSecret) {
    return { valid: false, errorCode: 'MISSING_SECRET', errorMessage: 'Interac webhook secret not configured' };
  }

  const expectedSignature = crypto
    .createHmac('sha512', webhookConfig.interacWebhookSecret)
    .update(payload, 'utf8')
    .digest('hex');

  try {
    const isValid = crypto.timingSafeEqual(
      Buffer.from(signature, 'hex'),
      Buffer.from(expectedSignature, 'hex')
    );
    
    if (!isValid) {
      return { valid: false, errorCode: 'INVALID_SIGNATURE', errorMessage: 'Signature verification failed' };
    }
    
    return { valid: true, timestamp: Date.now() };
  } catch {
    return { valid: false, errorCode: 'SIGNATURE_COMPARISON_FAILED', errorMessage: 'Failed to compare signatures' };
  }
}

export function validateCryptoSignature(
  signature: string,
  payload: string
): WebhookValidationResult {
  if (!webhookConfig.cryptoWebhookSecret) {
    return { valid: false, errorCode: 'MISSING_SECRET', errorMessage: 'Crypto webhook secret not configured' };
  }

  const expectedSignature = crypto
    .createHmac('sha512', webhookConfig.cryptoWebhookSecret)
    .update(payload, 'utf8')
    .digest('hex');

  try {
    const isValid = crypto.timingSafeEqual(
      Buffer.from(signature, 'hex'),
      Buffer.from(expectedSignature, 'hex')
    );
    
    if (!isValid) {
      return { valid: false, errorCode: 'INVALID_SIGNATURE', errorMessage: 'Signature verification failed' };
    }
    
    return { valid: true, timestamp: Date.now() };
  } catch {
    return { valid: false, errorCode: 'SIGNATURE_COMPARISON_FAILED', errorMessage: 'Failed to compare signatures' };
  }
}

export function validateWebhookSignature(
  gateway: PaymentGateway,
  signature: string,
  payload: string
): WebhookValidationResult {
  switch (gateway) {
    case 'stripe':
      return validateStripeSignature(signature, payload);
    case 'interac':
      return validateInteracSignature(signature, payload);
    case 'crypto':
      return validateCryptoSignature(signature, payload);
    default:
      return { valid: false, errorCode: 'UNKNOWN_GATEWAY', errorMessage: `Unknown payment gateway: ${gateway}` };
  }
}
