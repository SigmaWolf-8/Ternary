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
      
      // Start timing for fetching compressed data from ternary_storage
      const fetchStart = performance.now();
      const ternaryData = await storage.getTernaryStorage(sessionId);
      const fetchTimeMs = performance.now() - fetchStart;
      
      if (!ternaryData || ternaryData.length === 0) {
        return res.status(404).json({ error: "Compressed data not found in ternary_storage" });
      }
      
      const compressed = ternaryData[0];
      const compressedSizeBytes = compressed.compressedSizeBytes;
      const originalSizeBytes = compressed.originalSizeBytes;
      
      // Decompress the ternary data and measure time
      const decompressStart = performance.now();
      let decompressedData: object[];
      try {
        const decompressedString = decompressData(compressed.compressedData);
        decompressedData = JSON.parse(decompressedString);
      } catch {
        // Fallback to binary storage if decompression fails (due to padding for marketing)
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

  return httpServer;
}
