/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * This file is part of the Salvi Framework / PlenumNET platform.
 *
 * Inter-Cube Infrastructure Services — Express API Routes
 *
 * TypeScript implementation of the four inter-cube services (GLB, CON, CRS, FTS)
 * for the PlenumNET Express server. Uses the same pure-math routing principles
 * as the Rust crate: geometry IS the routing protocol, no routing tables.
 *
 * All addresses are Rep C (bijective ternary {1, 2, 3}). Zero never appears.
 */

import { Router, Request, Response } from 'express';
import type { TritC } from '../salvi-core/ternary-types';

const DIMENSIONS = 13;
const NEIGHBORS_PER_CUBE = 26;
const TOTAL_VERTICES = 1_594_323;
const VALID_DIGITS: TritC[] = [1, 2, 3];
const DEFAULT_FLOW_TTL_MS = 60_000;
const DEFAULT_MISS_THRESHOLD = 3;
const DEFAULT_RECOVERY_THRESHOLD = 5;
const DEFAULT_GRACE_PERIOD_MS = 5_000;

interface CubeAddr {
  trits: number[];
}

function validateRepC(trits: number[]): boolean {
  if (trits.length !== DIMENSIONS) return false;
  let valid = 1;
  for (let i = 0; i < DIMENSIONS; i++) {
    const t = trits[i];
    valid &= (t >= 1 && t <= 3) ? 1 : 0;
  }
  return valid === 1;
}

function cubeAddr(trits: number[]): CubeAddr {
  if (!validateRepC(trits)) {
    throw new Error('Invalid Rep C address: all trits must be in {1, 2, 3}');
  }
  return { trits: [...trits] };
}

function addrToString(addr: CubeAddr): string {
  return addr.trits.join(',');
}

function addrFromString(s: string): CubeAddr {
  const trits = s.split(',').map(Number);
  return cubeAddr(trits);
}

function computeDelta(src: CubeAddr, dst: CubeAddr): number[] {
  const delta: number[] = [];
  for (let i = 0; i < DIMENSIONS; i++) {
    if (src.trits[i] !== dst.trits[i]) {
      delta.push(i);
    }
  }
  return delta;
}

function hammingDistance(a: CubeAddr, b: CubeAddr): number {
  return computeDelta(a, b).length;
}

function stepToward(src: CubeAddr, dst: CubeAddr, dim: number): CubeAddr {
  const next = { trits: [...src.trits] };
  next.trits[dim] = dst.trits[dim];
  return next;
}

function computeNeighbors(addr: CubeAddr): { addr: CubeAddr; dim: number; alt: number }[] {
  const neighbors: { addr: CubeAddr; dim: number; alt: number }[] = [];
  for (let dim = 0; dim < DIMENSIONS; dim++) {
    const current = addr.trits[dim];
    for (const alt of VALID_DIGITS) {
      if (alt !== current) {
        const nbr = { trits: [...addr.trits] };
        nbr.trits[dim] = alt;
        neighbors.push({ addr: nbr, dim, alt });
      }
    }
  }
  return neighbors;
}

function factorial(n: number): number {
  if (n <= 1) return 1;
  let f = 1;
  for (let i = 2; i <= n; i++) f *= i;
  return f;
}

function hashFlowId(flowId: number): number {
  let hash = 2166136261;
  const bytes = [
    flowId & 0xFF,
    (flowId >> 8) & 0xFF,
    (flowId >> 16) & 0xFF,
    (flowId >> 24) & 0xFF,
  ];
  for (const b of bytes) {
    hash ^= b;
    hash = Math.imul(hash, 16777619) >>> 0;
  }
  return hash;
}

function addrEqual(a: CubeAddr, b: CubeAddr): boolean {
  for (let i = 0; i < DIMENSIONS; i++) {
    if (a.trits[i] !== b.trits[i]) return false;
  }
  return true;
}

function flatIndex(addr: CubeAddr): number {
  let idx = 0;
  let power = 1;
  for (let i = 0; i < DIMENSIONS; i++) {
    idx += (addr.trits[i] - 1) * power;
    power *= 3;
  }
  return idx;
}

function fromFlatIndex(idx: number): CubeAddr {
  const trits: number[] = [];
  let remaining = idx;
  for (let i = 0; i < DIMENSIONS; i++) {
    trits.push((remaining % 3) + 1);
    remaining = Math.floor(remaining / 3);
  }
  return { trits };
}

interface FlowEntry {
  flowHash: number;
  selectedIndex: number;
  lastActive: number;
  expires: number;
}

interface ForwardResult {
  nextHop: CubeAddr;
  dimensionFixed: number;
  totalDistance: number;
  availablePaths: number;
  isDetour: boolean;
}

class GeometricLoadBalancer {
  localCube: CubeAddr;
  activeFlows: Map<number, FlowEntry> = new Map();
  flowTtl: number = DEFAULT_FLOW_TTL_MS;
  deadNeighbors: Set<string> = new Set();
  stats = { totalForwards: 0, detours: 0, flowsExpired: 0 };

  constructor(localCube: CubeAddr) {
    this.localCube = localCube;
  }

  forward(destination: CubeAddr, flowId: number): ForwardResult | { error: string } {
    const delta = computeDelta(this.localCube, destination);
    if (delta.length === 0) {
      return { error: 'already_at_destination' };
    }

    const liveDelta = delta.filter(dim => {
      const candidate = stepToward(this.localCube, destination, dim);
      return !this.deadNeighbors.has(addrToString(candidate));
    });

    if (liveDelta.length === 0) {
      return this.computeDetour(destination, delta);
    }

    const flowHash = hashFlowId(flowId);
    const selectedIndex = flowHash % liveDelta.length;
    const fixDim = liveDelta[selectedIndex];
    const nextHop = stepToward(this.localCube, destination, fixDim);

    const now = Date.now();
    this.activeFlows.set(flowHash, {
      flowHash, selectedIndex,
      lastActive: now, expires: now + this.flowTtl,
    });

    this.stats.totalForwards++;

    return {
      nextHop, dimensionFixed: fixDim,
      totalDistance: delta.length,
      availablePaths: liveDelta.length,
      isDetour: false,
    };
  }

  private computeDetour(destination: CubeAddr, delta: number[]): ForwardResult | { error: string } {
    const deltaSet = new Set(delta);
    for (let dim = 0; dim < DIMENSIONS; dim++) {
      if (deltaSet.has(dim)) continue;
      for (const alt of VALID_DIGITS) {
        if (alt === this.localCube.trits[dim]) continue;
        const candidate = { trits: [...this.localCube.trits] };
        candidate.trits[dim] = alt;
        if (!this.deadNeighbors.has(addrToString(candidate))) {
          this.stats.detours++;
          return {
            nextHop: candidate,
            dimensionFixed: dim,
            totalDistance: delta.length + 2,
            availablePaths: 1,
            isDetour: true,
          };
        }
      }
    }
    return { error: 'isolated' };
  }

  setDeadNeighbors(dead: Set<string>) { this.deadNeighbors = dead; }
  addDead(addr: CubeAddr) { this.deadNeighbors.add(addrToString(addr)); }
  removeDead(addr: CubeAddr) { this.deadNeighbors.delete(addrToString(addr)); }

  expireFlows() {
    const now = Date.now();
    for (const [k, v] of this.activeFlows) {
      if (v.expires < now) {
        this.activeFlows.delete(k);
        this.stats.flowsExpired++;
      }
    }
  }
}

type TunnelState = 'unknown' | 'resolving' | 'connecting' | 'up' | 'down';

interface NeighborRecord {
  addr: CubeAddr;
  dimension: number;
  altValue: number;
  endpoint: string | null;
  publicKey: string | null;
  tunnelIface: string | null;
  state: TunnelState;
  lastHeartbeat: number | null;
  srttNs: number | null;
  bytesIn: number;
  bytesOut: number;
}

class CubeOverlayNetwork {
  localAddr: CubeAddr;
  neighbors: NeighborRecord[] = [];

  constructor(localAddr: CubeAddr) {
    this.localAddr = localAddr;
    const geo = computeNeighbors(localAddr);
    this.neighbors = geo.map((g) => ({
      addr: g.addr, dimension: g.dim, altValue: g.alt,
      endpoint: null, publicKey: null,
      tunnelIface: null, state: 'unknown' as TunnelState,
      lastHeartbeat: null, srttNs: null,
      bytesIn: 0, bytesOut: 0,
    }));
  }

  resolveNeighbor(addr: CubeAddr, endpoint: string, publicKey: string) {
    const nbr = this.neighbors.find(n => addrEqual(n.addr, addr));
    if (nbr) {
      nbr.endpoint = endpoint;
      nbr.publicKey = publicKey;
      nbr.state = 'connecting';
    }
  }

  tunnelUp(addr: CubeAddr, iface: string) {
    const nbr = this.neighbors.find(n => addrEqual(n.addr, addr));
    if (nbr) { nbr.state = 'up'; nbr.tunnelIface = iface; }
  }

  tunnelDown(addr: CubeAddr) {
    const nbr = this.neighbors.find(n => addrEqual(n.addr, addr));
    if (nbr) { nbr.state = 'down'; nbr.tunnelIface = null; }
  }

  recordHeartbeat(addr: CubeAddr, rttNs: number) {
    const nbr = this.neighbors.find(n => addrEqual(n.addr, addr));
    if (nbr) {
      nbr.lastHeartbeat = Date.now();
      nbr.srttNs = nbr.srttNs ? (nbr.srttNs * 7 + rttNs) / 8 : rttNs;
    }
  }

  stats() {
    const counts = { up: 0, down: 0, resolving: 0, connecting: 0, unknown: 0 };
    let totalIn = 0, totalOut = 0, rttSum = 0, rttCount = 0;
    for (const n of this.neighbors) {
      counts[n.state]++;
      totalIn += n.bytesIn;
      totalOut += n.bytesOut;
      if (n.srttNs) { rttSum += n.srttNs / 1e6; rttCount++; }
    }
    return {
      tunnelsUp: counts.up, tunnelsDown: counts.down,
      tunnelsResolving: counts.resolving, tunnelsConnecting: counts.connecting,
      tunnelsUnknown: counts.unknown,
      totalBytesIn: totalIn, totalBytesOut: totalOut,
      avgRttMs: rttCount ? rttSum / rttCount : null,
    };
  }

  static deriveTunnelKey(addrA: CubeAddr, addrB: CubeAddr): string {
    const a = addrToString(addrA);
    const b = addrToString(addrB);
    const [first, second] = a <= b ? [a, b] : [b, a];
    const input = `${first}:${second}:PlenumNET-CON-v1`;
    let hash = 2166136261;
    for (let i = 0; i < input.length; i++) {
      hash ^= input.charCodeAt(i);
      hash = Math.imul(hash, 16777619) >>> 0;
    }
    return hash.toString(16).padStart(8, '0');
  }
}

type CubeStatus = 'active' | 'draining' | 'offline';

interface CubeRecord {
  addr: CubeAddr;
  endpoint: string;
  publicKey: string;
  status: CubeStatus;
  lastHeartbeat: number;
  registeredAt: number;
}

interface RegistrationResult {
  address: CubeAddr;
  neighbors: { addr: CubeAddr; endpoint: string | null; publicKey: string | null; status: CubeStatus | null }[];
}

class CubeRegistrationService {
  private registry: Map<string, CubeRecord> = new Map();
  private usedAddresses: Set<number> = new Set();
  private nextHint = 0;

  register(endpoint: string, publicKey: string, desiredAddress?: CubeAddr): RegistrationResult {
    const now = Date.now();
    let addr: CubeAddr;

    if (desiredAddress) {
      if (!validateRepC(desiredAddress.trits)) throw new Error('Invalid Rep C address');
      const idx = flatIndex(desiredAddress);
      if (this.usedAddresses.has(idx)) throw new Error('Address already in use');
      this.usedAddresses.add(idx);
      addr = desiredAddress;
    } else {
      addr = this.allocateNext();
    }

    const record: CubeRecord = {
      addr, endpoint, publicKey,
      status: 'active', lastHeartbeat: now, registeredAt: now,
    };
    this.registry.set(addrToString(addr), record);

    const neighbors = this.computeNeighborInfo(addr);
    return { address: addr, neighbors };
  }

  private allocateNext(): CubeAddr {
    for (let offset = 0; offset < TOTAL_VERTICES; offset++) {
      const idx = (this.nextHint + offset) % TOTAL_VERTICES;
      if (!this.usedAddresses.has(idx)) {
        this.usedAddresses.add(idx);
        this.nextHint = (idx + 1) % TOTAL_VERTICES;
        return fromFlatIndex(idx);
      }
    }
    throw new Error('Address space exhausted');
  }

  computeNeighborInfo(addr: CubeAddr) {
    return computeNeighbors(addr).map(({ addr: nbrAddr }) => {
      const key = addrToString(nbrAddr);
      const record = this.registry.get(key);
      return {
        addr: nbrAddr,
        endpoint: record?.endpoint ?? null,
        publicKey: record?.publicKey ?? null,
        status: record?.status ?? null,
      };
    });
  }

  lookup(addr: CubeAddr): CubeRecord | undefined {
    return this.registry.get(addrToString(addr));
  }

  heartbeat(addr: CubeAddr, endpoint: string): boolean {
    const record = this.registry.get(addrToString(addr));
    if (!record) return false;
    record.lastHeartbeat = Date.now();
    record.endpoint = endpoint;
    if (record.status === 'offline') record.status = 'active';
    return true;
  }

  deregister(addr: CubeAddr): boolean {
    const key = addrToString(addr);
    if (this.registry.delete(key)) {
      this.usedAddresses.delete(flatIndex(addr));
      return true;
    }
    return false;
  }

  registeredCount() { return this.registry.size; }
  availableAddresses() { return TOTAL_VERTICES - this.usedAddresses.size; }
}

type NeighborState = 'up' | 'suspect' | 'down' | 'recovering';

interface NeighborHealth {
  addr: CubeAddr;
  dimension: number;
  state: NeighborState;
  srttNs: number;
  jitterNs: number;
  consecutiveMisses: number;
  consecutiveSuccesses: number;
  lastPong: number | null;
  suspectSince: number | null;
}

class FaultToleranceService {
  localAddr: CubeAddr;
  neighbors: NeighborHealth[] = [];
  deadSet: Set<string> = new Set();
  config = {
    missThreshold: DEFAULT_MISS_THRESHOLD,
    recoveryThreshold: DEFAULT_RECOVERY_THRESHOLD,
    gracePeriodMs: DEFAULT_GRACE_PERIOD_MS,
  };

  constructor(localAddr: CubeAddr) {
    this.localAddr = localAddr;
    this.neighbors = computeNeighbors(localAddr).map(({ addr, dim }) => ({
      addr, dimension: dim,
      state: 'up' as NeighborState,
      srttNs: 0, jitterNs: 0,
      consecutiveMisses: 0, consecutiveSuccesses: 0,
      lastPong: null, suspectSince: null,
    }));
  }

  recordPong(addr: CubeAddr, rttNs: number) {
    const nbr = this.neighbors.find(n => addrEqual(n.addr, addr));
    if (!nbr) return;

    if (nbr.srttNs === 0) {
      nbr.srttNs = rttNs;
      nbr.jitterNs = rttNs / 2;
    } else {
      const diff = Math.abs(rttNs - nbr.srttNs);
      nbr.jitterNs = (nbr.jitterNs * 3 + diff) / 4;
      nbr.srttNs = (nbr.srttNs * 7 + rttNs) / 8;
    }

    nbr.lastPong = Date.now();
    nbr.consecutiveMisses = 0;
    nbr.consecutiveSuccesses++;

    if (nbr.state === 'suspect' || nbr.state === 'down') {
      nbr.state = 'recovering';
      nbr.suspectSince = null;
      nbr.consecutiveSuccesses = 1;
    } else if (nbr.state === 'recovering' &&
               nbr.consecutiveSuccesses >= this.config.recoveryThreshold) {
      nbr.state = 'up';
    }
    this.rebuildDeadSet();
  }

  recordMiss(addr: CubeAddr) {
    const nbr = this.neighbors.find(n => addrEqual(n.addr, addr));
    if (!nbr) return;
    const now = Date.now();

    nbr.consecutiveMisses++;
    nbr.consecutiveSuccesses = 0;

    if (nbr.state === 'up' && nbr.consecutiveMisses >= this.config.missThreshold) {
      nbr.state = 'suspect';
      nbr.suspectSince = now;
    } else if (nbr.state === 'suspect' && nbr.suspectSince &&
               (now - nbr.suspectSince) >= this.config.gracePeriodMs) {
      nbr.state = 'down';
      nbr.suspectSince = null;
    } else if (nbr.state === 'recovering') {
      nbr.state = 'suspect';
      nbr.suspectSince = now;
    }
    this.rebuildDeadSet();
  }

  private rebuildDeadSet() {
    this.deadSet.clear();
    for (const n of this.neighbors) {
      if (n.state === 'down' || n.state === 'suspect') {
        this.deadSet.add(addrToString(n.addr));
      }
    }
  }

  stateCounts() {
    const c = { up: 0, suspect: 0, down: 0, recovering: 0 };
    for (const n of this.neighbors) c[n.state]++;
    return c;
  }
}

const crs = new CubeRegistrationService();
let currentStack: {
  addr: CubeAddr;
  glb: GeometricLoadBalancer;
  con: CubeOverlayNetwork;
  fts: FaultToleranceService;
} | null = null;

export function registerInterCubeRoutes(app: Router) {
  const router = Router();

  router.post('/crs/register', (req: Request, res: Response) => {
    try {
      const { endpoint, publicKey, desiredAddress } = req.body;
      const desired = desiredAddress ? cubeAddr(desiredAddress) : undefined;
      const result = crs.register(
        endpoint || '0.0.0.0:51820',
        publicKey || 'default-key',
        desired
      );

      currentStack = {
        addr: result.address,
        glb: new GeometricLoadBalancer(result.address),
        con: new CubeOverlayNetwork(result.address),
        fts: new FaultToleranceService(result.address),
      };

      res.json({
        address: result.address.trits,
        addressString: addrToString(result.address),
        neighbors: result.neighbors.map(n => ({
          addr: n.addr.trits,
          endpoint: n.endpoint,
          status: n.status,
        })),
        totalNeighbors: result.neighbors.length,
      });
    } catch (e: any) {
      res.status(400).json({ error: e.message });
    }
  });

  router.get('/crs/lookup/:address', (req: Request, res: Response) => {
    try {
      const addr = addrFromString(req.params.address);
      const record = crs.lookup(addr);
      if (!record) return res.status(404).json({ error: 'not_found' });
      res.json({
        address: record.addr.trits,
        endpoint: record.endpoint,
        status: record.status,
        lastHeartbeat: record.lastHeartbeat,
      });
    } catch (e: any) {
      res.status(400).json({ error: e.message });
    }
  });

  router.get('/crs/neighbors/:address', (req: Request, res: Response) => {
    try {
      const addr = addrFromString(req.params.address);
      const neighbors = crs.computeNeighborInfo(addr);
      res.json({
        address: addr.trits,
        neighbors: neighbors.map(n => ({
          addr: n.addr.trits,
          endpoint: n.endpoint,
          status: n.status,
        })),
        count: neighbors.length,
      });
    } catch (e: any) {
      res.status(400).json({ error: e.message });
    }
  });

  router.post('/crs/heartbeat', (req: Request, res: Response) => {
    try {
      const { address, endpoint } = req.body;
      const addr = cubeAddr(address);
      const ok = crs.heartbeat(addr, endpoint);
      res.json({ ack: ok, nextHeartbeatMs: 30000 });
    } catch (e: any) {
      res.status(400).json({ error: e.message });
    }
  });

  router.post('/crs/deregister', (req: Request, res: Response) => {
    try {
      const addr = cubeAddr(req.body.address);
      const ok = crs.deregister(addr);
      res.json({ ack: ok, gracePeriodS: 86400 });
    } catch (e: any) {
      res.status(400).json({ error: e.message });
    }
  });

  router.get('/crs/stats', (_req: Request, res: Response) => {
    res.json({
      registeredCubes: crs.registeredCount(),
      availableAddresses: crs.availableAddresses(),
      totalAddressSpace: TOTAL_VERTICES,
      dimensions: DIMENSIONS,
    });
  });

  router.post('/glb/forward', (req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    try {
      const { destination, flowId } = req.body;
      const dst = cubeAddr(destination);
      const result = currentStack.glb.forward(dst, flowId || 0);
      if ('error' in result) {
        return res.status(400).json(result);
      }
      res.json({
        nextHop: result.nextHop.trits,
        dimensionFixed: result.dimensionFixed,
        totalDistance: result.totalDistance,
        availablePaths: result.availablePaths,
        isDetour: result.isDetour,
        shortestPathCount: factorial(result.totalDistance),
      });
    } catch (e: any) {
      res.status(400).json({ error: e.message });
    }
  });

  router.get('/glb/stats', (_req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    const glb = currentStack.glb;
    res.json({
      activeFlows: glb.activeFlows.size,
      deadNeighbors: glb.deadNeighbors.size,
      liveNeighbors: NEIGHBORS_PER_CUBE - glb.deadNeighbors.size,
      ...glb.stats,
    });
  });

  router.get('/glb/health', (_req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    const glb = currentStack.glb;
    res.json({
      deadNeighbors: [...glb.deadNeighbors],
      liveNeighborCount: NEIGHBORS_PER_CUBE - glb.deadNeighbors.size,
    });
  });

  router.get('/con/neighbors', (_req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    res.json(currentStack.con.neighbors.map(n => ({
      addr: n.addr.trits,
      dimension: n.dimension,
      state: n.state,
      endpoint: n.endpoint,
      rttMs: n.srttNs ? n.srttNs / 1e6 : null,
    })));
  });

  router.get('/con/stats', (_req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    res.json(currentStack.con.stats());
  });

  router.post('/con/tunnel/refresh', (_req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    for (const nbr of currentStack.con.neighbors) {
      if (nbr.state !== 'up') nbr.state = 'resolving';
    }
    res.json({ ack: true, tunnelsRefreshed: currentStack.con.neighbors.length });
  });

  router.get('/fts/status', (_req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    res.json(currentStack.fts.neighbors.map(n => ({
      addr: n.addr.trits,
      dimension: n.dimension,
      state: n.state,
      srttMs: n.srttNs / 1e6,
      lastSeenAgoMs: n.lastPong ? Date.now() - n.lastPong : null,
    })));
  });

  router.get('/fts/dead', (_req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    res.json({
      dead: [...currentStack.fts.deadSet],
      count: currentStack.fts.deadSet.size,
    });
  });

  router.post('/fts/config', (req: Request, res: Response) => {
    if (!currentStack) return res.status(503).json({ error: 'not_initialized' });
    const { missThreshold, recoveryThreshold, gracePeriodMs } = req.body;
    const config = currentStack.fts.config;
    if (missThreshold) config.missThreshold = missThreshold;
    if (recoveryThreshold) config.recoveryThreshold = recoveryThreshold;
    if (gracePeriodMs) config.gracePeriodMs = gracePeriodMs;
    res.json({ ack: true, config });
  });

  router.post('/routing/compute', (req: Request, res: Response) => {
    try {
      const { source, destination } = req.body;
      const src = cubeAddr(source);
      const dst = cubeAddr(destination);
      const delta = computeDelta(src, dst);
      const distance = delta.length;
      const pathCount = factorial(distance);

      res.json({
        source: src.trits,
        destination: dst.trits,
        hammingDistance: distance,
        differingDimensions: delta,
        shortestPathCount: pathCount,
        maxHops: distance,
        routingAlgorithm: 'pure_math_hamming',
        routingTables: 'none',
      });
    } catch (e: any) {
      res.status(400).json({ error: e.message });
    }
  });

  router.post('/address/validate', (req: Request, res: Response) => {
    const { trits } = req.body;
    const valid = validateRepC(trits || []);
    const hasZero = (trits || []).some((t: number) => t === 0);
    res.json({
      valid,
      representation: 'C',
      digits: '{1, 2, 3}',
      zeroDetected: hasZero,
      forgeryIndicator: hasZero ? 'FORGERY: zero present in Rep C address' : null,
    });
  });

  router.get('/topology', (_req: Request, res: Response) => {
    res.json({
      dimensions: DIMENSIONS,
      vertices: TOTAL_VERTICES,
      neighborsPerCube: NEIGHBORS_PER_CUBE,
      maxHammingDistance: DIMENSIONS,
      addressRepresentation: 'Rep C (bijective: {1, 2, 3})',
      routingAlgorithm: 'Pure math Hamming — no routing tables',
      scalingLevels: [
        { level: 1, trits: 13, nodes: '1,594,323', scale: 'Campus network' },
        { level: 2, trits: 26, nodes: '2.54 trillion', scale: 'Every device on Earth × 300' },
        { level: 3, trits: 39, nodes: '4.05 quintillion', scale: 'More than grains of sand' },
      ],
      services: {
        GLB: 'Geometric Load Balancer — d! shortest paths via dimension ordering',
        CON: 'Cube Overlay Network — 26 encrypted tunnels to geometric neighbors',
        CRS: 'Cube Registration Service — address allocation + endpoint registry',
        FTS: 'Fault Tolerance Service — heartbeat monitoring + dead neighbor set',
      },
    });
  });

  app.use('/api/salvi/inter-cube', router);

  return router;
}
