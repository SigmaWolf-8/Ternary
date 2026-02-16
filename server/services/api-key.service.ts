/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import crypto from "crypto";
import { db } from "../db";
import { apiKeys, apiKeyLogs } from "@shared/schema";
import { eq, desc, and, lt, isNull, sql } from "drizzle-orm";
import { createLogger } from "../logger";

const log = createLogger("api-key-service");

const KEY_PREFIX = "plm_";

function hashKey(rawKey: string): string {
  return crypto.createHash("sha256").update(rawKey).digest("hex");
}

function constantTimeCompare(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  return crypto.timingSafeEqual(Buffer.from(a), Buffer.from(b));
}

export const AVAILABLE_SCOPES = [
  "read:ternary",
  "write:ternary",
  "read:phase",
  "write:phase",
  "read:hptp",
  "write:hptp",
  "read:compression",
  "write:compression",
  "read:calendar",
  "read:agent-array",
  "write:agent-array",
  "read:whitepaper",
  "admin:keys",
] as const;

export type ApiKeyScope = (typeof AVAILABLE_SCOPES)[number];

export const apiKeyService = {
  async generate(
    owner: string,
    name: string,
    scopes: string[],
    expiresDays: number = 90
  ) {
    const rawKey = crypto.randomBytes(32).toString("hex");
    const fullKey = `${KEY_PREFIX}${rawKey}`;
    const keyHash = hashKey(rawKey);
    const keyPrefix = fullKey.substring(0, 8);

    const [record] = await db
      .insert(apiKeys)
      .values({
        keyPrefix,
        keyHash,
        name,
        owner,
        scopes,
        expiresAt: expiresDays > 0
          ? new Date(Date.now() + expiresDays * 86400000)
          : null,
      })
      .returning();

    return {
      id: record.id,
      key: fullKey,
      keyPrefix,
      name,
      scopes,
      expiresAt: record.expiresAt,
      createdAt: record.createdAt,
    };
  },

  async validate(
    rawApiKey: string
  ): Promise<{
    valid: boolean;
    keyId: string;
    scopes: string[];
    owner: string;
    rateLimitRpm: number;
    rateLimitTier: string;
  } | null> {
    const stripped = rawApiKey.startsWith(KEY_PREFIX)
      ? rawApiKey.slice(KEY_PREFIX.length)
      : rawApiKey;

    const inputHash = hashKey(stripped);

    const [keyRecord] = await db
      .select()
      .from(apiKeys)
      .where(eq(apiKeys.keyHash, inputHash));

    if (!keyRecord) return null;

    if (!constantTimeCompare(inputHash, keyRecord.keyHash)) return null;

    if (!keyRecord.isActive) return null;
    if (keyRecord.revokedAt) return null;
    if (keyRecord.expiresAt && keyRecord.expiresAt < new Date()) return null;

    await db
      .update(apiKeys)
      .set({
        lastUsedAt: new Date(),
        usageCount: keyRecord.usageCount + 1,
      })
      .where(eq(apiKeys.id, keyRecord.id));

    return {
      valid: true,
      keyId: keyRecord.id,
      scopes: keyRecord.scopes as string[],
      owner: keyRecord.owner,
      rateLimitRpm: keyRecord.rateLimitRpm,
      rateLimitTier: keyRecord.rateLimitTier,
    };
  },

  async logUsage(
    keyId: string,
    endpoint: string,
    method: string,
    statusCode: number | null,
    ipAddress: string | null
  ) {
    await db.insert(apiKeyLogs).values({
      keyId,
      endpoint,
      method,
      statusCode,
      ipAddress,
    });
  },

  async listByOwner(owner: string) {
    return db
      .select({
        id: apiKeys.id,
        keyPrefix: apiKeys.keyPrefix,
        name: apiKeys.name,
        owner: apiKeys.owner,
        scopes: apiKeys.scopes,
        isActive: apiKeys.isActive,
        expiresAt: apiKeys.expiresAt,
        revokedAt: apiKeys.revokedAt,
        lastUsedAt: apiKeys.lastUsedAt,
        usageCount: apiKeys.usageCount,
        rotationScheduledAt: apiKeys.rotationScheduledAt,
        previousKeyId: apiKeys.previousKeyId,
        rateLimitTier: apiKeys.rateLimitTier,
        rateLimitRpm: apiKeys.rateLimitRpm,
        createdAt: apiKeys.createdAt,
      })
      .from(apiKeys)
      .where(eq(apiKeys.owner, owner))
      .orderBy(desc(apiKeys.createdAt));
  },

  async listAll() {
    return db
      .select({
        id: apiKeys.id,
        keyPrefix: apiKeys.keyPrefix,
        name: apiKeys.name,
        owner: apiKeys.owner,
        scopes: apiKeys.scopes,
        isActive: apiKeys.isActive,
        expiresAt: apiKeys.expiresAt,
        revokedAt: apiKeys.revokedAt,
        lastUsedAt: apiKeys.lastUsedAt,
        usageCount: apiKeys.usageCount,
        rotationScheduledAt: apiKeys.rotationScheduledAt,
        previousKeyId: apiKeys.previousKeyId,
        rateLimitTier: apiKeys.rateLimitTier,
        rateLimitRpm: apiKeys.rateLimitRpm,
        createdAt: apiKeys.createdAt,
      })
      .from(apiKeys)
      .orderBy(desc(apiKeys.createdAt));
  },

  async revoke(keyId: string) {
    const [result] = await db
      .update(apiKeys)
      .set({ revokedAt: new Date(), isActive: false })
      .where(eq(apiKeys.id, keyId))
      .returning();
    return result;
  },

  async getUsageLogs(keyId: string, limit: number = 50) {
    return db
      .select()
      .from(apiKeyLogs)
      .where(eq(apiKeyLogs.keyId, keyId))
      .orderBy(desc(apiKeyLogs.createdAt))
      .limit(limit);
  },

  async getStats() {
    const allKeys = await db.select().from(apiKeys);
    const active = allKeys.filter((k) => k.isActive && !k.revokedAt);
    const revoked = allKeys.filter((k) => !!k.revokedAt);
    const expired = allKeys.filter(
      (k) => k.expiresAt && k.expiresAt < new Date() && !k.revokedAt
    );
    const totalUsage = allKeys.reduce((sum, k) => sum + k.usageCount, 0);

    return {
      total: allKeys.length,
      active: active.length,
      revoked: revoked.length,
      expired: expired.length,
      totalUsage,
    };
  },

  async scheduleRotation(keyId: string, daysUntil: number = 90) {
    const scheduledAt = new Date(Date.now() + daysUntil * 86400000);
    await db
      .update(apiKeys)
      .set({ rotationScheduledAt: scheduledAt })
      .where(eq(apiKeys.id, keyId));
    return scheduledAt;
  },

  async rotateKey(oldKeyId: string) {
    const [oldKey] = await db
      .select()
      .from(apiKeys)
      .where(eq(apiKeys.id, oldKeyId));

    if (!oldKey) throw new Error("Key not found");
    if (oldKey.revokedAt || !oldKey.isActive)
      throw new Error("Cannot rotate an inactive or revoked key");

    const newKeyData = await this.generate(
      oldKey.owner,
      `${oldKey.name} (rotated)`,
      oldKey.scopes as string[],
      90
    );

    await db
      .update(apiKeys)
      .set({ previousKeyId: oldKeyId })
      .where(eq(apiKeys.id, newKeyData.id));

    await this.scheduleRotation(newKeyData.id, 90);

    const gracePeriodMs = 7 * 86400000;
    setTimeout(async () => {
      try {
        const [check] = await db
          .select()
          .from(apiKeys)
          .where(eq(apiKeys.id, oldKeyId));
        if (check && check.isActive && !check.revokedAt) {
          await this.revoke(oldKeyId);
          log.info(`Auto-revoked old key ${oldKey.keyPrefix}*** after rotation grace period`);
        }
      } catch (err) {
        log.error(`Failed to auto-revoke rotated key ${oldKeyId}:`, err);
      }
    }, gracePeriodMs);

    log.info(
      `Key rotated: ${oldKey.keyPrefix}*** -> ${newKeyData.keyPrefix}*** (7-day grace period)`
    );

    return {
      newKey: newKeyData,
      oldKeyId,
      graceEnds: new Date(Date.now() + gracePeriodMs),
    };
  },

  async getExpiringKeys(withinDays: number = 14) {
    const cutoff = new Date(Date.now() + withinDays * 86400000);
    return db
      .select({
        id: apiKeys.id,
        keyPrefix: apiKeys.keyPrefix,
        name: apiKeys.name,
        owner: apiKeys.owner,
        expiresAt: apiKeys.expiresAt,
        rotationScheduledAt: apiKeys.rotationScheduledAt,
        rateLimitTier: apiKeys.rateLimitTier,
        rateLimitRpm: apiKeys.rateLimitRpm,
      })
      .from(apiKeys)
      .where(
        and(
          isNull(apiKeys.revokedAt),
          eq(apiKeys.isActive, true),
          lt(apiKeys.expiresAt, cutoff)
        )
      )
      .orderBy(apiKeys.expiresAt);
  },

  async updateRateLimit(
    keyId: string,
    tier: string,
    rpm: number
  ) {
    const [result] = await db
      .update(apiKeys)
      .set({ rateLimitTier: tier, rateLimitRpm: rpm })
      .where(eq(apiKeys.id, keyId))
      .returning();
    return result;
  },

  async checkRotationsDue() {
    const now = new Date();
    const dueKeys = await db
      .select()
      .from(apiKeys)
      .where(
        and(
          isNull(apiKeys.revokedAt),
          eq(apiKeys.isActive, true),
          lt(apiKeys.rotationScheduledAt, now)
        )
      );

    for (const key of dueKeys) {
      try {
        await this.rotateKey(key.id);
        log.info(`Auto-rotated key: ${key.keyPrefix}*** (${key.name})`);
      } catch (err) {
        log.error(`Failed to auto-rotate key ${key.id}:`, err);
      }
    }

    return dueKeys.length;
  },

  startRotationCron() {
    setInterval(async () => {
      try {
        const count = await this.checkRotationsDue();
        if (count > 0) {
          log.info(`Rotation check completed: ${count} key(s) rotated`);
        }
      } catch (err) {
        log.error("Rotation cron error:", err);
      }
    }, 6 * 60 * 60 * 1000);
    log.info("Key rotation cron started (every 6 hours)");
  },
};
