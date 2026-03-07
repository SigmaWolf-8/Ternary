/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

/**
 * Dimensionless parameters from Section 8.2 of the unified model,
 * computed as real-time KPIs for PlenumNET.
 */

export interface PlenumMetrics {
  pi1: number;
  pi2: number;
  pi3: number;
  pi4: number;
}

export interface NetworkState {
  tonalFieldEnergy: number;
  networkLoadPressure: number;
  currentSyncRate: number;
  detectedResonance: number;
  metadataThroughput: number;
  syncBandwidth: number;
  D: number;
  q: number;
  gradPhiT: number;
  kT: number;
  viscosity: number;
  flowVelocity: number;
  pathLength: number;
}

export function computePlenumMetrics(state: NetworkState): PlenumMetrics {
  const safeDivide = (a: number, b: number) => b === 0 ? 0 : a / b;
  return {
    pi1: safeDivide(state.tonalFieldEnergy, state.networkLoadPressure),
    pi2: safeDivide(state.currentSyncRate, state.detectedResonance),
    pi3: safeDivide(state.metadataThroughput, state.syncBandwidth),
    pi4: safeDivide(
      state.D * state.q * state.gradPhiT / Math.max(state.kT, 1e-10),
      state.viscosity * state.flowVelocity / Math.max(state.pathLength ** 2, 1e-10)
    ),
  };
}

interface Thresholds {
  pi1: { target: number; warning: [number, number]; critical: [number, number] };
  pi2: { target: number; warning: [number, number]; critical: [number, number] };
  pi3: { target: number; warning: number };
  pi4: { contextDependent: boolean };
}

const THRESHOLDS: Thresholds = {
  pi1: { target: 1e-6, warning: [1e-8, 1e-4], critical: [1e-10, 1e-2] },
  pi2: { target: 1.0, warning: [0.8, 1.2], critical: [0.5, 2.0] },
  pi3: { target: 100, warning: 10 },
  pi4: { contextDependent: true },
};

export function assessHealth(metrics: PlenumMetrics): {
  status: 'healthy' | 'warning' | 'critical';
  issues: string[];
} {
  const issues: string[] = [];

  if (metrics.pi2 < THRESHOLDS.pi2.critical[0] || metrics.pi2 > THRESHOLDS.pi2.critical[1]) {
    issues.push(`Pi2 critical: ${metrics.pi2.toFixed(3)} (target ~1.0)`);
  } else if (metrics.pi2 < THRESHOLDS.pi2.warning[0] || metrics.pi2 > THRESHOLDS.pi2.warning[1]) {
    issues.push(`Pi2 warning: ${metrics.pi2.toFixed(3)} (target ~1.0)`);
  }

  if (metrics.pi1 < THRESHOLDS.pi1.critical[0] || metrics.pi1 > THRESHOLDS.pi1.critical[1]) {
    issues.push(`Pi1 critical: ${metrics.pi1.toExponential(2)} (target ~1e-6)`);
  } else if (metrics.pi1 < THRESHOLDS.pi1.warning[0] || metrics.pi1 > THRESHOLDS.pi1.warning[1]) {
    issues.push(`Pi1 warning: ${metrics.pi1.toExponential(2)} (target ~1e-6)`);
  }

  if (metrics.pi3 < THRESHOLDS.pi3.warning) {
    issues.push(`Pi3 low: ${metrics.pi3.toFixed(1)} (target >>1)`);
  }

  const hasCritical = issues.some(i => i.includes('critical'));
  const status = hasCritical ? 'critical' : issues.length > 0 ? 'warning' : 'healthy';

  return { status, issues };
}
