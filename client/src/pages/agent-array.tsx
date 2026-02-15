/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { useState, useRef, useCallback, useMemo } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { ArrowLeft, Play, Loader2, CheckCircle2, XCircle, Circle, ChevronDown, ChevronUp, Settings2, RotateCcw, Shield, MapPin, AlertTriangle, ListOrdered, FileText, Copy, Check, Eye, Filter, Globe } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Link } from "wouter";
import { useToast } from "@/hooks/use-toast";
import {
  AGENT_COUNT,
  STEPS_PER_AGENT,
  AGENT_STEP_NAMES,
  DEFAULT_SPECIALISTS,
  LAYER2_SECTIONS,
  AGENT_LANGUAGES,
  getAgentPositions,
  type AgentSpecialist,
  type AgentCategory,
  type AgentStepEvent,
  type AgentResult,
  type AgentPosition,
  type AgentLanguage,
  type Layer2Section,
  type ExecutiveSummary,
  type VerdictSignal,
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

const VERDICT_COLORS: Record<VerdictSignal, { bg: string; text: string; border: string }> = {
  GREEN: { bg: "bg-green-500/10", text: "text-green-600 dark:text-green-400", border: "border-green-500/30" },
  YELLOW: { bg: "bg-yellow-500/10", text: "text-yellow-600 dark:text-yellow-400", border: "border-yellow-500/30" },
  RED: { bg: "bg-red-500/10", text: "text-red-600 dark:text-red-400", border: "border-red-500/30" },
};

const STATUS_COLORS: Record<string, string> = {
  permitted: "text-green-600 dark:text-green-400",
  conditional: "text-yellow-600 dark:text-yellow-400",
  restricted: "text-orange-600 dark:text-orange-400",
  prohibited: "text-red-600 dark:text-red-400",
  unclear: "text-muted-foreground",
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
          <h2 className="text-lg font-bold">Dimensional Layer 2 — Category Analysis</h2>
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

function RiskBar({ score, label }: { score: number; label: string }) {
  const pct = Math.min(100, Math.max(0, score * 10));
  const color = score <= 3 ? "bg-green-500" : score <= 6 ? "bg-yellow-500" : "bg-red-500";
  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between gap-2 flex-wrap">
        <span className="text-xs text-muted-foreground">{label}</span>
        <span className="text-xs font-semibold">{score.toFixed(1)}/10</span>
      </div>
      <div className="w-full bg-secondary/50 rounded-full h-2">
        <div className={`h-2 rounded-full transition-all duration-500 ${color}`} style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

function ExecutiveSummaryDisplay({ summary }: { summary: ExecutiveSummary }) {
  const [openSection, setOpenSection] = useState<number | null>(0);

  const safeVerdict = summary.verdict || { signal: "YELLOW" as VerdictSignal, assessment: "Analysis pending.", confidence: 0.5 };
  const safeCompass = summary.jurisdictionalCompass || [];
  const safeRisk = summary.riskBarometer || { financial: [], technical: [], aggregateFinancial: 5, aggregateTechnical: 5 };
  const safePath = summary.criticalPath || [];
  const safePlain = summary.plainEnglish || { summary: "See detailed sections above.", boardRecommendation: "Review findings." };

  const vc = VERDICT_COLORS[safeVerdict.signal] || VERDICT_COLORS.YELLOW;

  const sections = [
    {
      title: "The Verdict",
      icon: <Shield className="w-4 h-4" />,
      content: (
        <div className="space-y-3">
          <div className={`inline-flex items-center gap-2 rounded-md border px-3 py-1.5 ${vc.bg} ${vc.border}`}>
            <span className={`text-lg font-bold ${vc.text}`} data-testid="text-verdict-signal">
              {safeVerdict.signal}
            </span>
            <span className="text-xs text-muted-foreground">
              Confidence: {((safeVerdict.confidence || 0.5) * 100).toFixed(0)}%
            </span>
          </div>
          <p className="text-sm text-foreground/90 leading-relaxed" data-testid="text-verdict-assessment">
            {safeVerdict.assessment || "Assessment pending."}
          </p>
        </div>
      ),
    },
    {
      title: "Jurisdictional Compass",
      icon: <MapPin className="w-4 h-4" />,
      content: (
        <div className="space-y-2">
          {safeCompass.length > 0 ? safeCompass.map((region, i) => (
            <div key={i} className="flex items-start gap-3 py-1.5 border-b border-border/30 last:border-0" data-testid={`region-${i}`}>
              <span className="text-sm font-medium min-w-[120px]">{region.region}</span>
              <Badge variant="outline" className={`text-[10px] ${STATUS_COLORS[region.status] || ""}`}>
                {(region.status || "unclear").toUpperCase()}
              </Badge>
              {region.notes && (
                <span className="text-xs text-muted-foreground flex-1">{region.notes}</span>
              )}
            </div>
          )) : (
            <p className="text-xs text-muted-foreground">No jurisdictional data available.</p>
          )}
        </div>
      ),
    },
    {
      title: "Risk Barometer",
      icon: <AlertTriangle className="w-4 h-4" />,
      content: (
        <div className="space-y-4">
          <div className="grid sm:grid-cols-2 gap-4">
            <div className="space-y-2">
              <h4 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Financial Risk</h4>
              <RiskBar score={safeRisk.aggregateFinancial ?? 5} label="Aggregate" />
              {(safeRisk.financial || []).map((r, i) => (
                <p key={i} className="text-xs text-foreground/80">{r.narrative}</p>
              ))}
            </div>
            <div className="space-y-2">
              <h4 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Technical Risk</h4>
              <RiskBar score={safeRisk.aggregateTechnical ?? 5} label="Aggregate" />
              {(safeRisk.technical || []).map((r, i) => (
                <p key={i} className="text-xs text-foreground/80">{r.narrative}</p>
              ))}
            </div>
          </div>
        </div>
      ),
    },
    {
      title: "Critical Path",
      icon: <ListOrdered className="w-4 h-4" />,
      content: (
        <div className="space-y-2">
          {safePath.length > 0 ? safePath.map((step) => (
            <div key={step.order} className="flex items-start gap-3 py-1.5" data-testid={`critical-step-${step.order}`}>
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-xs font-bold shrink-0">
                {step.order}
              </span>
              <div className="flex-1">
                <p className="text-sm text-foreground/90">{step.action}</p>
                <Badge variant="outline" className="text-[10px] mt-1">{(step.category || "general").replace(/_/g, " ")}</Badge>
              </div>
            </div>
          )) : (
            <p className="text-xs text-muted-foreground">No critical path steps available.</p>
          )}
        </div>
      ),
    },
    {
      title: "Plain English Summary",
      icon: <FileText className="w-4 h-4" />,
      content: (
        <div className="space-y-3">
          <p className="text-sm text-foreground/90 leading-relaxed" data-testid="text-plain-english-summary">
            {safePlain.summary || "See detailed sections above."}
          </p>
          <div className="border-t border-border/50 pt-3">
            <h4 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-1">Board Recommendation</h4>
            <p className="text-sm text-foreground font-medium" data-testid="text-board-recommendation">
              {safePlain.boardRecommendation || "Review detailed findings."}
            </p>
          </div>
        </div>
      ),
    },
  ];

  return (
    <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} data-testid="section-executive-summary">
      <Card className="p-5 border-primary/10">
        <div className="flex items-center gap-3 mb-4 flex-wrap">
          <h2 className="text-lg font-bold">Executive Summary</h2>
          <Badge variant="outline" className={`${vc.text} ${vc.border} ${vc.bg}`}>
            {safeVerdict.signal}
          </Badge>
          <Badge variant="outline">5 Sections</Badge>
        </div>
        <div className="space-y-2">
          {sections.map((sec, idx) => {
            const isOpen = openSection === idx;
            return (
              <div
                key={sec.title}
                className="rounded-md border border-border/50"
                data-testid={`executive-section-${idx}`}
              >
                <button
                  onClick={() => setOpenSection(isOpen ? null : idx)}
                  className="w-full flex items-center justify-between gap-2 p-4 text-left"
                  data-testid={`button-toggle-executive-${idx}`}
                >
                  <div className="flex items-center gap-2 flex-wrap">
                    {sec.icon}
                    <span className="font-semibold text-sm">{sec.title}</span>
                  </div>
                  {isOpen ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                </button>
                <AnimatePresence>
                  {isOpen && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.15 }}
                      className="overflow-hidden"
                    >
                      <div className="px-4 pb-4">
                        {sec.content}
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

function ResponseViewer({ results }: { results: AgentResult[] }) {
  const [selectedAgent, setSelectedAgent] = useState<number | null>(null);
  const [languageFilter, setLanguageFilter] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<number | null>(null);
  const [copiedAll, setCopiedAll] = useState(false);
  const { toast } = useToast();

  const handleFilterChange = useCallback((code: string | null) => {
    setLanguageFilter(code);
    setSelectedAgent(null);
  }, []);

  const successResults = useMemo(
    () => results.filter((r) => !r.response.startsWith("[Error]")),
    [results],
  );

  const filteredResults = useMemo(() => {
    if (!languageFilter) return successResults;
    return successResults.filter((r) => r.language?.code === languageFilter);
  }, [successResults, languageFilter]);

  const activeLanguages = useMemo(() => {
    const codes = new Set(successResults.map((r) => r.language?.code).filter(Boolean));
    return AGENT_LANGUAGES.filter((l) => codes.has(l.code));
  }, [successResults]);

  const copyToClipboard = useCallback(async (text: string, agentIdx?: number) => {
    try {
      await navigator.clipboard.writeText(text);
      if (agentIdx !== undefined) {
        setCopiedId(agentIdx);
        setTimeout(() => setCopiedId(null), 2000);
      } else {
        setCopiedAll(true);
        setTimeout(() => setCopiedAll(false), 2000);
      }
      toast({ title: "Copied", description: "Response copied to clipboard." });
    } catch {
      toast({ title: "Copy failed", description: "Could not copy to clipboard.", variant: "destructive" });
    }
  }, [toast]);

  const copyAllFiltered = useCallback(() => {
    const text = filteredResults
      .map((r) => `--- ${r.domain} [${r.language?.name || "Unknown"}] (A${String(r.z28).padStart(2, "0")}) ---\n${r.response}`)
      .join("\n\n");
    copyToClipboard(text);
  }, [filteredResults, copyToClipboard]);

  const selectedResult = selectedAgent !== null
    ? filteredResults.find((r) => r.z28 === selectedAgent) || null
    : null;

  return (
    <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} data-testid="section-response-viewer">
      <Card className="p-5 border-primary/10">
        <div className="flex items-center gap-3 mb-4 flex-wrap">
          <Eye className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-bold">Response Viewer</h2>
          <Badge variant="outline">{successResults.length} Responses</Badge>
          <Badge variant="outline">
            <Globe className="w-3 h-3 mr-1" />
            {activeLanguages.length} Languages
          </Badge>
          <div className="ml-auto">
            <Button
              variant="outline"
              size="sm"
              onClick={copyAllFiltered}
              disabled={filteredResults.length === 0}
              data-testid="button-copy-all"
            >
              {copiedAll ? <Check className="w-3.5 h-3.5 mr-1.5" /> : <Copy className="w-3.5 h-3.5 mr-1.5" />}
              {copiedAll ? "Copied!" : `Copy All${languageFilter ? " Filtered" : ""}`}
            </Button>
          </div>
        </div>

        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2 flex-wrap">
            <Filter className="w-3.5 h-3.5 text-muted-foreground" />
            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Filter by Language</span>
          </div>
          <div className="flex flex-wrap gap-1.5">
            <Button
              variant={languageFilter === null ? "default" : "outline"}
              size="sm"
              onClick={() => handleFilterChange(null)}
              data-testid="button-filter-all"
            >
              All ({successResults.length})
            </Button>
            {activeLanguages.map((lang) => {
              const count = successResults.filter((r) => r.language?.code === lang.code).length;
              return (
                <Button
                  key={lang.code}
                  variant={languageFilter === lang.code ? "default" : "outline"}
                  size="sm"
                  onClick={() => handleFilterChange(languageFilter === lang.code ? null : lang.code)}
                  data-testid={`button-filter-${lang.code}`}
                >
                  {lang.nativeName} ({count})
                </Button>
              );
            })}
          </div>
        </div>

        <div className="grid lg:grid-cols-[280px_1fr] gap-4">
          <div className="space-y-1 max-h-[600px] overflow-y-auto pr-1">
            {filteredResults.map((r) => (
              <button
                key={r.z28}
                onClick={() => setSelectedAgent(selectedAgent === r.z28 ? null : r.z28)}
                className={`w-full text-left rounded-md px-3 py-2 transition-colors ${
                  selectedAgent === r.z28
                    ? "bg-primary/10 border border-primary/30"
                    : "border border-transparent hover-elevate"
                }`}
                data-testid={`button-select-agent-${r.z28}`}
              >
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="font-mono text-[10px] text-muted-foreground">{r.agentLabel}</span>
                  <span className="text-xs font-medium truncate flex-1">{r.domain}</span>
                </div>
                <div className="flex items-center gap-2 mt-0.5 flex-wrap">
                  <Badge variant="outline" className={`text-[9px] px-1 py-0 ${CATEGORY_COLORS[r.category]}`}>
                    {r.category.split(" ")[0]}
                  </Badge>
                  <Badge variant="outline" className="text-[9px] px-1 py-0">
                    {r.language?.nativeName || "—"}
                  </Badge>
                </div>
              </button>
            ))}
          </div>

          <div className="border rounded-md bg-muted/20 min-h-[400px] max-h-[600px] overflow-y-auto">
            {selectedResult ? (
              <div className="p-4">
                <div className="flex items-center gap-2 mb-3 flex-wrap">
                  <h3 className="font-semibold text-sm">{selectedResult.domain}</h3>
                  <Badge variant="outline" className={`text-[10px] ${CATEGORY_COLORS[selectedResult.category]}`}>
                    {selectedResult.category}
                  </Badge>
                  <Badge variant="outline" className="text-[10px]">
                    <Globe className="w-2.5 h-2.5 mr-1" />
                    {selectedResult.language?.name || "Unknown"} ({selectedResult.language?.nativeName || "—"})
                  </Badge>
                  <span className="text-[10px] text-muted-foreground ml-auto">{selectedResult.totalDurationMs}ms</span>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => copyToClipboard(selectedResult.response, selectedResult.z28)}
                    data-testid={`button-copy-${selectedResult.z28}`}
                  >
                    {copiedId === selectedResult.z28 ? <Check className="w-3.5 h-3.5 text-green-500" /> : <Copy className="w-3.5 h-3.5" />}
                  </Button>
                </div>
                <div
                  className="text-sm text-foreground/90 leading-relaxed whitespace-pre-wrap select-text"
                  data-testid={`text-response-${selectedResult.z28}`}
                >
                  {selectedResult.response}
                </div>
              </div>
            ) : (
              <div className="flex items-center justify-center h-full p-8 text-center">
                <div>
                  <Eye className="w-8 h-8 text-muted-foreground/40 mx-auto mb-3" />
                  <p className="text-sm text-muted-foreground">Select an agent from the list to view its response</p>
                  <p className="text-xs text-muted-foreground/60 mt-1">
                    {filteredResults.length} responses available{languageFilter ? ` (filtered by ${activeLanguages.find(l => l.code === languageFilter)?.name})` : ""}
                  </p>
                </div>
              </div>
            )}
          </div>
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
  const [executiveSummary, setExecutiveSummary] = useState<ExecutiveSummary | null>(null);
  const [layer2Loading, setLayer2Loading] = useState(false);
  const [executiveLoading, setExecutiveLoading] = useState(false);
  const [totalDuration, setTotalDuration] = useState<number | null>(null);
  const [successCount, setSuccessCount] = useState(0);
  const [tribHash, setTribHash] = useState<string | null>(null);
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
    setExecutiveSummary(null);
    setLayer2Loading(false);
    setExecutiveLoading(false);
    setTotalDuration(null);
    setSuccessCount(0);
    setTribHash(null);
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

      es.addEventListener("executive_start", () => {
        setExecutiveLoading(true);
      });

      es.addEventListener("executive_complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          if (data.executiveSummary) {
            setExecutiveSummary(data.executiveSummary);
          }
          setExecutiveLoading(false);
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
          setTribHash(data.tribonacciHash || null);
          if (data.layer2) {
            setLayer2Sections(data.layer2);
          }
          if (data.executiveSummary) {
            setExecutiveSummary(data.executiveSummary);
          }
          setLayer2Loading(false);
          setExecutiveLoading(false);
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
            Launch 28 specialist AI agents in parallel via Tribonacci 13-step permutation scheduling.
            Each agent responds in a unique language across 28 world languages.
            Layer 1 delivers individual expert analyses. Layer 2 synthesizes a 5-section executive summary:
            Verdict, Jurisdictional Compass, Risk Barometer, Critical Path, and Plain English.
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
                placeholder="Enter your compliance query for the 28-agent array..."
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
                  <h2 className="text-lg font-bold">Layer 1 — Agent Deliberation</h2>
                  {isRunning && !layer2Loading && !executiveLoading && (
                    <Badge variant="outline" className="text-yellow-600 border-yellow-500/30 bg-yellow-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Processing
                    </Badge>
                  )}
                  {layer2Loading && (
                    <Badge variant="outline" className="text-blue-600 border-blue-500/30 bg-blue-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Synthesizing Layer 2...
                    </Badge>
                  )}
                  {executiveLoading && (
                    <Badge variant="outline" className="text-violet-600 border-violet-500/30 bg-violet-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Generating Executive Summary...
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

            {(() => {
              const completedResults = Array.from(agentStates.entries())
                .filter(([, s]) => s.status === "complete" && s.result)
                .map(([, s]) => s.result!);
              return completedResults.length > 0 && !isRunning ? (
                <ResponseViewer results={completedResults} />
              ) : null;
            })()}

            {executiveSummary && (
              <ExecutiveSummaryDisplay summary={executiveSummary} />
            )}

            {layer2Sections.length > 0 && (
              <Layer2Display sections={layer2Sections} />
            )}

            {consensus && !executiveSummary && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }}>
                <Card className="p-5 border-primary/10" data-testid="section-consensus">
                  <h2 className="text-lg font-bold mb-3">Final Consensus</h2>
                  <p className="text-sm text-foreground/90 leading-relaxed">{consensus}</p>
                </Card>
              </motion.div>
            )}

            {tribHash && !isRunning && (
              <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground">
                <span>Tribonacci Hash:</span>
                <code className="font-mono bg-muted px-2 py-0.5 rounded" data-testid="text-trib-hash">{tribHash}</code>
              </div>
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
                Enter a compliance query above and launch the agent array. All 28 specialist agents will process your query
                via Tribonacci 13-step permutation scheduling. The 5-section executive summary provides Verdict,
                Jurisdictional Compass, Risk Barometer, Critical Path, and Plain English analysis.
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
