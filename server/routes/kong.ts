/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import type { Express } from "express";
import type { IStorage } from "../storage";
import { createRequireAdmin, resolveGitHubToken } from "./middleware";
import { createLogger, toErrorMessage } from "../logger";

const log = createLogger("kong");

export function registerKongRoutes(app: Express, storage: IStorage): void {
  const requireAdmin = createRequireAdmin(storage);
  const KONG_API_BASE = "https://us.api.konghq.com/v2";
  const KONG_KONNECT_TOKEN = process.env.KONG_KONNECT_TOKEN;

  // Check Kong Konnect connection status with full gateway readiness
  app.get("/api/kong/status", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.json({ 
          connected: false, 
          error: "Kong Konnect token not configured" 
        });
      }

      const kongHeaders = {
        "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
        "Content-Type": "application/json"
      };

      const response = await fetch(`${KONG_API_BASE}/users/me`, { headers: kongHeaders });

      if (!response.ok) {
        return res.json({ 
          connected: false, 
          error: `API error: ${response.status}` 
        });
      }

      const user = await response.json();

      let controlPlanes: any[] = [];
      let gatewayReady = false;
      let activeProxyUrls: string[] = [];
      let dataPlaneGroups: any[] = [];

      // Fetch cloud gateway configurations (global API) for data plane info
      const KONG_GLOBAL_API = "https://global.api.konghq.com/v2";
      try {
        const cgResp = await fetch(`${KONG_GLOBAL_API}/cloud-gateways/configurations`, { headers: kongHeaders });
        if (cgResp.ok) {
          const cgData = await cgResp.json();
          for (const config of (cgData.data || [])) {
            for (const dpGroup of (config.dataplane_groups || [])) {
              const hostnames = dpGroup.hostnames || [];
              const isReady = dpGroup.state === "ready";
              if (isReady && hostnames.length > 0) {
                gatewayReady = true;
                activeProxyUrls.push(...hostnames.map((h: string) => `https://${h}`));
              }
              dataPlaneGroups.push({
                id: dpGroup.id,
                region: dpGroup.region,
                state: dpGroup.state,
                hostnames,
                controlPlaneId: config.control_plane_id,
                kind: config.kind
              });
            }
          }
        }
      } catch {}

      // Fetch control planes with service/route counts
      try {
        const cpResp = await fetch(`${KONG_API_BASE}/control-planes`, { headers: kongHeaders });
        if (cpResp.ok) {
          const cpData = await cpResp.json();
          for (const cp of (cpData.data || [])) {
            let serviceCount = 0;
            let routeCount = 0;
            try {
              const [svcResp, rtResp] = await Promise.all([
                fetch(`${KONG_API_BASE}/control-planes/${cp.id}/core-entities/services`, { headers: kongHeaders }),
                fetch(`${KONG_API_BASE}/control-planes/${cp.id}/core-entities/routes`, { headers: kongHeaders })
              ]);
              if (svcResp.ok) { const s = await svcResp.json(); serviceCount = s.data?.length || 0; }
              if (rtResp.ok) { const r = await rtResp.json(); routeCount = r.data?.length || 0; }
            } catch {}

            const cpDataPlanes = dataPlaneGroups.filter(dp => dp.controlPlaneId === cp.id);
            const cpHostnames = cpDataPlanes.flatMap((dp: any) => dp.hostnames.map((h: string) => `https://${h}`));
            const cpGatewayReady = cpDataPlanes.some((dp: any) => dp.state === "ready");

            controlPlanes.push({
              id: cp.id,
              name: cp.name,
              description: cp.description,
              clusterType: cp.config?.cluster_type,
              controlPlaneEndpoint: cp.config?.control_plane_endpoint,
              proxyUrls: cpHostnames,
              cloudGateway: cp.config?.cloud_gateway || false,
              dataPlaneState: cpGatewayReady ? "ready" : (cpDataPlanes.length > 0 ? cpDataPlanes[0].state : "none"),
              services: serviceCount,
              routes: routeCount,
              configSynced: serviceCount >= 17
            });
          }
        }
      } catch {}

      res.json({ 
        connected: true, 
        gatewayReady,
        configSynced: controlPlanes.some(cp => cp.configSynced),
        activeProxyUrls,
        dataPlaneGroups,
        user: {
          id: user.id,
          email: user.email,
          fullName: user.full_name,
          preferredName: user.preferred_name,
          active: user.active
        },
        controlPlanes
      });
    } catch (error: unknown) {
      res.json({ 
        connected: false, 
        error: toErrorMessage(error) 
      });
    }
  });

  // Get Kong organization info
  app.get("/api/kong/organization", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const response = await fetch(`${KONG_API_BASE}/organizations/me`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const org = await response.json();
      res.json(org);
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // List Kong control planes
  app.get("/api/kong/control-planes", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Get services for a control plane
  app.get("/api/kong/control-planes/:cpId/services", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Get routes for a control plane
  app.get("/api/kong/control-planes/:cpId/routes", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/routes`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Get plugins for a control plane
  app.get("/api/kong/control-planes/:cpId/plugins", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/plugins`, {
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        }
      });

      if (!response.ok) {
        return res.status(response.status).json({ error: `API error: ${response.status}` });
      }

      const data = await response.json();
      res.json(data);
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Get Kong configuration file (Admin only - may contain API keys)
  app.get("/api/kong/config", requireAdmin, async (req: any, res) => {
    try {
      const fs = await import('fs/promises');
      const path = await import('path');
      const configPath = path.join(process.cwd(), 'kong', 'kong.yaml');
      const config = await fs.readFile(configPath, 'utf-8');
      res.json({ success: true, config });
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Create a service in Kong Konnect (Admin only)
  app.post("/api/kong/control-planes/:cpId/services", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const { name, url, enabled = true, tags = [] } = req.body;

      if (!name || !url) {
        return res.status(400).json({ error: "Name and URL are required" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ name, url, enabled, tags })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return res.status(response.status).json({ 
          error: `API error: ${response.status}`,
          details: errorData 
        });
      }

      const data = await response.json();
      res.json({ success: true, service: data });
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Create a route for a service in Kong Konnect (Admin only)
  app.post("/api/kong/control-planes/:cpId/services/:serviceId/routes", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId, serviceId } = req.params;
      const { name, paths, methods = ['GET', 'POST'], strip_path = true, tags = [] } = req.body;

      if (!name || !paths || !paths.length) {
        return res.status(400).json({ error: "Name and paths are required" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/routes`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ name, paths, methods, strip_path, tags })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return res.status(response.status).json({ 
          error: `API error: ${response.status}`,
          details: errorData 
        });
      }

      const data = await response.json();
      res.json({ success: true, route: data });
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Add a plugin to a service (Admin only)
  app.post("/api/kong/control-planes/:cpId/services/:serviceId/plugins", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId, serviceId } = req.params;
      const { name, config = {}, tags = [] } = req.body;

      if (!name) {
        return res.status(400).json({ error: "Plugin name is required" });
      }

      const response = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/plugins`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ name, config, tags })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return res.status(response.status).json({ 
          error: `API error: ${response.status}`,
          details: errorData 
        });
      }

      const data = await response.json();
      res.json({ success: true, plugin: data });
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  function getPlenumnetServices() {
    const replitDomains = process.env.REPLIT_DOMAINS || process.env.REPLIT_DEV_DOMAIN;
    const baseUrl = replitDomains 
      ? `https://${replitDomains.split(',')[0]}`
      : 'https://plenumnet.replit.app';
    return { baseUrl, services: [
        {
          name: "plenumnet-timing",
          url: `${baseUrl}/api/salvi/timing`,
          tags: ["plenumnet", "timing", "hptp", "finra-cat", "mifid-ii"],
          routePath: "/api/salvi/timing",
          stripPath: false,
          rateLimit: { minute: 100, hour: 1000 },
          methods: ["GET", "POST"],
          endpointCount: 5,
          endpoints: ["GET /timestamp", "GET /metrics", "GET /batch/:count", "GET /self-test", "GET /error-budget"]
        },
        {
          name: "plenumnet-calendars",
          url: `${baseUrl}/api/salvi/timing/epoch`,
          tags: ["plenumnet", "calendars", "epoch", "synchronization"],
          routePath: "/api/salvi/timing/epoch",
          stripPath: false,
          rateLimit: { minute: 120, hour: 1200 },
          methods: ["GET"],
          endpointCount: 26,
          endpoints: ["GET /anchors", "GET /calendars", "GET /calendars/mayan", "GET /calendars/hebrew", "GET /calendars/chinese", "GET /calendars/vedic", "GET /calendars/egyptian", "GET /calendars/julian-day", "GET /calendars/islamic", "GET /calendars/byzantine", "GET /calendars/thirteen-moon", "GET /calendars/persian", "GET /calendars/ethiopian", "GET /calendars/coptic", "GET /calendars/japanese", "GET /calendars/korean", "GET /calendars/thai", "GET /calendars/indian-saka", "GET /calendars/tibetan", "GET /calendars/aztec", "GET /calendars/roman", "GET /calendars/bengali", "GET /calendars/berber", "GET /calendars/balinese", "GET /calendars/zoroastrian", "GET /calendars/aboriginal"]
        },
        {
          name: "plenumnet-ternary",
          url: `${baseUrl}/api/salvi/ternary`,
          tags: ["plenumnet", "ternary", "quantum-safe", "gf3"],
          routePath: "/api/salvi/ternary",
          stripPath: false,
          rateLimit: { minute: 200, hour: 2000 },
          methods: ["GET", "POST"],
          endpointCount: 8,
          endpoints: ["POST /convert", "POST /add", "POST /multiply", "POST /rotate", "POST /not", "POST /xor", "POST /batch", "GET /density/:tritCount"]
        },
        {
          name: "plenumnet-phase",
          url: `${baseUrl}/api/salvi/phase`,
          tags: ["plenumnet", "encryption", "phase", "quantum-safe"],
          routePath: "/api/salvi/phase",
          stripPath: false,
          rateLimit: { minute: 100, hour: 1000 },
          methods: ["GET", "POST"],
          endpointCount: 4,
          endpoints: ["GET /config/:mode", "POST /split", "POST /recombine", "GET /recommend"]
        },
        {
          name: "plenumnet-vm",
          url: `${baseUrl}/api/salvi/vm`,
          tags: ["plenumnet", "vm", "virtual-machine", "isa"],
          routePath: "/api/salvi/vm",
          stripPath: false,
          rateLimit: { minute: 100, hour: 1000 },
          methods: ["GET"],
          endpointCount: 2,
          endpoints: ["GET /spec", "GET /conformance"]
        },
        {
          name: "plenumnet-docs",
          url: `${baseUrl}/api/salvi/docs`,
          tags: ["plenumnet", "docs", "documentation", "openapi"],
          routePath: "/api/salvi/docs",
          stripPath: false,
          rateLimit: { minute: 200, hour: 2000 },
          methods: ["GET"],
          endpointCount: 1,
          endpoints: ["GET /"]
        },
        {
          name: "plenumnet-demo",
          url: `${baseUrl}/api/demo`,
          tags: ["plenumnet", "demo", "compression", "plenumdb"],
          routePath: "/api/demo",
          stripPath: false,
          rateLimit: { minute: 50, hour: 500 },
          methods: ["GET", "POST"],
          endpointCount: 7,
          endpoints: ["POST /run", "GET /stats", "GET /session/:id", "POST /upload", "GET /history", "GET /files", "GET /data/:id"]
        },
        {
          name: "plenumnet-compression",
          url: `${baseUrl}/api/compression`,
          tags: ["plenumnet", "compression", "storage", "plenumdb"],
          routePath: "/api/compression",
          stripPath: false,
          rateLimit: { minute: 50, hour: 500 },
          methods: ["GET", "POST", "DELETE"],
          endpointCount: 6,
          endpoints: ["POST /file", "POST /decompress", "POST /db/store", "GET /db/retrieve/:id", "GET /db/documents", "DELETE /db/documents/:id"]
        },
        {
          name: "plenumnet-whitepapers",
          url: `${baseUrl}/api/whitepapers`,
          tags: ["plenumnet", "whitepapers", "documentation"],
          routePath: "/api/whitepapers",
          stripPath: false,
          rateLimit: { minute: 100, hour: 1000 },
          methods: ["GET", "POST"],
          endpointCount: 4,
          endpoints: ["GET /", "GET /active", "GET /:id", "POST /"]
        },
        {
          name: "plenumnet-legal",
          url: `${baseUrl}/api/legal`,
          tags: ["plenumnet", "legal", "terms", "privacy"],
          routePath: "/api/legal",
          stripPath: false,
          rateLimit: { minute: 100, hour: 1000 },
          methods: ["GET"],
          endpointCount: 4,
          endpoints: ["GET /terms", "GET /privacy", "GET /security", "GET /aup"]
        },
        {
          name: "plenumnet-auth",
          url: `${baseUrl}/api/auth`,
          tags: ["plenumnet", "auth", "identity", "oauth"],
          routePath: "/api/auth",
          stripPath: false,
          rateLimit: { minute: 30, hour: 300 },
          methods: ["GET", "POST"],
          endpointCount: 3,
          endpoints: ["GET /login", "GET /callback", "POST /logout"]
        },
        {
          name: "plenumnet-user",
          url: `${baseUrl}/api/user`,
          tags: ["plenumnet", "user", "profile"],
          routePath: "/api/user",
          stripPath: false,
          rateLimit: { minute: 100, hour: 1000 },
          methods: ["GET"],
          endpointCount: 1,
          endpoints: ["GET /admin-status"]
        },
        {
          name: "plenumnet-developer-signup",
          url: `${baseUrl}/api/developer-signup`,
          tags: ["plenumnet", "developer", "signup", "waitlist"],
          routePath: "/api/developer-signup",
          stripPath: false,
          rateLimit: { minute: 20, hour: 200 },
          methods: ["GET", "POST"],
          endpointCount: 2,
          endpoints: ["POST /", "GET /count"]
        },
        {
          name: "plenumnet-admin",
          url: `${baseUrl}/api/admin`,
          tags: ["plenumnet", "admin", "management"],
          routePath: "/api/admin",
          stripPath: false,
          rateLimit: { minute: 60, hour: 600 },
          methods: ["GET", "POST", "DELETE"],
          endpointCount: 2,
          endpoints: ["GET /developer-signups", "DELETE /developer-signups/:id"]
        },
        {
          name: "plenumnet-github",
          url: `${baseUrl}/api/github`,
          tags: ["plenumnet", "github", "admin", "integration", "cicd"],
          routePath: "/api/github",
          stripPath: false,
          rateLimit: { minute: 60, hour: 600 },
          methods: ["GET", "POST", "PUT", "DELETE"],
          endpointCount: 9,
          endpoints: ["POST /token", "GET /status", "GET /repos/:owner/:repo/branches", "GET /repos/:owner/:repo/contents", "GET /file/:owner/:repo", "PUT /file/:owner/:repo", "DELETE /file/:owner/:repo", "POST /push-workflows/:owner/:repo", "POST /push-batch/:owner/:repo"]
        },
        {
          name: "plenumnet-kong",
          url: `${baseUrl}/api/kong`,
          tags: ["plenumnet", "kong", "gateway", "admin"],
          routePath: "/api/kong",
          stripPath: false,
          rateLimit: { minute: 60, hour: 600 },
          methods: ["GET", "POST"],
          endpointCount: 12,
          endpoints: ["GET /status", "GET /organization", "GET /control-planes", "GET /control-planes/:cpId/services", "GET /control-planes/:cpId/routes", "GET /control-planes/:cpId/plugins", "GET /config", "POST /control-planes/:cpId/services", "POST /control-planes/:cpId/sync-plenumnet", "POST /save-to-github", "GET /control-planes/:cpId/deploy-instructions", "POST /control-planes/:cpId/deploy-to-cloud"]
        },
        {
          name: "plenumnet-health",
          url: `${baseUrl}/api/health`,
          tags: ["plenumnet", "health", "monitoring", "observability"],
          routePath: "/api/health",
          stripPath: false,
          rateLimit: { minute: 300, hour: 3000 },
          methods: ["GET"],
          endpointCount: 1,
          endpoints: ["GET /"]
        }
      ]};
  }

  async function syncControlPlane(cpId: string) {
    if (!KONG_KONNECT_TOKEN) {
      throw new Error("Kong Konnect token not configured");
    }

    const { services } = getPlenumnetServices();
    const results: any[] = [];

    const existingServicesResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
      headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
    });
    const existingServicesData = existingServicesResponse.ok ? await existingServicesResponse.json() : { data: [] };
    const existingServices = existingServicesData.data || [];

    for (const service of services) {
      try {
        let serviceId: string | null = null;
        const existingService = existingServices.find((s: any) => s.name === service.name);
        
        if (existingService) {
          serviceId = existingService.id;
          results.push({ service: service.name, status: 'already_exists', id: serviceId });
        } else {
          const createResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services`, {
            method: 'POST',
            headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`, "Content-Type": "application/json" },
            body: JSON.stringify({ name: service.name, url: service.url, enabled: true, tags: service.tags })
          });

          if (createResponse.ok) {
            const createdService = await createResponse.json();
            serviceId = createdService.id;
            results.push({ service: service.name, status: 'created', id: serviceId });
          } else {
            const errorText = await createResponse.text();
            results.push({ service: service.name, status: 'error', error: `HTTP ${createResponse.status}: ${errorText}` });
            continue;
          }
        }

        if (!serviceId) continue;

        const routePaths = (service as any).routePaths || [service.routePath];
        const stripPath = (service as any).stripPath !== undefined ? (service as any).stripPath : true;
        const routeResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/routes`, {
          method: 'POST',
          headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`, "Content-Type": "application/json" },
          body: JSON.stringify({
            name: `${service.name}-route`,
            paths: routePaths,
            methods: service.methods,
            strip_path: stripPath,
            tags: service.tags
          })
        });

        if (routeResponse.ok) {
          const route = await routeResponse.json();
          results.push({ route: `${service.name}-route`, status: 'route_created', id: route.id });
        } else if (routeResponse.status === 409) {
          results.push({ route: `${service.name}-route`, status: 'route_exists' });
        }

        const pluginResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/core-entities/services/${serviceId}/plugins`, {
          method: 'POST',
          headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`, "Content-Type": "application/json" },
          body: JSON.stringify({
            name: "rate-limiting",
            config: { minute: service.rateLimit.minute, hour: service.rateLimit.hour, policy: "local", fault_tolerant: true, hide_client_headers: false },
            tags: ["plenumnet", "rate-limit"]
          })
        });

        if (pluginResponse.ok) {
          const plugin = await pluginResponse.json();
          results.push({ plugin: `rate-limiting (${service.rateLimit.minute}/min)`, service: service.name, status: 'plugin_created', id: plugin.id });
        } else if (pluginResponse.status === 409) {
          results.push({ plugin: 'rate-limiting', service: service.name, status: 'plugin_exists' });
        }

      } catch (err: unknown) {
        results.push({ service: service.name, status: 'error', error: toErrorMessage(err) });
      }
    }

    const totalEndpoints = services.reduce((sum, s) => sum + (s.endpointCount || 0), 0);
    return { 
      success: true, 
      services: results.filter(r => r.status === 'created' || r.status === 'already_exists').length,
      routes: results.filter(r => r.status === 'route_created' || r.status === 'route_exists').length,
      plugins: results.filter(r => r.status === 'plugin_created' || r.status === 'plugin_exists').length,
      errors: results.filter(r => r.status === 'error').length,
      totalEndpoints,
      totalServices: services.length,
      results 
    };
  }

  app.post("/api/kong/control-planes/:cpId/sync-plenumnet", requireAdmin, async (req: any, res) => {
    try {
      const result = await syncControlPlane(req.params.cpId);
      res.json(result);
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  app.post("/api/kong/sync-all-control-planes", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const kongHeaders = {
        "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
        "Content-Type": "application/json"
      };

      const cpResp = await fetch(`${KONG_API_BASE}/control-planes`, { headers: kongHeaders });
      if (!cpResp.ok) {
        return res.status(cpResp.status).json({ error: `Failed to fetch control planes: ${cpResp.status}` });
      }
      const cpData = await cpResp.json();
      const controlPlanes = cpData.data || [];

      if (controlPlanes.length === 0) {
        return res.json({ success: false, error: "No control planes found" });
      }

      const allResults: any[] = [];

      for (const cp of controlPlanes) {
        try {
          const syncResult = await syncControlPlane(cp.id);
          allResults.push({
            controlPlane: cp.name,
            controlPlaneId: cp.id,
            ...syncResult
          });
        } catch (err: unknown) {
          allResults.push({
            controlPlane: cp.name,
            controlPlaneId: cp.id,
            success: false,
            error: toErrorMessage(err)
          });
        }
      }

      res.json({
        success: true,
        controlPlanesProcessed: allResults.length,
        results: allResults
      });
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  app.get("/api/kong/service-catalog", async (_req, res) => {
    const replitDomains = process.env.REPLIT_DOMAINS || process.env.REPLIT_DEV_DOMAIN;
    const baseUrl = replitDomains 
      ? `https://${replitDomains.split(',')[0]}`
      : 'https://plenumnet.replit.app';

    const catalog = [
      { name: "plenumnet-timing", label: "HPTP Timing API", routePath: "/api/salvi/timing", endpointCount: 5, category: "core", endpoints: ["GET /timestamp", "GET /metrics", "GET /batch/:count", "GET /self-test", "GET /error-budget"] },
      { name: "plenumnet-calendars", label: "Calendar Synchronization", routePath: "/api/salvi/timing/epoch", endpointCount: 26, category: "core", endpoints: ["GET /anchors", "GET /calendars", "GET /calendars/mayan", "GET /calendars/hebrew", "GET /calendars/chinese", "GET /calendars/vedic", "GET /calendars/egyptian", "GET /calendars/julian-day", "GET /calendars/islamic", "GET /calendars/byzantine", "GET /calendars/thirteen-moon", "GET /calendars/persian", "GET /calendars/ethiopian", "GET /calendars/coptic", "GET /calendars/japanese", "GET /calendars/korean", "GET /calendars/thai", "GET /calendars/indian-saka", "GET /calendars/tibetan", "GET /calendars/aztec", "GET /calendars/roman", "GET /calendars/bengali", "GET /calendars/berber", "GET /calendars/balinese", "GET /calendars/zoroastrian", "GET /calendars/aboriginal"] },
      { name: "plenumnet-ternary", label: "Ternary Computing Engine", routePath: "/api/salvi/ternary", endpointCount: 8, category: "core", endpoints: ["POST /convert", "POST /add", "POST /multiply", "POST /rotate", "POST /not", "POST /xor", "POST /batch", "GET /density/:tritCount"] },
      { name: "plenumnet-phase", label: "Phase Encryption", routePath: "/api/salvi/phase", endpointCount: 4, category: "core", endpoints: ["GET /config/:mode", "POST /split", "POST /recombine", "GET /recommend"] },
      { name: "plenumnet-vm", label: "Ternary Virtual Machine", routePath: "/api/salvi/vm", endpointCount: 2, category: "core", endpoints: ["GET /spec", "GET /conformance"] },
      { name: "plenumnet-docs", label: "API Documentation", routePath: "/api/salvi/docs", endpointCount: 1, category: "reference", endpoints: ["GET /"] },
      { name: "plenumnet-demo", label: "Compression Demo", routePath: "/api/demo", endpointCount: 7, category: "tools", endpoints: ["POST /run", "GET /stats", "GET /session/:id", "POST /upload", "GET /history", "GET /files", "GET /data/:id"] },
      { name: "plenumnet-compression", label: "Compression Storage", routePath: "/api/compression", endpointCount: 6, category: "tools", endpoints: ["POST /file", "POST /decompress", "POST /db/store", "GET /db/retrieve/:id", "GET /db/documents", "DELETE /db/documents/:id"] },
      { name: "plenumnet-whitepapers", label: "Whitepaper Management", routePath: "/api/whitepapers", endpointCount: 4, category: "reference", endpoints: ["GET /", "GET /active", "GET /:id", "POST /"] },
      { name: "plenumnet-legal", label: "Legal Documents", routePath: "/api/legal", endpointCount: 4, category: "reference", endpoints: ["GET /terms", "GET /privacy", "GET /security", "GET /aup"] },
      { name: "plenumnet-auth", label: "Authentication", routePath: "/api/auth", endpointCount: 3, category: "platform", endpoints: ["GET /login", "GET /callback", "POST /logout"] },
      { name: "plenumnet-user", label: "User Management", routePath: "/api/user", endpointCount: 1, category: "platform", endpoints: ["GET /admin-status"] },
      { name: "plenumnet-developer-signup", label: "Developer Waitlist", routePath: "/api/developer-signup", endpointCount: 2, category: "platform", endpoints: ["POST /", "GET /count"] },
      { name: "plenumnet-admin", label: "Admin Dashboard", routePath: "/api/admin", endpointCount: 2, category: "admin", endpoints: ["GET /developer-signups", "DELETE /developer-signups/:id"] },
      { name: "plenumnet-github", label: "GitHub Integration", routePath: "/api/github", endpointCount: 9, category: "admin", endpoints: ["POST /token", "GET /status", "GET /repos/:owner/:repo/branches", "GET /repos/:owner/:repo/contents", "GET /file/:owner/:repo", "PUT /file/:owner/:repo", "DELETE /file/:owner/:repo", "POST /push-workflows/:owner/:repo", "POST /push-batch/:owner/:repo"] },
      { name: "plenumnet-kong", label: "Kong Gateway Management", routePath: "/api/kong", endpointCount: 12, category: "admin", endpoints: ["GET /status", "GET /organization", "GET /control-planes", "GET /control-planes/:cpId/services", "GET /control-planes/:cpId/routes", "GET /control-planes/:cpId/plugins", "GET /config", "POST /control-planes/:cpId/services", "POST /control-planes/:cpId/sync-plenumnet", "POST /save-to-github", "GET /control-planes/:cpId/deploy-instructions", "POST /control-planes/:cpId/deploy-to-cloud"] },
      { name: "plenumnet-health", label: "Health & Observability", routePath: "/api/health", endpointCount: 1, category: "platform", endpoints: ["GET /"] }
    ];

    const totalEndpoints = catalog.reduce((sum, s) => sum + s.endpointCount, 0);
    res.json({
      totalServices: catalog.length,
      totalEndpoints,
      baseUrl,
      categories: {
        core: catalog.filter(s => s.category === "core"),
        tools: catalog.filter(s => s.category === "tools"),
        reference: catalog.filter(s => s.category === "reference"),
        platform: catalog.filter(s => s.category === "platform"),
        admin: catalog.filter(s => s.category === "admin")
      },
      services: catalog
    });
  });

  // Save Kong config to GitHub (Admin only)
  app.post("/api/kong/save-to-github", requireAdmin, async (req: any, res) => {
    try {
      const user = req.adminUser; // Set by requireAdmin middleware
      const token = resolveGitHubToken(user);
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured. Please add your GitHub token in the GitHub Manager or set GITHUB_TOKEN env var." });
      }

      const { owner, repo, path = "kong/kong.yaml", message = "Update Kong Konnect configuration" } = req.body;
      
      if (!owner || !repo) {
        return res.status(400).json({ error: "Owner and repo are required" });
      }

      const fs = await import('fs/promises');
      const pathModule = await import('path');
      const configPath = pathModule.join(process.cwd(), 'kong', 'kong.yaml');
      const config = await fs.readFile(configPath, 'utf-8');
      const content = Buffer.from(config).toString('base64');

      const existingResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`, {
        headers: {
          "Authorization": `token ${token}`,
          "Accept": "application/vnd.github.v3+json"
        }
      });

      let sha: string | undefined;
      if (existingResponse.ok) {
        const existingFile = await existingResponse.json();
        sha = existingFile.sha;
      }

      const createResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`, {
        method: 'PUT',
        headers: {
          "Authorization": `token ${token}`,
          "Accept": "application/vnd.github.v3+json",
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          message,
          content,
          sha
        })
      });

      if (!createResponse.ok) {
        const errorData = await createResponse.json().catch(() => ({}));
        return res.status(createResponse.status).json({ 
          error: `GitHub API error: ${createResponse.status}`,
          details: errorData 
        });
      }

      const result = await createResponse.json();
      res.json({ 
        success: true, 
        message: sha ? "Configuration updated" : "Configuration created",
        url: result.content?.html_url,
        sha: result.content?.sha
      });
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Get data plane deployment instructions for a control plane
  app.get("/api/kong/control-planes/:cpId/deploy-instructions", async (req, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      
      // Get control plane details
      const cpResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}`, {
        headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
      });

      if (!cpResponse.ok) {
        return res.status(cpResponse.status).json({ error: "Failed to fetch control plane details" });
      }

      const cpData = await cpResponse.json();
      const controlPlaneEndpoint = cpData.config?.control_plane_endpoint;
      const telemetryEndpoint = cpData.config?.telemetry_endpoint;
      const clusterType = cpData.config?.cluster_type;
      const proxyUrls = cpData.config?.proxy_urls || [];

      // Generate Docker deployment command
      const dockerCommand = `docker run -d --name kong-dp \\
  -e "KONG_ROLE=data_plane" \\
  -e "KONG_DATABASE=off" \\
  -e "KONG_VITALS=off" \\
  -e "KONG_CLUSTER_MTLS=pki" \\
  -e "KONG_CLUSTER_CONTROL_PLANE=${controlPlaneEndpoint?.replace('https://', '')}:443" \\
  -e "KONG_CLUSTER_SERVER_NAME=${controlPlaneEndpoint?.replace('https://', '')}" \\
  -e "KONG_CLUSTER_TELEMETRY_ENDPOINT=${telemetryEndpoint?.replace('https://', '')}:443" \\
  -e "KONG_CLUSTER_TELEMETRY_SERVER_NAME=${telemetryEndpoint?.replace('https://', '')}" \\
  -e "KONG_CLUSTER_CERT=/config/tls.crt" \\
  -e "KONG_CLUSTER_CERT_KEY=/config/tls.key" \\
  -e "KONG_LUA_SSL_TRUSTED_CERTIFICATE=system" \\
  -e "KONG_KONNECT_MODE=on" \\
  -p 8000:8000 \\
  -p 8443:8443 \\
  kong/kong-gateway:3.6`;

      res.json({
        success: true,
        controlPlane: {
          id: cpId,
          name: cpData.name,
          clusterType,
          controlPlaneEndpoint,
          telemetryEndpoint,
          proxyUrls
        },
        hasProxyUrl: proxyUrls.length > 0,
        deploymentInstructions: {
          docker: {
            title: "Docker Deployment",
            description: "Run a Kong Gateway data plane using Docker",
            prerequisites: [
              "Docker installed on your machine",
              "Generate TLS certificates from Kong Konnect UI",
              "Download certificates to ./config/ directory"
            ],
            steps: [
              "Go to Kong Konnect → Gateway Manager → Data Plane Nodes",
              "Click 'New Data Plane Node' → 'Linux (Docker)'",
              "Download the generated certificates (tls.crt, tls.key)",
              "Place certificates in a ./config/ directory",
              "Run the Docker command below"
            ],
            command: dockerCommand
          },
          kubernetes: {
            title: "Kubernetes Deployment",
            description: "Deploy Kong Gateway on Kubernetes using Helm",
            command: `helm repo add kong https://charts.konghq.com
helm repo update
helm install kong kong/kong --namespace kong --create-namespace \\
  --set ingressController.enabled=false \\
  --set env.role=data_plane \\
  --set env.database=off \\
  --set env.cluster_control_plane="${controlPlaneEndpoint?.replace('https://', '')}:443" \\
  --set env.cluster_telemetry_endpoint="${telemetryEndpoint?.replace('https://', '')}:443"`
          }
        },
        proxyAccessUrls: proxyUrls.length > 0 ? {
          timing: `${proxyUrls[0]?.protocol}://${proxyUrls[0]?.host}/api/timing/timestamp`,
          ternary: `${proxyUrls[0]?.protocol}://${proxyUrls[0]?.host}/api/ternary/convert`,
          phase: `${proxyUrls[0]?.protocol}://${proxyUrls[0]?.host}/api/phase/config/balanced`
        } : null
      });
    } catch (error: unknown) {
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Generate deployment package with certificates (Admin only)
  app.post("/api/kong/control-planes/:cpId/generate-deployment", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const crypto = await import('crypto');

      // Generate self-signed certificate
      const { privateKey, publicKey } = crypto.generateKeyPairSync('rsa', {
        modulusLength: 2048,
        publicKeyEncoding: { type: 'spki', format: 'pem' },
        privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
      });

      // Create self-signed certificate using forge-like approach with Node.js
      const certInfo = {
        subject: '/CN=kong-dp/O=PlenumNET/C=US',
        issuer: '/CN=kong-dp/O=PlenumNET/C=US',
        serialNumber: crypto.randomBytes(16).toString('hex'),
        notBefore: new Date(),
        notAfter: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000 * 10) // 10 years
      };

      const { execFile } = await import('child_process');
      const { promisify } = await import('util');
      const execFileAsync = promisify(execFile);
      const fs = await import('fs/promises');
      const path = await import('path');
      
      const tempDir = path.join('/tmp', `kong-certs-${Date.now()}`);
      await fs.mkdir(tempDir, { recursive: true });
      
      const keyPath = path.join(tempDir, 'tls.key');
      const certPath = path.join(tempDir, 'tls.crt');

      await execFileAsync('openssl', [
        'req', '-new', '-x509', '-nodes', '-newkey', 'rsa:2048',
        '-subj', '/CN=kong-dp/O=PlenumNET/C=US',
        '-keyout', keyPath, '-out', certPath, '-days', '3650'
      ]);
      
      const tlsKey = await fs.readFile(keyPath, 'utf-8');
      const tlsCert = await fs.readFile(certPath, 'utf-8');

      // Upload certificate to Kong Konnect
      const uploadResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}/dp-client-certificates`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ cert: tlsCert })
      });

      let certUploadResult = null;
      if (uploadResponse.ok) {
        certUploadResult = await uploadResponse.json();
      } else {
        const errorText = await uploadResponse.text();
        log.error("Certificate upload failed:", errorText);
      }

      // Get control plane details for docker-compose
      const cpResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}`, {
        headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
      });
      const cpData = await cpResponse.json();
      const controlPlaneEndpoint = cpData.config?.control_plane_endpoint?.replace('https://', '') || '';
      const telemetryEndpoint = cpData.config?.telemetry_endpoint?.replace('https://', '') || '';

      // Generate docker-compose.yml
      const dockerCompose = `version: '3.8'

services:
  kong-dp:
    image: kong/kong-gateway:3.6
    container_name: kong-plenumnet-dp
    restart: unless-stopped
    environment:
      - KONG_ROLE=data_plane
      - KONG_DATABASE=off
      - KONG_VITALS=off
      - KONG_CLUSTER_MTLS=pki
      - KONG_CLUSTER_CONTROL_PLANE=${controlPlaneEndpoint}:443
      - KONG_CLUSTER_SERVER_NAME=${controlPlaneEndpoint}
      - KONG_CLUSTER_TELEMETRY_ENDPOINT=${telemetryEndpoint}:443
      - KONG_CLUSTER_TELEMETRY_SERVER_NAME=${telemetryEndpoint}
      - KONG_CLUSTER_CERT=/etc/secrets/tls.crt
      - KONG_CLUSTER_CERT_KEY=/etc/secrets/tls.key
      - KONG_LUA_SSL_TRUSTED_CERTIFICATE=system
      - KONG_KONNECT_MODE=on
      - KONG_PROXY_LISTEN=0.0.0.0:8000, 0.0.0.0:8443 ssl
    ports:
      - "8000:8000"   # HTTP Proxy
      - "8443:8443"   # HTTPS Proxy
    volumes:
      - ./certs:/etc/secrets:ro
    healthcheck:
      test: ["CMD", "kong", "health"]
      interval: 30s
      timeout: 10s
      retries: 3

# PlenumNET Kong Gateway Data Plane
# Generated: ${new Date().toISOString()}
# Control Plane: ${cpData.name}
# 
# SETUP INSTRUCTIONS:
# 1. Save this file as docker-compose.yml
# 2. Create a 'certs' directory: mkdir certs
# 3. Save tls.crt and tls.key to the certs directory
# 4. Run: docker-compose up -d
# 5. Access PlenumNET APIs at: http://localhost:8000/api/timing/timestamp
`;

      // Generate deployment script
      const deployScript = `#!/bin/bash
# PlenumNET Kong Gateway Deployment Script
# Generated: ${new Date().toISOString()}

set -e

echo "🚀 Deploying PlenumNET Kong Gateway Data Plane..."

# Create directories
mkdir -p kong-plenumnet/certs

# Write certificates
cat > kong-plenumnet/certs/tls.crt << 'CERTEOF'
${tlsCert}CERTEOF

cat > kong-plenumnet/certs/tls.key << 'KEYEOF'
${tlsKey}KEYEOF

# Write docker-compose
cat > kong-plenumnet/docker-compose.yml << 'COMPOSEEOF'
${dockerCompose}COMPOSEEOF

# Set permissions
chmod 600 kong-plenumnet/certs/tls.key

# Deploy
cd kong-plenumnet
docker-compose up -d

echo ""
echo "✅ Kong Gateway Data Plane deployed successfully!"
echo ""
echo "🔗 Proxy URLs:"
echo "   HTTP:  http://localhost:8000"
echo "   HTTPS: https://localhost:8443"
echo ""
echo "📡 PlenumNET API Endpoints:"
echo "   Timing:    http://localhost:8000/api/timing/timestamp"
echo "   Ternary:   http://localhost:8000/api/ternary/convert"
echo "   Phase:     http://localhost:8000/api/phase/config/balanced"
echo "   Demo:      http://localhost:8000/api/demo/stats"
echo ""
echo "📊 View logs: docker-compose logs -f"
`;

      // Cleanup temp files
      await fs.rm(tempDir, { recursive: true, force: true });

      res.json({
        success: true,
        message: "Deployment package generated successfully",
        certificateUploaded: !!certUploadResult,
        certificateId: certUploadResult?.id,
        controlPlane: {
          id: cpId,
          name: cpData.name,
          endpoint: controlPlaneEndpoint
        },
        files: {
          "tls.crt": tlsCert,
          "tls.key": tlsKey,
          "docker-compose.yml": dockerCompose,
          "deploy.sh": deployScript
        },
        instructions: [
          "1. Copy deploy.sh to your server",
          "2. Make it executable: chmod +x deploy.sh",
          "3. Run: ./deploy.sh",
          "4. Access APIs at http://your-server:8000/api/timing/timestamp"
        ]
      });
    } catch (error: unknown) {
      log.error("Generate deployment error:", error);
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });

  // Deploy Kong to cloud platform (Render/Railway) via GitHub
  app.post("/api/kong/control-planes/:cpId/deploy-to-cloud", requireAdmin, async (req: any, res) => {
    try {
      if (!KONG_KONNECT_TOKEN) {
        return res.status(401).json({ error: "Kong Konnect token not configured" });
      }

      const { cpId } = req.params;
      const { platform = "render", owner, repo } = req.body;
      const token = resolveGitHubToken(req.adminUser);

      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured. Set a personal token in GitHub Manager or ensure GITHUB_TOKEN env var is set." });
      }

      if (!owner || !repo) {
        return res.status(400).json({ error: "GitHub owner and repo required" });
      }

      const { execFile } = await import('child_process');
      const { promisify } = await import('util');
      const execFileAsync = promisify(execFile);
      const fs = await import('fs/promises');
      const path = await import('path');
      
      const tempDir = path.join('/tmp', `kong-cloud-${Date.now()}`);
      await fs.mkdir(tempDir, { recursive: true });
      
      const keyPath = path.join(tempDir, 'tls.key');
      const certPath = path.join(tempDir, 'tls.crt');

      await execFileAsync('openssl', [
        'req', '-new', '-x509', '-nodes', '-newkey', 'rsa:2048',
        '-subj', '/CN=kong-dp/O=PlenumNET/C=US',
        '-keyout', keyPath, '-out', certPath, '-days', '3650'
      ]);
      
      const tlsKey = await fs.readFile(keyPath, 'utf-8');
      const tlsCert = await fs.readFile(certPath, 'utf-8');

      // Upload certificate to Kong Konnect
      await fetch(`${KONG_API_BASE}/control-planes/${cpId}/dp-client-certificates`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${KONG_KONNECT_TOKEN}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ cert: tlsCert })
      });

      // Get control plane details
      const cpResponse = await fetch(`${KONG_API_BASE}/control-planes/${cpId}`, {
        headers: { "Authorization": `Bearer ${KONG_KONNECT_TOKEN}` }
      });
      const cpData = await cpResponse.json();
      const controlPlaneEndpoint = cpData.config?.control_plane_endpoint?.replace('https://', '') || '';
      const telemetryEndpoint = cpData.config?.telemetry_endpoint?.replace('https://', '') || '';

      // Create Dockerfile - private key loaded from env var at runtime (secure)
      // Note: Render uses $PORT env var, and health checks hit the main service port
      const dockerfile = `FROM kong/kong-gateway:3.6

ENV KONG_ROLE=data_plane
ENV KONG_DATABASE=off
ENV KONG_VITALS=off
ENV KONG_CLUSTER_MTLS=pki
ENV KONG_LUA_SSL_TRUSTED_CERTIFICATE=system
ENV KONG_KONNECT_MODE=on

ENV KONG_CLUSTER_CONTROL_PLANE=${controlPlaneEndpoint}:443
ENV KONG_CLUSTER_SERVER_NAME=${controlPlaneEndpoint}
ENV KONG_CLUSTER_TELEMETRY_ENDPOINT=${telemetryEndpoint}:443
ENV KONG_CLUSTER_TELEMETRY_SERVER_NAME=${telemetryEndpoint}

RUN mkdir -p /etc/kong/certs

# Cert is baked in, key is loaded at runtime from secret env var
COPY tls.crt /etc/kong/certs/tls.crt
ENV KONG_CLUSTER_CERT=/etc/kong/certs/tls.crt

# Startup script to load private key from env var securely
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Render uses PORT env var - status endpoint on same port for health checks
EXPOSE 8000

ENTRYPOINT ["/entrypoint.sh"]
`;

      // Entrypoint script that loads TLS key from env var (not in git)
      const entrypointScript = `#!/bin/sh
set -e

# Write private key from secret env var to file
if [ -n "\$KONG_TLS_KEY" ]; then
  echo "\$KONG_TLS_KEY" > /etc/kong/certs/tls.key
  chmod 600 /etc/kong/certs/tls.key
  export KONG_CLUSTER_CERT_KEY=/etc/kong/certs/tls.key
else
  echo "ERROR: KONG_TLS_KEY environment variable is required"
  exit 1
fi

# Configure proxy to listen on PORT (required for Render/Railway)
export KONG_PROXY_LISTEN="0.0.0.0:\${PORT:-8000}"
export KONG_STATUS_LISTEN="0.0.0.0:\${PORT:-8000}"

exec kong docker-start
`;

      // Create render.yaml - proper format for Render Blueprint
      const renderYaml = `services:
  - type: web
    name: kong-plenumnet
    runtime: docker
    dockerfilePath: ./kong-deploy/Dockerfile
    dockerContext: ./kong-deploy
    region: oregon
    plan: free
    healthCheckPath: /
    envVars:
      - key: PORT
        value: "8000"
      - key: KONG_TLS_KEY
        sync: false
`;

      // Push files to GitHub - NO private key!
      const filesToPush = [
        { path: "kong-deploy/Dockerfile", content: dockerfile },
        { path: "kong-deploy/entrypoint.sh", content: entrypointScript },
        { path: "kong-deploy/tls.crt", content: tlsCert },
        { path: "render.yaml", content: renderYaml }
      ];

      const pushErrors: string[] = [];
      for (const file of filesToPush) {
        const content = Buffer.from(file.content).toString('base64');
        
        // Check if file exists
        const checkResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${file.path}`, {
          headers: {
            "Authorization": `token ${token}`,
            "Accept": "application/vnd.github.v3+json"
          }
        });
        
        let sha: string | undefined;
        if (checkResponse.ok) {
          const existingFile = await checkResponse.json();
          sha = existingFile.sha;
        }

        const pushResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${file.path}`, {
          method: 'PUT',
          headers: {
            "Authorization": `token ${token}`,
            "Accept": "application/vnd.github.v3+json",
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            message: `Add Kong deployment: ${file.path}`,
            content,
            sha
          })
        });

        if (!pushResponse.ok) {
          const errData = await pushResponse.json().catch(() => ({}));
          pushErrors.push(`${file.path}: ${errData.message || pushResponse.statusText}`);
        }
      }

      if (pushErrors.length > 0) {
        return res.status(500).json({ 
          error: "Failed to push some files to GitHub", 
          details: pushErrors 
        });
      }

      // Cleanup
      await fs.rm(tempDir, { recursive: true, force: true });

      // Generate deploy URLs
      const renderDeployUrl = `https://render.com/deploy?repo=https://github.com/${owner}/${repo}`;
      const railwayDeployUrl = `https://railway.app/template?template=https://github.com/${owner}/${repo}`;

      res.json({
        success: true,
        message: "Deployment files pushed to GitHub!",
        platform,
        githubRepo: `https://github.com/${owner}/${repo}`,
        deployUrls: {
          render: renderDeployUrl,
          railway: railwayDeployUrl
        },
        controlPlane: {
          id: cpId,
          name: cpData.name,
          endpoint: controlPlaneEndpoint
        },
        // Include the private key for user to copy to cloud platform env vars
        tlsKey: tlsKey,
        instructions: [
          `1. Files pushed to https://github.com/${owner}/${repo}`,
          `2. Copy the TLS private key below`,
          `3. Click the deploy link for ${platform}`,
          `4. In the cloud platform, set KONG_TLS_KEY env var to the private key`,
          `5. Deploy the service`,
          `6. Your Kong proxy will be live at the provided URL`
        ]
      });
    } catch (error: unknown) {
      log.error("Cloud deployment error:", error);
      res.status(500).json({ error: toErrorMessage(error) });
    }
  });
}
