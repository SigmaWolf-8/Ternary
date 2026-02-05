import { FastifyPluginAsync, FastifyRequest, FastifyReply } from 'fastify';
import { OperationRequestSchema, OperationRequest } from '../../models/operation-request';
import { orchestrator } from '../../services/operation-orchestrator';

export const operationRouter: FastifyPluginAsync = async (fastify) => {
  fastify.post<{ Body: OperationRequest }>(
    '/',
    {
      schema: {
        description: 'Create a new SFK operation',
        tags: ['Operations'],
        body: {
          type: 'object',
          required: ['operationType'],
          properties: {
            operationType: { type: 'string', enum: ['payment_witness', 'data_attest', 'state_transition', 'consensus_round'] },
            securityMode: { type: 'string', enum: ['mode_zero', 'mode_one', 'phi_plus'], default: 'phi_plus' },
            priority: { type: 'string', enum: ['low', 'normal', 'high', 'critical'], default: 'normal' },
            paymentContext: {
              type: 'object',
              properties: {
                gateway: { type: 'string', enum: ['stripe', 'interac', 'crypto'] },
                paymentId: { type: 'string' },
                amount: { type: 'number' },
                currency: { type: 'string' }
              }
            },
            ternaryContext: {
              type: 'object',
              properties: {
                representation: { type: 'string', enum: ['A', 'B', 'C'] },
                dataHash: { type: 'string' },
                compressionEnabled: { type: 'boolean' }
              }
            },
            blockchainTargets: {
              type: 'object',
              properties: {
                witnessToHedera: { type: 'boolean' },
                settleOnXrpl: { type: 'boolean' },
                recordOnAlgorand: { type: 'boolean' }
              }
            },
            callbackUrl: { type: 'string', format: 'uri' },
            idempotencyKey: { type: 'string' },
            ttlSeconds: { type: 'integer', minimum: 1, maximum: 86400, default: 3600 }
          }
        },
        response: {
          202: {
            type: 'object',
            properties: {
              success: { type: 'boolean' },
              operationId: { type: 'string' },
              batchRef: { type: 'string' },
              kernelOpId: { type: 'string' },
              status: { type: 'string' },
              queuedAt: { type: 'string' },
              estimatedCompletionMs: { type: 'number' },
              trackingUrl: { type: 'string' }
            }
          }
        }
      }
    },
    async (request: FastifyRequest<{ Body: OperationRequest }>, reply: FastifyReply) => {
      try {
        const validatedRequest = OperationRequestSchema.parse(request.body);
        const result = await orchestrator.createOperation(validatedRequest);
        return reply.status(202).send(result);
      } catch (error) {
        if (error instanceof Error && error.name === 'ZodError') {
          return reply.status(400).send({
            success: false,
            error: {
              code: 'VALIDATION_ERROR',
              message: 'Invalid request body',
              details: error
            }
          });
        }
        throw error;
      }
    }
  );

  fastify.get(
    '/',
    {
      schema: {
        description: 'List SFK operations with pagination and filtering',
        tags: ['Operations'],
        querystring: {
          type: 'object',
          properties: {
            page: { type: 'integer', minimum: 1, default: 1 },
            limit: { type: 'integer', minimum: 1, maximum: 100, default: 20 },
            status: { type: 'string', enum: ['pending', 'queued', 'processing', 'witnessed', 'settled', 'failed', 'cancelled'] },
            operationType: { type: 'string', enum: ['payment_witness', 'data_attest', 'state_transition', 'consensus_round'] }
          }
        }
      }
    },
    async (request: FastifyRequest<{ Querystring: { page?: number; limit?: number; status?: string; operationType?: string } }>, reply: FastifyReply) => {
      const { page = 1, limit = 20, status, operationType } = request.query;
      
      const result = orchestrator.listOperations(page, limit, {
        status: status as any,
        operationType: operationType as any
      });

      return reply.status(200).send({
        operations: result.operations,
        pagination: {
          page,
          limit,
          total: result.total,
          totalPages: Math.ceil(result.total / limit)
        }
      });
    }
  );

  fastify.get<{ Params: { operationId: string } }>(
    '/:operationId',
    {
      schema: {
        description: 'Get a specific SFK operation by ID',
        tags: ['Operations'],
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
      
      if (!operation) {
        return reply.status(404).send({
          success: false,
          error: {
            code: 'OPERATION_NOT_FOUND',
            message: `Operation ${operationId} not found`
          }
        });
      }

      return reply.status(200).send(operation);
    }
  );

  fastify.get<{ Params: { operationId: string } }>(
    '/:operationId/metadata',
    {
      schema: {
        description: 'Get unified metadata for an operation',
        tags: ['Operations'],
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
      const metadata = orchestrator.getMetadata(operationId);
      
      if (!metadata) {
        return reply.status(404).send({
          success: false,
          error: {
            code: 'METADATA_NOT_FOUND',
            message: `Metadata for operation ${operationId} not found`
          }
        });
      }

      return reply.status(200).send(metadata);
    }
  );
};
