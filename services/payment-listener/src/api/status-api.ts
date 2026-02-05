import { Router, Request, Response } from 'express';
import { PaymentQueue } from '../queue/payment-queue';
import { PaymentStatusResponse } from '../models/webhook-models';

export const statusApiRouter = Router();

const paymentStore = new Map<string, PaymentStatusResponse>();

export function updatePaymentStatus(paymentId: string, status: PaymentStatusResponse): void {
  paymentStore.set(paymentId, status);
}

statusApiRouter.get('/:paymentId', (req: Request, res: Response) => {
  const { paymentId } = req.params;
  
  const storedStatus = paymentStore.get(paymentId);
  if (storedStatus) {
    return res.status(200).json(storedStatus);
  }
  
  const queuedPayment = PaymentQueue.getPayment(paymentId);
  
  if (queuedPayment) {
    const response: PaymentStatusResponse = {
      paymentId: queuedPayment.id,
      status: 'processing',
      amount: 0,
      currency: 'USD',
      gateway: queuedPayment.gateway,
      salviReferences: {
        batchRef: queuedPayment.salviBatchRef,
        kernelOpId: queuedPayment.kernelOpId
      },
      timingMetadata: {
        receivedAt: queuedPayment.receivedAt
      }
    };
    
    return res.status(200).json(response);
  }
  
  return res.status(404).json({
    error: 'Payment not found',
    code: 'PAYMENT_NOT_FOUND',
    paymentId
  });
});

statusApiRouter.get('/', (req: Request, res: Response) => {
  const page = parseInt(req.query.page as string) || 1;
  const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);
  const status = req.query.status as string;
  
  let payments = Array.from(paymentStore.values());
  
  if (status) {
    payments = payments.filter(p => p.status === status);
  }
  
  const startIndex = (page - 1) * limit;
  const endIndex = startIndex + limit;
  const paginatedPayments = payments.slice(startIndex, endIndex);
  
  res.status(200).json({
    payments: paginatedPayments,
    pagination: {
      page,
      limit,
      total: payments.length,
      totalPages: Math.ceil(payments.length / limit)
    }
  });
});

statusApiRouter.get('/:paymentId/timeline', (req: Request, res: Response) => {
  const { paymentId } = req.params;
  
  const storedStatus = paymentStore.get(paymentId);
  
  if (!storedStatus) {
    return res.status(404).json({
      error: 'Payment not found',
      code: 'PAYMENT_NOT_FOUND',
      paymentId
    });
  }
  
  const timeline = [
    {
      event: 'webhook_received',
      timestamp: storedStatus.timingMetadata.receivedAt,
      details: { gateway: storedStatus.gateway }
    }
  ];
  
  if (storedStatus.status !== 'pending') {
    timeline.push({
      event: 'processing_started',
      timestamp: storedStatus.timingMetadata.receivedAt,
      details: { batchRef: storedStatus.salviReferences.batchRef }
    });
  }
  
  if (storedStatus.timingMetadata.processedAt) {
    timeline.push({
      event: 'processing_completed',
      timestamp: storedStatus.timingMetadata.processedAt,
      details: { kernelOpId: storedStatus.salviReferences.kernelOpId }
    });
  }
  
  if (storedStatus.salviReferences.witnessTxId) {
    timeline.push({
      event: 'witnessed',
      timestamp: storedStatus.timingMetadata.processedAt || new Date().toISOString(),
      details: { witnessTxId: storedStatus.salviReferences.witnessTxId }
    });
  }
  
  if (storedStatus.settledAt) {
    timeline.push({
      event: 'settled',
      timestamp: storedStatus.settledAt,
      details: { amount: storedStatus.amount, currency: storedStatus.currency }
    });
  }
  
  res.status(200).json({
    paymentId,
    currentStatus: storedStatus.status,
    timeline
  });
});
