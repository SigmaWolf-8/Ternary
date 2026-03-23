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

import { useState, useMemo } from "react";
import { Link } from "wouter";
import { motion, AnimatePresence } from "framer-motion";
import {
  ArrowLeft,
  Download,
  Package,
  Search,
  ExternalLink,
  Terminal,
  Copy,
  Check,
  Shield,
  Cpu,
  Layers,
  Wrench,
  Server,
  Zap,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useToast } from "@/hooks/use-toast";
import { PLATFORM } from "@shared/constants";

const GITHUB_REPO = "https://github.com/SigmaWolf-8/Ternary";
const GITHUB_DOWNLOAD = `${GITHUB_REPO}/archive/refs/heads/main.zip`;
const GITHUB_RELEASE = `${GITHUB_REPO}/releases`;
const INSTALLER_WIN = "https://plenumnet.replit.app/install/Install-PlenumNET.bat";
const INSTALLER_UNIX = "https://plenumnet.replit.app/install/install.sh";
const DAEMON_DEPLOYER_BAT = "https://plenumnet.replit.app/api/deploy-daemon.bat";
const YODA_DEPLOYER_BAT = "https://plenumnet.replit.app/api/deploy-yoda.bat";

type Platform = "windows" | "mac" | "linux";

function detectPlatform(): Platform {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) return "windows";
  if (ua.includes("mac")) return "mac";
  return "linux";
}

interface Module {
  id: string;
  name: string;
  version: string;
  desc: string;
  category: string;
  size: string;
  format: string;
  path: string;
}

const CATEGORIES = [
  {
    id: "core",
    label: "Core Mathematics",
    icon: Cpu,
    desc: "Foundation ternary arithmetic, constants, and algebraic structures",
  },
  {
    id: "crypto",
    label: "Cryptography & Security",
    icon: Shield,
    desc: "Post-quantum primitives, sponge constructions, and CNSA 2.0 compliance",
  },
  {
    id: "network",
    label: "Network & Addressing",
    icon: Layers,
    desc: "TDNS ontological addressing, torus topology, and Inter-Cube routing",
  },
  {
    id: "tools",
    label: "Tools & SDKs",
    icon: Wrench,
    desc: "Client libraries, CLI tools, and visualization components",
  },
  {
    id: "infra",
    label: "Infrastructure",
    icon: Server,
    desc: "Deployment configs, API gateway, and container orchestration",
  },
];

const MODULES: Module[] = [
  {
    id: "kernel",
    name: "Ternary Kernel",
    version: "v0.1.0",
    desc: "Core bijective ternary arithmetic engine. Rep A/B/C interchange, base-3 addition with carry tracking, trit vector operations.",
    category: "core",
    size: "48 KB",
    format: ".rs / .wasm",
    path: "src/kernel",
  },
  {
    id: "constants",
    name: "Constants Module",
    version: "v0.1.0",
    desc: `Singular source of truth: 364\u00b0 circle, \u03c0=14, \u03c4 Tribonacci, Z\u2082\u2088 cyclic order. Every other module imports from here.`,
    category: "core",
    size: "12 KB",
    format: ".rs / .ts",
    path: "shared/constants.ts",
  },
  {
    id: "gf3-algebra",
    name: "GF(3) Algebra",
    version: "v0.1.0",
    desc: "Division-free Galois Field GF(3) arithmetic with exhaustive axiom verification. Conditional subtract only — no modulo in production.",
    category: "core",
    size: "10 KB",
    format: ".rs / .ts",
    path: "ternary-math/src/gf3_algebra.rs",
  },
  {
    id: "tribonacci",
    name: "Tribonacci Engine",
    version: "v0.1.0",
    desc: "Base-3 Tribonacci recurrence generator. Canonical word morphism, power-of-3 alignment detection, carry event tracking.",
    category: "core",
    size: "38 KB",
    format: ".rs / .wasm",
    path: "ternary-math/src/tribonacci.rs",
  },
  {
    id: "ternary-circle",
    name: "Ternary Circle (364\u00b0)",
    version: "v0.1.0",
    desc: `Z\u2082\u2088 cyclic group, angular conversions (ternary \u2194 standard), spiral walk engine, repunit verification, GF(3) projection.`,
    category: "core",
    size: "24 KB",
    format: ".rs / .ts / .wasm",
    path: "ternary-math/src/ternary_circle.rs",
  },
  {
    id: "borromean",
    name: "Borromean Validator",
    version: "v0.1.0",
    desc: "Three-word ternary XOR invariant for non-separable linking. Triple validation, pairwise separability check.",
    category: "core",
    size: "26 KB",
    format: ".rs / .wasm",
    path: "ternary-math/src/borromean.rs",
  },
  {
    id: "sponge",
    name: "TL-Sponge-385 (385-bit PQ)",
    version: "v2.0",
    desc: "Same proven sponge construction, post-quantum parameters (385-bit capacity, NIST Level 5+). Differential trail security inherits from the formal wide-trail analysis in TM-2026-008.",
    category: "crypto",
    size: "22 KB",
    format: ".rs",
    path: "src/kernel/src/crypto/sponge.rs",
  },
  {
    id: "tis27",
    name: "TIS-27 Wire Integrity",
    version: "v1.0",
    desc: "Construction proven secure by wide-trail analysis (DP \u2264 9\u207B\u2074\u2070\u2079\u2076). Same proven sponge as TL-Sponge-385, sized for fast integrity (43-bit cryptographic security). 191 ns, SSE2 SIMD.",
    category: "crypto",
    size: "9 KB",
    format: ".rs / .ts",
    path: "ternary-math/src/tis_sponge.rs",
  },
  {
    id: "tl-dsa",
    name: "TL-DSA Signatures",
    version: "v0.1.0",
    desc: `Post-quantum ternary lattice digital signatures. Three security levels (44/65/87). Integer NTT, AVX2 vectorized. ${PLATFORM.BENCH_TL_DSA_87_US}\u00b5s sign+verify.`,
    category: "crypto",
    size: "180 KB",
    format: ".rs",
    path: "src/kernel/src/crypto/tl_dsa.rs",
  },
  {
    id: "tl-kem",
    name: "TL-KEM Key Encapsulation",
    version: "v0.1.0",
    desc: `Post-quantum key encapsulation mechanism. ${PLATFORM.BENCH_KEM_ROUNDTRIP_MS}ms round-trip. CNSA 2.0 Phase 2 compliant.`,
    category: "crypto",
    size: "95 KB",
    format: ".rs",
    path: "src/kernel/src/crypto/tl_kem.rs",
  },
  {
    id: "phase-encryption",
    name: "Phase Encryption",
    version: "v0.1.0",
    desc: "Multi-phase encryption with ternary key scheduling. Configurable split ratios and phase counts for layered security.",
    category: "crypto",
    size: "45 KB",
    format: ".rs",
    path: "src/kernel/src/phase.rs",
  },
  {
    id: "tdns",
    name: "TDNS v2.5 Addressing",
    version: "v2.5.0",
    desc: "54-trit dual-layer ontological addressing. 27 classification + 27 identity anchor. 68.63 trillion address space.",
    category: "network",
    size: "85 KB",
    format: ".rs / .ts",
    path: "services/tdns-v2",
  },
  {
    id: "torus",
    name: "Torus Network Topology",
    version: "v0.1.0",
    desc: `Toroidal address mapping for the Torsion Network. \u03c4-scaled distance metric, Tribonacci torus walk, ${PLATFORM.HYPERCUBE_DIMENSIONS}D navigation.`,
    category: "network",
    size: "18 KB",
    format: ".rs / .wasm",
    path: "ternary-math/src/torus.rs",
  },
  {
    id: "inter-cube",
    name: "Inter-Cube Services",
    version: "v0.3.0",
    desc: `4-service geometric routing: GLB, CON, CRS, FTS. ${PLATFORM.HYPERCUBE_VERTICES} vertices, ${PLATFORM.INTER_CUBE_TUNNELS} PQ encrypted tunnels, ${PLATFORM.HYPERCUBE_NEIGHBORS} neighbors per node.`,
    category: "network",
    size: "120 KB",
    format: ".rs / .ts",
    path: "services/inter-cube",
  },
  {
    id: "metatronic-cube",
    name: "Metatronic Cube",
    version: "v0.1.0",
    desc: "13-dimensional ternary cube with Saturnian shells, Metatronic circles, correspondence edges, and automorphisms.",
    category: "network",
    size: "95 KB",
    format: ".rs",
    path: "src/kernel/src/crypto/metatronic_cube.rs",
  },
  {
    id: "hptp",
    name: "HPTP Timing Tool",
    version: "v0.9.0",
    desc: `High-Precision Timing Protocol client. Z\u2082\u2028 phase correction, sub-radian alignment, femtosecond timestamps. CNSA 2.0 aligned.`,
    category: "tools",
    size: "156 KB",
    format: "Binary (Linux / macOS / Win)",
    path: "src/kernel/src/timing.rs",
  },
  {
    id: "client-sdk",
    name: "PlenumNET Client SDK",
    version: "v0.1.0",
    desc: `Full TypeScript client library. ${PLATFORM.API_ENDPOINTS} API endpoint bindings, post-quantum key establishment, Borromean handshake protocol.`,
    category: "tools",
    size: "320 KB",
    format: ".ts (npm)",
    path: "shared",
  },
  {
    id: "triskellion-viz",
    name: "Triskellion Visualization",
    version: "v2.0.0",
    desc: `Interactive Z\u2082\u2028 radian spiral walk on HTML5 Canvas. 28-ray lattice, \u03c4-scaling, Rep A/B/C selector.`,
    category: "tools",
    size: "14 KB",
    format: ".html (standalone)",
    path: "client/src/components",
  },
  {
    id: "tdns-extension",
    name: "TDNS Chrome Extension",
    version: "v1.0.9",
    desc: "Browser extension for resolving .plm addresses. Dual-color display: classification in gold, identity anchor in sky blue.",
    category: "tools",
    size: "42 KB",
    format: "Chrome/Edge/Arc/Brave",
    path: "services/tdns-v2/extension-chromium",
  },
  {
    id: "kong-config",
    name: "Kong Gateway Config",
    version: "v1.2.0",
    desc: `API governance for Kong Konnect. ${PLATFORM.API_SERVICES} services, ${PLATFORM.API_ENDPOINTS} endpoints. Rate limiting, CORS, health checks.`,
    category: "infra",
    size: "28 KB",
    format: ".yml / .json",
    path: "deployments",
  },
  {
    id: "docker-stack",
    name: "Docker Deployment Stack",
    version: "v1.0.0",
    desc: "Container orchestration for production. Docker Compose, pinned images, health checks, non-root execution.",
    category: "infra",
    size: "8 KB",
    format: "docker-compose.yml",
    path: "deployments",
  },
];

const CATEGORY_STYLES: Record<string, string> = {
  core: "text-orange-600 dark:text-orange-400",
  crypto: "text-red-600 dark:text-red-400",
  network: "text-blue-600 dark:text-blue-400",
  tools: "text-emerald-600 dark:text-emerald-400",
  infra: "text-purple-600 dark:text-purple-400",
};

const CATEGORY_BADGE_STYLES: Record<string, string> = {
  core: "border-orange-500/20 bg-orange-500/5 text-orange-700 dark:text-orange-400",
  crypto: "border-red-500/20 bg-red-500/5 text-red-700 dark:text-red-400",
  network: "border-blue-500/20 bg-blue-500/5 text-blue-700 dark:text-blue-400",
  tools: "border-emerald-500/20 bg-emerald-500/5 text-emerald-700 dark:text-emerald-400",
  infra: "border-purple-500/20 bg-purple-500/5 text-purple-700 dark:text-purple-400",
};

function downloadFile(mod: Module) {
  const url = `${GITHUB_REPO}/tree/main/${mod.path}`;
  window.open(url, "_blank", "noopener,noreferrer");
}

function CopyCommand({ command, testIdPrefix = "install" }: { command: string; testIdPrefix?: string }) {
  const [copied, setCopied] = useState(false);
  const { toast } = useToast();

  const handleCopy = () => {
    navigator.clipboard.writeText(command).then(() => {
      setCopied(true);
      toast({ title: "Copied to clipboard" });
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <div
      className="flex items-center gap-2 bg-zinc-900 dark:bg-zinc-950 rounded-lg px-4 py-3 font-mono text-sm text-zinc-100 cursor-pointer group"
      onClick={handleCopy}
      data-testid={`copy-command-${testIdPrefix}`}
    >
      <Terminal className="w-4 h-4 text-zinc-500 shrink-0" />
      <code className="flex-1 overflow-x-auto whitespace-nowrap">{command}</code>
      <button
        className="shrink-0 p-1 rounded hover:bg-zinc-700 transition-colors"
        data-testid={`button-copy-command-${testIdPrefix}`}
      >
        {copied ? (
          <Check className="w-4 h-4 text-green-400" />
        ) : (
          <Copy className="w-4 h-4 text-zinc-400 group-hover:text-zinc-200" />
        )}
      </button>
    </div>
  );
}

function ModuleRow({ mod }: { mod: Module }) {
  return (
    <div
      className="flex items-center gap-4 py-3 px-4 rounded-lg hover:bg-muted/50 transition-colors group"
      data-testid={`row-module-${mod.id}`}
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <span className="font-medium text-sm" data-testid={`text-module-name-${mod.id}`}>
            {mod.name}
          </span>
          <Badge variant="outline" className="text-[10px] text-muted-foreground tracking-wide shrink-0">
            {mod.version}
          </Badge>
        </div>
        <p className="text-xs text-muted-foreground leading-relaxed line-clamp-1" data-testid={`text-module-desc-${mod.id}`}>
          {mod.desc}
        </p>
      </div>
      <div className="hidden sm:flex items-center gap-3 shrink-0 text-xs text-muted-foreground">
        <span>{mod.size}</span>
        <span className="text-[11px]">{mod.format}</span>
      </div>
      <Button
        variant="ghost"
        size="sm"
        className="shrink-0 opacity-60 group-hover:opacity-100 transition-opacity"
        onClick={() => downloadFile(mod)}
        data-testid={`button-download-${mod.id}`}
      >
        <ExternalLink className="w-3.5 h-3.5 mr-1.5" />
        View
      </Button>
    </div>
  );
}

function InstallSuiteCard() {
  const [platform, setPlatform] = useState<Platform>(detectPlatform);

  const platformConfig = {
    windows: {
      label: "Windows",
      installerUrl: INSTALLER_WIN,
      installerName: "Install-PlenumNET.bat",
      installPath: "C:\\PlenumNET",
      instructions: [
        'Click "Download Installer" to save Install-PlenumNET.bat',
        "Double-click the downloaded file — it clones the repo, builds the daemon, and generates identity keys",
        "If Windows SmartScreen appears, click 'More info' then 'Run anyway'",
        "Everything installs to C:\\PlenumNET — no manual steps required",
      ],
    },
    mac: {
      label: "macOS",
      installerUrl: INSTALLER_UNIX,
      installerName: "install.sh",
      oneLineInstall: `curl -fsSL https://plenumnet.replit.app/install/install.sh | bash`,
      installPath: "~/PlenumNET",
      instructions: [
        "Open Terminal (Applications > Utilities > Terminal)",
        "Paste the command below and press Enter",
        "It clones the repo, builds the daemon, and generates identity keys — no manual steps required",
      ],
    },
    linux: {
      label: "Linux",
      installerUrl: INSTALLER_UNIX,
      installerName: "install.sh",
      oneLineInstall: `curl -fsSL https://plenumnet.replit.app/install/install.sh | bash`,
      installPath: "~/PlenumNET",
      instructions: [
        "Open a terminal",
        "Paste the command below and press Enter",
        "It clones the repo, builds the daemon, and generates identity keys — no manual steps required",
      ],
    },
  };

  const config = platformConfig[platform];

  return (
    <Card className="p-6 mb-8 border-2" data-testid="card-install-suite">
      <div className="flex items-start gap-4 mb-5">
        <div className="w-10 h-10 rounded-lg bg-foreground/5 flex items-center justify-center shrink-0">
          <Zap className="w-5 h-5" />
        </div>
        <div className="flex-1">
          <h2 className="text-lg font-semibold mb-1" data-testid="text-install-title">
            Install Complete Suite
          </h2>
          <p className="text-sm text-muted-foreground">
            One-click installer for the entire framework: {MODULES.length} modules, {PLATFORM.TESTS_PASSING} passing tests,
            CNSA 2.0 compliant. Downloads, builds the daemon, and generates your first identity automatically. Run again to add more daemons.
          </p>
        </div>
      </div>

      <div className="flex gap-1.5 mb-4" data-testid="platform-selector">
        {(["windows", "mac", "linux"] as Platform[]).map((p) => (
          <Button
            key={p}
            variant={platform === p ? "default" : "outline"}
            size="sm"
            onClick={() => setPlatform(p)}
            className="text-xs capitalize"
            data-testid={`button-platform-${p}`}
          >
            {platformConfig[p].label}
          </Button>
        ))}
      </div>

      <div className="bg-muted/50 rounded-lg p-4 mb-4" data-testid="install-instructions">
        <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground">
          {config.instructions.map((step, i) => (
            <li key={i}>{step}</li>
          ))}
        </ol>
      </div>

      {"oneLineInstall" in config && config.oneLineInstall && (
        <CopyCommand command={config.oneLineInstall} />
      )}

      <div className="flex flex-wrap gap-3 mt-5">
        <Button
          data-testid="button-download-installer"
          onClick={(e) => { e.preventDefault(); window.open(config.installerUrl, "_blank"); }}
        >
          <Download className="w-4 h-4 mr-2" />
          Download Installer ({config.label})
        </Button>
        <Button
          variant="outline"
          data-testid="button-download-archive"
          onClick={(e) => { e.preventDefault(); window.open(GITHUB_DOWNLOAD, "_blank"); }}
        >
          <Package className="w-4 h-4 mr-2" />
          Source Archive (.zip)
        </Button>
        <Button
          variant="outline"
          data-testid="button-github-releases"
          onClick={(e) => { e.preventDefault(); window.open(GITHUB_RELEASE, "_blank"); }}
        >
          <ExternalLink className="w-4 h-4 mr-2" />
          GitHub Releases
        </Button>
      </div>

      <div className="flex flex-wrap gap-x-6 gap-y-1 mt-5 text-xs text-muted-foreground">
        <span>Installs to: <strong className="text-foreground font-medium">{config.installPath}</strong></span>
        <span>v{PLATFORM.PLATFORM_VERSION}</span>
        <span>{PLATFORM.TESTS_PASSING} tests passing</span>
        <span>{PLATFORM.KERNEL_LOC} lines of Rust</span>
        <span>CNSA 2.0 Phase 2</span>
      </div>
    </Card>
  );
}

function DaemonDeployCard() {
  return (
    <Card className="p-6 mb-8 border-2 border-blue-500/20" data-testid="card-daemon-deploy">
      <div className="flex items-start gap-4 mb-5">
        <div className="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center shrink-0">
          <Server className="w-5 h-5 text-blue-600 dark:text-blue-400" />
        </div>
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h2 className="text-lg font-semibold" data-testid="text-daemon-deploy-title">
              Deploy Cube Daemon
            </h2>
            <Badge variant="outline" className="text-[10px] border-blue-500/20 bg-blue-500/5 text-blue-700 dark:text-blue-400">
              v0.3.0
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground">
            One-click deployer for the Inter-Cube daemon. Pulls latest source, builds the daemon,
            and generates the next PT26-DSA identity automatically. Each run adds one more daemon — ports auto-increment.
          </p>
        </div>
      </div>

      <div className="bg-muted/50 rounded-lg p-4 mb-4" data-testid="daemon-deploy-instructions">
        <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground">
          <li>Click the button below to download the installer</li>
          <li>Double-click the downloaded file to run it</li>
          <li>If Windows SmartScreen appears, click "More info" then "Run anyway"</li>
        </ol>
      </div>

      <Button
        data-testid="button-download-daemon-bat"
        onClick={(e) => { e.preventDefault(); window.open(DAEMON_DEPLOYER_BAT, "_blank"); }}
      >
        <Download className="w-4 h-4 mr-2" />
        Download Daemon Installer
      </Button>

      <div className="bg-muted/50 rounded-lg p-4 mt-4 mb-4" data-testid="daemon-deploy-comparison">
        <p className="text-xs font-medium text-foreground mb-2">How is this different from the full installer above?</p>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs text-muted-foreground">
          <div>
            <span className="font-medium text-foreground/70">Full Suite (above)</span>
            <ul className="mt-1 space-y-0.5 list-disc list-inside">
              <li>Git clone + build inter-cube daemon</li>
              <li>Generates first identity automatically</li>
              <li>First-time setup from scratch</li>
            </ul>
          </div>
          <div>
            <span className="font-medium text-blue-700 dark:text-blue-400">Daemon Only (this)</span>
            <ul className="mt-1 space-y-0.5 list-disc list-inside">
              <li>Git pull (incremental updates)</li>
              <li>Stops running daemons, rebuilds, generates keys</li>
              <li>Prints startup commands for A/B/C</li>
            </ul>
          </div>
        </div>
      </div>

      <div className="flex flex-wrap gap-x-6 gap-y-1 text-xs text-muted-foreground">
        <span>Builds to: <strong className="text-foreground font-medium">C:\PlenumNET\target\release\inter-cube-daemon.exe</strong></span>
        <span>Auto-incrementing instances</span>
        <span>PT26-DSA identity keys</span>
      </div>
    </Card>
  );
}

function YodaDeployCard() {
  return (
    <Card className="p-6 mb-8 border-2 border-violet-500/20" data-testid="card-yoda-deploy">
      <div className="flex items-start gap-4 mb-5">
        <div className="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center shrink-0">
          <Zap className="w-5 h-5 text-violet-600 dark:text-violet-400" />
        </div>
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h2 className="text-lg font-semibold" data-testid="text-yoda-deploy-title">
              YODA 3-Node Deployment
            </h2>
            <Badge variant="outline" className="text-[10px] border-violet-500/20 bg-violet-500/5 text-violet-700 dark:text-violet-400">
              v0.3.0
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground">
            One-click 3-daemon deployment. Builds the daemon, generates 3 PT26-DSA identities,
            starts 3 cube daemons, registers all with PlenumNET CRS, and posts a deployment
            summary to the API. Creates a desktop launcher for future starts.
          </p>
        </div>
      </div>

      <div className="bg-muted/50 rounded-lg p-4 mb-4" data-testid="yoda-deploy-instructions">
        <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground">
          <li>Click the button below to download the installer</li>
          <li>Double-click the downloaded file to run it</li>
          <li>The installer handles everything: Rust, LLVM, daemon build, 3 identity generations, CRS registration, and networking</li>
          <li>When complete, a "Start YODA Daemons" shortcut appears on your Desktop</li>
        </ol>
      </div>

      <div className="flex flex-wrap gap-3">
        <Button
          data-testid="button-download-yoda-bat"
          className="bg-violet-600 hover:bg-violet-700 text-white"
          onClick={(e) => { e.preventDefault(); window.open(YODA_DEPLOYER_BAT, "_blank"); }}
        >
          <Download className="w-4 h-4 mr-2" />
          Download YODA Installer
        </Button>
        <CopyCommand
          command="irm https://plenumnet.replit.app/api/deploy-yoda | iex"
          testIdPrefix="yoda-oneliner"
        />
      </div>

      <div className="bg-muted/50 rounded-lg p-4 mt-4 mb-4" data-testid="yoda-deploy-layout">
        <p className="text-xs font-medium text-foreground mb-2">Network layout</p>
        <div className="grid grid-cols-3 gap-3 text-xs text-muted-foreground">
          <div className="bg-background rounded p-2 text-center">
            <span className="block font-medium text-violet-700 dark:text-violet-400">Daemon #1</span>
            <span className="text-[11px]">Port 8081</span>
          </div>
          <div className="bg-background rounded p-2 text-center">
            <span className="block font-medium text-violet-700 dark:text-violet-400">Daemon #2</span>
            <span className="text-[11px]">Port 8083</span>
          </div>
          <div className="bg-background rounded p-2 text-center">
            <span className="block font-medium text-violet-700 dark:text-violet-400">Daemon #3</span>
            <span className="text-[11px]">Port 8085</span>
          </div>
        </div>
      </div>

      <div className="flex flex-wrap gap-x-6 gap-y-1 text-xs text-muted-foreground">
        <span>CRS: <strong className="text-foreground font-medium">plenumnet.replit.app</strong></span>
        <span>3 PT26-DSA identities</span>
        <span>API: <strong className="text-foreground font-medium">/api/salvi/inter-cube/relay/deployments</strong></span>
      </div>
    </Card>
  );
}

export default function DistributionPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [activeCategory, setActiveCategory] = useState<string | null>(null);

  const filteredModules = useMemo(() => {
    const query = searchQuery.toLowerCase().trim();
    return MODULES.filter((mod) => {
      const matchesCategory = !activeCategory || mod.category === activeCategory;
      const matchesSearch =
        !query ||
        mod.name.toLowerCase().includes(query) ||
        mod.desc.toLowerCase().includes(query) ||
        mod.id.includes(query);
      return matchesCategory && matchesSearch;
    });
  }, [searchQuery, activeCategory]);

  const groupedModules = useMemo(() => {
    const groups: Record<string, Module[]> = {};
    for (const cat of CATEGORIES) {
      const mods = filteredModules.filter((m) => m.category === cat.id);
      if (mods.length > 0) groups[cat.id] = mods;
    }
    return groups;
  }, [filteredModules]);

  return (
    <div className="min-h-screen bg-background" data-testid="page-distribution">
      <div className="max-w-5xl mx-auto px-5 py-8">
        <div className="mb-8">
          <Button variant="ghost" size="sm" asChild data-testid="link-back-home">
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
          className="mb-10"
        >
          <Badge variant="outline" className="px-4 py-1.5 mb-4">
            Applied Physics Division
          </Badge>
          <h1 className="text-4xl md:text-5xl font-bold mb-3" data-testid="text-distribution-title">
            Module Distribution
          </h1>
          <p className="text-muted-foreground max-w-2xl leading-relaxed" data-testid="text-distribution-subtitle">
            The complete Salvi Framework. {MODULES.length} modules across {CATEGORIES.length} categories,
            independently deployable with cryptographic integrity verification.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: 0.1 }}
        >
          <InstallSuiteCard />
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: 0.2 }}
        >
          <DaemonDeployCard />
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: 0.3 }}
        >
          <YodaDeployCard />
        </motion.div>

        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 mb-6">
          <div className="relative flex-1 max-w-sm">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              placeholder="Search modules..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9 text-sm"
              data-testid="input-search-modules"
            />
          </div>
          <div className="flex flex-wrap gap-1.5">
            <Button
              variant={activeCategory === null ? "default" : "outline"}
              size="sm"
              onClick={() => setActiveCategory(null)}
              className="text-xs"
              data-testid="button-filter-all"
            >
              All ({MODULES.length})
            </Button>
            {CATEGORIES.map((cat) => {
              const count = MODULES.filter((m) => m.category === cat.id).length;
              return (
                <Button
                  key={cat.id}
                  variant={activeCategory === cat.id ? "default" : "outline"}
                  size="sm"
                  onClick={() => setActiveCategory(activeCategory === cat.id ? null : cat.id)}
                  className="text-xs"
                  data-testid={`button-filter-${cat.id}`}
                >
                  {cat.label} ({count})
                </Button>
              );
            })}
          </div>
        </div>

        <div className="space-y-8 mb-12">
          <AnimatePresence mode="wait">
            {Object.entries(groupedModules).map(([catId, mods]) => {
              const cat = CATEGORIES.find((c) => c.id === catId)!;
              const CatIcon = cat.icon;
              return (
                <motion.div
                  key={catId}
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -8 }}
                  transition={{ duration: 0.3 }}
                >
                  <Card className="overflow-hidden" data-testid={`card-category-${catId}`}>
                    <div className="px-5 py-4 border-b bg-muted/30">
                      <div className="flex items-center gap-3">
                        <CatIcon className={`w-5 h-5 ${CATEGORY_STYLES[catId]}`} />
                        <div>
                          <h2 className="font-semibold text-sm">{cat.label}</h2>
                          <p className="text-xs text-muted-foreground">{cat.desc}</p>
                        </div>
                        <Badge
                          variant="outline"
                          className={`ml-auto text-[10px] ${CATEGORY_BADGE_STYLES[catId]}`}
                        >
                          {mods.length} {mods.length === 1 ? "module" : "modules"}
                        </Badge>
                      </div>
                    </div>
                    <div className="divide-y">
                      {mods.map((mod) => (
                        <ModuleRow key={mod.id} mod={mod} />
                      ))}
                    </div>
                  </Card>
                </motion.div>
              );
            })}
          </AnimatePresence>
          {filteredModules.length === 0 && (
            <div className="text-center py-16 text-muted-foreground" data-testid="text-no-results">
              No modules match your search criteria.
            </div>
          )}
        </div>

        <div className="border-t pt-6 flex flex-col sm:flex-row items-center justify-between gap-4 flex-wrap text-xs text-muted-foreground">
          <span>2026 Capomastro Holdings Ltd. — All rights reserved. Proprietary.</span>
          <div className="flex items-center gap-4 flex-wrap">
            <a
              href={GITHUB_REPO}
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
