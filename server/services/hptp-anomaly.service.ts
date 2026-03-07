/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { db } from "../db";
import { hptpAnomalyEvents } from "@shared/schema";
import { eq, desc, and, gte, count, sql, lte } from "drizzle-orm";
import { createLogger } from "../logger";
import { securityAuditService } from "./security-audit.service";

const log = createLogger("hptp-anomaly");

type AnomalyType = "jitter_variance" | "clock_drift" | "sync_failure" | "glitch_detected";
type FallbackTier = "ptp" | "ntp" | "crystal" | "quartz" | "cesium";

interface FallbackChainEntry {
  status: "active" | "standby" | "failed";
  latency_ms?: number;
  jitter_variance?: number;
  frequency_ppm?: number;
  temperature_c?: number;
}

type FallbackChainData = {
  [key in FallbackTier]?: FallbackChainEntry;
};

const FALLBACK_TIER_METADATA = {
  ptp: { description: "IEEE 1588 PTP redundant stratum", switchoverTime: "<1ms", precision: "±100ns" },
  ntp: { description: "NTP GPS-disciplined fallback", switchoverTime: "<10ms", precision: "±1μs" },
  crystal: { description: "Local crystal oscillator holdover", switchoverTime: "immediate", precision: "±10μs/day drift" },
  quartz: { description: "Temperature-compensated quartz reference", switchoverTime: "immediate", precision: "±0.5ppm" },
  cesium: { description: "Cesium beam frequency standard", switchoverTime: "<100ms", precision: "±1×10⁻¹²" },
} as const;

export const hptpAnomalyService = {
  async reportAnomaly(params: {
    anomalyType: AnomalyType;
    severityScore: number;
    thresholdValue: number;
    observedValue: number;
    variancePercentage: number;
    fallbackChain: FallbackChainData;
    activeTier: FallbackTier;
  }) {
    // Validate severityScore is 0-10
    const clampedSeverityScore = Math.max(0, Math.min(10, params.severityScore));

    // Determine escalation based on severityScore
    let escalationTriggered = false;
    let auditLogId: number | null = null;
    let escalationTimestamp: Date | null = null;

    if (clampedSeverityScore >= 8.0) {
      escalationTriggered = true;
      escalationTimestamp = new Date();
      const auditEntry = await securityAuditService.logEvent({
        category: "hptp",
        eventType: "hptp_anomaly_escalation",
        severity: "critical",
        actor: "hptp-anomaly-detector",
        description: `HPTP anomaly detected: ${params.anomalyType} with severity score ${clampedSeverityScore.toFixed(2)}/10. Observed: ${params.observedValue}, Threshold: ${params.thresholdValue}`,
        affectedComponent: `fallback_tier_${params.activeTier}`,
        evidence: {
          anomalyType: params.anomalyType,
          severityScore: clampedSeverityScore,
          observedValue: params.observedValue,
          thresholdValue: params.thresholdValue,
          variancePercentage: params.variancePercentage,
          activeTier: params.activeTier,
          fallbackChain: params.fallbackChain,
        },
      });
      auditLogId = auditEntry.id;
    } else if (clampedSeverityScore >= 6.0) {
      escalationTriggered = true;
      escalationTimestamp = new Date();
      const auditEntry = await securityAuditService.logEvent({
        category: "hptp",
        eventType: "hptp_anomaly_escalation",
        severity: "high",
        actor: "hptp-anomaly-detector",
        description: `HPTP anomaly detected: ${params.anomalyType} with severity score ${clampedSeverityScore.toFixed(2)}/10. Observed: ${params.observedValue}, Threshold: ${params.thresholdValue}`,
        affectedComponent: `fallback_tier_${params.activeTier}`,
        evidence: {
          anomalyType: params.anomalyType,
          severityScore: clampedSeverityScore,
          observedValue: params.observedValue,
          thresholdValue: params.thresholdValue,
          variancePercentage: params.variancePercentage,
          activeTier: params.activeTier,
          fallbackChain: params.fallbackChain,
        },
      });
      auditLogId = auditEntry.id;
    } else if (clampedSeverityScore >= 4.0) {
      const auditEntry = await securityAuditService.logEvent({
        category: "hptp",
        eventType: "hptp_anomaly_escalation",
        severity: "warning",
        actor: "hptp-anomaly-detector",
        description: `HPTP anomaly detected: ${params.anomalyType} with severity score ${clampedSeverityScore.toFixed(2)}/10. Observed: ${params.observedValue}, Threshold: ${params.thresholdValue}`,
        affectedComponent: `fallback_tier_${params.activeTier}`,
        evidence: {
          anomalyType: params.anomalyType,
          severityScore: clampedSeverityScore,
          observedValue: params.observedValue,
          thresholdValue: params.thresholdValue,
          variancePercentage: params.variancePercentage,
          activeTier: params.activeTier,
          fallbackChain: params.fallbackChain,
        },
      });
      auditLogId = auditEntry.id;
    } else {
      // Info only - log but don't create audit event
      log.info("HPTP anomaly detected (info level)", {
        type: params.anomalyType,
        severityScore: clampedSeverityScore,
        tier: params.activeTier,
      });
    }

    // Insert anomaly event
    const [event] = await db
      .insert(hptpAnomalyEvents)
      .values({
        anomalyType: params.anomalyType,
        severityScore: clampedSeverityScore,
        thresholdValue: params.thresholdValue,
        observedValue: params.observedValue,
        variancePercentage: params.variancePercentage,
        fallbackChain: params.fallbackChain,
        activeTier: params.activeTier,
        escalationTriggered,
        escalationTimestamp,
        auditLogId,
        resolved: false,
        resolvedAt: null,
      })
      .returning();

    log.info("HPTP anomaly reported", {
      id: event.id,
      type: params.anomalyType,
      severityScore: clampedSeverityScore,
      escalated: escalationTriggered,
      tier: params.activeTier,
    });

    return event;
  },

  async getEvents(filters?: {
    anomalyType?: string;
    days?: number;
    limit?: number;
    offset?: number;
  }) {
    const conditions = [];

    if (filters?.anomalyType) {
      conditions.push(eq(hptpAnomalyEvents.anomalyType, filters.anomalyType));
    }

    if (filters?.days) {
      const sinceDate = new Date();
      sinceDate.setDate(sinceDate.getDate() - filters.days);
      conditions.push(gte(hptpAnomalyEvents.createdAt, sinceDate));
    }

    const query = conditions.length > 0
      ? db.select().from(hptpAnomalyEvents).where(and(...conditions))
      : db.select().from(hptpAnomalyEvents);

    return query
      .orderBy(desc(hptpAnomalyEvents.createdAt))
      .limit(filters?.limit || 100)
      .offset(filters?.offset || 0);
  },

  async getStatus() {
    const now = new Date();
    const oneDayAgo = new Date(now.getTime() - 24 * 60 * 60 * 1000);

    // Get most recent anomaly to determine current active tier
    const [latestAnomaly] = await db
      .select()
      .from(hptpAnomalyEvents)
      .orderBy(desc(hptpAnomalyEvents.createdAt))
      .limit(1);

    const activeTier = latestAnomaly?.activeTier || "ptp";
    const fallbackChain = latestAnomaly?.fallbackChain || {};

    // Count recent anomalies in last 24 hours
    const recentAnomaliesResult = await db
      .select({ count: count() })
      .from(hptpAnomalyEvents)
      .where(gte(hptpAnomalyEvents.createdAt, oneDayAgo));

    const recentAnomaliesCount = recentAnomaliesResult[0]?.count || 0;

    // Count escalations in last 24 hours
    const escalationCountResult = await db
      .select({ count: count() })
      .from(hptpAnomalyEvents)
      .where(
        and(
          gte(hptpAnomalyEvents.createdAt, oneDayAgo),
          eq(hptpAnomalyEvents.escalationTriggered, true)
        )
      );

    const escalationCount24h = escalationCountResult[0]?.count || 0;

    // Get last sync timestamp (most recent anomaly timestamp)
    const lastSync = latestAnomaly?.createdAt || null;

    return {
      activeTier,
      fallbackChain,
      recentAnomaliesCount,
      escalationCount24h,
      lastSync,
    };
  },

  async getFallbackAnalysis() {
    // Get performance metrics for each fallback tier
    const tiers: FallbackTier[] = ["ptp", "ntp", "crystal", "quartz", "cesium"];
    const analysis: Record<string, any> = {};

    for (const tier of tiers) {
      // Get all events using this tier
      const events = await db
        .select()
        .from(hptpAnomalyEvents)
        .where(eq(hptpAnomalyEvents.activeTier, tier))
        .orderBy(desc(hptpAnomalyEvents.createdAt));

      if (events.length === 0) {
        analysis[tier] = {
          description: FALLBACK_TIER_METADATA[tier].description,
          switchoverTime: FALLBACK_TIER_METADATA[tier].switchoverTime,
          precision: FALLBACK_TIER_METADATA[tier].precision,
          eventCount: 0,
          averageSeverityScore: null,
          escalationCount: 0,
          lastUsed: null,
        };
      } else {
        const escalationCount = events.filter(e => e.escalationTriggered).length;
        const avgSeverity = events.reduce((sum, e) => sum + e.severityScore, 0) / events.length;

        analysis[tier] = {
          description: FALLBACK_TIER_METADATA[tier].description,
          switchoverTime: FALLBACK_TIER_METADATA[tier].switchoverTime,
          precision: FALLBACK_TIER_METADATA[tier].precision,
          eventCount: events.length,
          averageSeverityScore: parseFloat(avgSeverity.toFixed(2)),
          escalationCount,
          lastUsed: events[0]?.createdAt,
          recentAnomalies: events.slice(0, 5).map(e => ({
            id: e.id,
            anomalyType: e.anomalyType,
            severityScore: e.severityScore,
            observedValue: e.observedValue,
            thresholdValue: e.thresholdValue,
            escalated: e.escalationTriggered,
            createdAt: e.createdAt,
          })),
        };
      }
    }

    return analysis;
  },

  async getStatistics(since?: Date) {
    const condition = since ? gte(hptpAnomalyEvents.createdAt, since) : undefined;

    // Group by anomalyType
    const byTypeResult = await db
      .select({
        anomalyType: hptpAnomalyEvents.anomalyType,
        count: count(),
      })
      .from(hptpAnomalyEvents)
      .where(condition)
      .groupBy(hptpAnomalyEvents.anomalyType);

    // Get escalation counts
    const escalationResult = await db
      .select({ count: count() })
      .from(hptpAnomalyEvents)
      .where(
        condition
          ? and(condition, eq(hptpAnomalyEvents.escalationTriggered, true))
          : eq(hptpAnomalyEvents.escalationTriggered, true)
      );

    // Get resolved counts
    const resolvedResult = await db
      .select({ count: count() })
      .from(hptpAnomalyEvents)
      .where(
        condition
          ? and(condition, eq(hptpAnomalyEvents.resolved, true))
          : eq(hptpAnomalyEvents.resolved, true)
      );

    // Calculate average severity across all events
    const severityResult = await db
      .select({
        avgSeverity: sql<number>`AVG(${hptpAnomalyEvents.severityScore})`,
      })
      .from(hptpAnomalyEvents)
      .where(condition);

    return {
      byType: Object.fromEntries(byTypeResult.map(r => [r.anomalyType, r.count])),
      totalEscalations: escalationResult[0]?.count || 0,
      totalResolved: resolvedResult[0]?.count || 0,
      averageSeverityScore: severityResult[0]?.avgSeverity ? parseFloat(severityResult[0].avgSeverity.toFixed(2)) : null,
    };
  },

  getThresholds() {
    return {
      jitter_variance: { warn: 50, critical: 200 },
      clock_drift: { warn: 0.1, critical: 1.0 },
      sync_failure: { warn: 100, critical: 1000 },
      glitch_detected: { warn: 5, critical: 45 },
    };
  },

  getFallbackModes() {
    return FALLBACK_TIER_METADATA;
  },

  getRedundancyArchitecture() {
    return {
      primary: {
        source: "HPTP Femtosecond Clock",
        precision: "±50 femtoseconds",
        protocol: "High-Precision Timing Protocol v2.1",
      },
      fallbackChain: [
        {
          tier: "ptp",
          level: 1,
          description: FALLBACK_TIER_METADATA.ptp.description,
          switchoverTime: FALLBACK_TIER_METADATA.ptp.switchoverTime,
          precision: FALLBACK_TIER_METADATA.ptp.precision,
        },
        {
          tier: "ntp",
          level: 2,
          description: FALLBACK_TIER_METADATA.ntp.description,
          switchoverTime: FALLBACK_TIER_METADATA.ntp.switchoverTime,
          precision: FALLBACK_TIER_METADATA.ntp.precision,
        },
        {
          tier: "crystal",
          level: 3,
          description: FALLBACK_TIER_METADATA.crystal.description,
          switchoverTime: FALLBACK_TIER_METADATA.crystal.switchoverTime,
          precision: FALLBACK_TIER_METADATA.crystal.precision,
        },
        {
          tier: "quartz",
          level: 4,
          description: FALLBACK_TIER_METADATA.quartz.description,
          switchoverTime: FALLBACK_TIER_METADATA.quartz.switchoverTime,
          precision: FALLBACK_TIER_METADATA.quartz.precision,
        },
        {
          tier: "cesium",
          level: 5,
          description: FALLBACK_TIER_METADATA.cesium.description,
          switchoverTime: FALLBACK_TIER_METADATA.cesium.switchoverTime,
          precision: FALLBACK_TIER_METADATA.cesium.precision,
        },
      ],
      monitoringIntervals: {
        normal: "100ms",
        elevated: "10ms",
        critical: "1ms",
      },
    };
  },
};
