from pathlib import Path
import sys

import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_bakeoff_geometry_national import (
    canonical_output_dir,
    metric_summary,
    projection_for_state,
)


def test_projection_selection_matches_frozen_regions() -> None:
    assert projection_for_state("AK") == "EPSG:3338"
    assert projection_for_state("HI") == "EPSG:3759"
    assert projection_for_state("RI") == "EPSG:5070"


def test_explicit_display_path_supports_temporary_regeneration(tmp_path) -> None:
    assert (
        canonical_output_dir(tmp_path / "package", "docs/experiments/package")
        == "docs/experiments/package"
    )


def test_metric_summary_separates_state_and_district_estimands() -> None:
    state_rows = [
        {"nrs_reock": 0.2, "comparator_reock": 0.4},
        {"nrs_reock": 0.6, "comparator_reock": 0.5},
    ]
    district_rows = [
        {"plan_family": "NRS-v0.3", "reock": 0.2},
        {"plan_family": "NRS-v0.3", "reock": 0.4},
        {"plan_family": "NRS-v0.3", "reock": 0.8},
        {"plan_family": "enacted-congressional-session-118", "reock": 0.1},
        {"plan_family": "enacted-congressional-session-118", "reock": 0.5},
        {"plan_family": "enacted-congressional-session-118", "reock": 0.6},
    ]
    for metric in ("polsby_popper", "convex_hull_ratio", "schwartzberg"):
        for row in state_rows:
            row[f"nrs_{metric}"] = row["nrs_reock"]
            row[f"comparator_{metric}"] = row["comparator_reock"]
        for row in district_rows:
            row[metric] = row["reock"]

    result = metric_summary(state_rows, district_rows)["reock"]

    assert result["state_weighted"]["nrs"] == 0.4
    assert result["district_weighted"]["nrs"] == pytest.approx(1.4 / 3.0)
    assert result["state_weighted"]["comparator_minus_nrs"] == pytest.approx(0.05)
