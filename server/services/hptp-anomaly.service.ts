/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { db } from "../db";
import { hptpAnomalyEvents } from "@shared/schema";
import { eq, desc, and, gte, count, sql } from "drizzle-orm";
import { createLogger } from "../logger";
import { securityAuditService } from "./security-audit.service";

const log = createLogger("hptp-anomaly");

const ANOMALY_THRESHOLDS = {
  jitter_ns: { warn: 50, critical: 200 },
  drift_ppm: { warn: 0.1, critical: 1.0 },
  skew_fs: { warn: 500, critical: 5000 },
  sync_loss_ms: { warn: 100, critical: 1000 },
  phase_deviation_deg: { warn: 5, critical: 45 },
};

const FALLBACK_MODES = {
  ntp: "NTP GPS-disciplined fallback",
  ptp: "IEEE 1588 PTP redundant stratum",
  crystal: "Local crystal oscillator holdover",
  quartz: "Temperature-compensated quartz reference",
  cesium: "Cesium beam frequency standard",
} as const;

type FallbackMode = keyof typeof FALLBACK_MODES;

export const hptpAnomalyService = {
  async evaluateAndLog(params: {
    anomalyType: string;
    detectedValue: number;
    expectedValue: number;
    sensorId?: string;
    details?: Record<string, unknown>;
  }) {
    const deviationPercent = params.expectedValue !== 0
      ? Math.abs((params.detectedValue - params.expectedValue) / params.expectedValue) * 100
      : 100;

    const threshold = ANOMALY_THRESHOLDS[params.anomalyType as keyof typeof ANOMALY_THRESHOLDS];
    let severity: "low" | "medium" | "high" | "critical" = "low";
    let fallbackTriggered = false;
    let fallbackMode: FallbackMode | null = null;
    let mitigationApplied: string | null = null;

    if (threshold) {
      if (deviationPercent >= threshold.critical) {
        severity = "critical";
        fallbackTriggered = true;
        fallbackMode = selectFallback(params.anomalyType);
        mitigationApplied = `Engaged ${FALLBACK_MODES[fallbackMode]}; isolating sensor ${params.sensorId || "unknown"}`;
      } else if (deviationPercent >= threshold.warn) {
        severity = "high";
        fallbackTriggered = true;
        fallbackMode = "ptp";
        mitigationApplied = `Switched to redundant PTP source; monitoring sensor ${params.sensorId || "unknown"}`;
      } else if (deviationPercent >= threshold.warn * 0.5) {
        severity = "medium";
        mitigationApplied = `Increased monitoring frequency for sensor ${params.sensorId || "unknown"}`;
      }
    } else {
      if (deviationPercent > 50) severity = "critical";
      else if (deviationPercent > 20) severity = "high";
      else if (deviationPercent > 5) severity = "medium";
    }

    const [event] = await db
      .insert(hptpAnomalyEvents)
      .values({
        anomalyType: params.anomalyType,
        severity,
        detectedValue: params.detectedValue,
        expectedValue: params.expectedValue,
        deviationPercent,
        sensorId: params.sensorId || null,
        fallbackTriggered,
        fallbackMode: fallbackMode || null,
        mitigationApplied,
        details: params.details || null,
      })
      .returning();

    if (severity === "high" || severity === "critical") {
      await securityAuditService.logEvent({
        eventType: "hptp_fallback",
        severity,
        source: "hptp-anomaly-detector",
        message: `HPTP anomaly detected: ${params.anomalyType} deviation ${deviationPercent.toFixed(2)}% on sensor ${params.sensorId || "unknown"}`,
        details: {
          anomalyId: event.id,
          anomalyType: params.anomalyType,
          detected: params.detectedValue,
          expected: params.expectedValue,
          deviation: deviationPercent,
          fallbackMode,
          mitigation: mitigationApplied,
        },
      });
    }

    log.info("HPTP anomaly processed", {
      id: event.id,
      type: params.anomalyType,
      severity,
      deviation: `${deviationPercent.toFixed(2)}%`,
      fallback: fallbackTriggered,
    });

    return event;
  },

  async getEvents(filters?: {
    anomalyType?: string;
    severity?: string;
    fallbackTriggered?: boolean;
    since?: Date;
    limit?: number;
    offset?: number;
  }) {
    const conditions = [];
    if (filters?.anomalyType) conditions.push(eq(hptpAnomalyEvents.anomalyType, filters.anomalyType));
    if (filters?.severity) conditions.push(eq(hptpAnomalyEvents.severity, filters.severity));
    if (filters?.fallbackTriggered !== undefined) conditions.push(eq(hptpAnomalyEvents.fallbackTriggered, filters.fallbackTriggered));
    if (filters?.since) conditions.push(gte(hptpAnomalyEvents.createdAt, filters.since));

    const query = conditions.length > 0
      ? db.select().from(hptpAnomalyEvents).where(and(...conditions))
      : db.select().from(hptpAnomalyEvents);

    return query
      .orderBy(desc(hptpAnomalyEvents.createdAt))
      .limit(filters?.limit || 100)
      .offset(filters?.offset || 0);
  },

  async getStatistics(since?: Date) {
    const condition = since ? gte(hptpAnomalyEvents.createdAt, since) : undefined;

    const severityCounts = await db
      .select({
        severity: hptpAnomalyEvents.severity,
        count: count(),
      })
      .from(hptpAnomalyEvents)
      .where(condition)
      .groupBy(hptpAnomalyEvents.severity);

    const typeCounts = await db
      .select({
        anomalyType: hptpAnomalyEvents.anomalyType,
        count: count(),
      })
      .from(hptpAnomalyEvents)
      .where(condition)
      .groupBy(hptpAnomalyEvents.anomalyType);

    const fallbackCount = await db
      .select({ count: count() })
      .from(hptpAnomalyEvents)
      .where(and(condition, eq(hptpAnomalyEvents.fallbackTriggered, true)));

    return {
      bySeverity: Object.fromEntries(severityCounts.map(r => [r.severity, r.count])),
      byType: Object.fromEntries(typeCounts.map(r => [r.anomalyType, r.count])),
      totalFallbacks: fallbackCount[0]?.count || 0,
    };
  },

  getThresholds() {
    return ANOMALY_THRESHOLDS;
  },

  getFallbackModes() {
    return FALLBACK_MODES;
  },

  getRedundancyArchitecture() {
    return {
      primary: {
        source: "HPTP Femtosecond Clock",
        precision: "±50 femtoseconds",
        protocol: "High-Precision Timing Protocol v2.1",
      },
      fallbackChain: [
        { level: 1, mode: "ptp", description: FALLBACK_MODES.ptp, switchoverTime: "<1ms", precision: "±100ns" },
        { level: 2, mode: "ntp", description: FALLBACK_MODES.ntp, switchoverTime: "<10ms", precision: "±1μs" },
        { level: 3, mode: "crystal", description: FALLBACK_MODES.crystal, switchoverTime: "immediate", precision: "±10μs/day drift" },
        { level: 4, mode: "quartz", description: FALLBACK_MODES.quartz, switchoverTime: "immediate", precision: "±0.5ppm" },
        { level: 5, mode: "cesium", description: FALLBACK_MODES.cesium, switchoverTime: "<100ms", precision: "±1×10⁻¹²" },
      ],
      monitoringIntervals: {
        normal: "100ms",
        elevated: "10ms",
        critical: "1ms",
      },
    };
  },
};

function selectFallback(anomalyType: string): FallbackMode {
  switch (anomalyType) {
    case "jitter_ns":
      return "ptp";
    case "drift_ppm":
      return "cesium";
    case "skew_fs":
      return "crystal";
    case "sync_loss_ms":
      return "ntp";
    case "phase_deviation_deg":
      return "quartz";
    default:
      return "ptp";
  }
}
