/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Task 1.5: Load Test Plan & Baseline Execution
 * Tests 1,500 events/sec sustained for 30 seconds (45,000 total events)
 * Measures p50, p95, p99 latencies and error rates.
 *
 * Run: npx tsx scripts/load-test-security.ts
 */

import { securityAuditService } from "../server/services/security-audit.service";

const TARGET_RPS = 1500;
const DURATION_SECONDS = 30;
const TOTAL_EVENTS = TARGET_RPS * DURATION_SECONDS;

const SEVERITIES = ["info", "warning", "high", "critical"] as const;
const CATEGORIES = ["auth", "crypto", "boot", "network", "hptp", "firmware", "privilege"] as const;
const EVENT_TYPES = [
  "auth_failure", "rate_limit_exceeded", "scope_violation", "key_revocation",
  "anomaly_detected", "threat_mitigated", "config_change", "privilege_escalation",
  "data_access", "encryption_failure", "hptp_fallback", "compliance_violation"
];

function randomPick<T>(arr: readonly T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

interface LatencyBucket {
  latencies: number[];
  errors: number;
  successes: number;
}

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const index = Math.ceil((p / 100) * sorted.length) - 1;
  return sorted[Math.max(0, index)];
}

async function runLoadTest() {
  console.log("=".repeat(70));
  console.log("  LOAD TEST — Security Audit Service");
  console.log("  Target: 1,500 events/sec for 30 seconds (45,000 total)");
  console.log("  Date:", new Date().toISOString());
  console.log("=".repeat(70));
  console.log();

  const allLatencies: number[] = [];
  let totalErrors = 0;
  let totalSuccesses = 0;
  const secondBuckets: Map<number, LatencyBucket> = new Map();

  const BATCH_SIZE = 50;
  const BATCHES_PER_SECOND = Math.ceil(TARGET_RPS / BATCH_SIZE);
  const INTERVAL_MS = 1000 / BATCHES_PER_SECOND;

  console.log(`  Config: ${BATCH_SIZE} events/batch, ${BATCHES_PER_SECOND} batches/sec, ${INTERVAL_MS.toFixed(1)}ms interval`);
  console.log();

  const globalStart = Date.now();

  for (let second = 0; second < DURATION_SECONDS; second++) {
    const bucket: LatencyBucket = { latencies: [], errors: 0, successes: 0 };
    const secondStart = Date.now();

    const batchPromises: Promise<void>[] = [];

    for (let batch = 0; batch < BATCHES_PER_SECOND; batch++) {
      const batchPromise = (async () => {
        const promises = Array.from({ length: BATCH_SIZE }, async () => {
          const start = Date.now();
          try {
            await securityAuditService.logEvent({
              severity: randomPick(SEVERITIES),
              category: randomPick(CATEGORIES),
              eventType: randomPick(EVENT_TYPES),
              actor: `load-test-actor-${Math.floor(Math.random() * 100)}`,
              description: `Load test event at ${Date.now()}`,
              affectedComponent: `component-${Math.floor(Math.random() * 20)}`,
              evidence: { loadTest: true, second, batch },
              ipAddress: `10.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}`,
            });
            const latency = Date.now() - start;
            bucket.latencies.push(latency);
            bucket.successes++;
          } catch (err) {
            bucket.errors++;
          }
        });
        await Promise.all(promises);
      })();

      batchPromises.push(batchPromise);

      if (batch < BATCHES_PER_SECOND - 1) {
        await new Promise(resolve => setTimeout(resolve, INTERVAL_MS));
      }
    }

    await Promise.all(batchPromises);

    const secondDuration = Date.now() - secondStart;
    const rps = bucket.successes / (secondDuration / 1000);

    secondBuckets.set(second, bucket);
    allLatencies.push(...bucket.latencies);
    totalSuccesses += bucket.successes;
    totalErrors += bucket.errors;

    if (second % 5 === 0 || second === DURATION_SECONDS - 1) {
      const sorted = [...bucket.latencies].sort((a, b) => a - b);
      console.log(`  [${String(second + 1).padStart(2)}s] ${bucket.successes} events, ${rps.toFixed(0)} rps, p50=${percentile(sorted, 50)}ms, p99=${percentile(sorted, 99)}ms, errors=${bucket.errors}`);
    }
  }

  const globalDuration = Date.now() - globalStart;

  console.log();
  console.log("=".repeat(70));
  console.log("  LOAD TEST RESULTS");
  console.log("=".repeat(70));

  const sorted = [...allLatencies].sort((a, b) => a - b);
  const p50 = percentile(sorted, 50);
  const p95 = percentile(sorted, 95);
  const p99 = percentile(sorted, 99);
  const pMax = sorted[sorted.length - 1] || 0;
  const pMin = sorted[0] || 0;
  const mean = allLatencies.reduce((a, b) => a + b, 0) / allLatencies.length;
  const overallRps = totalSuccesses / (globalDuration / 1000);
  const errorRate = (totalErrors / (totalSuccesses + totalErrors)) * 100;

  console.log();
  console.log(`  Duration:        ${globalDuration}ms (${(globalDuration / 1000).toFixed(1)}s)`);
  console.log(`  Total Events:    ${totalSuccesses + totalErrors}`);
  console.log(`  Successful:      ${totalSuccesses}`);
  console.log(`  Errors:          ${totalErrors}`);
  console.log(`  Error Rate:      ${errorRate.toFixed(2)}%`);
  console.log(`  Overall RPS:     ${overallRps.toFixed(0)}`);
  console.log();
  console.log("  Latency Distribution:");
  console.log(`    Min:    ${pMin}ms`);
  console.log(`    p50:    ${p50}ms`);
  console.log(`    p95:    ${p95}ms`);
  console.log(`    p99:    ${p99}ms`);
  console.log(`    Max:    ${pMax}ms`);
  console.log(`    Mean:   ${mean.toFixed(1)}ms`);
  console.log();

  // Success criteria
  const p99Pass = p99 < 210;
  const errorPass = errorRate === 0;

  console.log("  Success Criteria:");
  console.log(`    p99 < 210ms:     ${p99Pass ? "PASS" : "FAIL"} (${p99}ms)`);
  console.log(`    0% error rate:   ${errorPass ? "PASS" : "FAIL"} (${errorRate.toFixed(2)}%)`);
  console.log();

  console.log(`  Verdict: ${p99Pass && errorPass ? "LOAD TEST PASSED" : "LOAD TEST NEEDS OPTIMIZATION"}`);
  console.log("=".repeat(70));

  process.exit(p99Pass && errorPass ? 0 : 1);
}

runLoadTest().catch(err => {
  console.error("Load test fatal error:", err);
  process.exit(1);
});
