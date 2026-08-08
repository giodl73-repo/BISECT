from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_dfs_partition_census import tie
from verify_nrs_dfs_partition_census import count_distribution


def test_partition_counter_reads_instrumented_field() -> None:
    method = (
        "prefix; initial-dfs-minimum-deviation-cut-candidates=2; "
        "initial-dfs-minimum-deviation-cut-partitions=1; suffix"
    )

    assert (
        tie.method_counter(
            method, "initial-dfs-minimum-deviation-cut-partitions"
        )
        == 1
    )


def test_count_distribution_preserves_sorted_integer_counts() -> None:
    assert count_distribution([2, 1, 2, 4]) == {"1": 1, "2": 2, "4": 1}
