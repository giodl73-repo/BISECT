from pathlib import Path
import itertools
import sys

import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from analyze_neutral_algorithm_family_bakeoff_national import (
    AT_LARGE_EXCLUSIONS,
    FULL_STATES,
    PILOT_STATES,
    STATE_BY_CODE,
    BakeoffError,
    canonical_assignment,
    maximum_weight_assignment,
    normalize_edge_cut,
    optimal_overlap,
)
from run_neutral_algorithm_family_bakeoff_national import (
    command_template,
    scheduled_states,
)
import run_neutral_algorithm_family_bakeoff_national as national_runner


def test_schedule_covers_44_unique_multidistrict_states() -> None:
    assert len(FULL_STATES) == 44
    assert len(set(FULL_STATES)) == 44
    assert len(AT_LARGE_EXCLUSIONS) == 6
    assert set(FULL_STATES).isdisjoint(AT_LARGE_EXCLUSIONS)
    assert all(STATE_BY_CODE[state][1] >= 2 for state in FULL_STATES)
    assert FULL_STATES[: len(PILOT_STATES)] == PILOT_STATES


def test_phase_schedule_is_frozen() -> None:
    assert scheduled_states("pilot") == (
        "RI", "NE", "CT", "KY", "SC", "WI", "AZ", "CA"
    )
    assert scheduled_states("full") == FULL_STATES


def test_command_template_binds_state_and_controls() -> None:
    command = command_template("NE", "prime-factor")
    assert command[command.index("--state") + 1] == "NE"
    assert command[command.index("--structure") + 1] == "prime-factor"
    assert command[command.index("--partition-mode") + 1] == "edge-weighted"
    assert command[command.index("--weights-override") + 1] == "geographic"
    assert command[command.index("--search") + 1] == "single"
    assert command[command.index("--seed") + 1] == "0"


def test_hungarian_matches_bruteforce_on_small_matrix() -> None:
    weights = [[9, 2, 7], [6, 4, 3], [5, 8, 1]]
    brute = max(
        sum(weights[row][column] for row, column in enumerate(permutation))
        for permutation in itertools.permutations(range(3))
    )
    assert maximum_weight_assignment(weights) == brute


def test_hungarian_handles_52_district_identity_without_exponential_dp() -> None:
    weights = [[100 if row == column else 0 for column in range(52)] for row in range(52)]
    assert maximum_weight_assignment(weights) == 5200


def test_optimal_overlap_ignores_labels_and_rejects_universe_mismatch() -> None:
    left = canonical_assignment({"4": 7, "3": 7, "2": 8, "1": 8})
    right = {"1": 20, "2": 20, "3": 10, "4": 10}
    assert optimal_overlap(left, right)["matched_unit_rate"] == 1.0
    with pytest.raises(BakeoffError, match="assignment universes differ"):
        optimal_overlap({"1": 1}, {"2": 1})


def test_edge_cut_normalization_ignores_sub_precision_sum_order() -> None:
    assert normalize_edge_cut(2_557_787.746167) == 2_557_787.74617
    assert normalize_edge_cut(2_557_787.746166) == 2_557_787.74617


def test_resume_preserves_recorded_cells(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    package = tmp_path / "package"
    recorded = package / "states" / "ri" / "structures" / "standard-bisect" / "run.json"
    recorded.parent.mkdir(parents=True)
    recorded.write_text("{}", encoding="utf-8")
    binary = tmp_path / "bisect.exe"
    binary.write_bytes(b"test")
    calls = []
    monkeypatch.setattr(
        national_runner,
        "run_cell",
        lambda _package, _binary, state, structure: calls.append((state, structure)),
    )
    monkeypatch.setattr(
        national_runner, "write_derived", lambda _package, _phase: {"status": "pass"}
    )
    monkeypatch.setattr(national_runner, "build_manifest", lambda *_args: None)

    national_runner.run(package, binary, "pilot", force=False, resume=True)

    assert ("RI", "standard-bisect") not in calls
    assert len(calls) == len(PILOT_STATES) * 4 - 1
