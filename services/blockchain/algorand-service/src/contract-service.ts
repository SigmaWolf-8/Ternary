import { AlgorandClient, AlgorandTxResult } from './algorand-client';

export interface ContractCallRequest {
  operationId: string;
  appId: number;
  method: string;
  args: unknown[];
}

export interface ContractCallResponse {
  success: boolean;
  operationId: string;
  algorand: AlgorandTxResult;
  executedAt: string;
}

export interface TransactionStatus {
  txId: string;
  confirmed: boolean;
  round: number;
  type: string;
  appId?: number;
}

export class ContractService {
  private algorandClient: AlgorandClient;
  private callCache: Map<string, ContractCallResponse> = new Map();

  constructor(algorandClient: AlgorandClient) {
    this.algorandClient = algorandClient;
  }

  async callApplication(request: ContractCallRequest): Promise<ContractCallResponse> {
    const result = await this.algorandClient.callApplication({
      appId: request.appId,
      method: request.method,
      args: request.args
    });

    const response: ContractCallResponse = {
      success: true,
      operationId: request.operationId,
      algorand: result,
      executedAt: new Date().toISOString()
    };

    this.callCache.set(result.txId, response);

    return response;
  }

  async getTransactionStatus(txId: string): Promise<TransactionStatus> {
    const cached = this.callCache.get(txId);
    
    if (cached) {
      return {
        txId,
        confirmed: true,
        round: cached.algorand.confirmedRound,
        type: 'appl',
        appId: cached.algorand.appId
      };
    }

    const tx = await this.algorandClient.getTransaction(txId);
    
    if (!tx) {
      return {
        txId,
        confirmed: false,
        round: 0,
        type: 'unknown'
      };
    }

    return tx;
  }

  async verifyExecution(txId: string): Promise<{
    verified: boolean;
    round?: number;
    appId?: number;
  }> {
    const status = await this.getTransactionStatus(txId);
    
    return {
      verified: status.confirmed,
      round: status.round,
      appId: status.appId
    };
  }
}
