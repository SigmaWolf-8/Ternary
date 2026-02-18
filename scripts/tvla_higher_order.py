#!/usr/bin/env python3
"""
Higher-Order TVLA (Test Vector Leakage Assessment) -- Task 8B.5

Performs Welch's t-test at 1st, 2nd, and 3rd statistical orders
on power simulation traces from Verilator to validate that the
DOM masking implementation provides the claimed protection order.

Inputs:
  - VCD/CSV traces from Verilator toggle-count simulation
  - Fixed vs. random input classification

Outputs:
  - Per-order t-test values at each sample point
  - PASS/FAIL determination (threshold: |t| < 4.5)
  - Plot of t-test traces (optional)

Usage:
  python3 tvla_higher_order.py \\
    --traces traces/mask_unit_power.csv \\
    --labels traces/mask_unit_labels.csv \\
    --max-order 3 \\
    --num-traces 100000 \\
    --output results/tvla_report.json
"""

import argparse
import json
import sys
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple

T_THRESHOLD = 4.5


def load_traces(trace_path: str, label_path: str, max_traces: int
               ) -> Tuple[np.ndarray, np.ndarray]:
    """
    Load power simulation traces and their fixed/random labels.

    trace_path: CSV file with shape (num_traces, num_samples)
                Each row is one execution's power trace.
    label_path: CSV file with shape (num_traces,)
                0 = random input, 1 = fixed input
    """
    print(f"Loading traces from {trace_path}...")
    traces = np.loadtxt(trace_path, delimiter=',', max_rows=max_traces)
    labels = np.loadtxt(label_path, delimiter=',', max_rows=max_traces, dtype=int)

    assert traces.shape[0] == labels.shape[0], \
        f"Trace/label count mismatch: {traces.shape[0]} vs {labels.shape[0]}"

    print(f"  Loaded {traces.shape[0]} traces x {traces.shape[1]} samples")
    print(f"  Fixed: {np.sum(labels == 1)}, Random: {np.sum(labels == 0)}")

    return traces, labels


def compute_central_moments(traces: np.ndarray, labels: np.ndarray,
                            order: int) -> Tuple[np.ndarray, np.ndarray]:
    """
    Compute centralised statistical moments for fixed and random groups.

    For order d:
      moment_d(X) = E[(X - E[X])^d]

    First-order (d=1): mean (standard TVLA)
    Second-order (d=2): variance (detects 2nd-order leakage)
    Third-order (d=3): skewness (detects 3rd-order leakage)
    """
    fixed_mask = labels == 1
    random_mask = labels == 0

    t_fixed = traces[fixed_mask]
    t_random = traces[random_mask]

    if order == 1:
        return t_fixed, t_random
    else:
        f_centered = t_fixed - np.mean(t_fixed, axis=1, keepdims=True)
        r_centered = t_random - np.mean(t_random, axis=1, keepdims=True)

        f_moments = np.power(f_centered, order)
        r_moments = np.power(r_centered, order)

        return f_moments, r_moments


def welch_t_test(group_a: np.ndarray, group_b: np.ndarray) -> np.ndarray:
    """
    Compute Welch's t-test statistic at each sample point.

    t = (mean_A - mean_B) / sqrt(var_A/n_A + var_B/n_B)

    Returns array of t-values with shape (num_samples,).
    """
    n_a = group_a.shape[0]
    n_b = group_b.shape[0]

    mean_a = np.mean(group_a, axis=0)
    mean_b = np.mean(group_b, axis=0)

    var_a = np.var(group_a, axis=0, ddof=1)
    var_b = np.var(group_b, axis=0, ddof=1)

    denominator = np.sqrt(var_a / n_a + var_b / n_b)
    denominator = np.maximum(denominator, 1e-30)

    t_values = (mean_a - mean_b) / denominator

    return t_values


def run_tvla(traces: np.ndarray, labels: np.ndarray,
             max_order: int) -> Dict[int, Dict]:
    """
    Run TVLA at all orders from 1 to max_order.

    Returns dict mapping order -> {t_values, max_t, pass/fail}.
    """
    results = {}

    for order in range(1, max_order + 1):
        print(f"\n{'='*60}")
        print(f"  Order {order} TVLA Analysis")
        print(f"{'='*60}")

        group_a, group_b = compute_central_moments(traces, labels, order)
        t_values = welch_t_test(group_a, group_b)

        max_abs_t = float(np.max(np.abs(t_values)))
        passed = max_abs_t < T_THRESHOLD

        print(f"  Max |t| = {max_abs_t:.4f}")
        print(f"  Threshold = {T_THRESHOLD}")
        print(f"  Result: {'PASS' if passed else 'FAIL'}")

        if not passed:
            failing_samples = np.where(np.abs(t_values) >= T_THRESHOLD)[0]
            print(f"  Failing sample points: {len(failing_samples)}")
            print(f"  First 10 failing indices: {failing_samples[:10].tolist()}")

        results[order] = {
            "max_abs_t": max_abs_t,
            "threshold": T_THRESHOLD,
            "passed": passed,
            "num_traces": int(traces.shape[0]),
            "num_samples": int(traces.shape[1]),
            "t_values": t_values.tolist(),
        }

    return results


def generate_report(results: Dict[int, Dict], output_path: str) -> None:
    """Generate JSON report of TVLA results."""
    report = {
        "tool": "XPlenum TVLA Higher-Order Analysis",
        "task": "8B.5",
        "threshold": T_THRESHOLD,
        "overall_pass": all(r["passed"] for r in results.values()),
        "orders": {}
    }

    for order, data in results.items():
        report["orders"][str(order)] = {
            "max_abs_t": data["max_abs_t"],
            "passed": data["passed"],
            "num_traces": data["num_traces"],
        }

    with open(output_path, 'w') as f:
        json.dump(report, f, indent=2)

    print(f"\nReport saved to {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Higher-Order TVLA for XPlenum")
    parser.add_argument("--traces", required=True, help="CSV trace file")
    parser.add_argument("--labels", required=True, help="CSV label file (0=random, 1=fixed)")
    parser.add_argument("--max-order", type=int, default=3, help="Maximum TVLA order")
    parser.add_argument("--num-traces", type=int, default=100000, help="Max traces to load")
    parser.add_argument("--output", default="tvla_report.json", help="Output report path")
    args = parser.parse_args()

    traces, labels = load_traces(args.traces, args.labels, args.num_traces)
    results = run_tvla(traces, labels, args.max_order)
    generate_report(results, args.output)

    overall = all(r["passed"] for r in results.values())
    print(f"\n{'='*60}")
    print(f"  OVERALL: {'PASS' if overall else 'FAIL'}")
    print(f"{'='*60}")
    sys.exit(0 if overall else 1)


if __name__ == "__main__":
    main()
