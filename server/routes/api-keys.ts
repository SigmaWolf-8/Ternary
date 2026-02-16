/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { Router } from "express";
import { z } from "zod";
import { apiKeyService, AVAILABLE_SCOPES } from "../services/api-key.service";
import { createRequireAdmin } from "./middleware";
import type { IStorage } from "../storage";
import { createLogger } from "../logger";

const log = createLogger("api-keys");

const RATE_LIMIT_TIERS: Record<string, number> = {
  research: 100,
  pro: 500,
  admin: 2000,
};

const generateKeySchema = z.object({
  name: z.string().min(1).max(255),
  scopes: z.array(z.string()).min(1),
  expiresDays: z.number().int().min(0).max(3650).default(90),
  rateLimitTier: z.enum(["research", "pro", "admin"]).default("research"),
  enableRotation: z.boolean().default(true),
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

      const { name, scopes, expiresDays, rateLimitTier, enableRotation } = parsed.data;

      const invalidScopes = scopes.filter(
        (s) => !(AVAILABLE_SCOPES as readonly string[]).includes(s)
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
      const keyData = await apiKeyService.generate(owner, name, scopes, expiresDays);

      await apiKeyService.updateRateLimit(keyData.id, rateLimitTier, rpm);

      if (enableRotation && expiresDays > 0) {
        await apiKeyService.scheduleRotation(keyData.id, expiresDays);
      }

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
      const rpm = RATE_LIMIT_TIERS[parsed.data.tier] || 100;
      const result = await apiKeyService.updateRateLimit(id, parsed.data.tier, rpm);
      if (!result) {
        return res.status(404).json({ error: "Key not found" });
      }
      log.info(`Rate limit updated for key ${id}: ${parsed.data.tier} (${rpm} rpm)`);
      res.json({ success: true, tier: parsed.data.tier, rpm });
    } catch (err: any) {
      log.error("Rate limit update error:", err);
      res.status(500).json({ error: "Failed to update rate limit" });
    }
  });

  app.get("/api/keys/rate-limit-tiers", (_req, res) => {
    res.json({ tiers: RATE_LIMIT_TIERS });
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
