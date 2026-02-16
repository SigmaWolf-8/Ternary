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
