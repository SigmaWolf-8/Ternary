/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PLENUMNET RFC 3161 TIME-STAMPING AUTHORITY (TSA) SERVICE
 * Location:   server/services/tsa-service.ts
 *
 * Standards: RFC 3161, RFC 5816, RFC 5652 (CMS), ETSI EN 319 421/422
 *
 * Tiered policies per white paper §8:
 *   COMPLY    — Financial compliance (FINRA Rule 613, CAT)
 *   FORENSICS — Legal evidence, eDiscovery, Digital Evidence Vaults
 *   SENTINEL  — Government/military, ordering guaranteed
 *   SECURE    — Enterprise Zero-Trust (ZTNA)
 */

import * as crypto from 'crypto';
import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';
import * as asn1js from 'asn1js';
import {
  type CalendarServiceClient,
  type CalendarContext,
  enrichWithCalendars,
  serializeForExtension,
  CALENDAR_EXTENSION_OID,
} from './tsa-calendar-enrichment';
import { spongeHash } from '../crypto/sponge-hash';
import {
  buildCalendarExtension,
  type CalendarCompressionResult,
} from './tsa-calendar-compression';

export const TSA_POLICIES = {
  DEFAULT: '1.3.6.1.4.1.0.100.1.0',
  COMPLY: '1.3.6.1.4.1.0.100.1.1',
  FORENSICS: '1.3.6.1.4.1.0.100.1.2',
  SENTINEL: '1.3.6.1.4.1.0.100.1.3',
  SECURE: '1.3.6.1.4.1.0.100.1.4',
} as const;

export type TsaPolicyTier = keyof typeof TSA_POLICIES;

export const TSA_POLICY_METADATA: Record<string, {
  name: string;
  tier: TsaPolicyTier;
  accuracySeconds: number;
  accuracyMicros: number;
  orderingGuaranteed: boolean;
  auditRetentionYears: number;
  description: string;
  whitepaperSection: string;
}> = {
  [TSA_POLICIES.DEFAULT]: {
    name: 'PlenumNET General BTSP',
    tier: 'DEFAULT',
    accuracySeconds: 1,
    accuracyMicros: 0,
    orderingGuaranteed: false,
    auditRetentionYears: 7,
    description: 'General-purpose timestamping with 1-second declared accuracy.',
    whitepaperSection: 'N/A',
  },
  [TSA_POLICIES.COMPLY]: {
    name: 'PlenumNET COMPLY',
    tier: 'COMPLY',
    accuracySeconds: 0,
    accuracyMicros: 1,
    orderingGuaranteed: true,
    auditRetentionYears: 10,
    description: 'Financial compliance. ±1µs declared accuracy (RFC max 999µs); HPTP native ns resolution logged for FINRA Rule 613 / CAT nanosecond truncation.',
    whitepaperSection: '§8.1',
  },
  [TSA_POLICIES.FORENSICS]: {
    name: 'PlenumNET FORENSICS',
    tier: 'FORENSICS',
    accuracySeconds: 0,
    accuracyMicros: 100,
    orderingGuaranteed: true,
    auditRetentionYears: 25,
    description: 'Legal evidence. Digital Evidence Vault timestamps. Court-admissible. Offline verification. eDiscovery integration.',
    whitepaperSection: '§8.2',
  },
  [TSA_POLICIES.SENTINEL]: {
    name: 'PlenumNET SENTINEL',
    tier: 'SENTINEL',
    accuracySeconds: 0,
    accuracyMicros: 1,
    orderingGuaranteed: true,
    auditRetentionYears: 50,
    description: 'Government/military. Ordering guaranteed for timeline reconstruction. Centralized Timeline Viewer support.',
    whitepaperSection: '§8.3',
  },
  [TSA_POLICIES.SECURE]: {
    name: 'PlenumNET SECURE',
    tier: 'SECURE',
    accuracySeconds: 0,
    accuracyMicros: 10,
    orderingGuaranteed: true,
    auditRetentionYears: 10,
    description: 'Enterprise Zero-Trust. ZTNA temporal anomaly detection. Secure Collaboration Enclaves with verifiable TSTs on every access request.',
    whitepaperSection: '§8.4',
  },
};

export const HASH_ALGORITHM_OIDS: Record<string, string> = {
  '2.16.840.1.101.3.4.2.1': 'sha256',
  '2.16.840.1.101.3.4.2.2': 'sha384',
  '2.16.840.1.101.3.4.2.3': 'sha512',
  '2.16.840.1.101.3.4.2.8': 'sha3-256',
  '2.16.840.1.101.3.4.2.9': 'sha3-384',
  '2.16.840.1.101.3.4.2.10': 'sha3-512',
  '1.3.6.1.4.1.0.100.3.1': 'sponge-385',
};

export const HASH_NAME_TO_OID: Record<string, string> = Object.fromEntries(
  Object.entries(HASH_ALGORITHM_OIDS).map(([oid, name]) => [name, oid]),
);

export const HASH_EXPECTED_LENGTHS: Record<string, number> = {
  sha256: 32, sha384: 48, sha512: 64,
  'sha3-256': 32, 'sha3-384': 48, 'sha3-512': 64,
  'sponge-385': 49,
};

const RSA_SHA256_OID = '1.2.840.113549.1.1.11';
const CMS_SIGNED_DATA_OID = '1.2.840.113549.1.7.2';
const CMS_TST_INFO_OID = '1.2.840.113549.1.9.16.1.4';
const CONTENT_TYPE_OID = '1.2.840.113549.1.9.3';
const MESSAGE_DIGEST_OID = '1.2.840.113549.1.9.4';
const SIGNING_CERT_V2_OID = '1.2.840.113549.1.9.16.2.47';
const TSA_POLICY_OID_PREFIX = '1.3.6.1.4.1.0.100.1';

export interface HptpClient {
  getTimestamp(): Promise<{
    timestamp: string;
    precision: string;
    source: string;
    unixNano?: bigint;
  }>;
}

export interface TldsaClient {
  sign(hash: string): Promise<{
    signature: string;
    publicKeyId: string;
    securityLevel: string;
    algorithm: string;
  }>;
  verify?(hash: string, signature: string): Promise<boolean>;
}

export interface TsaConfig {
  privateKeyPath: string;
  certificatePath: string;
  chainPath: string;
  keysDirectory: string;
  defaultPolicy: string;
  enableDualSign: boolean;
  maxRequestSize: number;
}

export interface TsaTokenRecord {
  serialNumber: string;
  genTime: string;
  genTimeHptp: string;
  hptpPrecision: string;
  hptpSource: string;
  hashAlgorithm: string;
  hashedMessage: string;
  policyOid: string;
  policyTier: string;
  nonce: string | null;
  accuracy: { seconds: number; micros: number };
  ordering: boolean;
  requestIp: string;
  classicalSignatureAlgorithm: string;
  tldsaSignature: string | null;
  tldsaKeyId: string | null;
  merkleLeafHash: string;
  tokenSizeBytes: number;
  calendarSystems: string[];
  calendarSource: 'policy' | 'request' | 'merged' | 'none';
  createdAt: string;
}

export interface TsaHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  hptpAvailable: boolean;
  tldsaAvailable: boolean;
  tsaKeyLoaded: boolean;
  tsaCertValid: boolean;
  tsaCertExpiry: string;
  tsaCertSubject: string;
  serialNumber: string;
  tokensIssuedLast24h: number;
  policies: Array<{ oid: string; name: string; tier: string }>;
  supportedAlgorithms: string[];
  dualSignEnabled: boolean;
  merkleTreeDepth: number;
  merkleRoot: string;
  nativeHptpMaxPrecision: string;
  uptime: number;
}

export interface JsonTimestampRequest {
  hash: string;
  algorithm: string;
  policy?: string;
  nonce?: string;
  includeChain?: boolean;
  calendars?: string[];
}

export interface JsonTimestampResponse {
  granted: boolean;
  serialNumber: string;
  genTime: string;
  policy: string;
  policyTier: string;
  policyName: string;
  token: string;
  accuracy: { seconds: number; micros: number };
  ordering: boolean;
  hptpTimestamp: string;
  hptpPrecision: string;
  hptpSource: string;
  tldsaSignature: string | null;
  tldsaKeyId: string | null;
  merkleLeafHash: string;
  merkleRoot: string;
  verificationUrl: string;
  certificateUrl: string;
  calendarContext: CalendarContext | null;
}

export interface VerificationResult {
  valid: boolean;
  serialNumber: string;
  genTime: string;
  hashAlgorithm: string;
  policyOid: string;
  policyTier: string;
  policyName: string;
  accuracy: { seconds: number; micros: number };
  ordering: boolean;
  signerSubject: string;
  tldsaPresent: boolean;
  tldsaVerified?: boolean;
  tldsaAlgorithm?: string;
  tldsaKeyId?: string;
  verificationMethod: string;
  reason?: string;
}

function oidToAsn1(oid: string): asn1js.ObjectIdentifier {
  return new asn1js.ObjectIdentifier({ value: oid });
}

function bigintToAsn1Integer(value: bigint): asn1js.Integer {
  let hex = value.toString(16);
  if (hex.length % 2 !== 0) hex = '0' + hex;
  if (parseInt(hex[0], 16) >= 8) hex = '00' + hex;
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
  }
  return new asn1js.Integer({ valueHex: bytes.buffer });
}

class MerkleAuditLog {
  private leaves: string[] = [];
  private persistPath: string;
  private cachedRoot: string | null = null;

  constructor(keysDirectory: string) {
    this.persistPath = path.join(keysDirectory, 'merkle-leaves.jsonl');
    this.loadFromDisk();
  }

  private loadFromDisk(): void {
    if (fs.existsSync(this.persistPath)) {
      const lines = fs.readFileSync(this.persistPath, 'utf8').split('\n').filter(Boolean);
      this.leaves = lines.map(line => {
        try { return JSON.parse(line).hash; }
        catch { return ''; }
      }).filter(Boolean);
      this.cachedRoot = null;
      console.log(`Merkle audit log restored: ${this.leaves.length} leaves from ${this.persistPath}`);
    }
  }

  addLeaf(serialNumber: string, hashedMessage: string, genTime: string): string {
    const leafData = `${serialNumber}|${hashedMessage}|${genTime}`;
    const leafHash = spongeHash(Buffer.from(leafData, 'utf8'));
    this.leaves.push(leafHash);
    this.cachedRoot = null;
    fs.appendFileSync(this.persistPath,
      JSON.stringify({ serial: serialNumber, hash: leafHash, ts: genTime }) + '\n');
    return leafHash;
  }

  getRoot(): string {
    if (this.cachedRoot !== null) return this.cachedRoot;
    if (this.leaves.length === 0) {
      this.cachedRoot = spongeHash(Buffer.from('empty-tree', 'utf8'));
      return this.cachedRoot;
    }
    let level = [...this.leaves];
    while (level.length > 1) {
      const next: string[] = [];
      for (let i = 0; i < level.length; i += 2) {
        const left = level[i];
        const right = level[i + 1] || left;
        next.push(spongeHash(Buffer.from(left + right, 'utf8')));
      }
      level = next;
    }
    this.cachedRoot = level[0];
    return this.cachedRoot;
  }

  getDepth(): number {
    if (this.leaves.length === 0) return 0;
    return Math.ceil(Math.log2(this.leaves.length)) + 1;
  }

  getSize(): number { return this.leaves.length; }
}

export class TsaService {
  private config: TsaConfig;
  private hptpClient: HptpClient;
  private tldsaClient: TldsaClient;
  private privateKey: crypto.KeyObject | null = null;
  private certificate: Buffer | null = null;
  private certificatePem: string = '';
  private certificateParsed: crypto.X509Certificate | null = null;
  private serialCounter: bigint = BigInt(0);
  private tokenLog: TsaTokenRecord[] = [];
  private serialIndex: Map<string, TsaTokenRecord> = new Map();
  private merkleLog!: MerkleAuditLog;
  private calendarClient: CalendarServiceClient | null;
  private startTime: number = Date.now();

  constructor(config: TsaConfig, hptpClient: HptpClient, tldsaClient: TldsaClient, calendarClient?: CalendarServiceClient) {
    this.config = config;
    this.hptpClient = hptpClient;
    this.tldsaClient = tldsaClient;
    this.calendarClient = calendarClient ?? null;
  }

  async initialize(): Promise<{
    success: boolean;
    keyLoaded: boolean;
    certValid: boolean;
    certExpiry: string;
    certSubject: string;
    serialRestored: string;
  }> {
    if (!fs.existsSync(this.config.keysDirectory)) {
      fs.mkdirSync(this.config.keysDirectory, { recursive: true });
    }

    if (!fs.existsSync(this.config.privateKeyPath)) {
      this.generateTsaKeyPair();
    }

    const keyPem = fs.readFileSync(this.config.privateKeyPath, 'utf8');
    this.privateKey = crypto.createPrivateKey(keyPem);

    this.certificatePem = fs.readFileSync(this.config.certificatePath, 'utf8');
    this.certificate = Buffer.from(
      this.certificatePem
        .replace(/-----BEGIN CERTIFICATE-----/g, '')
        .replace(/-----END CERTIFICATE-----/g, '')
        .replace(/\s/g, ''),
      'base64',
    );
    this.certificateParsed = new crypto.X509Certificate(this.certificatePem);

    const counterPath = path.join(this.config.keysDirectory, 'serial-counter.txt');
    if (fs.existsSync(counterPath)) {
      this.serialCounter = BigInt(fs.readFileSync(counterPath, 'utf8').trim());
    }

    this.merkleLog = new MerkleAuditLog(this.config.keysDirectory);

    return {
      success: true,
      keyLoaded: this.privateKey !== null,
      certValid: this.isCertificateValid(),
      certExpiry: this.certificateParsed?.validTo || 'unknown',
      certSubject: this.certificateParsed?.subject || 'unknown',
      serialRestored: this.serialCounter.toString(),
    };
  }

  async processTimestampRequest(derRequest: Buffer, requestIp: string, calendarContext?: CalendarContext | null): Promise<Buffer> {
    let parsed: {
      version: number;
      hashAlgorithmOid: string;
      hashedMessage: Buffer;
      reqPolicy: string | null;
      nonce: bigint | null;
      certReq: boolean;
    };
    try {
      parsed = this.parseTimeStampReq(derRequest);
    } catch (error) {
      return this.buildErrorResponse(2, `Bad request: ${(error as Error).message}`);
    }

    const hashAlgName = HASH_ALGORITHM_OIDS[parsed.hashAlgorithmOid];
    if (!hashAlgName) {
      return this.buildErrorResponse(2, `Unsupported hash algorithm: ${parsed.hashAlgorithmOid}`);
    }

    const policyOid = parsed.reqPolicy || this.config.defaultPolicy;
    const policyMeta = TSA_POLICY_METADATA[policyOid];
    if (!policyMeta) {
      return this.buildErrorResponse(2, `Unknown policy: ${policyOid}`);
    }

    let hptpResult: { timestamp: string; precision: string; source: string };
    try {
      hptpResult = await this.hptpClient.getTimestamp();
    } catch {
      hptpResult = {
        timestamp: new Date().toISOString(),
        precision: 'millisecond-fallback',
        source: 'system-clock',
      };
    }

    const serialNumber = this.nextSerialNumber();

    const declaredAccuracy = {
      seconds: policyMeta.accuracySeconds,
      micros: Math.min(policyMeta.accuracyMicros, 999),
    };

    const genTime = this.formatGeneralizedTime(hptpResult.timestamp);
    const tstInfo = this.buildTSTInfo({
      serialNumber,
      genTime,
      hashAlgorithmOid: parsed.hashAlgorithmOid,
      hashedMessage: parsed.hashedMessage,
      policyOid,
      nonce: parsed.nonce,
      accuracy: declaredAccuracy,
      ordering: policyMeta.orderingGuaranteed,
      calendarContext: calendarContext || undefined,
    });

    const timeStampToken = this.buildCmsSignedData(tstInfo, parsed.certReq);

    let tldsaSig: { signature: string; publicKeyId: string } | null = null;
    if (this.config.enableDualSign) {
      try {
        const tstHash = crypto.createHash('sha3-256').update(tstInfo).digest('hex');
        const result = await this.tldsaClient.sign(tstHash);
        tldsaSig = { signature: result.signature, publicKeyId: result.publicKeyId };
      } catch {
        // Classical RSA sufficient; TL-DSA best-effort
      }
    }

    const response = this.buildTimeStampResp(0, timeStampToken);

    const merkleLeaf = this.merkleLog.addLeaf(
      serialNumber.toString(),
      parsed.hashedMessage.toString('hex'),
      genTime,
    );

    this.logToken({
      serialNumber: serialNumber.toString(),
      genTime,
      genTimeHptp: hptpResult.timestamp,
      hptpPrecision: hptpResult.precision,
      hptpSource: hptpResult.source,
      hashAlgorithm: hashAlgName,
      hashedMessage: parsed.hashedMessage.toString('hex'),
      policyOid,
      policyTier: policyMeta.tier,
      nonce: parsed.nonce?.toString() || null,
      accuracy: { seconds: policyMeta.accuracySeconds, micros: policyMeta.accuracyMicros },
      ordering: policyMeta.orderingGuaranteed,
      requestIp,
      classicalSignatureAlgorithm: 'RSA-4096-SHA256',
      tldsaSignature: tldsaSig?.signature || null,
      tldsaKeyId: tldsaSig?.publicKeyId || null,
      merkleLeafHash: merkleLeaf,
      tokenSizeBytes: response.length,
      calendarSystems: calendarContext
        ? calendarContext.calendars.map(c => c.system)
        : [],
      calendarSource: calendarContext
        ? (calendarContext.source.requested.length > 0
            ? (calendarContext.source.policy.length > 0 ? 'merged' : 'request')
            : 'policy')
        : 'none',
      createdAt: new Date().toISOString(),
    });

    return response;
  }

  async processJsonRequest(
    input: JsonTimestampRequest,
    requestIp: string,
  ): Promise<JsonTimestampResponse> {
    const oid = HASH_NAME_TO_OID[input.algorithm];
    if (!oid) {
      throw new Error(`Unsupported algorithm: ${input.algorithm}. Supported: ${Object.keys(HASH_NAME_TO_OID).join(', ')}`);
    }

    const hashBuffer = Buffer.from(input.hash, 'hex');
    const expected = HASH_EXPECTED_LENGTHS[input.algorithm];
    if (hashBuffer.length !== expected) {
      throw new Error(`Invalid hash length for ${input.algorithm}: expected ${expected} bytes, got ${hashBuffer.length}`);
    }

    const policyOid = input.policy || this.config.defaultPolicy;
    const policyMeta = TSA_POLICY_METADATA[policyOid];
    if (!policyMeta) throw new Error(`Unknown policy: ${policyOid}. Available: DEFAULT, COMPLY, FORENSICS, SENTINEL, SECURE`);

    const syntheticReq = this.buildTimeStampReq({
      hashAlgorithmOid: oid,
      hashedMessage: hashBuffer,
      reqPolicy: policyOid,
      nonce: input.nonce ? BigInt(`0x${input.nonce}`) : null,
      certReq: input.includeChain || false,
    });

    let calendarContext: CalendarContext | null = null;
    if (this.calendarClient) {
      calendarContext = await enrichWithCalendars(
        new Date().toISOString(),
        policyMeta.tier,
        this.calendarClient,
        input.calendars,
      );
    }

    const derResponse = await this.processTimestampRequest(syntheticReq, requestIp, calendarContext);
    const lastToken = this.tokenLog[this.tokenLog.length - 1];

    return {
      granted: true,
      serialNumber: lastToken.serialNumber,
      genTime: lastToken.genTime,
      policy: lastToken.policyOid,
      policyTier: lastToken.policyTier,
      policyName: policyMeta.name,
      token: derResponse.toString('base64'),
      accuracy: lastToken.accuracy,
      ordering: lastToken.ordering,
      hptpTimestamp: lastToken.genTimeHptp,
      hptpPrecision: lastToken.hptpPrecision,
      hptpSource: lastToken.hptpSource,
      tldsaSignature: lastToken.tldsaSignature,
      tldsaKeyId: lastToken.tldsaKeyId,
      merkleLeafHash: lastToken.merkleLeafHash,
      merkleRoot: this.merkleLog.getRoot(),
      verificationUrl: '/api/tsa/verify',
      certificateUrl: '/api/tsa/certificate',
      calendarContext,
    };
  }

  async verifyToken(tokenInput: Buffer): Promise<VerificationResult> {
    try {
      const parsed = this.parseTimeStampResp(tokenInput);
      const signatureValid = this.verifyCmsSignature(parsed.signedData);
      const tstInfo = this.parseTSTInfo(parsed.tstInfo);
      const policyMeta = TSA_POLICY_METADATA[tstInfo.policyOid];

      const tokenRecord = this.tokenLog.find(t => t.serialNumber === tstInfo.serialNumber);
      const tldsaPresent = !!(tokenRecord?.tldsaSignature);
      let tldsaVerified: boolean | undefined;
      let tldsaAlgorithm: string | undefined;
      let tldsaKeyId: string | undefined;

      if (tldsaPresent && tokenRecord) {
        tldsaKeyId = tokenRecord.tldsaKeyId || undefined;
        try {
          const tstHash = crypto.createHash('sha3-256').update(parsed.tstInfo).digest('hex');
          if (this.tldsaClient.verify) {
            tldsaVerified = await this.tldsaClient.verify(tstHash, tokenRecord.tldsaSignature!);
          } else {
            const recomputedResult = await this.tldsaClient.sign(tstHash);
            tldsaVerified = recomputedResult.signature === tokenRecord.tldsaSignature;
          }
          tldsaAlgorithm = 'TL-DSA-87';
        } catch {
          tldsaVerified = false;
        }
      }

      return {
        valid: signatureValid,
        serialNumber: tstInfo.serialNumber,
        genTime: tstInfo.genTime,
        hashAlgorithm: HASH_ALGORITHM_OIDS[tstInfo.hashAlgorithmOid] || tstInfo.hashAlgorithmOid,
        policyOid: tstInfo.policyOid,
        policyTier: policyMeta?.tier || 'UNKNOWN',
        policyName: policyMeta?.name || 'Unknown',
        accuracy: tstInfo.accuracy,
        ordering: tstInfo.ordering,
        signerSubject: this.certificateParsed?.subject || 'unknown',
        tldsaPresent,
        ...(tldsaPresent ? { tldsaVerified, tldsaAlgorithm, tldsaKeyId } : {}),
        verificationMethod: tldsaPresent
          ? 'online — dual-verified (RSA-4096 + TL-DSA-87)'
          : 'online (offline verification also supported via OpenSSL ts -verify)',
      };
    } catch (error) {
      return {
        valid: false,
        serialNumber: 'unknown', genTime: 'unknown', hashAlgorithm: 'unknown',
        policyOid: 'unknown', policyTier: 'unknown', policyName: 'unknown',
        accuracy: { seconds: 0, micros: 0 }, ordering: false,
        signerSubject: 'unknown', tldsaPresent: false,
        verificationMethod: 'online',
        reason: (error as Error).message,
      };
    }
  }

  getTsaCertificate(): {
    certificate: string;
    chain: string | null;
    subject: string;
    issuer: string;
    validFrom: string;
    validTo: string;
    serialNumber: string;
    fingerprint256: string;
    extendedKeyUsage: string;
    publicKeyAlgorithm: string;
    publicKeySize: string;
    offlineVerificationInstructions: string;
  } {
    if (!this.certificateParsed) throw new Error('TSA certificate not loaded');
    const certPem = fs.readFileSync(this.config.certificatePath, 'utf8');
    const chainPem = fs.existsSync(this.config.chainPath)
      ? fs.readFileSync(this.config.chainPath, 'utf8') : null;

    return {
      certificate: certPem,
      chain: chainPem,
      subject: this.certificateParsed.subject,
      issuer: this.certificateParsed.issuer,
      validFrom: this.certificateParsed.validFrom,
      validTo: this.certificateParsed.validTo,
      serialNumber: this.certificateParsed.serialNumber,
      fingerprint256: this.certificateParsed.fingerprint256,
      extendedKeyUsage: 'critical, id-kp-timeStamping (1.3.6.1.5.5.7.3.8)',
      publicKeyAlgorithm: this.privateKey?.asymmetricKeyType || 'rsa',
      publicKeySize: '4096',
      offlineVerificationInstructions:
        'openssl ts -verify -data <original-file> -in <response.tsr> -CAfile plenumnet-tsa.pem',
    };
  }

  getPolicyInfo(): {
    defaultPolicy: string;
    policies: Array<{
      oid: string; name: string; tier: string; accuracyDeclared: string;
      orderingGuaranteed: boolean; auditRetentionYears: number;
      description: string; whitepaperSection: string;
    }>;
    conformsTo: string[];
    operator: { name: string; division: string; country: string };
    timeSource: { name: string; description: string };
    signatureAlgorithms: { classical: string; postQuantum: string; dualSign: string };
    supportedHashAlgorithms: Array<{ name: string; oid: string }>;
    verificationMethods: { online: string; offline: string; tools: string[] };
  } {
    return {
      defaultPolicy: this.config.defaultPolicy,
      policies: Object.entries(TSA_POLICY_METADATA).map(([oid, m]) => ({
        oid, name: m.name, tier: m.tier,
        accuracyDeclared: m.accuracySeconds > 0 ? `${m.accuracySeconds}s` : `${m.accuracyMicros}µs`,
        orderingGuaranteed: m.orderingGuaranteed,
        auditRetentionYears: m.auditRetentionYears,
        description: m.description, whitepaperSection: m.whitepaperSection,
      })),
      conformsTo: [
        'RFC 3161 — Internet X.509 PKI Time-Stamp Protocol (TSP)',
        'RFC 5816 — ESSCertIDv2 Update to RFC 3161',
        'RFC 5652 — Cryptographic Message Syntax (CMS)',
        'ETSI EN 319 421 — Policy and Security Requirements for TSPs issuing Time-Stamps',
        'ETSI EN 319 422 — Time-stamping protocol and time-stamp token profiles',
      ],
      operator: {
        name: 'Capomastro Holdings Ltd.',
        division: 'Applied Physics Division — PlenumNET',
        country: 'CA',
      },
      timeSource: {
        name: 'PlenumNET HPTP — Hardened Precision Time Protocol',
        description: 'Distributed mesh of GPS/GNSS receivers with atomic clock backup. Hardened against jamming and spoofing through cross-constellation validation and anomaly detection.',
      },
      signatureAlgorithms: {
        classical: 'RSA-4096 with SHA-256 (interoperable with OpenSSL, Adobe, jarsigner, Authenticode)',
        postQuantum: 'TL-DSA-87 (Ternary Lattice Digital Signature Algorithm — NIST Level 5)',
        dualSign: 'Every token signed with classical RSA AND optionally TL-DSA for quantum-safe readiness',
      },
      supportedHashAlgorithms: Object.entries(HASH_ALGORITHM_OIDS)
        .map(([oid, name]) => ({ name: name.toUpperCase(), oid })),
      verificationMethods: {
        online: 'POST /api/tsa/verify',
        offline: 'openssl ts -verify -data <file> -in <response.tsr> -CAfile plenumnet-tsa.pem',
        tools: ['OpenSSL', 'Adobe Acrobat', 'jarsigner', 'Authenticode', 'pdf-rfc3161', 'rfc3161ng'],
      },
    };
  }

  async getHealth(): Promise<TsaHealth> {
    const now = Date.now();
    const last24h = new Date(now - 86400000).toISOString();
    const recentTokens = this.tokenLog.filter(t => t.createdAt >= last24h);

    let hptpAvailable = false;
    try { await this.hptpClient.getTimestamp(); hptpAvailable = true; } catch { /* */ }

    let tldsaAvailable = false;
    try { await this.tldsaClient.sign('healthcheck'); tldsaAvailable = true; } catch { /* */ }

    return {
      status: hptpAvailable && this.privateKey ? 'healthy'
        : this.privateKey ? 'degraded' : 'unhealthy',
      hptpAvailable,
      tldsaAvailable,
      tsaKeyLoaded: this.privateKey !== null,
      tsaCertValid: this.isCertificateValid(),
      tsaCertExpiry: this.certificateParsed?.validTo || 'unknown',
      tsaCertSubject: this.certificateParsed?.subject || 'unknown',
      serialNumber: this.serialCounter.toString(),
      tokensIssuedLast24h: recentTokens.length,
      policies: Object.entries(TSA_POLICY_METADATA).map(([oid, m]) => ({
        oid, name: m.name, tier: m.tier,
      })),
      supportedAlgorithms: Object.values(HASH_ALGORITHM_OIDS),
      dualSignEnabled: this.config.enableDualSign,
      merkleTreeDepth: this.merkleLog.getDepth(),
      merkleRoot: this.merkleLog.getRoot(),
      nativeHptpMaxPrecision: hptpAvailable ? 'femtosecond-class (HPTP native)' : 'unavailable',
      uptime: now - this.startTime,
    };
  }

  queryTokenLog(filters: {
    since?: string; until?: string;
    hashAlgorithm?: string; policyTier?: string;
    serialNumber?: string;
    limit?: number;
  }): { total: number; merkleRoot: string; merkleDepth: number; merkleLeaves: number; tokens: TsaTokenRecord[] } {
    if (filters.serialNumber) {
      const record = this.getTokenBySerial(filters.serialNumber);
      return {
        total: record ? 1 : 0,
        merkleRoot: this.merkleLog.getRoot(),
        merkleDepth: this.merkleLog.getDepth(),
        merkleLeaves: this.merkleLog.getSize(),
        tokens: record ? [record] : [],
      };
    }

    let records = [...this.tokenLog];
    if (filters.since) records = records.filter(r => r.createdAt >= filters.since!);
    if (filters.until) records = records.filter(r => r.createdAt <= filters.until!);
    if (filters.hashAlgorithm) records = records.filter(r => r.hashAlgorithm === filters.hashAlgorithm);
    if (filters.policyTier) records = records.filter(r => r.policyTier === filters.policyTier);
    records.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    return {
      total: records.length,
      merkleRoot: this.merkleLog.getRoot(),
      merkleDepth: this.merkleLog.getDepth(),
      merkleLeaves: this.merkleLog.getSize(),
      tokens: records.slice(0, filters.limit || 100),
    };
  }

  // ==========================================================================
  // ASN.1 METHODS — RFC 3161 wire format using asn1js
  // ==========================================================================

  private parseTimeStampReq(der: Buffer): {
    version: number; hashAlgorithmOid: string; hashedMessage: Buffer;
    reqPolicy: string | null; nonce: bigint | null; certReq: boolean;
  } {
    const asn1 = asn1js.fromBER(new Uint8Array(der).buffer);
    if (asn1.offset === -1) throw new Error('Invalid ASN.1 DER encoding');

    const seq = asn1.result as asn1js.Sequence;
    const values = seq.valueBlock.value;
    if (values.length < 2) throw new Error('TimeStampReq must have at least 2 fields');

    const version = (values[0] as asn1js.Integer).valueBlock.valueDec;
    const msgImprint = values[1] as asn1js.Sequence;
    const miValues = msgImprint.valueBlock.value;
    const algIdSeq = miValues[0] as asn1js.Sequence;
    const hashAlgorithmOid = (algIdSeq.valueBlock.value[0] as asn1js.ObjectIdentifier).valueBlock.toString();
    const hashedMessage = Buffer.from((miValues[1] as asn1js.OctetString).valueBlock.valueHexView);

    let reqPolicy: string | null = null;
    let nonce: bigint | null = null;
    let certReq = false;

    for (let i = 2; i < values.length; i++) {
      const v = values[i];
      if (v instanceof asn1js.ObjectIdentifier) {
        reqPolicy = v.valueBlock.toString();
      } else if (v instanceof asn1js.Integer) {
        const hex = Buffer.from(v.valueBlock.valueHexView).toString('hex');
        nonce = hex ? BigInt('0x' + hex) : null;
      } else if (v instanceof asn1js.Boolean) {
        certReq = v.valueBlock.value;
      }
    }

    return { version, hashAlgorithmOid, hashedMessage, reqPolicy, nonce, certReq };
  }

  private buildTimeStampReq(fields: {
    hashAlgorithmOid: string; hashedMessage: Buffer;
    reqPolicy: string | null; nonce: bigint | null; certReq: boolean;
  }): Buffer {
    const seqValues: asn1js.BaseBlock[] = [];

    seqValues.push(new asn1js.Integer({ value: 1 }));

    const msgImprint = new asn1js.Sequence({
      value: [
        new asn1js.Sequence({
          value: [
            oidToAsn1(fields.hashAlgorithmOid),
            new asn1js.Null(),
          ],
        }),
        new asn1js.OctetString({ valueHex: new Uint8Array(fields.hashedMessage).buffer }),
      ],
    });
    seqValues.push(msgImprint);

    if (fields.reqPolicy) {
      seqValues.push(oidToAsn1(fields.reqPolicy));
    }

    if (fields.nonce !== null) {
      seqValues.push(bigintToAsn1Integer(fields.nonce));
    }

    if (fields.certReq) {
      seqValues.push(new asn1js.Boolean({ value: true }));
    }

    const tsReq = new asn1js.Sequence({ value: seqValues });
    const ber = tsReq.toBER(false);
    return Buffer.from(ber);
  }

  private buildTSTInfo(fields: {
    serialNumber: bigint; genTime: string; hashAlgorithmOid: string;
    hashedMessage: Buffer; policyOid: string; nonce: bigint | null;
    accuracy: { seconds: number; micros: number }; ordering: boolean;
    calendarContext?: CalendarContext;
  }): Buffer {
    const seqValues: asn1js.BaseBlock[] = [];

    seqValues.push(new asn1js.Integer({ value: 1 }));

    seqValues.push(oidToAsn1(fields.policyOid));

    const msgImprint = new asn1js.Sequence({
      value: [
        new asn1js.Sequence({
          value: [
            oidToAsn1(fields.hashAlgorithmOid),
            new asn1js.Null(),
          ],
        }),
        new asn1js.OctetString({ valueHex: new Uint8Array(fields.hashedMessage).buffer }),
      ],
    });
    seqValues.push(msgImprint);

    seqValues.push(bigintToAsn1Integer(fields.serialNumber));

    seqValues.push(new asn1js.GeneralizedTime({ valueDate: this.parseGeneralizedTime(fields.genTime) }));

    const accValues: asn1js.BaseBlock[] = [];
    if (fields.accuracy.seconds > 0) {
      accValues.push(new asn1js.Integer({ value: fields.accuracy.seconds }));
    }
    if (fields.accuracy.micros > 0) {
      accValues.push(new asn1js.Constructed({
        idBlock: { tagClass: 3, tagNumber: 1 },
        value: [new asn1js.Integer({ value: fields.accuracy.micros })],
      }));
    }
    if (accValues.length > 0) {
      seqValues.push(new asn1js.Sequence({ value: accValues }));
    }

    if (fields.ordering) {
      seqValues.push(new asn1js.Boolean({ value: true }));
    }

    if (fields.nonce !== null) {
      seqValues.push(bigintToAsn1Integer(fields.nonce));
    }

    const tsaName = new asn1js.Constructed({
      idBlock: { tagClass: 3, tagNumber: 0 },
      value: [
        new asn1js.Constructed({
          idBlock: { tagClass: 3, tagNumber: 4 },
          value: [
            new asn1js.Sequence({
              value: [
                new asn1js.Set({
                  value: [
                    new asn1js.Sequence({
                      value: [
                        oidToAsn1('2.5.4.3'),
                        new asn1js.Utf8String({ value: 'PlenumNET Time-Stamping Authority' }),
                      ],
                    }),
                  ],
                }),
                new asn1js.Set({
                  value: [
                    new asn1js.Sequence({
                      value: [
                        oidToAsn1('2.5.4.10'),
                        new asn1js.Utf8String({ value: 'Capomastro Holdings Ltd.' }),
                      ],
                    }),
                  ],
                }),
              ],
            }),
          ],
        }),
      ],
    });
    seqValues.push(tsaName);

    if (fields.calendarContext) {
      const calendarJson = serializeForExtension(fields.calendarContext);
      const tier = fields.calendarContext.policyTier || 'DEFAULT';
      let extensionPayload: Uint8Array;
      try {
        const { buffer, compressed, metrics } = buildCalendarExtension(
          JSON.parse(calendarJson),
        );
        extensionPayload = buffer;
        if (compressed && metrics) {
          console.log(`Calendar extension compressed: ${metrics.originalSize}B → ${buffer.length}B (${(metrics.effectiveRatio * 100).toFixed(1)}%) pipeline=${metrics.pipelineId} tier=${tier}`);
        }
      } catch {
        extensionPayload = new TextEncoder().encode(calendarJson);
      }
      const calendarExtension = new asn1js.Sequence({
        value: [
          oidToAsn1(CALENDAR_EXTENSION_OID),
          new asn1js.Boolean({ value: false }),
          new asn1js.OctetString({
            valueHex: extensionPayload,
          }),
        ],
      });

      const extensions = new asn1js.Constructed({
        idBlock: { tagClass: 3, tagNumber: 1 },
        value: [
          new asn1js.Sequence({
            value: [calendarExtension],
          }),
        ],
      });
      seqValues.push(extensions);
    }

    const tstInfo = new asn1js.Sequence({ value: seqValues });
    return Buffer.from(tstInfo.toBER(false));
  }

  private signWithClassicalKey(tstInfo: Buffer): Buffer {
    if (!this.privateKey) throw new Error('TSA private key not loaded');
    const signer = crypto.createSign('SHA256');
    signer.update(tstInfo);
    return signer.sign(this.privateKey);
  }

  private buildCmsSignedData(tstInfo: Buffer, includeCerts: boolean): Buffer {
    if (!this.certificate || !this.privateKey) throw new Error('TSA key/certificate not loaded');

    const tstDigest = crypto.createHash('sha256').update(tstInfo).digest();

    const contentTypeAttr = new asn1js.Sequence({
      value: [
        oidToAsn1(CONTENT_TYPE_OID),
        new asn1js.Set({
          value: [oidToAsn1(CMS_TST_INFO_OID)],
        }),
      ],
    });

    const messageDigestAttr = new asn1js.Sequence({
      value: [
        oidToAsn1(MESSAGE_DIGEST_OID),
        new asn1js.Set({
          value: [
            new asn1js.OctetString({ valueHex: new Uint8Array(tstDigest).buffer }),
          ],
        }),
      ],
    });

    const attrsForSigning = new asn1js.Set({
      value: [contentTypeAttr, messageDigestAttr],
    });
    const attrsDer = Buffer.from(attrsForSigning.toBER(false));

    const signer = crypto.createSign('SHA256');
    signer.update(attrsDer);
    const signature = signer.sign(this.privateKey);

    const signedAttrs = new asn1js.Constructed({
      idBlock: { tagClass: 3, tagNumber: 0 },
      value: [contentTypeAttr, messageDigestAttr],
    });

    const issuerSerial = this.getIssuerAndSerial();

    const signerInfo = new asn1js.Sequence({
      value: [
        new asn1js.Integer({ value: 1 }),
        issuerSerial,
        new asn1js.Sequence({
          value: [oidToAsn1('2.16.840.1.101.3.4.2.1'), new asn1js.Null()],
        }),
        signedAttrs,
        new asn1js.Sequence({
          value: [oidToAsn1(RSA_SHA256_OID), new asn1js.Null()],
        }),
        new asn1js.OctetString({ valueHex: new Uint8Array(signature).buffer }),
      ],
    });

    const sdValues: asn1js.BaseBlock[] = [
      new asn1js.Integer({ value: 1 }),
      new asn1js.Set({
        value: [
          new asn1js.Sequence({
            value: [oidToAsn1('2.16.840.1.101.3.4.2.1'), new asn1js.Null()],
          }),
        ],
      }),
      new asn1js.Sequence({
        value: [
          oidToAsn1(CMS_TST_INFO_OID),
          new asn1js.Constructed({
            idBlock: { tagClass: 3, tagNumber: 0 },
            value: [
              new asn1js.OctetString({ valueHex: new Uint8Array(tstInfo).buffer }),
            ],
          }),
        ],
      }),
    ];

    if (includeCerts && this.certificate) {
      const certAsn1 = asn1js.fromBER(new Uint8Array(this.certificate).buffer);
      sdValues.push(new asn1js.Constructed({
        idBlock: { tagClass: 3, tagNumber: 0 },
        value: [certAsn1.result],
      }));
    }

    sdValues.push(new asn1js.Set({ value: [signerInfo] }));

    const signedData = new asn1js.Sequence({ value: sdValues });

    const contentInfo = new asn1js.Sequence({
      value: [
        oidToAsn1(CMS_SIGNED_DATA_OID),
        new asn1js.Constructed({
          idBlock: { tagClass: 3, tagNumber: 0 },
          value: [signedData],
        }),
      ],
    });

    return Buffer.from(contentInfo.toBER(false));
  }

  private buildTimeStampResp(status: number, token?: Buffer): Buffer {
    const statusInfo = new asn1js.Sequence({
      value: [new asn1js.Integer({ value: status })],
    });

    const respValues: asn1js.BaseBlock[] = [statusInfo];

    if (token && status === 0) {
      const tokenAsn1 = asn1js.fromBER(new Uint8Array(token).buffer);
      if (tokenAsn1.offset !== -1) {
        respValues.push(tokenAsn1.result);
      }
    }

    const resp = new asn1js.Sequence({ value: respValues });
    return Buffer.from(resp.toBER(false));
  }

  private buildErrorResponse(status: number, message: string): Buffer {
    const statusInfo = new asn1js.Sequence({
      value: [
        new asn1js.Integer({ value: status }),
        new asn1js.Sequence({
          value: [new asn1js.Utf8String({ value: message })],
        }),
      ],
    });
    const resp = new asn1js.Sequence({ value: [statusInfo] });
    return Buffer.from(resp.toBER(false));
  }

  private parseTimeStampResp(der: Buffer): { status: number; signedData: Buffer; tstInfo: Buffer } {
    const asn1Result = asn1js.fromBER(new Uint8Array(der).buffer);
    if (asn1Result.offset === -1) throw new Error('Invalid TimeStampResp DER');

    const seq = asn1Result.result as asn1js.Sequence;
    const values = seq.valueBlock.value;

    const statusSeq = values[0] as asn1js.Sequence;
    const status = (statusSeq.valueBlock.value[0] as asn1js.Integer).valueBlock.valueDec;

    if (status !== 0 || values.length < 2) {
      throw new Error(`TimeStampResp status: ${status} (not granted)`);
    }

    const contentInfo = values[1] as asn1js.Sequence;
    const ciValues = contentInfo.valueBlock.value;
    const contentType = (ciValues[0] as asn1js.ObjectIdentifier).valueBlock.toString();

    if (contentType !== CMS_SIGNED_DATA_OID) {
      throw new Error(`Unexpected content type: ${contentType}`);
    }

    const signedDataContainer = ciValues[1] as asn1js.Constructed;
    const signedDataSeq = signedDataContainer.valueBlock.value[0] as asn1js.Sequence;
    const sdValues = signedDataSeq.valueBlock.value;

    const encapContentInfo = sdValues[2] as asn1js.Sequence;
    const eciValues = encapContentInfo.valueBlock.value;

    let tstInfoDer: Buffer;
    if (eciValues.length > 1) {
      const explicit0 = eciValues[1] as asn1js.Constructed;
      const octetString = explicit0.valueBlock.value[0] as asn1js.OctetString;
      tstInfoDer = Buffer.from(octetString.valueBlock.valueHexView);
    } else {
      throw new Error('No encapsulated TSTInfo found in SignedData');
    }

    const signedDataDer = Buffer.from(signedDataSeq.toBER(false));

    return { status, signedData: signedDataDer, tstInfo: tstInfoDer };
  }

  private parseTSTInfo(der: Buffer): {
    serialNumber: string; genTime: string; hashAlgorithmOid: string;
    policyOid: string; accuracy: { seconds: number; micros: number }; ordering: boolean;
  } {
    const asn1Result = asn1js.fromBER(new Uint8Array(der).buffer);
    if (asn1Result.offset === -1) throw new Error('Invalid TSTInfo DER');

    const seq = asn1Result.result as asn1js.Sequence;
    const values = seq.valueBlock.value;

    const policyOid = (values[1] as asn1js.ObjectIdentifier).valueBlock.toString();

    const msgImprint = values[2] as asn1js.Sequence;
    const algIdSeq = msgImprint.valueBlock.value[0] as asn1js.Sequence;
    const hashAlgorithmOid = (algIdSeq.valueBlock.value[0] as asn1js.ObjectIdentifier).valueBlock.toString();

    const serialHex = Buffer.from((values[3] as asn1js.Integer).valueBlock.valueHexView).toString('hex');
    const serialNumber = BigInt('0x' + serialHex).toString();

    const genTime = (values[4] as asn1js.GeneralizedTime).valueBlock.value;

    let accuracy = { seconds: 0, micros: 0 };
    let ordering = false;

    for (let i = 5; i < values.length; i++) {
      const v = values[i];
      if (v instanceof asn1js.Sequence) {
        const accVals = v.valueBlock.value;
        for (const av of accVals) {
          if (av instanceof asn1js.Integer) {
            accuracy.seconds = av.valueBlock.valueDec;
          } else if (av instanceof asn1js.Constructed && av.idBlock.tagNumber === 1) {
            const inner = av.valueBlock.value[0] as asn1js.Integer;
            accuracy.micros = inner.valueBlock.valueDec;
          }
        }
      } else if (v instanceof asn1js.Boolean) {
        ordering = v.valueBlock.value;
      }
    }

    return { serialNumber, genTime, hashAlgorithmOid, policyOid, accuracy, ordering };
  }

  private verifyCmsSignature(signedData: Buffer): boolean {
    try {
      const asn1Result = asn1js.fromBER(new Uint8Array(signedData).buffer);
      if (asn1Result.offset === -1) return false;

      const sdSeq = asn1Result.result as asn1js.Sequence;
      const sdValues = sdSeq.valueBlock.value;

      const encapContentInfo = sdValues[2] as asn1js.Sequence;
      const eciValues = encapContentInfo.valueBlock.value;
      const explicit0 = eciValues[1] as asn1js.Constructed;
      const octetString = explicit0.valueBlock.value[0] as asn1js.OctetString;
      const tstInfoBytes = Buffer.from(octetString.valueBlock.valueHexView);

      let signerInfoSet: asn1js.Set | null = null;
      for (const v of sdValues) {
        if (v instanceof asn1js.Set && v !== sdValues[1]) {
          signerInfoSet = v;
        }
      }
      if (!signerInfoSet) return false;

      const signerInfo = signerInfoSet.valueBlock.value[0] as asn1js.Sequence;
      const siValues = signerInfo.valueBlock.value;

      let signatureOctet: asn1js.OctetString | null = null;
      let signedAttrsConstruct: asn1js.Constructed | null = null;

      for (const v of siValues) {
        if (v instanceof asn1js.OctetString) {
          signatureOctet = v;
        }
        if (v instanceof asn1js.Constructed && v.idBlock.tagClass === 3 && v.idBlock.tagNumber === 0) {
          signedAttrsConstruct = v;
        }
      }

      if (!signatureOctet) return false;
      const sigBytes = Buffer.from(signatureOctet.valueBlock.valueHexView);

      if (signedAttrsConstruct) {
        const attrsSet = new asn1js.Set({ value: signedAttrsConstruct.valueBlock.value });
        const attrsDer = Buffer.from(attrsSet.toBER(false));

        const verifier = crypto.createVerify('SHA256');
        verifier.update(attrsDer);
        const pubKey = crypto.createPublicKey(this.certificatePem);
        return verifier.verify(pubKey, sigBytes);
      } else {
        const verifier = crypto.createVerify('SHA256');
        verifier.update(tstInfoBytes);
        const pubKey = crypto.createPublicKey(this.certificatePem);
        return verifier.verify(pubKey, sigBytes);
      }
    } catch {
      return false;
    }
  }

  // ==========================================================================
  // KEY GENERATION
  // ==========================================================================

  private generateTsaKeyPair(): void {
    const { privateKey } = crypto.generateKeyPairSync('rsa', {
      modulusLength: 4096,
      publicKeyEncoding: { type: 'spki', format: 'pem' },
      privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
    });
    fs.writeFileSync(this.config.privateKeyPath, privateKey, { mode: 0o600 });

    const opensslConfig = [
      '[req]', 'default_bits = 4096', 'prompt = no', 'default_md = sha256',
      'distinguished_name = dn', 'x509_extensions = v3_tsa', '',
      '[dn]',
      'CN = PlenumNET Time-Stamping Authority',
      'O = Capomastro Holdings Ltd.',
      'OU = Applied Physics Division',
      'L = Alberta', 'C = CA', '',
      '[v3_tsa]',
      'basicConstraints = CA:FALSE',
      'keyUsage = critical, digitalSignature, nonRepudiation',
      'extendedKeyUsage = critical, timeStamping',
      'subjectKeyIdentifier = hash',
      'authorityKeyIdentifier = keyid,issuer',
    ].join('\n');

    const configPath = path.join(this.config.keysDirectory, 'tsa-openssl.cnf');
    fs.writeFileSync(configPath, opensslConfig);
    execSync(
      `openssl req -new -x509 -key "${this.config.privateKeyPath}" ` +
      `-out "${this.config.certificatePath}" -days 1826 -config "${configPath}"`,
      { stdio: 'pipe' },
    );
    fs.copyFileSync(this.config.certificatePath, this.config.chainPath);
    try { fs.unlinkSync(configPath); } catch { /* */ }
    console.log('TSA key pair generated: RSA-4096, EKU=critical,timeStamping');
  }

  // ==========================================================================
  // UTILITIES
  // ==========================================================================

  private isCertificateValid(): boolean {
    if (!this.certificateParsed) return false;
    try { return new Date() < new Date(this.certificateParsed.validTo); }
    catch { return false; }
  }

  private nextSerialNumber(): bigint {
    this.serialCounter += BigInt(1);
    const p = path.join(this.config.keysDirectory, 'serial-counter.txt');
    fs.writeFileSync(p, this.serialCounter.toString());
    return this.serialCounter;
  }

  private formatGeneralizedTime(isoTimestamp: string): string {
    const d = new Date(isoTimestamp);
    const pad = (n: number, w: number = 2) => n.toString().padStart(w, '0');
    const base = `${d.getUTCFullYear()}${pad(d.getUTCMonth() + 1)}${pad(d.getUTCDate())}` +
      `${pad(d.getUTCHours())}${pad(d.getUTCMinutes())}${pad(d.getUTCSeconds())}`;
    const ms = d.getUTCMilliseconds();
    return ms > 0 ? `${base}.${pad(ms, 3)}Z` : `${base}Z`;
  }

  private parseGeneralizedTime(gt: string): Date {
    const base = gt.replace('Z', '');
    const year = parseInt(base.substring(0, 4));
    const month = parseInt(base.substring(4, 6)) - 1;
    const day = parseInt(base.substring(6, 8));
    const hour = parseInt(base.substring(8, 10));
    const min = parseInt(base.substring(10, 12));
    const sec = parseInt(base.substring(12, 14));
    let ms = 0;
    if (base.includes('.')) {
      const frac = base.split('.')[1];
      ms = parseInt(frac.padEnd(3, '0').substring(0, 3));
    }
    return new Date(Date.UTC(year, month, day, hour, min, sec, ms));
  }

  private getIssuerAndSerial(): asn1js.Sequence {
    if (!this.certificateParsed) throw new Error('Certificate not loaded');

    const certAsn1 = asn1js.fromBER(new Uint8Array(this.certificate!).buffer);
    const certSeq = certAsn1.result as asn1js.Sequence;
    const tbsCert = certSeq.valueBlock.value[0] as asn1js.Sequence;
    const tbsValues = tbsCert.valueBlock.value;

    let issuer: asn1js.BaseBlock | null = null;
    let serial: asn1js.BaseBlock | null = null;

    let idx = 0;
    if (tbsValues[0] instanceof asn1js.Constructed && tbsValues[0].idBlock.tagClass === 3) {
      idx = 1;
    }
    serial = tbsValues[idx];
    idx++;
    idx++;
    issuer = tbsValues[idx];

    return new asn1js.Sequence({
      value: [issuer!, serial!],
    });
  }

  public getTokenBySerial(serialNumber: string): TsaTokenRecord | null {
    return this.serialIndex.get(serialNumber) || null;
  }

  private logToken(record: TsaTokenRecord): void {
    this.tokenLog.push(record);
    this.serialIndex.set(record.serialNumber, record);

    if (this.tokenLog.length > 100000) {
      this.tokenLog = this.tokenLog.slice(-50000);
      this.serialIndex.clear();
      for (const r of this.tokenLog) {
        this.serialIndex.set(r.serialNumber, r);
      }
    }

    const logPath = path.join(this.config.keysDirectory, 'token-audit.jsonl');
    fs.appendFileSync(logPath, JSON.stringify(record) + '\n');
  }
}
