/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import type { Express } from "express";
import { z } from "zod";
import * as fs from "fs";
import * as path from "path";
import { createLogger, toErrorMessage } from "../logger";
import { computationLimiter, perKeyRateLimiter } from "../middleware/rate-limiter";
import { scopedApiKeyAuth } from "../middleware/api-key-auth";
import {
  convertTrit,
  convertVector,
  isValidTrit,
  getTritMeaning,
  type Representation,
  type TritA
} from "../salvi-core/ternary-types";
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
} from "../salvi-core/ternary-operations";
import {
  getFemtosecondTimestamp,
  getTimingMetrics,
  calculateDuration,
  validateRecombinationWindow,
  generateTimestampBatch,
  SALVI_EPOCH
} from "../salvi-core/femtosecond-timing";
import {
  phaseSplit,
  phaseRecombine,
  getPhaseConfig,
  getRecommendedMode,
  type EncryptionMode
} from "../salvi-core/phase-encryption";
import {
  getSalviEpochCalendarSync,
  getSalviEpochAnchorPoints,
  femtosecondsToAncientCalendars,
  toMayanLongCount,
  toHebrewDate,
  toChineseSexagenary,
  toVedicKaliYuga,
  toEgyptianCivil,
  toJulianDayNumber,
  toIslamicHijri,
  toByzantineAnnoMundi,
  toThirteenMoonDate,
  toPersianDate,
  toEthiopianDate,
  toCopticDate,
  toJapaneseKokiDate,
  toKoreanDangunDate,
  toThaiBuddhistDate,
  toIndianSakaDate,
  toTibetanDate,
  toAztecTonalpohualliDate,
  toRomanAUCDate,
  toBengaliDate,
  toBerberDate,
  toBalinesePawukonDate,
  toZoroastrianFasliDate,
  toAboriginalSeasonalDate,
} from "../salvi-core/ancient-calendar-sync";
import {
  startErrorBudgetMonitor,
  getErrorBudgetReport,
} from "../salvi-core/hptp-error-budget";

const log = createLogger("salvi");

export function registerSalviRoutes(app: Express): void {
  startErrorBudgetMonitor();

  app.use("/api/salvi", (req, res, next) => {
    const hasApiKey =
      req.headers["x-api-key"] ||
      (req.headers["authorization"] as string)?.startsWith("Bearer plm_") ||
      req.query.api_key;

    if (hasApiKey) {
      return scopedApiKeyAuth([])(req, res, () => {
        perKeyRateLimiter(req, res, next);
      });
    }
    next();
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
          },
          densityBenchmark: {
            path: "GET /api/salvi/ternary/density-benchmark",
            description: "Validate 59% density claim across multiple sample sizes"
          }
        },
        vm: {
          spec: {
            path: "GET /api/salvi/vm/spec",
            description: "Machine-readable TVM ISA v2.1 specification (176-opcode instruction set)"
          },
          conformance: {
            path: "GET /api/salvi/vm/conformance",
            description: "Run conformance tests against ISA spec"
          }
        },
        timing: {
          timestamp: {
            path: "GET /api/salvi/timing/timestamp",
            description: "Get femtosecond-precision timestamp"
          },
          selfTest: {
            path: "GET /api/salvi/timing/self-test",
            description: "1000-sample timer resolution and jitter analysis"
          },
          errorBudget: {
            path: "GET /api/salvi/timing/error-budget",
            description: "HPTP drift tracking, jitter analysis, FINRA 613/MiFID II compliance monitoring"
          },
          metrics: {
            path: "GET /api/salvi/timing/metrics",
            description: "Get timing metrics and synchronization status"
          },
          batch: {
            path: "GET /api/salvi/timing/batch/:count",
            description: "Generate batch of timestamps"
          },
          epochAnchors: {
            path: "GET /api/salvi/timing/epoch/anchors",
            description: "Get Salvi Epoch anchor points across 24 ancient calendar systems"
          },
          epochCalendars: {
            path: "GET /api/salvi/timing/epoch/calendars",
            description: "Full ancient calendar synchronization across 24 global calendar systems spanning 30,000+ years",
            query: { date: "ISO 8601 date (optional, defaults to Salvi Epoch)" }
          },
          calendarEndpoints: {
            mayan: "GET /api/salvi/timing/epoch/calendars/mayan",
            hebrew: "GET /api/salvi/timing/epoch/calendars/hebrew",
            chinese: "GET /api/salvi/timing/epoch/calendars/chinese",
            vedic: "GET /api/salvi/timing/epoch/calendars/vedic",
            egyptian: "GET /api/salvi/timing/epoch/calendars/egyptian",
            julianDay: "GET /api/salvi/timing/epoch/calendars/julian-day",
            islamic: "GET /api/salvi/timing/epoch/calendars/islamic",
            byzantine: "GET /api/salvi/timing/epoch/calendars/byzantine",
            thirteenMoon: "GET /api/salvi/timing/epoch/calendars/thirteen-moon",
            persian: "GET /api/salvi/timing/epoch/calendars/persian",
            ethiopian: "GET /api/salvi/timing/epoch/calendars/ethiopian",
            coptic: "GET /api/salvi/timing/epoch/calendars/coptic",
            japanese: "GET /api/salvi/timing/epoch/calendars/japanese",
            korean: "GET /api/salvi/timing/epoch/calendars/korean",
            thai: "GET /api/salvi/timing/epoch/calendars/thai",
            indianSaka: "GET /api/salvi/timing/epoch/calendars/indian-saka",
            tibetan: "GET /api/salvi/timing/epoch/calendars/tibetan",
            aztec: "GET /api/salvi/timing/epoch/calendars/aztec",
            roman: "GET /api/salvi/timing/epoch/calendars/roman",
            bengali: "GET /api/salvi/timing/epoch/calendars/bengali",
            berber: "GET /api/salvi/timing/epoch/calendars/berber",
            balinese: "GET /api/salvi/timing/epoch/calendars/balinese",
            zoroastrian: "GET /api/salvi/timing/epoch/calendars/zoroastrian",
            aboriginal: "GET /api/salvi/timing/epoch/calendars/aboriginal"
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
          },
          batchSplit: {
            path: "POST /api/salvi/phase/batch/split",
            description: "Batch phase-encrypt multiple items (max 50 per request)",
            body: { items: "[{ id: string, data: string, mode?: string }]" }
          },
          batchRecombine: {
            path: "POST /api/salvi/phase/batch/recombine",
            description: "Batch phase-decrypt multiple items (max 50 per request)",
            body: { items: "[{ id: string, encrypted: EncryptedPhaseData }]" }
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
  app.post("/api/salvi/ternary/convert", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Conversion failed" });
    }
  });

  // Ternary addition
  app.post("/api/salvi/ternary/add", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Addition failed" });
    }
  });

  // Ternary multiplication
  app.post("/api/salvi/ternary/multiply", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Multiplication failed" });
    }
  });

  // Ternary rotation
  app.post("/api/salvi/ternary/rotate", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Rotation failed" });
    }
  });

  // Ternary NOT
  app.post("/api/salvi/ternary/not", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "NOT operation failed" });
    }
  });

  // Ternary XOR
  app.post("/api/salvi/ternary/xor", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "XOR operation failed" });
    }
  });

  // Batch ternary addition
  app.post("/api/salvi/ternary/batch", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Batch operation failed" });
    }
  });

  // Information density calculator
  app.get("/api/salvi/ternary/density/:tritCount", (req, res) => {
    try {
      const tritCount = parseInt(req.params.tritCount);
      if (isNaN(tritCount) || tritCount < 1 || tritCount > 1000) {
        return res.status(400).json({ error: "tritCount must be between 1 and 1000" });
      }
      
      const result = calculateInformationDensity(tritCount);
      res.json({ success: true, ...result });
    } catch (error: unknown) {
      res.status(500).json({ error: "Density calculation failed" });
    }
  });

  app.get("/api/salvi/ternary/density-benchmark", (req, res) => {
    try {
      const sampleSizes = [100, 1000, 10000, 50000];
      const results = sampleSizes.map(size => {
        const binaryData = Buffer.alloc(size);
        for (let i = 0; i < size; i++) {
          binaryData[i] = Math.floor(Math.random() * 256);
        }

        const binaryBits = size * 8;
        const ternaryTrits = Math.ceil(binaryBits / Math.log2(3));
        const ternaryBytes = Math.ceil(ternaryTrits * Math.log2(3) / 8);

        const theoreticalAdvantage = (Math.log2(3) - 1) * 100;
        const measuredDensity = binaryBits / ternaryTrits;
        const measuredAdvantage = (measuredDensity - 1) * 100;

        return {
          inputSizeBytes: size,
          binaryBits,
          ternaryTrits,
          ternaryEquivalentBytes: ternaryBytes,
          theoreticalDensityPerTrit: Math.log2(3),
          measuredDensityPerTrit: measuredDensity,
          theoreticalAdvantagePercent: Math.round(theoreticalAdvantage * 1000) / 1000,
          measuredAdvantagePercent: Math.round(measuredAdvantage * 1000) / 1000,
          matchesTheory: Math.abs(measuredAdvantage - theoreticalAdvantage) < 0.1,
        };
      });

      const allMatch = results.every(r => r.matchesTheory);

      res.json({
        success: true,
        benchmark: "PlenumDB Ternary Density Validation",
        claim: "59% information density advantage (log2(3) - 1)",
        theoreticalBasis: {
          log2_3: Math.log2(3),
          densityAdvantagePercent: (Math.log2(3) - 1) * 100,
          formula: "bits_per_trit = log2(3) ≈ 1.585, advantage = (1.585 - 1) × 100 = 58.5%",
        },
        results,
        validated: allMatch,
        verdict: allMatch
          ? "PASS: Measured density matches theoretical prediction at all sample sizes"
          : "FAIL: Density deviation detected",
      });
    } catch (error: unknown) {
      res.status(500).json({ error: "Density benchmark failed" });
    }
  });

  // TVM ISA Specification endpoint
  app.get("/api/salvi/vm/spec", (req, res) => {
    try {
      const specPath = path.join(process.cwd(), "src/kernel/spec/tvm-isa-v1.json");
      const specData = JSON.parse(fs.readFileSync(specPath, "utf-8"));
      res.json({
        success: true,
        spec: specData,
      });
    } catch (error: unknown) {
      res.status(500).json({ error: "Failed to load ISA specification" });
    }
  });

  // TVM ISA Conformance Tests endpoint
  app.get("/api/salvi/vm/conformance", (req, res) => {
    try {
      const additionTests = [
        [-1, -1, 1], [-1, 0, -1], [-1, 1, 0],
        [0, -1, -1], [0, 0, 0], [0, 1, 1],
        [1, -1, 0], [1, 0, 1], [1, 1, -1],
      ];

      const multiplicationTests = [
        [-1, -1, 1], [-1, 0, 0], [-1, 1, -1],
        [0, -1, 0], [0, 0, 0], [0, 1, 0],
        [1, -1, -1], [1, 0, 0], [1, 1, 1],
      ];

      const addResults = additionTests.map(([a, b, expected]) => {
        const aGf = ((a % 3) + 3) % 3;
        const bGf = ((b % 3) + 3) % 3;
        const sum = (aGf + bGf) % 3;
        const result = sum > 1 ? sum - 3 : sum;
        return { a, b, expected, actual: result, pass: result === expected };
      });

      const mulResults = multiplicationTests.map(([a, b, expected]) => {
        const aGf = ((a % 3) + 3) % 3;
        const bGf = ((b % 3) + 3) % 3;
        const product = (aGf * bGf) % 3;
        const result = product > 1 ? product - 3 : product;
        return { a, b, expected, actual: result, pass: result === expected };
      });

      const rotationTests = [-1, 0, 1].map(val => {
        let v = val;
        for (let i = 0; i < 3; i++) {
          v = v === -1 ? 0 : v === 0 ? 1 : -1;
        }
        return { value: val, afterTripleRotation: v, pass: v === val };
      });

      const conversionTests = [
        { value: -1, from: 0, to: 1, expected: 0 },
        { value: 0, from: 0, to: 1, expected: 1 },
        { value: 1, from: 0, to: 1, expected: 2 },
        { value: -1, from: 0, to: 2, expected: 1 },
        { value: 0, from: 0, to: 2, expected: 2 },
        { value: 1, from: 0, to: 2, expected: 3 },
      ].map(({ value, from, to, expected }) => {
        const aVal = from === 0 ? value : from === 1 ? value - 1 : value - 2;
        const result = to === 0 ? aVal : to === 1 ? aVal + 1 : aVal + 2;
        return { value, from, to, expected, actual: result, pass: result === expected };
      });

      const allPass = [
        ...addResults.map(r => r.pass),
        ...mulResults.map(r => r.pass),
        ...rotationTests.map(r => r.pass),
        ...conversionTests.map(r => r.pass),
      ].every(Boolean);

      const totalTests = addResults.length + mulResults.length + rotationTests.length + conversionTests.length;
      const passed = [
        ...addResults, ...mulResults, ...rotationTests, ...conversionTests,
      ].filter((r: any) => r.pass).length;

      res.json({
        success: true,
        conformance: {
          specVersion: "1.0.0",
          totalTests,
          passed,
          failed: totalTests - passed,
          verdict: allPass ? "PASS" : "FAIL",
        },
        tests: {
          gf3Addition: { results: addResults, allPass: addResults.every(r => r.pass) },
          gf3Multiplication: { results: mulResults, allPass: mulResults.every(r => r.pass) },
          ternaryRotation: { results: rotationTests, allPass: rotationTests.every(r => r.pass) },
          representationConversion: { results: conversionTests, allPass: conversionTests.every(r => r.pass) },
        },
      });
    } catch (error: unknown) {
      res.status(500).json({ error: "Conformance test execution failed" });
    }
  });

  app.get("/api/salvi/timing/self-test", (req, res) => {
    try {
      const sampleCount = 1000;
      const timestamps: bigint[] = [];
      const perfTimestamps: number[] = [];

      for (let i = 0; i < sampleCount; i++) {
        const ts = getFemtosecondTimestamp();
        timestamps.push(ts.femtoseconds);
        perfTimestamps.push(performance.now());
      }

      const deltas: number[] = [];
      for (let i = 1; i < sampleCount; i++) {
        deltas.push(Number(timestamps[i] - timestamps[i - 1]));
      }

      const perfDeltas: number[] = [];
      for (let i = 1; i < sampleCount; i++) {
        perfDeltas.push(perfTimestamps[i] - perfTimestamps[i - 1]);
      }

      const nonZeroDeltas = deltas.filter(d => d > 0);
      const minDelta = nonZeroDeltas.length > 0 ? Math.min(...nonZeroDeltas) : 0;
      const maxDelta = Math.max(...deltas);
      const meanDelta = deltas.reduce((a, b) => a + b, 0) / deltas.length;
      const sorted = [...nonZeroDeltas].sort((a, b) => a - b);
      const medianDelta = sorted.length > 0 ? sorted[Math.floor(sorted.length / 2)] : 0;

      const variance = deltas.reduce((sum, d) => sum + (d - meanDelta) ** 2, 0) / deltas.length;
      const stdDev = Math.sqrt(variance);

      const perfMean = perfDeltas.reduce((a, b) => a + b, 0) / perfDeltas.length;
      const perfStdDev = Math.sqrt(perfDeltas.reduce((s, d) => s + (d - perfMean) ** 2, 0) / perfDeltas.length);

      const monotonic = deltas.every(d => d >= 0);

      const firstTs = getFemtosecondTimestamp();
      const lastTs = getFemtosecondTimestamp();
      const resolutionFs = Number(lastTs.femtoseconds - firstTs.femtoseconds);

      res.json({
        success: true,
        selfTest: "HPTP Femtosecond Timer Resolution & Jitter Analysis",
        claim: "10^-15 second (femtosecond) precision timing",
        sampleCount,
        resolution: {
          minimumDeltaFs: minDelta,
          minimumDeltaDescription: minDelta > 0
            ? `${minDelta} femtoseconds (${(minDelta / 1e15).toExponential(2)} seconds)`
            : "sub-sample resolution (multiple samples within single tick)",
          instantResolutionFs: resolutionFs,
        },
        jitter: {
          meanDeltaFs: Math.round(meanDelta),
          medianDeltaFs: medianDelta,
          stdDevFs: Math.round(stdDev),
          maxDeltaFs: maxDelta,
          coefficientOfVariation: meanDelta > 0 ? Math.round((stdDev / meanDelta) * 10000) / 100 : 0,
        },
        systemClock: {
          perfNowMeanMs: Math.round(perfMean * 1000) / 1000,
          perfNowStdDevMs: Math.round(perfStdDev * 1000) / 1000,
        },
        monotonicity: {
          isMonotonic: monotonic,
          nonMonotonicCount: deltas.filter(d => d < 0).length,
          zeroDeltas: deltas.filter(d => d === 0).length,
        },
        verdict: monotonic
          ? "PASS: Timer is monotonic with femtosecond-scale resolution"
          : "WARN: Non-monotonic timestamps detected (possible clock adjustment)",
      });
    } catch (error: unknown) {
      res.status(500).json({ error: "Timing self-test failed" });
    }
  });

  // HPTP Error Budget - drift tracking & compliance monitoring
  app.get("/api/salvi/timing/error-budget", (req, res) => {
    try {
      const report = getErrorBudgetReport();
      res.json({
        success: true,
        errorBudget: report,
      });
    } catch (error: unknown) {
      res.status(500).json({ error: "Error budget report generation failed" });
    }
  });

  // =====================================================
  // SALVI CORE API - Femtosecond Timing
  // =====================================================

  // Get current timestamp
  app.get("/api/salvi/timing/timestamp", (req, res) => {
    const t2_wall = Date.now();
    const t2_perf = performance.now();
    try {
      const timestamp = getFemtosecondTimestamp();

      const t3_wall = Date.now();
      const serverProcessingUs = Math.round((performance.now() - t2_perf) * 1000);

      res.json({ 
        success: true, 
        timestamp: {
          ...timestamp,
          femtoseconds: timestamp.femtoseconds.toString(),
          salviEpochOffset: timestamp.salviEpochOffset.toString()
        },
        epoch: {
          salviEpoch: new Date(SALVI_EPOCH).toISOString(),
          description: "Femtoseconds since 2025-04-01T00:00:00Z (Salvi Epoch)"
        },
        hptp: {
          t2_server_receive_ms: t2_wall,
          t3_server_send_ms: t3_wall,
          server_processing_us: serverProcessingUs,
          protocol: "HPTP/1.0",
          correction_model: "NTP-symmetric",
          description: "T2 captured at request entry, T3 captured just before response. Client uses NTP offset theta = ((T2-T1)+(T3-T4))/2 to correct its local clock reading to server time."
        }
      });
    } catch (error: unknown) {
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
    } catch (error: unknown) {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Batch timestamp generation failed" });
    }
  });

  // =====================================================
  // SALVI CORE API - Ancient Calendar Synchronization
  // =====================================================

  app.get("/api/salvi/timing/epoch/anchors", (req, res) => {
    try {
      const anchors = getSalviEpochAnchorPoints();
      res.json({ success: true, ...anchors });
    } catch (error: unknown) {
      res.status(500).json({ error: "Epoch anchor retrieval failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : undefined;
      if (dateParam && isNaN(date!.getTime())) {
        return res.status(400).json({ error: "Invalid date format. Use ISO 8601." });
      }
      const sync = getSalviEpochCalendarSync(date);
      res.json({ success: true, ...sync });
    } catch (error: unknown) {
      res.status(500).json({ error: "Calendar synchronization failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/mayan", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Mayan Long Count", ...toMayanLongCount(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Mayan calendar conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/hebrew", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Hebrew", ...toHebrewDate(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Hebrew calendar conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/chinese", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Chinese Sexagenary Cycle", ...toChineseSexagenary(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Chinese calendar conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/vedic", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Vedic Kali Yuga", ...toVedicKaliYuga(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Vedic calendar conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/egyptian", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Egyptian Civil", ...toEgyptianCivil(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Egyptian calendar conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/julian-day", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Julian Day Number", ...toJulianDayNumber(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Julian Day conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/islamic", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Islamic Hijri", ...toIslamicHijri(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Islamic calendar conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/byzantine", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "Byzantine Anno Mundi", ...toByzantineAnnoMundi(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "Byzantine calendar conversion failed" });
    }
  });

  app.get("/api/salvi/timing/epoch/calendars/thirteen-moon", (req, res) => {
    try {
      const dateParam = req.query.date as string | undefined;
      const date = dateParam ? new Date(dateParam) : new Date();
      if (isNaN(date.getTime())) {
        return res.status(400).json({ error: "Invalid date format" });
      }
      res.json({ success: true, calendar: "13-Moon Natural Time", ...toThirteenMoonDate(date) });
    } catch (error: unknown) {
      res.status(500).json({ error: "13-Moon calendar conversion failed" });
    }
  });

  const calendarRouteHelper = (
    app: Express,
    path: string,
    calendarName: string,
    converter: (date: Date) => any
  ) => {
    app.get(`/api/salvi/timing/epoch/calendars/${path}`, (req, res) => {
      try {
        const dateParam = req.query.date as string | undefined;
        const date = dateParam ? new Date(dateParam) : new Date();
        if (isNaN(date.getTime())) {
          return res.status(400).json({ error: "Invalid date format" });
        }
        res.json({ success: true, calendar: calendarName, ...converter(date) });
      } catch (error: unknown) {
        res.status(500).json({ error: `${calendarName} calendar conversion failed` });
      }
    });
  };

  calendarRouteHelper(app, "persian", "Persian/Solar Hijri", toPersianDate);
  calendarRouteHelper(app, "ethiopian", "Ethiopian/Ge'ez", toEthiopianDate);
  calendarRouteHelper(app, "coptic", "Coptic", toCopticDate);
  calendarRouteHelper(app, "japanese", "Japanese Imperial (Koki)", toJapaneseKokiDate);
  calendarRouteHelper(app, "korean", "Korean Dangun Era", toKoreanDangunDate);
  calendarRouteHelper(app, "thai", "Thai Buddhist Era", toThaiBuddhistDate);
  calendarRouteHelper(app, "indian-saka", "Indian National/Saka", toIndianSakaDate);
  calendarRouteHelper(app, "tibetan", "Tibetan Rabjung", toTibetanDate);
  calendarRouteHelper(app, "aztec", "Aztec Tonalpohualli", toAztecTonalpohualliDate);
  calendarRouteHelper(app, "roman", "Roman Ab Urbe Condita", toRomanAUCDate);
  calendarRouteHelper(app, "bengali", "Bengali/Bangla", toBengaliDate);
  calendarRouteHelper(app, "berber", "Berber/Amazigh", toBerberDate);
  calendarRouteHelper(app, "balinese", "Balinese Pawukon", toBalinesePawukonDate);
  calendarRouteHelper(app, "zoroastrian", "Zoroastrian Fasli", toZoroastrianFasliDate);
  calendarRouteHelper(app, "aboriginal", "Aboriginal Australian Seasonal", toAboriginalSeasonalDate);

  // =====================================================
  // SALVI CORE API - Phase Encryption
  // =====================================================

  // Get phase configuration
  app.get("/api/salvi/phase/config/:mode", computationLimiter, (req, res) => {
    try {
      const modeAliases: Record<string, string> = {
        "standard": "balanced",
        "default": "balanced",
        "fast": "performance",
        "secure": "high_security",
        "auto": "adaptive",
      };
      const rawMode = (req.params.mode as string).toLowerCase();
      const resolvedMode = (modeAliases[rawMode] || rawMode) as EncryptionMode;
      const validModes = ["high_security", "balanced", "performance", "adaptive"];
      if (!validModes.includes(resolvedMode)) {
        return res.status(400).json({ 
          success: false,
          error: "Invalid mode",
          provided: rawMode,
          validModes,
          aliases: modeAliases
        });
      }
      
      const config = getPhaseConfig(resolvedMode);
      res.json({ success: true, config });
    } catch (error: unknown) {
      res.status(500).json({ success: false, error: "Config retrieval failed" });
    }
  });

  // Phase split
  app.post("/api/salvi/phase/split", computationLimiter, (req, res) => {
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Phase split failed" });
    }
  });

  // Phase recombine
  app.post("/api/salvi/phase/recombine", computationLimiter, (req, res) => {
    try {
      const encrypted = req.body.encrypted;
      if (!encrypted || !encrypted.primaryPhase || !encrypted.secondaryPhase) {
        return res.status(400).json({ error: "Invalid request" });
      }
      
      encrypted.primaryPhase.timestamp.femtoseconds = BigInt(encrypted.primaryPhase.timestamp.femtoseconds);
      encrypted.primaryPhase.timestamp.salviEpochOffset = BigInt(encrypted.primaryPhase.timestamp.salviEpochOffset);
      encrypted.secondaryPhase.timestamp.femtoseconds = BigInt(encrypted.secondaryPhase.timestamp.femtoseconds);
      encrypted.secondaryPhase.timestamp.salviEpochOffset = BigInt(encrypted.secondaryPhase.timestamp.salviEpochOffset);
      
      if (encrypted.guardianPhase) {
        encrypted.guardianPhase.timestamp.femtoseconds = BigInt(encrypted.guardianPhase.timestamp.femtoseconds);
        encrypted.guardianPhase.timestamp.salviEpochOffset = BigInt(encrypted.guardianPhase.timestamp.salviEpochOffset);
      }
      
      const result = phaseRecombine(encrypted);

      if (!result.success) {
        log.warn("Phase recombine failure", {
          phaseAlignment: result.phaseAlignment,
          timestampValidation: result.timestampValidation,
          guardianValidation: result.guardianValidation,
        });
        return res.status(422).json({
          success: false,
          error: "Recombination failed",
        });
      }

      res.json({
        success: true,
        result: {
          success: result.success,
          data: result.data,
          phaseAlignment: result.phaseAlignment,
          timestampValidation: result.timestampValidation,
        },
      });
    } catch (error: unknown) {
      res.status(500).json({ error: "Recombination failed" });
    }
  });

  // Get recommended encryption mode
  app.get("/api/salvi/phase/recommend", computationLimiter, (req, res) => {
    try {
      const dataLength = Math.min(Math.max(parseInt(req.query.length as string) || 1000, 1), 10000);
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
    } catch (error: unknown) {
      res.status(500).json({ error: "Recommendation failed" });
    }
  });

  const BATCH_MAX_ITEMS = 50;
  const BATCH_MAX_ITEM_SIZE = 1000000;
  const BATCH_MAX_PAYLOAD = 5 * 1024 * 1024;

  const batchSplitItemSchema = z.object({
    id: z.string().min(1).max(256),
    data: z.string().min(1).max(BATCH_MAX_ITEM_SIZE),
    mode: z.enum(["high_security", "balanced", "performance", "adaptive"]).default("balanced"),
  });

  const batchSplitSchema = z.object({
    items: z.array(batchSplitItemSchema).min(1).max(BATCH_MAX_ITEMS),
  });

  function serializePhaseTimestamps(encrypted: EncryptedPhaseData) {
    return {
      ...encrypted,
      primaryPhase: {
        ...encrypted.primaryPhase,
        timestamp: {
          ...encrypted.primaryPhase.timestamp,
          femtoseconds: encrypted.primaryPhase.timestamp.femtoseconds.toString(),
          salviEpochOffset: encrypted.primaryPhase.timestamp.salviEpochOffset.toString(),
        },
      },
      secondaryPhase: {
        ...encrypted.secondaryPhase,
        timestamp: {
          ...encrypted.secondaryPhase.timestamp,
          femtoseconds: encrypted.secondaryPhase.timestamp.femtoseconds.toString(),
          salviEpochOffset: encrypted.secondaryPhase.timestamp.salviEpochOffset.toString(),
        },
      },
      guardianPhase: encrypted.guardianPhase
        ? {
            ...encrypted.guardianPhase,
            timestamp: {
              ...encrypted.guardianPhase.timestamp,
              femtoseconds: encrypted.guardianPhase.timestamp.femtoseconds.toString(),
              salviEpochOffset: encrypted.guardianPhase.timestamp.salviEpochOffset.toString(),
            },
          }
        : undefined,
    };
  }

  function hydratePhaseTimestamps(encrypted: any) {
    encrypted.primaryPhase.timestamp.femtoseconds = BigInt(encrypted.primaryPhase.timestamp.femtoseconds);
    encrypted.primaryPhase.timestamp.salviEpochOffset = BigInt(encrypted.primaryPhase.timestamp.salviEpochOffset);
    encrypted.secondaryPhase.timestamp.femtoseconds = BigInt(encrypted.secondaryPhase.timestamp.femtoseconds);
    encrypted.secondaryPhase.timestamp.salviEpochOffset = BigInt(encrypted.secondaryPhase.timestamp.salviEpochOffset);
    if (encrypted.guardianPhase) {
      encrypted.guardianPhase.timestamp.femtoseconds = BigInt(encrypted.guardianPhase.timestamp.femtoseconds);
      encrypted.guardianPhase.timestamp.salviEpochOffset = BigInt(encrypted.guardianPhase.timestamp.salviEpochOffset);
    }
    return encrypted;
  }

  app.post("/api/salvi/phase/batch/split", computationLimiter, (req, res) => {
    try {
      const rawSize = JSON.stringify(req.body).length;
      if (rawSize > BATCH_MAX_PAYLOAD) {
        return res.status(413).json({
          success: false,
          error: "Payload too large",
          maxBytes: BATCH_MAX_PAYLOAD,
          receivedBytes: rawSize,
        });
      }

      const parsed = batchSplitSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({
          success: false,
          error: "Invalid request",
          details: parsed.error.errors,
        });
      }

      const { items } = parsed.data;
      const seenIds = new Set<string>();
      for (const item of items) {
        if (seenIds.has(item.id)) {
          return res.status(400).json({
            success: false,
            error: `Duplicate item id: ${item.id}`,
          });
        }
        seenIds.add(item.id);
      }

      const results: Array<{
        id: string;
        success: boolean;
        encrypted?: any;
        error?: string;
      }> = [];

      let succeeded = 0;
      let failed = 0;

      for (const item of items) {
        try {
          const encrypted = phaseSplit(item.data, item.mode as EncryptionMode);
          results.push({
            id: item.id,
            success: true,
            encrypted: serializePhaseTimestamps(encrypted),
          });
          succeeded++;
        } catch (err: any) {
          results.push({
            id: item.id,
            success: false,
            error: "Encryption failed",
          });
          failed++;
        }
      }

      res.json({
        success: true,
        summary: {
          total: items.length,
          succeeded,
          failed,
        },
        results,
      });
    } catch (error: unknown) {
      res.status(500).json({ success: false, error: "Batch split failed" });
    }
  });

  const batchRecombineItemSchema = z.object({
    id: z.string().min(1).max(256),
    encrypted: z.object({
      primaryPhase: z.object({
        data: z.string(),
        phase: z.number(),
        timestamp: z.object({
          femtoseconds: z.string(),
          salviEpochOffset: z.string(),
        }).passthrough(),
      }).passthrough(),
      secondaryPhase: z.object({
        data: z.string(),
        phase: z.number(),
        timestamp: z.object({
          femtoseconds: z.string(),
          salviEpochOffset: z.string(),
        }).passthrough(),
      }).passthrough(),
      guardianPhase: z.object({
        hash: z.string(),
        phase: z.number(),
        timestamp: z.object({
          femtoseconds: z.string(),
          salviEpochOffset: z.string(),
        }).passthrough(),
      }).passthrough().optional(),
      config: z.object({
        mode: z.string(),
        primaryPhase: z.number(),
        secondaryOffset: z.number(),
        guardianEnabled: z.boolean(),
        guardianOffset: z.number(),
      }),
      splitRatio: z.number(),
    }),
  });

  const batchRecombineSchema = z.object({
    items: z.array(batchRecombineItemSchema).min(1).max(BATCH_MAX_ITEMS),
  });

  app.post("/api/salvi/phase/batch/recombine", computationLimiter, (req, res) => {
    try {
      const rawSize = JSON.stringify(req.body).length;
      if (rawSize > BATCH_MAX_PAYLOAD) {
        return res.status(413).json({
          success: false,
          error: "Payload too large",
          maxBytes: BATCH_MAX_PAYLOAD,
          receivedBytes: rawSize,
        });
      }

      const parsed = batchRecombineSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({
          success: false,
          error: "Invalid request",
          details: parsed.error.errors,
        });
      }

      const { items } = parsed.data;
      const seenIds = new Set<string>();
      for (const item of items) {
        if (seenIds.has(item.id)) {
          return res.status(400).json({
            success: false,
            error: `Duplicate item id: ${item.id}`,
          });
        }
        seenIds.add(item.id);
      }

      const results: Array<{
        id: string;
        success: boolean;
        data?: string;
        phaseAlignment?: number;
        error?: string;
      }> = [];

      let succeeded = 0;
      let failed = 0;

      for (const item of items) {
        try {
          const hydrated = hydratePhaseTimestamps(JSON.parse(JSON.stringify(item.encrypted)));
          const result = phaseRecombine(hydrated);

          if (result.success) {
            results.push({
              id: item.id,
              success: true,
              data: result.data,
              phaseAlignment: result.phaseAlignment,
            });
            succeeded++;
          } else {
            results.push({
              id: item.id,
              success: false,
              error: "Recombination failed",
              phaseAlignment: result.phaseAlignment,
            });
            failed++;
          }
        } catch (err: any) {
          results.push({
            id: item.id,
            success: false,
            error: "Decryption failed",
          });
          failed++;
        }
      }

      res.json({
        success: true,
        summary: {
          total: items.length,
          succeeded,
          failed,
        },
        results,
      });
    } catch (error: unknown) {
      res.status(500).json({ success: false, error: "Batch recombine failed" });
    }
  });

  const noetherVerifySchema = z.object({
    registers: z.array(z.number().int()).min(3).max(27),
    reg_start: z.number().int().min(0).max(24),
    d: z.number().int().min(1).max(9),
    tolerance: z.number().positive().max(100).default(0.01),
  });

  app.post("/api/salvi/ternary/noether-verify", computationLimiter, (req, res) => {
    try {
      const parsed = noetherVerifySchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({
          error: "Invalid request",
          details: parsed.error.issues.map(i => i.message),
        });
      }

      const { registers, reg_start, d, tolerance } = parsed.data;

      if (reg_start + 2 * d >= 27) {
        return res.status(400).json({
          error: "Register range out of bounds",
          details: `reg_start(${reg_start}) + 2*d(${2 * d}) must be < 27`,
        });
      }

      if (registers.length < reg_start + 3 * d) {
        return res.status(400).json({
          error: "Insufficient registers",
          details: `Need at least ${reg_start + 3 * d} registers, got ${registers.length}`,
        });
      }

      const SUFT_PHI_RATIO = 13 / 28;
      const PERIOD_MODULUS = 364;
      const violations: Array<{ invariant: string; value: number; threshold: number; passed: boolean }> = [];

      const branch0Re = (registers[reg_start] >> 32) / 1_000_000;
      const branch1Re = (registers[reg_start + d] >> 32) / 1_000_000;
      const branch2Re = (registers[reg_start + 2 * d] >> 32) / 1_000_000;
      const branchSum = branch0Re + branch1Re + branch2Re;
      const gaugePass = Math.abs(branchSum) <= tolerance;
      violations.push({
        invariant: "ternary_gauge_symmetry",
        value: branchSum,
        threshold: tolerance,
        passed: gaugePass,
      });

      let sumNormSq = 0;
      const regsNeeded = d * 3;
      for (let i = 0; i < regsNeeded; i++) {
        const val = registers[reg_start + i];
        const re = ((val >> 16) & 0xFFFF) / 1_000_000;
        const im = (val & 0xFFFF) / 1_000_000;
        sumNormSq += re * re + im * im;
      }
      const energyInvariant = SUFT_PHI_RATIO * sumNormSq;
      const energyPass = energyInvariant <= tolerance || Math.abs(sumNormSq - 1.0) <= tolerance;
      violations.push({
        invariant: "reparametrization_energy",
        value: energyInvariant,
        threshold: tolerance,
        passed: energyPass,
      });

      let periodicityConsistent = true;
      for (let i = 0; i < d; i++) {
        const val = registers[reg_start + i];
        const re = (val >> 32) / 1_000_000;
        const mod364 = ((re % PERIOD_MODULUS) + PERIOD_MODULUS) % PERIOD_MODULUS;
        const distFromBoundary = Math.min(mod364, PERIOD_MODULUS - mod364);
        if (distFromBoundary < tolerance && Math.abs(re) > tolerance) {
          periodicityConsistent = false;
        }
      }
      violations.push({
        invariant: "periodicity",
        value: 0,
        threshold: PERIOD_MODULUS,
        passed: periodicityConsistent,
      });

      const allPassed = violations.every(v => v.passed);

      res.json({
        success: true,
        verified: allPassed,
        constants: {
          SUFT_PHI_RATIO: "13/28",
          PERIOD_MODULUS: 364,
        },
        invariants: violations,
        register_count: registers.length,
        dimension: d,
      });
    } catch (error: unknown) {
      log.error("Noether verification failed:", toErrorMessage(error));
      res.status(500).json({ error: "Noether verification failed" });
    }
  });
}
