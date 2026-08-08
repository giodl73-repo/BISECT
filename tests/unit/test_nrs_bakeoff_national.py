from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_bakeoff_national import projection_for_state, weighted_mean


def test_projection_selection_is_state_specific() -> None:
    assert projection_for_state("AK") == "EPSG:3338"
    assert projection_for_state("HI") == "EPSG:3759"
    assert projection_for_state("RI") == "EPSG:5070"


def test_weighted_mean_uses_requested_weight() -> None:
    rows = [
        {"rate": 0.25, "districts": 1},
        {"rate": 0.75, "districts": 3},
    ]

    assert weighted_mean(rows, "rate", "districts") == 0.625
