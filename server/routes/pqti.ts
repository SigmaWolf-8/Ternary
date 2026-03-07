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

import type { Express, Request, Response } from "express";
import { createLogger } from "../logger";

const log = createLogger("pqti-proxy");
const PQTI_BASE = "http://localhost:3001";

async function proxyToPqti(req: Request, res: Response) {
  const path = req.originalUrl;
  const targetUrl = `${PQTI_BASE}${path}`;

  try {
    const headers: Record<string, string> = {};
    if (req.headers["content-type"]) {
      headers["Content-Type"] = req.headers["content-type"] as string;
    }
    if (req.headers["authorization"]) {
      headers["Authorization"] = req.headers["authorization"] as string;
    }
    if (req.headers["x-api-key"]) {
      headers["X-API-Key"] = req.headers["x-api-key"] as string;
    }

    const fetchOptions: RequestInit = {
      method: req.method,
      headers,
    };

    if (req.method !== "GET" && req.method !== "HEAD") {
      fetchOptions.body = JSON.stringify(req.body);
      if (!headers["Content-Type"]) {
        headers["Content-Type"] = "application/json";
      }
    }

    const response = await fetch(targetUrl, fetchOptions);
    const contentType = response.headers.get("content-type") || "";

    if (contentType.includes("application/json")) {
      const data = await response.json();
      res.status(response.status).json(data);
    } else {
      const text = await response.text();
      res.status(response.status).type(contentType).send(text);
    }
  } catch (error: any) {
    log.error(`PQTI proxy error: ${error.message}`);
    res.status(502).json({
      success: false,
      error: "PQTI service unavailable",
      details: "The post-quantum cryptography service is not responding",
    });
  }
}

export function registerPqtiRoutes(app: Express) {
  app.use("/api/pqti", (req: Request, res: Response) => {
    proxyToPqti(req, res);
  });

  app.get("/api/pqti-status", async (_req: Request, res: Response) => {
    try {
      const response = await fetch(`${PQTI_BASE}/health`);
      const data = await response.json();
      res.json({ ...data, proxy: true });
    } catch {
      res.json({ status: "unavailable", proxy: true });
    }
  });
}
