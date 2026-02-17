/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { db } from "../db";
import { securityAuditLog } from "@shared/schema";
import { eq, desc, and, gte, sql, count } from "drizzle-orm";
import { createLogger } from "../logger";

const log = createLogger("security-audit");

export type AuditSeverity = "low" | "medium" | "high" | "critical";
export type AuditEventType =
  | "auth_failure"
  | "rate_limit_exceeded"
  | "scope_violation"
  | "key_revocation"
  | "anomaly_detected"
  | "threat_mitigated"
  | "config_change"
  | "privilege_escalation"
  | "data_access"
  | "encryption_failure"
  | "hptp_fallback"
  | "compliance_violation";

export const securityAuditService = {
  async logEvent(params: {
    eventType: string;
    severity: AuditSeverity;
    source: string;
    message: string;
    details?: Record<string, unknown>;
    ipAddress?: string;
    userId?: string;
  }) {
    try {
      const [entry] = await db
        .insert(securityAuditLog)
        .values({
          eventType: params.eventType,
          severity: params.severity,
          source: params.source,
          message: params.message,
          details: params.details || null,
          ipAddress: params.ipAddress || null,
          userId: params.userId || null,
        })
        .returning();
      log.info("Security event logged", { id: entry.id, type: params.eventType, severity: params.severity });
      return entry;
    } catch (err: any) {
      log.error("Failed to log security event", { error: err.message });
      throw err;
    }
  },

  async getEvents(filters?: {
    severity?: string;
    eventType?: string;
    source?: string;
    resolved?: boolean;
    since?: Date;
    limit?: number;
    offset?: number;
  }) {
    const conditions = [];
    if (filters?.severity) conditions.push(eq(securityAuditLog.severity, filters.severity));
    if (filters?.eventType) conditions.push(eq(securityAuditLog.eventType, filters.eventType));
    if (filters?.source) conditions.push(eq(securityAuditLog.source, filters.source));
    if (filters?.resolved !== undefined) conditions.push(eq(securityAuditLog.resolved, filters.resolved));
    if (filters?.since) conditions.push(gte(securityAuditLog.createdAt, filters.since));

    const query = conditions.length > 0
      ? db.select().from(securityAuditLog).where(and(...conditions))
      : db.select().from(securityAuditLog);

    return query
      .orderBy(desc(securityAuditLog.createdAt))
      .limit(filters?.limit || 100)
      .offset(filters?.offset || 0);
  },

  async resolveEvent(id: number) {
    const [updated] = await db
      .update(securityAuditLog)
      .set({ resolved: true })
      .where(eq(securityAuditLog.id, id))
      .returning();
    return updated;
  },

  async getSeverityCounts(since?: Date) {
    const condition = since ? gte(securityAuditLog.createdAt, since) : undefined;
    const rows = await db
      .select({
        severity: securityAuditLog.severity,
        count: count(),
      })
      .from(securityAuditLog)
      .where(condition)
      .groupBy(securityAuditLog.severity);

    const result: Record<string, number> = { low: 0, medium: 0, high: 0, critical: 0 };
    for (const row of rows) {
      result[row.severity] = row.count;
    }
    return result;
  },

  async getUnresolved() {
    return db
      .select()
      .from(securityAuditLog)
      .where(eq(securityAuditLog.resolved, false))
      .orderBy(desc(securityAuditLog.createdAt));
  },
};
