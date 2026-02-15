/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { useState, useRef, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { ArrowLeft, Play, Loader2, CheckCircle2, XCircle, Circle } from "lucide-react";
import { motion } from "framer-motion";
import { Link } from "wouter";
import { useToast } from "@/hooks/use-toast";
import {
  AGENT_COUNT,
  STEPS_PER_AGENT,
  AGENT_STEP_NAMES,
  getAgentPositions,
  type AgentStepEvent,
  type AgentResult,
  type AgentPosition,
} from "../../../shared/agent-array";

type AgentStatus = "idle" | "running" | "complete" | "error";

interface AgentState {
  status: AgentStatus;
  currentStep: number;
  result?: AgentResult;
}

function CircleVisualization({
  agents,
  agentStates,
}: {
  agents: AgentPosition[];
  agentStates: Map<number, AgentState>;
}) {
  const radius = 130;
  const cx = 160;
  const cy = 160;

  const dots = Array.from({ length: AGENT_COUNT }, (_, i) => {
    const angle = (2 * Math.PI * i) / AGENT_COUNT - Math.PI / 2;
    return {
      x: cx + radius * Math.cos(angle),
      y: cy + radius * Math.sin(angle),
      z28: i,
    };
  });

  const getColor = (z28: number) => {
    const state = agentStates.get(z28);
    if (!state) return "text-muted-foreground/40";
    switch (state.status) {
      case "running": return "text-yellow-500";
      case "complete": return "text-green-500";
      case "error": return "text-destructive";
      default: return "text-muted-foreground/40";
    }
  };

  const getRadius = (z28: number) => {
    const state = agentStates.get(z28);
    if (state?.status === "running") return 6;
    if (state?.status === "complete") return 5;
    return 4;
  };

  return (
    <div className="flex justify-center" data-testid="diagram-agent-array">
      <svg viewBox="0 0 320 320" className="w-56 h-56 md:w-72 md:h-72">
        <circle
          cx={cx} cy={cy} r={radius}
          fill="none" stroke="currentColor" strokeWidth="1"
          className="text-muted-foreground/20"
        />
        {dots.map((dot) => (
          <g key={dot.z28}>
            <circle
              cx={dot.x} cy={dot.y}
              r={getRadius(dot.z28)}
              fill="currentColor"
              className={`${getColor(dot.z28)} transition-all duration-300`}
            />
            {dot.z28 % 7 === 0 && (
              <text
                x={dot.x + (dot.x > cx ? 10 : -10)}
                y={dot.y + (dot.y > cy ? 12 : -6)}
                textAnchor={dot.x > cx ? "start" : "end"}
                className="text-muted-foreground fill-current"
                fontSize="9"
              >
                {dot.z28}
              </text>
            )}
          </g>
        ))}
        <text
          x={cx} y={cy + 4}
          textAnchor="middle"
          className="text-foreground fill-current font-semibold"
          fontSize="12"
        >
          Z&#8322;&#8328;
        </text>
      </svg>
    </div>
  );
}

function AgentCard({ position, state }: { position: AgentPosition; state?: AgentState }) {
  const statusIcon = () => {
    if (!state || state.status === "idle") return <Circle className="w-3 h-3 text-muted-foreground/40" />;
    if (state.status === "running") return <Loader2 className="w-3 h-3 text-yellow-500 animate-spin" />;
    if (state.status === "complete") return <CheckCircle2 className="w-3 h-3 text-green-500" />;
    return <XCircle className="w-3 h-3 text-destructive" />;
  };

  const progress = state ? Math.round((state.currentStep / STEPS_PER_AGENT) * 100) : 0;

  return (
    <Card
      className="p-3 border-primary/10 relative overflow-visible"
      data-testid={`card-agent-${position.z28}`}
    >
      <div className="flex items-center gap-2 mb-1.5 flex-wrap">
        {statusIcon()}
        <span className="font-mono text-xs font-semibold">{position.label}</span>
        <Badge variant="outline" className="text-[10px] px-1.5 py-0">{position.z28 * 13}°</Badge>
      </div>
      <p className="text-[11px] text-muted-foreground leading-tight mb-1.5 line-clamp-1">
        {position.domain}
      </p>
      {state && state.status !== "idle" && (
        <>
          <div className="w-full bg-secondary/50 rounded-full h-1 mb-1">
            <div
              className={`h-1 rounded-full transition-all duration-300 ${
                state.status === "error" ? "bg-destructive" : state.status === "complete" ? "bg-green-500" : "bg-yellow-500"
              }`}
              style={{ width: `${progress}%` }}
            />
          </div>
          <p className="text-[10px] text-muted-foreground">
            {state.status === "complete"
              ? `${state.result?.totalDurationMs || 0}ms`
              : state.currentStep < STEPS_PER_AGENT
                ? AGENT_STEP_NAMES[state.currentStep]
                : "Finalizing"}
          </p>
        </>
      )}
      {state?.status === "complete" && state.result && !state.result.response.startsWith("[Error]") && (
        <p className="text-[11px] text-foreground/80 mt-1.5 line-clamp-3 leading-snug">
          {state.result.response}
        </p>
      )}
    </Card>
  );
}

export default function AgentArrayPage() {
  const [prompt, setPrompt] = useState("");
  const [isRunning, setIsRunning] = useState(false);
  const [agentStates, setAgentStates] = useState<Map<number, AgentState>>(new Map());
  const [consensus, setConsensus] = useState<string | null>(null);
  const [totalDuration, setTotalDuration] = useState<number | null>(null);
  const [successCount, setSuccessCount] = useState(0);
  const abortRef = useRef<AbortController | null>(null);
  const { toast } = useToast();

  const positions = getAgentPositions();

  const launchAgentArray = useCallback(async () => {
    if (!prompt.trim() || isRunning) return;

    setIsRunning(true);
    setConsensus(null);
    setTotalDuration(null);
    setSuccessCount(0);
    setAgentStates(new Map());

    abortRef.current = new AbortController();

    try {
      const response = await fetch("/api/tribonacci/agent-array", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ prompt: prompt.trim() }),
        signal: abortRef.current.signal,
      });

      if (!response.ok) {
        const err = await response.json();
        throw new Error(err.error || "Request failed");
      }

      const reader = response.body?.getReader();
      if (!reader) throw new Error("No response stream");

      const decoder = new TextDecoder();
      let buffer = "";

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";

        for (const line of lines) {
          if (!line.startsWith("data: ")) continue;
          try {
            const event = JSON.parse(line.slice(6));

            if (event.type === "agent_step") {
              const step = event as AgentStepEvent;
              setAgentStates((prev) => {
                const next = new Map(prev);
                const current = next.get(step.z28) || { status: "idle" as AgentStatus, currentStep: 0 };
                next.set(step.z28, {
                  status: step.status === "complete" && step.stepIndex === STEPS_PER_AGENT - 1
                    ? "complete"
                    : step.status === "error"
                      ? "error"
                      : "running",
                  currentStep: step.stepIndex + (step.status === "complete" ? 1 : 0),
                  result: current.result,
                });
                return next;
              });
            }

            if (event.type === "complete") {
              const results = event.results as AgentResult[];
              const finalStates = new Map<number, AgentState>();
              for (const r of results) {
                finalStates.set(r.z28, {
                  status: r.response.startsWith("[Error]") ? "error" : "complete",
                  currentStep: STEPS_PER_AGENT,
                  result: r,
                });
              }
              setAgentStates(finalStates);
              setConsensus(event.consensus || null);
              setTotalDuration(event.totalDurationMs || null);
              setSuccessCount(results.filter((r: AgentResult) => !r.response.startsWith("[Error]")).length);
            }
          } catch {}
        }
      }
    } catch (err: unknown) {
      if (err instanceof Error && err.name === "AbortError") return;
      const msg = err instanceof Error ? err.message : "Unknown error";
      toast({ title: "Agent Array Error", description: msg, variant: "destructive" });
    } finally {
      setIsRunning(false);
    }
  }, [prompt, isRunning, toast]);

  return (
    <div className="min-h-screen bg-background" data-testid="page-agent-array">
      <div className="max-w-7xl mx-auto px-5 py-8">
        <div className="mb-6">
          <Button variant="ghost" size="sm" asChild data-testid="link-back-home" aria-label="Back to home page">
            <Link href="/">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Home
            </Link>
          </Button>
        </div>

        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5 }} className="mb-8">
          <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
            AI Orchestration
          </Badge>
          <h1 className="text-3xl md:text-4xl font-bold mb-3" data-testid="text-agent-array-title">
            28-Dimension Agent Array
          </h1>
          <p className="text-muted-foreground max-w-2xl" data-testid="text-agent-array-subtitle">
            Launch 28 AI agents simultaneously, each mapped to a Z&#8322;&#8328; position on the Tribonacci Circle.
            Every agent follows a 13-step execution model and contributes its domain expertise to a unified consensus.
          </p>
        </motion.div>

        <Card className="p-5 border-primary/10 mb-6" data-testid="section-launch-panel">
          <div className="flex flex-col sm:flex-row gap-3">
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Enter your query for the 28-agent array..."
              className="flex-1 min-h-[80px] rounded-md border border-input bg-background px-3 py-2 text-sm resize-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              maxLength={2000}
              disabled={isRunning}
              data-testid="input-prompt"
            />
            <div className="flex flex-col gap-2 sm:w-40">
              <Button
                onClick={launchAgentArray}
                disabled={!prompt.trim() || isRunning}
                data-testid="button-launch"
              >
                {isRunning ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Running...
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4 mr-2" />
                    Launch Array
                  </>
                )}
              </Button>
              <div className="text-xs text-muted-foreground text-center">
                {AGENT_COUNT} agents &middot; {STEPS_PER_AGENT} steps each
              </div>
            </div>
          </div>
        </Card>

        {(isRunning || agentStates.size > 0) && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.3 }}>
            <div className="grid lg:grid-cols-[1fr_auto] gap-6 mb-6">
              <Card className="p-5 border-primary/10" data-testid="section-agent-status">
                <div className="flex items-center gap-3 mb-4 flex-wrap">
                  <h2 className="text-lg font-bold">Agent Status</h2>
                  {isRunning && (
                    <Badge variant="outline" className="text-yellow-600 border-yellow-500/30 bg-yellow-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Processing
                    </Badge>
                  )}
                  {!isRunning && totalDuration !== null && (
                    <Badge variant="outline" className="text-green-600 border-green-500/30 bg-green-500/10">
                      <CheckCircle2 className="w-3 h-3 mr-1" />
                      {successCount}/{AGENT_COUNT} Complete &middot; {(totalDuration / 1000).toFixed(1)}s
                    </Badge>
                  )}
                </div>
                <div className="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-2">
                  {positions.map((pos) => (
                    <AgentCard key={pos.z28} position={pos} state={agentStates.get(pos.z28)} />
                  ))}
                </div>
              </Card>

              <CircleVisualization agents={positions} agentStates={agentStates} />
            </div>

            {consensus && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }}>
                <Card className="p-5 border-primary/10" data-testid="section-consensus">
                  <h2 className="text-lg font-bold mb-3">Consensus Synthesis</h2>
                  <p className="text-sm text-foreground/90 leading-relaxed">{consensus}</p>
                </Card>
              </motion.div>
            )}
          </motion.div>
        )}

        {!isRunning && agentStates.size === 0 && (
          <Card className="p-8 border-primary/10 text-center" data-testid="section-empty-state">
            <div className="max-w-md mx-auto">
              <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-4">
                <Play className="w-6 h-6 text-primary" />
              </div>
              <h3 className="font-semibold mb-2">Ready to Launch</h3>
              <p className="text-sm text-muted-foreground mb-4">
                Enter a prompt above and launch the agent array. All 28 agents will process your query
                simultaneously from their unique domain perspectives on the Tribonacci Circle.
              </p>
              <div className="flex flex-wrap justify-center gap-2">
                <Badge variant="outline">{AGENT_COUNT} Agents</Badge>
                <Badge variant="outline">{STEPS_PER_AGENT} Steps/Agent</Badge>
                <Badge variant="outline">364 Steps Total</Badge>
                <Badge variant="outline">Z&#8322;&#8328; Geometry</Badge>
              </div>
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}
