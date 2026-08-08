from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_fallback_candidate_census import TARGETS, stage_metrics


def test_target_universe_has_eight_stage_node_pairs() -> None:
    assert sum(
        len(stages)
        for nodes in TARGETS.values()
        for stages in nodes.values()
    ) == 8


def test_stage_metrics_reads_ordered_counts() -> None:
    method = (
        "prefix; nrs-v0-2-fallback-evaluated-candidates=16; "
        "nrs-v0-2-fallback-minimum-deviation-candidates=3; "
        "nrs-v0-2-fallback-minimum-deviation-cut-candidates=2; "
        "nrs-v0-2-fallback-minimum-deviation-cut-partitions=1; suffix"
    )

    assert stage_metrics(method, "v0.2") == {
        "evaluated_candidates": 16,
        "minimum_deviation_candidates": 3,
        "minimum_deviation_cut_candidates": 2,
        "minimum_deviation_cut_partitions": 1,
        "physical_partition_opportunity": False,
    }
