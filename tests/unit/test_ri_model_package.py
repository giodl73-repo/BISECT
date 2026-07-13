from __future__ import annotations

import importlib.util
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/analyze_ri_model_package.py"
SPEC = importlib.util.spec_from_file_location("ri_model_package", MODULE_PATH)
assert SPEC and SPEC.loader
MODEL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODEL)
MANIFEST = ROOT / "docs/experiments/scalable-certified/model-manifest.json"


def test_committed_ri_model_frontier_verifies() -> None:
    MODEL.verify(MANIFEST, check_local=False)
    report = json.loads(
        (MANIFEST.parent / "ri-model-frontier.json").read_text(encoding="utf-8")
    )
    assert report["population_stage"]["status"] == "verified-unsat"
    assert report["boundary_stage"]["status"] == "not-run"
    assert report["artifacts"]["01-population"]["variable_count"] == 1_228_520
    assert report["artifacts"]["03-canonical"]["variable_count"] == 1_264_715
    assert len(report["strengthened_boundary_branches"]) == 2
    assert {
        branch["connectivity_encoding"]
        for branch in report["strengthened_boundary_branches"]
    } == {"parent-depth-v3"}
