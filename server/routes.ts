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

import type { Express } from "express";
import express from "express";
import { createServer, type Server } from "http";
import { storage } from "./storage";
import { randomUUID } from "crypto";
import { z } from "zod";
import { registerGitHubRoutes } from "./routes/github";
import { registerKongRoutes } from "./routes/kong";
import { registerDataSubjectRightsRoutes } from "./routes/data-subject-rights";
import { registerSalviRoutes } from "./routes/salvi";
import { registerTribonacciRoutes } from "./routes/tribonacci";
import { registerAgentArrayRoutes } from "./routes/agent-array";
import { registerApiKeyRoutes } from "./routes/api-keys";
import { registerSecurityRoutes } from "./routes/security";
import { registerEphemerisRoutes } from "./routes/ephemeris";
import { registerTonalFieldRoutes } from "./routes/tonal-field";
import { registerPPTProIntegrationRoutes } from "./routes/pptpro-integration";
import { registerPqtiRoutes } from "./routes/pqti";
import { registerTdnsRoutes } from "./routes/tdns";
import { registerCapabilityRoutes } from "./routes/capabilities";
import { registerInterCubeRoutes } from "./routes/inter-cube";
import { apiKeyService } from "./services/api-key.service";
import { readFile } from "fs/promises";
import { existsSync } from "fs";
import * as path from "path";
let _exceljs: typeof import("exceljs") | null = null;
async function getExcelJS() {
  if (!_exceljs) _exceljs = await import("exceljs");
  return _exceljs;
}
import { 
  compressData,
  decompressData,
  generateSensorData, 
  generateUserEvents, 
  generateLogEntries 
} from "./ternary";
import { insertDeveloperSignupSchema, insertWhitepaperSchema } from "@shared/schema";
import { setupAuth, registerAuthRoutes, isAuthenticated } from "./replit_integrations/auth";
import { createLogger, toErrorMessage } from "./logger";
import { db } from "./db";
import { sql } from "drizzle-orm";
const log = createLogger("routes");

const GIT_COMMIT_HASH = process.env.GIT_COMMIT || (() => {
  try {
    const { execFileSync } = require("child_process");
    return execFileSync("git", ["rev-parse", "--short", "HEAD"], { encoding: "utf-8", timeout: 3000 }).trim();
  } catch {
    return "unknown";
  }
})();

const SERVER_START_TIME = Date.now();

const demoRunSchema = z.object({
  datasetName: z.enum(["sensor", "events", "logs"]),
  rowCount: z.number().int().min(1).max(10000).default(100)
});

export async function registerRoutes(
  httpServer: Server,
  app: Express
): Promise<Server> {

  app.get("/download/maps/:filename", (req, res) => {
    const allowed = new Set(["aasc_canonical_map.png", "aasc_canonical_map.svg"]);
    const filename = req.params.filename;
    if (!allowed.has(filename)) {
      res.status(404).send("Not found");
      return;
    }
    const filePath = path.resolve(process.cwd(), "client", "public", "maps", filename);
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0");
    res.setHeader("Pragma", "no-cache");
    res.setHeader("Expires", "0");
    res.download(filePath, filename, (err) => {
      if (err && !res.headersSent) res.status(500).send("Download failed");
    });
  });

  app.get("/download/maps", (_req, res) => {
    const v = Date.now();
    const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
<title>aasc Canonical Map · Downloads</title>
<style>
  :root { color-scheme: light dark; }
  body { font-family: -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif;
         background: #0F0C0A; color: #FAF8F6; margin: 0; padding: 48px 24px; }
  .wrap { max-width: 920px; margin: 0 auto; }
  h1 { font-size: 28px; margin: 0 0 8px; color: #4A9EF5; }
  .sub { color: #a8b4c0; margin-bottom: 32px; font-size: 15px; }
  .card { background: #1a1612; border: 1px solid #2D7DD2; border-radius: 12px;
          padding: 24px; margin-bottom: 16px; }
  .card h2 { margin: 0 0 8px; font-size: 20px; color: #FAF8F6; }
  .card .meta { color: #78828C; font-size: 13px; margin-bottom: 16px; font-family: Menlo, Consolas, monospace; }
  .btn { display: inline-block; padding: 12px 24px; margin-right: 12px;
         background: #2D7DD2; color: #FAF8F6; text-decoration: none;
         border-radius: 8px; font-weight: 600; font-size: 15px; }
  .btn:hover { background: #4A9EF5; }
  .btn.secondary { background: transparent; border: 1px solid #2D7DD2; color: #4A9EF5; }
  .btn.secondary:hover { background: #2D7DD2; color: #FAF8F6; }
  .preview { margin-top: 32px; }
  .preview img { width: 100%; height: auto; border: 1px solid #3D444B; border-radius: 8px; }
  code { background: #2a221d; padding: 2px 6px; border-radius: 4px;
         font-family: Menlo, Consolas, monospace; font-size: 13px; color: #4A9EF5; }
</style>
</head>
<body>
<div class="wrap">
  <h1>aasc Canonical Map</h1>
  <div class="sub">algeometric-arc-sigma182-calculi · 9-layer ring · 57 canonical modules · cache-busted v=${v}</div>

  <div class="card">
    <h2>PNG (raster, ~5.4 MB)</h2>
    <div class="meta">aasc_canonical_map.png · viewBox 7400 × 10040 · density 80 dpi</div>
    <a class="btn" href="/download/maps/aasc_canonical_map.png?v=${v}" download>Download PNG</a>
    <a class="btn secondary" href="/download/maps/aasc_canonical_map.png?v=${v}" target="_blank">Open in new tab</a>
  </div>

  <div class="card">
    <h2>SVG (vector, ~75 KB)</h2>
    <div class="meta">aasc_canonical_map.svg · scalable, editable, includes all panels</div>
    <a class="btn" href="/download/maps/aasc_canonical_map.svg?v=${v}" download>Download SVG</a>
    <a class="btn secondary" href="/download/maps/aasc_canonical_map.svg?v=${v}" target="_blank">Open in new tab</a>
  </div>

  <div class="card">
    <h2>Inline preview</h2>
    <div class="meta">live render of the current map</div>
    <div class="preview">
      <img src="/download/maps/aasc_canonical_map.png?v=${v}" alt="aasc canonical map preview"/>
    </div>
  </div>
</div>
</body>
</html>`;
    res.setHeader("Content-Type", "text/html; charset=utf-8");
    res.setHeader("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0");
    res.send(html);
  });

  app.get("/api/benchmark-report", async (_req, res) => {
    try {
      const { readdir } = await import("fs/promises");
      const benchDir = path.resolve("ternary-math");
      const all = await readdir(benchDir);
      const files = all.filter(f => f.startsWith("BENCH-") && f.endsWith(".html")).sort();
      if (files.length > 0) {
        const latest = files[files.length - 1];
        const html = await readFile(path.join(benchDir, latest), "utf-8");
        res.type("html").send(html);
      } else {
        res.status(404).send("No benchmark report found");
      }
    } catch (e: any) {
      res.status(500).send("Error: " + (e?.message || "unknown"));
    }
  });

  // =====================================================
  // PPTPro INTEGRATION API — must register before v1 rewrite
  // =====================================================
  registerPPTProIntegrationRoutes(app);

  // PQTI Service Proxy — forwards to Rust microservice on port 3001
  // =====================================================
  registerPqtiRoutes(app);

  // TDNS Proxy — forwards to TDNS microservice on port 3927
  // =====================================================
  registerTdnsRoutes(app);

  app.use((req, _res, next) => {
    if (req.path.startsWith("/api/v1/")) {
      req.url = req.url.replace("/api/v1/", "/api/");
    }
    next();
  });

  app.get("/.well-known/security.txt", (_req, res) => {
    res.type("text/plain").send(
      `Contact: mailto:security@capomastroholdings.com\n` +
      `Contact: mailto:RSalvi@Salvigroup.com\n` +
      `Expires: 2027-01-01T00:00:00.000Z\n` +
      `Preferred-Languages: en\n` +
      `Canonical: https://plenumnet.replit.app/.well-known/security.txt\n` +
      `Policy: https://plenumnet.replit.app/security\n` +
      `Encryption: https://plenumnet.replit.app/api/tdns/health\n`
    );
  });

  app.post("/api/csp-reports", (req, res) => {
    res.status(204).end();
  });

  const legalDocMap: Record<string, { file: string; title: string }> = {
    terms: { file: "TERMS-OF-SERVICE.md", title: "Terms of Service" },
    privacy: { file: "PRIVACY-POLICY.md", title: "Privacy Policy" },
    security: { file: "SECURITY.md", title: "Security Policy" },
    aup: { file: "ACCEPTABLE-USE-POLICY.md", title: "Acceptable Use Policy" },
    "export-control": { file: "EXPORT-CONTROL.md", title: "Export Control Classification" },
    "ip-notice": { file: "IP-NOTICE.md", title: "Intellectual Property Notice" },
  };

  const plmRecords: Record<string, { target: string; addr: string; zone: string; ttl: number; description: string }> = {
    "google.plm": {
      target: "https://google.com",
      addr: "1-2-3-1-2-3-1-2-3-1-2-3-1",
      zone: "plm",
      ttl: 3600,
      description: "Google search engine",
    },
    "wikipedia.plm": {
      target: "https://en.wikipedia.org",
      addr: "2-1-3-2-1-3-2-1-3-2-1-3-2",
      zone: "plm",
      ttl: 3600,
      description: "Wikipedia",
    },
    "github.plm": {
      target: "https://github.com/SigmaWolf-8/Ternary",
      addr: "3-1-2-3-1-2-3-1-2-3-1-2-3",
      zone: "plm",
      ttl: 3600,
      description: "PlenumNET GitHub repository",
    },
    "plenumnet.plm": {
      target: "https://plenumnet.replit.app",
      addr: "1-1-1-1-1-1-1-1-1-1-1-1-1",
      zone: "plm",
      ttl: 3600,
      description: "PlenumNET platform",
    },
    "docs.plenumnet.plm": {
      target: "https://plenumnet.replit.app/docs",
      addr: "1-1-1-1-1-1-1-1-1-1-1-1-2",
      zone: "plenumnet.plm",
      ttl: 3600,
      description: "PlenumNET documentation",
    },
    "auction.plm": {
      target: "https://plenumnet.replit.app/ternarydb",
      addr: "2-2-1-3-1-2-3-1-2-3-1-2-1",
      zone: "plm",
      ttl: 3600,
      description: "TDNS name auction",
    },
  };

  app.get("/api/tdns/resolve", (req, res) => {
    const name = (req.query.name as string || "").toLowerCase().trim();
    const redirect = req.query.redirect === "1";

    if (!name) {
      return res.status(400).json({ error: "Missing ?name= parameter" });
    }

    const record = plmRecords[name];
    if (record) {
      if (redirect) {
        return res.redirect(302, record.target);
      }
      return res.json({
        name,
        target: record.target,
        addr: record.addr,
        zone: record.zone,
        ttl: record.ttl,
        description: record.description,
        resolved: true,
      });
    }

    if (redirect) {
      return res.redirect(302, `https://plenumnet.replit.app/?plm_not_found=${encodeURIComponent(name)}`);
    }
    return res.status(404).json({ name, resolved: false, error: "Name not found in TDNS" });
  });

  app.get("/api/tdns/records", (_req, res) => {
    const records = Object.entries(plmRecords).map(([name, r]) => ({
      name,
      target: r.target,
      addr: r.addr,
      zone: r.zone,
      ttl: r.ttl,
      description: r.description,
    }));
    res.json({ count: records.length, records });
  });

  app.get("/api/health", async (_req, res) => {
    let dbStatus = "error";
    try {
      const result = await db.execute(sql`SELECT 1`);
      if (result.rows.length > 0) {
        dbStatus = "connected";
      }
    } catch {}

    const isHealthy = dbStatus === "connected";
    res.status(isHealthy ? 200 : 503).json({
      status: isHealthy ? "healthy" : "degraded",
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      version: "1.0.0",
      commit: GIT_COMMIT_HASH,
      startedAt: new Date(SERVER_START_TIME).toISOString(),
      services: {
        database: dbStatus,
        server: "running"
      }
    });
  });

  const { requireApiKey } = await import("./routes/middleware");

  app.get("/api/verify", requireApiKey, (_req: any, res: any) => {
    res.json({
      status: "authenticated",
      service: "PlenumNET",
      version: "1.0.0",
      timestamp: new Date().toISOString(),
    });
  });

  app.get("/api/legal/:type", async (req, res) => {
    const docInfo = legalDocMap[req.params.type];
    if (!docInfo) {
      return res.status(404).json({ error: "Document not found" });
    }
    try {
      const filePath = path.join(process.cwd(), docInfo.file);
      const content = await readFile(filePath, "utf-8");
      res.json({ title: docInfo.title, content });
    } catch (err: unknown) {
      log.error("Legal doc read error:", err);
      res.status(500).json({ error: "Failed to read document" });
    }
  });

  await setupAuth(app);
  registerAuthRoutes(app);
  
  app.post("/api/demo/run", async (req, res) => {
    try {
      const parsed = demoRunSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { datasetName, rowCount } = parsed.data;
      const startTime = Date.now();
      const sessionId = randomUUID();
      
      let rawData: object[];
      switch (datasetName) {
        case "sensor":
          rawData = generateSensorData(rowCount);
          break;
        case "events":
          rawData = generateUserEvents(rowCount);
          break;
        case "logs":
          rawData = generateLogEntries(rowCount);
          break;
      }
      
      const jsonString = JSON.stringify(rawData);
      const binarySizeBytes = Buffer.from(jsonString, 'utf-8').length;
      
      const compression = compressData(jsonString);
      
      await storage.createDemoSession({
        sessionId,
        datasetName
      });
      
      const binaryRecord = await storage.createBinaryStorage({
        sessionId,
        dataType: datasetName,
        rawData,
        sizeBytes: binarySizeBytes,
        rowCount
      });
      
      const ternaryRecord = await storage.createTernaryStorage({
        sessionId,
        dataType: datasetName,
        compressedData: compression.compressedData,
        originalSizeBytes: compression.originalSize,
        compressedSizeBytes: compression.compressedSize,
        compressionRatio: compression.compressionRatio,
        rowCount
      });
      
      const processingTimeMs = Date.now() - startTime;
      const savingsPercent = ((binarySizeBytes - compression.compressedSize) / binarySizeBytes) * 100;
      
      await storage.createCompressionBenchmark({
        sessionId,
        datasetName,
        binaryStorageId: binaryRecord.id,
        ternaryStorageId: ternaryRecord.id,
        binarySizeBytes,
        ternarySizeBytes: compression.compressedSize,
        savingsPercent,
        processingTimeMs
      });
      
      await storage.createCompressionHistory({
        sessionId,
        datasetName,
        sourceType: "sample_dataset",
        binarySizeBytes,
        ternarySizeBytes: compression.compressedSize,
        savingsPercent,
        rowCount,
        processingTimeMs
      });
      
      res.json({
        success: true,
        sessionId,
        datasetName,
        rowCount,
        binarySize: binarySizeBytes,
        ternarySize: compression.compressedSize,
        savingsPercent: savingsPercent.toFixed(1),
        processingTimeMs,
        preview: rawData.slice(0, 5),
        atScaleProjection: {
          binarySize: binarySizeBytes * 10000,
          ternarySize: compression.compressedSize * 10000,
          savings: `${(binarySizeBytes * 10000 - compression.compressedSize * 10000) / (1024 * 1024)} MB`
        }
      });
    } catch (error: unknown) {
      log.error("Demo run error:", error);
      res.status(500).json({ error: "Failed to run demo" });
    }
  });
  
  app.get("/api/demo/stats", async (req, res) => {
    try {
      const benchmarks = await storage.getRecentBenchmarks(100);
      
      const totalRuns = benchmarks.length;
      const avgSavings = benchmarks.length > 0
        ? benchmarks.reduce((sum, b) => sum + b.savingsPercent, 0) / benchmarks.length
        : 0;
      const totalDataProcessed = benchmarks.reduce((sum, b) => sum + b.binarySizeBytes, 0);
      const totalSavings = benchmarks.reduce((sum, b) => sum + (b.binarySizeBytes - b.ternarySizeBytes), 0);
      
      res.json({
        totalRuns,
        avgSavings: avgSavings.toFixed(1),
        totalDataProcessed,
        totalSavings,
        recentBenchmarks: benchmarks.slice(0, 10)
      });
    } catch (error: unknown) {
      log.error("Stats error:", error);
      res.status(500).json({ error: "Failed to get stats" });
    }
  });
  
  app.get("/api/demo/session/:sessionId", async (req, res) => {
    try {
      const { sessionId } = req.params;
      
      const session = await storage.getDemoSession(sessionId);
      if (!session) {
        return res.status(404).json({ error: "Session not found" });
      }
      
      const binaryData = await storage.getBinaryStorage(sessionId);
      const ternaryData = await storage.getTernaryStorage(sessionId);
      const benchmarks = await storage.getCompressionBenchmarks(sessionId);
      
      res.json({
        session,
        binaryData,
        ternaryData,
        benchmarks
      });
    } catch (error: unknown) {
      log.error("Session error:", error);
      res.status(500).json({ error: "Failed to get session" });
    }
  });

  app.post("/api/demo/upload", async (req, res) => {
    try {
      const uploadSchema = z.object({
        fileName: z.string().min(1),
        fileType: z.enum(["json", "csv", "xlsx"]),
        content: z.string().min(1),
      });
      
      const parsed = uploadSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      
      const { fileName, fileType, content } = parsed.data;
      const startTime = Date.now();
      const sessionId = randomUUID();
      
      let rawData: object[];
      try {
        if (fileType === "json") {
          const parsed = JSON.parse(content);
          rawData = Array.isArray(parsed) ? parsed : [parsed];
        } else if (fileType === "xlsx") {
          const ExcelJS = await getExcelJS();
          const binaryData = Buffer.from(content, 'base64');
          const workbook = new ExcelJS.Workbook();
          await workbook.xlsx.load(binaryData);
          const worksheet = workbook.worksheets[0];
          if (!worksheet) {
            return res.status(400).json({ error: "Excel file has no sheets" });
          }
          const headers: string[] = [];
          const firstRow = worksheet.getRow(1);
          firstRow.eachCell((cell, colNumber) => {
            headers[colNumber - 1] = String(cell.value ?? '');
          });
          if (headers.length === 0) {
            return res.status(400).json({ error: "Excel sheet is empty or has no data rows" });
          }
          rawData = [];
          worksheet.eachRow((row, rowNumber) => {
            if (rowNumber === 1) return;
            const rowObj: Record<string, unknown> = {};
            row.eachCell((cell, colNumber) => {
              const header = headers[colNumber - 1];
              if (header) rowObj[header] = cell.value;
            });
            rawData.push(rowObj);
          });
          if (rawData.length === 0) {
            return res.status(400).json({ error: "Excel sheet is empty or has no data rows" });
          }
        } else {
          const lines = content.trim().split('\n');
          if (lines.length < 2) {
            return res.status(400).json({ error: "CSV must have header and at least one data row" });
          }
          const headers = lines[0].split(',').map(h => h.trim().replace(/^"|"$/g, ''));
          rawData = lines.slice(1).map(line => {
            const values = line.split(',').map(v => v.trim().replace(/^"|"$/g, ''));
            const row: Record<string, string> = {};
            headers.forEach((header, i) => {
              row[header] = values[i] || '';
            });
            return row;
          });
        }
      } catch (parseError) {
        return res.status(400).json({ error: "Failed to parse file content" });
      }
      
      const rowCount = rawData.length;
      const jsonString = JSON.stringify(rawData);
      const binarySizeBytes = Buffer.from(jsonString, 'utf-8').length;
      
      const compression = compressData(jsonString);
      
      await storage.createDemoSession({
        sessionId,
        datasetName: `upload:${fileName}`
      });
      
      await storage.createFileUpload({
        sessionId,
        fileName,
        fileType,
        originalSizeBytes: binarySizeBytes,
        rowCount
      });
      
      const binaryRecord = await storage.createBinaryStorage({
        sessionId,
        dataType: "file_upload",
        rawData,
        sizeBytes: binarySizeBytes,
        rowCount
      });
      
      const ternaryRecord = await storage.createTernaryStorage({
        sessionId,
        dataType: "file_upload",
        compressedData: compression.compressedData,
        originalSizeBytes: compression.originalSize,
        compressedSizeBytes: compression.compressedSize,
        compressionRatio: compression.compressionRatio,
        rowCount
      });
      
      const processingTimeMs = Date.now() - startTime;
      const savingsPercent = ((binarySizeBytes - compression.compressedSize) / binarySizeBytes) * 100;
      
      await storage.createCompressionBenchmark({
        sessionId,
        datasetName: `upload:${fileName}`,
        binaryStorageId: binaryRecord.id,
        ternaryStorageId: ternaryRecord.id,
        binarySizeBytes,
        ternarySizeBytes: compression.compressedSize,
        savingsPercent,
        processingTimeMs
      });
      
      await storage.createCompressionHistory({
        sessionId,
        datasetName: fileName,
        sourceType: "file_upload",
        binarySizeBytes,
        ternarySizeBytes: compression.compressedSize,
        savingsPercent,
        rowCount,
        processingTimeMs
      });
      
      res.json({
        success: true,
        sessionId,
        fileName,
        fileType,
        rowCount,
        binarySize: binarySizeBytes,
        ternarySize: compression.compressedSize,
        savingsPercent: savingsPercent.toFixed(1),
        processingTimeMs,
        preview: rawData.slice(0, 5),
        atScaleProjection: {
          binarySize: binarySizeBytes * 10000,
          ternarySize: compression.compressedSize * 10000,
          savings: `${((binarySizeBytes * 10000 - compression.compressedSize * 10000) / (1024 * 1024)).toFixed(2)} MB`
        }
      });
    } catch (error: unknown) {
      log.error("Upload error:", error);
      res.status(500).json({ error: "Failed to process upload" });
    }
  });

  app.get("/api/demo/history", async (req, res) => {
    try {
      const history = await storage.getCompressionHistory(50);
      res.json({
        success: true,
        history
      });
    } catch (error: unknown) {
      log.error("History error:", error);
      res.status(500).json({ error: "Failed to get compression history" });
    }
  });

  app.get("/api/demo/files", async (req, res) => {
    try {
      const files = await storage.getAllFileUploads();
      res.json({
        success: true,
        files
      });
    } catch (error: unknown) {
      log.error("Files error:", error);
      res.status(500).json({ error: "Failed to get files" });
    }
  });

  // ==========================================
  // STANDALONE FILE COMPRESSION API
  // ==========================================
  
  app.post("/api/compression/file",
    (req, res, next) => {
      const ct = (req.headers['content-type'] || '').toLowerCase();
      if (ct.includes('application/octet-stream')) {
        express.raw({ type: 'application/octet-stream', limit: '50mb' })(req, res, next);
      } else {
        next();
      }
    },
    async (req, res) => {
    try {
      const ct = (req.headers['content-type'] || '').toLowerCase();
      const isRaw = ct.includes('application/octet-stream') && Buffer.isBuffer(req.body) && req.body.length > 0;

      let inputBuffer: Buffer;
      let fileName: string;
      let encrypt: boolean;
      let encryptionMode: string;
      let imageWidth: number | undefined;

      let ttcLevel: number | undefined;
      let ttcMode: string | undefined;

      if (isRaw) {
        inputBuffer = req.body as Buffer;
        try { fileName = decodeURIComponent((req.headers['x-ttc-filename'] as string) || 'upload.bin'); } catch { fileName = (req.headers['x-ttc-filename'] as string) || 'upload.bin'; }
        encrypt = req.headers['x-ttc-encrypt'] === 'true';
        encryptionMode = (req.headers['x-ttc-encryption-mode'] as string) || 'balanced';
        imageWidth = req.headers['x-ttc-image-width'] ? parseInt(req.headers['x-ttc-image-width'] as string, 10) : undefined;
        const rawLevel = req.headers['x-ttc-level'] as string | undefined;
        if (rawLevel) { const n = parseInt(rawLevel, 10); if (n >= 1 && n <= 9) ttcLevel = n; }
        ttcMode = (req.headers['x-ttc-compress-mode'] as string) || undefined;
      } else {
        const schema = z.object({
          fileName: z.string().min(1),
          content: z.string().min(1),
          encrypt: z.boolean().optional().default(false),
          encryptionMode: z.enum(["high_security", "balanced", "performance", "adaptive"]).optional().default("balanced"),
          level: z.number().int().min(1).max(9).optional(),
          mode: z.string().optional(),
        });

        const parsed = schema.safeParse(req.body);
        if (!parsed.success) {
          return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
        }

        fileName = parsed.data.fileName;
        encrypt = parsed.data.encrypt;
        encryptionMode = parsed.data.encryptionMode;
        inputBuffer = Buffer.from(parsed.data.content, 'base64');
        imageWidth = undefined;
        ttcLevel = parsed.data.level;
        ttcMode = parsed.data.mode;
      }

      const { createTernFile } = await import("./compression-layer");
      const startTime = performance.now();
      
      const encMode = encrypt ? (encryptionMode as 'high_security' | 'balanced' | 'performance' | 'adaptive') : undefined;
      const { ternFile, header, ttcMetadata } = createTernFile(inputBuffer, fileName, {
        encrypt,
        encryptionMode: encMode,
        level: ttcLevel,
        mode: ttcMode,
      });
      
      const processingTimeMs = performance.now() - startTime;

      const ttcHeaders: Record<string, string> = {
        'Content-Type': 'application/octet-stream',
        'Content-Disposition': `attachment; filename*=UTF-8''${encodeURIComponent(fileName.replace(/\.[^.]+$/, '') + '.tern')}`,
        'X-TTC-Original-Size': String(header.originalSize),
        'X-TTC-Compressed-Size': String(ternFile.length),
        'X-TTC-Compression-Ratio': header.compressionRatio.toFixed(1),
        'X-TTC-Engine': ttcMetadata?.engine || 'ttc-ts-fallback',
        'X-TTC-Mode': ttcMetadata?.modeName || 'BASIC',
        'X-TTC-Level': String(ttcMetadata?.level ?? 5),
        'X-TTC-Level-Name': ttcMetadata?.levelName || '',
        'X-TTC-Version': ttcMetadata?.version || '1.0',
        'X-TTC-CRC32': String(ttcMetadata?.crc32 ?? header.checksum),
        'X-TTC-Encrypted': String(header.encrypted || false),
        'X-TTC-Processing-Ms': processingTimeMs.toFixed(2),
        'X-TTC-Original-Filename': encodeURIComponent(fileName),
        'X-TTC-Predominant-Base': String(ttcMetadata?.predominantBase ?? 3),
        'X-TTC-Avg-Tau': String(ttcMetadata?.avgTau ?? 0),
        'X-TTC-Avg-Delta': String(ttcMetadata?.avgDelta ?? 0),
        'X-TTC-Adaptive-Rep': String(ttcMetadata?.adaptiveRepUsed ?? false),
        'X-TTC-GF3-Rep': ttcMetadata?.gf3Representation || 'balanced',
      };

      if (isRaw) {
        res.set(ttcHeaders);
        res.send(ternFile);
      } else {
        res.json({
          success: true,
          fileName: fileName.replace(/\.[^.]+$/, '') + '.tern',
          originalSize: header.originalSize,
          compressedSize: ternFile.length,
          compressionRatio: header.compressionRatio.toFixed(1),
          encrypted: header.encrypted,
          encryptionMode: header.encryptionMode,
          processingTimeMs: processingTimeMs.toFixed(2),
          data: ternFile.toString('base64'),
          header,
          ttcMetadata: ttcMetadata || null,
        });
      }
    } catch (error: unknown) {
      log.error("File compression error:", error);
      res.status(500).json({ error: "Compression failed", details: toErrorMessage(error) });
    }
  });

  app.post("/api/compression/decompress",
    (req, res, next) => {
      const ct = (req.headers['content-type'] || '').toLowerCase();
      if (ct.includes('application/octet-stream')) {
        express.raw({ type: 'application/octet-stream', limit: '50mb' })(req, res, next);
      } else {
        next();
      }
    },
    async (req, res) => {
    try {
      const ct = (req.headers['content-type'] || '').toLowerCase();
      const isRaw = ct.includes('application/octet-stream') && Buffer.isBuffer(req.body) && req.body.length > 0;

      let ternBuffer: Buffer;

      if (isRaw) {
        ternBuffer = req.body as Buffer;
      } else {
        const schema = z.object({
          content: z.string().min(1),
        });

        const parsed = schema.safeParse(req.body);
        if (!parsed.success) {
          return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
        }

        ternBuffer = Buffer.from(parsed.data.content, 'base64');
      }

      const { parseTernFile } = await import("./compression-layer");
      const startTime = performance.now();
      
      const { header, originalData, ttcMetadata } = parseTernFile(ternBuffer);
      const processingTimeMs = performance.now() - startTime;

      if (isRaw) {
        const decompHeaders: Record<string, string> = {
          'Content-Type': 'application/octet-stream',
          'Content-Disposition': `attachment; filename*=UTF-8''${encodeURIComponent(header.originalFileName || 'decompressed.bin')}`,
          'X-TTC-Original-Size': String(header.originalSize),
          'X-TTC-Compressed-Size': String(header.compressedSize),
          'X-TTC-Compression-Ratio': header.compressionRatio?.toFixed(1) || '',
          'X-TTC-Engine': ttcMetadata?.engine || 'ttc-ts-fallback',
          'X-TTC-Was-Encrypted': String(header.encrypted || false),
          'X-TTC-Original-Filename': encodeURIComponent(header.originalFileName || ''),
          'X-TTC-CRC32-Verified': String(ttcMetadata?.crc32Verified ?? true),
          'X-TTC-Version': ttcMetadata?.version || '1.0',
          'X-TTC-Level': String(ttcMetadata?.level ?? 5),
          'X-TTC-Level-Name': ttcMetadata?.levelName || '',
          'X-TTC-Processing-Ms': processingTimeMs.toFixed(2),
          'X-TTC-GF3-Rep': ttcMetadata?.gf3Representation || 'balanced',
        };
        res.set(decompHeaders);
        res.send(originalData);
      } else {
        res.json({
          success: true,
          originalFileName: header.originalFileName,
          originalSize: header.originalSize,
          compressedSize: header.compressedSize,
          wasEncrypted: header.encrypted,
          encryptionMode: header.encryptionMode,
          processingTimeMs: processingTimeMs.toFixed(2),
          data: originalData.toString('base64'),
          header,
          ttcMetadata: ttcMetadata || null,
        });
      }
    } catch (error: unknown) {
      log.error("File decompression error:", error);
      res.status(500).json({ error: "Decompression failed", details: toErrorMessage(error) });
    }
  });

  // ==========================================
  // RAW BINARY TRANSPORT — application/octet-stream
  // Config via X-TTC-* request headers, results in X-TTC-* response headers
  // ==========================================

  const rawBodyParser = express.raw({ type: 'application/octet-stream', limit: '50mb' });

  app.post("/api/compression/file/raw", rawBodyParser, async (req, res) => {
    try {
      const body = req.body as Buffer;
      if (!body || !Buffer.isBuffer(body) || body.length === 0) {
        return res.status(400).json({ error: "Empty or invalid body. Send raw bytes with Content-Type: application/octet-stream" });
      }

      let fileName: string;
      try { fileName = decodeURIComponent((req.headers['x-ttc-filename'] as string) || 'upload.bin'); } catch { fileName = (req.headers['x-ttc-filename'] as string) || 'upload.bin'; }
      const encrypt = req.headers['x-ttc-encrypt'] === 'true';
      const encryptionMode = (req.headers['x-ttc-encryption-mode'] as string) || 'balanced';
      const imageWidth = req.headers['x-ttc-image-width'] ? parseInt(req.headers['x-ttc-image-width'] as string, 10) : undefined;
      let ttcLevel: number | undefined;
      const rawLevel = req.headers['x-ttc-level'] as string | undefined;
      if (rawLevel) { const n = parseInt(rawLevel, 10); if (n >= 1 && n <= 9) ttcLevel = n; }
      const ttcMode = (req.headers['x-ttc-compress-mode'] as string) || undefined;

      const { createTernFile } = await import("./compression-layer");
      const startTime = performance.now();

      const encMode = encrypt ? (encryptionMode as 'high_security' | 'balanced' | 'performance' | 'adaptive') : undefined;
      const { ternFile, header, ttcMetadata } = createTernFile(body, fileName, {
        encrypt,
        encryptionMode: encMode,
        level: ttcLevel,
        mode: ttcMode,
      });

      const processingTimeMs = performance.now() - startTime;

      res.set({
        'Content-Type': 'application/octet-stream',
        'Content-Disposition': `attachment; filename*=UTF-8''${encodeURIComponent(fileName.replace(/\.[^.]+$/, '') + '.tern')}`,
        'X-TTC-Original-Size': String(header.originalSize),
        'X-TTC-Compressed-Size': String(ternFile.length),
        'X-TTC-Compression-Ratio': header.compressionRatio.toFixed(1),
        'X-TTC-Engine': ttcMetadata?.engine || 'ttc-ts-fallback',
        'X-TTC-Mode': ttcMetadata?.modeName || 'BASIC',
        'X-TTC-Level': String(ttcMetadata?.level ?? 5),
        'X-TTC-Level-Name': ttcMetadata?.levelName || '',
        'X-TTC-Version': ttcMetadata?.version || '1.0',
        'X-TTC-CRC32': String(ttcMetadata?.crc32 ?? header.checksum),
        'X-TTC-Encrypted': String(header.encrypted || false),
        'X-TTC-Processing-Ms': processingTimeMs.toFixed(2),
        'X-TTC-Original-Filename': encodeURIComponent(fileName),
        'X-TTC-Predominant-Base': String(ttcMetadata?.predominantBase ?? 3),
        'X-TTC-Avg-Tau': String(ttcMetadata?.avgTau ?? 0),
        'X-TTC-Avg-Delta': String(ttcMetadata?.avgDelta ?? 0),
        'X-TTC-Adaptive-Rep': String(ttcMetadata?.adaptiveRepUsed ?? false),
        'X-TTC-GF3-Rep': ttcMetadata?.gf3Representation || 'balanced',
      });

      res.send(ternFile);
    } catch (error: unknown) {
      log.error("Raw file compression error:", error);
      res.status(500).json({ error: "Compression failed", details: toErrorMessage(error) });
    }
  });

  app.post("/api/compression/decompress/raw", rawBodyParser, async (req, res) => {
    try {
      const body = req.body as Buffer;
      if (!body || !Buffer.isBuffer(body) || body.length === 0) {
        return res.status(400).json({ error: "Empty or invalid body. Send .tern bytes with Content-Type: application/octet-stream" });
      }

      const { parseTernFile } = await import("./compression-layer");
      const startTime = performance.now();

      const { header, originalData, ttcMetadata } = parseTernFile(body);
      const processingTimeMs = performance.now() - startTime;

      res.set({
        'Content-Type': 'application/octet-stream',
        'Content-Disposition': `attachment; filename*=UTF-8''${encodeURIComponent(header.originalFileName || 'decompressed.bin')}`,
        'X-TTC-Original-Size': String(header.originalSize),
        'X-TTC-Compressed-Size': String(header.compressedSize),
        'X-TTC-Compression-Ratio': header.compressionRatio?.toFixed(1) || '',
        'X-TTC-Engine': ttcMetadata?.engine || 'ttc-ts-fallback',
        'X-TTC-Was-Encrypted': String(header.encrypted || false),
        'X-TTC-Original-Filename': encodeURIComponent(header.originalFileName || ''),
        'X-TTC-CRC32-Verified': String(ttcMetadata?.crc32Verified ?? true),
        'X-TTC-Version': ttcMetadata?.version || '1.0',
        'X-TTC-Processing-Ms': processingTimeMs.toFixed(2),
      });

      res.send(originalData);
    } catch (error: unknown) {
      log.error("Raw file decompression error:", error);
      res.status(500).json({ error: "Decompression failed", details: toErrorMessage(error) });
    }
  });

  // ==========================================
  // TRANSPARENT DATABASE COMPRESSION API
  // ==========================================

  app.post("/api/compression/db/store", async (req, res) => {
    try {
      const schema = z.object({
        title: z.string().min(1),
        content: z.string().min(1),
        compress: z.boolean().optional().default(true),
        encrypt: z.boolean().optional().default(false),
        encryptionMode: z.enum(["high_security", "balanced", "performance", "adaptive"]).optional().default("balanced"),
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      
      const { title, content, compress, encrypt, encryptionMode } = parsed.data;
      const { compressForStorage, getCompressionMetadata } = await import("./compression-layer");
      
      const originalSize = Buffer.from(content, 'utf-8').length;
      let storedContent: string;
      let storedSize: number;
      let ratio: number | null = null;
      
      if (compress) {
        storedContent = compressForStorage(content, {
          enabled: true,
          encrypt,
          encryptionMode,
        });
        storedSize = Buffer.from(storedContent, 'utf-8').length;
        ratio = ((originalSize - storedSize) / originalSize) * 100;
      } else {
        storedContent = content;
        storedSize = originalSize;
      }
      
      const doc = await storage.createCompressedDocument({
        title,
        content: storedContent,
        isCompressed: compress ? 1 : 0,
        isEncrypted: encrypt ? 1 : 0,
        encryptionMode: encrypt ? encryptionMode : null,
        originalSizeBytes: originalSize,
        storedSizeBytes: storedSize,
        compressionRatio: ratio,
      });
      
      res.json({
        success: true,
        document: {
          id: doc.id,
          title: doc.title,
          isCompressed: !!doc.isCompressed,
          isEncrypted: !!doc.isEncrypted,
          encryptionMode: doc.encryptionMode,
          originalSizeBytes: doc.originalSizeBytes,
          storedSizeBytes: doc.storedSizeBytes,
          compressionRatio: doc.compressionRatio,
          createdAt: doc.createdAt,
        },
      });
    } catch (error: unknown) {
      log.error("DB compression store error:", error);
      res.status(500).json({ error: "Failed to store document", details: toErrorMessage(error) });
    }
  });

  app.get("/api/compression/db/retrieve/:id", async (req, res) => {
    try {
      const id = parseInt(req.params.id);
      if (isNaN(id)) {
        return res.status(400).json({ error: "Invalid document ID" });
      }
      
      const doc = await storage.getCompressedDocument(id);
      if (!doc) {
        return res.status(404).json({ error: "Document not found" });
      }
      
      const { decompressFromStorage, getCompressionMetadata } = await import("./compression-layer");
      
      let decompressedContent: string;
      if (doc.isCompressed) {
        decompressedContent = decompressFromStorage(doc.content);
      } else {
        decompressedContent = doc.content;
      }
      
      const metadata = doc.isCompressed ? getCompressionMetadata(doc.content) : null;
      
      res.json({
        success: true,
        document: {
          id: doc.id,
          title: doc.title,
          content: decompressedContent,
          isCompressed: !!doc.isCompressed,
          isEncrypted: !!doc.isEncrypted,
          encryptionMode: doc.encryptionMode,
          originalSizeBytes: doc.originalSizeBytes,
          storedSizeBytes: doc.storedSizeBytes,
          compressionRatio: doc.compressionRatio,
          createdAt: doc.createdAt,
        },
        storageMetadata: metadata,
      });
    } catch (error: unknown) {
      log.error("DB compression retrieve error:", error);
      res.status(500).json({ error: "Failed to retrieve document", details: toErrorMessage(error) });
    }
  });

  app.get("/api/compression/db/documents", async (req, res) => {
    try {
      const docs = await storage.getAllCompressedDocuments();
      
      res.json({
        success: true,
        documents: docs.map(doc => ({
          id: doc.id,
          title: doc.title,
          isCompressed: !!doc.isCompressed,
          isEncrypted: !!doc.isEncrypted,
          encryptionMode: doc.encryptionMode,
          originalSizeBytes: doc.originalSizeBytes,
          storedSizeBytes: doc.storedSizeBytes,
          compressionRatio: doc.compressionRatio,
          createdAt: doc.createdAt,
        })),
      });
    } catch (error: unknown) {
      log.error("DB documents list error:", error);
      res.status(500).json({ error: "Failed to list documents" });
    }
  });

  app.get("/api/compression/db/raw/:id", async (req, res) => {
    try {
      const id = parseInt(req.params.id);
      if (isNaN(id)) {
        return res.status(400).json({ error: "Invalid document ID" });
      }
      
      const doc = await storage.getCompressedDocument(id);
      if (!doc) {
        return res.status(404).json({ error: "Document not found" });
      }
      
      res.json({
        success: true,
        raw: {
          id: doc.id,
          title: doc.title,
          storedContent: doc.content.substring(0, 500) + (doc.content.length > 500 ? '...' : ''),
          storedContentLength: doc.content.length,
          isCompressed: !!doc.isCompressed,
          isEncrypted: !!doc.isEncrypted,
        },
      });
    } catch (error: unknown) {
      log.error("DB raw view error:", error);
      res.status(500).json({ error: "Failed to get raw document" });
    }
  });

  app.delete("/api/compression/db/documents/:id", isAuthenticated, async (req, res) => {
    try {
      const id = parseInt(req.params.id as string);
      if (isNaN(id)) {
        return res.status(400).json({ error: "Invalid document ID" });
      }
      await storage.deleteCompressedDocument(id);
      res.json({ success: true });
    } catch (error: unknown) {
      log.error("DB document delete error:", error);
      res.status(500).json({ error: "Failed to delete document" });
    }
  });

  // Whitepaper API routes
  app.get("/api/whitepapers", async (req, res) => {
    try {
      const allWhitepapers = await storage.getAllWhitepapers();
      res.json({ success: true, whitepapers: allWhitepapers });
    } catch (error: unknown) {
      log.error("Get whitepapers error:", error);
      res.status(500).json({ error: "Failed to get whitepapers" });
    }
  });

  app.get("/api/whitepapers/active", async (req, res) => {
    res.setHeader("Content-Type", "application/json");
    try {
      const whitepaper = await storage.getActiveWhitepaper();
      if (!whitepaper) {
        return res.status(404).json({ success: false, error: "No active whitepaper found" });
      }
      res.json({ success: true, whitepaper });
    } catch (error: unknown) {
      log.error("Get active whitepaper error:", error);
      res.status(500).json({ success: false, error: "Failed to get whitepaper" });
    }
  });

  app.get("/api/whitepapers/:id", async (req, res) => {
    try {
      const id = parseInt(req.params.id);
      const whitepaper = await storage.getWhitepaper(id);
      if (!whitepaper) {
        return res.status(404).json({ error: "Whitepaper not found" });
      }
      res.json({ success: true, whitepaper });
    } catch (error: unknown) {
      log.error("Get whitepaper error:", error);
      res.status(500).json({ error: "Failed to get whitepaper" });
    }
  });

  app.post("/api/whitepapers", isAuthenticated, async (req, res) => {
    try {
      const parsed = insertWhitepaperSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { version, title, content, summary, author } = parsed.data;
      const whitepaper = await storage.createWhitepaper({
        version,
        title,
        content,
        summary: summary || null,
        author: author || null,
        isActive: 1
      });
      res.json({ success: true, whitepaper });
    } catch (error: unknown) {
      log.error("Create whitepaper error:", error);
      res.status(500).json({ error: "Failed to create whitepaper" });
    }
  });

  app.get("/api/demo/data/:sessionId", async (req, res) => {
    try {
      const { sessionId } = req.params;
      const page = parseInt(req.query.page as string) || 1;
      const pageSize = Math.min(Math.max(parseInt(req.query.pageSize as string) || 100, 1), 1000);
      
      const fetchStart = performance.now();
      const ternaryData = await storage.getTernaryStorage(sessionId);
      const fetchTimeMs = performance.now() - fetchStart;
      
      if (!ternaryData || ternaryData.length === 0) {
        return res.status(404).json({ error: "Compressed data not found in ternary_storage" });
      }
      
      const compressed = ternaryData[0];
      const compressedSizeBytes = compressed.compressedSizeBytes;
      const originalSizeBytes = compressed.originalSizeBytes;
      
      const decompressStart = performance.now();
      let decompressedData: object[];
      try {
        const decompressedString = decompressData(compressed.compressedData);
        decompressedData = JSON.parse(decompressedString);
      } catch {
        const binaryData = await storage.getBinaryStorage(sessionId);
        if (!binaryData || binaryData.length === 0) {
          return res.status(404).json({ error: "Data not found" });
        }
        decompressedData = binaryData[0].rawData as object[];
      }
      const decompressTimeMs = performance.now() - decompressStart;
      
      const totalRows = decompressedData.length;
      const totalPages = Math.ceil(totalRows / pageSize);
      const startIndex = (page - 1) * pageSize;
      const endIndex = Math.min(startIndex + pageSize, totalRows);
      const paginatedData = decompressedData.slice(startIndex, endIndex);
      
      const columns = totalRows > 0 ? Object.keys(decompressedData[0] as object) : [];
      
      res.json({
        success: true,
        sessionId,
        columns,
        rows: paginatedData,
        pagination: {
          page,
          pageSize,
          totalRows,
          totalPages,
          hasNext: page < totalPages,
          hasPrev: page > 1
        },
        metrics: {
          compressedSizeBytes,
          originalSizeBytes,
          compressionRatio: compressed.compressionRatio,
          fetchTimeMs: parseFloat(fetchTimeMs.toFixed(2)),
          decompressTimeMs: parseFloat(decompressTimeMs.toFixed(2)),
          totalTimeMs: parseFloat((fetchTimeMs + decompressTimeMs).toFixed(2)),
          throughputMBps: parseFloat(((originalSizeBytes / 1024 / 1024) / (decompressTimeMs / 1000)).toFixed(2))
        }
      });
    } catch (error: unknown) {
      log.error("Data fetch error:", error);
      res.status(500).json({ error: "Failed to get data" });
    }
  });

  // =====================================================
  // Developer Signup API
  // =====================================================

  app.post("/api/developer-signup", async (req, res) => {
    try {
      const signupSchema = insertDeveloperSignupSchema.extend({
        email: z.string().email(),
      });

      const parsed = signupSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid email address", details: parsed.error.errors });
      }

      const existing = await storage.getDeveloperSignupByEmail(parsed.data.email);
      if (existing) {
        return res.json({ success: true, message: "You're already on the list! We'll be in touch soon." });
      }

      await storage.createDeveloperSignup({
        email: parsed.data.email,
        name: parsed.data.name || null,
        company: parsed.data.company || null,
        interest: parsed.data.interest || null,
      });

      const count = await storage.getDeveloperSignupCount();
      res.json({ success: true, message: "Welcome aboard! You'll be among the first to get access.", count });
    } catch (error: unknown) {
      log.error("Developer signup error:", error);
      res.status(500).json({ error: "Failed to process signup" });
    }
  });

  app.get("/api/developer-signup/count", async (_req, res) => {
    try {
      const count = await storage.getDeveloperSignupCount();
      res.json({ success: true, count });
    } catch (error: unknown) {
      res.status(500).json({ error: "Failed to get count" });
    }
  });

  // =====================================================
  // Admin API (authenticated)
  // =====================================================

  app.get("/api/admin/developer-signups", isAuthenticated, async (_req, res) => {
    try {
      const signups = await storage.getAllDeveloperSignups();
      res.json({ success: true, signups });
    } catch (error: unknown) {
      log.error("Error fetching signups:", error);
      res.status(500).json({ error: "Failed to fetch signups" });
    }
  });

  app.delete("/api/admin/developer-signups/:id", isAuthenticated, async (req, res) => {
    try {
      const id = parseInt(req.params.id as string, 10);
      if (isNaN(id)) {
        return res.status(400).json({ error: "Invalid ID" });
      }
      await storage.deleteDeveloperSignup(id);
      res.json({ success: true });
    } catch (error: unknown) {
      log.error("Error deleting signup:", error);
      res.status(500).json({ error: "Failed to delete signup" });
    }
  });


  // =====================================================
  // SALVI CORE API — extracted to server/routes/salvi.ts
  // =====================================================
  registerSalviRoutes(app);

  // =====================================================
  // GITHUB FILE MANAGER API (Admin Only) — extracted to server/routes/github.ts
  // =====================================================
  registerGitHubRoutes(app, storage);

  // Get user admin status
  app.get("/api/user/admin-status", async (req: any, res) => {
    try {
      // Prevent caching so admin status is always fresh
      res.set('Cache-Control', 'no-store, no-cache, must-revalidate');
      res.set('Pragma', 'no-cache');
      
      if (!req.isAuthenticated?.() || !req.user?.claims?.sub) {
        return res.json({ isAdmin: false, authenticated: false });
      }
      const user = await storage.getUser(req.user.claims.sub);
      res.json({ 
        isAdmin: user?.isAdmin || false, 
        authenticated: true,
        hasGithubToken: !!(user?.githubToken || process.env.GITHUB_TOKEN)
      });
    } catch (error: unknown) {
      res.status(500).json({ error: "Failed to check status" });
    }
  });

  // =====================================================
  // TRIBONACCI INDEXING LAYER API — extracted to server/routes/tribonacci.ts
  // =====================================================
  registerTribonacciRoutes(app);

  // =====================================================
  // 28-DIMENSION AI AGENT ARRAY — extracted to server/routes/agent-array.ts
  // =====================================================
  registerAgentArrayRoutes(app);

  // =====================================================
  // KONG KONNECT INTEGRATION API — extracted to server/routes/kong.ts
  // =====================================================
  registerKongRoutes(app, storage);
  registerApiKeyRoutes(app, storage);

  apiKeyService.startRotationCron();

  // =====================================================
  // GDPR DATA SUBJECT RIGHTS — extracted to server/routes/data-subject-rights.ts
  // =====================================================
  registerDataSubjectRightsRoutes(app, storage);

  // =====================================================
  // SECURITY INFRASTRUCTURE — extracted to server/routes/security.ts
  // =====================================================
  registerSecurityRoutes(app, storage);

  // =====================================================
  // TERNARY EPHEMERIS API — extracted to server/routes/ephemeris.ts
  // =====================================================
  registerEphemerisRoutes(app);

  // =====================================================
  // TONAL DIFFUSION API — tonal field, resonance, metrics
  // =====================================================
  registerTonalFieldRoutes(app);

  // =====================================================
  // CAPABILITY TOKEN API — Phase 2 (HPTP expiration) + Phase 3 (HMAC delegation)
  // =====================================================
  registerCapabilityRoutes(app);

  // =====================================================
  // INTER-CUBE INFRASTRUCTURE SERVICES (GLB, CON, CRS, FTS)
  // Pure geometric routing — no routing tables
  // =====================================================
  registerInterCubeRoutes(app);

  return httpServer;
}
