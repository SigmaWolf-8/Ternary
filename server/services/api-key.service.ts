/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import crypto from "crypto";
import { db } from "../db";
import { apiKeys, apiKeyLogs, apiKeyAuditEvents } from "@shared/schema";
import { eq, desc, and, lt, isNull, sql, gte } from "drizzle-orm";
import { createLogger } from "../logger";
import { phaseEncryptFields, phaseDecryptFields } from "../storage";

const log = createLogger("api-key-service");

const KEY_PREFIX = "plm_";

function hashKey(rawKey: string): string {
  return crypto.createHash("sha256").update(rawKey).digest("hex");
}

function constantTimeCompare(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  return crypto.timingSafeEqual(Buffer.from(a), Buffer.from(b));
}

import { getAllScopes, isValidScope } from "@shared/scopes";

const LEGACY_SCOPES = [
  "read:ternary",
  "write:ternary",
  "read:phase",
  "write:phase",
  "read:hptp",
  "write:hptp",
  "read:compression",
  "write:compression",
  "read:plenumdb",
  "write:plenumdb",
  "read:calendar",
  "read:agent-array",
  "write:agent-array",
  "read:whitepaper",
  "admin:keys",
] as const;

export const AVAILABLE_SCOPES = getAllScopes();

export type ApiKeyScope = string;

export function isScopeValid(scope: string): boolean {
  return isValidScope(scope) || (LEGACY_SCOPES as readonly string[]).includes(scope);
}

export const apiKeyService = {
  async generate(
    owner: string,
    name: string,
    scopes: string[],
    expiresDays: number = 90,
    wbs?: { entityType?: string; entityName?: string; project?: string; department?: string; tags?: string[]; notes?: string }
  ) {
    const rawKey = crypto.randomBytes(32).toString("hex");
    const fullKey = `${KEY_PREFIX}${rawKey}`;
    const keyHash = hashKey(rawKey);
    const keyPrefix = fullKey.substring(0, 8);

    const encryptedFields = phaseEncryptFields({
      name, owner, scopes,
      entityName: wbs?.entityName || null,
      project: wbs?.project || null,
      department: wbs?.department || null,
      tags: wbs?.tags || [],
      notes: wbs?.notes || null,
    });
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
        entityType: wbs?.entityType || null,
        entityName: wbs?.entityName || null,
        project: wbs?.project || null,
        department: wbs?.department || null,
        tags: wbs?.tags || [],
        notes: wbs?.notes || null,
        encryptedFields,
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

    const decrypted = phaseDecryptFields(keyRecord.encryptedFields);
    if (decrypted) {
      if (decrypted.name !== undefined) keyRecord.name = decrypted.name as string;
      if (decrypted.owner !== undefined) keyRecord.owner = decrypted.owner as string;
      if (decrypted.scopes !== undefined) keyRecord.scopes = decrypted.scopes as string[];
      if (decrypted.entityName !== undefined) keyRecord.entityName = decrypted.entityName as string | null;
      if (decrypted.project !== undefined) keyRecord.project = decrypted.project as string | null;
      if (decrypted.department !== undefined) keyRecord.department = decrypted.department as string | null;
      if (decrypted.tags !== undefined) keyRecord.tags = decrypted.tags as string[];
      if (decrypted.notes !== undefined) keyRecord.notes = decrypted.notes as string | null;
    }

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
    const encryptedFields = phaseEncryptFields({ ipAddress, endpoint });
    await db.insert(apiKeyLogs).values({
      keyId,
      endpoint,
      method,
      statusCode,
      ipAddress,
      encryptedFields,
    });
  },

  async listByOwner(owner: string) {
    const rows = await db
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
        entityType: apiKeys.entityType,
        entityName: apiKeys.entityName,
        project: apiKeys.project,
        department: apiKeys.department,
        tags: apiKeys.tags,
        notes: apiKeys.notes,
        createdAt: apiKeys.createdAt,
        encryptedFields: apiKeys.encryptedFields,
      })
      .from(apiKeys)
      .where(eq(apiKeys.owner, owner))
      .orderBy(desc(apiKeys.createdAt));
    return rows.map(({ encryptedFields: ef, ...row }) => {
      const dec = phaseDecryptFields(ef);
      if (dec) {
        if (dec.name !== undefined) row.name = dec.name as string;
        if (dec.owner !== undefined) row.owner = dec.owner as string;
        if (dec.scopes !== undefined) row.scopes = dec.scopes as string[];
        if (dec.entityName !== undefined) row.entityName = dec.entityName as string | null;
        if (dec.project !== undefined) row.project = dec.project as string | null;
        if (dec.department !== undefined) row.department = dec.department as string | null;
        if (dec.tags !== undefined) row.tags = dec.tags as string[];
        if (dec.notes !== undefined) row.notes = dec.notes as string | null;
      }
      return row;
    });
  },

  async listAll() {
    const rows = await db
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
        entityType: apiKeys.entityType,
        entityName: apiKeys.entityName,
        project: apiKeys.project,
        department: apiKeys.department,
        tags: apiKeys.tags,
        notes: apiKeys.notes,
        createdAt: apiKeys.createdAt,
        encryptedFields: apiKeys.encryptedFields,
      })
      .from(apiKeys)
      .orderBy(desc(apiKeys.createdAt));
    return rows.map(({ encryptedFields: ef, ...row }) => {
      const dec = phaseDecryptFields(ef);
      if (dec) {
        if (dec.name !== undefined) row.name = dec.name as string;
        if (dec.owner !== undefined) row.owner = dec.owner as string;
        if (dec.scopes !== undefined) row.scopes = dec.scopes as string[];
        if (dec.entityName !== undefined) row.entityName = dec.entityName as string | null;
        if (dec.project !== undefined) row.project = dec.project as string | null;
        if (dec.department !== undefined) row.department = dec.department as string | null;
        if (dec.tags !== undefined) row.tags = dec.tags as string[];
        if (dec.notes !== undefined) row.notes = dec.notes as string | null;
      }
      return row;
    });
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
    const rows = await db
      .select()
      .from(apiKeyLogs)
      .where(eq(apiKeyLogs.keyId, keyId))
      .orderBy(desc(apiKeyLogs.createdAt))
      .limit(limit);
    return rows.map(row => {
      const dec = phaseDecryptFields(row.encryptedFields);
      if (dec) {
        if (dec.ipAddress) row.ipAddress = dec.ipAddress as string;
        if (dec.endpoint) row.endpoint = dec.endpoint as string;
      }
      return row;
    });
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
          await this.logAuditEvent(oldKeyId, "auto_revoked", "system", null, {
            keyPrefix: oldKey.keyPrefix,
            keyName: oldKey.name,
            reason: "rotation_grace_period_expired",
            newKeyId: newKeyData.id,
          }, null);
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
    const rows = await db
      .select({
        id: apiKeys.id,
        keyPrefix: apiKeys.keyPrefix,
        name: apiKeys.name,
        owner: apiKeys.owner,
        expiresAt: apiKeys.expiresAt,
        rotationScheduledAt: apiKeys.rotationScheduledAt,
        rateLimitTier: apiKeys.rateLimitTier,
        rateLimitRpm: apiKeys.rateLimitRpm,
        encryptedFields: apiKeys.encryptedFields,
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
    return rows.map(({ encryptedFields: ef, ...row }) => {
      const dec = phaseDecryptFields(ef);
      if (dec) {
        if (dec.name !== undefined) row.name = dec.name as string;
        if (dec.owner !== undefined) row.owner = dec.owner as string;
      }
      return row;
    });
  },

  async updateKeyMetadata(
    keyId: string,
    updates: {
      name?: string;
      entityType?: string | null;
      entityName?: string | null;
      project?: string | null;
      department?: string | null;
      tags?: string[];
      notes?: string | null;
    }
  ) {
    const setValues: Record<string, unknown> = {};
    if (updates.name !== undefined) setValues.name = updates.name;
    if (updates.entityType !== undefined) setValues.entityType = updates.entityType;
    if (updates.entityName !== undefined) setValues.entityName = updates.entityName;
    if (updates.project !== undefined) setValues.project = updates.project;
    if (updates.department !== undefined) setValues.department = updates.department;
    if (updates.tags !== undefined) setValues.tags = updates.tags;
    if (updates.notes !== undefined) setValues.notes = updates.notes;

    if (Object.keys(setValues).length === 0) return null;

    const [current] = await db.select().from(apiKeys).where(eq(apiKeys.id, keyId));
    if (current) {
      const merged = {
        name: (updates.name ?? current.name) as string,
        owner: current.owner,
        scopes: current.scopes,
        entityName: (updates.entityName ?? current.entityName) as string | null,
        project: (updates.project ?? current.project) as string | null,
        department: (updates.department ?? current.department) as string | null,
        tags: (updates.tags ?? current.tags) as string[],
        notes: (updates.notes ?? current.notes) as string | null,
      };
      setValues.encryptedFields = phaseEncryptFields(merged);
    }

    const [result] = await db
      .update(apiKeys)
      .set(setValues)
      .where(eq(apiKeys.id, keyId))
      .returning();
    return result;
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

  async logAuditEvent(
    keyId: string,
    eventType: string,
    actorId: string,
    actorEmail: string | null,
    details: Record<string, unknown> | null,
    ipAddress: string | null
  ) {
    const encryptedFields = phaseEncryptFields({ actorId, actorEmail, details, ipAddress });
    await db.insert(apiKeyAuditEvents).values({
      keyId,
      eventType,
      actorId,
      actorEmail,
      details,
      ipAddress,
      encryptedFields,
    });
  },

  async getAuditEvents(keyId: string, limit: number = 50) {
    const rows = await db
      .select()
      .from(apiKeyAuditEvents)
      .where(eq(apiKeyAuditEvents.keyId, keyId))
      .orderBy(desc(apiKeyAuditEvents.createdAt))
      .limit(limit);
    return rows.map(row => {
      const dec = phaseDecryptFields(row.encryptedFields);
      if (dec) {
        if (dec.actorId !== undefined) row.actorId = dec.actorId as string;
        if (dec.actorEmail !== undefined) row.actorEmail = dec.actorEmail as string;
        if (dec.details !== undefined) row.details = dec.details as Record<string, unknown> | null;
        if (dec.ipAddress !== undefined) row.ipAddress = dec.ipAddress as string;
      }
      return row;
    });
  },

  async getRecentAuditEvents(limit: number = 100) {
    const rows = await db
      .select()
      .from(apiKeyAuditEvents)
      .orderBy(desc(apiKeyAuditEvents.createdAt))
      .limit(limit);
    return rows.map(row => {
      const dec = phaseDecryptFields(row.encryptedFields);
      if (dec) {
        if (dec.actorId !== undefined) row.actorId = dec.actorId as string;
        if (dec.actorEmail !== undefined) row.actorEmail = dec.actorEmail as string;
        if (dec.details !== undefined) row.details = dec.details as Record<string, unknown> | null;
        if (dec.ipAddress !== undefined) row.ipAddress = dec.ipAddress as string;
      }
      return row;
    });
  },

  async detectAnomalies(withinDays: number = 7) {
    const since = new Date(Date.now() - withinDays * 86400000);
    const anomalies: Array<{
      keyId: string;
      keyName: string;
      keyPrefix: string;
      type: string;
      severity: string;
      description: string;
      date: string;
      value: number;
    }> = [];

    const activeKeys = await db
      .select()
      .from(apiKeys)
      .where(and(eq(apiKeys.isActive, true), isNull(apiKeys.revokedAt)));

    for (const key of activeKeys) {
      const logs = await db
        .select({
          date: sql<string>`date_trunc('day', ${apiKeyLogs.createdAt})::text`,
          count: sql<number>`count(*)::int`,
        })
        .from(apiKeyLogs)
        .where(
          and(
            eq(apiKeyLogs.keyId, key.id),
            gte(apiKeyLogs.createdAt, since)
          )
        )
        .groupBy(sql`date_trunc('day', ${apiKeyLogs.createdAt})`)
        .orderBy(sql`date_trunc('day', ${apiKeyLogs.createdAt})`);

      for (let i = 1; i < logs.length; i++) {
        const prev = logs[i - 1];
        const curr = logs[i];
        if (prev.count > 0) {
          const pct = ((curr.count - prev.count) / prev.count) * 100;
          if (pct > 300 && curr.count > 20) {
            anomalies.push({
              keyId: key.id,
              keyName: key.name,
              keyPrefix: key.keyPrefix,
              type: "usage_spike",
              severity: pct > 1000 ? "high" : pct > 500 ? "medium" : "low",
              description: `${Math.round(pct)}% usage increase day-over-day (${prev.count} → ${curr.count})`,
              date: curr.date,
              value: Math.round(pct),
            });
          }
        }
      }

      const failedValidations = await db
        .select({ count: sql<number>`count(*)::int` })
        .from(apiKeyLogs)
        .where(
          and(
            eq(apiKeyLogs.keyId, key.id),
            gte(apiKeyLogs.createdAt, since),
            sql`${apiKeyLogs.statusCode} >= 400`
          )
        );

      if (failedValidations[0]?.count > 50) {
        anomalies.push({
          keyId: key.id,
          keyName: key.name,
          keyPrefix: key.keyPrefix,
          type: "high_failure_rate",
          severity: failedValidations[0].count > 200 ? "high" : "medium",
          description: `${failedValidations[0].count} failed requests in ${withinDays} days`,
          date: new Date().toISOString(),
          value: failedValidations[0].count,
        });
      }

      const distinctIps = await db
        .select({ count: sql<number>`count(DISTINCT ${apiKeyLogs.ipAddress})::int` })
        .from(apiKeyLogs)
        .where(
          and(
            eq(apiKeyLogs.keyId, key.id),
            gte(apiKeyLogs.createdAt, new Date(Date.now() - 86400000)),
            sql`${apiKeyLogs.ipAddress} IS NOT NULL`
          )
        );

      if (distinctIps[0]?.count > 10) {
        anomalies.push({
          keyId: key.id,
          keyName: key.name,
          keyPrefix: key.keyPrefix,
          type: "ip_dispersion",
          severity: distinctIps[0].count > 25 ? "high" : "medium",
          description: `Key used from ${distinctIps[0].count} distinct IPs in 24 hours`,
          date: new Date().toISOString(),
          value: distinctIps[0].count,
        });
      }
    }

    const tierChanges = await db
      .select()
      .from(apiKeyAuditEvents)
      .where(
        and(
          eq(apiKeyAuditEvents.eventType, "tier_change"),
          gte(apiKeyAuditEvents.createdAt, since)
        )
      )
      .orderBy(desc(apiKeyAuditEvents.createdAt));

    for (const tc of tierChanges) {
      const details = tc.details as any;
      if (details?.fromTier && details?.toTier) {
        const tierOrder = { research: 0, pro: 1, admin: 2 };
        const from = tierOrder[details.fromTier as keyof typeof tierOrder] ?? 0;
        const to = tierOrder[details.toTier as keyof typeof tierOrder] ?? 0;
        if (to > from) {
          anomalies.push({
            keyId: tc.keyId,
            keyName: details.keyName || tc.keyId,
            keyPrefix: details.keyPrefix || "",
            type: "tier_escalation",
            severity: to === 2 ? "medium" : "low",
            description: `Tier escalated from ${details.fromTier} to ${details.toTier} by ${tc.actorEmail || tc.actorId}`,
            date: tc.createdAt.toISOString(),
            value: to - from,
          });
        }
      }
    }

    anomalies.sort((a, b) => {
      const sev = { high: 3, medium: 2, low: 1 };
      return (sev[b.severity as keyof typeof sev] || 0) - (sev[a.severity as keyof typeof sev] || 0);
    });

    return anomalies;
  },
};
