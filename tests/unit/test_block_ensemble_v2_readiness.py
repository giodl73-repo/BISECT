import json
from pathlib import Path
import shutil
import sys

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

import verify_block_ensemble_v2_readiness as readiness


PRISTINE_LEDGER = {
    "schema_version": "nrs-block-ensemble-expansion-ledger-v2",
    "protocol_id": "nrs-v0.3-block-ensemble-expansion-v2",
    "status": "active",
    "completed": {
        "preflight": [],
        "preflight-replay": [],
        "primary": [],
        "replay": [],
    },
    "runner_wall_seconds": 0.0,
    "retained_bytes": 0,
    "failures": [],
}


def write_pristine_ledger(package: Path) -> None:
    (package / "ledger.json").write_text(
        json.dumps(PRISTINE_LEDGER), encoding="utf-8"
    )


@pytest.fixture
def synthetic_readiness(tmp_path: Path, monkeypatch) -> Path:
    root = tmp_path / "root"
    package = root / "docs/experiments/nrs-v0.3-block-ensemble-expansion-v2"
    package.mkdir(parents=True)
    for name in (
        "readiness.json",
        "input-audit-nh.json",
        "input-audit-nm.json",
        "input-audit-ga.json",
    ):
        shutil.copyfile(readiness.PACKAGE / name, package / name)
    write_pristine_ledger(package)

    record = json.loads((package / "readiness.json").read_text())
    hashes: dict[Path, str] = {}
    for expected in readiness.INPUTS.values():
        audit_path = package / f"input-audit-{expected['slug']}.json"
        audit = json.loads(audit_path.read_text())
        rctx = root / f"data/2020/certified/{expected['slug']}_blocks_2020.rctx"
        assignments = (
            root
            / "runs/nrs-v0.3/neutral-analysis/national-2020/states"
            / expected["slug"]
            / "package/baseline_assignments.json"
        )
        for path, digest in (
            (rctx, audit["rctx_sha256"]),
            (assignments, audit["assignments_sha256"]),
        ):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"fixture")
            hashes[path] = digest

    bound_paths = {
        "input-audit-nh.json": package / "input-audit-nh.json",
        "input-audit-nm.json": package / "input-audit-nm.json",
        "input-audit-ga.json": package / "input-audit-ga.json",
        "block_trace.exe": root / "target/release/examples/block_trace.exe",
        "validate_block_input.exe": (
            root / "target/release/examples/validate_block_input.exe"
        ),
        "block_trace.rs": root / "crates/bisect-ensemble/examples/block_trace.rs",
        "validate_block_input.rs": (
            root / "crates/bisect-ensemble/examples/validate_block_input.rs"
        ),
        "run_block_ensemble_expansion_v2.py": (
            root / "scripts/research/run_block_ensemble_expansion_v2.py"
        ),
        "verify_block_ensemble_expansion_v2.py": (
            root / "scripts/research/verify_block_ensemble_expansion_v2.py"
        ),
        "check_block_ensemble_host_capacity.py": (
            root / "scripts/research/check_block_ensemble_host_capacity.py"
        ),
        "expansion-v2-protocol.md": (
            root / "docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v2.md"
        ),
        "resource-audit-manifest.json": (
            root
            / "docs/experiments/nrs-v0.3-block-ensemble-resource-audit/manifest.json"
        ),
    }
    for name, path in bound_paths.items():
        if not path.exists():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"fixture")
        hashes[path] = record["sha256_bindings"][name]

    monkeypatch.setattr(readiness, "ROOT", root)
    monkeypatch.setattr(readiness, "sha256", lambda path: hashes[path])
    return package


def test_synthetic_readiness_package_passes(synthetic_readiness: Path) -> None:
    record = readiness.verify_readiness(synthetic_readiness)

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
        "input-audit-nh.json",
        "input-audit-nm.json",
        "input-audit-ga.json",
    ):
        (package / name).write_bytes((readiness.PACKAGE / name).read_bytes())
    write_pristine_ledger(package)
    audit = json.loads((package / "input-audit-nh.json").read_text())
    audit["units"] += 1
    (package / "input-audit-nh.json").write_text(
        json.dumps(audit), encoding="utf-8"
    )

    with pytest.raises(ValueError, match="NH input audit units drift"):
        readiness.verify_readiness(package)


def test_readiness_rejects_binary_binding_drift(
    synthetic_readiness: Path, monkeypatch
) -> None:
    bound_sha256 = readiness.sha256

    def changed(path: Path) -> str:
        if path.name == "block_trace.exe":
            return "0" * 64
        return bound_sha256(path)

    monkeypatch.setattr(readiness, "sha256", changed)
    with pytest.raises(ValueError, match="block_trace.exe"):
        readiness.verify_readiness(synthetic_readiness)


def test_readiness_rejects_process_artifact(tmp_path: Path) -> None:
    package = tmp_path / "package"
    package.mkdir()
    write_pristine_ledger(package)
    (package / "admission-preflight-nh-wilson-attempt-01.json").write_text("{}")

    with pytest.raises(ValueError, match="process artifacts exist"):
        readiness.verify_empty_package(package)
