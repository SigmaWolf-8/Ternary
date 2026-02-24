/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * NOTIFICATION SERVICE — TSA INTEGRATION (Phase 2/3)
 *
 * Outbound notification service with RFC 3161 cryptographic proof headers.
 * Phase 2 (default): Both legacy + TSA headers (zero-risk migration).
 * Phase 3: TSA token only; legacy headers removed.
 *
 * Control via env: TSA_NOTIFICATION_PHASE=2 (default) | 3
 */

import * as crypto from 'crypto';
import * as prom from 'prom-client';
import { TsaService, type JsonTimestampResponse, type HptpClient, type TldsaClient } from './tsa-service';
import { resolveTsaPolicy, resolveTierName } from './notification-tsa-policy';

const parsedPhase = parseInt(process.env.TSA_NOTIFICATION_PHASE || '2', 10);
const TSA_PHASE = (parsedPhase === 2 || parsedPhase === 3) ? parsedPhase : 2;

export const tsaMetricsRegistry = new prom.Registry();
prom.collectDefaultMetrics({ register: tsaMetricsRegistry });

const tsaRequestCounter = new prom.Counter({
  name: 'plenumnet_notification_tsa_requests_total',
  help: 'Total TSA token requests from notification service',
  labelNames: ['phase', 'channel', 'policy_tier', 'status'] as const,
  registers: [tsaMetricsRegistry],
});

const tsaDurationHistogram = new prom.Histogram({
  name: 'plenumnet_notification_tsa_request_duration_seconds',
  help: 'Duration of TSA token acquisition (seconds)',
  labelNames: ['phase', 'channel', 'policy_tier'] as const,
  buckets: [0.01, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
  registers: [tsaMetricsRegistry],
});

const tsaPhaseGauge = new prom.Gauge({
  name: 'plenumnet_notification_tsa_phase',
  help: 'Current TSA integration phase (2=dual, 3=tsa-only)',
  registers: [tsaMetricsRegistry],
});

const tsaTokenSizeHistogram = new prom.Histogram({
  name: 'plenumnet_notification_tsa_token_size_bytes',
  help: 'Base64-encoded TSA token size in characters (for header budget monitoring)',
  labelNames: ['policy_tier'] as const,
  buckets: [512, 1024, 2048, 3072, 4096, 6144, 8192],
  registers: [tsaMetricsRegistry],
});

tsaPhaseGauge.set(TSA_PHASE);

export interface NotificationServiceDeps {
  hptpClient: HptpClient;
  tldsaClient: TldsaClient;
  tsaService?: TsaService;
}

export interface NotificationResult {
  delivered: boolean;
  channel: string;
  proofMode: string;
  tsaSerial?: string;
  headers: Record<string, string>;
}

export class NotificationService {
  private tsaService: TsaService | null;
  private hptpClient: HptpClient;
  private tldsaClient: TldsaClient;

  constructor(deps: NotificationServiceDeps) {
    this.tsaService = deps.tsaService ?? null;
    this.hptpClient = deps.hptpClient;
    this.tldsaClient = deps.tldsaClient;
  }

  private async acquireTsaToken(
    payload: string | Buffer,
    channel: string,
    contentType?: string,
  ): Promise<JsonTimestampResponse | null> {
    if (!this.tsaService) {
      tsaRequestCounter.inc({
        phase: String(TSA_PHASE),
        channel,
        policy_tier: 'none',
        status: 'skipped_no_service',
      });
      return null;
    }

    const tierName = resolveTierName(channel, contentType);

    const endTimer = tsaDurationHistogram.startTimer({
      phase: String(TSA_PHASE),
      channel,
      policy_tier: tierName,
    });

    try {
      const payloadBuffer = Buffer.isBuffer(payload)
        ? payload
        : Buffer.from(payload, 'utf-8');
      const hash = crypto.createHash('sha256')
        .update(payloadBuffer)
        .digest('hex');

      const policyOid = resolveTsaPolicy(channel, contentType);

      const nonce = crypto.randomBytes(16).toString('hex');

      const response = await this.tsaService.processJsonRequest(
        {
          hash,
          algorithm: 'sha256',
          policy: policyOid,
          nonce,
          includeChain: false,
        },
        'notification-service',
      );

      endTimer();
      tsaRequestCounter.inc({
        phase: String(TSA_PHASE),
        channel,
        policy_tier: response.policyTier,
        status: 'success',
      });
      tsaTokenSizeHistogram.observe(
        { policy_tier: response.policyTier },
        response.token.length,
      );

      console.info('TSA token acquired', {
        channel,
        contentType,
        serial: response.serialNumber,
        tier: response.policyTier,
        merkleLeaf: response.merkleLeafHash.slice(0, 16) + '...',
        tokenChars: response.token.length,
        phase: TSA_PHASE,
      });

      return response;

    } catch (err) {
      endTimer();

      const status = (err as Error).message.includes('timeout')
        ? 'timeout'
        : 'error';

      tsaRequestCounter.inc({
        phase: String(TSA_PHASE),
        channel,
        policy_tier: tierName,
        status,
      });

      console.error('TSA token failed', {
        channel,
        contentType,
        error: (err as Error).message,
        phase: TSA_PHASE,
        fallback: TSA_PHASE <= 2 ? 'legacy' : 'none',
      });

      return null;
    }
  }

  private async buildProofHeaders(
    payload: string | Buffer,
    channel: string,
    contentType?: string,
  ): Promise<Record<string, string>> {
    const headers: Record<string, string> = {};

    const tsa = await this.acquireTsaToken(payload, channel, contentType);

    if (tsa) {
      headers['X-PlenumNET-TSA-Token'] = tsa.token;

      headers['X-PlenumNET-TSA-Serial'] = tsa.serialNumber;
      headers['X-PlenumNET-TSA-GenTime'] = tsa.genTime;
      headers['X-PlenumNET-TSA-Policy'] = tsa.policyTier;
      headers['X-PlenumNET-TSA-Merkle'] = tsa.merkleLeafHash;

      if (tsa.token.length > 4096) {
        console.warn('Large TST detected', {
          tokenChars: tsa.token.length,
          serial: tsa.serialNumber,
          tier: tsa.policyTier,
          note: 'Consider header folding per RFC 5322 §2.2.3',
        });
      }

      if (TSA_PHASE === 2) {
        const legacy = await this.buildLegacyHeaders(payload);
        Object.assign(headers, legacy);
        headers['X-PlenumNET-Proof-Mode'] = 'tsa+legacy';
      } else {
        headers['X-PlenumNET-Proof-Mode'] = 'tsa-only';
      }

    } else {
      if (TSA_PHASE <= 2) {
        const legacy = await this.buildLegacyHeaders(payload);
        Object.assign(headers, legacy);
        headers['X-PlenumNET-Proof-Mode'] = 'legacy-fallback';
      } else {
        headers['X-PlenumNET-Proof-Mode'] = 'none';
        console.warn('TSA unavailable in Phase 3 — notification sent without proof', {
          channel,
          contentType,
          phase: TSA_PHASE,
        });
      }
    }

    return headers;
  }

  private async buildLegacyHeaders(
    payload: string | Buffer,
  ): Promise<Record<string, string>> {
    const headers: Record<string, string> = {};

    try {
      const hptpResult = await this.hptpClient.getTimestamp();
      headers['X-PlenumNET-HPTP-Timestamp'] = hptpResult.timestamp;

      const payloadBuffer = Buffer.isBuffer(payload)
        ? payload
        : Buffer.from(payload, 'utf-8');
      const messageHash = crypto.createHash('sha3-256')
        .update(payloadBuffer)
        .digest('hex');
      headers['X-PlenumNET-Message-Hash'] = messageHash;

      try {
        const sig = await this.tldsaClient.sign(messageHash);
        headers['X-PlenumNET-TLDSA-Signature'] = sig.signature;
        headers['X-PlenumNET-TLDSA-KeyId'] = sig.publicKeyId;
      } catch {
        // TL-DSA failure is non-fatal
      }
    } catch (error) {
      console.error('Legacy header build failed', {
        error: (error as Error).message,
      });
    }

    return headers;
  }

  async sendEmail(
    to: string,
    subject: string,
    body: string,
    opts?: { contentType?: string },
  ): Promise<NotificationResult> {
    const headers = await this.buildProofHeaders(body, 'email', opts?.contentType);

    console.info('Email notification dispatched', {
      to,
      subject,
      proofMode: headers['X-PlenumNET-Proof-Mode'],
      tsaSerial: headers['X-PlenumNET-TSA-Serial'] || null,
    });

    return {
      delivered: true,
      channel: 'email',
      proofMode: headers['X-PlenumNET-Proof-Mode'] || 'none',
      tsaSerial: headers['X-PlenumNET-TSA-Serial'],
      headers,
    };
  }

  async sendWebhook(
    url: string,
    payload: object,
    opts?: { contentType?: string },
  ): Promise<NotificationResult> {
    const body = JSON.stringify(payload);
    const headers = await this.buildProofHeaders(body, 'webhook', opts?.contentType);

    console.info('Webhook notification dispatched', {
      url,
      proofMode: headers['X-PlenumNET-Proof-Mode'],
      tsaSerial: headers['X-PlenumNET-TSA-Serial'] || null,
    });

    return {
      delivered: true,
      channel: 'webhook',
      proofMode: headers['X-PlenumNET-Proof-Mode'] || 'none',
      tsaSerial: headers['X-PlenumNET-TSA-Serial'],
      headers,
    };
  }

  async sendSms(
    to: string,
    message: string,
    opts?: { contentType?: string },
  ): Promise<NotificationResult> {
    const proofHeaders = await this.buildProofHeaders(message, 'sms', opts?.contentType);

    console.info('SMS notification dispatched', {
      to,
      proofMode: proofHeaders['X-PlenumNET-Proof-Mode'],
      tsaSerial: proofHeaders['X-PlenumNET-TSA-Serial'] || null,
    });

    return {
      delivered: true,
      channel: 'sms',
      proofMode: proofHeaders['X-PlenumNET-Proof-Mode'] || 'none',
      tsaSerial: proofHeaders['X-PlenumNET-TSA-Serial'],
      headers: proofHeaders,
    };
  }

  async emitEvent(
    name: string,
    payload: object,
    opts?: { contentType?: string },
  ): Promise<NotificationResult> {
    const body = JSON.stringify(payload);
    const proofHeaders = await this.buildProofHeaders(body, 'event', opts?.contentType);

    console.info('Event notification dispatched', {
      eventName: name,
      proofMode: proofHeaders['X-PlenumNET-Proof-Mode'],
      tsaSerial: proofHeaders['X-PlenumNET-TSA-Serial'] || null,
    });

    return {
      delivered: true,
      channel: 'event',
      proofMode: proofHeaders['X-PlenumNET-Proof-Mode'] || 'none',
      tsaSerial: proofHeaders['X-PlenumNET-TSA-Serial'],
      headers: proofHeaders,
    };
  }

  async sendPush(
    userId: string,
    title: string,
    body: string,
    opts?: { contentType?: string },
  ): Promise<NotificationResult> {
    const proofHeaders = await this.buildProofHeaders(body, 'push', opts?.contentType);

    console.info('Push notification dispatched', {
      userId,
      title,
      proofMode: proofHeaders['X-PlenumNET-Proof-Mode'],
      tsaSerial: proofHeaders['X-PlenumNET-TSA-Serial'] || null,
    });

    return {
      delivered: true,
      channel: 'push',
      proofMode: proofHeaders['X-PlenumNET-Proof-Mode'] || 'none',
      tsaSerial: proofHeaders['X-PlenumNET-TSA-Serial'],
      headers: proofHeaders,
    };
  }

  getPhase(): number {
    return TSA_PHASE;
  }

  hasTsaService(): boolean {
    return this.tsaService !== null;
  }
}
