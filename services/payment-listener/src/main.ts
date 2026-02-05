import express, { Express, Request, Response, NextFunction } from 'express';
import helmet from 'helmet';
import cors from 'cors';
import compression from 'compression';
import rateLimit from 'express-rate-limit';
import { createLogger, format, transports } from 'winston';
import { stripeWebhookRouter } from './webhook/stripe-webhook';
import { interacWebhookRouter } from './webhook/interac-webhook';
import { cryptoWebhookRouter } from './webhook/crypto-webhook';
import { statusApiRouter } from './api/status-api';
import { healthApiRouter } from './api/health-api';
import { PaymentQueue } from './queue/payment-queue';
import { webhookConfig } from './config/webhook-config';

const logger = createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: format.combine(
    format.timestamp({ format: 'YYYY-MM-DD HH:mm:ss.SSS' }),
    format.errors({ stack: true }),
    format.json()
  ),
  defaultMeta: { service: 'payment-listener' },
  transports: [
    new transports.Console({
      format: format.combine(format.colorize(), format.simple())
    })
  ]
});

const app: Express = express();

app.use(helmet());
app.use(cors({
  origin: webhookConfig.allowedOrigins,
  methods: ['POST', 'GET'],
  allowedHeaders: ['Content-Type', 'Stripe-Signature', 'X-JPMC-Signature', 'X-CC-Webhook-Signature']
}));

const webhookLimiter = rateLimit({
  windowMs: webhookConfig.rateLimitWindowMs,
  max: webhookConfig.rateLimitMax,
  message: { error: 'Too many webhook requests', code: 'RATE_LIMIT_EXCEEDED' },
  standardHeaders: true,
  legacyHeaders: false
});

app.use('/webhook', webhookLimiter);

app.use('/webhook/stripe', express.raw({ type: 'application/json' }), stripeWebhookRouter);
app.use('/webhook/interac', express.raw({ type: 'application/json' }), interacWebhookRouter);
app.use('/webhook/crypto', express.raw({ type: 'application/json' }), cryptoWebhookRouter);

app.use(express.json({ limit: '1mb' }));
app.use(compression());

app.use('/api/payments', statusApiRouter);
app.use('/health', healthApiRouter);

app.use((err: Error, req: Request, res: Response, _next: NextFunction) => {
  logger.error('Unhandled error', { 
    error: err.message, 
    stack: err.stack,
    path: req.path,
    method: req.method
  });
  
  res.status(500).json({
    error: 'Internal server error',
    code: 'INTERNAL_ERROR',
    requestId: req.headers['x-request-id'] || 'unknown'
  });
});

async function startServer(): Promise<void> {
  try {
    await PaymentQueue.initialize();
    logger.info('Payment queue initialized');

    const port = parseInt(process.env.PORT || '3001', 10);
    
    app.listen(port, '0.0.0.0', () => {
      logger.info(`Payment Listener Service started`, {
        port,
        environment: process.env.NODE_ENV || 'development',
        version: '1.0.0'
      });
    });
  } catch (error) {
    logger.error('Failed to start server', { error });
    process.exit(1);
  }
}

process.on('SIGTERM', async () => {
  logger.info('SIGTERM received, shutting down gracefully');
  await PaymentQueue.shutdown();
  process.exit(0);
});

process.on('SIGINT', async () => {
  logger.info('SIGINT received, shutting down gracefully');
  await PaymentQueue.shutdown();
  process.exit(0);
});

startServer();

export { app, logger };
