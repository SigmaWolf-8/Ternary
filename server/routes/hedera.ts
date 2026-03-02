/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * HEDERA HCS WITNESSING API ROUTES
 * Location: server/routes/hedera.ts
 *
 * 6 endpoints under /api/hedera/*
 *
 *   POST /api/hedera/v1/witness        — Submit witness hash to HCS
 *   GET  /api/hedera/v1/witness/:txId  — Get witness status by transaction ID
 *   POST /api/hedera/v1/verify         — Verify witness via mirror node
 *   GET  /api/hedera/v1/topic          — Get topic info from network
 *   GET  /api/hedera/v1/health         — Service health check
 *   GET  /api/hedera/v1/stats          — Audit statistics
 */

import { Router, Request, Response } from 'express';
import { HederaWitnessingService } from '../services/hedera-witnessing-service';

import type {
  HederaWitnessRequest,
} from '../salvi-core/blockchain-integrations';
import type { SecurityMode, TorsionDimensions } from '../salvi-core/unified-metadata-schema';

export function createHederaRoutes(service: HederaWitnessingService): Router {
  const router = Router();

  router.post('/v1/witness', async (req: Request, res: Response) => {
    try {
      const {
        operation_id,
        witness_type,
        payload,
        metadata,
        topic,
        submission,
      } = req.body;

      if (!operation_id || !witness_type || !payload?.hash) {
        return res.status(400).json({
          error: 'Missing required fields',
          required: {
            operation_id: 'string — unique operation identifier',
            witness_type: "'MERKLE_ROOT_BATCH' | 'SINGLE_HASH' | 'AGGREGATE_PROOF'",
            payload: '{ hash: string, hash_algorithm: string, encoding: "hex" | "base64" }',
          },
          optional: {
            metadata: '{ salvi_batch_ref, kernel_op_id, ternary_context, payment_context, timing }',
            topic: '{ id, memo } — overrides default topic',
            submission: '{ max_fee_hbar, submit_key, require_consensus }',
          },
        });
      }

      const validWitnessTypes = ['MERKLE_ROOT_BATCH', 'SINGLE_HASH', 'AGGREGATE_PROOF'];
      if (!validWitnessTypes.includes(witness_type)) {
        return res.status(400).json({
          error: `Invalid witness_type: ${witness_type}`,
          valid: validWitnessTypes,
        });
      }

      const hashAlg = payload.hash_algorithm || 'SHA256';
      const validAlgorithms = ['SHA256', 'SHA384', 'SHA512', 'KECCAK256'];
      if (!validAlgorithms.includes(hashAlg)) {
        return res.status(400).json({
          error: `Invalid hash_algorithm: ${hashAlg}`,
          valid: validAlgorithms,
        });
      }

      const validEncodings = ['hex', 'base64'];
      const encoding = payload.encoding || 'hex';
      if (!validEncodings.includes(encoding)) {
        return res.status(400).json({
          error: `Invalid encoding: ${encoding}`,
          valid: validEncodings,
        });
      }

      const request: HederaWitnessRequest = {
        operation_id,
        witness_type,
        payload: {
          hash: payload.hash,
          hash_algorithm: hashAlg,
          encoding,
        },
        metadata: {
          salvi_batch_ref: metadata?.salvi_batch_ref || `batch_${Date.now().toString(36)}`,
          kernel_op_id: metadata?.kernel_op_id || operation_id,
          ternary_context: metadata?.ternary_context || {
            security_mode: 'one' as SecurityMode,
            phase_offset: 4,
            torsion_dimensions: 13 as TorsionDimensions,
            batch_size: 1,
            operation_count: 1,
          },
          payment_context: metadata?.payment_context || {
            gateway: 'internal',
            payment_id: 'N/A',
            amount: 0,
            currency: 'HBAR',
          },
          timing: metadata?.timing || {
            batch_start_ts: new Date().toISOString(),
            batch_end_ts: new Date().toISOString(),
            duration_ns: 0,
            femtosecond_sync_accuracy: 0,
          },
        },
        topic: topic || {
          id: service.getTopicId() || '',
          memo: 'PlenumNET witness',
        },
        submission: submission || {
          max_fee_hbar: 2,
          submit_key: '',
          require_consensus: true,
        },
      };

      const result = await service.submitWitness(request);
      res.status(201).json(result);
    } catch (error) {
      res.status(500).json({
        success: false,
        error: (error as Error).message,
      });
    }
  });

  router.get('/v1/witness/:txId', async (req: Request, res: Response) => {
    try {
      const result = await service.getWitnessStatus(req.params.txId);
      if (!result) {
        return res.status(404).json({
          error: 'Witness not found',
          transaction_id: req.params.txId,
          hint: 'Transaction may have expired from local cache. Query the mirror node directly.',
        });
      }
      res.status(200).json(result);
    } catch (error) {
      res.status(500).json({ error: (error as Error).message });
    }
  });

  router.post('/v1/verify', async (req: Request, res: Response) => {
    try {
      const { topic_id, sequence_number } = req.body;
      if (!topic_id || sequence_number === undefined) {
        return res.status(400).json({
          error: 'Missing required fields',
          required: {
            topic_id: 'string — e.g., "0.0.12345"',
            sequence_number: 'number — message sequence in topic',
          },
        });
      }

      const verified = await service.verifyWitness(topic_id, sequence_number);
      res.status(200).json({
        verified,
        topic_id,
        sequence_number,
        verified_at: new Date().toISOString(),
      });
    } catch (error) {
      res.status(500).json({ error: (error as Error).message });
    }
  });

  router.get('/v1/topic', async (_req: Request, res: Response) => {
    try {
      const info = await service.getTopicInfo();
      if (!info) {
        return res.status(503).json({
          error: 'Topic not available',
          hint: 'Service may not be initialized or topic may not exist',
        });
      }
      res.status(200).json(info);
    } catch (error) {
      res.status(500).json({ error: (error as Error).message });
    }
  });

  router.get('/v1/health', async (_req: Request, res: Response) => {
    try {
      const health = await service.getHealth();
      const statusCode = health.status === 'healthy' ? 200 :
                         health.status === 'degraded' ? 200 : 503;
      res.status(statusCode).json(health);
    } catch (error) {
      res.status(503).json({
        status: 'offline',
        error: (error as Error).message,
      });
    }
  });

  router.get('/v1/stats', (_req: Request, res: Response) => {
    res.status(200).json(service.getStats());
  });

  return router;
}
