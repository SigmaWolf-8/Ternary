import type { Express, Request, Response } from "express";
import { createLogger } from "../logger";

const log = createLogger("pqti-proxy");
const PQTI_BASE = "http://localhost:3001";

async function proxyToPqti(req: Request, res: Response) {
  const path = req.originalUrl;
  const targetUrl = `${PQTI_BASE}${path}`;

  try {
    const fetchOptions: RequestInit = {
      method: req.method,
      headers: { "Content-Type": "application/json" },
    };

    if (req.method !== "GET" && req.method !== "HEAD") {
      fetchOptions.body = JSON.stringify(req.body);
    }

    const response = await fetch(targetUrl, fetchOptions);
    const data = await response.json();
    res.status(response.status).json(data);
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
