from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/build_ri_tract_purity_branch.py"
SPEC = importlib.util.spec_from_file_location("ri_tract_purity", MODULE_PATH)
assert SPEC and SPEC.loader
PURITY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PURITY)


def test_pure_tracts_fix_positive_units_only() -> None:
    instance = {
        "unit_ids": [
            "440070001001001",
            "440070001001002",
            "440070002001001",
            "440070002001002",
            "440010001001001",
        ],
        "populations": [10, 0, 4, 5, 3],
    }
    fixed, report = PURITY.build_fixed_labels(instance, [1, 0, 0, 1, 0])
    assert fixed == [1, None, None, None, 0]
    assert report["pure_tract_count"] == 1
    assert report["split_tract_count"] == 1
