/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import rateLimit from "express-rate-limit";
import type { Request, Response } from "express";

export const globalLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 100,
  standardHeaders: true,
  legacyHeaders: false,
  message: { error: "Too many requests, please try again later." },
});

export const authLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 20,
  standardHeaders: true,
  legacyHeaders: false,
  message: { error: "Too many authentication attempts, please try again later." },
});

export const githubTokenLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 10,
  standardHeaders: true,
  legacyHeaders: false,
  message: { error: "Too many token operations, please try again later." },
});

export const computationLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 50,
  standardHeaders: true,
  legacyHeaders: false,
  message: { error: "Too many computation requests, please try again later." },
});

const apiKeyStores = new Map<string, Map<string, { count: number; resetAt: number }>>();

export const perKeyRateLimiter = (req: Request, res: Response, next: Function) => {
  if (!req.apiKeyAuth) return next();

  const { keyId, rateLimitRpm } = req.apiKeyAuth;
  const now = Date.now();
  const windowMs = 60 * 1000;

  if (!apiKeyStores.has(keyId)) {
    apiKeyStores.set(keyId, new Map());
  }

  const store = apiKeyStores.get(keyId)!;
  const windowKey = "rpm";
  const entry = store.get(windowKey);

  if (!entry || now > entry.resetAt) {
    const resetAt = now + windowMs;
    store.set(windowKey, { count: 1, resetAt });
    res.set("X-RateLimit-Limit", String(rateLimitRpm));
    res.set("X-RateLimit-Remaining", String(rateLimitRpm - 1));
    res.set("X-RateLimit-Reset", String(Math.ceil(resetAt / 1000)));
    return next();
  }

  if (entry.count >= rateLimitRpm) {
    const retryAfter = Math.ceil((entry.resetAt - now) / 1000);
    res.set("Retry-After", String(retryAfter));
    res.set("X-RateLimit-Limit", String(rateLimitRpm));
    res.set("X-RateLimit-Remaining", "0");
    res.set("X-RateLimit-Reset", String(Math.ceil(entry.resetAt / 1000)));
    return res.status(429).json({
      error: "Per-key rate limit exceeded",
      limit: rateLimitRpm,
      tier: req.apiKeyAuth.rateLimitTier,
      retryAfter,
    });
  }

  entry.count++;
  res.set("X-RateLimit-Limit", String(rateLimitRpm));
  res.set("X-RateLimit-Remaining", String(rateLimitRpm - entry.count));
  res.set("X-RateLimit-Reset", String(Math.ceil(entry.resetAt / 1000)));
  next();
};
