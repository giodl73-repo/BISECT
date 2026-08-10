import json
from pathlib import Path
import sys

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from analyze_block_ensemble import analyze_kernel, split_rhat


def test_split_rhat_is_one_for_identical_constant_chains() -> None:
    assert split_rhat([[2.0] * 20 for _ in range(4)]) == 1.0


def test_split_rhat_flags_separated_chains() -> None:
    chains = [[float(index)] * 20 for index in range(4)]
    assert split_rhat(chains) > 1.05


def test_kernel_analysis_uses_json_native_booleans() -> None:
    trace = {
        "sampler": "wilson",
        "chains": 4,
        "steps_per_chain": 4,
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

    result = analyze_kernel(trace, burn_in=0)

    assert type(result["metrics"]["cut_fraction"]["converged"]) is bool
    assert type(result["metrics"]["cut_fraction"]["extreme_tail_claim_authorized"]) is bool
    json.dumps(result)
