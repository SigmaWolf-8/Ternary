"""
PlenumNET 28-Agent International Compliance Engine
═══════════════════════════════════════════════════
Orchestrates the three-layer processing pipeline:
  Layer 1: 28 specialist agents deliberate in parallel (AsyncAnthropic)
  Layer 2: Enhanced synthesis with Etymology, Veritas Audit, Lexical Protocols
  Layer 3: Polyglot localization into 28 languages (Batch API for cost optimization)

Architecture:
  • Agents scheduled via Tribonacci 13-step permutation
  • Async parallel execution for Layer 1 (all 28 agents simultaneously)
  • Opus model for Layer 2 synthesis (highest reasoning capability)
  • Batch API for Layer 3 translations (50% cost reduction)

Applied Physics Division — Capomastro Holdings Ltd.
"""

from __future__ import annotations

import asyncio
import json
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

import yaml

try:
    from anthropic import AsyncAnthropic
except ImportError:
    AsyncAnthropic = None  # Handled in init

from .tribonacci_scheduler import (
    schedule_agents,
    generate_query_hash,
    agent_permutation,
    NUM_AGENTS,
    TERNARY_RADIAN,
    FULL_CIRCLE,
    CONVOLUTION_KERNEL,
)

# ── Configuration Loading ─────────────────────────────────────────────────────

CONFIG_DIR = Path(__file__).parent.parent.parent / "config"


def load_config(filename: str) -> Dict[str, Any]:
    """Load a YAML configuration file."""
    path = CONFIG_DIR / filename
    if not path.exists():
        raise FileNotFoundError(f"Configuration file not found: {path}")
    with open(path, "r", encoding="utf-8") as f:
        return yaml.safe_load(f)


# ── Pydantic-style Data Classes (no external deps beyond stdlib) ──────────────

class AgentDeliberation:
    """Layer 1 output from a single specialist agent."""

    def __init__(self, agent_id: int, agent_name: str, domain: str,
                 subdomain: str, raw_response: str, assessment: Dict[str, Any],
                 confidence: float, relevance_score: float,
                 processing_time_ms: int):
        self.agent_id = agent_id
        self.agent_name = agent_name
        self.domain = domain
        self.subdomain = subdomain
        self.raw_response = raw_response
        self.assessment = assessment
        self.confidence = confidence
        self.relevance_score = relevance_score
        self.processing_time_ms = processing_time_ms

    def to_dict(self) -> Dict[str, Any]:
        return {
            "agent_id": self.agent_id,
            "agent_name": self.agent_name,
            "domain": self.domain,
            "subdomain": self.subdomain,
            "relevance_score": self.relevance_score,
            "assessment": self.assessment,
            "confidence": self.confidence,
            "processing_time_ms": self.processing_time_ms,
        }


class ComplianceOutput:
    """Complete output bundle for a query."""

    def __init__(self, query_text: str):
        self.query_id = str(uuid.uuid4())
        self.timestamp = datetime.now(timezone.utc).isoformat()
        self.query_text = query_text
        self.tribonacci_hash = generate_query_hash(query_text)
        self.layer1_deliberations: List[Dict[str, Any]] = []
        self.layer2_executive_summary: Dict[str, Any] = {}
        self.layer3_translations: List[Dict[str, Any]] = []
        self.audit_log: Dict[str, Any] = {
            "etymology_checks": 0,
            "veritas_validations": 0,
            "lexical_protocol_version": "1.0",
            "agent_execution_order": agent_permutation(),
        }
        self.total_processing_time_ms = 0
        self.total_tokens = 0

    def to_dict(self) -> Dict[str, Any]:
        return {
            "metadata": {
                "query_id": self.query_id,
                "timestamp": self.timestamp,
                "engine_version": "1.0.0-plenumnet",
                "tribonacci_hash": self.tribonacci_hash,
                "processing_time_ms": self.total_processing_time_ms,
                "model_used": "claude-sonnet-4-5-20250929 / claude-opus-4-6",
                "total_tokens": self.total_tokens,
                "tribonacci_constants": {
                    "ternary_radian": TERNARY_RADIAN,
                    "num_agents": NUM_AGENTS,
                    "full_circle": FULL_CIRCLE,
                    "convolution_kernel": list(CONVOLUTION_KERNEL),
                },
            },
            "query": {
                "original_text": self.query_text,
            },
            "layer1_deliberations": self.layer1_deliberations,
            "layer2_executive_summary": self.layer2_executive_summary,
            "layer3_translations": self.layer3_translations,
            "audit_log": self.audit_log,
        }

    def to_json(self, indent: int = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent, ensure_ascii=False)


# ── Core Engine ───────────────────────────────────────────────────────────────

class ComplianceEngine:
    """
    28-Agent International Compliance Engine.
    
    Pipeline:
        query → Layer 1 (28 agents parallel) → Layer 2 (synthesis) → Layer 3 (translation) → output
    """

    def __init__(self, api_key: Optional[str] = None):
        if AsyncAnthropic is None:
            raise ImportError(
                "anthropic package is required. Install with: pip install anthropic"
            )

        self.client = AsyncAnthropic(api_key=api_key)
        self.agents_config = load_config("agents_config.yaml")
        self.languages_config = load_config("languages.yaml")
        self.engine_config = self.agents_config["engine"]
        self.agents = self.agents_config["agents"]
        self.languages = self.languages_config["languages"]

        # Validate 28-fold alignment
        assert len(self.agents) == NUM_AGENTS, (
            f"Expected {NUM_AGENTS} agents, found {len(self.agents)}"
        )
        assert len(self.languages) == NUM_AGENTS, (
            f"Expected {NUM_AGENTS} languages, found {len(self.languages)}"
        )

    # ── Layer 1: Parallel Agent Deliberation ──────────────────────────────────

    async def _run_single_agent(
        self, agent: Dict[str, Any], query: str
    ) -> AgentDeliberation:
        """Execute a single specialist agent against the query."""
        start = time.monotonic()

        system_prompt = agent["system_prompt"] + "\n\n" + self._agent_output_format()

        try:
            response = await self.client.messages.create(
                model=self.engine_config["model"],
                max_tokens=self.engine_config["max_tokens_per_agent"],
                temperature=self.engine_config["temperature"],
                system=system_prompt,
                messages=[{"role": "user", "content": query}],
            )

            raw_text = response.content[0].text
            elapsed_ms = int((time.monotonic() - start) * 1000)

            # Parse JSON from response
            assessment = self._parse_agent_response(raw_text)

            return AgentDeliberation(
                agent_id=agent["id"],
                agent_name=agent["name"],
                domain=agent["domain"],
                subdomain=agent["subdomain"],
                raw_response=raw_text,
                assessment=assessment.get("assessment", {}),
                confidence=assessment.get("confidence", 0.5),
                relevance_score=assessment.get("relevance_score", 0.5),
                processing_time_ms=elapsed_ms,
            )

        except Exception as e:
            elapsed_ms = int((time.monotonic() - start) * 1000)
            return AgentDeliberation(
                agent_id=agent["id"],
                agent_name=agent["name"],
                domain=agent["domain"],
                subdomain=agent["subdomain"],
                raw_response=f"ERROR: {str(e)}",
                assessment={"applicable": False, "risk_level": "none", "key_issues": [], "error": str(e)},
                confidence=0.0,
                relevance_score=0.0,
                processing_time_ms=elapsed_ms,
            )

    async def run_layer1(self, query: str) -> List[AgentDeliberation]:
        """
        Execute all 28 specialist agents in parallel.
        Agents are scheduled in Tribonacci 13-step permutation order.
        """
        execution_order = schedule_agents()
        ordered_agents = [self.agents[i] for i in execution_order]

        # Launch all 28 agents concurrently
        tasks = [self._run_single_agent(agent, query) for agent in ordered_agents]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Handle any exceptions
        deliberations = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                agent = ordered_agents[i]
                deliberations.append(AgentDeliberation(
                    agent_id=agent["id"],
                    agent_name=agent["name"],
                    domain=agent["domain"],
                    subdomain=agent["subdomain"],
                    raw_response=f"EXCEPTION: {str(result)}",
                    assessment={"applicable": False, "risk_level": "none", "key_issues": []},
                    confidence=0.0,
                    relevance_score=0.0,
                    processing_time_ms=0,
                ))
            else:
                deliberations.append(result)

        # Sort back to canonical order (0–27)
        deliberations.sort(key=lambda d: d.agent_id)
        return deliberations

    # ── Layer 2: Enhanced Synthesis ───────────────────────────────────────────

    async def run_layer2(
        self, query: str, deliberations: List[AgentDeliberation]
    ) -> Dict[str, Any]:
        """
        Synthesize all 28 agent deliberations into the 5-section Executive Summary.
        
        Pre-synthesis steps:
          1. Etymology & Cross-Cultural Word Synchronization
          2. Veritas Audit (5 languages, 3 cultures, 200+ years)
          3. Lexical Protocols application
        
        Uses Opus model for maximum reasoning capability.
        """
        # Prepare agent notes for synthesis
        agent_summaries = []
        for d in deliberations:
            agent_summaries.append({
                "agent_id": d.agent_id,
                "agent_name": d.agent_name,
                "domain": f"{d.domain}/{d.subdomain}",
                "relevance": d.relevance_score,
                "confidence": d.confidence,
                "assessment": d.assessment,
            })

        synthesis_prompt = self._build_synthesis_prompt(query, agent_summaries)

        response = await self.client.messages.create(
            model=self.engine_config["synthesis_model"],
            max_tokens=self.engine_config["max_tokens_synthesis"],
            temperature=self.engine_config["synthesis_temperature"],
            system=self._synthesis_system_prompt(),
            messages=[{"role": "user", "content": synthesis_prompt}],
        )

        raw_text = response.content[0].text
        return self._parse_synthesis_response(raw_text)

    # ── Layer 3: Polyglot Localization ────────────────────────────────────────

    async def run_layer3(
        self, executive_summary: Dict[str, Any]
    ) -> List[Dict[str, Any]]:
        """
        Translate the Executive Summary into all 28 languages.
        
        Uses the Batch API when batch_translations is enabled (50% cost reduction).
        Falls back to parallel async calls otherwise.
        """
        if self.engine_config.get("batch_translations", False):
            return await self._run_layer3_batch(executive_summary)
        else:
            return await self._run_layer3_parallel(executive_summary)

    async def _run_layer3_parallel(
        self, executive_summary: Dict[str, Any]
    ) -> List[Dict[str, Any]]:
        """Translate into all 28 languages using parallel async calls."""
        summary_text = json.dumps(executive_summary, ensure_ascii=False)

        async def translate_one(lang: Dict[str, Any]) -> Dict[str, Any]:
            prompt = self._build_translation_prompt(summary_text, lang)
            try:
                response = await self.client.messages.create(
                    model=self.engine_config["model"],
                    max_tokens=self.engine_config["max_tokens_translation"],
                    temperature=0.2,
                    system=self._translation_system_prompt(lang),
                    messages=[{"role": "user", "content": prompt}],
                )
                return {
                    "language_id": lang["id"],
                    "language_code": lang["code"],
                    "language_name": lang["name"],
                    "translated_summary": response.content[0].text,
                    "lexical_protocol_applied": True,
                    "translation_confidence": 0.85,
                }
            except Exception as e:
                return {
                    "language_id": lang["id"],
                    "language_code": lang["code"],
                    "language_name": lang["name"],
                    "translated_summary": f"TRANSLATION ERROR: {str(e)}",
                    "lexical_protocol_applied": False,
                    "translation_confidence": 0.0,
                }

        tasks = [translate_one(lang) for lang in self.languages]
        return await asyncio.gather(*tasks)

    async def _run_layer3_batch(
        self, executive_summary: Dict[str, Any]
    ) -> List[Dict[str, Any]]:
        """
        Translate using the Anthropic Message Batches API.
        50% cost reduction, processed within 1 hour typically.
        """
        summary_text = json.dumps(executive_summary, ensure_ascii=False)

        # Build batch requests (one per language)
        batch_requests = []
        for lang in self.languages:
            batch_requests.append({
                "custom_id": f"lang-{lang['id']:02d}-{lang['code']}",
                "params": {
                    "model": self.engine_config["model"],
                    "max_tokens": self.engine_config["max_tokens_translation"],
                    "temperature": 0.2,
                    "system": self._translation_system_prompt(lang),
                    "messages": [
                        {
                            "role": "user",
                            "content": self._build_translation_prompt(
                                summary_text, lang
                            ),
                        }
                    ],
                },
            })

        # Submit batch
        batch = await self.client.messages.batches.create(requests=batch_requests)

        # Poll for completion
        while batch.processing_status != "ended":
            await asyncio.sleep(30)
            batch = await self.client.messages.batches.retrieve(batch.id)

        # Collect results
        translations = []
        async for entry in await self.client.messages.batches.results(batch.id):
            lang_id = int(entry.custom_id.split("-")[1])
            lang = self.languages[lang_id]
            if entry.result.type == "succeeded":
                translations.append({
                    "language_id": lang_id,
                    "language_code": lang["code"],
                    "language_name": lang["name"],
                    "translated_summary": entry.result.message.content[0].text,
                    "lexical_protocol_applied": True,
                    "translation_confidence": 0.85,
                })
            else:
                translations.append({
                    "language_id": lang_id,
                    "language_code": lang["code"],
                    "language_name": lang["name"],
                    "translated_summary": f"BATCH ERROR: {entry.result.type}",
                    "lexical_protocol_applied": False,
                    "translation_confidence": 0.0,
                })

        translations.sort(key=lambda t: t["language_id"])
        return translations

    # ── Full Pipeline ─────────────────────────────────────────────────────────

    async def process_query(
        self,
        query: str,
        include_translations: bool = True,
        include_raw_deliberations: bool = True,
    ) -> ComplianceOutput:
        """
        Run the complete 3-layer compliance analysis pipeline.
        
        1. Layer 1: 28 specialists deliberate in parallel
        2. Layer 2: Synthesis with Etymology + Veritas + Lexical Protocols
        3. Layer 3: Translation into 28 languages (optional)
        """
        output = ComplianceOutput(query)
        pipeline_start = time.monotonic()

        # ── Layer 1 ──
        print(f"[Layer 1] Launching 28 specialist agents in parallel...")
        deliberations = await self.run_layer1(query)
        output.layer1_deliberations = [d.to_dict() for d in deliberations]
        l1_time = int((time.monotonic() - pipeline_start) * 1000)
        print(f"[Layer 1] Complete in {l1_time}ms")

        # ── Layer 2 ──
        print(f"[Layer 2] Synthesizing executive summary (Opus)...")
        l2_start = time.monotonic()
        output.layer2_executive_summary = await self.run_layer2(query, deliberations)
        l2_time = int((time.monotonic() - l2_start) * 1000)
        print(f"[Layer 2] Complete in {l2_time}ms")

        # ── Layer 3 ──
        if include_translations:
            print(f"[Layer 3] Translating into 28 languages...")
            l3_start = time.monotonic()
            output.layer3_translations = await self.run_layer3(
                output.layer2_executive_summary
            )
            l3_time = int((time.monotonic() - l3_start) * 1000)
            print(f"[Layer 3] Complete in {l3_time}ms")

        output.total_processing_time_ms = int(
            (time.monotonic() - pipeline_start) * 1000
        )
        print(f"[Engine] Total processing time: {output.total_processing_time_ms}ms")

        return output

    # ── Prompt Construction ───────────────────────────────────────────────────

    @staticmethod
    def _agent_output_format() -> str:
        return """
Respond ONLY with valid JSON in this exact structure:
{
  "relevance_score": 0.0-1.0,
  "confidence": 0.0-1.0,
  "assessment": {
    "applicable": true/false,
    "risk_level": "none|low|medium|high|critical",
    "key_issues": ["issue1", "issue2"],
    "applicable_laws": ["law1", "law2"],
    "jurisdictions_affected": ["jurisdiction1"],
    "recommendations": ["recommendation1"],
    "blocking_points": ["blocker1"]
  }
}
Do not include any text outside the JSON object.
"""

    @staticmethod
    def _synthesis_system_prompt() -> str:
        return """You are the PlenumNET Compliance Synthesis Engine. You receive deliberations from 28
specialist agents across international law, regional legal systems, finance, crypto, and
security domains.

Your task is to synthesize these into a 5-section Executive Summary following this exact
structure, with two critical pre-synthesis steps:

PRE-SYNTHESIS:
A) Etymology & Cross-Cultural Word Synchronization — Trace origin and evolution of key
   terms across cultures. Identify synchronized terms (consistent meaning across eras).
   Flag anachronistic or culturally dissonant language.

B) Veritas Audit — Validate factual assertions by cross-referencing sources in at least
   5 languages from a minimum of 3 distinct cultural traditions over 200+ years.
   Output confidence scores for each major claim.

FIVE SECTIONS:
1. The Verdict — GREEN/YELLOW/RED signal with prima facie assessment and blocking points.
2. The Jurisdictional Compass — Heat-map of compliance status across regions
   (US, EU, UK, China, MENA, LatAm, Africa, ASEAN).
3. The Fiduciary & Tech Risk Barometer — Quantified 0-10 scores for financial and
   technical risks with narratives.
4. The Critical Path — Ordered actionable steps: licenses, clauses, filings, timelines.
5. The Plain English Translation — Board-ready summary preserving essential Latin legal
   terms (bona fide, sui generis, pari passu) with etymological notes.

LEXICAL PROTOCOLS:
- Precision over colloquialism
- Etymological anchoring for critical terms
- All lexical choices logged for audit trail

Respond ONLY with valid JSON matching the executive_summary schema.
"""

    @staticmethod
    def _build_synthesis_prompt(query: str, agent_summaries: List[Dict]) -> str:
        return f"""ORIGINAL QUERY:
{query}

AGENT DELIBERATIONS (28 specialists):
{json.dumps(agent_summaries, indent=2, ensure_ascii=False)}

Synthesize the above into the 5-section Executive Summary with Etymology audit,
Veritas audit, and Lexical Protocols applied. Output valid JSON only.
"""

    @staticmethod
    def _translation_system_prompt(lang: Dict[str, Any]) -> str:
        return f"""You are a legal and financial translation specialist for {lang['name']}
(ISO 639-1: {lang['code']}, script: {lang['script']}, direction: {lang['direction']}).

LEXICAL PROTOCOLS:
- {lang.get('lexical_notes', 'Follow standard legal terminology.')}
- Maintain precision of legal terminology — do not simplify or omit legal concepts.
- Preserve all Latin legal maxims in their original form.
- Use standardized legal lexicons (UN terminology databases, local bar association glossaries).
- Flag any terms that lack precise equivalents in {lang['name']}.

Translate the entire executive summary into {lang['name']}. Maintain the JSON structure.
Output only the translated JSON.
"""

    @staticmethod
    def _build_translation_prompt(summary_text: str, lang: Dict[str, Any]) -> str:
        return f"""Translate the following Executive Summary into {lang['name']}.

CONTEXT: {lang.get('rationale', '')}

EXECUTIVE SUMMARY (English master copy):
{summary_text}

Output the complete translation as valid JSON, maintaining the same structure.
"""

    @staticmethod
    def _parse_agent_response(raw_text: str) -> Dict[str, Any]:
        """Parse JSON from agent response, handling code blocks."""
        text = raw_text.strip()
        if text.startswith("```"):
            text = text.split("\n", 1)[1] if "\n" in text else text[3:]
            if text.endswith("```"):
                text = text[:-3]
            text = text.strip()
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            return {
                "relevance_score": 0.5,
                "confidence": 0.3,
                "assessment": {
                    "applicable": True,
                    "risk_level": "medium",
                    "key_issues": ["Response could not be parsed as JSON"],
                    "raw_text": raw_text[:500],
                },
            }

    @staticmethod
    def _parse_synthesis_response(raw_text: str) -> Dict[str, Any]:
        """Parse JSON from synthesis response."""
        text = raw_text.strip()
        if text.startswith("```"):
            text = text.split("\n", 1)[1] if "\n" in text else text[3:]
            if text.endswith("```"):
                text = text[:-3]
            text = text.strip()
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            return {
                "verdict": {
                    "signal": "YELLOW",
                    "prima_facie_assessment": "Synthesis parsing error — raw text preserved",
                    "confidence_level": 0.3,
                    "raw_text": raw_text[:2000],
                },
                "parse_error": True,
            }
