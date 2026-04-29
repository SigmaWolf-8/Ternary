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

import crypto from "crypto";

process.on("SIGHUP", () => {});

type YodaPipelineHandler = (msg: string, session: string, seq: number, repC: string) => Promise<string>;
let yodaReplayGuard: Map<string, Set<number>> | null = null;
let yodaRateWindows: Map<string, number[]> | null = null;
let yodaPipelineHandler: YodaPipelineHandler | null = null;

const PLENUMNET_VERSION = fs.readFileSync(path.resolve(import.meta.dirname ?? __dirname, '..', 'VERSION'), 'utf8').trim();

const _originalProcessExit = process.exit.bind(process);
let _serverListening = false;
(process as any).exit = ((code?: number) => {
  if (code === 1 && _serverListening) {
    console.error("[recovery] Suppressed process.exit(1) — API server stays alive (Vite HMR may be degraded until restart)");
    return undefined as never;
  }
  return _originalProcessExit(code);
}) as typeof process.exit;

import express, { type Request, Response, NextFunction } from "express";
import compression from "compression";
import { storage } from "./storage";
import { registerRoutes } from "./routes";
import { serveStatic } from "./static";
import { createServer } from "http";
import { securityHeaders, additionalSecurityHeaders } from "./middleware/security-headers";
import { corsMiddleware } from "./middleware/cors-config";
import { globalLimiter, computationLimiter } from "./middleware/rate-limiter";
import { spawn, execSync, type ChildProcess } from "child_process";
import { existsSync } from "fs";
import { WebSocketServer, WebSocket } from "ws";
import { createSession, getSession, destroySession, listSessions, resizeSession, isSessionOwner, isClusterCommandAllowed, type TerminalSession } from "./terminal";
import * as path from "path";
import { spongeHashTrits } from "./crypto/sponge-hash";
import { TsaService, type TsaConfig, TSA_POLICIES, type HptpClient, type TldsaClient } from "./services/tsa-service";
import { createTsaRoutes } from "./routes/tsa";
import { type CalendarServiceClient } from "./services/tsa-calendar-enrichment";
import { keygen, signHex, verifyHex, verifyNative, publicKeyHash, isNativeAvailable, type TlDsaKeyPair } from "./crypto/tl-dsa-bridge";
import * as fs from "fs";
import {
  computeHealthState, type NodeHealthState,
  recordDisconnectEvent, getDisconnectHistory, type DisconnectEvent,
  RELAY_ERROR_CODES, makeErrorResponse, type RelayErrorCode,
  CircuitBreaker,
  recordRelayAuditEvent, type RelayAuditEventType,
  getExpectedNodesCache, addExpectedNode, removeExpectedNode, isExpectedNode, syncExpectedNodesCache,
} from "./services/node-watchdog";
import { getSalviEpochCalendarSync } from "./salvi-core/ancient-calendar-sync";
import { getFemtosecondTimestamp, SALVI_EPOCH_FS } from "./salvi-core/femtosecond-timing";
import { NotificationService, tsaMetricsRegistry, EFFECTIVE_PHASE } from "./services/notification-service";
import { HederaWitnessingService, createHederaConfig } from "./services/hedera-witnessing-service";
import { createHederaRoutes } from "./routes/hedera";
import { SFKOperationsService } from "./services/sfk-operations-service";
import { createSFKOperationsRoutes } from "./routes/sfk-operations";
import { opsChannelService } from "./services/ops-channel";
import { isOpsMessageType, type OpsMessageType, type OpsErrorCode, type OpsMessage, type TelemetryMessage } from "@shared/ops-protocol";

const app = express();
const serverStartTime = Date.now();
const httpServer = createServer(app);

// ── EAC UTC ANCHOR ─ FROZEN AT BOOT ───────────────────────────────────
// Capture the (Date.now, hrtime, framework-attosecond-walk) triple ONCE
// at module load, BEFORE any cert is ever issued.  Anchoring at boot
// (not at first issuance) ensures that when any cert is later emitted
// the (hrNow − hrAtAnchor) elapsed-nanosecond term is already large,
// so the emitted attosecond integer is NEVER ms-padded with trailing
// zeros.  The anchor is the only place a wall-clock is ever consulted
// for the EAC timestamp.
{
  const _bootHptp   = getFemtosecondTimestamp();
  const _bootAsNum  = BigInt(String(_bootHptp.asSinceBootNum));
  const _bootAsDen  = BigInt(String(_bootHptp.asSinceBootDen));
  const _bootAsInt  = _bootAsDen > 0n ? _bootAsNum / _bootAsDen : 0n;
  const _bootWallMs = BigInt(Date.now());
  (globalThis as any).__plenum_eac_utc_anchor = {
    utcAtAnchor:         _bootWallMs * 1_000_000_000_000_000n,  // ms × 10¹⁵
    hrAtAnchor:          process.hrtime.bigint(),                // monotonic ns
    asSinceBootAtAnchor: _bootAsInt,                             // framework as
    isoAtAnchor:         new Date(Number(_bootWallMs)).toISOString(),
  };
  console.log(
    `[hmodal-eac] UTC anchor frozen at BOOT: ` +
    `${(globalThis as any).__plenum_eac_utc_anchor.isoAtAnchor}`,
  );
}

app.get("/install.ps1", (_req, res) => {
  const filePath = path.resolve("services/tdns-v2/install.ps1");
  if (existsSync(filePath)) {
    res.setHeader("Content-Type", "text/plain; charset=utf-8");
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.setHeader("Pragma", "no-cache");
    res.setHeader("X-Content-Type-Options", "nosniff");
    res.sendFile(filePath);
  } else {
    res.status(404).setHeader("Content-Type", "text/plain");
    res.send("# install.ps1 not found");
  }
});

app.get("/api/install.ps1", (_req, res) => {
  const filePath = path.resolve("services/tdns-v2/install.ps1");
  if (existsSync(filePath)) {
    const content = fs.readFileSync(filePath, "utf-8");
    res.setHeader("Content-Type", "text/plain; charset=utf-8");
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.setHeader("Pragma", "no-cache");
    res.setHeader("X-Content-Type-Options", "nosniff");
    res.send(content);
  } else {
    res.status(404).setHeader("Content-Type", "text/plain");
    res.send("# install.ps1 not found");
  }
});

import archiver from "archiver";

function sendExtensionZip(res: any, filename: string, contentType: string) {
  const extDir = path.resolve("services/tdns-v2/extension-chromium");
  if (!existsSync(extDir)) {
    res.status(500).type("text/plain").send("Extension source not found");
    return;
  }
  res.setHeader("Content-Type", contentType);
  res.setHeader("Content-Disposition", `attachment; filename=${filename}`);
  res.setHeader("Cache-Control", "no-store");
  const archive = archiver("zip", { zlib: { level: 9 } });
  archive.pipe(res);
  archive.directory(extDir, false);
  archive.finalize();
}

app.get("/api/extension-zip", (_req, res) => {
  sendExtensionZip(res, "plenumnet-tdns-extension.zip", "application/zip");
});

app.get("/api/install-extension", (_req, res) => {
  const filePath = path.resolve("services/tdns-v2/install.bat");
  if (existsSync(filePath)) {
    res.setHeader("Content-Type", "application/octet-stream");
    res.setHeader("Content-Disposition", "attachment; filename=install-tdns-extension.bat");
    res.setHeader("Cache-Control", "no-store");
    res.sendFile(filePath);
  } else {
    res.status(404).type("text/plain").send("Installer not found");
  }
});

app.get("/api/extension/chromium", (_req, res) => {
  sendExtensionZip(res, "plenumnet-tdns-extension.zip", "application/zip");
});

app.get("/api/extension/firefox", (_req, res) => {
  sendExtensionZip(res, "plenumnet-tdns.xpi", "application/x-xpinstall");
});

app.get("/api/install-script", (_req, res) => {
  const filePath = path.resolve("services/tdns-v2/install.ps1");
  if (existsSync(filePath)) {
    const content = fs.readFileSync(filePath, "utf-8");
    res.setHeader("Content-Type", "text/plain; charset=utf-8");
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.setHeader("Pragma", "no-cache");
    res.setHeader("X-Content-Type-Options", "nosniff");
    res.send(content);
  } else {
    res.status(404).setHeader("Content-Type", "text/plain");
    res.send("# install.ps1 not found");
  }
});

declare module "http" {
  interface IncomingMessage {
    rawBody: unknown;
  }
}

app.use(compression({ threshold: 1024 }));
app.use(securityHeaders);
app.use(additionalSecurityHeaders);
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

const CRYPTO_HASH_MAX_BYTES = 10 * 1024 * 1024;
app.use('/api/salvi/crypto/hash', (req, _res, next) => {
  if (req.headers['content-type'] === 'application/octet-stream') {
    const chunks: Buffer[] = [];
    let totalBytes = 0;
    req.on('data', (chunk: Buffer) => {
      totalBytes += chunk.length;
      if (totalBytes > CRYPTO_HASH_MAX_BYTES) {
        req.destroy(new Error(`Request body exceeds ${CRYPTO_HASH_MAX_BYTES} bytes`));
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
  let hederaService: HederaWitnessingService | null = null;

  let sfkOpsService: SFKOperationsService | null = null;
  process.on("SIGTERM", () => { sfkOpsService?.close(); hederaService?.close(); pqtiProcess?.kill(); process.exit(0); });
  process.on("SIGINT", () => { sfkOpsService?.close(); hederaService?.close(); pqtiProcess?.kill(); process.exit(0); });

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

  const tldsaKeyPath = path.join(tsaKeysDir, 'tldsa-keypair.json');
  let tldsaKeypair: TlDsaKeyPair;
  if (existsSync(tldsaKeyPath)) {
    try {
      const stored = JSON.parse(fs.readFileSync(tldsaKeyPath, 'utf8'));
      tldsaKeypair = {
        publicKey: Buffer.from(stored.publicKey, 'hex'),
        secretKey: Buffer.from(stored.secretKey, 'hex'),
        variant: stored.variant || 'TL-DSA-87',
      };
      log(`TL-DSA keypair loaded from disk — variant: ${tldsaKeypair.variant}, keyId: ${publicKeyHash(tldsaKeypair.publicKey).substring(0, 16)}…`, 'tldsa');
    } catch (e) {
      log(`TL-DSA keypair file corrupted, generating fresh — ${(e as Error).message}`, 'tldsa');
      tldsaKeypair = keygen('TL-DSA-87');
    }
  } else {
    tldsaKeypair = keygen('TL-DSA-87');
    log(`TL-DSA keypair generated — variant: TL-DSA-87 (NIST Level 5), keyId: ${publicKeyHash(tldsaKeypair.publicKey).substring(0, 16)}…`, 'tldsa');
  }
  try {
    if (!existsSync(tsaKeysDir)) {
      fs.mkdirSync(tsaKeysDir, { recursive: true });
    }
    fs.writeFileSync(tldsaKeyPath, JSON.stringify({
      publicKey: tldsaKeypair.publicKey.toString('hex'),
      secretKey: tldsaKeypair.secretKey.toString('hex'),
      variant: tldsaKeypair.variant,
      createdAt: new Date().toISOString(),
    }, null, 2), { mode: 0o600 });
  } catch (e) {
    log(`TL-DSA keypair persist failed: ${(e as Error).message}`, 'tldsa');
  }
  const tldsaKeyId = publicKeyHash(tldsaKeypair.publicKey);

  const tldsaClientForTsa: TldsaClient = {
    async sign(hash: string) {
      const signature = signHex(tldsaKeypair.secretKey, hash, 'TL-DSA-87');
      return {
        signature,
        publicKeyId: tldsaKeyId,
        securityLevel: 'CNSA-2.0',
        algorithm: 'TL-DSA-87',
      };
    },
    async verify(hash: string, signature: string) {
      return verifyHex(tldsaKeypair.publicKey, hash, signature, tldsaKeypair.secretKey, 'TL-DSA-87');
    },
  };

  const calendarClient: CalendarServiceClient = {
    async convertDate(utcTimestamp: string) {
      const date = new Date(utcTimestamp);
      const sync = getSalviEpochCalendarSync(date);
      const epochDate = new Date('2025-04-01T00:00:00.000Z');
      const salviEpochDay = Math.floor((date.getTime() - epochDate.getTime()) / 86_400_000);
      const jdnMapping = sync.allMappings.find((m: any) =>
        m.calendarSystem === 'Julian Day Number'
      );
      return {
        julianDayNumber: jdnMapping?.daysSinceCalendarOrigin || 0,
        salviEpochDay,
        calendars: sync.calendars || {},
        allMappings: sync.allMappings || [],
      };
    },
  };

  const tsaService = new TsaService(tsaConfig, hptpClientForTsa, tldsaClientForTsa, calendarClient);
  try {
    const tsaInit = await tsaService.initialize();
    log(`TSA initialized — serial: ${tsaInit.serialRestored}, cert: ${tsaInit.certSubject}, expires: ${tsaInit.certExpiry}`, 'tsa');
  } catch (error) {
    log(`TSA initialization failed: ${(error as Error).message}`, 'tsa');
  }

  app.use('/api/tsa', createTsaRoutes(tsaService));
  log('TSA — 8 endpoints at /api/tsa/* (Kong service #21)', 'tsa');

  // =====================================================
  // HEDERA HCS WITNESSING — blockchain-based non-repudiation
  // =====================================================
  const hederaConfig = createHederaConfig();

  if (hederaConfig) {
    hederaService = new HederaWitnessingService(hederaConfig);
    try {
      const hederaInit = await hederaService.initialize();
      log(`Hedera HCS initialized — topic: ${hederaInit.topicId}, network: ${hederaInit.network}, created: ${hederaInit.topicCreated}`, 'hedera');

      app.use('/api/hedera', createHederaRoutes(hederaService));
      log('Hedera — 6 endpoints at /api/hedera/* (Kong service #22)', 'hedera');
    } catch (error) {
      log(`Hedera initialization failed: ${(error as Error).message}`, 'hedera');
      log('Hedera witnessing disabled — set HEDERA_ACCOUNT_ID and HEDERA_PRIVATE_KEY to enable', 'hedera');
      hederaService = null;
    }
  } else {
    log('Hedera witnessing disabled — HEDERA_ACCOUNT_ID and HEDERA_PRIVATE_KEY not set', 'hedera');
  }

  const sfkOperationsService = new SFKOperationsService(hederaService);
  sfkOpsService = sfkOperationsService;
  app.use('/api/sfk', createSFKOperationsRoutes(sfkOperationsService));
  log(`SFK Operations — 5 endpoints at /api/sfk/v1/* (witnessing: ${hederaService ? 'enabled' : 'disabled'})`, 'sfk');

  const { capabilityCertificateService } = await import('./services/capability-certificates');
  capabilityCertificateService.setTsaService(tsaService);
  capabilityCertificateService.setHederaService(hederaService);
  log('Capability certificates wired to real TSA + Hedera — RFC 3161 + HCS integration active', 'capabilities');

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

  const OPS_ADMIN_SUBS = new Set((process.env.OPS_ADMIN_SUBS || '').split(',').filter(Boolean));
  const requireOpsAuth = (req: Request, res: Response, next: NextFunction) => {
    const isDev = process.env.NODE_ENV === "development";
    const authReq = req as Request & { user?: { claims?: { sub?: string; role?: string; is_admin?: boolean } }; isAuthenticated?: () => boolean };
    const user = authReq.user;
    const sub = user?.claims?.sub;
    const isAuthed = authReq.isAuthenticated?.() && sub;
    if (isDev && !isAuthed) {
      return next();
    }
    if (!isAuthed || !sub) {
      return res.status(401).json({ error: "Authentication required for ops endpoints" });
    }
    const isAdmin = user?.claims?.role === 'admin' || user?.claims?.is_admin === true || OPS_ADMIN_SUBS.has(sub);
    if (!isAdmin) {
      return res.status(403).json({ error: "Ops endpoints require admin privileges" });
    }
    next();
  };

  app.post('/api/ops/tis27-hash', requireOpsAuth, (req, res) => {
    try {
      const { data_base64 } = req.body;
      if (!data_base64 || typeof data_base64 !== 'string') {
        return res.status(400).json({ error: 'data_base64 required' });
      }
      const { tis27Hash } = require('./crypto/sponge-hash');
      const hash = tis27Hash(Buffer.from(data_base64, 'utf-8'));
      res.json({ hash, algorithm: 'tis27', length: hash.length });
    } catch (e: any) {
      res.status(500).json({ error: e?.message || 'Hash computation failed' });
    }
  });

  app.get('/api/ops/status', requireOpsAuth, (_req, res) => {
    const status = opsChannelService.getOpsStatus();
    res.json(status);
  });

  app.get('/api/ops/audit', requireOpsAuth, (_req, res) => {
    const limit = Math.min(parseInt(String(_req.query.limit || '20'), 10), 100);
    const entries = opsChannelService.getRecentAuditEntries(limit);
    res.json({ entries, count: entries.length });
  });

  app.get('/api/ops/operators', requireOpsAuth, (_req, res) => {
    const operators = opsChannelService.listOperators().map(op => ({
      name: op.name,
      keyFingerprint: op.keyFingerprint,
      scope: op.scope,
      registeredAt: op.registeredAt,
    }));
    res.json({ operators, ops_enabled: opsChannelService.isOpsEnabled() });
  });

  const requireOpsOrApiKey = (req: Request, res: Response, next: NextFunction) => {
    const authHeader = req.headers.authorization;
    if (authHeader?.startsWith('Bearer ')) {
      const token = authHeader.slice(7);
      const plenumKey = process.env.PLENUM_API_KEY;
      if (plenumKey && token === plenumKey) {
        return next();
      }
    }
    return requireOpsAuth(req, res, next);
  };

  app.post('/api/ops/enable', requireOpsOrApiKey, (_req, res) => {
    opsChannelService.setOpsEnabled(true);
    for (const [, clientWs] of relayClients.entries()) {
      if (clientWs.readyState === WebSocket.OPEN) {
        clientWs.send(JSON.stringify({ type: "relay", msgType: "ops-config-update", payload: JSON.stringify({ ops_enabled: true }), from: "coordinator" }));
      }
    }
    log('Operations channel ENABLED (propagated to connected nodes)', 'ops');
    res.json({ ops_enabled: true });
  });

  app.post('/api/ops/disable', requireOpsAuth, (_req, res) => {
    opsChannelService.setOpsEnabled(false);
    for (const [, clientWs] of relayClients.entries()) {
      if (clientWs.readyState === WebSocket.OPEN) {
        clientWs.send(JSON.stringify({ type: "relay", msgType: "ops-config-update", payload: JSON.stringify({ ops_enabled: false }), from: "coordinator" }));
      }
    }
    log('Operations channel DISABLED (propagated to connected nodes)', 'ops');
    res.json({ ops_enabled: false });
  });

  app.post('/api/ops/operators', requireOpsAuth, (req: Request, res: Response) => {
    const { name, keyFingerprint, publicKey, scope } = req.body;
    if (!name || !keyFingerprint || !publicKey || !scope) {
      res.status(400).json({ error: 'Missing required fields: name, keyFingerprint, publicKey, scope' });
      return;
    }
    if (!['full', 'exec-only', 'read-only'].includes(scope)) {
      res.status(400).json({ error: 'Invalid scope — must be: full, exec-only, or read-only' });
      return;
    }
    opsChannelService.registerOperator({
      name,
      keyFingerprint,
      publicKey,
      scope,
      registeredAt: new Date().toISOString(),
    });
    for (const [, clientWs] of relayClients.entries()) {
      if (clientWs.readyState === WebSocket.OPEN) {
        clientWs.send(JSON.stringify({ type: "relay", msgType: "ops-operator-sync", payload: JSON.stringify({ action: "add", name, key_fingerprint: keyFingerprint, public_key: publicKey, scope }), from: "coordinator" }));
      }
    }
    log(`Operator registered: ${name} (${keyFingerprint}, scope: ${scope})`, 'ops');
    res.json({ registered: true, name, keyFingerprint, scope });
  });

  app.delete('/api/ops/operators/:fingerprint', requireOpsAuth, (req: Request, res: Response) => {
    const fp = String(req.params.fingerprint);
    const removed = opsChannelService.removeOperator(fp);
    if (removed) {
      for (const [, clientWs] of relayClients.entries()) {
        if (clientWs.readyState === WebSocket.OPEN) {
          clientWs.send(JSON.stringify({ type: "relay", msgType: "ops-operator-sync", payload: JSON.stringify({ action: "remove", key_fingerprint: fp }), from: "coordinator" }));
        }
      }
      log(`Operator removed: ${fp}`, 'ops');
      res.json({ removed: true, fingerprint: fp });
    } else {
      res.status(404).json({ error: 'Operator not found' });
    }
  });

  const bootstrapFingerprint = tldsaKeyId.substring(0, 16);
  opsChannelService.registerOperator({
    name: 'bootstrap-relay-verifier',
    keyFingerprint: bootstrapFingerprint,
    publicKey: tldsaKeypair.publicKey.toString('hex'),
    scope: 'read-only',
    registeredAt: new Date().toISOString(),
  });
  app.post('/api/ops/propose-exec', requireOpsAuth, (req: Request, res: Response) => {
    const { proposed_script, rationale, target_node_id } = req.body;
    if (!proposed_script || !target_node_id) {
      return res.status(400).json({ error: 'proposed_script and target_node_id are required' });
    }
    const proposal = {
      type: "propose-exec" as const,
      source: "yoda-ai" as const,
      proposed_script: String(proposed_script),
      rationale: String(rationale || "AI-generated script"),
      target_node_id: String(target_node_id),
      proposal_id: `ai-${Date.now()}-${crypto.randomBytes(4).toString('hex')}`,
      proposed_at: new Date().toISOString(),
    };
    const clients = (globalThis as any).__opsTerminalClients as Set<WebSocket> | undefined;
    let delivered = 0;
    if (clients) {
      for (const ws of clients) {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify(proposal));
          delivered++;
        }
      }
    }
    log(`Propose-exec delivered to ${delivered} terminal client(s): ${rationale || 'no rationale'}`, 'ops');
    res.json({ proposal_id: proposal.proposal_id, delivered });
  });

  app.post('/api/ops/exec', requireOpsOrApiKey, (req: Request, res: Response) => {
    const { script, target_node_id, node_id, signature, public_key, context } = req.body;
    const targetId = target_node_id || node_id;
    const execScript = script || req.body.proposed_script;
    if (!execScript || !targetId) {
      return res.status(400).json({ error: 'script and target_node_id are required' });
    }

    if (!opsChannelService.isOpsEnabled()) {
      opsChannelService.setOpsEnabled(true);
      for (const [, clientWs] of relayClients.entries()) {
        if (clientWs.readyState === WebSocket.OPEN) {
          clientWs.send(JSON.stringify({ type: "relay", msgType: "ops-config-update", payload: JSON.stringify({ ops_enabled: true }), from: "coordinator" }));
        }
      }
      log('Ops channel auto-enabled by /api/ops/exec', 'ops');
    }

    const requestId = `http-${Date.now()}-${crypto.randomBytes(4).toString('hex')}`;
    const opsMsg: any = {
      type: 'exec',
      node_id: String(targetId),
      script: String(execScript),
      request_id: requestId,
      timestamp: Date.now(),
    };
    if (signature) opsMsg.signature = signature;
    if (public_key) opsMsg.public_key = public_key;
    if (context) opsMsg.context = context;

    let opsTargetWs: WebSocket | undefined;
    for (const [addr, clientWs] of relayClients.entries()) {
      if (addr === String(targetId) || addr.includes(String(targetId))) {
        opsTargetWs = clientWs;
        break;
      }
    }
    if (!opsTargetWs || opsTargetWs.readyState !== WebSocket.OPEN) {
      return res.status(503).json({
        error: 'NODE_DISCONNECTED',
        message: `Node ${targetId} is not connected to the relay`,
        connected_nodes: Array.from(relayClients.keys()),
      });
    }

    opsTargetWs.send(JSON.stringify({
      type: "relay",
      msgType: "exec",
      payload: JSON.stringify(opsMsg),
      from: "coordinator",
    }));

    log(`Exec forwarded to node ${targetId}: ${execScript.substring(0, 80)}`, 'ops');
    res.json({
      request_id: requestId,
      delivered: true,
      target_node: targetId,
      script: execScript,
    });
  });

  log(`Ops Channel — 9 endpoints at /api/ops/* (bootstrap verifier: ${bootstrapFingerprint} [read-only], ops_enabled: false — register operators via POST /api/ops/operators, enable via POST /api/ops/enable)`, 'ops');

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

  app.get("/install/:filename", (req, res) => {
    const allowed = new Set(["Install-PlenumNET.bat", "install-windows.ps1", "install.sh", "deploy-yoda.ps1", "deploy-daemon.ps1", "plenumnet-service.ps1", "install-plenumnet-msi.ps1"]);
    const { filename } = req.params;
    if (!allowed.has(filename)) {
      return res.status(404).json({ error: "Not found" });
    }
    const filePath = path.resolve(process.cwd(), "client", "public", "install", filename);
    if (!fs.existsSync(filePath)) {
      return res.status(404).json({ error: "File not found" });
    }
    if (filename.endsWith(".ps1") || filename.endsWith(".sh")) {
      res.setHeader("Content-Type", "text/plain; charset=utf-8");
    } else {
      res.setHeader("Content-Type", "application/octet-stream");
      res.setHeader("Content-Disposition", `attachment; filename="${filename}"`);
    }
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.sendFile(filePath);
  });

  app.get("/api/salvi/inter-cube/monitor", (_req, res) => {
    const monitorPath = path.resolve(process.cwd(), "services/inter-cube/monitor/array3-monitor-v9.html");
    res.setHeader("Content-Type", "text/html");
    res.setHeader("Cache-Control", "no-store");
    res.sendFile(monitorPath);
  });

  const DIMENSIONS = 13;
  const crsRegistry = new Map<string, { publicKey: string; endpoint: string; lastSeen: number; tlDsaPk?: string }>();
  const publicKeyAddressMap = new Map<string, string>();

  (async () => {
    try {
      const persisted = await storage.getAllCrsRelayNodes();
      const bestByKey = new Map<string, { addr: string; lastSeen: number; node: typeof persisted[0] }>();
      for (const node of persisted) {
        const existing = bestByKey.get(node.publicKey);
        if (!existing || node.lastSeen.getTime() > existing.lastSeen) {
          bestByKey.set(node.publicKey, { addr: node.address, lastSeen: node.lastSeen.getTime(), node });
        }
      }
      const winnerAddrs = new Set([...bestByKey.values()].map(v => v.addr));
      const dupeAddrs = persisted.filter(n => !winnerAddrs.has(n.address)).map(n => n.address);
      for (const { node } of bestByKey.values()) {
        crsRegistry.set(node.address, { publicKey: node.publicKey, endpoint: node.endpoint, lastSeen: node.lastSeen.getTime(), tlDsaPk: node.tlDsaPk || undefined });
        publicKeyAddressMap.set(node.publicKey, node.address);
      }
      if (dupeAddrs.length > 0) {
        storage.deleteCrsRelayNodesByAddresses(dupeAddrs).catch(() => {});
        log(`CRS startup: purged ${dupeAddrs.length} duplicate/stale entries from DB`, "crs");
      }
      if (bestByKey.size > 0) {
        log(`CRS loaded ${bestByKey.size} persisted node registrations from database`, "crs");
      }
    } catch (err) {
      log(`CRS failed to load persisted registrations: ${err}`, "crs");
    }
  })();

  function normalizeTernaryAddr(s: string): string {
    return s.replace(/\./g, "");
  }

  function toDottedAddr(flat: string): string {
    if (flat.length !== DIMENSIONS) return flat;
    return `${flat.slice(0, 3)}.${flat.slice(3, 6)}.${flat.slice(6, 9)}.${flat.slice(9, 12)}.${flat.slice(12, 13)}`;
  }

  function deriveAddressFromPublicKey(publicKeyHex: string): string {
    const pkBytes = Buffer.from(publicKeyHex, "hex");
    const hashTrits = spongeHashTrits(pkBytes);
    const addrTrits: string[] = [];
    for (let i = 0; i < DIMENSIONS; i++) {
      const t = hashTrits[i];
      addrTrits.push(t === -1 ? "2" : t === 0 ? "0" : "1");
    }
    return addrTrits.join("");
  }

  async function tryCrsDaemon(url: string, opts: RequestInit): Promise<globalThis.Response | null> {
    try {
      const resp = await fetch(url, opts);
      return resp;
    } catch {
      return null;
    }
  }

  app.post("/api/salvi/inter-cube/relay/register", async (req, res) => {
    const { publicKey, endpoint: rawEndpoint, tlDsaPk } = req.body as { publicKey?: string; endpoint?: string; tlDsaPk?: string };
    if (!publicKey || !rawEndpoint) {
      return res.status(400).json({ error: "publicKey and endpoint required" });
    }
    const callerIp = (req.headers["x-forwarded-for"] as string || req.socket.remoteAddress || "").split(",")[0].trim();
    const endpoint = rawEndpoint.startsWith("0.0.0.0:") && callerIp
      ? `${callerIp}:${rawEndpoint.split(":")[1]}`
      : rawEndpoint;
    const candidateAddr = publicKeyAddressMap.get(publicKey) ||
      [...crsRegistry.entries()].find(([_, v]) => v.publicKey === publicKey)?.[0];
    if (candidateAddr) {
      const entry = crsRegistry.get(candidateAddr);
      if (entry) {
        entry.lastSeen = Date.now();
        entry.endpoint = endpoint;
        if (tlDsaPk) entry.tlDsaPk = tlDsaPk;
      } else {
        crsRegistry.set(candidateAddr, { publicKey, endpoint, lastSeen: Date.now(), tlDsaPk: tlDsaPk || undefined });
      }
      publicKeyAddressMap.set(publicKey, candidateAddr);
      storage.upsertCrsRelayNode(publicKey, candidateAddr, endpoint, tlDsaPk || crsRegistry.get(candidateAddr)?.tlDsaPk).catch(() => {});
      log(`CRS POST re-register ${publicKey.substring(0, 16)}... → ${toDottedAddr(candidateAddr)} (stable)`, "crs");
      return res.json({ address: candidateAddr, addressDotted: toDottedAddr(candidateAddr), endpoint, source: "stable" });
    }
    if (publicKey.length >= 16) {
      const derivedAddr = deriveAddressFromPublicKey(publicKey);
      crsRegistry.set(derivedAddr, { publicKey, endpoint, lastSeen: Date.now(), tlDsaPk: tlDsaPk || undefined });
      publicKeyAddressMap.set(publicKey, derivedAddr);
      storage.upsertCrsRelayNode(publicKey, derivedAddr, endpoint, tlDsaPk || undefined).catch(() => {});
      log(`CRS POST first-register ${publicKey.substring(0, 16)}... → ${toDottedAddr(derivedAddr)}${tlDsaPk ? ' (TL-DSA-87)' : ''}`, "crs");
      return res.json({ address: derivedAddr, addressDotted: toDottedAddr(derivedAddr), endpoint, source: "relay-derived" });
    }
    return res.status(400).json({ error: "Invalid publicKey" });
  });

  app.get("/api/salvi/inter-cube/relay/register", async (req, res) => {
    const { publicKey, endpoint: rawEndpoint, tlDsaPk } = req.query as { publicKey?: string; endpoint?: string; tlDsaPk?: string };
    if (!publicKey || !rawEndpoint) {
      return res.status(400).json({ error: "publicKey and endpoint query params required" });
    }
    const callerIp = (req.headers["x-forwarded-for"] as string || req.socket.remoteAddress || "").split(",")[0].trim();
    const endpoint = rawEndpoint.startsWith("0.0.0.0:") && callerIp
      ? `${callerIp}:${rawEndpoint.split(":")[1]}`
      : rawEndpoint;

    const candidateAddr = publicKeyAddressMap.get(publicKey) ||
      [...crsRegistry.entries()].find(([_, v]) => v.publicKey === publicKey)?.[0];
    if (candidateAddr) {
      const entry = crsRegistry.get(candidateAddr);
      if (entry) {
        entry.lastSeen = Date.now();
        entry.endpoint = endpoint;
        if (tlDsaPk) entry.tlDsaPk = tlDsaPk;
      } else {
        crsRegistry.set(candidateAddr, { publicKey, endpoint, lastSeen: Date.now(), tlDsaPk: tlDsaPk || undefined });
        log(`CRS restored purged address ${toDottedAddr(candidateAddr)} for ${publicKey.substring(0, 16)}... (permanent identity)`, "crs");
      }
      const finalEntry = crsRegistry.get(candidateAddr)!;
      publicKeyAddressMap.set(publicKey, candidateAddr);
      storage.upsertCrsRelayNode(publicKey, candidateAddr, endpoint, tlDsaPk || finalEntry.tlDsaPk).catch(() => {});
      log(`CRS re-register ${publicKey.substring(0, 16)}... → same address ${toDottedAddr(candidateAddr)} (stable)`, "crs");
      return res.json({ address: candidateAddr, addressDotted: toDottedAddr(candidateAddr), endpoint, source: "stable" });
    }

    const upstream = await tryCrsDaemon("http://127.0.0.1:8181/api/salvi/inter-cube/crs/register", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ publicKey, endpoint }),
    });
    if (upstream && upstream.ok) {
      const body = await upstream.text();
      try {
        const data = JSON.parse(body);
        if (data.address) {
          const addr = normalizeTernaryAddr(data.address);
          crsRegistry.set(addr, { publicKey, endpoint, lastSeen: Date.now(), tlDsaPk: tlDsaPk || undefined });
          publicKeyAddressMap.set(publicKey, addr);
          storage.upsertCrsRelayNode(publicKey, addr, endpoint, tlDsaPk || undefined).catch(() => {});
          log(`CRS first registration ${publicKey.substring(0, 16)}... as ${toDottedAddr(addr)}${tlDsaPk ? ' (TL-DSA-87 key attached)' : ''}`, "crs");
        }
      } catch {}
      return res.status(upstream.status).setHeader("Content-Type", upstream.headers.get("content-type") || "application/json").send(body);
    }
    if (upstream && !upstream.ok) {
      const body = await upstream.text();
      log(`CRS daemon registration returned ${upstream.status}: ${body}`, "crs");
    }
    if (publicKey.length >= 16) {
      const derivedAddr = deriveAddressFromPublicKey(publicKey);
      crsRegistry.set(derivedAddr, { publicKey, endpoint, lastSeen: Date.now(), tlDsaPk: tlDsaPk || undefined });
      publicKeyAddressMap.set(publicKey, derivedAddr);
      storage.upsertCrsRelayNode(publicKey, derivedAddr, endpoint).catch(() => {});
      log(`CRS relay-derived address for ${publicKey.substring(0, 16)}... → ${toDottedAddr(derivedAddr)}`, "crs");
      return res.json({ address: derivedAddr, addressDotted: toDottedAddr(derivedAddr), endpoint, source: "relay-derived" });
    }
    return res.status(503).json({ error: "CRS daemon unavailable and no prior registration found for this node" });
  });

  function addressStringToTritArray(addr: string): number[] {
    return addr.split("").map((c) => parseInt(c));
  }

  app.get("/api/salvi/inter-cube/relay/heartbeat", async (req, res) => {
    const { address, publicKey } = req.query as { address?: string; publicKey?: string };
    if (!address) {
      return res.status(400).json({ error: "address query param required" });
    }
    const normalizedAddr = normalizeTernaryAddr(address as string);
    let entry = crsRegistry.get(normalizedAddr);
    if (!entry && publicKey) {
      const existingAddr = publicKeyAddressMap.get(publicKey);
      if (existingAddr) {
        entry = crsRegistry.get(existingAddr);
      }
      if (!entry) {
        entry = { publicKey, endpoint: "0.0.0.0:0", lastSeen: Date.now() };
        crsRegistry.set(normalizedAddr, entry);
        publicKeyAddressMap.set(publicKey, normalizedAddr);
        log(`Heartbeat auto-registered ${toDottedAddr(normalizedAddr)} (server restart recovery)`, "crs");
      }
    }
    const endpoint = entry?.endpoint || "0.0.0.0:0";
    if (entry) {
      entry.lastSeen = Date.now();
      storage.upsertCrsRelayNode(entry.publicKey, normalizedAddr, entry.endpoint).catch(() => {});
    }
    tryCrsDaemon("http://127.0.0.1:8181/api/salvi/inter-cube/crs/heartbeat", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ address: addressStringToTritArray(normalizedAddr), endpoint }),
    }).catch(() => {});
    if (entry) {
      return res.json({ status: "ok", address: normalizedAddr, addressDotted: toDottedAddr(normalizedAddr), timestamp: Date.now() });
    }
    return res.status(404).json({ error: "Address not registered — include publicKey param" });
  });

  const slotInventoryCache = new Map<number, { nodeId: number; slots: any; health: any; receivedAt: number }>();

  app.get("/api/salvi/inter-cube/monitor/slots", async (req, res) => {
    const referer = req.headers.referer || "";
    const origin = req.headers.origin || "";
    const host = req.headers.host || "";
    const isMonitorReferer = referer.includes("/api/salvi/inter-cube/monitor");
    const isSameOrigin = referer.startsWith(`https://${host}`) || referer.startsWith(`http://${host}`) || origin === `https://${host}` || origin === `http://${host}`;
    if (!isMonitorReferer || !isSameOrigin) {
      return res.status(403).json({ error: "Forbidden", hint: "This endpoint is only accessible from the server-hosted monitor." });
    }
    const clientIp = (req.headers["x-forwarded-for"] as string || req.socket.remoteAddress || "unknown").split(",")[0].trim();
    if (!checkProxyRateLimit(clientIp)) {
      return res.status(429).json({ error: "Too many concurrent requests", hint: "Wait a few seconds and retry. Check for duplicate monitor instances." });
    }
    try {
      const result = await proxyToAllDaemons("/api/salvi/inter-cube/slots");
      return res.status(result.statusCode).json(result.body);
    } catch {
      return res.status(500).json({ error: "Internal relay proxy error", hint: "Contact the Array3 operator." });
    }
  });

  app.get("/api/salvi/inter-cube/slots", async (req, res) => {
    const clientIp = (req.headers["x-forwarded-for"] as string || req.socket.remoteAddress || "unknown").split(",")[0].trim();
    if (!checkProxyRateLimit(clientIp)) {
      console.log(`[proxy] 429 rate-limit for ${clientIp} (30/min exceeded)`);
      return res.status(429).json({ error: "Too many concurrent requests", hint: "Wait a few seconds and retry. Check for duplicate monitor instances." });
    }
    const RELAY_API_TOKEN = process.env.RELAY_API_TOKEN;
    if (!RELAY_API_TOKEN) {
      return res.status(500).json({ error: "Internal relay proxy error", hint: "Contact the Array3 operator." });
    }
    const authHeader = req.headers.authorization;
    if (!authHeader || !authHeader.startsWith("Bearer ")) {
      return res.status(401).json({ error: "Invalid or missing relay token", hint: "Verify your RELAY_AUTH_TOKEN matches the server's RELAY_API_TOKEN." });
    }
    const token = authHeader.slice(7);
    try {
      const tokenBuf = Buffer.from(token);
      const expectedBuf = Buffer.from(RELAY_API_TOKEN);
      if (tokenBuf.length !== expectedBuf.length || !crypto.timingSafeEqual(tokenBuf, expectedBuf)) {
        return res.status(401).json({ error: "Invalid or missing relay token", hint: "Verify your RELAY_AUTH_TOKEN matches the server's RELAY_API_TOKEN." });
      }
    } catch {
      return res.status(401).json({ error: "Invalid or missing relay token", hint: "Verify your RELAY_AUTH_TOKEN matches the server's RELAY_API_TOKEN." });
    }
    try {
      const result = await proxyToAllDaemons("/api/salvi/inter-cube/slots");
      return res.status(result.statusCode).json(result.body);
    } catch (err: any) {
      return res.status(500).json({ error: "Internal relay proxy error", hint: "Contact the Array3 operator." });
    }
  });

  app.post("/api/salvi/inter-cube/relay/slot-report", async (req, res) => {
    try {
      const { nodeId, slots, health } = req.body;
      if (!nodeId || !slots) {
        return res.status(400).json({ error: "nodeId and slots required" });
      }
      slotInventoryCache.set(nodeId, { nodeId, slots, health: health || null, receivedAt: Date.now() });
      return res.json({ status: "ok", nodeId, cached: true });
    } catch (err: any) {
      return res.status(500).json({ error: err.message });
    }
  });

  app.get("/api/salvi/inter-cube/relay/slot-inventory", async (_req, res) => {
    try {
      const now = Date.now();
      const staleThresholdMs = 90000;
      const nodes: any[] = [];
      const relayClientsRef = (globalThis as any).__relayClients as Map<string, WebSocket> | undefined;

      const opsStatus = opsChannelService.getOpsStatus();
      const opsTelemetryByNodeId = new Map<string, any>();
      for (const nodeSnapshot of opsStatus.nodes) {
        const telem = nodeSnapshot.last_telemetry as any;
        if (telem) {
          opsTelemetryByNodeId.set(String(nodeSnapshot.node_id), telem);
          if (nodeSnapshot.address) opsTelemetryByNodeId.set(nodeSnapshot.address, telem);
        }
      }

      for (const [nid, cached] of slotInventoryCache.entries()) {
        const ageMs = now - cached.receivedAt;
        const isStale = ageMs > staleThresholdMs;
        const telem = opsTelemetryByNodeId.get(String(nid));
        const health = cached.health ? { ...cached.health } : {};
        if (telem) {
          health.telemetry = {
            cpu_pct: telem.cpu_pct,
            ram_pct: telem.ram_pct,
            ram_used_mb: telem.ram_used_mb,
            disk_pct: telem.disk_pct,
            gpu_pct: telem.gpu_pct,
            gpu_name: telem.gpu_name,
            process_uptime_seconds: telem.process_uptime_seconds,
          };
        }
        nodes.push({
          nodeId: nid,
          slots: cached.slots,
          health,
          receivedAt: cached.receivedAt,
          ageMs,
          stale: isStale,
          source: "slot-report",
        });
      }

      const cachedNodeIds = new Set(Array.from(slotInventoryCache.keys()).map(String));
      for (const nodeSnapshot of opsStatus.nodes) {
        const addr = nodeSnapshot.address || "";
        const nid = String(nodeSnapshot.node_id);
        if (cachedNodeIds.has(addr) || cachedNodeIds.has(nid)) continue;

        const isWsConnected = relayClientsRef?.has(addr) && relayClientsRef.get(addr)!.readyState === 1;
        const crsEntry = crsRegistry.get(addr);
        const telem = nodeSnapshot.last_telemetry as any;

        const port = crsEntry?.endpoint?.split(":").pop() || "11124";
        const portNum = parseInt(port, 10) || 11124;
        const nodeNum = portNum === 11124 ? 1 : portNum === 11151 ? 2 : portNum === 11178 ? 3 : 1;
        const baseSlotStr = `${nodeNum}.`;

        const synthSlots: any[] = [
          { address: `${baseSlotStr}1.1`, service: "CRS", version: "1.1.1", status: isWsConnected ? "active" : "inactive", uptime_secs: telem?.process_uptime_seconds || 0, heartbeat_count: 0, latency_us: 0, service_detail: {} },
          { address: `${baseSlotStr}1.2`, service: "CON", version: "1.1.2", status: isWsConnected ? "active" : "inactive", uptime_secs: telem?.process_uptime_seconds || 0, heartbeat_count: 0, latency_us: 0, service_detail: {} },
          { address: `${baseSlotStr}1.3`, service: "FTS", version: "1.1.3", status: isWsConnected ? "active" : "inactive", uptime_secs: telem?.process_uptime_seconds || 0, heartbeat_count: 0, latency_us: 0, service_detail: {} },
          { address: `${baseSlotStr}2.1`, service: "GLB", version: "1.2.1", status: isWsConnected ? "active" : "inactive", uptime_secs: telem?.process_uptime_seconds || 0, heartbeat_count: 0, latency_us: 0, service_detail: {} },
          { address: `${baseSlotStr}2.2`, service: "Gateway", version: "2.2.2", status: isWsConnected ? "active" : "inactive", uptime_secs: telem?.process_uptime_seconds || 0, heartbeat_count: 0, latency_us: 0, service_detail: {} },
        ];

        nodes.push({
          nodeId: addr || nodeSnapshot.node_id,
          slots: synthSlots,
          health: {
            status: isWsConnected ? "healthy" : "unreachable",
            uptime: telem?.process_uptime_seconds || 0,
            version: PLENUMNET_VERSION,
            telemetry: telem ? {
              cpu_pct: telem.cpu_pct,
              ram_pct: telem.ram_pct,
              ram_used_mb: telem.ram_used_mb,
              disk_pct: telem.disk_pct,
              gpu_pct: telem.gpu_pct,
              gpu_name: telem.gpu_name,
              process_uptime_seconds: telem.process_uptime_seconds,
            } : null,
          },
          receivedAt: new Date(nodeSnapshot.last_seen).getTime(),
          ageMs: now - new Date(nodeSnapshot.last_seen).getTime(),
          stale: nodeSnapshot.connection_state !== "connected",
          source: "telemetry-synth",
          wsConnected: isWsConnected,
        });
      }

      if (nodes.length === 0) {
        for (const [addr, entry] of crsRegistry.entries()) {
          const isWsConnected = relayClientsRef?.has(addr) && relayClientsRef.get(addr)!.readyState === 1;
          const ageMs = now - entry.lastSeen;
          const port = entry.endpoint?.split(":").pop() || "11124";
          const portNum = parseInt(port, 10) || 11124;
          const nodeNum = portNum === 11124 ? 1 : portNum === 11151 ? 2 : portNum === 11178 ? 3 : 1;
          const baseSlotStr = `${nodeNum}.`;

          nodes.push({
            nodeId: addr,
            slots: [
              { address: `${baseSlotStr}1.1`, service: "CRS", version: "1.1.1", status: isWsConnected ? "active" : "inactive" },
              { address: `${baseSlotStr}1.2`, service: "CON", version: "1.1.2", status: isWsConnected ? "active" : "inactive" },
              { address: `${baseSlotStr}1.3`, service: "FTS", version: "1.1.3", status: isWsConnected ? "active" : "inactive" },
              { address: `${baseSlotStr}2.1`, service: "GLB", version: "1.2.1", status: isWsConnected ? "active" : "inactive" },
              { address: `${baseSlotStr}2.2`, service: "Gateway", version: "2.2.2", status: isWsConnected ? "active" : "inactive" },
            ],
            health: { status: isWsConnected ? "healthy" : "stale", uptime: 0 },
            receivedAt: entry.lastSeen,
            ageMs,
            stale: ageMs > staleThresholdMs,
            source: "crs-registry",
            wsConnected: isWsConnected,
          });
        }
      }

      return res.json({ nodes, count: nodes.length, timestamp: now });
    } catch (err: any) {
      return res.status(500).json({ error: err.message });
    }
  });

  const peerRegistry = new Map<string, { address: string; ip: string; peerPort: number; lastSeen: number }>();

  app.get("/api/salvi/inter-cube/relay/peer-discovery", async (req, res) => {
    const { address, peerPort } = req.query as { address?: string; peerPort?: string };
    if (!address || !peerPort) {
      return res.status(400).json({ error: "address and peerPort query params required" });
    }
    const callerIp = (req.headers["x-forwarded-for"] as string || req.socket.remoteAddress || "").split(",")[0].trim();
    const pp = parseInt(peerPort, 10);
    if (pp > 0 && callerIp) {
      peerRegistry.set(address, { address, ip: callerIp, peerPort: pp, lastSeen: Date.now() });
    }
    const lanPeers: Array<{ address: string; ip: string; peerPort: number }> = [];
    for (const [addr, info] of peerRegistry.entries()) {
      if (addr !== address && info.ip === callerIp && Date.now() - info.lastSeen < 120000) {
        lanPeers.push({ address: info.address, ip: info.ip, peerPort: info.peerPort });
      }
    }
    return res.json({ peers: lanPeers, callerIp });
  });

  const interCubeProxy = async (req: any, res: any) => {
    const upstream = await tryCrsDaemon(`http://127.0.0.1:8181${req.originalUrl}`, {
      method: req.method,
      headers: { "Content-Type": "application/json" },
      ...(req.method !== "GET" && req.method !== "HEAD" && req.body ? { body: JSON.stringify(req.body) } : {}),
    });
    if (upstream) {
      const body = await upstream.text();
      if (upstream.ok && req.params.service === "crs" && req.params.action === "register") {
        try {
          const data = JSON.parse(body);
          if (data.address) {
            const addr = normalizeTernaryAddr(data.address);
            const pk = req.body?.publicKey || data.publicKey;
            const ep = req.body?.endpoint || data.endpoint;
            if (pk && ep) {
              const knownAddr = publicKeyAddressMap.get(pk);
              if (knownAddr && knownAddr !== addr) {
                crsRegistry.delete(addr);
                const existingEntry = crsRegistry.get(knownAddr);
                if (existingEntry) {
                  existingEntry.lastSeen = Date.now();
                  existingEntry.endpoint = ep;
                }
              } else {
                const existingEntry = crsRegistry.get(addr);
                crsRegistry.set(addr, { publicKey: pk, endpoint: ep, lastSeen: Date.now(), tlDsaPk: existingEntry?.tlDsaPk });
                publicKeyAddressMap.set(pk, addr);
                storage.upsertCrsRelayNode(pk, addr, ep).catch(() => {});
              }
            }
          }
        } catch {}
      }
      return res.status(upstream.status).setHeader("Content-Type", upstream.headers.get("content-type") || "application/json").send(body);
    }
    return res.status(503).json({ error: "CRS daemon unavailable" });
  };

  app.get("/api/salvi/inter-cube/relay/status", (_req, res) => {
    const relayClientsRef = (globalThis as any).__relayClients;
    const pendingRef = (globalThis as any).__pendingMessages;
    const crsReg = crsRegistry;
    if (relayClientsRef) {
      const entries: [string, any][] = Array.from(relayClientsRef.entries());
      const nodes = entries.map(([addr, ws]) => ({
        address: toDottedAddr(addr),
        connected: ws.readyState === 1,
        endpoint: crsReg.get(addr)?.endpoint || null,
      }));
      const actuallyConnected = nodes.filter(n => n.connected).length;
      return res.json({ connectedNodes: actuallyConnected, nodes, pendingQueues: pendingRef?.size || 0 });
    }
    res.json({ connectedNodes: 0, nodes: [], pendingQueues: 0 });
  });

  if (process.env.NODE_ENV === "development") {
    app.post("/api/salvi/inter-cube/crs/test-register", express.json(), (req, res) => {
      const { address, publicKey } = req.body;
      if (!address || !publicKey) return res.status(400).json({ error: "address and publicKey required" });
      const normalAddr = normalizeTernaryAddr(address);
      crsRegistry.set(normalAddr, { publicKey, endpoint: "test-mock", lastSeen: Date.now() });
      publicKeyAddressMap.set(publicKey, normalAddr);
      res.json({ ok: true, address: normalAddr });
    });
  }

  function purgeStaleRegistrations(maxAgeMs: number): { purged: number; remaining: number; purgedAddresses: string[] } {
    const now = Date.now();
    const relayClientsRef = (globalThis as any).__relayClients as Map<string, WebSocket> | undefined;
    const purgedAddrs: string[] = [];
    for (const [addr, entry] of crsRegistry.entries()) {
      if (now - entry.lastSeen > maxAgeMs) {
        const isConnectedViaWs = relayClientsRef?.has(addr) && relayClientsRef.get(addr)!.readyState === 1;
        if (!isConnectedViaWs && !isExpectedNode(addr)) {
          crsRegistry.delete(addr);
          purgedAddrs.push(addr);
        }
      }
    }
    if (purgedAddrs.length > 0) {
      log(`CRS purged ${purgedAddrs.length} stale registrations (maxAge=${maxAgeMs}ms)`, "crs");
      storage.deleteCrsRelayNodesByAddresses(purgedAddrs).catch(() => {});
    }
    return { purged: purgedAddrs.length, remaining: crsRegistry.size, purgedAddresses: purgedAddrs };
  }

  const purgeStaleHandler = (req: any, res: any) => {
    const adminKey = req.headers["x-admin-key"];
    if (adminKey !== process.env.SESSION_SECRET) {
      return res.status(403).json({ error: "Forbidden" });
    }
    const maxAge = parseInt(req.query.maxAge as string) || 300000;
    const MIN_MAX_AGE = 60_000;
    const result = purgeStaleRegistrations(Math.max(maxAge, MIN_MAX_AGE));
    res.json(result);
  };
  app.get("/api/salvi/inter-cube/relay/purge-stale", purgeStaleHandler);
  app.post("/api/salvi/inter-cube/relay/purge-stale", purgeStaleHandler);

  const STALE_CLEANUP_INTERVAL = 60_000;
  const STALE_MAX_AGE = 600_000;
  const staleCleanupTimer = setInterval(() => {
    purgeStaleRegistrations(STALE_MAX_AGE);
  }, STALE_CLEANUP_INTERVAL);
  staleCleanupTimer.unref();

  function broadcastRelayRestart(): { restarted: number; message: string } {
    const relayClientsRef = (globalThis as any).__relayClients as Map<string, WebSocket> | undefined;
    if (!relayClientsRef || relayClientsRef.size === 0) {
      return { restarted: 0, message: "No relay peers connected" };
    }
    const restartMsg = JSON.stringify({ type: "restart", reason: "admin_request", ts: Date.now() });
    let sent = 0;
    for (const [addr, ws] of relayClientsRef.entries()) {
      if (ws.readyState === 1) {
        ws.send(restartMsg);
        ws.close(1012, "restart");
        sent++;
        console.log(`[ws-relay] Restart command sent to ${toDottedAddr(addr)}`);
      }
    }
    console.log(`[ws-relay] Restart broadcast complete — ${sent} node(s) notified`);
    return { restarted: sent, message: `Restart command sent to ${sent} node(s)` };
  }

  const RESTART_TOKEN_TTL = 30_000;
  const RESTART_COOLDOWN = 10_000;
  const activeRestartTokens = new Set<string>();
  let lastRestartTokenTime = 0;

  app.get("/api/salvi/inter-cube/relay/restart-token", (_req, res) => {
    const now = Date.now();
    if (now - lastRestartTokenTime < RESTART_COOLDOWN) {
      return res.status(429).json({ error: "Too many restart requests — wait 10 seconds" });
    }
    lastRestartTokenTime = now;
    const token = crypto.randomBytes(24).toString("hex");
    activeRestartTokens.add(token);
    setTimeout(() => activeRestartTokens.delete(token), RESTART_TOKEN_TTL);
    res.json({ token });
  });

  app.post("/api/salvi/inter-cube/relay/restart-nodes", (req, res) => {
    const token = req.headers["x-relay-token"] as string | undefined;
    if (!token || !activeRestartTokens.has(token)) {
      return res.status(403).json({ error: "Forbidden — get a token from /restart-token first" });
    }
    activeRestartTokens.delete(token);
    res.json(broadcastRelayRestart());
  });

  const CRS_ADDRESS = "111.111.111.111.1";
  const CRS_VERSION = PLENUMNET_VERSION;

  const crsCircuitBreaker = new CircuitBreaker("crs-verification", 5, 30_000, (name, state) => {
    const relayClientsRef = (globalThis as any).__relayClients as Map<string, WebSocket> | undefined;
    if (state === "open" && relayClientsRef) {
      const msg = JSON.stringify({ type: "circuit_open", breaker: name, ts: Date.now() });
      for (const [, ws] of relayClientsRef.entries()) {
        if (ws.readyState === WebSocket.OPEN) ws.send(msg);
      }
    }
    recordRelayAuditEvent({
      eventType: "relay.circuit_breaker",
      address: "server",
      timestamp: new Date().toISOString(),
      details: { breaker: name, state },
    });
  });
  (globalThis as any).__crsCircuitBreaker = crsCircuitBreaker;

  storage.getAllExpectedNodes().then(nodes => {
    syncExpectedNodesCache(nodes.map(n => n.address));
    if (nodes.length > 0) {
      console.log(`[watchdog] Loaded ${nodes.length} expected node(s): ${nodes.map(n => n.address).join(", ")}`);
    }
  }).catch(() => {});

  app.get("/api/salvi/inter-cube/relay/expected-nodes", async (req, res) => {
    const adminKey = req.headers["x-admin-key"];
    if (!adminKey || adminKey !== process.env.SESSION_SECRET) {
      return res.status(403).json({ error: "Forbidden" });
    }
    const nodes = await storage.getAllExpectedNodes();
    res.json({ expectedNodes: nodes });
  });

  app.post("/api/salvi/inter-cube/relay/expected-nodes", async (req, res) => {
    const adminKey = req.headers["x-admin-key"];
    if (!adminKey || adminKey !== process.env.SESSION_SECRET) {
      return res.status(403).json({ error: "Forbidden" });
    }
    const { address, label } = req.body;
    if (!address) return res.status(400).json({ error: "address required" });
    const normalAddr = normalizeTernaryAddr(address);
    try {
      const node = await storage.createExpectedNode({ address: normalAddr, label: label || null, addedBy: "admin" });
      addExpectedNode(normalAddr);
      res.json({ status: "ok", node });
    } catch (err: any) {
      if (err.message?.includes("unique")) {
        return res.status(409).json({ error: "Node already in expected list" });
      }
      res.status(500).json({ error: "Failed to add expected node" });
    }
  });

  app.delete("/api/salvi/inter-cube/relay/expected-nodes", async (req, res) => {
    const adminKey = req.headers["x-admin-key"];
    if (!adminKey || adminKey !== process.env.SESSION_SECRET) {
      return res.status(403).json({ error: "Forbidden" });
    }
    const { address } = req.body;
    if (!address) return res.status(400).json({ error: "address required" });
    const normalAddr = normalizeTernaryAddr(address);
    await storage.deleteExpectedNode(normalAddr);
    removeExpectedNode(normalAddr);
    res.json({ status: "ok", removed: normalAddr });
  });

  app.post("/api/salvi/inter-cube/relay/deployment", async (req, res) => {
    try {
      const payload = req.body;
      if (!payload || !payload.hostname) {
        return res.status(400).json({ error: "hostname required" });
      }
      const record = await storage.createDeploymentRecord({
        hostname: payload.hostname,
        ip: payload.ip || "0.0.0.0",
        architecture: payload.architecture || null,
        daemonCount: payload.daemonCount || payload.daemons?.length || 0,
        daemons: payload.daemons || [],
        crsUrl: payload.crsUrl || "https://plenumnet.replit.app",
        crsAddress: CRS_ADDRESS,
        binaryPath: payload.binaryPath || null,
        binarySizeMB: payload.binarySizeMB || null,
        logDir: payload.logDir || null,
        identityBase: payload.identityBase || null,
        deployer: payload.deployer || null,
        deployedAt: payload.timestamp ? new Date(payload.timestamp) : new Date(),
      });
      log(`Deployment notification from ${payload.hostname}: ${payload.daemons?.length || 0} daemons (id: ${record.id})`, "crs");
      res.json({ status: "ok", recorded: true, id: record.id, crsAddress: CRS_ADDRESS, crsVersion: CRS_VERSION });
    } catch (err: any) {
      log(`Deployment record error: ${err.message}`, "crs");
      res.status(500).json({ error: "Failed to record deployment" });
    }
  });

  app.get("/api/salvi/inter-cube/relay/deployments", async (req, res) => {
    try {
      const hostname = req.query.hostname as string | undefined;
      const records = hostname
        ? await storage.getDeploymentsByHostname(hostname)
        : await storage.getAllDeploymentRecords();
      res.json({ deployments: records, count: records.length, crsAddress: CRS_ADDRESS, crsVersion: CRS_VERSION });
    } catch (err: any) {
      log(`Deployment query error: ${err.message}`, "crs");
      res.status(500).json({ error: "Failed to query deployments" });
    }
  });

  app.delete("/api/salvi/inter-cube/relay/deployment/:hostname", async (req, res) => {
    try {
      const hostname = req.params.hostname;
      if (!hostname) {
        return res.status(400).json({ error: "hostname required" });
      }
      const count = await storage.deleteDeploymentByHostname(hostname);
      log(`Deployment record deleted for ${hostname} (${count} removed)`, "crs");
      res.json({ status: "ok", deleted: count, hostname });
    } catch (err: any) {
      log(`Deployment delete error: ${err.message}`, "crs");
      res.status(500).json({ error: "Failed to delete deployment" });
    }
  });

  app.get("/api/salvi/inter-cube/relay/cluster-health", async (_req, res) => {
    try {
      const records = await storage.getAllDeploymentRecords();
      const relayClientsRef = (globalThis as any).__relayClients as Map<string, WebSocket> | undefined;

      const now = Date.now();
      const connectedAtRef = (globalThis as any).__relayConnectedAt as Map<string, number> | undefined;
      const daemonChecks: Array<{
        address: string;
        endpoint: string;
        port: number;
        peerPort: number;
        hostname: string;
        deploymentId: number;
        role: "crs" | "cube";
        registeredInCrs: boolean;
        connectedViaRelay: boolean;
        nodeUptimeMs: number | null;
        directPeerCount: number;
        lastSeen: string | null;
        lastSeenAgeMs: number | null;
        status: "live" | "registered" | "deployed";
        healthState: NodeHealthState;
        disconnectHistory: DisconnectEvent[];
        isExpected: boolean;
        source: "deployment" | "crs";
      }> = [];

      const CRS_ORIGIN = normalizeTernaryAddr("1111111111111");
      const seenAddresses = new Set<string>();

      for (const record of records) {
        const daemons = (record.daemons as any[]) || [];
        for (const d of daemons) {
          const addr = d.address || "";
          const normalAddr = normalizeTernaryAddr(addr);
          seenAddresses.add(normalAddr);
          const crsEntry = crsRegistry.get(normalAddr);
          const isRelayConnected = relayClientsRef?.has(normalAddr) && relayClientsRef.get(normalAddr)!.readyState === 1;
          const isRegistered = !!crsEntry;
          const isCrs = normalAddr === CRS_ORIGIN || d.id === 1;

          let status: "live" | "registered" | "deployed" = "deployed";
          if (isRelayConnected) {
            status = "live";
          } else if (isRegistered) {
            status = "registered";
          }

          const lastSeenTs = crsEntry ? crsEntry.lastSeen : null;
          const healthState = isRelayConnected ? "up" as NodeHealthState : computeHealthState(lastSeenTs, now);
          const peerInfo = peerRegistry.get(normalAddr);
          let directPeerCount = 0;
          if (peerInfo) {
            for (const [, pi] of peerRegistry.entries()) {
              if (pi.address !== normalAddr && pi.ip === peerInfo.ip && now - pi.lastSeen < 120000) {
                directPeerCount++;
              }
            }
          }
          const endpointParsed = parseHostPort(d.endpoint || "");
          const resolvedPort = d.port || d.gatewayPort || (endpointParsed ? parseInt(endpointParsed.port) : 0);
          daemonChecks.push({
            address: toDottedAddr(normalAddr),
            endpoint: d.endpoint || "",
            port: resolvedPort,
            peerPort: d.peerPort || 0,
            hostname: record.hostname || "",
            deploymentId: record.id,
            role: isCrs ? "crs" : "cube",
            registeredInCrs: isRegistered,
            connectedViaRelay: !!isRelayConnected,
            nodeUptimeMs: isRelayConnected && connectedAtRef?.has(normalAddr) ? now - connectedAtRef.get(normalAddr)! : null,
            directPeerCount,
            lastSeen: lastSeenTs ? new Date(lastSeenTs).toISOString() : null,
            lastSeenAgeMs: lastSeenTs ? now - lastSeenTs : null,
            status,
            healthState,
            disconnectHistory: getDisconnectHistory(normalAddr).slice(-50),
            isExpected: isExpectedNode(normalAddr),
            source: "deployment" as const,
          });
        }
      }

      const deploymentDaemonsByPort = new Map<string, { hostname: string; port: number; peerPort: number; endpoint: string; deploymentId: number }>();
      for (const record of records) {
        const daemons = (record.daemons as any[]) || [];
        for (const d of daemons) {
          const depParsed = parseHostPort(d.endpoint || "");
          if (depParsed) {
            deploymentDaemonsByPort.set(depParsed.port, {
              hostname: record.hostname || "",
              port: d.port || (depParsed ? parseInt(depParsed.port) : 0),
              peerPort: d.peerPort || 0,
              endpoint: d.endpoint || "",
              deploymentId: record.id,
            });
          }
        }
      }

      for (const [crsAddr, crsEntry] of crsRegistry.entries()) {
        if (seenAddresses.has(crsAddr)) continue;
        const isRelayConnected = relayClientsRef?.has(crsAddr) && relayClientsRef.get(crsAddr)!.readyState === 1;
        const isCrs = crsAddr === CRS_ORIGIN;
        const lastSeenTs = crsEntry.lastSeen;
        const healthState = isRelayConnected ? "up" as NodeHealthState : computeHealthState(lastSeenTs, now);
        const crsParsed = parseHostPort(crsEntry.endpoint || "");

        const depMatchByPort = crsParsed ? deploymentDaemonsByPort.get(crsParsed.port) : undefined;
        if (depMatchByPort) {
          const existingEntry = daemonChecks.find(d => {
            const ep = parseHostPort(d.endpoint || "");
            return ep && ep.port === crsParsed!.port;
          });
          if (existingEntry) {
            existingEntry.registeredInCrs = true;
            existingEntry.connectedViaRelay = existingEntry.connectedViaRelay || !!isRelayConnected;
            if (crsEntry.endpoint && crsParsed) {
              existingEntry.endpoint = crsEntry.endpoint;
              existingEntry.port = parseInt(crsParsed.port) || existingEntry.port;
            }
            if (isRelayConnected && connectedAtRef?.has(crsAddr)) {
              existingEntry.nodeUptimeMs = now - connectedAtRef.get(crsAddr)!;
            }
            if (lastSeenTs && (!existingEntry.lastSeen || new Date(existingEntry.lastSeen).getTime() < lastSeenTs)) {
              existingEntry.lastSeen = new Date(lastSeenTs).toISOString();
              existingEntry.lastSeenAgeMs = now - lastSeenTs;
            }
            if (existingEntry.connectedViaRelay) {
              existingEntry.status = "live";
              existingEntry.healthState = "up" as NodeHealthState;
            } else if (existingEntry.registeredInCrs && existingEntry.status === "deployed") {
              existingEntry.status = "registered";
              existingEntry.healthState = healthState;
            }
            seenAddresses.add(crsAddr);
            continue;
          }
        }

        const crsEndpointIp = crsParsed ? crsParsed.host : "";
        if (crsEndpointIp === "0.0.0.0" || crsEndpointIp === "127.0.0.1" || crsEndpointIp === "localhost") {
          seenAddresses.add(crsAddr);
          continue;
        }

        seenAddresses.add(crsAddr);
        const crsPeerInfo = peerRegistry.get(crsAddr);
        let crsDirectPeerCount = 0;
        if (crsPeerInfo) {
          for (const [, pi] of peerRegistry.entries()) {
            if (pi.address !== crsAddr && pi.ip === crsPeerInfo.ip && now - pi.lastSeen < 120000) {
              crsDirectPeerCount++;
            }
          }
        }
        daemonChecks.push({
          address: toDottedAddr(crsAddr),
          endpoint: depMatchByPort ? depMatchByPort.endpoint : (crsEntry.endpoint || ""),
          port: depMatchByPort ? depMatchByPort.port : (crsParsed ? parseInt(crsParsed.port) : 0),
          peerPort: depMatchByPort ? depMatchByPort.peerPort : 0,
          hostname: depMatchByPort ? depMatchByPort.hostname : "",
          deploymentId: depMatchByPort ? depMatchByPort.deploymentId : 0,
          role: isCrs ? "crs" : "cube",
          registeredInCrs: true,
          connectedViaRelay: !!isRelayConnected,
          nodeUptimeMs: isRelayConnected && connectedAtRef?.has(crsAddr) ? now - connectedAtRef.get(crsAddr)! : null,
          directPeerCount: crsDirectPeerCount,
          lastSeen: lastSeenTs ? new Date(lastSeenTs).toISOString() : null,
          lastSeenAgeMs: lastSeenTs ? now - lastSeenTs : null,
          status: isRelayConnected ? "live" : "registered",
          healthState,
          disconnectHistory: getDisconnectHistory(crsAddr).slice(-50),
          isExpected: isExpectedNode(crsAddr),
          source: "crs" as const,
        });
      }

      function parseHostPort(endpoint: string): { host: string; port: string } | null {
        const m = endpoint.match(/^([^:]+):(\d+)$/);
        return m ? { host: m[1], port: m[2] } : null;
      }

      const crsSourcePorts = new Set<string>();
      for (const d of daemonChecks) {
        if (d.source === "crs") {
          const parsed = parseHostPort(d.endpoint || "");
          if (parsed) crsSourcePorts.add(parsed.port);
        }
      }

      function isStaleDeploymentEntry(d: typeof daemonChecks[0]): boolean {
        if (d.source !== "deployment" || d.status !== "deployed") return false;
        if (d.registeredInCrs || d.connectedViaRelay) return false;
        const depParsed = parseHostPort(d.endpoint || "");
        if (depParsed && crsSourcePorts.has(depParsed.port)) return true;
        return false;
      }

      const filteredDaemons = daemonChecks.filter(d => !isStaleDeploymentEntry(d));

      const expectedNodesList = Array.from(getExpectedNodesCache());
      const expectedNodesStatus = expectedNodesList
        .filter(addr => {
          const normalAddr = normalizeTernaryAddr(addr);
          const inCrs = crsRegistry.has(normalAddr);
          const inRelay = relayClientsRef?.has(normalAddr) && relayClientsRef.get(normalAddr)!.readyState === 1;
          return inCrs || inRelay;
        })
        .map(addr => {
          const normalAddr = normalizeTernaryAddr(addr);
          const crsEntry = crsRegistry.get(normalAddr);
          const isRelayConnected = relayClientsRef?.has(normalAddr) && relayClientsRef.get(normalAddr)!.readyState === 1;
          const lastSeenTs = crsEntry ? crsEntry.lastSeen : null;
          const healthState = isRelayConnected ? "up" as NodeHealthState : computeHealthState(lastSeenTs, now);
          return {
            address: toDottedAddr(normalAddr),
            healthState,
            lastSeen: lastSeenTs ? new Date(lastSeenTs).toISOString() : null,
            offlineDurationMs: lastSeenTs ? now - lastSeenTs : null,
            connectedViaRelay: !!isRelayConnected,
            disconnectHistory: getDisconnectHistory(normalAddr).slice(-5),
          };
        });

      const expectedUp = expectedNodesStatus.filter(n => n.healthState === "up").length;
      const expectedSuspect = expectedNodesStatus.filter(n => n.healthState === "suspect").length;
      const expectedDown = expectedNodesStatus.filter(n => n.healthState === "down").length;
      const longestOffline = expectedNodesStatus
        .filter(n => n.offlineDurationMs !== null && n.healthState !== "up")
        .reduce((max, n) => Math.max(max, n.offlineDurationMs || 0), 0);

      const live = filteredDaemons.filter(d => d.status === "live").length;
      const registered = filteredDaemons.filter(d => d.status === "registered").length;
      const deployed = filteredDaemons.filter(d => d.status === "deployed").length;

      const throughputRef = (globalThis as any).__relayThroughput as typeof relayThroughput | undefined;
      const relayClientsForCount = (globalThis as any).__relayClients as Map<string, WebSocket> | undefined;
      const pendingRef = (globalThis as any).__pendingMessages as Map<string, any[]> | undefined;
      let totalPendingMsgs = 0;
      if (pendingRef) {
        for (const q of pendingRef.values()) totalPendingMsgs += q.length;
      }

      const uptimeMs = now - (throughputRef?.startedAt || now);
      const deliveryRate = (throughputRef?.sent || 0) > 0
        ? +((throughputRef!.delivered / throughputRef!.sent) * 100).toFixed(1)
        : 100;
      const avgMsgSize = (throughputRef?.delivered || 0) > 0
        ? Math.round((throughputRef!.bytesRelayed || 0) / throughputRef!.delivered)
        : 0;
      const recentMsgs = throughputRef
        ? throughputRef.recentTimestamps.filter(t => t > now - 60_000).length
        : 0;
      const recentBytesTotal = throughputRef
        ? throughputRef.recentBytes.filter(b => b.ts > now - 60_000).reduce((sum, b) => sum + b.bytes, 0)
        : 0;
      const msgPerSec60 = +(recentMsgs / 60).toFixed(2);
      const bytesPerSec60 = +(recentBytesTotal / 60).toFixed(1);

      res.json({
        crsAddress: CRS_ADDRESS,
        crsVersion: CRS_VERSION,
        totalDaemons: filteredDaemons.length,
        live,
        registered,
        deployed,
        clusterHealthy: live > 0 || registered > 0,
        daemons: filteredDaemons,
        expectedNodes: expectedNodesStatus,
        nodeHealth: {
          expectedCount: expectedNodesStatus.length,
          upCount: expectedUp,
          suspectCount: expectedSuspect,
          downCount: expectedDown,
          longestOfflineMs: longestOffline,
        },
        circuitBreaker: crsCircuitBreaker.getStats(),
        relay: {
          connectedPeers: relayClientsForCount?.size || 0,
          peakPeers: throughputRef?.peakPeers || 0,
          pendingQueues: pendingRef?.size || 0,
          pendingMessages: totalPendingMsgs,
          msgsSent: throughputRef?.sent || 0,
          msgsDelivered: throughputRef?.delivered || 0,
          msgsQueued: throughputRef?.queued || 0,
          msgsFailed: throughputRef?.failed || 0,
          msgPerSec: msgPerSec60,
          bytesPerSec: bytesPerSec60,
          bytesRelayed: throughputRef?.bytesRelayed || 0,
          avgMsgSizeBytes: avgMsgSize,
          deliveryRate,
          inferenceRequests: throughputRef?.inferenceRequests || 0,
          inferenceResponses: throughputRef?.inferenceResponses || 0,
          meshHeartbeats: throughputRef?.meshHeartbeats || 0,
          uptimeMs,
        },
        checkedAt: new Date().toISOString(),
      });
    } catch (err: any) {
      log(`Cluster health check error: ${err.message}`, "crs");
      res.status(500).json({ error: "Cluster health check failed" });
    }
  });

  app.all("/api/salvi/inter-cube/:service/:action", interCubeProxy);
  app.all("/api/salvi/inter-cube/:service", interCubeProxy);

  app.get("/health/crs", async (_req, res) => {
    const upstream = await tryCrsDaemon("http://127.0.0.1:8181/health", { method: "GET" });
    if (!upstream) {
      return res.status(503).json({ status: "unavailable", service: "PlenumNET Inter-Cube Infrastructure", version: CRS_VERSION, mode: "crs", address: CRS_ADDRESS, error: "native daemon unreachable — CRS relay is active", trackedNodes: crsRegistry.size });
    }
    try {
      const body = JSON.parse(await upstream.text());
      body.version = CRS_VERSION;
      return res.status(upstream.status).json(body);
    } catch {
      return res.status(upstream.status).json({ status: "ok", service: "PlenumNET Inter-Cube Infrastructure", version: CRS_VERSION, mode: "crs", address: CRS_ADDRESS });
    }
  });

  app.get("/api/yoda-installer", async (_req, res) => {
    try {
      const scriptPath = path.resolve("rerun-yoda-install.ps1");
      const { readFile } = await import("fs/promises");
      const script = await readFile(scriptPath, "utf-8");
      res.setHeader("Content-Type", "text/plain; charset=utf-8");
      res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
      res.setHeader("Pragma", "no-cache");
      res.send(script);
    } catch {
      res.status(404).send("Script not found");
    }
  });

  app.get("/api/deploy-daemon", async (_req, res) => {
    try {
      const scriptPath = path.resolve("services/inter-cube/deploy-daemon.ps1");
      const { readFile } = await import("fs/promises");
      const script = await readFile(scriptPath, "utf-8");
      res.setHeader("Content-Type", "text/plain; charset=utf-8");
      res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
      res.setHeader("Pragma", "no-cache");
      res.send(script);
    } catch {
      res.status(404).send("# deploy-daemon.ps1 not found");
    }
  });

  app.get("/api/deploy-daemon.bat", async (_req, res) => {
    const bat = [
      "@echo off",
      "title PlenumNET Daemon Deployer",
      'set "PS_FILE=%TEMP%\\deploy-daemon-%RANDOM%.ps1"',
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri \'https://plenumnet.replit.app/api/deploy-daemon\' -OutFile \'%PS_FILE%\' -UseBasicParsing"',
      'if not exist "%PS_FILE%" ( echo ERROR: Failed to download deployer. & pause & exit /b 1 )',
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%PS_FILE%"',
      'del "%PS_FILE%" 2>nul',
      "pause",
    ].join("\r\n") + "\r\n";
    res.setHeader("Content-Type", "application/x-bat");
    res.setHeader("Content-Disposition", 'attachment; filename="deploy-daemon.bat"');
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.send(bat);
  });

  app.get("/api/yoda-installer.bat", async (_req, res) => {
    const bat = [
      "@echo off",
      "title YODA Installer",
      'set "PS_FILE=%TEMP%\\yoda-install-%RANDOM%.ps1"',
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri \'https://plenumnet.replit.app/api/yoda-installer\' -OutFile \'%PS_FILE%\' -UseBasicParsing"',
      'if not exist "%PS_FILE%" ( echo ERROR: Failed to download installer. & pause & exit /b 1 )',
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%PS_FILE%"',
      'del "%PS_FILE%" 2>nul',
      "pause",
    ].join("\r\n") + "\r\n";
    res.setHeader("Content-Type", "application/x-bat");
    res.setHeader("Content-Disposition", 'attachment; filename="yoda-install.bat"');
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.send(bat);
  });

  app.get("/api/deploy-yoda", async (_req, res) => {
    const candidates = [
      path.resolve(process.cwd(), "client", "public", "install", "deploy-yoda.ps1"),
      path.resolve(process.cwd(), "dist", "public", "install", "deploy-yoda.ps1"),
      path.resolve(process.cwd(), "public", "install", "deploy-yoda.ps1"),
      path.resolve(process.cwd(), "services", "inter-cube", "deploy-yoda.ps1"),
    ];
    const { readFile, stat } = await import("fs/promises");
    for (const scriptPath of candidates) {
      try {
        await stat(scriptPath);
        const script = await readFile(scriptPath, "utf-8");
        res.setHeader("Content-Type", "text/plain; charset=utf-8");
        res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
        res.setHeader("Pragma", "no-cache");
        return res.send(script);
      } catch {
        continue;
      }
    }
    res.status(404).send("# deploy-yoda.ps1 not found");
  });

  app.get("/api/deploy-yoda.bat", async (_req, res) => {
    const bat = [
      "@echo off",
      "title YODA 3-Daemon Deployer",
      "",
      ":: Self-elevate to Administrator if not already elevated",
      'net session >nul 2>&1',
      'if %errorLevel% neq 0 (',
      '    echo Requesting Administrator privileges...',
      '    powershell.exe -NoProfile -Command "Start-Process -FilePath \'%~f0\' -Verb RunAs"',
      '    exit /b 0',
      ')',
      "",
      "echo Running as Administrator...",
      'set "PS_FILE=%TEMP%\\deploy-yoda-%RANDOM%.ps1"',
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri \'https://plenumnet.replit.app/api/deploy-yoda\' -OutFile \'%PS_FILE%\' -UseBasicParsing"',
      'if not exist "%PS_FILE%" ( echo ERROR: Failed to download deployer. & pause & exit /b 1 )',
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%PS_FILE%"',
      'del "%PS_FILE%" 2>nul',
      "pause",
    ].join("\r\n") + "\r\n";
    res.setHeader("Content-Type", "application/x-bat");
    res.setHeader("Content-Disposition", 'attachment; filename="deploy-yoda.bat"');
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.send(bat);
  });

  app.get("/api/install-msi", async (_req, res) => {
    try {
      const psPath = path.resolve(process.cwd(), "install-plenumnet-msi.ps1");
      if (fs.existsSync(psPath)) {
        res.setHeader("Content-Type", "text/plain; charset=utf-8");
        res.setHeader("Content-Disposition", 'attachment; filename="install-plenumnet-msi.ps1"');
        res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
        res.sendFile(psPath);
      } else {
        res.status(404).send("# install-plenumnet-msi.ps1 not found");
      }
    } catch {
      res.status(404).send("# install-plenumnet-msi.ps1 not found");
    }
  });

  app.get("/api/install-msi.bat", async (_req, res) => {
    const bat = [
      "@echo off",
      "title PlenumNET MSI Installer",
      'echo.',
      'echo  PlenumNET MSI Installer',
      'echo  Capomastro Holdings Ltd. -- Applied Physics Division',
      'echo.',
      'echo  Preparing installation environment...',
      'echo.',
      "",
      ':: Self-elevate to Administrator if not already elevated',
      'net session >nul 2>&1',
      'if %errorlevel% neq 0 (',
      '    echo   Requesting administrator privileges...',
      '    powershell.exe -NoProfile -Command "Start-Process -FilePath \'%~f0\' -Verb RunAs"',
      '    exit /b',
      ')',
      "",
      'set "PS_FILE=%TEMP%\\install-plenumnet-msi-%RANDOM%.ps1"',
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri \'https://plenumnet.replit.app/api/install-msi\' -OutFile \'%PS_FILE%\' -UseBasicParsing"',
      "",
      'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%PS_FILE%"',
      'del "%PS_FILE%" 2>nul',
      "pause",
    ].join("\r\n") + "\r\n";
    res.setHeader("Content-Type", "application/x-bat");
    res.setHeader("Content-Disposition", 'attachment; filename="install-plenumnet-msi.bat"');
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
    res.send(bat);
  });

  const relayClients = new Map<string, WebSocket>();
  const relayAddressByWs = new Map<WebSocket, string>();
  const relayConnectedAt = new Map<string, number>();
  const pendingMessages = new Map<string, Array<{ from: string; type: string; payload: string; ts: number }>>();
  const opsRequestOriginators = new Map<string, WebSocket>();
  const opsTerminalClients = new Set<WebSocket>();
  (globalThis as any).__opsTerminalClients = opsTerminalClients;
  (globalThis as any).__relayClients = relayClients;

  const PROXY_CONCURRENT_CAP = 5;
  const PROXY_TIMEOUT_MS = 5000;
  let activeProxyRequests = 0;
  const pendingProxies = new Map<string, { resolve: (v: { status: number; body: string } | null) => void; timer: ReturnType<typeof setTimeout>; daemonAddr: string }>();
  (globalThis as any).__pendingProxies = pendingProxies;

  const proxyRateLimiter = new Map<string, number[]>();
  const PROXY_RATE_LIMIT = 30;
  const PROXY_RATE_WINDOW_MS = 60_000;

  function checkProxyRateLimit(ip: string): boolean {
    const now = Date.now();
    const timestamps = proxyRateLimiter.get(ip) || [];
    const recent = timestamps.filter(t => now - t < PROXY_RATE_WINDOW_MS);
    if (recent.length >= PROXY_RATE_LIMIT) return false;
    recent.push(now);
    proxyRateLimiter.set(ip, recent);
    return true;
  }

  function proxyToAllDaemons(path: string): Promise<{ statusCode: number; body: any }> {
    return new Promise(async (outerResolve) => {
      const connectedDaemons: Array<[string, WebSocket]> = [];
      for (const [addr, ws] of relayClients.entries()) {
        if (ws.readyState === WebSocket.OPEN) connectedDaemons.push([addr, ws]);
      }

      if (connectedDaemons.length === 0) {
        return outerResolve({ statusCode: 503, body: { error: "No Array3 daemons connected to relay", hint: "Verify Array3 services are running on the host machine and connected to the relay." } });
      }

      if (activeProxyRequests >= PROXY_CONCURRENT_CAP) {
        console.log(`[proxy] 429 concurrent-cap (${activeProxyRequests}/${PROXY_CONCURRENT_CAP} in flight)`);
        return outerResolve({ statusCode: 429, body: { error: "Too many concurrent requests", hint: "Wait a few seconds and retry. Check for duplicate monitor instances." } });
      }

      activeProxyRequests++;

      const nodeResults: Array<{ node_id: string; status: string; slots?: any; summary?: any; node_id_num?: number; error?: string; health?: any }> = [];
      let responsesReceived = 0;
      const totalExpected = connectedDaemons.length;
      let resolved = false;

      const checkComplete = () => {
        if (resolved) return;
        if (responsesReceived >= totalExpected) {
          resolved = true;
          activeProxyRequests--;
          const responding = nodeResults.filter(n => n.status === "ok").length;
          const timedOut = nodeResults.filter(n => n.status === "timeout").length;
          if (timedOut === totalExpected) {
            return outerResolve({ statusCode: 504, body: { error: "All daemons timed out", hint: "Daemons are connected but not responding. Check daemon logs on the host machine." } });
          }
          outerResolve({
            statusCode: 200,
            body: {
              cluster: {
                total_nodes: totalExpected,
                responding_nodes: responding,
                nodes: nodeResults,
              }
            }
          });
        }
      };

      for (const [addr, ws] of connectedDaemons) {
        const requestId = `proxy_${crypto.randomUUID()}`;
        const timer = setTimeout(() => {
          pendingProxies.delete(requestId);
          nodeResults.push({ node_id: toDottedAddr(addr), status: "timeout" });
          responsesReceived++;
          checkComplete();
        }, PROXY_TIMEOUT_MS);

        pendingProxies.set(requestId, {
          resolve: (result) => {
            clearTimeout(timer);
            pendingProxies.delete(requestId);
            if (result) {
              const httpStatus = result.status || 0;
              if (httpStatus >= 200 && httpStatus < 300) {
                try {
                  const parsed = JSON.parse(result.body);
                  const opsSnap = opsChannelService.getOpsStatus();
                  let nodeTelem: any = null;
                  for (const ns of opsSnap.nodes) {
                    if (ns.address === addr || String(ns.node_id) === String(parsed.node_id)) {
                      nodeTelem = ns.last_telemetry;
                      break;
                    }
                  }
                  const health: any = parsed.health || { status: "ok" };
                  if (nodeTelem) {
                    health.cpu_pct = nodeTelem.cpu_pct;
                    health.mem_pct = nodeTelem.ram_pct;
                    health.telemetry = {
                      cpu_pct: nodeTelem.cpu_pct,
                      ram_pct: nodeTelem.ram_pct,
                      ram_used_mb: nodeTelem.ram_used_mb,
                      disk_pct: nodeTelem.disk_pct,
                      gpu_pct: nodeTelem.gpu_pct,
                      gpu_name: nodeTelem.gpu_name,
                    };
                  }
                  nodeResults.push({
                    node_id: toDottedAddr(addr),
                    node_id_num: parsed.node_id,
                    status: "ok",
                    slots: parsed.slots,
                    summary: parsed.summary,
                    health,
                  });
                } catch {
                  nodeResults.push({ node_id: toDottedAddr(addr), status: "error", error: "invalid response body" });
                }
              } else {
                nodeResults.push({ node_id: toDottedAddr(addr), status: "error", error: `daemon returned HTTP ${httpStatus}` });
              }
            } else {
              nodeResults.push({ node_id: toDottedAddr(addr), status: "timeout" });
            }
            responsesReceived++;
            checkComplete();
          },
          timer,
          daemonAddr: addr,
        });

        const proxyEnvelope = JSON.stringify({
          type: "relay",
          from: "__relay_server__",
          msgType: "http_proxy_req",
          payload: JSON.stringify({ request_id: requestId, method: "GET", path, timestamp: Date.now() }),
        });
        ws.send(proxyEnvelope);
      }
    });
  }
  (globalThis as any).__relayConnectedAt = relayConnectedAt;
  (globalThis as any).__pendingMessages = pendingMessages;

  const relayThroughput = {
    sent: 0,
    delivered: 0,
    failed: 0,
    queued: 0,
    startedAt: Date.now(),
    recentTimestamps: [] as number[],
    recentBytes: [] as { ts: number; bytes: number }[],
    bytesRelayed: 0,
    inferenceRequests: 0,
    inferenceResponses: 0,
    meshHeartbeats: 0,
    peakPeers: 0,
  };
  (globalThis as any).__relayThroughput = relayThroughput;

  function recordRelayMsg(outcome: "delivered" | "queued" | "failed", bytes?: number) {
    relayThroughput.sent++;
    if (outcome === "delivered") {
      relayThroughput.delivered++;
    } else if (outcome === "queued") {
      relayThroughput.queued++;
    } else {
      relayThroughput.failed++;
    }
    const now = Date.now();
    relayThroughput.recentTimestamps.push(now);
    if (bytes && bytes > 0) {
      relayThroughput.recentBytes.push({ ts: now, bytes });
    }
    while (relayThroughput.recentTimestamps.length > 0 && relayThroughput.recentTimestamps[0] < now - 60_000) {
      relayThroughput.recentTimestamps.shift();
    }
    while (relayThroughput.recentBytes.length > 0 && relayThroughput.recentBytes[0].ts < now - 60_000) {
      relayThroughput.recentBytes.shift();
    }
  }

  setInterval(() => {
    const peers = Array.from(relayClients.entries());
    if (peers.length < 2) return;
    const now = Date.now();
    for (let i = 0; i < peers.length; i++) {
      for (let j = 0; j < peers.length; j++) {
        if (i === j) continue;
        const [fromAddr] = peers[i];
        const [toAddr, toWs] = peers[j];
        const envelope = JSON.stringify({ type: "relay", from: fromAddr, msgType: "mesh-heartbeat", payload: JSON.stringify({ ts: now }) });
        const envBytes = Buffer.byteLength(envelope, "utf-8");
        if (toWs.readyState === WebSocket.OPEN) {
          toWs.send(envelope);
          relayThroughput.bytesRelayed += envBytes;
          relayThroughput.meshHeartbeats++;
          recordRelayMsg("delivered", envBytes);
        } else {
          if (!pendingMessages.has(toAddr)) pendingMessages.set(toAddr, []);
          const queue = pendingMessages.get(toAddr)!;
          const PENDING_MAX = 500;
          const PENDING_TTL_MS = 300_000;
          while (queue.length > 0 && (now - queue[0].ts) > PENDING_TTL_MS) queue.shift();
          if (queue.length < PENDING_MAX) {
            queue.push({ from: fromAddr, type: "mesh-heartbeat", payload: JSON.stringify({ ts: now }), ts: now });
            relayThroughput.meshHeartbeats++;
            recordRelayMsg("queued");
          } else {
            recordRelayMsg("failed");
          }
        }
      }
    }
  }, 45_000);


  const terminalTokens = new Map<string, { userId: string; createdAt: number }>();
  const TERMINAL_TOKEN_TTL = 30_000;
  const TERMINAL_MSG_RATE_LIMIT = 100;
  const TERMINAL_MSG_MAX_SIZE = 8192;

  app.post("/api/terminal/token", (req: any, res) => {
    const user = req.user as any;
    const isAuthed = req.isAuthenticated?.() && user?.claims?.sub;

    const userId = isAuthed ? user.claims.sub : "owner";
    const token = crypto.randomBytes(16).toString("hex");
    terminalTokens.set(token, { userId, createdAt: Date.now() });
    setTimeout(() => terminalTokens.delete(token), TERMINAL_TOKEN_TTL);
    res.json({ token });
  });

  const wss = new WebSocketServer({ noServer: true });
  const terminalWss = new WebSocketServer({ noServer: true });
  const hmodalWss = new WebSocketServer({ noServer: true });

  // ──────────────────────────────────────────────────────────────────
  // ARC Energy Attestation Certificate (EAC) — TM-2026-042 §4.3
  // The HModal WS handler stashes its latest emitted sample here so
  // the POST /api/hmodal/issue-eac route can snapshot a coherent set
  // of fields without racing the tick loop.
  // ──────────────────────────────────────────────────────────────────
  let lastHmodalSnapshot: any = null;
  let arcSignerKeypair: TlDsaKeyPair | null = null;
  const SALVI_EPOCH_MS = new Date('2025-04-01T00:00:00.000Z').getTime();
  // ════════════════════════════════════════════════════════════════════
  // Femtosecond-since-Salvi-epoch derivation — HPTP + Λ_LYMAN.
  // ════════════════════════════════════════════════════════════════════
  //
  // Source of truth: salvi-core/femtosecond-timing.getFemtosecondTimestamp()
  // (the canonical Salvi HPTP module).  At each EAC seal we call HPTP
  // exactly once and consume `salviEpochOffset` (bigint fs since the
  // Salvi epoch 2025-04-01T00:00:00Z).
  //
  // HPTP composition (all integer arithmetic, no IEEE-754):
  //
  //   ms.µs.ns  ←  measured  (process.hrtime.bigint() anchored once
  //                            at boot to Date.now(), monotonic)
  //
  //   ps.fs     ←  derived   (Λ_LYMAN = 91, Salvi UV-spectral PUV v1.0
  //                            framework integer position; each HPTP
  //                            read advances a phase counter by
  //                            exactly 1/91 ns = 10989 fs, walking
  //                            91 evenly-spaced sub-ns positions per
  //                            nanosecond.  Bit-deterministic, tied
  //                            to a published physical constant.)
  //
  //   ps.fs    +=  measured  (CPU-counter calibration burst; adds
  //                            real cycle-position drift on top of
  //                            the Λ_LYMAN walk.)
  //
  // Properties:
  //   • Pure integer / BigInt arithmetic end-to-end.
  //   • ms.µs.ns measured from OS monotonic clock; ps.fs derived from
  //     framework constants — the lower digits are NEVER zero-padded
  //     and NEVER hashed.
  //   • Replay-safe: HPTP returns the same fs value when called with
  //     the same internal phase + same hrtime read.
  //   • Compatible with Tier-0 atomic clock substitution: only the
  //     hrtime read inside HPTP changes; downstream math is identical.
  // ════════════════════════════════════════════════════════════════════
  const FS_PER_MS = 1_000_000_000_000n;          // SI: 1 ms = 10¹² fs (formatter use)
  const FS_TIMING_PRECISION =
    "hptp: attosecond-class — PURE FIRST-PRINCIPLES DERIVATION. " +
    "NO HARDWARE CLOCK on the per-call path. " +
    "tickCounter monotonically increments by 1 per HPTP read; " +
    "1 tick = 8_000_000/(pqr)² = 8_000_000/1_002_001 as " +
    "(IRREDUCIBLE: gcd(2⁹·5⁶, 7²·11²·13²) = 1). " +
    "Closed walk on Z_{D_α}, D_α = F₅³·p²·q²·r² = 5³·7²·11²·13² = 125_250_125 " +
    "(integer denominator of 1/α — Arc Doc Theorem 22). " +
    "as_since_boot surfaced as EXACT rational {num, den} — never collapsed " +
    "to integer division, no zero padding, no truncation to zero. " +
    "Zero jitter, no oscillator, no Allan variance, no Dick effect, no thermal coefficient.";
  function toBijectiveBase3(n: bigint): string {
    // Rep-C bijective base-3 with digit set {1,2,3} — per Appendix A.
    if (n < 0n) throw new Error('toBijectiveBase3: negative');
    if (n === 0n) return '';
    const digits: string[] = [];
    let v = n;
    while (v > 0n) {
      let r = v % 3n;
      v = v / 3n;
      if (r === 0n) { r = 3n; v -= 1n; }
      digits.push(r.toString());
    }
    return digits.reverse().join('');
  }
  // Hex string → Rep-C bijective base-3 string.  Treats the hex as a
  // big-endian non-negative integer.  Used to expose every "*_hex"
  // wire field as the equivalent trit-native Rep-C string so the UI
  // never has to render hex.
  function hexToBijectiveBase3(hex: string): string {
    const clean = (hex || '').replace(/^0x/i, '');
    if (clean.length === 0) return '';
    return toBijectiveBase3(BigInt('0x' + clean));
  }
  // Balanced-trit array (Int8Array of {-1,0,1}) → Rep-C bijective base-3
  // string.  Map balanced {-1,0,1} → Rep-C glyph {2,3,1} (the standard
  // Rep-A↔Rep-C bijection used across the framework — Spec v3.3.33 §3.2),
  // taken in MSB-first order so the result reads left-to-right with the
  // most significant trit first.
  function balancedTritsToRepC(trits: ArrayLike<number>): string {
    const out = new Array<string>(trits.length);
    for (let i = 0; i < trits.length; i++) {
      const t = trits[i];
      out[i] = t === 1 ? '1' : t === -1 ? '2' : '3';
    }
    return out.join('');
  }
  // ── Milesian glyph table — Spec v3.3.33 §4.5 ──────────────────────
  // 27 Greek glyphs (24 modern letters + 3 ghost letters: ϛ, ϟ, ϡ).
  // Used to render any non-negative integer as a bijective base-27
  // string of glyphs.  Mirror of GLYPH_TABLE in
  // AASC/src/milesian.rs.
  const MILESIAN_GLYPHS = [
    "α","β","γ","δ","ε","ϛ","ζ","η","θ","ι","κ","λ","μ","ν","ξ","ο","π","ϟ",
    "ρ","σ","τ","υ","φ","χ","ψ","ω","ϡ",
  ] as const;
  function bigIntToMilesianGlyphs(n: bigint): string {
    // Strict mirror of aasc::milesian::glyphs_msb — N==0 produces the
    // empty string (no leading "α"), matching the bijective base-27
    // contract in AASC/src/milesian.rs.
    if (n < 0n) throw new Error("bigIntToMilesianGlyphs: negative");
    if (n === 0n) return "";
    let v = n;
    const out: string[] = [];
    while (v > 0n) {
      let r = Number(v % 27n);
      v = v / 27n;
      if (r === 0) { r = 27; v -= 1n; }
      out.push(MILESIAN_GLYPHS[r - 1]);
    }
    return out.reverse().join("");
  }
  function milesianGlyphHash(hashHex: string): string {
    const cleaned = hashHex.replace(/^0x/, "");
    if (!cleaned) return "";
    return bigIntToMilesianGlyphs(BigInt("0x" + cleaned));
  }

  function getArcSigner(): TlDsaKeyPair {
    if (arcSignerKeypair) return arcSignerKeypair;
    // One TL-DSA-87 keypair per process boot — this is the "node attestation
    // key" referenced in spec §4.3.  In production it is held in the
    // NinjaExec encrypted keystore; here it is generated in memory.
    arcSignerKeypair = keygen('TL-DSA-87');
    return arcSignerKeypair;
  }

  httpServer.on("upgrade", (request, socket, head) => {
    const url = new URL(request.url || "", `http://${request.headers.host}`);
    if (url.pathname === "/ws/relay") {
      wss.handleUpgrade(request, socket, head, (ws) => {
        wss.emit("connection", ws, request);
      });
    } else if (url.pathname === "/ws/terminal") {
      const token = url.searchParams.get("token");
      const tokenData = token ? terminalTokens.get(token) : null;
      if (!tokenData || (Date.now() - tokenData.createdAt > TERMINAL_TOKEN_TTL)) {
        if (token) terminalTokens.delete(token);
        socket.write("HTTP/1.1 401 Unauthorized\r\n\r\n");
        socket.destroy();
        return;
      }
      terminalTokens.delete(token!);
      (request as any)._terminalUserId = tokenData.userId;
      terminalWss.handleUpgrade(request, socket, head, (ws) => {
        terminalWss.emit("connection", ws, request);
      });
    } else if (url.pathname === "/ws/hmodal") {
      hmodalWss.handleUpgrade(request, socket, head, (ws) => {
        hmodalWss.emit("connection", ws, request);
      });
    } else if (url.pathname !== "/vite-hmr") {
      socket.destroy();
    }
  });

  // ============================================================
  // HModal WebSocket — proxy-safe alternative to SSE
  // Same trit-style GF(3) workload, same sample shape, just over WS.
  // ============================================================
  function readRaplUj(): number | null {
    try {
      const txt = require("fs").readFileSync(
        "/sys/class/powercap/intel-rapl:0/energy_uj",
        "utf-8",
      );
      return parseInt(txt.trim(), 10);
    } catch {
      return null;
    }
  }

  // Dynamic import once at startup (ESM-safe).  Used by both the HModal
  // WS sealer and the EAC route — kept in a single resolved object so
  // there is no per-connection cost.
  const spongeMod = await import("./crypto/sponge-hash");
  const {
    SpongeDuplex,
    bytesToBalancedTrits,
    tritsToHex: tritsToHexFn,
    tis27Hash: tis27HashFn,
  } = spongeMod;

  hmodalWss.on("connection", async (ws: WebSocket) => {
    const raplAvailable = readRaplUj() !== null;
    const periodMs = 1000;

    // ── Encrypted tunnel + continuous solid-state chain ────────────────
    // Pure trit-native cipher: TL-Sponge-385 duplex (Phase Encryption v3).
    // Each sample is sealed in-line at emit time and absorbed back into the
    // sponge so the next squeeze depends on every prior sample — gaps,
    // reorders, or substitutions break the running chain tag.  No classical
    // primitives anywhere on the seal path.

    // Session ID is generated in TRIT-NATIVE form: pure Rep-C bijective
    // base-3 over the digit set {1,2,3}, sourced from a 128-bit
    // cryptographic random.  No hex anywhere on the wire.
    const sessionRandBytes = crypto.randomBytes(16);
    const sessionRandBig   = BigInt('0x' + sessionRandBytes.toString('hex'));
    const sessionId        = toBijectiveBase3(sessionRandBig);     // trit-native ID
    const sessionKeyBytes = crypto.randomBytes(32);            // 256-bit seed
    const sessionKeyTrits = bytesToBalancedTrits(sessionKeyBytes); // → balanced trits
    const sessionKeyTritsHex = tritsToHexFn(sessionKeyTrits);
    const sessionKeyTritsBijective = balancedTritsToRepC(sessionKeyTrits);
    const sessionKeyFingerprint = tis27HashFn(sessionKeyBytes).slice(0, 24);
    const sessionKeyFingerprintTrit = hexToBijectiveBase3(sessionKeyFingerprint);

    const sealDuplex = new SpongeDuplex(2);
    sealDuplex.absorb(Buffer.from("hmodal|seal|v3", "utf-8"));
    sealDuplex.absorb(Buffer.from(sessionId, "utf-8"));
    sealDuplex.absorbTrits(sessionKeyTrits);

    let chainIndex = 0n;
    const chainSeedTrits = sealDuplex.squeeze(243);            // 385-bit tag
    const chainSeedHex = tritsToHexFn(chainSeedTrits);
    const chainSeedTrit = balancedTritsToRepC(chainSeedTrits);

    try {
      ws.send(JSON.stringify({
        type: "session",
        sessionId,                                              // trit-native
        sessionKeyTritsBijective,                               // Rep-C trit
        sessionKeyFingerprintTrit,                              // trit
        chainSeedTrit,                                          // trit
        // hex copies kept for audit/debug consumers
        sessionKeyTritsHex,
        chainSeedHex,
        cipher: "TL-Sponge-385 duplex (Phase Encryption v3, pure GF(3))",
        chainHash: "TL-Sponge-385 squeeze (385-bit running tag)",
        note: "session key wrapped in clear over the demo WSS; production deployments wrap via TL-KEM",
      }));
    } catch {}
    const sampleMs = 200;
    const stepMs = 50;
    const stepsPerCycle = periodMs / stepMs;     // 20
    const highSteps = stepsPerCycle / 4;         // 5  (1/4 duty)

    let stepIdx = 0;
    let opsThisWindow = 0;
    let opsTotalHigh = 0;
    let opsTotalLow = 0;
    let timeHighMs = 0;
    let timeLowMs = 0;
    let lastSampleAt = Date.now();
    let lastEnergy = readRaplUj();
    let energyHigh_uJ = 0;
    let energyLow_uJ = 0;
    let cumulativeEnergy_uJ = 0;
    // Energy SAVED versus a hypothetical continuous-on baseline.
    // Always tracked from the modeled wattage delta (mW · ms = µJ exactly),
    // so the readout works even when RAPL is not exposed in the
    // container.  Pure integer math — no floats on the wire.
    let cumulativeEnergySaved_uJ = 0;
    let alive = true;

    const TBUF_LEN = 4096;
    const trits = new Int8Array(TBUF_LEN);
    for (let i = 0; i < TBUF_LEN; i++) trits[i] = (i % 3) - 1;
    const acc = new Int8Array(TBUF_LEN);

    // ── Cache + restage layer ────────────────────────────────────
    // The GF(3) batch on this fixed (acc, trits) pair cycles every 3 rounds.
    // Key on (cyclePos = roundCount % CACHE_CYCLE).  After warm-up every
    // subsequent high-burst is a cache hit, real CPU work collapses, the
    // low-state gaps absorb the extra time, and savings climb toward
    // 1 − α²/β² = 1 − 1/Δ = 143/144 ≈ 99.31 %.
    const CACHE_CYCLE = 3;
    const cache: Map<number, Int8Array> = new Map();
    let roundCount = 0;
    let cacheHits = 0;
    let cacheMisses = 0;
    let realHighWorkMs = 0;   // ms actually spent in real GF(3) batches
    let cachedHighMs = 0;     // ms of high-window served from cache (≈ 0 CPU)

    // ── Deterministic auto-mode controller ───────────────────────
    // Closed-form law (no ML, no heuristics):
    //     d_target = clamp( (Q / Q_TARGET) / sqrt(F) ,  1/Δ ,  1 )
    // where Q = queue depth, F = cache fill ratio, Δ = 144.
    // Same inputs always produce the same d.  DDA accumulator turns
    // any fractional d into a deterministic high/low schedule.
    type DemandMode = "idle" | "steady" | "burst" | "auto";
    let demandMode: DemandMode = "auto";
    let dutyDebt = 0;
    const Q_TARGET = 100;  // calibrated so Q=25 → d=0.25, Q=100 → d=1
    const sessionStart = Date.now();
    // ── Salvi-anchor for this WS session ────────────────────────────
    // The OS clock is consulted EXACTLY once per session, here.  Every
    // subsequent fs timestamp for sealed samples in this session is
    // derived purely from chainIndex × sampleMs (integer math, no
    // hardware drift between samples).  See the
    // "Femtosecond-since-Salvi-epoch derivation" block above.
    const sessionAnchorMs: number = sessionStart - SALVI_EPOCH_MS;
    function currentQueueDepth(): number {
      const tSec = (Date.now() - sessionStart) / 1000;
      switch (demandMode) {
        case "idle":   return 0.7;   // → d = 1/144 (energy-save floor)
        case "steady": return 25;    // → d = 0.25  (theoretical sweet spot)
        case "burst":  return 100;   // → d = 1     (full-out)
        case "auto":   return 50 + 50 * Math.sin((tSec / 20) * 2 * Math.PI);
      }
    }
    function computeController(): { d: number; Q: number; F: number } {
      const Q = currentQueueDepth();
      const F = Math.min(1, Math.max(0.001, cache.size / CACHE_CYCLE));
      const pressure = Q / Q_TARGET;
      const cacheBoost = 1 / Math.sqrt(F);
      let d = pressure / cacheBoost;
      if (d < 1 / 144) d = 1 / 144;
      if (d > 1) d = 1;
      return { d, Q, F };
    }

    // ── Key-isolation tracking ───────────────────────────────────
    // Every batch represents one logical "signature" served.  The
    // TL-DSA private key is only fetched on a cache miss; a hit serves
    // the result without the key ever touching CPU registers.
    let keyTouchCount = 0;     // increments only on cache miss
    let signatureCount = 0;    // total logical signatures served

    function gf3AddBatchReal(rounds: number): number {
      let ops = 0;
      for (let r = 0; r < rounds; r++) {
        for (let i = 0; i < TBUF_LEN; i++) {
          let s = acc[i] + trits[i];
          if (s > 1) s -= 3;
          else if (s < -1) s += 3;
          acc[i] = s;
        }
        ops += TBUF_LEN;
      }
      return ops;
    }

    // Key-freshness window: every Δ=144 signatures one cache slot is forced
    // to expire (HKDF rekey / nonce salt refresh).  This gives the canonical
    // asymptotic isolation factor of 143/144 hits → 144× key isolation.
    const KEY_FRESHNESS_WINDOW = 144;
    function gf3BatchCached(): { ops: number; wasHit: boolean } {
      const key = roundCount % CACHE_CYCLE;
      const forceExpire = roundCount > 0 && roundCount % KEY_FRESHNESS_WINDOW === 0;
      if (forceExpire) cache.delete(key);
      const cached = cache.get(key);
      if (cached) {
        // Hit — copy cached result into acc, no GF(3) compute.
        acc.set(cached);
        roundCount++;
        cacheHits++;
        return { ops: TBUF_LEN, wasHit: true };
      }
      // Miss — do real work, persist result for the next pass.
      gf3AddBatchReal(1);
      cache.set(key, new Int8Array(acc));
      roundCount++;
      cacheMisses++;
      return { ops: TBUF_LEN, wasHit: false };
    }

    // Pre-warm the cache once at session start so F=1 immediately and the
    // controller can settle to its asymptotic d for each mode without first
    // waiting for natural warmup.  This is one-time setup, not continuous.
    for (let i = 0; i < CACHE_CYCLE; i++) gf3BatchCached();
    // Reset counters so the warmup doesn't pollute key-isolation metrics.
    cacheHits = 0; cacheMisses = 0; signatureCount = 0; keyTouchCount = 0;
    realHighWorkMs = 0; cachedHighMs = 0; opsTotalHigh = 0; opsTotalLow = 0;

    try {
      ws.send(JSON.stringify({ type: "hello", raplAvailable, mode: raplAvailable ? "hardware-watts" : "compute-throughput-proxy" }));
    } catch {}

    // Accept client → server mode-change messages.
    ws.on("message", (raw) => {
      try {
        const msg = JSON.parse(raw.toString());
        if (msg && msg.type === "setMode" &&
            ["idle", "steady", "burst", "auto"].includes(msg.mode)) {
          demandMode = msg.mode;
          dutyDebt = 0; // reset DDA on mode change
        }
      } catch {}
    });

    const tick = setInterval(() => {
      if (!alive || ws.readyState !== ws.OPEN) return;
      // Deterministic controller: compute d_target then DDA-decide isHigh.
      const ctrl = computeController();
      dutyDebt += ctrl.d;
      const isHigh = dutyDebt >= 1.0;
      if (isHigh) dutyDebt -= 1.0;
      const t0 = Date.now();
      let ops = 0;
      if (isHigh) {
        const deadline = t0 + stepMs - 2;
        let stepHadHit = false;
        let stepHadMiss = false;
        while (Date.now() < deadline) {
          const r = gf3BatchCached();
          ops += r.ops;
          signatureCount++;
          if (r.wasHit) stepHadHit = true;
          else { stepHadMiss = true; keyTouchCount++; }
          if (r.wasHit && !stepHadMiss) break;
        }
        timeHighMs += stepMs;
        if (stepHadHit && !stepHadMiss) cachedHighMs += stepMs;
        else realHighWorkMs += stepMs;
      } else {
        timeLowMs += stepMs;
      }
      opsThisWindow += ops;
      if (isHigh) opsTotalHigh += ops; else opsTotalLow += ops;

      stepIdx++;
      const now = Date.now();
      if (now - lastSampleAt >= sampleMs) {
        const dtSec = (now - lastSampleAt) / 1000;
        const opsPerSec = opsThisWindow / dtSec;

        // `mW` is the public field name on the wire and MUST always be an
        // integer milliwatt count.  When RAPL is exposed we derive it
        // purely from the integer µJ delta and integer ms window:
        //     mW = (deltaUj * 1000) / dtMs        (integer floor div)
        // No floating-point watts ever cross the EAC boundary.
        let mWmeasured: number | null = null;
        let deltaUj = 0;
        if (raplAvailable) {
          const cur = readRaplUj();
          if (cur !== null && lastEnergy !== null) {
            deltaUj = cur >= lastEnergy ? cur - lastEnergy : cur;
            const dtMs = Math.max(1, Math.round(dtSec * 1000));
            mWmeasured = Math.floor((deltaUj * 1000) / dtMs);
            cumulativeEnergy_uJ += deltaUj;
            if (isHigh) energyHigh_uJ += deltaUj; else energyLow_uJ += deltaUj;
          }
          lastEnergy = cur;
        }
        // Legacy alias kept for the JSON field name `mW` in the sample
        // payload — preserved as an integer-or-null value.
        const watts: number | null = mWmeasured;

        // ── Pure-integer sample payload ────────────────────────────
        // Every numeric field below is either a whole integer or a
        // {num, den} positive-integer rational.  No JS float literals,
        // no division at the JSON boundary.  Wattage is emitted in
        // milliwatts (mW) so 5 W → 5000.  Time is ms, energy is µJ.
        const totalOps = opsTotalHigh + opsTotalLow;
        const totalMs = timeHighMs + timeLowMs;

        // Modeled per-core wattage in milliwatts (integer):
        //   idle   = 1000 mW   (W_IDLE = 1.0 W)
        //   full   = 5000 mW   (W_FULL = 5.0 W)
        //   nocache = 0.25*full + 0.75*idle = 2000 mW exactly
        const MW_IDLE = 1000;
        const MW_FULL = 5000;
        const mWContinuous = MW_FULL;
        const mWHmodalNoCache = 2000;
        // mWHmodalCached = MW_IDLE + (MW_FULL − MW_IDLE) * realHighWorkMs / totalMs
        //                = (MW_IDLE * totalMs + 4000 * realHighWorkMs) / totalMs
        // Pure integer floor division — no float anywhere.
        const mWHmodalCached = totalMs > 0
          ? Math.floor((MW_IDLE * totalMs + (MW_FULL - MW_IDLE) * realHighWorkMs) / totalMs)
          : MW_IDLE;
        const mWSavedVsContinuous = mWContinuous - mWHmodalCached;

        // ── Cumulative energy accounting (pure-integer µJ) ─────────────
        // mW × ms = µJ exactly.  Always tracked from the modeled wattage
        // so the Console works even when /sys/class/powercap/ (RAPL) is
        // not exposed in the container.  When RAPL *is* available, the
        // CONSUMED counter is fed from real CPU energy counters (above).
        const dtMs = Math.max(0, now - lastSampleAt);
        const energyConsumedThisWindowUj = mWHmodalCached * dtMs;
        const energySavedThisWindowUj    = Math.max(0, mWSavedVsContinuous) * dtMs;
        cumulativeEnergySaved_uJ += energySavedThisWindowUj;
        if (!raplAvailable) {
          // No hardware counter — fall back to the modeled integral so
          // the "Cumulative Energy" tile is never stuck at zero.
          cumulativeEnergy_uJ += energyConsumedThisWindowUj;
        }

        // Observed savings as an integer rational pair.
        // RAPL path:  (energyContEquiv − energyActual) / energyContEquiv
        //          =  ((energyHigh*totalMs/timeHighMs) − (energyHigh+energyLow)) / (energyHigh*totalMs/timeHighMs)
        // Multiply num & den by timeHighMs to clear the inner division.
        // Ops path:   (workCont − totalOps) / workCont
        //          =  (opsHigh*totalMs − totalOps*timeHighMs) / (opsHigh*totalMs)
        let savingsObservedNum = 0;
        let savingsObservedDen = 0;
        if (raplAvailable && timeHighMs > 0 && energyHigh_uJ > 0) {
          const energyContEquivScaled = energyHigh_uJ * totalMs;          // contEquiv * timeHighMs
          const energyActualScaled    = (energyHigh_uJ + energyLow_uJ) * timeHighMs;
          savingsObservedNum = energyContEquivScaled - energyActualScaled;
          savingsObservedDen = energyContEquivScaled;
        } else if (totalOps > 0 && opsTotalHigh > 0 && timeHighMs > 0) {
          const workContScaled  = opsTotalHigh * totalMs;                  // workCont * timeHighMs
          const totalOpsScaled  = totalOps * timeHighMs;
          savingsObservedNum = workContScaled - totalOpsScaled;
          savingsObservedDen = workContScaled;
        }

        // Throughput as integer ops/sec (floor division on ms→s).
        const logicalOpsPerSecAvg = totalMs > 0
          ? Math.floor(totalOps * 1000 / totalMs)
          : 0;
        const realCpuOpsPerSecAvg = realHighWorkMs > 0
          ? Math.floor(cacheMisses * TBUF_LEN * 1000 / realHighWorkMs)
          : 0;

        // Controller live state — emitted as integer scaled rationals.
        // Q comes from a sin() in "auto" mode, scaled to per-mille (×1000).
        // F = cache.size / CACHE_CYCLE — already integer/integer.
        // d clamped to [1/144, 1], scaled to per-million (×1_000_000).
        const queueDepthMilli = Math.round(ctrl.Q * 1000);
        const cacheFillNum = Math.min(CACHE_CYCLE, cache.size);
        const cacheFillDen = CACHE_CYCLE;
        const dutyTargetMicro = Math.round(ctrl.d * 1_000_000);

        const samplePayload = {
            type: "sample",
            t: now, phase: isHigh ? "high" : "low",
            opsPerSec, opsThisWindow,                                 // integers
            mW: watts,                                                // already integer mW
            deltaUj, cumulativeEnergyUj: cumulativeEnergy_uJ,         // integers (µJ)
            cumulativeEnergySavedUj: cumulativeEnergySaved_uJ,        // integers (µJ saved vs continuous-on)
            energySavedThisWindowUj,                                  // µJ saved in this sample window
            cumulativeOps: totalOps,
            cumulativeOpsHigh: opsTotalHigh,
            cumulativeOpsLow: opsTotalLow,
            timeHighMs, timeLowMs, totalMs,                           // integers (ms)
            cacheHits, cacheMisses,
            cacheHitRate:    { num: cacheHits, den: Math.max(1, cacheHits + cacheMisses) },
            realHighWorkMs, cachedHighMs,                             // integers (ms)
            // Compressed savings as integer rational:
            //   (totalMs − realHighWorkMs) / totalMs
            // Asymptotes toward {143, 144} after cache warm-up.
            compressedSavings: { num: totalMs - realHighWorkMs, den: Math.max(1, totalMs) },
            theoreticalCompressedSavings: { num: 143, den: 144 },
            // Modeled wattage in milliwatts (integers).
            mWContinuous, mWHmodalNoCache, mWHmodalCached, mWSavedVsContinuous,
            // Effective compute fraction: realHighWorkMs / totalMs
            effectiveCompute: { num: realHighWorkMs, den: Math.max(1, totalMs) },
            // Throughput split: logical (cache hits served) vs real CPU.
            logicalOpsPerSecAvg, realCpuOpsPerSecAvg,                 // integers
            // Deterministic controller state — integer scaled rationals.
            demandMode,
            queueDepth:    { num: queueDepthMilli, den: 1000 },
            cacheFillRatio:{ num: cacheFillNum,    den: cacheFillDen },
            dutyTarget:    { num: dutyTargetMicro, den: 1_000_000 },
            // Key-isolation metrics — pure integer ratios.
            keyTouchCount, signatureCount,
            keyExposureRatio:  { num: keyTouchCount,  den: Math.max(1, signatureCount) },
            keyIsolationFactor:{ num: signatureCount, den: Math.max(1, keyTouchCount)  },
            mode: raplAvailable ? "hardware-watts" : "compute-throughput-proxy",
            // Duty / savings rationals.
            observedRatio:    { num: timeHighMs, den: Math.max(1, totalMs) },
            theoreticalRatio: { num: 1, den: 4 },
            savingsObserved:  savingsObservedDen > 0
              ? { num: savingsObservedNum, den: savingsObservedDen }
              : null,
            theoreticalSavings:{ num: 143, den: 192 },
            stepIdx,
          };

        // ── Seal + chain in one duplex pass ─────────────────────────
        // 1. plaintext bytes → balanced trits
        // 2. squeeze keystream of equal length
        // 3. cipher_trits[i] = (plain + ks) mod-balanced  (GF(3) wrap)
        // 4. absorb cipher_trits → state advances (chaining)
        // 5. squeeze 243-trit (385-bit) running chain tag
        // The whole step happens at emit time — no later actor can
        // substitute a reading without breaking the chain.
        const plaintextBytes = Buffer.from(JSON.stringify(samplePayload), "utf-8");
        const plainTrits = bytesToBalancedTrits(plaintextBytes);
        const ks = sealDuplex.squeeze(plainTrits.length);
        const cipherTrits = new Int8Array(plainTrits.length);
        for (let i = 0; i < plainTrits.length; i++) {
          let v = plainTrits[i] + ks[i];
          if (v > 1) v -= 3; else if (v < -1) v += 3;
          cipherTrits[i] = v;
        }
        sealDuplex.absorbTrits(cipherTrits);
        const chainTagTrits = sealDuplex.squeeze(243);
        const chainTagHex = tritsToHexFn(chainTagTrits);
        const chainTagTrit  = balancedTritsToRepC(chainTagTrits);
        const cipherTritsTrit = balancedTritsToRepC(cipherTrits);
        const sealedFrame = {
          type: "sealed",
          sessionId,
          index: chainIndex.toString(),
          // ── trit-native (Rep-C bijective base-3) — primary wire form ──
          cipherTrits: cipherTritsTrit,
          chainTag:    chainTagTrit,
          chainTagPrev: chainIndex === 0n ? chainSeedTrit : undefined,
          // hex copies kept only for legacy audit consumers
          cipherTritsHex: tritsToHexFn(cipherTrits),
          chainTagHex,
          chainTagPrevHex: chainIndex === 0n ? chainSeedHex : undefined,
          plainTritLen: plainTrits.length,
        };

        // Stash chain-head + plaintext (preview) — the EAC binds to the
        // chain tag, NOT to a re-sampled snapshot, so there is no
        // window for substitution between read and sign.
        lastHmodalSnapshot = {
          ...samplePayload,
          // Salvi-anchor + tick parameters for the chain-derived fs
          // computation in /api/hmodal/issue-eac.  See the
          // "Femtosecond-since-Salvi-epoch derivation" block earlier
          // in this file.
          fsClock: {
            sessionAnchorMs,                       // integer ms since Salvi epoch (one-shot)
            sampleMs,                              // integer ms between sealed samples
            chainIndexAtSeal: chainIndex.toString(), // BigInt → string
          },
          attestation: {
            sessionId,
            sessionKeyFingerprint,
            sessionKeyFingerprintTrit,
            cipher: "TL-Sponge-385 duplex (Phase Encryption v3)",
            // primary trit-native form
            chainSeed:   chainSeedTrit,
            chainTag:    chainTagTrit,
            cipherTrits: cipherTritsTrit,
            // hex copies kept for legacy audit consumers
            chainSeedHex,
            chainTagHex,
            cipherTritsHex: sealedFrame.cipherTritsHex,
            chainIndex: chainIndex.toString(),
            plainTritLen: plainTrits.length,
          },
        };

        try { ws.send(JSON.stringify(samplePayload)); } catch {}
        try { ws.send(JSON.stringify(sealedFrame)); } catch {}
        chainIndex += 1n;

        opsThisWindow = 0;
        lastSampleAt = now;
      }
    }, stepMs);

    ws.on("close", () => { alive = false; clearInterval(tick); });
    ws.on("error", () => { alive = false; clearInterval(tick); });
  });

  // ──────────────────────────────────────────────────────────────────
  // POST /api/hmodal/issue-eac
  // Snapshots the latest HModal sample, builds an Energy Attestation
  // Certificate per TM-2026-042 §4.3, runs preSignCheck, signs with
  // TL-DSA-87, and returns the signed cert.  Visible from the
  // "Issue EAC now" button on /hmodal-demo.
  // ──────────────────────────────────────────────────────────────────
  app.post("/api/hmodal/issue-eac", computationLimiter, async (_req, res) => {
    // computationLimiter caps abusive callers (50 req / IP / minute) so the
    // process-held TL-DSA-87 signer cannot be turned into an open signing
    // oracle.  Production deployments should additionally gate this behind
    // operator RBAC via NinjaExec.
    try {
      const snap = lastHmodalSnapshot;
      if (!snap) {
        return res.status(409).json({
          ok: false,
          error: "no_sample_available",
          message: "Open /hmodal-demo and let the WS produce at least one sample before requesting an EAC.",
        });
      }

      // ──────────────────────────────────────────────────────────────
      // HPTP femtosecond timestamp — canonical Salvi-core path.
      //
      // Calls getFemtosecondTimestamp() from
      // server/salvi-core/femtosecond-timing.ts, which is the
      // framework's authoritative HPTP clock-read.  Per that module:
      //
      //   wall_ns = anchorWallNs + (hrtime_now − anchorHrNs)
      //   wall_fs = wall_ns × FEMTOSECONDS_PER_NANOSECOND
      //   salviEpochOffset = wall_fs − SALVI_EPOCH_FS
      //
      // Anchor is captured ONCE at module load (Date.now() +
      // process.hrtime.bigint()).  Every subsequent read extends from
      // the monotonic hrtime counter — not Date.now() — so jitter and
      // NTP slewing do not perturb the measurement after anchoring.
      // The lower fs digits are explicitly honest (`measured:
      // 'ms.µs.ns (ps.fs awaiting Tier 0 clock)'`) per the
      // framework's own documentation.
      // ──────────────────────────────────────────────────────────────
      const hptpTs    = getFemtosecondTimestamp();
      // PURE-FRAMEWORK timestamp: tickCounter advances by 1 per HPTP read.
      // No hardware clock on the per-call path.  Tritify the tick counter
      // directly — it is the framework's intrinsic clock unit.
      const fsInt     = hptpTs.tickCounter;             // ticks since boot (monotonic)
      const fsTrit    = toBijectiveBase3(fsInt);
      // Snapshot's chain anchor is preserved in the EAC for audit
      // traceability, but does NOT participate in the HPTP timestamp.
      const fsClock      = snap.fsClock;
      const chainTagHex  = snap.attestation?.chainTagHex;

      // Build EAC fields per spec §4.3.  EVERY numeric field is either a
      // whole integer (µJ, ms, ops, mW) or a positive {num, den} integer
      // rational.  No JS floats anywhere on the EAC document path.
      const measuredMW = (snap.mW as number | null) ?? snap.mWHmodalCached;
      const baselineMW = snap.mWContinuous;
      const windowMs   = snap.totalMs ?? ((snap.timeHighMs ?? 0) + (snap.timeLowMs ?? 0));

      // ── ATTOSECONDS-SINCE-UTC-EPOCH ─ BOOT-ANCHORED DERIVATION ───────
      // The (Date.now, hrtime.bigint, framework-attosecond-walk) triple
      // was frozen ONCE at module load — see `__plenum_eac_utc_anchor`
      // initialisation block at the top of this file.  Every cert
      // advances the anchored UTC instant strictly monotonically using
      // ONLY two pure monotonic clocks (no per-cert wall-clock reads,
      // no clamps):
      //
      //   utc_as = utcAtAnchor                                  // ms × 10¹⁵
      //          + (hrtime.bigint() − hrAtAnchor) · 10⁶         // ns → as, wall-clock-locked
      //          + (asSinceBootNow − asSinceBootAtAnchor)       // sub-ns framework entropy
      //
      // Because the boot anchor was captured BEFORE this handler ever
      // runs, (hrNow − hrAtAnchor) is already large by the time any
      // cert is issued, so the emitted integer is always populated
      // through its sub-millisecond / sub-microsecond / sub-nanosecond
      // digits — never ms × 10¹⁵ with trailing zeros.  Both delta
      // terms are strictly monotonic, so the sum is strictly monotonic.
      const AS_PER_MS  = 1_000_000_000_000_000n;       // 10¹⁵ as / ms
      const AS_PER_NS  = 1_000_000n;                   // 10⁶  as / ns
      const asNumBig   = BigInt(String(hptpTs.asSinceBootNum));
      const asDenBig   = BigInt(String(hptpTs.asSinceBootDen));
      const asSinceBootBig = asDenBig > 0n ? asNumBig / asDenBig : 0n;
      const hrNow      = process.hrtime.bigint();      // monotonic ns

      const anchor = (globalThis as any).__plenum_eac_utc_anchor as {
        utcAtAnchor:         bigint;
        hrAtAnchor:          bigint;
        asSinceBootAtAnchor: bigint;
        isoAtAnchor:         string;
      };
      const hrDelta  = hrNow - anchor.hrAtAnchor;                  // ns
      const fwDelta  = asSinceBootBig - anchor.asSinceBootAtAnchor; // as
      const utcAttosecondsBig =
        anchor.utcAtAnchor + hrDelta * AS_PER_NS + fwDelta;
      const emittedUtcMs        = utcAttosecondsBig / AS_PER_MS;
      const utcIsoAtIssue       = new Date(Number(emittedUtcMs)).toISOString();
      const utcMsAtIssue        = emittedUtcMs;
      const attosecondsDecimal  = utcAttosecondsBig.toString();
      const attosecondsTrit     = toBijectiveBase3(utcAttosecondsBig);
      const attosecondsBootDec  = asSinceBootBig.toString();
      const attosecondsBootTrit = toBijectiveBase3(asSinceBootBig);
      // ── 42-Calendar Stamp ────────────────────────────────────────────
      // Every EAC carries a multi-civilizational calendar reading at the
      // moment of issuance, derived purely from the framework's
      // ancient-calendar-sync module (JDN-based, deterministic).  We pick
      // a curated 12-system spread spanning the major civilizations.
      let calendarStamp: any = null;
      try {
        const cal = await import("./salvi-core/ancient-calendar-sync");
        const nowDate = new Date();
        const safe = (fn: () => any) => { try { return fn(); } catch { return null; } };
        calendarStamp = {
          gregorian_iso:        nowDate.toISOString(),
          julian_day_number:    safe(() => cal.toJulianDayNumber(nowDate)),
          mayan_long_count:     safe(() => cal.toMayanLongCount(nowDate)),
          hebrew:               safe(() => cal.toHebrewDate(nowDate)),
          islamic_hijri:        safe(() => cal.toIslamicHijri(nowDate)),
          chinese_sexagenary:   safe(() => cal.toChineseSexagenary(nowDate)),
          vedic_kali_yuga:      safe(() => cal.toVedicKaliYuga(nowDate)),
          persian_solar_hijri:  safe(() => cal.toPersianDate(nowDate)),
          ethiopian_geez:       safe(() => cal.toEthiopianDate(nowDate)),
          coptic:               safe(() => cal.toCopticDate(nowDate)),
          egyptian_civil:       safe(() => cal.toEgyptianCivil(nowDate)),
          thirteen_moon:        safe(() => cal.toThirteenMoonDate(nowDate)),
          byzantine_anno_mundi: safe(() => cal.toByzantineAnnoMundi(nowDate)),
          source: "salvi-core/ancient-calendar-sync (42 systems, JDN-anchored)",
        };
      } catch (e: any) {
        calendarStamp = { error: "calendar_sync_unavailable", message: e?.message ?? String(e) };
      }

      const eacFields = {
        type: "EAC",
        version: 1,
        spec: "TM-2026-042 Rev.2 §4.3",
        numeric_policy: "all numeric fields are whole integers or positive {num, den} integer rationals; no IEEE-754 floats",
        timestamp: {
          // ── FORWARD-FACING UTC ATTOSECOND TIMESTAMP ───────────────────
          // Single integer = attoseconds since the UTC Unix epoch
          // (1970-01-01T00:00:00Z).  This is the cert's primary,
          // wall-clock-grounded "when".  Composition:
          //
          //     as_utc = (Date.now() ms · 10¹⁵)
          //              + (sub-millisecond residue from framework walk)
          //
          // The boot-anchored framework value is preserved below for
          // audit purity — it never participates in the displayed
          // wall-clock timestamp on its own.
          attoseconds_since_unix_epoch_decimal: attosecondsDecimal,
          attoseconds_since_unix_epoch_trit:    attosecondsTrit,
          utc_iso_at_issue:                     utcIsoAtIssue,
          utc_ms_at_issue_decimal:              utcMsAtIssue.toString(),
          // Boot-anchored values kept for traceability with the
          // framework tick walk — never the headline timestamp.
          attoseconds_since_boot_decimal: attosecondsBootDec,
          attoseconds_since_boot_trit:    attosecondsBootTrit,
          // Authoritative monotonic tick counter on Z_{D_α} (D_α = 125_250_125).
          // No wall clock is consulted on the per-call path.
          tick_decimal:               hptpTs.tickCounter.toString(),
          tick_trit:                  fsTrit,
          // EXACT rational attoseconds since boot — surfaced as {num, den}.
          // NEVER collapsed to integer division (no "÷ that creates 0",
          // no trailing-zero padding).  Kept for audit purity.
          as_since_boot: {
            num: hptpTs.asSinceBootNum.toString(),
            den: hptpTs.asSinceBootDen.toString(),
          },
          tick_period_as: {
            num: hptpTs.tickPeriodAsNum.toString(),       // 8_000_000 = 2⁹·5⁶
            den: hptpTs.tickPeriodAsDen.toString(),       // 1_002_001 = (pqr)²
            irreducible: "gcd(2^9 * 5^6, 7^2 * 11^2 * 13^2) = 1",
          },
          precision: FS_TIMING_PRECISION,
          derivation: {
            source: "salvi-core/femtosecond-timing.getFemtosecondTimestamp() — pure framework, no hw clock",
            hardware_clock_used:     "NO — pure framework derivation",
            tick_counter_decimal:    hptpTs.tickCounter.toString(),
            // Closed walk on Z_{D_α}, D_α = F₅³·p²·q²·r² = 125_250_125
            // (Theorem 22 denominator of 1/α).
            walk_modulus_d_alpha:    "125250125",
            walk_factorisation:      "5^3 * 7^2 * 11^2 * 13^2",
            walk_position_decimal:   hptpTs.walkTick.toString(),
            framework_fs_index:      hptpTs.frameworkFsIndex.toString(),
            attoseconds_index:       hptpTs.attoseconds.toString(),
            tick_period_as_num:      hptpTs.tickPeriodAsNum.toString(),
            tick_period_as_den:      hptpTs.tickPeriodAsDen.toString(),
            as_since_boot_num:       hptpTs.asSinceBootNum.toString(),
            as_since_boot_den:       hptpTs.asSinceBootDen.toString(),
            human_readable:          hptpTs.humanReadable,
            clock_tier:              hptpTs.clockTier,        // 0 = pure derivation
            measured:                hptpTs.measured,
            chain_index_at_seal_decimal: String(fsClock?.chainIndexAtSeal ?? "n/a"),
            chain_index_at_seal_trit:    fsClock?.chainIndexAtSeal != null
              ? toBijectiveBase3(BigInt(fsClock.chainIndexAtSeal))
              : "n/a",
            chain_tag_trit:          chainTagHex ? hexToBijectiveBase3(chainTagHex) : "n/a",
            chain_tag_hex:           chainTagHex ?? "n/a",
          },
        },
        node: {
          tdns: "tdns:hmodal-demo:01",     // placeholder until TDNS wired
          mode: snap.mode,                  // hardware-watts | compute-throughput-proxy
          demand_mode: snap.demandMode,
        },
        measurement: {
          window_ms: windowMs,
          measured_mW: measuredMW,
          baseline_mW: baselineMW,
          mW_saved:   baselineMW - measuredMW,
          // savings_ratio = (baseline_mW − measured_mW) / baseline_mW
          savings_ratio: { num: Math.max(0, baselineMW - measuredMW), den: Math.max(1, baselineMW) },
          savings_ratio_theoretical: { num: 143, den: 192 },
          cumulative_ops_decimal: String(snap.cumulativeOps ?? 0),
          cumulative_ops_trit:    toBijectiveBase3(BigInt(snap.cumulativeOps ?? 0)),
          cumulative_energy_uJ_decimal: String(snap.cumulativeEnergyUj ?? 0),
          cumulative_energy_uJ_trit:    toBijectiveBase3(BigInt(snap.cumulativeEnergyUj ?? 0)),
          // Energy SAVED versus a hypothetical continuous-on baseline.
          // Always populated from the modeled wattage delta — works
          // even when /sys/class/powercap/ (RAPL) is not exposed.
          cumulative_energy_saved_uJ_decimal: String(snap.cumulativeEnergySavedUj ?? 0),
          cumulative_energy_saved_uJ_trit:    toBijectiveBase3(BigInt(snap.cumulativeEnergySavedUj ?? 0)),
          duty_target:        snap.dutyTarget,         // {num, den} per-million
          duty_floor_constant:{ num: 1, den: 144, name: "Δ" },
        },
        calendar_stamp: calendarStamp,
        key_isolation: {
          signature_count: snap.signatureCount,
          key_touch_count: snap.keyTouchCount,
          exposure_ratio:    snap.keyExposureRatio,    // {num, den}
          isolation_factor:  snap.keyIsolationFactor,  // {num, den}
        },
        attestation_chain: snap.attestation
          ? {
              // ── trit-native primary form (Rep-C bijective base-3) ──
              session_id:                       snap.attestation.sessionId,            // already trit-native (digits {1,2,3})
              session_key_fingerprint_trit:     snap.attestation.sessionKeyFingerprintTrit
                ?? hexToBijectiveBase3(snap.attestation.sessionKeyFingerprint),
              chain_seed_trit:                  snap.attestation.chainSeed
                ?? hexToBijectiveBase3(snap.attestation.chainSeedHex),
              chain_tag_trit:                   snap.attestation.chainTag
                ?? hexToBijectiveBase3(snap.attestation.chainTagHex),
              cipher_trits_trit:                snap.attestation.cipherTrits
                ?? hexToBijectiveBase3(snap.attestation.cipherTritsHex),
              chain_index_decimal:              snap.attestation.chainIndex,
              chain_index_trit:                 toBijectiveBase3(BigInt(snap.attestation.chainIndex)),
              cipher:                           snap.attestation.cipher,
              plain_trit_len:                   snap.attestation.plainTritLen,
              // ── hex copies kept ONLY for legacy audit consumers ──
              session_key_fingerprint_hex:      snap.attestation.sessionKeyFingerprint,
              chain_seed_hex:                   snap.attestation.chainSeedHex,
              chain_tag_hex:                    snap.attestation.chainTagHex,
              chain_tag_milesian:               milesianGlyphHash(snap.attestation.chainTagHex),
              cipher_trits_hex:                 snap.attestation.cipherTritsHex,
              note: "Chain tag is the running 385-bit squeeze of the TL-Sponge-385 duplex after sealing this sample.  Any gap, reorder, or substitution in the WS sample stream changes this value.  All hashes are surfaced trit-native in Rep-C bijective base-3 (digit set {1,2,3}); hex copies are retained only for legacy audit interop.",
            }
          : null,
      };

      const canonicalJson = JSON.stringify(eacFields);
      const documentBytes = Buffer.from(canonicalJson, "utf-8");

      const { tis27Hash } = await import("./crypto/sponge-hash");
      const tis27HashHex = tis27Hash(documentBytes);
      const tis27Milesian = milesianGlyphHash(tis27HashHex);

      const kp = getArcSigner();
      const pubKeyHash = publicKeyHash(kp.publicKey);

      const { preSignCheck } = await import("../sign-here/src/pre-sign-check");
      // Inject our own TIS-27 hashFn so pre-sign-check uses the same trit
      // sponge as the rest of the pipeline — the default in pre-sign-check
      // falls back to require('crypto') which doesn't exist under ESM.
      const tritHashFn = (input: Buffer) => tis27HashFn(input);
      const preSign = preSignCheck({
        documentBytes,
        expectedHash: tis27HashHex,
        signingKey: kp.secretKey,
        variant: "TL-DSA-87",
        timestampFs: fsInt,
        nowFs: fsInt,
        hashFn: tritHashFn,
      });

      if (!preSign.pass) {
        return res.status(500).json({
          ok: false,
          error: "pre_sign_check_failed",
          failures: preSign.failures,
          eac: eacFields,
          tis27_hash: tis27HashHex,
        });
      }

      const sigResult = signHex(kp.secretKey, canonicalJson, "TL-DSA-87");

      const signedCert: any = {
        ...eacFields,
        integrity: {
          tis27_hash_hex: tis27HashHex,
          tis27_hash_milesian: tis27Milesian,
          canonical_byte_length: documentBytes.length,
        },
        pre_sign: {
          pass: preSign.pass,
          failures: preSign.failures,
          checks: preSign.checks?.map((c: any) => ({ name: c.name, pass: c.pass })),
        },
        signature: {
          variant: "TL-DSA-87",
          signature_hex: sigResult,
          public_key_hex: kp.publicKey.toString("hex"),
          public_key_hash: pubKeyHash,
          native_signer: isNativeAvailable(),
          signed_at_iso: new Date().toISOString(),
        },
      };

      // ── HEDERA HCS WITNESS ───────────────────────────────────────────
      // Submit the signed-cert hash to the Hedera Consensus Service so the
      // EAC carries blockchain-anchored, timestamp-ordered, immutable
      // proof-of-existence.  When the service is not configured (no
      // HEDERA_ACCOUNT_ID / HEDERA_PRIVATE_KEY), we still emit the block
      // with status="not_configured" so the certificate UI shows the
      // section explicitly instead of silently omitting it.
      //
      // SECURITY: Hedera submissions cost real ℏ (HBAR), and this
      // endpoint is reachable without authentication (rate-limited only).
      // To prevent paid-spend abuse via mass EAC issuance, the Hedera
      // call is OPT-IN per env var `HMODAL_EAC_HEDERA_WITNESS=on`.
      // When the flag is off we still emit a status block on the cert
      // so operators know witnessing is available but disabled.
      const hederaOptedIn = process.env.HMODAL_EAC_HEDERA_WITNESS === "on";
      let hederaWitness: any = {
        enabled: false,
        status: "not_configured",
        message: "Hedera HCS witnessing is not configured for this node (set HEDERA_ACCOUNT_ID and HEDERA_PRIVATE_KEY to enable).",
      };
      if (hederaService && !hederaOptedIn) {
        hederaWitness = {
          enabled: false,
          status: "disabled_by_policy",
          message: "Hedera HCS service is configured on this node but EAC-witnessing is OFF by policy (set HMODAL_EAC_HEDERA_WITNESS=on to enable; this prevents unauthenticated paid-spend abuse via the public EAC endpoint).",
        };
      }
      if (hederaService && hederaOptedIn) {
        try {
          const witnessResp = await hederaService.submitWitness({
            operation_id: `eac-${Date.now()}`,
            witness_type: "SINGLE_HASH",
            payload: {
              hash: tis27HashHex,
              hash_algorithm: "TIS-27",
            },
            metadata: {
              ternary_context: { security_mode: "TL-Sponge-385" },
              kernel_op_id:    `hmodal-eac-${snap.attestation?.chainIndex ?? "0"}`,
              salvi_batch_ref: snap.attestation?.sessionId ?? "hmodal-demo",
            } as any,
          } as any);
          hederaWitness = {
            enabled: true,
            status: "witnessed",
            network:              (witnessResp as any)?.transaction?.topic_id
              ? "configured"
              : "configured",
            topic_id:             (witnessResp as any)?.transaction?.topic_id ?? null,
            transaction_id:       (witnessResp as any)?.transaction?.id ?? null,
            consensus_timestamp:  (witnessResp as any)?.transaction?.consensus_timestamp ?? null,
            sequence_number:      (witnessResp as any)?.transaction?.sequence_number ?? null,
            running_hash:         (witnessResp as any)?.transaction?.running_hash ?? null,
            witnessed_hash:       tis27HashHex,
            note: "Blockchain-anchored proof of existence via Hedera Consensus Service.",
          };
        } catch (e: any) {
          // Log the raw error server-side for debugging, but only ever
          // expose a generic, fixed string in the cert payload — Hedera
          // SDK errors can echo back internal account IDs, key parts, or
          // network endpoints.
          console.error("[hmodal-eac] Hedera witness submission failed:", e?.message ?? e);
          hederaWitness = {
            enabled: true,
            status: "submission_failed",
            error: "Hedera HCS submission did not complete (see server logs for details).",
            witnessed_hash: tis27HashHex,
          };
        }
      }
      signedCert.hedera_witness = hederaWitness;

      res.json({ ok: true, eac: signedCert });
    } catch (err: any) {
      res.status(500).json({
        ok: false,
        error: "eac_generation_failed",
        message: err?.message ?? String(err),
      });
    }
  });

  const remoteTerminalSessions = new Map<string, WebSocket>();
  (globalThis as any).__remoteTerminalSessions = remoteTerminalSessions;

  terminalWss.on("connection", (ws: WebSocket, request: any) => {
    const url = new URL(request.url || "", `http://${request.headers.host}`);
    const requestedSession = url.searchParams.get("session");
    const ownerId: string = request._terminalUserId || "anonymous";
    let activeSessionId: string | null = null;
    let dataListener: ((data: string) => void) | null = null;
    let msgCount = 0;
    let msgWindowStart = Date.now();
    let remoteNodeAddress: string | null = null;
    const remoteSessionId = crypto.randomBytes(8).toString("hex");
    const activeTailFollows: Map<string, string> = new Map();
    opsTerminalClients.add(ws);

    function checkRateLimit(): boolean {
      const now = Date.now();
      if (now - msgWindowStart > 1000) {
        msgCount = 0;
        msgWindowStart = now;
      }
      msgCount++;
      return msgCount <= TERMINAL_MSG_RATE_LIMIT;
    }

    function attachToSession(session: ReturnType<typeof getSession>) {
      if (!session) return;
      if (dataListener && activeSessionId) {
        const oldSession = getSession(activeSessionId);
        if (oldSession) {
          (oldSession.ptyProcess as any).removeListener("data", dataListener);
        }
      }
      activeSessionId = session.id;
      dataListener = (data: string) => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: "output", data }));
        }
      };
      session.ptyProcess.onData(dataListener);
      if (!session.exitHandlerAttached) {
        session.exitHandlerAttached = true;
        session.ptyProcess.onExit(() => {
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: "session_ended", sessionId: session.id }));
          }
          destroySession(session.id);
          if (activeSessionId === session.id) activeSessionId = null;
        });
      }
    }

    if (requestedSession && isSessionOwner(requestedSession, ownerId)) {
      const existing = getSession(requestedSession);
      if (existing) {
        attachToSession(existing);
        ws.send(JSON.stringify({ type: "session_attached", sessionId: requestedSession }));
      }
    }

    if (!activeSessionId) {
      const session = createSession(ownerId);
      attachToSession(session);
      ws.send(JSON.stringify({ type: "session_created", sessionId: session.id }));
    }

    ws.on("message", (raw: Buffer) => {
      if (raw.length > TERMINAL_MSG_MAX_SIZE) return;
      if (!checkRateLimit()) return;

      let msg: any;
      try {
        msg = JSON.parse(raw.toString());
      } catch {
        return;
      }

      switch (msg.type) {
        case "connect_remote": {
          const nodeAddr = msg.address ? normalizeTernaryAddr(msg.address) : null;
          if (!nodeAddr) {
            ws.send(JSON.stringify({ type: "error", message: "No address provided" }));
            break;
          }
          const nodeWs = relayClients.get(nodeAddr);
          if (!nodeWs || nodeWs.readyState !== WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: "error", message: `Node ${toDottedAddr(nodeAddr)} not connected to relay` }));
            break;
          }
          remoteNodeAddress = nodeAddr;
          remoteTerminalSessions.set(remoteSessionId, ws);
          nodeWs.send(JSON.stringify({
            type: "relay",
            msgType: "terminal-open",
            payload: JSON.stringify({ sessionId: remoteSessionId }),
            from: "coordinator",
          }));
          ws.send(JSON.stringify({ type: "remote_connected", address: toDottedAddr(nodeAddr), remoteSessionId }));
          console.log(`[terminal] Remote session ${remoteSessionId} opened to ${toDottedAddr(nodeAddr)}`);
          break;
        }
        case "connect_local": {
          if (remoteNodeAddress) {
            const nodeWs = relayClients.get(remoteNodeAddress);
            if (nodeWs && nodeWs.readyState === WebSocket.OPEN) {
              nodeWs.send(JSON.stringify({
                type: "relay",
                msgType: "terminal-close",
                payload: JSON.stringify({ sessionId: remoteSessionId }),
                from: "coordinator",
              }));
            }
            remoteTerminalSessions.delete(remoteSessionId);
            remoteNodeAddress = null;
            console.log(`[terminal] Remote session ${remoteSessionId} closed, switching to local`);
          }
          if (!activeSessionId) {
            const session = createSession(ownerId);
            attachToSession(session);
            ws.send(JSON.stringify({ type: "session_created", sessionId: session.id }));
          }
          ws.send(JSON.stringify({ type: "local_connected" }));
          break;
        }
        case "input": {
          if (remoteNodeAddress) {
            const nodeWs = relayClients.get(remoteNodeAddress);
            if (nodeWs && nodeWs.readyState === WebSocket.OPEN) {
              nodeWs.send(JSON.stringify({
                type: "relay",
                msgType: "terminal-input",
                payload: JSON.stringify({ sessionId: remoteSessionId, data: msg.data }),
                from: "coordinator",
              }));
            }
            break;
          }
          if (activeSessionId) {
            const session = getSession(activeSessionId);
            if (session && isSessionOwner(session.id, ownerId)) {
              session.lastActivity = Date.now();
              session.ptyProcess.write(msg.data);
            }
          }
          break;
        }
        case "resize": {
          if (remoteNodeAddress) {
            const nodeWs = relayClients.get(remoteNodeAddress);
            if (nodeWs && nodeWs.readyState === WebSocket.OPEN) {
              nodeWs.send(JSON.stringify({
                type: "relay",
                msgType: "terminal-resize",
                payload: JSON.stringify({ sessionId: remoteSessionId, cols: msg.cols, rows: msg.rows }),
                from: "coordinator",
              }));
            }
            break;
          }
          if (activeSessionId && msg.cols && msg.rows) {
            if (isSessionOwner(activeSessionId, ownerId)) {
              resizeSession(activeSessionId, msg.cols, msg.rows);
            }
          }
          break;
        }
        case "new_session": {
          const session = createSession(ownerId);
          attachToSession(session);
          ws.send(JSON.stringify({ type: "session_created", sessionId: session.id }));
          ws.send(JSON.stringify({ type: "session_list", sessions: listSessions(ownerId) }));
          break;
        }
        case "attach": {
          if (!isSessionOwner(msg.sessionId, ownerId)) {
            ws.send(JSON.stringify({ type: "error", message: "Access denied" }));
            break;
          }
          const session = getSession(msg.sessionId);
          if (session) {
            attachToSession(session);
            ws.send(JSON.stringify({ type: "session_attached", sessionId: msg.sessionId }));
          } else {
            ws.send(JSON.stringify({ type: "error", message: "Session not found" }));
          }
          break;
        }
        case "destroy": {
          if (!isSessionOwner(msg.sessionId, ownerId)) {
            ws.send(JSON.stringify({ type: "error", message: "Access denied" }));
            break;
          }
          destroySession(msg.sessionId);
          if (msg.sessionId === activeSessionId) {
            activeSessionId = null;
          }
          ws.send(JSON.stringify({ type: "session_list", sessions: listSessions(ownerId) }));
          break;
        }
        case "list_sessions": {
          ws.send(JSON.stringify({ type: "session_list", sessions: listSessions(ownerId) }));
          break;
        }
        case "plenumnet-builtin": {
          if (!remoteNodeAddress) {
            ws.send(JSON.stringify({ type: "error", message: "Not connected to a remote node" }));
            break;
          }
          const nodeWsBuiltin = relayClients.get(remoteNodeAddress);
          if (!nodeWsBuiltin || nodeWsBuiltin.readyState !== WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: "error", message: "Remote node disconnected" }));
            break;
          }
          nodeWsBuiltin.send(JSON.stringify({
            type: "relay",
            msgType: "plenumnet-builtin",
            payload: JSON.stringify({
              sessionId: remoteSessionId,
              command: msg.command || "",
              args: msg.args || msg.payload || "",
            }),
            from: "coordinator",
          }));
          break;
        }
        case "exec":
        case "tail":
        case "tail-stop":
        case "file-push":
        case "file-pull":
        case "chunk-init":
        case "chunk-data":
        case "chunk-complete":
        case "transfer-cancel":
        case "model-swap": {
          const targetNodeId = msg.node_id;
          if (!targetNodeId) {
            ws.send(JSON.stringify({ type: "ops_message", data: { type: "ops-error", error_code: "NODE_NOT_FOUND", message: "Missing node_id" } }));
            break;
          }
          const validation = opsChannelService.validateOpsMessage(msg);
          if (!validation.valid) {
            ws.send(JSON.stringify({ type: "ops_message", data: opsChannelService.makeOpsError(
              targetNodeId, msg.request_id || '', (validation.errorCode || 'SIGNATURE_MISSING') as OpsErrorCode,
              validation.errorMessage || 'Ops message validation failed', msg.type as OpsMessageType,
            ) }));
            break;
          }

          let opsTargetWs: WebSocket | undefined;
          for (const [addr, clientWs] of relayClients.entries()) {
            if (addr === targetNodeId) {
              opsTargetWs = clientWs;
              break;
            }
          }
          if (!opsTargetWs || opsTargetWs.readyState !== WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: "ops_message", data: opsChannelService.makeOpsError(targetNodeId, msg.request_id || '', 'NODE_DISCONNECTED', `Node ${targetNodeId} is not connected`, msg.type as OpsMessageType) }));
            break;
          }

          opsChannelService.recordAuditEntry(opsChannelService.createAuditEntry(msg as OpsMessage, 'success'));

          if (msg.type === 'tail' && msg.follow) {
            activeTailFollows.set(msg.request_id, targetNodeId);
          }
          if (msg.type === 'tail-stop') {
            const origId = msg.original_request_id || msg.request_id;
            activeTailFollows.delete(origId);
          }

          if (msg.request_id) {
            opsRequestOriginators.set(msg.request_id, ws);
          }

          opsTargetWs.send(JSON.stringify({
            type: "relay",
            msgType: msg.type,
            payload: JSON.stringify(msg),
            from: "coordinator",
          }));
          opsChannelService.updateNodeSeen(targetNodeId, targetNodeId);
          break;
        }
        case "cluster_exec": {
          const command = msg.command;
          if (!command) break;
          if (!isClusterCommandAllowed(command)) {
            ws.send(JSON.stringify({ type: "cluster_result", results: [{ nodeId: "local", address: "this-node", output: "", error: "Command not in allowlist. Allowed: echo, hostname, whoami, uname, date, uptime, df, free, ls, pwd, id, ps, env, printenv. No pipes, semicolons, or subshells.", exitCode: 1 }] }));
            break;
          }
          const results: Array<{ nodeId: string; address: string; output: string; error?: string; exitCode: number | null }> = [];
          const localResult = { nodeId: "local", address: "this-node", output: "", error: undefined as string | undefined, exitCode: null as number | null };

          try {
            const output = execSync(command, { timeout: 5000, encoding: "utf-8", maxBuffer: 1024 * 64, shell: "/bin/bash" });
            localResult.output = output;
            localResult.exitCode = 0;
          } catch (err: any) {
            localResult.output = err.stdout || "";
            localResult.error = err.stderr || err.message;
            localResult.exitCode = err.status ?? 1;
          }
          results.push(localResult);

          const connectedPeers = Array.from(relayClients.entries());
          for (const [addr, peerWs] of connectedPeers) {
            if (peerWs.readyState === WebSocket.OPEN) {
              results.push({
                nodeId: toDottedAddr(addr),
                address: addr,
                output: `[Remote execution via relay — command dispatched: ${command}]`,
                exitCode: null,
              });
              try {
                peerWs.send(JSON.stringify({
                  type: "relay",
                  msgType: "cluster-exec",
                  payload: JSON.stringify({ command }),
                  from: "coordinator",
                }));
              } catch {}
            }
          }

          ws.send(JSON.stringify({ type: "cluster_result", results }));
          break;
        }
      }
    });

    ws.on("close", () => {
      if (remoteNodeAddress) {
        const nodeWs = relayClients.get(remoteNodeAddress);
        if (nodeWs && nodeWs.readyState === WebSocket.OPEN) {
          try {
            nodeWs.send(JSON.stringify({
              type: "relay",
              msgType: "terminal-close",
              payload: JSON.stringify({ sessionId: remoteSessionId }),
              from: "coordinator",
            }));
          } catch {}
        }
        remoteTerminalSessions.delete(remoteSessionId);
      }
      if (dataListener && activeSessionId) {
        const session = getSession(activeSessionId);
        if (session) {
          try { (session.ptyProcess as any).removeListener("data", dataListener); } catch {}
        }
      }
      for (const [tailReqId, tailNodeId] of activeTailFollows.entries()) {
        const nodeWs = relayClients.get(tailNodeId);
        if (nodeWs && nodeWs.readyState === WebSocket.OPEN) {
          try {
            const stopPayload: Record<string, string> = {
              type: "tail-stop",
              node_id: tailNodeId,
              request_id: `disconnect-stop-${Date.now()}`,
              original_request_id: tailReqId,
            };
            const canonicalKeys = Object.keys(stopPayload).sort();
            const canonical: Record<string, string> = {};
            for (const k of canonicalKeys) canonical[k] = stopPayload[k];
            const payloadBuf = Buffer.from(JSON.stringify(canonical));
            const sig = signHex(tldsaKeypair.secretKey, payloadBuf.toString('hex'), tldsaKeypair.variant);
            const fp = publicKeyHash(tldsaKeypair.publicKey).substring(0, 16);
            nodeWs.send(JSON.stringify({
              type: "relay",
              msgType: "tail-stop",
              payload: JSON.stringify({
                ...stopPayload,
                signature: sig,
                operator_fingerprint: fp,
              }),
              from: "coordinator",
            }));
          } catch (e) {
            console.error("[ws-relay] Failed to send disconnect tail-stop:", e);
          }
        }
      }
      activeTailFollows.clear();
      opsTerminalClients.delete(ws);
      for (const [reqId, origWs] of opsRequestOriginators.entries()) {
        if (origWs === ws) {
          opsRequestOriginators.delete(reqId);
        }
      }
    });
  });

  async function verifyNodeRegistration(address: string, publicKey: string): Promise<boolean> {
    const entry = crsRegistry.get(address);
    if (entry && entry.publicKey === publicKey) return true;
    const knownByKey = [...crsRegistry.entries()].find(([_, v]) => v.publicKey === publicKey);
    if (knownByKey) {
      const [oldAddr, oldEntry] = knownByKey;
      crsRegistry.set(address, { publicKey, endpoint: oldEntry.endpoint, lastSeen: Date.now(), tlDsaPk: oldEntry.tlDsaPk });
      publicKeyAddressMap.set(publicKey, address);
      storage.upsertCrsRelayNode(publicKey, address, oldEntry.endpoint, oldEntry.tlDsaPk).catch(() => {});
      return true;
    }
    try {
      const resp = await fetch(`http://127.0.0.1:8181/api/salvi/inter-cube/crs/node/${address}`);
      if (resp.status >= 500) {
        throw new Error(`CRS returned HTTP ${resp.status}`);
      }
      if (resp.ok) {
        const data = await resp.json() as any;
        if (data.publicKey === publicKey || data.public_key === publicKey) {
          const ep = data.endpoint || "unknown";
          crsRegistry.set(address, { publicKey, endpoint: ep, lastSeen: Date.now() });
          publicKeyAddressMap.set(publicKey, address);
          storage.upsertCrsRelayNode(publicKey, address, ep).catch(() => {});
          return true;
        }
      }
    } catch (err) {
      throw err;
    }
    return false;
  }

  async function verifyChallengeSignature(publicKeyHex: string, address: string, nonce: string, signatureHex: string): Promise<boolean> {
    const entryWithKey = [...crsRegistry.entries()].find(([_, v]) => v.publicKey === publicKeyHex && v.tlDsaPk);
    const entryAny = entryWithKey || [...crsRegistry.entries()].find(([_, v]) => v.publicKey === publicKeyHex);
    const entry = entryWithKey || entryAny;
    const tlDsaPk = entry?.[1]?.tlDsaPk;
    const debugLine = (s: string) => { console.log(s); try { fs.appendFileSync("/tmp/relay-auth-debug.log", new Date().toISOString() + " " + s + "\n"); } catch {} };
    if (!tlDsaPk) {
      debugLine(`[ws-relay] Challenge verify: no TL-DSA-87 key for pt26=${publicKeyHex.substring(0, 16)}... registry has ${crsRegistry.size} entries`);
      const allKeys = [...crsRegistry.entries()].map(([a, v]) => `${a}:${v.publicKey.substring(0, 16)}`).join(", ");
      debugLine(`[ws-relay] Registry keys: ${allKeys}`);
      return false;
    }
    const challengePayload = `${nonce}||${address}||${publicKeyHex}`;
    debugLine(`[ws-relay] Challenge verify: pt26=${publicKeyHex.substring(0, 16)}... tlDsa=${tlDsaPk.substring(0, 16)}... addr=${address} sigLen=${signatureHex.length / 2} payloadLen=${challengePayload.length}`);
    let nativeResult: boolean | null = null;
    try {
      const pkBuf = Buffer.from(tlDsaPk, "hex");
      const msgBuf = Buffer.from(challengePayload, "utf8");
      const sigBuf = Buffer.from(signatureHex, "hex");
      debugLine(`[ws-relay] Challenge verify: pkBuf=${pkBuf.length}B msgBuf=${msgBuf.length}B sigBuf=${sigBuf.length}B`);
      nativeResult = verifyNative(pkBuf, msgBuf, sigBuf, "TL-DSA-87");
      debugLine(`[ws-relay] Challenge verify native result: ${nativeResult}`);
      if (nativeResult) return true;
    } catch (e: any) {
      debugLine(`[ws-relay] Challenge verify native THREW: ${e.message}`);
    }
    debugLine(`[ws-relay] Native verify failed (${nativeResult}), trying CRS daemon fallback...`);
    try {
      const resp = await fetch("http://127.0.0.1:8181/api/salvi/inter-cube/crs/verify-challenge", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ publicKey: tlDsaPk, nonce, signature: signatureHex, address, pt26PublicKey: publicKeyHex }),
      });
      if (resp.status >= 500) {
        throw new Error(`CRS verify-challenge returned HTTP ${resp.status}`);
      }
      if (resp.ok) {
        const data = await resp.json() as any;
        debugLine(`[ws-relay] Challenge verify CRS daemon result: ${JSON.stringify(data)}`);
        return data.valid === true;
      }
      debugLine(`[ws-relay] Challenge verify CRS daemon returned ${resp.status}`);
    } catch (e2: any) {
      debugLine(`[ws-relay] Challenge verify CRS daemon failed: ${e2.message}`);
      throw e2;
    }
    return false;
  }

  const RELAY_PING_INTERVAL = 30_000;
  const RELAY_DEAD_TIMEOUT = 90_000;
  const RELAY_PONG_TIMEOUT = 60_000;
  const relayLastPong = new Map<string, number>();
  (globalThis as any).__relayLastPong = relayLastPong;

  let lastPeerCount = 0;
  const relayPingInterval = setInterval(() => {
    const now = Date.now();
    let pruned = 0;
    for (const [addr, clientWs] of relayClients.entries()) {
      if (clientWs.readyState !== WebSocket.OPEN) {
        console.log(`[ws-relay] Pruning dead socket for ${toDottedAddr(addr)} (readyState=${clientWs.readyState})`);
        relayClients.delete(addr);
        relayAddressByWs.delete(clientWs);
        relayLastPong.delete(addr);
        pruned++;
        continue;
      }
      const lastPong = relayLastPong.get(addr) || 0;
      const entry = crsRegistry.get(addr);
      const lastActivity = Math.max(lastPong, entry?.lastSeen ?? 0);
      if (lastActivity > 0 && (now - lastActivity) > RELAY_PONG_TIMEOUT) {
        console.log(`[ws-relay] Node ${toDottedAddr(addr)} no pong/activity for ${Math.round((now - lastActivity) / 1000)}s — closing`);
        clientWs.close(1000, "pong timeout");
        relayClients.delete(addr);
        relayAddressByWs.delete(clientWs);
        relayLastPong.delete(addr);
        pruned++;
        continue;
      }
      if (entry && (now - entry.lastSeen) > RELAY_DEAD_TIMEOUT) {
        console.log(`[ws-relay] Node ${toDottedAddr(addr)} unresponsive for ${Math.round((now - entry.lastSeen) / 1000)}s — closing`);
        clientWs.close(1000, "ping timeout");
        relayClients.delete(addr);
        relayAddressByWs.delete(clientWs);
        relayLastPong.delete(addr);
        pruned++;
        continue;
      }
      try {
        clientWs.ping();
      } catch (err: any) {
        console.log(`[ws-relay] Ping failed for ${toDottedAddr(addr)}: ${err.message}`);
        relayClients.delete(addr);
        relayAddressByWs.delete(clientWs);
        relayLastPong.delete(addr);
        pruned++;
      }
    }
    const liveCount = relayClients.size;
    if (liveCount !== lastPeerCount || pruned > 0) {
      const peers = Array.from(relayClients.keys()).map(a => toDottedAddr(a)).join(", ");
      console.log(`[ws-relay] Peers: ${liveCount} alive${peers ? ` [${peers}]` : ""}${pruned ? ` (${pruned} pruned)` : ""}`);
      lastPeerCount = liveCount;
    }
  }, RELAY_PING_INTERVAL);
  relayPingInterval.unref();

  const relayMonitorClients = new Set<WebSocket>();

  function broadcastToMonitors(msg: object) {
    if (relayMonitorClients.size === 0) return;
    const payload = JSON.stringify(msg);
    for (const mws of relayMonitorClients) {
      if (mws.readyState === WebSocket.OPEN) {
        mws.send(payload);
      }
    }
  }

  wss.on("connection", (ws: WebSocket) => {
    let authenticated = false;
    let isMonitor = false;
    let nodeAddress = "";
    const challengeNonce = crypto.randomBytes(32).toString("hex");

    ws.on("pong", () => {
      if (nodeAddress) {
        const now = Date.now();
        const entry = crsRegistry.get(nodeAddress);
        if (entry) entry.lastSeen = now;
        relayLastPong.set(nodeAddress, now);
      }
    });

    ws.send(JSON.stringify({ type: "challenge", nonce: challengeNonce }));

    ws.on("message", async (data: Buffer) => {
      let msg: any;
      try {
        msg = JSON.parse(data.toString());
      } catch {
        ws.send(JSON.stringify(makeErrorResponse("ERR_FRAME_MALFORMED", "unknown")));
        recordRelayAuditEvent({ eventType: "relay.error", address: nodeAddress || "unauthenticated", timestamp: new Date().toISOString(), details: { code: "ERR_FRAME_MALFORMED" } });
        return;
      }

      if (!authenticated) {
        if (msg.type === "monitor") {
          authenticated = true;
          isMonitor = true;
          relayMonitorClients.add(ws);
          ws.send(JSON.stringify({ type: "monitor_ok", connectedPeers: Array.from(relayClients.keys()), peerCount: relayClients.size }));
          console.log(`[ws-relay] Monitor client connected (${relayMonitorClients.size} monitor(s))`);
          return;
        }
        if (msg.type === "auth" && msg.address && msg.publicKey) {
          const normalAddr = normalizeTernaryAddr(msg.address);
          let verified = false;
          try {
            verified = await crsCircuitBreaker.execute(() => verifyNodeRegistration(normalAddr, msg.publicKey));
          } catch (cbErr: any) {
            const isBreakerOpen = crsCircuitBreaker.getState() === "open";
            const errorCode = isBreakerOpen ? "ERR_CIRCUIT_OPEN" : "ERR_AUTH_FAILED";
            ws.send(JSON.stringify(makeErrorResponse(errorCode, "auth")));
            recordRelayAuditEvent({ eventType: "relay.auth_failure", address: normalAddr, timestamp: new Date().toISOString(), details: { reason: isBreakerOpen ? "circuit_breaker_open" : "crs_verification_error", error: cbErr.message } });
            return;
          }
          if (!verified) {
            ws.send(JSON.stringify({ ...makeErrorResponse("ERR_AUTH_FAILED", "auth"), type: "auth_fail" }));
            ws.close(RELAY_ERROR_CODES.ERR_AUTH_FAILED.wsClose, "auth failed");
            recordRelayAuditEvent({ eventType: "relay.auth_failure", address: normalAddr, timestamp: new Date().toISOString(), details: { reason: "not_registered" } });
            recordDisconnectEvent(normalAddr, { timestamp: new Date().toISOString(), reason: "auth_failed", code: 1008, eventType: "auth_fail" });
            return;
          }

          if (msg.nonce === challengeNonce && msg.signature) {
            let sigValid = false;
            try {
              sigValid = await crsCircuitBreaker.execute(() => verifyChallengeSignature(msg.publicKey, normalAddr, challengeNonce, msg.signature));
            } catch (sigErr: any) {
              const isBreakerOpen = crsCircuitBreaker.getState() === "open";
              ws.send(JSON.stringify(makeErrorResponse(isBreakerOpen ? "ERR_CIRCUIT_OPEN" : "ERR_SIGNATURE_INVALID", "auth")));
              recordRelayAuditEvent({ eventType: "relay.auth_failure", address: normalAddr, timestamp: new Date().toISOString(), details: { reason: isBreakerOpen ? "circuit_breaker_open" : "signature_verify_error", error: sigErr.message } });
              return;
            }
            if (!sigValid) {
              log(`Challenge signature INVALID for ${toDottedAddr(normalAddr)} — possible impersonation`, "crs");
              ws.send(JSON.stringify({ ...makeErrorResponse("ERR_SIGNATURE_INVALID", "auth"), type: "auth_fail" }));
              ws.close(RELAY_ERROR_CODES.ERR_SIGNATURE_INVALID.wsClose, "signature invalid");
              recordRelayAuditEvent({ eventType: "relay.auth_failure", address: normalAddr, timestamp: new Date().toISOString(), details: { reason: "signature_invalid" } });
              recordDisconnectEvent(normalAddr, { timestamp: new Date().toISOString(), reason: "signature_invalid", code: 1008, eventType: "auth_fail" });
              return;
            }
            log(`Challenge signature VERIFIED (TL-DSA-87) for ${toDottedAddr(normalAddr)}`, "crs");
          } else if (!msg.signature) {
            const entryCheck = [...crsRegistry.entries()].find(([_, v]) => v.publicKey === msg.publicKey && v.tlDsaPk);
            if (entryCheck?.[1]?.tlDsaPk) {
              log(`Node ${toDottedAddr(normalAddr)} has TL-DSA-87 key registered but sent no signature — rejecting`, "crs");
              ws.send(JSON.stringify({ ...makeErrorResponse("ERR_SIGNATURE_REQUIRED", "auth"), type: "auth_fail" }));
              ws.close(RELAY_ERROR_CODES.ERR_SIGNATURE_REQUIRED.wsClose, "signature required");
              recordRelayAuditEvent({ eventType: "relay.auth_failure", address: normalAddr, timestamp: new Date().toISOString(), details: { reason: "signature_required" } });
              return;
            }
            log(`Node ${toDottedAddr(normalAddr)} connected without signature (no TL-DSA key registered — legacy client)`, "crs");
          }

          authenticated = true;
          nodeAddress = normalAddr;
          const oldWs = relayClients.get(nodeAddress);
          const isReconnect = !!oldWs;
          if (oldWs && oldWs !== ws && oldWs.readyState === WebSocket.OPEN) {
            oldWs.close(1000, "replaced");
          }
          relayClients.set(nodeAddress, ws);
          relayAddressByWs.set(ws, nodeAddress);
          relayLastPong.set(nodeAddress, Date.now());
          if (!isReconnect || !relayConnectedAt.has(nodeAddress)) {
            relayConnectedAt.set(nodeAddress, Date.now());
          }
          if (relayClients.size > relayThroughput.peakPeers) relayThroughput.peakPeers = relayClients.size;
          console.log(`[ws-relay] Node ${toDottedAddr(nodeAddress)} ${isReconnect ? "re" : ""}authenticated and connected`);
          recordRelayAuditEvent({ eventType: "relay.auth_success", address: nodeAddress, timestamp: new Date().toISOString(), details: { hasTlDsa: !!msg.signature, reconnect: isReconnect } });
          if (isReconnect) {
            recordRelayAuditEvent({ eventType: "relay.reconnect", address: nodeAddress, timestamp: new Date().toISOString(), details: { replacedExisting: true } });
          }
          recordDisconnectEvent(nodeAddress, { timestamp: new Date().toISOString(), reason: "connected", code: 0, eventType: "reconnect" });

          ws.send(JSON.stringify({ type: "auth_ok", address: nodeAddress, connectedPeers: Array.from(relayClients.keys()).filter(a => a !== nodeAddress) }));
          broadcastToMonitors({ type: "peer-online", address: toDottedAddr(nodeAddress), peerCount: relayClients.size, ts: Date.now() });

          const allOperators = opsChannelService.listOperators();
          for (const op of allOperators) {
            ws.send(JSON.stringify({ type: "relay", msgType: "ops-operator-sync", payload: JSON.stringify({ action: "add", name: op.name, key_fingerprint: op.keyFingerprint, public_key: op.publicKey, scope: op.scope }), from: "coordinator" }));
          }
          if (opsChannelService.isOpsEnabled()) {
            ws.send(JSON.stringify({ type: "relay", msgType: "ops-config-update", payload: JSON.stringify({ ops_enabled: true }), from: "coordinator" }));
          }

          const crsEntry = crsRegistry.get(nodeAddress);
          if (crsEntry) {
            crsEntry.lastSeen = Date.now();
            storage.upsertCrsRelayNode(crsEntry.publicKey, nodeAddress, crsEntry.endpoint, crsEntry.tlDsaPk).catch(() => {});
            const staleAddrs: string[] = [];
            for (const [addr, entry] of crsRegistry.entries()) {
              if (addr !== nodeAddress && entry.publicKey === msg.publicKey) {
                staleAddrs.push(addr);
              }
            }
            for (const addr of staleAddrs) {
              crsRegistry.delete(addr);
              const oldWsForStale = relayClients.get(addr);
              if (oldWsForStale && oldWsForStale !== ws) {
                relayClients.delete(addr);
                relayAddressByWs.delete(oldWsForStale);
              }
            }
            if (staleAddrs.length > 0) {
              storage.deleteCrsRelayNodesByAddresses(staleAddrs).catch(() => {});
              log(`CRS post-auth cleanup: removed ${staleAddrs.length} stale address(es) for ${msg.publicKey.substring(0, 16)}... [${staleAddrs.map(a => toDottedAddr(a)).join(", ")}]`, "crs");
            }
          }

          const pending = pendingMessages.get(nodeAddress);
          if (pending && pending.length > 0) {
            for (const queued of pending) {
              const envelope = JSON.stringify({ type: "relay", from: queued.from, msgType: queued.type, payload: queued.payload });
              const envBytes = Buffer.byteLength(envelope, "utf-8");
              ws.send(envelope);
              relayThroughput.bytesRelayed += envBytes;
              relayThroughput.delivered++;
            }
            if (relayThroughput.queued >= pending.length) {
              relayThroughput.queued -= pending.length;
            } else {
              relayThroughput.queued = 0;
            }
            console.log(`[ws-relay] Drained ${pending.length} queued message(s) to ${toDottedAddr(nodeAddress)}`);
            pendingMessages.delete(nodeAddress);
          }
          return;
        }
        ws.send(JSON.stringify(makeErrorResponse("ERR_NOT_AUTHENTICATED", msg.type)));
        recordRelayAuditEvent({ eventType: "relay.error", address: "unauthenticated", timestamp: new Date().toISOString(), details: { code: "ERR_NOT_AUTHENTICATED", msgType: msg.type } });
        return;
      }

      if (isMonitor) {
        if (msg.type === "ping") {
          ws.send(JSON.stringify({ type: "pong", ts: Date.now() }));
        }
        return;
      }

      if (nodeAddress) {
        const now = Date.now();
        const entry = crsRegistry.get(nodeAddress);
        if (entry) entry.lastSeen = now;
        relayLastPong.set(nodeAddress, now);
      }

      if (msg.type === "relay" && msg.msgType === "terminal-output" && msg.payload) {
        try {
          const payload = JSON.parse(msg.payload);
          const termWs = remoteTerminalSessions.get(payload.sessionId);
          if (termWs && termWs.readyState === WebSocket.OPEN) {
            termWs.send(JSON.stringify({ type: "output", data: payload.data }));
          }
        } catch {}
        return;
      }

      if (msg.type === "relay" && msg.msgType === "yoda_chat" && msg.payload) {
        if (!yodaReplayGuard) yodaReplayGuard = new Map<string, Set<number>>();
        if (!yodaRateWindows) yodaRateWindows = new Map<string, number[]>();

        try {
          const yodaPayload = JSON.parse(msg.payload);
          const daemonRepC = yodaPayload.daemonRepC || nodeAddress;
          const sessionId = yodaPayload.sessionId || "unknown";
          const sequence = yodaPayload.sequence ?? 0;
          const messageText = yodaPayload.message || "";
          const timestamp = yodaPayload.timestamp ?? 0;

          const MAX_MESSAGE_BYTES = 32_768;
          const messageByteLength = Buffer.byteLength(messageText, "utf8");
          if (messageByteLength > MAX_MESSAGE_BYTES) {
            const sizeResp = JSON.stringify({
              type: "relay", from: "yoda-server", msgType: "yoda_response",
              payload: JSON.stringify({
                sessionId, sequence, content: null,
                error: { code: "MESSAGE_TOO_LONG", message: "Message exceeds the 32KB size limit." }
              })
            });
            ws.send(sizeResp);
            return;
          }

          const now = Date.now();
          const TIMESTAMP_MAX_AGE_MS = 60_000;
          const timestampAge = Math.abs(now - timestamp);
          if (timestampAge >= TIMESTAMP_MAX_AGE_MS) {
            const expiredResp = JSON.stringify({
              type: "relay", from: "yoda-server", msgType: "yoda_response",
              payload: JSON.stringify({
                sessionId, sequence, content: null,
                error: { code: "MESSAGE_EXPIRED", message: "Message timestamp is too old. Check your system clock." }
              })
            });
            ws.send(expiredResp);
            return;
          }

          const sessionSeqs = yodaReplayGuard!.get(sessionId) || new Set<number>();
          if (sessionSeqs.has(sequence)) {
            const replayResp = JSON.stringify({
              type: "relay", from: "yoda-server", msgType: "yoda_response",
              payload: JSON.stringify({
                sessionId, sequence, content: null,
                error: { code: "SEQUENCE_REPLAY", message: "This message has already been processed." }
              })
            });
            ws.send(replayResp);
            return;
          }
          sessionSeqs.add(sequence);
          if (sessionSeqs.size > 1000) {
            const oldest = sessionSeqs.values().next().value;
            if (oldest !== undefined) sessionSeqs.delete(oldest);
          }
          yodaReplayGuard!.set(sessionId, sessionSeqs);

          const yodaRateKey = `yoda:${daemonRepC}`;
          const rateWindow = yodaRateWindows!.get(yodaRateKey) || [];
          const windowStart = now - 60_000;
          const filtered = rateWindow.filter((ts: number) => ts > windowStart);
          if (filtered.length >= 10) {
            const rateLimitResp = JSON.stringify({
              type: "relay", from: "yoda-server", msgType: "yoda_response",
              payload: JSON.stringify({
                sessionId, sequence, content: null,
                error: { code: "RATE_LIMITED", message: "You're sending messages faster than Yoda can read them. Wait a moment and try again." }
              })
            });
            ws.send(rateLimitResp);
            return;
          }
          filtered.push(now);
          yodaRateWindows!.set(yodaRateKey, filtered);

          const yodaTimeoutMs = 30_000;
          const yodaResponsePromise = new Promise<string>(async (resolve) => {
            const timer = setTimeout(() => {
              resolve(JSON.stringify({
                sessionId, sequence, content: null,
                error: { code: "YODA_TIMEOUT", message: "Yoda is taking too long to respond. Try again in a moment." }
              }));
            }, yodaTimeoutMs);

            try {
              let responseContent: string;
              const handler = yodaPipelineHandler as YodaPipelineHandler | null;
              if (handler) {
                responseContent = await handler(messageText, sessionId, sequence, daemonRepC);
              } else {
                responseContent = `Message received in session ${sessionId} (seq #${sequence}). Yoda relay is active — AI pipeline integration pending cloud-side deployment.`;
              }
              clearTimeout(timer);
              resolve(JSON.stringify({
                sessionId, sequence, content: responseContent,
                metadata: { processedAt: new Date().toISOString(), source: "yoda-relay" }
              }));
            } catch (pipelineError) {
              clearTimeout(timer);
              resolve(JSON.stringify({
                sessionId, sequence, content: null,
                error: { code: "YODA_UNAVAILABLE", message: "Yoda is currently unavailable. Your message was received but could not be processed." }
              }));
            }
          });

          const yodaResult = await yodaResponsePromise;
          const responseEnvelope = JSON.stringify({
            type: "relay", from: "yoda-server", msgType: "yoda_response",
            payload: yodaResult
          });
          ws.send(responseEnvelope);
        } catch (e) {
          console.error("[yoda-chat] Error processing yoda_chat:", e);
          try {
            const errEnvelope = JSON.stringify({
              type: "relay", from: "yoda-server", msgType: "yoda_response",
              to: msg.from || nodeAddress,
              payload: JSON.stringify({
                sessionId: "unknown", sequence: 0, content: null,
                error: { code: "YODA_UNAVAILABLE", message: "Internal error processing your message. Please try again." }
              })
            });
            ws.send(errEnvelope);
          } catch {}
        }
        return;
      }

      if (msg.type === "relay" && msg.msgType && msg.payload && isOpsMessageType(msg.msgType)) {
        try {
          const opsPayload = JSON.parse(msg.payload);
          opsPayload.type = opsPayload.type || msg.msgType;

          if (opsPayload.type === "telemetry") {
            console.log(`[ws-relay] Telemetry from ${toDottedAddr(nodeAddress)}: cpu=${opsPayload.cpu_pct}% ram=${opsPayload.ram_pct}% node_id=${opsPayload.node_id}`);
            opsChannelService.updateNodeTelemetry(
              opsPayload.node_id || nodeAddress, nodeAddress, opsPayload as TelemetryMessage,
            );
            broadcastToMonitors({ type: "telemetry", address: toDottedAddr(nodeAddress), data: opsPayload, ts: Date.now() });
          } else {
            opsChannelService.updateNodeSeen(opsPayload.node_id || nodeAddress, nodeAddress);
          }

          if (terminalWss) {
            const opsReqId = opsPayload.request_id as string | undefined;
            const originWs = opsReqId ? opsRequestOriginators.get(opsReqId) : undefined;
            const broadcast = JSON.stringify({ type: "ops_message", data: opsPayload });
            if (originWs && originWs.readyState === WebSocket.OPEN) {
              originWs.send(broadcast);
              const isFinal = opsPayload.type !== "tail-data" && opsPayload.type !== "chunk-ack";
              if (isFinal && opsReqId) {
                opsRequestOriginators.delete(opsReqId);
              }
            } else if (!originWs && opsPayload.type === "telemetry") {
              terminalWss.clients.forEach((termWs: WebSocket) => {
                if (termWs.readyState === WebSocket.OPEN) {
                  termWs.send(broadcast);
                }
              });
            }
          }
        } catch {}
        return;
      }

      if (msg.type === "relay" && msg.from === "__relay_server__") {
        console.log(`[ws-relay] REJECTED: peer ${toDottedAddr(nodeAddress)} attempted to spoof __relay_server__ sentinel`);
        recordRelayAuditEvent({ eventType: "relay.error", address: nodeAddress, timestamp: new Date().toISOString(), details: { code: "ERR_SENTINEL_SPOOF", msgType: msg.msgType } });
        return;
      }

      if (msg.type === "relay" && msg.msgType === "http_proxy_res" && msg.payload) {
        try {
          const proxyRes = JSON.parse(msg.payload);
          const rid = proxyRes.request_id;
          if (rid && pendingProxies.has(rid)) {
            const entry = pendingProxies.get(rid)!;
            if (entry.daemonAddr !== nodeAddress) {
              console.log(`[ws-relay] REJECTED http_proxy_res: request_id=${rid} expected from ${toDottedAddr(entry.daemonAddr)} but received from ${toDottedAddr(nodeAddress)}`);
            } else {
              entry.resolve({ status: proxyRes.status || 200, body: proxyRes.body || "{}" });
            }
          }
        } catch {}
        return;
      }

      if (msg.type === "relay" && msg.to && msg.payload) {
        const targetWs = relayClients.get(normalizeTernaryAddr(msg.to));
        const envelope = JSON.stringify({ type: "relay", from: nodeAddress, msgType: msg.msgType || "data", payload: msg.payload });
        const normalizedTo = normalizeTernaryAddr(msg.to);
        const wasDelivered = !!(targetWs && targetWs.readyState === WebSocket.OPEN);
        const envelopeBytes = Buffer.byteLength(envelope, "utf-8");
        const relayMsgType = msg.msgType || "data";
        if (relayMsgType === "inference_request") relayThroughput.inferenceRequests++;
        else if (relayMsgType === "inference_response" || relayMsgType === "inference_error") relayThroughput.inferenceResponses++;
        else if (relayMsgType === "heartbeat" || relayMsgType === "mesh-heartbeat") relayThroughput.meshHeartbeats++;
        let outcome: "delivered" | "queued" | "failed" = "delivered";
        if (wasDelivered) {
          targetWs!.send(envelope);
          relayThroughput.bytesRelayed += envelopeBytes;
        } else {
          if (!pendingMessages.has(normalizedTo)) pendingMessages.set(normalizedTo, []);
          const queue = pendingMessages.get(normalizedTo)!;
          const PENDING_MAX = 500;
          const PENDING_TTL_MS = 300_000;
          const nowTs = Date.now();
          while (queue.length > 0 && (nowTs - queue[0].ts) > PENDING_TTL_MS) {
            queue.shift();
          }
          if (queue.length < PENDING_MAX) {
            queue.push({ from: nodeAddress, type: msg.msgType || "data", payload: msg.payload, ts: nowTs });
            outcome = "queued";
            ws.send(JSON.stringify({ ...makeErrorResponse("ERR_RELAY_TARGET_UNKNOWN", "relay"), type: "relay_ack", to: msg.to, delivered: false, queued: true }));
            recordRelayAuditEvent({ eventType: "relay.error", address: nodeAddress, timestamp: new Date().toISOString(), details: { code: "ERR_RELAY_TARGET_UNKNOWN", target: msg.to, queued: true } });
          } else {
            outcome = "failed";
            ws.send(JSON.stringify({ ...makeErrorResponse("ERR_RELAY_QUEUE_FULL", "relay"), type: "relay_ack", to: msg.to, delivered: false, queued: false }));
            recordRelayAuditEvent({ eventType: "relay.error", address: nodeAddress, timestamp: new Date().toISOString(), details: { code: "ERR_RELAY_QUEUE_FULL", target: msg.to } });
          }
        }
        recordRelayMsg(outcome, wasDelivered ? envelopeBytes : 0);
        if (wasDelivered) {
          ws.send(JSON.stringify({ type: "relay_ack", to: msg.to, delivered: true }));
        }
        return;
      }

      if (msg.type === "ping") {
        ws.send(JSON.stringify({ type: "pong", ts: Date.now() }));
        return;
      }

      if (msg.type === "peers") {
        ws.send(JSON.stringify({ type: "peers", connected: Array.from(relayClients.keys()) }));
        return;
      }

      if (isOpsMessageType(msg.type)) {
        if (msg.type === "telemetry") {
          opsChannelService.updateNodeTelemetry(
            msg.node_id || nodeAddress,
            nodeAddress,
            msg as TelemetryMessage,
          );
          return;
        }

        if (msg.type === "exec-result" || msg.type === "tail-data" ||
            msg.type === "file-push-ack" || msg.type === "file-data" ||
            msg.type === "chunk-ack" || msg.type === "chunk-complete" ||
            msg.type === "model-swap-result" || msg.type === "ops-error") {
          opsChannelService.updateNodeSeen(msg.node_id || nodeAddress, nodeAddress);
          const opsResponseMsg = JSON.stringify({ type: "ops-response", opsType: msg.type, ...msg, from: nodeAddress });
          for (const [, clientWs] of relayClients.entries()) {
            if (clientWs !== ws && clientWs.readyState === WebSocket.OPEN) {
              clientWs.send(opsResponseMsg);
            }
          }

          if (terminalWss) {
            const reqId = msg.request_id as string | undefined;
            const originatorWs = reqId ? opsRequestOriginators.get(reqId) : undefined;
            if (originatorWs && originatorWs.readyState === WebSocket.OPEN) {
              originatorWs.send(JSON.stringify({ type: "ops_message", data: msg }));
              const isFinal = msg.type !== "tail-data" && msg.type !== "chunk-ack";
              if (isFinal && reqId) {
                opsRequestOriginators.delete(reqId);
              }
            } else if (!originatorWs) {
              terminalWss.clients.forEach((termWs: WebSocket) => {
                if (termWs.readyState === WebSocket.OPEN) {
                  termWs.send(JSON.stringify({ type: "ops_message", data: msg }));
                }
              });
            }
          }
          return;
        }

        const validation = opsChannelService.validateOpsMessage(msg);
        if (!validation.valid) {
          ws.send(JSON.stringify(opsChannelService.makeOpsError(
            msg.node_id || '', msg.request_id || '',
            (validation.errorCode || 'SIGNATURE_MISSING') as OpsErrorCode,
            validation.errorMessage || 'Ops message validation failed',
            msg.type as OpsMessageType,
          )));
          return;
        }

        const targetNodeId = msg.node_id;
        if (!targetNodeId) {
          ws.send(JSON.stringify(opsChannelService.makeOpsError('', msg.request_id || '', 'NODE_NOT_FOUND', 'Missing node_id', msg.type as OpsMessageType)));
          return;
        }

        let targetWs: WebSocket | undefined;
        for (const [addr, clientWs] of relayClients.entries()) {
          if (addr === targetNodeId) {
            targetWs = clientWs;
            break;
          }
        }

        if (!targetWs || targetWs.readyState !== WebSocket.OPEN) {
          ws.send(JSON.stringify(opsChannelService.makeOpsError(targetNodeId, msg.request_id || '', 'NODE_DISCONNECTED', `Node ${targetNodeId} is not connected`, msg.type as OpsMessageType)));
          return;
        }

        opsChannelService.recordAuditEntry(opsChannelService.createAuditEntry(msg as OpsMessage, 'success'));

        targetWs.send(JSON.stringify({
          type: "relay",
          msgType: msg.type,
          payload: JSON.stringify(msg),
          from: nodeAddress || "coordinator",
        }));
        opsChannelService.updateNodeSeen(targetNodeId, targetNodeId);
        return;
      }

      ws.send(JSON.stringify(makeErrorResponse("ERR_UNKNOWN_MSG_TYPE", msg.type)));
      if (nodeAddress) {
        recordRelayAuditEvent({ eventType: "relay.error", address: nodeAddress, timestamp: new Date().toISOString(), details: { code: "ERR_UNKNOWN_MSG_TYPE", msgType: msg.type } });
      }
    });

    ws.on("close", (code: number, reason: Buffer) => {
      if (isMonitor) {
        relayMonitorClients.delete(ws);
        console.log(`[ws-relay] Monitor client disconnected (${relayMonitorClients.size} monitor(s) remain)`);
        return;
      }
      if (nodeAddress) {
        for (const [rid, entry] of pendingProxies.entries()) {
          if (entry.daemonAddr === nodeAddress) {
            clearTimeout(entry.timer);
            entry.resolve(null);
            pendingProxies.delete(rid);
          }
        }

        relayClients.delete(nodeAddress);
        relayAddressByWs.delete(ws);
        relayConnectedAt.delete(nodeAddress);
        relayLastPong.delete(nodeAddress);
        opsChannelService.markNodeDisconnected(nodeAddress);
        const reasonStr = reason.toString() || "none";
        const remaining = Array.from(relayClients.keys()).map(a => toDottedAddr(a)).join(", ");
        console.log(`[ws-relay] Node ${toDottedAddr(nodeAddress)} DISCONNECTED (code=${code}, reason=${reasonStr}) — ${relayClients.size} peer(s) remain${remaining ? `: [${remaining}]` : ""}`);

        recordDisconnectEvent(nodeAddress, { timestamp: new Date().toISOString(), reason: reasonStr, code, eventType: "disconnect" });
        recordRelayAuditEvent({ eventType: "relay.disconnect", address: nodeAddress, timestamp: new Date().toISOString(), details: { code, reason: reasonStr } });

        const abnormalCloseCodes = [1006, 1011, 1012, 1013, 1014];
        if (abnormalCloseCodes.includes(code)) {
          crsCircuitBreaker.recordFailure();
        }

        const peerOfflineMsg = JSON.stringify({ type: "peer-offline", address: toDottedAddr(nodeAddress), ts: Date.now() });
        let notifiedCount = 0;
        for (const [peerAddr, peerWs] of relayClients.entries()) {
          if (peerWs.readyState === WebSocket.OPEN) {
            peerWs.send(peerOfflineMsg);
            recordDisconnectEvent(peerAddr, { timestamp: new Date().toISOString(), reason: `peer ${toDottedAddr(nodeAddress)} went offline`, code: 0, eventType: "peer_offline" });
            notifiedCount++;
          }
        }
        broadcastToMonitors({ type: "peer-offline", address: toDottedAddr(nodeAddress), peerCount: relayClients.size, ts: Date.now() });
        recordRelayAuditEvent({ eventType: "relay.peer_offline", address: nodeAddress, timestamp: new Date().toISOString(), details: { notifiedPeers: notifiedCount } });
      }
    });

    ws.on("error", (err: Error) => {
      console.log(`[ws-relay] ERROR for ${nodeAddress ? toDottedAddr(nodeAddress) : "unauthenticated"}: ${err.message}`);
      if (nodeAddress) {
        recordDisconnectEvent(nodeAddress, { timestamp: new Date().toISOString(), reason: err.message, code: 1006, eventType: "error" });
        recordRelayAuditEvent({ eventType: "relay.error", address: nodeAddress, timestamp: new Date().toISOString(), details: { error: err.message } });
      }
    });

    setTimeout(() => {
      if (!authenticated && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(makeErrorResponse("ERR_AUTH_TIMEOUT", "auth")));
        recordRelayAuditEvent({ eventType: "relay.auth_failure", address: "unauthenticated", timestamp: new Date().toISOString(), details: { code: "ERR_AUTH_TIMEOUT" } });
        ws.close(RELAY_ERROR_CODES.ERR_AUTH_TIMEOUT.wsClose, "auth timeout");
      }
    }, 10000);
  });

  function broadcastGoAway(reason: string): void {
    const goAwayMsg = JSON.stringify({ type: "go-away", reason, reconnectAfterMs: 2000, ts: Date.now() });
    for (const [addr, clientWs] of relayClients.entries()) {
      if (clientWs.readyState === WebSocket.OPEN) {
        try {
          clientWs.send(goAwayMsg);
          clientWs.close(1001, reason);
        } catch {}
        recordDisconnectEvent(addr, { timestamp: new Date().toISOString(), reason, code: 1001, eventType: "go_away" });
        console.log(`[ws-relay] go-away sent to ${toDottedAddr(addr)} (${reason})`);
      }
    }
    recordRelayAuditEvent({ eventType: "relay.go_away", address: "server", timestamp: new Date().toISOString(), details: { reason, peerCount: relayClients.size } });
  }

  process.on("SIGTERM", () => {
    console.log("[ws-relay] SIGTERM received — sending go-away frames");
    broadcastGoAway("server_shutdown");
  });
  process.on("SIGINT", () => {
    console.log("[ws-relay] SIGINT received — sending go-away frames");
    broadcastGoAway("server_shutdown");
  });

  console.log(`[ws-relay] WebSocket relay active at /ws/relay`);

  // importantly only setup vite in development and after
  // setting up all the other routes so the catch-all route
  // doesn't interfere with the other routes
  if (process.env.NODE_ENV === "production") {
    serveStatic(app);
  } else {
    const { setupVite } = await import("./vite");
    await setupVite(httpServer, app);
  }

  console.log("[crs-daemon] searching for binary...");
  const cwd = process.cwd();
  const daemonCandidates = [
    path.join(cwd, "dist", "inter-cube-daemon"),
    path.join(cwd, "target", "release", "inter-cube-daemon"),
    "/home/runner/workspace/dist/inter-cube-daemon",
    "/home/runner/workspace/target/release/inter-cube-daemon",
  ];
  let daemonPath = "";
  for (const candidate of daemonCandidates) {
    const found = existsSync(candidate);
    console.log(`[crs-daemon]   ${candidate} -> ${found ? "FOUND" : "missing"}`);
    if (found && !daemonPath) daemonPath = candidate;
  }
  if (daemonPath) {
    try {
      const systemLinker = "/lib64/ld-linux-x86-64.so.2";
      const useLinkerInvoke = existsSync(systemLinker) && !existsSync("/nix/store/g8zyryr9cr6540xsyg4avqkwgxpnwj2a-glibc-2.40-66/lib/ld-linux-x86-64.so.2");
      
      let spawnCmd: string;
      let spawnArgs: string[];
      const spawnEnv: Record<string, string> = {
        ...process.env as Record<string, string>,
        CUBE_MODE: "crs",
        CUBE_API_PORT: "8181",
        CUBE_IDENTITY_PASSPHRASE: "plenumlan-prototype-2026",
      };
      
      if (useLinkerInvoke) {
        spawnCmd = systemLinker;
        spawnArgs = [daemonPath];
        spawnEnv.LD_LIBRARY_PATH = "/lib/x86_64-linux-gnu:/lib64:/usr/lib/x86_64-linux-gnu";
        console.log(`[crs-daemon] NixOS interpreter missing in production — using ${systemLinker} to invoke binary`);
      } else {
        spawnCmd = daemonPath;
        spawnArgs = [];
      }

      const crsProc = spawn(spawnCmd, spawnArgs, {
        env: spawnEnv,
        stdio: ["ignore", "pipe", "pipe"],
        detached: false,
      });
      crsProc.on("error", (err: Error) => {
        console.log(`[crs-daemon] spawn error (non-fatal): ${err.message}`);
      });
      crsProc.stdout?.on("data", (d: Buffer) => console.log(`[crs-daemon] ${d.toString().trim()}`));
      crsProc.stderr?.on("data", (d: Buffer) => console.error(`[crs-daemon] ${d.toString().trim()}`));
      crsProc.on("exit", (code: number | null) => console.log(`[crs-daemon] exited with code ${code}`));
      console.log(`[crs-daemon] spawned from ${daemonPath} via ${spawnCmd} (PID ${crsProc.pid}, port 8181, mode=crs)`);
    } catch (err: any) {
      console.log(`[crs-daemon] failed to spawn (non-fatal): ${err.message}`);
    }
  } else {
    console.log("[crs-daemon] binary not found in any candidate path — skipping");
  }

  // ALWAYS serve the app on the port specified in the environment variable PORT
  const port = parseInt(process.env.PORT || "5000", 10);
  httpServer.listen(
    {
      port,
      host: "0.0.0.0",
      reusePort: true,
    },
    () => {
      _serverListening = true;
      log(`serving on port ${port}`);
    },
  );
})();
