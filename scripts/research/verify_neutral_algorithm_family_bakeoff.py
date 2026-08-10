#!/usr/bin/env python3
"""Verify and exactly regenerate the neutral algorithm-family proof slice."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from analyze_neutral_algorithm_family_bakeoff import (  # noqa: E402
    ANALYSIS_VERSION,
    EXPECTED_MODES,
    PROTOCOL_ID,
    STRUCTURES,
    canonical_assignment,
)
from run_neutral_algorithm_family_bakeoff import MANIFEST_VERSION  # noqa: E402


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    package = args.package.resolve()
    manifest = read_json(package / "manifest.json")
    if manifest.get("schema_version") != MANIFEST_VERSION:
        fail("unsupported manifest schema")
    if manifest.get("protocol_id") != PROTOCOL_ID or manifest.get("status") not in {"pass", "fail"}:
        fail("protocol or package status")
    analysis = read_json(package / "analysis.json")
    if (
        analysis.get("schema_version") != ANALYSIS_VERSION
        or analysis.get("status") != manifest.get("status")
    ):
        fail("analysis schema or status")

    binary = ROOT / manifest["binary"]["path"]
    if not binary.is_file() or sha256(binary) != manifest["binary"]["sha256"]:
        fail("binary hash mismatch")
    for row in manifest["code"]:
        path = ROOT / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"code hash mismatch for {row['path']}")
    for row in manifest["native_artifacts"]:
        path = package / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"native artifact hash mismatch for {row['path']}")
    for relative, expected in manifest["outputs"].items():
        path = package / relative
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash mismatch for {relative}")

    for structure in STRUCTURES:
        structure_dir = package / "structures" / structure
        run = read_json(structure_dir / "run.json")
        if run.get("status") != "pass" or run.get("requested_seed") != 0:
            fail(f"{structure} run status or requested seed")
        if run.get("structure") != structure or structure not in run.get("command", []):
            fail(f"{structure} command provenance")
        native_root = structure_dir / "native" / "2020" / "states" / "wisconsin"
        native = read_json(native_root / "manifest.json")
        if native.get("partition_mode") != EXPECTED_MODES[structure]:
            fail(f"{structure} effective mode")
        if native.get("binary_sha256") != manifest["binary"]["sha256"]:
            fail(f"{structure} native binary hash")
        audit = read_json(native_root / "audit-certificate.json")
        if native.get("audit_result") != audit.get("result"):
            fail(f"{structure} native audit-result inconsistency")
        raw = read_json(native_root / "data" / "final_assignments.json")
        expected_assignment = canonical_assignment(raw)
        if read_json(structure_dir / "canonical_assignments.json") != expected_assignment:
            fail(f"{structure} canonical assignment")

    target_temp = ROOT / "target" / "research-regeneration"
    target_temp.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=target_temp) as temp_dir:
        regenerated = Path(temp_dir) / "package"
        command = [
            sys.executable,
            str(ROOT / "scripts" / "research" / "run_neutral_algorithm_family_bakeoff.py"),
            "--output-dir", str(regenerated),
            "--binary", str(binary),
        ]
        completed = subprocess.run(command, cwd=ROOT, check=False)
        expected_returncode = 0 if manifest["status"] == "pass" else 1
        if completed.returncode != expected_returncode:
            fail(
                f"regeneration command returned {completed.returncode}, "
                f"expected {expected_returncode}"
            )
        deterministic = list(manifest["outputs"])
        for relative in deterministic:
            regenerated_path = regenerated / relative
            published_path = package / relative
            if not regenerated_path.is_file():
                fail(f"regeneration omitted {relative}")
            if regenerated_path.read_bytes() != published_path.read_bytes():
                fail(f"regenerated {relative} differs")
        regenerated_analysis = read_json(regenerated / "analysis.json")
        if regenerated_analysis != analysis:
            fail("regenerated analysis differs semantically")

    print(
        "neutral algorithm-family bakeoff evidence verification: PASS "
        f"(experiment status: {manifest['status'].upper()})"
    )


if __name__ == "__main__":
    main()
