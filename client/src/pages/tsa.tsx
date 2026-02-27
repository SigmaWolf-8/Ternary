/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Shield,
  ShieldCheck,
  Clock,
  Check,
  ArrowRight,
  Lock,
  Key,
  FileSignature,
  Hash,
  Layers,
  Globe,
  Calendar,
  AlertTriangle,
  ExternalLink,
  Fingerprint,
  Scale,
  Gavel,
  Building2,
  CircleDot,
  Radio,
  BadgeCheck,
  ListChecks,
} from "lucide-react";
import { motion } from "framer-motion";
import { Link } from "wouter";
import { useQuery } from "@tanstack/react-query";

const POLICY_TIERS = [
  {
    tier: "DEFAULT",
    oid: "1.3.6.1.4.1.0.100.1.0",
    name: "General BTSP",
    icon: Clock,
    color: "text-muted-foreground",
    bgColor: "bg-muted/50",
    borderColor: "border-muted-foreground/20",
    accuracy: "1 second",
    ordering: false,
    retention: "7 years",
    calendars: "None",
    calendarCount: 0,
    description: "General-purpose timestamping with 1-second declared accuracy. Suitable for routine document timestamping and audit trail creation.",
    useCase: "Internal records, general document notarization",
    whitepaperRef: "N/A",
  },
  {
    tier: "COMPLY",
    oid: "1.3.6.1.4.1.0.100.1.1",
    name: "Financial Compliance",
    icon: Scale,
    color: "text-blue-500 dark:text-blue-400",
    bgColor: "bg-blue-500/10",
    borderColor: "border-blue-500/20",
    accuracy: "±1 \u00B5s",
    ordering: true,
    retention: "10 years",
    calendars: "11 financial-market calendars",
    calendarCount: 11,
    description: "FINRA Rule 613 / CAT nanosecond truncation compliance. HPTP native nanosecond resolution logged. Auto-embeds 11 financial-market calendar systems.",
    useCase: "FINRA CAT, MiFID II, trade surveillance, regulatory reporting",
    whitepaperRef: "\u00A78.1",
  },
  {
    tier: "FORENSICS",
    oid: "1.3.6.1.4.1.0.100.1.2",
    name: "Legal Evidence",
    icon: Gavel,
    color: "text-amber-500 dark:text-amber-400",
    bgColor: "bg-amber-500/10",
    borderColor: "border-amber-500/20",
    accuracy: "±100 \u00B5s",
    ordering: true,
    retention: "25 years",
    calendars: "All 42 calendar systems",
    calendarCount: 42,
    description: "Digital Evidence Vault timestamps. Court-admissible. Offline verification. eDiscovery integration. Embeds all 42 calendar systems for jurisdictional completeness.",
    useCase: "Litigation holds, chain of custody, court evidence, eDiscovery",
    whitepaperRef: "\u00A78.2",
  },
  {
    tier: "SENTINEL",
    oid: "1.3.6.1.4.1.0.100.1.3",
    name: "Government / Military",
    icon: Shield,
    color: "text-red-500 dark:text-red-400",
    bgColor: "bg-red-500/10",
    borderColor: "border-red-500/20",
    accuracy: "±1 \u00B5s",
    ordering: true,
    retention: "50 years",
    calendars: "On request",
    calendarCount: 0,
    description: "Ordering guaranteed for timeline reconstruction. Centralized Timeline Viewer support. Maximum retention for government/military audit requirements.",
    useCase: "Intelligence agencies, military operations, classified document management",
    whitepaperRef: "\u00A78.3",
  },
  {
    tier: "SECURE",
    oid: "1.3.6.1.4.1.0.100.1.4",
    name: "Enterprise Zero-Trust",
    icon: Lock,
    color: "text-emerald-500 dark:text-emerald-400",
    bgColor: "bg-emerald-500/10",
    borderColor: "border-emerald-500/20",
    accuracy: "±10 \u00B5s",
    ordering: true,
    retention: "10 years",
    calendars: "On request",
    calendarCount: 0,
    description: "ZTNA temporal anomaly detection. Secure Collaboration Enclaves with verifiable TSTs on every access request.",
    useCase: "Zero-Trust architectures, secure enclaves, temporal access control",
    whitepaperRef: "\u00A78.4",
  },
];

const ENDPOINTS = [
  {
    method: "POST",
    path: "/api/tsa/timestamp",
    auth: "Bearer (app)",
    contentType: "application/timestamp-query",
    description: "Binary RFC 3161 timestamp request. Send a DER-encoded TimeStampReq, receive a DER-encoded TimeStampResp.",
  },
  {
    method: "POST",
    path: "/api/tsa/timestamp/json",
    auth: "Bearer (app)",
    contentType: "application/json",
    description: "JSON timestamp request. Send hash + algorithm, receive structured response with token, serial, policy, and calendar context.",
  },
  {
    method: "POST",
    path: "/api/tsa/verify",
    auth: "Public",
    contentType: "application/json",
    description: "Verify a timestamp token. Accepts binary timestamp-reply or JSON with base64-encoded token.",
  },
  {
    method: "GET",
    path: "/api/tsa/certificate",
    auth: "Public",
    contentType: "\u2014",
    description: "Retrieve the TSA signing certificate details: subject, issuer, validity, serial, and fingerprints.",
  },
  {
    method: "GET",
    path: "/api/tsa/certificate/download",
    auth: "Public",
    contentType: "\u2014",
    description: "Download the TSA certificate as a PEM file for offline verification.",
  },
  {
    method: "GET",
    path: "/api/tsa/policy",
    auth: "Public",
    contentType: "\u2014",
    description: "List all policy tiers with OIDs, accuracy, ordering, retention, and descriptions.",
  },
  {
    method: "GET",
    path: "/api/tsa/health",
    auth: "Public",
    contentType: "\u2014",
    description: "Service health check: certificate validity, key availability, serial counter, uptime.",
  },
  {
    method: "GET",
    path: "/api/tsa/audit/query",
    auth: "Bearer (readonly)",
    contentType: "\u2014",
    description: "Query the Merkle-backed audit log. Filter by serial, time range, hash algorithm, policy tier.",
  },
];

const HASH_ALGORITHMS = [
  { name: "SHA-256", oid: "2.16.840.1.101.3.4.2.1", bits: 256 },
  { name: "SHA-384", oid: "2.16.840.1.101.3.4.2.2", bits: 384 },
  { name: "SHA-512", oid: "2.16.840.1.101.3.4.2.3", bits: 512 },
  { name: "SHA3-256", oid: "2.16.840.1.101.3.4.2.8", bits: 256 },
  { name: "SHA3-384", oid: "2.16.840.1.101.3.4.2.9", bits: 384 },
  { name: "SHA3-512", oid: "2.16.840.1.101.3.4.2.10", bits: 512 },
];

const SECURITY_FEATURES = [
  {
    icon: Fingerprint,
    title: "Dual Signature",
    description: "RSA-4096 classical + post-quantum TL-DSA. Quantum-resistant from day one.",
  },
  {
    icon: Layers,
    title: "Merkle Audit Log",
    description: "Tamper-evident append-only log. Every token anchored in a cryptographic hash tree.",
  },
  {
    icon: Clock,
    title: "HPTP Timing",
    description: "Femtosecond-precision timestamps via the High-Precision Timing Protocol. Algorithms resolve to femtosecond granularity; hardware resolution to nanoseconds without paired atomic clock or GPS/GNSS receiver.",
  },
  {
    icon: FileSignature,
    title: "CMS SignedData",
    description: "RFC 5652 compliant with SET-encoded signed attributes for strict verification.",
  },
  {
    icon: Globe,
    title: "42 Calendar Systems",
    description: "Calendar Context Extension (OID 1.3.6.1.4.1.0.100.2.1) embeds multi-calendar timestamps.",
  },
  {
    icon: Key,
    title: "ASN.1 Wire Protocol",
    description: "Full ASN.1 DER encoding via asn1js. Standards-compliant binary transport.",
  },
];

const COMPLY_CALENDARS = [
  "Islamic Hijri", "Persian / Solar Hijri", "Hebrew", "Japanese Imperial",
  "Thai Buddhist Era", "Chinese Sexagenary", "Indian National (Saka)",
  "Vikram Samvat", "Korean (Dangun Era)", "Nanakshahi (Sikh)", "Minguo",
];

interface TsaHealth {
  status: string;
  tsaCertSubject: string;
  tsaCertExpiry: string;
  tsaKeyLoaded: boolean;
  tsaCertValid: boolean;
  serialNumber: string;
  uptime: number;
  tokensIssuedLast24h: number;
  dualSignEnabled: boolean;
  merkleTreeDepth: number;
  hptpAvailable: boolean;
}

function HeroSection() {
  return (
    <section className="relative overflow-hidden py-20 md:py-28" data-testid="section-tsa-hero">
      <div className="absolute inset-0 bg-gradient-to-b from-primary/5 via-transparent to-transparent pointer-events-none" />
      <div className="max-w-5xl mx-auto px-5 text-center relative z-10">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
        >
          <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
            Kong Service #21 \u00B7 RFC 3161
          </Badge>
        </motion.div>
        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
          className="text-3xl md:text-5xl font-bold tracking-tight mb-4"
          data-testid="text-tsa-title"
        >
          Time-Stamping Authority
        </motion.h1>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.15 }}
          className="text-lg text-muted-foreground max-w-3xl mx-auto mb-3"
          data-testid="text-tsa-subtitle"
        >
          Cryptographic proof-of-existence timestamps implementing RFC 3161, RFC 5652, and RFC 5816 with RSA-4096 and post-quantum TL-DSA dual signatures. Independently verifiable via OpenSSL.
        </motion.p>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="text-sm text-muted-foreground max-w-2xl mx-auto mb-8"
          data-testid="text-tsa-standards"
        >
          Designed for ETSI EN 319 421/422 conformance \u00B7 RFC 5816 \u00B7 RFC 5652 (CMS) \u00B7 HPTP Femtosecond Precision \u00B7 Post-Quantum TL-DSA
        </motion.p>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.25 }}
          className="flex flex-wrap justify-center gap-3"
        >
          <Button asChild data-testid="button-tsa-api-explorer">
            <Link href="/api-demo">
              API Explorer
              <ArrowRight className="w-4 h-4 ml-1" />
            </Link>
          </Button>
          <Button variant="outline" asChild data-testid="button-tsa-docs">
            <Link href="/docs">
              Documentation
              <ExternalLink className="w-4 h-4 ml-1" />
            </Link>
          </Button>
        </motion.div>
      </div>
    </section>
  );
}

function SecurityFeaturesSection() {
  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-tsa-security">
      <div className="max-w-6xl mx-auto px-5">
        <div className="text-center mb-10">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Security Architecture
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-security-title"
          >
            Dual-Signature, Post-Quantum Ready
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            Every timestamp token carries both RSA-4096 and post-quantum TL-DSA signatures,
            anchored in a Merkle tamper-evident audit log.
          </motion.p>
        </div>

        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 max-w-5xl mx-auto">
          {SECURITY_FEATURES.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: index * 0.08 }}
            >
              <Card className="p-5 h-full" data-testid={`card-security-feature-${index}`}>
                <div className="flex items-center gap-3 mb-3">
                  <div className="w-9 h-9 rounded-md bg-primary/10 flex items-center justify-center flex-shrink-0">
                    <feature.icon className="w-4 h-4 text-primary" />
                  </div>
                  <h3 className="font-semibold text-sm">{feature.title}</h3>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed">{feature.description}</p>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

const LIVE_CAPABILITIES = [
  "RFC 3161 compliant timestamp tokens with real RSA-4096 signatures",
  "Standards-compliant CMS SignedData wire format (RFC 5652)",
  "Dual-signature: RSA-4096 classical + post-quantum TL-DSA",
  "Merkle tamper-evident audit log with cryptographic hash tree",
  "5 policy tiers with unique OIDs and purpose-built configurations",
  "42 calendar system enrichment via Calendar Context Extension",
  "Independent verification via openssl ts -verify (with supplied CA file)",
  "6 supported hash algorithms (SHA-2 and SHA-3 families)",
];

const ROADMAP_ITEMS = [
  {
    title: "IANA Private Enterprise Number",
    description: "Register a PEN under the Capomastro Holdings arc to make policy OIDs globally unique. Currently uses PEN 0 (unregistered). IANA registration is free and takes a few weeks.",
    status: "pending" as const,
    effort: "Administrative",
  },
  {
    title: "CA-Chained Certificate",
    description: "Replace the self-signed TSA certificate with one issued under a publicly trusted CA chain with the id-kp-timeStamping EKU. This enables openssl ts -verify to work against system trust stores without requiring -CAfile.",
    status: "pending" as const,
    effort: "Administrative",
  },
  {
    title: "Traceable Time Source",
    description: "Pair with a traceable time source (GPS/GNSS disciplined oscillator, atomic clock, or stratum-1 NTP with documented uncertainty budget) to achieve full femtosecond hardware resolution. Without paired hardware, effective resolution is nanoseconds via process.hrtime().",
    status: "pending" as const,
    effort: "Infrastructure",
  },
];

function ComplianceStatusSection() {
  return (
    <section className="py-16 md:py-20" data-testid="section-tsa-compliance-status">
      <div className="max-w-6xl mx-auto px-5">
        <div className="text-center mb-10">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Implementation Status
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-compliance-status-title"
          >
            What Is Live Today
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-3xl mx-auto"
          >
            PlenumNET operates a cryptographic Time-Stamping Authority. The code is real, the infrastructure is real. The gap to regulated-grade TSA status is administrative, not technical.
          </motion.p>
        </div>

        <div className="max-w-5xl mx-auto grid lg:grid-cols-2 gap-6">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Card className="p-5 h-full border-emerald-500/20" data-testid="card-live-capabilities">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-9 h-9 rounded-md bg-emerald-500/10 flex items-center justify-center flex-shrink-0">
                  <ListChecks className="w-4 h-4 text-emerald-500" />
                </div>
                <div>
                  <h3 className="font-semibold text-sm">Live Capabilities</h3>
                  <p className="text-[10px] text-muted-foreground">Operational and verifiable today</p>
                </div>
              </div>
              <div className="space-y-2">
                {LIVE_CAPABILITIES.map((cap, index) => (
                  <div key={index} className="flex items-start gap-2 text-xs" data-testid={`text-live-cap-${index}`}>
                    <Check className="w-3.5 h-3.5 text-emerald-500 flex-shrink-0 mt-0.5" />
                    <span className="text-muted-foreground">{cap}</span>
                  </div>
                ))}
              </div>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
          >
            <Card className="p-5 h-full border-amber-500/20" data-testid="card-roadmap">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-9 h-9 rounded-md bg-amber-500/10 flex items-center justify-center flex-shrink-0">
                  <Radio className="w-4 h-4 text-amber-500" />
                </div>
                <div>
                  <h3 className="font-semibold text-sm">Path to Regulated-Grade TSA</h3>
                  <p className="text-[10px] text-muted-foreground">Administrative steps remaining</p>
                </div>
              </div>
              <div className="space-y-4">
                {ROADMAP_ITEMS.map((item, index) => (
                  <div key={index} className="space-y-1" data-testid={`card-roadmap-item-${index}`}>
                    <div className="flex items-center gap-2">
                      <CircleDot className="w-3.5 h-3.5 text-amber-500 flex-shrink-0" />
                      <span className="text-xs font-medium text-foreground">{item.title}</span>
                      <Badge variant="outline" className="text-[9px] ml-auto">{item.effort}</Badge>
                    </div>
                    <p className="text-[11px] text-muted-foreground pl-5 leading-relaxed">{item.description}</p>
                  </div>
                ))}
              </div>
            </Card>
          </motion.div>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="mt-6 max-w-5xl mx-auto"
        >
          <Card className="p-5 border-primary/20" data-testid="card-conformance-note">
            <div className="flex items-start gap-3">
              <div className="w-9 h-9 rounded-md bg-primary/10 flex items-center justify-center flex-shrink-0">
                <BadgeCheck className="w-4 h-4 text-primary" />
              </div>
              <div>
                <h3 className="font-semibold text-sm mb-1">Standards Conformance</h3>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  This system is designed for ETSI EN 319 421/422 conformance pending external audit and CA-chained certificate issuance.
                  The certificate is currently self-signed. Courts, financial regulators, and eIDAS recognize timestamps from TSAs whose
                  certificates chain to a trusted root CA. ETSI conformance claims require assessment by an accredited conformity assessment body.
                </p>
              </div>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function PolicyTiersSection() {
  return (
    <section className="py-16 md:py-20" data-testid="section-tsa-policies">
      <div className="max-w-6xl mx-auto px-5">
        <div className="text-center mb-10">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              5 Policy Tiers
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-policies-title"
          >
            Purpose-Built Timestamp Policies
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            Each policy tier is identified by a unique OID and tailored for specific regulatory,
            forensic, or enterprise requirements.
          </motion.p>
        </div>

        <div className="space-y-4 max-w-5xl mx-auto">
          {POLICY_TIERS.map((policy, index) => (
            <motion.div
              key={policy.tier}
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: index * 0.1 }}
            >
              <Card className={`p-5 border ${policy.borderColor}`} data-testid={`card-policy-${policy.tier.toLowerCase()}`}>
                <div className="flex flex-col lg:flex-row lg:items-start gap-4">
                  <div className="flex items-center gap-3 lg:w-64 flex-shrink-0">
                    <div className={`w-10 h-10 rounded-md ${policy.bgColor} flex items-center justify-center flex-shrink-0`}>
                      <policy.icon className={`w-5 h-5 ${policy.color}`} />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <h3 className="font-bold text-sm">{policy.tier}</h3>
                        {policy.whitepaperRef !== "N/A" && (
                          <span className="text-[10px] text-muted-foreground">{policy.whitepaperRef}</span>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground">{policy.name}</p>
                    </div>
                  </div>

                  <div className="flex-1 space-y-3">
                    <p className="text-xs text-foreground leading-relaxed">{policy.description}</p>
                    <div className="flex flex-wrap gap-x-6 gap-y-2 text-xs">
                      <div>
                        <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block">Accuracy</span>
                        <span className="font-mono text-foreground">{policy.accuracy}</span>
                      </div>
                      <div>
                        <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block">Ordering</span>
                        <span className="text-foreground">{policy.ordering ? "Guaranteed" : "Best-effort"}</span>
                      </div>
                      <div>
                        <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block">Retention</span>
                        <span className="text-foreground">{policy.retention}</span>
                      </div>
                      <div>
                        <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block">Calendars</span>
                        <span className="text-foreground">{policy.calendars}</span>
                      </div>
                    </div>
                    <div>
                      <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-0.5">Use Case</span>
                      <span className="text-xs text-muted-foreground">{policy.useCase}</span>
                    </div>
                    <div>
                      <span className="text-[10px] font-mono text-muted-foreground">OID: {policy.oid}</span>
                    </div>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function CalendarContextSection() {
  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-tsa-calendars">
      <div className="max-w-6xl mx-auto px-5">
        <div className="text-center mb-10">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Calendar Context Extension
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-calendars-title"
          >
            Multi-Calendar Timestamp Enrichment
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            A non-critical ASN.1 extension embeds culturally and jurisdictionally relevant
            calendar representations directly into each timestamp token.
          </motion.p>
        </div>

        <div className="max-w-5xl mx-auto grid lg:grid-cols-2 gap-6">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Card className="p-5 h-full" data-testid="card-calendar-extension">
              <div className="flex items-center gap-2 mb-4">
                <div className="w-8 h-8 rounded-md bg-primary/10 flex items-center justify-center">
                  <Calendar className="w-4 h-4 text-primary" />
                </div>
                <div>
                  <h3 className="font-semibold text-sm">Extension Details</h3>
                  <p className="text-[10px] text-muted-foreground font-mono">OID 1.3.6.1.4.1.0.100.2.1</p>
                </div>
              </div>
              <div className="space-y-3 text-xs">
                <div>
                  <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Encoding</span>
                  <p className="text-muted-foreground">Non-critical ASN.1 extension in TSTInfo. Compressed CBOR serialization with zlib deflate for bandwidth efficiency.</p>
                </div>
                <div>
                  <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Behavior</span>
                  <p className="text-muted-foreground">Best-effort enrichment. Calendar conversion failure never blocks token issuance. Request-level <code className="text-foreground font-mono bg-muted px-1 rounded">calendars[]</code> array is additive to policy defaults.</p>
                </div>
                <div>
                  <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Wildcard</span>
                  <p className="text-muted-foreground">Pass <code className="text-foreground font-mono bg-muted px-1 rounded">["*"]</code> to embed all 42 calendar systems regardless of policy tier.</p>
                </div>
              </div>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
          >
            <Card className="p-5 h-full" data-testid="card-comply-calendars">
              <div className="flex items-center gap-2 mb-4">
                <div className="w-8 h-8 rounded-md bg-blue-500/10 flex items-center justify-center">
                  <Building2 className="w-4 h-4 text-blue-500 dark:text-blue-400" />
                </div>
                <div>
                  <h3 className="font-semibold text-sm">COMPLY Auto-Embed</h3>
                  <p className="text-[10px] text-muted-foreground">11 financial-market calendars</p>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-1.5">
                {COMPLY_CALENDARS.map((cal) => (
                  <div key={cal} className="flex items-center gap-1.5 text-xs text-muted-foreground">
                    <Check className="w-3 h-3 text-blue-500 dark:text-blue-400 flex-shrink-0" />
                    <span>{cal}</span>
                  </div>
                ))}
              </div>
              <div className="mt-4 pt-3 border-t">
                <div className="flex items-center gap-1.5 text-xs">
                  <AlertTriangle className="w-3 h-3 text-amber-500 flex-shrink-0" />
                  <span className="text-muted-foreground">FORENSICS embeds all 42 systems automatically</span>
                </div>
              </div>
            </Card>
          </motion.div>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="mt-6 text-center"
        >
          <Button variant="outline" size="sm" asChild data-testid="button-view-calendars">
            <Link href="/calendar">
              Explore All 42 Calendar Systems
              <ArrowRight className="w-3.5 h-3.5 ml-1" />
            </Link>
          </Button>
        </motion.div>
      </div>
    </section>
  );
}

function HashAlgorithmsSection() {
  return (
    <section className="py-16 md:py-20" data-testid="section-tsa-hashes">
      <div className="max-w-6xl mx-auto px-5">
        <div className="text-center mb-10">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Hash Algorithms
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-hashes-title"
          >
            Supported Message Digests
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            Six hash algorithms from the SHA-2 and SHA-3 families. All CNSA 2.0 compliant.
          </motion.p>
        </div>

        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-3 max-w-4xl mx-auto">
          {HASH_ALGORITHMS.map((alg, index) => (
            <motion.div
              key={alg.name}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: index * 0.06 }}
            >
              <Card className="p-4" data-testid={`card-hash-${alg.name.toLowerCase()}`}>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <Hash className="w-3.5 h-3.5 text-primary" />
                    <span className="font-semibold text-sm">{alg.name}</span>
                  </div>
                  <Badge variant="secondary" className="text-[10px]">{alg.bits}-bit</Badge>
                </div>
                <p className="text-[10px] font-mono text-muted-foreground">{alg.oid}</p>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function EndpointsSection() {
  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-tsa-endpoints">
      <div className="max-w-6xl mx-auto px-5">
        <div className="text-center mb-10">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              8 Endpoints
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-endpoints-title"
          >
            API Reference
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            All endpoints are served under the <code className="font-mono text-foreground bg-muted px-1 rounded">/api/tsa</code> prefix,
            managed as Kong service #21.
          </motion.p>
        </div>

        <div className="space-y-3 max-w-5xl mx-auto">
          {ENDPOINTS.map((ep, index) => (
            <motion.div
              key={`${ep.method}-${ep.path}`}
              initial={{ opacity: 0, y: 15 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.3, delay: index * 0.05 }}
            >
              <Card className="p-4" data-testid={`card-endpoint-${index}`}>
                <div className="flex flex-col sm:flex-row sm:items-start gap-2 sm:gap-4">
                  <div className="flex items-center gap-2 sm:w-72 flex-shrink-0">
                    <Badge
                      variant={ep.method === "POST" ? "default" : "secondary"}
                      className="text-[10px] font-mono w-12 justify-center"
                    >
                      {ep.method}
                    </Badge>
                    <code className="text-xs font-mono text-foreground">{ep.path}</code>
                  </div>
                  <div className="flex-1">
                    <p className="text-xs text-muted-foreground">{ep.description}</p>
                  </div>
                  <div className="flex items-center gap-2 flex-shrink-0">
                    <Badge variant="outline" className="text-[10px]">
                      {ep.auth}
                    </Badge>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function HealthSection() {
  const { data: health, isLoading, error } = useQuery<TsaHealth>({
    queryKey: ["/api/tsa/health"],
    refetchInterval: 30000,
  });

  return (
    <section className="py-16 md:py-20" data-testid="section-tsa-health">
      <div className="max-w-6xl mx-auto px-5">
        <div className="text-center mb-10">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Live Status
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-health-title"
          >
            Service Health
          </motion.h2>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="max-w-3xl mx-auto"
        >
          <Card className="p-6" data-testid="card-tsa-health">
            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <div className="inline-flex items-center justify-center w-8 h-8 rounded-full border-2 border-primary/20 border-t-primary animate-spin" />
              </div>
            ) : error ? (
              <div className="text-center py-8">
                <AlertTriangle className="w-8 h-8 text-amber-500 mx-auto mb-3" />
                <p className="text-sm text-muted-foreground">Unable to reach TSA service</p>
              </div>
            ) : health ? (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className={`w-3 h-3 rounded-full ${health.status === "healthy" ? "bg-emerald-500 animate-pulse" : "bg-red-500"}`} />
                    <span className="font-semibold text-sm capitalize" data-testid="text-health-status">{health.status}</span>
                  </div>
                  <Badge variant="outline" className="text-[10px]" data-testid="text-health-serial">
                    Serial #{health.serialNumber}
                  </Badge>
                </div>

                <div className="grid sm:grid-cols-3 gap-4 pt-3 border-t">
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Certificate</span>
                    <p className="text-xs font-mono text-foreground truncate" data-testid="text-health-subject">
                      {health.tsaCertSubject?.split("\n")[0] || "N/A"}
                    </p>
                  </div>
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Valid Until</span>
                    <p className="text-xs text-foreground" data-testid="text-health-expiry">
                      {health.tsaCertExpiry || "N/A"}
                    </p>
                  </div>
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Signing Key</span>
                    <p className="text-xs text-foreground flex items-center gap-1" data-testid="text-health-signing">
                      {health.tsaKeyLoaded ? (
                        <>
                          <ShieldCheck className="w-3 h-3 text-emerald-500" /> Available
                        </>
                      ) : (
                        <>
                          <AlertTriangle className="w-3 h-3 text-amber-500" /> Unavailable
                        </>
                      )}
                    </p>
                  </div>
                </div>

                <div className="grid sm:grid-cols-3 gap-4 pt-3 border-t">
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Dual Sign</span>
                    <p className="text-xs text-foreground">{health.dualSignEnabled ? "RSA-4096 + TL-DSA" : "RSA-4096 only"}</p>
                  </div>
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Merkle Depth</span>
                    <p className="text-xs text-foreground">{health.merkleTreeDepth} levels</p>
                  </div>
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground block mb-1">Tokens (24h)</span>
                    <p className="text-xs text-foreground">{health.tokensIssuedLast24h}</p>
                  </div>
                </div>

                <div className="pt-3 border-t flex items-center justify-between text-xs text-muted-foreground">
                  <span>Uptime: {health.uptime ? `${Math.floor(health.uptime / 3600)}h ${Math.floor((health.uptime % 3600) / 60)}m` : "N/A"}</span>
                  <span>Auto-refreshes every 30s</span>
                </div>
              </div>
            ) : null}
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function CTASection() {
  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-tsa-cta">
      <div className="max-w-3xl mx-auto px-5 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
        >
          <h2 className="text-2xl md:text-3xl font-bold mb-3">Explore the Framework</h2>
          <p className="text-muted-foreground mb-6">
            Try the TSA endpoints in the API Explorer, browse the calendar subsystem, or read the whitepaper.
          </p>
          <div className="flex flex-wrap justify-center gap-3">
            <Button asChild data-testid="button-cta-api">
              <Link href="/api-demo">
                API Explorer
                <ArrowRight className="w-4 h-4 ml-1" />
              </Link>
            </Button>
            <Button variant="outline" asChild data-testid="button-cta-calendar">
              <Link href="/calendar">
                42 Calendar Systems
                <Calendar className="w-4 h-4 ml-1" />
              </Link>
            </Button>
            <Button variant="outline" asChild data-testid="button-cta-compliance">
              <Link href="/compliance">
                CNSA 2.0 Compliance
                <ShieldCheck className="w-4 h-4 ml-1" />
              </Link>
            </Button>
            <Button variant="outline" asChild data-testid="button-cta-whitepaper">
              <Link href="/whitepaper">
                Whitepaper
                <ExternalLink className="w-4 h-4 ml-1" />
              </Link>
            </Button>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

export default function TsaPage() {
  return (
    <div className="min-h-screen" data-testid="page-tsa">
      <HeroSection />
      <SecurityFeaturesSection />
      <ComplianceStatusSection />
      <PolicyTiersSection />
      <CalendarContextSection />
      <HashAlgorithmsSection />
      <EndpointsSection />
      <HealthSection />
      <CTASection />
    </div>
  );
}
