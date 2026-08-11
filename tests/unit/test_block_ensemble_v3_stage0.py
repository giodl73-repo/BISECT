from pathlib import Path
import shutil
import sys

import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_block_ensemble_expansion_v3 import ORDER, PACKAGE, new_ledger
from verify_block_ensemble_v3_stage0 import verify_stage0


def test_retained_v3_stage0_verifies_portably(tmp_path: Path) -> None:
    package = tmp_path / "stage0"
    shutil.copytree(PACKAGE, package)

    summary = verify_stage0(package)

    assert summary["preflights"] == 6
    assert summary["preflight_replays"] == 6
    assert summary["retained_bytes"] == 3_386_273
    assert summary["peak_rss_bytes"] <= 2_415_919_104


def test_stage0_requires_all_preflight_replays(monkeypatch, tmp_path: Path) -> None:
    ledger = new_ledger()
    ledger["completed"]["preflight"] = list(ORDER)
    monkeypatch.setattr(
        "verify_block_ensemble_v3_stage0.verify_package", lambda package: ledger
    )

    with pytest.raises(ValueError, match="preflight replay schedule is incomplete"):
        verify_stage0(tmp_path)
