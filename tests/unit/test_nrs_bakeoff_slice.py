from pathlib import Path
import json
import sys

import pandas as pd
import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from analyze_nrs_bakeoff_slice import (
    BakeoffError,
    optimal_overlap,
    select_land_blocks,
    select_numbered_districts,
    summarize_splits,
    validate_comparator_session,
)


def test_optimal_overlap_ignores_district_label_permutation() -> None:
    benchmark = {
        "440010001001000": 1,
        "440010001001001": 1,
        "440030002001000": 2,
        "440030002001001": 2,
    }
    comparator = {
        "440010001001000": 8,
        "440010001001001": 8,
        "440030002001000": 7,
        "440030002001001": 7,
    }

    result = optimal_overlap(benchmark, comparator)

    assert result["matched_blocks"] == 4
    assert result["moved_blocks"] == 0
    assert result["matched_block_rate"] == 1.0


def test_optimal_overlap_rejects_district_count_mismatch() -> None:
    fixture = json.loads(
        (
            PROJECT_ROOT
            / "docs"
            / "fixtures"
            / "nrs-bakeoff"
            / "district-count-mismatch"
            / "expected-failure.json"
        ).read_text(encoding="utf-8")
    )

    with pytest.raises(BakeoffError) as excinfo:
        optimal_overlap(fixture["benchmark"], fixture["comparator"])
    assert str(excinfo.value) == fixture["expected_error"]


def test_split_summary_reports_units_and_excess_pieces() -> None:
    assignments = {
        "440010001001000": 1,
        "440010001001001": 2,
        "440030002001000": 2,
    }

    assert summarize_splits(assignments, 5) == {
        "units": 2,
        "split_units": 1,
        "excess_pieces": 1,
    }


def test_comparator_session_identity_is_enforced() -> None:
    with pytest.raises(
        BakeoffError,
        match=r"comparator-session-mismatch expected=118 actual=\['116'\]",
    ):
        validate_comparator_session(["116", "116"], "118")


def test_water_only_blocks_are_excluded_uniformly() -> None:
    blocks = pd.DataFrame(
        {
            "GEOID20": ["land", "water"],
            "ALAND20": [1, 0],
        }
    )

    selected, excluded = select_land_blocks(blocks)

    assert selected["GEOID20"].tolist() == ["land"]
    assert excluded == 1


def test_invalid_land_area_is_rejected() -> None:
    blocks = pd.DataFrame({"GEOID20": ["bad"], "ALAND20": ["unknown"]})

    with pytest.raises(BakeoffError, match="block ALAND20 contains invalid values"):
        select_land_blocks(blocks)


def test_non_district_comparator_polygons_are_excluded() -> None:
    comparator = pd.DataFrame({"CD118FP": ["01", "02", "ZZ"]})

    selected, excluded = select_numbered_districts(comparator, "CD118FP")

    assert selected["CD118FP"].tolist() == ["01", "02"]
    assert excluded == ["ZZ"]
