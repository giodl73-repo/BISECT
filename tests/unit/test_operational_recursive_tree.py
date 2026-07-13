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
