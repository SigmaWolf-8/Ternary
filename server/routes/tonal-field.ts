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

import type { Express } from "express";
import { z } from "zod";
import { TonalField } from "../../services/tonal-field/field";
import { DiffusionSolver } from "../../services/tonal-field/diffusion";
import { ResonanceDetector } from "../resonance";
import { computePlenumMetrics, assessHealth, type NetworkState } from "../../services/tonal-field/metrics";

const field = new TonalField({ alpha: 0.1, couplingStrength: 0.01 });
const solver = new DiffusionSolver({ D: 0.1, kT: 1.0, dt: 0.01, freqCoupling: 0.001 });
const resonance = new ResonanceDetector();

const packetSchema = z.object({
  nodeId: z.string().min(1),
  packet: z.object({
    frequencyState: z.object({
      f_inst: z.number(),
      sidebands: z.tuple([z.number(), z.number(), z.number(), z.number()]),
      coherence: z.number().min(0).max(1),
    }),
    modulationIndex: z.number().int().min(0).max(255),
    networkHealth: z.union([z.literal(-1), z.literal(0), z.literal(1)]),
    entropyNonce: z.array(z.number().int().min(0).max(255)).length(8),
  }),
  address: z.object({
    eta: z.number(),
    theta: z.number(),
    psi: z.number(),
  }),
});

export function registerTonalFieldRoutes(app: Express) {
  app.get("/api/tonal/field", (_req, res) => {
    res.json({
      potential: field.getPotential(),
      gradient: field.getGradient(),
      neighborCount: field.getNeighborCount(),
      lastUpdate: field.getLastUpdate(),
    });
  });

  app.get("/api/tonal/neighbors", (_req, res) => {
    res.json(field.getNeighborStates());
  });

  app.post("/api/tonal/packet", (req, res) => {
    const parsed = packetSchema.safeParse(req.body);
    if (!parsed.success) {
      return res.status(400).json({ accepted: false, error: parsed.error.message });
    }
    try {
      const { nodeId, packet, address } = parsed.data;
      const packetData = {
        ...packet,
        entropyNonce: new Uint8Array(packet.entropyNonce),
      };
      field.updateFromPacket(nodeId, packetData, address);
      res.json({ accepted: true, potential: field.getPotential() });
    } catch (err) {
      res.status(400).json({ accepted: false, error: (err as Error).message });
    }
  });

  app.get("/api/resonance/status", (_req, res) => {
    res.json(resonance.getStatus());
  });

  app.post("/api/resonance/sweep", (_req, res) => {
    const result = resonance.sweep();
    resonance.applySweepResult(result);
    res.json(result);
  });

  app.post("/api/resonance/rtt", (req, res) => {
    const schema = z.object({ rttMs: z.number().positive() });
    const parsed = schema.safeParse(req.body);
    if (!parsed.success) {
      return res.status(400).json({ error: parsed.error.message });
    }
    resonance.recordRtt(parsed.data.rttMs);
    res.json({ recorded: true, medianRttMs: resonance.medianRtt });
  });

  app.get("/api/metrics/plenum", (_req, res) => {
    const networkState = field.getNetworkState();
    const fullState: NetworkState = {
      tonalFieldEnergy: Math.abs(networkState.potential),
      networkLoadPressure: Math.max(networkState.neighborCount * 0.1, 0.001),
      currentSyncRate: resonance.getSyncRate(),
      detectedResonance: resonance.getStatus().resonantFrequency,
      metadataThroughput: networkState.neighborCount * 100,
      syncBandwidth: Math.max(resonance.getSyncRate() * 10, 1),
      D: 0.1,
      q: 1.0,
      gradPhiT: Math.abs(networkState.gradient.eta),
      kT: 1.0,
      viscosity: 0.01,
      flowVelocity: 1.0,
      pathLength: 5.0,
    };
    const metrics = computePlenumMetrics(fullState);
    const health = assessHealth(metrics);
    res.json({ metrics, health });
  });
}
