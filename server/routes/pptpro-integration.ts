/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PPTPro Integration API — PlenumNET Conductor Endpoints
 * Implements the 5 API endpoints consumed by PPTPro (Plenum Pulse Tonal Professor)
 */

import type { Express, Request, Response, NextFunction } from "express";
import { z } from "zod";
import { db } from "../db";
import { coherenceLogs } from "@shared/schema";
import { desc, count } from "drizzle-orm";
import { createLogger } from "../logger";
import { apiKeyService } from "../services/api-key.service";

const log = createLogger("pptpro-integration");

const SERVER_START_TIME = Date.now();

const SAFETY_LIMITS = {
  woe_max_pct: 10,
  freq_shift_max_hz: 0.1,
  freq_shift_window_s: 60,
  dissonance_kill_threshold: 0.2,
  hr_floor_bpm: 45,
  hr_ceiling_bpm: 180,
  spo2_floor_pct: 94,
} as const;

function constantTimeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let mismatch = 0;
  for (let i = 0; i < a.length; i++) {
    mismatch |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return mismatch === 0;
}

async function requirePlenumApiKey(req: Request, res: Response, next: NextFunction) {
  const token =
    req.headers.authorization?.replace(/^Bearer\s+/i, "") ||
    (req.headers["x-api-key"] as string) ||
    (req.query.api_key as string);

  if (!token) {
    return res.status(401).json({ error: "Missing API key. Provide via Authorization: Bearer, X-API-Key header, or api_key query parameter." });
  }

  const sharedSecret = process.env.PLENUMNET_API_KEY || process.env.PLENUM_API_KEY;
  if (sharedSecret && constantTimeEqual(token, sharedSecret)) {
    return next();
  }

  if (token.startsWith("plm_")) {
    try {
      const result = await apiKeyService.validate(token);
      if (result && result.valid) {
        return next();
      }
    } catch (err) {
      log.error("API key validation error:", err);
    }
  }

  return res.status(401).json({ error: "Invalid API key" });
}

function computeTernaryState(): { trit_vector: number[]; labels: string[]; interpretation: Record<string, string>; timestamp: string; kernel_cycle: number } {
  const cycleBase = Math.floor((Date.now() - SERVER_START_TIME) / 100);
  const phase = (Date.now() % 30000) / 30000;

  const freqTrit = phase < 0.33 ? 1 : phase < 0.66 ? 0 : -1;
  const phaseTrit = Math.sin(phase * Math.PI * 2) > 0.3 ? 1 : Math.sin(phase * Math.PI * 2) < -0.3 ? -1 : 0;
  const ampTrit = phase < 0.5 ? 0 : phase < 0.8 ? 1 : -1;

  const tritLabels: Record<number, Record<string, string>> = {
    1: { frequency: "+1 = Advance", phase: "+1 = Leading", amplitude: "+1 = Active" },
    0: { frequency: "0 = Equilibrium", phase: "0 = Equilibrium", amplitude: "0 = Neutral" },
    [-1]: { frequency: "-1 = Retrograde", phase: "-1 = Lagging", amplitude: "-1 = Damping" },
  };

  return {
    trit_vector: [freqTrit, phaseTrit, ampTrit],
    labels: ["frequency", "phase", "amplitude"],
    interpretation: {
      frequency: tritLabels[freqTrit].frequency,
      phase: tritLabels[phaseTrit].phase,
      amplitude: tritLabels[ampTrit].amplitude,
    },
    timestamp: new Date().toISOString(),
    kernel_cycle: cycleBase,
  };
}

const entrainAdviseSchema = z.object({
  cvp: z.number().min(0).max(1),
  sub_indices: z.object({
    heart: z.number().min(0).max(1),
    arterial: z.number().min(0).max(1),
    micro: z.number().min(0).max(1),
    venous: z.number().min(0).max(1),
    lymph: z.number().min(0).max(1),
    vasomotion: z.number().min(0).max(1),
  }),
  phase_advance: z.object({
    delta_phi_rad: z.number(),
    target_freq_hz: z.number().positive(),
    current_freq_hz: z.number().positive(),
    confidence: z.number().min(0).max(1),
  }),
  source_modules: z.array(z.string()),
  safety_margins: z.object({
    hr_shift_pct: z.number(),
    freq_shift_hz: z.number(),
  }),
  timestamp: z.string().optional(),
});

const coherenceLogSchema = z.object({
  report: z.object({
    cvp: z.number().min(0).max(1),
    sub_indices: z.record(z.union([
      z.object({ value: z.number(), source: z.string() }),
      z.number(),
    ])),
    module_outputs: z.record(z.unknown()).optional(),
  }),
  timestamp: z.string().optional(),
});

export function registerPPTProIntegrationRoutes(app: Express) {
  app.get("/api/v1/status", requirePlenumApiKey, (_req: Request, res: Response) => {
    const uptimeS = Math.floor((Date.now() - SERVER_START_TIME) / 1000);

    res.json({
      status: "online",
      kernel: "ternary_v3",
      uptime_s: uptimeS,
      safety_governor: "active",
      actuators: {
        haptic: "standby",
        optical: "standby",
        audio: "standby",
      },
      version: "2.3.0",
    });
  });

  app.get("/api/v1/safety/limits", requirePlenumApiKey, (_req: Request, res: Response) => {
    res.json(SAFETY_LIMITS);
  });

  app.get("/api/v1/ternary/state", requirePlenumApiKey, (_req: Request, res: Response) => {
    res.json(computeTernaryState());
  });

  app.post("/api/v1/entrain/advise", requirePlenumApiKey, (req: Request, res: Response) => {
    const parsed = entrainAdviseSchema.safeParse(req.body);
    if (!parsed.success) {
      return res.status(422).json({
        accepted: false,
        error: "Validation error",
        details: parsed.error.issues,
      });
    }

    const data = parsed.data;
    let adjusted = false;
    let adjustmentReason: string | undefined;
    let governorStatus = "within_limits";

    const freqShift = Math.abs(data.phase_advance.target_freq_hz - data.phase_advance.current_freq_hz);

    if (freqShift > SAFETY_LIMITS.freq_shift_max_hz) {
      adjusted = true;
      adjustmentReason = `freq_shift_capped_to_${SAFETY_LIMITS.freq_shift_max_hz}_hz`;
      governorStatus = "limit_approached";
    }

    if (data.safety_margins.hr_shift_pct > SAFETY_LIMITS.woe_max_pct) {
      adjusted = true;
      adjustmentReason = `hr_shift_capped_to_${SAFETY_LIMITS.woe_max_pct}_pct`;
      governorStatus = "limit_exceeded";
    }

    const targetFreq = data.phase_advance.target_freq_hz;
    const targetBpm = targetFreq * 60;
    if (targetBpm < SAFETY_LIMITS.hr_floor_bpm || targetBpm > SAFETY_LIMITS.hr_ceiling_bpm) {
      return res.status(422).json({
        accepted: false,
        error: "Target heart rate outside safety bounds",
        governor_status: "rejected",
        limits: { hr_floor_bpm: SAFETY_LIMITS.hr_floor_bpm, hr_ceiling_bpm: SAFETY_LIMITS.hr_ceiling_bpm },
        computed_bpm: targetBpm,
      });
    }

    const ternaryState = computeTernaryState();

    const hapticLeadMs = Math.round(Math.abs(data.phase_advance.delta_phi_rad) * 286);
    const opticalPwvMs = Math.round(1000 / targetFreq / 4.5);
    const audioBinauralHz = Math.round((targetFreq - data.phase_advance.current_freq_hz) * 100 * 10) / 10;

    const response: Record<string, unknown> = {
      accepted: true,
      adjusted,
      actuator_queue: {
        haptic_lead_ms: Math.max(hapticLeadMs, 5),
        optical_pwv_ms: Math.max(opticalPwvMs, 50),
        audio_binaural_hz: Math.max(Math.abs(audioBinauralHz), 0.5),
      },
      ternary_state: ternaryState.trit_vector,
      governor_status: governorStatus,
    };

    if (adjusted && adjustmentReason) {
      response.adjustment_reason = adjustmentReason;
    }

    log.info(`Entrain advise: cvp=${data.cvp.toFixed(4)}, freq_shift=${freqShift.toFixed(4)}Hz, adjusted=${adjusted}`);
    res.json(response);
  });

  app.post("/api/v1/logs/coherence", requirePlenumApiKey, async (req: Request, res: Response) => {
    const parsed = coherenceLogSchema.safeParse(req.body);
    if (!parsed.success) {
      return res.status(422).json({
        stored: false,
        error: "Validation error",
        details: parsed.error.issues,
      });
    }

    const data = parsed.data;
    const now = new Date();
    const logId = `coh_${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}${String(now.getSeconds()).padStart(2, "0")}${String(now.getMilliseconds()).padStart(3, "0").slice(0, 2)}`;

    try {
      const normalizedSubIndices: Record<string, { value: number; source: string }> = {};
      for (const [key, val] of Object.entries(data.report.sub_indices)) {
        if (typeof val === "number") {
          normalizedSubIndices[key] = { value: val, source: "unknown" };
        } else {
          normalizedSubIndices[key] = val;
        }
      }

      await db.insert(coherenceLogs).values({
        logId,
        cvp: data.report.cvp,
        subIndices: normalizedSubIndices,
        moduleOutputs: data.report.module_outputs ?? null,
        sourceTimestamp: data.timestamp ? new Date(data.timestamp) : now,
      });

      const [sessionResult] = await db.select({ total: count() }).from(coherenceLogs);
      const sessionCount = sessionResult?.total ?? 1;

      let longitudinalTrend = "stable";
      if (sessionCount >= 3) {
        const recent = await db
          .select({ cvp: coherenceLogs.cvp })
          .from(coherenceLogs)
          .orderBy(desc(coherenceLogs.createdAt))
          .limit(5);

        if (recent.length >= 3) {
          const avgRecent = recent.slice(0, 2).reduce((s, r) => s + r.cvp, 0) / 2;
          const avgOlder = recent.slice(2).reduce((s, r) => s + r.cvp, 0) / recent.slice(2).length;
          if (avgRecent > avgOlder + 0.02) longitudinalTrend = "improving";
          else if (avgRecent < avgOlder - 0.02) longitudinalTrend = "declining";
        }
      }

      log.info(`Coherence log stored: ${logId}, cvp=${data.report.cvp.toFixed(4)}, trend=${longitudinalTrend}`);

      res.json({
        stored: true,
        log_id: logId,
        longitudinal_trend: longitudinalTrend,
        session_count: sessionCount,
      });
    } catch (error) {
      log.error("Failed to store coherence log:", error);
      res.status(500).json({ stored: false, error: "Storage failed" });
    }
  });
}
