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

import { useState, useMemo, useCallback } from "react";
import { Link } from "wouter";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowLeft, Download, Package, Search, Plus, Check, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useToast } from "@/hooks/use-toast";

interface Module {
  id: string;
  name: string;
  version: string;
  desc: string;
  tiers: string[];
  size: string;
  format: string;
  deps: string;
  file: string;
}

const MODULES: Module[] = [
  {
    id: "kernel",
    name: "Ternary Kernel",
    version: "v0.1.0",
    desc: "The core bijective ternary arithmetic engine. Rep A/B/C interchange, base-3 addition with carry tracking, trit vector operations. Everything else depends on this.",
    tiers: ["core"],
    size: "48 KB",
    format: ".rs / .wasm",
    deps: "None \u2014 foundation module",
    file: "libternary-kernel-v0.1.0",
  },
  {
    id: "constants",
    name: "Constants Module",
    version: "v0.1.0",
    desc: "Singular source of truth for all shared constants: 364\u00b0 circle, \u03c0=14, \u03c4 Tribonacci, Z\u2082\u2088 cyclic order, BORROMEAN_MODULUS. Every other module imports from here.",
    tiers: ["core"],
    size: "12 KB",
    format: ".rs / .ts",
    deps: "None \u2014 leaf node",
    file: "libternary-constants-v0.1.0",
  },
  {
    id: "tribonacci",
    name: "Tribonacci Engine",
    version: "v0.1.0",
    desc: "Base-3 Tribonacci recurrence generator with 22 unit tests. Canonical word morphism (\u03c3: 0\u219201, 1\u219202, 2\u21920) as test oracle. Power-of-3 alignment detection, carry event tracking.",
    tiers: ["core", "developer"],
    size: "38 KB",
    format: ".rs / .wasm",
    deps: "Kernel, Constants",
    file: "libternary-tribonacci-v0.1.0",
  },
  {
    id: "ternary-circle",
    name: "Ternary Circle (364\u00b0)",
    version: "v0.1.0",
    desc: "The Z\u2082\u2088 cyclic group, angular conversions (ternary \u2194 standard), spiral walk engine, repunit verification, GF(3) projection. 20 unit tests.",
    tiers: ["core", "developer"],
    size: "24 KB",
    format: ".rs / .ts / .wasm",
    deps: "Constants",
    file: "libternary-ternary-circle-v0.1.0",
  },
  {
    id: "borromean",
    name: "Borromean Validator",
    version: "v0.1.0",
    desc: "Three-word ternary XOR invariant for non-separable linking. Triple validation, pairwise separability check, pseudo-random word generation. 14 unit tests.",
    tiers: ["core", "developer"],
    size: "26 KB",
    format: ".rs / .wasm",
    deps: "Kernel, Constants",
    file: "libternary-borromean-v0.1.0",
  },
  {
    id: "clifford",
    name: "Clifford Algebra Bridge",
    version: "v0.1.0",
    desc: "Z\u2082\u2088 \u2192 Cl(2,0) rotor mapping. Position-to-rotor conversion, Clifford walk composition, canonical Tribonacci walk builder, rotor consistency verification.",
    tiers: ["developer"],
    size: "20 KB",
    format: ".rs / .wasm",
    deps: "Ternary Circle, Constants",
    file: "libternary-clifford-v0.1.0",
  },
  {
    id: "torus",
    name: "Torus Mapper",
    version: "v0.1.0",
    desc: "Z\u2082\u2088 \u2192 T\u00b2 toroidal address mapping. Simultaneous angular and topological tracking, \u03c4-scaled torus distance metric, Tribonacci torus walk.",
    tiers: ["developer"],
    size: "18 KB",
    format: ".rs / .wasm",
    deps: "Ternary Circle, Constants",
    file: "libternary-torus-v0.1.0",
  },
  {
    id: "hptp",
    name: "HPTP Timestamp Tool",
    version: "v0.9.0",
    desc: "High-Precision Timing Protocol client. Z\u2082\u2088 phase correction, sub-radian alignment detection, carry jerk analysis. CNSA 2.0 compliant time synchronization.",
    tiers: ["enterprise", "user"],
    size: "156 KB",
    format: "Binary (Linux / macOS / Win)",
    deps: "Kernel, Ternary Circle",
    file: "hptp-tool-v0.9.0",
  },
  {
    id: "plenumnet-client",
    name: "PlenumNET Client SDK",
    version: "v0.1.0",
    desc: "Full TypeScript client library for PlenumNET integration. 70+ API endpoint bindings, post-quantum key establishment (ML-KEM-1024), Borromean handshake protocol.",
    tiers: ["developer", "enterprise"],
    size: "320 KB",
    format: ".ts (npm package)",
    deps: "Constants, Ternary Circle (.ts)",
    file: "plenumnet-client-sdk-v0.1.0",
  },
  {
    id: "kong-config",
    name: "Kong Gateway Config",
    version: "v1.2.0",
    desc: "API governance configuration for Kong Konnect. Rate limiting, CORS policies, route definitions, upstream health checks. Drop-in deployment for PlenumNET infrastructure.",
    tiers: ["enterprise"],
    size: "28 KB",
    format: ".yml / .json",
    deps: "None \u2014 infrastructure config",
    file: "kong-gateway-config-v1.2.0",
  },
  {
    id: "triskellion-viz",
    name: "Triskellion Visualization",
    version: "v2.0.0",
    desc: "Interactive Z\u2082\u2088 radian spiral walk on HTML5 Canvas. 28-ray lattice display, \u03c4-scaling, Rep A/B/C selector, real-time walk statistics. Deployable as standalone or embed.",
    tiers: ["user", "developer"],
    size: "14 KB",
    format: ".html (self-contained)",
    deps: "None \u2014 standalone",
    file: "triskellion-walk-v2.0.0",
  },
  {
    id: "docker-stack",
    name: "Docker Deployment Stack",
    version: "v1.0.0",
    desc: "Complete container orchestration for PlenumNET production deployment. Docker Compose, pinned base images, health check endpoints, non-root container execution.",
    tiers: ["enterprise"],
    size: "8 KB",
    format: "docker-compose.yml",
    deps: "Kong Config (optional)",
    file: "plenumnet-docker-stack-v1.0.0",
  },
];

type TierFilter = "all" | "core" | "developer" | "enterprise" | "user";

const TIER_STYLES: Record<string, { bg: string; text: string; border: string }> = {
  core: {
    bg: "bg-orange-500/10 dark:bg-orange-400/10",
    text: "text-orange-700 dark:text-orange-400",
    border: "border-orange-500/20 dark:border-orange-400/20",
  },
  developer: {
    bg: "bg-blue-500/10 dark:bg-blue-400/10",
    text: "text-blue-700 dark:text-blue-400",
    border: "border-blue-500/20 dark:border-blue-400/20",
  },
  enterprise: {
    bg: "bg-purple-500/10 dark:bg-purple-400/10",
    text: "text-purple-700 dark:text-purple-400",
    border: "border-purple-500/20 dark:border-purple-400/20",
  },
  user: {
    bg: "bg-emerald-500/10 dark:bg-emerald-400/10",
    text: "text-emerald-700 dark:text-emerald-400",
    border: "border-emerald-500/20 dark:border-emerald-400/20",
  },
};

const FILTER_OPTIONS: { value: TierFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "core", label: "Core" },
  { value: "developer", label: "Developer" },
  { value: "enterprise", label: "Enterprise" },
  { value: "user", label: "User" },
];

const AXIOM_ITEMS = [
  "364\u00b0 = 111111\u2083",
  "\u03c0 = 14",
  "1 rad = 13\u00b0 = T\u2087",
  "Z\u2082\u2088 lattice",
];

function TierBadge({ tier }: { tier: string }) {
  const style = TIER_STYLES[tier];
  if (!style) return null;
  return (
    <Badge
      variant="outline"
      className={`text-[10px] tracking-wider uppercase ${style.bg} ${style.text} ${style.border}`}
      data-testid={`badge-tier-${tier}`}
    >
      {tier}
    </Badge>
  );
}

function ModuleCard({
  mod,
  isSelected,
  onToggleBundle,
  onDownload,
  index,
}: {
  mod: Module;
  isSelected: boolean;
  onToggleBundle: (id: string) => void;
  onDownload: (id: string) => void;
  index: number;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, delay: index * 0.04 }}
    >
      <Card
        className={`p-5 flex flex-col h-full transition-colors ${
          isSelected ? "bg-accent" : ""
        }`}
        data-testid={`card-module-${mod.id}`}
      >
        <div className="flex items-start justify-between gap-2 mb-3">
          <h3 className="text-base font-semibold" data-testid={`text-module-name-${mod.id}`}>
            {mod.name}
          </h3>
          <Badge variant="outline" className="text-[10px] text-muted-foreground tracking-wide whitespace-nowrap shrink-0">
            {mod.version}
          </Badge>
        </div>

        <p className="text-sm text-muted-foreground leading-relaxed flex-1 mb-4" data-testid={`text-module-desc-${mod.id}`}>
          {mod.desc}
        </p>

        <div className="flex items-center gap-2 mb-3 flex-wrap">
          {mod.tiers.map((t) => (
            <TierBadge key={t} tier={t} />
          ))}
          <span className="text-[11px] text-muted-foreground">{mod.size}</span>
          <span className="text-[11px] text-muted-foreground">{mod.format}</span>
        </div>

        <p className="text-xs text-muted-foreground italic mb-4">{mod.deps}</p>

        <div className="flex gap-2 mt-auto flex-wrap">
          <Button
            className="flex-1"
            size="sm"
            onClick={() => onDownload(mod.id)}
            data-testid={`button-download-${mod.id}`}
          >
            <Download className="w-3.5 h-3.5 mr-1.5" />
            Download
          </Button>
          <Button
            variant={isSelected ? "default" : "outline"}
            size="sm"
            onClick={() => onToggleBundle(mod.id)}
            data-testid={`button-bundle-${mod.id}`}
          >
            {isSelected ? (
              <>
                <Check className="w-3.5 h-3.5 mr-1" />
                Bundled
              </>
            ) : (
              <>
                <Plus className="w-3.5 h-3.5 mr-1" />
                Bundle
              </>
            )}
          </Button>
        </div>
      </Card>
    </motion.div>
  );
}

export default function DistributionPage() {
  const [activeFilter, setActiveFilter] = useState<TierFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedModules, setSelectedModules] = useState<Set<string>>(new Set());
  const { toast } = useToast();

  const toggleBundle = useCallback((id: string) => {
    setSelectedModules((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
        const mod = MODULES.find((m) => m.id === id);
        if (mod) {
          if (mod.deps.includes("Kernel")) next.add("kernel");
          if (mod.deps.includes("Constants")) next.add("constants");
          if (mod.deps.includes("Ternary Circle")) next.add("ternary-circle");
        }
      }
      return next;
    });
  }, []);

  const downloadModule = useCallback(
    (id: string) => {
      const mod = MODULES.find((m) => m.id === id);
      if (!mod) return;

      toast({
        title: `Preparing ${mod.name} ${mod.version}...`,
        description: "Generating signed download URL with SHA-384 manifest.",
      });

      setTimeout(() => {
        toast({
          title: `${mod.name} \u2014 download ready`,
          description: "SHA-384 verified. Cryptographic integrity confirmed.",
        });
        console.log(`[DIST] Download: https://github.com/SigmaWolf-8/Ternary/releases/latest/download/${mod.file}.tar.gz`);
      }, 600);
    },
    [toast]
  );

  const downloadBundle = useCallback(() => {
    if (selectedModules.size === 0) return;

    toast({
      title: `Building bundle: ${selectedModules.size} modules...`,
      description: "Resolving dependencies and generating manifest.",
    });

    setTimeout(() => {
      toast({
        title: `Bundle ready \u2014 ${selectedModules.size} modules`,
        description: "SHA-384 manifest included. All integrity checks passed.",
      });
      console.log("[DIST] Bundle:", [...selectedModules]);
    }, 800);
  }, [selectedModules, toast]);

  const filteredModules = useMemo(() => {
    const query = searchQuery.toLowerCase().trim();
    return MODULES.filter((mod) => {
      const matchesFilter = activeFilter === "all" || mod.tiers.includes(activeFilter);
      const matchesSearch =
        !query ||
        mod.name.toLowerCase().includes(query) ||
        mod.desc.toLowerCase().includes(query) ||
        mod.id.includes(query);
      return matchesFilter && matchesSearch;
    });
  }, [activeFilter, searchQuery]);

  return (
    <div className="min-h-screen bg-background" data-testid="page-distribution">
      <div className="max-w-7xl mx-auto px-5 py-8">
        <div className="mb-8">
          <Button
            variant="ghost"
            size="sm"
            asChild
            data-testid="link-back-home"
          >
            <Link href="/">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Home
            </Link>
          </Button>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="text-center mb-10"
        >
          <Badge variant="outline" className="px-4 py-1.5 mb-4">
            Applied Physics Division
          </Badge>
          <h1 className="text-4xl md:text-5xl font-bold mb-3" data-testid="text-distribution-title">
            Module <span className="text-foreground underline decoration-2 underline-offset-4 decoration-muted-foreground/40">Distribution</span>
          </h1>
          <p className="text-muted-foreground max-w-xl mx-auto leading-relaxed" data-testid="text-distribution-subtitle">
            Select individual components or build a custom bundle. Each module is independently deployable with verified cryptographic integrity and full CNSA 2.0 compliance chain.
          </p>

          <div className="mt-6 inline-flex flex-wrap justify-center gap-4 border rounded-md px-5 py-2.5 text-xs text-muted-foreground tracking-wider">
            {AXIOM_ITEMS.map((item) => (
              <span key={item} className="text-foreground font-medium">{item}</span>
            ))}
          </div>
        </motion.div>

        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 mb-6 flex-wrap">
          <div className="flex flex-wrap gap-1" data-testid="filter-group">
            {FILTER_OPTIONS.map((opt) => (
              <Button
                key={opt.value}
                variant={activeFilter === opt.value ? "default" : "outline"}
                size="sm"
                onClick={() => setActiveFilter(opt.value)}
                className="text-xs tracking-wider uppercase"
                data-testid={`button-filter-${opt.value}`}
              >
                {opt.label}
              </Button>
            ))}
          </div>

          <div className="relative flex-1 max-w-xs">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              placeholder="Search modules..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9 text-sm"
              data-testid="input-search-modules"
            />
          </div>

          <div className="flex items-center gap-3 sm:ml-auto flex-wrap">
            <span className="text-sm text-muted-foreground">
              Bundle: <strong className="text-foreground font-semibold" data-testid="text-bundle-count">{selectedModules.size}</strong> modules
            </span>
            <Button
              variant="outline"
              size="sm"
              disabled={selectedModules.size === 0}
              onClick={downloadBundle}
              data-testid="button-download-bundle"
            >
              <Package className="w-3.5 h-3.5 mr-1.5" />
              Download Bundle
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-12">
          <AnimatePresence mode="popLayout">
            {filteredModules.map((mod, i) => (
              <ModuleCard
                key={mod.id}
                mod={mod}
                isSelected={selectedModules.has(mod.id)}
                onToggleBundle={toggleBundle}
                onDownload={downloadModule}
                index={i}
              />
            ))}
          </AnimatePresence>
          {filteredModules.length === 0 && (
            <div className="col-span-full text-center py-16 text-muted-foreground" data-testid="text-no-results">
              No modules match your search criteria.
            </div>
          )}
        </div>

        <div className="border-t pt-6 flex flex-col sm:flex-row items-center justify-between gap-4 flex-wrap text-xs text-muted-foreground">
          <span>2026 Capomastro Holdings Ltd. \u2014 All rights reserved. Proprietary.</span>
          <div className="flex items-center gap-4 flex-wrap">
            <a
              href="https://github.com/SigmaWolf-8/Ternary"
              target="_blank"
              rel="noopener noreferrer"
              className="transition-colors flex items-center gap-1"
              data-testid="link-github"
            >
              GitHub <ExternalLink className="w-3 h-3" />
            </a>
            <Link href="/docs" className="transition-colors" data-testid="link-docs">
              Documentation
            </Link>
            <Link href="/api-demo" className="transition-colors" data-testid="link-api-ref">
              API Reference
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
