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
interface GeoResult {
  ip: string;
  city?: string;
  region?: string;
  country?: string;
  lat?: number;
  lon?: number;
  org?: string;
}

const geoCache = new Map<string, { result: GeoResult; expires: number }>();
const CACHE_TTL = 5 * 60 * 1000;

const IP_REGEX = /^(\d{1,3}\.){3}\d{1,3}$/;

function isValidPublicIPv4(ip: string): boolean {
  if (!IP_REGEX.test(ip)) return false;
  const parts = ip.split(".").map(Number);
  if (parts.some((p) => p < 0 || p > 255)) return false;
  if (parts[0] === 10) return false;
  if (parts[0] === 172 && parts[1] >= 16 && parts[1] <= 31) return false;
  if (parts[0] === 192 && parts[1] === 168) return false;
  if (parts[0] === 127) return false;
  if (parts[0] === 0) return false;
  return true;
}

export function extractClientIP(req: { headers: Record<string, string | string[] | undefined>; socket: { remoteAddress?: string }; ip?: string }): string {
  if (req.ip && req.ip !== "::1") return req.ip;

  const forwarded = req.headers["x-forwarded-for"];
  if (forwarded) {
    const parts = typeof forwarded === "string" ? forwarded.split(",") : forwarded;
    for (const part of parts) {
      const trimmed = (part || "").trim();
      if (isValidPublicIPv4(trimmed)) return trimmed;
    }
    const first = (parts[0] || "").trim();
    if (first) return first;
  }
  return req.socket.remoteAddress || "unknown";
}

export async function lookupGeo(ip: string): Promise<GeoResult> {
  if (!isValidPublicIPv4(ip)) {
    return { ip, city: "Local", region: "Local", country: "Local" };
  }

  const cached = geoCache.get(ip);
  if (cached && cached.expires > Date.now()) {
    return cached.result;
  }

  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 3000);

    const res = await fetch(`https://ipapi.co/${ip}/json/`, {
      signal: controller.signal,
      headers: { "User-Agent": "SignHere/1.0" },
    });
    clearTimeout(timeout);

    if (!res.ok) {
      return { ip };
    }

    const data = await res.json() as any;
    const result: GeoResult = {
      ip: data.ip || ip,
      city: data.city || undefined,
      region: data.region || undefined,
      country: data.country_name || undefined,
      lat: data.latitude || undefined,
      lon: data.longitude || undefined,
      org: data.org || undefined,
    };

    geoCache.set(ip, { result, expires: Date.now() + CACHE_TTL });

    if (geoCache.size > 500) {
      const now = Date.now();
      const keys = Array.from(geoCache.keys());
      for (const key of keys) {
        const val = geoCache.get(key);
        if (val && val.expires < now) geoCache.delete(key);
      }
    }

    return result;
  } catch {
    return { ip };
  }
}
