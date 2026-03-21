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
import { TsaService, type TsaConfig, TSA_POLICIES, type HptpClient, type TldsaClient } from "./services/tsa-service";
import { createTsaRoutes } from "./routes/tsa";
import { type CalendarServiceClient } from "./services/tsa-calendar-enrichment";
import { keygen, signHex, verifyHex, publicKeyHash, type TlDsaKeyPair } from "./crypto/tl-dsa-bridge";
import * as fs from "fs";
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
  const TOTAL_ADDRESSES = Math.pow(3, DIMENSIONS);
  const crsRegistry = new Map<string, { publicKey: string; endpoint: string; lastSeen: number }>();
  let crsAddressCounter = 1;

  function toTernary(n: number, digits: number): string {
    let result = "";
    let val = n;
    for (let i = 0; i < digits; i++) {
      result = ((val % 3) + 1).toString() + result;
      val = Math.floor(val / 3);
    }
    return result;
  }

  function getNeighbors(addr: string): { address: string; endpoint: string | null; registered: boolean }[] {
    const neighbors: { address: string; endpoint: string | null; registered: boolean }[] = [];
    for (let dim = 0; dim < DIMENSIONS; dim++) {
      const pos = DIMENSIONS - 1 - dim;
      const digit = parseInt(addr[pos]);
      for (const delta of [1, 2]) {
        const newDigit = ((digit - 1 + delta) % 3) + 1;
        const neighborAddr = addr.substring(0, pos) + newDigit.toString() + addr.substring(pos + 1);
        const entry = crsRegistry.get(neighborAddr);
        neighbors.push({
          address: neighborAddr,
          endpoint: entry?.endpoint || null,
          registered: !!entry,
        });
      }
    }
    return neighbors;
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
    const { publicKey, endpoint } = req.query as { publicKey?: string; endpoint?: string };
    if (!publicKey || !endpoint) {
      return res.status(400).json({ error: "publicKey and endpoint query params required" });
    }
    const upstream = await tryCrsDaemon("http://127.0.0.1:8181/api/salvi/inter-cube/crs/register", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ publicKey, endpoint }),
    });
    if (upstream) {
      const body = await upstream.text();
      return res.status(upstream.status).setHeader("Content-Type", upstream.headers.get("content-type") || "application/json").send(body);
    }
    const existingAddr = [...crsRegistry.entries()].find(([_, v]) => v.publicKey === publicKey)?.[0];
    let addr: string;
    if (existingAddr) {
      addr = existingAddr;
      crsRegistry.set(addr, { publicKey, endpoint, lastSeen: Date.now() });
    } else {
      addr = toTernary(crsAddressCounter++, DIMENSIONS);
      crsRegistry.set(addr, { publicKey, endpoint, lastSeen: Date.now() });
    }
    const neighbors = getNeighbors(addr);
    const registeredNeighbors = neighbors.filter((n) => n.registered).length;
    log(`CRS(TS) registered ${publicKey.substring(0, 16)}... as ${addr} (${registeredNeighbors}/${neighbors.length} neighbors)`, "crs");
    res.json({ address: addr, endpoint, neighbors, registeredNeighbors, totalNeighbors: neighbors.length });
  });

  app.get("/api/salvi/inter-cube/relay/heartbeat", async (req, res) => {
    const { address, publicKey } = req.query as { address?: string; publicKey?: string };
    if (!address) {
      return res.status(400).json({ error: "address query param required" });
    }
    const upstream = await tryCrsDaemon("http://127.0.0.1:8181/api/salvi/inter-cube/crs/heartbeat", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ address, publicKey: publicKey || "" }),
    });
    if (upstream) {
      const body = await upstream.text();
      return res.status(upstream.status).setHeader("Content-Type", upstream.headers.get("content-type") || "application/json").send(body);
    }
    const entry = crsRegistry.get(address as string);
    if (entry) {
      entry.lastSeen = Date.now();
      res.json({ status: "ok", address });
    } else {
      res.status(404).json({ error: "Address not registered" });
    }
  });

  const interCubeProxy = async (req: any, res: any) => {
    const upstream = await tryCrsDaemon(`http://127.0.0.1:8181${req.originalUrl}`, {
      method: req.method,
      headers: { "Content-Type": "application/json" },
      ...(req.method !== "GET" && req.method !== "HEAD" && req.body ? { body: JSON.stringify(req.body) } : {}),
    });
    if (upstream) {
      const body = await upstream.text();
      return res.status(upstream.status).setHeader("Content-Type", upstream.headers.get("content-type") || "application/json").send(body);
    }
    if (req.params.service === "crs" && req.params.action === "register") {
      const { publicKey, endpoint: ep } = req.body || {};
      if (!publicKey || !ep) return res.status(400).json({ error: "publicKey and endpoint required" });
      const addr = toTernary(crsAddressCounter++, DIMENSIONS);
      crsRegistry.set(addr, { publicKey, endpoint: ep, lastSeen: Date.now() });
      return res.json({ address: addr, endpoint: ep, neighbors: getNeighbors(addr), registeredNeighbors: 0, totalNeighbors: DIMENSIONS * 2 });
    }
    if (req.params.service === "crs" && req.params.action === "stats") {
      return res.json({ registeredCubes: crsRegistry.size, availableAddresses: TOTAL_ADDRESSES - crsRegistry.size, totalAddressSpace: TOTAL_ADDRESSES, dimensions: DIMENSIONS });
    }
    res.status(503).json({ error: "CRS daemon not available for this endpoint" });
  };

  app.get("/api/salvi/inter-cube/relay/status", (_req, res) => {
    const relayClientsRef = (globalThis as any).__relayClients;
    const pendingRef = (globalThis as any).__pendingMessages;
    const crsReg = crsRegistry;
    if (relayClientsRef) {
      const nodes = Array.from(relayClientsRef.entries()).map(([addr, ws]: [string, any]) => ({
        address: addr,
        connected: ws.readyState === 1,
        endpoint: crsReg.get(addr)?.endpoint || null,
      }));
      return res.json({ connectedNodes: nodes.length, nodes, pendingQueues: pendingRef?.size || 0 });
    }
    res.json({ connectedNodes: 0, nodes: [], pendingQueues: 0 });
  });

  app.all("/api/salvi/inter-cube/:service/:action", interCubeProxy);
  app.all("/api/salvi/inter-cube/:service", interCubeProxy);

  app.get("/health/crs", async (_req, res) => {
    const upstream = await tryCrsDaemon("http://127.0.0.1:8181/health", { method: "GET" });
    if (upstream) {
      const body = await upstream.text();
      return res.status(upstream.status).setHeader("Content-Type", "application/json").send(body);
    }
    res.json({ status: "ok", service: "PlenumNET Inter-Cube Infrastructure (TS fallback)", version: "0.2.0", mode: "crs", address: toTernary(0, DIMENSIONS), registeredCubes: crsRegistry.size });
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

  const relayClients = new Map<string, WebSocket>();
  const relayAddressByWs = new Map<WebSocket, string>();
  const pendingMessages = new Map<string, Array<{ from: string; type: string; payload: string; ts: number }>>();
  (globalThis as any).__relayClients = relayClients;
  (globalThis as any).__pendingMessages = pendingMessages;

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
    try {
      const resp = await fetch(`http://127.0.0.1:8181/api/salvi/inter-cube/crs/node/${address}`);
      if (resp.ok) {
        const data = await resp.json() as any;
        if (data.publicKey === publicKey || data.public_key === publicKey) {
          crsRegistry.set(address, { publicKey, endpoint: data.endpoint || "unknown", lastSeen: Date.now() });
          return true;
        }
      }
    } catch {}
    if (!entry) {
      crsRegistry.set(address, { publicKey, endpoint: "ws-relay", lastSeen: Date.now() });
      return true;
    }
    return false;
  }

  wss.on("connection", (ws: WebSocket) => {
    let authenticated = false;
    let nodeAddress = "";

    ws.on("message", async (data: Buffer) => {
      let msg: any;
      try {
        msg = JSON.parse(data.toString());
      } catch {
        ws.send(JSON.stringify({ error: "invalid JSON" }));
        return;
      }

      if (!authenticated) {
        if (msg.type === "auth" && msg.address && msg.publicKey) {
          const verified = await verifyNodeRegistration(msg.address, msg.publicKey);
          if (verified) {
            authenticated = true;
            nodeAddress = msg.address;
            const oldWs = relayClients.get(nodeAddress);
            if (oldWs && oldWs !== ws && oldWs.readyState === WebSocket.OPEN) {
              oldWs.close(1000, "replaced");
            }
            relayClients.set(nodeAddress, ws);
            relayAddressByWs.set(ws, nodeAddress);
            console.log(`[ws-relay] Node ${nodeAddress} authenticated and connected`);

            const pending = pendingMessages.get(nodeAddress);
            if (pending && pending.length > 0) {
              for (const queued of pending) {
                ws.send(JSON.stringify({ type: "relay", from: queued.from, msgType: queued.type, payload: queued.payload }));
              }
              console.log(`[ws-relay] Delivered ${pending.length} queued messages to ${nodeAddress}`);
              pendingMessages.delete(nodeAddress);
            }

            ws.send(JSON.stringify({ type: "auth_ok", address: nodeAddress, connectedPeers: Array.from(relayClients.keys()).filter(a => a !== nodeAddress) }));
          } else {
            ws.send(JSON.stringify({ type: "auth_fail", error: "address not registered or publicKey mismatch — register with CRS first" }));
            ws.close(1008, "auth failed");
          }
          return;
        }
        ws.send(JSON.stringify({ error: "must authenticate first: {type:'auth', address:'...', publicKey:'...'}" }));
        return;
      }

      if (msg.type === "relay" && msg.to && msg.payload) {
        const targetWs = relayClients.get(msg.to);
        const envelope = JSON.stringify({ type: "relay", from: nodeAddress, msgType: msg.msgType || "data", payload: msg.payload });
        if (targetWs && targetWs.readyState === WebSocket.OPEN) {
          targetWs.send(envelope);
        } else {
          if (!pendingMessages.has(msg.to)) pendingMessages.set(msg.to, []);
          const queue = pendingMessages.get(msg.to)!;
          if (queue.length < 100) {
            queue.push({ from: nodeAddress, type: msg.msgType || "data", payload: msg.payload, ts: Date.now() });
          }
        }
        ws.send(JSON.stringify({ type: "relay_ack", to: msg.to, delivered: !!(targetWs && targetWs.readyState === WebSocket.OPEN) }));
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

      ws.send(JSON.stringify({ error: "unknown message type", validTypes: ["relay", "ping", "peers"] }));
    });

    ws.on("close", () => {
      if (nodeAddress) {
        relayClients.delete(nodeAddress);
        relayAddressByWs.delete(ws);
        console.log(`[ws-relay] Node ${nodeAddress} disconnected`);
      }
    });

    ws.on("error", (err: Error) => {
      console.log(`[ws-relay] Error for ${nodeAddress || "unauthenticated"}: ${err.message}`);
    });

    setTimeout(() => {
      if (!authenticated && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ error: "auth timeout" }));
        ws.close(1008, "auth timeout");
      }
    }, 10000);
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
