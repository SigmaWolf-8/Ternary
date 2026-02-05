import Fastify from 'fastify';
import cors from '@fastify/cors';
import { createLogger, format, transports } from 'winston';
import { HPTPClient } from './hptp-client';
import { ClockDriver } from './clock-driver';
import { timingApiRouter } from './api/timing-api';

const logger = createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: format.combine(format.timestamp(), format.json()),
  defaultMeta: { service: 'femtosecond-service' },
  transports: [new transports.Console()]
});

async function main() {
  const fastify = Fastify({ logger: false });
  
  await fastify.register(cors, { origin: true });

  const clockDriver = new ClockDriver();
  const hptpClient = new HPTPClient(clockDriver);
  
  await hptpClient.initialize();

  fastify.decorate('clockDriver', clockDriver);
  fastify.decorate('hptpClient', hptpClient);

  await fastify.register(timingApiRouter, { prefix: '/api/timing/v1' });

  fastify.get('/health', async () => ({
    status: 'healthy',
    service: 'femtosecond-service',
    synchronized: hptpClient.isSynchronized(),
    timestamp: new Date().toISOString()
  }));

  fastify.get('/health/live', async () => ({
    status: 'alive',
    timestamp: new Date().toISOString()
  }));

  fastify.get('/health/ready', async () => ({
    status: hptpClient.isSynchronized() ? 'ready' : 'not_ready',
    synchronized: hptpClient.isSynchronized(),
    timestamp: new Date().toISOString()
  }));

  const port = parseInt(process.env.PORT || '3006', 10);
  await fastify.listen({ port, host: '0.0.0.0' });
  
  logger.info(`Femtosecond timing service started on port ${port}`, {
    synchronized: hptpClient.isSynchronized(),
    clockSource: clockDriver.getClockSource()
  });
}

main().catch(console.error);
