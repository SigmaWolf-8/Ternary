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

import { TonalField } from './field';
import type { Trit } from '../../shared/topology';

interface DiffusionConfig {
  D: number;
  kT: number;
  dt: number;
  freqCoupling: number;
}

export interface ClockState {
  offset: number;
  frequency: number;
  confidence: number;
}

export interface ClockCorrection {
  offsetAdjust: number;
  frequencyAdjust: number;
  newConfidence: number;
}

type NeighborGraph = Map<string, {
  neighborId: string;
  distance: { eta: number };
  coherence: number;
}[]>;

export class DiffusionSolver {
  private laplacian: Map<string, Map<string, number>> = new Map();
  private D: number;
  private kT: number;
  private dt: number;
  private freqCoupling: number;
  private neighborOffsets: Map<string, number> = new Map();

  constructor(config: DiffusionConfig) {
    this.D = config.D;
    this.kT = config.kT;
    this.dt = config.dt;
    this.freqCoupling = config.freqCoupling;
  }

  buildLaplacian(neighbors: NeighborGraph, field: TonalField): void {
    this.laplacian = new Map();
    for (const [nodeId, edges] of neighbors) {
      const row = new Map<string, number>();
      let degree = 0;
      for (const edge of edges) {
        const w = Math.exp(-field.alpha * edge.distance.eta) * edge.coherence;
        row.set(edge.neighborId, -w);
        degree += w;
      }
      row.set(nodeId, degree);
      this.laplacian.set(nodeId, row);
    }
  }

  updateNeighborOffset(nodeId: string, offset: number): void {
    this.neighborOffsets.set(nodeId, offset);
  }

  step(
    nodeId: string,
    localClock: ClockState,
    field: TonalField,
    ternGrad: { eta: Trit; theta: Trit; psi: Trit }
  ): ClockCorrection {
    const row = this.laplacian.get(nodeId);
    if (!row) {
      return { offsetAdjust: 0, frequencyAdjust: 0, newConfidence: localClock.confidence };
    }

    let laplacianTerm = 0;
    for (const [j, Lij] of row) {
      const neighborOffset = j === nodeId
        ? localClock.offset
        : (this.neighborOffsets.get(j) ?? 0);
      laplacianTerm += Lij * neighborOffset;
    }

    const diffusion = -this.D * laplacianTerm;
    const drift = (this.D * localClock.offset / this.kT) * ternGrad.eta;

    return {
      offsetAdjust: (diffusion + drift) * this.dt,
      frequencyAdjust: ternGrad.psi * this.freqCoupling,
      newConfidence: this.updateConfidence(localClock, field),
    };
  }

  private updateConfidence(localClock: ClockState, field: TonalField): number {
    const gradMag = Math.abs(field.getGradient().eta)
                  + Math.abs(field.getGradient().theta)
                  + Math.abs(field.getGradient().psi);
    const decay = 0.99;
    const boost = gradMag < 0.01 ? 0.01 : 0;
    const penalty = gradMag > 0.5 ? 0.02 : 0;
    return Math.min(1.0, Math.max(0.0,
      localClock.confidence * decay + boost - penalty
    ));
  }

  needsRebuild(currentNeighborCount: Map<string, number>): boolean {
    if (currentNeighborCount.size !== this.laplacian.size) return true;
    for (const [nodeId, count] of currentNeighborCount) {
      const row = this.laplacian.get(nodeId);
      if (!row || row.size - 1 !== count) return true;
    }
    return false;
  }
}
