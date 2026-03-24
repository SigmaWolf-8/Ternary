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
import { globalLimiter } from "./middleware/rate-limiter";
import { spawn, type ChildProcess } from "child_process";
import { existsSync } from "fs";
import { WebSocketServer, WebSocket } from "ws";
import * as path from "path";
import { spongeHashTrits } from "./crypto/sponge-hash";
import { TsaService, type TsaConfig, TSA_POLICIES, type HptpClient, type TldsaClient } from "./services/tsa-service";
import { createTsaRoutes } from "./routes/tsa";
import { type CalendarServiceClient } from "./services/tsa-calendar-enrichment";
import { keygen, signHex, verifyHex, verifyNative, publicKeyHash, type TlDsaKeyPair } from "./crypto/tl-dsa-bridge";
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
import { NotificationService, tsaMetricsRegistry, EFFECTIVE_PHASE } from "./services/notification-service";
import { HederaWitnessingService, createHederaConfig } from "./services/hedera-witnessing-service";
import { createHederaRoutes } from "./routes/hedera";
import { SFKOperationsService } from "./services/sfk-operations-service";
import { createSFKOperationsRoutes } from "./routes/sfk-operations";

const app = express();
const httpServer = createServer(app);

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
    const allowed = new Set(["Install-PlenumNET.bat", "install-windows.ps1", "install.sh"]);
    const { filename } = req.params;
    if (!allowed.has(filename)) {
      return res.status(404).json({ error: "Not found" });
    }
    const filePath = path.resolve(process.cwd(), "client", "public", "install", filename);
    if (!fs.existsSync(filePath)) {
      return res.status(404).json({ error: "File not found" });
    }
    res.setHeader("Content-Type", "application/octet-stream");
    res.setHeader("Content-Disposition", `attachment; filename="${filename}"`);
    res.sendFile(filePath);
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

  app.get("/api/salvi/inter-cube/relay/register", async (req, res) => {
    const { publicKey, endpoint, tlDsaPk } = req.query as { publicKey?: string; endpoint?: string; tlDsaPk?: string };
    if (!publicKey || !endpoint) {
      return res.status(400).json({ error: "publicKey and endpoint query params required" });
    }

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
        addressDotted: toDottedAddr(addr),
        connected: ws.readyState === 1,
        endpoint: crsReg.get(addr)?.endpoint || null,
      }));
      return res.json({ connectedNodes: nodes.length, nodes, pendingQueues: pendingRef?.size || 0 });
    }
    res.json({ connectedNodes: 0, nodes: [], pendingQueues: 0 });
  });

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
  const CRS_VERSION = "0.3.0";

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

  app.get("/api/salvi/inter-cube/relay/cluster-health", async (_req, res) => {
    try {
      const records = await storage.getAllDeploymentRecords();
      const relayClientsRef = (globalThis as any).__relayClients as Map<string, WebSocket> | undefined;

      const now = Date.now();
      const daemonChecks: Array<{
        address: string;
        endpoint: string;
        port: number;
        hostname: string;
        deploymentId: number;
        role: "crs" | "cube";
        registeredInCrs: boolean;
        connectedViaRelay: boolean;
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
          daemonChecks.push({
            address: toDottedAddr(normalAddr),
            endpoint: d.endpoint || "",
            port: d.port || 0,
            hostname: record.hostname || "",
            deploymentId: record.id,
            role: isCrs ? "crs" : "cube",
            registeredInCrs: isRegistered,
            connectedViaRelay: !!isRelayConnected,
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

      for (const [crsAddr, crsEntry] of crsRegistry.entries()) {
        if (seenAddresses.has(crsAddr)) continue;
        seenAddresses.add(crsAddr);
        const isRelayConnected = relayClientsRef?.has(crsAddr) && relayClientsRef.get(crsAddr)!.readyState === 1;
        const isCrs = crsAddr === CRS_ORIGIN;
        const lastSeenTs = crsEntry.lastSeen;
        const healthState = isRelayConnected ? "up" as NodeHealthState : computeHealthState(lastSeenTs, now);
        daemonChecks.push({
          address: toDottedAddr(crsAddr),
          endpoint: crsEntry.endpoint || "",
          port: 0,
          hostname: "",
          deploymentId: 0,
          role: isCrs ? "crs" : "cube",
          registeredInCrs: true,
          connectedViaRelay: !!isRelayConnected,
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

      const crsLiveAddresses = new Set<string>();
      const crsCoveredHostPorts = new Set<string>();
      for (const [addr, crsEntry] of crsRegistry.entries()) {
        crsLiveAddresses.add(addr);
        const parsed = parseHostPort(crsEntry.endpoint || "");
        if (parsed) crsCoveredHostPorts.add(`${parsed.host}:${parsed.port}`);
      }
      for (const d of daemonChecks) {
        if (d.source === "crs" && d.registeredInCrs) {
          const parsed = parseHostPort(d.endpoint || "");
          if (parsed) crsCoveredHostPorts.add(`${parsed.host}:${parsed.port}`);
        }
      }

      function isStaleDeploymentEntry(d: typeof daemonChecks[0]): boolean {
        if (d.source !== "deployment" || d.status !== "deployed") return false;
        if (d.registeredInCrs || d.connectedViaRelay) return false;
        const depParsed = parseHostPort(d.endpoint || "");
        if (depParsed && crsCoveredHostPorts.has(`${depParsed.host}:${depParsed.port}`)) return true;
        if (d.port && d.hostname) {
          for (const hp of crsCoveredHostPorts) {
            const [, p] = hp.split(":");
            if (p === String(d.port)) return true;
          }
        }
        return false;
      }

      const filteredDaemons = daemonChecks.filter(d => !isStaleDeploymentEntry(d));

      const expectedNodesList = Array.from(getExpectedNodesCache());
      const expectedNodesStatus = expectedNodesList
        .filter(addr => {
          const normalAddr = normalizeTernaryAddr(addr);
          const inCrs = crsRegistry.has(normalAddr);
          const inRelay = relayClientsRef?.has(normalAddr) && relayClientsRef.get(normalAddr)!.readyState === 1;
          const inDeployments = seenAddresses.has(normalAddr);
          if (!inCrs && !inRelay && !inDeployments) return false;
          return true;
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
          expectedCount: expectedNodesList.length,
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
    try {
      const scriptPath = path.resolve("services/inter-cube/deploy-yoda.ps1");
      const { readFile } = await import("fs/promises");
      const script = await readFile(scriptPath, "utf-8");
      res.setHeader("Content-Type", "text/plain; charset=utf-8");
      res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate");
      res.setHeader("Pragma", "no-cache");
      res.send(script);
    } catch {
      res.status(404).send("# deploy-yoda.ps1 not found");
    }
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

  const relayClients = new Map<string, WebSocket>();
  const relayAddressByWs = new Map<WebSocket, string>();
  const pendingMessages = new Map<string, Array<{ from: string; type: string; payload: string; ts: number }>>();
  (globalThis as any).__relayClients = relayClients;
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
        if (toWs.readyState === WebSocket.OPEN) {
          const envelope = JSON.stringify({ type: "relay", from: fromAddr, msgType: "mesh-heartbeat", payload: JSON.stringify({ ts: now }) });
          const envBytes = Buffer.byteLength(envelope, "utf-8");
          toWs.send(envelope);
          relayThroughput.bytesRelayed += envBytes;
          relayThroughput.meshHeartbeats++;
          recordRelayMsg("delivered", envBytes);
        }
      }
    }
  }, 45_000);

  const wss = new WebSocketServer({ noServer: true });

  httpServer.on("upgrade", (request, socket, head) => {
    const url = new URL(request.url || "", `http://${request.headers.host}`);
    if (url.pathname === "/ws/relay") {
      wss.handleUpgrade(request, socket, head, (ws) => {
        wss.emit("connection", ws, request);
      });
    } else {
      socket.destroy();
    }
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

  let lastPeerCount = 0;
  const relayPingInterval = setInterval(() => {
    const now = Date.now();
    let pruned = 0;
    for (const [addr, clientWs] of relayClients.entries()) {
      if (clientWs.readyState !== WebSocket.OPEN) {
        console.log(`[ws-relay] Pruning dead socket for ${toDottedAddr(addr)} (readyState=${clientWs.readyState})`);
        relayClients.delete(addr);
        relayAddressByWs.delete(clientWs);
        pruned++;
        continue;
      }
      const entry = crsRegistry.get(addr);
      if (entry && (now - entry.lastSeen) > RELAY_DEAD_TIMEOUT) {
        console.log(`[ws-relay] Node ${toDottedAddr(addr)} unresponsive for ${Math.round((now - entry.lastSeen) / 1000)}s — closing`);
        clientWs.close(1000, "ping timeout");
        relayClients.delete(addr);
        relayAddressByWs.delete(clientWs);
        pruned++;
        continue;
      }
      try {
        clientWs.ping();
      } catch (err: any) {
        console.log(`[ws-relay] Ping failed for ${toDottedAddr(addr)}: ${err.message}`);
        relayClients.delete(addr);
        relayAddressByWs.delete(clientWs);
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

  wss.on("connection", (ws: WebSocket) => {
    let authenticated = false;
    let nodeAddress = "";
    const challengeNonce = crypto.randomBytes(32).toString("hex");

    ws.on("pong", () => {
      if (nodeAddress) {
        const entry = crsRegistry.get(nodeAddress);
        if (entry) entry.lastSeen = Date.now();
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
          if (relayClients.size > relayThroughput.peakPeers) relayThroughput.peakPeers = relayClients.size;
          console.log(`[ws-relay] Node ${toDottedAddr(nodeAddress)} ${isReconnect ? "re" : ""}authenticated and connected`);
          recordRelayAuditEvent({ eventType: "relay.auth_success", address: nodeAddress, timestamp: new Date().toISOString(), details: { hasTlDsa: !!msg.signature, reconnect: isReconnect } });
          if (isReconnect) {
            recordRelayAuditEvent({ eventType: "relay.reconnect", address: nodeAddress, timestamp: new Date().toISOString(), details: { replacedExisting: true } });
          }
          recordDisconnectEvent(nodeAddress, { timestamp: new Date().toISOString(), reason: "connected", code: 0, eventType: "reconnect" });

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
              recordRelayMsg("delivered", envBytes);
            }
            if (relayThroughput.queued >= pending.length) {
              relayThroughput.queued -= pending.length;
            } else {
              relayThroughput.queued = 0;
            }
            console.log(`[ws-relay] Delivered ${pending.length} queued messages to ${toDottedAddr(nodeAddress)}`);
            pendingMessages.delete(nodeAddress);
          }

          ws.send(JSON.stringify({ type: "auth_ok", address: nodeAddress, connectedPeers: Array.from(relayClients.keys()).filter(a => a !== nodeAddress) }));
          return;
        }
        ws.send(JSON.stringify(makeErrorResponse("ERR_NOT_AUTHENTICATED", msg.type)));
        recordRelayAuditEvent({ eventType: "relay.error", address: "unauthenticated", timestamp: new Date().toISOString(), details: { code: "ERR_NOT_AUTHENTICATED", msgType: msg.type } });
        return;
      }

      if (nodeAddress) {
        const entry = crsRegistry.get(nodeAddress);
        if (entry) entry.lastSeen = Date.now();
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
          if (queue.length < 100) {
            queue.push({ from: nodeAddress, type: msg.msgType || "data", payload: msg.payload, ts: Date.now() });
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

      ws.send(JSON.stringify(makeErrorResponse("ERR_UNKNOWN_MSG_TYPE", msg.type)));
      if (nodeAddress) {
        recordRelayAuditEvent({ eventType: "relay.error", address: nodeAddress, timestamp: new Date().toISOString(), details: { code: "ERR_UNKNOWN_MSG_TYPE", msgType: msg.type } });
      }
    });

    ws.on("close", (code: number, reason: Buffer) => {
      if (nodeAddress) {
        relayClients.delete(nodeAddress);
        relayAddressByWs.delete(ws);
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
