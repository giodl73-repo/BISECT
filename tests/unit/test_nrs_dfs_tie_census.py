from pathlib import Path
import sys

import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_dfs_tie_census import CensusError, distribution, method_counter


def test_method_counter_reads_instrumented_fields() -> None:
    method = (
        "prefix; initial-dfs-minimum-deviation-candidates=4; "
        "initial-dfs-minimum-deviation-cut-candidates=2; suffix"
    )

    assert method_counter(method, "initial-dfs-minimum-deviation-candidates") == 4
    assert method_counter(method, "initial-dfs-minimum-deviation-cut-candidates") == 2


def test_method_counter_rejects_missing_field() -> None:
    with pytest.raises(CensusError, match="missing method counter"):
        method_counter("prefix", "missing")


def test_distribution_preserves_integer_counts() -> None:
    assert distribution([1, 1, 2, 4]) == {
        "minimum": 1,
        "maximum": 4,
        "counts": {"1": 2, "2": 1, "4": 1},
    }
