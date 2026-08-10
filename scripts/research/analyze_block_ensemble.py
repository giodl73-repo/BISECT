#!/usr/bin/env python3
"""Analyze governed Stage 1 block-level ensemble traces."""

from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path

import numpy as np
from scipy.stats import ks_2samp

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from analyze_real_ensemble import ess, hamming_diagnostics, midrank_percentile, rhat

METRICS = ("cut_fraction", "weighted_boundary_cut")


def fail(message: str) -> None:
    raise ValueError(message)


def validate_governed_trace(trace: dict, sampler: str) -> None:
    expected = {
        "schema_version": "nrs-block-ensemble-trace-v1",
        "status": "complete",
        "execution_class": "governed-stage1",
        "state": "RI",
        "year": 2020,
        "units": 25649,
        "districts": 2,
        "sampler": sampler,
        "chains": 4,
        "steps_per_chain": 2000,
        "population_tolerance": 0.005,
        "base_seed": 20260810,
        "snapshot_stride": 10,
    }
    for key, value in expected.items():
        if trace.get(key) != value:
            fail(f"{sampler} trace {key} drift")
    if len(trace["chain_traces"]) != 4:
        fail(f"{sampler} must contain four chains")
    if [chain["chain_index"] for chain in trace["chain_traces"]] != list(range(4)):
        fail(f"{sampler} chain index drift")
    if len({chain["seed"] for chain in trace["chain_traces"]}) != 4:
        fail(f"{sampler} chain seeds are not unique")
    for chain in trace["chain_traces"]:
        metrics = chain["metrics"]
        if [row["step"] for row in metrics] != list(range(1, 2001)):
            fail(f"{sampler} metric step sequence drift")
        if [row["step"] for row in chain["snapshots"]] != list(range(10, 2001, 10)):
            fail(f"{sampler} snapshot schedule drift")
        if any(row["max_population_deviation"] > 0.005 for row in metrics):
            fail(f"{sampler} population tolerance failure")


def split_rhat(chains: list[list[float]]) -> float:
    halves = []
    for chain in chains:
        midpoint = len(chain) // 2
        halves.extend((chain[:midpoint], chain[-midpoint:]))
    return rhat(halves)


def analyze_kernel(trace: dict, burn_in: int) -> dict:
    post = [
        [row for row in chain["metrics"] if row["step"] > burn_in]
        for chain in trace["chain_traces"]
    ]
    result = {
        "sampler": trace["sampler"],
        "chains": trace["chains"],
        "steps_per_chain": trace["steps_per_chain"],
        "post_burn_in_samples": sum(map(len, post)),
        "acceptance_rate": float(
            np.mean(
                [row["accepted"] for chain in trace["chain_traces"] for row in chain["metrics"]]
            )
        ),
        "maximum_population_deviation": max(
            row["max_population_deviation"]
            for chain in trace["chain_traces"]
            for row in chain["metrics"]
        ),
        "hamming": [
            hamming_diagnostics(chain["snapshots"], burn_in)
            for chain in trace["chain_traces"]
        ],
        "metrics": {},
    }
    convergence = []
    for metric in METRICS:
        chains = [[float(row[metric]) for row in chain] for chain in post]
        pooled = [value for chain in chains for value in chain]
        metric_rhat = split_rhat(chains)
        pooled_ess = ess(pooled)
        # NumPy scalar comparisons produce np.bool_, which the standard JSON
        # encoder deliberately does not coerce.  Keep the evidence schema to
        # native JSON types at the point where the decision is made.
        converged = bool(metric_rhat < 1.05 and pooled_ess >= 100)
        convergence.append(converged)
        benchmark = float(trace["baseline"][metric])
        result["metrics"][metric] = {
            "benchmark": benchmark,
            "mean": float(np.mean(pooled)),
            "std": float(np.std(pooled, ddof=1)),
            "quantiles": {
                key: float(np.quantile(pooled, quantile))
                for key, quantile in (("q01", 0.01), ("q05", 0.05), ("q50", 0.5), ("q95", 0.95), ("q99", 0.99))
            },
            "split_r_hat": metric_rhat,
            "ess_per_chain": [ess(chain) for chain in chains],
            "ess_pooled": pooled_ess,
            "benchmark_percentile": midrank_percentile(pooled, benchmark),
            "converged": converged,
            "extreme_tail_claim_authorized": bool(pooled_ess >= 1000),
        }
    result["converged"] = all(convergence)
    return result


def values(trace: dict, metric: str, burn_in: int) -> list[float]:
    return [
        float(row[metric])
        for chain in trace["chain_traces"]
        for row in chain["metrics"]
        if row["step"] > burn_in
    ]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--wilson", type=Path, required=True)
    parser.add_argument("--kruskal", type=Path, required=True)
    parser.add_argument("--burn-in", type=int, default=500)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--summary-csv", type=Path, required=True)
    args = parser.parse_args()
    traces = {
        "wilson": json.loads(args.wilson.read_text(encoding="utf-8")),
        "kruskal": json.loads(args.kruskal.read_text(encoding="utf-8")),
    }
    for sampler, trace in traces.items():
        validate_governed_trace(trace, sampler)
    if traces["wilson"]["baseline"] != traces["kruskal"]["baseline"]:
        fail("kernel baselines differ")
    kernels = {name: analyze_kernel(trace, args.burn_in) for name, trace in traces.items()}
    cross_kernel = {}
    for metric in METRICS:
        left = values(traces["wilson"], metric, args.burn_in)
        right = values(traces["kruskal"], metric, args.burn_in)
        statistic, p_value = ks_2samp(left, right)
        cross_kernel[metric] = {
            "mean_difference_wilson_minus_kruskal": float(np.mean(left) - np.mean(right)),
            "ks_statistic": float(statistic),
            "ks_p_value_descriptive_only": float(p_value),
        }
    output = {
        "schema_version": "nrs-block-ensemble-analysis-v1",
        "protocol_id": "nrs-v0.3-block-ensemble-gate-v1",
        "state": "RI",
        "burn_in": args.burn_in,
        "kernels": kernels,
        "cross_kernel": cross_kernel,
        "governed_trace_set_valid": True,
        "stage1_converged": all(result["converged"] for result in kernels.values()),
        "deterministic_replay_required": True,
        "claim_boundary": "Rhode Island block-graph kernel sensitivity only; no national, neutrality, legal, or mixing-proof claim.",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
    rows = []
    for sampler, result in kernels.items():
        for metric, record in result["metrics"].items():
            rows.append({"sampler": sampler, "metric": metric, **{key: record[key] for key in ("benchmark", "mean", "std", "split_r_hat", "ess_pooled", "benchmark_percentile", "converged")}})
    with args.summary_csv.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    main()
