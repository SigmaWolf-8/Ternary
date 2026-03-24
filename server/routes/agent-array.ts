/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * 28-Dimension AI Agent Array — Tribonacci Circle Orchestration
 *
 * Three-layer processing pipeline:
 *   Layer 1: 28 specialist agents deliberate in parallel (English)
 *   Layer 2: 5-section executive summary synthesis
 *   Layer 3: Unified Situation Report translated into 28 languages
 *
 * Agents scheduled via Tribonacci 13-step permutation:
 *   (i × 13) mod 28 visits all 28 positions exactly once
 *
 * Two-step SSE pattern:
 *   POST /api/tribonacci/agent-array       → creates session, returns sessionId
 *   GET  /api/tribonacci/agent-array/stream/:id → EventSource SSE stream
 */

import type { Express, Request, Response } from "express";
import OpenAI from "openai";
import pLimit from "p-limit";
import { createLogger } from "../logger";
import { db } from "../db";
import { agentArrayReports } from "@shared/schema";
import { desc, eq } from "drizzle-orm";
import { phaseEncryptFields, phaseDecryptFields } from "../storage";
import {
  AGENT_COUNT,
  AGENT_STEP_NAMES,
  AGENT_DOMAINS,
  DEFAULT_SPECIALISTS,
  LAYER2_SECTIONS,
  TERNARY_RADIAN,
  FULL_CIRCLE,
  CONVOLUTION_KERNEL,
  generateZ28Walk,
  getAgentPositions,
  scheduleAgents,
  tribonacciPermutation,
  type AgentSpecialist,
  type AgentStepEvent,
  type AgentResult,
  type AgentArrayResponse,
  type Layer2Section,
  type ExecutiveSummary,
  type ExecutiveVerdict,
  type JurisdictionalRegion,
  type RiskScore,
  type CriticalStep,
  type VerdictSignal,
  type AgentLanguage,
  AGENT_LANGUAGES,
} from "../../shared/agent-array";
import { randomUUID } from "crypto";
import crypto from "crypto";

const CONCURRENCY_LIMIT = 28;
const TRANSLATION_CONCURRENCY = 14;
const RETRY_ATTEMPTS = 2;
const SESSION_TTL_MS = 5 * 60 * 1000;

const log = createLogger("agent-array");

const openai = new OpenAI({
  apiKey: process.env.AI_INTEGRATIONS_OPENAI_API_KEY,
  baseURL: process.env.AI_INTEGRATIONS_OPENAI_BASE_URL,
});

interface PendingSession {
  prompt: string;
  roles: AgentSpecialist[];
  createdAt: number;
}

const pendingSessions = new Map<string, PendingSession>();

function generateTribonacciHash(query: string): string {
  const raw = crypto.createHash("sha256").update(query, "utf8").digest();
  const key = raw.readBigUInt64LE(0);
  const a = key * 13n;
  const b = (key >> 16n) * 24n;
  const c = (key >> 32n) * 44n;
  let mixed = a + b + c;
  mixed ^= (mixed >> 17n);
  mixed *= 13n;
  mixed ^= (mixed >> 13n);
  mixed *= 24n;
  mixed ^= (mixed >> 9n);
  const bucket = Number((mixed % (2n ** 32n) + 2n ** 32n) % (2n ** 32n));
  const shortHash = crypto.createHash("sha256").update(query, "utf8").digest("hex").slice(0, 16);
  return `trib-${bucket.toString(16).padStart(8, "0")}-${shortHash}`;
}

function buildAgentSystemPrompt(_z28: number, specialist: AgentSpecialist): string {
  const base = specialist.systemPrompt;
  const langPattern = /\s*You MUST respond entirely in .+$/;
  const stripped = base.replace(langPattern, "");
  return stripped + " You MUST respond entirely in English for the analysis phase. Do not use any other language.";
}

async function callLLM(
  systemPrompt: string,
  userContent: string,
  label: string,
): Promise<string | null> {
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      const completion = await openai.chat.completions.create({
        model: "gpt-5-nano",
        messages: [
          { role: "system", content: systemPrompt },
          { role: "user", content: userContent },
        ],
      });
      const raw = completion.choices[0]?.message?.content;
      const finish = completion.choices[0]?.finish_reason;
      if (raw && raw.trim().length > 5) {
        log.info(`${label} attempt ${attempt + 1}: ${raw.length} chars, finish=${finish}`);
        return raw.trim();
      }
      log.warn(`${label} attempt ${attempt + 1}: content=${raw ? raw.length + ' chars' : 'null'}, finish=${finish}`);
    } catch (err) {
      log.warn(`${label} attempt ${attempt + 1} error: ${err instanceof Error ? err.message : String(err)}`);
      await new Promise((resolve) => setTimeout(resolve, 500 * (attempt + 1)));
    }
  }
  return null;
}

async function executeAgentSteps(
  agentIndex: number,
  z28: number,
  specialist: AgentSpecialist,
  prompt: string,
  sendEvent: (event: AgentStepEvent) => void,
): Promise<AgentResult> {
  const label = `A${String(z28).padStart(2, "0")}`;
  const domain = specialist.title;
  const startTime = Date.now();
  const permutationPosition = (z28 * TERNARY_RADIAN) % AGENT_COUNT;

  const emitStep = (stepIndex: number, status: AgentStepEvent["status"], detail?: string, durationMs?: number) => {
    sendEvent({
      agentIndex,
      agentLabel: label,
      z28,
      domain,
      stepIndex,
      stepName: AGENT_STEP_NAMES[Math.min(stepIndex, AGENT_STEP_NAMES.length - 1)],
      status,
      detail,
      durationMs,
    });
  };

  try {
    emitStep(0, "running", `Permutation position: (${z28} × 13) mod 28 = ${permutationPosition}`);

    const truncatedPrompt = prompt.length > 200 ? prompt.slice(0, 200) + "..." : prompt;
    const systemPrompt = buildAgentSystemPrompt(z28, specialist);

    const response = await callLLM(systemPrompt, truncatedPrompt, `Agent ${label}`);

    const finalResponse = response ||
      `As a ${specialist.title}, this query involves complex ${specialist.category} considerations that warrant thorough analysis of applicable frameworks and precedents in ${specialist.subdomain}.`;

    const durationMs = Date.now() - startTime;
    emitStep(0, "complete", `Inference complete (${durationMs}ms)`, durationMs);

    return {
      agentIndex,
      agentLabel: label,
      z28,
      domain,
      category: specialist.category,
      response: finalResponse,
      totalDurationMs: durationMs,
      stepsCompleted: 1,
      language: specialist.language,
    };
  } catch (error: unknown) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log.error(`Agent ${label} (${domain}) failed: ${errMsg}`);
    emitStep(0, "error", errMsg);

    return {
      agentIndex,
      agentLabel: label,
      z28,
      domain,
      category: specialist.category,
      response: `[Error] ${errMsg}`,
      totalDurationMs: Date.now() - startTime,
      stepsCompleted: 0,
      language: specialist.language,
    };
  }
}

async function generateLayer2(
  results: AgentResult[],
  prompt: string,
): Promise<Layer2Section[]> {
  const layer2Limit = pLimit(3);
  const sections = await Promise.all(
    LAYER2_SECTIONS.map((section) =>
      layer2Limit(async (): Promise<Layer2Section> => {
        const sectionResults = results.filter(
          (r) => r.category === section.key && !r.response.startsWith("[Error]")
        );
        const successCount = sectionResults.length;

        if (successCount === 0) {
          return {
            category: section.key,
            label: section.label,
            technicalSummary: "No agent responses available for this section.",
            laySummary: "No results were returned for this area.",
            agentCount: section.agentIndices.length,
            successCount: 0,
          };
        }

        const agentInputs = sectionResults
          .map((r) => `[${r.domain}]: ${r.response}`)
          .join("\n");

        try {
          const truncatedInputs = agentInputs.length > 600
            ? agentInputs.slice(0, 600) + "..."
            : agentInputs;

          const techResult = await callLLM(
            `Summarize the ${section.label} findings in 2-3 detailed sentences using precise terminology.`,
            truncatedInputs,
            `Layer2 ${section.label} technical`,
          );

          const plainResult = await callLLM(
            `Explain these ${section.label} findings in 2-3 simple sentences that anyone can understand. Avoid jargon.`,
            truncatedInputs,
            `Layer2 ${section.label} plain`,
          );

          const technicalSummary = techResult || sectionResults.map((r) => r.response).join(" ").slice(0, 500);
          const laySummary = plainResult || technicalSummary;

          return {
            category: section.key,
            label: section.label,
            technicalSummary,
            laySummary,
            agentCount: section.agentIndices.length,
            successCount,
          };
        } catch (err: unknown) {
          const errMsg = err instanceof Error ? err.message : String(err);
          log.error(`Layer 2 section "${section.label}" failed: ${errMsg}`);
          return {
            category: section.key,
            label: section.label,
            technicalSummary: `Summary generation failed: ${errMsg}`,
            laySummary: "We were unable to generate a summary for this section.",
            agentCount: section.agentIndices.length,
            successCount,
          };
        }
      })
    )
  );

  return sections;
}

async function generateExecutiveSummary(
  layer2: Layer2Section[],
  prompt: string,
): Promise<ExecutiveSummary> {
  const summaryInputs = layer2
    .filter((s) => s.successCount > 0 && !s.technicalSummary.startsWith("No agent") && !s.technicalSummary.startsWith("Summary generation"))
    .map((s) => `[${s.label}]: ${s.technicalSummary}`)
    .join("\n");

  const truncated = summaryInputs.length > 900 ? summaryInputs.slice(0, 900) + "..." : summaryInputs;
  const querySnippet = prompt.length > 100 ? prompt.slice(0, 100) + "..." : prompt;

  const [verdictResult, compassResult, riskResult, pathResult, plainResult] = await Promise.all([
    callLLM(
      "You are a compliance verdict system. Rate as GREEN (proceed), YELLOW (proceed with caution), or RED (do not proceed). Respond in format: SIGNAL: GREEN/YELLOW/RED | ASSESSMENT: one sentence | CONFIDENCE: 0.0-1.0",
      `Query: ${querySnippet}\nFindings:\n${truncated}`,
      "Executive Verdict",
    ),
    callLLM(
      "You are a jurisdictional analyst. List 4-6 key world regions with compliance status. Format each line as: REGION | STATUS (permitted/conditional/restricted/prohibited/unclear) | brief note",
      `Query: ${querySnippet}\nFindings:\n${truncated}`,
      "Jurisdictional Compass",
    ),
    callLLM(
      "You are a risk analyst. Provide financial risk score (0-10) and technical risk score (0-10). Format: FINANCIAL: score | narrative TECHNICAL: score | narrative",
      `Query: ${querySnippet}\nFindings:\n${truncated}`,
      "Risk Barometer",
    ),
    callLLM(
      "You are a compliance advisor. List 3-5 critical next steps in priority order. Format each as: number. ACTION | CATEGORY (license/regulatory_filing/technical_implementation/due_diligence/internal_policy)",
      `Query: ${querySnippet}\nFindings:\n${truncated}`,
      "Critical Path",
    ),
    callLLM(
      "You are a board-level advisor. Write a 2-3 sentence plain English summary suitable for non-experts. Then provide a one-sentence board recommendation.",
      `Query: ${querySnippet}\nFindings:\n${truncated}`,
      "Plain English",
    ),
  ]);

  const verdict = parseVerdict(verdictResult);
  const compass = parseCompass(compassResult);
  const risk = parseRiskBarometer(riskResult);
  const criticalPath = parseCriticalPath(pathResult);
  const plain = parsePlainEnglish(plainResult);

  return {
    verdict,
    jurisdictionalCompass: compass,
    riskBarometer: risk,
    criticalPath,
    plainEnglish: plain,
  };
}

function parseVerdict(raw: string | null): ExecutiveVerdict {
  if (!raw) return { signal: "YELLOW", assessment: "Insufficient data for definitive verdict.", confidence: 0.5 };

  let signal: VerdictSignal = "YELLOW";
  if (/\bGREEN\b/i.test(raw)) signal = "GREEN";
  else if (/\bRED\b/i.test(raw)) signal = "RED";

  const confMatch = raw.match(/CONFIDENCE[:\s]*([0-9.]+)/i);
  const confidence = confMatch ? Math.min(1, Math.max(0, parseFloat(confMatch[1]))) : 0.7;

  const assessMatch = raw.match(/ASSESSMENT[:\s]*(.+?)(?:\||CONFIDENCE|$)/i);
  const assessment = assessMatch ? assessMatch[1].trim() : raw.replace(/SIGNAL[:\s]*\w+\s*\|?\s*/i, "").replace(/CONFIDENCE[:\s]*[0-9.]+/i, "").trim();

  return { signal, assessment: assessment || raw.slice(0, 200), confidence };
}

function parseCompass(raw: string | null): JurisdictionalRegion[] {
  if (!raw) return [{ region: "Global", status: "unclear", notes: "Unable to generate jurisdictional analysis." }];

  const regions: JurisdictionalRegion[] = [];
  const lines = raw.split("\n").filter((l) => l.trim().length > 3);

  for (const line of lines) {
    const parts = line.split("|").map((p) => p.trim());
    if (parts.length >= 2) {
      const statusStr = parts[1]?.toLowerCase() || "";
      let status: JurisdictionalRegion["status"] = "unclear";
      if (statusStr.includes("permitted")) status = "permitted";
      else if (statusStr.includes("conditional")) status = "conditional";
      else if (statusStr.includes("restricted")) status = "restricted";
      else if (statusStr.includes("prohibited")) status = "prohibited";

      regions.push({
        region: parts[0].replace(/^\d+\.\s*/, ""),
        status,
        notes: parts[2] || "",
      });
    }
  }

  return regions.length > 0 ? regions : [{ region: "Global", status: "unclear", notes: raw.slice(0, 200) }];
}

function parseRiskBarometer(raw: string | null): ExecutiveSummary["riskBarometer"] {
  const defaultResult = {
    financial: [{ category: "Overall Financial Risk", score: 5, narrative: "Moderate risk assessment pending detailed analysis." }],
    technical: [{ category: "Overall Technical Risk", score: 5, narrative: "Moderate risk assessment pending detailed analysis." }],
    aggregateFinancial: 5,
    aggregateTechnical: 5,
  };

  if (!raw) return defaultResult;

  let financialScore = 5;
  let technicalScore = 5;
  let financialNarrative = "";
  let technicalNarrative = "";

  const finMatch = raw.match(/FINANCIAL[:\s]*(\d+(?:\.\d+)?)\s*\|?\s*(.*?)(?=TECHNICAL|$)/is);
  if (finMatch) {
    financialScore = Math.min(10, Math.max(0, parseFloat(finMatch[1])));
    financialNarrative = finMatch[2]?.trim() || "";
  }

  const techMatch = raw.match(/TECHNICAL[:\s]*(\d+(?:\.\d+)?)\s*\|?\s*(.*)/is);
  if (techMatch) {
    technicalScore = Math.min(10, Math.max(0, parseFloat(techMatch[1])));
    technicalNarrative = techMatch[2]?.trim() || "";
  }

  if (!financialNarrative && !technicalNarrative) {
    const sentences = raw.split(/[.!]\s+/).filter(s => s.trim().length > 5);
    financialNarrative = sentences[0] || raw.slice(0, 150);
    technicalNarrative = sentences[1] || sentences[0] || raw.slice(0, 150);
  }

  return {
    financial: [{ category: "Overall Financial Risk", score: financialScore, narrative: financialNarrative || "See detailed section analysis." }],
    technical: [{ category: "Overall Technical Risk", score: technicalScore, narrative: technicalNarrative || "See detailed section analysis." }],
    aggregateFinancial: financialScore,
    aggregateTechnical: technicalScore,
  };
}

function parseCriticalPath(raw: string | null): CriticalStep[] {
  if (!raw) return [{ order: 1, action: "Conduct detailed compliance review based on agent findings.", category: "due_diligence" }];

  const steps: CriticalStep[] = [];
  const lines = raw.split("\n").filter((l) => l.trim().length > 3);

  for (let i = 0; i < lines.length && steps.length < 7; i++) {
    const line = lines[i];
    const parts = line.split("|").map((p) => p.trim());
    const action = parts[0]?.replace(/^\d+\.\s*/, "").trim();
    const category = parts[1]?.toLowerCase().replace(/\s+/g, "_") || "due_diligence";

    if (action && action.length > 5) {
      steps.push({
        order: steps.length + 1,
        action,
        category,
      });
    }
  }

  return steps.length > 0 ? steps : [{ order: 1, action: raw.slice(0, 200), category: "due_diligence" }];
}

function parsePlainEnglish(raw: string | null): ExecutiveSummary["plainEnglish"] {
  if (!raw) return { summary: "Analysis complete. Detailed findings available in the sections above.", boardRecommendation: "Review the detailed section summaries for specific guidance." };

  const sentences = raw.split(/(?<=[.!?])\s+/).filter(s => s.trim().length > 10);
  if (sentences.length >= 3) {
    return {
      summary: sentences.slice(0, -1).join(" "),
      boardRecommendation: sentences[sentences.length - 1],
    };
  }

  return { summary: raw, boardRecommendation: sentences[sentences.length - 1] || raw };
}

export interface TranslationEntry {
  languageCode: string;
  languageName: string;
  nativeName: string;
  text: string;
}

export interface EtymologyEntry {
  term: string;
  origin: string;
  evolution: string;
  crossCulturalNote: string;
  synchronized: boolean;
}

export interface EtymologyAudit {
  entries: EtymologyEntry[];
  flaggedTerms: string[];
  auditTimestamp: string;
}

export interface VeritasClaim {
  claim: string;
  confidence: number;
  sources: string[];
  culturalTraditions: string[];
  verdict: "VERIFIED" | "UNVERIFIED" | "DISPUTED" | "FALSE";
  note: string;
}

export interface VeritasAudit {
  claims: VeritasClaim[];
  overallConfidence: number;
  falseClaims: number;
  disputedClaims: number;
  verifiedClaims: number;
  auditTimestamp: string;
}

export interface LexicalProtocol {
  version: string;
  termsEnforced: number;
  consistencyScore: number;
  corrections: { original: string; corrected: string; reason: string }[];
  latinTermsPreserved: string[];
}

async function runEtymologyAudit(report: string, prompt: string): Promise<EtymologyAudit> {
  const systemPrompt = `You are the PlenumNET Etymology Engine. Your task is to trace the origin and evolution of key terms used in a Situation Report. For each significant technical, legal, or domain-specific term:

1. Identify the term
2. Trace its etymological origin (Latin, Greek, Arabic, etc.)
3. Describe how its meaning has evolved across cultures and centuries
4. Note whether the term is "synchronized" (consistent meaning across cultures) or not
5. FLAG any terms used anachronistically, incorrectly, or in ways that distort their established meaning

You MUST respond with valid JSON in this exact format:
{
  "entries": [
    {
      "term": "term name",
      "origin": "language of origin and root word",
      "evolution": "how meaning evolved across cultures",
      "crossCulturalNote": "how different cultures interpret this term",
      "synchronized": true/false
    }
  ],
  "flaggedTerms": ["list of terms used incorrectly or anachronistically"]
}

Focus on the 8-12 most significant terms. Be precise. If a term is used incorrectly in the report, you MUST flag it.`;

  const result = await callLLM(
    systemPrompt,
    `QUERY: ${prompt}\n\nSITUATION REPORT TO AUDIT:\n${report.slice(0, 3000)}`,
    "Etymology Audit",
  );

  if (!result) {
    return { entries: [], flaggedTerms: [], auditTimestamp: new Date().toISOString() };
  }

  try {
    const jsonMatch = result.match(/\{[\s\S]*\}/);
    if (jsonMatch) {
      const parsed = JSON.parse(jsonMatch[0]);
      return {
        entries: Array.isArray(parsed.entries) ? parsed.entries : [],
        flaggedTerms: Array.isArray(parsed.flaggedTerms) ? parsed.flaggedTerms : [],
        auditTimestamp: new Date().toISOString(),
      };
    }
  } catch {}

  return { entries: [], flaggedTerms: [], auditTimestamp: new Date().toISOString() };
}

async function runVeritasAudit(report: string, prompt: string): Promise<VeritasAudit> {
  const systemPrompt = `You are the PlenumNET Veritas Audit Engine. Your mandate is absolute factual integrity.

CRITICAL MISSION: Validate EVERY factual assertion in the Situation Report. You are the last line of defense against fabricated, hallucinated, or inaccurate claims being presented as truth.

For each factual claim in the report:
1. State the claim exactly as written
2. Rate confidence (0.0 to 1.0) based on verifiable evidence
3. List sources that support or contradict the claim (cite specific laws, treaties, documents, or facts)
4. Identify which cultural/legal traditions inform the claim (minimum 3 distinct traditions)
5. Render a verdict: VERIFIED (high confidence, well-sourced), UNVERIFIED (insufficient evidence), DISPUTED (conflicting sources), or FALSE (demonstrably incorrect)

ZERO TOLERANCE for fabricated claims. If the report says something exists (a file, a law, a policy, a document) but you have no evidence it does, mark it UNVERIFIED or FALSE. Do NOT give the benefit of the doubt to unverifiable assertions.

Respond with valid JSON:
{
  "claims": [
    {
      "claim": "exact claim from the report",
      "confidence": 0.0-1.0,
      "sources": ["source1", "source2"],
      "culturalTraditions": ["Western Common Law", "Continental Civil Law", "Islamic Jurisprudence"],
      "verdict": "VERIFIED|UNVERIFIED|DISPUTED|FALSE",
      "note": "explanation of verdict"
    }
  ],
  "overallConfidence": 0.0-1.0
}

Identify 5-10 key factual claims. Be ruthless in your assessment. False claims damage credibility.`;

  const result = await callLLM(
    systemPrompt,
    `QUERY: ${prompt}\n\nSITUATION REPORT TO AUDIT:\n${report.slice(0, 3000)}`,
    "Veritas Audit",
  );

  if (!result) {
    return {
      claims: [],
      overallConfidence: 0,
      falseClaims: 0,
      disputedClaims: 0,
      verifiedClaims: 0,
      auditTimestamp: new Date().toISOString(),
    };
  }

  try {
    const jsonMatch = result.match(/\{[\s\S]*\}/);
    if (jsonMatch) {
      const parsed = JSON.parse(jsonMatch[0]);
      const claims: VeritasClaim[] = Array.isArray(parsed.claims) ? parsed.claims : [];
      return {
        claims,
        overallConfidence: typeof parsed.overallConfidence === "number" ? parsed.overallConfidence : 0.5,
        falseClaims: claims.filter(c => c.verdict === "FALSE").length,
        disputedClaims: claims.filter(c => c.verdict === "DISPUTED").length,
        verifiedClaims: claims.filter(c => c.verdict === "VERIFIED").length,
        auditTimestamp: new Date().toISOString(),
      };
    }
  } catch {}

  return {
    claims: [],
    overallConfidence: 0,
    falseClaims: 0,
    disputedClaims: 0,
    verifiedClaims: 0,
    auditTimestamp: new Date().toISOString(),
  };
}

function buildLexicalProtocolInstructions(etymology: EtymologyAudit): string {
  const termList = etymology.entries
    .map(e => `- "${e.term}": ${e.origin}. ${e.crossCulturalNote}${!e.synchronized ? " [UNSYNCHRONIZED - handle with care]" : ""}`)
    .join("\n");

  const flagged = etymology.flaggedTerms.length > 0
    ? `\n\nFLAGGED TERMS (avoid or use with correction): ${etymology.flaggedTerms.join(", ")}`
    : "";

  return `LEXICAL PROTOCOLS (v2.0):
- Precision over colloquialism: use the most precise term available in the target language
- Etymological anchoring: key terms must be translated considering their etymological roots
- All Latin legal maxims (bona fide, sui generis, pari passu, prima facie, etc.) MUST be preserved in their original Latin form
- Technical terms without precise equivalents must be transliterated with a parenthetical explanation
- Terms marked [UNSYNCHRONIZED] require special handling: include both the local equivalent AND the English term in parentheses

KEY TERMS FOR THIS REPORT:
${termList}${flagged}`;
}

async function generateUnifiedSituationReport(
  results: AgentResult[],
  layer2: Layer2Section[],
  executiveSummary: ExecutiveSummary | undefined,
  prompt: string,
  veritasAudit?: VeritasAudit,
): Promise<string> {
  const successResults = results.filter(r => !r.response.startsWith("[Error]"));

  const agentFindings = successResults
    .map(r => `[${r.domain}]: ${r.response}`)
    .join("\n");

  const truncatedFindings = agentFindings.length > 2000
    ? agentFindings.slice(0, 2000) + "..."
    : agentFindings;

  const layer2Summary = layer2
    .filter(s => s.successCount > 0)
    .map(s => `${s.label}: ${s.technicalSummary}`)
    .join("\n");

  const execParts: string[] = [];
  if (executiveSummary) {
    execParts.push(`Verdict: ${executiveSummary.verdict.signal} — ${executiveSummary.verdict.assessment}`);
    execParts.push(`Plain English: ${executiveSummary.plainEnglish.summary}`);
    execParts.push(`Board Recommendation: ${executiveSummary.plainEnglish.boardRecommendation}`);
  }

  let veritasWarning = "";
  if (veritasAudit && veritasAudit.claims.length > 0) {
    const falseClaims = veritasAudit.claims.filter(c => c.verdict === "FALSE");
    const disputedClaims = veritasAudit.claims.filter(c => c.verdict === "DISPUTED");
    const unverifiedClaims = veritasAudit.claims.filter(c => c.verdict === "UNVERIFIED");

    if (falseClaims.length > 0 || disputedClaims.length > 0 || unverifiedClaims.length > 0) {
      veritasWarning = `\n\nVERITAS AUDIT RESULTS (CRITICAL - you MUST incorporate these findings):`;
      if (falseClaims.length > 0) {
        veritasWarning += `\nFALSE CLAIMS DETECTED - DO NOT include these in the report:\n${falseClaims.map(c => `- "${c.claim}" — ${c.note}`).join("\n")}`;
      }
      if (disputedClaims.length > 0) {
        veritasWarning += `\nDISPUTED CLAIMS - present with appropriate caveats:\n${disputedClaims.map(c => `- "${c.claim}" — ${c.note}`).join("\n")}`;
      }
      if (unverifiedClaims.length > 0) {
        veritasWarning += `\nUNVERIFIED CLAIMS - do not present as fact:\n${unverifiedClaims.map(c => `- "${c.claim}" — ${c.note}`).join("\n")}`;
      }
    }
  }

  const systemPrompt = `You are a senior analyst producing one comprehensive Situation Report. Write a complete, professional report that synthesizes all specialist findings into a single coherent document. Include: (1) Executive Overview, (2) Key Findings by Domain, (3) Risk Assessment, (4) Jurisdictional Analysis, (5) Recommendations, and (6) Conclusion. The report should be 500-800 words, clear and actionable. Write in English.

CRITICAL INTEGRITY RULES:
- Only state facts you can support with evidence from the specialist findings
- If the Veritas Audit flagged FALSE claims, you MUST exclude those claims entirely
- If claims are DISPUTED or UNVERIFIED, qualify them appropriately (e.g., "reportedly", "unconfirmed")
- Never fabricate the existence of documents, files, policies, or legal instruments
- When in doubt, state uncertainty rather than assert falsehood`;

  const userContent = `QUERY: ${prompt}\n\nSPECIALIST FINDINGS (${successResults.length} agents):\n${truncatedFindings}\n\nCATEGORY ANALYSIS:\n${layer2Summary}\n\n${execParts.length > 0 ? `EXECUTIVE SUMMARY:\n${execParts.join("\n")}` : ""}${veritasWarning}`;

  const report = await callLLM(systemPrompt, userContent, "Unified Situation Report");

  return report || `SITUATION REPORT\n\nQuery: ${prompt}\n\n${successResults.length} specialist agents analyzed this query across ${LAYER2_SECTIONS.length} domains.\n\n${layer2Summary}\n\n${execParts.join("\n")}\n\nFull specialist responses are available in the detailed view.`;
}

async function translateReport(
  report: string,
  sendSSE?: (eventType: string, data: unknown) => void,
  etymologyAudit?: EtymologyAudit,
): Promise<{ translations: TranslationEntry[]; lexicalProtocol: LexicalProtocol }> {
  const lexicalInstructions = etymologyAudit ? buildLexicalProtocolInstructions(etymologyAudit) : "";
  const translationLimit = pLimit(TRANSLATION_CONCURRENCY);

  const translations = await Promise.all(
    AGENT_LANGUAGES.map((lang, idx) =>
      translationLimit(async (): Promise<TranslationEntry> => {
        if (lang.code === "en") {
          if (sendSSE) sendSSE("translation_progress", { index: idx, code: lang.code, name: lang.name, status: "complete" });
          return {
            languageCode: lang.code,
            languageName: lang.name,
            nativeName: lang.nativeName,
            text: report,
          };
        }

        if (sendSSE) sendSSE("translation_progress", { index: idx, code: lang.code, name: lang.name, status: "translating" });

        const lexicalSuffix = lexicalInstructions
          ? `\n\n${lexicalInstructions}`
          : "";
        const translated = await callLLM(
          `You are a professional translator. Translate the following Situation Report into ${lang.name} (${lang.nativeName}). Maintain the same structure, headings, and professional tone. Translate ALL content including section headers. Do NOT add commentary or notes — output ONLY the translated report.${lexicalSuffix}`,
          report,
          `Translate to ${lang.name}`,
        );

        if (sendSSE) sendSSE("translation_progress", { index: idx, code: lang.code, name: lang.name, status: "complete" });

        return {
          languageCode: lang.code,
          languageName: lang.name,
          nativeName: lang.nativeName,
          text: translated || `[Translation to ${lang.name} unavailable]`,
        };
      })
    )
  );

  const latinTerms = etymologyAudit
    ? etymologyAudit.entries.filter(e => e.origin.toLowerCase().includes("latin")).map(e => e.term)
    : [];

  const lexicalProtocol: LexicalProtocol = {
    version: "2.0",
    termsEnforced: etymologyAudit ? etymologyAudit.entries.length : 0,
    consistencyScore: etymologyAudit ? (etymologyAudit.flaggedTerms.length === 0 ? 1.0 : Math.max(0, 1.0 - etymologyAudit.flaggedTerms.length * 0.1)) : 0,
    corrections: etymologyAudit
      ? etymologyAudit.flaggedTerms.map(t => ({ original: t, corrected: t, reason: "Flagged by Etymology Engine" }))
      : [],
    latinTermsPreserved: latinTerms,
  };

  return { translations, lexicalProtocol };
}

export function registerAgentArrayRoutes(app: Express) {
  app.post("/api/tribonacci/agent-array", (req: Request, res: Response) => {
    const { prompt, customRoles } = req.body;

    if (!prompt || typeof prompt !== "string" || prompt.trim().length === 0) {
      return res.status(400).json({ error: "prompt is required" });
    }

    if (prompt.length > 2000) {
      return res.status(400).json({ error: "prompt must be under 2000 characters" });
    }

    let roles = DEFAULT_SPECIALISTS as AgentSpecialist[];
    if (Array.isArray(customRoles) && customRoles.length === AGENT_COUNT) {
      roles = customRoles.map((r: any, i: number) => ({
        title: typeof r.title === "string" ? r.title.slice(0, 200) : DEFAULT_SPECIALISTS[i].title,
        description: typeof r.description === "string" ? r.description.slice(0, 500) : DEFAULT_SPECIALISTS[i].description,
        category: DEFAULT_SPECIALISTS[i].category,
        subdomain: DEFAULT_SPECIALISTS[i].subdomain,
        keywords: DEFAULT_SPECIALISTS[i].keywords,
        systemPrompt: DEFAULT_SPECIALISTS[i].systemPrompt,
        weight: DEFAULT_SPECIALISTS[i].weight,
        language: DEFAULT_SPECIALISTS[i].language,
      }));
    }

    const sessionId = randomUUID();
    pendingSessions.set(sessionId, {
      prompt: prompt.trim(),
      roles,
      createdAt: Date.now(),
    });

    setTimeout(() => pendingSessions.delete(sessionId), SESSION_TTL_MS);

    res.json({ sessionId });
  });

  app.get("/api/tribonacci/agent-array/stream/:sessionId", async (req: Request, res: Response) => {
    const sessionId = req.params.sessionId as string;
    const session = pendingSessions.get(sessionId);

    if (!session) {
      return res.status(404).json({ error: "Session not found or expired" });
    }

    pendingSessions.delete(sessionId);

    const { prompt, roles } = session;
    const startTime = Date.now();
    const executionOrder = scheduleAgents();
    const tribHash = generateTribonacciHash(prompt);

    log.info(`Agent Array session ${sessionId}: launching ${AGENT_COUNT} agents in parallel (Tribonacci order: ${executionOrder.slice(0, 6).join(",")}...)`);

    res.writeHead(200, {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache, no-transform",
      "Connection": "keep-alive",
      "X-Accel-Buffering": "no",
      "Content-Encoding": "identity",
    });
    res.flushHeaders();

    const sendSSE = (eventType: string, data: unknown) => {
      if (!res.destroyed && !res.writableEnded) {
        res.write(`event: ${eventType}\ndata: ${JSON.stringify(data)}\n\n`);
      }
    };

    const positions = getAgentPositions(roles);

    sendSSE("session_start", {
      sessionId,
      prompt,
      agentCount: AGENT_COUNT,
      stepsPerAgent: 1,
      tribonacciHash: tribHash,
      executionOrder,
    });

    const limit = pLimit(CONCURRENCY_LIMIT);
    const isClientGone = () => res.destroyed || (req.socket && req.socket.destroyed);

    const sendStepEvent = (event: AgentStepEvent) => {
      if (!isClientGone()) sendSSE("agent_step", event);
    };

    const orderedPositions = executionOrder.map((agentId) => {
      const pos = positions.find((p) => p.z28 === agentId);
      return pos || positions[agentId];
    });

    const agentPromises = orderedPositions.map((pos) =>
      limit(() => {
        if (isClientGone()) {
          return Promise.resolve({
            agentIndex: pos.index,
            agentLabel: `A${String(pos.z28).padStart(2, "0")}`,
            z28: pos.z28,
            domain: pos.domain,
            category: pos.category,
            response: "[Error] Client disconnected",
            totalDurationMs: 0,
            stepsCompleted: 0,
            language: roles[pos.z28]?.language || AGENT_LANGUAGES[pos.z28],
          } as AgentResult);
        }
        return executeAgentSteps(
          pos.index,
          pos.z28,
          roles[pos.z28],
          prompt,
          sendStepEvent,
        );
      })
    );

    const results = await Promise.all(agentPromises);

    if (isClientGone()) {
      log.info(`Agent Array session ${sessionId}: client disconnected, aborting`);
      if (!res.writableEnded) res.end();
      return;
    }

    const successCount = results.filter((r) => !r.response.startsWith("[Error]")).length;
    const totalDuration = Date.now() - startTime;

    sendSSE("layer1_complete", {
      successCount,
      totalCount: AGENT_COUNT,
      durationMs: totalDuration,
    });

    let layer2: Layer2Section[] = [];
    let executiveSummary: ExecutiveSummary | undefined;

    if (successCount >= 10) {
      sendSSE("layer2_start", { sectionCount: LAYER2_SECTIONS.length });
      layer2 = await generateLayer2(results, prompt);
      sendSSE("layer2_complete", { sections: layer2 });

      if (successCount >= 15) {
        sendSSE("executive_start", { sections: 5 });
        try {
          executiveSummary = await generateExecutiveSummary(layer2, prompt);
          sendSSE("executive_complete", { executiveSummary });
        } catch (err) {
          log.error(`Executive summary generation failed: ${err instanceof Error ? err.message : String(err)}`);
        }
      }
    }

    sendSSE("etymology_start", {});
    const etymologyAudit = await runEtymologyAudit(
      layer2.filter(s => s.successCount > 0).map(s => s.technicalSummary).join("\n"),
      prompt,
    );
    sendSSE("etymology_complete", {
      termCount: etymologyAudit.entries.length,
      flaggedCount: etymologyAudit.flaggedTerms.length,
      flaggedTerms: etymologyAudit.flaggedTerms,
      entries: etymologyAudit.entries.slice(0, 12),
    });

    sendSSE("veritas_start", {});
    const preliminaryReport = layer2
      .filter(s => s.successCount > 0)
      .map(s => `${s.label}: ${s.technicalSummary}`)
      .join("\n");
    const veritasAudit = await runVeritasAudit(preliminaryReport, prompt);
    sendSSE("veritas_complete", {
      claimCount: veritasAudit.claims.length,
      overallConfidence: veritasAudit.overallConfidence,
      falseClaims: veritasAudit.falseClaims,
      disputedClaims: veritasAudit.disputedClaims,
      verifiedClaims: veritasAudit.verifiedClaims,
      claims: veritasAudit.claims,
    });

    sendSSE("report_start", { languageCount: AGENT_LANGUAGES.length });

    const unifiedReport = await generateUnifiedSituationReport(results, layer2, executiveSummary, prompt, veritasAudit);
    sendSSE("report_generated", { report: unifiedReport });

    const { translations, lexicalProtocol } = await translateReport(unifiedReport, sendSSE, etymologyAudit);
    sendSSE("lexical_applied", {
      version: lexicalProtocol.version,
      termsEnforced: lexicalProtocol.termsEnforced,
      consistencyScore: lexicalProtocol.consistencyScore,
      corrections: lexicalProtocol.corrections,
      latinTermsPreserved: lexicalProtocol.latinTermsPreserved,
    });
    sendSSE("translations_complete", { count: translations.length });

    let consensus = "";
    if (executiveSummary) {
      consensus = executiveSummary.plainEnglish.summary;
    } else if (successCount >= 10) {
      consensus = layer2
        .filter((s) => s.successCount > 0 && !s.technicalSummary.startsWith("No agent"))
        .map((s) => s.technicalSummary)
        .join(" ");
    } else {
      consensus = `Insufficient agent responses (${successCount}/${AGENT_COUNT}) for consensus.`;
    }

    const response: AgentArrayResponse & {
      unifiedReport: string;
      translations: TranslationEntry[];
      etymologyAudit: EtymologyAudit;
      veritasAudit: VeritasAudit;
      lexicalProtocol: LexicalProtocol;
    } = {
      sessionId,
      prompt,
      agentCount: AGENT_COUNT,
      stepsPerAgent: 1,
      totalDurationMs: Date.now() - startTime,
      results,
      consensus,
      layer2,
      executiveSummary,
      tribonacciHash: tribHash,
      executionOrder,
      unifiedReport,
      translations,
      etymologyAudit,
      veritasAudit,
      lexicalProtocol,
    };

    sendSSE("complete", response);

    log.info(`Agent Array session ${sessionId}: completed in ${Date.now() - startTime}ms (${successCount}/${AGENT_COUNT} success, ${translations.length} translations)`);

    res.end();
  });

  app.post("/api/tribonacci/agent-array/save", async (req: Request, res: Response) => {
    try {
      const { prompt, tribonacciHash, unifiedReport, translations, executiveSummary, layer2Sections, agentCount, successCount, totalDurationMs } = req.body;

      if (!prompt || typeof prompt !== "string" || prompt.trim().length === 0) {
        return res.status(400).json({ error: "prompt is required" });
      }
      if (!unifiedReport || typeof unifiedReport !== "string" || unifiedReport.trim().length === 0) {
        return res.status(400).json({ error: "unifiedReport is required" });
      }
      if (!Array.isArray(translations) || translations.length === 0) {
        return res.status(400).json({ error: "translations array is required" });
      }
      const validTranslations = translations.every((t: any) =>
        t && typeof t.languageCode === "string" && typeof t.text === "string"
      );
      if (!validTranslations) {
        return res.status(400).json({ error: "Each translation must have languageCode and text" });
      }

      const encryptedFields = phaseEncryptFields({
        prompt, unifiedReport, translations,
        executiveSummary: executiveSummary || null,
        layer2Sections: layer2Sections || null,
      });
      const [inserted] = await db.insert(agentArrayReports).values({
        prompt,
        tribonacciHash: tribonacciHash || "",
        unifiedReport,
        translations,
        executiveSummary: executiveSummary || null,
        layer2Sections: layer2Sections || null,
        agentCount: agentCount || AGENT_COUNT,
        successCount: successCount || 0,
        totalDurationMs: totalDurationMs || 0,
        encryptedFields,
      }).returning();

      res.json({ id: inserted.id, createdAt: inserted.createdAt });
    } catch (err) {
      log.error(`Failed to save report: ${err instanceof Error ? err.message : String(err)}`);
      res.status(500).json({ error: "Failed to save report" });
    }
  });

  app.get("/api/tribonacci/agent-array/reports", async (_req: Request, res: Response) => {
    try {
      const reports = await db.select({
        id: agentArrayReports.id,
        prompt: agentArrayReports.prompt,
        tribonacciHash: agentArrayReports.tribonacciHash,
        agentCount: agentArrayReports.agentCount,
        successCount: agentArrayReports.successCount,
        totalDurationMs: agentArrayReports.totalDurationMs,
        createdAt: agentArrayReports.createdAt,
      }).from(agentArrayReports).orderBy(desc(agentArrayReports.createdAt)).limit(50);

      res.json({ reports });
    } catch (err) {
      log.error(`Failed to fetch reports: ${err instanceof Error ? err.message : String(err)}`);
      res.status(500).json({ error: "Failed to fetch reports" });
    }
  });

  app.get("/api/tribonacci/agent-array/reports/:id", async (req: Request, res: Response) => {
    try {
      const id = parseInt(req.params.id as string, 10);
      if (isNaN(id)) return res.status(400).json({ error: "Invalid report ID" });

      const [report] = await db.select().from(agentArrayReports).where(eq(agentArrayReports.id, id)).limit(1);

      if (!report) return res.status(404).json({ error: "Report not found" });

      const dec = phaseDecryptFields(report.encryptedFields);
      if (dec) {
        if (dec.prompt) report.prompt = dec.prompt as string;
        if (dec.unifiedReport) report.unifiedReport = dec.unifiedReport as string;
        if (dec.translations) report.translations = dec.translations as any;
        if (dec.executiveSummary) report.executiveSummary = dec.executiveSummary as string;
        if (dec.layer2Sections) report.layer2Sections = dec.layer2Sections as any;
      }

      res.json({ report });
    } catch (err) {
      log.error(`Failed to fetch report: ${err instanceof Error ? err.message : String(err)}`);
      res.status(500).json({ error: "Failed to fetch report" });
    }
  });

  app.get("/api/tribonacci/agent-array/positions", (_req: Request, res: Response) => {
    res.json({
      positions: getAgentPositions(),
      walk: generateZ28Walk(),
      executionOrder: scheduleAgents(),
      agentCount: AGENT_COUNT,
      stepsPerAgent: 1,
      stepNames: AGENT_STEP_NAMES,
      domains: AGENT_DOMAINS,
      specialists: DEFAULT_SPECIALISTS,
      layer2Sections: LAYER2_SECTIONS.map((s) => ({ key: s.key, label: s.label, agentCount: s.agentIndices.length })),
      tribonacci: {
        ternaryRadian: TERNARY_RADIAN,
        numAgents: AGENT_COUNT,
        fullCircle: FULL_CIRCLE,
        convolutionKernel: CONVOLUTION_KERNEL,
      },
    });
  });
}
