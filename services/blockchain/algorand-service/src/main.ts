import Fastify from 'fastify';
import { createLogger, format, transports } from 'winston';
import { AlgorandClient } from './algorand-client';
import { ContractService } from './contract-service';

const logger = createLogger({
  level: 'info',
  format: format.combine(format.timestamp(), format.json()),
  defaultMeta: { service: 'algorand-service' },
  transports: [new transports.Console()]
});

async function main() {
  const fastify = Fastify({ logger: false });
  const algorandClient = new AlgorandClient();
  const contractService = new ContractService(algorandClient);

  fastify.post('/internal/api/algorand/v1/application/call', async (request, reply) => {
    const { operationId, appId, method, args } = request.body as any;
    
    try {
      const result = await contractService.callApplication({
        operationId,
        appId,
        method,
        args: args || []
      });
      return reply.status(202).send(result);
    } catch (error) {
      logger.error('Application call failed', { error, operationId });
      return reply.status(500).send({ error: 'Application call failed' });
    }
  });

  fastify.get('/internal/api/algorand/v1/transactions/:txId', async (request, reply) => {
    const { txId } = request.params as any;
    const status = await contractService.getTransactionStatus(txId);
    return reply.status(200).send(status);
  });

  fastify.get('/internal/api/algorand/v1/applications/:appId', async (request, reply) => {
    const { appId } = request.params as any;
    const info = await algorandClient.getApplicationInfo(parseInt(appId, 10));
    return reply.status(200).send(info);
  });

  fastify.get('/health', async () => ({
    status: 'healthy',
    service: 'algorand-service',
    timestamp: new Date().toISOString()
  }));

  const port = parseInt(process.env.PORT || '3005', 10);
  await fastify.listen({ port, host: '0.0.0.0' });
  logger.info(`Algorand service started on port ${port}`);
}

main().catch(console.error);
