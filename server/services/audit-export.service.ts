/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * SIGNED AUDIT EXPORT SERVICE
 * @version 1.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/services/audit-export.service.ts
 *
 * Exports audit records as TL-DSA signed JSON and PDF documents.
 * Each export includes a tamper-evident signature chain: every record
 * signs over the previous record's signature hash, linking them
 * cryptographically.
 *
 * Required per TM-2026-020.1-PREREQ §6.2.
 */

import crypto from 'crypto';
import PDFDocument from 'pdfkit';
import { db } from '../db';
import { securityAuditLog } from '@shared/schema';
import { eq, desc, and, gte, lte, sql } from 'drizzle-orm';
import {
  signNative as tlDsaSign,
  verifyNative as tlDsaVerify,
  type TlDsaKeyPair,
  type TlDsaVariant,
} from '../crypto/tl-dsa-bridge';
import { getTlDsaTsaKeyPair } from '../crypto/key-management';

interface EvidencePayload {
  taskId?: string;
  reviewerVerdict?: string;
  resultVersion?: string;
  engineId?: string;
  modelVersion?: string;
  [key: string]: unknown;
}

export interface AuditRecord {
  recordId: string;
  taskId: string | null;
  eventType: string;
  severity: string;
  category: string;
  description: string;
  actor: string | null;
  affectedComponent: string | null;
  inferenceHash: string;
  timestamp: string;
  reviewerVerdict: string | null;
  resultVersion: string | null;
  engineId: string | null;
  modelVersion: string | null;
  evidence: Record<string, unknown> | null;
  resolutionStatus: string;
  contentDeleted: boolean;
}

export interface SignedAuditDocument {
  version: '1.0.0';
  exportedAt: string;
  exporterId: string;
  projectScope: string;
  totalRecords: number;
  records: AuditRecord[];
  signatureChain: SignatureChainEntry[];
  documentSignature: {
    algorithm: string;
    variant: TlDsaVariant;
    publicKeyHex: string;
    publicKeyHash: string;
    signature: string;
    signedHash: string;
  };
  retentionPolicy: {
    deletionProhibited: true;
    contentDeletionMarker: 'content_deleted';
    metadataRetained: true;
    description: string;
  };
  merkleRoot: string;
}

export interface SignatureChainEntry {
  recordId: string;
  recordHash: string;
  previousSignatureHash: string;
  chainSignature: string;
}

const GENESIS_HASH = crypto.createHash('sha3-256').update('audit-chain-genesis').digest('hex');

export async function tombstoneAuditContent(recordId: number): Promise<boolean> {
  const [updated] = await db
    .update(securityAuditLog)
    .set({
      description: '[CONTENT DELETED — retention policy]',
      evidence: { redacted: true, reason: 'content_deleted' },
      resolutionStatus: 'content_deleted',
    })
    .where(eq(securityAuditLog.id, recordId))
    .returning({ id: securityAuditLog.id });
  return !!updated;
}

export async function deleteAuditRecord(_recordId: number): Promise<never> {
  throw new Error(
    'Audit records cannot be deleted. Use tombstoneAuditContent() to redact ' +
    'content while preserving the record, its metadata, hashes, and signatures ' +
    'per TM-2026-020.1-PREREQ §6.2 retention policy.'
  );
}

function canonicalJsonStringify(obj: unknown): string {
  if (obj === null || obj === undefined) return JSON.stringify(obj);
  if (typeof obj !== 'object') return JSON.stringify(obj);
  if (Array.isArray(obj)) {
    return '[' + obj.map(canonicalJsonStringify).join(',') + ']';
  }
  const sortedKeys = Object.keys(obj as Record<string, unknown>).sort();
  const pairs = sortedKeys.map(
    (k) => `${JSON.stringify(k)}:${canonicalJsonStringify((obj as Record<string, unknown>)[k])}`
  );
  return '{' + pairs.join(',') + '}';
}

function computeRecordHash(record: AuditRecord): string {
  const canonical = canonicalJsonStringify(record);
  return crypto.createHash('sha3-256').update(canonical).digest('hex');
}

function computeInferenceHash(data: string): string {
  const hash = crypto.createHash('sha3-256').update(data).digest();
  return hash.subarray(0, 4).toString('hex');
}

function computeMerkleRoot(hashes: string[]): string {
  if (hashes.length === 0) {
    return crypto.createHash('sha3-256').update('empty-audit').digest('hex');
  }
  let level = [...hashes];
  while (level.length > 1) {
    const next: string[] = [];
    for (let i = 0; i < level.length; i += 2) {
      const left = level[i];
      const right = level[i + 1] || left;
      next.push(crypto.createHash('sha3-256').update(left + right).digest('hex'));
    }
    level = next;
  }
  return level[0];
}

export async function queryAuditRecords(filters: {
  authenticatedAppId: string;
  since?: string;
  until?: string;
  severity?: string;
  category?: string;
  eventType?: string;
  limit?: number;
}): Promise<AuditRecord[]> {
  const conditions = [];

  conditions.push(eq(securityAuditLog.userId, filters.authenticatedAppId));

  if (filters.since) {
    conditions.push(gte(securityAuditLog.createdAt, new Date(filters.since)));
  }
  if (filters.until) {
    conditions.push(lte(securityAuditLog.createdAt, new Date(filters.until)));
  }
  if (filters.severity) {
    conditions.push(eq(securityAuditLog.severity, filters.severity));
  }
  if (filters.category) {
    conditions.push(eq(securityAuditLog.category, filters.category));
  }
  if (filters.eventType) {
    conditions.push(eq(securityAuditLog.eventType, filters.eventType));
  }

  const rows = await db
    .select()
    .from(securityAuditLog)
    .where(and(...conditions))
    .orderBy(desc(securityAuditLog.createdAt))
    .limit(filters.limit || 500);

  return rows.map((row) => {
    const evidence = row.evidence as EvidencePayload | null;
    const isDeleted = row.resolutionStatus === 'content_deleted';

    return {
      recordId: `SAL-${row.id}`,
      taskId: evidence?.taskId || null,
      eventType: row.eventType,
      severity: row.severity,
      category: row.category,
      description: isDeleted ? '[content deleted per retention policy]' : row.description,
      actor: row.actor,
      affectedComponent: row.affectedComponent,
      inferenceHash: computeInferenceHash(
        `${row.id}|${row.eventType}|${row.severity}|${row.createdAt.toISOString()}`
      ),
      timestamp: row.createdAt.toISOString(),
      reviewerVerdict: evidence?.reviewerVerdict || null,
      resultVersion: evidence?.resultVersion || null,
      engineId: evidence?.engineId || null,
      modelVersion: evidence?.modelVersion || null,
      evidence: isDeleted ? null : (row.evidence as Record<string, unknown> | null),
      resolutionStatus: row.resolutionStatus,
      contentDeleted: isDeleted,
    };
  });
}

function buildSignatureChain(
  records: AuditRecord[],
  keyPair: TlDsaKeyPair,
): SignatureChainEntry[] {
  const chain: SignatureChainEntry[] = [];
  let previousSigHash = GENESIS_HASH;

  for (const record of records) {
    const recordHash = computeRecordHash(record);
    const chainPayload = `${recordHash}|${previousSigHash}`;
    const chainPayloadBuf = Buffer.from(chainPayload, 'utf8');

    const sigResult = tlDsaSign(keyPair.secretKey, chainPayloadBuf, keyPair.variant);
    const chainSignature = sigResult.signature.toString('hex');

    const sigHash = crypto.createHash('sha3-256')
      .update(chainSignature)
      .digest('hex');

    chain.push({
      recordId: record.recordId,
      recordHash,
      previousSignatureHash: previousSigHash,
      chainSignature,
    });

    previousSigHash = sigHash;
  }

  return chain;
}

export async function exportSignedJson(filters: {
  authenticatedAppId: string;
  since?: string;
  until?: string;
  severity?: string;
  category?: string;
  eventType?: string;
  limit?: number;
}): Promise<SignedAuditDocument> {
  const keyPair = getTlDsaTsaKeyPair();
  const records = await queryAuditRecords(filters);
  const signatureChain = buildSignatureChain(records, keyPair);

  const recordHashes = signatureChain.map((e) => e.recordHash);
  const merkleRoot = computeMerkleRoot(recordHashes);

  const retentionPolicy = {
    deletionProhibited: true as const,
    contentDeletionMarker: 'content_deleted' as const,
    metadataRetained: true as const,
    description: 'Audit records are immutable. Content deletion sets content_deleted marker; record metadata, hashes, and signatures are permanently retained.',
  };

  const documentPayload = {
    version: '1.0.0' as const,
    exportedAt: new Date().toISOString(),
    exporterId: 'plenumnet-tsa',
    projectScope: filters.authenticatedAppId,
    totalRecords: records.length,
    merkleRoot,
    records,
    signatureChain,
    retentionPolicy,
  };

  const documentHash = crypto.createHash('sha3-256')
    .update(canonicalJsonStringify(documentPayload))
    .digest('hex');

  const docSigResult = tlDsaSign(
    keyPair.secretKey,
    Buffer.from(documentHash, 'hex'),
    keyPair.variant,
  );

  const publicKeyHash = crypto.createHash('sha3-256')
    .update(keyPair.publicKey)
    .digest('hex');

  return {
    ...documentPayload,
    documentSignature: {
      algorithm: 'TL-DSA',
      variant: keyPair.variant,
      publicKeyHex: keyPair.publicKey.toString('hex'),
      publicKeyHash,
      signature: docSigResult.signature.toString('hex'),
      signedHash: documentHash,
    },
  };
}

export function verifySignedDocument(
  doc: SignedAuditDocument,
  publicKey: Buffer,
): { valid: boolean; errors: string[]; keyTrusted: boolean } {
  const errors: string[] = [];
  const variant = doc.documentSignature.variant;

  const tsaKeyPair = getTlDsaTsaKeyPair();
  const tsaPkHex = tsaKeyPair.publicKey.toString('hex');
  const docPkHex = doc.documentSignature.publicKeyHex;
  const keyTrusted = tsaPkHex === docPkHex;
  if (!keyTrusted) {
    errors.push(
      `Document public key does not match server TSA key. ` +
      `Document key: ${docPkHex.slice(0, 16)}…, Server TSA key: ${tsaPkHex.slice(0, 16)}…`
    );
  }

  for (let i = 0; i < doc.signatureChain.length; i++) {
    const entry = doc.signatureChain[i];
    const record = doc.records.find((r) => r.recordId === entry.recordId);
    if (!record) {
      errors.push(`Record ${entry.recordId} referenced in chain but not found in records`);
      continue;
    }

    const computedHash = computeRecordHash(record);
    if (computedHash !== entry.recordHash) {
      errors.push(`Record ${entry.recordId} hash mismatch: expected ${entry.recordHash}, got ${computedHash}`);
    }

    if (i === 0 && entry.previousSignatureHash !== GENESIS_HASH) {
      errors.push(`First chain entry should reference genesis hash`);
    }
    if (i > 0) {
      const prevSigHash = crypto.createHash('sha3-256')
        .update(doc.signatureChain[i - 1].chainSignature)
        .digest('hex');
      if (entry.previousSignatureHash !== prevSigHash) {
        errors.push(`Chain link ${i} previous hash mismatch`);
      }
    }

    const chainPayload = `${entry.recordHash}|${entry.previousSignatureHash}`;
    const chainPayloadBuf = Buffer.from(chainPayload, 'utf8');
    const chainSigBuf = Buffer.from(entry.chainSignature, 'hex');
    try {
      const chainSigValid = tlDsaVerify(publicKey, chainPayloadBuf, chainSigBuf, variant);
      if (!chainSigValid) {
        errors.push(`Chain link ${i} (${entry.recordId}) TL-DSA signature verification failed`);
      }
    } catch (e) {
      errors.push(`Chain link ${i} (${entry.recordId}) signature verify error: ${(e as Error).message}`);
    }
  }

  const recordHashes = doc.signatureChain.map((e) => e.recordHash);
  const computedMerkle = computeMerkleRoot(recordHashes);
  if (computedMerkle !== doc.merkleRoot) {
    errors.push(`Merkle root mismatch: expected ${doc.merkleRoot}, got ${computedMerkle}`);
  }

  const docPayload = {
    version: doc.version,
    exportedAt: doc.exportedAt,
    exporterId: doc.exporterId,
    projectScope: doc.projectScope,
    totalRecords: doc.totalRecords,
    merkleRoot: doc.merkleRoot,
    records: doc.records,
    signatureChain: doc.signatureChain,
    retentionPolicy: doc.retentionPolicy,
  };
  const reconstructedHash = crypto.createHash('sha3-256')
    .update(canonicalJsonStringify(docPayload))
    .digest('hex');
  if (reconstructedHash !== doc.documentSignature.signedHash) {
    errors.push(`Document hash mismatch: expected ${doc.documentSignature.signedHash}, got ${reconstructedHash}`);
  }

  try {
    const docSigBuf = Buffer.from(doc.documentSignature.signature, 'hex');
    const docHashBuf = Buffer.from(doc.documentSignature.signedHash, 'hex');
    const docSigValid = tlDsaVerify(publicKey, docHashBuf, docSigBuf, variant);
    if (!docSigValid) {
      errors.push('Document-level TL-DSA signature verification failed');
    }
  } catch (e) {
    errors.push(`Document signature verify error: ${(e as Error).message}`);
  }

  return { valid: errors.length === 0, errors, keyTrusted };
}

export async function exportSignedPdf(filters: {
  authenticatedAppId: string;
  since?: string;
  until?: string;
  severity?: string;
  category?: string;
  eventType?: string;
  limit?: number;
}): Promise<Buffer> {
  const signedDoc = await exportSignedJson(filters);

  return new Promise<Buffer>((resolve, reject) => {
    const doc = new PDFDocument({
      size: 'A4',
      margins: { top: 50, bottom: 50, left: 50, right: 50 },
      info: {
        Title: 'PlenumNET Signed Audit Export',
        Author: 'PlenumNET TSA Service',
        Subject: `Audit Export — ${signedDoc.projectScope}`,
        Creator: 'PlenumNET Audit Export Service v1.0.0',
      },
    });

    const chunks: Buffer[] = [];
    doc.on('data', (chunk: Buffer) => chunks.push(chunk));
    doc.on('end', () => resolve(Buffer.concat(chunks)));
    doc.on('error', reject);

    doc.fontSize(18).font('Helvetica-Bold')
      .text('PlenumNET Signed Audit Export', { align: 'center' });
    doc.moveDown(0.5);

    doc.fontSize(10).font('Helvetica')
      .text(`Export Date: ${signedDoc.exportedAt}`, { align: 'center' })
      .text(`Project Scope: ${signedDoc.projectScope}`, { align: 'center' })
      .text(`Total Records: ${signedDoc.totalRecords}`, { align: 'center' })
      .text(`Merkle Root: ${signedDoc.merkleRoot.substring(0, 32)}...`, { align: 'center' });
    doc.moveDown(1);

    doc.moveTo(50, doc.y).lineTo(545, doc.y).stroke();
    doc.moveDown(0.5);

    doc.fontSize(14).font('Helvetica-Bold').text('Document Signature');
    doc.moveDown(0.3);
    doc.fontSize(9).font('Helvetica');
    doc.text(`Algorithm: ${signedDoc.documentSignature.algorithm}`);
    doc.text(`Variant: ${signedDoc.documentSignature.variant}`);
    doc.text(`Public Key (SHA3-256): ${signedDoc.documentSignature.publicKeyHash}`);
    doc.text(`Public Key (hex): ${signedDoc.documentSignature.publicKeyHex}`);
    doc.text(`Document Hash (SHA3-256): ${signedDoc.documentSignature.signedHash}`);
    doc.moveDown(0.3);
    doc.fontSize(8).font('Helvetica-Oblique');
    doc.text('This PDF is a rendering of a signed JSON document. The document hash above is');
    doc.text('computed over the canonical JSON payload. Verify by comparing this hash against');
    doc.text('the JSON export via POST /api/tsa/export/verify.');
    doc.moveDown(1);

    doc.moveTo(50, doc.y).lineTo(545, doc.y).stroke();
    doc.moveDown(0.5);

    doc.fontSize(14).font('Helvetica-Bold').text('Audit Records');
    doc.moveDown(0.5);

    const maxRecords = Math.min(signedDoc.records.length, 50);
    for (let i = 0; i < maxRecords; i++) {
      const record = signedDoc.records[i];
      const chainEntry = signedDoc.signatureChain[i];

      if (doc.y > 700) {
        doc.addPage();
      }

      doc.fontSize(10).font('Helvetica-Bold')
        .text(`#${i + 1} — ${record.recordId}`, { underline: true });
      doc.fontSize(9).font('Helvetica');
      doc.text(`Event: ${record.eventType} | Severity: ${record.severity} | Category: ${record.category}`);
      doc.text(`Time: ${record.timestamp}`);
      doc.text(`Description: ${record.description.substring(0, 200)}${record.description.length > 200 ? '...' : ''}`);
      if (record.actor) doc.text(`Actor: ${record.actor}`);
      if (record.affectedComponent) doc.text(`Component: ${record.affectedComponent}`);
      if (record.reviewerVerdict) doc.text(`Reviewer Verdict: ${record.reviewerVerdict}`);
      if (record.resultVersion) doc.text(`Result Version: ${record.resultVersion}`);
      if (record.engineId) doc.text(`Engine ID: ${record.engineId}`);
      if (record.modelVersion) doc.text(`Model Version: ${record.modelVersion}`);
      doc.text(`TIS-27 Hash: ${record.inferenceHash}`);
      doc.text(`Resolution: ${record.resolutionStatus}`);
      if (record.contentDeleted) doc.text(`Content Status: DELETED (retention policy)`);
      if (chainEntry) {
        doc.text(`Record Hash: ${chainEntry.recordHash.substring(0, 32)}...`);
        doc.text(`Chain Sig: ${chainEntry.chainSignature.substring(0, 48)}...`);
      }
      doc.moveDown(0.5);
    }

    if (signedDoc.records.length > maxRecords) {
      doc.moveDown(0.5);
      doc.fontSize(10).font('Helvetica-Oblique')
        .text(`... and ${signedDoc.records.length - maxRecords} more records (see JSON export for full data)`);
    }

    doc.addPage();
    doc.fontSize(14).font('Helvetica-Bold').text('Signature Chain Verification');
    doc.moveDown(0.5);
    doc.fontSize(9).font('Helvetica');
    doc.text('Each record in this export is linked to the previous record via a TL-DSA');
    doc.text('signature chain. Tampering with any record breaks the chain.');
    doc.moveDown(0.5);
    doc.text(`Genesis Hash: ${GENESIS_HASH.substring(0, 48)}...`);
    doc.text(`Total Chain Links: ${signedDoc.signatureChain.length}`);
    doc.moveDown(0.5);

    const chainPreview = Math.min(signedDoc.signatureChain.length, 20);
    for (let i = 0; i < chainPreview; i++) {
      const entry = signedDoc.signatureChain[i];
      if (doc.y > 720) {
        doc.addPage();
      }
      doc.text(`Link ${i}: ${entry.recordId} → hash=${entry.recordHash.substring(0, 16)}... prev=${entry.previousSignatureHash.substring(0, 16)}...`);
    }

    doc.moveDown(1);
    doc.fontSize(8).font('Helvetica')
      .text('This document was generated by PlenumNET TSA Service. The embedded TL-DSA', { align: 'center' })
      .text('signature provides post-quantum tamper evidence. Verify using the', { align: 'center' })
      .text('POST /api/tsa/export/verify endpoint with the corresponding JSON export.', { align: 'center' });

    doc.moveDown(1);
    doc.fontSize(7).font('Helvetica')
      .text('Full signature (hex-encoded, for independent verification):', { align: 'center' });
    doc.moveDown(0.3);

    const fullSig = signedDoc.documentSignature.signature;
    const sigLines = [];
    for (let i = 0; i < fullSig.length; i += 96) {
      sigLines.push(fullSig.substring(i, i + 96));
    }
    for (const line of sigLines.slice(0, 20)) {
      doc.fontSize(6).font('Courier').text(line, { align: 'center' });
    }
    if (sigLines.length > 20) {
      doc.text(`... (${sigLines.length - 20} more lines)`, { align: 'center' });
    }

    doc.end();
  });
}
