import Fastify from 'fastify';
import { createLogger, format, transports } from 'winston';
import { XRPLClient } from './xrpl-client';
import { PaymentService } from './payment-service';

const logger = createLogger({
  level: 'info',
  format: format.combine(format.timestamp(), format.json()),
  defaultMeta: { service: 'xrpl-service' },
  transports: [new transports.Console()]
});

async function main() {
  const fastify = Fastify({ logger: false });
  const xrplClient = new XRPLClient();
  const paymentService = new PaymentService(xrplClient);

  await xrplClient.connect();

  fastify.post('/internal/api/xrpl/v1/payments', async (request, reply) => {
    const { operationId, amount, currency, destination, memo } = request.body as any;
    
    try {
      const result = await paymentService.submitPayment({
        operationId,
        amount,
        currency,
        destination,
        memo
      });
      return reply.status(202).send(result);
    } catch (error) {
      logger.error('Payment submission failed', { error, operationId });
      return reply.status(500).send({ error: 'Payment submission failed' });
    }
  });

  fastify.get('/internal/api/xrpl/v1/payments/:txHash', async (request, reply) => {
    const { txHash } = request.params as any;
    const status = await paymentService.getPaymentStatus(txHash);
    return reply.status(200).send(status);
  });

  fastify.get('/health', async () => ({
    status: xrplClient.isConnected() ? 'healthy' : 'degraded',
    service: 'xrpl-service',
    connected: xrplClient.isConnected(),
    timestamp: new Date().toISOString()
  }));

  const port = parseInt(process.env.PORT || '3004', 10);
  await fastify.listen({ port, host: '0.0.0.0' });
  logger.info(`XRPL service started on port ${port}`);
}

main().catch(console.error);
