#!/usr/bin/env python3
"""Analyze real Rust/GerryChain traces under the Pulse 04 preregistration."""

from __future__ import annotations

import argparse
import csv
import json
import math
import random
from pathlib import Path

import numpy as np
from scipy.stats import ks_2samp


METRICS = ["cut_fraction", "democratic_seats_2016", "democratic_seats_2020"]


def rhat(chains: list[list[float]]) -> float:
    matrix = np.asarray(chains, dtype=float)
    if matrix.shape[0] < 2 or matrix.shape[1] < 2:
        return float("nan")
    chain_means = matrix.mean(axis=1)
    within = matrix.var(axis=1, ddof=1).mean()
    between = matrix.shape[1] * chain_means.var(ddof=1)
    if within < 1e-15:
        return 1.0 if between < 1e-15 else float("inf")
    variance = ((matrix.shape[1] - 1) / matrix.shape[1]) * within + between / matrix.shape[1]
    return math.sqrt(variance / within)


def ess(trace: list[float]) -> float:
    values = np.asarray(trace, dtype=float)
    n = len(values)
    if n < 4:
        return float(n)
    centered = values - values.mean()
    variance = np.dot(centered, centered) / n
    if variance < 1e-15:
        return float(n)
    autocorr = []
    for lag in range(1, n // 2):
        value = np.dot(centered[:-lag], centered[lag:]) / (n * variance)
        autocorr.append(value)
    total = 0.0
    for index in range(0, len(autocorr) - 1, 2):
        pair = autocorr[index] + autocorr[index + 1]
        if pair <= 0:
            break
        total += pair
    return min(float(n), n / (1 + 2 * total))


def hamming_diagnostics(snapshots: list[dict], burn_in: int, max_lag: int = 20) -> dict:
    parts = [row["assignment"] for row in snapshots if row["step"] >= burn_in]
    if len(parts) < 2:
        return {"snapshot_count": len(parts), "mean_distance_by_lag": [], "tau_int": None}
    distances = [0.0]
    for lag in range(1, min(max_lag, len(parts) - 1) + 1):
        per_pair = []
        for index in range(len(parts) - lag):
            left, right = parts[index], parts[index + lag]
            per_pair.append(sum(a != b for a, b in zip(left, right)) / len(left))
        distances.append(float(np.mean(per_pair)))
    tau = 1.0
    for distance in distances[1:]:
        correlation = 1.0 - distance
        if correlation <= 0:
            break
        tau += 2 * correlation
    return {
        "snapshot_count": len(parts),
        "mean_distance_by_lag": distances,
        "tau_int": tau,
    }


def midrank_percentile(samples: list[float], benchmark: float) -> float:
    lower = sum(value < benchmark for value in samples)
    equal = sum(value == benchmark for value in samples)
    return (lower + 0.5 * equal) / len(samples)


def chain_bootstrap_interval(
    chains: list[list[float]], benchmark: float, seed: int, iterations: int = 2000
) -> list[float]:
    rng = random.Random(seed)
    percentiles = []
    for _ in range(iterations):
        selected = [rng.choice(chains) for _ in chains]
        samples = [value for chain in selected for value in chain]
        percentiles.append(midrank_percentile(samples, benchmark))
    return [float(np.quantile(percentiles, 0.025)), float(np.quantile(percentiles, 0.975))]


def analyze_trace(payload: dict, burn_in: int, bootstrap_seed: int) -> dict:
    chain_metrics = []
    for chain in payload["chain_traces"]:
        chain_metrics.append([row for row in chain["metrics"] if row["step"] > burn_in])
    result = {
        "implementation": payload["implementation"],
        "implementation_version": payload["implementation_version"],
        "tree_sampler": payload["tree_sampler"],
        "steps_per_chain": payload["steps_per_chain"],
        "chains": payload["chains"],
        "population_tolerance": payload["population_tolerance"],
        "acceptance_rate": float(
            np.mean(
                [
                    row["accepted"]
                    for chain in payload["chain_traces"]
                    for row in chain["metrics"]
                ]
            )
        ),
        "post_burn_in_samples": sum(len(chain) for chain in chain_metrics),
        "metrics": {},
        "hamming": [
            hamming_diagnostics(chain["snapshots"], burn_in)
            for chain in payload["chain_traces"]
        ],
    }
    for metric in METRICS:
        chains = [[float(row[metric]) for row in chain] for chain in chain_metrics]
        pooled = [value for chain in chains for value in chain]
        benchmark = float(payload["baseline"][metric])
        result["metrics"][metric] = {
            "benchmark": benchmark,
            "mean": float(np.mean(pooled)),
            "std": float(np.std(pooled, ddof=1)),
            "quantiles": {
                "q01": float(np.quantile(pooled, 0.01)),
                "q05": float(np.quantile(pooled, 0.05)),
                "q50": float(np.quantile(pooled, 0.50)),
                "q95": float(np.quantile(pooled, 0.95)),
                "q99": float(np.quantile(pooled, 0.99)),
            },
            "r_hat": rhat(chains),
            "ess_per_chain": [ess(chain) for chain in chains],
            "ess_pooled": ess(pooled),
            "benchmark_percentile": midrank_percentile(pooled, benchmark),
            "benchmark_percentile_ci95": chain_bootstrap_interval(
                chains, benchmark, bootstrap_seed
            ),
        }
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--states", nargs="+", required=True)
    parser.add_argument("--burn-in", type=int, default=500)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--summary-csv", type=Path, required=True)
    args = parser.parse_args()

    output = {
        "analysis_version": "g-real-ensemble-analysis v1",
        "burn_in": args.burn_in,
        "primary_election": "2020-president",
        "excluded_primary_metrics": {
            "democratic_seats_2016": "Incomplete 2010-to-2020 tract coverage; retained only as a diagnostic."
        },
        "states": {},
    }
    rows = []
    for state_index, state in enumerate(args.states):
        state_root = args.root / state.lower()
        rust_payload = json.loads((state_root / "rust-trace.json").read_text(encoding="utf-8"))
        gerry_payload = json.loads(
            (state_root / "gerrychain-trace.json").read_text(encoding="utf-8")
        )
        rust_result = analyze_trace(rust_payload, args.burn_in, 4100 + state_index)
        gerry_result = analyze_trace(gerry_payload, args.burn_in, 5100 + state_index)
        cross_tool = {}
        for metric in METRICS:
            rust_values = [
                float(row[metric])
                for chain in rust_payload["chain_traces"]
                for row in chain["metrics"]
                if row["step"] > args.burn_in
            ]
            gerry_values = [
                float(row[metric])
                for chain in gerry_payload["chain_traces"]
                for row in chain["metrics"]
                if row["step"] > args.burn_in
            ]
            statistic, p_value = ks_2samp(rust_values, gerry_values)
            cross_tool[metric] = {
                "mean_difference_rust_minus_gerrychain": float(
                    np.mean(rust_values) - np.mean(gerry_values)
                ),
                "std_difference_rust_minus_gerrychain": float(
                    np.std(rust_values, ddof=1) - np.std(gerry_values, ddof=1)
                ),
                "ks_statistic": float(statistic),
                "ks_p_value": float(p_value),
            }
        coverage = rust_payload["unmatched_geoids"]
        state_result = {
            "election_coverage": coverage,
            "rust": rust_result,
            "gerrychain": gerry_result,
            "cross_tool": cross_tool,
        }
        output["states"][state.upper()] = state_result
        for implementation, result in [("rust", rust_result), ("gerrychain", gerry_result)]:
            for metric, metric_result in result["metrics"].items():
                rows.append(
                    {
                        "state": state.upper(),
                        "implementation": implementation,
                        "metric": metric,
                        "benchmark": metric_result["benchmark"],
                        "mean": metric_result["mean"],
                        "std": metric_result["std"],
                        "r_hat": metric_result["r_hat"],
                        "ess_pooled": metric_result["ess_pooled"],
                        "percentile": metric_result["benchmark_percentile"],
                        "ci95_low": metric_result["benchmark_percentile_ci95"][0],
                        "ci95_high": metric_result["benchmark_percentile_ci95"][1],
                        "acceptance_rate": result["acceptance_rate"],
                    }
                )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2), encoding="utf-8")
    with args.summary_csv.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    main()
