from pathlib import Path
import sys

import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from analyze_neutral_algorithm_family_bakeoff import (
    BakeoffError,
    canonical_assignment,
    common_invariants,
    optimal_overlap,
)
from run_neutral_algorithm_family_bakeoff import command_template


def test_canonical_assignment_sorts_units_and_removes_label_arbitrariness() -> None:
    raw = {"11": 9, "2": 4, "10": 9, "1": 4}

    assert canonical_assignment(raw) == {"1": 1, "2": 1, "10": 2, "11": 2}


def test_optimal_overlap_ignores_district_label_permutation() -> None:
    left = {"1": 1, "2": 1, "3": 2, "4": 2}
    right = {"1": 8, "2": 8, "3": 7, "4": 7}

    assert optimal_overlap(left, right) == {
        "matched_units": 4,
        "different_units": 0,
        "matched_unit_rate": 1.0,
    }


def test_optimal_overlap_reports_real_assignment_difference() -> None:
    left = {"1": 1, "2": 1, "3": 2, "4": 2}
    right = {"1": 8, "2": 7, "3": 8, "4": 7}

    result = optimal_overlap(left, right)

    assert result["matched_units"] == 2
    assert result["different_units"] == 2
    assert result["matched_unit_rate"] == 0.5


def test_optimal_overlap_rejects_different_unit_universes() -> None:
    with pytest.raises(BakeoffError, match="assignment universes differ"):
        optimal_overlap({"1": 1}, {"2": 1})


def test_common_invariants_detect_mismatch() -> None:
    base = {
        "status": "pass",
        "execution_status": "pass",
        "adjacency_sha256": "same",
        "binary_sha256": "same",
        "population_source": "total",
        "balance_tolerance_pct": 0.5,
        "units": 10,
        "districts": 2,
        "ufactor": 5,
        "niter": 100,
        "alpha_county": 0.0,
        "directional_lambda": 0.0,
    }
    changed = {**base, "units": 11}

    assert common_invariants([base, base])["pass"] is True
    assert common_invariants([base, changed])["pass"] is False


def test_command_template_binds_frozen_controls_and_structure() -> None:
    command = command_template("prime-factor")

    assert command[command.index("--structure") + 1] == "prime-factor"
    assert command[command.index("--partition-mode") + 1] == "edge-weighted"
    assert command[command.index("--weights-override") + 1] == "geographic"
    assert command[command.index("--search") + 1] == "single"
    assert command[command.index("--seed") + 1] == "0"
