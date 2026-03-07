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

interface RingBuffer<T> {
  data: T[];
  capacity: number;
  head: number;
  count: number;
}

function createRingBuffer<T>(capacity: number): RingBuffer<T> {
  return { data: new Array(capacity), capacity, head: 0, count: 0 };
}

function pushRingBuffer<T>(buf: RingBuffer<T>, value: T): void {
  buf.data[buf.head] = value;
  buf.head = (buf.head + 1) % buf.capacity;
  if (buf.count < buf.capacity) buf.count++;
}

function ringBufferToArray<T>(buf: RingBuffer<T>): T[] {
  if (buf.count < buf.capacity) return buf.data.slice(0, buf.count);
  const start = buf.head;
  return [...buf.data.slice(start), ...buf.data.slice(0, start)];
}

export interface SweepResult {
  optimalRate: number;
  qualityAtOptimal: number;
  sweepRange: [number, number];
  samples: { frequency: number; quality: number }[];
}

export class ResonanceDetector {
  private rttHistory: RingBuffer<number>;
  private currentSyncRate: number;
  private averagePathLength: number;
  private longestPath: number;

  constructor(config?: {
    historySize?: number;
    initialSyncRate?: number;
    pathLength?: number;
    longestPath?: number;
  }) {
    this.rttHistory = createRingBuffer<number>(config?.historySize ?? 256);
    this.currentSyncRate = config?.initialSyncRate ?? 1.0;
    this.averagePathLength = config?.pathLength ?? 5.0;
    this.longestPath = config?.longestPath ?? 10.0;
  }

  recordRtt(rttMs: number): void {
    pushRingBuffer(this.rttHistory, rttMs);
  }

  get medianRtt(): number {
    const sorted = ringBufferToArray(this.rttHistory).sort((a, b) => a - b);
    if (sorted.length === 0) return 1.0;
    const mid = Math.floor(sorted.length / 2);
    return sorted.length % 2 === 0
      ? (sorted[mid - 1] + sorted[mid]) / 2
      : sorted[mid];
  }

  get networkWaveSpeed(): number {
    return this.averagePathLength / Math.max(this.medianRtt / 1000, 0.001);
  }

  get resonantFrequency(): number {
    return this.networkWaveSpeed / (4 * this.longestPath);
  }

  computeOptimalSyncRate(): number {
    const c0 = this.networkWaveSpeed;
    const L = this.averagePathLength;
    const T_roundtrip = 2 * L / c0;
    return 1 / (2 * T_roundtrip);
  }

  sweep(steps: number = 20): SweepResult {
    const fCenter = this.resonantFrequency;
    const fLow = fCenter * 0.8;
    const fHigh = fCenter * 1.2;
    const samples: { frequency: number; quality: number }[] = [];

    let bestFreq = fCenter;
    let bestQuality = 0;

    for (let i = 0; i < steps; i++) {
      const f = fLow + (fHigh - fLow) * (i / (steps - 1));
      const quality = this.estimateQualityAtFrequency(f);
      samples.push({ frequency: f, quality });
      if (quality > bestQuality) {
        bestQuality = quality;
        bestFreq = f;
      }
    }

    return {
      optimalRate: bestFreq,
      qualityAtOptimal: bestQuality,
      sweepRange: [fLow, fHigh],
      samples,
    };
  }

  private estimateQualityAtFrequency(f: number): number {
    const fRes = this.resonantFrequency;
    const Q = 10;
    const ratio = f / fRes;
    return 1 / Math.sqrt((1 - ratio ** 2) ** 2 + (ratio / Q) ** 2);
  }

  getSyncRate(): number {
    return this.currentSyncRate;
  }

  applySweepResult(result: SweepResult): void {
    this.currentSyncRate = result.optimalRate;
  }

  getStatus() {
    return {
      currentSyncRate: this.currentSyncRate,
      resonantFrequency: this.resonantFrequency,
      networkWaveSpeed: this.networkWaveSpeed,
      medianRttMs: this.medianRtt,
      optimalSyncRate: this.computeOptimalSyncRate(),
      rttSamples: this.rttHistory.count,
    };
  }
}
