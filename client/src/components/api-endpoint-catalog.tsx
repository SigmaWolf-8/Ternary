import { useState, useCallback } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Clock, Calculator, Shield, Database, TrendingUp, Cpu, Key,
  Globe, Network, Radio, Star, FileText, Activity, Lock,
  Search, Play, RefreshCw, ChevronDown, ChevronRight, Copy, Check,
  Server, GitBranch, Scale, Heart, Gauge, BookOpen, Layers, Workflow,
  Stamp, Link2, Hexagon
} from "lucide-react";
import { PLATFORM } from "@shared/constants";

type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

interface Endpoint {
  method: HttpMethod;
  path: string;
  desc: string;
  tryIt?: boolean;
  admin?: boolean;
}

interface EndpointCategory {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  endpoints: Endpoint[];
}

const METHOD_COLORS: Record<HttpMethod, string> = {
  GET: "outline",
  POST: "secondary",
  PUT: "default",
  PATCH: "default",
  DELETE: "destructive",
};

const CATEGORIES: EndpointCategory[] = [
  {
    id: "timing",
    name: "Timing API",
    description: "Femtosecond-precision timestamps and Salvi Epoch synchronization",
    icon: <Clock className="w-4 h-4" />,
    color: "text-blue-600",
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
    icon: <Globe className="w-4 h-4" />,
    color: "text-indigo-600",
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
    icon: <Calculator className="w-4 h-4" />,
    color: "text-green-600",
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
    icon: <Shield className="w-4 h-4" />,
    color: "text-red-600",
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
    icon: <Lock className="w-4 h-4" />,
    color: "text-amber-600",
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
    icon: <Stamp className="w-4 h-4" />,
    color: "text-emerald-700",
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
    icon: <Link2 className="w-4 h-4" />,
    color: "text-indigo-700",
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
    description: "Salvi Framework Kernel operation lifecycle: init → ternary processing → witnessing → finalization",
    icon: <Workflow className="w-4 h-4" />,
    color: "text-rose-700",
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
    icon: <Hexagon className="w-4 h-4" />,
    color: "text-cyan-700",
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
    id: "security",
    name: "Security Infrastructure",
    description: "Audit service, HPTP anomaly detection, threat model registry, and security dashboard",
    icon: <Shield className="w-4 h-4" />,
    color: "text-rose-600",
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
    icon: <Key className="w-4 h-4" />,
    color: "text-yellow-600",
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
    icon: <Layers className="w-4 h-4" />,
    color: "text-violet-600",
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
    name: "Compression",
    description: "Live ternary compression demos and database-backed document storage",
    icon: <Database className="w-4 h-4" />,
    color: "text-teal-600",
    endpoints: [
      { method: "POST", path: "/api/demo/run", desc: "Run compression on dataset" },
      { method: "POST", path: "/api/demo/upload", desc: "Upload custom data for compression" },
      { method: "GET", path: "/api/demo/stats", desc: "Aggregated compression statistics", tryIt: true },
      { method: "GET", path: "/api/demo/session/:sessionId", desc: "Session details" },
      { method: "GET", path: "/api/demo/data/:sessionId", desc: "Session data export" },
      { method: "GET", path: "/api/demo/history", desc: "Compression run history", tryIt: true },
      { method: "GET", path: "/api/demo/files", desc: "Available demo files", tryIt: true },
      { method: "POST", path: "/api/compression/file", desc: "Compress/decompress file" },
      { method: "POST", path: "/api/compression/decompress", desc: "Decompress data" },
      { method: "POST", path: "/api/compression/db/store", desc: "Store compressed data" },
      { method: "GET", path: "/api/compression/db/retrieve/:id", desc: "Retrieve compressed document" },
      { method: "GET", path: "/api/compression/db/documents", desc: "List compressed documents", tryIt: true },
      { method: "GET", path: "/api/compression/db/raw/:id", desc: "Raw stored data" },
      { method: "DELETE", path: "/api/compression/db/documents/:id", desc: "Delete compressed document" },
    ],
  },
  {
    id: "vm",
    name: "Virtual Machine",
    description: `Ternary VM with ${PLATFORM.VM_OPCODES}-opcode ISA ${PLATFORM.VM_ISA_VERSION}, 27 registers, and conformance testing`,
    icon: <Cpu className="w-4 h-4" />,
    color: "text-cyan-600",
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
    icon: <Server className="w-4 h-4" />,
    color: "text-emerald-600",
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
    icon: <GitBranch className="w-4 h-4" />,
    color: "text-gray-600",
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
    icon: <Radio className="w-4 h-4" />,
    color: "text-orange-600",
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
    icon: <Star className="w-4 h-4" />,
    color: "text-purple-600",
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
    icon: <BookOpen className="w-4 h-4" />,
    color: "text-sky-600",
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
    icon: <TrendingUp className="w-4 h-4" />,
    color: "text-fuchsia-600",
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
    icon: <Scale className="w-4 h-4" />,
    color: "text-slate-600",
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
    icon: <Activity className="w-4 h-4" />,
    color: "text-lime-600",
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
    icon: <Gauge className="w-4 h-4" />,
    color: "text-zinc-600",
    endpoints: [
      { method: "GET", path: "/api/admin/developer-signups", desc: "All developer signups", admin: true },
      { method: "DELETE", path: "/api/admin/developer-signups/:id", desc: "Remove signup", admin: true },
    ],
  },
];

function TryItPanel({ endpoint }: { endpoint: Endpoint }) {
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await fetch(endpoint.path);
      const text = await res.text();
      try {
        setResult(JSON.parse(text));
      } catch {
        setResult(text);
      }
    } catch (err: any) {
      setError(err.message || "Request failed");
    } finally {
      setLoading(false);
    }
  }, [endpoint.path]);

  return (
    <div className="mt-2 space-y-2">
      <Button
        size="sm"
        variant="outline"
        onClick={execute}
        disabled={loading}
        className="h-7 text-xs"
        data-testid={`tryit-${endpoint.path.replace(/\//g, "-").slice(1)}`}
      >
        {loading ? (
          <RefreshCw className="w-3 h-3 mr-1 animate-spin" />
        ) : (
          <Play className="w-3 h-3 mr-1" />
        )}
        Try it
      </Button>
      {error && (
        <div className="text-xs text-destructive bg-destructive/10 rounded px-2 py-1">
          {error}
        </div>
      )}
      {result && (
        <ScrollArea className="max-h-48">
          <pre className="text-xs bg-muted/50 rounded p-2 whitespace-pre-wrap break-all font-mono" data-testid={`result-${endpoint.path.replace(/\//g, "-").slice(1)}`}>
            {typeof result === "string" ? result : JSON.stringify(result, null, 2)}
          </pre>
        </ScrollArea>
      )}
    </div>
  );
}

function EndpointRow({ endpoint }: { endpoint: Endpoint }) {
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const copyPath = () => {
    navigator.clipboard.writeText(endpoint.path);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className="group border-b last:border-b-0 border-border/50 py-2">
      <div className="flex items-start gap-2">
        <Badge
          variant={METHOD_COLORS[endpoint.method] as any}
          className="shrink-0 text-xs font-mono w-16 justify-center"
        >
          {endpoint.method}
        </Badge>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1">
            <code className="text-xs break-all font-mono">{endpoint.path}</code>
            {endpoint.admin && (
              <Badge variant="outline" className="shrink-0 text-[10px] h-4 px-1 border-amber-300 text-amber-600">
                admin
              </Badge>
            )}
          </div>
          <div className="text-xs text-muted-foreground mt-0.5">{endpoint.desc}</div>
        </div>
        <div className="flex items-center gap-1 shrink-0">
          <Button
            size="icon"
            variant="ghost"
            onClick={copyPath}
            data-testid={`copy-${endpoint.path.replace(/\//g, "-").slice(1)}`}
          >
            {copied ? <Check className="w-3 h-3 text-green-500" /> : <Copy className="w-3 h-3" />}
          </Button>
          {endpoint.tryIt && endpoint.method === "GET" && !endpoint.path.includes(":") && (
            <Button
              size="icon"
              variant="ghost"
              onClick={() => setExpanded(!expanded)}
              data-testid={`expand-${endpoint.path.replace(/\//g, "-").slice(1)}`}
            >
              {expanded ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
            </Button>
          )}
        </div>
      </div>
      {expanded && endpoint.tryIt && endpoint.method === "GET" && !endpoint.path.includes(":") && (
        <TryItPanel endpoint={endpoint} />
      )}
    </div>
  );
}

function CategoryCard({ category, defaultOpen }: { category: EndpointCategory; defaultOpen: boolean }) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const getCount = category.endpoints.length;
  const adminCount = category.endpoints.filter(e => e.admin).length;
  const publicCount = getCount - adminCount;

  return (
    <Card data-testid={`category-${category.id}`}>
      <CardHeader
        className="pb-2 cursor-pointer select-none"
        onClick={() => setIsOpen(!isOpen)}
        data-testid={`toggle-category-${category.id}`}
      >
        <div className="flex items-center justify-between">
          <CardTitle className={`text-base flex items-center gap-2 ${category.color}`}>
            {category.icon}
            {category.name}
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="text-xs">
              {publicCount} public{adminCount > 0 ? ` + ${adminCount} admin` : ""}
            </Badge>
            {isOpen ? <ChevronDown className="w-4 h-4 text-muted-foreground" /> : <ChevronRight className="w-4 h-4 text-muted-foreground" />}
          </div>
        </div>
        <CardDescription>{category.description}</CardDescription>
      </CardHeader>
      {isOpen && (
        <CardContent>
          <div>
            {category.endpoints.map((ep, i) => (
              <EndpointRow key={`${ep.method}-${ep.path}-${i}`} endpoint={ep} />
            ))}
          </div>
        </CardContent>
      )}
    </Card>
  );
}

export default function APIEndpointCatalog() {
  const [search, setSearch] = useState("");
  const [activeFilter, setActiveFilter] = useState<string | null>(null);

  const totalEndpoints = CATEGORIES.reduce((sum, cat) => sum + cat.endpoints.length, 0);

  const filteredCategories = CATEGORIES.map(cat => {
    if (activeFilter && cat.id !== activeFilter) return null;
    if (!search) return cat;
    const lower = search.toLowerCase();
    const filteredEndpoints = cat.endpoints.filter(
      ep => ep.path.toLowerCase().includes(lower) || ep.desc.toLowerCase().includes(lower) || ep.method.toLowerCase().includes(lower)
    );
    if (filteredEndpoints.length === 0 && !cat.name.toLowerCase().includes(lower)) return null;
    return { ...cat, endpoints: filteredEndpoints.length > 0 ? filteredEndpoints : cat.endpoints };
  }).filter(Boolean) as EndpointCategory[];

  const filteredCount = filteredCategories.reduce((sum, cat) => sum + cat.endpoints.length, 0);

  return (
    <div className="space-y-6" data-testid="api-endpoint-catalog">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold" data-testid="text-catalog-title">
            Complete API Reference
          </h2>
          <p className="text-muted-foreground text-sm mt-1">
            {totalEndpoints} endpoints across {CATEGORIES.length} services.
            {search || activeFilter ? ` Showing ${filteredCount} endpoints.` : ""}
            {" "}Hover any endpoint to copy its path or try GET endpoints live.
          </p>
        </div>
        <div className="relative w-full md:w-72">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <Input
            placeholder="Search endpoints..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
            data-testid="input-search-endpoints"
          />
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          variant={activeFilter === null ? "default" : "outline"}
          onClick={() => setActiveFilter(null)}
          className="h-7 text-xs"
          data-testid="filter-all"
        >
          All ({totalEndpoints})
        </Button>
        {CATEGORIES.map(cat => (
          <Button
            key={cat.id}
            size="sm"
            variant={activeFilter === cat.id ? "default" : "outline"}
            onClick={() => setActiveFilter(activeFilter === cat.id ? null : cat.id)}
            className="h-7 text-xs"
            data-testid={`filter-${cat.id}`}
          >
            {cat.name} ({cat.endpoints.length})
          </Button>
        ))}
      </div>

      <div className="space-y-4">
        {filteredCategories.map(cat => (
          <CategoryCard
            key={cat.id}
            category={cat}
            defaultOpen={activeFilter === cat.id || filteredCategories.length <= 3}
          />
        ))}
        {filteredCategories.length === 0 && (
          <div className="text-center py-12 text-muted-foreground" data-testid="text-no-results">
            No endpoints match your search.
          </div>
        )}
      </div>
    </div>
  );
}
