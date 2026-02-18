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
import type { Request, Response, NextFunction } from "express";

declare global {
  namespace Express {
    interface Request {
      tenantId?: string;
    }
  }
}

export function tenantMiddleware(req: Request, res: Response, next: NextFunction) {
  const tenantId = req.headers["x-tenant-id"] as string | undefined;

  if (tenantId) {
    req.tenantId = tenantId;
  }

  next();
}

export function requireTenant(req: Request, res: Response, next: NextFunction) {
  const tenantId = req.headers["x-tenant-id"] as string | undefined;

  if (!tenantId) {
    return res.status(403).json({ error: "Tenant ID required (x-tenant-id header)" });
  }

  req.tenantId = tenantId;
  next();
}
