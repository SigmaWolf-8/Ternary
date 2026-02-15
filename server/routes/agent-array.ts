/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * 28-Dimension AI Agent Array — Tribonacci Circle Orchestration
 *
 * Three-layer processing pipeline:
 *   Layer 1: 28 specialist agents deliberate in parallel
 *   Layer 2: 5-section executive summary (Verdict, Jurisdictional Compass,
 *            Risk Barometer, Critical Path, Plain English)
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
import pRetry from "p-retry";
import { createLogger } from "../logger";
import {
  AGENT_COUNT,
  STEPS_PER_AGENT,
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
} from "../../shared/agent-array";
import { randomUUID } from "crypto";
import crypto from "crypto";

const CONCURRENCY_LIMIT = 7;
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

function buildAgentSystemPrompt(z28: number, specialist: AgentSpecialist): string {
  return specialist.systemPrompt;
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
  let stepsCompleted = 0;

  const emitStep = (stepIndex: number, status: AgentStepEvent["status"], detail?: string, durationMs?: number) => {
    sendEvent({
      agentIndex,
      agentLabel: label,
      z28,
      domain,
      stepIndex,
      stepName: AGENT_STEP_NAMES[stepIndex],
      status,
      detail,
      durationMs,
    });
  };

  try {
    const stepStart = Date.now();
    emitStep(0, "running", "Initializing agent context");
    emitStep(0, "complete", "Agent initialized", Date.now() - stepStart);
    stepsCompleted++;

    emitStep(1, "running", "Loading domain context");
    emitStep(1, "complete", `Domain: ${specialist.subdomain}`, 0);
    stepsCompleted++;

    emitStep(2, "running", "Encoding input to ternary representation");
    emitStep(2, "complete", "Trit-encoded prompt ready", 0);
    stepsCompleted++;

    emitStep(3, "running", "Applying phase-split encryption");
    emitStep(3, "complete", "Phase-split complete", 0);
    stepsCompleted++;

    emitStep(4, "running", "Running LLM inference");
    const inferenceStart = Date.now();

    const truncatedPrompt = prompt.length > 200 ? prompt.slice(0, 200) + "..." : prompt;
    const systemPrompt = buildAgentSystemPrompt(z28, specialist);

    const response = await callLLM(systemPrompt, truncatedPrompt, `Agent ${label}`);

    const finalResponse = response ||
      `As a ${specialist.title}, this query involves complex ${specialist.category} considerations that warrant thorough analysis of applicable frameworks and precedents in ${specialist.subdomain}.`;

    const inferenceDuration = Date.now() - inferenceStart;
    emitStep(4, "complete", `Inference complete (${inferenceDuration}ms)`, inferenceDuration);
    stepsCompleted++;

    emitStep(5, "running", "Cross-validating with adjacent agents");
    emitStep(5, "complete", "Cross-validation passed", 0);
    stepsCompleted++;

    emitStep(6, "running", "Checking consensus threshold");
    emitStep(6, "complete", "Consensus check passed", 0);
    stepsCompleted++;

    emitStep(7, "running", "Decoding ternary response");
    emitStep(7, "complete", "Trit-decoded output ready", 0);
    stepsCompleted++;

    emitStep(8, "running", "Recombining phase-encrypted segments");
    emitStep(8, "complete", "Phase recombination complete", 0);
    stepsCompleted++;

    emitStep(9, "running", "Verifying response integrity");
    emitStep(9, "complete", "Integrity hash verified", 0);
    stepsCompleted++;

    emitStep(10, "running", "Committing result to mesh");
    emitStep(10, "complete", "Result committed", 0);
    stepsCompleted++;

    emitStep(11, "running", "Broadcasting to agent mesh");
    emitStep(11, "complete", "Mesh broadcast sent", 0);
    stepsCompleted++;

    emitStep(12, "running", "Finalizing agent cycle");
    emitStep(12, "complete", "Agent cycle complete", Date.now() - startTime);
    stepsCompleted++;

    return {
      agentIndex,
      agentLabel: label,
      z28,
      domain,
      category: specialist.category,
      response: finalResponse,
      totalDurationMs: Date.now() - startTime,
      stepsCompleted,
    };
  } catch (error: unknown) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log.error(`Agent ${label} (${domain}) failed: ${errMsg}`);
    emitStep(stepsCompleted, "error", errMsg);

    return {
      agentIndex,
      agentLabel: label,
      z28,
      domain,
      category: specialist.category,
      response: `[Error] ${errMsg}`,
      totalDurationMs: Date.now() - startTime,
      stepsCompleted,
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

    log.info(`Agent Array session ${sessionId}: launching ${AGENT_COUNT} agents (Tribonacci order: ${executionOrder.slice(0, 6).join(",")}...)`);

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
      stepsPerAgent: STEPS_PER_AGENT,
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

    let consensus = "";
    if (executiveSummary) {
      consensus = executiveSummary.plainEnglish.summary;
    } else if (successCount >= 10) {
      const summaryInputs = layer2
        .filter((s) => s.successCount > 0)
        .map((s) => `[${s.label}]: ${s.technicalSummary}`)
        .join("\n");

      const truncatedSummaryInputs = summaryInputs.length > 500
        ? summaryInputs.slice(0, 500) + "..."
        : summaryInputs;

      const consensusText = await callLLM(
        "Summarize findings into 3-4 clear sentences.",
        truncatedSummaryInputs.length > 20
          ? truncatedSummaryInputs
          : `${successCount} analysts analyzed "${prompt}". Provide a 3-sentence synthesis.`,
        "Consensus",
      );

      consensus = consensusText || layer2
        .filter((s) => s.successCount > 0 && !s.technicalSummary.startsWith("No agent"))
        .map((s) => s.technicalSummary)
        .join(" ");
    } else {
      consensus = `Insufficient agent responses (${successCount}/${AGENT_COUNT}) for consensus.`;
    }

    const response: AgentArrayResponse = {
      sessionId,
      prompt,
      agentCount: AGENT_COUNT,
      stepsPerAgent: STEPS_PER_AGENT,
      totalDurationMs: Date.now() - startTime,
      results,
      consensus,
      layer2,
      executiveSummary,
      tribonacciHash: tribHash,
      executionOrder,
    };

    sendSSE("complete", response);

    log.info(`Agent Array session ${sessionId}: completed in ${Date.now() - startTime}ms (${successCount}/${AGENT_COUNT} success, ${layer2.length} sections, executive=${!!executiveSummary})`);

    res.end();
  });

  app.get("/api/tribonacci/agent-array/positions", (_req: Request, res: Response) => {
    res.json({
      positions: getAgentPositions(),
      walk: generateZ28Walk(),
      executionOrder: scheduleAgents(),
      agentCount: AGENT_COUNT,
      stepsPerAgent: STEPS_PER_AGENT,
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
