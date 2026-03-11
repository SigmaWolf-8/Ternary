/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * HEDERA CONSENSUS SERVICE (HCS) WITNESSING
 * Location: server/services/hedera-witnessing-service.ts
 *
 * Implements IHederaWitnessingService from blockchain-integrations.ts.
 * Submits cryptographic witness hashes to Hedera Consensus Service for
 * immutable, ordered, timestamped proof of PlenumNET operations.
 *
 * Each witness message contains:
 *   - SHA-256/384/512 hash of the operation or Merkle batch root
 *   - Security mode (φ+, φ, 1, 0)
 *   - Kernel operation ID for cross-reference
 *   - Salvi batch reference for audit trail linkage
 *
 * HCS provides:
 *   - Fair ordering (consensus timestamp from Hedera network)
 *   - Immutability (cannot be altered after consensus)
 *   - Mirror node queryability (any third party can verify)
 *   - Sub-$0.01 per message (predictable cost model)
 *
 * Configuration:
 *   HEDERA_NETWORK      — 'testnet' | 'mainnet' | 'previewnet' (default: testnet)
 *   HEDERA_ACCOUNT_ID   — Operator account (e.g., '0.0.12345')
 *   HEDERA_PRIVATE_KEY  — Ed25519 or ECDSA private key (DER hex)
 *   HEDERA_TOPIC_ID     — Existing topic ID (e.g., '0.0.67890')
 *                          If not set, service creates a new topic on init
 */

import {
  Client,
  TopicCreateTransaction,
  TopicMessageSubmitTransaction,
  TopicId,
  TopicInfoQuery,
  AccountId,
  PrivateKey,
  Hbar,
  Status,
} from '@hashgraph/sdk';

import * as crypto from 'crypto';
import * as fs from 'fs';
import * as path from 'path';

import type {
  IHederaWitnessingService,
  HederaWitnessRequest,
  HederaWitnessResponse,
  HederaTransactionDetails,
  HederaCosts,
  HederaVerification,
  HederaTiming,
} from '../salvi-core/blockchain-integrations';

export interface HederaWitnessingConfig {
  network: 'testnet' | 'mainnet' | 'previewnet';
  accountId: string;
  privateKey: string;
  topicId?: string;
  topicMemo?: string;
  maxFeeHbar?: number;
  auditLogDir?: string;
  mirrorNodeUrl?: string;
}

interface WitnessRecord {
  operation_id: string;
  transaction_id: string;
  topic_id: string;
  sequence_number: number;
  consensus_timestamp: string;
  payload_hash: string;
  witness_type: string;
  security_mode: string;
  submitted_at: string;
  latency_ms: number;
  fee_hbar: number;
  status: 'SUCCESS' | 'PENDING' | 'FAILED';
}

export class HederaWitnessingService implements IHederaWitnessingService {
  private client: Client;
  private operatorKey: PrivateKey;
  private operatorId: AccountId;
  private topicId: TopicId | null = null;
  private config: HederaWitnessingConfig;
  private auditLogPath: string;
  private initialized: boolean = false;
  private sequenceCounter: number = 0;
  private mirrorBaseUrl: string;

  private witnessCache: Map<string, WitnessRecord> = new Map();
  private static readonly MAX_CACHE_SIZE = 10_000;

  private verifyCache: Map<string, { result: boolean; cachedAt: number }> = new Map();
  private static readonly VERIFY_CACHE_TTL_MS = 300_000;
  private static readonly MAX_VERIFY_CACHE_SIZE = 5_000;

  constructor(config: HederaWitnessingConfig) {
    this.config = config;
    this.operatorId = AccountId.fromString(config.accountId);
    this.operatorKey = HederaWitnessingService.parsePrivateKey(config.privateKey);

    switch (config.network) {
      case 'mainnet':
        this.client = Client.forMainnet();
        this.mirrorBaseUrl = config.mirrorNodeUrl || 'https://mainnet.mirrornode.hedera.com';
        break;
      case 'previewnet':
        this.client = Client.forPreviewnet();
        this.mirrorBaseUrl = config.mirrorNodeUrl || 'https://previewnet.mirrornode.hedera.com';
        break;
      case 'testnet':
      default:
        this.client = Client.forTestnet();
        this.mirrorBaseUrl = config.mirrorNodeUrl || 'https://testnet.mirrornode.hedera.com';
        break;
    }

    this.client.setOperator(this.operatorId, this.operatorKey);
    this.client.setDefaultMaxTransactionFee(new Hbar(config.maxFeeHbar || 2));

    const logDir = config.auditLogDir || 'server/crypto/tsa-keys';
    if (!fs.existsSync(logDir)) fs.mkdirSync(logDir, { recursive: true });
    this.auditLogPath = path.join(logDir, 'hedera-witness-audit.jsonl');
  }

  async initialize(): Promise<{
    topicId: string;
    network: string;
    operatorAccount: string;
    topicCreated: boolean;
  }> {
    let topicCreated = false;

    if (this.config.topicId) {
      this.topicId = TopicId.fromString(this.config.topicId);

      try {
        const info = await new TopicInfoQuery()
          .setTopicId(this.topicId)
          .execute(this.client);

        this.sequenceCounter = Number(info.sequenceNumber) || 0;
        console.log(
          `[hedera] Connected to existing topic ${this.topicId.toString()} ` +
          `(${this.sequenceCounter} messages, memo: "${info.topicMemo}")`
        );
      } catch (error) {
        throw new Error(
          `Failed to query topic ${this.config.topicId}: ${(error as Error).message}. ` +
          `Verify the topic exists on ${this.config.network} and the operator has access.`
        );
      }
    } else {
      const memo = this.config.topicMemo ||
        `PlenumNET Witness Log — ${this.config.network} — ${new Date().toISOString().slice(0, 10)}`;

      const txResponse = await new TopicCreateTransaction()
        .setTopicMemo(memo)
        .setSubmitKey(this.operatorKey.publicKey)
        .setAdminKey(this.operatorKey.publicKey)
        .execute(this.client);

      const receipt = await txResponse.getReceipt(this.client);

      if (!receipt.topicId) {
        throw new Error('Topic creation succeeded but no topic ID returned');
      }

      this.topicId = receipt.topicId;
      topicCreated = true;
      console.log(
        `[hedera] Created new topic ${this.topicId.toString()} on ${this.config.network} ` +
        `(memo: "${memo}")`
      );

      const topicFile = path.join(
        this.config.auditLogDir || 'server/crypto/tsa-keys',
        'hedera-topic-id.txt',
      );
      fs.writeFileSync(topicFile, this.topicId.toString(), 'utf8');
      console.log(`[hedera] Topic ID persisted to ${topicFile}`);
    }

    this.initialized = true;

    return {
      topicId: this.topicId.toString(),
      network: this.config.network,
      operatorAccount: this.config.accountId,
      topicCreated,
    };
  }

  async submitWitness(request: HederaWitnessRequest): Promise<HederaWitnessResponse> {
    if (!this.initialized || !this.topicId) {
      throw new Error('HederaWitnessingService not initialized — call initialize() first');
    }

    const submittedAt = new Date();

    const witnessMessage = JSON.stringify({
      v: 1,
      op: request.operation_id,
      t: request.witness_type,
      h: request.payload.hash,
      alg: request.payload.hash_algorithm,
      m: request.metadata.ternary_context?.security_mode,
      k: request.metadata.kernel_op_id,
      b: request.metadata.salvi_batch_ref,
      ts: submittedAt.toISOString(),
    });

    const messageBytes = Buffer.from(witnessMessage, 'utf8');

    if (messageBytes.length > 6144) {
      throw new Error(
        `Witness message too large (${messageBytes.length} bytes). ` +
        `Max 6144 bytes (6 chunks × 1024). Consider MERKLE_ROOT_BATCH instead of SINGLE_HASH.`
      );
    }

    try {
      const submitTx = new TopicMessageSubmitTransaction()
        .setTopicId(this.topicId)
        .setMessage(messageBytes)
        .setMaxChunks(6);

      const txResponse = await submitTx.execute(this.client);
      const receipt = await txResponse.getReceipt(this.client);

      const consensusAt = new Date();
      const latencyMs = consensusAt.getTime() - submittedAt.getTime();

      const txIdStr = txResponse.transactionId.toString();
      const seqNum = receipt.topicSequenceNumber
        ? Number(receipt.topicSequenceNumber)
        : ++this.sequenceCounter;
      const runningHash = receipt.topicRunningHash
        ? Buffer.from(receipt.topicRunningHash).toString('hex')
        : '';

      const consensusTimestamp = consensusAt.toISOString();

      const transaction: HederaTransactionDetails = {
        id: txIdStr,
        status: receipt.status.toString() === 'SUCCESS' ? 'SUCCESS' : 'FAILED',
        consensus_timestamp: consensusTimestamp,
        topic_id: this.topicId.toString(),
        sequence_number: seqNum,
        running_hash: runningHash,
        chunk_info: {
          total: messageBytes.length > 1024 ? Math.ceil(messageBytes.length / 1024) : 1,
          number: 1,
          initial_transaction_id: txIdStr,
        },
      };

      const estimatedFeeHbar = 0.0001 * transaction.chunk_info.total;
      const costs: HederaCosts = {
        fee_hbar: estimatedFeeHbar,
        fee_usd: estimatedFeeHbar * 0.05,
        exchange_rate: 0.05,
      };

      const verification: HederaVerification = {
        verifiable: true,
        proof_available: true,
        query_endpoints: [
          `${this.mirrorBaseUrl}/api/v1/topics/${this.topicId.toString()}/messages/${seqNum}`,
          `${this.mirrorBaseUrl}/api/v1/transactions/${txIdStr}`,
        ],
      };

      const timing: HederaTiming = {
        submitted_at: submittedAt.toISOString(),
        consensus_at: consensusTimestamp,
        latency_ns: latencyMs * 1_000_000,
      };

      const response: HederaWitnessResponse = {
        success: transaction.status === 'SUCCESS',
        transaction,
        costs,
        verification,
        timing,
      };

      const record: WitnessRecord = {
        operation_id: request.operation_id,
        transaction_id: txIdStr,
        topic_id: this.topicId.toString(),
        sequence_number: seqNum,
        consensus_timestamp: consensusTimestamp,
        payload_hash: request.payload.hash,
        witness_type: request.witness_type,
        security_mode: request.metadata.ternary_context?.security_mode || 'unknown',
        submitted_at: submittedAt.toISOString(),
        latency_ms: latencyMs,
        fee_hbar: estimatedFeeHbar,
        status: transaction.status,
      };
      this.persistRecord(record);
      this.cacheRecord(txIdStr, record);

      return response;

    } catch (error) {
      const failedRecord: WitnessRecord = {
        operation_id: request.operation_id,
        transaction_id: 'FAILED',
        topic_id: this.topicId.toString(),
        sequence_number: -1,
        consensus_timestamp: '',
        payload_hash: request.payload.hash,
        witness_type: request.witness_type,
        security_mode: request.metadata.ternary_context?.security_mode || 'unknown',
        submitted_at: submittedAt.toISOString(),
        latency_ms: Date.now() - submittedAt.getTime(),
        fee_hbar: 0,
        status: 'FAILED',
      };
      this.persistRecord(failedRecord);

      throw new Error(`HCS submission failed: ${(error as Error).message}`);
    }
  }

  async getWitnessStatus(transactionId: string): Promise<HederaWitnessResponse | null> {
    const cached = this.witnessCache.get(transactionId);
    if (!cached) return null;

    return {
      success: cached.status === 'SUCCESS',
      transaction: {
        id: cached.transaction_id,
        status: cached.status,
        consensus_timestamp: cached.consensus_timestamp,
        topic_id: cached.topic_id,
        sequence_number: cached.sequence_number,
        running_hash: '',
        chunk_info: { total: 1, number: 1, initial_transaction_id: cached.transaction_id },
      },
      costs: {
        fee_hbar: cached.fee_hbar,
        fee_usd: cached.fee_hbar * 0.05,
        exchange_rate: 0.05,
      },
      verification: {
        verifiable: cached.status === 'SUCCESS',
        proof_available: cached.status === 'SUCCESS',
        query_endpoints: cached.status === 'SUCCESS' ? [
          `${this.mirrorBaseUrl}/api/v1/topics/${cached.topic_id}/messages/${cached.sequence_number}`,
        ] : [],
      },
      timing: {
        submitted_at: cached.submitted_at,
        consensus_at: cached.consensus_timestamp,
        latency_ns: cached.latency_ms * 1_000_000,
      },
    };
  }

  async verifyWitness(topicId: string, sequenceNumber: number): Promise<boolean> {
    const cacheKey = `${topicId}:${sequenceNumber}`;
    const cached = this.verifyCache.get(cacheKey);
    if (cached && (Date.now() - cached.cachedAt) < HederaWitnessingService.VERIFY_CACHE_TTL_MS) {
      return cached.result;
    }

    const url = `${this.mirrorBaseUrl}/api/v1/topics/${topicId}/messages/${sequenceNumber}`;

    try {
      const response = await fetch(url);
      if (!response.ok) {
        this.cacheVerifyResult(cacheKey, false);
        return false;
      }

      const data = await response.json() as {
        consensus_timestamp?: string;
        topic_id?: string;
        sequence_number?: number;
        message?: string;
      };

      if (!data.consensus_timestamp || !data.message) {
        this.cacheVerifyResult(cacheKey, false);
        return false;
      }

      const decoded = Buffer.from(data.message, 'base64').toString('utf8');
      try {
        const parsed = JSON.parse(decoded);
        const result = parsed.v === 1 && typeof parsed.h === 'string' && typeof parsed.op === 'string';
        this.cacheVerifyResult(cacheKey, result);
        return result;
      } catch {
        this.cacheVerifyResult(cacheKey, false);
        return false;
      }
    } catch {
      return false;
    }
  }

  private cacheVerifyResult(key: string, result: boolean): void {
    if (this.verifyCache.size >= HederaWitnessingService.MAX_VERIFY_CACHE_SIZE) {
      const oldest = this.verifyCache.keys().next().value;
      if (oldest !== undefined) this.verifyCache.delete(oldest);
    }
    this.verifyCache.set(key, { result, cachedAt: Date.now() });
  }

  async getHealth(): Promise<{
    status: 'healthy' | 'degraded' | 'offline';
    network: string;
    topic_id: string | null;
    messages_submitted: number;
    operator_account: string;
    initialized: boolean;
  }> {
    let status: 'healthy' | 'degraded' | 'offline' = 'offline';

    if (this.initialized && this.topicId) {
      try {
        const info = await new TopicInfoQuery()
          .setTopicId(this.topicId)
          .execute(this.client);
        status = 'healthy';
        this.sequenceCounter = Number(info.sequenceNumber) || this.sequenceCounter;
      } catch {
        status = 'degraded';
      }
    }

    return {
      status,
      network: this.config.network,
      topic_id: this.topicId?.toString() || null,
      messages_submitted: this.sequenceCounter,
      operator_account: this.config.accountId,
      initialized: this.initialized,
    };
  }

  async getTopicInfo(): Promise<{
    topic_id: string;
    memo: string;
    sequence_number: number;
    running_hash: string;
    expiration_time: string;
  } | null> {
    if (!this.topicId) return null;

    try {
      const info = await new TopicInfoQuery()
        .setTopicId(this.topicId)
        .execute(this.client);

      return {
        topic_id: this.topicId.toString(),
        memo: info.topicMemo || '',
        sequence_number: Number(info.sequenceNumber) || 0,
        running_hash: info.runningHash ? Buffer.from(info.runningHash).toString('hex') : '',
        expiration_time: info.expirationTime?.toDate()?.toISOString() || '',
      };
    } catch {
      return null;
    }
  }

  getStats(): {
    total_witnesses: number;
    successful: number;
    failed: number;
    cache_size: number;
    topic_id: string | null;
    network: string;
    audit_log_path: string;
  } {
    let successful = 0;
    let failed = 0;
    this.witnessCache.forEach(r => {
      if (r.status === 'SUCCESS') successful++;
      else if (r.status === 'FAILED') failed++;
    });

    return {
      total_witnesses: this.witnessCache.size,
      successful,
      failed,
      cache_size: this.witnessCache.size,
      topic_id: this.topicId?.toString() || null,
      network: this.config.network,
      audit_log_path: this.auditLogPath,
    };
  }

  getTopicId(): string | null {
    return this.topicId?.toString() || null;
  }

  isInitialized(): boolean {
    return this.initialized;
  }

  private persistRecord(record: WitnessRecord): void {
    try {
      fs.appendFileSync(this.auditLogPath, JSON.stringify(record) + '\n');
    } catch (error) {
      console.error(`[hedera] Failed to persist audit record: ${(error as Error).message}`);
    }
  }

  private cacheRecord(txId: string, record: WitnessRecord): void {
    this.witnessCache.set(txId, record);
    if (this.witnessCache.size > HederaWitnessingService.MAX_CACHE_SIZE) {
      const firstKey = this.witnessCache.keys().next().value;
      if (firstKey) this.witnessCache.delete(firstKey);
    }
  }

  private static parsePrivateKey(keyStr: string): PrivateKey {
    const strategies: Array<[string, () => PrivateKey]> = [
      ['ECDSA', () => PrivateKey.fromStringECDSA(keyStr)],
      ['ED25519', () => PrivateKey.fromStringED25519(keyStr)],
      ['DER', () => PrivateKey.fromStringDer(keyStr)],
    ];

    for (const [label, parse] of strategies) {
      try {
        const key = parse();
        console.log(`[hedera] Private key parsed as ${label}`);
        return key;
      } catch {
        continue;
      }
    }

    return PrivateKey.fromString(keyStr);
  }

  close(): void {
    try {
      this.client.close();
      console.log('[hedera] Client connection closed');
    } catch {
    }
  }
}

export function createHederaConfig(): HederaWitnessingConfig | null {
  const accountId = process.env.HEDERA_ACCOUNT_ID;
  const privateKey = process.env.HEDERA_PRIVATE_KEY;

  if (!accountId || !privateKey) {
    return null;
  }

  const persistedTopicPath = path.join(
    process.env.HEDERA_AUDIT_DIR || 'server/crypto/tsa-keys',
    'hedera-topic-id.txt',
  );
  let topicId = process.env.HEDERA_TOPIC_ID;
  if (!topicId && fs.existsSync(persistedTopicPath)) {
    topicId = fs.readFileSync(persistedTopicPath, 'utf8').trim();
    console.log(`[hedera] Loaded persisted topic ID: ${topicId}`);
  }

  return {
    network: (process.env.HEDERA_NETWORK as 'testnet' | 'mainnet' | 'previewnet') || 'testnet',
    accountId,
    privateKey,
    topicId: topicId || undefined,
    topicMemo: process.env.HEDERA_TOPIC_MEMO,
    maxFeeHbar: process.env.HEDERA_MAX_FEE_HBAR ? parseFloat(process.env.HEDERA_MAX_FEE_HBAR) : 2,
    auditLogDir: process.env.HEDERA_AUDIT_DIR || 'server/crypto/tsa-keys',
    mirrorNodeUrl: process.env.HEDERA_MIRROR_URL,
  };
}
