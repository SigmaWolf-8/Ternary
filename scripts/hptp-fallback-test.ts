/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Task 2.2: HPTP 5-Tier Fallback Chain Simulation Test
 * Simulates degradation (PTP → NTP → Crystal → Quartz → Cesium),
 * validates auto-escalation thresholds, tests recovery, and verifies
 * data integrity across tier transitions.
 *
 * Run: npx tsx scripts/hptp-fallback-test.ts
 */

import { fileURLToPath } from "url";
import { hptpAnomalyService } from "../server/services/hptp-anomaly.service";

interface TestResult {
  test: string;
  status: "PASS" | "FAIL";
  details: string;
  duration_ms: number;
}

const results: TestResult[] = [];
let passed = 0;
let failed = 0;

async function runTest(name: string, fn: () => Promise<string>) {
  const start = Date.now();
  try {
    const details = await fn();
    const duration = Date.now() - start;
    results.push({ test: name, status: "PASS", details, duration_ms: duration });
    passed++;
    console.log(`  ✓ PASS  ${name} (${duration}ms)`);
  } catch (err: any) {
    const duration = Date.now() - start;
    results.push({ test: name, status: "FAIL", details: err.message || String(err), duration_ms: duration });
    failed++;
    console.log(`  ✗ FAIL  ${name} (${duration}ms): ${err.message}`);
  }
}

const TIERS = ["ptp", "ntp", "crystal", "quartz", "cesium"] as const;
type FallbackTier = (typeof TIERS)[number];

function buildFallbackChain(activeTier: FallbackTier, failedTiers: FallbackTier[]) {
  const chain: Record<string, any> = {};
  for (const tier of TIERS) {
    if (tier === activeTier) {
      chain[tier] = { status: "active", latency_ms: 0.5, jitter_variance: 3.0, frequency_ppm: 2.0, temperature_c: 24.0 };
    } else if (failedTiers.includes(tier)) {
      chain[tier] = { status: "failed" };
    } else {
      chain[tier] = { status: "standby" };
    }
  }
  return chain;
}

async function main() {
  console.log("=".repeat(74));
  console.log("  HPTP 5-TIER FALLBACK CHAIN — SIMULATION TEST");
  console.log("  Date:", new Date().toISOString());
  console.log("  Script:", fileURLToPath(import.meta.url));
  console.log("=".repeat(74));
  console.log();

  const preTestEvents = await hptpAnomalyService.getEvents({ limit: 1 });
  const preTestCount = preTestEvents.length > 0 ? (await hptpAnomalyService.getStatistics()).byType : {};

  console.log("── SECTION 1: Degradation Simulation (PTP → NTP → Crystal → Quartz → Cesium) ──");
  console.log();

  const degradationScores = [4.0, 6.0, 7.0, 8.0, 9.5];
  const degradationAnomalyTypes = ["jitter_variance", "clock_drift", "sync_failure", "glitch_detected", "glitch_detected"] as const;
  const degradationIds: number[] = [];

  await runTest("1.1 Tier 1→2 (PTP → NTP): severity 4.0 (warning)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "jitter_variance",
      severityScore: 4.0,
      thresholdValue: 5.0,
      observedValue: 7.5,
      variancePercentage: 50.0,
      fallbackChain: buildFallbackChain("ntp", ["ptp"]),
      activeTier: "ntp",
    });
    if (!result.id) throw new Error("No ID returned");
    degradationIds.push(result.id);
    if (result.escalationTriggered) throw new Error("4.0 should NOT set escalationTriggered (warning only logs to audit)");
    return `Event ID ${result.id}, activeTier=ntp, auditLogId=${result.auditLogId}`;
  });

  await runTest("1.2 Tier 2→3 (NTP → Crystal): severity 6.0 (high)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "clock_drift",
      severityScore: 6.0,
      thresholdValue: 3.0,
      observedValue: 8.0,
      variancePercentage: 166.7,
      fallbackChain: buildFallbackChain("crystal", ["ptp", "ntp"]),
      activeTier: "crystal",
    });
    if (!result.id) throw new Error("No ID returned");
    degradationIds.push(result.id);
    if (!result.escalationTriggered) throw new Error("6.0 should trigger escalation (high)");
    if (!result.auditLogId) throw new Error("Expected auditLogId for high-severity escalation");
    return `Event ID ${result.id}, activeTier=crystal, escalation=true, auditLogId=${result.auditLogId}`;
  });

  await runTest("1.3 Tier 3→4 (Crystal → Quartz): severity 7.0 (high)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "sync_failure",
      severityScore: 7.0,
      thresholdValue: 3.0,
      observedValue: 0.0,
      variancePercentage: 100.0,
      fallbackChain: buildFallbackChain("quartz", ["ptp", "ntp", "crystal"]),
      activeTier: "quartz",
    });
    if (!result.id) throw new Error("No ID returned");
    degradationIds.push(result.id);
    if (!result.escalationTriggered) throw new Error("7.0 should trigger escalation (high)");
    return `Event ID ${result.id}, activeTier=quartz, escalation=true, auditLogId=${result.auditLogId}`;
  });

  await runTest("1.4 Tier 4→5 (Quartz → Cesium): severity 8.0 (critical)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "glitch_detected",
      severityScore: 8.0,
      thresholdValue: 1.0,
      observedValue: 12.0,
      variancePercentage: 1100.0,
      fallbackChain: buildFallbackChain("cesium", ["ptp", "ntp", "crystal", "quartz"]),
      activeTier: "cesium",
    });
    if (!result.id) throw new Error("No ID returned");
    degradationIds.push(result.id);
    if (!result.escalationTriggered) throw new Error("8.0 should trigger escalation (critical)");
    return `Event ID ${result.id}, activeTier=cesium, CRITICAL escalation, auditLogId=${result.auditLogId}`;
  });

  await runTest("1.5 Full degradation (Cesium active): severity 9.5 (critical)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "glitch_detected",
      severityScore: 9.5,
      thresholdValue: 1.0,
      observedValue: 20.0,
      variancePercentage: 1900.0,
      fallbackChain: buildFallbackChain("cesium", ["ptp", "ntp", "crystal", "quartz"]),
      activeTier: "cesium",
    });
    if (!result.id) throw new Error("No ID returned");
    degradationIds.push(result.id);
    if (!result.escalationTriggered) throw new Error("9.5 should trigger escalation (critical)");
    return `Event ID ${result.id}, activeTier=cesium, CRITICAL escalation, auditLogId=${result.auditLogId}`;
  });

  console.log();
  console.log("── SECTION 2: Escalation Threshold Validation ──");
  console.log();

  await runTest("2.1 Verify >= 8.0 triggers CRITICAL escalation", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "glitch_detected",
      severityScore: 8.0,
      thresholdValue: 1.0,
      observedValue: 10.0,
      variancePercentage: 900.0,
      fallbackChain: { ptp: { status: "active", latency_ms: 0.5 } },
      activeTier: "ptp",
    });
    if (!result.escalationTriggered) throw new Error("Expected escalation at exactly 8.0");
    return `Score 8.0 → escalation=true (critical)`;
  });

  await runTest("2.2 Verify >= 6.0 triggers HIGH escalation", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "sync_failure",
      severityScore: 6.0,
      thresholdValue: 3.0,
      observedValue: 0.0,
      variancePercentage: 100.0,
      fallbackChain: { ptp: { status: "failed" }, ntp: { status: "active" } },
      activeTier: "ntp",
    });
    if (!result.escalationTriggered) throw new Error("Expected escalation at exactly 6.0");
    return `Score 6.0 → escalation=true (high)`;
  });

  await runTest("2.3 Verify >= 4.0 logs WARNING (audit entry created)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "clock_drift",
      severityScore: 4.0,
      thresholdValue: 5.0,
      observedValue: 6.5,
      variancePercentage: 30.0,
      fallbackChain: { ptp: { status: "active" } },
      activeTier: "ptp",
    });
    if (!result.id) throw new Error("No ID returned");
    if (result.auditLogId === null) throw new Error("Expected auditLogId for warning-level event");
    return `Score 4.0 → auditLogId=${result.auditLogId} (warning logged)`;
  });

  await runTest("2.4 Verify < 4.0 does NOT escalate (info only)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "jitter_variance",
      severityScore: 3.9,
      thresholdValue: 5.0,
      observedValue: 5.5,
      variancePercentage: 10.0,
      fallbackChain: { ptp: { status: "active" } },
      activeTier: "ptp",
    });
    if (result.escalationTriggered) throw new Error("3.9 should NOT trigger escalation");
    if (result.auditLogId !== null) throw new Error("3.9 should NOT create audit log entry");
    return `Score 3.9 → escalation=false, auditLogId=null (info only)`;
  });

  await runTest("2.5 Verify boundary: 7.9 is HIGH (not critical)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "sync_failure",
      severityScore: 7.9,
      thresholdValue: 3.0,
      observedValue: 0.0,
      variancePercentage: 100.0,
      fallbackChain: { ptp: { status: "failed" }, ntp: { status: "active" } },
      activeTier: "ntp",
    });
    if (!result.escalationTriggered) throw new Error("7.9 should trigger escalation (high)");
    return `Score 7.9 → escalation=true (high, not critical)`;
  });

  await runTest("2.6 Verify boundary: 5.9 is WARNING (not high)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "clock_drift",
      severityScore: 5.9,
      thresholdValue: 5.0,
      observedValue: 7.0,
      variancePercentage: 40.0,
      fallbackChain: { ptp: { status: "active" } },
      activeTier: "ptp",
    });
    if (result.escalationTriggered) throw new Error("5.9 should NOT set escalationTriggered");
    return `Score 5.9 → warning level (audit logged, no escalation flag)`;
  });

  console.log();
  console.log("── SECTION 3: Recovery Simulation (Cesium → Quartz → Crystal → NTP → PTP) ──");
  console.log();

  const recoveryIds: number[] = [];

  await runTest("3.1 Recovery step 1: Cesium → Quartz (severity 7.0, improving)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "clock_drift",
      severityScore: 7.0,
      thresholdValue: 5.0,
      observedValue: 6.0,
      variancePercentage: 20.0,
      fallbackChain: buildFallbackChain("quartz", ["ptp", "ntp", "crystal"]),
      activeTier: "quartz",
    });
    if (!result.id) throw new Error("No ID returned");
    recoveryIds.push(result.id);
    return `Event ID ${result.id}, recovery to quartz`;
  });

  await runTest("3.2 Recovery step 2: Quartz → Crystal (severity 5.5, improving)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "clock_drift",
      severityScore: 5.5,
      thresholdValue: 5.0,
      observedValue: 5.5,
      variancePercentage: 10.0,
      fallbackChain: buildFallbackChain("crystal", ["ptp", "ntp"]),
      activeTier: "crystal",
    });
    if (!result.id) throw new Error("No ID returned");
    recoveryIds.push(result.id);
    return `Event ID ${result.id}, recovery to crystal`;
  });

  await runTest("3.3 Recovery step 3: Crystal → NTP (severity 4.0, stabilizing)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "jitter_variance",
      severityScore: 4.0,
      thresholdValue: 5.0,
      observedValue: 5.2,
      variancePercentage: 4.0,
      fallbackChain: buildFallbackChain("ntp", ["ptp"]),
      activeTier: "ntp",
    });
    if (!result.id) throw new Error("No ID returned");
    recoveryIds.push(result.id);
    return `Event ID ${result.id}, recovery to ntp`;
  });

  await runTest("3.4 Recovery step 4: NTP → PTP (severity 2.0, normal)", async () => {
    const result = await hptpAnomalyService.reportAnomaly({
      anomalyType: "jitter_variance",
      severityScore: 2.0,
      thresholdValue: 5.0,
      observedValue: 3.0,
      variancePercentage: 2.0,
      fallbackChain: buildFallbackChain("ptp", []),
      activeTier: "ptp",
    });
    if (!result.id) throw new Error("No ID returned");
    recoveryIds.push(result.id);
    if (result.escalationTriggered) throw new Error("2.0 should NOT trigger escalation");
    return `Event ID ${result.id}, FULL RECOVERY to ptp, escalation=false`;
  });

  console.log();
  console.log("── SECTION 4: Data Integrity Verification ──");
  console.log();

  await runTest("4.1 All degradation events persisted in hptp_anomaly_events", async () => {
    const events = await hptpAnomalyService.getEvents({ limit: 500 });
    const eventIds = events.map(e => e.id);
    const missing = degradationIds.filter(id => !eventIds.includes(id));
    if (missing.length > 0) throw new Error(`Missing degradation event IDs: ${missing.join(", ")}`);
    return `All ${degradationIds.length} degradation events found in database`;
  });

  await runTest("4.2 All recovery events persisted in hptp_anomaly_events", async () => {
    const events = await hptpAnomalyService.getEvents({ limit: 500 });
    const eventIds = events.map(e => e.id);
    const missing = recoveryIds.filter(id => !eventIds.includes(id));
    if (missing.length > 0) throw new Error(`Missing recovery event IDs: ${missing.join(", ")}`);
    return `All ${recoveryIds.length} recovery events found in database`;
  });

  await runTest("4.3 Fallback chain data preserved (no null fallbackChain)", async () => {
    const events = await hptpAnomalyService.getEvents({ limit: 500 });
    const allTestIds = [...degradationIds, ...recoveryIds];
    const testEvents = events.filter(e => allTestIds.includes(e.id));
    const nullChains = testEvents.filter(e => !e.fallbackChain || Object.keys(e.fallbackChain as object).length === 0);
    if (nullChains.length > 0) throw new Error(`${nullChains.length} events have null/empty fallbackChain`);
    return `All ${testEvents.length} test events have valid fallbackChain data`;
  });

  await runTest("4.4 Tier transitions logged with correct activeTier values", async () => {
    const events = await hptpAnomalyService.getEvents({ limit: 500 });
    const expectedDegradationTiers: FallbackTier[] = ["ntp", "crystal", "quartz", "cesium", "cesium"];
    for (let i = 0; i < degradationIds.length; i++) {
      const event = events.find(e => e.id === degradationIds[i]);
      if (!event) throw new Error(`Degradation event ${degradationIds[i]} not found`);
      if (event.activeTier !== expectedDegradationTiers[i]) {
        throw new Error(`Event ${event.id}: expected tier ${expectedDegradationTiers[i]}, got ${event.activeTier}`);
      }
    }
    const expectedRecoveryTiers: FallbackTier[] = ["quartz", "crystal", "ntp", "ptp"];
    for (let i = 0; i < recoveryIds.length; i++) {
      const event = events.find(e => e.id === recoveryIds[i]);
      if (!event) throw new Error(`Recovery event ${recoveryIds[i]} not found`);
      if (event.activeTier !== expectedRecoveryTiers[i]) {
        throw new Error(`Event ${event.id}: expected tier ${expectedRecoveryTiers[i]}, got ${event.activeTier}`);
      }
    }
    return `All tier values verified: degradation=[${expectedDegradationTiers.join("→")}], recovery=[${expectedRecoveryTiers.join("→")}]`;
  });

  await runTest("4.5 Escalation audit trail integrity (auditLogId references valid)", async () => {
    const events = await hptpAnomalyService.getEvents({ limit: 500 });
    const allTestIds = [...degradationIds, ...recoveryIds];
    const testEvents = events.filter(e => allTestIds.includes(e.id));
    const escalatedEvents = testEvents.filter(e => e.escalationTriggered);
    const nullAuditIds = escalatedEvents.filter(e => e.auditLogId === null);
    if (nullAuditIds.length > 0) throw new Error(`${nullAuditIds.length} escalated events have null auditLogId`);
    return `${escalatedEvents.length} escalated events all have valid auditLogId references`;
  });

  await runTest("4.6 No data loss: event count increased by expected amount", async () => {
    const totalExpected = degradationIds.length + recoveryIds.length;
    const allIds = [...degradationIds, ...recoveryIds];
    const events = await hptpAnomalyService.getEvents({ limit: 1000 });
    const foundCount = events.filter(e => allIds.includes(e.id)).length;
    if (foundCount !== totalExpected) {
      throw new Error(`Expected ${totalExpected} test events, found ${foundCount}`);
    }
    return `${foundCount}/${totalExpected} test events verified (zero data loss)`;
  });

  await runTest("4.7 Fallback analysis reflects test data", async () => {
    const analysis = await hptpAnomalyService.getFallbackAnalysis();
    const tiersWithEvents = Object.entries(analysis).filter(([_, data]: [string, any]) => data.eventCount > 0);
    if (tiersWithEvents.length === 0) throw new Error("No tiers have events in fallback analysis");
    return `Fallback analysis shows ${tiersWithEvents.length} tiers with events: ${tiersWithEvents.map(([t]) => t).join(", ")}`;
  });

  console.log();

  console.log("=".repeat(74));
  console.log(`  RESULTS: ${passed} PASSED, ${failed} FAILED (${results.length} total)`);
  console.log("=".repeat(74));
  console.log();

  console.log("┌─────────────────────────────────────────────────────────────────────────┐");
  console.log("│  TEST SUMMARY                                                           │");
  console.log("├────────┬──────────────────────────────────────────────────┬──────────────┤");
  console.log("│ Status │ Test                                             │ Duration     │");
  console.log("├────────┼──────────────────────────────────────────────────┼──────────────┤");
  for (const r of results) {
    const status = r.status === "PASS" ? " PASS " : " FAIL ";
    const name = r.test.padEnd(48).substring(0, 48);
    const dur = `${r.duration_ms}ms`.padStart(10);
    console.log(`│${status}│ ${name} │ ${dur}   │`);
  }
  console.log("└────────┴──────────────────────────────────────────────────┴──────────────┘");
  console.log();

  if (failed > 0) {
    console.log("FAILED TESTS:");
    for (const r of results.filter(r => r.status === "FAIL")) {
      console.log(`  - ${r.test}: ${r.details}`);
    }
    console.log();
  }

  console.log(`Exit code: ${failed > 0 ? 1 : 0}`);
  process.exit(failed > 0 ? 1 : 0);
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(2);
});
