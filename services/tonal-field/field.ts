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

import { EventEmitter } from 'events';
import type { ToroidalAddress } from '../../shared/topology';

export interface FmTimingPacketData {
  frequencyState: {
    f_inst: number;
    sidebands: [number, number, number, number];
    coherence: number;
  };
  modulationIndex: number;
  networkHealth: -1 | 0 | 1;
  entropyNonce: Uint8Array;
}

interface NeighborState {
  potential: number;
  coherence: number;
  distance: ToroidalAddress;
  lastUpdate: number;
}

interface TonalFieldConfig {
  alpha: number;
  couplingStrength: number;
  staleThresholdMs?: number;
}

export class TonalField extends EventEmitter {
  private potential: number = 0;
  private gradient = { eta: 0, theta: 0, psi: 0 };
  private neighbors: Map<string, NeighborState> = new Map();
  public alpha: number;
  private couplingStrength: number;
  private staleThresholdMs: number;
  private t: number = 0;
  private k = { eta: 1.0, theta: 0.0, psi: 0.0 };

  constructor(config: TonalFieldConfig) {
    super();
    this.alpha = config.alpha;
    this.couplingStrength = config.couplingStrength;
    this.staleThresholdMs = config.staleThresholdMs ?? 30_000;
  }

  updateFromPacket(
    nodeId: string,
    packet: FmTimingPacketData,
    distance: ToroidalAddress
  ): void {
    const attenuation = Math.exp(-this.alpha * distance.eta);
    const contribution = packet.frequencyState.coherence * attenuation;
    const dot = this.k.eta * distance.eta
              + this.k.theta * distance.theta
              + this.k.psi * distance.psi;

    this.neighbors.set(nodeId, {
      potential: contribution * Math.cos(
        packet.frequencyState.f_inst * this.t + dot
      ),
      coherence: packet.frequencyState.coherence,
      distance,
      lastUpdate: Date.now(),
    });

    this.pruneStaleNeighbors();
    this.recomputePotential();
    this.recomputeGradient();
    this.emit('update');
  }

  private pruneStaleNeighbors(): void {
    const now = Date.now();
    for (const [id, state] of this.neighbors) {
      if (now - state.lastUpdate > this.staleThresholdMs) {
        this.neighbors.delete(id);
      }
    }
  }

  private recomputePotential(): void {
    let sum = 0;
    for (const state of this.neighbors.values()) {
      sum += state.potential;
    }
    this.potential = this.neighbors.size > 0 ? sum / this.neighbors.size : 0;
  }

  private recomputeGradient(): void {
    let dEta = 0, dTheta = 0, dPsi = 0;
    let nEta = 0, nTheta = 0, nPsi = 0;

    for (const state of this.neighbors.values()) {
      const d = state.distance;
      if (Math.abs(d.eta) > Math.abs(d.theta) && Math.abs(d.eta) > Math.abs(d.psi)) {
        dEta += (state.potential - this.potential) / Math.max(d.eta, 0.001);
        nEta++;
      } else if (Math.abs(d.theta) > Math.abs(d.psi)) {
        dTheta += (state.potential - this.potential) / Math.max(d.theta, 0.001);
        nTheta++;
      } else {
        dPsi += (state.potential - this.potential) / Math.max(d.psi, 0.001);
        nPsi++;
      }
    }

    this.gradient = {
      eta: nEta > 0 ? -(dEta / nEta) : 0,
      theta: nTheta > 0 ? -(dTheta / nTheta) : 0,
      psi: nPsi > 0 ? -(dPsi / nPsi) : 0,
    };
  }

  getSyncCorrection(): number {
    return -this.gradient.eta * this.couplingStrength;
  }

  getConfidence(nodeId: string): number {
    return this.neighbors.get(nodeId)?.coherence ?? 0;
  }

  getPotential(): number { return this.potential; }
  getGradient() { return { ...this.gradient }; }
  getNeighborCount(): number { return this.neighbors.size; }
  getLastUpdate(): number {
    let latest = 0;
    for (const s of this.neighbors.values()) {
      if (s.lastUpdate > latest) latest = s.lastUpdate;
    }
    return latest;
  }
  getNeighborStates() {
    const out: Record<string, NeighborState> = {};
    for (const [id, state] of this.neighbors) { out[id] = state; }
    return out;
  }
  getNetworkState() {
    return {
      potential: this.potential,
      gradient: this.gradient,
      neighborCount: this.neighbors.size,
      averageCoherence: this.neighbors.size > 0
        ? [...this.neighbors.values()].reduce((s, n) => s + n.coherence, 0) / this.neighbors.size
        : 0,
    };
  }

  advanceTime(dt: number): void {
    this.t += dt;
  }
}
