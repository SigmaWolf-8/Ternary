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
import { ArrowLeft, ArrowRight, Circle, Orbit, Compass, Layers, RotateCw } from "lucide-react";
import { motion, useInView } from "framer-motion";
import { useRef } from "react";
import { Link } from "wouter";

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

function ClockDiagram() {
  const points = 28;
  const radius = 120;
  const centerX = 150;
  const centerY = 150;

  const dots = Array.from({ length: points }, (_, i) => {
    const angle = (2 * Math.PI * i) / points - Math.PI / 2;
    return {
      x: centerX + radius * Math.cos(angle),
      y: centerY + radius * Math.sin(angle),
      label: i,
    };
  });

  const jumpSequence: number[] = [];
  let current = 0;
  for (let i = 0; i < 28; i++) {
    jumpSequence.push(current);
    current = (current + 13) % 28;
  }

  const pathSegments = jumpSequence.map((from, i) => {
    if (i === jumpSequence.length - 1) return null;
    const to = jumpSequence[i + 1];
    return { from: dots[from], to: dots[to], index: i };
  }).filter(Boolean);

  return (
    <div className="flex justify-center" data-testid="diagram-28-clock">
      <svg viewBox="0 0 300 300" className="w-64 h-64 md:w-80 md:h-80">
        <circle
          cx={centerX}
          cy={centerY}
          r={radius}
          fill="none"
          stroke="currentColor"
          strokeWidth="1"
          className="text-muted-foreground/30"
        />
        {pathSegments.map((seg, i) => (
          <line
            key={i}
            x1={seg!.from.x}
            y1={seg!.from.y}
            x2={seg!.to.x}
            y2={seg!.to.y}
            stroke="currentColor"
            strokeWidth="0.5"
            className="text-primary/20"
          />
        ))}
        {dots.map((dot) => (
          <g key={dot.label}>
            <circle
              cx={dot.x}
              cy={dot.y}
              r={dot.label === 0 ? 6 : 4}
              fill="currentColor"
              className={dot.label === 0 ? "text-primary" : "text-primary/60"}
            />
            {dot.label % 7 === 0 && (
              <text
                x={dot.x + (dot.x > centerX ? 10 : -10)}
                y={dot.y + (dot.y > centerY ? 12 : -6)}
                textAnchor={dot.x > centerX ? "start" : "end"}
                className="text-muted-foreground fill-current"
                fontSize="10"
              >
                {dot.label}
              </text>
            )}
          </g>
        ))}
        <text
          x={centerX}
          y={centerY + 4}
          textAnchor="middle"
          className="text-foreground fill-current font-semibold"
          fontSize="14"
        >
          Z&#8322;&#8328;
        </text>
      </svg>
    </div>
  );
}

const concepts = [
  {
    icon: Circle,
    title: "28-Point Circle",
    description: "A circle divided into 28 equal positions, like a clock with 28 hours. Each point represents one ternary radian of 13 degrees.",
  },
  {
    icon: Compass,
    title: "Generator 13",
    description: "Jumping forward 13 marks on the 28-point circle visits every position exactly once before returning to start. This works because 13 and 28 share no common divisor.",
  },
  {
    icon: Orbit,
    title: "Tribonacci Coverage",
    description: "The Tribonacci sequence, taken modulo 28, eventually visits every position from 0 to 27. The sequence covers all the spots on the 28-point circle.",
  },
  {
    icon: RotateCw,
    title: "28-Fold vs 3-Fold",
    description: "Traditional systems use 3-fold symmetry (equilateral triangle, 120-degree rotations). PlenumNET replaces this with 28-fold symmetry providing far greater resolution.",
  },
];

export default function Tribonacci28DSPage() {
  return (
    <div className="min-h-screen bg-background" data-testid="page-tribonacci-28ds">
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
            Architecture
          </Badge>
          <h1 className="text-4xl md:text-5xl font-bold mb-4" data-testid="text-28ds-title">
            Tribonacci 28-Dimension Symmetry
          </h1>
          <p className="text-lg text-muted-foreground max-w-3xl" data-testid="text-28ds-subtitle">
            The geometric backbone of ternary computation, where the Tribonacci constant meets cyclic group theory.
          </p>
        </motion.div>

        <AnimatedSection delay={0.1}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-28ds-overview">
            <div className="flex items-start gap-4 mb-4">
              <div className="text-primary">
                <Layers className="w-8 h-8" />
              </div>
              <div>
                <h2 className="text-2xl font-bold mb-2">The 28-Hour Clock</h2>
              </div>
            </div>
            <div className="grid md:grid-cols-2 gap-8 items-center">
              <div>
                <p className="text-muted-foreground leading-relaxed mb-4" data-testid="text-28ds-description">
                  Imagine you have a circle marked with 28 evenly spaced points, like the numbers on a clock
                  but with 28 hours instead of 12. If you start at 0 and jump forward 13 marks each time,
                  you'll hit every single one of those 28 spots before coming back to the start (because 13
                  and 28 have no common divisor). This is the 28-fold symmetry -- the circle is divided into
                  28 equal slices.
                </p>
                <p className="text-muted-foreground leading-relaxed mb-4">
                  Now there's a special sequence of numbers called the Tribonacci word. It's like the Fibonacci
                  sequence, but each new number is the sum of the three before it. If you take those numbers
                  and only keep the remainder when you divide by 28, you'll eventually get every whole number
                  from 0 to 27. So the sequence "covers all the spots" on our 28-hour clock.
                </p>
                <p className="text-muted-foreground leading-relaxed">
                  In older, more common systems you'd have a 3-fold symmetry -- think of an equilateral triangle
                  that looks the same when you rotate it 120&#176;. Here we've replaced that with 28-fold
                  symmetry. Instead of three evenly spaced rotations, we have twenty-eight. The whole process
                  is like taking a walk that jumps from one clock hour to another, following a rule tied to the
                  Tribonacci constant (a special number about 1.839).
                </p>
              </div>
              <ClockDiagram />
            </div>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.15}>
          <div className="mb-8">
            <h2 className="text-2xl font-bold mb-6" data-testid="text-concepts-title">Core Concepts</h2>
            <div className="grid sm:grid-cols-2 gap-4">
              {concepts.map((concept, index) => (
                <Card
                  key={concept.title}
                  className="p-6 border-primary/10"
                  data-testid={`card-concept-${index}`}
                >
                  <concept.icon className="w-6 h-6 text-primary mb-3" />
                  <h3 className="font-semibold mb-2">{concept.title}</h3>
                  <p className="text-sm text-muted-foreground leading-relaxed">{concept.description}</p>
                </Card>
              ))}
            </div>
          </div>
        </AnimatedSection>

        <AnimatedSection delay={0.2}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-math-foundation">
            <div className="flex items-start gap-4 mb-4">
              <div className="text-primary">
                <Compass className="w-8 h-8" />
              </div>
              <div>
                <h2 className="text-2xl font-bold mb-2">Mathematical Foundation</h2>
              </div>
            </div>
            <div className="space-y-4">
              <div className="grid sm:grid-cols-3 gap-4">
                <Card className="p-4 border-primary/10 bg-secondary/30 text-center">
                  <div className="text-2xl font-bold text-primary mb-1">364&#176;</div>
                  <div className="text-xs text-muted-foreground">Full Circle = 111111&#8323;</div>
                </Card>
                <Card className="p-4 border-primary/10 bg-secondary/30 text-center">
                  <div className="text-2xl font-bold text-primary mb-1">13&#176;</div>
                  <div className="text-xs text-muted-foreground">1 Radian = 111&#8323; = T&#8327;</div>
                </Card>
                <Card className="p-4 border-primary/10 bg-secondary/30 text-center">
                  <div className="text-2xl font-bold text-primary mb-1">&#960; = 14</div>
                  <div className="text-xs text-muted-foreground">Ternary Pi (C / d)</div>
                </Card>
              </div>
              <p className="text-muted-foreground leading-relaxed text-sm">
                The ternary circle measures 364 degrees -- a six-digit base-3 repunit (111111&#8323;).
                One ternary radian equals 13 degrees (111&#8323;), which is also the seventh Tribonacci
                number T&#8327;. This links the recurrence to the geometry: 28 ternary radians complete
                the circle (364 = 13 &#215; 28), and the cyclic group Z&#8322;&#8328; governs the
                symmetry structure of the entire system.
              </p>
            </div>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.25}>
          <Card className="p-6 md:p-8 border-primary/10 mb-8" data-testid="section-symmetry-comparison">
            <h2 className="text-2xl font-bold mb-6">Symmetry Comparison</h2>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-primary/10">
                    <th className="text-left py-3 px-4 text-muted-foreground font-medium">Property</th>
                    <th className="text-left py-3 px-4 text-muted-foreground font-medium">3-Fold (Traditional)</th>
                    <th className="text-left py-3 px-4 text-primary font-medium">28-Fold (PlenumNET)</th>
                  </tr>
                </thead>
                <tbody>
                  <tr className="border-b border-primary/5">
                    <td className="py-3 px-4 text-muted-foreground">Rotation Steps</td>
                    <td className="py-3 px-4 text-muted-foreground">3</td>
                    <td className="py-3 px-4 font-medium">28</td>
                  </tr>
                  <tr className="border-b border-primary/5">
                    <td className="py-3 px-4 text-muted-foreground">Angular Resolution</td>
                    <td className="py-3 px-4 text-muted-foreground">120&#176;</td>
                    <td className="py-3 px-4 font-medium">13&#176; (111&#8323;)</td>
                  </tr>
                  <tr className="border-b border-primary/5">
                    <td className="py-3 px-4 text-muted-foreground">Generator Coverage</td>
                    <td className="py-3 px-4 text-muted-foreground">Partial</td>
                    <td className="py-3 px-4 font-medium">Complete (all 28 positions)</td>
                  </tr>
                  <tr className="border-b border-primary/5">
                    <td className="py-3 px-4 text-muted-foreground">Governing Group</td>
                    <td className="py-3 px-4 text-muted-foreground">Z&#8323;</td>
                    <td className="py-3 px-4 font-medium">Z&#8322;&#8328;</td>
                  </tr>
                  <tr>
                    <td className="py-3 px-4 text-muted-foreground">Sequence Foundation</td>
                    <td className="py-3 px-4 text-muted-foreground">Fibonacci</td>
                    <td className="py-3 px-4 font-medium">Tribonacci (&#964; &#8776; 1.839)</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </Card>
        </AnimatedSection>

        <AnimatedSection delay={0.3}>
          <Card className="p-6 md:p-8 border-primary/10 text-center" data-testid="section-cta">
            <h2 className="text-2xl font-bold mb-3">Explore the Framework</h2>
            <p className="text-muted-foreground mb-6 max-w-xl mx-auto">
              The 28-dimension symmetry is one of the foundational structures that makes PlenumNET's
              ternary computing platform possible. Discover the full architecture.
            </p>
            <div className="flex flex-wrap justify-center gap-3">
              <Button asChild data-testid="button-view-ternarydb">
                <Link href="/ternarydb">
                  PlenumDB Console
                  <ArrowRight className="w-4 h-4 ml-2" />
                </Link>
              </Button>
              <Button variant="outline" asChild data-testid="button-view-about">
                <Link href="/about">
                  About PlenumNET
                </Link>
              </Button>
            </div>
          </Card>
        </AnimatedSection>
      </div>
    </div>
  );
}
