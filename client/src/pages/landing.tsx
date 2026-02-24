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

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import plenumLogo from "@assets/grok-image-69a372f5-5c40-48be-b431-a4dbb4e92ff2_1771299513785.png";
import { PLATFORM } from "@shared/constants";
import { 
  Code, 
  Clock, 
  Cpu, 
  Check, 
  Github, 
  ArrowRight,
  Shield,
  Zap,
  Database,
  Network,
  Building2,
  FlaskConical,
  Factory,
  Mail,
  Terminal,
  Lock,
  Binary,
  Activity,
  Globe,
  Server,
  Copy,
  Calendar
} from "lucide-react";
import { useState, useRef, useEffect } from "react";
import { motion, useInView } from "framer-motion";
import { Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
const heroVideo = "/videos/hero-bg.mp4";



function AnimatedStat({ value, label, suffix, delay }: { value: string; label: string; suffix?: string; delay: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  
  return (
    <motion.div 
      ref={ref}
      initial={{ opacity: 0, y: 20 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 20 }}
      transition={{ duration: 0.5, delay }}
      className="flex flex-col"
      data-testid={`stat-${label.toLowerCase().replace(/\s+/g, '-')}`}
    >
      <span className="text-4xl md:text-5xl font-bold text-primary leading-none">
        {value}{suffix && <span className="text-2xl md:text-3xl">{suffix}</span>}
      </span>
      <span className="text-sm text-muted-foreground mt-2">{label}</span>
    </motion.div>
  );
}

function HeroVisual() {
  const layers = [
    { label: "Applications", items: ["PlenumDB", "Payment", "API", "Certs"], delay: 0.5 },
    { label: "Protocols", items: ["HPTP", "T3P", "TTP", "TDNS"], delay: 0.6 },
    { label: "Virtual Machine", items: [`${PLATFORM.VM_OPCODES} Opcodes`, "27 Registers", "TAGC", "GF(3)"], delay: 0.7 },
    { label: "Kernel", items: ["Scheduler", "Memory", "FS", "I/O"], delay: 0.8 },
    { label: "Hardware", items: ["x86_64", "AArch64", "RISC-V", "Gateway"], delay: 0.9 },
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.6, delay: 0.5 }}
      className="mt-12 max-w-2xl"
      aria-label="PlenumNET 5-layer architecture diagram"
      role="img"
      data-testid="hero-architecture-visual"
    >
      <div className="space-y-1.5">
        {layers.map((layer, i) => (
          <motion.div
            key={layer.label}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.4, delay: layer.delay }}
            className="flex items-center gap-3"
          >
            <span className="text-xs font-medium text-muted-foreground w-20 text-right flex-shrink-0">{layer.label}</span>
            <div className={`flex-1 flex gap-1.5 p-2 rounded-md border border-primary/${15 + i * 8} bg-primary/${5 + i * 3}`}>
              {layer.items.map((item) => (
                <span key={item} className="text-xs text-foreground/80 bg-background/60 rounded px-2 py-0.5 flex-1 text-center">{item}</span>
              ))}
            </div>
          </motion.div>
        ))}
      </div>
    </motion.div>
  );
}

function decimalToBalancedTernary(n: number): string {
  if (n === 0) return "0";
  const digits: string[] = [];
  let num = Math.abs(n);
  while (num > 0) {
    const rem = num % 3;
    if (rem === 0) { digits.push("0"); num = Math.floor(num / 3); }
    else if (rem === 1) { digits.push("+"); num = Math.floor(num / 3); }
    else { digits.push("\u2212"); num = Math.floor(num / 3) + 1; }
  }
  digits.reverse();
  if (n < 0) {
    return digits.map(d => d === "+" ? "\u2212" : d === "\u2212" ? "+" : "0").join("");
  }
  return digits.join("");
}

function HeroDemo() {
  const [inputValue, setInputValue] = useState("42");

  const num = parseInt(inputValue);
  const isValid = !isNaN(num) && inputValue.trim() !== "";
  const ternary = isValid ? decimalToBalancedTernary(num) : null;
  const binaryLen = isValid && num !== 0 ? Math.ceil(Math.log2(Math.abs(num) + 1)) : (isValid ? 1 : 0);
  const ternaryLen = ternary ? ternary.length : 0;
  const savings = binaryLen > 0 && ternaryLen > 0 ? Math.round((1 - ternaryLen / binaryLen) * 100) : 0;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay: 0.4 }}
      className="mt-8 max-w-md"
      data-testid="hero-demo-widget"
    >
      <Card className="p-4 border-primary/20 bg-card/90 backdrop-blur-sm">
        <div className="flex items-center gap-2 mb-3">
          <Zap className="w-4 h-4 text-primary" />
          <span className="text-xs font-medium text-muted-foreground">Live Ternary Converter</span>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex-1">
            <label htmlFor="hero-demo-input" className="sr-only">Enter a number to convert</label>
            <Input
              id="hero-demo-input"
              type="number"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder="Enter a number..."
              className="text-sm"
              data-testid="input-hero-demo"
            />
          </div>
          <ArrowRight className="w-4 h-4 text-muted-foreground flex-shrink-0" />
          <div className="flex-1 text-sm font-mono text-primary" data-testid="text-hero-demo-result">
            {ternary || "\u2014"}
          </div>
        </div>
        {ternary && (
          <div className="mt-2 text-xs text-muted-foreground">
            {binaryLen} binary digits → {ternaryLen} ternary trits{savings > 0 ? ` (${savings}% fewer digits)` : ""}
          </div>
        )}
      </Card>
    </motion.div>
  );
}

function HeroSection() {
  const [email, setEmail] = useState("");
  const [showSuccess, setShowSuccess] = useState(false);
  const { toast } = useToast();

  const signupMutation = useMutation({
    mutationFn: async (data: { email: string }) => {
      const res = await apiRequest("POST", "/api/developer-signup", data);
      return res.json();
    },
    onSuccess: (data) => {
      toast({ title: "You're in!", description: data.message });
      setEmail("");
      setShowSuccess(true);
    },
    onError: () => {
      toast({ title: "Something went wrong", description: "Please try again.", variant: "destructive" });
    },
  });

  return (
    <section id="hero" className="relative pt-16 pb-20 md:pt-24 md:pb-32 overflow-hidden" data-testid="section-hero" role="region" aria-labelledby="hero-title">
      <div className="absolute inset-0 w-full h-full overflow-hidden">
        <video
          autoPlay
          muted
          loop
          playsInline
          className="absolute inset-0 w-full h-full object-cover"
          data-testid="hero-video"
        >
          <source src={heroVideo} type="video/mp4" />
        </video>
        <div className="absolute inset-0 bg-gradient-to-b from-background/80 via-background/60 to-background/95 dark:from-background/85 dark:via-background/65 dark:to-background/95" />
      </div>
      
      <div className="relative z-10 max-w-7xl mx-auto px-5">
        <div className="max-w-4xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="flex flex-wrap gap-2 mb-6"
          >
            <Badge 
              variant="outline" 
              className="border-green-500/30 bg-green-500/10 text-green-700 px-4 py-1.5"
              data-testid="badge-status"
            >
              <Check className="w-3 h-3 mr-1" />
              Production Ready
            </Badge>
            <Badge 
              variant="outline" 
              className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5"
            >
              1,040 Tests Passing
            </Badge>
            <Badge 
              variant="outline" 
              className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5"
            >
              Post-Quantum Secure
            </Badge>
          </motion.div>
          
          <motion.h1 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.05 }}
            className="text-4xl md:text-5xl lg:text-6xl font-bold leading-tight mb-6"
            data-testid="text-hero-title"
            id="hero-title"
            style={{
              textShadow: `
                0 1px 0 rgba(255,255,255,0.5),
                0 -1px 0 rgba(0,0,0,0.2),
                0 2px 0 rgba(0,0,0,0.20),
                0 3px 0 rgba(0,0,0,0.18),
                0 4px 0 rgba(0,0,0,0.16),
                0 5px 0 rgba(0,0,0,0.14),
                0 6px 0 rgba(0,0,0,0.12),
                0 7px 0 rgba(0,0,0,0.10),
                0 8px 0 rgba(0,0,0,0.08),
                0 9px 0 rgba(0,0,0,0.06),
                0 10px 0 rgba(0,0,0,0.04),
                0 12px 8px rgba(0,0,0,0.14),
                0 18px 16px rgba(0,0,0,0.10),
                0 26px 30px rgba(0,0,0,0.06)
              `,
            }}
          >
            The World's First <span className="text-primary" style={{ textShadow: `
                0 1px 0 rgba(0,85,210,0.50),
                0 2px 0 rgba(0,80,200,0.45),
                0 3px 0 rgba(0,75,190,0.40),
                0 4px 0 rgba(0,70,180,0.35),
                0 5px 0 rgba(0,65,170,0.30),
                0 6px 0 rgba(0,60,160,0.26),
                0 7px 0 rgba(0,55,150,0.22),
                0 8px 0 rgba(0,50,140,0.18),
                0 9px 0 rgba(0,45,130,0.14),
                0 10px 0 rgba(0,40,120,0.10),
                0 12px 8px rgba(0,30,100,0.18),
                0 18px 16px rgba(0,30,100,0.12),
                0 26px 30px rgba(0,30,100,0.07)
              ` }}>Ternary Computing</span> Platform
          </motion.h1>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="flex items-start gap-4 mb-8"
            data-testid="text-hero-description"
          >
            <p className="text-lg md:text-xl text-muted-foreground max-w-xl">
              59% more information per digit. Femtosecond-precision timing. Post-quantum encryption. 
              A complete Rust kernel with virtual machine, network stack, and binary compatibility layer -- all shipping today.
            </p>
            <img src={plenumLogo} alt="PlenumNET" className="h-24 md:h-32 w-auto flex-shrink-0 -mt-4 -ml-4" data-testid="hero-logo" />
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="mb-10"
          >
            {showSuccess ? (
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                className="max-w-lg"
                data-testid="hero-signup-success"
              >
                <Card className="p-6 border-green-500/30 bg-green-500/5">
                  <div className="flex items-center gap-3 mb-2">
                    <Check className="w-5 h-5 text-green-500" />
                    <span className="font-semibold text-foreground">You're on the list!</span>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    We'll send you SDK access details, documentation links, and priority updates. Check your inbox soon.
                  </p>
                </Card>
              </motion.div>
            ) : (
              <>
                <form 
                  onSubmit={(e) => {
                    e.preventDefault();
                    if (email) signupMutation.mutate({ email });
                  }}
                  className="flex flex-col sm:flex-row gap-3 max-w-lg"
                  data-testid="form-hero-signup"
                >
                  <label htmlFor="hero-email" className="sr-only">Email address</label>
                  <Input
                    type="email"
                    placeholder="Enter your email for early access"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="flex-1"
                    required
                    data-testid="input-hero-email"
                    id="hero-email"
                    aria-label="Email address for early access"
                  />
                  <Button 
                    type="submit" 
                    size="default"
                    className="btn-raised"
                    disabled={signupMutation.isPending}
                    data-testid="button-hero-signup"
                  >
                    {signupMutation.isPending ? "Joining..." : "Join the Waitlist"}
                    <ArrowRight className="w-4 h-4 ml-2" />
                  </Button>
                </form>
                <p className="text-xs text-muted-foreground mt-2">No spam. Unsubscribe anytime.</p>
              </>
            )}
          </motion.div>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="flex flex-wrap gap-4 mb-12"
          >
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary btn-raised" data-testid="button-view-github">
              <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer">
                <Github className="w-4 h-4 mr-2" />
                View Source
              </a>
            </Button>
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary btn-raised" data-testid="button-view-demo">
              <a href="/ternarydb">
                <Zap className="w-4 h-4 mr-2" />
                Live Demo
              </a>
            </Button>
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary btn-raised" data-testid="button-view-docs">
              <a href="/whitepaper">
                <Shield className="w-4 h-4 mr-2" />
                Whitepaper
              </a>
            </Button>
          </motion.div>
          
          <div className="flex flex-wrap gap-8 md:gap-12">
            <AnimatedStat value="+59" suffix="%" label="vs Binary Density" delay={0.25} />
            <AnimatedStat value="1,040" label="Tests Passing" delay={0.28} />
            <AnimatedStat value="80/80" label="Milestones Complete" delay={0.31} />
            <AnimatedStat value={String(PLATFORM.VM_OPCODES)} label="VM Opcodes" delay={0.34} />
          </div>
          <HeroVisual />
          <HeroDemo />
        </div>
      </div>
    </section>
  );
}

function CapabilityCard({ cap, index }: { cap: { icon: any; title: string; description: string; stats: string }; index: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay: index * 0.08 }}
    >
      <Card 
        className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm"
        data-testid={`card-capability-${index}`}
      >
        <div className="flex items-start justify-between mb-4 gap-3">
          <div className="text-primary">
            <cap.icon className="w-8 h-8" />
          </div>
          <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary text-xs flex-shrink-0">
            {cap.stats}
          </Badge>
        </div>
        <h3 className="text-lg font-semibold mb-2 text-foreground">{cap.title}</h3>
        <p className="text-muted-foreground text-sm leading-relaxed">{cap.description}</p>
      </Card>
    </motion.div>
  );
}

function PlatformSection() {
  const capabilities = [
    {
      icon: Cpu,
      title: "Ternary Kernel",
      description: "Complete bare-metal kernel in Rust with GF(3) arithmetic, memory management, process scheduling, and IPC.",
      stats: "26 subsystems",
    },
    {
      icon: Terminal,
      title: `${PLATFORM.VM_OPCODES}-Opcode Virtual Machine`,
      description: "Register-based VM with ternary-native instructions, quantum-ternary simulation, garbage collection, and full execution engine.",
      stats: "27 registers",
    },
    {
      icon: Clock,
      title: "Femtosecond Timing (HPTP)",
      description: "High-Precision Timing Protocol with optical clock sync, designed for FINRA 613 & MiFID II timing requirements.",
      stats: "Sub-microsecond",
    },
    {
      icon: Binary,
      title: "Binary Compatibility",
      description: "Seamless Binary-Ternary Gateway enabling ternary computing on existing x86_64 hardware today.",
      stats: "Zero overhead",
    },
    {
      icon: Globe,
      title: "Torsion Network Stack",
      description: "N-dimensional torus topology with Ternary Transport Protocol, T3P application layer, and Ternary DNS.",
      stats: "Full stack",
    },
    {
      icon: Lock,
      title: "Post-Quantum Security",
      description: "CNSA 2.0 algorithm coverage with phase encryption, Lamport signatures, ternary HMAC, and sponge-based hashing resistant to quantum attacks.",
      stats: "CNSA 2.0",
    },
  ];

  return (
    <section id="platform" className="py-20 md:py-28" data-testid="section-platform" role="region" aria-labelledby="platform-title">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Complete Platform
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.06 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-platform-title"
            id="platform-title"
          >
            Everything You Need to Build on Ternary
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            From kernel primitives to application-layer protocols -- a fully integrated ternary computing stack, production-tested with 1,040 passing tests.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {capabilities.map((cap, index) => (
            <CapabilityCard key={cap.title} cap={cap} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}

function ArchitectureSection() {
  const layers = [
    {
      label: "Applications",
      items: ["PlenumDB", "Payment Listener", "SFK Core API", "Certification Service"],
      color: "bg-primary/10 border-primary/30",
    },
    {
      label: "Protocols",
      items: ["HPTP Timing", "T3P Application", "TTP Transport", "TDNS Resolution"],
      color: "bg-primary/15 border-primary/40",
    },
    {
      label: "Virtual Machine",
      items: [`${PLATFORM.VM_OPCODES}-Opcode ISA ${PLATFORM.VM_ISA_VERSION}`, "27 Ternary Registers", "TAGC Garbage Collector", "Quantum-Ternary Sim"],
      color: "bg-primary/20 border-primary/50",
    },
    {
      label: "Kernel Services",
      items: ["Process Scheduler", "Memory Manager", "Filesystem", "I/O Subsystem"],
      color: "bg-primary/25 border-primary/60",
    },
    {
      label: "Hardware Abstraction",
      items: ["x86_64 / AArch64 / RISC-V", "Binary-Ternary Gateway", "TPU / FPGA Drivers", "Optical Clock"],
      color: "bg-primary/30 border-primary/70",
    },
  ];

  return (
    <section id="architecture" className="py-20 md:py-28 bg-secondary/30" data-testid="section-architecture">
      <div className="max-w-7xl mx-auto px-5">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          <div>
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5 }}
            >
              <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
                Full-Stack Architecture
              </Badge>
            </motion.div>
            <motion.h2
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: 0.06 }}
              className="text-3xl md:text-4xl font-bold mb-6"
              data-testid="text-architecture-title"
            >
              Built From the Ground Up
            </motion.h2>
            <motion.p
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: 0.1 }}
              className="text-muted-foreground text-lg mb-8"
            >
              Five integrated layers spanning hardware abstraction to application services. 
              Every layer is production-tested, binary-compatible, and designed for the post-quantum era.
            </motion.p>
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: 0.12 }}
              className="space-y-3"
            >
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>Three bijective ternary representations: A {"{-1,0,+1}"}, B {"{0,1,2}"}, C {"{1,2,3}"}</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>Multi-architecture support: x86_64, AArch64, RISC-V</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>Runs on existing binary hardware via Binary-Ternary Gateway</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>CNSA 2.0 post-quantum algorithm coverage with ternary equivalents</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>Architecture targeting FINRA 613 & MiFID II timing requirements</span>
              </div>
            </motion.div>
          </div>

          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.12 }}
            className="space-y-3"
          >
            {layers.map((layer, index) => (
              <motion.div
                key={layer.label}
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.4, delay: 0.06 + index * 0.08 }}
              >
                <Card className={`p-4 border ${layer.color}`} data-testid={`layer-${index}`}>
                  <div className="flex items-center gap-3 mb-2">
                    <Badge variant="outline" className="border-primary/40 text-primary text-xs">
                      Layer {layers.length - index}
                    </Badge>
                    <span className="font-semibold text-sm">{layer.label}</span>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {layer.items.map((item) => (
                      <span key={item} className="text-xs text-muted-foreground bg-background/60 rounded px-2 py-1">
                        {item}
                      </span>
                    ))}
                  </div>
                </Card>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function ComponentCard({ 
  badge, 
  icon: Icon, 
  title, 
  description, 
  features, 
  index,
  link 
}: { 
  badge: string;
  icon: typeof Code;
  title: string;
  description: string;
  features: string[];
  index: number;
  link?: string;
}) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });

  const CardContent = (
    <Card 
      className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm transition-all duration-300 group hover-elevate"
      data-testid={`card-component-${index}`}
    >
      <Badge 
        variant="outline" 
        className="mb-4 border-primary/30 bg-primary/10 text-primary text-xs"
      >
        {badge}
      </Badge>
      
      <div className="text-primary mb-5 group-hover:scale-110 transition-transform duration-300">
        <Icon className="w-10 h-10" />
      </div>
      
      <h3 className="text-xl font-semibold mb-3 text-foreground">{title}</h3>
      <p className="text-muted-foreground mb-5 text-sm leading-relaxed">{description}</p>
      
      <ul className="space-y-2">
        {features.map((feature) => (
          <li key={feature} className="flex items-start gap-2.5 text-sm text-muted-foreground">
            <Check className="w-4 h-4 text-primary flex-shrink-0 mt-0.5" />
            <span>{feature}</span>
          </li>
        ))}
      </ul>
      
      {link && (
        <div className="mt-6 pt-4 border-t border-primary/10">
          <span className="text-primary text-sm font-medium flex items-center gap-1 group-hover:gap-2 transition-all">
            Explore <ArrowRight className="w-4 h-4" />
          </span>
        </div>
      )}
    </Card>
  );

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
    >
      {link ? (
        link.startsWith("http") ? (
          <a href={link} target="_blank" rel="noopener noreferrer" className="block h-full">
            {CardContent}
          </a>
        ) : (
          <Link href={link} className="block h-full">
            {CardContent}
          </Link>
        )
      ) : (
        CardContent
      )}
    </motion.div>
  );
}

function ComponentsSection() {
  const components = [
    {
      badge: "Core - 1,040 Tests",
      icon: Cpu,
      title: "Ternary Kernel",
      description: "Production-ready Rust kernel with GF(3) arithmetic, memory management, process scheduling, filesystem, and multi-architecture support.",
      link: "https://github.com/SigmaWolf-8/Ternary",
      features: [
        "Three bijective representations (A, B, C)",
        "Ticket spinlocks, semaphores, phase-aware mutexes",
        "Priority-based I/O scheduler & buffer cache",
        "CodeQL security scanning + GitHub Actions CI/CD",
      ],
    },
    {
      badge: "Virtual Machine",
      icon: Terminal,
      title: "Ternary VM (TVM)",
      description: `${PLATFORM.VM_OPCODES}-opcode register-based virtual machine (ISA ${PLATFORM.VM_ISA_VERSION}) with ternary-native arithmetic, quantum-ternary simulation opcodes, atomic operations, and automatic memory management.`,
      link: "https://github.com/SigmaWolf-8/Ternary",
      features: [
        "GF(3) ops: TAdd, TMul, TNeg, TRot, TXor",
        "27 ternary registers with flags",
        "TAGC mark-sweep garbage collector",
        "Generational GC with young/old/permanent",
      ],
    },
    {
      badge: "Live Demo",
      icon: Database,
      title: "PlenumDB",
      description: "Ternary compression engine proving 59% information density advantage with real data. Try it live right now.",
      features: [
        "59% more information per digit",
        "3:2 binary-to-ternary compression ratio",
        "Real-time benchmarks with your own data",
        "Upload CSV, JSON, XLSX for instant results",
      ],
      link: "/ternarydb",
    },
    {
      badge: "API Gateway",
      icon: Network,
      title: "Kong Konnect + Salvi API",
      description: "Full REST API for ternary operations with enterprise-grade API gateway, rate limiting, and key management.",
      link: "/api-demo",
      features: [
        "GF(3) field operations API",
        "Phase-split encryption endpoints",
        "Femtosecond timing service",
        "Kong Konnect gateway integration",
      ],
    },
    {
      badge: "Regulatory-Grade Timing",
      icon: Clock,
      title: "HPTP Timing Protocol",
      description: "Sub-microsecond precision timing with optical clock synchronization, designed for regulatory timing requirements.",
      link: "https://github.com/SigmaWolf-8/Ternary",
      features: [
        "Targeting FINRA 613 timing threshold (50ms)",
        "Targeting MiFID II granularity tiers (100us/1ms)",
        "Optical clock: Strontium, Ytterbium, Aluminum, Mercury",
        "Best master clock selection algorithm",
      ],
    },
    {
      badge: "Documentation",
      icon: Shield,
      title: "Whitepaper & Build Guides",
      description: "Comprehensive documentation covering the Unified Ternary Logic System, mathematical foundations, and build instructions.",
      link: "/whitepaper",
      features: [
        "Bijective mapping proofs",
        "Phase encryption methodology",
        "Network architecture design",
        "Step-by-step build guides + AI agent instructions",
      ],
    },
  ];

  return (
    <section id="components" className="py-20 md:py-28" data-testid="section-components">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Ship Today
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.06 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-components-title"
          >
            Deployable Components
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Every component is built, tested, and ready for integration. Not a roadmap -- this is what exists right now.
          </motion.p>
        </div>
        
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {components.map((component, index) => (
            <ComponentCard key={component.title} {...component} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}

function PerformanceSection() {
  const comparisonItems = [
    { label: "Information per Digit", current: "1.0 bit", ternary: "1.585 bits (+59%)", highlight: true },
    { label: "Storage Efficiency", current: "Baseline", ternary: "3:2 compression ratio", highlight: true },
    { label: "Quantum Resistance", current: "Vulnerable", ternary: "CNSA 2.0 ternary equivalents", highlight: true },
    { label: "Logic States", current: "2 states (0,1)", ternary: "3 states per trit", highlight: true },
    { label: "Timing Precision", current: "Milliseconds", ternary: "Femtosecond (10^-15s)", highlight: true },
    { label: "Representation Types", current: "Single (0,1)", ternary: "Three bijective (A, B, C)", highlight: true },
    { label: "Arithmetic Base", current: "Modulo 2", ternary: "GF(3) Galois field", highlight: true },
    { label: "Regulatory Timing", current: "Custom build", ternary: "Targeting FINRA 613 & MiFID II thresholds", highlight: true },
    { label: "Native Hardware", current: "Ubiquitous", ternary: "Emulated via Binary-Ternary Gateway", highlight: false },
    { label: "Ecosystem Maturity", current: "40+ years", ternary: "Emerging (v1.0)", highlight: false },
  ];

  return (
    <section id="performance" className="py-20 md:py-28 bg-secondary/30" data-testid="section-performance" role="region" aria-labelledby="performance-title">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Proven Results
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.06 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-performance-title"
            id="performance-title"
          >
            Why Ternary Wins
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Not theoretical advantages -- measured, tested, and verifiable performance improvements you can see in our live demo.
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.12 }}
        >
          <Card className="max-w-4xl mx-auto p-6 md:p-10 border-primary/10 bg-card/80 backdrop-blur-sm" role="table" aria-label="Binary vs Ternary comparison">
            <div className="flex flex-col md:flex-row items-start md:items-center justify-between mb-8 pb-6 border-b border-foreground/10 gap-2">
              <div className="flex-1 md:flex-[2]" />
              <div className="flex-1 text-center">
                <h3 className="text-xl font-semibold mb-2">Binary Systems</h3>
                <p className="text-sm text-muted-foreground">Current infrastructure</p>
              </div>
              <div className="flex-1 text-center">
                <h3 className="text-xl font-semibold text-primary mb-2">PlenumNET Ternary</h3>
                <p className="text-sm text-muted-foreground">Production-ready platform</p>
              </div>
            </div>

            <div className="space-y-0">
              {comparisonItems.map((item, index) => (
                <motion.div 
                  key={item.label}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.3, delay: index * 0.03 }}
                  className="flex flex-col md:flex-row items-start md:items-center justify-between py-4 border-b border-foreground/5 last:border-b-0 gap-2"
                  data-testid={`comparison-item-${index}`}
                >
                  <div className="flex-1 md:flex-[2] font-medium text-sm md:text-base">{item.label}</div>
                  <div className="flex-1 text-muted-foreground text-sm md:text-center">{item.current}</div>
                  <div className={`flex-1 text-sm md:text-center ${item.highlight ? "font-semibold text-primary" : "text-muted-foreground"}`}>
                    {item.ternary}
                  </div>
                </motion.div>
              ))}
            </div>

            <div className="mt-8 pt-6 border-t border-foreground/10 text-center">
              <Button asChild className="btn-raised" data-testid="button-try-demo">
                <a href="/ternarydb">
                  <Zap className="w-4 h-4 mr-2" />
                  Verify It Yourself -- Live Demo
                  <ArrowRight className="w-4 h-4 ml-2" />
                </a>
              </Button>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function TrustSignals() {
  const signals = [
    { label: "1,252+", description: "Git Commits" },
    { label: "1,040", description: "Tests Passing" },
    { label: "227", description: "Source Files" },
    { label: "175", description: "API Endpoints" },
  ];

  return (
    <section className="py-12 border-y border-primary/10 bg-muted/30" data-testid="section-trust-signals" aria-label="Trust signals and verified metrics">
      <div className="max-w-7xl mx-auto px-5">
        <div className="flex flex-col md:flex-row items-center justify-between gap-8">
          <div className="flex flex-wrap items-center justify-center gap-6">
            <Badge variant="outline" className="border-green-500/30 bg-green-500/10 text-green-700 dark:text-green-400 px-3 py-1">
              <Check className="w-3 h-3 mr-1" />
              CNSA 2.0 Architecture
            </Badge>
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-3 py-1">
              <Shield className="w-3 h-3 mr-1" />
              Targeting FIPS 140-3
            </Badge>
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-3 py-1">
              <Clock className="w-3 h-3 mr-1" />
              Targeting FINRA 613
            </Badge>
          </div>
          <div className="flex flex-wrap items-center justify-center gap-8">
            {signals.map((s) => (
              <div key={s.description} className="text-center" data-testid={`trust-metric-${s.description.toLowerCase().replace(/\s+/g, '-')}`}>
                <div className="text-lg font-bold text-foreground">{s.label}</div>
                <div className="text-xs text-muted-foreground">{s.description}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function CodeSnippet() {
  const [copied, setCopied] = useState(false);
  const code = `curl -X POST https://plenumnet.replit.app/api/salvi/ternary/convert \\
  -H "Content-Type: application/json" \\
  -d '{"value": 42, "from": "B", "to": "A"}'`;

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section className="py-16 md:py-20" data-testid="section-code-snippet">
      <div className="max-w-4xl mx-auto px-5">
        <div className="text-center mb-8">
          <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
            Try It Now
          </Badge>
          <h2 className="text-2xl md:text-3xl font-bold mb-3" data-testid="text-code-snippet-title">
            One API Call Away
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            175 live endpoints. No SDK required. Start converting ternary operations with a single HTTP request.
          </p>
        </div>
        <Card className="p-0 overflow-hidden border-primary/10 bg-card/80">
          <div className="flex items-center justify-between px-4 py-2 bg-muted/50 border-b border-primary/10">
            <div className="flex items-center gap-2">
              <Terminal className="w-4 h-4 text-muted-foreground" />
              <span className="text-xs text-muted-foreground font-medium">Quick Start</span>
            </div>
            <Button variant="ghost" size="sm" className="btn-raised" onClick={handleCopy} data-testid="button-copy-code" aria-label="Copy code to clipboard">
              {copied ? <Check className="w-3.5 h-3.5 text-green-500" /> : <Copy className="w-3.5 h-3.5" />}
              <span className="ml-1.5 text-xs">{copied ? "Copied" : "Copy"}</span>
            </Button>
          </div>
          <pre className="p-5 text-sm font-mono text-foreground/90 overflow-x-auto" data-testid="text-code-content">
            <code>{code}</code>
          </pre>
        </Card>
        <div className="flex justify-center mt-6">
          <Button variant="outline" asChild className="btn-raised" data-testid="button-explore-api">
            <Link href="/api-demo">
              Explore All 175 Endpoints
              <ArrowRight className="w-4 h-4 ml-2" />
            </Link>
          </Button>
        </div>
      </div>
    </section>
  );
}

function CalendarPreviewSection() {
  const problems = [
    {
      problem: "Calendar Fragmentation",
      description: "12+ epoch dates with incompatible rules. Hebrew lunisolar, Islamic lunar, and Mayan vigesimal need 144 conversion functions.",
      solution: "Single JDN intermediary: O(n) instead of O(n\u00B2). 24 functions cover all 12 calendars.",
      icon: Globe,
    },
    {
      problem: "Y2038 Overflow",
      description: "32-bit timestamps overflow January 19, 2038. Billions of systems will fail.",
      solution: "128-bit femtosecond timestamps. No rollover until year ~3.9 x 10\u00B2\u2079.",
      icon: Clock,
    },
    {
      problem: "Precision Drift",
      description: "IEEE 754 floating-point errors accumulate. 1ms/day becomes 365ms/year -- fails regulatory compliance.",
      solution: "Integer-only calculations. Zero accumulation error. Architecture targeting FINRA 613 & MiFID II timing thresholds.",
      icon: Shield,
    },
  ];

  return (
    <section className="py-20 md:py-28" data-testid="section-calendar-preview">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Calendar Synchronization API
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.06 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-calendar-preview-title"
          >
            One Timestamp. Every Calendar. 30,000 Years.
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-3xl mx-auto"
          >
            Convert any date across 12 major calendar systems -- from Mayan Long Count to Islamic Hijri -- with femtosecond precision.
            The 13-Moon Harmonic Calendar places the Day Out of Time at the golden ratio point (364/\u03C6 = Day 225, November 11),
            creating an 8/5 Fibonacci moon split that embeds organic growth mathematics into temporal architecture.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-3 gap-6 mb-12">
          {problems.map((item, index) => (
            <motion.div
              key={item.problem}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.06 }}
            >
              <Card 
                className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm"
                data-testid={`card-calendar-problem-${index}`}
              >
                <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary/10 text-primary mb-4">
                  <item.icon className="w-6 h-6" />
                </div>
                <h3 className="text-lg font-semibold mb-2">{item.problem}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed mb-4">{item.description}</p>
                <div className="pt-4 border-t border-primary/10">
                  <div className="flex items-start gap-2">
                    <Check className="w-4 h-4 text-primary flex-shrink-0 mt-0.5" />
                    <p className="text-sm font-medium">{item.solution}</p>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.18 }}
        >
          <Card className="max-w-4xl mx-auto p-6 md:p-8 border-primary/10 bg-card/80 backdrop-blur-sm">
            <div className="grid sm:grid-cols-3 gap-6 text-center mb-8">
              <div>
                <div className="text-3xl font-bold text-primary">12</div>
                <div className="text-sm text-muted-foreground mt-1">Calendar Systems</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-primary">30,000+</div>
                <div className="text-sm text-muted-foreground mt-1">Years of Coverage</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-primary">10\u207B\u00B9\u2075s</div>
                <div className="text-sm text-muted-foreground mt-1">Timing Precision</div>
              </div>
            </div>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-4 pt-6 border-t border-foreground/10">
              <Button asChild className="btn-raised" data-testid="button-explore-calendar">
                <Link href="/calendar">
                  <Globe className="w-4 h-4 mr-2" />
                  Explore Calendar API
                  <ArrowRight className="w-4 h-4 ml-2" />
                </Link>
              </Button>
              <Button variant="outline" asChild className="btn-raised" data-testid="button-calendar-docs">
                <Link href="/docs">View Documentation</Link>
              </Button>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function TargetMarketsSection() {
  const markets = [
    {
      icon: Building2,
      title: "Financial Services",
      description: "Precision timing targeting FINRA 613 & MiFID II thresholds for high-frequency trading, regulatory reporting, and immutable audit trails.",
      stats: "Regulatory-grade timing",
    },
    {
      icon: FlaskConical,
      title: "Research & HPC",
      description: "59% data density improvement for scientific computing, genomics, and large-scale simulations. Less bandwidth, more throughput.",
      stats: "59% density gain",
    },
    {
      icon: Factory,
      title: "Industrial IoT & Edge",
      description: "Bandwidth-optimized edge computing for manufacturing, autonomous systems, and real-time sensor networks.",
      stats: "3:2 compression",
    },
    {
      icon: Server,
      title: "Blockchain & DeFi",
      description: "Post-quantum secure witnessing, payment settlement via XRPL and Algorand, with Hedera HCS consensus integration.",
      stats: "Quantum-resistant",
    },
    {
      icon: Shield,
      title: "Defense & Intelligence",
      description: "Phase encryption with timing-window enforcement and Lamport one-time signatures for maximum security posture.",
      stats: "Post-quantum",
    },
    {
      icon: Activity,
      title: "Telecommunications",
      description: "Torsion Network topology with geodesic routing optimized for next-generation network infrastructure.",
      stats: "N-dimensional",
    },
  ];

  return (
    <section id="markets" className="py-20 md:py-28" data-testid="section-markets">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Market Opportunity
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.06 }}
            className="text-3xl md:text-4xl font-bold mb-4"
          >
            Built for Industries That Demand More
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Targeted deployments with measurable ROI across sectors where efficiency, security, and compliance are non-negotiable.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {markets.map((market, index) => (
            <motion.div
              key={market.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.05 }}
            >
              <Card 
                className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm"
                data-testid={`card-market-${index}`}
              >
                <div className="flex items-start justify-between mb-4 gap-3">
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary/10 text-primary flex-shrink-0">
                    <market.icon className="w-6 h-6" />
                  </div>
                  <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary text-xs flex-shrink-0">
                    {market.stats}
                  </Badge>
                </div>
                <h3 className="text-lg font-semibold mb-2">{market.title}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{market.description}</p>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function DeveloperCTASection() {
  const [email, setEmail] = useState("");
  const [name, setName] = useState("");
  const [showSuccess, setShowSuccess] = useState(false);
  const { toast } = useToast();

  const { data: countData } = useQuery<{ count: number }>({
    queryKey: ["/api/developer-signup/count"],
  });

  const signupMutation = useMutation({
    mutationFn: async (data: { email: string; name?: string }) => {
      const res = await apiRequest("POST", "/api/developer-signup", data);
      return res.json();
    },
    onSuccess: (data) => {
      toast({ title: "You're in!", description: data.message });
      setEmail("");
      setName("");
      setShowSuccess(true);
    },
    onError: () => {
      toast({ title: "Something went wrong", description: "Please try again.", variant: "destructive" });
    },
  });

  return (
    <section id="early-access" className="py-20 md:py-28 bg-secondary/30" data-testid="section-developer-cta">
      <div className="max-w-7xl mx-auto px-5">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
        >
          <Card className="max-w-4xl mx-auto p-8 md:p-12 lg:p-16 border-0 bg-gradient-to-br from-primary to-primary/80 text-primary-foreground">
            <div className="text-center mb-8">
              <h2 className="text-3xl md:text-4xl font-bold mb-4" data-testid="text-cta-title">
                Request Developer Preview
              </h2>
              <p className="text-lg opacity-90 max-w-2xl mx-auto mb-2">
                Get early access to the PlenumNET SDK, developer documentation, and direct support from the core team. 
                Be among the first to build applications on ternary infrastructure.
              </p>
              {countData && countData.count > 0 && (
                <p className="text-sm opacity-70" data-testid="text-signup-count">
                  {countData.count} developer{countData.count !== 1 ? "s" : ""} already signed up
                </p>
              )}
            </div>

            {showSuccess ? (
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                className="max-w-lg mx-auto text-center"
                data-testid="developer-signup-success"
              >
                <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary-foreground/20 mb-4">
                  <Check className="w-8 h-8 text-primary-foreground" />
                </div>
                <h3 className="text-2xl font-bold mb-2">Application Received!</h3>
                <p className="opacity-90">
                  Our team will review your request and reach out within 48 hours with SDK access credentials and onboarding documentation.
                </p>
              </motion.div>
            ) : (
              <>
                <form
                  onSubmit={(e) => {
                    e.preventDefault();
                    if (email) signupMutation.mutate({ email, name: name || undefined });
                  }}
                  className="max-w-lg mx-auto space-y-3"
                  data-testid="form-developer-signup"
                >
                  <label htmlFor="signup-name" className="sr-only">Your name</label>
                  <Input
                    type="text"
                    placeholder="Your name (optional)"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="bg-primary-foreground/10 border-primary-foreground/20 text-primary-foreground placeholder:text-primary-foreground/50"
                    data-testid="input-signup-name"
                    id="signup-name"
                  />
                  <label htmlFor="signup-email" className="sr-only">Email address</label>
                  <Input
                    type="email"
                    placeholder="developer@company.com"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                    className="bg-primary-foreground/10 border-primary-foreground/20 text-primary-foreground placeholder:text-primary-foreground/50"
                    data-testid="input-signup-email"
                    id="signup-email"
                  />
                  <Button 
                    type="submit" 
                    size="lg"
                    variant="secondary"
                    className="w-full bg-background text-foreground btn-raised"
                    disabled={signupMutation.isPending}
                    data-testid="button-developer-signup"
                  >
                    {signupMutation.isPending ? "Submitting..." : "Apply for SDK Access"}
                    <ArrowRight className="w-4 h-4 ml-2" />
                  </Button>
                </form>

                <div className="flex justify-center mt-4">
                  <Button variant="outline" size="lg" className="border-primary-foreground/30 text-primary-foreground btn-raised" asChild data-testid="button-book-demo">
                    <a href="mailto:Rsalvi@Salvigroup.com?subject=PlenumNET%20Demo%20Request">
                      <Calendar className="w-4 h-4 mr-2" />
                      Book a Demo
                    </a>
                  </Button>
                </div>

                <div className="flex flex-wrap gap-6 justify-center mt-8 text-sm opacity-80">
                  <div className="flex items-center gap-2">
                    <Check className="w-4 h-4" />
                    <span>SDK Access</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Check className="w-4 h-4" />
                    <span>Developer Docs</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Check className="w-4 h-4" />
                    <span>Core Team Support</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Check className="w-4 h-4" />
                    <span>Priority Updates</span>
                  </div>
                </div>
              </>
            )}
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function ChangelogSection() {
  const [commits, setCommits] = useState<any[]>([]);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    fetch("https://api.github.com/repos/SigmaWolf-8/Ternary/commits?per_page=5")
      .then(r => r.ok ? r.json() : Promise.reject())
      .then(data => { setCommits(data); setLoaded(true); })
      .catch(() => setLoaded(true));
  }, []);

  if (!loaded || commits.length === 0) return null;

  const timeAgo = (dateStr: string) => {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    if (days < 30) return `${days}d ago`;
    return `${Math.floor(days / 30)}mo ago`;
  };

  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-changelog">
      <div className="max-w-4xl mx-auto px-5">
        <div className="text-center mb-10">
          <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
            Active Development
          </Badge>
          <h2 className="text-2xl md:text-3xl font-bold mb-3">Recent Updates</h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            Continuous development on the Ternary kernel and platform.
          </p>
        </div>
        <div className="space-y-3">
          {commits.map((commit: any, i: number) => (
            <motion.div
              key={commit.sha}
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.3, delay: i * 0.05 }}
            >
              <a
                href={commit.html_url}
                target="_blank"
                rel="noopener noreferrer"
                className="block"
                data-testid={`changelog-commit-${i}`}
              >
                <Card className="p-4 border-primary/10 bg-card/70 hover-elevate">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-foreground truncate">{commit.commit.message.split("\n")[0]}</p>
                      <p className="text-xs text-muted-foreground mt-1">
                        {commit.commit.author?.name || "Unknown"} 
                      </p>
                    </div>
                    <span className="text-xs text-muted-foreground flex-shrink-0">{timeAgo(commit.commit.author?.date)}</span>
                  </div>
                </Card>
              </a>
            </motion.div>
          ))}
        </div>
        <div className="flex justify-center mt-6">
          <Button variant="outline" asChild className="btn-raised" data-testid="button-view-all-commits">
            <a href="https://github.com/SigmaWolf-8/Ternary/commits" target="_blank" rel="noopener noreferrer">
              View All Commits
              <ArrowRight className="w-4 h-4 ml-2" />
            </a>
          </Button>
        </div>
      </div>
    </section>
  );
}

function Footer() {
  const footerLinks = {
    Platform: [
      { label: "Ternary Kernel", href: "https://github.com/SigmaWolf-8/Ternary" },
      { label: "PlenumDB Demo", href: "/ternarydb" },
      { label: "Salvi API", href: "/api-demo" },
      { label: "Kong Gateway", href: "/kong-konnect" },
    ],
    Developers: [
      { label: "Whitepaper", href: "/whitepaper" },
      { label: "API Demo", href: "/api-demo" },
      { label: "Build Guide", href: "https://github.com/SigmaWolf-8/Ternary/blob/main/KERNEL-BUILD-GUIDE.md" },
      { label: "GitHub", href: "https://github.com/SigmaWolf-8/Ternary" },
    ],
    Company: [
      { label: "About", href: "/about" },
      { label: "Documentation", href: "/docs" },
      { label: "CNSA 2.0 Compliance", href: "/compliance" },
      { label: "Contact", href: "/contact" },
    ],
    Legal: [
      { label: "Privacy", href: "/privacy" },
      { label: "Terms", href: "/terms" },
      { label: "Security", href: "/security" },
      { label: "Acceptable Use", href: "/aup" },
    ],
  };

  return (
    <footer className="bg-background border-t border-primary/10 py-16" data-testid="footer">
      <div className="max-w-7xl mx-auto px-5">
        <div className="grid grid-cols-2 md:grid-cols-5 gap-8 mb-12">
          <div className="col-span-2 md:col-span-1">
            <a href="/" className="flex items-center gap-2 text-primary font-bold text-xl mb-4" data-testid="link-footer-logo">
              <img src={plenumLogo} alt="PlenumNET" className="w-4 h-4" />
              <span>PlenumNET</span>
            </a>
            <p className="text-sm text-muted-foreground mb-4">
              The world's first ternary computing platform. Post-quantum security, 59% density advantage, shipping today.
            </p>
            <div className="flex gap-3">
              <a 
                href="https://github.com/SigmaWolf-8/Ternary" 
                target="_blank" 
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-github"
                aria-label="GitHub repository"
              >
                <Github className="w-5 h-5" />
              </a>
              <a 
                href="mailto:Rsalvi@Salvigroup.com" 
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-email"
                aria-label="Send email"
              >
                <Mail className="w-5 h-5" />
              </a>
            </div>
          </div>

          {Object.entries(footerLinks).map(([category, links]) => (
            <div key={category}>
              <h4 className="font-semibold mb-4 text-foreground">{category}</h4>
              <ul className="space-y-2">
                {links.map((link) => (
                  <li key={link.label}>
                    {link.href.startsWith("/") && !link.href.startsWith("/#") ? (
                      <Link 
                        href={link.href}
                        className="text-sm text-muted-foreground hover:text-primary transition-colors"
                      >
                        {link.label}
                      </Link>
                    ) : (
                      <a 
                        href={link.href} 
                        className="text-sm text-muted-foreground hover:text-primary transition-colors"
                        target={link.href.startsWith("http") ? "_blank" : undefined}
                        rel={link.href.startsWith("http") ? "noopener noreferrer" : undefined}
                      >
                        {link.label}
                      </a>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="pt-8 border-t border-primary/10 text-center text-sm text-muted-foreground">
          <p>All Rights Reserved and Preserved | &copy; Capomastro Holdings Ltd 2026</p>
        </div>
      </div>
    </footer>
  );
}

export default function Landing() {
  return (
    <>
      <HeroSection />
      <PlatformSection />
      <ArchitectureSection />
      <ComponentsSection />
      <PerformanceSection />
      <TrustSignals />
      <CodeSnippet />
      <TargetMarketsSection />
      <ChangelogSection />
      <DeveloperCTASection />
    </>
  );
}
