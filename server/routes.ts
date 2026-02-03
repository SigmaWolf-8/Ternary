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
    if (!req.isAuthenticated?.() || !req.user?.claims?.sub) {
      return res.status(401).json({ error: "Authentication required" });
    }
    const user = await storage.getUser(req.user.claims.sub);
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

  // Helper to validate and sanitize paths
  const sanitizePath = (path: string): string => {
    return path
      .replace(/\.\./g, "")
      .replace(/^\/+/, "")
      .replace(/\/+$/, "");
  };

  // Get repository branches
  app.get("/api/github/repos/:owner/:repo/branches", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/branches`,
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
      res.json({ success: true, branches: data.map((b: any) => ({ name: b.name, protected: b.protected })) });
    } catch (error) {
      console.error("GitHub branches error:", error);
      res.status(500).json({ error: "Failed to fetch branches" });
    }
  });

  // Get repository contents (files and folders)
  app.get("/api/github/repos/:owner/:repo/contents", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const path = sanitizePath((req.query.path as string) || "");
      const branch = (req.query.branch as string) || "";
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const url = new URL(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`);
      if (branch) {
        url.searchParams.set("ref", branch);
      }

      const response = await fetch(
        url.toString(),
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
      const path = sanitizePath((req.query.path as string) || "");
      const branch = (req.query.branch as string) || "";
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const url = new URL(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`);
      if (branch) {
        url.searchParams.set("ref", branch);
      }

      const response = await fetch(
        url.toString(),
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
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const schema = z.object({
        path: z.string().min(1),
        content: z.string(),
        message: z.string().min(1),
        sha: z.string().optional(), // Required for updates, not for creates
        branch: z.string().optional() // Target branch
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { path: rawPath, content, message, sha, branch } = parsed.data;
      const filePath = sanitizePath(rawPath);
      const encodedContent = Buffer.from(content).toString("base64");

      const body: any = {
        message,
        content: encodedContent
      };
      if (sha) {
        body.sha = sha;
      }
      if (branch) {
        body.branch = branch;
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/contents/${filePath}`,
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
      const token = req.adminUser.githubToken;
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const schema = z.object({
        path: z.string().min(1),
        message: z.string().min(1),
        sha: z.string(),
        branch: z.string().optional()
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { path: rawPath, message, sha, branch } = parsed.data;
      const filePath = sanitizePath(rawPath);

      const deleteBody: any = { message, sha };
      if (branch) {
        deleteBody.branch = branch;
      }

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
          body: JSON.stringify(deleteBody)
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
        hasGithubToken: !!user?.githubToken
      });
    } catch (error) {
      res.status(500).json({ error: "Failed to check status" });
    }
  });

  // Kong Konnect Integration API Routes
  const KONG_API_BASE = "https://us.api.konghq.com/v2";
  const KONG_KONNECT_TOKEN = process.env.KONG_KONNECT_TOKEN;

  // Check Kong Konnect connection status
  app.get("/api/kong/status", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.json({ 
          connected: false, 
          error: "Kong Konnect token not configured" 
        });
      }

      const response = await fetch(`${KONG_API_BASE}/users/me`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.json({ 
          connected: false, 
          error: `API error: ${response.status}` 
        });
      }

      const user = await response.json();
      res.json({ 
        connected: true, 
        user: {
          id: user.id,
          email: user.email,
          fullName: user.full_name,
          preferredName: user.preferred_name,
          active: user.active
        }
      });
    } catch (error) {
      res.json({ 
        connected: false, 
        error: error instanceof Error ? error.message : "Connection failed" 
      });
    }
  });

  // Get Kong organization info
  app.get("/api/kong/organization", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const response = await fetch(`${KONG_API_BASE}/organizations/me`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const org = await response.json();
      res.json(org);
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to fetch organization" });
    }
  });

  // List Kong control planes
  app.get("/api/kong/control-planes", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to fetch control planes" });
    }
  });

  // Get services for a control plane
  app.get("/api/kong/control-planes/:cpId/services", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to fetch services" });
    }
  });

  // Get routes for a control plane
  app.get("/api/kong/control-planes/:cpId/routes", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/routes`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to fetch routes" });
    }
  });

  // Get plugins for a control plane
  app.get("/api/kong/control-planes/:cpId/plugins", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/plugins`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to fetch plugins" });
    }
  });

  // Get Kong configuration file (Admin only - may contain API keys)
  app.get("/api/kong/config", requireAdmin, async (req: any, res) => {
    try {
      const fs = await import('fs/promises');
      const path = await import('path');
      const configPath = path.join(process.cwd(), 'kong', 'kong.yaml');
      const config = await fs.readFile(configPath, 'utf-8');
      res.json({ success: true, config });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to read config" });
    }
  });

  // Create a service in Kong Konnect (Admin only)
  app.post("/api/kong/control-planes/:cpId/services", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const { name, url, enabled = true, tags = [] } = req.body;

      if (!name || !url) {
        return res.status(400).json({ error: "Name and URL are required" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ name, url, enabled, tags })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return res.status(response.status).json({ 
          error: `API error: ${response.status}`,
          details: errorData 
        });
      }

      const data = await response.json();
      res.json({ success: true, service: data });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to create service" });
    }
  });

  // Create a route for a service in Kong Konnect (Admin only)
  app.post("/api/kong/control-planes/:cpId/services/:serviceId/routes", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId, serviceId } = req.params;
      const { name, paths, methods = ['GET', 'POST'], strip_path = true, tags = [] } = req.body;

      if (!name || !paths || !paths.length) {
        return res.status(400).json({ error: "Name and paths are required" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/routes`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ name, paths, methods, strip_path, tags })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return res.status(response.status).json({ 
          error: `API error: ${response.status}`,
          details: errorData 
        });
      }

      const data = await response.json();
      res.json({ success: true, route: data });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to create route" });
    }
  });

  // Add a plugin to a service (Admin only)
  app.post("/api/kong/control-planes/:cpId/services/:serviceId/plugins", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId, serviceId } = req.params;
      const { name, config = {}, tags = [] } = req.body;

      if (!name) {
        return res.status(400).json({ error: "Plugin name is required" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/plugins`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ name, config, tags })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return res.status(response.status).json({ 
          error: `API error: ${response.status}`,
          details: errorData 
        });
      }

      const data = await response.json();
      res.json({ success: true, plugin: data });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to add plugin" });
    }
  });

  // Sync PlenumNET services to Kong Konnect (Admin only)
  app.post("/api/kong/control-planes/:cpId/sync-plenumnet", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      
      // Use REPLIT_DOMAINS env var or default to known domain - never trust client input
      const replitDomains = process.env.REPLIT_DOMAINS || process.env.REPLIT_DEV_DOMAIN;
      const baseUrl = replitDomains 
        ? `https://${replitDomains.split(',')[0]}`
        : 'https://plenumnet.replit.app';
      
      const services = [
        {
          name: "plenumnet-timing",
          url: `${baseUrl}/api/salvi/timing`,
          tags: ["plenumnet", "timing", "finra-cat"],
          routePath: "/api/timing",
          rateLimit: { minute: 100 }
        },
        {
          name: "plenumnet-ternary",
          url: `${baseUrl}/api/salvi/ternary`,
          tags: ["plenumnet", "ternary", "quantum-safe"],
          routePath: "/api/ternary",
          rateLimit: { minute: 200 }
        },
        {
          name: "plenumnet-phase",
          url: `${baseUrl}/api/salvi/phase`,
          tags: ["plenumnet", "encryption", "quantum-safe"],
          routePath: "/api/phase",
          rateLimit: { minute: 100 }
        },
        {
          name: "plenumnet-demo",
          url: `${baseUrl}/api/demo`,
          tags: ["plenumnet", "demo", "compression"],
          routePath: "/api/demo",
          rateLimit: { minute: 50 }
        },
        {
          name: "plenumnet-whitepapers",
          url: `${baseUrl}/api/whitepapers`,
          tags: ["plenumnet", "whitepapers", "documentation"],
          routePath: "/api/whitepapers",
          rateLimit: { minute: 100 }
        },
        {
          name: "plenumnet-docs",
          url: `${baseUrl}/api/salvi/docs`,
          tags: ["plenumnet", "docs", "documentation"],
          routePath: "/api/docs",
          rateLimit: { minute: 200 }
        }
      ];

      const results: any[] = [];
      
      // First, get all existing services to check what already exists
      const existingServicesResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
        headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
      });
      const existingServicesData = existingServicesResponse.ok ? await existingServicesResponse.json() : { data: [] };
      const existingServices = existingServicesData.data || [];

      for (const service of services) {
        try {
          // Step 1: Check if service exists, create if not
          let serviceId: string | null = null;
          const existingService = existingServices.find((s: any) => s.name === service.name);
          
          if (existingService) {
            serviceId = existingService.id;
            results.push({ 
              service: service.name, 
              status: 'already_exists',
              id: serviceId
            });
          } else {
            const createResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
              method: 'POST',
              headers: {
                "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
                "Content-Type": "application/json"
              },
              body: JSON.stringify({
                name: service.name,
                url: service.url,
                enabled: true,
                tags: service.tags
              })
            });

            if (createResponse.ok) {
              const createdService = await createResponse.json();
              serviceId = createdService.id;
              results.push({ 
                service: service.name, 
                status: 'created',
                id: serviceId
              });
            } else {
              const errorText = await createResponse.text();
              results.push({ 
                service: service.name, 
                status: 'error',
                error: `HTTP ${createResponse.status}: ${errorText}`
              });
              continue;
            }
          }

          if (!serviceId) continue;

          // Step 2: Create route for the service
          const routeResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/routes`, {
            method: 'POST',
            headers: {
              "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
              "Content-Type": "application/json"
            },
            body: JSON.stringify({
              name: `${service.name}-route`,
              paths: [service.routePath],
              methods: ["GET", "POST", "PUT", "DELETE", "PATCH"],
              strip_path: true,
              tags: service.tags
            })
          });

          if (routeResponse.ok) {
            const route = await routeResponse.json();
            results.push({
              route: `${service.name}-route`,
              status: 'route_created',
              id: route.id
            });
          } else if (routeResponse.status === 409) {
            results.push({
              route: `${service.name}-route`,
              status: 'route_exists'
            });
          }

          // Step 3: Add rate limiting plugin
          const pluginResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/plugins`, {
            method: 'POST',
            headers: {
              "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
              "Content-Type": "application/json"
            },
            body: JSON.stringify({
              name: "rate-limiting",
              config: {
                minute: service.rateLimit.minute,
                policy: "local"
              },
              tags: ["plenumnet", "rate-limit"]
            })
          });

          if (pluginResponse.ok) {
            const plugin = await pluginResponse.json();
            results.push({
              plugin: `rate-limiting (${service.rateLimit.minute}/min)`,
              service: service.name,
              status: 'plugin_created',
              id: plugin.id
            });
          } else if (pluginResponse.status === 409) {
            results.push({
              plugin: 'rate-limiting',
              service: service.name,
              status: 'plugin_exists'
            });
          }

        } catch (err) {
          results.push({ 
            service: service.name, 
            status: 'error',
            error: err instanceof Error ? err.message : 'Unknown error'
          });
        }
      }

      res.json({ 
        success: true, 
        services: results.filter(r => r.status === 'created' || r.status === 'already_exists').length,
        routes: results.filter(r => r.status === 'route_created' || r.status === 'route_exists').length,
        plugins: results.filter(r => r.status === 'plugin_created' || r.status === 'plugin_exists').length,
        errors: results.filter(r => r.status === 'error').length,
        results 
      });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to sync services" });
    }
  });

  // Save Kong config to GitHub (Admin only)
  app.post("/api/kong/save-to-github", requireAdmin, async (req: any, res) => {
    try {
      const user = req.adminUser; // Set by requireAdmin middleware
      if (!user?.githubToken) {
        return res.status(400).json({ error: "GitHub token not configured. Please add your GitHub token in the GitHub Manager." });
      }

      const { owner, repo, path = "kong/kong.yaml", message = "Update Kong Konnect configuration" } = req.body;
      
      if (!owner || !repo) {
        return res.status(400).json({ error: "Owner and repo are required" });
      }

      const fs = await import('fs/promises');
      const pathModule = await import('path');
      const configPath = pathModule.join(process.cwd(), 'kong', 'kong.yaml');
      const config = await fs.readFile(configPath, 'utf-8');
      const content = Buffer.from(config).toString('base64');

      const existingResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`, {
        headers: {
          "Authorization": `token ${user.githubToken}`,
          "Accept": "application/vnd.github.v3+json"
        }
      });

      let sha: string | undefined;
      if (existingResponse.ok) {
        const existingFile = await existingResponse.json();
        sha = existingFile.sha;
      }

      const createResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`, {
        method: 'PUT',
        headers: {
          "Authorization": `token ${user.githubToken}`,
          "Accept": "application/vnd.github.v3+json",
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          message,
          content,
          sha
        })
      });

      if (!createResponse.ok) {
        const errorData = await createResponse.json().catch(() => ({}));
        return res.status(createResponse.status).json({ 
          error: `GitHub API error: ${createResponse.status}`,
          details: errorData 
        });
      }

      const result = await createResponse.json();
      res.json({ 
        success: true, 
        message: sha ? "Configuration updated" : "Configuration created",
        url: result.content?.html_url,
        sha: result.content?.sha
      });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to save to GitHub" });
    }
  });

  // Get data plane deployment instructions for a control plane
  app.get("/api/kong/control-planes/:cpId/deploy-instructions", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      
      // Get control plane details
      const cpResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}`, {
        headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
      });

      if (!cpResponse.ok) {
        return res.status(cpResponse.status).json({ error: "Failed to fetch control plane details" });
      }

      const cpData = await cpResponse.json();
      const controlPlaneEndpoint = cpData.config?.control_plane_endpoint;
      const telemetryEndpoint = cpData.config?.telemetry_endpoint;
      const clusterType = cpData.config?.cluster_type;
      const proxyUrls = cpData.config?.proxy_urls || [];

      // Generate Docker deployment command
      const dockerCommand = `docker run -d --name kong-dp \\
  -e "KONG_ROLE=data_plane" \\
  -e "KONG_DATABASE=off" \\
  -e "KONG_VITALS=off" \\
  -e "KONG_CLUSTER_MTLS=pki" \\
  -e "KONG_CLUSTER_CONTROL_PLANE=${controlPlaneEndpoint?.replace('https://', '')}:443" \\
  -e "KONG_CLUSTER_SERVER_NAME=${controlPlaneEndpoint?.replace('https://', '')}" \\
  -e "KONG_CLUSTER_TELEMETRY_ENDPOINT=${telemetryEndpoint?.replace('https://', '')}:443" \\
  -e "KONG_CLUSTER_TELEMETRY_SERVER_NAME=${telemetryEndpoint?.replace('https://', '')}" \\
  -e "KONG_CLUSTER_CERT=/config/tls.crt" \\
  -e "KONG_CLUSTER_CERT_KEY=/config/tls.key" \\
  -e "KONG_LUA_SSL_TRUSTED_CERTIFICATE=system" \\
  -e "KONG_KONNECT_MODE=on" \\
  -p 8000:8000 \\
  -p 8443:8443 \\
  kong/kong-gateway:3.6`;

      res.json({
        success: true,
        controlPlane: {
          id: cpId,
          name: cpData.name,
          clusterType,
          controlPlaneEndpoint,
          telemetryEndpoint,
          proxyUrls
        },
        hasProxyUrl: proxyUrls.length > 0,
        deploymentInstructions: {
          docker: {
            title: "Docker Deployment",
            description: "Run a Kong Gateway data plane using Docker",
            prerequisites: [
              "Docker installed on your machine",
              "Generate TLS certificates from Kong Konnect UI",
              "Download certificates to ./config/ directory"
            ],
            steps: [
              "Go to Kong Konnect → Gateway Manager → Data Plane Nodes",
              "Click 'New Data Plane Node' → 'Linux (Docker)'",
              "Download the generated certificates (tls.crt, tls.key)",
              "Place certificates in a ./config/ directory",
              "Run the Docker command below"
            ],
            command: dockerCommand
          },
          kubernetes: {
            title: "Kubernetes Deployment",
            description: "Deploy Kong Gateway on Kubernetes using Helm",
            command: `helm repo add kong https://charts.konghq.com
helm repo update
helm install kong kong/kong --namespace kong --create-namespace \\
  --set ingressController.enabled=false \\
  --set env.role=data_plane \\
  --set env.database=off \\
  --set env.cluster_control_plane="${controlPlaneEndpoint?.replace('https://', '')}:443" \\
  --set env.cluster_telemetry_endpoint="${telemetryEndpoint?.replace('https://', '')}:443"`
          }
        },
        proxyAccessUrls: proxyUrls.length > 0 ? {
          timing: `${proxyUrls[0]?.protocol}://${proxyUrls[0]?.host}/api/timing/timestamp`,
          ternary: `${proxyUrls[0]?.protocol}://${proxyUrls[0]?.host}/api/ternary/convert`,
          phase: `${proxyUrls[0]?.protocol}://${proxyUrls[0]?.host}/api/phase/config/balanced`
        } : null
      });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to get deployment instructions" });
    }
  });

  // Generate deployment package with certificates (Admin only)
  app.post("/api/kong/control-planes/:cpId/generate-deployment", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const crypto = await import('crypto');

      // Generate self-signed certificate
      const { privateKey, publicKey } = crypto.generateKeyPairSync('rsa', {
        modulusLength: 2048,
        publicKeyEncoding: { type: 'spki', format: 'pem' },
        privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
      });

      // Create self-signed certificate using forge-like approach with Node.js
      const certInfo = {
        subject: '/CN=kong-dp/O=PlenumNET/C=US',
        issuer: '/CN=kong-dp/O=PlenumNET/C=US',
        serialNumber: crypto.randomBytes(16).toString('hex'),
        notBefore: new Date(),
        notAfter: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000 * 10) // 10 years
      };

      // Use openssl-like certificate generation via child process
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);
      const fs = await import('fs/promises');
      const path = await import('path');
      
      const tempDir = path.join('/tmp', `kong-certs-${Date.now()}`);
      await fs.mkdir(tempDir, { recursive: true });
      
      const keyPath = path.join(tempDir, 'tls.key');
      const certPath = path.join(tempDir, 'tls.crt');

      // Generate certificate using openssl
      await execAsync(`openssl req -new -x509 -nodes -newkey rsa:2048 -subj "/CN=kong-dp/O=PlenumNET/C=US" -keyout ${keyPath} -out ${certPath} -days 3650`);
      
      const tlsKey = await fs.readFile(keyPath, 'utf-8');
      const tlsCert = await fs.readFile(certPath, 'utf-8');

      // Upload certificate to Kong Konnect
      const uploadResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/dp-client-certificates`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ cert: tlsCert })
      });

      let certUploadResult = null;
      if (uploadResponse.ok) {
        certUploadResult = await uploadResponse.json();
      } else {
        const errorText = await uploadResponse.text();
        console.error("Certificate upload failed:", errorText);
      }

      // Get control plane details for docker-compose
      const cpResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}`, {
        headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
      });
      const cpData = await cpResponse.json();
      const controlPlaneEndpoint = cpData.config?.control_plane_endpoint?.replace('https://', '') || '';
      const telemetryEndpoint = cpData.config?.telemetry_endpoint?.replace('https://', '') || '';

      // Generate docker-compose.yml
      const dockerCompose = `version: '3.8'

services:
  kong-dp:
    image: kong/kong-gateway:3.6
    container_name: kong-plenumnet-dp
    restart: unless-stopped
    environment:
      - KONG_ROLE=data_plane
      - KONG_DATABASE=off
      - KONG_VITALS=off
      - KONG_CLUSTER_MTLS=pki
      - KONG_CLUSTER_CONTROL_PLANE=${controlPlaneEndpoint}:443
      - KONG_CLUSTER_SERVER_NAME=${controlPlaneEndpoint}
      - KONG_CLUSTER_TELEMETRY_ENDPOINT=${telemetryEndpoint}:443
      - KONG_CLUSTER_TELEMETRY_SERVER_NAME=${telemetryEndpoint}
      - KONG_CLUSTER_CERT=/etc/secrets/tls.crt
      - KONG_CLUSTER_CERT_KEY=/etc/secrets/tls.key
      - KONG_LUA_SSL_TRUSTED_CERTIFICATE=system
      - KONG_KONNECT_MODE=on
      - KONG_PROXY_LISTEN=0.0.0.0:8000, 0.0.0.0:8443 ssl
    ports:
      - "8000:8000"   # HTTP Proxy
      - "8443:8443"   # HTTPS Proxy
    volumes:
      - ./certs:/etc/secrets:ro
    healthcheck:
      test: ["CMD", "kong", "health"]
      interval: 30s
      timeout: 10s
      retries: 3

# PlenumNET Kong Gateway Data Plane
# Generated: ${new Date().toISOString()}
# Control Plane: ${cpData.name}
# 
# SETUP INSTRUCTIONS:
# 1. Save this file as docker-compose.yml
# 2. Create a 'certs' directory: mkdir certs
# 3. Save tls.crt and tls.key to the certs directory
# 4. Run: docker-compose up -d
# 5. Access PlenumNET APIs at: http://localhost:8000/api/timing/timestamp
`;

      // Generate deployment script
      const deployScript = `#!/bin/bash
# PlenumNET Kong Gateway Deployment Script
# Generated: ${new Date().toISOString()}

set -e

echo "🚀 Deploying PlenumNET Kong Gateway Data Plane..."

# Create directories
mkdir -p kong-plenumnet/certs

# Write certificates
cat > kong-plenumnet/certs/tls.crt << 'CERTEOF'
${tlsCert}CERTEOF

cat > kong-plenumnet/certs/tls.key << 'KEYEOF'
${tlsKey}KEYEOF

# Write docker-compose
cat > kong-plenumnet/docker-compose.yml << 'COMPOSEEOF'
${dockerCompose}COMPOSEEOF

# Set permissions
chmod 600 kong-plenumnet/certs/tls.key

# Deploy
cd kong-plenumnet
docker-compose up -d

echo ""
echo "✅ Kong Gateway Data Plane deployed successfully!"
echo ""
echo "🔗 Proxy URLs:"
echo "   HTTP:  http://localhost:8000"
echo "   HTTPS: https://localhost:8443"
echo ""
echo "📡 PlenumNET API Endpoints:"
echo "   Timing:    http://localhost:8000/api/timing/timestamp"
echo "   Ternary:   http://localhost:8000/api/ternary/convert"
echo "   Phase:     http://localhost:8000/api/phase/config/balanced"
echo "   Demo:      http://localhost:8000/api/demo/stats"
echo ""
echo "📊 View logs: docker-compose logs -f"
`;

      // Cleanup temp files
      await fs.rm(tempDir, { recursive: true, force: true });

      res.json({
        success: true,
        message: "Deployment package generated successfully",
        certificateUploaded: !!certUploadResult,
        certificateId: certUploadResult?.id,
        controlPlane: {
          id: cpId,
          name: cpData.name,
          endpoint: controlPlaneEndpoint
        },
        files: {
          "tls.crt": tlsCert,
          "tls.key": tlsKey,
          "docker-compose.yml": dockerCompose,
          "deploy.sh": deployScript
        },
        instructions: [
          "1. Copy deploy.sh to your server",
          "2. Make it executable: chmod +x deploy.sh",
          "3. Run: ./deploy.sh",
          "4. Access APIs at http://your-server:8000/api/timing/timestamp"
        ]
      });
    } catch (error) {
      console.error("Generate deployment error:", error);
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to generate deployment package" });
    }
  });

  // Deploy Kong to cloud platform (Render/Railway) via GitHub
  app.post("/api/kong/control-planes/:cpId/deploy-to-cloud", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const { platform = "render", owner, repo } = req.body;
      const token = req.adminUser?.githubToken;

      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      if (!owner || !repo) {
        return res.status(400).json({ error: "GitHub owner and repo required" });
      }

      // Generate certificates
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);
      const fs = await import('fs/promises');
      const path = await import('path');
      
      const tempDir = path.join('/tmp', `kong-cloud-${Date.now()}`);
      await fs.mkdir(tempDir, { recursive: true });
      
      const keyPath = path.join(tempDir, 'tls.key');
      const certPath = path.join(tempDir, 'tls.crt');

      await execAsync(`openssl req -new -x509 -nodes -newkey rsa:2048 -subj "/CN=kong-dp/O=PlenumNET/C=US" -keyout ${keyPath} -out ${certPath} -days 3650`);
      
      const tlsKey = await fs.readFile(keyPath, 'utf-8');
      const tlsCert = await fs.readFile(certPath, 'utf-8');

      // Upload certificate to Kong Konnect
      await fetch(`${KONG_API_BASE}/control-planes/${cpId}/dp-client-certificates`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ cert: tlsCert })
      });

      // Get control plane details
      const cpResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}`, {
        headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
      });
      const cpData = await cpResponse.json();
      const controlPlaneEndpoint = cpData.config?.control_plane_endpoint?.replace('https://', '') || '';
      const telemetryEndpoint = cpData.config?.telemetry_endpoint?.replace('https://', '') || '';

      // Create Dockerfile - private key loaded from env var at runtime (secure)
      const dockerfile = `FROM kong/kong-gateway:3.6

ENV KONG_ROLE=data_plane
ENV KONG_DATABASE=off
ENV KONG_VITALS=off
ENV KONG_CLUSTER_MTLS=pki
ENV KONG_LUA_SSL_TRUSTED_CERTIFICATE=system
ENV KONG_KONNECT_MODE=on
ENV KONG_PROXY_LISTEN=0.0.0.0:\${PORT:-8000}
ENV KONG_STATUS_LISTEN=0.0.0.0:8100

ENV KONG_CLUSTER_CONTROL_PLANE=${controlPlaneEndpoint}:443
ENV KONG_CLUSTER_SERVER_NAME=${controlPlaneEndpoint}
ENV KONG_CLUSTER_TELEMETRY_ENDPOINT=${telemetryEndpoint}:443
ENV KONG_CLUSTER_TELEMETRY_SERVER_NAME=${telemetryEndpoint}

RUN mkdir -p /etc/kong/certs

# Cert is baked in, key is loaded at runtime from secret env var
COPY tls.crt /etc/kong/certs/tls.crt
ENV KONG_CLUSTER_CERT=/etc/kong/certs/tls.crt

# Startup script to load private key from env var securely
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 8000 8100

HEALTHCHECK --interval=30s --timeout=10s CMD curl -sf http://localhost:8100/status || exit 1

ENTRYPOINT ["/entrypoint.sh"]
`;

      // Entrypoint script that loads TLS key from env var (not in git)
      const entrypointScript = `#!/bin/sh
set -e

# Write private key from secret env var to file
if [ -n "\$KONG_TLS_KEY" ]; then
  echo "\$KONG_TLS_KEY" > /etc/kong/certs/tls.key
  chmod 600 /etc/kong/certs/tls.key
  export KONG_CLUSTER_CERT_KEY=/etc/kong/certs/tls.key
else
  echo "ERROR: KONG_TLS_KEY environment variable is required"
  exit 1
fi

exec kong docker-start
`;

      // Create render.yaml - health check on status port
      const renderYaml = `services:
  - type: web
    name: kong-plenumnet
    runtime: docker
    dockerfilePath: ./kong-deploy/Dockerfile
    dockerContext: ./kong-deploy
    region: oregon
    plan: free
    envVars:
      - key: PORT
        value: 8000
      - key: KONG_TLS_KEY
        sync: false
`;

      // Push files to GitHub - NO private key!
      const filesToPush = [
        { path: "kong-deploy/Dockerfile", content: dockerfile },
        { path: "kong-deploy/entrypoint.sh", content: entrypointScript },
        { path: "kong-deploy/tls.crt", content: tlsCert },
        { path: "render.yaml", content: renderYaml }
      ];

      const pushErrors: string[] = [];
      for (const file of filesToPush) {
        const content = Buffer.from(file.content).toString('base64');
        
        // Check if file exists
        const checkResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${file.path}`, {
          headers: {
            "Authorization": `token ${token}`,
            "Accept": "application/vnd.github.v3+json"
          }
        });
        
        let sha: string | undefined;
        if (checkResponse.ok) {
          const existingFile = await checkResponse.json();
          sha = existingFile.sha;
        }

        const pushResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${file.path}`, {
          method: 'PUT',
          headers: {
            "Authorization": `token ${token}`,
            "Accept": "application/vnd.github.v3+json",
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            message: `Add Kong deployment: ${file.path}`,
            content,
            sha
          })
        });

        if (!pushResponse.ok) {
          const errData = await pushResponse.json().catch(() => ({}));
          pushErrors.push(`${file.path}: ${errData.message || pushResponse.statusText}`);
        }
      }

      if (pushErrors.length > 0) {
        return res.status(500).json({ 
          error: "Failed to push some files to GitHub", 
          details: pushErrors 
        });
      }

      // Cleanup
      await fs.rm(tempDir, { recursive: true, force: true });

      // Generate deploy URLs
      const renderDeployUrl = `https://render.com/deploy?repo=https://github.com/${owner}/${repo}`;
      const railwayDeployUrl = `https://railway.app/template?template=https://github.com/${owner}/${repo}`;

      res.json({
        success: true,
        message: "Deployment files pushed to GitHub!",
        platform,
        githubRepo: `https://github.com/${owner}/${repo}`,
        deployUrls: {
          render: renderDeployUrl,
          railway: railwayDeployUrl
        },
        controlPlane: {
          id: cpId,
          name: cpData.name,
          endpoint: controlPlaneEndpoint
        },
        // Include the private key for user to copy to cloud platform env vars
        tlsKey: tlsKey,
        instructions: [
          `1. Files pushed to https://github.com/${owner}/${repo}`,
          `2. Copy the TLS private key below`,
          `3. Click the deploy link for ${platform}`,
          `4. In the cloud platform, set KONG_TLS_KEY env var to the private key`,
          `5. Deploy the service`,
          `6. Your Kong proxy will be live at the provided URL`
        ]
      });
    } catch (error) {
      console.error("Cloud deployment error:", error);
      res.status(500).json({ error: error instanceof Error ? error.message : "Failed to deploy to cloud" });
    }
  });

  return httpServer;
}
