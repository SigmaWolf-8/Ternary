/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * CAPABILITY TOKEN API ROUTES — Phases 2 + 3
 * @version 2.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/routes/capabilities.ts
 *
 * Phase 2: HPTP-bound expiration demo endpoint
 * Phase 3: HMAC-chained delegation endpoints
 */

import type { Express, Request, Response } from 'express';
import { z } from 'zod';
import { capabilityService } from '../services/capability-service';
import { createLogger } from '../logger';

const log = createLogger('capabilities');

const constraintSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('recipient_domain'), value: z.string() }),
  z.object({ type: z.literal('vault_id'), value: z.string() }),
  z.object({ type: z.literal('template'), value: z.string() }),
  z.object({ type: z.literal('max_uses'), value: z.number().int().min(1) }),
  z.object({ type: z.literal('ip_range'), value: z.string() }),
  z.object({ type: z.literal('geo_country'), value: z.array(z.string()) }),
  z.object({ type: z.literal('document_id'), value: z.string() }),
  z.object({ type: z.literal('project_id'), value: z.string() }),
]);

const issueRequestSchema = z.object({
  subject: z.string().min(1),
  capabilities: z.array(z.object({
    resource: z.string().min(1),
    constraints: z.array(constraintSchema).default([]),
    ttl_seconds: z.number().int().min(1).max(86400).default(3600),
  })).min(1),
});

const validateRequestSchema = z.object({
  signed_token: z.object({
    token: z.object({
      sub: z.string(),
      cap: z.array(z.object({
        res: z.string(),
        constraints: z.array(constraintSchema),
        exp: z.string(),
      })),
      role: z.string().optional(),
      iat_hptp: z.string(),
      iss: z.literal('plenumnet.cap'),
      jti: z.string(),
      crv: z.string(),
    }),
    signature: z.string(),
    algorithm: z.literal('TL-DSA'),
  }),
  resource: z.string().min(1),
  context: z.object({
    recipient: z.string().optional(),
    vault_id: z.string().optional(),
    template: z.string().optional(),
    usage_count: z.number().optional(),
    source_ip: z.string().optional(),
    source_country: z.string().optional(),
    document_id: z.string().optional(),
    project_id: z.string().optional(),
  }).default({}),
});

const delegateRequestSchema = z.object({
  parent_token: z.object({
    token: z.object({
      sub: z.string(),
      cap: z.array(z.object({
        res: z.string(),
        constraints: z.array(constraintSchema),
        exp: z.string(),
      })),
      role: z.string().optional(),
      iat_hptp: z.string(),
      iss: z.literal('plenumnet.cap'),
      jti: z.string(),
      crv: z.string(),
    }),
    signature: z.string(),
    algorithm: z.literal('TL-DSA'),
  }),
  new_subject: z.string().min(1),
  attenuations: z.array(constraintSchema).min(1),
  ttl_seconds: z.number().int().min(1).max(86400).optional(),
});

const delegateChainRequestSchema = z.object({
  delegated_token: z.object({
    root_signature: z.string(),
    root_algorithm: z.literal('TL-DSA'),
    token: z.object({
      sub: z.string(),
      cap: z.array(z.object({
        res: z.string(),
        constraints: z.array(constraintSchema),
        exp: z.string(),
      })),
      role: z.string().optional(),
      iat_hptp: z.string(),
      iss: z.literal('plenumnet.cap'),
      jti: z.string(),
      crv: z.string(),
    }),
    delegation_chain: z.array(z.object({
      constraint: constraintSchema,
      hmac: z.string(),
    })),
    chain_depth: z.number(),
    parent_jti: z.string(),
    parent_token_hash: z.string(),
  }),
  new_subject: z.string().min(1),
  attenuations: z.array(constraintSchema).min(1),
  ttl_seconds: z.number().int().min(1).max(86400).optional(),
});

export function registerCapabilityRoutes(app: Express): void {
  log.info('Capability routes registered — Phase 2 (HPTP expiration) + Phase 3 (HMAC delegation)');

  app.get('/api/capabilities/demo/expiration', (_req: Request, res: Response) => {
    try {
      const result = capabilityService.runExpirationDemo();
      res.json({
        success: true,
        phase: 2,
        title: 'HPTP-Bound Capability Expiration Demo',
        description: 'Demonstrates nanosecond-precise capability expiry using HPTP timing engine. No client clock consulted — all expiration checked against authoritative HPTP.',
        ...result,
      });
    } catch (err) {
      log.error('Expiration demo failed:', err);
      res.status(500).json({ success: false, error: 'Demo execution failed' });
    }
  });

  app.get('/api/capabilities/demo/delegation', (_req: Request, res: Response) => {
    try {
      const result = capabilityService.runDelegationDemo();
      res.json({
        success: true,
        phase: 3,
        title: 'HMAC-Chained Delegation Demo',
        description: 'Demonstrates macaroon-style capability delegation: TL-DSA root → HMAC attenuation chain. Authority can only diminish through the chain, never grow. Each caveat is cryptographically sealed.',
        ...result,
      });
    } catch (err) {
      log.error('Delegation demo failed:', err);
      res.status(500).json({ success: false, error: 'Demo execution failed' });
    }
  });

  app.post('/api/capabilities/issue', (req: Request, res: Response) => {
    try {
      const parsed = issueRequestSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const { subject, capabilities } = parsed.data;
      const resources = capabilities.map(c => ({
        res: c.resource,
        constraints: c.constraints,
        ttlSeconds: c.ttl_seconds,
      }));

      const result = capabilityService.issueCapabilityToken(
        subject,
        resources,
        req.ip || req.socket.remoteAddress,
      );

      res.json({
        success: true,
        signed_token: result.signedToken,
        expiration: result.expiration,
        audit_event_hash: result.audit_event_hash,
      });
    } catch (err) {
      log.error('Issue capability failed:', err);
      res.status(500).json({ success: false, error: 'Issuance failed' });
    }
  });

  app.post('/api/capabilities/validate', (req: Request, res: Response) => {
    try {
      const parsed = validateRequestSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const { signed_token, resource, context } = parsed.data;
      const result = capabilityService.validateCapability(
        signed_token as any,
        resource,
        context,
        req.ip || req.socket.remoteAddress,
      );

      res.json({ success: true, validation: result });
    } catch (err) {
      log.error('Validate capability failed:', err);
      res.status(500).json({ success: false, error: 'Validation failed' });
    }
  });

  app.post('/api/capabilities/delegate', (req: Request, res: Response) => {
    try {
      const parsed = delegateRequestSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const { parent_token, new_subject, attenuations, ttl_seconds } = parsed.data;
      const result = capabilityService.delegateCapability(
        parent_token as any,
        new_subject,
        attenuations,
        ttl_seconds,
        req.ip || req.socket.remoteAddress,
      );

      res.json({
        success: true,
        delegated_token: result.delegated_token,
        expiration: result.expiration,
        chain_depth: result.chain_depth,
        audit_event_hash: result.audit_event_hash,
      });
    } catch (err) {
      log.error('Delegate capability failed:', err);
      res.status(500).json({ success: false, error: 'Delegation failed' });
    }
  });

  app.post('/api/capabilities/delegate/chain', (req: Request, res: Response) => {
    try {
      const parsed = delegateChainRequestSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const { delegated_token, new_subject, attenuations, ttl_seconds } = parsed.data;
      const result = capabilityService.delegateCapability(
        delegated_token as any,
        new_subject,
        attenuations,
        ttl_seconds,
        req.ip || req.socket.remoteAddress,
      );

      res.json({
        success: true,
        delegated_token: result.delegated_token,
        expiration: result.expiration,
        chain_depth: result.chain_depth,
        audit_event_hash: result.audit_event_hash,
      });
    } catch (err) {
      log.error('Chain delegate failed:', err);
      res.status(500).json({ success: false, error: 'Chain delegation failed' });
    }
  });

  app.post('/api/capabilities/verify-chain', (req: Request, res: Response) => {
    try {
      const delegatedToken = req.body;
      if (!delegatedToken?.delegation_chain || !delegatedToken?.root_signature) {
        return res.status(400).json({ success: false, error: 'Invalid delegated token' });
      }

      const result = capabilityService.verifyDelegationChain(delegatedToken);
      res.json({ success: true, verification: result });
    } catch (err) {
      log.error('Verify chain failed:', err);
      res.status(500).json({ success: false, error: 'Chain verification failed' });
    }
  });

  app.get('/api/capabilities/audit', (_req: Request, res: Response) => {
    try {
      const stats = capabilityService.getAuditStats();
      res.json({
        success: true,
        audit: {
          ...stats,
          hash_algorithm: 'SHA3-256',
          chain_type: 'Merkle',
          persistence: 'server/crypto/tsa-keys/capability-audit.jsonl',
        },
      });
    } catch (err) {
      log.error('Audit stats failed:', err);
      res.status(500).json({ success: false, error: 'Audit query failed' });
    }
  });

  app.get('/api/capabilities/status', (_req: Request, res: Response) => {
    const hptpNs = capabilityService.getHptpNow();
    res.json({
      success: true,
      service: 'PlenumNET Capability Token Service',
      version: '2.0.0',
      phases: {
        phase_1: { status: 'complete', description: 'Typed constraint registry + capability token schema + audit events' },
        phase_2: { status: 'complete', description: 'HPTP-bound expiration — timing engine wired into validation path' },
        phase_3: { status: 'complete', description: 'HMAC-chained delegation — macaroon-style attenuation with TL-DSA roots' },
        phase_4: { status: 'planned', description: 'Hardware-bound capabilities + HPTP challenge-response + single-use chains' },
        phase_5: { status: 'planned', description: 'RFC 3161 capability certificates' },
        phase_6: { status: 'planned', description: 'Inter-service capability mesh' },
      },
      current_hptp_ns: hptpNs,
      signing_algorithm: 'TL-DSA',
      constraint_registry_version: '1.0',
      supported_constraints: [
        'recipient_domain', 'vault_id', 'template', 'max_uses',
        'ip_range', 'geo_country', 'document_id', 'project_id',
      ],
      endpoints: {
        demo_expiration: 'GET /api/capabilities/demo/expiration',
        demo_delegation: 'GET /api/capabilities/demo/delegation',
        issue: 'POST /api/capabilities/issue',
        validate: 'POST /api/capabilities/validate',
        delegate: 'POST /api/capabilities/delegate',
        delegate_chain: 'POST /api/capabilities/delegate/chain',
        verify_chain: 'POST /api/capabilities/verify-chain',
        audit: 'GET /api/capabilities/audit',
        status: 'GET /api/capabilities/status',
      },
    });
  });
}
