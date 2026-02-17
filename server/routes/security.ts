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

export function registerSecurityRoutes(app: Router, storage: IStorage) {
  const requireAdmin = createRequireAdmin(storage);

  const logAuditEventSchema = z.object({
    eventType: z.string().min(1).max(100),
    severity: z.enum(["low", "medium", "high", "critical"]),
    source: z.string().min(1).max(100),
    message: z.string().min(1),
    details: z.record(z.unknown()).optional(),
    ipAddress: z.string().max(45).optional(),
    userId: z.string().max(255).optional(),
  });

  app.post("/api/security/audit", requireAdmin, async (req: any, res) => {
    try {
      const body = logAuditEventSchema.parse(req.body);
      const entry = await securityAuditService.logEvent(body);
      res.status(201).json(entry);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "Validation failed", details: err.errors });
      log.error("Failed to create audit event", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/audit", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.severity) filters.severity = req.query.severity;
      if (req.query.eventType) filters.eventType = req.query.eventType;
      if (req.query.source) filters.source = req.query.source;
      if (req.query.resolved !== undefined) filters.resolved = req.query.resolved === "true";
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

  app.get("/api/security/audit/unresolved", requireAdmin, async (_req: any, res) => {
    try {
      const events = await securityAuditService.getUnresolved();
      res.json({ events, count: events.length });
    } catch (err: any) {
      log.error("Failed to fetch unresolved events", { error: err.message });
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

  app.patch("/api/security/audit/:id/resolve", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const updated = await securityAuditService.resolveEvent(id);
      if (!updated) return res.status(404).json({ error: "Event not found" });
      res.json(updated);
    } catch (err: any) {
      log.error("Failed to resolve audit event", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  const hptpAnomalySchema = z.object({
    anomalyType: z.string().min(1).max(100),
    detectedValue: z.number(),
    expectedValue: z.number(),
    sensorId: z.string().max(100).optional(),
    details: z.record(z.unknown()).optional(),
  });

  app.post("/api/security/hptp/anomaly", requireAdmin, async (req: any, res) => {
    try {
      const body = hptpAnomalySchema.parse(req.body);
      const event = await hptpAnomalyService.evaluateAndLog(body);
      res.status(201).json(event);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "Validation failed", details: err.errors });
      log.error("Failed to log HPTP anomaly", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/hptp/anomalies", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.anomalyType) filters.anomalyType = req.query.anomalyType;
      if (req.query.severity) filters.severity = req.query.severity;
      if (req.query.fallbackTriggered !== undefined) filters.fallbackTriggered = req.query.fallbackTriggered === "true";
      if (req.query.since) filters.since = new Date(req.query.since);
      if (req.query.limit) filters.limit = parseInt(req.query.limit, 10);
      if (req.query.offset) filters.offset = parseInt(req.query.offset, 10);

      const events = await hptpAnomalyService.getEvents(filters);
      res.json({ events, count: events.length });
    } catch (err: any) {
      log.error("Failed to fetch HPTP anomalies", { error: err.message });
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
    category: z.string().min(1).max(100),
    threatVector: z.string().min(1).max(255),
    scope: z.enum(["in_scope", "out_of_scope", "deferred"]),
    adversaryType: z.string().min(1).max(100),
    currentMitigation: z.string().min(1),
    residualRisk: z.enum(["negligible", "low", "medium", "high", "critical"]),
    redundancyFallback: z.string().optional(),
    detectionMechanism: z.string().optional(),
    cvssScore: z.number().min(0).max(10).optional(),
    status: z.string().min(1).max(30),
  });

  app.post("/api/security/threats", requireAdmin, async (req: any, res) => {
    try {
      const body = threatModelSchema.parse(req.body);
      const entry = await threatModelService.create(body);
      res.status(201).json(entry);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "Validation failed", details: err.errors });
      log.error("Failed to create threat entry", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/threats", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.category) filters.category = req.query.category;
      if (req.query.scope) filters.scope = req.query.scope;
      if (req.query.adversaryType) filters.adversaryType = req.query.adversaryType;
      if (req.query.residualRisk) filters.residualRisk = req.query.residualRisk;
      if (req.query.status) filters.status = req.query.status;

      const entries = await threatModelService.getAll(filters);
      res.json({ entries, count: entries.length });
    } catch (err: any) {
      log.error("Failed to fetch threats", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/threats/matrix", requireAdmin, async (_req: any, res) => {
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
      adversaryTypes: threatModelService.getAdversaryTypes(),
      riskLevels: threatModelService.getRiskLevels(),
      scopes: threatModelService.getScopes(),
    });
  });

  app.get("/api/security/threats/:id", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
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
    component: z.string().min(1).max(255),
    category: z.string().min(1).max(100),
    status: z.enum(["proven", "in_progress", "planned", "concern", "blocked"]),
    completionPercent: z.number().int().min(0).max(100).optional(),
    evidence: z.string().optional(),
    githubPath: z.string().max(512).optional(),
    dependencies: z.array(z.string()).optional(),
    blockers: z.string().optional(),
    targetDate: z.string().max(20).optional(),
    phase: z.number().int().min(0).max(5).optional(),
    locCount: z.number().int().min(0).optional(),
    testCount: z.number().int().min(0).optional(),
    proofLines: z.number().int().min(0).optional(),
  });

  app.post("/api/security/implementation/entry", requireAdmin, async (req: any, res) => {
    try {
      const body = implStatusSchema.parse(req.body);
      const entry = await implementationStatusService.create(body);
      res.status(201).json(entry);
    } catch (err: any) {
      if (err.name === "ZodError") return res.status(400).json({ error: "Validation failed", details: err.errors });
      log.error("Failed to create impl status entry", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  app.get("/api/security/implementation", requireAdmin, async (req: any, res) => {
    try {
      const filters: any = {};
      if (req.query.category) filters.category = req.query.category;
      if (req.query.status) filters.status = req.query.status;
      if (req.query.phase !== undefined) filters.phase = parseInt(req.query.phase, 10);

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

  app.patch("/api/security/implementation/:id/verify", requireAdmin, async (req: any, res) => {
    try {
      const id = parseInt(req.params.id, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid ID" });
      const updated = await implementationStatusService.verify(id);
      if (!updated) return res.status(404).json({ error: "Implementation entry not found" });
      res.json(updated);
    } catch (err: any) {
      log.error("Failed to verify impl entry", { error: err.message });
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

      const [auditStats, hptpStats, threatStats, implSummary, unresolvedAudit] = await Promise.all([
        securityAuditService.getSeverityCounts(since),
        hptpAnomalyService.getStatistics(since),
        threatModelService.getSummaryStats(),
        implementationStatusService.getSummary(),
        securityAuditService.getUnresolved(),
      ]);

      res.json({
        period: { since: since.toISOString(), until: new Date().toISOString() },
        auditEvents: auditStats,
        hptpAnomalies: hptpStats,
        threatModel: threatStats,
        implementation: implSummary,
        unresolvedAlerts: unresolvedAudit.length,
      });
    } catch (err: any) {
      log.error("Failed to fetch security dashboard", { error: err.message });
      res.status(500).json({ error: "Internal server error" });
    }
  });

  log.info("Security routes registered");
}
