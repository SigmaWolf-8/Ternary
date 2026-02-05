import Fastify, { FastifyInstance } from 'fastify';
import cors from '@fastify/cors';
import helmet from '@fastify/helmet';
import rateLimit from '@fastify/rate-limit';
import swagger from '@fastify/swagger';
import swaggerUi from '@fastify/swagger-ui';
import { createLogger, format, transports } from 'winston';
import { operationRouter } from './routes/operations/operation-router';
import { statusRouter } from './routes/operations/status-router';
import { blockchainRouter } from './routes/internal/blockchain-router';

const logger = createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: format.combine(
    format.timestamp({ format: 'YYYY-MM-DD HH:mm:ss.SSS' }),
    format.errors({ stack: true }),
    format.json()
  ),
  defaultMeta: { service: 'sfk-core-api' },
  transports: [
    new transports.Console({
      format: format.combine(format.colorize(), format.simple())
    })
  ]
});

async function buildServer(): Promise<FastifyInstance> {
  const fastify = Fastify({
    logger: false,
    bodyLimit: 1048576,
    trustProxy: true
  });

  await fastify.register(cors, {
    origin: true,
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization', 'X-Request-ID', 'X-API-Key']
  });

  await fastify.register(helmet);

  await fastify.register(rateLimit, {
    max: 1000,
    timeWindow: '1 minute',
    errorResponseBuilder: () => ({
      error: 'Rate limit exceeded',
      code: 'RATE_LIMIT_EXCEEDED',
      retryAfter: 60
    })
  });

  await fastify.register(swagger, {
    openapi: {
      openapi: '3.0.3',
      info: {
        title: 'SFK Core API',
        description: 'Salvi Framework Kernel - Operation Management & Blockchain Integration',
        version: '1.0.0'
      },
      servers: [
        { url: 'http://localhost:3002', description: 'Development' },
        { url: 'https://api.salvi.network', description: 'Production' }
      ],
      tags: [
        { name: 'Operations', description: 'SFK Operation management endpoints' },
        { name: 'Status', description: 'Operation status and tracking endpoints' },
        { name: 'Blockchain', description: 'Internal blockchain integration endpoints' }
      ]
    }
  });

  await fastify.register(swaggerUi, {
    routePrefix: '/docs',
    uiConfig: {
      docExpansion: 'list',
      deepLinking: true
    }
  });

  await fastify.register(operationRouter, { prefix: '/api/sfk/v1/operations' });
  await fastify.register(statusRouter, { prefix: '/api/sfk/v1/status' });
  await fastify.register(blockchainRouter, { prefix: '/internal/api' });

  fastify.get('/health', async () => ({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    service: 'sfk-core-api'
  }));

  fastify.get('/health/live', async () => ({
    status: 'alive',
    timestamp: new Date().toISOString()
  }));

  fastify.get('/health/ready', async () => ({
    status: 'ready',
    timestamp: new Date().toISOString()
  }));

  fastify.setErrorHandler((error, _request, reply) => {
    logger.error('Unhandled error', {
      error: error.message,
      stack: error.stack,
      code: error.code
    });

    reply.status(error.statusCode || 500).send({
      error: error.message || 'Internal server error',
      code: error.code || 'INTERNAL_ERROR'
    });
  });

  return fastify;
}

async function startServer(): Promise<void> {
  try {
    const server = await buildServer();
    const port = parseInt(process.env.PORT || '3002', 10);

    await server.listen({ port, host: '0.0.0.0' });

    logger.info('SFK Core API started', {
      port,
      environment: process.env.NODE_ENV || 'development',
      version: '1.0.0',
      docsUrl: `http://localhost:${port}/docs`
    });
  } catch (error) {
    logger.error('Failed to start server', { error });
    process.exit(1);
  }
}

startServer();
