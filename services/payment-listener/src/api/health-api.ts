import { Router, Request, Response } from 'express';
import { PaymentQueue } from '../queue/payment-queue';
import { getIdempotencyStats } from '../validation/idempotency-checker';

export const healthApiRouter = Router();

interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  version: string;
  uptime: number;
  checks: {
    queue: ComponentHealth;
    idempotency: ComponentHealth;
    memory: ComponentHealth;
  };
}

interface ComponentHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  details?: Record<string, unknown>;
}

const startTime = Date.now();

function getQueueHealth(): ComponentHealth {
  const stats = PaymentQueue.getStats();
  const isHealthy = PaymentQueue.isHealthy();
  
  return {
    status: isHealthy ? 'healthy' : 'degraded',
    details: {
      pending: stats.pending,
      processing: stats.processing,
      completed: stats.completed,
      failed: stats.failed
    }
  };
}

function getIdempotencyHealth(): ComponentHealth {
  const stats = getIdempotencyStats();
  
  const isHealthy = stats.totalEntries < 100000;
  
  return {
    status: isHealthy ? 'healthy' : 'degraded',
    details: {
      totalEntries: stats.totalEntries,
      oldestEntryAgeMs: stats.oldestEntryAge,
      newestEntryAgeMs: stats.newestEntryAge
    }
  };
}

function getMemoryHealth(): ComponentHealth {
  const used = process.memoryUsage();
  const heapUsedMB = Math.round(used.heapUsed / 1024 / 1024);
  const heapTotalMB = Math.round(used.heapTotal / 1024 / 1024);
  const heapUsagePercent = (used.heapUsed / used.heapTotal) * 100;
  
  let status: 'healthy' | 'degraded' | 'unhealthy' = 'healthy';
  if (heapUsagePercent > 90) {
    status = 'unhealthy';
  } else if (heapUsagePercent > 75) {
    status = 'degraded';
  }
  
  return {
    status,
    details: {
      heapUsedMB,
      heapTotalMB,
      heapUsagePercent: Math.round(heapUsagePercent),
      rssMB: Math.round(used.rss / 1024 / 1024)
    }
  };
}

healthApiRouter.get('/', (_req: Request, res: Response) => {
  const queueHealth = getQueueHealth();
  const idempotencyHealth = getIdempotencyHealth();
  const memoryHealth = getMemoryHealth();
  
  const allHealthy = [queueHealth, idempotencyHealth, memoryHealth]
    .every(c => c.status === 'healthy');
  const anyUnhealthy = [queueHealth, idempotencyHealth, memoryHealth]
    .some(c => c.status === 'unhealthy');
  
  let overallStatus: 'healthy' | 'degraded' | 'unhealthy' = 'healthy';
  if (anyUnhealthy) {
    overallStatus = 'unhealthy';
  } else if (!allHealthy) {
    overallStatus = 'degraded';
  }
  
  const health: HealthStatus = {
    status: overallStatus,
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    uptime: Date.now() - startTime,
    checks: {
      queue: queueHealth,
      idempotency: idempotencyHealth,
      memory: memoryHealth
    }
  };
  
  const statusCode = overallStatus === 'healthy' ? 200 : 
                     overallStatus === 'degraded' ? 200 : 503;
  
  res.status(statusCode).json(health);
});

healthApiRouter.get('/live', (_req: Request, res: Response) => {
  res.status(200).json({ status: 'alive', timestamp: new Date().toISOString() });
});

healthApiRouter.get('/ready', (_req: Request, res: Response) => {
  const queueHealth = getQueueHealth();
  
  if (queueHealth.status === 'unhealthy') {
    return res.status(503).json({ 
      status: 'not_ready', 
      reason: 'Queue unhealthy',
      timestamp: new Date().toISOString() 
    });
  }
  
  res.status(200).json({ status: 'ready', timestamp: new Date().toISOString() });
});

healthApiRouter.get('/metrics', (_req: Request, res: Response) => {
  const queueStats = PaymentQueue.getStats();
  const idempotencyStats = getIdempotencyStats();
  const memoryUsage = process.memoryUsage();
  
  res.status(200).json({
    timestamp: new Date().toISOString(),
    uptime: Date.now() - startTime,
    queue: queueStats,
    idempotency: idempotencyStats,
    memory: {
      heapUsedMB: Math.round(memoryUsage.heapUsed / 1024 / 1024),
      heapTotalMB: Math.round(memoryUsage.heapTotal / 1024 / 1024),
      rssMB: Math.round(memoryUsage.rss / 1024 / 1024),
      externalMB: Math.round(memoryUsage.external / 1024 / 1024)
    },
    process: {
      pid: process.pid,
      nodeVersion: process.version,
      platform: process.platform
    }
  });
});
