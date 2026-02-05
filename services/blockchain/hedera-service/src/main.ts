import Fastify from 'fastify';
import { createLogger, format, transports } from 'winston';
import { HCSClient } from './hcs-client';
import { WitnessService } from './witness-service';

const logger = createLogger({
  level: 'info',
  format: format.combine(format.timestamp(), format.json()),
  defaultMeta: { service: 'hedera-service' },
  transports: [new transports.Console()]
});

async function main() {
  const fastify = Fastify({ logger: false });
  const hcsClient = new HCSClient();
  const witnessService = new WitnessService(hcsClient);

  fastify.post('/internal/api/hedera/v1/witness', async (request, reply) => {
    const { operationId, batchRef, dataHash, timestamp, securityMode } = request.body as any;
    
    try {
      const result = await witnessService.submitWitness({
        operationId,
        batchRef,
        dataHash,
        timestamp,
        securityMode
      });
      return reply.status(202).send(result);
    } catch (error) {
      logger.error('Witness submission failed', { error, operationId });
      return reply.status(500).send({ error: 'Witness submission failed' });
    }
  });

  fastify.get('/internal/api/hedera/v1/witness/:transactionId', async (request, reply) => {
    const { transactionId } = request.params as any;
    const status = await witnessService.getWitnessStatus(transactionId);
    return reply.status(200).send(status);
  });

  fastify.get('/health', async () => ({
    status: 'healthy',
    service: 'hedera-service',
    timestamp: new Date().toISOString()
  }));

  const port = parseInt(process.env.PORT || '3003', 10);
  await fastify.listen({ port, host: '0.0.0.0' });
  logger.info(`Hedera service started on port ${port}`);
}

main().catch(console.error);
