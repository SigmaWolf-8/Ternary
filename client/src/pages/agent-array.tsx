/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { useState, useRef, useCallback, useMemo, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { ArrowLeft, Play, Loader2, CheckCircle2, XCircle, Circle, ChevronDown, ChevronUp, Settings2, RotateCcw, Shield, ShieldCheck, MapPin, AlertTriangle, ListOrdered, FileText, Copy, Check, Eye, Filter, Globe, Save, History, Clock, Download, BookOpen, Languages } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Link } from "wouter";
import { useToast } from "@/hooks/use-toast";
import { useQuery } from "@tanstack/react-query";
import { apiRequest, queryClient } from "@/lib/queryClient";
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

interface TranslationEntry {
  languageCode: string;
  languageName: string;
  nativeName: string;
  text: string;
}

interface EtymologyEntry {
  term: string;
  origin: string;
  evolution: string;
  crossCulturalNote: string;
  synchronized: boolean;
}

interface EtymologyAuditData {
  entries: EtymologyEntry[];
  flaggedTerms: string[];
  auditTimestamp: string;
}

interface VeritasClaim {
  claim: string;
  confidence: number;
  sources: string[];
  culturalTraditions: string[];
  verdict: "VERIFIED" | "UNVERIFIED" | "DISPUTED" | "FALSE";
  note: string;
}

interface VeritasAuditData {
  claims: VeritasClaim[];
  overallConfidence: number;
  falseClaims: number;
  disputedClaims: number;
  verifiedClaims: number;
  auditTimestamp: string;
}

interface LexicalProtocolData {
  version: string;
  termsEnforced: number;
  consistencyScore: number;
  corrections: { original: string; corrected: string; reason: string }[];
  latinTermsPreserved: string[];
}

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
                  className="w-full flex items-center justify-between gap-2 p-3 text-left"
                  data-testid={`button-toggle-executive-${idx}`}
                >
                  <div className="flex items-center gap-2 flex-wrap">
                    {sec.icon}
                    <span className="text-sm font-semibold">{sec.title}</span>
                  </div>
                  {isOpen ? <ChevronUp className="w-3 h-3" /> : <ChevronDown className="w-3 h-3" />}
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
                      <div className="px-3 pb-3">{sec.content}</div>
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

function SituationReportViewer({
  unifiedReport,
  translations,
  prompt,
  tribHash,
  executiveSummary,
  layer2Sections,
  successCount,
  totalDurationMs,
  onSaved,
}: {
  unifiedReport: string;
  translations: TranslationEntry[];
  prompt: string;
  tribHash: string | null;
  executiveSummary: ExecutiveSummary | null;
  layer2Sections: Layer2Section[];
  successCount: number;
  totalDurationMs: number | null;
  onSaved?: () => void;
}) {
  const [languageFilter, setLanguageFilter] = useState<string | null>(null);
  const [copiedLang, setCopiedLang] = useState<string | null>(null);
  const [copiedAll, setCopiedAll] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const { toast } = useToast();

  const filteredTranslations = useMemo(() => {
    if (!languageFilter) return translations;
    return translations.filter(t => t.languageCode === languageFilter);
  }, [translations, languageFilter]);

  const selectedTranslation = useMemo(() => {
    if (languageFilter) return filteredTranslations[0] || null;
    return null;
  }, [languageFilter, filteredTranslations]);

  const copyToClipboard = useCallback(async (text: string, langCode?: string) => {
    try {
      await navigator.clipboard.writeText(text);
      if (langCode) {
        setCopiedLang(langCode);
        setTimeout(() => setCopiedLang(null), 2000);
      } else {
        setCopiedAll(true);
        setTimeout(() => setCopiedAll(false), 2000);
      }
      toast({ title: "Copied", description: "Report copied to clipboard." });
    } catch {
      toast({ title: "Copy failed", description: "Could not copy to clipboard.", variant: "destructive" });
    }
  }, [toast]);

  const copyAllTranslations = useCallback(() => {
    const text = filteredTranslations
      .map(t => `=== ${t.languageName} (${t.nativeName}) ===\n\n${t.text}`)
      .join("\n\n" + "=".repeat(60) + "\n\n");
    copyToClipboard(text);
  }, [filteredTranslations, copyToClipboard]);

  const saveToDatabase = useCallback(async () => {
    setSaving(true);
    try {
      await apiRequest("POST", "/api/tribonacci/agent-array/save", {
        prompt,
        tribonacciHash: tribHash || "",
        unifiedReport,
        translations,
        executiveSummary: executiveSummary || null,
        layer2Sections: layer2Sections || null,
        agentCount: AGENT_COUNT,
        successCount,
        totalDurationMs: totalDurationMs || 0,
      });
      setSaved(true);
      toast({ title: "Report Saved", description: "Situation report saved to database. You can re-query it from the history panel." });
      queryClient.invalidateQueries({ queryKey: ["/api/tribonacci/agent-array/reports"] });
      if (onSaved) onSaved();
    } catch (err) {
      toast({ title: "Save Failed", description: err instanceof Error ? err.message : "Could not save report.", variant: "destructive" });
    } finally {
      setSaving(false);
    }
  }, [prompt, tribHash, unifiedReport, translations, executiveSummary, layer2Sections, successCount, totalDurationMs, toast, onSaved]);

  const displayText = selectedTranslation ? selectedTranslation.text : unifiedReport;
  const displayLangLabel = selectedTranslation
    ? `${selectedTranslation.languageName} (${selectedTranslation.nativeName})`
    : "English (Original)";

  return (
    <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} data-testid="section-situation-report">
      <Card className="p-5 border-primary/10">
        <div className="flex items-center gap-3 mb-4 flex-wrap">
          <Globe className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-bold">Situation Report</h2>
          <Badge variant="outline">
            {translations.length} Languages
          </Badge>
          <div className="ml-auto flex items-center gap-2 flex-wrap">
            <Button
              variant="outline"
              size="sm"
              onClick={copyAllTranslations}
              disabled={filteredTranslations.length === 0}
              data-testid="button-copy-all-translations"
            >
              {copiedAll ? <Check className="w-3.5 h-3.5 mr-1.5" /> : <Copy className="w-3.5 h-3.5 mr-1.5" />}
              {copiedAll ? "Copied!" : `Copy ${languageFilter ? "Selected" : "All"}`}
            </Button>
            <Button
              variant={saved ? "outline" : "default"}
              size="sm"
              onClick={saveToDatabase}
              disabled={saving || saved}
              data-testid="button-save-report"
            >
              {saving ? (
                <Loader2 className="w-3.5 h-3.5 mr-1.5 animate-spin" />
              ) : saved ? (
                <Check className="w-3.5 h-3.5 mr-1.5 text-green-500" />
              ) : (
                <Save className="w-3.5 h-3.5 mr-1.5" />
              )}
              {saving ? "Saving..." : saved ? "Saved" : "Save to Database"}
            </Button>
          </div>
        </div>

        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2 flex-wrap">
            <Filter className="w-3.5 h-3.5 text-muted-foreground" />
            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Select Language</span>
          </div>
          <div className="flex flex-wrap gap-1.5">
            <Button
              variant={languageFilter === null ? "default" : "outline"}
              size="sm"
              onClick={() => setLanguageFilter(null)}
              data-testid="button-filter-all"
            >
              Original (English)
            </Button>
            {translations.map((t) => (
              <Button
                key={t.languageCode}
                variant={languageFilter === t.languageCode ? "default" : "outline"}
                size="sm"
                onClick={() => setLanguageFilter(languageFilter === t.languageCode ? null : t.languageCode)}
                data-testid={`button-filter-${t.languageCode}`}
              >
                {t.nativeName}
              </Button>
            ))}
          </div>
        </div>

        <div className="border rounded-md bg-muted/20">
          <div className="flex items-center justify-between gap-2 p-3 border-b border-border/50 flex-wrap">
            <div className="flex items-center gap-2 flex-wrap">
              <Globe className="w-4 h-4 text-primary" />
              <span className="text-sm font-semibold" data-testid="text-current-language">{displayLangLabel}</span>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => copyToClipboard(displayText, selectedTranslation?.languageCode || "en")}
              data-testid="button-copy-current"
            >
              {copiedLang === (selectedTranslation?.languageCode || "en") ? (
                <Check className="w-3.5 h-3.5 text-green-500" />
              ) : (
                <Copy className="w-3.5 h-3.5" />
              )}
            </Button>
          </div>
          <div
            className="p-4 text-sm text-foreground/90 leading-relaxed whitespace-pre-wrap select-text max-h-[600px] overflow-y-auto"
            data-testid="text-situation-report"
          >
            {displayText}
          </div>
        </div>

        {!languageFilter && translations.length > 0 && (
          <div className="mt-4">
            <div className="flex items-center gap-2 mb-3 flex-wrap">
              <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                All Translations Preview
              </span>
            </div>
            <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-2">
              {translations.filter(t => t.languageCode !== "en").map((t) => (
                <div
                  key={t.languageCode}
                  className="rounded-md border border-border/50 p-3 cursor-pointer hover-elevate"
                  onClick={() => setLanguageFilter(t.languageCode)}
                  data-testid={`preview-${t.languageCode}`}
                >
                  <div className="flex items-center justify-between gap-2 mb-1.5 flex-wrap">
                    <div className="flex items-center gap-2 flex-wrap">
                      <Badge variant="outline" className="text-[10px]">{t.languageCode.toUpperCase()}</Badge>
                      <span className="text-xs font-medium">{t.nativeName}</span>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={(e) => { e.stopPropagation(); copyToClipboard(t.text, t.languageCode); }}
                      data-testid={`button-copy-${t.languageCode}`}
                    >
                      {copiedLang === t.languageCode ? (
                        <Check className="w-3 h-3 text-green-500" />
                      ) : (
                        <Copy className="w-3 h-3" />
                      )}
                    </Button>
                  </div>
                  <p className="text-[11px] text-muted-foreground line-clamp-3 leading-snug">
                    {t.text.startsWith("[Translation") ? t.text : t.text.slice(0, 200) + (t.text.length > 200 ? "..." : "")}
                  </p>
                </div>
              ))}
            </div>
          </div>
        )}
      </Card>
    </motion.div>
  );
}

function ReportHistory() {
  const { data, isLoading } = useQuery<{ reports: any[] }>({
    queryKey: ["/api/tribonacci/agent-array/reports"],
  });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [loadedReport, setLoadedReport] = useState<any | null>(null);
  const [loadingReport, setLoadingReport] = useState(false);
  const { toast } = useToast();

  const loadReport = useCallback(async (id: number) => {
    setLoadingReport(true);
    try {
      const res = await fetch(`/api/tribonacci/agent-array/reports/${id}`);
      if (!res.ok) throw new Error("Failed to load report");
      const data = await res.json();
      setLoadedReport(data.report);
      setSelectedId(id);
    } catch (err) {
      toast({ title: "Load Failed", description: err instanceof Error ? err.message : "Could not load report.", variant: "destructive" });
    } finally {
      setLoadingReport(false);
    }
  }, [toast]);

  const reports = data?.reports || [];

  if (isLoading) {
    return (
      <Card className="p-5 border-primary/10" data-testid="section-report-history">
        <div className="flex items-center gap-2">
          <Loader2 className="w-4 h-4 animate-spin" />
          <span className="text-sm text-muted-foreground">Loading saved reports...</span>
        </div>
      </Card>
    );
  }

  if (reports.length === 0) return null;

  return (
    <Card className="p-5 border-primary/10" data-testid="section-report-history">
      <div className="flex items-center gap-3 mb-4 flex-wrap">
        <History className="w-5 h-5 text-primary" />
        <h2 className="text-lg font-bold">Saved Reports</h2>
        <Badge variant="outline">{reports.length} Reports</Badge>
      </div>

      <div className="space-y-2 mb-4">
        {reports.map((r: any) => (
          <button
            key={r.id}
            onClick={() => loadReport(r.id)}
            className={`w-full text-left rounded-md px-3 py-2.5 transition-colors border ${
              selectedId === r.id
                ? "bg-primary/10 border-primary/30"
                : "border-transparent hover-elevate"
            }`}
            data-testid={`button-load-report-${r.id}`}
          >
            <div className="flex items-center gap-2 flex-wrap">
              <Clock className="w-3 h-3 text-muted-foreground shrink-0" />
              <span className="text-xs text-muted-foreground">
                {new Date(r.createdAt).toLocaleString()}
              </span>
              <Badge variant="outline" className="text-[10px]">
                {r.successCount}/{r.agentCount} agents
              </Badge>
              <span className="text-xs text-muted-foreground">
                {(r.totalDurationMs / 1000).toFixed(1)}s
              </span>
            </div>
            <p className="text-sm font-medium mt-1 line-clamp-1">{r.prompt}</p>
          </button>
        ))}
      </div>

      {loadingReport && (
        <div className="flex items-center gap-2 p-4">
          <Loader2 className="w-4 h-4 animate-spin" />
          <span className="text-sm text-muted-foreground">Loading report...</span>
        </div>
      )}

      {loadedReport && selectedId && (
        <LoadedReportViewer report={loadedReport} />
      )}
    </Card>
  );
}

function LoadedReportViewer({ report }: { report: any }) {
  const translations: TranslationEntry[] = Array.isArray(report.translations) ? report.translations : [];
  const [langFilter, setLangFilter] = useState<string | null>(null);
  const [copiedLang, setCopiedLang] = useState<string | null>(null);
  const { toast } = useToast();

  const displayed = langFilter
    ? translations.find(t => t.languageCode === langFilter) || null
    : null;

  const displayText = displayed ? displayed.text : report.unifiedReport;
  const displayLabel = displayed ? `${displayed.languageName} (${displayed.nativeName})` : "English (Original)";

  const copyText = useCallback(async (text: string, code?: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedLang(code || "en");
      setTimeout(() => setCopiedLang(null), 2000);
      toast({ title: "Copied", description: "Report copied to clipboard." });
    } catch {
      toast({ title: "Copy failed", variant: "destructive" });
    }
  }, [toast]);

  return (
    <div className="border-t border-border/50 pt-4 mt-2">
      <div className="flex flex-wrap gap-1.5 mb-3">
        <Button
          variant={langFilter === null ? "default" : "outline"}
          size="sm"
          onClick={() => setLangFilter(null)}
          data-testid="button-history-filter-all"
        >
          Original
        </Button>
        {translations.map(t => (
          <Button
            key={t.languageCode}
            variant={langFilter === t.languageCode ? "default" : "outline"}
            size="sm"
            onClick={() => setLangFilter(langFilter === t.languageCode ? null : t.languageCode)}
            data-testid={`button-history-filter-${t.languageCode}`}
          >
            {t.nativeName}
          </Button>
        ))}
      </div>

      <div className="border rounded-md bg-muted/20">
        <div className="flex items-center justify-between gap-2 p-3 border-b border-border/50 flex-wrap">
          <span className="text-sm font-semibold">{displayLabel}</span>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => copyText(displayText, displayed?.languageCode)}
            data-testid="button-copy-history-current"
          >
            {copiedLang === (displayed?.languageCode || "en") ? (
              <Check className="w-3.5 h-3.5 text-green-500" />
            ) : (
              <Copy className="w-3.5 h-3.5" />
            )}
          </Button>
        </div>
        <div className="p-4 text-sm text-foreground/90 leading-relaxed whitespace-pre-wrap select-text max-h-[400px] overflow-y-auto">
          {displayText}
        </div>
      </div>
    </div>
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
  const [reportLoading, setReportLoading] = useState(false);
  const [translationProgress, setTranslationProgress] = useState<Map<string, string>>(new Map());
  const [totalDuration, setTotalDuration] = useState<number | null>(null);
  const [successCount, setSuccessCount] = useState(0);
  const [tribHash, setTribHash] = useState<string | null>(null);
  const [unifiedReport, setUnifiedReport] = useState<string | null>(null);
  const [translations, setTranslations] = useState<TranslationEntry[]>([]);
  const [etymologyAudit, setEtymologyAudit] = useState<EtymologyAuditData | null>(null);
  const [etymologyLoading, setEtymologyLoading] = useState(false);
  const [veritasAudit, setVeritasAudit] = useState<VeritasAuditData | null>(null);
  const [veritasLoading, setVeritasLoading] = useState(false);
  const [lexicalProtocol, setLexicalProtocol] = useState<LexicalProtocolData | null>(null);
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
    setReportLoading(false);
    setTranslationProgress(new Map());
    setTotalDuration(null);
    setSuccessCount(0);
    setTribHash(null);
    setUnifiedReport(null);
    setTranslations([]);
    setEtymologyAudit(null);
    setEtymologyLoading(false);
    setVeritasAudit(null);
    setVeritasLoading(false);
    setLexicalProtocol(null);
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

      es.addEventListener("layer1_complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          setSuccessCount(data.successCount || 0);
        } catch {}
      });

      es.addEventListener("layer2_start", () => {
        setLayer2Loading(true);
      });

      es.addEventListener("layer2_complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          if (data.sections) setLayer2Sections(data.sections);
          setLayer2Loading(false);
        } catch {}
      });

      es.addEventListener("executive_start", () => {
        setExecutiveLoading(true);
      });

      es.addEventListener("executive_complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          if (data.executiveSummary) setExecutiveSummary(data.executiveSummary);
          setExecutiveLoading(false);
        } catch {}
      });

      es.addEventListener("etymology_start", () => {
        setEtymologyLoading(true);
      });

      es.addEventListener("etymology_complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          setEtymologyAudit({
            entries: data.entries || [],
            flaggedTerms: data.flaggedTerms || [],
            auditTimestamp: new Date().toISOString(),
          });
          setEtymologyLoading(false);
        } catch {}
      });

      es.addEventListener("veritas_start", () => {
        setVeritasLoading(true);
      });

      es.addEventListener("veritas_complete", (e) => {
        try {
          const data = JSON.parse(e.data);
          setVeritasAudit({
            claims: data.claims || [],
            overallConfidence: data.overallConfidence || 0,
            falseClaims: data.falseClaims || 0,
            disputedClaims: data.disputedClaims || 0,
            verifiedClaims: data.verifiedClaims || 0,
            auditTimestamp: new Date().toISOString(),
          });
          setVeritasLoading(false);
        } catch {}
      });

      es.addEventListener("lexical_applied", (e) => {
        try {
          const data = JSON.parse(e.data);
          setLexicalProtocol({
            version: data.version || "2.0",
            termsEnforced: data.termsEnforced || 0,
            consistencyScore: data.consistencyScore || 0,
            corrections: data.corrections || [],
            latinTermsPreserved: data.latinTermsPreserved || [],
          });
        } catch {}
      });

      es.addEventListener("report_start", () => {
        setReportLoading(true);
      });

      es.addEventListener("report_generated", (e) => {
        try {
          const data = JSON.parse(e.data);
          if (data.report) setUnifiedReport(data.report);
        } catch {}
      });

      es.addEventListener("translation_progress", (e) => {
        try {
          const data = JSON.parse(e.data);
          setTranslationProgress(prev => {
            const next = new Map(prev);
            next.set(data.code, data.status);
            return next;
          });
        } catch {}
      });

      es.addEventListener("translations_complete", () => {
        setReportLoading(false);
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
          if (data.layer2) setLayer2Sections(data.layer2);
          if (data.executiveSummary) setExecutiveSummary(data.executiveSummary);
          if (data.unifiedReport) setUnifiedReport(data.unifiedReport);
          if (data.translations) setTranslations(data.translations);
          if (data.etymologyAudit) setEtymologyAudit(data.etymologyAudit);
          if (data.veritasAudit) setVeritasAudit(data.veritasAudit);
          if (data.lexicalProtocol) setLexicalProtocol(data.lexicalProtocol);
          setLayer2Loading(false);
          setExecutiveLoading(false);
          setReportLoading(false);
          setEtymologyLoading(false);
          setVeritasLoading(false);
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

  const translationsCompleted = Array.from(translationProgress.values()).filter(s => s === "complete").length;

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
            Launch 28 specialist AI agents simultaneously via Tribonacci 13-step permutation scheduling.
            All agents analyze your query in parallel, then produce one unified Situation Report
            automatically translated into 28 world languages. Reports are copyable and savable to the database for future retrieval.
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
                  <div>1 Report &middot; 28 Languages</div>
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
                  {isRunning && !layer2Loading && !executiveLoading && !reportLoading && (
                    <Badge variant="outline" className="text-yellow-600 border-yellow-500/30 bg-yellow-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Agents Processing
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
                      Executive Summary...
                    </Badge>
                  )}
                  {etymologyLoading && (
                    <Badge variant="outline" className="text-amber-600 border-amber-500/30 bg-amber-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Etymology Audit...
                    </Badge>
                  )}
                  {veritasLoading && (
                    <Badge variant="outline" className="text-red-600 border-red-500/30 bg-red-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Veritas Fact-Check...
                    </Badge>
                  )}
                  {reportLoading && (
                    <Badge variant="outline" className="text-emerald-600 border-emerald-500/30 bg-emerald-500/10">
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Translating Report ({translationsCompleted}/{AGENT_LANGUAGES.length})...
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

            {(etymologyAudit || veritasAudit || lexicalProtocol) && !isRunning && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} data-testid="section-audit-protocols">
                <Card className="p-5 border-primary/10">
                  <div className="flex items-center gap-3 mb-4 flex-wrap">
                    <Shield className="w-5 h-5 text-primary" />
                    <h2 className="text-lg font-bold">Integrity Protocols</h2>
                    {veritasAudit && (
                      <Badge
                        variant="outline"
                        className={veritasAudit.falseClaims > 0
                          ? "text-red-600 border-red-500/30 bg-red-500/10"
                          : veritasAudit.disputedClaims > 0
                          ? "text-amber-600 border-amber-500/30 bg-amber-500/10"
                          : "text-green-600 border-green-500/30 bg-green-500/10"
                        }
                      >
                        {veritasAudit.falseClaims > 0
                          ? `${veritasAudit.falseClaims} False Claims Filtered`
                          : veritasAudit.disputedClaims > 0
                          ? `${veritasAudit.disputedClaims} Disputed`
                          : `${veritasAudit.verifiedClaims} Verified`}
                      </Badge>
                    )}
                    {lexicalProtocol && (
                      <Badge variant="outline">
                        Lexical v{lexicalProtocol.version}
                      </Badge>
                    )}
                  </div>

                  <div className="grid md:grid-cols-3 gap-4">
                    {etymologyAudit && (
                      <div className="border rounded-md p-3" data-testid="panel-etymology">
                        <div className="flex items-center gap-2 mb-2 flex-wrap">
                          <BookOpen className="w-4 h-4 text-amber-500" />
                          <span className="text-sm font-semibold">Etymology Engine</span>
                          <Badge variant="outline" className="text-[10px]">{etymologyAudit.entries.length} Terms</Badge>
                        </div>
                        {etymologyAudit.flaggedTerms.length > 0 && (
                          <div className="mb-2 p-2 rounded bg-amber-500/10 border border-amber-500/20">
                            <span className="text-xs font-medium text-amber-600">Flagged: </span>
                            <span className="text-xs text-amber-700 dark:text-amber-400">{etymologyAudit.flaggedTerms.join(", ")}</span>
                          </div>
                        )}
                        <div className="space-y-1.5 max-h-[200px] overflow-y-auto">
                          {etymologyAudit.entries.map((entry, i) => (
                            <div key={i} className="text-xs border-b border-border/30 pb-1.5 last:border-0">
                              <div className="flex items-center gap-1.5 flex-wrap">
                                <span className="font-semibold">{entry.term}</span>
                                {!entry.synchronized && (
                                  <Badge variant="outline" className="text-[9px] text-amber-600 border-amber-500/30">Unsync</Badge>
                                )}
                              </div>
                              <div className="text-muted-foreground">{entry.origin}</div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {veritasAudit && (
                      <div className="border rounded-md p-3" data-testid="panel-veritas">
                        <div className="flex items-center gap-2 mb-2 flex-wrap">
                          <ShieldCheck className="w-4 h-4 text-blue-500" />
                          <span className="text-sm font-semibold">Veritas Audit</span>
                          <Badge variant="outline" className="text-[10px]">
                            {Math.round(veritasAudit.overallConfidence * 100)}% Conf.
                          </Badge>
                        </div>
                        <div className="flex gap-2 mb-2 flex-wrap">
                          <Badge variant="outline" className="text-[10px] text-green-600 border-green-500/30 bg-green-500/10">
                            {veritasAudit.verifiedClaims} Verified
                          </Badge>
                          {veritasAudit.disputedClaims > 0 && (
                            <Badge variant="outline" className="text-[10px] text-amber-600 border-amber-500/30 bg-amber-500/10">
                              {veritasAudit.disputedClaims} Disputed
                            </Badge>
                          )}
                          {veritasAudit.falseClaims > 0 && (
                            <Badge variant="outline" className="text-[10px] text-red-600 border-red-500/30 bg-red-500/10">
                              {veritasAudit.falseClaims} False
                            </Badge>
                          )}
                        </div>
                        <div className="space-y-1.5 max-h-[200px] overflow-y-auto">
                          {veritasAudit.claims.map((claim, i) => (
                            <div key={i} className="text-xs border-b border-border/30 pb-1.5 last:border-0">
                              <div className="flex items-center gap-1.5 flex-wrap">
                                <Badge
                                  variant="outline"
                                  className={`text-[9px] ${
                                    claim.verdict === "VERIFIED" ? "text-green-600 border-green-500/30" :
                                    claim.verdict === "FALSE" ? "text-red-600 border-red-500/30" :
                                    claim.verdict === "DISPUTED" ? "text-amber-600 border-amber-500/30" :
                                    "text-gray-600 border-gray-500/30"
                                  }`}
                                >
                                  {claim.verdict}
                                </Badge>
                                <span className="text-muted-foreground">{Math.round(claim.confidence * 100)}%</span>
                              </div>
                              <div className="mt-0.5 line-clamp-2">{claim.claim}</div>
                              {claim.note && <div className="text-muted-foreground mt-0.5 line-clamp-1">{claim.note}</div>}
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {lexicalProtocol && (
                      <div className="border rounded-md p-3" data-testid="panel-lexical">
                        <div className="flex items-center gap-2 mb-2 flex-wrap">
                          <Languages className="w-4 h-4 text-emerald-500" />
                          <span className="text-sm font-semibold">Lexical Protocol</span>
                          <Badge variant="outline" className="text-[10px]">v{lexicalProtocol.version}</Badge>
                        </div>
                        <div className="space-y-2 text-xs">
                          <div className="flex items-center justify-between gap-2">
                            <span className="text-muted-foreground">Terms Enforced</span>
                            <span className="font-medium">{lexicalProtocol.termsEnforced}</span>
                          </div>
                          <div className="flex items-center justify-between gap-2">
                            <span className="text-muted-foreground">Consistency</span>
                            <span className="font-medium">{Math.round(lexicalProtocol.consistencyScore * 100)}%</span>
                          </div>
                          {lexicalProtocol.latinTermsPreserved.length > 0 && (
                            <div>
                              <span className="text-muted-foreground">Latin Preserved:</span>
                              <div className="flex flex-wrap gap-1 mt-1">
                                {lexicalProtocol.latinTermsPreserved.map((t, i) => (
                                  <Badge key={i} variant="outline" className="text-[9px]">{t}</Badge>
                                ))}
                              </div>
                            </div>
                          )}
                          {lexicalProtocol.corrections.length > 0 && (
                            <div>
                              <span className="text-muted-foreground">Corrections Applied:</span>
                              <div className="mt-1 space-y-1">
                                {lexicalProtocol.corrections.map((c, i) => (
                                  <div key={i} className="text-[10px] text-muted-foreground">
                                    {c.original} — {c.reason}
                                  </div>
                                ))}
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                </Card>
              </motion.div>
            )}

            {unifiedReport && translations.length > 0 && !isRunning && (
              <SituationReportViewer
                unifiedReport={unifiedReport}
                translations={translations}
                prompt={prompt}
                tribHash={tribHash}
                executiveSummary={executiveSummary}
                layer2Sections={layer2Sections}
                successCount={successCount}
                totalDurationMs={totalDuration}
              />
            )}

            {executiveSummary && (
              <ExecutiveSummaryDisplay summary={executiveSummary} />
            )}

            {layer2Sections.length > 0 && (
              <Layer2Display sections={layer2Sections} />
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
          <div className="space-y-6">
            <Card className="p-8 border-primary/10 text-center" data-testid="section-empty-state">
              <div className="max-w-md mx-auto">
                <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-4">
                  <Play className="w-6 h-6 text-primary" />
                </div>
                <h3 className="font-semibold mb-2">Ready to Launch</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Enter a compliance query above and launch the agent array. All 28 specialist agents process your query
                  simultaneously, then produce one unified Situation Report translated into 28 languages.
                  Save reports to the database for future reference.
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

            <ReportHistory />
          </div>
        )}
      </div>
    </div>
  );
}
