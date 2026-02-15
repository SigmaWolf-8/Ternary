/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Tribonacci 28-Dimension AI Agent Array
 *
 * Maps the 28 positions of the Z₂₈ cyclic group (Tribonacci Circle)
 * to a parallel AI agent orchestration system. Each of the 28 agents
 * follows a 13-step execution model (1 ternary radian = T₇ = 13).
 *
 * Generator 13 visits all 28 positions: 0 → 13 → 26 → 11 → 24 → 9 → ...
 * This coprime walk ensures complete coverage with no collisions.
 *
 * Dimensional Layer 1: 28 specialist agents execute simultaneously
 * Dimensional Layer 2: 5-section executive summary synthesis
 */

export const AGENT_COUNT = 28;
export const STEPS_PER_AGENT = 13;

export const AGENT_STEP_NAMES = [
  "INIT",
  "CONTEXT_LOAD",
  "TRIT_ENCODE",
  "PHASE_SPLIT",
  "INFERENCE",
  "CROSS_VALIDATE",
  "CONSENSUS_CHECK",
  "TRIT_DECODE",
  "PHASE_RECOMBINE",
  "INTEGRITY_VERIFY",
  "RESULT_COMMIT",
  "MESH_BROADCAST",
  "FINALIZE",
] as const;

export type AgentStepName = typeof AGENT_STEP_NAMES[number];

export interface AgentSpecialist {
  title: string;
  description: string;
  category: AgentCategory;
}

export type AgentCategory =
  | "International Law"
  | "Regional Legal Systems"
  | "Finance"
  | "Crypto"
  | "Security";

export const LAYER2_SECTIONS: { key: AgentCategory; label: string; agentIndices: number[] }[] = [
  { key: "International Law", label: "International Law", agentIndices: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14] },
  { key: "Regional Legal Systems", label: "Regional Legal Systems", agentIndices: [15,16,17,18,19] },
  { key: "Finance", label: "Finance", agentIndices: [20,21,22] },
  { key: "Crypto", label: "Crypto", agentIndices: [23,24] },
  { key: "Security", label: "Security", agentIndices: [25,26,27] },
];

export const DEFAULT_SPECIALISTS: AgentSpecialist[] = [
  { title: "Public International Law Specialist", description: "Expert in treaties, state sovereignty, international organizations, and customary international law.", category: "International Law" },
  { title: "Private International Law Specialist", description: "Focuses on conflict of laws, jurisdiction, and choice of law in cross-border disputes.", category: "International Law" },
  { title: "International Trade Law Expert", description: "Specializes in WTO rules, regional trade agreements, tariffs, and trade remedies.", category: "International Law" },
  { title: "International Investment Law Expert", description: "Deals with bilateral investment treaties, investor-state arbitration, and foreign direct investment protections.", category: "International Law" },
  { title: "International Tax Law Specialist", description: "Master of cross-border taxation, transfer pricing, tax treaties, and base erosion strategies.", category: "International Law" },
  { title: "International IP Law Expert", description: "Covers global patents, trademarks, copyrights, and trade secrets.", category: "International Law" },
  { title: "International Environmental Law Expert", description: "Addresses climate change treaties, biodiversity, transboundary pollution, and sustainable development.", category: "International Law" },
  { title: "International Human Rights Law Specialist", description: "Focuses on human rights treaties, UN mechanisms, and corporate social responsibility.", category: "International Law" },
  { title: "International Criminal & Humanitarian Law Expert", description: "Handles war crimes, crimes against humanity, ICC proceedings, and Geneva Conventions.", category: "International Law" },
  { title: "International Maritime & Aviation Law Specialist", description: "Expert in law of the sea, shipping regulations, air transport, and liability.", category: "International Law" },
  { title: "International Space & Cyber Law Expert", description: "Covers outer space treaties, cyber norms, digital sovereignty, and emerging tech governance.", category: "International Law" },
  { title: "International Energy & Natural Resources Law", description: "Deals with oil, gas, mining, renewables, and cross-border energy projects.", category: "International Law" },
  { title: "International Labor & Employment Law Expert", description: "Focuses on global labor standards, migrant workers, and cross-border employment issues.", category: "International Law" },
  { title: "International Competition/Antitrust Law Specialist", description: "Handles cross-border mergers, antitrust investigations, and cartel enforcement.", category: "International Law" },
  { title: "International Dispute Resolution Expert", description: "Specializes in international arbitration, mediation, and litigation across jurisdictions.", category: "International Law" },
  { title: "Common Law Systems Expert", description: "Deep knowledge of US, UK, Canada, Australia, and other common law jurisdictions.", category: "Regional Legal Systems" },
  { title: "Civil Law Systems Expert", description: "Master of European, Latin American, and other civil law systems (e.g., France, Germany, Brazil).", category: "Regional Legal Systems" },
  { title: "Asian Legal Systems Expert", description: "Focuses on China, Japan, Korea, and Southeast Asian legal frameworks.", category: "Regional Legal Systems" },
  { title: "Islamic Law Expert", description: "Specializes in Sharia, Islamic finance, and family law in Muslim-majority countries.", category: "Regional Legal Systems" },
  { title: "African Legal Systems Expert", description: "Knowledge of customary law, regional bodies (AU, ECOWAS), and diverse national laws.", category: "Regional Legal Systems" },
  { title: "International Finance & Banking Specialist", description: "Expert in central banking, monetary policy, Basel regulations, and global financial markets.", category: "Finance" },
  { title: "Corporate Finance & Investment Banking Expert", description: "Handles M&A, capital markets, private equity, and cross-border transactions.", category: "Finance" },
  { title: "Financial Crime & AML Specialist", description: "Focuses on AML, counter-terrorism financing, fraud detection, and regulatory compliance.", category: "Finance" },
  { title: "Blockchain & Cryptocurrency Expert", description: "Technical master of consensus mechanisms, smart contracts, DeFi, and blockchain architecture.", category: "Crypto" },
  { title: "Crypto Legal & Regulatory Specialist", description: "Expert in legal status of cryptocurrencies, securities laws, compliance, and tax implications.", category: "Crypto" },
  { title: "Cybersecurity Technical Expert", description: "Specialist in network security, threat intelligence, incident response, and encryption.", category: "Security" },
  { title: "National Security & Geopolitical Risk Analyst", description: "Focuses on sanctions, export controls, geopolitical trends, and risk assessment.", category: "Security" },
  { title: "Data Privacy & Protection Specialist", description: "Expert in GDPR, CCPA, cross-border data flows, and privacy-by-design.", category: "Security" },
];

export const AGENT_DOMAINS = DEFAULT_SPECIALISTS.map((s) => s.title);

export interface AgentPosition {
  index: number;
  z28: number;
  angleDeg: number;
  label: string;
  domain: string;
  description: string;
  category: AgentCategory;
}

export function generateZ28Walk(): number[] {
  const walk: number[] = [];
  let pos = 0;
  for (let i = 0; i < AGENT_COUNT; i++) {
    walk.push(pos);
    pos = (pos + 13) % 28;
  }
  return walk;
}

export function getAgentPositions(customRoles?: AgentSpecialist[]): AgentPosition[] {
  const roles = customRoles || DEFAULT_SPECIALISTS;
  const walk = generateZ28Walk();
  return walk.map((z28, index) => ({
    index,
    z28,
    angleDeg: z28 * 13,
    label: `A${String(z28).padStart(2, "0")}`,
    domain: roles[z28]?.title || DEFAULT_SPECIALISTS[z28].title,
    description: roles[z28]?.description || DEFAULT_SPECIALISTS[z28].description,
    category: roles[z28]?.category || DEFAULT_SPECIALISTS[z28].category,
  }));
}

export type AgentStepStatus = "pending" | "running" | "complete" | "error";

export interface AgentStepEvent {
  agentIndex: number;
  agentLabel: string;
  z28: number;
  domain: string;
  stepIndex: number;
  stepName: AgentStepName;
  status: AgentStepStatus;
  detail?: string;
  durationMs?: number;
}

export interface AgentResult {
  agentIndex: number;
  agentLabel: string;
  z28: number;
  domain: string;
  category: AgentCategory;
  response: string;
  totalDurationMs: number;
  stepsCompleted: number;
}

export interface Layer2Section {
  category: AgentCategory;
  label: string;
  technicalSummary: string;
  laySummary: string;
  agentCount: number;
  successCount: number;
}

export interface AgentArrayRequest {
  prompt: string;
  model?: string;
  customRoles?: AgentSpecialist[];
}

export interface AgentArrayResponse {
  sessionId: string;
  prompt: string;
  agentCount: number;
  stepsPerAgent: number;
  totalDurationMs: number;
  results: AgentResult[];
  consensus: string;
  layer2?: Layer2Section[];
}
