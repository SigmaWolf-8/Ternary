// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
// TDNS Proxy — forwards /api/tdns/* to the TDNS microservice on port 3927
// Follows the same pattern as pqti.ts

import type { Express, Request, Response } from "express";
import { createLogger } from "../logger";

const log = createLogger("tdns-proxy");
const TDNS_BASE = "http://localhost:3927";

async function proxyToTdns(req: Request, res: Response) {
  const subPath = req.path;
  const targetUrl = `${TDNS_BASE}/api/v1${subPath}`;

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

    const fetchOptions: RequestInit = { method: req.method, headers };

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
    log.error(`TDNS proxy error for ${targetUrl}: ${error.message}`);
    res.status(502).json({
      success: false,
      error: "TDNS service unavailable",
      details: "The TDNS microservice is not responding on port 3927",
    });
  }
}

export function registerTdnsRoutes(app: Express) {
  app.use("/api/tdns", (req: Request, res: Response) => {
    proxyToTdns(req, res);
  });

  app.get("/api/tdns-status", async (_req: Request, res: Response) => {
    try {
      const response = await fetch(`${TDNS_BASE}/api/v1/health`);
      const data = await response.json();
      res.json({ ...data, proxy: true, endpoint: TDNS_BASE });
    } catch {
      res.json({ status: "unavailable", proxy: true, endpoint: TDNS_BASE });
    }
  });
}
