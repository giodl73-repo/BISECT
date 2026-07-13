from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/build_operational_recursive_tree.py"
SPEC = importlib.util.spec_from_file_location("operational_tree", MODULE_PATH)
assert SPEC and SPEC.loader
TREE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(TREE)


def test_ratio_arithmetic_population_floor() -> None:
    assert TREE.ratio_arithmetic_floor(100, 4, 2) == 0
    assert TREE.ratio_arithmetic_floor(101, 4, 2) == 2
    assert TREE.ratio_arithmetic_floor(101, 2, 1) == 1
    assert TREE.ratio_arithmetic_floor(100, 3, 2) == 1


def test_floor_discovery_screens_before_refining(monkeypatch, tmp_path: Path) -> None:
    calls: list[tuple[int, str]] = []
    screened_deviations = {1: 9, 2: 5, 3: 7}

    def fake_run_discovery(
        _bisect: Path,
        _context_path: Path,
        _districts: int,
        out_dir: Path,
        seed: int,
        refinement: str = "population",
        timeout_seconds: int | None = None,
    ) -> dict:
        assert timeout_seconds in (None, TREE.SCREEN_TIMEOUT_SECONDS)
        calls.append((seed, refinement))
        out_dir.mkdir(parents=True)
        deviation = (
            screened_deviations[seed]
            if refinement == "metis"
            else {1: 1, 2: 4, 3: 3}[seed]
        )
        return {
            "objective": {
                "primary": {
                    "max_population_deviation_scaled": deviation,
                    "total_population_deviation_scaled": deviation * 2,
                    "weighted_boundary_cut": seed,
                }
            }
        }

    monkeypatch.setattr(TREE, "run_discovery", fake_run_discovery)
    discovery, seed, screen_report = TREE.run_floor_discovery(
        tmp_path / "bisect",
        tmp_path / "context.rctx",
        4,
        tmp_path / "root",
        preferred_seed=1,
        population_floor=1,
        max_seed=3,
    )

    assert seed == 1
    assert discovery["objective"]["primary"]["max_population_deviation_scaled"] == 1
    assert [row["seed"] for row in screen_report] == [1, 2, 3]
    assert calls == [
        (1, "metis"),
        (2, "metis"),
        (3, "metis"),
        (2, "population"),
        (3, "population"),
        (1, "population"),
    ]


def test_floor_discovery_reuses_completed_node(tmp_path: Path) -> None:
    out_dir = tmp_path / "root"
    out_dir.mkdir()
    discovery = {
        "method": "standard-bisect-discovery; seed=7; refinement=population",
        "objective": {
            "primary": {
                "max_population_deviation_scaled": 1,
                "total_population_deviation_scaled": 2,
                "weighted_boundary_cut": 3,
            }
        },
    }
    (out_dir / "certified-discovery.json").write_text(
        __import__("json").dumps(discovery), encoding="utf-8"
    )

    reused, seed, report = TREE.run_floor_discovery(
        tmp_path / "bisect",
        tmp_path / "context.rctx",
        2,
        out_dir,
        preferred_seed=1,
        population_floor=1,
        max_seed=16,
    )

    assert reused == discovery
    assert seed == 7
    assert report[0]["status"] == "selected-node-reused"
