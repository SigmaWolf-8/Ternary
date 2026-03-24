/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { db } from "../db";
import { securityAuditLog } from "@shared/schema";
import { eq, desc, and, gte, ne, sql, count } from "drizzle-orm";
import { createLogger } from "../logger";
import { phaseEncryptFields, phaseDecryptFields } from "../storage";

const log = createLogger("security-audit");

export type AuditSeverity = "info" | "warning" | "high" | "critical";
export type AuditCategory = "auth" | "crypto" | "boot" | "network" | "hptp" | "firmware" | "privilege";
export type ResolutionStatus = "resolved" | "false_positive" | "acknowledged";
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
  | "hptp_anomaly_escalation"
  | "compliance_violation";

export const securityAuditService = {
  async logEvent(params: {
    severity: AuditSeverity;
    category: AuditCategory;
    eventType: string;
    actor?: string;
    description: string;
    affectedComponent?: string;
    evidence?: Record<string, unknown>;
    ipAddress?: string;
    userId?: string;
  }) {
    try {
      const encryptedFields = phaseEncryptFields({
        actor: params.actor || null,
        description: params.description,
        evidence: params.evidence || null,
        ipAddress: params.ipAddress || null,
        userId: params.userId || null,
      });
      const [entry] = await db
        .insert(securityAuditLog)
        .values({
          severity: params.severity,
          category: params.category,
          eventType: params.eventType,
          actor: params.actor || null,
          description: params.description,
          affectedComponent: params.affectedComponent || null,
          evidence: params.evidence || null,
          ipAddress: params.ipAddress || null,
          userId: params.userId || null,
          resolutionStatus: "unresolved",
          encryptedFields,
        })
        .returning();
      log.info("Security event logged", { id: entry.id, type: params.eventType, severity: params.severity, category: params.category });
      return entry;
    } catch (err: any) {
      log.error("Failed to log security event", { error: err.message });
      throw err;
    }
  },

  async getEvents(filters?: {
    severity?: AuditSeverity;
    category?: AuditCategory;
    eventType?: string;
    since?: Date;
    limit?: number;
    offset?: number;
  }) {
    const conditions = [];
    if (filters?.severity) conditions.push(eq(securityAuditLog.severity, filters.severity));
    if (filters?.category) conditions.push(eq(securityAuditLog.category, filters.category));
    if (filters?.eventType) conditions.push(eq(securityAuditLog.eventType, filters.eventType));
    if (filters?.since) conditions.push(gte(securityAuditLog.createdAt, filters.since));

    const query = conditions.length > 0
      ? db.select().from(securityAuditLog).where(and(...conditions))
      : db.select().from(securityAuditLog);

    const rows = await query
      .orderBy(desc(securityAuditLog.createdAt))
      .limit(filters?.limit || 100)
      .offset(filters?.offset || 0);
    return rows.map(row => {
      const dec = phaseDecryptFields(row.encryptedFields);
      if (dec) {
        if (dec.actor) row.actor = dec.actor as string;
        if (dec.description) row.description = dec.description as string;
        if (dec.ipAddress) row.ipAddress = dec.ipAddress as string;
        if (dec.userId) row.userId = dec.userId as string;
      }
      return row;
    });
  },

  async getEventById(id: number) {
    const [event] = await db
      .select()
      .from(securityAuditLog)
      .where(eq(securityAuditLog.id, id));
    if (event) {
      const dec = phaseDecryptFields(event.encryptedFields);
      if (dec) {
        if (dec.actor) event.actor = dec.actor as string;
        if (dec.description) event.description = dec.description as string;
        if (dec.ipAddress) event.ipAddress = dec.ipAddress as string;
        if (dec.userId) event.userId = dec.userId as string;
      }
    }
    return event;
  },

  async resolveEvent(id: number, params: {
    resolutionStatus: ResolutionStatus;
    resolutionNotes?: string;
    resolvedBy?: string;
  }) {
    try {
      const [updated] = await db
        .update(securityAuditLog)
        .set({
          resolutionStatus: params.resolutionStatus,
          resolutionNotes: params.resolutionNotes || null,
          resolvedBy: params.resolvedBy || null,
          resolvedAt: new Date(),
        })
        .where(eq(securityAuditLog.id, id))
        .returning();
      log.info("Security event resolved", { id, status: params.resolutionStatus });
      return updated;
    } catch (err: any) {
      log.error("Failed to resolve security event", { id, error: err.message });
      throw err;
    }
  },

  async getSeverityCounts(since?: Date) {
    const severityCondition = since ? gte(securityAuditLog.createdAt, since) : undefined;
    
    const severityRows = await db
      .select({
        severity: securityAuditLog.severity,
        count: count(),
      })
      .from(securityAuditLog)
      .where(severityCondition)
      .groupBy(securityAuditLog.severity);

    const categoryRows = await db
      .select({
        category: securityAuditLog.category,
        count: count(),
      })
      .from(securityAuditLog)
      .where(severityCondition)
      .groupBy(securityAuditLog.category);

    const bySeverity: Record<string, number> = { info: 0, warning: 0, high: 0, critical: 0 };
    const byCategory: Record<string, number> = { auth: 0, crypto: 0, boot: 0, network: 0, hptp: 0, firmware: 0, privilege: 0 };

    for (const row of severityRows) {
      bySeverity[row.severity] = row.count;
    }

    for (const row of categoryRows) {
      byCategory[row.category] = row.count;
    }

    return {
      by_severity: bySeverity,
      by_category: byCategory,
    };
  },

  async getUnresolved(severity?: AuditSeverity) {
    const conditions = [ne(securityAuditLog.resolutionStatus, "resolved")];
    if (severity) {
      conditions.push(eq(securityAuditLog.severity, severity));
    }

    const rows = await db
      .select()
      .from(securityAuditLog)
      .where(and(...conditions))
      .orderBy(desc(securityAuditLog.createdAt));
    return rows.map(row => {
      const dec = phaseDecryptFields(row.encryptedFields);
      if (dec) {
        if (dec.actor) row.actor = dec.actor as string;
        if (dec.description) row.description = dec.description as string;
        if (dec.ipAddress) row.ipAddress = dec.ipAddress as string;
        if (dec.userId) row.userId = dec.userId as string;
      }
      return row;
    });
  },
};
