from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/run_reduced_band_milp.py"
SPEC = importlib.util.spec_from_file_location("reduced_band_milp", MODULE_PATH)
assert SPEC and SPEC.loader
MILP = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MILP)


def test_fixed_core_components_are_excluded_from_cuts() -> None:
    graph = [[1], [0, 2], [1, 3], [2, 4], [3]]
    fixed = [0, None, None, None, 1]
    cuts = MILP.separate(graph, [0, 1, 0, 1, 1], fixed)
    assert {(label, tuple(component)) for label, component, _ in cuts} == {
        (0, (2,)),
        (1, (1,)),
    }
