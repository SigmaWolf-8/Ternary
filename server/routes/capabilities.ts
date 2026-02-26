/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * CAPABILITY TOKEN API ROUTES — Phases 1-6
 * @version 4.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/routes/capabilities.ts
 *
 * Phase 1: Typed constraint registry + capability token schema
 * Phase 2: HPTP-bound expiration demo endpoint
 * Phase 3: HMAC-chained delegation endpoints
 * Phase 4: Hardware-bound capabilities + HPTP challenge-response + single-use chains
 * Phase 5: RFC 3161 capability certificates
 * Phase 6: Inter-service capability mesh
 */

import type { Express, Request, Response } from 'express';
import { z } from 'zod';
import { capabilityService } from '../services/capability-service';
import { hardwareBindingEngine } from '../services/capability-hardware-binding';
import { capabilityCertificateService } from '../services/capability-certificates';
import { capabilityMeshService } from '../services/capability-mesh';
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

const hardwareRegisterSchema = z.object({
  device_id: z.string().min(1),
  binding_type: z.enum(['tpm', 'enclave', 'hsm']),
});

const hardwareChallengeSchema = z.object({
  device_id: z.string().min(1),
  window_ns: z.string().optional(),
});

const hardwareVerifySchema = z.object({
  challenge_id: z.string().min(1),
  nonce: z.string().min(1),
  signature: z.string().min(1),
  device_id: z.string().min(1),
  signed_at_hptp_ns: z.string().min(1),
});

const hardwareIssueSchema = z.object({
  subject: z.string().min(1),
  device_id: z.string().min(1),
  capabilities: z.array(z.object({
    resource: z.string().min(1),
    constraints: z.array(constraintSchema).default([]),
    ttl_seconds: z.number().int().min(1).max(86400).default(3600),
  })).min(1),
});

const certificateIssueSchema = z.object({
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
});

const certificateVerifySchema = z.object({
  certificate_id: z.string().min(1),
});

const evidenceChainSchema = z.object({
  certificate_ids: z.array(z.string().min(1)).min(1),
});

const meshRegisterSchema = z.object({
  service_id: z.string().min(1),
  service_name: z.string().min(1),
  capabilities: z.array(z.string().min(1)).min(1),
  endpoint: z.string().min(1),
  metadata: z.record(z.string()).optional(),
});

const meshIssueSchema = z.object({
  from_service: z.string().min(1),
  to_service: z.string().min(1),
  resources: z.array(z.object({
    resource: z.string().min(1),
    constraints: z.array(constraintSchema).default([]),
    ttl_seconds: z.number().int().min(1).max(86400).default(3600),
  })).min(1),
});

const meshPropagateSchema = z.object({
  mesh_token_id: z.string().min(1),
  next_service: z.string().min(1),
  attenuations: z.array(constraintSchema).default([]),
});

const meshValidateSchema = z.object({
  mesh_token_id: z.string().min(1),
  requesting_service: z.string().min(1),
  target_service: z.string().min(1),
});

export function registerCapabilityRoutes(app: Express): void {
  log.info('Capability routes registered — Phases 1-6 (full capability security stack)');

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

  app.get('/api/capabilities/demo/confinement', (_req: Request, res: Response) => {
    try {
      const result = hardwareBindingEngine.runConfinementDemo();
      res.json({
        success: true,
        phase: 4,
        title: 'Hardware-Bound Confinement Demo',
        description: 'Demonstrates the confinement solution: TPM device registration, hardware-bound token issuance, HPTP challenge-response authentication, replay attack rejection, single-use chain creation with first-use-wins enforcement. Copy the token text all you want — without the physical device, it is cryptographic garbage.',
        ...result,
      });
    } catch (err) {
      log.error('Confinement demo failed:', err);
      res.status(500).json({ success: false, error: 'Confinement demo failed' });
    }
  });

  app.get('/api/capabilities/demo/certificates', (_req: Request, res: Response) => {
    try {
      const result = capabilityCertificateService.runCertificateDemo();
      res.json({
        success: true,
        phase: 5,
        title: 'RFC 3161 Capability Certificates Demo',
        description: 'Demonstrates court-admissible capability certificates: RFC 3161 timestamping, dual TL-DSA + RSA-4096 signing, Merkle proof assembly, evidence chain creation, certificate revocation, and tamper detection. Every capability event is provably timestamped.',
        ...result,
      });
    } catch (err) {
      log.error('Certificate demo failed:', err);
      res.status(500).json({ success: false, error: 'Certificate demo failed' });
    }
  });

  app.get('/api/capabilities/demo/mesh', (_req: Request, res: Response) => {
    try {
      const result = capabilityMeshService.runMeshDemo();
      res.json({
        success: true,
        phase: 6,
        title: 'Inter-Service Capability Mesh Demo',
        description: 'Demonstrates the capability mesh: service registration, service-to-service capability issuance, capability propagation with per-hop attenuation, mesh validation, service discovery, topology retrieval, and health monitoring. Authority diminishes at each hop.',
        ...result,
      });
    } catch (err) {
      log.error('Mesh demo failed:', err);
      res.status(500).json({ success: false, error: 'Mesh demo failed' });
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

  app.post('/api/capabilities/hardware/register', (req: Request, res: Response) => {
    try {
      const parsed = hardwareRegisterSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = hardwareBindingEngine.registerDevice(parsed.data.device_id, parsed.data.binding_type);
      res.json({ success: true, hardware_binding: result });
    } catch (err: any) {
      log.error('Hardware register failed:', err);
      res.status(500).json({ success: false, error: err.message || 'Registration failed' });
    }
  });

  app.post('/api/capabilities/hardware/challenge', (req: Request, res: Response) => {
    try {
      const parsed = hardwareChallengeSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const windowNs = parsed.data.window_ns ? BigInt(parsed.data.window_ns) : undefined;
      const result = hardwareBindingEngine.issueChallenge(parsed.data.device_id, windowNs);
      res.json({ success: true, challenge: result });
    } catch (err: any) {
      log.error('Hardware challenge failed:', err);
      res.status(400).json({ success: false, error: err.message || 'Challenge issuance failed' });
    }
  });

  app.post('/api/capabilities/hardware/verify', (req: Request, res: Response) => {
    try {
      const parsed = hardwareVerifySchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = hardwareBindingEngine.verifyChallenge(parsed.data);
      res.json({ success: true, verification: result });
    } catch (err: any) {
      log.error('Hardware verify failed:', err);
      res.status(500).json({ success: false, error: err.message || 'Verification failed' });
    }
  });

  app.post('/api/capabilities/hardware/issue', (req: Request, res: Response) => {
    try {
      const parsed = hardwareIssueSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const resources = parsed.data.capabilities.map(c => ({
        res: c.resource,
        constraints: c.constraints,
        ttlSeconds: c.ttl_seconds,
      }));

      const result = hardwareBindingEngine.issueHardwareBoundToken(
        parsed.data.subject,
        resources,
        parsed.data.device_id,
      );

      res.json({ success: true, hardware_bound_token: result });
    } catch (err: any) {
      log.error('Hardware issue failed:', err);
      res.status(400).json({ success: false, error: err.message || 'Hardware token issuance failed' });
    }
  });

  app.post('/api/capabilities/certificate/issue', (req: Request, res: Response) => {
    try {
      const parsed = certificateIssueSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = capabilityCertificateService.issueCapabilityCertificate(parsed.data.signed_token as any);
      res.json({ success: true, certificate: result });
    } catch (err: any) {
      log.error('Certificate issue failed:', err);
      res.status(500).json({ success: false, error: err.message || 'Certificate issuance failed' });
    }
  });

  app.post('/api/capabilities/certificate/verify', (req: Request, res: Response) => {
    try {
      const parsed = certificateVerifySchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = capabilityCertificateService.verifyCapabilityCertificate(parsed.data.certificate_id);
      res.json({ success: true, verification: result });
    } catch (err: any) {
      log.error('Certificate verify failed:', err);
      res.status(500).json({ success: false, error: err.message || 'Certificate verification failed' });
    }
  });

  app.post('/api/capabilities/certificate/evidence-chain', (req: Request, res: Response) => {
    try {
      const parsed = evidenceChainSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = capabilityCertificateService.createEvidenceChain(parsed.data.certificate_ids);
      res.json({ success: true, evidence_chain: result });
    } catch (err: any) {
      log.error('Evidence chain failed:', err);
      res.status(400).json({ success: false, error: err.message || 'Evidence chain creation failed' });
    }
  });

  app.get('/api/capabilities/certificate/stats', (_req: Request, res: Response) => {
    try {
      const stats = capabilityCertificateService.getStats();
      res.json({ success: true, certificate_stats: stats });
    } catch (err) {
      log.error('Certificate stats failed:', err);
      res.status(500).json({ success: false, error: 'Stats query failed' });
    }
  });

  app.post('/api/capabilities/mesh/register', (req: Request, res: Response) => {
    try {
      const parsed = meshRegisterSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = capabilityMeshService.registerService(
        parsed.data.service_id,
        parsed.data.service_name,
        parsed.data.capabilities,
        parsed.data.endpoint,
        parsed.data.metadata,
      );
      res.json({ success: true, service_node: result });
    } catch (err: any) {
      log.error('Mesh register failed:', err);
      res.status(500).json({ success: false, error: err.message || 'Service registration failed' });
    }
  });

  app.post('/api/capabilities/mesh/issue', (req: Request, res: Response) => {
    try {
      const parsed = meshIssueSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const resources = parsed.data.resources.map(r => ({
        res: r.resource,
        constraints: r.constraints,
        ttlSeconds: r.ttl_seconds,
      }));

      const result = capabilityMeshService.issueServiceCapability(
        parsed.data.from_service,
        parsed.data.to_service,
        resources,
      );
      res.json({ success: true, mesh_capability: result });
    } catch (err: any) {
      log.error('Mesh issue failed:', err);
      res.status(400).json({ success: false, error: err.message || 'Mesh capability issuance failed' });
    }
  });

  app.post('/api/capabilities/mesh/propagate', (req: Request, res: Response) => {
    try {
      const parsed = meshPropagateSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = capabilityMeshService.propagateCapability(
        parsed.data.mesh_token_id,
        parsed.data.next_service,
        parsed.data.attenuations,
      );
      res.json({ success: true, propagated_capability: result });
    } catch (err: any) {
      log.error('Mesh propagate failed:', err);
      res.status(400).json({ success: false, error: err.message || 'Capability propagation failed' });
    }
  });

  app.get('/api/capabilities/mesh/discover', (req: Request, res: Response) => {
    try {
      const pattern = (req.query.pattern as string) || '*';
      const result = capabilityMeshService.discoverServices(pattern);
      res.json({
        success: true,
        pattern,
        services_found: result.length,
        services: result,
      });
    } catch (err) {
      log.error('Mesh discover failed:', err);
      res.status(500).json({ success: false, error: 'Service discovery failed' });
    }
  });

  app.post('/api/capabilities/mesh/validate', (req: Request, res: Response) => {
    try {
      const parsed = meshValidateSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const result = capabilityMeshService.validateMeshCapability(
        parsed.data.mesh_token_id,
        parsed.data.requesting_service,
        parsed.data.target_service,
      );
      res.json({ success: true, validation: result });
    } catch (err: any) {
      log.error('Mesh validate failed:', err);
      res.status(500).json({ success: false, error: err.message || 'Mesh validation failed' });
    }
  });

  app.get('/api/capabilities/mesh/topology', (_req: Request, res: Response) => {
    try {
      const result = capabilityMeshService.getMeshTopology();
      res.json({ success: true, topology: result });
    } catch (err) {
      log.error('Mesh topology failed:', err);
      res.status(500).json({ success: false, error: 'Topology retrieval failed' });
    }
  });

  app.get('/api/capabilities/mesh/health', (_req: Request, res: Response) => {
    try {
      const result = capabilityMeshService.getMeshHealth();
      res.json({ success: true, health: result });
    } catch (err) {
      log.error('Mesh health failed:', err);
      res.status(500).json({ success: false, error: 'Health check failed' });
    }
  });

  app.get('/api/capabilities/status', (_req: Request, res: Response) => {
    const hptpNs = capabilityService.getHptpNow();
    const certStats = capabilityCertificateService.getStats();
    const meshHealth = capabilityMeshService.getMeshHealth();

    res.json({
      success: true,
      service: 'PlenumNET Capability Token Service',
      version: '4.0.0',
      phases: {
        phase_1: { status: 'complete', description: 'Typed constraint registry + capability token schema + audit events' },
        phase_2: { status: 'complete', description: 'HPTP-bound expiration — timing engine wired into validation path' },
        phase_3: { status: 'complete', description: 'HMAC-chained delegation — macaroon-style attenuation with TL-DSA roots' },
        phase_4: { status: 'complete', description: 'Hardware-bound capabilities + HPTP challenge-response + single-use chains — confinement solved' },
        phase_5: { status: 'complete', description: 'RFC 3161 capability certificates — court-admissible evidence chain with dual TL-DSA + RSA-4096 signing' },
        phase_6: { status: 'complete', description: 'Inter-service capability mesh — distributed capability propagation with per-hop attenuation' },
      },
      current_hptp_ns: hptpNs,
      signing_algorithm: 'TL-DSA',
      constraint_registry_version: '1.0',
      supported_constraints: [
        'recipient_domain', 'vault_id', 'template', 'max_uses',
        'ip_range', 'geo_country', 'document_id', 'project_id',
      ],
      certificate_stats: certStats,
      mesh_health: meshHealth,
      endpoints: {
        demo_expiration: 'GET /api/capabilities/demo/expiration',
        demo_delegation: 'GET /api/capabilities/demo/delegation',
        demo_confinement: 'GET /api/capabilities/demo/confinement',
        demo_certificates: 'GET /api/capabilities/demo/certificates',
        demo_mesh: 'GET /api/capabilities/demo/mesh',
        issue: 'POST /api/capabilities/issue',
        validate: 'POST /api/capabilities/validate',
        delegate: 'POST /api/capabilities/delegate',
        delegate_chain: 'POST /api/capabilities/delegate/chain',
        verify_chain: 'POST /api/capabilities/verify-chain',
        audit: 'GET /api/capabilities/audit',
        hardware_register: 'POST /api/capabilities/hardware/register',
        hardware_challenge: 'POST /api/capabilities/hardware/challenge',
        hardware_verify: 'POST /api/capabilities/hardware/verify',
        hardware_issue: 'POST /api/capabilities/hardware/issue',
        certificate_issue: 'POST /api/capabilities/certificate/issue',
        certificate_verify: 'POST /api/capabilities/certificate/verify',
        certificate_evidence_chain: 'POST /api/capabilities/certificate/evidence-chain',
        certificate_stats: 'GET /api/capabilities/certificate/stats',
        mesh_register: 'POST /api/capabilities/mesh/register',
        mesh_issue: 'POST /api/capabilities/mesh/issue',
        mesh_propagate: 'POST /api/capabilities/mesh/propagate',
        mesh_discover: 'GET /api/capabilities/mesh/discover',
        mesh_validate: 'POST /api/capabilities/mesh/validate',
        mesh_topology: 'GET /api/capabilities/mesh/topology',
        mesh_health: 'GET /api/capabilities/mesh/health',
        status: 'GET /api/capabilities/status',
      },
    });
  });
}
