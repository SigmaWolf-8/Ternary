/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * SFK OPERATIONS API ROUTES
 * Location: server/routes/sfk-operations.ts
 *
 * 5 endpoints under /api/sfk/v1/*
 *
 *   POST   /api/sfk/v1/operations           — Submit new operation
 *   GET    /api/sfk/v1/operations/:id        — Get operation status
 *   GET    /api/sfk/v1/operations            — List operations
 *   DELETE /api/sfk/v1/operations/:id        — Cancel operation
 *   GET    /api/sfk/v1/stats                 — Service statistics
 */

import { Router, Request, Response } from 'express';
import { SFKOperationsService } from '../services/sfk-operations-service';
import type { SecurityMode } from '../salvi-core/unified-metadata-schema';
import type { OperationType, OperationStatus } from '../salvi-core/sfk-operations-api';

export function createSFKOperationsRoutes(service: SFKOperationsService): Router {
  const router = Router();

  router.post('/v1/operations', async (req: Request, res: Response) => {
    try {
      const body = req.body;

      if (!body.operation?.type || !body.operation?.parameters?.security_mode) {
        return res.status(400).json({
          error: 'Missing required fields',
          required: {
            'operation.type': "'TERNARY_BATCH_PROCESSING' | 'PHASE_ENCRYPTION' | 'TORSION_ROUTING' | 'WITNESS_SUBMISSION' | 'SETTLEMENT_EXECUTION'",
            'operation.parameters.security_mode': "'phi_plus' | 'phi' | 'one' | 'zero'",
          },
          optional: {
            'operation.parameters.batch_size': 'number (default: 1)',
            'operation.parameters.phase_offset': 'number (default: 4)',
            'operation.parameters.torsion_dimensions': '7 | 10 | 13 (default: 13)',
            'operation.trigger': '{ payment_id, gateway, settled_amount, settled_currency }',
            'metadata': '{ salvi_batch_ref, customer_id, correlation_id }',
          },
          example: {
            operation: {
              type: 'TERNARY_BATCH_PROCESSING',
              trigger: {
                payment_id: 'pay_test_001',
                gateway: 'internal',
                settled_amount: 0,
                settled_currency: 'USD',
              },
              parameters: {
                security_mode: 'phi',
                phase_offset: 4,
                torsion_dimensions: 13,
                batch_size: 1,
              },
              timing: {
                requested_start_window: new Date().toISOString(),
                max_duration_ns: 10_000_000_000,
                femtosecond_sync_required: false,
              },
            },
            metadata: {
              salvi_batch_ref: 'batch_test_001',
              customer_id: 'internal',
              correlation_id: 'corr_test_001',
            },
          },
        });
      }

      const validModes: SecurityMode[] = ['phi_plus', 'phi', 'one', 'zero'];
      if (!validModes.includes(body.operation.parameters.security_mode)) {
        return res.status(400).json({
          error: `Invalid security_mode: ${body.operation.parameters.security_mode}`,
          valid: validModes,
        });
      }

      const validTypes: OperationType[] = [
        'TERNARY_BATCH_PROCESSING', 'PHASE_ENCRYPTION',
        'TORSION_ROUTING', 'WITNESS_SUBMISSION', 'SETTLEMENT_EXECUTION',
      ];
      if (!validTypes.includes(body.operation.type)) {
        return res.status(400).json({
          error: `Invalid operation type: ${body.operation.type}`,
          valid: validTypes,
        });
      }

      const request = {
        operation: {
          id: '',
          type: body.operation.type as OperationType,
          trigger: {
            payment_id: body.operation.trigger?.payment_id || 'internal',
            gateway: body.operation.trigger?.gateway || 'internal',
            settled_amount: body.operation.trigger?.settled_amount ?? 0,
            settled_currency: body.operation.trigger?.settled_currency || 'USD',
          },
          parameters: {
            security_mode: body.operation.parameters.security_mode as SecurityMode,
            phase_offset: body.operation.parameters.phase_offset ?? 4,
            torsion_dimensions: body.operation.parameters.torsion_dimensions ?? 13,
            batch_size: body.operation.parameters.batch_size ?? 1,
          },
          timing: {
            requested_start_window: body.operation.timing?.requested_start_window || new Date().toISOString(),
            max_duration_ns: body.operation.timing?.max_duration_ns ?? 10_000_000_000,
            femtosecond_sync_required: body.operation.timing?.femtosecond_sync_required ?? false,
          },
        },
        metadata: {
          salvi_batch_ref: body.metadata?.salvi_batch_ref || `batch_${Date.now().toString(36)}`,
          customer_id: body.metadata?.customer_id || 'anonymous',
          correlation_id: body.metadata?.correlation_id || `corr_${Date.now().toString(36)}`,
        },
      };

      const result = await service.submitOperation(request);
      res.status(202).json(result);
    } catch (error) {
      const message = (error as Error).message;
      const status = message.includes('queue full') ? 429 : 500;
      res.status(status).json({ error: message });
    }
  });

  router.get('/v1/operations/:id', (req: Request, res: Response) => {
    const result = service.getOperationStatus(req.params.id);
    if (!result) {
      return res.status(404).json({
        error: 'Operation not found',
        operation_id: req.params.id,
        hint: 'Completed operations are evicted after 1 hour.',
      });
    }
    res.status(200).json(result);
  });

  router.get('/v1/operations', (req: Request, res: Response) => {
    const status = req.query.status as OperationStatus | undefined;
    const limit = req.query.limit ? parseInt(req.query.limit as string, 10) : undefined;
    const result = service.listOperations({ status, limit });
    res.status(200).json(result);
  });

  router.delete('/v1/operations/:id', (req: Request, res: Response) => {
    const cancelled = service.cancelOperation(req.params.id);
    if (!cancelled) {
      return res.status(404).json({
        error: 'Operation not found or already terminal',
        operation_id: req.params.id,
      });
    }
    res.status(200).json({
      cancelled: true,
      operation_id: req.params.id,
    });
  });

  router.get('/v1/stats', (_req: Request, res: Response) => {
    res.status(200).json(service.getStats());
  });

  return router;
}
