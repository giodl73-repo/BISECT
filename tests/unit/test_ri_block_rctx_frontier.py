from __future__ import annotations

import importlib.util
import json
from pathlib import Path

import pytest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/build_ri_block_rctx.py"
SPEC = importlib.util.spec_from_file_location("ri_block_rctx_frontier", MODULE_PATH)
assert SPEC and SPEC.loader
FRONTIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FRONTIER)
MANIFEST = ROOT / "docs/experiments/certified-recursive/manifest.json"


def test_committed_ri_block_rctx_frontier_verifies_without_local_data() -> None:
    FRONTIER.verify(MANIFEST, check_rctx=False)
    report = json.loads(
        (MANIFEST.parent / "ri-2020-root-frontier.json").read_text(encoding="utf-8")
    )
    assert report["graph"]["unit_count"] == 25_649
    assert report["graph"]["land_edge_count"] == 66_097
    assert report["graph"]["bridge_edge_count"] == 64
    assert report["graph"]["final_component_count"] == 1
    assert [row["unit_count"] for row in report["graph"]["land_components"]] == [
        25_585,
        64,
    ]


def test_ri_frontier_rejects_report_hash_tamper(tmp_path: Path) -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    manifest["files"][0]["sha256"] = "0" * 64
    report_name = manifest["files"][0]["path"]
    (tmp_path / report_name).write_bytes((MANIFEST.parent / report_name).read_bytes())
    tampered = tmp_path / "manifest.json"
    tampered.write_text(json.dumps(manifest), encoding="utf-8")
    with pytest.raises(SystemExit, match="report hash mismatch"):
        FRONTIER.verify(tampered, check_rctx=False)
