#!/usr/bin/env python3
"""Compute the frozen v3 State-by-kernel diagnostics from retained traces."""

from __future__ import annotations

import argparse
import csv
import gzip
import json
import sys
from pathlib import Path

import numpy as np
from scipy.optimize import linear_sum_assignment
from scipy.stats import ks_2samp

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from analyze_real_ensemble import ess, midrank_percentile, rhat
from run_block_ensemble_expansion_v3 import BASE_SEED, PROTOCOL_ID, validate_trace

STATES = ("NH", "NM", "GA")
SAMPLERS = ("wilson", "kruskal")
METRICS = ("cut_fraction", "weighted_boundary_cut")
BURN_IN = 500
MAX_HAMMING_LAG = 20


def canonicalize_numbers(value):
    """Remove non-scientific platform noise while preserving decisions."""
    if isinstance(value, float):
        return float(format(value, ".12g"))
    if isinstance(value, list):
        return [canonicalize_numbers(item) for item in value]
    if isinstance(value, dict):
        return {key: canonicalize_numbers(item) for key, item in value.items()}
    return value


def load_trace(path: Path) -> dict:
    with gzip.open(path, "rt", encoding="utf-8") as handle:
        return json.load(handle)


def split_rhat(chains: list[list[float]]) -> float:
    halves: list[list[float]] = []
    for chain in chains:
        midpoint = len(chain) // 2
        halves.extend((chain[:midpoint], chain[-midpoint:]))
    return rhat(halves)


def label_aligned_hamming(left: np.ndarray, right: np.ndarray, districts: int) -> float:
    """Return minimum Hamming distance over all right-label permutations."""
    if left.shape != right.shape or left.ndim != 1 or left.size == 0:
        raise ValueError("Hamming assignments must be nonempty vectors of equal length")
    left_labels, left_codes = np.unique(left, return_inverse=True)
    right_labels, right_codes = np.unique(right, return_inverse=True)
    if len(left_labels) > districts or len(right_labels) > districts:
        raise ValueError("Hamming assignment exceeds the frozen district count")
    cells = np.zeros((districts, districts), dtype=np.int64)
    np.add.at(cells, (left_codes, right_codes), 1)
    rows, columns = linear_sum_assignment(cells, maximize=True)
    matches = int(cells[rows, columns].sum())
    return (left.size - matches) / left.size


def hamming_diagnostics(
    snapshots: list[dict], burn_in: int, districts: int, max_lag: int = MAX_HAMMING_LAG
) -> dict:
    retained = [row["assignment"] for row in snapshots if row["step"] > burn_in]
    if len(retained) < 2:
        return {"snapshot_count": len(retained), "mean_distance_by_lag": [], "tau_int": None}
    assignments = np.asarray(retained, dtype=np.uint8)
    distances = [0.0]
    for lag in range(1, min(max_lag, len(assignments) - 1) + 1):
        distances.append(
            float(
                np.mean(
                    [
                        label_aligned_hamming(left, right, districts)
                        for left, right in zip(assignments[:-lag], assignments[lag:])
                    ]
                )
            )
        )
    tau = 1.0
    for distance in distances[1:]:
        correlation = 1.0 - distance
        if correlation <= 0:
            break
        tau += 2.0 * correlation
    return canonicalize_numbers({
        "snapshot_count": len(retained),
        "mean_distance_by_lag": distances,
        "tau_int": tau,
        "label_alignment": "minimum Hamming over right-label permutations",
    })


def analyze_kernel(trace: dict, burn_in: int) -> tuple[dict, dict[str, list[float]]]:
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
            np.mean([row["accepted"] for chain in trace["chain_traces"] for row in chain["metrics"]])
        ),
        "maximum_population_deviation": max(
            row["max_population_deviation"]
            for chain in trace["chain_traces"]
            for row in chain["metrics"]
        ),
        "hamming": [
            hamming_diagnostics(chain["snapshots"], burn_in, trace["districts"])
            for chain in trace["chain_traces"]
        ],
        "metrics": {},
    }
    pooled_values: dict[str, list[float]] = {}
    convergence = []
    for metric in METRICS:
        chains = [[float(row[metric]) for row in chain] for chain in post]
        pooled = [value for chain in chains for value in chain]
        pooled_values[metric] = pooled
        metric_rhat = split_rhat(chains)
        pooled_ess = ess(pooled)
        converged = bool(metric_rhat < 1.05 and pooled_ess >= 100)
        convergence.append(converged)
        benchmark = float(trace["baseline"][metric])
        result["metrics"][metric] = {
            "benchmark": benchmark,
            "mean": float(np.mean(pooled)),
            "std": float(np.std(pooled, ddof=1)),
            "quantiles": {
                key: float(np.quantile(pooled, quantile))
                for key, quantile in (
                    ("q01", 0.01), ("q05", 0.05), ("q50", 0.5),
                    ("q95", 0.95), ("q99", 0.99),
                )
            },
            "split_r_hat": metric_rhat,
            "ess_per_chain": [ess(chain) for chain in chains],
            "ess_pooled": pooled_ess,
            "benchmark_percentile": midrank_percentile(pooled, benchmark),
            "converged": converged,
            "extreme_tail_claim_authorized": bool(pooled_ess >= 1000),
        }
    result["converged"] = all(convergence)
    return result, pooled_values


def analyze_package(package: Path, burn_in: int = BURN_IN) -> dict:
    states = {}
    for state in STATES:
        kernels = {}
        pooled = {}
        baselines = []
        for sampler in SAMPLERS:
            path = package / f"governed-{state.lower()}-{sampler}.json.gz"
            trace = load_trace(path)
            validate_trace(trace, state, sampler, "primary")
            baselines.append(trace["baseline"])
            kernels[sampler], pooled[sampler] = analyze_kernel(trace, burn_in)
            del trace
        if baselines[0] != baselines[1]:
            raise ValueError(f"{state} kernel baselines differ")
        cross_kernel = {}
        for metric in METRICS:
            left, right = pooled["wilson"][metric], pooled["kruskal"][metric]
            statistic, p_value = ks_2samp(left, right)
            cross_kernel[metric] = {
                "mean_difference_wilson_minus_kruskal": float(np.mean(left) - np.mean(right)),
                "std_difference_wilson_minus_kruskal": float(
                    np.std(left, ddof=1) - np.std(right, ddof=1)
                ),
                "ks_statistic": float(statistic),
                "ks_p_value_descriptive_only": float(p_value),
            }
        states[state] = {
            "kernels": kernels,
            "cross_kernel": cross_kernel,
            "state_converged": all(record["converged"] for record in kernels.values()),
        }
    return canonicalize_numbers({
        "schema_version": "nrs-block-ensemble-expansion-analysis-v3",
        "protocol_id": PROTOCOL_ID,
        "base_seed": BASE_SEED,
        "burn_in": burn_in,
        "states": states,
        "governed_trace_set_valid": True,
        "gate_passed": all(record["state_converged"] for record in states.values()),
        "tail_rule": "Statements outside the 1st--99th percentiles require relevant ESS >= 1000.",
        "claim_boundary": (
            "Bounded 2020 NH/NM/GA block-graph feasibility and State-specific "
            "Wilson-versus-Kruskal sensitivity only; no national, mixing, "
            "sampler-equivalence, neutrality, fairness, VRA, legal, polygon, or "
            "all-valid-plans claim."
        ),
    })


def summary_rows(analysis: dict) -> list[dict]:
    rows = []
    for state, state_record in analysis["states"].items():
        for sampler, kernel in state_record["kernels"].items():
            for metric, record in kernel["metrics"].items():
                rows.append(
                    {
                        "state": state,
                        "sampler": sampler,
                        "metric": metric,
                        **{
                            key: record[key]
                            for key in (
                                "benchmark", "mean", "std", "split_r_hat",
                                "ess_pooled", "benchmark_percentile", "converged",
                            )
                        },
                    }
                )
    return rows


def write_analysis(analysis: dict, output: Path, summary_csv: Path) -> None:
    output.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8", newline="\n")
    rows = summary_rows(analysis)
    with summary_csv.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--summary-csv", type=Path)
    args = parser.parse_args()
    output = args.output or args.package / "analysis.json"
    summary_csv = args.summary_csv or args.package / "summary.csv"
    write_analysis(analyze_package(args.package), output, summary_csv)


if __name__ == "__main__":
    main()
