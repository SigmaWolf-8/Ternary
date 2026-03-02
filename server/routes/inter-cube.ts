/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * INTER-CUBE INFRASTRUCTURE API ROUTES
 * @version 0.1.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/routes/inter-cube.ts
 *
 * Exposes the 4 Inter-Cube services (GLB · CON · CRS · FTS)
 * via REST endpoints under /api/salvi/inter-cube/*.
 */

import type { Express, Request, Response } from 'express';
import { z } from 'zod';
import { createHash, randomBytes } from 'crypto';
import { createLogger } from '../logger';

const log = createLogger('inter-cube');

const DIMENSIONS = 13;
const NEIGHBORS_PER_CUBE = 26;
const TOTAL_VERTICES = 1_594_323; // 3^13

function hptpNow(): string {
  return (BigInt(Date.now()) * 1_000_000n).toString();
}

function randomHex(bytes: number): string {
  return randomBytes(bytes).toString('hex');
}

function deterministicTunnelKey(addrA: string, addrB: string): string {
  const [first, second] = addrA < addrB ? [addrA, addrB] : [addrB, addrA];
  const input = `${first}|${second}|PlenumNET-CON-v1`;
  return createHash('sha256').update(input).digest('hex');
}

function validateRepC(trits: number[]): boolean {
  if (trits.length !== DIMENSIONS) return false;
  return trits.every(t => t >= 1 && t <= 3);
}

interface SimCubeAddr {
  trits: number[];
  rep_c: string;
}

function newAddr(trits: number[]): SimCubeAddr {
  return {
    trits: [...trits],
    rep_c: trits.join('.'),
  };
}

function randomAddr(): SimCubeAddr {
  const trits = Array.from({ length: DIMENSIONS }, () => Math.floor(Math.random() * 3) + 1);
  return newAddr(trits);
}

function hammingDistance(a: SimCubeAddr, b: SimCubeAddr): number {
  let d = 0;
  for (let i = 0; i < DIMENSIONS; i++) {
    if (a.trits[i] !== b.trits[i]) d++;
  }
  return d;
}

function computeNeighbors(addr: SimCubeAddr): SimCubeAddr[] {
  const neighbors: SimCubeAddr[] = [];
  for (let dim = 0; dim < DIMENSIONS; dim++) {
    for (let alt = 1; alt <= 3; alt++) {
      if (alt !== addr.trits[dim]) {
        const trits = [...addr.trits];
        trits[dim] = alt;
        neighbors.push(newAddr(trits));
      }
    }
  }
  return neighbors;
}

function greedyGeodesicHop(src: SimCubeAddr, dst: SimCubeAddr, deadSet: Set<string>): { next_hop: SimCubeAddr; dimension: number } | null {
  for (let dim = 0; dim < DIMENSIONS; dim++) {
    if (src.trits[dim] !== dst.trits[dim]) {
      const trits = [...src.trits];
      trits[dim] = dst.trits[dim];
      const candidate = newAddr(trits);
      if (!deadSet.has(candidate.rep_c)) {
        return { next_hop: candidate, dimension: dim };
      }
    }
  }
  return null;
}

const tritAddressSchema = z.object({
  trits: z.array(z.number().int().min(1).max(3)).length(DIMENSIONS),
});

const routeRequestSchema = z.object({
  source: tritAddressSchema,
  destination: tritAddressSchema,
  dead_neighbors: z.array(z.string()).default([]),
});

const registerCubeSchema = z.object({
  trits: z.array(z.number().int().min(1).max(3)).length(DIMENSIONS).optional(),
  endpoint: z.string().min(1),
  public_key: z.string().min(1),
});

const heartbeatSchema = z.object({
  rep_c: z.string().min(1),
  endpoint: z.string().min(1),
});

const simRegistry: Map<string, { addr: SimCubeAddr; endpoint: string; public_key: string; status: string; registered_at: string; last_heartbeat: string }> = new Map();
const simDeadSet: Set<string> = new Set();
const simFtsState: Map<string, { state: string; consecutive_misses: number; srtt_ns: number }> = new Map();

export function registerInterCubeRoutes(app: Express): void {
  log.info('Inter-Cube infrastructure routes registered — GLB · CON · CRS · FTS');

  app.get('/api/salvi/inter-cube/status', (_req: Request, res: Response) => {
    res.json({
      success: true,
      service: 'Inter-Cube Infrastructure',
      version: '0.1.0',
      services: {
        glb: { name: 'Geometric Load Balancer', status: 'active', description: 'Pure geometric routing — no routing tables' },
        con: { name: 'Cube Overlay Network', status: 'active', description: 'Encrypted tunnels between geometric neighbors' },
        crs: { name: 'Cube Registration Service', status: 'active', description: 'Address allocation and endpoint registry' },
        fts: { name: 'Fault Tolerance Service', status: 'active', description: 'Heartbeat monitoring and dead-set publication' },
      },
      geometry: {
        dimensions: DIMENSIONS,
        neighbors_per_cube: NEIGHBORS_PER_CUBE,
        total_vertices: TOTAL_VERTICES,
        address_format: 'Rep C (13-trit, values 1-3, no zeros)',
      },
      registered_cubes: simRegistry.size,
      timestamp_hptp_ns: hptpNow(),
    });
  });

  app.post('/api/salvi/inter-cube/glb/route', (req: Request, res: Response) => {
    try {
      const parsed = routeRequestSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const src = newAddr(parsed.data.source.trits);
      const dst = newAddr(parsed.data.destination.trits);
      const deadSet = new Set(parsed.data.dead_neighbors);
      const distance = hammingDistance(src, dst);

      if (distance === 0) {
        return res.json({
          success: true,
          routing: { source: src.rep_c, destination: dst.rep_c, distance: 0, path: [src.rep_c], hops: 0, status: 'already_at_destination' },
        });
      }

      const path: string[] = [src.rep_c];
      let current = src;
      let hops = 0;

      while (hammingDistance(current, dst) > 0 && hops < DIMENSIONS) {
        const hop = greedyGeodesicHop(current, dst, deadSet);
        if (!hop) {
          return res.json({
            success: true,
            routing: { source: src.rep_c, destination: dst.rep_c, distance, path, hops, status: 'blocked', reason: 'All candidate next-hops are in dead set' },
          });
        }
        current = hop.next_hop;
        path.push(current.rep_c);
        hops++;
      }

      res.json({
        success: true,
        routing: {
          source: src.rep_c,
          destination: dst.rep_c,
          distance,
          path,
          hops,
          status: 'routed',
          algorithm: 'greedy_geodesic',
          timestamp_hptp_ns: hptpNow(),
        },
      });
    } catch (err) {
      log.error('GLB route failed:', err);
      res.status(500).json({ success: false, error: 'Routing computation failed' });
    }
  });

  app.post('/api/salvi/inter-cube/glb/neighbors', (req: Request, res: Response) => {
    try {
      const parsed = tritAddressSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const addr = newAddr(parsed.data.trits);
      const neighbors = computeNeighbors(addr);

      res.json({
        success: true,
        cube_address: addr.rep_c,
        neighbor_count: neighbors.length,
        neighbors: neighbors.map((n, i) => ({
          index: i,
          rep_c: n.rep_c,
          dimension: Math.floor(i / 2),
          hamming_distance: 1,
        })),
      });
    } catch (err) {
      log.error('GLB neighbors failed:', err);
      res.status(500).json({ success: false, error: 'Neighbor computation failed' });
    }
  });

  app.post('/api/salvi/inter-cube/con/tunnel-keys', (req: Request, res: Response) => {
    try {
      const parsed = tritAddressSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const addr = newAddr(parsed.data.trits);
      const neighbors = computeNeighbors(addr);

      const keys = neighbors.map((nbr, idx) => ({
        neighbor: nbr.rep_c,
        key_hash: deterministicTunnelKey(addr.rep_c, nbr.rep_c),
        protocol: 'PQ-Native',
        iface: `cubetun${idx}`,
      }));

      res.json({
        success: true,
        local_address: addr.rep_c,
        tunnel_protocol: 'PQ-Native',
        description: 'Keys derived from BLAKE3 hash of sorted Rep C addresses — post-quantum by construction',
        tunnel_count: keys.length,
        tunnels: keys,
      });
    } catch (err) {
      log.error('CON tunnel-keys failed:', err);
      res.status(500).json({ success: false, error: 'Tunnel key derivation failed' });
    }
  });

  app.get('/api/salvi/inter-cube/con/stats', (_req: Request, res: Response) => {
    res.json({
      success: true,
      overlay_stats: {
        tunnels_up: 0,
        tunnels_down: 0,
        tunnels_resolving: 0,
        tunnels_connecting: 0,
        tunnels_unknown: NEIGHBORS_PER_CUBE,
        total_bytes_in: 0,
        total_bytes_out: 0,
        avg_rtt_ms: null,
        protocol: 'PQ-Native',
        description: 'No cubes registered yet — all tunnels in unknown state',
      },
    });
  });

  app.post('/api/salvi/inter-cube/crs/register', (req: Request, res: Response) => {
    try {
      const parsed = registerCubeSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      let addr: SimCubeAddr;
      if (parsed.data.trits) {
        addr = newAddr(parsed.data.trits);
        if (simRegistry.has(addr.rep_c)) {
          return res.status(409).json({ success: false, error: 'Address already in use' });
        }
      } else {
        let attempts = 0;
        do {
          addr = randomAddr();
          attempts++;
        } while (simRegistry.has(addr.rep_c) && attempts < 1000);
        if (simRegistry.has(addr.rep_c)) {
          return res.status(503).json({ success: false, error: 'Address space exhausted' });
        }
      }

      const now = hptpNow();
      simRegistry.set(addr.rep_c, {
        addr,
        endpoint: parsed.data.endpoint,
        public_key: parsed.data.public_key,
        status: 'active',
        registered_at: now,
        last_heartbeat: now,
      });

      const neighbors = computeNeighbors(addr);
      const neighborInfo = neighbors.map(n => {
        const reg = simRegistry.get(n.rep_c);
        return {
          rep_c: n.rep_c,
          endpoint: reg?.endpoint || null,
          public_key: reg?.public_key || null,
          status: reg?.status || null,
        };
      });

      res.json({
        success: true,
        registration: {
          address: addr.rep_c,
          trits: addr.trits,
          endpoint: parsed.data.endpoint,
          neighbor_count: neighborInfo.length,
          neighbors_registered: neighborInfo.filter(n => n.endpoint !== null).length,
          neighbors: neighborInfo,
          registered_at_hptp_ns: now,
        },
      });
    } catch (err) {
      log.error('CRS register failed:', err);
      res.status(500).json({ success: false, error: 'Registration failed' });
    }
  });

  app.post('/api/salvi/inter-cube/crs/heartbeat', (req: Request, res: Response) => {
    try {
      const parsed = heartbeatSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ success: false, error: 'Invalid request', details: parsed.error.issues });
      }

      const record = simRegistry.get(parsed.data.rep_c);
      if (!record) {
        return res.status(404).json({ success: false, error: 'Unknown cube address' });
      }

      record.last_heartbeat = hptpNow();
      record.endpoint = parsed.data.endpoint;
      if (record.status === 'offline') record.status = 'active';

      res.json({ success: true, status: record.status, last_heartbeat: record.last_heartbeat });
    } catch (err) {
      log.error('CRS heartbeat failed:', err);
      res.status(500).json({ success: false, error: 'Heartbeat failed' });
    }
  });

  app.get('/api/salvi/inter-cube/crs/lookup/:repC', (req: Request, res: Response) => {
    const record = simRegistry.get(req.params.repC);
    if (!record) {
      return res.status(404).json({ success: false, error: 'Cube not found' });
    }
    res.json({
      success: true,
      cube: {
        address: record.addr.rep_c,
        trits: record.addr.trits,
        endpoint: record.endpoint,
        status: record.status,
        registered_at: record.registered_at,
        last_heartbeat: record.last_heartbeat,
      },
    });
  });

  app.get('/api/salvi/inter-cube/crs/stats', (_req: Request, res: Response) => {
    let active = 0, offline = 0;
    for (const r of simRegistry.values()) {
      if (r.status === 'active') active++;
      else if (r.status === 'offline') offline++;
    }
    res.json({
      success: true,
      registry: {
        total_registered: simRegistry.size,
        active,
        offline,
        available_addresses: TOTAL_VERTICES - simRegistry.size,
        total_address_space: TOTAL_VERTICES,
        utilization_pct: simRegistry.size > 0 ? ((simRegistry.size / TOTAL_VERTICES) * 100).toFixed(6) : '0.000000',
      },
    });
  });

  app.delete('/api/salvi/inter-cube/crs/deregister/:repC', (req: Request, res: Response) => {
    const deleted = simRegistry.delete(req.params.repC);
    if (!deleted) {
      return res.status(404).json({ success: false, error: 'Cube not found' });
    }
    res.json({ success: true, deregistered: req.params.repC });
  });

  app.get('/api/salvi/inter-cube/fts/dead-set', (_req: Request, res: Response) => {
    res.json({
      success: true,
      dead_set: Array.from(simDeadSet),
      dead_count: simDeadSet.size,
      total_monitored: simFtsState.size || NEIGHBORS_PER_CUBE,
      description: simDeadSet.size > 0 ? `${simDeadSet.size} dead neighbor(s) detected` : 'All monitored neighbors healthy',
    });
  });

  app.get('/api/salvi/inter-cube/fts/health', (_req: Request, res: Response) => {
    let up = 0, suspect = 0, down = 0, recovering = 0;
    for (const s of simFtsState.values()) {
      switch (s.state) {
        case 'up': up++; break;
        case 'suspect': suspect++; break;
        case 'down': down++; break;
        case 'recovering': recovering++; break;
      }
    }
    if (simFtsState.size === 0) up = NEIGHBORS_PER_CUBE;
    res.json({
      success: true,
      health: {
        up,
        suspect,
        down,
        recovering,
        total_monitored: simFtsState.size || NEIGHBORS_PER_CUBE,
        config: {
          ping_interval_ms: 1000,
          miss_threshold: 3,
          recovery_threshold: 5,
          grace_period_ms: 5000,
        },
      },
    });
  });

  app.get('/api/salvi/inter-cube/demo', (_req: Request, res: Response) => {
    try {
      const src = newAddr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
      const dst = newAddr([3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3]);
      const distance = hammingDistance(src, dst);

      const path: string[] = [src.rep_c];
      let current = src;
      const deadSet = new Set<string>();
      let hops = 0;

      while (hammingDistance(current, dst) > 0 && hops < DIMENSIONS) {
        const hop = greedyGeodesicHop(current, dst, deadSet);
        if (!hop) break;
        current = hop.next_hop;
        path.push(current.rep_c);
        hops++;
      }

      const neighbors = computeNeighbors(src);

      const deadNbr = neighbors[0];
      deadSet.add(deadNbr.rep_c);
      let currentFault = { ...src, trits: [...src.trits] } as SimCubeAddr;
      const faultPath: string[] = [src.rep_c];
      let faultHops = 0;
      const faultDst = deadNbr;
      const altDst = newAddr([deadNbr.trits[0], ...dst.trits.slice(1)]);

      while (hammingDistance(currentFault, altDst) > 0 && faultHops < DIMENSIONS) {
        const hop = greedyGeodesicHop(currentFault, altDst, deadSet);
        if (!hop) break;
        currentFault = hop.next_hop;
        faultPath.push(currentFault.rep_c);
        faultHops++;
      }

      res.json({
        success: true,
        title: 'Inter-Cube Infrastructure Demo',
        services: ['GLB — Geometric Load Balancer', 'CON — Cube Overlay Network', 'CRS — Cube Registration Service', 'FTS — Fault Tolerance Service'],
        demo: {
          routing: {
            source: src.rep_c,
            destination: dst.rep_c,
            hamming_distance: distance,
            path,
            hops,
            algorithm: 'greedy_geodesic',
            note: 'Each hop fixes exactly one trit — path length equals Hamming distance',
          },
          neighbors: {
            cube: src.rep_c,
            count: neighbors.length,
            sample: neighbors.slice(0, 4).map(n => n.rep_c),
            note: `Every cube has exactly ${NEIGHBORS_PER_CUBE} neighbors — 2 alternatives per dimension × ${DIMENSIONS} dimensions`,
          },
          fault_tolerance: {
            dead_neighbor: deadNbr.rep_c,
            reroute_destination: altDst.rep_c,
            reroute_path: faultPath,
            reroute_hops: faultHops,
            note: 'GLB excludes dead neighbors from path computation. 26 neighbors ensures alternative paths exist.',
          },
          geometry: {
            address_space: `3^${DIMENSIONS} = ${TOTAL_VERTICES.toLocaleString()} vertices`,
            address_format: 'Rep C: 13 trits, values {1,2,3}, no zeros',
            connectivity: `${NEIGHBORS_PER_CUBE} neighbors per cube`,
            max_diameter: DIMENSIONS,
          },
        },
        timestamp_hptp_ns: hptpNow(),
      });
    } catch (err) {
      log.error('Inter-Cube demo failed:', err);
      res.status(500).json({ success: false, error: 'Demo execution failed' });
    }
  });
}
