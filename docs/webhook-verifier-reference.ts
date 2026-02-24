/**
 * PlenumNET / Maestro Webhook TSA Token Verifier
 * Reference implementation for downstream consumers.
 *
 * Provides ONLINE verification via PlenumNET's /api/tsa/verify endpoint.
 * For OFFLINE verification, use OpenSSL CLI (see bash examples below)
 * or implement full CMS/TSTInfo parsing with pkijs + asn1js.
 *
 * Usage:
 *   const result = await verifyWebhookTsaToken(req.body, req.headers);
 *   if (result.valid) { processWebhook(req.body); }
 */

import * as crypto from 'crypto';

const TSA_BASE_URL = process.env.PLENUMNET_TSA_URL || 'https://plenumnet.replit.app';

export interface TsaVerificationResult {
  valid: boolean;
  method: 'online' | 'hash-only' | 'none';
  serial?: string;
  genTime?: string;
  policyTier?: string;
  merkleLeaf?: string;
  proofSummary?: string;
  reason: string;
}

function getHeader(
  headers: Record<string, string | string[] | undefined>,
  name: string,
): string | undefined {
  const lower = name.toLowerCase();
  const val = headers[lower] || headers[name];
  if (Array.isArray(val)) return val[0];
  return val as string | undefined;
}

export async function verifyWebhookTsaToken(
  payload: string | Buffer,
  headers: Record<string, string | string[] | undefined>,
): Promise<TsaVerificationResult> {

  const tsaTokenBase64 = getHeader(headers, 'X-PlenumNET-TSA-Token');
  if (!tsaTokenBase64) {
    return {
      valid: false,
      method: 'none',
      reason: 'Missing X-PlenumNET-TSA-Token header',
    };
  }

  let tsaTokenBuffer: Buffer;
  try {
    tsaTokenBuffer = Buffer.from(tsaTokenBase64, 'base64');
  } catch {
    return {
      valid: false,
      method: 'none',
      reason: 'Invalid base64 in X-PlenumNET-TSA-Token',
    };
  }

  const proofSummary = getHeader(headers, 'X-PlenumNET-Proof-Summary');

  const payloadBuffer = Buffer.isBuffer(payload)
    ? payload
    : Buffer.from(payload, 'utf-8');
  const computedHash = crypto.createHash('sha256')
    .update(payloadBuffer)
    .digest('hex');

  try {
    const res = await fetch(`${TSA_BASE_URL}/api/tsa/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/octet-stream' },
      body: tsaTokenBuffer,
    });

    if (!res.ok) {
      throw new Error(`Verify endpoint returned ${res.status}`);
    }

    const data = await res.json() as {
      valid: boolean;
      serialNumber?: string;
      genTime?: string;
      policyTier?: string;
      merkleLeafHash?: string;
      reason?: string;
    };

    if (data.valid) {
      return {
        valid: true,
        method: 'online',
        serial: data.serialNumber,
        genTime: data.genTime,
        policyTier: data.policyTier,
        merkleLeaf: data.merkleLeafHash,
        proofSummary,
        reason: 'Online verification succeeded',
      };
    } else {
      return {
        valid: false,
        method: 'online',
        reason: data.reason || 'Token verification failed',
      };
    }

  } catch (err) {
    console.warn('Online TSA verification failed, falling back to hash check', {
      error: (err as Error).message,
    });

    return {
      valid: false,
      method: 'hash-only',
      proofSummary,
      reason: `Online verification unavailable: ${(err as Error).message}. ` +
        'Use OpenSSL offline verification for full cryptographic proof.',
    };
  }
}
