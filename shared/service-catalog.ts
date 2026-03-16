/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * SINGLE SOURCE OF TRUTH — PlenumNET Service Catalog.
 * All API service definitions, endpoint lists, and domain groupings live here.
 * The API Demo page, Kong Konnect page, and backend Kong routes all derive from this file.
 */

import { PLATFORM } from "./constants";

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface EndpointDef {
  method: HttpMethod;
  path: string;
  desc: string;
  tryIt?: boolean;
  admin?: boolean;
}

export interface ServiceDef {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  deprecated?: boolean;
  kong: {
    serviceName: string;
    routePath: string;
    tags: string[];
    rateLimit: { minute: number; hour: number };
    methods: HttpMethod[];
  };
  endpoints: EndpointDef[];
}

export interface DomainGroupDef {
  id: string;
  name: string;
  description: string;
  serviceIds: string[];
}

export const SERVICE_CATALOG: ServiceDef[] = [
  {
    id: "timing",
    name: "Timing API",
    description: "Femtosecond-precision timestamps and Salvi Epoch synchronization",
    icon: "Clock",
    color: "text-blue-600",
    kong: {
      serviceName: "plenumnet-timing",
      routePath: "/api/salvi/timing",
      tags: ["plenumnet", "timing", "hptp", "finra-cat", "mifid-ii"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/salvi/timing/timestamp", desc: "Current femtosecond timestamp", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/metrics", desc: "Clock source and sync status", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/batch/:count", desc: "Batch timestamp generation" },
      { method: "GET", path: "/api/salvi/timing/self-test", desc: "Timing subsystem self-test", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/error-budget", desc: "Precision error budget", tryIt: true },
    ],
  },
  {
    id: "calendar",
    name: "Calendar / Epoch API",
    description: "42 calendar synchronization endpoints via Julian Day Number intermediary",
    icon: "Globe",
    color: "text-indigo-600",
    kong: {
      serviceName: "plenumnet-calendars",
      routePath: "/api/salvi/timing/epoch",
      tags: ["plenumnet", "calendars", "epoch", "synchronization"],
      rateLimit: { minute: 120, hour: 1200 },
      methods: ["GET"],
    },
    endpoints: [
      { method: "GET", path: "/api/salvi/timing/epoch/anchors", desc: "All epoch anchor points", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars", desc: "All 42 calendar conversions", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/gregorian", desc: "Gregorian (civil standard)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/hebrew", desc: "Hebrew Anno Mundi (3761 BCE)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/islamic", desc: "Islamic Hijri (lunar)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/chinese", desc: "Chinese Sexagenary (60-year)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/mayan", desc: "Mayan Long Count (3114 BCE)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/egyptian", desc: "Egyptian Civil (Sothic cycle)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/byzantine", desc: "Byzantine (5509 BCE)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/julian-day", desc: "Julian Day Number (4713 BCE)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/vedic", desc: "Vedic Kali Yuga (3102 BCE)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/thirteen-moon", desc: "13-Moon Harmonic (~30,000 yr)", tryIt: true },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/buddhist", desc: "Thai Buddhist Era (543 BCE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/ethiopian", desc: "Ethiopian / Ge'ez (13 mo)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/coptic", desc: "Coptic Era of Martyrs" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/japanese", desc: "Japanese Imperial Koki" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/korean", desc: "Korean Dangun Era (2333 BCE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/vietnamese", desc: "Vietnamese (lunisolar)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/tamil", desc: "Tamil (sidereal Sankranti)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/bengali", desc: "Bengali / Bangla" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/indian-saka", desc: "Indian National / Saka" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/vikram-samvat", desc: "Vikram Samvat (57 BCE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/solar-hijri", desc: "Persian / Solar Hijri" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/jain", desc: "Jain Vira Nirvana Samvat" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/zoroastrian", desc: "Zoroastrian Fasli" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/roman-auc", desc: "Roman Ab Urbe Condita" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/amazigh", desc: "Amazigh / Berber (Yennayer)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/igbo", desc: "Igbo (4-day week + 13 months)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/yoruba", desc: "Yoruba (4-day Ojo week)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/akan", desc: "Akan (42-day Adae cycle)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/khmer", desc: "Khmer (lunisolar)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/burmese", desc: "Burmese (638 CE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/javanese", desc: "Javanese (5+7 dual cycle)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/malayalam", desc: "Malayalam / Kollam (825 CE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/nepal-sambat", desc: "Nepal Sambat (879 CE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/balinese", desc: "Balinese Pawukon (210-day)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/tibetan", desc: "Tibetan Rabjung (60-year)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/nanakshahi", desc: "Nanakshahi / Sikh (1469 CE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/bahai", desc: "Baha'i Badi' (19x19)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/minguo", desc: "Minguo / ROC (1912 CE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/aboriginal", desc: "Aboriginal Seasonal (~65,000 yr)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/assyrian", desc: "Assyrian (4750 BCE)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/aztec", desc: "Aztec Tonalpohualli (260-day)" },
      { method: "GET", path: "/api/salvi/timing/epoch/calendars/nisgaa", desc: "Nisga'a Seasonal (oral)" },
    ],
  },
  {
    id: "ternary",
    name: "Ternary Operations",
    description: "GF(3) Galois field arithmetic, representation conversion, and density analysis",
    icon: "Calculator",
    color: "text-green-600",
    kong: {
      serviceName: "plenumnet-ternary",
      routePath: "/api/salvi/ternary",
      tags: ["plenumnet", "ternary", "quantum-safe", "gf3"],
      rateLimit: { minute: 200, hour: 2000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "POST", path: "/api/salvi/ternary/add", desc: "GF(3) addition" },
      { method: "POST", path: "/api/salvi/ternary/multiply", desc: "GF(3) multiplication" },
      { method: "POST", path: "/api/salvi/ternary/not", desc: "Ternary negation" },
      { method: "POST", path: "/api/salvi/ternary/xor", desc: "Ternary XOR (field addition)" },
      { method: "POST", path: "/api/salvi/ternary/rotate", desc: "Ternary rotation" },
      { method: "POST", path: "/api/salvi/ternary/convert", desc: "Convert between representations A/B/C" },
      { method: "POST", path: "/api/salvi/ternary/batch", desc: "Batch ternary operations" },
      { method: "POST", path: "/api/salvi/ternary/noether-verify", desc: "Noether symmetry verification" },
      { method: "GET", path: "/api/salvi/ternary/density/:tritCount", desc: "Information density for N trits" },
      { method: "GET", path: "/api/salvi/ternary/density-benchmark", desc: "Density benchmark suite", tryIt: true },
    ],
  },
  {
    id: "phase",
    name: "Phase Encryption",
    description: "Quantum-resistant phase-split encryption with multiple security modes",
    icon: "Shield",
    color: "text-red-600",
    kong: {
      serviceName: "plenumnet-phase",
      routePath: "/api/salvi/phase",
      tags: ["plenumnet", "encryption", "phase", "quantum-safe"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "POST", path: "/api/salvi/phase/split", desc: "Split data into phase components" },
      { method: "POST", path: "/api/salvi/phase/recombine", desc: "Recombine phase components" },
      { method: "POST", path: "/api/salvi/phase/batch/split", desc: "Batch phase split" },
      { method: "POST", path: "/api/salvi/phase/batch/recombine", desc: "Batch phase recombine" },
      { method: "GET", path: "/api/salvi/phase/config/:mode", desc: "Encryption mode configuration" },
      { method: "GET", path: "/api/salvi/phase/recommend", desc: "Recommended mode for use case", tryIt: true },
    ],
  },
  {
    id: "capabilities",
    name: "Capability-Based Security",
    description: "Unforgeable bearer tokens with TL-DSA signatures, HPTP-bound expiration, and delegation chains",
    icon: "Lock",
    color: "text-amber-600",
    kong: {
      serviceName: "plenumnet-capabilities",
      routePath: "/api/capabilities",
      tags: ["plenumnet", "capabilities", "security", "bearer-tokens", "tl-dsa"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/capabilities/status", desc: "Capability system status and phase info", tryIt: true },
      { method: "GET", path: "/api/capabilities/audit", desc: "Merkle-chained audit event log", tryIt: true },
      { method: "GET", path: "/api/capabilities/demo/expiration", desc: "Demo: HPTP-bound token expiration", tryIt: true },
      { method: "GET", path: "/api/capabilities/demo/delegation", desc: "Demo: HMAC-chained delegation (macaroon-style)", tryIt: true },
      { method: "GET", path: "/api/capabilities/demo/confinement", desc: "Demo: Hardware-bound confinement problem", tryIt: true },
      { method: "GET", path: "/api/capabilities/demo/certificates", desc: "Demo: RFC 3161 capability certificates", tryIt: true },
      { method: "GET", path: "/api/capabilities/demo/mesh", desc: "Demo: Inter-service capability mesh", tryIt: true },
      { method: "GET", path: "/api/capabilities/certificate/stats", desc: "Certificate issuance statistics", tryIt: true },
      { method: "GET", path: "/api/capabilities/certificate/:certId/verify-data", desc: "Verify certificate data integrity" },
      { method: "GET", path: "/api/capabilities/certificate/:certId/rfc3161", desc: "RFC 3161 timestamp for certificate" },
      { method: "GET", path: "/api/capabilities/mesh/topology", desc: "Service mesh topology graph", tryIt: true },
      { method: "GET", path: "/api/capabilities/mesh/discover", desc: "Service discovery by resource pattern", tryIt: true },
      { method: "GET", path: "/api/capabilities/mesh/health", desc: "Mesh health monitoring", tryIt: true },
      { method: "POST", path: "/api/capabilities/issue", desc: "Issue capability token" },
      { method: "POST", path: "/api/capabilities/validate", desc: "Validate capability token" },
      { method: "POST", path: "/api/capabilities/verify-chain", desc: "Verify delegation chain" },
      { method: "POST", path: "/api/capabilities/delegate", desc: "Delegate capability with attenuation" },
      { method: "POST", path: "/api/capabilities/delegate/chain", desc: "Multi-hop delegation chain" },
      { method: "POST", path: "/api/capabilities/hardware/register", desc: "Register hardware device" },
      { method: "POST", path: "/api/capabilities/hardware/challenge", desc: "Hardware challenge-response auth" },
      { method: "POST", path: "/api/capabilities/hardware/verify", desc: "Verify hardware binding" },
      { method: "POST", path: "/api/capabilities/hardware/issue", desc: "Issue hardware-bound capability" },
      { method: "POST", path: "/api/capabilities/certificate/issue", desc: "Issue RFC 3161 capability certificate" },
      { method: "POST", path: "/api/capabilities/certificate/verify", desc: "Verify certificate chain" },
      { method: "POST", path: "/api/capabilities/certificate/evidence-chain", desc: "Assemble evidence chain" },
      { method: "POST", path: "/api/capabilities/mesh/register", desc: "Register service in mesh" },
      { method: "POST", path: "/api/capabilities/mesh/issue", desc: "Issue service-to-service capability" },
      { method: "POST", path: "/api/capabilities/mesh/propagate", desc: "Propagate capability with hop attenuation" },
      { method: "POST", path: "/api/capabilities/mesh/validate", desc: "Validate mesh capability" },
    ],
  },
  {
    id: "tsa",
    name: "TSA / Ternary Notary Stamp",
    description: "RFC 3161 digital notary with dual-signature (RSA-4096 + TL-DSA), ternary compression, Hedera HCS anchoring, and 42 calendar proofs",
    icon: "Stamp",
    color: "text-emerald-700",
    kong: {
      serviceName: "plenumnet-tsa",
      routePath: "/api/tsa",
      tags: ["plenumnet", "tsa", "rfc3161", "notary", "timestamps"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "POST", path: "/api/tsa/timestamp", desc: "DER-encoded RFC 3161 timestamp (ASN.1 wire protocol)" },
      { method: "POST", path: "/api/tsa/timestamp/json", desc: "JSON timestamp with calendar proofs and TL-DSA co-signature" },
      { method: "POST", path: "/api/tsa/verify", desc: "Verify timestamp token (DER or JSON)" },
      { method: "GET", path: "/api/tsa/certificate", desc: "TSA certificate metadata (RSA-4096)", tryIt: true },
      { method: "GET", path: "/api/tsa/certificate/download", desc: "Download TSA certificate (PEM)", tryIt: true },
      { method: "GET", path: "/api/tsa/tokens", desc: "List issued timestamp tokens", admin: true },
      { method: "GET", path: "/api/tsa/policy", desc: "TSA policy configuration (4 policies)", tryIt: true },
      { method: "GET", path: "/api/tsa/health", desc: "TSA service health", tryIt: true },
      { method: "GET", path: "/api/tsa/audit/query", desc: "Merkle-chained audit log query", admin: true },
    ],
  },
  {
    id: "hedera",
    name: "Hedera HCS Witnessing",
    description: "Blockchain non-repudiation via Hedera Consensus Service — immutable, ordered, timestamped witness proofs",
    icon: "Link2",
    color: "text-indigo-700",
    kong: {
      serviceName: "plenumnet-hedera",
      routePath: "/api/hedera",
      tags: ["plenumnet", "hedera", "hcs", "blockchain", "witnessing"],
      rateLimit: { minute: 30, hour: 300 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "POST", path: "/api/hedera/v1/witness", desc: "Submit witness hash to HCS topic" },
      { method: "GET", path: "/api/hedera/v1/witness/:txId", desc: "Lookup witness by transaction ID" },
      { method: "POST", path: "/api/hedera/v1/verify", desc: "Verify witness proof via mirror node" },
      { method: "GET", path: "/api/hedera/v1/topic", desc: "HCS topic info and stats", tryIt: true },
      { method: "GET", path: "/api/hedera/v1/health", desc: "Hedera service health", tryIt: true },
      { method: "GET", path: "/api/hedera/v1/stats", desc: "Witnessing statistics", tryIt: true },
    ],
  },
  {
    id: "sfk",
    name: "SFK Operations Pipeline",
    description: "Salvi Framework Kernel operation lifecycle: init \u2192 ternary processing \u2192 witnessing \u2192 finalization",
    icon: "Workflow",
    color: "text-rose-700",
    kong: {
      serviceName: "plenumnet-sfk",
      routePath: "/api/sfk",
      tags: ["plenumnet", "sfk", "operations", "pipeline"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST", "DELETE"],
    },
    endpoints: [
      { method: "POST", path: "/api/sfk/v1/operations", desc: "Submit SFK operation" },
      { method: "GET", path: "/api/sfk/v1/operations", desc: "List all operations", tryIt: true },
      { method: "GET", path: "/api/sfk/v1/operations/:id", desc: "Operation status by ID" },
      { method: "DELETE", path: "/api/sfk/v1/operations/:id", desc: "Cancel operation" },
      { method: "GET", path: "/api/sfk/v1/stats", desc: "Pipeline statistics", tryIt: true },
    ],
  },
  {
    id: "inter-cube",
    name: "Inter-Cube Infrastructure",
    description: "Geometric routing across the 13D ternary cube network: GLB, CON, CRS, and FTS services",
    icon: "Hexagon",
    color: "text-cyan-700",
    kong: {
      serviceName: "plenumnet-inter-cube",
      routePath: "/",
      tags: ["plenumnet", "inter-cube", "geometric-routing", "13d"],
      rateLimit: { minute: 200, hour: 2000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "POST", path: "/crs/register", desc: "Register cube address in CRS" },
      { method: "GET", path: "/crs/lookup/:address", desc: "Lookup cube address" },
      { method: "GET", path: "/crs/neighbors/:address", desc: "Neighbor cubes for address" },
      { method: "POST", path: "/crs/heartbeat", desc: "CRS heartbeat keepalive" },
      { method: "POST", path: "/crs/deregister", desc: "Deregister cube address" },
      { method: "GET", path: "/crs/stats", desc: "CRS registry statistics", tryIt: true },
      { method: "POST", path: "/glb/forward", desc: "Forward request via geometric load balancer" },
      { method: "GET", path: "/glb/stats", desc: "GLB routing statistics", tryIt: true },
      { method: "GET", path: "/glb/health", desc: "GLB health check", tryIt: true },
      { method: "GET", path: "/con/neighbors", desc: "Cube overlay network neighbors", tryIt: true },
      { method: "GET", path: "/con/stats", desc: "CON tunnel statistics", tryIt: true },
      { method: "POST", path: "/con/tunnel/refresh", desc: "Refresh overlay tunnel" },
      { method: "GET", path: "/fts/status", desc: "Fault tolerance service status", tryIt: true },
      { method: "GET", path: "/fts/dead", desc: "Dead cube detection", tryIt: true },
      { method: "POST", path: "/fts/config", desc: "Update FTS configuration" },
      { method: "POST", path: "/routing/compute", desc: "Compute geometric route" },
      { method: "POST", path: "/address/validate", desc: "Validate cube address" },
      { method: "GET", path: "/topology", desc: "Full cube network topology", tryIt: true },
    ],
  },
  {
    id: "tdns",
    name: "TDNS v2.5 Addressing",
    description: "27-dimensional ontological addressing with 54-trit dual-layer protocol, TL-Sponge-43 identity, and TIS-27 wire integrity",
    icon: "Globe",
    color: "text-violet-700",
    kong: {
      serviceName: "plenumnet-tdns",
      routePath: "/api/tdns",
      tags: ["plenumnet", "tdns", "addressing", "ontological", "27d", "tis-27"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "POST", path: "/api/tdns/scan", desc: "Scan ternary address space" },
      { method: "POST", path: "/api/tdns/register", desc: "Register TDNS name" },
      { method: "GET", path: "/api/tdns/resolve/:name", desc: "Resolve TDNS name to address" },
      { method: "GET", path: "/api/tdns/resolve", desc: "Resolve by query parameter" },
      { method: "POST", path: "/api/tdns/org/create", desc: "Create organization entity" },
      { method: "POST", path: "/api/tdns/org/add-url", desc: "Add URL to organization" },
      { method: "GET", path: "/api/tdns/org/:name", desc: "Organization details" },
      { method: "GET", path: "/api/tdns/orgs", desc: "List all organizations", tryIt: true },
      { method: "GET", path: "/api/tdns/list", desc: "List all TDNS records", tryIt: true },
      { method: "GET", path: "/api/tdns/records", desc: "All registered records", tryIt: true },
      { method: "GET", path: "/api/tdns/health", desc: "TDNS service health", tryIt: true },
    ],
  },
  {
    id: "security",
    name: "Security Infrastructure",
    description: "Audit service, HPTP anomaly detection, threat model registry, and security dashboard",
    icon: "Shield",
    color: "text-rose-600",
    kong: {
      serviceName: "plenumnet-security",
      routePath: "/api/security",
      tags: ["plenumnet", "security", "audit", "threats", "hptp", "dashboard"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST", "PATCH", "DELETE"],
    },
    endpoints: [
      { method: "GET", path: "/api/security/dashboard", desc: "Security dashboard overview", tryIt: true },
      { method: "GET", path: "/api/security/kri", desc: "Key Risk Indicators", tryIt: true },
      { method: "GET", path: "/api/security/metadata/categories", desc: "Security categories metadata", tryIt: true },
      { method: "GET", path: "/api/security/metadata/types", desc: "Security types metadata", tryIt: true },
      { method: "GET", path: "/api/security/audit", desc: "Security audit entries", tryIt: true },
      { method: "GET", path: "/api/security/audit/stats", desc: "Audit statistics", tryIt: true },
      { method: "GET", path: "/api/security/audit/summary", desc: "Audit summary", tryIt: true },
      { method: "GET", path: "/api/security/audit/unresolved", desc: "Unresolved audit findings", tryIt: true },
      { method: "GET", path: "/api/security/audit/:id", desc: "Audit entry by ID" },
      { method: "PATCH", path: "/api/security/audit/:id/resolve", desc: "Resolve audit finding", admin: true },
      { method: "POST", path: "/api/security/audit", desc: "Create audit entry", admin: true },
      { method: "GET", path: "/api/security/threats", desc: "Threat model registry", tryIt: true },
      { method: "GET", path: "/api/security/threats/stats", desc: "Threat statistics", tryIt: true },
      { method: "GET", path: "/api/security/threats/risk-matrix", desc: "Risk matrix visualization", tryIt: true },
      { method: "GET", path: "/api/security/threats/meta", desc: "Threat metadata", tryIt: true },
      { method: "GET", path: "/api/security/threats/:id", desc: "Threat by ID" },
      { method: "POST", path: "/api/security/threats", desc: "Register threat", admin: true },
      { method: "POST", path: "/api/security/threats/seed", desc: "Seed threat database", admin: true },
      { method: "DELETE", path: "/api/security/threats/:id", desc: "Delete threat", admin: true },
      { method: "GET", path: "/api/security/implementation", desc: "Implementation status tracker", tryIt: true },
      { method: "GET", path: "/api/security/implementation/summary", desc: "Implementation summary", tryIt: true },
      { method: "GET", path: "/api/security/implementation/metrics", desc: "Implementation metrics", tryIt: true },
      { method: "GET", path: "/api/security/implementation/milestones", desc: "Implementation milestones", tryIt: true },
      { method: "GET", path: "/api/security/implementation/meta", desc: "Implementation metadata", tryIt: true },
      { method: "GET", path: "/api/security/implementation/:id", desc: "Implementation item by ID" },
      { method: "POST", path: "/api/security/implementation", desc: "Create implementation item", admin: true },
      { method: "POST", path: "/api/security/implementation/seed", desc: "Seed implementation data", admin: true },
      { method: "DELETE", path: "/api/security/implementation/:id", desc: "Delete implementation item", admin: true },
      { method: "PATCH", path: "/api/security/implementation/:id", desc: "Update implementation item", admin: true },
      { method: "GET", path: "/api/security/hptp/status", desc: "HPTP timing security status", tryIt: true },
      { method: "GET", path: "/api/security/hptp/stats", desc: "HPTP anomaly statistics", tryIt: true },
      { method: "GET", path: "/api/security/hptp/anomalies", desc: "Detected timing anomalies", tryIt: true },
      { method: "GET", path: "/api/security/hptp/thresholds", desc: "Anomaly detection thresholds", tryIt: true },
      { method: "GET", path: "/api/security/hptp/redundancy", desc: "Clock redundancy status", tryIt: true },
      { method: "GET", path: "/api/security/hptp/fallback-modes", desc: "Timing fallback modes", tryIt: true },
      { method: "GET", path: "/api/security/hptp/fallback-analysis", desc: "Fallback mode analysis", tryIt: true },
      { method: "POST", path: "/api/security/hptp/anomalies", desc: "Report timing anomaly", admin: true },
      { method: "PATCH", path: "/api/security/threats/:id", desc: "Update threat", admin: true },
    ],
  },
  {
    id: "api-keys",
    name: "API Key Management",
    description: "Key generation, validation, rotation, per-key rate limiting, anomaly detection, and WBS tagging",
    icon: "Key",
    color: "text-yellow-600",
    kong: {
      serviceName: "plenumnet-api-keys",
      routePath: "/api/keys",
      tags: ["plenumnet", "api-keys", "management", "rate-limiting"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST", "PATCH"],
    },
    endpoints: [
      { method: "GET", path: "/api/keys", desc: "List all API keys", tryIt: true },
      { method: "GET", path: "/api/keys/stats", desc: "API key statistics", tryIt: true },
      { method: "GET", path: "/api/keys/scopes", desc: "Available key scopes", tryIt: true },
      { method: "GET", path: "/api/keys/rate-limit-tiers", desc: "Rate limit tier definitions", tryIt: true },
      { method: "GET", path: "/api/keys/entity-types", desc: "WBS entity type definitions", tryIt: true },
      { method: "GET", path: "/api/keys/expiring", desc: "Keys approaching expiration", tryIt: true },
      { method: "GET", path: "/api/keys/anomalies", desc: "Anomaly detection alerts", tryIt: true },
      { method: "GET", path: "/api/keys/audit", desc: "Key audit trail", tryIt: true },
      { method: "GET", path: "/api/keys/validate-external", desc: "Validate external API key" },
      { method: "GET", path: "/api/keys/:id/audit", desc: "Audit trail for specific key" },
      { method: "GET", path: "/api/keys/:id/logs", desc: "Usage logs for specific key" },
      { method: "POST", path: "/api/keys/generate", desc: "Generate new API key" },
      { method: "POST", path: "/api/keys/rotate/:id", desc: "Rotate API key" },
      { method: "POST", path: "/api/keys/revoke/:id", desc: "Revoke API key" },
      { method: "PATCH", path: "/api/keys/:id/metadata", desc: "Update key metadata" },
      { method: "PATCH", path: "/api/keys/:id/rate-limit", desc: "Update key rate limit" },
    ],
  },
  {
    id: "tribonacci",
    name: "Tribonacci / Agent Array",
    description: "28-dimension agent array for parallel query analysis with etymology audit and fact-checking",
    icon: "Layers",
    color: "text-violet-600",
    kong: {
      serviceName: "plenumnet-tribonacci",
      routePath: "/api/tribonacci",
      tags: ["plenumnet", "tribonacci", "agent-array", "28d"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/tribonacci/agent-array/positions", desc: "28 agent positions in cube space", tryIt: true },
      { method: "GET", path: "/api/tribonacci/agent-array/reports", desc: "All situation reports", tryIt: true },
      { method: "GET", path: "/api/tribonacci/agent-array/reports/:id", desc: "Situation report by ID" },
      { method: "GET", path: "/api/tribonacci/agent-array/stream/:sessionId", desc: "Stream agent analysis progress" },
      { method: "GET", path: "/api/tribonacci/coverage", desc: "Tribonacci coverage analysis", tryIt: true },
      { method: "GET", path: "/api/tribonacci/hash", desc: "Tribonacci hash computation", tryIt: true },
      { method: "GET", path: "/api/tribonacci/hash-distribution", desc: "Hash distribution analysis", tryIt: true },
      { method: "GET", path: "/api/tribonacci/hook", desc: "Tribonacci hook endpoint", tryIt: true },
      { method: "GET", path: "/api/tribonacci/next-worker", desc: "Next worker assignment", tryIt: true },
      { method: "GET", path: "/api/tribonacci/permutation", desc: "Tribonacci permutation", tryIt: true },
      { method: "GET", path: "/api/tribonacci/sequence", desc: "Tribonacci sequence generation", tryIt: true },
      { method: "GET", path: "/api/tribonacci/skip-lookup", desc: "Skip-ahead lookup table", tryIt: true },
      { method: "POST", path: "/api/tribonacci/agent-array", desc: "Execute parallel agent query" },
      { method: "POST", path: "/api/tribonacci/agent-array/save", desc: "Save agent report" },
      { method: "POST", path: "/api/tribonacci/generate-id", desc: "Generate tribonacci ID" },
    ],
  },
  {
    id: "compression",
    name: "TTC Compression Engine",
    description: "TTC v4.2 ternary rANS compression with 9 levels, 4 modes, and database-backed document storage",
    icon: "Database",
    color: "text-teal-600",
    kong: {
      serviceName: "plenumnet-compression",
      routePath: "/api/compression",
      tags: ["plenumnet", "compression", "ttc", "ternary-rans"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST", "DELETE"],
    },
    endpoints: [
      { method: "POST", path: "/api/compression/file", desc: "Compress file (binary transport)" },
      { method: "POST", path: "/api/compression/decompress", desc: "Decompress TTC binary" },
      { method: "POST", path: "/api/compression/db/store", desc: "Compress and store to database" },
      { method: "GET", path: "/api/compression/db/retrieve/:id", desc: "Retrieve compressed document" },
      { method: "GET", path: "/api/compression/db/documents", desc: "List stored documents", tryIt: true },
      { method: "DELETE", path: "/api/compression/db/documents/:id", desc: "Delete stored document" },
    ],
  },
  {
    id: "demo",
    name: "Compression Demo",
    description: "Interactive compression demonstrations with session tracking and statistics",
    icon: "TrendingUp",
    color: "text-teal-500",
    kong: {
      serviceName: "plenumnet-demo",
      routePath: "/api/demo",
      tags: ["plenumnet", "demo", "compression", "interactive"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "POST", path: "/api/demo/run", desc: "Run compression on dataset" },
      { method: "POST", path: "/api/demo/upload", desc: "Upload custom data for compression" },
      { method: "GET", path: "/api/demo/stats", desc: "Aggregated compression statistics", tryIt: true },
      { method: "GET", path: "/api/demo/session/:sessionId", desc: "Session details" },
      { method: "GET", path: "/api/demo/data/:sessionId", desc: "Session data export" },
      { method: "GET", path: "/api/demo/history", desc: "Compression run history", tryIt: true },
      { method: "GET", path: "/api/demo/files", desc: "Available demo files", tryIt: true },
    ],
  },
  {
    id: "vm",
    name: "Virtual Machine",
    description: `Ternary VM with ${PLATFORM.VM_OPCODES}-opcode ISA ${PLATFORM.VM_ISA_VERSION}, 27 registers, and conformance testing`,
    icon: "Cpu",
    color: "text-cyan-600",
    kong: {
      serviceName: "plenumnet-vm",
      routePath: "/api/salvi/vm",
      tags: ["plenumnet", "vm", "virtual-machine", "isa"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET"],
    },
    endpoints: [
      { method: "GET", path: "/api/salvi/vm/spec", desc: `TVM ${PLATFORM.VM_OPCODES}-opcode ISA ${PLATFORM.VM_ISA_VERSION} specification`, tryIt: true },
      { method: "GET", path: "/api/salvi/vm/conformance", desc: "TVM conformance test suite", tryIt: true },
      { method: "GET", path: "/api/salvi/docs", desc: "Documentation index", tryIt: true },
    ],
  },
  {
    id: "kong",
    name: "Kong Konnect Gateway",
    description: "API gateway management, control planes, service catalog, and cloud deployment",
    icon: "Server",
    color: "text-emerald-600",
    kong: {
      serviceName: "plenumnet-kong",
      routePath: "/api/kong",
      tags: ["plenumnet", "kong", "gateway", "api-management"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/kong/status", desc: "Gateway connection status", tryIt: true },
      { method: "GET", path: "/api/kong/organization", desc: "Organization details", tryIt: true },
      { method: "GET", path: "/api/kong/config", desc: "Gateway configuration", tryIt: true },
      { method: "GET", path: "/api/kong/service-catalog", desc: "Service catalog", tryIt: true },
      { method: "GET", path: "/api/kong/control-planes", desc: "List control planes", tryIt: true },
      { method: "GET", path: "/api/kong/control-planes/:cpId/services", desc: "Services in control plane" },
      { method: "GET", path: "/api/kong/control-planes/:cpId/routes", desc: "Routes in control plane" },
      { method: "GET", path: "/api/kong/control-planes/:cpId/plugins", desc: "Plugins in control plane" },
      { method: "GET", path: "/api/kong/control-planes/:cpId/deploy-instructions", desc: "Deployment instructions" },
      { method: "POST", path: "/api/kong/control-planes/:cpId/services", desc: "Create service", admin: true },
      { method: "POST", path: "/api/kong/control-planes/:cpId/services/:serviceId/routes", desc: "Create route", admin: true },
      { method: "POST", path: "/api/kong/control-planes/:cpId/services/:serviceId/plugins", desc: "Add plugin", admin: true },
      { method: "POST", path: "/api/kong/control-planes/:cpId/sync-plenumnet", desc: "Sync PlenumNET config", admin: true },
      { method: "POST", path: "/api/kong/control-planes/:cpId/generate-deployment", desc: "Generate deployment", admin: true },
      { method: "POST", path: "/api/kong/control-planes/:cpId/deploy-to-cloud", desc: "Deploy to cloud", admin: true },
      { method: "POST", path: "/api/kong/save-to-github", desc: "Save config to GitHub", admin: true },
      { method: "POST", path: "/api/kong/sync-all-control-planes", desc: "Sync all control planes", admin: true },
    ],
  },
  {
    id: "github",
    name: "GitHub Integration",
    description: "Repository management, file operations, batch push, and environment deployment",
    icon: "GitBranch",
    color: "text-gray-600",
    kong: {
      serviceName: "plenumnet-github",
      routePath: "/api/github",
      tags: ["plenumnet", "github", "repository", "ci-cd"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST", "PUT", "DELETE"],
    },
    endpoints: [
      { method: "GET", path: "/api/github/status", desc: "GitHub connection status", tryIt: true },
      { method: "GET", path: "/api/github/repos/:owner/:repo/branches", desc: "Repository branches", admin: true },
      { method: "GET", path: "/api/github/repos/:owner/:repo/contents", desc: "Repository contents", admin: true },
      { method: "GET", path: "/api/github/file/:owner/:repo", desc: "Read file from repo", admin: true },
      { method: "PUT", path: "/api/github/file/:owner/:repo", desc: "Write file to repo", admin: true },
      { method: "DELETE", path: "/api/github/file/:owner/:repo", desc: "Delete file from repo", admin: true },
      { method: "POST", path: "/api/github/token", desc: "Configure GitHub token", admin: true },
      { method: "POST", path: "/api/github/push-workflows/:owner/:repo", desc: "Push workflow files", admin: true },
      { method: "POST", path: "/api/github/push-batch/:owner/:repo", desc: "Batch push files", admin: true },
      { method: "POST", path: "/api/github/push-env/:owner/:repo", desc: "Push environment config", admin: true },
    ],
  },
  {
    id: "tonal",
    name: "Tonal Diffusion / Resonance",
    description: "Network-wide time synchronization using FM timing packets and toroidal topology",
    icon: "Radio",
    color: "text-orange-600",
    kong: {
      serviceName: "plenumnet-tonal",
      routePath: "/api/tonal",
      tags: ["plenumnet", "tonal", "diffusion", "fm-timing"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/tonal/field", desc: "Current tonal field data", tryIt: true },
      { method: "GET", path: "/api/tonal/neighbors", desc: "Tonal neighbor information", tryIt: true },
      { method: "POST", path: "/api/tonal/packet", desc: "Send FM timing packet" },
      { method: "GET", path: "/api/resonance/status", desc: "Resonance detector status", tryIt: true },
      { method: "POST", path: "/api/resonance/sweep", desc: "Run resonance sweep" },
      { method: "POST", path: "/api/resonance/rtt", desc: "Round-trip time measurement" },
    ],
  },
  {
    id: "ephemeris",
    name: "Ternary Ephemeris",
    description: "Ternary degree conversion with resonance scoring and planetary ephemeris calculations",
    icon: "Star",
    color: "text-purple-600",
    kong: {
      serviceName: "plenumnet-ephemeris",
      routePath: "/api/ephemeris",
      tags: ["plenumnet", "ephemeris", "ternary-degrees", "astronomy"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/ephemeris/info", desc: "Ephemeris API metadata", tryIt: true },
      { method: "POST", path: "/api/ephemeris/convert", desc: "Standard to ternary degree conversion" },
      { method: "POST", path: "/api/ephemeris/position", desc: "Single planet ephemeris calculation" },
      { method: "POST", path: "/api/ephemeris/batch", desc: "Batch planetary ephemeris" },
    ],
  },
  {
    id: "whitepapers",
    name: "Whitepapers & Docs",
    description: "Technical whitepapers and documentation management",
    icon: "BookOpen",
    color: "text-sky-600",
    kong: {
      serviceName: "plenumnet-whitepapers",
      routePath: "/api/whitepapers",
      tags: ["plenumnet", "whitepapers", "documentation"],
      rateLimit: { minute: 100, hour: 1000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/whitepapers", desc: "List all whitepapers", tryIt: true },
      { method: "GET", path: "/api/whitepapers/active", desc: "Active whitepapers only", tryIt: true },
      { method: "GET", path: "/api/whitepapers/:id", desc: "Whitepaper by ID" },
      { method: "POST", path: "/api/whitepapers", desc: "Create whitepaper", admin: true },
    ],
  },
  {
    id: "v1",
    name: "V1 Entrainment API",
    description: "Versioned entrainment advisory and coherence logging",
    icon: "TrendingUp",
    color: "text-fuchsia-600",
    deprecated: true,
    kong: {
      serviceName: "plenumnet-v1",
      routePath: "/api/v1",
      tags: ["plenumnet", "v1", "entrainment", "legacy"],
      rateLimit: { minute: 60, hour: 600 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/v1/status", desc: "V1 API status", tryIt: true },
      { method: "GET", path: "/api/v1/ternary/state", desc: "Ternary system state", tryIt: true },
      { method: "GET", path: "/api/v1/safety/limits", desc: "Safety limits configuration", tryIt: true },
      { method: "POST", path: "/api/v1/entrain/advise", desc: "Request entrainment advisory" },
      { method: "POST", path: "/api/v1/logs/coherence", desc: "Log coherence data" },
    ],
  },
  {
    id: "gdpr",
    name: "GDPR / Privacy",
    description: "Data export, deletion, and privacy policy compliance",
    icon: "Scale",
    color: "text-slate-600",
    kong: {
      serviceName: "plenumnet-gdpr",
      routePath: "/api/gdpr",
      tags: ["plenumnet", "gdpr", "privacy", "compliance"],
      rateLimit: { minute: 30, hour: 300 },
      methods: ["GET", "DELETE"],
    },
    endpoints: [
      { method: "GET", path: "/api/gdpr/policy", desc: "Privacy policy details", tryIt: true },
      { method: "GET", path: "/api/gdpr/data-export", desc: "Export user data" },
      { method: "GET", path: "/api/gdpr/requests", desc: "GDPR request status" },
      { method: "DELETE", path: "/api/gdpr/delete-account", desc: "Request account deletion" },
    ],
  },
  {
    id: "system",
    name: "System / Health",
    description: "Health checks, platform metrics, post-quantum readiness, and legal documents",
    icon: "Activity",
    color: "text-lime-600",
    kong: {
      serviceName: "plenumnet-health",
      routePath: "/api/health",
      tags: ["plenumnet", "health", "metrics", "system"],
      rateLimit: { minute: 200, hour: 2000 },
      methods: ["GET", "POST"],
    },
    endpoints: [
      { method: "GET", path: "/api/health", desc: "System health check", tryIt: true },
      { method: "GET", path: "/api/metrics/plenum", desc: "Platform metrics", tryIt: true },
      { method: "GET", path: "/api/pqti-status", desc: "Post-quantum transition readiness", tryIt: true },
      { method: "GET", path: "/api/verify", desc: "Verification endpoint", tryIt: true },
      { method: "GET", path: "/api/user/admin-status", desc: "Check admin status", tryIt: true },
      { method: "GET", path: "/api/legal/:type", desc: "Legal documents (terms, privacy, security, aup)" },
      { method: "POST", path: "/api/developer-signup", desc: "Developer waitlist signup" },
      { method: "GET", path: "/api/developer-signup/count", desc: "Waitlist count", tryIt: true },
    ],
  },
  {
    id: "admin",
    name: "Admin",
    description: "Administrative endpoints requiring authentication",
    icon: "Gauge",
    color: "text-zinc-600",
    kong: {
      serviceName: "plenumnet-admin",
      routePath: "/api/admin",
      tags: ["plenumnet", "admin"],
      rateLimit: { minute: 30, hour: 300 },
      methods: ["GET", "DELETE"],
    },
    endpoints: [
      { method: "GET", path: "/api/admin/developer-signups", desc: "All developer signups", admin: true },
      { method: "DELETE", path: "/api/admin/developer-signups/:id", desc: "Remove signup", admin: true },
    ],
  },
];

export const DOMAIN_GROUPS: DomainGroupDef[] = [
  {
    id: "compute",
    name: "Core Ternary Computing",
    description: "Foundational compute primitives: GF(3) arithmetic, ternary VM, and compression engine",
    serviceIds: ["ternary", "vm", "compression", "demo"],
  },
  {
    id: "timing",
    name: "Timing & Calendars",
    description: "Femtosecond-precision HPTP timestamps and 42-calendar epoch synchronization",
    serviceIds: ["timing", "calendar"],
  },
  {
    id: "trust",
    name: "Cryptography & Trust",
    description: "Proof-of-existence pipeline: phase encryption, notary timestamping, blockchain witnessing",
    serviceIds: ["phase", "tsa", "hedera"],
  },
  {
    id: "security",
    name: "Security & Access Control",
    description: "Capability tokens, security audit infrastructure, HPTP anomaly detection, and API key management",
    serviceIds: ["capabilities", "security", "api-keys"],
  },
  {
    id: "network",
    name: "Network & Infrastructure",
    description: "TDNS addressing, geometric routing, operations pipeline, and API gateway management",
    serviceIds: ["tdns", "inter-cube", "sfk", "kong"],
  },
  {
    id: "signals",
    name: "Signals & Applied Physics",
    description: "Tonal diffusion, resonance detection, ternary ephemeris, agent array, and entrainment",
    serviceIds: ["tonal", "ephemeris", "tribonacci", "v1"],
  },
  {
    id: "platform",
    name: "Platform Operations",
    description: "Developer tooling, documentation, compliance, and system health",
    serviceIds: ["github", "whitepapers", "gdpr", "system", "admin"],
  },
];

export const SERVICE_MAP = new Map(SERVICE_CATALOG.map(s => [s.id, s]));

export function getTotalEndpoints(): number {
  return SERVICE_CATALOG.reduce((sum, s) => sum + s.endpoints.length, 0);
}

export function getTotalServices(): number {
  return SERVICE_CATALOG.length;
}

export function getKongServiceDefs(baseUrl: string) {
  return SERVICE_CATALOG.map(s => ({
    name: s.kong.serviceName,
    url: `${baseUrl}${s.kong.routePath}`,
    tags: s.kong.tags,
    routePath: s.kong.routePath,
    stripPath: false,
    rateLimit: s.kong.rateLimit,
    methods: s.kong.methods,
    endpointCount: s.endpoints.length,
    endpoints: s.endpoints.map(ep => {
      const relative = ep.path.startsWith(s.kong.routePath)
        ? ep.path.slice(s.kong.routePath.length) || "/"
        : ep.path;
      return `${ep.method} ${relative}`;
    }),
  }));
}

export type KongCatalogCategory = "core" | "tools" | "reference" | "platform" | "admin";

const KONG_CATEGORY_MAP: Record<string, KongCatalogCategory> = {
  timing: "core",
  calendar: "core",
  ternary: "core",
  phase: "core",
  vm: "core",
  tdns: "core",
  "inter-cube": "core",
  capabilities: "core",
  compression: "tools",
  demo: "tools",
  tsa: "tools",
  hedera: "tools",
  sfk: "tools",
  security: "tools",
  "api-keys": "tools",
  tribonacci: "tools",
  tonal: "tools",
  ephemeris: "reference",
  whitepapers: "reference",
  v1: "reference",
  kong: "platform",
  github: "platform",
  gdpr: "platform",
  system: "platform",
  admin: "admin",
};

export function getKongServiceCatalog(baseUrl: string) {
  const categories: Record<KongCatalogCategory, any[]> = {
    core: [],
    tools: [],
    reference: [],
    platform: [],
    admin: [],
  };

  const allServices = SERVICE_CATALOG.map(s => {
    const cat = KONG_CATEGORY_MAP[s.id] || "platform";
    const entry = {
      name: s.kong.serviceName,
      label: s.name,
      routePath: s.kong.routePath,
      endpointCount: s.endpoints.length,
      category: cat,
      endpoints: s.endpoints.map(ep => {
        const relative = ep.path.startsWith(s.kong.routePath)
          ? ep.path.slice(s.kong.routePath.length) || "/"
          : ep.path;
        return `${ep.method} ${relative}`;
      }),
    };
    categories[cat].push(entry);
    return entry;
  });

  return {
    totalServices: SERVICE_CATALOG.length,
    totalEndpoints: getTotalEndpoints(),
    baseUrl,
    categories,
    services: allServices,
  };
}
