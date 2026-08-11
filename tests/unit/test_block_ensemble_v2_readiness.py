import json
from pathlib import Path
import sys

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

import verify_block_ensemble_v2_readiness as readiness


def test_official_readiness_package_passes() -> None:
    record = readiness.verify_readiness()

    assert record["status"] == "pass"
    assert record["capacity_snapshot"]["required_free_bytes"] == 8 * 1024**3


def test_capacity_snapshot_is_not_reusable_admission() -> None:
    record = json.loads((readiness.PACKAGE / "readiness.json").read_text())

    assert "every process still requires a fresh" in record["capacity_snapshot"][
        "claim_boundary"
    ]
    assert "does not authorize a process" in record["claim_boundary"]


def test_readiness_rejects_tampered_input_audit(tmp_path: Path) -> None:
    package = tmp_path / "package"
    package.mkdir()
    for name in (
        "readiness.json",
        "ledger.json",
        "input-audit-nh.json",
        "input-audit-nm.json",
        "input-audit-ga.json",
    ):
        (package / name).write_bytes((readiness.PACKAGE / name).read_bytes())
    audit = json.loads((package / "input-audit-nh.json").read_text())
    audit["units"] += 1
    (package / "input-audit-nh.json").write_text(
        json.dumps(audit), encoding="utf-8"
    )

    with pytest.raises(ValueError, match="NH input audit units drift"):
        readiness.verify_readiness(package)


def test_readiness_rejects_binary_binding_drift(monkeypatch) -> None:
    original = readiness.sha256

    def changed(path: Path) -> str:
        if path.name == "block_trace.exe":
            return "0" * 64
        return original(path)

    monkeypatch.setattr(readiness, "sha256", changed)
    with pytest.raises(ValueError, match="block_trace.exe"):
        readiness.verify_readiness()


def test_readiness_rejects_process_artifact(tmp_path: Path) -> None:
    package = tmp_path / "package"
    package.mkdir()
    (package / "ledger.json").write_bytes(
        (readiness.PACKAGE / "ledger.json").read_bytes()
    )
    (package / "admission-preflight-nh-wilson-attempt-01.json").write_text("{}")

    with pytest.raises(ValueError, match="process artifacts exist"):
        readiness.verify_empty_package(package)
