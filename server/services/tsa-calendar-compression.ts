/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * TSA Calendar Context — Ternary Compression Integration
 *
 * Wraps the existing compression pipeline from server/ternary.ts
 * with a TERN-format envelope for embedding in X.509v3 extensions.
 *
 * @module server/services/tsa-calendar-compression
 */

import * as zlib from 'zlib';
import {
  compressData,
  decompressData,
} from '../ternary';

import type { CompressionResult } from '../ternary';

const TERN_MAGIC = Buffer.from('TERN', 'ascii');
const ENVELOPE_HEADER_SIZE = 18;
const COMPRESSION_THRESHOLD = 256;
const MAX_CALENDAR_JSON_SIZE = 65_536;
const COMPRESSION_TIMEOUT_MS = 5;

const PIPELINE = {
  TERN_FULL:         0x01,
  TERN_DEFLATE_ONLY: 0x02,
  TERN_PHASE:        0x03,
  TERN_NONE:         0xFF,
} as const;

const ENVELOPE_VERSION = 0x02;

export interface CalendarCompressionContext {
  v: number;
  utc: string;
  jdn: number;
  sed: number;
  tier: string;
  src: { policy: string[]; request: string[] };
  cal: CalendarCompressionEntry[];
  [key: string]: unknown;
}

interface CalendarCompressionEntry {
  sys: string;
  d: string;
  y?: number;
  m?: number | string;
  day?: number;
  era?: string;
  [key: string]: unknown;
}

interface CompressionEnvelope {
  magic: Buffer;
  envelopeVersion: number;
  pipelineId: number;
  originalSize: number;
  compressedSize: number;
  checksum: number;
  payload: Buffer;
}

export interface CalendarCompressionResult {
  originalSize: number;
  ternarySize: number;
  compressedSize: number;
  compressionRatio: number;
  pipelineId: number;
  envelopeOverhead: number;
  effectiveRatio: number;
}

export interface CalendarCompressionPolicy {
  enabled: boolean;
  pipelineId: number;
  threshold: number;
}

export const POLICY_COMPRESSION_CONFIG: Record<string, CalendarCompressionPolicy> = {
  'DEFAULT':    { enabled: true,  pipelineId: PIPELINE.TERN_FULL, threshold: 256 },
  'COMPLY':     { enabled: true,  pipelineId: PIPELINE.TERN_FULL, threshold: 256 },
  'FORENSICS':  { enabled: true,  pipelineId: PIPELINE.TERN_FULL, threshold: 256 },
  'SENTINEL':   { enabled: false, pipelineId: PIPELINE.TERN_NONE, threshold: 0   },
  'SECURE':     { enabled: false, pipelineId: PIPELINE.TERN_NONE, threshold: 0   },
};

const compressionMetrics = {
  compressed: 0,
  fallbacks: { below_threshold: 0, error: 0, no_gain: 0, timeout: 0, disabled: 0 },
  decompressionErrors: { unknown_format: 0, unknown_pipeline: 0, size_mismatch: 0, crc_mismatch: 0, parse_error: 0, bomb_guard: 0 },
  totalCompressed: 0,
  totalOriginal: 0,
  durations: [] as number[],
};

export function getCompressionMetrics() {
  const avgDuration = compressionMetrics.durations.length > 0
    ? compressionMetrics.durations.reduce((a, b) => a + b, 0) / compressionMetrics.durations.length
    : 0;
  const avgRatio = compressionMetrics.totalOriginal > 0
    ? compressionMetrics.totalCompressed / compressionMetrics.totalOriginal
    : 0;
  return {
    compressed: compressionMetrics.compressed,
    fallbacks: { ...compressionMetrics.fallbacks },
    decompressionErrors: { ...compressionMetrics.decompressionErrors },
    avgDurationMs: Math.round(avgDuration * 1000) / 1000,
    avgEffectiveRatio: Math.round(avgRatio * 1000) / 1000,
  };
}

const CRC32C_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let i = 0; i < 256; i++) {
    let crc = i;
    for (let j = 0; j < 8; j++) {
      crc = (crc & 1) ? (0x82F63B78 ^ (crc >>> 1)) : (crc >>> 1);
    }
    table[i] = crc;
  }
  return table;
})();

function crc32c(data: Buffer): number {
  let crc = 0xFFFFFFFF;
  for (let i = 0; i < data.length; i++) {
    crc = CRC32C_TABLE[(crc ^ data[i]) & 0xFF] ^ (crc >>> 8);
  }
  return (crc ^ 0xFFFFFFFF) >>> 0;
}

function pipelineName(id: number): string {
  switch (id) {
    case PIPELINE.TERN_FULL:         return 'tern_full';
    case PIPELINE.TERN_DEFLATE_ONLY: return 'tern_deflate_only';
    case PIPELINE.TERN_PHASE:        return 'tern_phase';
    case PIPELINE.TERN_NONE:         return 'tern_none';
    default:                         return `unknown_0x${id.toString(16)}`;
  }
}

function isCompressionDisabledByEnv(): boolean {
  return process.env.TSA_CALENDAR_COMPRESSION === 'disabled';
}

function getEnvPipelineOverride(): number | undefined {
  const v = process.env.TSA_CALENDAR_COMPRESSION_PIPELINE;
  if (v === 'deflate_only') return PIPELINE.TERN_DEFLATE_ONLY;
  if (v === 'none') return PIPELINE.TERN_NONE;
  return undefined;
}

function getEnvThreshold(): number {
  const v = process.env.TSA_CALENDAR_COMPRESSION_THRESHOLD;
  if (v) {
    const n = parseInt(v, 10);
    if (!isNaN(n) && n >= 0) return n;
  }
  return COMPRESSION_THRESHOLD;
}

function getEnvTimeout(): number {
  const v = process.env.TSA_CALENDAR_COMPRESSION_TIMEOUT_MS;
  if (v) {
    const n = parseFloat(v);
    if (!isNaN(n) && n > 0) return n;
  }
  return COMPRESSION_TIMEOUT_MS;
}

export function buildCalendarExtension(
  context: CalendarCompressionContext,
  pipelineOverride?: number,
): { buffer: Buffer; compressed: boolean; metrics: CalendarCompressionResult | null } {
  const json = JSON.stringify(context);
  const jsonBuf = Buffer.from(json, 'utf-8');

  if (isCompressionDisabledByEnv()) {
    compressionMetrics.fallbacks.disabled++;
    return { buffer: jsonBuf, compressed: false, metrics: null };
  }

  const tier = context.tier || 'DEFAULT';
  const policyConfig = POLICY_COMPRESSION_CONFIG[tier];
  if (policyConfig && !policyConfig.enabled) {
    compressionMetrics.fallbacks.disabled++;
    return { buffer: jsonBuf, compressed: false, metrics: null };
  }

  const threshold = getEnvThreshold();
  if (jsonBuf.length < threshold) {
    compressionMetrics.fallbacks.below_threshold++;
    return { buffer: jsonBuf, compressed: false, metrics: null };
  }

  const pipelineId = pipelineOverride ?? getEnvPipelineOverride() ?? policyConfig?.pipelineId ?? PIPELINE.TERN_FULL;

  if (pipelineId === PIPELINE.TERN_NONE) {
    return { buffer: jsonBuf, compressed: false, metrics: null };
  }

  let compressedPayload: Buffer;
  let pipelineResult: CompressionResult;

  try {
    const startTime = performance.now();

    if (pipelineId === PIPELINE.TERN_FULL) {
      pipelineResult = compressData(json);
      const rawCompressed = Buffer.from(pipelineResult.compressedData, 'base64');
      compressedPayload = rawCompressed.subarray(4);
    } else if (pipelineId === PIPELINE.TERN_DEFLATE_ONLY) {
      const deflated = zlib.deflateSync(jsonBuf, { level: 9 });
      compressedPayload = deflated;
      pipelineResult = {
        originalData: json,
        compressedData: '',
        originalSize: jsonBuf.length,
        ternarySize: 0,
        compressedSize: deflated.length,
        compressionRatio: ((jsonBuf.length - deflated.length) / jsonBuf.length) * 100,
      };
    } else {
      compressionMetrics.fallbacks.error++;
      return { buffer: jsonBuf, compressed: false, metrics: null };
    }

    const elapsed = performance.now() - startTime;
    compressionMetrics.durations.push(elapsed);
    if (compressionMetrics.durations.length > 1000) {
      compressionMetrics.durations = compressionMetrics.durations.slice(-500);
    }

    const timeout = getEnvTimeout();
    if (elapsed > timeout) {
      compressionMetrics.fallbacks.timeout++;
      console.warn('Calendar compression exceeded timeout', {
        elapsed: elapsed.toFixed(3),
        threshold: timeout,
        tier,
      });
      return { buffer: jsonBuf, compressed: false, metrics: null };
    }

  } catch (err) {
    compressionMetrics.fallbacks.error++;
    console.warn('Calendar compression failed, emitting raw JSON', {
      error: (err as Error).message,
      tier,
    });
    return { buffer: jsonBuf, compressed: false, metrics: null };
  }

  if (compressedPayload.length + ENVELOPE_HEADER_SIZE >= jsonBuf.length) {
    compressionMetrics.fallbacks.no_gain++;
    return { buffer: jsonBuf, compressed: false, metrics: null };
  }

  const checksum = crc32c(jsonBuf);
  const envelope = buildEnvelope({
    magic: TERN_MAGIC,
    envelopeVersion: ENVELOPE_VERSION,
    pipelineId,
    originalSize: jsonBuf.length,
    compressedSize: compressedPayload.length,
    checksum,
    payload: compressedPayload,
  });

  const metrics: CalendarCompressionResult = {
    originalSize: pipelineResult.originalSize,
    ternarySize: pipelineResult.ternarySize,
    compressedSize: pipelineResult.compressedSize,
    compressionRatio: pipelineResult.compressionRatio,
    pipelineId,
    envelopeOverhead: ENVELOPE_HEADER_SIZE,
    effectiveRatio: envelope.length / jsonBuf.length,
  };

  compressionMetrics.compressed++;
  compressionMetrics.totalCompressed += envelope.length;
  compressionMetrics.totalOriginal += jsonBuf.length;

  return { buffer: envelope, compressed: true, metrics };
}

function buildEnvelope(env: CompressionEnvelope): Buffer {
  const header = Buffer.alloc(ENVELOPE_HEADER_SIZE);
  env.magic.copy(header, 0);
  header.writeUInt8(env.envelopeVersion, 4);
  header.writeUInt8(env.pipelineId, 5);
  header.writeUInt32BE(env.originalSize, 6);
  header.writeUInt32BE(env.compressedSize, 10);
  header.writeUInt32BE(env.checksum, 14);
  return Buffer.concat([header, env.payload]);
}

export function parseCalendarExtension(octetString: Buffer): CalendarCompressionContext | null {
  if (octetString.length === 0) return null;

  try {
    if (
      octetString.length >= ENVELOPE_HEADER_SIZE &&
      octetString.subarray(0, 4).equals(TERN_MAGIC)
    ) {
      return decompressEnvelope(octetString);
    }

    if (octetString[0] === 0x7B) {
      const json = octetString.toString('utf-8');
      return validateCalendarContext(JSON.parse(json));
    }

    console.warn('Unknown CalendarContext format', {
      firstBytes: octetString.subarray(0, 4).toString('hex'),
    });
    compressionMetrics.decompressionErrors.unknown_format++;
    return null;

  } catch (err) {
    console.warn('CalendarContext parse failed', { error: (err as Error).message });
    compressionMetrics.decompressionErrors.parse_error++;
    return null;
  }
}

function decompressEnvelope(data: Buffer): CalendarCompressionContext | null {
  if (data.length < ENVELOPE_HEADER_SIZE) {
    throw new Error(`Envelope too short: ${data.length} < ${ENVELOPE_HEADER_SIZE}`);
  }

  const magic          = data.subarray(0, 4);
  const envelopeVer    = data.readUInt8(4);
  const pipelineId     = data.readUInt8(5);
  const originalSize   = data.readUInt32BE(6);
  const compressedSize = data.readUInt32BE(10);
  const checksum       = data.readUInt32BE(14);
  const payload        = data.subarray(ENVELOPE_HEADER_SIZE);

  if (!magic.equals(TERN_MAGIC)) {
    throw new Error(`Invalid magic: expected TERN, got ${magic.toString('ascii')}`);
  }

  if (envelopeVer !== ENVELOPE_VERSION) {
    throw new Error(`Unsupported envelope version: 0x${envelopeVer.toString(16)}`);
  }

  if (payload.length !== compressedSize) {
    throw new Error(
      `Compressed size mismatch: header says ${compressedSize}, payload is ${payload.length}`
    );
  }

  if (originalSize > MAX_CALENDAR_JSON_SIZE) {
    compressionMetrics.decompressionErrors.bomb_guard++;
    throw new Error(
      `Original size ${originalSize} exceeds maximum ${MAX_CALENDAR_JSON_SIZE}`
    );
  }

  const startTime = performance.now();
  let decompressedJson: string;

  switch (pipelineId) {
    case PIPELINE.TERN_FULL: {
      const sizePrefixed = Buffer.alloc(4 + payload.length);
      sizePrefixed.writeUInt32BE(originalSize, 0);
      payload.copy(sizePrefixed, 4);
      const base64 = sizePrefixed.toString('base64');
      decompressedJson = decompressData(base64);
      break;
    }

    case PIPELINE.TERN_DEFLATE_ONLY: {
      const inflated = zlib.inflateSync(payload);
      decompressedJson = inflated.toString('utf-8');
      break;
    }

    case PIPELINE.TERN_NONE: {
      decompressedJson = payload.toString('utf-8');
      break;
    }

    default:
      compressionMetrics.decompressionErrors.unknown_pipeline++;
      throw new Error(`Unknown pipeline ID: 0x${pipelineId.toString(16)}`);
  }

  const decompressedBuf = Buffer.from(decompressedJson, 'utf-8');

  if (decompressedBuf.length !== originalSize) {
    compressionMetrics.decompressionErrors.size_mismatch++;
    throw new Error(
      `Decompressed size mismatch: expected ${originalSize}, got ${decompressedBuf.length}`
    );
  }

  const actualChecksum = crc32c(decompressedBuf);
  if (actualChecksum !== checksum) {
    compressionMetrics.decompressionErrors.crc_mismatch++;
    throw new Error(
      `CRC-32C mismatch: expected 0x${checksum.toString(16).padStart(8, '0')}, ` +
      `got 0x${actualChecksum.toString(16).padStart(8, '0')}`
    );
  }

  return validateCalendarContext(JSON.parse(decompressedJson));
}

function validateCalendarContext(obj: unknown): CalendarCompressionContext | null {
  if (!obj || typeof obj !== 'object') return null;
  const ctx = obj as Record<string, unknown>;

  if (typeof ctx.v !== 'number') return null;
  if (typeof ctx.utc !== 'string') return null;
  if (typeof ctx.jdn !== 'number') return null;
  if (!Array.isArray(ctx.cal)) return null;

  for (const entry of ctx.cal as unknown[]) {
    if (!entry || typeof entry !== 'object') return null;
    const e = entry as Record<string, unknown>;
    if (typeof e.sys !== 'string') return null;
    if (typeof e.d !== 'string') return null;
  }

  return ctx as unknown as CalendarCompressionContext;
}

export function serializeForExtensionCompressed(
  context: { calendarJson: string; tier: string },
): { payload: Buffer; compressed: boolean; metrics: CalendarCompressionResult | null } {
  const parsed = JSON.parse(context.calendarJson);
  if (!parsed || !parsed.cal || !Array.isArray(parsed.cal)) {
    return { payload: Buffer.from(context.calendarJson, 'utf-8'), compressed: false, metrics: null };
  }
  parsed.tier = context.tier;
  return buildCalendarExtension(parsed);
}

export { PIPELINE, ENVELOPE_VERSION, ENVELOPE_HEADER_SIZE, TERN_MAGIC };
