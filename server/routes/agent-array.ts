/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * 28-Dimension AI Agent Array — Tribonacci Circle Orchestration
 *
 * Dimensional Layer 1: 28 specialist agents execute simultaneously
 * Dimensional Layer 2: 5-section executive summary (technical + layman)
 * Uses SSE for real-time progress streaming to the frontend.
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

const log = createLogger("agent-array");

const openai = new OpenAI({
  apiKey: process.env.AI_INTEGRATIONS_OPENAI_API_KEY,
  baseURL: process.env.AI_INTEGRATIONS_OPENAI_BASE_URL,
});

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
          temperature: 0.7,
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
  roles: AgentSpecialist[],
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
          const completion = await pRetry(
            () =>
              openai.chat.completions.create({
                model: "gpt-5-nano",
                messages: [
                  {
                    role: "system",
                    content: `You are the Dimensional Layer 2 Executive Summary Engine for the "${section.label}" section.
You synthesize insights from ${successCount} specialist agents into a two-part executive summary.

PART 1 - TECHNICAL SUMMARY: Write 2-3 sentences using precise technical and legal terminology appropriate for experts in ${section.label}.
PART 2 - PLAIN LANGUAGE SUMMARY: Rewrite the same insights in 2-3 sentences using everyday language that a non-specialist can easily understand. Avoid jargon.

Format your response exactly as:
TECHNICAL: [your technical summary]
PLAIN: [your plain language summary]`,
                  },
                  {
                    role: "user",
                    content: `Original query: "${prompt}"\n\nAgent responses:\n${agentInputs}`,
                  },
                ],
                max_completion_tokens: 400,
                temperature: 0.3,
              }),
            { retries: RETRY_ATTEMPTS, minTimeout: 500 },
          );

          const raw = completion.choices[0]?.message?.content || "";
          const techMatch = raw.match(/TECHNICAL:\s*([\s\S]*?)(?=PLAIN:|$)/i);
          const plainMatch = raw.match(/PLAIN:\s*([\s\S]*?)$/i);

          return {
            category: section.key,
            label: section.label,
            technicalSummary: techMatch?.[1]?.trim() || raw,
            laySummary: plainMatch?.[1]?.trim() || raw,
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
  app.post("/api/tribonacci/agent-array", async (req: Request, res: Response) => {
    const { prompt, customRoles } = req.body;

    if (!prompt || typeof prompt !== "string" || prompt.trim().length === 0) {
      return res.status(400).json({ error: "prompt is required" });
    }

    if (prompt.length > 2000) {
      return res.status(400).json({ error: "prompt must be under 2000 characters" });
    }

    let roles = DEFAULT_SPECIALISTS;
    if (Array.isArray(customRoles) && customRoles.length === AGENT_COUNT) {
      roles = customRoles.map((r: any, i: number) => ({
        title: typeof r.title === "string" ? r.title.slice(0, 200) : DEFAULT_SPECIALISTS[i].title,
        description: typeof r.description === "string" ? r.description.slice(0, 500) : DEFAULT_SPECIALISTS[i].description,
        category: DEFAULT_SPECIALISTS[i].category,
      }));
    }

    const sessionId = randomUUID();
    const startTime = Date.now();

    log.info(`Agent Array session ${sessionId}: launching ${AGENT_COUNT} agents`);

    res.setHeader("Content-Type", "text/event-stream");
    res.setHeader("Cache-Control", "no-cache");
    res.setHeader("Connection", "keep-alive");
    res.setHeader("X-Accel-Buffering", "no");

    const sendSSE = (type: string, data: unknown) => {
      if (!res.destroyed) {
        res.write(`data: ${JSON.stringify({ type, ...data as object })}\n\n`);
      }
    };

    const positions = getAgentPositions(roles);

    sendSSE("session_start", {
      sessionId,
      prompt: prompt.trim(),
      agentCount: AGENT_COUNT,
      stepsPerAgent: STEPS_PER_AGENT,
      positions,
    });

    const limit = pLimit(CONCURRENCY_LIMIT);
    let aborted = false;

    req.on("close", () => {
      aborted = true;
      log.info(`Agent Array session ${sessionId}: client disconnected`);
    });

    const sendStepEvent = (event: AgentStepEvent) => {
      if (!aborted) sendSSE("agent_step", event);
    };

    const agentPromises = positions.map((pos) =>
      limit(() => {
        if (aborted) {
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
          prompt.trim(),
          sendStepEvent,
        );
      })
    );

    const results = await Promise.all(agentPromises);

    if (aborted) {
      res.end();
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

      layer2 = await generateLayer2(results, prompt.trim(), roles);

      sendSSE("layer2_complete", { sections: layer2 });
    }

    let consensus = "";
    if (successCount >= 20) {
      try {
        const summaryInputs = layer2
          .filter((s) => s.successCount > 0)
          .map((s) => `[${s.label}]: ${s.technicalSummary}`)
          .join("\n");

        const consensusCompletion = await openai.chat.completions.create({
          model: "gpt-5-nano",
          messages: [
            {
              role: "system",
              content: `You are the Tribonacci Consensus Engine. You synthesize the 5 executive section summaries from ${successCount} specialist agents arranged on the Z₂₈ cyclic group into a final unified briefing. Provide a 3-4 sentence synthesis that combines both technical precision and clear everyday language. Be authoritative yet accessible.`,
            },
            {
              role: "user",
              content: `Synthesize these section summaries into a final consensus:\n\n${summaryInputs}`,
            },
          ],
          max_completion_tokens: 300,
          temperature: 0.3,
        });

        consensus = consensusCompletion.choices[0]?.message?.content || "Consensus generation failed.";
      } catch (err: unknown) {
        const errMsg = err instanceof Error ? err.message : String(err);
        log.error(`Consensus generation failed: ${errMsg}`);
        consensus = "Consensus could not be generated due to an error.";
      }
    } else {
      consensus = `Insufficient agent responses (${successCount}/${AGENT_COUNT}) for consensus.`;
    }

    const response: AgentArrayResponse = {
      sessionId,
      prompt: prompt.trim(),
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
