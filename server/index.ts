/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import express, { type Request, Response, NextFunction } from "express";
import { registerRoutes } from "./routes";
import { serveStatic } from "./static";
import { createServer } from "http";
import { securityHeaders } from "./middleware/security-headers";
import { corsMiddleware } from "./middleware/cors-config";
import { globalLimiter } from "./middleware/rate-limiter";
import { spawn, type ChildProcess } from "child_process";
import { existsSync } from "fs";
import * as path from "path";
import { TsaService, type TsaConfig, TSA_POLICIES, type HptpClient, type TldsaClient } from "./services/tsa-service";
import { createTsaRoutes } from "./routes/tsa";
import { NotificationService, tsaMetricsRegistry, EFFECTIVE_PHASE } from "./services/notification-service";

const app = express();
const httpServer = createServer(app);

declare module "http" {
  interface IncomingMessage {
    rawBody: unknown;
  }
}

app.use(securityHeaders);
app.use(corsMiddleware);
app.use("/api/", globalLimiter);

// Raw body parsing for RFC 3161 binary requests (before JSON parser)
const TSA_MAX_BODY_BYTES = 65536;
app.use('/api/tsa/timestamp', (req, _res, next) => {
  if (req.headers['content-type'] === 'application/timestamp-query') {
    const chunks: Buffer[] = [];
    let totalBytes = 0;
    req.on('data', (chunk: Buffer) => {
      totalBytes += chunk.length;
      if (totalBytes > TSA_MAX_BODY_BYTES) {
        req.destroy(new Error(`Request body exceeds ${TSA_MAX_BODY_BYTES} bytes`));
        return;
      }
      chunks.push(chunk);
    });
    req.on('end', () => { req.body = Buffer.concat(chunks); next(); });
  } else { next(); }
});

app.use(
  express.json({
    limit: '50mb',
    verify: (req, _res, buf) => {
      req.rawBody = buf;
    },
  }),
);

app.use(express.urlencoded({ extended: false, limit: '50mb' }));

export function log(message: string, source = "express") {
  const formattedTime = new Date().toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
    hour12: true,
  });

  console.log(`${formattedTime} [${source}] ${message}`);
}

app.use((req, res, next) => {
  const start = Date.now();
  const path = req.path;
  let capturedJsonResponse: Record<string, any> | undefined = undefined;

  const originalResJson = res.json;
  res.json = function (bodyJson, ...args) {
    capturedJsonResponse = bodyJson;
    return originalResJson.apply(res, [bodyJson, ...args]);
  };

  res.on("finish", () => {
    const duration = Date.now() - start;
    if (path.startsWith("/api")) {
      let logLine = `${req.method} ${path} ${res.statusCode} in ${duration}ms`;
      if (capturedJsonResponse) {
        logLine += ` :: ${JSON.stringify(capturedJsonResponse)}`;
      }

      log(logLine);
    }
  });

  next();
});

function startPqtiService(): ChildProcess | null {
  const binaryPath = path.resolve("target/release/pqti-service");
  if (!existsSync(binaryPath)) {
    log("PQTI binary not found at target/release/pqti-service — skipping", "pqti");
    return null;
  }
  try {
    const child = spawn(binaryPath, [], {
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env },
    });
    child.on("error", (err) => {
      log(`PQTI spawn error (non-fatal): ${err.message}`, "pqti");
    });
    child.stdout?.on("data", (data: Buffer) => {
      const msg = data.toString().trim();
      if (msg) log(msg, "pqti");
    });
    child.stderr?.on("data", (data: Buffer) => {
      const msg = data.toString().trim();
      if (msg) log(`error: ${msg}`, "pqti");
    });
    child.on("exit", (code) => {
      log(`PQTI service exited with code ${code}`, "pqti");
    });
    log("PQTI service started on port 3001", "pqti");
    return child;
  } catch (err) {
    log(`PQTI failed to start (non-fatal): ${(err as Error).message}`, "pqti");
    return null;
  }
}

(async () => {
  const pqtiProcess = startPqtiService();

  process.on("SIGTERM", () => { pqtiProcess?.kill(); process.exit(0); });
  process.on("SIGINT", () => { pqtiProcess?.kill(); process.exit(0); });

  // === RFC 3161 TIME-STAMPING AUTHORITY (Kong service #21) ===
  const tsaKeysDir = path.join(process.cwd(), 'server/crypto/tsa-keys');
  const tsaConfig: TsaConfig = {
    privateKeyPath: path.join(tsaKeysDir, 'tsa-private.pem'),
    certificatePath: path.join(tsaKeysDir, 'tsa-cert.pem'),
    chainPath: path.join(tsaKeysDir, 'tsa-chain.pem'),
    keysDirectory: tsaKeysDir,
    defaultPolicy: TSA_POLICIES.DEFAULT,
    enableDualSign: true,
    maxRequestSize: 65536,
  };

  const hptpClientForTsa: HptpClient = {
    async getTimestamp() {
      try {
        const resp = await fetch('http://localhost:5000/api/salvi/timing/now');
        if (!resp.ok) throw new Error(`HPTP ${resp.status}`);
        const data = await resp.json() as any;
        return {
          timestamp: data.timestamp || data.hptp_timestamp || new Date().toISOString(),
          precision: data.precision || data.hptp_precision || 'femtosecond',
          source: data.source || 'hptp-engine',
        };
      } catch {
        return {
          timestamp: new Date().toISOString(),
          precision: 'millisecond-fallback',
          source: 'system-clock',
        };
      }
    },
  };

  const tldsaClientForTsa: TldsaClient = {
    async sign(hash: string) {
      const resp = await fetch('http://localhost:5000/api/pqti/tldsa/sign', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message: hash }),
      });
      if (!resp.ok) throw new Error(`TL-DSA ${resp.status}`);
      const data = await resp.json() as any;
      return {
        signature: data.signature || '',
        publicKeyId: data.public_key_id || data.publicKeyId || 'tldsa-primary',
        securityLevel: data.security_level || 'CNSA-2.0',
        algorithm: data.algorithm || 'TL-DSA-44',
      };
    },
  };

  const tsaService = new TsaService(tsaConfig, hptpClientForTsa, tldsaClientForTsa);
  try {
    const tsaInit = await tsaService.initialize();
    log(`TSA initialized — serial: ${tsaInit.serialRestored}, cert: ${tsaInit.certSubject}, expires: ${tsaInit.certExpiry}`, 'tsa');
  } catch (error) {
    log(`TSA initialization failed: ${(error as Error).message}`, 'tsa');
  }

  app.use('/api/tsa', createTsaRoutes(tsaService));
  log('TSA — 8 endpoints at /api/tsa/* (Kong service #21)', 'tsa');

  const notificationService = new NotificationService({
    hptpClient: hptpClientForTsa,
    tldsaClient: tldsaClientForTsa,
    tsaService: tsaService,
  });
  log(`Notification TSA integration active — Phase ${EFFECTIVE_PHASE}`, 'notify');
  if (EFFECTIVE_PHASE === 3) {
    console.warn(
      'Notification TSA integration running Phase 3 (TSA-only). ' +
      'Legacy headers disabled. Confirm all downstream consumers have migrated.',
    );
  }

  app.get('/metrics/notification-tsa', async (_req, res) => {
    res.set('Content-Type', tsaMetricsRegistry.contentType);
    res.end(await tsaMetricsRegistry.metrics());
  });

  app.post('/api/notifications/test', async (req, res) => {
    try {
      const { channel = 'email', to = 'test@example.com', subject = 'Test', body = 'Test notification', contentType } = req.body || {};
      let result;
      switch (channel) {
        case 'webhook':
          result = await notificationService.sendWebhook(to, { message: body }, { contentType });
          break;
        case 'sms':
          result = await notificationService.sendSms(to, body, { contentType });
          break;
        case 'event':
          result = await notificationService.emitEvent(subject, { message: body }, { contentType });
          break;
        case 'push':
          result = await notificationService.sendPush(to, subject, body, { contentType });
          break;
        default:
          result = await notificationService.sendEmail(to, subject, body, { contentType });
          break;
      }
      res.json(result);
    } catch (error) {
      res.status(500).json({ error: (error as Error).message });
    }
  });

  app.get('/api/notifications/status', (_req, res) => {
    res.json({
      phase: notificationService.getPhase(),
      tsaAvailable: notificationService.hasTsaService(),
      channels: ['email', 'webhook', 'sms', 'event', 'push'],
      proofModes: notificationService.getPhase() === 2
        ? ['tsa+legacy', 'legacy-fallback']
        : ['tsa-only', 'none'],
    });
  });

  await registerRoutes(httpServer, app);

  app.use((err: any, _req: Request, res: Response, next: NextFunction) => {
    const status = err.status || err.statusCode || 500;
    const message = err.message || "Internal Server Error";

    console.error("Internal Server Error:", err);

    if (res.headersSent) {
      return next(err);
    }

    return res.status(status).json({ message });
  });

  // importantly only setup vite in development and after
  // setting up all the other routes so the catch-all route
  // doesn't interfere with the other routes
  if (process.env.NODE_ENV === "production") {
    serveStatic(app);
  } else {
    const { setupVite } = await import("./vite");
    await setupVite(httpServer, app);
  }

  // ALWAYS serve the app on the port specified in the environment variable PORT
  // Other ports are firewalled. Default to 5000 if not specified.
  // this serves both the API and the client.
  // It is the only port that is not firewalled.
  const port = parseInt(process.env.PORT || "5000", 10);
  httpServer.listen(
    {
      port,
      host: "0.0.0.0",
      reusePort: true,
    },
    () => {
      log(`serving on port ${port}`);
    },
  );
})();
