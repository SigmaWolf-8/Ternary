/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PLENUMNET RFC 3161 TSA API ROUTES
 * Location:   server/routes/tsa.ts
 *
 * 8 endpoints under /api/tsa/*
 * Kong service #21: plenumnet-tsa
 */

import { Router, Request, Response, NextFunction } from 'express';
import { TsaService, TSA_POLICIES } from '../services/tsa-service';

interface AuthenticatedRequest extends Request {
  auth?: { appId: string; role: 'admin' | 'app' | 'readonly' };
}

function requireAuth(minRole: 'readonly' | 'app' | 'admin') {
  const hierarchy = { readonly: 0, app: 1, admin: 2 };
  return (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    const authHeader = req.headers.authorization;
    if (!authHeader) return res.status(401).json({ error: 'Authentication required' });
    try {
      const token = authHeader.replace('Bearer ', '');
      const payload = JSON.parse(Buffer.from(token.split('.')[1] || token, 'base64').toString());
      const auth = { appId: payload.appId || 'unknown', role: (payload.role || 'readonly') as 'admin' | 'app' | 'readonly' };
      if (hierarchy[auth.role] < hierarchy[minRole]) return res.status(403).json({ error: 'Insufficient permissions' });
      req.auth = auth;
      next();
    } catch { return res.status(401).json({ error: 'Invalid authentication token' }); }
  };
}

export function createTsaRoutes(service: TsaService): Router {
  const router = Router();

  router.post('/timestamp', requireAuth('app'),
    async (req: AuthenticatedRequest, res: Response) => {
      if (req.headers['content-type'] !== 'application/timestamp-query') {
        return res.status(415).json({
          error: 'Unsupported Media Type',
          message: 'RFC 3161 requires Content-Type: application/timestamp-query',
          hint: 'For JSON, use POST /api/tsa/timestamp/json',
        });
      }
      try {
        const derReq = Buffer.isBuffer(req.body) ? req.body : Buffer.from(req.body);
        const derResp = await service.processTimestampRequest(derReq, req.ip || 'unknown');
        res.status(200).contentType('application/timestamp-reply').send(derResp);
      } catch (error) { res.status(500).json({ error: (error as Error).message }); }
    },
  );

  router.post('/timestamp/json', requireAuth('app'),
    async (req: AuthenticatedRequest, res: Response) => {
      try {
        const { hash, algorithm, policy, nonce, includeChain } = req.body;
        if (!hash || !algorithm) {
          return res.status(400).json({
            error: 'Missing required fields',
            required: { hash: 'hex-encoded hash', algorithm: 'sha256|sha384|sha512|sha3-256|sha3-384|sha3-512' },
            optional: {
              policy: `OID. DEFAULT=${TSA_POLICIES.DEFAULT}, COMPLY=${TSA_POLICIES.COMPLY}, FORENSICS=${TSA_POLICIES.FORENSICS}, SENTINEL=${TSA_POLICIES.SENTINEL}, SECURE=${TSA_POLICIES.SECURE}`,
              nonce: 'hex-encoded nonce for replay protection',
              includeChain: 'boolean — include TSA certificate chain',
            },
          });
        }
        const result = await service.processJsonRequest(
          { hash, algorithm, policy, nonce, includeChain }, req.ip || 'unknown',
        );
        res.status(200).json({ success: true, ...result, callerApp: (req as AuthenticatedRequest).auth?.appId });
      } catch (error) { res.status(422).json({ success: false, error: (error as Error).message }); }
    },
  );

  router.post('/verify', async (req: Request, res: Response) => {
    try {
      let derToken: Buffer;
      if (req.headers['content-type'] === 'application/timestamp-reply') {
        derToken = Buffer.isBuffer(req.body) ? req.body : Buffer.from(req.body);
      } else if (req.body?.token) {
        derToken = Buffer.from(req.body.token, 'base64');
      } else {
        return res.status(400).json({ error: 'Provide binary timestamp-reply or JSON { token: "base64" }' });
      }
      res.status(200).json(await service.verifyToken(derToken));
    } catch (error) { res.status(422).json({ valid: false, error: (error as Error).message }); }
  });

  router.get('/certificate', (_req: Request, res: Response) => {
    try { res.status(200).json(service.getTsaCertificate()); }
    catch (error) { res.status(500).json({ error: (error as Error).message }); }
  });

  router.get('/certificate/download', (_req: Request, res: Response) => {
    try {
      const cert = service.getTsaCertificate();
      res.status(200)
        .set('Content-Type', 'application/x-pem-file')
        .set('Content-Disposition', 'attachment; filename="plenumnet-tsa.pem"')
        .send(cert.certificate);
    } catch (error) { res.status(500).json({ error: (error as Error).message }); }
  });

  router.get('/tokens', requireAuth('admin'),
    (req: AuthenticatedRequest, res: Response) => {
      res.status(200).json(service.queryTokenLog({
        since: req.query.since as string, until: req.query.until as string,
        hashAlgorithm: req.query.hashAlgorithm as string,
        policyTier: req.query.policyTier as string,
        limit: req.query.limit ? parseInt(req.query.limit as string) : undefined,
      }));
    },
  );

  router.get('/policy', (_req: Request, res: Response) => {
    res.status(200).json(service.getPolicyInfo());
  });

  router.get('/health', async (_req: Request, res: Response) => {
    const health = await service.getHealth();
    res.status(health.status === 'unhealthy' ? 503 : 200).json(health);
  });

  return router;
}
