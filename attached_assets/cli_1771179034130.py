#!/usr/bin/env python3
"""
PlenumNET 28-Agent Compliance Engine — CLI Entry Point
═══════════════════════════════════════════════════════

Usage:
    python -m kernel.cli "Your compliance query here"
    python -m kernel.cli --query "query" --output results.json
    python -m kernel.cli --query "query" --no-translations
    python -m kernel.cli --verify  (run mathematical verification only)

Environment:
    ANTHROPIC_API_KEY — Required for API calls

Applied Physics Division — Capomastro Holdings Ltd.
"""

import argparse
import asyncio
import json
import os
import sys
from pathlib import Path

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent))


def run_verification():
    """Run mathematical verification of Tribonacci properties."""
    from kernel.core.tribonacci_scheduler import (
        tribonacci, tribonacci_mod, agent_permutation, verify_permutation,
        verify_coprimality, schedule_agents, generate_query_hash,
        TERNARY_RADIAN, NUM_AGENTS, FULL_CIRCLE, CONVOLUTION_KERNEL,
    )

    print("=" * 70)
    print("PlenumNET 28-Agent Compliance Engine — Mathematical Verification")
    print("Applied Physics Division — Capomastro Holdings Ltd.")
    print("=" * 70)

    print(f"\n  Ternary Radian (T_7):  {TERNARY_RADIAN}")
    print(f"  Agents (28-fold):      {NUM_AGENTS}")
    print(f"  Full Circle:           {FULL_CIRCLE} = 111111₃")
    print(f"  Convolution Kernel:    {CONVOLUTION_KERNEL}")

    # Verify permutation
    perm = agent_permutation()
    perm_ok = verify_permutation()
    print(f"\n  13-step permutation:   {perm}")
    print(f"  Covers all 28:         {'✓' if perm_ok else '✗'}")

    # Verify coprimality
    coprime_ok = verify_coprimality()
    print(f"  gcd(13, 28) = 1:       {'✓' if coprime_ok else '✗'}")

    # Verify key sequence values
    assert tribonacci(7) == 13, "T_7 ≠ 13"
    assert tribonacci(8) == 24, "T_8 ≠ 24"
    assert tribonacci(9) == 44, "T_9 ≠ 44"
    assert tribonacci(44) == 80641778674, "T_44 ≠ 80,641,778,674"
    print(f"\n  T_7 = {tribonacci(7)}     ✓")
    print(f"  T_8 = {tribonacci(8)}     ✓")
    print(f"  T_9 = {tribonacci(9)}     ✓ (the hook)")
    print(f"  T_44 = {tribonacci(44):,}  ✓ (the hook target)")

    # Verify scheduling
    order = schedule_agents()
    print(f"\n  Agent execution order:  {order[:10]}... ({len(order)} total)")

    # Hash example
    h = generate_query_hash("test compliance query")
    print(f"  Query hash example:    {h}")

    print("\n" + "=" * 70)
    all_pass = perm_ok and coprime_ok
    print(f"  ALL VERIFICATIONS {'PASSED ✓' if all_pass else 'FAILED ✗'}")
    print("=" * 70)
    return 0 if all_pass else 1


async def run_query(query: str, output_path: str | None = None,
                     include_translations: bool = True):
    """Run a compliance query through the full pipeline."""
    from kernel.core.engine import ComplianceEngine

    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("ERROR: ANTHROPIC_API_KEY environment variable is required.")
        print("Set it with: export ANTHROPIC_API_KEY='your-key-here'")
        return 1

    print("=" * 70)
    print("PlenumNET 28-Agent International Compliance Engine")
    print("Applied Physics Division — Capomastro Holdings Ltd.")
    print("=" * 70)
    print(f"\nQuery: {query}\n")

    engine = ComplianceEngine(api_key=api_key)
    result = await engine.process_query(
        query,
        include_translations=include_translations,
    )

    # Output
    output_json = result.to_json(indent=2)

    if output_path:
        output_file = Path(output_path)
        output_file.write_text(output_json, encoding="utf-8")
        print(f"\nResults written to: {output_file}")
    else:
        # Print summary to stdout
        summary = result.to_dict()
        l2 = summary.get("layer2_executive_summary", {})

        verdict = l2.get("verdict", {})
        signal = verdict.get("signal", "UNKNOWN")
        print(f"\n{'═' * 70}")
        print(f"  VERDICT: {signal}")
        print(f"  Confidence: {verdict.get('confidence_level', 'N/A')}")
        print(f"  Prima Facie: {verdict.get('prima_facie_assessment', 'N/A')[:200]}")
        print(f"{'═' * 70}")

        print(f"\n  Processing time: {summary['metadata']['processing_time_ms']}ms")
        print(f"  Tribonacci hash: {summary['metadata']['tribonacci_hash']}")
        print(f"  Agents consulted: {len(summary['layer1_deliberations'])}")
        if summary.get("layer3_translations"):
            print(f"  Languages: {len(summary['layer3_translations'])}")

    return 0


def main():
    parser = argparse.ArgumentParser(
        description="PlenumNET 28-Agent International Compliance Engine",
        epilog="Applied Physics Division — Capomastro Holdings Ltd.",
    )
    parser.add_argument(
        "query", nargs="?", default=None,
        help="Compliance query to analyze",
    )
    parser.add_argument(
        "--query", "-q", dest="query_flag",
        help="Compliance query (alternative to positional argument)",
    )
    parser.add_argument(
        "--output", "-o",
        help="Output file path (JSON). Prints summary to stdout if not specified.",
    )
    parser.add_argument(
        "--no-translations", action="store_true",
        help="Skip Layer 3 translations (faster, cheaper)",
    )
    parser.add_argument(
        "--verify", action="store_true",
        help="Run mathematical verification only (no API calls)",
    )

    args = parser.parse_args()

    if args.verify:
        sys.exit(run_verification())

    query = args.query or args.query_flag
    if not query:
        parser.print_help()
        print("\nERROR: Please provide a query.")
        sys.exit(1)

    exit_code = asyncio.run(
        run_query(
            query,
            output_path=args.output,
            include_translations=not args.no_translations,
        )
    )
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
