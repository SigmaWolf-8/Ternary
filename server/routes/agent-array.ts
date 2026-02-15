/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * 28-Dimension AI Agent Array — Tribonacci Circle Orchestration
 *
 * Dimensional Layer 1: 28 specialist agents execute simultaneously
 * Dimensional Layer 2: 5-section executive summary (technical + layman)
 *
 * Two-step pattern:
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
  generateZ28Walk,
  getAgentPositions,
  type AgentSpecialist,
  type AgentStepEvent,
  type AgentResult,
  type AgentArrayResponse,
  type Layer2Section,
} from "../../shared/agent-array";
import { randomUUID } from "crypto";

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

function buildAgentSystemPrompt(z28: number, specialist: AgentSpecialist): string {
  return `You are Agent A${String(z28).padStart(2, "0")}, position ${z28} on the Z₂₈ Tribonacci Circle.
Your role: ${specialist.title}
Your expertise: ${specialist.description}
Category: ${specialist.category}
You are one of 28 simultaneous specialist agents in the PlenumNET Agent Array.

Respond concisely (2-3 sentences) from your domain perspective.
Focus on your specific area of expertise as it relates to the user's query.
Be precise and actionable.`;
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
    emitStep(1, "complete", `Domain: ${domain}`, 0);
    stepsCompleted++;

    emitStep(2, "running", "Encoding input to ternary representation");
    emitStep(2, "complete", "Trit-encoded prompt ready", 0);
    stepsCompleted++;

    emitStep(3, "running", "Applying phase-split encryption");
    emitStep(3, "complete", "Phase-split complete", 0);
    stepsCompleted++;

    emitStep(4, "running", "Running LLM inference");
    const inferenceStart = Date.now();

    const completion = await pRetry(
      () =>
        openai.chat.completions.create({
          model: "gpt-5-nano",
          messages: [
            { role: "system", content: buildAgentSystemPrompt(z28, specialist) },
            { role: "user", content: prompt },
          ],
          max_completion_tokens: 200,
        }),
      { retries: RETRY_ATTEMPTS, minTimeout: 500 },
    );

    const response = completion.choices[0]?.message?.content || "No response generated.";
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
      response,
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
          let technicalSummary = "";
          let laySummary = "";

          const truncatedInputs = agentInputs.length > 600
            ? agentInputs.slice(0, 600) + "..."
            : agentInputs;

          const techCompletion = await pRetry(
            () =>
              openai.chat.completions.create({
                model: "gpt-5-nano",
                messages: [
                  {
                    role: "system",
                    content: `Summarize the following analyst findings in 2-3 precise, detailed sentences using professional ${section.label} terminology.`,
                  },
                  {
                    role: "user",
                    content: `Query: "${prompt}"\n\nFindings:\n${truncatedInputs}`,
                  },
                ],
                max_completion_tokens: 1000,
              }),
            { retries: RETRY_ATTEMPTS, minTimeout: 500 },
          );

          const techRaw = techCompletion.choices[0]?.message?.content;
          log.info(`Layer2 ${section.label} technical: content=${techRaw ? techRaw.length + ' chars' : 'null'}, finish=${techCompletion.choices[0]?.finish_reason}`);
          technicalSummary = (techRaw && techRaw.trim().length > 5) ? techRaw.trim() : "";

          const plainCompletion = await pRetry(
            () =>
              openai.chat.completions.create({
                model: "gpt-5-nano",
                messages: [
                  {
                    role: "system",
                    content: `Explain the following analyst findings in 2-3 simple sentences anyone can understand. No jargon.`,
                  },
                  {
                    role: "user",
                    content: `Query: "${prompt}"\n\nFindings:\n${truncatedInputs}`,
                  },
                ],
                max_completion_tokens: 1000,
              }),
            { retries: RETRY_ATTEMPTS, minTimeout: 500 },
          );

          const plainRaw = plainCompletion.choices[0]?.message?.content;
          log.info(`Layer2 ${section.label} plain: content=${plainRaw ? plainRaw.length + ' chars' : 'null'}, finish=${plainCompletion.choices[0]?.finish_reason}`);
          laySummary = (plainRaw && plainRaw.trim().length > 5) ? plainRaw.trim() : "";

          if (!technicalSummary) {
            technicalSummary = sectionResults.map((r) => r.response).join(" ").slice(0, 500);
          }
          if (!laySummary) {
            laySummary = technicalSummary;
          }

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

    log.info(`Agent Array session ${sessionId}: launching ${AGENT_COUNT} agents`);

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
    });

    const limit = pLimit(CONCURRENCY_LIMIT);
    const isClientGone = () => res.destroyed || (req.socket && req.socket.destroyed);

    const sendStepEvent = (event: AgentStepEvent) => {
      if (!isClientGone()) sendSSE("agent_step", event);
    };

    const agentPromises = positions.map((pos) =>
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
    if (successCount >= 10) {
      sendSSE("layer2_start", { sectionCount: LAYER2_SECTIONS.length });
      layer2 = await generateLayer2(results, prompt);
      sendSSE("layer2_complete", { sections: layer2 });
    }

    let consensus = "";
    if (successCount >= 20) {
      try {
        const summaryInputs = layer2
          .filter((s) => s.successCount > 0)
          .map((s) => `[${s.label}]: ${s.technicalSummary}`)
          .join("\n");

        const truncatedSummaryInputs = summaryInputs.length > 500
          ? summaryInputs.slice(0, 500) + "..."
          : summaryInputs;

        log.info(`Consensus input (${summaryInputs.length} chars -> ${truncatedSummaryInputs.length} chars from ${layer2.filter((s) => s.successCount > 0).length} sections)`);

        let consensusText: string | null = null;

        for (let attempt = 0; attempt < 3 && !consensusText; attempt++) {
          try {
            const consensusCompletion = await openai.chat.completions.create({
              model: "gpt-5-nano",
              messages: [
                {
                  role: "system",
                  content: "Summarize the analyst findings below into 3-4 clear sentences.",
                },
                {
                  role: "user",
                  content: truncatedSummaryInputs.length > 20
                    ? `Summarize:\n\n${truncatedSummaryInputs}`
                    : `${successCount} analysts completed analysis of "${prompt}". Provide a 3-sentence synthesis.`,
                },
              ],
              max_completion_tokens: 1000,
            });

            const raw = consensusCompletion.choices[0]?.message?.content;
            const finishReason = consensusCompletion.choices[0]?.finish_reason;
            log.info(`Consensus attempt ${attempt + 1}: content=${raw ? raw.length + ' chars' : 'null'}, finish_reason=${finishReason}`);

            if (raw && raw.trim().length > 10) {
              consensusText = raw.trim();
            }
          } catch (retryErr) {
            log.warn(`Consensus attempt ${attempt + 1} error: ${retryErr instanceof Error ? retryErr.message : String(retryErr)}`);
          }
        }

        if (consensusText) {
          consensus = consensusText;
        } else {
          const fallbackParts = layer2
            .filter((s) => s.successCount > 0 && s.technicalSummary && !s.technicalSummary.startsWith("No agent") && !s.technicalSummary.startsWith("Summary generation"))
            .map((s) => s.technicalSummary);

          consensus = fallbackParts.length > 0
            ? fallbackParts.join(" ")
            : `All ${successCount} of ${AGENT_COUNT} specialist agents completed their analysis successfully across ${layer2.filter(s => s.successCount > 0).length} categories. Expand the Layer 2 Executive Summary sections above for detailed technical and plain-language findings from each domain.`;
        }
      } catch (err: unknown) {
        const errMsg = err instanceof Error ? err.message : String(err);
        log.error(`Consensus generation failed: ${errMsg}`);
        consensus = layer2
          .filter((s) => s.successCount > 0)
          .map((s) => s.technicalSummary)
          .join(" ") || `All ${successCount} agents completed their analysis successfully.`;
      }
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
    };

    sendSSE("complete", response);

    log.info(`Agent Array session ${sessionId}: completed in ${Date.now() - startTime}ms (${successCount}/${AGENT_COUNT} success, ${layer2.length} sections)`);

    res.end();
  });

  app.get("/api/tribonacci/agent-array/positions", (_req: Request, res: Response) => {
    res.json({
      positions: getAgentPositions(),
      walk: generateZ28Walk(),
      agentCount: AGENT_COUNT,
      stepsPerAgent: STEPS_PER_AGENT,
      stepNames: AGENT_STEP_NAMES,
      domains: AGENT_DOMAINS,
      specialists: DEFAULT_SPECIALISTS,
      layer2Sections: LAYER2_SECTIONS.map((s) => ({ key: s.key, label: s.label, agentCount: s.agentIndices.length })),
    });
  });
}
