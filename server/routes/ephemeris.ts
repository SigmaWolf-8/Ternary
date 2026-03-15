/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Ternary Ephemeris API Routes
 * POST /api/v1/convert — degree/radian conversion with resonance
 * POST /api/v1/ephemeris — planetary position in ternary coordinates
 */

import type { Express, Request, Response, NextFunction } from "express";
import { z } from "zod";
import {
  convertDegrees,
  getEphemeris,
  SUPPORTED_PLANETS,
  FULL_CIRCLE_DEG,
  TERNARY_RADIAN_DEG,
  Z28_COUNT
} from "../ternary-ephemeris";
import { createLogger } from "../logger";
import { scopedApiKeyAuth } from "../middleware/api-key-auth";

const log = createLogger("ephemeris");

function extractApiKey(req: Request): string | undefined {
  return (req.headers["x-api-key"] as string) ||
    (req.headers["authorization"] as string)?.replace(/^Bearer\s+/i, "") ||
    (req.query.api_key as string) || undefined;
}

function optionalScopeAuth(scopes: string[]) {
  return (req: Request, res: Response, next: NextFunction) => {
    if (extractApiKey(req)) {
      return scopedApiKeyAuth(scopes)(req, res, next);
    }
    next();
  };
}

export function registerEphemerisRoutes(app: Express) {
  const convertSchema = z.object({
    type: z.enum(["std_deg", "std_rad", "ternary_deg"]),
    value: z.number(),
    return_resonance: z.boolean().optional().default(false)
  });

  app.post("/api/ephemeris/convert", optionalScopeAuth(["read:ephemeris"]), async (req, res) => {
    try {
      const parsed = convertSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({
          error: "Invalid request",
          details: parsed.error.errors,
          expected: { type: "std_deg | std_rad | ternary_deg", value: "number", return_resonance: "boolean (optional)" }
        });
      }

      const { type, value, return_resonance } = parsed.data;
      const result = convertDegrees(type, value, return_resonance);
      res.json(result);
    } catch (error: unknown) {
      log.error("Convert error:", error);
      res.status(500).json({ error: "Conversion failed" });
    }
  });

  const observerSchema = z.object({
    lat: z.number().min(-90).max(90),
    lon: z.number().min(-180).max(180),
    alt: z.number().optional().default(0)
  }).optional();

  const ephemerisSchema = z.object({
    planet: z.string().min(1),
    jd: z.number().min(2400000).max(2500000),
    include_resonance: z.boolean().optional().default(true),
    observer: observerSchema
  });

  app.post("/api/ephemeris/position", optionalScopeAuth(["read:ephemeris"]), async (req, res) => {
    try {
      const parsed = ephemerisSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({
          error: "Invalid request",
          details: parsed.error.errors,
          supported_planets: SUPPORTED_PLANETS,
          jd_range: [2400000, 2500000]
        });
      }

      const { planet, jd, include_resonance } = parsed.data;

      if (!SUPPORTED_PLANETS.includes(planet.toLowerCase())) {
        return res.status(400).json({
          error: `Unknown planet: ${planet}`,
          supported_planets: SUPPORTED_PLANETS
        });
      }

      const result = getEphemeris(planet, jd);

      const response: Record<string, any> = {
        planet: planet.toLowerCase(),
        jd,
        ecliptic_longitude: +result.ecliptic_longitude.toFixed(6),
        ecliptic_latitude: +result.ecliptic_latitude.toFixed(6),
        distance_au: +result.distance_au.toFixed(8),
        ternary_longitude: +result.ternary_longitude.toFixed(6),
        ternary_latitude: +result.ternary_latitude.toFixed(6),
        ternary_rad: +result.ternary_rad.toFixed(6)
      };

      if (include_resonance) {
        response.resonance = +result.resonance.toFixed(4);
        response.nearest_z28 = result.nearest_z28;
      }

      res.json(response);
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : "Ephemeris calculation failed";
      log.error("Ephemeris error:", error);
      res.status(500).json({ error: msg });
    }
  });

  const batchEphemerisSchema = z.object({
    planets: z.array(z.string()).min(1).max(15),
    jd: z.number().min(2400000).max(2500000),
    include_resonance: z.boolean().optional().default(true),
    observer: observerSchema
  });

  app.post("/api/ephemeris/batch", optionalScopeAuth(["read:ephemeris"]), async (req, res) => {
    try {
      const parsed = batchEphemerisSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({
          error: "Invalid request",
          details: parsed.error.errors
        });
      }

      const { planets, jd, include_resonance } = parsed.data;
      const results: Record<string, any> = {};

      for (const planet of planets) {
        const name = planet.toLowerCase();
        if (!SUPPORTED_PLANETS.includes(name)) {
          results[name] = { error: `Unknown planet: ${name}` };
          continue;
        }
        try {
          const r = getEphemeris(name, jd);
          const entry: Record<string, any> = {
            ecliptic_longitude: +r.ecliptic_longitude.toFixed(6),
            ecliptic_latitude: +r.ecliptic_latitude.toFixed(6),
            distance_au: +r.distance_au.toFixed(8),
            ternary_longitude: +r.ternary_longitude.toFixed(6),
            ternary_latitude: +r.ternary_latitude.toFixed(6),
            ternary_rad: +r.ternary_rad.toFixed(6)
          };
          if (include_resonance) {
            entry.resonance = +r.resonance.toFixed(4);
            entry.nearest_z28 = r.nearest_z28;
          }
          results[name] = entry;
        } catch (e: unknown) {
          results[name] = { error: e instanceof Error ? e.message : "Failed" };
        }
      }

      res.json({ jd, planets: results });
    } catch (error: unknown) {
      log.error("Batch ephemeris error:", error);
      res.status(500).json({ error: "Batch calculation failed" });
    }
  });

  app.get("/api/ephemeris/info", (_req, res) => {
    res.json({
      service: "PlenumNET Ternary Ephemeris",
      version: "1.0.0",
      system: {
        full_circle_deg: FULL_CIRCLE_DEG,
        ternary_radian_deg: TERNARY_RADIAN_DEG,
        z28_count: Z28_COUNT,
        description: "Continuous ternary angular system — no lattice snapping"
      },
      supported_planets: SUPPORTED_PLANETS,
      endpoints: [
        { method: "POST", path: "/api/v1/ephemeris/convert", description: "Convert between standard and ternary angular systems" },
        { method: "POST", path: "/api/v1/ephemeris/position", description: "Get planetary position in ternary coordinates" },
        { method: "POST", path: "/api/v1/ephemeris/batch", description: "Get multiple planetary positions at once" },
        { method: "GET",  path: "/api/v1/ephemeris/info", description: "This endpoint — API metadata" }
      ],
      jd_range: { min: 2400000, max: 2500000 },
      note: "Ternary longitudes are continuous floats (never snapped to lattice). Resonance is scored 0–1 based on proximity to Z₂₈ node.",
    accuracy: "Positions use simplified Keplerian elements (JPL J2000 with linear rates). Geocentric longitudes for all planets except Earth (heliocentric). Observer field is accepted but reserved for future topocentric corrections."
    });
  });
}
