/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { Router } from "express";
import { z } from "zod";
import { apiKeyService, AVAILABLE_SCOPES, isScopeValid } from "../services/api-key.service";
import { createRequireAdmin } from "./middleware";
import type { IStorage } from "../storage";
import { createLogger } from "../logger";

const log = createLogger("api-keys");

const RATE_LIMIT_TIERS: Record<string, number> = {
  research: 100,
  pro: 500,
  admin: 2000,
};

const ENTITY_TYPES = ["customer", "vendor", "partner", "internal", "contractor", "government"] as const;

const generateKeySchema = z.object({
  name: z.string().min(1).max(255),
  scopes: z.array(z.string()).min(1),
  expiresDays: z.number().int().min(0).max(3650).default(90),
  rateLimitTier: z.enum(["research", "pro", "admin"]).default("research"),
  enableRotation: z.boolean().default(true),
  entityType: z.enum(["customer", "vendor", "partner", "internal", "contractor", "government"]).optional(),
  entityName: z.string().max(255).optional(),
  project: z.string().max(255).optional(),
  department: z.string().max(255).optional(),
  tags: z.array(z.string().max(50)).max(20).optional(),
  notes: z.string().max(1000).optional(),
});

const updateKeyMetadataSchema = z.object({
  name: z.string().min(1).max(255).optional(),
  entityType: z.enum(["customer", "vendor", "partner", "internal", "contractor", "government"]).nullable().optional(),
  entityName: z.string().max(255).nullable().optional(),
  project: z.string().max(255).nullable().optional(),
  department: z.string().max(255).nullable().optional(),
  tags: z.array(z.string().max(50)).max(20).optional(),
  notes: z.string().max(1000).nullable().optional(),
});

const revokeKeySchema = z.object({
  id: z.string().uuid(),
});

const updateRateLimitSchema = z.object({
  tier: z.enum(["research", "pro", "admin"]),
});

export function registerApiKeyRoutes(app: Router, storage: IStorage) {
  const requireAdmin = createRequireAdmin(storage);

  app.get("/api/keys/scopes", (_req, res) => {
    res.json({ scopes: AVAILABLE_SCOPES });
  });

  app.post("/api/keys/generate", requireAdmin, async (req: any, res) => {
    try {
      const parsed = generateKeySchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { name, scopes, expiresDays, rateLimitTier, enableRotation, entityType, entityName, project, department, tags, notes } = parsed.data;

      const invalidScopes = scopes.filter(
        (s) => !isScopeValid(s)
      );
      if (invalidScopes.length > 0) {
        return res.status(400).json({
          error: "Invalid scopes",
          invalidScopes,
          availableScopes: AVAILABLE_SCOPES,
        });
      }

      const owner = req.adminUser?.email || req.adminUser?.id || "admin";
      const rpm = RATE_LIMIT_TIERS[rateLimitTier] || 100;
      const wbs = { entityType, entityName, project, department, tags, notes };
      const keyData = await apiKeyService.generate(owner, name, scopes, expiresDays, wbs);

      await apiKeyService.updateRateLimit(keyData.id, rateLimitTier, rpm);

      if (enableRotation && expiresDays > 0) {
        await apiKeyService.scheduleRotation(keyData.id, expiresDays);
      }

      const actorId = req.adminUser?.id || "admin";
      const actorEmail = req.adminUser?.email || null;
      const ip = req.ip || req.connection?.remoteAddress || null;
      await apiKeyService.logAuditEvent(keyData.id, "key_generated", actorId, actorEmail, {
        keyPrefix: keyData.keyPrefix,
        name,
        scopes,
        expiresDays,
        rateLimitTier,
        rateLimitRpm: rpm,
        enableRotation,
      }, ip);

      log.info(`API key generated: ${keyData.keyPrefix}*** for ${owner} (${name}) [${rateLimitTier}/${rpm}rpm]`);

      res.json({
        success: true,
        key: keyData.key,
        id: keyData.id,
        keyPrefix: keyData.keyPrefix,
        name: keyData.name,
        scopes: keyData.scopes,
        expiresAt: keyData.expiresAt,
        createdAt: keyData.createdAt,
        rateLimitTier,
        rateLimitRpm: rpm,
        warning: "Store this key securely. It will not be shown again.",
      });
    } catch (err: any) {
      log.error("Key generation error:", err);
      res.status(500).json({ error: "Failed to generate API key" });
    }
  });

  app.get("/api/keys", requireAdmin, async (req: any, res) => {
    try {
      const keys = await apiKeyService.listAll();
      res.json({ success: true, keys });
    } catch (err: any) {
      log.error("Key list error:", err);
      res.status(500).json({ error: "Failed to list API keys" });
    }
  });

  app.get("/api/keys/stats", requireAdmin, async (_req, res) => {
    try {
      const stats = await apiKeyService.getStats();
      res.json({ success: true, stats });
    } catch (err: any) {
      log.error("Key stats error:", err);
      res.status(500).json({ error: "Failed to get stats" });
    }
  });

  app.post("/api/keys/revoke/:id", requireAdmin, async (req: any, res) => {
    try {
      const { id } = req.params;
      const parsed = revokeKeySchema.safeParse({ id });
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid key ID" });
      }

      const result = await apiKeyService.revoke(id);
      if (!result) {
        return res.status(404).json({ error: "Key not found" });
      }

      const actorId = req.adminUser?.id || "admin";
      const actorEmail = req.adminUser?.email || null;
      const ip = req.ip || req.connection?.remoteAddress || null;
      await apiKeyService.logAuditEvent(id, "key_revoked", actorId, actorEmail, {
        keyPrefix: result.keyPrefix,
        keyName: result.name,
      }, ip);

      log.info(`API key revoked: ${result.keyPrefix}*** by ${req.adminUser?.email || "admin"}`);
      res.json({ success: true, message: "Key revoked" });
    } catch (err: any) {
      log.error("Key revoke error:", err);
      res.status(500).json({ error: "Failed to revoke key" });
    }
  });

  app.get("/api/keys/:id/logs", requireAdmin, async (req: any, res) => {
    try {
      const { id } = req.params;
      const limit = Math.min(parseInt(req.query.limit || "50", 10), 200);
      const logs = await apiKeyService.getUsageLogs(id, limit);
      res.json({ success: true, logs });
    } catch (err: any) {
      log.error("Key logs error:", err);
      res.status(500).json({ error: "Failed to get logs" });
    }
  });

  app.post("/api/keys/rotate/:id", requireAdmin, async (req: any, res) => {
    try {
      const { id } = req.params;
      const result = await apiKeyService.rotateKey(id);

      const actorId = req.adminUser?.id || "admin";
      const actorEmail = req.adminUser?.email || null;
      const ip = req.ip || req.connection?.remoteAddress || null;
      await apiKeyService.logAuditEvent(id, "key_rotated", actorId, actorEmail, {
        oldKeyId: result.oldKeyId,
        newKeyId: result.newKey.id,
        newKeyPrefix: result.newKey.keyPrefix,
        graceEnds: result.graceEnds,
      }, ip);

      log.info(`API key rotated: ${id} by ${req.adminUser?.email || "admin"}`);
      res.json({
        success: true,
        newKey: result.newKey.key,
        newKeyId: result.newKey.id,
        newKeyPrefix: result.newKey.keyPrefix,
        oldKeyId: result.oldKeyId,
        graceEnds: result.graceEnds,
        warning: "Store the new key securely. Old key remains valid for 7 days.",
      });
    } catch (err: any) {
      log.error("Key rotation error:", err);
      res.status(400).json({ error: err.message || "Failed to rotate key" });
    }
  });

  app.get("/api/keys/expiring", requireAdmin, async (req: any, res) => {
    try {
      const days = Math.min(parseInt(req.query.days || "14", 10), 90);
      const keys = await apiKeyService.getExpiringKeys(days);
      res.json({ success: true, keys, withinDays: days });
    } catch (err: any) {
      log.error("Expiring keys error:", err);
      res.status(500).json({ error: "Failed to get expiring keys" });
    }
  });

  app.patch("/api/keys/:id/rate-limit", requireAdmin, async (req: any, res) => {
    try {
      const { id } = req.params;
      const parsed = updateRateLimitSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const keys = await apiKeyService.listAll();
      const existingKey = keys.find((k: any) => k.id === id);
      const previousTier = existingKey?.rateLimitTier || "unknown";
      const previousRpm = existingKey?.rateLimitRpm || 0;

      const rpm = RATE_LIMIT_TIERS[parsed.data.tier] || 100;
      const result = await apiKeyService.updateRateLimit(id, parsed.data.tier, rpm);
      if (!result) {
        return res.status(404).json({ error: "Key not found" });
      }

      const actorId = req.adminUser?.id || "admin";
      const actorEmail = req.adminUser?.email || null;
      const ip = req.ip || req.connection?.remoteAddress || null;
      await apiKeyService.logAuditEvent(id, "tier_change", actorId, actorEmail, {
        keyPrefix: existingKey?.keyPrefix,
        keyName: existingKey?.name,
        fromTier: previousTier,
        toTier: parsed.data.tier,
        fromRpm: previousRpm,
        toRpm: rpm,
      }, ip);

      log.info(`Rate limit updated for key ${id}: ${previousTier} -> ${parsed.data.tier} (${rpm} rpm) by ${actorEmail || actorId}`);
      res.json({ success: true, tier: parsed.data.tier, rpm, previousTier });
    } catch (err: any) {
      log.error("Rate limit update error:", err);
      res.status(500).json({ error: "Failed to update rate limit" });
    }
  });

  app.get("/api/keys/rate-limit-tiers", (_req, res) => {
    res.json({ tiers: RATE_LIMIT_TIERS });
  });

  app.get("/api/keys/entity-types", (_req, res) => {
    res.json({ entityTypes: ENTITY_TYPES });
  });

  app.patch("/api/keys/:id/metadata", requireAdmin, async (req: any, res) => {
    try {
      const { id } = req.params;
      const parsed = updateKeyMetadataSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const result = await apiKeyService.updateKeyMetadata(id, parsed.data);
      if (!result) {
        return res.status(404).json({ error: "Key not found or no changes made" });
      }

      const actorId = req.adminUser?.id || "admin";
      const actorEmail = req.adminUser?.email || null;
      const ip = req.ip || req.connection?.remoteAddress || null;
      await apiKeyService.logAuditEvent(id, "metadata_updated", actorId, actorEmail, {
        keyPrefix: result.keyPrefix,
        keyName: result.name,
        updatedFields: Object.keys(parsed.data),
      }, ip);

      log.info(`Key metadata updated: ${result.keyPrefix}*** by ${actorEmail || actorId}`);
      res.json({ success: true, key: result });
    } catch (err: any) {
      log.error("Key metadata update error:", err);
      res.status(500).json({ error: "Failed to update key metadata" });
    }
  });

  app.get("/api/keys/anomalies", requireAdmin, async (req: any, res) => {
    try {
      const days = Math.min(parseInt(req.query.days || "7", 10), 30);
      const anomalies = await apiKeyService.detectAnomalies(days);
      res.json({ success: true, anomalies, withinDays: days });
    } catch (err: any) {
      log.error("Anomaly detection error:", err);
      res.status(500).json({ error: "Failed to detect anomalies" });
    }
  });

  app.get("/api/keys/audit", requireAdmin, async (req: any, res) => {
    try {
      const limit = Math.min(parseInt(req.query.limit || "100", 10), 500);
      const events = await apiKeyService.getRecentAuditEvents(limit);
      res.json({ success: true, events });
    } catch (err: any) {
      log.error("Audit events error:", err);
      res.status(500).json({ error: "Failed to get audit events" });
    }
  });

  app.get("/api/keys/:id/audit", requireAdmin, async (req: any, res) => {
    try {
      const { id } = req.params;
      const limit = Math.min(parseInt(req.query.limit || "50", 10), 200);
      const events = await apiKeyService.getAuditEvents(id, limit);
      res.json({ success: true, events });
    } catch (err: any) {
      log.error("Key audit events error:", err);
      res.status(500).json({ error: "Failed to get audit events" });
    }
  });

  app.get("/api/keys/validate-external", async (req, res) => {
    const apiKey =
      (req.headers["x-api-key"] as string) ||
      (req.headers["authorization"] as string)?.replace(/^Bearer\s+/i, "") ||
      (req.query.api_key as string);

    if (!apiKey) {
      return res.status(401).json({ valid: false, error: "No API key provided" });
    }

    try {
      const result = await apiKeyService.validate(apiKey);
      if (!result?.valid) {
        return res.status(403).json({ valid: false, error: "Invalid or expired key" });
      }
      res.json({
        valid: true,
        scopes: result.scopes,
        owner: result.owner,
      });
    } catch {
      res.status(500).json({ valid: false, error: "Validation failed" });
    }
  });
}
