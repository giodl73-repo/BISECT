from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/run_certified_metis_ensemble.py"
SPEC = importlib.util.spec_from_file_location("certified_metis_ensemble", MODULE_PATH)
assert SPEC and SPEC.loader
ENSEMBLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ENSEMBLE)


def test_objective_ranking_is_lexicographic() -> None:
    better_population = {
        "status": "accepted",
        "objective": {
            "max_population_deviation_scaled": 0,
            "total_population_deviation_scaled": 0,
            "weighted_boundary_cut": 999,
        },
    }
    better_boundary = {
        "status": "accepted",
        "objective": {
            "max_population_deviation_scaled": 1,
            "total_population_deviation_scaled": 2,
            "weighted_boundary_cut": 10,
        },
    }
    assert ENSEMBLE.objective_key(better_population) < ENSEMBLE.objective_key(
        better_boundary
    )


def test_rejected_seed_ranks_after_accepted_seed() -> None:
    rejected = {"status": "rejected"}
    accepted = {
        "status": "accepted",
        "objective": {
            "max_population_deviation_scaled": 10,
            "total_population_deviation_scaled": 20,
            "weighted_boundary_cut": 30,
        },
    }
    assert ENSEMBLE.objective_key(accepted) < ENSEMBLE.objective_key(rejected)
