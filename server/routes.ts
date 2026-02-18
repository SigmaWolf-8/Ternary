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
import { apiKeyService } from "./services/api-key.service";
import { readFile } from "fs/promises";
import * as path from "path";
import * as XLSX from "xlsx";
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

  app.use((req, _res, next) => {
    if (req.path.startsWith("/api/v1/")) {
      req.url = req.url.replace("/api/v1/", "/api/");
    }
    next();
  });

  const legalDocMap: Record<string, { file: string; title: string }> = {
    terms: { file: "TERMS-OF-SERVICE.md", title: "Terms of Service" },
    privacy: { file: "PRIVACY-POLICY.md", title: "Privacy Policy" },
    security: { file: "SECURITY.md", title: "Security Policy" },
    aup: { file: "ACCEPTABLE-USE-POLICY.md", title: "Acceptable Use Policy" },
    "export-control": { file: "EXPORT-CONTROL.md", title: "Export Control Classification" },
    "ip-notice": { file: "IP-NOTICE.md", title: "Intellectual Property Notice" },
  };

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
          const binaryData = Buffer.from(content, 'base64');
          const workbook = XLSX.read(binaryData, { type: 'buffer' });
          const firstSheetName = workbook.SheetNames[0];
          if (!firstSheetName) {
            return res.status(400).json({ error: "Excel file has no sheets" });
          }
          const worksheet = workbook.Sheets[firstSheetName];
          rawData = XLSX.utils.sheet_to_json(worksheet);
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
  
  app.post("/api/compression/file", async (req, res) => {
    try {
      const schema = z.object({
        fileName: z.string().min(1),
        content: z.string().min(1),
        encrypt: z.boolean().optional().default(false),
        encryptionMode: z.enum(["high_security", "balanced", "performance", "adaptive"]).optional().default("balanced"),
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      
      const { fileName, content, encrypt, encryptionMode } = parsed.data;
      const { createTernFile } = await import("./compression-layer");
      
      const inputBuffer = Buffer.from(content, 'base64');
      const startTime = performance.now();
      
      const { ternFile, header } = createTernFile(inputBuffer, fileName, {
        encrypt,
        encryptionMode: encrypt ? encryptionMode : undefined,
      });
      
      const processingTimeMs = performance.now() - startTime;
      
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
      });
    } catch (error: unknown) {
      log.error("File compression error:", error);
      res.status(500).json({ error: "Compression failed", details: toErrorMessage(error) });
    }
  });

  app.post("/api/compression/decompress", async (req, res) => {
    try {
      const schema = z.object({
        content: z.string().min(1),
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      
      const { parseTernFile } = await import("./compression-layer");
      const ternBuffer = Buffer.from(parsed.data.content, 'base64');
      const startTime = performance.now();
      
      const { header, originalData } = parseTernFile(ternBuffer);
      const processingTimeMs = performance.now() - startTime;
      
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
      });
    } catch (error: unknown) {
      log.error("File decompression error:", error);
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

  return httpServer;
}
