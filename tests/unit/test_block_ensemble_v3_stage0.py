from pathlib import Path
import json
import shutil
import sys

import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_block_ensemble_expansion_v3 import ORDER, PACKAGE, new_ledger
from verify_block_ensemble_expansion_v3 import verify_package
from verify_block_ensemble_v3_stage0 import verify_stage0


def test_retained_v3_package_verifies_portably(tmp_path: Path) -> None:
    package = tmp_path / "stage0"
    shutil.copytree(PACKAGE, package)

    ledger = verify_package(package)

    assert ledger["completed"]["preflight"] == ORDER
    assert ledger["completed"]["preflight-replay"] == ORDER
    assert ledger["completed"]["primary"] == [
        "NH:wilson",
        "NH:kruskal",
        "NM:wilson",
        "NM:kruskal",
        "GA:wilson",
        "GA:kruskal",
    ]
    assert ledger["completed"]["replay"] == [
        "NH:wilson",
        "NH:kruskal",
        "NM:wilson",
    ]
    assert ledger["retained_bytes"] == 20_664_538


def test_stage0_requires_all_preflight_replays(monkeypatch, tmp_path: Path) -> None:
    ledger = new_ledger()
    ledger["completed"]["preflight"] = list(ORDER)
    monkeypatch.setattr(
        "verify_block_ensemble_v3_stage0.verify_package", lambda package: ledger
    )

    with pytest.raises(ValueError, match="preflight replay schedule is incomplete"):
        verify_stage0(tmp_path)


def test_governed_resource_requires_exact_execution_binding(tmp_path: Path) -> None:
    package = tmp_path / "stage0"
    shutil.copytree(PACKAGE, package)
    binding_path = package / "stage0-execution-bindings.json"
    bindings = json.loads(binding_path.read_text(encoding="utf-8"))
    bindings["governed_platform_exact_sha256_bindings"].clear()
    binding_path.write_text(json.dumps(bindings), encoding="utf-8")

    with pytest.raises(ValueError, match="has no governed execution text binding"):
        verify_package(package)
