/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { useState, useRef, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { ArrowLeft, Play, Loader2, CheckCircle2, XCircle, Circle, ChevronDown, ChevronUp, Settings2, RotateCcw } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Link } from "wouter";
import { useToast } from "@/hooks/use-toast";
import {
  AGENT_COUNT,
  STEPS_PER_AGENT,
  AGENT_STEP_NAMES,
  DEFAULT_SPECIALISTS,
  LAYER2_SECTIONS,
  getAgentPositions,
  type AgentSpecialist,
  type AgentCategory,
  type AgentStepEvent,
  type AgentResult,
  type AgentPosition,
  type Layer2Section,
} from "../../../shared/agent-array";

type AgentStatus = "idle" | "running" | "complete" | "error";

interface AgentState {
  status: AgentStatus;
  currentStep: number;
  result?: AgentResult;
}

const CATEGORY_COLORS: Record<AgentCategory, string> = {
  "International Law": "text-blue-500",
  "Regional Legal Systems": "text-violet-500",
  "Finance": "text-emerald-500",
  "Crypto": "text-amber-500",
  "Security": "text-red-500",
};

const CATEGORY_BG: Record<AgentCategory, string> = {
  "International Law": "bg-blue-500/10 border-blue-500/20",
  "Regional Legal Systems": "bg-violet-500/10 border-violet-500/20",
  "Finance": "bg-emerald-500/10 border-emerald-500/20",
  "Crypto": "bg-amber-500/10 border-amber-500/20",
  "Security": "bg-red-500/10 border-red-500/20",
};

function CircleVisualization({
  agentStates,
  roles,
}: {
  agentStates: Map<number, AgentState>;
  roles: AgentSpecialist[];
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
    if (!state || state.status === "idle") {
      return CATEGORY_COLORS[roles[z28]?.category || "International Law"].replace("text-", "fill-") + "/40";
    }
    switch (state.status) {
      case "running": return "fill-yellow-500";
      case "complete": return "fill-green-500";
      case "error": return "fill-destructive";
      default: return "fill-muted-foreground/40";
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
      className="p-3 relative overflow-visible"
      data-testid={`card-agent-${position.z28}`}
    >
      <div className="flex items-center gap-2 mb-1.5 flex-wrap">
        {statusIcon()}
        <span className="font-mono text-xs font-semibold">{position.label}</span>
        <Badge variant="outline" className={`text-[10px] px-1.5 py-0 ${CATEGORY_COLORS[position.category]}`}>
          {position.category.split(" ")[0]}
        </Badge>
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

function RoleCustomizer({
  roles,
  onChange,
  onReset,
}: {
  roles: AgentSpecialist[];
  onChange: (index: number, field: "title" | "description", value: string) => void;
  onReset: () => void;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const [expandedCategory, setExpandedCategory] = useState<AgentCategory | null>(null);

  return (
    <Card className="border-primary/10" data-testid="section-role-customizer">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between gap-2 p-4 text-left hover-elevate rounded-md"
        data-testid="button-toggle-customizer"
      >
        <div className="flex items-center gap-2 flex-wrap">
          <Settings2 className="w-4 h-4 text-primary" />
          <span className="font-semibold text-sm">Customize Agent Roles</span>
          <Badge variant="outline" className="text-[10px]">28 Specialists</Badge>
        </div>
        {isOpen ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
      </button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-4">
              <div className="flex items-center justify-between mb-3 flex-wrap gap-2">
                <p className="text-xs text-muted-foreground">
                  Edit agent titles and descriptions below. Changes apply to your next query.
                </p>
                <Button variant="ghost" size="sm" onClick={onReset} data-testid="button-reset-roles">
                  <RotateCcw className="w-3 h-3 mr-1" />
                  Reset to Default
                </Button>
              </div>

              <div className="space-y-2">
                {LAYER2_SECTIONS.map((section) => (
                  <div key={section.key} className={`rounded-md border ${CATEGORY_BG[section.key]}`}>
                    <button
                      onClick={() => setExpandedCategory(expandedCategory === section.key ? null : section.key)}
                      className="w-full flex items-center justify-between gap-2 p-3 text-left"
                      data-testid={`button-expand-${section.key.replace(/\s/g, "-").toLowerCase()}`}
                    >
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className={`text-sm font-semibold ${CATEGORY_COLORS[section.key]}`}>
                          {section.label}
                        </span>
                        <Badge variant="outline" className="text-[10px]">
                          {section.agentIndices.length} agents
                        </Badge>
                      </div>
                      {expandedCategory === section.key ? (
                        <ChevronUp className="w-3 h-3" />
                      ) : (
                        <ChevronDown className="w-3 h-3" />
                      )}
                    </button>

                    <AnimatePresence>
                      {expandedCategory === section.key && (
                        <motion.div
                          initial={{ height: 0, opacity: 0 }}
                          animate={{ height: "auto", opacity: 1 }}
                          exit={{ height: 0, opacity: 0 }}
                          transition={{ duration: 0.15 }}
                          className="overflow-hidden"
                        >
                          <div className="px-3 pb-3 space-y-2">
                            {section.agentIndices.map((agentIdx) => (
                              <div key={agentIdx} className="grid gap-1.5">
                                <div className="flex items-center gap-2 flex-wrap">
                                  <span className="text-[10px] font-mono text-muted-foreground">
                                    A{String(agentIdx).padStart(2, "0")}
                                  </span>
                                  <input
                                    type="text"
                                    value={roles[agentIdx].title}
                                    onChange={(e) => onChange(agentIdx, "title", e.target.value)}
                                    className="flex-1 text-xs rounded-md border border-input bg-background px-2 py-1 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                    maxLength={200}
                                    data-testid={`input-role-title-${agentIdx}`}
                                  />
                                </div>
                                <textarea
                                  value={roles[agentIdx].description}
                                  onChange={(e) => onChange(agentIdx, "description", e.target.value)}
                                  className="w-full text-[11px] rounded-md border border-input bg-background px-2 py-1 resize-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring text-muted-foreground"
                                  rows={2}
                                  maxLength={500}
                                  data-testid={`input-role-desc-${agentIdx}`}
                                />
                              </div>
                            ))}
                          </div>
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>
                ))}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </Card>
  );
}

function Layer2Display({ sections }: { sections: Layer2Section[] }) {
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null);

  return (
    <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} data-testid="section-layer2">
      <Card className="p-5 border-primary/10">
        <div className="flex items-center gap-3 mb-4 flex-wrap">
          <h2 className="text-lg font-bold">Dimensional Layer 2 — Executive Summary</h2>
          <Badge variant="outline">{sections.length} Sections</Badge>
        </div>
        <p className="text-sm text-muted-foreground mb-4">
          Each section provides both a technical analysis for experts and a plain-language summary for easy understanding.
        </p>
        <div className="space-y-2">
          {sections.map((section, idx) => {
            const isExpanded = expandedIdx === idx;
            return (
              <div
                key={section.category}
                className={`rounded-md border ${CATEGORY_BG[section.category]}`}
                data-testid={`layer2-section-${section.category.replace(/\s/g, "-").toLowerCase()}`}
              >
                <button
                  onClick={() => setExpandedIdx(isExpanded ? null : idx)}
                  className="w-full flex items-center justify-between gap-2 p-4 text-left"
                  data-testid={`button-toggle-layer2-${section.category.replace(/\s/g, "-").toLowerCase()}`}
                >
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className={`font-semibold ${CATEGORY_COLORS[section.category]}`}>
                      {section.label}
                    </span>
                    <Badge variant="outline" className="text-[10px]">
                      {section.successCount}/{section.agentCount} agents
                    </Badge>
                  </div>
                  {isExpanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                </button>

                <AnimatePresence>
                  {isExpanded && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.15 }}
                      className="overflow-hidden"
                    >
                      <div className="px-4 pb-4 space-y-4">
                        <div>
                          <div className="flex items-center gap-2 mb-2 flex-wrap">
                            <Badge variant="outline" className="text-[10px]">Technical</Badge>
                          </div>
                          <p className="text-sm text-foreground/90 leading-relaxed" data-testid={`text-technical-${idx}`}>
                            {section.technicalSummary}
                          </p>
                        </div>
                        <div className="border-t border-border/50 pt-3">
                          <div className="flex items-center gap-2 mb-2 flex-wrap">
                            <Badge variant="outline" className="text-[10px]">Plain Language</Badge>
                          </div>
                          <p className="text-sm text-foreground/90 leading-relaxed" data-testid={`text-plain-${idx}`}>
                            {section.laySummary}
                          </p>
                        </div>
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            );
          })}
        </div>
      </Card>
    </motion.div>
  );
}

export default function AgentArrayPage() {
  const [prompt, setPrompt] = useState("");
  const [isRunning, setIsRunning] = useState(false);
  const [agentStates, setAgentStates] = useState<Map<number, AgentState>>(new Map());
  const [consensus, setConsensus] = useState<string | null>(null);
  const [layer2Sections, setLayer2Sections] = useState<Layer2Section[]>([]);
  const [layer2Loading, setLayer2Loading] = useState(false);
  const [totalDuration, setTotalDuration] = useState<number | null>(null);
  const [successCount, setSuccessCount] = useState(0);
  const [customRoles, setCustomRoles] = useState<AgentSpecialist[]>(
    DEFAULT_SPECIALISTS.map((s) => ({ ...s }))
  );
  const { toast } = useToast();

  const positions = getAgentPositions(customRoles);

  const handleRoleChange = useCallback((index: number, field: "title" | "description", value: string) => {
    setCustomRoles((prev) => {
      const next = [...prev];
      next[index] = { ...next[index], [field]: value };
      return next;
    });
  }, []);

  const handleResetRoles = useCallback(() => {
    setCustomRoles(DEFAULT_SPECIALISTS.map((s) => ({ ...s })));
    toast({ title: "Roles Reset", description: "All agent roles restored to defaults." });
  }, [toast]);

  const isCustomized = customRoles.some((r, i) =>
    r.title !== DEFAULT_SPECIALISTS[i].title || r.description !== DEFAULT_SPECIALISTS[i].description
  );

  const eventSourceRef = useRef<EventSource | null>(null);

  const launchAgentArray = useCallback(async () => {
    if (!prompt.trim() || isRunning) return;

    setIsRunning(true);
    setConsensus(null);
    setLayer2Sections([]);
    setLayer2Loading(false);
    setTotalDuration(null);
    setSuccessCount(0);
    setAgentStates(new Map());

    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    const body: Record<string, unknown> = { prompt: prompt.trim() };
    if (isCustomized) {
      body.customRoles = customRoles;
    }

    try {
      const createRes = await fetch("/api/tribonacci/agent-array", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });

      if (!createRes.ok) {
        const err = await createRes.json();
        throw new Error(err.error || "Request failed");
      }

      const { sessionId } = await createRes.json();

      const es = new EventSource(`/api/tribonacci/agent-array/stream/${sessionId}`);
      eventSourceRef.current = es;

      es.addEventListener("agent_step", (e) => {
        try {
          const step = JSON.parse(e.data) as AgentStepEvent;
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
        } catch {}
      });

      es.addEventListener("layer2_start", () => {
        setLayer2Loading(true);
      });

      es.addEventListener("layer2_complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          if (data.sections) {
            setLayer2Sections(data.sections);
          }
          setLayer2Loading(false);
        } catch {}
      });

      es.addEventListener("complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          const results = data.results as AgentResult[];
          const finalStates = new Map<number, AgentState>();
          for (const r of results) {
            finalStates.set(r.z28, {
              status: r.response.startsWith("[Error]") ? "error" : "complete",
              currentStep: STEPS_PER_AGENT,
              result: r,
            });
          }
          setAgentStates(finalStates);
          setConsensus(data.consensus || null);
          setTotalDuration(data.totalDurationMs || null);
          setSuccessCount(results.filter((r: AgentResult) => !r.response.startsWith("[Error]")).length);
          if (data.layer2) {
            setLayer2Sections(data.layer2);
          }
          setLayer2Loading(false);
        } catch {}
        es.close();
        eventSourceRef.current = null;
        setIsRunning(false);
      });

      es.onerror = () => {
        es.close();
        eventSourceRef.current = null;
        setIsRunning(false);
      };
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Unknown error";
      toast({ title: "Agent Array Error", description: msg, variant: "destructive" });
      setIsRunning(false);
    }
  }, [prompt, isRunning, toast, customRoles, isCustomized]);

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
            Launch 28 specialist AI agents simultaneously, each mapped to a Z&#8322;&#8328; position on the Tribonacci Circle.
            Layer 1 delivers individual expert analyses. Layer 2 synthesizes a 5-section executive summary
            with both technical detail and plain-language explanations.
          </p>
        </motion.div>

        <div className="space-y-4 mb-6">
          <RoleCustomizer
            roles={customRoles}
            onChange={handleRoleChange}
            onReset={handleResetRoles}
          />

          <Card className="p-5 border-primary/10" data-testid="section-launch-panel">
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
              <div className="flex flex-col gap-2 sm:w-44">
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
                <div className="text-xs text-muted-foreground text-center space-y-0.5">
                  <div>{AGENT_COUNT} agents &middot; {STEPS_PER_AGENT} steps each</div>
                  {isCustomized && (
                    <div className="text-primary font-medium">Custom roles active</div>
                  )}
                </div>
              </div>
            </div>
          </Card>
        </div>

        {(isRunning || agentStates.size > 0) && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.3 }} className="space-y-6">
            <div className="grid lg:grid-cols-[1fr_auto] gap-6">
              <Card className="p-5 border-primary/10" data-testid="section-agent-status">
                <div className="flex items-center gap-3 mb-4 flex-wrap">
                  <h2 className="text-lg font-bold">Layer 1 — Agent Status</h2>
                  {isRunning && !layer2Loading && (
                    <Badge variant="outline" className="text-yellow-600 border-yellow-500/30 bg-yellow-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Processing
                    </Badge>
                  )}
                  {layer2Loading && (
                    <Badge variant="outline" className="text-blue-600 border-blue-500/30 bg-blue-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Generating Layer 2...
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

              <CircleVisualization agentStates={agentStates} roles={customRoles} />
            </div>

            {layer2Sections.length > 0 && (
              <Layer2Display sections={layer2Sections} />
            )}

            {consensus && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }}>
                <Card className="p-5 border-primary/10" data-testid="section-consensus">
                  <h2 className="text-lg font-bold mb-3">Final Consensus</h2>
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
                Enter a prompt above and launch the agent array. All 28 specialist agents will process your query
                simultaneously. Layer 2 generates a 5-section executive summary with both expert and plain-language analysis.
              </p>
              <div className="flex flex-wrap justify-center gap-2">
                {LAYER2_SECTIONS.map((s) => (
                  <Badge key={s.key} variant="outline" className={CATEGORY_COLORS[s.key]}>
                    {s.label} ({s.agentIndices.length})
                  </Badge>
                ))}
              </div>
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}
