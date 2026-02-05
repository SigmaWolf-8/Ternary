export interface WebhookConfig {
  stripeWebhookSecret: string;
  interacWebhookSecret: string;
  cryptoWebhookSecret: string;
  signatureToleranceSeconds: number;
  rateLimitWindowMs: number;
  rateLimitMax: number;
  allowedOrigins: string[];
  idempotencyKeyTtlSeconds: number;
  maxPayloadSizeBytes: number;
  webhookTimeoutMs: number;
  retryAttempts: number;
  retryDelayMs: number;
}

export const webhookConfig: WebhookConfig = {
  stripeWebhookSecret: process.env.STRIPE_WEBHOOK_SECRET || '',
  interacWebhookSecret: process.env.INTERAC_WEBHOOK_SECRET || '',
  cryptoWebhookSecret: process.env.CRYPTO_WEBHOOK_SECRET || '',
  signatureToleranceSeconds: parseInt(process.env.SIGNATURE_TOLERANCE_SECONDS || '300', 10),
  rateLimitWindowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '60000', 10),
  rateLimitMax: parseInt(process.env.RATE_LIMIT_MAX || '100', 10),
  allowedOrigins: (process.env.ALLOWED_ORIGINS || '*').split(','),
  idempotencyKeyTtlSeconds: parseInt(process.env.IDEMPOTENCY_KEY_TTL_SECONDS || '86400', 10),
  maxPayloadSizeBytes: parseInt(process.env.MAX_PAYLOAD_SIZE_BYTES || '1048576', 10),
  webhookTimeoutMs: parseInt(process.env.WEBHOOK_TIMEOUT_MS || '30000', 10),
  retryAttempts: parseInt(process.env.RETRY_ATTEMPTS || '3', 10),
  retryDelayMs: parseInt(process.env.RETRY_DELAY_MS || '1000', 10)
};

export const RATE_LIMITS = {
  STRIPE: { requests_per_minute: 100, burst: 20 },
  INTERAC: { requests_per_minute: 50, burst: 10 },
  CRYPTO: { requests_per_minute: 200, burst: 40 }
} as const;

export const WEBHOOK_ENDPOINTS = {
  STRIPE: '/api/v1/webhooks/stripe',
  INTERAC: '/api/v1/webhooks/interac',
  CRYPTO: '/api/v1/webhooks/crypto'
} as const;

export const RESPONSE_CODES = {
  ACCEPTED: 202,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  TOO_MANY_REQUESTS: 429,
  INTERNAL_ERROR: 500
} as const;
