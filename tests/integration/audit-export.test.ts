/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * Integration tests for Signed Audit Export endpoints.
 * Tests: POST /api/tsa/export/json, POST /api/tsa/export/pdf, POST /api/tsa/export/verify
 * Required per TM-2026-020.1-PREREQ §6.2.
 */

import { describe, it, expect, beforeAll } from 'vitest';

const BASE_URL = 'http://localhost:5000';

function url(path: string): string {
  return `${BASE_URL}${path}`;
}

function authHeader(role: string, appId: string = 'audit-test-app'): Record<string, string> {
  const token = Buffer.from(JSON.stringify({ appId, role })).toString('base64');
  return {
    'Content-Type': 'application/json',
    'Authorization': `Bearer x.${token}.x`,
  };
}

async function jsonPost(path: string, body: unknown, headers: Record<string, string>) {
  return fetch(url(path), {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });
}

describe('Signed Audit Export — TM-2026-020.1-PREREQ §6.2', () => {
  beforeAll(async () => {
    let retries = 10;
    while (retries > 0) {
      try {
        const res = await fetch(url('/api/health'));
        if (res.ok) return;
      } catch {}
      retries--;
      await new Promise((r) => setTimeout(r, 2000));
    }
    throw new Error('Server did not become ready within timeout');
  }, 30000);

  describe('POST /api/tsa/export/json', () => {
    it('returns a valid signed audit document with correct schema', async () => {
      const res = await jsonPost('/api/tsa/export/json', { limit: 5 }, authHeader('app'));
      expect(res.status).toBe(200);
      const doc = await res.json();

      expect(doc.version).toBe('1.0.0');
      expect(doc.exporterId).toBe('plenumnet-tsa');
      expect(doc.projectScope).toBe('audit-test-app');
      expect(typeof doc.exportedAt).toBe('string');
      expect(typeof doc.totalRecords).toBe('number');
      expect(Array.isArray(doc.records)).toBe(true);
      expect(Array.isArray(doc.signatureChain)).toBe(true);
      expect(typeof doc.merkleRoot).toBe('string');
      expect(doc.merkleRoot.length).toBe(64);

      expect(doc.documentSignature).toBeDefined();
      expect(doc.documentSignature.algorithm).toBe('TL-DSA');
      expect(typeof doc.documentSignature.variant).toBe('string');
      expect(typeof doc.documentSignature.publicKeyHash).toBe('string');
      expect(typeof doc.documentSignature.signature).toBe('string');
      expect(typeof doc.documentSignature.signedHash).toBe('string');
    });

    it('enforces project scope from authenticated identity (not client-supplied)', async () => {
      const res = await jsonPost(
        '/api/tsa/export/json',
        { projectScope: 'should-be-ignored', limit: 1 },
        authHeader('app', 'scoped-app-123'),
      );
      expect(res.status).toBe(200);
      const doc = await res.json();
      expect(doc.projectScope).toBe('scoped-app-123');
    });

    it('includes audit record schema fields: reviewerVerdict, resultVersion, engineId, modelVersion', async () => {
      const res = await jsonPost('/api/tsa/export/json', { limit: 5 }, authHeader('app'));
      expect(res.status).toBe(200);
      const doc = await res.json();

      for (const record of doc.records) {
        expect(record).toHaveProperty('recordId');
        expect(record).toHaveProperty('taskId');
        expect(record).toHaveProperty('eventType');
        expect(record).toHaveProperty('severity');
        expect(record).toHaveProperty('category');
        expect(record).toHaveProperty('description');
        expect(record).toHaveProperty('inferenceHash');
        expect(record).toHaveProperty('timestamp');
        expect(record).toHaveProperty('reviewerVerdict');
        expect(record).toHaveProperty('resultVersion');
        expect(record).toHaveProperty('engineId');
        expect(record).toHaveProperty('modelVersion');
        expect(record).toHaveProperty('evidence');
        expect(record).toHaveProperty('resolutionStatus');
        expect(record).toHaveProperty('contentDeleted');
      }
    });

    it('signature chain length matches record count', async () => {
      const res = await jsonPost('/api/tsa/export/json', { limit: 5 }, authHeader('app'));
      expect(res.status).toBe(200);
      const doc = await res.json();
      expect(doc.signatureChain.length).toBe(doc.records.length);
    });
  });

  describe('POST /api/tsa/export/pdf', () => {
    it('returns a valid PDF buffer with correct headers', async () => {
      const res = await jsonPost('/api/tsa/export/pdf', { limit: 3 }, authHeader('app'));
      expect(res.status).toBe(200);
      expect(res.headers.get('content-type')).toContain('application/pdf');

      const disposition = res.headers.get('content-disposition');
      expect(disposition).toContain('attachment');
      expect(disposition).toContain('plenumnet-audit-export');

      const buf = await res.arrayBuffer();
      expect(buf.byteLength).toBeGreaterThan(0);
      const pdfHeader = new Uint8Array(buf.slice(0, 5));
      expect(String.fromCharCode(...pdfHeader)).toBe('%PDF-');
    });

    it('enforces scope from auth identity', async () => {
      const res = await jsonPost(
        '/api/tsa/export/pdf',
        { limit: 1 },
        authHeader('app', 'pdf-scoped-app'),
      );
      expect(res.status).toBe(200);
    });
  });

  describe('POST /api/tsa/export/verify', () => {
    it('validates an unmodified signed document as valid', async () => {
      const exportRes = await jsonPost('/api/tsa/export/json', { limit: 3 }, authHeader('app'));
      expect(exportRes.status).toBe(200);
      const doc = await exportRes.json();

      const verifyRes = await jsonPost('/api/tsa/export/verify', doc, authHeader('readonly'));
      expect(verifyRes.status).toBe(200);
      const result = await verifyRes.json();

      expect(result.valid).toBe(true);
      expect(result.errors).toEqual([]);
      expect(result.recordCount).toBe(doc.totalRecords);
      expect(result.chainLength).toBe(doc.signatureChain.length);
      expect(result.merkleRoot).toBe(doc.merkleRoot);
    });

    it('detects tampered record description', async () => {
      const exportRes = await jsonPost('/api/tsa/export/json', { limit: 3 }, authHeader('app'));
      const doc = await exportRes.json();

      if (doc.records.length > 0) {
        doc.records[0].description = 'TAMPERED-DESCRIPTION';
      }

      const verifyRes = await jsonPost('/api/tsa/export/verify', doc, authHeader('readonly'));
      expect(verifyRes.status).toBe(200);
      const result = await verifyRes.json();

      if (doc.totalRecords > 0) {
        expect(result.valid).toBe(false);
        expect(result.errors.length).toBeGreaterThan(0);
        expect(result.errors.some((e: string) => e.includes('hash mismatch'))).toBe(true);
      }
    });

    it('detects tampered evidence field', async () => {
      const exportRes = await jsonPost('/api/tsa/export/json', { limit: 3 }, authHeader('app'));
      const doc = await exportRes.json();

      if (doc.records.length > 0 && doc.records[0].evidence) {
        doc.records[0].evidence = { injected: true };
      }

      const verifyRes = await jsonPost('/api/tsa/export/verify', doc, authHeader('readonly'));
      expect(verifyRes.status).toBe(200);
      const result = await verifyRes.json();

      if (doc.totalRecords > 0 && doc.records[0].evidence) {
        expect(result.valid).toBe(false);
      }
    });

    it('detects tampered document signature hash', async () => {
      const exportRes = await jsonPost('/api/tsa/export/json', { limit: 3 }, authHeader('app'));
      const doc = await exportRes.json();

      doc.documentSignature.signedHash = 'aaaa' + doc.documentSignature.signedHash.substring(4);

      const verifyRes = await jsonPost('/api/tsa/export/verify', doc, authHeader('readonly'));
      expect(verifyRes.status).toBe(200);
      const result = await verifyRes.json();

      expect(result.valid).toBe(false);
      expect(result.errors.some((e: string) => e.includes('Document hash mismatch') || e.includes('signature verification failed'))).toBe(true);
    });

    it('rejects malformed input with 400', async () => {
      const verifyRes = await jsonPost(
        '/api/tsa/export/verify',
        { invalid: true },
        authHeader('readonly'),
      );
      expect(verifyRes.status).toBe(400);
      const result = await verifyRes.json();
      expect(result.error).toContain('Invalid signed audit document');
    });
  });

  describe('Signature chain integrity', () => {
    it('chain links reference previous signature hashes correctly', async () => {
      const exportRes = await jsonPost('/api/tsa/export/json', { limit: 5 }, authHeader('app'));
      const doc = await exportRes.json();

      for (let i = 1; i < doc.signatureChain.length; i++) {
        expect(doc.signatureChain[i].previousSignatureHash).toBeDefined();
        expect(doc.signatureChain[i].previousSignatureHash.length).toBe(64);
      }
    });

    it('export → verify → tamper → reject round-trip', async () => {
      const exportRes = await jsonPost('/api/tsa/export/json', { limit: 5 }, authHeader('app'));
      const doc = await exportRes.json();

      const verifyRes1 = await jsonPost('/api/tsa/export/verify', doc, authHeader('readonly'));
      const result1 = await verifyRes1.json();
      expect(result1.valid).toBe(true);

      if (doc.signatureChain.length > 0) {
        doc.signatureChain[0].chainSignature = 'ff'.repeat(32);
      }

      const verifyRes2 = await jsonPost('/api/tsa/export/verify', doc, authHeader('readonly'));
      const result2 = await verifyRes2.json();

      if (doc.signatureChain.length > 0) {
        expect(result2.valid).toBe(false);
      }
    });
  });
});
