import type { Express } from "express";
import { createServer, type Server } from "http";
import { storage } from "./storage";
import { randomUUID } from "crypto";
import { z } from "zod";
import * as XLSX from "xlsx";
import { 
  compressData,
  decompressData,
  generateSensorData, 
  generateUserEvents, 
  generateLogEntries 
} from "./ternary";
import { setupAuth, registerAuthRoutes } from "./replit_integrations/auth";
import {
  convertTrit,
  convertVector,
  isValidTrit,
  getTritMeaning,
  type Representation,
  type TritA
} from "./salvi-core/ternary-types";
import {
  ternaryAdd,
  ternaryMultiply,
  ternaryRotate,
  ternaryNot,
  ternaryXor,
  adaptiveTernaryAdd,
  batchTernaryAdd,
  calculateInformationDensity,
  type SecurityMode
} from "./salvi-core/ternary-operations";
import {
  getFemtosecondTimestamp,
  getTimingMetrics,
  calculateDuration,
  validateRecombinationWindow,
  generateTimestampBatch,
  SALVI_EPOCH
} from "./salvi-core/femtosecond-timing";
import {
  phaseSplit,
  phaseRecombine,
  getPhaseConfig,
  getRecommendedMode,
  type EncryptionMode
} from "./salvi-core/phase-encryption";

const demoRunSchema = z.object({
  datasetName: z.enum(["sensor", "events", "logs"]),
  rowCount: z.number().int().min(1).max(10000).default(100)
});

export async function registerRoutes(
  httpServer: Server,
  app: Express
): Promise<Server> {
  
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
    } catch (error) {
      console.error("Demo run error:", error);
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
    } catch (error) {
      console.error("Stats error:", error);
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
    } catch (error) {
      console.error("Session error:", error);
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
    } catch (error) {
      console.error("Upload error:", error);
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
    } catch (error) {
      console.error("History error:", error);
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
    } catch (error) {
      console.error("Files error:", error);
      res.status(500).json({ error: "Failed to get files" });
    }
  });

  // Whitepaper API routes
  app.get("/api/whitepapers", async (req, res) => {
    try {
      const allWhitepapers = await storage.getAllWhitepapers();
      res.json({ success: true, whitepapers: allWhitepapers });
    } catch (error) {
      console.error("Get whitepapers error:", error);
      res.status(500).json({ error: "Failed to get whitepapers" });
    }
  });

  app.get("/api/whitepapers/active", async (req, res) => {
    try {
      const whitepaper = await storage.getActiveWhitepaper();
      if (!whitepaper) {
        return res.status(404).json({ error: "No active whitepaper found" });
      }
      res.json({ success: true, whitepaper });
    } catch (error) {
      console.error("Get active whitepaper error:", error);
      res.status(500).json({ error: "Failed to get whitepaper" });
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
    } catch (error) {
      console.error("Get whitepaper error:", error);
      res.status(500).json({ error: "Failed to get whitepaper" });
    }
  });

  app.post("/api/whitepapers", async (req, res) => {
    try {
      const { version, title, content, summary, author } = req.body;
      if (!version || !title || !content) {
        return res.status(400).json({ error: "version, title, and content are required" });
      }
      const whitepaper = await storage.createWhitepaper({
        version,
        title,
        content,
        summary: summary || null,
        author: author || null,
        isActive: 1
      });
      res.json({ success: true, whitepaper });
    } catch (error) {
      console.error("Create whitepaper error:", error);
      res.status(500).json({ error: "Failed to create whitepaper" });
    }
  });

  app.get("/api/demo/data/:sessionId", async (req, res) => {
    try {
      const { sessionId } = req.params;
      const page = parseInt(req.query.page as string) || 1;
      const pageSize = parseInt(req.query.pageSize as string) || 100;
      
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
    } catch (error) {
      console.error("Data fetch error:", error);
      res.status(500).json({ error: "Failed to get data" });
    }
  });

  // =====================================================
  // SALVI CORE API - Ternary Operations
  // =====================================================

  // API Documentation endpoint
  app.get("/api/salvi/docs", (req, res) => {
    res.json({
      name: "Salvi Framework Core API",
      version: "1.0.0",
      description: "Implements the Unified Ternary Logic System from the whitepaper",
      endpoints: {
        ternary: {
          convert: {
            path: "POST /api/salvi/ternary/convert",
            description: "Convert between ternary representations (A, B, C)",
            body: { value: "number", from: "A|B|C", to: "A|B|C" }
          },
          add: {
            path: "POST /api/salvi/ternary/add",
            description: "Ternary addition in GF(3)",
            body: { a: "-1|0|1", b: "-1|0|1" }
          },
          multiply: {
            path: "POST /api/salvi/ternary/multiply",
            description: "Ternary multiplication in GF(3)",
            body: { a: "-1|0|1", b: "-1|0|1" }
          },
          rotate: {
            path: "POST /api/salvi/ternary/rotate",
            description: "Bijective ternary rotation",
            body: { value: "-1|0|1", steps: "number" }
          },
          batch: {
            path: "POST /api/salvi/ternary/batch",
            description: "Batch ternary operations",
            body: { pairs: "[{a, b}]" }
          },
          density: {
            path: "GET /api/salvi/ternary/density/:tritCount",
            description: "Calculate information density advantage"
          }
        },
        timing: {
          timestamp: {
            path: "GET /api/salvi/timing/timestamp",
            description: "Get femtosecond-precision timestamp"
          },
          metrics: {
            path: "GET /api/salvi/timing/metrics",
            description: "Get timing metrics and synchronization status"
          },
          batch: {
            path: "GET /api/salvi/timing/batch/:count",
            description: "Generate batch of timestamps"
          }
        },
        phase: {
          split: {
            path: "POST /api/salvi/phase/split",
            description: "Split data into phase-encrypted components",
            body: { data: "string", mode: "high_security|balanced|performance|adaptive" }
          },
          recombine: {
            path: "POST /api/salvi/phase/recombine",
            description: "Recombine phase-split data",
            body: { encrypted: "EncryptedPhaseData" }
          },
          config: {
            path: "GET /api/salvi/phase/config/:mode",
            description: "Get phase configuration for encryption mode"
          }
        }
      },
      references: {
        whitepaper: "/whitepaper",
        representations: {
          A: "Computational: {-1, 0, +1}",
          B: "Network: {0, 1, 2}",
          C: "Human: {1, 2, 3}"
        },
        bijections: {
          "A→B": "f(a) = a + 1",
          "A→C": "f(a) = a + 2",
          "B→C": "f(b) = b + 1"
        }
      }
    });
  });

  // Ternary conversion
  app.post("/api/salvi/ternary/convert", (req, res) => {
    try {
      const schema = z.object({
        value: z.number(),
        from: z.enum(["A", "B", "C"]),
        to: z.enum(["A", "B", "C"])
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { value, from, to } = parsed.data;
      
      if (!isValidTrit(value, from as Representation)) {
        return res.status(400).json({ 
          error: `Invalid trit value ${value} for representation ${from}`,
          validValues: from === "A" ? [-1, 0, 1] : from === "B" ? [0, 1, 2] : [1, 2, 3]
        });
      }
      
      const result = convertTrit(value as TritA, from as Representation, to as Representation);
      res.json({ success: true, ...result });
    } catch (error) {
      res.status(500).json({ error: "Conversion failed" });
    }
  });

  // Ternary addition
  app.post("/api/salvi/ternary/add", (req, res) => {
    try {
      const schema = z.object({
        a: z.number().int().min(-1).max(1),
        b: z.number().int().min(-1).max(1),
        mode: z.enum(["phi", "mode1", "mode0"]).optional()
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { a, b, mode } = parsed.data;
      
      const result = mode 
        ? adaptiveTernaryAdd(a as TritA, b as TritA, mode as SecurityMode)
        : ternaryAdd(a as TritA, b as TritA);
      
      res.json({ success: true, ...result });
    } catch (error) {
      res.status(500).json({ error: "Addition failed" });
    }
  });

  // Ternary multiplication
  app.post("/api/salvi/ternary/multiply", (req, res) => {
    try {
      const schema = z.object({
        a: z.number().int().min(-1).max(1),
        b: z.number().int().min(-1).max(1)
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { a, b } = parsed.data;
      
      const result = ternaryMultiply(a as TritA, b as TritA);
      res.json({ success: true, ...result });
    } catch (error) {
      res.status(500).json({ error: "Multiplication failed" });
    }
  });

  // Ternary rotation
  app.post("/api/salvi/ternary/rotate", (req, res) => {
    try {
      const schema = z.object({
        value: z.number().int().min(-1).max(1),
        steps: z.number().int().default(1)
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { value, steps } = parsed.data;
      
      const result = ternaryRotate(value as TritA, steps);
      res.json({ success: true, ...result });
    } catch (error) {
      res.status(500).json({ error: "Rotation failed" });
    }
  });

  // Ternary NOT
  app.post("/api/salvi/ternary/not", (req, res) => {
    try {
      const schema = z.object({
        value: z.number().int().min(-1).max(1)
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { value } = parsed.data;
      
      const result = ternaryNot(value as TritA);
      res.json({ success: true, ...result });
    } catch (error) {
      res.status(500).json({ error: "NOT operation failed" });
    }
  });

  // Ternary XOR
  app.post("/api/salvi/ternary/xor", (req, res) => {
    try {
      const schema = z.object({
        a: z.number().int().min(-1).max(1),
        b: z.number().int().min(-1).max(1)
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { a, b } = parsed.data;
      
      const result = ternaryXor(a as TritA, b as TritA);
      res.json({ success: true, ...result });
    } catch (error) {
      res.status(500).json({ error: "XOR operation failed" });
    }
  });

  // Batch ternary addition
  app.post("/api/salvi/ternary/batch", (req, res) => {
    try {
      const schema = z.object({
        pairs: z.array(z.object({
          a: z.number().int().min(-1).max(1),
          b: z.number().int().min(-1).max(1)
        })).max(1000)
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      
      const results = batchTernaryAdd(parsed.data.pairs as Array<{ a: TritA; b: TritA }>);
      res.json({ 
        success: true, 
        count: results.length,
        results 
      });
    } catch (error) {
      res.status(500).json({ error: "Batch operation failed" });
    }
  });

  // Information density calculator
  app.get("/api/salvi/ternary/density/:tritCount", (req, res) => {
    try {
      const tritCount = parseInt(req.params.tritCount);
      if (isNaN(tritCount) || tritCount < 1 || tritCount > 1000000) {
        return res.status(400).json({ error: "tritCount must be between 1 and 1000000" });
      }
      
      const result = calculateInformationDensity(tritCount);
      res.json({ success: true, ...result });
    } catch (error) {
      res.status(500).json({ error: "Density calculation failed" });
    }
  });

  // =====================================================
  // SALVI CORE API - Femtosecond Timing
  // =====================================================

  // Get current timestamp
  app.get("/api/salvi/timing/timestamp", (req, res) => {
    try {
      const timestamp = getFemtosecondTimestamp();
      res.json({ 
        success: true, 
        timestamp: {
          ...timestamp,
          femtoseconds: timestamp.femtoseconds.toString(),
          salviEpochOffset: timestamp.salviEpochOffset.toString()
        },
        epoch: {
          salviEpoch: new Date(SALVI_EPOCH).toISOString(),
          description: "Femtoseconds since 2024-01-01T00:00:00Z"
        }
      });
    } catch (error) {
      res.status(500).json({ error: "Timestamp generation failed" });
    }
  });

  // Get timing metrics
  app.get("/api/salvi/timing/metrics", (req, res) => {
    try {
      const metrics = getTimingMetrics();
      res.json({ 
        success: true, 
        ...metrics,
        timestamp: {
          ...metrics.timestamp,
          femtoseconds: metrics.timestamp.femtoseconds.toString(),
          salviEpochOffset: metrics.timestamp.salviEpochOffset.toString()
        }
      });
    } catch (error) {
      res.status(500).json({ error: "Metrics retrieval failed" });
    }
  });

  // Generate batch of timestamps
  app.get("/api/salvi/timing/batch/:count", (req, res) => {
    try {
      const count = parseInt(req.params.count);
      if (isNaN(count) || count < 1 || count > 100) {
        return res.status(400).json({ error: "count must be between 1 and 100" });
      }
      
      const timestamps = generateTimestampBatch(count).map(ts => ({
        ...ts,
        femtoseconds: ts.femtoseconds.toString(),
        salviEpochOffset: ts.salviEpochOffset.toString()
      }));
      
      res.json({ 
        success: true, 
        count: timestamps.length,
        timestamps 
      });
    } catch (error) {
      res.status(500).json({ error: "Batch timestamp generation failed" });
    }
  });

  // =====================================================
  // SALVI CORE API - Phase Encryption
  // =====================================================

  // Get phase configuration
  app.get("/api/salvi/phase/config/:mode", (req, res) => {
    try {
      const mode = req.params.mode as EncryptionMode;
      const validModes = ["high_security", "balanced", "performance", "adaptive"];
      if (!validModes.includes(mode)) {
        return res.status(400).json({ 
          error: "Invalid mode",
          validModes 
        });
      }
      
      const config = getPhaseConfig(mode);
      res.json({ success: true, config });
    } catch (error) {
      res.status(500).json({ error: "Config retrieval failed" });
    }
  });

  // Phase split
  app.post("/api/salvi/phase/split", (req, res) => {
    try {
      const schema = z.object({
        data: z.string().min(1).max(100000),
        mode: z.enum(["high_security", "balanced", "performance", "adaptive"]).default("balanced")
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }
      const { data, mode } = parsed.data;
      
      const encrypted = phaseSplit(data, mode as EncryptionMode);
      
      // Convert BigInt to string for JSON serialization
      const serialized = {
        ...encrypted,
        primaryPhase: {
          ...encrypted.primaryPhase,
          timestamp: {
            ...encrypted.primaryPhase.timestamp,
            femtoseconds: encrypted.primaryPhase.timestamp.femtoseconds.toString(),
            salviEpochOffset: encrypted.primaryPhase.timestamp.salviEpochOffset.toString()
          }
        },
        secondaryPhase: {
          ...encrypted.secondaryPhase,
          timestamp: {
            ...encrypted.secondaryPhase.timestamp,
            femtoseconds: encrypted.secondaryPhase.timestamp.femtoseconds.toString(),
            salviEpochOffset: encrypted.secondaryPhase.timestamp.salviEpochOffset.toString()
          }
        },
        guardianPhase: encrypted.guardianPhase ? {
          ...encrypted.guardianPhase,
          timestamp: {
            ...encrypted.guardianPhase.timestamp,
            femtoseconds: encrypted.guardianPhase.timestamp.femtoseconds.toString(),
            salviEpochOffset: encrypted.guardianPhase.timestamp.salviEpochOffset.toString()
          }
        } : undefined
      };
      
      res.json({ success: true, encrypted: serialized });
    } catch (error) {
      res.status(500).json({ error: "Phase split failed" });
    }
  });

  // Phase recombine
  app.post("/api/salvi/phase/recombine", (req, res) => {
    try {
      // Restore BigInt from strings
      const encrypted = req.body.encrypted;
      if (!encrypted || !encrypted.primaryPhase || !encrypted.secondaryPhase) {
        return res.status(400).json({ error: "Invalid encrypted data structure" });
      }
      
      // Convert string timestamps back to BigInt
      encrypted.primaryPhase.timestamp.femtoseconds = BigInt(encrypted.primaryPhase.timestamp.femtoseconds);
      encrypted.primaryPhase.timestamp.salviEpochOffset = BigInt(encrypted.primaryPhase.timestamp.salviEpochOffset);
      encrypted.secondaryPhase.timestamp.femtoseconds = BigInt(encrypted.secondaryPhase.timestamp.femtoseconds);
      encrypted.secondaryPhase.timestamp.salviEpochOffset = BigInt(encrypted.secondaryPhase.timestamp.salviEpochOffset);
      
      if (encrypted.guardianPhase) {
        encrypted.guardianPhase.timestamp.femtoseconds = BigInt(encrypted.guardianPhase.timestamp.femtoseconds);
        encrypted.guardianPhase.timestamp.salviEpochOffset = BigInt(encrypted.guardianPhase.timestamp.salviEpochOffset);
      }
      
      const result = phaseRecombine(encrypted);
      res.json({ success: true, result });
    } catch (error) {
      res.status(500).json({ error: "Phase recombine failed" });
    }
  });

  // Get recommended encryption mode
  app.get("/api/salvi/phase/recommend", (req, res) => {
    try {
      const dataLength = parseInt(req.query.length as string) || 1000;
      const isSensitive = req.query.sensitive === "true";
      
      const mode = getRecommendedMode(dataLength, isSensitive);
      const config = getPhaseConfig(mode);
      
      res.json({ 
        success: true, 
        recommendation: {
          mode,
          config,
          reasoning: isSensitive 
            ? "High security mode recommended for sensitive data"
            : dataLength > 10000
              ? "Performance mode recommended for large data"
              : "Balanced mode recommended for standard use"
        }
      });
    } catch (error) {
      res.status(500).json({ error: "Recommendation failed" });
    }
  });

  // =====================================================
  // GITHUB FILE MANAGER API (Admin Only)
  // =====================================================

  // Middleware to check admin access
  const requireAdmin = async (req: any, res: any, next: any) => {
    if (!req.isAuthenticated?.() || !req.user) {
      return res.status(401).json({ error: "Authentication required" });
    }
    const user = await storage.getUser(req.user.id);
    if (!user?.isAdmin) {
      return res.status(403).json({ error: "Admin access required" });
    }
    req.adminUser = user;
    next();
  };

  // Save GitHub Personal Access Token
  app.post("/api/github/token", requireAdmin, async (req: any, res) => {
    try {
      const schema = z.object({
        token: z.string().min(1)
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid token" });
      }
      
      await storage.updateUserGithubToken(req.adminUser.id, parsed.data.token);
      res.json({ success: true, message: "GitHub token saved" });
    } catch (error) {
      console.error("GitHub token save error:", error);
      res.status(500).json({ error: "Failed to save token" });
    }
  });

  // Check if GitHub token is configured
  app.get("/api/github/status", requireAdmin, async (req: any, res) => {
    try {
      const hasToken = !!req.adminUser.githubToken;
      res.json({ success: true, hasToken });
    } catch (error) {
      res.status(500).json({ error: "Failed to check status" });
    }
  });

  // Get repository contents (files and folders)
  app.get("/api/github/repos/:owner/:repo/contents", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const path = (req.query.path as string) || "";
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/contents/${path}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "User-Agent": "Salvi-Framework"
          }
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      res.json({ success: true, data });
    } catch (error) {
      console.error("GitHub contents error:", error);
      res.status(500).json({ error: "Failed to fetch contents" });
    }
  });

  // Get file content
  app.get("/api/github/file/:owner/:repo", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const path = (req.query.path as string) || "";
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/contents/${path}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "User-Agent": "Salvi-Framework"
          }
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      
      // Decode base64 content
      if (data.content) {
        const content = Buffer.from(data.content, "base64").toString("utf-8");
        res.json({ 
          success: true, 
          file: {
            name: data.name,
            path: data.path,
            sha: data.sha,
            size: data.size,
            content
          }
        });
      } else {
        res.status(400).json({ error: "Not a file" });
      }
    } catch (error) {
      console.error("GitHub file error:", error);
      res.status(500).json({ error: "Failed to fetch file" });
    }
  });

  // Create or update file
  app.put("/api/github/file/:owner/:repo", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const path = req.body.path || "";
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const schema = z.object({
        path: z.string().min(1),
        content: z.string(),
        message: z.string().min(1),
        sha: z.string().optional() // Required for updates, not for creates
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { content, message, sha } = parsed.data;
      const encodedContent = Buffer.from(content).toString("base64");

      const body: any = {
        message,
        content: encodedContent
      };
      if (sha) {
        body.sha = sha;
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/contents/${path}`,
        {
          method: "PUT",
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "Content-Type": "application/json",
            "User-Agent": "Salvi-Framework"
          },
          body: JSON.stringify(body)
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      res.json({ 
        success: true, 
        message: sha ? "File updated" : "File created",
        commit: data.commit
      });
    } catch (error) {
      console.error("GitHub file create/update error:", error);
      res.status(500).json({ error: "Failed to save file" });
    }
  });

  // Delete file
  app.delete("/api/github/file/:owner/:repo", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const path = req.body.path || "";
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const schema = z.object({
        path: z.string().min(1),
        message: z.string().min(1),
        sha: z.string()
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { path: filePath, message, sha } = parsed.data;

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/contents/${filePath}`,
        {
          method: "DELETE",
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "Content-Type": "application/json",
            "User-Agent": "Salvi-Framework"
          },
          body: JSON.stringify({ message, sha })
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      res.json({ 
        success: true, 
        message: "File deleted",
        commit: data.commit
      });
    } catch (error) {
      console.error("GitHub file delete error:", error);
      res.status(500).json({ error: "Failed to delete file" });
    }
  });

  // Get user admin status
  app.get("/api/user/admin-status", async (req: any, res) => {
    try {
      if (!req.isAuthenticated?.() || !req.user) {
        return res.json({ isAdmin: false, authenticated: false });
      }
      const user = await storage.getUser(req.user.id);
      res.json({ 
        isAdmin: user?.isAdmin || false, 
        authenticated: true,
        hasGithubToken: !!user?.githubToken
      });
    } catch (error) {
      res.status(500).json({ error: "Failed to check status" });
    }
  });

  return httpServer;
}
