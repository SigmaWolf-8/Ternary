/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

vi.mock('algosdk', () => ({
  default: {},
  Algodv2: vi.fn(),
  Indexer: vi.fn(),
  makeApplicationCallTxnFromObject: vi.fn(),
}));

vi.mock('@hashgraph/sdk', () => ({
  Client: { forTestnet: vi.fn(() => ({ setOperator: vi.fn() })) },
  TopicMessageSubmitTransaction: vi.fn(),
  TopicInfoQuery: vi.fn(),
  PrivateKey: { fromString: vi.fn() },
}));

vi.mock('xrpl', () => ({
  Client: vi.fn(() => ({
    connect: vi.fn(),
    disconnect: vi.fn(),
    submitAndWait: vi.fn(),
    request: vi.fn(),
  })),
  Wallet: { fromSeed: vi.fn() },
}));

import { AlgorandClient, AlgorandTxResult } from '../../services/blockchain/algorand-service/src/algorand-client';
import { ContractService, ContractCallRequest, ContractCallResponse, TransactionStatus } from '../../services/blockchain/algorand-service/src/contract-service';
import { HCSClient, HCSMessage, HCSSubmitResult } from '../../services/blockchain/hedera-service/src/hcs-client';
import { WitnessService, WitnessRequest, WitnessResponse, WitnessStatus } from '../../services/blockchain/hedera-service/src/witness-service';
import { XRPLClient, XRPLPaymentResult } from '../../services/blockchain/xrpl-service/src/xrpl-client';
import { PaymentService, PaymentRequest, PaymentResponse, PaymentStatus } from '../../services/blockchain/xrpl-service/src/payment-service';

describe('Blockchain Services', () => {

  describe('AlgorandClient', () => {
    let client: AlgorandClient;

    beforeEach(() => {
      client = new AlgorandClient();
    });

    it('should initialize with default configuration', () => {
      expect(client.getAlgodUrl()).toBe('https://testnet-api.algonode.cloud');
      expect(client.getIndexerUrl()).toBe('https://testnet-idx.algonode.cloud');
    });

    it('should initialize with environment variables', () => {
      const originalAlgod = process.env.ALGORAND_ALGOD_URL;
      const originalIndexer = process.env.ALGORAND_INDEXER_URL;
      process.env.ALGORAND_ALGOD_URL = 'https://custom-algod.example.com';
      process.env.ALGORAND_INDEXER_URL = 'https://custom-indexer.example.com';

      const customClient = new AlgorandClient();
      expect(customClient.getAlgodUrl()).toBe('https://custom-algod.example.com');
      expect(customClient.getIndexerUrl()).toBe('https://custom-indexer.example.com');

      if (originalAlgod === undefined) delete process.env.ALGORAND_ALGOD_URL;
      else process.env.ALGORAND_ALGOD_URL = originalAlgod;
      if (originalIndexer === undefined) delete process.env.ALGORAND_INDEXER_URL;
      else process.env.ALGORAND_INDEXER_URL = originalIndexer;
    });

    it('should return a valid AlgorandTxResult from callApplication', async () => {
      const result = await client.callApplication({
        appId: 12345,
        method: 'witness_operation',
        args: ['test-arg'],
      });

      expect(result).toBeDefined();
      expect(result.appId).toBe(12345);
      expect(typeof result.round).toBe('number');
      expect(typeof result.txId).toBe('string');
      expect(result.txId.length).toBe(52);
      expect(result.confirmedRound).toBe(result.round + 1);
      expect(result.globalStateDelta).toBeDefined();
    });

    it('should return valid transaction data from getTransaction', async () => {
      const tx = await client.getTransaction('SOMETXID');

      expect(tx).not.toBeNull();
      expect(tx!.txId).toBe('SOMETXID');
      expect(tx!.confirmed).toBe(true);
      expect(tx!.type).toBe('appl');
      expect(typeof tx!.round).toBe('number');
    });

    it('should return valid application info from getApplicationInfo', async () => {
      const info = await client.getApplicationInfo(99999);

      expect(info.appId).toBe(99999);
      expect(typeof info.creator).toBe('string');
      expect(info.creator.length).toBe(58);
      expect(info.globalStateSchema.numUint).toBe(16);
      expect(info.globalStateSchema.numByteSlice).toBe(16);
      expect(info.localStateSchema.numUint).toBe(0);
    });

    it('should return a current round number', async () => {
      const round = await client.getCurrentRound();
      expect(round).toBeGreaterThanOrEqual(30000000);
      expect(round).toBeLessThan(30002000);
    });

    it('should generate valid base32 transaction IDs', async () => {
      const result = await client.callApplication({
        appId: 1,
        method: 'test',
        args: [],
      });
      expect(result.txId).toMatch(/^[A-Z2-7]{52}$/);
    });
  });

  describe('ContractService', () => {
    let algorandClient: AlgorandClient;
    let contractService: ContractService;

    beforeEach(() => {
      algorandClient = new AlgorandClient();
      contractService = new ContractService(algorandClient);
    });

    it('should return a ContractCallResponse with correct shape', async () => {
      const request: ContractCallRequest = {
        operationId: 'op-123',
        appId: 12345,
        method: 'witness_operation',
        args: ['arg1', 'arg2'],
      };

      const response = await contractService.callApplication(request);

      expect(response.success).toBe(true);
      expect(response.operationId).toBe('op-123');
      expect(response.algorand).toBeDefined();
      expect(response.algorand.appId).toBe(12345);
      expect(typeof response.executedAt).toBe('string');
      expect(new Date(response.executedAt).getTime()).not.toBeNaN();
    });

    it('should cache call results and retrieve them by txId', async () => {
      const request: ContractCallRequest = {
        operationId: 'op-456',
        appId: 99999,
        method: 'record_batch',
        args: [],
      };

      const response = await contractService.callApplication(request);
      const status = await contractService.getTransactionStatus(response.algorand.txId);

      expect(status.txId).toBe(response.algorand.txId);
      expect(status.confirmed).toBe(true);
      expect(status.type).toBe('appl');
      expect(status.appId).toBe(99999);
      expect(status.round).toBe(response.algorand.confirmedRound);
    });

    it('should fall back to algorand client for uncached transactions', async () => {
      const status = await contractService.getTransactionStatus('UNCACHED_TX_ID');

      expect(status.txId).toBe('UNCACHED_TX_ID');
      expect(status.confirmed).toBe(true);
      expect(status.type).toBe('appl');
    });

    it('should handle null transaction from algorand client', async () => {
      vi.spyOn(algorandClient, 'getTransaction').mockResolvedValueOnce(null);

      const status = await contractService.getTransactionStatus('NONEXISTENT_TX');

      expect(status.txId).toBe('NONEXISTENT_TX');
      expect(status.confirmed).toBe(false);
      expect(status.round).toBe(0);
      expect(status.type).toBe('unknown');
    });

    it('should verify execution for confirmed transactions', async () => {
      const request: ContractCallRequest = {
        operationId: 'op-789',
        appId: 11111,
        method: 'verify_test',
        args: [],
      };

      const response = await contractService.callApplication(request);
      const verification = await contractService.verifyExecution(response.algorand.txId);

      expect(verification.verified).toBe(true);
      expect(verification.round).toBeDefined();
      expect(verification.appId).toBe(11111);
    });

    it('should report unverified for unknown transactions', async () => {
      vi.spyOn(algorandClient, 'getTransaction').mockResolvedValueOnce(null);

      const verification = await contractService.verifyExecution('UNKNOWN_TX');

      expect(verification.verified).toBe(false);
      expect(verification.round).toBe(0);
    });
  });

  describe('HCSClient', () => {
    let client: HCSClient;

    beforeEach(() => {
      client = new HCSClient();
    });

    it('should initialize with default configuration', () => {
      expect(client.getTopicId()).toBe('0.0.12345');
      expect(client.getNetwork()).toBe('testnet');
    });

    it('should initialize with environment variables', () => {
      const originalTopic = process.env.HEDERA_TOPIC_ID;
      const originalNetwork = process.env.HEDERA_NETWORK;
      process.env.HEDERA_TOPIC_ID = '0.0.99999';
      process.env.HEDERA_NETWORK = 'mainnet';

      const customClient = new HCSClient();
      expect(customClient.getTopicId()).toBe('0.0.99999');
      expect(customClient.getNetwork()).toBe('mainnet');

      if (originalTopic === undefined) delete process.env.HEDERA_TOPIC_ID;
      else process.env.HEDERA_TOPIC_ID = originalTopic;
      if (originalNetwork === undefined) delete process.env.HEDERA_NETWORK;
      else process.env.HEDERA_NETWORK = originalNetwork;
    });

    it('should return a valid HCSSubmitResult from submitMessage', async () => {
      const message: HCSMessage = {
        topicId: '0.0.12345',
        message: 'test witness record',
        operationId: 'op-test',
      };

      const result = await client.submitMessage(message);

      expect(result.topicId).toBe('0.0.12345');
      expect(typeof result.sequenceNumber).toBe('number');
      expect(result.sequenceNumber).toBeGreaterThan(0);
      expect(typeof result.transactionId).toBe('string');
      expect(result.transactionId).toContain('@');
      expect(typeof result.consensusTimestamp).toBe('string');
      expect(typeof result.runningHash).toBe('string');
      expect(result.runningHash.length).toBe(64);
    });

    it('should return message by sequence number', async () => {
      const message = await client.getMessageBySequence('0.0.12345', 42);

      expect(message).not.toBeNull();
      expect(message!.sequenceNumber).toBe(42);
      expect(typeof message!.contents).toBe('string');
      expect(typeof message!.consensusTimestamp).toBe('string');
      expect(message!.runningHash.length).toBe(64);
    });

    it('should return topic info', async () => {
      const info = await client.getTopicInfo('0.0.12345');

      expect(info.topicId).toBe('0.0.12345');
      expect(info.sequenceNumber).toBe(12345);
      expect(info.runningHash.length).toBe(64);
      expect(typeof info.expirationTime).toBe('string');
      expect(new Date(info.expirationTime).getTime()).toBeGreaterThan(Date.now());
    });

    it('should generate valid hex hashes', async () => {
      const result = await client.submitMessage({
        topicId: '0.0.12345',
        message: 'hash test',
        operationId: 'hash-op',
      });
      expect(result.runningHash).toMatch(/^[0-9a-f]{64}$/);
    });
  });

  describe('WitnessService', () => {
    let hcsClient: HCSClient;
    let witnessService: WitnessService;

    beforeEach(() => {
      hcsClient = new HCSClient();
      witnessService = new WitnessService(hcsClient);
    });

    it('should submit a witness and return a WitnessResponse', async () => {
      const request: WitnessRequest = {
        operationId: 'witness-op-1',
        batchRef: 'BATCH_001',
        dataHash: 'abc123def456',
        timestamp: new Date().toISOString(),
        securityMode: 'CNSA2',
      };

      const response = await witnessService.submitWitness(request);

      expect(response.success).toBe(true);
      expect(response.operationId).toBe('witness-op-1');
      expect(response.hedera).toBeDefined();
      expect(response.hedera.topicId).toBe('0.0.12345');
      expect(typeof response.witnessedAt).toBe('string');
    });

    it('should serialize correct JSON message to HCS', async () => {
      const submitSpy = vi.spyOn(hcsClient, 'submitMessage');

      const request: WitnessRequest = {
        operationId: 'witness-op-2',
        batchRef: 'BATCH_002',
        dataHash: 'deadbeef',
        timestamp: '2026-02-14T00:00:00.000Z',
        securityMode: 'FIPS_140_3',
      };

      await witnessService.submitWitness(request);

      expect(submitSpy).toHaveBeenCalledOnce();
      const call = submitSpy.mock.calls[0][0];
      const parsed = JSON.parse(call.message);

      expect(parsed.version).toBe('1.0');
      expect(parsed.type).toBe('salvi_witness');
      expect(parsed.operationId).toBe('witness-op-2');
      expect(parsed.batchRef).toBe('BATCH_002');
      expect(parsed.dataHash).toBe('deadbeef');
      expect(parsed.securityMode).toBe('FIPS_140_3');
    });

    it('should cache witness and return SUCCESS status', async () => {
      const request: WitnessRequest = {
        operationId: 'witness-op-3',
        batchRef: 'BATCH_003',
        dataHash: 'feedface',
        timestamp: new Date().toISOString(),
        securityMode: 'CNSA2',
      };

      const response = await witnessService.submitWitness(request);
      const status = await witnessService.getWitnessStatus(response.hedera.transactionId);

      expect(status.status).toBe('SUCCESS');
      expect(status.transactionId).toBe(response.hedera.transactionId);
      expect(status.topicId).toBe(response.hedera.topicId);
      expect(status.sequenceNumber).toBe(response.hedera.sequenceNumber);
    });

    it('should return PENDING for unknown transaction IDs', async () => {
      const status = await witnessService.getWitnessStatus('unknown-tx-id');

      expect(status.status).toBe('PENDING');
      expect(status.transactionId).toBe('unknown-tx-id');
      expect(status.consensusTimestamp).toBeUndefined();
    });

    it('should verify cached witnesses', async () => {
      const request: WitnessRequest = {
        operationId: 'witness-op-4',
        batchRef: 'BATCH_004',
        dataHash: 'cafebabe',
        timestamp: new Date().toISOString(),
        securityMode: 'CNSA2',
      };

      const response = await witnessService.submitWitness(request);
      const verification = await witnessService.verifyWitness(response.hedera.transactionId);

      expect(verification.verified).toBe(true);
      expect(verification.consensusTimestamp).toBeDefined();
    });

    it('should not verify unknown witnesses', async () => {
      const verification = await witnessService.verifyWitness('nonexistent-tx');

      expect(verification.verified).toBe(false);
    });
  });

  describe('XRPLClient', () => {
    let client: XRPLClient;

    beforeEach(() => {
      client = new XRPLClient();
    });

    it('should initialize with default configuration', () => {
      expect(client.getServerUrl()).toBe('wss://s.altnet.rippletest.net:51233');
      expect(client.getWalletAddress()).toBe('rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh');
    });

    it('should initialize with environment variables', () => {
      const originalUrl = process.env.XRPL_SERVER_URL;
      const originalAddr = process.env.XRPL_WALLET_ADDRESS;
      process.env.XRPL_SERVER_URL = 'wss://custom.xrpl.example.com';
      process.env.XRPL_WALLET_ADDRESS = 'rCustomAddress123';

      const customClient = new XRPLClient();
      expect(customClient.getServerUrl()).toBe('wss://custom.xrpl.example.com');
      expect(customClient.getWalletAddress()).toBe('rCustomAddress123');

      if (originalUrl === undefined) delete process.env.XRPL_SERVER_URL;
      else process.env.XRPL_SERVER_URL = originalUrl;
      if (originalAddr === undefined) delete process.env.XRPL_WALLET_ADDRESS;
      else process.env.XRPL_WALLET_ADDRESS = originalAddr;
    });

    it('should manage connection state', async () => {
      expect(client.isConnected()).toBe(false);

      await client.connect();
      expect(client.isConnected()).toBe(true);

      await client.disconnect();
      expect(client.isConnected()).toBe(false);
    });

    it('should return a valid XRPLPaymentResult from submitPayment', async () => {
      const result = await client.submitPayment({
        destination: 'rDestination123',
        amount: '1000000',
        currency: 'XRP',
        memo: 'test payment',
      });

      expect(typeof result.ledgerIndex).toBe('number');
      expect(result.ledgerIndex).toBeGreaterThanOrEqual(80000000);
      expect(typeof result.transactionHash).toBe('string');
      expect(result.transactionHash.length).toBe(64);
      expect(result.validated).toBe(true);
      expect(result.fee).toBe('12');
      expect(result.result).toBe('tesSUCCESS');
    });

    it('should return transaction by hash', async () => {
      const tx = await client.getTransaction('ABCDEF1234567890');

      expect(tx).not.toBeNull();
      expect(tx!.hash).toBe('ABCDEF1234567890');
      expect(tx!.validated).toBe(true);
      expect(tx!.result).toBe('tesSUCCESS');
    });

    it('should return current ledger index', async () => {
      const index = await client.getLedgerIndex();
      expect(index).toBeGreaterThanOrEqual(80000000);
      expect(index).toBeLessThan(80002000);
    });

    it('should generate valid uppercase hex transaction hashes', async () => {
      const result = await client.submitPayment({
        destination: 'rTest',
        amount: '100',
        currency: 'XRP',
      });
      expect(result.transactionHash).toMatch(/^[0-9A-F]{64}$/);
    });
  });

  describe('PaymentService', () => {
    let xrplClient: XRPLClient;
    let paymentService: PaymentService;

    beforeEach(() => {
      xrplClient = new XRPLClient();
      paymentService = new PaymentService(xrplClient);
    });

    it('should submit a payment and return a PaymentResponse', async () => {
      const request: PaymentRequest = {
        operationId: 'pay-001',
        amount: '5000000',
        currency: 'XRP',
        destination: 'rRecipient123',
        memo: 'salvi settlement',
      };

      const response = await paymentService.submitPayment(request);

      expect(response.success).toBe(true);
      expect(response.operationId).toBe('pay-001');
      expect(response.xrpl).toBeDefined();
      expect(response.xrpl.result).toBe('tesSUCCESS');
      expect(typeof response.settledAt).toBe('string');
      expect(new Date(response.settledAt).getTime()).not.toBeNaN();
    });

    it('should cache payment and retrieve status by txHash', async () => {
      const request: PaymentRequest = {
        operationId: 'pay-002',
        amount: '1000000',
        currency: 'XRP',
        destination: 'rDest456',
      };

      const response = await paymentService.submitPayment(request);
      const status = await paymentService.getPaymentStatus(response.xrpl.transactionHash);

      expect(status.transactionHash).toBe(response.xrpl.transactionHash);
      expect(status.validated).toBe(true);
      expect(status.result).toBe('tesSUCCESS');
      expect(status.fee).toBe('12');
      expect(status.ledgerIndex).toBe(response.xrpl.ledgerIndex);
    });

    it('should fall back to xrpl client for uncached payments', async () => {
      const status = await paymentService.getPaymentStatus('UNCACHED_HASH');

      expect(status.transactionHash).toBe('UNCACHED_HASH');
      expect(status.validated).toBe(true);
      expect(status.result).toBe('tesSUCCESS');
    });

    it('should handle null transaction from xrpl client', async () => {
      vi.spyOn(xrplClient, 'getTransaction').mockResolvedValueOnce(null);

      const status = await paymentService.getPaymentStatus('NONEXISTENT');

      expect(status.transactionHash).toBe('NONEXISTENT');
      expect(status.validated).toBe(false);
      expect(status.ledgerIndex).toBe(0);
      expect(status.result).toBe('NOT_FOUND');
    });

    it('should verify successful payments', async () => {
      const request: PaymentRequest = {
        operationId: 'pay-003',
        amount: '2000000',
        currency: 'XRP',
        destination: 'rVerify789',
      };

      const response = await paymentService.submitPayment(request);
      const verification = await paymentService.verifyPayment(response.xrpl.transactionHash);

      expect(verification.verified).toBe(true);
      expect(verification.ledgerIndex).toBeDefined();
    });

    it('should not verify nonexistent payments', async () => {
      vi.spyOn(xrplClient, 'getTransaction').mockResolvedValueOnce(null);

      const verification = await paymentService.verifyPayment('MISSING_HASH');

      expect(verification.verified).toBe(false);
      expect(verification.ledgerIndex).toBe(0);
    });

    it('should set success based on tesSUCCESS result', async () => {
      vi.spyOn(xrplClient, 'submitPayment').mockResolvedValueOnce({
        ledgerIndex: 80000001,
        transactionHash: 'FAILHASH',
        validated: true,
        fee: '12',
        result: 'tecUNFUNDED_PAYMENT',
      });

      const response = await paymentService.submitPayment({
        operationId: 'pay-fail',
        amount: '999999999',
        currency: 'XRP',
        destination: 'rBroke',
      });

      expect(response.success).toBe(false);
      expect(response.xrpl.result).toBe('tecUNFUNDED_PAYMENT');
    });
  });

  describe('Interface Contract Shapes', () => {
    it('AlgorandTxResult should have required fields', () => {
      const result: AlgorandTxResult = {
        appId: 1,
        round: 100,
        txId: 'TXID',
        confirmedRound: 101,
      };
      expect(result).toBeDefined();
      expect(result.globalStateDelta).toBeUndefined();
    });

    it('HCSSubmitResult should have required fields', () => {
      const result: HCSSubmitResult = {
        topicId: '0.0.1',
        sequenceNumber: 1,
        transactionId: '0.0.1@123.456',
        consensusTimestamp: '2026-01-01T00:00:00Z',
        runningHash: 'abc123',
      };
      expect(result.topicId).toBe('0.0.1');
      expect(result.sequenceNumber).toBe(1);
    });

    it('XRPLPaymentResult should have required fields', () => {
      const result: XRPLPaymentResult = {
        ledgerIndex: 80000000,
        transactionHash: 'HASH',
        validated: true,
        fee: '12',
        result: 'tesSUCCESS',
      };
      expect(result.validated).toBe(true);
      expect(result.result).toBe('tesSUCCESS');
    });

    it('WitnessRequest should enforce all required fields', () => {
      const request: WitnessRequest = {
        operationId: 'op-1',
        batchRef: 'BATCH_1',
        dataHash: 'hash',
        timestamp: '2026-01-01T00:00:00Z',
        securityMode: 'CNSA2',
      };
      expect(request.operationId).toBe('op-1');
      expect(request.securityMode).toBe('CNSA2');
    });

    it('ContractCallRequest should enforce all required fields', () => {
      const request: ContractCallRequest = {
        operationId: 'op-2',
        appId: 12345,
        method: 'test_method',
        args: [1, 'two', true],
      };
      expect(request.operationId).toBe('op-2');
      expect(request.args).toHaveLength(3);
    });

    it('PaymentRequest should enforce all required fields', () => {
      const request: PaymentRequest = {
        operationId: 'op-3',
        amount: '1000',
        currency: 'XRP',
        destination: 'rAddr',
      };
      expect(request.memo).toBeUndefined();
    });

    it('WitnessStatus should support all status values', () => {
      const pending: WitnessStatus = { transactionId: 'tx-1', status: 'PENDING' };
      const success: WitnessStatus = {
        transactionId: 'tx-2',
        status: 'SUCCESS',
        consensusTimestamp: '2026-01-01',
        topicId: '0.0.1',
        sequenceNumber: 1,
      };
      const failed: WitnessStatus = { transactionId: 'tx-3', status: 'FAILED' };

      expect(pending.status).toBe('PENDING');
      expect(success.status).toBe('SUCCESS');
      expect(failed.status).toBe('FAILED');
    });
  });
});
