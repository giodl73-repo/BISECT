from pathlib import Path
import sys

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from check_block_ensemble_host_capacity import GIB, capacity_report


def test_capacity_pass_reserves_all_three_components() -> None:
    report = capacity_report(
        free_bytes=8 * GIB,
        retained_used_bytes=0,
    )

    assert report["required_free_bytes"] == 8 * GIB
    assert report["status"] == "pass"
    assert report["process_launch_authorized"] is True


def test_capacity_reject_reports_exact_shortfall() -> None:
    report = capacity_report(
        free_bytes=7 * GIB,
        retained_used_bytes=0,
    )

    assert report["status"] == "reject"
    assert report["shortfall_bytes"] == GIB
    assert report["process_launch_authorized"] is False


def test_retained_custody_reduces_only_remaining_reservation() -> None:
    report = capacity_report(
        free_bytes=6 * GIB,
        retained_used_bytes=2 * GIB,
    )

    assert report["retained_remaining_bytes"] == GIB
    assert report["required_free_bytes"] == 6 * GIB
    assert report["status"] == "pass"


def test_capacity_rejects_invalid_existing_custody() -> None:
    with pytest.raises(ValueError, match="already exceeds"):
        capacity_report(
            free_bytes=20 * GIB,
            retained_used_bytes=4 * GIB,
        )
