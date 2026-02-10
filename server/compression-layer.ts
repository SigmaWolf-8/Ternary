import { compressData, decompressData, ternaryEncode, ternaryDecode, runLengthCompress, runLengthDecompress } from './ternary';
import { phaseSplit, phaseRecombine, type EncryptionMode, type EncryptedPhaseData } from './salvi-core/phase-encryption';

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

  return JSON.stringify(envelope);
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

export function compressFileBuffer(inputBuffer: Buffer): {
  compressed: Buffer;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
} {
  const originalSize = inputBuffer.length;
  const ternaryEncoded = ternaryEncode(inputBuffer);
  const compressed = runLengthCompress(ternaryEncoded);
  const compressedSize = compressed.length;
  const compressionRatio = ((originalSize - compressedSize) / originalSize) * 100;

  return { compressed, originalSize, compressedSize, compressionRatio };
}

export function decompressFileBuffer(compressedBuffer: Buffer): Buffer {
  const ternaryEncoded = runLengthDecompress(compressedBuffer);
  return ternaryDecode(ternaryEncoded);
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
  options: { encrypt?: boolean; encryptionMode?: EncryptionMode } = {}
): { ternFile: Buffer; header: TernFileHeader } {
  const { compressed, originalSize, compressedSize, compressionRatio } = compressFileBuffer(inputBuffer);

  let finalData: Buffer;
  let encrypted = false;
  let encryptionMode: string | undefined;

  if (options.encrypt) {
    const base64Compressed = compressed.toString('base64');
    const phaseResult = phaseSplit(base64Compressed, options.encryptionMode || 'balanced');
    const phaseJson = JSON.stringify(phaseResult);
    finalData = Buffer.from(phaseJson, 'utf-8');
    encrypted = true;
    encryptionMode = options.encryptionMode || 'balanced';
  } else {
    finalData = compressed;
  }

  const header: TernFileHeader = {
    magic: 'TERN',
    version: 1,
    originalFileName,
    originalSize,
    compressedSize: finalData.length,
    compressionRatio,
    encrypted,
    encryptionMode,
    checksum: simpleChecksum(inputBuffer),
    timestamp: new Date().toISOString(),
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
  };
}

export function parseTernFile(ternBuffer: Buffer): {
  header: TernFileHeader;
  originalData: Buffer;
} {
  const magic = ternBuffer.subarray(0, 4).toString('utf-8');
  if (magic !== 'TERN') {
    throw new Error('Invalid .tern file: bad magic bytes');
  }

  const headerLen = ternBuffer.readUInt32BE(4);
  const headerJson = ternBuffer.subarray(8, 8 + headerLen).toString('utf-8');
  const header: TernFileHeader = JSON.parse(headerJson);

  const dataBuffer = ternBuffer.subarray(8 + headerLen);

  let decompressedBuffer: Buffer;

  if (header.encrypted) {
    const phaseJson = dataBuffer.toString('utf-8');
    const phaseData: EncryptedPhaseData = JSON.parse(phaseJson);
    const recombined = phaseRecombine(phaseData);
    if (!recombined.success || !recombined.data) {
      throw new Error(`Phase decryption failed: ${recombined.error}`);
    }
    const compressedBuffer = Buffer.from(recombined.data, 'base64');
    decompressedBuffer = decompressFileBuffer(compressedBuffer);
  } else {
    decompressedBuffer = decompressFileBuffer(dataBuffer);
  }

  const actualChecksum = simpleChecksum(decompressedBuffer);
  if (actualChecksum !== header.checksum) {
    console.warn(`Checksum mismatch: expected ${header.checksum}, got ${actualChecksum}. File may be corrupted or truncated.`);
  }

  return { header, originalData: decompressedBuffer };
}
