/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { db } from "../db";
import { threatModelEntries } from "@shared/schema";
import { eq, desc, and, count } from "drizzle-orm";
import { createLogger } from "../logger";

const log = createLogger("threat-model");

const THREAT_CATEGORIES = [
  "timing_infrastructure",
  "cryptographic_modules",
  "network_protocol",
  "physical_security",
  "supply_chain",
  "side_channel",
  "insider_threat",
  "quantum_attack",
  "software_vulnerability",
  "compliance_gap",
] as const;

const ADVERSARY_TYPES = [
  "nation_state",
  "advanced_persistent_threat",
  "quantum_capable",
  "insider_privileged",
  "insider_unprivileged",
  "supply_chain_actor",
  "opportunistic",
  "organized_crime",
] as const;

const RISK_LEVELS = ["negligible", "low", "medium", "high", "critical"] as const;
const SCOPES = ["in_scope", "out_of_scope", "deferred"] as const;

export const threatModelService = {
  async create(params: {
    category: string;
    threatVector: string;
    scope: string;
    adversaryType: string;
    currentMitigation: string;
    residualRisk: string;
    redundancyFallback?: string;
    detectionMechanism?: string;
    cvssScore?: number;
    status: string;
  }) {
    const [entry] = await db
      .insert(threatModelEntries)
      .values({
        category: params.category,
        threatVector: params.threatVector,
        scope: params.scope,
        adversaryType: params.adversaryType,
        currentMitigation: params.currentMitigation,
        residualRisk: params.residualRisk,
        redundancyFallback: params.redundancyFallback || null,
        detectionMechanism: params.detectionMechanism || null,
        cvssScore: params.cvssScore ?? null,
        status: params.status,
      })
      .returning();

    log.info("Threat model entry created", { id: entry.id, category: params.category, vector: params.threatVector });
    return entry;
  },

  async update(id: number, params: Partial<{
    category: string;
    threatVector: string;
    scope: string;
    adversaryType: string;
    currentMitigation: string;
    residualRisk: string;
    redundancyFallback: string | null;
    detectionMechanism: string | null;
    cvssScore: number | null;
    status: string;
  }>) {
    const [updated] = await db
      .update(threatModelEntries)
      .set({ ...params, updatedAt: new Date(), lastReviewedAt: new Date() })
      .where(eq(threatModelEntries.id, id))
      .returning();
    return updated;
  },

  async getAll(filters?: {
    category?: string;
    scope?: string;
    adversaryType?: string;
    residualRisk?: string;
    status?: string;
  }) {
    const conditions = [];
    if (filters?.category) conditions.push(eq(threatModelEntries.category, filters.category));
    if (filters?.scope) conditions.push(eq(threatModelEntries.scope, filters.scope));
    if (filters?.adversaryType) conditions.push(eq(threatModelEntries.adversaryType, filters.adversaryType));
    if (filters?.residualRisk) conditions.push(eq(threatModelEntries.residualRisk, filters.residualRisk));
    if (filters?.status) conditions.push(eq(threatModelEntries.status, filters.status));

    const query = conditions.length > 0
      ? db.select().from(threatModelEntries).where(and(...conditions))
      : db.select().from(threatModelEntries);

    return query.orderBy(desc(threatModelEntries.createdAt));
  },

  async getById(id: number) {
    const [entry] = await db
      .select()
      .from(threatModelEntries)
      .where(eq(threatModelEntries.id, id));
    return entry;
  },

  async delete(id: number) {
    const [deleted] = await db
      .delete(threatModelEntries)
      .where(eq(threatModelEntries.id, id))
      .returning();
    return deleted;
  },

  async getRiskMatrix() {
    const entries = await db.select().from(threatModelEntries);
    const matrix: Record<string, Record<string, number>> = {};
    for (const cat of THREAT_CATEGORIES) {
      matrix[cat] = {};
      for (const risk of RISK_LEVELS) {
        matrix[cat][risk] = 0;
      }
    }
    for (const entry of entries) {
      if (matrix[entry.category] && entry.residualRisk in matrix[entry.category]) {
        matrix[entry.category][entry.residualRisk]++;
      }
    }
    return matrix;
  },

  async getSummaryStats() {
    const categoryCounts = await db
      .select({ category: threatModelEntries.category, count: count() })
      .from(threatModelEntries)
      .groupBy(threatModelEntries.category);

    const scopeCounts = await db
      .select({ scope: threatModelEntries.scope, count: count() })
      .from(threatModelEntries)
      .groupBy(threatModelEntries.scope);

    const riskCounts = await db
      .select({ risk: threatModelEntries.residualRisk, count: count() })
      .from(threatModelEntries)
      .groupBy(threatModelEntries.residualRisk);

    return {
      byCategory: Object.fromEntries(categoryCounts.map(r => [r.category, r.count])),
      byScope: Object.fromEntries(scopeCounts.map(r => [r.scope, r.count])),
      byRisk: Object.fromEntries(riskCounts.map(r => [r.risk, r.count])),
    };
  },

  async seedDefaults() {
    const existing = await db.select({ count: count() }).from(threatModelEntries);
    if (existing[0].count > 0) {
      log.info("Threat model already seeded, skipping");
      return { seeded: false, count: existing[0].count };
    }

    const defaults = [
      {
        category: "timing_infrastructure",
        threatVector: "GPS spoofing of HPTP reference clocks",
        scope: "in_scope",
        adversaryType: "nation_state",
        currentMitigation: "Multi-source cross-validation with cesium backup; anomaly detection on jitter/drift",
        residualRisk: "low",
        redundancyFallback: "5-tier HPTP fallback chain (PTP→NTP→crystal→quartz→cesium)",
        detectionMechanism: "Real-time jitter/drift anomaly detector with configurable thresholds",
        cvssScore: 7.5,
        status: "mitigated",
      },
      {
        category: "timing_infrastructure",
        threatVector: "Delay-box attack on network timing packets",
        scope: "in_scope",
        adversaryType: "advanced_persistent_threat",
        currentMitigation: "Authenticated timing with symmetric key; bounded staleness checks",
        residualRisk: "medium",
        redundancyFallback: "Local holdover mode with crystal oscillator",
        detectionMechanism: "Staleness window violation alerts",
        cvssScore: 6.8,
        status: "mitigated",
      },
      {
        category: "cryptographic_modules",
        threatVector: "Quantum factoring attack on RSA/ECC key exchange",
        scope: "in_scope",
        adversaryType: "quantum_capable",
        currentMitigation: "TL-KEM lattice-based key encapsulation; CNSA 2.0 compliant algorithms",
        residualRisk: "low",
        redundancyFallback: "Hybrid classical+post-quantum key exchange",
        detectionMechanism: "Key entropy monitoring; algorithm compliance checker",
        cvssScore: 9.0,
        status: "mitigated",
      },
      {
        category: "cryptographic_modules",
        threatVector: "Side-channel leakage from AES-256-GCM implementation",
        scope: "in_scope",
        adversaryType: "advanced_persistent_threat",
        currentMitigation: "Constant-time implementations; GF(3) masking",
        residualRisk: "medium",
        redundancyFallback: "Software-only fallback with additional masking rounds",
        detectionMechanism: "Power analysis anomaly detection in FPGA paths",
        cvssScore: 5.9,
        status: "monitoring",
      },
      {
        category: "network_protocol",
        threatVector: "Routing table poisoning in Torsion Network",
        scope: "in_scope",
        adversaryType: "nation_state",
        currentMitigation: "Authenticated TTP routing updates; greedy geodesic path verification",
        residualRisk: "low",
        redundancyFallback: "Static route fallback with pre-shared topology",
        detectionMechanism: "Route convergence time monitoring; topology hash verification",
        cvssScore: 7.2,
        status: "mitigated",
      },
      {
        category: "physical_security",
        threatVector: "Electromagnetic fault injection on FPGA",
        scope: "in_scope",
        adversaryType: "nation_state",
        currentMitigation: "Sensor mesh with voltage/temperature glitch detectors; auto-zeroization",
        residualRisk: "medium",
        redundancyFallback: "Triple modular redundancy on critical paths",
        detectionMechanism: "Glitch detector interrupt with audit trail",
        cvssScore: 6.5,
        status: "in_progress",
      },
      {
        category: "supply_chain",
        threatVector: "Compromised FPGA bitstream or silicon trojan",
        scope: "in_scope",
        adversaryType: "supply_chain_actor",
        currentMitigation: "Bitstream signature verification; SBOM tracking; trusted foundry sourcing",
        residualRisk: "high",
        redundancyFallback: "Dual-vendor FPGA strategy with cross-validation",
        detectionMechanism: "Post-synthesis LUT count verification against golden reference",
        cvssScore: 8.1,
        status: "monitoring",
      },
      {
        category: "side_channel",
        threatVector: "Timing side-channel in ternary arithmetic operations",
        scope: "in_scope",
        adversaryType: "advanced_persistent_threat",
        currentMitigation: "Constant-time GF(3) operations; balanced ternary conversion shielding",
        residualRisk: "low",
        redundancyFallback: "Randomized operation scheduling",
        detectionMechanism: "Execution time variance monitoring",
        cvssScore: 4.3,
        status: "mitigated",
      },
      {
        category: "quantum_attack",
        threatVector: "Grover's algorithm acceleration of brute-force on symmetric keys",
        scope: "in_scope",
        adversaryType: "quantum_capable",
        currentMitigation: "256-bit minimum key sizes; AES-256-GCM standard",
        residualRisk: "negligible",
        redundancyFallback: "Key size upgrade path to 384-bit",
        detectionMechanism: "CNSA 2.0 compliance validation on all key generation",
        cvssScore: 3.1,
        status: "mitigated",
      },
      {
        category: "insider_threat",
        threatVector: "Privileged admin key exfiltration",
        scope: "in_scope",
        adversaryType: "insider_privileged",
        currentMitigation: "SHA-256 hashed key storage; audit trail on all key operations; anomaly detection",
        residualRisk: "medium",
        redundancyFallback: "Automatic key rotation (6-hour cron); dual-key grace periods",
        detectionMechanism: "IP dispersion alerts (>10 IPs/24h); usage spike detection (>300% DoD)",
        cvssScore: 6.0,
        status: "mitigated",
      },
      {
        category: "compliance_gap",
        threatVector: "FIPS 140-3 module boundary not formally validated",
        scope: "in_scope",
        adversaryType: "opportunistic",
        currentMitigation: "Self-assessment checklist; CMVP boundary diagram; CNSA 2.0 algorithm set",
        residualRisk: "high",
        redundancyFallback: "Pre-validation review with accredited lab",
        detectionMechanism: "14-item compliance checklist automated scoring",
        cvssScore: 4.0,
        status: "in_progress",
      },
      {
        category: "software_vulnerability",
        threatVector: "Path traversal in API file operations",
        scope: "in_scope",
        adversaryType: "opportunistic",
        currentMitigation: "Null-byte stripping; double URL-decode protection; execFile()-only subprocess execution; hardened path sanitization",
        residualRisk: "low",
        redundancyFallback: "WAF layer with path normalization",
        detectionMechanism: "Request logging with pattern matching on traversal attempts",
        cvssScore: 5.3,
        status: "mitigated",
      },
    ];

    for (const entry of defaults) {
      await db.insert(threatModelEntries).values(entry);
    }
    log.info("Threat model seeded with default entries", { count: defaults.length });
    return { seeded: true, count: defaults.length };
  },

  getCategories() {
    return [...THREAT_CATEGORIES];
  },
  getAdversaryTypes() {
    return [...ADVERSARY_TYPES];
  },
  getRiskLevels() {
    return [...RISK_LEVELS];
  },
  getScopes() {
    return [...SCOPES];
  },
};
