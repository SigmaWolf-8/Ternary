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

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { ArrowLeft, ArrowRight, Check, Shield, Cpu, Globe, FileCode, Server, Users, FlaskConical, GitBranch, Lock, Rocket } from "lucide-react";
import { motion, useInView } from "framer-motion";
import { useRef } from "react";
import { Link } from "wouter";
import { PLATFORM } from "@shared/constants";

function AnimatedSection({ children, delay = 0 }: { children: React.ReactNode; delay?: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay }}
    >
      {children}
    </motion.div>
  );
}

const milestones = [
  { value: "1,040", label: "Tests Passing", icon: Check },
  { value: "227", label: "Source Files", icon: FileCode },
  { value: String(PLATFORM.API_ENDPOINTS), label: "API Endpoints", icon: Server },
  { value: "CNSA 2.0", label: "Architecture", icon: Shield },
];

export default function AboutPage() {
  return (
    <div className="min-h-screen bg-background" data-testid="page-about">
      <div className="max-w-7xl mx-auto px-5 py-8">
        <div className="mb-8">
          <Button
            variant="ghost"
            size="sm"
            asChild
            data-testid="link-back-home"
            aria-label="Back to home page"
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
          className="mb-12"
        >
          <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
            About Us
          </Badge>
          <h1 className="text-4xl md:text-5xl font-bold mb-4" data-testid="text-about-title">
            About PlenumNET
          </h1>
          <p className="text-lg text-muted-foreground max-w-3xl" data-testid="text-about-subtitle">
            Building the future of computing from the ground up with post-quantum ternary architecture.
          </p>
        </motion.div>

        <AnimatedSection delay={0.1}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-company-overview">
            <div className="flex items-start gap-4 mb-4">
              <div className="text-primary">
                <Cpu className="w-8 h-8" />
              </div>
              <div>
                <h2 className="text-2xl font-bold mb-2">Capomastro Holdings Ltd.</h2>
                <p className="text-muted-foreground text-sm">Applied Physics Division &middot; Alberta, Canada</p>
              </div>
            </div>
            <p className="text-muted-foreground leading-relaxed" data-testid="text-company-description">
              Capomastro Holdings Ltd. is a Canadian technology company focused on advancing
              the frontiers of computing through its Applied Physics Division. Based in Alberta, Canada,
              we are developing PlenumNET -- the world's first deployable post-quantum ternary computing
              platform, designed to operate on existing binary hardware while delivering the efficiency
              and security advantages of base-3 computation.
            </p>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.15}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-mission">
            <div className="flex items-start gap-4 mb-4">
              <div className="text-primary">
                <Globe className="w-8 h-8" />
              </div>
              <div>
                <h2 className="text-2xl font-bold mb-2">Our Mission</h2>
              </div>
            </div>
            <p className="text-muted-foreground leading-relaxed" data-testid="text-mission-description">
              Our mission is to make post-quantum ternary computing practical and accessible. Ternary
              logic delivers 59% more information per digit compared to binary, and when combined with
              post-quantum cryptographic primitives aligned to the NSA's CNSA 2.0 suite, it provides a
              computing foundation that is both more efficient and more secure than anything available
              today. PlenumNET is not a research project -- it is a production-grade platform with a
              complete kernel, virtual machine, network stack, and application layer, all shipping now.
            </p>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.2}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-vision">
            <div className="flex items-start gap-4 mb-4">
              <div className="text-primary">
                <Users className="w-8 h-8" />
              </div>
              <div>
                <h2 className="text-2xl font-bold mb-2">Technical Foundation</h2>
              </div>
            </div>
            <p className="text-muted-foreground leading-relaxed mb-4" data-testid="text-vision-description">
              PlenumNET is built on the Salvi Framework -- a vertically integrated ternary computing
              stack that spans from hardware abstraction to application-layer protocols. The framework
              includes a bare-metal Rust kernel with GF(3) field arithmetic, a {PLATFORM.VM_OPCODES}-opcode register-based
              virtual machine with quantum-ternary simulation, femtosecond-precision timing protocols, and a complete torsion network
              topology. Every component has been engineered from first principles to operate natively in
              base-3 while maintaining full binary compatibility through our Binary-Ternary Gateway.
            </p>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">Salvi Framework</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">GF(3) Arithmetic</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">{PLATFORM.VM_OPCODES}-Opcode VM</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">HPTP Timing</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">Post-Quantum Crypto</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">Tribonacci Test Oracle</Badge>
            </div>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.22}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-version-history">
            <div className="flex items-start gap-4 mb-6">
              <div className="text-primary">
                <GitBranch className="w-8 h-8" />
              </div>
              <div>
                <h2 className="text-2xl font-bold mb-1">PlenumNET Version History</h2>
                <p className="text-muted-foreground text-sm">Current release: ISA v2.1 &middot; Crypto Module v3.0.0 target</p>
              </div>
            </div>
            <div className="space-y-6">
              <div className="relative pl-8 border-l-2 border-primary/20">
                <div className="absolute -left-[9px] top-0.5 w-4 h-4 rounded-full bg-primary/20 flex items-center justify-center">
                  <div className="w-2 h-2 rounded-full bg-primary" />
                </div>
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="font-semibold" data-testid="text-version-v3">v3.0.0</h3>
                  <Badge variant="outline" className="border-yellow-500/40 bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 text-xs">Target</Badge>
                </div>
                <p className="text-sm text-muted-foreground mb-2">
                  Salvi Ternary Cryptographic Module &mdash; FIPS 140-3 CMVP submission release. Adds
                  certified post-quantum primitives (TL-KEM, TL-DSA), hardware-bound key storage, and
                  CNSA 2.0 full compliance. Formal verification coverage target: 85% of critical path.
                </p>
                <div className="flex flex-wrap gap-1.5">
                  <Badge variant="outline" className="text-xs border-primary/20"><Lock className="w-3 h-3 mr-1" />FIPS 140-3</Badge>
                  <Badge variant="outline" className="text-xs border-primary/20">CNSA 2.0 Full</Badge>
                  <Badge variant="outline" className="text-xs border-primary/20">TL-KEM / TL-DSA</Badge>
                  <Badge variant="outline" className="text-xs border-primary/20">Hardware Key Storage</Badge>
                </div>
              </div>

              <div className="relative pl-8 border-l-2 border-primary/40">
                <div className="absolute -left-[9px] top-0.5 w-4 h-4 rounded-full bg-primary flex items-center justify-center">
                  <div className="w-2 h-2 rounded-full bg-background" />
                </div>
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="font-semibold" data-testid="text-version-v2">v2.1</h3>
                  <Badge variant="outline" className="border-green-500/40 bg-green-500/10 text-green-600 dark:text-green-400 text-xs">Current</Badge>
                </div>
                <p className="text-sm text-muted-foreground mb-2">
                  {PLATFORM.VM_OPCODES}-opcode ISA with quantum-ternary simulation (opcodes 0xA0&ndash;0xAF), 27 ternary registers,
                  TAGC garbage collector, torsion network stack (TTP/T3P/TDNS), 5-tier HPTP femtosecond timing,
                  phase encryption, and the 28-Dimension Agent Array. Full binary compatibility via
                  Binary-Ternary Gateway.
                </p>
                <div className="flex flex-wrap gap-1.5">
                  <Badge variant="outline" className="text-xs border-primary/20"><Cpu className="w-3 h-3 mr-1" />{PLATFORM.VM_OPCODES} Opcodes</Badge>
                  <Badge variant="outline" className="text-xs border-primary/20">Quantum-Ternary Sim</Badge>
                  <Badge variant="outline" className="text-xs border-primary/20">HPTP 5-Tier</Badge>
                  <Badge variant="outline" className="text-xs border-primary/20">Torsion Network</Badge>
                  <Badge variant="outline" className="text-xs border-primary/20">28D Agent Array</Badge>
                </div>
              </div>

              <div className="relative pl-8 border-l-2 border-muted-foreground/20">
                <div className="absolute -left-[9px] top-0.5 w-4 h-4 rounded-full bg-muted-foreground/30 flex items-center justify-center">
                  <div className="w-2 h-2 rounded-full bg-muted-foreground/60" />
                </div>
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="font-semibold" data-testid="text-version-v1">v1.0</h3>
                  <Badge variant="outline" className="text-xs border-muted-foreground/30">Foundation</Badge>
                </div>
                <p className="text-sm text-muted-foreground mb-2">
                  Initial Salvi Framework release &mdash; 62-opcode ISA with GF(3) field arithmetic,
                  bare-metal Rust kernel, bitmap frame allocator, capability-based access control,
                  ternary hash and sponge primitives, and the core virtual machine. Established the
                  balanced ternary computing foundation that all subsequent versions build upon.
                </p>
                <div className="flex flex-wrap gap-1.5">
                  <Badge variant="outline" className="text-xs border-muted-foreground/20"><Rocket className="w-3 h-3 mr-1" />62 Opcodes</Badge>
                  <Badge variant="outline" className="text-xs border-muted-foreground/20">GF(3) Arithmetic</Badge>
                  <Badge variant="outline" className="text-xs border-muted-foreground/20">Rust Kernel</Badge>
                  <Badge variant="outline" className="text-xs border-muted-foreground/20">Capability System</Badge>
                </div>
              </div>
            </div>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.26}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-test-oracle">
            <div className="flex items-start gap-4 mb-4">
              <div className="text-primary">
                <FlaskConical className="w-8 h-8" />
              </div>
              <div>
                <h2 className="text-2xl font-bold mb-2">Tribonacci Canonical Test Oracle</h2>
              </div>
            </div>
            <p className="text-muted-foreground leading-relaxed mb-4" data-testid="text-test-oracle-description">
              Every ternary operation in the Salvi Framework is validated against the Tribonacci word -- the
              fixed point of the morphism 0&#8594;01, 1&#8594;02, 2&#8594;0. This 3-automatic sequence over
              the alphabet &#123;0, 1, 2&#125; serves as the canonical test oracle: any correct bijective
              ternary encoder, decoder, or arithmetic operation must preserve its structural properties.
              The Tribonacci word links the recurrence T(n) = T(n-1) + T(n-2) + T(n-3) to the ternary
              circle geometry (364&#176; = 111111&#8323;) and Borromean topology, providing a single
              mathematical object that validates all three modules simultaneously.
            </p>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">3-Automatic Sequence</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">Morphism: 0&#8594;01, 1&#8594;02, 2&#8594;0</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">Rep A/B/C Invariant</Badge>
              <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">Cross-Module Verification</Badge>
            </div>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.3}>
          <div className="mb-8">
            <h2 className="text-2xl font-bold mb-6" data-testid="text-milestones-title">Key Milestones</h2>
            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
              {milestones.map((milestone, index) => (
                <Card
                  key={milestone.label}
                  className="p-6 border-primary/10 text-center"
                  data-testid={`card-milestone-${index}`}
                >
                  <milestone.icon className="w-6 h-6 text-primary mx-auto mb-3" />
                  <div className="text-3xl font-bold text-primary mb-1">{milestone.value}</div>
                  <div className="text-sm text-muted-foreground">{milestone.label}</div>
                </Card>
              ))}
            </div>
          </div>
        </AnimatedSection>

        <AnimatedSection delay={0.35}>
          <Card className="p-6 md:p-8 border-primary/10 text-center" data-testid="section-cta">
            <h2 className="text-2xl font-bold mb-3">Join the Future of Computing</h2>
            <p className="text-muted-foreground mb-6 max-w-xl mx-auto">
              Be among the first to build on the world's only deployable ternary computing platform.
              Request early access to the SDK, documentation, and developer tools.
            </p>
            <div className="flex flex-wrap justify-center gap-3">
              <Button asChild data-testid="button-early-access">
                <Link href="/#early-access">
                  Request Early Access
                  <ArrowRight className="w-4 h-4 ml-2" />
                </Link>
              </Button>
              <Button variant="outline" asChild data-testid="button-contact-us">
                <Link href="/contact">
                  Contact Us
                </Link>
              </Button>
            </div>
          </Card>
        </AnimatedSection>
      </div>
    </div>
  );
}
