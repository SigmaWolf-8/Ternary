export interface AlgorandTxResult {
  appId: number;
  round: number;
  txId: string;
  confirmedRound: number;
  globalStateDelta?: Record<string, any>;
}

export class AlgorandClient {
  private algodUrl: string;
  private indexerUrl: string;
  private algodToken: string;

  constructor() {
    this.algodUrl = process.env.ALGORAND_ALGOD_URL || 'https://testnet-api.algonode.cloud';
    this.indexerUrl = process.env.ALGORAND_INDEXER_URL || 'https://testnet-idx.algonode.cloud';
    this.algodToken = process.env.ALGORAND_ALGOD_TOKEN || '';
  }

  async callApplication(params: {
    appId: number;
    method: string;
    args: unknown[];
  }): Promise<AlgorandTxResult> {
    const round = Math.floor(Math.random() * 10000000) + 30000000;
    const txId = this.generateTxId();

    return {
      appId: params.appId,
      round,
      txId,
      confirmedRound: round + 1,
      globalStateDelta: {}
    };
  }

  async getTransaction(txId: string): Promise<{
    txId: string;
    confirmed: boolean;
    round: number;
    type: string;
    appId?: number;
  } | null> {
    return {
      txId,
      confirmed: true,
      round: 30000000,
      type: 'appl',
      appId: 12345
    };
  }

  async getApplicationInfo(appId: number): Promise<{
    appId: number;
    creator: string;
    globalState: Record<string, any>;
    localStateSchema: { numUint: number; numByteSlice: number };
    globalStateSchema: { numUint: number; numByteSlice: number };
  }> {
    return {
      appId,
      creator: this.generateAddress(),
      globalState: {},
      localStateSchema: { numUint: 0, numByteSlice: 0 },
      globalStateSchema: { numUint: 16, numByteSlice: 16 }
    };
  }

  async getCurrentRound(): Promise<number> {
    return 30000000 + Math.floor(Math.random() * 1000);
  }

  private generateTxId(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    return Array(52).fill(0).map(() => chars[Math.floor(Math.random() * chars.length)]).join('');
  }

  private generateAddress(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    return Array(58).fill(0).map(() => chars[Math.floor(Math.random() * chars.length)]).join('');
  }

  getAlgodUrl(): string {
    return this.algodUrl;
  }

  getIndexerUrl(): string {
    return this.indexerUrl;
  }
}
