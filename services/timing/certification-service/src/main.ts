import Fastify from 'fastify';
import cors from '@fastify/cors';
import { createLogger, format, transports } from 'winston';
import { Certifier } from './certifier';
import { TimingVerifier } from './verifiers/timing-verifier';

const logger = createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: format.combine(format.timestamp(), format.json()),
  defaultMeta: { service: 'certification-service' },
  transports: [new transports.Console()]
});

async function main() {
  const fastify = Fastify({ logger: false });
  
  await fastify.register(cors, { origin: true });

  const timingVerifier = new TimingVerifier();
  const certifier = new Certifier(timingVerifier);

  fastify.post<{ Body: { operationId: string; timestamp: string; dataHash: string } }>(
    '/api/timing/v1/certify',
    async (request, reply) => {
      const { operationId, timestamp, dataHash } = request.body;
      
      try {
        const certificate = await certifier.certifyTimestamp({
          operationId,
          timestamp,
          dataHash
        });
        
        return reply.status(201).send({
          success: true,
          certificate
        });
      } catch (error) {
        logger.error('Certification failed', { error, operationId });
        return reply.status(400).send({
          success: false,
          error: error instanceof Error ? error.message : 'Certification failed'
        });
      }
    }
  );

  fastify.get<{ Params: { certificateId: string } }>(
    '/api/timing/v1/certificates/:certificateId',
    async (request, reply) => {
      const { certificateId } = request.params;
      const certificate = certifier.getCertificate(certificateId);
      
      if (!certificate) {
        return reply.status(404).send({
          success: false,
          error: 'Certificate not found'
        });
      }
      
      return reply.status(200).send({
        success: true,
        certificate
      });
    }
  );

  fastify.post<{ Body: { certificateId: string } }>(
    '/api/timing/v1/verify',
    async (request, reply) => {
      const { certificateId } = request.body;
      
      const verification = await certifier.verifyCertificate(certificateId);
      
      return reply.status(200).send({
        success: true,
        verification
      });
    }
  );

  fastify.get(
    '/api/timing/v1/compliance/finra-613',
    async (request, reply) => {
      const complianceStatus = certifier.getFINRA613ComplianceStatus();
      
      return reply.status(200).send({
        success: true,
        compliance: complianceStatus
      });
    }
  );

  fastify.get('/health', async () => ({
    status: 'healthy',
    service: 'certification-service',
    timestamp: new Date().toISOString()
  }));

  const port = parseInt(process.env.PORT || '3007', 10);
  await fastify.listen({ port, host: '0.0.0.0' });
  
  logger.info(`Certification service started on port ${port}`);
}

main().catch(console.error);
