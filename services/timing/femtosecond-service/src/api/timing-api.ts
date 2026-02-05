import { FastifyPluginAsync, FastifyRequest, FastifyReply } from 'fastify';

export const timingApiRouter: FastifyPluginAsync = async (fastify) => {
  fastify.get(
    '/current',
    {
      schema: {
        description: 'Get current femtosecond-precision timestamp',
        tags: ['Timing'],
        response: {
          200: {
            type: 'object',
            properties: {
              success: { type: 'boolean' },
              timestamp: { type: 'object' },
              syncStatus: { type: 'object' }
            }
          }
        }
      }
    },
    async (request: FastifyRequest, reply: FastifyReply) => {
      const hptpClient = (fastify as any).hptpClient;
      const femtosecondTs = hptpClient.getFemtosecondTimestamp();
      
      return reply.status(200).send({
        success: true,
        timestamp: {
          iso: femtosecondTs.isoTimestamp,
          femtoseconds: femtosecondTs.femtoseconds,
          components: femtosecondTs.components
        },
        syncStatus: femtosecondTs.syncStatus
      });
    }
  );

  fastify.get(
    '/sync-status',
    {
      schema: {
        description: 'Get HPTP synchronization status',
        tags: ['Timing']
      }
    },
    async (request: FastifyRequest, reply: FastifyReply) => {
      const hptpClient = (fastify as any).hptpClient;
      
      return reply.status(200).send({
        success: true,
        status: hptpClient.getSyncStatus()
      });
    }
  );

  fastify.get(
    '/peers',
    {
      schema: {
        description: 'Get HPTP peer status',
        tags: ['Timing']
      }
    },
    async (request: FastifyRequest, reply: FastifyReply) => {
      const hptpClient = (fastify as any).hptpClient;
      
      return reply.status(200).send({
        success: true,
        peers: hptpClient.getPeers()
      });
    }
  );

  fastify.get(
    '/calibration',
    {
      schema: {
        description: 'Get clock driver calibration status',
        tags: ['Timing']
      }
    },
    async (request: FastifyRequest, reply: FastifyReply) => {
      const clockDriver = (fastify as any).clockDriver;
      
      return reply.status(200).send({
        success: true,
        calibration: clockDriver.getCalibrationStatus()
      });
    }
  );

  fastify.post<{ Body: { count: number } }>(
    '/batch',
    {
      schema: {
        description: 'Get batch of femtosecond timestamps',
        tags: ['Timing'],
        body: {
          type: 'object',
          properties: {
            count: { type: 'integer', minimum: 1, maximum: 1000, default: 10 }
          }
        }
      }
    },
    async (request: FastifyRequest<{ Body: { count: number } }>, reply: FastifyReply) => {
      const hptpClient = (fastify as any).hptpClient;
      const count = request.body.count || 10;
      
      const timestamps = [];
      for (let i = 0; i < count; i++) {
        timestamps.push(hptpClient.getFemtosecondTimestamp());
      }
      
      return reply.status(200).send({
        success: true,
        count: timestamps.length,
        timestamps
      });
    }
  );

  fastify.get(
    '/metrics',
    {
      schema: {
        description: 'Get timing metrics for monitoring',
        tags: ['Timing']
      }
    },
    async (request: FastifyRequest, reply: FastifyReply) => {
      const hptpClient = (fastify as any).hptpClient;
      const clockDriver = (fastify as any).clockDriver;
      const syncStatus = hptpClient.getSyncStatus();
      const calibration = clockDriver.getCalibrationStatus();
      
      return reply.status(200).send({
        success: true,
        metrics: {
          synchronized: syncStatus.synchronized,
          stratum: syncStatus.stratum,
          offsetNs: syncStatus.offsetNs,
          jitterNs: syncStatus.jitterNs,
          driftPpb: syncStatus.driftPpb,
          peerCount: syncStatus.peerCount,
          clockSource: calibration.source,
          precision: calibration.precision,
          lastSync: syncStatus.lastSyncAt,
          lastCalibration: calibration.lastCalibration
        }
      });
    }
  );
};
