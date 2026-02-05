import { HCSClient, HCSSubmitResult } from './hcs-client';

export interface WitnessRequest {
  operationId: string;
  batchRef: string;
  dataHash: string;
  timestamp: string;
  securityMode: string;
}

export interface WitnessResponse {
  success: boolean;
  operationId: string;
  hedera: HCSSubmitResult;
  witnessedAt: string;
}

export interface WitnessStatus {
  transactionId: string;
  status: 'PENDING' | 'SUCCESS' | 'FAILED';
  consensusTimestamp?: string;
  topicId?: string;
  sequenceNumber?: number;
}

export class WitnessService {
  private hcsClient: HCSClient;
  private witnessCache: Map<string, WitnessResponse> = new Map();

  constructor(hcsClient: HCSClient) {
    this.hcsClient = hcsClient;
  }

  async submitWitness(request: WitnessRequest): Promise<WitnessResponse> {
    const message = JSON.stringify({
      version: '1.0',
      type: 'salvi_witness',
      operationId: request.operationId,
      batchRef: request.batchRef,
      dataHash: request.dataHash,
      timestamp: request.timestamp,
      securityMode: request.securityMode,
      witnessedAt: new Date().toISOString()
    });

    const result = await this.hcsClient.submitMessage({
      topicId: this.hcsClient.getTopicId(),
      message,
      operationId: request.operationId
    });

    const response: WitnessResponse = {
      success: true,
      operationId: request.operationId,
      hedera: result,
      witnessedAt: new Date().toISOString()
    };

    this.witnessCache.set(result.transactionId, response);

    return response;
  }

  async getWitnessStatus(transactionId: string): Promise<WitnessStatus> {
    const cached = this.witnessCache.get(transactionId);
    
    if (cached) {
      return {
        transactionId,
        status: 'SUCCESS',
        consensusTimestamp: cached.hedera.consensusTimestamp,
        topicId: cached.hedera.topicId,
        sequenceNumber: cached.hedera.sequenceNumber
      };
    }

    return {
      transactionId,
      status: 'PENDING'
    };
  }

  async verifyWitness(transactionId: string): Promise<{
    verified: boolean;
    message?: string;
    consensusTimestamp?: string;
  }> {
    const status = await this.getWitnessStatus(transactionId);
    
    return {
      verified: status.status === 'SUCCESS',
      consensusTimestamp: status.consensusTimestamp
    };
  }
}
