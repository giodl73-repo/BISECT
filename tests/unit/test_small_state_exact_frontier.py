from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path

import pytest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/analyze_small_state_exact_frontier.py"
SPEC = importlib.util.spec_from_file_location("small_state_exact_frontier", MODULE_PATH)
assert SPEC and SPEC.loader
FRONTIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FRONTIER)
REPORT_PATH = (
    ROOT / "docs/experiments/exact-canonical/ri-2020-block-frontier.json"
)


def test_committed_small_state_blocker_report_verifies() -> None:
    report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))
    FRONTIER.verify_report(report, check_sources=False)
    assert report["status"] == "blocked"
    assert report["observed_instance"]["tiger_block_rows"] == 25_649
    assert report["exact_reference"]["candidate_decimal_digits"] == 7_721


def test_small_state_blocker_rejects_search_scale_tamper() -> None:
    report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))
    tampered = copy.deepcopy(report)
    tampered["exact_reference"]["candidate_formula"] = "2^23-1"
    with pytest.raises(SystemExit, match="candidate formula mismatch"):
        FRONTIER.verify_report(tampered, check_sources=False)
