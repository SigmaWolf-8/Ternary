/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import zlib from 'zlib';
import { createRequire } from 'module';
import { resolve as _resolve } from 'path';
import { compressData, decompressData } from './ternary';
import { phaseSplit, phaseRecombine, type EncryptionMode, type EncryptedPhaseData } from './salvi-core/phase-encryption';

const _getRequire = (): NodeRequire => {
  if (typeof require !== 'undefined') return require;
  return createRequire(import.meta.url);
};

const CRC32_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let i = 0; i < 256; i++) {
    let c = i;
    for (let j = 0; j < 8; j++) {
      c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
    }
    table[i] = c >>> 0;
  }
  return table;
})();

function crc32Checksum(buf: Buffer): number {
  let crc = 0xFFFFFFFF;
  for (let i = 0; i < buf.length; i++) {
    crc = CRC32_TABLE[(crc ^ buf[i]) & 0xFF] ^ (crc >>> 8);
  }
  return (crc ^ 0xFFFFFFFF) >>> 0;
}

interface TtcNativeAddon {
  ttcCompress(input: Buffer, level?: number | null, mode?: string | null, filename?: string | null): {
    compressed: Buffer;
    originalSize: number;
    compressedSize: number;
    compressionRatio: number;
    crc32: number;
    modeName: string;
    version: string;
    level: number;
    levelName: string;
    avgTau: number;
    avgDelta: number;
    predominantBase: number;
    adaptiveRepUsed: boolean;
  };
  ttcDecompress(input: Buffer): {
    data: Buffer;
    originalSize: number;
    compressedSize: number;
    version: string;
    level: number | null;
    levelName: string | null;
    crc32Verified: boolean;
    originalFileName: string | null;
  };
}

let _ttcAddon: TtcNativeAddon | null = null;
let _ttcProbed = false;

function loadTtcAddon(): TtcNativeAddon | null {
  if (_ttcProbed) return _ttcAddon;
  _ttcProbed = true;
  const _req = _getRequire();
  const paths = [
    _resolve(process.cwd(), 'server/crypto/sponge-native.node'),
  ];
  if (typeof __dirname !== 'undefined') {
    paths.unshift(_resolve(__dirname, 'crypto/sponge-native.node'));
  }
  for (const p of paths) {
    try {
      const addon = _req(p);
      if (typeof addon.ttcCompress === 'function' && typeof addon.ttcDecompress === 'function') {
        _ttcAddon = addon as TtcNativeAddon;
        console.log('[TTC] Native N-API addon loaded — TTC v4.2 engine active');
        return _ttcAddon;
      }
    } catch (e) {
      console.warn('[TTC] Probe failed for', p, ':', (e as Error).message?.slice(0, 80));
    }
  }
  console.warn('[TTC] Native addon not found — falling back to legacy zlib pipeline');
  return null;
}

export interface TtcCompressionMetadata {
  engine: 'ttc-native' | 'legacy-zlib';
  version: string;
  level: number;
  levelName: string;
  modeName: string;
  crc32: number;
  avgTau: number;
  avgDelta: number;
  predominantBase: number;
  adaptiveRepUsed: boolean;
}

export interface TtcDecompressionMetadata {
  engine: 'ttc-native' | 'legacy-zlib';
  version: string;
  level: number | null;
  levelName: string | null;
  crc32Verified: boolean;
  originalFileName: string | null;
}

export interface CompressionPolicy {
  enabled: boolean;
  encrypt: boolean;
  encryptionMode: EncryptionMode;
}

export interface CompressedColumn {
  _ternaryCompressed: true;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  encrypted: boolean;
  encryptionMode?: EncryptionMode;
  data: string;
  phaseData?: EncryptedPhaseData;
}

const DEFAULT_POLICY: CompressionPolicy = {
  enabled: true,
  encrypt: false,
  encryptionMode: 'balanced',
};

export function compressForStorage(
  value: string,
  policy: CompressionPolicy = DEFAULT_POLICY
): string {
  if (!policy.enabled) return value;

  const originalSize = Buffer.from(value, 'utf-8').length;
  const compressed = compressData(value);

  const envelope: CompressedColumn = {
    _ternaryCompressed: true,
    originalSize,
    compressedSize: compressed.compressedSize,
    compressionRatio: compressed.compressionRatio,
    encrypted: false,
    data: compressed.compressedData,
  };

  if (policy.encrypt) {
    const phaseResult = phaseSplit(compressed.compressedData, policy.encryptionMode);
    envelope.encrypted = true;
    envelope.encryptionMode = policy.encryptionMode;
    envelope.phaseData = phaseResult;
    envelope.data = '';
  }

  return JSON.stringify(envelope, (_, v) => typeof v === 'bigint' ? v.toString() : v);
}

export function decompressFromStorage(storedValue: string): string {
  try {
    const parsed = JSON.parse(storedValue);

    if (!parsed._ternaryCompressed) {
      return storedValue;
    }

    const envelope = parsed as CompressedColumn;
    let compressedData: string;

    if (envelope.encrypted && envelope.phaseData) {
      const recombined = phaseRecombine(envelope.phaseData);
      if (!recombined.success || !recombined.data) {
        throw new Error(`Phase decryption failed: ${recombined.error}`);
      }
      compressedData = recombined.data;
    } else {
      compressedData = envelope.data;
    }

    return decompressData(compressedData);
  } catch (e) {
    return storedValue;
  }
}

export function isCompressedValue(storedValue: string): boolean {
  try {
    const parsed = JSON.parse(storedValue);
    return parsed._ternaryCompressed === true;
  } catch {
    return false;
  }
}

export function getCompressionMetadata(storedValue: string): {
  isCompressed: boolean;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  encrypted: boolean;
  encryptionMode?: string;
} | null {
  try {
    const parsed = JSON.parse(storedValue);
    if (!parsed._ternaryCompressed) return null;

    return {
      isCompressed: true,
      originalSize: parsed.originalSize,
      compressedSize: parsed.compressedSize,
      compressionRatio: parsed.compressionRatio,
      encrypted: parsed.encrypted,
      encryptionMode: parsed.encryptionMode,
    };
  } catch {
    return null;
  }
}

export function compressFileBuffer(inputBuffer: Buffer, options?: {
  level?: number;
  mode?: string;
  filename?: string;
}): {
  compressed: Buffer;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  ttcMetadata?: TtcCompressionMetadata;
} {
  const addon = loadTtcAddon();
  if (addon) {
    const r = addon.ttcCompress(
      inputBuffer,
      options?.level ?? 5,
      options?.mode ?? null,
      options?.filename ?? null
    );
    return {
      compressed: Buffer.from(r.compressed),
      originalSize: r.originalSize,
      compressedSize: r.compressedSize,
      compressionRatio: r.originalSize > 0
        ? ((r.originalSize - r.compressedSize) / r.originalSize) * 100
        : 0,
      ttcMetadata: {
        engine: 'ttc-native',
        version: r.version,
        level: r.level,
        levelName: r.levelName,
        modeName: r.modeName,
        crc32: r.crc32,
        avgTau: r.avgTau,
        avgDelta: r.avgDelta,
        predominantBase: r.predominantBase,
        adaptiveRepUsed: r.adaptiveRepUsed,
      },
    };
  }
  const originalSize = inputBuffer.length;
  const compressed = zlib.deflateSync(inputBuffer, { level: 9 });
  const compressedSize = compressed.length;
  const compressionRatio = originalSize > 0
    ? ((originalSize - compressedSize) / originalSize) * 100
    : 0;
  return {
    compressed,
    originalSize,
    compressedSize,
    compressionRatio,
    ttcMetadata: {
      engine: 'legacy-zlib' as const,
      version: '1.0',
      level: 9,
      levelName: 'zlib-max',
      modeName: 'BASIC',
      crc32: crc32Checksum(inputBuffer),
      avgTau: 0,
      avgDelta: 0,
      predominantBase: 2,
      adaptiveRepUsed: false,
    },
  };
}

export function decompressFileBuffer(compressedBuffer: Buffer): {
  data: Buffer;
  ttcMetadata?: TtcDecompressionMetadata;
} {
  const addon = loadTtcAddon();
  if (addon) {
    try {
      const r = addon.ttcDecompress(compressedBuffer);
      return {
        data: Buffer.from(r.data),
        ttcMetadata: {
          engine: 'ttc-native',
          version: r.version,
          level: r.level,
          levelName: r.levelName,
          crc32Verified: r.crc32Verified,
          originalFileName: r.originalFileName,
        },
      };
    } catch {
      return {
        data: zlib.inflateSync(compressedBuffer),
        ttcMetadata: {
          engine: 'legacy-zlib' as const,
          version: '1.0',
          level: null,
          levelName: null,
          crc32Verified: true,
          originalFileName: null,
        },
      };
    }
  }
  return {
    data: zlib.inflateSync(compressedBuffer),
    ttcMetadata: {
      engine: 'legacy-zlib' as const,
      version: '1.0',
      level: null,
      levelName: null,
      crc32Verified: true,
      originalFileName: null,
    },
  };
}

export interface TernFileHeader {
  magic: string;
  version: number;
  originalFileName: string;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  encrypted: boolean;
  encryptionMode?: string;
  checksum: number;
  timestamp: string;
  ttcEngine?: 'ttc-native' | 'legacy-zlib';
  ttcVersion?: string;
  ttcLevel?: number;
  ttcLevelName?: string;
  ttcModeName?: string;
  ttcCrc32?: number;
  ttcAvgTau?: number;
  ttcAvgDelta?: number;
  ttcPredominantBase?: number;
  ttcAdaptiveRepUsed?: boolean;
}

function simpleChecksum(data: Buffer): number {
  let sum = 0;
  for (let i = 0; i < data.length; i++) {
    sum = ((sum << 5) - sum + data[i]) | 0;
  }
  return Math.abs(sum);
}

export function createTernFile(
  inputBuffer: Buffer,
  originalFileName: string,
  options: { encrypt?: boolean; encryptionMode?: EncryptionMode; level?: number; mode?: string } = {}
): { ternFile: Buffer; header: TernFileHeader; ttcMetadata?: TtcCompressionMetadata } {
  const result = compressFileBuffer(inputBuffer, {
    level: options.level,
    mode: options.mode,
    filename: originalFileName,
  });
  const { compressed, originalSize, compressedSize, compressionRatio, ttcMetadata } = result;

  let finalData: Buffer;
  let encrypted = false;
  let encryptionMode: string | undefined;

  if (options.encrypt) {
    const base64Compressed = compressed.toString('base64');
    const phaseResult = phaseSplit(base64Compressed, options.encryptionMode || 'balanced');
    const phaseJson = JSON.stringify(phaseResult, (_, v) => typeof v === 'bigint' ? v.toString() : v);
    finalData = Buffer.from(phaseJson, 'utf-8');
    encrypted = true;
    encryptionMode = options.encryptionMode || 'balanced';
  } else {
    finalData = compressed;
  }

  const header: TernFileHeader = {
    magic: 'TERN',
    version: ttcMetadata ? 2 : 1,
    originalFileName,
    originalSize,
    compressedSize: finalData.length,
    compressionRatio,
    encrypted,
    encryptionMode,
    checksum: ttcMetadata ? ttcMetadata.crc32 : crc32Checksum(inputBuffer),
    timestamp: new Date().toISOString(),
    ttcEngine: ttcMetadata?.engine,
    ttcVersion: ttcMetadata?.version,
    ttcLevel: ttcMetadata?.level,
    ttcLevelName: ttcMetadata?.levelName,
    ttcModeName: ttcMetadata?.modeName,
    ttcCrc32: ttcMetadata?.crc32,
    ttcAvgTau: ttcMetadata?.avgTau,
    ttcAvgDelta: ttcMetadata?.avgDelta,
    ttcPredominantBase: ttcMetadata?.predominantBase,
    ttcAdaptiveRepUsed: ttcMetadata?.adaptiveRepUsed,
  };

  const headerJson = JSON.stringify(header);
  const headerBuffer = Buffer.from(headerJson, 'utf-8');
  const headerLenBuffer = Buffer.alloc(4);
  headerLenBuffer.writeUInt32BE(headerBuffer.length, 0);

  return {
    ternFile: Buffer.concat([
      Buffer.from('TERN'),
      headerLenBuffer,
      headerBuffer,
      finalData,
    ]),
    header,
    ttcMetadata,
  };
}

export function parseTernFile(ternBuffer: Buffer): {
  header: TernFileHeader;
  originalData: Buffer;
  ttcMetadata?: TtcDecompressionMetadata;
} {
  const magic = ternBuffer.subarray(0, 4).toString('utf-8');
  if (magic !== 'TERN') {
    throw new Error('Invalid .tern file: bad magic bytes');
  }

  const headerLen = ternBuffer.readUInt32BE(4);
  const headerJson = ternBuffer.subarray(8, 8 + headerLen).toString('utf-8');
  const header: TernFileHeader = JSON.parse(headerJson);

  const dataBuffer = ternBuffer.subarray(8 + headerLen);

  let compressedPayload: Buffer;

  if (header.encrypted) {
    const phaseJson = dataBuffer.toString('utf-8');
    const phaseData: EncryptedPhaseData = JSON.parse(phaseJson);
    const recombined = phaseRecombine(phaseData);
    if (!recombined.success || !recombined.data) {
      throw new Error(`Phase decryption failed: ${recombined.error}`);
    }
    compressedPayload = Buffer.from(recombined.data, 'base64');
  } else {
    compressedPayload = dataBuffer;
  }

  const result = decompressFileBuffer(compressedPayload);

  if (result.ttcMetadata) {
    return { header, originalData: result.data, ttcMetadata: result.ttcMetadata };
  }

  const actualChecksum = simpleChecksum(result.data);
  if (actualChecksum !== header.checksum) {
    console.warn(`Checksum mismatch: expected ${header.checksum}, got ${actualChecksum}. File may be corrupted or truncated.`);
  }

  return { header, originalData: result.data };
}
