/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import type { Request, Response, NextFunction } from "express";
import { apiKeyService } from "../services/api-key.service";

declare global {
  namespace Express {
    interface Request {
      apiKeyAuth?: {
        keyId: string;
        owner: string;
        scopes: string[];
        rateLimitRpm: number;
        rateLimitTier: string;
      };
    }
  }
}

export function scopedApiKeyAuth(requiredScopes: string[] = []) {
  return async (req: Request, res: Response, next: NextFunction) => {
    const apiKey =
      (req.headers["x-api-key"] as string) ||
      (req.headers["authorization"] as string)?.replace(/^Bearer\s+/i, "") ||
      (req.query.api_key as string);

    if (!apiKey) {
      return res.status(401).json({
        error: "API key required",
        hint: "Provide via X-API-Key header, Authorization: Bearer <key>, or api_key query parameter.",
      });
    }

    try {
      const result = await apiKeyService.validate(apiKey);
      if (!result?.valid) {
        return res.status(403).json({ error: "Invalid, expired, or revoked API key" });
      }

      if (
        requiredScopes.length > 0 &&
        !requiredScopes.every((s) => result.scopes.includes(s))
      ) {
        const missing = requiredScopes.filter((s) => !result.scopes.includes(s));
        return res.status(403).json({
          error: "Insufficient scopes",
          required: requiredScopes,
          missing,
        });
      }

      req.apiKeyAuth = {
        keyId: result.keyId,
        owner: result.owner,
        scopes: result.scopes,
        rateLimitRpm: result.rateLimitRpm,
        rateLimitTier: result.rateLimitTier,
      };

      const ip = req.ip || req.socket.remoteAddress || null;
      apiKeyService
        .logUsage(result.keyId, req.path, req.method, 200, ip)
        .catch(() => {});

      next();
    } catch (err) {
      return res.status(500).json({ error: "API key validation failed" });
    }
  };
}
