import { v4 as uuidv4 } from 'uuid';
import { QueuedPayment, PaymentGateway, StripeWebhookEvent, InteracWebhookEvent, CryptoWebhookEvent } from '../models/webhook-models';
import { webhookConfig } from '../../config/webhook-config';

interface QueueStats {
  pending: number;
  processing: number;
  completed: number;
  failed: number;
}

type PaymentEventHandler = (payment: QueuedPayment) => Promise<void>;

class PaymentQueueManager {
  private queue: Map<string, QueuedPayment> = new Map();
  private processingSet: Set<string> = new Set();
  private completedCount: number = 0;
  private failedCount: number = 0;
  private handlers: PaymentEventHandler[] = [];
  private isProcessing: boolean = false;
  private processingInterval: NodeJS.Timeout | null = null;

  async initialize(): Promise<void> {
    this.isProcessing = true;
    this.processingInterval = setInterval(() => {
      this.processQueue();
    }, 100);
  }

  async shutdown(): Promise<void> {
    this.isProcessing = false;
    if (this.processingInterval) {
      clearInterval(this.processingInterval);
      this.processingInterval = null;
    }
  }

  registerHandler(handler: PaymentEventHandler): void {
    this.handlers.push(handler);
  }

  async enqueue(
    gateway: PaymentGateway,
    event: StripeWebhookEvent | InteracWebhookEvent | CryptoWebhookEvent,
    salviBatchRef: string,
    kernelOpId: string
  ): Promise<string> {
    const id = uuidv4();
    
    const queuedPayment: QueuedPayment = {
      id,
      gateway,
      event,
      receivedAt: new Date().toISOString(),
      salviBatchRef,
      kernelOpId,
      retryCount: 0,
      maxRetries: webhookConfig.retryAttempts
    };

    this.queue.set(id, queuedPayment);
    
    return id;
  }

  private async processQueue(): Promise<void> {
    if (!this.isProcessing) return;

    for (const [id, payment] of this.queue.entries()) {
      if (this.processingSet.has(id)) continue;

      this.processingSet.add(id);

      try {
        for (const handler of this.handlers) {
          await handler(payment);
        }
        
        this.queue.delete(id);
        this.processingSet.delete(id);
        this.completedCount++;
      } catch (error) {
        payment.retryCount++;
        
        if (payment.retryCount >= payment.maxRetries) {
          this.queue.delete(id);
          this.failedCount++;
        }
        
        this.processingSet.delete(id);
      }
    }
  }

  getPayment(id: string): QueuedPayment | undefined {
    return this.queue.get(id);
  }

  getStats(): QueueStats {
    return {
      pending: this.queue.size - this.processingSet.size,
      processing: this.processingSet.size,
      completed: this.completedCount,
      failed: this.failedCount
    };
  }

  getQueueSize(): number {
    return this.queue.size;
  }

  isHealthy(): boolean {
    return this.isProcessing && this.queue.size < 10000;
  }
}

export const PaymentQueue = new PaymentQueueManager();

export async function defaultPaymentHandler(payment: QueuedPayment): Promise<void> {
  console.log(`Processing payment ${payment.id} from ${payment.gateway}`, {
    salviBatchRef: payment.salviBatchRef,
    kernelOpId: payment.kernelOpId
  });
}

PaymentQueue.registerHandler(defaultPaymentHandler);
