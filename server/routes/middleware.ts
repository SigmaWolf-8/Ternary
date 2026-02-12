/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import type { IStorage } from "../storage";

export function createRequireAdmin(storage: IStorage) {
  return async (req: any, res: any, next: any) => {
    if (!req.isAuthenticated?.() || !req.user?.claims?.sub) {
      return res.status(401).json({ error: "Authentication required" });
    }
    const user = await storage.getUser(req.user.claims.sub);
    if (!user?.isAdmin) {
      return res.status(403).json({ error: "Admin access required" });
    }
    req.adminUser = user;
    next();
  };
}

export const resolveGitHubToken = (adminUser: any): string | null => {
  if (adminUser?.githubToken) {
    const { decryptToken } = require('../crypto-utils');
    return decryptToken(adminUser.githubToken);
  }
  if (process.env.GITHUB_TOKEN) return process.env.GITHUB_TOKEN;
  return null;
};

export const sanitizePath = (inputPath: string): string => {
  let decoded = inputPath;
  decoded = decoded.replace(/\0/g, "");
  decoded = decoded.replace(/\\/g, "/");
  try {
    decoded = decodeURIComponent(decoded);
    decoded = decodeURIComponent(decoded);
  } catch (_e) {
  }
  decoded = decoded.replace(/\0/g, "");
  decoded = decoded.replace(/\\/g, "/");
  const nodePath = require('path');
  let normalized = nodePath.posix.normalize(decoded);
  normalized = normalized
    .replace(/\.\./g, "")
    .replace(/^\/+/, "")
    .replace(/\/+$/, "");
  if (normalized.includes("..") || normalized.includes("\0")) {
    return "";
  }
  return normalized;
};
