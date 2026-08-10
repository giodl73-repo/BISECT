from __future__ import annotations

import importlib.util
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/analyze_ri_certified_discovery.py"
SPEC = importlib.util.spec_from_file_location("ri_certified_discovery", MODULE_PATH)
assert SPEC and SPEC.loader
DISCOVERY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DISCOVERY)
MANIFEST = ROOT / "docs/experiments/scalable-certified/manifest.json"


def test_committed_ri_discovery_frontier_verifies() -> None:
    DISCOVERY.verify(MANIFEST, check_local=False)
    report = json.loads(
        (MANIFEST.parent / "ri-discovery-frontier.json").read_text(encoding="utf-8")
    )
    assert report["candidate"]["connected"] == [True, True]
    assert report["candidate"]["max_population_deviation_scaled"] == 1
    assert report["candidate"]["weighted_boundary_cut"] == 43_047_238
    assert report["certification_status"]["proof"] == "not-generated"


def test_analyzer_hash_is_line_ending_stable(tmp_path: Path) -> None:
    lf = tmp_path / "lf.py"
    crlf = tmp_path / "crlf.py"
    lf.write_bytes(b"first\nsecond\n")
    crlf.write_bytes(b"first\r\nsecond\r\n")
    assert DISCOVERY.text_sha256(lf) == DISCOVERY.text_sha256(crlf)
