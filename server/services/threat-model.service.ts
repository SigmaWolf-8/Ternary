/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { db } from "../db";
import { threatModelEntries } from "@shared/schema";
import { eq, desc, and, count, gte } from "drizzle-orm";
import { createLogger } from "../logger";
import { phaseEncryptFields, phaseDecryptFields } from "../storage";

const log = createLogger("threat-model");

const THREAT_CATEGORIES = [
  "timing",
  "crypto",
  "network",
  "physical",
  "supply_chain",
  "side_channel",
  "quantum",
  "insider",
  "compliance",
  "software",
] as const;

const LIKELIHOOD_LEVELS = ["low", "medium", "high", "critical"] as const;
const IMPACT_LEVELS = ["low", "medium", "high", "critical"] as const;
const MITIGATION_STATUSES = ["mitigated", "in_progress", "acknowledged", "not_addressed"] as const;

const LEVEL_MAP: Record<string, number> = { low: 1, medium: 2, high: 3, critical: 4 };

function calculateRiskScore(likelihood: string, impact: string): number {
  const l = LEVEL_MAP[likelihood] ?? 1;
  const i = LEVEL_MAP[impact] ?? 1;
  return parseFloat(((l * i) / 1.6).toFixed(1));
}

export const threatModelService = {
  async create(params: {
    threatId: string;
    threatName: string;
    description?: string;
    category: string;
    attackVector?: string;
    likelihood: string;
    impact: string;
    mitigationStatus: string;
    controls?: Array<{ controlId: string; controlName: string; status: string; evidence?: string }>;
    residualRisk?: number;
    notes?: string;
    createdBy?: string;
  }) {
    const riskScore = calculateRiskScore(params.likelihood, params.impact);

    const encryptedFields = phaseEncryptFields({
      description: params.description || null,
      controls: params.controls || null,
      notes: params.notes || null,
      attackVector: params.attackVector || null,
    });
    const [entry] = await db
      .insert(threatModelEntries)
      .values({
        threatId: params.threatId,
        threatName: params.threatName,
        description: params.description || null,
        category: params.category,
        attackVector: params.attackVector || null,
        likelihood: params.likelihood,
        impact: params.impact,
        riskScore,
        mitigationStatus: params.mitigationStatus,
        controls: params.controls || null,
        residualRisk: params.residualRisk ?? null,
        notes: params.notes || null,
        createdBy: params.createdBy || null,
        encryptedFields,
      })
      .returning();

    log.info("Threat model entry created", { id: entry.id, threatId: params.threatId, category: params.category });
    return entry;
  },

  async update(id: number, params: Partial<{
    threatName: string;
    description: string;
    category: string;
    attackVector: string | null;
    likelihood: string;
    impact: string;
    mitigationStatus: string;
    controls: Array<{ controlId: string; controlName: string; status: string; evidence?: string }> | null;
    residualRisk: number | null;
    notes: string | null;
    updatedBy: string;
  }>) {
    const updateData: Record<string, unknown> = { ...params, updatedAt: new Date() };
    const current = await this.getById(id);

    if (current && (params.likelihood || params.impact)) {
      const likelihood = params.likelihood || current.likelihood;
      const impact = params.impact || current.impact;
      updateData.riskScore = calculateRiskScore(likelihood, impact);
    }

    if (current) {
      updateData.encryptedFields = phaseEncryptFields({
        description: params.description ?? current.description,
        controls: params.controls ?? current.controls,
        notes: params.notes ?? current.notes,
        attackVector: params.attackVector ?? current.attackVector,
      });
    }

    const [updated] = await db
      .update(threatModelEntries)
      .set(updateData)
      .where(eq(threatModelEntries.id, id))
      .returning();
    return updated;
  },

  async getAll(filters?: {
    category?: string;
    mitigationStatus?: string;
    likelihood?: string;
    impact?: string;
  }) {
    const conditions = [];
    if (filters?.category) conditions.push(eq(threatModelEntries.category, filters.category));
    if (filters?.mitigationStatus) conditions.push(eq(threatModelEntries.mitigationStatus, filters.mitigationStatus));
    if (filters?.likelihood) conditions.push(eq(threatModelEntries.likelihood, filters.likelihood));
    if (filters?.impact) conditions.push(eq(threatModelEntries.impact, filters.impact));

    const query = conditions.length > 0
      ? db.select().from(threatModelEntries).where(and(...conditions))
      : db.select().from(threatModelEntries);

    const rows = await query.orderBy(desc(threatModelEntries.createdAt));
    return rows.map(row => {
      const dec = phaseDecryptFields(row.encryptedFields);
      if (dec) {
        if (dec.description) row.description = dec.description as string;
        if (dec.controls) row.controls = dec.controls as any;
        if (dec.notes) row.notes = dec.notes as string;
        if (dec.attackVector) row.attackVector = dec.attackVector as string;
      }
      return row;
    });
  },

  async getById(id: number) {
    const [entry] = await db
      .select()
      .from(threatModelEntries)
      .where(eq(threatModelEntries.id, id));
    if (entry) {
      const dec = phaseDecryptFields(entry.encryptedFields);
      if (dec) {
        if (dec.description) entry.description = dec.description as string;
        if (dec.controls) entry.controls = dec.controls as any;
        if (dec.notes) entry.notes = dec.notes as string;
        if (dec.attackVector) entry.attackVector = dec.attackVector as string;
      }
    }
    return entry;
  },

  async getByThreatId(threatId: string) {
    const [entry] = await db
      .select()
      .from(threatModelEntries)
      .where(eq(threatModelEntries.threatId, threatId));
    if (entry) {
      const dec = phaseDecryptFields(entry.encryptedFields);
      if (dec) {
        if (dec.description) entry.description = dec.description as string;
        if (dec.controls) entry.controls = dec.controls as any;
        if (dec.notes) entry.notes = dec.notes as string;
        if (dec.attackVector) entry.attackVector = dec.attackVector as string;
      }
    }
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

    const byCategory: Record<string, Array<{ threatId: string; threatName: string; riskScore: number; mitigationStatus: string }>> = {};
    for (const cat of THREAT_CATEGORIES) {
      byCategory[cat] = [];
    }
    for (const entry of entries) {
      if (!byCategory[entry.category]) byCategory[entry.category] = [];
      byCategory[entry.category].push({
        threatId: entry.threatId,
        threatName: entry.threatName,
        riskScore: entry.riskScore,
        mitigationStatus: entry.mitigationStatus,
      });
    }

    const byStatus: Record<string, number> = {};
    for (const status of MITIGATION_STATUSES) {
      byStatus[status] = 0;
    }
    for (const entry of entries) {
      byStatus[entry.mitigationStatus] = (byStatus[entry.mitigationStatus] || 0) + 1;
    }

    const highRiskCount = entries.filter(e => e.riskScore >= 6.0).length;

    return {
      by_category: byCategory,
      by_status: byStatus,
      high_risk_count: highRiskCount,
    };
  },

  async getSummaryStats() {
    const categoryCounts = await db
      .select({ category: threatModelEntries.category, count: count() })
      .from(threatModelEntries)
      .groupBy(threatModelEntries.category);

    const mitigationStatusCounts = await db
      .select({ mitigationStatus: threatModelEntries.mitigationStatus, count: count() })
      .from(threatModelEntries)
      .groupBy(threatModelEntries.mitigationStatus);

    return {
      byCategory: Object.fromEntries(categoryCounts.map(r => [r.category, r.count])),
      byMitigationStatus: Object.fromEntries(mitigationStatusCounts.map(r => [r.mitigationStatus, r.count])),
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
        threatId: "THREAT_001",
        threatName: "GPS Spoofing of HPTP Reference Clocks",
        description: "GPS spoofing of HPTP reference clocks targeting timing infrastructure",
        category: "timing",
        attackVector: "GPS spoofing of reference clock signals",
        likelihood: "medium",
        impact: "high",
        riskScore: calculateRiskScore("medium", "high"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_001", controlName: "Multi-source cross-validation", status: "implemented", evidence: "Cesium backup with anomaly detection on jitter/drift" },
          { controlId: "CTRL_002", controlName: "5-tier HPTP fallback chain", status: "implemented", evidence: "PTP→NTP→crystal→quartz→cesium fallback" },
        ],
        residualRisk: 2.5,
        notes: "Real-time jitter/drift anomaly detector with configurable thresholds",
        createdBy: "system",
      },
      {
        threatId: "THREAT_002",
        threatName: "Delay-Box Attack on Network Timing Packets",
        description: "Delay-box attack inserting latency into network timing packets",
        category: "timing",
        attackVector: "Network delay injection on timing packets",
        likelihood: "medium",
        impact: "medium",
        riskScore: calculateRiskScore("medium", "medium"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_003", controlName: "Authenticated timing with symmetric key", status: "implemented", evidence: "Bounded staleness checks active" },
          { controlId: "CTRL_004", controlName: "Local holdover mode", status: "implemented", evidence: "Crystal oscillator holdover fallback" },
        ],
        residualRisk: 3.0,
        notes: "Staleness window violation alerts configured",
        createdBy: "system",
      },
      {
        threatId: "THREAT_003",
        threatName: "Quantum Factoring Attack on RSA/ECC",
        description: "Quantum factoring attack on RSA/ECC key exchange using Shor's algorithm",
        category: "crypto",
        attackVector: "Quantum computing attack on asymmetric cryptography",
        likelihood: "low",
        impact: "critical",
        riskScore: calculateRiskScore("low", "critical"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_005", controlName: "TL-KEM lattice-based key encapsulation", status: "implemented", evidence: "CNSA 2.0 compliant algorithms deployed" },
          { controlId: "CTRL_006", controlName: "Hybrid classical+post-quantum key exchange", status: "implemented", evidence: "Dual key exchange in production" },
        ],
        residualRisk: 1.5,
        notes: "Key entropy monitoring and algorithm compliance checker active",
        createdBy: "system",
      },
      {
        threatId: "THREAT_004",
        threatName: "Side-Channel Leakage from AES-256-GCM",
        description: "Side-channel leakage from AES-256-GCM implementation via power analysis",
        category: "crypto",
        attackVector: "Power/EM side-channel analysis of cryptographic operations",
        likelihood: "medium",
        impact: "high",
        riskScore: calculateRiskScore("medium", "high"),
        mitigationStatus: "in_progress",
        controls: [
          { controlId: "CTRL_007", controlName: "Constant-time implementations", status: "implemented", evidence: "GF(3) masking applied" },
          { controlId: "CTRL_008", controlName: "Software fallback with additional masking", status: "in_progress", evidence: "Additional masking rounds under development" },
        ],
        residualRisk: 3.5,
        notes: "Power analysis anomaly detection in FPGA paths",
        createdBy: "system",
      },
      {
        threatId: "THREAT_005",
        threatName: "Routing Table Poisoning in Torsion Network",
        description: "Routing table poisoning targeting Torsion Network topology",
        category: "network",
        attackVector: "Malicious routing update injection",
        likelihood: "low",
        impact: "high",
        riskScore: calculateRiskScore("low", "high"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_009", controlName: "Authenticated TTP routing updates", status: "implemented", evidence: "Greedy geodesic path verification active" },
          { controlId: "CTRL_010", controlName: "Static route fallback", status: "implemented", evidence: "Pre-shared topology backup" },
        ],
        residualRisk: 1.8,
        notes: "Route convergence time monitoring and topology hash verification",
        createdBy: "system",
      },
      {
        threatId: "THREAT_006",
        threatName: "Electromagnetic Fault Injection on FPGA",
        description: "Electromagnetic fault injection targeting FPGA hardware components",
        category: "physical",
        attackVector: "EM fault injection on FPGA circuits",
        likelihood: "low",
        impact: "high",
        riskScore: calculateRiskScore("low", "high"),
        mitigationStatus: "in_progress",
        controls: [
          { controlId: "CTRL_011", controlName: "Sensor mesh with glitch detectors", status: "implemented", evidence: "Voltage/temperature glitch detectors with auto-zeroization" },
          { controlId: "CTRL_012", controlName: "Triple modular redundancy", status: "in_progress", evidence: "TMR on critical paths under implementation" },
        ],
        residualRisk: 3.0,
        notes: "Glitch detector interrupt with audit trail",
        createdBy: "system",
      },
      {
        threatId: "THREAT_007",
        threatName: "Compromised FPGA Bitstream or Silicon Trojan",
        description: "Compromised FPGA bitstream or silicon trojan insertion via supply chain",
        category: "supply_chain",
        attackVector: "Tampered hardware components or firmware",
        likelihood: "low",
        impact: "critical",
        riskScore: calculateRiskScore("low", "critical"),
        mitigationStatus: "acknowledged",
        controls: [
          { controlId: "CTRL_013", controlName: "Bitstream signature verification", status: "implemented", evidence: "SBOM tracking active" },
          { controlId: "CTRL_014", controlName: "Dual-vendor FPGA strategy", status: "planned", evidence: "Cross-validation framework designed" },
        ],
        residualRisk: 4.5,
        notes: "Post-synthesis LUT count verification against golden reference",
        createdBy: "system",
      },
      {
        threatId: "THREAT_008",
        threatName: "Timing Side-Channel in Ternary Arithmetic",
        description: "Timing side-channel attacks on ternary arithmetic operations",
        category: "side_channel",
        attackVector: "Execution timing variance analysis",
        likelihood: "medium",
        impact: "medium",
        riskScore: calculateRiskScore("medium", "medium"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_015", controlName: "Constant-time GF(3) operations", status: "implemented", evidence: "Balanced ternary conversion shielding" },
          { controlId: "CTRL_016", controlName: "Randomized operation scheduling", status: "implemented", evidence: "Execution time variance monitoring active" },
        ],
        residualRisk: 1.5,
        notes: "Execution time variance monitoring enabled",
        createdBy: "system",
      },
      {
        threatId: "THREAT_009",
        threatName: "Grover's Algorithm Brute-Force Acceleration",
        description: "Grover's algorithm acceleration of brute-force on symmetric keys",
        category: "quantum",
        attackVector: "Quantum brute-force on symmetric cryptography",
        likelihood: "low",
        impact: "medium",
        riskScore: calculateRiskScore("low", "medium"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_017", controlName: "256-bit minimum key sizes", status: "implemented", evidence: "AES-256-GCM standard enforced" },
          { controlId: "CTRL_018", controlName: "Key size upgrade path", status: "planned", evidence: "384-bit upgrade path documented" },
        ],
        residualRisk: 0.8,
        notes: "CNSA 2.0 compliance validation on all key generation",
        createdBy: "system",
      },
      {
        threatId: "THREAT_010",
        threatName: "Privileged Admin Key Exfiltration",
        description: "Privileged administrator exfiltrating cryptographic keys",
        category: "insider",
        attackVector: "Privileged access abuse for key theft",
        likelihood: "medium",
        impact: "high",
        riskScore: calculateRiskScore("medium", "high"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_019", controlName: "SHA-256 hashed key storage", status: "implemented", evidence: "Audit trail on all key operations" },
          { controlId: "CTRL_020", controlName: "Automatic key rotation", status: "implemented", evidence: "6-hour cron rotation with dual-key grace periods" },
        ],
        residualRisk: 2.8,
        notes: "IP dispersion alerts (>10 IPs/24h); usage spike detection (>300% DoD)",
        createdBy: "system",
      },
      {
        threatId: "THREAT_011",
        threatName: "FIPS 140-3 Module Boundary Not Validated",
        description: "FIPS 140-3 module boundary not formally validated by accredited lab",
        category: "compliance",
        attackVector: "Regulatory non-compliance exposure",
        likelihood: "high",
        impact: "medium",
        riskScore: calculateRiskScore("high", "medium"),
        mitigationStatus: "in_progress",
        controls: [
          { controlId: "CTRL_021", controlName: "Self-assessment checklist", status: "implemented", evidence: "CMVP boundary diagram and CNSA 2.0 algorithm set documented" },
          { controlId: "CTRL_022", controlName: "Pre-validation review", status: "in_progress", evidence: "Engagement with accredited lab initiated" },
        ],
        residualRisk: 4.0,
        notes: "14-item compliance checklist automated scoring",
        createdBy: "system",
      },
      {
        threatId: "THREAT_012",
        threatName: "Path Traversal in API File Operations",
        description: "Path traversal vulnerabilities in API file operation endpoints",
        category: "software",
        attackVector: "Malicious file path manipulation in API requests",
        likelihood: "medium",
        impact: "medium",
        riskScore: calculateRiskScore("medium", "medium"),
        mitigationStatus: "mitigated",
        controls: [
          { controlId: "CTRL_023", controlName: "Path sanitization and null-byte stripping", status: "implemented", evidence: "Double URL-decode protection active" },
          { controlId: "CTRL_024", controlName: "WAF layer with path normalization", status: "implemented", evidence: "execFile()-only subprocess execution enforced" },
        ],
        residualRisk: 1.2,
        notes: "Request logging with pattern matching on traversal attempts",
        createdBy: "system",
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

  getLikelihoodLevels() {
    return [...LIKELIHOOD_LEVELS];
  },

  getImpactLevels() {
    return [...IMPACT_LEVELS];
  },

  getMitigationStatuses() {
    return [...MITIGATION_STATUSES];
  },
};
