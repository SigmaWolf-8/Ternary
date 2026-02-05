import { FastifyPluginAsync, FastifyRequest, FastifyReply } from 'fastify';
import { orchestrator } from '../../services/operation-orchestrator';
import { operationToStatusResponse, OperationTimelineResponse } from '../../models/operation-response';

export const statusRouter: FastifyPluginAsync = async (fastify) => {
  fastify.get<{ Params: { operationId: string } }>(
    '/:operationId',
    {
      schema: {
        description: 'Get detailed status of an operation',
        tags: ['Status'],
        params: {
          type: 'object',
          required: ['operationId'],
          properties: {
            operationId: { type: 'string', format: 'uuid' }
          }
        },
        response: {
          200: {
            type: 'object',
            properties: {
              operationId: { type: 'string' },
              batchRef: { type: 'string' },
              kernelOpId: { type: 'string' },
              status: { type: 'string' },
              operationType: { type: 'string' },
              securityMode: { type: 'string' },
              progress: { type: 'object' },
              timing: { type: 'object' },
              blockchainStatus: { type: 'object' }
            }
          }
        }
      }
    },
    async (request, reply) => {
      const { operationId } = request.params;
      const operation = orchestrator.getOperation(operationId);
      
      if (!operation) {
        return reply.status(404).send({
          success: false,
          error: {
            code: 'OPERATION_NOT_FOUND',
            message: `Operation ${operationId} not found`
          }
        });
      }

      const statusResponse = operationToStatusResponse(operation);
      return reply.status(200).send(statusResponse);
    }
  );

  fastify.get<{ Params: { operationId: string } }>(
    '/:operationId/timeline',
    {
      schema: {
        description: 'Get the event timeline for an operation',
        tags: ['Status'],
        params: {
          type: 'object',
          required: ['operationId'],
          properties: {
            operationId: { type: 'string', format: 'uuid' }
          }
        }
      }
    },
    async (request, reply) => {
      const { operationId } = request.params;
      const operation = orchestrator.getOperation(operationId);
      const metadata = orchestrator.getMetadata(operationId);
      
      if (!operation || !metadata) {
        return reply.status(404).send({
          success: false,
          error: {
            code: 'OPERATION_NOT_FOUND',
            message: `Operation ${operationId} not found`
          }
        });
      }

      const timeline = metadata.auditTrail.map((entry, index) => {
        const previousTimestamp = index > 0 
          ? new Date(metadata.auditTrail[index - 1].timestamp).getTime()
          : new Date(entry.timestamp).getTime();
        const currentTimestamp = new Date(entry.timestamp).getTime();
        
        return {
          event: entry.event,
          timestamp: entry.timestamp,
          details: entry.details,
          durationFromPreviousMs: index > 0 ? currentTimestamp - previousTimestamp : 0
        };
      });

      const firstTimestamp = timeline.length > 0 ? new Date(timeline[0].timestamp).getTime() : Date.now();
      const lastTimestamp = timeline.length > 0 ? new Date(timeline[timeline.length - 1].timestamp).getTime() : Date.now();

      const response: OperationTimelineResponse = {
        operationId,
        status: operation.status,
        timeline,
        totalDurationMs: lastTimestamp - firstTimestamp
      };

      return reply.status(200).send(response);
    }
  );

  fastify.get(
    '/summary',
    {
      schema: {
        description: 'Get a summary of all operation statuses',
        tags: ['Status']
      }
    },
    async (_request, reply) => {
      const result = orchestrator.listOperations(1, 1000);
      
      const summary = {
        total: result.total,
        byStatus: {
          pending: 0,
          queued: 0,
          processing: 0,
          witnessed: 0,
          settled: 0,
          failed: 0,
          cancelled: 0
        },
        byType: {
          payment_witness: 0,
          data_attest: 0,
          state_transition: 0,
          consensus_round: 0
        },
        averageCompletionTimeMs: 0,
        timestamp: new Date().toISOString()
      };

      let totalCompletionTime = 0;
      let completedCount = 0;

      for (const op of result.operations) {
        summary.byStatus[op.status]++;
        summary.byType[op.operationType]++;
        
        if (op.completedAt && op.createdAt) {
          const completionTime = new Date(op.completedAt).getTime() - new Date(op.createdAt).getTime();
          totalCompletionTime += completionTime;
          completedCount++;
        }
      }

      if (completedCount > 0) {
        summary.averageCompletionTimeMs = Math.round(totalCompletionTime / completedCount);
      }

      return reply.status(200).send(summary);
    }
  );

  fastify.get(
    '/health',
    {
      schema: {
        description: 'Get health status of the status service',
        tags: ['Status']
      }
    },
    async (_request, reply) => {
      return reply.status(200).send({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        service: 'status-router'
      });
    }
  );
};
