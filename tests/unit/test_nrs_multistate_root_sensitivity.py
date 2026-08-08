from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_multistate_root_sensitivity import STATE_CONFIG, state_paths


def test_frozen_state_sample_covers_expected_root_schedules() -> None:
    assert sorted(STATE_CONFIG) == ["GA", "NH", "NM"]
    assert {tuple(config["child_seats"]) for config in STATE_CONFIG.values()} == {
        (1, 1),
        (1, 2),
        (7, 7),
    }
    assert sum(config["unit_count"] for config in STATE_CONFIG.values()) == 371_880


def test_frozen_population_tolerances_follow_engine_formula() -> None:
    for config in STATE_CONFIG.values():
        expected = (
            5 * min(config["child_seats"]) * config["population"] + 999
        ) // 1_000
        assert config["population_tolerance_scaled"] == expected


def test_state_paths_use_governed_2020_packages() -> None:
    binary = PROJECT_ROOT / "target" / "release" / "bisect.exe"
    paths = state_paths("NM", binary)

    assert paths["context"].as_posix().endswith(
        "data/2020/certified/nm_blocks_2020.rctx"
    )
    assert paths["benchmark_discovery"].as_posix().endswith(
        "states/nm/package/nodes/root/certified-discovery.json"
    )
