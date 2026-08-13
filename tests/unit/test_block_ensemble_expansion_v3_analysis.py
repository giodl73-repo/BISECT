import json
from pathlib import Path
import sys

import numpy as np

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from analyze_block_ensemble_expansion_v3 import (
    analyze_kernel,
    canonicalize_numbers,
    hamming_diagnostics,
    label_aligned_hamming,
    split_rhat,
)


def test_numeric_canonicalization_removes_last_bit_platform_noise() -> None:
    left = canonicalize_numbers({"value": 1.23456789012341})
    right = canonicalize_numbers({"value": 1.23456789012349})
    assert left == right == {"value": 1.23456789012}


def test_label_aligned_hamming_ignores_a_pure_label_swap() -> None:
    left = np.asarray([0, 0, 1, 1], dtype=np.uint8)
    right = np.asarray([1, 1, 0, 0], dtype=np.uint8)
    assert label_aligned_hamming(left, right, districts=2) == 0.0


def test_label_aligned_hamming_finds_partial_best_match() -> None:
    left = np.asarray([0, 0, 1, 1], dtype=np.uint8)
    right = np.asarray([1, 0, 0, 0], dtype=np.uint8)
    assert label_aligned_hamming(left, right, districts=2) == 0.25


def test_hamming_discards_snapshot_at_burn_in_boundary() -> None:
    snapshots = [
        {"step": 500, "assignment": [0, 0]},
        {"step": 510, "assignment": [0, 1]},
        {"step": 520, "assignment": [1, 0]},
    ]
    result = hamming_diagnostics(snapshots, burn_in=500, districts=2)
    assert result["snapshot_count"] == 2
    assert result["mean_distance_by_lag"] == [0.0, 0.0]


def test_split_rhat_flags_separated_chains() -> None:
    assert split_rhat([[float(index)] * 20 for index in range(4)]) > 1.05


def test_kernel_decisions_are_native_json_booleans() -> None:
    trace = {
        "sampler": "wilson",
        "chains": 4,
        "steps_per_chain": 4,
        "districts": 2,
        "baseline": {"cut_fraction": 0.5, "weighted_boundary_cut": 2.0},
        "chain_traces": [
            {
                "metrics": [
                    {
                        "step": step,
                        "accepted": True,
                        "cut_fraction": 0.5,
                        "weighted_boundary_cut": 2.0,
                        "max_population_deviation": 0.001,
                    }
                    for step in range(1, 5)
                ],
                "snapshots": [],
            }
            for _ in range(4)
        ],
    }
    result, _ = analyze_kernel(trace, burn_in=0)
    assert type(result["metrics"]["cut_fraction"]["converged"]) is bool
    assert type(result["metrics"]["cut_fraction"]["extreme_tail_claim_authorized"]) is bool
    json.dumps(result)
