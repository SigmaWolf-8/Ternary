/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import crypto from "crypto";
import { db } from "../db";
import { apiKeys, apiKeyLogs } from "@shared/schema";
import { eq, desc, and, sql } from "drizzle-orm";

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
};
