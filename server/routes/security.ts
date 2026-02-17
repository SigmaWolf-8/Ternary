/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { Router } from "express";
import { z } from "zod";
import { securityAuditService } from "../services/security-audit.service";
import { hptpAnomalyService } from "../services/hptp-anomaly.service";
import { threatModelService } from "../services/threat-model.service";
import { implementationStatusService } from "../services/implementation-status.service";
import { createRequireAdmin } from "./middleware";
import type { IStorage } from "../storage";
import { createLogger } from "../logger";

const log = createLogger("security-routes");

const FALLBACK_TIERS = ["ptp", "ntp", "crystal", "quartz", "cesium"] as const;

const fallbackChainEntrySchema = z.object({
  status: z.enum(["active", "standby", "failed"]),
  latency_ms: z.number().optional(),
  jitter_variance: z.number().optional(),
  frequency_ppm: z.number().optional(),
  temperature_c: z.number().optional(),
});

const fallbackChainSchema = z.object({
  ptp: fallbackChainEntrySchema.optional(),
  ntp: fallbackChainEntrySchema.optional(),
  crystal: fallbackChainEntrySchema.optional(),
  quartz: fallbackChainEntrySchema.optional(),
  cesium: fallbackChainEntrySchema.optional(),
}).refine(
  (chain) => Object.keys(chain).length > 0,
  { message: "Fallback chain must contain at least one tier" }
);

export function registerSecurityRoutes(app: Router, storage: IStorage) {
  const requireAdmin = createRequireAdmin(storage);

  const logAuditEventSchema = z.object({
    severity: z.enum(["info", "warning", "high", "critical"]),
    category: z.enum(["auth", "crypto", "boot", "network", "hptp", "firmware", "privilege"]),
    eventType: z.string().min(1).max(100),
    actor: z.string().max(255).optional(),
    description: z.string().min(1).max(5000),
    affectedComponent: z.string().max(255).optional(),
    evidence: z.record(z.unknown()).optional(),
    ipAddress: z.string().max(45).optional(),
    userId: z.string().max(255).optional(),
  });

  app.post("/api/security/audit", requireAdmin, async (req: any, res) => {
    try {
      const body = logAuditEventSchema.parse(req.body);
      const entry = await securityAuditService.logEvent(body);
      res.status(201).json(entry);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "ValidationError", status: 400, details: err.errors });
      log.error("Failed to create audit event", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/audit", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.severity) filters.severity = req.query.severity;
      if (req.query.category) filters.category = req.query.category;
      if (req.query.eventType) filters.eventType = req.query.eventType;
      if (req.query.since) filters.since = new Date(req.query.since);
      if (req.query.limit) filters.limit = parseInt(req.query.limit, 10);
      if (req.query.offset) filters.offset = parseInt(req.query.offset, 10);

      const events = await securityAuditService.getEvents(filters);
      res.json({ events, count: events.length });
    } catch (err: any) {
      log.error("Failed to fetch audit events", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/audit/unresolved", requireAdmin, async (req: any, res) => {
    try {
      const severity = req.query.severity as string | undefined;
      const events = await securityAuditService.getUnresolved(severity);
      res.json({ events, count: events.length });
    } catch (err: any) {
      log.error("Failed to fetch unresolved events", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/audit/summary", requireAdmin, async (req: any, res) => {
    try {
      const hours = req.query.hours ? parseInt(req.query.hours, 10) : 24;
      const since = new Date(Date.now() - hours * 3600000);
      const stats = await securityAuditService.getSeverityCounts(since);
      res.json(stats);
    } catch (err: any) {
      log.error("Failed to fetch audit summary", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/audit/stats", requireAdmin, async (req: any, res) => {
    try {
      const since = req.query.since ? new Date(req.query.since) : undefined;
      const stats = await securityAuditService.getSeverityCounts(since);
      res.json(stats);
    } catch (err: any) {
      log.error("Failed to fetch audit stats", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/audit/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const entry = await securityAuditService.getEventById(id);
      if (!entry) return res.status(404).json({ error: "Event not found" });
      res.json(entry);
    } catch (err: any) {
      log.error("Failed to fetch audit event", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  const resolveEventSchema = z.object({
    resolutionStatus: z.enum(["resolved", "false_positive", "acknowledged"]),
    resolutionNotes: z.string().optional(),
    resolvedBy: z.string().max(255).optional(),
  });

  app.patch("/api/security/audit/:id/resolve", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const body = resolveEventSchema.parse(req.body);
      const updated = await securityAuditService.resolveEvent(id, body);
      if (!updated) return res.status(404).json({ error: "Event not found" });
      res.json(updated);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "ValidationError", status: 400, details: err.errors });
      log.error("Failed to resolve audit event", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  const hptpAnomalySchema = z.object({
    anomalyType: z.enum(["jitter_variance", "clock_drift", "sync_failure", "glitch_detected"]),
    severityScore: z.number().min(0).max(10),
    thresholdValue: z.number(),
    observedValue: z.number(),
    variancePercentage: z.number(),
    fallbackChain: fallbackChainSchema,
    activeTier: z.enum(["ptp", "ntp", "crystal", "quartz", "cesium"]),
  }).refine(
    (data) => data.fallbackChain[data.activeTier] !== undefined,
    { message: "activeTier must be present in the fallbackChain" }
  );

  app.post("/api/security/hptp/anomalies", requireAdmin, async (req: any, res) => {
    try {
      const body = hptpAnomalySchema.parse(req.body);
      const event = await hptpAnomalyService.reportAnomaly(body);
      res.status(201).json(event);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "ValidationError", status: 400, details: err.errors });
      log.error("Failed to log HPTP anomaly", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/hptp/anomalies", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.anomalyType) filters.anomalyType = req.query.anomalyType;
      if (req.query.days) filters.days = parseInt(req.query.days, 10);
      if (req.query.limit) filters.limit = parseInt(req.query.limit, 10);
      if (req.query.offset) filters.offset = parseInt(req.query.offset, 10);

      const events = await hptpAnomalyService.getEvents(filters);
      res.json({ events, count: events.length });
    } catch (err: any) {
      log.error("Failed to fetch HPTP anomalies", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/hptp/status", requireAdmin, async (_req: any, res) => {
    try {
      const status = await hptpAnomalyService.getStatus();
      res.json(status);
    } catch (err: any) {
      log.error("Failed to fetch HPTP status", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/hptp/fallback-analysis", requireAdmin, async (_req: any, res) => {
    try {
      const analysis = await hptpAnomalyService.getFallbackAnalysis();
      res.json(analysis);
    } catch (err: any) {
      log.error("Failed to fetch fallback analysis", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/hptp/stats", requireAdmin, async (req: any, res) => {
    try {
      const since = req.query.since ? new Date(req.query.since) : undefined;
      const stats = await hptpAnomalyService.getStatistics(since);
      res.json(stats);
    } catch (err: any) {
      log.error("Failed to fetch HPTP stats", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/hptp/thresholds", requireAdmin, async (_req: any, res) => {
    res.json(hptpAnomalyService.getThresholds());
  });

  app.get("/api/security/hptp/fallback-modes", requireAdmin, async (_req: any, res) => {
    res.json(hptpAnomalyService.getFallbackModes());
  });

  app.get("/api/security/hptp/redundancy", requireAdmin, async (_req: any, res) => {
    res.json(hptpAnomalyService.getRedundancyArchitecture());
  });

  const threatModelSchema = z.object({
    threatId: z.string().min(1).max(50),
    threatName: z.string().min(1).max(255),
    description: z.string().optional(),
    category: z.string().min(1).max(50),
    attackVector: z.string().max(100).optional(),
    likelihood: z.enum(["low", "medium", "high", "critical"]),
    impact: z.enum(["low", "medium", "high", "critical"]),
    mitigationStatus: z.enum(["mitigated", "in_progress", "acknowledged", "not_addressed"]),
    controls: z.array(z.object({
      controlId: z.string(),
      controlName: z.string(),
      status: z.string(),
      evidence: z.string().optional(),
    })).optional(),
    residualRisk: z.number().min(0).max(10).optional(),
    notes: z.string().optional(),
    createdBy: z.string().max(255).optional(),
  });

  app.post("/api/security/threats", requireAdmin, async (req: any, res) => {
    try {
      const body = threatModelSchema.parse(req.body);
      const entry = await threatModelService.create(body);
      res.status(201).json(entry);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "ValidationError", status: 400, details: err.errors });
      log.error("Failed to create threat entry", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/threats", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.category) filters.category = req.query.category;
      if (req.query.mitigationStatus) filters.mitigationStatus = req.query.mitigationStatus;
      if (req.query.likelihood) filters.likelihood = req.query.likelihood;
      if (req.query.impact) filters.impact = req.query.impact;

      const entries = await threatModelService.getAll(filters);
      res.json({ entries, count: entries.length });
    } catch (err: any) {
      log.error("Failed to fetch threats", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/threats/risk-matrix", requireAdmin, async (_req: any, res) => {
    try {
      const matrix = await threatModelService.getRiskMatrix();
      res.json(matrix);
    } catch (err: any) {
      log.error("Failed to fetch risk matrix", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/threats/stats", requireAdmin, async (_req: any, res) => {
    try {
      const stats = await threatModelService.getSummaryStats();
      res.json(stats);
    } catch (err: any) {
      log.error("Failed to fetch threat stats", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/threats/meta", (_req: any, res) => {
    res.json({
      categories: threatModelService.getCategories(),
      likelihoodLevels: threatModelService.getLikelihoodLevels(),
      impactLevels: threatModelService.getImpactLevels(),
      mitigationStatuses: threatModelService.getMitigationStatuses(),
    });
  });

  app.get("/api/security/threats/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) {
        const entry = await threatModelService.getByThreatId(req.params.id);
        if (!entry) return res.status(404).json({ error: "Threat entry not found" });
        return res.json(entry);
      }
      const entry = await threatModelService.getById(id);
      if (!entry) return res.status(404).json({ error: "Threat entry not found" });
      res.json(entry);
    } catch (err: any) {
      log.error("Failed to fetch threat", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.patch("/api/security/threats/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const updated = await threatModelService.update(id, req.body);
      if (!updated) return res.status(404).json({ error: "Threat entry not found" });
      res.json(updated);
    } catch (err: any) {
      log.error("Failed to update threat", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.delete("/api/security/threats/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const deleted = await threatModelService.delete(id);
      if (!deleted) return res.status(404).json({ error: "Threat entry not found" });
      res.json({ success: true, deleted });
    } catch (err: any) {
      log.error("Failed to delete threat", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.post("/api/security/threats/seed", requireAdmin, async (_req: any, res) => {
    try {
      const result = await threatModelService.seedDefaults();
      res.json(result);
    } catch (err: any) {
      log.error("Failed to seed threats", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  const implStatusSchema = z.object({
    componentName: z.string().min(1).max(255),
    category: z.string().min(1).max(50),
    status: z.enum(["proven", "in_progress", "planned", "concern", "blocked"]),
    completionPercentage: z.number().int().min(0).max(100).optional(),
    description: z.string().optional(),
    locTotal: z.number().int().min(0).optional(),
    locTested: z.number().int().min(0).optional(),
    testCount: z.number().int().min(0).optional(),
    proofCount: z.number().int().min(0).optional(),
    proofCoveragePercentage: z.number().min(0).max(100).optional(),
    githubPath: z.string().max(255).optional(),
    responsibleTeam: z.string().max(100).optional(),
    milestoneDate: z.string().max(20).optional(),
    summaryLine: z.string().optional(),
    externalAuditStatus: z.string().max(50).optional(),
    externalAuditor: z.string().max(100).optional(),
  });

  app.post("/api/security/implementation", requireAdmin, async (req: any, res) => {
    try {
      const body = implStatusSchema.parse(req.body);
      const entry = await implementationStatusService.create(body);
      res.status(201).json(entry);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "ValidationError", status: 400, details: err.errors });
      log.error("Failed to create impl status entry", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/implementation", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.category) filters.category = req.query.category;
      if (req.query.status) filters.status = req.query.status;

      const entries = await implementationStatusService.getAll(filters);
      res.json({ entries, count: entries.length });
    } catch (err: any) {
      log.error("Failed to fetch impl status", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/implementation/summary", requireAdmin, async (_req: any, res) => {
    try {
      const summary = await implementationStatusService.getSummary();
      res.json(summary);
    } catch (err: any) {
      log.error("Failed to fetch impl summary", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/implementation/metrics", requireAdmin, async (_req: any, res) => {
    try {
      const metrics = await implementationStatusService.getMetrics();
      res.json(metrics);
    } catch (err: any) {
      log.error("Failed to fetch impl metrics", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/implementation/milestones", requireAdmin, async (req: any, res) => {
    try {
      const from = req.query.from as string | undefined;
      const to = req.query.to as string | undefined;
      const milestones = await implementationStatusService.getMilestones(from, to);
      res.json(milestones);
    } catch (err: any) {
      log.error("Failed to fetch impl milestones", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/implementation/meta", (_req: any, res) => {
    res.json({
      categories: implementationStatusService.getCategories(),
      statuses: implementationStatusService.getStatuses(),
    });
  });

  app.get("/api/security/implementation/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const entry = await implementationStatusService.getById(id);
      if (!entry) return res.status(404).json({ error: "Implementation entry not found" });
      res.json(entry);
    } catch (err: any) {
      log.error("Failed to fetch impl entry", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.patch("/api/security/implementation/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const body = implStatusSchema.partial().parse(req.body);
      const updated = await implementationStatusService.update(id, body);
      if (!updated) return res.status(404).json({ error: "Implementation entry not found" });
      res.json(updated);
    } catch (err: any) {
      log.error("Failed to update impl entry", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.delete("/api/security/implementation/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const deleted = await implementationStatusService.delete(id);
      if (!deleted) return res.status(404).json({ error: "Implementation entry not found" });
      res.json({ success: true, deleted });
    } catch (err: any) {
      log.error("Failed to delete impl entry", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.post("/api/security/implementation/seed", requireAdmin, async (_req: any, res) => {
    try {
      const result = await implementationStatusService.seedDefaults();
      res.json(result);
    } catch (err: any) {
      log.error("Failed to seed impl status", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/dashboard", requireAdmin, async (req: any, res) => {
    try {
      const since = req.query.since ? new Date(req.query.since) : new Date(Date.now() - 7 * 86400000);

      const [auditStats, hptpStats, hptpStatus, threatStats, implSummary, unresolvedAudit] = await Promise.all([
        securityAuditService.getSeverityCounts(since),
        hptpAnomalyService.getStatistics(since),
        hptpAnomalyService.getStatus(),
        threatModelService.getSummaryStats(),
        implementationStatusService.getSummary(),
        securityAuditService.getUnresolved(),
      ]);

      res.json({
        period: { since: since.toISOString(), until: new Date().toISOString() },
        auditEvents: auditStats,
        hptpAnomalies: hptpStats,
        hptpStatus,
        threatModel: threatStats,
        implementation: implSummary,
        unresolvedAlerts: unresolvedAudit.length,
      });
    } catch (err: any) {
      log.error("Failed to fetch security dashboard", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/metadata/categories", (_req: any, res) => {
    res.json({
      auditCategories: ["auth", "crypto", "boot", "network", "hptp", "firmware", "privilege"],
      threatCategories: threatModelService.getCategories(),
      implementationCategories: implementationStatusService.getCategories(),
    });
  });

  app.get("/api/security/metadata/types", (_req: any, res) => {
    res.json({
      auditSeverities: ["info", "warning", "high", "critical"],
      resolutionStatuses: ["unresolved", "resolved", "false_positive", "acknowledged"],
      anomalyTypes: ["jitter_variance", "clock_drift", "sync_failure", "glitch_detected"],
      fallbackTiers: FALLBACK_TIERS,
      likelihoodLevels: threatModelService.getLikelihoodLevels(),
      impactLevels: threatModelService.getImpactLevels(),
      mitigationStatuses: threatModelService.getMitigationStatuses(),
      implementationStatuses: implementationStatusService.getStatuses(),
    });
  });

  log.info("Security routes registered");
}
