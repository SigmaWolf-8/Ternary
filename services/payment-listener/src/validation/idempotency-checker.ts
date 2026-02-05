import { webhookConfig } from '../../config/webhook-config';

interface IdempotencyEntry {
  key: string;
  processedAt: number;
  result: IdempotencyResult;
}

interface IdempotencyResult {
  paymentId: string;
  salviBatchRef: string;
  status: 'processed' | 'duplicate';
}

const idempotencyStore = new Map<string, IdempotencyEntry>();

let cleanupInterval: NodeJS.Timeout | null = null;

export function initializeIdempotencyChecker(): void {
  cleanupInterval = setInterval(() => {
    cleanupExpiredEntries();
  }, 60000);
}

export function shutdownIdempotencyChecker(): void {
  if (cleanupInterval) {
    clearInterval(cleanupInterval);
    cleanupInterval = null;
  }
  idempotencyStore.clear();
}

function cleanupExpiredEntries(): void {
  const now = Date.now();
  const ttlMs = webhookConfig.idempotencyKeyTtlSeconds * 1000;
  
  for (const [key, entry] of idempotencyStore.entries()) {
    if (now - entry.processedAt > ttlMs) {
      idempotencyStore.delete(key);
    }
  }
}

export function generateIdempotencyKey(
  gateway: string,
  eventId: string,
  eventType: string
): string {
  return `${gateway}:${eventId}:${eventType}`;
}

export interface CheckResult {
  isDuplicate: boolean;
  existingResult?: IdempotencyResult;
}

export function checkIdempotency(idempotencyKey: string): CheckResult {
  const existing = idempotencyStore.get(idempotencyKey);
  
  if (existing) {
    return {
      isDuplicate: true,
      existingResult: existing.result
    };
  }
  
  return { isDuplicate: false };
}

export function recordProcessedWebhook(
  idempotencyKey: string,
  paymentId: string,
  salviBatchRef: string
): void {
  const entry: IdempotencyEntry = {
    key: idempotencyKey,
    processedAt: Date.now(),
    result: {
      paymentId,
      salviBatchRef,
      status: 'processed'
    }
  };
  
  idempotencyStore.set(idempotencyKey, entry);
}

export function getIdempotencyStats(): {
  totalEntries: number;
  oldestEntryAge: number | null;
  newestEntryAge: number | null;
} {
  if (idempotencyStore.size === 0) {
    return {
      totalEntries: 0,
      oldestEntryAge: null,
      newestEntryAge: null
    };
  }
  
  const now = Date.now();
  let oldestTime = now;
  let newestTime = 0;
  
  for (const entry of idempotencyStore.values()) {
    if (entry.processedAt < oldestTime) {
      oldestTime = entry.processedAt;
    }
    if (entry.processedAt > newestTime) {
      newestTime = entry.processedAt;
    }
  }
  
  return {
    totalEntries: idempotencyStore.size,
    oldestEntryAge: now - oldestTime,
    newestEntryAge: now - newestTime
  };
}
