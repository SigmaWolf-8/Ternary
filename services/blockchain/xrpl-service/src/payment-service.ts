import { XRPLClient, XRPLPaymentResult } from './xrpl-client';

export interface PaymentRequest {
  operationId: string;
  amount: string;
  currency: string;
  destination: string;
  memo?: string;
}

export interface PaymentResponse {
  success: boolean;
  operationId: string;
  xrpl: XRPLPaymentResult;
  settledAt: string;
}

export interface PaymentStatus {
  transactionHash: string;
  validated: boolean;
  ledgerIndex: number;
  result: string;
  fee?: string;
}

export class PaymentService {
  private xrplClient: XRPLClient;
  private paymentCache: Map<string, PaymentResponse> = new Map();

  constructor(xrplClient: XRPLClient) {
    this.xrplClient = xrplClient;
  }

  async submitPayment(request: PaymentRequest): Promise<PaymentResponse> {
    const result = await this.xrplClient.submitPayment({
      destination: request.destination,
      amount: request.amount,
      currency: request.currency,
      memo: request.memo
    });

    const response: PaymentResponse = {
      success: result.result === 'tesSUCCESS',
      operationId: request.operationId,
      xrpl: result,
      settledAt: new Date().toISOString()
    };

    this.paymentCache.set(result.transactionHash, response);

    return response;
  }

  async getPaymentStatus(txHash: string): Promise<PaymentStatus> {
    const cached = this.paymentCache.get(txHash);
    
    if (cached) {
      return {
        transactionHash: txHash,
        validated: cached.xrpl.validated,
        ledgerIndex: cached.xrpl.ledgerIndex,
        result: cached.xrpl.result,
        fee: cached.xrpl.fee
      };
    }

    const tx = await this.xrplClient.getTransaction(txHash);
    
    if (!tx) {
      return {
        transactionHash: txHash,
        validated: false,
        ledgerIndex: 0,
        result: 'NOT_FOUND'
      };
    }

    return {
      transactionHash: txHash,
      validated: tx.validated,
      ledgerIndex: tx.ledgerIndex,
      result: tx.result
    };
  }

  async verifyPayment(txHash: string): Promise<{
    verified: boolean;
    ledgerIndex?: number;
  }> {
    const status = await this.getPaymentStatus(txHash);
    
    return {
      verified: status.validated && status.result === 'tesSUCCESS',
      ledgerIndex: status.ledgerIndex
    };
  }
}
