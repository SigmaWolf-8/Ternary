/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { useState } from "react";
import { TernaryVMTerminal } from "@/components/ternary-vm-terminal";
import { Link } from "wouter";
import { Terminal, ChevronRight, Shield, Cpu, Zap, Lock } from "lucide-react";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

const quickCommands = [
  { label: "help", desc: "See all commands" },
  { label: "demo", desc: "Encryption pipeline" },
  { label: "demo-cap", desc: "Capability security" },
  { label: "demo-sidech", desc: "Side-channel masking" },
  { label: "status", desc: "VM state" },
  { label: "opcodes", desc: "All 160 opcodes" },
  { label: "cpuid", desc: "Processor info" },
  { label: "arch", desc: "Architecture" },
];

const features = [
  {
    icon: Cpu,
    title: "160-Opcode ISA",
    desc: "Complete instruction set across 7 categories: Core, Extended, Crypto, SIMD, System, Security, Debug",
  },
  {
    icon: Shield,
    title: "Capability Security",
    desc: "Sentinel-trit capabilities with O(1) grant/revoke \u2014 no external tag memory required",
  },
  {
    icon: Lock,
    title: "Post-Quantum Crypto",
    desc: "TL-KEM and TL-DSA hardware opcodes, CNSA 2.0 compliant, FIPS 140-3 boundary",
  },
  {
    icon: Zap,
    title: "Side-Channel Defense",
    desc: "Dual-layer protection: microarchitectural isolation + algebraic ternary masking",
  },
];

export default function VMDemo() {
  const [isFullscreen, setIsFullscreen] = useState(false);

  return (
    <div className="min-h-screen bg-background">
      <div className="bg-gradient-to-b from-slate-900 to-slate-800 dark:from-slate-950 dark:to-slate-900 text-white py-12 px-4">
        <div className="max-w-6xl mx-auto">
          <div className="flex items-center gap-2 text-sm text-slate-400 mb-4" data-testid="text-breadcrumb">
            <Link href="/" className="hover-elevate rounded px-1">
              Home
            </Link>
            <ChevronRight className="w-3 h-3" />
            <span>VM Demo</span>
          </div>

          <div className="flex flex-wrap items-start gap-4 justify-between">
            <div>
              <div className="flex items-center gap-3 mb-2">
                <Terminal className="w-8 h-8 text-blue-400" />
                <h1 className="text-3xl font-bold tracking-tight" data-testid="text-vm-title">
                  Ternary Virtual Machine
                </h1>
              </div>
              <p className="text-slate-300 max-w-2xl text-lg" data-testid="text-vm-subtitle">
                Interactive simulation of the Salvi Framework's 160-opcode ISA v2.0.
                Explore ternary computing, post-quantum cryptography, and capability-based security.
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="secondary" className="bg-blue-500/20 text-blue-300 border-blue-500/30">
                ISA v2.0
              </Badge>
              <Badge variant="secondary" className="bg-green-500/20 text-green-300 border-green-500/30">
                160 Opcodes
              </Badge>
              <Badge variant="secondary" className="bg-purple-500/20 text-purple-300 border-purple-500/30">
                27-Trit Word
              </Badge>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-6xl mx-auto px-4 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
          <div className="lg:col-span-3">
            <Card className="overflow-visible border-slate-700 dark:border-slate-800 bg-[#0a0e1a]">
              <div className="flex items-center justify-between px-4 py-2 bg-slate-800 dark:bg-slate-900 border-b border-slate-700 rounded-t-md">
                <div className="flex items-center gap-2">
                  <div className="flex gap-1.5">
                    <div className="w-3 h-3 rounded-full bg-red-500/80" />
                    <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
                    <div className="w-3 h-3 rounded-full bg-green-500/80" />
                  </div>
                  <span className="text-xs text-slate-400 ml-2 font-mono">salvi@vm ~ Ternary VM v2.0</span>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-slate-400 text-xs"
                  onClick={() => setIsFullscreen(!isFullscreen)}
                  data-testid="button-fullscreen-toggle"
                >
                  {isFullscreen ? "Exit Fullscreen" : "Fullscreen"}
                </Button>
              </div>
              <div
                style={{
                  height: isFullscreen ? "calc(100vh - 200px)" : "520px",
                  transition: "height 0.3s ease",
                }}
              >
                <TernaryVMTerminal />
              </div>
            </Card>

            <div className="mt-6 grid grid-cols-1 sm:grid-cols-2 gap-4">
              {features.map((f) => (
                <Card key={f.title} className="p-4">
                  <div className="flex items-start gap-3">
                    <div className="p-2 rounded-md bg-primary/10">
                      <f.icon className="w-5 h-5 text-primary" />
                    </div>
                    <div>
                      <h3 className="font-semibold text-sm" data-testid={`text-feature-${f.title.toLowerCase().replace(/\s/g, "-")}`}>
                        {f.title}
                      </h3>
                      <p className="text-xs text-muted-foreground mt-1">{f.desc}</p>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          </div>

          <div className="lg:col-span-1 space-y-4">
            <Card className="p-4">
              <h3 className="font-semibold text-sm mb-3 flex items-center gap-2" data-testid="text-quick-commands-heading">
                <Terminal className="w-4 h-4" />
                Quick Commands
              </h3>
              <div className="space-y-1.5">
                {quickCommands.map((c) => (
                  <div
                    key={c.label}
                    className="flex items-center justify-between text-xs p-2 rounded-md hover-elevate"
                    data-testid={`text-cmd-${c.label}`}
                  >
                    <code className="font-mono text-primary font-medium">{c.label}</code>
                    <span className="text-muted-foreground">{c.desc}</span>
                  </div>
                ))}
              </div>
            </Card>

            <Card className="p-4">
              <h3 className="font-semibold text-sm mb-3" data-testid="text-about-heading">About This Demo</h3>
              <p className="text-xs text-muted-foreground leading-relaxed">
                This interactive terminal simulates the Salvi Framework's Ternary Virtual Machine.
                The VM operates on 27-trit words across three bijective representations and implements
                160 opcodes including post-quantum cryptographic acceleration, capability-based security,
                and dual-layer side-channel protection.
              </p>
              <div className="mt-3 pt-3 border-t">
                <Link href="/isa-security">
                  <Button variant="outline" size="sm" className="w-full text-xs" data-testid="link-isa-paper">
                    Read the ISA Security Paper
                    <ChevronRight className="w-3 h-3 ml-1" />
                  </Button>
                </Link>
              </div>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}
