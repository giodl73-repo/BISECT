from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/build_ri_county_branches.py"
SPEC = importlib.util.spec_from_file_location("ri_county_branches", MODULE_PATH)
assert SPEC and SPEC.loader
BRANCHES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BRANCHES)


def test_branch_contract_is_five_way_and_ordered() -> None:
    instance = {
        "unit_ids": [
            "440010000000001",
            "440030000000001",
            "440050000000001",
            "440070000000001",
            "440090000000001",
        ]
    }
    branches = BRANCHES.branch_definitions(instance)
    assert [branch["branch_id"] for branch in branches] == [
        "outside-zero",
        "first-positive-001",
        "first-positive-003",
        "first-positive-005",
        "first-positive-009",
    ]
    assert len(branches[0]["constraints"]) == 4
    assert [len(branch["constraints"]) for branch in branches[1:]] == [1, 2, 3, 4]
