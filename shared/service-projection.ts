/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * TypeScript mirror of plenumlan/src/cube/projection.rs + port.rs + bridge.rs.
 * MUST produce byte-identical output for all 27-trit inputs.
 *
 * === SINGLE SOURCE OF TRUTH ===
 * If the Rust implementation changes, this file MUST be updated to match.
 * Cross-language test vectors (T006) verify identity.
 */

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS (imported by value from plenumlan/src/cube/constants.rs)
// ═══════════════════════════════════════════════════════════════════════

export const GF3_ORDER = 3;
export const CLASSIFICATION_DIMS = 27;
export const CUBE_DIMS = 3;
export const SLOTS_PER_NODE = GF3_ORDER ** CUBE_DIMS;        // 27
export const CLUSTER_SLOTS = SLOTS_PER_NODE * GF3_ORDER;     // 81
export const SLOTS_PER_PLANE = GF3_ORDER * GF3_ORDER;        // 9
export const DIMS_PER_GROUP = CLASSIFICATION_DIMS / CUBE_DIMS; // 9
export const MAX_NODES = GF3_ORDER;                           // 3
export const GATEWAY_NODE_ID = 1;
export const GATEWAY_OFFSET = 13;                             // = DIMENSIONS = T₇
export const DECIMAL_REPUNIT_DIGITS = 5;
export const BASE_PORT = ((10 ** DECIMAL_REPUNIT_DIGITS - 1) / 9); // 11111
export const REP_C_CENTER = 2;

// Verify at load time
if (BASE_PORT !== 11111) throw new Error(`BASE_PORT mismatch: ${BASE_PORT}`);
if (SLOTS_PER_NODE !== 27) throw new Error(`SLOTS_PER_NODE mismatch: ${SLOTS_PER_NODE}`);
if (CLUSTER_SLOTS !== 81) throw new Error(`CLUSTER_SLOTS mismatch: ${CLUSTER_SLOTS}`);

// ═══════════════════════════════════════════════════════════════════════
// SLOT ADDRESS
// ═══════════════════════════════════════════════════════════════════════

export interface SlotAddress {
  plane: number;    // Rep C: 1=Data, 2=Control, 3=Management
  role: number;     // Rep C: 1=Primary, 2=Secondary, 3=Tertiary
  instance: number; // Rep C: 1, 2, or 3
}

export function slotToOffset(slot: SlotAddress): number {
  return (slot.plane - 1) * 9 + (slot.role - 1) * 3 + (slot.instance - 1);
}

// ═══════════════════════════════════════════════════════════════════════
// GF(3) PROJECTION (mirrors ternary_math::gf3_algebra::project_to_gf3)
// ═══════════════════════════════════════════════════════════════════════

function projectToGf3(k: number, n: number): number {
  return Math.min(Math.floor((GF3_ORDER * k) / n), GF3_ORDER - 1);
}

// ═══════════════════════════════════════════════════════════════════════
// POLARITY TABLES
// ═══════════════════════════════════════════════════════════════════════

type Polarity = '+' | '-';
type DimEntry = [number, Polarity]; // [1-based dim, polarity]

const PLANE_DIMS: DimEntry[] = [
  [ 1, '+'],  // D1  EntityKind
  [ 2, '-'],  // D2  OperatorScale (inverted)
  [ 3, '+'],  // D3  OperatorTransparency
  [ 9, '-'],  // D9  Encryption (inverted)
  [10, '+'],  // D10 AuthNMethod
  [17, '+'],  // D17 Jurisdiction
  [19, '+'],  // D19 PolicyPresence
  [25, '+'],  // D25 AuditPosture
  [26, '+'],  // D26 TrackerCount
];

const ROLE_DIMS: DimEntry[] = [
  [ 5, '+'],  // D5  Interactivity
  [ 6, '+'],  // D6  MediaRichness
  [ 7, '+'],  // D7  DataPersistence
  [ 8, '+'],  // D8  Intelligence
  [12, '+'],  // D12 APIPresence
  [18, '+'],  // D18 DataAppetite
  [22, '-'],  // D22 Monetization (inverted)
  [23, '+'],  // D23 UpdateCadence
  [24, '-'],  // D24 Availability (inverted)
];

const INSTANCE_DIMS: DimEntry[] = [
  [ 4, '+'],  // D4  LifespanIntent
  [11, '+'],  // D11 ProtocolComplexity
  [13, '+'],  // D13 ContentVolatility
  [14, '+'],  // D14 UserBase
  [15, '+'],  // D15 ProtocolLayering
  [16, '+'],  // D16 Freshness
  [20, '+'],  // D20 CostModel
  [21, '+'],  // D21 GeographicReach
  [27, '+'],  // D27 Confidence
];

function invertTrit(t: number): number {
  if (t === 1) return 3;
  if (t === 3) return 1;
  return 2; // center stays center
}

function countHigh(classification: number[], group: DimEntry[]): number {
  let k = 0;
  for (const [dim1based, polarity] of group) {
    const raw = classification[dim1based - 1];
    const adjusted = polarity === '-' ? invertTrit(raw) : raw;
    if (adjusted === 3) k++;
  }
  return k;
}

// ═══════════════════════════════════════════════════════════════════════
// PROJECT TO SLOT (27 → 3 projection)
// ═══════════════════════════════════════════════════════════════════════

/**
 * Project 27 classification trits (Rep C {1,2,3}) to a 3-trit slot address.
 * Source-agnostic: works with any classification method.
 * Returns null if any input trit is outside {1,2,3} (zero-sentinel violation).
 */
export function projectToSlot(classification: number[]): SlotAddress | null {
  if (classification.length !== CLASSIFICATION_DIMS) return null;
  for (const t of classification) {
    if (t < 1 || t > 3) return null;
  }

  const planeK = countHigh(classification, PLANE_DIMS);
  const roleK = countHigh(classification, ROLE_DIMS);
  const instK = countHigh(classification, INSTANCE_DIMS);

  const n = DIMS_PER_GROUP;
  const plane = projectToGf3(planeK, n) + 1;    // lift GF(3) to Rep C
  const role = projectToGf3(roleK, n) + 1;
  const instance = projectToGf3(instK, n) + 1;

  return { plane, role, instance };
}

// ═══════════════════════════════════════════════════════════════════════
// PORT FORMULA
// ═══════════════════════════════════════════════════════════════════════

/**
 * Compute TCP port for a slot on a given node.
 * node_id: Rep C {1,2,3}. slot: 3-trit Rep C.
 * Returns null if node_id is invalid.
 */
export function slotPort(nodeId: number, slot: SlotAddress): number | null {
  if (nodeId < 1 || nodeId > MAX_NODES) return null;
  return BASE_PORT
    + (nodeId - 1) * SLOTS_PER_NODE
    + (slot.plane - 1) * SLOTS_PER_PLANE
    + (slot.role - 1) * GF3_ORDER
    + (slot.instance - 1);
}

/**
 * Gateway port for a node (slot center = offset 13).
 */
export function gatewayPort(nodeId: number): number | null {
  if (nodeId < 1 || nodeId > MAX_NODES) return null;
  return BASE_PORT + (nodeId - 1) * SLOTS_PER_NODE + GATEWAY_OFFSET;
}

/**
 * Port range [start, end] (inclusive) for a node.
 */
export function nodePortRange(nodeId: number): [number, number] | null {
  if (nodeId < 1 || nodeId > MAX_NODES) return null;
  const start = BASE_PORT + (nodeId - 1) * SLOTS_PER_NODE;
  return [start, start + SLOTS_PER_NODE - 1];
}

/**
 * Decode a port back to (nodeId, SlotAddress).
 */
export function portToSlot(port: number): { nodeId: number; slot: SlotAddress } | null {
  if (port < BASE_PORT) return null;
  const offset = port - BASE_PORT;
  if (offset >= SLOTS_PER_NODE * MAX_NODES) return null;

  const nodeIndex = Math.floor(offset / SLOTS_PER_NODE);
  const slotOffset = offset % SLOTS_PER_NODE;
  const planeGf3 = Math.floor(slotOffset / SLOTS_PER_PLANE);
  const remainder = slotOffset % SLOTS_PER_PLANE;
  const roleGf3 = Math.floor(remainder / GF3_ORDER);
  const instanceGf3 = remainder % GF3_ORDER;

  return {
    nodeId: nodeIndex + 1,
    slot: {
      plane: planeGf3 + 1,
      role: roleGf3 + 1,
      instance: instanceGf3 + 1,
    },
  };
}

// ═══════════════════════════════════════════════════════════════════════
// LEGACY BRIDGE
// ═══════════════════════════════════════════════════════════════════════

export interface LegacyBridge {
  protocol: 'DNS' | 'DHCP' | 'SMB' | 'IPP' | 'RADIUS';
  port: number;
}

/**
 * Derive legacy bridge port from 27 classification trits.
 * Matches D5, D6, D12, D15 patterns.
 */
export function deriveLegacyBridge(classification: number[]): LegacyBridge | null {
  if (classification.length !== CLASSIFICATION_DIMS) return null;
  const d5 = classification[4];
  const d6 = classification[5];
  const d12 = classification[11];
  const d15 = classification[14];

  if (d5 === 3 && d6 === 1 && d12 === 3 && d15 === 2) return { protocol: 'DNS', port: 53 };
  if (d5 === 3 && d6 === 1 && d12 === 3 && d15 === 1) return { protocol: 'DHCP', port: 67 };
  if (d5 === 2 && d6 === 1 && d12 === 1 && d15 === 2) return { protocol: 'SMB', port: 445 };
  if (d5 === 2 && d6 === 2 && d12 === 1)              return { protocol: 'IPP', port: 631 };
  if (d5 === 3 && d6 === 1 && d12 === 3 && d15 === 3) return { protocol: 'RADIUS', port: 1812 };

  return null;
}

// ═══════════════════════════════════════════════════════════════════════
// HAMMING DISTANCE & ROUTING
// ═══════════════════════════════════════════════════════════════════════

export type AuthLevel = 'loopback' | 'direct' | 'capability-token' | 'full-mutual-auth';

export function slotHammingDistance(a: SlotAddress, b: SlotAddress): number {
  let hd = 0;
  if (a.plane !== b.plane) hd++;
  if (a.role !== b.role) hd++;
  if (a.instance !== b.instance) hd++;
  return hd;
}

export function requiredAuthLevel(src: SlotAddress, dst: SlotAddress): AuthLevel {
  const hd = slotHammingDistance(src, dst);
  switch (hd) {
    case 0: return 'loopback';
    case 1: return 'direct';
    case 2: return 'capability-token';
    case 3: return 'full-mutual-auth';
    default: throw new Error(`impossible HD: ${hd}`);
  }
}
