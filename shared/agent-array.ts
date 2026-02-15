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
export const TERNARY_RADIAN = 13;
export const FULL_CIRCLE = 364;
export const CONVOLUTION_KERNEL = [13, 24, 44] as const;

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

export type AgentCategory =
  | "International Law"
  | "Regional Legal Systems"
  | "Finance"
  | "Crypto"
  | "Security";

export interface AgentLanguage {
  code: string;
  name: string;
  nativeName: string;
}

export const AGENT_LANGUAGES: AgentLanguage[] = [
  { code: "en", name: "English", nativeName: "English" },
  { code: "fr", name: "French", nativeName: "Fran\u00e7ais" },
  { code: "es", name: "Spanish", nativeName: "Espa\u00f1ol" },
  { code: "de", name: "German", nativeName: "Deutsch" },
  { code: "zh", name: "Chinese", nativeName: "\u4e2d\u6587" },
  { code: "ar", name: "Arabic", nativeName: "\u0627\u0644\u0639\u0631\u0628\u064a\u0629" },
  { code: "pt", name: "Portuguese", nativeName: "Portugu\u00eas" },
  { code: "ru", name: "Russian", nativeName: "\u0420\u0443\u0441\u0441\u043a\u0438\u0439" },
  { code: "ja", name: "Japanese", nativeName: "\u65e5\u672c\u8a9e" },
  { code: "ko", name: "Korean", nativeName: "\ud55c\uad6d\uc5b4" },
  { code: "hi", name: "Hindi", nativeName: "\u0939\u093f\u0928\u094d\u0926\u0940" },
  { code: "it", name: "Italian", nativeName: "Italiano" },
  { code: "nl", name: "Dutch", nativeName: "Nederlands" },
  { code: "sv", name: "Swedish", nativeName: "Svenska" },
  { code: "tr", name: "Turkish", nativeName: "T\u00fcrk\u00e7e" },
  { code: "pl", name: "Polish", nativeName: "Polski" },
  { code: "th", name: "Thai", nativeName: "\u0e44\u0e17\u0e22" },
  { code: "vi", name: "Vietnamese", nativeName: "Ti\u1ebfng Vi\u1ec7t" },
  { code: "id", name: "Indonesian", nativeName: "Bahasa Indonesia" },
  { code: "he", name: "Hebrew", nativeName: "\u05e2\u05d1\u05e8\u05d9\u05ea" },
  { code: "el", name: "Greek", nativeName: "\u0395\u03bb\u03bb\u03b7\u03bd\u03b9\u03ba\u03ac" },
  { code: "cs", name: "Czech", nativeName: "\u010ce\u0161tina" },
  { code: "ro", name: "Romanian", nativeName: "Rom\u00e2n\u0103" },
  { code: "uk", name: "Ukrainian", nativeName: "\u0423\u043a\u0440\u0430\u0457\u043d\u0441\u044c\u043a\u0430" },
  { code: "da", name: "Danish", nativeName: "Dansk" },
  { code: "fi", name: "Finnish", nativeName: "Suomi" },
  { code: "ms", name: "Malay", nativeName: "Bahasa Melayu" },
  { code: "sw", name: "Swahili", nativeName: "Kiswahili" },
];

export interface AgentSpecialist {
  title: string;
  description: string;
  category: AgentCategory;
  subdomain: string;
  keywords: string[];
  systemPrompt: string;
  weight: number;
  language: AgentLanguage;
}

export const LAYER2_SECTIONS: { key: AgentCategory; label: string; agentIndices: number[] }[] = [
  { key: "International Law", label: "International Law", agentIndices: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14] },
  { key: "Regional Legal Systems", label: "Regional Legal Systems", agentIndices: [15,16,17,18,19] },
  { key: "Finance", label: "Finance", agentIndices: [20,21,22] },
  { key: "Crypto", label: "Crypto", agentIndices: [23,24] },
  { key: "Security", label: "Security", agentIndices: [25,26,27] },
];

export const DEFAULT_SPECIALISTS: AgentSpecialist[] = [
  {
    title: "Public International Law Specialist",
    description: "Expert in treaties, state sovereignty, international organizations, and customary international law.",
    category: "International Law",
    subdomain: "public",
    keywords: ["treaties", "state sovereignty", "international organizations", "customary international law", "UN Charter", "ICJ"],
    systemPrompt: "You are a Public International Law specialist. Expertise: treaties, sovereignty, UN Charter, ICJ jurisprudence, customary norms. Analyze obligations and jurisdictional issues. Respond with: applicable (yes/no), risk_level (none/low/medium/high/critical), key_issues list, and recommendations. You MUST respond entirely in English (English).",
    weight: 1.0,
    language: AGENT_LANGUAGES[0],
  },
  {
    title: "Private International Law Specialist",
    description: "Focuses on conflict of laws, jurisdiction, and choice of law in cross-border disputes.",
    category: "International Law",
    subdomain: "private",
    keywords: ["conflict of laws", "jurisdiction", "choice of law", "cross-border disputes", "Hague Convention"],
    systemPrompt: "You are a Private International Law specialist. Expertise: conflict of laws, jurisdictional analysis, choice of law, forum selection, Hague Convention. Identify which jurisdiction's law applies and potential conflicts. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in French (Français).",
    weight: 1.0,
    language: AGENT_LANGUAGES[1],
  },
  {
    title: "International Trade Law Expert",
    description: "Specializes in WTO rules, regional trade agreements, tariffs, and trade remedies.",
    category: "International Law",
    subdomain: "trade",
    keywords: ["WTO", "tariffs", "trade agreements", "trade remedies", "customs", "GATT", "TRIPS"],
    systemPrompt: "You are an International Trade Law expert. Expertise: WTO, USMCA, CPTPP, RCEP, tariffs, anti-dumping, countervailing duties, customs. Analyze trade compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Spanish (Español).",
    weight: 1.0,
    language: AGENT_LANGUAGES[2],
  },
  {
    title: "International Investment Law Expert",
    description: "Deals with bilateral investment treaties, investor-state arbitration, and FDI protections.",
    category: "International Law",
    subdomain: "investment",
    keywords: ["bilateral investment treaties", "investor-state arbitration", "ICSID", "FDI", "expropriation"],
    systemPrompt: "You are an International Investment Law expert. Expertise: BITs, ISDS, ICSID arbitration, FDI protections, fair and equitable treatment, expropriation. Assess investment protection implications. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in German (Deutsch).",
    weight: 1.0,
    language: AGENT_LANGUAGES[3],
  },
  {
    title: "International Tax Law Specialist",
    description: "Master of cross-border taxation, transfer pricing, tax treaties, and BEPS.",
    category: "International Law",
    subdomain: "tax",
    keywords: ["cross-border taxation", "transfer pricing", "tax treaties", "BEPS", "OECD", "permanent establishment"],
    systemPrompt: "You are an International Tax Law specialist. Expertise: cross-border taxation, transfer pricing, double tax treaties, BEPS, permanent establishment, digital services tax. Analyze tax exposure and treaty benefits. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Chinese (中文).",
    weight: 1.0,
    language: AGENT_LANGUAGES[4],
  },
  {
    title: "International IP Law Expert",
    description: "Covers global patents, trademarks, copyrights, and trade secrets.",
    category: "International Law",
    subdomain: "ip",
    keywords: ["patents", "trademarks", "copyrights", "trade secrets", "WIPO", "Paris Convention", "Berne Convention"],
    systemPrompt: "You are an International IP Law expert. Expertise: PCT patents, Madrid trademarks, Berne Convention copyrights, trade secrets, WIPO. Assess IP risks and protection strategies. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Arabic (العربية).",
    weight: 1.0,
    language: AGENT_LANGUAGES[5],
  },
  {
    title: "International Environmental Law Expert",
    description: "Addresses climate change treaties, biodiversity, transboundary pollution, and ESG.",
    category: "International Law",
    subdomain: "environmental",
    keywords: ["climate change", "Paris Agreement", "biodiversity", "transboundary pollution", "CBD", "ESG"],
    systemPrompt: "You are an International Environmental Law expert. Expertise: Paris Agreement, UNFCCC, CBD, Nagoya Protocol, transboundary pollution, carbon markets, ESG. Assess environmental compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Portuguese (Português).",
    weight: 1.0,
    language: AGENT_LANGUAGES[6],
  },
  {
    title: "International Human Rights Law Specialist",
    description: "Focuses on human rights treaties, UN mechanisms, and corporate social responsibility.",
    category: "International Law",
    subdomain: "human_rights",
    keywords: ["human rights treaties", "UN mechanisms", "CSR", "due diligence", "UDHR", "ICCPR"],
    systemPrompt: "You are an International Human Rights Law specialist. Expertise: ICCPR, ICESCR, ECHR, UNGPs, mandatory human rights due diligence (EU CSDDD), corporate social responsibility. Assess human rights implications. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Russian (Русский).",
    weight: 1.0,
    language: AGENT_LANGUAGES[7],
  },
  {
    title: "International Criminal & Humanitarian Law Expert",
    description: "Handles war crimes, crimes against humanity, ICC proceedings, and Geneva Conventions.",
    category: "International Law",
    subdomain: "criminal",
    keywords: ["war crimes", "ICC", "Geneva Conventions", "crimes against humanity", "genocide", "humanitarian law"],
    systemPrompt: "You are an International Criminal and Humanitarian Law expert. Expertise: war crimes, ICC proceedings, Geneva Conventions, IHL, corporate complicity. Assess criminal and humanitarian law exposure. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Japanese (日本語).",
    weight: 0.8,
    language: AGENT_LANGUAGES[8],
  },
  {
    title: "International Maritime & Aviation Law Specialist",
    description: "Expert in law of the sea, shipping regulations, air transport, and liability.",
    category: "International Law",
    subdomain: "maritime_aviation",
    keywords: ["UNCLOS", "shipping", "maritime law", "ICAO", "air transport", "admiralty"],
    systemPrompt: "You are an International Maritime and Aviation Law specialist. Expertise: UNCLOS, IMO conventions, admiralty, Chicago Convention, ICAO, cabotage, flag state obligations. Assess maritime and aviation compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Korean (한국어).",
    weight: 0.8,
    language: AGENT_LANGUAGES[9],
  },
  {
    title: "International Space & Cyber Law Expert",
    description: "Covers outer space treaties, cyber norms, digital sovereignty, and emerging tech governance.",
    category: "International Law",
    subdomain: "space_cyber",
    keywords: ["outer space treaties", "cyber norms", "digital sovereignty", "Tallinn Manual", "satellite regulation"],
    systemPrompt: "You are an International Space and Cyber Law expert. Expertise: Outer Space Treaty, satellite regulation, Tallinn Manual, digital sovereignty, AI governance, emerging tech regulation. Assess space and cyber law compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Hindi (हिन्दी).",
    weight: 1.0,
    language: AGENT_LANGUAGES[10],
  },
  {
    title: "International Energy & Natural Resources Law",
    description: "Deals with oil, gas, mining, renewables, and cross-border energy projects.",
    category: "International Law",
    subdomain: "energy",
    keywords: ["oil", "gas", "mining", "renewables", "energy charter", "cross-border energy"],
    systemPrompt: "You are an International Energy and Natural Resources Law expert. Expertise: oil/gas regulation, mining law, renewables, Energy Charter Treaty, resource nationalism, energy transition. Assess energy sector compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Italian (Italiano).",
    weight: 0.8,
    language: AGENT_LANGUAGES[11],
  },
  {
    title: "International Labor & Employment Law Expert",
    description: "Focuses on global labor standards, migrant workers, and cross-border employment issues.",
    category: "International Law",
    subdomain: "labor",
    keywords: ["ILO", "labor standards", "migrant workers", "cross-border employment", "posted workers"],
    systemPrompt: "You are an International Labor and Employment Law expert. Expertise: ILO conventions, global labor standards, migrant workers, posted workers directives, supply chain labor compliance. Assess labor law implications. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Dutch (Nederlands).",
    weight: 0.8,
    language: AGENT_LANGUAGES[12],
  },
  {
    title: "International Competition/Antitrust Law Specialist",
    description: "Handles cross-border mergers, antitrust investigations, and cartel enforcement.",
    category: "International Law",
    subdomain: "competition",
    keywords: ["cross-border mergers", "antitrust", "cartels", "market dominance", "merger control"],
    systemPrompt: "You are an International Competition/Antitrust Law specialist. Expertise: cross-border merger control, antitrust investigations, cartel enforcement, abuse of dominance, state aid, digital market competition. Assess competition law risks. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Swedish (Svenska).",
    weight: 0.9,
    language: AGENT_LANGUAGES[13],
  },
  {
    title: "International Dispute Resolution Expert",
    description: "Specializes in international arbitration, mediation, and litigation across jurisdictions.",
    category: "International Law",
    subdomain: "dispute_resolution",
    keywords: ["international arbitration", "mediation", "ICC arbitration", "LCIA", "UNCITRAL", "enforcement"],
    systemPrompt: "You are an International Dispute Resolution expert. Expertise: ICC, LCIA, SIAC, HKIAC arbitration, UNCITRAL rules, mediation, New York Convention enforcement. Assess dispute resolution strategy. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Turkish (Türkçe).",
    weight: 0.9,
    language: AGENT_LANGUAGES[14],
  },
  {
    title: "Common Law Systems Expert",
    description: "Deep knowledge of US, UK, Canada, Australia, and other common law jurisdictions.",
    category: "Regional Legal Systems",
    subdomain: "common_law",
    keywords: ["US law", "UK law", "Canada", "Australia", "common law", "stare decisis", "precedent"],
    systemPrompt: "You are a Common Law systems expert. Deep knowledge of US, UK, Canada, Australia. Expertise: constitutional law, contracts, torts, corporate governance, stare decisis. Analyze common law implications. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Polish (Polski).",
    weight: 1.0,
    language: AGENT_LANGUAGES[15],
  },
  {
    title: "Civil Law Systems Expert",
    description: "Master of European, Latin American, and other civil law systems (e.g., France, Germany, Brazil).",
    category: "Regional Legal Systems",
    subdomain: "civil_law",
    keywords: ["European law", "Latin American law", "French law", "German law", "codified systems"],
    systemPrompt: "You are a Civil Law systems expert. Master of European, Latin American civil law (France, Germany, Brazil, Japan). Expertise: codified law, Napoleonic traditions, EU directives, Germanic legal traditions. Analyze civil law implications. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Thai (ไทย).",
    weight: 1.0,
    language: AGENT_LANGUAGES[16],
  },
  {
    title: "Asian Legal Systems Expert",
    description: "Focuses on China, Japan, Korea, and Southeast Asian legal frameworks.",
    category: "Regional Legal Systems",
    subdomain: "asian",
    keywords: ["Chinese law", "Japanese law", "Korean law", "ASEAN law", "Southeast Asian law"],
    systemPrompt: "You are an Asian Legal Systems expert. Focus: China, Japan, Korea, Southeast Asia. Expertise: Chinese commercial law, data localization, Japanese corporate law, Korean regulations, ASEAN harmonization. Analyze Asian jurisdictional compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Vietnamese (Tiếng Việt).",
    weight: 1.0,
    language: AGENT_LANGUAGES[17],
  },
  {
    title: "Islamic Law Expert",
    description: "Specializes in Sharia, Islamic finance, and family law in Muslim-majority countries.",
    category: "Regional Legal Systems",
    subdomain: "islamic",
    keywords: ["Sharia", "Islamic finance", "Sukuk", "Murabaha", "halal", "waqf"],
    systemPrompt: "You are an Islamic Law expert. Specialization: Sharia, Islamic finance (Sukuk, Murabaha, Ijara, Takaful), family law in Muslim-majority countries, waqf, harmonization of Islamic and civil law. Analyze Sharia compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Indonesian (Bahasa Indonesia).",
    weight: 0.9,
    language: AGENT_LANGUAGES[18],
  },
  {
    title: "African Legal Systems Expert",
    description: "Knowledge of customary law, regional bodies (AU, ECOWAS), and diverse national laws.",
    category: "Regional Legal Systems",
    subdomain: "african",
    keywords: ["African Union", "ECOWAS", "OHADA", "customary law", "AfCFTA"],
    systemPrompt: "You are an African Legal Systems expert. Knowledge: customary law, AU, ECOWAS, SADC, OHADA business law, AfCFTA implementation, diverse national systems. Analyze African jurisdictional compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Hebrew (עברית).",
    weight: 0.9,
    language: AGENT_LANGUAGES[19],
  },
  {
    title: "International Finance & Banking Specialist",
    description: "Expert in central banking, monetary policy, Basel regulations, and global financial markets.",
    category: "Finance",
    subdomain: "banking",
    keywords: ["central banking", "Basel", "monetary policy", "financial markets", "banking regulation"],
    systemPrompt: "You are an International Finance and Banking specialist. Expertise: central banking, monetary policy, Basel III/IV, capital adequacy, liquidity coverage, cross-border banking supervision. Assess financial regulatory compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Greek (Ελληνικά).",
    weight: 1.0,
    language: AGENT_LANGUAGES[20],
  },
  {
    title: "Corporate Finance & Investment Banking Expert",
    description: "Handles M&A, capital markets, private equity, and cross-border transactions.",
    category: "Finance",
    subdomain: "corporate",
    keywords: ["M&A", "capital markets", "private equity", "securities", "IPO", "cross-border transactions"],
    systemPrompt: "You are a Corporate Finance and Investment Banking expert. Expertise: M&A, capital markets, private equity, venture capital, securities (SEC, FCA, ESMA), IPOs, cross-border transaction structuring. Assess corporate finance implications. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Czech (Čeština).",
    weight: 1.0,
    language: AGENT_LANGUAGES[21],
  },
  {
    title: "Financial Crime & AML Specialist",
    description: "Focuses on AML, counter-terrorism financing, fraud detection, and regulatory compliance.",
    category: "Finance",
    subdomain: "aml",
    keywords: ["AML", "CFT", "FATF", "sanctions", "fraud", "KYC", "suspicious activity"],
    systemPrompt: "You are a Financial Crime and AML specialist. Expertise: AML/CFT frameworks (FATF), sanctions (OFAC, EU, UN), KYC/CDD, suspicious activity reporting, fraud detection, beneficial ownership. Assess financial crime exposure. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Romanian (Română).",
    weight: 1.0,
    language: AGENT_LANGUAGES[22],
  },
  {
    title: "Blockchain & Cryptocurrency Expert",
    description: "Technical master of consensus mechanisms, smart contracts, DeFi, and blockchain architecture.",
    category: "Crypto",
    subdomain: "technical",
    keywords: ["blockchain", "DeFi", "smart contracts", "consensus", "Layer 2", "NFT", "DAO"],
    systemPrompt: "You are a Blockchain Technology and Cryptocurrency expert. Mastery: consensus mechanisms, smart contract security, DeFi architecture, Layer 2 scaling, NFTs, DAO governance, cross-chain interop, ternary computing compatibility. Assess technical blockchain implications. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Ukrainian (Українська).",
    weight: 1.0,
    language: AGENT_LANGUAGES[23],
  },
  {
    title: "Crypto Legal & Regulatory Specialist",
    description: "Expert in legal status of cryptocurrencies, securities laws, compliance, and tax implications.",
    category: "Crypto",
    subdomain: "legal",
    keywords: ["MiCA", "crypto regulation", "securities law", "Howey test", "stablecoin", "VASP"],
    systemPrompt: "You are a Crypto Legal and Regulatory specialist. Expertise: crypto legal status, securities law (Howey test), MiCA (EU), stablecoin regulation, VASP licensing, DeFi regulatory challenges, digital asset taxation. Assess crypto-legal compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Danish (Dansk).",
    weight: 1.0,
    language: AGENT_LANGUAGES[24],
  },
  {
    title: "Cybersecurity Technical Expert",
    description: "Specialist in network security, threat intelligence, incident response, and encryption.",
    category: "Security",
    subdomain: "cyber",
    keywords: ["network security", "threat intelligence", "encryption", "incident response", "zero trust", "post-quantum"],
    systemPrompt: "You are a Cybersecurity Technical expert. Specialization: network security, threat intelligence, incident response, encryption standards including post-quantum cryptography, zero trust architecture, security auditing, ternary-native security. Assess cybersecurity posture. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Finnish (Suomi).",
    weight: 1.0,
    language: AGENT_LANGUAGES[25],
  },
  {
    title: "National Security & Geopolitical Risk Analyst",
    description: "Focuses on sanctions, export controls, geopolitical trends, and risk assessment.",
    category: "Security",
    subdomain: "geopolitical",
    keywords: ["sanctions", "export controls", "CFIUS", "geopolitical risk", "dual-use", "ITAR"],
    systemPrompt: "You are a National Security and Geopolitical Risk analyst. Focus: sanctions (OFAC, EU, UK), export controls (EAR, ITAR, Wassenaar), CFIUS review, geopolitical trends, dual-use technology controls, foreign ownership restrictions. Assess national security and geopolitical risk. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Malay (Bahasa Melayu).",
    weight: 1.0,
    language: AGENT_LANGUAGES[26],
  },
  {
    title: "Data Privacy & Protection Specialist",
    description: "Expert in GDPR, CCPA, cross-border data flows, and privacy-by-design.",
    category: "Security",
    subdomain: "privacy",
    keywords: ["GDPR", "CCPA", "cross-border data", "privacy-by-design", "data localization", "AI Act"],
    systemPrompt: "You are a Data Privacy and Protection specialist. Expertise: GDPR, CCPA/CPRA, PIPL (China), LGPD (Brazil), cross-border data transfers (SCCs, adequacy), privacy-by-design, data localization, EU AI Act. Assess data privacy compliance. Respond with: applicable, risk_level, key_issues, recommendations. You MUST respond entirely in Swahili (Kiswahili).",
    weight: 1.0,
    language: AGENT_LANGUAGES[27],
  },
];

export const AGENT_DOMAINS = DEFAULT_SPECIALISTS.map((s) => s.title);

export function tribonacciPermutation(): number[] {
  return Array.from({ length: AGENT_COUNT }, (_, i) => (i * TERNARY_RADIAN) % AGENT_COUNT);
}

export function scheduleAgents(): number[] {
  const perm = tribonacciPermutation();
  return perm;
}

export interface AgentPosition {
  index: number;
  z28: number;
  angleDeg: number;
  label: string;
  domain: string;
  description: string;
  category: AgentCategory;
  language: AgentLanguage;
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
    language: roles[z28]?.language || DEFAULT_SPECIALISTS[z28].language,
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
  language: AgentLanguage;
}

export interface Layer2Section {
  category: AgentCategory;
  label: string;
  technicalSummary: string;
  laySummary: string;
  agentCount: number;
  successCount: number;
}

export type VerdictSignal = "GREEN" | "YELLOW" | "RED";

export interface ExecutiveVerdict {
  signal: VerdictSignal;
  assessment: string;
  confidence: number;
}

export interface JurisdictionalRegion {
  region: string;
  status: "permitted" | "conditional" | "restricted" | "prohibited" | "unclear";
  notes: string;
}

export interface RiskScore {
  category: string;
  score: number;
  narrative: string;
}

export interface CriticalStep {
  order: number;
  action: string;
  category: string;
}

export interface ExecutiveSummary {
  verdict: ExecutiveVerdict;
  jurisdictionalCompass: JurisdictionalRegion[];
  riskBarometer: {
    financial: RiskScore[];
    technical: RiskScore[];
    aggregateFinancial: number;
    aggregateTechnical: number;
  };
  criticalPath: CriticalStep[];
  plainEnglish: {
    summary: string;
    boardRecommendation: string;
  };
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
  executiveSummary?: ExecutiveSummary;
  tribonacciHash?: string;
  executionOrder?: number[];
}
