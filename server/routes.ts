import type { Express } from "express";
import { createServer, type Server } from "http";
import { storage } from "./storage";
import { randomUUID } from "crypto";
import { z } from "zod";
import { 
  compressData, 
  generateSensorData, 
  generateUserEvents, 
  generateLogEntries 
} from "./ternary";

const demoRunSchema = z.object({
  datasetName: z.enum(["sensor", "events", "logs"]),
  rowCount: z.number().int().min(1).max(10000).default(100)
});

export async function registerRoutes(
  httpServer: Server,
  app: Express
): Promise<Server> {
  
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
        fileType: z.enum(["json", "csv"]),
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

  return httpServer;
}
